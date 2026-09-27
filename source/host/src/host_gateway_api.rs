use std::collections::{HashMap, VecDeque};

use serde_json::Value;

pub const HOST_CAPABILITIES: [&str; 2] = ["orderedReplicasV1", "sendAcceptanceV1"];
pub const CREATE_AGENT_NONCE_LEDGER_CAP: usize = 64;
pub const DISABLE_SEND_ACCEPT_RETURN_ENV: &str = "SAND_DISABLE_SEND_ACCEPT_RETURN";

pub fn is_sand_agent_purpose(value: &str) -> bool {
    matches!(value, "disk-saver" | "plugin-auth")
}

pub fn sanitize_template_id(value: &str) -> Option<&str> {
    let valid_len = (1..=64).contains(&value.len());
    let valid_chars = value
        .bytes()
        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-');
    (valid_len && valid_chars).then_some(value)
}

pub fn sanitize_create_agent_args(args: &Value) -> Value {
    let mut projected = args.clone();
    let Some(object) = projected.as_object_mut() else {
        return projected;
    };
    if object
        .get("purpose")
        .and_then(Value::as_str)
        .is_some_and(|purpose| !is_sand_agent_purpose(purpose))
    {
        object.remove("purpose");
    }
    if object
        .get("templateId")
        .and_then(Value::as_str)
        .is_some_and(|template_id| sanitize_template_id(template_id).is_none())
    {
        object.remove("templateId");
    }
    projected
}

#[derive(Debug, Default)]
pub struct CreateAgentNonceLedger<T> {
    order: VecDeque<String>,
    values: HashMap<String, T>,
}

impl<T> CreateAgentNonceLedger<T> {
    pub fn insert(&mut self, nonce: impl Into<String>, value: T) {
        let nonce = nonce.into();
        if !self.values.contains_key(&nonce) {
            self.order.push_back(nonce.clone());
        }
        self.values.insert(nonce, value);
        while self.order.len() > CREATE_AGENT_NONCE_LEDGER_CAP {
            if let Some(expired) = self.order.pop_front() {
                self.values.remove(&expired);
            }
        }
    }

    pub fn get(&self, nonce: &str) -> Option<&T> {
        self.values.get(nonce)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}
