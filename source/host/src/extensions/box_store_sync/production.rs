use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, File};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Condvar, Mutex,
    atomic::{AtomicBool, AtomicUsize, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

use serde_json::{Map, Value, json};
use sha2::{Digest, Sha256};

use crate::extensions::box_store_sync::box_object_store::{
    AgentStoreObjectStoreProvider, BoxObjectStore, BoxObjectStoreProvider, LocalFsObjectStore,
};
use crate::extensions::box_store_sync::agent_store_sand_files::AgentStoreClientDependencies;
use crate::extensions::box_store_sync::box_store_pack::{
    BOX_STORE_PACK_INDEX_KEY, BOX_STORE_PACK_RETIRED_KEY, BOX_STORE_PACKS_PREFIX,
    PACK_INDEX_VERSION, PACK_MAX_MEMBER_SIZE_SUM, PACK_MEMBER_MAX_BYTES, PackEntry, PackIndex,
    PackSource, build_pack_file, is_box_store_pack_build_enabled, parse_pack_index,
    parse_pack_retired, plan_pack_maintenance, serialize_pack_index, serialize_pack_retired,
};
use crate::extensions::box_store_sync::box_store_pack_pipeline::{
    PACK_BUILD_MIN_BYTES, PACK_BUILD_MIN_MEMBERS, PACK_TMP_DIR_NAME, PACK_TMP_MAX_AGE_MS,
};
use crate::extensions::box_store_sync::box_store_manifest::{
    AGENT_STORE_DB_BASENAMES, BoxManifestMap, set_manifest_entry,
};
use crate::extensions::box_store_sync::box_store_manifest_format::{
    BOX_STORE_BLOBS_PREFIX, BOX_STORE_LEGACY_MANIFEST_VERSION, BOX_STORE_MANIFEST_REL_PATH,
    BOX_STORE_MANIFEST_VERSION, BoxStoreManifest, BoxStoreManifestEntry, SAND_MANIFEST_V2_ENV,
    parse_box_store_manifest,
};
use crate::extensions::box_store_sync::box_store_transfer::{
    BOX_STORE_SNAPSHOT_TMP_SUFFIX, CategoryTransferSummary, glob_matches_path,
};
use crate::extensions::box_store_sync::box_store_sync::LARGE_OBJECT_THRESHOLD_BYTES;
use crate::extensions::box_store_sync::box_store_sync_service::BOX_STORE_CHROME_INTERVAL_MS;
use crate::extensions::box_store_sync::store_db_snapshot_upload::{
    SnapshotUploadOutcome, StoreDbSnapshotRuntimePort, StoreDbSnapshotUpload,
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
use crate::storage::store_db::{
    get_sand_agent_db_write_generation, has_live_sand_agent_db_handle,
};
use crate::extensions::box_store_sync::store_db_bundle_capture::AgentDbCaptureQueues;

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
    AgentStore,
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
        self.inner.flush_waiters.fetch_add(1, Ordering::AcqRel);
        let result = self.inner.run_local_cycle(false, true, false, false);
        self.inner.flush_waiters.fetch_sub(1, Ordering::AcqRel);
        result?;
        Ok(self.status())
    }

    pub fn schedule_store_db_snapshot(&self, agent_id: &str) -> bool {
        let agent_id = agent_id.trim();
        if agent_id.is_empty()
            || self.inner.stopped.load(Ordering::Acquire)
            || !matches!(
                &self.inner.mode,
                ProductionBoxStoreSyncMode::LocalFs { .. } | ProductionBoxStoreSyncMode::AgentStore
            )
        {
            return false;
        }
        let delay = Duration::from_millis(
            self.inner.deps.scheduling.store_db_debounce.delay_ms.max(1),
        );
        let mut queue = self
            .inner
            .store_db_debounce
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        queue.schedule(agent_id.to_string(), Instant::now() + delay);
        drop(queue);
        self.inner.store_db_wake.notify_one();
        true
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
    store_db_worker: Mutex<Option<JoinHandle<()>>>,
}

impl ProductionBoxStoreSyncService {
    pub fn new(deps: BoxStoreSyncExtensionDeps) -> Self {
        let env = std::env::vars().collect::<BTreeMap<_, _>>();
        let mode = resolve_production_box_store_sync_mode(&env);
        let backend = mode_name(&mode).to_string();
        let enabled = matches!(
            mode,
            ProductionBoxStoreSyncMode::LocalFs { .. } | ProductionBoxStoreSyncMode::AgentStore
        );
        Self {
            inner: Arc::new(ProductionBoxStoreSyncInner {
                deps,
                env,
                mode,
                stopped: AtomicBool::new(false),
                cycle_lock: Mutex::new(()),
                store_db_debounce: Mutex::new(StoreDbDebounceQueue::default()),
                store_db_wake: Condvar::new(),
                agent_db_capture_queues: AgentDbCaptureQueues::default(),
                last_pack_sync: Mutex::new(None),
                flush_waiters: AtomicUsize::new(0),
                status: Mutex::new(ProductionBoxStoreSyncStatus {
                    enabled,
                    backend,
                    ..ProductionBoxStoreSyncStatus::default()
                }),
            }),
            watcher: Mutex::new(None),
            poller: Mutex::new(None),
            store_db_worker: Mutex::new(None),
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
            ProductionBoxStoreSyncMode::LocalFs { .. }
            | ProductionBoxStoreSyncMode::AgentStore => {}
        }

        let chrome_inner = Arc::clone(&self.inner);
        let watcher = ChromeSessionWatcher::with_logger(
            CHROME_SESSION_DB_DIR,
            CHROME_SESSION_DB_NAMES.iter().copied(),
            self.inner.deps.scheduling.chrome_session_debounce.delay_ms,
            Arc::new(move || {
                if let Err(error) = chrome_inner.run_local_cycle(true, false, false, false) {
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

        let store_db_inner = Arc::clone(&self.inner);
        match thread::Builder::new()
            .name("box-store-db-debounce".into())
            .spawn(move || run_store_db_debounce_loop(store_db_inner))
        {
            Ok(handle) => {
                *self
                    .store_db_worker
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(handle);
            }
            Err(error) => {
                self.inner.log(&format!("store.db debounce worker failed to start: {error}"));
                self.inner
                    .diagnostic("startup", "store-db-debounce-worker-start-failed", false);
            }
        }

        let poll_inner = Arc::clone(&self.inner);
        let interval_ms = self.inner.deps.scheduling.polling.interval_ms.max(1);
        let handle = thread::Builder::new()
            .name("box-store-sync-poller".into())
            .spawn(move || {
                while !poll_inner.stopped.load(Ordering::Acquire) {
                    if !sleep_interruptibly(&poll_inner.stopped, interval_ms) {
                        break;
                    }
                    let idle = (poll_inner.deps.is_idle)();
                    let include_store_dbs = idle;
                    let include_packs = idle && poll_inner.pack_sync_due();
                    if let Err(error) =
                        poll_inner.run_local_cycle(false, include_store_dbs, true, include_packs)
                    {
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
                let backend = mode_name(&self.inner.mode);
                self.inner.log(&format!("box-store sync {backend} production slice enabled"));
                self.inner
                    .diagnostic("startup", &format!("enabled-{backend}"), true);
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
        self.inner.store_db_wake.notify_all();
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
        if let Some(handle) = self
            .store_db_worker
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
        {
            let _ = handle.join();
        }
    }
}

#[derive(Default)]
struct StoreDbDebounceQueue {
    pending: BTreeMap<String, Instant>,
}

impl StoreDbDebounceQueue {
    fn schedule(&mut self, agent_id: String, deadline: Instant) {
        self.pending.insert(agent_id, deadline);
    }

    fn take_due(&mut self, now: Instant) -> Vec<String> {
        let due = self
            .pending
            .iter()
            .filter(|(_, deadline)| **deadline <= now)
            .map(|(agent_id, _)| agent_id.clone())
            .collect::<Vec<_>>();
        for agent_id in &due {
            self.pending.remove(agent_id);
        }
        due
    }

    fn next_deadline(&self) -> Option<Instant> {
        self.pending.values().min().copied()
    }
}

fn run_store_db_debounce_loop(inner: Arc<ProductionBoxStoreSyncInner>) {
    loop {
        let due = {
            let mut queue = inner
                .store_db_debounce
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            loop {
                if inner.stopped.load(Ordering::Acquire) {
                    return;
                }
                let now = Instant::now();
                let due = queue.take_due(now);
                if !due.is_empty() {
                    break due;
                }
                queue = if let Some(deadline) = queue.next_deadline() {
                    let wait = deadline.saturating_duration_since(now);
                    let (queue, _) = inner
                        .store_db_wake
                        .wait_timeout(queue, wait)
                        .unwrap_or_else(|poisoned| poisoned.into_inner());
                    queue
                } else {
                    inner
                        .store_db_wake
                        .wait(queue)
                        .unwrap_or_else(|poisoned| poisoned.into_inner())
                };
            }
        };
        for agent_id in due {
            if inner.stopped.load(Ordering::Acquire) {
                return;
            }
            if let Err(error) = inner.run_local_agent_db_snapshot(&agent_id) {
                inner.log(&format!(
                    "turn-end store.db snapshot rejected for {agent_id}: {error}"
                ));
                inner.diagnostic("store-db-turn-end", &error, false);
            }
        }
    }
}

struct ProductionBoxStoreSyncInner {
    deps: BoxStoreSyncExtensionDeps,
    env: BTreeMap<String, String>,
    mode: ProductionBoxStoreSyncMode,
    stopped: AtomicBool,
    cycle_lock: Mutex<()>,
    store_db_debounce: Mutex<StoreDbDebounceQueue>,
    store_db_wake: Condvar,
    agent_db_capture_queues: AgentDbCaptureQueues,
    last_pack_sync: Mutex<Option<Instant>>,
    flush_waiters: AtomicUsize,
    status: Mutex<ProductionBoxStoreSyncStatus>,
}

impl ProductionBoxStoreSyncInner {
    fn resolve_object_store(&self) -> Result<(String, Box<dyn BoxObjectStore>), String> {
        let store_id = match &self.mode {
            ProductionBoxStoreSyncMode::LocalFs {
                store_id_override: Some(store_id),
                ..
            } => store_id.clone(),
            ProductionBoxStoreSyncMode::LocalFs { .. } | ProductionBoxStoreSyncMode::AgentStore => {
                (self.deps.resolve_store_id)()?
            }
            _ => return Err("box-store backend is not active".into()),
        };

        if !is_frozen_agent_store_source_id(&store_id) {
            return Err(format!(
                "resolved BoxStore source id is not a frozen AgentStore source id: {store_id}"
            ));
        }

        match &self.mode {
            ProductionBoxStoreSyncMode::LocalFs { base_dir, .. } => Ok((
                store_id.clone(),
                Box::new(LocalFsObjectStore::new(base_dir.join(&store_id))),
            )),
            ProductionBoxStoreSyncMode::AgentStore => {
                let backend_url = self
                    .deps
                    .backend_url
                    .clone()
                    .ok_or_else(|| "AgentStore backend URL is not configured".to_string())?;
                let get_access_token = self
                    .deps
                    .get_access_token
                    .clone()
                    .ok_or_else(|| "AgentStore auth token resolver is not configured".to_string())?;
                let get_machine_id = self
                    .deps
                    .get_machine_id
                    .clone()
                    .ok_or_else(|| "AgentStore machine id resolver is not configured".to_string())?;
                let provider = AgentStoreObjectStoreProvider::new(AgentStoreClientDependencies {
                    backend_url,
                    get_access_token,
                    get_machine_id,
                })?;
                Ok((store_id.clone(), provider.for_store(&store_id)))
            }
            _ => Err("box-store backend is not active".into()),
        }
    }

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

    fn pack_sync_due(&self) -> bool {
        let last = self
            .last_pack_sync
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
        match *last {
            None => true,
            Some(observed) => {
                observed.elapsed() >= Duration::from_millis(BOX_STORE_CHROME_INTERVAL_MS)
            }
        }
    }

    fn run_local_cycle(
        &self,
        chrome_only: bool,
        include_store_dbs: bool,
        skip_live_handle_store_dbs: bool,
        include_packs: bool,
    ) -> Result<(), String> {
        let _cycle = self
            .cycle_lock
            .lock()
            .map_err(|_| "box-store sync cycle lock poisoned".to_string())?;
        if self.stopped.load(Ordering::Acquire) {
            return Err("stopped".into());
        }
        let (store_id, store) = self.resolve_object_store()?;
        let manifest_baseline = store.get(BOX_STORE_MANIFEST_REL_PATH)?;
        let mut manifest = parse_manifest_bytes(manifest_baseline.as_deref())?;
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
            categories.push(sync_chrome_session(store.as_ref(), &mut entries, manifest_v2)?);
        } else {
            let sand_root = get_sand_root_dir();
            categories.push(sync_tree_category(
                store.as_ref(),
                &mut entries,
                &sand_root,
                SAND_DATA_REL_PREFIX,
                "sand-data",
                SAND_DATA_EXCLUDES,
                None,
                manifest_v2,
            )?);
            if include_store_dbs {
                categories.push(sync_store_db_snapshots(
                    store.as_ref(),
                    &store_id,
                    &mut entries,
                    &sand_root,
                    skip_live_handle_store_dbs,
                    None,
                )?);
            }
            let workspace_ignore =
                load_workspace_ignore(WORKSPACE_ROOT, SAND_BOX_WORKSPACE_DEFAULT_IGNORE_PATTERNS);
            categories.push(sync_tree_category(
                store.as_ref(),
                &mut entries,
                Path::new(WORKSPACE_ROOT),
                WORKSPACE_REL_PREFIX,
                "workspace",
                &[],
                Some(&workspace_ignore),
                manifest_v2,
            )?);
            categories.push(sync_tree_category(
                store.as_ref(),
                &mut entries,
                Path::new(CLI_CONFIG_ROOT),
                CLI_CONFIG_REL_PREFIX,
                "cli-config",
                CLI_CONFIG_EXCLUDES,
                None,
                manifest_v2,
            )?);
        }

        write_manifest_if_unchanged(
            store.as_ref(),
            manifest_baseline.as_deref(),
            &entries,
            manifest_v2,
            format!("mahayana-host-{}", std::process::id()),
        )?;

        if include_packs
            && is_box_store_pack_build_enabled(
                self.env.get("SAND_BOX_STORE_PACKS").map(String::as_str),
            )
        {
            let pack_summary = sync_local_packs(store.as_ref(), &entries, || {
                self.flush_waiters.load(Ordering::Acquire) > 0
            });
            if pack_summary.failures > 0 {
                self.log("pack maintenance round failed; preserving the committed manifest and retrying later");
            }
            categories.push(pack_summary);
            *self
                .last_pack_sync
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner()) = Some(Instant::now());
        }

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

    fn run_local_agent_db_snapshot(&self, agent_id: &str) -> Result<(), String> {
        self.agent_db_capture_queues.run_serialized(agent_id, || {
            self.run_local_agent_db_snapshot_unqueued(agent_id)
        })
    }

    fn run_local_agent_db_snapshot_unqueued(&self, agent_id: &str) -> Result<(), String> {
        let _cycle = self
            .cycle_lock
            .lock()
            .map_err(|_| "box-store sync cycle lock poisoned".to_string())?;
        if self.stopped.load(Ordering::Acquire) {
            return Err("stopped".into());
        }
        let (store_id, store) = self.resolve_object_store()?;
        let manifest_baseline = store.get(BOX_STORE_MANIFEST_REL_PATH)?;
        let mut parsed = parse_manifest_bytes(manifest_baseline.as_deref())?;
        let manifest_v2 = self
            .env
            .get(SAND_MANIFEST_V2_ENV)
            .is_some_and(|value| value == "1")
            || parsed
                .as_ref()
                .is_some_and(|value| value.version == BOX_STORE_MANIFEST_VERSION);
        let mut entries = parsed
            .take()
            .map(|value| value.entries)
            .unwrap_or_default();
        let entries_before = entries.clone();
        let summary = sync_store_db_snapshots(
            store.as_ref(),
            &store_id,
            &mut entries,
            &get_sand_root_dir(),
            false,
            Some(agent_id),
        )?;
        if entries != entries_before {
            write_manifest_if_unchanged(
                store.as_ref(),
                manifest_baseline.as_deref(),
                &entries,
                manifest_v2,
                format!("mahayana-host-{}", std::process::id()),
            )?;
        }
        let failures = summary.failures + summary.oversize + summary.metadata_failures;
        self.log(&format!(
            "turn-end store.db {agent_id}: {} file(s) / {}B uploaded, {} failure(s)",
            summary.files_uploaded, summary.bytes_uploaded, failures
        ));
        self.diagnostic(
            "store-db-turn-end",
            if failures == 0 { "ok" } else { "category-failures" },
            failures == 0,
        );
        if failures == 0 {
            Ok(())
        } else {
            Err(format!("store.db capture had {failures} failure(s)"))
        }
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
                .filter(|value| is_frozen_agent_store_source_id(value))
                .map(str::to_string);
            ProductionBoxStoreSyncMode::LocalFs {
                base_dir,
                store_id_override,
            }
        }
        BoxStoreBackendKind::AgentStore => ProductionBoxStoreSyncMode::AgentStore,
        backend => ProductionBoxStoreSyncMode::UnsupportedRemote { backend },
    }
}

fn is_frozen_agent_store_source_id(value: &str) -> bool {
    fn valid_uuid(value: &str) -> bool {
        let bytes = value.as_bytes();
        if bytes.len() != 36
            || bytes[8] != b'-'
            || bytes[13] != b'-'
            || bytes[18] != b'-'
            || bytes[23] != b'-'
        {
            return false;
        }
        if !matches!(bytes[14], b'1'..=b'5') {
            return false;
        }
        if !matches!(bytes[19].to_ascii_lowercase(), b'8' | b'9' | b'a' | b'b') {
            return false;
        }
        bytes.iter().enumerate().all(|(index, byte)| {
            matches!(index, 8 | 13 | 18 | 23) || byte.is_ascii_hexdigit()
        })
    }

    if valid_uuid(value) {
        return true;
    }
    let Some(rest) = value
        .strip_prefix("bc-")
        .or_else(|| value.strip_prefix("BC-"))
        .or_else(|| value.strip_prefix("Bc-"))
        .or_else(|| value.strip_prefix("bC-"))
    else {
        return false;
    };
    if rest.len() == 36 {
        return valid_uuid(rest);
    }
    if rest.len() <= 37 {
        return false;
    }
    let split = rest.len() - 36;
    let (prefix_with_dash, uuid) = rest.split_at(split);
    let Some(prefix) = prefix_with_dash.strip_suffix('-') else {
        return false;
    };
    !prefix.is_empty()
        && prefix.bytes().enumerate().all(|(index, byte)| {
            let lowered = byte.to_ascii_lowercase();
            (index == 0 && (lowered.is_ascii_lowercase() || lowered.is_ascii_digit()))
                || (index > 0
                    && (lowered.is_ascii_lowercase()
                        || lowered.is_ascii_digit()
                        || lowered == b'-'))
        })
        && valid_uuid(uuid)
}

fn mode_name(mode: &ProductionBoxStoreSyncMode) -> &'static str {
    match mode {
        ProductionBoxStoreSyncMode::Disabled => "disabled",
        ProductionBoxStoreSyncMode::LocalFs { .. } => "local-fs",
        ProductionBoxStoreSyncMode::AgentStore => "agent-store",
        ProductionBoxStoreSyncMode::UnsupportedRemote {
            backend: BoxStoreBackendKind::AgentStore,
        } => "agent-store-invalid",
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
    let bytes = store.get(BOX_STORE_MANIFEST_REL_PATH)?;
    parse_manifest_bytes(bytes.as_deref())
}

fn parse_manifest_bytes(bytes: Option<&[u8]>) -> Result<Option<BoxStoreManifest>, String> {
    let Some(bytes) = bytes else {
        return Ok(None);
    };
    let value = serde_json::from_slice::<Value>(bytes)
        .map_err(|error| format!("manifest JSON is invalid: {error}"))?;
    parse_box_store_manifest(&value)
        .map(Some)
        .ok_or_else(|| "manifest schema is invalid".to_string())
}

fn serialize_manifest(
    entries: &BoxManifestMap,
    manifest_v2: bool,
    writer_window_id: String,
) -> Result<Vec<u8>, String> {
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
    serde_json::to_vec(&payload).map_err(|error| error.to_string())
}

fn write_manifest_if_unchanged(
    store: &dyn BoxObjectStore,
    baseline: Option<&[u8]>,
    entries: &BoxManifestMap,
    manifest_v2: bool,
    writer_window_id: String,
) -> Result<(), String> {
    let bytes = serialize_manifest(entries, manifest_v2, writer_window_id)?;
    if store.put_if_unchanged(BOX_STORE_MANIFEST_REL_PATH, baseline, &bytes)? {
        Ok(())
    } else {
        Err("box-store manifest canonical write lost a concurrent-write race".into())
    }
}

fn write_manifest(
    store: &dyn BoxObjectStore,
    entries: &BoxManifestMap,
    manifest_v2: bool,
    writer_window_id: String,
) -> Result<(), String> {
    let bytes = serialize_manifest(entries, manifest_v2, writer_window_id)?;
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


#[derive(Clone)]
struct LocalFsStoreDbSnapshotRuntime {
    store: LocalFsObjectStore,
}

impl StoreDbSnapshotRuntimePort for LocalFsStoreDbSnapshotRuntime {
    fn put_bytes(
        &self,
        _store_id: &str,
        key: &str,
        bytes: &[u8],
        _content_addressed: bool,
    ) -> Result<(), String> {
        self.store.put(key, bytes)
    }

    fn put_from_file(
        &self,
        _store_id: &str,
        key: &str,
        path: &Path,
        _sha: &str,
        _size: u64,
    ) -> Result<(), String> {
        self.store.put_from_file(key, path)
    }
}


fn resolve_pack_source_path(rel_path: &str) -> Option<PathBuf> {
    for (prefix, root) in [
        (SAND_DATA_REL_PREFIX, get_sand_root_dir()),
        (WORKSPACE_REL_PREFIX, PathBuf::from(WORKSPACE_ROOT)),
        (CLI_CONFIG_REL_PREFIX, PathBuf::from(CLI_CONFIG_ROOT)),
    ] {
        if rel_path == prefix {
            return Some(root);
        }
        if let Some(suffix) = rel_path.strip_prefix(&format!("{prefix}/")) {
            return Some(root.join(suffix));
        }
    }
    None
}

fn sweep_pack_temp_dir(dir: &Path) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(metadata) = entry.metadata() else {
            continue;
        };
        let Ok(modified) = metadata.modified() else {
            continue;
        };
        if now
            .duration_since(modified)
            .is_ok_and(|age| age.as_millis() as u64 > PACK_TMP_MAX_AGE_MS)
        {
            let _ = fs::remove_file(path);
        }
    }
}

fn sync_local_packs<F>(
    store: &dyn BoxObjectStore,
    manifest: &BoxManifestMap,
    should_abort: F,
) -> CategoryTransferSummary
where
    F: Fn() -> bool,
{
    let mut summary = CategoryTransferSummary {
        name: "packs".into(),
        ..CategoryTransferSummary::default()
    };
    let result = (|| -> Result<(), String> {
    let mut live = HashMap::<String, u64>::new();
    let mut eligible = HashMap::<String, u64>::new();
    let mut local_by_sha = HashMap::<String, PathBuf>::new();

    for (rel_path, entry) in manifest {
        let (sha, size) = match entry {
            BoxStoreManifestEntry::LegacyFile { sha, size }
            | BoxStoreManifestEntry::File { sha, size, .. } => (sha, *size),
            BoxStoreManifestEntry::Symlink { .. } => continue,
        };
        live.insert(sha.clone(), size);
        if size >= PACK_MEMBER_MAX_BYTES || eligible.contains_key(sha) {
            continue;
        }
        let Some(abs_path) = resolve_pack_source_path(rel_path) else {
            continue;
        };
        let Ok(metadata) = fs::symlink_metadata(&abs_path) else {
            continue;
        };
        if !metadata.is_file() || metadata.len() != size {
            continue;
        }
        eligible.insert(sha.clone(), size);
        local_by_sha.insert(sha.clone(), abs_path);
    }

    let raw_index = store.get(BOX_STORE_PACK_INDEX_KEY)?;
    let index = raw_index
        .as_deref()
        .and_then(|raw| std::str::from_utf8(raw).ok())
        .and_then(parse_pack_index);
    let plan = plan_pack_maintenance(
        index.as_ref(),
        &live,
        &eligible,
        PACK_MAX_MEMBER_SIZE_SUM,
        PACK_BUILD_MIN_MEMBERS,
        PACK_BUILD_MIN_BYTES,
    );
    summary.skipped_unchanged = plan.kept_packs.len();
    summary.removed = plan.retired_pack_ids.len();
    summary.files_scanned = plan.new_packs.len();
    if plan.new_packs.is_empty() && plan.retired_pack_ids.is_empty() {
        return Ok(());
    }

    let tmp_dir = std::env::temp_dir().join(PACK_TMP_DIR_NAME);
    fs::create_dir_all(&tmp_dir).map_err(|error| error.to_string())?;
    sweep_pack_temp_dir(&tmp_dir);
    let mut built_packs = Vec::<PackEntry>::new();

    for (group_index, planned) in plan.new_packs.iter().enumerate() {
        if should_abort() {
            return Ok(());
        }
        let sources = planned
            .iter()
            .filter_map(|member| {
                local_by_sha.get(&member.sha).map(|abs_path| PackSource {
                    abs_path: abs_path.clone(),
                    sha: member.sha.clone(),
                    size: member.size,
                    vmtime: member.vmtime,
                })
            })
            .collect::<Vec<_>>();
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        let tmp_path = tmp_dir.join(format!(
            "build-{}-{nonce}-{group_index}",
            std::process::id()
        ));
        let built = match build_pack_file(&tmp_path, &sources, || should_abort()) {
            Ok(Some(value)) => value,
            Ok(None) => {
                let _ = fs::remove_file(&tmp_path);
                return Ok(());
            }
            Err(error) => {
                let _ = fs::remove_file(&tmp_path);
                return Err(error);
            }
        };
        let built_size_sum = built.members.iter().map(|member| member.size).sum::<u64>();
        if built.members.is_empty()
            || (built.members.len() < PACK_BUILD_MIN_MEMBERS
                && built_size_sum < PACK_BUILD_MIN_BYTES)
        {
            let _ = fs::remove_file(&tmp_path);
            continue;
        }

        let pack_id = sha256_file(&tmp_path)?;
        let key = format!("{BOX_STORE_PACKS_PREFIX}/{pack_id}");
        let upload = store.put_from_file(&key, &tmp_path);
        let _ = fs::remove_file(&tmp_path);
        upload?;
        summary.files_uploaded += 1;
        summary.bytes_uploaded = summary.bytes_uploaded.saturating_add(built.file_bytes);
        built_packs.push(PackEntry {
            id: pack_id,
            bytes: built.file_bytes,
            members: built.members,
        });
    }

    if should_abort() {
        return Ok(());
    }
    if built_packs.is_empty() && plan.retired_pack_ids.is_empty() {
        return Ok(());
    }

    let max_built_vmtime = built_packs
        .iter()
        .flat_map(|pack| pack.members.iter().map(|member| member.vmtime))
        .max()
        .unwrap_or(0);
    let next_index = PackIndex {
        version: PACK_INDEX_VERSION,
        max_vmtime: index
            .as_ref()
            .map(|index| index.max_vmtime)
            .unwrap_or(0)
            .max(max_built_vmtime),
        packs: plan
            .kept_packs
            .iter()
            .cloned()
            .chain(built_packs)
            .collect(),
    };
    let referenced = next_index
        .packs
        .iter()
        .map(|pack| pack.id.as_str())
        .collect::<HashSet<_>>();
    let retiring = plan
        .retired_pack_ids
        .iter()
        .filter(|id| !referenced.contains(id.as_str()))
        .cloned()
        .collect::<Vec<_>>();

    let raw_retired = store.get(BOX_STORE_PACK_RETIRED_KEY)?;
    let existing_retired = match raw_retired.as_deref() {
        None => Vec::new(),
        Some(raw) => {
            let text = match std::str::from_utf8(raw) {
                Ok(value) => value,
                Err(_) if retiring.is_empty() => "",
                Err(_) => {
                    return Err("packs/retired.json is present but unreadable; refusing to overwrite".into());
                }
            };
            match parse_pack_retired(text) {
                Some(value) => value,
                None if retiring.is_empty() => Vec::new(),
                None => {
                    return Err("packs/retired.json is present but unreadable; refusing to overwrite".into());
                }
            }
        }
    };
    let mut merged_retired = existing_retired.clone();
    for pack_id in &retiring {
        if !merged_retired.contains(pack_id) {
            merged_retired.push(pack_id.clone());
        }
    }
    merged_retired.retain(|id| !referenced.contains(id.as_str()));
    if merged_retired != existing_retired {
        let bytes = serialize_pack_retired(&merged_retired)
            .map_err(|error| error.to_string())?;
        store.put(BOX_STORE_PACK_RETIRED_KEY, bytes.as_bytes())?;
    }

    let index_bytes = serialize_pack_index(&next_index).map_err(|error| error.to_string())?;
    store.put(BOX_STORE_PACK_INDEX_KEY, index_bytes.as_bytes())?;
    for pack_id in retiring {
        let _ = store.delete(&format!("{BOX_STORE_PACKS_PREFIX}/{pack_id}"));
    }
        Ok(())
    })();
    if result.is_err() {
        summary.failures += 1;
    }
    summary
}

fn sync_store_db_snapshots(
    store: &dyn BoxObjectStore,
    store_id: &str,
    manifest: &mut BoxManifestMap,
    sand_root: &Path,
    skip_live_handles: bool,
    only_agent_id: Option<&str>,
) -> Result<CategoryTransferSummary, String> {
    let mut summary = CategoryTransferSummary {
        name: "store.db".into(),
        ..CategoryTransferSummary::default()
    };
    let agents_root = sand_root.join("agents");
    let mut seen = HashSet::new();
    let mut blocked_from_prune = HashSet::new();
    let mut walk_complete = true;
    let agent_dirs = match fs::read_dir(&agents_root) {
        Ok(entries) => Some(entries),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => None,
        Err(_) => {
            summary.metadata_failures += 1;
            walk_complete = false;
            None
        }
    };

    let runtime: Arc<dyn StoreDbSnapshotRuntimePort> = Arc::new(LocalFsStoreDbSnapshotRuntime {
        store: store.clone(),
    });
    let uploader = StoreDbSnapshotUpload::new(
        runtime,
        Arc::new(now_ms),
        Arc::new(|message| eprintln!("[box-store-sync] {message}")),
        DEFAULT_MAX_OBJECT_BYTES,
        LARGE_OBJECT_THRESHOLD_BYTES,
    );

    if let Some(agent_dirs) = agent_dirs {
        for agent_entry in agent_dirs {
            let agent_entry = match agent_entry {
                Ok(entry) => entry,
                Err(_) => {
                    summary.metadata_failures += 1;
                    walk_complete = false;
                    continue;
                }
            };
            let file_type = match agent_entry.file_type() {
                Ok(file_type) => file_type,
                Err(_) => {
                    summary.metadata_failures += 1;
                    walk_complete = false;
                    continue;
                }
            };
            if !file_type.is_dir() {
                continue;
            }
            let agent_id = agent_entry.file_name().to_string_lossy().to_string();
            if agent_id.is_empty() {
                summary.metadata_failures += 1;
                walk_complete = false;
                continue;
            }
            if only_agent_id.is_some_and(|target| target != agent_id) {
                continue;
            }
            let agent_dir = agent_entry.path();
            let bundle_paths = AGENT_STORE_DB_BASENAMES
                .iter()
                .map(|basename| {
                    format!("{SAND_DATA_REL_PREFIX}/agents/{agent_id}/{basename}")
                })
                .collect::<Vec<_>>();

            if skip_live_handles
                && has_live_sand_agent_db_handle(&agent_dir.join("store.db"))
            {
                blocked_from_prune.extend(bundle_paths);
                continue;
            }
            if agent_has_pending_db_recovery(&agent_dir) {
                summary.failures += 1;
                blocked_from_prune.extend(bundle_paths);
                continue;
            }
            let initial_identity = match agent_db_bundle_identity(&agent_dir) {
                Ok(identity) => identity,
                Err(error) => {
                    eprintln!(
                        "[box-store-sync] agent db bundle identity failed {agent_id}: {error}"
                    );
                    summary.metadata_failures += 1;
                    blocked_from_prune.extend(bundle_paths);
                    continue;
                }
            };

            let mut staged_manifest = manifest.clone();
            let mut bundle_ok = true;
            let mut present_paths = HashSet::new();
            for basename in AGENT_STORE_DB_BASENAMES {
                let source_path = agent_dir.join(basename);
                let metadata = match fs::metadata(&source_path) {
                    Ok(metadata) if metadata.is_file() => metadata,
                    Ok(_) => {
                        summary.metadata_failures += 1;
                        bundle_ok = false;
                        continue;
                    }
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => continue,
                    Err(_) => {
                        summary.metadata_failures += 1;
                        bundle_ok = false;
                        continue;
                    }
                };
                let rel_path = format!(
                    "{SAND_DATA_REL_PREFIX}/agents/{agent_id}/{basename}"
                );
                seen.insert(rel_path.clone());
                present_paths.insert(rel_path.clone());
                summary.files_scanned += 1;

                let temp_path = PathBuf::from(format!(
                    "{}{}{}",
                    source_path.display(),
                    BOX_STORE_SNAPSHOT_TMP_SUFFIX,
                    uuid::Uuid::new_v4().simple(),
                ));
                if let Err(error) = uploader.run_vacuum_off_thread(&source_path, &temp_path) {
                    uploader.discard_snapshot_temp(&temp_path, &rel_path);
                    eprintln!(
                        "[box-store-sync] store.db snapshot capture failed {rel_path}: {error}"
                    );
                    summary.failures += 1;
                    bundle_ok = false;
                    continue;
                }

                let result = uploader.upload_agent_db_snapshot(
                    store_id,
                    &mut staged_manifest,
                    &rel_path,
                    &temp_path,
                    file_mode(&metadata),
                );
                uploader.discard_snapshot_temp(&temp_path, &rel_path);
                match result.outcome {
                    SnapshotUploadOutcome::Uploaded => {
                        summary.files_uploaded += 1;
                        summary.bytes_uploaded = summary
                            .bytes_uploaded
                            .saturating_add(result.bytes_uploaded);
                    }
                    SnapshotUploadOutcome::Unchanged => summary.skipped_unchanged += 1,
                    SnapshotUploadOutcome::Oversize => {
                        summary.oversize += 1;
                        bundle_ok = false;
                    }
                    SnapshotUploadOutcome::Error => {
                        summary.failures += 1;
                        bundle_ok = false;
                    }
                }
            }

            let final_identity = agent_db_bundle_identity(&agent_dir);
            if final_identity.as_ref().ok() != Some(&initial_identity) {
                summary.failures += 1;
                bundle_ok = false;
            }
            if !bundle_ok {
                blocked_from_prune.extend(bundle_paths);
                continue;
            }

            for basename in AGENT_STORE_DB_BASENAMES {
                let rel_path =
                    format!("{SAND_DATA_REL_PREFIX}/agents/{agent_id}/{basename}");
                if present_paths.contains(&rel_path) {
                    if let Some(entry) = staged_manifest.get(&rel_path).cloned() {
                        set_manifest_entry(manifest, &rel_path, entry);
                    }
                } else if manifest.remove(&rel_path).is_some() {
                    summary.removed += 1;
                }
            }
        }
    }

    if walk_complete && only_agent_id.is_none() {
        let prefix = format!("{SAND_DATA_REL_PREFIX}/agents/");
        let stale = manifest
            .keys()
            .filter(|path| path.starts_with(&prefix))
            .filter(|path| is_agent_store_db_path(path))
            .filter(|path| !seen.contains(*path))
            .filter(|path| !blocked_from_prune.contains(*path))
            .cloned()
            .collect::<Vec<_>>();
        for path in stale {
            manifest.remove(&path);
            summary.removed += 1;
        }
    }

    Ok(summary)
}

fn agent_has_pending_db_recovery(agent_dir: &Path) -> bool {
    let Ok(entries) = fs::read_dir(agent_dir) else {
        return true;
    };
    entries.filter_map(Result::ok).any(|entry| {
        let name = entry.file_name().to_string_lossy().to_string();
        name == "conversation-blobs.db.pending"
            || (name.starts_with("conversation-blobs.db.corrupt-")
                && (name.ends_with(".intent") || name.ends_with(".pending")))
    })
}

fn agent_db_bundle_identity(agent_dir: &Path) -> Result<String, String> {
    let mut fields = Vec::new();
    for basename in AGENT_STORE_DB_BASENAMES {
        let path = agent_dir.join(basename);
        match fs::metadata(&path) {
            Ok(metadata) => {
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                    .map(|value| value.as_nanos())
                    .unwrap_or_default();
                let wal_path = PathBuf::from(format!("{}-wal", path.display()));
                let (wal_size, wal_modified) = match fs::metadata(&wal_path) {
                    Ok(wal) => (
                        wal.len(),
                        wal.modified()
                            .ok()
                            .and_then(|value| value.duration_since(UNIX_EPOCH).ok())
                            .map(|value| value.as_nanos())
                            .unwrap_or_default(),
                    ),
                    Err(error) if error.kind() == std::io::ErrorKind::NotFound => (0, 0),
                    Err(error) => return Err(error.to_string()),
                };
                let write_generation = if basename == "store.db" {
                    get_sand_agent_db_write_generation(&path)
                } else {
                    0
                };
                fields.push(format!(
                    "{basename}:present:{write_generation}:{}:{modified}:{}:{wal_size}:{wal_modified}",
                    metadata.len(),
                    file_mode(&metadata),
                ));
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fields.push(format!("{basename}:absent"));
            }
            Err(error) => return Err(error.to_string()),
        }
    }
    Ok(fields.join("|"))
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
    fn conditional_manifest_write_rejects_stale_cycle_baseline() {
        let root = temp_root("manifest-cas");
        let store = LocalFsObjectStore::new(root.join("store"));
        let mut first = BoxManifestMap::new();
        first.insert(
            "workspace/a.txt".into(),
            BoxStoreManifestEntry::File {
                sha: "a".into(),
                size: 1,
                mode: 0o644,
            },
        );
        write_manifest(store.as_ref(), &first, true, "writer-a".into()).expect("seed manifest");
        let baseline = store
            .get(BOX_STORE_MANIFEST_REL_PATH)
            .expect("read baseline")
            .expect("baseline exists");

        let mut concurrent = first.clone();
        concurrent.insert(
            "workspace/b.txt".into(),
            BoxStoreManifestEntry::File {
                sha: "b".into(),
                size: 1,
                mode: 0o644,
            },
        );
        write_manifest(store.as_ref(), &concurrent, true, "writer-b".into())
            .expect("concurrent manifest");

        let error = write_manifest_if_unchanged(
            store.as_ref(),
            Some(&baseline),
            &first,
            true,
            "stale-writer".into(),
        )
        .expect_err("stale writer must lose");
        assert!(error.contains("concurrent-write race"));

        let persisted = load_manifest(store.as_ref())
            .expect("load winning manifest")
            .expect("manifest exists");
        assert!(persisted.entries.contains_key("workspace/b.txt"));

        fs::remove_dir_all(root).expect("cleanup");
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
            store.as_ref(),
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

        write_manifest(store.as_ref(), &manifest, true, "test-window".into())
            .expect("write manifest");
        let persisted = load_manifest(store.as_ref())
            .expect("read manifest")
            .expect("manifest exists");
        assert_eq!(persisted.version, BOX_STORE_MANIFEST_VERSION);
        assert_eq!(persisted.writer_window_id.as_deref(), Some("test-window"));
        assert_eq!(persisted.entries, manifest);

        fs::remove_file(workspace.join("src/main.ts")).expect("remove source file");
        let summary = sync_tree_category(
            store.as_ref(),
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
    fn local_snapshot_captures_agent_store_databases_instead_of_skipping_them() {
        let root = temp_root("store-db");
        let sand_root = root.join("sand-data");
        let agent_dir = sand_root.join("agents/agent-a");
        let store_root = root.join("store");
        fs::create_dir_all(&agent_dir).expect("create agent dir");

        let source_db = agent_dir.join("store.db");
        {
            let db = rusqlite::Connection::open(&source_db).expect("open source store.db");
            db.execute_batch(
                "CREATE TABLE state (id INTEGER PRIMARY KEY, value TEXT);
                 INSERT INTO state(value) VALUES ('durable');",
            )
            .expect("seed source store.db");
        }

        let store = LocalFsObjectStore::new(&store_root);
        let mut manifest = BoxManifestMap::new();
        let summary = sync_store_db_snapshots(
            store.as_ref(),
            "store-a",
            &mut manifest,
            &sand_root,
            false,
            None,
        )
        .expect("snapshot store.db");

        assert_eq!(summary.failures, 0);
        assert_eq!(summary.metadata_failures, 0);
        assert_eq!(summary.files_scanned, 1);
        assert_eq!(summary.files_uploaded, 1);
        let rel_path = "home/box/sand-data/agents/agent-a/store.db";
        let sha = match manifest.get(rel_path).expect("store.db manifest entry") {
            BoxStoreManifestEntry::File { sha, size, .. } => {
                assert!(*size > 0);
                sha.clone()
            }
            other => panic!("expected file manifest entry, got {other:?}"),
        };
        assert!(store_root.join(BOX_STORE_BLOBS_PREFIX).join(sha).is_file());

        fs::remove_file(&source_db).expect("remove source db");
        let summary = sync_store_db_snapshots(
            store.as_ref(),
            "store-a",
            &mut manifest,
            &sand_root,
            false,
            None,
        )
        .expect("prune removed store.db");
        assert_eq!(summary.removed, 1);
        assert!(!manifest.contains_key(rel_path));

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[test]
    fn agent_db_bundle_does_not_publish_partial_manifest_on_capture_failure() {
        let root = temp_root("store-db-bundle-failure");
        let sand_root = root.join("sand-data");
        let agent_dir = sand_root.join("agents/agent-a");
        let store_root = root.join("store");
        fs::create_dir_all(&agent_dir).expect("create agent dir");

        let source_db = agent_dir.join("store.db");
        {
            let db = rusqlite::Connection::open(&source_db).expect("open source store.db");
            db.execute_batch(
                "CREATE TABLE state (id INTEGER PRIMARY KEY, value TEXT);
                 INSERT INTO state(value) VALUES ('durable');",
            )
            .expect("seed source store.db");
        }
        fs::write(agent_dir.join("conversation-blobs.db"), b"not-a-sqlite-db")
            .expect("write corrupt conversation blob db");

        let store = LocalFsObjectStore::new(&store_root);
        let mut manifest = BoxManifestMap::new();
        let summary = sync_store_db_snapshots(
            store.as_ref(),
            "store-a",
            &mut manifest,
            &sand_root,
            false,
            None,
        )
        .expect("run failing bundle snapshot");

        assert!(summary.failures >= 1);
        assert!(
            !manifest.contains_key("home/box/sand-data/agents/agent-a/store.db"),
            "a successful first DB must not publish a partial Agent DB bundle"
        );
        assert!(
            !manifest.contains_key(
                "home/box/sand-data/agents/agent-a/conversation-blobs.db"
            )
        );

        fs::remove_dir_all(root).expect("cleanup temp root");
    }

    #[test]
    fn store_db_periodic_sweep_skips_live_handle_without_pruning() {
        let root = temp_root("store-db-live-handle");
        let sand_root = root.join("sand-data");
        let agent_dir = sand_root.join("agents/agent-a");
        let store_root = root.join("store");
        fs::create_dir_all(&agent_dir).expect("create agent dir");
        let source_db = agent_dir.join("store.db");
        {
            let db = rusqlite::Connection::open(&source_db).expect("open store.db");
            db.execute_batch(
                "CREATE TABLE state (id INTEGER PRIMARY KEY, value TEXT);
                 INSERT INTO state(value) VALUES ('first');",
            )
            .expect("seed store.db");
        }
        let store = LocalFsObjectStore::new(&store_root);
        let mut manifest = BoxManifestMap::new();
        sync_store_db_snapshots(
            store.as_ref(),
            "store-a",
            &mut manifest,
            &sand_root,
            false,
            None,
        )
        .expect("initial capture");
        let baseline = manifest.clone();

        crate::storage::store_db::register_live_db_handle(&source_db);
        {
            let db = rusqlite::Connection::open(&source_db).expect("reopen store.db");
            db.execute("INSERT INTO state(value) VALUES ('second')", [])
                .expect("mutate live db");
        }
        let summary = sync_store_db_snapshots(
            store.as_ref(),
            "store-a",
            &mut manifest,
            &sand_root,
            true,
            None,
        )
        .expect("live-handle sweep");
        crate::storage::store_db::release_live_db_handle(&source_db);

        assert_eq!(summary.files_scanned, 0);
        assert_eq!(summary.files_uploaded, 0);
        assert_eq!(manifest, baseline, "live Agent DB must remain durable but untouched");
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn store_db_bundle_identity_fences_shipping_write_generation() {
        let root = temp_root("store-db-generation");
        let agent_dir = root.join("agents/agent-a");
        fs::create_dir_all(&agent_dir).expect("create agent dir");
        let source_db = agent_dir.join("store.db");
        fs::write(&source_db, b"sqlite-placeholder").expect("seed store.db");

        let before = agent_db_bundle_identity(&agent_dir).expect("identity before");
        crate::storage::store_db::bump_db_write_generation(&source_db);
        let after = agent_db_bundle_identity(&agent_dir).expect("identity after");
        crate::storage::store_db::delete_sand_agent_db_write_generation(&source_db);

        assert_ne!(before, after, "a shipping DB mutation must invalidate capture identity");
        fs::remove_dir_all(root).expect("cleanup");
    }

    #[test]
    fn store_db_debounce_queue_coalesces_per_agent_and_requeues_after_fire() {
        let now = Instant::now();
        let mut queue = StoreDbDebounceQueue::default();
        queue.schedule("agent-a".into(), now + Duration::from_millis(50));
        queue.schedule("agent-a".into(), now + Duration::from_millis(100));
        queue.schedule("agent-b".into(), now + Duration::from_millis(75));

        assert!(queue.take_due(now + Duration::from_millis(60)).is_empty());
        assert_eq!(
            queue.take_due(now + Duration::from_millis(80)),
            vec!["agent-b".to_string()]
        );
        assert_eq!(
            queue.take_due(now + Duration::from_millis(120)),
            vec!["agent-a".to_string()]
        );

        queue.schedule("agent-a".into(), now + Duration::from_millis(150));
        assert_eq!(
            queue.take_due(now + Duration::from_millis(160)),
            vec!["agent-a".to_string()]
        );
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
                store.as_ref(),
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
