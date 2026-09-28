use std::collections::{BTreeMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::extensions::box_store_sync::box_object_store::{
    BoxObjectStore, LocalFsObjectStore,
};
use crate::extensions::box_store_sync::box_store_manifest::{
    AGENT_STORE_DB_BASENAMES, BoxManifestMap,
};
use crate::extensions::box_store_sync::box_store_manifest_format::{
    BOX_STORE_BLOBS_PREFIX, BOX_STORE_LEGACY_MANIFEST_VERSION, BOX_STORE_MANIFEST_REL_PATH,
    BOX_STORE_MANIFEST_VERSION, BoxStoreManifest, BoxStoreManifestEntry, SAND_MANIFEST_V2_ENV,
    parse_box_store_manifest,
};
use crate::extensions::box_store_sync::box_store_transfer::{
    CategoryTransferSummary, glob_matches_path,
};
use crate::extensions::box_store_sync::chrome_session_stage::{
    CHROME_SESSION_DB_DIR, CHROME_SESSION_DB_NAMES, stage_box_chrome_session,
};
use crate::extensions::box_store_sync::chrome_session_watcher::ChromeSessionWatcher;
use crate::extensions::box_store_sync::extension::{
    BoxStoreSyncExtensionDeps, BoxStoreSyncService, BoxStoreSyncServiceFactory,
};
use crate::extensions::box_store_sync::workspace_ignore::{
    SAND_BOX_WORKSPACE_DEFAULT_IGNORE_PATTERNS, WorkspaceIgnore, load_workspace_ignore,
};
use crate::host_paths::get_sand_root_dir;
use crate::r#box::box_store_backend_policy::{
    BoxStoreBackendKind, is_box_store_sync_enabled, resolve_box_store_backend_policy,
};

const SAND_BOX_STORE_ID_ENV: &str = "SAND_BOX_STORE_ID";
const WORKSPACE_ROOT: &str = "/workspace";
const CLI_CONFIG_ROOT: &str = "/home/box/cli-config";
const SAND_DATA_REL_PREFIX: &str = "home/box/sand-data";
const WORKSPACE_REL_PREFIX: &str = "workspace";
const CLI_CONFIG_REL_PREFIX: &str = "home/box/cli-config";
const POLL_SLEEP_SLICE_MS: u64 = 250;
const DEFAULT_MAX_OBJECT_BYTES: u64 = 32 * 1024 * 1024 * 1024;

const SAND_DATA_EXCLUDES: &[&str] = &[
    "home/box/sand-data/host.lock",
    "home/box/sand-data/gateway.json",
    "home/box/sand-data/box-store-sync.lock",
    "home/box/sand-data/box-store-hydration-handoff.json",
];

const CLI_CONFIG_EXCLUDES: &[&str] = &[
    "home/box/cli-config/.cli-auth-tmp*",
    "home/box/cli-config/*.cli-auth-old",
    "home/box/cli-config/**/*.cli-auth-old",
];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProductionBoxStoreSyncMode {
    Disabled,
    LocalFs {
        base_dir: PathBuf,
        store_id_override: Option<String>,
    },
    UnsupportedRemote { backend: BoxStoreBackendKind },
    InvalidLocalConfiguration { reason: String },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ProductionBoxStoreSyncStatus {
    pub enabled: bool,
    pub backend: String,
    pub last_ok: Option<bool>,
    pub last_reason: Option<String>,
    pub last_manifest_entries: usize,
    pub last_files_uploaded: usize,
    pub last_bytes_uploaded: u64,
}

#[derive(Clone)]
pub struct ProductionBoxStoreSyncApi {
    inner: Arc<ProductionBoxStoreSyncInner>,
}

impl ProductionBoxStoreSyncApi {
    pub fn status(&self) -> ProductionBoxStoreSyncStatus {
        self.inner
            .status
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clone()
    }

    pub fn snapshot_local_now(&self) -> Result<ProductionBoxStoreSyncStatus, String> {
        self.inner.run_local_cycle(false)?;
        Ok(self.status())
    }
}

pub struct ProductionBoxStoreSyncFactory;

impl BoxStoreSyncServiceFactory for ProductionBoxStoreSyncFactory {
    type Service = ProductionBoxStoreSyncService;

    fn create(&self, deps: BoxStoreSyncExtensionDeps) -> Self::Service {
        ProductionBoxStoreSyncService::new(deps)
    }
}

pub struct ProductionBoxStoreSyncService {
    inner: Arc<ProductionBoxStoreSyncInner>,
    watcher: Mutex<Option<ChromeSessionWatcher>>,
    poller: Mutex<Option<JoinHandle<()>>>,
}

impl ProductionBoxStoreSyncService {
    pub fn new(deps: BoxStoreSyncExtensionDeps) -> Self {
        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        let mode = resolve_production_box_store_sync_mode(&env);
        let backend = mode_name(&mode).to_string();
        let enabled = matches!(mode, ProductionBoxStoreSyncMode::LocalFs { .. });
        Self {
            inner: Arc::new(ProductionBoxStoreSyncInner {
                deps,
                env,
                mode,
                stopped: AtomicBool::new(false),
                cycle_lock: Mutex::new(()),
                status: Mutex::new(ProductionBoxStoreSyncStatus {
                    enabled,
                    backend,
                    ..ProductionBoxStoreSyncStatus::default()
                }),
            }),
            watcher: Mutex::new(None),
            poller: Mutex::new(None),
        }
    }

    pub fn mode(&self) -> &ProductionBoxStoreSyncMode {
        &self.inner.mode
    }
}

impl BoxStoreSyncService for ProductionBoxStoreSyncService {
    type Api = ProductionBoxStoreSyncApi;

    fn api(&self) -> Self::Api {
        ProductionBoxStoreSyncApi {
            inner: Arc::clone(&self.inner),
        }
    }

    fn start(&self) {
        if self.inner.stopped.load(Ordering::Acquire) {
            return;
        }

        match &self.inner.mode {
            ProductionBoxStoreSyncMode::Disabled => {
                self.inner.log("box-store sync disabled by SAND_BOX_STORE_SYNC");
                self.inner.diagnostic("startup", "disabled", false);
                return;
            }
            ProductionBoxStoreSyncMode::UnsupportedRemote { backend } => {
                self.inner.log(&format!(
                    "box-store sync backend {backend:?} is not production-wired in Rust yet; refusing to downgrade"
                ));
                self.inner
                    .diagnostic("startup", "remote-backend-not-wired", false);
                return;
            }
            ProductionBoxStoreSyncMode::InvalidLocalConfiguration { reason } => {
                self.inner.log(&format!(
                    "box-store sync local configuration rejected: {reason}"
                ));
                self.inner
                    .diagnostic("startup", "invalid-local-configuration", false);
                return;
            }
            ProductionBoxStoreSyncMode::LocalFs { .. } => {}
        }

        let chrome_inner = Arc::clone(&self.inner);
        let watcher = ChromeSessionWatcher::with_logger(
            CHROME_SESSION_DB_DIR,
            CHROME_SESSION_DB_NAMES.iter().copied(),
            self.inner.deps.scheduling.chrome_session_debounce.delay_ms,
            Arc::new(move || {
                if let Err(error) = chrome_inner.run_local_cycle(true) {
                    chrome_inner.log(&format!(
                        "chrome-session snapshot rejected: {error}"
                    ));
                    chrome_inner.diagnostic("chrome-session", &error, false);
                }
            }),
            {
                let inner = Arc::clone(&self.inner);
                Arc::new(move |message| inner.log(&message))
            },
        );
        watcher.start();
        *self
            .watcher
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(watcher);

        let poll_inner = Arc::clone(&self.inner);
        let interval_ms = self.inner.deps.scheduling.polling.interval_ms.max(1);
        let handle = thread::Builder::new()
            .name("box-store-sync-poller".into())
            .spawn(move || {
                while !poll_inner.stopped.load(Ordering::Acquire) {
                    if !sleep_interruptibly(&poll_inner.stopped, interval_ms) {
                        break;
                    }
                    if let Err(error) = poll_inner.run_local_cycle(false) {
                        poll_inner.log(&format!("periodic snapshot rejected: {error}"));
                        poll_inner.diagnostic("periodic", &error, false);
                    }
                }
            });
        match handle {
            Ok(handle) => {
                *self
                    .poller
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(handle);
                self.inner.log("box-store sync local-fs production slice enabled");
                self.inner.diagnostic("startup", "enabled-local-fs", true);
            }
            Err(error) => {
                self.inner.log(&format!("box-store poller failed to start: {error}"));
                self.inner.diagnostic("startup", "poller-start-failed", false);
            }
        }
    }

    fn dispose(&self) {
        if self.inner.stopped.swap(true, Ordering::AcqRel) {
            return;
        }
        if let Some(watcher) = self
            .watcher
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            watcher.stop();
        }
        if let Some(handle) = self
            .poller
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = handle.join();
        }
    }
}

struct ProductionBoxStoreSyncInner {
    deps: BoxStoreSyncExtensionDeps,
    env: BTreeMap<String, String>,
    mode: ProductionBoxStoreSyncMode,
    stopped: AtomicBool,
    cycle_lock: Mutex<()>,
    status: Mutex<ProductionBoxStoreSyncStatus>,
}

impl ProductionBoxStoreSyncInner {
    fn log(&self, message: &str) {
        (self.deps.log)(&format!("[box-store-sync] {message}"));
    }

    fn diagnostic(&self, phase: &str, reason: &str, ok: bool) {
        let mut diagnostic = Map::new();
        diagnostic.insert("extension".into(), Value::String("box-store-sync".into()));
        diagnostic.insert("phase".into(), Value::String(phase.into()));
        diagnostic.insert("reason".into(), Value::String(reason.into()));
        diagnostic.insert("ok".into(), Value::Bool(ok));
        diagnostic.insert("backend".into(), Value::String(mode_name(&self.mode).into()));
        (self.deps.report_host_extension_diagnostic)(&diagnostic);
    }

    fn run_local_cycle(&self, chrome_only: bool) -> Result<(), String> {
        let _cycle = self
            .cycle_lock
            .lock()
            .map_err(|_| "box-store sync cycle lock poisoned".to_string())?;
        if self.stopped.load(Ordering::Acquire) {
            return Err("stopped".into());
        }
        let ProductionBoxStoreSyncMode::LocalFs {
            base_dir,
            store_id_override,
        } = &self.mode
        else {
            return Err("local-fs backend is not active".into());
        };
        let store_id = match store_id_override {
            Some(store_id) => store_id.clone(),
            None => (self.deps.resolve_store_id)()?,
        };
        let normalized_store_id = crate::extensions::box_store_sync::agent_store_sand_files::normalize_rel_path(&store_id)
            .map_err(|error| format!("resolved BoxStore source id is unsafe: {error}"))?;
        if normalized_store_id != store_id {
            return Err("resolved BoxStore source id must already be canonical".into());
        }

        let store = LocalFsObjectStore::new(base_dir.join(&store_id));
        let mut manifest = load_manifest(&store)?;
        let manifest_v2 = self
            .env
            .get(SAND_MANIFEST_V2_ENV)
            .is_some_and(|value| value == "1")
            || manifest
                .as_ref()
                .is_some_and(|value| value.version == BOX_STORE_MANIFEST_VERSION);
        let mut entries = manifest
            .take()
            .map(|value| value.entries)
            .unwrap_or_default();
        let mut categories = Vec::new();

        if chrome_only {
            categories.push(sync_chrome_session(&store, &mut entries, manifest_v2)?);
        } else {
            let sand_root = get_sand_root_dir();
            categories.push(sync_tree_category(
                &store,
                &mut entries,
                &sand_root,
                SAND_DATA_REL_PREFIX,
                "sand-data",
                SAND_DATA_EXCLUDES,
                None,
                manifest_v2,
            )?);
            let workspace_ignore =
                load_workspace_ignore(WORKSPACE_ROOT, SAND_BOX_WORKSPACE_DEFAULT_IGNORE_PATTERNS);
            categories.push(sync_tree_category(
                &store,
                &mut entries,
                Path::new(WORKSPACE_ROOT),
                WORKSPACE_REL_PREFIX,
                "workspace",
                &[],
                Some(&workspace_ignore),
                manifest_v2,
            )?);
            categories.push(sync_tree_category(
                &store,
                &mut entries,
                Path::new(CLI_CONFIG_ROOT),
                CLI_CONFIG_REL_PREFIX,
                "cli-config",
                CLI_CONFIG_EXCLUDES,
                None,
                manifest_v2,
            )?);
        }

        write_manifest(
            &store,
            &entries,
            manifest_v2,
            format!("mahayana-host-{}", std::process::id()),
        )?;

        let files_uploaded = categories.iter().map(|value| value.files_uploaded).sum();
        let bytes_uploaded = categories.iter().map(|value| value.bytes_uploaded).sum();
        let failures = categories
            .iter()
            .map(|value| value.failures + value.oversize + value.metadata_failures)
            .sum::<usize>();
        let ok = failures == 0;
        {
            let mut status = self
                .status
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            status.last_ok = Some(ok);
            status.last_reason = (!ok).then(|| "category-failures".to_string());
            status.last_manifest_entries = entries.len();
            status.last_files_uploaded = files_uploaded;
            status.last_bytes_uploaded = bytes_uploaded;
        }
        self.log(&format!(
            "cycle {}: {} file(s) / {}B uploaded, {} manifest entries, {} failure(s)",
            if chrome_only { "chrome" } else { "periodic" },
            files_uploaded,
            bytes_uploaded,
            entries.len(),
            failures,
        ));
        self.diagnostic(
            if chrome_only { "chrome-session" } else { "periodic" },
            if ok { "ok" } else { "category-failures" },
            ok,
        );
        Ok(())
    }
}

pub fn resolve_production_box_store_sync_mode(env: &BTreeMap<String, String>) -> ProductionBoxStoreSyncMode {
    if !is_box_store_sync_enabled(env) {
        return ProductionBoxStoreSyncMode::Disabled;
    }
    let policy = resolve_box_store_backend_policy(env);
    match policy.kind {
        BoxStoreBackendKind::LocalFs => {
            let Some(base_dir) = policy.local_dir else {
                return ProductionBoxStoreSyncMode::InvalidLocalConfiguration {
                    reason: "local-fs backend requires an absolute SAND_BOX_STORE_LOCAL_DIR".into(),
                };
            };
            let store_id_override = env
                .get(SAND_BOX_STORE_ID_ENV)
                .map(String::as_str)
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(str::to_string);
            if let Some(store_id) = store_id_override.as_deref() {
                let normalized =
                    crate::extensions::box_store_sync::agent_store_sand_files::normalize_rel_path(store_id);
                if normalized.as_deref() != Ok(store_id) {
                    return ProductionBoxStoreSyncMode::InvalidLocalConfiguration {
                        reason: "SAND_BOX_STORE_ID must be a canonical safe relative source id".into(),
                    };
                }
            }
            ProductionBoxStoreSyncMode::LocalFs {
                base_dir,
                store_id_override,
            }
        }
        backend => ProductionBoxStoreSyncMode::UnsupportedRemote { backend },
    }
}

fn mode_name(mode: &ProductionBoxStoreSyncMode) -> &'static str {
    match mode {
        ProductionBoxStoreSyncMode::Disabled => "disabled",
        ProductionBoxStoreSyncMode::LocalFs { .. } => "local-fs",
        ProductionBoxStoreSyncMode::UnsupportedRemote {
            backend: BoxStoreBackendKind::AgentStore,
        } => "agent-store-unwired",
        ProductionBoxStoreSyncMode::UnsupportedRemote {
            backend: BoxStoreBackendKind::SandBoxStoreV2,
        } => "sand-box-store-v2-unwired",
        ProductionBoxStoreSyncMode::UnsupportedRemote {
            backend: BoxStoreBackendKind::LocalFs,
        } => "local-fs-unwired",
        ProductionBoxStoreSyncMode::InvalidLocalConfiguration { .. } => "local-fs-invalid",
    }
}

fn load_manifest(store: &dyn BoxObjectStore) -> Result<Option<BoxStoreManifest>, String> {
    let Some(bytes) = store.get(BOX_STORE_MANIFEST_REL_PATH)? else {
        return Ok(None);
    };
    let value = serde_json::from_slice::<Value>(&bytes)
        .map_err(|error| format!("manifest JSON is invalid: {error}"))?;
    parse_box_store_manifest(&value)
        .map(Some)
        .ok_or_else(|| "manifest schema is invalid".to_string())
}

fn write_manifest(
    store: &dyn BoxObjectStore,
    entries: &BoxManifestMap,
    manifest_v2: bool,
    writer_window_id: String,
) -> Result<(), String> {
    let mut values = Map::new();
    for (path, entry) in entries {
        let value = match entry {
            BoxStoreManifestEntry::LegacyFile { sha, size } => {
                json!({ "sha": sha, "size": size })
            }
            BoxStoreManifestEntry::File { sha, size, mode } => {
                if !manifest_v2 {
                    json!({ "sha": sha, "size": size })
                } else {
                    json!({ "kind": "file", "sha": sha, "size": size, "mode": mode })
                }
            }
            BoxStoreManifestEntry::Symlink { target } => {
                if !manifest_v2 {
                    return Err(
                        "manifest v1 cannot represent a symlink; enable SAND_MANIFEST_V2".into(),
                    );
                }
                json!({ "kind": "symlink", "target": target })
            }
        };
        values.insert(path.clone(), value);
    }
    let payload = json!({
        "version": if manifest_v2 { BOX_STORE_MANIFEST_VERSION } else { BOX_STORE_LEGACY_MANIFEST_VERSION },
        "updatedAtMs": now_ms(),
        "writerWindowId": writer_window_id,
        "entries": values,
    });
    let bytes = serde_json::to_vec(&payload).map_err(|error| error.to_string())?;
    store.put(BOX_STORE_MANIFEST_REL_PATH, &bytes)
}

fn sync_chrome_session(
    store: &dyn BoxObjectStore,
    manifest: &mut BoxManifestMap,
    manifest_v2: bool,
) -> Result<CategoryTransferSummary, String> {
    let staged = stage_box_chrome_session().map_err(|error| error.to_string())?;
    let mut summary = CategoryTransferSummary {
        name: "chrome-session".into(),
        ..CategoryTransferSummary::default()
    };
    for file in &staged.files {
        summary.files_scanned += 1;
        match sync_file(
            store,
            manifest,
            &file.abs_path,
            &file.rel_path,
            Some(file.mode),
            manifest_v2,
        ) {
            Ok((uploaded, size)) => {
                if uploaded {
                    summary.files_uploaded += 1;
                    summary.bytes_uploaded = summary.bytes_uploaded.saturating_add(size);
                } else {
                    summary.skipped_unchanged += 1;
                }
            }
            Err(error) => {
                summary.failures += 1;
                eprintln!("[box-store-sync] chrome staged file failed {}: {error}", file.rel_path);
            }
        }
    }
    summary.failures += staged.skipped;
    staged.cleanup().map_err(|error| error.to_string())?;
    Ok(summary)
}

fn sync_tree_category(
    store: &dyn BoxObjectStore,
    manifest: &mut BoxManifestMap,
    root: &Path,
    rel_prefix: &str,
    name: &str,
    excludes: &[&str],
    ignore: Option<&WorkspaceIgnore>,
    manifest_v2: bool,
) -> Result<CategoryTransferSummary, String> {
    let mut summary = CategoryTransferSummary {
        name: name.to_string(),
        ..CategoryTransferSummary::default()
    };
    let mut seen = HashSet::new();
    let mut walk_complete = true;
    if root.exists() {
        walk_tree(
            root,
            root,
            rel_prefix,
            excludes,
            ignore,
            &mut seen,
            &mut summary,
            &mut walk_complete,
            store,
            manifest,
            manifest_v2,
        );
    }

    if walk_complete {
        let prefix = format!("{rel_prefix}/");
        let stale = manifest
            .keys()
            .filter(|path| path.starts_with(&prefix))
            .filter(|path| !seen.contains(*path))
            .filter(|path| !is_excluded(path, excludes))
            .filter(|path| !is_agent_store_db_path(path))
            .cloned()
            .collect::<Vec<_>>();
        for path in stale {
            manifest.remove(&path);
            summary.removed += 1;
        }
    }
    Ok(summary)
}

#[allow(clippy::too_many_arguments)]
fn walk_tree(
    root: &Path,
    dir: &Path,
    rel_prefix: &str,
    excludes: &[&str],
    ignore: Option<&WorkspaceIgnore>,
    seen: &mut HashSet<String>,
    summary: &mut CategoryTransferSummary,
    walk_complete: &mut bool,
    store: &dyn BoxObjectStore,
    manifest: &mut BoxManifestMap,
    manifest_v2: bool,
) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => {
            summary.metadata_failures += 1;
            *walk_complete = false;
            return;
        }
    };
    for entry in entries {
        let Ok(entry) = entry else {
            summary.metadata_failures += 1;
            *walk_complete = false;
            continue;
        };
        let path = entry.path();
        let relative = match path.strip_prefix(root) {
            Ok(value) => value.to_string_lossy().replace('\\', "/"),
            Err(_) => {
                summary.metadata_failures += 1;
                *walk_complete = false;
                continue;
            }
        };
        let rel_path = format!("{rel_prefix}/{relative}");
        if is_excluded(&rel_path, excludes) || is_agent_store_db_path(&rel_path) {
            continue;
        }

        let metadata = match fs::symlink_metadata(&path) {
            Ok(metadata) => metadata,
            Err(_) => {
                seen.insert(rel_path);
                summary.metadata_failures += 1;
                continue;
            }
        };
        let file_type = metadata.file_type();
        if file_type.is_dir() {
            if ignore.is_some_and(|matcher| matcher.can_prune_dir(&relative)) {
                summary.pruned_dirs += 1;
                continue;
            }
            walk_tree(
                root,
                &path,
                rel_prefix,
                excludes,
                ignore,
                seen,
                summary,
                walk_complete,
                store,
                manifest,
                manifest_v2,
            );
            continue;
        }

        if ignore.is_some_and(|matcher| matcher.ignores(&relative)) {
            if file_type.is_file() {
                summary.excluded_files += 1;
                summary.excluded_bytes = summary.excluded_bytes.saturating_add(metadata.len());
            }
            continue;
        }

        seen.insert(rel_path.clone());
        if file_type.is_symlink() {
            if !manifest_v2 {
                summary.failures += 1;
                continue;
            }
            match fs::read_link(&path) {
                Ok(target) => {
                    manifest.insert(
                        rel_path,
                        BoxStoreManifestEntry::Symlink {
                            target: target.to_string_lossy().to_string(),
                        },
                    );
                }
                Err(_) => summary.metadata_failures += 1,
            }
            continue;
        }
        if !file_type.is_file() {
            summary.metadata_failures += 1;
            continue;
        }

        summary.files_scanned += 1;
        match sync_file(store, manifest, &path, &rel_path, None, manifest_v2) {
            Ok((uploaded, size)) => {
                if uploaded {
                    summary.files_uploaded += 1;
                    summary.bytes_uploaded = summary.bytes_uploaded.saturating_add(size);
                } else {
                    summary.skipped_unchanged += 1;
                }
            }
            Err(error) if error == "oversize" => summary.oversize += 1,
            Err(_) => summary.failures += 1,
        }
    }
}

fn sync_file(
    store: &dyn BoxObjectStore,
    manifest: &mut BoxManifestMap,
    path: &Path,
    rel_path: &str,
    mode_override: Option<u32>,
    manifest_v2: bool,
) -> Result<(bool, u64), String> {
    let metadata = fs::metadata(path).map_err(|error| error.to_string())?;
    let size = metadata.len();
    if size > DEFAULT_MAX_OBJECT_BYTES {
        return Err("oversize".into());
    }
    let sha = sha256_file(path)?;
    let mode = mode_override.unwrap_or_else(|| file_mode(&metadata));
    let next = if manifest_v2 {
        BoxStoreManifestEntry::File {
            sha: sha.clone(),
            size,
            mode,
        }
    } else {
        BoxStoreManifestEntry::LegacyFile {
            sha: sha.clone(),
            size,
        }
    };
    if manifest.get(rel_path) == Some(&next) {
        return Ok((false, size));
    }
    let blob_key = format!("{BOX_STORE_BLOBS_PREFIX}/{sha}");
    if store.get(&blob_key)?.is_none() {
        store.put_from_file(&blob_key, path)?;
    }
    manifest.insert(rel_path.to_string(), next);
    Ok((true, size))
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let mut file = File::open(path).map_err(|error| error.to_string())?;
    let mut digest = Sha256::new();
    let mut buffer = [0_u8; 128 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        digest.update(&buffer[..read]);
    }
    Ok(format!("{:x}", digest.finalize()))
}

fn file_mode(metadata: &fs::Metadata) -> u32 {
    #[cfg(unix)]
    {
        metadata.permissions().mode() & 0o777
    }
    #[cfg(not(unix))]
    {
        if metadata.permissions().readonly() {
            0o444
        } else {
            0o666
        }
    }
}

fn is_excluded(rel_path: &str, excludes: &[&str]) -> bool {
    excludes.iter().any(|pattern| glob_matches_path(pattern, rel_path))
}

fn is_agent_store_db_path(rel_path: &str) -> bool {
    if !rel_path.contains("/agents/") {
        return false;
    }
    AGENT_STORE_DB_BASENAMES.iter().any(|basename| {
        rel_path.ends_with(&format!("/{basename}"))
            || rel_path.ends_with(&format!("/{basename}-wal"))
            || rel_path.ends_with(&format!("/{basename}-shm"))
            || rel_path.ends_with(&format!("/{basename}-journal"))
            || rel_path.contains(&format!("/{basename}.corrupt-"))
            || rel_path.contains(&format!("/{basename}.replacement"))
            || rel_path.ends_with(&format!("/{basename}.pending"))
    })
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_millis().min(u128::from(u64::MAX)) as u64)
        .unwrap_or(0)
}

fn sleep_interruptibly(stopped: &AtomicBool, total_ms: u64) -> bool {
    let mut remaining = total_ms;
    while remaining > 0 {
        if stopped.load(Ordering::Acquire) {
            return false;
        }
        let slice = remaining.min(POLL_SLEEP_SLICE_MS);
        thread::sleep(Duration::from_millis(slice));
        remaining -= slice;
    }
    !stopped.load(Ordering::Acquire)
}


#[cfg(test)]
mod tests {
    use super::*;

    fn temp_root(label: &str) -> PathBuf {
        let path = std::env::temp_dir().join(format!(
            "fabushi-box-store-production-{label}-{}",
            uuid::Uuid::new_v4().simple()
        ));
        fs::create_dir_all(&path).expect("create temp root");
        path
    }

    #[test]
    fn local_snapshot_round_trips_blob_manifest_ignore_and_pruning() {
        let root = temp_root("snapshot");
        let workspace = root.join("workspace");
        let store_root = root.join("store");
        fs::create_dir_all(workspace.join("src")).expect("create source dir");
        fs::create_dir_all(workspace.join("node_modules/pkg")).expect("create ignored dir");
        fs::write(workspace.join("src/main.ts"), b"export const value = 42;\n")
            .expect("write source file");
        fs::write(workspace.join("node_modules/pkg/index.js"), b"ignored")
            .expect("write ignored file");

        let store = LocalFsObjectStore::new(store_root.clone());
        let ignore = load_workspace_ignore(
            &workspace,
            SAND_BOX_WORKSPACE_DEFAULT_IGNORE_PATTERNS,
        );
        let mut manifest = BoxManifestMap::new();
        let summary = sync_tree_category(
            &store,
            &mut manifest,
            &workspace,
            WORKSPACE_REL_PREFIX,
            "workspace",
            &[],
            Some(&ignore),
            true,
        )
        .expect("snapshot workspace");

        assert_eq!(summary.failures, 0);
        assert_eq!(summary.metadata_failures, 0);
        assert_eq!(summary.files_uploaded, 1);
        assert!(manifest.contains_key("workspace/src/main.ts"));
        assert!(!manifest.contains_key("workspace/node_modules/pkg/index.js"));

        let entry = manifest
            .get("workspace/src/main.ts")
            .expect("source manifest entry");
        let sha = match entry {
            BoxStoreManifestEntry::File { sha, size, mode: _ } => {
                assert_eq!(*size, b"export const value = 42;\n".len() as u64);
                sha.clone()
            }
            other => panic!("expected v2 file entry, got {other:?}"),
        };
        assert!(store_root.join(BOX_STORE_BLOBS_PREFIX).join(&sha).is_file());

        write_manifest(&store, &manifest, true, "test-window".into())
            .expect("write manifest");
        let persisted = load_manifest(&store)
            .expect("read manifest")
            .expect("manifest exists");
        assert_eq!(persisted.version, BOX_STORE_MANIFEST_VERSION);
        assert_eq!(persisted.writer_window_id.as_deref(), Some("test-window"));
        assert_eq!(persisted.entries, manifest);

        fs::remove_file(workspace.join("src/main.ts")).expect("remove source file");
        let summary = sync_tree_category(
            &store,
            &mut manifest,
            &workspace,
            WORKSPACE_REL_PREFIX,
            "workspace",
            &[],
            Some(&ignore),
            true,
        )
        .expect("snapshot pruned workspace");
        assert_eq!(summary.removed, 1);
        assert!(!manifest.contains_key("workspace/src/main.ts"));

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[test]
    fn manifest_v1_fails_closed_for_symlink_semantics() {
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;

            let root = temp_root("symlink");
            let workspace = root.join("workspace");
            let store_root = root.join("store");
            fs::create_dir_all(&workspace).expect("create workspace");
            fs::write(workspace.join("target.txt"), b"target").expect("write target");
            symlink("target.txt", workspace.join("link.txt")).expect("create symlink");

            let store = LocalFsObjectStore::new(store_root);
            let mut manifest = BoxManifestMap::new();
            let summary = sync_tree_category(
                &store,
                &mut manifest,
                &workspace,
                WORKSPACE_REL_PREFIX,
                "workspace",
                &[],
                None,
                false,
            )
            .expect("snapshot v1 workspace");

            assert!(summary.failures >= 1);
            assert!(!manifest.contains_key("workspace/link.txt"));
            fs::remove_dir_all(root).expect("cleanup temp root");
        }
    }
}
