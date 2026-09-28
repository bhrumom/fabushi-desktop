use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::notify_bus::extension::HostNotifyBusExtension;

use super::backend_transport::ReqwestAutomationsBackendTransport;
use super::connect_unary::{SandConnectAuth, SandConnectUnaryClient};
use super::listener_connect_watcher_production::{
    ListenerConnectedCallback, ProductionListenerConnectWatcher,
};
use super::listener_integrations::{
    PlatformConnectionReader, ProductionListenerIntegrations,
};
use super::production::{
    FireDispatch, ProductionAutomationsBackendRuntime, ProductionLog, RelayEventSink,
    RelayListeners,
};

/// Single shipping owner for the Grok Automations background lifecycle.
///
/// Transcript remains the authority for trigger matching and execution. This
/// owner composes authenticated backend transport, NotifyBus-driven polling,
/// and the integration-connect watcher, and joins every worker on shutdown.
pub struct ProductionAutomationsLifecycle {
    backend: Arc<ProductionAutomationsBackendRuntime>,
    watcher: Arc<ProductionListenerConnectWatcher>,
    integrations: Arc<ProductionListenerIntegrations>,
    stopped: AtomicBool,
}

impl ProductionAutomationsLifecycle {
    pub fn start(
        backend_url: String,
        auth: Arc<HostAuthExtension>,
        notify_bus: HostNotifyBusExtension,
        listeners: RelayListeners,
        relay_sink: RelayEventSink,
        fire_dispatch: FireDispatch,
        on_connected: ListenerConnectedCallback,
        log: ProductionLog,
    ) -> Result<Arc<Self>, String> {
        let transport = Arc::new(
            ReqwestAutomationsBackendTransport::new(backend_url, Arc::clone(&auth))
                .map_err(|error| format!("failed to create Automations backend transport: {error}"))?,
        );
        let connect_auth: Arc<dyn SandConnectAuth> = auth;
        let connect = Arc::new(
            SandConnectUnaryClient::new(connect_auth)
                .map_err(|error| format!("failed to create Automations Connect client: {error}"))?,
        );
        let integrations = Arc::new(ProductionListenerIntegrations::new(connect));
        let connection_reader: Arc<dyn PlatformConnectionReader> = integrations.clone();
        let watcher =
            ProductionListenerConnectWatcher::start(connection_reader, on_connected);
        let backend = ProductionAutomationsBackendRuntime::start(
            transport,
            notify_bus,
            listeners,
            relay_sink,
            fire_dispatch,
            log,
        );

        Ok(Arc::new(Self {
            backend,
            watcher,
            integrations,
            stopped: AtomicBool::new(false),
        }))
    }

    pub fn watch_listener_connection(
        &self,
        agent_id: impl Into<String>,
        platform: impl Into<String>,
    ) {
        if !self.stopped.load(Ordering::Acquire) {
            self.watcher.watch(agent_id, platform);
        }
    }

    pub fn get_connect_url(&self, platform: &str) -> String {
        self.integrations.get_connect_url(platform)
    }

    pub fn request_reconcile(&self) {
        if !self.stopped.load(Ordering::Acquire) {
            self.backend.request_reconcile();
        }
    }

    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::AcqRel) {
            return;
        }
        self.watcher.stop();
        self.backend.stop();
    }

    pub fn is_stopped(&self) -> bool {
        self.stopped.load(Ordering::Acquire)
    }
}

impl Drop for ProductionAutomationsLifecycle {
    fn drop(&mut self) {
        self.stop();
    }
}
