use mahayana_host_runtime::extensions::cross_user_sharing::extension::*;
use mahayana_host_runtime::extensions::cross_user_sharing::xuser_entry_publisher::*;
use mahayana_host_runtime::extensions::cross_user_sharing::xuser_relay::*;
use mahayana_host_runtime::extensions::cross_user_sharing::xuser_sharing_service::*;
use mahayana_host_runtime::extensions::cross_user_sharing::xuser_state_reconcile::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

struct FakeTransport {
    requests: Mutex<Vec<String>>,
    value: serde_json::Value,
    status: u16,
}
impl XuserRelayTransport for FakeTransport {
    fn request(
        &self,
        _: &str,
        url: &str,
        _: &str,
        _: Option<&serde_json::Value>,
    ) -> Result<serde_json::Value, SandXuserRelayHttpError> {
        self.requests.lock().unwrap().push(url.into());
        if self.status >= 400 {
            Err(SandXuserRelayHttpError {
                status: self.status,
                path: url.into(),
            })
        } else {
            Ok(self.value.clone())
        }
    }
}
fn service(value: serde_json::Value) -> Arc<SandXuserSharingService> {
    let t = Arc::new(FakeTransport {
        requests: Mutex::new(vec![]),
        value,
        status: 200,
    });
    let relay = Arc::new(SandXuserRelayClient::new(
        "https://example.invalid".into(),
        Arc::new(|| Ok("token".into())),
        t,
    ));
    Arc::new(SandXuserSharingService::new(relay))
}
#[test]
fn relay_errors_keep_frozen_user_messages() {
    assert_eq!(
        describe_relay_error(&SandXuserRelayHttpError {
            status: 403,
            path: "x".into()
        }),
        "Sharing isn't enabled for your account."
    );
    assert!(
        describe_relay_error(&SandXuserRelayHttpError {
            status: 429,
            path: "x".into()
        })
        .contains("too often")
    );
}
#[test]
fn remote_ids_and_publish_payload_are_fail_closed() {
    assert_eq!(
        parse_remote_agent_id("sand-remote:owner/agent"),
        Some(("owner".into(), "agent".into()))
    );
    assert!(parse_remote_agent_id("local").is_none());
    assert!(
        build_publish_payload(
            &serde_json::json!({"id":"1","kind":"message","content":""}),
            "me"
        )
        .is_none()
    );
    assert_eq!(
        build_publish_payload(
            &serde_json::json!({"id":"1","kind":"message","content":"hi"}),
            "me"
        )
        .unwrap()["text"],
        "hi"
    );
}
#[test]
fn reconcile_reports_added_and_revoked_rooms() {
    let old = vec![XuserRoom {
        room_id: "old".into(),
        name: "Old".into(),
        host_auth_id: "me".into(),
        members: vec![],
        avatar_data_url: None,
    }];
    let new = vec![XuserRoom {
        room_id: "new".into(),
        name: "New".into(),
        host_auth_id: "me".into(),
        members: vec![],
        avatar_data_url: None,
    }];
    let r = reconcile_rooms(&old, new, Some("me"));
    assert_eq!(r.revoked_room_ids, vec!["old"]);
    assert_eq!(r.added_room_ids, vec!["new"]);
}
#[test]
fn extension_applies_dev_environment_gate_before_feature_gate() {
    let svc = service(serde_json::json!({"rooms":[],"pendingJoinRequests":[]}));
    let ext = CrossUserSharingExtension::new(
        Arc::clone(&svc),
        "https://api2.cursor.sh",
        &BTreeMap::new(),
    );
    ext.set_feature_gate(true);
    assert!(!ext.is_enabled());
    assert!(ext.reason().is_some());
    let mut env = BTreeMap::new();
    env.insert("SAND_DEV_XUSER_SHARING".into(), "1".into());
    env.insert("SAND_XUSER_SHARING_ALLOW_PROD".into(), "1".into());
    let ext = CrossUserSharingExtension::new(svc, "https://api2.cursor.sh", &env);
    ext.set_feature_gate(true);
    assert!(ext.is_enabled());
}
#[test]
fn enabled_service_reconciles_backend_state() {
    let svc = service(
        serde_json::json!({"pendingJoinRequests":[{"id":"j"}],"rooms":[{"roomId":"r","name":"Room","hostAuthId":"me","members":[]}]}),
    );
    svc.set_enabled(true);
    svc.set_self_auth_id(Some("me".into()));
    let s = svc.reconcile_share_state().unwrap();
    assert_eq!(s.rooms.len(), 1);
    assert_eq!(s.pending_join_requests.len(), 1);
}
