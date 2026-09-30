use std::collections::{BTreeMap, BTreeSet};

use serde_json::{Map, Value, json};

use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::registry::HOST_EXTENSION_ORDER;
use mahayana_host_runtime::extensions::inference::extension::{
    INFERENCE_DEPENDENCIES, inference_extension_id,
};
use mahayana_host_runtime::extensions::inference::production::ProductionInferenceExtension;
use mahayana_host_runtime::host_production_extensions::{
    CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS, ProductionBrowserUaLog,
    ProductionHostExtensions, project_conversation_gc, project_host_diagnostic,
    project_session_diagnostic,
};

const PRODUCTION_OWNER: &str = include_str!("../src/host_production_extensions.rs");
const MEMORY_PRODUCTION: &str = include_str!("../src/extensions/memory/production.rs");
const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");

#[test]
fn current_shipping_subset_is_declared_in_the_frozen_35_slot_registry() {
    let all = HOST_EXTENSION_ORDER.iter().copied().collect::<BTreeSet<_>>();
    let shipping = CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS
        .iter()
        .copied()
        .collect::<BTreeSet<_>>();
    assert_eq!(
        shipping.len(),
        CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS.len(),
        "shipping production extension ids must not contain duplicates"
    );
    assert!(
        shipping.iter().all(|id| all.contains(id)),
        "every shipping production extension must occupy a frozen Grok registry slot"
    );
    assert_eq!(HOST_EXTENSION_ORDER.len(), 35);
    assert!(shipping.contains(&HostExtensionId::Auth));
    assert!(shipping.contains(&HostExtensionId::Automations));
    assert!(shipping.contains(&HostExtensionId::Settings));
    assert!(shipping.contains(&HostExtensionId::BrowserUa));
    assert!(shipping.contains(&HostExtensionId::CodebaseTelemetry));
    assert!(shipping.contains(&HostExtensionId::ContentSearch));
    assert!(shipping.contains(&HostExtensionId::Telemetry));
    assert!(shipping.contains(&HostExtensionId::LocalToolPermission));
    assert!(shipping.contains(&HostExtensionId::LocalExec));
    assert!(shipping.contains(&HostExtensionId::CloudAgents));
    assert!(shipping.contains(&HostExtensionId::WebauthnProxy));
    assert!(shipping.contains(&HostExtensionId::StateBackstop));
    assert!(shipping.contains(&HostExtensionId::Inference));
    assert!(shipping.contains(&HostExtensionId::Mcp));
    assert!(shipping.contains(&HostExtensionId::Wallpaper));
    assert!(shipping.contains(&HostExtensionId::ForeverBox));
    assert!(shipping.contains(&HostExtensionId::Attachments));
    assert!(shipping.contains(&HostExtensionId::Secrets));
    assert_eq!(inference_extension_id(), HostExtensionId::Inference);
    assert_eq!(
        INFERENCE_DEPENDENCIES,
        &[
            HostExtensionId::Auth,
            HostExtensionId::Experiments,
            HostExtensionId::Settings,
        ]
    );
}

#[test]
fn shipping_production_extension_owner_remains_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<ProductionHostExtensions>();
    assert_send_sync::<mahayana_host_runtime::host_production_extensions::ProductionHostBoxExtensions>();
    assert_send_sync::<ProductionBrowserUaLog>();
    assert_send_sync::<ProductionInferenceExtension>();
}


#[test]
fn browser_ua_and_wallpaper_have_one_production_lifecycle_owner() {
    for needle in [
        "pub browser_ua: Mutex<Option<BrowserUaExtensionRuntime>>",
        "pub wallpaper: Mutex<Option<HostWallpaperExtension>>",
        "let browser_ua = start_production_browser_ua(",
        "let wallpaper = start_wallpaper_extension(",
        "browser_ua: Mutex::new(Some(browser_ua))",
        "wallpaper: Mutex::new(Some(wallpaper))",
    ] {
        assert!(
            PRODUCTION_OWNER.contains(needle),
            "ProductionHostExtensions must own the centralized lifecycle: {needle}"
        );
    }

    for forbidden in [
        "let browser_ua_runtime = start_production_browser_ua(",
        "let mut wallpaper_extension = start_wallpaper_extension(",
        "browser_ua_runtime.stop()",
        "wallpaper_extension.stop()",
    ] {
        assert!(
            !SHIPPING_HOST.contains(forbidden),
            "shipping Host must not recreate a second extension lifecycle: {forbidden}"
        );
    }
}


#[test]
fn box_stage_extensions_have_one_production_composition_owner() {
    for needle in [
        "pub struct ProductionHostBoxExtensions",
        "start_production_host_box_extensions(",
        "let forever_box = start_forever_box_extension(",
        "let attachments = start_attachments_extension(",
        "let secrets = Arc::new(start_secrets_extension(",
    ] {
        assert!(
            PRODUCTION_OWNER.contains(needle),
            "host production composition must own the Box-stage extension: {needle}"
        );
    }

    for forbidden in [
        "let forever_box = start_forever_box_extension(",
        "let attachments_extension = start_attachments_extension(",
        "Arc::new(start_secrets_extension(",
    ] {
        assert!(
            !SHIPPING_HOST.contains(forbidden),
            "shipping Host must not recreate a second Box-stage extension owner: {forbidden}"
        );
    }

    assert!(
        SHIPPING_HOST.contains("start_production_host_box_extensions(&production_extensions, production_box)"),
        "shipping Host must consume the centralized Box-stage owner"
    );
}


#[test]
fn mcp_plugin_skills_have_one_shipping_production_owner() {
    for needle in [
        "HostExtensionId::Mcp",
        "mcp: Mutex<Option<McpExtensionRuntime>>",
        "pub fn start_mcp(",
        "CoordinatorPluginSkillsLoader::new(relay)",
        "SandPluginSkillsService::new(",
        "McpHostService::new(backend, Some(plugin_port))",
        "PluginSkillsAuthenticatedStartup::start(",
        "cleanup_legacy_mcp_auth_credentials(sand_root_dir)",
        "sweep_legacy_plugin_skill_references(&sweep_root, &catalog)",
        "Some(on_startup_sync_succeeded)",
        "RealPluginSkillsPolling::daily()",
        "create_production_skill_publish(",
        "pub fn stop_mcp(&self)",
    ] {
        assert!(
            PRODUCTION_OWNER.contains(needle),
            "ProductionHostExtensions must own the MCP production lifecycle: {needle}"
        );
    }

    assert!(
        SHIPPING_HOST.contains("production_extensions.start_mcp("),
        "shipping Host must start MCP through the centralized production extension owner"
    );
    assert!(
        SHIPPING_HOST.contains("production_extensions.stop_mcp()"),
        "shipping Host must stop MCP through the centralized production extension owner"
    );
    assert!(
        !SHIPPING_HOST.contains("McpHostService::new(mcp_manager_backend, None)"),
        "shipping Host must never bypass Plugin Skills with a None port"
    );
}


#[test]
fn automations_have_one_shipping_production_owner() {
    for needle in [
        "HostExtensionId::Automations",
        "automations: Mutex<Option<Arc<ProductionAutomationsLifecycle>>>",
        "pub fn start_automations(",
        "ProductionAutomationsLifecycle::start(",
        "pub fn stop_automations(&self)",
    ] {
        assert!(
            PRODUCTION_OWNER.contains(needle),
            "ProductionHostExtensions must own the Automations production lifecycle: {needle}"
        );
    }
}

#[test]
fn state_backstop_is_composed_after_box_store_sync_and_stopped_before_it() {
    for needle in [
        "HostExtensionId::StateBackstop",
        "state_backstop: Mutex<Option<HostStateBackstopExtension>>",
        "let box_api = extension.api()",
        "create_production_state_backstop_runtime()",
        "store_api.object_store_for(source_id)",
        "let source_map = Arc::clone(&self.source_map)",
        ".get_or_create(agent_id)",
        "start_state_backstop_extension(options)",
        "state_backstop.stop()",
    ] {
        assert!(
            PRODUCTION_OWNER.contains(needle),
            "production Host must own frozen StateBackstop composition: {needle}"
        );
    }
}


#[test]
fn memory_synthesis_is_pinned_to_authenticated_statsig_and_shipping_inference() {
    for needle in [
        "start_authenticated_statsig_bootstrap(",
        "pin_gate_on_authenticated_bootstrap(",
        "\"sand_memory_dreaming\"",
        "MemorySynthesisReport::SkippedGate",
        "create_production_memory_synthesis(",
    ] {
        assert!(
            PRODUCTION_OWNER.contains(needle),
            "production Host must preserve frozen authenticated Memory synthesis wiring: {needle}"
        );
    }
    assert!(
        !PRODUCTION_OWNER.contains("if experiments.check_feature_gate(\"sand_memory_dreaming\")"),
        "Memory synthesis must not read the unpinned startup fallback before authenticated Statsig bootstrap"
    );

    for needle in [
        "run_summarization_prompt(",
        "synthesis_system_prompt()",
        "verification_system_prompt()",
        "is_cancelled()",
        "memory_synthesis_telemetry_report",
        "report_memory_synthesis",
    ] {
        assert!(
            MEMORY_PRODUCTION.contains(needle),
            "shipping Memory production must consume the shared inference/telemetry path: {needle}"
        );
    }
}

#[test]
fn structured_log_domain_reporter_adapters_preserve_frozen_fields() {
    let host = project_host_diagnostic(&mahayana_host_runtime::host_diagnostics::HostDiagnostic {
        kind: "send_ledger_degraded".into(),
        fields: Map::from_iter([
            ("stage".into(), json!("commit")),
            ("agentId".into(), json!("agent-1")),
            ("reason".into(), json!("io")),
            ("errorClass".into(), json!("disk")),
        ]),
    });
    assert_eq!(host.stage.as_deref(), Some("commit"));
    assert_eq!(host.agent_id.as_deref(), Some("agent-1"));
    assert_eq!(host.error_class.as_deref(), Some("disk"));

    let session = project_session_diagnostic(
        &mahayana_host_runtime::extensions::session::session_diagnostics::SessionDiagnostic {
            family: "store_db".into(),
            kind: "quarantine_copied".into(),
            metadata: BTreeMap::from([
                ("agentId".into(), Value::String("agent-1".into())),
                ("outcome".into(), Value::String("recovered".into())),
                ("salvagedKv".into(), json!(3)),
            ]),
        },
    )
    .expect("known session family");
    assert_eq!(session.agent_id.as_deref(), Some("agent-1"));
    assert_eq!(session.outcome.as_deref(), Some("recovered"));
    assert_eq!(session.salvaged_kv, Some(3));

    let gc = project_conversation_gc(
        &mahayana_host_runtime::extensions::session::conversation_size_limits::ConversationGcReport {
            trigger: "size-cap".into(),
            agent_id: "agent-1".into(),
            outcome: "skipped".into(),
            skip_reason: Some("unresolved-refs".into()),
            unresolved_proto_refs: Some(4),
            deleted_rows: None,
            deleted_bytes: None,
            live_rows: None,
            live_bytes: None,
            vacuumed: None,
            still_over_cap: true,
        },
    );
    match gc {
        mahayana_host_runtime::extensions::telemetry::conversation_gc_telemetry::ConversationGcReport::Skipped {
            skip_reason,
            unresolved_proto_refs,
            ..
        } => {
            assert_eq!(skip_reason, "unresolved-refs");
            assert_eq!(unresolved_proto_refs, Some(4.0));
        }
        other => panic!("unexpected GC projection: {other:?}"),
    }
}
