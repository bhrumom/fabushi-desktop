pub const SAND_UPGRADE_RESUME_FILE_NAME: &str = "host-upgrade-resume.json";
pub const SAND_ACK_OBLIGATIONS_FILE_NAME: &str = "ack-obligations.json";
pub const SAND_PENDING_WAKE_FILE_NAME: &str = "host-pending-wakes.json";
pub const SAND_XUSER_TURN_DEDUPE_FILE_NAME: &str = "host-xuser-turn-nonces.json";
pub const SAND_DISK_PRESSURE_REMINDERS_FILE_NAME: &str = "host-disk-pressure-reminders.json";
pub const SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME: &str = "host-local-exec-operations.json";
pub const HOST_CONFIG_REVISION_KEY_FILE_NAME: &str = "host-config-revision-key.bin";
pub const HOST_CONFIG_REVISION_KEY_TEMP_FILE_PATTERN: &str = "host-config-revision-key.bin.*.tmp";

/// Installation key material, including incomplete publication files, is never
/// portable Box state. The caller supplies a basename after path normalization.
pub fn is_host_config_revision_key_file_name(name: &str) -> bool {
    name == HOST_CONFIG_REVISION_KEY_FILE_NAME
        || name
            .strip_prefix(HOST_CONFIG_REVISION_KEY_FILE_NAME)
            .and_then(|suffix| suffix.strip_prefix('.'))
            .is_some_and(|suffix| suffix.ends_with(".tmp"))
}

pub const BOX_STORE_SAND_DATA_EXCLUDED_FILE_NAMES: [&str; 8] = [
    SAND_UPGRADE_RESUME_FILE_NAME,
    SAND_ACK_OBLIGATIONS_FILE_NAME,
    SAND_PENDING_WAKE_FILE_NAME,
    SAND_XUSER_TURN_DEDUPE_FILE_NAME,
    SAND_DISK_PRESSURE_REMINDERS_FILE_NAME,
    SAND_LOCAL_EXEC_OPERATION_JOURNAL_FILE_NAME,
    HOST_CONFIG_REVISION_KEY_FILE_NAME,
    HOST_CONFIG_REVISION_KEY_TEMP_FILE_PATTERN,
];
