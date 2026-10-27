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

#[test]
fn test_ocrm_plugins_boundary_invariants() {
    // Tests equivalent behavior from upstream src/plugins/contracts/boundary-invariants.test.ts
    use mahayana_host_runtime::extensions::plugins::hook_runner::HookBoundaryInvariantGuard;

    // 1. Allowed bundled typed hooks
    assert!(HookBoundaryInvariantGuard::is_allowed_typed_hook("agent_end"));
    assert!(HookBoundaryInvariantGuard::is_allowed_typed_hook("before_prompt_build"));
    assert!(HookBoundaryInvariantGuard::is_allowed_typed_hook("session_end"));
    assert!(!HookBoundaryInvariantGuard::is_allowed_typed_hook("unknown_untyped_hook"));

    // 2. Production code off raw registerHook calls
    let clean_sources = [
        ("extensions/acpx/index.ts", "api.on('reply_dispatch', () => {})"),
        ("extensions/codex/index.ts", "api.on('inbound_claim', () => {})"),
    ];
    assert!(HookBoundaryInvariantGuard::assert_no_raw_register_hook_calls(&clean_sources).is_ok());

    let bad_sources = [
        ("extensions/legacy/index.ts", "api.registerHook('reply_dispatch', () => {})"),
    ];
    let violations = HookBoundaryInvariantGuard::assert_no_raw_register_hook_calls(&bad_sources).unwrap_err();
    assert_eq!(violations, vec!["extensions/legacy/index.ts"]);

    // 3. Bundled extension import boundary guards
    assert!(HookBoundaryInvariantGuard::is_allowed_bundled_extension_import("extensions/browser/api.js"));
    assert!(HookBoundaryInvariantGuard::is_allowed_bundled_extension_import("extensions/discord/runtime-api.js"));
    assert!(!HookBoundaryInvariantGuard::is_allowed_bundled_extension_import("extensions/browser/src/internal-helper.js"));
    assert!(!HookBoundaryInvariantGuard::is_allowed_bundled_extension_import("extensions/discord/subagent-hooks-api.ts"));
}

#[test]
fn test_ocrm_plugins_agent_event_payload_and_active_run_steering() {
    // Tests equivalent behavior from agent-event-payload.contract.test.ts and active-run-steering.contract.test.ts
    use mahayana_host_runtime::extensions::plugins::hook_runner::{
        ActiveRunSteeringManager, AgentEventSubscription, EventPayloadMode, SteerActiveRunParams,
    };

    // 1. Agent event subscription filtering & metadata isolation
    let sub = AgentEventSubscription {
        id: "events".to_string(),
        streams: vec!["tool".to_string()],
        phases: vec![" start ".to_string()], // trimmed matching
        payload_mode: EventPayloadMode::Metadata,
    };

    assert!(sub.matches("tool", Some("start")));
    assert!(!sub.matches("tool", Some("result")));
    assert!(!sub.matches("chat", Some("start")));

    let full_data = json!({
        "phase": "result",
        "count": 42,
        "enabled": false,
        "short_string": "hello",
        "long_string": "a".repeat(300),
        "nested": {"key": "val"},
        "arr": [1, 2, 3]
    });

    let filtered = sub.filter_payload(&full_data);
    assert_eq!(filtered.get("phase").unwrap().as_str().unwrap(), "result");
    assert_eq!(filtered.get("count").unwrap().as_i64().unwrap(), 42);
    assert_eq!(filtered.get("short_string").unwrap().as_str().unwrap(), "hello");
    // Long string > 256, nested, and arr are excluded in metadata mode
    assert!(filtered.get("long_string").is_none());
    assert!(filtered.get("nested").is_none());
    assert!(filtered.get("arr").is_none());

    // 2. Active run steering
    let steering = ActiveRunSteeringManager::new();

    // Success on active streaming run
    let params = SteerActiveRunParams {
        session_key: "agent:main:steering-test".to_string(),
        text: "Use the corrected result.".to_string(),
        display_text: Some("Check the result.".to_string()),
        idempotency_key: Some("review-1".to_string()),
    };
    let res = steering.steer_active_run(true, Some("steered-run"), true, &params);
    assert!(res.steered);
    assert_eq!(res.run_id.as_deref(), Some("steered-run"));

    // Duplicate idempotency key rejected
    let res_dup = steering.steer_active_run(true, Some("steered-run"), true, &params);
    assert!(!res_dup.steered);
    assert_eq!(res_dup.reason.as_deref(), Some("duplicate"));

    // Disabled prompt injection
    let res_disabled = steering.steer_active_run(false, Some("steered-run"), true, &params);
    assert!(!res_disabled.steered);
    assert!(res_disabled.reason.unwrap().contains("allowPromptInjection=false"));

    // No active run
    let res_no_run = steering.steer_active_run(true, None, true, &params);
    assert!(!res_no_run.steered);
    assert_eq!(res_no_run.reason.as_deref(), Some("no_active_run"));

    // Not streaming
    let res_not_streaming = steering.steer_active_run(true, Some("steered-run"), false, &params);
    assert!(!res_not_streaming.steered);
    assert_eq!(res_not_streaming.reason.as_deref(), Some("not_streaming"));
}

#[test]
fn test_ocrm_plugins_architectural_boundary_guardrails() {
    // Tests equivalent behavior from bundled-extension-config-api-guardrails, core-extension-facade-boundary,
    // deprecated-internal-config-api, and model-catalog-core-imports contract test suites.
    use mahayana_host_runtime::extensions::plugins::hook_runner::{
        BundledExtensionConfigApiGuard, CoreExtensionFacadeBoundaryGuard,
        DeprecatedInternalConfigApiGuard, ModelCatalogCoreImportsGuard,
    };

    // 1. Bundled extension config API guard
    assert!(BundledExtensionConfigApiGuard::validate_extension_config_schema(
        "extensions/telegram/src/config-schema.ts",
        "import { createChannelConfigSchema } from 'openclaw/plugin-sdk/channel-config-schema';"
    ).is_ok());

    let err_forbidden_bundled = BundledExtensionConfigApiGuard::validate_extension_config_schema(
        "extensions/telegram/src/config-schema.ts",
        "import { schema } from 'openclaw/plugin-sdk/bundled-channel-config-schema';"
    ).unwrap_err();
    assert!(err_forbidden_bundled.contains("missing allowed openclaw/plugin-sdk/channel-config-schema"));

    assert!(BundledExtensionConfigApiGuard::validate_extension_config_schema(
        "extensions/whatsapp/src/config-schema.ts",
        "import { defineChannelConfig } from '../config-api.js';"
    ).is_ok());

    let err_forbidden_whatsapp = BundledExtensionConfigApiGuard::validate_extension_config_schema(
        "extensions/whatsapp/src/config-schema.ts",
        "import { defineChannelConfig } from 'openclaw/plugin-sdk/channel-config-schema';"
    ).unwrap_err();
    assert!(err_forbidden_whatsapp.contains("missing allowed ../config-api.js"));

    // 2. Core extension facade boundary guard
    let clean_paths = ["src/plugin-sdk/index.ts", "src/plugins/registry.ts"];
    assert!(CoreExtensionFacadeBoundaryGuard::assert_no_ollama_facades(&clean_paths).is_ok());

    let violating_paths = ["src/plugin-sdk/ollama.ts", "src/plugin-sdk/index.ts"];
    let err_facade = CoreExtensionFacadeBoundaryGuard::assert_no_ollama_facades(&violating_paths).unwrap_err();
    assert_eq!(err_facade, vec!["src/plugin-sdk/ollama.ts"]);

    let clean_fixtures = [
        ("auth-choice.test.ts", "export const config = { provider: 'test' };"),
        ("discovery.test.ts", "test('discovery', () => {});"),
    ];
    assert!(CoreExtensionFacadeBoundaryGuard::assert_no_bundled_provider_terms_in_fixtures(&clean_fixtures).is_ok());

    let dirty_fixtures = [
        ("tts-contract-suites.ts", "const p = 'Ollama';"),
    ];
    let err_terms = CoreExtensionFacadeBoundaryGuard::assert_no_bundled_provider_terms_in_fixtures(&dirty_fixtures).unwrap_err();
    assert!(err_terms[0].contains("contains forbidden term 'Ollama'"));

    // 3. Deprecated internal config API guard
    let clean_config_sources = [("src/config.ts", "export function loadConfig() {}")];
    assert!(DeprecatedInternalConfigApiGuard::assert_no_deprecated_config_api(&clean_config_sources).is_ok());

    let deprecated_sources = [("src/legacy.ts", "loadDeprecatedPluginConfig(id);")];
    let err_deprecated = DeprecatedInternalConfigApiGuard::assert_no_deprecated_config_api(&deprecated_sources).unwrap_err();
    assert!(err_deprecated[0].contains("loadDeprecatedPluginConfig"));

    // 4. Model catalog core imports guard
    let clean_catalog_sources = [("src/agents.ts", "import { normalize } from '@openclaw/model-catalog-core/model-catalog-normalize';")];
    assert!(ModelCatalogCoreImportsGuard::assert_no_legacy_bridge_imports(&clean_catalog_sources).is_ok());

    let dirty_catalog_sources = [("src/agents.ts", "import { normalize } from './src/model-catalog/normalize.js';")];
    let err_bridges = ModelCatalogCoreImportsGuard::assert_no_legacy_bridge_imports(&dirty_catalog_sources).unwrap_err();
    assert!(err_bridges[0].contains("imports legacy bridge"));
}

#[test]
fn test_ocrm_plugins_runtime_import_side_effects_and_seams() {
    // Tests equivalent behavior from runtime-import-side-effects and runtime-seams contract test suites.
    use mahayana_host_runtime::extensions::plugins::runtime_final::{
        RuntimeFacadeSeamManager, RuntimeImportSideEffectsGuard,
    };

    // 1. Runtime import side-effects
    let clean_runtime_sources = [("src/plugins/runtime/runtime-channel.ts", "export const ok = true;")];
    assert!(RuntimeImportSideEffectsGuard::assert_no_direct_channel_registry_imports(&clean_runtime_sources).is_ok());

    let dirty_runtime_sources = [("src/plugins/runtime/index.ts", "import '../../channels/plugins/registry';")];
    let err_direct = RuntimeImportSideEffectsGuard::assert_no_direct_channel_registry_imports(&dirty_runtime_sources).unwrap_err();
    assert_eq!(err_direct, vec!["src/plugins/runtime/index.ts"]);

    // Markdown table mode resolution
    assert_eq!(
        RuntimeImportSideEffectsGuard::resolve_markdown_table_mode("telegram", true, None),
        "bullets"
    );
    assert_eq!(
        RuntimeImportSideEffectsGuard::resolve_markdown_table_mode("telegram", true, Some("block")),
        "block"
    );
    assert_eq!(
        RuntimeImportSideEffectsGuard::resolve_markdown_table_mode("discord", true, None),
        "block"
    );
    assert_eq!(
        RuntimeImportSideEffectsGuard::resolve_markdown_table_mode("discord", false, None),
        "bullets"
    );

    // 2. Runtime facade seam
    assert!(RuntimeFacadeSeamManager::resolve_bundled_plugin_public_surface_access("line", true));
    assert!(!RuntimeFacadeSeamManager::resolve_bundled_plugin_public_surface_access("line", false));

    // SSRF fetch guard
    assert!(RuntimeFacadeSeamManager::fetch_with_ssrf_guard("https://public.example/api", false).is_ok());
    let err_ssrf = RuntimeFacadeSeamManager::fetch_with_ssrf_guard("http://192.168.1.1/admin", true).unwrap_err();
    assert!(err_ssrf.contains("SSRF blocked access to private network"));
}


