//! Grok-aligned Mahayana Host process entrypoint.
//!
//! The shipping desktop Host process is owned by `source/host/app`. During the
//! migration, `mahayana-unified-app-host` remains an internal compatibility
//! backend so existing product commands keep working while Grok Host/Runner
//! modules are moved behind this process boundary. Electron must never launch
//! the legacy third_party desktop Host binary directly.

use mahayana_host_runtime::extensions::auth::auth_service::HostAuthServiceOptions;
use mahayana_host_runtime::extensions::auth::extension::{
    HostAuthExtension, start_host_auth_extension_with_options,
};
use mahayana_host_runtime::extensions::auth::user_full_name_service::production_user_full_name_fetch;
use mahayana_host_runtime::extensions::box_lifecycle::box_lifecycle_service::BoxLifecycleService;
use mahayana_host_runtime::extensions::box_lifecycle::extension::start_box_lifecycle_extension;
use mahayana_host_runtime::extensions::box_lifecycle::production::{
    ProductionBoxLifecycleClient, ProductionBoxLifecycleClientFactory,
};
use mahayana_host_runtime::extensions::session::production::ProductionSessionWorkers;
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderMessage, ProviderSessionError, RoutedProvider, RoutedToolDefinition,
};
use mahayana_host_runtime::extensions::managed_setup::team_rules::ProductionTeamRulesResolver;
use mahayana_host_runtime::extensions::auth::credential_renewer::RenewalOutcome;
use mahayana_host_runtime::host_request_context::create_host_request_context;
use mahayana_host_runtime::runner::routed_provider_runtime::{
    RoutedProviderRun, RoutedProviderTaskRegistry, RoutedToolBridge,
    RunnerRequestContextSnapshot, RunnerRequestContextSource,
    run_routed_provider_in_runner,
};
use mahayana_host_runtime::gateway_config::{gateway_scheme, resolve_gateway_server_config};
use mahayana_host_runtime::gateway_server::{
    GatewayApi, GatewayCommandError, GatewayCommandReport, GatewayEventHub, GatewayServerDeps, start_gateway_server,
};
use mahayana_host_runtime::host_discovery::{
    GatewayDiscoveryInfo, clear_gateway_discovery, write_gateway_discovery,
};
use mahayana_host_runtime::host_lock::acquire_host_lock;
use mahayana_host_runtime::host_paths::{get_gateway_discovery_path, get_host_lock_path};
use mahayana_host_runtime::r#box::box_env::BoxEnvironmentUpdate;
use mahayana_host_runtime::r#box::production::{
    BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD, ProductionBoxEnvironment,
};
use mahayana_unified_app_host::{
    PlatformRequestHost, UnifiedAppHost, default_unified_app_data_dir, dispatch_json,
    is_platform_request_json,
};
use std::collections::BTreeMap;
use std::fs;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

fn ensure_managed_runtime_layout(app_data_dir: &Path) -> io::Result<()> {
    // The desktop product owns this fallback workspace.  It must exist before
    // the native engine canonicalizes the path while opening the first Agent
    // session.  User-selected workspace paths are validated elsewhere and are
    // never created implicitly.
    fs::create_dir_all(app_data_dir.join("feature-host/runtime/workspace"))
}


struct ProductionHostExtensions {
    auth: Arc<HostAuthExtension>,
    team_rules: Arc<ProductionTeamRulesResolver>,
    team_rules_renewal_subscription: Option<u64>,
    _box_lifecycle: BoxLifecycleService<ProductionBoxLifecycleClient<HostAuthExtension>>,
}

impl Drop for ProductionHostExtensions {
    fn drop(&mut self) {
        if let Some(subscription) = self.team_rules_renewal_subscription.take() {
            self.auth.service().unsubscribe_from_renewal(subscription);
        }
    }
}

struct ProductionRunnerRequestContextSource {
    auth: Arc<HostAuthExtension>,
    team_rules: Arc<ProductionTeamRulesResolver>,
    transcripts_folder: PathBuf,
}

impl RunnerRequestContextSource for ProductionRunnerRequestContextSource {
    fn resolve(&self) -> RunnerRequestContextSnapshot {
        let rules = self.team_rules.resolve_rules();
        let provider = create_host_request_context(
            self.transcripts_folder.to_string_lossy().into_owned(),
            || None,
            || rules.clone(),
            || self.auth.get_user_full_name(),
        );
        RunnerRequestContextSnapshot {
            context: provider.resolve(),
            rules: provider.resolve_rules(),
        }
    }
}

fn start_production_host_extensions() -> Result<ProductionHostExtensions, String> {
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
    let team_rules = Arc::new(ProductionTeamRulesResolver::new(
        backend_url,
        Arc::clone(&auth),
    ));
    team_rules.preload();
    let weak_team_rules = Arc::downgrade(&team_rules);
    let team_rules_renewal_subscription =
        auth.service().subscribe_to_renewal(Arc::new(move |event| {
            if event.result.outcome != RenewalOutcome::Renewed {
                return;
            }
            let Some(resolver) = weak_team_rules.upgrade() else {
                return;
            };
            let _ = thread::Builder::new()
                .name("host-managed-team-rules-renewal".into())
                .spawn(move || {
                    let _ = resolver.refresh();
                });
        }));

    let factory =
        ProductionBoxLifecycleClientFactory::from_process_env().map_err(|error| error.to_string())?;
    let box_lifecycle = start_box_lifecycle_extension(Arc::clone(&auth), &factory);

    Ok(ProductionHostExtensions {
        auth,
        team_rules,
        team_rules_renewal_subscription: Some(team_rules_renewal_subscription),
        _box_lifecycle: box_lifecycle,
    })
}

enum HostLaneRequest {
    Stdin(String),
    Gateway {
        method: String,
        args: serde_json::Value,
        reply: mpsc::SyncSender<Result<serde_json::Value, GatewayCommandError>>,
    },
    StdinClosed,
}

const RUNNER_START_ROUTED_PROVIDER_GATEWAY_METHOD: &str = "runner.startRoutedProvider";
const RUNNER_CANCEL_ROUTED_PROVIDER_GATEWAY_METHOD: &str = "runner.cancelRoutedProvider";
const RUNNER_INFERENCE_EVENT_CHANNEL: &str = "runner-inference";

struct UnifiedGatewayApi {
    host_tx: mpsc::Sender<HostLaneRequest>,
    events: GatewayEventHub,
    data_dir: PathBuf,
    request_context: Arc<dyn RunnerRequestContextSource>,
    session_workers: Arc<ProductionSessionWorkers>,
    routed_provider_tasks: Arc<RoutedProviderTaskRegistry>,
}

fn call_host_lane(
    host_tx: &mpsc::Sender<HostLaneRequest>,
    method: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
    let (reply, result) = mpsc::sync_channel(1);
    host_tx
        .send(HostLaneRequest::Gateway {
            method: method.to_string(),
            args,
            reply,
        })
        .map_err(|_| GatewayCommandError::Internal("Mahayana Host lane is closed".into()))?;
    result
        .recv_timeout(Duration::from_secs(120))
        .map_err(|_| GatewayCommandError::Internal("Mahayana Host gateway request timed out".into()))?
}

#[derive(Clone)]
struct HostLaneRoutedToolBridge {
    host_tx: mpsc::Sender<HostLaneRequest>,
    agent_id: String,
}

impl RoutedToolBridge for HostLaneRoutedToolBridge {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        let value = call_host_lane(&self.host_tx, "listRoutedMcpTools", serde_json::json!({}))
            .map_err(|error| ProviderSessionError::Tool(error.to_string()))?;
        decode_routed_tools(value)
    }

    fn call_tool(
        &self,
        tool: &RoutedToolDefinition,
        args: serde_json::Value,
        tool_call_id: &str,
    ) -> Result<serde_json::Value, ProviderSessionError> {
        call_host_lane(
            &self.host_tx,
            "executeRoutedMcpTool",
            serde_json::json!({
                "providerIdentifier": tool.provider_identifier,
                "name": tool.name,
                "toolName": tool.tool_name,
                "args": args,
                "toolCallId": tool_call_id,
                "agentId": self.agent_id,
            }),
        )
        .map_err(|error| ProviderSessionError::Tool(error.to_string()))
    }
}

fn decode_routed_tools(
    value: serde_json::Value,
) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
    let rows = value.as_array().ok_or_else(|| {
        ProviderSessionError::Protocol("listRoutedMcpTools did not return an array".into())
    })?;
    Ok(rows
        .iter()
        .filter_map(|row| {
            Some(RoutedToolDefinition {
                name: row.get("name")?.as_str()?.to_string(),
                provider_identifier: row.get("providerIdentifier")?.as_str()?.to_string(),
                tool_name: row.get("toolName")?.as_str()?.to_string(),
                description: row.get("description").and_then(serde_json::Value::as_str).map(str::to_string),
                input_schema: row.get("inputSchema").cloned().unwrap_or_else(|| {
                    serde_json::json!({"type":"object","additionalProperties":true})
                }),
            })
        })
        .collect())
}

fn decode_provider_messages(
    args: &serde_json::Value,
) -> Result<Vec<ProviderMessage>, GatewayCommandError> {
    let rows = args
        .get("messages")
        .and_then(serde_json::Value::as_array)
        .ok_or_else(|| GatewayCommandError::Internal(
            "runner.startRoutedProvider requires messages".into()
        ))?;
    let messages = rows
        .iter()
        .filter_map(|row| {
            Some(ProviderMessage {
                role: row.get("role")?.as_str()?.to_string(),
                content: row.get("content")?.as_str()?.to_string(),
            })
        })
        .collect::<Vec<_>>();
    if messages.is_empty() {
        return Err(GatewayCommandError::Internal(
            "runner.startRoutedProvider requires at least one message".into()
        ));
    }
    Ok(messages)
}

fn start_routed_provider_task(
    host_tx: mpsc::Sender<HostLaneRequest>,
    events: GatewayEventHub,
    data_dir: PathBuf,
    request_context: Arc<dyn RunnerRequestContextSource>,
    session_workers: Arc<ProductionSessionWorkers>,
    routed_provider_tasks: Arc<RoutedProviderTaskRegistry>,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
    let provider_name = args.get("provider").and_then(serde_json::Value::as_str).unwrap_or("");
    let provider = RoutedProvider::parse(provider_name)
        .filter(|provider| *provider != RoutedProvider::Cursor)
        .ok_or_else(|| GatewayCommandError::Internal(format!(
            "unsupported routed provider: {provider_name}"
        )))?;
    let agent_id = args.get("agentId").and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| GatewayCommandError::Internal(
            "runner.startRoutedProvider requires agentId".into()
        ))?
        .to_string();
    let stream_id = args.get("streamId").and_then(serde_json::Value::as_str)
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| GatewayCommandError::Internal(
            "runner.startRoutedProvider requires streamId".into()
        ))?
        .to_string();
    let messages = decode_provider_messages(&args)?;
    let cancellation = routed_provider_tasks.register(&stream_id).map_err(|error| {
        GatewayCommandError::Internal(error.to_string())
    })?;
    session_workers.prepare_existing_agent(&agent_id).map_err(|error| {
        GatewayCommandError::Internal(format!(
            "could not prepare production session worker state for {agent_id}: {error}"
        ))
    })?;
    let resolved_request_context = request_context.resolve();
    let worker_events = events.clone();
    let accepted_stream_id = stream_id.clone();
    let worker_stream_id = stream_id.clone();
    let worker_tasks = Arc::clone(&routed_provider_tasks);
    let worker_cancellation = cancellation.clone();
    let spawn = thread::Builder::new()
        .name(format!("mahayana-runner-provider-{agent_id}"))
        .spawn(move || {
            let bridge: Arc<dyn RoutedToolBridge> = Arc::new(HostLaneRoutedToolBridge {
                host_tx,
                agent_id,
            });
            let delta_events = worker_events.clone();
            let delta_stream_id = stream_id.clone();
            let mut on_text_delta = move |_delta: &str, accumulated: &str| {
                delta_events.publish(serde_json::json!({
                    "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                    "payload": {
                        "streamId": delta_stream_id,
                        "type": "delta",
                        "content": accumulated
                    }
                }));
            };
            let result = run_routed_provider_in_runner(
                RoutedProviderRun {
                    provider,
                    data_dir: &data_dir,
                    messages: &messages,
                    bridge,
                    request_context: resolved_request_context,
                    cancellation,
                },
                &mut on_text_delta,
            );
            if !worker_cancellation.is_cancelled() {
                match result {
                    Ok(content) => worker_events.publish(serde_json::json!({
                        "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                        "payload": {
                            "streamId": stream_id,
                            "type": "completed",
                            "content": content
                        }
                    })),
                    Err(error) => worker_events.publish(serde_json::json!({
                        "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                        "payload": {
                            "streamId": stream_id,
                            "type": "failed",
                            "message": error.to_string()
                        }
                    })),
                }
            }
            worker_tasks.finish(&worker_stream_id);
        });
    if let Err(error) = spawn {
        routed_provider_tasks.finish(&accepted_stream_id);
        return Err(GatewayCommandError::Internal(format!(
            "could not start routed provider Runner: {error}"
        )));
    }
    Ok(serde_json::json!({
        "accepted": true,
        "streamId": accepted_stream_id,
        "provider": provider.as_str(),
    }))
}

impl GatewayApi for UnifiedGatewayApi {
    fn call(
        &self,
        method: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, GatewayCommandError> {
        if method == RUNNER_START_ROUTED_PROVIDER_GATEWAY_METHOD {
            return start_routed_provider_task(
                self.host_tx.clone(),
                self.events.clone(),
                self.data_dir.clone(),
                Arc::clone(&self.request_context),
                Arc::clone(&self.session_workers),
                Arc::clone(&self.routed_provider_tasks),
                args,
            );
        }
        if method == RUNNER_CANCEL_ROUTED_PROVIDER_GATEWAY_METHOD {
            let stream_id = args
                .get("streamId")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .ok_or_else(|| GatewayCommandError::Internal(
                    "runner.cancelRoutedProvider requires streamId".into()
                ))?;
            let reason = args
                .get("reason")
                .and_then(serde_json::Value::as_str)
                .filter(|value| !value.trim().is_empty())
                .unwrap_or("Runner provider request cancelled");
            let cancelled = self.routed_provider_tasks.cancel(stream_id, reason);
            if cancelled {
                self.events.publish(serde_json::json!({
                    "channel": RUNNER_INFERENCE_EVENT_CHANNEL,
                    "payload": {
                        "streamId": stream_id,
                        "type": "cancelled",
                        "message": reason
                    }
                }));
            }
            return Ok(serde_json::json!({
                "streamId": stream_id,
                "cancelled": cancelled,
            }));
        }
        // UnifiedAppHost owns a QuickJS runtime and is intentionally !Send.
        // Product calls remain on its owner lane while the Runner provider
        // worker above streams through the Host event hub.
        call_host_lane(&self.host_tx, method, args)
    }

    fn on_command_complete(&self, report: GatewayCommandReport) {
        log_gateway_command_report("complete", &report);
    }

    fn on_command_error(&self, report: GatewayCommandReport) {
        log_gateway_command_report("error", &report);
    }
}

fn log_gateway_command_report(kind: &str, report: &GatewayCommandReport) {
    let mut value = serde_json::json!({
        "kind": kind,
        "method": report.method,
        "durationMs": report.duration_ms,
        "status": report.status,
    });
    if let Some(request_id) = report.request_id.as_deref() {
        value["requestId"] = serde_json::Value::String(request_id.to_string());
    }
    if let Some(traceparent) = report.traceparent.as_deref() {
        value["traceparent"] = serde_json::Value::String(traceparent.to_string());
    }
    if let Some(error) = report.error.as_deref() {
        value["error"] = serde_json::Value::String(error.to_string());
    }
    eprintln!("mahayana-host-gateway-command {value}");
}

fn decode_box_environment_update(
    args: &serde_json::Value,
) -> Result<BoxEnvironmentUpdate, GatewayCommandError> {
    let object = args.as_object().ok_or_else(|| {
        GatewayCommandError::Internal("box.applyEnvironment params must be an object".into())
    })?;
    let raw_env = object.get("env").and_then(serde_json::Value::as_object).ok_or_else(|| {
        GatewayCommandError::Internal("box.applyEnvironment params.env must be an object".into())
    })?;
    let replace = object
        .get("replace")
        .and_then(serde_json::Value::as_bool)
        .ok_or_else(|| {
            GatewayCommandError::Internal("box.applyEnvironment params.replace must be a boolean".into())
        })?;
    let mut env = BTreeMap::new();
    for (name, value) in raw_env {
        let value = value.as_str().ok_or_else(|| {
            GatewayCommandError::Internal(format!(
                "box.applyEnvironment env value for {name} must be a string"
            ))
        })?;
        env.insert(name.clone(), value.to_string());
    }
    Ok(BoxEnvironmentUpdate { env, replace })
}

fn dispatch_box_environment_call<Apply>(
    method: &str,
    args: &serde_json::Value,
    mut apply: Apply,
) -> Option<Result<serde_json::Value, GatewayCommandError>>
where
    Apply: FnMut(&BoxEnvironmentUpdate) -> Result<(), String>,
{
    if method != BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD {
        return None;
    }
    Some(
        decode_box_environment_update(args).and_then(|update| {
            apply(&update)
                .map_err(GatewayCommandError::Internal)
                .map(|()| serde_json::json!({ "applied": true }))
        }),
    )
}

fn dispatch_gateway_call(
    host: &UnifiedAppHost,
    production_box: &ProductionBoxEnvironment,
    method: &str,
    args: serde_json::Value,
) -> Result<serde_json::Value, GatewayCommandError> {
    if let Some(result) = dispatch_box_environment_call(method, &args, |update| {
        production_box
            .apply_environment(update)
            .map_err(|error| error.to_string())
    }) {
        return result;
    }

    let request = serde_json::json!({
        "id": "gateway",
        "method": method,
        "params": args,
    });
    let encoded = serde_json::to_string(&request)
        .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
    let response = dispatch_json(host, &encoded);
    let parsed: serde_json::Value = serde_json::from_str(&response)
        .map_err(|error| GatewayCommandError::Internal(error.to_string()))?;
    if parsed.get("ok").and_then(serde_json::Value::as_bool) == Some(true) {
        return Ok(parsed.get("result").cloned().unwrap_or(serde_json::Value::Null));
    }
    let message = parsed
        .get("error")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("Mahayana Host gateway request failed")
        .to_string();
    if message.contains("unknown") {
        Err(GatewayCommandError::UnknownMethod(method.to_string()))
    } else {
        Err(GatewayCommandError::Internal(message))
    }
}

fn started_at_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u64::MAX as u128) as u64)
        .unwrap_or_default()
}

fn write_response(stdout: &Mutex<io::Stdout>, response: &str) -> io::Result<()> {
    let mut stdout = stdout
        .lock()
        .map_err(|_| io::Error::other("desktop host stdout lock poisoned"))?;
    writeln!(stdout, "{response}")?;
    stdout.flush()
}

fn write_runtime_event(
    stdout: &Mutex<io::Stdout>,
    gateway_events: &GatewayEventHub,
    event: serde_json::Value,
) -> io::Result<()> {
    gateway_events.publish(event.clone());
    let frame = serde_json::json!({ "event": event });
    let encoded = serde_json::to_string(&frame)
        .map_err(|error| io::Error::other(format!("event serialization failed: {error}")))?;
    write_response(stdout, &encoded)
}

fn drain_ready_runtime_events(
    host: &UnifiedAppHost,
    stdout: &Mutex<io::Stdout>,
    gateway_events: &GatewayEventHub,
) -> io::Result<()> {
    loop {
        let event = host.receive_feature_event(Duration::ZERO);
        match event {
            Ok(Some(event)) => write_runtime_event(stdout, gateway_events, event)?,
            Ok(None) => return Ok(()),
            Err(error) => {
                eprintln!("failed to drain Mahayana runtime event: {error}");
                return Ok(());
            }
        }
    }
}

fn main() {
    let app_data_dir = default_unified_app_data_dir();
    if let Err(error) = ensure_managed_runtime_layout(&app_data_dir) {
        eprintln!(
            "failed to initialize managed Mahayana runtime layout at {}: {error}",
            app_data_dir.display()
        );
        std::process::exit(1);
    }

    let host = match UnifiedAppHost::new(app_data_dir.clone()) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("failed to initialize unified Mahayana app host: {error}");
            std::process::exit(1);
        }
    };
    let production_box = ProductionBoxEnvironment::from_process_env();
    let (host_tx, host_rx) = mpsc::channel::<HostLaneRequest>();
    // Platform/account HTTP may legitimately take tens of seconds. Keep it on
    // a dedicated Rust product lane so feature.receive and Agent commands keep
    // flowing through the primary serial Host lane without starvation. The
    // child protocol already correlates responses by id, so out-of-order
    // platform replies are safe.
    let platform_host = match PlatformRequestHost::new(app_data_dir.clone()) {
        Ok(host) => host,
        Err(error) => {
            eprintln!("failed to initialize Mahayana platform request lane: {error}");
            std::process::exit(1);
        }
    };
    let stdout = Arc::new(Mutex::new(io::stdout()));

    let host_lock_path = get_host_lock_path();
    let host_lock = match acquire_host_lock(&host_lock_path, std::process::id()) {
        Ok(acquisition) => acquisition.lock,
        Err(error) => {
            eprintln!(
                "failed to acquire Mahayana Host lock at {}: {error}",
                host_lock_path.display()
            );
            return;
        }
    };

    let production_extensions = match start_production_host_extensions() {
        Ok(extensions) => extensions,
        Err(error) => {
            eprintln!("failed to start production Host extensions: {error}");
            return;
        }
    };
    let runner_request_context: Arc<dyn RunnerRequestContextSource> =
        Arc::new(ProductionRunnerRequestContextSource {
            auth: Arc::clone(&production_extensions.auth),
            team_rules: Arc::clone(&production_extensions.team_rules),
            transcripts_folder: app_data_dir.join("transcripts"),
        });
    let session_workers = Arc::new(ProductionSessionWorkers::production());
    let routed_provider_tasks = Arc::new(RoutedProviderTaskRegistry::default());

    let gateway_config = match resolve_gateway_server_config() {
        Ok(config) => config,
        Err(error) => {
            eprintln!("failed to resolve Mahayana gateway configuration: {error}");
            return;
        }
    };
    let gateway_started_at = started_at_ms();
    let gateway_events = GatewayEventHub::default();
    let gateway_server = match start_gateway_server(GatewayServerDeps {
        api: Arc::new(UnifiedGatewayApi {
            host_tx: host_tx.clone(),
            events: gateway_events.clone(),
            data_dir: app_data_dir.clone(),
            request_context: runner_request_context,
            session_workers: Arc::clone(&session_workers),
            routed_provider_tasks: Arc::clone(&routed_provider_tasks),
        }),
        events: gateway_events.clone(),
        local_exec: None,
        webauthn: None,
        config: gateway_config.clone(),
        started_at: gateway_started_at,
    }) {
        Ok(server) => server,
        Err(error) => {
            eprintln!("failed to start Mahayana Host gateway: {error}");
            return;
        }
    };

    let gateway_discovery_path = get_gateway_discovery_path();
    let gateway_discovery = GatewayDiscoveryInfo {
        port: gateway_server.port(),
        pid: std::process::id(),
        started_at: gateway_started_at,
        scheme: Some(gateway_scheme(&gateway_config).to_string()),
        host: Some(gateway_config.host.clone()),
        token: gateway_config.auth_token.clone(),
    };
    if let Err(error) = write_gateway_discovery(&gateway_discovery, &gateway_discovery_path) {
        eprintln!(
            "failed to publish Mahayana Host gateway discovery at {}: {error}",
            gateway_discovery_path.display()
        );
        return;
    }

    // Runtime events travel as unsolicited JSON frames. The event worker blocks
    // in Rust instead of issuing 500 ms JSON-RPC receive requests from Electron.
    // Test mode has a non-blocking deterministic backend, so a small sleep keeps
    // that lane from spinning while CI is idle.
    let event_source = host.feature_event_source();
    let event_stdout = Arc::clone(&stdout);
    let event_gateway = gateway_events.clone();
    let _event_worker = thread::spawn(move || loop {
        match event_source.receive(Duration::from_secs(30)) {
            Ok(Some(event)) => {
                if write_runtime_event(&event_stdout, &event_gateway, event).is_err() {
                    break;
                }
            }
            Ok(None) => thread::sleep(Duration::from_millis(25)),
            Err(error) => {
                eprintln!("Mahayana runtime event stream failed: {error}");
                thread::sleep(Duration::from_millis(100));
            }
        }
    });

    let platform_stdout = Arc::clone(&stdout);
    let (platform_tx, platform_rx) = mpsc::channel::<String>();
    let _platform_worker = thread::spawn(move || {
        while let Ok(line) = platform_rx.recv() {
            let response = platform_host.dispatch_json(&line);
            if write_response(&platform_stdout, &response).is_err() {
                break;
            }
        }
    });

    // Read stdin on a lightweight transport thread. The UnifiedAppHost itself
    // stays on this owner thread so QuickJS and the rest of the Host runtime
    // never cross a Send/Sync boundary. Gateway requests join the same serial
    // lane through HostLaneRequest.
    let stdin_tx = host_tx.clone();
    let _stdin_worker = thread::spawn(move || {
        let stdin = io::stdin();
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(line) => line,
                Err(error) => {
                    eprintln!("failed to read host request: {error}");
                    break;
                }
            };
            if line.trim().is_empty() {
                continue;
            }
            if stdin_tx.send(HostLaneRequest::Stdin(line)).is_err() {
                return;
            }
        }
        let _ = stdin_tx.send(HostLaneRequest::StdinClosed);
    });
    drop(host_tx);

    while let Ok(request) = host_rx.recv() {
        match request {
            HostLaneRequest::Gateway { method, args, reply } => {
                let _ = reply.send(dispatch_gateway_call(&host, &production_box, &method, args));
            }
            HostLaneRequest::StdinClosed => break,
            HostLaneRequest::Stdin(line) => {
                if is_platform_request_json(&line) {
                    if platform_tx.send(line).is_err() {
                        break;
                    }
                    continue;
                }
                let response = dispatch_json(&host, &line);
                if write_response(&stdout, &response).is_err() {
                    break;
                }
                // Commands may enqueue product-local events that are not backed
                // by the model runtime receiver. Drain those immediately so
                // they are pushed in the same turn.
                if drain_ready_runtime_events(&host, &stdout, &gateway_events).is_err() {
                    break;
                }
            }
        }
    }
    drop(platform_tx);
    drop(gateway_server);
    routed_provider_tasks.cancel_all("Mahayana Host shutting down");
    session_workers.shutdown();
    if let Err(error) = clear_gateway_discovery(&gateway_discovery_path) {
        eprintln!(
            "failed to clear Mahayana Host gateway discovery at {}: {error}",
            gateway_discovery_path.display()
        );
    }
    if let Err(error) = host_lock.release() {
        eprintln!(
            "failed to release Mahayana Host lock at {}: {error}",
            host_lock_path.display()
        );
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD, ProductionHostExtensions,
        ProductionRunnerRequestContextSource, UnifiedGatewayApi, decode_provider_messages,
        dispatch_box_environment_call, ensure_managed_runtime_layout, is_platform_request_json,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn shipping_production_extension_graph_is_send_sync() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<ProductionHostExtensions>();
        assert_send_sync::<ProductionSessionWorkers>();
        assert_send_sync::<RoutedProviderTaskRegistry>();
    }

    #[test]
    fn gateway_proxy_is_send_sync_without_moving_the_unified_host() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UnifiedGatewayApi>();
        assert_send_sync::<ProductionRunnerRequestContextSource>();
    }

    #[test]
    fn routes_platform_http_away_from_the_feature_event_lane() {
        assert!(is_platform_request_json(
            r#"{"id":1,"method":"platform.request","params":{}}"#,
        ));
        assert!(!is_platform_request_json(
            r#"{"id":2,"method":"feature.receive","params":{"timeoutMs":500}}"#,
        ));
        assert!(!is_platform_request_json(
            r#"{"id":3,"method":"feature.execute","params":{}}"#,
        ));
    }



    #[test]
    fn runner_provider_gateway_rejects_empty_message_batches() {
        assert!(decode_provider_messages(&serde_json::json!({"messages": []})).is_err());
        let messages = decode_provider_messages(&serde_json::json!({
            "messages": [{"role":"user","content":"hello"}]
        }))
        .expect("provider messages");
        assert_eq!(messages.len(), 1);
        assert_eq!(messages[0].role, "user");
        assert_eq!(messages[0].content, "hello");
    }

    #[test]
    fn shipping_gateway_routes_box_environment_to_production_box_owner() {
        let args = serde_json::json!({
            "env": { "FABUSHI_AGENT": "enabled", "SHELL": "/bin/zsh" },
            "replace": true
        });
        let mut observed = None;
        let result = dispatch_box_environment_call(
            BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD,
            &args,
            |update| {
                observed = Some(update.clone());
                Ok(())
            },
        )
        .expect("box route should be owned by production Host")
        .expect("box route should succeed");

        assert_eq!(result, serde_json::json!({ "applied": true }));
        let observed = observed.expect("production box update");
        assert_eq!(observed.env["FABUSHI_AGENT"], "enabled");
        assert_eq!(observed.env["SHELL"], "/bin/zsh");
        assert!(observed.replace);
    }

    #[test]
    fn creates_product_owned_fallback_workspace_before_host_startup() {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!(
            "fabushi-mahayana-desktop-layout-{}-{suffix}",
            std::process::id()
        ));

        ensure_managed_runtime_layout(&root).expect("initialize managed runtime layout");
        assert!(root.join("feature-host/runtime/workspace").is_dir());

        fs::remove_dir_all(root).expect("remove test layout");
    }
}
