use std::collections::BTreeMap;
use std::fs;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::experiments::{
    ExperimentDiagnostic, PRE_PIN_DIAGNOSTICS_BUFFER_CAP,
    pin_experiments_diagnostics_reporter, report_experiments_diagnostic,
};
use mahayana_host_runtime::extensions::telemetry::extension::{
    FATAL_TELEMETRY_FLUSH_TIMEOUT, HOST_CRASH_MARKER_FORWARD_INTERVAL,
    TELEMETRY_DEPENDENCIES, TELEMETRY_EXTENSION_ID, TELEMETRY_FLUSH_TICK,
    telemetry_identity_tags_from,
};
use mahayana_host_runtime::extensions::telemetry::host_telemetry_service::{
    HostTelemetryService, PersistedHostTelemetryRecord,
};

const PRODUCTION_OWNER: &str = include_str!("../src/host_production_extensions.rs");
const SHIPPING_HOST: &str = include_str!("../app/src/main.rs");
const TELEMETRY_EXTENSION: &str = include_str!("../src/extensions/telemetry/extension.rs");

fn temp_root() -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-telemetry-extension-parity-{}-{suffix}",
        std::process::id()
    ))
}

#[test]
fn telemetry_identity_dependencies_and_frozen_scheduling_are_explicit() {
    assert_eq!(TELEMETRY_EXTENSION_ID, "telemetry");
    assert_eq!(
        TELEMETRY_DEPENDENCIES,
        &[
            HostExtensionId::Auth,
            HostExtensionId::Experiments,
            HostExtensionId::Inference,
        ]
    );
    assert_eq!(TELEMETRY_FLUSH_TICK, Duration::from_millis(3_000));
    assert_eq!(
        HOST_CRASH_MARKER_FORWARD_INTERVAL,
        Duration::from_millis(5 * 60_000)
    );
    assert_eq!(
        FATAL_TELEMETRY_FLUSH_TIMEOUT,
        Duration::from_millis(2_000)
    );
}

#[test]
fn telemetry_identity_includes_frozen_store_backend_policy_kind() {
    let v2 = telemetry_identity_tags_from(&BTreeMap::from([
        ("SAND_BOX_AUTH_ID".to_string(), "box-auth".to_string()),
        ("SAND_BOX_STORE_BACKEND".to_string(), "v2".to_string()),
    ]));
    assert_eq!(v2.get("auth_id").map(String::as_str), Some("box-auth"));
    assert_eq!(
        v2.get("store_backend").map(String::as_str),
        Some("sand-box-store-v2")
    );

    let local_dir = std::env::temp_dir()
        .join("fabushi-telemetry-store")
        .to_string_lossy()
        .to_string();
    let local = telemetry_identity_tags_from(&BTreeMap::from([(
        "SAND_BOX_STORE_LOCAL_DIR".to_string(),
        local_dir,
    )]));
    assert_eq!(local.get("store_backend").map(String::as_str), Some("local-fs"));

    let fallback = telemetry_identity_tags_from(&BTreeMap::new());
    assert_eq!(
        fallback.get("store_backend").map(String::as_str),
        Some("agent-store")
    );
}

#[test]
fn shipping_owner_starts_telemetry_after_auth_experiments_and_inference() {
    let auth = PRODUCTION_OWNER.find("let auth = Arc::new(").expect("Auth start");
    let experiments = PRODUCTION_OWNER
        .find("let experiments = Arc::new(start_host_experiments_extension())")
        .expect("Experiments start");
    let inference = PRODUCTION_OWNER
        .find("let inference = Arc::new(start_production_inference_extension(")
        .expect("Inference start");
    let telemetry = PRODUCTION_OWNER
        .find("let telemetry = start_host_telemetry_extension(")
        .expect("Telemetry start");
    assert!(auth < experiments && experiments < inference && inference < telemetry);

    for needle in [
        "Arc::clone(&auth)",
        "Arc::clone(&experiments)",
        "Arc::clone(&inference)",
        "self.telemetry.dispose()",
    ] {
        assert!(
            PRODUCTION_OWNER.contains(needle),
            "production Telemetry graph must retain frozen dependency/lifecycle wiring: {needle}"
        );
    }
}

#[test]
fn telemetry_extension_owns_start_dispose_polling_diagnostics_and_subscription_cleanup() {
    for needle in [
        "service.start()?",
        "StructuredLogFlushPolling::start(service.logs.clone())",
        "HostCrashMarkerForwarder::start(service.logs.clone())",
        "ProductionAnalyticsRuntime::start(",
        "inference.on_model_experiment_applied(",
        "pin_experiments_diagnostics_reporter(Some(",
        "pin_experiments_diagnostics_reporter(None)",
        "fn flush_for_fatal_exit(&self)",
        "recv_timeout(FATAL_TELEMETRY_FLUSH_TIMEOUT)",
        "stop_rx.recv_timeout(HOST_CRASH_MARKER_FORWARD_INTERVAL)",
        "shipper.stop_polling()",
        "shipper.checkpoint_offsets()",
        "self.service.logs.flush()",
        "self.service.dispose()",
    ] {
        assert!(
            TELEMETRY_EXTENSION.contains(needle),
            "Telemetry lifecycle must own frozen behavior: {needle}"
        );
    }
}

#[test]
fn experiments_diagnostics_pin_flushes_the_frozen_pre_pin_buffer() {
    pin_experiments_diagnostics_reporter(None);
    for index in 0..(PRE_PIN_DIAGNOSTICS_BUFFER_CAP + 3) {
        report_experiments_diagnostic(ExperimentDiagnostic {
            kind: format!("buffered-{index}"),
            ..ExperimentDiagnostic::default()
        });
    }
    let observed = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&observed);
    pin_experiments_diagnostics_reporter(Some(Arc::new(move |diagnostic| {
        sink.lock().expect("diagnostic sink").push(diagnostic.kind);
    })));
    let observed = observed.lock().expect("observed diagnostics");
    assert_eq!(observed.len(), PRE_PIN_DIAGNOSTICS_BUFFER_CAP);
    assert_eq!(observed.first().map(String::as_str), Some("buffered-0"));
    assert_eq!(
        observed.last().map(String::as_str),
        Some("buffered-63")
    );
    drop(observed);
    pin_experiments_diagnostics_reporter(None);
}

#[test]
fn service_start_and_dispose_are_idempotent_and_flush_durable_records() {
    let root = temp_root();
    let path = root.join("host-events.jsonl");
    let service = HostTelemetryService::open(&path).expect("service");
    service.start().expect("first start");
    service.start().expect("second start");
    assert!(service.is_started());
    assert!(!service.is_disposed());

    service
        .logs
        .report_experiments_diagnostic(&ExperimentDiagnostic {
            kind: "bootstrap_resolved".into(),
            authenticated: Some(true),
            ..ExperimentDiagnostic::default()
        })
        .expect("diagnostic report");
    service.dispose().expect("first dispose");
    service.dispose().expect("second dispose");
    assert!(service.is_disposed());

    let text = fs::read_to_string(&path).expect("durable telemetry");
    let record: PersistedHostTelemetryRecord =
        serde_json::from_str(text.lines().next().expect("record")).expect("record json");
    assert_eq!(record.event, "sand.experiments.diagnostic");
    assert_eq!(record.payload["level"], "info");
    assert_eq!(record.payload["metadata"]["kind"], "bootstrap_resolved");
    assert_eq!(record.payload["metadata"]["authenticated"], "true");
    let _ = fs::remove_dir_all(root);
}

#[test]
fn fatal_flush_and_normal_shutdown_are_shipping_wired_once() {
    assert!(SHIPPING_HOST.contains("_process_crash_guard.set_reporter(Some("));
    assert!(SHIPPING_HOST.contains("fatal_telemetry.flush_for_fatal_exit()"));
    assert!(SHIPPING_HOST.contains("production_extensions.telemetry.dispose()"));
    assert_eq!(
        SHIPPING_HOST.matches("fatal_telemetry.flush_for_fatal_exit()").count(),
        1
    );
}
