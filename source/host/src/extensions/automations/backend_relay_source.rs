use serde_json::Value;

use crate::automations::automation_trigger::listener_matches_event;

#[derive(Debug, Clone, PartialEq)]
pub struct BackendRelaySource {
    kind: String,
    listeners: Vec<Value>,
    started: bool,
}

impl BackendRelaySource {
    pub fn new(kind: impl Into<String>) -> Self {
        Self { kind: kind.into(), listeners: Vec::new(), started: false }
    }

    pub fn kind(&self) -> &str { &self.kind }
    pub fn is_started(&self) -> bool { self.started }
    pub fn listeners(&self) -> &[Value] { &self.listeners }

    pub fn set_listeners(&mut self, listeners: Vec<Value>) {
        self.listeners = listeners;
    }

    pub fn start(&mut self) { self.started = true; }
    pub fn stop(&mut self) { self.started = false; }

    pub fn accepts(
        &self,
        event: &Value,
        platform_matched: bool,
        admit_missing_subject: bool,
    ) -> bool {
        self.started && self.listeners.iter().any(|listener| {
            listener.get("type").and_then(Value::as_str) == Some(self.kind.as_str())
                && listener_matches_event(
                    listener,
                    event,
                    platform_matched,
                    admit_missing_subject,
                )
        })
    }
}
