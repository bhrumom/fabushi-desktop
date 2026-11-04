use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, atomic::{AtomicBool, Ordering}};

pub use super::legacy_telegram_retirement::{
    LegacyTelegramRetirementError, LegacyTelegramRetirementReport, RetiredTelegramMarker,
    TelegramKeyClearHook,
};
use super::legacy_telegram_retirement::retire_legacy_telegram_data_internal;

use sha2::{Digest, Sha256};
use uuid::Uuid;

use crate::agent_isolation::{
    AgentWorkerPool, ProductionAgentStoreWorkerBackend,
    create_production_agent_store_worker_backend,
};
use crate::agents::agent_profile::SandAgentProfile;
use crate::storage::agent_paths::get_sand_agents_root_dir;

use super::agent_db::{
    AgentDbSerdeSnapshot, SandAgentDb, add_persisted_conversation_partner,
    append_persisted_transcript_entries, clear_persisted_agent_profile_prompt_snapshot,
    clear_persisted_memory_prompt_snapshot,
    clear_persisted_transient_state, delete_persisted_transcript_entry,
    mark_persisted_activity, mark_persisted_read, mark_persisted_unread,
    mark_persisted_viewed, read_persisted_agent_profile_prompt_snapshot,
    read_persisted_automation_spend_guard_state,
    read_persisted_conversation_partner_ids, read_persisted_introduction_pending,
    read_persisted_newest_divider_anchor_timestamp_ms,
    record_persisted_episode_turn, record_persisted_request_id,
    set_persisted_agent_profile_prompt_snapshot, set_persisted_automation_spend_guard_state,
    set_persisted_introduction_pending, set_persisted_memory_prompt_snapshot,
    update_persisted_transcript_entry,
};
use super::forward_batch_journal::{
    HumanForwardBatch, load_forward_batches, read_forward_attachment_bytes,
    snapshot_forward_attachments, store_forward_batch,
};
use super::agent_db_transcript_pages::{
    HumanTranscriptPage, HumanTranscriptPageQuery,
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
};
use super::call_session::{CallSession, CallSessionStore, CallSignal, transition_target};
use super::agent_db_serde::{
    AwaitingUserResponse, EpisodeTurn, MemoryPromptSnapshot, SandProfile, SpendGuardState,
    UnreadState,
};
use super::session_conversation_state::{
    ConversationOutlineItem, ResolvedConversationState, SessionConversationState, TranscriptThread,
};
use super::conversation_blobs_path::conversation_blobs_path;
use super::conversation_size_limits::{
    ConversationGcTarget, ConversationSizeMaintenance, ConversationSizePolicy,
};
use super::session_paths::{
    get_agent_db_path, get_connector_secrets_root, get_native_conversation_db_path,
    get_native_conversations_root,
};
use super::connector_secret_store::SandConnectorSecretStore;
use super::channel_store::{ChannelConfig, ChannelConnection, FileChannelStore};
use super::session_store_factories::{
    automation_store_for_db_path_with_time_zone_resolver, channel_store_for_db_path,
    workflow_store_for_db_path_with_time_zone_resolver,
};
use crate::extensions::memory::memory_service::{FileMemoryStore, MemoryService};
use crate::extensions::memory::project_membership::AgentProjectMembership;
use crate::automations::automation_store::{FileAutomationStore, UserTimeZoneResolver, agent_has_automations};
use crate::workflows::workflow_store::{FileWorkflowStore, agent_has_workflows};
use super::session_maintenance::{
    backfill_transcript_from_outline, clear_stale_checkpoint_roots_once,
    recover_conversation_root_if_missing, repair_hidden_transcript_entries_once,
    retire_legacy_store_blobs_once, sync_recovered_profile_name,
};
use super::session_materialization::{
    MAX_AGENTS_PER_USER, MaterializedAgentRecord, SessionMintQueue, count_owned_agents,
    is_agent_cap_reached, list_agent_record_ids, list_pruned_placeholder_ids,
    materialize_new_session, open_existing_session, report_materialization_failure,
};
use super::pending_card_sweeps::{
    expire_pending_auto_review_approval_entries,
    expire_pending_local_tool_permission_ask_entries,
};
use super::session_roster::{RosterExtrasCache, list_agents, summarize_agent_by_id};
use super::session_summaries::AgentSummary;
use super::session_profile_files::{
    AgentAvatarResponse, AgentProfileUpdate, get_agent_avatar as get_profile_avatar,
    get_agent_avatar_png as get_profile_avatar_png,
    get_agent_profile_text as read_agent_profile_text, write_agent_profile_update,
};
use super::session_mutations::set_agent_avatar_bytes as mutate_agent_avatar_bytes;
use super::production_agent_store::{ProductionAgentStore, ProductionWorkerBlobStore};
use super::native_messaging::{
    FabushiHumanReactionError, FabushiHumanSendError, FabushiNativeMessagingClient, FabushiRemoteHumanAttachment, FabushiRemoteHumanCall,
    FabushiRemoteHumanCallEvent, FabushiRemoteHumanMessage, FabushiRemoteHumanReaction,
    fabushi_identity_text, fabushi_message_identity_text,
};

#[derive(Debug, thiserror::Error)]
enum ForwardSourceFailure {
    #[error("{0}")]
    Policy(String),
    #[error("{0}")]
    Storage(String),
}

// One invocation keeps its original credential binding and first qualified
// reaction refusal. This is not another durable message or credential owner.
pub(crate) struct HumanSyncRecoveryContext {
    local_human_id: String,
    credential_fingerprint: String,
    first_reaction_refusal: Option<String>,
}

impl HumanSyncRecoveryContext {
    fn is_continuing(&self) -> bool {
        self.first_reaction_refusal.is_some()
    }

    fn finish<T>(self, value: T) -> Result<T, String> {
        match self.first_reaction_refusal {
            Some(error) => Err(error),
            None => Ok(value),
        }
    }
}

pub const PRODUCTION_BLOB_BUSY_TIMEOUT_MS: u64 = 5_000;

pub type ProductionAgentWorkerPool = AgentWorkerPool<ProductionAgentStoreWorkerBackend>;

#[derive(Debug, Clone, PartialEq)]
pub struct PreparedAgentBlobStore {
    pub agent_id: String,
    pub session_db_path: PathBuf,
    pub blob_db_path: PathBuf,
    pub latest_root_blob_id: Option<Vec<u8>>,
    pub persisted_root_blob_id: Vec<u8>,
    pub session_state: AgentDbSerdeSnapshot,
    pub transcript_tail: TranscriptPage,
    pub profile_file: Option<SandAgentProfile>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum FallbackSession {
    Existing(PreparedAgentBlobStore),
    Created(MaterializedAgentRecord),
}

pub struct ProductionMaterializedSession {
    pub record: MaterializedAgentRecord,
    pub prepared: PreparedAgentBlobStore,
    pub db: Arc<SandAgentDb>,
    pub agent_store: Arc<ProductionAgentStore>,
    pub memory: FileMemoryStore,
    pub project_membership: AgentProjectMembership,
    pub automations: FileAutomationStore,
    pub workflows: FileWorkflowStore,
    pub channels: FileChannelStore,
    conversation_state: Option<ResolvedConversationState>,
}

impl ProductionMaterializedSession {
    pub fn conversation_state(&self) -> Option<&ResolvedConversationState> {
        self.conversation_state.as_ref()
    }

    pub fn reset_from_db(
        &mut self,
        owner: &ProductionSessionWorkers,
    ) -> Result<bool, String> {
        self.conversation_state = owner.read_agent_conversation_state(&self.record.id)?;
        Ok(self.conversation_state.is_some())
    }
}

// Use the existing lock registry without retaining one entry forever for each
// retried nonce. A waiter owns its Arc before locking; under the registry lock,
// releasing this scope under the registry lock makes final-waiter cleanup
// atomic with lookup and with another finishing scope's cleanup.
#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum HumanCreationStage {
    BeforeLock,
    Locked,
    BeforeInitialize,
    BeforePublish,
}

#[cfg(test)]
pub(crate) type HumanCreationHook = Arc<
    dyn Fn(HumanCreationStage, &str, Option<&Arc<SandAgentDb>>) + Send + Sync,
>;

/// Shipping Host owner for the Grok session materialization worker boundary.
///
/// The pool is retained for the Host lifetime so per-agent blob workers are
/// reused across turns and are closed as part of Host shutdown. The remaining
/// recovery/metadata-GC/session materialization modules stay explicitly
/// non-final until their frozen Grok contracts are ported.
pub struct ProductionSessionWorkers {
    pub(crate) agents_root: PathBuf,
    pub(crate) pool: Arc<ProductionAgentWorkerPool>,
    pub(crate) conversation_size_maintenance: ConversationSizeMaintenance,
    pub(crate) conversation_state: SessionConversationState,
    pub(crate) memory_service: Arc<MemoryService>,
    pub(crate) mint_queue: SessionMintQueue,
    pub(crate) db_owners: Mutex<BTreeMap<String, Arc<SandAgentDb>>>,
    pub(crate) human_message_locks: Mutex<BTreeMap<String, Arc<Mutex<()>>>>,
    #[cfg(test)]
    pub(crate) human_creation_hook: Option<HumanCreationHook>,
    pub(crate) agent_store_owners: Mutex<BTreeMap<String, Arc<ProductionAgentStore>>>,
    pub(crate) deleting_agents: Mutex<BTreeSet<String>>,
    pub(crate) roster_extras_cache: RosterExtrasCache,
    pub(crate) user_time_zone_resolver: UserTimeZoneResolver,
    pub(crate) local_human_id: Option<String>,
    pub(crate) native_messaging: Option<Arc<FabushiNativeMessagingClient>>,
    pub(crate) native_messaging_required: bool,
    pub(crate) native_messaging_error: Option<String>,
    pub(crate) call_session_store: Option<Arc<CallSessionStore>>,
    pub(crate) call_session_store_error: Option<String>,
    pub(crate) busy_timeout_ms: u64,
    pub(crate) mini_app_package_verifier: Mutex<Option<MiniAppPackageVerifier>>,
    pub(crate) miniapp_launch_sessions: Mutex<BTreeMap<String, HostMiniAppLaunchSession>>,
    pub(crate) miniapp_grants: Mutex<BTreeMap<(String, String), fabushi_messaging_core::MiniAppGrant>>,
    pub(crate) has_valid_native_credential: Arc<AtomicBool>,
    pub(crate) account_data_root: Arc<Mutex<Option<PathBuf>>>,
    pub(crate) telegram_key_clear_callback: Arc<Mutex<Option<TelegramKeyClearHook>>>,
    pub(crate) media_cache: Arc<Mutex<fabushi_messaging_core::MediaCache>>,
    pub(crate) miniapp_service_calls: Arc<Mutex<std::collections::HashMap<String, fabushi_messaging_core::MiniAppServiceCallSession>>>,
}

#[path = "tdrp_forum_topics.rs"]
pub mod tdrp_forum_topics;
pub use tdrp_forum_topics::*;
#[path = "tdrp_chat_admin_moderation.rs"]
pub mod tdrp_chat_admin_moderation;
pub use tdrp_chat_admin_moderation::*;
#[path = "tdrp_calls_media.rs"]
pub mod tdrp_calls_media;
pub use tdrp_calls_media::*;
#[path = "tdrp_miniapp_bridge.rs"]
pub mod tdrp_miniapp_bridge;
pub use tdrp_miniapp_bridge::*;
#[path = "tdrp_settings_credentials.rs"]
pub mod tdrp_settings_credentials;
pub use tdrp_settings_credentials::*;
#[path = "tdrp_dialog_state.rs"]
pub mod tdrp_dialog_state;
pub use tdrp_dialog_state::*;
#[path = "tdrp_forward_share.rs"]
pub mod tdrp_forward_share;
pub use tdrp_forward_share::*;
#[path = "tdrp_share_link.rs"]
pub mod tdrp_share_link;
pub use tdrp_share_link::*;
#[path = "tdrp_message_retry.rs"]
pub mod tdrp_message_retry;
pub use tdrp_message_retry::*;
#[path = "tdrp_message_mutations.rs"]
pub mod tdrp_message_mutations;
pub use tdrp_message_mutations::*;
#[path = "tdrp_reactions_sync.rs"]
pub mod tdrp_reactions_sync;
pub use tdrp_reactions_sync::*;

impl ProductionSessionWorkers {
    pub fn production() -> Self {
        Self::production_with_user_time_zone_resolver(Arc::new(|| None))
    }

    pub fn production_with_user_time_zone_resolver(
        user_time_zone_resolver: UserTimeZoneResolver,
    ) -> Self {
        let agents_root = get_sand_agents_root_dir(None);
        let memory_service = Arc::new(MemoryService::new(agents_root.clone()));
        Self::with_agents_root_and_dependencies(
            agents_root,
            PRODUCTION_BLOB_BUSY_TIMEOUT_MS,
            user_time_zone_resolver,
            memory_service,
        )
    }

    pub fn production_with_dependencies(
        user_time_zone_resolver: UserTimeZoneResolver,
        memory_service: Arc<MemoryService>,
    ) -> Self {
        Self::production_with_identity_and_dependencies(None, user_time_zone_resolver, memory_service)
    }

    pub fn production_with_identity_and_dependencies(
        local_human_id: Option<String>,
        user_time_zone_resolver: UserTimeZoneResolver,
        memory_service: Arc<MemoryService>,
    ) -> Self {
        let native_messaging_required = local_human_id
            .as_deref()
            .is_some_and(|value| !value.trim().is_empty());
        let (native_messaging, native_messaging_error) = if native_messaging_required {
            match FabushiNativeMessagingClient::from_env() {
                Ok(client) => (Some(Arc::new(client)), None),
                Err(error) => (None, Some(error)),
            }
        } else {
            (None, None)
        };
        Self::with_agents_root_identity_messaging_and_dependencies(
            get_sand_agents_root_dir(None),
            PRODUCTION_BLOB_BUSY_TIMEOUT_MS,
            local_human_id,
            native_messaging,
            native_messaging_required,
            native_messaging_error,
            user_time_zone_resolver,
            memory_service,
        )
    }

    pub fn with_agents_root(
        agents_root: impl Into<PathBuf>,
        busy_timeout_ms: u64,
    ) -> Self {
        Self::with_agents_root_and_user_time_zone_resolver(
            agents_root,
            busy_timeout_ms,
            Arc::new(|| None),
        )
    }

    pub fn with_agents_root_and_user_time_zone_resolver(
        agents_root: impl Into<PathBuf>,
        busy_timeout_ms: u64,
        user_time_zone_resolver: UserTimeZoneResolver,
    ) -> Self {
        let agents_root = agents_root.into();
        let memory_service = Arc::new(MemoryService::new(agents_root.clone()));
        Self::with_agents_root_and_dependencies(
            agents_root,
            busy_timeout_ms,
            user_time_zone_resolver,
            memory_service,
        )
    }

    pub fn with_agents_root_and_dependencies(
        agents_root: impl Into<PathBuf>,
        busy_timeout_ms: u64,
        user_time_zone_resolver: UserTimeZoneResolver,
        memory_service: Arc<MemoryService>,
    ) -> Self {
        Self::with_agents_root_identity_and_dependencies(
            agents_root,
            busy_timeout_ms,
            None,
            user_time_zone_resolver,
            memory_service,
        )
    }

    pub fn with_agents_root_identity_and_dependencies(
        agents_root: impl Into<PathBuf>,
        busy_timeout_ms: u64,
        local_human_id: Option<String>,
        user_time_zone_resolver: UserTimeZoneResolver,
        memory_service: Arc<MemoryService>,
    ) -> Self {
        Self::with_agents_root_identity_messaging_and_dependencies(
            agents_root,
            busy_timeout_ms,
            local_human_id,
            None,
            false,
            None,
            user_time_zone_resolver,
            memory_service,
        )
    }

    pub fn with_agents_root_identity_messaging_and_dependencies(
        agents_root: impl Into<PathBuf>,
        busy_timeout_ms: u64,
        local_human_id: Option<String>,
        native_messaging: Option<Arc<FabushiNativeMessagingClient>>,
        native_messaging_required: bool,
        native_messaging_error: Option<String>,
        user_time_zone_resolver: UserTimeZoneResolver,
        memory_service: Arc<MemoryService>,
    ) -> Self {
        let agents_root = agents_root.into();
        let local_human_id = local_human_id
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let (call_session_store, call_session_store_error) =
            match CallSessionStore::open(&agents_root, busy_timeout_ms) {
                Ok(store) => (Some(Arc::new(store)), None),
                Err(error) => (None, Some(error)),
            };
        let default_has_valid_native = local_human_id.is_some()
            && native_messaging_error.is_none()
            && (native_messaging.is_some() || !native_messaging_required);
        Self {
            memory_service,
            agents_root,
            pool: Arc::new(AgentWorkerPool::new(
                create_production_agent_store_worker_backend(busy_timeout_ms),
            )),
            conversation_size_maintenance: ConversationSizeMaintenance::default(),
            conversation_state: SessionConversationState::new(busy_timeout_ms),
            mint_queue: SessionMintQueue::default(),
            db_owners: Mutex::new(BTreeMap::new()),
            human_message_locks: Mutex::new(BTreeMap::new()),
            #[cfg(test)]
            human_creation_hook: None,
            agent_store_owners: Mutex::new(BTreeMap::new()),
            deleting_agents: Mutex::new(BTreeSet::new()),
            roster_extras_cache: RosterExtrasCache::default(),
            user_time_zone_resolver,
            local_human_id,
            native_messaging,
            native_messaging_required,
            native_messaging_error,
            call_session_store,
            call_session_store_error,
            busy_timeout_ms,
            mini_app_package_verifier: Mutex::new(None),
            miniapp_launch_sessions: Mutex::new(BTreeMap::new()),
            miniapp_grants: Mutex::new(BTreeMap::new()),
            has_valid_native_credential: Arc::new(AtomicBool::new(default_has_valid_native)),
            account_data_root: Arc::new(Mutex::new(None)),
            telegram_key_clear_callback: Arc::new(Mutex::new(None)),
            media_cache: Arc::new(Mutex::new(
                fabushi_messaging_core::MediaCache::new(64 * 1024 * 1024)
                    .expect("default media cache budget of 64 MiB must be valid"),
            )),
            miniapp_service_calls: Arc::new(Mutex::new(std::collections::HashMap::new())),
        }
    }

    pub fn call_session_store_path(&self) -> Result<&Path, String> {
        self.call_session_store
            .as_ref()
            .map(|store| store.path())
            .ok_or_else(|| {
                self.call_session_store_error
                    .clone()
                    .unwrap_or_else(|| "CallSession store is unavailable".into())
            })
    }

    fn call_sessions(&self) -> Result<&Arc<CallSessionStore>, String> {
        self.call_session_store.as_ref().ok_or_else(|| {
            self.call_session_store_error
                .clone()
                .unwrap_or_else(|| "CallSession store is unavailable".into())
        })
    }

    fn local_call_peer(&self, participant_ids: &[String]) -> Result<String, String> {
        let local = self
            .local_human_id
            .as_deref()
            .ok_or_else(|| "CallSession requires an authenticated Human identity".to_string())?;
        let mut participants = participant_ids
            .iter()
            .map(|value| value.trim())
            .filter(|value| !value.is_empty())
            .collect::<Vec<_>>();
        participants.sort_unstable();
        participants.dedup();
        if participants.len() != 2 || !participants.iter().any(|value| *value == local) {
            return Err("shipping Human calls require exactly the authenticated Human and one peer".into());
        }
        participants
            .into_iter()
            .find(|value| *value != local)
            .map(str::to_string)
            .ok_or_else(|| "shipping Human call peer is unavailable".to_string())
    }

    fn remote_call_peer(&self, remote: &FabushiRemoteHumanCall) -> Result<String, String> {
        let local = self
            .local_human_id
            .as_deref()
            .ok_or_else(|| "CallSession requires an authenticated Human identity".to_string())?;
        let creator = fabushi_identity_text(&remote.creator_user_id)?;
        let peer = fabushi_identity_text(&remote.peer_user_id)?;
        if creator == local && peer != local {
            Ok(peer)
        } else if peer == local && creator != local {
            Ok(creator)
        } else {
            Err("remote Human call does not belong to the authenticated Human identity".into())
        }
    }

    fn validate_remote_call(
        &self,
        remote: &FabushiRemoteHumanCall,
        local: &CallSession,
    ) -> Result<(), String> {
        if remote.call_id != local.id {
            return Err("remote Human call id does not match CallSession owner".into());
        }
        let peer = self.remote_call_peer(remote)?;
        if self.local_call_peer(&local.participant_ids)? != peer {
            return Err("remote Human call participants do not match CallSession owner".into());
        }
        Ok(())
    }

    fn stable_call_event_id(
        prefix: &str,
        generation: u64,
        payload: &serde_json::Value,
    ) -> Result<String, String> {
        let bytes = serde_json::to_vec(payload).map_err(|error| error.to_string())?;
        let digest = Sha256::digest(bytes);
        let suffix = digest
            .iter()
            .take(16)
            .map(|byte| format!("{byte:02x}"))
            .collect::<String>();
        Ok(format!("{prefix}:{generation}:{suffix}"))
    }

    fn apply_remote_call_event(
        &self,
        event: &FabushiRemoteHumanCallEvent,
    ) -> Result<(), String> {
        let current = self.authorized_call_session(&event.call_id)?;
        match event.kind.as_str() {
            "transition" => {
                let action = event
                    .payload
                    .get("action")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "remote call transition omitted action".to_string())?;
                let target_state = event
                    .payload
                    .get("state")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "remote call transition omitted state".to_string())?;
                if current.state == target_state && current.generation == event.generation {
                    return Ok(());
                }
                let terminal_reason = event
                    .payload
                    .get("terminalReason")
                    .and_then(serde_json::Value::as_str);
                let (projected_state, projected_generation, _, _) =
                    transition_target(&current, action, terminal_reason)?;
                if projected_state != target_state || projected_generation != event.generation {
                    return Err("remote call transition disagrees with canonical Host state machine".into());
                }
                self.call_sessions()?
                    .transition(&event.call_id, current.generation, action, terminal_reason)?;
            }
            "signal" => {
                let kind = event
                    .payload
                    .get("signalKind")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "remote call signal omitted signalKind".to_string())?;
                let payload = event
                    .payload
                    .get("signal")
                    .ok_or_else(|| "remote call signal omitted signal payload".to_string())?;
                self.call_sessions()?.append_remote_signal(
                    &event.call_id,
                    event.generation,
                    event.seq,
                    &event.device_id,
                    kind,
                    payload,
                )?;
            }
            "media" => {
                if current.generation != event.generation {
                    return Err("remote call media event has stale generation".into());
                }
                let capabilities = event
                    .payload
                    .get("mediaCapabilities")
                    .ok_or_else(|| "remote call media event omitted mediaCapabilities".to_string())?;
                let devices = event
                    .payload
                    .get("deviceSelection")
                    .ok_or_else(|| "remote call media event omitted deviceSelection".to_string())?;
                if current.media_capabilities == *capabilities && current.device_selection == *devices {
                    return Ok(());
                }
                self.call_sessions()?.update_media(
                    &event.call_id,
                    event.generation,
                    Some(capabilities),
                    Some(devices),
                )?;
            }
            "participant" => {
                return Err("remote call participant mutation is unsupported by the current CallSession owner".into());
            }
            _ => return Err(format!("unsupported remote call event kind: {}", event.kind)),
        }
        Ok(())
    }

    pub fn sync_call_session_from_remote(&self, call_id: &str) -> Result<CallSession, String> {
        self.sync_call_session_from_remote_with_recovery(call_id, None)
    }

    fn sync_call_session_from_remote_with_recovery(
        &self,
        call_id: &str,
        recovery: Option<&HumanSyncRecoveryContext>,
    ) -> Result<CallSession, String> {
        const PAGE_LIMIT: usize = 200;
        const MAX_PAGES: usize = 32;

        self.check_human_sync_continuation(recovery)?;
        let local = self.authorized_call_session(call_id)?;
        let Some(client) = self.shipping_native_messaging()? else {
            return Ok(local);
        };
        let identity = client.identity()?;
        if identity.user_id != self.local_human_id()? {
            return Err("Fabushi call credential identity does not match active Human identity".into());
        }

        let continuation_owner = if recovery.is_some_and(HumanSyncRecoveryContext::is_continuing) {
            let owner = self.open_human_conversation_db_owner(&local.scope_id)?;
            let metadata = Self::read_human_conversation_metadata(&owner, &local.scope_id, true)?;
            if self.peer_human_id_from_metadata(&metadata)? != self.local_call_peer(&local.participant_ids)? {
                return Err("remote Human call participants do not match CallSession owner".into());
            }
            Some((owner, metadata))
        } else { None };
        let check_continuation = || -> Result<(), String> {
            self.check_human_sync_continuation(recovery)?;
            if let Some((owner, metadata)) = &continuation_owner {
                self.revalidate_human_recovery_owner(&local.scope_id, owner, metadata)?;
                let current = self.authorized_call_session(call_id)?;
                if current.scope_id != local.scope_id {
                    return Err("remote Human call resolved to a different canonical conversation scope".into());
                }
                if current.participant_ids != local.participant_ids {
                    return Err("remote Human call participants do not match CallSession owner".into());
                }
            }
            Ok(())
        };
        check_continuation()?;
        let mut cursor = self.call_sessions()?.remote_event_seq(call_id)?;
        for _ in 0..MAX_PAGES {
            check_continuation()?;
            let (remote, events, next_after_seq) =
                client.get_human_call(call_id, cursor, PAGE_LIMIT)?;
            check_continuation()?;
            self.validate_remote_call(&remote, &self.authorized_call_session(call_id)?)?;
            let prior_cursor = cursor;
            for event in events {
                if continuation_owner.is_some() && event.call_id != call_id {
                    return Err("remote Human call event does not match requested CallSession owner".into());
                }
                if event.seq <= cursor {
                    continue;
                }
                if event.seq != cursor + 1 {
                    return Err(format!(
                        "remote Human call event sequence gap: expected {}, received {}",
                        cursor + 1,
                        event.seq
                    ));
                }
                check_continuation()?;
                self.apply_remote_call_event(&event)?;
                check_continuation()?;
                self.call_sessions()?.set_remote_event_seq(call_id, event.seq)?;
                cursor = event.seq;
            }
            if cursor >= remote.event_seq {
                check_continuation()?;
                return self.authorized_call_session(call_id);
            }
            if cursor == prior_cursor || next_after_seq <= cursor {
                return Err("remote Human call sync made no progress before reaching backend head".into());
            }
        }
        Err("remote Human call sync exceeded the bounded replay window".into())
    }

    pub fn call_transport_identity(&self) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?.to_string();
        let client = self
            .shipping_native_messaging()?
            .ok_or_else(|| "shipping Human call transport requires native messaging".to_string())?;
        let identity = client.identity()?;
        if identity.user_id != local_human_id {
            return Err("Fabushi call credential identity does not match active Human identity".into());
        }
        Ok(serde_json::json!({
            "userId": identity.user_id,
            "deviceId": identity.device_id,
        }))
    }

    pub fn call_ice_servers(&self) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?.to_string();
        let client = self
            .shipping_native_messaging()?
            .ok_or_else(|| "shipping Human call media transport requires native messaging".to_string())?;
        let identity = client.identity()?;
        if identity.user_id != local_human_id {
            return Err("Fabushi call credential identity does not match active Human identity".into());
        }
        let (ice_servers, ttl_seconds) = client.get_human_call_ice_servers()?;
        Ok(serde_json::json!({
            "iceServers": ice_servers,
            "ttlSeconds": ttl_seconds,
        }))
    }

    pub fn call_transport_lease(&self, call_id: &str) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?.to_string();
        let local = self.authorized_call_session(call_id)?;
        let client = self
            .shipping_native_messaging()?
            .ok_or_else(|| "shipping Human call media transport requires native messaging".to_string())?;
        let identity = client.identity()?;
        if identity.user_id != local_human_id {
            return Err("Fabushi call credential identity does not match active Human identity".into());
        }
        let cursor = self.call_sessions()?.remote_event_seq(call_id)?;
        let (remote, _, _) = client.get_human_call(call_id, cursor, 1)?;
        self.validate_remote_call(&remote, &local)?;
        if remote.event_seq < cursor {
            return Err("Fabushi call backend event cursor regressed behind the canonical Host owner".into());
        }
        let creator = fabushi_identity_text(&remote.creator_user_id)?;
        let role = if creator == local_human_id { "creator" } else { "peer" };
        let claimed_device_id = if role == "creator" {
            remote.creator_device_id.as_deref()
        } else {
            remote.peer_device_id.as_deref()
        };
        Ok(serde_json::json!({
            "userId": local_human_id,
            "deviceId": identity.device_id,
            "role": role,
            "claimedDeviceId": claimed_device_id,
            "isOwner": claimed_device_id == Some(identity.device_id.as_str()),
            "claimAvailable": claimed_device_id.is_none(),
            "creatorDeviceId": remote.creator_device_id,
            "peerDeviceId": remote.peer_device_id,
            "state": remote.state,
            "generation": remote.generation,
            "eventSeq": remote.event_seq,
        }))
    }

    pub fn sync_human_calls(&self) -> Result<Vec<CallSession>, String> {
        self.sync_human_calls_with_recovery(None)
    }

    fn sync_human_calls_with_recovery(
        &self,
        recovery: Option<&HumanSyncRecoveryContext>,
    ) -> Result<Vec<CallSession>, String> {
        self.check_human_sync_continuation(recovery)?;
        let Some(client) = self.shipping_native_messaging()? else {
            return Ok(Vec::new());
        };
        let local_human_id = self.local_human_id()?.to_string();
        let identity = client.identity()?;
        if identity.user_id != local_human_id {
            return Err("Fabushi call credential identity does not match active Human identity".into());
        }
        self.check_human_sync_continuation(recovery)?;
        let remote_calls = client.list_human_calls(200)?;
        self.check_human_sync_continuation(recovery)?;
        let mut synced = Vec::with_capacity(remote_calls.len());
        for remote in remote_calls {
            self.check_human_sync_continuation(recovery)?;
            let peer = self.remote_call_peer(&remote)?;
            let conversation = self.create_human_conversation_with_recovery(&peer, &peer, recovery)?;
            self.check_human_sync_continuation(recovery)?;
            let scope_id = conversation
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "Human call discovery could not resolve canonical conversation scope".to_string())?;
            let continuation_owner = if recovery.is_some_and(HumanSyncRecoveryContext::is_continuing) {
                let owner = self.open_human_conversation_db_owner(scope_id)?;
                let metadata = Self::read_human_conversation_metadata(&owner, scope_id, true)?;
                if self.peer_human_id_from_metadata(&metadata)? != peer {
                    return Err("remote Human call participants do not match CallSession owner".into());
                }
                Some((owner, metadata))
            } else { None };
            let check_continuation = || -> Result<(), String> {
                self.check_human_sync_continuation(recovery)?;
                if let Some((owner, metadata)) = &continuation_owner {
                    self.revalidate_human_recovery_owner(scope_id, owner, metadata)?;
                }
                Ok(())
            };
            check_continuation()?;
            if self.call_sessions()?.get(&remote.call_id)?.is_none() {
                let creator = fabushi_identity_text(&remote.creator_user_id)?;
                let participants = vec![local_human_id.clone(), peer.clone()];
                check_continuation()?;
                self.call_sessions()?.create_with_id(
                    &remote.call_id,
                    scope_id,
                    &creator,
                    &participants,
                )?;
            }
            check_continuation()?;
            let call = self.authorized_call_session(&remote.call_id)?;
            if call.scope_id != scope_id {
                return Err("remote Human call resolved to a different canonical conversation scope".into());
            }
            self.validate_remote_call(&remote, &call)?;
            synced.push(self.sync_call_session_from_remote_with_recovery(&remote.call_id, recovery)?);
            check_continuation()?;
        }
        self.check_human_sync_continuation(recovery)?;
        Ok(synced)
    }

    pub fn create_call_session(
        &self,
        scope_id: &str,
        participant_ids: &[String],
    ) -> Result<CallSession, String> {
        let creator_id = self
            .local_human_id
            .as_deref()
            .ok_or_else(|| "CallSession creation requires an authenticated Human identity".to_string())?;
        self.read_human_conversation_transcript(scope_id)?;
        let Some(client) = self.shipping_native_messaging()? else {
            return self.call_sessions()?.create(scope_id, creator_id, participant_ids);
        };
        let peer = self.local_call_peer(participant_ids)?;
        let identity = client.identity()?;
        if identity.user_id != creator_id {
            return Err("Fabushi call credential identity does not match CallSession creator".into());
        }
        let call_id = Uuid::new_v4().to_string();
        let remote = client.create_human_call(&call_id, &peer)?;
        if remote.call_id != call_id
            || fabushi_identity_text(&remote.creator_user_id)? != creator_id
            || self.remote_call_peer(&remote)? != peer
            || remote.state != "invited"
            || remote.generation != 0
            || remote.event_seq != 0
        {
            return Err("Fabushi call backend returned an invalid newly-created call".into());
        }
        self.call_sessions()?
            .create_with_id(&call_id, scope_id, creator_id, participant_ids)
    }

    fn authorized_call_session(&self, call_id: &str) -> Result<CallSession, String> {
        let local_human_id = self
            .local_human_id
            .as_deref()
            .ok_or_else(|| "CallSession access requires an authenticated Human identity".to_string())?;
        let call = self
            .call_sessions()?
            .get(call_id)?
            .ok_or_else(|| "call session not found".to_string())?;
        if !call.participant_ids.iter().any(|participant| participant == local_human_id) {
            return Err("call session does not belong to the authenticated Human identity".into());
        }
        Ok(call)
    }

    pub fn get_call_session(&self, call_id: &str) -> Result<Option<CallSession>, String> {
        match self.authorized_call_session(call_id) {
            Ok(call) => {
                if self.shipping_native_messaging()?.is_some() {
                    self.sync_call_session_from_remote(call_id).map(Some)
                } else {
                    Ok(Some(call))
                }
            }
            Err(error) if error == "call session not found" => Ok(None),
            Err(error) => Err(error),
        }
    }

    pub fn list_call_sessions(
        &self,
        scope_id: &str,
        limit: usize,
    ) -> Result<Vec<CallSession>, String> {
        let local_human_id = self
            .local_human_id
            .as_deref()
            .ok_or_else(|| "CallSession access requires an authenticated Human identity".to_string())?;
        self.read_human_conversation_transcript(scope_id)?;
        if self.shipping_native_messaging()?.is_some() {
            self.sync_human_calls()?;
        }
        Ok(self
            .call_sessions()?
            .list_for_scope(scope_id, limit)?
            .into_iter()
            .filter(|call| call.participant_ids.iter().any(|participant| participant == local_human_id))
            .collect())
    }

    pub fn transition_call_session(
        &self,
        call_id: &str,
        expected_generation: u64,
        action: &str,
        terminal_reason: Option<&str>,
    ) -> Result<CallSession, String> {
        let current = if self.shipping_native_messaging()?.is_some() {
            self.sync_call_session_from_remote(call_id)?
        } else {
            self.authorized_call_session(call_id)?
        };
        let Some(client) = self.shipping_native_messaging()? else {
            return self.call_sessions()?
                .transition(call_id, expected_generation, action, terminal_reason);
        };
        if current.generation != expected_generation {
            return Err(format!(
                "stale call generation: expected {}, current {}",
                expected_generation, current.generation
            ));
        }
        let (next_state, next_generation, _, normalized_reason) =
            transition_target(&current, action, terminal_reason)?;
        let mut payload = serde_json::json!({
            "action": action,
            "state": next_state,
        });
        if let Some(reason) = normalized_reason {
            payload["terminalReason"] = serde_json::json!(reason);
        }
        let client_event_id = format!("transition:{expected_generation}:{action}");
        client.append_human_call_event(
            call_id,
            &client_event_id,
            next_generation,
            "transition",
            &payload,
        )?;
        self.sync_call_session_from_remote(call_id)
    }

    pub fn update_call_media(
        &self,
        call_id: &str,
        expected_generation: u64,
        media_capabilities: Option<&serde_json::Value>,
        device_selection: Option<&serde_json::Value>,
    ) -> Result<CallSession, String> {
        let current = if self.shipping_native_messaging()?.is_some() {
            self.sync_call_session_from_remote(call_id)?
        } else {
            self.authorized_call_session(call_id)?
        };
        let Some(client) = self.shipping_native_messaging()? else {
            return self.call_sessions()?.update_media(
                call_id,
                expected_generation,
                media_capabilities,
                device_selection,
            );
        };
        if current.generation != expected_generation {
            return Err(format!(
                "stale call generation: expected {}, current {}",
                expected_generation, current.generation
            ));
        }
        if matches!(current.state.as_str(), "ended" | "failed") {
            return Err("terminal call session cannot update media state".into());
        }
        if media_capabilities.is_none() && device_selection.is_none() {
            return Err("call media update requires capabilities or device selection".into());
        }
        let capabilities = media_capabilities.unwrap_or(&current.media_capabilities);
        let devices = device_selection.unwrap_or(&current.device_selection);
        if !capabilities.is_object() || !devices.is_object() {
            return Err("call media capabilities and device selection must be objects".into());
        }
        let payload = serde_json::json!({
            "mediaCapabilities": capabilities,
            "deviceSelection": devices,
        });
        let client_event_id = Self::stable_call_event_id("media", expected_generation, &payload)?;
        client.append_human_call_event(
            call_id,
            &client_event_id,
            expected_generation,
            "media",
            &payload,
        )?;
        self.sync_call_session_from_remote(call_id)
    }

    pub fn append_call_signal(
        &self,
        call_id: &str,
        expected_generation: u64,
        seq: u64,
        sender_device_id: &str,
        kind: &str,
        payload: &serde_json::Value,
    ) -> Result<CallSignal, String> {
        let current = if self.shipping_native_messaging()?.is_some() {
            self.sync_call_session_from_remote(call_id)?
        } else {
            self.authorized_call_session(call_id)?
        };
        let Some(client) = self.shipping_native_messaging()? else {
            return self.call_sessions()?.append_signal(
                call_id,
                expected_generation,
                seq,
                sender_device_id,
                kind,
                payload,
            );
        };
        if current.generation != expected_generation {
            return Err(format!(
                "stale call generation: expected {}, current {}",
                expected_generation, current.generation
            ));
        }
        if matches!(current.state.as_str(), "ended" | "failed") {
            return Err("terminal call session cannot accept signaling".into());
        }
        let identity = client.identity()?;
        if identity.device_id != sender_device_id {
            return Err("call signal senderDeviceId must match the authenticated Host device".into());
        }
        let remote_payload = serde_json::json!({
            "senderDeviceId": sender_device_id,
            "signalKind": kind,
            "signal": payload,
        });
        let id_payload = serde_json::json!({
            "deviceId": identity.device_id,
            "kind": kind,
            "signal": payload,
        });
        let client_event_id = Self::stable_call_event_id("signal", expected_generation, &id_payload)?;
        let (_, accepted_event) = client.append_human_call_event(
            call_id,
            &client_event_id,
            expected_generation,
            "signal",
            &remote_payload,
        )?;
        self.sync_call_session_from_remote(call_id)?;
        self.call_sessions()?
            .list_signals(
                call_id,
                expected_generation,
                accepted_event.seq.saturating_sub(1),
                1,
            )?
            .into_iter()
            .find(|signal| signal.seq == accepted_event.seq)
            .ok_or_else(|| "remote call signal did not materialize in canonical CallSession owner".to_string())
    }

    pub fn list_call_signals(
        &self,
        call_id: &str,
        generation: u64,
        after_seq: u64,
        limit: usize,
    ) -> Result<Vec<CallSignal>, String> {
        self.authorized_call_session(call_id)?;
        if self.shipping_native_messaging()?.is_some() {
            self.sync_call_session_from_remote(call_id)?;
        }
        self.call_sessions()?
            .list_signals(call_id, generation, after_seq, limit)
    }

    pub fn worker_pool(&self) -> Arc<ProductionAgentWorkerPool> {
        Arc::clone(&self.pool)
    }

    pub fn memory_service(&self) -> Arc<MemoryService> {
        Arc::clone(&self.memory_service)
    }

    pub fn resolve_user_time_zone(&self) -> Option<String> {
        (self.user_time_zone_resolver)()
    }

    pub fn agents_root(&self) -> &Path {
        &self.agents_root
    }

    pub fn is_agent_cap_reached_after_reclaim(
        &self,
        active_agent_id: Option<&str>,
    ) -> Result<bool, String> {
        if self.count_owned_agents()? < MAX_AGENTS_PER_USER {
            return Ok(false);
        }
        let _ = self.reclaim_pruned_placeholders(active_agent_id)?;
        Ok(self.count_owned_agents()? >= MAX_AGENTS_PER_USER)
    }

    pub fn busy_timeout_ms(&self) -> u64 {
        self.busy_timeout_ms
    }

    pub fn mint_agent_with<T>(
        &self,
        mint: impl FnOnce(&str) -> Result<T, String>,
    ) -> Result<T, String> {
        self.mint_queue.run(|| {
            fs::create_dir_all(&self.agents_root)
                .map_err(|error| error.to_string())?;
            if is_agent_cap_reached(&self.agents_root)
                .map_err(|error| error.to_string())?
            {
                return Err("Agent limit of 50 reached".to_string());
            }
            let agent_id = loop {
                let candidate = Uuid::new_v4().to_string();
                if !self.agents_root.join(&candidate).exists() {
                    break candidate;
                }
            };
            mint(&agent_id)
        })
    }

    pub fn materialize_new_session(
        &self,
        profile: Option<&SandAgentProfile>,
        origin: &str,
        purpose: Option<&str>,
    ) -> Result<MaterializedAgentRecord, String> {
        self.materialize_new_session_with_active(profile, origin, purpose, None)
    }

    pub fn materialize_new_session_with_active(
        &self,
        profile: Option<&SandAgentProfile>,
        origin: &str,
        purpose: Option<&str>,
        active_agent_id: Option<&str>,
    ) -> Result<MaterializedAgentRecord, String> {
        let record = self.mint_queue.run(|| {
            if self.is_agent_cap_reached_after_reclaim(active_agent_id)? {
                return Err(format!("Agent limit of {MAX_AGENTS_PER_USER} reached"));
            }
            materialize_new_session(
                &self.agents_root,
                self.busy_timeout_ms,
                profile,
                origin,
                purpose,
            )
            .map_err(|error| error.to_string())
        })?;
        let _ = self.open_agent_db_owner(&record.id)?;
        Ok(record)
    }

    pub fn create_fallback_session(
        &self,
        active_agent_id: Option<&str>,
    ) -> Result<FallbackSession, String> {
        let result = self.mint_queue.run(|| {
            if self.is_agent_cap_reached_after_reclaim(active_agent_id)? {
                for agent_id in self.list_agent_record_ids()? {
                    match self.prepare_existing_agent(&agent_id) {
                        Ok(Some(prepared)) => return Ok(FallbackSession::Existing(prepared)),
                        Ok(None) => continue,
                        Err(_) => {
                            report_materialization_failure(
                                "fallback_adopt_failed",
                                &agent_id,
                                "String",
                            );
                            continue;
                        }
                    }
                }
                return Err(format!("Agent limit of {MAX_AGENTS_PER_USER} reached"));
            }
            materialize_new_session(
                &self.agents_root,
                self.busy_timeout_ms,
                None,
                "user",
                None,
            )
            .map(FallbackSession::Created)
            .map_err(|error| error.to_string())
        })?;
        if let FallbackSession::Created(record) = &result {
            let _ = self.open_agent_db_owner(&record.id)?;
        }
        Ok(result)
    }

    pub fn compose_materialized_session(
        &self,
        record: MaterializedAgentRecord,
        prepared: PreparedAgentBlobStore,
    ) -> Result<ProductionMaterializedSession, String> {
        let db = self.open_agent_db_owner(&record.id)?;
        let agent_store = self.open_agent_store_owner(&record.id)?;
        let memory = self
            .memory_service
            .create_agent_store(self.agents_root.join(&record.id));
        let project_membership = self
            .memory_service
            .project_membership_for_agent(&record.id);
        let automations = self.open_automation_store(&record.id)?;
        let workflows = self.open_workflow_store(&record.id)?;
        let channels = self.open_channel_store(&record.id)?;
        let conversation_state = self.read_agent_conversation_state(&record.id)?;
        Ok(ProductionMaterializedSession {
            record,
            prepared,
            db,
            agent_store,
            memory,
            project_membership,
            automations,
            workflows,
            channels,
            conversation_state,
        })
    }

    pub fn materialize_session_with_active(
        &self,
        profile: Option<&SandAgentProfile>,
        origin: &str,
        purpose: Option<&str>,
        active_agent_id: Option<&str>,
    ) -> Result<ProductionMaterializedSession, String> {
        let record = self.materialize_new_session_with_active(
            profile,
            origin,
            purpose,
            active_agent_id,
        )?;
        let prepared = self
            .prepare_existing_agent(&record.id)?
            .ok_or_else(|| "newly materialized session disappeared before composition".to_string())?;
        self.compose_materialized_session(record, prepared)
    }

    pub fn open_materialized_session(
        &self,
        agent_id: &str,
    ) -> Result<Option<ProductionMaterializedSession>, String> {
        let Some(prepared) = self.prepare_existing_agent(agent_id)? else {
            return Ok(None);
        };
        let profile = prepared
            .profile_file
            .clone()
            .ok_or_else(|| "prepared session is missing its profile".to_string())?;
        let record = MaterializedAgentRecord {
            id: agent_id.to_string(),
            db_path: prepared.session_db_path.clone(),
            profile,
        };
        Ok(Some(self.compose_materialized_session(record, prepared)?))
    }

    pub fn read_agent_transcript_entries(
        &self,
        agent_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        self.open_agent_db_owner(agent_id)?
            .get_transcript_entries()
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_transcript_page(
        &self,
        agent_id: &str,
        query: TranscriptPageQuery,
    ) -> Result<TranscriptPage, String> {
        self.open_agent_db_owner(agent_id)?
            .get_transcript_page(query)
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_transcript_window(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptWindow<std::collections::BTreeMap<String, usize>>, String> {
        self.open_agent_db_owner(agent_id)?
            .get_transcript_window(query)
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_transcript_tail(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, String> {
        self.open_agent_db_owner(agent_id)?
            .get_transcript_tail(query)
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_thread(
        &self,
        agent_id: &str,
        root_id: &str,
    ) -> Result<TranscriptThread, String> {
        self.open_agent_db_owner(agent_id)?
            .get_thread_entries(root_id)
            .map(|entries| TranscriptThread { entries })
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_outline(
        &self,
        agent_id: &str,
    ) -> Result<Vec<ConversationOutlineItem>, String> {
        let db_path = self.session_db_path(agent_id)?;
        let blob_db_path = conversation_blobs_path(&db_path);
        self.conversation_state
            .read_agent_outline(
                Arc::clone(&self.pool),
                agent_id,
                &db_path,
                &blob_db_path,
            )
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_conversation_state(
        &self,
        agent_id: &str,
    ) -> Result<Option<ResolvedConversationState>, String> {
        let db_path = self.session_db_path(agent_id)?;
        let blob_db_path = conversation_blobs_path(&db_path);
        self.conversation_state
            .read_agent_conversation_state(
                Arc::clone(&self.pool),
                agent_id,
                &db_path,
                &blob_db_path,
            )
            .map_err(|error| error.to_string())
    }

    pub fn connector_secret_store(&self) -> SandConnectorSecretStore {
        SandConnectorSecretStore::new(get_connector_secrets_root(Some(&self.agents_root)))
    }

    pub fn open_channel_store(&self, agent_id: &str) -> Result<FileChannelStore, String> {
        let db_path = self.session_db_path(agent_id)?;
        Ok(channel_store_for_db_path(&db_path))
    }

    pub fn open_automation_store(&self, agent_id: &str) -> Result<FileAutomationStore, String> {
        let db_path = self.session_db_path(agent_id)?;
        Ok(automation_store_for_db_path_with_time_zone_resolver(
            &db_path,
            Arc::clone(&self.user_time_zone_resolver),
        ))
    }

    pub fn agent_has_automations(&self, agent_id: &str) -> Result<bool, String> {
        let db_path = self.session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        Ok(agent_has_automations(agent_dir))
    }

    pub fn open_workflow_store(&self, agent_id: &str) -> Result<FileWorkflowStore, String> {
        let db_path = self.session_db_path(agent_id)?;
        Ok(workflow_store_for_db_path_with_time_zone_resolver(
            &db_path,
            Arc::clone(&self.user_time_zone_resolver),
        ))
    }

    pub fn agent_has_workflows(&self, agent_id: &str) -> Result<bool, String> {
        let db_path = self.session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        Ok(agent_has_workflows(agent_dir))
    }

    pub fn list_agent_channels(&self, agent_id: &str) -> Result<Vec<ChannelConnection>, String> {
        Ok(self
            .open_channel_store(agent_id)?
            .list_connections()
            .into_iter()
            .filter(|connection| {
                self.connector_secret_store()
                    .get_secret(agent_id, &connection.platform, "token")
                    .is_some()
            })
            .collect())
    }

    pub fn list_channel_configs(&self, agent_id: &str) -> Result<Vec<ChannelConfig>, String> {
        let store = self.open_channel_store(agent_id)?;
        let secrets = self.connector_secret_store();
        let mut configs = Vec::new();
        for platform in store.list_platforms() {
            let Some(token) = secrets.get_secret(agent_id, &platform, "token") else {
                continue;
            };
            configs.push(ChannelConfig {
                label: store.read_label(&platform).unwrap_or_else(|| platform.clone()),
                platform,
                token,
            });
        }
        Ok(configs)
    }

    pub fn store_connector_credential(
        &self,
        agent_id: &str,
        platform: &str,
        field: &str,
        value: &str,
    ) -> Result<bool, String> {
        let stored = self
            .connector_secret_store()
            .set_secret(agent_id, platform, field, value)
            .map_err(|error| error.to_string())?;
        if !stored {
            return Ok(false);
        }
        self.open_channel_store(agent_id)?
            .write_metadata(platform, "")
            .map_err(|error| error.to_string())
    }

    pub fn get_connector_secret(
        &self,
        agent_id: &str,
        platform: &str,
        field: &str,
    ) -> Result<Option<String>, String> {
        Ok(self
            .connector_secret_store()
            .get_secret(agent_id, platform, field))
    }

    pub fn remove_connector_platform_secret(
        &self,
        agent_id: &str,
        platform: &str,
    ) -> Result<bool, String> {
        self.connector_secret_store()
            .remove_agent_platform(agent_id, platform)
            .map_err(|error| error.to_string())
    }

    pub fn disconnect_channel(&self, agent_id: &str, platform: &str) -> Result<bool, String> {
        let _ = self
            .connector_secret_store()
            .remove_agent_platform(agent_id, platform)
            .map_err(|error| error.to_string())?;
        self.open_channel_store(agent_id)?
            .remove(platform)
            .map_err(|error| error.to_string())
    }

    fn existing_session_db_path(&self, agent_id: &str) -> Result<PathBuf, String> {
        let db_path = self.session_db_path(agent_id)?;
        if !db_path.is_file() {
            return Err(format!("Agent missing: {agent_id}"));
        }
        Ok(db_path)
    }

    pub fn open_agent_db_owner(
        &self,
        agent_id: &str,
    ) -> Result<Arc<SandAgentDb>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        let mut owners = self
            .db_owners
            .lock()
            .map_err(|_| "agent db owner map poisoned".to_string())?;
        if let Some(owner) = owners.get(agent_id) {
            return Ok(Arc::clone(owner));
        }
        let owner = Arc::new(
            SandAgentDb::open(&db_path, self.busy_timeout_ms)
                .map_err(|error| error.to_string())?,
        );
        owners.insert(agent_id.to_string(), Arc::clone(&owner));
        Ok(owner)
    }

    pub fn close_agent_db_owner(
        &self,
        agent_id: &str,
        checkpoint: bool,
    ) -> bool {
        let owner = self
            .db_owners
            .lock()
            .ok()
            .and_then(|mut owners| owners.remove(agent_id));
        if let Some(owner) = owner {
            owner.close(checkpoint);
            true
        } else {
            false
        }
    }

    pub fn open_agent_store_owner(
        &self,
        agent_id: &str,
    ) -> Result<Arc<ProductionAgentStore>, String> {
        if let Some(owner) = self
            .agent_store_owners
            .lock()
            .map_err(|_| "agent store owner map poisoned".to_string())?
            .get(agent_id)
            .cloned()
        {
            return Ok(owner);
        }

        let db = self.open_agent_db_owner(agent_id)?;
        let blob_store = self.create_agent_blob_store(agent_id)?;
        let candidate = Arc::new(ProductionAgentStore::new(db, blob_store));
        let _ = candidate.try_reset_from_db();

        let mut owners = self
            .agent_store_owners
            .lock()
            .map_err(|_| "agent store owner map poisoned".to_string())?;
        if let Some(owner) = owners.get(agent_id) {
            return Ok(Arc::clone(owner));
        }
        owners.insert(agent_id.to_string(), Arc::clone(&candidate));
        Ok(candidate)
    }

    pub fn close_agent_store_owner(
        &self,
        agent_id: &str,
        flush: bool,
    ) -> bool {
        let owner = self
            .agent_store_owners
            .lock()
            .ok()
            .and_then(|mut owners| owners.remove(agent_id));
        if let Some(owner) = owner {
            if flush {
                owner.flush();
            }
            true
        } else {
            false
        }
    }

    pub fn active_agent_store_owner_count(&self) -> usize {
        self.agent_store_owners
            .lock()
            .map(|owners| owners.len())
            .unwrap_or_default()
    }

    pub fn roster_extras_cache_entry_count(&self) -> usize {
        self.roster_extras_cache.entry_count()
    }

    pub fn active_agent_db_owner_count(&self) -> usize {
        self.db_owners
            .lock()
            .map(|owners| owners.len())
            .unwrap_or_default()
    }

    pub fn begin_agent_delete(&self, agent_id: &str) {
        self.roster_extras_cache.remove(agent_id);
        if let Ok(mut deleting) = self.deleting_agents.lock() {
            deleting.insert(agent_id.to_string());
        }
    }

    pub fn end_agent_delete(&self, agent_id: &str) {
        if let Ok(mut deleting) = self.deleting_agents.lock() {
            deleting.remove(agent_id);
        }
    }

    pub fn is_agent_being_deleted(&self, agent_id: &str) -> bool {
        self.deleting_agents
            .lock()
            .map(|deleting| deleting.contains(agent_id))
            .unwrap_or(false)
    }


    pub fn set_agent_sand_profile(
        &self,
        agent_id: &str,
        profile: &SandProfile,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .set_sand_profile(profile)
            .map_err(|error| error.to_string())
    }

    pub fn mark_agent_activity(&self, agent_id: &str, at: f64) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .mark_activity(at)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_unread_state(&self, agent_id: &str) -> Result<UnreadState, String> {
        self.open_agent_db_owner(agent_id)?
            .get_unread_state()
            .map_err(|error| error.to_string())
    }

    pub fn mark_agent_viewed(
        &self,
        agent_id: &str,
        at: f64,
        preserve_manual_unread: bool,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .mark_viewed(at, preserve_manual_unread)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_unread(
        &self,
        agent_id: &str,
        unread: bool,
        at: f64,
    ) -> Result<bool, String> {
        let owner = self.open_agent_db_owner(agent_id)?;
        if unread {
            owner.mark_unread(at)
        } else {
            owner.mark_read(at)
        }
        .map_err(|error| error.to_string())
    }

    pub fn get_agent_introduction_pending(&self, agent_id: &str) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .get_introduction_pending()
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_introduction_pending(
        &self,
        agent_id: &str,
        pending: bool,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .set_introduction_pending(pending)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_automation_spend_guard_state(
        &self,
        agent_id: &str,
    ) -> Result<SpendGuardState, String> {
        self.open_agent_db_owner(agent_id)?
            .get_automation_spend_guard_state()
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_automation_spend_guard_state(
        &self,
        agent_id: &str,
        state: &SpendGuardState,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .set_automation_spend_guard_state(state)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_conversation_partner_ids(
        &self,
        agent_id: &str,
    ) -> Result<Vec<String>, String> {
        self.open_agent_db_owner(agent_id)?
            .get_conversation_partner_ids()
            .map_err(|error| error.to_string())
    }

    pub fn add_agent_conversation_partner(
        &self,
        agent_id: &str,
        partner_id: &str,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .add_conversation_partner(partner_id)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_newest_divider_anchor_timestamp_ms(
        &self,
        agent_id: &str,
    ) -> Result<f64, String> {
        self.open_agent_db_owner(agent_id)?
            .get_newest_divider_anchor_timestamp_ms()
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_awaiting_user_response(
        &self,
        agent_id: &str,
    ) -> Result<Option<AwaitingUserResponse>, String> {
        self.open_agent_db_owner(agent_id)?
            .get_awaiting_user_response()
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_awaiting_user_response(
        &self,
        agent_id: &str,
        state: Option<&AwaitingUserResponse>,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .set_awaiting_user_response(state)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_awaiting_user_response_for_tab(
        &self,
        agent_id: &str,
        tab_id: &str,
        state: Option<&AwaitingUserResponse>,
        if_since_before: Option<f64>,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .set_awaiting_user_response_for_tab(tab_id, state, if_since_before)
            .map_err(|error| error.to_string())
    }

    pub fn record_agent_request_id(
        &self,
        agent_id: &str,
        request_id: &str,
        at: f64,
        prompt: Option<&str>,
        source: Option<&str>,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .record_request_id(request_id, at, prompt, source)
            .map_err(|error| error.to_string())
    }

    pub fn record_agent_episode_turn(
        &self,
        agent_id: &str,
        turn: &EpisodeTurn,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .record_episode_turn(turn)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_memory_prompt_snapshot(
        &self,
        agent_id: &str,
    ) -> Result<Option<MemoryPromptSnapshot>, String> {
        self.open_agent_db_owner(agent_id)?
            .get_memory_prompt_snapshot()
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_memory_prompt_snapshot(
        &self,
        agent_id: &str,
        snapshot: &serde_json::Value,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .set_memory_prompt_snapshot(snapshot)
            .map_err(|error| error.to_string())
    }

    pub fn clear_agent_memory_prompt_snapshot(&self, agent_id: &str) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .clear_memory_prompt_snapshot()
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_profile_prompt_snapshot(
        &self,
        agent_id: &str,
    ) -> Result<Option<serde_json::Value>, String> {
        self.open_agent_db_owner(agent_id)?
            .get_agent_profile_prompt_snapshot()
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_profile_prompt_snapshot(
        &self,
        agent_id: &str,
        snapshot: &serde_json::Value,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .set_agent_profile_prompt_snapshot(snapshot)
            .map_err(|error| error.to_string())
    }

    pub fn clear_agent_profile_prompt_snapshot(&self, agent_id: &str) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .clear_agent_profile_prompt_snapshot()
            .map_err(|error| error.to_string())
    }

    pub fn clear_agent_transient_state(&self, agent_id: &str) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .clear_transient_state()
            .map_err(|error| error.to_string())
    }

    pub fn append_agent_transcript_entries(
        &self,
        agent_id: &str,
        entries: &[serde_json::Value],
    ) -> Result<usize, String> {
        self.open_agent_db_owner(agent_id)?
            .append_transcript_entries(entries)
            .map_err(|error| error.to_string())
    }

    pub fn update_agent_transcript_entry(
        &self,
        agent_id: &str,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, String> {
        self.open_agent_db_owner(agent_id)?
            .update_transcript_entry(entry_id, next)
            .map_err(|error| error.to_string())
    }

    pub fn delete_agent_transcript_entry(
        &self,
        agent_id: &str,
        entry_id: &str,
    ) -> Result<bool, String> {
        self.open_agent_db_owner(agent_id)?
            .delete_transcript_entry(entry_id)
            .map_err(|error| error.to_string())
    }

    pub fn clear_agent_conversation(&self, agent_id: &str) -> Result<bool, String> {
        let store = self.create_agent_blob_store(agent_id)?;
        let db_path = self.existing_session_db_path(agent_id)?;
        futures::executor::block_on(self.pool.clear_blobs(
            agent_id,
            &store.blob_db_path,
            store.legacy_blob_db_path.as_deref(),
        ))
        .map_err(|error| error.to_string())?;
        self.open_agent_db_owner(agent_id)?
            .clear_conversation()
            .map_err(|error| error.to_string())
    }

    pub fn expire_pending_auto_review_approvals(
        &self,
        agent_id: &str,
        only_request_id: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        expire_pending_auto_review_approval_entries(
            &db_path,
            self.busy_timeout_ms,
            only_request_id,
        )
        .map_err(|error| error.to_string())
    }

    pub fn expire_pending_local_tool_permission_asks(
        &self,
        agent_id: &str,
        only_request_id: Option<&str>,
        if_pending_before_ms: Option<f64>,
    ) -> Result<Vec<String>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        expire_pending_local_tool_permission_ask_entries(
            &db_path,
            self.busy_timeout_ms,
            only_request_id,
            if_pending_before_ms,
        )
        .map_err(|error| error.to_string())
    }

    pub fn get_agent_profile_text(
        &self,
        agent_id: &str,
    ) -> Result<Option<SandAgentProfile>, String> {
        let db_path = self.session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        Ok(read_agent_profile_text(agent_dir))
    }

    pub fn update_agent_profile(
        &self,
        agent_id: &str,
        update: &AgentProfileUpdate,
        active_agent_id: Option<&str>,
    ) -> Result<Option<AgentSummary>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        write_agent_profile_update(agent_dir, update)
            .map_err(|error| error.to_string())?;
        self.summarize_agent_by_id(agent_id, active_agent_id)
    }

    pub fn get_agent_avatar(
        &self,
        agent_id: &str,
    ) -> Result<AgentAvatarResponse, String> {
        let db_path = self.session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        Ok(get_profile_avatar(
            agent_dir,
            &db_path,
            self.busy_timeout_ms,
        ))
    }

    pub fn get_agent_avatar_png(
        &self,
        agent_id: &str,
    ) -> Result<Option<Vec<u8>>, String> {
        let db_path = self.session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        Ok(get_profile_avatar_png(
            agent_dir,
            &db_path,
            self.busy_timeout_ms,
        ))
    }

    pub fn set_agent_avatar_bytes(
        &self,
        agent_id: &str,
        png_bytes: Option<&[u8]>,
        active_agent_id: Option<&str>,
    ) -> Result<Option<AgentSummary>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        mutate_agent_avatar_bytes(&db_path, self.busy_timeout_ms, png_bytes)?;
        self.summarize_agent_by_id(agent_id, active_agent_id)
    }


    pub fn local_human_id(&self) -> Result<&str, String> {
        self.local_human_id
            .as_deref()
            .ok_or_else(|| "local Human identity is unavailable".to_string())
    }

    pub(crate) fn metadata_has_local_human(&self, metadata: &serde_json::Value) -> Result<bool, String> {
        let local_human_id = self.local_human_id()?;
        Ok(metadata
            .get("participantIds")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(local_human_id))))
    }

    fn native_conversation_owner_key(conversation_id: &str) -> String {
        format!("conversation:{conversation_id}")
    }

    // Every Human row writer holds the existing conversation mutation lane.
    // Reconcile activity after the durable row write, and also on replay so a
    // crash between the two commits cannot leave the roster permanently stale.
    pub(crate) fn update_human_activity(owner: &SandAgentDb, timestamp_ms: f64) -> Result<(), String> {
        if owner.is_closed() {
            return Err("Human conversation owner is closed".into());
        }
        if !timestamp_ms.is_finite() {
            return Err("Human message activity timestamp must be finite".into());
        }
        let latest = owner.get_metadata("lastActivityAt")
            .map_err(|error| error.to_string())?
            .as_ref().and_then(serde_json::Value::as_f64)
            .filter(|value| value.is_finite())
            .unwrap_or(timestamp_ms).max(timestamp_ms);
        if !owner.set_metadata("lastActivityAt", serde_json::json!(latest))
            .map_err(|error| error.to_string())?
        {
            return Err("Human message activity was not durably updated".into());
        }
        Ok(())
    }

    pub(crate) fn shipping_native_messaging(&self) -> Result<Option<&FabushiNativeMessagingClient>, String> {
        match (&self.native_messaging, self.native_messaging_required) {
            (Some(client), _) => Ok(Some(client.as_ref())),
            (None, true) => Err(self.native_messaging_error.clone().unwrap_or_else(|| {
                "Fabushi native messaging is required but unavailable.".to_string()
            })),
            (None, false) => Ok(None),
        }
    }

    fn peer_human_id_from_metadata(
        &self,
        metadata: &serde_json::Value,
    ) -> Result<String, String> {
        let local = self.local_human_id()?;
        metadata
            .get("participantIds")
            .and_then(serde_json::Value::as_array)
            .and_then(|participants| {
                participants
                    .iter()
                    .filter_map(serde_json::Value::as_str)
                    .find(|participant| *participant != local)
            })
            .map(str::to_string)
            .ok_or_else(|| "Human conversation is missing its peer identity".to_string())
    }

    fn validate_human_attachment_resource(attachment: &FabushiRemoteHumanAttachment) -> Result<(), String> {
        let resource_id = attachment.resource_id.trim();
        if resource_id.is_empty()
            || resource_id.len() > 128
            || !resource_id
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
        {
            return Err("remote Human attachment resource id is invalid".into());
        }
        if attachment.size == 0 || attachment.size > 32 * 1024 * 1024 {
            return Err("remote Human attachment size exceeds the supported limit".into());
        }
        Ok(())
    }

    fn materialize_remote_human_attachment(
        &self,
        attachment: &FabushiRemoteHumanAttachment,
    ) -> Result<serde_json::Value, String> {
        Self::validate_human_attachment_resource(attachment)?;
        let resource_id = attachment.resource_id.trim();
        let mut name = attachment
            .name
            .chars()
            .map(|character| match character {
                '/' | '\\' | '\0' => '_',
                character if character.is_control() => '_',
                character => character,
            })
            .take(255)
            .collect::<String>();
        if name.trim().is_empty() {
            name = "attachment".into();
        }
        let resource_root = self
            .agents_root
            .join("human-message-resources")
            .join(resource_id);
        fs::create_dir_all(&resource_root).map_err(|error| {
            format!(
                "remote Human attachment directory could not be created: {error}"
            )
        })?;
        let path = resource_root.join(name);
        let ready = fs::metadata(&path)
            .ok()
            .is_some_and(|metadata| metadata.is_file() && metadata.len() == attachment.size);
        if !ready {
            let client = self
                .shipping_native_messaging()?
                .ok_or_else(|| "remote Human attachment requires shipping native messaging".to_string())?;
            let bytes = client.download_direct_message_resource_for_user(
                resource_id, attachment.size, self.local_human_id()?,
            )?;
            let temporary_path = resource_root.join(".download.tmp");
            fs::write(&temporary_path, &bytes).map_err(|error| {
                format!("remote Human attachment could not be staged: {error}")
            })?;
            if let Err(error) = fs::rename(&temporary_path, &path) {
                let _ = fs::remove_file(&temporary_path);
                return Err(format!(
                    "remote Human attachment could not be committed atomically: {error}"
                ));
            }
        }
        Ok(serde_json::json!({
            "path": path.to_string_lossy(),
            "name": attachment.name,
            "size": attachment.size,
            "mimeType": attachment.content_type,
            "resourceId": attachment.resource_id,
            "remoteCreatedAt": attachment.created_at,
        }))
    }

    fn validate_remote_human_message(
        &self,
        remote: &FabushiRemoteHumanMessage,
        local_human_id: &str,
        peer_human_id: &str,
        existing: &[serde_json::Value],
    ) -> Result<(), String> {
        let remote_id = fabushi_message_identity_text(&remote.id)?;
        let sender_id = fabushi_identity_text(&remote.sender_user_id)?;
        let recipient_id = fabushi_identity_text(&remote.recipient_user_id)?;
        if !((sender_id == local_human_id && recipient_id == peer_human_id)
            || (sender_id == peer_human_id && recipient_id == local_human_id))
        {
            return Err("remote Human message does not belong to the selected Human conversation".into());
        }
        if let Some(read_at) = remote.read_at.as_deref()
            .map(str::trim).filter(|value| !value.is_empty())
        {
            chrono::DateTime::parse_from_rfc3339(read_at)
                .map_err(|error| format!("remote Human message read timestamp is invalid: {error}"))?;
        }

        let canonical_id = format!("human-server-message:{remote_id}");
        let by_id = existing.iter().find(|row| {
            row.get("id").and_then(serde_json::Value::as_str) == Some(canonical_id.as_str())
        });
        if by_id.is_some_and(|row| {
            row.get("authorId").and_then(serde_json::Value::as_str) != Some(sender_id.as_str())
        }) {
            return Err("remote Human canonical identity conflicts with the persisted sender".into());
        }
        if let Some(reactions) = remote.reactions.as_deref() {
            Self::validate_remote_human_reactions(reactions)?;
        }

        // Peer request provenance can never acknowledge a local intent. On our own
        // outgoing echo, both canonical-ID and nonce lookups must pass the same
        // immutable local-send checks, even when a replay omits/changes the nonce.
        if sender_id != local_human_id {
            return Ok(());
        }
        let nonce = remote.client_request_id.as_deref()
            .map(str::trim).filter(|value| !value.is_empty());
        let by_nonce = nonce.and_then(|nonce| existing.iter().find(|row| {
            row.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id)
                && row.get("clientNonce").and_then(serde_json::Value::as_str) == Some(nonce)
        }));
        for current in [by_id, by_nonce].into_iter().flatten() {
            if current.get("authorId").and_then(serde_json::Value::as_str) != Some(local_human_id)
                || current.get("clientNonce").and_then(serde_json::Value::as_str) != nonce
                || nonce.is_none()
                || current.get("content").and_then(serde_json::Value::as_str)
                    != Some(remote.text.trim())
            {
                return Err("remote Human acknowledgement conflicts with the local send identity or text".into());
            }
            if let Some(prior_remote) = current.get("remoteMessageId") {
                if fabushi_identity_text(prior_remote)? != remote_id {
                    return Err("remote Human acknowledgement changed a settled remote identity".into());
                }
            }
            let expected_reply = if let Some(id) = current.get("remoteReplyToMessageId") {
                Some(fabushi_identity_text(id)?)
            } else if let Some(reply_id) = current.get("replyToId")
                .and_then(serde_json::Value::as_str)
            {
                let target = existing.iter().find(|row| {
                    row.get("id").and_then(serde_json::Value::as_str) == Some(reply_id)
                }).ok_or_else(|| "remote Human acknowledgement reply target disappeared".to_string())?;
                Some(fabushi_identity_text(target.get("remoteMessageId").ok_or_else(|| {
                    "remote Human acknowledgement reply has no settled remote identity".to_string()
                })?)?)
            } else {
                None
            };
            let actual_reply = remote.reply_to_message_id.as_ref()
                .map(fabushi_identity_text).transpose()?;
            if expected_reply != actual_reply {
                return Err("remote Human acknowledgement changed the dispatched reply".into());
            }

            let previously_dispatched = current.get("remoteDispatchAttachments")
                .or_else(|| current.get("attachments"));
            let mut expected_resources = Vec::new();
            if let Some(attachments) = previously_dispatched {
                let attachments = attachments.as_array().ok_or_else(|| {
                    "persisted Human dispatched resources are not an array".to_string()
                })?;
                for attachment in attachments {
                    expected_resources.push(
                        attachment.get("resourceId")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| "persisted Human attachment lacks dispatched resource identity".to_string())?
                            .to_string()
                    );
                }
            }
            let actual_resources = remote.attachments.iter()
                .map(|attachment| attachment.resource_id.as_str()).collect::<Vec<_>>();
            if expected_resources.iter().map(String::as_str).collect::<Vec<_>>() != actual_resources {
                return Err("remote Human acknowledgement changed dispatched resources".into());
            }
        }
        Ok(())
    }

    fn materialize_remote_human_message(
        &self,
        owner: &SandAgentDb,
        remote: &FabushiRemoteHumanMessage,
    ) -> Result<serde_json::Value, String> {
        let existing = owner.get_transcript_entries().map_err(|error| error.to_string())?;
        self.materialize_remote_human_message_with_snapshot(owner, remote, &existing)
    }

    // The caller owns the conversation mutation lane and updates this snapshot
    // only after each durable write. Avoid reparsing the entire SQLite transcript
    // for every row of a bounded history page.
    fn materialize_remote_human_message_with_snapshot(
        &self,
        owner: &SandAgentDb,
        remote: &FabushiRemoteHumanMessage,
        existing: &[serde_json::Value],
    ) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?;
        let remote_id = fabushi_message_identity_text(&remote.id)?;
        let sender_id = fabushi_identity_text(&remote.sender_user_id)?;
        let recipient_id = fabushi_identity_text(&remote.recipient_user_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        let peer_human_id = self.peer_human_id_from_metadata(&metadata)?;
        self.validate_remote_human_message(remote, local_human_id, &peer_human_id, existing)?;
        let timestamp_ms = chrono::DateTime::parse_from_rfc3339(&remote.created_at)
            .map_err(|error| format!("remote Human message timestamp is invalid: {error}"))?
            .timestamp_millis() as f64;
        let entry_id = format!("human-server-message:{remote_id}");
        if sender_id == local_human_id {
            // Validate only this message's existing local provenance. A corrupt
            // intent is retained; it cannot be overwritten by a remote snapshot
            // that carries no evidence of the original queue observation.
            let local_origin = existing.iter().find(|candidate|
                candidate.get("id").and_then(serde_json::Value::as_str) == Some(entry_id.as_str())
                    && candidate.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id))
                .or_else(|| remote.client_request_id.as_deref().map(str::trim)
                    .filter(|nonce| !nonce.is_empty()).and_then(|nonce| existing.iter().find(|candidate|
                        candidate.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id)
                            && candidate.get("clientNonce").and_then(serde_json::Value::as_str) == Some(nonce))));
            if let Some(current) = local_origin {
                Self::validate_human_queue_origin(current)?;
            }
        }
        let mut entry = serde_json::json!({
            "id": entry_id,
            "kind": "message",
            "role": "user",
            "authorKind": "human",
            "authorId": sender_id,
            "content": remote.text,
            "timestampMs": timestamp_ms,
            "delivery": "sent",
            "remoteMessageId": remote_id,
            "remoteCreatedAt": remote.created_at,
        });
        let object = entry
            .as_object_mut()
            .ok_or_else(|| "remote Human message projection failed".to_string())?;
        if let Some(author_name) = remote
            .sender_username
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            object.insert("authorName".into(), serde_json::json!(author_name));
        }
        if let Some(client_nonce) = remote
            .client_request_id
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            if sender_id == local_human_id {
                object.insert("clientNonce".into(), serde_json::json!(client_nonce));
            } else {
                object.insert("remoteClientRequestId".into(), serde_json::json!(client_nonce));
            }
        }
        if let Some(read_at) = remote
            .read_at
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            object.insert("readAt".into(), serde_json::json!(read_at));
        }
        if let Some(reply_to_message_id) = remote
            .reply_to_message_id
            .as_ref()
            .map(fabushi_identity_text)
            .transpose()?
        {
            object.insert(
                "replyToId".into(),
                serde_json::json!(format!("human-server-message:{reply_to_message_id}")),
            );
            object.insert(
                "remoteReplyToMessageId".into(),
                serde_json::json!(reply_to_message_id),
            );
        }
        if !remote.attachments.is_empty() {
            let attachments = remote
                .attachments
                .iter()
                .map(|attachment| self.materialize_remote_human_attachment(attachment))
                .collect::<Result<Vec<_>, _>>()?;
            object.insert("attachments".into(), serde_json::Value::Array(attachments));
        }
        let peer_human_id = if sender_id == local_human_id {
            recipient_id.as_str()
        } else {
            sender_id.as_str()
        };
        if let Some(reactions) = remote.reactions.as_deref() {
            let projected_reactions =
                Self::project_remote_human_reactions(reactions, peer_human_id);
            object.insert("remoteReactions".into(), serde_json::to_value(reactions)
                .map_err(|error| format!("remote Human reactions could not be projected: {error}"))?);
            object.insert("reactions".into(), serde_json::Value::Array(projected_reactions));
        }
        if let Some(current) = existing.iter().find(|candidate| {
            candidate.get("id").and_then(serde_json::Value::as_str)
                == entry.get("id").and_then(serde_json::Value::as_str)
        }) {
            // A remote refresh must not erase local identity or delivery context.
            if let (Some(next), Some(prior)) = (entry.as_object_mut(), current.as_object()) {
                for (key, value) in prior {
                    if !next.contains_key(key) {
                        next.insert(key.clone(), value.clone());
                    }
                }
            }
            if let Some(read_at) = current.get("readAt").and_then(serde_json::Value::as_str) {
                let prior = chrono::DateTime::parse_from_rfc3339(read_at).ok();
                let newer = remote.read_at.as_deref()
                    .map(str::trim).filter(|value| !value.is_empty())
                    .and_then(|value| chrono::DateTime::parse_from_rfc3339(value).ok())
                    .is_some_and(|incoming| prior.is_none_or(|prior| incoming > prior));
                if !newer {
                    entry.as_object_mut().expect("entry is an object")
                        .insert("readAt".into(), serde_json::json!(read_at));
                }
            }
            if current == &entry {
                Self::update_human_activity(owner, timestamp_ms)?;
                return Ok(current.clone());
            }
            let refreshed = owner
                .update_transcript_entry(
                    entry
                        .get("id")
                        .and_then(serde_json::Value::as_str)
                        .ok_or_else(|| "remote Human message projection omitted its id".to_string())?,
                    &entry,
                )
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "remote Human message could not be durably refreshed".to_string())?;
            Self::update_human_activity(owner, timestamp_ms)?;
            return Ok(refreshed);
        }
        if let Some(client_nonce) = entry.get("clientNonce").and_then(serde_json::Value::as_str) {
            if let Some(current) = existing.iter().find(|candidate| {
                candidate.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id)
                    && candidate.get("clientNonce").and_then(serde_json::Value::as_str) == Some(client_nonce)
            }) {
                if let Some(context) = current.get("forwardContext").cloned() {
                    entry
                        .as_object_mut()
                        .ok_or_else(|| "remote Human message projection failed".to_string())?
                        .insert("forwardContext".into(), context);
                }
                let current_id = current.get("id").and_then(serde_json::Value::as_str)
                    .ok_or_else(|| "local Human settlement row omitted its id".to_string())?;
                let current_remote = current.get("remoteMessageId").and_then(serde_json::Value::as_str);
                if let Some(current_remote) = current_remote {
                    if current_remote != remote_id {
                        return Err("local Human settlement row conflicts with a different remote identity".into());
                    }
                } else if !matches!(
                    current.get("delivery").and_then(serde_json::Value::as_str),
                    Some("pending" | "dispatching")
                ) {
                    return Err("local Human message already uses the server clientRequestId without a retryable settlement state".into());
                }
                let origin = Self::validate_human_queue_origin(current)?;
                let object = entry.as_object_mut().expect("remote Human message is an object");
                if let Some(origin) = origin {
                    object.insert("queuedOffline".into(), serde_json::json!(origin));
                }
                if let Some(composed) = current.get("composedAtMs").filter(|value|
                    value.as_f64().is_some_and(f64::is_finite))
                {
                    object.insert("composedAtMs".into(), composed.clone());
                }
                let settled = owner
                    .settle_transcript_entry_identity(current_id, &entry)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "local Human message disappeared before remote identity settlement".to_string())?;
                Self::update_human_activity(owner, timestamp_ms)?;
                return Ok(settled);
            }
        }
        if !owner
            .append_transcript_entry(&entry)
            .map_err(|error| error.to_string())?
        {
            let replay = owner
                .get_transcript_entries()
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|candidate| candidate.get("id") == entry.get("id"))
                .ok_or_else(|| "remote Human message was not durably materialized".to_string())?;
            let timestamp = replay.get("timestampMs").and_then(serde_json::Value::as_f64)
                .unwrap_or(timestamp_ms);
            Self::update_human_activity(owner, timestamp)?;
            return Ok(replay);
        }
        Self::update_human_activity(owner, timestamp_ms)?;
        Ok(entry)
    }

    pub(crate) fn human_message_lock(&self, conversation_id: &str) -> Result<Arc<Mutex<()>>, String> {
        let mut locks = self.human_message_locks.lock()
            .map_err(|_| "Human conversation mutation lock registry poisoned".to_string())?;
        Ok(Arc::clone(locks.entry(conversation_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))))
    }

    pub(crate) fn acquire_human_send_settlement_guard<'a>(
        client: &FabushiNativeMessagingClient,
        serial: &'a Mutex<()>,
        held_conversation_guard: Option<&std::sync::MutexGuard<'_, ()>>,
        request_credential_fingerprint: &str,
        response: &Result<FabushiRemoteHumanMessage, FabushiHumanSendError>,
    ) -> Result<Option<std::sync::MutexGuard<'a, ()>>, String> {
        let guard = if held_conversation_guard.is_none() {
            Some(serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?)
        } else {
            None
        };
        // A completed HTTP/credential rejection already has a causal refusal.
        // It cannot enter transport recovery or be replaced by a later lease
        // error while waiting for this conversation's mutation lane.
        if let Err(FabushiHumanSendError::Rejected(error)) = response {
            return Err(error.clone());
        }
        // Successful and ambiguous transport outcomes may settle only while
        // every private credential field still matches the dispatch snapshot.
        if client.request_credential_fingerprint()? != request_credential_fingerprint {
            return Err("Fabushi native messaging credential changed before Human settlement".into());
        }
        Ok(guard)
    }

    pub(crate) fn update_human_replay_snapshot(
        snapshot: &mut Vec<serde_json::Value>,
        settled: serde_json::Value,
    ) {
        let prior = snapshot.iter().position(|entry| {
            entry.get("id") == settled.get("id")
                || (settled.get("clientNonce").is_some()
                    && entry.get("clientNonce") == settled.get("clientNonce")
                    && entry.get("authorId") == settled.get("authorId"))
        });
        if let Some(index) = prior {
            snapshot[index] = settled;
        } else {
            snapshot.push(settled);
        }
    }

    pub fn sync_human_conversations(&self) -> Result<Vec<serde_json::Value>, String> {
        let Some(client) = self.shipping_native_messaging()? else {
            return self.list_human_conversations();
        };
        let local_human_id = self.local_human_id()?;
        let identity = client.identity()?;
        if identity.user_id != local_human_id {
            return Err(format!(
                "Fabushi native messaging credential user {} does not match active Human identity {}",
                identity.user_id,
                local_human_id
            ));
        }
        let mut recovery = self.capture_human_sync_recovery(client)?;
        self.resume_human_forward_batches()?;
        self.check_human_sync_binding(client, &recovery)?;
        let friends = client.list_friends_for_user(local_human_id)?;
        self.check_human_sync_continuation(Some(&recovery))?;
        let mut synced_conversations = BTreeSet::new();
        for friend in friends {
            // Conversation discovery itself can create a durable store.
            self.check_human_sync_continuation(Some(&recovery))?;
            let peer_human_id = fabushi_identity_text(&friend.user_id)?;
            let expected_id = Self::human_direct_conversation_id(local_human_id, &peer_human_id);
            // Duplicate roster rows cannot grant another reaction attempt or
            // another 32-page history budget for the same canonical scope.
            if !synced_conversations.insert(expected_id) { continue; }
            let title = friend.display_name.trim();
            let conversation = self.create_human_conversation_with_recovery(
                &peer_human_id,
                if title.is_empty() { &friend.username } else { title },
                Some(&recovery),
            )?;
            let conversation_id = conversation
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "synced Human conversation omitted its id".to_string())?;
            // The top-level roster invocation already recovered the outbox.
            // A failed target gets one attempt per invocation, not per friend.
            self.sync_human_conversation_history(conversation_id, Some(&mut recovery))?;
        }
        self.check_human_sync_continuation(Some(&recovery))?;
        self.sync_human_calls_with_recovery(Some(&recovery))?;
        self.check_human_sync_continuation(Some(&recovery))?;
        recovery.finish(self.list_human_conversations()?)
    }

    fn human_direct_conversation_id(first: &str, second: &str) -> String {
        let mut participants = [first, second];
        participants.sort();
        let mut digest = Sha256::new();
        for participant in participants {
            digest.update((participant.len() as u64).to_be_bytes());
            digest.update(participant.as_bytes());
        }
        format!("human-direct-{:x}", digest.finalize())
    }

    pub fn append_human_agent_message(
        &self,
        conversation_id: &str,
        agent_id: &str,
        stream_id: &str,
        content: &str,
        completed_at_ms: f64,
    ) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?;
        let conversation_id = conversation_id.trim();
        let agent_id = agent_id.trim();
        let stream_id = stream_id.trim();
        let content = content.trim();
        if conversation_id.is_empty()
            || agent_id.is_empty()
            || stream_id.is_empty()
            || content.is_empty()
        {
            return Err(
                "Human handoff Agent result requires conversationId, agentId, streamId, and content"
                    .into(),
            );
        }
        if !completed_at_ms.is_finite() {
            return Err("Human handoff Agent activity timestamp must be finite".into());
        }
        let serial = self.human_message_lock(conversation_id)?;
        let _exclusive = serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?;
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        let is_participant = metadata
            .get("participantIds")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(local_human_id)));
        if !is_participant {
            return Err("local Human identity is not a conversation participant".into());
        }

        let entry_id = format!("human-agent-message:{stream_id}");
        let existing = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        if let Some(entry) = existing.iter().find(|entry| {
            entry.get("id").and_then(serde_json::Value::as_str) == Some(entry_id.as_str())
        }) {
            let same_agent =
                entry.get("authorId").and_then(serde_json::Value::as_str) == Some(agent_id);
            let same_content =
                entry.get("content").and_then(serde_json::Value::as_str) == Some(content);
            let same_stream =
                entry.get("sourceStreamId").and_then(serde_json::Value::as_str) == Some(stream_id);
            if same_agent && same_content && same_stream {
                let timestamp = entry.get("timestampMs").and_then(serde_json::Value::as_f64)
                    .unwrap_or(completed_at_ms);
                Self::update_human_activity(&owner, timestamp)?;
                return Ok(entry.clone());
            }
            return Err("Human handoff stream already identifies different Agent content".into());
        }

        let entry = serde_json::json!({
            "id": entry_id,
            "kind": "message",
            "role": "assistant",
            "authorKind": "agent",
            "authorId": agent_id,
            "authorName": agent_id,
            "content": content,
            "sourceStreamId": stream_id,
            "timestampMs": completed_at_ms,
            "delivery": "sent",
        });
        if !owner
            .append_transcript_entry(&entry)
            .map_err(|error| error.to_string())?
        {
            let replay = owner
                .get_transcript_entries()
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|candidate| candidate.get("id") == entry.get("id"))
                .ok_or_else(|| "Human handoff Agent result was not durably appended".to_string())?;
            let same_agent =
                replay.get("authorId").and_then(serde_json::Value::as_str) == Some(agent_id);
            let same_content =
                replay.get("content").and_then(serde_json::Value::as_str) == Some(content);
            let same_stream =
                replay.get("sourceStreamId").and_then(serde_json::Value::as_str) == Some(stream_id);
            if same_agent && same_content && same_stream {
                let timestamp = replay.get("timestampMs").and_then(serde_json::Value::as_f64)
                    .unwrap_or(completed_at_ms);
                Self::update_human_activity(&owner, timestamp)?;
                return Ok(replay);
            }
            return Err("Human handoff stream already identifies different Agent content".into());
        }
        Self::update_human_activity(&owner, completed_at_ms)?;
        Ok(entry)
    }

    pub fn search_human_messages(
        &self,
        conversation_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>, String> {
        let query = query.trim().to_lowercase();
        if query.is_empty() {
            return Err("searchHumanMessages requires a non-empty query".into());
        }
        let transcript = self.read_human_conversation_transcript(conversation_id)?;
        let limit = limit.clamp(1, 200);
        let mut matches = Vec::new();
        for entry in transcript.into_iter().rev() {
            let mut fields = Vec::new();
            if let Some(author_name) = entry
                .get("authorName")
                .and_then(serde_json::Value::as_str)
            {
                fields.push(author_name);
            }
            if let Some(origin_author) = Self::human_entry_visible_forward_origin_author(&entry) {
                fields.push(origin_author);
            }
            if let Some(content) = entry.get("content").and_then(serde_json::Value::as_str) {
                fields.push(content);
            }
            if let Some(attachments) = entry
                .get("attachments")
                .and_then(serde_json::Value::as_array)
            {
                for attachment in attachments {
                    if let Some(name) = attachment
                        .get("name")
                        .and_then(serde_json::Value::as_str)
                    {
                        fields.push(name);
                    }
                }
            }
            if fabushi_messaging_core::query_matches_fields(&query, &fields) {
                matches.push(entry);
                if matches.len() >= limit {
                    break;
                }
            }
        }
        Ok(matches)
    }

    // Post-commit projection may run while the writer still owns the Human
    // conversation mutation lane. Read only its already registered SQLite
    // owner; a notification must never open/create a store or replay history.
    pub(crate) fn read_human_transcript_mutation_entries(
        &self,
        conversation_id: &str,
        origin_db_path: &str,
        entry_ids: &[String],
    ) -> Result<Vec<serde_json::Value>, String> {
        if origin_db_path.is_empty() {
            return Err("Human transcript mutation has no qualified origin".into());
        }
        let expected_path = get_native_conversation_db_path(&self.agents_root, conversation_id)
            .map_err(|error| error.to_string())?;
        let key = Self::native_conversation_owner_key(conversation_id);
        let owner = {
            let owners = self.db_owners.lock()
                .map_err(|_| "conversation db owner map poisoned".to_string())?;
            owners.get(&key).cloned()
        }.ok_or_else(|| "Human transcript mutation has no registered owner".to_string())?;
        // Both paths originate in this Session, never in the notification.
        let expected_origin =
            crate::transcript_mutation_events::transcript_mutation_origin(&expected_path)
                .ok_or_else(|| "Human transcript owner origin is unavailable".to_string())?;
        if origin_db_path != expected_origin.as_str()
            || crate::transcript_mutation_events::transcript_mutation_origin(owner.db_path())
                .as_deref() != Some(expected_origin.as_str())
        {
            return Err("Human transcript mutation belongs to a different database".into());
        }
        let metadata = Self::read_human_conversation_metadata(&owner, conversation_id, true)?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        let mut seen = BTreeSet::new();
        let mut current = Vec::new();
        for entry_id in entry_ids {
            if entry_id.is_empty() || entry_id.trim() != entry_id.as_str() {
                return Err("Human transcript mutation has an invalid entry identity".into());
            }
            if !seen.insert(entry_id.as_str()) {
                continue;
            }
            let Some(entry) = owner.get_transcript_entry_by_id(entry_id)
                .map_err(|error| error.to_string())? else {
                continue;
            };
            if entry.get("id").and_then(serde_json::Value::as_str) != Some(entry_id.as_str()) {
                return Err("Human transcript entry conflicts with its persisted identity".into());
            }
            current.push(entry);
        }
        Ok(current)
    }

    pub fn read_human_conversation_transcript(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        self.read_human_conversation_transcript_for_topic(conversation_id, None)
    }

    pub fn read_human_conversation_transcript_tail(
        &self,
        conversation_id: &str,
        query: impl Into<HumanTranscriptPageQuery>,
    ) -> Result<HumanTranscriptPage, String> {
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        owner
            .get_human_transcript_tail(query.into())
            .map_err(|error| error.to_string())
    }

    pub fn list_agent_summaries(
        &self,
        active_agent_id: Option<&str>,
    ) -> Result<Vec<AgentSummary>, String> {
        let is_deleting = |agent_id: &str| self.is_agent_being_deleted(agent_id);
        list_agents(
            &self.agents_root,
            self.busy_timeout_ms,
            active_agent_id,
            &is_deleting,
            &self.roster_extras_cache,
        )
    }

    pub fn summarize_agent_by_id(
        &self,
        agent_id: &str,
        active_agent_id: Option<&str>,
    ) -> Result<Option<AgentSummary>, String> {
        let is_deleting = |candidate: &str| self.is_agent_being_deleted(candidate);
        summarize_agent_by_id(
            &self.agents_root,
            self.busy_timeout_ms,
            agent_id,
            active_agent_id,
            &is_deleting,
            &self.roster_extras_cache,
        )
    }

    pub fn create_agent_blob_store(
        &self,
        agent_id: &str,
    ) -> Result<ProductionWorkerBlobStore, String> {
        let session_db_path = self.session_db_path(agent_id)?;
        Ok(ProductionWorkerBlobStore::new(
            Arc::clone(&self.pool),
            agent_id,
            conversation_blobs_path(&session_db_path),
            Some(session_db_path),
        ))
    }

    /// Opens the concrete SQLite conversation-blob backend for an already
    /// materialized production agent before Runner inference begins.
    ///
    /// Missing session DBs are not created here: session creation remains owned
    /// by the Session extension. Existing sessions, however, now traverse the
    /// real AgentWorkerPool and legacy-adoption backend on the shipping path.
    pub fn prepare_existing_agent(
        &self,
        agent_id: &str,
    ) -> Result<Option<PreparedAgentBlobStore>, String> {
        let Some(materialized) = open_existing_session(
            &self.agents_root,
            self.busy_timeout_ms,
            agent_id,
        )
        .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let db_owner = self.open_agent_db_owner(agent_id)?;
        let store = self.create_agent_blob_store(agent_id)?;
        let session_db_path = materialized.db_path.clone();

        let recovered_root = recover_conversation_root_if_missing(
            Arc::clone(&self.pool),
            agent_id,
            &session_db_path,
            &store.blob_db_path,
            self.busy_timeout_ms,
        )?;
        if recovered_root {
            let _ = sync_recovered_profile_name(
                &session_db_path,
                self.busy_timeout_ms,
                &materialized.profile.name,
            )?;
        }
        let recovery_turns = if recovered_root {
            self.conversation_state
                .read_agent_recovery_outline_turns(
                    Arc::clone(&self.pool),
                    agent_id,
                    &session_db_path,
                    &store.blob_db_path,
                )
                .map_err(|error| error.to_string())?
        } else {
            self.conversation_state
                .read_agent_recovery_outline_turns(
                    Arc::clone(&self.pool),
                    agent_id,
                    &session_db_path,
                    &store.blob_db_path,
                )
                .unwrap_or_default()
        };
        if !recovery_turns.is_empty() {
            let _ = backfill_transcript_from_outline(
                &session_db_path,
                self.busy_timeout_ms,
                &recovery_turns,
            )?;
        }
        let recovery_outline = recovery_turns
            .iter()
            .flat_map(|turn| turn.iter().cloned())
            .collect::<Vec<_>>();
        let _ = repair_hidden_transcript_entries_once(
            &session_db_path,
            self.busy_timeout_ms,
            &recovery_outline,
        )?;
        let persisted_root_blob_id = db_owner
            .get_latest_root_blob_id()
            .map_err(|error| error.to_string())?;
        let session_state = db_owner
            .serde_snapshot()
            .map_err(|error| error.to_string())?;
        let transcript_tail = db_owner
            .get_transcript_tail(TranscriptWindowQuery {
                before_seq: None,
                limit: 500,
            })
            .map_err(|error| error.to_string())?;
        let profile_file = Some(materialized.profile.clone());

        let latest_root_blob_id = futures::executor::block_on(
            self.pool.find_latest_root_blob_id(
                agent_id,
                &store.blob_db_path,
                store.legacy_blob_db_path.as_deref(),
            ),
        )
        .map_err(|error| error.to_string())?;

        if let Some(blob_id) = latest_root_blob_id.as_deref() {
            let _ = futures::executor::block_on(store.get_blob(&(), blob_id))
                .map_err(|error| error.to_string())?;
        }
        let _ = clear_stale_checkpoint_roots_once(
            Arc::clone(&self.pool),
            agent_id,
            &session_db_path,
            &store.blob_db_path,
            self.busy_timeout_ms,
        )?;
        let _ = retire_legacy_store_blobs_once(
            Arc::clone(&self.pool),
            agent_id,
            &session_db_path,
            &store.blob_db_path,
            self.busy_timeout_ms,
        )?;

        let prepared = PreparedAgentBlobStore {
            agent_id: agent_id.to_string(),
            session_db_path,
            blob_db_path: store.blob_db_path.clone(),
            latest_root_blob_id,
            persisted_root_blob_id,
            session_state,
            transcript_tail,
            profile_file,
        };
        let _ = self.schedule_conversation_size_maintenance(
            &prepared,
            ConversationSizePolicy::from_environment(),
        );
        Ok(Some(prepared))
    }

    pub fn schedule_conversation_size_maintenance(
        &self,
        prepared: &PreparedAgentBlobStore,
        policy: ConversationSizePolicy,
    ) -> bool {
        self.conversation_size_maintenance
            .schedule_conversation_size_maintenance(
                Arc::clone(&self.pool),
                ConversationGcTarget::from_root(
                    prepared.agent_id.clone(),
                    prepared.blob_db_path.clone(),
                    prepared.session_db_path.clone(),
                    &prepared.persisted_root_blob_id,
                ),
                policy,
            )
    }

    pub fn ensure_capacity_for_turn(
        &self,
        prepared: &PreparedAgentBlobStore,
    ) -> Result<(), String> {
        self.conversation_size_maintenance
            .ensure_conversation_capacity_for_turn(
                Arc::clone(&self.pool),
                ConversationGcTarget::from_root(
                    prepared.agent_id.clone(),
                    prepared.blob_db_path.clone(),
                    prepared.session_db_path.clone(),
                    &prepared.persisted_root_blob_id,
                ),
                ConversationSizePolicy::from_environment(),
            )
            .map_err(|error| error.to_string())
    }

    pub fn active_worker_count(&self) -> usize {
        self.pool.active_worker_count()
    }

    pub fn shutdown(&self) {
        self.shutdown_inner(false);
    }

    pub fn shutdown_with_checkpoint(&self) {
        self.shutdown_inner(true);
    }

    fn shutdown_inner(&self, checkpoint: bool) {
        let agent_stores = self
            .agent_store_owners
            .lock()
            .map(|mut owners| std::mem::take(&mut *owners).into_values().collect::<Vec<_>>())
            .unwrap_or_default();
        for owner in &agent_stores {
            owner.flush();
        }
        drop(agent_stores);

        let db_owners = self
            .db_owners
            .lock()
            .map(|mut owners| std::mem::take(&mut *owners).into_values().collect::<Vec<_>>())
            .unwrap_or_default();
        for owner in db_owners {
            owner.close(checkpoint);
        }
        futures::executor::block_on(self.pool.close_all());
    }
}


#[cfg(test)]
#[path = "human_creation_tests.rs"]
mod human_creation_tests;

#[cfg(test)]
mod sharebox_shipping_tests {
    use super::*;

    fn test_workers(root: &Path) -> ProductionSessionWorkers {
        ProductionSessionWorkers::with_agents_root_identity_and_dependencies(
            root.to_path_buf(),
            2_000,
            Some("human-local".to_string()),
            Arc::new(|| None),
            Arc::new(MemoryService::new(root.to_path_buf())),
        )
    }

    fn conversation_id(value: &serde_json::Value) -> String {
        value.get("id").and_then(serde_json::Value::as_str).unwrap().to_string()
    }

    #[test]
    fn shipping_human_forward_reuses_session_owner_across_retry_and_restart() {
        let root = std::env::temp_dir().join(format!("fabushi-forward-{}", Uuid::new_v4()));
        let workers = test_workers(&root);
        let source = conversation_id(&workers.create_human_conversation("human-source", "Source").unwrap());
        let alpha = conversation_id(&workers.create_human_conversation("human-alpha", "Alpha").unwrap());
        let beta = conversation_id(&workers.create_human_conversation("human-beta", "Beta").unwrap());

        let source_entry = workers
            .append_human_message(&source, "forward me", "source-nonce", Some(10.0), None, &[])
            .unwrap();
        let source_entry_id = source_entry.get("id").and_then(serde_json::Value::as_str).unwrap();

        let search = workers
            .search_human_recipients(&source, source_entry_id, "beta", 50)
            .unwrap();
        assert_eq!(search.len(), 1);
        assert_eq!(search[0].get("id").and_then(serde_json::Value::as_str), Some(beta.as_str()));

        let missing = "human-direct-does-not-exist".to_string();
        let destinations = vec![alpha.clone(), missing.clone(), alpha.clone()];
        let first = workers
            .forward_human_message(&source, source_entry_id, &destinations, "forward-request-1", false, true)
            .unwrap();
        let settlement = first.get("destinations").and_then(serde_json::Value::as_array).unwrap();
        assert_eq!(settlement.len(), 2, "duplicate destinations must collapse before fan-out");
        assert_eq!(settlement[0].get("status").and_then(serde_json::Value::as_str), Some("sent"));
        assert_eq!(settlement[1].get("status").and_then(serde_json::Value::as_str), Some("failed"));

        let alpha_entries = workers.read_human_conversation_transcript(&alpha).unwrap();
        assert_eq!(alpha_entries.len(), 1);
        let forwarded = &alpha_entries[0];
        let context = forwarded.get("forwardContext").unwrap();
        assert_eq!(context.pointer("/privacy/dropSenderNames").and_then(serde_json::Value::as_bool), Some(true));
        assert_eq!(context.pointer("/privacy/dropCaptions").and_then(serde_json::Value::as_bool), Some(true));
        assert!(context.get("origin").is_none(), "drop captions must also suppress sender provenance");

        let replay = workers
            .forward_human_message(&source, source_entry_id, &destinations, "forward-request-1", false, true)
            .unwrap();
        assert_eq!(
            replay.pointer("/destinations/0/status").and_then(serde_json::Value::as_str),
            Some("sent"),
        );
        assert_eq!(workers.read_human_conversation_transcript(&alpha).unwrap().len(), 1);

        let changed = workers
            .forward_human_message(&source, source_entry_id, &[alpha.clone()], "forward-request-1", false, false)
            .expect_err("privacy reuse cannot change a persisted batch");
        assert!(changed.contains("different privacy"));

        workers.shutdown();

        let restarted = test_workers(&root);
        let replay_after_restart = restarted
            .forward_human_message(&source, source_entry_id, &[alpha.clone()], "forward-request-1", false, true)
            .unwrap();
        assert_eq!(
            replay_after_restart.pointer("/destinations/0/status").and_then(serde_json::Value::as_str),
            Some("sent"),
        );
        assert_eq!(restarted.read_human_conversation_transcript(&alpha).unwrap().len(), 1);

        // Destination permission & typed-share policy enforcement across searchHumanRecipients and forwardHumanMessage
        let gamma = conversation_id(&restarted.create_human_conversation("human-gamma", "Gamma Blocked").unwrap());
        let gamma_owner = restarted.open_human_conversation_db_owner(&gamma).unwrap();
        assert!(gamma_owner.set_metadata("permissions", serde_json::json!({ "canSendMessages": false })).unwrap());

        let delta = conversation_id(&restarted.create_human_conversation("human-delta", "Delta NoOther").unwrap());
        let delta_owner = restarted.open_human_conversation_db_owner(&delta).unwrap();
        assert!(delta_owner.set_metadata("permissions", serde_json::json!({ "canSendOther": false })).unwrap());

        let source_owner = restarted.open_human_conversation_db_owner(&source).unwrap();
        assert!(source_owner.append_transcript_entry(&serde_json::json!({
            "id": "human-server-message:901",
            "kind": "message",
            "role": "user",
            "authorKind": "human",
            "authorId": "human-source",
            "content": "shared link with send-other requirement",
            "requireSendOther": true,
            "delivery": "sent",
            "timestampMs": 25.0,
            "remoteMessageId": "901",
            "remoteCreatedAt": "1970-01-01T00:00:25Z"
        })).unwrap());

        let ordinary_recipients = restarted
            .search_human_recipients(&source, source_entry_id, "", 50)
            .unwrap();
        assert!(
            ordinary_recipients.iter().all(|item| item.get("id").and_then(serde_json::Value::as_str) != Some(gamma.as_str())),
            "canSendMessages=false destination must be excluded before exposure",
        );
        assert!(
            ordinary_recipients.iter().any(|item| item.get("id").and_then(serde_json::Value::as_str) == Some(delta.as_str())),
            "canSendOther=false destination remains eligible for ordinary text forwards",
        );

        let other_recipients = restarted
            .search_human_recipients(&source, "human-server-message:901", "", 50)
            .unwrap();
        assert!(
            other_recipients.iter().all(|item| item.get("id").and_then(serde_json::Value::as_str) != Some(gamma.as_str())),
            "canSendMessages=false destination must remain excluded",
        );
        assert!(
            other_recipients.iter().all(|item| item.get("id").and_then(serde_json::Value::as_str) != Some(delta.as_str())),
            "canSendOther=false destination must be excluded when requireSendOther=true",
        );
        assert!(
            other_recipients.iter().any(|item| item.get("id").and_then(serde_json::Value::as_str) == Some(beta.as_str())),
            "unrestricted destination remains eligible when requireSendOther=true",
        );

        let policy_settlement = restarted
            .forward_human_message(
                &source,
                "human-server-message:901",
                &[beta.clone(), gamma.clone(), delta.clone()],
                "forward-request-policy-1",
                false,
                false,
            )
            .unwrap();
        let policy_dests = policy_settlement.get("destinations").and_then(serde_json::Value::as_array).unwrap();
        assert_eq!(policy_dests.len(), 3);
        let status_for = |conv_id: &str| {
            policy_dests
                .iter()
                .find(|item| item.get("conversationId").and_then(serde_json::Value::as_str) == Some(conv_id))
                .unwrap()
        };
        assert_eq!(status_for(&beta).get("status").and_then(serde_json::Value::as_str), Some("sent"));
        assert_eq!(status_for(&gamma).get("status").and_then(serde_json::Value::as_str), Some("failed"));
        let gamma_err = status_for(&gamma).get("error").and_then(serde_json::Value::as_str).unwrap_or_default();
        assert!(gamma_err.contains("rejected the source message policy"));
        assert!(gamma_err.contains("MessageSendPermissionDenied"));
        assert_eq!(status_for(&delta).get("status").and_then(serde_json::Value::as_str), Some("failed"));
        let delta_err = status_for(&delta).get("error").and_then(serde_json::Value::as_str).unwrap_or_default();
        assert!(delta_err.contains("rejected the source message policy"));
        assert!(delta_err.contains("OtherSendPermissionDenied"));
        assert_eq!(restarted.read_human_conversation_transcript(&beta).unwrap().len(), 1);
        assert_eq!(restarted.read_human_conversation_transcript(&gamma).unwrap().len(), 0);
        assert_eq!(restarted.read_human_conversation_transcript(&delta).unwrap().len(), 0);

        // Source-eligibility forward-options clamp (CanHideForwardAuthor / NormalizeForwardOptions):
        // when the source entry requires preserving forward attribution (forcedForwardInfo=true),
        // requested drop_sender_names/drop_captions are clamped to PreserveInfo and origin is retained.
        assert!(source_owner.append_transcript_entry(&serde_json::json!({
            "id": "human-server-message:902",
            "kind": "message",
            "role": "user",
            "authorKind": "human",
            "authorId": "human-source",
            "authorName": "Forced Source Author",
            "content": "mandatory attribution message",
            "forcedForwardInfo": true,
            "delivery": "sent",
            "timestampMs": 30.0,
            "remoteMessageId": "902",
            "remoteCreatedAt": "1970-01-01T00:00:30Z"
        })).unwrap());
        let clamped = restarted
            .forward_human_message(
                &source,
                "human-server-message:902",
                &[beta.clone()],
                "forward-request-forced-preserve-1",
                true,
                true,
            )
            .unwrap();
        assert_eq!(
            clamped.pointer("/destinations/0/status").and_then(serde_json::Value::as_str),
            Some("sent"),
        );
        let beta_entries = restarted.read_human_conversation_transcript(&beta).unwrap();
        assert_eq!(beta_entries.len(), 2);
        let forced_forwarded = &beta_entries[1];
        let forced_context = forced_forwarded.get("forwardContext").unwrap();
        assert_eq!(
            forced_context.pointer("/privacy/dropSenderNames").and_then(serde_json::Value::as_bool),
            Some(false),
        );
        assert_eq!(
            forced_context.pointer("/privacy/dropCaptions").and_then(serde_json::Value::as_bool),
            Some(false),
        );
        assert_eq!(
            forced_context.pointer("/privacy/forcedPreserveInfo").and_then(serde_json::Value::as_bool),
            Some(true),
        );
        assert_eq!(
            forced_context.pointer("/origin/authorName").and_then(serde_json::Value::as_str),
            Some("Forced Source Author"),
        );
        let clamped_replay = restarted
            .forward_human_message(
                &source,
                "human-server-message:902",
                &[beta.clone()],
                "forward-request-forced-preserve-1",
                true,
                true,
            )
            .unwrap();
        assert_eq!(
            clamped_replay.pointer("/destinations/0/status").and_then(serde_json::Value::as_str),
            Some("sent"),
        );
        assert_eq!(restarted.read_human_conversation_transcript(&beta).unwrap().len(), 2);

        // Temporal recipient policy refresh (TEMP-TDRP-SHARE-RECIPIENT-POLICY-REFRESH-001):
        // A legacy persisted ConversationPermissions snapshot missing can_send_other/can_send_inline/can_send_games
        // fails closed for content-derived sticker/inline/game source entries until authoritative policy refresh
        // updates the canonical permissions in SandAgentDb, and the refreshed policy survives restart.
        let epsilon = conversation_id(
            &restarted
                .create_human_conversation("human-epsilon", "Epsilon LegacyPolicy")
                .unwrap(),
        );
        let epsilon_owner = restarted.open_human_conversation_db_owner(&epsilon).unwrap();
        assert!(epsilon_owner
            .set_metadata(
                "permissions",
                serde_json::json!({
                    "canSendMessages": true,
                    "canSendMedia": true,
                    "canSendPolls": true,
                    "canAddMembers": true,
                    "canPinMessages": true,
                    "canManageTopics": true,
                    "canManageCalls": true
                }),
            )
            .unwrap());

        assert!(source_owner
            .append_transcript_entry(&serde_json::json!({
                "id": "human-server-message:903",
                "kind": "message",
                "role": "user",
                "authorKind": "human",
                "authorId": "human-source",
                "content": "inline game message",
                "game": { "shortName": "chess" },
                "viaBotId": "bot-inline-1",
                "delivery": "sent",
                "timestampMs": 35.0,
                "remoteMessageId": "903",
                "remoteCreatedAt": "1970-01-01T00:00:35Z"
            }))
            .unwrap());

        let before_refresh = restarted
            .search_human_recipients(&source, "human-server-message:903", "", 50)
            .unwrap();
        assert!(
            before_refresh
                .iter()
                .all(|item| item.get("id").and_then(serde_json::Value::as_str) != Some(epsilon.as_str())),
            "legacy permissions missing can_send_other/can_send_inline/can_send_games must fail closed before policy refresh",
        );

        restarted
            .refresh_human_conversation_recipient_policy(
                &epsilon,
                fabushi_messaging_core::ConversationPermissions::default(),
                None,
            )
            .unwrap();

        let after_refresh = restarted
            .search_human_recipients(&source, "human-server-message:903", "", 50)
            .unwrap();
        assert!(
            after_refresh
                .iter()
                .any(|item| item.get("id").and_then(serde_json::Value::as_str) == Some(epsilon.as_str())),
            "authoritative policy refresh must expose the destination for inline/game/send-other forwards",
        );

        // Secret-content vs standard-destination compatibility fails closed before exposure and at forward boundary
        let secret_dest = conversation_id(
            &restarted
                .create_human_conversation("human-secret", "Secret Peer")
                .unwrap(),
        );
        let secret_dest_owner = restarted.open_human_conversation_db_owner(&secret_dest).unwrap();
        assert!(secret_dest_owner
            .set_metadata("secret", serde_json::json!(true))
            .unwrap());
        let standard_for_secret_check = restarted
            .search_human_recipients(&source, source_entry_id, "", 50)
            .unwrap();
        assert!(
            standard_for_secret_check
                .iter()
                .all(|item| item.get("id").and_then(serde_json::Value::as_str) != Some(secret_dest.as_str())),
            "secret destination must be excluded for standard non-secret source messages",
        );

        restarted.shutdown();

        let restarted_after_policy = test_workers(&root);
        let after_restart_refresh = restarted_after_policy
            .search_human_recipients(&source, "human-server-message:903", "", 50)
            .unwrap();
        assert!(
            after_restart_refresh
                .iter()
                .any(|item| item.get("id").and_then(serde_json::Value::as_str) == Some(epsilon.as_str())),
            "refreshed recipient policy must survive restart",
        );
        let refreshed_settlement = restarted_after_policy
            .forward_human_message(
                &source,
                "human-server-message:903",
                &[epsilon.clone(), secret_dest.clone()],
                "forward-request-refreshed-policy-1",
                false,
                false,
            )
            .unwrap();
        let refreshed_dests = refreshed_settlement
            .get("destinations")
            .and_then(serde_json::Value::as_array)
            .unwrap();
        assert_eq!(refreshed_dests[0].get("status").and_then(serde_json::Value::as_str), Some("sent"));
        assert_eq!(refreshed_dests[1].get("status").and_then(serde_json::Value::as_str), Some("failed"));
        assert_eq!(
            restarted_after_policy
                .read_human_conversation_transcript(&epsilon)
                .unwrap()
                .len(),
            1
        );
        assert_eq!(
            restarted_after_policy
                .read_human_conversation_transcript(&secret_dest)
                .unwrap()
                .len(),
            0
        );

        restarted_after_policy.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}


