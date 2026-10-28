//! Strict node-worker supervisor identity and receipt admission.
//!
//! This module owns correlation validation only. Process/container lifecycle,
//! durable launch receipts, cleanup, and worker side effects remain with the
//! existing worker/supervisor owners.

use serde_json::Value;

pub const WORKER_RESULT_JSON_MAX_BYTES: usize = 64 * 1024;
pub const WORKER_ERROR_TEXT_MAX_BYTES: usize = 4 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeWorkerSupervisorIdentity {
    pub launch_id: String,
    pub plan_hash: String,
    pub environment_id: String,
    pub session_id: String,
    pub owner_epoch: u64,
    pub placement_generation: u64,
    pub run_id: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NodeWorkerSupervisorReceiptState {
    Pending,
    Running,
    Completed { result_json: String },
    Failed { error_text: String },
    Interrupted { error_text: String },
    Cancelled { error_text: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NodeWorkerSupervisorReceipt {
    pub identity: NodeWorkerSupervisorIdentity,
    pub state: NodeWorkerSupervisorReceiptState,
}

pub fn parse_worker_identity(value: &Value) -> Result<NodeWorkerSupervisorIdentity, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "INVALID_REQUEST: worker identity must be an object".to_string())?;
    require_exact_keys(
        object,
        &[
            "launchId",
            "planHash",
            "environmentId",
            "sessionId",
            "ownerEpoch",
            "placementGeneration",
            "runId",
        ],
    )?;
    parse_identity_fields(object)
}

fn parse_identity_fields(object: &serde_json::Map<String, Value>) -> Result<NodeWorkerSupervisorIdentity, String> {
    Ok(NodeWorkerSupervisorIdentity {
        launch_id: identifier(object, "launchId")?,
        plan_hash: plan_hash(object, "planHash")?,
        environment_id: identifier(object, "environmentId")?,
        session_id: identifier(object, "sessionId")?,
        owner_epoch: non_negative_integer(object, "ownerEpoch")?,
        placement_generation: non_negative_integer(object, "placementGeneration")?,
        run_id: identifier(object, "runId")?,
    })
}

pub fn parse_worker_receipt(value: &Value) -> Result<NodeWorkerSupervisorReceipt, String> {
    let object = value
        .as_object()
        .ok_or_else(|| "INVALID_REQUEST: worker receipt must be an object".to_string())?;
    let identity = parse_identity_fields(object)?;
    let state = object
        .get("state")
        .and_then(Value::as_str)
        .ok_or_else(|| "INVALID_REQUEST: worker receipt state is required".to_string())?;
    let state = match state {
        "pending" => {
            require_exact_keys(object, &identity_keys_with("state"))?;
            NodeWorkerSupervisorReceiptState::Pending
        }
        "running" => {
            require_exact_keys(object, &identity_keys_with("state"))?;
            NodeWorkerSupervisorReceiptState::Running
        }
        "completed" => {
            require_exact_keys(object, &identity_keys_with_two("state", "resultJson"))?;
            NodeWorkerSupervisorReceiptState::Completed {
                result_json: bounded_result_json(object.get("resultJson"))?,
            }
        }
        "failed" | "interrupted" | "cancelled" => {
            require_exact_keys(object, &identity_keys_with("state", "errorText"))?;
            let error_text = bounded_error_text(object.get("errorText"))?;
            match state {
                "failed" => NodeWorkerSupervisorReceiptState::Failed { error_text },
                "interrupted" => NodeWorkerSupervisorReceiptState::Interrupted { error_text },
                _ => NodeWorkerSupervisorReceiptState::Cancelled { error_text },
            }
        }
        _ => return Err("INVALID_REQUEST: invalid worker receipt state".into()),
    };
    Ok(NodeWorkerSupervisorReceipt { identity, state })
}

pub fn worker_turn_matches_identity(
    receipt: &NodeWorkerSupervisorIdentity,
    expected: &NodeWorkerSupervisorIdentity,
) -> bool {
    receipt == expected
}

fn identifier(object: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    let value = object
        .get(key)
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.len() <= 256 && value.trim() == *value && !value.contains('\0'))
        .ok_or_else(|| format!("INVALID_REQUEST: {key} must be a bounded identifier"))?;
    Ok(value.to_string())
}

fn plan_hash(object: &serde_json::Map<String, Value>, key: &str) -> Result<String, String> {
    let value = identifier(object, key)?;
    if value.len() != 64 || !value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase()) {
        return Err("INVALID_REQUEST: planHash must be 64 lowercase hexadecimal characters".into());
    }
    Ok(value)
}

fn non_negative_integer(object: &serde_json::Map<String, Value>, key: &str) -> Result<u64, String> {
    object
        .get(key)
        .and_then(Value::as_u64)
        .ok_or_else(|| format!("INVALID_REQUEST: {key} must be a non-negative integer"))
}

fn bounded_result_json(value: Option<&Value>) -> Result<String, String> {
    let value = value
        .and_then(Value::as_str)
        .filter(|value| !value.is_empty() && value.as_bytes().len() <= WORKER_RESULT_JSON_MAX_BYTES)
        .ok_or_else(|| "INVALID_REQUEST: resultJson must be bounded JSON".to_string())?;
    let parsed: Value = serde_json::from_str(value)
        .map_err(|_| "INVALID_REQUEST: resultJson must be valid JSON".to_string())?;
    if !parsed.is_object() {
        return Err("INVALID_REQUEST: resultJson must be a JSON object".into());
    }
    Ok(value.to_owned())
}

fn bounded_error_text(value: Option<&Value>) -> Result<String, String> {
    let value = value
        .and_then(Value::as_str)
        .filter(|value| {
            !value.is_empty()
                && value.as_bytes().len() <= WORKER_ERROR_TEXT_MAX_BYTES
                && !value.contains('\r')
                && !value.contains('\n')
        })
        .ok_or_else(|| "INVALID_REQUEST: errorText must be bounded single-line text".to_string())?;
    Ok(value.to_string())
}

fn identity_keys_with(extra: &str) -> Vec<&str> {
    vec![
        "launchId",
        "planHash",
        "environmentId",
        "sessionId",
        "ownerEpoch",
        "placementGeneration",
        "runId",
        extra,
    ]
}

fn identity_keys_with_two(first: &str, second: &str) -> Vec<&str> {
    let mut keys = identity_keys_with(first);
    keys.push(second);
    keys
}

fn require_exact_keys(object: &serde_json::Map<String, Value>, keys: &[&str]) -> Result<(), String> {
    if object.len() != keys.len() || object.keys().any(|key| !keys.contains(&key.as_str())) {
        return Err("INVALID_REQUEST: unsupported worker protocol fields".into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{parse_worker_receipt, parse_worker_identity, worker_turn_matches_identity};
    use serde_json::json;

    fn identity() -> serde_json::Value {
        json!({
            "launchId":"launch-1",
            "planHash":"0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
            "environmentId":"env-1",
            "sessionId":"session-1",
            "ownerEpoch":2,
            "placementGeneration":3,
            "runId":"run-1"
        })
    }

    #[test]
    fn identity_is_exactly_bound_and_unknown_fields_fail() {
        let parsed = parse_worker_identity(&identity()).expect("identity");
        assert_eq!(parsed.owner_epoch, 2);
        let mut unknown = identity();
        unknown.as_object_mut().expect("object").insert("extra".into(), json!(true));
        assert!(parse_worker_identity(&unknown).is_err());
    }

    #[test]
    fn receipts_are_bounded_and_match_all_generation_identity_fields() {
        let mut receipt = identity();
        receipt.as_object_mut().expect("object").extend([
            ("state".into(), json!("completed")),
            ("resultJson".into(), json!(r#"{"ok":true}"#)),
        ]);
        let parsed = parse_worker_receipt(&receipt).expect("receipt");
        assert!(matches!(parsed.state, super::NodeWorkerSupervisorReceiptState::Completed { .. }));
        assert!(worker_turn_matches_identity(&parsed.identity, &parse_worker_identity(&identity()).expect("identity")));
    }
}
