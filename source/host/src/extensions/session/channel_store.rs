use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::Serialize;
use serde_json::Value;

use crate::storage::folder_id::is_safe_folder_id;

pub const CHANNELS_DIRNAME: &str = "channels";
pub const CHANNEL_CONFIG_FILENAME: &str = "connection.json";
pub const CHANNEL_CHANGE_DEBOUNCE_MS: u64 = 50;

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

#[derive(Clone)]
pub struct FileChannelStore {
    channels_dir: PathBuf,
    on_change: Arc<Mutex<Option<ChannelChangeListener>>>,
}

impl FileChannelStore {
    pub fn new(channels_dir: impl Into<PathBuf>) -> Self {
        Self {
            channels_dir: channels_dir.into(),
            on_change: Arc::new(Mutex::new(None)),
        }
    }

    pub fn get_location(&self) -> &Path {
        &self.channels_dir
    }

    pub fn set_on_change(&self, on_change: Option<ChannelChangeListener>) {
        if let Ok(mut slot) = self.on_change.lock() {
            *slot = on_change;
        }
    }

    pub fn config_path(&self, platform: &str) -> PathBuf {
        self.channels_dir
            .join(platform)
            .join(CHANNEL_CONFIG_FILENAME)
    }

    pub fn list_platforms(&self) -> Vec<String> {
        let Ok(entries) = fs::read_dir(&self.channels_dir) else {
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
        self.notify();
        Ok(true)
    }

    pub fn remove(&self, platform: &str) -> Result<bool, ChannelStoreError> {
        if !is_safe_folder_id(platform) {
            return Ok(false);
        }
        let platform_dir = self.channels_dir.join(platform);
        match fs::metadata(&platform_dir) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => return Ok(false),
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
            Err(error) => return Err(error.into()),
        }
        fs::remove_dir_all(platform_dir)?;
        self.notify();
        Ok(true)
    }

    fn notify(&self) {
        let callback = self
            .on_change
            .lock()
            .ok()
            .and_then(|slot| slot.as_ref().map(Arc::clone));
        if let Some(callback) = callback {
            callback();
        }
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
