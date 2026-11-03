use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpStream;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use flate2::read::GzDecoder;
use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentsBox, AttachmentsService, LINK_CACHE_DIRNAME, LINK_CACHE_TTL_MS,
    LINK_CACHE_VERSION, LinkMetadata,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
use mahayana_host_runtime::extensions::mcp::coordinator_relay::{
    CoordinatorMcpLifecycleRelay, CoordinatorMcpManagerBackend,
    HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL, MCP_AUTHENTICATE_SERVER_METHOD,
    MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD,
};
use mahayana_host_runtime::extensions::mcp::mcp_service::McpManagerBackend;
use mahayana_host_runtime::runner::coordinator_tool_relay::{
    CoordinatorToolRelay, ROUTED_TOOL_EXECUTE_METHOD,
    RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD, RUNNER_TOOL_REQUEST_EVENT_CHANNEL,
};
use mahayana_host_runtime::gateway_config::{GatewayServerConfig, GatewayTlsConfig};
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayBridgeHub, GatewayCommandContext, GatewayCommandError,
    GatewayCommandReport, GatewayEventHub, GatewayHealth, GatewayServer, GatewayServerDeps,
    start_gateway_server,
};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

#[derive(Debug)]
struct TestApi;

impl GatewayApi for TestApi {
    fn redeem_device_pairing_join_code(&self, shortcode: &str) -> Result<Option<Value>, GatewayCommandError> {
        Ok((shortcode == "A234567890123456789012").then(|| json!({
            "deviceId": "device-join-1",
            "token": "single-use-token",
        })))
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        match method {
            "listAgents" => Ok(args.get("agents").cloned().unwrap_or_else(|| json!([
                {"id":"agent-1","name":"One","avatarDataUrl":"data:image/png;base64,aGVsbG8="}
            ]))),
            "getAgentAvatar" => Ok(json!({
                "dataUrl": "data:image/png;base64,aGVsbG8=",
                "version": "avatar-v1"
            })),
            "getTranscript" => Ok(args),
            method if method.starts_with("feature.") => Ok(json!({
                "method": method,
                "args": args,
            })),
            method if matches!(
                method,
                "runner.acceptRoutedPrompt"
                    | "runner.startRoutedProvider"
                    | "runner.cancelRoutedProvider"
                    | "runner.resolveRoutedToolRequest"
                    | "mcp.resolveLifecycleRequest"
            ) => Ok(json!({
                "method": method,
                "args": args,
            })),
            method if method.starts_with("device.pair.")
                || method.starts_with("node.")
                || method == "getHostSettings"
                || method == "setHostSettings" => Ok(json!({
                "method": method,
                "args": args,
            })),
            "conflict" => Err(GatewayCommandError::Conflict("conflict".into())),
            other => Err(GatewayCommandError::UnknownMethod(other.to_string())),
        }
    }

    fn health(&self) -> GatewayHealth {
        GatewayHealth {
            is_busy: true,
            busy_only_awaiting_approval: Some(false),
            active_agent_id: Some("agent-1".into()),
            last_busy_at_ms: Some(123),
        }
    }

    fn prepare_for_upgrade(&self) -> Result<Value, GatewayCommandError> {
        Ok(json!({ "quiescing": true, "runningTurns": 2 }))
    }
}

struct NodeTokenApi {
    owner: DevicePairingOwner,
    token: String,
    calls: Mutex<Vec<(String, Value)>>,
}

impl Default for NodeTokenApi {
    fn default() -> Self {
        let owner = DevicePairingOwner::in_memory().expect("node pairing owner");
        let pending = owner.handle_pair_request(&json!({
            "deviceId": "node-1", "publicKey": "node-contract-key",
            "role": "node", "roles": ["node"], "scopes": [],
        })).expect("node pairing request");
        owner.handle_pair_approve(&json!({"requestId": pending["requestId"]}))
            .expect("node pairing approval");
        let rotated = owner.handle_pair_rotate_token(&json!({
            "deviceId": "node-1", "role": "node",
        })).expect("node token issuance");
        let token = rotated["token"].as_str().expect("issued node token").to_owned();
        Self { owner, token, calls: Mutex::new(Vec::new()) }
    }
}

impl GatewayApi for NodeTokenApi {
    fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.owner.authenticate_device_token(token)
    }

    fn authorize_device_token_request(
        &self,
        authorization: &DeviceTokenAuthorization,
        method: &str,
        args: &Value,
    ) -> Result<(), GatewayCommandError> {
        if authorization.role == "node"
            && method == "node.invoke"
            && args.get("nodeId").and_then(Value::as_str) != Some(authorization.device_id.as_str())
        {
            return Err(GatewayCommandError::Forbidden("DEVICE_SCOPE_MISMATCH".into()));
        }
        Ok(())
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.calls
            .lock()
            .expect("node dispatch observations")
            .push((method.to_string(), args.clone()));
        Ok(json!({"method": method, "args": args}))
    }
}

#[test]
fn device_pairing_owner_authenticates_issued_token_with_role_and_scopes() {
    let owner = DevicePairingOwner::in_memory().expect("pairing owner");
    let pending = owner
        .handle_pair_request(&json!({
            "deviceId": "device-auth-1",
            "publicKey": "public-key-auth-1",
            "platform": "macos",
            "deviceFamily": "desktop",
            "roles": ["operator"],
            "scopes": ["operator.read", "operator.write"]
        }))
        .expect("pair request");
    let request_id = pending
        .get("requestId")
        .and_then(Value::as_str)
        .expect("request id");
    owner
        .handle_pair_approve(&json!({ "requestId": request_id }))
        .expect("pair approval");
    let status = owner
        .handle_pair_status(&json!({ "deviceId": "device-auth-1" }))
        .expect("pair status");
    assert_eq!(status["device"]["platform"], "macos");
    assert_eq!(status["device"]["deviceFamily"], "desktop");
    let rotated = owner
        .handle_pair_rotate_token(&json!({ "deviceId": "device-auth-1", "role": "operator" }))
        .expect("token rotation");
    let token = rotated
        .get("token")
        .and_then(Value::as_str)
        .expect("rotated token");

    let authorization = owner
        .authenticate_device_token(token)
        .expect("token lookup")
        .expect("issued token authorization");
    assert_eq!(authorization.device_id, "device-auth-1");
    assert_eq!(authorization.role, "operator");
    assert_eq!(
        authorization.scopes,
        vec!["operator.read".to_string(), "operator.write".to_string()]
    );
    assert!(owner
        .authenticate_device_token("sand-device-tok-invalid")
        .expect("invalid token lookup")
        .is_none());
}

#[test]
fn gateway_binds_node_token_to_the_addressed_node_before_dispatch() {
    let api = Arc::new(NodeTokenApi::default());
    let server = start_gateway_server(GatewayServerDeps {
        api: api.clone(),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(Some("gateway-secret")),
        started_at: 100,
    })
    .expect("gateway server");
    let port = server.port();

    let invoke = |node_id: &str| {
        let body = json!({"nodeId": node_id}).to_string();
        request(
            port,
            &format!(
                "POST /api/node.invoke HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                api.token,
                body.len(),
            ),
        )
    };
    let rejected = invoke("node-2");
    assert!(rejected.starts_with("HTTP/1.1 403 Forbidden"), "{rejected}");
    assert!(rejected.contains("DEVICE_SCOPE_MISMATCH"), "{rejected}");
    assert!(api.calls.lock().expect("node dispatch observations").is_empty());

    let accepted = invoke("node-1");
    assert!(accepted.starts_with("HTTP/1.1 200 OK"), "{accepted}");
    assert_eq!(json_body(&accepted), json!({"method":"node.invoke", "args":{"nodeId":"node-1"}}));
    assert_eq!(
        *api.calls.lock().expect("node dispatch observations"),
        vec![("node.invoke".to_string(), json!({"nodeId":"node-1"}))],
    );

    server.close();
}

struct PairingOwnerApi {
    owner: Arc<DevicePairingOwner>,
}

impl GatewayApi for PairingOwnerApi {
    fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<mahayana_host_runtime::device_pairing_owner::DeviceTokenAuthorization>, GatewayCommandError> {
        self.owner.authenticate_device_token(token)
    }

    fn call(&self, method: &str, _args: Value) -> Result<Value, GatewayCommandError> {
        if method == "device.pair.list" {
            return Ok(json!({"paired": [], "pending": []}));
        }
        Err(GatewayCommandError::UnknownMethod(method.to_string()))
    }

    fn redeem_device_pairing_join_code(
        &self,
        shortcode: &str,
    ) -> Result<Option<Value>, GatewayCommandError> {
        self.owner.redeem_device_pairing_join_code(shortcode)
    }
}

#[test]
fn gateway_accepts_pairing_owner_device_token_without_gateway_secret() {
    let owner = Arc::new(DevicePairingOwner::in_memory().expect("pairing owner"));
    let pending = owner
        .handle_pair_request(&json!({
            "deviceId": "device-gateway-auth-1",
            "publicKey": "public-key-gateway-auth-1"
        }))
        .expect("pair request");
    let request_id = pending
        .get("requestId")
        .and_then(Value::as_str)
        .expect("request id");
    owner
        .handle_pair_approve(&json!({ "requestId": request_id }))
        .expect("pair approval");
    let rotated = owner
        .handle_pair_rotate_token(&json!({ "deviceId": "device-gateway-auth-1" }))
        .expect("token rotation");
    let token = rotated
        .get("token")
        .and_then(Value::as_str)
        .expect("rotated token");

    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(PairingOwnerApi { owner }),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(Some("gateway-token")),
        started_at: 3,
    })
    .expect("gateway server");
    let port = server.port();
    let accepted = request(
        port,
        &format!(
            "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(accepted.starts_with("HTTP/1.1 200 OK"), "{accepted}");
    let rejected = request(
        port,
        "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer invalid-device-token\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(rejected.starts_with("HTTP/1.1 401 Unauthorized"), "{rejected}");
    server.close();
}

#[test]
fn gateway_rejects_node_token_on_operator_pairing_surface() {
    let owner = Arc::new(DevicePairingOwner::in_memory().expect("pairing owner"));
    let pending = owner
        .handle_pair_request(&json!({
            "deviceId": "device-node-only",
            "publicKey": "public-key-node-only",
            "roles": ["node"]
        }))
        .expect("pair request");
    let request_id = pending
        .get("requestId")
        .and_then(Value::as_str)
        .expect("request id");
    owner
        .handle_pair_approve(&json!({ "requestId": request_id }))
        .expect("pair approval");
    let rotated = owner
        .handle_pair_rotate_token(&json!({
            "deviceId": "device-node-only",
            "role": "node"
        }))
        .expect("node token rotation");
    let token = rotated
        .get("token")
        .and_then(Value::as_str)
        .expect("node token");

    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(PairingOwnerApi { owner }),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 100,
    })
    .expect("gateway server");
    let response = request(
        server.port(),
        &format!(
            "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(response.starts_with("HTTP/1.1 403 Forbidden"), "{response}");
    assert!(response.contains("ROLE_MISMATCH"), "{response}");
    server.close();
}

#[test]
fn gateway_pairing_join_route_reaches_sqlite_owner_and_burns_code() {
    let owner = Arc::new(DevicePairingOwner::in_memory().expect("pairing owner"));
    let expires_at_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock")
        .as_millis() as i64
        + 60_000;
    let code = owner
        .register_device_pairing_join_code(&json!({
            "deviceId": "device-owner-backed",
            "setupCode": "secret-payload",
        }), expires_at_ms)
        .expect("register owner-backed join code");
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(PairingOwnerApi { owner }),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(Some("gateway-token")),
        started_at: 2,
    })
    .expect("gateway server");
    let port = server.port();
    let first = request(
        port,
        &format!(
            "GET /j/{code} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(first.starts_with("HTTP/1.1 200 OK"), "{first}");
    assert_eq!(json_body(&first)["deviceId"], "device-owner-backed");
    let second = request(
        port,
        &format!(
            "GET /j/{code} HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        ),
    );
    assert!(second.starts_with("HTTP/1.1 404 Not Found"), "{second}");
    server.close();
}

#[test]
fn gateway_public_pairing_join_route_redeems_once_and_rejects_query_or_empty_code() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(Some("secret")),
        started_at: 1,
    })
    .expect("gateway server");
    let port = server.port();

    let redeemed = request(
        port,
        "GET /j/A234567890123456789012 HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(redeemed.starts_with("HTTP/1.1 200 OK"), "{redeemed}");
    assert_eq!(json_body(&redeemed)["deviceId"], "device-join-1");
    assert!(redeemed.to_ascii_lowercase().contains("cache-control: no-store"));

    let queried = request(
        port,
        "GET /j/A234567890123456789012?x=1 HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(queried.starts_with("HTTP/1.1 404 Not Found"), "{queried}");
    assert!(queried.to_ascii_lowercase().contains("cache-control: no-store"));

    let empty = request(
        port,
        "GET /j HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(empty.starts_with("HTTP/1.1 404 Not Found"), "{empty}");
    assert!(empty.to_ascii_lowercase().contains("cache-control: no-store"));
    server.close();
}


#[derive(Clone, Default)]
struct ReportingApi {
    reports: Arc<Mutex<Vec<(String, GatewayCommandReport)>>>,
    contexts: Arc<Mutex<Vec<(
        Option<String>,
        Option<String>,
        Option<String>,
        Option<String>,
        bool,
    )>>>,
}

impl GatewayApi for ReportingApi {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        if method == "getTranscript" && args.get("fail").and_then(Value::as_bool) == Some(true) {
            return Err(GatewayCommandError::Internal("synthetic internal failure".into()));
        }
        if method == "getTranscript" {
            return Ok(args);
        }
        Err(GatewayCommandError::UnknownMethod(method.to_string()))
    }

    fn call_with_context(
        &self,
        method: &str,
        args: Value,
        context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        self.contexts
            .lock()
            .expect("context lock")
            .push((
                context.request_id.clone(),
                context.traceparent.clone(),
                context.trace_id.clone(),
                context.span_id.clone(),
                context.authenticated,
            ));
        self.call(method, args)
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        self.reports
            .lock()
            .expect("report lock")
            .push(("complete".into(), report));
    }

    fn on_command_error(&self, report: GatewayCommandReport) {
        self.reports
            .lock()
            .expect("report lock")
            .push(("error".into(), report));
    }
}

fn config(token: Option<&str>) -> GatewayServerConfig {
    GatewayServerConfig {
        host: "127.0.0.1".into(),
        port: None,
        auth_token: token.map(str::to_string),
        tls: None,
    }
}

fn request_bytes(port: u16, request: &str) -> Vec<u8> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect gateway");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout");
    stream.write_all(request.as_bytes()).expect("write request");
    stream.flush().expect("flush request");
    let mut response = Vec::new();
    stream
        .read_to_end(&mut response)
        .expect("read gateway response");
    response
}

fn request(port: u16, request: &str) -> String {
    String::from_utf8(request_bytes(port, request)).expect("utf8 gateway response")
}

fn json_body(response: &str) -> Value {
    let (_, body) = response
        .split_once("\r\n\r\n")
        .expect("HTTP response body");
    serde_json::from_str(body).expect("JSON body")
}

struct BoxMcpRouteCase {
    method: &'static str,
    args: Value,
    result: Value,
}

fn box_mcp_route_cases() -> [BoxMcpRouteCase; 3] {
    [
        BoxMcpRouteCase {
            method: "loadBoxMcpServers",
            args: json!({
                "configJson": "{\n  \"stdio-one\": {\"command\": \"fixture-mcp\", \"args\": [\" keep spaces \", \"snowman ☃\"]}\n}\n"
            }),
            result: json!({"serverIdentifiers": ["stdio-one", "stdio-two"]}),
        },
        BoxMcpRouteCase {
            method: "listBoxMcpToolsRaw",
            args: json!({"payloadHex": " \t0a00FF00a5\n"}),
            result: json!({"payloadHex": "1200aB00FF"}),
        },
        BoxMcpRouteCase {
            method: "executeBoxMcpToolRaw",
            args: json!({"payloadHex": "120700A5ff001122"}),
            result: json!({"payloadHex": "0a00bB0010"}),
        },
    ]
}

// These probes exercise the production HTTP router and its GatewayApi port.
// Opaque payloads and owner replies are controlled here; no Box/stdio process,
// provider account, or per-tool permission decision is exercised by this fixture.
#[derive(Default)]
struct CountingBoxMcpApi {
    calls: Mutex<Vec<(String, Value)>>,
    pairing_owner: Option<Arc<DevicePairingOwner>>,
    device_authorizations: Mutex<Vec<DeviceTokenAuthorization>>,
    owner_error: Option<fn() -> GatewayCommandError>,
}

impl CountingBoxMcpApi {
    fn recorded_calls(&self) -> Vec<(String, Value)> {
        self.calls.lock().expect("Box MCP calls").clone()
    }

    fn recorded_device_authorizations(&self) -> Vec<DeviceTokenAuthorization> {
        self.device_authorizations
            .lock()
            .expect("Box MCP device authorizations")
            .clone()
    }
}

impl GatewayApi for CountingBoxMcpApi {
    fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        let Some(owner) = self.pairing_owner.as_ref() else {
            return Ok(None);
        };
        let authorization = owner.authenticate_device_token(token)?;
        if let Some(authorization) = authorization.as_ref() {
            self.device_authorizations
                .lock()
                .expect("record Box MCP device authorization")
                .push(authorization.clone());
        }
        Ok(authorization)
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.calls
            .lock()
            .expect("record Box MCP owner call")
            .push((method.to_string(), args.clone()));
        if let Some(error) = self.owner_error {
            return Err(error());
        }
        if method == "getTranscript" {
            return Ok(args);
        }
        box_mcp_route_cases()
            .into_iter()
            .find(|case| case.method == method)
            .map(|case| case.result)
            .ok_or_else(|| GatewayCommandError::UnknownMethod(method.to_string()))
    }
}

fn start_box_mcp_gateway(
    api: &Arc<CountingBoxMcpApi>,
    config: GatewayServerConfig,
) -> GatewayServer {
    start_gateway_server(GatewayServerDeps {
        api: api.clone(),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config,
        started_at: 1,
    })
    .expect("Box MCP gateway server")
}

fn box_mcp_post(
    port: u16,
    method: &str,
    bearer: Option<&str>,
    extra_headers: &str,
    body: &str,
) -> String {
    let authorization = bearer
        .map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    request(
        port,
        &format!(
            "POST /api/{method} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}{extra_headers}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    )
}

fn assert_box_mcp_round_trip(
    api: &CountingBoxMcpApi,
    port: u16,
    bearer: Option<&str>,
    headers: &str,
    case: &BoxMcpRouteCase,
) {
    let mut expected_calls = api.recorded_calls();
    let response = box_mcp_post(port, case.method, bearer, headers, &case.args.to_string());
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{}: {response}", case.method);
    assert_eq!(json_body(&response), case.result, "{} result", case.method);
    expected_calls.push((case.method.to_string(), case.args.clone()));
    assert_eq!(
        api.recorded_calls(),
        expected_calls,
        "{} must reach the owner exactly once with the original arguments",
        case.method
    );
}

fn assert_box_mcp_native_only(response: &str) {
    assert!(response.starts_with("HTTP/1.1 403 Forbidden"), "{response}");
    assert!(
        json_body(response)["error"]
            .as_str()
            .expect("native-only error string")
            .contains("MCP_BOX_NATIVE_ONLY"),
        "{response}"
    );
}

fn issue_box_mcp_device_token(
    owner: &DevicePairingOwner,
    role: &str,
) -> (String, DeviceTokenAuthorization) {
    let device_id = format!("box-mcp-{role}");
    let scopes = if role == "operator" {
        vec!["operator.read", "operator.write"]
    } else {
        vec![]
    };
    issue_gateway_device_token(
        owner, &device_id, &format!("box-mcp-public-key-{role}"), role, &scopes,
    )
}

fn issue_gateway_device_token(
    owner: &DevicePairingOwner,
    device_id: &str,
    public_key: &str,
    role: &str,
    scopes: &[&str],
) -> (String, DeviceTokenAuthorization) {
    let pending = owner
        .handle_pair_request(&json!({
            "deviceId": device_id,
            "publicKey": public_key,
            "role": role,
            "roles": [role],
            "scopes": scopes,
        }))
        .expect("Box MCP pair request");
    let request_id = pending["requestId"].as_str().expect("Box MCP pair request id");
    owner
        .handle_pair_approve(&json!({"requestId": request_id}))
        .expect("Box MCP pair approval");
    let rotated = owner
        .handle_pair_rotate_token(&json!({"deviceId": device_id, "role": role}))
        .expect("Box MCP device token rotation");
    let token = rotated["token"].as_str().expect("Box MCP device token").to_string();
    let authorization = owner
        .authenticate_device_token(&token)
        .expect("Box MCP issued-token lookup")
        .expect("Box MCP token must be real pairing authorization");
    assert_eq!(authorization.device_id, device_id);
    assert_eq!(authorization.role, role);
    assert_eq!(
        authorization.scopes,
        scopes.iter().map(|scope| scope.to_string()).collect::<Vec<_>>()
    );
    (token, authorization)
}

#[test]
fn native_box_mcp_http_routes_preserve_arguments_results_and_single_dispatch() {
    for token in [None, Some("box-native-shared-secret")] {
        let api = Arc::new(CountingBoxMcpApi::default());
        let server = start_box_mcp_gateway(&api, config(token));
        for case in box_mcp_route_cases() {
            for headers in [
                "",
                "x-sand-role: operator\r\n",
                "x-sand-scope: operator.write\r\n",
                "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n",
            ] {
                assert_box_mcp_round_trip(&api, server.port(), token, headers, &case);
            }
        }
        server.close();
        assert_eq!(api.recorded_calls().len(), 12, "no extra owner dispatch");
    }
}

#[test]
fn native_box_mcp_configured_secret_cannot_be_replaced_by_headers() {
    let api = Arc::new(CountingBoxMcpApi::default());
    let secret = "box-native-shared-secret";
    let server = start_box_mcp_gateway(&api, config(Some(secret)));
    for case in box_mcp_route_cases() {
        for bearer in [None, Some("wrong-box-native-secret")] {
            for headers in [
                "",
                "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n",
            ] {
                let before = api.recorded_calls();
                let response = box_mcp_post(
                    server.port(), case.method, bearer, headers, &case.args.to_string(),
                );
                assert!(response.starts_with("HTTP/1.1 401 Unauthorized"), "{response}");
                assert_eq!(json_body(&response), json!({"error": "unauthorized"}));
                assert_eq!(api.recorded_calls(), before, "unauthenticated owner call");
            }
        }
        assert_box_mcp_round_trip(&api, server.port(), Some(secret), "", &case);
    }
    server.close();
    assert_eq!(api.recorded_calls().len(), 3);
}

#[test]
fn native_box_mcp_rejects_restricted_unknown_and_empty_role_or_scope_before_owner() {
    for token in [None, Some("box-native-shared-secret")] {
        let api = Arc::new(CountingBoxMcpApi::default());
        let server = start_box_mcp_gateway(&api, config(token));
        for case in box_mcp_route_cases() {
            for headers in [
                "x-sand-role: node\r\n",
                "x-sand-role: unknown-role\r\n",
                "x-sand-role: Operator\r\n",
                "x-sand-role: \r\n",
                "x-sand-role: operator\r\nx-sand-scope: operator.read\r\n",
                "x-sand-scope: unknown-scope\r\n",
                "x-sand-scope: operator.write operator.read\r\n",
                "x-sand-scope: \r\n",
            ] {
                let before = api.recorded_calls();
                let response = box_mcp_post(
                    server.port(), case.method, token, headers, &case.args.to_string(),
                );
                assert_box_mcp_native_only(&response);
                assert_eq!(api.recorded_calls(), before, "restricted owner call: {headers:?}");
            }
            assert_box_mcp_round_trip(
                &api, server.port(), token,
                "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n", &case,
            );
        }
        server.close();
        assert_eq!(api.recorded_calls().len(), 3);
    }
}

#[test]
fn native_box_mcp_rejects_real_node_and_operator_write_device_tokens_before_owner() {
    for configured_token in [None, Some("box-native-shared-secret")] {
        let owner = Arc::new(DevicePairingOwner::in_memory().expect("Box MCP pairing owner"));
        let tokens = [
            issue_box_mcp_device_token(&owner, "node"),
            issue_box_mcp_device_token(&owner, "operator"),
        ];
        let api = Arc::new(CountingBoxMcpApi {
            pairing_owner: Some(owner),
            ..CountingBoxMcpApi::default()
        });
        let server = start_box_mcp_gateway(&api, config(configured_token));
        for (device_token, authorization) in tokens {
            // The same issued credential remains usable on an ordinary route.
            let ordinary_args = json!({"pairedDeviceControl": authorization.device_id});
            let mut expected_calls = api.recorded_calls();
            let ordinary = box_mcp_post(
                server.port(), "getTranscript", Some(&device_token), "", &ordinary_args.to_string(),
            );
            assert!(ordinary.starts_with("HTTP/1.1 200 OK"), "{ordinary}");
            assert_eq!(json_body(&ordinary), ordinary_args);
            expected_calls.push(("getTranscript".to_string(), ordinary_args));
            assert_eq!(api.recorded_calls(), expected_calls);

            for case in box_mcp_route_cases() {
                for headers in [
                    "",
                    "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n",
                ] {
                    let before = api.recorded_calls();
                    let mut expected_authorizations = api.recorded_device_authorizations();
                    let response = box_mcp_post(
                        server.port(), case.method, Some(&device_token), headers, &case.args.to_string(),
                    );
                    assert_box_mcp_native_only(&response);
                    assert_eq!(api.recorded_calls(), before, "paired-device owner call");
                    expected_authorizations.push(authorization.clone());
                    assert_eq!(
                        api.recorded_device_authorizations(), expected_authorizations,
                        "denial must consume actual pairing-owner authorization"
                    );
                }
            }
        }
        for case in box_mcp_route_cases() {
            assert_box_mcp_round_trip(&api, server.port(), configured_token, "", &case);
        }
        server.close();
        assert_eq!(api.recorded_calls().len(), 5, "only two ordinary and three native controls");
    }
}

#[test]
fn native_box_mcp_retains_origin_and_unauthenticated_host_checks() {
    for token in [None, Some("box-native-shared-secret")] {
        let api = Arc::new(CountingBoxMcpApi::default());
        let server = start_box_mcp_gateway(&api, config(token));
        for case in box_mcp_route_cases() {
            let before = api.recorded_calls();
            let body = case.args.to_string();
            let origin = box_mcp_post(
                server.port(), case.method, token, "Origin: https://untrusted.example\r\n", &body,
            );
            assert!(origin.starts_with("HTTP/1.1 403 Forbidden"), "{origin}");
            assert_eq!(
                json_body(&origin),
                json!({"error": "browser-origin gateway requests are not allowed"})
            );
            assert_eq!(api.recorded_calls(), before);
            if token.is_none() {
                let untrusted_host = request(
                    server.port(),
                    &format!(
                        "POST /api/{} HTTP/1.1\r\nHost: untrusted.example\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        case.method, body.len()
                    ),
                );
                assert!(untrusted_host.starts_with("HTTP/1.1 403 Forbidden"), "{untrusted_host}");
                assert_eq!(json_body(&untrusted_host), json!({"error": "untrusted gateway host"}));
                assert_eq!(api.recorded_calls(), before);
            }
            assert_box_mcp_round_trip(&api, server.port(), token, "", &case);
        }
        server.close();
        assert_eq!(api.recorded_calls().len(), 3);
    }
}

#[test]
fn native_box_mcp_nonloopback_configuration_requires_authentication() {
    for token in [None, Some("box-native-shared-secret")] {
        let api = Arc::new(CountingBoxMcpApi::default());
        let mut gateway_config = config(token);
        gateway_config.host = "0.0.0.0".to_string();
        let server = start_box_mcp_gateway(&api, gateway_config);
        for case in box_mcp_route_cases() {
            if token.is_some() {
                assert_box_mcp_round_trip(&api, server.port(), token, "", &case);
            } else {
                // A loopback Host header cannot make a wildcard listener trusted.
                let response = box_mcp_post(
                    server.port(), case.method, None,
                    "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n",
                    &case.args.to_string(),
                );
                assert!(response.starts_with("HTTP/1.1 401 Unauthorized"), "{response}");
                assert_eq!(json_body(&response), json!({"error": "unauthorized"}));
                assert!(api.recorded_calls().is_empty(), "unprotected listener reached owner");
            }
        }
        if token.is_none() {
            let control = box_mcp_post(
                server.port(), "getTranscript", None, "", r#"{"listenerControl":true}"#,
            );
            assert!(control.starts_with("HTTP/1.1 401 Unauthorized"), "{control}");
            assert_eq!(json_body(&control), json!({"error": "unauthorized"}));
            assert!(api.recorded_calls().is_empty(), "ordinary wildcard request reached owner");
        }
        server.close();
        if token.is_none() {
            // Repair only the listener's trust boundary around the same owner.
            let native = start_box_mcp_gateway(&api, config(None));
            for case in box_mcp_route_cases() {
                assert_box_mcp_round_trip(&api, native.port(), None, "", &case);
            }
            let ordinary = BoxMcpRouteCase {
                method: "getTranscript",
                args: json!({"listenerControl": true}),
                result: json!({"listenerControl": true}),
            };
            assert_box_mcp_round_trip(&api, native.port(), None, "", &ordinary);
            native.close();
        }
        assert_eq!(api.recorded_calls().len(), if token.is_some() { 3 } else { 4 });
    }
}

#[test]
fn native_box_mcp_malformed_json_and_unknown_neighbors_never_reach_owner() {
    for token in [None, Some("box-native-shared-secret")] {
        let api = Arc::new(CountingBoxMcpApi::default());
        let server = start_box_mcp_gateway(&api, config(token));
        for case in box_mcp_route_cases() {
            let before = api.recorded_calls();
            let malformed = box_mcp_post(server.port(), case.method, token, "", "{");
            assert!(malformed.starts_with("HTTP/1.1 400 Bad Request"), "{malformed}");
            assert!(
                json_body(&malformed)["error"].as_str().expect("JSON parse error")
                    .starts_with("invalid JSON body:"),
                "{malformed}"
            );
            assert_eq!(api.recorded_calls(), before, "malformed JSON reached owner");

            let neighbor = format!("{}Extra", case.method);
            let unknown = box_mcp_post(server.port(), &neighbor, token, "", &case.args.to_string());
            assert!(unknown.starts_with("HTTP/1.1 404 Not Found"), "{unknown}");
            assert_eq!(
                json_body(&unknown),
                json!({"error": format!("unknown gateway method: {neighbor}")})
            );
            assert_eq!(api.recorded_calls(), before, "unknown neighbor reached owner");
            assert_box_mcp_round_trip(&api, server.port(), token, "", &case);
        }
        server.close();
        assert_eq!(api.recorded_calls().len(), 3);
    }
}

#[test]
fn native_box_mcp_owner_errors_keep_status_message_and_single_dispatch() {
    let errors: [(fn() -> GatewayCommandError, &str, &str); 4] = [
        (
            || GatewayCommandError::BadRequest("owner invalid payload".into()),
            "HTTP/1.1 400 Bad Request", "owner invalid payload",
        ),
        (
            || GatewayCommandError::Forbidden("owner permission refusal".into()),
            "HTTP/1.1 403 Forbidden", "forbidden: owner permission refusal",
        ),
        (
            || GatewayCommandError::Conflict("owner generation changed".into()),
            "HTTP/1.1 409 Conflict", "owner generation changed",
        ),
        (
            || GatewayCommandError::Internal("owner outcome unknown; do not replay".into()),
            "HTTP/1.1 500 Internal Server Error", "owner outcome unknown; do not replay",
        ),
    ];
    for token in [None, Some("box-native-shared-secret")] {
        for (error, status, message) in errors {
            let api = Arc::new(CountingBoxMcpApi {
                owner_error: Some(error),
                ..CountingBoxMcpApi::default()
            });
            let server = start_box_mcp_gateway(&api, config(token));
            let mut expected_calls = Vec::new();
            for case in box_mcp_route_cases() {
                let response = box_mcp_post(server.port(), case.method, token, "", &case.args.to_string());
                assert!(response.starts_with(status), "{}: {response}", case.method);
                assert_eq!(json_body(&response), json!({"error": message}));
                expected_calls.push((case.method.to_string(), case.args));
                assert_eq!(api.recorded_calls(), expected_calls, "owner error must not replay dispatch");
            }
            server.close();
            assert_eq!(api.recorded_calls(), expected_calls);
        }
    }
}

// LPG-v1: exercise the real HTTP registry/authority and AttachmentsService.
// Only unrelated image-generation/Box dependencies and the ordinary-route
// control are inert. Preview results always come from the actual service;
// these fresh-cache fixtures do not establish internet or desktop UI parity.
struct LinkPreviewUnusedDependencies;

impl GenerateImageAuth for LinkPreviewUnusedDependencies {
    fn get_access_token(&self) -> Result<String, String> {
        panic!("cached link preview must not request image-generation credentials")
    }

    fn get_machine_id(&self) -> Result<String, String> {
        panic!("cached link preview must not request an image-generation machine id")
    }
}

impl AttachmentsBox for LinkPreviewUnusedDependencies {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        panic!("cached link preview must not inspect a Box")
    }

    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        panic!("cached link preview must not upload to a Box")
    }
}

struct LinkPreviewCacheRecord {
    label: &'static str,
    raw_url: &'static str,
    blocked: bool,
    metadata: LinkMetadata,
    path: PathBuf,
    bytes: Vec<u8>,
}

struct LinkPreviewCache {
    root: PathBuf,
    directory: PathBuf,
    records: Vec<LinkPreviewCacheRecord>,
}

impl LinkPreviewCache {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!(
            "fabushi-native-link-preview-{}", uuid::Uuid::new_v4(),
        ));
        // Establish ownership before installing the cleanup guard. A failed
        // create must never authorize removal of an existing directory.
        fs::create_dir(&root).expect("create owned link-preview fixture directory");
        let mut cache = Self {
            directory: root.join("desktop-user-data/link-preview-cache"),
            root,
            records: Vec::new(),
        };
        let records_dir = cache.directory.join(LINK_CACHE_DIRNAME);
        fs::create_dir_all(&records_dir).expect("create preview records directory");
        for (label, raw_url, blocked) in [
            (
                "public-host",
                "https://PREVIEW.example.test:443/a/../public?fixture=host",
                false,
            ),
            (
                "public-mapped",
                "https://[::ffff:1.1.1.1]:443/a/../public?fixture=mapped",
                false,
            ),
            (
                "private-dotted",
                "https://[::ffff:127.0.0.1]/private?fixture=dotted",
                true,
            ),
            (
                "private-hex",
                "https://[::ffff:7f00:1]/private?fixture=hex",
                true,
            ),
            (
                "private-lan",
                "https://[::ffff:10.1.2.3]/private?fixture=lan",
                true,
            ),
        ] {
            // Canonicalize with ordinary URL parsing, independently of the
            // admission predicate whose private-cache refusal is under test.
            let canonical = url::Url::parse(raw_url).expect("preview fixture URL");
            let fetched_at = u64::try_from(
                SystemTime::now().duration_since(UNIX_EPOCH).expect("fixture clock").as_millis(),
            ).expect("fixture timestamp");
            let metadata = LinkMetadata {
                url: canonical.to_string(),
                canonical_url: format!("{}#canonical-{label}", canonical.as_str()),
                title: format!("LPG cached title {label}"),
                description: format!("Distinct cached description for {label} ☃"),
                site_name: format!("LPG site {label}"),
                hostname: canonical.host_str().expect("fixture hostname").to_string(),
                image_data_url: Some(concat!(
                    "data:image/png;base64,",
                    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8",
                    "AAwMCAO+aVBEAAAAASUVORK5CYII=",
                ).to_string()),
                favicon_data_url: None,
                fetched_at,
            };
            let mut record = serde_json::to_value(&metadata).expect("typed preview metadata");
            record.as_object_mut().expect("metadata object")
                .insert("cacheVersion".into(), json!(LINK_CACHE_VERSION));
            let bytes = serde_json::to_vec_pretty(&record).expect("preview cache bytes");
            let path = records_dir.join(format!(
                "{:x}.json", Sha256::digest(canonical.as_str().as_bytes()),
            ));
            fs::write(&path, &bytes).expect("seed fresh preview cache");
            cache.records.push(LinkPreviewCacheRecord {
                label, raw_url, blocked, metadata, path, bytes,
            });
        }
        cache.assert_unchanged_and_fresh();
        cache
    }

    fn args(&self, record: &LinkPreviewCacheRecord) -> Value {
        json!({"cacheDir": self.directory.to_string_lossy(), "url": record.raw_url})
    }

    fn assert_unchanged_and_fresh(&self) {
        let records_dir = self.directory.join(LINK_CACHE_DIRNAME);
        let cache_children = fs::read_dir(&self.directory).expect("preview cache directory")
            .map(|entry| entry.expect("cache directory entry").path()).collect::<Vec<_>>();
        assert_eq!(cache_children, vec![records_dir.clone()], "unexpected cache sibling");
        let mut actual_paths = fs::read_dir(&records_dir).expect("preview records directory")
            .map(|entry| entry.expect("preview record entry").path()).collect::<Vec<_>>();
        let mut expected_paths = self.records.iter().map(|record| record.path.clone())
            .collect::<Vec<_>>();
        actual_paths.sort();
        expected_paths.sort();
        assert_eq!(actual_paths, expected_paths, "preview must not add or delete cache records");
        let now = u64::try_from(
            SystemTime::now().duration_since(UNIX_EPOCH).expect("cache check clock").as_millis(),
        ).expect("cache check timestamp");
        for record in &self.records {
            let bytes = fs::read(&record.path).expect("preview cache remains readable");
            assert_eq!(bytes, record.bytes, "{} cache bytes changed", record.label);
            let saved: Value = serde_json::from_slice(&bytes).expect("valid cache JSON");
            assert_eq!(saved["cacheVersion"], json!(LINK_CACHE_VERSION));
            let saved_metadata: LinkMetadata = serde_json::from_slice(&bytes)
                .expect("complete typed cache metadata");
            assert_eq!(saved_metadata, record.metadata, "{} metadata", record.label);
            let age = now.checked_sub(saved_metadata.fetched_at)
                .expect("cache timestamp must not be in the future");
            assert!(age < LINK_CACHE_TTL_MS, "{} fixture must remain fresh", record.label);
        }
    }
}

impl Drop for LinkPreviewCache {
    fn drop(&mut self) {
        let cleanup = fs::remove_dir_all(&self.root);
        if !thread::panicking() {
            cleanup.expect("remove owned link-preview fixture directory");
        }
    }
}

struct LinkPreviewApi {
    attachments: AttachmentsService,
    calls: Mutex<Vec<(String, Value)>>,
    pairing_owner: Option<Arc<DevicePairingOwner>>,
    device_authorizations: Mutex<Vec<DeviceTokenAuthorization>>,
    // Keep owned files alive with the actual service, including a connection
    // unwinding after a failed client assertion.
    _cache: Arc<LinkPreviewCache>,
}

impl LinkPreviewApi {
    fn recorded_calls(&self) -> Vec<(String, Value)> {
        self.calls.lock().expect("preview owner calls").clone()
    }

    fn recorded_device_authorizations(&self) -> Vec<DeviceTokenAuthorization> {
        self.device_authorizations.lock().expect("preview device authorizations").clone()
    }
}

impl GatewayApi for LinkPreviewApi {
    fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        let Some(owner) = self.pairing_owner.as_ref() else {
            return Ok(None);
        };
        let authorization = owner.authenticate_device_token(token)?;
        if let Some(authorization) = authorization.as_ref() {
            self.device_authorizations.lock().expect("record preview device authorization")
                .push(authorization.clone());
        }
        Ok(authorization)
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.calls.lock().expect("record preview owner dispatch")
            .push((method.to_string(), args.clone()));
        if method == "getTranscript" {
            return Ok(args);
        }
        // Match the actual Host app's dispatch_gateway_call composition,
        // including its existing String -> Internal error classification.
        match self.attachments.dispatch_gateway(method, &args) {
            Some(result) => result.map_err(GatewayCommandError::Internal),
            None => Err(GatewayCommandError::UnknownMethod(method.to_string())),
        }
    }
}

struct LinkPreviewGateway {
    // GatewayServer owns listener shutdown/join. Drop it before the service and
    // cache guard, including during unwinding. The existing HTTP helper drains
    // each connection to EOF; this fixture starts no additional worker threads.
    server: GatewayServer,
    api: Arc<LinkPreviewApi>,
    cache: Arc<LinkPreviewCache>,
}

impl LinkPreviewGateway {
    fn new(
        cache: Arc<LinkPreviewCache>,
        gateway_config: GatewayServerConfig,
        pairing_owner: Option<Arc<DevicePairingOwner>>,
    ) -> Self {
        let api = Arc::new(LinkPreviewApi {
            attachments: AttachmentsService::new(
                cache.root.clone(),
                Arc::new(LinkPreviewUnusedDependencies),
                Arc::new(LinkPreviewUnusedDependencies),
                None,
            ),
            calls: Mutex::new(Vec::new()),
            pairing_owner,
            device_authorizations: Mutex::new(Vec::new()),
            _cache: cache.clone(),
        });
        Self::with_api(cache, gateway_config, api)
    }

    fn with_api(
        cache: Arc<LinkPreviewCache>,
        gateway_config: GatewayServerConfig,
        api: Arc<LinkPreviewApi>,
    ) -> Self {
        let server = start_gateway_server(GatewayServerDeps {
            api: api.clone(),
            events: GatewayEventHub::default(),
            local_exec: None,
            webauthn: None,
            config: gateway_config,
            started_at: 1,
        }).expect("real link-preview GatewayServer");
        Self { server, api, cache }
    }

    fn assert_preview(
        &self,
        record: &LinkPreviewCacheRecord,
        bearer: Option<&str>,
        headers: &str,
    ) {
        self.cache.assert_unchanged_and_fresh();
        let args = self.cache.args(record);
        let mut expected_calls = self.api.recorded_calls();
        let response = box_mcp_post(
            self.server.port(), "fetchLinkMetadata", bearer, headers, &args.to_string(),
        );
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{}: {response}", record.label);
        let expected = if record.blocked {
            Value::Null
        } else {
            serde_json::to_value(&record.metadata).expect("expected preview metadata")
        };
        assert_eq!(json_body(&response), expected, "{} preview result", record.label);
        expected_calls.push(("fetchLinkMetadata".to_string(), args));
        assert_eq!(self.api.recorded_calls(), expected_calls, "preview must dispatch exactly once");
        self.cache.assert_unchanged_and_fresh();
    }

    fn assert_native_control(&self, bearer: Option<&str>) {
        self.assert_preview(&self.cache.records[0], bearer, "");
    }

    fn assert_ordinary_control(&self, bearer: Option<&str>, args: Value) {
        let mut expected_calls = self.api.recorded_calls();
        let response = box_mcp_post(
            self.server.port(), "getTranscript", bearer, "", &args.to_string(),
        );
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
        assert_eq!(json_body(&response), args);
        expected_calls.push(("getTranscript".to_string(), args));
        assert_eq!(self.api.recorded_calls(), expected_calls, "ordinary-route control dispatch");
        self.cache.assert_unchanged_and_fresh();
    }

    fn assert_no_dispatch(
        &self,
        method: &str,
        bearer: Option<&str>,
        headers: &str,
        body: &str,
        status: &str,
    ) -> Value {
        self.cache.assert_unchanged_and_fresh();
        let before = self.api.recorded_calls();
        let response = box_mcp_post(self.server.port(), method, bearer, headers, body);
        assert!(response.starts_with(status), "{method}: {response}");
        assert_eq!(self.api.recorded_calls(), before, "refusal must precede owner access");
        self.cache.assert_unchanged_and_fresh();
        json_body(&response)
    }
}

fn assert_link_preview_native_only(error: Value) {
    assert_eq!(error, json!({
        "error": "ATTACHMENTS_NATIVE_ONLY: Link preview requires the native control-plane gateway",
    }));
}

#[test]
fn native_link_preview_http_routes_preserve_fresh_cache_and_mapped_ip_admission() {
    for token in [None, Some("preview-native-shared-secret")] {
        let cache = Arc::new(LinkPreviewCache::new());
        let gateway = LinkPreviewGateway::new(cache.clone(), config(token), None);
        for record in &cache.records {
            for headers in [
                "",
                "x-sand-role: operator\r\n",
                "x-sand-scope: operator.write\r\n",
                "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n",
            ] {
                gateway.assert_preview(record, token, headers);
                if record.blocked {
                    gateway.assert_native_control(token);
                }
            }
        }
        // A new HTTP server and a new actual service reuse the same disk
        // records; no accepted-job state or policy memo survives the restart.
        drop(gateway);
        cache.assert_unchanged_and_fresh();
        let restarted = LinkPreviewGateway::new(cache.clone(), config(token), None);
        for record in &cache.records {
            restarted.assert_preview(record, token, "");
            if record.blocked {
                restarted.assert_native_control(token);
            }
        }
    }
}

#[test]
fn native_link_preview_configured_secret_cannot_be_replaced_by_headers() {
    let secret = "preview-native-shared-secret";
    let cache = Arc::new(LinkPreviewCache::new());
    let gateway = LinkPreviewGateway::new(cache.clone(), config(Some(secret)), None);
    let valid_body = cache.args(&cache.records[0]).to_string();
    for bearer in [None, Some("wrong-preview-native-secret")] {
        for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n"] {
            for body in [valid_body.as_str(), "{"] {
                let error = gateway.assert_no_dispatch(
                    "fetchLinkMetadata", bearer, headers, body, "HTTP/1.1 401 Unauthorized",
                );
                assert_eq!(error, json!({"error": "unauthorized"}));
                gateway.assert_native_control(Some(secret));
            }
        }
    }
}

#[test]
fn native_link_preview_rejects_restricted_unknown_and_empty_role_or_scope_before_owner() {
    for token in [None, Some("preview-native-shared-secret")] {
        let cache = Arc::new(LinkPreviewCache::new());
        let gateway = LinkPreviewGateway::new(cache.clone(), config(token), None);
        let valid_body = cache.args(&cache.records[0]).to_string();
        for headers in [
            "x-sand-role: node\r\n",
            "x-sand-role: unknown-role\r\n",
            "x-sand-role: Operator\r\n",
            "x-sand-role: \r\n",
            "x-sand-role: operator\r\nx-sand-scope: operator.read\r\n",
            "x-sand-scope: operator.read\r\n",
            "x-sand-scope: unknown-scope\r\n",
            "x-sand-scope: operator.write operator.read\r\n",
            "x-sand-scope: \r\n",
        ] {
            for body in [valid_body.as_str(), "{"] {
                assert_link_preview_native_only(gateway.assert_no_dispatch(
                    "fetchLinkMetadata", token, headers, body, "HTTP/1.1 403 Forbidden",
                ));
                gateway.assert_native_control(token);
            }
        }
    }
}

#[test]
fn native_link_preview_rejects_real_node_and_operator_write_device_tokens_before_owner() {
    for configured_token in [None, Some("preview-native-shared-secret")] {
        let cache = Arc::new(LinkPreviewCache::new());
        let owner = Arc::new(DevicePairingOwner::in_memory().expect("preview pairing owner"));
        let tokens = [
            issue_gateway_device_token(
                &owner, "preview-node", "preview-node-public-key", "node", &[],
            ),
            issue_gateway_device_token(
                &owner, "preview-operator", "preview-operator-public-key", "operator",
                &["operator.read", "operator.write"],
            ),
        ];
        let gateway = LinkPreviewGateway::new(cache.clone(), config(configured_token), Some(owner));
        let valid_body = cache.args(&cache.records[0]).to_string();
        for (device_token, authorization) in tokens {
            // This issued token is live and accepted on an ordinary route;
            // preview denial must not be caused by a fabricated/bad token.
            gateway.assert_ordinary_control(
                Some(&device_token), json!({"pairedDeviceControl": authorization.device_id}),
            );
            for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n"] {
                for body in [valid_body.as_str(), "{"] {
                    let mut expected_authorizations = gateway.api.recorded_device_authorizations();
                    assert_link_preview_native_only(gateway.assert_no_dispatch(
                        "fetchLinkMetadata", Some(&device_token), headers, body,
                        "HTTP/1.1 403 Forbidden",
                    ));
                    expected_authorizations.push(authorization.clone());
                    assert_eq!(
                        gateway.api.recorded_device_authorizations(), expected_authorizations,
                        "refusal must consume actual pairing-owner authorization",
                    );
                    gateway.assert_native_control(configured_token);
                }
            }
        }
    }
}

#[test]
fn native_link_preview_retains_origin_and_unauthenticated_host_checks() {
    for token in [None, Some("preview-native-shared-secret")] {
        let cache = Arc::new(LinkPreviewCache::new());
        let gateway = LinkPreviewGateway::new(cache.clone(), config(token), None);
        let body = cache.args(&cache.records[0]).to_string();
        let error = gateway.assert_no_dispatch(
            "fetchLinkMetadata", token, "Origin: https://untrusted.example\r\n", &body,
            "HTTP/1.1 403 Forbidden",
        );
        assert_eq!(error, json!({"error": "browser-origin gateway requests are not allowed"}));
        gateway.assert_native_control(token);
        if token.is_none() {
            let before = gateway.api.recorded_calls();
            let untrusted = request(gateway.server.port(), &format!(
                "POST /api/fetchLinkMetadata HTTP/1.1\r\nHost: untrusted.example\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len(),
            ));
            assert!(untrusted.starts_with("HTTP/1.1 403 Forbidden"), "{untrusted}");
            assert_eq!(json_body(&untrusted), json!({"error": "untrusted gateway host"}));
            assert_eq!(gateway.api.recorded_calls(), before, "untrusted Host reached owner");
            cache.assert_unchanged_and_fresh();
            gateway.assert_native_control(token);
        }
    }
}

#[test]
fn native_link_preview_nonloopback_configuration_requires_authentication() {
    for token in [None, Some("preview-native-shared-secret")] {
        let cache = Arc::new(LinkPreviewCache::new());
        let mut gateway_config = config(token);
        gateway_config.host = "0.0.0.0".to_string();
        let gateway = LinkPreviewGateway::new(cache.clone(), gateway_config, None);
        if token.is_some() {
            gateway.assert_native_control(token);
        } else {
            let body = cache.args(&cache.records[0]).to_string();
            assert_eq!(gateway.assert_no_dispatch(
                "fetchLinkMetadata", None,
                "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n", &body,
                "HTTP/1.1 401 Unauthorized",
            ), json!({"error": "unauthorized"}));
            assert_eq!(gateway.assert_no_dispatch(
                "getTranscript", None, "", r#"{"listenerControl":true}"#,
                "HTTP/1.1 401 Unauthorized",
            ), json!({"error": "unauthorized"}));
            let api = gateway.api.clone();
            drop(gateway);
            // Restore a trusted listener around the same actual service, so
            // the native positive control is not a replacement owner.
            let native = LinkPreviewGateway::with_api(cache.clone(), config(None), api);
            native.assert_native_control(None);
            native.assert_ordinary_control(None, json!({"listenerControl": true}));
        }
    }
}

#[test]
fn native_link_preview_malformed_json_and_unknown_neighbors_never_reach_owner() {
    for token in [None, Some("preview-native-shared-secret")] {
        let cache = Arc::new(LinkPreviewCache::new());
        let gateway = LinkPreviewGateway::new(cache.clone(), config(token), None);
        let malformed = gateway.assert_no_dispatch(
            "fetchLinkMetadata", token, "", "{", "HTTP/1.1 400 Bad Request",
        );
        assert!(
            malformed["error"].as_str().expect("JSON error").starts_with("invalid JSON body:"),
            "{malformed}",
        );
        gateway.assert_native_control(token);
        for neighbor in [
            "fetchLinkMetadataExtra", "fetchLinkMetadata.raw", "getLinkMetadata",
            "readRemoteMediaBuffer", "saveRemoteMedia", "saveMediaSource",
        ] {
            let error = gateway.assert_no_dispatch(
                neighbor, token, "", "{}", "HTTP/1.1 404 Not Found",
            );
            assert_eq!(error, json!({"error": format!("unknown gateway method: {neighbor}")}));
            gateway.assert_native_control(token);
        }
    }
}

#[test]
fn native_link_preview_service_argument_errors_keep_status_and_single_dispatch() {
    for token in [None, Some("preview-native-shared-secret")] {
        let cache = Arc::new(LinkPreviewCache::new());
        let gateway = LinkPreviewGateway::new(cache.clone(), config(token), None);
        let valid = cache.args(&cache.records[0]);
        for (args, message) in [
            (Value::Null, "fetchLinkMetadata.cacheDir must be a string"),
            (json!({}), "fetchLinkMetadata.cacheDir must be a string"),
            (
                json!({"cacheDir": 7, "url": valid["url"]}),
                "fetchLinkMetadata.cacheDir must be a string",
            ),
            (
                json!({"cacheDir": valid["cacheDir"]}),
                "fetchLinkMetadata.url must be a string",
            ),
            (
                json!({"cacheDir": valid["cacheDir"], "url": false}),
                "fetchLinkMetadata.url must be a string",
            ),
        ] {
            let mut expected_calls = gateway.api.recorded_calls();
            let response = box_mcp_post(
                gateway.server.port(), "fetchLinkMetadata", token, "", &args.to_string(),
            );
            assert!(response.starts_with("HTTP/1.1 500 Internal Server Error"), "{response}");
            assert_eq!(json_body(&response), json!({"error": message}));
            expected_calls.push(("fetchLinkMetadata".to_string(), args));
            assert_eq!(gateway.api.recorded_calls(), expected_calls, "service error must not replay");
            cache.assert_unchanged_and_fresh();
            gateway.assert_native_control(token);
        }
    }
}

#[test]
fn gateway_health_command_security_and_upgrade_routes_match_grok_contract() {
    let events = GatewayEventHub::default();
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events,
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 777,
    })
    .expect("gateway server");
    let port = server.port();

    let health = request(
        port,
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(health.starts_with("HTTP/1.1 200 OK"), "{health}");
    let health_json = json_body(&health);
    assert_eq!(health_json["ok"], true);
    assert_eq!(health_json["isBusy"], true);
    assert_eq!(health_json["busyOnlyAwaitingApproval"], false);
    assert_eq!(health_json["activeAgentId"], "agent-1");
    assert_eq!(health_json["startedAt"], 777);

    let body = r#"{"value":"hello"}"#;
    let transcript = request(
        port,
        &format!(
            "POST /api/getTranscript HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(transcript.starts_with("HTTP/1.1 200 OK"), "{transcript}");
    assert!(
        transcript.to_ascii_lowercase()
            .contains("x-sand-mint-dedupe: 1"),
        "{transcript}"
    );
    assert_eq!(json_body(&transcript), json!({ "value": "hello" }));

    let unknown = request(
        port,
        "POST /api/noSuchMethod HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(unknown.starts_with("HTTP/1.1 404 Not Found"), "{unknown}");
    assert!(
        !unknown
            .to_ascii_lowercase()
            .contains("x-sand-mint-dedupe:"),
        "error responses must not carry the successful command dedupe marker: {unknown}"
    );

    let upgrade = request(
        port,
        "POST /prepare-upgrade HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(upgrade.starts_with("HTTP/1.1 200 OK"), "{upgrade}");
    assert_eq!(
        json_body(&upgrade),
        json!({ "quiescing": true, "runningTurns": 2 })
    );

    let origin = request(
        port,
        "GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: https://example.com\r\nConnection: close\r\n\r\n",
    );
    assert!(origin.starts_with("HTTP/1.1 403 Forbidden"), "{origin}");

    let untrusted_host = request(
        port,
        "GET /health HTTP/1.1\r\nHost: attacker.example\r\nConnection: close\r\n\r\n",
    );
    assert!(
        untrusted_host.starts_with("HTTP/1.1 403 Forbidden"),
        "{untrusted_host}"
    );

    server.close();
}


#[test]
fn gateway_serves_real_tls_when_cert_and_key_are_configured() {
    let certified = generate_simple_self_signed(vec!["localhost".to_string()])
        .expect("generate TLS certificate");
    let cert_pem = certified.cert.pem();
    let key_pem = certified.key_pair.serialize_pem();

    let mut roots = RootCertStore::empty();
    roots
        .add(certified.cert.der().clone())
        .expect("trust generated TLS certificate");
    let client = ClientConfig::builder()
        .with_root_certificates(roots)
        .with_no_client_auth();

    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: GatewayServerConfig {
            host: "127.0.0.1".into(),
            port: None,
            auth_token: None,
            tls: Some(GatewayTlsConfig {
                cert: cert_pem.into_bytes(),
                key: key_pem.into_bytes(),
            }),
        },
        started_at: 321,
    })
    .expect("TLS gateway server");

    let socket = TcpStream::connect(("127.0.0.1", server.port())).expect("connect TLS gateway");
    socket
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("TLS read timeout");
    let name = ServerName::try_from("localhost")
        .expect("valid TLS server name")
        .to_owned();
    let connection =
        ClientConnection::new(Arc::new(client), name).expect("create TLS client connection");
    let mut stream = StreamOwned::new(connection, socket);
    stream
        .write_all(b"GET /health HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n")
        .expect("write TLS health request");
    stream.flush().expect("flush TLS health request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read TLS health response");
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    assert_eq!(json_body(&response)["startedAt"], 321);

    server.close();
}

#[test]
fn gateway_bearer_auth_is_required_when_configured() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(Some("secret")),
        started_at: 1,
    })
    .expect("gateway server");
    let port = server.port();

    let missing = request(
        port,
        "POST /api/getTranscript HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(missing.starts_with("HTTP/1.1 401 Unauthorized"), "{missing}");
    assert!(
        !missing
            .to_ascii_lowercase()
            .contains("x-sand-mint-dedupe:"),
        "authentication errors must not look like successful minted responses: {missing}"
    );

    let authorized = request(
        port,
        "POST /api/getTranscript HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer secret\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(
        authorized.starts_with("HTTP/1.1 200 OK"),
        "{authorized}"
    );
    assert_eq!(json_body(&authorized), json!({}));

    server.close();
}

#[test]
fn gateway_carries_explicit_fabushi_extensions_without_widening_unknown_methods() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 777,
    })
    .expect("gateway server");
    let port = server.port();

    let extensions = [
        "feature.info",
        "feature.execute",
        "feature.marketplace.browse",
        "feature.marketplace.release",
        "feature.plugin.install",
        "feature.plugin.uninstall",
        "feature.plugin.rollback",
        "feature.plugin.active",
        "feature.plugin.listInstalled",
        "feature.plugin.uiDocument",
        "feature.auth.status",
        "feature.auth.providers",
        "feature.auth.browserStart",
        "feature.auth.browserPoll",
        "feature.auth.browserCancel",
        "feature.auth.browserReopen",
        "feature.auth.passwordLogin",
        "feature.auth.oauthStart",
        "feature.auth.oauthPoll",
        "feature.auth.logout",
        "feature.interrupt",
        "feature.approval.resolve",
    ];

    for method in extensions {
        let response = request(
            port,
            &format!(
                "POST /api/{method} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
            ),
        );
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "{method} was not admitted through the explicit Fabushi extension boundary: {response}"
        );
        assert_eq!(json_body(&response)["method"], method);
    }
}


#[test]
fn gateway_mcp_lifecycle_receipts_require_native_auth_not_paired_operator_write() {
    for configured_token in [None, Some("native-mcp-gateway-secret")] {
        let owner = Arc::new(DevicePairingOwner::in_memory().expect("MCP pairing owner"));
        let issued = [
            issue_box_mcp_device_token(&owner, "node"),
            issue_box_mcp_device_token(&owner, "operator"),
        ];
        let api = Arc::new(CountingBoxMcpApi {
            pairing_owner: Some(owner),
            ..CountingBoxMcpApi::default()
        });
        let server = start_box_mcp_gateway(&api, config(configured_token));
        for (token, authorization) in issued {
            // Even an actual operator.write device credential is not the
            // Coordinator's native MCP lifecycle settlement authority.
            for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n"] {
                let before = api.recorded_calls();
                let response = box_mcp_post(
                    server.port(), "mcp.resolveLifecycleRequest", Some(&token),
                    headers, r#"{"requestId":"forged","ok":true}"#,
                );
                assert!(response.starts_with("HTTP/1.1 403 Forbidden"), "{response}");
                assert!(response.contains("MCP_LIFECYCLE_NATIVE_ONLY"), "{response}");
                assert_eq!(api.recorded_calls(), before, "paired-device receipt reached owner");
            }
            assert!(!authorization.device_id.is_empty());
        }
        server.close();
    }

    let secret = "native-mcp-gateway-secret";
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi), events: GatewayEventHub::default(),
        local_exec: None, webauthn: None, config: config(Some(secret)), started_at: 1,
    }).expect("native MCP gateway");
    let denied = box_mcp_post(server.port(), "mcp.resolveLifecycleRequest", None, "", "{}");
    assert!(denied.starts_with("HTTP/1.1 401 Unauthorized"), "{denied}");
    let admitted = box_mcp_post(
        server.port(), "mcp.resolveLifecycleRequest", Some(secret), "", "{}",
    );
    assert!(admitted.starts_with("HTTP/1.1 200 OK"), "{admitted}");
    assert_eq!(json_body(&admitted)["method"], "mcp.resolveLifecycleRequest");
    server.close();
}

// This fixture controls only remote replies. Authentication, HTTP admission,
// pending request ownership and completion use the production owners.
struct McpControlApi {
    pairing_owner: DevicePairingOwner,
    lifecycle: Arc<CoordinatorMcpLifecycleRelay>,
    tools: Arc<CoordinatorToolRelay>,
    calls: Mutex<Vec<(String, Value)>>,
}

impl McpControlApi {
    fn recorded_calls(&self) -> Vec<(String, Value)> {
        self.calls.lock().expect("MCP control calls").clone()
    }
}

impl GatewayApi for McpControlApi {
    fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.pairing_owner.authenticate_device_token(token)
    }

    fn validate_paired_device_authorization(
        &self,
        authorization: &DeviceTokenAuthorization,
    ) -> Result<(), GatewayCommandError> {
        self.pairing_owner.validate_current_device_authorization(authorization)
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.calls
            .lock()
            .expect("record MCP control delegate")
            .push((method.to_string(), args.clone()));
        match method {
            MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD => self.lifecycle.resolve(&args)
                .map_err(|error| GatewayCommandError::BadRequest(error.to_string())),
            RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD => self.tools.resolve(&args)
                .map_err(|error| GatewayCommandError::BadRequest(error.to_string())),
            "uninstallMcpPlugin" | "refreshMcp" | "syncPluginSkills" | "getPluginSyncStatus" => {
                Ok(json!({"method": method, "args": args}))
            }
            _ => Err(GatewayCommandError::UnknownMethod(method.to_string())),
        }
    }
}

struct McpControlGateway {
    server: GatewayServer,
    api: Arc<McpControlApi>,
    events: GatewayEventHub,
    devices: Vec<(String, DeviceTokenAuthorization)>,
}

impl McpControlGateway {
    fn new(configured_token: Option<&str>) -> Self {
        let events = GatewayEventHub::default();
        let pairing_owner = DevicePairingOwner::in_memory().expect("MCP test pairing owner");
        let devices = [
            ("mcp-read", "operator", vec!["operator.read"]),
            ("mcp-write", "operator", vec!["operator.read", "operator.write"]),
            ("mcp-node", "node", vec![]),
        ].into_iter().map(|(device_id, role, scopes)| {
            issue_gateway_device_token(
                &pairing_owner, device_id, &format!("{device_id}-public-key"), role, &scopes,
            )
        }).collect();
        let api = Arc::new(McpControlApi {
            pairing_owner,
            lifecycle: Arc::new(CoordinatorMcpLifecycleRelay::with_timeout(
                events.clone(), Duration::from_secs(60),
            )),
            tools: Arc::new(CoordinatorToolRelay::with_timeout(
                events.clone(), Duration::from_secs(60),
            )),
            calls: Mutex::new(Vec::new()),
        });
        let server = start_gateway_server(GatewayServerDeps {
            api: api.clone(),
            events: events.clone(),
            local_exec: None,
            webauthn: None,
            config: config(configured_token),
            started_at: 1,
        }).expect("MCP control gateway");
        Self { server, api, events, devices }
    }
}

impl Drop for McpControlGateway {
    fn drop(&mut self) {
        // Wake actual relay waiters even when an assertion unwinds the fixture.
        self.api.lifecycle.cancel_all("MCP gateway contract fixture closed");
        self.api.tools.cancel_all("MCP gateway contract fixture closed");
    }
}

const MCP_CONTROL_RESOLVERS: [&str; 2] = [
    MCP_RESOLVE_LIFECYCLE_GATEWAY_METHOD,
    RUNNER_RESOLVE_ROUTED_TOOL_GATEWAY_METHOD,
];

struct PendingMcpControlRequests {
    events: [Value; 2],
    replies: [mpsc::Receiver<Result<Value, String>>; 2],
    workers: Vec<thread::JoinHandle<()>>,
}

impl PendingMcpControlRequests {
    fn start(gateway: &McpControlGateway) -> Self {
        let published = gateway.events.subscribe();
        let backend = CoordinatorMcpManagerBackend::new(
            gateway.api.lifecycle.clone(), Arc::new(|_, _| Ok(Vec::new())),
        );
        let (lifecycle_tx, lifecycle_rx) = mpsc::channel();
        let lifecycle_worker = thread::spawn(move || {
            let result = backend.authenticate_server(
                "fixture-server", "fixture-account", Some("fixture-agent"), true,
            );
            let _ = lifecycle_tx.send(result);
        });
        let lifecycle_event = published.recv_timeout(Duration::from_secs(5))
            .expect("actual backend published its lifecycle request");
        assert_eq!(lifecycle_event["channel"], HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL);
        assert_eq!(lifecycle_event["payload"]["method"], MCP_AUTHENTICATE_SERVER_METHOD);
        assert_eq!(lifecycle_event["payload"]["args"], json!({
            "serverId": "fixture-server",
            "accountKey": "fixture-account",
            "requestingAgentId": "fixture-agent",
            "forceReauth": true,
        }));

        let tools = gateway.api.tools.clone();
        let tool_args = json!({"toolName": "fixture-tool", "args": {"opaque": ["a", 7, null]}});
        let expected_tool_args = tool_args.clone();
        let (tool_tx, tool_rx) = mpsc::channel();
        let tool_worker = thread::spawn(move || {
            let result = tools.request_tool_execution(tool_args).map_err(|error| error.to_string());
            let _ = tool_tx.send(result);
        });
        let tool_event = published.recv_timeout(Duration::from_secs(5))
            .expect("actual execution relay published its tool request");
        assert_eq!(tool_event["channel"], RUNNER_TOOL_REQUEST_EVENT_CHANNEL);
        assert_eq!(tool_event["payload"]["method"], ROUTED_TOOL_EXECUTE_METHOD);
        assert_eq!(tool_event["payload"]["args"], expected_tool_args);
        let pending = Self {
            events: [lifecycle_event, tool_event],
            replies: [lifecycle_rx, tool_rx],
            workers: vec![lifecycle_worker, tool_worker],
        };
        assert!(pending.events[0]["payload"]["requestId"].as_str()
            .expect("real lifecycle id").starts_with("host-mcp-"));
        assert!(pending.events[1]["payload"]["requestId"].as_str()
            .expect("real tool id").starts_with("runner-tool-"));
        assert_ne!(pending.events[0]["payload"]["requestId"], pending.events[1]["payload"]["requestId"]);
        pending.assert_pending(gateway);
        pending
    }

    fn assert_pending(&self, gateway: &McpControlGateway) {
        assert_eq!(gateway.api.lifecycle.pending_count(), 1, "lifecycle waiter retained");
        assert_eq!(gateway.api.tools.pending_count(), 1, "execution waiter retained");
        for reply in &self.replies {
            assert!(matches!(reply.try_recv(), Err(mpsc::TryRecvError::Empty)),
                "refused completion must not deliver a result");
        }
    }

    fn complete(self, gateway: &McpControlGateway, native_bearer: Option<&str>) {
        let results = [
            json!({"authenticated": true, "state": {"opaque": [null, "native", 9]}}),
            json!({
                "content": [{"type": "text", "text": "original native tool completion"}],
                "structuredContent": {"opaque": [null, "native", 9]},
                "isError": false,
            }),
        ];
        let mut expected_calls = gateway.api.recorded_calls();
        for (index, method) in MCP_CONTROL_RESOLVERS.into_iter().enumerate() {
            let request_id = self.events[index]["payload"]["requestId"].as_str().expect("real request id");
            let args = json!({"requestId": request_id, "ok": true, "result": results[index]});
            let response = box_mcp_post(
                gateway.server.port(), method, native_bearer,
                "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n", &args.to_string(),
            );
            assert!(response.starts_with("HTTP/1.1 200 OK"), "{method}: {response}");
            assert_eq!(json_body(&response), json!({"resolved": true, "requestId": request_id}));
            expected_calls.push((method.to_string(), args.clone()));
            assert_eq!(gateway.api.recorded_calls(), expected_calls, "one native delegate per request");
            assert_eq!(
                self.replies[index].recv_timeout(Duration::from_secs(5))
                    .expect("native completion reaches the original waiter")
                    .expect("native completion has a definitive successful result"),
                results[index],
                "{method} must return the original value",
            );

            let duplicate = box_mcp_post(
                gateway.server.port(), method, native_bearer, "", &args.to_string(),
            );
            assert!(duplicate.starts_with("HTTP/1.1 400 Bad Request"), "{duplicate}");
            assert!(json_body(&duplicate)["error"].as_str().expect("duplicate error")
                .contains("unknown or already-settled"), "{duplicate}");
            expected_calls.push((method.to_string(), args));
            assert_eq!(gateway.api.recorded_calls(), expected_calls, "duplicate is rejected by the real relay");
        }
        for worker in self.workers {
            worker.join().expect("actual relay request worker");
        }
        for reply in self.replies {
            assert!(matches!(reply.try_recv(), Err(mpsc::TryRecvError::Disconnected)),
                "each real waiter must receive exactly one completion");
        }
        assert_eq!(gateway.api.lifecycle.pending_count(), 0);
        assert_eq!(gateway.api.tools.pending_count(), 0);
    }
}

fn assert_mcp_management_request(
    gateway: &McpControlGateway,
    method: &str,
    args: &Value,
    bearer: Option<&str>,
    headers: &str,
    denial: Option<&str>,
) {
    let mut expected_calls = gateway.api.recorded_calls();
    let response = box_mcp_post(
        gateway.server.port(), method, bearer, headers, &args.to_string(),
    );
    if let Some(code) = denial {
        assert!(response.starts_with("HTTP/1.1 403 Forbidden"), "{method}: {response}");
        assert!(json_body(&response)["error"].as_str().expect("management error")
            .contains(code), "{response}");
    } else {
        assert!(response.starts_with("HTTP/1.1 200 OK"), "{method}: {response}");
        assert_eq!(json_body(&response), json!({"method": method, "args": args}));
        expected_calls.push((method.to_string(), args.clone()));
    }
    assert_eq!(gateway.api.recorded_calls(), expected_calls,
        "{method}: refusals dispatch zero calls; admitted intents dispatch once with original arguments");
}

#[test]
fn desktop_mcp_management_uses_issued_operator_scopes_without_header_escalation() {
    for configured_token in [None, Some("native-mcp-management-secret")] {
        let gateway = McpControlGateway::new(configured_token);
        for (method, args) in [
            ("uninstallMcpPlugin", json!({"pluginId": "official-plugin"})),
            ("refreshMcp", json!({})),
            ("syncPluginSkills", json!({})),
            ("getPluginSyncStatus", json!({})),
        ] {
            let mutating = method != "getPluginSyncStatus";
            for (token, authorization) in &gateway.devices {
                let denial = if authorization.role == "node" {
                    Some("ROLE_MISMATCH")
                } else if mutating && !authorization.scopes.iter().any(|scope| scope == "operator.write") {
                    Some("INSUFFICIENT_SCOPE")
                } else {
                    None
                };
                for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n"] {
                    assert_mcp_management_request(
                        &gateway, method, &args, Some(token), headers, denial,
                    );
                }
            }
            for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n"] {
                assert_mcp_management_request(
                    &gateway, method, &args, configured_token, headers, None,
                );
            }
            assert_mcp_management_request(
                &gateway, method, &args, configured_token, "x-sand-role: node\r\n",
                Some("ROLE_MISMATCH"),
            );
            assert_mcp_management_request(
                &gateway, method, &args, configured_token, "x-sand-scope: operator.read\r\n",
                mutating.then_some("INSUFFICIENT_SCOPE"),
            );
        }
    }
}

fn assert_mcp_receipts_refused(
    gateway: &McpControlGateway,
    pending: &PendingMcpControlRequests,
    bearer: Option<&str>,
    headers: &str,
    status: &str,
    common_error: Option<&str>,
) {
    for (index, method) in MCP_CONTROL_RESOLVERS.into_iter().enumerate() {
        // Both IDs were emitted by live production relays. This is a valid
        // definitive MCP result, so neither correlation nor result validation
        // can explain the caller's refusal.
        let args = json!({
            "requestId": pending.events[index]["payload"]["requestId"],
            "ok": true,
            "result": {
                "content": [{"type": "text", "text": "unauthorized fixture completion"}],
                "structuredContent": {"fixture": true},
                "isError": false,
            },
        });
        let response = box_mcp_post(
            gateway.server.port(), method, bearer, headers, &args.to_string(),
        );
        assert!(response.starts_with(status), "{method}: {response}");
        let expected_error = common_error.unwrap_or(if index == 0 {
            "MCP_LIFECYCLE_NATIVE_ONLY"
        } else {
            "RUNNER_TOOL_NATIVE_ONLY"
        });
        assert!(json_body(&response)["error"].as_str().expect("receipt refusal")
            .contains(expected_error), "{response}");
        assert!(gateway.api.recorded_calls().is_empty(), "refused receipt reached a resolver");
        pending.assert_pending(gateway);
    }
}

#[test]
fn internal_mcp_receipts_preserve_real_waiters_until_native_completion() {
    for (configured_token, native_bearer) in [
        (None, None),
        (Some("native-mcp-receipt-secret"), Some("native-mcp-receipt-secret")),
    ] {
        let gateway = McpControlGateway::new(configured_token);
        let pending = PendingMcpControlRequests::start(&gateway);
        // An explicit failed credential is never credential-absent native mode.
        assert_mcp_receipts_refused(
            &gateway, &pending, Some("unrecognized-local-bearer"), "",
            "HTTP/1.1 401 Unauthorized", Some("unauthorized"),
        );
        for (token, _) in &gateway.devices {
            for headers in ["", "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n"] {
                assert_mcp_receipts_refused(
                    &gateway, &pending, Some(token), headers, "HTTP/1.1 403 Forbidden", None,
                );
            }
        }
        for headers in [
            "x-sand-role: node\r\n",
            "x-sand-role: unknown\r\n",
            "x-sand-role: \r\n",
            "x-sand-scope: operator.read\r\n",
            "x-sand-scope: unknown\r\n",
            "x-sand-scope: \r\n",
        ] {
            assert_mcp_receipts_refused(
                &gateway, &pending, native_bearer, headers, "HTTP/1.1 403 Forbidden", None,
            );
        }
        assert_mcp_receipts_refused(
            &gateway, &pending, native_bearer, "Origin: https://fixture.invalid\r\n",
            "HTTP/1.1 403 Forbidden", Some("browser-origin gateway requests are not allowed"),
        );
        if configured_token.is_some() {
            for bearer in [None, Some("incorrect-native-secret")] {
                assert_mcp_receipts_refused(
                    &gateway, &pending, bearer, "", "HTTP/1.1 401 Unauthorized", Some("unauthorized"),
                );
            }
        } else {
            assert_mcp_receipts_refused(
                &gateway, &pending, native_bearer, "Host: fixture.invalid\r\n",
                "HTTP/1.1 403 Forbidden", Some("untrusted gateway host"),
            );
        }
        pending.complete(&gateway, native_bearer);
    }
}

fn open_mcp_event_stream(
    port: u16,
    bearer: Option<&str>,
    authority_headers: &str,
    target: &str,
    slim: bool,
    gzip: bool,
) -> Box<dyn BufRead> {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect MCP SSE");
    stream.set_read_timeout(Some(Duration::from_secs(5))).expect("MCP SSE read bound");
    let authorization = bearer.map(|token| format!("Authorization: Bearer {token}\r\n"))
        .unwrap_or_default();
    let slim_header = if slim { "x-sand-slim-avatars: 1\r\n" } else { "" };
    let gzip_header = if gzip { "Accept-Encoding: gzip\r\n" } else { "" };
    write!(
        stream,
        "GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\n{authorization}{authority_headers}{slim_header}{gzip_header}Connection: keep-alive\r\n\r\n"
    ).expect("write MCP SSE request");
    stream.flush().expect("flush MCP SSE request");
    let mut reader = BufReader::new(stream);
    let mut headers = String::new();
    loop {
        let mut line = String::new();
        assert!(reader.read_line(&mut line).expect("read MCP SSE headers") > 0,
            "MCP SSE closed before its headers");
        let complete = line == "\r\n";
        headers.push_str(&line);
        assert!(headers.len() < 8192, "unexpected MCP SSE headers");
        if complete {
            break;
        }
    }
    assert!(headers.starts_with("HTTP/1.1 200 OK"), "{headers}");
    assert!(headers.to_ascii_lowercase().contains("content-type: text/event-stream"), "{headers}");
    assert_eq!(headers.to_ascii_lowercase().contains("content-encoding: gzip"), gzip, "{headers}");
    // Production subscribes before flushing these headers. Preserve any body
    // already buffered and decode the actual ongoing stream, including gzip.
    if gzip {
        Box::new(BufReader::new(GzDecoder::new(reader)))
    } else {
        Box::new(reader)
    }
}

fn read_mcp_events_through_marker(reader: &mut dyn BufRead, marker: &str) -> Vec<Value> {
    let mut events = Vec::new();
    let mut saw_retry = false;
    let mut bytes = 0;
    loop {
        let mut line = String::new();
        let count = reader.read_line(&mut line).expect("read MCP SSE through ordinary marker");
        assert!(count > 0, "MCP SSE ended before the ordinary marker");
        bytes += count;
        assert!(bytes < 64 * 1024, "MCP SSE never reached its bounded marker");
        if line.trim() == "retry: 1000" {
            saw_retry = true;
        }
        if let Some(data) = line.strip_prefix("data: ") {
            let event: Value = serde_json::from_str(data.trim_end()).expect("actual MCP SSE JSON");
            let reached_marker = event["payload"]["marker"].as_str() == Some(marker);
            events.push(event);
            if reached_marker {
                assert!(saw_retry, "must read the subscribed stream through the later marker");
                return events;
            }
        }
    }
}

#[test]
fn internal_mcp_events_are_native_only_before_channel_slim_and_gzip_projection() {
    let selected = format!(
        "/events?channels={HOST_MCP_LIFECYCLE_REQUEST_EVENT_CHANNEL},{RUNNER_TOOL_REQUEST_EVENT_CHANNEL},agent-upserted"
    );
    for (configured_token, native_bearer) in [
        (None, None),
        (Some("native-mcp-event-secret"), Some("native-mcp-event-secret")),
    ] {
        for (target, slim, gzip) in [
            ("/events", false, false),
            (selected.as_str(), false, false),
            ("/events", true, false),
            (selected.as_str(), true, true),
        ] {
            let gateway = McpControlGateway::new(configured_token);
            let before = gateway.events.subscriber_count();
            let headers = format!(
                "{}{}",
                if slim { "x-sand-slim-avatars: 1\r\n" } else { "" },
                if gzip { "Accept-Encoding: gzip\r\n" } else { "" },
            );
            let refused = request(gateway.server.port(), &format!(
                "GET {target} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer unrecognized-local-bearer\r\n{headers}Connection: close\r\n\r\n",
            ));
            assert!(refused.starts_with("HTTP/1.1 401 Unauthorized"), "{refused}");
            assert_eq!(json_body(&refused)["error"], "unauthorized");
            assert!(!refused.contains("retry:") && !refused.contains("data:"), "{refused}");
            assert_eq!(gateway.events.subscriber_count(), before,
                "an invalid bearer must be refused before subscribing");
            assert!(gateway.api.recorded_calls().is_empty());
            let mut streams = vec![(
                "native".to_string(),
                true,
                open_mcp_event_stream(
                    gateway.server.port(), native_bearer,
                    "x-sand-role: operator\r\nx-sand-scope: operator.write\r\n", target, slim, gzip,
                ),
            )];
            for (token, authorization) in &gateway.devices {
                let headers = format!(
                    "x-sand-role: {}\r\nx-sand-scope: operator.write\r\n", authorization.role,
                );
                streams.push((
                    authorization.device_id.clone(),
                    false,
                    open_mcp_event_stream(gateway.server.port(), Some(token), &headers, target, slim, gzip),
                ));
            }
            for (label, headers) in [
                ("native-read-scope", "x-sand-scope: operator.read\r\n"),
                ("native-node-role", "x-sand-role: node\r\n"),
            ] {
                streams.push((
                    label.to_string(),
                    false,
                    open_mcp_event_stream(gateway.server.port(), native_bearer, headers, target, slim, gzip),
                ));
            }

            // Every HTTP subscriber is established before either real relay
            // publishes. Seeing the later marker proves progress past both IDs.
            let pending = PendingMcpControlRequests::start(&gateway);
            let marker = "mcp-control-stream-progress";
            let marker_event = json!({
                "channel": "agent-upserted",
                "payload": {
                    "marker": marker,
                    "agent": {"id": "fixture-agent", "avatarDataUrl": "data:image/png;base64,Zml4dHVyZQ=="},
                },
            });
            gateway.events.publish(marker_event.clone());
            let mut projected_marker = marker_event;
            if slim {
                projected_marker["payload"]["agent"]["avatarDataUrl"] = Value::Null;
            }
            for (label, native, mut reader) in streams {
                let received = read_mcp_events_through_marker(reader.as_mut(), marker);
                let expected = if native {
                    vec![pending.events[0].clone(), pending.events[1].clone(), projected_marker.clone()]
                } else {
                    vec![projected_marker.clone()]
                };
                assert_eq!(received, expected, "{label}: target={target}, slim={slim}, gzip={gzip}");
                if !native {
                    let encoded = serde_json::to_string(&received).expect("observed SSE frames");
                    for event in &pending.events {
                        assert!(!encoded.contains(event["payload"]["requestId"].as_str().expect("real id")),
                            "{label} received an internal request ID");
                    }
                }
            }
            assert!(gateway.api.recorded_calls().is_empty(), "SSE must not settle either waiter");
            pending.assert_pending(&gateway);
            pending.complete(&gateway, native_bearer);
        }
    }
}

#[test]
fn gateway_admits_only_explicit_internal_mcp_lifecycle_settlement_method() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");

    let method = "mcp.resolveLifecycleRequest";
    let response = request(
        server.port(),
        &format!(
            "POST /api/{method} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(
        response.starts_with("HTTP/1.1 200 OK"),
        "{method} was not admitted through the explicit Host MCP lifecycle settlement boundary: {response}"
    );
    assert_eq!(json_body(&response)["method"], method);

    let denied = request(
        server.port(),
        "POST /api/mcp.anythingElse HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(
        denied.starts_with("HTTP/1.1 404 Not Found"),
        "the mcp namespace must stay fail-closed: {denied}"
    );
    assert_eq!(
        json_body(&denied)["error"],
        "unknown gateway method: mcp.anythingElse"
    );

    server.close();
}

#[test]
fn gateway_admits_only_explicit_internal_runner_methods() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");

    for method in [
        "runner.acceptRoutedPrompt",
        "runner.startRoutedProvider",
        "runner.cancelRoutedProvider",
        "runner.resolveRoutedToolRequest",
    ] {
        let response = request(
            server.port(),
            &format!(
                "POST /api/{method} HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
            ),
        );
        assert!(
            response.starts_with("HTTP/1.1 200 OK"),
            "{method} was not admitted through the explicit internal Runner boundary: {response}"
        );
        assert_eq!(json_body(&response)["method"], method);
    }

    let denied = request(
        server.port(),
        "POST /api/runner.anythingElse HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(
        denied.starts_with("HTTP/1.1 404 Not Found"),
        "the runner namespace must stay fail-closed: {denied}"
    );
    assert_eq!(
        json_body(&denied)["error"],
        "unknown gateway method: runner.anythingElse"
    );

    server.close();
}

#[test]
fn gateway_rejects_unknown_commands_before_host_dispatch() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let response = request(
        server.port(),
        "POST /api/notARealGrokMethod HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(response.starts_with("HTTP/1.1 404 Not Found"), "{response}");
    assert_eq!(json_body(&response)["error"], "unknown gateway method: notARealGrokMethod");
    server.close();
}

#[test]
fn gateway_slim_avatar_projection_matches_grok_contract() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let response = request(
        server.port(),
        "POST /api/listAgents HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-slim-avatars: 1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let body = json_body(&response);
    assert_eq!(body[0]["id"], "agent-1");
    assert_eq!(body[0]["avatarDataUrl"], Value::Null);
    server.close();
}

#[test]
fn gateway_avatar_endpoint_serves_versioned_bytes_and_security_headers() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let response = request(
        server.port(),
        "GET /avatars/agent-1?v=avatar-v1 HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(response.starts_with("HTTP/1.1 200 OK"), "{response}");
    let lower = response.to_ascii_lowercase();
    assert!(lower.contains("content-type: image/png"), "{response}");
    assert!(lower.contains("content-disposition: attachment"), "{response}");
    assert!(lower.contains("x-content-type-options: nosniff"), "{response}");
    assert!(lower.contains("cache-control: private, max-age=31536000, immutable"), "{response}");
    assert!(response.contains("ETag: \"avatar-v1\""), "{response}");
    assert!(response.ends_with("hello"), "{response}");

    let not_modified = request(
        server.port(),
        "GET /avatars/agent-1?v=avatar-v1 HTTP/1.1\r\nHost: 127.0.0.1\r\nIf-None-Match: \"avatar-v1\"\r\nConnection: close\r\n\r\n",
    );
    assert!(not_modified.starts_with("HTTP/1.1 304 Not Modified"), "{not_modified}");

    let cross_site = request(
        server.port(),
        "GET /avatars/agent-1 HTTP/1.1\r\nHost: 127.0.0.1\r\nSec-Fetch-Site: cross-site\r\nConnection: close\r\n\r\n",
    );
    assert!(cross_site.starts_with("HTTP/1.1 403 Forbidden"), "{cross_site}");
    server.close();
}

#[test]
fn gateway_bridge_channels_require_auth_and_round_trip_frames() {
    let unauthenticated_local = GatewayBridgeHub::default();
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: Some(unauthenticated_local),
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let denied = request(
        server.port(),
        "GET /local-exec/requests HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n",
    );
    assert!(denied.starts_with("HTTP/1.1 401 Unauthorized"), "{denied}");
    server.close();

    let local_exec = GatewayBridgeHub::default();
    let webauthn = GatewayBridgeHub::default();
    let local_responses = local_exec.subscribe_responses();
    let webauthn_responses = webauthn.subscribe_responses();
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: Some(local_exec.clone()),
        webauthn: Some(webauthn.clone()),
        config: config(Some("secret")),
        started_at: 1,
    })
    .expect("authenticated gateway server");
    let port = server.port();

    let local_client = thread::spawn(move || {
        let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect local-exec SSE");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("local-exec read timeout");
        stream
            .write_all(
                b"GET /local-exec/requests HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer secret\r\nConnection: keep-alive\r\n\r\n",
            )
            .expect("write local-exec SSE request");
        stream.flush().expect("flush local-exec request");
        let mut bytes = Vec::new();
        let mut chunk = [0_u8; 1024];
        loop {
            let count = stream.read(&mut chunk).expect("read local-exec SSE");
            assert!(count > 0, "local-exec SSE ended before request frame");
            bytes.extend_from_slice(&chunk[..count]);
            let text = String::from_utf8_lossy(&bytes);
            if text.contains("\"kind\":\"execute\"") {
                return text.into_owned();
            }
        }
    });
    for _ in 0..100 {
        if local_exec.request_subscriber_count() > 0 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(local_exec.request_subscriber_count(), 1);
    local_exec.publish_request(json!({"kind":"execute","requestId":"local-1"}));
    let local_stream = local_client.join().expect("local-exec client");
    assert!(local_stream.contains("retry: 1000"), "{local_stream}");

    let local_body = r#"{"kind":"result","requestId":"local-1","value":7}"#;
    let local_post = request(
        port,
        &format!(
            "POST /local-exec/responses HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer secret\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{local_body}",
            local_body.len()
        ),
    );
    assert!(local_post.starts_with("HTTP/1.1 200 OK"), "{local_post}");
    assert_eq!(
        local_responses.recv_timeout(Duration::from_secs(1)).expect("local response"),
        json!({"kind":"result","requestId":"local-1","value":7})
    );

    let web_body = r#"{"kind":"hello","computerId":"computer-1"}"#;
    let web_post = request(
        port,
        &format!(
            "POST /webauthn/responses HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer secret\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{web_body}",
            web_body.len()
        ),
    );
    assert!(web_post.starts_with("HTTP/1.1 200 OK"), "{web_post}");
    assert_eq!(
        webauthn_responses.recv_timeout(Duration::from_secs(1)).expect("webauthn response"),
        json!({"kind":"hello","computerId":"computer-1"})
    );

    server.close();
}

#[test]
fn gateway_events_stream_retries_filters_and_delivers_runtime_events() {
    let events = GatewayEventHub::default();
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: events.clone(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let port = server.port();

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect SSE");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout");
    stream
        .write_all(
            b"GET /events?channels=runtime HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: keep-alive\r\n\r\n",
        )
        .expect("write SSE request");
    stream.flush().expect("flush SSE request");

    for _ in 0..100 {
        if events.subscriber_count() > 0 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(events.subscriber_count(), 1, "SSE subscriber registered");

    events.publish(json!({
        "channel": "ignored",
        "payload": { "value": 0 }
    }));
    events.publish(json!({
        "channel": "runtime",
        "payload": { "value": 7 }
    }));

    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    loop {
        let count = stream.read(&mut chunk).expect("read SSE frame");
        assert!(count > 0, "SSE stream closed before event");
        bytes.extend_from_slice(&chunk[..count]);
        let text = String::from_utf8_lossy(&bytes);
        if text.contains("\"value\":7") {
            assert!(text.contains("HTTP/1.1 200 OK"), "{text}");
            assert!(text.contains("retry: 1000"), "{text}");
            assert!(!text.contains("\"value\":0"), "{text}");
            break;
        }
        assert!(bytes.len() < 32 * 1024, "unexpectedly large SSE response");
    }

    drop(stream);
    server.close();
}


#[test]
fn gateway_gzips_only_large_successful_command_json_when_requested() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let port = server.port();

    let payload = "x".repeat(2_000);
    let body = serde_json::to_string(&json!({ "payload": payload })).expect("serialize large request");
    let response = request_bytes(
        port,
        &format!(
            "POST /api/getTranscript HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept-Encoding: br, gzip\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    let header_end = response
        .windows(4)
        .position(|window| window == b"\r\n\r\n")
        .map(|index| index + 4)
        .expect("gzip response header");
    let headers = String::from_utf8(response[..header_end].to_vec()).expect("gzip response headers utf8");
    let lower = headers.to_ascii_lowercase();
    assert!(headers.starts_with("HTTP/1.1 200 OK"), "{headers}");
    assert!(lower.contains("content-encoding: gzip"), "{headers}");
    assert!(lower.contains("vary: accept-encoding"), "{headers}");
    assert!(lower.contains("x-sand-mint-dedupe: 1"), "{headers}");

    let mut decoder = GzDecoder::new(&response[header_end..]);
    let mut decoded = String::new();
    decoder.read_to_string(&mut decoded).expect("decode gateway gzip");
    assert_eq!(serde_json::from_str::<Value>(&decoded).expect("decoded JSON"), serde_json::from_str::<Value>(&body).unwrap());

    let small = request(
        port,
        "POST /api/getTranscript HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept-Encoding: gzip\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(
        !small.to_ascii_lowercase().contains("content-encoding: gzip"),
        "small command responses must stay uncompressed: {small}"
    );

    server.close();
}


#[test]
fn gateway_events_negotiate_gzip_for_sse_streams() {
    let events = GatewayEventHub::default();
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: events.clone(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let port = server.port();

    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect gzip SSE");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("gzip SSE read timeout");
    stream
        .write_all(
            b"GET /events?channels=runtime HTTP/1.1\r\nHost: 127.0.0.1\r\nAccept-Encoding: gzip\r\nConnection: keep-alive\r\n\r\n",
        )
        .expect("write gzip SSE request");
    stream.flush().expect("flush gzip SSE request");

    for _ in 0..100 {
        if events.subscriber_count() > 0 {
            break;
        }
        thread::sleep(Duration::from_millis(10));
    }
    assert_eq!(events.subscriber_count(), 1, "gzip SSE subscriber registered");
    events.publish(json!({
        "channel": "runtime",
        "payload": { "value": 9 }
    }));

    let mut bytes = Vec::new();
    let mut chunk = [0_u8; 1024];
    let header_end = loop {
        let count = stream.read(&mut chunk).expect("read gzip SSE frame");
        assert!(count > 0, "gzip SSE closed before compressed body");
        bytes.extend_from_slice(&chunk[..count]);
        if let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") {
            let end = index + 4;
            if bytes.len() >= end + 10 {
                break end;
            }
        }
        assert!(bytes.len() < 32 * 1024, "unexpectedly large gzip SSE response");
    };
    let headers = String::from_utf8(bytes[..header_end].to_vec()).expect("gzip SSE headers utf8");
    let lower = headers.to_ascii_lowercase();
    assert!(headers.starts_with("HTTP/1.1 200 OK"), "{headers}");
    assert!(lower.contains("content-encoding: gzip"), "{headers}");
    assert!(lower.contains("vary: accept-encoding"), "{headers}");
    assert_eq!(&bytes[header_end..header_end + 2], &[0x1f, 0x8b]);

    drop(stream);
    server.close();
}


#[test]
fn gateway_command_telemetry_preserves_request_trace_and_server_error_semantics() {
    let api = ReportingApi::default();
    let reports = Arc::clone(&api.reports);
    let contexts = Arc::clone(&api.contexts);
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(api),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 1,
    })
    .expect("gateway server");
    let port = server.port();
    let valid_traceparent =
        "00-4bf92f3577b34da6a3ce929d0e0e4736-00f067aa0ba902b7-01";

    let success = request(
        port,
        &format!(
            "POST /api/getTranscript HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-request-id: request-ok\r\ntraceparent: {valid_traceparent}\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(success.starts_with("HTTP/1.1 200 OK"), "{success}");

    let failed_body = r#"{"fail":true}"#;
    let failed = request(
        port,
        &format!(
            "POST /api/getTranscript HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-request-id: request-fail\r\ntraceparent: invalid\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{failed_body}",
            failed_body.len()
        ),
    );
    assert!(failed.starts_with("HTTP/1.1 500 Internal Server Error"), "{failed}");

    let contexts = contexts.lock().expect("read contexts");
    assert_eq!(contexts.len(), 2);
    assert_eq!(contexts[0].0.as_deref(), Some("request-ok"));
    assert_eq!(contexts[0].1.as_deref(), Some(valid_traceparent));
    assert_eq!(
        contexts[0].2.as_deref(),
        Some("4bf92f3577b34da6a3ce929d0e0e4736")
    );
    assert_eq!(contexts[0].3.as_deref(), Some("00f067aa0ba902b7"));
    assert!(contexts[0].4);
    assert_eq!(contexts[1].0.as_deref(), Some("request-fail"));
    assert_eq!(contexts[1].1, None);
    assert_eq!(contexts[1].2, None);
    assert_eq!(contexts[1].3, None);
    assert!(contexts[1].4);
    drop(contexts);

    let reports = reports.lock().expect("read reports");
    assert_eq!(reports.len(), 2);
    assert_eq!(reports[0].0, "complete");
    assert_eq!(reports[0].1.method, "getTranscript");
    assert_eq!(reports[0].1.request_id.as_deref(), Some("request-ok"));
    assert_eq!(reports[0].1.traceparent.as_deref(), Some(valid_traceparent));
    assert_eq!(reports[0].1.status, 200);
    assert_eq!(reports[0].1.error, None);

    assert_eq!(reports[1].0, "error");
    assert_eq!(reports[1].1.method, "getTranscript");
    assert_eq!(reports[1].1.request_id.as_deref(), Some("request-fail"));
    assert_eq!(reports[1].1.traceparent, None);
    assert_eq!(reports[1].1.status, 500);
    assert_eq!(
        reports[1].1.error.as_deref(),
        Some("synthetic internal failure")
    );
    assert_eq!(reports[1].1.reason.as_deref(), Some("application"));
    assert_eq!(reports[1].1.error_class.as_deref(), Some("GatewayCommandError"));
    assert_eq!(reports[1].1.errno, None);

    server.close();
}

#[test]
fn openclaw_gateway_device_pairing_handshake_allows_unauthenticated() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(Some("configured-secret-token")),
        started_at: 100,
    })
    .expect("gateway server");
    let port = server.port();

    let pair_req = request(
        port,
        "POST /api/device.pair.request HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(pair_req.starts_with("HTTP/1.1 200 OK"), "{pair_req}");

    let pair_status = request(
        port,
        "POST /api/device.pair.status HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(pair_status.starts_with("HTTP/1.1 200 OK"), "{pair_status}");

    let pair_list_unauth = request(
        port,
        "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(pair_list_unauth.starts_with("HTTP/1.1 401 Unauthorized"), "{pair_list_unauth}");

    let pair_list_auth = request(
        port,
        "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer configured-secret-token\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(pair_list_auth.starts_with("HTTP/1.1 200 OK"), "{pair_list_auth}");

    server.close();
}

#[test]
fn openclaw_gateway_device_timestamp_skew_and_nonce_require_authentication() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(Some("configured-secret-token")),
        started_at: 100,
    })
    .expect("gateway server");
    let port = server.port();

    let skew_req = request(
        port,
        "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-device-timestamp: 1000\r\nx-sand-device-nonce: nonce-skew-test\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(skew_req.starts_with("HTTP/1.1 401 Unauthorized"), "{skew_req}");
    assert!(skew_req.contains("TIMESTAMP_SKEW"), "{skew_req}");

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let valid_req = request(
        port,
        &format!(
            "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-device-timestamp: {now}\r\nx-sand-device-nonce: fresh-nonce-999\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(valid_req.starts_with("HTTP/1.1 401 Unauthorized"), "{valid_req}");
    assert!(valid_req.contains("unauthorized"), "{valid_req}");

    let authenticated_req = request(
        port,
        &format!(
            "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer configured-secret-token\r\nx-sand-device-timestamp: {now}\r\nx-sand-device-nonce: fresh-nonce-authenticated\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(authenticated_req.starts_with("HTTP/1.1 200 OK"), "{authenticated_req}");

    let replay_req = request(
        port,
        &format!(
            "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer configured-secret-token\r\nx-sand-device-timestamp: {now}\r\nx-sand-device-nonce: fresh-nonce-authenticated\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(replay_req.starts_with("HTTP/1.1 401 Unauthorized"), "{replay_req}");
    assert!(replay_req.contains("NONCE_REPLAY"), "{replay_req}");

    let stale_req = request(
        port,
        &format!(
            "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-device-timestamp: {now}\r\nx-sand-device-nonce: nonce-stale-rev\r\nx-sand-pairing-revision: stale-1\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
        ),
    );
    assert!(stale_req.starts_with("HTTP/1.1 401 Unauthorized"), "{stale_req}");
    assert!(stale_req.contains("PAIRING_CHANGED"), "{stale_req}");

    server.close();
}

#[test]
fn openclaw_gateway_role_and_scope_enforcement() {
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: GatewayEventHub::default(),
        local_exec: None,
        webauthn: None,
        config: config(None),
        started_at: 100,
    })
    .expect("gateway server");
    let port = server.port();

    let role_mismatch = request(
        port,
        "POST /api/device.pair.revoke HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-role: node\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(role_mismatch.starts_with("HTTP/1.1 403 Forbidden"), "{role_mismatch}");
    assert!(role_mismatch.contains("ROLE_MISMATCH"), "{role_mismatch}");

    let scope_insufficient = request(
        port,
        "POST /api/device.pair.revoke HTTP/1.1\r\nHost: 127.0.0.1\r\nx-sand-scope: operator.read\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(scope_insufficient.starts_with("HTTP/1.1 403 Forbidden"), "{scope_insufficient}");
    assert!(scope_insufficient.contains("INSUFFICIENT_SCOPE"), "{scope_insufficient}");

    let origin_req = request(
        port,
        "POST /api/device.pair.list HTTP/1.1\r\nHost: 127.0.0.1\r\nOrigin: http://evil.com\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{}",
    );
    assert!(origin_req.starts_with("HTTP/1.1 403 Forbidden"), "{origin_req}");

    server.close();
}

struct SessionModelNativeContextApi {
    pairing: DevicePairingOwner,
}

impl GatewayApi for SessionModelNativeContextApi {
    fn call(&self, _method: &str, _args: Value) -> Result<Value, GatewayCommandError> {
        Ok(json!({}))
    }
    fn authenticate_device_token(
        &self,
        token: &str,
    ) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.pairing.authenticate_device_token(token)
    }
    fn call_with_context(
        &self,
        method: &str,
        _args: Value,
        context: &GatewayCommandContext,
    ) -> Result<Value, GatewayCommandError> {
        Ok(json!({
            "method": method,
            "authenticated": context.authenticated,
            "native": context.native_operator_control,
        }))
    }
}

#[test]
fn gateway_session_model_native_authority_is_not_promoted_by_paired_operator_write() {
    let pairing = DevicePairingOwner::in_memory().unwrap();
    let pending = pairing.handle_pair_request(&json!({
        "deviceId": "model-operator",
        "publicKey": "model-operator-public-key",
        "platform": "macos",
        "roles": ["operator"],
        "scopes": ["operator.read", "operator.write"],
    })).unwrap();
    pairing.handle_pair_approve(&json!({"requestId": pending["requestId"]})).unwrap();
    let rotated = pairing.handle_pair_rotate_token(&json!({
        "deviceId": "model-operator", "role": "operator",
    })).unwrap();
    let paired = rotated["token"].as_str().unwrap();
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(SessionModelNativeContextApi { pairing }),
        events: GatewayEventHub::default(), local_exec: None, webauthn: None,
        config: config(Some("native-secret")), started_at: 1,
    }).unwrap();
    let port = server.port();
    let invoke = |token: &str, extra: &str, method: &str| request(port, &format!(
        "POST /api/{method} HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer {token}\r\n{extra}Content-Type: application/json\r\nContent-Length: 2\r\nConnection: close\r\n\r\n{{}}"
    ));
    let native = invoke("native-secret", "", "sessions.get");
    assert!(native.starts_with("HTTP/1.1 200 OK"), "{native}");
    assert_eq!(json_body(&native)["native"], true);
    let paired_read = invoke(paired, "", "sessions.get");
    assert!(paired_read.starts_with("HTTP/1.1 200 OK"), "{paired_read}");
    assert_eq!(json_body(&paired_read)["authenticated"], true);
    assert_eq!(json_body(&paired_read)["native"], false);
    let paired_write = invoke(paired, "x-sand-scope: operator.write\r\n", "sessions.patch");
    assert!(paired_write.starts_with("HTTP/1.1 200 OK"), "{paired_write}");
    assert_eq!(json_body(&paired_write)["native"], false);
    let narrowed = invoke("native-secret", "x-sand-scope: operator.read\r\n", "sessions.get");
    assert!(narrowed.starts_with("HTTP/1.1 200 OK"), "{narrowed}");
    assert_eq!(json_body(&narrowed)["native"], false);
    let rejected_write = invoke("native-secret", "x-sand-scope: operator.read\r\n", "sessions.patch");
    assert!(rejected_write.starts_with("HTTP/1.1 403 Forbidden"), "{rejected_write}");
    let unknown = invoke("not-issued", "", "sessions.patch");
    assert!(unknown.starts_with("HTTP/1.1 401 Unauthorized"), "{unknown}");
    server.close();
}
