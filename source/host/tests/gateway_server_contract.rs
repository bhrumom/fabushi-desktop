use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandError, GatewayEventHub, GatewayHealth, GatewayServerDeps,
    start_gateway_server,
};
use serde_json::{Value, json};

#[derive(Debug)]
struct TestApi;

impl GatewayApi for TestApi {
    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        match method {
            "echo" => Ok(args),
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
    let echo = request(
        port,
        &format!(
            "POST /api/echo HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        ),
    );
    assert!(echo.starts_with("HTTP/1.1 200 OK"), "{echo}");
    assert!(
        echo.to_ascii_lowercase()
            .contains("x-sand-mint-dedupe: 1"),
        "{echo}"
    );
    assert_eq!(json_body(&echo), json!({ "value": "hello" }));

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
        config: config(Some("secret")),
        started_at: 1,
    })
    .expect("gateway server");
    let port = server.port();

    let missing = request(
        port,
        "POST /api/echo HTTP/1.1\r\nHost: 127.0.0.1\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(missing.starts_with("HTTP/1.1 401 Unauthorized"), "{missing}");

    let authorized = request(
        port,
        "POST /api/echo HTTP/1.1\r\nHost: 127.0.0.1\r\nAuthorization: Bearer secret\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
    );
    assert!(
        authorized.starts_with("HTTP/1.1 200 OK"),
        "{authorized}"
    );
    assert_eq!(json_body(&authorized), json!({}));

    server.close();
}

#[test]
fn gateway_events_stream_retries_filters_and_delivers_runtime_events() {
    let events = GatewayEventHub::default();
    let server = start_gateway_server(GatewayServerDeps {
        api: Arc::new(TestApi),
        events: events.clone(),
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
