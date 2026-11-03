use super::device_pairing_revocation_tests::{
    NodeProjectionFixture, metadata, node_surfaces, role_token, seed_node_surfaces,
    status_without_timestamp,
};
use serde_json::json;

// Reuse the actual App projection helpers and durable-owner fixture. These
// assertions cover native App consumers; they do not stand in for a physical
// node, external WebSocket client or credential-cache compatibility result.
#[test]
fn token_rotation_shipping_operator_replacement_preserves_node_and_local_after_reopen() {
    let mut fixture = NodeProjectionFixture::new();
    let operator = fixture.pair(
        "dtr-app-mixed", &["operator", "node"], &["operator.pairing", "node.read"],
    );
    fixture.pair("dtr-app-survivor", &["node"], &["node.read"]);
    let old_operator = role_token(&fixture, "dtr-app-mixed", "operator");
    let old_node = role_token(&fixture, "dtr-app-mixed", "node");
    let node = fixture.owner().authenticate_device_token(&old_node)
        .expect("node lookup").expect("real current node proof");
    seed_node_surfaces(&fixture, "dtr-app-mixed");
    let surfaces = node_surfaces(&fixture, "dtr-app-mixed");
    let before = status_without_timestamp(&fixture, "dtr-app-mixed");
    let survivor = status_without_timestamp(&fixture, "dtr-app-survivor");
    let revision = fixture.owner().current_revision();
    let receipt = fixture.owner().rotate_token_with_authority(
        &json!({"deviceId": " dtr-app-mixed ", "role": " operator ",
            "scopes": ["operator.pairing"]}), Some(&operator), Some("operator.pairing"),
    ).expect("real bound self rotation");
    assert_eq!(receipt.as_object().expect("rotation receipt").len(), 6);
    assert_eq!(receipt["deviceId"], "dtr-app-mixed");
    assert_eq!(receipt["role"], "operator");
    assert_eq!(receipt["scopes"], json!(["operator.pairing"]));
    assert_eq!(receipt["tokenDelivery"], "in-band");
    assert!(receipt["rotatedAtMs"].as_i64().is_some_and(|time| time > 0));
    let replacement = receipt["token"].as_str().expect("self replacement").to_owned();
    assert!(!replacement.is_empty() && replacement != old_operator);
    assert!(receipt.get("revision").is_none());
    assert_eq!(fixture.owner().current_revision(), revision + 1);
    assert!(fixture.owner().authenticate_device_token(&old_operator)
        .expect("old credential lookup").is_none());
    let successor = fixture.owner().authenticate_device_token(&replacement)
        .expect("successor lookup").expect("committed successor is current");
    fixture.owner().validate_current_device_authorization(&successor)
        .expect("successor sealed proof is usable");
    fixture.owner().validate_current_device_authorization(&node)
        .expect("other role remains current");
    assert_eq!(node_surfaces(&fixture, "dtr-app-mixed"), surfaces);
    assert_eq!(status_without_timestamp(&fixture, "dtr-app-mixed"), before);
    assert_eq!(status_without_timestamp(&fixture, "dtr-app-survivor"), survivor);
    fixture.local_is_live();
    fixture.reopen();
    assert_eq!(fixture.owner().current_revision(), revision + 1);
    assert!(fixture.owner().authenticate_device_token(&old_operator)
        .expect("durable predecessor lookup").is_none());
    assert!(fixture.owner().authenticate_device_token(&replacement)
        .expect("durable successor lookup").is_some());
    assert!(fixture.owner().authenticate_device_token(&old_node)
        .expect("durable other-role lookup").is_some());
    assert_eq!(node_surfaces(&fixture, "dtr-app-mixed"), surfaces);
    assert_eq!(status_without_timestamp(&fixture, "dtr-app-mixed"), before);
    assert_eq!(status_without_timestamp(&fixture, "dtr-app-survivor"), survivor);
    fixture.local_is_live();
}

#[test]
fn token_rotation_shipping_node_replacement_withholds_secret_and_preserves_pairing_after_reopen() {
    let mut fixture = NodeProjectionFixture::new();
    let admin = fixture.pair("dtr-app-admin", &["operator"], &["operator.admin"]);
    let operator = fixture.pair(
        "dtr-app-node", &["operator", "node"], &["operator.write", "node.read"],
    );
    let operator_token = role_token(&fixture, "dtr-app-node", "operator");
    let old_node = role_token(&fixture, "dtr-app-node", "node");
    let historical = fixture.runtime.connect(metadata("dtr-app-node", false));
    assert!(fixture.runtime.disconnect(&historical).is_empty());
    let history = fixture.runtime.get("dtr-app-node").expect("disconnected runtime history");
    seed_node_surfaces(&fixture, "dtr-app-node");
    let surfaces = node_surfaces(&fixture, "dtr-app-node");
    assert!(surfaces.0.is_some() && surfaces.1.is_some());
    let before = status_without_timestamp(&fixture, "dtr-app-node");
    let revision = fixture.owner().current_revision();
    let receipt = fixture.owner().rotate_token_with_authority(
        &json!({"deviceId": "dtr-app-node", "role": "node", "scopes": ["node.read"]}),
        Some(&admin), None,
    ).expect("real cross-device node rotation");
    assert_eq!(receipt.as_object().expect("withheld receipt").len(), 5);
    assert_eq!(receipt["deviceId"], "dtr-app-node");
    assert_eq!(receipt["role"], "node");
    assert_eq!(receipt["scopes"], json!(["node.read"]));
    assert_eq!(receipt["tokenDelivery"], "withheld-cross-device");
    assert!(receipt["rotatedAtMs"].as_i64().is_some_and(|time| time > 0));
    assert!(receipt.get("token").is_none());
    assert!(receipt.get("revision").is_none());
    let successor = role_token(&fixture, "dtr-app-node", "node");
    assert!(!successor.is_empty() && successor != old_node);
    assert!(!receipt.to_string().contains(&successor));
    assert_eq!(fixture.owner().current_revision(), revision + 1);
    assert_eq!(node_surfaces(&fixture, "dtr-app-node"), (None, None));
    assert!(fixture.owner().authenticate_device_token(&old_node)
        .expect("old node lookup").is_none());
    assert!(fixture.owner().authenticate_device_token(&successor)
        .expect("new node lookup").is_some());
    fixture.owner().validate_current_device_authorization(&operator)
        .expect("other role survives node rotation");
    assert_eq!(status_without_timestamp(&fixture, "dtr-app-node"), before);
    assert_eq!(fixture.runtime.get("dtr-app-node"), Some(history.clone()));
    fixture.local_is_live();
    // Generic trusted-native App dispatch reaches this same owner and still
    // withholds the new bearer; it does not inherit paired self identity.
    let native = fixture.owner().handle_pairing_command("device.token.rotate", &json!({
        "deviceId": "dtr-app-node", "role": "node", "scopes": ["node.read"],
    })).expect("native owner dispatch");
    assert_eq!(native["tokenDelivery"], "withheld-cross-device");
    assert!(native.get("token").is_none());
    assert_eq!(fixture.owner().current_revision(), revision + 2);
    assert!(fixture.owner().authenticate_device_token(&successor)
        .expect("intermediate node lookup").is_none());
    let final_token = role_token(&fixture, "dtr-app-node", "node");
    fixture.reopen();
    assert_eq!(fixture.owner().current_revision(), revision + 2);
    assert!(fixture.owner().authenticate_device_token(&old_node)
        .expect("durable old node lookup").is_none());
    assert!(fixture.owner().authenticate_device_token(&final_token)
        .expect("durable final node lookup").is_some());
    assert!(fixture.owner().authenticate_device_token(&operator_token)
        .expect("durable operator lookup").is_some());
    assert_eq!(node_surfaces(&fixture, "dtr-app-node"), (None, None));
    assert_eq!(status_without_timestamp(&fixture, "dtr-app-node"), before);
    assert_eq!(fixture.runtime.get("dtr-app-node"), Some(history));
    fixture.local_is_live();
}

fn rotation_shipping_bindings(source: &str) -> bool {
    let Some((_, api)) = source.split_once("impl GatewayApi for UnifiedGatewayApi {")
        else { return false; };
    let Some((_, hook)) = api.split_once("    fn rotate_device_token(\n")
        else { return false; };
    let Some((hook, _)) = hook.split_once("    fn revoke_device_token(\n")
        else { return false; };
    let Some((_, dispatch)) = source.split_once("    fn call_openclaw_gateway_command(\n")
        else { return false; };
    let Some((dispatch, _)) = dispatch.split_once("    fn call_public_settings_command(\n")
        else { return false; };
    hook.contains("authorization: Option<&mahayana_host_runtime::device_pairing_owner::DeviceTokenAuthorization>")
        && hook.contains("self.device_pairing.rotate_token_with_authority(&args, authorization, scope_ceiling)")
        && hook.matches(".note_desktop_contact(").count() == 1
        && !hook.contains("call_host_lane")
        && !hook.contains("rotate_token_for_authenticated_device")
        && dispatch.contains("| \"device.token.rotate\"")
        && dispatch.contains("| \"device.pair.setupCode\" => self.device_pairing.handle_pairing_command(method, &args)")
}

#[test]
fn token_rotation_shipping_hook_keeps_sealed_identity_native_ceiling_and_existing_owner() {
    let source = include_str!("../main.rs")
        .split_once("\n#[cfg(test)]\nmod tests {").expect("shipping/test boundary").0;
    assert!(rotation_shipping_bindings(source));
    for (from, to) in [
        ("self.device_pairing.rotate_token_with_authority(&args, authorization, scope_ceiling)",
            "self.device_pairing.rotate_token_with_authority(&args, None, scope_ceiling)"),
        ("self.device_pairing.rotate_token_with_authority(&args, authorization, scope_ceiling)",
            "self.device_pairing.rotate_token_with_authority(&args, authorization, None)"),
        ("self.device_pairing.rotate_token_with_authority(&args, authorization, scope_ceiling)",
            "discarded_rotation(&args)"),
        ("| \"device.token.rotate\"", "| \"unregistered.token.rotate\""),
        ("| \"device.pair.setupCode\" => self.device_pairing.handle_pairing_command(method, &args)",
            "| \"device.pair.setupCode\" => Ok(serde_json::json!({}))"),
    ] {
        assert!(source.contains(from));
        assert!(!rotation_shipping_bindings(&source.replacen(from, to, 1)));
    }
}
