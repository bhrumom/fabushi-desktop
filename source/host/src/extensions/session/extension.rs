use std::sync::Arc;

use crate::extensions::browser_ua::extension::StopSubscription;
use crate::extensions::experiments::HostExperimentsExtension;

use super::box_handoff_service::{
    BoxHandoffDeps, BoxHandoffService, HandoffRequest, HandoffStartResult, HandoffTrigger,
    PendingHandoff,
};
use super::conversation_size_limits::pin_conversation_gc;
use super::production::ProductionSessionWorkers;
use super::session_maintenance::{
    pin_legacy_store_blob_retirement, pin_stale_root_gc,
};

pub const SESSION_EXTENSION_ID: &str = "session";
pub const SESSION_EXTENSION_DEPENDENCIES: &[&str] =
    &["experiments", "forever-box", "settings", "telemetry"];

pub struct SessionExtension {
    store: Arc<ProductionSessionWorkers>,
    handoff: BoxHandoffService,
    stop_experiment_subscription: Option<StopSubscription>,
}

impl SessionExtension {
    pub fn store(&self) -> Arc<ProductionSessionWorkers> {
        Arc::clone(&self.store)
    }

    pub fn pending_handoff(&self, agent_id: &str) -> Option<PendingHandoff> {
        self.handoff.get(agent_id)
    }

    pub fn start_handoff(&self, request: HandoffRequest) -> HandoffStartResult {
        self.handoff.start(request)
    }

    pub fn end_handoff(
        &self,
        agent_id: &str,
        trigger: HandoffTrigger,
    ) -> Result<bool, String> {
        self.handoff.end(agent_id, trigger)
    }

    pub fn forget_handoff(&self, agent_id: &str) {
        self.handoff.forget(agent_id);
    }

    pub fn shutdown(&self) {
        self.store.shutdown();
    }
}

impl Drop for SessionExtension {
    fn drop(&mut self) {
        if let Some(stop) = self.stop_experiment_subscription.take() {
            stop();
        }
        self.store.shutdown();
    }
}

pub fn start_session_extension(
    experiments: Arc<HostExperimentsExtension>,
    store: Arc<ProductionSessionWorkers>,
    handoff_deps: BoxHandoffDeps,
) -> SessionExtension {
    apply_experiment_pins(&experiments);
    let subscribed_experiments = Arc::clone(&experiments);
    let stop_experiment_subscription = experiments.subscribe(Arc::new(move || {
        apply_experiment_pins(&subscribed_experiments);
    }));

    SessionExtension {
        store,
        handoff: BoxHandoffService::new(handoff_deps),
        stop_experiment_subscription: Some(stop_experiment_subscription),
    }
}

fn apply_experiment_pins(experiments: &HostExperimentsExtension) {
    pin_stale_root_gc(experiments.check_feature_gate("sand_stale_root_gc"));
    pin_legacy_store_blob_retirement(
        experiments.check_feature_gate("sand_legacy_store_blob_retirement"),
    );
    pin_conversation_gc(experiments.check_feature_gate("grok_bot_conversation_gc"));
}
