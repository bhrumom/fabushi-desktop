use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fs::{File, OpenOptions};
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};

pub const SNAPSHOT_EXPORT_ENVELOPE_VERSION: u32 = 1;

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
