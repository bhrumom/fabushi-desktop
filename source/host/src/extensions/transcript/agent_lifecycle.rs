use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

use crate::agents::agent_clone::{clone_agent_dir, clone_agent_display_name};
use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::FallbackSession;
use crate::extensions::session::gateway::{
    SessionGatewayError, optional_bool as session_optional_bool,
    optional_string as session_optional_string, parse_create_agent_profile,
    parse_profile_update,
};
use crate::transcript_mutation_events::publish_transcript_mutation;
use crate::extensions::session::production::ProductionSessionWorkers;

pub type AgentDeletionHook = Arc<dyn Fn(&str) -> Result<(), String> + Send + Sync + 'static>;

#[derive(Clone, Default)]
pub struct AgentDeletionRuntimeDeps {
    pub mark_deleting: Option<AgentDeletionHook>,
    pub clear_deleting: Option<AgentDeletionHook>,
    pub cancel_runner: Option<AgentDeletionHook>,
    pub forget_ack: Option<AgentDeletionHook>,
    pub sharing_departure: Option<AgentDeletionHook>,
    pub clear_trays: Option<AgentDeletionHook>,
    pub dispose_background_work: Option<AgentDeletionHook>,
    pub drain_runner: Option<AgentDeletionHook>,
    pub release_box: Option<AgentDeletionHook>,
    pub forget_handoff: Option<AgentDeletionHook>,
    pub clear_pending_wakes: Option<AgentDeletionHook>,
}

impl AgentDeletionRuntimeDeps {
    fn mark_deleting(&self, agent_id: &str) -> Result<(), String> {
        if let Some(mark_deleting) = self.mark_deleting.as_ref() {
            mark_deleting(agent_id)?;
        }
        Ok(())
    }

    fn clear_deleting(&self, agent_id: &str) -> Result<(), String> {
        if let Some(clear_deleting) = self.clear_deleting.as_ref() {
            clear_deleting(agent_id)?;
        }
        Ok(())
    }

    fn before_delete(&self, agent_id: &str) -> Result<(), String> {
        if let Some(cancel_runner) = self.cancel_runner.as_ref() {
            cancel_runner(agent_id)?;
        }
        if let Some(forget_ack) = self.forget_ack.as_ref() {
            forget_ack(agent_id)?;
        }
        if let Some(sharing_departure) = self.sharing_departure.as_ref() {
            sharing_departure(agent_id)?;
        }
        if let Some(clear_trays) = self.clear_trays.as_ref() {
            clear_trays(agent_id)?;
        }
        if let Some(dispose_background_work) = self.dispose_background_work.as_ref() {
            dispose_background_work(agent_id)?;
        }
        if let Some(drain_runner) = self.drain_runner.as_ref() {
            drain_runner(agent_id)?;
        }
        Ok(())
    }

    fn after_delete(&self, agent_id: &str) -> Result<(), String> {
        if let Some(release_box) = self.release_box.as_ref() {
            release_box(agent_id)?;
        }
        if let Some(forget_handoff) = self.forget_handoff.as_ref() {
            forget_handoff(agent_id)?;
        }
        if let Some(clear_pending_wakes) = self.clear_pending_wakes.as_ref() {
            clear_pending_wakes(agent_id)?;
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AgentLifecycleGatewayError {
    BadRequest(String),
    Internal(String),
}

impl AgentLifecycleGatewayError {
    fn bad(message: impl Into<String>) -> Self {
        Self::BadRequest(message.into())
    }

    fn internal(message: impl Into<String>) -> Self {
        Self::Internal(message.into())
    }
}

fn map_session_gateway_error(error: SessionGatewayError) -> AgentLifecycleGatewayError {
    match error {
        SessionGatewayError::BadRequest(message) => AgentLifecycleGatewayError::BadRequest(message),
        SessionGatewayError::Internal(message) => AgentLifecycleGatewayError::Internal(message),
    }
}

pub struct ProductionAgentLifecycle {
    store: SandAgentSessionStore,
    deletion_runtime: AgentDeletionRuntimeDeps,
}

impl ProductionAgentLifecycle {
    pub fn new(production: Arc<ProductionSessionWorkers>) -> Self {
        Self::with_deletion_runtime(production, AgentDeletionRuntimeDeps::default())
    }

    pub fn with_deletion_runtime(
        production: Arc<ProductionSessionWorkers>,
        deletion_runtime: AgentDeletionRuntimeDeps,
    ) -> Self {
        Self {
            store: SandAgentSessionStore::new(production),
            deletion_runtime,
        }
    }

    fn create_agent_from_args(
        &self,
        args: &Value,
        activate: bool,
    ) -> Result<Value, AgentLifecycleGatewayError> {
        let profile = parse_create_agent_profile(args).map_err(map_session_gateway_error)?;
        let origin = session_optional_string(args, "origin")
            .map_err(map_session_gateway_error)?
            .unwrap_or("user");
        let purpose =
            session_optional_string(args, "purpose").map_err(map_session_gateway_error)?;
        let introduction_suppressed = session_optional_bool(args, "isIntroductionSuppressed")
            .map_err(map_session_gateway_error)?
            .unwrap_or(false);

        let record = self
            .store
            .create_session(Some(&profile), origin, purpose)
            .map_err(AgentLifecycleGatewayError::internal)?;
        if introduction_suppressed {
            self.store
                .production()
                .set_agent_introduction_pending(&record.id, false)
                .map_err(AgentLifecycleGatewayError::internal)?;
        }
        if activate {
            self.store
                .mark_agent_viewed(&record.id, system_now_ms(), false)
                .map_err(AgentLifecycleGatewayError::internal)?;
            self.store
                .write_active_agent_id(&record.id)
                .map_err(|error| AgentLifecycleGatewayError::internal(error.to_string()))?;
        }

        let summary = self
            .store
            .summarize_agent_by_id(&record.id)
            .map_err(AgentLifecycleGatewayError::internal)?
            .ok_or_else(|| {
                AgentLifecycleGatewayError::internal("failed to summarize newly created agent")
            })?;
        let transcript = self
            .store
            .read_agent_transcript_entries(&record.id)
            .map_err(AgentLifecycleGatewayError::internal)?;
        serde_json::to_value(summary)
            .map_err(|error| AgentLifecycleGatewayError::internal(error.to_string()))
            .map(|agent| json!({ "agent": agent, "transcript": transcript }))
    }

    fn update_agent_from_args(
        &self,
        args: &Value,
    ) -> Result<Value, AgentLifecycleGatewayError> {
        let agent_id = required_string(args, "id")?;
        let update = parse_profile_update(args).map_err(map_session_gateway_error)?;
        let summary = self
            .store
            .update_agent_profile(agent_id, &update)
            .map_err(AgentLifecycleGatewayError::internal)?;
        serde_json::to_value(summary)
            .map_err(|error| AgentLifecycleGatewayError::internal(error.to_string()))
    }

    pub fn clone_agent(&self, source_id: &str) -> Result<Value, String> {
        let summary = self
            .store
            .list_agents()?
            .into_iter()
            .find(|agent| agent.id == source_id)
            .ok_or_else(|| "That agent no longer exists.".to_string())?;
        if summary.is_group {
            return Err("Groups can't be duplicated yet.".to_string());
        }

        let source_dir = self.store.get_agent_dir(source_id);
        let clone_name = clone_agent_display_name(&summary.name);
        let production = Arc::clone(self.store.production());
        let new_id = production.mint_agent_with(|new_id| {
            clone_agent_dir(
                &source_dir,
                &production.agents_root().join(new_id),
                new_id,
                &clone_name,
                production.busy_timeout_ms(),
            )
            .map_err(|error| error.to_string())?;
            Ok(new_id.to_string())
        })?;

        let opened = (|| {
            let _ = self.store.open_session(&new_id)?;
            let now = system_now_ms();
            let _ = self.store.mark_agent_viewed(&new_id, now, false)?;
            self.store
                .write_active_agent_id(&new_id)
                .map_err(|error| error.to_string())?;
            let mutation = Map::from_iter([
                (
                    "kind".to_string(),
                    Value::String("agent-needs-reindex".to_string()),
                ),
                ("agentId".to_string(), Value::String(new_id.clone())),
            ]);
            publish_transcript_mutation(&mutation);
            let agent = self
                .store
                .summarize_agent_by_id(&new_id)?
                .ok_or_else(|| "minted agent could not be summarized".to_string())?;
            let transcript = self.store.read_agent_transcript_entries(&new_id)?;
            serde_json::to_value(agent)
                .map_err(|error| error.to_string())
                .map(|agent| json!({ "agent": agent, "transcript": transcript }))
        })();

        if opened.is_err() {
            let _ = self.store.delete_session(&new_id);
        }
        opened
    }

    pub fn delete_agent(&self, agent_id: &str) -> Result<Value, String> {
        self.delete_agents(std::slice::from_ref(&agent_id.to_string()))
    }

    pub fn delete_agents(&self, agent_ids: &[String]) -> Result<Value, String> {
        let mut seen = HashSet::new();
        let ids = agent_ids
            .iter()
            .map(|id| id.trim())
            .filter(|id| !id.is_empty())
            .filter(|id| seen.insert((*id).to_string()))
            .map(ToOwned::to_owned)
            .collect::<Vec<_>>();
        if ids.is_empty() {
            return self.current_transcript();
        }

        let deleting = ids.iter().cloned().collect::<HashSet<_>>();
        let active_before = self.store.read_active_agent_id();

        for agent_id in &ids {
            self.deletion_runtime.mark_deleting(agent_id)?;
            let deleted = (|| {
                self.deletion_runtime.before_delete(agent_id)?;
                self.store.delete_session(agent_id)?;
                self.deletion_runtime.after_delete(agent_id)?;
                Ok::<(), String>(())
            })();
            if let Err(error) = deleted {
                if self.store.agent_exists(agent_id) {
                    let _ = self.deletion_runtime.clear_deleting(agent_id);
                }
                return Err(error);
            }
        }

        if active_before
            .as_ref()
            .is_some_and(|active| deleting.contains(active))
        {
            let successor = self
                .store
                .list_agent_record_ids()?
                .into_iter()
                .find(|candidate| !deleting.contains(candidate));

            if let Some(successor) = successor {
                let _ = self
                    .store
                    .mark_agent_viewed(&successor, system_now_ms(), false)?;
                self.store
                    .write_active_agent_id(&successor)
                    .map_err(|error| error.to_string())?;
                let transcript = self.store.read_agent_transcript_entries(&successor)?;
                return Ok(json!({ "transcript": transcript }));
            }

            match self.store.create_fallback_session() {
                Ok(fallback) => {
                    let fallback_id = match fallback {
                        FallbackSession::Existing(prepared) => prepared.agent_id,
                        FallbackSession::Created(record) => record.id,
                    };
                    let _ = self
                        .store
                        .mark_agent_viewed(&fallback_id, system_now_ms(), false)?;
                    self.store
                        .write_active_agent_id(&fallback_id)
                        .map_err(|error| error.to_string())?;
                    let transcript = self.store.read_agent_transcript_entries(&fallback_id)?;
                    return Ok(json!({ "transcript": transcript }));
                }
                Err(error) if error.contains("Agent limit of") => {
                    self.store
                        .clear_active_agent_id()
                        .map_err(|error| error.to_string())?;
                    return Ok(json!({ "transcript": [] }));
                }
                Err(error) => return Err(error),
            }
        }

        self.current_transcript()
    }

    fn current_transcript(&self) -> Result<Value, String> {
        let Some(active) = self.store.read_active_agent_id() else {
            return Ok(json!({ "transcript": [] }));
        };
        if !self.store.agent_exists(&active) {
            self.store
                .clear_active_agent_id()
                .map_err(|error| error.to_string())?;
            return Ok(json!({ "transcript": [] }));
        }
        Ok(json!({
            "transcript": self.store.read_agent_transcript_entries(&active)?
        }))
    }
}

pub fn dispatch_production_agent_lifecycle_gateway_call(
    production: &Arc<ProductionSessionWorkers>,
    method: &str,
    args: &Value,
) -> Option<Result<Value, AgentLifecycleGatewayError>> {
    dispatch_production_agent_lifecycle_gateway_call_with_runtime(
        production,
        &AgentDeletionRuntimeDeps::default(),
        method,
        args,
    )
}

pub fn dispatch_production_agent_lifecycle_gateway_call_with_runtime(
    production: &Arc<ProductionSessionWorkers>,
    deletion_runtime: &AgentDeletionRuntimeDeps,
    method: &str,
    args: &Value,
) -> Option<Result<Value, AgentLifecycleGatewayError>> {
    let lifecycle = ProductionAgentLifecycle::with_deletion_runtime(
        Arc::clone(production),
        deletion_runtime.clone(),
    );
    let result = match method {
        "createAgent" => lifecycle.create_agent_from_args(args, true),
        "createBackgroundAgent" => lifecycle.create_agent_from_args(args, false),
        "updateAgent" => lifecycle.update_agent_from_args(args),
        "duplicateAgent" => required_string(args, "id").and_then(|agent_id| {
            lifecycle
                .clone_agent(agent_id)
                .map_err(AgentLifecycleGatewayError::internal)
        }),
        "deleteAgent" => required_string(args, "id")
            .and_then(|agent_id| {
                lifecycle
                    .delete_agent(agent_id)
                    .map_err(AgentLifecycleGatewayError::internal)
            }),
        "deleteAgents" => parse_agent_ids(args)
            .and_then(|ids| {
                lifecycle
                    .delete_agents(&ids)
                    .map_err(AgentLifecycleGatewayError::internal)
            }),
        _ => return None,
    };
    Some(result)
}

fn parse_agent_ids(args: &Value) -> Result<Vec<String>, AgentLifecycleGatewayError> {
    let values = args
        .get("ids")
        .and_then(Value::as_array)
        .ok_or_else(|| AgentLifecycleGatewayError::bad("missing or invalid ids"))?;
    let mut ids = Vec::with_capacity(values.len());
    for value in values {
        let id = value
            .as_str()
            .map(str::trim)
            .filter(|id| !id.is_empty())
            .ok_or_else(|| AgentLifecycleGatewayError::bad("invalid ids entry"))?;
        ids.push(id.to_string());
    }
    Ok(ids)
}

fn required_string<'a>(
    args: &'a Value,
    field: &str,
) -> Result<&'a str, AgentLifecycleGatewayError> {
    args.get(field)
        .and_then(Value::as_str)
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or_else(|| {
            AgentLifecycleGatewayError::bad(format!("missing or invalid {field}"))
        })
}

fn system_now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs_f64()
        * 1000.0
}
