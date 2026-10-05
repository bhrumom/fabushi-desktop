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
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
};
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
    FabushiNativeMessagingClient, FabushiRemoteHumanMessage, fabushi_identity_text,
};

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
    agent_store_owners: Mutex<BTreeMap<String, Arc<ProductionAgentStore>>>,
    deleting_agents: Mutex<BTreeSet<String>>,
    roster_extras_cache: RosterExtrasCache,
    user_time_zone_resolver: UserTimeZoneResolver,
    local_human_id: Option<String>,
    native_messaging: Option<Arc<FabushiNativeMessagingClient>>,
    native_messaging_required: bool,
    native_messaging_error: Option<String>,
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
            agent_store_owners: Mutex::new(BTreeMap::new()),
            deleting_agents: Mutex::new(BTreeSet::new()),
            roster_extras_cache: RosterExtrasCache::default(),
            user_time_zone_resolver,
            local_human_id,
            native_messaging,
            native_messaging_required,
            native_messaging_error,
            busy_timeout_ms,
        }
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
        let existing = owner
            .get_transcript_entries()
            .map_err(|error| error.to_string())?;
        if let Some(current) = existing.iter().find(|candidate| {
            candidate.get("id").and_then(serde_json::Value::as_str)
                == entry.get("id").and_then(serde_json::Value::as_str)
        }) {
            return Ok(current.clone());
        }
        if let Some(client_nonce) = entry.get("clientNonce").and_then(serde_json::Value::as_str) {
            if let Some(current) = existing.iter().find(|candidate| {
                candidate.get("clientNonce").and_then(serde_json::Value::as_str)
                    == Some(client_nonce)
            }) {
                if current.get("remoteMessageId").is_some() {
                    return Ok(current.clone());
                }
                return Err(
                    "local Human message already uses the server clientRequestId without a remote identity"
                        .into(),
                );
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
            return Ok(replay);
        }
        let _ = owner
            .set_metadata("lastActivityAt", serde_json::json!(timestamp_ms))
            .map_err(|error| error.to_string())?;
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
            .insert(Self::native_conversation_owner_key(&conversation_id), Arc::clone(&owner));
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
        let key = Self::native_conversation_owner_key(conversation_id);
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

    pub fn append_human_message(
        &self,
        conversation_id: &str,
        text: &str,
        client_nonce: &str,
        composed_at_ms: Option<f64>,
        reply_to_id: Option<&str>,
        attachments: &[serde_json::Value],
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
        if let Some(entry) = existing.iter().find(|entry| {
            entry.get("clientNonce").and_then(serde_json::Value::as_str) == Some(client_nonce)
        }) {
            let same_sender = entry.get("authorId").and_then(serde_json::Value::as_str) == Some(sender_id);
            let same_text = entry.get("content").and_then(serde_json::Value::as_str) == Some(text);
            let same_reply = entry.get("replyToId").and_then(serde_json::Value::as_str) == normalized_reply;
            let same_attachments = entry
                .get("attachments")
                .and_then(serde_json::Value::as_array)
                .is_none_or(|value| value.is_empty())
                && normalized_attachments.is_empty()
                || entry.get("attachments") == Some(&serde_json::Value::Array(normalized_attachments.clone()));
            if same_sender && same_text && same_reply && same_attachments {
                return Ok(entry.clone());
            }
            return Err("sendHumanMessage clientNonce already identifies different content".into());
        }

        if let Some(client) = self.shipping_native_messaging()? {
            if normalized_reply.is_some() || !normalized_attachments.is_empty() {
                return Err(
                    "Fabushi native messaging backend does not yet accept reply or attachment metadata; refusing local-only divergence"
                        .into(),
                );
            }
            let peer_human_id = self.peer_human_id_from_metadata(&metadata)?;
            let remote = client.send_direct_message(&peer_human_id, text, client_nonce)?;
            let remote_sender = fabushi_identity_text(&remote.sender_user_id)?;
            let remote_recipient = fabushi_identity_text(&remote.recipient_user_id)?;
            if remote_sender != sender_id
                || remote_recipient != peer_human_id
                || remote.text.trim() != text
                || remote.client_request_id.as_deref() != Some(client_nonce)
            {
                return Err("Fabushi Human message backend returned mismatched persisted content".into());
            }
            return self.materialize_remote_human_message(&owner, &remote);
        }

        let timestamp_ms = composed_at_ms.unwrap_or_else(|| {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis() as f64
        });
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
