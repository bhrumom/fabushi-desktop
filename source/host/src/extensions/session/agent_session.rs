use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

use crate::agents::agent_profile::SandAgentProfile;
use crate::agents::settings_file::{get_sand_settings_path, write_sand_settings_file};
use crate::storage::store_db::delete_sand_agent_db_write_generation;
use crate::transcript_mutation_events::publish_transcript_mutation;

use super::agent_db_serde::AwaitingUserResponse;
use super::channel_store::ChannelConnection;
use super::conversation_blobs_path::conversation_blobs_path;
use super::production::{PreparedAgentBlobStore, ProductionSessionWorkers};
use super::session_materialization::MaterializedAgentRecord;
use super::session_paths::ACTIVE_AGENT_FILENAME;
use super::session_profile_files::{
    AgentAvatarResponse, AgentProfileUpdate, resolve_profile_name as resolve_profile_name_impl,
};
use super::session_summaries::AgentSummary;

pub fn resolve_profile_name(
    trimmed_name: &str,
    current: Option<&SandAgentProfile>,
) -> String {
    resolve_profile_name_impl(trimmed_name, current)
}

pub struct SandAgentSessionStore {
    production: Arc<ProductionSessionWorkers>,
}

impl SandAgentSessionStore {
    pub fn new(production: Arc<ProductionSessionWorkers>) -> Self {
        Self { production }
    }

    pub fn production(&self) -> &Arc<ProductionSessionWorkers> {
        &self.production
    }

    pub fn get_root_dir(&self) -> &Path {
        self.production.agents_root()
    }

    pub fn get_agent_dir(&self, agent_id: &str) -> PathBuf {
        self.get_root_dir().join(agent_id)
    }

    pub fn agent_exists(&self, agent_id: &str) -> bool {
        self.production
            .session_db_path(agent_id)
            .map(|path| path.is_file())
            .unwrap_or(false)
    }

    pub fn agent_dir_exists(&self, agent_id: &str) -> bool {
        self.get_agent_dir(agent_id).is_dir()
    }

    pub fn active_agent_pointer_path(&self) -> PathBuf {
        self.get_root_dir().join(ACTIVE_AGENT_FILENAME)
    }

    pub fn read_active_agent_id(&self) -> Option<String> {
        let raw = fs::read_to_string(self.active_agent_pointer_path()).ok()?;
        let parsed = serde_json::from_str::<Value>(&raw).ok()?;
        parsed
            .get("activeAgentId")
            .and_then(Value::as_str)
            .filter(|value| !value.is_empty())
            .map(ToOwned::to_owned)
    }

    pub fn write_active_agent_id(&self, agent_id: &str) -> io::Result<()> {
        fs::create_dir_all(self.get_root_dir())?;
        let path = self.active_agent_pointer_path();
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = PathBuf::from(format!(
            "{}.{}.{}.tmp",
            path.display(),
            process::id(),
            nanos
        ));
        fs::write(
            &temporary,
            serde_json::to_vec(&serde_json::json!({ "activeAgentId": agent_id }))
                .map_err(io::Error::other)?,
        )?;
        if let Err(error) = fs::rename(&temporary, &path) {
            let _ = fs::remove_file(&temporary);
            return Err(error);
        }
        Ok(())
    }

    pub fn clear_active_agent_id(&self) -> io::Result<()> {
        match fs::remove_file(self.active_agent_pointer_path()) {
            Ok(()) => Ok(()),
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            Err(error) => Err(error),
        }
    }

    pub fn list_agent_record_ids(&self) -> Result<Vec<String>, String> {
        self.production.list_agent_record_ids()
    }

    pub fn count_owned_agents(&self) -> Result<usize, String> {
        self.production.count_owned_agents()
    }

    pub fn create_session(
        &self,
        profile: Option<&SandAgentProfile>,
        origin: &str,
        purpose: Option<&str>,
    ) -> Result<MaterializedAgentRecord, String> {
        let active = self.read_active_agent_id();
        self.production
            .materialize_new_session_with_active(
                profile,
                origin,
                purpose,
                active.as_deref(),
            )
    }

    pub fn create_fallback_session(
        &self,
    ) -> Result<super::production::FallbackSession, String> {
        let active = self.read_active_agent_id();
        self.production.create_fallback_session(active.as_deref())
    }

    pub fn open_session(
        &self,
        agent_id: &str,
    ) -> Result<Option<PreparedAgentBlobStore>, String> {
        self.production.prepare_existing_agent(agent_id)
    }

    pub fn delete_session(&self, agent_id: &str) -> Result<(), String> {
        self.production.begin_agent_delete(agent_id);
        let result = (|| {
            let db_path = self.production.session_db_path(agent_id)?;
            let blob_path = conversation_blobs_path(&db_path);
            let _ = self.production.close_agent_db_owner(agent_id, false);
            futures::executor::block_on(self.production.worker_pool().close_store(&blob_path));
            delete_sand_agent_db_write_generation(&db_path);
            match fs::remove_dir_all(self.get_agent_dir(agent_id)) {
                Ok(()) => {}
                Err(error) if error.kind() == io::ErrorKind::NotFound => {}
                Err(error) => return Err(error.to_string()),
            }
            let mutation = Map::from_iter([
                ("kind".to_string(), Value::String("agent-removed".to_string())),
                ("agentId".to_string(), Value::String(agent_id.to_string())),
            ]);
            publish_transcript_mutation(&mutation);
            Ok(())
        })();
        self.production.end_agent_delete(agent_id);
        result
    }

    pub fn list_agents(&self) -> Result<Vec<AgentSummary>, String> {
        let active = self.read_active_agent_id();
        self.production.list_agent_summaries(active.as_deref())
    }

    pub fn summarize_agent_by_id(
        &self,
        agent_id: &str,
    ) -> Result<Option<AgentSummary>, String> {
        let active = self.read_active_agent_id();
        self.production
            .summarize_agent_by_id(agent_id, active.as_deref())
    }

    pub fn update_agent_profile(
        &self,
        agent_id: &str,
        update: &AgentProfileUpdate,
    ) -> Result<Option<AgentSummary>, String> {
        let active = self.read_active_agent_id();
        self.production
            .update_agent_profile(agent_id, update, active.as_deref())
    }

    pub fn get_agent_profile_text(
        &self,
        agent_id: &str,
    ) -> Result<Option<SandAgentProfile>, String> {
        self.production.get_agent_profile_text(agent_id)
    }

    pub fn get_agent_avatar(&self, agent_id: &str) -> Result<AgentAvatarResponse, String> {
        self.production.get_agent_avatar(agent_id)
    }

    pub fn get_agent_avatar_png(&self, agent_id: &str) -> Result<Option<Vec<u8>>, String> {
        self.production.get_agent_avatar_png(agent_id)
    }

    pub fn set_agent_avatar_bytes(
        &self,
        agent_id: &str,
        png_bytes: Option<&[u8]>,
    ) -> Result<Option<AgentSummary>, String> {
        let active = self.read_active_agent_id();
        self.production
            .set_agent_avatar_bytes(agent_id, png_bytes, active.as_deref())
    }

    pub fn read_agent_transcript_entries(
        &self,
        agent_id: &str,
    ) -> Result<Vec<Value>, String> {
        self.production.read_agent_transcript_entries(agent_id)
    }

    pub fn mark_agent_viewed(
        &self,
        agent_id: &str,
        at: f64,
        preserve_manual_unread: bool,
    ) -> Result<bool, String> {
        self.production
            .mark_agent_viewed(agent_id, at, preserve_manual_unread)
    }

    pub fn mark_agent_activity(&self, agent_id: &str, at: f64) -> Result<bool, String> {
        self.production.mark_agent_activity(agent_id, at)
    }

    pub fn set_session_unread(
        &self,
        agent_id: &str,
        unread: bool,
        at: f64,
    ) -> Result<bool, String> {
        self.production.set_agent_unread(agent_id, unread, at)
    }

    pub fn set_awaiting_user_response(
        &self,
        agent_id: &str,
        state: Option<&AwaitingUserResponse>,
    ) -> Result<bool, String> {
        self.production
            .set_agent_awaiting_user_response(agent_id, state)
    }

    pub fn set_session_notify_on_updates(
        &self,
        agent_id: &str,
        enabled: bool,
    ) -> Result<(), String> {
        self.write_settings(
            agent_id,
            Map::from_iter([(
                "notifyOnAgentUpdates".to_string(),
                Value::Bool(enabled),
            )]),
        )
    }

    pub fn set_session_hidden_from_sidebar(
        &self,
        agent_id: &str,
        hidden: bool,
    ) -> Result<(), String> {
        self.write_settings(
            agent_id,
            Map::from_iter([(
                "hiddenFromSidebar".to_string(),
                Value::Bool(hidden),
            )]),
        )
    }

    fn write_settings(&self, agent_id: &str, update: Map<String, Value>) -> Result<(), String> {
        if !self.agent_dir_exists(agent_id) {
            return Err(format!("Agent missing: {agent_id}"));
        }
        write_sand_settings_file(
            get_sand_settings_path(self.get_agent_dir(agent_id)),
            &update,
        )
        .map_err(|error| error.to_string())
    }

    pub fn list_agent_channels(&self, agent_id: &str) -> Result<Vec<ChannelConnection>, String> {
        self.production.list_agent_channels(agent_id)
    }

    pub fn store_connector_credential(
        &self,
        agent_id: &str,
        platform: &str,
        field: &str,
        value: &str,
    ) -> Result<bool, String> {
        self.production
            .store_connector_credential(agent_id, platform, field, value)
    }

    pub fn disconnect_channel(&self, agent_id: &str, platform: &str) -> Result<bool, String> {
        self.production.disconnect_channel(agent_id, platform)
    }

    pub fn close_worker_pool(&self) {
        self.production.shutdown();
    }
}
