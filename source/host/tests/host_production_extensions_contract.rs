use std::collections::BTreeSet;

use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::registry::HOST_EXTENSION_ORDER;
use mahayana_host_runtime::extensions::inference::extension::{
    INFERENCE_DEPENDENCIES, inference_extension_id,
};
use mahayana_host_runtime::extensions::inference::production::ProductionInferenceExtension;
use mahayana_host_runtime::host_production_extensions::{
    CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS, ProductionBrowserUaLog,
    ProductionHostExtensions,
};

const PRODUCTION_OWNER: &str = include_str!("../src/host_production_extensions.rs");
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
    assert!(shipping.contains(&HostExtensionId::Settings));
    assert!(shipping.contains(&HostExtensionId::BrowserUa));
    assert!(shipping.contains(&HostExtensionId::CodebaseTelemetry));
    assert!(shipping.contains(&HostExtensionId::ContentSearch));
    assert!(shipping.contains(&HostExtensionId::Telemetry));
    assert!(shipping.contains(&HostExtensionId::LocalToolPermission));
    assert!(shipping.contains(&HostExtensionId::LocalExec));
    assert!(shipping.contains(&HostExtensionId::CloudAgents));
    assert!(shipping.contains(&HostExtensionId::WebauthnProxy));
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
            "shipping Host must not recreate a Box-stage extension owner: {forbidden}"
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
