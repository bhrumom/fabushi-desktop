use std::fs;
use std::path::PathBuf;

use mahayana_host_runtime::extensions::box_store_sync::box_copy_in::{
    CopyInOutcome, run_local_box_copy_in,
};
use mahayana_host_runtime::extensions::box_store_sync::box_object_store::{
    BoxObjectStore, LocalFsObjectStore,
};
use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest_format::{
    BOX_STORE_MANIFEST_REL_PATH,
};
use mahayana_host_runtime::extensions::box_store_sync::box_store_pack::{
    BOX_STORE_PACK_INDEX_KEY, BOX_STORE_PACKS_PREFIX, PACK_INDEX_VERSION, PackEntry, PackIndex,
    PackSource, build_pack_file, serialize_pack_index, sha256_hex,
};

fn temp_root(label: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "fabushi-box-copy-in-pack-{label}-{}",
        uuid::Uuid::new_v4().simple()
    ));
    fs::create_dir_all(&root).expect("create temp root");
    root
}

#[test]
fn local_copy_in_hydrates_bulk_small_files_from_pack_without_loose_blobs() {
    let root = temp_root("only");
    let store_root = root.join("store");
    let source_root = root.join("source");
    let target_root = root.join("target");
    fs::create_dir_all(&source_root).expect("create source");
    fs::create_dir_all(&target_root).expect("create target");
    let store = LocalFsObjectStore::new(&store_root);

    let mut sources = Vec::new();
    let mut manifest_entries = serde_json::Map::new();
    let mut expected = Vec::new();
    for index in 0..16_u64 {
        let rel_path = format!("workspace/bulk-{index}.txt");
        let bytes = format!("bulk-small-pack-member-{index}\n").into_bytes();
        let sha = sha256_hex(&bytes);
        let abs_path = source_root.join(format!("member-{index}"));
        fs::write(&abs_path, &bytes).expect("write source member");
        sources.push(PackSource {
            abs_path,
            sha: sha.clone(),
            size: bytes.len() as u64,
            vmtime: index + 1,
        });
        manifest_entries.insert(
            rel_path.clone(),
            serde_json::json!({
                "kind": "file",
                "sha": sha,
                "size": bytes.len(),
                "mode": 0o600,
            }),
        );
        expected.push((rel_path, bytes, sha));
    }

    let pack_path = root.join("built.pack");
    let built = build_pack_file(&pack_path, &sources, || false)
        .expect("build pack")
        .expect("pack not aborted");
    assert_eq!(built.members.len(), 16);
    let pack_bytes = fs::metadata(&pack_path).expect("pack metadata").len();
    let pack_id = sha256_hex(&fs::read(&pack_path).expect("read pack"));
    store
        .put_from_file(&format!("{BOX_STORE_PACKS_PREFIX}/{pack_id}"), &pack_path)
        .expect("store pack");
    let index = PackIndex {
        version: PACK_INDEX_VERSION,
        max_vmtime: 16,
        packs: vec![PackEntry {
            id: pack_id,
            bytes: pack_bytes,
            members: built.members,
        }],
    };
    store
        .put(
            BOX_STORE_PACK_INDEX_KEY,
            serialize_pack_index(&index).expect("serialize index").as_bytes(),
        )
        .expect("store pack index");
    store
        .put(
            BOX_STORE_MANIFEST_REL_PATH,
            serde_json::to_vec(&serde_json::json!({
                "version": 2,
                "updatedAtMs": 1,
                "writerWindowId": "test",
                "entries": manifest_entries,
            }))
            .expect("serialize manifest")
            .as_slice(),
        )
        .expect("store manifest");

    for (_, _, sha) in &expected {
        assert!(
            store
                .get(&format!("blobs/{sha}"))
                .expect("probe loose blob")
                .is_none(),
            "test must prove hydration does not rely on loose blobs"
        );
    }

    let result = run_local_box_copy_in(&store, &target_root);
    assert_eq!(result.outcome, CopyInOutcome::Hydrated);
    assert_eq!(result.manifest_entries, 16);
    assert_eq!(result.files, 16);
    assert_eq!(result.verified, 16);
    assert!(result.failures.is_empty());

    for (rel_path, bytes, _) in expected {
        let restored = target_root.join(&rel_path);
        assert_eq!(fs::read(&restored).expect("read restored file"), bytes);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                fs::metadata(&restored).expect("restored metadata").permissions().mode() & 0o777,
                0o600
            );
        }
    }

    fs::remove_dir_all(root).expect("cleanup");
}
