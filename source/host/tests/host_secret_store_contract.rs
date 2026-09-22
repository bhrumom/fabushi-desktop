use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use mahayana_host_runtime::host_secret_store::{
    clear_host_machine_id_cache_for_tests, get_or_create_host_machine_id, read_machine_id,
    write_machine_id,
};

fn temp_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!("fabushi-{label}-{suffix}"))
}

#[test]
fn host_secret_store_reads_writes_and_process_caches_machine_id() {
    clear_host_machine_id_cache_for_tests();
    let root = temp_root("host-secret-store");
    let path = root.join("nested").join("host-secrets.json");

    assert_eq!(read_machine_id(&path), None);
    write_machine_id(&path, "machine-existing").expect("write existing id");
    assert_eq!(read_machine_id(&path).as_deref(), Some("machine-existing"));

    let first = get_or_create_host_machine_id(Some(&path)).expect("read existing id");
    assert_eq!(first, "machine-existing");

    write_machine_id(&path, "machine-changed-on-disk").expect("replace id on disk");
    let cached = get_or_create_host_machine_id(Some(&path)).expect("cached id");
    assert_eq!(cached, "machine-existing");

    clear_host_machine_id_cache_for_tests();
    let reloaded = get_or_create_host_machine_id(Some(&path)).expect("reload id");
    assert_eq!(reloaded, "machine-changed-on-disk");

    fs::remove_dir_all(root).expect("remove test root");
    clear_host_machine_id_cache_for_tests();
}

#[test]
fn host_secret_store_creates_uuid_and_ignores_invalid_json() {
    clear_host_machine_id_cache_for_tests();
    let root = temp_root("host-secret-create");
    fs::create_dir_all(&root).expect("create root");
    let path = root.join("host-secrets.json");
    fs::write(&path, "{not-json").expect("write invalid file");
    assert_eq!(read_machine_id(&path), None);

    let created = get_or_create_host_machine_id(Some(&path)).expect("create machine id");
    assert!(!created.is_empty());
    assert_eq!(read_machine_id(&path).as_deref(), Some(created.as_str()));

    fs::remove_dir_all(root).expect("remove test root");
    clear_host_machine_id_cache_for_tests();
}
