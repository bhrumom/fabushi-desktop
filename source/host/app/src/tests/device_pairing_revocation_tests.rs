use super::super::{project_openclaw_node_list, project_openclaw_node_status};
use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::extensions::node_host::runtime_state::NodeHostRuntimeState;
use mahayana_host_runtime::extensions::node_host::session_registry::NodeHostSessionMetadata;
use serde_json::{Value, json};
use std::{fs, path::PathBuf};

// These are the actual helpers called by UnifiedGatewayApi, with its actual
// durable pairing and node-runtime owners. No copied projection or remote
// transport fixture stands in for a shipping App/device acceptance result.
pub(super) struct NodeProjectionFixture {
    root: PathBuf,
    owner: Option<DevicePairingOwner>,
    pub(super) runtime: NodeHostRuntimeState,
}

impl NodeProjectionFixture {
    pub(super) fn new() -> Self {
        let root = std::env::temp_dir().join(format!("dvr-app-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("create owned node projection fixture");
        let mut fixture = Self { root, owner: None, runtime: NodeHostRuntimeState::new() };
        fixture.owner = Some(DevicePairingOwner::new(fixture.root.join("pairings.sqlite"))
            .expect("open real App pairing owner"));
        let local = fixture.runtime.connect(metadata("sand-local-node", true));
        assert!(fixture.runtime.set_active(&local, true));
        fixture
    }

    pub(super) fn owner(&self) -> &DevicePairingOwner {
        self.owner.as_ref().expect("fixture owner open")
    }

    pub(super) fn pair(&self, id: &str, roles: &[&str], scopes: &[&str]) -> DeviceTokenAuthorization {
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

    pub(super) fn status(&self, id: &str) -> Value {
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

    pub(super) fn local_is_live(&self) {
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

    pub(super) fn reopen(&mut self) {
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

pub(super) fn metadata(id: &str, local: bool) -> NodeHostSessionMetadata {
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

pub(super) fn role_token(fixture: &NodeProjectionFixture, id: &str, role: &str) -> String {
    let conn = rusqlite::Connection::open_with_flags(
        fixture.root.join("pairings.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ).expect("independent read-only credential observer");
    let stored: String = conn.query_row(
        "SELECT tokens_json FROM device_pairing_paired WHERE device_id = ?1",
        [id], |row| row.get(0),
    ).expect("observe the existing paired row");
    conn.close().expect("close independent observer");
    let entries: Value = serde_json::from_str(&stored).expect("valid fixture token entries");
    entries[role]["token"].as_str().expect("real issued role credential").to_owned()
}

pub(super) fn node_surfaces(fixture: &NodeProjectionFixture, id: &str) -> (Option<String>, Option<String>) {
    let conn = rusqlite::Connection::open_with_flags(
        fixture.root.join("pairings.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY,
    ).expect("independent read-only surface observer");
    let surfaces = conn.query_row(
        "SELECT node_surface_json, pending_node_surface_json FROM device_pairing_paired
         WHERE device_id = ?1", [id], |row| Ok((row.get(0)?, row.get(1)?)),
    ).expect("the pairing remains present");
    conn.close().expect("close independent surface observer");
    surfaces
}

pub(super) fn seed_node_surfaces(fixture: &NodeProjectionFixture, id: &str) {
    let conn = rusqlite::Connection::open_with_flags(
        fixture.root.join("pairings.sqlite"), rusqlite::OpenFlags::SQLITE_OPEN_READ_WRITE,
    ).expect("open existing fixture for explicit surface setup");
    assert_eq!(conn.execute(
        "UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2
         WHERE device_id = ?3",
        rusqlite::params!["{ \"generation\" : 7, \"commands\" : [\"system.run\"] }",
            "{ \"generation\" : 8, \"commands\" : [\"system.notify\"] }", id],
    ).expect("seed distinct raw surface bytes after issuance"), 1);
    conn.close().expect("close surface fixture writer");
}

pub(super) fn status_without_timestamp(fixture: &NodeProjectionFixture, id: &str) -> Value {
    let mut status = fixture.status(id);
    assert!(status.as_object_mut().expect("node object").remove("ts").is_some());
    status
}

#[test]
fn token_revocation_shipping_operator_retirement_preserves_node_and_local_projection_after_reopen() {
    let mut fixture = NodeProjectionFixture::new();
    let operator = fixture.pair("drt-app-mixed", &["operator", "node"], &["operator.pairing", "node.read"]);
    fixture.pair("drt-app-survivor", &["node"], &["node.read"]);
    let retired = role_token(&fixture, "drt-app-mixed", "operator");
    let node_token = role_token(&fixture, "drt-app-mixed", "node");
    let node = fixture.owner().authenticate_device_token(&node_token)
        .expect("authenticate node").expect("live node proof");
    seed_node_surfaces(&fixture, "drt-app-mixed");
    let surfaces = node_surfaces(&fixture, "drt-app-mixed");
    let before = status_without_timestamp(&fixture, "drt-app-mixed");
    let survivor = status_without_timestamp(&fixture, "drt-app-survivor");
    assert_eq!(before["paired"], true);
    assert_eq!(before["connected"], false);
    assert_eq!(before["active"], false);
    let revision = fixture.owner().current_revision();
    let receipt = fixture.owner().revoke_token_with_authority(
        &json!({"deviceId": "drt-app-mixed", "role": "operator"}),
        Some(&operator), Some("operator.pairing"),
    ).expect("real bound selected operator retirement");
    assert_eq!(receipt.as_object().expect("committed receipt").len(), 3);
    assert_eq!(receipt["deviceId"], "drt-app-mixed");
    assert_eq!(receipt["role"], "operator");
    assert!(receipt["revokedAtMs"].as_i64().is_some_and(|time| time > 0));
    assert!(!receipt.to_string().contains(&retired));
    assert_eq!(fixture.owner().current_revision(), revision + 1);
    assert!(fixture.owner().authenticate_device_token(&retired).expect("retired lookup").is_none());
    fixture.owner().validate_current_device_authorization(&node).expect("other role remains current");
    assert_eq!(node_surfaces(&fixture, "drt-app-mixed"), surfaces);
    assert_eq!(status_without_timestamp(&fixture, "drt-app-mixed"), before);
    assert_eq!(status_without_timestamp(&fixture, "drt-app-survivor"), survivor);
    fixture.local_is_live();
    fixture.reopen();
    assert_eq!(fixture.owner().current_revision(), revision + 1);
    assert!(fixture.owner().authenticate_device_token(&retired).expect("durable retired lookup").is_none());
    assert!(fixture.owner().authenticate_device_token(&node_token).expect("durable live lookup").is_some());
    assert_eq!(node_surfaces(&fixture, "drt-app-mixed"), surfaces);
    assert_eq!(status_without_timestamp(&fixture, "drt-app-mixed"), before);
    assert_eq!(status_without_timestamp(&fixture, "drt-app-survivor"), survivor);
    fixture.local_is_live();
}

#[test]
fn token_revocation_shipping_node_retirement_keeps_pairing_history_and_other_role_after_reopen() {
    let mut fixture = NodeProjectionFixture::new();
    let admin = fixture.pair("drt-app-admin", &["operator"], &["operator.admin"]);
    let operator = fixture.pair("drt-app-node", &["operator", "node"], &["operator.write", "node.read"]);
    let operator_token = role_token(&fixture, "drt-app-node", "operator");
    let node_token = role_token(&fixture, "drt-app-node", "node");
    let historical = fixture.runtime.connect(metadata("drt-app-node", false));
    assert!(fixture.runtime.disconnect(&historical).is_empty());
    let history = fixture.runtime.get("drt-app-node").expect("disconnected transport history");
    seed_node_surfaces(&fixture, "drt-app-node");
    let surfaces = node_surfaces(&fixture, "drt-app-node");
    assert!(surfaces.0.is_some() && surfaces.1.is_some());
    let before = status_without_timestamp(&fixture, "drt-app-node");
    assert_eq!(before["displayName"], "paired drt-app-node");
    assert_eq!(before["paired"], true);
    assert_eq!(before["connected"], false);
    assert_eq!(before["active"], false);
    let revision = fixture.owner().current_revision();
    fixture.owner().revoke_token_with_authority(
        &json!({"deviceId": "drt-app-node", "role": "node"}), Some(&admin), None,
    ).expect("admin retires just the node role");
    assert_eq!(fixture.owner().current_revision(), revision + 1);
    assert_eq!(node_surfaces(&fixture, "drt-app-node"), (None, None));
    assert!(fixture.owner().authenticate_device_token(&node_token).expect("retired node lookup").is_none());
    fixture.owner().validate_current_device_authorization(&operator).expect("operator authority preserved");
    assert_eq!(status_without_timestamp(&fixture, "drt-app-node"), before);
    assert_eq!(fixture.runtime.get("drt-app-node"), Some(history.clone()));
    fixture.local_is_live();
    // Trusted native dispatch repeats the same selected mutation without
    // reviving either surface or removing the paired row.
    fixture.owner().handle_pairing_command("device.token.revoke", &json!({
        "deviceId": "drt-app-node", "role": "node",
    })).expect("native dispatch reaches the same owner");
    assert_eq!(fixture.owner().current_revision(), revision + 2);
    fixture.reopen();
    assert_eq!(fixture.owner().current_revision(), revision + 2);
    assert_eq!(node_surfaces(&fixture, "drt-app-node"), (None, None));
    assert!(fixture.owner().authenticate_device_token(&node_token).expect("durable retired node").is_none());
    assert!(fixture.owner().authenticate_device_token(&operator_token).expect("durable operator").is_some());
    assert_eq!(status_without_timestamp(&fixture, "drt-app-node"), before);
    assert_eq!(fixture.runtime.get("drt-app-node"), Some(history));
    fixture.local_is_live();
}

fn token_revocation_shipping_bindings(source: &str) -> bool {
    let Some(api) = source.split_once("impl GatewayApi for UnifiedGatewayApi {").map(|(_, api)| api)
        else { return false; };
    let Some(revoke) = shipping_body(api, "    fn revoke_device_token(\n", "    fn redeem_device_pairing_join_code(\n")
        else { return false; };
    let Some(dispatch) = shipping_body(source, "    fn call_openclaw_gateway_command(\n", "    fn call_public_settings_command(\n")
        else { return false; };
    revoke.contains("authorization: Option<&mahayana_host_runtime::device_pairing_owner::DeviceTokenAuthorization>")
        && revoke.contains("self.device_pairing.revoke_token_with_authority(&args, authorization, scope_ceiling)")
        && revoke.matches(".note_desktop_contact(").count() == 1
        && !revoke.contains("call_host_lane")
        && dispatch.contains("| \"device.token.revoke\"")
        && dispatch.contains("| \"device.pair.setupCode\" => self.device_pairing.handle_pairing_command(method, &args)")
        && revocation_shipping_bindings(source)
}

#[test]
fn token_revocation_shipping_hook_keeps_paired_proof_native_ceiling_and_existing_owner() {
    let shipping = include_str!("../main.rs")
        .split_once("\n#[cfg(test)]\nmod tests {").expect("shipping/test boundary").0;
    assert!(token_revocation_shipping_bindings(shipping));
    for (from, to) in [
        ("self.device_pairing.revoke_token_with_authority(&args, authorization, scope_ceiling)",
            "self.device_pairing.revoke_token_with_authority(&args, None, scope_ceiling)"),
        ("self.device_pairing.revoke_token_with_authority(&args, authorization, scope_ceiling)",
            "self.device_pairing.revoke_token_with_authority(&args, authorization, None)"),
        ("self.device_pairing.revoke_token_with_authority(&args, authorization, scope_ceiling)",
            "discarded_role_revocation(&args)"),
        ("| \"device.token.revoke\"", "| \"unregistered.token.revoke\""),
        ("| \"device.pair.setupCode\" => self.device_pairing.handle_pairing_command(method, &args)",
            "| \"device.pair.setupCode\" => Ok(serde_json::json!({}))"),
    ] {
        assert!(shipping.contains(from));
        assert!(!token_revocation_shipping_bindings(&shipping.replacen(from, to, 1)));
    }
}
