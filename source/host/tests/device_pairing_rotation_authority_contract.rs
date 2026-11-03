//! DPA-v1 owner contract: real file-backed production APIs and independent SQL.
//! Spec: projects/openclaw-rust/docs/specs/device-pairing-authenticated-rotation.md.
//! Fixed U: 1440bb1771372334bea23d7642bb52deb7db1845.
//! The scope cases below are explicit oracle examples from U's
//! src/shared/operator-scope-compat.ts, not a second implementation of its policy.
//! These tests make no HTTP, shipping-binary, or physical-device acceptance claim.

use std::cell::RefCell;
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::PathBuf;

use base64::Engine as _;
use mahayana_host_runtime::device_pairing_owner::{
    DevicePairingOwner, DeviceTokenAuthorization,
};
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CALLER: &str = "dpa-device-a";
const TARGET: &str = "dpa-device-b";
const REVISION_KEY: &str = "devicePairing.revision";
type StoredRow = BTreeMap<String, SqlValue>;

#[derive(Clone, Debug, PartialEq)]
struct StoredState {
    paired: Vec<StoredRow>,
    pending: Vec<StoredRow>,
    revision: Vec<StoredRow>,
}

fn read_rows(conn: &Connection, sql: &str) -> Vec<StoredRow> {
    let mut statement = conn.prepare(sql).expect("prepare independent SQL observation");
    let columns: Vec<String> = statement.column_names().into_iter().map(str::to_owned).collect();
    let rows = statement
        .query_map([], |row| {
            columns
                .iter()
                .enumerate()
                .map(|(index, column)| Ok((column.clone(), row.get::<_, SqlValue>(index)?)))
                .collect::<rusqlite::Result<StoredRow>>()
        })
        .expect("query complete fixture rows")
        .collect::<rusqlite::Result<Vec<_>>>()
        .expect("retain each raw SQL value");
    rows
}

fn text<'a>(row: &'a StoredRow, column: &str) -> &'a str {
    match row.get(column) {
        Some(SqlValue::Text(value)) => value,
        _ => panic!("expected SQL TEXT in {column}"),
    }
}

fn paired<'a>(state: &'a StoredState, device: &str) -> &'a StoredRow {
    let matches: Vec<_> = state.paired.iter().filter(|row| text(row, "device_id") == device).collect();
    assert_eq!(matches.len(), 1, "exactly one paired fixture device");
    matches[0]
}

fn tokens(state: &StoredState, device: &str) -> Value {
    serde_json::from_str(text(paired(state, device), "tokens_json"))
        .expect("decode independently observed token map")
}

fn token(state: &StoredState, device: &str, role: &str) -> String {
    tokens(state, device)[role]["token"].as_str().expect("persisted fixture token").to_owned()
}

fn revision(state: &StoredState) -> u64 {
    assert_eq!(state.revision.len(), 1);
    assert_eq!(text(&state.revision[0], "sql_type"), "text");
    text(&state.revision[0], "value_json").trim_matches('"').parse().expect("known valid counter")
}

fn assert_revision(state: &StoredState, expected: u64) {
    assert_eq!(state.revision.len(), 1);
    assert_eq!(text(&state.revision[0], "state_key"), REVISION_KEY);
    assert_eq!(text(&state.revision[0], "sql_type"), "text");
    assert_eq!(text(&state.revision[0], "value_json"), format!("\"{expected}\""));
    assert!(matches!(state.revision[0].get("updated_at_ms"), Some(SqlValue::Integer(value)) if *value > 0));
}

fn assert_scopes(actual: &Value, expected: &[&str]) {
    let actual: Vec<_> = actual.as_array().expect("scope array").iter()
        .map(|value| value.as_str().expect("string scope")).collect();
    assert_eq!(actual.len(), expected.len(), "no silently discarded or additional scopes");
    assert_eq!(actual.into_iter().collect::<BTreeSet<_>>(), expected.iter().copied().collect::<BTreeSet<_>>());
}

struct Fixture {
    root: PathBuf,
    db_path: PathBuf,
    owner: Option<DevicePairingOwner>,
    issued_tokens: RefCell<Vec<String>>,
}

impl Fixture {
    fn new(label: &str) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-dpa-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create owned UUID fixture directory");
        let mut fixture = Self {
            db_path: root.join("pairing.db"),
            root,
            owner: None,
            issued_tokens: RefCell::new(Vec::new()),
        };
        fixture.reopen();
        fixture
    }

    fn seeded(label: &str, caller_scopes: &[&str], target_scopes: &[&str]) -> Self {
        let fixture = Self::new(label);
        for (device, scopes) in [(CALLER, caller_scopes), (TARGET, target_scopes)] {
            let mut scopes = scopes.to_vec();
            if !scopes.contains(&"node.invoke") {
                scopes.push("node.invoke");
            }
            fixture.pair(device, &["operator", "node"], &scopes);
            fixture.seed_surfaces(device);
            let pending = fixture.request(device, "repair", &["operator", "node"], &scopes);
            assert_eq!(pending["isRepair"], true);
        }
        let state = fixture.observe();
        assert_eq!(state.paired.len(), 2);
        assert_eq!(state.pending.len(), 2);
        assert_revision(&state, 6);
        fixture
    }

    fn owner(&self) -> &DevicePairingOwner {
        self.owner.as_ref().expect("open file-backed production owner")
    }

    fn reopen(&mut self) {
        drop(self.owner.take());
        self.owner = Some(DevicePairingOwner::new(&self.db_path).expect("open durable pairing owner"));
    }

    fn observe(&self) -> StoredState {
        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open a separate read-only SQLite observer");
        conn.execute_batch("BEGIN DEFERRED").expect("pin one read-only observation snapshot");
        let state = StoredState {
            paired: read_rows(&conn, "SELECT * FROM device_pairing_paired ORDER BY device_id"),
            pending: read_rows(&conn, "SELECT * FROM device_pairing_pending ORDER BY request_id"),
            revision: read_rows(&conn,
                "SELECT state_key, typeof(value_json) AS sql_type, value_json, updated_at_ms
                 FROM config_machine_state WHERE state_key = 'devicePairing.revision'"),
        };
        conn.execute_batch("COMMIT").expect("finish observation snapshot");
        conn.close().expect("close observer before mutation or reopen");
        state
    }

    fn request(&self, device: &str, key_suffix: &str, roles: &[&str], scopes: &[&str]) -> Value {
        self.owner().handle_pairing_command("device.pair.request", &json!({
            "deviceId": device,
            "publicKey": format!("{device}-{key_suffix}-public-key"),
            "displayName": format!("DPA {device}"),
            "platform": "linux",
            "role": roles[0],
            "roles": roles,
            "scopes": scopes,
        })).expect("request pairing through the production public command")
    }

    fn pair(&self, device: &str, roles: &[&str], scopes: &[&str]) {
        let request = self.request(device, "original", roles, scopes);
        let approval = self.owner().handle_pairing_command(
            "device.pair.approve", &json!({"requestId": request["requestId"]}),
        ).expect("approve through the production public command");
        assert_eq!(approval["decision"], "approved");
        for role in roles {
            self.native_rotate(device, role);
        }
    }

    fn native_rotate(&self, device: &str, role: &str) -> Value {
        let result = self.owner().handle_pairing_command(
            "device.pair.rotateToken",
            &json!({"deviceId": device, "role": role, "crossDevice": false}),
        ).expect("obtain a real token through the existing native public API");
        assert_eq!(result["tokenDelivery"], "in-band");
        self.issued_tokens.borrow_mut().push(result["token"].as_str().expect("native token").to_owned());
        result
    }

    fn token(&self, device: &str, role: &str) -> String {
        token(&self.observe(), device, role)
    }

    fn authorization(&self, device: &str, role: &str) -> DeviceTokenAuthorization {
        self.authorize_token(&self.token(device, role))
    }

    fn authorize_token(&self, token: &str) -> DeviceTokenAuthorization {
        self.owner().authenticate_device_token(token)
            .expect("authenticate through the production owner")
            .expect("fixture bearer has a real owner-issued proof")
    }

    fn write_column(&self, device: &str, column: &str, value: &SqlValue) {
        assert!(matches!(column, "role" | "roles_json" | "scopes_json" | "approved_scopes_json" | "tokens_json"));
        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("open only the existing owned fixture for explicit fault injection");
        assert_eq!(conn.execute(
            &format!("UPDATE device_pairing_paired SET {column} = ?1 WHERE device_id = ?2"),
            params![value, device],
        ).expect("change exactly the designated fixture column"), 1);
        conn.close().expect("close explicit fixture writer");
    }

    fn set_token_scopes(&self, device: &str, role: &str, scopes: &[&str]) {
        let mut entries = tokens(&self.observe(), device);
        entries[role]["scopes"] = json!(scopes);
        self.write_column(device, "tokens_json", &SqlValue::Text(entries.to_string()));
    }

    fn remove_role_token(&self, device: &str, role: &str) {
        let mut entries = tokens(&self.observe(), device);
        assert!(entries.as_object_mut().expect("role map").remove(role).is_some());
        self.write_column(device, "tokens_json", &SqlValue::Text(entries.to_string()));
    }

    fn set_revision(&self, value: &SqlValue) {
        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("open existing fixture for explicit revision fault or repair");
        assert_eq!(conn.execute(
            "UPDATE config_machine_state SET value_json = ?1, updated_at_ms = 1234 WHERE state_key = ?2",
            params![value, REVISION_KEY],
        ).expect("replace only the existing fixture revision"), 1);
        conn.close().expect("close revision fixture writer");
    }

    fn seed_surfaces(&self, device: &str) {
        let conn = Connection::open_with_flags(&self.db_path, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("open existing fixture to retain two different node surfaces");
        assert_eq!(conn.execute(
            "UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2
             WHERE device_id = ?3",
            params![
                json!({"device": device, "generation": 7, "commands": ["system.run"]}).to_string(),
                json!({"device": device, "generation": 8, "commands": ["system.notify"]}).to_string(),
                device,
            ],
        ).expect("retain both node surfaces on exactly one fixture device"), 1);
        conn.close().expect("close surface fixture writer");
    }

    fn assert_secret_free(&self, diagnostic: &str) {
        assert!(!diagnostic.contains("sand-device-tok-"), "diagnostic exposes token bytes");
        for secret in self.issued_tokens.borrow().iter() {
            assert!(!diagnostic.contains(secret), "diagnostic exposes a prior or current bearer");
            let digest = Sha256::digest(secret.as_bytes());
            assert!(!diagnostic.contains(&format!("{digest:x}")), "diagnostic exposes token fingerprint");
            assert!(!diagnostic.contains(&format!("{digest:X}")), "diagnostic exposes token fingerprint");
            assert!(!diagnostic.contains(&format!("{:?}", digest.to_vec())), "diagnostic exposes fingerprint bytes");
        }
    }

    fn assert_safe_diagnostic(&self, diagnostic: &str, state: &StoredState) {
        self.assert_secret_free(diagnostic);
        for row in state.paired.iter().chain(&state.pending).chain(&state.revision) {
            for (column, value) in row {
                if column.ends_with("_json") {
                    if let SqlValue::Text(raw) = value {
                        if raw.len() > 2 {
                            assert!(!diagnostic.contains(raw), "diagnostic exposes raw persisted JSON");
                        }
                    }
                }
            }
        }
    }

    fn refuse(
        &self,
        authorization: &DeviceTokenAuthorization,
        args: Value,
        ceiling: Option<&str>,
        status: u16,
    ) -> GatewayCommandError {
        let before = self.observe();
        let error = match self.owner().rotate_token_for_authenticated_device(&args, authorization, ceiling) {
            Err(error) => error,
            Ok(_) => panic!("denied rotation returned a successful receipt"),
        };
        assert_eq!(error.status(), status, "refusal classification");
        self.assert_safe_diagnostic(&format!("{error:?}\n{error}"), &before);
        assert!(self.observe() == before, "refusal changed paired, pending, surface, token, or raw revision state");
        error
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.owner.take());
        if std::thread::panicking() {
            eprintln!("DPA fixture retained for Actions capture: {}", self.root.display());
        } else {
            fs::remove_dir_all(&self.root).expect("remove only this owned UUID fixture directory");
        }
    }
}

fn assert_authorized(fixture: &Fixture, bearer: &str, device: &str, role: &str, scopes: &[&str]) {
    let authorization = fixture.authorize_token(bearer);
    assert_eq!(authorization.device_id, device);
    assert_eq!(authorization.role, role);
    assert_scopes(&json!(authorization.scopes), scopes);
    fixture.assert_secret_free(&format!("{authorization:?}"));
}

fn assert_invalid(fixture: &Fixture, bearer: &str) {
    assert!(fixture.owner().authenticate_device_token(bearer)
        .expect("resolve replaced or revoked bearer").is_none());
}

fn assert_rotation(
    fixture: &Fixture,
    before: &StoredState,
    receipt: &Value,
    target: &str,
    role: &str,
    expected_scopes: &[&str],
    in_band: bool,
) -> String {
    assert_rotation_with_refused_unselected(fixture, before, receipt, target, role, expected_scopes, in_band, None)
}

fn assert_rotation_with_refused_unselected(
    fixture: &Fixture,
    before: &StoredState,
    receipt: &Value,
    target: &str,
    role: &str,
    expected_scopes: &[&str],
    in_band: bool,
    refused_unselected: Option<(&str, &str)>,
) -> String {
    let after = fixture.observe();
    let expected_revision = revision(before).checked_add(1).expect("successful fixture has counter headroom");
    assert_eq!(receipt["revision"].as_u64(), Some(expected_revision));
    assert_revision(&after, expected_revision);
    assert_eq!(receipt["deviceId"], target);
    assert_eq!(receipt["role"], role);
    assert_eq!(after.pending, before.pending, "all pending rows and their timestamps survive rotation");
    assert_eq!(after.paired.len(), before.paired.len());

    for prior in &before.paired {
        let device = text(prior, "device_id");
        let current = paired(&after, device);
        if device != target {
            assert_eq!(current, prior, "the other complete paired row must be unchanged");
        } else {
            for (column, value) in prior {
                if !matches!(column.as_str(), "tokens_json" | "node_surface_json" | "pending_node_surface_json") {
                    assert_eq!(current.get(column), Some(value), "rotation changed unrelated column {column}");
                }
            }
            assert_eq!(current.get("node_surface_json"), Some(&SqlValue::Null));
            assert_eq!(current.get("pending_node_surface_json"), Some(&SqlValue::Null));
        }
    }

    let prior_entries = tokens(before, target);
    let current_entries = tokens(&after, target);
    let new_bearer = current_entries[role]["token"].as_str().expect("committed replacement token").to_owned();
    let mut expected_entries = prior_entries.clone();
    expected_entries[role] = current_entries[role].clone();
    assert_eq!(current_entries, expected_entries, "only the selected role entry may change");
    assert_eq!(current_entries[role]["role"], role);
    assert_scopes(&current_entries[role]["scopes"], expected_scopes);
    assert_eq!(current_entries[role]["rotatedAtMs"], receipt["rotatedAtMs"]);
    assert_eq!(current_entries[role]["createdAtMs"], receipt["rotatedAtMs"]);
    assert!(current_entries[role].get("revokedAtMs")
        .is_none_or(|value| value.is_null() || value.as_i64() == Some(0)));
    assert!(receipt["rotatedAtMs"].as_i64().is_some_and(|value| value > 0));
    let payload = new_bearer.strip_prefix("sand-device-tok-").expect("unchanged native token prefix");
    assert_eq!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(payload).expect("base64url token").len(), 32);
    if let Some(old_bearer) = prior_entries[role]["token"].as_str() {
        assert_ne!(new_bearer, old_bearer);
        assert_invalid(fixture, old_bearer);
    }
    if in_band {
        assert_eq!(receipt["tokenDelivery"], "in-band");
        assert_eq!(receipt["token"].as_str(), Some(new_bearer.as_str()));
    } else {
        assert_eq!(receipt["tokenDelivery"], "withheld-cross-device");
        assert!(receipt.get("token").is_none());
        assert!(!receipt.to_string().contains(&new_bearer), "cross-device response leaks committed token");
    }
    fixture.issued_tokens.borrow_mut().push(new_bearer.clone());
    assert_authorized(fixture, &new_bearer, target, role, expected_scopes);
    let mut refused_count = 0;
    for prior in &before.paired {
        let device = text(prior, "device_id");
        let entries = tokens(before, device);
        for (entry_role, entry) in entries.as_object().expect("prior role map") {
            if device == target && entry_role == role {
                continue;
            }
            let scopes: Vec<_> = entry["scopes"].as_array().expect("unchanged scope array").iter()
                .map(|value| value.as_str().expect("unchanged string scope")).collect();
            let bearer = entry["token"].as_str().expect("unchanged bearer");
            if refused_unselected == Some((device, entry_role.as_str())) {
                assert_invalid(fixture, bearer);
                refused_count += 1;
            } else {
                assert_authorized(fixture, bearer, device, entry_role, &scopes);
            }
        }
    }
    assert_eq!(refused_count, usize::from(refused_unselected.is_some()), "pin the exact unselected invalid credential");
    new_bearer
}

#[test]
fn paired_rotation_proofs_bind_owner_and_sealed_projection() {
    let mut fixture = Fixture::seeded("proof", &["operator.write"], &["operator.write"]);
    let bearer = fixture.token(CALLER, "operator");
    let proof = fixture.authorize_token(&bearer);
    fixture.assert_secret_free(&format!("{proof:?}"));

    let foreign = Fixture::seeded("foreign-proof", &["operator.write"], &["operator.write"]);
    let foreign_error = foreign.refuse(&proof, json!({"deviceId": CALLER}), None, 401);
    fixture.assert_secret_free(&format!("{foreign_error:?}\n{foreign_error}"));
    let second_owner = DevicePairingOwner::new(&fixture.db_path).expect("independent issuer on the same file");
    let second_proof = second_owner.authenticate_device_token(&bearer).unwrap().unwrap();
    fixture.refuse(&second_proof, json!({"deviceId": CALLER}), None, 401);
    drop(second_owner);

    let mut altered = proof.clone();
    altered.device_id = TARGET.to_owned();
    altered.role = "node".to_owned();
    altered.scopes = vec!["operator.admin".to_owned()];
    fixture.refuse(&altered, json!({"deviceId": TARGET, "scopes": ["operator.write"]}), None, 403);
    fixture.refuse(&altered, json!({"deviceId": CALLER, "scopes": ["operator.admin"]}), None, 403);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER, "scopes": ["operator.write"], "crossDevice": true}),
        &altered, None,
    ).expect("the sealed legitimate self principal remains authoritative despite its altered projection");
    let successor = assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);

    let issued_before_reopen = fixture.authorize_token(&successor);
    let durable = fixture.observe();
    fixture.reopen();
    assert_eq!(fixture.observe(), durable, "reopen preserves all persisted authority");
    fixture.refuse(&issued_before_reopen, json!({"deviceId": CALLER}), None, 401);
    let fresh = fixture.authorize_token(&successor);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER}), &fresh, None,
    ).expect("reopened owner admits a newly issued proof for the persisted token");
    assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);
}

#[test]
fn paired_rotation_operator_permissions_follow_fixed_scope_implications() {
    // These literal outcomes pin U's narrow implication set, including prefix negatives.
    let cases: &[(&str, &[&str], &[&str], bool)] = &[
        ("write-implies", &["operator.write"],
            &["operator.read", "operator.talk", "operator.cron", "operator.sessions.read", "operator.sessions.write"], true),
        ("admin-implies-unknown-operator", &["operator.admin"], &["operator.future.permission"], true),
        ("write-no-admin", &["operator.write"], &["operator.admin"], false),
        ("write-no-approvals", &["operator.write"], &["operator.approvals"], false),
        ("write-no-unknown", &["operator.write"], &["operator.future.permission"], false),
        ("write-no-prefix-wildcard", &["operator.write"], &["operator.write.extra"], false),
        ("admin-no-node-prefix", &["operator.admin"], &["node.invoke"], false),
        ("read-cannot-rotate", &["operator.read"], &[], false),
        ("sessions-write-cannot-rotate", &["operator.sessions.write"], &[], false),
        ("lookalike-write-cannot-rotate", &["operator.write.extra"], &[], false),
        ("unknown-cannot-rotate", &["operator.future.permission"], &[], false),
        ("empty-cannot-rotate", &[], &[], false),
    ];
    for (label, grant, desired, allowed) in cases {
        let fixture = Fixture::seeded(label, grant, &["operator.write"]);
        let proof = fixture.authorization(CALLER, "operator");
        let args = json!({"deviceId": CALLER, "scopes": desired});
        if *allowed {
            let before = fixture.observe();
            let receipt = fixture.owner().rotate_token_for_authenticated_device(&args, &proof, None)
                .expect("explicit fixed-U positive scope implication");
            assert_rotation(&fixture, &before, &receipt, CALLER, "operator", desired, true);
        } else {
            fixture.refuse(&proof, args, None, 403);
        }
    }

    // A separately authenticated admin isolates the target baseline's read/session rules.
    let target_cases: &[(&str, &[&str], &[&str], bool)] = &[
        ("read-implies-session-read", &["operator.read"], &["operator.sessions.read"], true),
        ("session-write-implies-session-read", &["operator.sessions.write"], &["operator.sessions.read"], true),
        ("read-no-session-write", &["operator.read"], &["operator.sessions.write"], false),
        ("session-write-no-general-read", &["operator.sessions.write"], &["operator.read"], false),
        ("read-no-talk", &["operator.read"], &["operator.talk"], false),
        ("session-write-no-general-write", &["operator.sessions.write"], &["operator.write"], false),
    ];
    for (label, baseline, desired, allowed) in target_cases {
        let fixture = Fixture::seeded(label, &["operator.admin"], baseline);
        let proof = fixture.authorization(CALLER, "operator");
        let args = json!({"deviceId": TARGET, "scopes": desired});
        if *allowed {
            let before = fixture.observe();
            let receipt = fixture.owner().rotate_token_for_authenticated_device(&args, &proof, None)
                .expect("target baseline permits this exact fixed-U implication");
            assert_rotation(&fixture, &before, &receipt, TARGET, "operator", desired, false);
        } else {
            fixture.refuse(&proof, args, None, 403);
        }
    }
}

#[test]
fn paired_rotation_header_ceiling_and_body_flags_cannot_gain_authority() {
    let writer = Fixture::seeded("writer-ceiling", &["operator.write"], &["operator.write"]);
    let proof = writer.authorization(CALLER, "operator");
    for target in [TARGET, "dpa-absent-device"] {
        for cross_device in [false, true] {
            writer.refuse(&proof, json!({
                "deviceId": target, "crossDevice": cross_device,
                "callerDeviceId": target, "callerRole": "operator",
                "callerScopes": ["operator.admin"], "admin": true,
            }), Some("operator.admin"), 403);
        }
    }
    writer.refuse(&proof, json!({"deviceId": TARGET}), None, 403);
    for role in ["node", "unapproved-role"] {
        writer.refuse(&proof, json!({"deviceId": CALLER, "role": role}), None, 403);
    }
    writer.write_column(TARGET, "tokens_json", &SqlValue::Text("DPA-unreadable-target-token-map".to_owned()));
    writer.refuse(&proof, json!({"deviceId": TARGET}), None, 403);

    let admin = Fixture::seeded("admin-ceiling", &["operator.admin"], &["operator.write"]);
    let proof = admin.authorization(CALLER, "operator");
    admin.refuse(&proof, json!({"deviceId": TARGET}), Some("operator.write"), 403);
    admin.refuse(&proof, json!({"deviceId": CALLER, "scopes": ["operator.admin"]}), Some("operator.write"), 403);
    for ceiling in ["", "  ", "operator.read", "operator.write.extra", "admin", "node.write"] {
        admin.refuse(&proof, json!({"deviceId": CALLER, "scopes": []}), Some(ceiling), 403);
    }
    let before = admin.observe();
    let receipt = admin.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": TARGET, "scopes": ["operator.write"], "crossDevice": false,
            "callerDeviceId": TARGET, "admin": true}),
        &proof, Some("operator.admin"),
    ).expect("real admin permits cross-device rotation but body flags cannot cause disclosure");
    assert_rotation(&admin, &before, &receipt, TARGET, "operator", &["operator.write"], false);

    let before = admin.observe();
    let receipt = admin.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER, "scopes": ["operator.write"], "crossDevice": true,
            "callerDeviceId": TARGET}),
        &proof, Some("operator.write"),
    ).expect("write ceiling permits only legitimate self write and still delivers the self token");
    assert_rotation(&admin, &before, &receipt, CALLER, "operator", &["operator.write"], true);

    let implied = Fixture::seeded("ceiling-implications", &["operator.admin"], &["operator.write"]);
    let proof = implied.authorization(CALLER, "operator");
    implied.refuse(&proof, json!({"deviceId": CALLER, "scopes": ["operator.future.permission"]}),
        Some("operator.write"), 403);
    let desired = ["operator.read", "operator.talk", "operator.cron", "operator.sessions.read", "operator.sessions.write"];
    let before = implied.observe();
    let receipt = implied.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER, "scopes": desired}), &proof, Some("operator.write"),
    ).expect("the constraining header retains precisely the fixed-U write implications");
    assert_rotation(&implied, &before, &receipt, CALLER, "operator", &desired, true);
}

#[test]
fn paired_rotation_desired_scopes_preserve_narrow_tokens_and_explicit_empty() {
    let fixture = Fixture::seeded("desired", &["operator.admin"], &["operator.read", "operator.write"]);
    let proof = fixture.authorization(CALLER, "operator");
    for (desired, expected) in [
        (Some(json!(["operator.read"])), vec!["operator.read"]),
        (None, vec!["operator.read"]),
        (Some(json!([])), vec![]),
        (None, vec![]),
        (Some(json!(["operator.write"])), vec!["operator.write"]),
    ] {
        let mut args = json!({"deviceId": TARGET});
        if let Some(desired) = desired {
            args["scopes"] = desired;
        }
        let before = fixture.observe();
        let receipt = fixture.owner().rotate_token_for_authenticated_device(&args, &proof, None)
            .expect("explicit, omitted, and empty desired scopes have distinct durable meanings");
        assert_rotation(&fixture, &before, &receipt, TARGET, "operator", &expected, false);
    }
    fixture.refuse(&proof, json!({
        "deviceId": TARGET, "scopes": ["operator.read", "operator.admin"],
    }), None, 403);

    fixture.remove_role_token(TARGET, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": TARGET}), &proof, None,
    ).expect("only an absent selected role token falls back to its role-filtered approval");
    assert_rotation(&fixture, &before, &receipt, TARGET, "operator", &["operator.read", "operator.write"], false);

    // SQL NULL alone permits the legacy scopes column to supply the baseline.
    fixture.write_column(TARGET, "approved_scopes_json", &SqlValue::Null);
    fixture.write_column(TARGET, "scopes_json", &SqlValue::Text(json!(["operator.read", "node.invoke"]).to_string()));
    fixture.remove_role_token(TARGET, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": TARGET}), &proof, None,
    ).expect("SQL NULL approval uses a valid legacy baseline and filters its other role scopes");
    assert_rotation(&fixture, &before, &receipt, TARGET, "operator", &["operator.read"], false);

    // A non-NULL approval, even an empty one, must never fall back to wider legacy scopes.
    fixture.write_column(TARGET, "scopes_json", &SqlValue::Text(json!(["operator.admin", "node.invoke"]).to_string()));
    fixture.write_column(TARGET, "approved_scopes_json", &SqlValue::Text(json!(["operator.read", "node.invoke"]).to_string()));
    fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": ["operator.write"]}), None, 403);
    let node = fixture.authorize_token(&fixture.token(TARGET, "node"));
    fixture.write_column(TARGET, "approved_scopes_json", &SqlValue::Text("[]".to_owned()));
    fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": ["operator.read"]}), None, 403);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": TARGET, "scopes": []}), &proof, None,
    ).expect("explicit empty desired scopes remain bounded by an explicitly empty approval");
    assert_rotation_with_refused_unselected(&fixture, &before, &receipt, TARGET, "operator", &[], false, Some((TARGET, "node")));
    let after = fixture.observe();
    assert_eq!(fixture.owner().validate_current_device_authorization(&node).unwrap_err().status(), 401);
    assert!(fixture.observe() == after, "unselected retired proof validation preserves the committed rotation");
}

#[test]
fn paired_rotation_live_credentials_and_approval_narrow_prior_proofs() {
    // The token bytes remain identical: this exercises live grants, not token replacement.
    for narrow_approval_too in [false, true] {
        let fixture = Fixture::seeded("live-narrowing", &["operator.admin"], &["operator.write"]);
        let bearer = fixture.token(CALLER, "operator");
        let proof = fixture.authorize_token(&bearer);
        fixture.set_token_scopes(CALLER, "operator", &["operator.write"]);
        if narrow_approval_too {
            fixture.write_column(CALLER, "approved_scopes_json",
                &SqlValue::Text(json!(["operator.write", "node.invoke"]).to_string()));
        }
        assert_eq!(fixture.token(CALLER, "operator"), bearer);
        fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": ["operator.write"]}), None, 403);
        fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": ["operator.admin"]}), None, 403);
        let args = if narrow_approval_too {
            json!({"deviceId": CALLER, "scopes": ["operator.write"]})
        } else {
            json!({"deviceId": CALLER})
        };
        let before = fixture.observe();
        let receipt = fixture.owner().rotate_token_for_authenticated_device(
            &args, &proof, None,
        ).expect("a narrowed live write grant still permits a bounded self-write rotation");
        assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);
    }

    // DPA-02: the entire live token grant must remain approved. Narrowing only
    // approval leaves a now-unapproved admin token and invalidates the credential.
    let mut fixture = Fixture::seeded("approval-only-narrowing", &["operator.admin"], &["operator.write"]);
    let bearer = fixture.token(CALLER, "operator");
    let proof = fixture.authorize_token(&bearer);
    fixture.write_column(CALLER, "approved_scopes_json",
        &SqlValue::Text(json!(["operator.write", "node.invoke"]).to_string()));
    fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": ["operator.write"]}), None, 401);
    fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": []}), None, 401);
    let durable = fixture.observe();
    fixture.reopen();
    assert_eq!(fixture.observe(), durable);
    // Fresh admission must refuse this persisted unapproved grant after reopen.
    let after_reopen = fixture.observe();
    assert_invalid(&fixture, &bearer);
    assert_eq!(fixture.observe(), after_reopen, "fresh refusal does not repair narrowed approval");

    // Live amplification cannot retroactively enlarge what authentication sealed.
    let fixture = Fixture::seeded("sealed-grant", &["operator.write"], &["operator.write"]);
    let bearer = fixture.token(CALLER, "operator");
    let proof = fixture.authorize_token(&bearer);
    fixture.set_token_scopes(CALLER, "operator", &["operator.admin"]);
    fixture.write_column(CALLER, "approved_scopes_json",
        &SqlValue::Text(json!(["operator.admin", "node.invoke"]).to_string()));
    assert_eq!(fixture.token(CALLER, "operator"), bearer);
    fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": ["operator.write"]}), None, 403);
    fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": ["operator.admin"]}), None, 403);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER, "scopes": ["operator.write"]}), &proof, None,
    ).expect("sealed write remains the ceiling even when the same live token is widened");
    assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);
}

#[test]
fn paired_rotation_stored_corruption_and_bad_requests_preserve_all_rows() {
    let fixture = Fixture::seeded("request-types", &["operator.admin"], &["operator.write"]);
    let proof = fixture.authorization(CALLER, "operator");
    for args in [
        Value::Null,
        json!([]),
        json!("not-an-object"),
        json!({}),
        json!({"deviceId": 7}),
        json!({"deviceId": ""}),
        json!({"deviceId": "  "}),
        json!({"deviceId": TARGET, "role": false}),
        json!({"deviceId": TARGET, "role": null}),
        json!({"deviceId": TARGET, "role": ""}),
        json!({"deviceId": TARGET, "scopes": null}),
        json!({"deviceId": TARGET, "scopes": "operator.read"}),
        json!({"deviceId": TARGET, "scopes": {}}),
        json!({"deviceId": TARGET, "scopes": ["operator.read", 7]}),
        json!({"deviceId": TARGET, "crossDevice": "false"}),
        json!({"deviceId": TARGET, "crossDevice": null}),
    ] {
        fixture.refuse(&proof, args, None, 400);
    }

    let faults = [
        ("approval-syntax", "approved_scopes_json", SqlValue::Text("DPA-broken-approved-json".to_owned())),
        ("approval-object", "approved_scopes_json", SqlValue::Text(json!({"raw": "DPA-private-baseline"}).to_string())),
        ("approval-json-null", "approved_scopes_json", SqlValue::Text("null".to_owned())),
        ("approval-number", "approved_scopes_json", SqlValue::Text("[7]".to_owned())),
        ("approval-mixed", "approved_scopes_json", SqlValue::Text("[\"operator.write\",false]".to_owned())),
        ("approval-sql-blob", "approved_scopes_json", SqlValue::Blob(vec![0, 255, 7])),
        ("roles-syntax", "roles_json", SqlValue::Text("DPA-broken-roles-json".to_owned())),
        ("roles-object", "roles_json", SqlValue::Text("{\"DPA-private-roles\":true}".to_owned())),
        ("tokens-syntax", "tokens_json", SqlValue::Text("DPA-broken-token-json".to_owned())),
        ("tokens-array", "tokens_json", SqlValue::Text("[]".to_owned())),
        ("tokens-null", "tokens_json", SqlValue::Text("null".to_owned())),
        ("tokens-sql-blob", "tokens_json", SqlValue::Blob(vec![255, 0, 9])),
    ];
    for (label, column, raw) in &faults {
        for device in [CALLER, TARGET] {
            let mut fixture = Fixture::seeded(label, &["operator.admin"], &["operator.write"]);
            let bearer = fixture.token(CALLER, "operator");
            let proof = fixture.authorize_token(&bearer);
            let valid = fixture.observe();
            fixture.write_column(device, column, raw);
            fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": ["operator.read"]}), None, 500);
            if device == TARGET {
                let corrupt = fixture.observe();
                fixture.reopen();
                assert_eq!(fixture.observe(), corrupt, "reopen must not repair corrupt authorization");
                // Issue fresh provenance without depending on the unordered authentication
                // scan's behavior around another row's wrong SQL type; then restore the
                // exact corrupt snapshot before the decisive rotation admission.
                fixture.write_column(device, column, paired(&valid, device).get(*column).unwrap());
                let fresh = fixture.authorize_token(&bearer);
                fixture.write_column(device, column, raw);
                assert_eq!(fixture.observe(), corrupt);
                fixture.refuse(&fresh, json!({"deviceId": TARGET, "scopes": ["operator.read"]}), None, 500);
            }
            fixture.write_column(device, column, paired(&valid, device).get(*column).unwrap());
            let fresh = fixture.authorize_token(&bearer);
            let before = fixture.observe();
            let receipt = fixture.owner().rotate_token_for_authenticated_device(
                &json!({"deviceId": TARGET, "scopes": ["operator.read"]}), &fresh, None,
            ).expect("restoring only the known good fixture field permits the same bounded action");
            assert_rotation(&fixture, &before, &receipt, TARGET, "operator", &["operator.read"], false);
        }
    }

    // Missing both baseline columns is different from an explicitly empty array.
    for device in [CALLER, TARGET] {
        let fixture = Fixture::seeded("missing-baseline", &["operator.admin"], &["operator.write"]);
        let proof = fixture.authorization(CALLER, "operator");
        fixture.write_column(device, "approved_scopes_json", &SqlValue::Null);
        fixture.write_column(device, "scopes_json", &SqlValue::Null);
        fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": []}), None, 500);
        fixture.write_column(device, "scopes_json", &SqlValue::Text("DPA-corrupt-legacy-baseline".to_owned()));
        fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": []}), None, 500);
    }

    // An existing role entry with malformed scopes cannot become the missing-entry fallback.
    for device in [CALLER, TARGET] {
        for malformed_scopes in [Value::Null, json!("operator.write"), json!(["operator.write", 9])] {
            let fixture = Fixture::seeded("entry-shape", &["operator.admin"], &["operator.write"]);
            let proof = fixture.authorization(CALLER, "operator");
            let mut entries = tokens(&fixture.observe(), device);
            entries["operator"]["scopes"] = malformed_scopes;
            fixture.write_column(device, "tokens_json", &SqlValue::Text(entries.to_string()));
            fixture.refuse(&proof, json!({"deviceId": TARGET}), None, 500);
        }
    }
}

#[test]
fn paired_rotation_revision_failure_rolls_back_and_recovers_durably() {
    for (label, raw) in [
        ("corrupt", SqlValue::Text("DPA-private-undecodable-revision".to_owned())),
        ("range", SqlValue::Text("\"18446744073709551616\"".to_owned())),
        ("blob", SqlValue::Blob(vec![255, 0, 1])),
        ("exhausted", SqlValue::Text(format!("\"{}\"", u64::MAX))),
    ] {
        let mut fixture = Fixture::seeded(label, &["operator.write"], &["operator.write"]);
        let known = fixture.observe();
        let bearer = token(&known, CALLER, "operator");
        let node_bearer = token(&known, CALLER, "node");
        let proof = fixture.authorize_token(&bearer);
        fixture.set_revision(&raw);
        let corrupt = fixture.observe();
        assert_eq!(corrupt.revision[0].get("value_json"), Some(&raw));
        fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": ["operator.write"]}), None, 500);
        assert_authorized(&fixture, &bearer, CALLER, "operator", &["operator.write"]);
        assert_authorized(&fixture, &node_bearer, CALLER, "node", &["node.invoke"]);

        fixture.reopen();
        assert_eq!(fixture.observe(), corrupt, "corrupt raw revision and both surfaces survive reopen");
        let fresh = fixture.authorize_token(&bearer);
        fixture.refuse(&fresh, json!({"deviceId": CALLER, "scopes": ["operator.write"]}), None, 500);
        assert_authorized(&fixture, &bearer, CALLER, "operator", &["operator.write"]);
        assert_authorized(&fixture, &node_bearer, CALLER, "node", &["node.invoke"]);

        fixture.set_revision(known.revision[0].get("value_json").unwrap());
        let before = fixture.observe();
        let receipt = fixture.owner().rotate_token_for_authenticated_device(
            &json!({"deviceId": CALLER, "scopes": ["operator.write"]}), &fresh, None,
        ).expect("controlled restoration of the original counter permits exactly one committed mutation");
        let successor = assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);
        let committed = fixture.observe();
        fixture.reopen();
        assert_eq!(fixture.observe(), committed, "committed repair control survives reopen");
        assert_invalid(&fixture, &bearer);
        assert_authorized(&fixture, &successor, CALLER, "operator", &["operator.write"]);
        assert_authorized(&fixture, &node_bearer, CALLER, "node", &["node.invoke"]);
    }

    let mut fixture = Fixture::seeded("last-revision", &["operator.write"], &["operator.write"]);
    fixture.set_revision(&SqlValue::Text(format!("\"{}\"", u64::MAX - 1)));
    let proof = fixture.authorization(CALLER, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER}), &proof, None,
    ).expect("the last representable increment remains a valid committed rotation");
    let successor = assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);
    assert_revision(&fixture.observe(), u64::MAX);
    let fresh = fixture.authorize_token(&successor);
    fixture.refuse(&fresh, json!({"deviceId": CALLER}), None, 500);
    let exhausted = fixture.observe();
    fixture.reopen();
    assert_eq!(fixture.observe(), exhausted);
    let reopened = fixture.authorize_token(&successor);
    fixture.refuse(&reopened, json!({"deviceId": CALLER}), None, 500);
    assert_authorized(&fixture, &successor, CALLER, "operator", &["operator.write"]);
}

#[test]
fn paired_rotation_replacement_revocation_and_reopen_retire_proofs() {
    let mut fixture = Fixture::seeded("retirement", &["operator.write"], &["operator.write"]);
    let original = fixture.token(CALLER, "operator");
    let proof = fixture.authorize_token(&original);

    // An unrelated revision advance is not a credential generation change.
    let before = fixture.observe();
    let native = fixture.native_rotate(TARGET, "operator");
    assert_rotation(&fixture, &before, &native, TARGET, "operator", &["operator.write"], true);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER}), &proof, None,
    ).expect("a still-current bearer survives an unrelated device's revision mutation");
    let first = assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);
    fixture.refuse(&proof, json!({"deviceId": CALLER}), None, 401);
    assert_invalid(&fixture, &original);

    // Retirement through the pre-existing native command also fences a captured proof.
    let captured = fixture.authorize_token(&first);
    let before = fixture.observe();
    let native = fixture.native_rotate(CALLER, "operator");
    let second = assert_rotation(&fixture, &before, &native, CALLER, "operator", &["operator.write"], true);
    fixture.refuse(&captured, json!({"deviceId": CALLER}), None, 401);
    assert_invalid(&fixture, &first);

    let proof_before_reopen = fixture.authorize_token(&second);
    let durable = fixture.observe();
    fixture.reopen();
    assert_eq!(fixture.observe(), durable);
    fixture.refuse(&proof_before_reopen, json!({"deviceId": CALLER}), None, 401);
    assert_invalid(&fixture, &original);
    assert_invalid(&fixture, &first);
    let fresh = fixture.authorize_token(&second);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER}), &fresh, None,
    ).expect("fresh proof for the persisted successor works after reopen");
    let third = assert_rotation(&fixture, &before, &receipt, CALLER, "operator", &["operator.write"], true);

    let captured = fixture.authorize_token(&third);
    let before = fixture.observe();
    let removed_node = token(&before, CALLER, "node");
    let revoke = fixture.owner().handle_pairing_command(
        "device.pair.revoke", &json!({"deviceId": CALLER}),
    ).expect("existing native whole-device revoke retires every role");
    let retired = fixture.observe();
    assert_eq!(revoke["revision"].as_u64(), Some(revision(&before) + 1));
    assert_revision(&retired, revision(&before) + 1);
    assert_eq!(retired.pending, before.pending);
    assert_eq!(retired.paired.len(), before.paired.len() - 1);
    assert_eq!(paired(&retired, TARGET), paired(&before, TARGET));
    assert!(!retired.paired.iter().any(|row| text(row, "device_id") == CALLER));
    fixture.refuse(&captured, json!({"deviceId": CALLER}), None, 401);
    assert_invalid(&fixture, &third);
    assert_invalid(&fixture, &removed_node);
    fixture.reopen();
    assert_eq!(fixture.observe(), retired);
    fixture.refuse(&captured, json!({"deviceId": CALLER}), None, 401);
    assert_invalid(&fixture, &third);
    assert_invalid(&fixture, &removed_node);
    let unaffected = fixture.authorization(TARGET, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": TARGET}), &unaffected, None,
    ).expect("the other device's durable authority remains usable after revoke and reopen");
    assert_rotation(&fixture, &before, &receipt, TARGET, "operator", &["operator.write"], true);

    for retirement in ["missing-token", "revoked-entry", "unapproved-role"] {
        let fixture = Fixture::seeded(retirement, &["operator.write"], &["operator.write"]);
        let proof = fixture.authorization(CALLER, "operator");
        match retirement {
            "missing-token" => fixture.remove_role_token(CALLER, "operator"),
            "revoked-entry" => {
                let mut entries = tokens(&fixture.observe(), CALLER);
                entries["operator"]["revokedAtMs"] = json!(1234);
                fixture.write_column(CALLER, "tokens_json", &SqlValue::Text(entries.to_string()));
            }
            "unapproved-role" => {
                fixture.write_column(CALLER, "role", &SqlValue::Text("node".to_owned()));
                fixture.write_column(CALLER, "roles_json", &SqlValue::Text("[\"node\"]".to_owned()));
            }
            _ => unreachable!(),
        }
        fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": ["operator.write"]}), None, 401);
    }
}

#[test]
fn paired_rotation_admin_other_roles_require_literal_prefixed_baseline() {
    let fixture = Fixture::seeded("admin-node", &["operator.admin"], &["operator.write", "node.read", "node.write"]);
    let proof = fixture.authorization(CALLER, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": TARGET, "role": "node", "scopes": ["node.read"], "crossDevice": false}),
        &proof, None,
    ).expect("real operator admin can manage a different approved node role within its literal baseline");
    assert_rotation(&fixture, &before, &receipt, TARGET, "node", &["node.read"], false);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_for_authenticated_device(
        &json!({"deviceId": CALLER, "role": "node", "scopes": ["node.invoke"], "crossDevice": true}),
        &proof, None,
    ).expect("admin self-node rotation delivers in band based on sealed device identity");
    assert_rotation(&fixture, &before, &receipt, CALLER, "node", &["node.invoke"], true);
    fixture.refuse(&proof, json!({"deviceId": TARGET, "role": "unapproved-role", "scopes": []}), None, 403);

    let node_cases: &[(&str, &[&str], &[&str])] = &[
        ("node-no-write-implication", &["node.write"], &["node.read"]),
        ("node-no-prefix-wildcard", &["node.invoke"], &["node.invoke.child"]),
        ("node-no-star-wildcard", &["node.*"], &["node.read"]),
        ("node-no-operator-implication", &["operator.admin"], &["operator.read"]),
        ("node-reject-other-prefix", &["other.read"], &["other.read"]),
        ("node-reject-lookalike-prefix", &["nodes.read"], &["nodes.read"]),
    ];
    for (label, baseline, desired) in node_cases {
        let fixture = Fixture::seeded(label, &["operator.admin"], baseline);
        let proof = fixture.authorization(CALLER, "operator");
        fixture.refuse(&proof, json!({"deviceId": TARGET, "role": "node", "scopes": desired}), None, 403);
    }

    // Publicly approved non-operator tokens still cannot acquire operator management.
    for role in ["node", "unknown-role"] {
        let fixture = Fixture::new("non-operator-caller");
        let granted = format!("{role}.write");
        fixture.pair(CALLER, &[role], &[&granted]);
        fixture.pair(TARGET, &["operator", "node"], &["operator.write", "node.invoke"]);
        fixture.seed_surfaces(CALLER);
        fixture.seed_surfaces(TARGET);
        let mut proof = fixture.authorization(CALLER, role);
        proof.device_id = TARGET.to_owned();
        proof.role = "operator".to_owned();
        proof.scopes = vec!["operator.admin".to_owned()];
        fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": []}), Some("operator.admin"), 403);
        fixture.refuse(&proof, json!({"deviceId": TARGET, "scopes": []}), Some("operator.admin"), 403);

        // DPA-02 precedes role policy: the same captured non-operator proof must
        // become an invalid credential after replacement, then remain invalid
        // after removal, rather than retaining its earlier 403 classification.
        let original = fixture.token(CALLER, role);
        let before = fixture.observe();
        let replacement = fixture.native_rotate(CALLER, role);
        let successor = assert_rotation(
            &fixture, &before, &replacement, CALLER, role, &[&granted], true,
        );
        fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": []}), Some("operator.admin"), 401);
        assert_invalid(&fixture, &original);
        let captured_before_removal = fixture.authorize_token(&successor);
        fixture.refuse(&captured_before_removal, json!({"deviceId": CALLER, "scopes": []}), None, 403);
        let before = fixture.observe();
        let revoke = fixture.owner().handle_pairing_command(
            "device.pair.revoke", &json!({"deviceId": CALLER}),
        ).expect("retire the currently authenticated non-operator role through native whole-device revoke");
        let retired = fixture.observe();
        assert_eq!(revoke["revision"].as_u64(), Some(revision(&before) + 1));
        assert_revision(&retired, revision(&before) + 1);
        assert_eq!(retired.pending, before.pending);
        assert_eq!(retired.paired.len(), before.paired.len() - 1);
        assert_eq!(paired(&retired, TARGET), paired(&before, TARGET));
        assert!(!retired.paired.iter().any(|row| text(row, "device_id") == CALLER));
        fixture.refuse(&proof, json!({"deviceId": CALLER, "scopes": []}), Some("operator.admin"), 401);
        fixture.refuse(&captured_before_removal, json!({"deviceId": CALLER, "scopes": []}), None, 401);
        assert_invalid(&fixture, &successor);
    }
}
