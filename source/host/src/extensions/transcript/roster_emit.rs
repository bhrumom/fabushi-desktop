use std::sync::{Arc, Mutex};

use serde_json::{Value, json};

use crate::extensions::session::agent_session::SandAgentSessionStore;
use crate::extensions::session::production::ProductionSessionWorkers;

use super::production_runtime::ProductionTranscriptRuntime;
use super::replica_writer::ReplicaStamp;

pub const ROSTER_REPLICA_KEY: &str = "roster";

pub type RosterEventSink = Arc<dyn Fn(Value) + Send + Sync + 'static>;

#[derive(Debug, Default)]
struct RosterEmitState {
    cached_agent_summaries: Vec<Value>,
    roster_cache_seeded: bool,
}

pub struct ProductionRosterEmit {
    sessions: Arc<ProductionSessionWorkers>,
    transcript: Arc<ProductionTranscriptRuntime>,
    event_sink: RosterEventSink,
    state: Mutex<RosterEmitState>,
}

impl ProductionRosterEmit {
    pub fn new(
        sessions: Arc<ProductionSessionWorkers>,
        transcript: Arc<ProductionTranscriptRuntime>,
        event_sink: RosterEventSink,
    ) -> Self {
        Self {
            sessions,
            transcript,
            event_sink,
            state: Mutex::new(RosterEmitState::default()),
        }
    }

    pub fn emit_agents(&self) -> Result<(), String> {
        let active_agent_id = self.active_agent_id();
        let summaries = self
            .sessions
            .list_agent_summaries(active_agent_id.as_deref())?;
        let mut agents = serde_json::to_value(summaries)
            .map_err(|error| format!("could not encode roster summaries: {error}"))?;
        self.transcript.decorate_agent_summaries(&mut agents);
        let rows = agents.as_array().cloned().unwrap_or_default();
        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| "roster emit state mutex poisoned".to_string())?;
            state.cached_agent_summaries = rows.clone();
            state.roster_cache_seeded = true;
        }
        let ordered = stamp_json(self.transcript.next_replica_stamp(ROSTER_REPLICA_KEY));
        (self.event_sink)(json!({
            "channel": "agents",
            "payload": {
                "activeAgentId": active_agent_id.unwrap_or_default(),
                "agents": rows,
                "ordered": ordered,
                "coverage": { "kind": "complete-roster" }
            }
        }));
        Ok(())
    }

    pub fn emit_agent_update(&self, agent_id: &str) -> Result<(), String> {
        if agent_id.trim().is_empty() {
            return self.emit_agents();
        }
        let seeded = self
            .state
            .lock()
            .map(|state| state.roster_cache_seeded)
            .unwrap_or(false);
        if !seeded {
            return self.emit_agents();
        }

        let active_agent_id = self.active_agent_id();
        let Some(summary) = self
            .sessions
            .summarize_agent_by_id(agent_id, active_agent_id.as_deref())?
        else {
            return self.emit_agents();
        };
        let mut decorated = serde_json::to_value(vec![summary])
            .map_err(|error| format!("could not encode roster summary: {error}"))?;
        self.transcript.decorate_agent_summaries(&mut decorated);
        let Some(agent) = decorated.as_array().and_then(|rows| rows.first()).cloned() else {
            return self.emit_agents();
        };

        {
            let mut state = self
                .state
                .lock()
                .map_err(|_| "roster emit state mutex poisoned".to_string())?;
            if let Some(index) = state.cached_agent_summaries.iter().position(|row| {
                row.get("id").and_then(Value::as_str) == Some(agent_id)
            }) {
                state.cached_agent_summaries[index] = agent.clone();
            } else {
                state.cached_agent_summaries.push(agent.clone());
            }
        }

        let ordered = stamp_json(self.transcript.next_replica_stamp(ROSTER_REPLICA_KEY));
        (self.event_sink)(json!({
            "channel": "agent-upserted",
            "payload": {
                "activeAgentId": active_agent_id.unwrap_or_default(),
                "agent": agent,
                "ordered": ordered
            }
        }));
        Ok(())
    }

    pub fn publish_profile_changed(&self, agent_id: &str) {
        (self.event_sink)(json!({
            "channel": "profile-changed",
            "payload": { "agentId": agent_id }
        }));
    }

    pub fn publish_name_changed(&self, agent_id: &str, from: &str, to: &str) {
        (self.event_sink)(json!({
            "channel": "timeline",
            "payload": {
                "agentId": agent_id,
                "event": {
                    "type": "name-changed",
                    "from": from,
                    "to": to
                }
            }
        }));
    }

    pub fn cached_agent_summaries(&self) -> Vec<Value> {
        self.state
            .lock()
            .map(|state| state.cached_agent_summaries.clone())
            .unwrap_or_default()
    }

    pub fn cache_seeded(&self) -> bool {
        self.state
            .lock()
            .map(|state| state.roster_cache_seeded)
            .unwrap_or(false)
    }

    fn active_agent_id(&self) -> Option<String> {
        SandAgentSessionStore::new(Arc::clone(&self.sessions)).read_active_agent_id()
    }
}

fn stamp_json(stamp: ReplicaStamp) -> Value {
    json!({
        "replicaKey": stamp.replica_key,
        "epoch": stamp.epoch,
        "sequence": stamp.sequence
    })
}
