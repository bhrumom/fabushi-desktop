//! DTR-v1 bound wire rotation owner contracts: production authentication/mutations, real SQLite files,
//! and independent complete-row observations. No HTTP or device acceptance claim.
//! Spec: projects/openclaw-rust/docs/specs/device-token-rotation-wire.md.
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

const CALLER: &str = "dtr-caller";
const TARGET: &str = "dtr-target";
const BYSTANDER: &str = "dtr-bystander";
const ABSENT: &str = "dtr-absent";
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
        let root = std::env::temp_dir().join(format!("fabushi-dtr-{label}-{}", uuid::Uuid::new_v4()));
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
        assert_eq!(fixture.request("dtr-unpaired", "pending", &["operator"], &["operator.read"])["isRepair"], false);
        let conn = fixture.writer();
        conn.execute(
            "INSERT INTO config_machine_state (state_key, value_json, updated_at_ms) VALUES (?1, ?2, ?3)",
            params!["dtr.unrelatedSetting", "{\"preserve\":\"DTR-private-config-marker\"}", 41],
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
            "displayName": format!("DTR {device}"), "platform": "linux",
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
        assert!(!diagnostic.contains("DTR-private"), "private storage witnesses must be redacted");
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
        self.error(self.owner().rotate_token_with_authority(&args, Some(proof), ceiling), status, &before);
        assert!(self.observe() == before, "refusal changed paired/pending/token/surface/configuration/revision state");
    }

    fn deny_native(&self, args: Value, ceiling: Option<&str>, status: u16) {
        let before = self.observe();
        self.error(self.owner().rotate_token_with_authority(&args, None, ceiling), status, &before);
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

    fn rotated(&self, before: &Snapshot, receipt: &Value, device: &str, role: &str, scopes: &[&str], in_band: bool) {
        self.rotated_with_roles(before, receipt, device, role, scopes, in_band, None);
    }

    fn rotated_with_roles(&self, before: &Snapshot, receipt: &Value, device: &str, role: &str, scopes: &[&str], in_band: bool, expected_roles: Option<&Value>) {
        let fields = receipt.as_object().expect("closed receipt object");
        assert_eq!(fields.len(), if in_band { 6 } else { 5 });
        for name in ["deviceId", "role", "scopes", "rotatedAtMs", "tokenDelivery"] {
            assert!(fields.contains_key(name), "required receipt field {name}");
        }
        assert_eq!(receipt["deviceId"], device);
        assert_eq!(receipt["role"], role);
        assert_eq!(receipt["scopes"], json!(scopes), "exact normalized result order and members");
        let now = receipt["rotatedAtMs"].as_i64().expect("signed integer rotation time");
        assert!((1..=9_007_199_254_740_991).contains(&now));
        assert_eq!(receipt["tokenDelivery"], if in_band { "in-band" } else { "withheld-cross-device" });
        let after = self.observe();
        let replacement = after.token(device, role);
        self.secrets.borrow_mut().push(replacement.clone());
        assert!(replacement.starts_with("sand-device-tok-"));
        assert_eq!(base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(
            replacement.strip_prefix("sand-device-tok-").unwrap(),
        ).expect("native opaque credential encoding").len(), 32);
        if in_band {
            assert!(receipt["token"].as_str() == Some(replacement.as_str()), "delivered bearer equals committed credential");
        } else {
            assert!(!fields.contains_key("token"), "withheld token must be absent");
            self.assert_secret_free(&receipt.to_string());
        }
        assert_eq!(after.paired.len(), before.paired.len());
        for previous in &before.paired {
            let id = text(previous, "device_id");
            let current = after.device(id);
            if id != device {
                assert!(current == previous, "unrelated complete paired row changed");
                continue;
            }
            for (column, old) in previous {
                if column == "tokens_json" { continue; }
                if column == "roles_json" {
                    if let Some(roles) = expected_roles {
                        assert!(current.get(column) == Some(&SqlValue::Text(roles.to_string())), "only the explicitly expected selected role may be materialized");
                        continue;
                    }
                }
                if role == "node" && matches!(column.as_str(), "node_surface_json" | "pending_node_surface_json") {
                    assert!(current.get(column) == Some(&SqlValue::Null));
                } else {
                    assert!(current.get(column) == Some(old), "unselected SQL column changed: {column}");
                }
            }
            let old_entries = before.entries(device);
            let mut expected = old_entries.clone();
            let selected = expected.as_object_mut().unwrap().entry(role).or_insert_with(|| json!({}));
            let selected = selected.as_object_mut().unwrap();
            for name in ["token", "role", "scopes", "rotatedAtMs", "revokedAtMs"] { selected.remove(name); }
            let actual_entries = after.entries(device);
            let mut actual = actual_entries.clone();
            let actual_selected = actual[role].as_object_mut().unwrap();
            assert!(actual_selected.get("revokedAtMs").is_none());
            assert_eq!(actual_selected.get("role"), Some(&json!(role)));
            assert_eq!(actual_selected.get("scopes"), Some(&json!(scopes)));
            assert_eq!(actual_selected.get("rotatedAtMs"), Some(&json!(now)));
            for name in ["token", "role", "scopes", "rotatedAtMs"] { actual_selected.remove(name); }
            if selected.get("createdAtMs").is_none_or(Value::is_null) {
                selected.insert("createdAtMs".into(), json!(now));
            }
            assert!(actual.to_string() == expected.to_string(), "selected lifecycle numeric representation/issuer/unknown metadata or unselected entries changed");
            if let Some(old) = old_entries.get(role).and_then(|entry| entry.get("token")).and_then(Value::as_str) {
                assert!(replacement != old, "rotation must issue a new token identity");
                if !old.is_empty() { self.token_absent(old); }
            }
        }
        assert!(after.pending == before.pending, "pending pairings changed");
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
        // The persisted successor must authenticate even when the reply withheld it.
        let successor = self.authenticate(&replacement);
        assert_eq!(successor.device_id, device);
        assert_eq!(successor.role, role);
        assert_eq!(successor.scopes, scopes);
        self.current(&successor);
    }

    fn token_absent(&self, token: &str) {
        let before = self.observe();
        assert!(self.owner().authenticate_device_token(token).expect("lookup predecessor bearer").is_none());
        assert!(self.observe() == before);
    }
}

impl Drop for Fixture {
    fn drop(&mut self) {
        drop(self.owner.take());
        if std::thread::panicking() {
            eprintln!("DTR fixture retained for Actions capture: {}", self.root.display());
        } else {
            fs::remove_dir_all(&self.root).expect("remove only the owned fixture after dropping its owner");
        }
    }
}

fn args(device: &str, role: &str) -> Value { json!({"deviceId": device, "role": role}) }

fn scoped(device: &str, role: &str, scopes: &[&str]) -> Value {
    json!({"deviceId": device, "role": role, "scopes": scopes})
}

#[test]
fn wire_rotation_pairing_self_delivers_committed_successor_and_survives_reopen() {
    let mut f = Fixture::seeded("self", &["operator", "node"], &["operator.pairing", "node.invoke"]);
    let proof = f.proof(CALLER, "operator");
    let node = f.proof(CALLER, "node");
    let bystander = f.proof(BYSTANDER, "node");
    let before = f.observe();
    let receipt = f.owner().rotate_token_with_authority(
        &args(&format!(" {CALLER}\t"), " operator "), Some(&proof), Some(" operator.pairing "),
    ).expect("pairing-only self rotation of a mixed-role device");
    f.rotated(&before, &receipt, CALLER, "operator", &["operator.pairing"], true);
    f.stale(&proof, 401);
    f.deny(&proof, args(CALLER, "operator"), None, 401);
    f.current(&node);
    f.current(&bystander);
    let replacement = receipt["token"].as_str().unwrap().to_owned();
    f.current(&f.authenticate(&replacement));
    let committed = f.observe();
    f.reopen();
    assert!(f.observe() == committed);
    f.stale(&node, 401); // An old allocation's proof cannot cross owner reopen.
    f.current(&f.authenticate(&replacement));
    f.current(&f.proof(CALLER, "node"));
    f.token_absent(&before.token(CALLER, "operator"));
}

#[test]
fn wire_rotation_admin_self_other_role_delivers_but_cross_device_withholds() {
    for (device, role, in_band, scopes) in [
        (CALLER, "node", true, &["node.invoke"][..]),
        (TARGET, "node", false, &["node.invoke"][..]),
        (TARGET, "operator", false, &["operator.pairing"][..]),
    ] {
        let f = Fixture::seeded("admin-delivery", &["operator", "node"], &["operator.admin", "node.invoke"]);
        let admin = f.proof(CALLER, "operator");
        let selected = f.proof(device, role);
        let before = f.observe();
        let result = f.owner().rotate_token_with_authority(&args(device, role), Some(&admin), Some("operator.admin"))
            .expect("admin may select approved other roles/devices");
        f.rotated(&before, &result, device, role, scopes, in_band);
        f.stale(&selected, 401);
        f.current(&admin);
        let before_repeat = f.observe();
        let second = f.owner().rotate_token_with_authority(&args(device, role), Some(&admin), None).unwrap();
        f.rotated(&before_repeat, &second, device, role, scopes, in_band);
        f.token_absent(&before_repeat.token(device, role));
    }
}

#[test]
fn wire_rotation_native_dispatch_always_withholds_and_honors_singular_ceiling() {
    for ceiling in [None, Some("operator.admin"), Some(" operator.pairing ")] {
        let f = Fixture::seeded("native", &["operator"], &["operator.read"]);
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&args(TARGET, "operator"), None, ceiling).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
    }
    let f = Fixture::seeded("dispatch", &["operator"], &["operator.admin"]);
    let before = f.observe();
    let receipt = f.owner().handle_pairing_command("device.token.rotate", &args(TARGET, "node")).unwrap();
    f.rotated(&before, &receipt, TARGET, "node", &["node.invoke"], false);
    for ceiling in ["", " \t", "operator.write", "operator.read", "node.invoke", "operator.pairing,operator.admin", "operator.admin operator.pairing", "operator.pairing.extra"] {
        f.deny_native(args(CALLER, "operator"), Some(ceiling), 403);
        f.deny_native(args(ABSENT, "operator"), Some(ceiling), 403);
    }
    for target in [TARGET, ABSENT] { f.deny_native(args(target, "node"), Some("operator.pairing"), 403); }
    f.deny_native(args(CALLER, "operator"), Some("operator.pairing"), 403);
    let before = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&scoped(CALLER, "operator", &[]), None, Some("operator.pairing")).unwrap();
    f.rotated(&before, &receipt, CALLER, "operator", &[], false);
}

#[test]
fn wire_rotation_closed_request_rejects_all_invalid_shapes_before_any_mutation() {
    let f = Fixture::seeded("request", &["operator"], &["operator.admin"]);
    let proof = f.proof(CALLER, "operator");
    let mut invalid = vec![Value::Null, json!([]), json!(true), json!(7), json!("request"), json!({}),
        json!({"deviceId": TARGET}), json!({"role": "operator"})];
    for field in ["deviceId", "role"] {
        for value in [Value::Null, json!(false), json!(7), json!([]), json!({}), json!(""), json!(" \t")] {
            let mut request = args(TARGET, "operator"); request[field] = value; invalid.push(request);
        }
    }
    for extra in ["crossDevice", "callerDeviceId", "callerRole", "callerScopes", "admin", "revision", "tokenDelivery", "token", "unknown"] {
        let mut request = args(TARGET, "operator"); request[extra] = json!(true); invalid.push(request);
    }
    for scopes in [Value::Null, json!(true), json!(7), json!("operator.read"), json!({}), json!([null]), json!([false]), json!([7]), json!([[]]), json!([{}]), json!([""]), json!(["operator.read", ""])] {
        let mut request = args(TARGET, "operator"); request["scopes"] = scopes; invalid.push(request);
    }
    for request in invalid {
        f.deny(&proof, request.clone(), None, 400);
        f.deny_native(request, None, 400);
    }
    for (device, role) in [(ABSENT, "operator"), (TARGET, "Operator"), (TARGET, "observer")] {
        f.deny(&proof, args(device, role), None, 400);
        f.deny_native(args(device, role), None, 400);
    }
    let before = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &[" \t", "\u{feff}"]), Some(&proof), None).unwrap();
    f.rotated(&before, &receipt, TARGET, "operator", &[], false);
}

#[test]
fn wire_rotation_normalization_matches_fixed_u_implications_and_utf16_order() {
    // Fixed U device-auth.ts examples: FEFF trims, NEL remains significant;
    // U+10000 sorts before U+E000 under JavaScript's UTF-16 code-unit order.
    for (requested, expected) in [
        (vec!["operator.write", " operator.write ", "\u{feff}operator.pairing\u{feff}", "\t"], vec!["operator.pairing", "operator.read", "operator.write"]),
        (vec!["operator.admin"], vec!["operator.admin", "operator.read", "operator.write"]),
        (vec!["operator.\u{e000}", "operator.\u{10000}", "operator.\u{85}", "operator.\u{e000}"], vec!["operator.\u{85}", "operator.\u{10000}", "operator.\u{e000}"]),
    ] {
        let f = Fixture::seeded("normalize", &["operator"], &["operator.admin"]);
        f.baseline(TARGET, &["operator.admin"]);
        let proof = f.proof(CALLER, "operator");
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &requested), Some(&proof), None).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &expected, false);
    }
    let f = Fixture::seeded("nel-is-significant", &["operator"], &["operator.admin"]);
    f.baseline(TARGET, &["operator.read"]);
    let proof = f.proof(CALLER, "operator");
    for scope in ["\u{85}", "\u{85}operator.read", "operator.read\u{85}"] {
        f.deny(&proof, scoped(TARGET, "operator", &[scope]), None, 403);
    }
    let before = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &["\u{feff}operator.read\u{feff}"]), Some(&proof), None).unwrap();
    f.rotated(&before, &receipt, TARGET, "operator", &["operator.read"], false);
}

#[test]
fn wire_rotation_desired_scopes_select_explicit_existing_or_device_fallback() {
    for selected in [None, Some(Value::Null), Some(json!([])), Some(json!(["operator.read"]))] {
        let f = Fixture::seeded("scope-selection", &["operator"], &["operator.admin"]);
        f.write_column(TARGET, "scopes_json", &SqlValue::Text(json!(["operator.write"]).to_string()));
        f.baseline(TARGET, &["operator.admin"]);
        f.edit_entries(TARGET, |entries| {
            if let Some(value) = selected.clone() { entries["operator"]["scopes"] = value; }
            else { entries["operator"].as_object_mut().unwrap().remove("scopes"); }
        });
        let proof = f.proof(CALLER, "operator");
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&proof), None).unwrap();
        let expected = match selected {
            Some(Value::Array(ref items)) if items.is_empty() => &[][..],
            Some(Value::Array(_)) => &["operator.read"][..],
            _ => &["operator.read", "operator.write"][..],
        };
        f.rotated(&before, &receipt, TARGET, "operator", expected, false);
    }
    let f = Fixture::seeded("absent-device-fallback", &["operator"], &["operator.admin"]);
    f.edit_entries(TARGET, |entries| { entries.as_object_mut().unwrap().remove("operator"); });
    f.write_column(TARGET, "scopes_json", &SqlValue::Null);
    let before = f.observe();
    let receipt = f.owner().handle_token_rotate(&args(TARGET, "operator")).unwrap();
    f.rotated(&before, &receipt, TARGET, "operator", &[], false);
}

#[test]
fn wire_rotation_explicit_empty_repairs_excessive_or_malformed_selected_grants() {
    for old in [json!(["operator.admin"]), json!(["node.invoke"]), json!("bad"), json!({}), json!(["operator.pairing", 7])] {
        let f = Fixture::seeded("repair-operator", &["operator"], &["operator.admin"]);
        f.edit_entries(TARGET, |entries| entries["operator"]["scopes"] = old.clone());
        let admin = f.proof(CALLER, "operator");
        f.deny(&admin, args(TARGET, "operator"), None, if old.is_array() && old.as_array().unwrap().iter().all(Value::is_string) { 403 } else { 500 });
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &[]), None, Some("operator.pairing")).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &[], false);
    }
    for old in [json!(["node.legacy"]), json!(["operator.admin"]), json!([false]), json!(7)] {
        let f = Fixture::seeded("repair-node", &["operator"], &["operator.admin"]);
        f.edit_entries(TARGET, |entries| entries["node"]["scopes"] = old);
        let admin = f.proof(CALLER, "operator");
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "node", &[]), Some(&admin), None).unwrap();
        f.rotated(&before, &receipt, TARGET, "node", &[], false);
    }
}

#[test]
fn wire_rotation_omission_never_filters_cross_role_device_fallback() {
    let f = Fixture::seeded("mixed-fallback", &["operator"], &["operator.admin"]);
    let admin = f.proof(CALLER, "operator");
    for role in ["operator", "node"] {
        f.edit_entries(TARGET, |entries| { entries[role].as_object_mut().unwrap().remove("scopes"); });
        f.deny(&admin, args(TARGET, role), None, 403);
        let scopes = if role == "node" { &["node.invoke"][..] } else { &["operator.pairing"][..] };
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, role, scopes), Some(&admin), None).unwrap();
        f.rotated(&before, &receipt, TARGET, role, scopes, false);
    }
}

#[test]
fn wire_rotation_target_baseline_is_mandatory_even_for_empty_repair() {
    for raw in [SqlValue::Null, SqlValue::Blob(vec![0xff]), SqlValue::Text("DTR-private-invalid".into()),
        SqlValue::Text("null".into()), SqlValue::Text("{}".into()), SqlValue::Text("[\"operator.pairing\",7]".into())] {
        let f = Fixture::seeded("bad-baseline", &["operator"], &["operator.admin"]);
        let proof = f.proof(CALLER, "operator");
        f.write_column(TARGET, "approved_scopes_json", &raw);
        if raw == SqlValue::Null { f.write_column(TARGET, "scopes_json", &SqlValue::Null); }
        f.deny(&proof, scoped(TARGET, "operator", &[]), None, 500);
        f.deny_native(scoped(TARGET, "node", &[]), None, 500);
    }
    for empty in [true, false] {
        let f = Fixture::seeded("baseline-control", &["operator"], &["operator.admin"]);
        let proof = f.proof(CALLER, "operator");
        if empty { f.baseline(TARGET, &[]); }
        else { f.write_column(TARGET, "approved_scopes_json", &SqlValue::Null); }
        if empty { f.deny(&proof, args(TARGET, "operator"), None, 403); }
        let desired = if empty { &[][..] } else { &["operator.pairing"][..] };
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", desired), Some(&proof), None).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", desired, false);
    }
}

#[test]
fn wire_rotation_only_decisive_scope_storage_is_read_and_malformed_fallback_refuses() {
    for raw in [SqlValue::Blob(vec![0xff]), SqlValue::Text("DTR-private-invalid".into()),
        SqlValue::Text("null".into()), SqlValue::Text("{}".into()), SqlValue::Text("[7]".into())] {
        let mut f = Fixture::seeded("scope-fallback-corrupt", &["operator"], &["operator.admin"]);
        let proof = f.proof(CALLER, "operator");
        f.write_column(TARGET, "scopes_json", &raw);
        f.edit_entries(TARGET, |entries| entries["operator"]["scopes"] = Value::Null);
        f.deny(&proof, args(TARGET, "operator"), None, 500);
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &[]), Some(&proof), None).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &[], false);
        let committed = f.observe();
        let bearer = committed.token(TARGET, "operator");
        f.reopen();
        assert!(f.observe() == committed);
        f.current(&f.authenticate(&bearer));
        let successor = f.authenticate(&bearer);
        f.deny(&successor, args(TARGET, "operator"), None, 403);
    }
}

#[test]
fn wire_rotation_role_union_allows_missing_entry_but_never_stray_approval() {
    for (roles, primary, role, expected_roles) in [
        (json!([]), SqlValue::Text(" node ".into()), "node", Some(json!(["node"]))),
        (json!([" operator ", "", "operator"]), SqlValue::Null, "operator", None),
        (json!(["node"]), SqlValue::Text(" \t".into()), "node", None),
        (json!(["node"]), SqlValue::Text("operator".into()), "operator", Some(json!(["node", "operator"]))),
    ] {
        let f = Fixture::seeded("union", &["operator"], &["operator.admin"]);
        let admin = f.proof(CALLER, "operator");
        f.write_column(TARGET, "roles_json", &SqlValue::Text(roles.to_string()));
        f.write_column(TARGET, "role", &primary);
        f.edit_entries(TARGET, |entries| { entries.as_object_mut().unwrap().remove(role); });
        let scopes = if role == "node" { &["node.invoke"][..] } else { &["operator.pairing"][..] };
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, role, scopes), Some(&admin), None).unwrap();
        f.rotated_with_roles(&before, &receipt, TARGET, role, scopes, false, expected_roles.as_ref());
    }
    let f = Fixture::seeded("stray-token", &["operator"], &["operator.admin"]);
    f.write_column(TARGET, "roles_json", &SqlValue::Text("[]".into()));
    f.write_column(TARGET, "role", &SqlValue::Null);
    f.deny_native(scoped(TARGET, "operator", &[]), None, 400);
    f.deny_native(scoped(TARGET, "node", &[]), None, 400);
}

#[test]
fn wire_rotation_replaces_obsolete_entry_fields_and_every_retired_marker() {
    for marker in [Value::Null, json!(0), json!(1), json!(-1), json!(0.0), json!("0"), json!(false), json!([]), json!({})] {
        let f = Fixture::seeded("obsolete-entry", &["operator"], &["operator.admin"]);
        let admin = f.proof(CALLER, "operator");
        f.edit_entries(TARGET, |entries| {
            entries["operator"]["revokedAtMs"] = marker;
            entries["operator"]["token"] = json!(false);
            entries["operator"]["role"] = json!("obsolete-role");
            entries["operator"]["unknown"] = json!({"preserved": [7, null]});
        });
        let before = f.observe();
        let receipt = f.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&admin), None).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
    }
    let f = Fixture::seeded("empty-object", &["operator"], &["operator.admin"]);
    f.edit_entries(TARGET, |entries| entries["operator"] = json!({}));
    let before = f.observe();
    let receipt = f.owner().handle_token_rotate(&scoped(TARGET, "operator", &[])).unwrap();
    f.rotated(&before, &receipt, TARGET, "operator", &[], false);
}

#[test]
fn wire_rotation_preserves_opaque_metadata_and_safe_lifecycle_values() {
    for (created, last_used) in [
        (None, None), (Some(Value::Null), Some(Value::Null)),
        (Some(json!(0)), Some(json!(0))),
        (Some(json!(0.0)), Some(json!(-0.0))),
        (Some(json!(-0.0)), Some(json!(1.0))),
        (Some(json!(1.0)), Some(json!(9_007_199_254_740_991.0_f64))),
        (Some(json!(9_007_199_254_740_991_i64)), Some(json!(9_007_199_254_740_991_i64))),
    ] {
        let f = Fixture::seeded("metadata", &["operator"], &["operator.admin"]);
        f.edit_entries(TARGET, |entries| {
            let entry = entries["operator"].as_object_mut().unwrap();
            if let Some(value) = created { entry.insert("createdAtMs".into(), value); } else { entry.remove("createdAtMs"); }
            if let Some(value) = last_used { entry.insert("lastUsedAtMs".into(), value); } else { entry.remove("lastUsedAtMs"); }
            entry.insert("issuer".into(), json!({"kind": "shared-gateway-auth", "generation": "DTR-private-issuer", "future": [false, null]}));
            entry.insert("extension".into(), json!(["retain", {"order": 17}]));
            entry.insert("revokedAtMs".into(), json!(5));
            entries["unselected-opaque"] = json!([false, 7]);
        });
        let before = f.observe();
        let receipt = f.owner().handle_token_rotate(&args(TARGET, "operator")).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
    }
    for issuer in [Value::Null, json!(7), json!("opaque"), json!([false])] {
        let f = Fixture::seeded("issuer-opaque", &["operator"], &["operator.admin"]);
        f.edit_entries(TARGET, |entries| entries["operator"]["issuer"] = issuer);
        let before = f.observe();
        let receipt = f.owner().handle_token_rotate(&args(TARGET, "operator")).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
    }
}

#[test]
fn wire_rotation_rejects_non_safe_lifecycle_and_nonobject_selected_entries() {
    for field in ["createdAtMs", "lastUsedAtMs"] {
        for value in [json!(-1), json!(-1.0), json!(1.5), json!(9_007_199_254_740_992_i64), json!(9_007_199_254_740_992.0_f64), json!(u64::MAX), json!("1"), json!(false), json!([]), json!({})] {
            let f = Fixture::seeded("unsafe-lifecycle", &["operator"], &["operator.admin"]);
            let admin = f.proof(CALLER, "operator");
            f.edit_entries(TARGET, |entries| entries["operator"][field] = value);
            f.deny(&admin, scoped(TARGET, "operator", &[]), None, 500);
            f.deny_native(scoped(TARGET, "operator", &[]), None, 500);
        }
    }
    for entry in [Value::Null, json!([]), json!(7), json!("bad"), json!(false)] {
        let f = Fixture::seeded("entry-shape", &["operator"], &["operator.admin"]);
        f.edit_entries(TARGET, |entries| entries["operator"] = entry);
        f.deny_native(scoped(TARGET, "operator", &[]), None, 500);
    }
}

#[test]
fn wire_rotation_target_storage_types_are_strict_and_nonoperator_scopes_are_literal() {
    for (column, raw) in [
        ("roles_json", SqlValue::Null), ("roles_json", SqlValue::Blob(vec![0xff])),
        ("roles_json", SqlValue::Text("DTR-private-bad-roles".into())),
        ("roles_json", SqlValue::Text("null".into())), ("roles_json", SqlValue::Text("{}".into())),
        ("roles_json", SqlValue::Text("[\"operator\",7]".into())),
        ("role", SqlValue::Blob(vec![0xff])),
        ("tokens_json", SqlValue::Blob(vec![0xff])), ("tokens_json", SqlValue::Text("DTR-private-bad-map".into())),
        ("tokens_json", SqlValue::Text("null".into())), ("tokens_json", SqlValue::Text("[]".into())),
    ] {
        let f = Fixture::seeded("target-storage", &["operator"], &["operator.admin"]);
        let admin = f.proof(CALLER, "operator");
        f.write_column(TARGET, column, &raw);
        f.deny(&admin, scoped(TARGET, "operator", &[]), None, 500);
        f.deny_native(scoped(TARGET, "node", &[]), None, 500);
    }
    let f = Fixture::seeded("node-literal", &["operator"], &["operator.admin"]);
    let admin = f.proof(CALLER, "operator");
    f.baseline(TARGET, &["node.invoke", "operator.admin", "other.invoke"]);
    for scopes in [&["node.unknown"][..], &["other.invoke"][..], &["operator.admin"][..], &["node.invoke", "operator.read"][..]] {
        f.deny(&admin, scoped(TARGET, "node", scopes), None, 403);
    }
    let before = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "node", &["node.invoke"]), Some(&admin), None).unwrap();
    f.rotated(&before, &receipt, TARGET, "node", &["node.invoke"], false);
}

#[test]
fn wire_rotation_scope_implications_keep_caller_and_target_ceilings_independent() {
    let f = Fixture::seeded("implications", &["operator"], &["operator.pairing", "operator.write"]);
    let proof = f.proof(CALLER, "operator");
    for scope in ["operator.admin", "operator.approvals", "operator.pairing.extra", "node.invoke"] {
        f.deny(&proof, scoped(CALLER, "operator", &[scope]), None, 403);
    }
    let before = f.observe();
    let scopes = ["operator.cron", "operator.pairing", "operator.read", "operator.sessions.read", "operator.sessions.write", "operator.talk"];
    let receipt = f.owner().rotate_token_with_authority(&scoped(CALLER, "operator", &scopes), Some(&proof), None).unwrap();
    f.rotated(&before, &receipt, CALLER, "operator", &scopes, true);

    let f = Fixture::seeded("result-cap", &["operator"], &["operator.pairing", "operator.read"]);
    let proof = f.proof(CALLER, "operator");
    f.baseline(CALLER, &["operator.admin"]);
    f.deny(&proof, scoped(CALLER, "operator", &["operator.write"]), None, 403);
    let before = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&scoped(CALLER, "operator", &["operator.sessions.read"]), Some(&proof), None).unwrap();
    f.rotated(&before, &receipt, CALLER, "operator", &["operator.sessions.read"], true);
}

#[test]
fn wire_rotation_preserves_legacy_dpa_unicode_and_response_behavior() {
    for (scope, status) in [("\u{85}operator.read\u{85}", None), ("\u{feff}operator.read\u{feff}", Some(403))] {
        let f = Fixture::seeded("legacy-unicode", &["operator"], &["operator.admin"]);
        f.baseline(TARGET, &["operator.admin"]);
        let proof = f.proof(CALLER, "operator");
        let before = f.observe();
        let result = f.owner().rotate_token_for_authenticated_device(&scoped(TARGET, "operator", &[scope]), &proof, None);
        if let Some(status) = status {
            f.error(result, status, &before);
            assert!(f.observe() == before);
        } else {
            let receipt = result.expect("legacy native trim still removes NEL");
            assert!(receipt.get("revision").is_some());
            assert!(receipt.get("scopes").is_none());
            assert_eq!(receipt["tokenDelivery"], "withheld-cross-device");
            assert!(receipt.get("token").is_none());
            assert_eq!(f.observe().entries(TARGET)["operator"]["scopes"], json!(["operator.read"]));
            assert!(f.observe().device(TARGET).get("node_surface_json") == Some(&SqlValue::Null));
            assert!(f.observe().device(TARGET).get("pending_node_surface_json") == Some(&SqlValue::Null));
        }
    }
    let f = Fixture::seeded("legacy-native", &["operator"], &["operator.admin"]);
    let receipt = f.owner().handle_pair_rotate_token(&json!({"deviceId": TARGET})).unwrap();
    assert_eq!(receipt["tokenDelivery"], "in-band");
    assert!(receipt["token"].as_str().is_some_and(|token| !token.is_empty()));
    assert!(receipt.get("revision").is_some());
    assert!(receipt.get("scopes").is_none());
}

#[test]
fn wire_rotation_two_current_admin_owners_serialize_replacements_without_lost_updates() {
    for primary_only in [false, true] {
    let f = Fixture::seeded("admin-race", &["operator"], &["operator.admin"]);
    if primary_only {
        f.write_column(TARGET, "roles_json", &SqlValue::Text("[\" node \",\"\",\"node\"]".into()));
        f.write_column(TARGET, "role", &SqlValue::Text(" operator ".into()));
        f.stale(&f.proof(TARGET, "operator"), 401);
    }
    let writer = f.writer();
    writer.execute_batch(
        "CREATE TABLE dtr_rotation_audit(sequence INTEGER PRIMARY KEY, tokens_json TEXT NOT NULL, roles_json TEXT NOT NULL);
         CREATE TRIGGER dtr_rotation_audit_write AFTER UPDATE OF tokens_json ON device_pairing_paired
         WHEN NEW.device_id = 'dtr-target'
         BEGIN INSERT INTO dtr_rotation_audit(tokens_json, roles_json) VALUES(NEW.tokens_json, NEW.roles_json); END;",
    ).unwrap();
    writer.close().unwrap();
    let first_owner = f.owner();
    let second_owner = DevicePairingOwner::new(&f.db).unwrap();
    let first_proof = f.proof(CALLER, "operator");
    let second_proof = second_owner.authenticate_device_token(&f.observe().token(CALLER, "operator")).unwrap().unwrap();
    let before = f.observe();
    let start = Barrier::new(3);
    let (first, second) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            start.wait();
            first_owner.rotate_token_with_authority(&scoped(TARGET, "operator", &["operator.pairing"]), Some(&first_proof), None)
        });
        let second = scope.spawn(|| {
            start.wait();
            second_owner.rotate_token_with_authority(&scoped(TARGET, "operator", &[]), Some(&second_proof), None)
        });
        start.wait();
        (first.join().unwrap().expect("first independent admin commits"), second.join().unwrap().expect("second independent admin commits"))
    });
    assert_eq!(first["scopes"], json!(["operator.pairing"]));
    assert_eq!(second["scopes"], json!([]));
    for result in [&first, &second] {
        assert_eq!(result.as_object().unwrap().len(), 5);
        assert!(result.get("token").is_none());
        assert_eq!(result["tokenDelivery"], "withheld-cross-device");
    }
    let after = f.observe();
    let observer = Connection::open_with_flags(&f.db, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
    let audit = rows(&observer, "SELECT * FROM dtr_rotation_audit ORDER BY sequence");
    observer.close().unwrap();
    assert_eq!(audit.len(), 2, "both successful commands must update the actual target");
    let expected_roles = if primary_only { json!([" node ", "", "node", "operator"]).to_string() }
        else { text(before.device(TARGET), "roles_json").to_owned() };
    for write in &audit {
        assert_eq!(text(write, "roles_json"), expected_roles, "each serialized issuance has exactly one selected membership");
    }
    let first_write: Value = serde_json::from_str(text(&audit[0], "tokens_json")).unwrap();
    let final_write: Value = serde_json::from_str(text(&audit[1], "tokens_json")).unwrap();
    let intermediate = first_write["operator"]["token"].as_str().unwrap();
    let latest = final_write["operator"]["token"].as_str().unwrap();
    assert!(!intermediate.is_empty() && !latest.is_empty() && intermediate != latest);
    assert!(intermediate != before.token(TARGET, "operator") && latest != before.token(TARGET, "operator"));
    assert!(after.entries(TARGET) == final_write, "final committed credential follows actual SQLite write order");
    assert!(first_write["operator"]["scopes"] != final_write["operator"]["scopes"]);
    f.token_absent(intermediate);
    assert_eq!(after.revision(), before.revision() + 2);
    assert!(after.pending == before.pending);
    assert!(after.device(CALLER) == before.device(CALLER));
    assert!(after.device(BYSTANDER) == before.device(BYSTANDER));
    assert!(after.entries(TARGET)["node"] == before.entries(TARGET)["node"]);
    assert!(after.entries(TARGET)["operator"]["scopes"] == first["scopes"] || after.entries(TARGET)["operator"]["scopes"] == second["scopes"]);
    let final_receipt = if final_write["operator"]["scopes"] == first["scopes"] { &first } else { &second };
    let mut expected_entries = before.entries(TARGET);
    expected_entries["operator"]["token"] = json!(latest);
    expected_entries["operator"]["scopes"] = final_receipt["scopes"].clone();
    expected_entries["operator"]["rotatedAtMs"] = final_receipt["rotatedAtMs"].clone();
    let mut expected = before.clone();
    expected.paired.iter_mut().find(|row| text(row, "device_id") == TARGET).unwrap()
        .insert("tokens_json".into(), SqlValue::Text(expected_entries.to_string()));
    expected.paired.iter_mut().find(|row| text(row, "device_id") == TARGET).unwrap()
        .insert("roles_json".into(), SqlValue::Text(expected_roles));
    *expected.machine.iter_mut().find(|row| text(row, "state_key") == REVISION).unwrap() = after.counter().clone();
    assert!(after == expected, "serial admin rotations preserve every unrelated SQL column and configuration row");
    f.token_absent(&before.token(TARGET, "operator"));
    f.current(&f.proof(TARGET, "operator"));
    f.current(&first_proof);
    second_owner.validate_current_device_authorization(&second_proof).unwrap();
    }
}

#[test]
fn wire_rotation_and_role_revoke_race_consumes_original_self_authority_once() {
    let f = Fixture::seeded("rotate-revoke", &["operator", "node"], &["operator.pairing", "node.invoke"]);
    let rotator = f.owner();
    let revoker = DevicePairingOwner::new(&f.db).unwrap();
    let before = f.observe();
    let bearer = before.token(CALLER, "operator");
    let rotate_proof = f.authenticate(&bearer);
    let revoke_proof = revoker.authenticate_device_token(&bearer).unwrap().unwrap();
    let start = Barrier::new(3);
    let (rotation, retirement) = std::thread::scope(|scope| {
        let rotate = scope.spawn(|| {
            start.wait();
            rotator.rotate_token_with_authority(&args(CALLER, "operator"), Some(&rotate_proof), None)
        });
        let revoke = scope.spawn(|| {
            start.wait();
            revoker.revoke_token_with_authority(&args(CALLER, "operator"), Some(&revoke_proof), None)
        });
        start.wait();
        (rotate.join().unwrap(), revoke.join().unwrap())
    });
    match (rotation, retirement) {
        (Ok(receipt), Err(error)) => {
            f.error::<()>(Err(error), 401, &before);
            f.rotated(&before, &receipt, CALLER, "operator", &["operator.pairing"], true);
        }
        (Err(error), Ok(receipt)) => {
            f.error::<()>(Err(error), 401, &before);
            let after = f.observe();
            let mut expected = before.entries(CALLER);
            expected["operator"]["revokedAtMs"] = receipt["revokedAtMs"].clone();
            assert!(after.entries(CALLER) == expected, "losing rotation cannot issue a replacement after retirement");
            assert_eq!(receipt.as_object().unwrap().len(), 3);
            assert!(receipt["revokedAtMs"].as_i64().is_some_and(|time| time > 0));
            assert_eq!(after.revision(), before.revision() + 1);
            let mut expected_state = before.clone();
            expected_state.paired.iter_mut().find(|row| text(row, "device_id") == CALLER).unwrap()
                .insert("tokens_json".into(), SqlValue::Text(expected.to_string()));
            *expected_state.machine.iter_mut().find(|row| text(row, "state_key") == REVISION).unwrap() = after.counter().clone();
            assert!(after == expected_state, "losing rotation preserves every effect of the committed role retirement");
        }
        _ => panic!("exactly one operation may consume the original self credential"),
    }
    f.token_absent(&bearer);
    f.stale(&rotate_proof, 401);
    f.error(revoker.validate_current_device_authorization(&revoke_proof), 401, &before);
    f.current(&f.proof(CALLER, "node"));
}

#[test]
fn wire_rotation_refusal_preserves_a_competing_committed_retirement_or_approval_change() {
    for change in ["rotate", "revoke", "approval", "remove"] {
        let f = Fixture::seeded("ordered-race", &["operator"], &["operator.pairing", "operator.approvals"]);
        let proof = f.proof(CALLER, "operator");
        let second_owner = DevicePairingOwner::new(&f.db).unwrap();
        match change {
            "rotate" => { second_owner.handle_token_rotate(&args(CALLER, "operator")).unwrap(); }
            "revoke" => { second_owner.handle_token_revoke(&args(CALLER, "operator")).unwrap(); }
            "remove" => { second_owner.handle_pair_revoke(&json!({"deviceId": CALLER})).unwrap(); }
            "approval" => f.baseline(CALLER, &["operator.pairing"]),
            _ => unreachable!(),
        }
        let competing_commit = f.observe();
        f.error(f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &[]), Some(&proof), None), 401, &competing_commit);
        assert!(f.observe() == competing_commit, "refusal must preserve the independently committed state");
        let before = f.observe();
        let receipt = f.owner().handle_token_rotate(&args(TARGET, "operator")).unwrap();
        f.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
    }
}

#[test]
fn wire_rotation_active_marker_variants_require_a_current_caller_even_for_empty_repair() {
    for (marker, active) in [
        (None, true), (Some(Value::Null), true), (Some(json!(0)), true),
        (Some(json!(1)), false), (Some(json!(-1)), false), (Some(json!(0.0)), false),
        (Some(json!(1.5)), false), (Some(json!("0")), false), (Some(json!("")), false),
        (Some(json!(false)), false), (Some(json!(true)), false), (Some(json!([])), false),
        (Some(json!({})), false), (Some(json!(u64::MAX)), false),
    ] {
        let f = Fixture::seeded("caller-markers", &["operator", "node"], &["operator.pairing", "node.invoke"]);
        let proof = f.proof(CALLER, "operator");
        let old = f.observe().token(CALLER, "operator");
        let node = f.proof(CALLER, "node");
        f.edit_entries(CALLER, |entries| {
            if let Some(marker) = marker { entries["operator"]["revokedAtMs"] = marker; }
            else { entries["operator"].as_object_mut().unwrap().remove("revokedAtMs"); }
        });
        if active {
            f.current(&proof);
            let before = f.observe();
            let receipt = f.owner().rotate_token_with_authority(&scoped(CALLER, "operator", &[]), Some(&proof), None).unwrap();
            f.rotated(&before, &receipt, CALLER, "operator", &[], true);
        } else {
            f.token_absent(&old);
            f.stale(&proof, 401);
            f.deny(&proof, scoped(CALLER, "operator", &[]), None, 401);
            let before = f.observe();
            let receipt = f.owner().handle_token_rotate(&scoped(CALLER, "operator", &[])).unwrap();
            f.rotated(&before, &receipt, CALLER, "operator", &[], false);
        }
        f.current(&node);
        f.stale(&proof, 401);
        f.token_absent(&old);
    }
}

#[test]
fn wire_rotation_absent_reply_readback_does_not_reveal_or_reissue_replacement() {
    let f = Fixture::seeded("lost-reply", &["operator"], &["operator.pairing"]);
    let proof = f.proof(CALLER, "operator");
    let before = f.observe();
    // The caller deliberately discards a successful result. This is an owner
    // unknown-delivery oracle, not a simulated TCP write-failure assertion.
    drop(f.owner().rotate_token_with_authority(&args(CALLER, "operator"), Some(&proof), None).unwrap());
    let committed = f.observe();
    let successor = committed.token(CALLER, "operator");
    f.secrets.borrow_mut().push(successor.clone());
    assert_eq!(committed.revision(), before.revision() + 1);
    f.deny(&proof, args(CALLER, "operator"), None, 401);
    let listed = f.owner().handle_pair_list(&json!({})).unwrap();
    f.assert_secret_free(&listed.to_string());
    assert!(f.observe() == committed, "list must not manufacture replacement or repair an unknown result");
    f.current(&f.authenticate(&successor));
    f.token_absent(&before.token(CALLER, "operator"));
}

#[test]
fn wire_rotation_two_owner_distinct_roles_preserve_both_committed_replacements() {
    let f = Fixture::seeded("distinct-role-race", &["operator"], &["operator.admin"]);
    let first_owner = f.owner();
    let second_owner = DevicePairingOwner::new(&f.db).unwrap();
    let first_proof = f.proof(CALLER, "operator");
    let second_proof = second_owner.authenticate_device_token(&f.observe().token(CALLER, "operator")).unwrap().unwrap();
    let before = f.observe();
    let start = Barrier::new(3);
    let (operator, node) = std::thread::scope(|scope| {
        let first = scope.spawn(|| {
            start.wait();
            first_owner.rotate_token_with_authority(&args(TARGET, "operator"), Some(&first_proof), None)
        });
        let second = scope.spawn(|| {
            start.wait();
            second_owner.rotate_token_with_authority(&args(TARGET, "node"), Some(&second_proof), None)
        });
        start.wait();
        (first.join().unwrap().expect("operator replacement commits"), second.join().unwrap().expect("node replacement commits"))
    });
    let after = f.observe();
    assert_eq!(after.revision(), before.revision() + 2);
    let mut expected_entries = before.entries(TARGET);
    for (role, receipt, scopes) in [("operator", &operator, &["operator.pairing"][..]), ("node", &node, &["node.invoke"][..])] {
        assert_eq!(receipt.as_object().unwrap().len(), 5);
        assert_eq!(receipt["deviceId"], TARGET);
        assert_eq!(receipt["role"], role);
        assert_eq!(receipt["tokenDelivery"], "withheld-cross-device");
        assert_eq!(receipt["scopes"], json!(scopes));
        assert!(receipt.get("token").is_none());
        assert!(receipt["rotatedAtMs"].as_i64().is_some_and(|time| time > 0));
        let successor = after.token(TARGET, role);
        assert!(successor != before.token(TARGET, role));
        expected_entries[role]["token"] = json!(successor);
        expected_entries[role]["scopes"] = json!(scopes);
        expected_entries[role]["rotatedAtMs"] = receipt["rotatedAtMs"].clone();
        f.token_absent(&before.token(TARGET, role));
        f.current(&f.proof(TARGET, role));
    }
    let mut expected = before.clone();
    let target = expected.paired.iter_mut().find(|row| text(row, "device_id") == TARGET).unwrap();
    target.insert("tokens_json".into(), SqlValue::Text(expected_entries.to_string()));
    target.insert("node_surface_json".into(), SqlValue::Null);
    target.insert("pending_node_surface_json".into(), SqlValue::Null);
    *expected.machine.iter_mut().find(|row| text(row, "state_key") == REVISION).unwrap() = after.counter().clone();
    assert!(after == expected, "one transaction must retain the other's updated role and all unrelated bytes");
    f.current(&first_proof);
    second_owner.validate_current_device_authorization(&second_proof).unwrap();
}


#[test]
fn wire_rotation_target_nondisclosure_precedes_malformed_or_absent_target_reads() {
    let fixture = Fixture::seeded("target-address", &["operator", "node"], &["operator.pairing", "node.invoke"]);
    let proof = fixture.proof(CALLER, "operator");
    let healthy = fixture.observe();
    fixture.write_column(TARGET, "tokens_json", &SqlValue::Blob(vec![0xff, 7]));
    fixture.write_column(TARGET, "roles_json", &SqlValue::Null);
    for role in ["operator", "node", "nonexistent"] {
        let before = fixture.observe();
        let existing = fixture.error(fixture.owner().rotate_token_with_authority(&args(TARGET, role), Some(&proof), None), 403, &before);
        let absent = fixture.error(fixture.owner().rotate_token_with_authority(&args(ABSENT, role), Some(&proof), None), 403, &before);
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
    let receipt = fixture.owner().rotate_token_with_authority(&args(CALLER, "operator"), Some(&proof), None)
        .expect("same pairing-only caller still has its permitted self operation");
    fixture.rotated(&before, &receipt, CALLER, "operator", &["operator.pairing"], true);
}


#[test]
fn wire_rotation_pairing_permission_and_singular_headers_do_not_promote_proofs() {
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
    let receipt = fixture.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&proof), Some(" operator.admin "))
        .expect("admin-only sealed grant and singular admin ceiling are sufficient");
    fixture.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
}


#[test]
fn wire_rotation_sealed_issuer_and_projection_control_identity_and_permissions() {
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
    let receipt = fixture.owner().rotate_token_with_authority(&args(CALLER, "operator"), Some(&edited), None)
        .expect("sealed self identity and grant survive projection edits");
    fixture.rotated(&before, &receipt, CALLER, "operator", &["operator.pairing"], true);
    fixture.stale(&proof, 401);
    fixture.stale(&edited, 401);
}


#[test]
fn wire_rotation_currentness_precedes_operator_policy_for_every_sealed_role() {
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
fn wire_rotation_caller_identity_failures_keep_established_unauthorized_status() {
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
        fixture.deny(&proof, scoped(CALLER, "operator", &[]), None, 401);
        fixture.stale(&proof, 401);
    }
}


#[test]
fn wire_rotation_caller_corruption_and_null_approval_fallback_are_distinct() {
    for (column, value) in [
        ("roles_json", SqlValue::Null), ("roles_json", SqlValue::Blob(vec![0xff])),
        ("roles_json", SqlValue::Text("{}".into())), ("roles_json", SqlValue::Text("[\"operator\",7]".into())),
        ("tokens_json", SqlValue::Text("DTR-private-invalid-map".into())),
        ("tokens_json", SqlValue::Text("[]".into())), ("tokens_json", SqlValue::Blob(vec![0xff])),
        ("approved_scopes_json", SqlValue::Text("null".into())),
        ("approved_scopes_json", SqlValue::Text("[\"operator.admin\",false]".into())),
        ("approved_scopes_json", SqlValue::Blob(vec![0xff])),
    ] {
        let fixture = Fixture::seeded("corrupt-caller", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.write_column(CALLER, column, &value);
        fixture.deny(&proof, args(TARGET, "operator"), None, 500);
        fixture.deny(&proof, scoped(CALLER, "operator", &[]), None, 500);
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
    let receipt = fixture.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&proof), None)
        .expect("SQL NULL approval uses existing valid legacy scopes");
    fixture.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
}


#[test]
fn wire_rotation_intersects_sealed_live_approval_and_target_grants() {
    let fixture = Fixture::seeded("original-cap", &["operator"], &["operator.pairing"]);
    let proof = fixture.proof(CALLER, "operator");
    fixture.grant(CALLER, "operator", &["operator.admin"]);
    fixture.baseline(CALLER, &["operator.admin"]);
    fixture.deny(&proof, args(TARGET, "operator"), None, 403);
    fixture.deny(&proof, args(CALLER, "operator"), None, 403); // Cannot reissue the now wider grant.
    let fresh = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&fresh), None)
        .expect("new authentication admits the approved wider grant");
    fixture.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
    let before = fixture.observe();
    let receipt = fixture.owner().rotate_token_with_authority(&scoped(CALLER, "operator", &[]), Some(&proof), None)
        .expect("sealed pairing grant can issue an empty repair without inheriting the wider old target grant");
    fixture.rotated(&before, &receipt, CALLER, "operator", &[], true);

    for narrow_approval in [false, true] {
        let fixture = Fixture::seeded("live-cap", &["operator"], &["operator.admin"]);
        let proof = fixture.proof(CALLER, "operator");
        fixture.grant(CALLER, "operator", &["operator.pairing"]);
        if narrow_approval { fixture.baseline(CALLER, &["operator.pairing"]); }
        fixture.deny(&proof, args(TARGET, "operator"), None, 403);
        let before = fixture.observe();
        let receipt = fixture.owner().rotate_token_with_authority(&args(CALLER, "operator"), Some(&proof), None)
            .expect("existing management intersection semantics permit narrowed pairing-only self rotation");
        fixture.rotated(&before, &receipt, CALLER, "operator", &["operator.pairing"], true);
    }
    let fixture = Fixture::seeded("withdrawn-approval", &["operator"], &["operator.pairing", "operator.approvals"]);
    let proof = fixture.proof(CALLER, "operator");
    fixture.baseline(CALLER, &["operator.pairing"]);
    fixture.deny(&proof, args(CALLER, "operator"), None, 401);
}


#[test]
fn wire_rotation_revision_missing_corrupt_and_overflow_roll_back_on_reopen() {
    for raw in [None, Some(SqlValue::Text("DTR-private-invalid-revision".into())),
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
        let receipt = fixture.owner().rotate_token_with_authority(&args(TARGET, "node"), Some(&fresh), None)
            .expect("explicit repaired-counter control commits once");
        fixture.rotated(&healthy, &receipt, TARGET, "node", &["node.invoke"], false);
    }
}


#[test]
fn wire_rotation_last_revision_and_legacy_bootstrap_behaviors_remain_distinct() {
    let fixture = Fixture::seeded("last-counter", &["operator"], &["operator.admin"]);
    fixture.set_revision(&SqlValue::Text(format!("\"{}\"", u64::MAX - 1)));
    let before = fixture.observe();
    let receipt = fixture.owner().handle_token_rotate(&args(TARGET, "node"))
        .expect("last representable increment may commit");
    fixture.rotated(&before, &receipt, TARGET, "node", &["node.invoke"], false);
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
fn wire_rotation_sql_and_row_count_faults_roll_back_then_recover() {
    for fault in ["zero-target", "target-error", "zero-revision", "revision-error"] {
        let mut fixture = Fixture::seeded(fault, &["operator"], &["operator.admin"]);
        let admin = fixture.proof(CALLER, "operator");
        let sql = match fault {
            "zero-target" => "CREATE TRIGGER dtr_fault BEFORE UPDATE OF tokens_json ON device_pairing_paired WHEN OLD.device_id = 'dtr-target' BEGIN SELECT RAISE(IGNORE); END;",
            "target-error" => "CREATE TRIGGER dtr_fault BEFORE UPDATE OF tokens_json ON device_pairing_paired WHEN OLD.device_id = 'dtr-target' BEGIN SELECT RAISE(ABORT, 'DTR-private-storage-fault'); END;",
            "zero-revision" => "CREATE TRIGGER dtr_fault BEFORE UPDATE OF value_json ON config_machine_state WHEN OLD.state_key = 'devicePairing.revision' BEGIN SELECT RAISE(IGNORE); END;",
            "revision-error" => "CREATE TRIGGER dtr_fault BEFORE UPDATE OF value_json ON config_machine_state WHEN OLD.state_key = 'devicePairing.revision' BEGIN SELECT RAISE(ABORT, 'DTR-private-storage-fault'); END;",
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
        writer.execute_batch("DROP TRIGGER dtr_fault").unwrap();
        writer.close().unwrap();
        let receipt = fixture.owner().rotate_token_with_authority(&args(TARGET, "node"), Some(&fresh), None)
            .expect("healthy control after removing precisely the owned fault");
        fixture.rotated(&before, &receipt, TARGET, "node", &["node.invoke"], false);
    }
}


#[test]
fn wire_rotation_immediate_write_lock_precedes_policy_and_target_read_decisions() {
    let fixture = Fixture::seeded("reserved-lock", &["operator"], &["operator.read"]);
    let proof = fixture.proof(CALLER, "operator");
    let writer = fixture.writer();
    let journal: String = writer.query_row("PRAGMA journal_mode=DELETE", [], |row| row.get(0)).unwrap();
    assert_eq!(journal, "delete");
    writer.execute_batch("BEGIN IMMEDIATE").unwrap();
    fixture.current(&proof); // A retained read must work under a RESERVED writer.
    let before = fixture.observe();
    for result in [
        fixture.owner().rotate_token_with_authority(&args(CALLER, "operator"), Some(&proof), None),
        fixture.owner().rotate_token_with_authority(&args(ABSENT, "operator"), None, None),
    ] {
        let error = fixture.error(result, 500, &before);
        assert_eq!(error.to_string(), "failed to begin token rotation transaction");
    }
    assert!(fixture.observe() == before);
    writer.execute_batch("ROLLBACK").unwrap();
    writer.close().unwrap();
    fixture.deny(&proof, args(CALLER, "operator"), None, 403);
    fixture.deny_native(args(ABSENT, "operator"), None, 400);
    let receipt = fixture.owner().handle_token_rotate(&args(TARGET, "operator"))
        .expect("same owner commits after only the blocking connection is released");
    fixture.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
}


#[test]
fn wire_rotation_failed_commit_rolls_back_token_surfaces_and_counter_durably() {
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
        let result = fixture.owner().rotate_token_with_authority(
            &args(TARGET, "node"), if paired { Some(&proof) } else { None }, None,
        );
        let error = fixture.error(result, 500, &before);
        assert_eq!(error.to_string(), "failed to commit token rotation transaction", "failure must reach COMMIT");
        reader.execute_batch("COMMIT").unwrap();
        reader.close().unwrap();
        assert!(fixture.observe() == before, "known failed COMMIT rolls back every durable write");
        fixture.current(&proof);
        fixture.current(&fixture.proof(TARGET, "node"));
        fixture.reopen();
        assert!(fixture.observe() == before, "complete rollback survives actual owner reopen");
        let fresh = fixture.proof(CALLER, "operator");
        let receipt = fixture.owner().rotate_token_with_authority(
            &args(TARGET, "node"), if paired { Some(&fresh) } else { None }, None,
        ).expect("explicit recovery after reader close commits exactly once");
        fixture.rotated(&before, &receipt, TARGET, "node", &["node.invoke"], false);
    }
}


#[test]
fn wire_rotation_two_owner_self_race_commits_once_and_rejects_retired_proof() {
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
            first_owner.rotate_token_with_authority(&args(CALLER, "operator"), Some(&first_proof), None)
        });
        let second = scope.spawn(|| {
            start.wait();
            second_owner.rotate_token_with_authority(&args(CALLER, "operator"), Some(&second_proof), None)
        });
        start.wait();
        (first.join().unwrap(), second.join().unwrap())
    });
    let (receipt, error) = match results {
        (Ok(receipt), Err(error)) | (Err(error), Ok(receipt)) => (receipt, error),
        _ => panic!("exactly one of two independent owners must commit self rotation"),
    };
    fixture.error::<()>(Err(error), 401, &before);
    fixture.rotated(&before, &receipt, CALLER, "operator", &["operator.pairing"], true);
    fixture.stale(&first_proof, 401);
    fixture.error(second_owner.validate_current_device_authorization(&second_proof), 401, &before);
    fixture.current(&fixture.proof(CALLER, "node"));
    drop(second_owner);
}


#[test]
fn wire_rotation_storage_read_failure_is_redacted_without_repair_or_mutation() {
    let fixture = Fixture::seeded("missing-table", &["operator"], &["operator.admin"]);
    let proof = fixture.proof(CALLER, "operator");
    let before = fixture.observe();
    let writer = fixture.writer();
    writer.execute_batch("ALTER TABLE device_pairing_paired RENAME TO dtr_owned_unavailable_pairings").unwrap();
    writer.close().unwrap();
    for result in [
        fixture.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&proof), None),
        fixture.owner().handle_token_rotate(&args(TARGET, "operator")),
    ] {
        let error = fixture.error(result, 500, &before);
        assert_eq!(error.to_string(), "stored pairing authority is invalid");
    }
    let writer = fixture.writer();
    writer.execute_batch("ALTER TABLE dtr_owned_unavailable_pairings RENAME TO device_pairing_paired").unwrap();
    writer.close().unwrap();
    assert!(fixture.observe() == before);
    fixture.current(&proof);
    let receipt = fixture.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&proof), None)
        .expect("restored table permits the unchanged caller authority");
    fixture.rotated(&before, &receipt, TARGET, "operator", &["operator.pairing"], false);
}

#[test]
fn wire_rotation_minted_nel_literal_never_becomes_admin_in_existing_consumers() {
    for admin_baseline in [false, true] {
    let mut f = Fixture::seeded("minted-nel", &["operator"], &["operator.admin"]);
    let literal = "operator.admin\u{85}";
    let desired = [literal, "operator.pairing", "operator.write"];
    let expected = [literal, "operator.pairing", "operator.read", "operator.write"];
    f.baseline(TARGET, if admin_baseline { &["operator.admin"][..] } else { &desired[..] });
    let admin = f.proof(CALLER, "operator");
    let before = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &desired), Some(&admin), None).unwrap();
    f.rotated(&before, &receipt, TARGET, "operator", &expected, false);
    let bearer = f.observe().token(TARGET, "operator");
    f.reopen();
    let successor = f.authenticate(&bearer);
    assert_eq!(successor.scopes, expected);
    f.current(&successor);
    for (device, role) in [(CALLER, "operator"), (TARGET, "node")] {
        let state = f.observe();
        f.error(f.owner().rotate_token_for_authenticated_device(&args(device, role), &successor, None), 403, &state);
        assert!(f.observe() == state, "legacy DPA must not promote the literal to admin");
        f.error(f.owner().revoke_token_with_authority(&args(device, role), Some(&successor), None), 403, &state);
        assert!(f.observe() == state, "DRT must not promote the literal to admin");
        f.error(f.owner().revoke_pairing_for_authenticated_device(&args(device, role), &successor, None), 403, &state);
        assert!(f.observe() == state, "DVR cross-device/mixed-role policy must still require real admin");
        f.deny(&successor, scoped(device, role, &[]), None, 403);
        f.current(&successor);
    }
    let state = f.observe();
    let own = f.owner().rotate_token_with_authority(&scoped(TARGET, "operator", &desired), Some(&successor), None)
        .expect("genuine pairing permits the same literal-bearing caller's own rotation");
    f.rotated(&state, &own, TARGET, "operator", &expected, true);
    f.stale(&successor, 401);
    let own_successor = f.proof(TARGET, "operator");
    let state = f.observe();
    let legacy = f.owner().rotate_token_for_authenticated_device(&args(TARGET, "operator"), &own_successor, None)
        .expect("genuine write permits omitted-scope DPA self rotation without reinterpreting the literal");
    assert_eq!(legacy["tokenDelivery"], "in-band");
    let legacy_successor = f.authenticate(legacy["token"].as_str().unwrap());
    assert_eq!(legacy_successor.scopes, expected);
    assert_eq!(f.observe().revision(), state.revision() + 1);
    f.current(&legacy_successor);
    f.stale(&own_successor, 401);
    let state = f.observe();
    let retired = f.owner().revoke_token_with_authority(&args(TARGET, "operator"), Some(&legacy_successor), None)
        .expect("genuine pairing and the exact literal grant permit DRT self retirement");
    assert_eq!(retired["deviceId"], TARGET);
    assert_eq!(f.observe().revision(), state.revision() + 1);
    f.stale(&legacy_successor, 401);
    let true_admin = f.proof(CALLER, "operator");
    let state = f.observe();
    let node = f.owner().rotate_token_with_authority(&scoped(TARGET, "node", &[]), Some(&true_admin), None).unwrap();
    f.rotated(&state, &node, TARGET, "node", &[], false);
    }
}

#[test]
fn wire_rotation_literal_baseline_and_bom_requests_have_one_scope_meaning() {
    let f = Fixture::seeded("literal-baseline", &["operator"], &["operator.admin"]);
    let literal = "operator.admin\u{85}";
    f.baseline(TARGET, &[literal]);
    for request in ["operator.admin", "\u{feff}operator.admin\u{feff}"] {
        f.deny_native(scoped(TARGET, "operator", &[request]), None, 403);
    }
    let admin = f.proof(CALLER, "operator");
    let state = f.observe();
    f.error(f.owner().rotate_token_for_authenticated_device(&scoped(TARGET, "operator", &[literal]), &admin, None), 403, &state);
    assert!(f.observe() == state, "DPA input trim cannot invent canonical admin in a literal-only baseline");
    let receipt = f.owner().handle_token_rotate(&scoped(TARGET, "operator", &[literal])).unwrap();
    f.rotated(&state, &receipt, TARGET, "operator", &[literal], false);
    let literal_proof = f.proof(TARGET, "operator");
    f.deny(&literal_proof, scoped(TARGET, "operator", &[]), None, 403);
    f.baseline(TARGET, &["\u{feff}operator.admin\u{feff}"]);
    let state = f.observe();
    let receipt = f.owner().handle_token_rotate(&scoped(TARGET, "operator", &["\u{feff}operator.admin\u{feff}"])).unwrap();
    f.rotated(&state, &receipt, TARGET, "operator", &["operator.admin", "operator.read", "operator.write"], false);
    let canonical = f.proof(TARGET, "operator");
    let state = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&scoped(BYSTANDER, "node", &[]), Some(&canonical), None).unwrap();
    f.rotated(&state, &receipt, BYSTANDER, "node", &[], false);
}

#[test]
fn wire_rotation_native_and_legacy_producers_preserve_canonical_literal_grants() {
    let literal = "operator.admin\u{85}";
    let stored = [literal, "\u{feff}operator.write\u{feff}", "operator.pairing", literal];
    let expected = [literal, "operator.write", "operator.pairing"];
    let f = Fixture::seeded("producer-scopes", &["operator"], &stored);
    let approved = f.proof(CALLER, "operator");
    assert_eq!(approved.scopes, expected, "native approval canonicalizes without adding implied read");
    f.current(&approved);
    f.deny(&approved, scoped(TARGET, "operator", &[]), None, 403);
    let native_token = f.native_rotate(CALLER, "operator");
    let native = f.authenticate(&native_token);
    assert_eq!(native.scopes, expected, "native reissue preserves the literal and legacy result ordering");
    f.current(&native);
    f.stale(&approved, 401);
    f.deny(&native, scoped(TARGET, "operator", &[]), None, 403);
    f.pair("dtr-scope-manager", &["operator"], &["operator.admin"]);
    let manager = f.proof("dtr-scope-manager", "operator");
    let old = f.observe();
    let receipt = f.owner().rotate_token_for_authenticated_device(&args(CALLER, "operator"), &manager, None).unwrap();
    assert_eq!(receipt["tokenDelivery"], "withheld-cross-device");
    assert!(receipt.get("token").is_none());
    assert_eq!(f.observe().revision(), old.revision() + 1);
    let omitted = f.proof(CALLER, "operator");
    assert_eq!(omitted.scopes, expected, "omitted DPA scopes use credential canonicalization");
    f.current(&omitted);
    f.stale(&native, 401);
    f.deny(&omitted, scoped(TARGET, "operator", &[]), None, 403);
    // Fresh authentication and sealed admission also canonicalize persisted strings,
    // rather than relying solely on the producer having normalized them already.
    f.grant(CALLER, "operator", &stored);
    let decoded = f.proof(CALLER, "operator");
    assert_eq!(decoded.scopes, expected);
    f.current(&decoded);
    f.deny(&decoded, scoped(TARGET, "operator", &[]), None, 403);
}

#[test]
fn wire_rotation_pending_scope_identity_distinguishes_nel_and_reuses_bom_subsets() {
    let f = Fixture::seeded("pending-scope-identity", &["operator"], &["operator.admin"]);
    for (index, first_scopes, second_scopes, reuses) in [
        (0, vec!["operator.admin\u{85}"], vec!["operator.admin"], false),
        (1, vec!["operator.admin"], vec!["operator.admin\u{85}"], false),
        (2, vec!["\u{feff}operator.read\u{feff}", "operator.write"], vec!["operator.read"], true),
        (3, vec!["operator.read", "operator.write"], vec!["\u{feff}operator.read\u{feff}"], true),
    ] {
        let device = format!("dtr-pending-scope-{index}");
        let initial = f.observe();
        let first = f.request(&device, "same-key", &["operator", "\u{85}node\u{85}"], &first_scopes);
        let before = f.observe();
        let second = f.request(&device, "same-key", &["operator", "node"], &second_scopes);
        assert_eq!(first["created"], true);
        assert_eq!(second["created"], !reuses);
        assert_eq!(first["requestId"] == second["requestId"], reuses);
        let after = f.observe();
        assert!(after.paired == initial.paired && after.machine == initial.machine);
        let matching: Vec<_> = after.pending.iter().filter(|row| text(row, "device_id") == device).collect();
        assert_eq!(matching.len(), 1);
        assert_eq!(text(matching[0], "request_id"), second["requestId"].as_str().unwrap());
        if reuses {
            assert_eq!(text(matching[0], "scopes_json"), json!(first_scopes).to_string());
            assert_eq!(text(matching[0], "roles_json"), json!(["operator", "\u{85}node\u{85}"]).to_string(), "role sets retain the native normalization contract");
        } else {
            assert!(!after.pending.iter().any(|row| text(row, "request_id") == first["requestId"].as_str().unwrap()));
            assert_eq!(text(matching[0], "scopes_json"), json!(second_scopes).to_string());
        }
        let unaffected_before: Vec<_> = before.pending.iter().filter(|row| text(row, "device_id") != device).collect();
        let unaffected_after: Vec<_> = after.pending.iter().filter(|row| text(row, "device_id") != device).collect();
        assert!(unaffected_before == unaffected_after, "scope identity changes only this pending device");
    }
}

#[test]
fn wire_rotation_primary_only_successors_reopen_and_later_withdrawal_retires_them() {
    for role in ["operator", "node"] {
        for missing_entry in [false, true] {
            let mut f = Fixture::seeded("primary-successor", &["operator"], &["operator.admin"]);
            let admin = f.proof(CALLER, "operator");
            let predecessor = f.proof(TARGET, role);
            let other = if role == "operator" { "node" } else { "operator" };
            let original = json!([format!(" {other} "), "", other]);
            f.write_column(TARGET, "roles_json", &SqlValue::Text(format!("  {original}\n")));
            f.write_column(TARGET, "role", &SqlValue::Text(format!(" {role} ")));
            if missing_entry { f.edit_entries(TARGET, |entries| { entries.as_object_mut().unwrap().remove(role); }); }
            f.stale(&predecessor, 401);
            f.deny(&predecessor, scoped(TARGET, role, &[]), None, 401);
            let scopes = if role == "operator" { &["operator.pairing"][..] } else { &["node.invoke"][..] };
            let mut materialized = original.clone();
            materialized.as_array_mut().unwrap().push(json!(role));
            let before = f.observe();
            let receipt = f.owner().rotate_token_with_authority(&scoped(TARGET, role, scopes), Some(&admin), None).unwrap();
            f.rotated_with_roles(&before, &receipt, TARGET, role, scopes, false, Some(&materialized));
            f.stale(&predecessor, 401);
            let committed = f.observe();
            let bearer = committed.token(TARGET, role);
            f.reopen();
            assert!(f.observe() == committed);
            let successor = f.authenticate(&bearer);
            f.current(&successor);
            if role == "operator" {
                let receipt = f.owner().rotate_token_with_authority(&args(TARGET, role), Some(&successor), None).unwrap();
                f.rotated(&committed, &receipt, TARGET, role, scopes, true);
                f.stale(&successor, 401);
            } else {
                f.deny(&successor, scoped(TARGET, role, &[]), None, 403);
            }
            let latest = f.proof(TARGET, role);
            f.write_column(TARGET, "roles_json", &SqlValue::Text(original.to_string()));
            f.stale(&latest, 401);
            f.deny(&latest, scoped(TARGET, role, &[]), None, 401);
            assert_eq!(text(f.observe().device(TARGET), "role"), format!(" {role} "), "primary remains while role-array withdrawal retires the credential");
        }
    }
}

#[test]
fn wire_rotation_unselected_primary_never_revives_and_existing_role_bytes_survive() {
    let f = Fixture::seeded("unselected-primary", &["operator"], &["operator.admin"]);
    let admin = f.proof(CALLER, "operator");
    let old_node = f.proof(TARGET, "node");
    let raw_roles = " [ \" operator \", \"\", \"operator\" ] \n";
    f.write_column(TARGET, "roles_json", &SqlValue::Text(raw_roles.into()));
    f.write_column(TARGET, "role", &SqlValue::Text(" node ".into()));
    f.stale(&old_node, 401);
    let state = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&args(TARGET, "operator"), Some(&admin), None).unwrap();
    f.rotated(&state, &receipt, TARGET, "operator", &["operator.pairing"], false);
    assert_eq!(text(f.observe().device(TARGET), "roles_json"), raw_roles);
    f.stale(&old_node, 401);
    f.deny(&old_node, scoped(TARGET, "node", &[]), None, 401);
    let state = f.observe();
    let receipt = f.owner().rotate_token_with_authority(&args(TARGET, "node"), Some(&admin), None).unwrap();
    f.rotated_with_roles(&state, &receipt, TARGET, "node", &["node.invoke"], false, Some(&json!([" operator ", "", "operator", "node"])));
    f.stale(&old_node, 401);
    f.current(&f.proof(TARGET, "node"));
}

#[test]
fn wire_rotation_primary_materialization_rolls_back_with_target_revision_and_commit_faults() {
    for fault in ["zero-target", "target-error", "zero-revision", "revision-error", "revision-missing", "revision-overflow", "commit"] {
        let mut f = Fixture::seeded(fault, &["operator"], &["operator.admin"]);
        f.write_column(TARGET, "roles_json", &SqlValue::Text("[ \"operator\" ]".into()));
        f.write_column(TARGET, "role", &SqlValue::Text("node".into()));
        let old_node = f.proof(TARGET, "node");
        f.stale(&old_node, 401);
        let healthy = f.observe();
        let writer = f.writer();
        let sql = match fault {
            "zero-target" => "CREATE TRIGGER dtr_primary_fault BEFORE UPDATE OF tokens_json ON device_pairing_paired WHEN OLD.device_id = 'dtr-target' BEGIN SELECT RAISE(IGNORE); END;",
            "target-error" => "CREATE TRIGGER dtr_primary_fault BEFORE UPDATE OF roles_json ON device_pairing_paired WHEN OLD.device_id = 'dtr-target' BEGIN SELECT RAISE(ABORT, 'DTR-private-primary-fault'); END;",
            "zero-revision" => "CREATE TRIGGER dtr_primary_fault BEFORE UPDATE OF value_json ON config_machine_state WHEN OLD.state_key = 'devicePairing.revision' BEGIN SELECT RAISE(IGNORE); END;",
            "revision-error" => "CREATE TRIGGER dtr_primary_fault BEFORE UPDATE OF value_json ON config_machine_state WHEN OLD.state_key = 'devicePairing.revision' BEGIN SELECT RAISE(ABORT, 'DTR-private-primary-fault'); END;",
            "revision-missing" => "DELETE FROM config_machine_state WHERE state_key = 'devicePairing.revision';",
            "revision-overflow" => "UPDATE config_machine_state SET value_json = '\"18446744073709551615\"' WHERE state_key = 'devicePairing.revision';",
            "commit" => "PRAGMA journal_mode=DELETE;",
            _ => unreachable!(),
        };
        writer.execute_batch(sql).unwrap();
        writer.close().unwrap();
        let reader = if fault == "commit" {
            let reader = Connection::open_with_flags(&f.db, OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
            reader.execute_batch("BEGIN DEFERRED").unwrap();
            let count: i64 = reader.query_row("SELECT COUNT(*) FROM device_pairing_paired", [], |row| row.get(0)).unwrap();
            assert_eq!(count, 3);
            Some(reader)
        } else { None };
        let before = f.observe();
        let admin = f.proof(CALLER, "operator");
        let error = f.error(f.owner().rotate_token_with_authority(&args(TARGET, "node"), Some(&admin), None), 500, &before);
        if fault == "commit" { assert_eq!(error.to_string(), "failed to commit token rotation transaction"); }
        if let Some(reader) = reader { reader.execute_batch("COMMIT").unwrap(); reader.close().unwrap(); }
        assert!(f.observe() == before, "failed token replacement cannot materialize role approval");
        f.stale(&old_node, 401);
        f.reopen();
        assert!(f.observe() == before);
        f.stale(&f.proof(TARGET, "node"), 401);
        let writer = f.writer();
        if matches!(fault, "revision-missing" | "revision-overflow") {
            let counter = healthy.counter();
            writer.execute("INSERT INTO config_machine_state(state_key, value_json, updated_at_ms) VALUES (?1, ?2, ?3) ON CONFLICT(state_key) DO UPDATE SET value_json = excluded.value_json, updated_at_ms = excluded.updated_at_ms",
                params![REVISION, counter.get("value_json").unwrap(), counter.get("updated_at_ms").unwrap()]).unwrap();
        } else if fault != "commit" { writer.execute_batch("DROP TRIGGER dtr_primary_fault").unwrap(); }
        writer.close().unwrap();
        assert!(f.observe() == healthy, "repair removes only the injected refusal");
        let admin = f.proof(CALLER, "operator");
        let receipt = f.owner().rotate_token_with_authority(&args(TARGET, "node"), Some(&admin), None).unwrap();
        f.rotated_with_roles(&healthy, &receipt, TARGET, "node", &["node.invoke"], false, Some(&json!(["operator", "node"])));
    }
}
