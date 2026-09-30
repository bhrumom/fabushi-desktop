use std::collections::BTreeMap;
use std::path::Path;
use std::sync::{Arc, Mutex};

use crate::host_paths::get_sand_root_dir;
use crate::host_diagnostics::{
    HostDiagnostic as DomainHostDiagnostic, pin_host_diagnostics_reporter,
};
use crate::extensions::session::conversation_size_limits::{
    ConversationGcReport as DomainConversationGcReport, pin_conversation_gc_reporter,
};
use crate::extensions::session::session_diagnostics::{
    SessionDiagnostic as DomainSessionDiagnostic, pin_session_diagnostics_reporter,
};
use crate::extensions::action_audit::extension::{
    ActionAuditExtension, start_action_audit_extension,
};
use crate::extensions::attachments::extension::{
    HostAttachmentsExtension, start_attachments_extension,
};
use crate::extensions::auth::auth_service::HostAuthServiceOptions;
use crate::extensions::auth::extension::{
    HostAuthExtension, start_host_auth_extension_with_options,
};
use crate::extensions::auth::user_full_name_service::production_user_full_name_fetch;
use crate::extensions::automations::listener_connect_watcher_production::ListenerConnectedCallback;
use crate::extensions::automations::production::{FireDispatch, FirePollStateReader, ProductionLog, RelayEventSink, RelayListeners};
use crate::extensions::automations::sand_automation_cloud_sync::is_server_schedulable;
use crate::extensions::automations::listener_integrations::AgentChannelsReader;
use crate::extensions::automations::production_lifecycle::{
    CloudAgentIdsReader, CloudDefinitionsReader, CloudTimeZoneReader,
    ProductionAutomationsLifecycle,
};
use crate::extensions::box_lifecycle::box_lifecycle_service::BoxLifecycleService;
use crate::extensions::box_lifecycle::extension::start_box_lifecycle_extension;
use crate::extensions::box_lifecycle::production::{
    ProductionBoxLifecycleClient, ProductionBoxLifecycleClientFactory,
};
use crate::extensions::box_store_sync::extension::{
    BoxStoreScheduling, BoxStoreSyncExtension, BoxStoreSyncExtensionDeps,
    start_box_store_sync_extension,
};
use crate::extensions::box_store_sync::production::{
    ProductionBoxStoreSyncApi, ProductionBoxStoreSyncFactory, ProductionBoxStoreSyncService,
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
    HostExperimentsExtension, ProductionStatsigBootstrapRuntime,
    start_authenticated_statsig_bootstrap, start_host_experiments_extension,
};
use crate::extensions::forever_box::{
    ForeverBoxExtensionOptions, ForeverBoxLifecycle, ForeverBoxService,
    start_forever_box_extension,
};
use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::inference::production::{
    ProductionInferenceExtension, start_production_inference_extension,
};
use crate::extensions::managed_setup::extension::{
    ManagedSetupExtension, start_managed_setup_extension,
};
use crate::extensions::mcp::coordinator_relay::{
    CoordinatorMcpLifecycleRelay, CoordinatorPluginSkillsLoader,
};
use crate::extensions::mcp::extension::{
    AuthRenewalPort, McpExtensionRuntime, McpPluginSkillsService,
    PluginSkillsAuthenticatedStartup, PollingPort,
};
use crate::extensions::mcp::legacy_live_references::{
    PluginCatalogEntry, PluginSkillCatalogEntry, sweep_legacy_plugin_skill_references,
};
use crate::extensions::mcp::mcp_service::{
    McpHostService, McpManagerBackend, PluginSkillsPort,
};
use crate::extensions::mcp::plugin_skills::{
    PluginSkillsLoader, SandPluginSkillsService,
};
use crate::extensions::mcp::production::{
    LegacyMcpAuthCleanupOutcome, RealPluginSkillsPolling,
    cleanup_legacy_mcp_auth_credentials, create_production_skill_publish,
};
use crate::extensions::local_exec::extension::{
    HostLocalExecExtension, start_local_exec_extension,
};
use crate::extensions::local_tool_permission::extension::{
    HostLocalToolPermissionExtension, start_local_tool_permission_extension,
};
use crate::extensions::memory::extension::HostMemoryExtension;
use crate::extensions::memory::production::{
    create_production_memory_synthesis, start_production_memory_extension,
};
use crate::extensions::notify_bus::extension::{
    HostNotifyBusExtension, start_notify_bus_extension,
};
use crate::extensions::source_map::extension::start_source_map_extension;
use crate::extensions::source_map::source_map_service::SandSourceMap;
use crate::extensions::secrets::extension::{
    HostSecretsExtension, start_secrets_extension,
};
use crate::extensions::settings::extension::start_settings_extension;
use crate::extensions::settings::settings_service::{SettingsService, SettingsSubscription};
use crate::extensions::state_backstop::extension::{
    HostStateBackstopExtension, start_state_backstop_extension,
};
use crate::extensions::state_backstop::state_backstop_service::{
    StateBackstopObjectStore, StateBackstopOptions,
};
use crate::extensions::telemetry::extension::{
    HostTelemetryExtension, start_host_telemetry_extension,
};
use crate::extensions::telemetry::memory_synthesis_telemetry::MemorySynthesisReport;
use crate::extensions::telemetry::conversation_gc_telemetry::ConversationGcReport;
use crate::extensions::telemetry::host_diagnostic_telemetry::HostDiagnostic;
use crate::extensions::telemetry::host_event_bus_telemetry::HostEventBusReport;
use crate::extensions::telemetry::disk_pressure_telemetry::DiskPressureReport;
use crate::extensions::telemetry::session_diagnostic_telemetry::{
    SessionDiagnosticFamily, SessionTelemetryDiagnostic,
};
use crate::extensions::telemetry::webauthn_proxy_telemetry::{
    WebAuthnProxyReport,
};
use crate::extensions::trays::extension::{
    HostTraysExtension, start_trays_extension,
};
use crate::extensions::webauthn_proxy::extension::{
    HostWebAuthnProxyExtension, start_webauthn_proxy_extension,
};
use crate::extensions::wallpaper::extension::{
    HostWallpaperExtension, start_wallpaper_extension,
};
use crate::host_event_bus::{SandHostEventBus, pin_host_event_failure_reporter};
use crate::production_binding_providers::{
    create_production_state_backstop_runtime, production_cloud_agent_trace_converter,
    production_secrets_log,
};
use crate::r#box::production::ProductionBoxEnvironment;
use crate::extensions::box_store_sync::box_object_store::BoxObjectStore;

/// Grok-shaped owner for the production extension subset that is already
/// shipping in the Rust Host.
///
/// This is deliberately not presented as the complete frozen 35-slot registry:
/// missing extension implementations stay visible in the architecture manifest
/// until their real production owners exist.
pub const CURRENT_SHIPPING_PRODUCTION_EXTENSION_IDS: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Automations,
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
    HostExtensionId::Mcp,
    HostExtensionId::SourceMap,
    HostExtensionId::Trays,
    HostExtensionId::BoxLifecycle,
    HostExtensionId::BoxStoreSync,
    HostExtensionId::StateBackstop,
    HostExtensionId::WebauthnProxy,
    HostExtensionId::BrowserUa,
    HostExtensionId::LocalToolPermission,
    HostExtensionId::LocalExec,
    HostExtensionId::Inference,
    HostExtensionId::Wallpaper,
    HostExtensionId::ForeverBox,
    HostExtensionId::Attachments,
    HostExtensionId::Secrets,
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

struct ProductionStateBackstopObjectStore {
    store: Result<Arc<dyn BoxObjectStore>, String>,
}

impl ProductionStateBackstopObjectStore {
    fn new(store: Result<Arc<dyn BoxObjectStore>, String>) -> Self {
        Self { store }
    }

    fn store(&self) -> Result<&Arc<dyn BoxObjectStore>, String> {
        self.store.as_ref().map_err(Clone::clone)
    }
}

impl StateBackstopObjectStore for ProductionStateBackstopObjectStore {
    fn put(&self, path: &str, bytes: &[u8]) -> Result<(), String> {
        self.store()?.put(path, bytes)
    }

    fn get(&self, path: &str) -> Result<Option<Vec<u8>>, String> {
        self.store()?.get(path)
    }
}

fn diagnostic_string(
    values: &impl DiagnosticValues,
    names: &[&str],
) -> Option<String> {
    names.iter().find_map(|name| values.string_value(name))
}

trait DiagnosticValues {
    fn string_value(&self, name: &str) -> Option<String>;
    fn integer_value(&self, name: &str) -> Option<i64>;
    fn float_value(&self, name: &str) -> Option<f64>;
}

impl DiagnosticValues for serde_json::Map<String, serde_json::Value> {
    fn string_value(&self, name: &str) -> Option<String> {
        self.get(name).and_then(|value| match value {
            serde_json::Value::String(value) => Some(value.clone()),
            serde_json::Value::Number(value) => Some(value.to_string()),
            serde_json::Value::Bool(value) => Some(value.to_string()),
            _ => None,
        })
    }

    fn integer_value(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(serde_json::Value::as_i64)
    }

    fn float_value(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(serde_json::Value::as_f64)
    }
}

impl DiagnosticValues for BTreeMap<String, serde_json::Value> {
    fn string_value(&self, name: &str) -> Option<String> {
        self.get(name).and_then(|value| match value {
            serde_json::Value::String(value) => Some(value.clone()),
            serde_json::Value::Number(value) => Some(value.to_string()),
            serde_json::Value::Bool(value) => Some(value.to_string()),
            _ => None,
        })
    }

    fn integer_value(&self, name: &str) -> Option<i64> {
        self.get(name).and_then(serde_json::Value::as_i64)
    }

    fn float_value(&self, name: &str) -> Option<f64> {
        self.get(name).and_then(serde_json::Value::as_f64)
}
}

pub fn project_box_disk_pressure(report: &DomainHostDiagnostic) -> Option<DiskPressureReport> {
    if report.kind != "disk_pressure" {
        return None;
    }
    Some(DiskPressureReport {
        level: diagnostic_string(&report.fields, &["level"] )?,
        volume: diagnostic_string(&report.fields, &["volume"] )?,
        trigger: diagnostic_string(&report.fields, &["trigger"] )?,
        total_bytes: report.fields.float_value("totalBytes")
            .or_else(|| report.fields.float_value("total_bytes"))?,
        available_bytes: report.fields.float_value("availableBytes")
            .or_else(|| report.fields.float_value("available_bytes"))?,
        used_percent: report.fields.float_value("usedPercent")
            .or_else(|| report.fields.float_value("used_percent"))?,
    })
}


pub fn project_host_event_bus_failure(report: &serde_json::Value) -> Option<HostEventBusReport> {
    let object = report.as_object()?;
    Some(HostEventBusReport {
        kind: object.get("kind")?.as_str()?.to_string(),
        topic: object.get("topic").and_then(serde_json::Value::as_str).map(str::to_string),
        error_class: object
            .get("errorClass")
            .or_else(|| object.get("error_class"))?
            .as_str()?
            .to_string(),
    })
}

pub fn project_host_diagnostic(report: &DomainHostDiagnostic) -> HostDiagnostic {
    HostDiagnostic {
        kind: report.kind.clone(),
        stage: diagnostic_string(&report.fields, &["stage"]),
        agent_id: diagnostic_string(&report.fields, &["agentId", "agent_id"]),
        reason: diagnostic_string(&report.fields, &["reason"]),
        error_class: diagnostic_string(&report.fields, &["errorClass", "error_class"]),
    }
}

pub fn project_session_diagnostic(
    report: &DomainSessionDiagnostic,
) -> Option<SessionTelemetryDiagnostic> {
    let family = match report.family.as_str() {
        "store_db" => SessionDiagnosticFamily::StoreDb,
        "maintenance" => SessionDiagnosticFamily::Maintenance,
        "materialize" => SessionDiagnosticFamily::Materialize,
        "summary_build" => SessionDiagnosticFamily::SummaryBuild,
        _ => return None,
    };
    Some(SessionTelemetryDiagnostic {
        family,
        kind: report.kind.clone(),
        agent_id: diagnostic_string(&report.metadata, &["agentId", "agent_id"]),
        error_class: diagnostic_string(&report.metadata, &["errorClass", "error_class"]),
        outcome: diagnostic_string(&report.metadata, &["outcome"]),
        quarantine: diagnostic_string(&report.metadata, &["quarantine"]),
        salvaged_kv: report.metadata.integer_value("salvagedKv")
            .or_else(|| report.metadata.integer_value("salvaged_kv")),
        salvaged_blobs: report.metadata.integer_value("salvagedBlobs")
            .or_else(|| report.metadata.integer_value("salvaged_blobs")),
        salvaged_transcript: report.metadata.integer_value("salvagedTranscript")
            .or_else(|| report.metadata.integer_value("salvaged_transcript")),
    })
}

pub fn project_conversation_gc(report: &DomainConversationGcReport) -> ConversationGcReport {
    match report.outcome.as_str() {
        "failed" => ConversationGcReport::Failed {
            trigger: report.trigger.clone(),
            agent_id: report.agent_id.clone(),
        },
        "skipped" => ConversationGcReport::Skipped {
            trigger: report.trigger.clone(),
            agent_id: report.agent_id.clone(),
            skip_reason: report.skip_reason.clone().unwrap_or_else(|| "unknown".into()),
            unresolved_proto_refs: report.unresolved_proto_refs.map(|value| value as f64),
        },
        _ => ConversationGcReport::Collected {
            trigger: report.trigger.clone(),
            agent_id: report.agent_id.clone(),
            still_over_cap: report.still_over_cap,
            deleted_rows: report.deleted_rows.unwrap_or_default() as f64,
            deleted_bytes: report.deleted_bytes.unwrap_or_default() as f64,
            live_rows: report.live_rows.unwrap_or_default() as f64,
            live_bytes: report.live_bytes.unwrap_or_default() as f64,
            vacuumed: report.vacuumed.unwrap_or(false),
        },
    }
}

fn pin_structured_log_domain_reporters(logs: crate::extensions::telemetry::host_telemetry_service::HostStructuredLogTelemetry) {
    let event_logs = logs.clone();
    pin_host_event_failure_reporter(Some(Arc::new(move |report| {
        if let Some(projected) = project_host_event_bus_failure(report) {
            let _ = event_logs.report_host_event_bus_failure(&projected);
        }
    })));
    let host_logs = logs.clone();
    pin_host_diagnostics_reporter(Some(Arc::new(move |report| {
        if let Some(disk_pressure) = project_box_disk_pressure(report) {
            let _ = host_logs.report_box_disk_pressure(&disk_pressure);
        } else {
            let _ = host_logs.report_host_diagnostic(&project_host_diagnostic(report));
        }
    })));
    let session_logs = logs.clone();
    pin_session_diagnostics_reporter(Some(Arc::new(move |report| {
        if let Some(projected) = project_session_diagnostic(report) {
            let _ = session_logs.report_session_diagnostic(&projected);
        }
    })));
    pin_conversation_gc_reporter(Some(Arc::new(move |report| {
        let _ = logs.report_conversation_gc(&project_conversation_gc(report));
    })));
}

fn unpin_structured_log_domain_reporters() {
    pin_host_event_failure_reporter(None);
    pin_conversation_gc_reporter(None);
    pin_session_diagnostics_reporter(None);
    pin_host_diagnostics_reporter(None);
}

pub struct ProductionHostExtensions {
    pub telemetry: HostTelemetryExtension,
    pub auth: Arc<HostAuthExtension>,
    pub settings: Arc<SettingsService>,
    pub experiments: Arc<HostExperimentsExtension>,
    pub browser_ua: Mutex<Option<BrowserUaExtensionRuntime>>,
    pub wallpaper: Mutex<Option<HostWallpaperExtension>>,
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
    backend_url: String,
    mcp: Mutex<Option<McpExtensionRuntime>>,
    box_store_sync: Mutex<Option<BoxStoreSyncExtension<ProductionBoxStoreSyncService>>>,
    state_backstop: Mutex<Option<HostStateBackstopExtension>>,
    automations: Mutex<Option<Arc<ProductionAutomationsLifecycle>>>,
    // Keep the Experiments lifecycle owner last so dependent extensions settle first.
    _statsig_bootstrap: ProductionStatsigBootstrapRuntime,
}

impl Drop for ProductionHostExtensions {
    fn drop(&mut self) {
        // Stop global reporter callbacks before disposing their single Host owner.
        unpin_structured_log_domain_reporters();
        // Dependents settle while Auth / Experiments / Inference remain live.
        self.cloud_agents.stop();
        self.telemetry.dispose();
    }
}

pub fn start_production_host_extensions(
    app_data_dir: &Path,
    events: SandHostEventBus,
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
    let settings = start_settings_extension();
    let experiments = Arc::new(start_host_experiments_extension());
    let statsig_bootstrap = start_authenticated_statsig_bootstrap(
        Arc::clone(&experiments),
        Arc::clone(&auth),
        backend_url.clone(),
        get_sand_root_dir(),
    )?;
    let browser_ua = start_production_browser_ua(
        Arc::clone(&auth),
        Arc::clone(&experiments),
    );
    let wallpaper = start_wallpaper_extension(
        Arc::clone(&settings),
        Arc::new(|message| eprintln!("mahayana-host-wallpaper {message}")),
    );
    let inference = Arc::new(start_production_inference_extension(
        Arc::clone(&auth),
        Arc::clone(&experiments),
        Arc::clone(&settings),
    ));
    let telemetry = start_host_telemetry_extension(
        app_data_dir,
        backend_url.clone(),
        Arc::clone(&auth),
        Arc::clone(&experiments),
        Arc::clone(&inference),
    )
    .map_err(|error| error.to_string())?;
    let telemetry_logs = telemetry.logs.clone();
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
    let memory_for_gate = memory.clone();
    let inference_for_memory = Arc::clone(&inference);
    let memory_data_dir = app_data_dir.to_path_buf();
    let memory_logs = telemetry.logs.clone();
    experiments.pin_gate_on_authenticated_bootstrap(
        "sand_memory_dreaming",
        Arc::new(move |enabled| {
            if !enabled {
                let _ = memory_logs.report_memory_synthesis(&MemorySynthesisReport::SkippedGate);
                return;
            }
            let synthesis = create_production_memory_synthesis(
                memory_for_gate.service(),
                Arc::clone(&inference_for_memory),
                memory_data_dir.clone(),
                memory_logs.clone(),
            );
            memory_for_gate.enable_memory_synthesis(synthesis);
        }),
    );
    let managed_setup = start_managed_setup_extension(
        backend_url.clone(),
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
    let webauthn_logs = telemetry.logs.clone();
    let webauthn_proxy = Arc::new(start_webauthn_proxy_extension(Arc::new(
        move |report: WebAuthnProxyReport| {
            if let Err(error) = webauthn_logs.report_webauthn_proxy(&report) {
                eprintln!("mahayana-host webauthn_proxy_telemetry_failed error={error}");
            }
        },
    )));

    pin_structured_log_domain_reporters(telemetry.logs.clone());

    Ok(ProductionHostExtensions {
        telemetry,
        auth,
        settings,
        experiments,
        browser_ua: Mutex::new(Some(browser_ua)),
        wallpaper: Mutex::new(Some(wallpaper)),
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
        backend_url,
        mcp: Mutex::new(None),
        box_store_sync: Mutex::new(None),
        state_backstop: Mutex::new(None),
        automations: Mutex::new(None),
        _statsig_bootstrap: statsig_bootstrap,
    })
}


impl ProductionHostExtensions {
    pub fn stop_structured_log_domain_reporters(&self) {
        unpin_structured_log_domain_reporters();
    }

    pub fn stop_cloud_agents(&self) {
        self.cloud_agents.stop();
    }

    pub fn start_automations(
        &self,
        listeners: RelayListeners,
        relay_sink: RelayEventSink,
        fire_dispatch: FireDispatch,
        on_connected: ListenerConnectedCallback,
        agent_channels: AgentChannelsReader,
        cloud_definitions: CloudDefinitionsReader,
        cloud_agent_ids: CloudAgentIdsReader,
        cloud_time_zone: CloudTimeZoneReader,
        log: ProductionLog,
    ) -> Result<Arc<ProductionAutomationsLifecycle>, String> {
        let mut slot = self
            .automations
            .lock()
            .map_err(|_| "production Automations runtime lock poisoned".to_string())?;
        if slot.is_some() {
            return Err("production Automations runtime is already started".into());
        }
        let fire_definitions = Arc::clone(&cloud_definitions);
        let fire_inference = Arc::clone(&self.inference);
        let fire_poll_state: FirePollStateReader = Arc::new(move || {
            let has_server_schedulable = fire_definitions()
                .map(|entries| {
                    entries.into_iter().any(|(_, automation)| {
                        automation.is_enabled && is_server_schedulable(&automation.trigger)
                    })
                })
                .unwrap_or(false);
            (fire_inference.is_ready(), has_server_schedulable)
        });
        let runtime = ProductionAutomationsLifecycle::start(
            self.backend_url.clone(),
            Arc::clone(&self.auth),
            self.notify_bus.clone(),
            listeners,
            relay_sink,
            fire_dispatch,
            on_connected,
            agent_channels,
            cloud_definitions,
            cloud_agent_ids,
            cloud_time_zone,
            fire_poll_state,
            Arc::clone(&self.trays),
            self.telemetry.logs.clone(),
            log,
        )?;
        *slot = Some(Arc::clone(&runtime));
        Ok(runtime)
    }

    pub fn stop_automations(&self) -> Result<(), String> {
        let runtime = self
            .automations
            .lock()
            .map_err(|_| "production Automations runtime lock poisoned".to_string())?
            .take();
        if let Some(runtime) = runtime {
            runtime.stop();
        }
        Ok(())
    }

    pub fn start_mcp(
        &self,
        sand_root_dir: &Path,
        relay: Arc<CoordinatorMcpLifecycleRelay>,
        backend: Arc<dyn McpManagerBackend>,
    ) -> Result<Arc<McpHostService>, String> {
        let mut slot = self.mcp.lock().map_err(|_| "production MCP runtime lock poisoned".to_string())?;
        if slot.is_some() {
            return Err("production MCP runtime is already started".into());
        }

        let loader: Arc<dyn PluginSkillsLoader> =
            Arc::new(CoordinatorPluginSkillsLoader::new(relay));
        let plugin_sync_logs = self.telemetry.logs.clone();
        let plugin_skills = Arc::new(
            SandPluginSkillsService::new(sand_root_dir.to_path_buf(), loader).with_reporter(
                Arc::new(move |report| {
                    let _ = plugin_sync_logs.report_plugin_skills_sync(
                        &report.trigger,
                        &report.outcome,
                        report.changed,
                        report.skill_count,
                        report.error_class.as_deref(),
                        report.duration_ms,
                    );
                }),
            ),
        );
        let plugin_port: Arc<dyn PluginSkillsPort> = plugin_skills.clone();
        let backend_for_sweep = Arc::clone(&backend);
        let service = Arc::new(McpHostService::new(backend, Some(plugin_port)));

        let (cleanup_outcome, removed_credentials) =
            cleanup_legacy_mcp_auth_credentials(sand_root_dir);
        let cleanup_tag = if cleanup_outcome == LegacyMcpAuthCleanupOutcome::Error {
            "error"
        } else {
            "ok"
        };
        let _ = self
            .telemetry
            .logs
            .report_mcp_auth_cleanup(cleanup_tag, removed_credentials);
        if cleanup_outcome == LegacyMcpAuthCleanupOutcome::Error {
            eprintln!(
                "MCP legacy auth cleanup completed with errors after removing {removed_credentials} file(s)"
            );
        }

        let auth: Arc<dyn AuthRenewalPort> = self.auth.clone();
        let plugin_runtime: Arc<dyn McpPluginSkillsService> = plugin_skills.clone();
        let polling: Arc<dyn PollingPort> = Arc::new(RealPluginSkillsPolling::daily());
        let sweep_root = sand_root_dir.to_path_buf();
        let on_startup_sync_succeeded: Arc<dyn Fn() + Send + Sync> = Arc::new(move || {
            let Ok(catalog) = backend_for_sweep.list_catalog(false) else {
                eprintln!("MCP legacy live-reference sweep could not load the marketplace catalog");
                return;
            };
            let catalog = catalog
                .into_iter()
                .map(|plugin| PluginCatalogEntry {
                    plugin_id: plugin.id,
                    skills: plugin
                        .skills
                        .into_iter()
                        .map(|skill| PluginSkillCatalogEntry {
                            source_url: skill.source_url,
                        })
                        .collect(),
                })
                .collect::<Vec<_>>();
            let removed = sweep_legacy_plugin_skill_references(&sweep_root, &catalog);
            if removed > 0 {
                eprintln!(
                    "[sand:plugin-skills] retired {removed} legacy live-reference record(s)"
                );
            }
        });
        let startup = PluginSkillsAuthenticatedStartup::start(
            auth,
            plugin_runtime,
            polling,
            Some(on_startup_sync_succeeded),
        );
        let skill_edge_logs = self.telemetry.logs.clone();
        let skill_publish = Arc::new(
            create_production_skill_publish(
                sand_root_dir.to_path_buf(),
                self.backend_url.clone(),
                Arc::clone(&self.auth),
                plugin_skills,
            )
            .with_edge_reporter(Arc::new(move |failure| {
                let _ = skill_edge_logs
                    .report_skill_publish_edge_failed(failure.stage, &failure.error_class);
            })),
        );
        *slot = Some(McpExtensionRuntime::new(
            startup,
            service.clone(),
            skill_publish,
        ));
        Ok(service)
    }

    pub fn stop_mcp(&self) -> Result<(), String> {
        let runtime = self
            .mcp
            .lock()
            .map_err(|_| "production MCP runtime lock poisoned".to_string())?
            .take();
        runtime.map(|runtime| runtime.stop()).unwrap_or(Ok(()))
    }

    pub fn start_box_store_sync(
        &self,
        is_idle: Arc<dyn Fn() -> bool + Send + Sync>,
    ) -> Result<(), String> {
        let mut slot = self
            .box_store_sync
            .lock()
            .map_err(|_| "production BoxStoreSync runtime lock poisoned".to_string())?;
        if slot.is_some() {
            return Err("production BoxStoreSync runtime is already started".into());
        }
        if self
            .mcp
            .lock()
            .map_err(|_| "production MCP runtime lock poisoned".to_string())?
            .is_none()
        {
            return Err("production BoxStoreSync requires MCP to be started first".into());
        }
        let logs = self.telemetry.logs.clone();
        let diagnostic_logs = logs.clone();
        let box_store_sync_logs = logs.clone();
        let box_store_db_capture_logs = logs.clone();
        let box_store_manifest_conflict_logs = logs.clone();
        let source_map = Arc::clone(&self.source_map);
        let token_auth = Arc::clone(&self.auth);
        let machine_auth = Arc::clone(&self.auth);
        let extension = start_box_store_sync_extension(
            &ProductionBoxStoreSyncFactory,
            BoxStoreSyncExtensionDeps {
                is_idle,
                resolve_store_id: Arc::new(move || {
                    source_map
                        .get_or_create_box_store()
                        .map(|entry| entry.source_id)
                        .map_err(|error| error.to_string())
                }),
                backend_url: Some(self.backend_url.clone()),
                get_access_token: Some(Arc::new(move || {
                    token_auth.get_access_token().map_err(|error| error.to_string())
                })),
                get_machine_id: Some(Arc::new(move || {
                    machine_auth.get_machine_id().map_err(|error| error.to_string())
                })),
                log: Arc::new(move |message| eprintln!("{message}")),
                report_host_extension_diagnostic: Arc::new(move |diagnostic| {
                    let _ = diagnostic_logs.report_host_extension_diagnostic(
                        &serde_json::Value::Object(diagnostic.clone()),
                    );
                }),
                report_box_store_sync_cycle: Arc::new(move |level, metadata| {
                    let _ = box_store_sync_logs.report_box_store_sync_cycle(level, metadata);
                }),
                report_box_store_db_capture: Arc::new(move |level, metadata| {
                    let _ = box_store_db_capture_logs.report_box_store_db_capture(level, metadata);
                }),
                report_box_store_manifest_conflict: Arc::new(move |level, metadata| {
                    let _ = box_store_manifest_conflict_logs
                        .report_box_store_manifest_conflict(level, metadata);
                }),
                scheduling: BoxStoreScheduling::default(),
            },
        );
        let box_api = extension.api();
        *slot = Some(extension);
        drop(slot);

        let mut backstop_slot = self
            .state_backstop
            .lock()
            .map_err(|_| "production StateBackstop runtime lock poisoned".to_string())?;
        if backstop_slot.is_some() {
            return Err("production StateBackstop runtime is already started".into());
        }
        let runtime = create_production_state_backstop_runtime();
        let store_api = box_api.clone();
        let provider = Arc::new(move |source_id: &str| -> Arc<dyn StateBackstopObjectStore> {
            Arc::new(ProductionStateBackstopObjectStore::new(
                store_api.object_store_for(source_id),
            ))
        });
        let source_map = Arc::clone(&self.source_map);
        let source_id_for_agent = Arc::new(move |agent_id: &str| {
            source_map
                .get_or_create(agent_id)
                .map(|entry| entry.source_id)
                .map_err(|error| error.to_string())
        });
        let options = StateBackstopOptions::new(
            provider,
            source_id_for_agent,
            runtime.agents_root_dir,
            runtime.read_db_bytes,
        );
        let state_backstop = start_state_backstop_extension(options);
        if state_backstop.is_enabled() {
            eprintln!("SandState S3 backstop enabled");
        }
        *backstop_slot = Some(state_backstop);
        Ok(())
    }

    pub fn box_store_sync_api(&self) -> Result<Option<ProductionBoxStoreSyncApi>, String> {
        let slot = self
            .box_store_sync
            .lock()
            .map_err(|_| "production BoxStoreSync runtime lock poisoned".to_string())?;
        Ok(slot.as_ref().map(|extension| extension.api()))
    }

    pub fn stop_box_store_sync(&self) -> Result<(), String> {
        let state_backstop = self
            .state_backstop
            .lock()
            .map_err(|_| "production StateBackstop runtime lock poisoned".to_string())?
            .take();
        if let Some(state_backstop) = state_backstop {
            state_backstop.stop();
        }
        let extension = self
            .box_store_sync
            .lock()
            .map_err(|_| "production BoxStoreSync runtime lock poisoned".to_string())?
            .take();
        drop(extension);
        Ok(())
    }
}


/// Second production-extension stage. These frozen Grok slots require the
/// platform Box environment, so they are composed only after the core Host
/// extensions exist. Keeping this stage in the same composition module avoids
/// giving app/main.rs a second extension registry.
pub struct ProductionHostBoxExtensions {
    pub forever_box: Arc<ForeverBoxService>,
    pub attachments: HostAttachmentsExtension,
    pub secrets: Arc<HostSecretsExtension>,
}

impl ProductionHostBoxExtensions {
    pub fn stop(&self) {
        self.secrets.stop();
        self.forever_box.dispose();
    }
}

pub fn start_production_host_box_extensions(
    core: &ProductionHostExtensions,
    environment: ProductionBoxEnvironment,
) -> ProductionHostBoxExtensions {
    let lifecycle: Arc<dyn ForeverBoxLifecycle> = core.box_lifecycle.clone();
    let forever_box = start_forever_box_extension(
        environment,
        lifecycle,
        ForeverBoxExtensionOptions::from_process_env(),
    );
    let attachment_logs = core.telemetry.logs.clone();
    let attachments = start_attachments_extension(
        Arc::clone(&core.auth),
        Arc::clone(&forever_box),
        Some(Arc::new(move |diagnostic| {
            let _ = attachment_logs.report_host_extension_diagnostic(&diagnostic);
        })),
    );
    let secrets = Arc::new(start_secrets_extension(
        Arc::clone(&forever_box),
        production_secrets_log(),
    ));
    ProductionHostBoxExtensions {
        forever_box,
        attachments,
        secrets,
    }
}
