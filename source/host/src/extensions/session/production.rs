use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

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
struct HumanSyncRecoveryContext {
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
struct HumanRetryLane<'a> {
    key: String,
    serial: Option<Arc<Mutex<()>>>,
    registry: &'a Mutex<BTreeMap<String, Arc<Mutex<()>>>>,
}

impl Drop for HumanRetryLane<'_> {
    fn drop(&mut self) {
        if let Ok(mut locks) = self.registry.lock() {
            let Some(serial) = self.serial.take() else { return; };
            let same_lane = locks.get(&self.key).is_some_and(|registered| Arc::ptr_eq(registered, &serial));
            drop(serial);
            if same_lane && locks.get(&self.key).is_some_and(|registered| Arc::strong_count(registered) == 1) {
                locks.remove(&self.key);
            }
        }
    }
}

#[cfg(test)]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum HumanCreationStage {
    BeforeLock,
    Locked,
    BeforeInitialize,
    BeforePublish,
}

#[cfg(test)]
type HumanCreationHook = Arc<
    dyn Fn(HumanCreationStage, &str, Option<&Arc<SandAgentDb>>) + Send + Sync,
>;

/// Shipping Host owner for the Grok session materialization worker boundary.
///
/// The pool is retained for the Host lifetime so per-agent blob workers are
/// reused across turns and are closed as part of Host shutdown. The remaining
/// recovery/metadata-GC/session materialization modules stay explicitly
/// non-final until their frozen Grok contracts are ported.
pub struct ProductionSessionWorkers {
    agents_root: PathBuf,
    pool: Arc<ProductionAgentWorkerPool>,
    conversation_size_maintenance: ConversationSizeMaintenance,
    conversation_state: SessionConversationState,
    memory_service: Arc<MemoryService>,
    mint_queue: SessionMintQueue,
    db_owners: Mutex<BTreeMap<String, Arc<SandAgentDb>>>,
    human_message_locks: Mutex<BTreeMap<String, Arc<Mutex<()>>>>,
    #[cfg(test)]
    human_creation_hook: Option<HumanCreationHook>,
    agent_store_owners: Mutex<BTreeMap<String, Arc<ProductionAgentStore>>>,
    deleting_agents: Mutex<BTreeSet<String>>,
    roster_extras_cache: RosterExtrasCache,
    user_time_zone_resolver: UserTimeZoneResolver,
    local_human_id: Option<String>,
    native_messaging: Option<Arc<FabushiNativeMessagingClient>>,
    native_messaging_required: bool,
    native_messaging_error: Option<String>,
    call_session_store: Option<Arc<CallSessionStore>>,
    call_session_store_error: Option<String>,
    busy_timeout_ms: u64,
}

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

    pub fn session_db_path(&self, agent_id: &str) -> Result<PathBuf, String> {
        get_agent_db_path(&self.agents_root, agent_id).map_err(|error| error.to_string())
    }

    pub fn list_agent_record_ids(&self) -> Result<Vec<String>, String> {
        list_agent_record_ids(&self.agents_root).map_err(|error| error.to_string())
    }

    pub fn count_owned_agents(&self) -> Result<usize, String> {
        count_owned_agents(&self.agents_root).map_err(|error| error.to_string())
    }

    pub fn is_agent_cap_reached(&self) -> Result<bool, String> {
        is_agent_cap_reached(&self.agents_root).map_err(|error| error.to_string())
    }

    pub fn reclaim_pruned_placeholders(
        &self,
        active_agent_id: Option<&str>,
    ) -> Result<Vec<String>, String> {
        let visible_agent_ids = self
            .list_agent_summaries(active_agent_id)?
            .into_iter()
            .map(|summary| summary.id)
            .collect::<BTreeSet<_>>();
        let candidates = list_pruned_placeholder_ids(
            &self.agents_root,
            active_agent_id,
            &visible_agent_ids,
        )
        .map_err(|error| error.to_string())?;
        let mut reclaimed = Vec::new();
        for agent_id in candidates {
            let db_path = self.session_db_path(&agent_id)?;
            let blob_path = conversation_blobs_path(&db_path);
            let _ = self.close_agent_store_owner(&agent_id, false);
            let _ = self.close_agent_db_owner(&agent_id, false);
            futures::executor::block_on(self.pool.close_store(&blob_path));
            match fs::remove_dir_all(self.agents_root.join(&agent_id)) {
                Ok(()) => reclaimed.push(agent_id),
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
                Err(error) => {
                    report_materialization_failure(
                        "placeholder_reclaim_failed",
                        &agent_id,
                        &format!("io::{:?}", error.kind()),
                    );
                }
            }
        }
        Ok(reclaimed)
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

    fn metadata_has_local_human(&self, metadata: &serde_json::Value) -> Result<bool, String> {
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
    fn update_human_activity(owner: &SandAgentDb, timestamp_ms: f64) -> Result<(), String> {
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

    fn shipping_native_messaging(&self) -> Result<Option<&FabushiNativeMessagingClient>, String> {
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

    fn validate_remote_human_reactions(
        reactions: &[FabushiRemoteHumanReaction],
    ) -> Result<(), String> {
        let mut emojis = BTreeSet::new();
        for reaction in reactions {
            if !matches!(
                (reaction.count, reaction.reacted_by_me),
                (0, false) | (1, false) | (1, true) | (2, true)
            ) {
                return Err("remote Human reaction summary is inconsistent with direct participants".into());
            }
            if !emojis.insert(reaction.emoji.as_str()) {
                return Err("remote Human reaction summary contains duplicate emoji".into());
            }
        }
        Ok(())
    }

    // Call only after direct-pair summary admission. Projection work depends on
    // summary items, never on an externally supplied numeric count.
    fn project_remote_human_reactions(
        reactions: &[FabushiRemoteHumanReaction],
        peer_human_id: &str,
    ) -> Vec<serde_json::Value> {
        let mut projected = Vec::new();
        for reaction in reactions {
            if reaction.reacted_by_me {
                projected.push(serde_json::json!({
                    "emoji": reaction.emoji,
                    "by": "me",
                }));
            }
            if reaction.count > u64::from(reaction.reacted_by_me) {
                projected.push(serde_json::json!({
                    "emoji": reaction.emoji,
                    "by": peer_human_id,
                }));
            }
        }
        projected
    }

    // Admission is pure and happens before remote resources or transcript writes.
    // Server request ids are scoped to their actual author, not to this device.
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

    fn human_message_lock(&self, conversation_id: &str) -> Result<Arc<Mutex<()>>, String> {
        let mut locks = self.human_message_locks.lock()
            .map_err(|_| "Human conversation mutation lock registry poisoned".to_string())?;
        Ok(Arc::clone(locks.entry(conversation_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))))
    }

    fn pending_human_forward_is_batch_owned(
        &self, conversation_id: &str, entry: &serde_json::Value,
    ) -> Result<bool, String> {
        // A damaged version must not turn source-owned intent into an ordinary
        // send. The separate marker also survives replacement of its context.
        if entry.get("forwardBatchOwned").is_some() {
            return Ok(true);
        }
        let context = entry.get("forwardContext");
        if context.is_some_and(|value| value.get("batchVersion").is_some()) {
            return Ok(true);
        }
        let nonce = entry.get("clientNonce").and_then(serde_json::Value::as_str)
            .ok_or_else(|| "persisted Human pending nonce is invalid".to_string())?;
        let author = entry.get("authorId").and_then(serde_json::Value::as_str)
            .ok_or_else(|| "persisted Human pending author is invalid".to_string())?;
        let mut legacy_source_reference = None;
        if let Some(context) = context.filter(|value|
            ["sourceConversationId", "sourceEntryId", "requestId"].iter()
                .any(|key| value.get(*key).is_some())
        ) {
            let source = context.get("sourceConversationId").and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "persisted Human forward source is invalid".to_string())?;
            let request = context.get("requestId").and_then(serde_json::Value::as_str)
                .filter(|value| !value.is_empty())
                .ok_or_else(|| "persisted Human forward request is invalid".to_string())?;
            // This is journal-only ownership discovery. A rebound source must remain
            // discoverable so generic outbox replay cannot bypass its denial;
            // it grants no transcript or resource authority.
            let owner = self.open_human_forward_recovery_owner(source)?;
            if load_forward_batches(&owner, source)?.iter().any(|batch|
                (batch.request_id == request && batch.account_id == author)
                    || batch.destinations.iter().any(|target|
                        target.conversation_id == conversation_id && target.client_nonce == nonce
                    )
            ) {
                return Ok(true);
            }
            // The pre-journal shipping path used this exact nonce contract.
            // A readable but unrelated source alone cannot authorize downgrade
            // of a v1 target whose real source/journal is no longer available.
            let mut legacy_digest = Sha256::new();
            legacy_digest.update(request.as_bytes());
            legacy_digest.update([0_u8]);
            legacy_digest.update(conversation_id.as_bytes());
            if nonce == format!("human-forward-{:x}", legacy_digest.finalize()) {
                let source_entry = context.get("sourceEntryId").and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "persisted Human forward source entry is invalid".to_string())?;
                legacy_source_reference = Some((source, source_entry));
            }
        }
        // Older rows did not have the independent marker. If their entire
        // context was lost/replaced, their stable target nonce still identifies
        // the source batch. Unrelated ordinary reply metadata is not itself a
        // forwarding identity, and genuine legacy nonces remain replayable.
        if !nonce.starts_with("human-forward-") { return Ok(false); }
        let entries = match fs::read_dir(get_native_conversations_root(&self.agents_root)) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                return Err("persisted Human forward ownership cannot be resolved from unavailable source stores".into());
            }
            Err(error) => return Err(error.to_string()),
        };
        for source in entries {
            let source = source.map_err(|error| error.to_string())?;
            if !source.file_type().map_err(|error| error.to_string())?.is_dir() { continue; }
            let source_id = source.file_name().to_string_lossy().into_owned();
            let db_path = get_native_conversation_db_path(&self.agents_root, &source_id)
                .map_err(|error| error.to_string())?;
            if !db_path.is_file() { continue; }
            let owner = self.open_human_forward_recovery_owner(&source_id)?;
            if load_forward_batches(&owner, &source_id)?.iter().any(|batch|
                batch.destinations.iter().any(|target|
                    target.conversation_id == conversation_id && target.client_nonce == nonce
                )
            ) {
                return Ok(true);
            }
        }
        let Some((source, source_entry)) = legacy_source_reference else {
            return Err("persisted Human forward ownership is unresolved; retain pending intent until its source can be recovered".into());
        };
        // Legacy compatibility is not a permission bypass. Use the same
        // current source policy as new batches before ordinary outbox replay;
        // retain the original pending payload and report any source denial.
        self.authorize_forward_source(source, source_entry)
            .map_err(|error| error.to_string())?;
        Ok(false)
    }

    fn acquire_human_send_settlement_guard<'a>(
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

    fn update_human_replay_snapshot(
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

    fn store_human_coverage(owner: &SandAgentDb, after_id: u64, complete: bool) -> Result<(), String> {
        if !owner.set_metadata(
            "humanHistoryCoverage",
            serde_json::json!({"version": 1, "afterId": after_id, "complete": complete}),
        ).map_err(|error| error.to_string())? {
            return Err("Human history coverage checkpoint was not durably stored".into());
        }
        Ok(())
    }

    fn capture_human_sync_recovery(
        &self,
        client: &FabushiNativeMessagingClient,
    ) -> Result<HumanSyncRecoveryContext, String> {
        let local_human_id = self.local_human_id()?.to_string();
        if client.identity()?.user_id != local_human_id {
            return Err("Fabushi native messaging credential changed Human account".into());
        }
        let recovery = HumanSyncRecoveryContext {
            local_human_id,
            credential_fingerprint: client.request_credential_fingerprint()?,
            first_reaction_refusal: None,
        };
        self.check_human_sync_binding(client, &recovery)?;
        Ok(recovery)
    }

    fn check_human_sync_binding(
        &self,
        client: &FabushiNativeMessagingClient,
        recovery: &HumanSyncRecoveryContext,
    ) -> Result<(), String> {
        if self.local_human_id()? != recovery.local_human_id
            || client.identity()?.user_id != recovery.local_human_id
        {
            return Err("Fabushi native messaging credential changed Human account".into());
        }
        if client.request_credential_fingerprint()? != recovery.credential_fingerprint {
            return Err("Fabushi native messaging credential changed during Human request".into());
        }
        Ok(())
    }

    fn check_human_sync_continuation(
        &self,
        recovery: Option<&HumanSyncRecoveryContext>,
    ) -> Result<(), String> {
        let Some(recovery) = recovery.filter(|value| value.is_continuing()) else {
            return Ok(());
        };
        let client = self.shipping_native_messaging()?
            .ok_or_else(|| "Human recovery continuation requires native messaging".to_string())?;
        self.check_human_sync_binding(client, recovery)
    }

    fn revalidate_human_recovery_owner(
        &self,
        conversation_id: &str,
        owner: &SandAgentDb,
        original_metadata: &serde_json::Value,
    ) -> Result<(), String> {
        // A retained response cannot reopen or register a replacement after
        // Session shutdown. Reuse only the original, still registered owner.
        if owner.is_closed() {
            return Err("Human conversation owner is closed".into());
        }
        let db_path = get_native_conversation_db_path(&self.agents_root, conversation_id)
            .map_err(|error| error.to_string())?;
        let directory = db_path.parent()
            .ok_or_else(|| "native Human conversation database has no parent".to_string())?;
        let directory_type = fs::symlink_metadata(directory).map_err(|error| error.to_string())?;
        if directory_type.file_type().is_symlink() || !directory_type.is_dir() {
            return Err(format!("Human conversation parent is not a real directory: {conversation_id}"));
        }
        let file_type = fs::symlink_metadata(&db_path).map_err(|error| error.to_string())?;
        if file_type.file_type().is_symlink() || !file_type.is_file() {
            return Err(format!("Human conversation store is not a regular non-symlink file: {conversation_id}"));
        }
        let key = Self::native_conversation_owner_key(conversation_id);
        let current = {
            let owners = self.db_owners.lock()
                .map_err(|_| "conversation db owner map poisoned".to_string())?;
            owners.get(&key).cloned()
        }.ok_or_else(|| "Human conversation owner changed during reaction recovery".to_string())?;
        // SandAgentDb resolves relative paths against the current directory
        // when it opens the existing owner. Match that same path contract.
        let expected_owner_path = if db_path.is_absolute() {
            db_path
        } else {
            std::env::current_dir().map_err(|error| error.to_string())?.join(db_path)
        };
        if !std::ptr::eq(current.as_ref(), owner) || owner.db_path() != expected_owner_path.as_path() {
            return Err("Human conversation owner changed during reaction recovery".into());
        }
        if current.is_closed() {
            return Err("Human conversation owner is closed".into());
        }
        let metadata = Self::read_human_conversation_metadata(&current, conversation_id, true)?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        if metadata.get("participantIds") != original_metadata.get("participantIds") {
            return Err("Human conversation participants changed during reaction recovery".into());
        }
        Ok(())
    }

    fn check_human_history_continuation(
        &self,
        conversation_id: &str,
        owner: &SandAgentDb,
        metadata: &serde_json::Value,
        recovery: Option<&HumanSyncRecoveryContext>,
    ) -> Result<(), String> {
        if recovery.is_some_and(HumanSyncRecoveryContext::is_continuing) {
            self.check_human_sync_continuation(recovery)?;
            self.revalidate_human_recovery_owner(conversation_id, owner, metadata)?;
        }
        Ok(())
    }

    pub fn sync_human_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        let client = self.shipping_native_messaging()?;
        let mut recovery = client.map(|client| self.capture_human_sync_recovery(client)).transpose()?;
        // Capture before forward recovery; its completion cannot authorize a
        // replacement credential. Forward delivery still precedes destination locks.
        self.resume_human_forward_batches()?;
        if let (Some(client), Some(recovery)) = (client, recovery.as_ref()) {
            self.check_human_sync_binding(client, recovery)?;
        }
        let entries = self.sync_human_conversation_history(conversation_id, recovery.as_mut())?;
        match recovery {
            Some(recovery) => recovery.finish(entries),
            None => Ok(entries),
        }
    }

    fn sync_human_conversation_history(
        &self,
        conversation_id: &str,
        mut recovery: Option<&mut HumanSyncRecoveryContext>,
    ) -> Result<Vec<serde_json::Value>, String> {
        const PAGE_LIMIT: usize = 200;
        const MAX_GAP_PAGES: usize = 32;
        const MAX_SAFE_ID: u64 = 9_007_199_254_740_991;
        self.check_human_sync_continuation(recovery.as_deref())?;
        let serial = self.human_message_lock(conversation_id)?;
        let _exclusive = serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?;
        self.check_human_sync_continuation(recovery.as_deref())?;
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        let Some(client) = self.shipping_native_messaging()? else {
            return owner.get_human_transcript_entries().map_err(|error| error.to_string());
        };
        let local_human_id = self.local_human_id()?.to_string();
        if client.identity()?.user_id != local_human_id {
            return Err("Fabushi native messaging credential changed Human account".into());
        }
        self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
        let coverage = owner.get_metadata("humanHistoryCoverage").map_err(|error| error.to_string())?;
        let (covered, complete) = match coverage.as_ref() {
            None | Some(serde_json::Value::Null) => (0, false),
            Some(checkpoint) => {
                let after_id = checkpoint.get("afterId")
                    .and_then(serde_json::Value::as_u64)
                    .filter(|value| *value <= MAX_SAFE_ID)
                    .ok_or_else(|| "persisted Human history coverage cursor is invalid".to_string())?;
                let complete = checkpoint.get("complete")
                    .and_then(serde_json::Value::as_bool)
                    .ok_or_else(|| "persisted Human history coverage completion flag is invalid".to_string())?;
                match checkpoint.get("version") {
                    Some(version) if version.as_u64() == Some(1) => (after_id, complete),
                    None => {
                        // Older clients persisted this shape without requiring
                        // historyCursor. Preserve their rows and checkpoint until
                        // a qualified origin page is durably applied.
                        (0, false)
                    }
                    Some(_) => return Err("persisted Human history coverage version is unsupported".into()),
                }
            }
        };
        self.replay_human_reaction_intent_with_recovery(
            &owner, &metadata, client, recovery.as_deref_mut(),
        )?;
        self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
        let pending = owner.get_transcript_entries().map_err(|error| error.to_string())?
            .into_iter().filter(|entry| {
                entry.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id.as_str())
                    && entry.get("remoteMessageId").is_none()
                    && matches!(entry.get("delivery").and_then(serde_json::Value::as_str), Some("pending" | "dispatching"))
                    && entry.get("clientNonce").and_then(serde_json::Value::as_str).is_some()
            }).collect::<Vec<_>>();
        for entry in pending {
            self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
            // Earlier parents in this ordered replay may have atomically
            // repaired this child's reply edge. Consume the current durable
            // row, not the reply ID captured in the initial pending snapshot.
            let Some(entry) = owner.get_transcript_entries().map_err(|error| error.to_string())?
                .into_iter().find(|current| {
                    current.get("id") == entry.get("id")
                        && current.get("clientNonce") == entry.get("clientNonce")
                        && current.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id.as_str())
                        && current.get("remoteMessageId").is_none()
                        && matches!(current.get("delivery").and_then(serde_json::Value::as_str), Some("pending" | "dispatching"))
                }) else { continue; };
            // A source-owned forwarding outbox never becomes a generic send
            // because its version/context was damaged or replaced.
            if self.pending_human_forward_is_batch_owned(conversation_id, &entry)? {
                continue;
            }
            let Some(nonce) = entry.get("clientNonce").and_then(serde_json::Value::as_str) else { continue; };
            let text = entry.get("content").and_then(serde_json::Value::as_str).unwrap_or_default();
            let reply = entry.get("replyToId").and_then(serde_json::Value::as_str);
            let composed = entry.get("composedAtMs").and_then(serde_json::Value::as_f64);
            let attachments = entry.get("attachments").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
            let result = self.append_human_message_with_context(
                conversation_id, text, nonce, composed, reply, &attachments,
                entry.get("forwardContext"), Some(&_exclusive), false,
            );
            if recovery.as_deref().is_some_and(HumanSyncRecoveryContext::is_continuing) {
                // This branch was unreachable after a refused reaction before.
                // Recheck binding first, then preserve every append error unchanged.
                self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
                result?;
            }
        }
        let peer = self.peer_human_id_from_metadata(&metadata)?;
        let mut after = if complete && covered < PAGE_LIMIT as u64 { 0 } else { covered };
        let mut rejected: Option<String> = None;
        let mut reached_end = false;
        let mut pages_used = 0;
        for _ in 0..MAX_GAP_PAGES {
            self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
            pages_used += 1;
            let page = client.list_direct_messages_after_for_user(&peer, &local_human_id, after, PAGE_LIMIT)?;
            self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
            if client.identity()?.user_id != local_human_id {
                return Err("Fabushi native messaging credential changed Human account".into());
            }
            if page.len() > PAGE_LIMIT {
                return Err("Fabushi Human history page exceeded the 200-message response budget".into());
            }
            if page.is_empty() {
                if rejected.is_none() {
                    self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
                    Self::store_human_coverage(&owner, after.max(covered), true)?;
                }
                reached_end = true;
                break;
            }
            let is_last = page.len() < PAGE_LIMIT;
            let mut snapshot = owner.get_transcript_entries().map_err(|error| error.to_string())?;
            let mut last = after;
            let mut accepted = Vec::new();
            for remote in &page {
                let id = fabushi_message_identity_text(&remote.id)?.parse::<u64>()
                    .map_err(|_| "remote Human history id is not a positive integer".to_string())?;
                if id <= last || id > MAX_SAFE_ID {
                    return Err("Fabushi Human history afterId did not advance monotonically".into());
                }
                last = id;
                match self.validate_remote_human_message(remote, &local_human_id, &peer, &snapshot) {
                    Ok(()) => accepted.push(remote.clone()),
                    Err(error) => {
                        if recovery.as_deref().is_some_and(HumanSyncRecoveryContext::is_continuing) {
                            return Err(error);
                        }
                        rejected.get_or_insert(error);
                    }
                }
            }
            for remote in accepted {
                self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
                if let Err(error) = self.validate_remote_human_message(&remote, &local_human_id, &peer, &snapshot) {
                    if recovery.as_deref().is_some_and(HumanSyncRecoveryContext::is_continuing) {
                        return Err(error);
                    }
                    rejected.get_or_insert(error);
                    continue;
                }
                let settled = self.materialize_remote_human_message_with_snapshot(&owner, &remote, &snapshot)?;
                Self::update_human_replay_snapshot(&mut snapshot, settled);
                // A synchronous committed-row callback may rotate credentials.
                // Finish that existing materialization/activity unit, then stop.
                self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
            }
            if rejected.is_none() {
                self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
                Self::store_human_coverage(&owner, last, is_last)?;
            }
            after = last;
            if is_last { reached_end = true; break; }
        }
        // A 32-page window is an absolute request budget, including the last
        // short or empty page. A latest-snapshot refresh may use only spare capacity.
        if reached_end && rejected.is_none() && pages_used < MAX_GAP_PAGES && after >= PAGE_LIMIT as u64 {
            self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
            let latest = client.list_direct_messages_for_user(&peer, &local_human_id, None, PAGE_LIMIT)?;
            self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
            let mut snapshot = owner.get_transcript_entries().map_err(|error| error.to_string())?;
            let known = snapshot.iter().filter_map(|entry| entry.get("remoteMessageId")
                .and_then(serde_json::Value::as_str).map(str::to_string)).collect::<BTreeSet<_>>();
            for remote in latest {
                let remote_id = fabushi_message_identity_text(&remote.id)?;
                if !known.contains(remote_id.as_str()) { continue; }
                self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
                self.validate_remote_human_message(&remote, &local_human_id, &peer, &snapshot)?;
                let settled = self.materialize_remote_human_message_with_snapshot(&owner, &remote, &snapshot)?;
                Self::update_human_replay_snapshot(&mut snapshot, settled);
                self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
            }
        }
        self.check_human_history_continuation(conversation_id, &owner, &metadata, recovery.as_deref())?;
        if let Some(error) = rejected { return Err(error); }
        // Full 32-page windows persist a nonterminal cursor for the next sync/restart.
        owner.get_human_transcript_entries().map_err(|error| error.to_string())
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

    fn validate_human_retry_conversation(
        metadata: &serde_json::Value, conversation_id: &str, account: &str,
    ) -> Result<(), String> {
        let participants = metadata.get("participantIds").and_then(serde_json::Value::as_array)
            .filter(|participants| participants.len() == 2)
            .ok_or_else(|| "Human retry requires exactly the original two participants".to_string())?;
        let parse = |index: usize| participants[index].as_str()
            .filter(|value| !value.is_empty() && *value == value.trim())
            .ok_or_else(|| "Human retry persisted participant identity is invalid".to_string());
        let first = parse(0)?;
        let second = parse(1)?;
        if first == second || (first != account && second != account)
            || Self::human_direct_conversation_id(first, second) != conversation_id
        {
            return Err("Human retry participants conflict with the original conversation identity".into());
        }
        Ok(())
    }

    pub fn create_human_conversation(
        &self,
        peer_human_id: &str,
        title: &str,
    ) -> Result<serde_json::Value, String> {
        self.create_human_conversation_with_recovery(peer_human_id, title, None)
    }

    fn create_human_conversation_with_recovery(
        &self,
        peer_human_id: &str,
        title: &str,
        recovery: Option<&HumanSyncRecoveryContext>,
    ) -> Result<serde_json::Value, String> {
        self.check_human_sync_continuation(recovery)?;
        let local_human_id = self.local_human_id()?;
        let peer_human_id = peer_human_id.trim();
        let title = title.trim();
        if peer_human_id.is_empty() {
            return Err("human conversation requires peerHumanId".into());
        }
        if local_human_id == peer_human_id {
            return Err("human conversation participants must be distinct".into());
        }
        let conversation_id = Self::human_direct_conversation_id(local_human_id, peer_human_id);
        let serial = self.human_message_lock(&conversation_id)?;
        #[cfg(test)]
        self.observe_human_creation(HumanCreationStage::BeforeLock, &conversation_id, None);
        let _exclusive = serial.lock()
            .map_err(|_| "Human conversation serial lock poisoned".to_string())?;
        #[cfg(test)]
        self.observe_human_creation(HumanCreationStage::Locked, &conversation_id, None);
        // A continuation can wait behind another creator; its old pre-lock
        // credential check cannot authorize the following filesystem writes.
        self.check_human_sync_continuation(recovery)?;

        // The canonical path is shared by every discovery of this pair. Keep
        // creation in the same mutation lane as send/sync through publication.
        let db_path = get_native_conversation_db_path(&self.agents_root, &conversation_id)
            .map_err(|error| error.to_string())?;
        let key = Self::native_conversation_owner_key(&conversation_id);
        let cached_owner = self.db_owners
            .lock()
            .map_err(|_| "conversation db owner map poisoned".to_string())?
            .get(&key)
            .cloned();
        let owner = match cached_owner {
            Some(owner) => owner,
            None => {
                let conversation_dir = db_path
                    .parent()
                    .ok_or_else(|| "native conversation database has no parent".to_string())?;
                fs::create_dir_all(conversation_dir).map_err(|error| error.to_string())?;
                Arc::new(
                    SandAgentDb::open_human_conversation_store(&db_path, self.busy_timeout_ms)
                        .map_err(|error| error.to_string())?,
                )
            }
        };
        // A schema-only store left by an interrupted/failed initialization is
        // retryable. read_metadata alone would synthesize Agent defaults here.
        if owner.read_kv("metadata").map_err(|error| error.to_string())?.is_none() {
            let display_title = if title.is_empty() { peer_human_id } else { title };
            let created_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64;
            let metadata = serde_json::json!({
                "conversationKind": "human",
                "conversationId": conversation_id,
                "title": display_title,
                "participantIds": [local_human_id, peer_human_id],
                "createdAt": created_at,
                "lastActivityAt": 0.0,
            });
            #[cfg(test)]
            self.observe_human_creation(
                HumanCreationStage::BeforeInitialize, &conversation_id, Some(&owner),
            );
            self.check_human_sync_continuation(recovery)?;
            // false can mean existing metadata or a busy write, never ownership
            // of this directory. Validate durable metadata below in either case.
            let _ = owner.initialize_human_metadata_if_empty(metadata)
                .map_err(|error| error.to_string())?;
        }
        let conversation = Self::human_creation_summary(
            &owner, &conversation_id, local_human_id, peer_human_id,
        )?;
        #[cfg(test)]
        self.observe_human_creation(
            HumanCreationStage::BeforePublish, &conversation_id, Some(&owner),
        );
        self.check_human_sync_continuation(recovery)?;
        // Read-side openers do not take the mutation lock and may have published
        // a valid owner after initialization. Retain that Arc; dropping only our
        // unpublished candidate must never close or replace the cached winner.
        let published_owner = {
            let mut owners = self.db_owners.lock()
                .map_err(|_| "conversation db owner map poisoned".to_string())?;
            Arc::clone(owners.entry(key).or_insert_with(|| Arc::clone(&owner)))
        };
        if Arc::ptr_eq(&owner, &published_owner) {
            Ok(conversation)
        } else {
            Self::human_creation_summary(
                &published_owner, &conversation_id, local_human_id, peer_human_id,
            )
        }
    }

    fn human_creation_summary(
        owner: &SandAgentDb,
        conversation_id: &str,
        local_human_id: &str,
        peer_human_id: &str,
    ) -> Result<serde_json::Value, String> {
        let Some(raw) = owner.read_kv("metadata").map_err(|error| error.to_string())? else {
            return Err("failed to initialize native Human conversation metadata".into());
        };
        // The shared metadata decoder reads hex byte pairs. Reject non-ASCII
        // corruption before it can slice through a UTF-8 character boundary.
        if !raw.trim().is_ascii() {
            return Err("existing Human conversation metadata encoding is invalid".into());
        }
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if metadata.get("conversationKind").and_then(serde_json::Value::as_str) != Some("human") {
            return Err(format!("Conversation is not a Human conversation: {conversation_id}"));
        }
        if metadata.get("conversationId").and_then(serde_json::Value::as_str) != Some(conversation_id) {
            return Err("existing Human conversation identity does not match its canonical path".into());
        }
        let participants = metadata.get("participantIds")
            .and_then(serde_json::Value::as_array)
            .filter(|participants| participants.len() == 2 && participants.iter().all(serde_json::Value::is_string))
            .ok_or_else(|| "existing Human conversation participants are invalid".to_string())?;
        if !participants.iter().any(|id| id.as_str() == Some(local_human_id)) {
            return Err("existing Human conversation does not contain the local identity".into());
        }
        if !participants.iter().any(|id| id.as_str() == Some(peer_human_id)) {
            return Err("existing Human conversation does not contain the requested peer".into());
        }
        let title = match metadata.get("title") {
            Some(serde_json::Value::String(title)) => title.as_str(),
            None => peer_human_id,
            Some(_) => return Err("existing Human conversation title is invalid".into()),
        };
        Ok(serde_json::json!({
            "id": conversation_id,
            "kind": "human",
            "title": title,
            "participantIds": participants,
        }))
    }

    #[cfg(test)]
    fn observe_human_creation(
        &self,
        stage: HumanCreationStage,
        conversation_id: &str,
        owner: Option<&Arc<SandAgentDb>>,
    ) {
        if let Some(hook) = &self.human_creation_hook {
            hook(stage, conversation_id, owner);
        }
    }

    fn read_human_conversation_metadata(
        owner: &SandAgentDb,
        conversation_id: &str,
        require_canonical_pair: bool,
    ) -> Result<serde_json::Value, String> {
        let raw = owner.read_kv("metadata").map_err(|error| error.to_string())?
            .ok_or_else(|| "existing Human conversation metadata is missing".to_string())?;
        // Match the creation boundary before the shared hex decoder can slice
        // corrupt UTF-8. Missing identity must never become Agent defaults.
        if !raw.trim().is_ascii() {
            return Err("existing Human conversation metadata encoding is invalid".into());
        }
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if metadata.get("conversationKind").and_then(serde_json::Value::as_str) != Some("human")
            || metadata.get("conversationId").and_then(serde_json::Value::as_str) != Some(conversation_id)
        {
            return Err(format!("Conversation is not the requested Human conversation: {conversation_id}"));
        }
        let participants = metadata.get("participantIds").and_then(serde_json::Value::as_array)
            .ok_or_else(|| "existing Human conversation participants are invalid".to_string())?;
        if participants.len() != 2
            || participants.iter().any(|value| value.as_str()
                .is_none_or(|id| id.trim().is_empty() || id.trim() != id))
            || participants[0] == participants[1]
        {
            return Err("existing Human conversation participants are invalid".into());
        }
        let first = participants[0].as_str().expect("validated Human participant");
        let second = participants[1].as_str().expect("validated Human participant");
        if require_canonical_pair && Self::human_direct_conversation_id(first, second) != conversation_id {
            return Err("existing Human conversation participants conflict with the canonical identity".into());
        }
        Ok(metadata)
    }

    pub fn open_human_conversation_db_owner(
        &self,
        conversation_id: &str,
    ) -> Result<Arc<SandAgentDb>, String> {
        self.open_human_conversation_db_owner_with_pair_policy(conversation_id, true)
    }

    // This private opener is reserved for retained forwarding-journal discovery
    // and denial settlement. It grants no ordinary transcript/resource authority:
    // authorize_forward_source rechecks the canonical pair before reading a row.
    fn open_human_forward_recovery_owner(
        &self,
        conversation_id: &str,
    ) -> Result<Arc<SandAgentDb>, String> {
        self.open_human_conversation_db_owner_with_pair_policy(conversation_id, false)
    }

    fn open_human_conversation_db_owner_with_pair_policy(
        &self,
        conversation_id: &str,
        require_canonical_pair: bool,
    ) -> Result<Arc<SandAgentDb>, String> {
        let db_path = get_native_conversation_db_path(&self.agents_root, conversation_id)
            .map_err(|error| error.to_string())?;
        let directory = db_path.parent()
            .ok_or_else(|| "native Human conversation database has no parent".to_string())?;
        let directory_type = fs::symlink_metadata(directory).map_err(|error| error.to_string())?;
        if directory_type.file_type().is_symlink() || !directory_type.is_dir() {
            return Err(format!("Human conversation parent is not a real directory: {conversation_id}"));
        }
        let file_type = fs::symlink_metadata(&db_path).map_err(|error| error.to_string())?;
        if file_type.file_type().is_symlink() || !file_type.is_file() {
            return Err(format!("Human conversation store is not a regular non-symlink file: {conversation_id}"));
        }
        let key = Self::native_conversation_owner_key(conversation_id);
        let mut owners = self
            .db_owners
            .lock()
            .map_err(|_| "conversation db owner map poisoned".to_string())?;
        if let Some(owner) = owners.get(&key).cloned() {
            // An independent SQLite writer may have changed this source after
            // the owner was cached. Cached handles cannot bypass the same
            // conversation identity check as a freshly opened source.
            drop(owners);
            Self::read_human_conversation_metadata(&owner, conversation_id, require_canonical_pair)?;
            return Ok(owner);
        }
        let owner = Arc::new(
            SandAgentDb::open_human_conversation_store(&db_path, self.busy_timeout_ms)
                .map_err(|error| error.to_string())?,
        );
        if let Err(error) = Self::read_human_conversation_metadata(&owner, conversation_id, require_canonical_pair) {
            owner.close(false);
            return Err(error);
        }
        owners.insert(key, Arc::clone(&owner));
        Ok(owner)
    }

    pub(crate) fn metadata_history_coverage_incomplete(metadata: &serde_json::Value) -> bool {
        metadata
            .get("humanHistoryCoverage")
            .and_then(|coverage| coverage.get("complete"))
            .and_then(serde_json::Value::as_bool)
            == Some(false)
    }

    pub fn human_conversation_coverage_incomplete(
        &self,
        conversation_id: &str,
    ) -> Result<bool, String> {
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        Ok(Self::metadata_history_coverage_incomplete(&metadata))
    }

    pub fn list_human_conversations(&self) -> Result<Vec<serde_json::Value>, String> {
        let root = get_native_conversations_root(&self.agents_root);
        let entries = match fs::read_dir(&root) {
            Ok(entries) => entries.filter_map(Result::ok).collect::<Vec<_>>(),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
            Err(error) => return Err(error.to_string()),
        };
        let mut conversations = Vec::new();
        for entry in entries {
            if !entry.file_type().map(|kind| kind.is_dir()).unwrap_or(false) {
                continue;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            let Ok(owner) = self.open_human_conversation_db_owner(&id) else {
                continue;
            };
            let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
            if !self.metadata_has_local_human(&metadata)? {
                continue;
            }
            let title = metadata
                .get("title")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("Human conversation");
            let participant_ids = metadata
                .get("participantIds")
                .cloned()
                .unwrap_or_else(|| serde_json::json!([]));
            let updated_at = metadata
                .get("lastActivityAt")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_default();
            let mut conversation = serde_json::json!({
                "id": id,
                "kind": "human",
                "title": title,
                "participantIds": participant_ids,
                "updatedAt": updated_at,
            });
            if Self::metadata_history_coverage_incomplete(&metadata) {
                if let Some(object) = conversation.as_object_mut() {
                    object.insert("historyCoverageComplete".into(), serde_json::json!(false));
                }
            }
            conversations.push(conversation);
        }
        conversations.sort_by(|left, right| {
            left.get("id")
                .and_then(serde_json::Value::as_str)
                .cmp(&right.get("id").and_then(serde_json::Value::as_str))
        });
        Ok(conversations)
    }

    pub(crate) fn source_entry_requires_preserving_forward_info(
        source_entry: &serde_json::Value,
    ) -> bool {
        source_entry
            .get("forcedForwardInfo")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            || source_entry
                .get("forcePreserveForwardInfo")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
    }

    pub(crate) fn human_recipient_requirements_from_entry(
        source_entry: &serde_json::Value,
    ) -> fabushi_messaging_core::RecipientSearchRequirements {
        let has_non_null = |key: &str| source_entry.get(key).is_some_and(|v| !v.is_null());
        let has_non_empty_str = |key: &str| {
            source_entry
                .get(key)
                .and_then(serde_json::Value::as_str)
                .is_some_and(|s| !s.trim().is_empty())
        };
        let has_game = has_non_null("game");
        let has_other_content = has_game
            || has_non_null("sticker")
            || has_non_null("story")
            || has_non_null("giveaway")
            || has_non_null("location")
            || has_non_null("venue")
            || has_non_null("contact")
            || has_non_null("dice");
        let has_inline_content = has_non_null("miniApp")
            || has_non_empty_str("viaBotId")
            || has_non_empty_str("inlineBotId");
        fabushi_messaging_core::RecipientSearchRequirements {
            require_media: source_entry
                .get("attachments")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|attachments| !attachments.is_empty())
                || source_entry
                    .get("requireMedia")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            require_polls: source_entry
                .get("requirePolls")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                || has_non_null("poll"),
            require_send_other: source_entry
                .get("requireSendOther")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                || has_other_content,
            require_inline: source_entry
                .get("requireInline")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                || has_inline_content,
            require_game: source_entry
                .get("requireGame")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
                || has_game,
            source_protected_content: source_entry
                .get("protectedContent")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            content_kind: if source_entry
                .get("secretContent")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
            {
                fabushi_messaging_core::RecipientContentKind::Secret
            } else {
                fabushi_messaging_core::RecipientContentKind::Standard
            },
            ..fabushi_messaging_core::RecipientSearchRequirements::default()
        }
    }

    pub(crate) fn human_recipient_authorization_from_metadata(
        metadata: &serde_json::Value,
        sender_is_participant: bool,
    ) -> fabushi_messaging_core::RecipientAuthorizationInput {
        let mut input =
            fabushi_messaging_core::RecipientAuthorizationInput::direct_default(
                sender_is_participant,
            );
        if let Some(permissions_val) = metadata.get("permissions") {
            if let Some(permissions) = permissions_val.as_object() {
                let is_canonical_snake_snapshot = permissions.contains_key("can_send_messages")
                    || permissions.contains_key("can_send_media")
                    || permissions.contains_key("can_send_polls")
                    || permissions.contains_key("can_send_other")
                    || permissions.contains_key("can_send_inline")
                    || permissions.contains_key("can_send_games");
                if is_canonical_snake_snapshot {
                    if let Ok(parsed) = serde_json::from_value::<
                        fabushi_messaging_core::ConversationPermissions,
                    >(permissions_val.clone())
                    {
                        input.can_send_messages = parsed.can_send_messages;
                        input.can_send_media = parsed.can_send_media;
                        input.can_send_polls = parsed.can_send_polls;
                        input.can_send_other = parsed.can_send_other;
                        input.can_send_inline = parsed.can_send_inline;
                        input.can_send_games = parsed.can_send_games;
                    }
                }
                if let Some(v) = permissions
                    .get("canSendMessages")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.can_send_messages = v;
                }
                if let Some(v) = permissions
                    .get("canSendMedia")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.can_send_media = v;
                }
                if let Some(v) = permissions
                    .get("canSendPolls")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.can_send_polls = v;
                }
                if let Some(v) = permissions
                    .get("canSendOther")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.can_send_other = v;
                }
                if let Some(v) = permissions
                    .get("canSendInline")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.can_send_inline = v;
                }
                if let Some(v) = permissions
                    .get("canSendGames")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.can_send_games = v;
                }
            }
        }
        if let Some(restrictions_val) = metadata.get("restrictions") {
            if let Ok(parsed) = serde_json::from_value::<
                fabushi_messaging_core::MemberRestrictions,
            >(restrictions_val.clone())
            {
                input.community_restrict_messages = parsed.send_messages;
                input.community_restrict_media = parsed.send_media;
                input.community_restrict_polls = parsed.send_polls;
                input.community_restrict_other = parsed.send_other;
                input.community_restrict_inline = parsed.send_inline;
                input.community_restrict_games = parsed.send_games;
            }
            if let Some(restrictions) = restrictions_val.as_object() {
                if let Some(v) = restrictions
                    .get("sendMessages")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.community_restrict_messages = v;
                }
                if let Some(v) = restrictions
                    .get("sendMedia")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.community_restrict_media = v;
                }
                if let Some(v) = restrictions
                    .get("sendPolls")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.community_restrict_polls = v;
                }
                if let Some(v) = restrictions
                    .get("sendOther")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.community_restrict_other = v;
                }
                if let Some(v) = restrictions
                    .get("sendInline")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.community_restrict_inline = v;
                }
                if let Some(v) = restrictions
                    .get("sendGames")
                    .and_then(serde_json::Value::as_bool)
                {
                    input.community_restrict_games = v;
                }
            }
        }
        if let Some(v) = metadata
            .get("communityForbidden")
            .and_then(serde_json::Value::as_bool)
        {
            input.community_forbidden = v;
        }
        if let Some(v) = metadata
            .get("channelPostingAllowed")
            .and_then(serde_json::Value::as_bool)
        {
            input.channel_posting_allowed = v;
        }
        if metadata
            .get("isBroadcastChannel")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            || metadata
                .get("conversationSubkind")
                .and_then(serde_json::Value::as_str)
                == Some("channel")
        {
            input.conversation_kind = fabushi_messaging_core::ConversationKind::Channel;
        }
        if metadata
            .get("secret")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            || metadata
                .get("conversationSubkind")
                .and_then(serde_json::Value::as_str)
                == Some("secret")
        {
            input.conversation_kind = fabushi_messaging_core::ConversationKind::Secret;
        }
        input
    }

    pub fn refresh_human_conversation_recipient_policy(
        &self,
        conversation_id: &str,
        permissions: fabushi_messaging_core::ConversationPermissions,
        restrictions: Option<fabushi_messaging_core::MemberRestrictions>,
    ) -> Result<(), String> {
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        let permissions_json =
            serde_json::to_value(permissions).map_err(|error| error.to_string())?;
        owner
            .set_metadata("permissions", permissions_json)
            .map_err(|error| error.to_string())?;
        if let Some(restrictions) = restrictions {
            let restrictions_json =
                serde_json::to_value(restrictions).map_err(|error| error.to_string())?;
            owner
                .set_metadata("restrictions", restrictions_json)
                .map_err(|error| error.to_string())?;
        }
        Ok(())
    }

    pub fn search_human_recipients(
        &self,
        source_conversation_id: &str,
        source_entry_id: &str,
        query: &str,
        limit: usize,
    ) -> Result<Vec<serde_json::Value>, String> {
        let source_owner = self.open_human_conversation_db_owner(source_conversation_id)?;
        let source_metadata = source_owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&source_metadata)? {
            return Err("local Human identity is not a participant in the source conversation".into());
        }
        let source_entry_id = source_entry_id.trim();
        if source_entry_id.is_empty() {
            return Err("recipient search requires sourceEntryId".into());
        }
        let source_entry = source_owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(source_entry_id))
            .ok_or_else(|| "recipient search source message is missing".to_string())?;
        if source_entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
            || source_entry.get("delivery").and_then(serde_json::Value::as_str) != Some("sent")
        {
            return Err("recipient search source must be a settled canonical message".into());
        }
        let requirements = Self::human_recipient_requirements_from_entry(&source_entry);
        let local_human_id = self.local_human_id()?;
        let normalized_query = query.trim().to_lowercase();
        let mut recipients = self
            .list_human_conversations()?
            .into_iter()
            .filter(|conversation| {
                let sender_is_participant = conversation
                    .get("participantIds")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|ids| {
                        ids.iter().any(|id| id.as_str() == Some(local_human_id))
                    });
                let auth_input = conversation
                    .get("id")
                    .and_then(serde_json::Value::as_str)
                    .and_then(|id| self.open_human_conversation_db_owner(id).ok())
                    .and_then(|owner| owner.read_metadata().ok())
                    .map(|metadata| {
                        Self::human_recipient_authorization_from_metadata(
                            &metadata,
                            sender_is_participant,
                        )
                    })
                    .unwrap_or_else(|| {
                        fabushi_messaging_core::RecipientAuthorizationInput::direct_default(
                            sender_is_participant,
                        )
                    });
                if !fabushi_messaging_core::recipient_search_authorized(
                    auth_input,
                    requirements,
                ) {
                    return false;
                }
                if normalized_query.is_empty() {
                    return true;
                }
                let mut fields = Vec::new();
                if let Some(title) = conversation
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                {
                    fields.push(title);
                }
                if let Some(ids) = conversation
                    .get("participantIds")
                    .and_then(serde_json::Value::as_array)
                {
                    for id in ids.iter().filter_map(serde_json::Value::as_str) {
                        fields.push(id);
                    }
                }
                fabushi_messaging_core::query_matches_fields(&normalized_query, &fields)
            })
            .collect::<Vec<_>>();

        recipients.sort_by(|left, right| {
            let left_updated = left
                .get("updatedAt")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_default();
            let right_updated = right
                .get("updatedAt")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or_default();
            right_updated
                .partial_cmp(&left_updated)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| {
                    left.get("title")
                        .and_then(serde_json::Value::as_str)
                        .cmp(&right.get("title").and_then(serde_json::Value::as_str))
                })
                .then_with(|| {
                    left.get("id")
                        .and_then(serde_json::Value::as_str)
                        .cmp(&right.get("id").and_then(serde_json::Value::as_str))
                })
        });
        recipients.truncate(limit.clamp(1, 100));
        Ok(recipients)
    }

    /// The same private Native credential owner supplies account and issuer;
    /// a token or device-session refresh does not rewrite durable intent.
    fn human_forward_scope(&self) -> Result<(String, String), String> {
        let account = self.local_human_id()?.to_string();
        let issuer = if let Some(client) = self.shipping_native_messaging()? {
            let (bound_account, issuer) = client.forward_account_scope()?;
            if bound_account != account {
                return Err("Fabushi native messaging credential changed Human account".into());
            }
            issuer
        } else {
            "local-only".to_string()
        };
        Ok((account, issuer))
    }

    /// Scan even now-ineligible source records, so removing a conversation
    /// cannot free its old request ID for another source under one account.
    fn find_forward_request(
        &self, account: &str, issuer: &str, request_id: &str,
    ) -> Result<Option<HumanForwardBatch>, String> {
        let root = get_native_conversations_root(&self.agents_root);
        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error.to_string()),
        };
        let mut matched: Option<HumanForwardBatch> = None;
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_symlink() {
                return Err("Human forward source directory is a symbolic link; retained identity cannot be skipped".into());
            }
            if !kind.is_dir() {
                continue;
            }
            let id = entry.file_name().to_string_lossy().into_owned();
            // A retained directory with unavailable storage may own this
            // request identity. Never free it for another source by skipping
            // a failed stat/open; retry only after storage recovers.
            let owner = self.open_human_forward_recovery_owner(&id)?;
            for batch in load_forward_batches(&owner, &id)? {
                if batch.account_id == account && batch.issuer == issuer && batch.request_id == request_id {
                    if matched.is_some() {
                        return Err("Human forward request identity is duplicated across source conversations".into());
                    }
                    matched = Some(batch);
                }
            }
        }
        Ok(matched)
    }

    fn authorize_forward_source(
        &self, source_id: &str, source_entry_id: &str,
    ) -> Result<serde_json::Value, ForwardSourceFailure> {
        let owner = self.open_human_forward_recovery_owner(source_id)
            .map_err(ForwardSourceFailure::Storage)?;
        let metadata = Self::read_human_conversation_metadata(&owner, source_id, false)
            .map_err(ForwardSourceFailure::Storage)?;
        let participants = metadata.get("participantIds").and_then(serde_json::Value::as_array)
            .ok_or_else(|| ForwardSourceFailure::Storage("existing Human conversation participants are invalid".into()))?;
        let first = participants[0].as_str().expect("structurally validated Human participant");
        let second = participants[1].as_str().expect("structurally validated Human participant");
        if Self::human_direct_conversation_id(first, second) != source_id {
            return Err(ForwardSourceFailure::Policy(
                "forwardHumanMessage source participant pair no longer matches its canonical identity".into()));
        }
        if !self.metadata_has_local_human(&metadata).map_err(ForwardSourceFailure::Storage)? {
            return Err(ForwardSourceFailure::Policy(
                "local Human identity is not a participant in the source conversation".into()));
        }
        let entry = owner.get_transcript_entries()
            .map_err(|error| ForwardSourceFailure::Storage(error.to_string()))?
            .into_iter()
            .find(|item| item.get("id").and_then(serde_json::Value::as_str) == Some(source_entry_id))
            .ok_or_else(|| ForwardSourceFailure::Policy(
                "forwardHumanMessage source message is missing".into()))?;
        if entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
            || entry.get("delivery").and_then(serde_json::Value::as_str) != Some("sent")
        {
            return Err(ForwardSourceFailure::Policy(
                "forwardHumanMessage source must be a settled canonical message".into()));
        }
        if entry.get("protectedContent").and_then(serde_json::Value::as_bool) == Some(true) {
            return Err(ForwardSourceFailure::Policy(
                "forwardHumanMessage source is protected and cannot be forwarded".into()));
        }
        if entry.get("content").and_then(serde_json::Value::as_str).unwrap_or_default().trim().is_empty()
            && !entry.get("attachments").and_then(serde_json::Value::as_array).is_some_and(|a| !a.is_empty())
        {
            return Err(ForwardSourceFailure::Policy(
                "forwardHumanMessage source has no forwardable content".into()));
        }
        Ok(entry)
    }

    fn deliver_forward_batch(
        &self, source_id: &str, request_id: &str, requested: Option<&[String]>,
    ) -> Result<HumanForwardBatch, String> {
        self.deliver_forward_batch_from_intent(source_id, request_id, requested, false)
    }

    fn deliver_forward_batch_from_intent(
        &self, source_id: &str, request_id: &str, requested: Option<&[String]>, require_existing: bool,
    ) -> Result<HumanForwardBatch, String> {
        let (account, issuer) = self.human_forward_scope()?;
        let serial = self.human_message_lock(&format!("forward-batch:{account}:{issuer}:{request_id}"))?;
        let exclusive = serial.lock().map_err(|_| "Human forward batch lock poisoned".to_string())?;
        self.deliver_forward_batch_while_locked(
            source_id, request_id, requested, require_existing, &account, &issuer, &exclusive,
        )
    }

    fn deliver_forward_batch_while_locked(
        &self, source_id: &str, request_id: &str, requested: Option<&[String]>, require_existing: bool,
        account: &str, issuer: &str, _exclusive: &std::sync::MutexGuard<'_, ()>,
    ) -> Result<HumanForwardBatch, String> {
        // This lane delivers an existing account/issuer-bound journal. Its
        // source may be revoked: load the retained intent and let per-target
        // authorization below persist its denial without granting row authority.
        // require_existing applies only to the destination outbox, so automatic
        // recovery can still dispatch a previously unsubmitted target.
        let owner = self.open_human_forward_recovery_owner(source_id)?;
        let mut batch = load_forward_batches(&owner, source_id)?.into_iter().find(|item|
            item.account_id == account && item.issuer == issuer && item.request_id == request_id
        ).ok_or_else(|| "forwardHumanMessage durable request was not found".to_string())?;
        if let Some(subset) = requested {
            if !batch.contains_requested(subset) {
                return Err("forwardHumanMessage request may not expand its original destination set".into());
            }
        }
        for index in 0..batch.destinations.len() {
            let target = &batch.destinations[index];
            if target.completed || requested.is_some_and(|subset|
                !subset.iter().any(|id| id == &target.conversation_id)
            ) {
                continue;
            }
            // Reauthorize before *each* network dispatch, not merely when the
            // original snapshot was accepted. The immutable payload never
            // changes when the source text is later edited.
            let source_entry = match self.authorize_forward_source(source_id, &batch.source_entry_id) {
                Ok(entry) => entry,
                Err(ForwardSourceFailure::Policy(denial)) => {
                    // Persist a visible denial without sending any remaining
                    // destination. Ordinary inbound history may still sync.
                    for destination in &mut batch.destinations {
                        if !destination.completed {
                            destination.error = Some(denial.clone());
                        }
                    }
                    store_forward_batch(&owner, &batch)?;
                    return Ok(batch);
                }
                Err(ForwardSourceFailure::Storage(error)) => return Err(error),
            };
            if Self::source_entry_requires_preserving_forward_info(&source_entry)
                && (batch.context.pointer("/privacy/dropSenderNames").and_then(serde_json::Value::as_bool) == Some(true)
                    || batch.context.pointer("/privacy/dropCaptions").and_then(serde_json::Value::as_bool) == Some(true))
            {
                let denial = "forwardHumanMessage source requires preserving forward attribution".to_string();
                for destination in &mut batch.destinations {
                    if !destination.completed {
                        destination.error = Some(denial.clone());
                    }
                }
                store_forward_batch(&owner, &batch)?;
                return Ok(batch);
            }
            if self.human_forward_scope()? != (account.to_string(), issuer.to_string()) {
                return Err("Fabushi native messaging credential changed Human account or issuer".into());
            }
            let destination_id = target.conversation_id.clone();
            let nonce = target.client_nonce.clone();
            let result = (|| -> Result<serde_json::Value, String> {
                let destination_owner = self.open_human_conversation_db_owner(&destination_id)?;
                let metadata = destination_owner.read_metadata().map_err(|error| error.to_string())?;
                let participants = metadata.get("participantIds").and_then(serde_json::Value::as_array)
                    .ok_or_else(|| "forward destination participants are invalid".to_string())?;
                if participants.len() != 2 || !self.metadata_has_local_human(&metadata)? {
                    return Err("forward destination is not an authorized Human conversation".into());
                }
                let mut requirements = Self::human_recipient_requirements_from_entry(&source_entry);
                requirements.require_media |= !batch.attachments.is_empty();
                let auth_input = Self::human_recipient_authorization_from_metadata(&metadata, true);
                let permitted = fabushi_messaging_core::recipient_search_authorized(
                    auth_input,
                    requirements,
                );
                if !permitted {
                    return Err("forward destination rejected the source message policy".into());
                }
                self.append_human_message_from_intent(
                    &destination_id, &batch.text, &nonce, None, None,
                    &batch.attachments, Some(&batch.context), None, require_existing, false,
                )
            })();
            match result {
                Ok(_) => {
                    batch.destinations[index].completed = true;
                    batch.destinations[index].error = None;
                }
                Err(error) => {
                    batch.destinations[index].completed = false;
                    batch.destinations[index].error = Some(error);
                }
            }
            // A failed progress write must stop any later destination. A
            // successfully sent destination can replay through its canonical
            // nonce after a crash between its HTTP settlement and this write.
            store_forward_batch(&owner, &batch)?;
        }
        Ok(batch)
    }

    fn resume_human_forward_batches(&self) -> Result<(), String> {
        if self.shipping_native_messaging()?.is_none() { return Ok(()); }
        let (account, issuer) = self.human_forward_scope()?;
        // Ordinary roster discovery is best-effort. Durable forward recovery
        // must instead surface unreadable or malformed stores before dispatch,
        // without turning another account's valid source into local authority.
        let root = get_native_conversations_root(&self.agents_root);
        let entries = match fs::read_dir(root) {
            Ok(entries) => entries,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.to_string()),
        };
        let mut sources = Vec::new();
        for entry in entries {
            let entry = entry.map_err(|error| error.to_string())?;
            let kind = entry.file_type().map_err(|error| error.to_string())?;
            if kind.is_symlink() {
                return Err("Human forward source directory is a symbolic link; retained identity cannot be skipped".into());
            }
            if !kind.is_dir() { continue; }
            let id = entry.file_name().to_string_lossy().into_owned();
            let owner = self.open_human_forward_recovery_owner(&id)?;
            // An account-owned intent remains discoverable after source
            // membership is revoked. Delivery persists its policy denial;
            // other accounts' valid batches are filtered below and untouched.
            let batches = load_forward_batches(&owner, &id)?;
            sources.push((id, batches));
        }
        for (source_id, batches) in sources {
            for batch in batches {
                if batch.account_id != account || batch.issuer != issuer
                    || batch.destinations.iter().all(|target| target.completed)
                { continue; }
                // Do not erase or bypass an invalid source on recovery. An
                // unavailable source retains the original intent, and no
                // destination is sent without reauthorization.
                let serial = self.human_message_lock(&format!(
                    "forward-batch:{account}:{issuer}:{}", batch.request_id,
                ))?;
                let exclusive = match serial.try_lock() {
                    Ok(guard) => guard,
                    Err(std::sync::TryLockError::WouldBlock) => continue,
                    Err(std::sync::TryLockError::Poisoned(_)) => {
                        return Err("Human forward batch lock poisoned".to_string());
                    }
                };
                // Keep the admitted guard through delivery. Releasing it here
                // and taking the blocking wrapper would stall history again.
                self.deliver_forward_batch_while_locked(
                    &source_id, &batch.request_id, None, false, &account, &issuer, &exclusive,
                )?;
            }
        }
        Ok(())
    }

    pub fn forward_human_message(
        &self,
        source_conversation_id: &str,
        source_entry_id: &str,
        destination_conversation_ids: &[String],
        client_nonce: &str,
        drop_sender_names: bool,
        drop_captions: bool,
    ) -> Result<serde_json::Value, String> {
        let client_nonce = client_nonce.trim();
        if client_nonce.is_empty() || client_nonce.len() > 200 {
            return Err("forwardHumanMessage requires a valid clientNonce".into());
        }
        let source_entry_id = source_entry_id.trim();
        if source_entry_id.is_empty() {
            return Err("forwardHumanMessage requires a sourceEntryId".into());
        }
        if destination_conversation_ids.is_empty() {
            return Err("forwardHumanMessage requires at least one destination".into());
        }
        let mut seen = BTreeSet::new();
        let mut destinations = Vec::new();
        for id in destination_conversation_ids {
            let id = id.trim();
            if id.is_empty() {
                return Err("forwardHumanMessage destination id must not be empty".into());
            }
            if seen.insert(id.to_string()) { destinations.push(id.to_string()); }
        }
        if destinations.len() > 32 {
            return Err("forwardHumanMessage supports at most 32 unique destinations".into());
        }
        let requested_privacy = fabushi_messaging_core::ForwardPrivacy { drop_sender_names, drop_captions }.normalized();
        let (account, issuer) = self.human_forward_scope()?;
        // Serialize request-ID discovery and initial source persistence by
        // account/issuer. Release this short lane before destination locks/HTTP.
        let staging = self.human_message_lock(&format!("forward-account:{account}:{issuer}"))?;
        let stage_guard = staging.lock().map_err(|_| "Human forward staging lock poisoned".to_string())?;
        let existing = self.find_forward_request(&account, &issuer, client_nonce)?;
        if let Some(ref prior) = existing {
            if prior.source_conversation_id != source_conversation_id || prior.source_entry_id != source_entry_id {
                return Err("forwardHumanMessage clientNonce already belongs to another source message".into());
            }
            let forced_preserve = prior
                .context
                .pointer("/privacy/forcedPreserveInfo")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false);
            let privacy = requested_privacy.normalized_for_source(forced_preserve);
            if prior.context.pointer("/privacy/dropSenderNames").and_then(serde_json::Value::as_bool)
                != Some(privacy.drop_sender_names)
                || prior.context.pointer("/privacy/dropCaptions").and_then(serde_json::Value::as_bool)
                    != Some(privacy.drop_captions)
            {
                return Err("forwardHumanMessage clientNonce already identifies different privacy".into());
            }
            if !prior.contains_requested(&destinations) {
                return Err("forwardHumanMessage request may not expand its original destination set".into());
            }
        } else {
            let source_entry = self.authorize_forward_source(source_conversation_id, source_entry_id)
                .map_err(|error| error.to_string())?;
            let force_preserve = Self::source_entry_requires_preserving_forward_info(&source_entry);
            let privacy = requested_privacy.normalized_for_source(force_preserve);
            let source_text = source_entry.get("content").and_then(serde_json::Value::as_str).unwrap_or_default();
            let attachments = source_entry.get("attachments")
                .and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
            let source_owner = self.open_human_conversation_db_owner(source_conversation_id)?;
            // A path is only an input to accepting a new forward. Every later
            // destination uses the source Session's immutable byte snapshot.
            // No resource/message POST can precede this or the journal commit.
            let attachments = snapshot_forward_attachments(
                &source_owner, &account, &issuer, client_nonce, &attachments,
            )?;
            if self.human_forward_scope()? != (account.clone(), issuer.clone()) {
                return Err("Fabushi native messaging credential changed Human account or issuer".into());
            }
            let text = if privacy.drop_captions && !attachments.is_empty() {
                String::new()
            } else {
                source_text.to_string()
            };
            let mut privacy_json = serde_json::json!({
                "dropSenderNames": privacy.drop_sender_names,
                "dropCaptions": privacy.drop_captions,
            });
            if force_preserve {
                privacy_json
                    .as_object_mut()
                    .expect("privacy object")
                    .insert("forcedPreserveInfo".into(), serde_json::Value::Bool(true));
            }
            let mut context = serde_json::json!({
                "batchVersion": 1,
                "requestId": client_nonce,
                "sourceConversationId": source_conversation_id,
                "sourceEntryId": source_entry_id,
                "privacy": privacy_json,
            });
            let source_metadata = source_owner.read_metadata().ok();
            if let Some(origin) = Self::resolve_human_forward_origin(
                &source_entry,
                source_metadata.as_ref(),
                &account,
                privacy,
            ) {
                context
                    .as_object_mut()
                    .expect("forward context object")
                    .insert("origin".into(), origin);
            }
            let batch = HumanForwardBatch::new(
                account, issuer, client_nonce.to_string(), source_conversation_id.to_string(),
                source_entry_id.to_string(), text, attachments, context, &destinations,
            );
            // The complete ordered batch (including destinations not yet
            // dispatched) commits atomically before the first POST.
            store_forward_batch(&source_owner, &batch)?;
        }
        drop(stage_guard);
        let batch = self.deliver_forward_batch(source_conversation_id, client_nonce, Some(&destinations))?;
        Ok(batch.response(&destinations))
    }

    fn normalize_human_reply_identity(
        existing: &[serde_json::Value],
        requested: Option<&str>,
        local_human_id: &str,
    ) -> Result<Option<String>, String> {
        let Some(reply_id) = requested.map(str::trim).filter(|id| !id.is_empty()) else {
            return Ok(None);
        };
        let local_nonce = reply_id.strip_prefix("human-message:");
        if let Some(target) = existing.iter().find(|row| {
            row.get("id").and_then(serde_json::Value::as_str) == Some(reply_id)
        }) {
            if let Some(nonce) = local_nonce {
                if nonce.is_empty()
                    || target.get("kind").and_then(serde_json::Value::as_str) != Some("message")
                    || target.get("authorId").and_then(serde_json::Value::as_str) != Some(local_human_id)
                    || target.get("authorKind").and_then(serde_json::Value::as_str) != Some("human")
                    || target.get("clientNonce").and_then(serde_json::Value::as_str) != Some(nonce)
                    || target.get("remoteMessageId").is_some()
                    || !matches!(target.get("delivery").and_then(serde_json::Value::as_str),
                        Some("pending" | "dispatching" | "sent"))
                {
                    return Err("sendHumanMessage local reply identity conflicts with its author, nonce or settlement".into());
                }
                return Ok(Some(reply_id.to_string()));
            }
            let remote_id = target.get("remoteMessageId")
                .map(fabushi_message_identity_text).transpose()
                .map_err(|error| format!("sendHumanMessage reply remote identity is invalid: {error}"))?
                .ok_or_else(|| "sendHumanMessage reply has no canonical remote identity".to_string())?;
            if reply_id != format!("human-server-message:{remote_id}")
                || target.get("kind").and_then(serde_json::Value::as_str) != Some("message")
                || target.get("delivery").and_then(serde_json::Value::as_str) != Some("sent")
                || target.get("authorKind").and_then(serde_json::Value::as_str) != Some("human")
                || !target.get("authorId").and_then(serde_json::Value::as_str)
                    .is_some_and(|author| !author.trim().is_empty())
            {
                return Err("sendHumanMessage reply canonical identity is inconsistent".into());
            }
            return Ok(Some(reply_id.to_string()));
        }
        // Older versions could rekey the parent while leaving a durable local
        // reply ID behind. Its exact nonce is evidence only for this author's
        // row in this conversation; peer provenance and other stores are never
        // aliases. Ambiguous or damaged canonical rows remain fail-closed.
        let nonce = local_nonce.filter(|nonce| !nonce.is_empty())
            .ok_or_else(|| "sendHumanMessage reply target is not in this Human conversation".to_string())?;
        let mut candidates = existing.iter().filter(|row| {
            row.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id)
                && row.get("clientNonce").and_then(serde_json::Value::as_str) == Some(nonce)
        });
        let target = candidates.next()
            .ok_or_else(|| "sendHumanMessage reply target is not in this Human conversation".to_string())?;
        if candidates.next().is_some() {
            return Err("sendHumanMessage local reply nonce has ambiguous canonical identities".into());
        }
        let remote_id = target.get("remoteMessageId").map(fabushi_message_identity_text).transpose()
            .map_err(|error| format!("sendHumanMessage local reply remote identity is invalid: {error}"))?
            .ok_or_else(|| "sendHumanMessage local reply has no canonical remote identity".to_string())?;
        let canonical_id = format!("human-server-message:{remote_id}");
        if target.get("id").and_then(serde_json::Value::as_str) != Some(canonical_id.as_str())
            || target.get("kind").and_then(serde_json::Value::as_str) != Some("message")
            || target.get("delivery").and_then(serde_json::Value::as_str) != Some("sent")
            || target.get("authorKind").and_then(serde_json::Value::as_str) != Some("human")
        {
            return Err("sendHumanMessage local reply canonical identity is inconsistent".into());
        }
        Ok(Some(canonical_id))
    }

    /// Retry is an identity-only operation on the existing durable intent. It
    /// never accepts replacement payload or reconstructs committed resources.
    pub fn retry_human_message(
        &self, conversation_id: &str, client_nonce: &str,
    ) -> Result<serde_json::Value, String> {
        if conversation_id.is_empty() || conversation_id != conversation_id.trim()
            || client_nonce.is_empty() || client_nonce != client_nonce.trim() || client_nonce.len() > 200
        {
            return Err("retryHumanMessage requires unpadded conversationId and clientNonce (at most 200 UTF-8 bytes)".into());
        }
        let (account, issuer) = self.human_forward_scope()?;
        // Serialize only explicit retries of this one intent. The ordinary
        // conversation mutation lane remains free during the message POST.
        let retry_key = serde_json::to_string(&("human-retry", &account, &issuer, conversation_id, client_nonce))
            .map_err(|error| error.to_string())?;
        let lane = HumanRetryLane {
            serial: Some(self.human_message_lock(&retry_key)?), key: retry_key, registry: &self.human_message_locks,
        };
        let _retry_guard = lane.serial.as_ref().expect("active retry lane").lock()
            .map_err(|_| "Human retry lock poisoned".to_string())?;
        if self.human_forward_scope()? != (account.clone(), issuer.clone()) {
            return Err("Human retry account or issuer changed while awaiting admission".into());
        }
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let snapshot = {
            let serial = self.human_message_lock(conversation_id)?;
            let _staging = serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?;
            if self.human_forward_scope()? != (account.clone(), issuer.clone()) {
                return Err("Human retry account or issuer changed while awaiting conversation admission".into());
            }
            let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
            Self::validate_human_retry_conversation(&metadata, conversation_id, &account)?;
            if !self.metadata_has_local_human(&metadata)? {
                return Err("local Human identity is not a conversation participant".into());
            }
            self.peer_human_id_from_metadata(&metadata)?;
            let entries = owner.get_transcript_entries().map_err(|error| error.to_string())?;
            let entry = Self::human_retry_entry(&entries, &account, client_nonce)?;
            if entry.get("delivery").and_then(serde_json::Value::as_str) == Some("sent") {
                return Ok(entry.clone());
            }
            entry.clone()
        };
        if self.shipping_native_messaging()?.is_none() {
            return Err("Human retry requires the Native messaging service for an unsettled intent".into());
        }
        let mut admitted_retry = false;
        let result = (|| -> Result<serde_json::Value, String> {
            if self.human_forward_scope()? != (account.clone(), issuer.clone()) {
                return Err("Human retry account or issuer changed before dispatch".into());
            }
            if self.pending_human_forward_is_batch_owned(conversation_id, &snapshot)?
                || snapshot.get("forwardContext").is_some()
            {
                let context = snapshot.get("forwardContext")
                    .filter(|context| context.get("batchVersion").and_then(serde_json::Value::as_u64) == Some(1))
                    .ok_or_else(|| "Human retry requires the original supported forward source context".to_string())?;
                let source_id = context.get("sourceConversationId").and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty() && *value == value.trim())
                    .ok_or_else(|| "Human retry forward source is missing or invalid".to_string())?;
                let request_id = context.get("requestId").and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty() && *value == value.trim())
                    .ok_or_else(|| "Human retry forward request is missing or invalid".to_string())?;
                let batch = self.find_forward_request(&account, &issuer, request_id)?
                    .ok_or_else(|| "Human retry original forward batch is unavailable".to_string())?;
                if batch.source_conversation_id != source_id || &batch.context != context
                    || !batch.destinations.iter().any(|target|
                        target.conversation_id == conversation_id && target.client_nonce == client_nonce)
                {
                    return Err("Human retry does not match its original forward source and target".into());
                }
                admitted_retry = true;
                let delivered = self.deliver_forward_batch_from_intent(source_id, request_id, Some(&[conversation_id.to_string()]), true)?;
                let target = delivered.destinations.iter().find(|target|
                    target.conversation_id == conversation_id && target.client_nonce == client_nonce)
                    .ok_or_else(|| "Human retry original forward target disappeared".to_string())?;
                if !target.completed {
                    return Err(target.error.clone().unwrap_or_else(|| "Human forward retry remains unsettled".to_string()));
                }
                let serial = self.human_message_lock(conversation_id)?;
                let _settlement = serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?;
                if self.human_forward_scope()? != (account.clone(), issuer.clone()) {
                    return Err("Human retry account or issuer changed before settlement".into());
                }
                let entries = owner.get_transcript_entries().map_err(|error| error.to_string())?;
                let settled = Self::human_retry_entry(&entries, &account, client_nonce)?;
                if settled.get("delivery").and_then(serde_json::Value::as_str) != Some("sent") {
                    return Err("Human forward journal completed without a canonical target settlement".into());
                }
                return Ok(settled.clone());
            }
            admitted_retry = true;
            self.append_human_message_from_intent(
                conversation_id,
                snapshot.get("content").and_then(serde_json::Value::as_str).expect("validated retry content"),
                client_nonce,
                snapshot.get("composedAtMs").and_then(serde_json::Value::as_f64),
                snapshot.get("replyToId").and_then(serde_json::Value::as_str),
                snapshot.get("attachments").and_then(serde_json::Value::as_array).map(Vec::as_slice).unwrap_or(&[]),
                None, None, true, false,
            )
        })();
        let result = result.and_then(|settled| {
            if self.human_forward_scope()? != (account.clone(), issuer.clone()) {
                return Err("Human retry account or issuer changed before returning settlement".into());
            }
            Ok(settled)
        });
        if let (true, Err(error)) = (admitted_retry, &result) {
            // Source-policy and provenance failures happen before the common
            // sender. Preserve them on the same pending row for reload/UI,
            // without ever resurrecting a row already settled by sync.
            let serial = self.human_message_lock(conversation_id)?;
            let _failure = serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?;
            let entries = owner.get_transcript_entries().map_err(|error| error.to_string())?;
            if let Ok(current) = Self::human_retry_entry(&entries, &account, client_nonce) {
                if current.get("remoteMessageId").is_none()
                    && current.get("content") == snapshot.get("content")
                    && current.get("forwardContext") == snapshot.get("forwardContext")
                    && current.get("lastDispatchError").and_then(serde_json::Value::as_str) != Some(error.as_str())
                {
                    let mut failed = current.clone();
                    failed["delivery"] = serde_json::json!("pending");
                    failed["lastDispatchError"] = serde_json::json!(error);
                    failed["lastDispatchFailedAtMs"] = serde_json::json!(
                        std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64);
                    owner.update_transcript_entry(failed["id"].as_str().expect("validated retry identity"), &failed)
                        .map_err(|failure| format!("{error}; retry failure could not be persisted: {failure}"))?
                        .ok_or_else(|| format!("{error}; retry intent disappeared while preserving its error"))?;
                }
            }
        }
        result
    }

    fn validate_human_queue_origin(entry: &serde_json::Value) -> Result<Option<bool>, String> {
        let origin = match entry.get("queuedOffline") {
            None => None,
            Some(serde_json::Value::Bool(value)) => Some(*value),
            Some(_) => return Err("Human queuedOffline origin must be a boolean".into()),
        };
        if origin == Some(true) && !entry.get("composedAtMs")
            .and_then(serde_json::Value::as_f64).is_some_and(f64::is_finite)
        {
            return Err("Human offline queue origin requires a finite composedAtMs".into());
        }
        Ok(origin)
    }

    fn human_retry_entry<'a>(
        entries: &'a [serde_json::Value], account: &str, client_nonce: &str,
    ) -> Result<&'a serde_json::Value, String> {
        let mut matches = entries.iter().filter(|entry|
            entry.get("authorId").and_then(serde_json::Value::as_str) == Some(account)
                && entry.get("clientNonce").and_then(serde_json::Value::as_str) == Some(client_nonce));
        let entry = matches.next().ok_or_else(|| "Human retry durable local intent was not found".to_string())?;
        if matches.next().is_some() {
            return Err("Human retry nonce identifies duplicate local intents".into());
        }
        if entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
            || entry.get("role").and_then(serde_json::Value::as_str) != Some("user")
            || entry.get("authorKind").and_then(serde_json::Value::as_str) != Some("human")
        {
            return Err("Human retry intent is not a local Human message".into());
        }
        Self::validate_human_queue_origin(entry)?;
        let content = entry.get("content").and_then(serde_json::Value::as_str)
            .ok_or_else(|| "Human retry persisted content is invalid".to_string())?;
        let attachments = match entry.get("attachments") {
            None => &[][..],
            Some(serde_json::Value::Array(items)) => items.as_slice(),
            Some(_) => return Err("Human retry persisted attachments are invalid".into()),
        };
        if (content.trim().is_empty() && attachments.is_empty()) || attachments.len() > 6 {
            return Err("Human retry persisted message payload is invalid".into());
        }
        for attachment in attachments {
            for field in ["path", "name"] {
                if attachment.get(field).and_then(serde_json::Value::as_str)
                    .filter(|value| !value.is_empty() && *value == value.trim()).is_none()
                {
                    return Err(format!("Human retry persisted attachment {field} is invalid"));
                }
            }
        }
        for field in ["composedAtMs", "timestampMs"] {
            if entry.get(field).is_some_and(|value| !value.is_null()
                && !value.as_f64().is_some_and(f64::is_finite))
            {
                return Err(format!("Human retry persisted {field} is invalid"));
            }
        }
        if entry.get("replyToId").is_some_and(|value| !value.is_null()
            && !value.as_str().is_some_and(|value| !value.is_empty() && value == value.trim()))
        {
            return Err("Human retry persisted reply identity is invalid".into());
        }
        if let Some(dispatched) = entry.get("remoteDispatchAttachments") {
            let resources = serde_json::from_value::<Vec<FabushiRemoteHumanAttachment>>(dispatched.clone())
                .map_err(|error| format!("Human retry persisted dispatch resources are invalid: {error}"))?;
            for resource in &resources {
                Self::validate_human_attachment_resource(resource)?;
            }
            if resources.len() != attachments.len() || resources.iter().zip(attachments).any(|(resource, attachment)|
                resource.resource_id.trim().is_empty() || resource.resource_id != resource.resource_id.trim()
                    || attachment.get("name").and_then(serde_json::Value::as_str) != Some(resource.name.as_str()))
            {
                return Err("Human retry persisted dispatch resources conflict with the original attachments".into());
            }
        }
        if entry.get("delivery").and_then(serde_json::Value::as_str) == Some("sent") {
            let remote_id = entry.get("remoteMessageId").ok_or_else(||
                "Human retry settled intent lacks a canonical remote identity".to_string())
                .and_then(fabushi_message_identity_text)?;
            if entry.get("id").and_then(serde_json::Value::as_str) != Some(format!("human-server-message:{remote_id}").as_str()) {
                return Err("Human retry settled intent has a conflicting canonical identity".into());
            }
        } else if entry.get("remoteMessageId").is_some()
            || !matches!(entry.get("delivery").and_then(serde_json::Value::as_str), Some("pending" | "dispatching"))
            || entry.get("id").and_then(serde_json::Value::as_str) != Some(format!("human-message:{client_nonce}").as_str())
        {
            return Err("Human retry intent is not in an admitted pending or canonical state".into());
        }
        Ok(entry)
    }

    pub fn append_human_message(
        &self,
        conversation_id: &str,
        text: &str,
        client_nonce: &str,
        composed_at_ms: Option<f64>,
        reply_to_id: Option<&str>,
        attachments: &[serde_json::Value],
    ) -> Result<serde_json::Value, String> {
        self.append_human_message_with_queue_origin(
            conversation_id, text, client_nonce, composed_at_ms, reply_to_id, attachments, false,
        )
    }

    pub fn append_human_message_with_queue_origin(
        &self,
        conversation_id: &str,
        text: &str,
        client_nonce: &str,
        composed_at_ms: Option<f64>,
        reply_to_id: Option<&str>,
        attachments: &[serde_json::Value],
        queued_offline: bool,
    ) -> Result<serde_json::Value, String> {
        self.append_human_message_with_context(
            conversation_id,
            text,
            client_nonce,
            composed_at_ms,
            reply_to_id,
            attachments,
            None,
            None,
            queued_offline,
        )
    }

    fn append_human_message_with_context(
        &self,
        conversation_id: &str,
        text: &str,
        client_nonce: &str,
        composed_at_ms: Option<f64>,
        reply_to_id: Option<&str>,
        attachments: &[serde_json::Value],
        forward_context: Option<&serde_json::Value>,
        held_conversation_guard: Option<&std::sync::MutexGuard<'_, ()>>,
        queued_offline: bool,
    ) -> Result<serde_json::Value, String> {
        self.append_human_message_from_intent(
            conversation_id, text, client_nonce, composed_at_ms, reply_to_id, attachments,
            forward_context, held_conversation_guard, false, queued_offline,
        )
    }

    fn append_human_message_from_intent(
        &self,
        conversation_id: &str,
        text: &str,
        client_nonce: &str,
        composed_at_ms: Option<f64>,
        reply_to_id: Option<&str>,
        attachments: &[serde_json::Value],
        forward_context: Option<&serde_json::Value>,
        held_conversation_guard: Option<&std::sync::MutexGuard<'_, ()>>,
        require_existing: bool,
        queued_offline: bool,
    ) -> Result<serde_json::Value, String> {
        let sender_id = self.local_human_id()?;
        let text = text.trim();
        let client_nonce = client_nonce.trim();
        if client_nonce.is_empty() {
            return Err("sendHumanMessage requires clientNonce".into());
        }
        if text.is_empty() && attachments.is_empty() {
            return Err("sendHumanMessage requires text or attachments".into());
        }
        if client_nonce.len() > 200 {
            return Err("sendHumanMessage clientNonce is too long".into());
        }
        if queued_offline && !composed_at_ms.is_some_and(f64::is_finite) {
            return Err("Human offline queue origin requires a finite composedAtMs".into());
        }
        let serial = self.human_message_lock(conversation_id)?;
        let mut staging_guard = if held_conversation_guard.is_none() {
            Some(serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?)
        } else {
            None
        };
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        let is_participant = metadata
            .get("participantIds")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(sender_id)));
        if !is_participant {
            return Err("local Human identity is not a conversation participant".into());
        }
        if require_existing {
            Self::validate_human_retry_conversation(&metadata, conversation_id, sender_id)?;
        }
        let existing = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        if require_existing {
            let current = Self::human_retry_entry(&existing, sender_id, client_nonce)?;
            if current.get("delivery").and_then(serde_json::Value::as_str) == Some("sent") {
                return Ok(current.clone());
            }
        }
        let normalized_reply_id = Self::normalize_human_reply_identity(&existing, reply_to_id, sender_id)?;
        let normalized_reply = normalized_reply_id.as_deref();
        let normalized_attachments = attachments
            .iter()
            .map(|attachment| {
                let object = attachment
                    .as_object()
                    .ok_or_else(|| "sendHumanMessage attachment must be an object".to_string())?;
                let path = object
                    .get("path")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "sendHumanMessage attachment requires path".to_string())?;
                let name = object
                    .get("name")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| "sendHumanMessage attachment requires name".to_string())?;
                let mut normalized = serde_json::json!({ "path": path, "name": name });
                if let Some(mime_type) = object
                    .get("mimeType")
                    .and_then(serde_json::Value::as_str)
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                {
                    normalized
                        .as_object_mut()
                        .expect("attachment object")
                        .insert("mimeType".into(), serde_json::Value::String(mime_type.into()));
                }
                for dim in ["width", "height"] {
                    if let Some(value) = object
                        .get(dim)
                        .and_then(|value| {
                            value.as_i64().or_else(|| {
                                value
                                    .as_f64()
                                    .filter(|number| number.is_finite())
                                    .map(|number| number.round() as i64)
                            })
                        })
                        .filter(|value| *value > 0)
                    {
                        normalized
                            .as_object_mut()
                            .expect("attachment object")
                            .insert(dim.into(), serde_json::json!(value));
                    }
                }
                if forward_context.is_some() {
                    if let Some(snapshot) = object.get("forwardSnapshot") {
                        normalized.as_object_mut().expect("attachment object")
                            .insert("forwardSnapshot".into(), snapshot.clone());
                    }
                }
                Ok(normalized)
            })
            .collect::<Result<Vec<_>, String>>()?;
        let timestamp_ms = composed_at_ms.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        });
        if !timestamp_ms.is_finite() {
            return Err("Human message activity timestamp must be finite".into());
        }
        let mut existing_pending: Option<serde_json::Value> = None;
        if let Some(entry) = existing.iter().find(|entry| {
            entry.get("authorId").and_then(serde_json::Value::as_str) == Some(sender_id)
                && entry.get("clientNonce").and_then(serde_json::Value::as_str) == Some(client_nonce)
        }) {
            Self::validate_human_queue_origin(entry)?;
            if forward_context.is_none()
                && entry.get("remoteMessageId").is_none()
                && self.pending_human_forward_is_batch_owned(conversation_id, entry)?
            {
                return Err("source-owned Human pending intent requires its original forward batch".into());
            }
            let same_sender = entry.get("authorId").and_then(serde_json::Value::as_str) == Some(sender_id);
            let same_text = entry.get("content").and_then(serde_json::Value::as_str) == Some(text);
            let persisted_reply = Self::normalize_human_reply_identity(
                &existing, entry.get("replyToId").and_then(serde_json::Value::as_str), sender_id,
            )?;
            let same_reply = persisted_reply.as_deref() == normalized_reply;
            if let Some(dispatched_reply) = entry.get("remoteReplyToMessageId") {
                let expected = normalized_reply.and_then(|reply_id| existing.iter().find(|row| {
                    row.get("id").and_then(serde_json::Value::as_str) == Some(reply_id)
                })).and_then(|row| row.get("remoteMessageId"))
                    .map(fabushi_identity_text).transpose()?;
                let dispatched_reply = fabushi_identity_text(dispatched_reply)?;
                if expected.as_deref() != Some(dispatched_reply.as_str()) {
                    return Err("sendHumanMessage reply conflicts with its dispatched remote identity".into());
                }
            }
            let same_attachments = match entry.get("attachments").and_then(serde_json::Value::as_array) {
                Some(values) => values.len() == normalized_attachments.len()
                    && values.iter().zip(normalized_attachments.iter()).all(|(current, requested)| {
                        current.get("name") == requested.get("name")
                            && (current.get("path") == requested.get("path") || current.get("resourceId").is_some())
                    }),
                None => normalized_attachments.is_empty(),
            };
            let same_forward_context = entry.get("forwardContext") == forward_context;
            if !(same_sender && same_text && same_reply && same_attachments && same_forward_context) {
                return Err("sendHumanMessage clientNonce already identifies different content or forward context".into());
            }
            if entry.get("remoteMessageId").is_some()
                || entry.get("delivery").and_then(serde_json::Value::as_str) == Some("sent")
            {
                let timestamp = entry.get("timestampMs").and_then(serde_json::Value::as_f64)
                    .unwrap_or(timestamp_ms);
                Self::update_human_activity(&owner, timestamp)?;
                return Ok(entry.clone());
            }
            if !matches!(
                entry.get("delivery").and_then(serde_json::Value::as_str),
                Some("pending" | "dispatching")
            ) {
                return Err("sendHumanMessage clientNonce is not in a retryable settlement state".into());
            }
            if entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
                || entry.get("authorKind").and_then(serde_json::Value::as_str) != Some("human")
                || entry.get("id").and_then(serde_json::Value::as_str) != Some(format!("human-message:{client_nonce}").as_str())
            {
                return Err("sendHumanMessage pending identity is not a local Human outbox row".into());
            }
            existing_pending = Some(entry.clone());
        }

        if let Some(client) = self.shipping_native_messaging()? {
            let local_entry_id = format!("human-message:{client_nonce}");
            let mut pending_entry = existing_pending.unwrap_or_else(|| {
                let mut entry = serde_json::json!({
                    "id": local_entry_id.clone(),
                    "kind": "message",
                    "role": "user",
                    "authorKind": "human",
                    "authorId": sender_id,
                    "content": text,
                    "clientNonce": client_nonce,
                    "composedAtMs": composed_at_ms,
                    "timestampMs": timestamp_ms,
                    "delivery": "pending",
                });
                let object = entry.as_object_mut().expect("Human pending message must be an object");
                if queued_offline { object.insert("queuedOffline".into(), serde_json::json!(true)); }
                if let Some(reply_id) = normalized_reply { object.insert("replyToId".into(), serde_json::json!(reply_id)); }
                if !normalized_attachments.is_empty() { object.insert("attachments".into(), serde_json::Value::Array(normalized_attachments.clone())); }
                if let Some(context) = forward_context { object.insert("forwardContext".into(), context.clone()); }
                entry
            });
            if let Some(reply_id) = normalized_reply {
                pending_entry["replyToId"] = serde_json::json!(reply_id);
            }
            if forward_context.is_some_and(|context| context.get("batchVersion").is_some()) {
                pending_entry.as_object_mut().expect("Human pending message must be an object")
                    .insert("forwardBatchOwned".into(), serde_json::json!(true));
            }
            if !existing.iter().any(|entry| entry.get("id") == pending_entry.get("id"))
                && !owner.append_transcript_entry(&pending_entry).map_err(|error| error.to_string())?
            {
                return Err("Human pending send could not be durably recorded before dispatch".into());
            }
            if existing.iter().any(|entry| entry.get("id") == pending_entry.get("id")
                && entry.get("forwardBatchOwned") != pending_entry.get("forwardBatchOwned"))
            {
                owner.update_transcript_entry(&local_entry_id, &pending_entry)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "Human forward ownership could not be durably recorded before dispatch".to_string())?;
            }

            let dispatch_result = (|| -> Result<serde_json::Value, String> {
                let peer_human_id = self.peer_human_id_from_metadata(&metadata)?;
                let remote_reply_to_id = normalized_reply
                    .map(|reply_id| {
                        owner
                            .get_transcript_entries()
                            .map_err(|error| error.to_string())?
                            .into_iter()
                            .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(reply_id))
                            .and_then(|entry| entry.get("remoteMessageId").cloned())
                            .map(|value| fabushi_identity_text(&value))
                            .transpose()?
                            .ok_or_else(|| "Fabushi Human reply target has no canonical remote message identity".to_string())
                    })
                    .transpose()?;

                let mut uploaded = pending_entry
                    .get("remoteDispatchAttachments")
                    .cloned()
                    .map(serde_json::from_value::<Vec<FabushiRemoteHumanAttachment>>)
                    .transpose()
                    .map_err(|error| format!("persisted Human dispatch attachments are invalid: {error}"))?
                    .filter(|values| {
                        values.len() == normalized_attachments.len()
                            && values.iter().zip(normalized_attachments.iter()).all(|(remote, local)| {
                                local.get("name").and_then(serde_json::Value::as_str) == Some(remote.name.as_str())
                            })
                    })
                    .unwrap_or_default();
                if uploaded.is_empty() && !normalized_attachments.is_empty() {
                    uploaded = Vec::with_capacity(normalized_attachments.len());
                    let mut forward_bytes = if let Some(context) = forward_context.filter(|context|
                        context.get("batchVersion").and_then(serde_json::Value::as_u64) == Some(1)
                    ) {
                        let source_id = context.get("sourceConversationId")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| "Human forward source identity is missing".to_string())?;
                        let request_id = context.get("requestId")
                            .and_then(serde_json::Value::as_str)
                            .ok_or_else(|| "Human forward request identity is missing".to_string())?;
                        let source_owner = self.open_human_conversation_db_owner(source_id)?;
                        let (account, issuer) = self.human_forward_scope()?;
                        if normalized_attachments.len() > 6 {
                            return Err("Human forward exceeds the six-attachment backend limit".into());
                        }
                        // Validate the complete target before any resource
                        // POST, then consume those exact buffers. This is
                        // bounded by six files of at most 32 MiB each and
                        // cannot reopen a changed path after preflight.
                        let mut verified = Vec::with_capacity(normalized_attachments.len());
                        for (index, attachment) in normalized_attachments.iter().enumerate() {
                            verified.push(read_forward_attachment_bytes(
                                &source_owner, &account, &issuer, request_id, index, attachment,
                            )?);
                        }
                        Some(verified.into_iter())
                    } else {
                        None
                    };
                    for attachment in &normalized_attachments {
                        let path = attachment.get("path").and_then(serde_json::Value::as_str)
                            .ok_or_else(|| "sendHumanMessage attachment requires path".to_string())?;
                        let name = attachment.get("name").and_then(serde_json::Value::as_str)
                            .ok_or_else(|| "sendHumanMessage attachment requires name".to_string())?;
                        let resource = if let Some(buffers) = forward_bytes.as_mut() {
                            let bytes = buffers.next().ok_or_else(||
                                "Human forward attachment preflight omitted a buffer".to_string())?;
                            // Use the verified buffer itself. Reopening its
                            // path in the uploader would reintroduce the race.
                            client.upload_direct_message_resource_bytes_for_user(bytes, name, sender_id)?
                        } else {
                            client.upload_direct_message_resource_for_user(std::path::Path::new(path), name, sender_id)?
                        };
                        uploaded.push(resource);
                    }
                }
                if let Some(object) = pending_entry.as_object_mut() {
                    object.insert("delivery".into(), serde_json::json!("dispatching"));
                    object.remove("lastDispatchError");
                    object.remove("lastDispatchFailedAtMs");
                    if !uploaded.is_empty() {
                        object.insert("remoteDispatchAttachments".into(), serde_json::to_value(&uploaded)
                            .map_err(|error| format!("Human dispatch attachments could not be persisted: {error}"))?);
                    }
                }
                owner.update_transcript_entry(&local_entry_id, &pending_entry)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "Human pending send disappeared before dispatch".to_string())?;

                // Initial staging is visible atomically to a history snapshot.
                // Release before the message POST so sync can discover an
                // accepted send while this request is still in flight.
                drop(staging_guard.take());
                let request_credential_fingerprint = client.request_credential_fingerprint()?;
                let response = client.send_direct_message_for_settlement(&peer_human_id, text, client_nonce, remote_reply_to_id.as_deref(), &uploaded, sender_id);
                // The POST may outlive a sync/reaction that already settled the
                // same nonce. Rejoin the existing conversation mutation lane
                // before inspecting or applying its eventual response. Sync
                // supplies its guard when replaying a persisted send, avoiding
                // recursive acquisition of the non-reentrant mutex.
                let serial = self.human_message_lock(conversation_id)?;
                // The transport checked credential stability at send/body read.
                // Recheck after regaining the Session lock: credentials may
                // change while an old response waits behind sync/reaction.
                let _settlement_guard = Self::acquire_human_send_settlement_guard(
                    client, serial.as_ref(), held_conversation_guard,
                    &request_credential_fingerprint, &response,
                )?;
                let durable = owner.get_transcript_entries().map_err(|error| error.to_string())?;
                let settled = durable.iter().find(|entry| {
                    let remote_id = entry.get("remoteMessageId").and_then(serde_json::Value::as_str);
                    let resources = entry.get("attachments").and_then(serde_json::Value::as_array)
                        .map(|items| items.iter().map(|item| item.get("resourceId").and_then(serde_json::Value::as_str))
                            .collect::<Vec<_>>()).unwrap_or_default();
                    remote_id.is_some_and(|id| entry.get("id").and_then(serde_json::Value::as_str)
                        == Some(format!("human-server-message:{id}").as_str()))
                        && entry.get("delivery").and_then(serde_json::Value::as_str) == Some("sent")
                        && entry.get("authorId").and_then(serde_json::Value::as_str) == Some(sender_id)
                        && entry.get("clientNonce").and_then(serde_json::Value::as_str) == Some(client_nonce)
                        && entry.get("content").and_then(serde_json::Value::as_str) == Some(text)
                        && entry.get("remoteReplyToMessageId").and_then(serde_json::Value::as_str)
                            == remote_reply_to_id.as_deref()
                        && resources == uploaded.iter().map(|item| Some(item.resource_id.as_str())).collect::<Vec<_>>()
                        && entry.get("forwardContext") == forward_context
                });
                let remote = match response {
                    Ok(remote) => remote,
                    Err(FabushiHumanSendError::Transport(error)) => {
                        let settled = settled.ok_or(error)?;
                        let timestamp = settled.get("timestampMs").and_then(serde_json::Value::as_f64)
                            .unwrap_or(timestamp_ms);
                        Self::update_human_activity(&owner, timestamp)?;
                        return Ok(settled.clone());
                    }
                    Err(FabushiHumanSendError::Rejected(error)) => return Err(error),
                };
                let remote_sender = fabushi_identity_text(&remote.sender_user_id)?;
                let remote_recipient = fabushi_identity_text(&remote.recipient_user_id)?;
                let remote_reply_matches = match (&remote_reply_to_id, &remote.reply_to_message_id) {
                    (None, None) => true,
                    (Some(expected), Some(actual)) => fabushi_identity_text(actual).is_ok_and(|actual| actual == *expected),
                    _ => false,
                };
                let remote_resources = remote.attachments.iter().map(|attachment| attachment.resource_id.as_str()).collect::<Vec<_>>();
                let uploaded_resources = uploaded.iter().map(|attachment| attachment.resource_id.as_str()).collect::<Vec<_>>();
                if remote_sender != sender_id
                    || remote_recipient != peer_human_id
                    || remote.text.trim() != text
                    || remote.client_request_id.as_deref() != Some(client_nonce)
                    || !remote_reply_matches
                    || remote_resources != uploaded_resources
                {
                    return Err("Fabushi Human message backend returned mismatched persisted content".into());
                }
                self.validate_remote_human_message(&remote, sender_id, &peer_human_id, &durable)?;
                chrono::DateTime::parse_from_rfc3339(&remote.created_at)
                    .map_err(|error| format!("remote Human message timestamp is invalid: {error}"))?;
                if let Some(settled) = settled {
                    // Admission still rejects a changed canonical identity,
                    // nonce, resources, reply or malformed reaction summary.
                    // A valid late acknowledgement has no freshness authority
                    // over reactions/receipts already persisted by sync or POST.
                    let timestamp = settled.get("timestampMs").and_then(serde_json::Value::as_f64)
                        .unwrap_or(timestamp_ms);
                    Self::update_human_activity(&owner, timestamp)?;
                    return Ok(settled.clone());
                }
                self.materialize_remote_human_message(&owner, &remote)
            })();
            // A resource/preflight failure may return before the POST release.
            drop(staging_guard.take());

            if let Err(error) = dispatch_result.as_ref() {
                // Settlement may have advanced while a transport/preflight
                // error was returning. Serialize this lookup and pending-only
                // update with sync so it cannot restore an obsolete row.
                let serial = self.human_message_lock(conversation_id)?;
                let _failure_guard = if held_conversation_guard.is_none() {
                    Some(serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?)
                } else {
                    None
                };
                if let Ok(entries) = owner.get_transcript_entries() {
                    if let Some(mut pending) = entries.into_iter().find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(local_entry_id.as_str())) {
                        if let Some(object) = pending.as_object_mut() {
                            object.insert("delivery".into(), serde_json::json!("pending"));
                            object.insert("lastDispatchError".into(), serde_json::json!(error));
                            object.insert("lastDispatchFailedAtMs".into(), serde_json::json!(
                                std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_millis() as f64
                            ));
                        }
                        let _ = owner.update_transcript_entry(&local_entry_id, &pending);
                    }
                }
            }
            return dispatch_result;
        }

        let mut entry = serde_json::json!({
            "id": format!("human-message:{client_nonce}"),
            "kind": "message",
            "role": "user",
            "authorKind": "human",
            "authorId": sender_id,
            "content": text,
            "clientNonce": client_nonce,
            "composedAtMs": composed_at_ms,
            "timestampMs": timestamp_ms,
            "delivery": "sent",
        });
        let object = entry
            .as_object_mut()
            .ok_or_else(|| "native Human message projection failed".to_string())?;
        if queued_offline { object.insert("queuedOffline".into(), serde_json::json!(true)); }
        if let Some(reply_id) = normalized_reply {
            object.insert("replyToId".into(), serde_json::json!(reply_id));
        }
        if !normalized_attachments.is_empty() {
            object.insert(
                "attachments".into(),
                serde_json::Value::Array(normalized_attachments.clone()),
            );
        }
        if let Some(context) = forward_context {
            object.insert("forwardContext".into(), context.clone());
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
                .ok_or_else(|| "native Human message was not durably appended".to_string())?;
            if replay == entry {
                Self::update_human_activity(&owner, timestamp_ms)?;
                return Ok(replay);
            }
            return Err("sendHumanMessage clientNonce already identifies different content".into());
        }
        Self::update_human_activity(&owner, timestamp_ms)?;
        Ok(entry)
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

    fn human_reaction_remote_identity(entry: &serde_json::Value) -> Result<String, String> {
        let value = entry.get("remoteMessageId")
            .ok_or_else(|| "Human reaction target has no canonical remote message identity".to_string())?;
        let remote_id = fabushi_message_identity_text(value)
            .map_err(|_| "Human reaction target has an invalid canonical remote message identity".to_string())?;
        let canonical_id = format!("human-server-message:{remote_id}");
        if entry.get("id").and_then(serde_json::Value::as_str) != Some(canonical_id.as_str())
            || entry.get("delivery").and_then(serde_json::Value::as_str) != Some("sent")
        {
            return Err("Human reaction target is not a settled canonical message".into());
        }
        Ok(remote_id)
    }

    fn validate_human_reaction_target(
        entry: &serde_json::Value,
        local_human_id: &str,
        peer_human_id: &str,
    ) -> Result<(), String> {
        let author = entry.get("authorId").and_then(serde_json::Value::as_str);
        if entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
            || entry.get("authorKind").and_then(serde_json::Value::as_str) != Some("human")
            || !matches!(author, Some(value) if value == local_human_id || value == peer_human_id)
        {
            return Err("persisted Human reaction target is outside the selected Human conversation".into());
        }
        Ok(())
    }

    fn replay_human_reaction_intent(
        &self,
        owner: &SandAgentDb,
        metadata: &serde_json::Value,
        client: &FabushiNativeMessagingClient,
    ) -> Result<Option<serde_json::Value>, String> {
        self.replay_human_reaction_intent_with_recovery(owner, metadata, client, None)
    }

    fn replay_human_reaction_intent_with_recovery(
        &self,
        owner: &SandAgentDb,
        metadata: &serde_json::Value,
        client: &FabushiNativeMessagingClient,
        mut recovery: Option<&mut HumanSyncRecoveryContext>,
    ) -> Result<Option<serde_json::Value>, String> {
        let Some(intent) = owner.get_metadata("humanReactionIntent")
            .map_err(|error| error.to_string())?.filter(|value| !value.is_null()) else {
            return Ok(None);
        };
        let get = |field: &str| intent.get(field).and_then(serde_json::Value::as_str)
            .ok_or_else(|| format!("persisted Human reaction intent lacks {field}"));
        let entry_id = get("entryId")?;
        let remote_id = get("remoteMessageId")?;
        let emoji = get("emoji")?;
        let author = get("localHumanId")?;
        let active = intent.get("active").and_then(serde_json::Value::as_bool)
            .ok_or_else(|| "persisted Human reaction intent lacks desired state".to_string())?;
        if author != self.local_human_id()? || !self.metadata_has_local_human(metadata)? {
            return Err("persisted Human reaction intent belongs to a different account".into());
        }
        if emoji.is_empty() || emoji != emoji.trim() || emoji.as_bytes().len() > 32 {
            return Err("persisted Human reaction intent contains invalid emoji".into());
        }
        if remote_id.parse::<u64>().ok().filter(|id| {
            *id > 0 && *id <= 9_007_199_254_740_991 && id.to_string() == remote_id
        }).is_none() {
            return Err("persisted Human reaction intent contains invalid remote identity".into());
        }
        let peer = self.peer_human_id_from_metadata(metadata)?;
        let current = owner.get_transcript_entry_by_id(entry_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "persisted Human reaction target disappeared".to_string())?;
        Self::validate_human_reaction_target(&current, author, &peer)?;
        if Self::human_reaction_remote_identity(&current)? != remote_id {
            return Err("persisted Human reaction remote identity changed".into());
        }
        let revalidate_owner = || -> Result<(), String> {
            let conversation_id = metadata.get("conversationId")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "existing Human conversation metadata is missing its identity".to_string())?;
            self.revalidate_human_recovery_owner(conversation_id, owner, metadata)
        };
        if let Some(recovery) = recovery.as_deref() {
            self.check_human_sync_binding(client, recovery)?;
            if recovery.is_continuing() { revalidate_owner()?; }
        }
        let response = client.set_direct_message_reaction_for_user_classified(
            remote_id, emoji, active, author,
        );
        if let Some(recovery) = recovery.as_deref() {
            self.check_human_sync_binding(client, recovery)?;
            if recovery.is_continuing() { revalidate_owner()?; }
        }
        let remote = match response {
            Ok(remote) => remote,
            Err(FabushiHumanReactionError::RemoteRefusal { status, message }) => {
                let Some(recovery) = recovery.as_deref_mut() else { return Err(message); };
                if !matches!(status, 404 | 409) { return Err(message); }
                // A transport refusal is only a candidate. The request cannot
                // lend its old admission to an independently changed SQLite row.
                revalidate_owner()?;
                if owner.get_metadata("humanReactionIntent").map_err(|error| error.to_string())?.as_ref()
                    != Some(&intent)
                {
                    return Err("persisted Human reaction intent changed before recovery continuation".into());
                }
                let current_target = owner.get_transcript_entry_by_id(entry_id)
                    .map_err(|error| error.to_string())?
                    .ok_or_else(|| "persisted Human reaction target disappeared".to_string())?;
                Self::validate_human_reaction_target(&current_target, author, &peer)?;
                if Self::human_reaction_remote_identity(&current_target)? != remote_id {
                    return Err("persisted Human reaction remote identity changed".into());
                }
                if current_target.get("authorId") != current.get("authorId") {
                    return Err("persisted Human reaction target author changed".into());
                }
                self.check_human_sync_binding(client, recovery)?;
                recovery.first_reaction_refusal.get_or_insert(message);
                return Ok(None);
            }
            Err(error) => return Err(error.into_message()),
        };
        Self::validate_remote_human_reactions(&remote)?;
        // HTTP success is not settlement: the service must confirm the requested
        // participant state. Keep the durable intent if a stale/misrouted reply
        // reports the opposite state so restart can retry without retoggling.
        let observed_active = remote.iter().any(|reaction| {
            reaction.emoji == emoji && reaction.reacted_by_me
        });
        if observed_active != active {
            return Err("Fabushi Human reaction response did not confirm the desired state".into());
        }
        let projected = Self::project_remote_human_reactions(&remote, &peer);
        let mut next = owner.get_transcript_entry_by_id(entry_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Human reaction target disappeared before durable settlement".to_string())?;
        Self::validate_human_reaction_target(&next, author, &peer)?;
        if Self::human_reaction_remote_identity(&next)? != remote_id {
            return Err("persisted Human reaction remote identity changed".into());
        }
        let is_peer = next.get("authorId").and_then(serde_json::Value::as_str) != Some(author);
        let object = next.as_object_mut()
            .ok_or_else(|| "Human reaction target is not a message object".to_string())?;
        if is_peer {
            if let Some(legacy_nonce) = object.remove("clientNonce") {
                object.entry("remoteClientRequestId".to_string()).or_insert(legacy_nonce);
            }
        }
        object.insert("remoteReactions".into(), serde_json::to_value(&remote)
            .map_err(|error| format!("remote Human reactions could not be projected: {error}"))?);
        object.insert("reactions".into(), serde_json::Value::Array(projected));
        let settled = owner.update_transcript_entry(entry_id, &next)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Human reaction target disappeared before durable settlement".to_string())?;
        if let Some(recovery) = recovery.as_deref().filter(|value| value.is_continuing()) {
            self.check_human_sync_binding(client, recovery)?;
            revalidate_owner()?;
        }
        if !owner.set_metadata("humanReactionIntent", serde_json::Value::Null)
            .map_err(|error| error.to_string())? {
            return Err("Human reaction intent could not be durably acknowledged".into());
        }
        Ok(Some(settled))
    }

    pub fn toggle_human_message_reaction(
        &self,
        conversation_id: &str,
        entry_id: &str,
        emoji: &str,
    ) -> Result<serde_json::Value, String> {
        if emoji.trim().is_empty() { return Err("reactHumanMessage requires emoji".into()); }
        if emoji != emoji.trim() {
            return Err("Fabushi Human reaction emoji must not contain surrounding whitespace".into());
        }
        if emoji.as_bytes().len() > 32 {
            return Err("Fabushi Human reaction emoji exceeds the backend limit".into());
        }
        let serial = self.human_message_lock(conversation_id)?;
        let _exclusive = serial.lock().map_err(|_| "Human conversation serial lock poisoned".to_string())?;
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        let client = self.shipping_native_messaging()?
            .ok_or_else(|| "Fabushi Human reactions require the shipping messaging backend".to_string())?;
        let prior = owner.get_metadata("humanReactionIntent").map_err(|error| error.to_string())?;
        let same_intent = prior.as_ref().is_some_and(|intent| {
            intent.get("entryId").and_then(serde_json::Value::as_str) == Some(entry_id)
                && intent.get("emoji").and_then(serde_json::Value::as_str) == Some(emoji)
        });
        if let Some(replayed) = self.replay_human_reaction_intent(&owner, &metadata, client)? {
            if same_intent { return Ok(replayed); }
        }
        let current = owner.get_transcript_entry_by_id(entry_id)
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "Human reaction target is not in this conversation".to_string())?;
        Self::validate_human_reaction_target(&current, self.local_human_id()?, &self.peer_human_id_from_metadata(&metadata)?)?;
        let remote_id = Self::human_reaction_remote_identity(&current)?;
        let active = current.get("remoteReactions")
            .and_then(serde_json::Value::as_array).is_some_and(|reactions| {
                reactions.iter().any(|reaction| {
                    reaction.get("emoji").and_then(serde_json::Value::as_str) == Some(emoji)
                        && reaction.get("reactedByMe").and_then(serde_json::Value::as_bool) == Some(true)
                })
            });
        let author = self.local_human_id()?;
        if client.identity()?.user_id != author {
            return Err("Fabushi native messaging credential changed Human account".into());
        }
        if !owner.set_metadata("humanReactionIntent", serde_json::json!({
            "entryId": entry_id, "remoteMessageId": remote_id,
            "emoji": emoji, "active": !active, "localHumanId": author,
        })).map_err(|error| error.to_string())? {
            return Err("Human reaction intent could not be durably staged".into());
        }
        self.replay_human_reaction_intent(&owner, &metadata, client)?
            .ok_or_else(|| "Human reaction intent disappeared before settlement".to_string())
    }

    pub(crate) fn resolve_human_forward_origin(
        source_entry: &serde_json::Value,
        source_metadata: Option<&serde_json::Value>,
        local_human_id: &str,
        privacy: fabushi_messaging_core::ForwardPrivacy,
    ) -> Option<serde_json::Value> {
        let source_context = source_entry.get("forwardContext");
        let source_forward_privacy =
            source_context.map(|ctx| fabushi_messaging_core::ForwardPrivacy {
                drop_sender_names: ctx
                    .pointer("/privacy/dropSenderNames")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
                drop_captions: ctx
                    .pointer("/privacy/dropCaptions")
                    .and_then(serde_json::Value::as_bool)
                    .unwrap_or(false),
            });
        let source_origin_author_id = source_context
            .and_then(|ctx| ctx.pointer("/origin/authorId"))
            .and_then(serde_json::Value::as_str);
        let source_origin_author_name = source_context
            .and_then(|ctx| ctx.pointer("/origin/authorName"))
            .and_then(serde_json::Value::as_str);
        let source_author_id = source_entry
            .get("authorId")
            .and_then(serde_json::Value::as_str);
        let source_author_name = source_entry
            .get("authorName")
            .and_then(serde_json::Value::as_str);
        let peer_title_fallback = if source_author_id.is_some_and(|id| id.trim() != local_human_id)
        {
            source_metadata
                .and_then(|metadata| metadata.get("title"))
                .and_then(serde_json::Value::as_str)
        } else {
            None
        };
        let attribution = privacy.resolve_forward_origin_attribution(
            source_forward_privacy,
            source_origin_author_id,
            source_origin_author_name,
            source_author_id,
            source_author_name,
            peer_title_fallback,
        )?;
        let mut origin = serde_json::Map::new();
        if let Some(author_id) = attribution.author_id {
            origin.insert(
                "authorId".into(),
                serde_json::Value::String(author_id.to_string()),
            );
        }
        if let Some(author_name) = attribution.author_name {
            origin.insert(
                "authorName".into(),
                serde_json::Value::String(author_name.to_string()),
            );
        }
        (!origin.is_empty()).then_some(serde_json::Value::Object(origin))
    }

    pub(crate) fn human_entry_visible_forward_origin_author(
        entry: &serde_json::Value,
    ) -> Option<&str> {
        let context = entry.get("forwardContext")?;
        let privacy = fabushi_messaging_core::ForwardPrivacy {
            drop_sender_names: context
                .pointer("/privacy/dropSenderNames")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            drop_captions: context
                .pointer("/privacy/dropCaptions")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        };
        let origin_author_name = context
            .pointer("/origin/authorName")
            .and_then(serde_json::Value::as_str);
        fabushi_messaging_core::visible_forward_origin_author(privacy, origin_author_name)
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
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        owner
            .get_human_transcript_entries()
            .map_err(|error| error.to_string())
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
mod human_send_settlement_credential_tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, Instant};

    fn write_credential(path: &Path, issuer: &str, token: &str) {
        fs::write(path, serde_json::json!({
            "accessToken": token, "deviceId": "device-a", "userId": 1,
            "sessionId": "fixture-session", "issuer": issuer,
        }).to_string()).expect("write private settlement credential fixture");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))
                .expect("private settlement fixture permissions");
        }
    }

    fn remote_message() -> serde_json::Value {
        serde_json::json!({
            "id": 96, "senderUserId": 1, "recipientUserId": 2,
            "text": "post-lock lease", "clientRequestId": "post-lock-lease",
            "createdAt": "2026-10-05T01:00:36Z", "isOutgoing": true,
            "attachments": [], "reactions": [],
        })
    }

    fn successful_response() -> Result<FabushiRemoteHumanMessage, FabushiHumanSendError> {
        Ok(serde_json::from_value(remote_message()).expect("valid persisted message fixture"))
    }

    fn completed_http_fixture(disconnect: bool) -> (String, thread::JoinHandle<()>) {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind settlement HTTP fixture");
        listener.set_nonblocking(true).expect("bounded settlement fixture accept");
        let address = listener.local_addr().expect("settlement fixture address");
        let server = thread::spawn(move || {
            let deadline = Instant::now() + Duration::from_secs(15);
            let mut stream = loop {
                match listener.accept() {
                    Ok((stream, _)) => break stream,
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        assert!(Instant::now() < deadline, "settlement fixture accept timed out");
                        thread::yield_now();
                    }
                    Err(error) => panic!("settlement fixture accept failed: {error}"),
                }
            };
            stream.set_nonblocking(false).expect("blocking settlement fixture stream");
            stream.set_read_timeout(Some(Duration::from_secs(15)))
                .expect("bounded settlement request read");
            stream.set_write_timeout(Some(Duration::from_secs(15)))
                .expect("bounded settlement response write");
            let mut request = Vec::new();
            let mut buffer = [0_u8; 1024];
            let header_end = loop {
                let count = stream.read(&mut buffer).expect("read settlement request headers");
                assert!(count > 0, "settlement request closed before headers");
                request.extend_from_slice(&buffer[..count]);
                assert!(request.len() <= 64 * 1024, "settlement request exceeded fixture budget");
                if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                    break index + 4;
                }
            };
            let headers = std::str::from_utf8(&request[..header_end]).expect("HTTP request headers");
            assert!(headers.starts_with("POST /api/social/messages "));
            let content_length = headers.lines().find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().expect("request body length"))
            }).expect("bounded request body declaration");
            assert!(content_length <= 64 * 1024, "settlement request body exceeded fixture budget");
            while request.len() < header_end + content_length {
                let count = stream.read(&mut buffer).expect("read settlement request body");
                assert!(count > 0, "settlement request closed before its body");
                request.extend_from_slice(&buffer[..count]);
            }
            let body: serde_json::Value = serde_json::from_slice(
                &request[header_end..header_end + content_length],
            ).expect("settlement request JSON");
            assert_eq!(body["clientRequestId"], "post-lock-lease");
            if disconnect { return; }
            let body = serde_json::json!({"success": true, "message": remote_message()}).to_string();
            write!(stream,
                "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(), body,
            ).expect("write completed settlement response");
            stream.flush().expect("flush completed settlement response");
        });
        (format!("http://{address}"), server)
    }

    #[test]
    fn shipping_human_settlement_credential_rotation_after_completed_http_is_rejected() {
        for disconnect in [false, true] {
            let root = std::env::temp_dir().join(format!("fabushi-settlement-{}", Uuid::new_v4()));
            fs::create_dir_all(&root).expect("settlement fixture directory");
            let path = root.join("credential.json");
            let (issuer, server) = completed_http_fixture(disconnect);
            write_credential(&path, &issuer, "original-fixture-token");
            let client = FabushiNativeMessagingClient::new(&issuer, &path)
                .expect("real Native credential and HTTP owner");
            let fingerprint = client.request_credential_fingerprint().expect("dispatch fingerprint");
            let serial = Arc::new(Mutex::new(()));
            let competing_guard = serial.lock().expect("hold competing Session mutation");
            let (http_done, http_wait) = mpsc::channel();
            let (settled, settlement_wait) = mpsc::channel();
            let worker_serial = Arc::clone(&serial);
            let worker = thread::spawn(move || {
                let response = client.send_direct_message_for_settlement(
                    "2", "post-lock lease", "post-lock-lease", None, &[], "1",
                );
                if disconnect {
                    assert!(matches!(&response, Err(FabushiHumanSendError::Transport(_))),
                        "complete disconnect must be an ambiguous transport outcome");
                } else {
                    assert!(response.is_ok(), "complete HTTP body must be accepted before rotation");
                }
                // This signal follows the real Native send method, including
                // body consumption and its final credential stability check.
                http_done.send(()).expect("observe completed Native HTTP result");
                let result = ProductionSessionWorkers::acquire_human_send_settlement_guard(
                    &client, worker_serial.as_ref(), None, &fingerprint, &response,
                ).map(|_guard| ());
                settled.send(result).expect("observe post-lock settlement result");
            });
            http_wait.recv_timeout(Duration::from_secs(15))
                .expect("Native HTTP must complete while competing mutation holds the lock");
            // The lock is still held here. No scheduler delay or socket-write
            // observation is used to infer that the response was consumed.
            write_credential(&path, &issuer, "rotated-after-completed-http");
            drop(competing_guard);
            let result = settlement_wait.recv_timeout(Duration::from_secs(15))
                .expect("post-lock settlement must terminate");
            assert_eq!(result, Err(
                "Fabushi native messaging credential changed before Human settlement".to_string(),
            ));
            worker.join().expect("settlement worker");
            server.join().expect("settlement HTTP fixture");
            fs::remove_dir_all(root).expect("remove settlement fixture");
        }
    }

    #[test]
    fn shipping_human_settlement_credential_unchanged_retains_the_existing_lock() {
        let root = std::env::temp_dir().join(format!("fabushi-settlement-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("settlement fixture directory");
        let path = root.join("credential.json");
        let issuer = "http://127.0.0.1:4200";
        write_credential(&path, issuer, "unchanged-fixture-token");
        let client = FabushiNativeMessagingClient::new(issuer, &path).expect("Native credential owner");
        let fingerprint = client.request_credential_fingerprint().expect("dispatch fingerprint");
        let serial = Mutex::new(());
        for response in [successful_response(), Err(FabushiHumanSendError::Transport("disconnect".into()))] {
            let guard = ProductionSessionWorkers::acquire_human_send_settlement_guard(
                &client, &serial, None, &fingerprint, &response,
            ).expect("unchanged completed result can enter settlement");
            assert!(guard.is_some(), "ordinary send must acquire the mutation guard");
            assert!(matches!(serial.try_lock(), Err(std::sync::TryLockError::WouldBlock)),
                "returned guard must protect the caller's later durable settlement");
            drop(guard);
            let held = serial.lock().expect("sync already owns the mutation lane");
            let borrowed = ProductionSessionWorkers::acquire_human_send_settlement_guard(
                &client, &serial, Some(&held), &fingerprint, &response,
            ).expect("sync must reuse its existing guard without recursive acquisition");
            assert!(borrowed.is_none());
            assert!(matches!(serial.try_lock(), Err(std::sync::TryLockError::WouldBlock)));
            drop(held);
        }
        fs::remove_dir_all(root).expect("remove settlement fixture");
    }

    #[test]
    fn shipping_human_settlement_credential_rejection_preserves_the_original_refusal() {
        let root = std::env::temp_dir().join(format!("fabushi-settlement-{}", Uuid::new_v4()));
        fs::create_dir_all(&root).expect("settlement fixture directory");
        let path = root.join("credential.json");
        let issuer = "http://127.0.0.1:4200";
        let client = FabushiNativeMessagingClient::new(issuer, &path).expect("Native credential owner");
        let serial = Mutex::new(());
        for refusal in [
            "Fabushi native messaging credential changed during Human request",
            "Fabushi Human message send was rejected (HTTP 403): forbidden",
        ] {
            write_credential(&path, issuer, "original-fixture-token");
            let fingerprint = client.request_credential_fingerprint().expect("dispatch fingerprint");
            write_credential(&path, issuer, "later-fixture-token");
            let response = Err(FabushiHumanSendError::Rejected(refusal.into()));
            let error = ProductionSessionWorkers::acquire_human_send_settlement_guard(
                &client, &serial, None, &fingerprint, &response,
            ).expect_err("explicit rejection never enters successful settlement or transport recovery");
            assert_eq!(error, refusal);
            fs::remove_file(&path).expect("invalidate the later credential snapshot");
            let held = serial.lock().expect("sync mutation guard");
            let error = ProductionSessionWorkers::acquire_human_send_settlement_guard(
                &client, &serial, Some(&held), &fingerprint, &response,
            ).expect_err("later credential unavailability cannot replace the original rejection");
            assert_eq!(error, refusal);
        }
        fs::remove_dir_all(root).expect("remove settlement fixture");
    }
}

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
        assert!(status_for(&gamma).get("error").and_then(serde_json::Value::as_str).unwrap_or_default().contains("rejected the source message policy"));
        assert_eq!(status_for(&delta).get("status").and_then(serde_json::Value::as_str), Some("failed"));
        assert!(status_for(&delta).get("error").and_then(serde_json::Value::as_str).unwrap_or_default().contains("rejected the source message policy"));
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
                    "can_send_messages": true,
                    "can_send_media": true,
                    "can_send_polls": true
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


