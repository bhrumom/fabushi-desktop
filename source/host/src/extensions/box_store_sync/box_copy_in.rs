use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File, OpenOptions};
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use fs2::FileExt;

use sha2::{Digest, Sha256};

use super::agent_store_sand_files::normalize_rel_path;
use super::box_object_store::{
    BoxObjectStore, BoxObjectStoreProvider, BoxObjectStoreProviderDependencies,
    resolve_box_object_store_provider,
};
use super::box_store_download::{
    is_critical_rel_path, resolve_restore_destination, symlink_target_stays_within_root,
};
use super::box_store_pack::{
    BOX_STORE_PACK_INDEX_KEY, BOX_STORE_PACKS_PREFIX, PACK_INDEX_MAX_BYTES, PackExtractionSink,
    PackMember, extract_pack_members, parse_pack_index,
};
use super::box_store_pack_pipeline::{
    PACK_EXTRACT_CONCURRENCY, PACK_TMP_DIR_NAME, should_restore_from_pack,
};
use super::box_store_sync::LARGE_OBJECT_THRESHOLD_BYTES;
use super::box_store_manifest::count_store_db_manifest_entries;
use super::box_store_manifest_format::{
    BOX_STORE_MANIFEST_REL_PATH, BoxStoreManifestEntry, parse_box_store_manifest,
};
use crate::extensions::auth::auth_service::HostAuthService;
use crate::extensions::auth::credential_renewer::get_configured_backend_url;
use crate::host_paths::get_sand_root_dir;
use crate::r#box::box_store_backend_policy::{
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
const COPY_IN_RETRY_BASE_MS: u64 = 1_000;
const COPY_IN_RETRY_MAX_MS: u64 = 15_000;

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
    if result.reason.starts_with("lock-held")
        || result.reason.contains("primary V2 store.db coverage is below")
        || result.reason.contains("no store.db identities to validate V2")
        || result.reason.starts_with("legacy source manifest is empty")
    {
        return false;
    }
    if result.reason.starts_with("partial hydrate")
        || result.reason.starts_with("incomplete legacy hydrate")
    {
        return true;
    }
    let text = result
        .failures
        .first()
        .map(String::as_str)
        .unwrap_or(&result.reason)
        .to_ascii_lowercase();
    if text.contains("no inference credential")
        || text.contains("401")
        || text.contains("403")
        || text.contains("forbidden")
        || text.contains("access denied")
        || text.contains("unauthorized")
        || text.contains("unauthenticated")
        || text.contains("not authorized")
        || text.contains("access is not enabled")
    {
        return false;
    }
    true
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

#[derive(Debug)]
struct CopyInStoreLock {
    file: File,
}

impl Drop for CopyInStoreLock {
    fn drop(&mut self) {
        let _ = FileExt::unlock(&self.file);
    }
}

fn try_acquire_copy_in_lock(path: &Path) -> io::Result<Option<CopyInStoreLock>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(path)?;
    match file.try_lock_exclusive() {
        Ok(()) => Ok(Some(CopyInStoreLock { file })),
        Err(error) if error.kind() == io::ErrorKind::WouldBlock => Ok(None),
        Err(error) => Err(error),
    }
}

pub fn build_copy_in_status_from_result(result: &CopyInResult) -> serde_json::Value {
    serde_json::json!({
        "phase": if result.outcome == CopyInOutcome::Failed { "failed" } else { "done" },
        "restored": result.files,
        "total": result.manifest_entries,
        "bytes": result.bytes,
        "outcome": match classify_copy_in_metered_outcome(result) {
            CopyInMeteredOutcome::Hydrated => "hydrated",
            CopyInMeteredOutcome::Empty => "empty",
            CopyInMeteredOutcome::Partial => "partial",
            CopyInMeteredOutcome::Failed => "failed",
        },
        "storeDbEntries": result.store_db_entries,
    })
}

pub fn write_copy_in_status_atomic(path: &Path, status: &serde_json::Value) -> io::Result<()> {
    let temp_path = path.with_extension("tmp");
    fs::write(&temp_path, serde_json::to_vec(status).map_err(io::Error::other)?)?;
    fs::rename(temp_path, path)
}

fn run_copy_in_with_provider(
    environment: &BTreeMap<String, String>,
    target_root: &Path,
    provider: &dyn BoxObjectStoreProvider,
) -> CopyInResult {
    let Some(store_id) = environment
        .get("SAND_BOX_STORE_ID")
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
    else {
        return empty("no SAND_BOX_STORE_ID; booting fresh");
    };
    let store_id = match normalize_rel_path(store_id) {
        Ok(value) => value,
        Err(error) => return failed_copy_in(format!("invalid store id: {error}")),
    };
    let store = provider.for_store(&store_id);
    run_local_box_copy_in(store.as_ref(), target_root)
}

fn report_copy_in_result(result: &CopyInResult) -> i32 {
    match result {
        CopyInResult { outcome: CopyInOutcome::Hydrated | CopyInOutcome::Noop, .. } => {
            eprintln!(
                "[box-copy-in] outcome={:?} files={} bytes={} verified={} reason={}",
                result.outcome, result.files, result.bytes, result.verified, result.reason
            );
        }
        _ => {
            eprintln!(
                "[box-copy-in] failed files={} verified={} failures={} reason={}",
                result.files,
                result.verified,
                result.failures.len(),
                result.reason
            );
        }
    }
    outcome_to_exit_code(result.outcome)
}

fn run_copy_in_with_retry(
    environment: &BTreeMap<String, String>,
    target_root: &Path,
    provider: &dyn BoxObjectStoreProvider,
) -> CopyInResult {
    let attempts = resolve_copy_in_attempts(environment).max(1);
    let mut last = failed_copy_in("copy-in did not run".into());
    for attempt in 1..=attempts {
        last = run_copy_in_with_provider(environment, target_root, provider);
        if last.outcome != CopyInOutcome::Failed
            || !is_transient_copy_in_failure(&last)
            || attempt == attempts
        {
            return last;
        }
        let exponent = u32::try_from(attempt.saturating_sub(1)).unwrap_or(u32::MAX).min(20);
        let backoff = COPY_IN_RETRY_BASE_MS
            .saturating_mul(1_u64 << exponent)
            .min(COPY_IN_RETRY_MAX_MS);
        eprintln!(
            "[box-copy-in] transient failure (attempt {attempt}/{attempts}): {}; retrying in {backoff}ms",
            last.reason
        );
        thread::sleep(Duration::from_millis(backoff));
    }
    last
}

/// Dependency-injected copy-in entrypoint. Without remote dependencies it is
/// appropriate for LocalFS/tests; the shipping CLI uses
/// execute_production_box_copy_in_from_env so AgentStore/V2 reuse Host auth.
pub fn execute_box_copy_in_from_env(
    environment: &BTreeMap<String, String>,
    target_root: &Path,
) -> i32 {
    execute_box_copy_in_from_env_with_provider_dependencies(
        environment,
        target_root,
        BoxObjectStoreProviderDependencies::default(),
    )
}

/// Shipping copy-in entrypoint. LocalFS requires no authentication; remote
/// AgentStore/SandBoxStoreV2 reuse the canonical Host auth and machine-id
/// owners plus the same BoxObjectStore provider resolver used by the long-lived
/// production sync service.
pub fn execute_production_box_copy_in_from_env(
    environment: &BTreeMap<String, String>,
    target_root: &Path,
) -> i32 {
    if !is_box_store_copy_in_enabled(environment) {
        return BOX_COPY_IN_EXIT_NOOP;
    }

    let lock_path = get_sand_root_dir().join("box-store-sync.lock");
    let _lock = match try_acquire_copy_in_lock(&lock_path) {
        Ok(Some(lock)) => lock,
        Ok(None) => {
            let result = failed_copy_in("lock-held".into());
            let _ = write_copy_in_status_atomic(
                Path::new(SAND_BOX_COPY_IN_STATUS_PATH),
                &build_copy_in_status_from_result(&result),
            );
            eprintln!("[box-copy-in] could not acquire box-store lock; failing closed");
            return BOX_COPY_IN_EXIT_FAILED;
        }
        Err(error) => {
            let result = failed_copy_in(format!("store lock error: {error}"));
            let _ = write_copy_in_status_atomic(
                Path::new(SAND_BOX_COPY_IN_STATUS_PATH),
                &build_copy_in_status_from_result(&result),
            );
            return report_copy_in_result(&result);
        }
    };

    let _ = write_copy_in_status_atomic(
        Path::new(SAND_BOX_COPY_IN_STATUS_PATH),
        &serde_json::json!({"phase":"copying","restored":0,"total":0,"bytes":0}),
    );

    let policy = resolve_box_store_backend_policy(environment);
    let deps = if policy.local_dir.is_some() {
        BoxObjectStoreProviderDependencies::default()
    } else {
        let backend_url = match get_configured_backend_url() {
            Ok(url) => url,
            Err(error) => {
                let result = failed_copy_in(format!("invalid production backend URL: {error}"));
                let _ = write_copy_in_status_atomic(
                    Path::new(SAND_BOX_COPY_IN_STATUS_PATH),
                    &build_copy_in_status_from_result(&result),
                );
                return report_copy_in_result(&result);
            }
        };
        let auth = match HostAuthService::production(|message| {
            eprintln!("[box-copy-in][auth] {message}");
        }) {
            Ok(auth) => Arc::new(auth),
            Err(error) => {
                let result = failed_copy_in(format!("failed to initialize production auth: {error}"));
                let _ = write_copy_in_status_atomic(
                    Path::new(SAND_BOX_COPY_IN_STATUS_PATH),
                    &build_copy_in_status_from_result(&result),
                );
                return report_copy_in_result(&result);
            }
        };
        let token_auth = Arc::clone(&auth);
        let machine_auth = Arc::clone(&auth);
        BoxObjectStoreProviderDependencies {
            backend_url: Some(backend_url),
            get_access_token: Some(Arc::new(move || {
                token_auth.get_access_token().map_err(|error| error.to_string())
            })),
            get_machine_id: Some(Arc::new(move || {
                machine_auth.get_machine_id().map_err(|error| error.to_string())
            })),
        }
    };

    let provider = match resolve_box_object_store_provider(environment, deps) {
        Ok(provider) => provider,
        Err(error) => {
            let result = failed_copy_in(format!("object-store provider unavailable: {error}"));
            let _ = write_copy_in_status_atomic(
                Path::new(SAND_BOX_COPY_IN_STATUS_PATH),
                &build_copy_in_status_from_result(&result),
            );
            return report_copy_in_result(&result);
        }
    };
    let result = run_copy_in_with_retry(environment, target_root, provider.as_ref());
    let _ = write_copy_in_status_atomic(
        Path::new(SAND_BOX_COPY_IN_STATUS_PATH),
        &build_copy_in_status_from_result(&result),
    );
    report_copy_in_result(&result)
}

pub fn execute_box_copy_in_from_env_with_provider_dependencies(
    environment: &BTreeMap<String, String>,
    target_root: &Path,
    deps: BoxObjectStoreProviderDependencies,
) -> i32 {
    if !is_box_store_copy_in_enabled(environment) {
        return BOX_COPY_IN_EXIT_NOOP;
    }
    let provider = match resolve_box_object_store_provider(environment, deps) {
        Ok(provider) => provider,
        Err(error) => return report_copy_in_result(&failed_copy_in(format!(
            "object-store provider unavailable: {error}"
        ))),
    };
    execute_box_copy_in_with_provider(environment, target_root, provider.as_ref())
}

pub fn execute_box_copy_in_with_provider(
    environment: &BTreeMap<String, String>,
    target_root: &Path,
    provider: &dyn BoxObjectStoreProvider,
) -> i32 {
    report_copy_in_result(&run_copy_in_with_provider(environment, target_root, provider))
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
    let pack_restored = restore_bulk_small_from_packs(store, target_root, &manifest.entries);
    let mut files = pack_restored.len();
    let mut bytes = pack_restored
        .iter()
        .filter_map(|rel_path| manifest.entries.get(rel_path))
        .filter_map(manifest_file_size)
        .sum::<u64>();
    let mut verified = files;
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
                if pack_restored.contains(rel_path) {
                    if let BoxStoreManifestEntry::File { mode, .. } = entry {
                        #[cfg(unix)]
                        {
                            use std::os::unix::fs::PermissionsExt;
                            let permissions = fs::Permissions::from_mode(*mode);
                            if let Err(error) = fs::set_permissions(&destination, permissions) {
                                failures.push(format!("set mode {rel_path}: {error}"));
                            }
                        }
                    }
                    continue;
                }
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


#[derive(Debug, Clone)]
struct PackRestoreGroup {
    size: u64,
    rel_paths: Vec<String>,
}

struct CopyInPackSink {
    target_root: PathBuf,
    groups: HashMap<String, PackRestoreGroup>,
    restored: Mutex<HashSet<String>>,
}

impl PackExtractionSink for CopyInPackSink {
    fn wants(&self, member: &PackMember) -> bool {
        let key = format!("{}:{}", member.sha, member.size);
        let Some(group) = self.groups.get(&key) else {
            return false;
        };
        let restored = self
            .restored
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        group.rel_paths.iter().any(|rel_path| !restored.contains(rel_path))
    }

    fn on_blob(&self, member: &PackMember, bytes: Vec<u8>) -> Result<(), String> {
        let key = format!("{}:{}", member.sha, member.size);
        let Some(group) = self.groups.get(&key) else {
            return Ok(());
        };
        for rel_path in &group.rel_paths {
            let reserved = {
                let mut restored = self
                    .restored
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner());
                if restored.contains(rel_path) {
                    false
                } else {
                    restored.insert(rel_path.clone());
                    true
                }
            };
            if !reserved {
                continue;
            }
            let Some(destination) = resolve_restore_destination(&self.target_root, rel_path) else {
                self.restored
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(rel_path);
                continue;
            };
            if local_file_matches(&destination, &member.sha, member.size) {
                continue;
            }
            if write_pack_blob_atomically(
                &destination,
                rel_path,
                &member.sha,
                member.size,
                &bytes,
            )
            .is_err()
            {
                self.restored
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .remove(rel_path);
            }
        }
        Ok(())
    }
}

fn manifest_file_size(entry: &BoxStoreManifestEntry) -> Option<u64> {
    match entry {
        BoxStoreManifestEntry::LegacyFile { size, .. }
        | BoxStoreManifestEntry::File { size, .. } => Some(*size),
        BoxStoreManifestEntry::Symlink { .. } => None,
    }
}

fn local_file_matches(path: &Path, sha: &str, size: u64) -> bool {
    let Ok(metadata) = fs::symlink_metadata(path) else {
        return false;
    };
    metadata.file_type().is_file()
        && metadata.len() == size
        && verify_restored_file(path, sha, size).is_ok()
}

fn write_pack_blob_atomically(
    destination: &Path,
    rel_path: &str,
    sha: &str,
    size: u64,
    bytes: &[u8],
) -> Result<(), String> {
    if bytes.len() as u64 != size || format!("{:x}", Sha256::digest(bytes)) != sha {
        return Err(format!("pack member sha/size mismatch for {rel_path}"));
    }
    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let temp = PathBuf::from(format!(
        "{}.box-store-pack-part-{}-{nonce}",
        destination.display(),
        std::process::id()
    ));
    let result = (|| -> Result<(), String> {
        fs::write(&temp, bytes).map_err(|error| error.to_string())?;
        remove_existing_restore_path(destination)?;
        fs::rename(&temp, destination).map_err(|error| error.to_string())?;
        verify_restored_file(destination, sha, size)
    })();
    if result.is_err() {
        let _ = fs::remove_file(&temp);
    }
    result
}

fn restore_bulk_small_from_packs(
    store: &dyn BoxObjectStore,
    target_root: &Path,
    manifest: &BTreeMap<String, BoxStoreManifestEntry>,
) -> HashSet<String> {
    let mut groups = HashMap::<String, PackRestoreGroup>::new();
    for (rel_path, entry) in manifest {
        if is_critical_rel_path(rel_path) {
            continue;
        }
        let (sha, size) = match entry {
            BoxStoreManifestEntry::LegacyFile { sha, size }
            | BoxStoreManifestEntry::File { sha, size, .. } => (sha, *size),
            BoxStoreManifestEntry::Symlink { .. } => continue,
        };
        if size >= LARGE_OBJECT_THRESHOLD_BYTES {
            continue;
        }
        let key = format!("{sha}:{size}");
        groups
            .entry(key)
            .and_modify(|group| group.rel_paths.push(rel_path.clone()))
            .or_insert_with(|| PackRestoreGroup {
                size,
                rel_paths: vec![rel_path.clone()],
            });
    }
    if groups.is_empty() {
        return HashSet::new();
    }

    let raw_index = match store.get(BOX_STORE_PACK_INDEX_KEY) {
        Ok(Some(bytes)) if bytes.len() as u64 <= PACK_INDEX_MAX_BYTES => bytes,
        _ => return HashSet::new(),
    };
    let Some(index) = std::str::from_utf8(&raw_index).ok().and_then(parse_pack_index) else {
        return HashSet::new();
    };
    if index.packs.is_empty() {
        return HashSet::new();
    }

    let sink = std::sync::Arc::new(CopyInPackSink {
        target_root: target_root.to_path_buf(),
        groups: groups.clone(),
        restored: Mutex::new(HashSet::new()),
    });
    let tmp_dir = std::env::temp_dir().join(PACK_TMP_DIR_NAME);
    if fs::create_dir_all(&tmp_dir).is_err() {
        return HashSet::new();
    }

    for (ordinal, pack) in index.packs.iter().enumerate() {
        let mut usable_count = 0usize;
        let mut usable_clen = 0u64;
        let mut usable_size = 0u64;
        for member in &pack.members {
            let key = format!("{}:{}", member.sha, member.size);
            let Some(group) = groups.get(&key) else {
                continue;
            };
            let restored = sink
                .restored
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let pending = group.rel_paths.iter().any(|rel_path| !restored.contains(rel_path));
            drop(restored);
            if pending {
                usable_count += 1;
                usable_clen = usable_clen.saturating_add(member.clen);
                usable_size = usable_size.saturating_add(member.size);
            }
        }
        if !should_restore_from_pack(usable_count, usable_clen, usable_size, pack.bytes) {
            continue;
        }

        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let temp = tmp_dir.join(format!(
            "restore-{}-{nonce}-{ordinal}",
            std::process::id()
        ));
        let result = (|| -> Result<(), String> {
            let Some(written) = store.get_to_file(
                &format!("{BOX_STORE_PACKS_PREFIX}/{}", pack.id),
                &temp,
                Some(pack.bytes),
            )? else {
                return Ok(());
            };
            if written != pack.bytes {
                return Ok(());
            }
            let _ = extract_pack_members(
                &temp,
                &pack.members,
                PACK_EXTRACT_CONCURRENCY,
                sink.clone(),
            )?;
            Ok(())
        })();
        let _ = fs::remove_file(&temp);
        if result.is_err() {
            continue;
        }
    }

    let restored = sink
        .restored
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .clone();
    restored
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
