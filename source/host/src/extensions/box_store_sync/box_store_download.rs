use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use std::sync::Mutex;

use super::agent_store_sand_files::normalize_rel_path;

pub const COPY_IN_LARGE_BLOB_CONCURRENCY: usize = 4;
pub const COPY_IN_WAVE_SIZE: usize = 500;
pub const BOX_STORE_RESTORE_TMP_SUFFIX: &str = ".box-store-part-";
pub const LARGE_OBJECT_FREE_SPACE_FACTOR: u64 = 2;

pub fn is_critical_rel_path(rel_path: &str) -> bool {
    let basename = rel_path.rsplit('/').next().unwrap_or(rel_path);
    matches!(
        basename,
        "store.db"
            | "conversation-blobs.db"
            | "Cookies"
            | "Login Data"
            | "Web Data"
            | "source-map.json"
    )
}

pub fn resolve_restore_destination(target_root: &Path, rel_path: &str) -> Option<PathBuf> {
    let normalized = normalize_rel_path(rel_path).ok()?;
    Some(target_root.join(normalized))
}

pub fn symlink_target_stays_within_root(
    target_root: &Path,
    symlink_path: &Path,
    target: &str,
) -> bool {
    if Path::new(target).is_absolute() || target.contains('\0') {
        return false;
    }
    let Some(parent) = symlink_path.parent() else {
        return false;
    };
    let candidate = lexical_normalize(&parent.join(target));
    let root = lexical_normalize(target_root);
    candidate.starts_with(root)
}

fn lexical_normalize(path: &Path) -> PathBuf {
    let mut out = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    out
}

#[derive(Debug)]
pub struct BoxStoreByteBudget {
    budget_bytes: u64,
    in_flight: Mutex<u64>,
}

impl BoxStoreByteBudget {
    pub fn new(budget_bytes: u64) -> Self {
        Self {
            budget_bytes,
            in_flight: Mutex::new(0),
        }
    }

    pub fn try_acquire(&self, bytes: u64) -> bool {
        if bytes > self.budget_bytes {
            return false;
        }
        let mut in_flight = self.in_flight.lock().expect("byte budget poisoned");
        if in_flight.saturating_add(bytes) > self.budget_bytes {
            return false;
        }
        *in_flight += bytes;
        true
    }

    pub fn release(&self, bytes: u64) {
        let mut in_flight = self.in_flight.lock().expect("byte budget poisoned");
        *in_flight = in_flight.saturating_sub(bytes);
    }

    pub fn in_flight_bytes(&self) -> u64 {
        *self.in_flight.lock().expect("byte budget poisoned")
    }
}

pub fn claimed_destinations_are_unique<I>(paths: I) -> bool
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut seen = HashSet::new();
    paths.into_iter().all(|path| seen.insert(path))
}
