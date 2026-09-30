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
    assert!(host_extensions.contains("backend_url: Some(self.backend_url.clone())"));
    assert!(host_extensions.contains("get_access_token: Some("));
    assert!(host_extensions.contains("get_machine_id: Some("));
}

#[test]
fn production_mode_wires_agent_store_and_sand_box_store_v2() {
    let env = BTreeMap::from([
        ("SAND_BOX_STORE_SYNC".to_string(), "1".to_string()),
        ("SAND_BOX_STORE_BACKEND".to_string(), "v2".to_string()),
    ]);
    assert_eq!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::SandBoxStoreV2
    );

    let env = BTreeMap::from([
        ("SAND_BOX_STORE_SYNC".to_string(), "1".to_string()),
    ]);
    assert_eq!(
        resolve_production_box_store_sync_mode(&env),
        ProductionBoxStoreSyncMode::AgentStore
    );
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
fn production_owner_wires_real_chrome_watcher_periodic_cycle_and_remote_providers() {
    let production = include_str!("../src/extensions/box_store_sync/production.rs");
    assert!(production.contains("ChromeSessionWatcher::with_logger"));
    assert!(production.contains("run_local_cycle(true, false, false, false, false, true, false)"));
    assert!(production.contains("plan_periodic_cycle("));
    assert!(production.contains("plan.include_store_dbs"));
    assert!(production.contains("plan.skip_live_handle_store_dbs"));
    assert!(production.contains("plan.include_idle_only"));
    assert!(production.contains("stage_box_chrome_session()"));
    assert!(production.contains("sync_store_db_snapshots("));
    assert!(production.contains("StoreDbSnapshotUpload::new("));
    assert!(production.contains("run_vacuum_off_thread(&source_path, &temp_path)"));
    assert!(production.contains("write_manifest("));
    assert!(production.contains("store.put_from_file(&blob_key, path)"));
    assert!(production.contains("ProductionBoxStoreSyncMode::AgentStore"));
    assert!(production.contains("ProductionBoxStoreSyncMode::SandBoxStoreV2"));
    assert!(production.contains("resolve_box_object_store_provider("));
    assert!(production.contains("BoxObjectStoreProviderDependencies {"));
    assert!(production.contains("has_live_sand_agent_db_handle"));
    assert!(production.contains("run_store_db_debounce_loop"));
    assert!(production.contains("AgentDbCaptureQueues"));
    assert!(production.contains("get_sand_agent_db_write_generation"));
    assert!(production.contains("sync_local_packs"));
    assert!(production.contains("PACK_TMP_DIR_NAME"));
    assert!(production.contains("BOX_STORE_PACK_RETIRED_KEY"));
    assert!(production.contains("BOX_STORE_PACK_INDEX_KEY"));
    assert!(production.contains("plan.include_packs"));

    let provider_owner = include_str!("../src/extensions/box_store_sync/box_object_store.rs");
    assert!(provider_owner.contains("BoxStoreBackendKind::AgentStore =>"));
    assert!(provider_owner.contains("AgentStoreObjectStoreProvider::new(remote)?"));
    assert!(provider_owner.contains("AgentStoreClient::new(deps)?"));
    assert!(provider_owner.contains("client.get_object(&self.source_id, key)?"));
    let agent_store_impl = provider_owner
        .split_once("impl BoxObjectStore for AgentStoreObjectStore {")
        .map(|(_, body)| body)
        .expect("AgentStoreObjectStore production BoxObjectStore implementation");
    let agent_store_put = agent_store_impl
        .split_once("fn put(&self, key: &str, bytes: &[u8])")
        .map(|(_, body)| body)
        .and_then(|body| body.split_once("fn put_if_unchanged(").map(|(put, _)| put))
        .expect("AgentStoreObjectStore production put method");
    let agent_store_put_compact: String = agent_store_put
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        agent_store_put_compact
            .contains(".client.put_bytes(&self.source_id,key,bytes,precondition)?;"),
        "AgentStoreObjectStore::put must call AgentStoreClient::put_bytes with the resolved precondition"
    );
    let agent_store_put_file = agent_store_impl
        .split_once("fn put_from_file(&self, key: &str, src_path: &Path)")
        .map(|(_, body)| body)
        .and_then(|body| body.split_once("fn list(&self, prefix: &str)").map(|(put_file, _)| put_file))
        .expect("AgentStoreObjectStore production put_from_file method");
    let agent_store_put_file_compact: String = agent_store_put_file
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        agent_store_put_file_compact
            .contains(".client.put_file_content_addressed(&self.source_id,key,src_path)?"),
        "AgentStoreObjectStore::put_from_file must use the content-addressed AgentStore upload path"
    );

    let agent_store_list = agent_store_impl
        .split_once("fn list(&self, prefix: &str)")
        .map(|(_, body)| body)
        .and_then(|body| body.split_once("fn delete(&self, key: &str)").map(|(list, _)| list))
        .expect("AgentStoreObjectStore production list method");
    let agent_store_list_compact: String = agent_store_list
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect();
    assert!(
        agent_store_list_compact.contains(".client.list_objects(&self.source_id,prefix)"),
        "AgentStoreObjectStore::list must delegate to AgentStoreClient::list_objects"
    );

    let agent_store = include_str!("../src/extensions/box_store_sync/agent_store_sand_files.rs");
    assert!(agent_store.contains("fn token_for(&self, source_id: &str)"));
    assert!(agent_store.contains("PRESIGN_AGENT_STORE_READS_PATH"));
    assert!(agent_store.contains("PRESIGN_AGENT_STORE_WRITES_PATH"));
    assert!(agent_store.contains("validate_presigned_url(&self.inner.deps.backend_url"));
    assert!(agent_store.contains("if response.status().as_u16() == 409"));
    assert!(agent_store.contains("status == 412 || status == 409"));
    assert!(agent_store.contains("pub fn get_object_to_file("));
    assert!(agent_store.contains("pub fn put_file_content_addressed("));
    assert!(agent_store.contains("pub fn presign_read_batch("));
    assert!(agent_store.contains("pub fn list_objects("));
    assert!(provider_owner.contains("BoxStoreBackendKind::SandBoxStoreV2 =>"));
    assert!(provider_owner.contains("SandBoxStoreServiceProvider::new(client)?"));

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


#[test]
fn production_manifest_owner_consumes_frozen_retry_hydration_and_conflict_contract() {
    let production = include_str!("../src/extensions/box_store_sync/production.rs");
    assert!(production.contains("load_manifest_for_write("));
    assert!(production.contains("write_manifest_with_retry("));
    assert!(production.contains("fully_hydrated"));
    assert!(production.contains("BOX_STORE_HYDRATION_HANDOFF_FILE_NAME"));
    assert!(production.contains("BOX_STORE_MANIFEST_RETRY_ATTEMPTS"));
    assert!(production.contains("BOX_STORE_MANIFEST_RETRY_DELAY_MS"));
    assert!(
        !production.contains("fn serialize_manifest("),
        "shipping production must not retain a second manifest serializer"
    );

    let manifest = include_str!("../src/extensions/box_store_sync/box_store_manifest.rs");
    for expected in [
        "BoxStoreManifestParseError::UnsupportedVersion",
        "BoxStoreManifestParseError::PathConflict",
        "configured_manifest_entries(",
        "load_manifest_for_write(",
        "write_manifest_with_retry(",
        "accept_matching_canonical_on_conflict",
        "manifest_write_conflict",
        "mark_legacy_hydration_incomplete(",
        "mark_legacy_hydration_complete_for_handoff(",
        "prepare_canonical_manifest_reset(",
    ] {
        assert!(
            manifest.contains(expected),
            "canonical manifest owner must retain frozen behavior marker {expected}"
        );
    }

    let object_store = include_str!("../src/extensions/box_store_sync/box_object_store.rs");
    assert!(object_store.contains("put_if_unchanged_detailed("));
    assert!(object_store.contains("conflict_rel_path"));
    assert!(object_store.contains("base_etag.or(fallback_base_etag)"));
    assert!(object_store.contains("agent-store-etag"));
}


#[test]
fn shipping_box_store_sync_core_writer_lock_is_consumed_by_every_mutating_owner() {
    let core = include_str!("../src/extensions/box_store_sync/box_store_sync.rs");
    for expected in [
        "pub struct BoxStoreWriterLock",
        "FileExt::try_lock_exclusive(&file)",
        "BOX_STORE_WRITER_LOCK_FILE_NAME",
        "assert_no_symlink_in_path(lock_path)",
        "FileExt::unlock(&self.file)",
    ] {
        assert!(
            core.contains(expected),
            "box-store-sync core must retain writer-lock marker {expected}"
        );
    }

    let production = include_str!("../src/extensions/box_store_sync/production.rs");
    for expected in [
        "writer_lock: Mutex<Option<BoxStoreWriterLock>>",
        "fn ensure_writer_lock(&self) -> Result<bool, String>",
        "BoxStoreWriterLock::try_acquire(&lock_path, &window_id)?",
        "self.inner.release_writer_lock();",
    ] {
        assert!(
            production.contains(expected),
            "shipping ProductionBoxStoreSync owner must consume core writer-lock marker {expected}"
        );
    }
    assert!(
        production.matches("if !self.ensure_writer_lock()?").count() >= 4,
        "snapshot, turn-end capture, clear, and forget must all fence mutation through the long-lived writer lock"
    );

    let clear = production
        .split_once("fn clear_store_now(&self) -> Result<(), String>")
        .map(|(_, body)| body)
        .and_then(|body| body.split_once("fn reset_canonical_store(").map(|(clear, _)| clear))
        .expect("clear_store_now production body");
    let acquire = clear
        .find("if !self.ensure_writer_lock()?")
        .expect("clear writer-lock acquire");
    let legacy_reset = clear
        .find("self.reset_canonical_store(&store_id, legacy.as_ref())")
        .expect("legacy canonical reset");
    assert!(
        acquire < legacy_reset,
        "SandBoxStoreV2 legacy reset must never write before the canonical writer lock is held"
    );
}
