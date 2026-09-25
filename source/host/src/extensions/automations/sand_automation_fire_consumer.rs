use std::collections::VecDeque;

use serde_json::Value;

#[derive(Debug, Clone, PartialEq)]
pub struct AutomationFireEnvelope {
    pub agent_id: String,
    pub automation_id: String,
    pub event: Value,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AutomationFireFailure {
    pub agent_id: String,
    pub automation_id: String,
    pub error: String,
}

#[derive(Debug, Default)]
pub struct SandAutomationFireConsumer {
    queue: VecDeque<AutomationFireEnvelope>,
    firing: bool,
}

impl SandAutomationFireConsumer {
    pub fn enqueue(&mut self, envelope: AutomationFireEnvelope) {
        self.queue.push_back(envelope);
    }

    pub fn is_firing(&self) -> bool { self.firing }
    pub fn pending_len(&self) -> usize { self.queue.len() }

    pub fn drain(
        &mut self,
        mut fire: impl FnMut(&AutomationFireEnvelope) -> Result<(), String>,
    ) -> Vec<AutomationFireFailure> {
        if self.firing {
            return Vec::new();
        }
        self.firing = true;
        let mut failures = Vec::new();

        while let Some(envelope) = self.queue.pop_front() {
            if let Err(error) = fire(&envelope) {
                failures.push(AutomationFireFailure {
                    agent_id: envelope.agent_id.clone(),
                    automation_id: envelope.automation_id.clone(),
                    error,
                });
            }
        }

        self.firing = false;
        failures
    }
}
