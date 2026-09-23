use std::collections::HashSet;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Value, json};

use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;

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

pub struct ProductionAgentLifecycle {
    store: SandAgentSessionStore,
}

impl ProductionAgentLifecycle {
    pub fn new(production: Arc<ProductionSessionWorkers>) -> Self {
        Self {
            store: SandAgentSessionStore::new(production),
        }
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
            self.store.delete_session(agent_id)?;
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

            self.store
                .clear_active_agent_id()
                .map_err(|error| error.to_string())?;
            return Ok(json!({ "transcript": [] }));
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
    let lifecycle = ProductionAgentLifecycle::new(Arc::clone(production));
    let result = match method {
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
