use std::collections::hash_map::DefaultHasher;
use std::fs;
use std::hash::{Hash, Hasher};
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use crate::storage::folder_id::is_safe_folder_id;

pub const CHANNELS_DIRNAME: &str = "channels";
pub const CHANNEL_CONFIG_FILENAME: &str = "connection.json";
pub const CHANNEL_CHANGE_DEBOUNCE_MS: u64 = 50;
const CHANNEL_WATCH_POLL_MS: u64 = 25;

pub type ChannelChangeListener = Arc<dyn Fn() + Send + Sync + 'static>;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChannelConnection {
    pub platform: String,
    pub label: String,
    pub status: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ChannelConfig {
    pub platform: String,
    pub token: String,
    pub label: String,
}

#[derive(Debug, thiserror::Error)]
pub enum ChannelStoreError {
    #[error("channel store filesystem error: {0}")]
    Io(#[from] io::Error),
    #[error("channel store serialization error: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Default)]
struct DebounceState {
    callback: Option<ChannelChangeListener>,
}

struct WatchWorker {
    stop: Arc<AtomicBool>,
    handle: Option<JoinHandle<()>>,
}

impl WatchWorker {
    fn stop(mut self) {
        self.stop.store(true, Ordering::Release);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
    }
}

struct ChannelWatchInner {
    channels_dir: PathBuf,
    debounce: Arc<Mutex<DebounceState>>,
    watcher: Mutex<Option<WatchWorker>>,
}

impl Drop for ChannelWatchInner {
    fn drop(&mut self) {
        if let Ok(slot) = self.watcher.get_mut() {
            if let Some(worker) = slot.take() {
                worker.stop();
            }
        }
    }
}

#[derive(Clone)]
pub struct FileChannelStore {
    inner: Arc<ChannelWatchInner>,
}

impl FileChannelStore {
    pub fn new(channels_dir: impl Into<PathBuf>) -> Self {
        Self {
            inner: Arc::new(ChannelWatchInner {
                channels_dir: channels_dir.into(),
                debounce: Arc::new(Mutex::new(DebounceState::default())),
                watcher: Mutex::new(None),
            }),
        }
    }

    pub fn get_location(&self) -> &Path {
        &self.inner.channels_dir
    }

    pub fn set_on_change(&self, on_change: Option<ChannelChangeListener>) {
        let enabled = on_change.is_some();
        if let Ok(mut state) = self.inner.debounce.lock() {
            state.callback = on_change;
        }
        if enabled {
            self.ensure_watcher();
        } else {
            self.stop_watcher();
        }
    }

    pub fn config_path(&self, platform: &str) -> PathBuf {
        self.inner
            .channels_dir
            .join(platform)
            .join(CHANNEL_CONFIG_FILENAME)
    }

    pub fn list_platforms(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.inner.channels_dir) else {
            return Vec::new();
        };
        let mut platforms = entries
            .filter_map(Result::ok)
            .filter_map(|entry| {
                let name = entry.file_name().into_string().ok()?;
                let kind = entry.file_type().ok()?;
                (kind.is_dir() && is_safe_folder_id(&name) && self.read_label(&name).is_some())
                    .then_some(name)
            })
            .collect::<Vec<_>>();
        platforms.sort();
        platforms
    }

    pub fn read_label(&self, platform: &str) -> Option<String> {
        let raw = fs::read_to_string(self.config_path(platform)).ok()?;
        let value = serde_json::from_str::<Value>(&raw).ok()?;
        let object = value.as_object()?;
        Some(label_for(
            platform,
            object.get("label").and_then(Value::as_str),
        ))
    }

    pub fn list_connections(&self) -> Vec<ChannelConnection> {
        self.list_platforms()
            .into_iter()
            .map(|platform| ChannelConnection {
                label: self
                    .read_label(&platform)
                    .unwrap_or_else(|| platform.clone()),
                platform,
                status: "configured",
            })
            .collect()
    }

    pub fn write_metadata(
        &self,
        platform: &str,
        label: &str,
    ) -> Result<bool, ChannelStoreError> {
        if !is_safe_folder_id(platform) {
            return Ok(false);
        }
        let path = self.config_path(platform);
        let Some(parent) = path.parent() else {
            return Ok(false);
        };
        fs::create_dir_all(parent)?;
        let mut serialized = serde_json::to_string_pretty(&serde_json::json!({
            "label": label_for(platform, Some(label)),
        }))?;
        serialized.push('\n');
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos();
        let temporary = PathBuf::from(format!(
            "{}.{}.{}.tmp",
            path.display(),
            process::id(),
            nanos
        ));
        fs::write(&temporary, serialized)?;
        if let Err(error) = fs::rename(&temporary, &path) {
            let _ = fs::remove_file(&temporary);
            return Err(error.into());
        }
        Ok(true)
    }

    pub fn remove(&self, platform: &str) -> Result<bool, ChannelStoreError> {
        if !is_safe_folder_id(platform) {
            return Ok(false);
        }
        let platform_dir = self.inner.channels_dir.join(platform);
        match fs::metadata(&platform_dir) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        }
        fs::remove_dir_all(platform_dir)?;
        Ok(true)
    }

    fn ensure_watcher(&self) {
        let Ok(mut slot) = self.inner.watcher.lock() else {
            return;
        };
        if slot.is_some() {
            return;
        }
        let stop = Arc::new(AtomicBool::new(false));
        let worker_stop = Arc::clone(&stop);
        let channels_dir = self.inner.channels_dir.clone();
        let debounce = Arc::clone(&self.inner.debounce);
        // Establish the baseline before spawning so a mutation immediately
        // after set_on_change cannot become the watcher's initial snapshot.
        let initial_fingerprint = directory_fingerprint(&channels_dir);
        let handle = thread::Builder::new()
            .name("sand-channel-store-watch".into())
            .spawn(move || {
                let mut fingerprint = initial_fingerprint;
                let mut last_change_at: Option<Instant> = None;
                while !worker_stop.load(Ordering::Acquire) {
                    thread::sleep(Duration::from_millis(CHANNEL_WATCH_POLL_MS));
                    if worker_stop.load(Ordering::Acquire) {
                        break;
                    }
                    let next = directory_fingerprint(&channels_dir);
                    if next != fingerprint {
                        fingerprint = next;
                        last_change_at = Some(Instant::now());
                        continue;
                    }
                    if last_change_at.is_some_and(|changed_at| {
                        changed_at.elapsed()
                            >= Duration::from_millis(CHANNEL_CHANGE_DEBOUNCE_MS)
                    }) {
                        last_change_at = None;
                        let callback = debounce
                            .lock()
                            .ok()
                            .and_then(|state| state.callback.as_ref().map(Arc::clone));
                        if let Some(callback) = callback {
                            callback();
                        }
                    }
                }
            })
            .ok();
        *slot = Some(WatchWorker { stop, handle });
    }

    fn stop_watcher(&self) {
        let worker = self
            .inner
            .watcher
            .lock()
            .ok()
            .and_then(|mut slot| slot.take());
        if let Some(worker) = worker {
            worker.stop();
        }
    }
}

fn directory_fingerprint(root: &Path) -> u64 {
    let mut hasher = DefaultHasher::new();
    fingerprint_path(root, root, &mut hasher);
    hasher.finish()
}

fn fingerprint_path(root: &Path, path: &Path, hasher: &mut DefaultHasher) {
    let relative = path.strip_prefix(root).unwrap_or(path);
    relative.hash(hasher);
    let Ok(metadata) = fs::symlink_metadata(path) else {
        "missing".hash(hasher);
        return;
    };
    metadata.len().hash(hasher);
    metadata.file_type().is_dir().hash(hasher);
    metadata.file_type().is_file().hash(hasher);
    metadata.file_type().is_symlink().hash(hasher);
    if let Ok(modified) = metadata.modified()
        && let Ok(duration) = modified.duration_since(UNIX_EPOCH)
    {
        duration.as_nanos().hash(hasher);
    }
    if metadata.is_file() {
        if let Ok(bytes) = fs::read(path) {
            bytes.hash(hasher);
        }
        return;
    }
    if !metadata.is_dir() {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        return;
    };
    let mut children = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .collect::<Vec<_>>();
    children.sort();
    for child in children {
        fingerprint_path(root, &child, hasher);
    }
}

pub fn get_agent_channels_dir(agent_dir: &Path) -> PathBuf {
    agent_dir.join(CHANNELS_DIRNAME)
}

pub fn label_for(platform: &str, raw: Option<&str>) -> String {
    let clamped = raw
        .unwrap_or_default()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(80)
        .collect::<String>();
    if !clamped.is_empty() {
        return clamped;
    }
    match platform {
        "discord" => "Discord".to_string(),
        "slack" => "Slack".to_string(),
        _ => platform.to_string(),
    }
}
