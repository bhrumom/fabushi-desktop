use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use mahayana_host_runtime::durable_file_policy::HOST_CONFIG_REVISION_KEY_FILE_NAME;
use mahayana_host_runtime::extensions::box_store_sync::box_object_store::BoxObjectStore;
use mahayana_host_runtime::extensions::box_store_sync::box_store_download::{
    BoxStoreByteBudget, BoxStoreDownloadOptions, BoxStoreDownloadSummary, BoxStoreDownloadTrace, download_manifest,
    symlink_target_stays_within_root,
};
use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest_format::BoxStoreManifestEntry;
use mahayana_host_runtime::extensions::box_store_sync::box_store_pack::{
    BOX_STORE_PACK_INDEX_KEY, BOX_STORE_PACKS_PREFIX, PACK_INDEX_VERSION, PackEntry, PackIndex,
    PackSource, build_pack_file, serialize_pack_index, sha256_hex,
};

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-box-store-download-{label}-{}-{nonce}",
        std::process::id()
    ))
}

#[derive(Default)]
struct CountingStore {
    objects: Mutex<HashMap<String, Vec<u8>>>,
    gets: Mutex<Vec<String>>,
    file_gets: Mutex<Vec<String>>,
    prefetches: Mutex<Vec<Vec<String>>>,
    calls: Mutex<Vec<&'static str>>,
    fail_prefetch: bool,
}

impl CountingStore {
    fn with_objects(objects: impl IntoIterator<Item = (String, Vec<u8>)>) -> Self {
        Self {
            objects: Mutex::new(objects.into_iter().collect()),
            ..Self::default()
        }
    }

    fn with_prefetch_failure(objects: impl IntoIterator<Item = (String, Vec<u8>)>) -> Self {
        Self {
            objects: Mutex::new(objects.into_iter().collect()),
            fail_prefetch: true,
            ..Self::default()
        }
    }

    fn get_count(&self, key: &str) -> usize {
        self.gets
            .lock()
            .expect("gets")
            .iter()
            .filter(|candidate| candidate.as_str() == key)
            .count()
    }

    fn file_get_count(&self, key: &str) -> usize {
        self.file_gets
            .lock()
            .expect("file gets")
            .iter()
            .filter(|candidate| candidate.as_str() == key)
            .count()
    }

    fn get_order(&self) -> Vec<String> {
        self.gets.lock().expect("gets").clone()
    }
}

impl BoxObjectStore for CountingStore {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>, String> {
        self.calls.lock().expect("calls").push("get");
        self.gets.lock().expect("gets").push(key.to_string());
        Ok(self.objects.lock().expect("objects").get(key).cloned())
    }

    fn put(&self, key: &str, bytes: &[u8]) -> Result<(), String> {
        self.calls.lock().expect("calls").push("put");
        self.objects
            .lock()
            .expect("objects")
            .insert(key.to_string(), bytes.to_vec());
        Ok(())
    }

    fn get_to_file(
        &self,
        key: &str,
        dest_path: &Path,
        max_bytes: Option<u64>,
    ) -> Result<Option<u64>, String> {
        self.calls.lock().expect("calls").push("get_to_file");
        self.file_gets
            .lock()
            .expect("file gets")
            .push(key.to_string());
        let Some(bytes) = self.objects.lock().expect("objects").get(key).cloned() else {
            return Ok(None);
        };
        if max_bytes.is_some_and(|max| bytes.len() as u64 > max) {
            return Err("object exceeds maximum restore bytes".into());
        }
        if let Some(parent) = dest_path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        fs::write(dest_path, &bytes).map_err(|error| error.to_string())?;
        Ok(Some(bytes.len() as u64))
    }

    fn prefetch_reads(&self, keys: &[String]) -> Result<(), String> {
        self.calls.lock().expect("calls").push("prefetch_reads");
        self.prefetches
            .lock()
            .expect("prefetches")
            .push(keys.to_vec());
        if self.fail_prefetch {
            Err("synthetic prefetch failure".into())
        } else {
            Ok(())
        }
    }

    fn put_from_file(&self, key: &str, src_path: &Path) -> Result<(), String> {
        self.calls.lock().expect("calls").push("put_from_file");
        let bytes = fs::read(src_path).map_err(|error| error.to_string())?;
        self.put(key, &bytes)
    }

    fn list(&self, prefix: &str) -> Result<Vec<String>, String> {
        self.calls.lock().expect("calls").push("list");
        let mut keys = self
            .objects
            .lock()
            .expect("objects")
            .keys()
            .filter(|key| key.starts_with(prefix))
            .cloned()
            .collect::<Vec<_>>();
        keys.sort();
        Ok(keys)
    }

    fn delete(&self, key: &str) -> Result<(), String> {
        self.calls.lock().expect("calls").push("delete");
        self.objects.lock().expect("objects").remove(key);
        Ok(())
    }
}

fn file_entry(payload: &[u8], mode: u32) -> (String, BoxStoreManifestEntry) {
    let sha = format!("{:x}", Sha256::digest(payload));
    (
        sha.clone(),
        BoxStoreManifestEntry::File {
            sha,
            size: payload.len() as u64,
            mode,
        },
    )
}

#[test]
fn duplicate_small_blob_is_fetched_once_and_local_hits_are_reused() {
    let root = temp_root("small-dedupe");
    let payload = b"shared-small-payload";
    let (sha, entry) = file_entry(payload, 0o600);
    let key = format!("blobs/{sha}");
    let store = CountingStore::with_objects([(key.clone(), payload.to_vec())]);
    let manifest = BTreeMap::from([
        ("home/box/a/store.db".to_string(), entry.clone()),
        ("home/box/b/store.db".to_string(), entry),
    ]);

    let first = download_manifest(
        &store,
        &root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert!(first.failures.is_empty(), "{:?}", first.failures);
    assert_eq!((first.files, first.verified), (2, 2));
    assert_eq!(first.bytes, (payload.len() * 2) as u64);
    assert_eq!(store.get_count(&key), 1);
    assert_eq!(fs::read(root.join("home/box/a/store.db")).unwrap(), payload);
    assert_eq!(fs::read(root.join("home/box/b/store.db")).unwrap(), payload);

    let local_only = CountingStore::default();
    let second = download_manifest(
        &local_only,
        &root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert!(second.failures.is_empty(), "{:?}", second.failures);
    assert_eq!((second.files, second.verified), (2, 2));
    assert_eq!(local_only.get_count(&key), 0);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn critical_files_precede_bulk_and_prefetch_failure_falls_back_to_blob_reads() {
    let root = temp_root("phase-order");
    let critical = b"critical";
    let bulk = b"bulk";
    let (critical_sha, critical_entry) = file_entry(critical, 0o600);
    let (bulk_sha, bulk_entry) = file_entry(bulk, 0o600);
    let critical_key = format!("blobs/{critical_sha}");
    let bulk_key = format!("blobs/{bulk_sha}");
    let store = CountingStore::with_prefetch_failure([
        (critical_key.clone(), critical.to_vec()),
        (bulk_key.clone(), bulk.to_vec()),
    ]);
    let manifest = BTreeMap::from([
        (
            "home/box/sand-data/agents/a/store.db".to_string(),
            critical_entry,
        ),
        ("home/box/workspace/note.txt".to_string(), bulk_entry),
    ]);
    let logs = Arc::new(Mutex::new(Vec::<String>::new()));
    let log_sink = Arc::clone(&logs);

    let summary = download_manifest(
        &store,
        &root,
        &manifest,
        BoxStoreDownloadOptions {
            on_log: Some(Arc::new(move |message| {
                log_sink.lock().expect("logs").push(message);
            })),
            ..BoxStoreDownloadOptions::default()
        },
    );

    assert!(summary.failures.is_empty(), "{:?}", summary.failures);
    let order = store.get_order();
    let critical_pos = order
        .iter()
        .position(|key| key == &critical_key)
        .expect("critical blob read");
    let bulk_pos = order
        .iter()
        .position(|key| key == &bulk_key)
        .expect("bulk blob read");
    assert!(critical_pos < bulk_pos, "read order: {order:?}");
    assert!(
        logs.lock()
            .expect("logs")
            .iter()
            .any(|line| line.contains("falling back to per-blob presign"))
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn duplicate_large_blob_streams_once_then_copies_from_the_verified_local_file() {
    let root = temp_root("large-dedupe");
    let payload = vec![b'x'; 128 * 1024];
    let (sha, entry) = file_entry(&payload, 0o600);
    let key = format!("blobs/{sha}");
    let store = CountingStore::with_objects([(key.clone(), payload.clone())]);
    let manifest = BTreeMap::from([
        ("home/box/workspace/one.bin".to_string(), entry.clone()),
        ("home/box/workspace/two.bin".to_string(), entry),
    ]);

    let summary = download_manifest(
        &store,
        &root,
        &manifest,
        BoxStoreDownloadOptions {
            large_object_threshold: 1,
            ..BoxStoreDownloadOptions::default()
        },
    );

    assert!(summary.failures.is_empty(), "{:?}", summary.failures);
    assert_eq!((summary.files, summary.verified), (2, 2));
    assert_eq!(store.file_get_count(&key), 1);
    assert_eq!(fs::read(root.join("home/box/workspace/one.bin")).unwrap(), payload);
    assert_eq!(
        fs::read(root.join("home/box/workspace/two.bin")).unwrap(),
        fs::read(root.join("home/box/workspace/one.bin")).unwrap()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn byte_budget_admits_an_oversized_item_when_idle() {
    let budget = BoxStoreByteBudget::new(10);
    assert!(budget.try_acquire(20));
    assert_eq!(budget.in_flight_bytes(), 20);
    assert!(!budget.try_acquire(1));
    budget.release(20);
    assert!(budget.try_acquire(10));
    budget.release(10);
}

#[test]
fn pack_restore_reports_each_restored_path_and_skips_loose_blobs() {
    let root = temp_root("pack-progress");
    let source_root = root.join("pack-sources");
    let target_root = root.join("target");
    fs::create_dir_all(&source_root).expect("create pack source root");
    fs::create_dir_all(&target_root).expect("create target root");

    let mut sources = Vec::new();
    let mut manifest = BTreeMap::new();
    let mut expected = Vec::new();
    for index in 0..16_u64 {
        let rel_path = format!("workspace/pack-{index}.txt");
        let bytes = format!("pack-progress-{index}\n").into_bytes();
        let sha = sha256_hex(&bytes);
        let source = source_root.join(format!("source-{index}"));
        fs::write(&source, &bytes).expect("write pack source");
        sources.push(PackSource {
            abs_path: source,
            sha: sha.clone(),
            size: bytes.len() as u64,
            vmtime: index + 1,
        });
        manifest.insert(
            rel_path.clone(),
            BoxStoreManifestEntry::File {
                sha,
                size: bytes.len() as u64,
                mode: 0o600,
            },
        );
        expected.push((rel_path, bytes));
    }

    let pack_path = root.join("pack.gz");
    let built = build_pack_file(&pack_path, &sources, || false)
        .expect("build pack")
        .expect("pack build not aborted");
    assert_eq!(built.members.len(), 16);
    let pack_bytes = fs::read(&pack_path).expect("read pack");
    let pack_id = sha256_hex(&pack_bytes);
    let pack_key = format!("{BOX_STORE_PACKS_PREFIX}/{pack_id}");
    let index = PackIndex {
        version: PACK_INDEX_VERSION,
        max_vmtime: 16,
        packs: vec![PackEntry {
            id: pack_id,
            bytes: pack_bytes.len() as u64,
            members: built.members,
        }],
    };
    let store = CountingStore::with_objects([
        (
            BOX_STORE_PACK_INDEX_KEY.to_string(),
            serialize_pack_index(&index)
                .expect("serialize pack index")
                .into_bytes(),
        ),
        (pack_key.clone(), pack_bytes),
    ]);
    let progress = Arc::new(Mutex::new(Vec::<usize>::new()));
    let progress_sink = Arc::clone(&progress);

    let summary = download_manifest(
        &store,
        &target_root,
        &manifest,
        BoxStoreDownloadOptions {
            on_progress: Some(Arc::new(move |snapshot| {
                progress_sink.lock().expect("progress").push(snapshot.files);
            })),
            ..BoxStoreDownloadOptions::default()
        },
    );

    assert!(summary.failures.is_empty(), "{:?}", summary.failures);
    assert_eq!((summary.files, summary.verified), (16, 16));
    assert_eq!(store.file_get_count(&pack_key), 1);
    let mut progress_files = progress.lock().expect("progress").clone();
    progress_files.sort_unstable();
    assert_eq!(progress_files, (1_usize..=16).collect::<Vec<_>>());
    for (rel_path, bytes) in expected {
        assert_eq!(
            fs::read(target_root.join(rel_path)).expect("read restored pack member"),
            bytes
        );
    }

    let _ = fs::remove_dir_all(root);
}

#[test]
fn all_local_pack_members_skip_pack_download_and_are_counted_by_bulk_phase() {
    let root = temp_root("pack-local-skip");
    let target_root = root.join("target");
    fs::create_dir_all(&target_root).expect("create target root");

    let mut manifest = BTreeMap::new();
    let mut members = Vec::new();
    for index in 0..16_u64 {
        let rel_path = format!("workspace/local-{index}.txt");
        let bytes = format!("already-local-{index}\n").into_bytes();
        let sha = sha256_hex(&bytes);
        let destination = target_root.join(&rel_path);
        fs::create_dir_all(destination.parent().expect("destination parent")).expect("create parent");
        fs::write(&destination, &bytes).expect("write local file");
        manifest.insert(
            rel_path,
            BoxStoreManifestEntry::File {
                sha: sha.clone(),
                size: bytes.len() as u64,
                mode: 0o600,
            },
        );
        members.push(mahayana_host_runtime::extensions::box_store_sync::box_store_pack::PackMember {
            sha,
            size: bytes.len() as u64,
            offset: index,
            clen: 1,
            vmtime: index + 1,
        });
    }
    let pack_key = format!("{BOX_STORE_PACKS_PREFIX}/unused-local-pack");
    let index = PackIndex {
        version: PACK_INDEX_VERSION,
        max_vmtime: 16,
        packs: vec![PackEntry {
            id: "unused-local-pack".into(),
            bytes: 16,
            members,
        }],
    };
    let store = CountingStore::with_objects([(
        BOX_STORE_PACK_INDEX_KEY.to_string(),
        serialize_pack_index(&index)
            .expect("serialize pack index")
            .into_bytes(),
    )]);

    let summary = download_manifest(
        &store,
        &target_root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );

    assert!(summary.failures.is_empty(), "{:?}", summary.failures);
    assert_eq!((summary.files, summary.verified), (16, 16));
    assert_eq!(store.file_get_count(&pack_key), 0);

    let _ = fs::remove_dir_all(root);
}

#[cfg(unix)]
#[test]
fn symlink_restore_reports_all_frozen_steps_and_allows_absolute_targets_inside_root() {
    let root = temp_root("symlink-trace");
    fs::create_dir_all(root.join("home/box/link")).unwrap();
    let inside_absolute = root.join("home/box/inside");
    assert!(symlink_target_stays_within_root(
        &root,
        &root.join("home/box/link"),
        inside_absolute.to_str().unwrap(),
    ));
    assert!(!symlink_target_stays_within_root(
        &root,
        &root.join("home/box/link"),
        "/etc/passwd",
    ));

    let manifest = BTreeMap::from([(
        "home/box/link".to_string(),
        BoxStoreManifestEntry::Symlink {
            target: "../target".into(),
        },
    )]);
    let traces = Arc::new(Mutex::new(Vec::<BoxStoreDownloadTrace>::new()));
    let trace_sink = Arc::clone(&traces);

    let summary = download_manifest(
        &CountingStore::default(),
        &root,
        &manifest,
        BoxStoreDownloadOptions {
            download_concurrency: 1,
            on_trace: Some(Arc::new(move |trace| {
                trace_sink.lock().expect("traces").push(trace);
            })),
            ..BoxStoreDownloadOptions::default()
        },
    );
    assert!(summary.failures.is_empty(), "{:?}", summary.failures);
    assert_eq!((summary.files, summary.verified), (1, 1));
    assert_eq!(fs::read_link(root.join("home/box/link")).unwrap(), PathBuf::from("../target"));

    let traces = traces.lock().expect("traces");
    let started_steps = traces
        .iter()
        .filter(|trace| trace.event == "symlink-step-started")
        .filter_map(|trace| trace.symlink_step.clone())
        .collect::<Vec<_>>();
    for step in [
        "prepare-parent",
        "match-existing",
        "create-temp",
        "inspect-destination",
        "remove-directory",
        "rename-temp",
        "apply-owner",
        "cleanup-temp",
        "verify-target",
    ] {
        assert!(
            started_steps.iter().any(|candidate| candidate == step),
            "missing symlink trace step {step}: {started_steps:?}"
        );
    }
    let finished = traces
        .iter()
        .find(|trace| trace.event == "symlink-entry-finished")
        .expect("symlink entry finished trace");
    assert_eq!(finished.symlinks_started, 1);
    assert_eq!(finished.symlinks_completed, 1);
    assert_eq!(finished.symlinks_in_flight, 0);
    assert_eq!(finished.symlink_outcome.as_deref(), Some("restored"));
    for step in [
        "prepare-parent",
        "match-existing",
        "create-temp",
        "inspect-destination",
        "remove-directory",
        "rename-temp",
        "apply-owner",
        "verify-target",
        "cleanup-temp",
    ] {
        assert_eq!(finished.active_symlink_steps.get(step), Some(&0));
    }

    let _ = fs::remove_dir_all(root);
}

const REVISION_KEY_RESTORE_FAILURE: &str =
    "manifest may replace installation revision key material";

struct RevisionKeyRestoreFixture {
    root: PathBuf,
}

impl RevisionKeyRestoreFixture {
    fn new(label: &str) -> Self {
        let root = temp_root(label);
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&root).expect("create private restore fixture");
        Self { root }
    }

    fn data_root(&self) -> PathBuf {
        self.root.join("home/box/sand-data")
    }

    fn install_key(&self) -> PathBuf {
        let data_root = self.data_root();
        fs::create_dir_all(&data_root).expect("create fixture installation");
        let path = data_root.join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
        fs::write(&path, [0x31_u8; 32]).expect("write synthetic installed key");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .expect("set fixture key mode");
        }
        path
    }

    fn stage_key(&self, label: &str) -> PathBuf {
        let data_root = self.data_root();
        fs::create_dir_all(&data_root).expect("create fixture staging directory");
        let path = data_root.join(format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.{label}.tmp"));
        fs::write(&path, [0x54_u8; 32]).expect("write synthetic staged key");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&path, fs::Permissions::from_mode(0o600))
                .expect("set fixture staging mode");
        }
        path
    }
}

impl Drop for RevisionKeyRestoreFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[derive(PartialEq, Eq)]
struct RestoreTreeEntry {
    kind: &'static str,
    contents: Vec<u8>,
    target: Option<PathBuf>,
    modified: Option<SystemTime>,
    #[cfg(unix)]
    mode: u32,
    #[cfg(unix)]
    inode: u64,
}

fn restore_tree_snapshot(root: &Path) -> BTreeMap<PathBuf, RestoreTreeEntry> {
    fn visit(root: &Path, path: &Path, entries: &mut BTreeMap<PathBuf, RestoreTreeEntry>) {
        let metadata = fs::symlink_metadata(path).expect("snapshot fixture metadata");
        let is_link = metadata.file_type().is_symlink();
        #[cfg(unix)]
        use std::os::unix::fs::MetadataExt;
        entries.insert(
            path.strip_prefix(root).expect("owned fixture path").to_path_buf(),
            RestoreTreeEntry {
                kind: if is_link { "link" } else if metadata.is_dir() { "directory" } else { "file" },
                contents: if metadata.is_file() {
                    fs::read(path).expect("snapshot fixture file")
                } else {
                    Vec::new()
                },
                target: is_link.then(|| fs::read_link(path).expect("snapshot fixture symlink")),
                modified: metadata.modified().ok(),
                #[cfg(unix)]
                mode: metadata.mode(),
                #[cfg(unix)]
                inode: metadata.ino(),
            },
        );
        if metadata.is_dir() {
            for entry in fs::read_dir(path).expect("snapshot fixture directory") {
                visit(root, &entry.expect("fixture directory entry").path(), entries);
            }
        }
    }
    let mut entries = BTreeMap::new();
    visit(root, root, &mut entries);
    entries
}

fn assert_revision_key_preflight_refused(
    summary: &BoxStoreDownloadSummary,
    store: &CountingStore,
    entries: usize,
) {
    assert_eq!(summary.manifest_entries, entries);
    assert_eq!((summary.files, summary.bytes, summary.verified), (0, 0, 0));
    assert_eq!(summary.failures, vec![REVISION_KEY_RESTORE_FAILURE.to_string()]);
    assert!(store.calls.lock().expect("calls").is_empty(), "preflight accessed object store");
}

fn recording_restore_options(effects: &Arc<AtomicUsize>) -> BoxStoreDownloadOptions {
    let progress = Arc::clone(effects);
    let trace = Arc::clone(effects);
    let log = Arc::clone(effects);
    BoxStoreDownloadOptions {
        on_progress: Some(Arc::new(move |_| {
            progress.fetch_add(1, Ordering::SeqCst);
        })),
        on_trace: Some(Arc::new(move |_| {
            trace.fetch_add(1, Ordering::SeqCst);
        })),
        on_log: Some(Arc::new(move |_| {
            log.fetch_add(1, Ordering::SeqCst);
        })),
        ..BoxStoreDownloadOptions::default()
    }
}

fn manifest_with_restore_control(
    forbidden_path: String,
    forbidden_entry: BoxStoreManifestEntry,
) -> (CountingStore, BTreeMap<String, BoxStoreManifestEntry>) {
    let payload = [0x72_u8; 32];
    let (sha, ordinary) = file_entry(&payload, 0o600);
    let store = CountingStore::with_objects([(format!("blobs/{sha}"), payload.to_vec())]);
    let manifest = BTreeMap::from([
        ("home/box/sand-data/agents/control/store.db".to_string(), ordinary),
        (forbidden_path, forbidden_entry),
    ]);
    (store, manifest)
}

#[test]
fn incoming_key_and_publication_artifacts_refuse_the_entire_restore_without_mutation() {
    let payload = [0x72_u8; 32];
    let (sha, file) = file_entry(&payload, 0o600);
    let entries = [
        file.clone(),
        BoxStoreManifestEntry::LegacyFile { sha, size: payload.len() as u64 },
        BoxStoreManifestEntry::Symlink { target: "foreign-key.bin".into() },
    ];
    let paths = [
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.foreign.tmp"),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}..tmp"),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}/child"),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.foreign.tmp/child"),
    ];
    for path in paths {
        for entry in &entries {
            let fixture = RevisionKeyRestoreFixture::new("key-preflight");
            let installed = fixture.install_key();
            let before = restore_tree_snapshot(&fixture.root);
            let (store, manifest) = manifest_with_restore_control(path.clone(), entry.clone());
            let summary = download_manifest(
                &store,
                &fixture.root,
                &manifest,
                BoxStoreDownloadOptions::default(),
            );
            assert_revision_key_preflight_refused(&summary, &store, manifest.len());
            assert!(fs::read(installed).expect("read installed key") == [0x31_u8; 32],
                "installed key changed");
            assert!(restore_tree_snapshot(&fixture.root) == before, "rejected restore changed fixture");
        }
    }
}

#[test]
fn empty_installation_cannot_import_a_foreign_key_or_create_restore_directories() {
    let fixture = RevisionKeyRestoreFixture::new("empty-key-import");
    let target = fixture.root.join("uncreated-installation");
    let before = restore_tree_snapshot(&fixture.root);
    let (_, foreign) = file_entry(&[0x72_u8; 32], 0o600);
    let (store, manifest) = manifest_with_restore_control(
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        foreign,
    );
    let summary = download_manifest(
        &store,
        &target,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert!(!target.exists(), "rejected restore created the target root");
    assert!(restore_tree_snapshot(&fixture.root) == before, "rejected restore changed fixture");
}

#[test]
fn file_and_symlink_ancestors_cannot_remove_an_installed_revision_key() {
    let (_, file) = file_entry(&[0x72_u8; 32], 0o600);
    for ancestor in ["home", "home/box", "home/box/sand-data"] {
        for entry in [
            file.clone(),
            BoxStoreManifestEntry::Symlink { target: "workspace".into() },
        ] {
            let fixture = RevisionKeyRestoreFixture::new("key-ancestor");
            fixture.install_key();
            let before = restore_tree_snapshot(&fixture.root);
            let (store, manifest) = manifest_with_restore_control(ancestor.into(), entry);
            let summary = download_manifest(
                &store,
                &fixture.root,
                &manifest,
                BoxStoreDownloadOptions::default(),
            );
            assert_revision_key_preflight_refused(&summary, &store, manifest.len());
            assert!(restore_tree_snapshot(&fixture.root) == before, "ancestor restore changed fixture");
        }
    }
}

#[cfg(unix)]
#[test]
fn existing_and_manifest_symlink_aliases_cannot_route_restore_into_key_paths() {
    for use_manifest_aliases in [false, true] {
        let fixture = RevisionKeyRestoreFixture::new("key-alias");
        fixture.install_key();
        fs::create_dir(fixture.root.join("workspace")).expect("create fixture workspace");
        if !use_manifest_aliases {
            std::os::unix::fs::symlink(
                "../home/box/sand-data",
                fixture.root.join("workspace/alias"),
            ).expect("create existing data alias");
            std::os::unix::fs::symlink("alias", fixture.root.join("workspace/second"))
                .expect("create existing alias chain");
        }
        let before = restore_tree_snapshot(&fixture.root);
        let (store, mut manifest) = manifest_with_restore_control(
            format!("workspace/second/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
            BoxStoreManifestEntry::Symlink { target: "../foreign-key.bin".into() },
        );
        if use_manifest_aliases {
            manifest.insert(
                "workspace/alias".into(),
                BoxStoreManifestEntry::Symlink { target: "../home/box/sand-data".into() },
            );
            manifest.insert(
                "workspace/second".into(),
                BoxStoreManifestEntry::Symlink { target: "alias".into() },
            );
        }
        let summary = download_manifest(
            &store,
            &fixture.root,
            &manifest,
            BoxStoreDownloadOptions::default(),
        );
        assert_revision_key_preflight_refused(&summary, &store, manifest.len());
        assert!(restore_tree_snapshot(&fixture.root) == before, "alias restore changed fixture");
    }
}

#[cfg(unix)]
#[test]
fn symlink_target_checks_expand_existing_links_before_parent_components() {
    let fixture = RevisionKeyRestoreFixture::new("key-target-parent");
    fixture.install_key();
    fs::create_dir(fixture.data_root().join("child")).expect("create data child");
    fs::create_dir(fixture.root.join("workspace")).expect("create fixture workspace");
    std::os::unix::fs::symlink(
        "../home/box/sand-data/child",
        fixture.root.join("workspace/jump"),
    ).expect("create target alias");
    let before = restore_tree_snapshot(&fixture.root);
    let (store, manifest) = manifest_with_restore_control(
        "workspace/key-link".into(),
        BoxStoreManifestEntry::Symlink {
            target: format!("jump/../{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        },
    );
    let summary = download_manifest(
        &store,
        &fixture.root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert!(restore_tree_snapshot(&fixture.root) == before, "target alias restore changed fixture");
}

#[cfg(unix)]
#[test]
fn restore_cannot_detach_an_existing_symlink_to_the_installation_directory() {
    let fixture = RevisionKeyRestoreFixture::new("key-directory-alias");
    let installed = fixture.install_key();
    let actual_data = fixture.root.join("actual-data");
    fs::rename(fixture.data_root(), &actual_data).expect("move fixture installation");
    std::os::unix::fs::symlink(&actual_data, fixture.data_root())
        .expect("create installation directory alias");
    let before = restore_tree_snapshot(&fixture.root);
    let (_, file) = file_entry(&[0x72_u8; 32], 0o600);
    let (store, manifest) = manifest_with_restore_control(
        "home/box/sand-data/settings.json".into(),
        file,
    );
    let summary = download_manifest(
        &store,
        &fixture.root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert!(fs::read(installed).expect("read aliased installed key") == [0x31_u8; 32],
        "installed key namespace changed");
    assert!(restore_tree_snapshot(&fixture.root) == before, "installation alias was detached");
}

#[test]
fn ordinary_restore_preserves_key_and_allows_unrelated_matching_basenames() {
    let fixture = RevisionKeyRestoreFixture::new("ordinary-key-control");
    let installed = fixture.install_key();
    let key_before = restore_tree_snapshot(&fixture.root)
        .remove(Path::new("home/box/sand-data").join(HOST_CONFIG_REVISION_KEY_FILE_NAME).as_path())
        .expect("fixture key snapshot");
    let payload = b"ordinary portable state";
    let (sha, entry) = file_entry(payload, 0o600);
    let store = CountingStore::with_objects([(format!("blobs/{sha}"), payload.to_vec())]);
    let paths = [
        "home/box/sand-data/settings.json".to_string(),
        format!("workspace/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        "workspace/unrelated/HOST-CONFIG-REVISION-KEY.BIN".to_string(),
        format!("workspace/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.foreign.tmp"),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.notes"),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.tmp"),
    ];
    let mut manifest = paths.iter().map(|path| (path.clone(), entry.clone()))
        .collect::<BTreeMap<_, _>>();
    #[cfg(unix)]
    manifest.insert(
        "workspace/data-alias".into(),
        BoxStoreManifestEntry::Symlink { target: "../home/box/sand-data".into() },
    );
    let summary = download_manifest(
        &store,
        &fixture.root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert!(summary.failures.is_empty(), "ordinary restore failed");
    assert_eq!((summary.files, summary.verified), (manifest.len(), manifest.len()));
    assert!(!store.calls.lock().expect("calls").is_empty(), "ordinary restore did no object work");
    for path in paths {
        assert!(fs::read(fixture.root.join(path)).expect("read ordinary restored file") == payload,
            "ordinary payload changed");
    }
    assert!(fs::read(installed).expect("read preserved key") == [0x31_u8; 32],
        "ordinary restore changed key");
    let key_after = restore_tree_snapshot(&fixture.root)
        .remove(Path::new("home/box/sand-data").join(HOST_CONFIG_REVISION_KEY_FILE_NAME).as_path())
        .expect("restored key snapshot");
    assert!(key_after == key_before, "ordinary restore changed key metadata");
}

#[cfg(windows)]
#[test]
fn native_windows_separators_case_and_filename_aliases_cannot_import_a_key() {
    for path in [
        "home\\box\\sand-data\\HOST-CONFIG-REVISION-KEY.BIN".to_string(),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}. "),
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}:foreign"),
    ] {
        let fixture = RevisionKeyRestoreFixture::new("windows-key-alias");
        let before = restore_tree_snapshot(&fixture.root);
        let (_, file) = file_entry(&[0x72_u8; 32], 0o600);
        let (store, manifest) = manifest_with_restore_control(path, file);
        let summary = download_manifest(
            &store,
            &fixture.root,
            &manifest,
            BoxStoreDownloadOptions::default(),
        );
        assert_revision_key_preflight_refused(&summary, &store, manifest.len());
        assert!(restore_tree_snapshot(&fixture.root) == before, "aliased restore changed fixture");
    }
}

#[test]
fn protected_portable_namespace_case_aliases_are_refused_before_restore() {
    for path in [
        "HOME/BOX/SAND-DATA/HOST-CONFIG-REVISION-KEY.BIN",
        "home/box/sand-data/HOST-CONFIG-REVISION-KEY.BIN.FOREIGN.TMP",
    ] {
        let fixture = RevisionKeyRestoreFixture::new("portable-key-case");
        fixture.install_key();
        let before = restore_tree_snapshot(&fixture.root);
        let (_, file) = file_entry(&[0x72_u8; 32], 0o600);
        let (store, manifest) = manifest_with_restore_control(path.into(), file);
        let summary = download_manifest(
            &store,
            &fixture.root,
            &manifest,
            BoxStoreDownloadOptions::default(),
        );
        assert_revision_key_preflight_refused(&summary, &store, manifest.len());
        assert!(restore_tree_snapshot(&fixture.root) == before, "case alias restore changed fixture");
    }
}

#[test]
fn hardlinked_local_hit_cannot_change_installed_key_metadata() {
    let fixture = RevisionKeyRestoreFixture::new("hardlinked-key");
    let installed = fixture.install_key();
    fs::create_dir(fixture.root.join("workspace")).expect("create fixture workspace");
    let alias = fixture.root.join("workspace/ordinary-local-hit.bin");
    fs::hard_link(&installed, &alias).expect("create actual installed-key hardlink");
    let before = restore_tree_snapshot(&fixture.root);
    // The payload matches the installed bytes, so without native admission the
    // restore's local-hit path would apply this broader mode to the same file.
    let (_, local_hit) = file_entry(&[0x31_u8; 32], 0o644);
    let (store, manifest) = manifest_with_restore_control(
        "workspace/ordinary-local-hit.bin".into(),
        local_hit,
    );
    let summary = download_manifest(
        &store,
        &fixture.root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert!(fs::read(installed).expect("read hardlinked key") == [0x31_u8; 32],
        "hardlinked key bytes changed");
    assert!(restore_tree_snapshot(&fixture.root) == before, "hardlinked key metadata changed");
}

#[test]
fn distinct_same_content_and_unrelated_hardlinks_remain_ordinary_local_hits() {
    let fixture = RevisionKeyRestoreFixture::new("ordinary-hardlink-control");
    let installed = fixture.install_key();
    fs::create_dir(fixture.root.join("workspace")).expect("create fixture workspace");
    let same_contents = fixture.root.join("workspace/same-contents.bin");
    fs::write(&same_contents, [0x31_u8; 32]).expect("write distinct same-content file");
    let ordinary_source = fixture.root.join("workspace/source.bin");
    fs::write(&ordinary_source, b"ordinary hardlink payload").expect("write ordinary source");
    let ordinary_alias = fixture.root.join("workspace/linked.bin");
    fs::hard_link(&ordinary_source, &ordinary_alias).expect("create ordinary hardlink");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&same_contents, &ordinary_source] {
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))
                .expect("set initial ordinary mode");
        }
    }
    let key_relative = Path::new("home/box/sand-data").join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
    let key_before = restore_tree_snapshot(&fixture.root)
        .remove(&key_relative)
        .expect("fixture key snapshot");
    let (same_sha, same_entry) = file_entry(&[0x31_u8; 32], 0o644);
    let (ordinary_sha, ordinary_entry) = file_entry(b"ordinary hardlink payload", 0o644);
    let manifest = BTreeMap::from([
        ("workspace/same-contents.bin".into(), same_entry),
        ("workspace/linked.bin".into(), ordinary_entry),
    ]);
    let store = CountingStore::default();
    let summary = download_manifest(
        &store,
        &fixture.root,
        &manifest,
        BoxStoreDownloadOptions::default(),
    );
    assert!(summary.failures.is_empty(), "ordinary native-identity control failed");
    assert_eq!((summary.files, summary.verified), (2, 2));
    assert_eq!(store.get_count(&format!("blobs/{same_sha}")), 0);
    assert_eq!(store.get_count(&format!("blobs/{ordinary_sha}")), 0);
    assert!(fs::read(&same_contents).expect("read distinct file") == [0x31_u8; 32],
        "distinct same-content payload changed");
    assert!(fs::read(&ordinary_alias).expect("read ordinary alias") == b"ordinary hardlink payload",
        "ordinary hardlink payload changed");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        for path in [&same_contents, &ordinary_source, &ordinary_alias] {
            assert_eq!(fs::metadata(path).expect("ordinary metadata").permissions().mode() & 0o777, 0o644);
        }
    }
    assert!(fs::read(installed).expect("read preserved installed key") == [0x31_u8; 32],
        "ordinary native-identity control changed key bytes");
    let key_after = restore_tree_snapshot(&fixture.root)
        .remove(&key_relative)
        .expect("preserved key snapshot");
    assert!(key_after == key_before, "ordinary native-identity control changed key metadata");
}

#[test]
fn staged_key_hardlinks_refuse_but_distinct_same_content_local_hits_restore() {
    for hardlinked in [true, false] {
        let fixture = RevisionKeyRestoreFixture::new("staged-key-identity");
        let staged = fixture.stage_key("pending");
        let installed = fixture.data_root().join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
        assert!(!installed.exists(), "staging fixture already has a final key");
        fs::create_dir(fixture.root.join("workspace")).expect("create fixture workspace");
        let alias = fixture.root.join("workspace/staged-local-hit.bin");
        if hardlinked {
            fs::hard_link(&staged, &alias).expect("create actual staging-file hardlink");
        } else {
            fs::write(&alias, [0x54_u8; 32]).expect("write distinct staging-content control");
        }
        let before = restore_tree_snapshot(&fixture.root);
        let (_, entry) = file_entry(&[0x54_u8; 32], 0o644);
        let (store, manifest) = manifest_with_restore_control(
            "workspace/staged-local-hit.bin".into(),
            entry,
        );
        let effects = Arc::new(AtomicUsize::new(0));
        let summary = download_manifest(
            &store,
            &fixture.root,
            &manifest,
            recording_restore_options(&effects),
        );
        if hardlinked {
            assert_revision_key_preflight_refused(&summary, &store, manifest.len());
            assert_eq!(effects.load(Ordering::SeqCst), 0, "refused staging restore invoked callbacks");
            assert!(restore_tree_snapshot(&fixture.root) == before, "staged hardlink changed fixture");
        } else {
            assert!(summary.failures.is_empty(), "distinct staging-content control failed");
            assert_eq!((summary.files, summary.verified), (manifest.len(), manifest.len()));
            assert!(effects.load(Ordering::SeqCst) > 0, "ordinary control invoked no callbacks");
            let after = restore_tree_snapshot(&fixture.root);
            let relative = staged.strip_prefix(&fixture.root).expect("owned staging path");
            assert!(after.get(relative) == before.get(relative), "distinct control changed staged key");
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                assert_eq!(fs::metadata(&alias).expect("ordinary local-hit metadata").permissions().mode() & 0o777, 0o644);
            }
        }
        assert!(!installed.exists(), "restore materialized a final key");
        assert!(fs::read(staged).expect("read staged key") == [0x54_u8; 32],
            "staged key bytes changed");
    }
}

#[test]
fn staging_identity_limit_admits_32_and_refuses_33_without_effects() {
    for staged_count in [32, 33] {
        let fixture = RevisionKeyRestoreFixture::new("staging-identity-limit");
        fixture.install_key();
        for index in 0..staged_count {
            fixture.stage_key(&format!("pending-{index}"));
        }
        let before = restore_tree_snapshot(&fixture.root);
        let data_before = restore_tree_snapshot(&fixture.data_root());
        let payload = b"ordinary staging-limit control";
        let (sha, entry) = file_entry(payload, 0o600);
        let paths = [
            "workspace/control.txt".to_string(),
            format!("workspace/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
            format!("workspace/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending-0.tmp"),
        ];
        let manifest = paths.iter().map(|path| (path.clone(), entry.clone()))
            .collect::<BTreeMap<_, _>>();
        let store = CountingStore::with_objects([(format!("blobs/{sha}"), payload.to_vec())]);
        let effects = Arc::new(AtomicUsize::new(0));
        let summary = download_manifest(
            &store,
            &fixture.root,
            &manifest,
            recording_restore_options(&effects),
        );
        if staged_count == 33 {
            assert_revision_key_preflight_refused(&summary, &store, manifest.len());
            assert_eq!(effects.load(Ordering::SeqCst), 0, "overflow invoked restore callbacks");
            assert!(restore_tree_snapshot(&fixture.root) == before, "staging overflow changed fixture");
        } else {
            assert!(summary.failures.is_empty(), "32 staging identities blocked ordinary restore");
            assert_eq!((summary.files, summary.verified), (manifest.len(), manifest.len()));
            assert!(effects.load(Ordering::SeqCst) > 0, "ordinary control invoked no callbacks");
            assert!(!store.calls.lock().expect("calls").is_empty(), "ordinary control did no object work");
            for path in paths {
                assert!(fs::read(fixture.root.join(path)).expect("read ordinary bound control") == payload,
                    "ordinary bound-control payload changed");
            }
            assert!(restore_tree_snapshot(&fixture.data_root()) == data_before,
                "ordinary bound control changed installation material");
        }
    }
}

#[cfg(unix)]
#[test]
fn unreadable_staging_identity_refuses_before_store_or_restore_effects() {
    let fixture = RevisionKeyRestoreFixture::new("staging-identity-unreadable");
    fixture.install_key();
    std::os::unix::fs::symlink(
        "missing-staged-key",
        fixture.data_root().join(format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.dangling.tmp")),
    ).expect("create unreadable staging identity");
    let before = restore_tree_snapshot(&fixture.root);
    let (_, entry) = file_entry(&[0x72_u8; 32], 0o600);
    let (store, manifest) = manifest_with_restore_control("workspace/ordinary.bin".into(), entry);
    let effects = Arc::new(AtomicUsize::new(0));
    let summary = download_manifest(
        &store,
        &fixture.root,
        &manifest,
        recording_restore_options(&effects),
    );
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert_eq!(effects.load(Ordering::SeqCst), 0, "unreadable staging identity invoked callbacks");
    assert!(restore_tree_snapshot(&fixture.root) == before, "unreadable staging identity changed fixture");
}

fn alternate_installation(fixture: &RevisionKeyRestoreFixture) -> PathBuf {
    let root = fixture.root.join("workspace/private-state");
    fs::create_dir_all(&root).expect("create explicit alternate installation");
    for (name, byte) in [
        (HOST_CONFIG_REVISION_KEY_FILE_NAME.to_string(), 0x41_u8),
        (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"), 0x62_u8),
    ] {
        let path = root.join(name);
        fs::write(&path, [byte; 32]).expect("write synthetic alternate installation material");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(path, fs::Permissions::from_mode(0o600))
                .expect("set alternate installation mode");
        }
    }
    root
}

#[test]
fn actual_installation_root_refuses_key_staging_and_ancestor_imports_without_effects() {
    for installed in [false, true] {
        for path in [
            format!("workspace/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
            format!("workspace/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.foreign.tmp"),
            "workspace/private-state".into(),
            "workspace".into(),
            format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        ] {
            for entry in [
                file_entry(&[0x72_u8; 32], 0o644).1,
                BoxStoreManifestEntry::Symlink { target: "foreign.bin".into() },
            ] {
                let fixture = RevisionKeyRestoreFixture::new("actual-root-refusal");
                let actual = if installed {
                    fixture.install_key();
                    alternate_installation(&fixture)
                } else {
                    fixture.root.join("workspace/private-state")
                };
                let before = restore_tree_snapshot(&fixture.root);
                let (store, manifest) = manifest_with_restore_control(path.clone(), entry);
                let effects = Arc::new(AtomicUsize::new(0));
                let mut options = recording_restore_options(&effects);
                options.installation_data_root = Some(actual);
                let summary = download_manifest(&store, &fixture.root, &manifest, options);
                assert_revision_key_preflight_refused(&summary, &store, manifest.len());
                assert_eq!(effects.load(Ordering::SeqCst), 0, "actual-root refusal invoked callbacks");
                assert!(restore_tree_snapshot(&fixture.root) == before,
                    "actual-root refusal changed the installation or restore namespace");
            }
        }
    }
}

#[test]
fn actual_installation_key_and_staging_native_aliases_refuse_local_hit_changes() {
    for staged in [false, true] {
        let fixture = RevisionKeyRestoreFixture::new("actual-root-native-alias");
        fixture.install_key();
        let actual = alternate_installation(&fixture);
        let (name, byte) = if staged {
            (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"), 0x62_u8)
        } else {
            (HOST_CONFIG_REVISION_KEY_FILE_NAME.to_string(), 0x41_u8)
        };
        fs::hard_link(actual.join(name), fixture.root.join("workspace/local-hit.bin"))
            .expect("create actual installation hardlink");
        let before = restore_tree_snapshot(&fixture.root);
        let (_, entry) = file_entry(&[byte; 32], 0o644);
        let (store, manifest) = manifest_with_restore_control("workspace/local-hit.bin".into(), entry);
        let effects = Arc::new(AtomicUsize::new(0));
        let mut options = recording_restore_options(&effects);
        options.installation_data_root = Some(actual);
        let summary = download_manifest(&store, &fixture.root, &manifest, options);
        assert_revision_key_preflight_refused(&summary, &store, manifest.len());
        assert_eq!(effects.load(Ordering::SeqCst), 0, "actual native alias invoked callbacks");
        assert!(restore_tree_snapshot(&fixture.root) == before,
            "actual installation alias changed bytes or metadata");
    }
}

#[test]
fn actual_installation_neighbors_and_unrelated_matching_names_restore() {
    let fixture = RevisionKeyRestoreFixture::new("actual-root-neighbor-control");
    fixture.install_key();
    let actual = alternate_installation(&fixture);
    let before = restore_tree_snapshot(&fixture.root);
    let payload = b"ordinary alternate-root neighbor";
    let (sha, entry) = file_entry(payload, 0o600);
    let store = CountingStore::with_objects([(format!("blobs/{sha}"), payload.to_vec())]);
    let paths = [
        "workspace/private-state/settings.json".to_string(),
        "home/box/sand-data/settings.json".to_string(),
        format!("workspace/unrelated/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        format!("workspace/unrelated/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"),
        format!("workspace/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.notes"),
    ];
    let manifest = paths.iter().map(|path| (path.clone(), entry.clone())).collect();
    let effects = Arc::new(AtomicUsize::new(0));
    let mut options = recording_restore_options(&effects);
    options.installation_data_root = Some(actual);
    let summary = download_manifest(&store, &fixture.root, &manifest, options);
    assert!(summary.failures.is_empty(), "ordinary alternate-root restore failed");
    assert_eq!((summary.files, summary.verified), (paths.len(), paths.len()));
    assert!(effects.load(Ordering::SeqCst) > 0, "ordinary actual-root control had no callbacks");
    assert!(!store.calls.lock().expect("store calls").is_empty(), "ordinary control did no payload work");
    for path in paths {
        assert!(fs::read(fixture.root.join(path)).expect("read ordinary neighbor") == payload,
            "ordinary neighbor changed");
    }
    let after = restore_tree_snapshot(&fixture.root);
    for path in [
        format!("home/box/sand-data/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        format!("workspace/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
        format!("workspace/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"),
    ] {
        assert!(before.get(Path::new(&path)) == after.get(Path::new(&path)),
            "ordinary alternate-root restore changed protected material");
    }
}

#[test]
fn recursive_replacement_refuses_descendant_key_and_staging_hardlinks_without_effects() {
    for staged in [false, true] {
        for entry in [
            file_entry(&[0x72_u8; 32], 0o600).1,
            BoxStoreManifestEntry::Symlink { target: "ordinary-target".into() },
        ] {
            let fixture = RevisionKeyRestoreFixture::new("delete-descendant-hardlink");
            let key = fixture.install_key();
            let source = if staged { fixture.stage_key("pending") } else { key };
            let victim = fixture.root.join("workspace/replace-me/nested");
            fs::create_dir_all(&victim).expect("create removal subtree");
            fs::hard_link(source, victim.join("ordinary-name.bin"))
                .expect("create protected descendant hardlink");
            let before = restore_tree_snapshot(&fixture.root);
            let (store, manifest) = manifest_with_restore_control("workspace/replace-me".into(), entry);
            let effects = Arc::new(AtomicUsize::new(0));
            let summary = download_manifest(
                &store, &fixture.root, &manifest, recording_restore_options(&effects),
            );
            assert_revision_key_preflight_refused(&summary, &store, manifest.len());
            assert_eq!(effects.load(Ordering::SeqCst), 0, "descendant refusal invoked callbacks");
            assert!(restore_tree_snapshot(&fixture.root) == before,
                "recursive replacement changed protected descendant or other state");
        }
    }
}

fn ordinary_directory_replacement_control(symlink: bool) {
    let fixture = RevisionKeyRestoreFixture::new("ordinary-directory-replacement");
    let key = fixture.install_key();
    fixture.stage_key("pending");
    let protected_before = restore_tree_snapshot(&fixture.data_root());
    let victim = fixture.root.join("workspace/replace-me");
    fs::create_dir_all(victim.join("nested")).expect("create ordinary removal subtree");
    fs::write(victim.join("nested").join(HOST_CONFIG_REVISION_KEY_FILE_NAME), b"unrelated")
        .expect("create unrelated same-basename removal entry");
    fs::create_dir(fixture.root.join("workspace/ordinary-target")).expect("create link target");
    #[cfg(unix)]
    std::os::unix::fs::symlink(&key, victim.join("key-reference"))
        .expect("create ordinary unlinked reference");
    let payload = b"ordinary directory replacement";
    let (sha, file) = file_entry(payload, 0o600);
    let store = CountingStore::with_objects([(format!("blobs/{sha}"), payload.to_vec())]);
    let entry = if symlink {
        BoxStoreManifestEntry::Symlink { target: "ordinary-target".into() }
    } else {
        file
    };
    let manifest = BTreeMap::from([("workspace/replace-me".into(), entry)]);
    let effects = Arc::new(AtomicUsize::new(0));
    let summary = download_manifest(
        &store, &fixture.root, &manifest, recording_restore_options(&effects),
    );
    assert!(summary.failures.is_empty(), "ordinary directory replacement failed");
    assert_eq!((summary.files, summary.verified), (1, 1));
    assert!(effects.load(Ordering::SeqCst) > 0, "ordinary replacement invoked no callbacks");
    if symlink {
        assert_eq!(fs::read_link(&victim).expect("replacement link"), Path::new("ordinary-target"));
    } else {
        assert!(fs::read(&victim).expect("replacement file") == payload, "replacement payload changed");
        assert!(!store.calls.lock().expect("store calls").is_empty(), "file control did no object work");
    }
    assert!(fs::read(key).expect("preserved installation key") == [0x31_u8; 32],
        "ordinary replacement changed key bytes");
    assert!(restore_tree_snapshot(&fixture.data_root()) == protected_before,
        "ordinary replacement changed protected metadata");
}

#[test]
fn ordinary_directory_file_replacement_preserves_installation_material() {
    ordinary_directory_replacement_control(false);
}

#[cfg(unix)]
#[test]
fn ordinary_directory_symlink_replacement_preserves_installation_material() {
    ordinary_directory_replacement_control(true);
}

#[test]
fn recursive_removal_entry_limit_refuses_without_effects() {
    let fixture = RevisionKeyRestoreFixture::new("delete-entry-limit");
    fixture.install_key();
    let victim = fixture.root.join("workspace/replace-me");
    fs::create_dir_all(&victim).expect("create bounded removal subtree");
    // 16,384 children plus the removal root exceed the whole-manifest bound.
    for index in 0..16_384 {
        fs::write(victim.join(format!("entry-{index}")), b"").expect("create bounded removal entry");
    }
    let before = restore_tree_snapshot(&fixture.root);
    let (_, entry) = file_entry(&[0x72_u8; 32], 0o600);
    let (store, manifest) = manifest_with_restore_control("workspace/replace-me".into(), entry);
    let effects = Arc::new(AtomicUsize::new(0));
    let summary = download_manifest(
        &store, &fixture.root, &manifest, recording_restore_options(&effects),
    );
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert_eq!(effects.load(Ordering::SeqCst), 0, "removal entry overflow invoked callbacks");
    assert!(restore_tree_snapshot(&fixture.root) == before, "removal entry overflow changed fixture");
}

#[cfg(unix)]
#[test]
fn recursive_removal_depth_limit_refuses_without_effects() {
    let fixture = RevisionKeyRestoreFixture::new("delete-depth-limit");
    fixture.install_key();
    let mut nested = fixture.root.join("workspace/replace-me");
    fs::create_dir_all(&nested).expect("create removal root");
    for _ in 0..65 {
        nested.push("d");
        fs::create_dir(&nested).expect("create bounded-depth removal directory");
    }
    let before = restore_tree_snapshot(&fixture.root);
    let (_, entry) = file_entry(&[0x72_u8; 32], 0o600);
    let (store, manifest) = manifest_with_restore_control("workspace/replace-me".into(), entry);
    let effects = Arc::new(AtomicUsize::new(0));
    let summary = download_manifest(
        &store, &fixture.root, &manifest, recording_restore_options(&effects),
    );
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert_eq!(effects.load(Ordering::SeqCst), 0, "removal depth overflow invoked callbacks");
    assert!(restore_tree_snapshot(&fixture.root) == before, "removal depth overflow changed fixture");
}

#[cfg(unix)]
#[test]
fn recursive_removal_unreadable_identity_refuses_without_effects() {
    use std::os::unix::fs::PermissionsExt;
    assert_ne!(unsafe { libc::geteuid() }, 0, "this Actions control requires an unprivileged user");
    let fixture = RevisionKeyRestoreFixture::new("delete-unreadable-identity");
    fixture.install_key();
    let victim = fixture.root.join("workspace/replace-me");
    fs::create_dir_all(&victim).expect("create removal subtree");
    let unreadable = victim.join("unreadable.bin");
    fs::write(&unreadable, b"ordinary unreadable descendant").expect("create unreadable control");
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o600)).expect("set control mode");
    let before = restore_tree_snapshot(&fixture.root);
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0)).expect("deny descendant read");
    let (_, entry) = file_entry(&[0x72_u8; 32], 0o600);
    let (store, manifest) = manifest_with_restore_control("workspace/replace-me".into(), entry);
    let effects = Arc::new(AtomicUsize::new(0));
    let summary = download_manifest(
        &store, &fixture.root, &manifest, recording_restore_options(&effects),
    );
    let after_mode = fs::metadata(&unreadable).expect("preserved unreadable descendant").permissions().mode() & 0o777;
    fs::set_permissions(&unreadable, fs::Permissions::from_mode(0o600)).expect("restore owned fixture access");
    assert_revision_key_preflight_refused(&summary, &store, manifest.len());
    assert_eq!(effects.load(Ordering::SeqCst), 0, "unreadable removal identity invoked callbacks");
    assert_eq!(after_mode, 0, "unreadable descendant permissions changed");
    assert!(restore_tree_snapshot(&fixture.root) == before, "unreadable removal identity changed fixture");
}

#[cfg(target_os = "linux")]
struct LinuxBindRestoreFixture {
    fixture: Option<RevisionKeyRestoreFixture>,
    mount_path: PathBuf,
    mounted: bool,
}

#[cfg(target_os = "linux")]
impl LinuxBindRestoreFixture {
    fn mount(fixture: RevisionKeyRestoreFixture, source: &Path, mount_path: PathBuf) -> Self {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        assert!(source.starts_with(&fixture.root) && mount_path.starts_with(&fixture.root),
            "bind fixture paths must be exclusively owned");
        fs::create_dir_all(&mount_path).expect("create owned bind mount point");
        let source_c = CString::new(source.as_os_str().as_bytes()).expect("owned source path");
        let target_c = CString::new(mount_path.as_os_str().as_bytes()).expect("owned mount path");
        let mounted = unsafe {
            libc::mount(source_c.as_ptr(), target_c.as_ptr(), std::ptr::null(), libc::MS_BIND, std::ptr::null())
        };
        assert_eq!(mounted, 0, "real bind mount requires Linux Actions CAP_SYS_ADMIN: {}", std::io::Error::last_os_error());
        let owned = Self { fixture: Some(fixture), mount_path, mounted: true };
        let source_identity = same_file::Handle::from_path(source).expect("read source native identity");
        let mount_identity = same_file::Handle::from_path(&owned.mount_path).expect("read mount native identity");
        assert!(source_identity == mount_identity, "fixture did not establish a real protected native alias");
        owned
    }

    fn unmount(&mut self) -> std::io::Result<()> {
        use std::ffi::CString;
        use std::os::unix::ffi::OsStrExt;
        if self.mounted {
            let target = CString::new(self.mount_path.as_os_str().as_bytes()).expect("owned mount path");
            if unsafe { libc::umount2(target.as_ptr(), 0) } != 0 {
                return Err(std::io::Error::last_os_error());
            }
            self.mounted = false;
        }
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Drop for LinuxBindRestoreFixture {
    fn drop(&mut self) {
        if self.unmount().is_err() {
            // Never let the ordinary fixture's recursive cleanup visit a mount
            // whose teardown is unresolved. Retain only this owned fixture.
            if let Some(fixture) = self.fixture.take() {
                eprintln!("owned bind fixture retained after unmount failure: {}", fixture.root.display());
                std::mem::forget(fixture);
            }
        }
    }
}

#[cfg(target_os = "linux")]
fn linux_bind_descendant_restore_control(symlink: bool) {
    for alternate in [false, true] {
        for protected_descendant in [true, false] {
            let fixture = RevisionKeyRestoreFixture::new("linux-bind-descendant");
            fixture.install_key();
            let data_root = if alternate { alternate_installation(&fixture) } else { fixture.data_root() };
            let victim = fixture.root.join("workspace/replace-me");
            fs::create_dir_all(victim.join("ordinary-child")).expect("create directory replacement control");
            fs::write(victim.join("ordinary-child/keep-until-replaced"), b"ordinary")
                .expect("write directory control");
            fs::create_dir_all(fixture.root.join("workspace/ordinary-target")).expect("create link control target");
            let mount_path = if protected_descendant {
                victim.join("mounted")
            } else {
                fixture.root.join("workspace/mount-capability-control")
            };
            let mut mounted = LinuxBindRestoreFixture::mount(fixture, &data_root, mount_path);
            let fixture = mounted.fixture.as_ref().expect("live owned fixture");
            let before = restore_tree_snapshot(&fixture.root);
            let protected_before = restore_tree_snapshot(&data_root);
            let payload = b"ordinary Linux replacement";
            let (sha, file) = file_entry(payload, 0o600);
            let store = CountingStore::with_objects([(format!("blobs/{sha}"), payload.to_vec())]);
            let entry = if symlink {
                BoxStoreManifestEntry::Symlink { target: "ordinary-target".into() }
            } else {
                file
            };
            let manifest = BTreeMap::from([("workspace/replace-me".into(), entry)]);
            let effects = Arc::new(AtomicUsize::new(0));
            let mut options = recording_restore_options(&effects);
            options.installation_data_root = alternate.then(|| data_root.clone());
            let summary = download_manifest(&store, &fixture.root, &manifest, options);
            if protected_descendant {
                assert_revision_key_preflight_refused(&summary, &store, manifest.len());
                assert_eq!(effects.load(Ordering::SeqCst), 0, "mounted descendant invoked callbacks");
                assert!(restore_tree_snapshot(&fixture.root) == before,
                    "mounted descendant refusal changed filesystem state");
            } else {
                assert!(summary.failures.is_empty(), "Linux ordinary directory control failed");
                assert_eq!((summary.files, summary.verified), (1, 1));
                assert!(effects.load(Ordering::SeqCst) > 0, "Linux ordinary control did no restore work");
                if symlink {
                    assert_eq!(fs::read_link(&victim).expect("ordinary replacement link"), Path::new("ordinary-target"));
                } else {
                    assert!(fs::read(&victim).expect("ordinary replacement file") == payload,
                        "Linux ordinary replacement payload changed");
                    assert!(!store.calls.lock().expect("store calls").is_empty(), "ordinary file did no payload work");
                }
            }
            assert!(restore_tree_snapshot(&data_root) == protected_before,
                "Linux replacement changed protected bytes or metadata");
            mounted.unmount().expect("unmount exact owned fixture before recursive cleanup");
        }
    }
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires Linux Actions private mount namespace and CAP_SYS_ADMIN"]
fn linux_bind_mounted_descendant_file_restore_boundary() {
    linux_bind_descendant_restore_control(false);
}

#[cfg(target_os = "linux")]
#[test]
#[ignore = "requires Linux Actions private mount namespace and CAP_SYS_ADMIN"]
fn linux_bind_mounted_descendant_symlink_restore_boundary() {
    linux_bind_descendant_restore_control(true);
}
