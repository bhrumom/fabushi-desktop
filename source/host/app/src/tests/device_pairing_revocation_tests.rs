use super::super::{project_openclaw_node_list, project_openclaw_node_status};
use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::extensions::node_host::runtime_state::NodeHostRuntimeState;
use mahayana_host_runtime::extensions::node_host::session_registry::NodeHostSessionMetadata;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

// These are the actual helpers called by UnifiedGatewayApi, with its actual
// durable pairing and node-runtime owners. No copied projection or remote
// transport fixture stands in for a shipping App/device acceptance result.
struct NodeProjectionFixture {
    root: PathBuf,
    owner: Option<DevicePairingOwner>,
    runtime: NodeHostRuntimeState,
}

impl NodeProjectionFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("dvr-app-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("create owned node projection fixture");
        let mut fixture = Self { root, owner: None, runtime: NodeHostRuntimeState::new() };
        fixture.owner = Some(DevicePairingOwner::new(fixture.root.join("pairings.sqlite"))
            .expect("open real App pairing owner"));
        let local = fixture.runtime.connect(metadata("sand-local-node", true));
        assert!(fixture.runtime.set_active(&local, true));
        fixture
    }

    fn owner(&self) -> &DevicePairingOwner {
        self.owner.as_ref().expect("fixture owner open")
    }

    fn pair(&self, id: &str, roles: &[&str], scopes: &[&str]) -> DeviceTokenAuthorization {
        let request = self.owner().handle_pair_request(&json!({
            "deviceId": id, "publicKey": format!("projection-key-{id}"),
            "displayName": format!("paired {id}"), "platform": "projection-platform",
            "role": roles[0], "roles": roles, "scopes": scopes,
        })).expect("actual pairing request");
        self.owner().handle_pair_approve(&json!({"requestId": request["requestId"]}))
            .expect("actual pairing approval");
        let issued = self.owner().handle_pair_rotate_token(&json!({
            "deviceId": id, "role": roles[0],
        })).expect("actual credential issuance");
        self.owner().authenticate_device_token(issued["token"].as_str().expect("issued token"))
            .expect("authenticate issued credential").expect("issued credential accepted")
    }

    fn list(&self) -> Value {
        project_openclaw_node_list(self.owner(), &self.runtime).expect("shipping node list helper")
    }

    fn status(&self, id: &str) -> Value {
        project_openclaw_node_status(self.owner(), &self.runtime, &json!({"nodeId": id}))
            .expect("shipping node status helper")
    }

    fn absent(&self, id: &str) {
        assert!(!self.list()["nodes"].as_array().expect("nodes")
            .iter().any(|node| node["nodeId"] == id));
        let error = project_openclaw_node_status(
            self.owner(), &self.runtime, &json!({"nodeId": id}),
        ).expect_err("removed stored node must be unknown to the shipping helper");
        assert_eq!(error.status(), 400);
        assert_eq!(error.to_string(), format!("unknown nodeId: {id}"));
    }

    fn local_is_live(&self) {
        let list = self.list();
        assert_eq!(list["activeNodeId"], "sand-local-node");
        let local = self.status("sand-local-node");
        assert_eq!(local["connected"], true);
        assert_eq!(local["paired"], true);
        assert_eq!(local["active"], true);
        assert_eq!(local["gatewayLocal"], true);
        assert_eq!(local["commands"], json!(["system.run"]));
        assert_eq!(list["nodes"].as_array().expect("nodes").iter()
            .filter(|node| node["nodeId"] == "sand-local-node").count(), 1);
    }

    fn reopen(&mut self) {
        drop(self.owner.take());
        self.owner = Some(DevicePairingOwner::new(self.root.join("pairings.sqlite"))
            .expect("reopen the same durable App owner"));
    }
}

impl Drop for NodeProjectionFixture {
    fn drop(&mut self) {
        drop(self.owner.take());
        if !std::thread::panicking() {
            fs::remove_dir_all(&self.root).expect("remove only owned fixture after owner close");
        }
    }
}

fn metadata(id: &str, local: bool) -> NodeHostSessionMetadata {
    NodeHostSessionMetadata {
        node_id: id.into(), display_name: format!("session {id}"),
        platform: "runtime-platform".into(), device_family: "fixture".into(),
        commands: vec!["system.run".into()], capabilities: vec!["exec".into()],
        paired: true, gateway_local: local,
    }
}

#[test]
fn revocation_shipping_node_projection_removes_stored_remote_and_preserves_local_after_reopen() {
    let mut fixture = NodeProjectionFixture::new();
    let admin = fixture.pair("projection-admin", &["operator"], &["operator.admin"]);
    fixture.pair("projection-remote", &["node"], &["node.read"]);
    fixture.pair("projection-survivor", &["node"], &["node.read"]);
    let remote = fixture.status("projection-remote");
    assert_eq!(remote["paired"], true);
    assert_eq!(remote["connected"], false);
    assert_eq!(remote["commands"], json!([]));
    fixture.local_is_live();
    let before = fixture.owner().current_revision();
    let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": "projection-remote"}), &admin, None,
    ).expect("real bound admin revocation");
    assert_eq!(receipt["revision"], before + 1);
    fixture.absent("projection-remote");
    assert_eq!(fixture.status("projection-survivor")["paired"], true);
    fixture.local_is_live();
    fixture.reopen();
    assert_eq!(fixture.owner().current_revision(), before + 1);
    fixture.absent("projection-remote");
    assert_eq!(fixture.status("projection-survivor")["paired"], true);
    fixture.local_is_live();
}

#[test]
fn revocation_shipping_projection_ignores_disconnected_history_and_preserves_denied_pairing() {
    let mut fixture = NodeProjectionFixture::new();
    let writer = fixture.pair("projection-mixed", &["operator", "node"], &["operator.write"]);
    let historical = fixture.runtime.connect(metadata("projection-mixed", false));
    assert!(fixture.runtime.set_active(&historical, true));
    assert!(fixture.runtime.disconnect(&historical).is_empty());
    let history = fixture.runtime.get("projection-mixed").expect("retained disconnected history");
    assert!(!history.connected);
    let before = fixture.list();
    let revision = fixture.owner().current_revision();
    let error = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": "projection-mixed", "role": "operator"}), &writer, None,
    ).expect_err("ordinary writer cannot delete mixed authority");
    assert_eq!(error.status(), 403);
    assert_eq!(fixture.owner().current_revision(), revision);
    assert_eq!(fixture.list()["nodes"], before["nodes"]);
    assert_eq!(fixture.status("projection-mixed")["displayName"], "paired projection-mixed");
    assert_eq!(fixture.status("projection-mixed")["commands"], json!([]));
    fixture.owner().handle_pair_revoke(&json!({"deviceId": "projection-mixed"}))
        .expect("native whole pairing removal");
    fixture.absent("projection-mixed");
    assert_eq!(fixture.runtime.get("projection-mixed"), Some(history));
    fixture.local_is_live();
    fixture.reopen();
    fixture.absent("projection-mixed");
    fixture.local_is_live();
}

fn shipping_body<'a>(source: &'a str, start: &str, end: &str) -> Option<&'a str> {
    source.split_once(start)?.1.split_once(end).map(|(body, _)| body)
}

fn revocation_shipping_bindings(source: &str) -> bool {
    let Some(api) = source.split_once("impl GatewayApi for UnifiedGatewayApi {").map(|(_, api)| api)
        else { return false; };
    let Some(revoke) = shipping_body(api, "    fn revoke_paired_device(\n", "    fn validate_paired_device_authorization(\n")
        else { return false; };
    let Some(retained) = shipping_body(api, "    fn validate_paired_device_authorization(\n", "    fn authorize_device_token_request(\n")
        else { return false; };
    let Some(list) = shipping_body(source, "    fn call_openclaw_node_list(\n", "    fn call_openclaw_node_status(\n")
        else { return false; };
    let Some(status) = shipping_body(source, "    fn call_openclaw_node_status(\n", "    fn call_openclaw_node_invoke(\n")
        else { return false; };
    revoke.contains("self.device_pairing.revoke_pairing_for_authenticated_device(\n            &args, authorization, scope_ceiling,")
        && revoke.matches(".note_desktop_contact(").count() == 1
        && !revoke.contains("call_host_lane")
        && retained.contains("self.device_pairing.validate_current_device_authorization(authorization)")
        && !retained.contains("note_desktop_contact")
        && !retained.contains("call_host_lane")
        && list.contains("project_openclaw_node_list(self.device_pairing.as_ref(), self.node_host_runtime.as_ref())")
        && status.contains("project_openclaw_node_status(self.device_pairing.as_ref(), self.node_host_runtime.as_ref(), args)")
        && source.contains("\"node.list\" => self.call_openclaw_node_list(&args)")
        && source.contains("\"node.status\" => self.call_openclaw_node_status(&args)")
}

#[test]
fn revocation_shipping_hooks_and_node_consumers_use_the_same_existing_owners() {
    let shipping = include_str!("../main.rs")
        .split_once("\n#[cfg(test)]\nmod tests {").expect("shipping/test boundary").0;
    assert!(revocation_shipping_bindings(shipping));
    // The source binding oracle supplements the two real helper tests and App
    // compilation; each changed owner/dead delegation must make it fail.
    for (from, to) in [
        ("self.device_pairing.revoke_pairing_for_authenticated_device(", "discarded_revoke("),
        ("self.device_pairing.validate_current_device_authorization(authorization)", "Ok(())"),
        ("project_openclaw_node_list(self.device_pairing.as_ref(), self.node_host_runtime.as_ref())", "Ok(serde_json::json!({}))"),
        ("project_openclaw_node_status(self.device_pairing.as_ref(), self.node_host_runtime.as_ref(), args)", "Ok(serde_json::json!({}))"),
    ] {
        assert!(shipping.contains(from));
        assert!(!revocation_shipping_bindings(&shipping.replacen(from, to, 1)));
    }
}
