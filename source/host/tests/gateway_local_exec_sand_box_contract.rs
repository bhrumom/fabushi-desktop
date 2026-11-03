use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::process::{Child, Command};
use std::time::{Duration, Instant};

use mahayana_host_runtime::durable_file_policy::SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME;
use mahayana_host_runtime::extensions::local_exec::extension::{
    HostLocalExecExtension, start_local_exec_extension,
};
use mahayana_host_runtime::extensions::local_exec::gateway_local_exec_sand_box::{
    DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT, DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES,
    MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES, GatewayLocalExecFailureReport, GatewayLocalExecManager,
    GatewayLocalExecOperationRecord, GatewayLocalExecOperationStatus, GatewayLocalExecSandBox,
    GatewayLocalExecShellResult, GatewayLocalToolGate, GatewayLocalToolScope,
    create_bridge_user_computers, local_exec_file_too_large_message,
};
use mahayana_host_runtime::extensions::local_exec::local_exec_failure_classifier::LocalExecFailureClass;
use mahayana_host_runtime::extensions::local_exec::local_exec_bridge::{
    LocalExecProviderRegistration, SandLocalExecBridge,
};
use mahayana_host_runtime::extensions::local_exec::local_exec_error::SandLocalExecError;
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::HostTelemetryService;
use mahayana_host_runtime::host_paths::{SAND_DATA_ROOT_ENV, get_sand_root_dir};
use serde_json::{Value, json};

#[derive(Default)]
struct FakeGate {
    blocked: Mutex<Option<String>>,
    calls: Mutex<Vec<(Option<String>, String, String)>>,
}

impl GatewayLocalToolGate for FakeGate {
    fn blocked_reason(&self) -> Option<String> {
        self.blocked.lock().expect("blocked").clone()
    }

    fn requires_approval(&self) -> bool {
        true
    }

    fn authorize(
        &self,
        scope: &GatewayLocalToolScope,
        action: &str,
        target: &str,
    ) -> Result<Option<String>, SandLocalExecError> {
        self.calls.lock().expect("calls").push((
            scope.agent_id.clone(),
            action.to_string(),
            target.to_string(),
        ));
        Ok(Some("approval-1".into()))
    }
}

fn bridge_with_provider() -> (
    SandLocalExecBridge,
    mahayana_host_runtime::extensions::local_exec::local_exec_bridge::LocalExecProviderRegistration,
    mpsc::Receiver<serde_json::Value>,
) {
    let sequence = Arc::new(std::sync::atomic::AtomicU64::new(1));
    let next = Arc::clone(&sequence);
    let bridge = SandLocalExecBridge::with_sources(
        Arc::new(|| 1000),
        Arc::new(move || format!(
            "fixture-request-{}",
            next.fetch_add(1, std::sync::atomic::Ordering::Relaxed),
        )),
    );
    let (send, receive) = mpsc::channel();
    let registration = bridge.register_provider(send);
    let welcome = receive.recv_timeout(Duration::from_millis(100)).expect("welcome");
    bridge.submit_responses(json!({
        "providerId": welcome["providerId"],
        "frames": [{
            "kind": "hello",
            "localRoot": "/Users/test",
            "terminalsFolder": "terminal-files",
            "computerId": "mac-1",
            "label": "My Mac"
        }]
    }));
    (bridge, registration, receive)
}

#[test]
fn sandbox_projects_provider_state_and_user_computers() {
    let (bridge, _registration, _receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let sandbox = GatewayLocalExecSandBox::new(bridge.clone(), gate.clone());
    assert_eq!(sandbox.run_state(), "running");
    assert_eq!(sandbox.ensure_ready().terminals_folder, "terminal-files");
    assert!(sandbox.list_boxes().is_empty());

    let computers = create_bridge_user_computers(bridge, gate);
    assert_eq!(computers.list().len(), 1);
    let (computer, selected) = computers.resolve(Some("mac-1")).expect("computer");
    assert_eq!(computer.label, "My Mac");
    assert_eq!(selected.run_state(), "running");
}

#[test]
fn upload_and_download_enforce_gate_and_use_frozen_frames() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let sandbox = GatewayLocalExecSandBox::new(bridge.clone(), gate.clone());

    let upload_box = sandbox.clone();
    let upload = thread::spawn(move || {
        upload_box.upload_file(Some("agent-1"), "/tmp/a.txt", b"hello")
    });

    let upload_start = receive.recv_timeout(Duration::from_secs(1)).expect("upload-start frame");
    assert_eq!(upload_start["kind"], "upload-start");
    assert_eq!(upload_start["path"], "/tmp/a.txt");
    assert_eq!(upload_start["totalBytes"], 5);
    assert_eq!(upload_start["approvalId"], "approval-1");
    let transfer_id = upload_start["transferId"]
        .as_str()
        .expect("upload-start transferId")
        .to_string();
    assert_eq!(upload_start["sha256"], "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824");
    bridge.submit_responses(json!({
        "frames": [{
            "kind": "file",
            "requestId": upload_start["requestId"]
        }]
    }));
    let start_cancel = receive.recv_timeout(Duration::from_secs(1)).expect("upload-start cleanup cancel");
    assert_eq!(start_cancel["kind"], "cancel");
    assert_eq!(start_cancel["requestId"], upload_start["requestId"]);

    let upload_chunk = receive.recv_timeout(Duration::from_secs(1)).expect("upload-chunk frame");
    assert_eq!(upload_chunk["kind"], "upload-chunk");
    assert_eq!(upload_chunk["path"], "/tmp/a.txt");
    assert_eq!(upload_chunk["transferId"], transfer_id);
    assert_eq!(upload_chunk["offset"], 0);
    assert_eq!(upload_chunk["bytesBase64"], "aGVsbG8=");
    assert_eq!(upload_chunk["approvalId"], "approval-1");
    bridge.submit_responses(json!({
        "frames": [{
            "kind": "file",
            "requestId": upload_chunk["requestId"]
        }]
    }));
    let chunk_cancel = receive.recv_timeout(Duration::from_secs(1)).expect("upload-chunk cleanup cancel");
    assert_eq!(chunk_cancel["kind"], "cancel");
    assert_eq!(chunk_cancel["requestId"], upload_chunk["requestId"]);

    let upload_commit = receive.recv_timeout(Duration::from_secs(1)).expect("upload-commit frame");
    assert_eq!(upload_commit["kind"], "upload-commit");
    assert_eq!(upload_commit["path"], "/tmp/a.txt");
    assert_eq!(upload_commit["transferId"], transfer_id);
    assert_eq!(upload_commit["approvalId"], "approval-1");
    bridge.submit_responses(json!({
        "frames": [{
            "kind": "file",
            "requestId": upload_commit["requestId"]
        }]
    }));
    let commit_cancel = receive.recv_timeout(Duration::from_secs(1)).expect("upload-commit cleanup cancel");
    assert_eq!(commit_cancel["kind"], "cancel");
    assert_eq!(commit_cancel["requestId"], upload_commit["requestId"]);
    upload.join().expect("upload thread").expect("upload");

    let download_box = sandbox.clone();
    let download = thread::spawn(move || {
        download_box.download_file(Some("agent-1"), "/tmp/b.txt")
    });
    let download_frame = receive.recv_timeout(Duration::from_secs(1)).expect("download frame");
    assert_eq!(download_frame["kind"], "download");
    assert_eq!(download_frame["path"], "/tmp/b.txt");
    assert_eq!(download_frame["approvalId"], "approval-1");
    bridge.submit_responses(json!({
        "frames": [{
            "kind": "file",
            "requestId": download_frame["requestId"],
            "bytesBase64": "d29ybGQ="
        }]
    }));
    assert_eq!(
        download.join().expect("download thread").expect("download"),
        b"world"
    );

    let calls = gate.calls.lock().expect("calls");
    assert_eq!(
        calls.as_slice(),
        &[
            (Some("agent-1".into()), "write-file".into(), "/tmp/a.txt".into()),
            (Some("agent-1".into()), "read-file".into(), "/tmp/b.txt".into()),
        ]
    );
}

#[test]
fn unavailable_computer_fails_before_permission_approval() {
    let bridge = SandLocalExecBridge::with_sources(
        Arc::new(|| 1000),
        Arc::new(|| "provider-1".to_string()),
    );
    let gate = Arc::new(FakeGate::default());
    let sandbox = GatewayLocalExecSandBox::new(bridge, gate.clone());

    let error = sandbox
        .upload_file(Some("agent"), "/tmp/file", b"hello")
        .expect_err("missing provider");
    assert_eq!(
        error.message,
        mahayana_host_runtime::extensions::local_exec::local_exec_bridge::SAND_NO_LOCAL_MACHINE_MESSAGE
    );
    assert!(gate.calls.lock().expect("calls").is_empty());

    let (bridge, _registration, _receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let sandbox = GatewayLocalExecSandBox::new(bridge, gate.clone()).for_computer("missing");
    let error = sandbox
        .download_file(Some("agent"), "/tmp/file")
        .expect_err("unknown computer");
    assert_eq!(
        error.message,
        mahayana_host_runtime::extensions::local_exec::local_exec_bridge::SAND_NO_LOCAL_MACHINE_MESSAGE
    );
    assert!(gate.calls.lock().expect("calls").is_empty());
}

#[test]
fn oversized_file_and_blocked_gate_fail_before_provider_side_effect() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let sandbox = GatewayLocalExecSandBox::new(bridge, gate.clone()).with_max_file_bytes(4);
    let error = sandbox
        .upload_file(Some("agent"), "/tmp/large", b"12345")
        .expect_err("large file rejected");
    assert_eq!(error.message, local_exec_file_too_large_message(5, 4));
    assert!(receive.recv_timeout(Duration::from_millis(20)).is_err());

    *gate.blocked.lock().expect("blocked") = Some("disabled".into());
    let error = sandbox
        .download_file(Some("agent"), "/tmp/file")
        .expect_err("blocked");
    assert_eq!(error.message, "disabled");
    assert!(receive.recv_timeout(Duration::from_millis(20)).is_err());

    assert_eq!(DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES, 100 * 1024 * 1024);
}


#[test]
fn exec_manager_streams_shell_and_read_through_permissioned_provider() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let manager = GatewayLocalExecSandBox::new(bridge.clone(), gate.clone())
        .remote_resource_accessor()
        .manager();

    let shell_manager = Arc::clone(&manager);
    let shell = thread::spawn(move || {
        shell_manager.execute_shell(
            &GatewayLocalToolScope {
                agent_id: Some("agent-1".into()),
                tool_call_id: Some("shell-call".into()),
                action: Some("run-command".into()),
            },
            "pwd",
            "/tmp",
            "shell-call",
        )
    });
    let shell_frame = receive.recv_timeout(Duration::from_secs(1)).expect("shell frame");
    assert_eq!(shell_frame["kind"], "exec");
    assert_eq!(shell_frame["approvalId"], "approval-1");
    assert_eq!(shell_frame["serverMessage"]["shellStreamArgs"]["command"], "pwd");
    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":shell_frame["requestId"],
        "message":{"id":1,"shellStream":{"stdout":{"data":"hello\n"}}}
    }]}));
    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":shell_frame["requestId"],
        "message":{"id":1,"shellStream":{"exit":{"code":0,"cwd":"/tmp","aborted":false}}}
    }]}));
    let shell_result = shell.join().expect("shell thread").expect("shell");
    assert_eq!(shell_result.stdout, "hello\n");
    assert_eq!(shell_result.exit_code, Some(0));
    let _ = receive.recv_timeout(Duration::from_secs(1)).expect("shell cancel");

    let read_manager = Arc::clone(&manager);
    let read = thread::spawn(move || {
        read_manager.execute_read(
            &GatewayLocalToolScope {
                agent_id: Some("agent-1".into()),
                tool_call_id: Some("read-call".into()),
                action: Some("read-file".into()),
            },
            "/tmp/a.txt",
            "read-call",
            Some(2),
            Some(4),
            Some("utf-8"),
        )
    });
    let read_frame = receive.recv_timeout(Duration::from_secs(1)).expect("read frame");
    assert_eq!(read_frame["kind"], "exec");
    assert_eq!(read_frame["serverMessage"]["readArgs"]["path"], "/tmp/a.txt");
    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":read_frame["requestId"],
        "message":{"id":2,"readResult":{"success":{"path":"/tmp/a.txt","content":"hello"}}}
    }]}));
    let read_result = read.join().expect("read thread").expect("read");
    assert_eq!(read_result["success"]["content"], "hello");
    let _ = receive.recv_timeout(Duration::from_secs(1)).expect("read cancel");

    let calls = gate.calls.lock().expect("calls");
    assert!(calls.iter().any(|(_, action, target)| action == "run-command" && target == "pwd"));
    assert!(calls.iter().any(|(_, action, target)| action == "read-file" && target == "/tmp/a.txt"));
}

#[test]
fn shell_send_failure_finishes_pre_send_operation_as_failed() {
    let (bridge, _registration, receive) = bridge_with_provider();
    drop(receive);
    let gate = Arc::new(FakeGate::default());
    let manager = GatewayLocalExecSandBox::new(bridge, gate)
        .remote_resource_accessor()
        .manager();

    let error = manager
        .execute_shell(
            &GatewayLocalToolScope {
                agent_id: Some("agent-1".into()),
                tool_call_id: Some("send-failure".into()),
                action: Some("run-command".into()),
            },
            "pwd",
            "/tmp",
            "send-failure",
        )
        .expect_err("provider send failure must fail the shell operation");
    assert!(!error.message.is_empty());
    assert!(!error.is_outcome_unknown(), "a rejected transport send entered no execution");
    let record = manager
        .operation("send-failure")
        .expect("failed pre-send operation remains auditable");
    assert_eq!(record.status, GatewayLocalExecOperationStatus::Failed);
}


#[test]
fn exec_manager_reports_control_throw_with_frozen_failure_classification() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let reports = Arc::new(Mutex::new(Vec::<GatewayLocalExecFailureReport>::new()));
    let report_sink = Arc::clone(&reports);
    let manager = GatewayLocalExecSandBox::new(bridge.clone(), gate)
        .with_failure_reporter(Arc::new(move |report| {
            report_sink.lock().expect("reports").push(report);
        }))
        .remote_resource_accessor()
        .manager();

    let shell_manager = Arc::clone(&manager);
    let shell = thread::spawn(move || {
        shell_manager.execute_shell(
            &GatewayLocalToolScope {
                agent_id: Some("agent-telemetry".into()),
                tool_call_id: Some("shell-failure".into()),
                action: Some("run-command".into()),
            },
            "rg needle",
            "/tmp",
            "shell-failure",
        )
    });
    let frame = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("exec frame");
    bridge.submit_responses(json!({"frames":[{
        "kind":"control",
        "requestId":frame["requestId"],
        "message":{"throw":{
            "error":"Error: spawn rg ENOENT",
            "stackTrace":"provider-stack"
        }}
    }]}));

    let error = shell
        .join()
        .expect("shell thread")
        .expect_err("provider throw must fail");
    assert!(error.to_string().contains("Error: spawn rg ENOENT"));
    assert!(error.to_string().contains("provider-stack"));
    assert!(error.is_outcome_unknown(), "error wording is not a positive pre-entry witness");
    assert_eq!(
        manager.operation("shell-failure").expect("uncertain operation").status,
        GatewayLocalExecOperationStatus::OutcomeUnknown,
    );
    let _ = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("exec cleanup cancel");

    assert_eq!(
        reports.lock().expect("reports").as_slice(),
        &[GatewayLocalExecFailureReport {
            error_class: LocalExecFailureClass::SpawnEnoent,
            errno: Some("ENOENT".into()),
            site: "exec".into(),
            conversation_id: Some("agent-telemetry".into()),
        }]
    );
}


#[test]
fn exec_manager_journals_stable_operation_id_and_preserves_uncertain_cancel() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let manager = GatewayLocalExecSandBox::new(bridge.clone(), gate)
        .remote_resource_accessor()
        .manager();

    let shell_manager = Arc::clone(&manager);
    let shell = thread::spawn(move || {
        shell_manager.execute_shell(
            &GatewayLocalToolScope {
                agent_id: Some("agent-journal".into()),
                tool_call_id: Some("operation-stable-1".into()),
                action: Some("run-command".into()),
            },
            "sleep 30",
            "/tmp",
            "operation-stable-1",
        )
    });

    let exec = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("exec frame");
    let bridge_request_id = exec["requestId"]
        .as_str()
        .expect("bridge request id")
        .to_string();
    let active = manager
        .operation_for_agent("agent-journal", "operation-stable-1")
        .expect("active operation");
    assert_eq!(active.operation_id, "operation-stable-1");
    assert_eq!(active.agent_id.as_deref(), Some("agent-journal"));
    assert_eq!(active.bridge_request_id, bridge_request_id);
    assert!(
        matches!(
            active.status,
            GatewayLocalExecOperationStatus::Accepted | GatewayLocalExecOperationStatus::Running
        ),
        "operation must be durably accepted before provider execution and may advance to running"
    );
    assert!(manager.operation_for_agent("other-agent", "operation-stable-1").is_none());
    assert!(!manager.cancel_operation_for_agent("other-agent", "operation-stable-1"));

    assert!(manager.cancel_operation_for_agent("agent-journal", "operation-stable-1"));
    assert!(!manager.cancel_operation("missing-operation"));
    let cancel = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("provider cancel");
    assert_eq!(cancel["kind"], "cancel");
    assert_eq!(cancel["requestId"], bridge_request_id);

    bridge.submit_responses(json!({"frames":[{
        "kind":"control",
        "requestId":bridge_request_id,
        "message":{"streamClose":{"id":1}}
    }]}));
    let result = shell.join().expect("shell thread").expect("unknown cancellation result");
    assert_eq!(result.exit_code, None);
    assert!(result.outcome_unknown, "generic stream closure does not establish Exit");
    assert!(!result.aborted, "cancel delivery does not establish an aborted Exit");

    let settled = manager
        .operation_for_agent("agent-journal", "operation-stable-1")
        .expect("settled operation");
    assert_eq!(settled.status, GatewayLocalExecOperationStatus::OutcomeUnknown);
    assert_eq!(settled.bridge_request_id, bridge_request_id);
    assert_eq!(manager.operation_history_for_agent("agent-journal").len(), 1);
    assert!(manager.operation_history_for_agent("other-agent").is_empty());
}

#[test]
fn exec_manager_does_not_mark_cancel_when_provider_rejects_delivery() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let manager = GatewayLocalExecSandBox::new(bridge.clone(), gate)
        .remote_resource_accessor()
        .manager();

    let shell_manager = Arc::clone(&manager);
    let shell = thread::spawn(move || {
        shell_manager.execute_shell(
            &GatewayLocalToolScope {
                agent_id: Some("agent-undelivered-cancel".into()),
                tool_call_id: Some("operation-undelivered-cancel".into()),
                action: Some("run-command".into()),
            },
            "echo provider-disconnected",
            "/tmp",
            "operation-undelivered-cancel",
        )
    });

    let exec = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("exec frame");
    let request_id = exec["requestId"].as_str().expect("request id").to_string();
    drop(receive);

    assert!(!manager.cancel_operation_for_agent(
        "agent-undelivered-cancel",
        "operation-undelivered-cancel"
    ));

    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":request_id,
        "message":{"id":1,"shellStream":{"exit":{"code":0,"cwd":"/tmp","aborted":false}}}
    }]}));
    let result = shell
        .join()
        .expect("shell thread")
        .expect("provider exit remains observable");
    assert_eq!(result.exit_code, Some(0));
    assert!(!result.aborted);
    assert!(!result.outcome_unknown);

    let settled = manager
        .operation_for_agent("agent-undelivered-cancel", "operation-undelivered-cancel")
        .expect("settled operation");
    assert_eq!(settled.status, GatewayLocalExecOperationStatus::Completed);
}


fn real_shell_output(command: &str) -> std::process::Output {
    #[cfg(windows)]
    {
        Command::new("cmd")
            .args(["/C", command])
            .output()
            .expect("spawn real Windows shell")
    }
    #[cfg(not(windows))]
    {
        Command::new("/bin/sh")
            .args(["-c", command])
            .output()
            .expect("spawn real POSIX shell")
    }
}

#[test]
fn exec_manager_drives_real_os_process_to_bounded_terminal_result() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let manager = GatewayLocalExecSandBox::new(bridge.clone(), gate)
        .remote_resource_accessor()
        .manager();
    let working_dir = std::env::temp_dir().to_string_lossy().into_owned();
    let shell_manager = Arc::clone(&manager);
    let shell_working_dir = working_dir.clone();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let agent_id = format!("agent-real-process-{unique}");
    let op_id = format!("operation-real-process-{unique}");
    let shell_agent_id = agent_id.clone();
    let shell_op_id = op_id.clone();
    let shell = thread::spawn(move || {
        shell_manager.execute_shell(
            &GatewayLocalToolScope {
                agent_id: Some(shell_agent_id),
                tool_call_id: Some(shell_op_id.clone()),
                action: Some("run-command".into()),
            },
            "echo fabushi-terminal-real-process",
            &shell_working_dir,
            &shell_op_id,
        )
    });

    let exec = receive
        .recv_timeout(Duration::from_secs(2))
        .expect("real process exec frame");
    assert_eq!(exec["kind"], "exec");
    let request_id = exec["requestId"].as_str().expect("request id").to_string();
    let command = exec["serverMessage"]["shellStreamArgs"]["command"]
        .as_str()
        .expect("shell command");
    let output = real_shell_output(command);
    assert!(output.status.success(), "real child process failed: {:?}", output.status);
    let stdout = String::from_utf8(output.stdout).expect("utf8 stdout");
    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":request_id,
        "message":{"id":1,"shellStream":{"stdout":{"data":stdout}}}
    }]}));
    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":request_id,
        "message":{"id":1,"shellStream":{"exit":{"code":0,"cwd":working_dir,"aborted":false}}}
    }]}));

    let result = shell.join().expect("shell thread").expect("real shell result");
    assert!(result.stdout.contains("fabushi-terminal-real-process"));
    assert_eq!(result.exit_code, Some(0));
    assert!(!result.outcome_unknown);
    let settled = manager
        .operation_for_agent(&agent_id, &op_id)
        .expect("settled real operation");
    assert_eq!(settled.status, GatewayLocalExecOperationStatus::Completed);
    assert!(settled.stdout.contains("fabushi-terminal-real-process"));
    assert!(!settled.stdout_truncated);
    let _ = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("real process cleanup cancel");
}

fn real_cancellable_child() -> Child {
    #[cfg(windows)]
    {
        Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", "Start-Sleep -Seconds 30"])
            .spawn()
            .expect("spawn cancellable Windows child")
    }
    #[cfg(not(windows))]
    {
        Command::new("/bin/sh")
            .args(["-c", "sleep 30"])
            .spawn()
            .expect("spawn cancellable POSIX child")
    }
}

#[test]
fn exec_manager_cancel_maps_to_real_child_termination_and_history() {
    let (bridge, _registration, receive) = bridge_with_provider();
    let gate = Arc::new(FakeGate::default());
    let manager = GatewayLocalExecSandBox::new(bridge.clone(), gate)
        .remote_resource_accessor()
        .manager();
    let working_dir = std::env::temp_dir().to_string_lossy().into_owned();
    let shell_manager = Arc::clone(&manager);
    let shell_working_dir = working_dir.clone();
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let agent_id = format!("agent-real-cancel-{unique}");
    let op_id = format!("operation-real-cancel-{unique}");
    let shell_agent_id = agent_id.clone();
    let shell_op_id = op_id.clone();
    let shell = thread::spawn(move || {
        shell_manager.execute_shell(
            &GatewayLocalToolScope {
                agent_id: Some(shell_agent_id),
                tool_call_id: Some(shell_op_id.clone()),
                action: Some("run-command".into()),
            },
            "echo fabushi-terminal-cancel-started",
            &shell_working_dir,
            &shell_op_id,
        )
    });

    let exec = receive
        .recv_timeout(Duration::from_secs(2))
        .expect("cancel exec frame");
    let request_id = exec["requestId"].as_str().expect("request id").to_string();
    let command = exec["serverMessage"]["shellStreamArgs"]["command"]
        .as_str()
        .expect("shell command");
    assert_eq!(command, "echo fabushi-terminal-cancel-started");
    let mut child = real_cancellable_child();
    assert!(
        child.try_wait().expect("poll cancellable child").is_none(),
        "real child must still be running before cancellation"
    );
    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":request_id,
        "message":{"id":2,"shellStream":{"stdout":{"data":"fabushi-terminal-cancel-started\\n"}}}
    }]}));

    assert!(manager.cancel_operation_for_agent(&agent_id, &op_id));
    let cancel = receive
        .recv_timeout(Duration::from_secs(2))
        .expect("provider cancel frame");
    assert_eq!(cancel["kind"], "cancel");
    assert_eq!(cancel["requestId"], request_id);
    child.kill().expect("terminate cancellable real child");
    let child_status = child.wait().expect("wait for cancelled real child");
    assert!(!child_status.success(), "cancelled real child must not report success");
    bridge.submit_responses(json!({"frames":[{
        "kind":"client",
        "requestId":request_id,
        "message":{"id":2,"shellStream":{"exit":{"code":130,"cwd":working_dir,"aborted":true}}}
    }]}));

    let result = shell.join().expect("cancel shell thread").expect("cancelled result");
    assert_eq!(result.exit_code, Some(130));
    assert!(result.aborted);
    assert!(!result.outcome_unknown, "real termination plus aborted Exit is definitive");
    let settled = manager
        .operation_for_agent(&agent_id, &op_id)
        .expect("settled cancelled operation");
    assert_eq!(settled.status, GatewayLocalExecOperationStatus::Cancelled);
    assert_eq!(manager.operation_history_for_agent(&agent_id).len(), 1);
}


const RETENTION_CHILD_TEST_ENV: &str = "FABUSHI_LXR_CHILD_TEST";
const RETENTION_CHILD_TOKEN_ENV: &str = "FABUSHI_LXR_CHILD_TOKEN";
const RETENTION_AGENT: &str = "lxr-agent";
const RETENTION_BOUND: Duration = Duration::from_secs(5);

struct RetentionChild {
    child: Child,
    reaped: bool,
}

impl RetentionChild {
    fn wait(&mut self) -> Result<std::process::ExitStatus, String> {
        let deadline = Instant::now() + Duration::from_secs(120);
        loop {
            match self.child.try_wait() {
                Ok(Some(status)) => {
                    self.reaped = true;
                    return Ok(status);
                }
                Ok(None) => {}
                Err(error) => return Err(format!("owned retention child wait failed: {error}")),
            }
            if Instant::now() >= deadline {
                return Err("owned retention child exceeded its deadline".into());
            }
            thread::sleep(Duration::from_millis(5));
        }
    }

    fn stop(&mut self) {
        if self.reaped {
            return;
        }
        if matches!(self.child.try_wait(), Ok(Some(_))) {
            self.reaped = true;
            return;
        }
        // This Child was spawned here and creates only worker threads. Never
        // search for or signal a process by name or an inferred process group.
        let kill_result = self.child.kill();
        let deadline = Instant::now() + RETENTION_BOUND;
        while Instant::now() < deadline {
            if matches!(self.child.try_wait(), Ok(Some(_))) {
                self.reaped = true;
                return;
            }
            thread::sleep(Duration::from_millis(5));
        }
        eprintln!(
            "retention child {} could not be reaped; kill result: {kill_result:?}",
            self.child.id(),
        );
    }
}

impl Drop for RetentionChild {
    fn drop(&mut self) {
        self.stop();
    }
}

fn run_retention_contract(test_name: &str, case: impl FnOnce(&Path)) {
    if std::env::var(RETENTION_CHILD_TEST_ENV).ok().as_deref() == Some(test_name) {
        let root = PathBuf::from(
            std::env::var_os(SAND_DATA_ROOT_ENV).expect("private retention child root"),
        );
        assert_eq!(get_sand_root_dir(), root);
        case(&root);
        fs::write(
            root.join("completed.json"),
            json!({
                "test": test_name,
                "pid": std::process::id(),
                "token": std::env::var(RETENTION_CHILD_TOKEN_ENV).expect("child invocation token"),
            }).to_string(),
        ).expect("write receipt only after all retention assertions");
        return;
    }

    // Set the production root only on an exact child invocation. Other tests
    // in this integration binary may run concurrently and keep their own root.
    let token = uuid::Uuid::new_v4().to_string();
    let root = std::env::temp_dir().join(format!("fabushi-local-exec-retention-{token}"));
    fs::create_dir(&root).expect("create private retention root");
    let stdout_path = root.join("child-stdout.txt");
    let stderr_path = root.join("child-stderr.txt");
    let mut child = RetentionChild {
        child: Command::new(std::env::current_exe().expect("integration test executable"))
            .args(["--exact", test_name, "--nocapture", "--test-threads=1"])
            .env(RETENTION_CHILD_TEST_ENV, test_name)
            .env(RETENTION_CHILD_TOKEN_ENV, &token)
            .env(SAND_DATA_ROOT_ENV, &root)
            .stdout(fs::File::create(&stdout_path).expect("owned child stdout"))
            .stderr(fs::File::create(&stderr_path).expect("owned child stderr"))
            .spawn()
            .expect("spawn exact retention contract child"),
        reaped: false,
    };
    let pid = child.child.id();
    let outcome = child.wait();
    if outcome.is_err() {
        child.stop();
    }
    let stdout = fs::read_to_string(&stdout_path).expect("read child stdout");
    let stderr = fs::read_to_string(&stderr_path).expect("read child stderr");
    let status = outcome.unwrap_or_else(|error| panic!(
        "{error}; reaped={}; retained artifacts: {}\n{stdout}\n{stderr}",
        child.reaped, root.display(),
    ));
    assert!(
        status.success(),
        "retention child failed: {status}; retained artifacts: {}\n{stdout}\n{stderr}",
        root.display(),
    );
    let receipt: Value = serde_json::from_slice(
        &fs::read(root.join("completed.json")).expect("child assertion completion receipt"),
    ).expect("completion receipt JSON");
    assert_eq!(receipt["test"], test_name, "an empty test selection cannot pass");
    assert_eq!(receipt["pid"].as_u64(), Some(u64::from(pid)));
    assert_eq!(receipt["token"], token);
    fs::remove_dir_all(&root).expect("remove only the reaped successful child's root");
}

type RetentionShell = thread::JoinHandle<Result<GatewayLocalExecShellResult, SandLocalExecError>>;

/// Real Host ownership, publication and transport use the existing gate fixture.
/// Provider replies are injected only after receiving each actual exec frame.
struct HostRetentionFixture {
    extension: HostLocalExecExtension,
    _registration: LocalExecProviderRegistration,
    receive: mpsc::Receiver<Value>,
    provider_id: String,
    requests: Mutex<HashSet<String>>,
    root: PathBuf,
}

impl HostRetentionFixture {
    fn new(root: &Path) -> Self {
        assert_eq!(get_sand_root_dir(), root);
        let telemetry = HostTelemetryService::open_with_identity_tags(
            root.join("retention-telemetry.jsonl"), Default::default(),
        ).expect("real Host telemetry");
        let extension = start_local_exec_extension(telemetry.logs.clone());
        let (send, receive) = mpsc::channel();
        let registration = extension.register_provider(send);
        let welcome = receive.recv_timeout(RETENTION_BOUND).expect("Host provider welcome");
        assert_eq!(welcome["kind"], "welcome");
        let provider_id = welcome["providerId"].as_str().expect("provider identity").to_string();
        extension.submit_responses(json!({
            "providerId": provider_id,
            "frames": [{
                "kind": "hello", "localRoot": root, "terminalsFolder": root.join("terminals"),
                "computerId": "lxr-computer", "label": "Retention contract",
            }],
        }));
        Self {
            extension,
            _registration: registration,
            receive,
            provider_id,
            requests: Mutex::new(HashSet::new()),
            root: root.to_path_buf(),
        }
    }

    fn facade(&self) -> GatewayLocalExecManager {
        // Independent permission gates and a cloned Host extension must still
        // share the same capacity, operation identities and journal transaction.
        self.extension.clone()
            .sandbox(Arc::new(FakeGate::default()))
            .exec_manager()
    }

    fn scope(operation_id: &str) -> GatewayLocalToolScope {
        GatewayLocalToolScope {
            agent_id: Some(RETENTION_AGENT.into()),
            tool_call_id: Some(operation_id.into()),
            action: Some("run-command".into()),
        }
    }

    fn start(&self, manager: &GatewayLocalExecManager, operation_id: &str) -> RetentionShell {
        let manager = manager.clone();
        let operation_id = operation_id.to_string();
        let cwd = self.root.to_string_lossy().into_owned();
        thread::spawn(move || manager.execute_shell_with_timeout(
            &Self::scope(&operation_id), "retention-contract", &cwd, &operation_id,
            Some(RETENTION_BOUND),
        ))
    }

    fn exec(&self, operation_id: &str) -> Value {
        let frame = self.receive.recv_timeout(RETENTION_BOUND).expect("actual provider exec");
        assert_eq!(frame["kind"], "exec");
        assert_eq!(frame["serverMessage"]["shellStreamArgs"]["toolCallId"], operation_id);
        assert_eq!(frame["serverMessage"]["shellStreamArgs"]["command"], "retention-contract");
        let request_id = frame["requestId"].as_str().expect("bridge request identity").to_string();
        assert_ne!(request_id, self.provider_id);
        assert!(
            self.requests.lock().expect("captured request identities").insert(request_id),
            "each delivered invocation must use a distinct real bridge request identity",
        );
        frame
    }

    fn reply(&self, exec: &Value, frames: Vec<Value>) {
        let frames = frames.into_iter().map(|mut frame| {
            frame["requestId"] = exec["requestId"].clone();
            frame
        }).collect::<Vec<_>>();
        self.extension.submit_responses(json!({
            "providerId": self.provider_id, "frames": frames,
        }));
    }

    fn finish(
        &self,
        exec: &Value,
        shell: RetentionShell,
    ) -> Result<GatewayLocalExecShellResult, SandLocalExecError> {
        let result = shell.join().expect("bounded shell worker");
        let cancel = self.receive.recv_timeout(RETENTION_BOUND).expect("request cleanup frame");
        assert_eq!(cancel["kind"], "cancel");
        assert_eq!(cancel["requestId"], exec["requestId"]);
        result
    }

    fn unknown(
        &self,
        manager: &GatewayLocalExecManager,
        operation_id: &str,
        stdout: &str,
        stderr: &str,
    ) -> GatewayLocalExecOperationRecord {
        let shell = self.start(manager, operation_id);
        let exec = self.exec(operation_id);
        self.reply(&exec, vec![
            json!({"kind":"client","message":{
                "id":exec["serverMessage"]["id"],"shellStream":{"stdout":{"data":stdout}},
            }}),
            json!({"kind":"client","message":{
                "id":exec["serverMessage"]["id"],"shellStream":{"stderr":{"data":stderr}},
            }}),
            json!({"kind":"control","message":{"streamClose":{"id":exec["serverMessage"]["id"]}}}),
        ]);
        let result = self.finish(&exec, shell).expect("no-Exit unknown result");
        assert!(result.outcome_unknown);
        assert_eq!(result.request_id, operation_id);
        assert_eq!(result.exit_code, None);
        assert!(!result.aborted);
        assert_eq!(result.stdout, stdout);
        assert_eq!(result.stderr, stderr);
        assert!(!result.stdout_truncated);
        assert!(!result.stderr_truncated);
        let record = manager.operation_for_agent(RETENTION_AGENT, operation_id)
            .expect("unknown record remains owned");
        assert_eq!(record.status, GatewayLocalExecOperationStatus::OutcomeUnknown);
        assert_eq!(record.operation_id, operation_id);
        assert_eq!(record.bridge_request_id, exec["requestId"].as_str().unwrap());
        assert_eq!(record.exit_code, None);
        assert!(!record.aborted);
        assert!(record.settled_at_ms.is_some());
        record
    }

    fn exit(&self, exec: &Value, aborted: bool) {
        self.reply(exec, vec![json!({"kind":"client","message":{
            "id":exec["serverMessage"]["id"],
            "shellStream":{"exit":{"code":if aborted { 130 } else { 0 },"cwd":self.root,"aborted":aborted}},
        }})]);
    }

    fn journal_path(&self) -> PathBuf {
        self.root.join(SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME)
    }

    fn journal_records(&self) -> Vec<GatewayLocalExecOperationRecord> {
        let journal: Value = serde_json::from_slice(
            &fs::read(self.journal_path()).expect("read real operation journal"),
        ).expect("operation journal JSON");
        assert_eq!(journal["version"], 3, "protected custody uses the version-3 writer");
        serde_json::from_value(journal["records"].clone()).expect("same private record shape")
    }

    fn assert_refused_without_dispatch(
        &self,
        manager: &GatewayLocalExecManager,
        operation_id: &str,
        cause: &str,
    ) {
        let history = manager.operation_history();
        let bytes = fs::read(self.journal_path()).expect("journal before refused admission");
        let error = manager.execute_shell_with_timeout(
            &Self::scope(operation_id), "must-not-replay", &self.root.to_string_lossy(),
            operation_id, Some(RETENTION_BOUND),
        ).expect_err("admission must refuse before any provider frame");
        assert!(!error.is_outcome_unknown(), "refused admission entered no effect");
        assert!(error.message.contains(cause), "unexpected admission error: {error}");
        assert_eq!(manager.operation_history(), history, "refusal cannot retire another operation");
        assert_eq!(fs::read(self.journal_path()).unwrap(), bytes, "refusal cannot rewrite custody");
        assert!(
            matches!(
                self.receive.recv_timeout(Duration::from_millis(25)),
                Err(mpsc::RecvTimeoutError::Timeout),
            ),
            "no exec or cancel frame may follow a refused admission",
        );
    }
}

#[test]
fn host_local_exec_unknown_survives_churn_and_restart_without_replay() {
    run_retention_contract(
        "host_local_exec_unknown_survives_churn_and_restart_without_replay",
        |root| {
            let operation_id = "lxr-original-unknown";
            let retained = {
                let fixture = HostRetentionFixture::new(root);
                let first = fixture.facade();
                let second = fixture.facade();
                assert!(first.operation_history().is_empty());
                let output = "界".repeat(MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES / 3 + 1);
                let retained = fixture.unknown(
                    &first, operation_id, &output, "original stderr remains queryable",
                );
                assert_eq!(retained.stdout, "界".repeat(MAX_LOCAL_EXEC_DURABLE_OUTPUT_BYTES / 3));
                assert!(retained.stdout_truncated);
                assert_eq!(retained.stderr, "original stderr remains queryable");
                assert!(!retained.stderr_truncated);
                assert_eq!(second.operation(operation_id), Some(retained.clone()));

                // More than the production capacity of known results used to
                // evict this older unknown. Exercise all three eligible states.
                let churn = DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT + 2;
                for index in 0..churn {
                    let known_id = format!("lxr-known-{index}");
                    let manager = if index % 2 == 0 { &first } else { &second };
                    let shell = fixture.start(manager, &known_id);
                    let exec = fixture.exec(&known_id);
                    let status = if index % 3 == 2 {
                        fixture.reply(&exec, vec![json!({"kind":"control","message":{"throw":{
                            "error":"controlled pre-entry refusal",
                            "errorCode":"LOCAL_EXEC_PRE_ENTRY_REFUSED",
                        }}})]);
                        let error = fixture.finish(&exec, shell).expect_err("known refusal control");
                        assert!(!error.is_outcome_unknown());
                        assert!(error.message.contains("controlled pre-entry refusal"));
                        GatewayLocalExecOperationStatus::Failed
                    } else {
                        let aborted = index % 3 == 1;
                        fixture.exit(&exec, aborted);
                        let result = fixture.finish(&exec, shell).expect("observed Exit control");
                        assert!(!result.outcome_unknown);
                        assert_eq!(result.aborted, aborted);
                        assert_eq!(result.exit_code, Some(if aborted { 130 } else { 0 }));
                        if aborted {
                            GatewayLocalExecOperationStatus::Cancelled
                        } else {
                            GatewayLocalExecOperationStatus::Completed
                        }
                    };
                    assert_eq!(manager.operation(&known_id).unwrap().status, status);
                }

                let history = first.operation_history();
                assert_eq!(history.len(), DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT);
                assert_eq!(history.first(), Some(&retained));
                assert_eq!(history[1].operation_id, "lxr-known-3", "evict oldest eligible known history");
                for index in 0..3 {
                    assert!(first.operation(&format!("lxr-known-{index}")).is_none());
                }
                assert_eq!(second.operation_history(), history, "independent facades share retention");
                assert_eq!(fixture.journal_records(), history);
                assert_eq!(fixture.requests.lock().unwrap().len(), churn + 1);
                assert_eq!(first.operation_for_agent(RETENTION_AGENT, operation_id), Some(retained.clone()));
                assert!(second.operation_for_agent("other-agent", operation_id).is_none());
                assert!(second.operation_history_for_agent("other-agent").is_empty());
                retained
            };

            // Every original facade and its Host owner is gone. This owner must
            // recover the identical unresolved identity/output from real disk.
            let reopened = HostRetentionFixture::new(root);
            let fresh = reopened.facade();
            assert_eq!(fresh.operation_for_agent(RETENTION_AGENT, operation_id), Some(retained.clone()));
            assert_eq!(fresh.operation_history().first(), Some(&retained));
            reopened.assert_refused_without_dispatch(&fresh, operation_id, "already exists");
            assert_eq!(fresh.operation(operation_id), Some(retained));
            assert!(reopened.requests.lock().unwrap().is_empty());
        },
    );
}

#[test]
fn host_local_exec_shared_capacity_refuses_dispatch_until_known_terminal() {
    run_retention_contract(
        "host_local_exec_shared_capacity_refuses_dispatch_until_known_terminal",
        |root| {
            let protected = {
                let fixture = HostRetentionFixture::new(root);
                let first = fixture.facade();
                let second = fixture.facade();
                assert!(first.operation_history().is_empty());
                let mut protected = Vec::new();
                for index in 0..(DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT - 1) {
                    let operation_id = format!("lxr-capacity-unknown-{index}");
                    let output = format!("observed output {index}");
                    let record = fixture.unknown(&first, &operation_id, &output, "uncertain stderr");
                    assert_eq!(record.stdout, output);
                    assert_eq!(record.stderr, "uncertain stderr");
                    assert!(!record.stdout_truncated);
                    assert!(!record.stderr_truncated);
                    protected.push(record);
                }
                assert_eq!(second.operation_history(), protected);

                let active_id = "lxr-final-reserved-place";
                let shell = fixture.start(&first, active_id);
                let exec = fixture.exec(active_id);
                let active = first.operation(active_id).expect("active reservation");
                assert_eq!(active.status, GatewayLocalExecOperationStatus::Accepted);
                assert_eq!(second.operation(active_id), Some(active.clone()));
                let snapshot = fixture.journal_records();
                assert_eq!(snapshot.len(), DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT);
                assert_eq!(snapshot.iter().find(|record| record.operation_id == active_id), Some(&active));
                fixture.assert_refused_without_dispatch(&second, "lxr-active-capacity-refusal", "capacity");
                assert_eq!(first.operation(active_id), Some(active), "refusal cannot settle the active owner");
                assert_eq!(first.operation_history(), protected);
                assert!(second.operation("lxr-active-capacity-refusal").is_none());

                // Only a positively observed Exit makes the reserved place
                // eligible for reuse. All preceding unknowns remain protected.
                fixture.exit(&exec, false);
                let result = fixture.finish(&exec, shell).expect("known terminal releases capacity");
                assert!(!result.outcome_unknown);
                assert_eq!(result.exit_code, Some(0));
                assert_eq!(first.operation(active_id).unwrap().status, GatewayLocalExecOperationStatus::Completed);
                let replacement = fixture.unknown(
                    &second, "lxr-reused-known-place", "replacement output", "replacement stderr",
                );
                assert_eq!(replacement.stdout, "replacement output");
                assert_eq!(replacement.stderr, "replacement stderr");
                protected.push(replacement);
                assert!(first.operation(active_id).is_none(), "only the known terminal was evicted");
                assert_eq!(protected.len(), DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT);
                assert_eq!(first.operation_history(), protected);
                assert_eq!(second.operation_history(), protected);
                assert_eq!(fixture.journal_records(), protected);
                fixture.assert_refused_without_dispatch(&first, "lxr-all-unknown-refusal", "capacity");
                assert!(second.operation("lxr-all-unknown-refusal").is_none());
                assert_eq!(fixture.requests.lock().unwrap().len(), DEFAULT_LOCAL_EXEC_OPERATION_HISTORY_LIMIT + 1);
                protected
            };

            let reopened = HostRetentionFixture::new(root);
            let fresh = reopened.facade();
            assert_eq!(fresh.operation_history(), protected, "all unknown records survive a fresh owner");
            reopened.assert_refused_without_dispatch(&fresh, "lxr-reopened-capacity-refusal", "capacity");
            assert!(fresh.operation("lxr-reopened-capacity-refusal").is_none());
            assert_eq!(reopened.journal_records(), protected);
            assert!(reopened.requests.lock().unwrap().is_empty());
        },
    );
}
