use std::fs;
use std::sync::{Arc, Mutex};

use mahayana_host_runtime::{
    attachment_paths::{get_agent_assets_dir, get_agent_attachments_dir, get_agent_media_store_roots},
    automations::automation_id::stable_automation_id,
    r#box::box_monitor_layout::{display_space_sentence, SAND_MONITOR_HEIGHT, SAND_MONITOR_WIDTH},
    durable_file_policy::{
        BOX_STORE_SAND_DATA_EXCLUDED_FILE_NAMES, HOST_CONFIG_REVISION_KEY_FILE_NAME,
        HOST_CONFIG_REVISION_KEY_TEMP_FILE_PATTERN, SAND_UPGRADE_RESUME_FILE_NAME,
        is_host_config_revision_key_file_name,
    },
    extensions::box_store_sync::{
        box_object_store::{BoxObjectStore, LocalFsObjectStore},
        box_store_manifest::BoxManifestMap,
        box_store_manifest_format::BOX_STORE_BLOBS_PREFIX,
        box_store_transfer::{BoxStoreTransfer, glob_matches_path},
    },
    host_diagnostics::{pin_host_diagnostics_reporter, report_host_diagnostic, HostDiagnostic},
    host_paths::{get_host_config_revision_key_path, get_sand_root_dir},
    ports::{transport::{SandTransport, SandUpdate}, user_computer::{SingleUserComputer, DEFAULT_SAND_COMPUTER_ID}},
    sand_quiet_work_origin::SAND_QUIET_WORK_ORIGIN_KEY,
    sand_user_identity::{normalize_sand_user_full_name, render_user_identity_system_prompt, MAX_FULL_NAME_LENGTH},
    sha256::sha256_hex,
    storage::folder_id::is_safe_folder_id,
};
use serde_json::Map;

#[test]
fn host_foundation_constants_and_hashing_match_reference_contracts() {
    assert_eq!(SAND_QUIET_WORK_ORIGIN_KEY, "sand.quiet-work-origin");
    assert_eq!(sha256_hex("abc"), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad");
    assert!(is_safe_folder_id("agent-42"));
    for unsafe_id in ["", ".", "..", "a/b", "a\\b", "a\0b"] { assert!(!is_safe_folder_id(unsafe_id)); }
    assert_eq!(SAND_MONITOR_WIDTH, 1280);
    assert_eq!(SAND_MONITOR_HEIGHT, 800);
    assert!(display_space_sentence(None, None).contains("0..1279 × 0..799"));
    assert_eq!(BOX_STORE_SAND_DATA_EXCLUDED_FILE_NAMES[0], SAND_UPGRADE_RESUME_FILE_NAME);
}

#[test]
fn automation_id_is_stable_uuid_shaped_and_versioned() {
    let a = stable_automation_id("agent", "local");
    let b = stable_automation_id("agent", "local");
    assert_eq!(a, b);
    assert_eq!(a.len(), 36);
    assert_eq!(&a[14..15], "5");
    assert!(matches!(&a[19..20], "8" | "9" | "a" | "b"));
}

#[test]
fn attachment_paths_and_single_computer_follow_grok_defaults() {
    assert_eq!(get_agent_attachments_dir("/tmp/a").to_string_lossy(), "/tmp/a/attachments");
    assert_eq!(get_agent_assets_dir("/tmp/a").to_string_lossy(), "/tmp/a/assets");
    assert_eq!(get_agent_media_store_roots("/tmp/a").len(), 2);
    let computer = SingleUserComputer::always_connected(7_u32, None);
    assert_eq!(computer.list()[0].id, DEFAULT_SAND_COMPUTER_ID);
    assert_eq!(computer.resolve(None).unwrap().computer, &7);
    assert!(computer.resolve(Some("other")).is_none());
}

#[test]
fn transport_tracks_only_send_and_reaction_outcomes() {
    let mut next = 0_u32;
    let mut transport = SandTransport::new(|update: &SandUpdate| { next += 1; Some(format!("{}-{next}", update.kind)) });
    transport.on_update(&SandUpdate { kind: "send-message".into(), fields: Map::new() });
    assert_eq!(transport.last_sent_message_id(), Some("send-message-1"));
    transport.on_update(&SandUpdate { kind: "react-to-message".into(), fields: Map::new() });
    assert!(transport.last_reaction_applied());
}

#[test]
fn user_identity_clamps_and_diagnostics_reporter_is_replaceable() {
    assert_eq!(normalize_sand_user_full_name(Some("  Ada   Lovelace ")).as_deref(), Some("Ada Lovelace"));
    assert_eq!(normalize_sand_user_full_name(Some("   ")), None);
    assert_eq!(normalize_sand_user_full_name(Some(&"x".repeat(300))).unwrap().len(), MAX_FULL_NAME_LENGTH);
    assert!(render_user_identity_system_prompt(Some("Ada")).contains("Your user is Ada"));
    let seen = Arc::new(Mutex::new(Vec::new()));
    let sink = Arc::clone(&seen);
    pin_host_diagnostics_reporter(Some(Arc::new(move |d| sink.lock().unwrap().push(d.kind.clone()))));
    report_host_diagnostic(&HostDiagnostic { kind: "ready".into(), fields: Map::new() });
    pin_host_diagnostics_reporter(None);
    assert_eq!(&*seen.lock().unwrap(), &["ready".to_string()]);
}

#[test]
fn config_revision_key_path_and_portable_filename_matchers_agree() {
    assert_eq!(get_host_config_revision_key_path(), get_sand_root_dir().join(HOST_CONFIG_REVISION_KEY_FILE_NAME));
    for (name, expected) in [
        (HOST_CONFIG_REVISION_KEY_FILE_NAME.to_string(), true),
        (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.candidate.tmp"), true),
        (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}..tmp"), true),
        (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.tmp"), false),
        (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.notes"), false),
        (format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.candidate.tmp.bak"), false),
        (format!("other-{HOST_CONFIG_REVISION_KEY_FILE_NAME}"), false),
    ] {
        assert_eq!(is_host_config_revision_key_file_name(&name), expected);
        let portable_match = [HOST_CONFIG_REVISION_KEY_FILE_NAME, HOST_CONFIG_REVISION_KEY_TEMP_FILE_PATTERN]
            .iter().any(|pattern| glob_matches_path(pattern, &name));
        assert_eq!(portable_match, expected);
    }
}

struct PrivatePortableFixture(std::path::PathBuf);

impl PrivatePortableFixture {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "fabushi-config-portable-{}", uuid::Uuid::new_v4().simple()
        ));
        let mut builder = fs::DirBuilder::new();
        #[cfg(unix)]
        {
            use std::os::unix::fs::DirBuilderExt;
            builder.mode(0o700);
        }
        builder.create(&path).expect("create private portable fixture");
        Self(path)
    }
}

impl Drop for PrivatePortableFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

#[test]
fn config_revision_key_and_candidates_never_enter_portable_box_uploads() {
    let fixture = PrivatePortableFixture::new();
    let sand_data = fixture.0.join("sand-data");
    fs::create_dir(&sand_data).expect("create sand-data fixture");
    let key_bytes = [0x31_u8; 32];
    let candidate_bytes = [0x73_u8; 32];
    let partial_bytes = [0x19_u8; 7];
    let candidate_name = format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.candidate.tmp");
    let partial_name = format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.partial.tmp");
    let ordinary_name = format!("{HOST_CONFIG_REVISION_KEY_FILE_NAME}.notes");
    fs::write(sand_data.join(HOST_CONFIG_REVISION_KEY_FILE_NAME), key_bytes).expect("seed synthetic key");
    fs::write(sand_data.join(&candidate_name), candidate_bytes).expect("seed complete candidate");
    fs::write(sand_data.join(&partial_name), partial_bytes).expect("seed incomplete candidate");
    fs::write(sand_data.join("settings.json"), br#"{"timezone":"UTC"}"#).expect("seed settings control");
    fs::write(sand_data.join(&ordinary_name), b"ordinary-note").expect("seed ordinary filename control");
    let prefix = "home/box/sand-data";
    // The production sand_data_excludes owner prefixes this same policy list.
    let owned_excludes = BOX_STORE_SAND_DATA_EXCLUDED_FILE_NAMES.iter()
        .map(|name| format!("{prefix}/{name}")).collect::<Vec<_>>();
    let excludes = owned_excludes.iter().map(String::as_str).collect::<Vec<_>>();
    let store = LocalFsObjectStore::new(fixture.0.join("objects"));
    let transfer = BoxStoreTransfer::new(1024, 1, 1024, None, Arc::new(|_| {}));
    let mut manifest = BoxManifestMap::new();
    let summary = transfer.sync_tree_category(
        &store, &mut manifest, &sand_data, prefix, "sand-data", &excludes, None, false, true,
    ).expect("sync portable sand-data through existing transfer owner");
    assert_eq!(summary.failures, 0);
    assert_eq!(summary.metadata_failures, 0);
    assert_eq!(summary.files_uploaded, 2);
    assert_eq!(manifest.len(), 2);
    assert!(manifest.contains_key(&format!("{prefix}/settings.json")));
    assert!(manifest.contains_key(&format!("{prefix}/{ordinary_name}")));
    for name in [HOST_CONFIG_REVISION_KEY_FILE_NAME, &candidate_name, &partial_name] {
        assert!(!manifest.contains_key(&format!("{prefix}/{name}")), "installation material entered manifest");
    }
    for secret in [&key_bytes[..], &candidate_bytes[..], &partial_bytes[..]] {
        let object_key = format!("{BOX_STORE_BLOBS_PREFIX}/{}", sha256_hex(secret));
        assert!(store.get(&object_key).expect("check excluded object").is_none(), "installation material was uploaded");
    }
    assert_eq!(store.list(BOX_STORE_BLOBS_PREFIX).expect("list uploaded controls").len(), 2);

    let workspace = fixture.0.join("workspace");
    fs::create_dir(&workspace).expect("create workspace control");
    fs::write(workspace.join(HOST_CONFIG_REVISION_KEY_FILE_NAME), b"ordinary-workspace-file")
        .expect("seed same-basename workspace control");
    let workspace_summary = transfer.sync_tree_category(
        &store, &mut manifest, &workspace, "home/box/workspace", "workspace", &excludes, None, false, true,
    ).expect("sync ordinary workspace control");
    assert_eq!(workspace_summary.files_uploaded, 1);
    assert!(manifest.contains_key(&format!("home/box/workspace/{HOST_CONFIG_REVISION_KEY_FILE_NAME}")));
}
