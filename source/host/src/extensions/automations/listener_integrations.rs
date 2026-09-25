use std::collections::BTreeMap;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListenerConnectionState {
    pub kind: String,
    pub connected: bool,
}

#[derive(Debug, Clone, Default)]
pub struct ListenerIntegrations {
    states: BTreeMap<String, bool>,
}

impl ListenerIntegrations {
    pub fn set_connected(&mut self, kind: impl Into<String>, connected: bool) {
        self.states.insert(kind.into(), connected);
    }

    pub fn is_connected(&self, kind: &str) -> Option<bool> {
        self.states.get(kind).copied()
    }

    pub fn connection_states(&self) -> Vec<ListenerConnectionState> {
        self.states.iter().map(|(kind, connected)| ListenerConnectionState {
            kind: kind.clone(),
            connected: *connected,
        }).collect()
    }

    pub fn listener_kind(listener: &Value) -> Option<&str> {
        match listener.get("type").and_then(Value::as_str)? {
            "slack" => Some("slack"),
            "github" => Some("github"),
            "microsoftTeams" => Some("microsoftTeams"),
            "linear" => Some("linear"),
            "sentry" => Some("sentry"),
            "pagerduty" => Some("pagerduty"),
            _ => None,
        }
    }

    pub fn listener_is_connected(&self, listener: &Value) -> Option<bool> {
        Self::listener_kind(listener).and_then(|kind| self.is_connected(kind))
    }
}
