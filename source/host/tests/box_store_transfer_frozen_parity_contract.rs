use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use sha2::{Digest, Sha256};

use mahayana_host_runtime::durable_file_policy::HOST_CONFIG_REVISION_KEY_FILE_NAME;
use mahayana_host_runtime::extensions::box_store_sync::box_object_store::{
    BoxObjectStore, LocalFsObjectStore,
};
use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest::BoxManifestMap;
use mahayana_host_runtime::extensions::box_store_sync::box_store_manifest_format::BoxStoreManifestEntry;
use mahayana_host_runtime::extensions::box_store_sync::box_store_transfer::{
    BOX_STORE_SNAPSHOT_TMP_SUFFIX, BoxStoreTransfer, StagedTransferFile, glob_matches_path,
};

fn temp_root(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-box-store-transfer-{label}-{}-{nonce}",
        std::process::id()
    ))
}

fn transfer(marker: Option<PathBuf>, threshold: u64) -> BoxStoreTransfer {
    BoxStoreTransfer::new(
        64 * 1024 * 1024,
        4,
        threshold,
        marker,
        Arc::new(|_| {}),
    )
}

#[test]
fn frozen_transfer_owns_small_large_symlink_ignore_and_prune_semantics() {
    let root = temp_root("shipping");
    let workspace = root.join("workspace");
    let store_root = root.join("store");
    fs::create_dir_all(workspace.join("src")).expect("create source");
    fs::write(workspace.join("src/small.txt"), b"small").expect("write small");
    fs::write(workspace.join("src/large.bin"), vec![7u8; 4096]).expect("write large");
    fs::write(workspace.join("ignored.tmp"), b"ignore").expect("write ignore");
    fs::write(
        workspace.join(format!("leak{BOX_STORE_SNAPSHOT_TMP_SUFFIX}deadbeef")),
        b"temp",
    )
    .expect("write leaked temp");

    #[cfg(unix)]
    std::os::unix::fs::symlink("small.txt", workspace.join("src/link.txt"))
        .expect("create symlink");

    let store = LocalFsObjectStore::new(&store_root);
    let transfer = transfer(None, 1024);
    let mut manifest = BoxManifestMap::new();
    let summary = transfer
        .sync_tree_category(
            &store,
            &mut manifest,
            &workspace,
            "workspace",
            "workspace",
            &["workspace/ignored.tmp"],
            None,
            false,
            true,
        )
        .expect("first transfer");

    assert_eq!(summary.failures, 0);
    assert_eq!(summary.metadata_failures, 0);
    #[cfg(unix)]
    assert_eq!(summary.files_uploaded, 3);
    #[cfg(not(unix))]
    assert_eq!(summary.files_uploaded, 2);
    assert!(manifest.contains_key("workspace/src/small.txt"));
    assert!(manifest.contains_key("workspace/src/large.bin"));
    assert!(!manifest.contains_key("workspace/ignored.tmp"));

    #[cfg(unix)]
    assert!(matches!(
        manifest.get("workspace/src/link.txt"),
        Some(BoxStoreManifestEntry::Symlink { target }) if target == "small.txt"
    ));

    for rel in ["workspace/src/small.txt", "workspace/src/large.bin"] {
        let sha = match manifest.get(rel).expect("manifest file") {
            BoxStoreManifestEntry::File { sha, .. } => sha,
            other => panic!("expected file entry, got {other:?}"),
        };
        assert!(
            store
                .get(&format!("blobs/{sha}"))
                .expect("read blob")
                .is_some()
        );
    }

    let second = transfer
        .sync_tree_category(
            &store,
            &mut manifest,
            &workspace,
            "workspace",
            "workspace",
            &["workspace/ignored.tmp"],
            None,
            false,
            true,
        )
        .expect("second transfer");
    assert_eq!(second.files_uploaded, 0);
    assert!(second.skipped_unchanged >= 2);

    fs::remove_file(workspace.join("src/small.txt")).expect("remove small");
    let pruned = transfer
        .sync_tree_category(
            &store,
            &mut manifest,
            &workspace,
            "workspace",
            "workspace",
            &["workspace/ignored.tmp"],
            None,
            false,
            true,
        )
        .expect("prune transfer");
    assert!(pruned.removed >= 1);
    assert!(!manifest.contains_key("workspace/src/small.txt"));

    let removed = transfer.sweep_leaked_temps(&[workspace.clone()]);
    assert_eq!(removed, 1);
    assert!(
        !workspace
            .join(format!("leak{BOX_STORE_SNAPSHOT_TMP_SUFFIX}deadbeef"))
            .exists()
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn frozen_transfer_skips_hydration_marker_and_its_temp_variants() {
    let root = temp_root("hydration-marker");
    let workspace = root.join("workspace");
    fs::create_dir_all(&workspace).expect("create workspace");
    let marker = workspace.join("box-store-hydration-handoff.json");
    fs::write(&marker, b"marker").expect("marker");
    fs::write(
        workspace.join("box-store-hydration-handoff.json.abc.tmp"),
        b"temp",
    )
    .expect("marker temp");
    fs::write(workspace.join("keep.txt"), b"keep").expect("keep");

    let store = LocalFsObjectStore::new(root.join("store"));
    let transfer = transfer(Some(marker), 1024);
    let mut manifest = BoxManifestMap::new();
    let summary = transfer
        .sync_tree_category(
            &store,
            &mut manifest,
            &workspace,
            "workspace",
            "workspace",
            &[],
            None,
            false,
            true,
        )
        .expect("transfer");

    assert_eq!(summary.files_uploaded, 1);
    assert!(manifest.contains_key("workspace/keep.txt"));
    assert!(!manifest.contains_key("workspace/box-store-hydration-handoff.json"));
    assert!(!manifest.contains_key("workspace/box-store-hydration-handoff.json.abc.tmp"));
    let _ = fs::remove_dir_all(root);
}

#[test]
fn frozen_glob_exclusion_covers_descendants_not_only_exact_directory() {
    assert!(glob_matches_path(
        "home/box/chrome-profile/*/Cache",
        "home/box/chrome-profile/Default/Cache/data/index"
    ));
    assert!(glob_matches_path(
        "home/**/store.db",
        "home/box/sand-data/agents/a/store.db"
    ));
    assert!(!glob_matches_path(
        "home/box/chrome-profile/*/Cache",
        "home/box/chrome-profile/Default/Cookies"
    ));
}

#[test]
fn shipping_production_has_one_transfer_owner_and_no_parallel_sync_runtime() {
    let production = include_str!("../src/extensions/box_store_sync/production.rs");
    assert!(production.contains("transfer: BoxStoreTransfer"));
    assert!(production.contains("self.transfer.sync_tree_category("));
    assert!(production.contains("transfer.sync_staged_files("));
    assert!(production.contains("poll_inner.transfer.sweep_leaked_temps(&roots)"));
    assert!(!production.contains("\nfn sync_tree_category("));
    assert!(!production.contains("\nfn sync_file("));
}

#[test]
fn transfer_excludes_actual_key_staging_and_native_aliases_in_every_category() {
    let root = temp_root("actual-installation");
    let workspace = root.join("workspace");
    let installation = workspace.join("private-state");
    fs::create_dir_all(&installation).expect("create explicit installation fixture");
    fs::create_dir_all(workspace.join("unrelated")).expect("create ordinary fixture");
    let key = installation.join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
    let staged = installation.join(format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"));
    fs::write(&key, [0x31_u8; 32]).expect("write synthetic key");
    fs::write(&staged, [0x54_u8; 32]).expect("write synthetic staged key");
    fs::hard_link(&key, workspace.join("key-alias.bin")).expect("create key native alias");
    fs::hard_link(&staged, workspace.join("staged-alias.bin")).expect("create staging native alias");
    let ordinary = b"ordinary same-basename content";
    let ordinary_path = workspace.join("unrelated").join(HOST_CONFIG_REVISION_KEY_FILE_NAME);
    fs::write(&ordinary_path, ordinary).expect("write unrelated same-basename file");
    fs::write(installation.join("settings.json"), b"{}").expect("write ordinary settings");
    let key_sha = format!("{:x}", Sha256::digest([0x31_u8; 32]));
    let staged_sha = format!("{:x}", Sha256::digest([0x54_u8; 32]));
    let store = LocalFsObjectStore::new(root.join("store"));
    let transfer = transfer(None, 1024).with_installation_data_root(installation.clone());
    let mut manifest = BoxManifestMap::new();

    // Category labels and portable prefixes must never select a weaker policy.
    for prefix in ["workspace", "home/box/cli-config", "home/box/other-category"] {
        let forbidden = [
            format!("{prefix}/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
            format!("{prefix}/private-state/{HOST_CONFIG_REVISION_KEY_FILE_NAME}.pending.tmp"),
            format!("{prefix}/key-alias.bin"),
            format!("{prefix}/staged-alias.bin"),
        ];
        for path in &forbidden {
            manifest.insert(path.clone(), BoxStoreManifestEntry::File {
                sha: key_sha.clone(), size: 32, mode: 0o600,
            });
        }
        let summary = transfer.sync_tree_category(
            &store, &mut manifest, &workspace, prefix, "explicit-category", &[], None, false, true,
        ).expect("transfer explicitly rooted category");
        assert_eq!((summary.failures, summary.metadata_failures), (0, 0));
        for path in forbidden {
            assert!(!manifest.contains_key(&path), "category retained installation material");
        }
        assert!(manifest.contains_key(&format!("{prefix}/private-state/settings.json")));
        assert!(manifest.contains_key(&format!("{prefix}/unrelated/{HOST_CONFIG_REVISION_KEY_FILE_NAME}")));
    }

    let files = vec![
        StagedTransferFile { abs_path: key, rel_path: "workspace/staged-final.bin".into(), mode: 0o600 },
        StagedTransferFile { abs_path: staged, rel_path: "workspace/staged-pending.bin".into(), mode: 0o600 },
        StagedTransferFile {
            abs_path: ordinary_path,
            rel_path: format!("workspace/staged-ordinary/{HOST_CONFIG_REVISION_KEY_FILE_NAME}"),
            mode: 0o600,
        },
    ];
    let staged_summary = transfer.sync_staged_files(&store, &mut manifest, "staged", &files, 0, true)
        .expect("transfer staged inputs");
    assert_eq!((staged_summary.failures, staged_summary.files_uploaded, staged_summary.excluded_files), (0, 1, 2));
    assert!(!manifest.contains_key("workspace/staged-final.bin"));
    assert!(!manifest.contains_key("workspace/staged-pending.bin"));
    assert!(manifest.contains_key(&format!("workspace/staged-ordinary/{HOST_CONFIG_REVISION_KEY_FILE_NAME}")));
    assert!(store.get(&format!("blobs/{key_sha}")).expect("inspect key object absence").is_none(),
        "a category uploaded installation key bytes");
    assert!(store.get(&format!("blobs/{staged_sha}")).expect("inspect staged object absence").is_none(),
        "a category uploaded staged key bytes");
    let ordinary_sha = format!("{:x}", Sha256::digest(ordinary));
    assert!(store.get(&format!("blobs/{ordinary_sha}")).expect("inspect ordinary object").as_deref() == Some(ordinary.as_slice()),
        "ordinary same-basename payload did not transfer");
    fs::remove_dir_all(root).expect("cleanup owned transfer fixture");
}
