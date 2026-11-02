use super::super::{dispatch_public_settings_command, SettingsService};
use mahayana_host_runtime::durable_file_policy::HOST_CONFIG_REVISION_KEY_FILE_NAME;
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use serde_json::{Value, json};
use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

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
