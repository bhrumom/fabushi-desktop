use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

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
