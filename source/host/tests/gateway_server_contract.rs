use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use flate2::read::GzDecoder;
use mahayana_host_runtime::gateway_config::{GatewayServerConfig, GatewayTlsConfig};
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayBridgeHub, GatewayCommandError, GatewayCommandReport, GatewayEventHub, GatewayHealth,
    GatewayServerDeps, start_gateway_server,
};
use rcgen::generate_simple_self_signed;
use rustls::pki_types::ServerName;
use rustls::{ClientConfig, ClientConnection, RootCertStore, StreamOwned};
use serde_json::{Value, json};

#[derive(Debug)]
struct TestApi;

impl GatewayApi for TestApi {
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
            ) => Ok(json!({
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


#[derive(Clone, Default)]
struct ReportingApi {
    reports: Arc<Mutex<Vec<(String, GatewayCommandReport)>>>,
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
