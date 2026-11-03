use super::super::{dispatch_public_settings_command, SettingsService};
use mahayana_host_runtime::durable_file_policy::HOST_CONFIG_REVISION_KEY_FILE_NAME;
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};
use std::sync::mpsc::{self, Receiver, SyncSender};
use std::time::{Duration, Instant};
use mahayana_host_runtime::device_pairing_owner::{DevicePairingOwner, DeviceTokenAuthorization};
use mahayana_host_runtime::gateway_config::GatewayServerConfig;
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandReport, GatewayEventHub, GatewayServer, GatewayServerDeps,
    start_gateway_server,
};
use rusqlite::{Connection, OpenFlags, params, types::Value as SqlValue};

const AGENT_SECRET: &str = "synthetic-agent-credential-for-contract";
const SKILL_SECRET: &str = "synthetic-skill-credential-for-contract";
const ENV_SECRET: &str = "synthetic-env-value-for-contract";

struct Fixture {
    root: PathBuf,
    owner: Arc<SettingsService>,
}

impl Fixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-config-public-{}", uuid::Uuid::new_v4()));
        fs::create_dir(&root).expect("exclusive private fixture directory");
        let path = root.join("settings.json");
        fs::write(&path, json!({
            "version":1, "notifications":{"isEnabled":false},
            "inferenceProvider":"fabushi", "userTimeZone":"UTC", "localToolPermission":"ask",
            "agents":{"entries":{"primary":{"name":"Primary", "apiKey":AGENT_SECRET, "maxTokens":4096}}},
            "skills":{"entries":{"demo":{"enabled":true, "apiKey":SKILL_SECRET, "env":{"REGION":ENV_SECRET}}}},
        }).to_string()).unwrap();
        Self { root, owner:Arc::new(SettingsService::new(path)) }
    }

    fn call(&self, method: &str, args: &Value) -> Result<Value, GatewayCommandError> {
        dispatch_public_settings_command(&self.owner, method, args, |_, _| {})
    }

    fn bytes(&self) -> Vec<u8> { fs::read(self.owner.get_settings_path()).unwrap() }

    fn entries(&self) -> Vec<std::ffi::OsString> {
        let mut entries = fs::read_dir(&self.root)
            .expect("read private installation directory")
            .map(|entry| entry.expect("read installation entry").file_name())
            .collect::<Vec<_>>();
        entries.sort();
        entries
    }
}

impl Drop for Fixture {
    fn drop(&mut self) { let _ = fs::remove_dir_all(&self.root); }
}

fn assert_public(value: &Value, forbidden: &[&str]) {
    let serialized = value.to_string();
    for secret in forbidden {
        assert!(!serialized.contains(secret), "public response contains a protected value");
    }
}

fn settings_view<'a>(method: &str, value: &'a Value) -> &'a Value {
    if method.starts_with("config.") { &value["settings"] } else { value }
}

#[test]
fn masked_skill_round_trips_preserve_stored_bytes_and_explicit_updates_keep_normalization() {
    for method in ["config.set", "setHostSettings"] {
        for marker in ["[REDACTED]", "__OPENCLAW_REDACTED__"] {
            let fixture = Fixture::new();
            let values = ["  synthetic padded value  ", "", " \t\n ", "\nsynthetic\nmultiline\n"];
            let mut stored: Value = serde_json::from_slice(&fixture.bytes()).unwrap();
            for (index, value) in values.iter().enumerate() {
                stored["skills"]["entries"][format!("entry{index}")] = json!({
                    "enabled":true, "apiKey":value,
                    "env":{"RAW":value," PADDED_KEY ":value},
                });
            }
            fs::write(fixture.owner.get_settings_path(), stored.to_string()).unwrap();
            let read = fixture.call("config.get", &json!({})).unwrap();
            let mut masked = read["settings"].clone();
            masked["userTimeZone"] = json!("Asia/Tokyo");
            for index in 0..values.len() {
                let entry = &mut masked["skills"]["entries"][format!("entry{index}")];
                entry["enabled"] = json!(false);
                entry["apiKey"] = json!(marker);
                entry["env"]["RAW"] = json!(marker);
                entry["env"][" PADDED_KEY "] = json!(marker);
            }
            let args = if method == "config.set" {
                json!({"baseHash":read["hash"],"settings":masked})
            } else { masked };
            fixture.call(method, &args).unwrap();
            let reopened = SettingsService::new(fixture.owner.get_settings_path().to_path_buf());
            let current = reopened.get_host_settings_snapshot();
            let durable: Value = serde_json::from_slice(&fixture.bytes()).unwrap();
            for (index, value) in values.iter().enumerate() {
                for document in [&current.settings, &durable] {
                    let entry = &document["skills"]["entries"][format!("entry{index}")];
                    assert_eq!(entry["enabled"], false);
                    assert!(entry["apiKey"].as_str() == Some(*value), "masked API key changed");
                    assert!(entry["env"]["RAW"].as_str() == Some(*value), "masked environment value changed");
                    assert!(entry["env"][" PADDED_KEY "].as_str() == Some(*value), "masked original key/value changed");
                    assert!(entry["env"].get("PADDED_KEY").is_none(), "masked key must not be renamed");
                }
            }
            assert_eq!(current.settings["userTimeZone"], "Asia/Tokyo");

            // Explicit new input retains the established trim/removal behavior.
            let update = json!({"skills":{"entries":{"entry0":{
                "apiKey":"", "env":{"RAW":"", "EXPLICIT":"  synthetic replacement  "}
            }}}});
            let args = if method == "config.set" {
                let read = fixture.call("config.get", &json!({})).unwrap();
                json!({"baseHash":read["hash"],"settings":update})
            } else { update };
            fixture.call(method, &args).unwrap();
            let durable: Value = serde_json::from_slice(&fixture.bytes()).unwrap();
            let entry = &durable["skills"]["entries"]["entry0"];
            assert!(entry.get("apiKey").is_none());
            assert!(entry["env"].get("RAW").is_none());
            assert!(entry["env"]["EXPLICIT"].as_str() == Some("synthetic replacement"));
            assert!(entry["env"][" PADDED_KEY "].as_str() == Some(values[0]));
        }
    }
}

#[test]
fn all_public_settings_routes_mask_values_and_internal_digests() {
    let fixture = Fixture::new();
    let trusted = fixture.owner.get_host_settings_snapshot();
    let raw_hash = trusted.config_source["rawHash"].as_str().unwrap();
    for method in ["config.get", "getHostSettings"] {
        let reply = fixture.call(method, &json!({})).unwrap();
        assert_public(&reply, &[AGENT_SECRET, SKILL_SECRET, ENV_SECRET, &trusted.hash, raw_hash]);
        let view = settings_view(method, &reply);
        assert_eq!(view["inferenceProvider"], "fabushi");
        assert_eq!(view["agents"]["entries"]["primary"]["name"], "Primary");
        assert_eq!(view["agents"]["entries"]["primary"]["maxTokens"], 4096);
        assert_eq!(view["agents"]["entries"]["primary"]["apiKey"], "[REDACTED]");
        assert_eq!(view["skills"]["entries"]["demo"]["env"]["REGION"], "[REDACTED]");
        if method == "config.get" {
            assert_eq!(reply["config"], reply["settings"]);
            let token = reply["hash"].as_str().unwrap();
            assert!(token.starts_with("hmac-sha256:v1:"));
            assert!(reply["configSource"]["rawHash"].as_str().unwrap().starts_with("hmac-sha256:v1:"));
            assert_eq!(fixture.call(method, &json!({"baseHash":token})).unwrap()["matches"], true);
            assert_eq!(fixture.call(method, &json!({"baseHash":trusted.hash})).unwrap()["matches"], false);
        }
    }

    for (method, shape, zone) in [
        ("config.set", "config", "Asia/Tokyo"),
        ("config.set", "settings", "Europe/London"),
        ("config.set", "raw", "America/New_York"),
        ("setHostSettings", "legacy", "UTC"),
    ] {
        let read = fixture.call("config.get", &json!({})).unwrap();
        let mut update = read["config"].clone();
        update["userTimeZone"] = json!(zone);
        let args = if shape == "legacy" { update.clone() } else {
            let mut request = json!({"baseHash":read["hash"]});
            request[shape] = if shape == "raw" { json!(update.to_string()) } else { update.clone() };
            request
        };
        let mut callbacks = 0;
        let reply = dispatch_public_settings_command(&fixture.owner, method, &args, |captured, permission| {
            callbacks += 1;
            assert!(captured["agents"]["entries"]["primary"]["apiKey"].as_str() == Some(AGENT_SECRET));
            assert!(captured["skills"]["entries"]["demo"]["apiKey"].as_str() == Some(SKILL_SECRET));
            assert_eq!(captured["userTimeZone"], zone);
            assert!(permission, "full public document retains the original permission-notification shape");
        }).unwrap();
        assert_eq!(callbacks, 1);
        let current = fixture.owner.get_host_settings_snapshot();
        assert_public(&reply, &[AGENT_SECRET, SKILL_SECRET, ENV_SECRET, &current.hash, current.config_source["rawHash"].as_str().unwrap()]);
        assert_eq!(settings_view(method, &reply)["userTimeZone"], zone);
        if method == "config.set" {
            assert_eq!(reply["config"], reply["settings"]);
            assert_eq!(reply["ok"], true);
        }
        assert!(current.settings["agents"]["entries"]["primary"]["apiKey"].as_str() == Some(AGENT_SECRET));
        assert!(current.settings["skills"]["entries"]["demo"]["apiKey"].as_str() == Some(SKILL_SECRET));
        assert!(current.settings["skills"]["entries"]["demo"]["env"]["REGION"].as_str() == Some(ENV_SECRET));
    }
}

#[test]
fn public_settings_conflicts_and_invalid_requests_do_not_expose_values_or_call_consumers() {
    let fixture = Fixture::new();
    let trusted = fixture.owner.get_host_settings_snapshot();
    let current = fixture.call("config.get", &json!({})).unwrap();
    let original = fixture.bytes();
    let notifications = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&notifications);
    let _subscription = fixture.owner.subscribe_to_changes(Arc::new(move |_| { sink.fetch_add(1, Ordering::SeqCst); }));
    let cases = [
        (json!({"baseHash":trusted.hash, "config":{"skills":{"entries":{"demo":{"apiKey":"synthetic-rejected-new-secret"}}}}}), 409),
        (json!({"baseHash":"synthetic-untrusted-base-hash", "config":{"userTimeZone":"Asia/Tokyo"}}), 409),
        (json!({"baseHash":current["hash"], "config":{"userTimeZone":"[REDACTED]"}}), 500),
        (json!({"raw":"{synthetic-invalid-json-secret"}), 400),
        (json!({"config":null}), 400),
        (json!({}), 400),
    ];
    for (args, status) in cases {
        let result = dispatch_public_settings_command(&fixture.owner, "config.set", &args, |_, _| {
            panic!("rejected request must not call trusted commit consumers")
        });
        let error = result.expect_err("request should be refused");
        assert_eq!(error.status(), status);
        let message = error.to_string();
        for protected in [AGENT_SECRET, SKILL_SECRET, ENV_SECRET, trusted.hash.as_str(), "synthetic-untrusted-base-hash", "synthetic-invalid-json-secret", "synthetic-rejected-new-secret"] {
            assert!(!message.contains(protected), "public error contains a protected value");
        }
        assert!(fixture.bytes() == original, "refusal must preserve settings bytes");
        assert_eq!(notifications.load(Ordering::SeqCst), 0);
    }
}

#[test]
fn public_config_set_rejects_malformed_base_hash_before_owner_effects() {
    let malformed_values = [
        Value::Null,
        json!(true),
        json!(false),
        json!(0),
        json!(1.5),
        json!([]),
        json!(["synthetic-untrusted-revision"]),
        json!({}),
        json!({"hash":"synthetic-untrusted-revision"}),
        json!(""),
    ];
    for shape in ["raw", "config", "settings"] {
        let fixture = Fixture::new();
        let mut stored: Value = serde_json::from_slice(&fixture.bytes()).unwrap();
        stored["notifications"] = json!({"isEnabled":true,"legacy":"preserve-before-validation"});
        fs::write(fixture.owner.get_settings_path(), stored.to_string()).unwrap();
        // Do not call a Settings getter: it could normalize these bytes or create a key.
        let original = fixture.bytes();
        let entries = fixture.entries();
        let key_path = fixture.root.join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
        assert!(!key_path.exists());
        assert_eq!(entries, vec![std::ffi::OsString::from("settings.json")]);

        let notifications = Arc::new(AtomicUsize::new(0));
        let sink = Arc::clone(&notifications);
        let _changes = fixture.owner.subscribe_to_changes(Arc::new(move |_| {
            sink.fetch_add(1, Ordering::SeqCst);
        }));
        let sink = Arc::clone(&notifications);
        let _time_zone = fixture.owner.subscribe_to_user_time_zone(Arc::new(move |_| {
            sink.fetch_add(1, Ordering::SeqCst);
        }));
        let sink = Arc::clone(&notifications);
        let _features = fixture.owner.subscribe_to_feature_flag_overrides(Arc::new(move |_| {
            sink.fetch_add(1, Ordering::SeqCst);
        }));
        let update = json!({
            "userTimeZone":"Asia/Tokyo",
            "localToolPermission":"allow",
            "featureFlagOverrides":{"syntheticRevisionInputOracle":true},
        });
        for malformed in &malformed_values {
            let mut args = json!({"baseHash":malformed});
            args[shape] = if shape == "raw" { json!(update.to_string()) } else { update.clone() };
            let error = dispatch_public_settings_command(&fixture.owner, "config.set", &args, |_, _| {
                panic!("invalid revision input must not reach trusted commit consumers")
            }).expect_err("present malformed revision must be refused");
            assert!(matches!(&error, GatewayCommandError::BadRequest(_)));
            assert_eq!(error.status(), 400);
            assert_eq!(error.to_string(), "config.set baseHash must be a non-empty string");
            assert!(fixture.bytes() == original, "refusal must preserve unnormalized settings bytes");
            assert_eq!(notifications.load(Ordering::SeqCst), 0);
            assert!(!key_path.exists(), "invalid input must not create revision-key material");
            assert_eq!(fixture.entries(), entries, "invalid input must leave no key or staging files");
        }
    }
}

#[test]
fn public_config_set_revision_input_preserves_newer_commit_and_legacy_controls() {
    let fixture = Fixture::new();
    let old_model = json!({
        "provider":"fabushi", "modelId":"Model/A", "maxMode":true, "parameters":[],
    });
    let newer_model = json!({
        "provider":"fabushi", "modelId":"Model/B", "maxMode":true, "parameters":[],
    });
    fixture.owner.set_host_settings(&json!({"agentDefaultModel":old_model})).unwrap();
    let old_read = fixture.call("config.get", &json!({})).unwrap();
    let old_form = old_read["config"].clone();
    let old_hash = old_read["hash"].as_str().unwrap().to_string();
    assert_eq!(old_form["agentDefaultModel"], old_model);

    fixture.owner.set_host_settings(&json!({"agentDefaultModel":newer_model})).unwrap();
    let current_read = fixture.call("config.get", &json!({})).unwrap();
    let current_hash = current_read["hash"].as_str().unwrap().to_string();
    assert_ne!(old_hash, current_hash);
    assert_eq!(current_read["settings"]["agentDefaultModel"], newer_model);
    let committed_bytes = fixture.bytes();
    let key_path = fixture.root.join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
    let key_bytes = fs::read(&key_path).unwrap();
    let entries = fixture.entries();
    let notifications = Arc::new(AtomicUsize::new(0));
    let sink = Arc::clone(&notifications);
    let _subscription = fixture.owner.subscribe_to_changes(Arc::new(move |_| {
        sink.fetch_add(1, Ordering::SeqCst);
    }));

    for malformed in [Value::Null, json!({"hash":old_hash}), json!("")] {
        let error = dispatch_public_settings_command(&fixture.owner, "config.set", &json!({
            "baseHash":malformed, "config":old_form,
        }), |_, _| {
            panic!("malformed stale form must not call committed consumers")
        }).expect_err("malformed old precondition must not overwrite a newer commit");
        assert_eq!(error.status(), 400);
        assert_eq!(error.to_string(), "config.set baseHash must be a non-empty string");
        assert!(fixture.bytes() == committed_bytes, "newer model bytes must survive malformed input");
        assert!(fs::read(&key_path).unwrap() == key_bytes, "revision key must remain unchanged");
        assert_eq!(fixture.entries(), entries);
        assert_eq!(notifications.load(Ordering::SeqCst), 0);
    }

    // Nonempty strings still reach the existing opaque CAS without trimming.
    for token in [old_hash.clone(), format!(" \t{current_hash}\n"), " \t\n ".to_string()] {
        let error = dispatch_public_settings_command(&fixture.owner, "config.set", &json!({
            "baseHash":token, "settings":old_form,
        }), |_, _| {
            panic!("a conflicting revision must not call committed consumers")
        }).expect_err("stale or whitespace-modified token must conflict");
        assert_eq!(error.status(), 409);
        assert_eq!(error.to_string(), "config baseHash mismatch");
        assert!(fixture.bytes() == committed_bytes, "conflict must preserve the newer commit");
        assert!(fs::read(&key_path).unwrap() == key_bytes, "conflict must preserve the key");
        assert_eq!(fixture.entries(), entries);
        assert_eq!(notifications.load(Ordering::SeqCst), 0);
    }

    let reopened = SettingsService::new(fixture.owner.get_settings_path().to_path_buf());
    assert_eq!(reopened.get_agent_default_model_checked().unwrap(), Some(newer_model.clone()));
    assert!(fixture.bytes() == committed_bytes, "reopening must read the retained commit");
    assert!(fs::read(&key_path).unwrap() == key_bytes);

    assert_eq!(fixture.call("config.get", &json!({"baseHash":current_hash})).unwrap()["matches"], true);
    assert_eq!(fixture.call("config.get", &json!({"baseHash":old_hash})).unwrap()["matches"], false);
    assert_eq!(fixture.call("config.get", &json!({"baseHash":""})).unwrap()["matches"], false);
    let unconditioned_read = fixture.call("config.get", &json!({"baseHash":null})).unwrap();
    assert!(unconditioned_read.get("matches").is_none(), "read arguments retain their existing semantics");
    assert_eq!(unconditioned_read["hash"], current_hash);
    assert_eq!(unconditioned_read["settings"]["agentDefaultModel"], newer_model);
    assert!(fixture.bytes() == committed_bytes);
    assert_eq!(notifications.load(Ordering::SeqCst), 0);

    let mut callbacks = 0;
    for (index, (method, args, zone)) in [
        ("config.set", json!({
            "baseHash":current_hash, "settings":{"userTimeZone":"Asia/Tokyo"},
        }), "Asia/Tokyo"),
        ("config.set", json!({
            "raw":json!({"userTimeZone":"Europe/London"}).to_string(),
        }), "Europe/London"),
        ("setHostSettings", json!({
            "baseHash":null, "userTimeZone":"America/New_York",
        }), "America/New_York"),
    ].into_iter().enumerate() {
        let reply = dispatch_public_settings_command(&fixture.owner, method, &args, |trusted, permission| {
            callbacks += 1;
            assert!(!permission);
            assert_eq!(trusted["agentDefaultModel"], newer_model);
            assert_eq!(trusted["userTimeZone"], zone);
        }).expect("current, omitted and legacy revisions retain accepted writes");
        let view = settings_view(method, &reply);
        assert_eq!(view["agentDefaultModel"], newer_model);
        assert_eq!(view["userTimeZone"], zone);
        assert_public(&reply, &[AGENT_SECRET, SKILL_SECRET, ENV_SECRET]);
        if method == "config.set" {
            assert_eq!(reply["ok"], true);
        } else {
            assert!(reply.get("ok").is_none() && reply.get("hash").is_none());
        }
        let durable: Value = serde_json::from_slice(&fixture.bytes()).unwrap();
        assert_eq!(durable["agentDefaultModel"], newer_model);
        assert_eq!(durable["userTimeZone"], zone);
        assert_eq!(callbacks, index + 1);
        assert_eq!(notifications.load(Ordering::SeqCst), index + 1);
        assert!(fs::read(&key_path).unwrap() == key_bytes, "accepted writes retain the installation key");
        assert_eq!(fixture.entries(), entries);
    }
    let final_read = fixture.call("config.get", &json!({})).unwrap();
    assert_eq!(fixture.call("getHostSettings", &json!({})).unwrap(), final_read["settings"]);
    assert_eq!(fixture.call("config.get", &json!({"baseHash":final_read["hash"]})).unwrap()["matches"], true);
}

#[test]
fn public_settings_response_and_trusted_callback_keep_their_commit_after_reentry() {
    let fixture = Fixture::new();
    let owner = Arc::downgrade(&fixture.owner);
    let changed = Arc::new(AtomicBool::new(false));
    let once = Arc::clone(&changed);
    let _subscription = fixture.owner.subscribe_to_changes(Arc::new(move |_| {
        if !once.swap(true, Ordering::SeqCst) {
            owner.upgrade().unwrap().set_host_settings(&json!({
                "userTimeZone":"Europe/London",
                "skills":{"entries":{"demo":{"apiKey":"synthetic-reentrant-new-secret"}}},
            })).unwrap();
        }
    }));
    let captured = Mutex::new(None);
    let reply = dispatch_public_settings_command(&fixture.owner, "config.set", &json!({
        "config":{"userTimeZone":"Asia/Tokyo"},
    }), |trusted, permission| {
        assert!(!permission);
        assert_eq!(trusted["userTimeZone"], "Asia/Tokyo");
        assert!(trusted["skills"]["entries"]["demo"]["apiKey"].as_str() == Some(SKILL_SECRET));
        *captured.lock().unwrap() = Some(trusted.clone());
    }).unwrap();
    assert!(changed.load(Ordering::SeqCst));
    assert!(captured.lock().unwrap().is_some());
    assert_eq!(reply["settings"]["userTimeZone"], "Asia/Tokyo");
    assert_public(&reply, &[SKILL_SECRET, "synthetic-reentrant-new-secret"]);
    let latest = fixture.call("config.get", &json!({})).unwrap();
    assert_eq!(latest["settings"]["userTimeZone"], "Europe/London");
    assert_ne!(reply["hash"], latest["hash"]);
    assert_ne!(reply["configSource"]["rawHash"], latest["configSource"]["rawHash"]);
    let trusted = fixture.owner.get_host_settings();
    assert!(trusted["skills"]["entries"]["demo"]["apiKey"].as_str() == Some("synthetic-reentrant-new-secret"));
}

#[test]
fn legacy_public_masked_write_restores_the_current_committed_secret() {
    let fixture = Fixture::new();
    let mut old_form = fixture.call("getHostSettings", &json!({})).unwrap();
    fixture.owner.set_host_settings(&json!({"skills":{"entries":{"demo":{"apiKey":"synthetic-new-current-secret"}}}})).unwrap();
    old_form["userTimeZone"] = json!("Asia/Tokyo");
    let reply = dispatch_public_settings_command(&fixture.owner, "setHostSettings", &old_form, |trusted, _| {
        assert!(trusted["skills"]["entries"]["demo"]["apiKey"].as_str() == Some("synthetic-new-current-secret"));
    }).unwrap();
    assert_eq!(reply["userTimeZone"], "Asia/Tokyo");
    assert_public(&reply, &[SKILL_SECRET, "synthetic-new-current-secret"]);
    let trusted = fixture.owner.get_host_settings();
    assert!(trusted["skills"]["entries"]["demo"]["apiKey"].as_str() == Some("synthetic-new-current-secret"));
}

#[test]
fn public_key_refusal_precedes_all_four_routes_and_read_normalization() {
    let fixture = Fixture::new();
    let mut raw: Value = serde_json::from_slice(&fixture.bytes()).unwrap();
    raw["notifications"] = json!({"isEnabled":true,"legacy":"must remain unchanged"});
    fs::write(fixture.owner.get_settings_path(), raw.to_string()).unwrap();
    let key_path = fixture.root.join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
    fs::write(&key_path, b"invalid-synthetic-key").unwrap();
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&key_path, fs::Permissions::from_mode(0o600)).unwrap();
    }
    let original = fixture.bytes();
    for method in ["config.get", "config.set", "getHostSettings", "setHostSettings"] {
        let args = if method == "config.set" { json!({"config":{"userTimeZone":"Asia/Tokyo"}}) }
            else { json!({"userTimeZone":"Asia/Tokyo"}) };
        let error = dispatch_public_settings_command(&fixture.owner, method, &args, |_, _| {
            panic!("key refusal cannot invoke committed consumers")
        }).expect_err("corrupt key must refuse public operation");
        assert_eq!(error.status(), 500);
        assert_eq!(error.to_string(), "configuration revision key is unavailable");
        assert!(fixture.bytes() == original, "refusal must preserve settings bytes");
        assert!(fs::read(&key_path).unwrap().as_slice() == b"invalid-synthetic-key");
    }
}

// DFA-v1: actual owner + TCP ingress + shipping Settings helper. The separate
// source-binding test below establishes wiring, not full UnifiedGatewayApi startup.
const DFA_NATIVE_SECRET: &str = "dfa-private-native-secret";
const DFA_WAIT: Duration = Duration::from_secs(15);
type DfaRow = BTreeMap<String, SqlValue>;

#[derive(PartialEq)]
struct DfaPairingState { paired: Vec<DfaRow>, pending: Vec<DfaRow>, machine: Vec<DfaRow> }

fn dfa_rows(conn: &Connection, sql: &str) -> Vec<DfaRow> {
    let mut statement = conn.prepare(sql).unwrap();
    let names: Vec<_> = statement.column_names().into_iter().map(str::to_owned).collect();
    statement.query_map([], |row| {
        names.iter().enumerate().map(|(index, name)| {
            Ok((name.clone(), row.get::<_, SqlValue>(index)?))
        }).collect::<rusqlite::Result<DfaRow>>()
    }).unwrap().collect::<rusqlite::Result<Vec<_>>>().unwrap()
}

#[derive(Default)]
struct DfaCounts {
    lookups: AtomicUsize, accepted: AtomicUsize, bound: AtomicUsize,
    dispatches: AtomicUsize, commits: AtomicUsize, completed: AtomicUsize, errors: AtomicUsize,
}

impl DfaCounts {
    fn read(&self) -> [usize; 7] {
        [&self.lookups, &self.accepted, &self.bound, &self.dispatches, &self.commits, &self.completed, &self.errors]
            .map(|counter| counter.load(Ordering::SeqCst))
    }
}

struct DfaGatewayApi {
    owner: Option<DevicePairingOwner>,
    settings: Arc<SettingsService>,
    counts: Arc<DfaCounts>,
    closed: SyncSender<()>,
}

impl GatewayApi for DfaGatewayApi {
    fn authenticate_device_token(&self, token: &str) -> Result<Option<DeviceTokenAuthorization>, GatewayCommandError> {
        self.counts.lookups.fetch_add(1, Ordering::SeqCst);
        let result = self.owner.as_ref().unwrap().authenticate_device_token(token)?;
        if result.is_some() { self.counts.accepted.fetch_add(1, Ordering::SeqCst); }
        Ok(result)
    }

    fn authorize_device_token_request(&self, _authorization: &DeviceTokenAuthorization, _method: &str, _args: &Value) -> Result<(), GatewayCommandError> {
        self.counts.bound.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }

    fn call(&self, method: &str, args: Value) -> Result<Value, GatewayCommandError> {
        self.counts.dispatches.fetch_add(1, Ordering::SeqCst);
        dispatch_public_settings_command(&self.settings, method, &args, |_, _| {
            self.counts.commits.fetch_add(1, Ordering::SeqCst);
        })
    }

    fn on_command_complete(&self, _report: GatewayCommandReport) {
        self.counts.completed.fetch_add(1, Ordering::SeqCst);
    }

    fn on_command_error(&self, _report: GatewayCommandReport) {
        self.counts.errors.fetch_add(1, Ordering::SeqCst);
    }
}

impl Drop for DfaGatewayApi {
    fn drop(&mut self) { drop(self.owner.take()); let _ = self.closed.try_send(()); }
}

struct DfaHttpFixture {
    settings: Option<Fixture>,
    api: Option<Arc<DfaGatewayApi>>,
    server: Option<GatewayServer>,
    closed: Option<Receiver<()>>,
    counts: Arc<DfaCounts>,
    configured: bool,
    shutdown_failed: bool,
}

impl DfaHttpFixture {
    fn new(configured: bool) -> Self {
        let settings = Fixture::new();
        settings.call("config.get", &json!({})).unwrap();
        let mut fixture = Self { settings: Some(settings), api: None, server: None, closed: None,
            counts: Arc::new(DfaCounts::default()), configured, shutdown_failed: false };
        fixture.open();
        for (device, roles, scopes) in [
            ("dfa-writer", vec!["operator", "node"], vec!["operator.write", "node.invoke"]),
            ("dfa-reader", vec!["operator"], vec!["operator.read"]),
            ("dfa-other", vec!["operator"], vec!["operator.write"]),
        ] {
            let request = fixture.owner().handle_pair_request(&json!({
                "deviceId":device, "publicKey":format!("{device}-public-key"),
                "role":roles[0], "roles":roles, "scopes":scopes,
            })).unwrap();
            fixture.owner().handle_pair_approve(&json!({"requestId":request["requestId"]})).unwrap();
        }
        fixture.owner().handle_pair_request(&json!({
            "deviceId":"dfa-pending", "publicKey":"pending-public-key", "role":"operator", "scopes":["operator.read"],
        })).unwrap();
        let conn = fixture.writer();
        conn.execute("UPDATE device_pairing_paired SET node_surface_json = ?1, pending_node_surface_json = ?2 WHERE device_id = 'dfa-writer'",
            params!["{\"generation\":7}", "{\"generation\":8}"]).unwrap();
        conn.close().unwrap();
        fixture
    }

    fn settings(&self) -> &Fixture { self.settings.as_ref().unwrap() }
    fn db(&self) -> PathBuf { self.settings().root.join("dfa-pairing.db") }
    fn owner(&self) -> &DevicePairingOwner { self.api.as_ref().unwrap().owner.as_ref().unwrap() }
    fn writer(&self) -> Connection { Connection::open_with_flags(self.db(), OpenFlags::SQLITE_OPEN_READ_WRITE).unwrap() }

    fn open(&mut self) {
        let (closed, receiver) = mpsc::sync_channel(1);
        let api = Arc::new(DfaGatewayApi {
            owner: Some(DevicePairingOwner::new(self.db()).unwrap()), settings: Arc::clone(&self.settings().owner),
            counts: Arc::clone(&self.counts), closed,
        });
        self.server = Some(start_gateway_server(GatewayServerDeps {
            api: api.clone(), events: GatewayEventHub::default(), local_exec: None, webauthn: None,
            config: GatewayServerConfig { host:"127.0.0.1".into(), port:None,
                auth_token:self.configured.then(|| DFA_NATIVE_SECRET.into()), tls:None }, started_at:1,
        }).unwrap());
        self.api = Some(api);
        self.closed = Some(receiver);
    }

    fn stop(&mut self) -> bool {
        if let Some(server) = self.server.take() { server.close(); }
        drop(self.api.take());
        if let Some(receiver) = self.closed.take() {
            self.shutdown_failed |= receiver.recv_timeout(DFA_WAIT).is_err();
        }
        !self.shutdown_failed
    }

    fn reopen(&mut self) {
        assert!(self.stop(), "all HTTP handlers and their pairing connection close before reopen");
        self.open();
    }

    fn observe(&self) -> DfaPairingState {
        let conn = Connection::open_with_flags(self.db(), OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
        conn.execute_batch("BEGIN DEFERRED").unwrap();
        let state = DfaPairingState {
            paired: dfa_rows(&conn, "SELECT * FROM device_pairing_paired ORDER BY device_id"),
            pending: dfa_rows(&conn, "SELECT * FROM device_pairing_pending ORDER BY request_id"),
            machine: dfa_rows(&conn, "SELECT *, typeof(value_json) AS value_type, typeof(updated_at_ms) AS timestamp_type FROM config_machine_state ORDER BY state_key"),
        };
        conn.execute_batch("COMMIT").unwrap();
        conn.close().unwrap();
        state
    }

    fn token(&self, device: &str, role: &str) -> String {
        let conn = self.writer();
        let raw: String = conn.query_row("SELECT tokens_json FROM device_pairing_paired WHERE device_id = ?1", [device], |row| row.get(0)).unwrap();
        conn.close().unwrap();
        serde_json::from_str::<Value>(&raw).unwrap()[role]["token"].as_str().unwrap().to_owned()
    }

    fn post(&self, bearer: Option<&str>, header: &str, zone: &str) -> (u16, Value, String) {
        let address = SocketAddr::from(([127,0,0,1], self.server.as_ref().unwrap().port()));
        let mut stream = TcpStream::connect_timeout(&address, DFA_WAIT).unwrap();
        stream.set_write_timeout(Some(DFA_WAIT)).unwrap();
        let auth = bearer.map(|token| format!("Authorization: Bearer {token}\r\n")).unwrap_or_default();
        let body = json!({"config":{"userTimeZone":zone}}).to_string();
        write!(stream, "POST /api/config.set HTTP/1.1\r\nHost: 127.0.0.1\r\n{auth}{header}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}", body.len()).unwrap();
        stream.flush().unwrap();
        let deadline = Instant::now() + DFA_WAIT;
        let mut bytes = Vec::new();
        loop {
            let remaining = deadline.checked_duration_since(Instant::now()).filter(|remaining| !remaining.is_zero()).expect("bounded HTTP response");
            stream.set_read_timeout(Some(remaining)).unwrap();
            let mut buffer = [0_u8; 4096];
            let count = stream.read(&mut buffer).unwrap();
            if count == 0 { break; }
            bytes.extend_from_slice(&buffer[..count]);
            assert!(bytes.len() <= 1024 * 1024, "bounded response size");
        }
        let raw = String::from_utf8(bytes).unwrap();
        let status = raw.lines().next().unwrap().split_whitespace().nth(1).unwrap().parse().unwrap();
        let body = serde_json::from_str(raw.split_once("\r\n\r\n").unwrap().1).unwrap();
        (status, body, raw)
    }

    fn refused(&self, bearer: &str, status: u16, admitted: bool) {
        let state = self.observe();
        let bytes = self.settings().bytes();
        let entries = self.settings().entries();
        let mut expected = self.counts.read();
        expected[0] += 1;
        expected[1] += usize::from(admitted);
        let (actual, body, raw) = self.post(Some(bearer), "X-Sand-Scope: operator.admin\r\n", "Asia/Tokyo");
        assert_eq!(actual, status, "fresh HTTP refusal status, response omitted");
        assert!(body["error"].as_str().is_some_and(|error| !error.is_empty()));
        if status == 500 { assert_eq!(body["error"], "stored pairing authority is invalid"); }
        for secret in [bearer, DFA_NATIVE_SECRET, AGENT_SECRET, SKILL_SECRET, ENV_SECRET] {
            assert!(!raw.contains(secret), "HTTP response discloses private material");
        }
        assert!(!raw.contains("dfa-private-malformed"));
        assert_eq!(self.counts.read(), expected, "ingress refusal cannot dispatch, commit or report a command");
        assert!(self.observe() == state, "HTTP refusal preserves every pairing/pending/revision row");
        assert!(self.settings().bytes() == bytes, "HTTP refusal preserves actual Settings bytes");
        assert_eq!(self.settings().entries(), entries, "HTTP refusal creates no Settings sidecar");
    }

    fn committed(&self, bearer: Option<&str>, paired: bool, zone: &str) {
        let state = self.observe();
        let bytes = self.settings().bytes();
        let mut expected = self.counts.read();
        if paired { expected[0] += 1; expected[1] += 1; expected[2] += 1; }
        expected[3] += 1; expected[4] += 1; expected[5] += 1;
        let (status, body, raw) = self.post(bearer, "", zone);
        assert_eq!(status, 200, "healthy request reaches actual Settings consumer");
        assert_eq!(body["settings"]["userTimeZone"], zone);
        assert_public(&body, &[AGENT_SECRET, SKILL_SECRET, ENV_SECRET, DFA_NATIVE_SECRET]);
        if let Some(token) = bearer { assert!(!raw.contains(token)); }
        assert_eq!(self.counts.read(), expected);
        assert!(self.observe() == state, "positive authentication does not record last-use or alter pairing");
        assert!(self.settings().bytes() != bytes, "positive writes the real Settings file");
        let reopened = SettingsService::new(self.settings().owner.get_settings_path().to_path_buf());
        assert_eq!(reopened.get_host_settings()["userTimeZone"], zone, "commit survives a new Settings owner");
    }
}

impl Drop for DfaHttpFixture {
    fn drop(&mut self) {
        if !self.stop() {
            // Never remove a fixture whose connection teardown was not acknowledged.
            std::mem::forget(self.settings.take());
            assert!(std::thread::panicking(), "bounded HTTP adapter teardown failed");
        }
    }
}

#[test]
fn fresh_bearer_http_config_mutation_requires_current_approval() {
    for configured in [false, true] {
        let mut fixture = DfaHttpFixture::new(configured);
        let bearer = fixture.token("dfa-writer", "operator");
        let node = fixture.token("dfa-writer", "node");
        let reader = fixture.token("dfa-reader", "operator");
        let other = fixture.token("dfa-other", "operator");
        let initial = fixture.observe();
        let row = initial.paired.iter().find(|row| row.get("device_id") == Some(&SqlValue::Text("dfa-writer".into()))).unwrap();
        let original_tokens = match row.get("tokens_json").unwrap() { SqlValue::Text(raw) => raw.clone(), _ => panic!("fixture tokens text") };
        for corruption in ["overgrant", "partial-overgrant", "role", "malformed"] {
            let conn = fixture.writer();
            conn.execute("UPDATE device_pairing_paired SET roles_json = ?1, approved_scopes_json = ?2, tokens_json = ?3 WHERE device_id = 'dfa-writer'",
                params![row.get("roles_json").unwrap(), row.get("approved_scopes_json").unwrap(), &original_tokens]).unwrap();
            match corruption {
                "overgrant" => { conn.execute("UPDATE device_pairing_paired SET approved_scopes_json = '[\"operator.read\",\"node.invoke\"]' WHERE device_id = 'dfa-writer'", []).unwrap(); }
                "partial-overgrant" => {
                    let mut tokens: Value = serde_json::from_str(&original_tokens).unwrap();
                    tokens["operator"]["scopes"] = json!(["operator.write", "operator.approvals"]);
                    conn.execute("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = 'dfa-writer'", [tokens.to_string()]).unwrap();
                }
                "role" => { conn.execute("UPDATE device_pairing_paired SET roles_json = '[\"node\"]' WHERE device_id = 'dfa-writer'", []).unwrap(); }
                "malformed" => { conn.execute("UPDATE device_pairing_paired SET approved_scopes_json = 'dfa-private-malformed' WHERE device_id = 'dfa-writer'", []).unwrap(); }
                _ => unreachable!(),
            }
            conn.close().unwrap();
            fixture.refused(&bearer, if corruption == "malformed" { 500 } else { 401 }, false);
            if corruption == "overgrant" {
                let durable = fixture.observe();
                fixture.reopen();
                assert!(fixture.observe() == durable);
                fixture.refused(&bearer, 401, false);
            }
            if corruption != "malformed" {
                let before = fixture.observe();
                assert!(fixture.owner().authenticate_device_token(&node).unwrap().is_some(), "unselected node remains valid");
                assert!(fixture.observe() == before);
            }
        }
        let conn = fixture.writer();
        conn.execute("UPDATE device_pairing_paired SET roles_json = ?1, approved_scopes_json = NULL, tokens_json = ?2 WHERE device_id = 'dfa-writer'",
            params![row.get("roles_json").unwrap(), original_tokens]).unwrap();
        conn.close().unwrap();
        fixture.committed(Some(&bearer), true, "Asia/Tokyo");
        fixture.refused(&reader, 403, true);
        fixture.refused("dfa-unknown-explicit-bearer", 401, false);
        fixture.committed(Some(&other), true, "Europe/London");
        let native = configured.then_some(DFA_NATIVE_SECRET);
        fixture.committed(native, false, "America/New_York");

        // The same ordinary HTTP request must retain complete large-grant
        // admission, including one invalid tail entry, without a timing oracle.
        let mut scopes: Vec<String> = (0..20_000).map(|index| format!("operator.s{index:05}")).collect();
        scopes.push("operator.write".into());
        let current = fixture.observe();
        let writer = current.paired.iter().find(|row| row.get("device_id") == Some(&SqlValue::Text("dfa-writer".into()))).unwrap();
        let raw = match writer.get("tokens_json").unwrap() { SqlValue::Text(raw) => raw, _ => panic!("fixture tokens text") };
        let mut tokens: Value = serde_json::from_str(raw).unwrap();
        tokens["operator"]["scopes"] = json!(scopes);
        let mut approved = scopes.clone();
        approved.push("node.invoke".into());
        let mut roles: Vec<String> = (0..20_000).map(|index| format!("observer{index:05}")).collect();
        roles.extend([" operator ".into(), "operator".into(), "node".into(), "\t".into()]);
        let conn = fixture.writer();
        assert_eq!(conn.execute("UPDATE device_pairing_paired SET approved_scopes_json = ?1, tokens_json = ?2, roles_json = ?3 WHERE device_id = 'dfa-writer'",
            params![json!(approved).to_string(), tokens.to_string(), json!(roles).to_string()]).unwrap(), 1);
        conn.close().unwrap();
        fixture.committed(Some(&bearer), true, "Asia/Tokyo");

        let mut malformed_roles = json!(roles);
        malformed_roles.as_array_mut().unwrap().push(json!(7));
        let conn = fixture.writer();
        assert_eq!(conn.execute("UPDATE device_pairing_paired SET roles_json = ?1 WHERE device_id = 'dfa-writer'", [malformed_roles.to_string()]).unwrap(), 1);
        conn.close().unwrap();
        fixture.refused(&bearer, 500, false);
        let conn = fixture.writer();
        assert_eq!(conn.execute("UPDATE device_pairing_paired SET roles_json = ?1 WHERE device_id = 'dfa-writer'", [json!(roles).to_string()]).unwrap(), 1);
        conn.close().unwrap();

        tokens["operator"]["scopes"].as_array_mut().unwrap().push(json!("operator.zz-unapproved-tail"));
        let conn = fixture.writer();
        assert_eq!(conn.execute("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = 'dfa-writer'", [tokens.to_string()]).unwrap(), 1);
        conn.close().unwrap();
        fixture.refused(&bearer, 401, false);
        fixture.committed(Some(&other), true, "Europe/London");

        tokens["operator"]["scopes"] = json!(scopes);
        let conn = fixture.writer();
        assert_eq!(conn.execute("UPDATE device_pairing_paired SET tokens_json = ?1 WHERE device_id = 'dfa-writer'", [tokens.to_string()]).unwrap(), 1);
        conn.close().unwrap();
        let repaired = fixture.observe();
        fixture.reopen();
        assert!(fixture.observe() == repaired);
        fixture.committed(Some(&bearer), true, "Asia/Tokyo");
    }
}

fn fresh_bearer_shipping_bindings(source: &str) -> bool {
    let body = |start: &str, end: &str| source.split_once(start).and_then(|(_, rest)| rest.split_once(end).map(|(body, _)| body));
    let Some(api) = body("impl GatewayApi for UnifiedGatewayApi {", "    fn rotate_paired_device_token(\n") else { return false; };
    let Some(dispatch) = body("    fn call_openclaw_gateway_command(\n", "    fn call_public_settings_command(\n") else { return false; };
    let Some(settings) = body("    fn call_public_settings_command(\n", "    fn call_openclaw_config_get(\n") else { return false; };
    let Some(config) = body("    fn call_openclaw_config_set(\n", "    fn call_openclaw_system_info(\n") else { return false; };
    let Some(consumer) = body("fn dispatch_public_settings_command(\n", "struct UnifiedGatewayApi {") else { return false; };
    let Some(composition) = body("    let gateway_api = Arc::new(UnifiedGatewayApi {", "    let channel_inbound_gateway =") else { return false; };
    api.contains("self.device_pairing.authenticate_device_token(token)")
        && !api.contains("Ok(Some(") && !api.contains("Ok(None)")
        && dispatch.contains("\"config.set\" => self.call_openclaw_config_set(&args)")
        && settings.contains("dispatch_public_settings_command(&self.settings, method, args,")
        && config.contains("self.call_public_settings_command(\"config.set\", args)")
        && consumer.contains("settings.set_public_host_settings_if_current(base_hash, target.clone())")
        && source.contains("return self.call_openclaw_gateway_command(method, args);")
        && source.contains("let device_pairing = match DevicePairingOwner::new(&device_pairing_db_path)")
        && composition.contains("            device_pairing,\n")
        && composition.contains("\n            settings: Arc::clone(&settings_extension),")
        && source.contains("api: gateway_api.clone(),")
}

#[test]
fn fresh_bearer_shipping_authentication_and_settings_bindings_use_existing_owners() {
    // Read against imported main.rs blob 7ec9bd8a22fd595f63cb3f4b632673e298294dfa.
    // This is mutation-sensitive source evidence, separate from the TCP test.
    let source = include_str!("../main.rs").split_once("\n#[cfg(test)]\nmod tests {").unwrap().0;
    assert!(fresh_bearer_shipping_bindings(source));
    for (from, to) in [
        ("self.device_pairing.authenticate_device_token(token)", "Ok(None)"),
        ("\"config.set\" => self.call_openclaw_config_set(&args)", "\"config.set\" => discarded_config(&args)"),
        ("dispatch_public_settings_command(&self.settings, method, args,", "discarded_settings_command(&self.settings, method, args,"),
        ("self.call_public_settings_command(\"config.set\", args)", "discarded_config_set(args)"),
        ("settings.set_public_host_settings_if_current(base_hash, target.clone())", "discarded_settings_commit()"),
        ("return self.call_openclaw_gateway_command(method, args);", "return discarded_gateway_command(method, args);"),
        ("let device_pairing = match DevicePairingOwner::new(&device_pairing_db_path)", "let device_pairing = match unrelated_owner()"),
        ("            device_pairing,\n", "            device_pairing: unrelated_owner,\n"),
        ("\n            settings: Arc::clone(&settings_extension),", "\n            settings: Arc::clone(&unrelated_settings),"),
        ("api: gateway_api.clone(),", "api: unrelated_api.clone(),"),
    ] {
        assert!(source.contains(from), "negative binding control must edit real shipping source");
        assert!(!fresh_bearer_shipping_bindings(&source.replacen(from, to, 1)), "dead or changed owner binding must fail");
    }
}
