use std::collections::{BTreeMap, HashMap, HashSet};
use std::fs::{self, OpenOptions};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};
use std::sync::{Arc, Condvar, Mutex};

use sha2::{Digest, Sha256};

use super::box_object_store::BoxObjectStore;
use super::box_store_manifest_format::{BOX_STORE_BLOBS_PREFIX, BoxStoreManifestEntry};
use super::box_store_pack::{
    BOX_STORE_PACK_INDEX_KEY, BOX_STORE_PACKS_PREFIX, PACK_INDEX_MAX_BYTES, PackExtractionSink,
    PackMember, extract_pack_members, parse_pack_index,
};
use super::box_store_pack_pipeline::{
    BoxStoreBlobGroup, PACK_EXTRACT_CONCURRENCY, PACK_TMP_DIR_NAME, should_restore_from_pack,
};
use super::box_store_sync::{
    DEFAULT_COPY_IN_CONCURRENCY, DOWNLOAD_IN_FLIGHT_BYTE_BUDGET, LARGE_OBJECT_THRESHOLD_BYTES,
};
use crate::r#box::box_transfer::{for_each_bounded, for_each_wave_pipelined};

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
    if rel_path.is_empty() || rel_path.contains('\0') || Path::new(rel_path).is_absolute() {
        return None;
    }
    if rel_path
        .split('/')
        .any(|segment| segment.is_empty() || segment == "." || segment == "..")
    {
        return None;
    }
    let root = lexical_normalize(target_root);
    let destination = lexical_normalize(&root.join(rel_path));
    (destination != root && destination.starts_with(&root)).then_some(destination)
}

pub fn symlink_target_stays_within_root(
    target_root: &Path,
    symlink_path: &Path,
    target: &str,
) -> bool {
    if target.contains('\0') {
        return false;
    }
    let root = lexical_normalize(target_root);
    let target_path = Path::new(target);
    let candidate = if target_path.is_absolute() {
        lexical_normalize(target_path)
    } else {
        let Some(parent) = symlink_path.parent() else {
            return false;
        };
        lexical_normalize(&parent.join(target_path))
    };
    candidate == root || candidate.starts_with(&root)
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
    state: Mutex<u64>,
    wake: Condvar,
}

impl BoxStoreByteBudget {
    pub fn new(budget_bytes: u64) -> Self {
        Self {
            budget_bytes,
            state: Mutex::new(0),
            wake: Condvar::new(),
        }
    }

    pub fn try_acquire(&self, bytes: u64) -> bool {
        let mut in_flight = self.state.lock().expect("byte budget poisoned");
        if *in_flight > 0 && in_flight.saturating_add(bytes) > self.budget_bytes {
            return false;
        }
        *in_flight = in_flight.saturating_add(bytes);
        true
    }

    pub fn acquire(&self, bytes: u64) {
        if bytes == 0 {
            return;
        }
        let mut in_flight = self.state.lock().expect("byte budget poisoned");
        while *in_flight > 0 && in_flight.saturating_add(bytes) > self.budget_bytes {
            in_flight = self.wake.wait(in_flight).expect("byte budget wait poisoned");
        }
        *in_flight = in_flight.saturating_add(bytes);
    }

    pub fn release(&self, bytes: u64) {
        let mut in_flight = self.state.lock().expect("byte budget poisoned");
        *in_flight = in_flight.saturating_sub(bytes);
        self.wake.notify_all();
    }

    pub fn in_flight_bytes(&self) -> u64 {
        *self.state.lock().expect("byte budget poisoned")
    }
}

pub fn claimed_destinations_are_unique<I>(paths: I) -> bool
where
    I: IntoIterator<Item = PathBuf>,
{
    let mut seen = HashSet::new();
    paths.into_iter().all(|path| seen.insert(path))
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BoxStoreDownloadProgress {
    pub files: usize,
    pub bytes: u64,
    pub verified: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoxStoreDownloadTrace {
    pub event: String,
    pub manifest_entries: usize,
    pub file_entries: usize,
    pub symlink_entries: usize,
    pub symlinks_started: usize,
    pub symlinks_completed: usize,
    pub symlinks_in_flight: usize,
    pub active_symlink_steps: BTreeMap<String, usize>,
    pub symlink_ordinal: Option<usize>,
    pub symlink_step: Option<String>,
    pub symlink_outcome: Option<String>,
}

#[derive(Clone)]
pub struct BoxStoreDownloadOptions {
    pub download_concurrency: usize,
    pub large_object_threshold: u64,
    pub download_byte_budget: u64,
    pub owner: Option<(u32, u32)>,
    pub on_progress: Option<Arc<dyn Fn(BoxStoreDownloadProgress) + Send + Sync>>,
    pub on_trace: Option<Arc<dyn Fn(BoxStoreDownloadTrace) + Send + Sync>>,
    pub on_log: Option<Arc<dyn Fn(String) + Send + Sync>>,
}

impl Default for BoxStoreDownloadOptions {
    fn default() -> Self {
        Self {
            download_concurrency: DEFAULT_COPY_IN_CONCURRENCY,
            large_object_threshold: LARGE_OBJECT_THRESHOLD_BYTES,
            download_byte_budget: DOWNLOAD_IN_FLIGHT_BYTE_BUDGET,
            owner: None,
            on_progress: None,
            on_trace: None,
            on_log: None,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct BoxStoreDownloadSummary {
    pub manifest_entries: usize,
    pub files: usize,
    pub bytes: u64,
    pub verified: usize,
    pub failures: Vec<String>,
}

#[derive(Debug, Clone)]
struct PlannedDownload {
    manifest: BTreeMap<String, BoxStoreManifestEntry>,
    destinations: HashMap<String, PathBuf>,
    critical_small: Vec<BoxStoreBlobGroup>,
    critical_large: Vec<BoxStoreBlobGroup>,
    bulk_small: Vec<BoxStoreBlobGroup>,
    bulk_large: Vec<BoxStoreBlobGroup>,
    symlinks: Vec<(String, String)>,
    failures: Vec<String>,
}

fn file_identity(entry: &BoxStoreManifestEntry) -> Option<(&str, u64)> {
    match entry {
        BoxStoreManifestEntry::LegacyFile { sha, size }
        | BoxStoreManifestEntry::File { sha, size, .. } => Some((sha, *size)),
        BoxStoreManifestEntry::Symlink { .. } => None,
    }
}

fn plan_download(
    target_root: &Path,
    manifest: &BTreeMap<String, BoxStoreManifestEntry>,
    large_object_threshold: u64,
) -> PlannedDownload {
    let mut destinations = HashMap::new();
    let mut claimed = HashSet::new();
    let mut failures = Vec::new();
    let mut critical_small = HashMap::<String, BoxStoreBlobGroup>::new();
    let mut critical_large = HashMap::<String, BoxStoreBlobGroup>::new();
    let mut bulk_small = HashMap::<String, BoxStoreBlobGroup>::new();
    let mut bulk_large = HashMap::<String, BoxStoreBlobGroup>::new();
    let mut symlinks = Vec::new();

    for (rel_path, entry) in manifest {
        let Some(destination) = resolve_restore_destination(target_root, rel_path) else {
            failures.push(format!("{rel_path}: unsafe manifest path"));
            continue;
        };
        if !claimed.insert(destination.clone()) {
            failures.push(format!("{rel_path}: unsafe manifest path"));
            continue;
        }
        destinations.insert(rel_path.clone(), destination);
        match entry {
            BoxStoreManifestEntry::Symlink { target } => {
                symlinks.push((rel_path.clone(), target.clone()));
            }
            _ => {
                let (sha, size) = file_identity(entry).expect("file entry");
                let groups = if is_critical_rel_path(rel_path) {
                    if size >= large_object_threshold {
                        &mut critical_large
                    } else {
                        &mut critical_small
                    }
                } else if size >= large_object_threshold {
                    &mut bulk_large
                } else {
                    &mut bulk_small
                };
                let key = format!("{sha}:{size}");
                groups
                    .entry(key)
                    .and_modify(|group| group.rel_paths.push(rel_path.clone()))
                    .or_insert_with(|| BoxStoreBlobGroup {
                        sha: sha.to_string(),
                        size,
                        rel_paths: vec![rel_path.clone()],
                    });
            }
        }
    }

    PlannedDownload {
        manifest: manifest.clone(),
        destinations,
        critical_small: critical_small.into_values().collect(),
        critical_large: critical_large.into_values().collect(),
        bulk_small: bulk_small.into_values().collect(),
        bulk_large: bulk_large.into_values().collect(),
        symlinks,
        failures,
    }
}

fn ensure_owned_parent(target_root: &Path, destination: &Path, owner: Option<(u32, u32)>) -> Result<(), String> {
    let parent = destination
        .parent()
        .ok_or_else(|| "restore destination has no parent".to_string())?;
    let relative = parent
        .strip_prefix(target_root)
        .map_err(|_| "restore directory escapes target root".to_string())?;
    if fs::symlink_metadata(target_root)
        .map_err(|error| error.to_string())?
        .file_type()
        .is_symlink()
    {
        return Err("restore target root is a symlink".into());
    }
    let mut current = target_root.to_path_buf();
    for segment in relative.components() {
        current.push(segment.as_os_str());
        let mut created = false;
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.is_dir() => {}
            Ok(_) => {
                remove_existing_restore_path(&current)?;
                fs::create_dir(&current).map_err(|error| error.to_string())?;
                created = true;
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                fs::create_dir(&current).map_err(|error| error.to_string())?;
                created = true;
            }
            Err(error) => return Err(error.to_string()),
        }
        if created {
            apply_owner(&current, owner, false)?;
        }
    }
    Ok(())
}

fn apply_owner(path: &Path, owner: Option<(u32, u32)>, symlink: bool) -> Result<(), String> {
    let Some((uid, gid)) = owner else {
        return Ok(());
    };
    #[cfg(unix)]
    {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        if symlink {
            let raw = CString::new(path.as_os_str().as_bytes()).map_err(|error| error.to_string())?;
            let rc = unsafe { libc::lchown(raw.as_ptr(), uid, gid) };
            if rc != 0 {
                return Err(std::io::Error::last_os_error().to_string());
            }
            Ok(())
        } else {
            std::os::unix::fs::chown(path, Some(uid), Some(gid)).map_err(|error| error.to_string())
        }
    }
    #[cfg(not(unix))]
    {
        let _ = (path, uid, gid, symlink);
        Ok(())
    }
}

fn entry_mode(entry: &BoxStoreManifestEntry) -> Option<u32> {
    match entry {
        BoxStoreManifestEntry::File { mode, .. } => Some(*mode),
        _ => None,
    }
}

fn apply_file_metadata(
    path: &Path,
    entry: &BoxStoreManifestEntry,
    owner: Option<(u32, u32)>,
    legacy_replacement_mode: Option<u32>,
) -> Result<(), String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("restore destination is not a regular file".into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if let Some(mode) = entry_mode(entry).or(legacy_replacement_mode) {
            fs::set_permissions(path, fs::Permissions::from_mode(mode))
                .map_err(|error| error.to_string())?;
        }
    }
    apply_owner(path, owner, false)
}

fn existing_regular_file_mode(path: &Path) -> Option<u32> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::symlink_metadata(path)
            .ok()
            .filter(|metadata| metadata.is_file())
            .map(|metadata| metadata.permissions().mode() & 0o777)
    }
    #[cfg(not(unix))]
    {
        let _ = path;
        None
    }
}

fn unique_temp_path(destination: &Path) -> PathBuf {
    PathBuf::from(format!(
        "{}{}{}",
        destination.display(),
        BOX_STORE_RESTORE_TMP_SUFFIX,
        uuid::Uuid::new_v4().simple()
    ))
}

fn install_verified_temp(
    temp: &Path,
    destination: &Path,
    entry: &BoxStoreManifestEntry,
    owner: Option<(u32, u32)>,
) -> Result<(), String> {
    let legacy_mode = matches!(entry, BoxStoreManifestEntry::LegacyFile { .. })
        .then(|| existing_regular_file_mode(destination))
        .flatten();
    apply_file_metadata(temp, entry, None, legacy_mode)?;
    if fs::symlink_metadata(destination).is_ok_and(|metadata| metadata.is_dir()) {
        fs::remove_dir_all(destination).map_err(|error| error.to_string())?;
    }
    fs::rename(temp, destination).map_err(|error| error.to_string())?;
    apply_owner(destination, owner, false)
}

fn sha256_file(path: &Path) -> Result<String, String> {
    let metadata = fs::symlink_metadata(path).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("hash source is not a regular file".into());
    }
    let mut file = fs::File::open(path).map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = file.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        hash.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hash.finalize()))
}

fn local_file_matches(path: &Path, group: &BoxStoreBlobGroup) -> bool {
    fs::symlink_metadata(path).is_ok_and(|metadata| metadata.is_file() && metadata.len() == group.size)
        && sha256_file(path).ok().as_deref() == Some(group.sha.as_str())
}

fn copy_file_hashing(source: &Path, destination: &Path) -> Result<(String, u64), String> {
    let metadata = fs::symlink_metadata(source).map_err(|error| error.to_string())?;
    if !metadata.is_file() {
        return Err("copy source is not a regular file".into());
    }
    let mut reader = fs::File::open(source).map_err(|error| error.to_string())?;
    let mut writer = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(destination)
        .map_err(|error| error.to_string())?;
    let mut hash = Sha256::new();
    let mut size = 0_u64;
    let mut buffer = vec![0_u8; 1024 * 1024];
    loop {
        let read = reader.read(&mut buffer).map_err(|error| error.to_string())?;
        if read == 0 {
            break;
        }
        writer.write_all(&buffer[..read]).map_err(|error| error.to_string())?;
        hash.update(&buffer[..read]);
        size = size.saturating_add(read as u64);
    }
    writer.flush().map_err(|error| error.to_string())?;
    Ok((format!("{:x}", hash.finalize()), size))
}

fn has_disk_space_for_large_object(destination: &Path, object_bytes: u64) -> bool {
    let parent = destination.parent().unwrap_or(destination);
    fs2::available_space(parent)
        .map(|available| available >= object_bytes.saturating_mul(LARGE_OBJECT_FREE_SPACE_FACTOR))
        .unwrap_or(false)
}

fn report_progress(
    options: &BoxStoreDownloadOptions,
    state: &Mutex<BoxStoreDownloadSummary>,
    total: usize,
) {
    let Some(callback) = options.on_progress.as_ref() else {
        return;
    };
    let summary = state.lock().expect("download summary lock").clone();
    callback(BoxStoreDownloadProgress {
        files: summary.files,
        bytes: summary.bytes,
        verified: summary.verified,
        total,
    });
}

fn record_failure(state: &Mutex<BoxStoreDownloadSummary>, failure: String) {
    state
        .lock()
        .expect("download summary lock")
        .failures
        .push(failure);
}

fn restore_small_group(
    store: &dyn BoxObjectStore,
    target_root: &Path,
    planned: &PlannedDownload,
    group: &BoxStoreBlobGroup,
    pending: &[String],
    budget: &BoxStoreByteBudget,
    options: &BoxStoreDownloadOptions,
    state: &Mutex<BoxStoreDownloadSummary>,
) {
    budget.acquire(group.size);
    let result = store.get(&format!("{BOX_STORE_BLOBS_PREFIX}/{}", group.sha));
    match result {
        Ok(Some(blob)) if blob.len() as u64 == group.size
            && format!("{:x}", Sha256::digest(&blob)) == group.sha =>
        {
            for rel_path in pending {
                {
                    let mut summary = state.lock().expect("download summary lock");
                    summary.verified = summary.verified.saturating_add(1);
                }
                let Some(destination) = planned.destinations.get(rel_path) else {
                    record_failure(state, format!("{rel_path}: unsafe manifest path"));
                    continue;
                };
                let Some(entry) = planned.manifest.get(rel_path) else {
                    record_failure(state, format!("{rel_path}: missing manifest entry"));
                    continue;
                };
                let result = (|| -> Result<(), String> {
                    ensure_owned_parent(target_root, destination, options.owner)?;
                    let temp = unique_temp_path(destination);
                    let write = (|| -> Result<(), String> {
                        let mut file = OpenOptions::new()
                            .write(true)
                            .create_new(true)
                            .open(&temp)
                            .map_err(|error| error.to_string())?;
                        file.write_all(&blob).map_err(|error| error.to_string())?;
                        file.flush().map_err(|error| error.to_string())?;
                        install_verified_temp(&temp, destination, entry, options.owner)
                    })();
                    if write.is_err() {
                        let _ = fs::remove_file(&temp);
                    }
                    write
                })();
                match result {
                    Ok(()) => {
                        let mut summary = state.lock().expect("download summary lock");
                        summary.files = summary.files.saturating_add(1);
                        summary.bytes = summary.bytes.saturating_add(blob.len() as u64);
                        drop(summary);
                        report_progress(options, state, planned.manifest.len());
                    }
                    Err(error) => record_failure(state, format!("{rel_path}: {error}")),
                }
            }
        }
        Ok(Some(_)) => {
            for rel_path in pending {
                record_failure(state, format!("{rel_path}: sha/size mismatch"));
            }
        }
        Ok(None) => {
            for rel_path in pending {
                record_failure(state, format!("{rel_path}: blob {} missing", group.sha));
            }
        }
        Err(error) => {
            for rel_path in pending {
                record_failure(state, format!("{rel_path}: {error}"));
            }
        }
    }
    budget.release(group.size);
}

fn restore_large_group(
    store: &dyn BoxObjectStore,
    target_root: &Path,
    planned: &PlannedDownload,
    group: &BoxStoreBlobGroup,
    pending: &[String],
    options: &BoxStoreDownloadOptions,
    state: &Mutex<BoxStoreDownloadSummary>,
) {
    let mut restored_source: Option<PathBuf> = None;
    for rel_path in pending {
        let Some(destination) = planned.destinations.get(rel_path) else {
            record_failure(state, format!("{rel_path}: unsafe manifest path"));
            continue;
        };
        let Some(entry) = planned.manifest.get(rel_path) else {
            record_failure(state, format!("{rel_path}: missing manifest entry"));
            continue;
        };
        if let Err(error) = ensure_owned_parent(target_root, destination, options.owner) {
            record_failure(state, format!("{rel_path}: {error}"));
            continue;
        }
        if !has_disk_space_for_large_object(destination, group.size) {
            record_failure(
                state,
                format!(
                    "{rel_path}: insufficient disk space for {}B restore (x{} required)",
                    group.size, LARGE_OBJECT_FREE_SPACE_FACTOR
                ),
            );
            continue;
        }

        let temp = unique_temp_path(destination);
        let verified_temp = if let Some(source) = restored_source.as_ref() {
            match copy_file_hashing(source, &temp) {
                Ok((sha, size)) if sha == group.sha && size == group.size => Ok(()),
                Ok(_) => Err("sha/size mismatch".to_string()),
                Err(error) => Err(error),
            }
        } else {
            match store.get_to_file(
                &format!("{BOX_STORE_BLOBS_PREFIX}/{}", group.sha),
                &temp,
                Some(group.size),
            ) {
                Ok(Some(written)) if written == group.size => match sha256_file(&temp) {
                    Ok(sha) if sha == group.sha => Ok(()),
                    Ok(_) => Err("sha/size mismatch".to_string()),
                    Err(error) => Err(error),
                },
                Ok(Some(_)) => Err("sha/size mismatch".to_string()),
                Ok(None) => Err(format!("blob {} missing", group.sha)),
                Err(error) => Err(error),
            }
        };

        if let Err(error) = verified_temp {
            let _ = fs::remove_file(&temp);
            record_failure(state, format!("{rel_path}: {error}"));
            continue;
        }
        {
            let mut summary = state.lock().expect("download summary lock");
            summary.verified = summary.verified.saturating_add(1);
        }
        if let Err(error) = install_verified_temp(&temp, destination, entry, options.owner) {
            let _ = fs::remove_file(&temp);
            record_failure(state, format!("{rel_path}: {error}"));
            continue;
        }
        if restored_source.is_none() {
            restored_source = Some(destination.clone());
        }
        let mut summary = state.lock().expect("download summary lock");
        summary.files = summary.files.saturating_add(1);
        summary.bytes = summary.bytes.saturating_add(group.size);
        drop(summary);
        report_progress(options, state, planned.manifest.len());
    }
}

#[derive(Debug, Clone)]
struct PackRestoreGroup {
    size: u64,
    rel_paths: Vec<String>,
}

struct DownloadPackSink {
    target_root: PathBuf,
    manifest: BTreeMap<String, BoxStoreManifestEntry>,
    groups: HashMap<String, PackRestoreGroup>,
    restored: Mutex<HashSet<String>>,
    owner: Option<(u32, u32)>,
}

impl PackExtractionSink for DownloadPackSink {
    fn wants(&self, member: &PackMember) -> bool {
        let key = format!("{}:{}", member.sha, member.size);
        let Some(group) = self.groups.get(&key) else {
            return false;
        };
        let restored = self.restored.lock().expect("pack restored lock");
        group.rel_paths.iter().any(|rel_path| !restored.contains(rel_path))
    }

    fn on_blob(&self, member: &PackMember, bytes: Vec<u8>) -> Result<(), String> {
        let key = format!("{}:{}", member.sha, member.size);
        let Some(group) = self.groups.get(&key) else {
            return Ok(());
        };
        if bytes.len() as u64 != member.size
            || format!("{:x}", Sha256::digest(&bytes)) != member.sha
        {
            return Err("pack member sha/size mismatch".into());
        }
        for rel_path in &group.rel_paths {
            let reserved = {
                let mut restored = self.restored.lock().expect("pack restored lock");
                restored.insert(rel_path.clone())
            };
            if !reserved {
                continue;
            }
            let Some(destination) = resolve_restore_destination(&self.target_root, rel_path) else {
                self.restored
                    .lock()
                    .expect("pack restored lock")
                    .remove(rel_path);
                continue;
            };
            let Some(entry) = self.manifest.get(rel_path) else {
                self.restored
                    .lock()
                    .expect("pack restored lock")
                    .remove(rel_path);
                continue;
            };
            if local_file_matches(
                &destination,
                &BoxStoreBlobGroup {
                    sha: member.sha.clone(),
                    size: member.size,
                    rel_paths: Vec::new(),
                },
            ) {
                if apply_file_metadata(&destination, entry, self.owner, None).is_ok() {
                    continue;
                }
            }
            let result = (|| -> Result<(), String> {
                ensure_owned_parent(&self.target_root, &destination, self.owner)?;
                let temp = unique_temp_path(&destination);
                let write = (|| -> Result<(), String> {
                    let mut file = OpenOptions::new()
                        .write(true)
                        .create_new(true)
                        .open(&temp)
                        .map_err(|error| error.to_string())?;
                    file.write_all(&bytes).map_err(|error| error.to_string())?;
                    file.flush().map_err(|error| error.to_string())?;
                    install_verified_temp(&temp, &destination, entry, self.owner)
                })();
                if write.is_err() {
                    let _ = fs::remove_file(&temp);
                }
                write
            })();
            if result.is_err() {
                self.restored
                    .lock()
                    .expect("pack restored lock")
                    .remove(rel_path);
            }
        }
        Ok(())
    }
}

fn restore_bulk_small_from_packs(
    store: &dyn BoxObjectStore,
    target_root: &Path,
    planned: &PlannedDownload,
    owner: Option<(u32, u32)>,
) -> HashSet<String> {
    let mut groups = HashMap::<String, PackRestoreGroup>::new();
    for group in &planned.bulk_small {
        groups.insert(
            format!("{}:{}", group.sha, group.size),
            PackRestoreGroup {
                size: group.size,
                rel_paths: group.rel_paths.clone(),
            },
        );
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
    let sink = Arc::new(DownloadPackSink {
        target_root: target_root.to_path_buf(),
        manifest: planned.manifest.clone(),
        groups: groups.clone(),
        restored: Mutex::new(HashSet::new()),
        owner,
    });
    let temp_dir = std::env::temp_dir().join(PACK_TMP_DIR_NAME);
    if fs::create_dir_all(&temp_dir).is_err() {
        return HashSet::new();
    }
    for (ordinal, pack) in index.packs.iter().enumerate() {
        let restored = sink.restored.lock().expect("pack restored lock");
        let mut usable_count = 0usize;
        let mut usable_clen = 0u64;
        let mut usable_size = 0u64;
        for member in &pack.members {
            let key = format!("{}:{}", member.sha, member.size);
            let Some(group) = groups.get(&key) else {
                continue;
            };
            if group.rel_paths.iter().any(|path| !restored.contains(path)) {
                usable_count += 1;
                usable_clen = usable_clen.saturating_add(member.clen);
                usable_size = usable_size.saturating_add(member.size);
            }
        }
        drop(restored);
        if !should_restore_from_pack(usable_count, usable_clen, usable_size, pack.bytes) {
            continue;
        }
        let temp = temp_dir.join(format!(
            "restore-{}-{}-{ordinal}",
            std::process::id(),
            uuid::Uuid::new_v4().simple()
        ));
        let restored_pack = (|| -> Result<(), String> {
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
            let _ = extract_pack_members(&temp, &pack.members, PACK_EXTRACT_CONCURRENCY, sink.clone())?;
            Ok(())
        })();
        let _ = fs::remove_file(&temp);
        if restored_pack.is_err() {
            continue;
        }
    }
    let restored = sink.restored.lock().expect("pack restored lock").clone();
    restored
}

pub fn download_manifest(
    store: &dyn BoxObjectStore,
    target_root: &Path,
    manifest: &BTreeMap<String, BoxStoreManifestEntry>,
    options: BoxStoreDownloadOptions,
) -> BoxStoreDownloadSummary {
    if let Err(error) = fs::create_dir_all(target_root) {
        return BoxStoreDownloadSummary {
            manifest_entries: manifest.len(),
            failures: vec![format!("create restore target root: {error}")],
            ..BoxStoreDownloadSummary::default()
        };
    }
    let planned = plan_download(target_root, manifest, options.large_object_threshold);
    let state = Mutex::new(BoxStoreDownloadSummary {
        manifest_entries: manifest.len(),
        failures: planned.failures.clone(),
        ..BoxStoreDownloadSummary::default()
    });

    if let Some(callback) = options.on_trace.as_ref() {
        callback(BoxStoreDownloadTrace {
            event: "manifest-planned".into(),
            manifest_entries: manifest.len(),
            file_entries: manifest.values().filter(|entry| entry.is_file()).count(),
            symlink_entries: planned.symlinks.len(),
            ..BoxStoreDownloadTrace::default()
        });
    }

    let budget = BoxStoreByteBudget::new(options.download_byte_budget);
    let run_phase = |groups: &[BoxStoreBlobGroup], concurrency: usize, large: bool| {
        let _ = for_each_wave_pipelined(
            groups,
            COPY_IN_WAVE_SIZE,
            concurrency,
            |wave| {
                let mut prepared = Vec::new();
                for group in wave {
                    let mut pending = Vec::new();
                    for rel_path in &group.rel_paths {
                        let Some(destination) = planned.destinations.get(rel_path) else {
                            continue;
                        };
                        if local_file_matches(destination, group) {
                            if let Some(entry) = planned.manifest.get(rel_path) {
                                match apply_file_metadata(destination, entry, options.owner, None) {
                                    Ok(()) => {
                                        let mut summary = state.lock().expect("download summary lock");
                                        summary.verified += 1;
                                        summary.files += 1;
                                        summary.bytes = summary.bytes.saturating_add(group.size);
                                        drop(summary);
                                        report_progress(&options, &state, manifest.len());
                                    }
                                    Err(error) => {
                                        record_failure(&state, format!("{rel_path}: {error}"));
                                    }
                                }
                            }
                        } else {
                            pending.push(rel_path.clone());
                        }
                    }
                    if !pending.is_empty() {
                        prepared.push((group.clone(), pending));
                    }
                }
                if !prepared.is_empty() {
                    let keys = prepared
                        .iter()
                        .map(|(group, _)| format!("{BOX_STORE_BLOBS_PREFIX}/{}", group.sha))
                        .collect::<Vec<_>>();
                    let _ = store.prefetch_reads(&keys);
                }
                Ok::<_, ()>(prepared)
            },
            |(group, pending)| {
                if large {
                    restore_large_group(
                        store,
                        target_root,
                        &planned,
                        &group,
                        &pending,
                        &options,
                        &state,
                    );
                } else {
                    restore_small_group(
                        store,
                        target_root,
                        &planned,
                        &group,
                        &pending,
                        &budget,
                        &options,
                        &state,
                    );
                }
                Ok::<(), ()>(())
            },
        );
    };

    run_phase(&planned.critical_small, options.download_concurrency.max(1), false);
    run_phase(&planned.critical_large, COPY_IN_LARGE_BLOB_CONCURRENCY, true);

    let restored_by_pack = restore_bulk_small_from_packs(store, target_root, &planned, options.owner);
    if !restored_by_pack.is_empty() {
        let mut summary = state.lock().expect("download summary lock");
        for rel_path in &restored_by_pack {
            if let Some(entry) = planned.manifest.get(rel_path) {
                summary.verified += 1;
                summary.files += 1;
                summary.bytes = summary.bytes.saturating_add(file_identity(entry).map(|(_, size)| size).unwrap_or(0));
            }
        }
        drop(summary);
        report_progress(&options, &state, manifest.len());
    }
    let remaining_bulk_small = planned
        .bulk_small
        .iter()
        .filter_map(|group| {
            let rel_paths = group
                .rel_paths
                .iter()
                .filter(|path| !restored_by_pack.contains(*path))
                .cloned()
                .collect::<Vec<_>>();
            (!rel_paths.is_empty()).then(|| BoxStoreBlobGroup {
                sha: group.sha.clone(),
                size: group.size,
                rel_paths,
            })
        })
        .collect::<Vec<_>>();
    run_phase(&remaining_bulk_small, options.download_concurrency.max(1), false);
    run_phase(&planned.bulk_large, COPY_IN_LARGE_BLOB_CONCURRENCY, true);

    let trace_state = Mutex::new((
        0usize,
        0usize,
        BTreeMap::<String, usize>::new(),
    ));
    if !planned.symlinks.is_empty() {
        if let Some(callback) = options.on_trace.as_ref() {
            callback(BoxStoreDownloadTrace {
                event: "symlink-phase-started".into(),
                manifest_entries: manifest.len(),
                file_entries: manifest.values().filter(|entry| entry.is_file()).count(),
                symlink_entries: planned.symlinks.len(),
                ..BoxStoreDownloadTrace::default()
            });
        }
        let indexed = planned
            .symlinks
            .iter()
            .enumerate()
            .map(|(index, (path, target))| (index + 1, path.clone(), target.clone()))
            .collect::<Vec<_>>();
        let _ = for_each_bounded(&indexed, options.download_concurrency.max(1), |item| {
            let (ordinal, rel_path, target) = item;
            let Some(destination) = planned.destinations.get(rel_path) else {
                return Ok::<(), ()>(());
            };
            {
                let mut trace = trace_state.lock().expect("symlink trace lock");
                trace.0 += 1;
                trace.2.insert("prepare-parent".into(), 1);
                if let Some(callback) = options.on_trace.as_ref() {
                    callback(BoxStoreDownloadTrace {
                        event: "symlink-entry-started".into(),
                        manifest_entries: manifest.len(),
                        file_entries: manifest.values().filter(|entry| entry.is_file()).count(),
                        symlink_entries: planned.symlinks.len(),
                        symlinks_started: trace.0,
                        symlinks_completed: trace.1,
                        symlinks_in_flight: trace.0.saturating_sub(trace.1),
                        active_symlink_steps: trace.2.clone(),
                        symlink_ordinal: Some(*ordinal),
                        symlink_step: Some("prepare-parent".into()),
                        symlink_outcome: None,
                    });
                }
            }
            let restored = (|| -> Result<(), String> {
                ensure_owned_parent(target_root, destination, options.owner)?;
                if !symlink_target_stays_within_root(target_root, destination, target) {
                    return Err("unsafe symlink target".into());
                }
                if fs::read_link(destination).ok().as_deref() == Some(Path::new(target)) {
                    return apply_owner(destination, options.owner, true);
                }
                let temp = unique_temp_path(destination);
                #[cfg(unix)]
                {
                    std::os::unix::fs::symlink(target, &temp).map_err(|error| error.to_string())?;
                }
                #[cfg(not(unix))]
                {
                    return Err("symlink restore unsupported on this platform".into());
                }
                if fs::symlink_metadata(destination).is_ok_and(|metadata| metadata.is_dir()) {
                    fs::remove_dir_all(destination).map_err(|error| error.to_string())?;
                }
                let result = fs::rename(&temp, destination).map_err(|error| error.to_string());
                if result.is_err() {
                    let _ = fs::remove_file(&temp);
                    result?;
                }
                apply_owner(destination, options.owner, true)?;
                if fs::read_link(destination).ok().as_deref() != Some(Path::new(target)) {
                    return Err("restored symlink target does not match".into());
                }
                Ok(())
            })();
            {
                let mut trace = trace_state.lock().expect("symlink trace lock");
                trace.2.clear();
                if restored.is_ok() {
                    trace.1 += 1;
                }
                if let Some(callback) = options.on_trace.as_ref() {
                    callback(BoxStoreDownloadTrace {
                        event: "symlink-entry-finished".into(),
                        manifest_entries: manifest.len(),
                        file_entries: manifest.values().filter(|entry| entry.is_file()).count(),
                        symlink_entries: planned.symlinks.len(),
                        symlinks_started: trace.0,
                        symlinks_completed: trace.1,
                        symlinks_in_flight: trace.0.saturating_sub(trace.1),
                        active_symlink_steps: trace.2.clone(),
                        symlink_ordinal: Some(*ordinal),
                        symlink_step: None,
                        symlink_outcome: Some(if restored.is_ok() { "restored" } else { "failed" }.into()),
                    });
                }
            }
            match restored {
                Ok(()) => {
                    let mut summary = state.lock().expect("download summary lock");
                    summary.verified += 1;
                    summary.files += 1;
                    drop(summary);
                    report_progress(&options, &state, manifest.len());
                }
                Err(error) => record_failure(&state, format!("{rel_path}: {error}")),
            }
            Ok::<(), ()>(())
        });
        if let Some(callback) = options.on_trace.as_ref() {
            let trace = trace_state.lock().expect("symlink trace lock");
            callback(BoxStoreDownloadTrace {
                event: "symlink-phase-finished".into(),
                manifest_entries: manifest.len(),
                file_entries: manifest.values().filter(|entry| entry.is_file()).count(),
                symlink_entries: planned.symlinks.len(),
                symlinks_started: trace.0,
                symlinks_completed: trace.1,
                symlinks_in_flight: trace.0.saturating_sub(trace.1),
                active_symlink_steps: trace.2.clone(),
                ..BoxStoreDownloadTrace::default()
            });
        }
    }

    state.into_inner().expect("download summary mutex")
}

pub fn remove_existing_restore_path(path: &Path) -> Result<(), String> {
    match fs::symlink_metadata(path) {
        Ok(metadata) if metadata.is_dir() => fs::remove_dir_all(path).map_err(|error| error.to_string()),
        Ok(_) => fs::remove_file(path).map_err(|error| error.to_string()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.to_string()),
    }
}
