#[test]
fn shipping_box_copy_in_uses_real_host_producer_canonical_mapper_and_unique_owner() {
    let producer = include_str!("../src/extensions/box_store_sync/box_copy_in.rs");
    assert!(producer.contains("pub fn execute_production_box_copy_in_from_env("));
    assert!(producer.contains("HostTelemetryService::open_with_identity_tags("));
    assert!(producer.contains(r#"telemetry_identity_tags.insert("store_backend".into(), store_backend.into())"#));
    assert!(producer.contains("build_copy_in_started_telemetry()"));
    assert!(producer.contains("build_copy_in_result_telemetry(result, duration_ms)"));
    assert!(producer.contains("telemetry.report_box_copy_in(event.level, event.metadata)"));
    assert!(producer.contains("build_copy_in_watchdog_event("));
    assert!(
        !producer.contains("fn telemetry_projection("),
        "copy-in producer must not own a second structured-log projection"
    );
    assert!(
        !producer.contains("report_projection(&telemetry_projection"),
        "copy-in producer must route through the typed Host facade"
    );

    let owner = include_str!("../src/extensions/telemetry/host_telemetry_service.rs");
    assert!(owner.contains("pub fn report_box_copy_in("));
    assert!(owner.contains("self.report_projection(&box_copy_in_telemetry(level, metadata))"));

    let mapper = include_str!("../src/extensions/telemetry/box_copy_in_telemetry.rs");
    assert_eq!(
        mapper.matches("sand.box_copy_in").count(),
        1,
        "canonical mapper must be the only fixed event literal in its owner"
    );
    assert!(mapper.contains("event: Some(BOX_COPY_IN_EVENT)"));

    let coordinator = include_str!("../../node-agent-coordinator/src/main.rs");
    assert!(!coordinator.contains("reportBoxCopyIn"));
    assert!(!coordinator.contains("sand.box_copy_in"));

    let electron = include_str!("../../electron-main/telemetry/desktop-structured-log-telemetry.ts");
    assert!(!electron.contains("reportBoxCopyIn"));
    assert!(!electron.contains("sand.box_copy_in"));
}

#[test]
fn shipping_copy_in_preserves_frozen_started_watchdog_terminal_and_failure_level_decisions() {
    let producer = include_str!("../src/extensions/box_store_sync/box_copy_in.rs");
    for needle in [
        r#"level: "info""#,
        r#"("outcome".into(), "started".into())"#,
        r#"("reason".into(), "hydrate-started".into())"#,
        r#"level: if progressed_since_last_tick { "info" } else { "warn" }"#,
        r#""moving-data-in-slow-but-advancing""#,
        r#""moving-data-in-exceeded-threshold""#,
        r#"if result.outcome == CopyInOutcome::Failed"#,
        r#""error""#,
        r#""info""#,
        r#""manifest_entries".into()"#,
        r#""store_db_entries".into()"#,
        r#""error_class".into()"#,
        r#""error_summary".into()"#,
        r#""duration_ms".into()"#,
    ] {
        assert!(producer.contains(needle), "missing frozen copy-in producer semantic: {needle}");
    }
    assert!(
        !producer.contains(r#"("hydrate_source".into(), result.hydrate_source.clone().unwrap_or_default())"#),
        "undefined frozen hydrate_source must be omitted rather than serialized as an empty string"
    );
    assert!(
        !producer.contains(r#"("store_backend".into(), store_backend.to_string())"#),
        "store_backend belongs to structured-log identity tags, not each copy-in event payload"
    );
}

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::SystemTime;

use sha2::{Digest, Sha256};

use mahayana_host_runtime::durable_file_policy::HOST_CONFIG_REVISION_KEY_FILE_NAME;
use mahayana_host_runtime::extensions::box_store_sync::box_copy_in::{
    CopyInOutcome, CopyInRuntimeOptions, run_box_copy_in_with_sources,
};
use mahayana_host_runtime::extensions::box_store_sync::box_object_store::{
    BoxObjectStore, LocalFsObjectStore,
};
use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest_format::BOX_STORE_MANIFEST_REL_PATH;
use mahayana_host_runtime::host_paths::{SAND_DATA_ROOT_ENV, get_sand_root_dir_with};

struct CopyInBoundaryFixture {
    root: PathBuf,
    target: PathBuf,
    actual: PathBuf,
}

impl CopyInBoundaryFixture {
    fn new() -> Self {
        let root = std::env::temp_dir().join(format!("fabushi-copy-in-key-{}", uuid::Uuid::new_v4().simple()));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&root).expect("create private copy-in fixture");
        let target = root.join("target");
        let actual = target.join("workspace/private-state");
        let environment = BTreeMap::from([(
            SAND_DATA_ROOT_ENV.to_string(), actual.to_string_lossy().into_owned(),
        )]);
        let actual = get_sand_root_dir_with(&root.join("home"), &[], &environment, &root);
        let portable = target.join("home/box/sand-data");
        for installation in [&actual, &portable] {
            fs::create_dir_all(installation).expect("create copy-in installation");
            for (name, byte) in [
                (HOST_CONFIG_REVISION_KEY_FILE_NAME.to_string(), 0x31_u8),
                (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"), 0x54_u8),
            ] {
                let path = installation.join(name);
                fs::write(&path, [byte; 32]).expect("write synthetic installation material");
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    fs::set_permissions(path, fs::Permissions::from_mode(0o600)).expect("set fixture key mode");
                }
            }
        }
        Self { root, target, actual }
    }
}

impl Drop for CopyInBoundaryFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(PartialEq, Eq)]
struct CopyInPathState {
    directory: bool,
    contents: Vec<u8>,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    mode: u32,
    #[cfg(unix)]
    inode: u64,
}

fn copy_in_tree_state(root: &Path) -> BTreeMap<PathBuf, CopyInPathState> {
    fn visit(root: &Path, path: &Path, result: &mut BTreeMap<PathBuf, CopyInPathState>) {
        let metadata = fs::symlink_metadata(path).expect("read owned copy-in metadata");
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        result.insert(path.strip_prefix(root).expect("owned copy-in path").to_path_buf(), CopyInPathState {
            directory: metadata.is_dir(),
            contents: if metadata.is_file() { fs::read(path).expect("read owned copy-in file") } else { Vec::new() },
            modified: metadata.modified().ok(),
            #[cfg(unix)]
            mode: metadata.mode(),
            #[cfg(unix)]
            inode: metadata.ino(),
        });
        if metadata.is_dir() {
            for entry in fs::read_dir(path).expect("read owned copy-in directory") {
                visit(root, &entry.expect("owned directory entry").path(), result);
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}

struct CopyInBoundaryStore {
    inner: LocalFsObjectStore,
    calls: Mutex<Vec<String>>,
}

impl CopyInBoundaryStore {
    fn record(&self, operation: &str, key: &str) {
        self.calls.lock().expect("store calls").push(format!("{operation}:{key}"));
    }
}

impl BoxObjectStore for CopyInBoundaryStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        self.record("get", key);
        self.inner.get(key)
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<(), String> {
        self.record("put", key);
        self.inner.put(key, bytes)
    }

    fn get_to_file(&self, key: &str, destination: &Path, max_bytes: Option<u64>) -> Result<Option<u64>, String> {
        self.record("get_to_file", key);
        self.inner.get_to_file(key, destination, max_bytes)
    }

    fn prefetch_reads(&self, keys: &[String]) -> Result<(), String> {
        self.record("prefetch_reads", "");
        self.inner.prefetch_reads(keys)
    }

    fn put_from_file(&self, key: &str, source: &Path) -> Result<(), String> {
        self.record("put_from_file", key);
        self.inner.put_from_file(key, source)
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, String> {
        self.record("list", prefix);
        self.inner.list(prefix)
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        self.record("delete", key);
        self.inner.delete(key)
    }
}

fn copy_in_boundary_store(
    fixture: &CopyInBoundaryFixture,
    paths: &[String],
    payload: &[u8],
) -> CopyInBoundaryStore {
    let inner = LocalFsObjectStore::new(fixture.root.join("store"));
    let sha = format!("{:x}", Sha256::digest(payload));
    inner.put(&format!("blobs/{sha}"), payload).expect("seed synthetic object");
    let entries = paths.iter().map(|path| (path.clone(), serde_json::json!({
        "kind": "file", "sha": sha.clone(), "size": payload.len(), "mode": 0o644,
    }))).collect::<serde_json::Map<_, _>>();
    let manifest = serde_json::json!({
        "version": 2, "updatedAtMs": 1, "fullyHydrated": true, "entries": entries,
    });
    inner.put(BOX_STORE_MANIFEST_REL_PATH, &serde_json::to_vec(&manifest).expect("encode fixture manifest"))
        .expect("seed fixture manifest");
    CopyInBoundaryStore { inner, calls: Mutex::new(Vec::new()) }
}

fn copy_in_boundary_options(actual: &Path, effects: &Arc<AtomicUsize>) -> CopyInRuntimeOptions {
    let progress = Arc::clone(effects);
    let trace = Arc::clone(effects);
    CopyInRuntimeOptions {
        installation_data_root: Some(actual.to_path_buf()),
        on_progress: Some(Arc::new(move |_| { progress.fetch_add(1, Ordering::SeqCst); })),
        on_trace: Some(Arc::new(move |_| { trace.fetch_add(1, Ordering::SeqCst); })),
        ..CopyInRuntimeOptions::default()
    }
}

#[test]
fn production_copy_in_propagates_actual_installation_root_before_payload_or_mutation() {
    for (relative, byte) in [
        (format!("workspace/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"), 0x72_u8),
        (format!("workspace/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"), 0x72_u8),
        ("workspace/private-state".to_string(), 0x72_u8),
        ("workspace/key-native-alias.bin".to_string(), 0x31_u8),
        ("workspace/stage-native-alias.bin".to_string(), 0x54_u8),
        (format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"), 0x72_u8),
    ] {
        let fixture = CopyInBoundaryFixture::new();
        fs::hard_link(
            fixture.actual.join(HOST_CONFIG_REVISION_KEY_FILE_NAME),
            fixture.target.join("workspace/key-native-alias.bin"),
        ).expect("create copy-in key alias");
        fs::hard_link(
            fixture.actual.join(format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp")),
            fixture.target.join("workspace/stage-native-alias.bin"),
        ).expect("create copy-in staging alias");
        let store = copy_in_boundary_store(&fixture, &[relative, "workspace/a-control.txt".into()], &[byte; 32]);
        let before = copy_in_tree_state(&fixture.target);
        let effects = Arc::new(AtomicUsize::new(0));
        let result = run_box_copy_in_with_sources(
            &store, None, &fixture.target, None, copy_in_boundary_options(&fixture.actual, &effects),
        );
        assert_eq!(result.outcome, CopyInOutcome::Failed);
        assert_eq!((result.files, result.bytes, result.verified), (0, 0, 0));
        assert_eq!(result.failures, vec!["manifest may replace installation revision key material".to_string()]);
        assert_eq!(*store.calls.lock().expect("store calls"), vec![format!("get:{BOX_STORE_MANIFEST_REL_PATH}")],
            "copy-in did work beyond its authorized manifest acquisition before refusal");
        assert_eq!(effects.load(Ordering::SeqCst), 0, "copy-in refusal invoked restore callbacks");
        assert!(copy_in_tree_state(&fixture.target) == before, "copy-in refusal changed bytes, metadata or namespace");
    }
}

#[test]
fn production_copy_in_preserves_actual_key_and_restores_neighbors_with_matching_names() {
    let fixture = CopyInBoundaryFixture::new();
    let paths = [
        "workspace/private-state/settings.json".to_string(),
        "home/box/sand-data/settings.json".to_string(),
        format!("workspace/unrelated/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        format!("workspace/unrelated/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"),
    ];
    let payload = b"ordinary copy-in content";
    let store = copy_in_boundary_store(&fixture, &paths, payload);
    let before = copy_in_tree_state(&fixture.target);
    let effects = Arc::new(AtomicUsize::new(0));
    let result = run_box_copy_in_with_sources(
        &store, None, &fixture.target, None, copy_in_boundary_options(&fixture.actual, &effects),
    );
    assert_eq!(result.outcome, CopyInOutcome::Hydrated);
    assert_eq!((result.files, result.verified), (paths.len(), paths.len()));
    assert!(result.failures.is_empty(), "ordinary production copy-in failed");
    assert!(effects.load(Ordering::SeqCst) > 0, "ordinary copy-in invoked no callbacks");
    assert!(store.calls.lock().expect("store calls").iter().any(|call| call.starts_with("get:blobs/")),
        "ordinary copy-in did not read its payload");
    for path in paths {
        assert!(fs::read(fixture.target.join(path)).expect("read copied ordinary file") == payload,
            "ordinary copy-in changed payload");
    }
    let after = copy_in_tree_state(&fixture.target);
    for prefix in ["workspace/private-state", "home/box/sand-data"] {
        for name in [HOST_CONFIG_REVISION_KEY_FILE_NAME.to_string(), format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp")] {
            let path = Path::new(prefix).join(name);
            assert!(before.get(&path) == after.get(&path), "ordinary copy-in changed protected installation material");
        }
    }
}

#[test]
fn shipping_copy_in_binds_captured_path_owner_root_to_download_options() {
    let source = include_str!("../src/extensions/box_store_sync/box_copy_in.rs");
    let production = source.split_once("pub fn execute_production_box_copy_in_from_env(")
        .expect("production copy-in owner").1
        .split_once("pub fn execute_box_copy_in_from_env_with_provider_dependencies(")
        .expect("production copy-in boundary").0;
    assert!(production.contains("let sand_root = get_sand_root_dir();"));
    assert!(production.contains("installation_data_root: Some(sand_root.clone())"));
    let restore = source.split_once("fn restore_manifest(").expect("existing restore owner").1
        .split_once("pub fn run_box_copy_in_with_sources(").expect("restore caller boundary").0;
    assert!(restore.contains("installation_data_root: options.installation_data_root.clone()"));
    let shipping = include_str!("../app/src/main.rs");
    assert!(shipping.contains("execute_production_box_copy_in_from_env("));
}
