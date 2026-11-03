use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use mahayana_host_runtime::device_pairing_owner::DevicePairingOwner;
use mahayana_host_runtime::extensions::attachments::attachments_service::AttachmentsService;
use mahayana_host_runtime::extensions::attachments::media_store::{
    CleanOldMediaOptions, MEDIA_MAX_BYTES, PLAYBACK_TRANSCODE_SUBDIR, MANAGED_OUTGOING_SUBDIR,
    clean_old_media, delete_media_buffer, extract_original_filename,
    prune_playback_transcode_cache_to_size, read_media_buffer,
    resolve_media_scoped_dir, resolve_media_subdir, sanitize_filename, save_media_buffer,
    save_media_source, write_playback_transcode_cache,
};
use mahayana_host_runtime::extensions::attachments::media_store_shared::SaveMediaSourceErrorCode;
use serde_json::json;

const MEDIA_STORE_SOURCE: &str = include_str!("../src/extensions/attachments/media_store.rs");
const ATTACHMENTS_EXTENSION_SOURCE: &str = include_str!("../src/extensions/attachments/extension.rs");
const HOST_PRODUCTION_SOURCE: &str = include_str!("../src/host_production_extensions.rs");
const HOST_APP_SOURCE: &str = include_str!("../app/src/main.rs");

struct TempTestDir {
    path: PathBuf,
}

#[test]
fn test_ocrm_media_store_lifecycle_has_one_production_worker_and_all_consumers() {
    for needle in [
        "pub struct MediaStoreLifecycleWorker",
        "run_media_store_lifecycle_pass",
        "clean_old_media(DEFAULT_TTL_MS",
        "prune_playback_transcode_cache(None)",
        "prune_outbound_media(None)",
        "MEDIA_STORE_LIFECYCLE_INTERVAL",
    ] {
        assert!(MEDIA_STORE_SOURCE.contains(needle), "missing lifecycle owner contract: {needle}");
    }
    for needle in [
        "start_lifecycle_worker",
        "MediaStoreLifecycleWorker::start(reporter)",
        "stop_lifecycle_worker",
        "worker.stop()",
    ] {
        assert!(
            ATTACHMENTS_EXTENSION_SOURCE.contains(needle),
            "Attachments must own lifecycle worker startup/shutdown: {needle}"
        );
    }
    assert!(
        HOST_PRODUCTION_SOURCE.contains("start_attachments_extension(")
            && HOST_PRODUCTION_SOURCE.contains(")?;"),
        "production Box composition must fail closed when the lifecycle worker cannot start"
    );
    assert!(
        HOST_APP_SOURCE.contains("start_production_host_box_extensions")
            && HOST_APP_SOURCE.contains("failed to start production Box extensions"),
        "Host startup must surface lifecycle composition failure"
    );
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ocrm-media-store-test-{name}-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }
}

#[test]
fn remote_media_download_is_bounded_before_payload_acceptance() {
    let remote_source = MEDIA_STORE_SOURCE
        .split("pub fn save_remote_media_for_store(")
        .nth(1)
        .expect("remote media owner exists")
        .split("pub fn resolve_media_buffer_path(")
        .next()
        .expect("remote media owner boundary exists");
    assert!(remote_source.contains("take(max_bytes.saturating_add(1))"));
    assert!(remote_source.contains("read_to_end(&mut bytes)"));
    assert!(!remote_source.contains("response.bytes()"));
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn set_media_modified(path: &Path, modified: SystemTime) {
    fs::OpenOptions::new()
        .write(true)
        .open(path)
        .unwrap()
        .set_times(fs::FileTimes::new().set_modified(modified))
        .unwrap();
}

fn save_aged_media(root: &Path, subdir: &str, name: &str, modified: SystemTime) -> PathBuf {
    let saved = save_media_buffer(
        b"media retention fixture",
        Some("text/plain"),
        subdir,
        MEDIA_MAX_BYTES,
        Some(name),
        None,
        Some(root),
    ).unwrap();
    let path = PathBuf::from(saved.path);
    set_media_modified(&path, modified);
    path
}

#[test]
fn test_ocrm_media_store_cleanup_depth_modes_and_postorder_pruning() {
    for recursive in [Some(false), None, Some(true)] {
        for prune in [false, true] {
            let temp = TempTestDir::new("cleanup-depth");
            let now = SystemTime::now();
            let old = now - Duration::from_secs(3_600);
            let future = now + Duration::from_secs(86_400);
            let root_file = save_aged_media(&temp.path, "", "root.txt", old);
            let inbound_file = save_aged_media(&temp.path, "inbound", "inbound.txt", old);
            let deep_file = save_aged_media(&temp.path, "cache/session/images", "old.txt", old);
            let live_file = save_aged_media(&temp.path, "cache/session/images", "live.txt", now);
            let future_file = save_aged_media(&temp.path, "cache/session/docs", "future.txt", future);
            let expired_branch = save_aged_media(&temp.path, "expired/a/b", "old.txt", old);
            let nested_outgoing = save_aged_media(&temp.path, "cache/outgoing/deep", "old.txt", old);
            let nested_playback = save_aged_media(&temp.path, "cache/playback-transcode/deep", "old.txt", old);
            let outgoing = save_aged_media(&temp.path, "outgoing/deep/tree", "owned.txt", old);
            let playback = save_aged_media(&temp.path, "playback-transcode/deep/tree", "owned.txt", old);
            let options = CleanOldMediaOptions { recursive, prune_empty_dirs: Some(prune) };

            clean_old_media(60_000, &options, Some(&temp.path)).unwrap();

            assert!(!root_file.exists(), "root sweep: {recursive:?}, prune={prune}");
            assert_eq!(inbound_file.exists(), recursive == Some(false));
            for file in [&deep_file, &expired_branch, &nested_outgoing, &nested_playback] {
                assert_eq!(file.exists(), recursive != Some(true), "{}: {recursive:?}", file.display());
            }
            for file in [&live_file, &future_file, &outgoing, &playback] {
                assert!(file.exists(), "retained media: {}", file.display());
            }
            assert_eq!(temp.path.join("inbound").exists(), !(prune && recursive != Some(false)));
            assert_eq!(temp.path.join("expired").exists(), !(prune && recursive == Some(true)));
            assert!(temp.path.join("cache/session/images").is_dir());
            assert!(temp.path.join("cache/session/docs").is_dir());
            assert!(temp.path.is_dir(), "the media root is never pruned");
        }
    }
}

#[test]
fn test_ocrm_media_store_cleanup_missing_root_and_root_admission_errors() {
    let temp = TempTestDir::new("cleanup-root");
    let missing = temp.path.join("missing");
    clean_old_media(0, &CleanOldMediaOptions::default(), Some(&missing)).unwrap();
    assert!(!missing.exists());
    let file = temp.path.join("file");
    fs::write(&file, b"not a directory").unwrap();
    assert!(clean_old_media(0, &CleanOldMediaOptions::default(), Some(&file)).is_err());
    assert_eq!(fs::read(file).unwrap(), b"not a directory");
}

#[cfg(any(windows, target_os = "macos"))]
#[test]
fn test_ocrm_media_store_cleanup_protects_top_level_case_variants() {
    let temp = TempTestDir::new("cleanup-protected-case");
    let old = SystemTime::now() - Duration::from_secs(3_600);
    let mut retained = Vec::new();
    for (variant, canonical) in [("Outgoing", "outgoing"), ("Playback-Transcode", "playback-transcode")] {
        // Seed the stored spelling before the canonical consumer opens it.
        fs::create_dir_all(temp.path.join(variant)).unwrap();
        let variant_file = save_aged_media(&temp.path, &format!("{variant}/deep/tree"), "variant.txt", old);
        let canonical_file = save_aged_media(&temp.path, &format!("{canonical}/deep/tree"), "canonical.txt", old);
        assert!(fs::read_dir(&temp.path).unwrap().any(|entry| entry.unwrap().file_name() == variant));
        // On a case-insensitive volume both spellings must reach both saves.
        // A case-sensitive volume instead exercises the conservative retention
        // policy for two distinct top-level directories; neither is skipped.
        let canonical_via_variant = temp.path.join(variant).join("deep/tree")
            .join(canonical_file.file_name().unwrap());
        if canonical_via_variant.exists() {
            assert_eq!(fs::read(&canonical_via_variant).unwrap(), b"media retention fixture");
            let variant_via_canonical = temp.path.join(canonical).join("deep/tree")
                .join(variant_file.file_name().unwrap());
            assert_eq!(fs::read(&variant_via_canonical).unwrap(), b"media retention fixture");
        }
        retained.extend([variant_file, canonical_file]);
    }
    let ordinary = save_aged_media(&temp.path, "ordinary/deep", "expired.txt", old);
    let nested_outgoing = save_aged_media(&temp.path, "cache/Outgoing/deep", "expired.txt", old);
    let nested_playback = save_aged_media(&temp.path, "cache/Playback-Transcode/deep", "expired.txt", old);
    let options = CleanOldMediaOptions { recursive: Some(true), prune_empty_dirs: Some(true) };

    clean_old_media(60_000, &options, Some(&temp.path)).unwrap();

    for file in retained {
        assert_eq!(fs::read(file).unwrap(), b"media retention fixture");
    }
    for file in [ordinary, nested_outgoing, nested_playback] {
        assert!(!file.exists(), "ordinary nested lookalikes remain eligible");
    }
    assert!(!temp.path.join("ordinary").exists());
    assert!(!temp.path.join("cache").exists());
    assert!(temp.path.is_dir());
}

#[cfg(any(unix, windows))]
#[test]
fn test_ocrm_media_store_recursive_cleanup_retains_links_and_outside_targets() {
    let temp = TempTestDir::new("cleanup-links");
    let root = temp.path.join("media");
    let outside = temp.path.join("outside");
    let old = SystemTime::now() - Duration::from_secs(3_600);
    let outside_file = save_aged_media(&outside, "deep", "outside.txt", old);
    let expired = save_aged_media(&root, "ordinary/deep", "expired.txt", old);
    let links = root.join("links/deep");
    fs::create_dir_all(&links).unwrap();
    #[cfg(unix)]
    {
        use std::os::unix::fs::symlink;
        symlink(&outside, root.join("outside-link")).unwrap();
        symlink(&outside, links.join("directory-link")).unwrap();
        symlink(&outside_file, links.join("file-link")).unwrap();
        symlink(&root, links.join("cycle")).unwrap();
        symlink(&root, temp.path.join("root-link")).unwrap();
    }
    #[cfg(windows)]
    {
        use std::os::windows::fs::{symlink_dir, symlink_file};
        // Lack of symlink capability is an explicit platform fixture failure.
        symlink_dir(&outside, root.join("outside-link")).unwrap();
        symlink_dir(&outside, links.join("directory-link")).unwrap();
        symlink_file(&outside_file, links.join("file-link")).unwrap();
        symlink_dir(&root, links.join("cycle")).unwrap();
        symlink_dir(&root, temp.path.join("root-link")).unwrap();
    }
    let options = CleanOldMediaOptions { recursive: Some(true), prune_empty_dirs: Some(true) };
    clean_old_media(60_000, &options, Some(&root)).unwrap();
    assert!(!expired.exists());
    assert!(!root.join("ordinary").exists());
    assert_eq!(fs::read(&outside_file).unwrap(), b"media retention fixture");
    for link in [root.join("outside-link"), links.join("directory-link"), links.join("file-link"), links.join("cycle")] {
        assert!(fs::symlink_metadata(link).unwrap().file_type().is_symlink());
    }
    assert!(links.is_dir(), "links keep their ancestors nonempty");
    assert!(clean_old_media(60_000, &options, Some(&temp.path.join("root-link"))).is_err());
    assert!(outside_file.exists());
}

#[cfg(all(unix, not(target_vendor = "apple")))]
#[test]
fn test_ocrm_media_store_recursive_cleanup_preserves_raw_names() {
    use std::ffi::OsString;
    use std::os::unix::ffi::OsStringExt;
    let temp = TempTestDir::new("cleanup-raw-names");
    let raw_dir = temp.path.join(OsString::from_vec(b"raw-\xff".to_vec())).join("deep");
    fs::create_dir_all(&raw_dir).unwrap();
    let raw_file = raw_dir.join(OsString::from_vec(b"expired-\xfe.bin".to_vec()));
    fs::write(&raw_file, b"expired raw name").unwrap();
    set_media_modified(&raw_file, SystemTime::now() - Duration::from_secs(3_600));
    let options = CleanOldMediaOptions { recursive: Some(true), prune_empty_dirs: Some(true) };

    clean_old_media(60_000, &options, Some(&temp.path)).unwrap();

    assert!(!raw_file.exists());
    assert!(!raw_dir.parent().unwrap().exists());
    assert!(temp.path.is_dir());
}

#[cfg(unix)]
#[test]
fn test_ocrm_media_store_recursive_cleanup_preserves_unicode_names_and_special_entries() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    let temp = TempTestDir::new("cleanup-unicode-special");
    let unicode_dir = temp.path.join("unicode-é").join("deep");
    fs::create_dir_all(&unicode_dir).unwrap();
    let unicode_file = unicode_dir.join("expired-附件.bin");
    fs::write(&unicode_file, b"expired Unicode name").unwrap();
    set_media_modified(&unicode_file, SystemTime::now() - Duration::from_secs(3_600));
    let special_dir = temp.path.join("special/deep");
    fs::create_dir_all(&special_dir).unwrap();
    let fifo = special_dir.join("retained.fifo");
    let c_path = CString::new(fifo.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(c_path.as_ptr(), 0o600) }, 0);
    let options = CleanOldMediaOptions { recursive: Some(true), prune_empty_dirs: Some(true) };

    clean_old_media(60_000, &options, Some(&temp.path)).unwrap();

    assert!(!unicode_file.exists());
    assert!(!unicode_dir.parent().unwrap().exists());
    assert!(fs::symlink_metadata(&fifo).is_ok());
    assert!(special_dir.is_dir());
    assert!(temp.path.is_dir());
}

#[test]
fn test_ocrm_media_store_save_and_read_buffer() {
    let temp = TempTestDir::new("save-read");
    let content = b"%PDF-1.4\n1 0 obj\n<<>>\nendobj\ntrailer\n<<>>\n%%EOF\n";

    let saved = save_media_buffer(
        content,
        Some("application/pdf"),
        "inbound",
        MEDIA_MAX_BYTES,
        Some("my report (2026).pdf"),
        None,
        Some(&temp.path),
    ).expect("save_media_buffer should succeed");

    assert!(saved.id.starts_with("my_report_2026---"), "ID should have sanitized prefix: {}", saved.id);
    assert!(saved.id.ends_with(".pdf"), "ID should have pdf extension: {}", saved.id);
    assert_eq!(saved.size, content.len() as u64);
    assert_eq!(saved.content_type.as_deref(), Some("application/pdf"));

    let (read_bytes, resolved_path) = read_media_buffer(
        &saved.id,
        "inbound",
        MEDIA_MAX_BYTES,
        Some(&temp.path),
    ).expect("read_media_buffer should succeed");

    assert_eq!(read_bytes, content);
    assert_eq!(resolved_path.to_string_lossy(), saved.path);

    // Verify original filename restoration
    let extracted = extract_original_filename(&saved.path);
    assert_eq!(extracted, "my_report_2026.pdf");
}

#[test]
fn test_ocrm_media_store_size_limit_rejection() {
    let temp = TempTestDir::new("size-limit");
    let limit = 256 * 1024;
    let oversized = vec![0u8; limit + 1];

    let err = save_media_buffer(
        &oversized,
        Some("application/octet-stream"),
        "inbound",
        limit as u64,
        Some("too-big.bin"),
        None,
        Some(&temp.path),
    ).expect_err("Oversized payload must be rejected");

    assert_eq!(err.code, SaveMediaSourceErrorCode::TooLarge);
    assert_eq!(err.message, "Media exceeds 256KB limit");
}

#[test]
fn test_ocrm_media_store_filename_sanitization_and_truncation() {
    // Basic sanitization on stem
    let clean = sanitize_filename("Hello, World! (Draft #1)");
    assert_eq!(clean, "Hello_World_Draft_1");

    // Collapsed consecutive underscores and trimming
    let underscores = sanitize_filename("___foo___bar___");
    assert_eq!(underscores, "foo_bar");

    // Truncation to 60 UTF-16 code units
    let long_name = "a".repeat(100);
    let sanitized_long = sanitize_filename(&long_name);
    assert_eq!(sanitized_long.len(), 60);

    // NFC normalization
    let combining = "e\u{0301}"; // e + combining acute
    let sanitized_combining = sanitize_filename(combining);
    assert_eq!(sanitized_combining, "\u{00e9}"); // normalized to single precomposed character
}

#[test]
fn test_ocrm_media_store_subdir_traversal_protection() {
    let temp = TempTestDir::new("traversal");

    // Traversal rejection
    assert!(resolve_media_subdir("../escaped", "test").is_err());
    assert!(resolve_media_subdir("foo/../../escaped", "test").is_err());
    assert!(resolve_media_subdir("/absolute/path", "test").is_err());
    assert!(resolve_media_subdir("null\0byte", "test").is_err());

    // Valid nested subdirs
    let ok_sub = resolve_media_subdir("inbound/documents", "test").expect("Valid subpath");
    assert_eq!(ok_sub, "inbound/documents");

    let scoped = resolve_media_scoped_dir(&temp.path, "inbound/documents", "test")
        .expect("Scoped dir resolution");
    assert_eq!(scoped, temp.path.join("inbound/documents"));

    #[cfg(unix)]
    {
        let outside = TempTestDir::new("traversal-outside");
        let sentinel = outside.path.join("retained.txt");
        fs::write(&sentinel, b"outside media root").unwrap();
        let linked = temp.path.join("linked");
        std::os::unix::fs::symlink(&outside.path, &linked).unwrap();
        assert!(
            resolve_media_scoped_dir(&temp.path, "linked/new", "test").is_err(),
            "an existing symlinked ancestor must not escape the media root"
        );
        assert!(!outside.path.join("new").exists());
        assert_eq!(fs::read(&sentinel).unwrap(), b"outside media root");
    }

    let escaped_file = temp.path.join("escaped.txt");
    fs::write(&escaped_file, b"must stay inaccessible").unwrap();
    assert!(
        read_media_buffer("../escaped.txt", "inbound", MEDIA_MAX_BYTES, Some(&temp.path))
            .is_err(),
        "media IDs must not traverse out of the scoped directory"
    );
    assert!(
        delete_media_buffer("../escaped.txt", "inbound", Some(&temp.path)).is_err(),
        "media deletion IDs must not traverse out of the scoped directory"
    );
    assert!(
        write_playback_transcode_cache(
            "../escaped-cache.bin",
            b"must stay inaccessible",
            MEDIA_MAX_BYTES,
            Some(&temp.path),
        )
        .is_err(),
        "playback cache names must not traverse out of the cache directory"
    );
    assert!(escaped_file.exists());
    assert!(!temp.path.join("escaped-cache.bin").exists());

    #[cfg(unix)]
    {
        let inbound = temp.path.join("inbound");
        fs::create_dir_all(&inbound).unwrap();
        std::os::unix::fs::symlink(&escaped_file, inbound.join("linked.txt")).unwrap();
        assert!(
            read_media_buffer("linked.txt", "inbound", MEDIA_MAX_BYTES, Some(&temp.path))
                .is_err(),
            "media readers must reject symlinked files"
        );
    }
}

#[test]
fn test_ocrm_media_store_local_file_source_boundaries() {
    let temp = TempTestDir::new("local-source");
    let ws_dir = temp.path.join("workspace");
    let outside_dir = temp.path.join("outside");
    fs::create_dir_all(&ws_dir).unwrap();
    fs::create_dir_all(&outside_dir).unwrap();

    // 1. Regular file inside workspace root
    let normal_file = ws_dir.join("image.png");
    fs::write(&normal_file, b"\x89PNG\r\n\x1a\nfake-png-data").unwrap();

    let saved = save_media_source(
        normal_file.to_str().unwrap(),
        None,
        "inbound",
        MEDIA_MAX_BYTES,
        Some(&temp.path),
        Some(&ws_dir),
    ).expect("Regular file inside workspace should save successfully");

    assert_eq!(saved.content_type.as_deref(), Some("image/png"));

    // 2. Directory source rejection (must be not-file)
    let dir_err = save_media_source(
        ws_dir.to_str().unwrap(),
        None,
        "inbound",
        MEDIA_MAX_BYTES,
        Some(&temp.path),
        Some(&ws_dir),
    ).expect_err("Directory source must be rejected");
    assert_eq!(dir_err.code, SaveMediaSourceErrorCode::NotFile);

    // 3. Nonexistent source rejection
    let missing_err = save_media_source(
        ws_dir.join("nonexistent.bin").to_str().unwrap(),
        None,
        "inbound",
        MEDIA_MAX_BYTES,
        Some(&temp.path),
        Some(&ws_dir),
    ).expect_err("Missing source must be rejected");
    assert_eq!(missing_err.code, SaveMediaSourceErrorCode::NotFound);

    // 4. Outside workspace root rejection
    let outside_file = outside_dir.join("secret.txt");
    fs::write(&outside_file, b"outside secret content").unwrap();

    let outside_err = save_media_source(
        outside_file.to_str().unwrap(),
        None,
        "inbound",
        MEDIA_MAX_BYTES,
        Some(&temp.path),
        Some(&ws_dir),
    ).expect_err("Outside workspace source must be rejected");
    assert_eq!(outside_err.code, SaveMediaSourceErrorCode::InvalidPath);
    assert_eq!(outside_err.message, "Media path is outside workspace root");

    // 5. Symlink source rejection
    #[cfg(unix)]
    {
        let symlink_path = ws_dir.join("symlink-to-normal.png");
        let _ = std::os::unix::fs::symlink(&normal_file, &symlink_path);
        let sym_err = save_media_source(
            symlink_path.to_str().unwrap(),
            None,
            "inbound",
            MEDIA_MAX_BYTES,
            Some(&temp.path),
            Some(&ws_dir),
        ).expect_err("Symlink source must be rejected");
        assert_eq!(sym_err.code, SaveMediaSourceErrorCode::InvalidPath);
        assert_eq!(sym_err.message, "Media path must not be a symlink");
    }
}

#[test]
fn test_ocrm_media_store_delete_and_lifecycle() {
    let temp = TempTestDir::new("delete-lifecycle");
    let content = b"temporary scratch media";

    let saved = save_media_buffer(
        content,
        Some("text/plain"),
        "inbound",
        MEDIA_MAX_BYTES,
        Some("temp.txt"),
        None,
        Some(&temp.path),
    ).unwrap();

    assert!(Path::new(&saved.path).exists());

    delete_media_buffer(&saved.id, "inbound", Some(&temp.path)).unwrap();
    assert!(!Path::new(&saved.path).exists());
}

#[test]
fn test_ocrm_media_store_playback_cache_and_clean_old_media() {
    let temp = TempTestDir::new("playback-cache");

    // Write transcode cache
    let cache_file = "rendition-001.mp4";
    let data = vec![0xaa; 1024];
    let cached_path = write_playback_transcode_cache(
        cache_file,
        &data,
        10 * 1024,
        Some(&temp.path),
    ).unwrap();
    assert!(cached_path.exists());

    // Write a second cache file
    let cache_file2 = "rendition-002.mp4";
    let cached_path2 = write_playback_transcode_cache(
        cache_file2,
        &data,
        10 * 1024,
        Some(&temp.path),
    ).unwrap();
    assert!(cached_path2.exists());

    // Prune cache to 1500 bytes (should evict oldest file, rendition-001)
    let cache_dir = temp.path.join(PLAYBACK_TRANSCODE_SUBDIR);
    prune_playback_transcode_cache_to_size(&cache_dir, 1500).unwrap();
    assert!(!cached_path.exists(), "Oldest transcode file must be evicted");
    assert!(cached_path2.exists(), "Newer transcode file must be retained");

    // clean_old_media must NEVER delete files in playback-transcode or outgoing
    let outgoing_dir = temp.path.join(MANAGED_OUTGOING_SUBDIR);
    fs::create_dir_all(&outgoing_dir).unwrap();
    let outgoing_file = outgoing_dir.join("history-record.png");
    fs::write(&outgoing_file, b"persisted history media").unwrap();

    let opts = CleanOldMediaOptions {
        recursive: Some(true),
        prune_empty_dirs: Some(false),
    };
    clean_old_media(0, &opts, Some(&temp.path)).unwrap();

    assert!(cached_path2.exists(), "Playback transcode must not be swept by clean_old_media");
    assert!(outgoing_file.exists(), "Outgoing history must not be swept by clean_old_media");
}

struct TestAuth;
impl mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth for TestAuth {
    fn get_access_token(&self) -> Result<String, String> {
        Ok("token".into())
    }
    fn get_machine_id(&self) -> Result<String, String> {
        Ok("machine".into())
    }
}

#[derive(Default)]
struct TestBox;
impl mahayana_host_runtime::extensions::attachments::attachments_service::AttachmentsBox for TestBox {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        Ok("running".into())
    }
    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

#[test]
fn test_ocrm_media_store_gateway_service_dispatch() {
    use std::sync::Arc;
    let temp = TempTestDir::new("service-dispatch");
    struct RestoreDataRoot(Option<std::ffi::OsString>);
    impl Drop for RestoreDataRoot {
        fn drop(&mut self) {
            unsafe {
                match &self.0 {
                    Some(value) => std::env::set_var("SAND_DATA_ROOT", value),
                    None => std::env::remove_var("SAND_DATA_ROOT"),
                }
            }
        }
    }
    // This existing environment-dependent service fixture runs serially.
    let _restore_data_root = RestoreDataRoot(std::env::var_os("SAND_DATA_ROOT"));
    unsafe {
        std::env::set_var("SAND_DATA_ROOT", &temp.path);
    }

    let service = AttachmentsService::new(
        temp.path.clone(),
        Arc::new(TestAuth),
        Arc::new(TestBox::default()),
        None,
    );

    // 1. saveMediaBuffer via gateway dispatch
    use base64::Engine;
    let b64 = base64::prelude::BASE64_STANDARD.encode(b"hello media world");
    let save_args = json!({
        "bufferBase64": b64,
        "contentType": "text/plain",
        "subdir": "inbound",
        "originalFilename": "test_note.txt"
    });

    let saved_val = service.dispatch_gateway("saveMediaBuffer", &save_args)
        .expect("Dispatch saveMediaBuffer")
        .expect("Result should be Ok");

    let id = saved_val.get("id").and_then(|v| v.as_str()).expect("id string");
    assert!(id.starts_with("test_note---"));

    // 2. readMediaBuffer via gateway dispatch
    let read_args = json!({
        "id": id,
        "subdir": "inbound"
    });
    let read_val = service.dispatch_gateway("readMediaBuffer", &read_args)
        .expect("Dispatch readMediaBuffer")
        .expect("Result should be Ok");

    let read_b64 = read_val.get("bufferBase64").and_then(|v| v.as_str()).unwrap();
    assert_eq!(read_b64, b64);

    // 3. extractOriginalFilename via gateway dispatch
    let extract_args = json!({
        "path": format!("/var/media/inbound/{id}")
    });
    let orig_name = service.dispatch_gateway("extractOriginalFilename", &extract_args)
        .expect("Dispatch extractOriginalFilename")
        .expect("Result should be Ok");
    assert_eq!(orig_name.as_str(), Some("test_note.txt"));

    // 4. deleteMediaBuffer via gateway dispatch
    let del_args = json!({
        "id": id,
        "subdir": "inbound"
    });
    let del_val = service.dispatch_gateway("deleteMediaBuffer", &del_args)
        .expect("Dispatch deleteMediaBuffer")
        .expect("Result should be Ok");
    assert_eq!(del_val, serde_json::Value::Bool(true));

    // 5. The real service consumer preserves the omitted/explicit depth modes.
    let nested = service.dispatch_gateway("saveMediaBuffer", &json!({
        "bufferBase64": b64,
        "contentType": "text/plain",
        "subdir": "cache/session/images",
        "originalFilename": "deep.txt"
    })).unwrap().unwrap();
    let nested_path = PathBuf::from(nested["path"].as_str().unwrap());
    let media_root = temp.path.join("media").canonicalize().unwrap();
    assert!(nested_path.starts_with(&media_root));
    set_media_modified(&nested_path, SystemTime::now() - Duration::from_secs(3_600));

    let shallow = service.dispatch_gateway("cleanOldMedia", &json!({
        "ttlMs": 60_000,
        "pruneEmptyDirs": true
    })).unwrap().unwrap();
    assert_eq!(shallow, serde_json::Value::Bool(true));
    assert!(nested_path.exists(), "omitted recursive stays one level deep");

    let recursive = service.dispatch_gateway("cleanOldMedia", &json!({
        "ttlMs": 60_000,
        "recursive": true,
        "pruneEmptyDirs": true
    })).unwrap().unwrap();
    assert_eq!(recursive, serde_json::Value::Bool(true));
    assert!(!nested_path.exists(), "Gateway explicit recursion reaches the real nested save");
    assert!(!media_root.join("cache").exists(), "expired branches are pruned in postorder");
    assert!(media_root.is_dir());
}

#[test]
fn test_ocrm_device_pairing_qr_wiring() {
    let owner = DevicePairingOwner::in_memory().expect("in-memory pairing owner");
    let setup_res = owner.handle_pair_setup_code(&json!({})).expect("handle_pair_setup_code should succeed");

    let obj = setup_res.as_object().expect("json object");
    let setup_code = obj.get("setupCode").and_then(|v| v.as_str()).expect("setupCode string");
    assert!(setup_code.starts_with("sand-setup-"));

    // Verify qrDataUrl generation and format
    let qr_data_url = obj.get("qrDataUrl").and_then(|v| v.as_str()).expect("qrDataUrl should be present");
    assert!(qr_data_url.starts_with("data:image/png;base64,"));

    // Verify qrTerminal generation and format
    let qr_terminal = obj.get("qrTerminal").and_then(|v| v.as_str()).expect("qrTerminal should be present");
    assert!(!qr_terminal.is_empty(), "qrTerminal should be non-empty");
}

#[test]
fn test_ocrm_media_store_remote_fixture_scope_and_filename_support() {
    use mahayana_host_runtime::extensions::attachments::media_store::{
        check_active_remote_fixtures_empty, expect_saved_original_filename_case,
        StoreRemoteFixtureScope,
    };

    // 1. Initial state: active scopes must be empty
    assert!(check_active_remote_fixtures_empty().is_ok());

    let target_url = "http://127.0.0.1:43123/fixture.bin?case=one";
    {
        let mut fixture = StoreRemoteFixtureScope::new(target_url);
        assert!(fixture.active);
        assert_eq!(fixture.matched_count, 0);

        // While active, checking empty must fail with active count error
        let err = check_active_remote_fixtures_empty().unwrap_err();
        assert!(err.contains("Store remote fixtures are still active"));

        // Match exact URL
        assert!(fixture.matches(target_url));
        assert_eq!(fixture.matched_count, 1);

        // Unmatched URL
        assert!(!fixture.matches("http://127.0.0.1:43123/other.bin"));
        assert_eq!(fixture.matched_count, 1);

        // Deactivating shuts off matching
        fixture.deactivate();
        assert!(!fixture.active);
        assert!(!fixture.matches(target_url));
    }

    // Dropped / deactivated scope cleans up atomically
    assert!(check_active_remote_fixtures_empty().is_ok());

    // 2. Test expect_saved_original_filename_case
    let temp = TempTestDir::new("orig-fn");
    let saved_with_orig = save_media_buffer(
        b"data with name",
        Some("text/plain"),
        "inbound",
        1024,
        Some("my-report.txt"),
        None,
        Some(&temp.path),
    ).expect("save with original filename");

    assert!(expect_saved_original_filename_case(
        &saved_with_orig,
        Some("my-report.txt"),
        false,
    ).is_ok());

    let saved_uuid_only = save_media_buffer(
        b"data no name",
        Some("text/plain"),
        "inbound",
        1024,
        None,
        None,
        Some(&temp.path),
    ).expect("save without original filename");

    assert!(expect_saved_original_filename_case(
        &saved_uuid_only,
        None,
        true,
    ).is_ok());
}
