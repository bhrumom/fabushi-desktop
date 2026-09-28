use std::collections::BTreeMap;
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
use crate::extensions::content_search::extension::{
    ProductionContentSearchExtension, start_production_content_search_extension,
};
use crate::extensions::codebase_telemetry::extension::{
    CodebaseTelemetryExtension, start_codebase_telemetry_extension,
};
use crate::extensions::experiments::{
    HostExperimentsExtension, start_host_experiments_extension,
};
use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::inference::production::{
    ProductionInferenceExtension, start_production_inference_extension,
};
use crate::extensions::managed_setup::extension::{
    ManagedSetupExtension, start_managed_setup_extension,
};
use crate::extensions::local_exec::extension::{
    HostLocalExecExtension, start_local_exec_extension,
};
use crate::extensions::local_tool_permission::extension::{
    HostLocalToolPermissionExtension, start_local_tool_permission_extension,
};
use crate::extensions::memory::extension::HostMemoryExtension;
use crate::extensions::memory::production::start_production_memory_extension;
use crate::extensions::notify_bus::extension::{
    HostNotifyBusExtension, start_notify_bus_extension,
};
use crate::extensions::source_map::extension::start_source_map_extension;
use crate::extensions::source_map::source_map_service::SandSourceMap;
use crate::extensions::settings::extension::start_settings_extension;
use crate::extensions::settings::settings_service::{SettingsService, SettingsSubscription};
use crate::extensions::telemetry::extension::{
    HostTelemetryExtension, start_host_telemetry_extension,
};
use crate::extensions::telemetry::webauthn_proxy_telemetry::{
    WebAuthnProxyReport, webauthn_proxy_telemetry,
};
use crate::extensions::trays::extension::{
    HostTraysExtension, start_trays_extension,
};
use crate::extensions::webauthn_proxy::extension::{
    HostWebAuthnProxyExtension, start_webauthn_proxy_extension,
};
use crate::host_event_bus::SandHostEventBus;
use crate::production_binding_providers::production_cloud_agent_trace_converter;

/// Grok-shaped owner for the production extension subset that is already
/// shipping in the Rust Host.
///
/// This is deliberately not presented as the complete frozen 35-slot registry:
/// missing extension implementations stay visible in the architecture manifest
/// until their real production owners exist.
pub const CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Telemetry,
    HostExtensionId::ContentSearch,
    HostExtensionId::Settings,
    HostExtensionId::Experiments,
    HostExtensionId::CodebaseTelemetry,
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
    HostExtensionId::LocalToolPermission,
    HostExtensionId::LocalExec,
    HostExtensionId::Inference,
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
    pub telemetry: HostTelemetryExtension,
    pub auth: Arc<HostAuthExtension>,
    pub settings: Arc<SettingsService>,
    pub experiments: Arc<HostExperimentsExtension>,
    pub inference: Arc<ProductionInferenceExtension>,
    pub content_search: Arc<ProductionContentSearchExtension>,
    pub local_tool_permission: Arc<HostLocalToolPermissionExtension>,
    pub local_exec: Arc<HostLocalExecExtension>,
    _settings_feature_override_subscription:
        SettingsSubscription<dyn Fn(BTreeMap<String, bool>) + Send + Sync + 'static>,
    pub codebase_telemetry: CodebaseTelemetryExtension,
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
    events: SandHostEventBus,
) -> Result<ProductionHostExtensions, String> {
    let telemetry = start_host_telemetry_extension(app_data_dir)
        .map_err(|error| error.to_string())?;
    let telemetry_logs = telemetry.logs.clone();
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
    let settings = start_settings_extension();
    let experiments = Arc::new(start_host_experiments_extension());
    let inference = Arc::new(start_production_inference_extension(
        Arc::clone(&auth),
        Arc::clone(&experiments),
        Arc::clone(&settings),
    ));
    let content_search_logs = telemetry_logs.clone();
    let content_search = Arc::new(start_production_content_search_extension(
        Arc::clone(&experiments),
        Arc::new(move |health| {
            let payload = serde_json::json!({
                "kind": health.kind,
                "stage": health.stage,
                "count": health.count,
                "errorClass": health.error_class,
            });
            let _ = content_search_logs.report_search_index_health(&payload);
        }),
    ));
    let local_tool_permission = Arc::new(start_local_tool_permission_extension(Arc::clone(&settings)));
    let local_exec = Arc::new(start_local_exec_extension(telemetry.logs.clone()));
    let experiments_for_settings = Arc::clone(&experiments);
    let settings_feature_override_subscription =
        settings.subscribe_to_feature_flag_overrides(Arc::new(move |overrides| {
            experiments_for_settings.replace_feature_flag_overrides(overrides);
        }));
    let codebase_telemetry = start_codebase_telemetry_extension(
        Arc::clone(&auth),
        Arc::clone(&experiments),
        events,
        app_data_dir,
        backend_url.clone(),
    )?;
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
        telemetry,
        auth,
        settings,
        experiments,
        inference,
        content_search,
        local_tool_permission,
        local_exec,
        _settings_feature_override_subscription: settings_feature_override_subscription,
        codebase_telemetry,
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
