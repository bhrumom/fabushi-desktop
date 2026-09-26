use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::Value;

use crate::extensions::experiments::HostExperimentsExtension;
use crate::extensions::extension_ids_generated::HostExtensionId;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::settings::settings_service::SettingsService;
use crate::runner::sand_auto_review::{
    SandAutoReviewController, SandAutoReviewEvent, SandAutoReviewMode,
    SandAutoReviewModes,
};

use super::auto_review_service::{
    AutoReviewService, AutoReviewTelemetrySink, AutoReviewUpdateSink,
};

pub const AUTO_REVIEW_DEPENDENCIES: &[HostExtensionId] = &[
    HostExtensionId::Auth,
    HostExtensionId::Experiments,
    HostExtensionId::Settings,
    HostExtensionId::Telemetry,
    HostExtensionId::Transcript,
];

pub fn auto_review_extension_id() -> HostExtensionId {
    HostExtensionId::AutoReview
}

pub fn parse_local_auto_review_mode(value: Option<&str>) -> Option<SandAutoReviewMode> {
    match value.map(str::trim) {
        Some("off") => Some(SandAutoReviewMode::Off),
        Some("shadow") => Some(SandAutoReviewMode::Shadow),
        Some("enforce") => Some(SandAutoReviewMode::Enforce),
        _ => None,
    }
}

pub struct HostAutoReviewExtension {
    service: Arc<AutoReviewService>,
    experiments: Arc<HostExperimentsExtension>,
    settings: Arc<SettingsService>,
    local_mode: Option<SandAutoReviewMode>,
}

impl HostAutoReviewExtension {
    pub fn service(&self) -> Arc<AutoReviewService> {
        Arc::clone(&self.service)
    }

    pub fn current_modes(&self) -> SandAutoReviewModes {
        AutoReviewService::current_modes(
            self.settings.get_auto_review_instructions().is_enabled,
            self.experiments.check_feature_gate("sand_auto_review"),
            self.local_mode,
        )
    }

    pub fn bind_runner(
        &self,
        agent_id: &str,
        approvals_resolvable: bool,
    ) -> Arc<SandAutoReviewController> {
        self.service.bind_runner(agent_id, approvals_resolvable)
    }

    pub fn stop(&self) {
        self.service.stop();
    }
}

pub fn start_auto_review_extension(
    sessions: Arc<ProductionSessionWorkers>,
    experiments: Arc<HostExperimentsExtension>,
    settings: Arc<SettingsService>,
    host_generation: impl Into<String>,
    on_update: AutoReviewUpdateSink,
    telemetry: AutoReviewTelemetrySink,
) -> HostAutoReviewExtension {
    let service = AutoReviewService::new(
        sessions,
        host_generation,
        on_update,
        telemetry,
    );
    service.sweep_stale_boot_state(now_ms());
    HostAutoReviewExtension {
        service,
        experiments,
        settings,
        local_mode: parse_local_auto_review_mode(
            std::env::var("SAND_AUTO_REVIEW_MODE").ok().as_deref(),
        ),
    }
}

pub fn no_op_auto_review_update_sink() -> AutoReviewUpdateSink {
    Arc::new(|_: &str, _: Value| {})
}

pub fn no_op_auto_review_telemetry_sink() -> AutoReviewTelemetrySink {
    Arc::new(|_: &SandAutoReviewEvent| {})
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
