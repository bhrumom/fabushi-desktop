use std::path::Path;
use std::sync::Arc;

use crate::extensions::action_audit::extension::{
    ActionAuditExtension, start_action_audit_extension,
};
use crate::extensions::auth::auth_service::HostAuthServiceOptions;
use crate::extensions::auth::extension::{
    HostAuthExtension, start_host_auth_extension_with_options,
};
use crate::extensions::auth::user_full_name_service::production_user_full_name_fetch;
use crate::extensions::box_lifecycle::box_lifecycle_service::BoxLifecycleService;
use crate::extensions::box_lifecycle::extension::start_box_lifecycle_extension;
use crate::extensions::box_lifecycle::production::{
    ProductionBoxLifecycleClient, ProductionBoxLifecycleClientFactory,
};
use crate::extensions::browser_ua::{
    BrowserUaExtensionRuntime, BrowserUaHostLog, start_browser_ua_extension,
};
use crate::extensions::cloud_agents::extension::{
    CloudAgentsExtension, start_cloud_agents_extension,
};
use crate::extensions::experiments::{
    HostExperimentsExtension, start_host_experiments_extension,
};
use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::managed_setup::extension::{
    ManagedSetupExtension, start_managed_setup_extension,
};
use crate::extensions::memory::extension::HostMemoryExtension;
use crate::extensions::memory::production::start_production_memory_extension;
use crate::extensions::notify_bus::extension::{
    HostNotifyBusExtension, start_notify_bus_extension,
};
use crate::extensions::source_map::extension::start_source_map_extension;
use crate::extensions::source_map::source_map_service::SandSourceMap;
use crate::extensions::telemetry::host_telemetry_service::HostStructuredLogTelemetry;
use crate::extensions::telemetry::webauthn_proxy_telemetry::{
    WebAuthnProxyReport, webauthn_proxy_telemetry,
};
use crate::extensions::trays::extension::{
    HostTraysExtension, start_trays_extension,
};
use crate::extensions::webauthn_proxy::extension::{
    HostWebAuthnProxyExtension, start_webauthn_proxy_extension,
};
use crate::production_binding_providers::production_cloud_agent_trace_converter;

/// Grok-shaped owner for the production extension subset that is already
/// shipping in the Rust Host.
///
/// This is deliberately not presented as the complete frozen 35-slot registry:
/// missing extension implementations stay visible in the architecture manifest
/// until their real production owners exist.
pub const CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Experiments,
    HostExtensionId::ActionAudit,
    HostExtensionId::CloudAgents,
    HostExtensionId::NotifyBus,
    HostExtensionId::Memory,
    HostExtensionId::ManagedSetup,
    HostExtensionId::SourceMap,
    HostExtensionId::Trays,
    HostExtensionId::BoxLifecycle,
    HostExtensionId::WebauthnProxy,
    HostExtensionId::BrowserUa,
];

pub struct ProductionBrowserUaLog;

impl BrowserUaHostLog for ProductionBrowserUaLog {
    fn log(&self, message: &str) {
        eprintln!("mahayana-host-browser-ua {message}");
    }
}

pub fn start_production_browser_ua(
    auth: Arc<HostAuthExtension>,
    experiments: Arc<HostExperimentsExtension>,
) -> BrowserUaExtensionRuntime {
    start_browser_ua_extension(
        auth,
        experiments,
        Arc::new(ProductionBrowserUaLog),
        None,
        None,
    )
}

pub struct ProductionHostExtensions {
    pub auth: Arc<HostAuthExtension>,
    pub experiments: Arc<HostExperimentsExtension>,
    pub notify_bus: HostNotifyBusExtension,
    pub memory: HostMemoryExtension,
    pub managed_setup: Arc<ManagedSetupExtension>,
    pub source_map: Arc<SandSourceMap>,
    pub trays: Arc<HostTraysExtension>,
    pub box_lifecycle:
        Arc<BoxLifecycleService<ProductionBoxLifecycleClient<HostAuthExtension>>>,
    pub webauthn_proxy: Arc<HostWebAuthnProxyExtension>,
    pub action_audit: ActionAuditExtension,
    pub cloud_agents: CloudAgentsExtension,
}

pub fn start_production_host_extensions(
    app_data_dir: &Path,
    telemetry_logs: HostStructuredLogTelemetry,
) -> Result<ProductionHostExtensions, String> {
    let auth_options = HostAuthServiceOptions::production(|message| {
        eprintln!("mahayana-host-auth {message}");
    })
    .map_err(|error| error.to_string())?;

    let backend_url = auth_options
        .backend_url
        .clone()
        .ok_or_else(|| "production Auth requires a configured backend URL".to_string())?;
    let auth = Arc::new(
        start_host_auth_extension_with_options(
            auth_options,
            production_user_full_name_fetch(backend_url.clone()),
        )
        .map_err(|error| error.to_string())?,
    );
    let experiments = Arc::new(start_host_experiments_extension());
    let action_audit = start_action_audit_extension(
        backend_url.clone(),
        Arc::clone(&auth),
        Arc::clone(&experiments),
        telemetry_logs,
    );
    let cloud_agents = start_cloud_agents_extension(
        backend_url.clone(),
        Arc::clone(&auth),
        production_cloud_agent_trace_converter(),
    );
    let notify_bus = start_notify_bus_extension(
        Arc::clone(&auth),
        Arc::clone(&experiments),
        Arc::new(|message| eprintln!("{message}")),
    )
    .map_err(|error| error.to_string())?;
    let memory = start_production_memory_extension();
    let managed_setup = start_managed_setup_extension(
        backend_url,
        Arc::clone(&auth),
        app_data_dir,
    );

    let source_map = Arc::new(start_source_map_extension());
    let trays = Arc::new(start_trays_extension());

    let factory =
        ProductionBoxLifecycleClientFactory::from_process_env()
            .map_err(|error| error.to_string())?;
    let box_lifecycle =
        Arc::new(start_box_lifecycle_extension(Arc::clone(&auth), &factory));
    let webauthn_proxy = Arc::new(start_webauthn_proxy_extension(Arc::new(
        |report: WebAuthnProxyReport| {
            let projection = webauthn_proxy_telemetry(&report);
            eprintln!(
                "mahayana-host-webauthn level={} event={} metadata={}",
                projection.level.unwrap_or("info"),
                projection.event.unwrap_or("sand.webauthn_proxy"),
                serde_json::to_string(&projection.metadata)
                    .unwrap_or_else(|_| "{}".into()),
            );
        },
    )));

    Ok(ProductionHostExtensions {
        auth,
        experiments,
        notify_bus,
        memory,
        managed_setup,
        source_map,
        trays,
        box_lifecycle,
        webauthn_proxy,
        action_audit,
        cloud_agents,
    })
}
