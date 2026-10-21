use std::collections::BTreeMap;

use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const ADVICE: &str = "This same tool with identical arguments has returned the same failure again. Repetition is not evidence of progress. Use the actual diagnostic to choose a justified new action or report the blocker and finish. Reconcile any uncertain write before retrying. This observation does not cancel the turn.";

fn meaningful(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(value) => *value,
        Value::Number(value) => value.as_f64().is_some_and(|value| value != 0.0),
        Value::String(value) => !value.trim().is_empty(),
        Value::Array(value) => !value.is_empty(),
        Value::Object(value) => !value.is_empty(),
    }
}

pub(super) fn is_execution_failure(result: &Value) -> bool {
    // Refusals are policy decisions, not evidence that execution failed.
    if ["rejected", "permissionDenied", "guardrail"].iter()
        .any(|key| result.get(key).is_some_and(meaningful)) {
        return false;
    }
    result.get("isError").and_then(Value::as_bool) == Some(true)
        || result.get("error").is_some_and(meaningful)
        || result.get("failed").and_then(Value::as_u64).is_some_and(|n| n > 0)
        || result.get("status").and_then(Value::as_str).is_some_and(|s| matches!(s, "error" | "failed"))
}

fn digest(value: &Value) -> String {
    // serde_json's map is key ordered in this build. Explicit recursion keeps
    // argument identity stable if preserve_order is enabled by another consumer.
    fn canonical(value: &Value) -> Value {
        match value {
            Value::Object(map) => {
                let ordered: BTreeMap<_, _> = map.iter().map(|(k,v)| (k.clone(), canonical(v))).collect();
                Value::Object(ordered.into_iter().collect())
            }
            Value::Array(values) => Value::Array(values.iter().map(canonical).collect()),
            other => other.clone(),
        }
    }
    format!("{:x}", Sha256::digest(serde_json::to_vec(&canonical(value)).unwrap_or_default()))
}

#[derive(Default)]
pub(super) struct ToolFailureObservations {
    failures: BTreeMap<String, (String, usize)>,
}

impl ToolFailureObservations {
    pub fn observe(&mut self, name: &str, args: &Value, result: &mut Value) {
        let key = digest(&json!([name, args]));
        if !is_execution_failure(result) {
            self.failures.remove(&key);
            return;
        }
        if self.failures.len() >= 64 && !self.failures.contains_key(&key) { return; }
        // The Box resource adapter already supplies this same observation.
        // Normalize its advisory for identity, and never add a second one.
        let mut diagnostic = result.clone();
        let existing_advice = diagnostic.get("error").and_then(Value::as_str)
            .and_then(|s| s.find("\n[Tool observation:"));
        if let Some(index) = existing_advice {
            let original = diagnostic["error"].as_str().unwrap()[..index].to_string();
            diagnostic["error"] = Value::String(original);
        }
        let fingerprint = digest(&diagnostic);
        let (previous, count) = self.failures.entry(key).or_insert((fingerprint.clone(), 0));
        if *previous != fingerprint { *previous = fingerprint; *count = 0; }
        *count = count.saturating_add(1);
        if *count == 2 && existing_advice.is_none() {
            if let Some(object) = result.as_object_mut() {
                object.insert("toolObservation".into(), Value::String(ADVICE.into()));
            }
        }
    }
}
