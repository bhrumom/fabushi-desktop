use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, OnceLock, Weak};
use std::time::{SystemTime, UNIX_EPOCH};

use reqwest::blocking::Client;
use serde_json::{Value, json};

use crate::send_trace_host::{
    HostTrace, TraceContext, TraceFactoryOptions, TraceSpan, clear_host_trace_factory,
    mint_traceparent, parse_traceparent, set_host_trace_factory,
};

use serde_json::Value;

pub const SAND_CLIENT_TYPE: &str = "sand";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportResultCode {
    Success,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpanExporterOptions {
    pub url: String,
    pub headers: BTreeMap<String, String>,
    pub reject_unauthorized: bool,
}

pub trait SpanExporter: Send + Sync {
    fn export(&self, spans: &[Value]) -> ExportResultCode;
    fn shutdown(&self) -> Result<(), String>;
    fn force_flush(&self) -> Result<(), String> {
        Ok(())
    }
}

pub type SpanExporterFactory =
    Arc<dyn Fn(SpanExporterOptions) -> Arc<dyn SpanExporter> + Send + Sync>;

pub struct TokenRefreshingSpanExporter {
    url: String,
    get_token: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    base_headers: BTreeMap<String, String>,
    insecure: bool,
    create_exporter: SpanExporterFactory,
    delegate: Mutex<Option<(String, Arc<dyn SpanExporter>)>>,
}

impl TokenRefreshingSpanExporter {
    pub fn new(
        url: impl Into<String>,
        get_token: Arc<dyn Fn() -> Option<String> + Send + Sync>,
        base_headers: BTreeMap<String, String>,
        insecure: bool,
        create_exporter: SpanExporterFactory,
    ) -> Self {
        Self {
            url: url.into(),
            get_token,
            base_headers,
            insecure,
            create_exporter,
            delegate: Mutex::new(None),
        }
    }

    pub fn resolve_delegate(&self) -> Option<Arc<dyn SpanExporter>> {
        let token = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| (self.get_token)()))
            .ok()
            .flatten()
            .filter(|value| !value.is_empty());

        let mut delegate = self
            .delegate
            .lock()
            .expect("host tracing exporter poisoned");
        let Some(token) = token else {
            return delegate.as_ref().map(|(_, exporter)| Arc::clone(exporter));
        };

        if delegate
            .as_ref()
            .is_none_or(|(previous, _)| previous != &token)
        {
            let mut headers = self.base_headers.clone();
            headers.insert("authorization".into(), format!("Bearer {token}"));
            let next = (self.create_exporter)(SpanExporterOptions {
                url: self.url.clone(),
                headers,
                reject_unauthorized: !self.insecure,
            });
            if let Some((_, previous)) = delegate.replace((token, Arc::clone(&next))) {
                let _ =
                    std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| previous.shutdown()));
            }
            return Some(next);
        }
        delegate.as_ref().map(|(_, exporter)| Arc::clone(exporter))
    }

    pub fn export(&self, spans: &[Value]) -> ExportResultCode {
        let Some(delegate) = self.resolve_delegate() else {
            return ExportResultCode::Failed;
        };
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| delegate.export(spans)))
            .unwrap_or(ExportResultCode::Failed)
    }

    pub fn shutdown(&self) {
        if let Some((_, delegate)) = self
            .delegate
            .lock()
            .expect("host tracing exporter poisoned")
            .take()
        {
            let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| delegate.shutdown()));
        }
    }

    pub fn force_flush(&self) {
        if let Some((_, delegate)) = self
            .delegate
            .lock()
            .expect("host tracing exporter poisoned")
            .as_ref()
        {
            let _ =
                std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| delegate.force_flush()));
        }
    }
}

pub trait HostTracerProvider: Send + Sync {
    fn register(&self) -> Result<(), String>;
    fn force_flush(&self) -> Result<(), String>;
    fn shutdown(&self) -> Result<(), String>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HostTracingResource {
    pub attributes: BTreeMap<String, String>,
}

pub fn host_tracing_resource(service_version: Option<&str>) -> HostTracingResource {
    let mut attributes = BTreeMap::new();
    attributes.insert("service.name".into(), "sand-host".into());
    attributes.insert(
        "service.version".into(),
        service_version.unwrap_or("unknown").into(),
    );
    attributes.insert(
        "host.name".into(),
        std::env::var("HOSTNAME").unwrap_or_else(|_| "unknown".into()),
    );
    attributes.insert("os.type".into(), std::env::consts::OS.into());
    attributes.insert("process.runtime.name".into(), "rust".into());
    attributes.insert(
        "process.runtime.version".into(),
        option_env!("RUSTC_VERSION").unwrap_or("unknown").into(),
    );
    attributes.insert("deployment.environment".into(), "box".into());
    HostTracingResource { attributes }
}

pub fn host_tracing_base_headers() -> BTreeMap<String, String> {
    BTreeMap::from([
        ("x-ghost-mode".into(), "false".into()),
        ("x-cursor-client-type".into(), SAND_CLIENT_TYPE.into()),
        ("x-cursor-client-version".into(), "sand-host".into()),
    ])
}

pub fn trace_export_url(backend_url: &str) -> String {
    format!("{}/v1/traces", backend_url.trim_end_matches('/'))
}

pub struct HostTracing {
    provider: Arc<dyn HostTracerProvider>,
    exporter: Arc<TokenRefreshingSpanExporter>,
    disposed: Mutex<bool>,
}

impl HostTracing {
    pub fn new(
        provider: Arc<dyn HostTracerProvider>,
        exporter: Arc<TokenRefreshingSpanExporter>,
    ) -> Result<Self, String> {
        provider.register()?;
        Ok(Self {
            provider,
            exporter,
            disposed: Mutex::new(false),
        })
    }

    pub fn flush(&self) {
        let _ =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.provider.force_flush()));
        self.exporter.force_flush();
    }

    pub fn dispose(&self) {
        let mut disposed = self.disposed.lock().expect("host tracing state poisoned");
        if *disposed {
            return;
        }
        *disposed = true;
        let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| self.provider.shutdown()));
        self.exporter.shutdown();
    }

    pub fn is_disposed(&self) -> bool {
        *self.disposed.lock().expect("host tracing state poisoned")
    }
}

pub struct HostTracingInitializer {
    initialized: Mutex<Option<Arc<HostTracing>>>,
}

impl Default for HostTracingInitializer {
    fn default() -> Self {
        Self {
            initialized: Mutex::new(None),
        }
    }
}

impl HostTracingInitializer {
    pub fn init(
        &self,
        build: impl FnOnce() -> Result<HostTracing, String>,
    ) -> Option<Arc<HostTracing>> {
        let mut initialized = self.initialized.lock().expect("host tracing init poisoned");
        if let Some(existing) = initialized.as_ref() {
            return Some(Arc::clone(existing));
        }
        let tracing = Arc::new(build().ok()?);
        *initialized = Some(Arc::clone(&tracing));
        Some(tracing)
    }

    pub fn reset_for_tests(&self) {
        self.initialized
            .lock()
            .expect("host tracing init poisoned")
            .take();
    }
}


#[derive(Debug)]
struct ProductionTraceContext {
    trace_id: String,
    span_id: String,
}

struct ProductionSpanState {
    attributes: BTreeMap<String, Value>,
    exception: Option<String>,
    status_code: u8,
}

struct ProductionTraceSpan {
    provider: Weak<ProductionHostTracerProvider>,
    context: Arc<ProductionTraceContext>,
    parent_span_id: Option<String>,
    name: String,
    start_unix_nanos: u128,
    ended: AtomicBool,
    state: Mutex<ProductionSpanState>,
}

impl TraceSpan for ProductionTraceSpan {
    fn set_attribute(&self, key: &str, value: Value) {
        if self.ended.load(Ordering::Acquire) {
            return;
        }
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .attributes
            .insert(key.to_string(), value);
    }

    fn record_exception(&self, error: &str) {
        if self.ended.load(Ordering::Acquire) {
            return;
        }
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .exception = Some(error.to_string());
    }

    fn set_status(&self, code: u8) {
        if self.ended.load(Ordering::Acquire) {
            return;
        }
        self.state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .status_code = code;
    }

    fn end(&self) {
        if self.ended.swap(true, Ordering::AcqRel) {
            return;
        }
        let Some(provider) = self.provider.upgrade() else {
            return;
        };
        let state = self
            .state
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        let attributes = state
            .attributes
            .iter()
            .map(|(key, value)| json!({
                "key": key,
                "value": otlp_any_value(value),
            }))
            .collect::<Vec<_>>();
        let mut span = json!({
            "traceId": self.context.trace_id,
            "spanId": self.context.span_id,
            "name": self.name,
            "kind": 1,
            "startTimeUnixNano": self.start_unix_nanos.to_string(),
            "endTimeUnixNano": unix_nanos().to_string(),
            "attributes": attributes,
            "status": {
                "code": state.status_code,
            },
        });
        if let Some(parent_span_id) = self.parent_span_id.as_deref() {
            span["parentSpanId"] = Value::String(parent_span_id.to_string());
        }
        if let Some(exception) = state.exception.as_deref() {
            span["events"] = json!([{
                "timeUnixNano": unix_nanos().to_string(),
                "name": "exception",
                "attributes": [{
                    "key": "exception.message",
                    "value": {"stringValue": exception},
                }],
            }]);
        }
        provider.enqueue(span);
    }
}

fn unix_nanos() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos()
}

fn otlp_any_value(value: &Value) -> Value {
    match value {
        Value::Bool(value) => json!({"boolValue": value}),
        Value::Number(value) => {
            if let Some(integer) = value.as_i64() {
                json!({"intValue": integer.to_string()})
            } else if let Some(float) = value.as_f64() {
                json!({"doubleValue": float})
            } else {
                json!({"stringValue": value.to_string()})
            }
        }
        Value::String(value) => json!({"stringValue": value}),
        _ => json!({"stringValue": value.to_string()}),
    }
}

pub struct ProductionHostTracerProvider {
    exporter: Arc<TokenRefreshingSpanExporter>,
    queue: Mutex<Vec<Value>>,
    self_ref: Weak<ProductionHostTracerProvider>,
    registered: AtomicBool,
    shutdown: AtomicBool,
}

impl ProductionHostTracerProvider {
    pub fn new(exporter: Arc<TokenRefreshingSpanExporter>) -> Arc<Self> {
        Arc::new_cyclic(|weak| Self {
            exporter,
            queue: Mutex::new(Vec::new()),
            self_ref: weak.clone(),
            registered: AtomicBool::new(false),
            shutdown: AtomicBool::new(false),
        })
    }

    fn enqueue(&self, span: Value) {
        if self.shutdown.load(Ordering::Acquire) {
            return;
        }
        self.queue
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .push(span);
    }

    fn make_trace(&self, options: TraceFactoryOptions) -> Option<HostTrace> {
        if self.shutdown.load(Ordering::Acquire) {
            return None;
        }

        let mut parent_span_id = None;
        let (trace_id, span_id) = if let Some(parent) = options
            .parent_ctx
            .as_ref()
            .and_then(|value| value.as_ref().downcast_ref::<ProductionTraceContext>())
        {
            parent_span_id = Some(parent.span_id.clone());
            let minted = mint_traceparent(true);
            let child = parse_traceparent(&minted.traceparent)?;
            (parent.trace_id.clone(), child.span_id)
        } else if let Some(remote) = options
            .traceparent
            .as_deref()
            .and_then(parse_traceparent)
        {
            parent_span_id = Some(remote.span_id);
            let minted = mint_traceparent(remote.trace_flags & 1 == 1);
            let child = parse_traceparent(&minted.traceparent)?;
            (remote.trace_id, child.span_id)
        } else {
            let minted = mint_traceparent(true);
            let parsed = parse_traceparent(&minted.traceparent)?;
            (parsed.trace_id, parsed.span_id)
        };

        let context = Arc::new(ProductionTraceContext { trace_id, span_id });
        let span = Arc::new(ProductionTraceSpan {
            provider: self.self_ref.clone(),
            context: Arc::clone(&context),
            parent_span_id,
            name: options.name,
            start_unix_nanos: options
                .start_time
                .filter(|value| value.is_finite() && *value >= 0.0)
                .map(|value| (value * 1_000_000.0).round() as u128)
                .unwrap_or_else(unix_nanos),
            ended: AtomicBool::new(false),
            state: Mutex::new(ProductionSpanState {
                attributes: options.inheritable_attributes,
                exception: None,
                status_code: 0,
            }),
        });
        let trace_context: TraceContext = context;
        Some(HostTrace {
            span,
            context: Some(trace_context),
        })
    }

    fn flush_queue(&self) -> Result<(), String> {
        let spans = {
            let mut queue = self
                .queue
                .lock()
                .map_err(|_| "host tracing queue mutex poisoned".to_string())?;
            if queue.is_empty() {
                return Ok(());
            }
            std::mem::take(&mut *queue)
        };
        match self.exporter.export(&spans) {
            ExportResultCode::Success => Ok(()),
            ExportResultCode::Failed => Err("host tracing exporter failed".into()),
        }
    }
}

impl HostTracerProvider for ProductionHostTracerProvider {
    fn register(&self) -> Result<(), String> {
        if self.registered.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let weak = self.self_ref.clone();
        set_host_trace_factory(Arc::new(move |options| {
            weak.upgrade()?.make_trace(options)
        }));
        Ok(())
    }

    fn force_flush(&self) -> Result<(), String> {
        self.flush_queue()
    }

    fn shutdown(&self) -> Result<(), String> {
        if self.shutdown.swap(true, Ordering::AcqRel) {
            return Ok(());
        }
        let result = self.flush_queue();
        clear_host_trace_factory();
        result
    }
}

struct OtlpJsonHttpSpanExporter {
    options: SpanExporterOptions,
    resource: HostTracingResource,
    client: Client,
}

impl OtlpJsonHttpSpanExporter {
    fn new(options: SpanExporterOptions, resource: HostTracingResource) -> Result<Self, String> {
        let client = Client::builder()
            .danger_accept_invalid_certs(!options.reject_unauthorized)
            .build()
            .map_err(|error| error.to_string())?;
        Ok(Self {
            options,
            resource,
            client,
        })
    }

    fn body(&self, spans: &[Value]) -> Value {
        let attributes = self
            .resource
            .attributes
            .iter()
            .map(|(key, value)| json!({
                "key": key,
                "value": {"stringValue": value},
            }))
            .collect::<Vec<_>>();
        json!({
            "resourceSpans": [{
                "resource": {"attributes": attributes},
                "scopeSpans": [{
                    "scope": {"name": "sand-host"},
                    "spans": spans,
                }],
            }],
        })
    }
}

impl SpanExporter for OtlpJsonHttpSpanExporter {
    fn export(&self, spans: &[Value]) -> ExportResultCode {
        if spans.is_empty() {
            return ExportResultCode::Success;
        }
        let mut request = self
            .client
            .post(&self.options.url)
            .header("content-type", "application/json");
        for (key, value) in &self.options.headers {
            request = request.header(key.as_str(), value.as_str());
        }
        match request.json(&self.body(spans)).send() {
            Ok(response) if response.status().is_success() => ExportResultCode::Success,
            _ => ExportResultCode::Failed,
        }
    }

    fn shutdown(&self) -> Result<(), String> {
        Ok(())
    }
}

fn production_host_tracing_initializer() -> &'static HostTracingInitializer {
    static INITIALIZER: OnceLock<HostTracingInitializer> = OnceLock::new();
    INITIALIZER.get_or_init(HostTracingInitializer::default)
}

pub fn init_production_host_tracing(
    backend_url: impl Into<String>,
    get_token: Arc<dyn Fn() -> Option<String> + Send + Sync>,
    service_version: Option<&str>,
) -> Option<Arc<HostTracing>> {
    let backend_url = backend_url.into();
    let service_version = service_version.map(str::to_string);
    production_host_tracing_initializer().init(move || {
        let resource = host_tracing_resource(service_version.as_deref());
        let resource_for_factory = resource.clone();
        let factory: SpanExporterFactory = Arc::new(move |options| {
            OtlpJsonHttpSpanExporter::new(options.clone(), resource_for_factory.clone())
                .map(|exporter| Arc::new(exporter) as Arc<dyn SpanExporter>)
                .unwrap_or_else(|_| Arc::new(FailedSpanExporter))
        });
        let exporter = Arc::new(TokenRefreshingSpanExporter::new(
            trace_export_url(&backend_url),
            get_token,
            host_tracing_base_headers(),
            std::env::var("SAND_INSECURE_BACKEND_TLS").as_deref() == Ok("1"),
            factory,
        ));
        let provider = ProductionHostTracerProvider::new(Arc::clone(&exporter));
        HostTracing::new(provider, exporter)
    })
}

struct FailedSpanExporter;

impl SpanExporter for FailedSpanExporter {
    fn export(&self, _spans: &[Value]) -> ExportResultCode {
        ExportResultCode::Failed
    }

    fn shutdown(&self) -> Result<(), String> {
        Ok(())
    }
}
