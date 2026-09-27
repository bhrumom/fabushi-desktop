pub const BOX_COPY_IN_EXIT_NOOP: i32 = 0;
pub const BOX_COPY_IN_EXIT_FAILED: i32 = 1;
pub const BOX_COPY_IN_EXIT_HYDRATED: i32 = 10;
pub const SAND_BOX_COPY_IN_STATUS_PATH: &str = "/tmp/sand-copy-in-status.json";
pub const COPY_IN_STATUS_THROTTLE_MS: u64 = 750;
pub const COPY_IN_HYDRATE_ATTEMPTS: usize = 8;
pub const COPY_IN_STUCK_THRESHOLD_MS: u64 = 5 * 60_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyInOutcome {
    Hydrated,
    Noop,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CopyInResult {
    pub outcome: CopyInOutcome,
    pub reason: String,
    pub manifest_entries: usize,
    pub store_db_entries: usize,
    pub restored_store_db_entries: Option<usize>,
    pub files: usize,
    pub bytes: u64,
    pub verified: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CopyInMeteredOutcome {
    Hydrated,
    Empty,
    Partial,
    Failed,
}

pub fn resolve_copy_in_concurrency(raw: Option<&str>) -> Option<usize> {
    parse_positive_usize(raw)
}

pub fn resolve_copy_in_attempts(raw: Option<&str>) -> usize {
    parse_positive_usize(raw).unwrap_or(COPY_IN_HYDRATE_ATTEMPTS)
}

pub fn resolve_copy_in_stuck_threshold_ms(raw: Option<&str>) -> u64 {
    raw.and_then(|value| value.trim().parse::<u64>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(COPY_IN_STUCK_THRESHOLD_MS)
}

fn parse_positive_usize(raw: Option<&str>) -> Option<usize> {
    raw.and_then(|value| value.trim().parse::<usize>().ok())
        .filter(|value| *value > 0)
}

pub fn classify_copy_in_metered_outcome(result: &CopyInResult) -> CopyInMeteredOutcome {
    match result.outcome {
        CopyInOutcome::Hydrated => CopyInMeteredOutcome::Hydrated,
        CopyInOutcome::Noop => CopyInMeteredOutcome::Empty,
        CopyInOutcome::Failed
            if result.reason.starts_with("partial hydrate")
                || result.reason.starts_with("incomplete legacy hydrate") =>
        {
            CopyInMeteredOutcome::Partial
        }
        CopyInOutcome::Failed => CopyInMeteredOutcome::Failed,
    }
}

pub fn outcome_to_exit_code(outcome: CopyInOutcome) -> i32 {
    match outcome {
        CopyInOutcome::Hydrated => BOX_COPY_IN_EXIT_HYDRATED,
        CopyInOutcome::Noop => BOX_COPY_IN_EXIT_NOOP,
        CopyInOutcome::Failed => BOX_COPY_IN_EXIT_FAILED,
    }
}

pub fn is_transient_copy_in_failure(result: &CopyInResult) -> bool {
    if result.outcome != CopyInOutcome::Failed {
        return false;
    }
    let reason = result.reason.to_ascii_lowercase();
    [
        "timeout",
        "timed out",
        "temporar",
        "connection",
        "network",
        "locked",
        "in-flight",
        "503",
        "429",
    ]
    .iter()
    .any(|needle| reason.contains(needle))
}

pub fn redact_copy_in_error_for_telemetry(raw: &str) -> String {
    raw.split_whitespace()
        .map(|token| {
            if token.starts_with("http://") || token.starts_with("https://") {
                "<url>"
            } else if token.to_ascii_lowercase().contains("token=")
                || token.to_ascii_lowercase().contains("authorization:")
            {
                "<redacted>"
            } else {
                token
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(512)
        .collect()
}

pub fn empty(reason: impl Into<String>) -> CopyInResult {
    CopyInResult {
        outcome: CopyInOutcome::Noop,
        reason: reason.into(),
        manifest_entries: 0,
        store_db_entries: 0,
        restored_store_db_entries: None,
        files: 0,
        bytes: 0,
        verified: 0,
        failures: Vec::new(),
    }
}
