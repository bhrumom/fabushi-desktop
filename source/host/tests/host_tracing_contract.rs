use std::collections::BTreeMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::telemetry::host_tracing::{
    ExportResultCode, HostTracerProvider, HostTracing, HostTracingInitializer,
    ProductionHostTracerProvider, SpanExporter,
    SpanExporterFactory, SpanExporterOptions, TokenRefreshingSpanExporter,
    host_tracing_base_headers, host_tracing_resource, trace_export_url,
};

#[derive(Default)]
struct Exporter {
    exports: AtomicUsize,
    shutdowns: AtomicUsize,
    flushes: AtomicUsize,
}

impl SpanExporter for Exporter {
    fn export(&self, _spans: &[serde_json::Value]) -> ExportResultCode {
        self.exports.fetch_add(1, Ordering::SeqCst);
        ExportResultCode::Success
    }
    fn shutdown(&self) -> Result<(), String> {
        self.shutdowns.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn force_flush(&self) -> Result<(), String> {
        self.flushes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[derive(Default)]
struct Provider {
    registers: AtomicUsize,
    flushes: AtomicUsize,
    shutdowns: AtomicUsize,
}
impl HostTracerProvider for Provider {
    fn register(&self) -> Result<(), String> {
        self.registers.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn force_flush(&self) -> Result<(), String> {
        self.flushes.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
    fn shutdown(&self) -> Result<(), String> {
        self.shutdowns.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

#[test]
fn token_refresh_replaces_delegate_and_keeps_last_delegate_when_token_is_missing() {
    let token = Arc::new(Mutex::new(Some("a".to_string())));
    let created = Arc::new(Mutex::new(
        Vec::<(SpanExporterOptions, Arc<Exporter>)>::new(),
    ));
    let factory: SpanExporterFactory = {
        let created = created.clone();
        Arc::new(move |options| {
            let exporter = Arc::new(Exporter::default());
            created.lock().unwrap().push((options, exporter.clone()));
            exporter
        })
    };
    let exporter = TokenRefreshingSpanExporter::new(
        "https://example.test/v1/traces",
        {
            let token = token.clone();
            Arc::new(move || token.lock().unwrap().clone())
        },
        BTreeMap::from([("x-test".into(), "1".into())]),
        false,
        factory,
    );
    assert_eq!(exporter.export(&[]), ExportResultCode::Success);
    assert_eq!(created.lock().unwrap().len(), 1);
    assert_eq!(
        created.lock().unwrap()[0]
            .0
            .headers
            .get("authorization")
            .map(String::as_str),
        Some("Bearer a")
    );

    *token.lock().unwrap() = None;
    assert_eq!(exporter.export(&[]), ExportResultCode::Success);
    assert_eq!(created.lock().unwrap().len(), 1);

    *token.lock().unwrap() = Some("b".into());
    assert_eq!(exporter.export(&[]), ExportResultCode::Success);
    assert_eq!(created.lock().unwrap().len(), 2);
    assert_eq!(
        created.lock().unwrap()[0]
            .1
            .shutdowns
            .load(Ordering::SeqCst),
        1
    );
}

#[test]
fn no_token_without_previous_delegate_fails_closed() {
    let factory: SpanExporterFactory = Arc::new(|_| Arc::new(Exporter::default()));
    let exporter = TokenRefreshingSpanExporter::new(
        "https://example.test/v1/traces",
        Arc::new(|| None),
        BTreeMap::new(),
        false,
        factory,
    );
    assert_eq!(exporter.export(&[]), ExportResultCode::Failed);
}

#[test]
fn tracing_resource_headers_url_and_lifecycle_match_frozen_contract() {
    assert_eq!(
        trace_export_url("https://backend///"),
        "https://backend/v1/traces"
    );
    let headers = host_tracing_base_headers();
    assert_eq!(
        headers.get("x-ghost-mode").map(String::as_str),
        Some("false")
    );
    assert_eq!(
        host_tracing_resource(Some("1.2.3"))
            .attributes
            .get("service.version")
            .map(String::as_str),
        Some("1.2.3")
    );

    let provider = Arc::new(Provider::default());
    let concrete_exporter = Arc::new(Exporter::default());
    let factory: SpanExporterFactory = {
        let concrete_exporter = concrete_exporter.clone();
        Arc::new(move |_| concrete_exporter.clone())
    };
    let exporter = Arc::new(TokenRefreshingSpanExporter::new(
        "https://backend/v1/traces",
        Arc::new(|| Some("token".into())),
        headers,
        false,
        factory,
    ));
    exporter.resolve_delegate();
    let tracing = HostTracing::new(provider.clone(), exporter).unwrap();
    tracing.flush();
    tracing.dispose();
    tracing.dispose();
    assert_eq!(provider.registers.load(Ordering::SeqCst), 1);
    assert_eq!(provider.flushes.load(Ordering::SeqCst), 1);
    assert_eq!(provider.shutdowns.load(Ordering::SeqCst), 1);
    assert_eq!(concrete_exporter.shutdowns.load(Ordering::SeqCst), 1);
}

#[test]
fn initializer_is_singleton_until_test_reset() {
    let initializer = HostTracingInitializer::default();
    let builds = AtomicUsize::new(0);
    let build = || {
        builds.fetch_add(1, Ordering::SeqCst);
        let provider = Arc::new(Provider::default());
        let exporter = Arc::new(TokenRefreshingSpanExporter::new(
            "https://backend/v1/traces",
            Arc::new(|| None),
            BTreeMap::new(),
            false,
            Arc::new(|_| Arc::new(Exporter::default())),
        ));
        HostTracing::new(provider, exporter)
    };
    let first = initializer.init(build).unwrap();
    let second = initializer.init(|| panic!("must not build twice")).unwrap();
    assert!(Arc::ptr_eq(&first, &second));
    assert_eq!(builds.load(Ordering::SeqCst), 1);
    initializer.reset_for_tests();
}


#[test]
fn production_provider_registers_send_trace_factory_and_flushes_finished_spans() {
    use mahayana_host_runtime::send_trace_host::{BeginTurnTraceOptions, begin_turn_trace};

    let concrete_exporter = Arc::new(Exporter::default());
    let factory: SpanExporterFactory = {
        let concrete_exporter = Arc::clone(&concrete_exporter);
        Arc::new(move |_| Arc::clone(&concrete_exporter) as Arc<dyn SpanExporter>)
    };
    let exporter = Arc::new(TokenRefreshingSpanExporter::new(
        "https://backend/v1/traces",
        Arc::new(|| Some("token".into())),
        BTreeMap::new(),
        false,
        factory,
    ));
    exporter.resolve_delegate();
    let provider = ProductionHostTracerProvider::new(Arc::clone(&exporter));
    let tracing = HostTracing::new(provider, exporter).expect("production tracing");

    let trace = begin_turn_trace(BeginTurnTraceOptions {
        conversation_id: "agent-trace".into(),
        turn_type: "user".into(),
        parent_ctx: None,
        start_time: None,
        sample_ratio: Some(1.0),
        attributes: BTreeMap::new(),
    })
    .expect("registered trace factory");
    trace.span.set_attribute("sand.test", serde_json::json!("yes"));
    trace.span.end();
    tracing.flush();

    assert_eq!(concrete_exporter.exports.load(Ordering::SeqCst), 1);
    tracing.dispose();
}
