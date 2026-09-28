use std::collections::BTreeMap;
use std::path::PathBuf;

use mahayana_host_runtime::extensions::box_store_sync::production::{
    ProductionBoxStoreSyncMode, resolve_production_box_store_sync_mode,
};

#[test]
fn production_mode_is_disabled_without_sync_flag() {
    let env = BTreeMap::new();
    assert_eq!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::Disabled
    );
}

#[test]
fn production_mode_selects_local_fs_only_with_safe_absolute_configuration() {
    let mut env = BTreeMap::from([
        ("SAND_BOX_STORE_SYNC".to_string(), "1".to_string()),
        ("SAND_BOX_STORE_LOCAL_DIR".to_string(), "/tmp/fabushi-box-store".to_string()),
        ("SAND_BOX_STORE_ID".to_string(), "123e4567-e89b-12d3-a456-426614174000".to_string()),
    ]);
    assert_eq!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::LocalFs {
            base_dir: PathBuf::from("/tmp/fabushi-box-store"),
            store_id_override: Some("123e4567-e89b-12d3-a456-426614174000".to_string()),
        }
    );

    env.insert("SAND_BOX_STORE_ID".to_string(), "../escape".to_string());
    assert_eq!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::LocalFs {
            base_dir: PathBuf::from("/tmp/fabushi-box-store"),
            store_id_override: None,
        }
    );
}

#[test]
fn production_mode_uses_source_map_when_local_store_id_is_not_overridden() {
    let env = BTreeMap::from([
        ("SAND_BOX_STORE_SYNC".to_string(), "1".to_string()),
        ("SAND_BOX_STORE_LOCAL_DIR".to_string(), "/tmp/fabushi-box-store".to_string()),
    ]);
    assert_eq!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::LocalFs {
            base_dir: PathBuf::from("/tmp/fabushi-box-store"),
            store_id_override: None,
        }
    );

    let host_extensions = include_str!("../src/host_production_extensions.rs");
    assert!(host_extensions.contains("get_or_create_box_store()"));
    assert!(host_extensions.contains("resolve_store_id:"));
}

#[test]
fn production_mode_refuses_unwired_remote_backends_instead_of_downgrading() {
    let env = BTreeMap::from([
        ("SAND_BOX_STORE_SYNC".to_string(), "1".to_string()),
        ("SAND_BOX_STORE_BACKEND".to_string(), "v2".to_string()),
    ]);
    assert!(matches!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::UnsupportedRemote { .. }
    ));

    let env = BTreeMap::from([
        ("SAND_BOX_STORE_SYNC".to_string(), "1".to_string()),
    ]);
    assert!(matches!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::UnsupportedRemote { .. }
    ));
}

#[test]
fn shipping_host_starts_box_store_sync_after_mcp_and_stops_it_before_mcp() {
    let host_extensions = include_str!("../src/host_production_extensions.rs");
    assert!(host_extensions.contains("HostExtensionId::BoxStoreSync"));
    assert!(host_extensions.contains("pub fn start_box_store_sync("));
    assert!(host_extensions.contains("production BoxStoreSync requires MCP to be started first"));

    let shipping_main = include_str!("../app/src/main.rs");
    let start_mcp = shipping_main
        .find("production_extensions.start_mcp(")
        .expect("shipping MCP start");
    let start_box_store = shipping_main
        .find("production_extensions.start_box_store_sync(")
        .expect("shipping BoxStoreSync start");
    assert!(
        start_mcp < start_box_store,
        "BoxStoreSync must start only after its frozen MCP dependency"
    );

    let stop_box_store = shipping_main
        .find("production_extensions.stop_box_store_sync()")
        .expect("shipping BoxStoreSync stop");
    let stop_mcp = shipping_main
        .find("production_extensions.stop_mcp()")
        .expect("shipping MCP stop");
    assert!(
        stop_box_store < stop_mcp,
        "BoxStoreSync must stop before MCP so it cannot outlive its dependency"
    );
}

#[test]
fn production_owner_wires_real_chrome_watcher_and_periodic_cycle() {
    let production = include_str!("../src/extensions/box_store_sync/production.rs");
    assert!(production.contains("ChromeSessionWatcher::with_logger"));
    assert!(production.contains("run_local_cycle(true, false, false)"));
    assert!(production.contains("run_local_cycle(false, include_store_dbs, true)"));
    assert!(production.contains("stage_box_chrome_session()"));
    assert!(production.contains("sync_store_db_snapshots("));
    assert!(production.contains("StoreDbSnapshotUpload::new("));
    assert!(production.contains("run_vacuum_off_thread(&source_path, &temp_path)"));
    assert!(production.contains("write_manifest("));
    assert!(production.contains("store.put_from_file(&blob_key, path)"));
    assert!(production.contains("remote-backend-not-wired"));
    assert!(production.contains("has_live_sand_agent_db_handle"));
    assert!(production.contains("run_store_db_debounce_loop"));
    assert!(production.contains("AgentDbCaptureQueues"));
    assert!(production.contains("get_sand_agent_db_write_generation"));

    let shipping_main = include_str!("../app/src/main.rs");
    assert!(shipping_main.contains("schedule_store_db_snapshot(&agent_id)"));
    let retire = shipping_main
        .find("retire_idle_live_session(&worker_retire_sessions, &agent_id)")
        .expect("live Agent DB session retirement");
    let schedule = shipping_main
        .find("schedule_store_db_snapshot(&agent_id)")
        .expect("turn-end store.db scheduling");
    assert!(
        retire < schedule,
        "turn-end capture must be scheduled only after the live Agent DB owner retires"
    );
}
