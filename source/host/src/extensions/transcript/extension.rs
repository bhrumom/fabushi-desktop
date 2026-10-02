use std::path::Path;
use std::sync::{Arc, Mutex};

use serde_json::json;

use super::profile_watch::ProductionProfileWatch;
use super::roster_emit::{ProductionRosterEmit, RosterEventSink};
use super::roster_projection::OUTLINE_STREAM_COALESCE_MS;
use super::transcript_manager::{
    TranscriptManager, TranscriptManagerServices, TranscriptTurnExecutionPort,
};
use crate::extensions::attachments::attachments_service::AttachmentsService;
use crate::extensions::content_search::extension::ProductionContentSearchExtension;
use crate::extensions::memory::extension::HostMemoryExtension;
use crate::extensions::session::production::ProductionSessionWorkers;
use crate::extensions::telemetry::extension::HostTelemetryExtension;
use crate::extensions::trays::extension::HostTraysExtension;
use crate::extensions::turn_execution::turn_execution_service::TurnExecutionRegistry;
use crate::host_event_bus::{HostEventFailureMode, SandHostEventBus};

pub const TRANSCRIPT_EXTENSION_ID: &str = "transcript";
pub const TRANSCRIPT_EXTENSION_DEPENDENCIES: &[&str] = &[
    "attachments",
    "content-search",
    "memory",
    "session",
    "telemetry",
    "trays",
    "turn-execution",
];

#[derive(Clone)]
pub struct TranscriptExtensionDeps {
    pub attachments: Arc<AttachmentsService>,
    pub content_search: Arc<ProductionContentSearchExtension>,
    pub memory: HostMemoryExtension,
    pub telemetry: HostTelemetryExtension,
    pub trays: Arc<HostTraysExtension>,
    pub turn_execution: Arc<Mutex<TurnExecutionRegistry>>,
    pub events: SandHostEventBus,
}

#[derive(Clone)]
pub struct TranscriptExtensionEventBridge {
    events: SandHostEventBus,
}

impl TranscriptExtensionEventBridge {
    pub fn new(events: SandHostEventBus) -> Self {
        Self { events }
    }

    fn emit(&self, topic: &str, payload: serde_json::Value) {
        let _ = self
            .events
            .emit_topic(topic, &payload, HostEventFailureMode::Continue);
    }

    pub fn automation_config_changed(&self) {
        self.emit("transcript.automation-config-changed", json!({}));
    }

    pub fn listener_connect_card(&self, agent_id: &str, platform: &str) {
        self.emit(
            "transcript.listener-connect-card",
            json!({"agentId": agent_id, "platform": platform}),
        );
    }

    pub fn run_started(&self, request_id: &str) {
        self.emit("transcript.run-started", json!({"requestId": request_id}));
    }

    pub fn run_ended(&self, request_id: &str) {
        self.emit("transcript.run-ended", json!({"requestId": request_id}));
    }
}

/// Shipping Grok-shaped Transcript extension owner.
///
/// This is the production composition boundary corresponding to frozen
/// `source/host/extensions/transcript/extension.ts`: it retains the exact Host
/// dependency owners and installs the Host-event observers at extension start,
/// while delegated transcript domains remain independently testable modules.
pub struct TranscriptExtension {
    manager: Arc<TranscriptManager>,
    roster_emit: Arc<ProductionRosterEmit>,
    profile_watch: Option<ProductionProfileWatch>,
    profile_watch_error: Option<String>,
    deps: Option<TranscriptExtensionDeps>,
    events: Option<TranscriptExtensionEventBridge>,
}

impl TranscriptExtension {
    pub fn manager(&self) -> Arc<TranscriptManager> {
        Arc::clone(&self.manager)
    }

    pub fn roster_emit(&self) -> Arc<ProductionRosterEmit> {
        Arc::clone(&self.roster_emit)
    }

    pub fn event_bridge(&self) -> Option<TranscriptExtensionEventBridge> {
        self.events.clone()
    }

    pub fn profile_watch_active(&self) -> bool {
        self.profile_watch.is_some()
    }

    pub fn profile_watch_error(&self) -> Option<&str> {
        self.profile_watch_error.as_deref()
    }
}

impl Drop for TranscriptExtension {
    fn drop(&mut self) {
        self.roster_emit.stop_outline_stream_coalescing();
        self.manager.dispose();
    }
}

pub fn start_transcript_extension(
    root_dir: &Path,
    sessions: Arc<ProductionSessionWorkers>,
    event_sink: RosterEventSink,
) -> TranscriptExtension {
    let manager = Arc::new(TranscriptManager::new(root_dir, Arc::clone(&sessions)));
    let roster_emit = Arc::new(ProductionRosterEmit::new(
        Arc::clone(&sessions),
        manager.transcript_runtime(),
        event_sink,
    ));
    roster_emit.set_outline_stream_coalescing_ms(OUTLINE_STREAM_COALESCE_MS);
    let (profile_watch, profile_watch_error) =
        match ProductionProfileWatch::start(sessions, Arc::clone(&roster_emit)) {
            Ok(watch) => (Some(watch), None),
            Err(error) => (None, Some(error.to_string())),
        };
    TranscriptExtension {
        manager,
        roster_emit,
        profile_watch,
        profile_watch_error,
        deps: None,
        events: None,
    }
}

pub fn start_production_transcript_extension(
    root_dir: &Path,
    sessions: Arc<ProductionSessionWorkers>,
    deps: TranscriptExtensionDeps,
) -> TranscriptExtension {
    let roster_events = deps.events.clone();
    let mut extension = start_transcript_extension(
        root_dir,
        sessions,
        Arc::new(move |event| roster_events.publish(event)),
    );
    extension
        .manager
        .set_production_services(TranscriptManagerServices {
            telemetry: Arc::clone(&deps.telemetry.brain),
            product_analytics: deps.telemetry.analytics.clone(),
            trace_flusher: deps.telemetry.api(),
            memory: deps.memory.clone(),
            content_search: Arc::clone(&deps.content_search),
            attachments: Arc::clone(&deps.attachments),
            trays: Arc::clone(&deps.trays),
        })
        .expect("Transcript production services must be configured exactly once");
    extension
        .manager
        .set_turn_execution(TranscriptTurnExecutionPort::new(Arc::clone(
            &deps.turn_execution,
        )));
    let events = TranscriptExtensionEventBridge::new(deps.events.clone());
    let lifecycle_events = events.clone();
    extension
        .manager
        .transcript_runtime()
        .set_agent_run_lifecycle_observer(Some(Arc::new(move |event_type, request_id| {
            if event_type == "started" {
                lifecycle_events.run_started(request_id);
            } else if event_type == "ended" {
                lifecycle_events.run_ended(request_id);
            }
        })));
    extension.deps = Some(deps);
    extension.events = Some(events);
    extension
}
