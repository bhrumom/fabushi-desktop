use std::fs;

use mahayana_host_runtime::extensions::box_store_sync::box_store_hydration::{
    BOX_STORE_HYDRATION_HANDOFF_MANIFEST_PATH, HydrationEvidence,
    is_box_store_fully_hydrated, is_hydration_handoff_manifest_path,
    remove_hydration_handoff_marker, write_hydration_handoff_marker,
};
use uuid::Uuid;

#[test]
fn hydration_evidence_requires_complete_counts_and_no_failures() {
    let complete = HydrationEvidence {
        failures: Some(Vec::new()),
        manifest_entries: Some(3),
        files: Some(3),
        verified: Some(3),
        ..HydrationEvidence::default()
    };
    assert!(is_box_store_fully_hydrated(Some(&complete)));

    let mut failed = complete.clone();
    failed.failures = Some(vec!["download failed".into()]);
    assert!(!is_box_store_fully_hydrated(Some(&failed)));

    let mut partial = complete.clone();
    partial.verified = Some(2);
    assert!(!is_box_store_fully_hydrated(Some(&partial)));
    assert!(!is_box_store_fully_hydrated(None));
}

#[test]
fn legacy_hydration_uses_authoritative_store_db_counts() {
    let legacy = HydrationEvidence {
        failures: Some(Vec::new()),
        manifest_entries: Some(100),
        files: Some(1),
        verified: Some(0),
        hydrate_source: Some("legacy".into()),
        authoritative_store_db_entries: Some(4),
        restored_store_db_entries: Some(4),
    };
    assert!(is_box_store_fully_hydrated(Some(&legacy)));

    let mut incomplete = legacy.clone();
    incomplete.restored_store_db_entries = Some(3);
    assert!(!is_box_store_fully_hydrated(Some(&incomplete)));

    let mut missing = legacy;
    missing.authoritative_store_db_entries = None;
    assert!(!is_box_store_fully_hydrated(Some(&missing)));
}

#[test]
fn hydration_handoff_path_recognizes_marker_and_pid_temp_files_only() {
    assert!(is_hydration_handoff_manifest_path(
        BOX_STORE_HYDRATION_HANDOFF_MANIFEST_PATH
    ));
    assert!(is_hydration_handoff_manifest_path(&format!(
        "{BOX_STORE_HYDRATION_HANDOFF_MANIFEST_PATH}.123.tmp"
    )));
    assert!(!is_hydration_handoff_manifest_path(&format!(
        "{BOX_STORE_HYDRATION_HANDOFF_MANIFEST_PATH}.tmp"
    )));
    assert!(!is_hydration_handoff_manifest_path("manifest.json"));
}

#[test]
fn hydration_handoff_marker_is_durable_and_removable() {
    let root = std::env::temp_dir().join(format!(
        "fabushi-hydration-marker-{}",
        Uuid::new_v4().simple()
    ));
    let marker = root.join("nested").join(".box-store-legacy-hydration-complete");

    write_hydration_handoff_marker(&marker).expect("write marker");
    assert_eq!(fs::read_to_string(&marker).expect("read marker"), "complete\n");
    let parent = marker.parent().expect("marker parent");
    let temp_prefix = format!(
        "{}.",
        marker.file_name().expect("marker name").to_string_lossy()
    );
    assert!(!fs::read_dir(parent)
        .expect("list parent")
        .filter_map(Result::ok)
        .any(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with(&temp_prefix) && name.ends_with(".tmp")
        }));

    remove_hydration_handoff_marker(&marker).expect("remove marker");
    assert!(!marker.exists());
    assert!(parent.is_dir());

    let _ = fs::remove_dir_all(root);
}
