use mahayana_host_runtime::extensions::plugins::hook_runner::{
    HookName, PluginHookRunner, ProviderExtensionBridge,
};
use mahayana_host_runtime::extensions::plugins::installed_index_store::{
    InstalledPluginIndexRecord, InstalledPluginIndexStore, INSTALLED_PLUGIN_INDEX_VERSION,
    INSTALLED_PLUGIN_INDEX_WARNING,
};
use mahayana_host_runtime::extensions::plugins::manifest_doctor::{
    check_plugin_manifest_health, DiagnosticSeverity, PluginDoctorReport,
};
use serde_json::json;
use std::time::Duration;

#[test]
fn test_ocrm_plugins_installed_index_store() {
    let store = InstalledPluginIndexStore::new();
    let snapshot = store.get_snapshot();
    assert_eq!(snapshot.version, INSTALLED_PLUGIN_INDEX_VERSION);
    assert_eq!(snapshot.warning, INSTALLED_PLUGIN_INDEX_WARNING);

    // Insert record
    store.set_record(InstalledPluginIndexRecord {
        plugin_id: "plugin-alpha".to_string(),
        version: "1.0.0".to_string(),
        package_name: Some("@openclaw/alpha".to_string()),
        install_path: "/var/plugins/alpha".to_string(),
        source_type: "clawhub".to_string(),
        signature: Some("sig-xyz".to_string()),
        enabled: true,
    });

    store.set_record(InstalledPluginIndexRecord {
        plugin_id: "plugin-beta".to_string(),
        version: "2.1.0".to_string(),
        package_name: None,
        install_path: "/var/plugins/beta".to_string(),
        source_type: "npm".to_string(),
        signature: None,
        enabled: false,
    });

    let rec = store.get_record("plugin-alpha").expect("Found record");
    assert_eq!(rec.version, "1.0.0");
    assert!(rec.enabled);

    // Filter enabled
    let enabled = store.list_enabled_records();
    assert_eq!(enabled.len(), 1);
    assert_eq!(enabled[0].plugin_id, "plugin-alpha");

    // Toggle enabled
    assert!(store.set_enabled("plugin-beta", true));
    assert_eq!(store.list_enabled_records().len(), 2);

    // Remove record
    let removed = store.remove_record("plugin-alpha").expect("Removed");
    assert_eq!(removed.plugin_id, "plugin-alpha");
    assert!(store.get_record("plugin-alpha").is_none());
}

#[test]
fn test_ocrm_plugins_manifest_doctor() {
    let mut report = PluginDoctorReport::new();

    // 1. Missing manifest
    let diags1 = check_plugin_manifest_health("broken-plugin", false, None, false);
    for d in diags1 {
        report.add_diagnostic(d);
    }
    assert!(report.has_errors());
    assert_eq!(report.errors().len(), 1);
    assert_eq!(report.errors()[0].code, "MISSING_MANIFEST");

    // 2. Valid manifest with deprecated channel config warning
    let diags2 = check_plugin_manifest_health("legacy-channel-plugin", true, Some("main.js"), true);
    assert_eq!(diags2.len(), 1);
    assert_eq!(diags2[0].code, "DEPRECATED_CHANNEL_CONFIG");
    assert_eq!(diags2[0].severity, DiagnosticSeverity::Warning);

    // 3. Clean plugin
    let diags3 = check_plugin_manifest_health("clean-plugin", true, Some("index.js"), false);
    assert!(diags3.is_empty());
}

#[test]
fn test_ocrm_plugins_hook_runner_and_provider_bridge() {
    let hook_runner = PluginHookRunner::new();

    // Register hooks with priority: higher priority runs first
    // priority 10 (runs second)
    hook_runner.register_hook(HookName::BeforeReply, "plugin-mid", 10, |payload| {
        let mut obj = payload.as_object().unwrap().clone();
        let current_text = obj.get("text").and_then(|t| t.as_str()).unwrap_or("");
        obj.insert("text".to_string(), json!(format!("{current_text} -> mid")));
        Ok(Some(json!(obj)))
    });

    // priority 100 (runs first)
    hook_runner.register_hook(HookName::BeforeReply, "plugin-first", 100, |payload| {
        let mut obj = payload.as_object().unwrap().clone();
        let current_text = obj.get("text").and_then(|t| t.as_str()).unwrap_or("");
        obj.insert("text".to_string(), json!(format!("{current_text} -> first")));
        Ok(Some(json!(obj)))
    });

    // priority 5 (runs third, returns Err but shouldn't abort)
    hook_runner.register_hook(HookName::BeforeReply, "plugin-faulty", 5, |_| {
        Err("Transient hook execution failure".to_string())
    });

    let initial = json!({"text": "start"});
    let final_payload = hook_runner
        .run_hooks(HookName::BeforeReply, &initial)
        .expect("Hook pipeline executes with fail-soft");

    assert_eq!(
        final_payload.get("text").unwrap().as_str().unwrap(),
        "start -> first -> mid"
    );

    // Provider extension bridge
    let bridge = ProviderExtensionBridge::new();
    bridge.register_provider("speech-eleven", |op, params| match op {
        "synthesize" => {
            let text = params.get("text").and_then(|t| t.as_str()).unwrap_or("");
            Ok(json!({"audio_len": text.len(), "sample_rate": 24000}))
        }
        _ => Err("Unsupported operation".to_string()),
    });

    let res = bridge
        .invoke_provider(
            "speech-eleven",
            "synthesize",
            &json!({"text": "Hello world"}),
            Duration::from_secs(1),
        )
        .expect("Invoke provider");

    assert_eq!(res.get("audio_len").unwrap().as_u64().unwrap(), 11);
    assert_eq!(res.get("sample_rate").unwrap().as_u64().unwrap(), 24000);
}
