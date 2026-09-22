#[cfg(unix)]
mod unix {
    use mahayana_node_agent_coordinator::protocol::{
        CoordinatorFrame, Failure, LifecyclePhase, ReplyOutcome, COORDINATOR_DISCONNECTED,
        COORDINATOR_PROTOCOL_VERSION,
    };
    use serde_json::json;
    use std::fs;
    use std::io::{BufRead, BufReader, Write};
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::process::{Command, Stdio};
    use std::sync::mpsc;
    use std::thread;
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    fn send(stdin: &mut impl Write, frame: &CoordinatorFrame) {
        serde_json::to_writer(&mut *stdin, frame).expect("serialize coordinator frame");
        writeln!(stdin).expect("write coordinator frame");
        stdin.flush().expect("flush coordinator frame");
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
while IFS= read -r line; do
  case "$line" in
    *'"method":"echo"'*)
      printf '%s\n' '{"event":{"type":"host.test","value":1}}'
      printf '%s\n' '{"id":"r-echo","ok":true,"result":{"echoed":true}}'
      ;;
    *'"method":"crash"'*)
      exit 17
      ;;
    *)
      printf '%s\n' '{"id":"unknown","ok":false,"error":"unexpected fake-host request"}'
      ;;
  esac
done
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
        let coordinator = env!("CARGO_BIN_EXE_mahayana-node-agent-coordinator");
        let mut child = Command::new(coordinator)
            .env("MAHAYANA_APP_HOST_BIN", &host)
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
                let parsed = serde_json::from_str::<CoordinatorFrame>(&line);
                if tx.send(parsed).is_err() {
                    break;
                }
            }
        });

        send(
            &mut stdin,
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
            CoordinatorFrame::ready()
        );

        send(
            &mut stdin,
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
            let frame = rx
                .recv_timeout(Duration::from_secs(2))
                .expect("frame while serving echo")
                .expect("valid coordinator frame");
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
            let frame = rx
                .recv_timeout(Duration::from_secs(3))
                .expect("frame while settling crashed Host")
                .expect("valid coordinator frame");
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
        let _ = fs::remove_dir_all(host.parent().expect("fake host parent"));
    }
}
