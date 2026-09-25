use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicU64, Ordering},
};
use std::thread;
use std::time::Duration;

pub const SAND_STATE_BACKSTOP_REL_PATH: &str = "state/store.db";
pub const DEFAULT_MAX_SNAPSHOT_BYTES: usize = 64 * 1024 * 1024;
pub const STATE_BACKSTOP_DEBOUNCE_MS: u64 = 5_000;

pub trait StateBackstopObjectStore: Send + Sync {
    fn put(&self, path: &str, bytes: &[u8]) -> Result<(), String>;
    fn get(&self, path: &str) -> Result<Option<Vec<u8>>, String>;
}

pub type StateBackstopStoreProvider =
    Arc<dyn Fn(&str) -> Arc<dyn StateBackstopObjectStore> + Send + Sync>;
pub type StateBackstopSourceResolver =
    Arc<dyn Fn(&str) -> Result<String, String> + Send + Sync>;
pub type StateBackstopReadDb =
    Arc<dyn Fn(&Path) -> Result<Option<Vec<u8>>, String> + Send + Sync>;
pub type StateBackstopLog = Arc<dyn Fn(&str) + Send + Sync>;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateBackstopSnapshotResult {
    Uploaded { bytes: usize },
    Skipped { reason: String },
    Error { error: String },
}

pub struct StateBackstopOptions {
    pub object_store_provider: StateBackstopStoreProvider,
    pub source_id_for_agent: StateBackstopSourceResolver,
    pub agents_root_dir: PathBuf,
    pub read_db_bytes: StateBackstopReadDb,
    pub max_snapshot_bytes: usize,
    pub debounce: Duration,
    pub log: StateBackstopLog,
}

impl StateBackstopOptions {
    pub fn new(
        object_store_provider: StateBackstopStoreProvider,
        source_id_for_agent: StateBackstopSourceResolver,
        agents_root_dir: PathBuf,
        read_db_bytes: StateBackstopReadDb,
    ) -> Self {
        Self {
            object_store_provider,
            source_id_for_agent,
            agents_root_dir,
            read_db_bytes,
            max_snapshot_bytes: DEFAULT_MAX_SNAPSHOT_BYTES,
            debounce: Duration::from_millis(STATE_BACKSTOP_DEBOUNCE_MS),
            log: Arc::new(|message| eprintln!("[sand-state-backstop] {message}")),
        }
    }
}

pub fn read_store_db_bytes(
    db_path: &Path,
    checkpoint: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<Option<Vec<u8>>, String> {
    if !db_path.exists() {
        return Ok(None);
    }
    checkpoint(db_path)?;
    fs::read(db_path)
        .map(Some)
        .map_err(|error| error.to_string())
}

pub struct SandStateBackstop {
    object_store_provider: StateBackstopStoreProvider,
    source_id_for_agent: StateBackstopSourceResolver,
    agents_root_dir: PathBuf,
    read_db_bytes: StateBackstopReadDb,
    max_snapshot_bytes: usize,
    debounce: Duration,
    log: StateBackstopLog,
    disposed: Arc<AtomicBool>,
    next_generation: AtomicU64,
    pending: Arc<Mutex<HashMap<String, u64>>>,
}

impl SandStateBackstop {
    pub fn new(options: StateBackstopOptions) -> Arc<Self> {
        Arc::new(Self {
            object_store_provider: options.object_store_provider,
            source_id_for_agent: options.source_id_for_agent,
            agents_root_dir: options.agents_root_dir,
            read_db_bytes: options.read_db_bytes,
            max_snapshot_bytes: options.max_snapshot_bytes,
            debounce: options.debounce,
            log: options.log,
            disposed: Arc::new(AtomicBool::new(false)),
            next_generation: AtomicU64::new(1),
            pending: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    pub fn db_path_for(&self, agent_id: &str) -> PathBuf {
        self.agents_root_dir.join(agent_id).join("store.db")
    }

    pub fn schedule_snapshot(self: &Arc<Self>, agent_id: impl Into<String>) {
        if self.disposed.load(Ordering::Acquire) {
            return;
        }
        let agent_id = agent_id.into();
        let generation = self.next_generation.fetch_add(1, Ordering::AcqRel);
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .insert(agent_id.clone(), generation);

        let this = Arc::clone(self);
        let debounce = self.debounce;
        let _ = thread::Builder::new()
            .name("sand-state-backstop-snapshot".into())
            .spawn(move || {
                thread::sleep(debounce);
                if this.disposed.load(Ordering::Acquire) {
                    return;
                }
                let is_current = this
                    .pending
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .get(&agent_id)
                    .copied()
                    == Some(generation);
                if !is_current {
                    return;
                }
                let result = this.snapshot_now(&agent_id);
                if let StateBackstopSnapshotResult::Error { error } = result {
                    (this.log)(&format!(
                        "snapshot rejected for {agent_id}: {error}"
                    ));
                }
            });
    }

    pub fn snapshot_now(&self, agent_id: &str) -> StateBackstopSnapshotResult {
        let result = (|| -> Result<StateBackstopSnapshotResult, String> {
            let path = self.db_path_for(agent_id);
            let Some(bytes) = (self.read_db_bytes)(&path)? else {
                return Ok(StateBackstopSnapshotResult::Skipped {
                    reason: "no store.db".into(),
                });
            };
            if bytes.len() > self.max_snapshot_bytes {
                let reason = format!(
                    "store.db {}B over {}B cap",
                    bytes.len(),
                    self.max_snapshot_bytes
                );
                (self.log)(&format!("skip {agent_id}: {reason}"));
                return Ok(StateBackstopSnapshotResult::Skipped { reason });
            }
            let source_id = (self.source_id_for_agent)(agent_id)?;
            let store = (self.object_store_provider)(&source_id);
            store.put(SAND_STATE_BACKSTOP_REL_PATH, &bytes)?;
            (self.log)(&format!("uploaded {agent_id} ({}B)", bytes.len()));
            Ok(StateBackstopSnapshotResult::Uploaded { bytes: bytes.len() })
        })();

        match result {
            Ok(result) => result,
            Err(error) => {
                (self.log)(&format!("snapshot {agent_id} failed: {error}"));
                StateBackstopSnapshotResult::Error { error }
            }
        }
    }

    pub fn read_snapshot(&self, agent_id: &str) -> Result<Option<Vec<u8>>, String> {
        let source_id = (self.source_id_for_agent)(agent_id)?;
        (self.object_store_provider)(&source_id).get(SAND_STATE_BACKSTOP_REL_PATH)
    }

    pub fn dispose(&self) {
        self.disposed.store(true, Ordering::Release);
        self.pending
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .clear();
    }
}

pub fn is_state_backstop_enabled_value(raw: Option<&str>) -> bool {
    raw.map(str::trim)
        .map(str::to_ascii_lowercase)
        .is_some_and(|value| matches!(value.as_str(), "1" | "true" | "yes"))
}

pub fn is_state_backstop_enabled() -> bool {
    is_state_backstop_enabled_value(
        std::env::var("SAND_STATE_S3_BACKSTOP").ok().as_deref(),
    )
}
