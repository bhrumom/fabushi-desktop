use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};

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
