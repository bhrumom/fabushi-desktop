use std::collections::BTreeMap;

use serde_json::Value;

use super::backend_relay_source::BackendRelaySource;
use super::listener_connect_watcher::ListenerConnectWatcher;
use super::listener_integrations::ListenerIntegrations;
use super::sand_automation_cloud_sync::{DesiredCloudTrigger, desired_cloud_triggers};
use super::sand_automation_fire_consumer::{AutomationFireEnvelope, AutomationFireFailure, SandAutomationFireConsumer};
use super::sand_trigger_hub::{ScheduledAutomation, desired_listeners_by_kind, matching_fires};

#[derive(Debug, Default)]
pub struct AutomationExtensionRuntime {
    sources: BTreeMap<String, BackendRelaySource>,
    watcher: ListenerConnectWatcher,
    integrations: ListenerIntegrations,
    consumer: SandAutomationFireConsumer,
    stopped: bool,
}

impl AutomationExtensionRuntime {
    pub fn with_source_kinds(kinds: impl IntoIterator<Item = String>) -> Self {
        let sources = kinds.into_iter()
            .map(|kind| (kind.clone(), BackendRelaySource::new(kind)))
            .collect();
        Self {
            sources,
            watcher: ListenerConnectWatcher::default(),
            integrations: ListenerIntegrations::default(),
            consumer: SandAutomationFireConsumer::default(),
            stopped: false,
        }
    }

    pub fn watcher_mut(&mut self) -> &mut ListenerConnectWatcher {
        &mut self.watcher
    }

    pub fn integrations(&self) -> &ListenerIntegrations {
        &self.integrations
    }

    pub fn set_listener_connected(&mut self, kind: impl Into<String>, connected: bool) {
        self.integrations.set_connected(kind, connected);
    }

    pub fn poll_listener_connections(&mut self, now_ms: u64) -> Vec<(String, String)> {
        let integrations = &self.integrations;
        self.watcher.tick(now_ms, |kind| {
            integrations.is_connected(kind)
                .ok_or_else(|| format!("connection state unavailable for {kind}"))
        })
    }

    pub fn desired_cloud_triggers(
        &self,
        scheduled: &[ScheduledAutomation],
        should_sync: impl Fn(&str, &ScheduledAutomation) -> bool,
    ) -> Vec<DesiredCloudTrigger> {
        if self.stopped {
            return Vec::new();
        }
        desired_cloud_triggers(scheduled, should_sync)
    }

    pub fn stop(&mut self) {
        self.stopped = true;
        self.watcher.dispose();
        for source in self.sources.values_mut() {
            source.stop();
            source.set_listeners(Vec::new());
        }
    }

    pub fn reconcile(
        &mut self,
        scheduled: &[ScheduledAutomation],
        ready: bool,
        should_schedule_locally: impl Fn(&str, &ScheduledAutomation) -> bool,
    ) {
        let desired = if ready && !self.stopped {
            desired_listeners_by_kind(scheduled, should_schedule_locally)
        } else {
            BTreeMap::new()
        };

        for (kind, source) in &mut self.sources {
            let listeners = desired.get(kind).cloned().unwrap_or_default();
            let active = !listeners.is_empty();
            source.set_listeners(listeners);
            if active {
                source.start();
            } else {
                source.stop();
            }
        }
    }

    pub fn ingest_event(
        &mut self,
        scheduled: &[ScheduledAutomation],
        event: Value,
        ready: bool,
        platform_matched: bool,
        should_schedule_locally: impl Fn(&str, &ScheduledAutomation) -> bool,
    ) -> usize {
        if self.stopped || !ready {
            return 0;
        }
        let Some(kind) = event.get("source").and_then(Value::as_str) else {
            return 0;
        };
        let Some(source) = self.sources.get(kind) else {
            return 0;
        };
        if !source.accepts(&event, platform_matched, false) {
            return 0;
        }

        let fires = matching_fires(
            scheduled,
            &event,
            ready,
            self.stopped,
            platform_matched,
            should_schedule_locally,
        );
        let count = fires.len();
        for fire in fires {
            self.consumer.enqueue(AutomationFireEnvelope {
                agent_id: fire.agent_id,
                automation_id: fire.automation_id,
                event: event.clone(),
            });
        }
        count
    }

    pub fn drain_fires(
        &mut self,
        fire: impl FnMut(&AutomationFireEnvelope) -> Result<(), String>,
    ) -> Vec<AutomationFireFailure> {
        self.consumer.drain(fire)
    }

    pub fn source(&self, kind: &str) -> Option<&BackendRelaySource> {
        self.sources.get(kind)
    }
}
