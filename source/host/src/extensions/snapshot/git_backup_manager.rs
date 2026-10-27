use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use std::collections::BTreeMap;

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};

pub const SNAPSHOT_EXPORT_ENVELOPE_VERSION: u32 = 1;
pub const SQLITE_SIDECAR_SUFFIXES: [&str; 3] = ["-wal", "-shm", "-journal"];

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SnapshotExportEnvelope {
    pub version: u32,
    pub snapshot_id: String,
    pub created_at_ms: u64,
    pub manifest: Value,
    pub checksum: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitBackupMetadata {
    pub repository_path: String,
    pub branch: String,
    pub head_commit: String,
    pub total_blobs: usize,
    pub archive_bytes: u64,
}

/// Immutable evidence captured for a database artifact before it is published.
///
/// This is the Rust-side admission boundary corresponding to OpenClaw's
/// `captureDatabaseArtifact`: only an unaliased regular file is admitted and
/// its bytes are hashed while the opened handle is held.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DatabaseArtifact {
    pub path: PathBuf,
    pub sha256: String,
    pub size_bytes: u64,
}

/// A monotonic generation fence for snapshot export/restore work.
///
/// Producers must capture `current()` before beginning work and call
/// `assert_current()` before publication. A newer generation invalidates the
/// stale operation instead of allowing it to overwrite newer state.
#[derive(Debug, Clone, Default)]
pub struct SnapshotGenerationFence {
    generation: Arc<AtomicU64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresArtifactDigest {
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresTableDigest {
    pub rows: u64,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresSequenceAllocation {
    pub value: String,
    pub called: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresSourceIdentity {
    pub engine: String,
    pub store_id: String,
    pub owner: String,
    pub namespace: String,
    pub schema_version: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresVerification {
    pub tables: BTreeMap<String, PostgresTableDigest>,
    pub allocations: BTreeMap<String, PostgresSequenceAllocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresArtifactSet {
    #[serde(rename = "database.sqlite")]
    pub database: PostgresArtifactDigest,
    #[serde(rename = "anchor.sqlite")]
    pub anchor: PostgresArtifactDigest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PostgresExportManifest {
    pub schema_version: u32,
    pub snapshot_id: String,
    pub created_at: String,
    pub profile: String,
    pub source: PostgresSourceIdentity,
    pub export_schema_version: u32,
    pub active_fragments: Vec<String>,
    pub verification: PostgresVerification,
    pub artifacts: PostgresArtifactSet,
    pub sanitized_differences: Vec<String>,
}

fn valid_identifier(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else { return false };
    value.len() <= 255
        && first.is_ascii_alphanumeric()
        && chars.all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '.' | '_' | '-'))
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn validate_artifact(label: &str, artifact: &PostgresArtifactDigest) -> Result<(), String> {
    if !valid_digest(&artifact.sha256) {
        return Err(format!("{label} has an invalid SHA-256 digest"));
    }
    if artifact.size_bytes == 0 {
        return Err(format!("{label} must have positive size"));
    }
    Ok(())
}

/// Parse and validate OpenClaw's schema-v2 Postgres export envelope.
///
/// This is format validation only; it does not materialize a database or
/// establish that the referenced artifacts are present on disk.
pub fn parse_postgres_export_manifest(
    value: &Value,
    expected_id: &str,
    required_profile: Option<&str>,
) -> Result<PostgresExportManifest, String> {
    let manifest: PostgresExportManifest = serde_json::from_value(value.clone())
        .map_err(|error| format!("invalid Postgres export manifest: {error}"))?;
    if manifest.schema_version != 2 {
        return Err(format!("unsupported Postgres export schema: {}", manifest.schema_version));
    }
    if manifest.snapshot_id != expected_id || !valid_identifier(&manifest.snapshot_id) {
        return Err("export manifest id does not match its directory".into());
    }
    if !matches!(manifest.profile.as_str(), "exact" | "user-backup") {
        return Err(format!("unsupported export profile: {}", manifest.profile));
    }
    if let Some(required) = required_profile {
        if manifest.profile != required {
            return Err("export profile does not match the requested restore".into());
        }
    }
    if manifest.created_at.parse::<chrono::DateTime<chrono::FixedOffset>>().is_err() {
        return Err("createdAt must be an RFC3339 timestamp".into());
    }
    if manifest.source.engine != "postgres"
        || uuid::Uuid::parse_str(&manifest.source.store_id).is_err()
        || !valid_identifier(&manifest.source.owner)
        || !valid_identifier(&manifest.source.namespace)
        || manifest.source.schema_version > i32::MAX as u32
        || manifest.export_schema_version > i32::MAX as u32
    {
        return Err("export source identity is invalid".into());
    }
    if manifest.active_fragments.iter().any(|fragment| !valid_identifier(fragment)) {
        return Err("export contains an invalid active fragment".into());
    }
    if manifest.profile == "exact" && !manifest.sanitized_differences.is_empty() {
        return Err("exact exports must not contain sanitization differences".into());
    }
    validate_artifact("database.sqlite", &manifest.artifacts.database)?;
    validate_artifact("anchor.sqlite", &manifest.artifacts.anchor)?;
    for (table, digest) in &manifest.verification.tables {
        if !valid_identifier(table) || !valid_digest(&digest.sha256) {
            return Err(format!("invalid table verification entry: {table}"));
        }
    }
    for (sequence, allocation) in &manifest.verification.allocations {
        if !valid_identifier(sequence)
            || !allocation.value.chars().all(|ch| ch.is_ascii_digit())
            || allocation.value.is_empty()
        {
            return Err(format!("invalid sequence allocation entry: {sequence}"));
        }
        if allocation.value.len() > 19 || allocation.value.parse::<u64>().is_err() {
            return Err(format!("sequence allocation is out of range: {sequence}"));
        }
    }
    Ok(manifest)
}

#[cfg(unix)]
fn create_manifest_file(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .write(true)
        .create_new(true)
        .mode(0o600)
        .open(path)
}

#[cfg(not(unix))]
fn create_manifest_file(path: &Path) -> io::Result<File> {
    OpenOptions::new().write(true).create_new(true).open(path)
}

/// Write a validated schema-v2 manifest without replacing an existing one.
pub fn write_postgres_export_manifest(
    path: impl AsRef<Path>,
    value: &Value,
    expected_id: &str,
) -> Result<(), String> {
    let path = path.as_ref();
    let manifest = parse_postgres_export_manifest(value, expected_id, None)?;
    let bytes = serde_json::to_vec_pretty(&manifest)
        .map_err(|error| format!("serialize Postgres export manifest: {error}"))?;
    let mut file = create_manifest_file(path)
        .map_err(|error| format!("create Postgres export manifest {}: {error}", path.display()))?;
    file.write_all(&bytes)
        .and_then(|_| file.write_all(b"\n"))
        .and_then(|_| file.sync_all())
        .map_err(|error| format!("persist Postgres export manifest {}: {error}", path.display()))
}

impl SnapshotGenerationFence {
    pub fn new(initial_generation: u64) -> Self {
        Self {
            generation: Arc::new(AtomicU64::new(initial_generation)),
        }
    }

    pub fn current(&self) -> u64 {
        self.generation.load(Ordering::Acquire)
    }

    pub fn advance(&self) -> u64 {
        self.generation.fetch_add(1, Ordering::AcqRel) + 1
    }

    pub fn assert_current(&self, expected_generation: u64) -> Result<(), String> {
        let current = self.current();
        if current == expected_generation {
            Ok(())
        } else {
            Err(format!(
                "stale snapshot generation: expected {}, current {}",
                expected_generation, current
            ))
        }
    }
}

#[cfg(unix)]
fn open_unaliased_readonly(path: &Path) -> io::Result<File> {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW)
        .open(path)
}

#[cfg(not(unix))]
fn open_unaliased_readonly(path: &Path) -> io::Result<File> {
    let metadata = std::fs::symlink_metadata(path)?;
    if metadata.file_type().is_symlink() {
        return Err(io::Error::new(
            io::ErrorKind::PermissionDenied,
            "database artifact must not be a symbolic link",
        ));
    }
    OpenOptions::new().read(true).open(path)
}

/// Capture a stable hash/size admission record for a database artifact.
pub fn capture_database_artifact(path: impl AsRef<Path>) -> Result<DatabaseArtifact, String> {
    let path = path.as_ref();
    let file = open_unaliased_readonly(path)
        .map_err(|error| format!("open database artifact {}: {error}", path.display()))?;
    let metadata = file
        .metadata()
        .map_err(|error| format!("stat database artifact {}: {error}", path.display()))?;
    if !metadata.is_file() {
        return Err(format!(
            "database artifact must be a regular file: {}",
            path.display()
        ));
    }
    #[cfg(unix)]
    if metadata.nlink() != 1 {
        return Err(format!(
            "database artifact must be unaliased (nlink={}): {}",
            metadata.nlink(),
            path.display()
        ));
    }

    let mut reader = file;
    let mut digest = Sha256::new();
    let mut bytes = 0u64;
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = reader
            .read(&mut buffer)
            .map_err(|error| format!("read database artifact {}: {error}", path.display()))?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
        bytes += read as u64;
    }

    Ok(DatabaseArtifact {
        path: path.to_path_buf(),
        sha256: format!("{:x}", digest.finalize()),
        size_bytes: bytes,
    })
}

/// Re-capture an artifact and reject any mutation since admission.
pub fn assert_database_artifact(expected: &DatabaseArtifact) -> Result<(), String> {
    let actual = capture_database_artifact(&expected.path)?;
    if actual == *expected {
        Ok(())
    } else {
        Err(format!(
            "database artifact changed: {}",
            expected.path.display()
        ))
    }
}

/// Reject SQLite sidecars before admitting an export or restore target.
pub fn assert_no_sqlite_sidecars(path: impl AsRef<Path>, error_prefix: &str) -> Result<(), String> {
    let path = path.as_ref();
    for suffix in SQLITE_SIDECAR_SUFFIXES {
        let sidecar = PathBuf::from(format!("{}{}", path.display(), suffix));
        if std::fs::symlink_metadata(&sidecar).is_ok() {
            return Err(format!("{error_prefix}: {}", sidecar.display()));
        }
    }
    Ok(())
}

/// Admit a restore target only when neither the database nor its sidecars exist.
pub fn assert_fresh_restore_target(path: impl AsRef<Path>) -> Result<(), String> {
    let path = path.as_ref();
    if std::fs::symlink_metadata(path).is_ok() {
        return Err(format!("fresh SQLite restore path already exists: {}", path.display()));
    }
    assert_no_sqlite_sidecars(path, "fresh SQLite restore path already exists")
}

pub fn create_export_envelope(snapshot_id: &str, manifest: Value) -> SnapshotExportEnvelope {
    let now_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0);

    let manifest_bytes = serde_json::to_vec(&manifest).unwrap_or_default();
    let checksum = format!("{:x}", Sha256::digest(&manifest_bytes));

    SnapshotExportEnvelope {
        version: SNAPSHOT_EXPORT_ENVELOPE_VERSION,
        snapshot_id: snapshot_id.to_string(),
        created_at_ms: now_ms,
        manifest,
        checksum,
    }
}

pub fn verify_export_envelope(envelope: &SnapshotExportEnvelope) -> Result<(), String> {
    if envelope.version != SNAPSHOT_EXPORT_ENVELOPE_VERSION {
        return Err(format!(
            "Unsupported snapshot envelope version: {}",
            envelope.version
        ));
    }

    let manifest_bytes = serde_json::to_vec(&envelope.manifest)
        .map_err(|e| format!("Manifest serialization failed: {e}"))?;
    let expected_checksum = format!("{:x}", Sha256::digest(&manifest_bytes));

    if envelope.checksum.eq_ignore_ascii_case(&expected_checksum) {
        Ok(())
    } else {
        Err(format!(
            "Snapshot checksum mismatch: expected {}, got {}",
            expected_checksum, envelope.checksum
        ))
    }
}
