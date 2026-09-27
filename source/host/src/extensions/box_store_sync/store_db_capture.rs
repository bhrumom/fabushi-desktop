use super::box_store_transfer::CategoryTransferSummary;
use super::store_db_bundle_capture::StoreDbCaptureTrace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StoreDbCaptureOutcome {
    Error,
    Oversize,
    Uploaded,
    Skipped,
    Unchanged,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreDbSweepResult {
    pub summary: CategoryTransferSummary,
    pub complete: bool,
    pub agent_count: usize,
    pub captured_this_sweep: Vec<String>,
    pub capture_trace: StoreDbCaptureTrace,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreDbSnapshotResult {
    pub outcome: StoreDbCaptureOutcome,
    pub summary: CategoryTransferSummary,
    pub capture_trace: StoreDbCaptureTrace,
    pub is_committed: bool,
    pub store_id: Option<String>,
}

pub fn aggregate_store_db_sweep_outcome(
    failures: usize,
    oversize: usize,
    files_uploaded: usize,
    files_scanned: usize,
) -> StoreDbCaptureOutcome {
    if failures > 0 {
        StoreDbCaptureOutcome::Error
    } else if oversize > 0 {
        StoreDbCaptureOutcome::Oversize
    } else if files_uploaded > 0 {
        StoreDbCaptureOutcome::Uploaded
    } else if files_scanned == 0 {
        StoreDbCaptureOutcome::Skipped
    } else {
        StoreDbCaptureOutcome::Unchanged
    }
}

pub fn box_store_db_capture_telemetry_level(outcome: StoreDbCaptureOutcome) -> &'static str {
    match outcome {
        StoreDbCaptureOutcome::Error | StoreDbCaptureOutcome::Oversize => "warn",
        StoreDbCaptureOutcome::Uploaded
        | StoreDbCaptureOutcome::Skipped
        | StoreDbCaptureOutcome::Unchanged => "info",
    }
}
