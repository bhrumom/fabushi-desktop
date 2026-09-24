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
use crate::extensions::memory::memory_service::FileMemoryStore;
use crate::automations::automation::{AUTOMATION_UI_LIMIT, AutomationRecord, AutomationSpec};
use crate::automations::automation_store::FileAutomationStore;
use crate::workflows::workflow_library::WorkflowSpec;
use crate::workflows::workflow_store::{
    FileWorkflowStore, WorkflowImportBatch, WorkflowImportResult, WorkflowImportSkipped,
    WorkflowRecord,
};

use super::agent_db_serde::AwaitingUserResponse;
use super::agent_db_transcript_pages::{
    TranscriptPage, TranscriptPageQuery, TranscriptWindow, TranscriptWindowQuery,
};
use super::session_conversation_state::{ConversationOutlineItem, TranscriptThread};
use super::channel_store::{ChannelConfig, ChannelConnection, FileChannelStore};
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

#[derive(Debug, Clone, PartialEq)]
pub struct AgentAutomationEntry {
    pub agent_id: String,
    pub automation: AutomationRecord,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorkflowImportResponse {
    pub workflows: Vec<WorkflowRecord>,
    pub result: WorkflowImportBatch,
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

    pub fn create_memory_store(&self, agent_dir: impl AsRef<Path>) -> FileMemoryStore {
        self.production.memory_service().create_agent_store(agent_dir)
    }

    pub fn agent_has_content(&self, agent_dir: impl AsRef<Path>) -> bool {
        self.production.memory_service().agent_has_content(agent_dir)
    }

    pub fn get_user_time_zone(&self) -> Option<String> {
        self.production.resolve_user_time_zone()
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

    pub fn list_agent_ids(&self) -> Result<Vec<String>, String> {
        self.production.list_agent_record_ids()
    }

    pub fn is_agent_cap_reached(&self) -> Result<bool, String> {
        self.production.is_agent_cap_reached()
    }

    pub fn create_session(
        &self,
        profile: Option<&SandAgentProfile>,
        origin: &str,
        purpose: Option<&str>,
    ) -> Result<MaterializedAgentRecord, String> {
        let active = self.read_active_agent_id();
        self.production
            .materialize_session_with_active(
                profile,
                origin,
                purpose,
                active.as_deref(),
            )
            .map(|session| session.record)
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
        self.production
            .open_materialized_session(agent_id)
            .map(|session| session.map(|session| session.prepared))
    }

    pub fn delete_session(&self, agent_id: &str) -> Result<(), String> {
        self.production.begin_agent_delete(agent_id);
        let result = (|| {
            let db_path = self.production.session_db_path(agent_id)?;
            let blob_path = conversation_blobs_path(&db_path);
            let _ = self.production.close_agent_store_owner(agent_id, false);
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

    pub fn set_agent_avatar_bytes_by_id(
        &self,
        agent_id: &str,
        png_bytes: Option<&[u8]>,
    ) -> Result<Option<AgentSummary>, String> {
        self.set_agent_avatar_bytes(agent_id, png_bytes)
    }

    pub fn read_agent_transcript_entries(
        &self,
        agent_id: &str,
    ) -> Result<Vec<Value>, String> {
        self.production.read_agent_transcript_entries(agent_id)
    }

    pub fn read_agent_transcript_page(
        &self,
        agent_id: &str,
        query: TranscriptPageQuery,
    ) -> Result<TranscriptPage, String> {
        self.production.read_agent_transcript_page(agent_id, query)
    }

    pub fn read_agent_transcript_window(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptWindow<std::collections::BTreeMap<String, usize>>, String> {
        self.production.read_agent_transcript_window(agent_id, query)
    }

    pub fn read_agent_transcript_tail(
        &self,
        agent_id: &str,
        query: TranscriptWindowQuery,
    ) -> Result<TranscriptPage, String> {
        self.production.read_agent_transcript_tail(agent_id, query)
    }

    pub fn read_agent_thread(
        &self,
        agent_id: &str,
        root_id: &str,
    ) -> Result<TranscriptThread, String> {
        self.production.read_agent_thread(agent_id, root_id)
    }

    pub fn get_agent_transcript_entries(
        &self,
        agent_id: &str,
    ) -> Result<Vec<Value>, String> {
        self.production.read_agent_transcript_entries(agent_id)
    }

    pub fn get_agent_outline(
        &self,
        agent_id: &str,
    ) -> Result<Vec<ConversationOutlineItem>, String> {
        self.production.read_agent_outline(agent_id)
    }

    pub fn get_session_outline(
        &self,
        session: &PreparedAgentBlobStore,
    ) -> Result<Vec<ConversationOutlineItem>, String> {
        self.production.read_agent_outline(&session.agent_id)
    }

    pub fn mark_session_viewed(
        &self,
        session: &PreparedAgentBlobStore,
        at: f64,
        preserve_manual_unread: bool,
    ) -> Result<bool, String> {
        self.production
            .mark_agent_viewed(&session.agent_id, at, preserve_manual_unread)
    }

    pub fn mark_session_activity(
        &self,
        session: &PreparedAgentBlobStore,
        at: f64,
    ) -> Result<bool, String> {
        self.production.mark_agent_activity(&session.agent_id, at)
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

    pub fn set_awaiting_user_response_for_tab(
        &self,
        agent_id: &str,
        tab_id: &str,
        state: Option<&AwaitingUserResponse>,
        if_since_before: Option<f64>,
    ) -> Result<bool, String> {
        self.production.set_agent_awaiting_user_response_for_tab(
            agent_id,
            tab_id,
            state,
            if_since_before,
        )
    }

    pub fn expire_pending_auto_review_approvals(
        &self,
        agent_id: &str,
        only_request_id: Option<&str>,
    ) -> Result<Vec<String>, String> {
        self.production
            .expire_pending_auto_review_approvals(agent_id, only_request_id)
    }

    pub fn expire_pending_local_tool_permission_asks(
        &self,
        agent_id: &str,
        only_request_id: Option<&str>,
        if_pending_before_ms: Option<f64>,
    ) -> Result<Vec<String>, String> {
        self.production.expire_pending_local_tool_permission_asks(
            agent_id,
            only_request_id,
            if_pending_before_ms,
        )
    }

    pub fn clear_agent_memory_prompt_snapshot(
        &self,
        agent_id: &str,
    ) -> Result<bool, String> {
        self.production.clear_agent_memory_prompt_snapshot(agent_id)
    }

    pub fn ensure_conversation_capacity_for_turn(
        &self,
        session: &PreparedAgentBlobStore,
    ) -> Result<(), String> {
        self.production.ensure_capacity_for_turn(session)
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

    pub fn automation_store_for(&self, agent_id: &str) -> Result<FileAutomationStore, String> {
        self.production.open_automation_store(agent_id)
    }

    pub fn list_agent_automations(&self, agent_id: &str) -> Result<Vec<AutomationRecord>, String> {
        Ok(surface_automations(&self.automation_store_for(agent_id)?))
    }

    pub fn set_agent_automation_enabled(
        &self,
        agent_id: &str,
        automation_id: &str,
        enabled: bool,
    ) -> Result<Vec<AutomationRecord>, String> {
        let store = self.automation_store_for(agent_id)?;
        let _ = store
            .set_enabled(automation_id, enabled)
            .map_err(|error| error.to_string())?;
        Ok(surface_automations(&store))
    }

    pub fn create_agent_automation(
        &self,
        agent_id: &str,
        spec: &AutomationSpec,
    ) -> Result<Vec<AutomationRecord>, String> {
        let store = self.automation_store_for(agent_id)?;
        let _ = store
            .upsert(spec, now_ms())
            .map_err(|error| error.to_string())?;
        Ok(surface_automations(&store))
    }

    pub fn update_agent_automation(
        &self,
        agent_id: &str,
        automation_id: &str,
        spec: &AutomationSpec,
    ) -> Result<Vec<AutomationRecord>, String> {
        let store = self.automation_store_for(agent_id)?;
        let _ = store
            .update(automation_id, spec)
            .map_err(|error| error.to_string())?;
        Ok(surface_automations(&store))
    }

    pub fn remove_agent_automation(
        &self,
        agent_id: &str,
        automation_id: &str,
    ) -> Result<Vec<AutomationRecord>, String> {
        let store = self.automation_store_for(agent_id)?;
        let _ = store
            .remove(automation_id)
            .map_err(|error| error.to_string())?;
        Ok(surface_automations(&store))
    }

    pub fn workflow_store_for(&self, agent_id: &str) -> Result<FileWorkflowStore, String> {
        self.production.open_workflow_store(agent_id)
    }

    pub fn list_agent_workflows(&self, agent_id: &str) -> Result<Vec<WorkflowRecord>, String> {
        Ok(surface_workflows(&self.workflow_store_for(agent_id)?))
    }

    pub fn get_agent_workflow(
        &self,
        agent_id: &str,
        workflow_id: &str,
    ) -> Result<Option<WorkflowRecord>, String> {
        Ok(self.workflow_store_for(agent_id)?.get(workflow_id))
    }

    pub fn create_agent_workflow(
        &self,
        agent_id: &str,
        spec: &WorkflowSpec,
    ) -> Result<Vec<WorkflowRecord>, String> {
        let store = self.workflow_store_for(agent_id)?;
        let _ = store.create(spec)?;
        Ok(surface_workflows(&store))
    }

    pub fn update_agent_workflow(
        &self,
        agent_id: &str,
        workflow_id: &str,
        spec: &WorkflowSpec,
    ) -> Result<Vec<WorkflowRecord>, String> {
        let store = self.workflow_store_for(agent_id)?;
        let _ = store.update(workflow_id, spec)?;
        Ok(surface_workflows(&store))
    }

    pub fn set_agent_workflow_enabled(
        &self,
        agent_id: &str,
        workflow_id: &str,
        enabled: bool,
    ) -> Result<Vec<WorkflowRecord>, String> {
        let store = self.workflow_store_for(agent_id)?;
        let _ = store.set_enabled_for_agent(workflow_id, enabled)?;
        Ok(surface_workflows(&store))
    }

    pub fn remove_agent_workflow(
        &self,
        agent_id: &str,
        workflow_id: &str,
    ) -> Result<Vec<WorkflowRecord>, String> {
        let store = self.workflow_store_for(agent_id)?;
        let _ = store.remove(workflow_id)?;
        Ok(surface_workflows(&store))
    }

    pub fn import_agent_workflow_markdown(
        &self,
        agent_id: &str,
        markdown: &str,
        fallback_name: Option<&str>,
    ) -> Result<WorkflowImportResponse, String> {
        let store = self.workflow_store_for(agent_id)?;
        let result = match store.import_markdown(markdown, fallback_name)? {
            Some((id, name)) => WorkflowImportBatch {
                imported: vec![WorkflowImportResult { id, name }],
                skipped: Vec::new(),
            },
            None => WorkflowImportBatch {
                imported: Vec::new(),
                skipped: vec![WorkflowImportSkipped {
                    source: "pasted skill".into(),
                    reason: "empty or invalid".into(),
                }],
            },
        };
        Ok(WorkflowImportResponse {
            workflows: surface_workflows(&store),
            result,
        })
    }

    pub fn import_agent_workflow_source(
        &self,
        agent_id: &str,
        source: &str,
        fallback_name: Option<&str>,
    ) -> Result<WorkflowImportResponse, String> {
        let store = self.workflow_store_for(agent_id)?;
        let result = match store.import_live_source(source, fallback_name)? {
            Some((id, name)) => WorkflowImportBatch {
                imported: vec![WorkflowImportResult { id, name }],
                skipped: Vec::new(),
            },
            None => WorkflowImportBatch {
                imported: Vec::new(),
                skipped: vec![WorkflowImportSkipped {
                    source: source.to_string(),
                    reason: "could not link".into(),
                }],
            },
        };
        Ok(WorkflowImportResponse {
            workflows: surface_workflows(&store),
            result,
        })
    }

    pub fn port_agent_local_skills_from(
        &self,
        agent_id: &str,
        home_dir: &Path,
        cwd: &Path,
    ) -> Result<WorkflowImportResponse, String> {
        let store = self.workflow_store_for(agent_id)?;
        let result = store.port_local_skills(home_dir, cwd)?;
        Ok(WorkflowImportResponse {
            workflows: surface_workflows(&store),
            result,
        })
    }

    pub fn port_agent_local_skills(
        &self,
        agent_id: &str,
    ) -> Result<WorkflowImportResponse, String> {
        let home = std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."));
        let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        self.port_agent_local_skills_from(agent_id, &home, &cwd)
    }

    pub fn list_all_automations_from(
        &self,
        definitions_only: bool,
    ) -> Result<Vec<AgentAutomationEntry>, String> {
        let mut result = Vec::new();
        for agent_id in self.list_agent_ids()? {
            let Ok(store) = self.automation_store_for(&agent_id) else {
                continue;
            };
            let automations = if definitions_only {
                store.list_definitions()
            } else {
                store.list()
            };
            result.extend(automations.into_iter().map(|automation| AgentAutomationEntry {
                agent_id: agent_id.clone(),
                automation,
            }));
        }
        Ok(result)
    }

    pub fn list_all_automations(&self) -> Result<Vec<AgentAutomationEntry>, String> {
        self.list_all_automations_from(false)
    }

    pub fn list_all_automation_definitions(&self) -> Result<Vec<AgentAutomationEntry>, String> {
        self.list_all_automations_from(true)
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

    pub fn open_channel_store(&self, agent_id: &str) -> Result<FileChannelStore, String> {
        self.production.open_channel_store(agent_id)
    }

    pub fn list_agent_channels(&self, agent_id: &str) -> Result<Vec<ChannelConnection>, String> {
        self.production.list_agent_channels(agent_id)
    }

    pub fn list_channel_configs(&self, agent_id: &str) -> Result<Vec<ChannelConfig>, String> {
        self.production.list_channel_configs(agent_id)
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

fn surface_automations(store: &FileAutomationStore) -> Vec<AutomationRecord> {
    store
        .list()
        .into_iter()
        .take(AUTOMATION_UI_LIMIT)
        .collect()
}

fn surface_workflows(store: &FileWorkflowStore) -> Vec<WorkflowRecord> {
    const WORKFLOW_UI_LIMIT: usize = 100;
    let mut managed = Vec::new();
    let mut user = Vec::new();
    for workflow in store.list_all() {
        if matches!(workflow.source.as_str(), "managed" | "plugin") {
            managed.push(workflow);
        } else {
            user.push(workflow);
        }
    }
    managed.extend(user.into_iter().take(WORKFLOW_UI_LIMIT));
    managed
}

fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as f64
}
