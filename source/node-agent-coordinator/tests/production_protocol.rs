#[cfg(unix)]
mod unix {
    use mahayana_node_agent_coordinator::carrier::{
        CarrierChannel, CarrierEnvelope,
    };
    use mahayana_node_agent_coordinator::protocol::{
        CoordinatorFrame, Failure, LifecyclePhase, ReplyOutcome, COORDINATOR_DISCONNECTED,
        COORDINATOR_PROTOCOL_VERSION,
    };
    use serde_json::{Value, json};
    use std::fs;
    use std::io::{self, BufRead, BufReader, Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::net::{TcpListener, TcpStream};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicUsize, Ordering},
        mpsc,
    };
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn send(stdin: &mut impl Write, channel: CarrierChannel, frame: &CoordinatorFrame) {
        let envelope = CarrierEnvelope::new(
            channel,
            serde_json::to_value(frame).expect("serialize coordinator frame"),
        );
        serde_json::to_writer(&mut *stdin, &envelope).expect("serialize carrier envelope");
        writeln!(stdin).expect("write carrier envelope");
        stdin.flush().expect("flush carrier envelope");
    }

    #[derive(Debug, Clone)]
    struct GatewayApiRequest {
        http_method: String,
        path: String,
        body: Option<Vec<u8>>,
    }

    struct FakeGateway {
        address: std::net::SocketAddr,
        token: String,
        oauth_callback_port: u16,
        oauth_completion: Arc<Mutex<Option<Value>>>,
        oauth_completion_attempts: Arc<AtomicUsize>,
        webauthn_batches: Arc<Mutex<Vec<Value>>>,
        api_requests: Arc<Mutex<Vec<GatewayApiRequest>>>,
        stop: Arc<AtomicBool>,
        worker: Option<thread::JoinHandle<()>>,
    }

    impl FakeGateway {
        fn start(data_dir: &Path) -> Self {
            let listener = TcpListener::bind("127.0.0.1:0").expect("bind fake Host gateway");
            listener
                .set_nonblocking(true)
                .expect("set fake Host gateway nonblocking");
            let address = listener.local_addr().expect("fake Host gateway address");
            let token = "production-protocol-token".to_string();
            let oauth_callback_probe =
                TcpListener::bind("127.0.0.1:0").expect("reserve OAuth callback port");
            let oauth_callback_port = oauth_callback_probe
                .local_addr()
                .expect("OAuth callback address")
                .port();
            drop(oauth_callback_probe);
            let oauth_completion = Arc::new(Mutex::new(None));
            let oauth_completion_attempts = Arc::new(AtomicUsize::new(0));
            let webauthn_batches = Arc::new(Mutex::new(Vec::new()));
            let api_requests = Arc::new(Mutex::new(Vec::new()));
            let crash_path = data_dir.join("fake-host-crash");
            let stop = Arc::new(AtomicBool::new(false));
            let worker_stop = Arc::clone(&stop);
            let worker_token = token.clone();
            let worker_oauth_completion = Arc::clone(&oauth_completion);
            let worker_oauth_completion_attempts = Arc::clone(&oauth_completion_attempts);
            let worker_webauthn_batches = Arc::clone(&webauthn_batches);
            let worker_api_requests = Arc::clone(&api_requests);
            let worker = thread::spawn(move || {
                let mut handlers = Vec::new();
                loop {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let token = worker_token.clone();
                            let crash_path = crash_path.clone();
                            let handler_stop = Arc::clone(&worker_stop);
                            let oauth_completion = Arc::clone(&worker_oauth_completion);
                            let oauth_completion_attempts =
                                Arc::clone(&worker_oauth_completion_attempts);
                            let webauthn_batches = Arc::clone(&worker_webauthn_batches);
                            let api_requests = Arc::clone(&worker_api_requests);
                            handlers.push(thread::spawn(move || {
                                let _ = serve_fake_gateway(
                                    stream,
                                    &token,
                                    &crash_path,
                                    handler_stop.as_ref(),
                                    oauth_callback_port,
                                    oauth_completion.as_ref(),
                                    oauth_completion_attempts.as_ref(),
                                    webauthn_batches.as_ref(),
                                    api_requests.as_ref(),
                                );
                            }));
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
                            if worker_stop.load(Ordering::Acquire) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(5));
                        }
                        Err(_) => {
                            if worker_stop.load(Ordering::Acquire) {
                                break;
                            }
                            thread::sleep(Duration::from_millis(5));
                        }
                    }
                }
                // The test stops its Coordinator before stopping this listener.
                // Drain the accepted backlog and join every owned exchange so
                // the final request log cannot miss a delayed or duplicate call.
                for handler in handlers {
                    handler.join().expect("fake Host gateway handler panicked");
                }
            });
            Self {
                address,
                token,
                oauth_callback_port,
                oauth_completion,
                oauth_completion_attempts,
                webauthn_batches,
                api_requests,
                stop,
                worker: Some(worker),
            }
        }

        fn port(&self) -> u16 {
            self.address.port()
        }

        fn token(&self) -> &str {
            &self.token
        }

        fn oauth_callback_port(&self) -> u16 {
            self.oauth_callback_port
        }

        fn oauth_completion_attempts(&self) -> usize {
            self.oauth_completion_attempts.load(Ordering::Acquire)
        }

        fn api_requests(&self) -> Vec<GatewayApiRequest> {
            self.api_requests.lock().expect("gateway API requests lock").clone()
        }

        fn stop_and_join(&mut self) -> thread::Result<()> {
            let Some(worker) = self.worker.take() else {
                return Ok(());
            };
            self.stop.store(true, Ordering::Release);
            let _ = TcpStream::connect(self.address);
            worker.join()
        }

        fn wait_for_oauth_completion(&self) -> Value {
            for _ in 0..200 {
                if let Some(value) = self
                    .oauth_completion
                    .lock()
                    .expect("OAuth completion lock")
                    .clone()
                {
                    return value;
                }
                thread::sleep(Duration::from_millis(10));
            }
            panic!("shipping Coordinator did not complete MCP OAuth");
        }

        fn webauthn_result(&self) -> Option<Value> {
            self.webauthn_batches
                .lock()
                .expect("WebAuthn batches lock")
                .iter()
                .find_map(|batch| {
                    batch
                        .get("frames")
                        .and_then(Value::as_array)
                        .and_then(|frames| {
                            frames.iter().find_map(|frame| {
                                (frame.get("kind").and_then(Value::as_str) == Some("result"))
                                    .then(|| frame.clone())
                            })
                        })
                })
        }

        fn wait_for_webauthn_result(&self) -> Value {
            for _ in 0..300 {
                if let Some(value) = self.webauthn_result() {
                    return value;
                }
                thread::sleep(Duration::from_millis(10));
            }
            panic!("shipping Coordinator did not deliver the WebAuthn result");
        }
    }

    impl Drop for FakeGateway {
        fn drop(&mut self) {
            let _ = self.stop_and_join();
        }
    }

    fn serve_fake_gateway(
        mut stream: TcpStream,
        expected_token: &str,
        crash_path: &Path,
        stop: &AtomicBool,
        oauth_callback_port: u16,
        oauth_completion: &Mutex<Option<Value>>,
        oauth_completion_attempts: &AtomicUsize,
        webauthn_batches: &Mutex<Vec<Value>>,
        api_requests: &Mutex<Vec<GatewayApiRequest>>,
    ) -> io::Result<()> {
        // The fake gateway listener is non-blocking so its accept loop can poll for
        // shutdown. Accepted sockets may inherit that mode on macOS, making the
        // first read/write in a retry race with the client and fail spuriously.
        // Normalize each accepted HTTP exchange to blocking mode; the explicit
        // timeouts below still bound the test and preserve production semantics.
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(Duration::from_secs(2)))?;
        stream.set_write_timeout(Some(Duration::from_secs(2)))?;

        let mut request = Vec::with_capacity(2048);
        let mut chunk = [0_u8; 2048];
        let header_end = loop {
            let count = stream.read(&mut chunk)?;
            if count == 0 {
                return Ok(());
            }
            request.extend_from_slice(&chunk[..count]);
            if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
                break index + 4;
            }
            if request.len() > 64 * 1024 {
                return write_fake_response(&mut stream, 431, "");
            }
        };

        let headers = String::from_utf8_lossy(&request[..header_end]).into_owned();
        let mut lines = headers.lines();
        let request_line = lines.next().unwrap_or_default();
        let mut parts = request_line.split_whitespace();
        let method = parts.next().unwrap_or_default().to_string();
        let path = parts.next().unwrap_or_default().to_string();
        let api_request_index = if path.starts_with("/api/") {
            let mut requests = api_requests
                .lock()
                .map_err(|_| io::Error::other("gateway API requests lock poisoned"))?;
            let index = requests.len();
            requests.push(GatewayApiRequest {
                http_method: method.clone(),
                path: path.clone(),
                body: None,
            });
            Some(index)
        } else {
            None
        };
        let content_length = headers
            .lines()
            .find_map(|line| {
                let (name, value) = line.split_once(':')?;
                name.eq_ignore_ascii_case("content-length")
                    .then(|| value.trim().parse::<usize>().ok())
                    .flatten()
            })
            .unwrap_or(0);
        let authorized = headers.lines().any(|line| {
            line.eq_ignore_ascii_case(&format!("authorization: Bearer {expected_token}"))
        });
        if !authorized {
            return write_fake_response(&mut stream, 401, "");
        }

        let total = header_end.saturating_add(content_length);
        while request.len() < total {
            let count = stream.read(&mut chunk)?;
            if count == 0 {
                break;
            }
            request.extend_from_slice(&chunk[..count]);
        }
        if request.len() < total {
            return write_fake_response(&mut stream, 400, "");
        }
        if let Some(index) = api_request_index {
            api_requests
                .lock()
                .map_err(|_| io::Error::other("gateway API requests lock poisoned"))?[index]
                .body = Some(request[header_end..total].to_vec());
        }

        match (method.as_str(), path.as_str()) {
            ("GET", "/webauthn/requests") => {
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n"
                )?;
                writeln!(
                    stream,
                    "data: {}\n",
                    json!({
                        "kind": "welcome",
                        "providerId": "provider-production"
                    })
                )?;
                writeln!(
                    stream,
                    "data: {}\n",
                    json!({
                        "kind": "ceremony",
                        "requestId": "webauthn-production-1",
                        "ceremony": {
                            "kind": "get",
                            "origin": "https://example.test",
                            "optionsJson": "{\"rpId\":\"example.test\"}",
                            "challenge": "production"
                        }
                    })
                )?;
                stream.flush()?;
                while !stop.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(())
            }
            ("POST", "/webauthn/responses") => {
                let payload = serde_json::from_slice::<Value>(&request[header_end..total])
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                webauthn_batches
                    .lock()
                    .map_err(|_| io::Error::other("WebAuthn batches lock poisoned"))?
                    .push(payload);
                write_fake_response(&mut stream, 200, r#"{"ok":true}"#)
            }
            ("GET", "/events") => {
                write!(
                    stream,
                    "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n"
                )?;
                writeln!(
                    stream,
                    "data: {}\n",
                    json!({
                        "channel": "runtime",
                        "payload": { "type": "host.test", "value": 1 }
                    })
                )?;
                writeln!(
                    stream,
                    "data: {}\n",
                    json!({
                        "channel": "mcp-oauth-pending",
                        "payload": {
                            "redirectUrl": format!(
                                "http://127.0.0.1:{oauth_callback_port}/oauth/callback"
                            ),
                            "state": "state-live",
                            "serverName": "github"
                        }
                    })
                )?;
                writeln!(
                    stream,
                    "data: {}\n",
                    json!({
                        "channel": "client-side-tool-v2",
                        "payload": {
                            "version": 1,
                            "kind": "call",
                            "accountSlot": "host",
                            "agentId": "agent-tool",
                            "epoch": "epoch-1",
                            "sequence": 1,
                            "message": {
                                "encoding": "protobuf-base64",
                                "messageType": "aiserver.v1.ClientSideToolV2Call",
                                "bytes": "GgZjYWxsLTc="
                            }
                        }
                    })
                )?;
                stream.flush()?;
                while !stop.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(10));
                }
                Ok(())
            }
            ("POST", "/api/echo") => write_fake_response(
                &mut stream,
                200,
                r#"{"echoed":true}"#,
            ),
            ("POST", "/api/sendPrompt") => write_fake_response(
                &mut stream,
                200,
                r#"{"accepted":true}"#,
            ),
            ("POST", "/api/loadBoxMcpServers") => {
                let args: Value = serde_json::from_slice(&request[header_end..total])
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                if args["configJson"] == " {\"mcpServers\":{\"native-error\":{}}}\n" {
                    return write_fake_response(
                        &mut stream,
                        400,
                        r#"{"error":"Box MCP original configuration refusal"}"#,
                    );
                }
                write_fake_response(
                    &mut stream,
                    200,
                    r#"{"serverIdentifiers":["native-box"],"opaque":{"preserved":true}}"#,
                )
            }
            ("POST", "/api/listBoxMcpToolsRaw") => {
                let args: Value = serde_json::from_slice(&request[header_end..total])
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                if args["payloadHex"] == "deadbeef01" {
                    return write_fake_response(
                        &mut stream,
                        400,
                        r#"{"error":"Box MCP original discovery refusal"}"#,
                    );
                }
                write_fake_response(&mut stream, 200, r#"{"payloadHex":"00Aa11Ff"}"#)
            }
            ("POST", "/api/executeBoxMcpToolRaw") => {
                let args: Value = serde_json::from_slice(&request[header_end..total])
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                if args["payloadHex"] == "deadbeef02" {
                    return write_fake_response(
                        &mut stream,
                        503,
                        r#"{"error":"Box MCP original execution outcome unknown"}"#,
                    );
                }
                write_fake_response(&mut stream, 200, r#"{"payloadHex":"00Bb22Ee"}"#)
            }
            ("POST", "/api/listBoxMcpServers") => write_fake_response(
                &mut stream,
                200,
                r#"[{"serverIdentifier":"existing-box","tools":[]}]"#,
            ),
            ("POST", "/api/completeMcpOAuth") => {
                let attempt = oauth_completion_attempts.fetch_add(1, Ordering::AcqRel);
                if attempt == 0 {
                    return write_fake_response(
                        &mut stream,
                        503,
                        r#"{"error":"transient oauth completion failure"}"#,
                    );
                }
                let payload = serde_json::from_slice::<Value>(&request[header_end..total])
                    .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
                *oauth_completion
                    .lock()
                    .map_err(|_| io::Error::other("OAuth completion lock poisoned"))? =
                    Some(payload);
                write_fake_response(&mut stream, 200, r#"{"completed":true}"#)
            }
            ("POST", "/api/crash") => {
                fs::write(crash_path, b"crash")?;
                thread::sleep(Duration::from_millis(750));
                Ok(())
            }
            _ => write_fake_response(&mut stream, 404, ""),
        }
    }

    fn write_fake_response(stream: &mut TcpStream, status: u16, body: &str) -> io::Result<()> {
        let reason = match status {
            200 => "OK",
            400 => "Bad Request",
            401 => "Unauthorized",
            404 => "Not Found",
            431 => "Request Header Fields Too Large",
            503 => "Service Unavailable",
            _ => "Response",
        };
        write!(
            stream,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
        stream.flush()
    }

    fn service_control_frame(
        stdin: &mut impl Write,
        seen_control_methods: &mut Vec<String>,
        frame: CoordinatorFrame,
    ) {
        match frame {
            CoordinatorFrame::Request {
                request_id,
                method,
                args,
            } => {
                seen_control_methods.push(method.clone());
                let outcome = match method.as_str() {
                    "resolveGatewayConnection" => ReplyOutcome::Failed {
                        failure: Failure::new(
                            "SAND_CLIENT_PAUSE",
                            "SAND_CLIENT_PAUSE: fake main keeps LocalExec paused",
                        ),
                    },
                    "mintLocalExecDaemonCredential" => ReplyOutcome::Ok {
                        value: Value::Null,
                    },
                    "requestWebAuthnConsent" => {
                        assert_eq!(args["origin"], "https://example.test");
                        assert_eq!(args["rpId"], "example.test");
                        ReplyOutcome::Ok {
                            value: json!({
                                "approved": true,
                                "promptId": "prompt-production",
                                "windowHandle": 4_294_967_297_u64
                            }),
                        }
                    }
                    "requestWebAuthnPin" => {
                        assert_eq!(args["promptId"], "prompt-production");
                        assert_eq!(args["invalid"], false);
                        ReplyOutcome::Ok {
                            value: json!({ "pin": "2468" }),
                        }
                    }
                    "updateWebAuthnConsent" => ReplyOutcome::Ok {
                        value: Value::Null,
                    },
                    "finishWebAuthnConsent" => ReplyOutcome::Ok {
                        value: Value::Null,
                    },
                    "getRpcTraceWindowTraceparent"
                    | "reportTransportStage"
                    | "reportGatewayCommandSpan" => ReplyOutcome::Ok {
                        value: Value::Null,
                    },
                    _ => ReplyOutcome::Failed {
                        failure: Failure::new(
                            "TEST_UNKNOWN_CONTROL_COMMAND",
                            format!("fake main does not implement {method}"),
                        ),
                    },
                };
                send(
                    stdin,
                    CarrierChannel::Control,
                    &CoordinatorFrame::Reply {
                        request_id,
                        outcome,
                    },
                );
            }
            other => panic!("unexpected post-handshake control frame: {other:?}"),
        }
    }

    fn recv_application_frame(
        rx: &mpsc::Receiver<Result<(CarrierChannel, CoordinatorFrame), serde_json::Error>>,
        stdin: &mut impl Write,
        seen_control_methods: &mut Vec<String>,
        timeout: Duration,
    ) -> (CarrierChannel, CoordinatorFrame) {
        loop {
            let (channel, frame) = rx
                .recv_timeout(timeout)
                .expect("Coordinator frame")
                .expect("valid Coordinator frame");
            if channel != CarrierChannel::Control {
                return (channel, frame);
            }
            service_control_frame(stdin, seen_control_methods, frame);
        }
    }

    fn recv_request_reply(
        rx: &mpsc::Receiver<Result<(CarrierChannel, CoordinatorFrame), serde_json::Error>>,
        stdin: &mut impl Write,
        seen_control_methods: &mut Vec<String>,
        expected_channel: CarrierChannel,
        expected_request_id: &str,
    ) -> ReplyOutcome {
        let deadline = std::time::Instant::now() + Duration::from_secs(3);
        loop {
            let remaining = deadline
                .checked_duration_since(std::time::Instant::now())
                .expect("shipping Coordinator did not settle the requested command");
            let (channel, frame) =
                recv_application_frame(rx, stdin, seen_control_methods, remaining);
            match frame {
                CoordinatorFrame::Reply { request_id, outcome } => {
                    assert_eq!(channel, expected_channel);
                    assert_eq!(request_id, expected_request_id);
                    return outcome;
                }
                CoordinatorFrame::Event { .. } => {}
                other => panic!("unexpected frame while awaiting {expected_request_id}: {other:?}"),
            }
        }
    }

    fn assert_box_mcp_http_requests(
        requests: &[GatewayApiRequest],
        expected: &[(&str, Value)],
    ) {
        let box_requests: Vec<_> = requests
            .iter()
            .filter(|request| request.path.contains("BoxMcp"))
            .collect();
        assert_eq!(
            box_requests.len(),
            expected.len(),
            "unexpected, missing, or repeated Box MCP HTTP request: {box_requests:?}"
        );
        for (request, (method, args)) in box_requests.iter().zip(expected) {
            assert_eq!(request.http_method, "POST");
            assert_eq!(request.path, format!("/api/{method}"));
            let body: Value = serde_json::from_slice(
                request.body.as_deref().expect("complete Box MCP HTTP body"),
            )
            .expect("Box MCP HTTP request JSON");
            assert_eq!(&body, args, "{method} must preserve the original payload");
        }
    }

    fn exercise_box_mcp_native_routing(
        rx: &mpsc::Receiver<Result<(CarrierChannel, CoordinatorFrame), serde_json::Error>>,
        stdin: &mut impl Write,
        seen_control_methods: &mut Vec<String>,
        gateway: &FakeGateway,
    ) -> Vec<(&'static str, Value)> {
        let initial_http_count = gateway.api_requests().len();
        let mut expected_http = Vec::new();
        assert_box_mcp_http_requests(&gateway.api_requests(), &expected_http);

        for (method, success_args, success_value, error_args, error) in [
            (
                "loadBoxMcpServers",
                json!({"configJson": " \n{\"mcpServers\":{\"native-box\":{\"command\":\"fixture\",\"args\":[\"--opaque=é\"]}}}\t"}),
                json!({"serverIdentifiers": ["native-box"], "opaque": {"preserved": true}}),
                json!({"configJson": " {\"mcpServers\":{\"native-error\":{}}}\n"}),
                Failure::new("gateway-command-failed", "Box MCP original configuration refusal"),
            ),
            (
                "listBoxMcpToolsRaw",
                json!({"payloadHex": "00Aa1100"}),
                json!({"payloadHex": "00Aa11Ff"}),
                json!({"payloadHex": "deadbeef01"}),
                Failure::new("gateway-command-failed", "Box MCP original discovery refusal"),
            ),
            (
                "executeBoxMcpToolRaw",
                json!({"payloadHex": "00Bb2200"}),
                json!({"payloadHex": "00Bb22Ee"}),
                json!({"payloadHex": "deadbeef02"}),
                Failure::with_transport_kind(
                    "gateway-unreachable",
                    "Box MCP original execution outcome unknown",
                    "http_5xx",
                ),
            ),
        ] {
            let request_id = format!("r-box-data-{method}");
            send(
                stdin,
                CarrierChannel::Data,
                &CoordinatorFrame::Request {
                    request_id: request_id.clone(),
                    method: method.into(),
                    args: success_args.clone(),
                },
            );
            match recv_request_reply(
                rx,
                stdin,
                seen_control_methods,
                CarrierChannel::Data,
                &request_id,
            ) {
                ReplyOutcome::Failed { failure } => {
                    assert_eq!(failure.code, "MCP_BOX_NATIVE_ONLY", "{method}");
                }
                other => panic!("renderer Box MCP request was not refused: {method}: {other:?}"),
            }
            assert_eq!(
                gateway.api_requests().len(),
                initial_http_count + expected_http.len(),
                "refused renderer command {method} reached the HTTP gateway"
            );

            for (case, args, expected_reply) in [
                ("success", success_args, ReplyOutcome::Ok { value: success_value }),
                ("error", error_args, ReplyOutcome::Failed { failure: error }),
            ] {
                let request_id = format!("r-box-main-{method}-{case}");
                send(
                    stdin,
                    CarrierChannel::MainData,
                    &CoordinatorFrame::Request {
                        request_id: request_id.clone(),
                        method: method.into(),
                        args: args.clone(),
                    },
                );
                assert_eq!(
                    recv_request_reply(
                        rx,
                        stdin,
                        seen_control_methods,
                        CarrierChannel::MainData,
                        &request_id,
                    ),
                    expected_reply,
                    "native {method} must return the original gateway {case}"
                );
                expected_http.push((method, args));
                let observed = gateway.api_requests();
                assert_eq!(observed.len(), initial_http_count + expected_http.len());
                assert_box_mcp_http_requests(&observed, &expected_http);
            }
        }

        // An existing sibling remains usable from Data, and similarly named
        // unknown methods reach the gateway's ordinary 404 rather than being
        // misclassified by a prefix gate. Neither control uses the gate helper.
        for (method, args, expected_reply) in [
            (
                "listBoxMcpServers",
                json!({"serverIdentifiers": ["existing-box"]}),
                ReplyOutcome::Ok {
                    value: json!([{"serverIdentifier": "existing-box", "tools": []}]),
                },
            ),
            (
                "loadBoxMcpServers.extra",
                json!({"configJson": "{}"}),
                ReplyOutcome::Failed {
                    failure: Failure::new(
                        "gateway-command-failed",
                        "gateway loadBoxMcpServers.extra failed with HTTP 404",
                    ),
                },
            ),
            (
                "listBoxMcpToolsRawExtra",
                json!({"payloadHex": "00Cc33"}),
                ReplyOutcome::Failed {
                    failure: Failure::new(
                        "gateway-command-failed",
                        "gateway listBoxMcpToolsRawExtra failed with HTTP 404",
                    ),
                },
            ),
            (
                "executeBoxMcpToolRawExtra",
                json!({"payloadHex": "00Dd44"}),
                ReplyOutcome::Failed {
                    failure: Failure::new(
                        "gateway-command-failed",
                        "gateway executeBoxMcpToolRawExtra failed with HTTP 404",
                    ),
                },
            ),
        ] {
            let request_id = format!("r-box-sibling-{method}");
            send(
                stdin,
                CarrierChannel::Data,
                &CoordinatorFrame::Request {
                    request_id: request_id.clone(),
                    method: method.into(),
                    args: args.clone(),
                },
            );
            assert_eq!(
                recv_request_reply(
                    rx,
                    stdin,
                    seen_control_methods,
                    CarrierChannel::Data,
                    &request_id,
                ),
                expected_reply
            );
            expected_http.push((method, args));
            let observed = gateway.api_requests();
            assert_eq!(observed.len(), initial_http_count + expected_http.len());
            assert_box_mcp_http_requests(&observed, &expected_http);
        }
        expected_http
    }

    fn pump_control_until_webauthn_result(
        rx: &mpsc::Receiver<Result<(CarrierChannel, CoordinatorFrame), serde_json::Error>>,
        stdin: &mut impl Write,
        seen_control_methods: &mut Vec<String>,
        gateway: &FakeGateway,
    ) -> Value {
        let deadline = std::time::Instant::now() + Duration::from_secs(5);
        loop {
            if let Some(value) = gateway.webauthn_result() {
                return value;
            }
            assert!(
                std::time::Instant::now() < deadline,
                "shipping Coordinator did not deliver the WebAuthn result"
            );
            match rx.recv_timeout(Duration::from_millis(50)) {
                Ok(Ok((CarrierChannel::Control, frame))) => {
                    service_control_frame(stdin, seen_control_methods, frame);
                }
                Ok(Ok((_channel, CoordinatorFrame::Event { .. }))) => {}
                Ok(Ok((channel, frame))) => panic!(
                    "unexpected application frame while awaiting WebAuthn result on {channel:?}: {frame:?}"
                ),
                Ok(Err(error)) => panic!("invalid Coordinator frame: {error}"),
                Err(mpsc::RecvTimeoutError::Timeout) => {}
                Err(mpsc::RecvTimeoutError::Disconnected) => {
                    panic!("Coordinator output closed while awaiting WebAuthn result")
                }
            }
        }
    }

    fn complete_live_oauth(port: u16) -> String {
        let mut stream = None;
        for _ in 0..200 {
            match TcpStream::connect(("127.0.0.1", port)) {
                Ok(value) => {
                    stream = Some(value);
                    break;
                }
                Err(_) => thread::sleep(Duration::from_millis(10)),
            }
        }
        let mut stream = stream.expect("connect to Coordinator OAuth callback listener");
        stream
            .set_read_timeout(Some(Duration::from_secs(2)))
            .expect("set OAuth callback read timeout");
        stream
            .set_write_timeout(Some(Duration::from_secs(2)))
            .expect("set OAuth callback write timeout");
        write!(
            stream,
            "GET /oauth/callback?code=code-live&state=state-live HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nConnection: close\r\n\r\n"
        )
        .expect("write OAuth callback");
        stream.flush().expect("flush OAuth callback");
        let mut response = String::new();
        stream
            .read_to_string(&mut response)
            .expect("read OAuth callback response");
        response
    }


    fn fake_webauthn_signer(data_dir: &Path) -> (PathBuf, PathBuf, PathBuf) {
        let request_path = data_dir.join("webauthn-signer-request.json");
        let pin_path = data_dir.join("webauthn-signer-pin.json");
        let path = data_dir.join("fake-webauthn-signer.sh");
        fs::write(
            &path,
            format!(
                "#!/bin/sh\n\
                 IFS= read -r request\n\
                 printf '%s' \"$request\" > \"{}\"\n\
                 printf '%s\\n' '[signer-event] {{\"kind\":\"pin-required\"}}' >&2\n\
                 IFS= read -r pin\n\
                 printf '%s' \"$pin\" > \"{}\"\n\
                 printf '%s' '{{\"ok\":true,\"credentialJson\":{{\"id\":\"credential-production\"}}}}'\n",
                request_path.display(),
                pin_path.display(),
            ),
        )
        .expect("write fake WebAuthn signer");
        let mut permissions = fs::metadata(&path)
            .expect("fake WebAuthn signer metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make fake WebAuthn signer executable");
        (path, request_path, pin_path)
    }

    fn fake_host() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock after unix epoch")
            .as_nanos();
        let dir = std::env::temp_dir().join(format!(
            "mahayana-coordinator-production-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&dir).expect("create fake host directory");
        let path = dir.join("fake-host.sh");
        fs::write(
            &path,
            r#"#!/bin/sh
cat > "$SAND_DATA_ROOT/gateway.json" <<EOF
{"port":${FABUSHI_TEST_GATEWAY_PORT},"pid":$$,"startedAt":1,"scheme":"http","host":"127.0.0.1","token":"${FABUSHI_TEST_GATEWAY_TOKEN}"}
EOF
printf '%s\n' '{"event":{"type":"host.test","value":1}}'
while [ ! -f "$SAND_DATA_ROOT/fake-host-crash" ]; do
  sleep 0.1
done
exit 17
"#,
        )
        .expect("write fake host");
        let mut permissions = fs::metadata(&path).expect("fake host metadata").permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&path, permissions).expect("make fake host executable");
        path
    }


    #[test]
    fn shipping_coordinator_binary_enforces_protocol_and_settles_host_crash() {
        let host = fake_host();
        let data_dir = host.parent().expect("fake host parent").to_path_buf();
        let mut gateway = FakeGateway::start(&data_dir);
        let (webauthn_signer, webauthn_request_path, webauthn_pin_path) =
            fake_webauthn_signer(&data_dir);
        let coordinator = env!("CARGO_BIN_EXE_mahayana-node-agent-coordinator");
        let bootstrap = format!(
            "--bootstrap={}",
            json!({
                "processConfig": {
                    "appVersion": "test-0.18",
                    "isPackaged": false,
                    "dataDir": host.parent().expect("fake host parent").to_string_lossy()
                }
            })
        );
        let mut child = Command::new(coordinator)
            .arg(bootstrap)
            .env("MAHAYANA_APP_HOST_BIN", &host)
            .env("FABUSHI_TEST_GATEWAY_PORT", gateway.port().to_string())
            .env("FABUSHI_TEST_GATEWAY_TOKEN", gateway.token())
            .env("SAND_WEBAUTHN_SIGNER_PATH", &webauthn_signer)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .expect("spawn shipping coordinator binary");

        let mut stdin = child.stdin.take().expect("coordinator stdin");
        let stdout = child.stdout.take().expect("coordinator stdout");
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            for line in BufReader::new(stdout).lines() {
                let Ok(line) = line else { break };
                let parsed = serde_json::from_str::<CarrierEnvelope>(&line).and_then(|envelope| {
                    let channel = envelope
                        .classify()
                        .ok_or_else(|| serde_json::Error::io(std::io::Error::other("unknown channel")))?;
                    let frame = serde_json::from_value::<CoordinatorFrame>(envelope.frame)?;
                    Ok((channel, frame))
                });
                if tx.send(parsed).is_err() {
                    break;
                }
            }
        });

        let (channel, hello) = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("control hello")
            .expect("valid control hello");
        assert_eq!(channel, CarrierChannel::Control);
        assert_eq!(hello, CoordinatorFrame::hello());
        send(&mut stdin, CarrierChannel::Control, &CoordinatorFrame::ready());
        let mut seen_control_methods = Vec::new();

        for channel in [CarrierChannel::Data, CarrierChannel::MainData] {
            send(
                &mut stdin,
                channel,
                &CoordinatorFrame::Lifecycle {
                    phase: LifecyclePhase::Hello,
                    protocol_version: Some(COORDINATOR_PROTOCOL_VERSION),
                    reason: None,
                    detail: None,
                },
            );
            assert_eq!(
                recv_application_frame(
                    &rx,
                    &mut stdin,
                    &mut seen_control_methods,
                    Duration::from_secs(2),
                ),
                (channel, CoordinatorFrame::ready())
            );
        }

        send(
            &mut stdin,
            CarrierChannel::Data,
            &CoordinatorFrame::Request {
                request_id: "r-health".into(),
                method: "coordinator.health".into(),
                args: json!({}),
            },
        );
        let (channel, health) = recv_application_frame(
            &rx,
            &mut stdin,
            &mut seen_control_methods,
            Duration::from_secs(2),
        );
        assert_eq!(channel, CarrierChannel::Data);
        match health {
            CoordinatorFrame::Reply {
                request_id,
                outcome: ReplyOutcome::Ok { value },
            } => {
                assert_eq!(request_id, "r-health");
                assert_eq!(value["protocolVersion"], 1);
                assert_eq!(value["processConfig"]["appVersion"], "test-0.18");
                assert_eq!(value["processConfig"]["isPackaged"], false);
                assert_eq!(value["inference"]["provider"], "cursor");
                assert_eq!(value["inference"]["queueWorkers"], 0);
            }
            other => panic!("unexpected health frame: {other:?}"),
        }

        send(
            &mut stdin,
            CarrierChannel::Data,
            &CoordinatorFrame::Request {
                request_id: "r-echo".into(),
                method: "echo".into(),
                args: json!({"value": 1}),
            },
        );

        let mut saw_host_running = false;
        let mut saw_runtime_event = false;
        let mut saw_tool_event = false;
        let mut saw_echo_reply = false;
        for _ in 0..12 {
            let (channel, frame) = recv_application_frame(
                &rx,
                &mut stdin,
                &mut seen_control_methods,
                Duration::from_secs(2),
            );
            assert_eq!(channel, CarrierChannel::Data);
            match frame {
                CoordinatorFrame::Event { family, payload } if family == "runtime" => {
                    if payload.get("type").and_then(|value| value.as_str()) == Some("host.lifecycle")
                        && payload.get("lifecycle").and_then(|value| value.as_str()) == Some("running")
                    {
                        saw_host_running = true;
                    }
                    if payload.get("type").and_then(|value| value.as_str()) == Some("host.test") {
                        saw_runtime_event = true;
                    }
                }
                CoordinatorFrame::Event { family, payload }
                    if family == "client-side-tool-v2" =>
                {
                    assert_eq!(payload["agentId"], "agent-tool");
                    assert_eq!(payload["epoch"], "epoch-1");
                    assert_eq!(payload["sequence"], 1);
                    saw_tool_event = true;
                }
                CoordinatorFrame::Reply {
                    request_id,
                    outcome: ReplyOutcome::Ok { value },
                } if request_id == "r-echo" => {
                    assert_eq!(value, json!({"echoed": true}));
                    saw_echo_reply = true;
                }
                _ => {}
            }
            if saw_host_running && saw_runtime_event && saw_tool_event && saw_echo_reply {
                break;
            }
        }
        assert!(saw_host_running, "shipping Coordinator did not expose Host lifecycle");
        assert!(saw_runtime_event, "shipping Coordinator did not relay Host event");
        assert!(saw_tool_event, "shipping Coordinator did not relay client-side tool events");
        assert!(saw_echo_reply, "shipping Coordinator did not settle Host reply");

        send(
            &mut stdin,
            CarrierChannel::Data,
            &CoordinatorFrame::Request {
                request_id: "r-send".into(),
                method: "sendPrompt".into(),
                args: json!({
                    "clientNonce": "production-send-1",
                    "message": "production gateway command policy"
                }),
            },
        );
        let (channel, send_reply) = recv_application_frame(
            &rx,
            &mut stdin,
            &mut seen_control_methods,
            Duration::from_secs(2),
        );
        assert_eq!(channel, CarrierChannel::Data);
        match send_reply {
            CoordinatorFrame::Reply {
                request_id,
                outcome: ReplyOutcome::Ok { value },
            } => {
                assert_eq!(request_id, "r-send");
                assert_eq!(value["accepted"], true);
            }
            other => panic!("unexpected sendPrompt frame: {other:?}"),
        }

        let webauthn_result = pump_control_until_webauthn_result(
            &rx,
            &mut stdin,
            &mut seen_control_methods,
            &gateway,
        );
        assert_eq!(webauthn_result["credentialJson"]["id"], "credential-production");
        let signer_request: Value = serde_json::from_str(
            &fs::read_to_string(&webauthn_request_path).expect("read WebAuthn signer request"),
        )
        .expect("WebAuthn signer request JSON");
        assert_eq!(signer_request["windowHandle"].as_u64(), Some(4_294_967_297));
        let signer_pin: Value = serde_json::from_str(
            &fs::read_to_string(&webauthn_pin_path).expect("read WebAuthn signer PIN reply"),
        )
        .expect("WebAuthn signer PIN JSON");
        assert_eq!(signer_pin, json!({ "kind": "pin", "pin": "2468" }));
        for method in [
            "requestWebAuthnConsent",
            "requestWebAuthnPin",
            "finishWebAuthnConsent",
        ] {
            assert!(
                seen_control_methods.iter().any(|seen| seen == method),
                "shipping Coordinator did not wire production WebAuthn control method {method}"
            );
        }

        assert!(
            seen_control_methods
                .iter()
                .any(|method| method == "resolveGatewayConnection"),
            "shipping Coordinator did not start LocalExec through the control port"
        );
        assert!(
            seen_control_methods
                .iter()
                .any(|method| method == "mintLocalExecDaemonCredential"),
            "shipping Coordinator did not request the LocalExec credential through the control port"
        );

        let oauth_response = complete_live_oauth(gateway.oauth_callback_port());
        assert!(
            oauth_response.starts_with("HTTP/1.1 200 OK"),
            "Coordinator OAuth callback did not complete successfully: {oauth_response}"
        );
        assert_eq!(
            gateway.wait_for_oauth_completion(),
            json!({
                "code": "code-live",
                "state": "state-live"
            }),
            "Coordinator did not forward the browser callback to completeMcpOAuth"
        );
        assert_eq!(
            gateway.oauth_completion_attempts(),
            2,
            "shipping Coordinator did not retry one transient OAuth completion failure"
        );

        let expected_box_mcp_requests = exercise_box_mcp_native_routing(
            &rx,
            &mut stdin,
            &mut seen_control_methods,
            &gateway,
        );

        send(
            &mut stdin,
            CarrierChannel::Data,
            &CoordinatorFrame::Request {
                request_id: "r-crash".into(),
                method: "crash".into(),
                args: json!({}),
            },
        );

        let expected_failure = Failure::new(
            COORDINATOR_DISCONNECTED,
            "placeholder overwritten below",
        );
        let mut saw_crash_reply = false;
        let mut saw_stopped = false;
        for _ in 0..10 {
            let (channel, frame) = recv_application_frame(
                &rx,
                &mut stdin,
                &mut seen_control_methods,
                Duration::from_secs(3),
            );
            assert_eq!(channel, CarrierChannel::Data);
            match frame {
                CoordinatorFrame::Reply {
                    request_id,
                    outcome: ReplyOutcome::Failed { failure },
                } if request_id == "r-crash" => {
                    assert_eq!(failure.code, expected_failure.code);
                    assert!(failure.message.contains("Host exited"));
                    saw_crash_reply = true;
                }
                CoordinatorFrame::Event { family, payload } if family == "runtime" => {
                    if payload.get("type").and_then(|value| value.as_str()) == Some("host.lifecycle")
                        && payload.get("lifecycle").and_then(|value| value.as_str()) == Some("stopped")
                    {
                        saw_stopped = true;
                    }
                }
                _ => {}
            }
            if saw_crash_reply && saw_stopped {
                break;
            }
        }
        assert!(saw_crash_reply, "Host crash did not reject the pending request");
        assert!(saw_stopped, "Host crash did not emit stopped lifecycle");

        fs::write(
            data_dir.join("settings.json"),
            r#"{"inferenceProvider":"codex"}"#,
        )
        .expect("switch shipping Coordinator to local inference provider");
        send(
            &mut stdin,
            CarrierChannel::Data,
            &CoordinatorFrame::Request {
                request_id: "r-local-inference".into(),
                method: "sendPrompt".into(),
                args: json!({
                    "agentId": "agent-local",
                    "prompt": "",
                    "clientNonce": "local-inference-1"
                }),
            },
        );
        let mut saw_local_accept = false;
        for _ in 0..8 {
            let (channel, frame) = recv_application_frame(
                &rx,
                &mut stdin,
                &mut seen_control_methods,
                Duration::from_secs(2),
            );
            assert_eq!(channel, CarrierChannel::Data);
            if let CoordinatorFrame::Reply {
                request_id,
                outcome: ReplyOutcome::Ok { value },
            } = frame
                && request_id == "r-local-inference"
            {
                assert_eq!(value["accepted"], true);
                assert_eq!(value["provider"], "codex");
                assert_eq!(value["clientNonce"], "local-inference-1");
                saw_local_accept = true;
                break;
            }
        }
        assert!(
            saw_local_accept,
            "shipping Coordinator did not accept a local-provider sendPrompt"
        );

        // Provider/router failures remain recoverable diagnostics, but the
        // renderer-visible transcript is Host-owned and must not receive a
        // synthetic assistant "Router error" message.
        let inference_path = data_dir.join("inference-router-transcript.json");
        let mut saw_private_local_error = false;
        for _ in 0..100 {
            if let Ok(raw) = fs::read_to_string(&inference_path)
                && let Ok(value) = serde_json::from_str::<Value>(&raw)
            {
                saw_private_local_error = value["agents"]["agent-local"]
                    .as_array()
                    .into_iter()
                    .flatten()
                    .any(|entry| {
                        entry["role"] == "assistant"
                            && entry["content"]
                                .as_str()
                                .is_some_and(|content| content.contains("Router error:"))
                    });
                if saw_private_local_error {
                    break;
                }
            }
            thread::sleep(Duration::from_millis(20));
        }
        assert!(
            saw_private_local_error,
            "shipping Coordinator did not retain a failed local-provider turn in private inference diagnostics"
        );

        drop(stdin);
        let status = child.wait().expect("wait for Coordinator");
        assert!(status.success(), "Coordinator did not shut down cleanly: {status}");
        gateway.stop_and_join().expect("join fake Host gateway and all owned HTTP exchanges");
        // Coordinator exit plus the drained listener/handler join closes the
        // observation interval. No sleep or early empty snapshot proves zero:
        // all three Data requests are absent, and even the uncertain execution
        // failure reached HTTP exactly once without a transport retry.
        assert_box_mcp_http_requests(&gateway.api_requests(), &expected_box_mcp_requests);
        drop(gateway);
        let _ = fs::remove_dir_all(data_dir);
    }
}
