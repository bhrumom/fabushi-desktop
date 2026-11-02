use base64::Engine as _;
use mahayana_host_runtime::device_pairing_owner::DevicePairingOwner;
use serde_json::json;

#[test]
fn test_ocrm_pair_o01_request_success() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let res = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-alpha-001",
                "publicKey": "base64-pubkey-sample",
                "displayName": "Alpha Device",
                "platform": "macos"
            }),
        )
        .expect("pair request call");

    assert_eq!(res["status"], "pending");
    assert!(res["requestId"].as_str().unwrap().starts_with("pair-"));
    assert_eq!(res["deviceId"], "dev-alpha-001");
    assert_eq!(res["created"], true);
    assert_eq!(res["isRepair"], false);
    assert!(res["expiresAtMs"].as_i64().unwrap() > 0);
}

#[test]
fn test_ocrm_pair_request_reuses_compatible_pending_identity() {
    let owner = DevicePairingOwner::in_memory().expect("pairing owner");
    let first = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-idempotent",
                "publicKey": "pubkey-idempotent",
                "browserOrigin": "https://browser-a.example",
                "roles": ["operator"],
                "scopes": ["operator.read", "operator.write"]
            }),
        )
        .unwrap();
    let first_request_id = first["requestId"].as_str().unwrap().to_string();

    let refreshed = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-idempotent",
                "publicKey": "pubkey-idempotent",
                "browserOrigin": "https://browser-a.example",
                "roles": ["operator"],
                "scopes": ["operator.read"]
            }),
        )
        .unwrap();
    assert_eq!(refreshed["requestId"], first_request_id);
    assert_eq!(refreshed["created"], false);
    assert!(refreshed["refreshedAtMs"].as_i64().unwrap() > 0);

    let status = owner
        .handle_pairing_command(
            "device.pair.status",
            &json!({ "requestId": first_request_id }),
        )
        .unwrap();
    assert_eq!(status["status"], "pending");

    let superseding = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-idempotent",
                "publicKey": "pubkey-idempotent",
                "browserOrigin": "https://browser-b.example",
                "roles": ["operator"],
                "scopes": ["operator.read"]
            }),
        )
        .unwrap();
    assert_ne!(superseding["requestId"], first_request_id);
    let superseded_status = owner
        .handle_pairing_command(
            "device.pair.status",
            &json!({ "requestId": first_request_id }),
        )
        .unwrap();
    assert_eq!(superseded_status["status"], "rejected");
}

#[test]
fn test_ocrm_pair_o02_request_requires_device_id_and_public_key() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let err1 = owner
        .handle_pairing_command("device.pair.request", &json!({ "publicKey": "some-key" }))
        .unwrap_err();
    assert_eq!(err1.status(), 400);

    let err2 = owner
        .handle_pairing_command("device.pair.request", &json!({ "deviceId": "dev-01" }))
        .unwrap_err();
    assert_eq!(err2.status(), 400);
}

#[test]
fn test_ocrm_pair_o03_and_o04_pair_status() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let req = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-status-test",
                "publicKey": "pubkey-status-test"
            }),
        )
        .expect("pair request");
    let request_id = req["requestId"].as_str().unwrap();

    // Valid pending request
    let status_pending = owner
        .handle_pairing_command("device.pair.status", &json!({ "requestId": request_id }))
        .expect("status call");
    assert_eq!(status_pending["status"], "pending");
    assert_eq!(status_pending["requestId"], request_id);

    // Unknown request -> rejected
    let status_unknown = owner
        .handle_pairing_command(
            "device.pair.status",
            &json!({ "requestId": "non-existent-req-id" }),
        )
        .expect("status call");
    assert_eq!(status_unknown["status"], "rejected");
}

#[test]
fn test_ocrm_pair_o05_and_o06_pair_approve() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let initial_rev = owner.current_revision();

    let req = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-approve-test",
                "publicKey": "pubkey-approve-test",
                "displayName": "Approval Test Device"
            }),
        )
        .expect("pair request");
    let request_id = req["requestId"].as_str().unwrap();

    // Unknown requestId rejection (O06)
    let err_unknown = owner
        .handle_pairing_command(
            "device.pair.approve",
            &json!({ "requestId": "pair-unknown-999" }),
        )
        .unwrap_err();
    assert_eq!(err_unknown.status(), 400);

    // Successful approval (O05)
    let approve_res = owner
        .handle_pairing_command("device.pair.approve", &json!({ "requestId": request_id }))
        .expect("approve call");

    assert_eq!(approve_res["decision"], "approved");
    assert_eq!(approve_res["deviceId"], "dev-approve-test");
    let new_rev = approve_res["revision"].as_u64().unwrap();
    assert!(new_rev > initial_rev);
    assert_eq!(owner.current_revision(), new_rev);

    // Verify status transitions to approved
    let status = owner
        .handle_pairing_command(
            "device.pair.status",
            &json!({ "deviceId": "dev-approve-test" }),
        )
        .expect("status check");
    assert_eq!(status["status"], "approved");
    assert_eq!(status["device"]["deviceId"], "dev-approve-test");
}

#[test]
fn test_ocrm_pair_o07_pair_reject() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let req = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-reject-test",
                "publicKey": "pubkey-reject-test"
            }),
        )
        .expect("pair request");
    let request_id = req["requestId"].as_str().unwrap();

    let rej_res = owner
        .handle_pairing_command("device.pair.reject", &json!({ "requestId": request_id }))
        .expect("reject call");
    assert_eq!(rej_res["decision"], "rejected");
    assert_eq!(rej_res["deviceId"], "dev-reject-test");

    // Status after rejection returns rejected
    let status = owner
        .handle_pairing_command("device.pair.status", &json!({ "requestId": request_id }))
        .expect("status check");
    assert_eq!(status["status"], "rejected");
}

#[test]
fn test_ocrm_pair_o08_pair_list() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");

    let _req1 = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({ "deviceId": "dev-list-pending", "publicKey": "pk-1" }),
        )
        .unwrap();
    let req2 = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({ "deviceId": "dev-list-paired", "publicKey": "pk-2" }),
        )
        .unwrap();

    owner
        .handle_pairing_command(
            "device.pair.approve",
            &json!({ "requestId": req2["requestId"].as_str().unwrap() }),
        )
        .unwrap();

    let list_res = owner
        .handle_pairing_command("device.pair.list", &json!({}))
        .unwrap();
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
    let err_unknown = owner
        .handle_pairing_command("device.pair.revoke", &json!({ "deviceId": "dev-ghost" }))
        .unwrap_err();
    assert_eq!(err_unknown.status(), 400);

    // Create & approve device
    let req = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({ "deviceId": "dev-to-revoke", "publicKey": "pk-rev" }),
        )
        .unwrap();
    owner
        .handle_pairing_command(
            "device.pair.approve",
            &json!({ "requestId": req["requestId"].as_str().unwrap() }),
        )
        .unwrap();
    let rotated = owner
        .handle_pairing_command(
            "device.pair.rotateToken",
            &json!({ "deviceId": "dev-to-revoke" }),
        )
        .unwrap();
    let token = rotated["token"].as_str().unwrap().to_string();
    assert!(owner.authenticate_device_token(&token).unwrap().is_some());
    let rev_before = owner.current_revision();

    // Revoke device (O09)
    let revoke_res = owner
        .handle_pairing_command(
            "device.pair.revoke",
            &json!({ "deviceId": "dev-to-revoke" }),
        )
        .unwrap();
    assert_eq!(revoke_res["deviceId"], "dev-to-revoke");
    let new_rev = revoke_res["revision"].as_u64().unwrap();
    assert!(new_rev > rev_before);

    // Subsequent status check returns rejected
    let status = owner
        .handle_pairing_command(
            "device.pair.status",
            &json!({ "deviceId": "dev-to-revoke" }),
        )
        .unwrap();
    assert_eq!(status["status"], "rejected");
    assert!(owner.authenticate_device_token(&token).unwrap().is_none());
}

#[test]
fn test_ocrm_pair_o11_rotate_token_in_band_and_cross_device() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");

    let req = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({ "deviceId": "dev-rotate-target", "publicKey": "pk-rot" }),
        )
        .unwrap();
    owner
        .handle_pairing_command(
            "device.pair.approve",
            &json!({ "requestId": req["requestId"].as_str().unwrap() }),
        )
        .unwrap();

    // In-band rotation: returns token directly
    let rot_in_band = owner
        .handle_pairing_command(
            "device.pair.rotateToken",
            &json!({ "deviceId": "dev-rotate-target", "crossDevice": false }),
        )
        .unwrap();
    assert_eq!(rot_in_band["tokenDelivery"], "in-band");
    let first_token = rot_in_band["token"].as_str().unwrap().to_string();
    assert!(owner
        .authenticate_device_token(&first_token)
        .unwrap()
        .is_some());
    assert!(
        rot_in_band["token"]
            .as_str()
            .unwrap()
            .starts_with("sand-device-tok-")
    );
    let token_body = rot_in_band["token"]
        .as_str()
        .unwrap()
        .strip_prefix("sand-device-tok-")
        .unwrap();
    assert_eq!(
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(token_body)
            .unwrap()
            .len(),
        32
    );

    // Cross-device rotation: withholds token in-band
    let rot_cross = owner
        .handle_pairing_command(
            "device.pair.rotateToken",
            &json!({ "deviceId": "dev-rotate-target", "crossDevice": true }),
        )
        .unwrap();
    assert_eq!(rot_cross["tokenDelivery"], "withheld-cross-device");
    assert!(rot_cross.get("token").is_none());
    assert!(owner
        .authenticate_device_token(&first_token)
        .unwrap()
        .is_none());

    let rejected_role = owner.handle_pairing_command(
        "device.pair.rotateToken",
        &json!({ "deviceId": "dev-rotate-target", "role": "node" }),
    );
    assert!(rejected_role.is_err());

    let multi_role_request = owner
        .handle_pairing_command(
            "device.pair.request",
            &json!({
                "deviceId": "dev-multi-role",
                "publicKey": "pk-multi-role",
                "roles": ["node", "operator"]
            }),
        )
        .unwrap();
    owner
        .handle_pairing_command(
            "device.pair.approve",
            &json!({ "requestId": multi_role_request["requestId"] }),
        )
        .unwrap();
    let operator_rotation = owner
        .handle_pairing_command(
            "device.pair.rotateToken",
            &json!({ "deviceId": "dev-multi-role", "role": "operator" }),
        )
        .unwrap();
    let operator_token = operator_rotation["token"].as_str().unwrap().to_string();
    let node_rotation = owner
        .handle_pairing_command(
            "device.pair.rotateToken",
            &json!({ "deviceId": "dev-multi-role", "role": "node" }),
        )
        .unwrap();
    assert_eq!(node_rotation["role"], "node");
    assert!(owner.authenticate_device_token(&operator_token).unwrap().is_some());
}

#[test]
fn test_ocrm_pair_o12_setup_code() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let res = owner
        .handle_pairing_command("device.pair.setupCode", &json!({}))
        .unwrap();

    assert!(res["setupId"].as_str().unwrap().starts_with("setup-"));
    assert!(
        res["setupCode"]
            .as_str()
            .unwrap()
            .starts_with("sand-setup-")
    );
    assert_eq!(res["auth"], "token");
    assert_eq!(res["access"], "full");
    assert!(res["expiresAtMs"].as_i64().unwrap() > 0);
    assert!(res.get("gatewayUrl").is_none());

    let configured = owner
        .handle_pair_setup_code(&json!({ "gatewayUrl": "wss://gateway.example.test" }))
        .unwrap();
    assert_eq!(configured["gatewayUrl"], "wss://gateway.example.test");

    let rejected = owner.handle_pair_setup_code(
        &json!({ "gatewayUrl": "ws://gateway.example.test" }),
    );
    assert!(rejected.is_err());
}

#[test]
fn test_ocrm_pair_o12_setup_code_is_expiring_and_single_use() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let issued = owner
        .handle_pairing_command("device.pair.setupCode", &json!({}))
        .expect("issue setup code");
    let setup_code = issued["setupCode"].as_str().expect("setup code");

    let consumed = owner
        .consume_bootstrap_token(setup_code)
        .expect("consume setup code")
        .expect("live setup code");
    assert_eq!(consumed["setupId"], issued["setupId"]);
    assert_eq!(consumed["expiresAtMs"], issued["expiresAtMs"]);
    assert!(consumed["expiresAtMs"].as_i64().unwrap() > consumed["issuedAtMs"].as_i64().unwrap());

    assert!(owner
        .consume_bootstrap_token(setup_code)
        .expect("second consume")
        .is_none());
}

#[test]
fn test_ocrm_pair_o13_join_code_validation_and_path_parsing() {
    use mahayana_host_runtime::device_pairing_owner::*;

    // Valid 22-char base64url codes
    assert!(is_device_pairing_join_code("A1b2C3d4E5f6G7h8I9j0_-"));
    assert!(is_device_pairing_join_code("abcdefghijklmnopqrstuv"));

    // Invalid length or chars
    assert!(!is_device_pairing_join_code("too-short"));
    assert!(!is_device_pairing_join_code("A1b2C3d4E5f6G7h8I9j0_+=")); // + and = are standard base64, not url
    assert!(!is_device_pairing_join_code(
        "this-code-is-longer-than-22-chars"
    ));

    // Path parsing
    assert_eq!(
        parse_device_pairing_join_request_path("/gateway/j"),
        Some("")
    );
    assert_eq!(parse_device_pairing_join_request_path("/j"), Some(""));
    assert_eq!(
        parse_device_pairing_join_request_path("/proxy/j/app/j/code123"),
        Some("code123")
    );
    assert_eq!(
        parse_device_pairing_join_request_path("/gateway/j/shortcode"),
        Some("shortcode")
    );
    assert_eq!(parse_device_pairing_join_request_path("/api/status"), None);
}

#[test]
fn test_ocrm_pair_o14_safe_keys_and_labels() {
    use mahayana_host_runtime::device_pairing_owner::*;

    // Key sanitization
    assert_eq!(safe_channel_key("Telegram").unwrap(), "telegram");
    assert_eq!(
        safe_channel_key("slack/team:work*channel").unwrap(),
        "slack_team_work_channel"
    );
    assert_eq!(safe_account_key("user..profile").unwrap(), "user_profile");

    // Empty and sanitized-empty keys rejected
    assert!(safe_channel_key("   ").is_err());
    assert!(safe_channel_key("..").is_err());
    assert!(safe_account_key("/").is_err());

    // ID labels
    assert_eq!(resolve_pairing_id_label(None), "userId");
    assert_eq!(resolve_pairing_id_label(Some("telegramId")), "telegramId");
}

#[test]
fn test_ocrm_pair_o15_pairing_reply_formatting() {
    use mahayana_host_runtime::device_pairing_owner::*;

    let reply = build_pairing_reply("discord", "Discord User: @alice#1234", "PAIR-9876");
    assert!(reply.contains("OpenClaw: access not configured."));
    assert!(reply.contains("Discord User: @alice#1234"));
    assert!(reply.contains("PAIR-9876"));
    assert!(reply.contains("openclaw pairing approve discord PAIR-9876"));
}

#[test]
fn test_ocrm_pair_o16_mobile_pairing_security_policy() {
    use mahayana_host_runtime::device_pairing_owner::*;

    // Secure WSS and HTTPS always allowed
    assert!(validate_mobile_pairing_url("wss://gateway.example.com/ws").is_ok());
    assert!(validate_mobile_pairing_url("https://gateway.example.com/ws").is_ok());

    // Loopback and LAN cleartext allowed
    assert!(validate_mobile_pairing_url("ws://localhost:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://127.0.0.1:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://[::1]:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://192.168.1.100:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://10.0.1.20:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://172.20.0.5:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://169.254.10.20:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://macbook.local:8080").is_ok());
    assert!(validate_mobile_pairing_url("ws://10.0.2.2:8080").is_ok()); // Android emulator

    // Public cleartext internet rejected
    let public_err = validate_mobile_pairing_url("ws://8.8.8.8:8080").unwrap_err();
    assert!(public_err.contains("require a secure gateway URL"));
    let domain_err = validate_mobile_pairing_url("ws://public.openclaw.ai:8080").unwrap_err();
    assert!(domain_err.contains("require a secure gateway URL"));
}

#[test]
fn test_ocrm_pair_o17_join_code_registration_and_single_use_redeem() {
    use mahayana_host_runtime::device_pairing_owner::DEVICE_PAIRING_JOIN_CODE_BYTES;

    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let expires_at_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64
        + 60_000;
    let payload = json!({
        "url": "wss://gateway.example.test/ws",
        "bootstrapToken": "bootstrap-token",
        "expiresAtMs": expires_at_ms
    });

    let shortcode = owner
        .register_device_pairing_join_code(&payload, expires_at_ms)
        .expect("register join code");
    assert_eq!(shortcode.len(), 22);
    assert!(shortcode.len() >= (DEVICE_PAIRING_JOIN_CODE_BYTES * 4 / 3));
    assert!(
        shortcode
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    );

    let redeemed = owner
        .redeem_device_pairing_join_code(&shortcode)
        .expect("redeem join code")
        .expect("live payload");
    assert_eq!(redeemed, payload);
    assert!(
        owner
            .redeem_device_pairing_join_code(&shortcode)
            .expect("second redeem")
            .is_none()
    );
}

#[test]
fn test_ocrm_pair_o18_join_code_expiry_and_payload_bounds_fail_closed() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory db");
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis() as i64;
    assert!(
        owner
            .register_device_pairing_join_code(
                &json!({"url": "wss://gateway.example.test"}),
                now_ms
            )
            .is_err()
    );
    assert!(
        owner
            .redeem_device_pairing_join_code("invalid-shortcode")
            .expect("invalid shortcode is a safe miss")
            .is_none()
    );
    let oversized = json!("x".repeat(64 * 1024));
    assert!(
        owner
            .register_device_pairing_join_code(&oversized, now_ms + 60_000)
            .is_err()
    );
}

mod revision_admission {
    //! DPR-v1: projects/openclaw-rust/docs/specs/device-pairing-revision-admission.md.
    //! Discovery base: 8a4612e2042eefa32f12763c17141daabfd452e3.
    //! Real owner dispatch and independent SQL observations; no HTTP/device claim.

    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;

    use mahayana_host_runtime::device_pairing_owner::DevicePairingOwner;
    use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
    use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};
    use serde_json::{Value, json};

    const DEVICE_ID: &str = "dpr-durable-device";
    const ORIGINAL_KEY: &str = "dpr-original-public-key";
    const REPAIR_KEY: &str = "dpr-replacement-public-key";
    const REVISION_KEY: &str = "devicePairing.revision";
    const KNOWN_REVISION: u64 = 3;
    type StoredRow = BTreeMap<String, SqlValue>;

    #[derive(Debug, PartialEq)]
    struct StoredState {
        revision: Vec<StoredRow>,
        pending: Vec<StoredRow>,
        paired: Vec<StoredRow>,
    }

    fn read_rows(conn: &Connection, sql: &str, key: &str) -> Vec<StoredRow> {
        let mut statement = conn.prepare(sql).expect("prepare independent SQL observation");
        let columns: Vec<String> = statement
            .column_names()
            .into_iter()
            .map(str::to_owned)
            .collect();
        let rows = statement
            .query_map([key], |row| {
                columns
                    .iter()
                    .enumerate()
                    .map(|(index, name)| Ok((name.clone(), row.get::<_, SqlValue>(index)?)))
                    .collect::<rusqlite::Result<StoredRow>>()
            })
            .expect("query target rows")
            .collect::<rusqlite::Result<Vec<_>>>()
            .expect("retain raw SQL values");
        rows
    }

    fn text<'a>(row: &'a StoredRow, column: &str) -> &'a str {
        match row.get(column) {
            Some(SqlValue::Text(value)) => value,
            _ => panic!("expected SQL TEXT in {column}"),
        }
    }

    fn stored_tokens(state: &StoredState) -> Value {
        assert_eq!(state.paired.len(), 1);
        serde_json::from_str(text(&state.paired[0], "tokens_json"))
            .expect("actual durable role token map")
    }

    fn assert_revision(state: &StoredState, expected: u64) {
        assert_eq!(state.revision.len(), 1);
        assert_eq!(text(&state.revision[0], "sql_type"), "text");
        assert_eq!(
            text(&state.revision[0], "value_json"),
            format!("\"{expected}\"")
        );
    }

    struct Fixture {
        root: PathBuf,
        db_path: PathBuf,
        owner: Option<DevicePairingOwner>,
    }

    impl Fixture {
        fn new(label: &str) -> Self {
            let root = std::env::temp_dir().join(format!(
                "fabushi-pairing-revision-{label}-{}",
                uuid::Uuid::new_v4(),
            ));
            fs::create_dir(&root).expect("exclusively create UUID fixture directory");
            let mut fixture = Self {
                db_path: root.join("device-pairing.db"),
                root,
                owner: None,
            };
            fixture.reopen();
            fixture
        }

        fn owner(&self) -> &DevicePairingOwner {
            self.owner.as_ref().expect("open file-backed pairing owner")
        }

        fn reopen(&mut self) {
            drop(self.owner.take());
            self.owner = Some(DevicePairingOwner::new(&self.db_path).expect("reopen durable owner"));
        }

        fn observe(&self) -> StoredState {
            let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .expect("open independent read-only observer");
            let state = StoredState {
                revision: read_rows(
                    &conn,
                    "SELECT typeof(value_json) AS sql_type, value_json, updated_at_ms
                     FROM config_machine_state WHERE state_key = ?1",
                    REVISION_KEY,
                ),
                pending: read_rows(
                    &conn,
                    "SELECT * FROM device_pairing_pending WHERE device_id = ?1 ORDER BY request_id",
                    DEVICE_ID,
                ),
                paired: read_rows(
                    &conn,
                    "SELECT * FROM device_pairing_paired WHERE device_id = ?1",
                    DEVICE_ID,
                ),
            };
            conn.close().expect("close independent observer before any reopen/cleanup");
            state
        }

        fn set_revision(&self, value: &SqlValue) {
            let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
                .expect("open existing fixture for explicit counter fault or repair");
            assert_eq!(
                conn.execute(
                    "UPDATE config_machine_state SET value_json = ?1, updated_at_ms = 1234
                     WHERE state_key = ?2",
                    params![value, REVISION_KEY],
                )
                .expect("replace only the fixture's existing revision"),
                1,
            );
            conn.close().expect("close counter fixture writer");
        }

        fn seed_surfaces(&self) {
            let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
                .expect("open existing fixture for retained surfaces");
            assert_eq!(
                conn.execute(
                    "UPDATE device_pairing_paired
                     SET node_surface_json = ?1, pending_node_surface_json = ?2
                     WHERE device_id = ?3",
                    params![
                        "{\"generation\":7,\"commands\":[\"system.run\"]}",
                        "{\"generation\":8,\"commands\":[\"system.notify\"]}",
                        DEVICE_ID,
                    ],
                )
                .expect("seed both retained node surfaces"),
                1,
            );
            conn.close().expect("close retained-surface fixture writer");
        }

        fn request(&self, public_key: &str) -> Value {
            let response = self.owner().handle_pairing_command(
                "device.pair.request",
                &json!({
                    "deviceId": DEVICE_ID,
                    "publicKey": public_key,
                    "displayName": "DPR durable device",
                    "platform": "linux",
                    "roles": ["operator", "node"],
                    "scopes": ["operator.read", "operator.write", "node.invoke"]
                }),
            ).expect("create fresh pending request through public dispatch");
            assert_eq!(response["status"], "pending");
            assert_eq!(response["created"], true);
            response
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            drop(self.owner.take());
            if std::thread::panicking() {
                eprintln!("DPR fixture retained for Actions capture: {}", self.root.display());
            } else {
                fs::remove_dir_all(&self.root).expect("remove only the owned UUID fixture directory");
            }
        }
    }

    struct Authority {
        operator_token: String,
        node_token: String,
        request_id: String,
    }

    fn assert_authorized(owner: &DevicePairingOwner, token: &str, role: &str) {
        let authorization = owner.authenticate_device_token(token)
            .expect("read durable token authority")
            .expect("prior or replacement token remains authorized");
        assert_eq!(authorization.device_id, DEVICE_ID);
        assert_eq!(authorization.role, role);
        let expected_scopes = if role == "operator" {
            vec!["operator.read", "operator.write"]
        } else {
            vec!["node.invoke"]
        };
        assert_eq!(authorization.scopes, expected_scopes);
    }

    fn assert_invalid(owner: &DevicePairingOwner, token: &str) {
        assert!(owner.authenticate_device_token(token)
            .expect("read replaced or revoked token authority").is_none());
    }

    fn seeded_fixture(label: &str) -> (Fixture, Authority) {
        let fixture = Fixture::new(label);
        assert!(fixture.observe().revision.is_empty());
        let initial = fixture.request(ORIGINAL_KEY);
        let approval = fixture.owner().handle_pairing_command(
            "device.pair.approve", &json!({ "requestId": initial["requestId"] }),
        ).expect("establish durable paired device");
        assert_eq!(approval["revision"].as_u64(), Some(1));
        for (role, expected) in [("operator", 2), ("node", KNOWN_REVISION)] {
            let rotation = fixture.owner().handle_pairing_command(
                "device.pair.rotateToken", &json!({ "deviceId": DEVICE_ID, "role": role }),
            ).expect("establish known durable role tokens and revision history");
            assert_eq!(rotation["revision"].as_u64(), Some(expected));
        }
        fixture.seed_surfaces();
        let repair = fixture.request(REPAIR_KEY);
        assert_eq!(repair["isRepair"], true);
        let state = fixture.observe();
        assert_revision(&state, KNOWN_REVISION);
        assert_eq!(state.pending.len(), 1);
        assert_eq!(text(&state.pending[0], "public_key"), REPAIR_KEY);
        assert_eq!(text(&state.paired[0], "public_key"), ORIGINAL_KEY);
        assert!(matches!(state.paired[0].get("node_surface_json"), Some(SqlValue::Text(_))));
        assert!(matches!(state.paired[0].get("pending_node_surface_json"), Some(SqlValue::Text(_))));
        let tokens = stored_tokens(&state);
        let authority = Authority {
            operator_token: tokens["operator"]["token"].as_str().expect("operator token").to_owned(),
            node_token: tokens["node"]["token"].as_str().expect("node token").to_owned(),
            request_id: repair["requestId"].as_str().expect("fresh repair request id").to_owned(),
        };
        assert_authorized(fixture.owner(), &authority.operator_token, "operator");
        assert_authorized(fixture.owner(), &authority.node_token, "node");
        (fixture, authority)
    }

    #[derive(Clone, Copy, Debug)]
    enum Mutation {
        Approve,
        Revoke,
        Rotate,
    }

    impl Mutation {
        fn invoke(self, fixture: &Fixture, request_id: &str) -> Result<Value, GatewayCommandError> {
            let (method, args) = match self {
                Self::Approve => ("device.pair.approve", json!({ "requestId": request_id })),
                Self::Revoke => ("device.pair.revoke", json!({ "deviceId": DEVICE_ID })),
                Self::Rotate => ("device.pair.rotateToken", json!({
                    "deviceId": DEVICE_ID, "role": "node", "crossDevice": false
                })),
            };
            fixture.owner().handle_pairing_command(method, &args)
        }
    }

    fn assert_refusal(
        fixture: &Fixture,
        mutation: Mutation,
        authority: &Authority,
        before: &StoredState,
    ) {
        // An Err cannot carry a success receipt or replacement token. A panic
        // fails the test normally; it is never caught and accepted as refusal.
        let error = mutation.invoke(fixture, &authority.request_id)
            .expect_err("invalid existing revision must refuse the mutation");
        assert!(matches!(&error, GatewayCommandError::Internal(_)));
        assert_eq!(error.status(), 500); // Existing Gateway projection, not an HTTP execution claim.
        let diagnostic = error.to_string();
        assert!(!diagnostic.contains(&authority.operator_token));
        assert!(!diagnostic.contains(&authority.node_token));
        if let Some(SqlValue::Text(raw)) = before.revision[0].get("value_json") {
            assert!(!diagnostic.contains(raw), "diagnostic must not expose persisted revision text");
        }
        assert_eq!(&fixture.observe(), before, "{mutation:?} must retain raw counter and target rows");
        assert_authorized(fixture.owner(), &authority.operator_token, "operator");
        assert_authorized(fixture.owner(), &authority.node_token, "node");
    }

    fn assert_healthy_mutation(
        fixture: &Fixture,
        mutation: Mutation,
        authority: &Authority,
        before: &StoredState,
        receipt: &Value,
        expected_revision: u64,
    ) {
        assert_eq!(receipt["deviceId"], DEVICE_ID);
        assert_eq!(receipt["revision"].as_u64(), Some(expected_revision));
        let after = fixture.observe();
        assert_revision(&after, expected_revision);
        match mutation {
            Mutation::Approve => {
                assert_eq!(receipt["decision"], "approved");
                assert!(receipt.get("token").is_none());
                assert!(after.pending.is_empty());
                assert_eq!(text(&after.paired[0], "public_key"), REPAIR_KEY);
                assert_invalid(fixture.owner(), &authority.operator_token);
                assert_invalid(fixture.owner(), &authority.node_token);
            }
            Mutation::Revoke => {
                assert!(receipt.get("token").is_none());
                assert!(after.paired.is_empty());
                assert_eq!(after.pending, before.pending);
                assert_invalid(fixture.owner(), &authority.operator_token);
                assert_invalid(fixture.owner(), &authority.node_token);
                return;
            }
            Mutation::Rotate => {
                assert_eq!(receipt["role"], "node");
                assert_eq!(receipt["tokenDelivery"], "in-band");
                assert_eq!(after.pending, before.pending);
                let tokens_before = stored_tokens(before);
                let tokens_after = stored_tokens(&after);
                assert_eq!(tokens_after["operator"], tokens_before["operator"]);
                assert_eq!(tokens_after["node"]["token"], receipt["token"]);
                assert_ne!(receipt["token"].as_str(), Some(authority.node_token.as_str()));
                assert_invalid(fixture.owner(), &authority.node_token);
                let mut identity_before = before.paired[0].clone();
                let mut identity_after = after.paired[0].clone();
                for column in ["tokens_json", "node_surface_json", "pending_node_surface_json"] {
                    identity_before.remove(column);
                    identity_after.remove(column);
                }
                assert_eq!(identity_after, identity_before);
            }
        }
        assert_eq!(after.paired[0].get("node_surface_json"), Some(&SqlValue::Null));
        assert_eq!(after.paired[0].get("pending_node_surface_json"), Some(&SqlValue::Null));
        let tokens = stored_tokens(&after);
        for role in ["operator", "node"] {
            assert_authorized(fixture.owner(), tokens[role]["token"].as_str().expect("durable token"), role);
        }
    }

    #[test]
    fn pairing_revision_refuses_corrupt_counters_without_committing_mutations() {
        for mutation in [Mutation::Approve, Mutation::Revoke, Mutation::Rotate] {
            for (fault, raw) in [
                ("nonnumeric", SqlValue::Text("DPR-invalid-persisted-counter".into())),
                ("out-of-range", SqlValue::Text("18446744073709551616".into())),
                ("blob", SqlValue::Blob(vec![0xff, 0x00, 0x31])),
                ("exhausted", SqlValue::Text(format!("\"{}\"", u64::MAX))),
            ] {
                let (mut fixture, authority) = seeded_fixture(&format!("{mutation:?}-{fault}"));
                fixture.set_revision(&raw);
                let before = fixture.observe();
                assert_eq!(before.revision[0].get("value_json"), Some(&raw));
                assert_eq!(text(&before.revision[0], "sql_type"), if fault == "blob" { "blob" } else { "text" });
                assert_refusal(&fixture, mutation, &authority, &before);
                assert_refusal(&fixture, mutation, &authority, &before);
                fixture.reopen();
                assert_eq!(fixture.observe(), before);
                assert_refusal(&fixture, mutation, &authority, &before);

                // Explicit restoration of the fixture's last known good counter;
                // the product must never perform this repair itself.
                fixture.set_revision(&SqlValue::Text(format!("\"{KNOWN_REVISION}\"")));
                let repaired = fixture.observe();
                assert_revision(&repaired, KNOWN_REVISION);
                assert_eq!(repaired.pending, before.pending);
                assert_eq!(repaired.paired, before.paired);
                let receipt = mutation.invoke(&fixture, &authority.request_id)
                    .expect("known counter repair admits the same operation");
                assert_healthy_mutation(&fixture, mutation, &authority, &before, &receipt, KNOWN_REVISION + 1);
                let committed = fixture.observe();
                fixture.reopen();
                assert_eq!(fixture.observe(), committed);
                assert_healthy_mutation(&fixture, mutation, &authority, &before, &receipt, KNOWN_REVISION + 1);
            }
        }
    }

    #[test]
    fn pairing_revision_last_increment_is_durable_and_exhaustion_retains_authority() {
        let (mut fixture, authority) = seeded_fixture("last-increment");
        fixture.set_revision(&SqlValue::Text(format!("\"{}\"", u64::MAX - 1)));
        let before = fixture.observe();
        let receipt = Mutation::Rotate.invoke(&fixture, &authority.request_id)
            .expect("the final representable increment remains writable");
        assert_healthy_mutation(&fixture, Mutation::Rotate, &authority, &before, &receipt, u64::MAX);
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        assert_healthy_mutation(&fixture, Mutation::Rotate, &authority, &before, &receipt, u64::MAX);
        let current_authority = Authority {
            operator_token: authority.operator_token,
            node_token: receipt["token"].as_str().expect("last successful replacement token").to_owned(),
            request_id: authority.request_id,
        };
        assert_refusal(&fixture, Mutation::Rotate, &current_authority, &committed);
        assert_refusal(&fixture, Mutation::Revoke, &current_authority, &committed);
        fixture.reopen();
        assert_eq!(fixture.observe(), committed);
        assert_refusal(&fixture, Mutation::Approve, &current_authority, &committed);
    }

    #[test]
    fn pairing_revision_missing_and_legacy_decimal_counters_keep_monotonic_receipts() {
        {
            let mut fixture = Fixture::new("missing-revision");
            assert!(fixture.observe().revision.is_empty());
            let request = fixture.request(ORIGINAL_KEY);
            assert!(fixture.observe().revision.is_empty());
            let approval = fixture.owner().handle_pairing_command(
                "device.pair.approve", &json!({ "requestId": request["requestId"] }),
            ).expect("absent revision initializes on first successful mutation");
            assert_eq!(approval["revision"].as_u64(), Some(1));
            let first = fixture.observe();
            assert_revision(&first, 1);
            assert!(first.pending.is_empty());
            assert_eq!(text(&first.paired[0], "public_key"), ORIGINAL_KEY);
            let tokens = stored_tokens(&first);
            let authority = Authority {
                operator_token: tokens["operator"]["token"].as_str().unwrap().to_owned(),
                node_token: tokens["node"]["token"].as_str().unwrap().to_owned(),
                request_id: request["requestId"].as_str().unwrap().to_owned(),
            };
            fixture.reopen();
            assert_eq!(fixture.observe(), first);
            assert_authorized(fixture.owner(), &authority.operator_token, "operator");
            assert_authorized(fixture.owner(), &authority.node_token, "node");
            let rotation = Mutation::Rotate.invoke(&fixture, &authority.request_id)
                .expect("next fresh-store mutation advances to two");
            assert_healthy_mutation(&fixture, Mutation::Rotate, &authority, &first, &rotation, 2);
            let second = fixture.observe();
            fixture.reopen();
            assert_eq!(fixture.observe(), second);
            assert_healthy_mutation(&fixture, Mutation::Rotate, &authority, &first, &rotation, 2);
        }
        for (label, raw) in [
            ("raw-decimal", "41"),
            ("quoted-decimal", "\"41\""),
            ("legacy-quote-trim", "\"\"41\"\""),
        ] {
            let (mut fixture, authority) = seeded_fixture(label);
            fixture.set_revision(&SqlValue::Text(raw.to_owned()));
            let before = fixture.observe();
            assert_eq!(text(&before.revision[0], "value_json"), raw);
            fixture.reopen();
            assert_eq!(fixture.observe(), before);
            let receipt = Mutation::Rotate.invoke(&fixture, &authority.request_id)
                .expect("legacy decimal interpretation remains writable");
            assert_healthy_mutation(&fixture, Mutation::Rotate, &authority, &before, &receipt, 42);
            let committed = fixture.observe();
            fixture.reopen();
            assert_eq!(fixture.observe(), committed);
            assert_healthy_mutation(&fixture, Mutation::Rotate, &authority, &before, &receipt, 42);
        }
    }
}
