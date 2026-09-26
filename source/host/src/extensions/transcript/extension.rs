use std::path::Path;
use std::sync::Arc;

use super::profile_watch::ProductionProfileWatch;
use super::roster_emit::{ProductionRosterEmit, RosterEventSink};
use super::transcript_manager::TranscriptManager;
use crate::extensions::session::production::ProductionSessionWorkers;

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

/// Shipping Grok-shaped Transcript extension owner.
///
/// The current Rust port centralizes the production manager, roster event
/// projection and active-profile watcher here. Remaining frozen dependency
/// setters and delegated domains stay explicit manifest work rather than being
/// hidden behind Host main.
pub struct TranscriptExtension {
    manager: Arc<TranscriptManager>,
    roster_emit: Arc<ProductionRosterEmit>,
    profile_watch: Option<ProductionProfileWatch>,
    profile_watch_error: Option<String>,
}

impl TranscriptExtension {
    pub fn manager(&self) -> Arc<TranscriptManager> {
        Arc::clone(&self.manager)
    }

    pub fn roster_emit(&self) -> Arc<ProductionRosterEmit> {
        Arc::clone(&self.roster_emit)
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
    }
}
