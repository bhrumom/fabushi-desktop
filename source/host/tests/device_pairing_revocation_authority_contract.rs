//! DVR-v1 / DFA-v1 owner contracts: production authentication/mutations, real SQLite files,
//! and independent complete-row observations. No HTTP or device acceptance claim.
//! Spec: projects/openclaw-rust/docs/specs/device-pairing-revocation-continuity.md.
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

const CALLER: &str = "dvr-caller";
const TARGET: &str = "dvr-target";
const BYSTANDER: &str = "dvr-bystander";
const ABSENT: &str = "dvr-absent";
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
        let root = std::env::temp_dir().join(format!("fabushi-dvr-{label}-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusively create this owned UUID directory");
        let mut fixture = Self {
            db: root.join("pairing.db"), root, owner: None, secrets: RefCell::new(Vec::new()),
        };
        fixture.reopen();
        fixture.pair(CALLER, roles, scopes);
        fixture.pair(TARGET, &["operator", "node"], &["operator.write", "node.invoke"]);
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
        assert_eq!(fixture.request(TARGET, "repair", &["operator", "node"], &["operator.write", "node.invoke"])["isRepair"], true);
        assert_eq!(fixture.request("dvr-unpaired", "pending", &["operator"], &["operator.read"])["isRepair"], false);
        let conn = fixture.writer();
        conn.execute(
            "INSERT INTO config_machine_state (state_key, value_json, updated_at_ms) VALUES (?1, ?2, ?3)",
            params!["dvr.unrelatedSetting", "{\"preserve\":\"DVR-private-config-marker\"}", 41],
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
            "displayName": format!("DVR {device}"), "platform": "linux",
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
        assert!(!diagnostic.contains("DVR-private-storage-fault"), "SQL fault payload must be redacted");
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
        self.error(self.owner().revoke_pairing_for_authenticated_device(&args, proof, ceiling), status, &before);
        assert!(self.observe() == before, "refusal changed paired/pending/token/surface/configuration/revision state");
    }

    fn deny_native(&self, args: Value, status: u16) {
        let before = self.observe();
        self.error(self.owner().handle_pairing_command("device.pair.revoke", &args), status, &before);
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

    fn removed(&self, before: &Snapshot, receipt: &Value, device: &str, label: &str, revision: u64) {
        assert_eq!(receipt["deviceId"], device);
        assert_eq!(receipt["role"], label);
        assert_eq!(receipt["revision"].as_u64(), Some(revision));
        assert!(receipt["revokedAtMs"].as_i64().is_some_and(|value| value > 0));
        assert!(receipt.get("token").is_none());
        self.assert_secret_free(&receipt.to_string());
        let after = self.observe();
        assert_eq!(after.paired.len() + 1, before.paired.len());
        let remaining: Vec<_> = before.paired.iter().filter(|row| text(row, "device_id") != device).cloned().collect();
        assert!(after.paired == remaining, "whole target deletion must preserve every other paired column");
        assert!(after.pending == before.pending, "native pending repairs and unrelated pending requests must survive");
        let old_config: Vec<_> = before.machine.iter().filter(|row| text(row, "state_key") != REVISION).collect();
        let new_config: Vec<_> = after.machine.iter().filter(|row| text(row, "state_key") != REVISION).collect();
        assert!(new_config == old_config, "removal changed unrelated configuration");
        assert_eq!(after.machine.len(), before.machine.len());
        assert_eq!(text(after.counter(), "value_type"), "text");
        assert_eq!(text(after.counter(), "timestamp_type"), "integer");
        assert_eq!(text(after.counter(), "value_json"), format!("\"{revision}\""));
        assert!(matches!(after.counter().get("updated_at_ms"), Some(SqlValue::Integer(value)) if *value > 0));
    }

    fn token_absent(&self, token: &str) {
        assert!(self.owner().authenticate_device_token(token)
            .expect("authenticate a removed or replaced credential").is_none());
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.owner.take());
        if std::thread::panicking() {
            eprintln!("DVR fixture retained for Actions capture: {}", self.root.display());
        } else {
            fs::remove_dir_all(&self.root).expect("remove only this owned UUID fixture after owner drop");
        }
    }
}

#[test]
fn paired_revocation_operator_self_commits_and_preserves_repairs_on_reopen() {
    let mut fixture = Fixture::seeded("self", &["operator"], &["operator.write"]);
    let proof = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let original = before.token(CALLER, "operator");
    let survivor = fixture.proof(BYSTANDER, "node");
    let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": format!("  {CALLER}  ")}), &proof, None,
    ).expect("a current writer may remove its own operator-only pairing");
    fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
    fixture.token_absent(&original);
    fixture.stale(&proof, 401);
    fixture.current(&survivor);
    fixture.deny(&proof, json!({"deviceId": CALLER}), None, 401);
    fixture.deny_native(json!({"deviceId": CALLER}), 400);
    let committed = fixture.observe();
    fixture.reopen();
    assert!(fixture.observe() == committed, "committed deletion and every pending repair survive reopen");
    fixture.token_absent(&original);
    fixture.stale(&proof, 401);
    fixture.current(&fixture.proof(BYSTANDER, "node"));
    fixture.deny_native(json!({"deviceId": CALLER}), 400);
}

#[test]
fn paired_revocation_admin_only_grant_removes_mixed_and_corrupt_targets() {
    for target in [CALLER, TARGET] {
        let fixture = Fixture::seeded("admin", &["operator", "node"], &["operator.admin", "node.invoke"]);
        let proof = fixture.proof(CALLER, "operator");
        assert_eq!(proof.scopes, vec!["operator.admin"]);
        let before = fixture.observe();
        let old_operator = before.token(target, "operator");
        let old_node = before.token(target, "node");
        let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
            &json!({"deviceId": target, "role": "operator"}), &proof, None,
        ).expect("operator admin alone implies write and authorizes whole mixed-pairing removal");
        fixture.removed(&before, &receipt, target, "operator", before.revision() + 1);
        fixture.token_absent(&old_operator);
        fixture.token_absent(&old_node);
    }
    for malformed in [SqlValue::Text("DVR-private-corrupt-target-tokens".into()), SqlValue::Blob(vec![0xff, 0, 7])] {
        let fixture = Fixture::seeded("admin-corrupt-target", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(TARGET, "tokens_json", &malformed);
        fixture.write_column(TARGET, "roles_json", &SqlValue::Blob(vec![0xff, 8]));
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
            &json!({"deviceId": TARGET}), &proof, None,
        ).expect("admin removal need not decode the target's corrupt authority payload");
        fixture.removed(&before, &receipt, TARGET, "all", before.revision() + 1);
        fixture.current(&proof);
    }
}

#[test]
fn native_revocation_labels_keep_whole_row_semantics_and_surviving_repair() {
    for (role, label) in [
        (None, "all"), (Some(json!("operator")), "operator"),
        (Some(json!("node")), "node"), (Some(json!("")), ""), (Some(json!(false)), "all"),
    ] {
        let mut fixture = Fixture::seeded("native-label", &["operator", "node"], &["operator.write", "node.invoke"]);
        let before = fixture.observe();
        let old_operator = before.token(CALLER, "operator");
        let old_node = before.token(CALLER, "node");
        let repair = before.pending.iter().find(|row| text(row, "device_id") == CALLER).expect("surviving repair");
        let request_id = text(repair, "request_id").to_owned();
        let mut args = json!({"deviceId": CALLER});
        if let Some(role) = role { args["role"] = role; }
        let receipt = fixture.owner().handle_pairing_command("device.pair.revoke", &args)
            .expect("trusted native removal keeps its established receipt-label contract");
        fixture.removed(&before, &receipt, CALLER, label, before.revision() + 1);
        fixture.token_absent(&old_operator);
        fixture.token_absent(&old_node);
        let committed = fixture.observe();
        fixture.reopen();
        assert!(fixture.observe() == committed);
        fixture.deny_native(args, 400);
        let approval = fixture.owner().handle_pairing_command(
            "device.pair.approve", &json!({"requestId": request_id}),
        ).expect("removal is not a ban: native approval may consume the surviving repair");
        assert_eq!(approval["revision"].as_u64(), Some(committed.revision() + 1));
        fixture.current(&fixture.proof(CALLER, "operator"));
        fixture.current(&fixture.proof(CALLER, "node"));
        fixture.token_absent(&old_operator);
        fixture.token_absent(&old_node);
        let repaired = fixture.observe();
        assert_eq!(text(repaired.device(CALLER), "public_key"), format!("{CALLER}-repair-public-key"));
        assert!(repaired.device(TARGET) == before.device(TARGET));
        assert!(repaired.device(BYSTANDER) == before.device(BYSTANDER));
    }
}

#[test]
fn paired_revocation_cross_device_denial_precedes_target_inspection() {
    let fixture = Fixture::seeded("cross", &["operator"], &["operator.write"]);
    let proof = fixture.proof(CALLER, "operator");
    for target in [TARGET, ABSENT] {
        for ceiling in [None, Some("operator.admin")] {
            fixture.deny(&proof, json!({
                "deviceId": target, "role": "operator", "crossDevice": false,
                "callerDeviceId": target, "callerRole": "operator",
                "callerScopes": ["operator.admin"], "admin": true,
            }), ceiling, 403);
        }
    }
    fixture.write_column(TARGET, "tokens_json", &SqlValue::Blob(vec![0xff, 0, 9]));
    fixture.write_column(TARGET, "approved_scopes_json", &SqlValue::Text("DVR-private-target-baseline".into()));
    fixture.deny(&proof, json!({"deviceId": TARGET}), None, 403);
    fixture.deny(&proof, json!({"deviceId": ABSENT}), None, 403);
}

#[test]
fn paired_revocation_self_checks_primary_approved_and_every_token_role() {
    for source in ["primary", "approved", "extra-active-token", "extra-revoked-token"] {
        let fixture = Fixture::seeded(source, &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        match source {
            "primary" => fixture.write_column(CALLER, "role", &SqlValue::Text("node".into())),
            "approved" => fixture.write_column(CALLER, "roles_json", &SqlValue::Text(json!(["operator", "node"]).to_string())),
            _ => {
                let mut other_role = fixture.observe().entries(TARGET)["node"].clone();
                if source == "extra-revoked-token" { other_role["revokedAtMs"] = json!(1234); }
                fixture.edit_entries(CALLER, |entries| entries["orphaned-token"] = other_role);
            }
        }
        // The label is not a role-token revocation and cannot conceal a mixed row.
        fixture.deny(&proof, json!({"deviceId": CALLER, "role": "operator"}), None, 403);
        fixture.deny(&proof, json!({"deviceId": CALLER}), Some("operator.admin"), 403);
    }
}

#[test]
fn paired_revocation_decisive_role_corruption_is_internal_and_atomic() {
    for (column, raw) in [
        ("role", SqlValue::Null), ("role", SqlValue::Text("".into())),
        ("role", SqlValue::Text(" \t".into())), ("role", SqlValue::Blob(vec![0xff, 7])),
        ("roles_json", SqlValue::Null),
        ("roles_json", SqlValue::Text(json!(["operator", ""]).to_string())),
        ("roles_json", SqlValue::Text(json!(["operator", null]).to_string())),
        ("roles_json", SqlValue::Text(json!(["operator", 9]).to_string())),
    ] {
        let fixture = Fixture::seeded("decisive-role", &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(CALLER, column, &raw);
        fixture.deny(&proof, json!({"deviceId": CALLER, "role": "operator"}), None, 500);
    }
    for extra in [
        json!({"role": null, "revokedAtMs": 12}), json!({"role": 7}),
        json!({"role": ""}), json!({"role": " \t"}), json!({"revokedAtMs": 12}),
        Value::Null, json!("DVR-private-extra-entry"),
    ] {
        let fixture = Fixture::seeded("extra-role", &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.edit_entries(CALLER, |entries| entries["retired-extra-entry"] = extra);
        fixture.deny(&proof, json!({"deviceId": CALLER}), None, 500);
    }
}

#[test]
fn paired_revocation_management_requires_current_operator_write() {
    let cases: &[(&str, &[&str])] = &[
        ("operator", &[]), ("operator", &["operator.read"]),
        ("operator", &["operator.sessions.write"]), ("operator", &["operator.write.extra"]),
        ("operator", &["operator.future.permission"]),
        ("node", &["node.write"]), ("observer", &["observer.listen"]),
    ];
    for (role, scopes) in cases {
        let fixture = Fixture::seeded("management-grant", &[*role], scopes);
        let proof = fixture.proof(CALLER, role);
        fixture.current(&proof);
        fixture.deny(&proof, json!({"deviceId": CALLER}), None, 403);
        fixture.deny(&proof, json!({"deviceId": TARGET}), Some("operator.admin"), 403);
    }
}

#[test]
fn paired_revocation_header_and_body_fields_cannot_supply_authority() {
    let fixture = Fixture::seeded("ceiling", &["operator"], &["operator.admin"]);
    let proof = fixture.proof(CALLER, "operator");
    for ceiling in ["", " \t", "operator.read", "node.write", "operator.write.extra", "operator.write,operator.admin", "operator.admin operator.write"] {
        fixture.deny(&proof, json!({"deviceId": CALLER, "admin": true}), Some(ceiling), 403);
    }
    fixture.deny(&proof, json!({"deviceId": TARGET, "callerDeviceId": TARGET, "crossDevice": false}), Some("operator.write"), 403);
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": CALLER, "role": "node", "crossDevice": true, "callerDeviceId": TARGET}),
        &proof, Some(" operator.write "),
    ).expect("write ceiling allows operator-only self deletion; role is merely a receipt label");
    fixture.removed(&before, &receipt, CALLER, "node", before.revision() + 1);

    let mixed = Fixture::seeded("mixed-ceiling", &["operator", "node"], &["operator.admin", "node.invoke"]);
    let proof = mixed.proof(CALLER, "operator");
    mixed.deny(&proof, json!({"deviceId": CALLER, "role": "operator"}), Some("operator.write"), 403);
    let before = mixed.observe();
    let receipt = mixed.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": TARGET, "admin": false, "callerDeviceId": TARGET}), &proof, Some("operator.admin"),
    ).expect("real sealed admin remains sufficient regardless of body caller assertions");
    mixed.removed(&before, &receipt, TARGET, "all", before.revision() + 1);
}

#[test]
fn paired_revocation_proofs_bind_issuer_and_ignore_public_projection() {
    let fixture = Fixture::seeded("sealed", &["operator"], &["operator.write"]);
    let original = fixture.observe().token(CALLER, "operator");
    let proof = fixture.authenticate(&original);
    let other_owner = DevicePairingOwner::new(&fixture.db).expect("second issuer over the same durable file");
    let other_proof = other_owner.authenticate_device_token(&original).unwrap().unwrap();
    assert_eq!(other_proof.device_id, proof.device_id);
    assert_eq!(other_proof.role, proof.role);
    assert_eq!(other_proof.scopes, proof.scopes);
    assert!(other_proof != proof, "equal public projections do not establish issuer identity");
    fixture.deny(&other_proof, json!({"deviceId": CALLER}), None, 401);
    fixture.stale(&other_proof, 401);
    drop(other_owner);

    let foreign = Fixture::seeded("foreign", &["operator"], &["operator.write"]);
    foreign.write_column(CALLER, "tokens_json", fixture.observe().device(CALLER).get("tokens_json").unwrap());
    let foreign_proof = foreign.authenticate(&original);
    foreign.current(&foreign_proof);
    foreign.stale(&proof, 401);
    foreign.deny(&proof, json!({"deviceId": CALLER}), None, 401);

    let mut altered = proof.clone();
    altered.device_id = TARGET.to_owned();
    altered.role = "node".to_owned();
    altered.scopes = vec!["operator.admin".to_owned()];
    fixture.current(&altered);
    fixture.deny(&altered, json!({"deviceId": TARGET}), Some("operator.admin"), 403);
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": CALLER}), &altered, None,
    ).expect("the sealed legitimate principal wins over every altered public projection");
    fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
    fixture.stale(&altered, 401);
    foreign.current(&foreign_proof);
}

#[test]
fn paired_revocation_exact_credential_retirement_precedes_role_policy() {
    for (role, scope) in [("operator", "operator.read"), ("node", "node.invoke"), ("observer", "observer.listen")] {
        let fixture = Fixture::seeded("retirement-order", &[role], &[scope]);
        let original = fixture.observe().token(CALLER, role);
        let mut proof = fixture.authenticate(&original);
        proof.device_id = TARGET.to_owned();
        proof.role = "operator".to_owned();
        proof.scopes = vec!["operator.admin".to_owned()];
        fixture.current(&proof);
        fixture.deny(&proof, json!({"deviceId": CALLER}), Some("operator.admin"), 403);
        let successor = fixture.native_rotate(CALLER, role);
        fixture.token_absent(&original);
        fixture.stale(&proof, 401);
        fixture.deny(&proof, json!({"deviceId": CALLER}), Some(""), 401);
        fixture.deny(&proof, json!({"deviceId": TARGET}), Some("operator.admin"), 401);
        let current = fixture.authenticate(&successor);
        fixture.current(&current);
        fixture.deny(&current, json!({"deviceId": CALLER}), None, 403);
        let before = fixture.observe();
        let receipt = fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": CALLER}))
            .expect("native removal independently retires the captured non-management credential");
        fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
        fixture.deny(&current, json!({"deviceId": CALLER}), None, 401);
        fixture.stale(&current, 401);
    }

    // DVR-008: exact bytes alone do not keep a nonoperator credential current.
    // The shared DPA admission must classify the complete live grant before its
    // operator-only management refusal, for both removal and token rotation.
    let refuse_management = |fixture: &Fixture, proof: &DeviceTokenAuthorization, role: &str, status: u16| {
        let original_proof = proof.clone();
        for ceiling in [None, Some("")] {
            fixture.deny(proof, json!({"deviceId": CALLER}), ceiling, status);
            let before = fixture.observe();
            fixture.error(fixture.owner().rotate_token_for_authenticated_device(
                &json!({"deviceId": CALLER, "role": role}), proof, ceiling,
            ), status, &before);
            assert!(fixture.observe() == before,
                "rotation refusal changed paired/pending/token/surface/configuration/revision state");
        }
        assert!(proof == &original_proof, "refusal replaced or altered the original sealed proof");
    };
    for (role, scope, additional_scope) in [
        ("node", "node.invoke", "node.observe"),
        ("observer", "observer.listen", "observer.read"),
    ] {
        let mut fixture = Fixture::seeded("nonoperator-grant-order", &[role], &[scope, additional_scope]);
        let healthy = fixture.observe();
        let bearer = healthy.token(CALLER, role);
        let proof = fixture.authenticate(&bearer);
        let original_proof = proof.clone();
        fixture.current(&proof);
        refuse_management(&fixture, &proof, role, 403);
        assert!(fixture.observe() == healthy, "current nonoperator refusal must be read-only");

        for narrowed in [json!([]), json!([scope])] {
            fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Text(narrowed.to_string()));
            let after_edit = fixture.observe();
            assert!(after_edit.token(CALLER, role) == bearer, "approval edit preserves exact credential bytes");
            assert!(after_edit.counter() == healthy.counter(), "independent approval edit does not revise history");
            refuse_management(&fixture, &proof, role, 401);
            fixture.stale(&proof, 401);
            assert!(fixture.observe() == after_edit, "withdrawn approval refusal preserves every post-edit SQL value");
            fixture.write_column(CALLER, "approved_scopes_json", healthy.device(CALLER).get("approved_scopes_json").unwrap());
            assert!(fixture.observe() == healthy, "explicit approval repair restores every original byte");
            fixture.current(&proof);
            refuse_management(&fixture, &proof, role, 403);
        }

        for malformed in [Value::Null, json!(scope), json!([scope, 7])] {
            fixture.edit_entries(CALLER, |entries| entries[role]["scopes"] = malformed);
            let after_edit = fixture.observe();
            assert!(after_edit.token(CALLER, role) == bearer, "malformed grant leaves the admitted credential intact");
            assert!(after_edit.counter() == healthy.counter());
            refuse_management(&fixture, &proof, role, 500);
            fixture.stale(&proof, 500);
            assert!(fixture.observe() == after_edit, "malformed live grant refusal cannot repair or mutate storage");
            fixture.write_column(CALLER, "tokens_json", healthy.device(CALLER).get("tokens_json").unwrap());
            assert!(fixture.observe() == healthy);
            fixture.current(&proof);
            refuse_management(&fixture, &proof, role, 403);
        }

        for malformed in [
            SqlValue::Text("DVR-private-nonoperator-baseline".into()),
            SqlValue::Text("null".into()),
            SqlValue::Text(json!([scope, false]).to_string()),
            SqlValue::Blob(vec![0xff, 0, 8]),
        ] {
            fixture.write_column(CALLER, "approved_scopes_json", &malformed);
            let after_edit = fixture.observe();
            assert!(after_edit.token(CALLER, role) == bearer);
            assert!(after_edit.counter() == healthy.counter());
            refuse_management(&fixture, &proof, role, 500);
            fixture.stale(&proof, 500);
            assert!(fixture.observe() == after_edit, "malformed present baseline refusal preserves raw SQL types and values");
            fixture.write_column(CALLER, "approved_scopes_json", healthy.device(CALLER).get("approved_scopes_json").unwrap());
            assert!(fixture.observe() == healthy);
            fixture.current(&proof);
            refuse_management(&fixture, &proof, role, 403);
        }

        assert!(proof == original_proof, "storage changes must never refresh the captured sealed admission");
        fixture.reopen();
        assert!(fixture.observe() == healthy, "all refusals and explicit repairs survive owner reopen unchanged");
        refuse_management(&fixture, &proof, role, 401);
        let fresh = fixture.authenticate(&bearer);
        assert_eq!(fresh.device_id, proof.device_id);
        assert_eq!(fresh.role, proof.role);
        assert_eq!(fresh.scopes, proof.scopes);
        assert!(fresh != proof, "reopen must issue new owner provenance for the same public projection");
        fixture.current(&fresh);
        refuse_management(&fixture, &fresh, role, 403);
        assert!(fixture.observe() == healthy, "fresh current nonoperator still has no management mutation authority");
        assert!(proof == original_proof);
    }

    // Missing or ill-typed selected identity fields keep the established DPA 401.
    for retirement in ["missing-entry", "missing-token", "wrong-token-type", "missing-role", "wrong-role-type", "different-role", "replaced-token", "revoked-entry", "unapproved-role"] {
        let fixture = Fixture::seeded(retirement, &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        if retirement == "unapproved-role" {
            fixture.write_column(CALLER, "roles_json", &SqlValue::Text("[]".into()));
        } else {
            let replacement = fixture.observe().token(TARGET, "operator");
            fixture.edit_entries(CALLER, |entries| {
                match retirement {
                    "missing-entry" => { entries.as_object_mut().unwrap().remove("operator"); }
                    "missing-token" => { entries["operator"].as_object_mut().unwrap().remove("token"); }
                    "wrong-token-type" => entries["operator"]["token"] = json!(7),
                    "missing-role" => { entries["operator"].as_object_mut().unwrap().remove("role"); }
                    "wrong-role-type" => entries["operator"]["role"] = json!(7),
                    "different-role" => entries["operator"]["role"] = json!("node"),
                    "replaced-token" => entries["operator"]["token"] = json!(replacement),
                    "revoked-entry" => entries["operator"]["revokedAtMs"] = json!(1234),
                    _ => unreachable!(),
                }
            });
        }
        fixture.deny(&proof, json!({"deviceId": CALLER}), None, 401);
        fixture.stale(&proof, 401);
    }
}

#[test]
fn paired_revocation_bad_requests_and_authorized_absence_preserve_state() {
    let fixture = Fixture::seeded("request-types", &["operator"], &["operator.admin"]);
    let proof = fixture.proof(CALLER, "operator");
    for args in [
        Value::Null, json!([]), json!("not-an-object"), json!({}),
        json!({"deviceId": null}), json!({"deviceId": 7}), json!({"deviceId": ""}), json!({"deviceId": " \t"}),
        json!({"deviceId": TARGET, "role": null}), json!({"deviceId": TARGET, "role": false}),
        json!({"deviceId": TARGET, "role": []}), json!({"deviceId": TARGET, "role": 7}),
        json!({"deviceId": TARGET, "role": ""}), json!({"deviceId": TARGET, "role": " \t"}),
    ] {
        fixture.deny(&proof, args, None, 400);
    }
    for _ in 0..2 {
        fixture.deny(&proof, json!({"deviceId": ABSENT}), None, 400);
        fixture.deny_native(json!({"deviceId": ABSENT}), 400);
    }
    fixture.current(&proof);
}

#[test]
fn paired_revocation_corrupt_caller_and_baseline_fallback_fail_closed() {
    for (column, raw) in [
        ("approved_scopes_json", SqlValue::Text("DVR-private-broken-baseline".into())),
        ("approved_scopes_json", SqlValue::Text("null".into())),
        ("approved_scopes_json", SqlValue::Text(json!({"DVR-private-authority": true}).to_string())),
        ("approved_scopes_json", SqlValue::Text(json!(["operator.write", false]).to_string())),
        ("approved_scopes_json", SqlValue::Blob(vec![0xff, 0, 9])),
        ("roles_json", SqlValue::Text("DVR-private-broken-roles".into())),
        ("roles_json", SqlValue::Text("{}".into())),
        ("roles_json", SqlValue::Blob(vec![0xff, 7])),
        ("tokens_json", SqlValue::Text("DVR-private-broken-token-map".into())),
        ("tokens_json", SqlValue::Text("[]".into())),
        ("tokens_json", SqlValue::Text("null".into())),
        ("tokens_json", SqlValue::Blob(vec![0xff, 0, 1])),
    ] {
        let fixture = Fixture::seeded("caller-corruption", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(CALLER, column, &raw);
        fixture.deny(&proof, json!({"deviceId": TARGET}), None, 500);
        fixture.stale(&proof, 500);
    }
    for malformed in [Value::Null, json!("operator.write"), json!(["operator.write", 7])] {
        let fixture = Fixture::seeded("entry-scopes", &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.edit_entries(CALLER, |entries| entries["operator"]["scopes"] = malformed);
        fixture.deny(&proof, json!({"deviceId": CALLER}), None, 500);
        fixture.stale(&proof, 500);
    }
    for legacy in [SqlValue::Null, SqlValue::Text("DVR-private-corrupt-legacy-scopes".into()), SqlValue::Blob(vec![0xff, 2])] {
        let fixture = Fixture::seeded("legacy-corrupt", &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Null);
        fixture.write_column(CALLER, "scopes_json", &legacy);
        fixture.deny(&proof, json!({"deviceId": CALLER}), None, 500);
        fixture.stale(&proof, 500);
    }
    for baseline in [json!([]), json!(["operator.read"])] {
        let fixture = Fixture::seeded("present-baseline", &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Text(baseline.to_string()));
        fixture.deny(&proof, json!({"deviceId": CALLER}), None, 401);
        fixture.stale(&proof, 401);
    }
    let fixture = Fixture::seeded("legacy-null", &["operator"], &["operator.write"]);
    let proof = fixture.proof(CALLER, "operator");
    fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Null);
    fixture.current(&proof);
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": CALLER}), &proof, None,
    ).expect("SQL NULL alone retains the valid legacy approval fallback");
    fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
}

#[test]
fn paired_revocation_live_grants_bound_sealed_management_admission() {
    for narrow_approval in [false, true] {
        let fixture = Fixture::seeded("narrow-management", &["operator"], &["operator.admin"]);
        let bearer = fixture.observe().token(CALLER, "operator");
        let proof = fixture.authenticate(&bearer);
        fixture.grant(CALLER, "operator", &["operator.write"]);
        if narrow_approval { fixture.baseline(CALLER, &["operator.write"]); }
        assert!(fixture.observe().token(CALLER, "operator") == bearer, "exercise changed grants without replacing bytes");
        fixture.stale(&proof, 401);
        fixture.deny(&proof, json!({"deviceId": TARGET}), None, 403);
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
            &json!({"deviceId": CALLER}), &proof, None,
        ).expect("bound management keeps DPA's intersection semantics for a narrowed live write grant");
        fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
    }
    let fixture = Fixture::seeded("whole-live-grant", &["operator"], &["operator.write", "operator.approvals"]);
    let proof = fixture.proof(CALLER, "operator");
    fixture.baseline(CALLER, &["operator.write"]);
    fixture.deny(&proof, json!({"deviceId": CALLER}), None, 401);
    fixture.stale(&proof, 401);
    let before = fixture.observe();
    assert!(fixture.owner().authenticate_device_token(&before.token(CALLER, "operator")).unwrap().is_none());
    assert!(fixture.observe() == before, "fresh refusal preserves the narrowed state");
}

#[test]
fn revocation_revision_corruption_rolls_back_and_recovers_after_reopen() {
    for paired in [false, true] {
        for raw in [
            SqlValue::Text("DVR-private-invalid-revision".into()),
            SqlValue::Text("\"18446744073709551616\"".into()),
            SqlValue::Text("3.5".into()), SqlValue::Blob(vec![0xff, 0, 0x33]),
            SqlValue::Text(format!("\"{}\"", u64::MAX)),
        ] {
            let mut fixture = Fixture::seeded("revision-refusal", &["operator"], &["operator.write"]);
            let healthy = fixture.observe();
            let bearer = healthy.token(CALLER, "operator");
            let proof = fixture.authenticate(&bearer);
            fixture.set_revision(&raw);
            let corrupt = fixture.observe();
            assert!(corrupt.counter().get("value_json") == Some(&raw));
            assert!(corrupt.counter().get("updated_at_ms") == Some(&SqlValue::Integer(1234)));
            for _ in 0..2 {
                if paired { fixture.deny(&proof, json!({"deviceId": CALLER}), None, 500); }
                else { fixture.deny_native(json!({"deviceId": CALLER}), 500); }
            }
            fixture.current(&proof); // Currentness neither repairs nor needs the mutation counter.
            fixture.reopen();
            assert!(fixture.observe() == corrupt, "rollback must retain raw revision and every row after reopen");
            fixture.stale(&proof, 401);
            let fresh = fixture.authenticate(&bearer);
            fixture.current(&fresh);
            if paired { fixture.deny(&fresh, json!({"deviceId": CALLER}), None, 500); }
            else { fixture.deny_native(json!({"deviceId": CALLER}), 500); }
            fixture.set_revision(healthy.counter().get("value_json").unwrap());
            let restored = fixture.observe();
            let result = if paired {
                fixture.owner().revoke_pairing_for_authenticated_device(&json!({"deviceId": CALLER}), &fresh, None)
            } else {
                fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": CALLER}))
            };
            let receipt = result.expect("explicit known-good fixture repair admits one new removal attempt");
            fixture.removed(&restored, &receipt, CALLER, "all", healthy.revision() + 1);
            let committed = fixture.observe();
            fixture.reopen();
            assert!(fixture.observe() == committed, "the controlled recovery commit is durable");
            fixture.token_absent(&bearer);
        }
    }
}

#[test]
fn paired_revocation_final_revision_increment_is_durable() {
    let mut fixture = Fixture::seeded("last-revision", &["operator"], &["operator.admin"]);
    fixture.set_revision(&SqlValue::Text(format!("\"{}\"", u64::MAX - 1)));
    let proof = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": TARGET}), &proof, None,
    ).expect("the final representable revision remains a successful deletion");
    fixture.removed(&before, &receipt, TARGET, "all", u64::MAX);
    fixture.deny(&proof, json!({"deviceId": CALLER}), None, 500);
    fixture.deny_native(json!({"deviceId": CALLER}), 500);
    fixture.deny_native(json!({"deviceId": TARGET}), 400);
    let exhausted = fixture.observe();
    fixture.reopen();
    assert!(fixture.observe() == exhausted);
    let fresh = fixture.proof(CALLER, "operator");
    fixture.current(&fresh);
    fixture.deny(&fresh, json!({"deviceId": CALLER}), None, 500);
}

#[test]
fn revocation_checks_delete_count_and_redacts_transaction_failures() {
    for paired in [false, true] {
        for fault in ["zero-delete", "delete-error", "revision-error"] {
            let mut fixture = Fixture::seeded(fault, &["operator"], &["operator.write"]);
            let proof = fixture.proof(CALLER, "operator");
            let sql = match fault {
                "zero-delete" =>
                    "CREATE TRIGGER dvr_fault BEFORE DELETE ON device_pairing_paired
                     WHEN OLD.device_id = 'dvr-caller' BEGIN SELECT RAISE(IGNORE); END;",
                "delete-error" =>
                    "CREATE TRIGGER dvr_fault BEFORE DELETE ON device_pairing_paired
                     WHEN OLD.device_id = 'dvr-caller'
                     BEGIN SELECT RAISE(ABORT, 'DVR-private-storage-fault'); END;",
                "revision-error" =>
                    "CREATE TRIGGER dvr_fault BEFORE UPDATE OF value_json ON config_machine_state
                     WHEN OLD.state_key = 'devicePairing.revision'
                     BEGIN SELECT RAISE(ABORT, 'DVR-private-storage-fault'); END;",
                _ => unreachable!(),
            };
            let conn = fixture.writer();
            conn.execute_batch(sql).expect("install one owned deterministic SQLite fault");
            conn.close().expect("close trigger injector");
            let before = fixture.observe();
            if paired { fixture.deny(&proof, json!({"deviceId": CALLER}), None, 500); }
            else { fixture.deny_native(json!({"deviceId": CALLER}), 500); }
            fixture.current(&proof);
            fixture.reopen();
            assert!(fixture.observe() == before, "failed transaction and trigger retain every durable row on reopen");
            let fresh = fixture.proof(CALLER, "operator");
            if paired { fixture.deny(&fresh, json!({"deviceId": CALLER}), None, 500); }
            else { fixture.deny_native(json!({"deviceId": CALLER}), 500); }
            let conn = fixture.writer();
            conn.execute_batch("DROP TRIGGER dvr_fault").expect("remove exactly this test's injected fault");
            conn.close().expect("close trigger repair writer");
            let result = if paired {
                fixture.owner().revoke_pairing_for_authenticated_device(&json!({"deviceId": CALLER}), &fresh, None)
            } else {
                fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": CALLER}))
            };
            let receipt = result.expect("a controlled healthy attempt commits after the injected fault is removed");
            fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
        }
    }
}

#[test]
fn revocation_immediate_transaction_precedes_policy_and_absence_decisions() {
    let fixture = Fixture::seeded("immediate", &["operator"], &["operator.read"]);
    let proof = fixture.proof(CALLER, "operator");
    let writer = fixture.writer();
    let journal: String = writer.query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0))
        .expect("select rollback-journal locking for this causal oracle");
    assert_eq!(journal, "delete");
    writer.execute_batch("BEGIN IMMEDIATE").expect("a second connection owns the reserved writer lock");

    // A RESERVED writer still permits reads. Currentness must not acquire a
    // writer lock or update last-seen/revision just to admit a retained reader.
    fixture.current(&proof);
    // A deferred transaction or pre-transaction policy/absence refusal would
    // return 403 or 400 here. Require write admission before those decisions;
    // elapsed time is never the success oracle. Source review separately checks
    // that no pre-transaction read is saved and reused after BEGIN IMMEDIATE.
    fixture.deny(&proof, json!({"deviceId": CALLER}), None, 500);
    fixture.deny_native(json!({"deviceId": ABSENT}), 500);
    writer.execute_batch("ROLLBACK").expect("release only this fixture's reserved lock");
    writer.close().expect("close independent locking connection");
    fixture.current(&proof);
    fixture.deny(&proof, json!({"deviceId": CALLER}), None, 403);
    fixture.deny_native(json!({"deviceId": ABSENT}), 400);
}

#[test]
fn revocation_commit_failure_rolls_back_before_reopen_and_explicit_recovery() {
    for paired in [false, true] {
        let mut fixture = Fixture::seeded("commit-lock", &["operator"], &["operator.write"]);
        let proof = fixture.proof(CALLER, "operator");
        let before = fixture.observe();
        let journal_writer = fixture.writer();
        let journal: String = journal_writer.query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0))
            .expect("use rollback journal so the separate reader obstructs COMMIT");
        assert_eq!(journal, "delete");
        journal_writer.close().expect("close journal configuration connection");
        let reader = Connection::open_with_flags(&fixture.db, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .expect("open a real independent SQLite reader");
        reader.execute_batch("BEGIN DEFERRED").expect("start a pinned read transaction");
        let paired_count: i64 = reader.query_row("SELECT COUNT(*) FROM device_pairing_paired", [], |row| row.get(0))
            .expect("take the SHARED lock before production write admission");
        assert_eq!(paired_count, 3);
        // SHARED permits BEGIN IMMEDIATE and the row writes, but prevents the
        // exclusive COMMIT. Require the returned error and the complete rollback.
        let result = if paired {
            fixture.owner().revoke_pairing_for_authenticated_device(&json!({"deviceId": CALLER}), &proof, None)
        } else {
            fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": CALLER}))
        };
        fixture.error(result, 500, &before);
        reader.execute_batch("COMMIT").expect("release the independent reader after the failed mutation returns");
        reader.close().expect("close the pinned reader before any fixture reopen");
        assert!(fixture.observe() == before, "commit refusal must roll back deletion and revision together");
        fixture.current(&proof);
        fixture.reopen();
        assert!(fixture.observe() == before, "complete rollback survives a real owner close/reopen");
        let fresh = fixture.proof(CALLER, "operator");
        let result = if paired {
            fixture.owner().revoke_pairing_for_authenticated_device(&json!({"deviceId": CALLER}), &fresh, None)
        } else {
            fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": CALLER}))
        };
        let receipt = result.expect("explicit healthy control commits once after the obstructing reader closes");
        fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
    }
}

#[test]
fn paired_revocation_concurrent_self_removal_commits_exactly_once() {
    let fixture = Fixture::seeded("concurrent-self", &["operator"], &["operator.write"]);
    let proof = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let owner = fixture.owner();
    let start = Barrier::new(3);
    let results = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            start.wait();
            owner.revoke_pairing_for_authenticated_device(&json!({"deviceId": CALLER}), &proof, None)
        });
        let second = scope.spawn(|| {
            start.wait();
            owner.revoke_pairing_for_authenticated_device(&json!({"deviceId": CALLER}), &proof, None)
        });
        start.wait();
        (first.join().expect("first owned removal thread must finish"),
         second.join().expect("second owned removal thread must finish"))
    });
    let (receipt, error) = match results {
        (Ok(receipt), Err(error)) | (Err(error), Ok(receipt)) => (receipt, error),
        _ => panic!("exactly one removal must commit and the other must refuse its retired proof"),
    };
    fixture.error::<()>(Err(error), 401, &before);
    fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
    fixture.stale(&proof, 401);
}

#[test]
fn retained_authorization_supports_current_roles_without_durable_writes() {
    let cases: &[(&str, &[&str])] = &[
        ("operator", &[]), ("operator", &["operator.read"]),
        ("operator", &["operator.write"]), ("operator", &["operator.admin"]),
        ("node", &["node.invoke"]), ("observer", &["observer.listen"]),
    ];
    for (role, grant) in cases {
        let fixture = Fixture::seeded("retained-current", &[*role], grant);
        let proof = fixture.proof(CALLER, role);
        assert_eq!(proof.device_id, CALLER);
        assert_eq!(proof.role, *role);
        let before = fixture.observe();
        for _ in 0..3 { fixture.current(&proof); }
        assert!(fixture.observe() == before, "repeated currentness checks cannot refresh timestamps or activity state");
    }
}

#[test]
fn retained_authorization_narrowing_retires_original_grant_without_replacement() {
    let cases: &[(&str, &[&str], &[&str])] = &[
        ("operator", &["operator.admin"], &["operator.write"]),
        ("operator", &["operator.read"], &["operator.sessions.read"]),
        ("node", &["node.invoke", "node.observe"], &["node.invoke"]),
    ];
    for (role, admitted, narrowed) in cases {
        for narrow_approval in [false, true] {
            let fixture = Fixture::seeded("retained-narrow", &[*role], admitted);
            let bearer = fixture.observe().token(CALLER, role);
            let proof = fixture.authenticate(&bearer);
            fixture.grant(CALLER, role, narrowed);
            if narrow_approval { fixture.baseline(CALLER, narrowed); }
            assert!(fixture.observe().token(CALLER, role) == bearer, "narrowing must exercise unchanged credential bytes");
            fixture.stale(&proof, 401);
            let fresh = fixture.authenticate(&bearer);
            fixture.current(&fresh);
            assert!(proof.scopes != fresh.scopes, "old admission was not silently refreshed");
            fixture.stale(&proof, 401);
        }
    }
}

#[test]
fn retained_authorization_expansion_does_not_promote_sealed_admission() {
    for admitted in [&[][..], &["operator.read"][..], &["operator.write"][..]] {
        let fixture = Fixture::seeded("retained-expansion", &["operator"], admitted);
        let bearer = fixture.observe().token(CALLER, "operator");
        let proof = fixture.authenticate(&bearer);
        fixture.grant(CALLER, "operator", &["operator.admin"]);
        fixture.baseline(CALLER, &["operator.admin"]);
        assert!(fixture.observe().token(CALLER, "operator") == bearer);
        fixture.current(&proof);
        fixture.deny(&proof, json!({"deviceId": TARGET}), Some("operator.admin"), 403);
        if admitted != &["operator.write"][..] {
            fixture.deny(&proof, json!({"deviceId": CALLER}), Some("operator.admin"), 403);
        }
        let fresh = fixture.authenticate(&bearer);
        fixture.current(&fresh);
        assert_eq!(fresh.scopes, vec!["operator.admin"]);
        let before = fixture.observe();
        let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
            &json!({"deviceId": TARGET}), &fresh, None,
        ).expect("new authentication can admit the now-approved wider grant");
        fixture.removed(&before, &receipt, TARGET, "all", before.revision() + 1);
        fixture.current(&proof);
        fixture.deny(&proof, json!({"deviceId": ABSENT}), Some("operator.admin"), 403);
    }
}

#[test]
fn retained_authorization_uses_fixed_role_implications_and_complete_live_grant() {
    // Literal positive/negative examples from fixed-U operator-scope-compat.ts;
    // no copy of its scope predicate is used to calculate expected outcomes.
    let cases: &[(&str, &[&str], &[&str], bool)] = &[
        ("operator", &["operator.write"], &["operator.read", "operator.talk", "operator.cron", "operator.sessions.read", "operator.sessions.write"], true),
        ("operator", &["operator.admin"], &["operator.future.permission"], true),
        ("operator", &["operator.read"], &["operator.sessions.read"], true),
        ("operator", &["operator.sessions.write"], &["operator.sessions.read"], true),
        ("operator", &["operator.write"], &["operator.write", "operator.approvals"], false),
        ("operator", &["operator.write"], &["operator.write.extra"], false),
        ("operator", &["operator.read"], &["operator.sessions.write"], false),
        ("node", &["node.invoke"], &["node.invoke"], true),
        ("node", &["node.write"], &["node.read"], false),
        ("node", &["node.*"], &["node.invoke"], false),
        ("node", &["operator.admin"], &["operator.read"], false),
        ("node", &["other.read"], &["other.read"], false),
    ];
    for (role, baseline, live, allowed) in cases {
        let fixture = Fixture::seeded("retained-implication", &[*role], baseline);
        if *allowed {
            fixture.grant(CALLER, role, live);
            let proof = fixture.proof(CALLER, role);
            fixture.current(&proof);
        } else {
            // A genuine empty grant fits even a cross-role baseline. The later
            // refusal must be caused by the complete invalid live grant.
            fixture.grant(CALLER, role, &[]);
            let proof = fixture.proof(CALLER, role);
            fixture.grant(CALLER, role, live);
            fixture.stale(&proof, 401);
            let before = fixture.observe();
            assert!(fixture.owner().authenticate_device_token(&before.token(CALLER, role)).unwrap().is_none());
            assert!(fixture.observe() == before);
        }
    }
}

#[test]
fn fresh_authorization_requires_approved_role_and_entire_canonical_grant() {
    let cases: &[(&str, &[&str], &[&str], bool)] = &[
        ("operator", &["operator.read"], &["operator.write"], false),
        ("operator", &["operator.write"], &["operator.read", "operator.approvals"], false),
        ("operator", &["operator.write"], &["operator.read", "operator.sessions.write"], true),
        ("operator", &["operator.admin"], &["operator.future.permission"], true),
        ("operator", &[], &[], true),
        ("node", &["node.invoke"], &["node.invoke"], true),
        ("observer", &["observer.read"], &["observer.read"], true),
        ("node", &["other.read"], &["other.read"], false),
        ("operator", &["operator.write"], &["\u{feff}operator.write\u{feff}", "operator.write"], true),
        ("operator", &["operator.write"], &["operator.write\u{85}"], false),
        ("operator", &["operator.write\u{85}"], &["operator.write\u{85}"], true),
    ];
    for (role, baseline, live, admitted) in cases {
        let fixture = Fixture::seeded("fresh-grant", &[*role], baseline);
        fixture.grant(CALLER, role, &[]);
        let prior = fixture.proof(CALLER, role);
        fixture.grant(CALLER, role, live);
        let before = fixture.observe();
        let result = fixture.owner().authenticate_device_token(&before.token(CALLER, role)).unwrap();
        assert_eq!(result.is_some(), *admitted, "literal fresh grant case");
        if let Some(proof) = result {
            assert_eq!(proof.role, *role);
            assert_eq!(proof.device_id, CALLER);
            fixture.assert_secret_free(&format!("{proof:?}"));
            fixture.current(&proof);
            if live.first().is_some_and(|scope| scope.starts_with('\u{feff}')) {
                assert_eq!(proof.scopes, ["operator.write"]);
            }
        } else {
            fixture.stale(&prior, 401);
        }
        fixture.current(&fixture.proof(BYSTANDER, "node"));
        assert!(fixture.observe() == before, "all fresh outcomes preserve complete raw state");
    }

    // A valid literal grant can be large without granting a wildcard or admin.
    // Inject only the explicit approved/grant fields; the production owner
    // still issues and validates every proof, including after durable reopen.
    let mut fixture = Fixture::seeded("fresh-large-grant", &["node"], &["node.invoke"]);
    let scopes: Vec<String> = (0..20_000).map(|index| format!("node.s{index:05}")).collect();
    let scope_refs: Vec<&str> = scopes.iter().map(String::as_str).collect();
    fixture.baseline(CALLER, &scope_refs);
    fixture.grant(CALLER, "node", &scope_refs);
    let mut roles: Vec<String> = (0..20_000).map(|index| format!("observer{index:05}")).collect();
    roles.extend([" node ".into(), "node".into(), "\t".into()]);
    fixture.write_column(CALLER, "roles_json", &SqlValue::Text(json!(roles).to_string()));
    let before = fixture.observe();
    let bearer = before.token(CALLER, "node");
    let proof = fixture.authenticate(&bearer);
    assert_eq!(proof.scopes, scopes, "retain the entire canonical literal grant");
    fixture.current(&proof);
    fixture.current(&fixture.proof(BYSTANDER, "node"));
    assert!(fixture.observe() == before);

    let mut malformed_roles = json!(roles);
    malformed_roles.as_array_mut().unwrap().push(json!(7));
    fixture.write_column(CALLER, "roles_json", &SqlValue::Text(malformed_roles.to_string()));
    let malformed = fixture.observe();
    let error = fixture.error(fixture.owner().authenticate_device_token(&bearer), 500, &malformed);
    assert_eq!(error.to_string(), "stored pairing authority is invalid");
    fixture.stale(&proof, 500);
    fixture.current(&fixture.proof(BYSTANDER, "node"));
    assert!(fixture.observe() == malformed, "a matching role cannot hide malformed trailing authority");
    fixture.write_column(CALLER, "roles_json", &SqlValue::Text(json!(roles).to_string()));
    assert!(fixture.observe() == before, "role repair restores exact bytes and valid admission");
    fixture.current(&fixture.authenticate(&bearer));

    let mut overgrant = scope_refs.clone();
    overgrant.push("node.zz-unapproved-tail");
    fixture.grant(CALLER, "node", &overgrant);
    let withdrawn = fixture.observe();
    assert!(fixture.owner().authenticate_device_token(&bearer).unwrap().is_none());
    fixture.stale(&proof, 401);
    fixture.current(&fixture.proof(BYSTANDER, "node"));
    assert!(fixture.observe() == withdrawn, "a single tail scope refuses the whole grant without writes");

    fixture.grant(CALLER, "node", &scope_refs);
    assert!(fixture.observe() == before, "repair restores the exact original state");
    fixture.reopen();
    let reopened = fixture.authenticate(&bearer);
    assert_eq!(reopened.scopes, scopes);
    fixture.current(&reopened);
    fixture.current(&fixture.proof(BYSTANDER, "node"));
    assert!(fixture.observe() == before, "large-grant authentication remains read-only after reopen");

    for mismatch in ["withdraw-role", "entry-role", "map-key"] {
        let fixture = Fixture::seeded("fresh-role", &["operator", "node"], &["operator.write", "node.invoke"]);
        let before = fixture.observe();
        let bearer = before.token(CALLER, "operator");
        let prior = fixture.authenticate(&bearer);
        match mismatch {
            "withdraw-role" => fixture.write_column(CALLER, "roles_json", &SqlValue::Text("[\"node\"]".into())),
            "entry-role" => fixture.edit_entries(CALLER, |entries| entries["operator"]["role"] = json!("observer")),
            "map-key" => fixture.edit_entries(CALLER, |entries| {
                let entry = entries.as_object_mut().unwrap().remove("operator").unwrap();
                entries["other"] = entry;
            }),
            _ => unreachable!(),
        }
        let withdrawn = fixture.observe();
        assert_eq!(text(withdrawn.device(CALLER), "role"), "operator", "primary label cannot restore approval");
        assert!(fixture.owner().authenticate_device_token(&bearer).unwrap().is_none());
        fixture.stale(&prior, 401);
        fixture.current(&fixture.proof(CALLER, "node"));
        fixture.current(&fixture.proof(BYSTANDER, "node"));
        assert!(fixture.observe() == withdrawn);
    }
}

#[test]
fn fresh_authorization_rejects_corrupt_decisive_storage_without_writes() {
    for field in ["roles_json", "approved_scopes_json"] {
        for raw in ["DVR-private-malformed-authority", "null", "{}", "[\"operator.write\",7]"] {
            let fixture = Fixture::seeded("fresh-shape", &["operator"], &["operator.write"]);
            let bearer = fixture.observe().token(CALLER, "operator");
            fixture.write_column(CALLER, field, &SqlValue::Text(raw.into()));
            let before = fixture.observe();
            let error = fixture.error(fixture.owner().authenticate_device_token(&bearer), 500, &before);
            assert_eq!(error.to_string(), "stored pairing authority is invalid");
            assert!(fixture.observe() == before);
        }
        let fixture = Fixture::seeded("fresh-sql-type", &["operator"], &["operator.write"]);
        let bearer = fixture.observe().token(CALLER, "operator");
        fixture.write_column(CALLER, field, &SqlValue::Blob(vec![0xff, 0, 0x33]));
        let before = fixture.observe();
        let error = fixture.error(fixture.owner().authenticate_device_token(&bearer), 500, &before);
        assert_eq!(error.to_string(), "stored pairing authority is invalid");
        assert!(fixture.observe() == before);
    }
    for scopes in [None, Some(Value::Null), Some(json!("operator.write")), Some(json!(["operator.write", 7]))] {
        let fixture = Fixture::seeded("fresh-token-shape", &["operator"], &["operator.write"]);
        let bearer = fixture.observe().token(CALLER, "operator");
        fixture.edit_entries(CALLER, |entries| match scopes {
            Some(value) => { entries["operator"]["scopes"] = value; }
            None => { entries["operator"].as_object_mut().unwrap().remove("scopes"); }
        });
        let before = fixture.observe();
        let error = fixture.error(fixture.owner().authenticate_device_token(&bearer), 500, &before);
        assert_eq!(error.to_string(), "stored pairing authority is invalid");
        assert!(fixture.observe() == before);
    }

    let fixture = Fixture::seeded("fresh-baseline-choice", &["operator"], &["operator.write"]);
    let bearer = fixture.observe().token(CALLER, "operator");
    fixture.write_column(CALLER, "approved_scopes_json", &SqlValue::Null);
    let fallback = fixture.observe();
    fixture.current(&fixture.authenticate(&bearer));
    assert!(fixture.observe() == fallback, "SQL NULL approval uses valid legacy scope baseline");
    fixture.write_column(CALLER, "scopes_json", &SqlValue::Null);
    let absent = fixture.observe();
    let error = fixture.error(fixture.owner().authenticate_device_token(&bearer), 500, &absent);
    assert_eq!(error.to_string(), "stored pairing authority is invalid");
    assert!(fixture.observe() == absent, "missing both baselines must not repair storage");
    fixture.baseline(CALLER, &["operator.write"]);
    fixture.write_column(CALLER, "scopes_json", &SqlValue::Blob(vec![0xff, 0, 0x44]));
    let decisive = fixture.observe();
    fixture.current(&fixture.authenticate(&bearer));
    assert!(fixture.observe() == decisive, "valid present approval ignores malformed unused fallback");
    fixture.write_column(CALLER, "roles_json", &SqlValue::Null);
    let missing_roles = fixture.observe();
    let error = fixture.error(fixture.owner().authenticate_device_token(&bearer), 500, &missing_roles);
    assert_eq!(error.to_string(), "stored pairing authority is invalid");
    assert!(fixture.observe() == missing_roles);
}

#[test]
fn fresh_authorization_reopen_preserves_current_approval_and_issuer_boundary() {
    let mut fixture = Fixture::seeded("fresh-reopen", &["operator", "node"], &["operator.write", "node.invoke"]);
    let initial = fixture.observe();
    let bearer = initial.token(CALLER, "operator");
    let old = fixture.authenticate(&bearer);
    fixture.baseline(CALLER, &["operator.read", "node.invoke"]);
    let withdrawn = fixture.observe();
    assert!(fixture.owner().authenticate_device_token(&bearer).unwrap().is_none());
    fixture.stale(&old, 401);
    assert!(fixture.observe() == withdrawn);
    fixture.reopen();
    assert!(fixture.observe() == withdrawn);
    assert!(fixture.owner().authenticate_device_token(&bearer).unwrap().is_none());
    assert!(fixture.observe() == withdrawn, "persisted overgrant still refuses after reopen");
    fixture.baseline(CALLER, &["operator.write", "node.invoke"]);
    assert!(fixture.observe() == initial, "explicit repair changes only the injected approval");
    let repaired = fixture.authenticate(&bearer);
    assert_eq!(repaired.device_id, old.device_id);
    assert_eq!(repaired.role, old.role);
    assert_eq!(repaired.scopes, old.scopes);
    fixture.current(&repaired);
    fixture.stale(&old, 401);
    fixture.reopen();
    let reopened = fixture.observe();
    assert!(reopened == initial);
    fixture.current(&fixture.authenticate(&bearer));
    fixture.stale(&repaired, 401);
    assert!(fixture.owner().authenticate_device_token("dfa-unknown-bearer").unwrap().is_none());
    fixture.edit_entries(CALLER, |entries| entries["operator"]["revokedAtMs"] = json!(1));
    let retired = fixture.observe();
    assert!(fixture.owner().authenticate_device_token(&bearer).unwrap().is_none());
    fixture.current(&fixture.proof(CALLER, "node"));
    assert!(fixture.observe() == retired);
}

#[test]
fn retained_authorization_replacement_leaves_other_roles_and_devices_current() {
    let fixture = Fixture::seeded("role-independence", &["operator", "node"], &["operator.read", "node.invoke"]);
    let original = fixture.observe();
    let operator = fixture.proof(CALLER, "operator");
    let node = fixture.proof(CALLER, "node");
    let bystander = fixture.proof(BYSTANDER, "node");
    fixture.native_rotate(TARGET, "operator");
    fixture.current(&operator);
    fixture.current(&node);
    fixture.current(&bystander);
    let new_node = fixture.native_rotate(CALLER, "node");
    fixture.stale(&node, 401);
    fixture.current(&operator);
    fixture.current(&bystander);
    let node_successor = fixture.authenticate(&new_node);
    fixture.current(&node_successor);
    let new_operator = fixture.native_rotate(CALLER, "operator");
    fixture.stale(&operator, 401);
    fixture.current(&node_successor);
    fixture.current(&bystander);
    fixture.token_absent(&original.token(CALLER, "operator"));
    fixture.token_absent(&original.token(CALLER, "node"));
    let operator_successor = fixture.authenticate(&new_operator);
    fixture.current(&operator_successor);
    let before = fixture.observe();
    let receipt = fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": TARGET}))
        .expect("remove an unrelated device through the real owner");
    fixture.removed(&before, &receipt, TARGET, "all", before.revision() + 1);
    fixture.current(&operator_successor);
    fixture.current(&node_successor);
    fixture.current(&bystander);
    let before = fixture.observe();
    let receipt = fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": CALLER, "role": "operator"}))
        .expect("whole-pairing native removal retires both remaining role proofs");
    fixture.removed(&before, &receipt, CALLER, "operator", before.revision() + 1);
    fixture.stale(&operator_successor, 401);
    fixture.stale(&node_successor, 401);
    fixture.current(&bystander);
}

#[test]
fn retained_authorization_reopen_requires_fresh_owner_issued_provenance() {
    let mut fixture = Fixture::seeded("retained-reopen", &["operator", "node"], &["operator.read", "node.invoke"]);
    let durable = fixture.observe();
    let old_operator = fixture.proof(CALLER, "operator");
    let old_node = fixture.proof(CALLER, "node");
    fixture.reopen();
    assert!(fixture.observe() == durable, "reopening preserves credential bytes and stored authority");
    for old in [&old_operator, &old_node] { fixture.stale(old, 401); }
    for (role, old) in [("operator", &old_operator), ("node", &old_node)] {
        let fresh = fixture.authenticate(&durable.token(CALLER, role));
        assert_eq!(fresh.device_id, old.device_id);
        assert_eq!(fresh.role, old.role);
        assert_eq!(fresh.scopes, old.scopes);
        assert!(&fresh != old, "same public projection does not preserve the old allocation-bound issuer");
        fixture.current(&fresh);
        fixture.stale(old, 401);
    }
}

#[test]
fn revocation_and_retained_authorization_propagate_storage_read_failure() {
    let fixture = Fixture::seeded("storage-read", &["operator"], &["operator.write"]);
    let proof = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let conn = fixture.writer();
    conn.execute_batch("ALTER TABLE device_pairing_paired RENAME TO dvr_owned_unavailable_pairings")
        .expect("make the existing fixture table unavailable without deleting its rows");
    conn.close().expect("close the explicit schema-fault injector");
    let errors = [
        fixture.error(fixture.owner().validate_current_device_authorization(&proof), 500, &before),
        fixture.error(fixture.owner().revoke_pairing_for_authenticated_device(&json!({"deviceId": CALLER}), &proof, None), 500, &before),
        fixture.error(fixture.owner().handle_pairing_command("device.pair.revoke", &json!({"deviceId": CALLER})), 500, &before),
    ];
    for error in errors {
        let diagnostic = format!("{error:?}\n{error}");
        assert!(!diagnostic.contains("no such table"), "storage internals must not escape fixed diagnostics");
        assert!(!diagnostic.contains("dvr_owned_unavailable_pairings"));
    }
    let conn = fixture.writer();
    conn.execute_batch("ALTER TABLE dvr_owned_unavailable_pairings RENAME TO device_pairing_paired")
        .expect("restore only the known fixture table identity");
    conn.close().expect("close schema repair writer");
    assert!(fixture.observe() == before, "failed reads neither delete state nor invent revision history");
    fixture.current(&proof);
    let receipt = fixture.owner().revoke_pairing_for_authenticated_device(
        &json!({"deviceId": CALLER}), &proof, None,
    ).expect("the unchanged credential works after the controlled storage restoration");
    fixture.removed(&before, &receipt, CALLER, "all", before.revision() + 1);
}
