use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use sha2::{Digest, Sha256};

use super::agent_store_sand_files::normalize_rel_path;
use super::box_object_store::{BoxObjectStore, LocalFsObjectStore};
use super::box_store_download::{resolve_restore_destination, symlink_target_stays_within_root};
use super::box_store_manifest::count_store_db_manifest_entries;
use super::box_store_manifest_format::{
    BOX_STORE_MANIFEST_REL_PATH, BoxStoreManifestEntry, parse_box_store_manifest,
};
use crate::box::box_store_backend_policy::{
    is_box_store_copy_in_enabled, resolve_box_store_backend_policy,
};

pub const BOX_COPY_IN_ARG: &str = "--box-copy-in";
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

/// Executes the shipping Host's copy-in bootstrap for the local object-store
/// backend. The CLI must never fall through into the long-lived Host when
/// copy-in was explicitly requested. Remote AgentStore/V2 adapters remain
/// fail-closed until their production transport is wired into this owner.
pub fn execute_box_copy_in_from_env(
    environment: &BTreeMap<String, String>,
    target_root: &Path,
) -> i32 {
    if !is_box_store_copy_in_enabled(environment) {
        return BOX_COPY_IN_EXIT_NOOP;
    }

    let policy = resolve_box_store_backend_policy(environment);
    let Some(local_dir) = policy.local_dir else {
        eprintln!(
            "[box-copy-in] remote object-store adapter is not production-wired; refusing to start Host"
        );
        return BOX_COPY_IN_EXIT_FAILED;
    };
    let Some(store_id) = environment
        .get("SAND_BOX_STORE_ID")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        eprintln!("[box-copy-in] no SAND_BOX_STORE_ID; booting fresh");
        return BOX_COPY_IN_EXIT_NOOP;
    };
    let store_id = match normalize_rel_path(store_id) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[box-copy-in] invalid store id: {error}");
            return BOX_COPY_IN_EXIT_FAILED;
        }
    };
    let store = LocalFsObjectStore::new(local_dir.join(store_id));
    match run_local_box_copy_in(&store, target_root) {
        result @ CopyInResult { outcome: CopyInOutcome::Hydrated, .. }
        | result @ CopyInResult { outcome: CopyInOutcome::Noop, .. } => {
            eprintln!(
                "[box-copy-in] outcome={:?} files={} bytes={} verified={} reason={}",
                result.outcome, result.files, result.bytes, result.verified, result.reason
            );
            outcome_to_exit_code(result.outcome)
        }
        result => {
            eprintln!(
                "[box-copy-in] failed files={} verified={} failures={} reason={}",
                result.files,
                result.verified,
                result.failures.len(),
                result.reason
            );
            BOX_COPY_IN_EXIT_FAILED
        }
    }
}

/// Canonical local-store copy-in orchestration used by the shipping
/// `--box-copy-in` process mode and focused contract tests.
pub fn run_local_box_copy_in(store: &dyn BoxObjectStore, target_root: &Path) -> CopyInResult {
    let manifest_bytes = match store.get(BOX_STORE_MANIFEST_REL_PATH) {
        Ok(Some(bytes)) => bytes,
        Ok(None) => return empty("store empty; first boot"),
        Err(error) => return failed_copy_in(format!("store manifest unreadable: {error}")),
    };
    let raw_manifest: serde_json::Value = match serde_json::from_slice(&manifest_bytes) {
        Ok(value) => value,
        Err(error) => return failed_copy_in(format!("store manifest unreadable: {error}")),
    };
    let Some(manifest) = parse_box_store_manifest(&raw_manifest) else {
        return failed_copy_in("store manifest unreadable: invalid manifest".to_string());
    };
    if manifest.fully_hydrated == Some(false) {
        return failed_copy_in(
            "incomplete legacy hydrate: local manifest is not fully hydrated".to_string(),
        );
    }
    if manifest.entries.is_empty() {
        return empty("store empty; first boot");
    }

    let manifest_entries = manifest.entries.len();
    let store_db_entries = count_store_db_manifest_entries(Some(&manifest.entries));
    let mut files = 0usize;
    let mut bytes = 0u64;
    let mut verified = 0usize;
    let mut failures = Vec::new();

    for (rel_path, entry) in &manifest.entries {
        let Some(destination) = resolve_restore_destination(target_root, rel_path) else {
            failures.push(format!("unsafe restore path: {rel_path}"));
            continue;
        };
        match entry {
            BoxStoreManifestEntry::Symlink { target } => {
                if !symlink_target_stays_within_root(target_root, &destination, target) {
                    failures.push(format!("unsafe symlink target: {rel_path}"));
                    continue;
                }
                if let Some(parent) = destination.parent() {
                    if let Err(error) = fs::create_dir_all(parent) {
                        failures.push(format!("create parent {rel_path}: {error}"));
                        continue;
                    }
                }
                if let Err(error) = remove_existing_restore_path(&destination) {
                    failures.push(format!("remove existing {rel_path}: {error}"));
                    continue;
                }
                #[cfg(unix)]
                {
                    if let Err(error) = std::os::unix::fs::symlink(target, &destination) {
                        failures.push(format!("restore symlink {rel_path}: {error}"));
                        continue;
                    }
                    files += 1;
                    verified += 1;
                }
                #[cfg(not(unix))]
                {
                    failures.push(format!("symlink restore unsupported on this platform: {rel_path}"));
                }
            }
            BoxStoreManifestEntry::LegacyFile { sha, size }
            | BoxStoreManifestEntry::File { sha, size, .. } => {
                if let Err(error) = remove_existing_restore_path(&destination) {
                    failures.push(format!("remove existing {rel_path}: {error}"));
                    continue;
                }
                let key = format!("blobs/{sha}");
                match store.get_to_file(&key, &destination, Some(*size)) {
                    Ok(Some(restored_bytes)) => {
                        files += 1;
                        bytes = bytes.saturating_add(restored_bytes);
                        match verify_restored_file(&destination, sha, *size) {
                            Ok(()) => verified += 1,
                            Err(error) => failures.push(format!("verify {rel_path}: {error}")),
                        }
                    }
                    Ok(None) => failures.push(format!("missing blob for {rel_path}: {sha}")),
                    Err(error) => failures.push(format!("restore {rel_path}: {error}")),
                }
                if let BoxStoreManifestEntry::File { mode, .. } = entry {
                    #[cfg(unix)]
                    {
                        use std::os::unix::fs::PermissionsExt;
                        if destination.exists() {
                            let permissions = fs::Permissions::from_mode(*mode);
                            if let Err(error) = fs::set_permissions(&destination, permissions) {
                                failures.push(format!("set mode {rel_path}: {error}"));
                            }
                        }
                    }
                }
            }
        }
    }

    let restored_store_db_entries = manifest
        .entries
        .iter()
        .filter(|(rel_path, entry)| {
            !entry.is_symlink()
                && rel_path.starts_with("home/box/sand-data/agents/")
                && rel_path.ends_with("/store.db")
                && target_root.join(rel_path).is_file()
        })
        .count();

    if failures.is_empty() && files == manifest_entries && verified == manifest_entries {
        CopyInResult {
            outcome: CopyInOutcome::Hydrated,
            reason: "store hydrated".into(),
            manifest_entries,
            store_db_entries,
            restored_store_db_entries: Some(restored_store_db_entries),
            files,
            bytes,
            verified,
            failures,
        }
    } else {
        CopyInResult {
            outcome: CopyInOutcome::Failed,
            reason: format!(
                "partial hydrate ({files}/{manifest_entries} files, {} failures)",
                failures.len()
            ),
            manifest_entries,
            store_db_entries,
            restored_store_db_entries: Some(restored_store_db_entries),
            files,
            bytes,
            verified,
            failures,
        }
    }
}

fn verify_restored_file(path: &Path, expected_sha: &str, expected_size: u64) -> Result<(), String> {
    let bytes = fs::read(path).map_err(|error| error.to_string())?;
    if bytes.len() as u64 != expected_size {
        return Err(format!(
            "size mismatch expected={expected_size} actual={}",
            bytes.len()
        ));
    }
    let actual_sha = format!("{:x}", Sha256::digest(&bytes));
    if actual_sha != expected_sha {
        return Err(format!("sha mismatch expected={expected_sha} actual={actual_sha}"));
    }
    Ok(())
}

fn remove_existing_restore_path(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.file_type().is_dir() => {
            fs::remove_dir_all(path).map_err(|error| error.to_string())
        }
        Ok(_) => fs::remove_file(path).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}

fn failed_copy_in(reason: String) -> CopyInResult {
    CopyInResult {
        outcome: CopyInOutcome::Failed,
        reason: reason.clone(),
        manifest_entries: 0,
        store_db_entries: 0,
        restored_store_db_entries: None,
        files: 0,
        bytes: 0,
        verified: 0,
        failures: vec![reason],
    }
}
