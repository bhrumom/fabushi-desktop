use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayBridgeHub, GatewayCommandError, GatewayEventHub, GatewayHealth,
    GatewayServerDeps, start_gateway_server,
};
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

fn config(token: Option<&str>) -> GatewayServerConfig {
    GatewayServerConfig {
        host: "127.0.0.1".into(),
        port: None,
        auth_token: token.map(str::to_string),
        tls: None,
    }
}

fn request(port: u16, request: &str) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect gateway");
    stream
        .set_read_timeout(Some(Duration::from_secs(2)))
        .expect("read timeout");
    stream.write_all(request.as_bytes()).expect("write request");
    stream.flush().expect("flush request");
    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .expect("read gateway response");
    response
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
