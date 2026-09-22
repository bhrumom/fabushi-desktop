use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde_json::{Value, json};
use uuid::Uuid;

use crate::host_paths::get_host_secrets_path;

static MACHINE_ID_CACHE: OnceLock<Mutex<Option<String>>> = OnceLock::new();

fn machine_id_cache() -> &'static Mutex<Option<String>> {
    MACHINE_ID_CACHE.get_or_init(|| Mutex::new(None))
}

pub fn get_or_create_host_machine_id(path: Option<&Path>) -> io::Result<String> {
    let mut cache = machine_id_cache()
        .lock()
        .map_err(|_| io::Error::other("host machine-id cache poisoned"))?;
    if let Some(machine_id) = cache.as_ref() {
        return Ok(machine_id.clone());
    }

    let owned_path;
    let path = match path {
        Some(path) => path,
        None => {
            owned_path = get_host_secrets_path();
            &owned_path
        }
    };

    if let Some(machine_id) = read_machine_id(path) {
        *cache = Some(machine_id.clone());
        return Ok(machine_id);
    }

    let machine_id = Uuid::new_v4().to_string();
    write_machine_id(path, &machine_id)?;
    *cache = Some(machine_id.clone());
    Ok(machine_id)
}

pub fn read_machine_id(path: &Path) -> Option<String> {
    let raw = fs::read_to_string(path).ok()?;
    let parsed: Value = serde_json::from_str(&raw).ok()?;
    let object = parsed.as_object()?;
    let machine_id = object.get("machineId")?.as_str()?;
    (!machine_id.is_empty()).then(|| machine_id.to_string())
}

pub fn write_machine_id(path: &Path, machine_id: &str) -> io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let temp_path = temp_path_for(path);
    let bytes = serde_json::to_vec_pretty(&json!({ "machineId": machine_id }))
        .map_err(|error| io::Error::other(format!("serialize host secrets: {error}")))?;
    fs::write(&temp_path, bytes)?;
    fs::rename(temp_path, path)
}

fn temp_path_for(path: &Path) -> PathBuf {
    PathBuf::from(format!("{}.{}.tmp", path.to_string_lossy(), std::process::id()))
}

pub fn clear_host_machine_id_cache_for_tests() {
    if let Ok(mut cache) = machine_id_cache().lock() {
        *cache = None;
    }
}
