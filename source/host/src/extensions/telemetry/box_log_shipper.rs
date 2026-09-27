pub const DEFAULT_LOG_DIR: &str = "/tmp";
pub const BOX_LOG_SHIP_INTERVAL_MS: u64 = 2_000;
pub const BOX_LOG_SHIP_PROGRESS_INTERVAL_MS: u64 = 5 * 60_000;
pub const DEFAULT_MAX_BYTES_PER_READ: u64 = 256 * 1024;
pub const DEFAULT_MAX_LINES_PER_POLL: usize = 1_000;
pub const DEFAULT_MAX_LAG_BYTES: u64 = 16 * 1024 * 1024;
pub const DEFAULT_MAX_LINE_BYTES: usize = 64 * 1024;
pub const OFFSETS_FILE_NAME: &str = "sand-log-shipper.offsets.json";
pub const LOG_SUFFIX: &str = ".log";
pub const BOX_TELEMETRY_SOURCE: &str = "sand-box-telemetry";
pub const NEWLINE: u8 = 10;
pub const DEFAULT_EXCLUDED_SOURCE_PREFIXES: [&str; 1] = ["sand-notify-injector"];

pub fn is_box_log_shipping_enabled(host_in_box: Option<&str>, disabled: Option<&str>) -> bool {
    if host_in_box != Some("1") {
        return false;
    }
    !matches!(
        disabled.map(str::trim).map(str::to_ascii_lowercase).as_deref(),
        Some("1" | "true" | "yes")
    )
}

pub fn to_source_name(file_name: &str) -> &str {
    file_name.strip_suffix(LOG_SUFFIX).unwrap_or(file_name)
}

pub fn classify_offset_save_errno(errno: Option<&str>) -> &'static str {
    match errno {
        Some("ENOSPC" | "EDQUOT") => "no_space",
        Some("EACCES" | "EPERM") => "permission_denied",
        Some("EROFS") => "read_only",
        Some("ENOENT") => "missing_parent",
        None => "unknown",
        Some(_) => "io",
    }
}

pub fn saturating_add(total: u64, value: u64) -> u64 {
    total.saturating_add(value)
}

pub fn to_nonnegative_safe_integer(value: f64) -> u64 {
    const MAX_SAFE_INTEGER: u64 = 9_007_199_254_740_991;
    if !value.is_finite() {
        return MAX_SAFE_INTEGER;
    }
    value
        .round()
        .clamp(0.0, MAX_SAFE_INTEGER as f64) as u64
}
