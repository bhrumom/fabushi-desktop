use super::box_store_transfer::CategoryTransferSummary;

pub const DEFAULT_MAX_OBJECT_BYTES: u64 = 32 * 1024 * 1024 * 1024;
pub const DEFAULT_COPY_IN_CONCURRENCY: usize = 128;
pub const DEFAULT_SNAPSHOT_OUT_CONCURRENCY: usize = 8;
pub const LARGE_OBJECT_THRESHOLD_BYTES: u64 = 64 * 1024 * 1024;
pub const DOWNLOAD_IN_FLIGHT_BYTE_BUDGET: u64 = 256 * 1024 * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxStoreCycleSummary {
    pub ok: bool,
    pub reason: Option<String>,
    pub store_id: Option<String>,
    pub duration_ms: u64,
    pub categories: Vec<CategoryTransferSummary>,
    pub manifest_entries: usize,
    pub store_db_entries: usize,
    pub store_db_complete: bool,
    pub agent_dir_entries: usize,
    pub total_files_uploaded: usize,
    pub total_bytes_uploaded: u64,
    pub total_failures: usize,
    pub metadata_failures: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxStoreFlushEvaluation {
    pub ok: bool,
    pub manifest_entries: usize,
    pub store_db_entries: usize,
    pub agent_dir_entries: usize,
    pub store_db_complete: bool,
    pub files_uploaded: usize,
    pub reason: String,
}

pub fn evaluate_box_store_flush(
    summary: Option<&BoxStoreCycleSummary>,
    session_category_name: &str,
) -> BoxStoreFlushEvaluation {
    let Some(summary) = summary else {
        return failed("error", None);
    };
    if !summary.ok {
        return failed(summary.reason.as_deref().unwrap_or("error"), Some(summary));
    }
    if summary
        .categories
        .iter()
        .any(|category| category.name == "store.db")
        && !summary.store_db_complete
    {
        return failed("store-db-incomplete", Some(summary));
    }
    if summary.metadata_failures > 0 {
        return failed("filesystem-metadata-incomplete", Some(summary));
    }
    if summary
        .categories
        .iter()
        .find(|category| category.name == session_category_name)
        .is_some_and(|category| category.failures > 0)
    {
        return failed("chrome-session-stage-failed", Some(summary));
    }
    BoxStoreFlushEvaluation {
        ok: true,
        manifest_entries: summary.manifest_entries,
        store_db_entries: summary.store_db_entries,
        agent_dir_entries: summary.agent_dir_entries,
        store_db_complete: summary.store_db_complete,
        files_uploaded: summary.total_files_uploaded,
        reason: String::new(),
    }
}

fn failed(reason: &str, summary: Option<&BoxStoreCycleSummary>) -> BoxStoreFlushEvaluation {
    BoxStoreFlushEvaluation {
        ok: false,
        manifest_entries: summary.map_or(0, |value| value.manifest_entries),
        store_db_entries: summary.map_or(0, |value| value.store_db_entries),
        agent_dir_entries: summary.map_or(0, |value| value.agent_dir_entries),
        store_db_complete: summary.is_some_and(|value| value.store_db_complete),
        files_uploaded: summary.map_or(0, |value| value.total_files_uploaded),
        reason: reason.to_string(),
    }
}

pub fn telemetry_level(summary: &BoxStoreCycleSummary) -> Option<&'static str> {
    if summary.ok {
        Some("info")
    } else if summary.reason.as_deref() == Some("in-flight") {
        None
    } else {
        Some("warn")
    }
}
