use mahayana_host_runtime::device_pairing_owner::DevicePairingOwner;
use serde_json::json;

#[test]
fn test_ocrm_pair_o01_request_success() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let res = owner.handle_pairing_command(
        "device.pair.request",
        &json!({
            "deviceId": "dev-alpha-001",
            "publicKey": "base64-pubkey-sample",
            "displayName": "Alpha Device",
            "platform": "macos"
        }),
    ).expect("pair request call");

    assert_eq!(res["status"], "pending");
    assert!(res["requestId"].as_str().unwrap().starts_with("pair-"));
    assert_eq!(res["deviceId"], "dev-alpha-001");
    assert_eq!(res["created"], true);
    assert_eq!(res["isRepair"], false);
    assert!(res["expiresAtMs"].as_i64().unwrap() > 0);
}

#[test]
fn test_ocrm_pair_o02_request_requires_device_id_and_public_key() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let err1 = owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "publicKey": "some-key" }),
    ).unwrap_err();
    assert_eq!(err1.status(), 400);

    let err2 = owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "deviceId": "dev-01" }),
    ).unwrap_err();
    assert_eq!(err2.status(), 400);
}

#[test]
fn test_ocrm_pair_o03_and_o04_pair_status() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let req = owner.handle_pairing_command(
        "device.pair.request",
        &json!({
            "deviceId": "dev-status-test",
            "publicKey": "pubkey-status-test"
        }),
    ).expect("pair request");
    let request_id = req["requestId"].as_str().unwrap();

    // Valid pending request
    let status_pending = owner.handle_pairing_command(
        "device.pair.status",
        &json!({ "requestId": request_id }),
    ).expect("status call");
    assert_eq!(status_pending["status"], "pending");
    assert_eq!(status_pending["requestId"], request_id);

    // Unknown request -> rejected
    let status_unknown = owner.handle_pairing_command(
        "device.pair.status",
        &json!({ "requestId": "non-existent-req-id" }),
    ).expect("status call");
    assert_eq!(status_unknown["status"], "rejected");
}

#[test]
fn test_ocrm_pair_o05_and_o06_pair_approve() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let initial_rev = owner.current_revision();

    let req = owner.handle_pairing_command(
        "device.pair.request",
        &json!({
            "deviceId": "dev-approve-test",
            "publicKey": "pubkey-approve-test",
            "displayName": "Approval Test Device"
        }),
    ).expect("pair request");
    let request_id = req["requestId"].as_str().unwrap();

    // Unknown requestId rejection (O06)
    let err_unknown = owner.handle_pairing_command(
        "device.pair.approve",
        &json!({ "requestId": "pair-unknown-999" }),
    ).unwrap_err();
    assert_eq!(err_unknown.status(), 400);

    // Successful approval (O05)
    let approve_res = owner.handle_pairing_command(
        "device.pair.approve",
        &json!({ "requestId": request_id }),
    ).expect("approve call");

    assert_eq!(approve_res["decision"], "approved");
    assert_eq!(approve_res["deviceId"], "dev-approve-test");
    let new_rev = approve_res["revision"].as_u64().unwrap();
    assert!(new_rev > initial_rev);
    assert_eq!(owner.current_revision(), new_rev);

    // Verify status transitions to approved
    let status = owner.handle_pairing_command(
        "device.pair.status",
        &json!({ "deviceId": "dev-approve-test" }),
    ).expect("status check");
    assert_eq!(status["status"], "approved");
    assert_eq!(status["device"]["deviceId"], "dev-approve-test");
}

#[test]
fn test_ocrm_pair_o07_pair_reject() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let req = owner.handle_pairing_command(
        "device.pair.request",
        &json!({
            "deviceId": "dev-reject-test",
            "publicKey": "pubkey-reject-test"
        }),
    ).expect("pair request");
    let request_id = req["requestId"].as_str().unwrap();

    let rej_res = owner.handle_pairing_command(
        "device.pair.reject",
        &json!({ "requestId": request_id }),
    ).expect("reject call");
    assert_eq!(rej_res["decision"], "rejected");
    assert_eq!(rej_res["deviceId"], "dev-reject-test");

    // Status after rejection returns rejected
    let status = owner.handle_pairing_command(
        "device.pair.status",
        &json!({ "requestId": request_id }),
    ).expect("status check");
    assert_eq!(status["status"], "rejected");
}

#[test]
fn test_ocrm_pair_o08_pair_list() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");

    let req1 = owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "deviceId": "dev-list-pending", "publicKey": "pk-1" }),
    ).unwrap();
    let req2 = owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "deviceId": "dev-list-paired", "publicKey": "pk-2" }),
    ).unwrap();

    owner.handle_pairing_command(
        "device.pair.approve",
        &json!({ "requestId": req2["requestId"].as_str().unwrap() }),
    ).unwrap();

    let list_res = owner.handle_pairing_command("device.pair.list", &json!({})).unwrap();
    let pending = list_res["pending"].as_array().unwrap();
    let paired = list_res["paired"].as_array().unwrap();

    assert_eq!(pending.len(), 1);
    assert_eq!(pending[0]["deviceId"], "dev-list-pending");
    assert_eq!(paired.len(), 1);
    assert_eq!(paired[0]["deviceId"], "dev-list-paired");
}

#[test]
fn test_ocrm_pair_o09_and_o10_pair_revoke() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");

    // Unknown device revoke rejects (O10)
    let err_unknown = owner.handle_pairing_command(
        "device.pair.revoke",
        &json!({ "deviceId": "dev-ghost" }),
    ).unwrap_err();
    assert_eq!(err_unknown.status(), 400);

    // Create & approve device
    let req = owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "deviceId": "dev-to-revoke", "publicKey": "pk-rev" }),
    ).unwrap();
    owner.handle_pairing_command(
        "device.pair.approve",
        &json!({ "requestId": req["requestId"].as_str().unwrap() }),
    ).unwrap();
    let rev_before = owner.current_revision();

    // Revoke device (O09)
    let revoke_res = owner.handle_pairing_command(
        "device.pair.revoke",
        &json!({ "deviceId": "dev-to-revoke" }),
    ).unwrap();
    assert_eq!(revoke_res["deviceId"], "dev-to-revoke");
    let new_rev = revoke_res["revision"].as_u64().unwrap();
    assert!(new_rev > rev_before);

    // Subsequent status check returns rejected
    let status = owner.handle_pairing_command(
        "device.pair.status",
        &json!({ "deviceId": "dev-to-revoke" }),
    ).unwrap();
    assert_eq!(status["status"], "rejected");
}

#[test]
fn test_ocrm_pair_o11_rotate_token_in_band_and_cross_device() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");

    let req = owner.handle_pairing_command(
        "device.pair.request",
        &json!({ "deviceId": "dev-rotate-target", "publicKey": "pk-rot" }),
    ).unwrap();
    owner.handle_pairing_command(
        "device.pair.approve",
        &json!({ "requestId": req["requestId"].as_str().unwrap() }),
    ).unwrap();

    // In-band rotation: returns token directly
    let rot_in_band = owner.handle_pairing_command(
        "device.pair.rotateToken",
        &json!({ "deviceId": "dev-rotate-target", "crossDevice": false }),
    ).unwrap();
    assert_eq!(rot_in_band["tokenDelivery"], "in-band");
    assert!(rot_in_band["token"].as_str().unwrap().starts_with("sand-device-tok-"));

    // Cross-device rotation: withholds token in-band
    let rot_cross = owner.handle_pairing_command(
        "device.pair.rotateToken",
        &json!({ "deviceId": "dev-rotate-target", "crossDevice": true }),
    ).unwrap();
    assert_eq!(rot_cross["tokenDelivery"], "withheld-cross-device");
    assert!(rot_cross.get("token").is_none());
}

#[test]
fn test_ocrm_pair_o12_setup_code() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let res = owner.handle_pairing_command("device.pair.setupCode", &json!({})).unwrap();

    assert!(res["setupId"].as_str().unwrap().starts_with("setup-"));
    assert!(res["setupCode"].as_str().unwrap().starts_with("sand-setup-"));
    assert_eq!(res["auth"], "token");
    assert_eq!(res["access"], "full");
    assert!(res["expiresAtMs"].as_i64().unwrap() > 0);
}
