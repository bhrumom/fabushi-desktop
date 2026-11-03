//! Private durable image for the existing Plugin Skills owner's uninstall custody.
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

const IMAGE_FILE: &str = "uninstall-state.json";
const MARKER_FILE: &str = "uninstall-state.version";
const MARKER_BYTES: &[u8] = b"1\n";
const INVALID_STATE: &str = "plugin uninstall state is unavailable";
const CHANGED_STATE: &str = "plugin uninstall state changed";
const WRITE_FAILED: &str = "could not persist plugin uninstall state";
const PUBLICATION_UNKNOWN: &str = "plugin uninstall state publication outcome is unknown";

#[derive(Clone, Default, PartialEq, Eq)]
pub(super) struct UninstallState {
    pub confirmed: BTreeSet<String>,
    pub pending: BTreeMap<String, String>,
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
    let JsonObject(stored) = serde_json::from_slice::<JsonObject<StoredUninstallImage>>(bytes)
        .map_err(|_| INVALID_STATE.to_string())?;
    if stored.format_version != 1 {
        return Err(INVALID_STATE.into());
    }
    let mut state = UninstallState::default();
    for id in stored.confirmed_plugin_ids {
        if !state.confirmed.insert(id) {
            return Err(INVALID_STATE.into());
        }
    }
    for JsonObject(pending) in stored.pending {
        if state
            .pending
            .insert(pending.plugin_id, pending.operation_id)
            .is_some()
        {
            return Err(INVALID_STATE.into());
        }
    }
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
    Ok(())
}

fn encode_state(state: &UninstallState) -> Result<Vec<u8>, String> {
    validate_state(state)?;
    let stored = StoredUninstallImage {
        format_version: 1,
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
    require_unchanged(cache_dir, expected)?;
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
        let path = self.path.as_ref().expect("staged uninstall file is owned");
        fs::rename(path, destination).map_err(|_| WRITE_FAILED.to_string())?;
        // A later barrier/readback failure must not erase the published image.
        // Nor may Drop remove another file later created at the staging name.
        self.path = None;
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

#[cfg(test)]
mod tests {
    use super::*;

    const OP_A: &str = "11111111-1111-4111-8111-111111111111";
    const OP_B: &str = "22222222-2222-4222-8222-222222222222";

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
        };
        let bytes = encode_state(&state).unwrap();
        let JsonObject(decoded) = serde_json::from_slice::<JsonObject<StoredUninstallImage>>(&bytes).unwrap();
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
}
