use std::collections::BTreeMap;
use std::fs;
use std::panic::{AssertUnwindSafe, catch_unwind};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{mpsc, Arc, Barrier, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::host_diagnostics::{HostDiagnostic, pin_host_diagnostics_reporter};
use mahayana_host_runtime::extensions::settings::settings_service::{
    MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH, SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS,
    SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES, SandAutoReviewInstructions,
    HostSettingsUpdateError, SandLocalToolPermission, SettingsService, is_valid_iana_time_zone,
    get_json_value_at_path, set_json_value_at_path,
};
use mahayana_host_runtime::gateway_command_error::GatewayCommandError;
use mahayana_host_runtime::sha256::sha256_hex;

fn temp_path(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-settings-{label}-{}-{suffix}.json",
        std::process::id()
    ))
}

fn malformed_provider_bound_models() -> Vec<(&'static str, serde_json::Value)> {
    let valid = serde_json::json!({
        "provider": "fabushi", "modelId": "Org/Model@Stable", "maxMode": false,
        "parameters": [{"id": "reasoning", "value": "high"}]
    });
    let mut cases = Vec::new();
    for (label, provider) in [
        ("null-provider", serde_json::Value::Null),
        ("boolean-provider", serde_json::json!(true)),
        ("number-provider", serde_json::json!(7)),
        ("array-provider", serde_json::json!(["fabushi"])),
        ("object-provider", serde_json::json!({"id": "fabushi"})),
        ("blank-provider", serde_json::json!("")),
        ("provider-case", serde_json::json!("Fabushi")),
        ("provider-leading-space", serde_json::json!(" fabushi")),
        ("provider-trailing-space", serde_json::json!("fabushi ")),
        ("provider-alias", serde_json::json!("claude")),
        ("provider-underscore-alias", serde_json::json!("claude_code")),
        ("unknown-provider", serde_json::json!("openai")),
    ] {
        let mut value = valid.clone();
        value["provider"] = provider;
        cases.push((label, value));
    }
    for (label, field, replacement) in [
        ("null-model", "modelId", serde_json::Value::Null),
        ("number-model", "modelId", serde_json::json!(7)),
        ("empty-model", "modelId", serde_json::json!("")),
        ("blank-model", "modelId", serde_json::json!(" \u{2003} ")),
        ("newline-model", "modelId", serde_json::json!("\nOrg/Model")),
        ("tab-model", "modelId", serde_json::json!("Org\t/Model")),
        ("nul-model", "modelId", serde_json::json!("Org\0/Model")),
        ("unicode-control-model", "modelId", serde_json::json!("Org\u{0085}/Model")),
        ("invalid-max-mode", "maxMode", serde_json::json!("true")),
        ("null-parameters", "parameters", serde_json::Value::Null),
        ("non-object-parameter", "parameters", serde_json::json!(["high"])),
        ("empty-parameter-id", "parameters", serde_json::json!([{"id": "", "value": "high"}])),
        ("invalid-parameter-value", "parameters", serde_json::json!([{"id": "reasoning", "value": 1}])),
    ] {
        let mut value = valid.clone();
        value[field] = replacement;
        cases.push((label, value));
    }
    for field in ["modelId", "maxMode", "parameters"] {
        let mut value = valid.clone();
        value.as_object_mut().expect("model object").remove(field);
        cases.push((field, value));
    }
    cases
}

#[test]
fn checked_agent_default_model_reads_are_nonmutating_and_preserve_selection_fields() {
    let path = temp_path("checked-model-projection");
    let service = SettingsService::new(&path);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));
    assert_eq!(service.get_agent_default_model_checked().expect("missing source"), None);
    assert!(!path.exists(), "the checked reader cannot create missing settings");

    for default in [None, Some(serde_json::Value::Null)] {
        let mut source = serde_json::json!({
            "version": 1, "notifications": {"isEnabled": true, "legacy": "keep"}
        });
        if let Some(default) = default {
            source["agentDefaultModel"] = default;
        }
        let original = serde_json::to_vec(&source).expect("absent model source");
        fs::write(&path, &original).expect("seed absent model");
        assert_eq!(service.get_agent_default_model_checked().expect("absent default"), None);
        assert_eq!(fs::read(&path).expect("unchanged source"), original);
    }

    for provider in [None, Some("fabushi")] {
        let mut model = serde_json::json!({
            "modelId": " Org/Nested/Model@Stable ", "maxMode": false,
            "parameters": [{"id": "reasoning", "value": "high", "extension": {"keep": true}}],
            "extension": {"revision": 17}
        });
        if let Some(provider) = provider {
            model["provider"] = serde_json::json!(provider);
        }
        let original = serde_json::to_vec(&serde_json::json!({
            "version": 1, "agentDefaultModel": model,
            "notifications": {"isEnabled": true, "legacy": "keep"}
        })).expect("valid model source");
        fs::write(&path, &original).expect("seed valid model");
        let mut expected = model.clone();
        expected["maxMode"] = serde_json::json!(true);
        assert_eq!(service.get_agent_default_model_checked().expect("valid default"), Some(expected.clone()));
        assert_eq!(service.get_agent_default_model(), Some(expected.clone()));
        assert_eq!(SettingsService::new(&path).get_agent_default_model_checked().expect("reopened default"), Some(expected));
        assert_eq!(fs::read(&path).expect("unnormalized source"), original);
    }
    assert!(observed.lock().expect("observed changes").is_empty());
    let _ = fs::remove_file(path);
}

#[test]
fn checked_agent_default_model_rejects_malformed_present_bound_and_legacy_values() {
    let path = temp_path("checked-malformed-model");
    let service = SettingsService::new(&path);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));
    let mut cases = malformed_provider_bound_models();
    cases.extend([
        ("legacy-string", serde_json::json!("model")),
        ("legacy-array", serde_json::json!([])),
        ("legacy-empty-object", serde_json::json!({})),
        ("legacy-blank-model", serde_json::json!({"modelId": "   ", "maxMode": true, "parameters": []})),
        ("legacy-control-model", serde_json::json!({"modelId": "Model\n", "maxMode": true, "parameters": []})),
        ("legacy-malformed-parameters", serde_json::json!({"modelId": "Model", "maxMode": true, "parameters": [null]})),
    ]);
    for (label, model) in cases {
        let original = serde_json::to_vec(&serde_json::json!({
            "version": 1, "agentDefaultModel": model,
            "notifications": {"isEnabled": true, "legacy": "keep"}, "unrelated": [1, 2]
        })).expect("malformed selection fixture");
        fs::write(&path, &original).expect("seed malformed selection");
        assert!(service.get_agent_default_model_checked().is_err(), "{label}");
        assert_eq!(fs::read(&path).expect("unchanged malformed source"), original, "{label}");
        assert!(observed.lock().expect("observed changes").is_empty(), "{label}");
    }
    let _ = fs::remove_file(path);
}

#[test]
fn canonical_provider_bound_models_round_trip_through_both_settings_writers() {
    let path = temp_path("canonical-bound-models");
    fs::write(&path, br#"{"version":1,"inferenceProvider":"cursor","unrelated":{"keep":true}}"#)
        .expect("seed independent route and unknown field");
    let service = SettingsService::new(&path);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));
    for provider in ["fabushi", "cursor", "codex", "claude-code", "openrouter"] {
        for whole_document in [false, true] {
            let model = serde_json::json!({
                "provider": provider,
                "modelId": format!(" Org/Nested/Model@{provider}-{whole_document} "),
                "maxMode": false,
                "parameters": [{"id": "reasoning", "value": "high", "unknown": [1, 2]}],
                "extension": {"keep": true}
            });
            let mut expected = model.clone();
            expected["maxMode"] = serde_json::json!(true);
            if whole_document {
                let reply = service.set_host_settings(&serde_json::json!({"agentDefaultModel": model}))
                    .expect("whole-document model commit");
                assert_eq!(reply["agentDefaultModel"], expected);
            } else {
                assert!(service.set_agent_default_model(Some(&model)).expect("specialized model commit"));
            }
            assert_eq!(service.get_agent_default_model_checked().expect("checked committed model"), Some(expected.clone()));
            assert_eq!(service.get_agent_default_model(), Some(expected.clone()));
            assert_eq!(SettingsService::new(&path).get_agent_default_model_checked().expect("reopened model"), Some(expected.clone()));
            let stored: serde_json::Value = serde_json::from_slice(&fs::read(&path).expect("persisted bytes"))
                .expect("persisted JSON");
            assert_eq!(stored["agentDefaultModel"], expected);
            assert_eq!(stored["unrelated"], serde_json::json!({"keep": true}));
            assert_eq!(stored["inferenceProvider"], "cursor", "a model binding cannot select its provider route");
        }
    }
    let expected_notifications = vec![vec!["agentDefaultModel".to_string()]; 10];
    assert_eq!(observed.lock().expect("observed changes").as_slice(), expected_notifications.as_slice());
    assert!(service.set_agent_default_model(None).expect("clear specialized default"));
    assert_eq!(service.get_agent_default_model_checked().expect("cleared default"), None);
    assert!(!service.set_agent_default_model(None).expect("redundant clear"));
    assert_eq!(observed.lock().expect("observed changes").len(), 11);
    let _ = fs::remove_file(path);
}

#[test]
fn malformed_bound_model_updates_preserve_bytes_and_all_observers() {
    let path = temp_path("refused-bound-model");
    let original = br#"{"version":1,"mcpCustomInstructionsAccountScope":"account-a","notifications":{"isEnabled":true,"legacy":"keep"},"agentDefaultModel":{"provider":"fabushi","modelId":"Original/Model","maxMode":true,"parameters":[]},"localToolPermission":"allow","skills":{"entries":{"demo":{"enabled":true}}},"unrelated":{"keep":true}}"#;
    fs::write(&path, original).expect("seed model and account state");
    let service = SettingsService::new(&path);
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _changes = service.subscribe_to_changes(Arc::new(move |_| {
        sink.lock().expect("observed changes").push("settings");
    }));
    let sink = Arc::clone(&observed);
    let _timezone = service.subscribe_to_user_time_zone(Arc::new(move |_| {
        sink.lock().expect("observed changes").push("timezone");
    }));
    let sink = Arc::clone(&observed);
    let _features = service.subscribe_to_feature_flag_overrides(Arc::new(move |_| {
        sink.lock().expect("observed changes").push("features");
    }));

    for (label, model) in malformed_provider_bound_models() {
        assert!(service.set_agent_default_model(Some(&model)).is_err(), "{label}");
        assert_eq!(fs::read(&path).expect("source after specialized refusal"), original, "{label}");
        let result = service.set_host_settings_if_current(None, serde_json::json!({
            "agentDefaultModel": model,
            "mcpCustomInstructionsAccountScope": "account-b",
            "skills": {"entries": {"demo": {"enabled": false}}},
            "userTimeZone": "UTC", "featureFlagOverrides": {"model_validation": true}
        }));
        assert!(matches!(result, Err(HostSettingsUpdateError::Rejected(_))), "{label}");
        assert_eq!(fs::read(&path).expect("source after whole-document refusal"), original, "{label}");
        assert!(observed.lock().expect("observed changes").is_empty(), "{label}");
        assert_eq!(service.get_agent_default_model_checked().expect("original default remains")
            .expect("original selection")["modelId"], "Original/Model");
    }
    let _ = fs::remove_file(path);
}

#[test]
fn legacy_unbound_writes_and_computer_models_keep_their_compatibility_contract() {
    let path = temp_path("legacy-model-writes");
    let service = SettingsService::new(&path);
    let mut legacy = serde_json::json!({"modelId": "   ", "maxMode": false, "parameters": []});
    assert!(service.set_agent_default_model(Some(&legacy)).expect("legacy blank model behavior"));
    legacy["maxMode"] = serde_json::json!(true);
    assert_eq!(service.get_agent_default_model(), Some(legacy.clone()));
    assert!(service.get_agent_default_model_checked().is_err());
    let before = fs::read(&path).expect("legacy bytes");
    assert!(!service.set_agent_default_model(Some(&serde_json::json!({"modelId": "missing-shape"})))
        .expect("malformed unbound specialized update is ignored"));
    assert!(!service.set_agent_default_model(Some(&serde_json::Value::Null))
        .expect("specialized null retains its old no-op meaning"));
    assert_eq!(fs::read(&path).expect("unchanged legacy bytes"), before);

    let reply = service.set_host_settings(&serde_json::json!({
        "agentDefaultModel": {"modelId": "missing-shape"}, "userTimeZone": "UTC"
    })).expect("malformed unbound whole-document field remains ignored");
    assert_eq!(reply["agentDefaultModel"], legacy);
    assert_eq!(reply["userTimeZone"], "UTC");
    let legacy_control = serde_json::json!({"modelId": "Model\n", "maxMode": false, "parameters": []});
    service.set_host_settings(&serde_json::json!({"agentDefaultModel": legacy_control}))
        .expect("legacy whole-document control-containing model behavior");
    assert_eq!(service.get_agent_default_model().expect("compatibility projection")["modelId"], "Model\n");
    assert!(service.get_agent_default_model_checked().is_err());

    let computer = serde_json::json!({"provider": null, "modelId": "   ", "maxMode": false, "parameters": []});
    assert!(service.set_computer_use_model(Some(&computer)).expect("unchanged computer model validator"));
    assert_eq!(service.get_computer_use_model(), Some(computer));
    let computer_control = serde_json::json!({"provider": "legacy-alias", "modelId": "Computer\n", "maxMode": false, "parameters": []});
    let reply = service.set_host_settings(&serde_json::json!({"computerUseModel": computer_control}))
        .expect("unchanged whole-document computer model validator");
    assert_eq!(reply["computerUseModel"], computer_control);
    let _ = fs::remove_file(path);
}

#[test]
fn provider_bound_model_account_clearing_and_same_update_replacement_keep_their_order() {
    let path = temp_path("bound-model-account-order");
    let service = SettingsService::new(&path);
    service.set_host_settings(&serde_json::json!({
        "mcpCustomInstructionsAccountScope": "account-a",
        "agentDefaultModel": {"provider": "fabushi", "modelId": "A/Model", "maxMode": false, "parameters": []},
        "localToolPermission": "allow"
    })).expect("first account default");
    let reply = service.set_host_settings(&serde_json::json!({
        "mcpCustomInstructionsAccountScope": "account-b",
        "agentDefaultModel": {"provider": "openrouter", "modelId": "B/Model", "maxMode": false, "parameters": []}
    })).expect("clear old account before replacing its model");
    let expected = serde_json::json!({"provider": "openrouter", "modelId": "B/Model", "maxMode": true, "parameters": []});
    assert_eq!(reply["agentDefaultModel"], expected);
    assert_eq!(reply["localToolPermission"], "ask");
    assert_eq!(SettingsService::new(&path).get_agent_default_model_checked().expect("reopened account model"), Some(expected.clone()));
    assert!(service.scope_to_account("account-c").expect("clear account-bound model"));
    assert_eq!(service.get_agent_default_model_checked().expect("new account without default"), None);
    assert!(service.set_agent_default_model(Some(&expected)).expect("replace cleared model"));
    let reply = service.set_host_settings(&serde_json::json!({"agentDefaultModel": null}))
        .expect("whole-document null clears the bound model");
    assert!(reply.get("agentDefaultModel").is_none());
    assert_eq!(service.get_agent_default_model_checked().expect("explicitly cleared model"), None);
    let _ = fs::remove_file(path);
}

#[test]
fn conditional_settings_reply_keeps_its_commit_when_a_listener_commits_again() {
    let path = temp_path("cas-reentrant-listener");
    let service = Arc::new(SettingsService::new(&path));
    let initial = service.get_host_settings_snapshot();
    let entered = Arc::new(AtomicBool::new(false));
    let entered_for_listener = Arc::clone(&entered);
    let observed = Arc::new(Mutex::new(None));
    let sink = Arc::clone(&observed);
    let weak_owner = Arc::downgrade(&service);
    let _subscription = service.subscribe_to_changes(Arc::new(move |_| {
        if entered_for_listener.swap(true, Ordering::SeqCst) {
            return;
        }
        let owner = weak_owner.upgrade().expect("settings owner remains live");
        let first = owner.get_host_settings_snapshot();
        let second = owner
            .set_host_settings_if_current(
                Some(&first.hash),
                serde_json::json!({"inferenceProvider": "claude-code"}),
            )
            .expect("listener can synchronously commit a follow-up");
        *sink.lock().expect("callback snapshots") = Some((first, second));
    }));

    let owner = Arc::clone(&service);
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let result = owner.set_host_settings_if_current(
            Some(&initial.hash),
            serde_json::json!({"inferenceProvider": "openrouter"}),
        );
        sender.send(result).expect("deliver original commit");
    });
    let reply = receiver
        .recv_timeout(Duration::from_secs(10))
        .expect("callbacks must run outside the settings writer lock")
        .expect("original transaction commits");
    worker.join().expect("settings writer thread");

    let (first, second) = observed
        .lock()
        .expect("callback snapshots")
        .take()
        .expect("callback read and follow-up write both completed");
    assert_eq!(reply, first);
    assert_eq!(reply.settings["inferenceProvider"], "openrouter");
    assert_eq!(second.settings["inferenceProvider"], "claude-code");
    assert_ne!(reply.hash, second.hash);
    assert_ne!(reply.config_source["rawHash"], second.config_source["rawHash"]);
    assert_eq!(
        reply.hash,
        sha256_hex(serde_json::to_vec(&reply.settings).expect("serialize original reply"))
    );
    assert_eq!(service.get_host_settings_snapshot(), second);
    let _ = fs::remove_file(path);
}

#[test]
fn concurrent_specialized_commits_wait_for_the_active_notification_batch() {
    let path = temp_path("concurrent-notification-order");
    let service = Arc::new(SettingsService::new(&path));
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _changes = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("ordered changes").push(fields);
    }));
    let (entered_sender, entered_receiver) = mpsc::channel();
    let (release_sender, release_receiver) = mpsc::channel();
    let release_receiver = Mutex::new(release_receiver);
    let _timezone = service.subscribe_to_user_time_zone(Arc::new(move |_| {
        entered_sender.send(()).expect("signal active timezone callback");
        release_receiver.lock().expect("release receiver")
            .recv_timeout(Duration::from_secs(10))
            .expect("release active timezone callback");
    }));
    let first_owner = Arc::clone(&service);
    let (first_sender, first_receiver) = mpsc::channel();
    let first = std::thread::spawn(move || {
        first_sender.send(first_owner.set_user_time_zone(Some("Asia/Tokyo")))
            .expect("deliver timezone receipt");
    });
    entered_receiver.recv_timeout(Duration::from_secs(10))
        .expect("timezone callback starts");

    let later_owner = Arc::clone(&service);
    let (later_sender, later_receiver) = mpsc::channel();
    let later = std::thread::spawn(move || {
        let provider = later_owner.set_inference_provider("openrouter");
        let onboarding = later_owner.set_has_seen_onboarding(true);
        later_sender.send((provider, onboarding)).expect("deliver later receipts");
    });
    let (provider, onboarding) = later_receiver.recv_timeout(Duration::from_secs(10))
        .expect("active callbacks must not retain the writer lock");
    assert!(provider.expect("provider commit"));
    assert!(onboarding.expect("onboarding commit"));
    assert_eq!(service.get_inference_provider(), "openrouter");
    assert_eq!(service.get_has_seen_onboarding(), Some(true));
    assert!(observed.lock().expect("ordered changes").is_empty(),
        "later commits return receipts but cannot overtake A's batch");

    release_sender.send(()).expect("allow notifications to drain");
    assert!(first_receiver.recv_timeout(Duration::from_secs(10))
        .expect("timezone receipt after queue drains").expect("timezone commit"));
    first.join().expect("timezone writer");
    later.join().expect("later writers");
    assert_eq!(observed.lock().expect("ordered changes").as_slice(), &[
        vec!["userTimeZone".to_string()],
        vec!["inferenceProvider".to_string()],
        vec!["hasSeenOnboarding".to_string()],
    ]);
    let _ = fs::remove_file(path);
}

#[test]
fn listener_panic_drains_remaining_batches_and_releases_the_drainer() {
    let path = temp_path("notification-panic-recovery");
    let service = Arc::new(SettingsService::new(&path));
    let panic_once = Arc::new(AtomicBool::new(false));
    let panic_once_for_listener = Arc::clone(&panic_once);
    let weak_owner = Arc::downgrade(&service);
    let _timezone = service.subscribe_to_user_time_zone(Arc::new(move |_| {
        if !panic_once_for_listener.swap(true, Ordering::SeqCst) {
            weak_owner.upgrade().expect("settings owner")
                .set_host_settings_if_current(None, serde_json::json!({
                    "inferenceProvider": "codex",
                    "featureFlagOverrides": {"panic_recovery_gate": false}
                }))
                .expect("reentrant commit before the listener panic");
            panic!("intentional settings listener panic");
        }
    }));
    let second_timezone_calls = Arc::new(Mutex::new(0));
    let second_timezone_sink = Arc::clone(&second_timezone_calls);
    let _second_timezone = service.subscribe_to_user_time_zone(Arc::new(move |_| {
        *second_timezone_sink.lock().expect("later timezone listener") += 1;
    }));
    let overrides = Arc::new(Mutex::new(Vec::new()));
    let override_sink = Arc::clone(&overrides);
    let _features = service.subscribe_to_feature_flag_overrides(Arc::new(move |values| {
        override_sink.lock().expect("ordered overrides").push(values["panic_recovery_gate"]);
    }));
    let changes = Arc::new(Mutex::new(Vec::new()));
    let changes_sink = Arc::clone(&changes);
    let _changes = service.subscribe_to_changes(Arc::new(move |fields| {
        changes_sink.lock().expect("ordered changes").push(fields);
    }));

    let owner = Arc::clone(&service);
    let (sender, receiver) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            owner.set_host_settings_if_current(None, serde_json::json!({
                "userTimeZone": "Asia/Tokyo",
                "inferenceProvider": "openrouter",
                "featureFlagOverrides": {"panic_recovery_gate": true}
            }))
        }));
        sender.send(outcome).expect("deliver recovered unwind");
    });
    assert!(receiver.recv_timeout(Duration::from_secs(10))
        .expect("listener unwind cannot strand its drainer").is_err());
    worker.join().expect("panic was caught after draining");
    assert_eq!(*second_timezone_calls.lock().expect("later timezone listener"), 1);
    assert_eq!(overrides.lock().expect("ordered overrides").as_slice(), &[true, false]);
    assert_eq!(service.get_inference_provider(), "codex");
    assert_eq!(changes.lock().expect("ordered changes").len(), 2);

    service.set_host_settings_if_current(None, serde_json::json!({
        "userTimeZone": "UTC",
        "inferenceProvider": "cursor",
        "featureFlagOverrides": {"panic_recovery_gate": true}
    })).expect("later commit can own and drain the notification queue");
    assert_eq!(*second_timezone_calls.lock().expect("later timezone listener"), 2);
    assert_eq!(overrides.lock().expect("ordered overrides").as_slice(), &[true, false, true]);
    assert_eq!(changes.lock().expect("ordered changes").len(), 3);
    assert_eq!(service.get_inference_provider(), "cursor");
    let _ = fs::remove_file(path);
}

#[test]
fn conditional_readonly_rejection_preserves_bytes_and_observers() {
    let path = temp_path("cas-readonly");
    let original = br#"{"version":1,"readOnly":true,"notifications":{"isEnabled":true},"inferenceProvider":"openrouter"}"#;
    fs::write(&path, original).expect("seed readonly settings");
    let service = SettingsService::new(&path);
    let current = service.get_host_settings_snapshot();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));

    let error = service
        .set_host_settings_if_current(
            Some(&current.hash),
            serde_json::json!({"inferenceProvider": "cursor"}),
        )
        .expect_err("readonly config rejects a matching hash");
    assert!(matches!(&error, HostSettingsUpdateError::Rejected(message) if message.contains("readOnly")));
    assert_eq!(GatewayCommandError::from(error).status(), 500);
    assert_eq!(fs::read(&path).expect("unchanged readonly settings"), original);
    assert_eq!(service.get_host_settings_snapshot(), current);
    assert!(observed.lock().expect("observed changes").is_empty());
    let _ = fs::remove_file(path);
}

#[test]
fn conditional_unreadable_source_rejects_candidate_and_leaves_no_commit_event() {
    let root = temp_path("cas-io-failure-root");
    let path = root.join("settings.json");
    // A directory is an unreadable settings source even for privileged CI users.
    fs::create_dir_all(&path).expect("create blocked settings destination");
    fs::write(path.join("keep"), b"untouched").expect("seed destination marker");
    let service = SettingsService::new(&path);
    assert!(service.get_agent_default_model_checked().is_err());
    let current = service.get_host_settings_snapshot();
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
        sink.lock().expect("observed changes").push(fields);
    }));

    let error = service
        .set_host_settings_if_current(
            Some(&current.hash),
            serde_json::json!({"inferenceProvider": "openrouter"}),
        )
        .expect_err("unreadable input cannot acknowledge a replacement candidate");
    assert!(matches!(error, HostSettingsUpdateError::Rejected(_)));
    assert!(path.is_dir());
    assert_eq!(fs::read(path.join("keep")).expect("destination marker"), b"untouched");
    assert_eq!(service.get_host_settings_snapshot(), current);
    assert!(observed.lock().expect("observed changes").is_empty());
    assert_eq!(fs::read_dir(&root).expect("settings directory").count(), 1);
    let _ = fs::remove_dir_all(root);
}

#[test]
fn invalid_sources_are_not_replaced_by_getter_normalization_or_settings_writers() {
    let fixtures: [(&str, &[u8]); 5] = [
        ("invalid-json", b"{broken settings source"),
        ("invalid-utf8", b"\xff\xfe"),
        ("non-object", b"[1, 2, 3]"),
        ("unsupported-version", br#"{"version":2,"keep":"original"}"#),
        ("missing-version", br#"{"keep":"original"}"#),
    ];
    for (label, original) in fixtures {
        let path = temp_path(label);
        fs::write(&path, original).expect("seed invalid settings source");
        let service = SettingsService::new(&path);
        let observed = Arc::new(Mutex::new(Vec::new()));
        let sink = Arc::clone(&observed);
        let _subscription = service.subscribe_to_changes(Arc::new(move |fields| {
            sink.lock().expect("observed changes").push(fields);
        }));

        assert!(service.get_agent_default_model_checked().is_err(), "{label}");
        assert_eq!(service.get_agent_default_model(), None, "legacy read compatibility");
        assert_eq!(service.get_notification_config()["isEnabled"], false);
        let fallback = service.get_host_settings_snapshot();
        assert_eq!(service.get_host_settings(), fallback.settings);
        assert_eq!(fallback.config_source["exists"], true);
        assert_eq!(fallback.config_source["valid"], false);
        assert_eq!(fallback.config_source["rawHash"], sha256_hex(original));
        assert_eq!(fs::read(&path).expect("source after getters"), original);

        assert!(matches!(
            service.set_host_settings_if_current(
                Some(&fallback.hash),
                serde_json::json!({"inferenceProvider": "openrouter"}),
            ),
            Err(HostSettingsUpdateError::Rejected(_))
        ));
        assert!(service.set_has_seen_onboarding(true).is_err());
        assert!(service.set_host_settings(&serde_json::json!({"userTimeZone": "UTC"})).is_err());
        assert_eq!(fs::read(&path).expect("source after rejected writers"), original);
        assert!(observed.lock().expect("observed changes").is_empty());
        let _ = fs::remove_file(path);
    }
}

#[test]
fn stale_account_snapshot_cannot_restore_cleared_models_and_restart_reads_the_commit() {
    let path = temp_path("cas-account-restart");
    fs::write(&path, br#"{"version":1,"unrelated":{"keep":true}}"#)
        .expect("seed unknown stored field");
    let service = SettingsService::new(&path);
    let old_account = service
        .set_host_settings_if_current(
            None,
            serde_json::json!({
                "mcpCustomInstructionsAccountScope": "account-a",
                "agentDefaultModel": {"modelId": "model-a", "maxMode": false, "parameters": []},
                "computerUseModel": {"modelId": "computer-a", "maxMode": false, "parameters": []},
                "localToolPermission": "allow",
                "mcpCustomInstructions": {"Demo": "old account instruction"}
            }),
        )
        .expect("first account settings");
    assert!(service.scope_to_account("account-b").expect("switch account"));
    let current = service.get_host_settings_snapshot();
    let bytes_before_stale_write = fs::read(&path).expect("account-b bytes");
    assert_eq!(current.settings["mcpCustomInstructionsAccountScope"], "account-b");
    assert!(current.settings.get("agentDefaultModel").is_none());
    assert!(current.settings.get("computerUseModel").is_none());
    assert_eq!(current.settings["localToolPermission"], "ask");
    assert_eq!(current.settings["mcpCustomInstructions"], serde_json::json!({}));
    assert!(matches!(
        service.set_host_settings_if_current(Some(&old_account.hash), old_account.settings),
        Err(HostSettingsUpdateError::Conflict { .. })
    ));
    assert_eq!(fs::read(&path).expect("unchanged account-b bytes"), bytes_before_stale_write);

    let committed = service
        .set_host_settings_if_current(
            Some(&current.hash),
            serde_json::json!({
                "inferenceProvider": "openrouter",
                "agentDefaultModel": {"modelId": "model-b", "maxMode": false, "parameters": []}
            }),
        )
        .expect("new account model commits");
    drop(service);
    let reopened = SettingsService::new(&path);
    assert_eq!(reopened.get_host_settings_snapshot(), committed);
    assert_eq!(reopened.get_agent_default_model().expect("persisted model")["modelId"], "model-b");
    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("persisted settings"))
            .expect("persisted JSON");
    assert_eq!(stored["unrelated"], serde_json::json!({"keep": true}));
    let _ = fs::remove_file(path);
}

#[test]
fn normalization_and_specialized_writers_preserve_each_others_commits() {
    let path = temp_path("shared-writer-normalization");
    fs::write(
        &path,
        br#"{"version":1,"notifications":{"isEnabled":true,"legacy":"drop"},"unrelated":{"keep":true}}"#,
    )
    .expect("seed legacy settings");
    let service = Arc::new(SettingsService::new(&path));
    let barrier = Arc::new(Barrier::new(8));
    let mut handles = Vec::new();
    for action in 0..7 {
        let owner = Arc::clone(&service);
        let barrier = Arc::clone(&barrier);
        handles.push(std::thread::spawn(move || {
            barrier.wait();
            match action {
                0 => {
                    owner.get_notification_config();
                }
                1 => {
                    assert!(owner.set_agent_default_model(Some(&serde_json::json!({
                        "modelId": "shared-writer-model", "maxMode": false, "parameters": []
                    }))).expect("model writer"));
                }
                2 => {
                    assert!(owner.set_user_time_zone(Some("Asia/Tokyo")).expect("timezone writer"));
                }
                3 => {
                    assert!(owner.set_mcp_custom_instruction_by_server_id("17", "Demo", "keep instruction", true)
                        .expect("MCP writer"));
                }
                4 => {
                    assert!(owner.set_has_seen_onboarding(true).expect("onboarding writer"));
                }
                _ => {
                    assert!(owner.record_inference_usage("openrouter", Some(2.0), Some(1.0), None, None)
                        .expect("usage writer"));
                }
            }
        }));
    }
    barrier.wait();
    for handle in handles {
        handle.join().expect("specialized settings writer thread");
    }

    let stored: serde_json::Value =
        serde_json::from_slice(&fs::read(&path).expect("persisted settings"))
            .expect("persisted JSON");
    assert_eq!(stored["notifications"], serde_json::json!({"isEnabled": false}));
    assert_eq!(stored["unrelated"], serde_json::json!({"keep": true}));
    assert_eq!(stored["agentDefaultModel"]["modelId"], "shared-writer-model");
    assert_eq!(stored["agentDefaultModel"]["maxMode"], true);
    assert_eq!(stored["userTimeZone"], "Asia/Tokyo");
    assert_eq!(stored["mcpCustomInstructionsByServerId"]["17"], "keep instruction");
    assert_eq!(stored["hasSeenOnboarding"], true);
    assert_eq!(stored["inferenceRouterUsage"]["providers"]["openrouter"]["requests"], 2);
    assert_eq!(stored["inferenceRouterUsage"]["providers"]["openrouter"]["inputTokens"], 4);
    let _ = fs::remove_file(path);
}

#[test]
fn host_settings_projects_secret_audit_findings_without_secret_values() {
    let path = temp_path("secret-audit");
    let service = SettingsService::new(&path);
    let observed = Arc::new(Mutex::new(Vec::<HostDiagnostic>::new()));
    let sink = Arc::clone(&observed);
    pin_host_diagnostics_reporter(Some(Arc::new(move |diagnostic| {
        if diagnostic.kind == "security.secrets.finding" {
            sink.lock().expect("secret diagnostics").push(diagnostic.clone());
        }
    })));

    service
        .set_host_settings(&serde_json::json!({
            "agents": {
                "entries": {
                    "main": { "apiKey": "sk-settings-plaintext-secret" }
                }
            }
        }))
        .expect("host settings");

    pin_host_diagnostics_reporter(None);
    let findings = observed.lock().expect("secret diagnostics");
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].fields["findingKind"], "plaintext_residue");
    assert!(!serde_json::to_string(&findings[0].fields)
        .expect("serialize findings")
        .contains("sk-settings-plaintext-secret"));
    let _ = fs::remove_file(path);
}

#[test]
fn host_settings_updates_skill_entries_without_overwriting_redacted_secrets() {
    let path = temp_path("skill-mutation");
    fs::write(
        &path,
        r#"{"version":1,"skills":{"entries":{"demo":{"enabled":true,"apiKey":"existing","env":{"KEEP":"old","REMOVE":"gone"}}}}}"#,
    )
    .expect("seed skill settings");
    let service = SettingsService::new(&path);

    let host = service
        .set_host_settings(&serde_json::json!({
            "skills": {
                "entries": {
                    "demo": {
                        "enabled": false,
                        "apiKey": " [REDACTED] ",
                        "env": {
                            "KEEP": " [REDACTED] ",
                            "REMOVE": "",
                            "NEW": " value "
                        }
                    }
                }
            }
        }))
        .expect("skill settings");

    assert_eq!(host["skills"]["entries"]["demo"]["enabled"], false);
    assert_eq!(host["skills"]["entries"]["demo"]["apiKey"], "existing");
    assert_eq!(host["skills"]["entries"]["demo"]["env"]["KEEP"], "old");
    assert_eq!(host["skills"]["entries"]["demo"]["env"]["NEW"], "value");
    assert!(host["skills"]["entries"]["demo"]["env"]
        .get("REMOVE")
        .is_none());
    let _ = fs::remove_file(path);
}

#[test]
fn timezone_settings_validate_persist_preserve_unknown_fields_and_notify_effective_changes() {
    let path = temp_path("timezone");
    fs::write(
        &path,
        r#"{"version":1,"unrelated":{"keep":true},"userTimeZone":"UTC"}"#,
    )
    .expect("seed settings");
    let service = SettingsService::new(&path);
    assert!(is_valid_iana_time_zone("America/Los_Angeles"));
    assert!(!is_valid_iana_time_zone("Not/AZone"));
    assert_eq!(service.get_user_time_zone().as_deref(), Some("UTC"));

    let observed = Arc::new(Mutex::new(Vec::<Option<String>>::new()));
    let observed_for_listener = Arc::clone(&observed);
    let _subscription = service.subscribe_to_user_time_zone(Arc::new(move |zone| {
        observed_for_listener.lock().expect("observed lock").push(zone);
    }));

    assert!(!service
        .set_user_time_zone_override(Some("Not/AZone"))
        .expect("invalid zone ignored"));
    assert!(service
        .set_user_time_zone_override(Some(" America/Los_Angeles "))
        .expect("set override"));
    assert_eq!(
        service.get_user_time_zone().as_deref(),
        Some("America/Los_Angeles")
    );

    assert!(service
        .set_user_time_zone(Some("Europe/Berlin"))
        .expect("set detected zone"));
    assert_eq!(
        service.get_user_time_zone().as_deref(),
        Some("America/Los_Angeles")
    );

    assert!(service
        .set_user_time_zone_override(Some(""))
        .expect("clear override"));
    assert_eq!(service.get_user_time_zone().as_deref(), Some("Europe/Berlin"));

    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("settings text"))
            .expect("settings json");
    assert_eq!(persisted["unrelated"]["keep"], true);
    assert_eq!(
        observed.lock().expect("observed lock").as_slice(),
        &[
            Some("America/Los_Angeles".to_string()),
            Some("Europe/Berlin".to_string())
        ]
    );
    let _ = fs::remove_file(path);
}

#[test]
fn auto_review_instructions_default_normalize_persist_and_notify() {
    let path = temp_path("auto-review");
    let service = SettingsService::new(&path);
    assert_eq!(
        service.get_auto_review_instructions(),
        SandAutoReviewInstructions::default()
    );

    let changed = Arc::new(Mutex::new(Vec::<Vec<String>>::new()));
    let changed_for_listener = Arc::clone(&changed);
    let _subscription = service.subscribe_to_changes(Arc::new(move |keys| {
        changed_for_listener.lock().expect("changed lock").push(keys);
    }));

    let oversized = "x".repeat(SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS + 50);
    let mut allow = vec!["  allow shell  ".to_string(), "allow shell".to_string(), oversized];
    for index in 0..(SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES + 10) {
        allow.push(format!("allow-{index}"));
    }
    let instructions = SandAutoReviewInstructions {
        is_enabled: false,
        allow_instructions: allow,
        block_instructions: vec!["  block delete  ".into(), "".into()],
    };
    assert!(service
        .set_auto_review_instructions(&instructions)
        .expect("persist auto-review instructions"));

    let normalized = service.get_auto_review_instructions();
    assert!(!normalized.is_enabled);
    assert_eq!(normalized.allow_instructions[0], "allow shell");
    assert_eq!(normalized.allow_instructions.len(), SAND_AUTO_REVIEW_INSTRUCTION_MAX_ENTRIES);
    assert_eq!(
        normalized.allow_instructions[1].chars().count(),
        SAND_AUTO_REVIEW_INSTRUCTION_MAX_CHARS
    );
    assert_eq!(normalized.block_instructions, vec!["block delete".to_string()]);
    assert_eq!(
        changed.lock().expect("changed lock").as_slice(),
        &[vec!["autoReviewInstructions".to_string()]]
    );
    assert!(!service
        .set_auto_review_instructions(&normalized)
        .expect("identical normalized settings are stable"));

    let host = service.get_host_settings();
    assert_eq!(host["autoReviewInstructions"]["isEnabled"], false);
    let _ = fs::remove_file(path);
}


#[test]
fn frozen_mcp_settings_normalize_migrate_delete_and_account_scope() {
    let path = temp_path("mcp");
    let service = SettingsService::new(&path);
    let oversized = "x".repeat(MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH + 32);

    service
        .set_mcp_custom_instructions(BTreeMap::from([
            ("hex".into(), "".into()),
            ("plain".into(), "".into()),
            ("alpha".into(), oversized),
        ]))
        .expect("legacy instructions");
    let legacy = service.get_mcp_custom_instructions();
    assert_eq!(legacy.get("hex").map(String::as_str), Some(""));
    assert!(!legacy.contains_key("plain"));
    assert_eq!(
        legacy.get("alpha").map(|value| value.chars().count()),
        Some(MCP_CUSTOM_INSTRUCTIONS_MAX_LENGTH)
    );

    service
        .set_mcp_custom_instructions(BTreeMap::from([(
            "Legacy".into(),
            "instruction".into(),
        )]))
        .expect("legacy seed");
    assert!(service
        .migrate_mcp_custom_instruction_to_server_id("11", "Legacy")
        .expect("migrate"));
    assert_eq!(
        service
            .get_raw_mcp_custom_instruction_by_server_id("11")
            .as_deref(),
        Some("instruction")
    );
    assert!(service
        .delete_mcp_custom_instruction_by_server_id("11", "Legacy", true)
        .expect("delete"));
    assert!(service.get_raw_mcp_custom_instruction("Legacy").is_none());

    service
        .set_mcp_disabled_tools_by_server_id(BTreeMap::from([
            (
                "12".into(),
                vec!["A".into(), "A".into(), "".into(), "B".into()],
            ),
            ("bad".into(), vec!["X".into()]),
        ]))
        .expect("disabled tools");
    assert_eq!(
        service.get_mcp_disabled_tools_by_server_id(),
        BTreeMap::from([("12".into(), vec!["A".into(), "B".into()])])
    );

    service.scope_to_account("account-a").expect("initial scope");
    service
        .set_mcp_custom_instructions(BTreeMap::from([("x".into(), "y".into())]))
        .expect("scoped mcp");
    service
        .set_local_tool_permission(SandLocalToolPermission::Always)
        .expect("permission");
    assert!(service.scope_to_account("account-b").expect("scope change"));
    assert!(service.get_mcp_custom_instructions().is_empty());
    assert_eq!(
        service.get_local_tool_permission(),
        SandLocalToolPermission::Ask
    );
    let _ = fs::remove_file(path);
}

#[test]
fn permission_ceiling_feature_override_and_inference_usage_match_frozen_contract() {
    let path = temp_path("runtime");
    let service = SettingsService::new(&path);
    service
        .set_local_tool_permission(SandLocalToolPermission::Always)
        .expect("choice");
    service
        .set_local_tool_permission_ceiling(Some(SandLocalToolPermission::Ask))
        .expect("ceiling");
    assert_eq!(
        service.get_local_tool_permission(),
        SandLocalToolPermission::Ask
    );

    let observed = Arc::new(Mutex::new(Vec::<BTreeMap<String, bool>>::new()));
    let sink = Arc::clone(&observed);
    let _subscription =
        service.subscribe_to_feature_flag_overrides(Arc::new(move |overrides| {
            sink.lock().expect("feature overrides").push(overrides);
        }));
    service
        .set_host_settings(&serde_json::json!({
            "featureFlagOverrides": {
                "sand_multitask": false,
                "sand_spotlight": true
            }
        }))
        .expect("host settings");
    assert_eq!(
        observed.lock().expect("feature overrides").as_slice(),
        &[BTreeMap::from([
            ("sand_multitask".into(), false),
            ("sand_spotlight".into(), true),
        ])]
    );

    assert_eq!(service.get_inference_provider(), "cursor");
    assert!(service
        .set_inference_provider("codex")
        .expect("set provider"));
    assert!(!service
        .set_inference_provider("invalid")
        .expect("reject provider"));
    service
        .record_inference_usage(
            "codex",
            Some(10.4),
            Some(4.6),
            Some(-1.0),
            Some(f64::NAN),
        )
        .expect("record usage");
    let usage = service.get_inference_router_usage();
    assert_eq!(usage["providers"]["codex"]["requests"], 1);
    assert_eq!(usage["providers"]["codex"]["inputTokens"], 10);
    assert_eq!(usage["providers"]["codex"]["outputTokens"], 5);
    assert_eq!(usage["providers"]["codex"]["cacheReadTokens"], 0);
    assert_eq!(usage["providers"]["codex"]["cacheWriteTokens"], 0);
    assert!(usage["providers"]["codex"]["lastUsedAt"].is_string());

    let host = service.get_host_settings();
    assert_eq!(host["notifications"]["isEnabled"], false);
    assert_eq!(host["notifications"]["minIntervalMs"], 5000);
    let _ = fs::remove_file(path);
}


#[test]
fn host_settings_switch_scope_before_applying_same_update_and_normalizes_invalid_permission() {
    let path = temp_path("host-update-order");
    let service = SettingsService::new(&path);
    service.scope_to_account("account-a").expect("first scope");
    service
        .set_mcp_custom_instructions(BTreeMap::from([("old".into(), "old-value".into())]))
        .expect("old instructions");
    service
        .set_has_seen_onboarding(true)
        .expect("onboarding owner");

    let host = service
        .set_host_settings(&serde_json::json!({
            "mcpCustomInstructionsAccountScope": "account-b",
            "mcpCustomInstructions": { "new": "new-value" },
            "localToolPermission": { "not": "a permission" }
        }))
        .expect("scoped update");

    assert_eq!(
        host["mcpCustomInstructions"],
        serde_json::json!({ "new": "new-value" })
    );
    assert_eq!(host["localToolPermission"], "ask");
    assert!(host.get("hasSeenOnboarding").is_none());
    let _ = fs::remove_file(path);
}


#[test]
fn frozen_notification_and_default_auto_review_persistence_are_normalized() {
    let path = temp_path("normalization");
    fs::write(
        &path,
        r#"{"version":1,"notifications":{"isEnabled":true,"extra":"drop"},"autoReviewInstructions":{"isEnabled":false,"allowInstructions":["x"],"blockInstructions":[]}}"#,
    )
    .expect("seed settings");
    let service = SettingsService::new(&path);

    let notifications = service.get_notification_config();
    assert_eq!(notifications["isEnabled"], false);
    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("settings text"))
            .expect("settings json");
    assert_eq!(
        persisted["notifications"],
        serde_json::json!({"isEnabled": false})
    );

    assert!(service
        .set_auto_review_instructions(&SandAutoReviewInstructions::default())
        .expect("clear default auto review override"));
    let persisted: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&path).expect("settings text"))
            .expect("settings json");
    assert!(persisted.get("autoReviewInstructions").is_none());
    let _ = fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// OCRM-CONFIG-O01: Config Path Mutation & Prototype Pollution Defense
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_config_o01_path_mutation() {
    let mut config = serde_json::json!({
        "gateway": {
            "port": 8080
        }
    });

    // Valid path read and update
    assert_eq!(get_json_value_at_path(&config, "gateway.port"), Some(&serde_json::json!(8080)));
    assert!(set_json_value_at_path(&mut config, "gateway.auth.mode", serde_json::json!("token")).is_ok());
    assert_eq!(get_json_value_at_path(&config, "gateway.auth.mode"), Some(&serde_json::json!("token")));

    // Falsifiable: Prototype pollution prevention
    assert!(set_json_value_at_path(&mut config, "__proto__.polluted", serde_json::json!(true)).is_err());
    assert!(set_json_value_at_path(&mut config, "gateway.prototype.danger", serde_json::json!(true)).is_err());
    assert!(set_json_value_at_path(&mut config, "constructor.name", serde_json::json!("bad")).is_err());
    assert_eq!(get_json_value_at_path(&config, "__proto__.polluted"), None);
}

// ---------------------------------------------------------------------------
// OCRM-CONFIG-O02: Immutable Config Mode and Read-Only Guard
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_config_o02_immutable_mode_guard() {
    let path = temp_path("immutable-guard");
    fs::write(&path, r#"{"version":1,"readOnly":true}"#).expect("seed");
    let service = SettingsService::new(&path);

    // Reading settings works normally
    let host = service.get_host_settings();
    assert_eq!(host.get("version"), Some(&serde_json::json!(1)));
    assert_eq!(host.get("readOnly"), Some(&serde_json::json!(true)));
    assert!(service.is_read_only());

    // Mutations must fail closed and leave disk untouched
    assert!(service.set_has_seen_onboarding(true).is_err());
    assert!(service.set_host_settings(&serde_json::json!({"userTimeZone": "UTC"})).is_err());

    let on_disk = fs::read_to_string(&path).expect("read");
    assert!(!on_disk.contains("hasSeenOnboarding"), "Disk must remain untouched in readOnly mode");

    let _ = fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// OCRM-CONFIG-O03: Single Authoritative Writer Mutex
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_config_o03_single_writer_lock() {
    let path = temp_path("writer-lock");
    fs::write(&path, r#"{"version":1,"counter":0}"#).expect("seed");
    let service = Arc::new(SettingsService::new(&path));

    // Parallel mutation attempts through single service must serialize safely
    let mut handles = Vec::new();
    for i in 0..5 {
        let svc = Arc::clone(&service);
        handles.push(std::thread::spawn(move || {
            let _ = svc.set_host_settings(&serde_json::json!({
                "mcpCustomInstructionsAccountScope": format!("account-{}", i)
            }));
        }));
    }
    for h in handles {
        h.join().expect("thread join");
    }

    let final_settings = service.get_host_settings();
    assert!(final_settings.get("mcpCustomInstructionsAccountScope").is_some());
    let _ = fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// OCRM-CONFIG-O04: Atomic Settings IO Snapshot and Staging Rename
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_config_o04_io_atomic_snapshot() {
    let path = temp_path("atomic-io");
    fs::write(&path, r#"{"version":1,"key":"original"}"#).expect("seed");
    let service = SettingsService::new(&path);

    assert!(service.set_has_seen_onboarding(true).is_ok());

    // Verify file content on disk is valid JSON
    let text = fs::read_to_string(&path).expect("read");
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid json");
    assert_eq!(parsed.get("key"), Some(&serde_json::json!("original")));
    assert_eq!(parsed.get("hasSeenOnboarding"), Some(&serde_json::json!(true)));
    let _ = fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// OCRM-CONFIG-O05: Schema Normalization & Unknown Field Preservation
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_config_o05_schema_normalization() {
    let path = temp_path("schema-normalization");
    fs::write(
        &path,
        r#"{"version":1,"customUserField":"keep-me","userTimeZone":"UTC"}"#,
    )
    .expect("seed");
    let service = SettingsService::new(&path);

    // Update known field
    assert!(service.set_user_time_zone(Some("Asia/Tokyo")).is_ok());

    // Verify unknown field is preserved
    let on_disk = fs::read_to_string(&path).expect("read");
    assert!(on_disk.contains("customUserField"), "Unknown field must not be dropped");
    assert!(on_disk.contains("keep-me"));
    let _ = fs::remove_file(path);
}

// ---------------------------------------------------------------------------
// OCRM-CONFIG-O06: Agent Workspace Directory Bounds
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_config_o06_agent_workspace_dirs() {
    use mahayana_host_runtime::storage::agent_paths::{
        assert_valid_sand_agent_id, resolve_sand_agent_dir,
    };
    // Safe directory resolution
    assert!(assert_valid_sand_agent_id("agent-safe-01").is_ok());
    assert!(assert_valid_sand_agent_id("../escape").is_err());
    assert!(assert_valid_sand_agent_id("agent/sub").is_err());
}

#[test]
fn test_ocrm_roster_include_write_guard_is_server_owned_and_fail_closed() {
    let included_path = temp_path("roster-include-owned");
    fs::write(
        &included_path,
        r#"{"version":1,"agents":{"entries":{"$include":"agents.json"}}}"#,
    )
    .expect("seed include-owned settings");
    let included = SettingsService::new(&included_path);

    // Ordinary settings writes must not bypass the guard merely because no
    // doctorFix marker was supplied by the caller.
    assert!(included
        .set_host_settings(&serde_json::json!({
            "agents": {"entries": {"local": {}}}
        }))
        .is_err());
    let unchanged: serde_json::Value =
        serde_json::from_str(&fs::read_to_string(&included_path).expect("read"))
            .expect("valid json");
    assert_eq!(
        unchanged["agents"]["entries"]["$include"],
        serde_json::json!("agents.json")
    );
    let _ = fs::remove_file(included_path);

    let target_include_path = temp_path("roster-include-target");
    let target_include = SettingsService::new(&target_include_path);
    // A client cannot smuggle an include-owned roster into a clean config.
    assert!(target_include
        .set_host_settings(&serde_json::json!({
            "agents": {"entries": {"$include": "agents.json"}}
        }))
        .is_err());
    let _ = fs::remove_file(target_include_path);
}

