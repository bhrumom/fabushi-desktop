use std::path::{Path, PathBuf};
use std::sync::Arc;

use crate::agent_isolation::{
    AgentWorkerPool, ProductionAgentStoreWorkerBackend, WorkerBlobStore,
    create_production_agent_store_worker_backend,
};
use crate::agents::agent_profile::SandAgentProfile;
use crate::storage::agent_paths::get_sand_agents_root_dir;

use super::agent_db::{
    AgentDbSerdeSnapshot, add_persisted_conversation_partner,
    append_persisted_transcript_entries, clear_persisted_agent_profile_prompt_snapshot,
    clear_persisted_conversation, clear_persisted_memory_prompt_snapshot,
    clear_persisted_transient_state, delete_persisted_transcript_entry,
    mark_persisted_activity, mark_persisted_read, mark_persisted_unread,
    mark_persisted_viewed, read_persisted_agent_profile_prompt_snapshot,
    read_persisted_agent_serde_snapshot, read_persisted_automation_spend_guard_state,
    read_persisted_conversation_partner_ids, read_persisted_introduction_pending,
    read_persisted_latest_root_blob_id, read_persisted_newest_divider_anchor_timestamp_ms,
    record_persisted_episode_turn, record_persisted_request_id,
    set_persisted_agent_profile_prompt_snapshot, set_persisted_automation_spend_guard_state,
    set_persisted_awaiting_user_response, set_persisted_awaiting_user_response_for_tab,
    set_persisted_introduction_pending, set_persisted_memory_prompt_snapshot,
    set_persisted_sand_profile, update_persisted_transcript_entry,
};
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
};
use super::agent_db_serde::{
    AwaitingUserResponse, EpisodeTurn, SandProfile, SpendGuardState,
};
use super::session_conversation_state::{
    ConversationOutlineItem, SessionConversationState, TranscriptThread,
};
use super::conversation_blobs_path::conversation_blobs_path;
use super::conversation_size_limits::{
    ConversationGcTarget, ConversationSizeMaintenance, ConversationSizePolicy,
};
use super::session_paths::{get_agent_db_path, get_connector_secrets_root};
use super::connector_secret_store::SandConnectorSecretStore;
use super::channel_store::{ChannelConfig, ChannelConnection, FileChannelStore};
use super::session_store_factories::channel_store_for_db_path;
use super::session_maintenance::{
    clear_stale_checkpoint_roots_once, recover_conversation_root_if_missing,
    retire_legacy_store_blobs_once,
};
use super::session_materialization::{
    MaterializedAgentRecord, SessionMintQueue, count_owned_agents, is_agent_cap_reached,
    list_agent_record_ids, materialize_new_session, open_existing_session,
};
use super::pending_card_sweeps::{
    expire_pending_auto_review_approval_entries,
    expire_pending_local_tool_permission_ask_entries,
};
use super::session_roster::{list_agents, summarize_agent_by_id};
use super::session_summaries::AgentSummary;
use super::session_profile_files::{
    AgentAvatarResponse, AgentProfileUpdate, get_agent_avatar as get_profile_avatar,
    get_agent_avatar_png as get_profile_avatar_png,
    get_agent_profile_text as read_agent_profile_text, write_agent_profile_update,
};
use super::session_mutations::set_agent_avatar_bytes as mutate_agent_avatar_bytes;

pub const PRODUCTION_BLOB_BUSY_TIMEOUT_MS: u64 = 5_000;

pub type ProductionAgentWorkerPool = AgentWorkerPool<ProductionAgentStoreWorkerBackend>;
pub type ProductionWorkerBlobStore = WorkerBlobStore<ProductionAgentStoreWorkerBackend>;

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
    mint_queue: SessionMintQueue,
    busy_timeout_ms: u64,
}

impl ProductionSessionWorkers {
    pub fn production() -> Self {
        Self::with_agents_root(
            get_sand_agents_root_dir(None),
            PRODUCTION_BLOB_BUSY_TIMEOUT_MS,
        )
    }

    pub fn with_agents_root(
        agents_root: impl Into<PathBuf>,
        busy_timeout_ms: u64,
    ) -> Self {
        Self {
            agents_root: agents_root.into(),
            pool: Arc::new(AgentWorkerPool::new(
                create_production_agent_store_worker_backend(busy_timeout_ms),
            )),
            conversation_size_maintenance: ConversationSizeMaintenance::default(),
            conversation_state: SessionConversationState::new(busy_timeout_ms),
            mint_queue: SessionMintQueue::default(),
            busy_timeout_ms,
        }
    }

    pub fn worker_pool(&self) -> Arc<ProductionAgentWorkerPool> {
        Arc::clone(&self.pool)
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

    pub fn materialize_new_session(
        &self,
        profile: Option<&SandAgentProfile>,
        origin: &str,
        purpose: Option<&str>,
    ) -> Result<MaterializedAgentRecord, String> {
        self.mint_queue.run(|| {
            materialize_new_session(
                &self.agents_root,
                self.busy_timeout_ms,
                profile,
                origin,
                purpose,
            )
            .map_err(|error| error.to_string())
        })
    }

    pub fn read_agent_transcript_entries(
        &self,
        agent_id: &str,
    ) -> Result<Vec<serde_json::Value>, String> {
        let db_path = self.session_db_path(agent_id)?;
        self.conversation_state
            .read_agent_transcript_entries(&db_path)
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_transcript_page(
        &self,
        agent_id: &str,
        query: TranscriptPageQuery,
    ) -> Result<TranscriptPage, String> {
        let db_path = self.session_db_path(agent_id)?;
        self.conversation_state
            .read_agent_transcript_page(&db_path, query)
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_transcript_window(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptWindow<std::collections::BTreeMap<String, usize>>, String> {
        let db_path = self.session_db_path(agent_id)?;
        self.conversation_state
            .read_agent_transcript_window(&db_path, query)
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_transcript_tail(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, String> {
        let db_path = self.session_db_path(agent_id)?;
        self.conversation_state
            .read_agent_transcript_tail(&db_path, query)
            .map_err(|error| error.to_string())
    }

    pub fn read_agent_thread(
        &self,
        agent_id: &str,
        root_id: &str,
    ) -> Result<TranscriptThread, String> {
        let db_path = self.session_db_path(agent_id)?;
        self.conversation_state
            .read_agent_thread(&db_path, root_id)
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

    pub fn connector_secret_store(&self) -> SandConnectorSecretStore {
        SandConnectorSecretStore::new(get_connector_secrets_root(Some(&self.agents_root)))
    }

    pub fn open_channel_store(&self, agent_id: &str) -> Result<FileChannelStore, String> {
        let db_path = self.session_db_path(agent_id)?;
        Ok(channel_store_for_db_path(&db_path))
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

    pub fn set_agent_sand_profile(
        &self,
        agent_id: &str,
        profile: &SandProfile,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        set_persisted_sand_profile(&db_path, self.busy_timeout_ms, profile)
            .map_err(|error| error.to_string())
    }

    pub fn mark_agent_activity(&self, agent_id: &str, at: f64) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        mark_persisted_activity(&db_path, self.busy_timeout_ms, at)
            .map_err(|error| error.to_string())
    }

    pub fn mark_agent_viewed(
        &self,
        agent_id: &str,
        at: f64,
        preserve_manual_unread: bool,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        mark_persisted_viewed(
            &db_path,
            self.busy_timeout_ms,
            at,
            preserve_manual_unread,
        )
        .map_err(|error| error.to_string())
    }

    pub fn set_agent_unread(
        &self,
        agent_id: &str,
        unread: bool,
        at: f64,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        if unread {
            mark_persisted_unread(&db_path, self.busy_timeout_ms, at)
        } else {
            mark_persisted_read(&db_path, self.busy_timeout_ms, at)
        }
        .map_err(|error| error.to_string())
    }

    pub fn get_agent_introduction_pending(&self, agent_id: &str) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        read_persisted_introduction_pending(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_introduction_pending(
        &self,
        agent_id: &str,
        pending: bool,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        set_persisted_introduction_pending(&db_path, self.busy_timeout_ms, pending)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_automation_spend_guard_state(
        &self,
        agent_id: &str,
    ) -> Result<SpendGuardState, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        read_persisted_automation_spend_guard_state(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_automation_spend_guard_state(
        &self,
        agent_id: &str,
        state: &SpendGuardState,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        set_persisted_automation_spend_guard_state(&db_path, self.busy_timeout_ms, state)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_conversation_partner_ids(
        &self,
        agent_id: &str,
    ) -> Result<Vec<String>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        read_persisted_conversation_partner_ids(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn add_agent_conversation_partner(
        &self,
        agent_id: &str,
        partner_id: &str,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        add_persisted_conversation_partner(
            &db_path,
            self.busy_timeout_ms,
            partner_id,
        )
        .map_err(|error| error.to_string())
    }

    pub fn get_agent_newest_divider_anchor_timestamp_ms(
        &self,
        agent_id: &str,
    ) -> Result<f64, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        read_persisted_newest_divider_anchor_timestamp_ms(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_awaiting_user_response(
        &self,
        agent_id: &str,
        state: Option<&AwaitingUserResponse>,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        set_persisted_awaiting_user_response(&db_path, self.busy_timeout_ms, state)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_awaiting_user_response_for_tab(
        &self,
        agent_id: &str,
        tab_id: &str,
        state: Option<&AwaitingUserResponse>,
        if_since_before: Option<f64>,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        set_persisted_awaiting_user_response_for_tab(
            &db_path,
            self.busy_timeout_ms,
            tab_id,
            state,
            if_since_before,
        )
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
        let db_path = self.existing_session_db_path(agent_id)?;
        record_persisted_request_id(
            &db_path,
            self.busy_timeout_ms,
            request_id,
            at,
            prompt,
            source,
        )
        .map_err(|error| error.to_string())
    }

    pub fn record_agent_episode_turn(
        &self,
        agent_id: &str,
        turn: &EpisodeTurn,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        record_persisted_episode_turn(&db_path, self.busy_timeout_ms, turn)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_memory_prompt_snapshot(
        &self,
        agent_id: &str,
        snapshot: &serde_json::Value,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        set_persisted_memory_prompt_snapshot(&db_path, self.busy_timeout_ms, snapshot)
            .map_err(|error| error.to_string())
    }

    pub fn clear_agent_memory_prompt_snapshot(&self, agent_id: &str) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        clear_persisted_memory_prompt_snapshot(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn get_agent_profile_prompt_snapshot(
        &self,
        agent_id: &str,
    ) -> Result<Option<serde_json::Value>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        read_persisted_agent_profile_prompt_snapshot(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn set_agent_profile_prompt_snapshot(
        &self,
        agent_id: &str,
        snapshot: &serde_json::Value,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        set_persisted_agent_profile_prompt_snapshot(&db_path, self.busy_timeout_ms, snapshot)
            .map_err(|error| error.to_string())
    }

    pub fn clear_agent_profile_prompt_snapshot(&self, agent_id: &str) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        clear_persisted_agent_profile_prompt_snapshot(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn clear_agent_transient_state(&self, agent_id: &str) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        clear_persisted_transient_state(&db_path, self.busy_timeout_ms)
            .map_err(|error| error.to_string())
    }

    pub fn append_agent_transcript_entries(
        &self,
        agent_id: &str,
        entries: &[serde_json::Value],
    ) -> Result<usize, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        append_persisted_transcript_entries(&db_path, self.busy_timeout_ms, entries)
            .map_err(|error| error.to_string())
    }

    pub fn update_agent_transcript_entry(
        &self,
        agent_id: &str,
        entry_id: &str,
        next: &serde_json::Value,
    ) -> Result<Option<serde_json::Value>, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        update_persisted_transcript_entry(
            &db_path,
            self.busy_timeout_ms,
            entry_id,
            next,
        )
        .map_err(|error| error.to_string())
    }

    pub fn delete_agent_transcript_entry(
        &self,
        agent_id: &str,
        entry_id: &str,
    ) -> Result<bool, String> {
        let db_path = self.existing_session_db_path(agent_id)?;
        delete_persisted_transcript_entry(
            &db_path,
            self.busy_timeout_ms,
            entry_id,
        )
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
        clear_persisted_conversation(&db_path, self.busy_timeout_ms)
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

    pub fn list_agent_summaries(
        &self,
        active_agent_id: Option<&str>,
    ) -> Result<Vec<AgentSummary>, String> {
        list_agents(
            &self.agents_root,
            self.busy_timeout_ms,
            active_agent_id,
        )
    }

    pub fn summarize_agent_by_id(
        &self,
        agent_id: &str,
        active_agent_id: Option<&str>,
    ) -> Result<Option<AgentSummary>, String> {
        summarize_agent_by_id(
            &self.agents_root,
            self.busy_timeout_ms,
            agent_id,
            active_agent_id,
        )
    }

    pub fn create_agent_blob_store(
        &self,
        agent_id: &str,
    ) -> Result<ProductionWorkerBlobStore, String> {
        let session_db_path = self.session_db_path(agent_id)?;
        Ok(WorkerBlobStore::new(
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
        let store = self.create_agent_blob_store(agent_id)?;
        let session_db_path = materialized.db_path.clone();

        let _ = recover_conversation_root_if_missing(
            Arc::clone(&self.pool),
            agent_id,
            &session_db_path,
            &store.blob_db_path,
            self.busy_timeout_ms,
        )?;
        let persisted_root_blob_id =
            read_persisted_latest_root_blob_id(&session_db_path, self.busy_timeout_ms)
                .map_err(|error| error.to_string())?;
        let session_state =
            read_persisted_agent_serde_snapshot(&session_db_path, self.busy_timeout_ms)
                .map_err(|error| error.to_string())?;
        let transcript_tail = self
            .conversation_state
            .read_agent_transcript_tail(
                &session_db_path,
                TranscriptWindowQuery {
                    before_seq: None,
                    limit: 500,
                },
            )
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

        Ok(Some(PreparedAgentBlobStore {
            agent_id: agent_id.to_string(),
            session_db_path,
            blob_db_path: store.blob_db_path.clone(),
            latest_root_blob_id,
            persisted_root_blob_id,
            session_state,
            transcript_tail,
            profile_file,
        }))
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
        futures::executor::block_on(self.pool.close_all());
    }

    pub fn agents_root(&self) -> &Path {
        &self.agents_root
    }
}
