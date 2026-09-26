use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::Duration;

use mahayana_host_runtime::extensions::local_exec::gateway_local_exec_sand_box::{
    DEFAULT_MAX_LOCAL_EXEC_FILE_BYTES, GatewayLocalExecSandBox, GatewayLocalToolGate,
    create_bridge_user_computers, local_exec_file_too_large_message,
};
use mahayana_host_runtime::extensions::local_exec::local_exec_bridge::SandLocalExecBridge;
use mahayana_host_runtime::extensions::local_exec::local_exec_error::SandLocalExecError;
use serde_json::json;

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
        agent_id: Option<&str>,
        action: &str,
        target: &str,
    ) -> Result<Option<String>, SandLocalExecError> {
        self.calls.lock().expect("calls").push((
            agent_id.map(str::to_string),
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
    let bridge = SandLocalExecBridge::with_sources(
        Arc::new(|| 1000),
        Arc::new(|| "provider-1".to_string()),
    );
    let (send, receive) = mpsc::channel();
    let registration = bridge.register_provider(send);
    let _ = receive.recv_timeout(Duration::from_millis(100)).expect("welcome");
    bridge.submit_responses(json!({
        "providerId": "provider-1",
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
    let upload_frame = receive.recv_timeout(Duration::from_secs(1)).expect("upload frame");
    assert_eq!(upload_frame["kind"], "upload");
    assert_eq!(upload_frame["path"], "/tmp/a.txt");
    assert_eq!(upload_frame["bytesBase64"], "aGVsbG8=");
    assert_eq!(upload_frame["approvalId"], "approval-1");
    bridge.submit_responses(json!({
        "frames": [{
            "kind": "file",
            "requestId": upload_frame["requestId"]
        }]
    }));
    upload.join().expect("upload thread").expect("upload");
    // The frozen bridge sends a terminal cancel from request cleanup even
    // after a successful file response. Consume and verify that lifecycle
    // frame before the next independent request.
    let upload_cancel = receive
        .recv_timeout(Duration::from_secs(1))
        .expect("upload cleanup cancel");
    assert_eq!(upload_cancel["kind"], "cancel");
    assert_eq!(upload_cancel["requestId"], upload_frame["requestId"]);

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
