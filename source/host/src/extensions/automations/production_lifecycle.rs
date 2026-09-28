use std::collections::{BTreeMap, BTreeSet};
use std::sync::{
    Arc, Mutex, mpsc,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};

use crate::extensions::auth::extension::HostAuthExtension;
use crate::extensions::browser_ua::extension::{BrowserUaAuthApi, StopSubscription};
use crate::extensions::notify_bus::extension::HostNotifyBusExtension;

use super::backend_transport::ReqwestAutomationsBackendTransport;
use super::connect_unary::{SandConnectAuth, SandConnectUnaryClient};
use super::listener_connect_watcher_production::{
    ListenerConnectedCallback, ProductionListenerConnectWatcher,
};
use super::listener_integrations::{
    AgentChannelsReader, PlatformConnectionReader, ProductionListenerIntegrations,
    listener_agent_channels_view,
};
use super::backend_relay_source::RelayStatus;
use super::production::{
    FireDispatch, ProductionAutomationsBackendRuntime, ProductionLog, RelayEventSink,
    RelayListeners,
};
use super::sand_automation_cloud_sync::{
    CloudSyncClient, ProductionCloudSyncClient, SandAutomationCloudSync,
    ScheduledCloudAutomation, sand_cloud_definition,
};

pub type CloudDefinitionsReader = Arc<
    dyn Fn() -> Result<Vec<(String, ScheduledCloudAutomation)>, String> + Send + Sync + 'static,
>;
pub type CloudAgentIdsReader =
    Arc<dyn Fn() -> Result<Vec<String>, String> + Send + Sync + 'static>;
pub type CloudTimeZoneReader =
    Arc<dyn Fn() -> Option<String> + Send + Sync + 'static>;

/// Single shipping owner for the Grok Automations background lifecycle.
///
/// Transcript remains the authority for trigger matching and execution. This
/// owner composes authenticated backend transport, NotifyBus-driven polling,
/// cloud shadow reconciliation/scheduling authority, and the integration-connect
/// watcher, and joins every worker on shutdown.
pub struct ProductionAutomationsLifecycle {
    backend: Arc<ProductionAutomationsBackendRuntime>,
    watcher: Arc<ProductionListenerConnectWatcher>,
    integrations: Arc<ProductionListenerIntegrations>,
    agent_channels: AgentChannelsReader,
    cloud: Arc<Mutex<SandAutomationCloudSync>>,
    cloud_wake: mpsc::Sender<()>,
    cloud_worker: Mutex<Option<JoinHandle<()>>>,
    cloud_auth_stop: Mutex<Option<StopSubscription>>,
    stopped: Arc<AtomicBool>,
}

impl ProductionAutomationsLifecycle {
    #[allow(clippy::too_many_arguments)]
    pub fn start(
        backend_url: String,
        auth: Arc<HostAuthExtension>,
        notify_bus: HostNotifyBusExtension,
        listeners: RelayListeners,
        relay_sink: RelayEventSink,
        fire_dispatch: FireDispatch,
        on_connected: ListenerConnectedCallback,
        agent_channels: AgentChannelsReader,
        cloud_definitions: CloudDefinitionsReader,
        cloud_agent_ids: CloudAgentIdsReader,
        cloud_time_zone: CloudTimeZoneReader,
        log: ProductionLog,
    ) -> Result<Arc<Self>, String> {
        let transport = Arc::new(
            ReqwestAutomationsBackendTransport::new(backend_url, Arc::clone(&auth))
                .map_err(|error| format!("failed to create Automations backend transport: {error}"))?,
        );
        let connect_auth: Arc<dyn SandConnectAuth> = auth.clone();
        let connect = Arc::new(
            SandConnectUnaryClient::new(connect_auth)
                .map_err(|error| format!("failed to create Automations Connect client: {error}"))?,
        );
        let integrations = Arc::new(ProductionListenerIntegrations::new(Arc::clone(&connect)));
        let connection_reader: Arc<dyn PlatformConnectionReader> = integrations.clone();
        let watcher =
            ProductionListenerConnectWatcher::start(connection_reader, on_connected);
        let backend = ProductionAutomationsBackendRuntime::start(
            transport,
            notify_bus,
            listeners,
            relay_sink,
            fire_dispatch,
            Arc::clone(&log),
        );

        let cloud_client: Arc<dyn CloudSyncClient> =
            Arc::new(ProductionCloudSyncClient::new(connect));
        let cloud = Arc::new(Mutex::new(SandAutomationCloudSync::new(cloud_client)));
        let stopped = Arc::new(AtomicBool::new(false));
        let (cloud_wake, cloud_rx) = mpsc::channel::<()>();

        let worker_cloud = Arc::clone(&cloud);
        let worker_backend = Arc::clone(&backend);
        let worker_auth = Arc::clone(&auth);
        let worker_stopped = Arc::clone(&stopped);
        let worker_log = Arc::clone(&log);
        let worker = thread::Builder::new()
            .name("mahayana-automations-cloud-sync".into())
            .spawn(move || {
                let mut known_agent_ids = BTreeSet::<String>::new();
                loop {
                    if cloud_rx.recv().is_err() || worker_stopped.load(Ordering::Acquire) {
                        break;
                    }
                    while cloud_rx.try_recv().is_ok() {}
                    if worker_stopped.load(Ordering::Acquire) {
                        break;
                    }
                    if worker_auth.peek_access_token().is_none() {
                        continue;
                    }

                    let entries = match cloud_definitions() {
                        Ok(entries) => entries,
                        Err(error) => {
                            worker_log(&format!(
                                "[sand:automations] cloud sync list-local failed: {error}"
                            ));
                            continue;
                        }
                    };
                    let listed_agent_ids = match cloud_agent_ids() {
                        Ok(agent_ids) => agent_ids,
                        Err(error) => {
                            worker_log(&format!(
                                "[sand:automations] cloud sync list-agents failed: {error}"
                            ));
                            continue;
                        }
                    };
                    let time_zone = cloud_time_zone();

                    let mut desired_by_agent =
                        BTreeMap::<String, Vec<super::sand_automation_cloud_sync::CloudDefinition>>::new();
                    let mut current_agent_ids =
                        listed_agent_ids.into_iter().collect::<BTreeSet<_>>();
                    for (agent_id, automation) in entries {
                        current_agent_ids.insert(agent_id.clone());
                        if let Some(definition) =
                            sand_cloud_definition(&agent_id, &automation, time_zone.as_deref())
                        {
                            desired_by_agent
                                .entry(agent_id)
                                .or_default()
                                .push(definition);
                        }
                    }

                    let removed_agent_ids = known_agent_ids
                        .difference(&current_agent_ids)
                        .cloned()
                        .collect::<Vec<_>>();
                    let mut authority_changed = false;
                    {
                        let mut sync = worker_cloud
                            .lock()
                            .unwrap_or_else(|poisoned| poisoned.into_inner());
                        for agent_id in removed_agent_ids {
                            match sync.delete_agent(&agent_id) {
                                Ok(outcome) => {
                                    authority_changed |= outcome.scheduling_authority_changed;
                                    if !outcome.converged {
                                        worker_log(&format!(
                                            "[sand:automations] cloud agent deletion did not converge agent={agent_id}"
                                        ));
                                    }
                                }
                                Err(error) => worker_log(&format!(
                                    "[sand:automations] cloud agent deletion failed agent={agent_id}: {error}"
                                )),
                            }
                        }

                        for agent_id in &current_agent_ids {
                            let desired = desired_by_agent.remove(agent_id).unwrap_or_default();
                            match sync.reconcile_agent(agent_id, desired) {
                                Ok(outcome) => {
                                    authority_changed |= outcome.scheduling_authority_changed;
                                    if !outcome.converged {
                                        worker_log(&format!(
                                            "[sand:automations] cloud reconciliation did not converge agent={agent_id}"
                                        ));
                                    }
                                }
                                Err(error) => worker_log(&format!(
                                    "[sand:automations] cloud reconciliation failed agent={agent_id}: {error}"
                                )),
                            }
                        }
                    }
                    known_agent_ids = current_agent_ids;
                    if authority_changed {
                        worker_backend.request_reconcile();
                    }
                }
            })
            .map_err(|error| format!("failed to start Automations cloud sync worker: {error}"))?;

        let renewal_wake = cloud_wake.clone();
        let cloud_auth_stop = auth.subscribe_to_renewal(Arc::new(move |event| {
            if event.outcome == "renewed" {
                let _ = renewal_wake.send(());
            }
        }));

        let lifecycle = Arc::new(Self {
            backend,
            watcher,
            integrations,
            agent_channels,
            cloud,
            cloud_wake,
            cloud_worker: Mutex::new(Some(worker)),
            cloud_auth_stop: Mutex::new(Some(cloud_auth_stop)),
            stopped,
        });
        let _ = lifecycle.cloud_wake.send(());
        Ok(lifecycle)
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

    pub fn is_platform_connected(&self, platform: &str) -> Result<bool, String> {
        self.integrations
            .is_platform_connected(platform)
            .map_err(|error| error.to_string())
    }

    pub fn listener_source_status(&self, platform: &str) -> Option<RelayStatus> {
        self.backend.listener_source_status(platform)
    }

    pub fn get_agent_channels(&self, agent_id: &str) -> Result<serde_json::Value, String> {
        let channels = (self.agent_channels)(agent_id)?;
        Ok(listener_agent_channels_view(&channels))
    }

    pub fn should_schedule_locally(
        &self,
        agent_id: &str,
        automation: &ScheduledCloudAutomation,
    ) -> bool {
        self.cloud
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .should_schedule_locally(agent_id, automation)
    }

    pub fn request_reconcile(&self) {
        if !self.stopped.load(Ordering::Acquire) {
            self.backend.request_reconcile();
            let _ = self.cloud_wake.send(());
        }
    }

    pub fn stop(&self) {
        if self.stopped.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Some(stop) = self
            .cloud_auth_stop
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            stop();
        }
        let _ = self.cloud_wake.send(());
        if let Some(worker) = self
            .cloud_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = worker.join();
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
