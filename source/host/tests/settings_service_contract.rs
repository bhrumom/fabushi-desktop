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

    let malformed_binding = serde_json::json!({
        "provider": null, "modelId": "bad-binding", "maxMode": true, "parameters": []
    });
    fs::write(&path, serde_json::to_vec(&serde_json::json!({
        "version": 1, "agentDefaultModel": malformed_binding
    })).expect("encode malformed existing binding"))
        .expect("seed malformed existing binding");
    assert!(service.get_agent_default_model_checked().is_err());
    assert!(service.set_has_seen_onboarding(true).expect("unrelated writer keeps its contract"));
    assert!(service.get_agent_default_model_checked().is_err(),
        "an unrelated write does not repair the invalid model binding");
    let persisted: serde_json::Value = serde_json::from_slice(
        &fs::read(&path).expect("read unrelated write"),
    ).expect("decode unrelated write");
    assert_eq!(persisted["hasSeenOnboarding"], true);
    assert_eq!(persisted["agentDefaultModel"], malformed_binding);
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

// OCRM-CONFIG-PUBLIC-SECRET-v1: exercise the real Settings transaction owner.
// These fixtures never share an installation directory or revision-key path.
mod public_settings_owner_contract {
    use std::collections::BTreeMap;
    use std::fs;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicBool, Ordering};
    use std::sync::{Arc, Barrier, Mutex, mpsc};
    use std::time::Duration;

    use mahayana_host_runtime::extensions::settings::settings_service::{
        HostSettingsUpdateError, PublicHostSettingsCommit, PublicHostSettingsError,
        PublicHostSettingsSnapshot, SettingsService, SettingsSubscription,
    };
    use mahayana_host_runtime::sha256::sha256_hex;
    use serde_json::{Value, json};
    use uuid::Uuid;

    const MASK: &str = "[REDACTED]";
    const UPSTREAM_MASK: &str = "__OPENCLAW_REDACTED__";
    const AGENT_A: &str = "synthetic-agent-a";
    const AGENT_B: &str = "synthetic-agent-b";
    const SKILL_KEY: &str = "synthetic-skill-key";
    const SKILL_ENV: &str = "synthetic-skill-env";
    const LOCAL_ENV: &str = "synthetic-local-env";
    const PRIVATE_URL: &str = "https://fixture:synthetic-url@example.invalid/callback";

    struct Installation {
        root: PathBuf,
    }

    impl Installation {
        fn new() -> Self {
            let root = std::env::temp_dir().join(format!(
                "fabushi-public-settings-{}", Uuid::new_v4().simple()
            ));
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            {
                use std::os::unix::fs::DirBuilderExt;
                builder.mode(0o700);
            }
            // Exclusive creation is the ownership proof for recursive cleanup.
            builder.create(&root).expect("create private settings installation");
            Self { root }
        }

        fn seeded(value: &Value) -> Self {
            let installation = Self::new();
            installation.write(value);
            installation
        }

        fn settings_path(&self) -> PathBuf {
            self.root.join("settings.json")
        }

        fn key_path(&self) -> PathBuf {
            self.root.join("host-config-revision-key.bin")
        }

        fn write(&self, value: &Value) {
            let bytes = serde_json::to_vec(value).expect("serialize synthetic settings fixture");
            fs::write(self.settings_path(), bytes).expect("write owned settings fixture");
        }

        fn bytes(&self) -> Vec<u8> {
            fs::read(self.settings_path()).expect("read owned settings fixture")
        }

        fn stored(&self) -> Value {
            serde_json::from_slice(&self.bytes()).expect("read valid owned settings document")
        }

        fn owner(&self) -> SettingsService {
            SettingsService::new(self.settings_path())
        }
    }

    impl Drop for Installation {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[derive(Clone, PartialEq)]
    enum Notice {
        Changes(Vec<String>),
        TimeZone(Option<String>),
        Features(BTreeMap<String, bool>),
    }

    struct Observers {
        notices: Arc<Mutex<Vec<Notice>>>,
        _changes: SettingsSubscription<dyn Fn(Vec<String>) + Send + Sync + 'static>,
        _time_zone: SettingsSubscription<dyn Fn(Option<String>) + Send + Sync + 'static>,
        _features: SettingsSubscription<dyn Fn(BTreeMap<String, bool>) + Send + Sync + 'static>,
    }

    impl Observers {
        fn new(owner: &SettingsService) -> Self {
            let notices = Arc::new(Mutex::new(Vec::new()));
            let sink = Arc::clone(&notices);
            let changes = owner.subscribe_to_changes(Arc::new(move |fields| {
                sink.lock().expect("record settings notice").push(Notice::Changes(fields));
            }));
            let sink = Arc::clone(&notices);
            let time_zone = owner.subscribe_to_user_time_zone(Arc::new(move |zone| {
                sink.lock().expect("record timezone notice").push(Notice::TimeZone(zone));
            }));
            let sink = Arc::clone(&notices);
            let features = owner.subscribe_to_feature_flag_overrides(Arc::new(move |flags| {
                sink.lock().expect("record feature notice").push(Notice::Features(flags));
            }));
            Self { notices, _changes: changes, _time_zone: time_zone, _features: features }
        }

        fn captured(&self) -> Vec<Notice> {
            self.notices.lock().expect("read settings notices").clone()
        }

        fn commit_count(&self) -> usize {
            self.captured().iter().filter(|notice| matches!(notice, Notice::Changes(_))).count()
        }
    }

    fn source_document() -> Value {
        json!({
            "version": 1,
            "notifications": {"isEnabled": false},
            "agents": {"ownership": "explicit", "list": [
                {"id": "alpha", "default": true, "name": "Alpha", "apiKey": AGENT_A,
                 "model": {"provider": "openrouter", "modelId": "org/model"},
                 "maxTokens": 4096, "tokenBudget": 128, "passwordFile": "local-password.txt",
                 "localService": {"command": "fixture-service", "env": {"PLAIN_NAME": LOCAL_ENV}},
                 "callback": PRIVATE_URL},
                {"id": "beta", "name": "Beta", "apiKey": AGENT_B}
            ]},
            "skills": {"entries": {"demo": {
                "enabled": true, "apiKey": SKILL_KEY,
                "env": {"PLAIN_NAME": SKILL_ENV, "maxTokens": "synthetic-env-budget"},
                "extension": {"keep": true}
            }}},
            "inferenceProvider": "openrouter",
            "agentDefaultModel": {"provider": "openrouter", "modelId": "org/model",
                                  "maxMode": true, "parameters": []},
            "localToolPermission": "ask", "userTimeZone": "UTC",
            "pinnedAgentIds": ["alpha"],
            "unrelatedStoredExtension": {"keep": [1, 2]}
        })
    }

    fn snapshot(owner: &SettingsService) -> PublicHostSettingsSnapshot {
        match owner.get_public_host_settings_snapshot() {
            Ok(snapshot) => snapshot,
            Err(_) => panic!("expected a readable public settings snapshot"),
        }
    }

    fn commit(owner: &SettingsService, token: Option<&str>, update: Value) -> PublicHostSettingsCommit {
        match owner.set_public_host_settings_if_current(token, update) {
            Ok(receipt) => receipt,
            Err(_) => panic!("expected a valid public settings transaction"),
        }
    }

    fn assert_token(token: &str) {
        let encoded = token.strip_prefix("hmac-sha256:v1:")
            .expect("public revisions must use the versioned opaque format");
        assert!(encoded.len() == 43 && encoded.bytes().all(|byte| {
            byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_')
        }), "public revisions must contain one unpadded base64url SHA-256 MAC");
    }

    fn assert_no_disclosure(snapshot: &PublicHostSettingsSnapshot, hidden: &[&str]) {
        let serialized = serde_json::to_string(&json!({
            "settings": &snapshot.settings, "hash": &snapshot.hash,
            "configSource": &snapshot.config_source
        })).expect("serialize complete public owner response");
        for value in hidden {
            assert!(!serialized.contains(*value), "complete public snapshot disclosed a protected value");
        }
        assert_token(&snapshot.hash);
        if let Some(raw_hash) = snapshot.config_source["rawHash"].as_str() {
            assert_token(raw_hash);
        }
    }

    fn reject_without_effects(
        installation: &Installation,
        owner: &SettingsService,
        observers: &Observers,
        token: Option<&str>,
        update: Value,
        expected: PublicHostSettingsError,
    ) {
        let before = installation.bytes();
        let before_notices = observers.captured();
        let result = owner.set_public_host_settings_if_current(token, update);
        let error = result.err().expect("the invalid public transaction must be rejected");
        let message = match &expected {
            PublicHostSettingsError::Conflict => "config baseHash mismatch",
            PublicHostSettingsError::Rejected => "configuration update rejected",
            PublicHostSettingsError::KeyUnavailable => "configuration revision key is unavailable",
        };
        assert!(error == expected && error.to_string() == message,
            "public transaction must return only its bounded static error category and message");
        assert!(installation.bytes() == before, "rejected transaction changed settings bytes");
        assert!(observers.captured() == before_notices, "rejected transaction notified an observer");
    }

    #[test]
    fn masked_save_reopens_with_original_agent_skill_and_environment_values() {
        let installation = Installation::seeded(&source_document());
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let before = snapshot(&owner);
        let trusted = owner.get_host_settings_snapshot();
        assert_no_disclosure(&before, &[
            AGENT_A, AGENT_B, SKILL_KEY, SKILL_ENV, LOCAL_ENV, PRIVATE_URL,
            "synthetic-env-budget", &trusted.hash,
            trusted.config_source["rawHash"].as_str().expect("trusted source hash"),
        ]);
        let first = &before.settings["agents"]["list"][0];
        assert!(first["apiKey"] == MASK && first["localService"]["env"]["PLAIN_NAME"] == MASK,
            "agent and local service credentials must be masked");
        assert!(first["callback"] == MASK, "credential URLs must be masked in ordinary fields");
        assert!(first["maxTokens"] == 4096 && first["tokenBudget"] == 128
            && first["passwordFile"] == "local-password.txt",
            "ordinary token budgets and password-file paths must remain visible");
        assert!(before.settings["skills"]["entries"]["demo"]["env"]["maxTokens"] == MASK,
            "environment values must be masked even under normally excluded names");
        assert!(before.settings["agentDefaultModel"] == trusted.settings["agentDefaultModel"]
            && before.settings["localToolPermission"] == "ask"
            && before.settings["pinnedAgentIds"] == json!(["alpha"]),
            "provider/model, permissions and roster presentation must remain usable");

        for marker in [MASK, UPSTREAM_MASK] {
            let base = snapshot(&owner);
            let mut update = base.settings.clone();
            update["userTimeZone"] = json!("Europe/London");
            update["agents"]["list"][0]["apiKey"] = json!(marker);
            update["skills"]["entries"]["demo"]["apiKey"] = json!(marker);
            update["skills"]["entries"]["demo"]["env"]["PLAIN_NAME"] = json!(marker);
            update["skills"]["entries"]["demo"]["enabled"] = json!(false);
            let receipt = commit(&owner, Some(&base.hash), update);
            let reopened = installation.owner();
            let durable = reopened.get_host_settings_snapshot();
            assert!(receipt.trusted_settings == durable.settings,
                "the returned trusted receipt must describe the durable commit");
            assert!(receipt.snapshot == snapshot(&reopened), "public receipt must survive reopening");
            assert!(durable.settings["agents"]["list"][0]["apiKey"] == AGENT_A
                && durable.settings["agents"]["list"][1]["apiKey"] == AGENT_B
                && durable.settings["agents"]["list"][0]["localService"]["env"]["PLAIN_NAME"] == LOCAL_ENV
                && durable.settings["agents"]["list"][0]["callback"] == PRIVATE_URL
                && durable.settings["skills"]["entries"]["demo"]["apiKey"] == SKILL_KEY
                && durable.settings["skills"]["entries"]["demo"]["env"]["PLAIN_NAME"] == SKILL_ENV,
                "a masked save must preserve credentials at their original locations");
            assert!(durable.settings["userTimeZone"] == "Europe/London"
                && durable.settings["skills"]["entries"]["demo"]["enabled"] == false,
                "nonsecret edits must be committed alongside restored credentials");
            let stored = installation.stored();
            assert!(stored["skills"]["entries"]["demo"]["extension"]["keep"] == true
                && stored["unrelatedStoredExtension"] == json!({"keep": [1, 2]}),
                "existing skill merge and unknown-field preservation must remain intact");
            assert_no_disclosure(&receipt.snapshot, &[AGENT_A, AGENT_B, SKILL_KEY, SKILL_ENV, LOCAL_ENV, PRIVATE_URL]);
        }
        assert!(observers.commit_count() == 2, "each valid masked save must notify once");
    }

    #[test]
    fn explicit_replacements_and_skill_empty_string_removal_keep_owner_semantics() {
        let installation = Installation::seeded(&source_document());
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        let receipt = commit(&owner, Some(&base.hash), json!({
            "skills": {"entries": {"demo": {"apiKey": "",
                "env": {"PLAIN_NAME": "", "maxTokens": UPSTREAM_MASK, "NEW": "synthetic-new-env"}}}}
        }));
        let skill = &receipt.trusted_settings["skills"]["entries"]["demo"];
        assert!(skill.get("apiKey").is_none() && skill["env"].get("PLAIN_NAME").is_none(),
            "empty strings must retain the established skill credential removal behavior");
        assert!(skill["env"]["maxTokens"] == "synthetic-env-budget"
            && skill["env"]["NEW"] == "synthetic-new-env",
            "restoration and explicit environment replacement must compose");
        assert!(receipt.trusted_settings["agents"] == source_document()["agents"],
            "omitted agent configuration must remain unchanged");
        let replacement = commit(&owner, Some(&receipt.snapshot.hash), json!({
            "skills": {"entries": {"demo": {"apiKey": "synthetic-replacement"}}}
        }));
        assert!(replacement.trusted_settings["skills"]["entries"]["demo"]["apiKey"] == "synthetic-replacement",
            "explicit replacement must be accepted without a marker");
        assert!(snapshot(&installation.owner()) == replacement.snapshot,
            "replacement and removal must be durable");
        assert!(observers.commit_count() == 2, "only successful updates may notify");
    }

    #[test]
    fn opaque_revisions_are_installation_local_durable_and_domain_separated() {
        let mut source = source_document();
        source["readOnly"] = json!(true);
        let first = Installation::seeded(&source);
        let trusted = first.owner().get_host_settings_snapshot();
        // A readOnly source preserves these exact normalized bytes on read. This
        // makes both internal digests equal, so token inequality proves domains.
        first.write(&trusted.settings);
        let second = Installation::seeded(&trusted.settings);
        let owner = first.owner();
        let internal = owner.get_host_settings_snapshot();
        assert!(internal.config_source["rawHash"] == internal.hash,
            "domain-separation control requires the same internal digest");
        let public = snapshot(&owner);
        let other = snapshot(&second.owner());
        let other_internal = second.owner().get_host_settings_snapshot();
        assert!(other_internal.hash == internal.hash
            && other_internal.config_source["rawHash"] == internal.config_source["rawHash"],
            "independent installations must start with identical configuration bytes");
        assert!(public.hash != public.config_source["rawHash"].as_str().expect("public raw revision"),
            "normalized and raw revisions must use separate MAC domains");
        assert!(public.hash != other.hash && public.config_source["rawHash"] != other.config_source["rawHash"],
            "independent installations must not share a public revision key");
        let key_before = fs::read(first.key_path()).expect("read owned installation key");
        assert!(key_before.len() == 32, "installation revision keys must contain exactly 32 bytes");
        assert!(snapshot(&first.owner()) == public, "a restarted owner must reuse its installation key");
        assert!(fs::read(first.key_path()).expect("reread owned key") == key_before,
            "ordinary reads and restarts must not replace an existing key");
        assert_no_disclosure(&public, &[AGENT_A, AGENT_B, SKILL_KEY, &internal.hash]);
    }

    #[test]
    fn secret_only_change_invalidates_cas_and_legacy_masked_write_restores_current_secret() {
        let installation = Installation::seeded(&source_document());
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let stale = snapshot(&owner);
        let rotated = owner.set_host_settings_if_current(None, json!({
            "skills": {"entries": {"demo": {"apiKey": "synthetic-rotated"}}}
        })).expect("trusted secret-only rotation must succeed");
        let current = snapshot(&owner);
        assert!(stale.settings == current.settings, "secret-only rotation must preserve the visible masked document");
        assert!(stale.hash != current.hash, "secret-only rotation must invalidate the public revision");
        let mut old_masked_document = stale.settings.clone();
        old_masked_document["userTimeZone"] = json!("Asia/Tokyo");
        reject_without_effects(&installation, &owner, &observers, Some(&stale.hash),
            old_masked_document.clone(), PublicHostSettingsError::Conflict);
        let receipt = commit(&owner, None, old_masked_document);
        assert!(receipt.trusted_settings["skills"]["entries"]["demo"]["apiKey"] == "synthetic-rotated",
            "legacy no-token writes must restore against the current secret under the owner lock");
        assert!(receipt.trusted_settings["skills"] == rotated.settings["skills"]
            && receipt.trusted_settings["userTimeZone"] == "Asia/Tokyo",
            "the legacy edit must commit without reverting a concurrent secret rotation");
        assert!(installation.owner().get_host_settings_snapshot().settings == receipt.trusted_settings,
            "the current secret and legacy nonsecret edit must survive reopening");
        assert!(observers.commit_count() == 2, "the failed CAS must add no notification");
    }

    #[test]
    fn public_cas_refuses_both_internal_digests_and_trusted_sha_cas_stays_separate() {
        let installation = Installation::seeded(&source_document());
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let public = snapshot(&owner);
        let trusted = owner.get_host_settings_snapshot();
        let raw_hash = trusted.config_source["rawHash"].as_str().expect("trusted source digest");
        for digest in [trusted.hash.as_str(), raw_hash] {
            reject_without_effects(&installation, &owner, &observers, Some(digest),
                json!({"userTimeZone": "Europe/London"}), PublicHostSettingsError::Conflict);
        }
        let before = installation.bytes();
        let wrong_boundary = owner.set_host_settings_if_current(Some(&public.hash),
            json!({"userTimeZone": "Europe/London"}));
        assert!(matches!(wrong_boundary, Err(HostSettingsUpdateError::Conflict { expected, actual })
            if expected == trusted.hash && actual == public.hash),
            "trusted SHA CAS must retain its existing typed expected/actual error semantics");
        assert!(installation.bytes() == before && observers.captured().is_empty(),
            "cross-boundary digest refusal must not change settings or observers");
        let accepted = commit(&owner, Some(&public.hash), json!({"userTimeZone": "Europe/London"}));
        let latest = owner.get_host_settings_snapshot();
        let trusted_commit = owner.set_host_settings_if_current(Some(&latest.hash),
            json!({"userTimeZone": "Asia/Tokyo"})).expect("trusted SHA CAS remains supported");
        assert!(trusted_commit.settings["userTimeZone"] == "Asia/Tokyo"
            && accepted.snapshot.settings["userTimeZone"] == "Europe/London",
            "public token CAS and trusted SHA CAS must both retain valid success controls");
        assert!(observers.commit_count() == 2, "only the two valid boundary-specific commits may notify");
    }

    #[test]
    fn simultaneous_writers_with_one_public_token_produce_exactly_one_commit() {
        let installation = Installation::seeded(&source_document());
        let owner = Arc::new(installation.owner());
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        let barrier = Arc::new(Barrier::new(3));
        let (sender, receiver) = mpsc::channel();
        let mut workers = Vec::new();
        for zone in ["Europe/London", "Asia/Tokyo"] {
            let owner = Arc::clone(&owner);
            let barrier = Arc::clone(&barrier);
            let sender = sender.clone();
            let token = base.hash.clone();
            let mut update = base.settings.clone();
            update["userTimeZone"] = json!(zone);
            workers.push(std::thread::spawn(move || {
                barrier.wait();
                let result = owner.set_public_host_settings_if_current(Some(&token), update);
                let _ = sender.send((zone, result));
            }));
        }
        drop(sender);
        barrier.wait();
        let mut successes = Vec::new();
        let mut conflicts = 0;
        for _ in 0..2 {
            let (zone, result) = receiver.recv_timeout(Duration::from_secs(10))
                .expect("both contending owner transactions must complete");
            match result {
                Ok(receipt) => successes.push((zone, receipt)),
                Err(PublicHostSettingsError::Conflict) => conflicts += 1,
                Err(_) => panic!("contention must not become a key or validation failure"),
            }
        }
        for worker in workers {
            worker.join().unwrap_or_else(|_| panic!("contending settings writer panicked"));
        }
        assert!(successes.len() == 1 && conflicts == 1, "a public revision may authorize only one concurrent commit");
        let (zone, winner) = successes.pop().expect("one successful transaction");
        assert!(winner.trusted_settings["userTimeZone"] == zone
            && installation.owner().get_host_settings_snapshot().settings == winner.trusted_settings,
            "durable state must equal the single winning receipt");
        assert!(winner.trusted_settings["skills"]["entries"]["demo"]["apiKey"] == SKILL_KEY,
            "concurrent masked writes must preserve the current secret");
        assert!(observers.commit_count() == 1 && observers.captured().len() == 2,
            "the losing CAS must emit neither a settings nor a timezone notification");
    }

    #[test]
    fn actual_agent_array_reordering_keeps_identity_and_refuses_secret_borrowing() {
        let installation = Installation::seeded(&source_document());
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        let mut update = base.settings.clone();
        update["agents"]["list"].as_array_mut().expect("agent array").swap(0, 1);
        update["agents"]["list"][0]["name"] = json!("Beta edited");
        update["agents"]["list"][1]["apiKey"] = json!(UPSTREAM_MASK);
        let reordered = commit(&owner, Some(&base.hash), update);
        let entries = &reordered.trusted_settings["agents"]["list"];
        assert!(entries[0]["id"] == "beta" && entries[0]["apiKey"] == AGENT_B
            && entries[1]["id"] == "alpha" && entries[1]["apiKey"] == AGENT_A,
            "reordered agent credentials must follow literal identities, never old positions");
        assert!(installation.owner().get_host_settings_snapshot().settings == reordered.trusted_settings,
            "identity-preserving reorder must be durable");

        let mut forged = reordered.snapshot.settings.clone();
        forged["agents"]["list"][0]["id"] = json!("gamma");
        reject_without_effects(&installation, &owner, &observers, Some(&reordered.snapshot.hash),
            forged.clone(), PublicHostSettingsError::Rejected);
        for matched_secret in [MASK, "synthetic-explicit-beta"] {
            let reserved_index = json!({"agents": {"ownership": "explicit", "list": [
                {"name": "unidentified", "apiKey": MASK},
                {"id": "beta", "apiKey": matched_secret}
            ]}});
            reject_without_effects(&installation, &owner, &observers, Some(&reordered.snapshot.hash),
                reserved_index, PublicHostSettingsError::Rejected);
        }
        forged["agents"]["list"][0]["apiKey"] = json!("synthetic-gamma");
        let explicit = commit(&owner, Some(&reordered.snapshot.hash), forged);
        assert!(explicit.trusted_settings["agents"]["list"][0]["id"] == "gamma"
            && explicit.trusted_settings["agents"]["list"][0]["apiKey"] == "synthetic-gamma"
            && explicit.trusted_settings["agents"]["list"][1]["apiKey"] == AGENT_A,
            "a new identity may commit an explicit credential without borrowing another entry's value");
        assert!(observers.commit_count() == 2, "identity refusals must not add notifications");
    }

    #[test]
    fn legacy_duplicate_missing_and_environment_ids_keep_only_bounded_positional_fallback() {
        for ids in [
            [Some("duplicate"), Some("duplicate")],
            [None, None],
            [Some("${FIRST}"), Some("${SECOND}")],
            [Some("$${FIRST}"), Some("$${SECOND}")],
        ] {
            let mut original_items = vec![
                json!({"apiKey": "synthetic-first"}),
                json!({"apiKey": "synthetic-second"}),
            ];
            for (item, id) in original_items.iter_mut().zip(ids) {
                if let Some(id) = id {
                    item["id"] = json!(id);
                }
            }
            let mut source = source_document();
            source["agents"]["extensionItems"] = json!(original_items);
            let installation = Installation::seeded(&source);
            let owner = installation.owner();
            let observers = Observers::new(&owner);
            let base = snapshot(&owner);
            let mut update = base.settings.clone();
            update["agents"]["extensionItems"].as_array_mut().expect("extension array").swap(0, 1);
            let accepted = commit(&owner, Some(&base.hash), update);
            let items = &accepted.trusted_settings["agents"]["extensionItems"];
            assert!(items[0]["apiKey"] == "synthetic-first" && items[1]["apiKey"] == "synthetic-second",
                "nonunique or environment-authored ids must retain positional secret restoration");
            assert!(items[0].get("id") == original_items[1].get("id")
                && items[1].get("id") == original_items[0].get("id"),
                "fallback must not silently rewrite the submitted identity expression");
            let mut orphan = accepted.snapshot.settings.clone();
            orphan["agents"]["extensionItems"].as_array_mut().expect("extension array")
                .push(json!({"apiKey": MASK}));
            reject_without_effects(&installation, &owner, &observers, Some(&accepted.snapshot.hash),
                orphan, PublicHostSettingsError::Rejected);
            assert!(observers.commit_count() == 1, "an appended fallback orphan must not become a commit");
        }
    }

    #[test]
    fn masked_secret_refs_require_the_same_explicit_source_and_provider() {
        let mut source = source_document();
        source["agents"]["list"][0]["apiKey"] = json!({
            "source": "env", "provider": "fixture", "id": "SYNTHETIC_REF_A"
        });
        let installation = Installation::seeded(&source);
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        assert!(base.settings["agents"]["list"][0]["apiKey"] == json!({
            "source": "env", "provider": "fixture", "id": MASK
        }), "a public SecretRef must preserve visible backend identity and conceal its id");
        let unchanged = commit(&owner, Some(&base.hash), base.settings.clone());
        assert!(unchanged.trusted_settings["agents"]["list"][0]["apiKey"]
            == source["agents"]["list"][0]["apiKey"], "unchanged SecretRefs must round-trip through the real owner");
        for field in ["source", "provider"] {
            for replacement in [Some(json!("different")), None, Some(json!(UPSTREAM_MASK))] {
                let mut update = unchanged.snapshot.settings.clone();
                let reference = update["agents"]["list"][0]["apiKey"].as_object_mut().expect("reference object");
                if let Some(value) = replacement {
                    reference.insert(field.into(), value);
                } else {
                    reference.remove(field);
                }
                reject_without_effects(&installation, &owner, &observers, Some(&unchanged.snapshot.hash),
                    update, PublicHostSettingsError::Rejected);
            }
        }
        let mut explicit = unchanged.snapshot.settings.clone();
        explicit["agents"]["list"][0]["apiKey"] = json!({
            "source": "file", "provider": "replacement", "id": "SYNTHETIC_REF_B"
        });
        let replacement = commit(&owner, Some(&unchanged.snapshot.hash), explicit);
        assert!(replacement.trusted_settings["agents"]["list"][0]["apiKey"]["id"] == "SYNTHETIC_REF_B"
            && replacement.snapshot.settings["agents"]["list"][0]["apiKey"]["source"] == "file"
            && replacement.snapshot.settings["agents"]["list"][0]["apiKey"]["id"] == MASK,
            "changing a SecretRef backend requires and accepts an explicit replacement id");
        assert!(snapshot(&installation.owner()) == replacement.snapshot && observers.commit_count() == 2,
            "only the unchanged and explicit-replacement SecretRefs may commit");
    }

    #[test]
    fn credential_urls_in_secret_ref_identity_require_explicit_repair() {
        for field in ["source", "provider"] {
            let mut source = source_document();
            source["agents"]["list"][0]["apiKey"] = json!({
                "source": "env", "provider": "fixture", "id": "SYNTHETIC_REF_A"
            });
            source["agents"]["list"][0]["apiKey"][field] = json!(PRIVATE_URL);
            let installation = Installation::seeded(&source);
            let owner = installation.owner();
            let observers = Observers::new(&owner);
            let base = snapshot(&owner);
            assert!(base.settings["agents"]["list"][0]["apiKey"][field] == MASK,
                "a credential-bearing SecretRef identity must not be disclosed");
            assert_no_disclosure(&base, &[PRIVATE_URL, "SYNTHETIC_REF_A"]);
            reject_without_effects(&installation, &owner, &observers, Some(&base.hash),
                base.settings.clone(), PublicHostSettingsError::Rejected);
            let mut original_backend = base.settings.clone();
            original_backend["agents"]["list"][0]["apiKey"][field] = json!(PRIVATE_URL);
            reject_without_effects(&installation, &owner, &observers, Some(&base.hash),
                original_backend, PublicHostSettingsError::Rejected);
            let mut repaired = base.settings.clone();
            repaired["agents"]["list"][0]["apiKey"] = json!({
                "source": "env", "provider": "repaired", "id": "EXPLICIT_REPAIRED_ID"
            });
            let receipt = commit(&owner, Some(&base.hash), repaired);
            assert!(receipt.trusted_settings["agents"]["list"][0]["apiKey"]["id"] == "EXPLICIT_REPAIRED_ID"
                && observers.commit_count() == 1, "an explicit safe backend and id must repair the configuration");
        }
    }

    #[test]
    fn wrong_missing_container_and_nonobject_markers_never_write_or_notify() {
        let installation = Installation::seeded(&source_document());
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        for marker in [MASK, UPSTREAM_MASK] {
            for update in [
                json!({"agents": {"list": [{"id": "alpha", "name": marker}]}}),
                json!({"skills": {"entries": {"new-entry": {"apiKey": marker}}}}),
                json!({"agents": {"list": [{"id": "alpha", "localService": {"env": marker}}]}}),
                json!({"ignoredExtension": [{"missingSecret": marker}]}),
                json!(marker),
                json!([{"missingSecret": marker}]),
                json!([marker]),
            ] {
                reject_without_effects(&installation, &owner, &observers, Some(&base.hash),
                    update, PublicHostSettingsError::Rejected);
            }
        }
        // Nonobject no-ops are an existing compatibility behavior. The marker
        // refusal above must not accidentally turn every nonobject into an error.
        let bytes = installation.bytes();
        for update in [Value::Null, json!(7), json!(["ordinary"])] {
            let receipt = commit(&owner, Some(&base.hash), update);
            assert!(receipt.snapshot == base && installation.bytes() == bytes,
                "ordinary nonobject updates must retain normalized no-op compatibility");
        }
        assert!(observers.captured().is_empty(), "neither rejected markers nor compatibility no-ops may notify");
        let mut valid = base.settings.clone();
        valid["userTimeZone"] = json!("Europe/London");
        commit(&owner, Some(&base.hash), valid);
        assert!(observers.commit_count() == 1, "the same owner and token must accept a valid masked control");
    }

    #[test]
    fn a_marker_in_the_original_cannot_survive_restoration_as_a_credential() {
        for marker in [MASK, UPSTREAM_MASK] {
            let mut source = source_document();
            source["skills"]["entries"]["demo"]["apiKey"] = json!(marker);
            let installation = Installation::seeded(&source);
            let owner = installation.owner();
            let observers = Observers::new(&owner);
            let base = snapshot(&owner);
            reject_without_effects(&installation, &owner, &observers, Some(&base.hash),
                base.settings.clone(), PublicHostSettingsError::Rejected);
            let mut repaired = base.settings.clone();
            repaired["skills"]["entries"]["demo"]["apiKey"] = json!("synthetic-repaired");
            let receipt = commit(&owner, Some(&base.hash), repaired);
            assert!(receipt.trusted_settings["skills"]["entries"]["demo"]["apiKey"] == "synthetic-repaired"
                && observers.commit_count() == 1, "an explicit credential must repair a residual original marker");
        }
    }

    #[test]
    fn public_receipt_a_stays_a_when_a_listener_commits_b_and_notifications_remain_fifo() {
        let installation = Installation::seeded(&source_document());
        let owner = Arc::new(installation.owner());
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        let entered = Arc::new(AtomicBool::new(false));
        let entered_in_listener = Arc::clone(&entered);
        let captured = Arc::new(Mutex::new(None));
        let sink = Arc::clone(&captured);
        let weak_owner = Arc::downgrade(&owner);
        let _reentrant = owner.subscribe_to_changes(Arc::new(move |_| {
            if entered_in_listener.swap(true, Ordering::SeqCst) {
                return;
            }
            let owner = weak_owner.upgrade().expect("the transaction owner remains live");
            let public_a = snapshot(&owner);
            let trusted_a = owner.get_host_settings_snapshot();
            let receipt_b = commit(&owner, Some(&public_a.hash), json!({
                "userTimeZone": "Asia/Tokyo", "inferenceProvider": "codex",
                "skills": {"entries": {"demo": {"apiKey": "synthetic-reentrant-b"}}},
                "featureFlagOverrides": {"receipt_b": true}
            }));
            *sink.lock().expect("capture reentrant receipts") = Some((public_a, trusted_a, receipt_b));
        }));
        let writer = Arc::clone(&owner);
        let (sender, receiver) = mpsc::channel();
        let worker = std::thread::spawn(move || {
            let result = writer.set_public_host_settings_if_current(Some(&base.hash), json!({
                "userTimeZone": "Europe/London", "inferenceProvider": "claude-code",
                "skills": {"entries": {"demo": {"apiKey": "synthetic-reentrant-a"}}},
                "featureFlagOverrides": {"receipt_a": true}
            }));
            let _ = sender.send(result);
        });
        let result = receiver.recv_timeout(Duration::from_secs(10))
            .expect("reentrant reads and commits must complete outside the writer mutex");
        let receipt_a = match result {
            Ok(receipt) => receipt,
            Err(_) => panic!("the original public transaction must succeed"),
        };
        worker.join().unwrap_or_else(|_| panic!("the reentrant settings writer panicked"));
        let (public_a, trusted_a, receipt_b) = captured.lock().expect("read reentrant receipts")
            .take().expect("the listener must have captured both committed generations");
        assert!(receipt_a.snapshot == public_a && receipt_a.trusted_settings == trusted_a.settings,
            "the public and trusted receipt must describe A even after its listener commits B");
        assert!(receipt_a.trusted_settings["skills"]["entries"]["demo"]["apiKey"] == "synthetic-reentrant-a"
            && receipt_b.trusted_settings["skills"]["entries"]["demo"]["apiKey"] == "synthetic-reentrant-b",
            "trusted post-commit consumers must receive the credential generation of their own receipt");
        assert!(receipt_a.snapshot.hash != receipt_b.snapshot.hash
            && receipt_a.snapshot.config_source["rawHash"] != receipt_b.snapshot.config_source["rawHash"],
            "each returned receipt must retain its own opaque normalized and source revisions");
        assert!(snapshot(&owner) == receipt_b.snapshot
            && installation.owner().get_host_settings_snapshot().settings == receipt_b.trusted_settings,
            "latest-state readers and reopened owners must observe B");
        assert_no_disclosure(&receipt_a.snapshot, &["synthetic-reentrant-a", "synthetic-reentrant-b"]);
        assert_no_disclosure(&receipt_b.snapshot, &["synthetic-reentrant-a", "synthetic-reentrant-b"]);
        let notices = observers.captured();
        assert!(notices.len() == 6, "both commits must drain every subscribed observer exactly once");
        assert!(notices[0] == Notice::TimeZone(Some("Europe/London".into()))
            && notices[1] == Notice::Features(BTreeMap::from([("receipt_a".into(), true)]))
            && matches!(&notices[2], Notice::Changes(fields) if fields.iter().any(|field| field == "skills"))
            && notices[3] == Notice::TimeZone(Some("Asia/Tokyo".into()))
            && notices[4] == Notice::Features(BTreeMap::from([("receipt_b".into(), true)]))
            && matches!(&notices[5], Notice::Changes(fields) if fields.iter().any(|field| field == "skills")),
            "reentrant callbacks must preserve A's complete notification batch before B's batch");
    }

    #[test]
    fn corrupt_or_nonregular_keys_fail_before_public_read_normalization_and_writes() {
        #[derive(Clone, Copy)]
        enum KeyFault { Length(usize), Directory }
        for fault in [KeyFault::Length(0), KeyFault::Length(31), KeyFault::Length(33), KeyFault::Directory] {
            let mut source = source_document();
            source["notifications"] = json!({"isEnabled": true, "legacy": "keep-until-valid-key"});
            let installation = Installation::seeded(&source);
            let owner = installation.owner();
            let authored = installation.bytes();
            let healthy = snapshot(&owner);
            assert!(installation.bytes() != authored,
                "the healthy key control must prove this source normally needs read normalization");
            assert!(fs::read(installation.key_path()).expect("read healthy fixture key").len() == 32,
                "the corruption control must begin with a valid installation key");
            installation.write(&source);
            match fault {
                KeyFault::Length(length) => {
                    fs::write(installation.key_path(), vec![0x54_u8; length]).expect("install owned corrupt key fixture");
                }
                KeyFault::Directory => {
                    fs::remove_file(installation.key_path()).expect("remove only the owned fixture key");
                    fs::create_dir(installation.key_path()).expect("install nonregular fixture key");
                    fs::write(installation.key_path().join("owned-marker"), b"retain-directory")
                        .expect("seed nonregular key marker");
                }
            }
            let observers = Observers::new(&owner);
            let before = installation.bytes();
            assert!(owner.get_public_host_settings_snapshot().err() == Some(PublicHostSettingsError::KeyUnavailable),
                "public reads must fail closed when their installation key becomes unusable");
            assert!(installation.bytes() == before && observers.captured().is_empty(),
                "key failure must precede even read-time normalization and all notifications");
            reject_without_effects(&installation, &owner, &observers, Some(&healthy.hash),
                json!({"userTimeZone": "Europe/London", "featureFlagOverrides": {"key_control": true}}),
                PublicHostSettingsError::KeyUnavailable);
            match fault {
                KeyFault::Length(length) => {
                    assert!(fs::read(installation.key_path()).expect("reread corrupt fixture key") == vec![0x54_u8; length],
                        "an invalid existing key must never rotate or be repaired implicitly");
                }
                KeyFault::Directory => {
                    assert!(installation.key_path().is_dir()
                        && fs::read(installation.key_path().join("owned-marker")).expect("read nonregular key marker")
                            == b"retain-directory",
                        "nonregular key material must remain untouched");
                }
            }
            assert!(fs::read_dir(&installation.root).expect("inspect owned key fixture").count() == 2,
                "a failed key load must not leave a new key candidate or settings staging file");
        }
        let control = Installation::seeded(&source_document());
        let owner = control.owner();
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        commit(&owner, Some(&base.hash), json!({
            "userTimeZone": "Europe/London", "featureFlagOverrides": {"key_control": true}
        }));
        assert!(observers.commit_count() == 1 && observers.captured().len() == 3,
            "a healthy key must permit the same settings, timezone and flag notifications");
    }

    #[test]
    fn key_creation_io_failure_cannot_create_settings_or_notify() {
        let installation = Installation::new();
        let blocked = installation.root.join("blocked-parent");
        fs::write(&blocked, b"owned-nondirectory").expect("create deterministic key-parent fault");
        let owner = SettingsService::new(blocked.join("settings.json"));
        let observers = Observers::new(&owner);
        assert!(owner.get_public_host_settings_snapshot().err() == Some(PublicHostSettingsError::KeyUnavailable),
            "a failed installation-key parent must reject public reads");
        assert!(owner.set_public_host_settings_if_current(None, json!({"userTimeZone": "UTC"})).err()
            == Some(PublicHostSettingsError::KeyUnavailable),
            "a failed installation-key parent must reject public writes before settings IO");
        assert!(fs::read(&blocked).expect("read blocked-parent marker") == b"owned-nondirectory"
            && fs::read_dir(&installation.root).expect("inspect blocked installation").count() == 1
            && observers.captured().is_empty(),
            "key creation failure must preserve the obstruction and every observer");

        let control = SettingsService::new(installation.root.join("valid-parent").join("settings.json"));
        let control_observers = Observers::new(&control);
        let base = snapshot(&control);
        let receipt = commit(&control, Some(&base.hash), json!({"userTimeZone": "UTC"}));
        assert!(receipt.trusted_settings["userTimeZone"] == "UTC"
            && control_observers.commit_count() == 1,
            "a creatable private installation must permit the same first-use transaction");
    }

    #[test]
    fn invalid_public_sources_remain_invalid_until_explicit_fixture_repair() {
        let invalid_sources: [&[u8]; 5] = [
            b"{broken settings source", b"\xff\xfe", b"[1,2,3]",
            br#"{"version":2,"keep":"invalid-version"}"#,
            br#"{"keep":"missing-version"}"#,
        ];
        for original in invalid_sources {
            let installation = Installation::new();
            fs::write(installation.settings_path(), original).expect("seed owned invalid source");
            let owner = installation.owner();
            let observers = Observers::new(&owner);
            let public = snapshot(&owner);
            assert!(public.config_source["exists"] == true && public.config_source["valid"] == false,
                "invalid input must retain invalid source provenance in public snapshots");
            assert!(public.config_source["rawHash"] != sha256_hex(original),
                "even invalid source digests must remain opaque at the public boundary");
            assert_token(public.config_source["rawHash"].as_str().expect("existing invalid source has a public raw revision"));
            assert!(installation.bytes() == original && observers.captured().is_empty(),
                "public default projection must not replace an invalid source");
            reject_without_effects(&installation, &owner, &observers, Some(&public.hash),
                json!({"userTimeZone": "Europe/London", "featureFlagOverrides": {"source_control": true}}),
                PublicHostSettingsError::Rejected);
            reject_without_effects(&installation, &owner, &observers, None,
                json!({"userTimeZone": "Europe/London"}), PublicHostSettingsError::Rejected);
            installation.write(&source_document());
            let repaired = snapshot(&owner);
            let receipt = commit(&owner, Some(&repaired.hash), json!({
                "userTimeZone": "Europe/London", "featureFlagOverrides": {"source_control": true}
            }));
            assert!(receipt.trusted_settings["userTimeZone"] == "Europe/London"
                && observers.commit_count() == 1 && observers.captured().len() == 3,
                "the same owner must accept valid source after explicit fixture repair");
        }
    }

    #[test]
    fn unreadable_settings_directory_keeps_null_public_raw_revision_and_no_commit() {
        let installation = Installation::new();
        fs::create_dir(installation.settings_path()).expect("create owned unreadable settings source");
        let marker = installation.settings_path().join("owned-marker");
        fs::write(&marker, b"retain-settings-directory").expect("seed unreadable source marker");
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let public = snapshot(&owner);
        assert!(public.config_source["exists"] == true && public.config_source["valid"] == false
            && public.config_source["rawHash"].is_null(),
            "an unreadable source must preserve null raw revision rather than inventing a digest");
        assert_token(&public.hash);
        assert!(owner.set_public_host_settings_if_current(Some(&public.hash),
            json!({"userTimeZone": "Europe/London"})).err() == Some(PublicHostSettingsError::Rejected),
            "an unreadable source must not acknowledge a replacement commit");
        assert!(installation.settings_path().is_dir()
            && fs::read(&marker).expect("read unchanged source marker") == b"retain-settings-directory"
            && fs::read_dir(&installation.root).expect("inspect unreadable installation").count() == 2
            && observers.captured().is_empty(),
            "unreadable source rejection must preserve owned files and observers");
        fs::remove_file(marker).expect("remove only the owned source marker");
        fs::remove_dir(installation.settings_path()).expect("repair owned source shape");
        installation.write(&source_document());
        let repaired = snapshot(&owner);
        commit(&owner, Some(&repaired.hash), json!({"userTimeZone": "Europe/London"}));
        assert!(observers.commit_count() == 1, "a regular valid source must pass the same owner path");
    }

    #[test]
    fn public_readonly_guard_prevents_normalization_writes_and_all_update_observers() {
        let mut source = source_document();
        source["readOnly"] = json!(true);
        source["notifications"] = json!({"isEnabled": true, "legacy": "retain"});
        let installation = Installation::seeded(&source);
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let original = installation.bytes();
        let public = snapshot(&owner);
        assert!(public.settings["readOnly"] == true && public.settings["notifications"]["isEnabled"] == false,
            "readOnly public projection must preserve policy and existing normalized display behavior");
        assert!(installation.bytes() == original, "readOnly display normalization must not touch source bytes");
        for token in [Some(public.hash.as_str()), None] {
            reject_without_effects(&installation, &owner, &observers, token, json!({
                "readOnly": false, "userTimeZone": "Europe/London",
                "skills": {"entries": {"demo": {"apiKey": "synthetic-refused"}}},
                "featureFlagOverrides": {"readonly_control": true}
            }), PublicHostSettingsError::Rejected);
        }
        source.as_object_mut().expect("source object").remove("readOnly");
        installation.write(&source);
        let writable = snapshot(&owner);
        commit(&owner, Some(&writable.hash), json!({
            "userTimeZone": "Europe/London", "featureFlagOverrides": {"readonly_control": true}
        }));
        assert!(observers.commit_count() == 1 && observers.captured().len() == 3,
            "the same owner must accept the valid writable control and notify all intended observers");
    }

    #[cfg(unix)]
    #[test]
    fn persistence_io_failure_after_valid_read_and_key_leaves_bytes_and_observers_unchanged() {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;

        let installation = Installation::new();
        let directory = CString::new(installation.root.as_os_str().as_bytes())
            .expect("owned fixture path has no nul byte");
        // Query this filesystem's real component bound. Reading the longest
        // legal source succeeds; Settings' longer staging component cannot.
        let maximum = unsafe { libc::pathconf(directory.as_ptr(), libc::_PC_NAME_MAX) };
        assert!(maximum > 0 && maximum <= 1024, "the IO fixture requires a bounded filesystem component limit");
        let path = installation.root.join("s".repeat(maximum as usize));
        let original = serde_json::to_vec(&source_document()).expect("serialize IO control source");
        fs::write(&path, &original).expect("the longest legal source component must be writable");
        let owner = SettingsService::new(&path);
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        assert!(base.config_source["valid"] == true
            && fs::read(&path).expect("read IO control source") == original,
            "the failing writer must begin with a valid readable and already-normalized source");
        let key = fs::read(installation.key_path()).expect("read healthy IO fixture key");
        assert!(key.len() == 32, "persistence refusal must not be caused by an unavailable revision key");
        let update = json!({"userTimeZone": "Europe/London", "featureFlagOverrides": {"io_control": true}});
        assert!(owner.set_public_host_settings_if_current(Some(&base.hash), update.clone()).err()
            == Some(PublicHostSettingsError::Rejected), "a failed settings publication must return the bounded rejection");
        assert!(fs::read(&path).expect("read source after refused publication") == original
            && fs::read(installation.key_path()).expect("read unchanged IO fixture key") == key
            && observers.captured().is_empty()
            && fs::read_dir(&installation.root).expect("inspect failed staging directory").count() == 2,
            "failed persistence must retain source/key bytes, leave no staging entry and notify nobody");

        let control_path = installation.root.join("short-settings.json");
        fs::write(&control_path, &original).expect("seed short-path IO control");
        let control = SettingsService::new(&control_path);
        let control_observers = Observers::new(&control);
        let control_base = snapshot(&control);
        assert!(control_base.hash == base.hash,
            "the successful control must use identical normalized settings and the same healthy key");
        let receipt = commit(&control, Some(&control_base.hash), update);
        assert!(receipt.trusted_settings["userTimeZone"] == "Europe/London"
            && control_observers.commit_count() == 1 && control_observers.captured().len() == 3,
            "the identical candidate must succeed when only its staging destination can be created");
    }

    #[test]
    fn masked_skill_save_preserves_latest_imported_bytes_after_stale_snapshot() {
        let mut source = source_document();
        let installation = Installation::seeded(&source);
        let owner = installation.owner();
        let initial = snapshot(&owner);
        let mut update = initial.settings.clone();
        update["userTimeZone"] = json!("Europe/London");
        source["skills"]["entries"]["demo"]["apiKey"] = json!("  synthetic imported key  ");
        source["skills"]["entries"]["demo"]["env"] = json!({
            "REGION":" \t\n ", "PRESERVE_EMPTY":"", "MULTILINE":"\nline one\nline two\n"
        });
        installation.write(&source);
        update["skills"]["entries"]["demo"]["env"] = json!({
            "REGION":MASK, "PRESERVE_EMPTY":UPSTREAM_MASK, "MULTILINE":MASK
        });
        let observers = Observers::new(&owner);
        reject_without_effects(&installation, &owner, &observers, Some(&initial.hash),
            update.clone(), PublicHostSettingsError::Conflict);
        let receipt = commit(&owner, None, update);
        let reopened = installation.owner().get_host_settings_snapshot();
        for actual in [&receipt.trusted_settings, &reopened.settings, &installation.stored()] {
            assert!(actual["skills"] == source["skills"],
                "the masked save must preserve current stored credential bytes exactly");
            assert_eq!(actual["userTimeZone"], "Europe/London");
        }
        assert_eq!(observers.commit_count(), 1);
    }

    #[test]
    fn public_writes_preserve_include_ownership_and_specialized_writer_compatibility() {
        let mut source = source_document();
        source["agents"] = json!({"entries": {"$include": "agents-fixture.json"}});
        let installation = Installation::seeded(&source);
        let owner = installation.owner();
        let observers = Observers::new(&owner);
        let base = snapshot(&owner);
        reject_without_effects(&installation, &owner, &observers, Some(&base.hash),
            json!({"agents": {"entries": {"local": {}}}}), PublicHostSettingsError::Rejected);
        assert!(owner.set_user_time_zone(Some("Europe/London")).expect("specialized timezone control"),
            "include ownership must preserve unrelated specialized Settings writers");
        assert!(installation.stored()["agents"] == source["agents"] && observers.commit_count() == 1,
            "the specialized update must keep include-owned roster data intact");

        let target = Installation::seeded(&source_document());
        let target_owner = target.owner();
        let target_observers = Observers::new(&target_owner);
        let target_base = snapshot(&target_owner);
        reject_without_effects(&target, &target_owner, &target_observers, Some(&target_base.hash),
            json!({"agents": {"entries": {"$include": "agents-fixture.json"}}}), PublicHostSettingsError::Rejected);
        commit(&target_owner, Some(&target_base.hash), json!({"userTimeZone": "Europe/London"}));
        assert!(target_observers.commit_count() == 1,
            "refusing an include-owned target must retain ordinary public update compatibility");
    }
}

