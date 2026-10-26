use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

use mahayana_host_runtime::device_pairing_owner::DevicePairingOwner;
use mahayana_host_runtime::extensions::attachments::attachments_service::AttachmentsService;
use mahayana_host_runtime::extensions::attachments::media_store::{
    CleanOldMediaOptions, MEDIA_MAX_BYTES, PLAYBACK_TRANSCODE_SUBDIR, MANAGED_OUTGOING_SUBDIR,
    build_saved_media_id, clean_old_media, delete_media_buffer, extract_original_filename,
    prune_playback_transcode_cache_to_size, read_media_buffer, resolve_media_buffer_path,
    resolve_media_scoped_dir, resolve_media_subdir, sanitize_filename, save_media_buffer,
    save_media_source, write_playback_transcode_cache,
};
use mahayana_host_runtime::extensions::attachments::media_store_shared::{
    MEDIA_FILE_MODE, SaveMediaSourceErrorCode, format_media_size,
};
use serde_json::json;

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ocrm-media-store-test-{name}-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
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
