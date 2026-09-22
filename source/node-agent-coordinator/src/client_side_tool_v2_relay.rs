use std::collections::HashMap;
use serde_json::Value;
use crate::protocol::Failure;

#[derive(Debug, Default)]
pub struct ClientSideToolV2Relay { pending: HashMap<String, String> }

impl ClientSideToolV2Relay {
    pub fn begin(&mut self, call_id: impl Into<String>, tool_name: impl Into<String>) -> Result<(), Failure> {
        let call_id = call_id.into();
        if call_id.trim().is_empty() || self.pending.contains_key(&call_id) {
            return Err(Failure::new("TOOL_RELAY_DUPLICATE", "tool call id is empty or already pending"));
        }
        self.pending.insert(call_id, tool_name.into());
        Ok(())
    }
    pub fn settle(&mut self, call_id: &str, output: Value) -> Result<(String, Value), Failure> {
        let tool = self.pending.remove(call_id).ok_or_else(|| Failure::new(
            "TOOL_RELAY_UNKNOWN_CALL", format!("unknown client-side tool call {call_id}")
        ))?;
        Ok((tool, output))
    }
}
