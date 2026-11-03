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
use crate::attachment_paths::{OutgoingMediaCustody, get_agent_media_store_roots};
use crate::agents::agent_profile::SandAgentProfile;
use crate::storage::agent_paths::get_sand_agents_root_dir;
use crate::agents::settings_file::{get_sand_settings_path, write_sand_settings_file};

use super::agent_db::{
    AgentDbProjectionError, AgentDbSerdeSnapshot, CompactionCommit, CompactionCommitOutcome, CronSessionRunRecord,
    ChannelInboundAppendAttempt, ChannelInboundAppendFailure, ChannelInboundAppendFailureKind,
    ChannelInboundAppendOutcome,
    PendingMemoryMutation,
    PendingProfileMutation, ProfileFileSnapshot, SandAgentDb,
    ProviderReviewContinuationOutcome, SessionProviderReview,
    SessionWorkAdmissionRecord,
    SessionWorkResultCommitOutcome,
    OriginalTurnReference, OriginalTurnReceiptCommit, OriginalTurnTerminalRecord,
    OriginalTurnTerminalFence, SameTurnCompactionBinding,
};
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
};
use super::agent_session::{
    read_session_model_entry, write_session_model_entry,
};
use crate::extensions::inference::model_picker_service::{
    apply_session_model_selection as apply_model_selection,
    ApplySessionModelSelectionParams, ApplySessionModelSelectionResult,
};

fn hide_session_work_receipts(entries: &mut Vec<serde_json::Value>) {
    entries.retain(|entry| {
        !matches!(
            entry.get("kind").and_then(serde_json::Value::as_str),
            Some("background-result-receipt" | "outgoing-media-settlement")
        )
    });
}

/// Preserve the existing Session view for semantic planning while the fresh
/// append owner separately retains every physical ID for allocation.
pub(crate) fn project_agent_transcript_entries(
    snapshot: &[serde_json::Value],
) -> Vec<serde_json::Value> {
    let mut entries = snapshot.iter()
        .filter(|entry| super::agent_db_serde::is_valid_owner_transcript_entry(entry))
        .cloned()
        .collect::<Vec<_>>();
    hide_session_work_receipts(&mut entries);
    entries
}
use super::call_session::{CallSession, CallSessionStore, CallSignal, transition_target};
use super::agent_db_serde::{
    AwaitingUserResponse, EpisodeTurn, MemoryPromptSnapshot, SandProfile, SpendGuardState,
    UnreadState,
};
use super::session_conversation_state::{
    ConversationOutlineItem, PendingConversationTurnRegistry, ResolvedConversationState,
    SessionConversationState, TranscriptThread,
};
use super::conversation_blobs_path::conversation_blobs_path;
use super::conversation_size_limits::{
    ConversationGcTarget, ConversationSizeMaintenance, ConversationSizePolicy,
};
use super::session_paths::{
    assert_valid_agent_session_id, get_agent_db_path, get_connector_secrets_root,
    get_native_conversation_db_path, get_native_conversations_root,
};
use super::connector_secret_store::SandConnectorSecretStore;
use super::channel_store::{ChannelConfig, ChannelConnection, FileChannelStore};
use super::session_store_factories::{
    automation_store_for_db_path_with_time_zone_resolver, channel_store_for_db_path,
    workflow_store_for_db_path_with_time_zone_resolver,
};
use crate::extensions::memory::memory_service::{
    FileMemoryStore, MemoryKind, MemoryRecoveryRecord, MemoryService, SynthesisApplyResult,
};
use crate::extensions::memory::agent_state::{MemoryTier, StateWriteResult, MEMORY_NOTE_PREFIX};
use crate::extensions::memory::project_membership::AgentProjectMembership;
use crate::automations::automation::AutomationRecord;
use crate::automations::automation_store::{FileAutomationStore, UserTimeZoneResolver, agent_has_automations};
use crate::workflows::workflow_store::{FileWorkflowStore, agent_has_workflows};
use crate::extensions::mcp::plugin_skills::{
    PluginSkillsAdmissionQuery, PluginSkillsAdmissionReader,
};
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
use super::session_mutations::{
    capture_profile_mutation_files, profile_mutation_files_match,
    restore_profile_mutation_files,
};
use super::typed_replay::{TypedReplayEntry, project_typed_replay_entries};
use super::production_agent_store::{
    ProductionAgentStore, ProductionAgentCheckpointSnapshot, ProductionWorkerBlobStore,
};
use super::native_messaging::{
    FabushiNativeMessagingClient, FabushiRemoteHumanAttachment, FabushiRemoteHumanCall,
    FabushiRemoteHumanCallEvent, FabushiRemoteHumanMessage, fabushi_identity_text,
};

pub const PRODUCTION_BLOB_BUSY_TIMEOUT_MS: u64 = 5_000;

pub type ProductionAgentWorkerPool = AgentWorkerPool<ProductionAgentStoreWorkerBackend>;

/// Checked owner data for Runner's blob qualification. The raw SQL fence is
/// deliberately private and is rechecked by Session after those blob reads.
pub(crate) struct OriginalTurnTerminalView {
    pub(crate) original_user_content: String,
    pub(crate) checkpoint: ProductionAgentCheckpointSnapshot,
    pub(crate) store: Arc<ProductionAgentStore>,
    pub(crate) settlement_lifecycle_revision: String,
    pub(crate) compaction: Option<SameTurnCompactionBinding>,
    original: OriginalTurnReference,
    fence: OriginalTurnTerminalFence,
}

pub(crate) struct OriginalTurnTerminalRead {
    pub(crate) record: OriginalTurnTerminalRecord,
    pub(crate) view: OriginalTurnTerminalView,
}

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

/// Safe folder names can still collide across the Agent and Human domains.
#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
enum SessionDbOwnerKey {
    Agent(String),
    Human(String),
}

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
    pending_conversation_turns: PendingConversationTurnRegistry,
    memory_service: Arc<MemoryService>,
    mint_queue: SessionMintQueue,
    db_owners: Mutex<BTreeMap<SessionDbOwnerKey, Arc<SandAgentDb>>>,
    agent_store_owners: Mutex<BTreeMap<String, Arc<ProductionAgentStore>>>,
    filesystem_mutation_locks: Mutex<BTreeMap<String, Arc<Mutex<()>>>>,
    deleting_agents: Mutex<BTreeSet<String>>,
    roster_extras_cache: RosterExtrasCache,
    user_time_zone_resolver: UserTimeZoneResolver,
    local_human_id: Option<String>,
    native_messaging: Option<Arc<FabushiNativeMessagingClient>>,
    native_messaging_required: bool,
    native_messaging_error: Option<String>,
    call_session_store: Option<Arc<CallSessionStore>>,
    call_session_store_error: Option<String>,
    provider_review_capabilities: Mutex<BTreeMap<String, ProviderReviewCapabilityState>>,
    plugin_skills_admission: Arc<SessionPluginSkillsAdmission>,
    busy_timeout_ms: u64,
}

/// Every Session-opened workflow store keeps this same forwarding handle.
/// Only the Plugin Skills owner holds refusal facts; an unbound Session keeps
/// the existing committed-snapshot behavior.
#[derive(Default)]
struct SessionPluginSkillsAdmission {
    reader: Mutex<Option<PluginSkillsAdmissionReader>>,
}

impl PluginSkillsAdmissionQuery for SessionPluginSkillsAdmission {
    fn is_denied(&self, plugin_id: &str) -> bool {
        let reader = match self.reader.lock() {
            Ok(slot) => slot.clone(),
            Err(_) => return true,
        };
        // Do not enter the owner's query (or any delegate) under our slot lock.
        reader.is_some_and(|reader| reader.is_denied(plugin_id))
    }
}

#[derive(Debug)]
struct ProviderReviewCapabilityState {
    agent_id: String,
    lifecycle_revision: String,
    review_id: String,
    next_run_id: String,
}

/// Host-issued, single-claim provider-review capability. It is deliberately
/// not serde-serializable; request JSON cannot manufacture one.
#[derive(Debug)]
pub struct ProviderReviewAcknowledgment {
    token: String,
}

impl ProductionSessionWorkers {
    pub fn bind_memory_synthesis_gate(self: &Arc<Self>) {
        let weak = Arc::downgrade(self);
        self.memory_service.set_synthesis_apply_gate(Some(Arc::new(
            move |agent_id, store, snapshot, changes, now_ms| {
                let Some(owner) = weak.upgrade() else {
                    return SynthesisApplyResult::Invalid;
                };
                owner
                    .with_agent_filesystem_mutation(agent_id, || {
                        let canonical_location = owner
                            .memory_service()
                            .store_for_agent(agent_id)
                            .get_location();
                        if store.get_location() != canonical_location {
                            return Ok(SynthesisApplyResult::Invalid);
                        }
                        owner.assert_agent_work_admission_locked(agent_id)?;
                        Ok(
                            store
                                .apply_synthesis(snapshot, changes, now_ms)
                                .unwrap_or(SynthesisApplyResult::Failed),
                        )
                    })
                    .unwrap_or(SynthesisApplyResult::Invalid)
            },
        )));
    }

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
            pending_conversation_turns: PendingConversationTurnRegistry::new(),
            mint_queue: SessionMintQueue::default(),
            db_owners: Mutex::new(BTreeMap::new()),
            agent_store_owners: Mutex::new(BTreeMap::new()),
            filesystem_mutation_locks: Mutex::new(BTreeMap::new()),
            deleting_agents: Mutex::new(BTreeSet::new()),
            roster_extras_cache: RosterExtrasCache::default(),
            user_time_zone_resolver,
            local_human_id,
            native_messaging,
            native_messaging_required,
            native_messaging_error,
            call_session_store,
            call_session_store_error,
            provider_review_capabilities: Mutex::new(BTreeMap::new()),
            plugin_skills_admission: Arc::new(SessionPluginSkillsAdmission::default()),
            busy_timeout_ms,
        }
    }

    pub fn pending_conversation_turns(&self) -> PendingConversationTurnRegistry {
        self.pending_conversation_turns.clone()
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
        const PAGE_LIMIT: usize = 200;
        const MAX_PAGES: usize = 32;

        let local = self.authorized_call_session(call_id)?;
        let Some(client) = self.shipping_native_messaging()? else {
            return Ok(local);
        };
        let identity = client.identity()?;
        if identity.user_id != self.local_human_id()? {
            return Err("Fabushi call credential identity does not match active Human identity".into());
        }

        let mut cursor = self.call_sessions()?.remote_event_seq(call_id)?;
        for _ in 0..MAX_PAGES {
            let (remote, events, next_after_seq) =
                client.get_human_call(call_id, cursor, PAGE_LIMIT)?;
            self.validate_remote_call(&remote, &self.authorized_call_session(call_id)?)?;
            let prior_cursor = cursor;
            for event in events {
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
                self.apply_remote_call_event(&event)?;
                self.call_sessions()?.set_remote_event_seq(call_id, event.seq)?;
                cursor = event.seq;
            }
            if cursor >= remote.event_seq {
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
        let Some(client) = self.shipping_native_messaging()? else {
            return Ok(Vec::new());
        };
        let local_human_id = self.local_human_id()?.to_string();
        let identity = client.identity()?;
        if identity.user_id != local_human_id {
            return Err("Fabushi call credential identity does not match active Human identity".into());
        }
        let remote_calls = client.list_human_calls(200)?;
        let mut synced = Vec::with_capacity(remote_calls.len());
        for remote in remote_calls {
            let peer = self.remote_call_peer(&remote)?;
            let conversation = self.create_human_conversation(&peer, &peer)?;
            let scope_id = conversation
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "Human call discovery could not resolve canonical conversation scope".to_string())?;
            if self.call_sessions()?.get(&remote.call_id)?.is_none() {
                let creator = fabushi_identity_text(&remote.creator_user_id)?;
                let participants = vec![local_human_id.clone(), peer.clone()];
                self.call_sessions()?.create_with_id(
                    &remote.call_id,
                    scope_id,
                    &creator,
                    &participants,
                )?;
            }
            let call = self.authorized_call_session(&remote.call_id)?;
            if call.scope_id != scope_id {
                return Err("remote Human call resolved to a different canonical conversation scope".into());
            }
            self.validate_remote_call(&remote, &call)?;
            synced.push(self.sync_call_session_from_remote(&remote.call_id)?);
        }
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
            let result = self.with_agent_filesystem_mutation(&agent_id, || {
                self.retire_agent_owners_locked(&agent_id, false)?;
                match fs::remove_dir_all(self.agents_root.join(&agent_id)) {
                    Ok(()) => Ok(true),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(false),
                    Err(error) => Err(format!("io::{:?}", error.kind())),
                }
            });
            match result {
                Ok(true) => reclaimed.push(agent_id),
                Ok(false) => {}
                Err(error) => report_materialization_failure(
                    "placeholder_reclaim_failed",
                    &agent_id,
                    &error,
                ),
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
        self.ensure_main_session_work_admission(&record.id)?;
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
            self.ensure_main_session_work_admission(&record.id)?;
        }
        Ok(result)
    }

    pub fn compose_materialized_session(
        &self,
        record: MaterializedAgentRecord,
        prepared: PreparedAgentBlobStore,
    ) -> Result<ProductionMaterializedSession, String> {
        self.ensure_main_session_work_admission(&record.id)?;
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
        let mut entries = self
            .open_agent_db_owner(agent_id)?
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        hide_session_work_receipts(&mut entries);
        Ok(entries)
    }

    /// Preserve the public transcript projection inside an already-held
    /// filesystem/admission boundary without recursively acquiring its mutex.
    pub(crate) fn read_agent_transcript_entries_locked(
        &self,
        agent_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        let mut entries = self
            .open_agent_db_owner_locked(agent_id)?
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        hide_session_work_receipts(&mut entries);
        Ok(entries)
    }

    pub fn read_agent_transcript_page(
        &self,
        agent_id: &str,
        query: TranscriptPageQuery,
    ) -> Result<TranscriptPage, String> {
        let mut page = self
            .open_agent_db_owner(agent_id)?
            .get_transcript_page(query)
            .map_err(|error| error.to_string())?;
        hide_session_work_receipts(&mut page.entries);
        Ok(page)
    }

    pub fn read_agent_transcript_window(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptWindow<std::collections::BTreeMap<String, usize>>, String> {
        let mut window = self
            .open_agent_db_owner(agent_id)?
            .get_transcript_window(query)
            .map_err(|error| error.to_string())?;
        hide_session_work_receipts(&mut window.entries);
        Ok(window)
    }

    pub fn read_agent_transcript_tail(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, String> {
        let mut page = self
            .open_agent_db_owner(agent_id)?
            .get_transcript_tail(query)
            .map_err(|error| error.to_string())?;
        hide_session_work_receipts(&mut page.entries);
        Ok(page)
    }

    pub fn read_agent_thread(
        &self,
        agent_id: &str,
        root_id: &str,
    ) -> Result<TranscriptThread, String> {
        let mut entries = self
            .open_agent_db_owner(agent_id)?
            .get_thread_entries(root_id)
            .map_err(|error| error.to_string())?;
        hide_session_work_receipts(&mut entries);
        Ok(TranscriptThread { entries })
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

    /// Resolve one backend target under the same Session boundary as its
    /// writers and deletion owner. Only checked directory absence can bypass
    /// durable admission; a present agent with unavailable state is an error.
    pub fn find_automation_for_backend_delivery(
        &self,
        agent_id: &str,
        backend_id: &str,
    ) -> Result<Option<AutomationRecord>, String> {
        let db_path = self.session_db_path(agent_id)?;
        self.with_agent_filesystem_mutation(agent_id, || {
            if self.deleting_agents.lock()
                .map_err(|_| "agent deletion state is poisoned".to_string())?
                .contains(agent_id)
            {
                return Err(format!(
                    "agent {agent_id} deletion cleanup is pending; backend delivery is not admitted"
                ));
            }
            let agent_dir = db_path.parent()
                .ok_or_else(|| "agent database has no parent directory".to_string())?;
            if !Self::checked_backend_agent_directory_exists(agent_dir)? {
                return Ok(None);
            }
            self.assert_agent_work_admission_locked(agent_id)?;
            let store = automation_store_for_db_path_with_time_zone_resolver(
                &db_path,
                Arc::clone(&self.user_time_zone_resolver),
            );
            store.find_for_backend_delivery(agent_id, backend_id)
                .map_err(|error| error.to_string())
        })
    }

    fn checked_backend_agent_directory_exists(agent_dir: &Path) -> Result<bool, String> {
        let mut missing = false;
        // A NotFound below a dangling ancestor does not establish absence.
        // Walk back only after NotFound until a real directory is checked;
        // existing root/agent links retain their normal followed semantics.
        for path in agent_dir.ancestors()
            .filter(|path| !path.as_os_str().is_empty())
            .chain(std::iter::once(Path::new(".")))
        {
            let metadata = match fs::symlink_metadata(path) {
                Ok(metadata) => metadata,
                Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                    missing = true;
                    continue;
                }
                Err(error) => return Err(format!(
                    "backend delivery agent directory {} is unavailable: {error}", path.display()
                )),
            };
            let metadata = if metadata.file_type().is_symlink() {
                fs::metadata(path).map_err(|error| format!(
                    "backend delivery agent directory {} is unavailable: {error}", path.display()
                ))?
            } else {
                metadata
            };
            if !metadata.is_dir() {
                return Err(format!(
                    "backend delivery agent path {} is not a directory", path.display()
                ));
            }
            return Ok(!missing);
        }
        Err("backend delivery agent directory absence could not be established".into())
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
        let reader: PluginSkillsAdmissionReader = self.plugin_skills_admission.clone();
        Ok(workflow_store_for_db_path_with_time_zone_resolver(
            &db_path,
            Arc::clone(&self.user_time_zone_resolver),
        ).with_plugin_skills_admission(reader))
    }

    pub fn bind_plugin_skills_admission(
        &self,
        reader: PluginSkillsAdmissionReader,
    ) -> Result<(), String> {
        let mut slot = self.plugin_skills_admission.reader.lock()
            .map_err(|_| "Session plugin skills admission reader is poisoned".to_string())?;
        match slot.as_ref() {
            Some(current) if Arc::ptr_eq(current, &reader) => Ok(()),
            Some(_) => Err("Session plugin skills admission is already bound to another owner".into()),
            None => {
                *slot = Some(reader);
                Ok(())
            }
        }
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
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
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
        })
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
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            self.connector_secret_store()
                .remove_agent_platform(agent_id, platform)
                .map_err(|error| error.to_string())
        })
    }

    pub fn disconnect_channel(&self, agent_id: &str, platform: &str) -> Result<bool, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let _ = self
                .connector_secret_store()
                .remove_agent_platform(agent_id, platform)
                .map_err(|error| error.to_string())?;
            self.open_channel_store(agent_id)?
                .remove(platform)
                .map_err(|error| error.to_string())
        })
    }

    pub fn write_agent_settings(
        &self,
        agent_id: &str,
        update: serde_json::Map<String, serde_json::Value>,
    ) -> Result<(), String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let db_path = self.existing_session_db_path(agent_id)?;
            let agent_dir = db_path
                .parent()
                .ok_or_else(|| "agent database has no parent directory".to_string())?;
            write_sand_settings_file(get_sand_settings_path(agent_dir), &update)
                .map_err(|error| error.to_string())
        })
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
        assert_valid_agent_session_id(agent_id).map_err(|error| error.to_string())?;
        self.with_agent_filesystem_mutation(agent_id, || {
            if self.is_agent_being_deleted(agent_id) {
                return Err(format!(
                    "agent {agent_id} deletion cleanup is pending; owner open is not admitted"
                ));
            }
            self.open_agent_db_owner_locked(agent_id)
        })
    }

    /// Open or reuse the per-agent SQLite owner while the caller already
    /// holds the filesystem/deletion serialization boundary. All owner-open
    /// recovery must run under that boundary, but mutation producers cannot
    /// reacquire the non-reentrant filesystem mutex.
    fn open_agent_db_owner_locked(
        &self,
        agent_id: &str,
    ) -> Result<Arc<SandAgentDb>, String> {
        self.open_agent_db_owner_locked_with_recovery(agent_id, true)
    }

    fn open_agent_db_owner_locked_with_recovery(
        &self,
        agent_id: &str,
        recover_pending: bool,
    ) -> Result<Arc<SandAgentDb>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        // Re-entry must reject a stale cached root before pending recovery can
        // settle journals or restore files. Release the owner-map lock before
        // this validation performs blob/DB I/O.
        let cached_store = self.agent_store_owners.lock()
            .map_err(|_| "agent store owner map poisoned".to_string())?
            .get(agent_id).cloned();
        if let Some(cached_store) = cached_store {
            cached_store.validate_cached_checkpoint_checked()?;
        }
        let blob_store = self.create_agent_blob_store(agent_id)?;
        let mut owners = self
            .db_owners
            .lock()
            .map_err(|_| "agent db owner map poisoned".to_string())?;
        let key = SessionDbOwnerKey::Agent(agent_id.to_string());
        if let Some(owner) = owners.get(&key) {
            ProductionAgentStore::validate_persisted_checkpoint_checked(owner, &blob_store)?;
            self.validate_owner_session_identity(agent_id, owner)?;
            if recover_pending {
                self.recover_pending_profile_mutation(agent_id, owner)?;
                self.recover_pending_memory_mutation(agent_id, owner)?;
            }
            return Ok(Arc::clone(owner));
        }
        let owner = Arc::new(if recover_pending {
            SandAgentDb::open_existing_agent(&db_path, self.busy_timeout_ms)
        } else {
            SandAgentDb::open_existing_agent_without_recovery(&db_path, self.busy_timeout_ms)
        }.map_err(|error| error.to_string())?);
        ProductionAgentStore::validate_persisted_checkpoint_checked(&owner, &blob_store)?;
        self.validate_owner_session_identity(agent_id, &owner)?;
        if !recover_pending && owner.read_session_work_admission()
            .map_err(|error| error.to_string())?
            .is_some_and(|record| record.admitted_media_work_id.is_some()
                || !record.admitted_media_paths.is_empty())
        {
            // Ordinary media recovery runs only on a first open. Do not cache
            // a strict-open candidate that skipped that required work, or a
            // later ordinary open would mistake it for an already recovered
            // owner. Retain the fence for that existing recovery path.
            return Err("terminal inspection found pending media custody on first open".into());
        }
        if recover_pending {
            self.recover_pending_media_custody(agent_id, &owner)?;
            self.recover_pending_profile_mutation(agent_id, &owner)?;
            self.recover_pending_memory_mutation(agent_id, &owner)?;
        }
        owners.insert(key, Arc::clone(&owner));
        Ok(owner)
    }

    fn validate_owner_session_identity(
        &self,
        agent_id: &str,
        owner: &SandAgentDb,
    ) -> Result<(), String> {
        let Some(record) = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
        else {
            return Ok(());
        };
        let metadata_agent_id = owner
            .get_metadata("agentId")
            .map_err(|error| error.to_string())?
            .and_then(|value| value.as_str().map(str::trim).map(ToOwned::to_owned));
        let expected_session_key = format!("agent:{agent_id}:main");
        if record.session_id != agent_id
            || metadata_agent_id.as_deref() != Some(agent_id)
            || record.session_id != metadata_agent_id.as_deref().unwrap_or_default()
            || record.session_key != expected_session_key
            || record.lifecycle_revision.trim().is_empty()
        {
            return Err(format!(
                "agent {agent_id} session admission identity is malformed or rebound"
            ));
        }
        Ok(())
    }

    /// Reconcile one durable media fence when an agent database is first
    /// opened after a process interruption. A committed settlement receipt
    /// retains its media; without one, only paths proven to remain inside the
    /// agent media roots are rolled back. Any uncertainty leaves the fence in
    /// place so archive/delete cannot race an unknown outcome.
    fn recover_pending_media_custody(
        &self,
        agent_id: &str,
        owner: &SandAgentDb,
    ) -> Result<(), String> {
        let Some(record) = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
        else {
            return Ok(());
        };
        let Some(work_id) = record.admitted_media_work_id.as_deref() else {
            if record.admitted_media_paths.is_empty() {
                return Ok(());
            }
            return Err("media admission has paths without a work identity".into());
        };
        let expected_revision = record.lifecycle_revision.clone();
        let receipt_id = format!("outgoing-media-settlement:{work_id}");
        if let Some(receipt) = owner
            .get_transcript_entry_by_id(&receipt_id)
            .map_err(|error| error.to_string())?
        {
            let valid = receipt.get("kind").and_then(serde_json::Value::as_str)
                == Some("outgoing-media-settlement")
                && receipt.get("sessionKey").and_then(serde_json::Value::as_str)
                    == Some(record.session_key.as_str())
                && receipt.get("mediaWorkId").and_then(serde_json::Value::as_str)
                    == Some(work_id)
                && receipt
                    .get("lifecycleRevision")
                    .and_then(serde_json::Value::as_str)
                    == Some(expected_revision.as_str());
            if !valid {
                return Err("outgoing media settlement receipt is malformed or rebound".into());
            }
            if !Self::clear_media_fence_from_owner(owner, &expected_revision, work_id)? {
                return Err("could not clear committed outgoing media fence".into());
            }
            return Ok(());
        }
        if record.admitted_media_paths.is_empty() {
            return Err("pending outgoing media has no durable materialized paths".into());
        }
        let media_roots = get_agent_media_store_roots(self.agents_root.join(agent_id));
        let mut custody = OutgoingMediaCustody::default();
        for raw_path in record.admitted_media_paths {
            let path = PathBuf::from(raw_path);
            let Some(verified_path) = Self::verify_pending_media_path(&path, &media_roots)? else {
                continue;
            };
            custody.track(verified_path)
                .map_err(|_| "recovery media custody is already settled".to_string())?;
        }
        let cleanup_errors = custody.rollback();
        if !cleanup_errors.is_empty() {
            return Err(format!(
                "could not roll back pending outgoing media paths: {} cleanup errors",
                cleanup_errors.len()
            ));
        }
        if !Self::clear_media_fence_from_owner(owner, &expected_revision, work_id)? {
            return Err("could not clear rolled-back outgoing media fence".into());
        }
        Ok(())
    }

    fn verify_pending_media_path(
        path: &Path,
        media_roots: &[PathBuf; 2],
    ) -> Result<Option<PathBuf>, String> {
        let metadata = match fs::symlink_metadata(path) {
            Ok(metadata) => metadata,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(format!(
                    "could not inspect pending outgoing media path {}: {error}",
                    path.display()
                ));
            }
        };
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "pending outgoing media path is a symlink: {}",
                path.display()
            ));
        }
        let canonical_path = fs::canonicalize(path).map_err(|error| {
            format!(
                "could not canonicalize pending outgoing media path {}: {error}",
                path.display()
            )
        })?;
        let inside_media_root = media_roots
            .iter()
            .filter_map(|root| {
                fs::canonicalize(root)
                    .ok()
                    .map(|canonical_root| canonical_path.starts_with(canonical_root))
            })
            .any(|inside| inside);
        if !inside_media_root {
            return Err(format!(
                "pending outgoing media path is outside the agent media store: {}",
                path.display()
            ));
        }
        Ok(Some(canonical_path))
    }

    fn clear_media_fence_from_owner(
        owner: &SandAgentDb,
        expected_lifecycle_revision: &str,
        work_id: &str,
    ) -> Result<bool, String> {
        let Some(mut record) = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
        else {
            return Ok(false);
        };
        if record.lifecycle_revision != expected_lifecycle_revision
            || record.admitted_media_work_id.as_deref() != Some(work_id)
        {
            return Ok(false);
        }
        record.admitted_media_work_id = None;
        record.admitted_media_paths.clear();
        owner
            .compare_and_set_session_work_admission(
                Some(expected_lifecycle_revision),
                &record,
            )
            .map_err(|error| error.to_string())
    }

    fn recover_pending_memory_mutation(
        &self,
        agent_id: &str,
        owner: &SandAgentDb,
    ) -> Result<(), String> {
        let Some(mutation) = owner
            .read_pending_memory_mutation()
            .map_err(|error| error.to_string())?
        else {
            return Ok(());
        };
        let admission = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "pending memory mutation has no session admission".to_string())?;
        if mutation.mutation_id.trim().is_empty()
            || mutation.session_key != admission.session_key
            || mutation.session_id != admission.session_id
            || mutation.lifecycle_revision != admission.lifecycle_revision
            || !matches!(mutation.operation.as_str(), "remove" | "clear")
        {
            return Err("pending memory mutation journal is malformed or rebound".into());
        }
        let current_snapshot = owner
            .get_memory_prompt_snapshot()
            .map_err(|error| error.to_string())?;
        if current_snapshot != mutation.prior_snapshot {
            return Err("pending memory mutation snapshot is rebound or outcome-unknown".into());
        }
        let store = self.memory_service.store_for_agent(agent_id);
        Self::restore_memory_records(&store, &mutation.records)?;
        if !owner
            .clear_pending_memory_mutation(&mutation)
            .map_err(|error| error.to_string())?
        {
            return Err("pending memory mutation journal could not be cleared".into());
        }
        Ok(())
    }

    fn recover_pending_profile_mutation(
        &self,
        agent_id: &str,
        owner: &SandAgentDb,
    ) -> Result<(), String> {
        let Some(mutation) = owner
            .read_pending_profile_mutation()
            .map_err(|error| error.to_string())?
        else {
            return Ok(());
        };
        let admission = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "pending profile mutation has no session admission".to_string())?;
        if mutation.mutation_id.trim().is_empty()
            || !matches!(
                mutation.operation.as_str(),
                "profile-update" | "avatar-replace" | "avatar-remove"
            )
            || mutation.session_key != admission.session_key
            || mutation.session_id != admission.session_id
            || mutation.lifecycle_revision != admission.lifecycle_revision
            || mutation.before_files.is_empty()
            || mutation.after_files.is_empty()
        {
            return Err("pending profile mutation journal is malformed or rebound".into());
        }
        if owner
            .get_sand_profile()
            .map_err(|error| error.to_string())?
            != mutation.previous_profile
        {
            return Err("pending profile mutation owner state is rebound or outcome-unknown".into());
        }
        let db_path = self.session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        if profile_mutation_files_match(agent_dir, &mutation.before_files) {
            if !owner
                .clear_pending_profile_mutation(&mutation)
                .map_err(|error| error.to_string())?
            {
                return Err("could not clear already-before profile mutation journal".into());
            }
            return Ok(());
        }
        if !profile_mutation_files_match(agent_dir, &mutation.after_files) {
            return Err("pending profile mutation files do not match before or after state".into());
        }
        restore_profile_mutation_files(agent_dir, &mutation.before_files)?;
        if !owner
            .clear_pending_profile_mutation(&mutation)
            .map_err(|error| error.to_string())?
        {
            return Err("could not clear rolled-back profile mutation journal".into());
        }
        Ok(())
    }

    pub fn close_agent_db_owner(
        &self,
        agent_id: &str,
        checkpoint: bool,
    ) -> bool {
        if assert_valid_agent_session_id(agent_id).is_err() {
            return false;
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            Ok(self.close_agent_db_owner_locked(agent_id, checkpoint))
        })
        .unwrap_or(false)
    }

    fn close_agent_db_owner_locked(
        &self,
        agent_id: &str,
        checkpoint: bool,
    ) -> bool {
        let owner = self
            .db_owners
            .lock()
            .ok()
            .and_then(|mut owners| owners.remove(&SessionDbOwnerKey::Agent(agent_id.to_string())));
        if let Some(owner) = owner {
            owner.close(checkpoint);
            true
        } else {
            false
        }
    }

    /// Retire the exact per-agent owners while holding the same filesystem
    /// boundary used by deletion. This keeps owner close and blob-worker
    /// shutdown from interleaving with directory cleanup.
    pub fn retire_agent_owners(
        &self,
        agent_id: &str,
        flush: bool,
    ) -> Result<bool, String> {
        assert_valid_agent_session_id(agent_id).map_err(|error| error.to_string())?;
        self.with_agent_filesystem_mutation(agent_id, || {
            self.retire_agent_owners_locked(agent_id, flush)
        })
    }

    /// Close the exact per-agent owners while the caller already holds the
    /// filesystem/deletion boundary.
    pub(crate) fn retire_agent_owners_locked(
        &self,
        agent_id: &str,
        flush: bool,
    ) -> Result<bool, String> {
        let db_path = self.session_db_path(agent_id)?;
        let blob_path = conversation_blobs_path(&db_path);
        let store_closed = self.close_agent_store_owner_locked(agent_id, flush);
        let db_closed = self.close_agent_db_owner_locked(agent_id, flush);
        self.pool.close_store_blocking(&blob_path);
        Ok(store_closed || db_closed)
    }

    pub fn open_agent_store_owner(
        &self,
        agent_id: &str,
    ) -> Result<Arc<ProductionAgentStore>, String> {
        assert_valid_agent_session_id(agent_id).map_err(|error| error.to_string())?;
        self.with_agent_filesystem_mutation(agent_id, || {
            if self.is_agent_being_deleted(agent_id) {
                return Err(format!(
                    "agent {agent_id} deletion cleanup is pending; owner open is not admitted"
                ));
            }
            self.open_agent_store_owner_locked(agent_id)
        })
    }

    /// Open or reuse the AgentStore while the caller already holds the
    /// per-agent filesystem/deletion boundary.
    fn open_agent_store_owner_locked(
        &self,
        agent_id: &str,
    ) -> Result<Arc<ProductionAgentStore>, String> {
        self.open_agent_store_owner_locked_with_recovery(agent_id, true)
    }

    fn open_agent_store_owner_locked_with_recovery(
        &self,
        agent_id: &str,
        recover_pending: bool,
    ) -> Result<Arc<ProductionAgentStore>, String> {
        assert_valid_agent_session_id(agent_id).map_err(|error| error.to_string())?;
        let cached_owner = self.agent_store_owners.lock()
            .map_err(|_| "agent store owner map poisoned".to_string())?
            .get(agent_id).cloned();
        if let Some(owner) = cached_owner {
            owner.validate_cached_checkpoint_checked()?;
            let _ = self.open_agent_db_owner_locked_with_recovery(agent_id, recover_pending)?;
            return Ok(owner);
        }

        let db = self.open_agent_db_owner_locked_with_recovery(agent_id, recover_pending)?;
        let blob_store = self.create_agent_blob_store(agent_id)?;
        let candidate = Arc::new(ProductionAgentStore::new(db, blob_store));
        candidate.reset_from_db_checked()?;

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

    /// Retain Session's existing filesystem/deletion boundary while preserving
    /// the strict terminal error type. Only this module supplies the operation;
    /// no Runner/provider callback runs while the boundary is held.
    fn with_original_turn_terminal_owner<T>(
        &self,
        original: &OriginalTurnReference,
        operation: impl FnOnce(&SandAgentDb) -> Result<T, AgentDbProjectionError>,
    ) -> Result<T, AgentDbProjectionError> {
        original.validate().map_err(AgentDbProjectionError::OriginalTurnInvalid)?;
        self.with_agent_filesystem_mutation(&original.agent_id, || {
            Ok((|| {
                if self.deleting_agents.lock().map_err(|_| {
                    AgentDbProjectionError::OriginalTurnUnavailable(
                        "Session deletion registry is poisoned".into(),
                    )
                })?.contains(&original.agent_id) {
                    return Err(AgentDbProjectionError::OriginalTurnStale(
                        "agent deletion owns the terminal store".into(),
                    ));
                }
                let owner = self.open_agent_db_owner_locked_with_recovery(&original.agent_id, false)
                    .map_err(AgentDbProjectionError::OriginalTurnUnavailable)?;
                operation(owner.as_ref())
            })())
        }).map_err(AgentDbProjectionError::OriginalTurnUnavailable)?
    }

    fn original_turn_terminal_view_locked(
        &self,
        original: &OriginalTurnReference,
        owner: &SandAgentDb,
        fence: OriginalTurnTerminalFence,
    ) -> Result<OriginalTurnTerminalView, AgentDbProjectionError> {
        let store = self.open_agent_store_owner_locked_with_recovery(&original.agent_id, false)
            .map_err(AgentDbProjectionError::OriginalTurnUnavailable)?;
        let checkpoint = store.checkpoint_snapshot_checked()
            .map_err(AgentDbProjectionError::OriginalTurnUnavailable)?
            .ok_or_else(|| AgentDbProjectionError::OriginalTurnUnavailable(
                "original terminal has no canonical checkpoint".into(),
            ))?;
        if checkpoint.root_blob_id != fence.final_root_blob_id {
            return Err(AgentDbProjectionError::OriginalTurnStale(
                "checked canonical root differs from the inspected terminal owner".into(),
            ));
        }
        // Worker/blob I/O above does not hold the SQLite connection mutex.
        owner.recheck_original_turn_terminal_fence(&fence)?;
        Ok(OriginalTurnTerminalView {
            original_user_content: fence.original_user_content.clone(),
            settlement_lifecycle_revision: fence.settlement_lifecycle_revision.clone(),
            compaction: fence.compaction.clone(),
            original: original.clone(),
            checkpoint,
            store,
            fence,
        })
    }

    pub(crate) fn prepare_original_turn_terminal_view(
        &self,
        original: &OriginalTurnReference,
        expected_final_root: &[u8],
    ) -> Result<OriginalTurnTerminalView, AgentDbProjectionError> {
        self.with_original_turn_terminal_owner(original, |owner| {
            let fence = owner.prepare_original_turn_terminal_fence(original, expected_final_root)?;
            self.original_turn_terminal_view_locked(original, owner, fence)
        })
    }

    fn recheck_original_turn_checkpoint_locked(
        &self,
        view: &OriginalTurnTerminalView,
    ) -> Result<(), AgentDbProjectionError> {
        let store = self.open_agent_store_owner_locked_with_recovery(&view.original.agent_id, false)
            .map_err(AgentDbProjectionError::OriginalTurnUnavailable)?;
        if !Arc::ptr_eq(&store, &view.store) {
            return Err(AgentDbProjectionError::OriginalTurnStale(
                "canonical AgentStore owner changed after terminal inspection".into(),
            ));
        }
        let current = store.checkpoint_snapshot_checked()
            .map_err(AgentDbProjectionError::OriginalTurnUnavailable)?;
        if current.as_ref() != Some(&view.checkpoint) {
            return Err(AgentDbProjectionError::OriginalTurnStale(
                "checked canonical checkpoint changed after terminal qualification".into(),
            ));
        }
        Ok(())
    }

    pub(crate) fn commit_original_turn_terminal_record(
        &self,
        view: &OriginalTurnTerminalView,
        record: &OriginalTurnTerminalRecord,
    ) -> Result<OriginalTurnReceiptCommit, AgentDbProjectionError> {
        self.with_original_turn_terminal_owner(&view.original, |owner| {
            self.recheck_original_turn_checkpoint_locked(view)?;
            owner.commit_original_turn_terminal_record(&view.fence, record)
        })
    }

    pub(crate) fn read_original_turn_terminal_candidate(
        &self,
        original: &OriginalTurnReference,
    ) -> Result<Option<OriginalTurnTerminalRead>, AgentDbProjectionError> {
        self.with_original_turn_terminal_owner(original, |owner| {
            let Some((record, fence)) = owner.read_original_turn_terminal_candidate_checked(original)? else {
                return Ok(None);
            };
            let view = self.original_turn_terminal_view_locked(original, owner, fence)?;
            Ok(Some(OriginalTurnTerminalRead { record, view }))
        })
    }

    pub(crate) fn recheck_original_turn_terminal_read(
        &self,
        read: &OriginalTurnTerminalRead,
    ) -> Result<(), AgentDbProjectionError> {
        if !read.view.fence.contains_record(&read.record) {
            return Err(AgentDbProjectionError::OriginalTurnStale(
                "terminal candidate differs from the exact inspected slot".into(),
            ));
        }
        self.with_original_turn_terminal_owner(&read.view.original, |owner| {
            self.recheck_original_turn_checkpoint_locked(&read.view)?;
            owner.recheck_original_turn_terminal_fence(&read.view.fence)
        })
    }

    pub fn close_agent_store_owner(
        &self,
        agent_id: &str,
        flush: bool,
    ) -> bool {
        if assert_valid_agent_session_id(agent_id).is_err() {
            return false;
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            Ok(self.close_agent_store_owner_locked(agent_id, flush))
        })
        .unwrap_or(false)
    }

    fn close_agent_store_owner_locked(
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

    /// Serialize per-agent filesystem-backed mutation with the existing
    /// directory deletion owner. The caller must perform the durable
    /// admission check while this guard is held.
    pub fn with_agent_filesystem_mutation<T, F>(
        &self,
        agent_id: &str,
        operation: F,
    ) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        let lock = self
            .filesystem_mutation_locks
            .lock()
            .map_err(|_| "filesystem mutation lock registry is poisoned".to_string())?
            .entry(agent_id.to_string())
            .or_insert_with(|| Arc::new(Mutex::new(())))
            .clone();
        let _guard = lock
            .lock()
            .map_err(|_| format!("filesystem mutation lock is poisoned for agent {agent_id}"))?;
        operation()
    }

    /// Serialize an external per-agent mutation and perform the durable
    /// admission check while the same boundary is held. The operation must
    /// not call the public admission facade recursively.
    pub fn with_agent_admitted_filesystem_mutation<T, F>(
        &self,
        agent_id: &str,
        operation: F,
    ) -> Result<T, String>
    where
        F: FnOnce() -> Result<T, String>,
    {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            operation()
        })
    }

    /// Compose admitted filesystem and database mutations without reopening
    /// the public Session facade while its non-reentrant mutex is held.
    /// The callback must use this owner or existing locked helpers only.
    pub(crate) fn with_agent_admitted_db_mutation<T>(
        &self,
        agent_id: &str,
        operation: impl FnOnce(&SandAgentDb) -> Result<T, String>,
    ) -> Result<T, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            operation(owner.as_ref())
        })
    }

    pub fn begin_agent_delete(&self, agent_id: &str) -> Result<(), String> {
        let db_path = self.session_db_path(agent_id)?;
        self.roster_extras_cache.remove(agent_id);
        if let Ok(mut deleting) = self.deleting_agents.lock() {
            deleting.insert(agent_id.to_string());
        }
        let result = self.with_agent_filesystem_mutation(agent_id, || {
            if !db_path.is_file() {
                return Ok(());
            }
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let Some(mut record) = owner
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
            else {
                return Ok(());
            };
            if record.deletion_pending {
                return Ok(());
            }
            let expected_revision = record.lifecycle_revision.clone();
            record.deletion_pending = true;
            if owner
                .transition_session_deletion_pending(
                    Some(&expected_revision),
                    false,
                    &record,
                )
                .map_err(|error| error.to_string())?
            {
                Ok(())
            } else {
                Err(format!(
                    "agent {agent_id} lifecycle changed while publishing deletion fence"
                ))
            }
        });
        if result.is_err() {
            let _ = self
                .deleting_agents
                .lock()
                .map(|mut deleting| deleting.remove(agent_id));
        }
        result
    }

    pub fn clear_agent_delete_fast_path(&self, agent_id: &str) {
        if let Ok(mut deleting) = self.deleting_agents.lock() {
            deleting.remove(agent_id);
        }
    }

    pub fn end_agent_delete(&self, agent_id: &str) -> Result<(), String> {
        let result = self.with_agent_filesystem_mutation(agent_id, || {
            self.clear_agent_delete_fast_path(agent_id);
            let db_path = self.session_db_path(agent_id)?;
            if !db_path.is_file() {
                return Ok(());
            }
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let Some(mut record) = owner
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
            else {
                return Ok(());
            };
            if !record.deletion_pending {
                return Ok(());
            }
            let expected_revision = record.lifecycle_revision.clone();
            record.deletion_pending = false;
            if owner
                .clear_session_deletion_pending(&expected_revision, &record)
                .map_err(|error| error.to_string())?
            {
                Ok(())
            } else {
                Err(format!(
                    "agent {agent_id} lifecycle changed while clearing deletion fence"
                ))
            }
        });
        result
    }

    pub fn is_agent_being_deleted(&self, agent_id: &str) -> bool {
        self.deleting_agents
            .lock()
            .map(|deleting| deleting.contains(agent_id))
            .unwrap_or(false)
    }

    /// Reject new session-owned writes while the existing deletion owner has
    /// custody of the agent directory. Deletion itself uses its own file and
    /// store owner path and does not call these transcript mutation wrappers.
    pub fn assert_agent_work_admission(&self, agent_id: &str) -> Result<(), String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)
        })
    }

    /// Admission check for callers that already hold the per-agent
    /// filesystem/deletion boundary. This must not call the public admission
    /// facade, because that facade intentionally acquires the same mutex.
    fn assert_agent_work_admission_locked(&self, agent_id: &str) -> Result<(), String> {
        if self.is_agent_being_deleted(agent_id) {
            return Err(format!(
                "agent {agent_id} deletion cleanup is pending; new session work is not admitted"
            ));
        }
        self.existing_session_db_path(agent_id)?;
        let Some(record) = self
            .open_agent_db_owner_locked(agent_id)?
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
        else {
            return Ok(());
        };
        let expected_session_key = format!("agent:{agent_id}:main");
        if record.session_id != agent_id
            || record.session_key != expected_session_key
            || record.lifecycle_revision.trim().is_empty()
        {
            return Err(format!(
                "agent {agent_id} session admission identity is malformed"
            ));
        }
        if record.deletion_pending {
            return Err(format!(
                "agent {agent_id} durable deletion cleanup is pending; new session work is not admitted"
            ));
        }
        Ok(())
    }

    fn assert_agent_transcript_write_admission_locked(
        &self,
        agent_id: &str,
    ) -> Result<(), String> {
        self.assert_agent_work_admission_locked(agent_id)?;
        if self
            .open_agent_db_owner_locked(agent_id)?
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
            .is_some_and(|record| record.archived_at_ms.is_some())
        {
            return Err(format!(
                "agent {agent_id} session is archived; agent-only transcript writes are not admitted"
            ));
        }
        Ok(())
    }

    fn assert_agent_transcript_write_admission(&self, agent_id: &str) -> Result<(), String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)
        })
    }

    pub fn ensure_main_session_work_admission(&self, agent_id: &str) -> Result<(), String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.ensure_main_session_work_admission_locked(agent_id)
        })
    }

    fn ensure_main_session_work_admission_locked(&self, agent_id: &str) -> Result<(), String> {
        let db = self.open_agent_db_owner_locked(agent_id)?;
        let expected_session_key = format!("agent:{agent_id}:main");
        if let Some(record) = db
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
        {
            if record.session_id != agent_id
                || record.session_key != expected_session_key
                || record.lifecycle_revision.trim().is_empty()
            {
                return Err(format!(
                    "agent {agent_id} session admission identity is malformed"
                ));
            }
            return Ok(());
        }
        let lifecycle_revision = db
            .ensure_lifecycle_revision()
            .map_err(|error| error.to_string())?;
        let record = SessionWorkAdmissionRecord {
            session_key: format!("agent:{agent_id}:main"),
            session_id: agent_id.to_string(),
            lifecycle_revision,
            deletion_pending: false,
            archived_at_ms: None,
            archived_by: None,
            archive_reason: None,
            admitted_work_id: None,
            admitted_media_work_id: None,
            admitted_media_paths: Vec::new(),
            terminal_result_idempotency_key: None,
            provider_review_id: None,
            provider_review_phase: None,
            provider_review_continuation_run_id: None,
            provider_review: None,
        };
        let changed = db
            .compare_and_set_session_work_admission(None, &record)
            .map_err(|error| error.to_string())?;
        if changed {
            return Ok(());
        }
        let current = db
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| {
                format!(
                    "agent {agent_id} session admission creation lost its race without a record"
                )
            })?;
        if current.session_id != agent_id
            || current.session_key != expected_session_key
            || current.lifecycle_revision.trim().is_empty()
        {
            return Err(format!(
                "agent {agent_id} session admission identity is malformed after create race"
            ));
        }
        Ok(())
    }

    pub fn assert_session_work_admission(
        &self,
        agent_id: &str,
        session_key: Option<&str>,
    ) -> Result<(), String> {
        self.assert_agent_work_admission(agent_id)?;
        let Some(session_key) = session_key.map(str::trim).filter(|value| !value.is_empty())
        else {
            return Ok(());
        };
        let Some(record) = self.read_session_work_admission(agent_id)? else {
            return Ok(());
        };
        if record.session_key != session_key {
            return Err(format!(
                "session work admission key mismatch: expected {}, got {}",
                record.session_key, session_key
            ));
        }
        if record.archived_at_ms.is_some() {
            return Err(format!(
                "session {session_key} is archived and does not admit new work"
            ));
        }
        Ok(())
    }

    /// Capture the generation that a background producer is admitting work
    /// against. The caller must carry this value to its terminal edge and
    /// recheck it before treating a detached result as committed.
    pub fn capture_session_work_admission(
        &self,
        agent_id: &str,
        session_key: &str,
        work_id: &str,
    ) -> Result<String, String> {
        let session_key = session_key.trim();
        let work_id = work_id.trim();
        if session_key.is_empty() || work_id.is_empty() {
            return Err("session work admission requires a session key and work id".into());
        }
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.ensure_main_session_work_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let record = owner
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "session work admission record is unavailable".to_string())?;
            if record.session_key != session_key {
                return Err(format!(
                    "session work admission key mismatch: expected {}, got {}",
                    record.session_key, session_key
                ));
            }
            if record.archived_at_ms.is_some() {
                return Err(format!(
                    "session {session_key} is archived and does not admit new work"
                ));
            }
            if let Some(admitted_work_id) = record.admitted_work_id.as_deref() {
                if admitted_work_id != work_id {
                    return Err(format!(
                        "session {session_key} already has admitted background work"
                    ));
                }
                return Ok(record.lifecycle_revision);
            }
            let lifecycle_revision = record.lifecycle_revision.clone();
            let mut next = record;
            next.admitted_work_id = Some(work_id.to_string());
            if !owner
                .compare_and_set_session_work_admission(Some(&lifecycle_revision), &next)
                .map_err(|error| error.to_string())?
            {
                return Err(format!(
                    "session {session_key} generation changed while admitting work"
                ));
            }
            Ok(lifecycle_revision)
        })
    }

    /// Recheck the generation immediately before a background terminal result
    /// is exposed to its consumer. A missing record, rebound session, archive,
    /// deletion fence, or changed lifecycle is fail-closed.
    pub fn assert_session_work_admission_generation(
        &self,
        agent_id: &str,
        session_key: &str,
        expected_lifecycle_revision: &str,
    ) -> Result<(), String> {
        self.assert_agent_work_admission(agent_id)?;
        let record = self
            .read_session_work_admission(agent_id)?
            .ok_or_else(|| "session work admission record disappeared".to_string())?;
        if record.session_key != session_key
            || record.lifecycle_revision != expected_lifecycle_revision
            || record.archived_at_ms.is_some()
        {
            return Err(format!(
                "session work admission generation changed for {session_key}"
            ));
        }
        Ok(())
    }

    /// Capture one durable media-custody identity before an owned attachment
    /// becomes visible to the transcript/channel path.
    pub fn capture_session_media_admission(
        &self,
        agent_id: &str,
        session_key: &str,
        work_id: &str,
        media_path: Option<&Path>,
    ) -> Result<String, String> {
        let session_key = session_key.trim();
        let work_id = work_id.trim();
        if session_key.is_empty() || work_id.is_empty() {
            return Err("media custody requires a session key and work id".into());
        }
        let media_path = media_path
            .map(|path| {
                path.to_str()
                    .map(ToOwned::to_owned)
                    .ok_or_else(|| "media custody path must be valid UTF-8".to_string())
            })
            .transpose()?;
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let record = owner
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "session work admission record is unavailable".to_string())?;
            if record.session_key != session_key {
                return Err(format!(
                    "media custody session key mismatch: expected {}, got {}",
                    record.session_key, session_key
                ));
            }
            if record.archived_at_ms.is_some() {
                return Err(format!(
                    "session {session_key} is archived and does not admit media custody"
                ));
            }
            if record.admitted_media_work_id.is_none() && !record.admitted_media_paths.is_empty() {
                return Err(format!(
                    "session {session_key} has media paths without a pending media identity"
                ));
            }
            if let Some(admitted_media_work_id) = record.admitted_media_work_id.as_deref() {
                if admitted_media_work_id != work_id {
                    return Err(format!(
                        "session {session_key} already has pending outgoing media"
                    ));
                }
                let lifecycle_revision = record.lifecycle_revision.clone();
                let mut next = record;
                if let Some(path) = media_path.as_deref() {
                    if !next.admitted_media_paths.iter().any(|known| known == path) {
                        next.admitted_media_paths.push(path.to_string());
                        if !owner
                            .compare_and_set_session_work_admission(
                                Some(&lifecycle_revision),
                                &next,
                            )
                            .map_err(|error| error.to_string())?
                        {
                            return Err(format!(
                                "session {session_key} generation changed while recording media custody"
                            ));
                        }
                    }
                }
                return Ok(lifecycle_revision);
            }
            let lifecycle_revision = record.lifecycle_revision.clone();
            if media_path.is_none() {
                return Err(
                    "media custody requires a materialized path for new media work".into(),
                );
            }
            let mut next = record;
            next.admitted_media_work_id = Some(work_id.to_string());
            next.admitted_media_paths = media_path.into_iter().collect();
            if !owner
                .compare_and_set_session_work_admission(Some(&lifecycle_revision), &next)
                .map_err(|error| error.to_string())?
            {
                return Err(format!(
                    "session {session_key} generation changed while admitting media custody"
                ));
            }
            Ok(lifecycle_revision)
        })
    }

    /// Release media custody only for the captured session generation and
    /// identity. A failed release intentionally leaves the durable fence in
    /// place so archive/restore remains fail-closed.
    pub fn release_session_media_admission(
        &self,
        agent_id: &str,
        expected_lifecycle_revision: &str,
        work_id: &str,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let Some(mut record) = owner
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
            else {
                return Ok(false);
            };
            if record.lifecycle_revision != expected_lifecycle_revision
                || record.admitted_media_work_id.as_deref() != Some(work_id.trim())
            {
                return Ok(false);
            }
            record.admitted_media_work_id = None;
            record.admitted_media_paths.clear();
            owner
                .compare_and_set_session_work_admission(
                    Some(expected_lifecycle_revision),
                    &record,
                )
                .map_err(|error| error.to_string())
        })
    }

    pub fn read_session_work_admission(
        &self,
        agent_id: &str,
    ) -> Result<Option<SessionWorkAdmissionRecord>, String> {
        if agent_id.trim().is_empty() {
            return Err("session admission read requires a non-empty agent identity".into());
        }
        let record = self
            .open_agent_db_owner(agent_id)?
            .read_session_work_admission()
            .map_err(|error| error.to_string())?;
        let Some(record) = record else {
            return Ok(None);
        };
        let expected_session_key = format!("agent:{agent_id}:main");
        if record.session_id != agent_id
            || record.session_key != expected_session_key
            || record.lifecycle_revision.trim().is_empty()
        {
            return Err(format!(
                "agent {agent_id} session admission identity is malformed"
            ));
        }
        Ok(Some(record))
    }

    /// Read one agent-scoped KV projection while retaining the Session owner
    /// serialization boundary. Tool adapters must not retain a raw
    /// `SandAgentDb` handle as their production state owner.
    pub fn read_agent_kv_projection(
        &self,
        agent_id: &str,
        key: &str,
    ) -> Result<Option<String>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .read_kv(key)
                .map_err(|error| error.to_string())
        })
    }

    /// Write one agent-scoped KV projection through the admitted Session
    /// owner and its per-agent filesystem/deletion boundary.
    pub fn write_agent_kv_projection(
        &self,
        agent_id: &str,
        key: &str,
        value: &str,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .write_kv(key, value)
                .map_err(|error| error.to_string())
        })
    }

    /// Read the purpose projection through the Session owner without exposing
    /// the general agent database handle to transcript lifecycle producers.
    pub fn read_agent_purpose(&self, agent_id: &str) -> Result<Option<String>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .get_agent_purpose()
                .map_err(|error| error.to_string())
        })
    }

    /// Read the unread projection through the Session owner without exposing
    /// the general agent database handle to transcript lifecycle producers.
    pub fn read_agent_unread_state(&self, agent_id: &str) -> Result<UnreadState, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .get_unread_state()
                .map_err(|error| error.to_string())
        })
    }

    /// Apply a session-scoped model selection while holding the same
    /// per-agent filesystem/deletion boundary used by every other session
    /// mutation. The policy evaluator remains the model-picker owner; this
    /// method owns only admission, serialization, and the owner-local write
    /// edge.
    pub fn apply_session_model_selection(
        &self,
        agent_id: &str,
        params: ApplySessionModelSelectionParams<'_>,
        current_time_ms: u64,
    ) -> Result<ApplySessionModelSelectionResult, String> {
        if !params.request.reset_to_default
            && (params.request.provider.trim().is_empty()
                || params.request.model.trim().is_empty())
        {
            return Err("session model selection requires provider and model".into());
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let db = self.open_agent_db_owner_locked(agent_id)?;
            let admission = db
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "session model selection requires session admission".to_string())?;
            if admission.session_id != agent_id
                || admission.session_key.trim().is_empty()
                || admission.lifecycle_revision.trim().is_empty()
            {
                return Err("session model selection admission identity is malformed".into());
            }
            let expected_lifecycle_revision = admission.lifecycle_revision.clone();
            let mut state = read_session_model_entry(&db, agent_id)?;
            let result = apply_model_selection(&mut state, params, current_time_ms);
            if matches!(&result, ApplySessionModelSelectionResult::Applied(_))
                && !write_session_model_entry(
                    &db,
                    agent_id,
                    &expected_lifecycle_revision,
                    &state,
                )?
            {
                return Err("SESSION_MODEL_SELECTION_NOT_COMMITTED_OR_STALE".into());
            }
            Ok(result)
        })
    }

    /// Register one background automation run as a durable per-run session
    /// projection. The row remains owned by the agent DB; cleanup and archive
    /// are intentionally separate reaper responsibilities.
    pub fn begin_cron_session_run(
        &self,
        agent_id: &str,
        automation_id: &str,
        run_id: &str,
        started_at_ms: f64,
    ) -> Result<bool, String> {
        if agent_id.trim().is_empty()
            || automation_id.trim().is_empty()
            || run_id.trim().is_empty()
        {
            return Err("cron session run requires agent, automation, and run identities".into());
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let db = self.open_agent_db_owner_locked(agent_id)?;
            let admission = db
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "cron session run requires session admission".to_string())?;
            let session_key = format!(
                "agent:{agent_id}:cron:{automation_id}:{run_id}"
            );
            db.begin_cron_session_run(
                &session_key,
                automation_id,
                run_id,
                &session_key,
                &admission.lifecycle_revision,
                started_at_ms,
            )
            .map_err(|error| error.to_string())
        })
    }

    /// Settle the same per-run projection only against the current session
    /// identity and lifecycle revision. A changed lifecycle returns false and
    /// remains visible for the later reconciliation/reaper slice.
    pub fn finish_cron_session_run(
        &self,
        agent_id: &str,
        automation_id: &str,
        run_id: &str,
        outcome: &str,
        finished_at_ms: f64,
        detail: Option<&str>,
    ) -> Result<bool, String> {
        if agent_id.trim().is_empty()
            || automation_id.trim().is_empty()
            || run_id.trim().is_empty()
        {
            return Err("cron session settlement requires agent, automation, and run identities".into());
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let db = self.open_agent_db_owner_locked(agent_id)?;
            let admission = db
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "cron session run requires session admission".to_string())?;
            let session_key = format!(
                "agent:{agent_id}:cron:{automation_id}:{run_id}"
            );
            db.finish_cron_session_run(
                &session_key,
                &session_key,
                &admission.lifecycle_revision,
                outcome,
                finished_at_ms,
                detail,
            )
            .map_err(|error| error.to_string())
        })
    }

    pub fn finish_cron_session_run_with_fences(
        &self,
        agent_id: &str,
        automation_id: &str,
        run_id: &str,
        outcome: &str,
        finished_at_ms: f64,
        detail: Option<&str>,
        active_controller: bool,
        pending_outgoing_media: bool,
        unsettled_subagent: bool,
    ) -> Result<bool, String> {
        if agent_id.trim().is_empty()
            || automation_id.trim().is_empty()
            || run_id.trim().is_empty()
        {
            return Err("cron session settlement requires agent, automation, and run identities".into());
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let db = self.open_agent_db_owner_locked(agent_id)?;
            let admission = db
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "cron session settlement requires session admission".to_string())?;
            let session_key = format!(
                "agent:{agent_id}:cron:{automation_id}:{run_id}"
            );
            db.finish_cron_session_run_with_fences(
                &session_key,
                &session_key,
                &admission.lifecycle_revision,
                outcome,
                finished_at_ms,
                detail,
                active_controller,
                pending_outgoing_media,
                unsettled_subagent,
            )
            .map_err(|error| error.to_string())
        })
    }

    pub fn publish_cron_session_run_fences(
        &self,
        agent_id: &str,
        automation_id: &str,
        run_id: &str,
        active_controller: bool,
        pending_outgoing_media: bool,
        unsettled_subagent: bool,
    ) -> Result<bool, String> {
        if agent_id.trim().is_empty()
            || automation_id.trim().is_empty()
            || run_id.trim().is_empty()
        {
            return Err("cron fence publication requires agent, automation, and run identities".into());
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let db = self.open_agent_db_owner_locked(agent_id)?;
            let admission = db
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
                .ok_or_else(|| "cron fence publication requires session admission".to_string())?;
            let session_key = format!(
                "agent:{agent_id}:cron:{automation_id}:{run_id}"
            );
            db.publish_cron_session_run_fences(
                &session_key,
                &session_key,
                &admission.lifecycle_revision,
                active_controller,
                pending_outgoing_media,
                unsettled_subagent,
            )
            .map_err(|error| error.to_string())
        })
    }

    pub fn list_cron_session_runs(
        &self,
        agent_id: &str,
    ) -> Result<Vec<CronSessionRunRecord>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            Ok(self
                .open_agent_db_owner_locked(agent_id)?
                .list_cron_session_runs()
                .map_err(|error| error.to_string())?)
        })
    }

    /// Project only rows that are eligible for a future guarded cleanup
    /// operation. This method never mutates the row or transcript artifacts.
    pub fn list_expired_cron_session_runs(
        &self,
        agent_id: &str,
        now_ms: f64,
        retention_ms: Option<u64>,
    ) -> Result<Vec<CronSessionRunRecord>, String> {
        let Some(retention_ms) = retention_ms else {
            return Ok(Vec::new());
        };
        if !now_ms.is_finite() || now_ms < 0.0 || retention_ms == 0 {
            return Ok(Vec::new());
        }
        let rows = self.list_cron_session_runs(agent_id)?;
        let expected_session_key_prefix = format!("agent:{agent_id}:cron:");
        let now_ms = now_ms.floor();
        let retention_ms = retention_ms as f64;
        Ok(rows
            .into_iter()
            .filter(|row| {
                !row.base_session
                    && !row.automation_id.trim().is_empty()
                    && !row.run_id.trim().is_empty()
                    && row.session_key
                        == format!(
                            "{}{}:{}",
                            expected_session_key_prefix,
                            row.automation_id,
                            row.run_id
                        )
                    && row.session_id == row.session_key
                    && !row.lifecycle_revision.trim().is_empty()
                    && row.state == "terminal"
                    && !row.active_controller
                    && !row.pending_outgoing_media
                    && !row.unsettled_subagent
                    && row.fence_state_known
                    && row.updated_at_ms.is_finite()
                    && row.updated_at_ms >= 0.0
                    && now_ms >= row.updated_at_ms
                    && now_ms - row.updated_at_ms >= retention_ms
            })
            .collect())
    }

    pub(crate) fn read_session_work_admission_locked(
        &self,
        agent_id: &str,
    ) -> Result<Option<SessionWorkAdmissionRecord>, String> {
        if agent_id.trim().is_empty() {
            return Err("locked session admission read requires a non-empty agent identity".into());
        }
        let record = self
            .open_agent_db_owner_locked(agent_id)?
            .read_session_work_admission()
            .map_err(|error| error.to_string())?;
        let Some(record) = record else {
            return Ok(None);
        };
        let expected_session_key = format!("agent:{agent_id}:main");
        if record.session_id != agent_id
            || record.session_key != expected_session_key
            || record.lifecycle_revision.trim().is_empty()
        {
            return Err(format!(
                "agent {agent_id} session admission identity is malformed"
            ));
        }
        Ok(Some(record))
    }

    pub fn read_session_communication_policy(
        &self,
        agent_id: &str,
    ) -> Result<
        Option<super::agent_session::SessionCommunicationPolicyOverride>,
        String,
    > {
        let db = self.open_agent_db_owner(agent_id)?;
        super::agent_session::read_session_communication_policy(&db, agent_id)
    }

    pub fn write_session_communication_policy(
        &self,
        agent_id: &str,
        expected_session_id: &str,
        expected_lifecycle_revision: &str,
        policy: &super::agent_session::SessionCommunicationPolicyOverride,
    ) -> Result<(), String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let db = self.open_agent_db_owner_locked(agent_id)?;
            if !super::agent_session::write_session_communication_policy(
                &db,
                expected_session_id,
                expected_lifecycle_revision,
                policy,
            )? {
                return Err("SESSION_LIFECYCLE_CHANGED".into());
            }
            Ok(())
        })
    }

    pub fn compare_and_set_session_work_admission(
        &self,
        agent_id: &str,
        expected_lifecycle_revision: Option<&str>,
        next: &SessionWorkAdmissionRecord,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .compare_and_set_session_work_admission(expected_lifecycle_revision, next)
                .map_err(|error| error.to_string())
        })
    }

    pub fn read_session_provider_review(
        &self,
        agent_id: &str,
    ) -> Result<Option<SessionProviderReview>, String> {
        self.open_agent_db_owner(agent_id)?
            .read_session_provider_review()
            .map_err(|error| error.to_string())
    }

    pub fn record_session_provider_review(
        &self,
        agent_id: &str,
        expected_lifecycle_revision: &str,
        review: &SessionProviderReview,
    ) -> Result<SessionProviderReview, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .record_session_provider_review(expected_lifecycle_revision, review)
                .map_err(|error| error.to_string())
        })
    }

    pub fn issue_provider_review_acknowledgment(
        &self,
        agent_id: &str,
        expected_lifecycle_revision: &str,
        review_id: &str,
        next_run_id: &str,
    ) -> Result<ProviderReviewAcknowledgment, String> {
        let agent_id = agent_id.trim();
        let expected_lifecycle_revision = expected_lifecycle_revision.trim();
        let review_id = review_id.trim();
        let next_run_id = next_run_id.trim();
        if agent_id.is_empty()
            || expected_lifecycle_revision.is_empty()
            || review_id.is_empty()
            || next_run_id.is_empty()
        {
            return Err("provider review acknowledgment requires bounded identity".into());
        }
        self.assert_agent_work_admission(agent_id)?;
        let record = self
            .read_session_work_admission(agent_id)?
            .ok_or_else(|| "session work admission record is unavailable".to_string())?;
        let review = record
            .provider_review
            .clone()
            .ok_or_else(|| "provider review is unavailable".to_string())?;
        let continuation_supported = review
            .continuation_message
            .as_deref()
            .is_some_and(|message| !message.trim().is_empty())
            && review.provider == "openai"
            && ((review.runtime_id == "codex"
                && review.native_thread_id.as_deref().is_some_and(|value| !value.trim().is_empty())
                && review.native_turn_id.as_deref().is_some_and(|value| !value.trim().is_empty()))
                || (review.runtime_id == "openclaw"
                    && review.api.as_deref() == Some("openai-chatgpt-responses")));
        if record.lifecycle_revision != expected_lifecycle_revision
            || record.session_id != review.session_id
            || record.provider_review_id.as_deref() != Some(review_id)
            || record.provider_review_phase.as_deref() != Some("pending")
            || review.id != review_id
            || review.run_id == next_run_id
            || !continuation_supported
        {
            return Err("provider review changed or cannot be continued".into());
        }
        let token = Uuid::new_v4().to_string();
        self.provider_review_capabilities
            .lock()
            .map_err(|_| "provider review capability owner is poisoned".to_string())?
            .insert(
                token.clone(),
                ProviderReviewCapabilityState {
                    agent_id: agent_id.to_string(),
                    lifecycle_revision: expected_lifecycle_revision.to_string(),
                    review_id: review_id.to_string(),
                    next_run_id: next_run_id.to_string(),
                },
            );
        Ok(ProviderReviewAcknowledgment { token })
    }

    pub fn claim_provider_review_acknowledgment(
        &self,
        acknowledgment: ProviderReviewAcknowledgment,
        next_run_id: &str,
        provider: &str,
        model: &str,
        runtime_id: &str,
        api: Option<&str>,
    ) -> Result<SessionProviderReview, String> {
        let state = self
            .provider_review_capabilities
            .lock()
            .map_err(|_| "provider review capability owner is poisoned".to_string())?
            .remove(&acknowledgment.token)
            .ok_or_else(|| "provider review acknowledgment is no longer current".to_string())?;
        self.with_agent_filesystem_mutation(&state.agent_id, || {
            self.assert_agent_work_admission_locked(&state.agent_id)?;
            if provider.trim().is_empty()
                || model.trim().is_empty()
                || runtime_id.trim().is_empty()
                || next_run_id.trim() != state.next_run_id
            {
                return Err("provider review run or runtime identity is incomplete or changed".into());
            }
            let owner = self.open_agent_db_owner_locked(&state.agent_id)?;
            let Some(mut record) = owner
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
            else {
                return Err("session work admission record is unavailable".into());
            };
            let review = record
                .provider_review
                .clone()
                .ok_or_else(|| "provider review is unavailable".to_string())?;
            if record.lifecycle_revision != state.lifecycle_revision
                || record.session_id != review.session_id
                || record.provider_review_id.as_deref() != Some(state.review_id.as_str())
                || record.provider_review_phase.as_deref() != Some("pending")
                || record.admitted_work_id.as_deref() != Some(state.next_run_id.as_str())
                || record.deletion_pending
                || review.id != state.review_id
                || review.provider != provider.trim()
                || review.model != model.trim()
                || review.runtime_id != runtime_id.trim()
                || review.api.as_deref() != api.map(str::trim)
            {
                return Err("provider review runtime or lifecycle identity changed".into());
            }
            record.provider_review_phase = Some("accepted".into());
            record.provider_review_continuation_run_id = Some(state.next_run_id.clone());
            if !owner
                .compare_and_set_session_work_admission(Some(&state.lifecycle_revision), &record)
                .map_err(|error| error.to_string())?
            {
                return Err("provider review acknowledgment lost its lifecycle race".into());
            }
            Ok(review)
        })
    }

    pub fn settle_provider_review_continuation(
        &self,
        agent_id: &str,
        expected_lifecycle_revision: &str,
        continuation_run_id: &str,
        outcome: ProviderReviewContinuationOutcome,
    ) -> Result<bool, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .settle_provider_review_continuation(
                    expected_lifecycle_revision,
                    continuation_run_id,
                    outcome,
                )
                .map_err(|error| error.to_string())
        })
    }

    pub fn release_provider_review_continuation(
        &self,
        agent_id: &str,
        expected_lifecycle_revision: &str,
        continuation_run_id: &str,
    ) -> Result<bool, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .release_provider_review_continuation(
                    expected_lifecycle_revision,
                    continuation_run_id,
                )
                .map_err(|error| error.to_string())
        })
    }

    pub fn transition_session_work_archive(
        &self,
        agent_id: &str,
        session_key: &str,
        expected_session_id: &str,
        expected_lifecycle_revision: Option<&str>,
        archived: bool,
        archived_by: Option<&str>,
        archive_reason: Option<&str>,
        archived_at_ms: f64,
    ) -> Result<SessionWorkAdmissionRecord, String> {
        let expected_key = format!("agent:{agent_id}:main");
        if session_key.trim() != expected_key {
            return Err("session archive transition requires the canonical main session key".into());
        }
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            self.ensure_main_session_work_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .transition_session_work_archive(
                    session_key,
                    expected_session_id,
                    expected_lifecycle_revision,
                    archived,
                    archived_by,
                    archive_reason,
                    archived_at_ms,
                )
                .map_err(|error| error.to_string())
        })
    }

    pub fn commit_session_work_result(
        &self,
        agent_id: &str,
        session_key: &str,
        expected_lifecycle_revision: &str,
        work_id: &str,
        idempotency_key: &str,
        content: &str,
        sent_message_count: usize,
    ) -> Result<SessionWorkResultCommitOutcome, String> {
        let publication = self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .commit_session_work_result_deferred(
                    session_key,
                    expected_lifecycle_revision,
                    work_id,
                    idempotency_key,
                    content,
                    sent_message_count,
                )
                .map_err(|error| error.to_string())
        })?;
        // Listeners may read through this same Session facade. Its non-reentrant
        // filesystem/deletion boundary and the DB connection are both released.
        Ok(publication.publish())
    }

    pub fn release_session_work_admission(
        &self,
        agent_id: &str,
        expected_lifecycle_revision: &str,
        work_id: &str,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let Some(mut record) = owner
                .read_session_work_admission()
                .map_err(|error| error.to_string())?
            else {
                return Ok(false);
            };
            if record.lifecycle_revision != expected_lifecycle_revision
                || record.admitted_work_id.as_deref() != Some(work_id)
            {
                return Ok(false);
            }
            record.admitted_work_id = None;
            owner
                .compare_and_set_session_work_admission(
                    Some(expected_lifecycle_revision),
                    &record,
                )
                .map_err(|error| error.to_string())
        })
    }


    pub fn set_agent_sand_profile(
        &self,
        agent_id: &str,
        profile: &SandProfile,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .set_sand_profile(profile)
                .map_err(|error| error.to_string())
        })
    }

    pub fn mark_agent_activity(&self, agent_id: &str, at: f64) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .mark_activity(at)
                .map_err(|error| error.to_string())
        })
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
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .mark_viewed(at, preserve_manual_unread)
                .map_err(|error| error.to_string())
        })
    }

    pub fn set_agent_unread(
        &self,
        agent_id: &str,
        unread: bool,
        at: f64,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            if unread {
                owner.mark_unread(at)
            } else {
                owner.mark_read(at)
            }
            .map_err(|error| error.to_string())
        })
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
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .set_introduction_pending(pending)
                .map_err(|error| error.to_string())
        })
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
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .set_automation_spend_guard_state(state)
                .map_err(|error| error.to_string())
        })
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
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .add_conversation_partner(partner_id)
                .map_err(|error| error.to_string())
        })
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
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .set_awaiting_user_response(state)
                .map_err(|error| error.to_string())
        })
    }

    pub fn set_agent_awaiting_user_response_for_tab(
        &self,
        agent_id: &str,
        tab_id: &str,
        state: Option<&AwaitingUserResponse>,
        if_since_before: Option<f64>,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .set_awaiting_user_response_for_tab(tab_id, state, if_since_before)
                .map_err(|error| error.to_string())
        })
    }

    pub fn record_agent_request_id(
        &self,
        agent_id: &str,
        request_id: &str,
        at: f64,
        prompt: Option<&str>,
        source: Option<&str>,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .record_request_id(request_id, at, prompt, source)
                .map_err(|error| error.to_string())
        })
    }

    pub fn record_agent_episode_turn(
        &self,
        agent_id: &str,
        turn: &EpisodeTurn,
    ) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .record_episode_turn(turn)
                .map_err(|error| error.to_string())
        })
    }

    pub fn get_agent_pending_episode_turns(
        &self,
        agent_id: &str,
    ) -> Result<Vec<EpisodeTurn>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .get_pending_episode_turns()
                .map_err(|error| error.to_string())
        })
    }

    pub fn clear_agent_pending_episode_turns(&self, agent_id: &str) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .clear_pending_episode_turns()
                .map_err(|error| error.to_string())
        })
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
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .set_memory_prompt_snapshot(snapshot)
                .map_err(|error| error.to_string())
        })
    }

    pub fn clear_agent_memory_prompt_snapshot(&self, agent_id: &str) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .clear_memory_prompt_snapshot()
                .map_err(|error| error.to_string())
        })
    }

    fn restore_memory_records(
        store: &FileMemoryStore,
        records: &[MemoryRecoveryRecord],
    ) -> Result<(), String> {
        store
            .restore_records_exact(records)
            .map_err(|error| error.to_string())
    }

    fn prepare_memory_mutation(
        owner: &SandAgentDb,
        operation: &str,
        records: Vec<MemoryRecoveryRecord>,
        prior_snapshot: Option<MemoryPromptSnapshot>,
    ) -> Result<PendingMemoryMutation, String> {
        let admission = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "memory mutation requires a session admission".to_string())?;
        let mutation = PendingMemoryMutation {
            mutation_id: format!("memory-{operation}:{}", Uuid::new_v4()),
            operation: operation.to_string(),
            session_key: admission.session_key,
            session_id: admission.session_id,
            lifecycle_revision: admission.lifecycle_revision,
            records,
            prior_snapshot,
        };
        if !owner
            .prepare_pending_memory_mutation(&mutation)
            .map_err(|error| error.to_string())?
        {
            return Err("memory mutation could not establish its durable journal".into());
        }
        Ok(mutation)
    }

    fn rollback_memory_mutation(
        owner: &SandAgentDb,
        store: &FileMemoryStore,
        mutation: &PendingMemoryMutation,
    ) -> Result<(), String> {
        Self::restore_memory_records(store, &mutation.records)?;
        if !owner
            .clear_pending_memory_mutation(mutation)
            .map_err(|error| error.to_string())?
        {
            return Err("memory mutation journal could not be cleared after rollback".into());
        }
        Ok(())
    }

    pub fn write_agent_memory(
        &self,
        agent_id: &str,
        content: &str,
        tier: MemoryTier,
    ) -> StateWriteResult {
        let result = self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let store = self.memory_service.store_for_agent(agent_id);
            let payload = match tier {
                MemoryTier::Note => format!("{MEMORY_NOTE_PREFIX}{}", content.trim()),
                MemoryTier::Profile | MemoryTier::Log => content.trim().to_string(),
            };
            let kind = match tier {
                MemoryTier::Profile => MemoryKind::Profile,
                MemoryTier::Note | MemoryTier::Log => MemoryKind::Log,
            };
            store
                .add_memory(&payload, chrono::Utc::now().timestamp_millis(), kind)
                .map_err(|error| error.to_string())
        });
        match result {
            Ok(Some(record)) => StateWriteResult::success(format!(
                "Remembered in agent memory ({}): {}",
                match tier {
                    MemoryTier::Profile => "profile",
                    MemoryTier::Note => "note",
                    MemoryTier::Log => "log",
                },
                record.content
            )),
            Ok(None) => StateWriteResult::failure(
                "nothing was saved to agent memory - the fact was empty or already recorded.",
            ),
            Err(error) => StateWriteResult::failure(format!("could not update agent memory: {error}")),
        }
    }

    pub fn remove_agent_memory(&self, agent_id: &str, memory_id: &str) -> Result<bool, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let prior_snapshot = owner
                .get_memory_prompt_snapshot()
                .map_err(|error| error.to_string())?;
            let store = self.memory_service.store_for_agent(agent_id);
            let previous = store
                .list_memories(usize::MAX)
                .into_iter()
                .find(|record| record.id == memory_id);
            let Some(previous) = previous else {
                return Ok(false);
            };
            let mutation = Self::prepare_memory_mutation(
                &owner,
                "remove",
                store.snapshot_for_recovery(&[previous.clone()]),
                prior_snapshot,
            )?;
            let removed = match store.remove_memory(memory_id) {
                Ok(removed) => removed,
                Err(error) => {
                    Self::rollback_memory_mutation(&owner, &store, &mutation)
                        .map_err(|rollback| format!("memory remove failed: {error}; {rollback}"))?;
                    return Err(error.to_string());
                }
            };
            if !removed {
                Self::rollback_memory_mutation(&owner, &store, &mutation)?;
                return Ok(false);
            }
            match owner.commit_pending_memory_mutation(&mutation) {
                Ok(true) => Ok(true),
                Ok(false) => {
                    Self::rollback_memory_mutation(&owner, &store, &mutation)
                        .map_err(|rollback| format!("memory settlement was rejected; {rollback}"))?;
                    Err("memory settlement was rejected by the session owner".into())
                }
                Err(error) => Err(format!(
                    "memory settlement outcome unknown; journal retained for recovery: {error}"
                )),
            }
        })
    }

    pub fn clear_agent_memories(&self, agent_id: &str) -> Result<(), String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let prior_snapshot = owner
                .get_memory_prompt_snapshot()
                .map_err(|error| error.to_string())?;
            let store = self.memory_service.store_for_agent(agent_id);
            let previous = store.snapshot_for_recovery(&store.list_memories(usize::MAX));
            let mutation = Self::prepare_memory_mutation(
                &owner,
                "clear",
                previous,
                prior_snapshot,
            )?;
            if let Err(error) = store.clear_memories() {
                Self::rollback_memory_mutation(&owner, &store, &mutation)
                    .map_err(|rollback| format!("memory clear failed: {error}; {rollback}"))?;
                return Err(error.to_string());
            }
            match owner.commit_pending_memory_mutation(&mutation) {
                Ok(true) => Ok(()),
                Ok(false) => {
                    Self::rollback_memory_mutation(&owner, &store, &mutation)
                        .map_err(|rollback| format!("memory settlement was rejected; {rollback}"))?;
                    Err("memory settlement was rejected by the session owner".into())
                }
                Err(error) => Err(format!(
                    "memory settlement outcome unknown; journal retained for recovery: {error}"
                )),
            }
        })
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
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .set_agent_profile_prompt_snapshot(snapshot)
                .map_err(|error| error.to_string())
        })
    }

    pub fn clear_agent_profile_prompt_snapshot(&self, agent_id: &str) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .clear_agent_profile_prompt_snapshot()
                .map_err(|error| error.to_string())
        })
    }

    pub fn clear_agent_transient_state(&self, agent_id: &str) -> Result<bool, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            self.open_agent_db_owner_locked(agent_id)?
                .clear_transient_state()
                .map_err(|error| error.to_string())
        })
    }

    /// Commit a pure transcript plan against the existing owner's transaction.
    /// Keep the planner free of I/O, owner calls and pending-work mutations.
    /// Publication is deliberately outside the non-reentrant Session lock.
    pub(crate) fn commit_agent_transcript_entries<T>(
        &self,
        agent_id: &str,
        session_key: Option<&str>,
        expected_lifecycle_revision: Option<&str>,
        planner: impl FnOnce(&[serde_json::Value])
            -> Result<(T, Vec<serde_json::Value>), String>,
    ) -> Result<T, AgentDbProjectionError> {
        let committed = self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            Ok(owner.commit_transcript_entries(
                session_key, expected_lifecycle_revision, planner,
            ))
        }).map_err(AgentDbProjectionError::WorkAdmissionInvalid)??;
        Ok(committed.publish())
    }

    pub fn append_agent_transcript_entries(
        &self,
        agent_id: &str,
        entries: &[serde_json::Value],
    ) -> Result<usize, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .append_transcript_entries(entries)
                .map_err(|error| error.to_string())
        })
    }

    pub fn append_agent_transcript_entries_if_session_admitted(
        &self,
        agent_id: &str,
        session_key: &str,
        entries: &[serde_json::Value],
    ) -> Result<usize, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .append_transcript_entries_if_session_admitted(session_key, entries)
                .map_err(|error| error.to_string())
        })
    }

    /// Keep owner open, one-time preparation and append inside the existing
    /// non-reentrant filesystem/admission boundary. The DB owner preserves the
    /// attempt's original binding and returns the durable outcome directly.
    pub fn append_channel_inbound_entries(
        &self,
        agent_id: &str,
        attempt: &mut Option<ChannelInboundAppendAttempt>,
        prepare: impl FnOnce(
            &SessionWorkAdmissionRecord,
            &[serde_json::Value],
        ) -> Result<Vec<serde_json::Value>, ChannelInboundAppendFailure>,
    ) -> ChannelInboundAppendOutcome {
        let result = self.with_agent_filesystem_mutation(agent_id, || {
            if self.is_agent_being_deleted(agent_id) {
                return Ok((ChannelInboundAppendOutcome::NotWritten(ChannelInboundAppendFailure {
                    kind: ChannelInboundAppendFailureKind::Admission,
                    detail: "channel append rejected while agent deletion owns cleanup".into(),
                }), None));
            }
            let cached = self.db_owners.lock()
                .map_err(|_| "agent db owner map poisoned".to_string())?
                .get(&SessionDbOwnerKey::Agent(agent_id.to_string())).cloned();
            if cached.as_ref().is_some_and(|owner| owner.is_closed()) {
                return Ok((ChannelInboundAppendOutcome::NotWritten(ChannelInboundAppendFailure {
                    kind: ChannelInboundAppendFailureKind::Closed,
                    detail: "channel append owner is closed".into(),
                }), None));
            }
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            Ok(owner.append_channel_inbound_entries_deferred(attempt, prepare))
        });
        match result {
            Ok((outcome, publication)) => {
                if let Some(publication) = publication {
                    publication.publish();
                }
                outcome
            }
            Err(detail) => ChannelInboundAppendOutcome::NotWritten(ChannelInboundAppendFailure {
                kind: ChannelInboundAppendFailureKind::Storage,
                detail,
            }),
        }
    }

    /// Append room-owned transcript state while the caller already holds the
    /// per-agent filesystem/admission boundary. This keeps external producers
    /// from reacquiring the non-reentrant mutation lock.
    pub(crate) fn append_agent_transcript_entries_locked(
        &self,
        agent_id: &str,
        session_key: Option<&str>,
        entries: &[serde_json::Value],
    ) -> Result<usize, String> {
        self.assert_agent_transcript_write_admission_locked(agent_id)?;
        let owner = self.open_agent_db_owner_locked(agent_id)?;
        match session_key {
            Some(session_key) => owner
                .append_transcript_entries_if_session_admitted(session_key, entries)
                .map_err(|error| error.to_string()),
            None => owner
                .append_transcript_entries(entries)
                .map_err(|error| error.to_string()),
        }
    }

    pub fn append_agent_spend_guard_card_if_session_admitted(
        &self,
        agent_id: &str,
        session_key: &str,
        entry: &serde_json::Value,
        state: &SpendGuardState,
        activity_at: f64,
    ) -> Result<bool, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .append_spend_guard_card_if_session_admitted(
                    session_key,
                    entry,
                    state,
                    activity_at,
                )
                .map_err(|error| error.to_string())
        })
    }

    pub fn append_agent_transcript_entries_and_release_media(
        &self,
        agent_id: &str,
        session_key: &str,
        entries: &[serde_json::Value],
        expected_lifecycle_revision: &str,
        media_work_id: &str,
    ) -> Result<usize, AgentDbProjectionError> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            // The outer boundary can refuse before the operation. The inner
            // result must retain uncertain transaction outcomes for custody.
            Ok(owner.append_transcript_entries_and_release_media(
                    entries,
                    session_key,
                    expected_lifecycle_revision,
                    media_work_id,
                ))
        })
        .map_err(AgentDbProjectionError::WorkAdmissionInvalid)?
    }

    pub fn commit_agent_compaction(
        &self,
        agent_id: &str,
        commit: &CompactionCommit,
    ) -> Result<CompactionCommitOutcome, String> {
        self.commit_agent_compaction_with_cancellation(agent_id, commit, &|| false)
    }

    /// Cancellation is checked only after acquiring each canonical owner lock.
    /// The reader must not block, panic, or reenter Session/DB methods.
    pub fn commit_agent_compaction_with_cancellation(
        &self,
        agent_id: &str,
        commit: &CompactionCommit,
        should_cancel: &dyn Fn() -> bool,
    ) -> Result<CompactionCommitOutcome, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            if should_cancel() {
                return Ok(CompactionCommitOutcome::RejectedCancelled);
            }
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .commit_compaction_with_cancellation(commit, should_cancel)
                .map_err(|error| error.to_string())
        })
    }

    pub fn read_agent_compacted_transcript_entries(
        &self,
        agent_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        let mut entries = self
            .open_agent_db_owner(agent_id)?
            .get_compacted_transcript_entries()
            .map_err(|error| error.to_string())?;
        hide_session_work_receipts(&mut entries);
        Ok(entries)
    }

    /// Read the canonical compacted replay through the strict typed transcript
    /// boundary. Unsupported rows are rejected before any inference owner can
    /// serialize them into provider text.
    pub fn read_agent_typed_compacted_transcript_entries(
        &self,
        agent_id: &str,
    ) -> Result<Vec<TypedReplayEntry>, String> {
        let entries = self.read_agent_compacted_transcript_entries(agent_id)?;
        if !entries.iter().any(|entry| {
            entry.get("kind").and_then(serde_json::Value::as_str) == Some("compaction")
        }) {
            return Ok(Vec::new());
        }
        project_typed_replay_entries(&entries).map_err(|error| error.to_string())
    }

    pub fn update_agent_transcript_entry(
        &self,
        agent_id: &str,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .update_transcript_entry(entry_id, next)
                .map_err(|error| error.to_string())
        })
    }

    pub fn update_agent_transcript_entry_if_session_admitted(
        &self,
        agent_id: &str,
        session_key: &str,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .update_transcript_entry_if_session_admitted(session_key, entry_id, next)
                .map_err(|error| error.to_string())
        })
    }

    pub fn delete_agent_transcript_entry(
        &self,
        agent_id: &str,
        entry_id: &str,
    ) -> Result<bool, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            self.open_agent_db_owner_locked(agent_id)?
                .delete_transcript_entry(entry_id)
                .map_err(|error| error.to_string())
        })
    }

    pub fn clear_agent_conversation(&self, agent_id: &str) -> Result<bool, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_transcript_write_admission_locked(agent_id)?;
            // Validate and retain the existing owner before deleting the
            // canonical blob. Reopening it afterwards would reject the root
            // that this very clear operation has just removed.
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let store = self.create_agent_blob_store(agent_id)?;
            self.existing_session_db_path(agent_id)?;
            futures::executor::block_on(self.pool.clear_blobs(
                agent_id,
                &store.blob_db_path,
                store.legacy_blob_db_path.as_deref(),
            ))
            .map_err(|error| error.to_string())?;
            let cleared = owner.clear_conversation().map_err(|error| error.to_string())?;
            if cleared {
                self.close_agent_store_owner_locked(agent_id, false);
            }
            Ok(cleared)
        })
    }

    pub fn expire_pending_auto_review_approvals(
        &self,
        agent_id: &str,
        only_request_id: Option<&str>,
    ) -> Result<Vec<String>, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            expire_pending_auto_review_approval_entries(
                owner.as_ref(),
                only_request_id,
            )
            .map_err(|error| error.to_string())
        })
    }

    pub fn expire_pending_local_tool_permission_asks(
        &self,
        agent_id: &str,
        only_request_id: Option<&str>,
        if_pending_before_ms: Option<f64>,
    ) -> Result<Vec<String>, String> {
        self.with_agent_admitted_filesystem_mutation(agent_id, || {
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            expire_pending_local_tool_permission_ask_entries(
                owner.as_ref(),
                only_request_id,
                if_pending_before_ms,
            )
            .map_err(|error| error.to_string())
        })
    }

    pub fn get_agent_profile_text(
        &self,
        agent_id: &str,
    ) -> Result<Option<SandAgentProfile>, String> {
        let db_path = self.session_db_path(agent_id)?;
        if db_path.is_file() {
            let _owner = self.open_agent_db_owner(agent_id)?;
        }
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        Ok(read_agent_profile_text(agent_dir))
    }

    fn prepare_profile_mutation(
        owner: &SandAgentDb,
        operation: &str,
        before_files: Vec<ProfileFileSnapshot>,
        next_profile: Option<SandProfile>,
    ) -> Result<PendingProfileMutation, String> {
        let admission = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
            .ok_or_else(|| "profile mutation requires a session admission".to_string())?;
        let previous_profile = owner
            .get_sand_profile()
            .map_err(|error| error.to_string())?;
        let mutation = PendingProfileMutation {
            mutation_id: format!("profile-{}:{}", operation, Uuid::new_v4()),
            operation: operation.to_string(),
            session_key: admission.session_key,
            session_id: admission.session_id,
            lifecycle_revision: admission.lifecycle_revision,
            before_files,
            after_files: Vec::new(),
            previous_profile,
            next_profile,
        };
        if !owner
            .prepare_pending_profile_mutation(&mutation)
            .map_err(|error| error.to_string())?
        {
            return Err("profile mutation could not establish its durable journal".into());
        }
        Ok(mutation)
    }

    fn rollback_profile_mutation(
        owner: &SandAgentDb,
        agent_dir: &Path,
        mutation: &PendingProfileMutation,
    ) -> Result<(), String> {
        if !profile_mutation_files_match(agent_dir, &mutation.before_files) {
            if !profile_mutation_files_match(agent_dir, &mutation.after_files) {
                return Err("profile mutation rollback found an unknown file state".into());
            }
            restore_profile_mutation_files(agent_dir, &mutation.before_files)?;
        }
        if !owner
            .clear_pending_profile_mutation(mutation)
            .map_err(|error| error.to_string())?
        {
            return Err("profile mutation journal could not be cleared after rollback".into());
        }
        Ok(())
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
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let before_files = capture_profile_mutation_files(agent_dir, true)?;
            let mutation = Self::prepare_profile_mutation(
                &owner,
                "profile-update",
                before_files,
                None,
            )?;
            if let Err(error) = write_agent_profile_update(agent_dir, update) {
                Self::rollback_profile_mutation(&owner, agent_dir, &mutation)
                    .map_err(|rollback| format!("profile update failed: {error}; {rollback}"))?;
                return Err(error.to_string());
            }
            let mut sealed = mutation.clone();
            sealed.after_files = capture_profile_mutation_files(agent_dir, true)?;
            match owner.seal_pending_profile_mutation(&sealed) {
                Ok(true) => {}
                Ok(false) => {
                    Self::rollback_profile_mutation(&owner, agent_dir, &mutation)
                        .map_err(|rollback| format!("profile mutation seal rejected; {rollback}"))?;
                    return Err("profile mutation seal was rejected by the session owner".into());
                }
                Err(error) => {
                    return Err(format!(
                        "profile mutation seal outcome unknown; journal retained for recovery: {error}"
                    ));
                }
            }
            match owner.commit_pending_profile_mutation(&sealed) {
                Ok(true) => Ok(()),
                Ok(false) => {
                    Self::rollback_profile_mutation(&owner, agent_dir, &sealed)
                        .map_err(|rollback| format!("profile settlement was rejected; {rollback}"))?;
                    Err("profile settlement was rejected by the session owner".into())
                }
                Err(error) => Err(format!(
                    "profile settlement outcome unknown; journal retained for recovery: {error}"
                )),
            }
        })?;
        self.summarize_agent_by_id(agent_id, active_agent_id)
    }

    pub fn update_agent_profile_fields(
        &self,
        agent_id: &str,
        name: Option<&str>,
        description: Option<&str>,
    ) -> Result<(), String> {
        let current = self
            .get_agent_profile_text(agent_id)?
            .unwrap_or_else(|| SandAgentProfile {
                name: String::new(),
                description: String::new(),
                title: String::new(),
                avatar_shape: String::new(),
                avatar_color: String::new(),
            });
        let update = AgentProfileUpdate {
            name: name.unwrap_or(&current.name).to_string(),
            description: description.unwrap_or(&current.description).to_string(),
            title: Some(current.title),
            avatar_shape: Some(current.avatar_shape),
            avatar_color: Some(current.avatar_color),
        };
        self.update_agent_profile(agent_id, &update, None)?;
        Ok(())
    }

    pub fn get_agent_avatar(
        &self,
        agent_id: &str,
    ) -> Result<AgentAvatarResponse, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            let db_path = self.session_db_path(agent_id)?;
            if db_path.is_file() {
                let _owner = self.open_agent_db_owner_locked(agent_id)?;
            }
            let agent_dir = db_path
                .parent()
                .ok_or_else(|| "agent database has no parent directory".to_string())?;
            Ok(get_profile_avatar(
                agent_dir,
                &db_path,
                self.busy_timeout_ms,
            ))
        })
    }

    pub fn get_agent_avatar_png(
        &self,
        agent_id: &str,
    ) -> Result<Option<Vec<u8>>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            let db_path = self.session_db_path(agent_id)?;
            if db_path.is_file() {
                let _owner = self.open_agent_db_owner_locked(agent_id)?;
            }
            let agent_dir = db_path
                .parent()
                .ok_or_else(|| "agent database has no parent directory".to_string())?;
            Ok(get_profile_avatar_png(
                agent_dir,
                &db_path,
                self.busy_timeout_ms,
            ))
        })
    }

    pub fn set_agent_avatar_bytes(
        &self,
        agent_id: &str,
        png_bytes: Option<&[u8]>,
        active_agent_id: Option<&str>,
    ) -> Result<Option<AgentSummary>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        let agent_dir = db_path
            .parent()
            .ok_or_else(|| "agent database has no parent directory".to_string())?;
        self.with_agent_filesystem_mutation(agent_id, || {
            self.assert_agent_work_admission_locked(agent_id)?;
            let owner = self.open_agent_db_owner_locked(agent_id)?;
            let before_files = capture_profile_mutation_files(agent_dir, false)?;
            let previous_profile = owner
                .get_sand_profile()
                .map_err(|error| error.to_string())?;
            let next_profile = png_bytes.is_none().then(|| SandProfile {
                description: previous_profile.description.clone(),
                avatar_path: None,
            });
            let mutation = Self::prepare_profile_mutation(
                &owner,
                if png_bytes.is_some() {
                    "avatar-replace"
                } else {
                    "avatar-remove"
                },
                before_files,
                next_profile,
            )?;
            if let Err(error) = mutate_agent_avatar_bytes(
                &db_path,
                self.busy_timeout_ms,
                png_bytes,
                |_| Ok(()),
            ) {
                Self::rollback_profile_mutation(&owner, agent_dir, &mutation)
                    .map_err(|rollback| format!("avatar mutation failed: {error}; {rollback}"))?;
                return Err(error);
            }
            let mut sealed = mutation.clone();
            sealed.after_files = capture_profile_mutation_files(agent_dir, false)?;
            match owner.seal_pending_profile_mutation(&sealed) {
                Ok(true) => {}
                Ok(false) => {
                    Self::rollback_profile_mutation(&owner, agent_dir, &mutation)
                        .map_err(|rollback| format!("avatar mutation seal rejected; {rollback}"))?;
                    return Err("avatar mutation seal was rejected by the session owner".into());
                }
                Err(error) => {
                    return Err(format!(
                        "avatar mutation seal outcome unknown; journal retained for recovery: {error}"
                    ));
                }
            }
            match owner.commit_pending_profile_mutation(&sealed) {
                Ok(true) => Ok(()),
                Ok(false) => {
                    Self::rollback_profile_mutation(&owner, agent_dir, &sealed)
                        .map_err(|rollback| format!("avatar settlement was rejected; {rollback}"))?;
                    Err("avatar settlement was rejected by the session owner".into())
                }
                Err(error) => Err(format!(
                    "avatar settlement outcome unknown; journal retained for recovery: {error}"
                )),
            }
        })?;
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

    fn materialize_remote_human_attachment(
        &self,
        attachment: &FabushiRemoteHumanAttachment,
    ) -> Result<serde_json::Value, String> {
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
            let bytes = client.download_direct_message_resource(resource_id, attachment.size)?;
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

    fn materialize_remote_human_message(
        &self,
        owner: &SandAgentDb,
        remote: &FabushiRemoteHumanMessage,
    ) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?;
        let remote_id = fabushi_identity_text(&remote.id)?;
        let sender_id = fabushi_identity_text(&remote.sender_user_id)?;
        let recipient_id = fabushi_identity_text(&remote.recipient_user_id)?;
        if sender_id != local_human_id && recipient_id != local_human_id {
            return Err("remote Human message does not belong to the local Fabushi account".into());
        }
        let timestamp_ms = chrono::DateTime::parse_from_rfc3339(&remote.created_at)
            .map_err(|error| format!("remote Human message timestamp is invalid: {error}"))?
            .timestamp_millis() as f64;
        let entry_id = format!("human-server-message:{remote_id}");
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
            object.insert("clientNonce".into(), serde_json::json!(client_nonce));
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
        let mut projected_reactions = Vec::new();
        for reaction in &remote.reactions {
            if reaction.reacted_by_me {
                projected_reactions.push(serde_json::json!({
                    "emoji": reaction.emoji,
                    "by": "me",
                }));
            }
            let peer_reaction_count = reaction
                .count
                .saturating_sub(if reaction.reacted_by_me { 1 } else { 0 });
            for _ in 0..peer_reaction_count {
                projected_reactions.push(serde_json::json!({
                    "emoji": reaction.emoji,
                    "by": peer_human_id,
                }));
            }
        }
        object.insert(
            "remoteReactions".into(),
            serde_json::to_value(&remote.reactions).map_err(|error| {
                format!("remote Human reactions could not be projected: {error}")
            })?,
        );
        object.insert(
            "reactions".into(),
            serde_json::Value::Array(projected_reactions),
        );
        let admitted_session_key = owner
            .read_session_work_admission()
            .map_err(|error| error.to_string())?
            .map(|record| record.session_key)
            .filter(|value| !value.trim().is_empty());
        let existing = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        if let Some(current) = existing.iter().find(|candidate| {
            candidate.get("id").and_then(serde_json::Value::as_str)
                == entry.get("id").and_then(serde_json::Value::as_str)
        }) {
            if let Some(context) = current.get("forwardContext").cloned() {
                entry
                    .as_object_mut()
                    .ok_or_else(|| "remote Human message projection failed".to_string())?
                    .insert("forwardContext".into(), context);
            }
            if current == &entry {
                return Ok(current.clone());
            }
            let entry_id = entry
                .get("id")
                .and_then(serde_json::Value::as_str)
                .map(ToOwned::to_owned)
                .ok_or_else(|| "remote Human message projection omitted its id".to_string())?;
            let updated = if let Some(session_key) = admitted_session_key.as_deref() {
                entry["sessionKey"] = serde_json::Value::String(session_key.to_string());
                owner
                    .update_transcript_entry_if_session_admitted(
                        session_key,
                        &entry_id,
                        &entry,
                    )
                    .map_err(|error| error.to_string())?
            } else {
                owner
                    .update_transcript_entry(&entry_id, &entry)
                    .map_err(|error| error.to_string())?
            };
            return updated.ok_or_else(|| "remote Human message could not be durably refreshed".to_string());
        }
        if let Some(client_nonce) = entry.get("clientNonce").and_then(serde_json::Value::as_str) {
            if let Some(current) = existing.iter().find(|candidate| {
                candidate.get("clientNonce").and_then(serde_json::Value::as_str) == Some(client_nonce)
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
                let settled = if let Some(session_key) = admitted_session_key.as_deref() {
                    entry["sessionKey"] = serde_json::Value::String(session_key.to_string());
                    owner
                        .settle_transcript_entry_identity_if_session_admitted(
                            session_key,
                            current_id,
                            &entry,
                        )
                        .map_err(|error| error.to_string())?
                } else {
                    owner
                        .settle_transcript_entry_identity(current_id, &entry)
                        .map_err(|error| error.to_string())?
                };
                return settled.ok_or_else(|| {
                    "local Human message disappeared before remote identity settlement".to_string()
                });
            }
        }
        if let Some(session_key) = admitted_session_key {
            entry["sessionKey"] = serde_json::Value::String(session_key.clone());
            if owner
                .append_transcript_entries_if_session_admitted(&session_key, &[entry.clone()])
                .map_err(|error| error.to_string())?
                == 0
            {
                let replay = owner
                    .get_transcript_entries()
                    .map_err(|error| error.to_string())?
                    .into_iter()
                    .find(|candidate| candidate.get("id") == entry.get("id"))
                    .ok_or_else(|| "remote Human message was not durably materialized".to_string())?;
                return Ok(replay);
            }
        } else if !owner
            .append_transcript_entry(&entry)
            .map_err(|error| error.to_string())?
        {
            let replay = owner
                .get_transcript_entries()
                .map_err(|error| error.to_string())?
                .into_iter()
                .find(|candidate| candidate.get("id") == entry.get("id"))
                .ok_or_else(|| "remote Human message was not durably materialized".to_string())?;
            return Ok(replay);
        }
        if !owner
            .set_metadata("lastActivityAt", serde_json::json!(timestamp_ms))
            .map_err(|error| error.to_string())?
        {
            return Err("remote Human message activity metadata was not durably updated".into());
        }
        Ok(entry)
    }

    pub fn sync_human_conversation(
        &self,
        conversation_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        const PAGE_LIMIT: usize = 200;
        const MAX_GAP_PAGES: usize = 32;

        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        let Some(client) = self.shipping_native_messaging()? else {
            return owner.get_transcript_entries().map_err(|error| error.to_string());
        };
        let local_human_id = self.local_human_id()?.to_string();
        let pending = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?
            .into_iter()
            .filter(|entry| {
                entry.get("authorId").and_then(serde_json::Value::as_str) == Some(local_human_id.as_str())
                    && entry.get("remoteMessageId").is_none()
                    && matches!(
                        entry.get("delivery").and_then(serde_json::Value::as_str),
                        Some("pending" | "dispatching")
                    )
                    && entry.get("clientNonce").and_then(serde_json::Value::as_str).is_some()
            })
            .collect::<Vec<_>>();
        for entry in pending {
            let Some(client_nonce) = entry.get("clientNonce").and_then(serde_json::Value::as_str) else { continue; };
            let text = entry.get("content").and_then(serde_json::Value::as_str).unwrap_or_default();
            let reply_to_id = entry.get("replyToId").and_then(serde_json::Value::as_str);
            let composed_at_ms = entry.get("composedAtMs").and_then(serde_json::Value::as_f64);
            let attachments = entry.get("attachments").and_then(serde_json::Value::as_array).cloned().unwrap_or_default();
            let _ = self.append_human_message(conversation_id, text, client_nonce, composed_at_ms, reply_to_id, &attachments);
        }
        let peer_human_id = self.peer_human_id_from_metadata(&metadata)?;
        let existing = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        let known_remote_ids = existing
            .iter()
            .filter_map(|entry| entry.get("remoteMessageId").and_then(serde_json::Value::as_str))
            .map(str::to_string)
            .collect::<BTreeSet<_>>();

        // The shipping endpoint selects newest rows but returns each page
        // oldest-first. Walk bounded older pages until one known server message
        // proves overlap, or the server reaches the end of history. This closes
        // reconnect and fresh-device gaps without introducing a second message
        // store or treating renderer state as truth.
        let mut before: Option<String> = None;
        let mut unseen_pages: Vec<Vec<FabushiRemoteHumanMessage>> = Vec::new();
        let mut pages_read = 0usize;
        loop {
            let page = client.list_direct_messages(
                &peer_human_id,
                before.as_deref(),
                PAGE_LIMIT,
            )?;
            if page.is_empty() {
                break;
            }
            pages_read += 1;

            let mut overlap_at = None;
            for (index, remote) in page.iter().enumerate().rev() {
                let remote_id = fabushi_identity_text(&remote.id)?;
                if known_remote_ids.contains(&remote_id) {
                    overlap_at = Some(index);
                    break;
                }
            }
            let unseen_start = overlap_at.map_or(0, |index| index + 1);
            if unseen_start < page.len() {
                unseen_pages.push(page[unseen_start..].to_vec());
            }

            if overlap_at.is_some() || page.len() < PAGE_LIMIT {
                break;
            }
            if pages_read >= MAX_GAP_PAGES {
                return Err(format!(
                    "Fabushi Human message reconnect gap exceeded the bounded recovery window of {} messages",
                    PAGE_LIMIT * MAX_GAP_PAGES
                ));
            }

            let next_before = page
                .first()
                .map(|message| message.created_at.trim())
                .filter(|value| !value.is_empty())
                .ok_or_else(|| {
                    "Fabushi Human message sync page omitted its oldest timestamp".to_string()
                })?
                .to_string();
            if before.as_deref() == Some(next_before.as_str()) {
                return Err(
                    "Fabushi Human message sync cursor did not advance; refusing an unbounded retry"
                        .into(),
                );
            }
            before = Some(next_before);
        }

        // Page windows arrive newest-window first while rows inside each page are
        // oldest-first. Reverse only the page windows so canonical
        // Session/Transcript replay remains deterministic after restart.
        for page in unseen_pages.into_iter().rev() {
            for remote in page {
                self.materialize_remote_human_message(&owner, &remote)?;
            }
        }
        owner.get_transcript_entries().map_err(|error| error.to_string())
    }

    pub fn sync_human_conversations(&self) -> Result<Vec<serde_json::Value>, String> {
        let Some(client) = self.shipping_native_messaging()? else {
            return self.list_human_conversations();
        };
        let identity = client.identity()?;
        if identity.user_id != self.local_human_id()? {
            return Err(format!(
                "Fabushi native messaging credential user {} does not match active Human identity {}",
                identity.user_id,
                self.local_human_id()?
            ));
        }
        for friend in client.list_friends()? {
            let peer_human_id = fabushi_identity_text(&friend.user_id)?;
            let title = friend.display_name.trim();
            let conversation = self.create_human_conversation(
                &peer_human_id,
                if title.is_empty() { &friend.username } else { title },
            )?;
            let conversation_id = conversation
                .get("id")
                .and_then(serde_json::Value::as_str)
                .ok_or_else(|| "synced Human conversation omitted its id".to_string())?;
            self.sync_human_conversation(conversation_id)?;
        }
        self.sync_human_calls()?;
        self.list_human_conversations()
    }

    pub fn create_human_conversation(
        &self,
        peer_human_id: &str,
        title: &str,
    ) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?;
        let peer_human_id = peer_human_id.trim();
        let title = title.trim();
        if peer_human_id.is_empty() {
            return Err("human conversation requires peerHumanId".into());
        }
        if local_human_id == peer_human_id {
            return Err("human conversation participants must be distinct".into());
        }
        let mut participant_ids = [local_human_id.to_string(), peer_human_id.to_string()];
        participant_ids.sort();
        let mut digest = Sha256::new();
        for participant_id in &participant_ids {
            digest.update((participant_id.len() as u64).to_be_bytes());
            digest.update(participant_id.as_bytes());
        }
        let conversation_id = format!("human-direct-{:x}", digest.finalize());
        let db_path = get_native_conversation_db_path(&self.agents_root, &conversation_id)
            .map_err(|error| error.to_string())?;
        if db_path.is_file() {
            let owner = self.open_human_conversation_db_owner(&conversation_id)?;
            let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
            if !self.metadata_has_local_human(&metadata)? {
                return Err("existing Human conversation does not contain the local identity".into());
            }
            let existing_participants = metadata
                .get("participantIds")
                .and_then(serde_json::Value::as_array)
                .ok_or_else(|| "existing Human conversation participants are invalid".to_string())?;
            if !existing_participants
                .iter()
                .any(|value| value.as_str() == Some(peer_human_id))
            {
                return Err("existing Human conversation does not contain the requested peer".into());
            }
            let existing_title = metadata
                .get("title")
                .and_then(serde_json::Value::as_str)
                .unwrap_or(peer_human_id);
            return Ok(serde_json::json!({
                "id": conversation_id,
                "kind": "human",
                "title": existing_title,
                "participantIds": metadata.get("participantIds").cloned().unwrap_or_else(|| serde_json::json!([])),
            }));
        }
        let conversation_dir = db_path
            .parent()
            .ok_or_else(|| "native conversation database has no parent".to_string())?;
        fs::create_dir_all(conversation_dir).map_err(|error| error.to_string())?;
        let owner = Arc::new(
            SandAgentDb::open_conversation_store(&db_path, self.busy_timeout_ms)
                .map_err(|error| error.to_string())?,
        );
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
        if !owner
            .initialize_metadata_if_missing(metadata)
            .map_err(|error| error.to_string())?
        {
            owner.close(false);
            let _ = fs::remove_dir_all(conversation_dir);
            return Err("failed to initialize native Human conversation metadata".into());
        }
        self.db_owners
            .lock()
            .map_err(|_| "conversation db owner map poisoned".to_string())?
            .insert(SessionDbOwnerKey::Human(conversation_id.clone()), Arc::clone(&owner));
        Ok(serde_json::json!({
            "id": conversation_id,
            "kind": "human",
            "title": display_title,
            "participantIds": [local_human_id, peer_human_id],
        }))
    }

    pub fn open_human_conversation_db_owner(
        &self,
        conversation_id: &str,
    ) -> Result<Arc<SandAgentDb>, String> {
        let db_path = get_native_conversation_db_path(&self.agents_root, conversation_id)
            .map_err(|error| error.to_string())?;
        if !db_path.is_file() {
            return Err(format!("Human conversation missing: {conversation_id}"));
        }
        let key = SessionDbOwnerKey::Human(conversation_id.to_string());
        let mut owners = self
            .db_owners
            .lock()
            .map_err(|_| "conversation db owner map poisoned".to_string())?;
        if let Some(owner) = owners.get(&key) {
            return Ok(Arc::clone(owner));
        }
        let owner = Arc::new(
            SandAgentDb::open_conversation_store(&db_path, self.busy_timeout_ms)
                .map_err(|error| error.to_string())?,
        );
        if owner
            .get_metadata("conversationKind")
            .map_err(|error| error.to_string())?
            .and_then(|value| value.as_str().map(ToOwned::to_owned))
            .as_deref()
            != Some("human")
        {
            owner.close(false);
            return Err(format!("Conversation is not a Human conversation: {conversation_id}"));
        }
        owners.insert(key, Arc::clone(&owner));
        Ok(owner)
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
            conversations.push(serde_json::json!({
                "id": id,
                "kind": "human",
                "title": title,
                "participantIds": participant_ids,
                "updatedAt": updated_at,
            }));
        }
        conversations.sort_by(|left, right| {
            left.get("id")
                .and_then(serde_json::Value::as_str)
                .cmp(&right.get("id").and_then(serde_json::Value::as_str))
        });
        Ok(conversations)
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
        let requirements = fabushi_messaging_core::RecipientSearchRequirements {
            require_media: source_entry
                .get("attachments")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|attachments| !attachments.is_empty()),
            source_protected_content: source_entry
                .get("protectedContent")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            ..fabushi_messaging_core::RecipientSearchRequirements::default()
        };
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
                if !fabushi_messaging_core::recipient_search_authorized(
                    fabushi_messaging_core::RecipientAuthorizationInput::direct_default(
                        sender_is_participant,
                    ),
                    requirements,
                ) {
                    return false;
                }
                if normalized_query.is_empty() {
                    return true;
                }
                let title_matches = conversation
                    .get("title")
                    .and_then(serde_json::Value::as_str)
                    .is_some_and(|title| title.to_lowercase().contains(&normalized_query));
                let participant_matches = conversation
                    .get("participantIds")
                    .and_then(serde_json::Value::as_array)
                    .is_some_and(|ids| {
                        ids.iter().any(|id| {
                            id.as_str()
                                .is_some_and(|id| id.to_lowercase().contains(&normalized_query))
                        })
                    });
                title_matches || participant_matches
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
        if destination_conversation_ids.is_empty() {
            return Err("forwardHumanMessage requires at least one destination".into());
        }

        let mut seen_destinations = BTreeSet::new();
        let mut destinations = Vec::new();
        for destination in destination_conversation_ids {
            let destination = destination.trim();
            if destination.is_empty() {
                return Err("forwardHumanMessage destination id must not be empty".into());
            }
            if seen_destinations.insert(destination.to_string()) {
                destinations.push(destination.to_string());
            }
        }
        if destinations.len() > 32 {
            return Err("forwardHumanMessage supports at most 32 unique destinations".into());
        }

        let source_owner = self.open_human_conversation_db_owner(source_conversation_id)?;
        let source_metadata = source_owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&source_metadata)? {
            return Err("local Human identity is not a participant in the source conversation".into());
        }
        let source_entry_id = source_entry_id.trim();
        let source_entry = source_owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?
            .into_iter()
            .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(source_entry_id))
            .ok_or_else(|| "forwardHumanMessage source message is missing".to_string())?;
        if source_entry.get("kind").and_then(serde_json::Value::as_str) != Some("message")
            || source_entry.get("delivery").and_then(serde_json::Value::as_str) != Some("sent")
        {
            return Err("forwardHumanMessage source must be a settled canonical message".into());
        }
        if source_entry
            .get("protectedContent")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
        {
            return Err("forwardHumanMessage source is protected and cannot be forwarded".into());
        }

        let source_text = source_entry
            .get("content")
            .and_then(serde_json::Value::as_str)
            .unwrap_or_default();
        let attachments = source_entry
            .get("attachments")
            .and_then(serde_json::Value::as_array)
            .cloned()
            .unwrap_or_default();
        if source_text.trim().is_empty() && attachments.is_empty() {
            return Err("forwardHumanMessage source has no forwardable content".into());
        }

        let privacy = fabushi_messaging_core::ForwardPrivacy {
            drop_sender_names,
            drop_captions,
        }
        .normalized();
        let drop_sender_names = privacy.drop_sender_names;
        let drop_captions = privacy.drop_captions;
        let forwarded_text = if drop_captions && !attachments.is_empty() {
            ""
        } else {
            source_text
        };
        let mut forward_context = serde_json::json!({
            "requestId": client_nonce,
            "sourceConversationId": source_conversation_id,
            "sourceEntryId": source_entry_id,
            "privacy": {
                "dropSenderNames": drop_sender_names,
                "dropCaptions": drop_captions,
            },
        });
        if !drop_sender_names {
            let mut origin = serde_json::Map::new();
            if let Some(author_id) = source_entry.get("authorId").cloned() {
                origin.insert("authorId".into(), author_id);
            }
            if let Some(author_name) = source_entry.get("authorName").cloned() {
                origin.insert("authorName".into(), author_name);
            }
            if !origin.is_empty() {
                forward_context
                    .as_object_mut()
                    .expect("forward context is an object")
                    .insert("origin".into(), serde_json::Value::Object(origin));
            }
        }

        let mut settlement = Vec::with_capacity(destinations.len());
        for destination_conversation_id in destinations {
            let mut digest = Sha256::new();
            digest.update(client_nonce.as_bytes());
            digest.update([0]);
            digest.update(destination_conversation_id.as_bytes());
            let destination_nonce = format!("human-forward-{:x}", digest.finalize());

            match self.append_human_message_with_context(
                &destination_conversation_id,
                forwarded_text,
                &destination_nonce,
                None,
                None,
                &attachments,
                Some(&forward_context),
            ) {
                Ok(entry) => settlement.push(serde_json::json!({
                    "conversationId": destination_conversation_id,
                    "clientNonce": destination_nonce,
                    "status": "sent",
                    "entry": entry,
                })),
                Err(error) => settlement.push(serde_json::json!({
                    "conversationId": destination_conversation_id,
                    "clientNonce": destination_nonce,
                    "status": "failed",
                    "error": error,
                })),
            }
        }

        Ok(serde_json::json!({
            "requestId": client_nonce,
            "privacy": {
                "dropSenderNames": drop_sender_names,
                "dropCaptions": drop_captions,
            },
            "destinations": settlement,
        }))
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
        self.append_human_message_with_context(
            conversation_id,
            text,
            client_nonce,
            composed_at_ms,
            reply_to_id,
            attachments,
            None,
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
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        let is_participant = metadata
            .get("participantIds")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|ids| ids.iter().any(|id| id.as_str() == Some(sender_id)));
        if !is_participant {
            return Err("local Human identity is not a conversation participant".into());
        }
        let existing = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        let normalized_reply = reply_to_id
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(reply_id) = normalized_reply {
            if !existing.iter().any(|entry| {
                entry.get("id").and_then(serde_json::Value::as_str) == Some(reply_id)
            }) {
                return Err("sendHumanMessage reply target is not in this Human conversation".into());
            }
        }
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
                Ok(serde_json::json!({ "path": path, "name": name }))
            })
            .collect::<Result<Vec<_>, String>>()?;
        let timestamp_ms = composed_at_ms.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        });
        let mut existing_pending: Option<serde_json::Value> = None;
        if let Some(entry) = existing.iter().find(|entry| {
            entry.get("clientNonce").and_then(serde_json::Value::as_str) == Some(client_nonce)
        }) {
            let same_sender = entry.get("authorId").and_then(serde_json::Value::as_str) == Some(sender_id);
            let same_text = entry.get("content").and_then(serde_json::Value::as_str) == Some(text);
            let same_reply = entry.get("replyToId").and_then(serde_json::Value::as_str) == normalized_reply;
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
                return Ok(entry.clone());
            }
            if !matches!(
                entry.get("delivery").and_then(serde_json::Value::as_str),
                Some("pending" | "dispatching")
            ) {
                return Err("sendHumanMessage clientNonce is not in a retryable settlement state".into());
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
                if let Some(reply_id) = normalized_reply { object.insert("replyToId".into(), serde_json::json!(reply_id)); }
                if !normalized_attachments.is_empty() { object.insert("attachments".into(), serde_json::Value::Array(normalized_attachments.clone())); }
                if let Some(context) = forward_context { object.insert("forwardContext".into(), context.clone()); }
                entry
            });
            if !existing.iter().any(|entry| entry.get("id") == pending_entry.get("id"))
                && !owner.append_transcript_entry(&pending_entry).map_err(|error| error.to_string())?
            {
                return Err("Human pending send could not be durably recorded before dispatch".into());
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
                    for attachment in &normalized_attachments {
                        let path = attachment.get("path").and_then(serde_json::Value::as_str)
                            .ok_or_else(|| "sendHumanMessage attachment requires path".to_string())?;
                        let name = attachment.get("name").and_then(serde_json::Value::as_str)
                            .ok_or_else(|| "sendHumanMessage attachment requires name".to_string())?;
                        uploaded.push(client.upload_direct_message_resource(std::path::Path::new(path), name)?);
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

                let remote = client.send_direct_message(&peer_human_id, text, client_nonce, remote_reply_to_id.as_deref(), &uploaded)?;
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
                self.materialize_remote_human_message(&owner, &remote)
            })();

            if let Err(error) = dispatch_result.as_ref() {
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
                return Ok(replay);
            }
            return Err("sendHumanMessage clientNonce already identifies different content".into());
        }
        if !owner
            .set_metadata("lastActivityAt", serde_json::json!(timestamp_ms))
            .map_err(|error| error.to_string())?
        {
            return Err("native Human message activity metadata was not durably updated".into());
        }
        Ok(entry)
    }

    pub fn append_human_agent_message(
        &self,
        conversation_id: &str,
        agent_id: &str,
        stream_id: &str,
        session_key: Option<&str>,
        content: &str,
        completed_at_ms: f64,
    ) -> Result<serde_json::Value, String> {
        let conversation_id = conversation_id.trim();
        let agent_id = agent_id.trim();
        let stream_id = stream_id.trim();
        let session_key = match session_key {
            Some(value) => {
                let value = value.trim();
                if value.is_empty() {
                    return Err("Human handoff Agent result session key must be non-empty".into());
                }
                Some(value)
            }
            None => None,
        };
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

        let append = || {
            self.append_human_agent_message_to_conversation(
                conversation_id,
                agent_id,
                stream_id,
                session_key,
                content,
                completed_at_ms,
            )
        };
        if let Some(session_key) = session_key {
            // The session key belongs to the source Agent. Keep its lifecycle
            // boundary through the destination commit; the Human transcript
            // must never acquire the Agent's admission record.
            self.with_agent_filesystem_mutation(agent_id, || {
                self.assert_agent_transcript_write_admission_locked(agent_id)?;
                let admission = self
                    .read_session_work_admission_locked(agent_id)?
                    .ok_or_else(|| {
                        "Human handoff Agent result requires source session admission"
                            .to_string()
                    })?;
                if admission.session_key != session_key {
                    return Err("Human handoff Agent result source session key mismatch".into());
                }
                append()
            })
        } else {
            append()
        }
    }

    fn append_human_agent_message_to_conversation(
        &self,
        conversation_id: &str,
        agent_id: &str,
        stream_id: &str,
        session_key: Option<&str>,
        content: &str,
        completed_at_ms: f64,
    ) -> Result<serde_json::Value, String> {
        let local_human_id = self.local_human_id()?;
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
                if let Some(expected_session_key) = session_key {
                    if let Some(actual_session_key) = entry
                        .get("sessionKey")
                        .and_then(serde_json::Value::as_str)
                    {
                        if actual_session_key != expected_session_key {
                            return Err(
                                "Human handoff stream already identifies a different session key"
                                    .into(),
                            );
                        }
                    } else {
                        let mut enriched = entry.clone();
                        enriched["sessionKey"] = serde_json::Value::String(
                            expected_session_key.to_string(),
                        );
                        return owner
                            .update_transcript_entry(&entry_id, &enriched)
                            .map_err(|error| error.to_string())?
                            .ok_or_else(|| {
                                "Human handoff Agent result disappeared during provenance settlement"
                                    .to_string()
                            });
                    }
                }
                return Ok(entry.clone());
            }
            return Err("Human handoff stream already identifies different Agent content".into());
        }

        let mut entry = serde_json::json!({
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
        if let Some(session_key) = session_key {
            entry["sessionKey"] = serde_json::Value::String(session_key.to_string());
        }
        let inserted = owner
            .append_transcript_entry(&entry)
            .map_err(|error| error.to_string())?;
        if !inserted {
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
                if let Some(expected_session_key) = session_key {
                    if let Some(actual_session_key) = replay
                        .get("sessionKey")
                        .and_then(serde_json::Value::as_str)
                    {
                        if actual_session_key != expected_session_key {
                            return Err(
                                "Human handoff stream already identifies a different session key"
                                    .into(),
                            );
                        }
                    } else {
                        let mut enriched = replay.clone();
                        enriched["sessionKey"] = serde_json::Value::String(
                            expected_session_key.to_string(),
                        );
                        return owner
                            .update_transcript_entry(&entry_id, &enriched)
                            .map_err(|error| error.to_string())?
                            .ok_or_else(|| {
                                "Human handoff Agent result disappeared during replay provenance settlement"
                                    .to_string()
                            });
                    }
                }
                return Ok(replay);
            }
            return Err("Human handoff stream already identifies different Agent content".into());
        }
        if !owner
            .set_metadata("lastActivityAt", serde_json::json!(completed_at_ms))
            .map_err(|error| error.to_string())?
        {
            return Err("Human handoff Agent activity metadata was not durably updated".into());
        }
        Ok(entry)
    }

    pub fn toggle_human_message_reaction(
        &self,
        conversation_id: &str,
        entry_id: &str,
        emoji: &str,
    ) -> Result<serde_json::Value, String> {
        let emoji = emoji.trim();
        if emoji.is_empty() {
            return Err("reactHumanMessage requires emoji".into());
        }
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        let entries = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        let current = entries
            .iter()
            .find(|entry| entry.get("id").and_then(serde_json::Value::as_str) == Some(entry_id))
            .cloned()
            .ok_or_else(|| "Human reaction target is not in this conversation".to_string())?;
        let remote_message_id = current
            .get("remoteMessageId")
            .map(fabushi_identity_text)
            .transpose()?
            .ok_or_else(|| "Human reaction target has no canonical remote message identity".to_string())?;
        let currently_active = current
            .get("remoteReactions")
            .and_then(serde_json::Value::as_array)
            .is_some_and(|reactions| {
                reactions.iter().any(|reaction| {
                    reaction.get("emoji").and_then(serde_json::Value::as_str) == Some(emoji)
                        && reaction
                            .get("reactedByMe")
                            .and_then(serde_json::Value::as_bool)
                            == Some(true)
                })
            });
        let client = self
            .shipping_native_messaging()?
            .ok_or_else(|| "Fabushi Human reactions require the shipping messaging backend".to_string())?;
        let remote_reactions =
            client.set_direct_message_reaction(&remote_message_id, emoji, !currently_active)?;
        let peer_human_id = self.peer_human_id_from_metadata(&metadata)?;
        let mut projected_reactions = Vec::new();
        for reaction in &remote_reactions {
            if reaction.reacted_by_me {
                projected_reactions.push(serde_json::json!({
                    "emoji": reaction.emoji,
                    "by": "me",
                }));
            }
            let peer_reaction_count = reaction
                .count
                .saturating_sub(if reaction.reacted_by_me { 1 } else { 0 });
            for _ in 0..peer_reaction_count {
                projected_reactions.push(serde_json::json!({
                    "emoji": reaction.emoji,
                    "by": peer_human_id,
                }));
            }
        }
        let mut next = current;
        let object = next
            .as_object_mut()
            .ok_or_else(|| "Human reaction target is not a message object".to_string())?;
        object.insert(
            "remoteReactions".into(),
            serde_json::to_value(&remote_reactions).map_err(|error| {
                format!("remote Human reactions could not be projected: {error}")
            })?,
        );
        object.insert(
            "reactions".into(),
            serde_json::Value::Array(projected_reactions),
        );
        let updated = if let Some(session_key) = next
            .get("sessionKey")
            .and_then(serde_json::Value::as_str)
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
            owner
                .update_transcript_entry_if_session_admitted(session_key, entry_id, &next)
                .map_err(|error| error.to_string())?
        } else {
            owner
                .update_transcript_entry(entry_id, &next)
                .map_err(|error| error.to_string())?
        };
        updated.ok_or_else(|| "Human reaction target disappeared before durable settlement".to_string())
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
            let content_match = entry
                .get("content")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.to_lowercase().contains(&query));
            let author_match = entry
                .get("authorName")
                .and_then(serde_json::Value::as_str)
                .is_some_and(|value| value.to_lowercase().contains(&query));
            let attachment_match = entry
                .get("attachments")
                .and_then(serde_json::Value::as_array)
                .is_some_and(|attachments| {
                    attachments.iter().any(|attachment| {
                        attachment
                            .get("name")
                            .and_then(serde_json::Value::as_str)
                            .is_some_and(|value| value.to_lowercase().contains(&query))
                    })
                });
            if content_match || author_match || attachment_match {
                matches.push(entry);
                if matches.len() >= limit {
                    break;
                }
            }
        }
        Ok(matches)
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
            .get_transcript_entries()
            .map_err(|error| error.to_string())
    }

    pub fn read_human_conversation_transcript_tail(
        &self,
        conversation_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, String> {
        let owner = self.open_human_conversation_db_owner(conversation_id)?;
        let metadata = owner.read_metadata().map_err(|error| error.to_string())?;
        if !self.metadata_has_local_human(&metadata)? {
            return Err("local Human identity is not a conversation participant".into());
        }
        owner
            .get_transcript_tail(query)
            .map_err(|error| error.to_string())
    }

    pub fn list_agent_summaries(
        &self,
        active_agent_id: Option<&str>,
    ) -> Result<Vec<AgentSummary>, String> {
        let is_deleting = |agent_id: &str| self.is_agent_being_deleted(agent_id);
        let open_agent_owner = |agent_id: &str| self.open_agent_db_owner_locked(agent_id);
        let with_agent_mutation: &dyn for<'a> Fn(
            &'a str,
            Box<dyn FnOnce() -> Result<Option<AgentSummary>, String> + 'a>,
        ) -> Result<Option<AgentSummary>, String> = &|agent_id, operation| {
            self.with_agent_filesystem_mutation(agent_id, operation)
        };
        list_agents(
            &self.agents_root,
            self.busy_timeout_ms,
            active_agent_id,
            &is_deleting,
            &self.roster_extras_cache,
            &open_agent_owner,
            with_agent_mutation,
        )
    }

    pub fn summarize_agent_by_id(
        &self,
        agent_id: &str,
        active_agent_id: Option<&str>,
    ) -> Result<Option<AgentSummary>, String> {
        self.with_agent_filesystem_mutation(agent_id, || {
            self.summarize_agent_by_id_locked(agent_id, active_agent_id)
        })
    }

    fn summarize_agent_by_id_locked(
        &self,
        agent_id: &str,
        active_agent_id: Option<&str>,
    ) -> Result<Option<AgentSummary>, String> {
        let is_deleting = |candidate: &str| self.is_agent_being_deleted(candidate);
        let owner = self
            .session_db_path(agent_id)
            .ok()
            .filter(|path| path.is_file())
            .map(|_| self.open_agent_db_owner_locked(agent_id))
            .transpose()?;
        summarize_agent_by_id(
            &self.agents_root,
            self.busy_timeout_ms,
            agent_id,
            active_agent_id,
            &is_deleting,
            &self.roster_extras_cache,
            owner.as_deref(),
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
        assert_valid_agent_session_id(agent_id).map_err(|error| error.to_string())?;
        self.with_agent_filesystem_mutation(agent_id, || {
            self.prepare_existing_agent_locked(agent_id)
        })
    }

    fn prepare_existing_agent_locked(
        &self,
        agent_id: &str,
    ) -> Result<Option<PreparedAgentBlobStore>, String> {
        if !self.session_db_path(agent_id)?.is_file() {
            return Ok(None);
        }
        // Check durable metadata/root before materialization repair or pending
        // recovery can write files, journals or admission state.
        let db_owner = self.open_agent_db_owner_locked(agent_id)?;
        let store = self.create_agent_blob_store(agent_id)?;
        let Some(materialized) = open_existing_session(
            &self.agents_root,
            self.busy_timeout_ms,
            agent_id,
        )
        .map_err(|error| error.to_string())?
        else {
            return Ok(None);
        };
        let session_db_path = materialized.db_path.clone();

        // Only a persisted, explicitly empty root may enter legacy repair.
        // Missing or malformed metadata is not a new empty conversation.
        let persisted_root = db_owner.read_checkpoint_root_checked()
            .map_err(|error| error.to_string())?;
        let recovered_root = if persisted_root.is_none() {
            recover_conversation_root_if_missing(
                Arc::clone(&self.pool),
                agent_id,
                &session_db_path,
                &store.blob_db_path,
                self.busy_timeout_ms,
                db_owner.as_ref(),
            )?
        } else {
            false
        };
        if recovered_root {
            let cached_store = self.agent_store_owners.lock()
                .map_err(|_| "agent store owner map poisoned".to_string())?
                .get(agent_id).cloned();
            if let Some(cached_store) = cached_store {
                // Repair advanced durable None to a root: refresh before the
                // normal re-entry guard compares its prior empty snapshot.
                cached_store.reset_from_db_checked()?;
            }
        }
        let agent_store = self.open_agent_store_owner_locked(agent_id)?;
        let persisted_root_blob_id = agent_store.checkpoint_snapshot_checked()?
            .map(|snapshot| snapshot.root_blob_id)
            .unwrap_or_default();
        if recovered_root {
            let _ = sync_recovered_profile_name(
                &session_db_path,
                self.busy_timeout_ms,
                &materialized.profile.name,
                db_owner.as_ref(),
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
                db_owner.as_ref(),
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
            db_owner.as_ref(),
        )?;
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
            db_owner.as_ref(),
        )?;
        let _ = retire_legacy_store_blobs_once(
            Arc::clone(&self.pool),
            agent_id,
            &session_db_path,
            &store.blob_db_path,
            self.busy_timeout_ms,
            db_owner.as_ref(),
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
mod original_turn_terminal_tests {
    use super::*;
    use rusqlite::{OptionalExtension, params};
    use serde_json::json;
    use crate::extensions::session::agent_db::{
        CanonicalTurnCommitRecord, OriginalTurnScope, StoredCompletedTerminal,
        ORIGINAL_TURN_TERMINAL_KEY,
    };
    use crate::extensions::session::agent_db_recovery::{
        AUTOMATION_CUSTODY_UNAVAILABLE_KV, automation_custody_recovery_marker_path,
    };
    use crate::extensions::session::agent_db_schema::{GET_KV_SQL, SET_KV_SQL};

    struct Fixture {
        root: PathBuf,
        workers: ProductionSessionWorkers,
        original: OriginalTurnReference,
        root_id: Vec<u8>,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "fabushi-original-terminal-facade-{label}-{}", Uuid::new_v4(),
            ));
            let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 0);
            let materialized = workers.materialize_new_session(None, "user", None).unwrap();
            let admission = workers.read_session_work_admission(&materialized.id).unwrap().unwrap();
            let original = OriginalTurnReference {
                version: 1,
                scope: OriginalTurnScope::NativeOrdinaryUserV1,
                agent_id: materialized.id.clone(),
                session_key: admission.session_key,
                session_id: materialized.id,
                original_lifecycle_revision: admission.lifecycle_revision,
                stream_id: "original-stream".into(),
                input_digest: "a".repeat(64),
                client_nonce: Some("original-nonce".into()),
                user_message_id: "original-user".into(),
                echo_entry_id: "original-user".into(),
                request_source: None,
                provider: "fabushi".into(),
                model_id: "resolved-native-model".into(),
            };
            let db = workers.open_agent_db_owner(&original.agent_id).unwrap();
            assert!(db.append_transcript_entry(&json!({
                "id": original.user_message_id, "kind": "message", "role": "user",
                "content": "original durable user text", "sessionKey": original.session_key,
                "clientNonce": original.client_nonce, "isStreaming": false,
            })).unwrap());
            let store = workers.open_agent_store_owner(&original.agent_id).unwrap();
            // A real empty protobuf checkpoint gives this facade a nonempty
            // content-addressed root and actual user confirmation. Neutral
            // records below are storage data, never Runner completion proofs.
            let root_id = store.handle_checkpoint_bytes_and_confirm_user_message_blocking(
                &[], &original.user_message_id,
            ).unwrap();
            Self { root, workers, original, root_id }
        }

        fn db_path(&self) -> PathBuf {
            self.workers.session_db_path(&self.original.agent_id).unwrap()
        }

        fn raw_db(&self) -> rusqlite::Connection {
            rusqlite::Connection::open(self.db_path()).unwrap()
        }

        fn raw_kv(&self, key: &str) -> Option<String> {
            self.raw_db().query_row(GET_KV_SQL, params![key], |row| row.get(0)).optional().unwrap()
        }

        fn rows(&self) -> (Vec<(String, String)>, Vec<(i64, String, String)>) {
            let db = self.raw_db();
            let mut statement = db.prepare("SELECT key, value FROM kv ORDER BY key").unwrap();
            let kv = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
                .unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
            let mut statement = db.prepare("SELECT seq, id, entry FROM transcript_entries ORDER BY seq").unwrap();
            let entries = statement.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
                .unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap();
            (kv, entries)
        }

        fn record(&self, view: &OriginalTurnTerminalView) -> OriginalTurnTerminalRecord {
            OriginalTurnTerminalRecord {
                version: 1,
                original: self.original.clone(),
                terminal: StoredCompletedTerminal::Completed,
                final_result_sha256: [7; 32],
                sent_message_count: 0,
                reacted: false,
                custody_json: "{}".into(),
                canonical: vec![CanonicalTurnCommitRecord {
                    ordinal: 0,
                    request_id: self.original.stream_id.clone(),
                    message_id: self.original.user_message_id.clone(),
                    prior_root_blob_id: None,
                    root_blob_id: view.checkpoint.root_blob_id.clone(),
                    user_blob_id: vec![2; 32],
                    turn_blob_id: vec![3; 32],
                    step_blob_ids: vec![vec![4; 32]],
                    assistant_result_sha256: [7; 32],
                }],
                settlement_lifecycle_revision: view.settlement_lifecycle_revision.clone(),
                compaction: view.compaction.clone(),
            }
        }

        fn publish(&self) -> OriginalTurnTerminalRecord {
            let view = self.workers.prepare_original_turn_terminal_view(&self.original, &self.root_id).unwrap();
            let record = self.record(&view);
            assert_eq!(self.workers.commit_original_turn_terminal_record(&view, &record).unwrap(),
                OriginalTurnReceiptCommit::Inserted);
            record
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            self.workers.shutdown();
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn checked_facade_reopens_actual_worker_blob_and_preserves_source_bytes() {
        let fixture = Fixture::new("reopen");
        let record = fixture.publish();
        let before = fixture.rows();
        let owner = fixture.workers.open_agent_db_owner(&fixture.original.agent_id).unwrap();
        assert_ne!(owner.read_metadata().unwrap()["lifecycleRevision"].as_str().unwrap(),
            fixture.original.original_lifecycle_revision);
        drop(owner);
        fixture.workers.shutdown_with_checkpoint();
        let restarted = ProductionSessionWorkers::with_agents_root(fixture.root.join("agents"), 0);
        let read = restarted.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
        assert_eq!(read.record, record);
        assert_eq!(read.view.original_user_content, "original durable user text");
        assert_eq!(read.view.checkpoint.root_blob_id, fixture.root_id);
        assert_eq!(read.view.store.get_blob(&fixture.root_id).unwrap(), Some(Vec::new()));
        restarted.recheck_original_turn_terminal_read(&read).unwrap();
        let second = restarted.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
        restarted.recheck_original_turn_terminal_read(&second).unwrap();
        assert_eq!(second.record, record);
        assert_eq!(fixture.rows(), before);
        restarted.shutdown();
    }

    #[test]
    fn final_facade_recheck_refuses_changed_slot_user_and_canonical_root() {
        for field in ["slot", "user", "root"] {
            let fixture = Fixture::new(field);
            fixture.publish();
            let read = fixture.workers.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
            match field {
                "slot" => { fixture.raw_db().execute("UPDATE kv SET value = value || ' ' WHERE key = ?1",
                    params![ORIGINAL_TURN_TERMINAL_KEY]).unwrap(); }
                "user" => { fixture.raw_db().execute("UPDATE transcript_entries SET entry = entry || ' ' WHERE id = ?1",
                    params![fixture.original.user_message_id]).unwrap(); }
                "root" => {
                    // Actual generated field 17: self_summary_count = 1.
                    let changed_root = read.view.store.handle_checkpoint_bytes_blocking(&[0x88, 0x01, 0x01]).unwrap();
                    assert_ne!(changed_root, fixture.root_id);
                }
                _ => unreachable!(),
            }
            let changed = fixture.rows();
            assert!(fixture.workers.recheck_original_turn_terminal_read(&read).is_err(), "{field}");
            assert!(fixture.workers.commit_original_turn_terminal_record(&read.view, &read.record).is_err(), "{field}");
            assert_eq!(fixture.rows(), changed);
        }
    }

    #[test]
    fn reopened_owner_or_mutated_candidate_cannot_reuse_an_old_view() {
        let fixture = Fixture::new("retired-view");
        fixture.publish();
        let mut read = fixture.workers.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
        let before = fixture.rows();
        read.record.reacted = true;
        assert!(fixture.workers.recheck_original_turn_terminal_read(&read).is_err());
        read.record.reacted = false;
        assert!(fixture.workers.retire_agent_owners(&fixture.original.agent_id, true).unwrap());
        assert!(fixture.workers.recheck_original_turn_terminal_read(&read).is_err());
        let current = fixture.workers.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
        fixture.workers.recheck_original_turn_terminal_read(&current).unwrap();
        assert!(!Arc::ptr_eq(&read.view.store, &current.view.store));
        assert_eq!(fixture.rows(), before);
    }

    #[test]
    fn strict_first_open_retains_pending_recovery_inputs_for_the_ordinary_owner() {
        for kind in ["memory", "profile", "media"] {
            let fixture = Fixture::new(kind);
            fixture.publish();
            let owner = fixture.workers.open_agent_db_owner(&fixture.original.agent_id).unwrap();
            let mut pending_media_path = None;
            let mut retained_media_path = None;
            let (key, raw) = match kind {
                "memory" => ("pendingExternalMemoryMutation", serde_json::to_string(&PendingMemoryMutation {
                    mutation_id: "pending-memory".into(), operation: "clear".into(),
                    session_key: fixture.original.session_key.clone(), session_id: fixture.original.session_id.clone(),
                    lifecycle_revision: fixture.original.original_lifecycle_revision.clone(),
                    records: Vec::new(), prior_snapshot: owner.get_memory_prompt_snapshot().unwrap(),
                }).unwrap()),
                "profile" => {
                    let files = capture_profile_mutation_files(fixture.db_path().parent().unwrap(), true).unwrap();
                    ("pendingExternalProfileMutation", serde_json::to_string(&PendingProfileMutation {
                        mutation_id: "pending-profile".into(), operation: "profile-update".into(),
                        session_key: fixture.original.session_key.clone(), session_id: fixture.original.session_id.clone(),
                        lifecycle_revision: fixture.original.original_lifecycle_revision.clone(),
                        before_files: files.clone(), after_files: files,
                        previous_profile: owner.get_sand_profile().unwrap(), next_profile: None,
                    }).unwrap())
                }
                "media" => {
                    let media_path = fixture.db_path().parent().unwrap()
                        .join("attachments").join("pending-original-turn.bin");
                    fs::create_dir_all(media_path.parent().unwrap()).unwrap();
                    fs::write(&media_path, b"owned pending original-turn media").unwrap();
                    let retained_path = media_path.with_file_name("unrelated-original-turn.bin");
                    fs::write(&retained_path, b"unrelated retained media").unwrap();
                    retained_media_path = Some(retained_path);
                    fixture.workers.capture_session_media_admission(
                        &fixture.original.agent_id, &fixture.original.session_key,
                        "pending-media", Some(&media_path),
                    ).unwrap();
                    let admission = owner.read_session_work_admission().unwrap().unwrap();
                    assert_eq!(admission.admitted_media_work_id.as_deref(), Some("pending-media"));
                    assert_eq!(admission.admitted_media_paths,
                        vec![media_path.to_str().unwrap().to_string()]);
                    pending_media_path = Some(media_path);
                    ("sessionWorkAdmission", serde_json::to_string(&admission).unwrap())
                }
                _ => unreachable!(),
            };
            drop(owner);
            fixture.workers.shutdown_with_checkpoint();
            if kind != "media" {
                fixture.raw_db().execute(SET_KV_SQL, params![key, raw]).unwrap();
            }
            assert_eq!(fixture.raw_kv(key).as_deref(), Some(raw.as_str()));
            let before = fixture.rows();
            let restarted = ProductionSessionWorkers::with_agents_root(fixture.root.join("agents"), 0);
            assert!(restarted.read_original_turn_terminal_candidate(&fixture.original).is_err(), "{kind}");
            assert!(restarted.prepare_original_turn_terminal_view(&fixture.original, &fixture.root_id).is_err(), "{kind}");
            assert_eq!(fixture.rows(), before, "strict inspection cannot recover {kind}");
            assert_eq!(fixture.raw_kv(key).as_deref(), Some(raw.as_str()));
            if let Some(media_path) = &pending_media_path {
                assert_eq!(fs::read(media_path).unwrap(), b"owned pending original-turn media");
                assert_eq!(fs::read(retained_media_path.as_ref().unwrap()).unwrap(), b"unrelated retained media");
            }
            // This is the existing ordinary recovery owner, deliberately
            // called only after the strict path has retained the input.
            let ordinary = restarted.open_agent_db_owner(&fixture.original.agent_id).unwrap();
            if kind == "media" {
                let admission = ordinary.read_session_work_admission().unwrap().unwrap();
                assert!(admission.admitted_media_work_id.is_none());
                assert!(admission.admitted_media_paths.is_empty());
                assert!(!pending_media_path.as_ref().unwrap().exists());
                assert_eq!(fs::read(retained_media_path.as_ref().unwrap()).unwrap(), b"unrelated retained media");
            } else {
                assert!(ordinary.read_kv(key).unwrap().is_none());
            }
            restarted.shutdown();
        }
    }

    #[test]
    fn strict_and_ordinary_first_open_retain_unknown_media_without_durable_paths() {
        let fixture = Fixture::new("unknown-media-no-paths");
        fixture.publish();
        let owner = fixture.workers.open_agent_db_owner(&fixture.original.agent_id).unwrap();
        let mut admission = owner.read_session_work_admission().unwrap().unwrap();
        assert!(admission.admitted_media_work_id.is_none());
        assert!(admission.admitted_media_paths.is_empty());
        admission.admitted_media_work_id = Some("unknown-pending-media".into());
        let raw = serde_json::to_string(&admission).unwrap();
        drop(owner);
        fixture.workers.shutdown_with_checkpoint();
        fixture.raw_db().execute(SET_KV_SQL, params!["sessionWorkAdmission", raw]).unwrap();
        let before = fixture.rows();
        let restarted = ProductionSessionWorkers::with_agents_root(fixture.root.join("agents"), 0);
        for _ in 0..2 {
            assert!(restarted.read_original_turn_terminal_candidate(&fixture.original).is_err());
            assert!(restarted.prepare_original_turn_terminal_view(&fixture.original, &fixture.root_id).is_err());
            assert_eq!(fixture.rows(), before, "strict inspection must retain unknown media custody");
            let error = restarted.open_agent_db_owner(&fixture.original.agent_id)
                .err().expect("unknown media custody must not be cleared by ordinary open");
            assert_eq!(error, "pending outgoing media has no durable materialized paths");
            assert_eq!(fixture.rows(), before);
            assert_eq!(fixture.raw_kv("sessionWorkAdmission").as_deref(), Some(raw.as_str()));
            assert_eq!(restarted.active_agent_db_owner_count(), 0,
                "failed inspection or recovery cannot cache an unrecovered owner");
        }
        restarted.shutdown();
    }

    #[test]
    fn recovery_markers_block_the_facade_before_an_absent_slot() {
        let fixture = Fixture::new("marker-before-none");
        let view = fixture.workers.prepare_original_turn_terminal_view(&fixture.original, &fixture.root_id).unwrap();
        let record = fixture.record(&view);
        let marker = automation_custody_recovery_marker_path(&fixture.db_path());
        fs::write(&marker, []).unwrap();
        assert!(fixture.workers.read_original_turn_terminal_candidate(&fixture.original).is_err());
        assert!(fixture.workers.commit_original_turn_terminal_record(&view, &record).is_err());
        assert!(fixture.raw_kv(ORIGINAL_TURN_TERMINAL_KEY).is_none());
        fs::remove_file(&marker).unwrap();
        fixture.raw_db().execute(SET_KV_SQL, params![AUTOMATION_CUSTODY_UNAVAILABLE_KV, "malformed marker"]).unwrap();
        assert!(fixture.workers.read_original_turn_terminal_candidate(&fixture.original).is_err());
        assert_eq!(fixture.raw_kv(AUTOMATION_CUSTODY_UNAVAILABLE_KV).as_deref(), Some("malformed marker"));
        assert!(fixture.raw_kv(ORIGINAL_TURN_TERMINAL_KEY).is_none());
    }

    #[test]
    fn strict_facade_open_does_not_rebuild_corruption() {
        let fixture = Fixture::new("no-corruption-recovery");
        fixture.publish();
        fixture.workers.shutdown_with_checkpoint();
        let db_path = fixture.db_path();
        let damaged = b"not a sqlite database: retain the original bytes";
        fs::write(&db_path, damaged).unwrap();
        let restarted = ProductionSessionWorkers::with_agents_root(fixture.root.join("agents"), 0);
        assert!(restarted.read_original_turn_terminal_candidate(&fixture.original).is_err());
        assert_eq!(fs::read(&db_path).unwrap(), damaged);
        assert!(!automation_custody_recovery_marker_path(&db_path).exists());
        restarted.shutdown();
    }

    #[test]
    fn actual_facade_clear_handles_a_nonempty_canonical_root_and_retires_cache() {
        let fixture = Fixture::new("clear");
        fixture.publish();
        let read = fixture.workers.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
        assert!(!read.view.checkpoint.root_blob_id.is_empty());
        assert_eq!(fixture.workers.active_agent_store_owner_count(), 1);
        assert!(fixture.workers.clear_agent_conversation(&fixture.original.agent_id).unwrap());
        assert!(fixture.raw_kv(ORIGINAL_TURN_TERMINAL_KEY).is_none());
        assert_eq!(fixture.workers.active_agent_store_owner_count(), 0);
        assert!(fixture.workers.recheck_original_turn_terminal_read(&read).is_err());
        let store = fixture.workers.open_agent_store_owner(&fixture.original.agent_id).unwrap();
        assert!(store.checkpoint_snapshot_checked().unwrap().is_none());
        let owner = fixture.workers.open_agent_db_owner(&fixture.original.agent_id).unwrap();
        assert!(owner.get_transcript_entries().unwrap().is_empty());
        assert!(fixture.workers.read_original_turn_terminal_candidate(&fixture.original).unwrap().is_none());
    }

    #[test]
    fn failed_clear_keeps_sql_evidence_and_cannot_acknowledge_removed_blobs() {
        let fixture = Fixture::new("clear-failure");
        fixture.publish();
        let read = fixture.workers.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
        fixture.raw_db().execute_batch(
            "CREATE TRIGGER terminal_refuse_clear BEFORE DELETE ON kv
             WHEN OLD.key = 'original-routed-terminal:current'
             BEGIN SELECT RAISE(ABORT, 'clear refused'); END;",
        ).unwrap();
        let before = fixture.rows();
        assert!(fixture.workers.clear_agent_conversation(&fixture.original.agent_id).is_err());
        assert_eq!(fixture.rows(), before);
        // The existing clear order spans two stores: blob deletion can have
        // happened before the SQLite failure. No cross-store rollback or
        // successful terminal acknowledgment is inferred from retained rows.
        assert!(read.view.store.get_blob(&fixture.root_id).unwrap().is_none());
        assert!(fixture.workers.read_original_turn_terminal_candidate(&fixture.original).is_err());
        assert!(fixture.workers.recheck_original_turn_terminal_read(&read).is_err());
    }

    #[test]
    fn deletion_boundary_and_missing_directory_cannot_recreate_terminal_authority() {
        let fixture = Fixture::new("delete");
        fixture.publish();
        let read = fixture.workers.read_original_turn_terminal_candidate(&fixture.original).unwrap().unwrap();
        fixture.workers.begin_agent_delete(&fixture.original.agent_id).unwrap();
        assert!(fixture.workers.read_original_turn_terminal_candidate(&fixture.original).is_err());
        assert!(fixture.workers.recheck_original_turn_terminal_read(&read).is_err());
        fixture.workers.retire_agent_owners(&fixture.original.agent_id, false).unwrap();
        let directory = fixture.db_path().parent().unwrap().to_path_buf();
        fixture.workers.with_agent_filesystem_mutation(&fixture.original.agent_id, || {
            fs::remove_dir_all(&directory).map_err(|error| error.to_string())
        }).unwrap();
        fixture.workers.clear_agent_delete_fast_path(&fixture.original.agent_id);
        assert!(fixture.workers.read_original_turn_terminal_candidate(&fixture.original).is_err());
        assert!(!directory.exists());
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
            .unwrap();
        assert_eq!(
            changed.pointer("/destinations/0/status").and_then(serde_json::Value::as_str),
            Some("failed"),
        );
        assert!(changed.pointer("/destinations/0/error").and_then(serde_json::Value::as_str)
            .is_some_and(|error| error.contains("forward context")));

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
        restarted.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}

#[cfg(test)]
#[path = "production_fresh_append_tests.rs"]
mod fresh_append_tests;

#[cfg(test)]
mod plugin_skills_admission_tests {
    use super::*;
    use crate::extensions::mcp::plugin_skills::{
        LoadedPlugins, PluginSkillsLoader, SandPluginSkillsService,
    };
    use serde_json::{Value, json};
    use std::sync::Weak;

    struct Loader(LoadedPlugins);

    impl PluginSkillsLoader for Loader {
        fn load(&self) -> Result<LoadedPlugins, String> {
            Ok(self.0.clone())
        }
    }

    fn service(root: &Path, blocked: bool) -> SandPluginSkillsService {
        let loaded: LoadedPlugins = serde_json::from_value(json!({
            "plugins":[],
            "authBlocked": if blocked { vec![json!({"pluginId":"9", "pluginName":"Private"})] }
                else { Vec::<Value>::new() },
            "listedPluginIds": if blocked { vec!["9"] } else { Vec::<&str>::new() },
            "listedCacheKeys":[], "publisherFacts":{}, "currentUserId":12
        })).unwrap();
        let service = SandPluginSkillsService::new(root, Arc::new(Loader(loaded)));
        assert!(service.sync("real-forwarder-fixture").unwrap().is_empty());
        service
    }

    #[test]
    fn first_bind_same_arc_and_non_overwriting_rebind_preserve_real_owner_decisions() {
        let root = std::env::temp_dir().join(format!("fabushi-plugin-forward-bind-{}", Uuid::new_v4()));
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        assert!(!workers.plugin_skills_admission.is_denied("9"), "standalone unbound behavior");
        let owner = service(&root, true);
        let reader = owner.admission_reader();
        assert!(reader.is_denied("9"));
        workers.bind_plugin_skills_admission(Arc::clone(&reader)).unwrap();
        workers.bind_plugin_skills_admission(owner.admission_reader()).unwrap();
        let unrelated = service(&root.join("unrelated"), false);
        assert!(!unrelated.admission_reader().is_denied("9"));
        assert!(workers.bind_plugin_skills_admission(unrelated.admission_reader()).is_err());
        assert!(workers.plugin_skills_admission.is_denied("9"));
        assert!(!workers.plugin_skills_admission.is_denied("10"));
        let slot = workers.plugin_skills_admission.reader.lock().unwrap();
        assert!(Arc::ptr_eq(slot.as_ref().unwrap(), &reader));
        drop(slot);
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }

    struct LockObservingReader {
        forwarding: Weak<SessionPluginSkillsAdmission>,
        delegate: PluginSkillsAdmissionReader,
    }

    impl PluginSkillsAdmissionQuery for LockObservingReader {
        fn is_denied(&self, plugin_id: &str) -> bool {
            let forwarding = self.forwarding.upgrade().unwrap();
            let slot = forwarding.reader.try_lock();
            assert!(slot.is_ok(), "forwarding must release its slot before calling the real owner");
            drop(slot);
            self.delegate.is_denied(plugin_id)
        }
    }

    #[test]
    fn forwarding_releases_its_slot_before_querying_the_real_owner() {
        let root = std::env::temp_dir().join(format!("fabushi-plugin-forward-unlock-{}", Uuid::new_v4()));
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let owner = service(&root, true);
        let observer: PluginSkillsAdmissionReader = Arc::new(LockObservingReader {
            forwarding: Arc::downgrade(&workers.plugin_skills_admission),
            delegate: owner.admission_reader(),
        });
        workers.bind_plugin_skills_admission(observer).unwrap();
        assert!(workers.plugin_skills_admission.is_denied("9"));
        assert!(!workers.plugin_skills_admission.is_denied("10"));
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn poisoned_forwarding_slot_refuses_queries_and_rebinding() {
        let root = std::env::temp_dir().join(format!("fabushi-plugin-forward-poison-{}", Uuid::new_v4()));
        let workers = ProductionSessionWorkers::with_agents_root(root.join("agents"), 500);
        let owner = service(&root, true);
        workers.bind_plugin_skills_admission(owner.admission_reader()).unwrap();
        assert!(!workers.plugin_skills_admission.is_denied("10"));
        let poisoned = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _slot = workers.plugin_skills_admission.reader.lock().unwrap();
            panic!("deliberately poison the actual forwarding slot");
        }));
        assert!(poisoned.is_err());
        assert!(workers.plugin_skills_admission.is_denied("9"));
        assert!(workers.plugin_skills_admission.is_denied("10"), "poison must not become an empty refusal set");
        assert!(workers.bind_plugin_skills_admission(owner.admission_reader()).is_err());
        workers.shutdown();
        let _ = fs::remove_dir_all(root);
    }
}
