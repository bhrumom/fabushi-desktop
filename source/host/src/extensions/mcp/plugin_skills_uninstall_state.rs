//! Private durable image for the existing Plugin Skills owner's admission custody.
//! The caller holds its publication lock; this is not a second writer or a
//! cross-process compare-and-swap. Loading never creates or repairs files.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::marker::PhantomData;
use std::path::{Path, PathBuf};

use serde::de::{MapAccess, Visitor, value::MapAccessDeserializer};
use serde::{Deserialize, Deserializer, Serialize};
use uuid::Uuid;

use super::super::mcp_service::is_canonical_uninstall_plugin_id;
use super::super::plugin_skills_cache::PluginAuthBlock;
use super::is_canonical_plugin_id;

const IMAGE_FILE: &str = "uninstall-state.json";
const MARKER_FILE: &str = "uninstall-state.version";
const MARKER_BYTES: &[u8] = b"1\n";
const INVALID_STATE: &str = "plugin uninstall state is unavailable";
const CHANGED_STATE: &str = "plugin uninstall state changed";
const WRITE_FAILED: &str = "could not persist plugin uninstall state";
const PUBLICATION_UNKNOWN: &str = "plugin uninstall state publication outcome is unknown";
const ADMISSION_LOCK_FILE: &str = ".plugin-skills-admission.lock";
const REFRESH_LOCK_FILE: &str = ".plugin-skills-refresh.lock";
const LOCK_UNAVAILABLE: &str = "plugin skills admission lock is unavailable";
const REFRESH_LOCK_UNAVAILABLE: &str = "plugin skills refresh lock is unavailable";

/// Cross-process lock files live beside the existing Plugin Skills state.
/// The admission lock serializes durable state and consumers; the separate
/// refresh lock is a recoverable process lease held only by an active refresh.
pub(super) struct AdmissionFileLock {
    _file: File,
}

impl AdmissionFileLock {
    pub(super) fn shared(sand_root: &Path) -> Result<Self, String> {
        let file = open_lock_file(sand_root, ADMISSION_LOCK_FILE)?;
        fs2::FileExt::lock_shared(&file).map_err(|_| LOCK_UNAVAILABLE.to_string())?;
        Ok(Self { _file: file })
    }

    /// Observation must not wait behind a paused publisher. A missing lease
    /// permits only negative information from an unlocked image read.
    pub(super) fn try_shared(sand_root: &Path) -> Result<Option<Self>, String> {
        let file = open_lock_file(sand_root, ADMISSION_LOCK_FILE)?;
        match fs2::FileExt::try_lock_shared(&file) {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(error) if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() => Ok(None),
            Err(_) => Err(LOCK_UNAVAILABLE.into()),
        }
    }

    pub(super) fn exclusive(sand_root: &Path) -> Result<Self, String> {
        ensure_directory_durable(sand_root)?;
        let file = open_lock_file(sand_root, ADMISSION_LOCK_FILE)?;
        fs2::FileExt::lock_exclusive(&file).map_err(|_| LOCK_UNAVAILABLE.to_string())?;
        Ok(Self { _file: file })
    }

    pub(super) fn try_refresh(sand_root: &Path) -> Result<Option<Self>, String> {
        ensure_directory_durable(sand_root)?;
        let file = open_lock_file(sand_root, REFRESH_LOCK_FILE)
            .map_err(|_| REFRESH_LOCK_UNAVAILABLE.to_string())?;
        match fs2::FileExt::try_lock_exclusive(&file) {
            Ok(()) => Ok(Some(Self { _file: file })),
            Err(error) if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() => Ok(None),
            Err(_) => Err(REFRESH_LOCK_UNAVAILABLE.into()),
        }
    }
}

fn open_lock_file(sand_root: &Path, name: &str) -> Result<File, String> {
    if !sand_root.is_dir() {
        return Err(LOCK_UNAVAILABLE.into());
    }
    let path = sand_root.join(name);
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600).custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let file = options.open(&path).map_err(|_| LOCK_UNAVAILABLE.to_string())?;
    let path_metadata = fs::symlink_metadata(&path).map_err(|_| LOCK_UNAVAILABLE.to_string())?;
    let opened_metadata = file.metadata().map_err(|_| LOCK_UNAVAILABLE.to_string())?;
    if !path_metadata.file_type().is_file() || !opened_metadata.file_type().is_file() {
        return Err(LOCK_UNAVAILABLE.into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if path_metadata.dev() != opened_metadata.dev()
            || path_metadata.ino() != opened_metadata.ino()
        {
            return Err(LOCK_UNAVAILABLE.into());
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if path_metadata.file_attributes() & 0x0400 != 0
            || opened_metadata.file_attributes() & 0x0400 != 0
        {
            return Err(LOCK_UNAVAILABLE.into());
        }
    }
    Ok(file)
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct UninstallState {
    pub confirmed: BTreeSet<String>,
    pub pending: BTreeMap<String, String>,
    pub auth_refusals: BTreeMap<String, PluginAuthBlock>,
    pub refresh_intent: Option<String>,
    pub uninstall_observation: Option<String>,
}

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct DurableUninstallImage {
    pub state: UninstallState,
    // Keep absence and the exact accepted bytes, including JSON formatting.
    // Equal decoded states alone cannot authorize replacing an observed image.
    image_bytes: Option<Vec<u8>>,
    marker_bytes: Option<Vec<u8>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredUninstallImage {
    format_version: u32,
    confirmed_plugin_ids: Vec<String>,
    pending: Vec<JsonObject<StoredPendingUninstall>>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredAdmissionImage {
    format_version: u32,
    confirmed_plugin_ids: Vec<String>,
    pending: Vec<JsonObject<StoredPendingUninstall>>,
    auth_refusals: Vec<JsonObject<StoredAuthRefusal>>,
    refresh_intent: StoredRefreshIntent,
    uninstall_observation: StoredUninstallObservation,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredAdmissionImageV2 {
    format_version: u32,
    confirmed_plugin_ids: Vec<String>,
    pending: Vec<JsonObject<StoredPendingUninstall>>,
    auth_refusals: Vec<JsonObject<StoredAuthRefusal>>,
    refresh_intent: StoredRefreshIntent,
}

#[derive(Deserialize)]
#[serde(untagged)]
enum VersionedImage {
    Current(JsonObject<StoredAdmissionImage>),
    AuthRecovery(JsonObject<StoredAdmissionImageV2>),
    Legacy(JsonObject<StoredUninstallImage>),
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredAuthRefusal {
    plugin_id: String,
    plugin_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    marketplace_name: Option<String>,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredRefreshOperation {
    operation_id: String,
}

// A required nullable field cannot be Option<T>: serde would accept a missing
// field as None. deserialize_any rejects a missing field while accepting JSON
// null or exactly the declared operation object.
#[derive(Serialize)]
#[serde(untagged)]
enum StoredRefreshIntent {
    Empty(()),
    Active(JsonObject<StoredRefreshOperation>),
}

impl<'de> Deserialize<'de> for StoredRefreshIntent {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct IntentVisitor;
        impl<'de> Visitor<'de> for IntentVisitor {
            type Value = StoredRefreshIntent;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("null or a refresh operation object")
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(StoredRefreshIntent::Empty(()))
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                StoredRefreshOperation::deserialize(MapAccessDeserializer::new(map))
                    .map(|operation| StoredRefreshIntent::Active(JsonObject(operation)))
            }
        }
        deserializer.deserialize_any(IntentVisitor)
    }
}

// This nullable field is required in v3. Deserialize it explicitly so missing
// fields, arrays, objects and wrong scalar types cannot become a legacy None.
#[derive(Serialize)]
#[serde(transparent)]
struct StoredUninstallObservation(Option<String>);

impl<'de> Deserialize<'de> for StoredUninstallObservation {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObservationVisitor;
        impl<'de> Visitor<'de> for ObservationVisitor {
            type Value = StoredUninstallObservation;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("null or a canonical uninstall observation UUID string")
            }

            fn visit_unit<E: serde::de::Error>(self) -> Result<Self::Value, E> {
                Ok(StoredUninstallObservation(None))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                Ok(StoredUninstallObservation(Some(value.to_string())))
            }
        }
        deserializer.deserialize_any(ObservationVisitor)
    }
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct StoredPendingUninstall {
    plugin_id: String,
    operation_id: String,
}

// A derived struct deserializer also accepts a positional sequence. These
// durable DTOs require JSON objects, including each pending entry, while the
// derived field visitors retain duplicate-field and unknown-field rejection.
#[derive(Serialize)]
#[serde(transparent)]
struct JsonObject<T>(T);

impl<'de, T: Deserialize<'de>> Deserialize<'de> for JsonObject<T> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct ObjectVisitor<T>(PhantomData<T>);

        impl<'de, T: Deserialize<'de>> Visitor<'de> for ObjectVisitor<T> {
            type Value = JsonObject<T>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("an object")
            }

            fn visit_map<A: MapAccess<'de>>(self, map: A) -> Result<Self::Value, A::Error> {
                T::deserialize(MapAccessDeserializer::new(map)).map(JsonObject)
            }
        }

        deserializer.deserialize_map(ObjectVisitor(PhantomData))
    }
}

fn decode_state(bytes: &[u8]) -> Result<UninstallState, String> {
    let (confirmed, pending, auth_refusals, refresh_intent, uninstall_observation) = match
        serde_json::from_slice::<VersionedImage>(bytes).map_err(|_| INVALID_STATE.to_string())?
    {
        VersionedImage::Legacy(JsonObject(stored)) if stored.format_version == 1 => {
            (stored.confirmed_plugin_ids, stored.pending, Vec::new(), None, None)
        }
        VersionedImage::AuthRecovery(JsonObject(stored)) if stored.format_version == 2 => {
            let refresh = match stored.refresh_intent {
                StoredRefreshIntent::Empty(()) => None,
                StoredRefreshIntent::Active(JsonObject(operation)) => Some(operation.operation_id),
            };
            (stored.confirmed_plugin_ids, stored.pending, stored.auth_refusals, refresh, None)
        }
        VersionedImage::Current(JsonObject(stored)) if stored.format_version == 3 => {
            let refresh = match stored.refresh_intent {
                StoredRefreshIntent::Empty(()) => None,
                StoredRefreshIntent::Active(JsonObject(operation)) => Some(operation.operation_id),
            };
            (stored.confirmed_plugin_ids, stored.pending, stored.auth_refusals,
                refresh, stored.uninstall_observation.0)
        }
        _ => return Err(INVALID_STATE.into()),
    };
    let mut state = UninstallState::default();
    for id in confirmed {
        if !state.confirmed.insert(id) {
            return Err(INVALID_STATE.into());
        }
    }
    for JsonObject(pending) in pending {
        if state
            .pending
            .insert(pending.plugin_id, pending.operation_id)
            .is_some()
        {
            return Err(INVALID_STATE.into());
        }
    }
    for JsonObject(block) in auth_refusals {
        let block = PluginAuthBlock {
            plugin_id: block.plugin_id,
            plugin_name: block.plugin_name,
            marketplace_name: block.marketplace_name,
        };
        if state.auth_refusals.insert(block.plugin_id.clone(), block).is_some() {
            return Err(INVALID_STATE.into());
        }
    }
    state.refresh_intent = refresh_intent;
    state.uninstall_observation = uninstall_observation;
    validate_state(&state)?;
    Ok(state)
}

fn validate_state(state: &UninstallState) -> Result<(), String> {
    if state.confirmed.iter().any(|id| !is_canonical_uninstall_plugin_id(id)) {
        return Err(INVALID_STATE.into());
    }
    let mut operation_ids = BTreeSet::new();
    for (id, operation_id) in &state.pending {
        let canonical_operation = Uuid::parse_str(operation_id)
            .is_ok_and(|parsed| parsed.to_string() == *operation_id);
        if !is_canonical_uninstall_plugin_id(id)
            || !canonical_operation
            || !operation_ids.insert(operation_id)
        {
            return Err(INVALID_STATE.into());
        }
    }
    if state.auth_refusals.iter().any(|(id, block)| {
        id != &block.plugin_id || !is_canonical_plugin_id(id)
    }) {
        return Err(INVALID_STATE.into());
    }
    if let Some(operation_id) = &state.refresh_intent {
        if !Uuid::parse_str(operation_id).is_ok_and(|parsed| parsed.to_string() == *operation_id)
            || operation_ids.contains(operation_id)
        {
            return Err(INVALID_STATE.into());
        }
    }
    if let Some(observation) = &state.uninstall_observation {
        if !Uuid::parse_str(observation).is_ok_and(|parsed| parsed.to_string() == *observation) {
            return Err(INVALID_STATE.into());
        }
    }
    Ok(())
}

fn encode_state(state: &UninstallState) -> Result<Vec<u8>, String> {
    validate_state(state)?;
    let stored = StoredAdmissionImage {
        format_version: 3,
        confirmed_plugin_ids: state.confirmed.iter().cloned().collect(),
        pending: state
            .pending
            .iter()
            .map(|(plugin_id, operation_id)| {
                JsonObject(StoredPendingUninstall {
                    plugin_id: plugin_id.clone(),
                    operation_id: operation_id.clone(),
                })
            })
            .collect(),
        auth_refusals: state.auth_refusals.values().map(|block| {
            JsonObject(StoredAuthRefusal {
                plugin_id: block.plugin_id.clone(),
                plugin_name: block.plugin_name.clone(),
                marketplace_name: block.marketplace_name.clone(),
            })
        }).collect(),
        refresh_intent: match &state.refresh_intent {
            Some(operation_id) => StoredRefreshIntent::Active(JsonObject(StoredRefreshOperation {
                operation_id: operation_id.clone(),
            })),
            None => StoredRefreshIntent::Empty(()),
        },
        uninstall_observation: StoredUninstallObservation(state.uninstall_observation.clone()),
    };
    serde_json::to_vec_pretty(&stored).map_err(|_| WRITE_FAILED.to_string())
}

/// Both absent means legacy state. An image without its marker is authoritative
/// interrupted initialization; a marker without its image is never empty state.
pub(super) fn load_uninstall_image(cache_dir: &Path) -> Result<DurableUninstallImage, String> {
    let image_bytes = read_optional_file(&cache_dir.join(IMAGE_FILE))?;
    let marker_bytes = read_optional_file(&cache_dir.join(MARKER_FILE))?;
    if marker_bytes.as_deref().is_some_and(|bytes| bytes != MARKER_BYTES) {
        return Err(INVALID_STATE.into());
    }
    let state = match &image_bytes {
        Some(bytes) => decode_state(bytes)?,
        None if marker_bytes.is_none() => UninstallState::default(),
        None => return Err(INVALID_STATE.into()),
    };
    Ok(DurableUninstallImage { state, image_bytes, marker_bytes })
}

/// Every error is a refusal, including an error after rename. The owner must
/// retain a live recovery fault and must not admit the external uninstall on an
/// uncertain reservation. In particular, no failure rolls a visible image back.
pub(super) fn persist_uninstall_image(
    cache_dir: &Path,
    expected: &DurableUninstallImage,
    next: &UninstallState,
) -> Result<DurableUninstallImage, String> {
    // Reject malformed candidates before creating even the writer's lock root.
    validate_state(next)?;
    let sand_root = cache_dir.parent().ok_or_else(|| LOCK_UNAVAILABLE.to_string())?;
    let _lock = AdmissionFileLock::exclusive(sand_root)?;
    persist_uninstall_image_locked(cache_dir, expected, next)
}

pub(super) fn persist_uninstall_image_locked(
    cache_dir: &Path,
    expected: &DurableUninstallImage,
    next: &UninstallState,
) -> Result<DurableUninstallImage, String> {
    require_unchanged(cache_dir, expected)?;
    if expected.state.uninstall_observation.is_some() && next.uninstall_observation.is_none() {
        return Err(CHANGED_STATE.into());
    }
    let image_bytes = encode_state(next)?;
    ensure_directory_durable(cache_dir)?;
    let image_path = cache_dir.join(IMAGE_FILE);
    let staged_image = StagedUninstallFile::stage(&image_path, &image_bytes)?;
    // Staging is fallible I/O. Refuse newly observed drift before replacing the
    // current file, and let the owned staging guard clean up only its own file.
    require_unchanged(cache_dir, expected)?;
    staged_image.publish(&image_path)?;

    let mut published = DurableUninstallImage {
        state: next.clone(),
        image_bytes: Some(image_bytes),
        marker_bytes: expected.marker_bytes.clone(),
    };
    require_unchanged(cache_dir, &published)?;
    if published.marker_bytes.is_none() {
        // The full image and its directory entry are durable before a marker
        // can claim initialization. Never create a marker during a mere load.
        let marker_path = cache_dir.join(MARKER_FILE);
        let staged_marker = StagedUninstallFile::stage(&marker_path, MARKER_BYTES)?;
        require_unchanged(cache_dir, &published)?;
        staged_marker.publish(&marker_path)?;
        published.marker_bytes = Some(MARKER_BYTES.to_vec());
    }
    require_unchanged(cache_dir, &published)?;
    Ok(published)
}

/// A failed auth transition may retain the expected bytes or expose exactly
/// its candidate before a later barrier fails. Neither observation proves
/// durability, but both distinguish known negative admission from foreign or
/// unreadable state. The caller retains adverse live facts on every error.
pub(super) fn inspect_failed_auth_publication(
    cache_dir: &Path,
    expected: &DurableUninstallImage,
    next: &UninstallState,
) -> Result<DurableUninstallImage, String> {
    let sand_root = cache_dir.parent().ok_or_else(|| LOCK_UNAVAILABLE.to_string())?;
    let _lock = AdmissionFileLock::shared(sand_root)?;
    inspect_failed_auth_publication_locked(cache_dir, expected, next)
}

pub(super) fn inspect_failed_auth_publication_locked(
    cache_dir: &Path,
    expected: &DurableUninstallImage,
    next: &UninstallState,
) -> Result<DurableUninstallImage, String> {
    let observed = load_uninstall_image(cache_dir)?;
    if observed == *expected {
        return Ok(observed);
    }
    let mut candidate = DurableUninstallImage {
        state: next.clone(),
        image_bytes: Some(encode_state(next)?),
        marker_bytes: expected.marker_bytes.clone(),
    };
    if observed == candidate {
        return Ok(observed);
    }
    if candidate.marker_bytes.is_none() {
        candidate.marker_bytes = Some(MARKER_BYTES.to_vec());
        if observed == candidate {
            return Ok(observed);
        }
    }
    Err(CHANGED_STATE.into())
}

fn require_unchanged(cache_dir: &Path, expected: &DurableUninstallImage) -> Result<(), String> {
    if load_uninstall_image(cache_dir)? != *expected {
        return Err(CHANGED_STATE.into());
    }
    Ok(())
}

fn read_optional_file(path: &Path) -> Result<Option<Vec<u8>>, String> {
    let metadata = match fs::symlink_metadata(path) {
        Ok(metadata) => metadata,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(None),
        Err(_) => return Err(INVALID_STATE.into()),
    };
    // A dangling link is present unknown data, not an absent legacy image.
    // Reject nonregular entries before opening, including FIFOs that can block.
    if !metadata.file_type().is_file() {
        return Err(INVALID_STATE.into());
    }
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::OpenOptionsExt;
        options.custom_flags(0x0020_0000); // FILE_FLAG_OPEN_REPARSE_POINT
    }
    let mut file = options.open(path).map_err(|_| INVALID_STATE.to_string())?;
    let opened = file.metadata().map_err(|_| INVALID_STATE.to_string())?;
    if !opened.file_type().is_file() {
        return Err(INVALID_STATE.into());
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        if metadata.dev() != opened.dev() || metadata.ino() != opened.ino() {
            return Err(INVALID_STATE.into());
        }
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::MetadataExt;
        if metadata.file_attributes() & 0x0400 != 0 || opened.file_attributes() & 0x0400 != 0 {
            return Err(INVALID_STATE.into());
        }
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes).map_err(|_| INVALID_STATE.to_string())?;
    Ok(Some(bytes))
}

fn directory_parent(path: &Path) -> Result<&Path, String> {
    path.parent()
        .map(|parent| if parent.as_os_str().is_empty() { Path::new(".") } else { parent })
        .ok_or_else(|| WRITE_FAILED.to_string())
}

fn ensure_directory_durable(path: &Path) -> Result<(), String> {
    match fs::metadata(path) {
        Ok(metadata) if metadata.is_dir() => return sync_directory_entry(path),
        Ok(_) => return Err(WRITE_FAILED.into()),
        Err(error) if error.kind() == io::ErrorKind::NotFound => {}
        Err(_) => return Err(WRITE_FAILED.into()),
    }
    let parent = directory_parent(path)?;
    ensure_directory_durable(parent)?;
    fs::create_dir(path).map_err(|_| WRITE_FAILED.to_string())?;
    sync_directory_entry(path)
}

fn sync_directory_entry(path: &Path) -> Result<(), String> {
    // Sync each newly created entry in its parent, including plugin-skills
    // itself. Syncing only files inside that directory does not cover its name.
    // Also cover an existing directory left visible by interrupted preparation.
    sync_directory(path).map_err(|_| PUBLICATION_UNKNOWN.to_string())?;
    if path.parent().is_some() {
        sync_directory(directory_parent(path)?).map_err(|_| PUBLICATION_UNKNOWN.to_string())?;
    }
    Ok(())
}

struct StagedUninstallFile {
    path: Option<PathBuf>,
    file: Option<File>,
}

impl StagedUninstallFile {
    fn stage(destination: &Path, bytes: &[u8]) -> Result<Self, String> {
        let mut name = destination
            .file_name()
            .ok_or_else(|| WRITE_FAILED.to_string())?
            .to_os_string();
        name.push(format!(".{}.tmp", Uuid::new_v4().simple()));
        Self::create(destination.with_file_name(name), bytes)
    }

    fn create(path: PathBuf, bytes: &[u8]) -> Result<Self, String> {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let file = options.open(&path).map_err(|_| WRITE_FAILED.to_string())?;
        // Ownership begins only after create_new succeeds. A colliding foreign
        // file must not be adopted by the cleanup guard.
        let mut staged = Self { path: Some(path), file: Some(file) };
        let file = staged.file.as_mut().expect("staged uninstall file is open");
        file.write_all(bytes).map_err(|_| WRITE_FAILED.to_string())?;
        file.sync_all().map_err(|_| WRITE_FAILED.to_string())?;
        drop(staged.file.take());
        Ok(staged)
    }

    fn publish(mut self, destination: &Path) -> Result<(), String> {
        #[cfg(test)]
        let injected = image_publish_failure(destination);
        #[cfg(test)]
        if matches!(injected, Some(ImagePublishFailure::BeforeRename)) {
            return Err(WRITE_FAILED.into());
        }
        let path = self.path.as_ref().expect("staged uninstall file is owned");
        fs::rename(path, destination).map_err(|_| WRITE_FAILED.to_string())?;
        // A later barrier/readback failure must not erase the published image.
        // Nor may Drop remove another file later created at the staging name.
        self.path = None;
        #[cfg(test)]
        if matches!(injected, Some(ImagePublishFailure::AfterRename)) {
            return Err(PUBLICATION_UNKNOWN.into());
        }
        sync_directory(directory_parent(destination)?)
            .map_err(|_| PUBLICATION_UNKNOWN.to_string())
    }
}

impl Drop for StagedUninstallFile {
    fn drop(&mut self) {
        drop(self.file.take());
        if let Some(path) = self.path.take() {
            let _ = fs::remove_file(path);
        }
    }
}

#[cfg(unix)]
fn sync_directory(path: &Path) -> io::Result<()> {
    File::open(path)?.sync_all()
}

#[cfg(not(unix))]
fn sync_directory(_path: &Path) -> io::Result<()> {
    // Keep the Host stores' established std-only non-Unix limit. File sync and
    // atomic rename do not establish a portable directory durability barrier.
    Ok(())
}

// Narrow I/O fault instrumentation: only this thread's nth image publication,
// never its marker or another test's operation. Shipping builds have no hook.
#[cfg(test)]
#[derive(Clone, Copy)]
pub(super) enum ImagePublishFailure {
    BeforeRename,
    AfterRename,
}

#[cfg(test)]
thread_local! {
    static IMAGE_PUBLISH_FAILURE: std::cell::RefCell<Option<(usize, ImagePublishFailure)>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(test)]
pub(super) fn with_image_publish_failure<T>(
    nth: usize,
    phase: ImagePublishFailure,
    operation: impl FnOnce() -> T,
) -> T {
    assert!(nth > 0, "publication ordinal must be positive");
    struct Restore(Option<(usize, ImagePublishFailure)>);
    impl Drop for Restore {
        fn drop(&mut self) {
            IMAGE_PUBLISH_FAILURE.with(|slot| *slot.borrow_mut() = self.0.take());
        }
    }
    let _restore = Restore(IMAGE_PUBLISH_FAILURE.with(|slot| slot.replace(Some((nth, phase)))));
    operation()
}

#[cfg(test)]
fn image_publish_failure(destination: &Path) -> Option<ImagePublishFailure> {
    if destination.file_name() != Some(std::ffi::OsStr::new(IMAGE_FILE)) {
        return None;
    }
    IMAGE_PUBLISH_FAILURE.with(|slot| {
        let mut pending = slot.borrow_mut();
        let (remaining, phase) = pending.as_mut()?;
        if *remaining > 1 {
            *remaining -= 1;
            return None;
        }
        let phase = *phase;
        *pending = None;
        Some(phase)
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const OP_A: &str = "11111111-1111-4111-8111-111111111111";
    const OP_B: &str = "22222222-2222-4222-8222-222222222222";
    const OP_C: &str = "33333333-3333-4333-8333-333333333333";

    struct TempRoot(PathBuf);

    impl TempRoot {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("plugin-uninstall-state-{}", Uuid::new_v4()));
            fs::create_dir(&path).expect("create owned uninstall-state fixture");
            Self(path)
        }

        fn cache(&self) -> PathBuf {
            self.0.join("plugin-skills")
        }

        fn seed_image(&self, bytes: &[u8]) -> PathBuf {
            let cache = self.cache();
            fs::create_dir_all(&cache).unwrap();
            fs::write(cache.join(IMAGE_FILE), bytes).unwrap();
            cache
        }
    }

    impl Drop for TempRoot {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn pending_state() -> UninstallState {
        UninstallState {
            confirmed: BTreeSet::from(["9".into()]),
            pending: BTreeMap::from([("9".into(), OP_A.into())]),
            ..UninstallState::default()
        }
    }

    #[test]
    fn absent_legacy_load_has_no_filesystem_side_effects() {
        let root = TempRoot::new();
        let cache = root.cache();
        let image = load_uninstall_image(&cache).unwrap();
        assert!(image == DurableUninstallImage::default());
        assert!(!cache.exists());
        assert_eq!(fs::read_dir(&root.0).unwrap().count(), 0);
    }

    #[test]
    fn first_transition_and_empty_terminal_state_round_trip() {
        let root = TempRoot::new();
        let cache = root.cache();
        let legacy = load_uninstall_image(&cache).unwrap();
        let next = pending_state();
        let pending = persist_uninstall_image(&cache, &legacy, &next).unwrap();
        assert!(pending.state == next);
        assert!(load_uninstall_image(&cache).unwrap() == pending);
        assert_eq!(fs::read(cache.join(MARKER_FILE)).unwrap(), MARKER_BYTES);
        assert_eq!(fs::read_dir(&cache).unwrap().count(), 2);
        let terminal = persist_uninstall_image(&cache, &pending, &UninstallState::default()).unwrap();
        assert!(terminal.state == UninstallState::default());
        assert!(terminal != DurableUninstallImage::default());
        assert!(load_uninstall_image(&cache).unwrap() == terminal);
        assert!(cache.join(IMAGE_FILE).is_file());
        assert!(cache.join(MARKER_FILE).is_file());
    }

    #[test]
    fn first_transition_creates_the_missing_directory_chain() {
        let root = TempRoot::new();
        let cache = root.0.join("new-sand-root").join("plugin-skills");
        let legacy = load_uninstall_image(&cache).unwrap();
        assert!(!cache.parent().unwrap().exists());
        let committed = persist_uninstall_image(&cache, &legacy, &pending_state()).unwrap();
        assert!(load_uninstall_image(&cache).unwrap() == committed);
        assert_eq!(fs::read_dir(cache).unwrap().count(), 2);
    }

    #[test]
    fn image_only_initialization_remains_authoritative_until_explicit_transition() {
        let root = TempRoot::new();
        let state = pending_state();
        let bytes = encode_state(&state).unwrap();
        let cache = root.seed_image(&bytes);
        let loaded = load_uninstall_image(&cache).unwrap();
        assert!(loaded.state == state);
        assert!(loaded.marker_bytes.is_none());
        assert!(!cache.join(MARKER_FILE).exists());
        assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes);
        let mut next = state;
        next.pending.clear();
        let transitioned = persist_uninstall_image(&cache, &loaded, &next).unwrap();
        assert!(transitioned.state == next);
        assert_eq!(fs::read(cache.join(MARKER_FILE)).unwrap(), MARKER_BYTES);
    }

    #[test]
    fn marker_without_image_and_invalid_markers_are_never_legacy() {
        let root = TempRoot::new();
        let cache = root.cache();
        fs::create_dir(&cache).unwrap();
        let marker = cache.join(MARKER_FILE);
        fs::write(&marker, MARKER_BYTES).unwrap();
        assert!(load_uninstall_image(&cache).is_err());
        assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
        assert!(!cache.join(IMAGE_FILE).exists());
        assert_eq!(fs::read(&marker).unwrap(), MARKER_BYTES);

        let bytes = encode_state(&pending_state()).unwrap();
        fs::write(cache.join(IMAGE_FILE), &bytes).unwrap();
        for invalid in [b"".as_slice(), b"1", b"1\r\n", b"2\n", b"1\n\n"] {
            fs::write(&marker, invalid).unwrap();
            assert!(load_uninstall_image(&cache).is_err());
            assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
            assert_eq!(fs::read(&marker).unwrap(), invalid);
            assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes);
        }
    }

    #[test]
    fn strict_object_format_rejects_unknown_duplicate_missing_and_mistyped_fields() {
        let invalid = [
            "null",
            "[1,[],[]]",
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[],"extra":true}"#,
            r#"{"formatVersion":1,"formatVersion":1,"confirmedPluginIds":[],"pending":[]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"confirmedPluginIds":[],"pending":[]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[],"pending":[]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[]}"#,
            r#"{"formatVersion":1,"pending":[]}"#,
            r#"{"confirmedPluginIds":[],"pending":[]}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[]}"#,
            r#"{"formatVersion":1.0,"confirmedPluginIds":[],"pending":[]}"#,
            r#"{"formatVersion":"1","confirmedPluginIds":[],"pending":[]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[9],"pending":[]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":{}}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[["9","11111111-1111-4111-8111-111111111111"]]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[{"pluginId":"9","operationId":"11111111-1111-4111-8111-111111111111","extra":true}]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[{"pluginId":"9","pluginId":"9","operationId":"11111111-1111-4111-8111-111111111111"}]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[{"pluginId":"9","operationId":"11111111-1111-4111-8111-111111111111","operationId":"11111111-1111-4111-8111-111111111111"}]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[{"pluginId":"9"}]}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[]} false"#,
        ];
        let root = TempRoot::new();
        for bytes in invalid {
            let cache = root.seed_image(bytes.as_bytes());
            assert!(load_uninstall_image(&cache).is_err());
            assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
            assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes.as_bytes());
            assert!(!cache.join(MARKER_FILE).exists());
        }
    }

    #[test]
    fn semantic_duplicates_are_refused_but_confirmed_and_pending_may_overlap() {
        let duplicate_confirmed = r#"{"formatVersion":1,"confirmedPluginIds":["9","\u0039"],"pending":[]}"#;
        let duplicate_pending = format!(
            r#"{{"formatVersion":1,"confirmedPluginIds":[],"pending":[{{"pluginId":"9","operationId":"{OP_A}"}},{{"pluginId":"9","operationId":"{OP_B}"}}]}}"#
        );
        let duplicate_ticket = format!(
            r#"{{"formatVersion":1,"confirmedPluginIds":[],"pending":[{{"pluginId":"9","operationId":"{OP_A}"}},{{"pluginId":"10","operationId":"{OP_A}"}}]}}"#
        );
        for bytes in [duplicate_confirmed, duplicate_pending.as_str(), duplicate_ticket.as_str()] {
            assert!(decode_state(bytes.as_bytes()).is_err());
        }
        let overlapping = pending_state();
        assert!(decode_state(&encode_state(&overlapping).unwrap()).unwrap() == overlapping);
    }

    #[test]
    fn canonical_plugin_and_operation_identity_is_required_on_read_and_write() {
        for invalid_id in ["", " 9", "9 ", "\u{feff}9", "9\u{feff}", "9\n", "9\0", "09", "00"] {
            for pending in [false, true] {
                let stored = StoredUninstallImage {
                    format_version: 1,
                    confirmed_plugin_ids: if pending { vec![] } else { vec![invalid_id.into()] },
                    pending: if pending {
                        vec![JsonObject(StoredPendingUninstall { plugin_id: invalid_id.into(), operation_id: OP_A.into() })]
                    } else { vec![] },
                };
                assert!(decode_state(&serde_json::to_vec(&stored).unwrap()).is_err());
            }
            let state = UninstallState {
                confirmed: BTreeSet::from([invalid_id.into()]),
                pending: BTreeMap::new(),
                ..UninstallState::default()
            };
            assert!(encode_state(&state).is_err());
        }
        for invalid_operation in [
            "",
            "not-a-uuid",
            "11111111111141118111111111111111",
            "{11111111-1111-4111-8111-111111111111}",
            "urn:uuid:11111111-1111-4111-8111-111111111111",
            "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA",
            " 11111111-1111-4111-8111-111111111111",
        ] {
            let state = UninstallState {
                confirmed: BTreeSet::new(),
                pending: BTreeMap::from([("9".into(), invalid_operation.into())]),
                ..UninstallState::default()
            };
            assert!(encode_state(&state).is_err());
            let stored = StoredUninstallImage {
                format_version: 1,
                confirmed_plugin_ids: vec![],
                pending: vec![JsonObject(StoredPendingUninstall { plugin_id: "9".into(), operation_id: invalid_operation.into() })],
            };
            assert!(decode_state(&serde_json::to_vec(&stored).unwrap()).is_err());
        }
        let mut duplicate_tickets = pending_state();
        duplicate_tickets.pending.insert("10".into(), OP_A.into());
        assert!(encode_state(&duplicate_tickets).is_err());
        let root = TempRoot::new();
        assert!(persist_uninstall_image(&root.cache(), &DurableUninstallImage::default(), &duplicate_tickets).is_err());
        assert!(!root.cache().exists());
    }

    #[test]
    fn writes_are_sorted_and_opaque_canonical_ids_are_preserved_exactly() {
        let state = UninstallState {
            confirmed: BTreeSet::from(["official.plugin@catalog".into(), "9".into(), "10".into()]),
            pending: BTreeMap::from([("9".into(), OP_B.into()), ("10".into(), OP_A.into())]),
            ..UninstallState::default()
        };
        let bytes = encode_state(&state).unwrap();
        let JsonObject(decoded) = serde_json::from_slice::<JsonObject<StoredAdmissionImage>>(&bytes).unwrap();
        assert_eq!(decoded.format_version, 3);
        assert!(decoded.uninstall_observation.0.is_none());
        assert!(decoded.auth_refusals.is_empty());
        assert!(matches!(decoded.refresh_intent, StoredRefreshIntent::Empty(())));
        assert_eq!(decoded.confirmed_plugin_ids, ["10", "9", "official.plugin@catalog"]);
        assert_eq!(decoded.pending.iter().map(|entry| entry.0.plugin_id.as_str()).collect::<Vec<_>>(), ["10", "9"]);
        assert!(decode_state(&bytes).unwrap() == state);
        assert_eq!(encode_state(&state).unwrap(), bytes);
    }

    #[test]
    fn exact_byte_drift_and_marker_loss_prevent_replacement() {
        let root = TempRoot::new();
        let cache = root.cache();
        let state = pending_state();
        let expected = persist_uninstall_image(&cache, &DurableUninstallImage::default(), &state).unwrap();
        let image_path = cache.join(IMAGE_FILE);
        let mut changed_bytes = fs::read(&image_path).unwrap();
        changed_bytes.push(b'\n');
        fs::write(&image_path, &changed_bytes).unwrap();
        assert!(load_uninstall_image(&cache).unwrap().state == expected.state);
        assert!(persist_uninstall_image(&cache, &expected, &UninstallState::default()).is_err());
        assert_eq!(fs::read(&image_path).unwrap(), changed_bytes);

        fs::write(&image_path, expected.image_bytes.as_ref().unwrap()).unwrap();
        fs::remove_file(cache.join(MARKER_FILE)).unwrap();
        assert!(persist_uninstall_image(&cache, &expected, &UninstallState::default()).is_err());
        assert!(!cache.join(MARKER_FILE).exists());
        assert_eq!(fs::read(&image_path).unwrap(), *expected.image_bytes.as_ref().unwrap());
    }

    #[test]
    fn stale_image_and_corruption_cannot_overwrite_current_custody() {
        let root = TempRoot::new();
        let cache = root.cache();
        let initial = DurableUninstallImage::default();
        let expected = persist_uninstall_image(&cache, &initial, &pending_state()).unwrap();
        let mut newer_state = expected.state.clone();
        newer_state.pending.insert("10".into(), OP_B.into());
        let newer = persist_uninstall_image(&cache, &expected, &newer_state).unwrap();
        assert!(persist_uninstall_image(&cache, &expected, &UninstallState::default()).is_err());
        assert!(persist_uninstall_image(&cache, &initial, &UninstallState::default()).is_err());
        assert!(load_uninstall_image(&cache).unwrap() == newer);

        let invalid = b"private invalid payload must not be echoed";
        fs::write(cache.join(IMAGE_FILE), invalid).unwrap();
        let error = persist_uninstall_image(&cache, &newer, &UninstallState::default()).err().unwrap();
        assert_eq!(error, INVALID_STATE);
        assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), invalid);
        assert_eq!(fs::read(cache.join(MARKER_FILE)).unwrap(), MARKER_BYTES);
    }

    #[test]
    fn nonregular_entries_are_preserved_as_recovery_faults() {
        for name in [IMAGE_FILE, MARKER_FILE] {
            let root = TempRoot::new();
            let cache = root.cache();
            fs::create_dir(&cache).unwrap();
            fs::create_dir(cache.join(name)).unwrap();
            let sentinel = cache.join(name).join("unrelated");
            fs::write(&sentinel, b"retained").unwrap();
            assert!(load_uninstall_image(&cache).is_err());
            assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
            assert_eq!(fs::read(sentinel).unwrap(), b"retained");
        }
    }

    #[cfg(unix)]
    #[test]
    fn dangling_and_regular_file_symlinks_are_not_missing_or_authoritative_images() {
        use std::os::unix::fs::symlink;

        let root = TempRoot::new();
        let cache = root.cache();
        fs::create_dir(&cache).unwrap();
        let outside = root.0.join("unrelated");
        let image_path = cache.join(IMAGE_FILE);
        symlink(&outside, &image_path).unwrap();
        assert!(load_uninstall_image(&cache).is_err());
        assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
        assert!(fs::symlink_metadata(&image_path).unwrap().file_type().is_symlink());
        let bytes = encode_state(&pending_state()).unwrap();
        fs::write(&outside, &bytes).unwrap();
        assert!(load_uninstall_image(&cache).is_err());
        assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
        assert_eq!(fs::read(outside).unwrap(), bytes);
    }

    #[test]
    fn staging_cleanup_removes_only_a_successfully_created_owned_file() {
        let root = TempRoot::new();
        let foreign = root.0.join("uninstall-state.json.foreign.tmp");
        fs::write(&foreign, b"foreign").unwrap();
        assert!(StagedUninstallFile::create(foreign.clone(), b"replacement").is_err());
        assert_eq!(fs::read(&foreign).unwrap(), b"foreign");

        let destination = root.0.join(IMAGE_FILE);
        let staged = StagedUninstallFile::stage(&destination, b"owned").unwrap();
        let owned = staged.path.as_ref().unwrap().clone();
        assert!(owned.exists());
        drop(staged);
        assert!(!owned.exists());
        assert!(!destination.exists());
        assert_eq!(fs::read(&foreign).unwrap(), b"foreign");

        fs::create_dir(&destination).unwrap();
        fs::write(destination.join("retained"), b"retained").unwrap();
        let staged = StagedUninstallFile::stage(&destination, b"owned").unwrap();
        let owned = staged.path.as_ref().unwrap().clone();
        assert!(staged.publish(&destination).is_err());
        assert!(!owned.exists());
        assert_eq!(fs::read(destination.join("retained")).unwrap(), b"retained");
        assert_eq!(fs::read(&foreign).unwrap(), b"foreign");
    }

    #[test]
    fn marker_publication_failure_does_not_erase_the_visible_image() {
        let root = TempRoot::new();
        let cache = root.cache();
        fs::create_dir(&cache).unwrap();
        let bytes = encode_state(&pending_state()).unwrap();
        let image_path = cache.join(IMAGE_FILE);
        StagedUninstallFile::stage(&image_path, &bytes).unwrap().publish(&image_path).unwrap();
        let marker_path = cache.join(MARKER_FILE);
        fs::create_dir(&marker_path).unwrap();
        let staged = StagedUninstallFile::stage(&marker_path, MARKER_BYTES).unwrap();
        let owned = staged.path.as_ref().unwrap().clone();
        assert!(staged.publish(&marker_path).is_err());
        assert!(!owned.exists());
        assert_eq!(fs::read(&image_path).unwrap(), bytes);
        assert!(marker_path.is_dir());
        assert!(load_uninstall_image(&cache).is_err());
        fs::remove_dir(marker_path).unwrap();
        assert!(load_uninstall_image(&cache).unwrap().state == pending_state());
    }

    #[test]
    fn legacy_v1_migrates_only_on_an_explicit_transition_with_all_custody_retained() {
        let root = TempRoot::new();
        let bytes = format!(
            "{{\n  \"formatVersion\":1,\"confirmedPluginIds\":[\"9\"],\"pending\":[{{\"pluginId\":\"9\",\"operationId\":\"{OP_A}\"}}]\n}}"
        );
        let cache = root.seed_image(bytes.as_bytes());
        let legacy = load_uninstall_image(&cache).unwrap();
        assert!(legacy.state == pending_state());
        assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes.as_bytes());
        assert!(!cache.join(MARKER_FILE).exists());
        let mut next = legacy.state.clone();
        next.refresh_intent = Some(OP_B.into());
        let migrated = persist_uninstall_image(&cache, &legacy, &next).unwrap();
        assert!(migrated.state == next);
        assert!(load_uninstall_image(&cache).unwrap() == migrated);
        let written: serde_json::Value = serde_json::from_slice(
            &fs::read(cache.join(IMAGE_FILE)).unwrap(),
        ).unwrap();
        assert_eq!(written, serde_json::json!({
            "formatVersion": 3, "confirmedPluginIds": ["9"],
            "pending": [{"pluginId": "9", "operationId": OP_A}],
            "authRefusals": [], "refreshIntent": {"operationId": OP_B},
            "uninstallObservation": null,
        }));
        assert_eq!(fs::read(cache.join(MARKER_FILE)).unwrap(), b"1\n");
        assert!(serde_json::from_slice::<JsonObject<StoredUninstallImage>>(
            &fs::read(cache.join(IMAGE_FILE)).unwrap(),
        ).is_err(), "the old strict reader must not discard upgraded custody");
        assert!(serde_json::from_slice::<JsonObject<StoredAdmissionImageV2>>(
            &fs::read(cache.join(IMAGE_FILE)).unwrap(),
        ).is_err(), "the v2 strict reader must not discard the v3 observation field");
    }

    #[test]
    fn auth_identity_metadata_and_uninstall_custody_remain_distinct_and_sorted() {
        let mut state = pending_state();
        for (id, name) in [("9", "Actual nine"), ("09", "Distinct leading zero")] {
            state.auth_refusals.insert(id.into(), PluginAuthBlock {
                plugin_id: id.into(), plugin_name: name.into(),
                marketplace_name: Some("Actual market".into()),
            });
        }
        state.refresh_intent = Some(OP_B.into());
        state.uninstall_observation = Some(OP_C.into());
        let encoded = encode_state(&state).unwrap();
        assert!(decode_state(&encoded).unwrap() == state);
        let json: serde_json::Value = serde_json::from_slice(&encoded).unwrap();
        assert_eq!(json["authRefusals"], serde_json::json!([
            {"pluginId":"09","pluginName":"Distinct leading zero","marketplaceName":"Actual market"},
            {"pluginId":"9","pluginName":"Actual nine","marketplaceName":"Actual market"},
        ]));
        assert_eq!(json["confirmedPluginIds"], serde_json::json!(["9"]));
        assert_eq!(json["pending"], serde_json::json!([{"pluginId":"9","operationId":OP_A}]));
        assert_eq!(json["refreshIntent"], serde_json::json!({"operationId":OP_B}));
        assert_eq!(json["uninstallObservation"], serde_json::json!(OP_C));
        assert_eq!(encode_state(&state).unwrap(), encoded);
    }

    #[test]
    fn v2_requires_strict_auth_objects_and_an_explicit_nullable_refresh_field() {
        let valid = r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null}"#;
        assert!(decode_state(valid.as_bytes()).unwrap() == UninstallState::default());
        let invalid = [
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[]}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"refreshIntent":null}"#,
            r#"{"formatVersion":1,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null}"#,
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":null,"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":[]}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":{}}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":{"operationId":"11111111-1111-4111-8111-111111111111","extra":true}}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":{"operationId":"11111111-1111-4111-8111-111111111111","operationId":"11111111-1111-4111-8111-111111111111"}}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"authRefusals":[],"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[["9","name"]],"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[{"pluginId":"9"}],"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[{"pluginId":"9","pluginName":"name","extra":true}],"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[{"pluginId":"9","pluginId":"9","pluginName":"name"}],"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[{"pluginId":"9","pluginName":"name","pluginName":"name"}],"refreshIntent":null}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[{"pluginId":"9","pluginName":"name"},{"pluginId":"\u0039","pluginName":"name"}],"refreshIntent":null}"#,
        ];
        for bytes in invalid {
            let root = TempRoot::new();
            let cache = root.seed_image(bytes.as_bytes());
            assert!(load_uninstall_image(&cache).is_err(), "invalid durable shape: {bytes}");
            assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
            assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes.as_bytes());
            assert!(!cache.join(MARKER_FILE).exists());
        }
    }

    #[test]
    fn v2_auth_and_refresh_identity_reject_malformed_or_conflicting_values() {
        for id in ["", " 9", "9 ", "9\n"] {
            let mut state = UninstallState::default();
            state.auth_refusals.insert(id.into(), PluginAuthBlock {
                plugin_id: id.into(), plugin_name: "Actual name".into(), marketplace_name: None,
            });
            assert!(encode_state(&state).is_err());
        }
        let mut mismatch = UninstallState::default();
        mismatch.auth_refusals.insert("9".into(), PluginAuthBlock {
            plugin_id: "10".into(), plugin_name: "Actual name".into(), marketplace_name: None,
        });
        assert!(encode_state(&mismatch).is_err());
        for operation in ["", "bad", "11111111111141118111111111111111", "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA", OP_A] {
            let mut state = pending_state();
            state.refresh_intent = Some(operation.into());
            assert!(encode_state(&state).is_err());
            let invalid = serde_json::json!({
                "formatVersion":2,"confirmedPluginIds":["9"],
                "pending":[{"pluginId":"9","operationId":OP_A}],
                "authRefusals":[],"refreshIntent":{"operationId":operation},
            });
            assert!(decode_state(&serde_json::to_vec(&invalid).unwrap()).is_err());
        }
    }

    #[test]
    fn failed_auth_write_accepts_only_exact_expected_or_its_visible_candidate() {
        let root = TempRoot::new();
        let cache = root.cache();
        let mut initial = pending_state();
        initial.uninstall_observation = Some(OP_C.into());
        let expected = persist_uninstall_image(&cache, &DurableUninstallImage::default(), &initial).unwrap();
        let mut next = expected.state.clone();
        next.refresh_intent = Some(OP_B.into());
        next.auth_refusals.insert("9".into(), PluginAuthBlock {
            plugin_id:"9".into(),plugin_name:"Actual nine".into(),marketplace_name:None,
        });
        let before = with_image_publish_failure(1, ImagePublishFailure::BeforeRename, || {
            persist_uninstall_image(&cache, &expected, &next)
        });
        assert!(before.is_err());
        assert!(inspect_failed_auth_publication(&cache, &expected, &next).unwrap() == expected);
        let after = with_image_publish_failure(1, ImagePublishFailure::AfterRename, || {
            persist_uninstall_image(&cache, &expected, &next)
        });
        assert!(after.is_err());
        let visible = inspect_failed_auth_publication(&cache, &expected, &next).unwrap();
        assert!(visible.state == next);
        assert!(visible != expected);
        let mut foreign = fs::read(cache.join(IMAGE_FILE)).unwrap();
        foreign.push(b'\n');
        fs::write(cache.join(IMAGE_FILE), &foreign).unwrap();
        assert!(load_uninstall_image(&cache).unwrap().state == next);
        assert!(inspect_failed_auth_publication(&cache, &expected, &next).is_err());
        assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), foreign);
        assert_eq!(fs::read(cache.join(MARKER_FILE)).unwrap(), MARKER_BYTES);
    }
    #[test]
    fn legacy_v2_migrates_to_v3_without_inventing_uninstall_observation() {
        for initialized in [false, true] {
            let root = TempRoot::new();
            let bytes = format!(
                r#"{{"formatVersion":2,"confirmedPluginIds":["9"],"pending":[{{"pluginId":"9","operationId":"{OP_A}"}}],"authRefusals":[{{"pluginId":"9","pluginName":"Actual nine"}}],"refreshIntent":{{"operationId":"{OP_B}"}}}}"#
            );
            let cache = root.seed_image(bytes.as_bytes());
            if initialized {
                fs::write(cache.join(MARKER_FILE), MARKER_BYTES).unwrap();
            }
            let legacy = load_uninstall_image(&cache).unwrap();
            assert!(legacy.state.uninstall_observation.is_none());
            assert_eq!(legacy.state.confirmed, BTreeSet::from(["9".into()]));
            assert_eq!(legacy.state.pending, BTreeMap::from([("9".into(), OP_A.into())]));
            assert_eq!(legacy.state.auth_refusals["9"].plugin_name, "Actual nine");
            assert_eq!(legacy.state.refresh_intent.as_deref(), Some(OP_B));
            assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes.as_bytes());
            assert_eq!(cache.join(MARKER_FILE).exists(), initialized);

            let mut next = legacy.state.clone();
            next.refresh_intent = None;
            let migrated = persist_uninstall_image(&cache, &legacy, &next).unwrap();
            assert!(migrated.state == next);
            assert!(load_uninstall_image(&cache).unwrap() == migrated);
            let written = fs::read(cache.join(IMAGE_FILE)).unwrap();
            assert_eq!(serde_json::from_slice::<serde_json::Value>(&written).unwrap(), serde_json::json!({
                "formatVersion": 3,
                "confirmedPluginIds": ["9"],
                "pending": [{"pluginId":"9","operationId":OP_A}],
                "authRefusals": [{"pluginId":"9","pluginName":"Actual nine"}],
                "refreshIntent": null,
                "uninstallObservation": null,
            }));
            assert!(serde_json::from_slice::<JsonObject<StoredAdmissionImageV2>>(&written).is_err());
            assert!(serde_json::from_slice::<JsonObject<StoredUninstallImage>>(&written).is_err());
            assert_eq!(fs::read(cache.join(MARKER_FILE)).unwrap(), MARKER_BYTES);
        }
    }

    #[test]
    fn v3_requires_explicit_nullable_canonical_uninstall_observation() {
        let valid = r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null}"#;
        assert!(decode_state(valid.as_bytes()).unwrap() == UninstallState::default());
        let mut observed = pending_state();
        observed.uninstall_observation = Some(OP_C.into());
        let encoded = encode_state(&observed).unwrap();
        assert!(decode_state(&encoded).unwrap() == observed);
        assert_eq!(
            serde_json::from_slice::<serde_json::Value>(&encoded).unwrap()["uninstallObservation"],
            OP_C,
        );
        let mut invalid = vec![
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null,"uninstallObservation":null}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null,"uninstall\u004fbservation":null}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null,"extra":true}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"refreshIntent":null,"uninstallObservation":null}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[{"pluginId":"9","pluginName":"name"},{"pluginId":"\u0039","pluginName":"name"}],"refreshIntent":null,"uninstallObservation":null}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"uninstallObservation":null}"#.to_string(),
            r#"{"formatVersion":3,"confirmedPluginIds":[],"pending":[],"refreshIntent":null,"uninstallObservation":null}"#.to_string(),
            r#"{"formatVersion":2,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null}"#.to_string(),
            r#"{"formatVersion":4,"confirmedPluginIds":[],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null}"#.to_string(),
            r#"[3,[],[],[],null,null]"#.to_string(),
        ];
        for value in [
            serde_json::json!(false), serde_json::json!(3), serde_json::json!([]),
            serde_json::json!({}), serde_json::json!({"operationId": OP_C}),
        ] {
            let mut image: serde_json::Value = serde_json::from_str(valid).unwrap();
            image["uninstallObservation"] = value;
            invalid.push(serde_json::to_string(&image).unwrap());
        }
        for value in [
            "", "bad", "33333333333343338333333333333333",
            "{33333333-3333-4333-8333-333333333333}",
            "urn:uuid:33333333-3333-4333-8333-333333333333",
            "AAAAAAAA-AAAA-4AAA-8AAA-AAAAAAAAAAAA",
            " 33333333-3333-4333-8333-333333333333",
            "33333333-3333-4333-8333-333333333333 ",
        ] {
            let mut image: serde_json::Value = serde_json::from_str(valid).unwrap();
            image["uninstallObservation"] = serde_json::json!(value);
            invalid.push(serde_json::to_string(&image).unwrap());
            let mut state = pending_state();
            state.uninstall_observation = Some(value.into());
            assert!(encode_state(&state).is_err(), "invalid observation accepted on write: {value}");
        }
        for bytes in invalid {
            let root = TempRoot::new();
            let cache = root.seed_image(bytes.as_bytes());
            assert!(load_uninstall_image(&cache).is_err(), "invalid v3 shape: {bytes}");
            assert!(persist_uninstall_image(&cache, &DurableUninstallImage::default(), &pending_state()).is_err());
            assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes.as_bytes());
            assert!(!cache.join(MARKER_FILE).exists());
        }
    }

    #[test]
    fn v3_observation_cannot_be_dropped_by_an_explicit_image_transition() {
        let root = TempRoot::new();
        let cache = root.cache();
        let mut initial = pending_state();
        initial.uninstall_observation = Some(OP_C.into());
        let expected = persist_uninstall_image(&cache, &DurableUninstallImage::default(), &initial).unwrap();
        let bytes = fs::read(cache.join(IMAGE_FILE)).unwrap();
        let mut lost = expected.state.clone();
        lost.uninstall_observation = None;
        assert_eq!(persist_uninstall_image(&cache, &expected, &lost).err().unwrap(), CHANGED_STATE);
        assert_eq!(fs::read(cache.join(IMAGE_FILE)).unwrap(), bytes);
        assert_eq!(fs::read(cache.join(MARKER_FILE)).unwrap(), MARKER_BYTES);

        let mut rotated = expected.state.clone();
        rotated.uninstall_observation = Some(OP_B.into());
        let current = persist_uninstall_image(&cache, &expected, &rotated).unwrap();
        assert!(current.state == rotated);
        assert!(load_uninstall_image(&cache).unwrap() == current);
        assert!(persist_uninstall_image(&cache, &expected, &initial).is_err());
        assert!(load_uninstall_image(&cache).unwrap() == current);
    }

    struct ObservationLoader {
        result: std::sync::Mutex<Result<super::super::LoadedPlugins, String>>,
        calls: std::sync::atomic::AtomicUsize,
    }

    impl super::super::PluginSkillsLoader for ObservationLoader {
        fn load(&self) -> Result<super::super::LoadedPlugins, String> {
            self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            self.result.lock().unwrap().clone()
        }
    }

    fn observation_loader(
        result: Result<super::super::LoadedPlugins, String>,
    ) -> std::sync::Arc<ObservationLoader> {
        std::sync::Arc::new(ObservationLoader {
            result: std::sync::Mutex::new(result),
            calls: std::sync::atomic::AtomicUsize::new(0),
        })
    }

    fn zero_skill_nine(root: &TempRoot) -> super::super::LoadedPlugins {
        let install_path = root.0.join("plugins/cache/market/nine");
        fs::create_dir_all(&install_path).unwrap();
        super::super::LoadedPlugins {
            plugins: vec![super::super::InstalledPlugin {
                identifier: super::super::PluginIdentifier {
                    source: "cursor-first-party".into(),
                    name: "nine".into(),
                    plugin_db_id: Some("9".into()),
                    version: Some("same".into()),
                },
                display_name: Some("Actual nine".into()),
                load_error: None,
                install_path,
                skills: Vec::new(),
            }],
            auth_blocked: Vec::new(),
            listed_plugin_ids: vec!["9".into()],
            listed_cache_keys: vec![super::super::ListedCacheKey {
                marketplace_slug: "market".into(), plugin_id: "nine".into(),
            }],
            publisher_facts: std::collections::HashMap::new(),
            current_user_id: None,
        }
    }

    fn advanced_observation(root: &TempRoot, previous: Option<&str>) -> String {
        let current = load_uninstall_image(&root.cache()).unwrap()
            .state.uninstall_observation.expect("explicit uninstall transition has an observation");
        assert_eq!(Uuid::parse_str(&current).unwrap().to_string(), current);
        assert_ne!(Some(current.as_str()), previous);
        current
    }

    #[test]
    fn uninstall_observation_rotates_for_real_transitions_and_survives_auth_refreshes() {
        use super::super::{PluginUninstallSettlement, SandPluginSkillsService};
        use std::sync::Arc;

        let root = TempRoot::new();
        let loaded = zero_skill_nine(&root);
        let loader = observation_loader(Ok(loaded.clone()));
        let service = SandPluginSkillsService::new(&root.0, loader.clone())
            .with_now(Arc::new(|| 1_000));
        assert!(service.sync("seed-zero-skill").unwrap().is_empty());
        assert!(load_uninstall_image(&root.cache()).unwrap().state.uninstall_observation.is_none());
        let mut previous = None;
        for settlement in [
            PluginUninstallSettlement::NotDispatched,
            PluginUninstallSettlement::NotRemoved,
            PluginUninstallSettlement::Removed,
        ] {
            let intent = service.begin_uninstall("9").unwrap();
            let reserved = advanced_observation(&root, previous.as_deref());
            assert_eq!(load_uninstall_image(&root.cache()).unwrap().state.pending["9"], intent.operation_id);
            service.settle_uninstall(&intent, settlement).unwrap();
            let settled = advanced_observation(&root, Some(&reserved));
            let image = load_uninstall_image(&root.cache()).unwrap();
            assert!(image.state.pending.is_empty());
            assert_eq!(image.state.confirmed.contains("9"), settlement == PluginUninstallSettlement::Removed);
            let bytes = fs::read(root.cache().join(IMAGE_FILE)).unwrap();
            assert!(service.settle_uninstall(&intent, settlement).is_err());
            assert_eq!(fs::read(root.cache().join(IMAGE_FILE)).unwrap(), bytes);
            previous = Some(settled);
        }
        let confirmed = previous.unwrap();
        *loader.result.lock().unwrap() = Err("offline".into());
        assert_eq!(service.sync("known-offline").unwrap_err(), "offline");
        assert_eq!(load_uninstall_image(&root.cache()).unwrap().state.uninstall_observation.as_deref(), Some(confirmed.as_str()));

        let mut refused = loaded.clone();
        refused.auth_blocked.push(PluginAuthBlock {
            plugin_id: "9".into(), plugin_name: "Actual nine".into(), marketplace_name: None,
        });
        *loader.result.lock().unwrap() = Ok(refused);
        assert!(service.sync("auth-only-negative").unwrap().is_empty());
        let negative = load_uninstall_image(&root.cache()).unwrap();
        assert_eq!(negative.state.uninstall_observation.as_deref(), Some(confirmed.as_str()));
        assert!(negative.state.auth_refusals.contains_key("9"));
        assert!(negative.state.confirmed.contains("9"));
        assert!(service.admission_reader().is_denied("9"));

        let obsolete = root.0.join("plugins/cache/market/obsolete");
        fs::create_dir_all(&obsolete).unwrap();
        fs::write(obsolete.join("retained-until-commit"), b"old").unwrap();
        *loader.result.lock().unwrap() = Ok(loaded);
        assert!(service.sync("fresh-zero-skill-release").unwrap().is_empty());
        let released = advanced_observation(&root, Some(&confirmed));
        let positive = load_uninstall_image(&root.cache()).unwrap();
        assert!(positive.state.confirmed.is_empty());
        assert!(positive.state.auth_refusals.is_empty());
        assert!(!service.admission_reader().is_denied("9"));
        assert!(!obsolete.exists(), "this commit's own observation rotation must not suppress its valid cleanup");
        assert!(service.sync("stable-auth-only-refresh").unwrap().is_empty());
        assert_eq!(load_uninstall_image(&root.cache()).unwrap().state.uninstall_observation.as_deref(), Some(released.as_str()));
        assert_eq!(loader.calls.load(std::sync::atomic::Ordering::SeqCst), 5);
    }

    #[test]
    fn retained_v3_observation_loss_to_legacy_or_null_latches_recovery() {
        use super::super::SandPluginSkillsService;

        for legacy in [
            r#"{"formatVersion":1,"confirmedPluginIds":["9"],"pending":[]}"#,
            r#"{"formatVersion":2,"confirmedPluginIds":["9"],"pending":[],"authRefusals":[],"refreshIntent":null}"#,
            r#"{"formatVersion":3,"confirmedPluginIds":["9"],"pending":[],"authRefusals":[],"refreshIntent":null,"uninstallObservation":null}"#,
        ] {
            for writer_first in [false, true] {
                let root = TempRoot::new();
                let initial = UninstallState {
                    confirmed: BTreeSet::from(["9".into()]),
                    uninstall_observation: Some(OP_C.into()),
                    ..UninstallState::default()
                };
                let accepted = persist_uninstall_image(
                    &root.cache(), &DurableUninstallImage::default(), &initial,
                ).unwrap();
                let bytes = fs::read(root.cache().join(IMAGE_FILE)).unwrap();
                let loader = observation_loader(Err("must-not-load".into()));
                let service = SandPluginSkillsService::new(&root.0, loader.clone());
                let reader = service.admission_reader();
                assert!(reader.is_denied("9"));
                assert!(!reader.is_denied("10"));
                assert!(decode_state(legacy.as_bytes()).unwrap().uninstall_observation.is_none());
                fs::write(root.cache().join(IMAGE_FILE), legacy).unwrap();

                if writer_first {
                    assert_eq!(service.begin_uninstall("10").unwrap_err(), super::super::UNINSTALL_RECOVERY_UNAVAILABLE);
                } else {
                    assert!(reader.is_denied("10"));
                }
                assert!(reader.is_denied("9"));
                assert!(reader.is_denied("10"));
                assert_eq!(service.sync("lost-causal-identity").unwrap_err(), super::super::UNINSTALL_RECOVERY_UNAVAILABLE);
                assert_eq!(service.uninstall_recovery_status(), Some(serde_json::json!({
                    "confirmedPluginIds":["9"], "pending":[], "fault":"uninstall-state-unavailable",
                })));
                assert!(service.admission.state.lock().unwrap().durable == accepted);
                assert_eq!(loader.calls.load(std::sync::atomic::Ordering::SeqCst), 0);
                assert_eq!(fs::read(root.cache().join(IMAGE_FILE)).unwrap(), legacy.as_bytes());
                assert_eq!(fs::read(root.cache().join(MARKER_FILE)).unwrap(), MARKER_BYTES);

                fs::write(root.cache().join(IMAGE_FILE), &bytes).unwrap();
                assert!(reader.is_denied("10"), "restoration cannot heal this retained owner");
                let fresh = SandPluginSkillsService::new(&root.0, loader.clone());
                assert!(fresh.admission_reader().is_denied("9"));
                assert!(!fresh.admission_reader().is_denied("10"));
                assert_eq!(loader.calls.load(std::sync::atomic::Ordering::SeqCst), 0);
                assert_eq!(fs::read(root.cache().join(IMAGE_FILE)).unwrap(), bytes);
            }
        }
    }

}
