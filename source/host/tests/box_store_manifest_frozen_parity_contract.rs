use std::collections::BTreeMap;
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::extensions::box_store_sync::box_object_store::{
    BoxObjectStore, LocalFsObjectStore,
};
use mahayana_host_runtime::extensions::box_store_sync::box_store_hydration::
    write_hydration_handoff_marker;
use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest::{
    BoxStoreManifestParseError, ManifestSaveOptions, configured_manifest_entries,
    load_manifest_for_write, mark_legacy_hydration_complete_for_handoff,
    mark_legacy_hydration_incomplete, parse_manifest_bytes, read_manifest_strict,
    serialize_manifest_bytes, write_manifest_with_retry,
};
use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest_format::{
    BOX_STORE_LEGACY_MANIFEST_VERSION, BOX_STORE_MANIFEST_REL_PATH, BOX_STORE_MANIFEST_VERSION,
    BoxStoreManifestEntry,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-manifest-parity-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn file(sha: &str) -> BoxStoreManifestEntry {
    BoxStoreManifestEntry::File {
        sha: sha.into(),
        size: 1,
        mode: 0o600,
    }
}

fn manifest(entries: &[(&str, BoxStoreManifestEntry)]) -> BTreeMap<String, BoxStoreManifestEntry> {
    entries
        .iter()
        .map(|(path, entry)| ((*path).to_string(), entry.clone()))
        .collect()
}

#[test]
fn frozen_manifest_parse_and_v1_projection_are_fail_closed_and_deterministic() {
    assert!(parse_manifest_bytes(b"{not-json").unwrap().is_none());
    assert!(matches!(
        parse_manifest_bytes(br#"{"version":3}"#),
        Err(BoxStoreManifestParseError::UnsupportedVersion(3))
    ));
    let conflict = br#"{
        "version":2,
        "updatedAtMs":1,
        "entries":{
            "a":{"kind":"file","sha":"a","size":1,"mode":384},
            "a/b":{"kind":"file","sha":"b","size":1,"mode":384}
        }
    }"#;
    assert!(matches!(
        parse_manifest_bytes(conflict),
        Err(BoxStoreManifestParseError::PathConflict)
    ));

    let entries = manifest(&[
        ("home/box/file", file("file")),
        (
            "home/box/link",
            BoxStoreManifestEntry::Symlink { target: "file".into() },
        ),
    ]);
    let configured = configured_manifest_entries(&entries, false);
    assert_eq!(configured.len(), 1);
    assert!(matches!(
        configured.get("home/box/file"),
        Some(BoxStoreManifestEntry::LegacyFile { sha, size: 1 }) if sha == "file"
    ));
    let bytes = serialize_manifest_bytes(&entries, false, Some("writer"), Some(false), 7).unwrap();
    let decoded = parse_manifest_bytes(&bytes).unwrap().unwrap();
    assert_eq!(decoded.version, BOX_STORE_LEGACY_MANIFEST_VERSION);
    assert_eq!(decoded.fully_hydrated, Some(false));
    assert_eq!(decoded.entries, configured);
}

#[test]
fn hydration_handoff_load_is_fail_closed_and_legacy_manifest_clears_stale_marker() {
    let root = temp_root("handoff");
    let store = LocalFsObjectStore::new(root.join("objects"));
    let marker = root.join("state/.box-store-legacy-hydration-complete");
    write_hydration_handoff_marker(&marker).expect("write marker");

    let missing = load_manifest_for_write(&store, true, Some(&marker)).unwrap_err();
    assert!(missing.contains("manifest missing while legacy hydration handoff is pending"));
    store.put(BOX_STORE_MANIFEST_REL_PATH, b"{invalid").unwrap();
    let unreadable = load_manifest_for_write(&store, true, Some(&marker)).unwrap_err();
    assert!(unreadable.contains("manifest unreadable while legacy hydration handoff is pending"));

    let entries = manifest(&[("home/box/file", file("a"))]);
    let legacy = serialize_manifest_bytes(&entries, false, Some("legacy"), None, 1).unwrap();
    store.put(BOX_STORE_MANIFEST_REL_PATH, &legacy).unwrap();
    let loaded = load_manifest_for_write(&store, false, Some(&marker)).unwrap();
    assert_eq!(loaded.manifest, configured_manifest_entries(&entries, false));
    assert!(!loaded.manifest_v2);
    assert!(!marker.exists());
    let _ = fs::remove_dir_all(root);
}

#[test]
fn manifest_cas_reloads_winner_retries_and_preserves_fully_hydrated() {
    let root = temp_root("retry");
    let store = LocalFsObjectStore::new(root.join("objects"));
    let first = manifest(&[("home/box/a", file("a"))]);
    let first_bytes = serialize_manifest_bytes(&first, true, Some("writer-a"), Some(false), 1).unwrap();
    store.put(BOX_STORE_MANIFEST_REL_PATH, &first_bytes).unwrap();
    let loaded = load_manifest_for_write(&store, true, None).unwrap();

    let concurrent = manifest(&[("home/box/b", file("b"))]);
    let concurrent_bytes = serialize_manifest_bytes(&concurrent, true, Some("writer-b"), Some(false), 2).unwrap();
    store.put(BOX_STORE_MANIFEST_REL_PATH, &concurrent_bytes).unwrap();

    let intended = manifest(&[("home/box/c", file("c"))]);
    let result = write_manifest_with_retry(
        &store, "store-a", loaded.baseline, &intended, true, Some("writer-c"),
        loaded.fully_hydrated, ManifestSaveOptions::default(), 3, 0, &|_| {}, &|| false,
    ).unwrap();
    assert_eq!(result.attempts, 2);
    assert!(result.written);
    assert_eq!(result.fully_hydrated, Some(false));
    let persisted = read_manifest_strict(&store).unwrap().unwrap();
    assert_eq!(persisted.version, BOX_STORE_MANIFEST_VERSION);
    assert_eq!(persisted.entries, intended);
    assert_eq!(persisted.fully_hydrated, Some(false));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn exhausted_conflict_can_accept_exact_canonical_winner_and_handoff_updates_round_trip() {
    let root = temp_root("winner");
    let store = LocalFsObjectStore::new(root.join("objects"));
    let marker = root.join("state/.box-store-legacy-hydration-complete");

    let initial = manifest(&[("home/box/a", file("a"))]);
    let initial_bytes = serialize_manifest_bytes(&initial, true, Some("initial"), Some(true), 1).unwrap();
    store.put(BOX_STORE_MANIFEST_REL_PATH, &initial_bytes).unwrap();
    let loaded = load_manifest_for_write(&store, true, None).unwrap();

    let intended = manifest(&[("home/box/winner", file("winner"))]);
    let winner = serialize_manifest_bytes(&intended, true, Some("winner"), Some(true), 2).unwrap();
    store.put(BOX_STORE_MANIFEST_REL_PATH, &winner).unwrap();
    let accepted = write_manifest_with_retry(
        &store, "store-a", loaded.baseline, &intended, true, Some("loser"),
        loaded.fully_hydrated,
        ManifestSaveOptions { accept_matching_canonical_on_conflict: true, ..ManifestSaveOptions::default() },
        1, 0, &|_| {}, &|| false,
    ).unwrap();
    assert!(!accepted.written);
    assert!(accepted.accepted_matching_canonical);

    write_hydration_handoff_marker(&marker).unwrap();
    mark_legacy_hydration_incomplete(
        &store, "store-a", &marker, true, Some("hydrate"), 3, 0, &|_| {},
    ).unwrap();
    assert!(!marker.exists());
    assert_eq!(read_manifest_strict(&store).unwrap().unwrap().fully_hydrated, Some(false));
    mark_legacy_hydration_complete_for_handoff(&marker).unwrap();
    assert!(marker.exists());
    let _ = fs::remove_dir_all(root);
}
