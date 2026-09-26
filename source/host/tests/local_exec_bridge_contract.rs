use std::sync::{
    Arc, Mutex,
    atomic::{AtomicU64, Ordering},
    mpsc,
};
use std::time::Duration;

use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::local_exec::extension::{
    LOCAL_EXEC_DEPENDENCIES, local_exec_extension_id,
};
use mahayana_host_runtime::extensions::local_exec::local_exec_bridge::{
    DEFAULT_SAND_COMPUTER_ID, SAND_LOCAL_EXEC_LIVENESS_WINDOW_MS,
    SandLocalExecBridge, bounded_local_exec_variant, local_exec_provider_rank,
};
use serde_json::json;

#[test]
fn provider_rank_and_variant_projection_match_frozen_grok() {
    assert_eq!(local_exec_provider_rank(true, Some("sand")), 6);
    assert_eq!(local_exec_provider_rank(true, Some("sand-lab")), 5);
    assert_eq!(local_exec_provider_rank(false, Some("sand")), 2);
    assert_eq!(local_exec_provider_rank(false, Some("sand-dev")), 0);
    assert_eq!(bounded_local_exec_variant(Some("sand")), Some("sand"));
    assert_eq!(bounded_local_exec_variant(Some("sand-lab")), Some("sand-lab"));
    assert_eq!(bounded_local_exec_variant(Some("sand-dev")), Some("sand-dev"));
    assert_eq!(bounded_local_exec_variant(Some("custom")), Some("unknown"));
    assert_eq!(bounded_local_exec_variant(None), None);
    assert_eq!(local_exec_extension_id(), HostExtensionId::LocalExec);
    assert_eq!(
        LOCAL_EXEC_DEPENDENCIES,
        &[HostExtensionId::LocalToolPermission, HostExtensionId::Telemetry]
    );
}

#[test]
fn provider_registration_hello_liveness_and_selection_follow_frozen_transport() {
    let now = Arc::new(AtomicU64::new(1000));
    let now_for_bridge = Arc::clone(&now);
    let ids = Arc::new(Mutex::new(vec![
        "provider-b".to_string(),
        "provider-a".to_string(),
    ]));
    let ids_for_bridge = Arc::clone(&ids);
    let bridge = SandLocalExecBridge::with_sources(
        Arc::new(move || now_for_bridge.load(Ordering::SeqCst)),
        Arc::new(move || ids_for_bridge.lock().expect("ids").pop().expect("id")),
    );

    let (send, receive) = mpsc::channel();
    let registration = bridge.register_provider(send);
    assert_eq!(
        receive.recv_timeout(Duration::from_millis(100)).expect("welcome"),
        json!({"kind":"welcome","providerId":"provider-a"})
    );
    assert!(bridge.has_provider());
    assert!(bridge.ever_registered());

    bridge.submit_responses(json!({
        "providerId": "provider-a",
        "frames": [{
            "kind": "hello",
            "localRoot": "/Users/test",
            "terminalsFolder": "terminals",
            "computerId": "mac-1",
            "label": "My Mac",
            "supervised": true,
            "variant": "sand"
        }]
    }));
    assert_eq!(
        bridge.active_computer().expect("active").id,
        "mac-1"
    );
    assert_eq!(
        bridge.get_provider_info().expect("info").terminals_folder,
        "terminals"
    );

    bridge.submit_responses(json!({
        "providerId": "provider-a",
        "frames": [{"kind":"ping","supervised":true}]
    }));
    now.store(1000 + SAND_LOCAL_EXEC_LIVENESS_WINDOW_MS + 1, Ordering::SeqCst);
    assert!(!bridge.is_computer_live("mac-1"));
    assert!(!bridge.check_live_computer_for_ask());

    drop(registration);
    assert!(!bridge.has_provider());
}

#[test]
fn request_response_and_retire_approval_flow_through_registered_provider() {
    let bridge = SandLocalExecBridge::with_sources(
        Arc::new(|| 5000),
        Arc::new({
            let next = Arc::new(AtomicU64::new(1));
            move || format!("id-{}", next.fetch_add(1, Ordering::SeqCst))
        }),
    );
    let (send, receive) = mpsc::channel();
    let _registration = bridge.register_provider(send);
    let _ = receive.recv_timeout(Duration::from_millis(100)).expect("welcome");

    bridge.submit_responses(json!({
        "frames": [{
            "kind": "hello",
            "localRoot": "/tmp",
            "terminalsFolder": "terminals"
        }]
    }));
    assert_eq!(
        bridge.active_computer().expect("active").id,
        DEFAULT_SAND_COMPUTER_ID
    );

    let mut request = bridge.request(json!({"kind":"download","path":"/tmp/a"}), None)
        .expect("request");
    let outbound = receive.recv_timeout(Duration::from_millis(100)).expect("request frame");
    let request_id = outbound.get("requestId").and_then(|value| value.as_str()).expect("request id");
    assert_eq!(outbound["kind"], "download");

    bridge.submit_responses(json!({
        "frames": [{
            "kind": "file",
            "requestId": request_id,
            "bytesBase64": "YQ=="
        }]
    }));
    assert_eq!(
        request.recv_timeout(Duration::from_millis(100)).expect("response")["kind"],
        "file"
    );
    request.close();
    assert_eq!(
        receive.recv_timeout(Duration::from_millis(100)).expect("cancel")["kind"],
        "cancel"
    );

    bridge.retire_approval("approval-1");
    let retired = receive.recv_timeout(Duration::from_millis(100)).expect("retire");
    assert_eq!(retired["kind"], "retire-approval");
    assert_eq!(retired["approvalId"], "approval-1");
}
