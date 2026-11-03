//! DRT-v1 selected-role owner contracts: production authentication/mutations, real SQLite files,
//! and independent complete-row observations. No HTTP or device acceptance claim.
//! Spec: projects/openclaw-rust/docs/specs/device-role-token-revocation.md.
//! Fixed U: 1440bb1771372334bea23d7642bb52deb7db1845.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Barrier;

use base64::Engine as _;
use mahayana_host_runtime::device_pairing_owner::{
    DevicePairingOwner, DeviceTokenAuthorization,
};
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

const CALLER: &str = "drt-caller";
const TARGET: &str = "drt-target";
const BYSTANDER: &str = "drt-bystander";
const ABSENT: &str = "drt-absent";
const REVISION: &str = "devicePairing.revision";
type RawRow = BTreeMap<String, SqlValue>;

// Deliberately no Debug: a failed equality must not dump durable credentials.
#[derive(Clone, PartialEq)]
struct Snapshot {
    paired: Vec<RawRow>,
    pending: Vec<RawRow>,
    machine: Vec<RawRow>,
}

fn rows(conn: &Connection, sql: &str) -> Vec<RawRow> {
    let mut statement = conn.prepare(sql).expect("prepare independent observation");
    let names: Vec<_> = statement.column_names().into_iter().map(str::to_owned).collect();
    let mut cursor = statement.query([]).expect("query independent observation");
    let mut result = Vec::new();
    while let Some(row) = cursor.next().expect("read every observed row") {
        let mut values = RawRow::new();
        for (index, name) in names.iter().enumerate() {
            values.insert(name.clone(), row.get::<_, SqlValue>(index).expect("retain raw SQL type and value"));
        }
        result.push(values);
    }
    result
}

fn text<'a>(row: &'a RawRow, column: &str) -> &'a str {
    match row.get(column) {
        Some(SqlValue::Text(value)) => value,
        _ => panic!("expected SQL TEXT in fixture column {column}"),
    }
}

impl Snapshot {
    fn device(&self, device: &str) -> &RawRow {
        let mut matching = self.paired.iter().filter(|row| text(row, "device_id") == device);
        let row = matching.next().expect("one paired fixture device");
        assert!(matching.next().is_none(), "device identity must be unique");
        row
    }

    fn entries(&self, device: &str) -> Value {
        serde_json::from_str(text(self.device(device), "tokens_json"))
            .expect("decode a known valid fixture token map")
    }

    fn token(&self, device: &str, role: &str) -> String {
        self.entries(device)[role]["token"].as_str().expect("real durable role token").to_owned()
    }

    fn counter(&self) -> &RawRow {
        self.machine.iter().find(|row| text(row, "state_key") == REVISION)
            .expect("one durable revision row")
    }

    fn revision(&self) -> u64 {
        assert_eq!(text(self.counter(), "value_type"), "text");
        text(self.counter(), "value_json").trim_matches('"').parse().expect("known valid revision")
    }
}

struct Fixture {
    root: PathBuf,
    db: PathBuf,
    owner: Option<DevicePairingOwner>,
    secrets: RefCell<Vec<String>>,
}

impl Fixture {
    fn seeded(label: &str, roles: &[&str], scopes: &[&str]) -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-drt-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create this owned UUID directory");
        let mut fixture = Self {
            db: root.join("pairing.db"), root, owner: None, secrets: RefCell::new(Vec::new()),
        };
        fixture.reopen();
        fixture.pair(CALLER, roles, scopes);
        fixture.pair(TARGET, &["operator", "node"], &["operator.pairing", "node.invoke"]);
        fixture.pair(BYSTANDER, &["node"], &["node.invoke"]);
        for device in [CALLER, TARGET, BYSTANDER] {
            let conn = fixture.writer();
            assert_eq!(conn.execute(
                "UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2
                 WHERE device_id = ?3",
                params![
                    json!({"device": device, "generation": 7, "commands": ["system.run"]}).to_string(),
                    json!({"device": device, "generation": 8, "commands": ["system.notify"]}).to_string(),
                    device,
                ],
            ).expect("seed both distinct retained surfaces"), 1);
            conn.close().expect("close surface fixture writer");
        }
        assert_eq!(fixture.request(CALLER, "repair", roles, scopes)["isRepair"], true);
        assert_eq!(fixture.request(TARGET, "repair", &["operator", "node"], &["operator.pairing", "node.invoke"])["isRepair"], true);
        assert_eq!(fixture.request("drt-unpaired", "pending", &["operator"], &["operator.read"])["isRepair"], false);
        let conn = fixture.writer();
        conn.execute(
            "INSERT INTO config_machine_state (state_key, value_json, updated_at_ms) VALUES (?1, ?2, ?3)",
            params!["drt.unrelatedSetting", "{\"preserve\":\"DRT-private-config-marker\"}", 41],
        ).expect("seed unrelated configuration");
        conn.close().expect("close configuration fixture writer");
        let state = fixture.observe();
        assert_eq!(state.paired.len(), 3);
        assert_eq!(state.pending.len(), 3);
        assert_eq!(state.revision(), 3);
        fixture
    }

    fn owner(&self) -> &DevicePairingOwner {
        self.owner.as_ref().expect("open production pairing owner")
    }

    fn reopen(&mut self) {
        drop(self.owner.take());
        self.owner = Some(DevicePairingOwner::new(&self.db).expect("open real durable pairing owner"));
    }

    fn writer(&self) -> Connection {
        Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_WRITE)
            .expect("open only this existing owned fixture for explicit injection")
    }

    fn observe(&self) -> Snapshot {
        let conn = Connection::open_with_flags(&self.db, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open an independent read-only observer");
        conn.execute_batch("BEGIN DEFERRED").expect("pin one observation snapshot");
        let state = Snapshot {
            paired: rows(&conn, "SELECT * FROM device_pairing_paired ORDER BY device_id"),
            pending: rows(&conn, "SELECT * FROM device_pairing_pending ORDER BY request_id"),
            machine: rows(&conn,
                "SELECT *, typeof(value_json) AS value_type, typeof(updated_at_ms) AS timestamp_type
                 FROM config_machine_state ORDER BY state_key"),
        };
        conn.execute_batch("COMMIT").expect("finish observation snapshot");
        conn.close().expect("close observer before mutation, reopen, or cleanup");
        state
    }

    fn request(&self, device: &str, key: &str, roles: &[&str], scopes: &[&str]) -> Value {
        self.owner().handle_pairing_command("device.pair.request", &json!({
            "deviceId": device, "publicKey": format!("{device}-{key}-public-key"),
            "displayName": format!("DRT {device}"), "platform": "linux",
            "role": roles[0], "roles": roles, "scopes": scopes,
        })).expect("request through the production public command")
    }

    fn pair(&self, device: &str, roles: &[&str], scopes: &[&str]) {
        let request = self.request(device, "original", roles, scopes);
        let receipt = self.owner().handle_pairing_command(
            "device.pair.approve", &json!({"requestId": request["requestId"]}),
        ).expect("approve and mint real credentials through the production command");
        assert_eq!(receipt["decision"], "approved");
        let state = self.observe();
        for role in roles {
            self.secrets.borrow_mut().push(state.token(device, role));
        }
    }

    fn proof(&self, device: &str, role: &str) -> DeviceTokenAuthorization {
        self.authenticate(&self.observe().token(device, role))
    }

    fn authenticate(&self, bearer: &str) -> DeviceTokenAuthorization {
        self.secrets.borrow_mut().push(bearer.to_owned());
        let proof = self.owner().authenticate_device_token(bearer)
            .expect("authenticate a real credential")
            .expect("production authentication must issue the proof");
        self.assert_secret_free(&format!("{proof:?}"));
        proof
    }

    fn native_rotate(&self, device: &str, role: &str) -> String {
        let receipt = self.owner().handle_pairing_command(
            "device.pair.rotateToken", &json!({"deviceId": device, "role": role}),
        ).expect("replace a credential through the native production command");
        assert_eq!(receipt["tokenDelivery"], "in-band");
        let token = receipt["token"].as_str().expect("native replacement token").to_owned();
        self.secrets.borrow_mut().push(token.clone());
        token
    }

    fn write_column(&self, device: &str, column: &str, value: &SqlValue) {
        assert!(matches!(column, "role" | "roles_json" | "scopes_json" | "approved_scopes_json" | "tokens_json"));
        let conn = self.writer();
        assert_eq!(conn.execute(
            &format!("UPDATE device_pairing_paired SET {column} = ?1 WHERE device_id = ?2"),
            params![value, device],
        ).expect("inject exactly one selected fixture field"), 1);
        conn.close().expect("close authority fixture writer");
    }

    fn edit_entries(&self, device: &str, edit: impl FnOnce(&mut Value)) {
        let mut entries = self.observe().entries(device);
        edit(&mut entries);
        self.write_column(device, "tokens_json", &SqlValue::Text(entries.to_string()));
    }

    fn grant(&self, device: &str, role: &str, scopes: &[&str]) {
        self.edit_entries(device, |entries| entries[role]["scopes"] = json!(scopes));
    }

    fn baseline(&self, device: &str, scopes: &[&str]) {
        self.write_column(device, "approved_scopes_json", &SqlValue::Text(json!(scopes).to_string()));
    }

    fn set_revision(&self, value: &SqlValue) {
        let conn = self.writer();
        assert_eq!(conn.execute(
            "UPDATE config_machine_state SET value_json = ?1, updated_at_ms = 1234 WHERE state_key = ?2",
            params![value, REVISION],
        ).expect("replace only the known fixture counter"), 1);
        conn.close().expect("close counter fixture writer");
    }

    fn assert_secret_free(&self, diagnostic: &str) {
        assert!(!diagnostic.contains("sand-device-tok-"), "diagnostic contains bearer material");
        for secret in self.secrets.borrow().iter() {
            assert!(!diagnostic.contains(secret), "diagnostic contains an issued bearer");
            let digest = Sha256::digest(secret.as_bytes());
            for fingerprint in [
                format!("{digest:x}"), format!("{digest:X}"), format!("{:?}", digest.to_vec()),
                base64::engine::general_purpose::STANDARD.encode(&digest),
                base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&digest),
            ] {
                assert!(!diagnostic.contains(&fingerprint), "diagnostic contains a sealed fingerprint");
            }
        }
    }

    fn error<T>(&self, result: Result<T, GatewayCommandError>, status: u16, state: &Snapshot) -> GatewayCommandError {
        let error = match result {
            Err(error) => error,
            Ok(_) => panic!("required refusal returned success"),
        };
        assert_eq!(error.status(), status, "typed refusal status");
        let diagnostic = format!("{error:?}\n{error}");
        self.assert_secret_free(&diagnostic);
        assert!(!diagnostic.contains("DRT-private"), "private storage witnesses must be redacted");
        for row in state.paired.iter().chain(&state.pending).chain(&state.machine) {
            for (column, value) in row {
                if column.ends_with("_json") {
                    if let SqlValue::Text(raw) = value {
                        if raw.len() > 2 {
                            assert!(!diagnostic.contains(raw), "diagnostic contains raw persisted authority");
                        }
                    }
                }
            }
        }
        error
    }

    fn deny(&self, proof: &DeviceTokenAuthorization, args: Value, ceiling: Option<&str>, status: u16) {
        let before = self.observe();
        self.error(self.owner().revoke_token_with_authority(&args, Some(proof), ceiling), status, &before);
        assert!(self.observe() == before, "refusal changed paired/pending/token/surface/configuration/revision state");
    }

    fn deny_native(&self, args: Value, ceiling: Option<&str>, status: u16) {
        let before = self.observe();
        self.error(self.owner().revoke_token_with_authority(&args, None, ceiling), status, &before);
        assert!(self.observe() == before, "native refusal changed durable state");
    }

    fn current(&self, proof: &DeviceTokenAuthorization) {
        let before = self.observe();
        self.owner().validate_current_device_authorization(proof)
            .expect("current retained credential must remain admitted");
        assert!(self.observe() == before, "currentness admission wrote durable state");
    }

    fn stale(&self, proof: &DeviceTokenAuthorization, status: u16) {
        let before = self.observe();
        self.error(self.owner().validate_current_device_authorization(proof), status, &before);
        assert!(self.observe() == before, "currentness refusal wrote durable state");
    }

    fn retired(&self, before: &Snapshot, receipt: &Value, device: &str, role: &str) {
        assert_eq!(receipt.as_object().expect("closed receipt object").len(), 3);
        assert_eq!(receipt["deviceId"], device);
        assert_eq!(receipt["role"], role);
        let timestamp = receipt["revokedAtMs"].as_i64().expect("signed integer retirement time");
        assert!(timestamp > 0);
        self.assert_secret_free(&receipt.to_string());
        let after = self.observe();
        assert_eq!(after.paired.len(), before.paired.len(), "retirement preserves every paired row");
        for previous in &before.paired {
            let id = text(previous, "device_id");
            let current = after.device(id);
            if id != device {
                assert!(current == previous, "unrelated device changed");
                continue;
            }
            for (column, old) in previous {
                if column == "tokens_json" { continue; }
                if role == "node" && matches!(column.as_str(), "node_surface_json" | "pending_node_surface_json") {
                    assert!(current.get(column) == Some(&SqlValue::Null), "selected node clears both surfaces");
                } else {
                    assert!(current.get(column) == Some(old), "unselected column changed: {column}");
                }
            }
            let old_entries = before.entries(device);
            let new_entries = after.entries(device);
            let old_map = old_entries.as_object().unwrap();
            let new_map = new_entries.as_object().unwrap();
            assert_eq!(new_map.len(), old_map.len(), "no role entry issued or removed");
            for (name, old) in old_map {
                let current = new_map.get(name).expect("every prior role survives");
                if name != role {
                    assert!(current == old, "unselected entry must survive even malformed");
                    continue;
                }
                let mut old_metadata = old.as_object().unwrap().clone();
                old_metadata.remove("revokedAtMs");
                let mut new_metadata = current.as_object().unwrap().clone();
                assert_eq!(new_metadata.remove("revokedAtMs"), Some(json!(timestamp)));
                assert!(new_metadata == old_metadata, "selected token/role/scopes/unknown metadata changed");
            }
        }
        assert!(after.pending == before.pending, "retirement changed pending requests");
        assert_eq!(after.machine.len(), before.machine.len());
        for row in &before.machine {
            let key = text(row, "state_key");
            if key != REVISION {
                assert!(after.machine.iter().find(|candidate| text(candidate, "state_key") == key) == Some(row));
            }
        }
        assert_eq!(after.revision(), before.revision() + 1);
        assert_eq!(text(after.counter(), "value_json"), format!("\"{}\"", before.revision() + 1));
        assert_eq!(text(after.counter(), "timestamp_type"), "integer");
        assert!(matches!(after.counter().get("updated_at_ms"), Some(SqlValue::Integer(value)) if *value > 0));
        self.token_absent(&before.token(device, role));
    }

    fn token_absent(&self, token: &str) {
        let before = self.observe();
        assert!(self.owner().authenticate_device_token(token)
            .expect("fresh lookup of retired bearer").is_none());
        assert!(self.observe() == before, "fresh refusal must not write state");
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.owner.take());
        if std::thread::panicking() {
            eprintln!("DRT fixture retained for Actions capture: {}", self.root.display());
        } else {
            fs::remove_dir_all(&self.root).expect("remove only this owned fixture after owner drop");
        }
    }
}

fn args(device: &str, role: &str) -> Value {
    json!({"deviceId": device, "role": role})
}

#[test]
fn role_retirement_pairing_only_self_preserves_mixed_pair_and_survives_reopen() {
    let mut fixture = Fixture::seeded("self", &["operator", "node"], &["operator.pairing", "node.invoke"]);
    let proof = fixture.proof(CALLER, "operator");
    let node = fixture.proof(CALLER, "node");
    let other = fixture.proof(BYSTANDER, "node");
    fixture.edit_entries(CALLER, |entries| {
        entries["operator"]["unknownMetadata"] = json!({"retain": [null, 7, "exact"]});
        entries["unselected-malformed"] = json!([false, 7]);
    });
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_token_with_authority(
        &args(&format!(" {CALLER}\t"), " operator "), Some(&proof), Some(" operator.pairing "),
    ).expect("pairing-only operator may retire its own operator token in a mixed pair");
    fixture.retired(&before, &receipt, CALLER, "operator");
    fixture.stale(&proof, 401);
    fixture.deny(&proof, args(CALLER, "operator"), None, 401);
    fixture.current(&node);
    fixture.current(&other);
    let committed = fixture.observe();
    fixture.reopen();
    assert!(fixture.observe() == committed);
    fixture.token_absent(&before.token(CALLER, "operator"));
    fixture.stale(&proof, 401);
    fixture.stale(&node, 401);
    fixture.current(&fixture.proof(CALLER, "node"));
    fixture.current(&fixture.proof(BYSTANDER, "node"));
}

#[test]
fn role_retirement_admin_repeats_node_retirement_and_clears_both_surfaces() {
    let fixture = Fixture::seeded("admin-node", &["operator"], &["operator.admin"]);
    let admin = fixture.proof(CALLER, "operator");
    let operator = fixture.proof(TARGET, "operator");
    let node = fixture.proof(TARGET, "node");
    let before = fixture.observe();
    let first = fixture.owner().revoke_token_with_authority(&args(TARGET, "node"), Some(&admin), None)
        .expect("admin-only scope implies pairing and permits another role");
    fixture.retired(&before, &first, TARGET, "node");
    fixture.stale(&node, 401);
    fixture.current(&operator);
    fixture.current(&admin);
    let writer = fixture.writer();
    writer.execute("UPDATE device_pairing_paired SET node_surface_json = 'regrown', pending_node_surface_json = 'pending-regrown' WHERE device_id = ?1", params![TARGET])
        .expect("inject surfaces to prove repeat also clears them");
    writer.close().unwrap();
    fixture.edit_entries(TARGET, |entries| entries["node"]["revokedAtMs"] = json!(1));
    let before_repeat = fixture.observe();
    let second = fixture.owner().revoke_token_with_authority(&args(TARGET, "node"), Some(&admin), Some("operator.admin"))
        .expect("separately current admin can explicitly repeat retirement");
    fixture.retired(&before_repeat, &second, TARGET, "node");
    assert!(second["revokedAtMs"].as_i64().unwrap() > 1, "repeat must rewrite the marker");
    fixture.current(&operator);
    fixture.current(&admin);
}

#[test]
fn role_retirement_native_dispatch_and_narrow_native_ceiling_obey_target_caps() {
    for ceiling in [None, Some("operator.admin"), Some(" operator.pairing ")] {
        let fixture = Fixture::seeded("native-pairing", &["operator"], &["operator.read"]);
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), None, ceiling)
            .expect("native pairing ceiling admits a pairing-only operator target");
        fixture.retired(&before, &receipt, TARGET, "operator");
        fixture.current(&fixture.proof(TARGET, "node"));
    }
    let fixture = Fixture::seeded("native-dispatch", &["operator"], &["operator.admin"]);
    let before = fixture.observe();
    let receipt = fixture.owner().handle_pairing_command("device.token.revoke", &args(TARGET, "node"))
        .expect("generic native dispatch uses selected-role owner");
    fixture.retired(&before, &receipt, TARGET, "node");
    for ceiling in ["", " \t", "operator.write", "operator.read", "node.invoke", "operator.pairing,operator.admin", "operator.admin operator.pairing", "operator.pairing.extra"] {
        fixture.deny_native(args(CALLER, "operator"), Some(ceiling), 403);
        fixture.deny_native(args(ABSENT, "operator"), Some(ceiling), 403);
    }
    for target in [TARGET, ABSENT] {
        fixture.deny_native(args(target, "node"), Some("operator.pairing"), 403);
    }
    fixture.deny_native(args(CALLER, "operator"), Some("operator.pairing"), 403);
    let before = fixture.observe();
    let receipt = fixture.owner().handle_token_revoke(&args(CALLER, "operator"))
        .expect("unrestricted native direct handler retires an admin grant");
    fixture.retired(&before, &receipt, CALLER, "operator");
}

#[test]
fn role_retirement_closed_request_rejects_types_missing_fields_and_assertions() {
    let fixture = Fixture::seeded("closed-request", &["operator"], &["operator.admin"]);
    let proof = fixture.proof(CALLER, "operator");
    let mut invalid = vec![Value::Null, json!([]), json!(true), json!(7), json!("request"), json!({}),
        json!({"deviceId": TARGET}), json!({"role": "operator"})];
    for field in ["deviceId", "role"] {
        for value in [Value::Null, json!(false), json!(7), json!([]), json!({}), json!(""), json!(" \t")] {
            let mut request = args(TARGET, "operator");
            request[field] = value;
            invalid.push(request);
        }
    }
    for extra in ["scopes", "crossDevice", "callerDeviceId", "callerRole", "callerScopes", "admin", "unknown"] {
        let mut request = args(TARGET, "operator");
        request[extra] = json!(true);
        invalid.push(request);
    }
    for request in invalid {
        fixture.deny(&proof, request.clone(), None, 400);
        fixture.deny_native(request, None, 400);
    }
    for (device, role) in [(ABSENT, "operator"), (TARGET, "Operator"), (TARGET, "observer")] {
        fixture.deny(&proof, args(device, role), None, 400);
        fixture.deny_native(args(device, role), None, 400);
    }
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&proof), None)
        .expect("same authority and valid closed shape is a positive control");
    fixture.retired(&before, &receipt, TARGET, "operator");
}

#[test]
fn role_retirement_target_nondisclosure_precedes_malformed_or_absent_target_reads() {
    let fixture = Fixture::seeded("target-address", &["operator", "node"], &["operator.pairing", "node.invoke"]);
    let proof = fixture.proof(CALLER, "operator");
    let healthy = fixture.observe();
    fixture.write_column(TARGET, "tokens_json", &SqlValue::Blob(vec![0xff, 7]));
    fixture.write_column(TARGET, "roles_json", &SqlValue::Null);
    for role in ["operator", "node", "nonexistent"] {
        let before = fixture.observe();
        let existing = fixture.error(fixture.owner().revoke_token_with_authority(&args(TARGET, role), Some(&proof), None), 403, &before);
        let absent = fixture.error(fixture.owner().revoke_token_with_authority(&args(ABSENT, role), Some(&proof), None), 403, &before);
        assert_eq!(existing.to_string(), absent.to_string(), "inaccessible existence must not alter refusal");
        assert!(fixture.observe() == before);
    }
    fixture.deny(&proof, args(CALLER, "node"), Some("operator.admin"), 403);
    // Restore the independently injected target corruption before checking fresh
    // lookup; fresh enumeration's preexisting SQL storage-error policy is separate.
    for column in ["tokens_json", "roles_json"] {
        fixture.write_column(TARGET, column, healthy.device(TARGET).get(column).unwrap());
    }
    assert!(fixture.observe() == healthy);
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_token_with_authority(&args(CALLER, "operator"), Some(&proof), None)
        .expect("same pairing-only caller still has its permitted self operation");
    fixture.retired(&before, &receipt, CALLER, "operator");
}

#[test]
fn role_retirement_pairing_permission_and_singular_headers_do_not_promote_proofs() {
    for grant in [&[][..], &["operator.read"][..], &["operator.write"][..], &["operator.approvals"][..], &["operator.pairing.extra"][..]] {
        let fixture = Fixture::seeded("missing-pairing", &["operator"], grant);
        let proof = fixture.proof(CALLER, "operator");
        fixture.current(&proof);
        fixture.deny(&proof, args(CALLER, "operator"), None, 403);
        fixture.deny(&proof, args(CALLER, "operator"), Some("operator.admin"), 403);
    }
    let fixture = Fixture::seeded("header-cap", &["operator"], &["operator.admin"]);
    let proof = fixture.proof(CALLER, "operator");
    for ceiling in ["", " \t", "operator.write", "operator.read", "operator.pairing,operator.admin", "operator.admin operator.pairing", "node.invoke"] {
        fixture.deny(&proof, args(TARGET, "operator"), Some(ceiling), 403);
    }
    fixture.deny(&proof, args(TARGET, "operator"), Some("operator.pairing"), 403);
    fixture.deny(&proof, args(CALLER, "operator"), Some("operator.pairing"), 403);
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&proof), Some(" operator.admin "))
        .expect("admin-only sealed grant and singular admin ceiling are sufficient");
    fixture.retired(&before, &receipt, TARGET, "operator");
}

#[test]
fn role_retirement_sealed_issuer_and_projection_control_identity_and_permissions() {
    let fixture = Fixture::seeded("sealed", &["operator"], &["operator.pairing"]);
    let bearer = fixture.observe().token(CALLER, "operator");
    let proof = fixture.authenticate(&bearer);
    let second = DevicePairingOwner::new(&fixture.db).unwrap();
    let foreign = second.authenticate_device_token(&bearer).unwrap().unwrap();
    assert_eq!(foreign.device_id, proof.device_id);
    assert_eq!(foreign.role, proof.role);
    assert_eq!(foreign.scopes, proof.scopes);
    fixture.deny(&foreign, args(CALLER, "operator"), None, 401);
    drop(second);
    let mut edited = proof.clone();
    edited.device_id = TARGET.into();
    edited.role = "node".into();
    edited.scopes = vec!["operator.admin".into()];
    fixture.deny(&edited, args(TARGET, "operator"), Some("operator.admin"), 403);
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_token_with_authority(&args(CALLER, "operator"), Some(&edited), None)
        .expect("sealed self identity and grant survive projection edits");
    fixture.retired(&before, &receipt, CALLER, "operator");
    fixture.stale(&proof, 401);
    fixture.stale(&edited, 401);
}

#[test]
fn role_retirement_currentness_precedes_operator_policy_for_every_sealed_role() {
    for (role, grant, additional) in [("node", "node.invoke", "node.observe"), ("observer", "observer.listen", "observer.read")] {
        let fixture = Fixture::seeded("currentness-order", &[role], &[grant, additional]);
        let proof = fixture.proof(CALLER, role);
        let healthy = fixture.observe();
        fixture.current(&proof);
        fixture.deny(&proof, args(TARGET, "operator"), None, 403);
        for baseline in [json!([]), json!([grant])] {
            fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Text(baseline.to_string()));
            fixture.deny(&proof, args(TARGET, "operator"), Some(""), 401);
            fixture.stale(&proof, 401);
            fixture.write_column(CALLER, "approved_scopes_json", healthy.device(CALLER).get("approved_scopes_json").unwrap());
        }
        for malformed in [Value::Null, json!(grant), json!([grant, false])] {
            fixture.edit_entries(CALLER, |entries| entries[role]["scopes"] = malformed);
            fixture.deny(&proof, args(TARGET, "operator"), Some(""), 500);
            fixture.stale(&proof, 500);
            fixture.write_column(CALLER, "tokens_json", healthy.device(CALLER).get("tokens_json").unwrap());
        }
        fixture.current(&proof);
        fixture.deny(&proof, args(TARGET, "operator"), None, 403);
        let successor = fixture.native_rotate(CALLER, role);
        fixture.deny(&proof, args(TARGET, "operator"), None, 401);
        fixture.token_absent(&healthy.token(CALLER, role));
        let current = fixture.authenticate(&successor);
        fixture.current(&current);
        fixture.deny(&current, args(TARGET, "operator"), None, 403);
    }
}

#[test]
fn role_retirement_caller_identity_failures_keep_established_unauthorized_status() {
    for fault in ["missing-entry", "missing-token", "token-type", "empty-token", "missing-role", "role-type", "wrong-role", "unapproved-role", "retired"] {
        let fixture = Fixture::seeded(fault, &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        if fault == "unapproved-role" {
            fixture.write_column(CALLER, "roles_json", &SqlValue::Text("[]".into()));
        } else {
            fixture.edit_entries(CALLER, |entries| match fault {
                "missing-entry" => { entries.as_object_mut().unwrap().remove("operator"); }
                "missing-token" => { entries["operator"].as_object_mut().unwrap().remove("token"); }
                "token-type" => entries["operator"]["token"] = json!(7),
                "empty-token" => entries["operator"]["token"] = json!(""),
                "missing-role" => { entries["operator"].as_object_mut().unwrap().remove("role"); }
                "role-type" => entries["operator"]["role"] = json!(7),
                "wrong-role" => entries["operator"]["role"] = json!("node"),
                "retired" => entries["operator"]["revokedAtMs"] = json!(1),
                _ => unreachable!(),
            });
        }
        fixture.deny(&proof, args(TARGET, "operator"), Some(""), 401);
        fixture.stale(&proof, 401);
    }
}

#[test]
fn role_retirement_caller_corruption_and_null_approval_fallback_are_distinct() {
    for (column, value) in [
        ("roles_json", SqlValue::Null), ("roles_json", SqlValue::Blob(vec![0xff])),
        ("roles_json", SqlValue::Text("{}".into())), ("roles_json", SqlValue::Text("[\"operator\",7]".into())),
        ("tokens_json", SqlValue::Text("DRT-private-invalid-map".into())),
        ("tokens_json", SqlValue::Text("[]".into())), ("tokens_json", SqlValue::Blob(vec![0xff])),
        ("approved_scopes_json", SqlValue::Text("null".into())),
        ("approved_scopes_json", SqlValue::Text("[\"operator.admin\",false]".into())),
        ("approved_scopes_json", SqlValue::Blob(vec![0xff])),
    ] {
        let fixture = Fixture::seeded("corrupt-caller", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(CALLER, column, &value);
        fixture.deny(&proof, args(TARGET, "operator"), None, 500);
    }
    for legacy in [SqlValue::Null, SqlValue::Text("not-json".into()), SqlValue::Blob(vec![0xff])] {
        let fixture = Fixture::seeded("corrupt-fallback", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Null);
        fixture.write_column(CALLER, "scopes_json", &legacy);
        fixture.deny(&proof, args(TARGET, "operator"), None, 500);
    }
    let fixture = Fixture::seeded("valid-fallback", &["operator"], &["operator.admin"]);
    let proof = fixture.proof(CALLER, "operator");
    fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Null);
    fixture.current(&proof);
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&proof), None)
        .expect("SQL NULL approval uses existing valid legacy scopes");
    fixture.retired(&before, &receipt, TARGET, "operator");
}

#[test]
fn role_retirement_intersects_sealed_live_approval_and_target_grants() {
    let fixture = Fixture::seeded("original-cap", &["operator"], &["operator.pairing"]);
    let proof = fixture.proof(CALLER, "operator");
    fixture.grant(CALLER, "operator", &["operator.admin"]);
    fixture.baseline(CALLER, &["operator.admin"]);
    fixture.deny(&proof, args(TARGET, "operator"), None, 403);
    fixture.deny(&proof, args(CALLER, "operator"), None, 403); // Cannot retire the now wider grant.
    let fresh = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&fresh), None)
        .expect("new authentication admits the approved wider grant");
    fixture.retired(&before, &receipt, TARGET, "operator");

    for narrow_approval in [false, true] {
        let fixture = Fixture::seeded("live-cap", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.grant(CALLER, "operator", &["operator.pairing"]);
        if narrow_approval { fixture.baseline(CALLER, &["operator.pairing"]); }
        fixture.deny(&proof, args(TARGET, "operator"), None, 403);
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_token_with_authority(&args(CALLER, "operator"), Some(&proof), None)
            .expect("existing management intersection semantics permit narrowed pairing-only self retirement");
        fixture.retired(&before, &receipt, CALLER, "operator");
    }
    let fixture = Fixture::seeded("withdrawn-approval", &["operator"], &["operator.pairing", "operator.approvals"]);
    let proof = fixture.proof(CALLER, "operator");
    fixture.baseline(CALLER, &["operator.pairing"]);
    fixture.deny(&proof, args(CALLER, "operator"), None, 401);
}

#[test]
fn role_retirement_target_scope_shape_is_strict_even_on_repeat() {
    for malformed in [None, Some(Value::Null), Some(json!("operator.pairing")), Some(json!({})), Some(json!(["operator.pairing", 7]))] {
        for retired in [false, true] {
            let fixture = Fixture::seeded("target-grant-shape", &["operator"], &["operator.admin"]);
            let proof = fixture.proof(CALLER, "operator");
            fixture.edit_entries(TARGET, |entries| {
                if let Some(value) = malformed.clone() { entries["operator"]["scopes"] = value; }
                else { entries["operator"].as_object_mut().unwrap().remove("scopes"); }
                if retired { entries["operator"]["revokedAtMs"] = json!(1); }
            });
            fixture.deny(&proof, args(TARGET, "operator"), None, 500);
            fixture.deny_native(args(TARGET, "operator"), None, 500);
        }
    }
    for scopes in [json!([]), json!([" operator.pairing ", "", "operator.pairing", " \t"])] {
        let fixture = Fixture::seeded("target-grant-valid", &["operator"], &["operator.admin"]);
        fixture.edit_entries(TARGET, |entries| entries["operator"]["scopes"] = scopes);
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), None, Some("operator.pairing"))
            .expect("empty and normalized narrow grants fit the native pairing ceiling");
        fixture.retired(&before, &receipt, TARGET, "operator");
    }
}

#[test]
fn role_retirement_entire_target_grant_cap_applies_to_active_and_retired_entries() {
    for scopes in [json!(["operator.admin"]), json!(["operator.pairing", "operator.write"]), json!(["node.invoke"])] {
        for retired in [false, true] {
            let fixture = Fixture::seeded("entire-target-cap", &["operator"], &["operator.admin"]);
            fixture.edit_entries(TARGET, |entries| {
                entries["operator"]["scopes"] = scopes.clone();
                if retired { entries["operator"]["revokedAtMs"] = json!(1); }
            });
            fixture.deny_native(args(TARGET, "operator"), Some("operator.pairing"), 403);
            let admin = fixture.proof(CALLER, "operator");
            if scopes == json!(["node.invoke"]) {
                fixture.deny(&admin, args(TARGET, "operator"), None, 403);
                fixture.deny_native(args(TARGET, "operator"), None, 403);
            } else {
                let before = fixture.observe();
                let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&admin), None)
                    .expect("admin ceiling permits the valid entire operator grant");
                fixture.retired(&before, &receipt, TARGET, "operator");
            }
        }
    }
}

#[test]
fn role_retirement_target_approval_and_nonoperator_scopes_are_not_issuance_prerequisites() {
    for role in ["operator", "node"] {
        let fixture = Fixture::seeded("target-not-issuance", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(TARGET, "scopes_json", &SqlValue::Blob(vec![0xff]));
        fixture.write_column(TARGET, "approved_scopes_json", &SqlValue::Text("DRT-private-invalid-approval".into()));
        if role == "node" {
            fixture.edit_entries(TARGET, |entries| entries["node"]["scopes"] = json!({"not": "an issuance grant"}));
        }
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, role), Some(&proof), None)
            .expect("separately current admin can retire despite withdrawn target issuance baseline");
        fixture.retired(&before, &receipt, TARGET, role);
        fixture.current(&proof);
    }
}

#[test]
fn role_retirement_target_role_selection_unions_strict_array_with_optional_primary() {
    for (roles, primary, selected) in [
        (json!([]), SqlValue::Text(" node ".into()), "node"),
        (json!([" operator ", "", "operator"]), SqlValue::Null, "operator"),
        (json!(["node"]), SqlValue::Text(" \t".into()), "node"),
        (json!(["node"]), SqlValue::Text("operator".into()), "operator"),
    ] {
        let fixture = Fixture::seeded("role-union", &["operator"], &["operator.admin"]);
        let admin = fixture.proof(CALLER, "operator");
        fixture.write_column(TARGET, "roles_json", &SqlValue::Text(roles.to_string()));
        fixture.write_column(TARGET, "role", &primary);
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, selected), Some(&admin), None)
            .expect("normalized target array plus optional primary form the selected-role union");
        fixture.retired(&before, &receipt, TARGET, selected);
    }
    let fixture = Fixture::seeded("no-inferred-role", &["operator"], &["operator.admin"]);
    let admin = fixture.proof(CALLER, "operator");
    fixture.write_column(TARGET, "roles_json", &SqlValue::Text("[]".into()));
    fixture.write_column(TARGET, "role", &SqlValue::Null);
    fixture.deny(&admin, args(TARGET, "operator"), None, 400);
    fixture.deny_native(args(TARGET, "node"), None, 400);
    fixture.write_column(TARGET, "roles_json", &SqlValue::Text("[\"operator\"]".into()));
    fixture.edit_entries(TARGET, |entries| { entries.as_object_mut().unwrap().remove("operator"); });
    fixture.deny(&admin, args(TARGET, "operator"), None, 400);
}

#[test]
fn role_retirement_required_target_storage_corruption_is_redacted_and_atomic() {
    for (column, raw) in [
        ("roles_json", SqlValue::Null), ("roles_json", SqlValue::Blob(vec![0xff, 7])),
        ("roles_json", SqlValue::Text("DRT-private-invalid-roles".into())),
        ("roles_json", SqlValue::Text("null".into())), ("roles_json", SqlValue::Text("{}".into())),
        ("roles_json", SqlValue::Text("[\"operator\",7]".into())),
        ("role", SqlValue::Blob(vec![0xff, 8])),
        ("tokens_json", SqlValue::Blob(vec![0xff, 9])),
        ("tokens_json", SqlValue::Text("DRT-private-invalid-map".into())),
        ("tokens_json", SqlValue::Text("null".into())), ("tokens_json", SqlValue::Text("[]".into())),
    ] {
        let fixture = Fixture::seeded("corrupt-target", &["operator"], &["operator.admin"]);
        let admin = fixture.proof(CALLER, "operator");
        fixture.write_column(TARGET, column, &raw);
        fixture.deny(&admin, args(TARGET, "operator"), None, 500);
        fixture.deny_native(args(TARGET, "operator"), None, 500);
    }
    for malformed in [Value::Null, json!([]), json!(7), json!({}),
        json!({"role": "operator", "token": ""}),
        json!({"role": "operator", "token": 7}),
        json!({"role": "node", "token": "DRT-private-selected-token"}),
        json!({"role": false, "token": "DRT-private-selected-token"})] {
        let fixture = Fixture::seeded("corrupt-entry", &["operator"], &["operator.admin"]);
        let admin = fixture.proof(CALLER, "operator");
        fixture.edit_entries(TARGET, |entries| entries["operator"] = malformed);
        fixture.deny(&admin, args(TARGET, "operator"), None, 500);
        fixture.deny_native(args(TARGET, "operator"), None, 500);
    }
}

#[test]
fn role_retirement_fresh_and_retained_admission_share_all_marker_variants() {
    let variants = [
        (None, true), (Some(Value::Null), true), (Some(json!(0)), true),
        (Some(json!(1)), false), (Some(json!(-1)), false), (Some(json!(0.0)), false),
        (Some(json!(1.5)), false), (Some(json!("0")), false), (Some(json!("")), false),
        (Some(json!(false)), false), (Some(json!(true)), false), (Some(json!([])), false),
        (Some(json!({})), false), (Some(json!(u64::MAX)), false),
    ];
    for role in ["operator", "node"] {
        for (marker, active) in &variants {
            let fixture = Fixture::seeded("marker", &["operator", "node"], &["operator.pairing", "node.invoke"]);
            let original = fixture.observe();
            let bearer = original.token(CALLER, role);
            let proof = fixture.authenticate(&bearer);
            let unaffected_role = if role == "operator" { "node" } else { "operator" };
            let unaffected = fixture.proof(CALLER, unaffected_role);
            fixture.edit_entries(CALLER, |entries| {
                if let Some(marker) = marker { entries[role]["revokedAtMs"] = marker.clone(); }
                else { entries[role].as_object_mut().unwrap().remove("revokedAtMs"); }
            });
            let before = fixture.observe();
            if *active {
                let fresh = fixture.authenticate(&bearer);
                fixture.current(&fresh);
                fixture.current(&proof);
            } else {
                fixture.token_absent(&bearer);
                fixture.stale(&proof, 401);
                fixture.deny(&proof, args(TARGET, "operator"), None, 401);
            }
            fixture.current(&unaffected);
            fixture.current(&fixture.proof(BYSTANDER, "node"));
            assert!(fixture.observe() == before, "marker validation must remain read-only");
        }
    }
}

#[test]
fn role_retirement_rotation_and_reapproval_issue_successors_without_reviving_old_bearers() {
    let mut fixture = Fixture::seeded("successors", &["operator"], &["operator.admin"]);
    let admin = fixture.proof(CALLER, "operator");
    let original = fixture.observe().token(TARGET, "operator");
    let proof = fixture.authenticate(&original);
    fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&admin), None).unwrap();
    fixture.stale(&proof, 401);
    let successor = fixture.native_rotate(TARGET, "operator");
    assert!(successor != original);
    fixture.token_absent(&original);
    let successor_proof = fixture.authenticate(&successor);
    fixture.current(&successor_proof);
    fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&admin), None).unwrap();
    fixture.stale(&successor_proof, 401);
    let before = fixture.observe();
    let request = before.pending.iter().find(|row| text(row, "device_id") == TARGET).unwrap();
    fixture.owner().handle_pair_approve(&json!({"requestId": text(request, "request_id")}))
        .expect("explicit approval may mint another generation");
    let latest = fixture.observe().token(TARGET, "operator");
    assert!(latest != successor && latest != original);
    fixture.current(&fixture.authenticate(&latest));
    let committed = fixture.observe();
    fixture.reopen();
    assert!(fixture.observe() == committed);
    fixture.token_absent(&original);
    fixture.token_absent(&successor);
    fixture.stale(&admin, 401);
    fixture.current(&fixture.authenticate(&latest));
}

#[test]
fn role_retirement_revision_missing_corrupt_and_overflow_roll_back_on_reopen() {
    for raw in [None, Some(SqlValue::Text("DRT-private-invalid-revision".into())),
        Some(SqlValue::Text("3.5".into())), Some(SqlValue::Blob(vec![0xff, 0, 9])),
        Some(SqlValue::Text("\"18446744073709551616\"".into())),
        Some(SqlValue::Text(format!("\"{}\"", u64::MAX)))] {
        let mut fixture = Fixture::seeded("revision", &["operator"], &["operator.admin"]);
        let admin = fixture.proof(CALLER, "operator");
        let healthy = fixture.observe();
        if let Some(raw) = &raw { fixture.set_revision(raw); }
        else {
            let writer = fixture.writer();
            assert_eq!(writer.execute("DELETE FROM config_machine_state WHERE state_key = ?1", params![REVISION]).unwrap(), 1);
            writer.close().unwrap();
        }
        let broken = fixture.observe();
        fixture.deny(&admin, args(TARGET, "node"), None, 500);
        fixture.deny_native(args(TARGET, "operator"), None, 500);
        fixture.current(&admin);
        fixture.reopen();
        assert!(fixture.observe() == broken, "all failed token/surface/revision writes roll back durably");
        fixture.stale(&admin, 401);
        let fresh = fixture.proof(CALLER, "operator");
        fixture.deny(&fresh, args(TARGET, "node"), None, 500);
        let writer = fixture.writer();
        let counter = healthy.counter();
        writer.execute("INSERT INTO config_machine_state(state_key, value_json, updated_at_ms) VALUES (?1, ?2, ?3) ON CONFLICT(state_key) DO UPDATE SET value_json = excluded.value_json, updated_at_ms = excluded.updated_at_ms",
            params![REVISION, counter.get("value_json").unwrap(), counter.get("updated_at_ms").unwrap()]).unwrap();
        writer.close().unwrap();
        assert!(fixture.observe() == healthy, "controlled counter restoration returns the complete original state");
        let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "node"), Some(&fresh), None)
            .expect("explicit repaired-counter control commits once");
        fixture.retired(&healthy, &receipt, TARGET, "node");
    }
}

#[test]
fn role_retirement_last_revision_and_legacy_bootstrap_behaviors_remain_distinct() {
    let fixture = Fixture::seeded("last-counter", &["operator"], &["operator.admin"]);
    fixture.set_revision(&SqlValue::Text(format!("\"{}\"", u64::MAX - 1)));
    let before = fixture.observe();
    let receipt = fixture.owner().handle_token_revoke(&args(TARGET, "node"))
        .expect("last representable increment may commit");
    fixture.retired(&before, &receipt, TARGET, "node");
    assert_eq!(fixture.observe().revision(), u64::MAX);
    fixture.deny_native(args(TARGET, "operator"), None, 500);

    let legacy = Fixture::seeded("legacy-bootstrap", &["operator"], &["operator.admin"]);
    let writer = legacy.writer();
    writer.execute("DELETE FROM config_machine_state WHERE state_key = ?1", params![REVISION]).unwrap();
    writer.close().unwrap();
    legacy.deny_native(args(TARGET, "operator"), None, 500);
    legacy.native_rotate(TARGET, "operator");
    assert_eq!(legacy.observe().revision(), 1, "preexisting rotation still bootstraps its absent counter");
}

#[test]
fn role_retirement_sql_and_row_count_faults_roll_back_then_recover() {
    for fault in ["zero-target", "target-error", "zero-revision", "revision-error"] {
        let mut fixture = Fixture::seeded(fault, &["operator"], &["operator.admin"]);
        let admin = fixture.proof(CALLER, "operator");
        let sql = match fault {
            "zero-target" => "CREATE TRIGGER drt_fault BEFORE UPDATE OF tokens_json ON device_pairing_paired WHEN OLD.device_id = 'drt-target' BEGIN SELECT RAISE(IGNORE); END;",
            "target-error" => "CREATE TRIGGER drt_fault BEFORE UPDATE OF tokens_json ON device_pairing_paired WHEN OLD.device_id = 'drt-target' BEGIN SELECT RAISE(ABORT, 'DRT-private-storage-fault'); END;",
            "zero-revision" => "CREATE TRIGGER drt_fault BEFORE UPDATE OF value_json ON config_machine_state WHEN OLD.state_key = 'devicePairing.revision' BEGIN SELECT RAISE(IGNORE); END;",
            "revision-error" => "CREATE TRIGGER drt_fault BEFORE UPDATE OF value_json ON config_machine_state WHEN OLD.state_key = 'devicePairing.revision' BEGIN SELECT RAISE(ABORT, 'DRT-private-storage-fault'); END;",
            _ => unreachable!(),
        };
        let writer = fixture.writer();
        writer.execute_batch(sql).unwrap();
        writer.close().unwrap();
        let before = fixture.observe();
        fixture.deny(&admin, args(TARGET, "node"), None, 500);
        fixture.deny_native(args(TARGET, "operator"), None, 500);
        fixture.current(&admin);
        fixture.current(&fixture.proof(TARGET, "node"));
        fixture.reopen();
        assert!(fixture.observe() == before);
        let fresh = fixture.proof(CALLER, "operator");
        fixture.deny(&fresh, args(TARGET, "node"), None, 500);
        let writer = fixture.writer();
        writer.execute_batch("DROP TRIGGER drt_fault").unwrap();
        writer.close().unwrap();
        let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "node"), Some(&fresh), None)
            .expect("healthy control after removing precisely the owned fault");
        fixture.retired(&before, &receipt, TARGET, "node");
    }
}

#[test]
fn role_retirement_immediate_write_lock_precedes_policy_and_target_read_decisions() {
    let fixture = Fixture::seeded("reserved-lock", &["operator"], &["operator.read"]);
    let proof = fixture.proof(CALLER, "operator");
    let writer = fixture.writer();
    let journal: String = writer.query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0)).unwrap();
    assert_eq!(journal, "delete");
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    fixture.current(&proof); // A retained read must work under a RESERVED writer.
    let before = fixture.observe();
    for result in [
        fixture.owner().revoke_token_with_authority(&args(CALLER, "operator"), Some(&proof), None),
        fixture.owner().revoke_token_with_authority(&args(ABSENT, "operator"), None, None),
    ] {
        let error = fixture.error(result, 500, &before);
        assert_eq!(error.to_string(), "failed to begin token revocation transaction");
    }
    assert!(fixture.observe() == before);
    writer.execute_batch("ROLLBACK").unwrap();
    writer.close().unwrap();
    fixture.deny(&proof, args(CALLER, "operator"), None, 403);
    fixture.deny_native(args(ABSENT, "operator"), None, 400);
    let receipt = fixture.owner().handle_token_revoke(&args(TARGET, "operator"))
        .expect("same owner commits after only the blocking connection is released");
    fixture.retired(&before, &receipt, TARGET, "operator");
}

#[test]
fn role_retirement_failed_commit_rolls_back_token_surfaces_and_counter_durably() {
    for paired in [false, true] {
        let mut fixture = Fixture::seeded("commit-failure", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        let writer = fixture.writer();
        let journal: String = writer.query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0)).unwrap();
        assert_eq!(journal, "delete");
        writer.close().unwrap();
        let before = fixture.observe();
        let reader = Connection::open_with_flags(&fixture.db, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        reader.execute_batch("BEGIN DEFERRED").unwrap();
        let count: i64 = reader.query_row("SELECT COUNT(*) FROM device_pairing_paired", [], |row| row.get(0)).unwrap();
        assert_eq!(count, 3, "take a SHARED lock that permits IMMEDIATE but obstructs COMMIT");
        let result = fixture.owner().revoke_token_with_authority(
            &args(TARGET, "node"), if paired { Some(&proof) } else { None }, None,
        );
        let error = fixture.error(result, 500, &before);
        assert_eq!(error.to_string(), "failed to commit token revocation transaction", "failure must reach COMMIT");
        reader.execute_batch("COMMIT").unwrap();
        reader.close().unwrap();
        assert!(fixture.observe() == before, "known failed COMMIT rolls back every durable write");
        fixture.current(&proof);
        fixture.current(&fixture.proof(TARGET, "node"));
        fixture.reopen();
        assert!(fixture.observe() == before, "complete rollback survives actual owner reopen");
        let fresh = fixture.proof(CALLER, "operator");
        let receipt = fixture.owner().revoke_token_with_authority(
            &args(TARGET, "node"), if paired { Some(&fresh) } else { None }, None,
        ).expect("explicit recovery after reader close commits exactly once");
        fixture.retired(&before, &receipt, TARGET, "node");
    }
}

#[test]
fn role_retirement_two_owner_self_race_commits_once_and_rejects_retired_proof() {
    let fixture = Fixture::seeded("two-owner-self", &["operator", "node"], &["operator.pairing", "node.invoke"]);
    let first_owner = fixture.owner();
    let second_owner = DevicePairingOwner::new(&fixture.db).unwrap();
    let before = fixture.observe();
    let bearer = before.token(CALLER, "operator");
    let first_proof = fixture.authenticate(&bearer);
    let second_proof = second_owner.authenticate_device_token(&bearer).unwrap().unwrap();
    let start = Barrier::new(3);
    let results = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            start.wait();
            first_owner.revoke_token_with_authority(&args(CALLER, "operator"), Some(&first_proof), None)
        });
        let second = scope.spawn(|| {
            start.wait();
            second_owner.revoke_token_with_authority(&args(CALLER, "operator"), Some(&second_proof), None)
        });
        start.wait();
        (first.join().unwrap(), second.join().unwrap())
    });
    let (receipt, error) = match results {
        (Ok(receipt), Err(error)) | (Err(error), Ok(receipt)) => (receipt, error),
        _ => panic!("exactly one of two independent owners must commit self retirement"),
    };
    fixture.error::<()>(Err(error), 401, &before);
    fixture.retired(&before, &receipt, CALLER, "operator");
    fixture.stale(&first_proof, 401);
    fixture.error(second_owner.validate_current_device_authorization(&second_proof), 401, &before);
    fixture.current(&fixture.proof(CALLER, "node"));
    drop(second_owner);
}

#[test]
fn role_retirement_two_owner_rotation_race_has_one_credential_generation_winner() {
    let fixture = Fixture::seeded("rotate-race", &["operator", "node"], &["operator.admin", "node.invoke"]);
    let revoker = fixture.owner();
    let rotator = DevicePairingOwner::new(&fixture.db).unwrap();
    let before = fixture.observe();
    let bearer = before.token(CALLER, "operator");
    let revoke_proof = fixture.authenticate(&bearer);
    let rotate_proof = rotator.authenticate_device_token(&bearer).unwrap().unwrap();
    let start = Barrier::new(3);
    let (retirement, rotation) = std::thread::scope(|scope| {
        let retire = scope.spawn(|| {
            start.wait();
            revoker.revoke_token_with_authority(&args(CALLER, "operator"), Some(&revoke_proof), None)
        });
        let rotate = scope.spawn(|| {
            start.wait();
            rotator.rotate_token_for_authenticated_device(&args(CALLER, "operator"), &rotate_proof, None)
        });
        start.wait();
        (retire.join().unwrap(), rotate.join().unwrap())
    });
    match (retirement, rotation) {
        (Ok(receipt), Err(error)) => {
            fixture.error::<()>(Err(error), 401, &before);
            fixture.retired(&before, &receipt, CALLER, "operator");
        }
        (Err(error), Ok(receipt)) => {
            fixture.error::<()>(Err(error), 401, &before);
            let after = fixture.observe();
            assert_eq!(after.revision(), before.revision() + 1);
            assert!(after.pending == before.pending);
            assert!(after.device(TARGET) == before.device(TARGET));
            assert!(after.device(BYSTANDER) == before.device(BYSTANDER));
            let successor = receipt["token"].as_str().expect("successful self rotation returns its new credential");
            assert!(successor != bearer);
            assert!(after.token(CALLER, "operator") == successor);
            assert!(after.entries(CALLER)["operator"].get("revokedAtMs").is_none(), "losing revoke cannot retire the successor");
            fixture.current(&fixture.authenticate(successor));
            fixture.token_absent(&bearer);
        }
        _ => panic!("rotation and self retirement cannot both consume the same original proof"),
    }
    fixture.stale(&revoke_proof, 401);
    fixture.error(rotator.validate_current_device_authorization(&rotate_proof), 401, &before);
    fixture.current(&fixture.proof(CALLER, "node"));
    drop(rotator);
}

#[test]
fn role_retirement_two_connections_preserve_both_distinct_selected_updates() {
    let fixture = Fixture::seeded("two-roles", &["operator"], &["operator.admin"]);
    let first_owner = fixture.owner();
    let second_owner = DevicePairingOwner::new(&fixture.db).unwrap();
    let before = fixture.observe();
    let bearer = before.token(CALLER, "operator");
    let first_proof = fixture.authenticate(&bearer);
    let second_proof = second_owner.authenticate_device_token(&bearer).unwrap().unwrap();
    let start = Barrier::new(3);
    let (operator, node) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            start.wait();
            first_owner.revoke_token_with_authority(&args(TARGET, "operator"), Some(&first_proof), None)
        });
        let second = scope.spawn(|| {
            start.wait();
            second_owner.revoke_token_with_authority(&args(TARGET, "node"), Some(&second_proof), None)
        });
        start.wait();
        (first.join().unwrap().expect("operator retirement must commit"), second.join().unwrap().expect("node retirement must commit"))
    });
    let after = fixture.observe();
    assert_eq!(after.revision(), before.revision() + 2);
    assert!(after.pending == before.pending);
    assert!(after.device(CALLER) == before.device(CALLER));
    assert!(after.device(BYSTANDER) == before.device(BYSTANDER));
    let entries = after.entries(TARGET);
    for (role, receipt) in [("operator", operator), ("node", node)] {
        assert_eq!(receipt.as_object().unwrap().len(), 3);
        assert_eq!(entries[role]["revokedAtMs"], receipt["revokedAtMs"]);
        assert!(receipt["revokedAtMs"].as_i64().is_some_and(|value| value > 0));
        assert!(after.token(TARGET, role) == before.token(TARGET, role));
        fixture.token_absent(&before.token(TARGET, role));
    }
    assert!(after.device(TARGET).get("node_surface_json") == Some(&SqlValue::Null));
    assert!(after.device(TARGET).get("pending_node_surface_json") == Some(&SqlValue::Null));
    fixture.current(&first_proof);
    second_owner.validate_current_device_authorization(&second_proof).unwrap();
    drop(second_owner);
}

#[test]
fn role_retirement_storage_read_failure_is_redacted_without_repair_or_mutation() {
    let fixture = Fixture::seeded("missing-table", &["operator"], &["operator.admin"]);
    let proof = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let writer = fixture.writer();
    writer.execute_batch("ALTER TABLE device_pairing_paired RENAME TO drt_owned_unavailable_pairings").unwrap();
    writer.close().unwrap();
    for result in [
        fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&proof), None),
        fixture.owner().handle_token_revoke(&args(TARGET, "operator")),
    ] {
        let error = fixture.error(result, 500, &before);
        assert_eq!(error.to_string(), "stored pairing authority is invalid");
    }
    let writer = fixture.writer();
    writer.execute_batch("ALTER TABLE drt_owned_unavailable_pairings RENAME TO device_pairing_paired").unwrap();
    writer.close().unwrap();
    assert!(fixture.observe() == before);
    fixture.current(&proof);
    let receipt = fixture.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&proof), None)
        .expect("restored table permits the unchanged caller authority");
    fixture.retired(&before, &receipt, TARGET, "operator");
}
