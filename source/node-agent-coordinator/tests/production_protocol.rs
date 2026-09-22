#[cfg(unix)]
mod unix {
    use mahayana_node_agent_coordinator::carrier::{
        CarrierChannel, CarrierEnvelope,
    };
    use mahayana_node_agent_coordinator::protocol::{
        CoordinatorFrame, Failure, LifecyclePhase, ReplyOutcome, COORDINATOR_DISCONNECTED,
        COORDINATOR_PROTOCOL_VERSION,
    };
    use serde_json::json;
    use std::fs;
    use std::io::{self, BufRead, BufReader, Read, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::net::{TcpListener, TcpStream};
    use std::path::{Path, PathBuf};
    use std::process::{Command, Stdio};
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
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

    struct FakeGateway {
        address: std::net::SocketAddr,
        token: String,
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
            let crash_path = data_dir.join("fake-host-crash");
            let stop = Arc::new(AtomicBool::new(false));
            let worker_stop = Arc::clone(&stop);
            let worker_token = token.clone();
            let worker = thread::spawn(move || {
                while !worker_stop.load(Ordering::Acquire) {
                    match listener.accept() {
                        Ok((stream, _)) => {
                            let token = worker_token.clone();
                            let crash_path = crash_path.clone();
                            let handler_stop = Arc::clone(&worker_stop);
                            thread::spawn(move || {
                                let _ = serve_fake_gateway(
                                    stream,
                                    &token,
                                    &crash_path,
                                    handler_stop.as_ref(),
                                );
                            });
                        }
                        Err(error) if error.kind() == io::ErrorKind::WouldBlock => {
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
            });
            Self {
                address,
                token,
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
    }

    impl Drop for FakeGateway {
        fn drop(&mut self) {
            self.stop.store(true, Ordering::Release);
            let _ = TcpStream::connect(self.address);
            if let Some(worker) = self.worker.take() {
                let _ = worker.join();
            }
        }
    }

    fn serve_fake_gateway(
        mut stream: TcpStream,
        expected_token: &str,
        crash_path: &Path,
        stop: &AtomicBool,
    ) -> io::Result<()> {
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

        match (method.as_str(), path.as_str()) {
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
            _ => "Response",
        };
        write!(
            stream,
            "HTTP/1.1 {status} {reason}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        )?;
        stream.flush()
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
        let gateway = FakeGateway::start(&data_dir);
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
                rx.recv_timeout(Duration::from_secs(2))
                    .expect("ready frame")
                    .expect("valid ready frame"),
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
        let (channel, health) = rx
            .recv_timeout(Duration::from_secs(2))
            .expect("health reply")
            .expect("valid health reply");
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
        let mut saw_echo_reply = false;
        for _ in 0..8 {
            let (channel, frame) = rx
                .recv_timeout(Duration::from_secs(2))
                .expect("frame while serving echo")
                .expect("valid coordinator frame");
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
                CoordinatorFrame::Reply {
                    request_id,
                    outcome: ReplyOutcome::Ok { value },
                } if request_id == "r-echo" => {
                    assert_eq!(value, json!({"echoed": true}));
                    saw_echo_reply = true;
                }
                _ => {}
            }
            if saw_host_running && saw_runtime_event && saw_echo_reply {
                break;
            }
        }
        assert!(saw_host_running, "shipping Coordinator did not expose Host lifecycle");
        assert!(saw_runtime_event, "shipping Coordinator did not relay Host event");
        assert!(saw_echo_reply, "shipping Coordinator did not settle Host reply");

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
            let (channel, frame) = rx
                .recv_timeout(Duration::from_secs(3))
                .expect("frame while settling crashed Host")
                .expect("valid coordinator frame");
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

        drop(stdin);
        let status = child.wait().expect("wait for Coordinator");
        assert!(status.success(), "Coordinator did not shut down cleanly: {status}");
        drop(gateway);
        let _ = fs::remove_dir_all(data_dir);
    }
}
