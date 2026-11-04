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
                // Poll non-blockingly under the runtime mutex and sleep outside
                // the guard when idle so OS unfair mutexes cannot starve stdin
                // request dispatch while waiting on the runtime event channel.
                let received = pump_runtime
                    .lock()
                    .map_err(|_| "runtime mutex poisoned".to_string())
                    .and_then(|runtime| runtime.receive(0));
                let raw_event = match received {
                    Ok(Some(event)) => event,
                    Ok(None) => {
                        if !pump_shutdown.load(Ordering::Acquire) {
                            thread::sleep(Duration::from_millis(10));
                        }
                        continue;
                    }
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
                if let Err(error) = publish_projected_events(&pump_peer, projected, |frame| {
                    write_value(&pump_output, frame)
                }) {
                    eprintln!("mahayana-gateway event delivery failed: {error}");
                    pump_shutdown.store(true, Ordering::Release);
                    let _ = write_value(&pump_output, &rpc_error(
                        Value::Null, -32000, "gateway event delivery failed",
                    ));
                    return;
                }
            }
        })
        .map_err(|error| error.to_string())?;

    for line in io::stdin().lock().lines() {
        if shutdown.load(Ordering::Acquire) {
            return Err("gateway event delivery stopped; recover storage before restarting".into());
        }
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
            let mut runtime = runtime
                .lock()
                .map_err(|_| "runtime mutex poisoned".to_string())?;
            resolve_peer_response(&peer, &mut *runtime, &request)?;
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
            let mut guarded = PeerGuardedRuntime { runtime: &mut *runtime, peer: &peer };
            dispatch_request(method, &params, &mut guarded, &mut state, now_ms())
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
) -> Result<Vec<Value>, String> {
    let mut peer = peer.lock().map_err(|_| "gateway peer mutex poisoned".to_string())?;
    let mut frames = Vec::new();
    for event in events {
        let GatewayEvent::ApprovalRequest(payload) = &event.event else {
            continue;
        };
        // The owner checks duplicates against the latest durable state while
        // holding the same file lock used to commit the new request.
        if let Some(issued) = peer.issue_approval_request(
            event.session_id.clone(),
            approval_params(payload),
        ).map_err(|error| error.to_string())? {
            frames.push(issued.frame);
        }
    }
    Ok(frames)
}

fn publish_projected_events(
    peer: &Arc<Mutex<ServerRequestRegistry>>,
    events: Vec<GatewayEventEnvelope>,
    mut write: impl FnMut(&Value) -> Result<(), String>,
) -> Result<(), String> {
    // Even the legacy semantic approval notification waits for peer commit.
    // An issuance failure cannot expose an approval that has no durable srq.
    let peer_frames = issue_server_requests(peer, &events)?;
    for event in events {
        let value = serde_json::to_value(JsonRpcEventNotification::new(event))
            .map_err(|error| error.to_string())?;
        write(&value)?;
    }
    for frame in peer_frames {
        write(&frame)?;
    }
    Ok(())
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

fn resolve_peer_response<R: GatewayRuntime>(
    peer: &Arc<Mutex<ServerRequestRegistry>>,
    runtime: &mut R,
    response: &Value,
) -> Result<(), String> {
    let resolved = {
        let mut peer = peer.lock().map_err(|_| "gateway peer mutex poisoned".to_string())?;
        peer.resolve_response(response).map_err(|error| error.to_string())?
    };
    // Both the peer mutex and its file lock have been released before Host.
    if let Some(resolved) = resolved {
        resolve_peer_request(runtime, resolved)?;
    }
    Ok(())
}

fn resolve_peer_request<R: GatewayRuntime>(
    runtime: &mut R,
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
        .resolve_approval(request_id, decision)
        .map(|_| ())
}

// Reuse dispatch_request's existing validation and canonical decision mapping.
// Only a validated Host-facing approval call reaches this persistence fence.
struct PeerGuardedRuntime<'a, R> {
    runtime: &'a mut R,
    peer: &'a Arc<Mutex<ServerRequestRegistry>>,
}

impl<R: GatewayRuntime> GatewayRuntime for PeerGuardedRuntime<'_, R> {
    fn execute(&mut self, command: Value) -> Result<Value, String> {
        self.runtime.execute(command)
    }

    fn interrupt(&mut self, turn_id: &str) -> Result<Value, String> {
        self.runtime.interrupt(turn_id)
    }

    fn resolve_approval(&mut self, approval_id: &str, decision: &str) -> Result<Value, String> {
        {
            let mut peer = self.peer.lock().map_err(|_| "gateway peer mutex poisoned".to_string())?;
            peer.resolve_approval_request(approval_id, json!({"decision": decision}))
                .map_err(|error| error.to_string())?;
        }
        // A healthy absent peer record retains the existing legacy Host
        // admission path. A storage error above never becomes that outcome.
        self.runtime.resolve_approval(approval_id, decision)
    }
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

        fn block_commit(&self) {
            std::fs::create_dir(self.root.join("gateway-state-v1.json.bak")).unwrap();
        }

        fn unblock_commit(&self) {
            std::fs::remove_dir(self.root.join("gateway-state-v1.json.bak")).unwrap();
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
        let frames = issue_server_requests(&peer, &[event]).unwrap();
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
        let stale_peer = fixture.open();
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
        let first = issue_server_requests(&peer, &[event.clone()]).unwrap();
        assert_eq!(first.len(), 1);
        assert!(peer.lock().unwrap().durability_info()["lastError"].is_null());
        let original = peer.lock().unwrap().export_state();
        assert_eq!(original.len(), 1);
        assert_eq!(first[0]["id"], original[0].id);
        let persisted = std::fs::read(fixture.state_path()).expect("persist the first approval");
        assert!(issue_server_requests(&peer, &[event.clone()]).unwrap().is_empty());
        assert!(issue_server_requests(&stale_peer, &[event.clone()]).unwrap().is_empty());
        assert_eq!(stale_peer.lock().unwrap().export_state(), original);
        assert_eq!(peer.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);
        drop(peer);

        let restored = fixture.open();
        assert_eq!(restored.lock().unwrap().durability_info()["restoredRequests"], 1);
        assert_eq!(restored.lock().unwrap().export_state(), original);
        let pending = restored.lock().unwrap().open_requests("session-1");
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0]["id"], first[0]["id"]);
        assert!(issue_server_requests(&restored, &[event.clone()]).unwrap().is_empty());
        assert_eq!(restored.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);

        let independent_fixture = ApprovalFixture::new();
        let independent = independent_fixture.open();
        assert!(independent.lock().unwrap().is_empty());
        let independent_frames = issue_server_requests(&independent, &[event]).unwrap();
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
            "productSessionPath": root.join("session.json"),
            "productSurfaceStatePath": root.join("surfaces.json"),
            "inheritInstalledPlugins": false
        }))
        .expect("direct Host starts");
        let status = host.execute_value(json!({"@type": "mahayana.runtime.status"})).unwrap();
        assert_eq!(status["runtimeAbiVersion"], 1);
        // Status/Ready alone also succeed with UnavailableAgentBackend.
        // A fresh native backend owns an empty, locally queried MCP inventory.
        let servers = host.execute_value(json!({"@type": "mahayana.mcp.servers"}))
            .expect("native Agent backend exposes its local MCP inventory");
        assert_eq!(servers, json!({"@type": "mahayana.mcp.servers", "data": []}));
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

    fn approval_event(request_id: &str) -> GatewayEventEnvelope {
        GatewayEventEnvelope::new(
            "session-1", "turn-1", 1, "2026-09-15T00:00:00Z", "epoch-1",
            GatewayEvent::ApprovalRequest(ApprovalRequestPayload {
                request_id: request_id.into(),
                title: "Approve".into(),
                description: None,
                tool_id: None,
                metadata: None,
            }),
        )
    }

    struct RecordingRuntime {
        peer: Arc<Mutex<ServerRequestRegistry>>,
        path: PathBuf,
        approvals: Vec<(String, String, Value)>,
        ordinary_calls: usize,
    }

    impl RecordingRuntime {
        fn new(peer: &Arc<Mutex<ServerRequestRegistry>>, fixture: &ApprovalFixture) -> Self {
            Self {
                peer: Arc::clone(peer), path: fixture.state_path(),
                approvals: Vec::new(), ordinary_calls: 0,
            }
        }
    }

    impl GatewayRuntime for RecordingRuntime {
        fn execute(&mut self, _command: Value) -> Result<Value, String> {
            self.ordinary_calls += 1;
            Err("unexpected ordinary dispatch".into())
        }

        fn interrupt(&mut self, _turn_id: &str) -> Result<Value, String> {
            self.ordinary_calls += 1;
            Err("unexpected interrupt".into())
        }

        fn resolve_approval(&mut self, approval_id: &str, decision: &str) -> Result<Value, String> {
            // Observe the real production adapter's ordering independently:
            // no peer mutex is held, and committed bytes are already visible.
            assert!(self.peer.try_lock().is_ok(), "peer mutex held across Host call");
            let disk: Value = serde_json::from_slice(&std::fs::read(&self.path).unwrap()).unwrap();
            self.approvals.push((approval_id.into(), decision.into(), disk));
            Ok(json!({"resolved": true}))
        }
    }

    #[test]
    fn failed_peer_commit_publishes_no_approval_event_or_frame_and_recovers() {
        let fixture = ApprovalFixture::new();
        let peer = fixture.open();
        peer.lock().unwrap().issue("other-session", "clarify", json!({})).unwrap();
        let original = peer.lock().unwrap().export_state();
        let persisted = std::fs::read(fixture.state_path()).unwrap();
        fixture.block_commit();
        let mut published = Vec::new();
        let failed = publish_projected_events(&peer, vec![approval_event("approval-failed")], |frame| {
            published.push(frame.clone());
            Ok(())
        });
        assert!(failed.is_err());
        assert!(published.is_empty(), "neither semantic approval nor srq may escape");
        assert_eq!(peer.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);
        assert!(peer.lock().unwrap().durability_info()["lastError"].is_string());
        fixture.unblock_commit();
        publish_projected_events(&peer, vec![approval_event("approval-failed")], |frame| {
            let disk: Value = serde_json::from_slice(&std::fs::read(fixture.state_path()).unwrap()).unwrap();
            assert!(disk["peer"]["requests"].as_array().unwrap().iter().any(|request| {
                request["params"]["requestId"] == "approval-failed"
            }), "request must be durable even before the semantic notification");
            published.push(frame.clone());
            Ok(())
        }).unwrap();
        assert_eq!(published.len(), 2);
        assert_eq!(published[0]["method"], "event");
        assert_eq!(published[1]["method"], "approval");
        assert_eq!(published[1]["params"]["requestId"], "approval-failed");
        let restored = fixture.open();
        assert_eq!(restored.lock().unwrap().len(), 2);
        assert_eq!(restored.lock().unwrap().export_state(), peer.lock().unwrap().export_state());
    }

    #[test]
    fn response_storage_failure_preserves_pending_and_never_calls_host() {
        let fixture = ApprovalFixture::new();
        let peer = fixture.open();
        let frames = issue_server_requests(&peer, &[approval_event("approval-direct")]).unwrap();
        let response = json!({"jsonrpc":"2.0", "id":frames[0]["id"], "result":{"decision":"allow-once"}});
        let original = peer.lock().unwrap().export_state();
        let persisted = std::fs::read(fixture.state_path()).unwrap();
        let mut runtime = RecordingRuntime::new(&peer, &fixture);
        fixture.block_commit();
        assert!(resolve_peer_response(&peer, &mut runtime, &response).is_err());
        assert!(runtime.approvals.is_empty());
        assert_eq!(runtime.ordinary_calls, 0);
        assert_eq!(peer.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);
        fixture.unblock_commit();
        assert_eq!(fixture.open().lock().unwrap().export_state(), original);
        resolve_peer_response(&peer, &mut runtime, &response).unwrap();
        assert_eq!(runtime.approvals.len(), 1);
        assert_eq!(runtime.approvals[0].0, "approval-direct");
        assert_eq!(runtime.approvals[0].1, "accept");
        assert_eq!(runtime.approvals[0].2["peer"]["requests"], json!([]));
        resolve_peer_response(&peer, &mut runtime, &response).unwrap();
        assert_eq!(runtime.approvals.len(), 1, "late duplicate response cannot call Host again");
        assert!(fixture.open().lock().unwrap().is_empty());
    }

    #[test]
    fn compatibility_approval_validates_then_commits_before_host() {
        let fixture = ApprovalFixture::new();
        let peer = fixture.open();
        issue_server_requests(&peer, &[approval_event("approval-legacy")]).unwrap();
        let mut state = GatewayState::with_path(ReplayLimits::default(), Some(fixture.state_path()));
        let mut runtime = RecordingRuntime::new(&peer, &fixture);
        let original = peer.lock().unwrap().export_state();
        let persisted = std::fs::read(fixture.state_path()).unwrap();
        fixture.block_commit();
        let invalid = dispatch_request(
            "approval.respond", &json!({"approvalId":"approval-legacy", "decision":"unknown"}),
            &mut PeerGuardedRuntime { runtime: &mut runtime, peer: &peer }, &mut state, 1,
        ).unwrap_err();
        assert_eq!(invalid.code, -32602);
        let failed = dispatch_request(
            "approval.respond", &json!({"approval_id":"approval-legacy", "decision":"allow-session"}),
            &mut PeerGuardedRuntime { runtime: &mut runtime, peer: &peer }, &mut state, 2,
        );
        assert!(failed.is_err());
        assert!(runtime.approvals.is_empty());
        assert_eq!(runtime.ordinary_calls, 0);
        assert_eq!(peer.lock().unwrap().export_state(), original);
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), persisted);
        fixture.unblock_commit();
        let dispatched = dispatch_request(
            "approval.respond", &json!({"approvalId":"approval-legacy", "decision":"allow-session"}),
            &mut PeerGuardedRuntime { runtime: &mut runtime, peer: &peer }, &mut state, 3,
        ).unwrap();
        assert_eq!(dispatched.result["resolved"], true);
        assert_eq!(runtime.approvals.len(), 1);
        assert_eq!(runtime.approvals[0].0, "approval-legacy");
        assert_eq!(runtime.approvals[0].1, "acceptForSession");
        assert_eq!(runtime.approvals[0].2["peer"]["requests"], json!([]));
        assert!(fixture.open().lock().unwrap().is_empty());
    }

    #[test]
    fn compatibility_approval_distinguishes_healthy_absence_from_broken_storage() {
        let fixture = ApprovalFixture::new();
        let peer = fixture.open();
        let mut state = GatewayState::with_path(ReplayLimits::default(), Some(fixture.state_path()));
        let mut runtime = RecordingRuntime::new(&peer, &fixture);
        std::fs::write(fixture.state_path(), b"{").unwrap();
        let params = json!({"approvalId":"host-owned-approval", "decision":"deny"});
        let failed = dispatch_request(
            "approval.respond", &params,
            &mut PeerGuardedRuntime { runtime: &mut runtime, peer: &peer }, &mut state, 1,
        );
        assert!(failed.is_err());
        assert!(runtime.approvals.is_empty());
        assert_eq!(std::fs::read(fixture.state_path()).unwrap(), b"{");
        std::fs::write(fixture.state_path(), br#"{"schemaVersion":1,"peer":{"requests":[]}}"#).unwrap();
        dispatch_request(
            "approval.respond", &params,
            &mut PeerGuardedRuntime { runtime: &mut runtime, peer: &peer }, &mut state, 2,
        ).unwrap();
        assert_eq!(runtime.approvals.len(), 1);
        assert_eq!(runtime.approvals[0].0, "host-owned-approval");
        assert_eq!(runtime.approvals[0].1, "decline");
        assert_eq!(runtime.ordinary_calls, 0);
    }
}
