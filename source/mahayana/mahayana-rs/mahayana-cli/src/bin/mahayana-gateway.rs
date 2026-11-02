use mahayana_core::{ApprovalDecision, ApprovalId, OperationId, RuntimeCommand, RuntimeEvent};
use mahayana_host::{HostCreateConfig, MahayanaHost};
use mahayana_gateway::{GatewayRuntime, GatewayState, ReplayLimits, RpcFailure, dispatch_request};
use mahayana_gateway_peer::{PersistentServerRequestRegistry as ServerRequestRegistry, ResolvedServerRequest, ServerRequestResponse};
use mahayana_gateway_protocol::{
    ApprovalRequestPayload, GatewayEvent, GatewayEventEnvelope, JsonRpcEventNotification,
};
use serde_json::{Value, json};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn main() {
    if let Err(error) = run() {
        eprintln!("mahayana-gateway: {error}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let runtime = Arc::new(Mutex::new(RuntimeHandle::create()?));
    let state = Arc::new(Mutex::new(GatewayState::new(ReplayLimits::default())));
    let peer = Arc::new(Mutex::new(ServerRequestRegistry::default()));
    let output = Arc::new(Mutex::new(()));
    let shutdown = Arc::new(AtomicBool::new(false));

    let pump_runtime = Arc::clone(&runtime);
    let pump_state = Arc::clone(&state);
    let pump_peer = Arc::clone(&peer);
    let pump_output = Arc::clone(&output);
    let pump_shutdown = Arc::clone(&shutdown);
    let event_pump = thread::Builder::new()
        .name("mahayana-gateway-events".into())
        .spawn(move || {
            while !pump_shutdown.load(Ordering::Acquire) {
                let received = pump_runtime
                    .lock()
                    .map_err(|_| "runtime mutex poisoned".to_string())
                    .and_then(|runtime| runtime.receive(250));
                let raw_event = match received {
                    Ok(Some(event)) => event,
                    Ok(None) => continue,
                    Err(error) => {
                        eprintln!("mahayana-gateway runtime receive failed: {error}");
                        break;
                    }
                };
                let runtime_event = match serde_json::from_value::<RuntimeEvent>(raw_event) {
                    Ok(event) => event,
                    Err(error) => {
                        eprintln!("mahayana-gateway rejected malformed runtime event: {error}");
                        continue;
                    }
                };
                let projected = match pump_state.lock() {
                    Ok(mut state) => state.ingest_runtime_event(&runtime_event, now_ms()),
                    Err(_) => break,
                };
                let peer_frames = issue_server_requests(&pump_peer, &projected);
                for event in projected {
                    if write_notification(&pump_output, event).is_err() {
                        return;
                    }
                }
                for frame in peer_frames {
                    if write_value(&pump_output, &frame).is_err() {
                        return;
                    }
                }
            }
        })
        .map_err(|error| error.to_string())?;

    for line in io::stdin().lock().lines() {
        let line = line.map_err(|error| error.to_string())?;
        let line = line.trim();
        if line.is_empty() {
            continue;
        }
        let request = match serde_json::from_str::<Value>(line) {
            Ok(request) => request,
            Err(error) => {
                write_value(
                    &output,
                    &rpc_error(Value::Null, -32700, &format!("parse error: {error}")),
                )?;
                continue;
            }
        };

        if ServerRequestRegistry::is_response_frame(&request) {
            let resolved = peer
                .lock()
                .map_err(|_| "gateway peer mutex poisoned".to_string())?
                .resolve_response(&request);
            if let Some(resolved) = resolved {
                resolve_peer_request(&runtime, resolved)?;
            }
            continue;
        }

        if request.get("jsonrpc").and_then(Value::as_str) != Some("2.0") {
            let id = request.get("id").cloned().unwrap_or(Value::Null);
            write_value(&output, &rpc_error(id, -32600, "jsonrpc must be 2.0"))?;
            continue;
        }
        let id = request.get("id").cloned();
        let method = request
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default();
        if method.is_empty() {
            let id = id.unwrap_or(Value::Null);
            write_value(&output, &rpc_error(id, -32600, "method is required"))?;
            continue;
        }
        let params = request.get("params").cloned().unwrap_or_else(|| json!({}));
        let legacy_approval_request_id = if method == "approval.respond" {
            params
                .get("approval_id")
                .or_else(|| params.get("approvalId"))
                .or_else(|| params.get("requestId"))
                .and_then(Value::as_str)
                .map(ToOwned::to_owned)
        } else {
            None
        };

        // Runtime -> state is the shared lock order used by the event pump too.
        // Holding both through prompt.submit prevents a fast runtime event from
        // racing ahead of turn/session registration.
        let dispatched = {
            let mut runtime = runtime
                .lock()
                .map_err(|_| "runtime mutex poisoned".to_string())?;
            let mut state = state
                .lock()
                .map_err(|_| "gateway state mutex poisoned".to_string())?;
            dispatch_request(method, &params, &mut *runtime, &mut state, now_ms())
        };

        match dispatched {
            Ok(dispatched) => {
                // RPC acknowledgement is written before semantic events so a
                // client can bind its turn id before message.start arrives.
                if let Some(id) = id {
                    write_value(&output, &rpc_result(id, dispatched.result))?;
                }
                for event in dispatched.events {
                    write_notification(&output, event)?;
                }
                if let Some(request_id) = legacy_approval_request_id.as_deref() {
                    close_peer_approval(&peer, request_id)?;
                }
            }
            Err(RpcFailure { code, message }) => {
                if let Some(id) = id {
                    write_value(&output, &rpc_error(id, code, message.as_str()))?;
                }
            }
        }

        if method == "gateway.shutdown" {
            break;
        }
    }

    shutdown.store(true, Ordering::Release);
    let _ = event_pump.join();
    Ok(())
}

fn issue_server_requests(
    peer: &Arc<Mutex<ServerRequestRegistry>>,
    events: &[GatewayEventEnvelope],
) -> Vec<Value> {
    let Ok(mut peer) = peer.lock() else {
        return Vec::new();
    };
    let mut frames = Vec::new();
    for event in events {
        let GatewayEvent::ApprovalRequest(payload) = &event.event else {
            continue;
        };
        if pending_approval_exists(&peer, &payload.request_id) {
            continue;
        }
        if let Ok(issued) = peer.issue(
            event.session_id.clone(),
            "approval",
            approval_params(payload),
        ) {
            frames.push(issued.frame);
        }
    }
    frames
}

fn approval_params(payload: &ApprovalRequestPayload) -> Value {
    json!({
        "requestId": payload.request_id,
        "title": payload.title,
        "description": payload.description,
        "toolId": payload.tool_id,
        "metadata": payload.metadata,
        "choices": ["allow-once", "allow-session", "deny"],
    })
}

fn pending_approval_exists(peer: &ServerRequestRegistry, request_id: &str) -> bool {
    peer.export_state().iter().any(|request| {
        request.method == "approval"
            && request.params.get("requestId").and_then(Value::as_str) == Some(request_id)
    })
}

fn close_peer_approval(
    peer: &Arc<Mutex<ServerRequestRegistry>>,
    request_id: &str,
) -> Result<(), String> {
    let mut peer = peer
        .lock()
        .map_err(|_| "gateway peer mutex poisoned".to_string())?;
    let pending = peer.export_state().into_iter().find(|request| {
        request.method == "approval"
            && request.params.get("requestId").and_then(Value::as_str) == Some(request_id)
    });
    if let Some(request) = pending {
        let _ = peer.resolve_response(&json!({
            "jsonrpc": "2.0",
            "id": request.id,
            "result": {"decision": "resolved-by-approval.respond"},
        }));
    }
    Ok(())
}

fn resolve_peer_request(
    runtime: &Arc<Mutex<RuntimeHandle>>,
    resolved: ResolvedServerRequest,
) -> Result<(), String> {
    if resolved.request.method != "approval" {
        return Ok(());
    }
    let Some(request_id) = resolved
        .request
        .params
        .get("requestId")
        .and_then(Value::as_str)
    else {
        return Err("approval server request is missing requestId".to_string());
    };

    let decision = peer_decision(&resolved.response);
    runtime
        .lock()
        .map_err(|_| "runtime mutex poisoned".to_string())?
        .resolve_approval(request_id, decision)
        .map(|_| ())
}

fn rpc_result(id: Value, result: Value) -> Value {
    json!({ "jsonrpc": "2.0", "id": id, "result": result })
}

fn rpc_error(id: Value, code: i64, message: &str) -> Value {
    json!({
        "jsonrpc": "2.0",
        "id": id,
        "error": { "code": code, "message": message },
    })
}

fn write_notification(output: &Arc<Mutex<()>>, event: GatewayEventEnvelope) -> Result<(), String> {
    let value = serde_json::to_value(JsonRpcEventNotification::new(event))
        .map_err(|error| error.to_string())?;
    write_value(output, &value)
}

fn write_value(output: &Arc<Mutex<()>>, value: &Value) -> Result<(), String> {
    let _guard = output
        .lock()
        .map_err(|_| "stdout mutex poisoned".to_string())?;
    let stdout = io::stdout();
    let mut stdout = stdout.lock();
    serde_json::to_writer(&mut stdout, value).map_err(|error| error.to_string())?;
    stdout.write_all(b"\n").map_err(|error| error.to_string())?;
    stdout.flush().map_err(|error| error.to_string())
}

fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or(Duration::ZERO)
        .as_millis()
        .min(i64::MAX as u128) as i64
}

// The stdio gateway shares the same native Rust Host owner as the desktop.
// Keep the separately exported C ABI for clients that actually need FFI.
struct RuntimeHandle(MahayanaHost);

impl RuntimeHandle {
    fn create() -> Result<Self, String> {
        let cwd = std::env::current_dir().map_err(|error| error.to_string())?;
        let use_codex_account = std::env::var("MAHAYANA_USE_CODEX_ACCOUNT").as_deref() == Ok("1");
        let mut config = json!({
            "codexExecutablePath": Value::Null,
            "hostPlatform": "cli",
            "cwd": cwd,
            "workspaceRoots": [cwd],
            "useCodexAccount": use_codex_account,
        });
        if use_codex_account && let Some(codex_home) = std::env::var_os("MAHAYANA_CODEX_HOME") {
            config["codexHome"] = serde_json::to_value(PathBuf::from(codex_home))
                .map_err(|error| error.to_string())?;
        }
        if let Ok(base_url) = std::env::var("MAHAYANA_RESPONSES_BASE_URL")
            && !base_url.trim().is_empty()
        {
            config["model"] = json!({ "baseUrl": base_url });
        }
        Self::create_with_config(config)
    }

    fn create_with_config(config: Value) -> Result<Self, String> {
        let config: HostCreateConfig =
            serde_json::from_value(config).map_err(|error| error.to_string())?;
        MahayanaHost::create(config).map(Self).map_err(|error| error.to_string())
    }

    fn execute_value(&self, command: Value) -> Result<Value, String> {
        let command: RuntimeCommand =
            serde_json::from_value(command).map_err(|error| error.to_string())?;
        let response = self.0.execute(command).map_err(|error| error.to_string())?;
        serde_json::to_value(response).map_err(|error| error.to_string())
    }

    fn receive(&self, timeout_ms: u64) -> Result<Option<Value>, String> {
        self.0
            .receive(Duration::from_millis(timeout_ms))
            .map_err(|error| error.to_string())?
            .map(|event| serde_json::to_value(event).map_err(|error| error.to_string()))
            .transpose()
    }

    fn interrupt_value(&self, turn_id: &str) -> Result<Value, String> {
        let operation_id = OperationId::new(turn_id).map_err(|error| error.to_string())?;
        let response = self.0.interrupt(operation_id).map_err(|error| error.to_string())?;
        serde_json::to_value(response).map_err(|error| error.to_string())
    }

    fn resolve_approval_value(&self, approval_id: &str, decision: &str) -> Result<Value, String> {
        let approval_id = ApprovalId::new(approval_id).map_err(|error| error.to_string())?;
        let decision: ApprovalDecision = serde_json::from_value(json!(decision))
            .map_err(|error| format!("approval decision is invalid: {error}"))?;
        let response = self.0
            .resolve_approval(approval_id, decision, Value::Null)
            .map_err(|error| error.to_string())?;
        serde_json::to_value(response).map_err(|error| error.to_string())
    }
}

impl GatewayRuntime for RuntimeHandle {
    fn execute(&mut self, command: Value) -> Result<Value, String> {
        self.execute_value(command)
    }

    fn interrupt(&mut self, turn_id: &str) -> Result<Value, String> {
        self.interrupt_value(turn_id)
    }

    fn resolve_approval(&mut self, approval_id: &str, decision: &str) -> Result<Value, String> {
        self.resolve_approval_value(approval_id, decision)
    }
}

fn peer_decision(response: &ServerRequestResponse) -> &'static str {
    match response {
        ServerRequestResponse::Result(result) => match result.get("decision").and_then(Value::as_str) {
            Some("allow-once" | "accept") => "accept",
            Some("allow-session" | "accept-for-session" | "acceptForSession") => "acceptForSession",
            Some("cancel") => "cancel",
            _ => "decline",
        },
        ServerRequestResponse::Error(_) => "decline",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct ApprovalFixture {
        root: PathBuf,
    }

    impl ApprovalFixture {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "mahayana-gateway-approval-{}",
                uuid::Uuid::new_v4(),
            ));
            std::fs::create_dir(&root).expect("create an exclusively owned approval fixture");
            Self { root }
        }

        fn state_path(&self) -> PathBuf {
            self.root.join("gateway-state-v1.json")
        }

        fn open(&self) -> Arc<Mutex<ServerRequestRegistry>> {
            let registry = ServerRequestRegistry::new(Some(self.state_path()));
            assert!(registry.durability_info()["lastError"].is_null());
            Arc::new(Mutex::new(registry))
        }
    }

    impl Drop for ApprovalFixture {
        fn drop(&mut self) {
            // Only this fixture's UUID directory is owned; never clean the
            // production default store or alter global environment variables.
            let _ = std::fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn approval_event_emits_peer_request_with_same_runtime_request_id() {
        let fixture = ApprovalFixture::new();
        let peer = fixture.open();
        assert!(peer.lock().unwrap().is_empty());
        let event = GatewayEventEnvelope::new(
            "session-1",
            "turn-1",
            1,
            "2026-09-15T00:00:00Z",
            "epoch-1",
            GatewayEvent::ApprovalRequest(ApprovalRequestPayload {
                request_id: "approval-1".to_string(),
                title: "Run shell".to_string(),
                description: Some("echo hello".to_string()),
                tool_id: Some("tool-1".to_string()),
                metadata: None,
            }),
        );
        let frames = issue_server_requests(&peer, &[event]);
        assert_eq!(frames.len(), 1);
        assert!(peer.lock().unwrap().durability_info()["lastError"].is_null());
        assert_eq!(frames[0]["method"], "approval");
        assert_eq!(frames[0]["params"]["requestId"], "approval-1");
        assert!(frames[0]["id"].as_str().unwrap().starts_with("srq-"));
    }

    #[test]
    fn repeated_approval_event_does_not_duplicate_peer_lock() {
        let fixture = ApprovalFixture::new();
        let peer = fixture.open();
        assert!(peer.lock().unwrap().is_empty());
        let event = GatewayEventEnvelope::new(
            "session-1",
            "turn-1",
            1,
            "2026-09-15T00:00:00Z",
            "epoch-1",
            GatewayEvent::ApprovalRequest(ApprovalRequestPayload {
                request_id: "approval-1".to_string(),
                title: "Approve".to_string(),
                description: None,
                tool_id: None,
                metadata: None,
            }),
        );
        let first = issue_server_requests(&peer, &[event.clone()]);
        assert_eq!(first.len(), 1);
        assert!(peer.lock().unwrap().durability_info()["lastError"].is_null());
        let original = peer.lock().unwrap().export_state();
        assert_eq!(original.len(), 1);
        assert_eq!(first[0]["id"], original[0].id);
        let persisted = std::fs::read(fixture.state_path()).expect("persist the first approval");
        assert!(issue_server_requests(&peer, &[event.clone()]).is_empty());
        assert_eq!(peer.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);
        drop(peer);

        let restored = fixture.open();
        assert_eq!(restored.lock().unwrap().durability_info()["restoredRequests"], 1);
        assert_eq!(restored.lock().unwrap().export_state(), original);
        let pending = restored.lock().unwrap().open_requests("session-1");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0]["id"], first[0]["id"]);
        assert!(issue_server_requests(&restored, &[event.clone()]).is_empty());
        assert_eq!(restored.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);

        let independent_fixture = ApprovalFixture::new();
        let independent = independent_fixture.open();
        assert!(independent.lock().unwrap().is_empty());
        let independent_frames = issue_server_requests(&independent, &[event]);
        assert_eq!(independent_frames.len(), 1, "the same approval ID is isolated across fixture stores");
        assert!(independent.lock().unwrap().durability_info()["lastError"].is_null());
        assert_eq!(independent_frames[0]["params"]["requestId"], "approval-1");
        assert_eq!(independent.lock().unwrap().len(), 1);
        assert_eq!(restored.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);
    }

    #[test]
    fn peer_decision_maps_to_canonical_host_enum_and_declines_unknown() {
        for (external, expected) in [
            ("allow-once", "accept"),
            ("accept", "accept"),
            ("allow-session", "acceptForSession"),
            ("accept-for-session", "acceptForSession"),
            ("acceptForSession", "acceptForSession"),
            ("deny", "decline"),
            ("unknown", "decline"),
            ("cancel", "cancel"),
        ] {
            let result = ServerRequestResponse::Result(json!({"decision": external}));
            let decision = peer_decision(&result);
            assert_eq!(decision, expected);
            let canonical: ApprovalDecision =
                serde_json::from_value(json!(decision)).expect("Host accepts canonical decision");
            assert_eq!(serde_json::to_value(canonical).unwrap(), json!(expected));
        }
    }

    #[test]
    fn direct_gateway_host_status_and_ready_without_ffi() {
        let root = std::env::temp_dir().join(format!("mahayana-gateway-host-{}", uuid::Uuid::new_v4()));
        std::fs::create_dir_all(&root).expect("isolated gateway root");
        let host = RuntimeHandle::create_with_config(json!({
            "hostPlatform": "cli",
            "cwd": root, "workspaceRoots": [root], "dataDir": root,
            "inheritInstalledPlugins": false
        }))
        .expect("direct Host starts");
        let status = host.execute_value(json!({"@type": "mahayana.runtime.status"})).unwrap();
        assert_eq!(status["runtimeAbiVersion"], 1);
        let ready = host.receive(100).unwrap().expect("initial ready event");
        assert_eq!(ready["@type"], "mahayana.runtime.ready");
        drop(host);
        let _ = std::fs::remove_dir_all(root);
    }

    #[test]
    fn direct_gateway_host_rejects_legacy_telegram_handoff() {
        let error = RuntimeHandle::create_with_config(json!({
            "hostPlatform": "cli", "telegramSelfUserId": 42
        }))
        .err()
        .expect("legacy handoff must fail");
        assert!(error.contains("legacy Telegram handoff"), "{error}");
    }
}
