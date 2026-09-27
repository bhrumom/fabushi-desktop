use std::collections::VecDeque;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest_format::BoxStoreManifestEntry;
use mahayana_host_runtime::extensions::box_store_sync::store_db_snapshot_upload::{
    BoxManifestMap, SnapshotFailurePhase, SnapshotUploadOutcome,
    StoreDbSnapshotRuntimePort, StoreDbSnapshotUpload, sha256_bytes,
};

#[derive(Default)]
struct FakeRuntime {
    byte_puts: Mutex<Vec<(String, String, Vec<u8>, bool)>>,
    file_puts: Mutex<Vec<(String, String, PathBuf, String, u64)>>,
    fail_upload: Mutex<Option<String>>,
}

impl StoreDbSnapshotRuntimePort for FakeRuntime {
    fn put_bytes(
        &self,
        store_id: &str,
        key: &str,
        bytes: &[u8],
        content_addressed: bool,
    ) -> Result<(), String> {
        if let Some(error) = self.fail_upload.lock().unwrap().clone() {
            return Err(error);
        }
        self.byte_puts.lock().unwrap().push((
            store_id.to_string(),
            key.to_string(),
            bytes.to_vec(),
            content_addressed,
        ));
        Ok(())
    }

    fn put_from_file(
        &self,
        store_id: &str,
        key: &str,
        path: &Path,
        sha: &str,
        size: u64,
    ) -> Result<(), String> {
        if let Some(error) = self.fail_upload.lock().unwrap().clone() {
            return Err(error);
        }
        self.file_puts.lock().unwrap().push((
            store_id.to_string(),
            key.to_string(),
            path.to_path_buf(),
            sha.to_string(),
            size,
        ));
        Ok(())
    }
}

fn temp_file(name: &str, bytes: &[u8]) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "fabushi-store-db-snapshot-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&root).unwrap();
    let path = root.join(name);
    fs::write(&path, bytes).unwrap();
    path
}

fn service(
    runtime: Arc<FakeRuntime>,
    max: u64,
    threshold: u64,
    times: Vec<u64>,
    logs: Arc<Mutex<Vec<String>>>,
) -> StoreDbSnapshotUpload {
    let times = Arc::new(Mutex::new(VecDeque::from(times)));
    StoreDbSnapshotUpload::new(
        runtime,
        {
            let times = Arc::clone(&times);
            Arc::new(move || times.lock().unwrap().pop_front().unwrap_or(0))
        },
        Arc::new(move |message| logs.lock().unwrap().push(message.to_string())),
        max,
        threshold,
    )
}

#[test]
fn small_snapshot_uploads_content_addressed_bytes_and_updates_manifest() {
    let runtime = Arc::new(FakeRuntime::default());
    let logs = Arc::new(Mutex::new(Vec::new()));
    let service = service(runtime.clone(), 1024, 100, vec![10, 25], logs);
    let path = temp_file("store.db", b"hello");
    let sha = sha256_bytes(b"hello");
    let mut manifest = BoxManifestMap::new();

    let result = service.upload_agent_db_snapshot(
        "store-1",
        &mut manifest,
        "agents/a/store.db",
        &path,
        0o100644,
    );

    assert_eq!(result.outcome, SnapshotUploadOutcome::Uploaded);
    assert_eq!(result.bytes_uploaded, 5);
    assert_eq!(result.blob_upload_duration_ms, 15);
    let puts = runtime.byte_puts.lock().unwrap();
    assert_eq!(puts.len(), 1);
    assert_eq!(puts[0].0, "store-1");
    assert_eq!(puts[0].1, format!("blobs/{sha}"));
    assert_eq!(puts[0].2, b"hello");
    assert!(puts[0].3);
    assert_eq!(
        manifest.get("agents/a/store.db"),
        Some(&BoxStoreManifestEntry::File {
            sha,
            size: 5,
            mode: 0o644,
        })
    );

    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn unchanged_snapshot_skips_upload_but_refreshes_mode_entry() {
    let runtime = Arc::new(FakeRuntime::default());
    let logs = Arc::new(Mutex::new(Vec::new()));
    let service = service(runtime.clone(), 1024, 100, vec![], logs);
    let path = temp_file("store.db", b"same");
    let sha = sha256_bytes(b"same");
    let mut manifest = BoxManifestMap::from([(
        "store.db".into(),
        BoxStoreManifestEntry::File {
            sha: sha.clone(),
            size: 4,
            mode: 0o600,
        },
    )]);

    let result =
        service.upload_agent_db_snapshot("s", &mut manifest, "store.db", &path, 0o100755);

    assert_eq!(result.outcome, SnapshotUploadOutcome::Unchanged);
    assert!(runtime.byte_puts.lock().unwrap().is_empty());
    assert!(runtime.file_puts.lock().unwrap().is_empty());
    assert_eq!(
        manifest.get("store.db"),
        Some(&BoxStoreManifestEntry::File {
            sha,
            size: 4,
            mode: 0o755,
        })
    );

    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn large_snapshot_streams_from_file_and_oversize_is_capture_failure() {
    let runtime = Arc::new(FakeRuntime::default());
    let logs = Arc::new(Mutex::new(Vec::new()));
    let path = temp_file("large.db", b"0123456789");
    let uploader = service(runtime.clone(), 20, 4, vec![100, 103], Arc::clone(&logs));
    let mut manifest = BoxManifestMap::new();

    let result =
        uploader.upload_agent_db_snapshot("s", &mut manifest, "large.db", &path, 0o600);
    assert_eq!(result.outcome, SnapshotUploadOutcome::Uploaded);
    assert_eq!(result.blob_upload_duration_ms, 3);
    assert_eq!(runtime.file_puts.lock().unwrap().len(), 1);
    assert!(runtime.byte_puts.lock().unwrap().is_empty());

    let oversize = service(runtime.clone(), 5, 4, vec![], Arc::clone(&logs))
        .upload_agent_db_snapshot("s", &mut manifest, "large.db", &path, 0o600);
    assert_eq!(oversize.outcome, SnapshotUploadOutcome::Oversize);
    assert_eq!(oversize.failure_phase, Some(SnapshotFailurePhase::Capture));
    assert!(logs.lock().unwrap().iter().any(|line| line.contains("oversize large.db")));

    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn upload_failure_is_blob_phase_and_does_not_mutate_manifest() {
    let runtime = Arc::new(FakeRuntime::default());
    *runtime.fail_upload.lock().unwrap() = Some("backend down".into());
    let logs = Arc::new(Mutex::new(Vec::new()));
    let service = service(runtime, 1024, 100, vec![20, 37], Arc::clone(&logs));
    let path = temp_file("store.db", b"payload");
    let mut manifest = BoxManifestMap::new();

    let result =
        service.upload_agent_db_snapshot("s", &mut manifest, "store.db", &path, 0o600);
    assert_eq!(result.outcome, SnapshotUploadOutcome::Error);
    assert_eq!(result.failure_phase, Some(SnapshotFailurePhase::BlobUpload));
    assert_eq!(result.blob_upload_duration_ms, 17);
    assert!(manifest.is_empty());
    assert!(logs.lock().unwrap().iter().any(|line| line.contains("backend down")));

    let _ = fs::remove_dir_all(path.parent().unwrap());
}

#[test]
fn native_vacuum_runs_off_thread_and_temp_cleanup_is_idempotent() {
    let runtime = Arc::new(FakeRuntime::default());
    let logs = Arc::new(Mutex::new(Vec::new()));
    let service = service(runtime, 1024, 100, vec![], logs);
    let root = std::env::temp_dir().join(format!(
        "fabushi-vacuum-{}",
        uuid::Uuid::new_v4()
    ));
    fs::create_dir_all(&root).unwrap();
    let src = root.join("src.db");
    let dest = root.join("dest.db");
    {
        let db = rusqlite::Connection::open(&src).unwrap();
        db.execute_batch("CREATE TABLE t(v TEXT); INSERT INTO t VALUES ('ok');")
            .unwrap();
    }

    service.run_vacuum_off_thread(&src, &dest).unwrap();
    let db = rusqlite::Connection::open(&dest).unwrap();
    let value: String = db.query_row("SELECT v FROM t", [], |row| row.get(0)).unwrap();
    assert_eq!(value, "ok");
    drop(db);

    service.discard_snapshot_temp(&dest, "test");
    assert!(!dest.exists());
    service.discard_snapshot_temp(&dest, "test");

    let _ = fs::remove_dir_all(root);
}
