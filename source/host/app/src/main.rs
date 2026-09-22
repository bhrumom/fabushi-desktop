//! Grok-aligned Mahayana Host process entrypoint.
//!
//! The shipping desktop Host process is owned by `source/host/app`. During the
//! migration, `mahayana-unified-app-host` remains an internal compatibility
//! backend so existing product commands keep working while Grok Host/Runner
//! modules are moved behind this process boundary. Electron must never launch
//! the legacy third_party desktop Host binary directly.

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
use std::path::Path;
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

enum HostLaneRequest {
    Stdin(String),
    Gateway {
        method: String,
        args: serde_json::Value,
        reply: mpsc::SyncSender<Result<serde_json::Value, GatewayCommandError>>,
    },
    StdinClosed,
}

struct UnifiedGatewayApi {
    host_tx: mpsc::Sender<HostLaneRequest>,
}

impl GatewayApi for UnifiedGatewayApi {
    fn call(
        &self,
        method: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, GatewayCommandError> {
        // UnifiedAppHost owns a QuickJS runtime and is intentionally !Send.
        // Keep the Host on one owner thread and route gateway calls onto that
        // lane instead of smuggling it across threads behind Arc<Mutex<_>>.
        let (reply, result) = mpsc::sync_channel(1);
        self.host_tx
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
    let platform_host = match PlatformRequestHost::new(app_data_dir) {
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
        BOX_APPLY_ENVIRONMENT_GATEWAY_METHOD, UnifiedGatewayApi,
        dispatch_box_environment_call, ensure_managed_runtime_layout, is_platform_request_json,
    };
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn gateway_proxy_is_send_sync_without_moving_the_unified_host() {
        fn assert_send_sync<T: Send + Sync>() {}
        assert_send_sync::<UnifiedGatewayApi>();
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
