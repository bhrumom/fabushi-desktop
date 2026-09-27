#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum StoreDbCaptureFailurePhase {
    Capture,
    BlobUpload,
    ManifestCommit,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct StoreDbCaptureTrace {
    pub queue_duration_ms: u64,
    pub capture_duration_ms: u64,
    pub blob_upload_duration_ms: u64,
    pub manifest_commit_duration_ms: u64,
    pub failure_phase: Option<StoreDbCaptureFailurePhase>,
}

pub fn create_store_db_capture_trace() -> StoreDbCaptureTrace {
    StoreDbCaptureTrace::default()
}

pub fn record_store_db_capture_failure(
    trace: &mut StoreDbCaptureTrace,
    phase: StoreDbCaptureFailurePhase,
) {
    if trace.failure_phase.is_none_or(|current| phase > current) {
        trace.failure_phase = Some(phase);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DbBundleTarget {
    pub rel_path: String,
    pub abs_path: String,
    pub effective_size: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StoreDbCaptureSignature {
    pub write_generation: u64,
    pub mtime_ms: u64,
    pub size: u64,
    pub sha: String,
    pub mode: u32,
    pub wal_size: u64,
    pub wal_mtime_ms: u64,
}

pub fn bundle_identity_matches(
    before: &StoreDbCaptureSignature,
    after: &StoreDbCaptureSignature,
) -> bool {
    before == after
}

pub fn elapsed_duration_ms(now_ms: u64, started_at_ms: u64) -> u64 {
    now_ms.saturating_sub(started_at_ms)
}
