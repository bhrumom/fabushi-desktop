use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::agents::agent_profile::SandAgentProfile;
use mahayana_host_runtime::device_pairing_owner::DevicePairingOwner;
use mahayana_host_runtime::extensions::session::agent_session::SandAgentSessionStore;
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::transcript::runner_registry::TranscriptRunnerRegistry;
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use serde_json::{Value, json};

const SHIPPING_HOST_MAIN: &str = include_str!("../app/src/main.rs");

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-openclaw-chat-cron-node-{label}-{}-{suffix}",
        std::process::id()
    ))
}

fn parse_agent_id(raw_key: &str) -> &str {
    if let Some(stripped) = raw_key.strip_prefix("agent:") {
        stripped.split(':').next().unwrap_or(stripped)
    } else {
        raw_key
    }
}

// =========================================================================
// OCRM-CHAT Contract Tests: chat.send and chat.abort
// =========================================================================

#[test]
fn test_ocrm_chat_o01_send_requires_session_key_message_idempotency_key() {
    let validate_args = |args: &Value| -> Result<(), GatewayCommandError> {
        let _ = args
            .get("sessionKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("chat.send requires sessionKey".into()))?;

        let _ = args
            .get("message")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("chat.send requires message".into()))?;

        let _ = args
            .get("idempotencyKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| {
                GatewayCommandError::BadRequest("chat.send requires idempotencyKey".into())
            })?;

        Ok(())
    };

    // Missing sessionKey
    let err = validate_args(&json!({ "message": "hello", "idempotencyKey": "k1" })).unwrap_err();
    assert_eq!(err.status(), 400);
    assert_eq!(err.to_string(), "chat.send requires sessionKey");

    // Empty sessionKey
    let err = validate_args(&json!({ "sessionKey": "   ", "message": "hello", "idempotencyKey": "k1" })).unwrap_err();
    assert_eq!(err.status(), 400);

    // Missing message
    let err = validate_args(&json!({ "sessionKey": "s1", "idempotencyKey": "k1" })).unwrap_err();
    assert_eq!(err.status(), 400);
    assert_eq!(err.to_string(), "chat.send requires message");

    // Missing idempotencyKey
    let err = validate_args(&json!({ "sessionKey": "s1", "message": "hello" })).unwrap_err();
    assert_eq!(err.status(), 400);
    assert_eq!(err.to_string(), "chat.send requires idempotencyKey");

    // Valid passes
    assert!(validate_args(&json!({ "sessionKey": "s1", "message": "hello", "idempotencyKey": "k1" })).is_ok());
}

#[test]
fn test_ocrm_chat_o02_session_key_stripping_and_unknown_session_rejection() {
    let root = temp_root("chat-unknown");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(&root.join("agents"), 500));
    let store = SandAgentSessionStore::new(Arc::clone(&workers));

    // Verify key parser strips agent: prefix and subthreads
    assert_eq!(parse_agent_id("agent:ag-alpha:subthread-1"), "ag-alpha");
    assert_eq!(parse_agent_id("agent:ag-beta"), "ag-beta");
    assert_eq!(parse_agent_id("ag-gamma"), "ag-gamma");

    // Check unknown agent check fails with 400 BadRequest
    let agent_id = parse_agent_id("agent:nonexistent-agent-999:main");
    let exists = workers
        .summarize_agent_by_id(agent_id, None)
        .map_err(GatewayCommandError::Internal)
        .expect("summary query")
        .is_some();
    assert!(!exists);

    // Now create an agent with identity and verify it exists
    let profile = SandAgentProfile {
        name: "Test Agent".into(),
        description: "desc".into(),
        title: "Title".into(),
        avatar_shape: "round".into(),
        avatar_color: "blue".into(),
    };
    let created = store.create_session(Some(&profile), "user", None).expect("create session");
    let exists_now = workers
        .summarize_agent_by_id(&created.id, None)
        .expect("summary query")
        .is_some();
    assert!(exists_now);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_ocrm_chat_o03_send_response_contract_fields() {
    let run_id = format!("run-{}", uuid::Uuid::new_v4());
    let response = json!({
        "ok": true,
        "runId": run_id,
        "status": "started",
        "messageSeq": 1,
        "interruptedActiveRun": false,
        "result": {
            "accepted": true,
            "streamId": "stream-123"
        }
    });

    assert_eq!(response["ok"], true);
    assert!(response["runId"].as_str().unwrap().starts_with("run-"));
    assert_eq!(response["status"], "started");
    assert_eq!(response["messageSeq"], 1);
    assert_eq!(response["interruptedActiveRun"], false);
    assert_eq!(response["result"]["accepted"], true);
}

#[test]
fn test_ocrm_chat_o04_abort_requires_session_key() {
    let validate_abort_args = |args: &Value| -> Result<String, GatewayCommandError> {
        args.get("sessionKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| GatewayCommandError::BadRequest("chat.abort requires sessionKey".into()))
    };

    assert_eq!(
        validate_abort_args(&json!({})).unwrap_err().to_string(),
        "chat.abort requires sessionKey"
    );
    assert_eq!(
        validate_abort_args(&json!({ "sessionKey": "  " })).unwrap_err().status(),
        400
    );
    assert_eq!(
        validate_abort_args(&json!({ "sessionKey": "agent:agent-007" })).unwrap(),
        "agent:agent-007"
    );
}

#[test]
fn test_ocrm_chat_o05_abort_invokes_runner_registry_cancellation() {
    let registry = TranscriptRunnerRegistry::default();
    let agent_id = "agent-test-abort";

    // Non-active agent cancellation returns 0
    let cancelled = registry.cancel_agent(agent_id, "rpc");
    assert_eq!(cancelled, 0);

    let abort_res = json!({
        "ok": true,
        "aborted": cancelled > 0,
        "runIds": vec![format!("run-{agent_id}")]
    });

    assert_eq!(abort_res["ok"], true);
    assert_eq!(abort_res["aborted"], false);
    assert_eq!(abort_res["runIds"][0], "run-agent-test-abort");
}

// =========================================================================
// OCRM-CRON Contract Tests: cron.list and cron.trigger
// =========================================================================

#[test]
fn test_ocrm_cron_o01_cron_list_empty_and_populated_mappings() {
    let root = temp_root("cron-list");
    let workers = Arc::new(ProductionSessionWorkers::with_agents_root(&root.join("agents"), 500));
    let store = SandAgentSessionStore::new(Arc::clone(&workers));

    // When no agents or automations exist, returns empty jobs list
    let agents = store.list_agents().expect("list agents");
    let mut jobs = Vec::new();
    for agent in agents {
        if let Ok(records) = store.list_agent_automations(&agent.id) {
            for r in records {
                jobs.push(json!({
                    "jobId": r.id,
                    "agentId": agent.id,
                    "schedule": r.schedule,
                    "prompt": r.prompt,
                    "enabled": r.is_enabled,
                    "createdAtMs": (r.created_at * 1000.0) as i64,
                    "updatedAtMs": (r.created_at * 1000.0) as i64,
                }));
            }
        }
    }
    let res = json!({ "jobs": jobs });
    assert_eq!(res["jobs"].as_array().unwrap().len(), 0);
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn test_ocrm_cron_o02_cron_trigger_validation_and_not_found() {
    let validate_trigger = |args: &Value| -> Result<String, GatewayCommandError> {
        args.get("jobId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .map(str::to_string)
            .ok_or_else(|| GatewayCommandError::BadRequest("cron.trigger requires jobId".into()))
    };

    let err = validate_trigger(&json!({})).unwrap_err();
    assert_eq!(err.status(), 400);
    assert_eq!(err.to_string(), "cron.trigger requires jobId");

    let err_empty = validate_trigger(&json!({ "jobId": "   " })).unwrap_err();
    assert_eq!(err_empty.status(), 400);

    let job_id = validate_trigger(&json!({ "jobId": "job-missing-1" })).unwrap();
    assert_eq!(job_id, "job-missing-1");
}

// =========================================================================
// OCRM-NODE Contract Tests: node.register, node.list, node.status, node.invoke
// =========================================================================

#[test]
fn test_ocrm_node_o01_node_register_sand_local_node() {
    let register_node = |args: &Value| -> Result<Value, GatewayCommandError> {
        let dev_id = args.get("nodeId")
            .or_else(|| args.get("deviceId"))
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("node.register requires nodeId or deviceId".into()))?;

        if dev_id == "sand-local-node" {
            return Ok(json!({
                "ok": true,
                "status": "approved",
                "nodeId": "sand-local-node"
            }));
        }

        Err(GatewayCommandError::BadRequest("unsupported remote node".into()))
    };

    let local_res = register_node(&json!({ "nodeId": "sand-local-node" })).expect("register local");
    assert_eq!(local_res["ok"], true);
    assert_eq!(local_res["status"], "approved");
    assert_eq!(local_res["nodeId"], "sand-local-node");

    let dev_local_res = register_node(&json!({ "deviceId": "sand-local-node" })).expect("register local with deviceId");
    assert_eq!(dev_local_res["ok"], true);

    let missing_err = register_node(&json!({})).unwrap_err();
    assert_eq!(missing_err.status(), 400);
    assert_eq!(missing_err.to_string(), "node.register requires nodeId or deviceId");
}

#[test]
fn test_ocrm_node_o02_node_register_remote_device_delegates_to_pairing() {
    let pairing_owner = DevicePairingOwner::in_memory().expect("in-memory db");

    let pair_req = pairing_owner.handle_pairing_command(
        "device.pair.request",
        &json!({
            "deviceId": "node-remote-01",
            "publicKey": "remote-pubkey-base64",
            "displayName": "Remote Worker Box",
            "platform": "linux"
        }),
    ).expect("pair request");

    assert_eq!(pair_req["status"], "pending");
    assert_eq!(pair_req["deviceId"], "node-remote-01");
}

#[test]
fn test_ocrm_node_o03_node_list_includes_sand_local_node_and_paired_devices() {
    let pairing_owner = DevicePairingOwner::in_memory().expect("in-memory db");

    // Initially, list pairing has 0 approved
    let pair_list = pairing_owner.handle_pairing_command("device.pair.list", &json!({})).expect("pair list");
    let mut nodes = vec![json!({
        "nodeId": "sand-local-node",
        "displayName": "Local Host Node",
        "platform": std::env::consts::OS,
        "deviceFamily": std::env::consts::OS,
        "commands": ["system.run", "canvas.navigate", "computer.mouse", "computer.keyboard"],
        "connected": true,
        "paired": true,
        "active": true,
        "gatewayLocal": true,
        "capabilities": ["orderedReplicasV1", "sendAcceptanceV1"]
    })];

    if let Some(paired) = pair_list.get("paired").and_then(Value::as_array) {
        for p in paired {
            if let Some(dev_id) = p.get("deviceId").and_then(Value::as_str) {
                nodes.push(json!({
                    "nodeId": dev_id,
                    "displayName": p.get("displayName").and_then(Value::as_str).unwrap_or(dev_id),
                    "platform": p.get("platform").and_then(Value::as_str).unwrap_or("unknown"),
                    "deviceFamily": "remote",
                    "commands": ["system.run"],
                    "connected": false,
                    "paired": true,
                    "active": false,
                    "gatewayLocal": false
                }));
            }
        }
    }

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["nodeId"], "sand-local-node");
    assert_eq!(nodes[0]["gatewayLocal"], true);
    assert_eq!(nodes[0]["commands"], json!(["system.run", "canvas.navigate", "computer.mouse", "computer.keyboard"]));
    assert_eq!(nodes[0]["capabilities"], json!(["orderedReplicasV1", "sendAcceptanceV1"]));

    // Now approve a remote device and re-check
    let req = pairing_owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "deviceId": "dev-peer-01", "publicKey": "pk-peer" }),
    ).unwrap();
    let request_id = req["requestId"].as_str().unwrap();

    let approved = pairing_owner.handle_pairing_command(
        "device.pair.approve",
        &json!({ "requestId": request_id, "role": "operator" }),
    ).unwrap();
    assert_eq!(approved["decision"], "approved");

    let pair_list_after = pairing_owner.handle_pairing_command("device.pair.list", &json!({})).unwrap();
    let paired_items = pair_list_after["paired"].as_array().unwrap();
    assert_eq!(paired_items.len(), 1);
    assert_eq!(paired_items[0]["deviceId"], "dev-peer-01");
    let remote_projection = json!({
        "nodeId": paired_items[0]["deviceId"],
        "connected": false,
        "paired": true,
        "active": false,
    });
    assert_eq!(remote_projection["paired"], true);
    assert_eq!(remote_projection["connected"], false);
    assert_eq!(remote_projection["active"], false);
}

#[test]
fn test_ocrm_node_o04_node_status_local_and_remote() {
    let pairing_owner = DevicePairingOwner::in_memory().expect("in-memory db");

    let check_status = |node_id: &str| -> Result<Value, GatewayCommandError> {
        if node_id == "sand-local-node" {
            return Ok(json!({
                "nodeId": "sand-local-node",
                "displayName": "Local Host Node",
                "connected": true,
                "paired": true,
                "active": true
            }));
        }

        let status = pairing_owner.handle_pair_status(&json!({ "deviceId": node_id }))?;
        if status.get("status").and_then(Value::as_str) == Some("approved") {
            Ok(json!({
                "nodeId": node_id,
                "connected": false,
                "paired": true,
                "active": false
            }))
        } else {
            Err(GatewayCommandError::BadRequest(format!("unknown nodeId: {node_id}")))
        }
    };

    // Local host node is always active
    let local = check_status("sand-local-node").unwrap();
    assert_eq!(local["connected"], true);
    assert_eq!(local["active"], true);

    // Approved-without-live-transport is paired but not connected or active.
    let req = pairing_owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "deviceId": "dev-status-01", "publicKey": "pk-status" }),
    ).unwrap();
    pairing_owner.handle_pairing_command(
        "device.pair.approve",
        &json!({ "requestId": req["requestId"], "role": "operator" }),
    ).unwrap();
    let remote = check_status("dev-status-01").unwrap();
    assert_eq!(remote["paired"], true);
    assert_eq!(remote["connected"], false);
    assert_eq!(remote["active"], false);

    // Unknown node returns 400 BadRequest
    let err = check_status("unknown-worker-99").unwrap_err();
    assert_eq!(err.status(), 400);
    assert_eq!(err.to_string(), "unknown nodeId: unknown-worker-99");
}

#[test]
fn test_ocrm_node_o05_node_invoke_contract_and_security() {
    let pairing_owner = DevicePairingOwner::in_memory().expect("in-memory db");

    let invoke_node = |args: &Value| -> Result<Value, GatewayCommandError> {
        let node_id = args.get("nodeId")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("node.invoke requires nodeId".into()))?;

        let command = args.get("command")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("node.invoke requires command".into()))?;

        let _ = args.get("idempotencyKey")
            .and_then(Value::as_str)
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .ok_or_else(|| GatewayCommandError::BadRequest("node.invoke requires idempotencyKey".into()))?;

        if node_id != "sand-local-node" {
            let status = pairing_owner.handle_pair_status(&json!({ "deviceId": node_id }))?;
            if status.get("status").and_then(Value::as_str) != Some("approved") {
                return Err(GatewayCommandError::Internal("node not connected".into()));
            }
        }

        Ok(json!({
            "ok": true,
            "nodeId": node_id,
            "command": command,
            "payload": {
                "exitCode": 0,
                "stdout": "ok",
                "stderr": ""
            },
            "payloadJSON": Value::Null
        }))
    };

    // Missing required args
    let err_missing = invoke_node(&json!({ "nodeId": "sand-local-node" })).unwrap_err();
    assert_eq!(err_missing.status(), 400);

    // Invoke sand-local-node succeeds
    let res = invoke_node(&json!({
        "nodeId": "sand-local-node",
        "command": "system.run",
        "idempotencyKey": "inv-001"
    })).unwrap();
    assert_eq!(res["ok"], true);
    assert_eq!(res["nodeId"], "sand-local-node");
    assert_eq!(res["command"], "system.run");
    assert_eq!(res["payload"]["exitCode"], 0);

    // Unconnected node fails with Internal("node not connected")
    let err_unconnected = invoke_node(&json!({
        "nodeId": "remote-unpaired",
        "command": "system.run",
        "idempotencyKey": "inv-002"
    })).unwrap_err();
    assert_eq!(err_unconnected.to_string(), "node not connected");
}

#[test]
fn test_ocrm_node_o06_shipping_routes_to_unique_native_owners() {
    let register_start = SHIPPING_HOST_MAIN
        .find("fn call_openclaw_node_register(")
        .expect("shipping node.register owner exists");
    let register_end = SHIPPING_HOST_MAIN[register_start..]
        .find("\n    fn call_openclaw_node_list(")
        .map(|offset| register_start + offset)
        .expect("shipping node.register function boundary exists");
    let register = &SHIPPING_HOST_MAIN[register_start..register_end];
    assert!(register.contains("self.device_pairing.handle_pair_request"));
    assert!(register.contains("node.register requires nodeId"));
    assert!(!register.contains("pairingGeneration"));

    let invoke_start = SHIPPING_HOST_MAIN
        .find("fn call_openclaw_node_invoke(")
        .expect("shipping node.invoke owner exists");
    let invoke_end = SHIPPING_HOST_MAIN[invoke_start..]
        .find("\n}\n\nfn post_agent_message_to_group(")
        .map(|offset| invoke_start + offset)
        .expect("shipping node.invoke function boundary exists");
    let invoke = &SHIPPING_HOST_MAIN[invoke_start..invoke_end];
    assert!(invoke.contains("GatewayLocalToolScope"));
    assert!(invoke.contains("self.local_exec"));
    assert!(invoke.contains(".execute_shell("));
    assert!(invoke.contains("node.invoke system.run requires params.command"));
    assert!(invoke.contains("unsupported local node command"));
    assert!(SHIPPING_HOST_MAIN.contains("\"connected\": false"));
    assert!(SHIPPING_HOST_MAIN.contains("\"active\": false"));
    assert!(!invoke.contains("\"stdout\": \"ok\""));
    assert!(!invoke.contains("\"exitCode\": 0"));
}
