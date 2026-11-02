use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier, Mutex};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentTextPreview, AttachmentsBox, AttachmentsService, LINK_CACHE_DIRNAME,
    LINK_CACHE_TTL_MS, LINK_CACHE_VERSION, LinkMetadata, SandAttachmentError,
    ingest_attachment_bytes, persist_image_bytes,
};
use mahayana_host_runtime::extensions::attachments::extension::{
    ATTACHMENTS_DEPENDENCIES, ATTACHMENTS_EXTENSION_ID,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::{
    GenerateImageAuth, GenerateImageBackend, GeneratedImage, PersistGeneratedImage,
    PersistedImage, SandGenerateImageError, SandGenerateImageService,
};
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;

fn temp_dir(label: &str) -> PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "fabushi-attachments-service-{label}-{}-{suffix}",
        std::process::id()
    ))
}

struct TestAuth;

impl GenerateImageAuth for TestAuth {
    fn get_access_token(&self) -> Result<String, String> {
        Ok("token".into())
    }

    fn get_machine_id(&self) -> Result<String, String> {
        Ok("machine".into())
    }
}

#[derive(Default)]
struct TestBox {
    state: Mutex<String>,
    uploads: Mutex<Vec<(String, String, Vec<u8>)>>,
}

impl TestBox {
    fn running() -> Self {
        Self {
            state: Mutex::new("running".into()),
            uploads: Mutex::new(Vec::new()),
        }
    }
}

impl AttachmentsBox for TestBox {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        Ok(self.state.lock().expect("state").clone())
    }

    fn upload_file(&self, agent_id: &str, path: &str, data: &[u8]) -> Result<(), String> {
        self.uploads.lock().expect("uploads").push((
            agent_id.to_string(),
            path.to_string(),
            data.to_vec(),
        ));
        Ok(())
    }
}

fn png_2x3() -> Vec<u8> {
    let mut bytes = vec![0_u8; 24];
    bytes[..8].copy_from_slice(b"\x89PNG\r\n\x1a\n");
    bytes[12..16].copy_from_slice(b"IHDR");
    bytes[16..20].copy_from_slice(&2_u32.to_be_bytes());
    bytes[20..24].copy_from_slice(&3_u32.to_be_bytes());
    bytes
}

#[test]
fn attachment_extension_declares_frozen_identity_and_dependencies() {
    assert_eq!(ATTACHMENTS_EXTENSION_ID, HostExtensionId::Attachments);
    assert_eq!(
        ATTACHMENTS_DEPENDENCIES,
        &[
            HostExtensionId::Auth,
            HostExtensionId::ForeverBox,
            HostExtensionId::Telemetry,
        ]
    );
}

#[test]
fn service_upload_read_text_chunk_and_gateway_use_single_agent_media_store() {
    let root = temp_dir("io");
    let agent_dir = root.join("agents/agent-a");
    fs::create_dir_all(&agent_dir).expect("agent dir");

    let box_ = Arc::new(TestBox::running());
    let service = AttachmentsService::new(
        root.clone(),
        Arc::new(TestAuth),
        box_.clone(),
        None,
    );

    let encoded = STANDARD.encode(b"hello attachment");
    let uploaded = service
        .upload("notes.txt", Some(&encoded), Some("agent-a"))
        .expect("upload");
    assert!(uploaded.starts_with(agent_dir.join("attachments")));
    assert_eq!(fs::read(&uploaded).expect("uploaded bytes"), b"hello attachment");

    let duplicate = service
        .upload("notes.txt", Some(&encoded), Some("agent-a"))
        .expect("duplicate upload");
    assert_eq!(duplicate, uploaded, "content addressed upload must dedupe");

    assert_eq!(
        service.read_text(&uploaded, None),
        Some(AttachmentTextPreview::Text {
            text: "hello attachment".into(),
            truncated: false,
            bytes: 16,
        })
    );

    let chunk = service
        .read_chunk(&uploaded, None, 6, 10, false)
        .expect("chunk");
    assert_eq!(STANDARD.decode(chunk.bytes_base64).unwrap(), b"attachment");
    assert_eq!(chunk.total_size, 16);
    assert_eq!(chunk.mime, None);

    let gateway = service
        .dispatch_gateway(
            "readAttachmentChunk",
            &json!({
                "path": uploaded.to_string_lossy(),
                "offset": 0,
                "length": 5,
                "videoPlayback": false,
            }),
        )
        .expect("owned method")
        .expect("gateway result");
    assert_eq!(
        STANDARD.decode(gateway["bytesBase64"].as_str().unwrap()).unwrap(),
        b"hello"
    );
    assert_eq!(gateway["totalSize"], 16);

    assert!(service.dispatch_gateway("notAnAttachmentMethod", &json!({})).is_none());
    assert!(service
        .read_chunk(Path::new("/tmp/outside-fabushi-attachment.txt"), None, 0, 1, false)
        .is_none());

    let staged = service.stage_into_box("agent-a", std::slice::from_ref(&uploaded));
    let staged_name = uploaded.file_name().unwrap().to_string_lossy();
    let expected_box_path = format!("/workspace/uploads/{staged_name}");
    assert_eq!(
        staged.get(&uploaded).map(String::as_str),
        Some(expected_box_path.as_str())
    );
    let uploads = box_.uploads.lock().expect("uploads");
    assert_eq!(uploads.len(), 1);
    assert_eq!(uploads[0].0, "agent-a");
    assert_eq!(uploads[0].1, expected_box_path);
    assert_eq!(uploads[0].2, b"hello attachment");

    let _ = fs::remove_dir_all(root);
}

#[test]
fn image_reads_and_persistence_preserve_content_addressed_dimensions() {
    let root = temp_dir("image");
    let agent_dir = root.join("agents/agent-a");
    let assets_dir = agent_dir.join("assets");
    fs::create_dir_all(&assets_dir).expect("assets");

    let service = AttachmentsService::new(
        root.clone(),
        Arc::new(TestAuth),
        Arc::new(TestBox::running()),
        None,
    );
    let image = png_2x3();
    let persisted = service
        .persist_image_bytes(&assets_dir, &image, "image/png")
        .expect("persist image");
    assert!(persisted.absolute_path.starts_with(&assets_dir));
    assert!(persisted.file_url.starts_with("file:"));
    assert_eq!(persisted.bytes, image.len() as u64);
    assert_eq!(persisted.width, Some(2));
    assert_eq!(persisted.height, Some(3));

    let read = service.read_image(&persisted.absolute_path).expect("read image");
    assert!(read.data_url.starts_with("data:image/png;base64,"));
    assert_eq!(read.width, Some(2));
    assert_eq!(read.height, Some(3));

    let gateway = service
        .dispatch_gateway(
            "readAttachmentImage",
            &json!({ "path": persisted.absolute_path.to_string_lossy() }),
        )
        .expect("owned method")
        .expect("gateway result");
    assert_eq!(gateway["width"], 2);
    assert_eq!(gateway["height"], 3);

    let _ = fs::remove_dir_all(root);
}

#[test]
fn text_preview_marks_known_binary_extensions_and_embedded_nul_as_binary() {
    let root = temp_dir("binary");
    let agent_dir = root.join("agents/agent-a");
    let attachments = agent_dir.join("attachments");
    fs::create_dir_all(&attachments).expect("attachments");
    let binary = attachments.join("blob.bin");
    fs::write(&binary, b"abc").expect("binary");
    let nul_text = attachments.join("bad.txt");
    fs::write(&nul_text, b"abc\0def").expect("nul text");

    let service = AttachmentsService::new(
        root.clone(),
        Arc::new(TestAuth),
        Arc::new(TestBox::running()),
        None,
    );
    assert_eq!(
        service.read_text(&binary, None),
        Some(AttachmentTextPreview::Binary { bytes: 3 })
    );
    assert_eq!(
        service.read_text(&nul_text, None),
        Some(AttachmentTextPreview::Binary { bytes: 7 })
    );

    let _ = fs::remove_dir_all(root);
}

#[test]
fn gateway_link_metadata_rejects_mapped_private_cache_and_preserves_public_cache() {
    let root = temp_dir("mapped-ip-cache");
    let cache_dir = root.join("preview-cache");
    let records_dir = cache_dir.join(LINK_CACHE_DIRNAME);
    fs::create_dir_all(&records_dir).expect("link cache directory");
    let service = AttachmentsService::new(
        root.clone(),
        Arc::new(TestAuth),
        Arc::new(TestBox::running()),
        None,
    );

    for (label, raw_url, blocked) in [
        (
            "public-host",
            "https://PREVIEW.example.test:443/a/../public?fixture=host",
            false,
        ),
        (
            "public-mapped",
            "https://[::ffff:1.1.1.1]:443/a/../public?fixture=mapped",
            false,
        ),
        (
            "private-dotted",
            "https://[::ffff:127.0.0.1]/private?fixture=dotted",
            true,
        ),
        (
            "private-hex",
            "https://[::ffff:7f00:1]/private?fixture=hex",
            true,
        ),
        (
            "private-lan",
            "https://[::ffff:10.1.2.3]/private?fixture=lan",
            true,
        ),
    ] {
        // Ordinary URL canonicalization must not prequalify the fixture through
        // the admission policy being tested. Each case gets a canonical cache key.
        let canonical = url::Url::parse(raw_url).expect("cache URL fixture");
        let fetched_at = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("cache fixture clock")
                .as_millis(),
        )
        .expect("cache fixture timestamp");
        let metadata = LinkMetadata {
            url: canonical.to_string(),
            canonical_url: format!("{}#canonical-{label}", canonical.as_str()),
            title: format!("LPIP cached title {label}"),
            description: format!("Distinct cached description for {label}"),
            site_name: format!("LPIP site {label}"),
            hostname: canonical.host_str().expect("cache hostname").to_string(),
            image_data_url: Some(
                concat!(
                    "data:image/png;base64,",
                    "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8",
                    "AAwMCAO+aVBEAAAAASUVORK5CYII="
                )
                .to_string(),
            ),
            favicon_data_url: None,
            fetched_at,
        };
        let expected_metadata = serde_json::to_value(&metadata).expect("typed metadata JSON");
        let mut cache_record = expected_metadata.clone();
        cache_record
            .as_object_mut()
            .expect("metadata object")
            .insert("cacheVersion".into(), json!(LINK_CACHE_VERSION));
        let cache_bytes = serde_json::to_vec_pretty(&cache_record).expect("cache bytes");
        let cache_path = records_dir.join(format!(
            "{:x}.json",
            Sha256::digest(canonical.as_str().as_bytes())
        ));
        fs::write(&cache_path, &cache_bytes).expect("seed fresh preview cache");

        // Verify the written bytes, typed fields, version and freshness before
        // gateway dispatch can reach the real network fallback.
        let saved_bytes = fs::read(&cache_path).expect("read seeded cache");
        assert_eq!(saved_bytes, cache_bytes);
        let saved_record: Value = serde_json::from_slice(&saved_bytes).expect("valid cache JSON");
        assert_eq!(saved_record["cacheVersion"], json!(LINK_CACHE_VERSION));
        assert_eq!(saved_record["fetchedAt"], json!(fetched_at));
        assert_eq!(saved_record["canonicalUrl"], expected_metadata["canonicalUrl"]);
        let saved_metadata: LinkMetadata =
            serde_json::from_slice(&saved_bytes).expect("complete typed cache metadata");
        assert_eq!(saved_metadata, metadata);
        let now = u64::try_from(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("cache admission clock")
                .as_millis(),
        )
        .expect("cache admission timestamp");
        let age = now
            .checked_sub(saved_metadata.fetched_at)
            .expect("nonfuture cache timestamp");
        assert!(
            age < LINK_CACHE_TTL_MS,
            "fixture must be fresh before dispatch"
        );

        let result = service
            .dispatch_gateway(
                "fetchLinkMetadata",
                &json!({ "cacheDir": cache_dir.to_string_lossy(), "url": raw_url }),
            )
            .expect("real link metadata gateway method")
            .expect("link metadata gateway result");
        assert_eq!(
            fs::read(&cache_path).expect("cache remains readable"),
            cache_bytes,
            "gateway admission must not rewrite the {label} cache record"
        );
        if blocked {
            assert_eq!(
                result,
                Value::Null,
                "a fresh {label} cache record cannot authorize a private mapped preview"
            );
        } else {
            assert_eq!(
                result,
                expected_metadata,
                "the real gateway must return exact cached metadata for {label}"
            );
        }
    }

    let _ = fs::remove_dir_all(root);
}

struct PublicationFixture {
    root: PathBuf,
}

impl PublicationFixture {
    fn new(label: &str) -> Self {
        let root = temp_dir(label);
        fs::create_dir_all(root.join("agents/agent-a/attachments")).expect("attachment directory");
        Self { root }
    }

    fn service(&self) -> AttachmentsService {
        AttachmentsService::new(
            self.root.clone(),
            Arc::new(TestAuth),
            Arc::new(TestBox::running()),
            None,
        )
    }

    fn agent_dir(&self) -> PathBuf {
        self.root.join("agents/agent-a")
    }

    fn attachments_dir(&self) -> PathBuf {
        self.agent_dir().join("attachments")
    }

    fn target(&self, bytes: &[u8], extension: &str) -> PathBuf {
        self.attachments_dir().join(format!("{:x}.{extension}", Sha256::digest(bytes)))
    }
}

impl Drop for PublicationFixture {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.root);
    }
}

#[cfg(unix)]
struct RestorePermissions {
    path: PathBuf,
    original: fs::Permissions,
}

#[cfg(unix)]
impl RestorePermissions {
    fn set_mode(path: &Path, mode: u32) -> Self {
        use std::os::unix::fs::PermissionsExt;

        let restore = Self {
            path: path.to_path_buf(),
            original: fs::metadata(path).expect("original fixture permissions").permissions(),
        };
        fs::set_permissions(path, fs::Permissions::from_mode(mode)).expect("fixture permissions");
        restore
    }
}

#[cfg(unix)]
impl Drop for RestorePermissions {
    fn drop(&mut self) {
        let _ = fs::set_permissions(&self.path, self.original.clone());
    }
}

fn gateway_upload(service: &AttachmentsService, filename: &str, bytes: &[u8]) -> Result<Value, String> {
    service.dispatch_gateway(
        "uploadAttachment",
        &json!({
            "filename": filename,
            "bytesBase64": STANDARD.encode(bytes),
            "agentId": "agent-a",
        }),
    ).expect("registered upload method")
}

fn corrupt_winner_cases(bytes: &[u8]) -> Vec<(&'static str, Vec<u8>)> {
    let mut same_size = bytes.to_vec();
    same_size[0] ^= 0xff;
    let mut oversized = bytes.to_vec();
    oversized.push(0x7e);
    vec![
        ("empty", Vec::new()),
        ("truncated", bytes[..bytes.len() - 1].to_vec()),
        ("same-size", same_size),
        ("oversized", oversized),
    ]
}

fn generated_image_service(target_dir: &Path, image: &[u8]) -> SandGenerateImageService {
    // Only the backend response is controlled. The persistence callback uses
    // the same owner function as the App's generated-images callback.
    let image_data = STANDARD.encode(image);
    let backend: GenerateImageBackend = Arc::new(move |_, _| {
        Ok(GeneratedImage {
            image_data: image_data.clone(),
            mime_type: "image/png".into(),
        })
    });
    let target_dir = target_dir.to_path_buf();
    let persist: PersistGeneratedImage = Arc::new(move |bytes, mime_type| {
        let persisted = persist_image_bytes(&target_dir, bytes, mime_type)
            .map_err(|error| error.to_string())?;
        Ok(Some(PersistedImage {
            absolute_path: persisted.absolute_path.to_string_lossy().into_owned(),
        }))
    });
    SandGenerateImageService::new(backend, persist)
}

#[test]
fn gateway_upload_rejects_corrupt_content_addressed_winners_without_mutation() {
    let bytes = b"complete attachment payload for durable deduplication";
    for (label, corrupt) in corrupt_winner_cases(bytes) {
        let fixture = PublicationFixture::new(&format!("corrupt-{label}"));
        let target = fixture.target(bytes, "txt");
        fs::write(&target, &corrupt).expect("seed interrupted or corrupt object");
        let service = fixture.service();
        let error = gateway_upload(&service, "payload.TXT", bytes).expect_err("reject corrupt winner");
        assert!(!error.is_empty());
        assert!(!error.contains(std::str::from_utf8(bytes).unwrap()), "error must not expose bytes");
        assert_eq!(fs::read(&target).expect("retained corrupt object"), corrupt, "{label}");
        drop(service);

        let reopened = fixture.service();
        assert_eq!(
            gateway_upload(&reopened, "payload.TXT", bytes).expect_err("restart still rejects corruption"),
            error,
        );
        assert_eq!(fs::read(&target).expect("restart retained object"), corrupt, "{label}");
        assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 1);
    }
}

#[test]
fn gateway_upload_rejects_directory_winners_without_mutation() {
    let fixture = PublicationFixture::new("directory-winner");
    let bytes = b"expected attachment bytes";
    let target = fixture.target(bytes, "txt");
    fs::create_dir(&target).expect("directory at final path");
    let marker = target.join("retain.txt");
    fs::write(&marker, b"unrelated directory content").expect("directory marker");

    assert!(gateway_upload(&fixture.service(), "payload.txt", bytes).is_err());
    assert!(fs::symlink_metadata(&target).unwrap().is_dir());
    assert_eq!(fs::read(&marker).unwrap(), b"unrelated directory content");
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 1);
}

#[test]
#[cfg(any(unix, windows))]
fn gateway_upload_rejects_symlink_winners_without_following_them() {
    let bytes = b"valid bytes behind an invalid link";
    for dangling in [false, true] {
        let fixture = PublicationFixture::new(if dangling { "dangling-winner" } else { "link-winner" });
        let target = fixture.target(bytes, "txt");
        let outside = fixture.root.join("outside-attachments.txt");
        if !dangling {
            fs::write(&outside, bytes).expect("link target with exact expected bytes");
        }
        #[cfg(unix)]
        std::os::unix::fs::symlink(&outside, &target).expect("symlink fixture");
        #[cfg(windows)]
        std::os::windows::fs::symlink_file(&outside, &target)
            .expect("symlink fixture requires an enabled Windows symlink capability");

        assert!(gateway_upload(&fixture.service(), "payload.txt", bytes).is_err());
        assert!(fs::symlink_metadata(&target).unwrap().file_type().is_symlink());
        assert_eq!(fs::read_link(&target).unwrap(), outside);
        if dangling {
            assert!(!outside.exists(), "upload must not create the dangling link target");
        } else {
            assert_eq!(fs::read(&outside).unwrap(), bytes);
        }
        assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 1);
    }
}

#[test]
#[cfg(unix)]
fn gateway_upload_rejects_fifo_winners_without_opening_them() {
    use std::ffi::CString;
    use std::os::unix::ffi::OsStrExt;
    use std::os::unix::fs::FileTypeExt;

    let fixture = PublicationFixture::new("fifo-winner");
    let bytes = b"attachment bytes";
    let target = fixture.target(bytes, "txt");
    let c_path = CString::new(target.as_os_str().as_bytes()).expect("fixture path has no nul");
    // The CString remains alive for the syscall and the fixture owns this path.
    assert_eq!(unsafe { libc::mkfifo(c_path.as_ptr(), 0o600) }, 0, "create FIFO fixture");

    assert!(gateway_upload(&fixture.service(), "payload.txt", bytes).is_err());
    assert!(fs::symlink_metadata(&target).unwrap().file_type().is_fifo());
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 1);
}

#[test]
#[cfg(unix)]
fn gateway_upload_refuses_unreadable_objects_and_unwritable_destinations() {
    // Privileged execution can bypass Unix mode bits and cannot supply this
    // negative evidence. Fail setup instead of treating it as a skipped pass.
    assert_ne!(unsafe { libc::geteuid() }, 0, "permission fixtures require a non-root runner");

    let fixture = PublicationFixture::new("unreadable-winner");
    let bytes = b"valid object that becomes unreadable";
    let target = fixture.target(bytes, "txt");
    fs::write(&target, bytes).expect("valid pre-existing object");
    let foreign = fixture.attachments_dir().join(".attachment-foreign.tmp");
    fs::write(&foreign, b"retained foreign stage").expect("foreign stage");
    let service = fixture.service();
    let control = gateway_upload(&service, "permissions.txt", bytes).expect("readable positive control");

    let unreadable = RestorePermissions::set_mode(&target, 0o000);
    let error = gateway_upload(&service, "permissions.txt", bytes).expect_err("refuse unreadable existing object");
    assert!(!error.is_empty());
    assert_eq!(fs::symlink_metadata(&target).unwrap().len(), bytes.len() as u64);
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 2);
    drop(unreadable);
    assert_eq!(fs::read(&target).unwrap(), bytes);
    assert_eq!(fs::read(&foreign).unwrap(), b"retained foreign stage");
    assert_eq!(gateway_upload(&service, "permissions.txt", bytes).unwrap(), control);

    let fixture = PublicationFixture::new("unwritable-destination");
    let bytes = b"new object requires a writable destination";
    let target = fixture.target(bytes, "txt");
    let foreign = fixture.attachments_dir().join(".attachment-foreign.tmp");
    fs::write(&foreign, b"retained foreign stage").expect("foreign stage");
    let service = fixture.service();
    let unwritable = RestorePermissions::set_mode(&fixture.attachments_dir(), 0o500);
    let error = gateway_upload(&service, "permissions.txt", bytes).expect_err("refuse unwritable destination");
    assert!(!error.is_empty());
    assert!(!target.exists(), "failed staging must not leave a final object");
    assert_eq!(fs::read(&foreign).unwrap(), b"retained foreign stage");
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 1);
    drop(unwritable);

    let published = gateway_upload(&service, "permissions.txt", bytes).expect("writable positive control");
    assert_eq!(published, json!({ "path": target.to_string_lossy() }));
    assert_eq!(fs::read(&target).unwrap(), bytes);
    assert_eq!(fs::read(&foreign).unwrap(), b"retained foreign stage");
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 2);
}

#[test]
fn valid_legacy_winner_is_reused_without_changing_its_metadata() {
    let fixture = PublicationFixture::new("readonly-winner");
    let bytes = b"valid legacy attachment bytes";
    let target = fixture.target(bytes, "txt");
    fs::write(&target, bytes).expect("legacy object");
    let original_permissions = fs::metadata(&target).unwrap().permissions();
    let mut readonly = original_permissions.clone();
    readonly.set_readonly(true);
    fs::set_permissions(&target, readonly).expect("read-only legacy object");
    let before = fs::metadata(&target).unwrap();
    #[cfg(unix)]
    let original_directory_permissions = {
        use std::os::unix::fs::PermissionsExt;
        let original = fs::metadata(fixture.attachments_dir()).unwrap().permissions();
        let readonly_directory = fs::Permissions::from_mode(original.mode() & !0o222);
        fs::set_permissions(fixture.attachments_dir(), readonly_directory).unwrap();
        original
    };

    let result = gateway_upload(&fixture.service(), "legacy.TXT", bytes);
    let after = fs::metadata(&target).unwrap();
    #[cfg(unix)]
    fs::set_permissions(fixture.attachments_dir(), original_directory_permissions).unwrap();
    fs::set_permissions(&target, original_permissions).unwrap();

    assert_eq!(result.expect("reuse valid read-only legacy winner"), json!({ "path": target.to_string_lossy() }));
    assert_eq!(before.len(), after.len());
    assert_eq!(before.modified().unwrap(), after.modified().unwrap());
    assert_eq!(before.permissions().readonly(), after.permissions().readonly());
    #[cfg(unix)]
    {
        use std::os::unix::fs::MetadataExt;
        assert_eq!(before.dev(), after.dev());
        assert_eq!(before.ino(), after.ino());
        assert_eq!(before.mode(), after.mode());
    }
    assert_eq!(fs::read(&target).unwrap(), bytes);
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 1);
}

#[test]
#[cfg(unix)]
fn publication_preserves_default_file_and_directory_permissions() {
    use std::os::unix::fs::PermissionsExt;

    let fixture = PublicationFixture::new("default-permissions");
    let control_dir = fixture.root.join("default-control");
    fs::create_dir(&control_dir).expect("directory with default permissions");
    let control_file = control_dir.join("control.png");
    let image = png_2x3();
    fs::write(&control_file, &image).expect("file with legacy fs::write permissions");
    let published_dir = fixture.root.join("published");
    let published = persist_image_bytes(&published_dir, &image, "image/png").expect("publish image");

    assert_eq!(
        fs::metadata(&published_dir).unwrap().permissions().mode() & 0o7777,
        fs::metadata(&control_dir).unwrap().permissions().mode() & 0o7777,
    );
    assert_eq!(
        fs::metadata(&published.absolute_path).unwrap().permissions().mode() & 0o7777,
        fs::metadata(&control_file).unwrap().permissions().mode() & 0o7777,
    );
    assert_eq!(fs::read(published.absolute_path).unwrap(), image);
    assert_eq!(fs::read_dir(published_dir).unwrap().count(), 1);
}

#[test]
fn restart_ignores_orphan_staging_files_and_preserves_valid_resources() {
    let fixture = PublicationFixture::new("restart-orphan");
    let bytes = b"complete attachment after restart";
    let target = fixture.target(bytes, "txt");
    let orphan = fixture.attachments_dir().join(".attachment-00000000000000000000000000000000.tmp");
    fs::write(&orphan, &bytes[..7]).expect("interrupted unpublished staging fixture");
    assert!(!target.exists());

    let service = fixture.service();
    let first = gateway_upload(&service, "restart.txt", bytes).expect("publish despite unrelated orphan");
    assert_eq!(first, json!({ "path": target.to_string_lossy() }));
    assert_eq!(fs::read(&target).unwrap(), bytes);
    drop(service);

    let reopened = fixture.service();
    assert_eq!(gateway_upload(&reopened, "restart.txt", bytes).unwrap(), first);
    assert_eq!(fs::read(&orphan).unwrap(), &bytes[..7]);
    assert_eq!(
        reopened.read_text(&target, Some("agent-a")),
        Some(AttachmentTextPreview::Text {
            text: String::from_utf8(bytes.to_vec()).unwrap(),
            truncated: false,
            bytes: bytes.len() as u64,
        }),
    );
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 2);
}

#[test]
fn concurrent_same_content_uploads_return_one_complete_resource() {
    let fixture = PublicationFixture::new("concurrent-upload");
    let bytes = Arc::new(vec![0x51; 128 * 1024 + 29]);
    let target = fixture.target(&bytes, "txt");
    let ready = Arc::new(Barrier::new(4));
    let mut workers = Vec::new();
    for _ in 0..4 {
        let root = fixture.root.clone();
        let bytes = bytes.clone();
        let ready = ready.clone();
        workers.push(thread::spawn(move || {
            let service = AttachmentsService::new(
                root, Arc::new(TestAuth), Arc::new(TestBox::running()), None,
            );
            let encoded = STANDARD.encode(&*bytes);
            ready.wait();
            service.upload("same.TXT", Some(&encoded), Some("agent-a"))
        }));
    }
    let results: Vec<_> = workers.into_iter().map(|worker| worker.join()).collect();
    for result in results {
        assert_eq!(result.expect("upload thread").expect("complete upload"), target);
    }
    assert_eq!(fs::read(&target).unwrap(), *bytes);
    assert_eq!(fs::metadata(&target).unwrap().len(), bytes.len() as u64);
    assert_eq!(fs::read_dir(fixture.attachments_dir()).unwrap().count(), 1);

    let chunk = fixture.service().read_chunk(&target, Some("agent-a"), 64 * 1024, 31, false)
        .expect("actual chunk consumer");
    assert_eq!(STANDARD.decode(chunk.bytes_base64).unwrap(), bytes[64 * 1024..64 * 1024 + 31]);
    assert_eq!(chunk.total_size, bytes.len() as u64);
}

#[test]
fn generated_image_service_rejects_corrupt_winners_and_propagates_persistence_errors() {
    let image = png_2x3();
    for (label, corrupt) in corrupt_winner_cases(&image) {
        let fixture = PublicationFixture::new(&format!("generated-corrupt-{label}"));
        let generated_dir = fixture.agent_dir().join("generated-images");
        fs::create_dir(&generated_dir).expect("generated image directory");
        let target = generated_dir.join(format!("{:x}.png", Sha256::digest(&image)));
        fs::write(&target, &corrupt).expect("seed corrupt generated resource");
        let persistence_error = fixture.service()
            .persist_image_bytes(&generated_dir, &image, "image/png")
            .expect_err("attachment owner rejects corrupt image");
        let generator = generated_image_service(&generated_dir, &image);

        assert_eq!(
            generator.generate("controlled image fixture", &[]),
            Err(SandGenerateImageError::Generate(persistence_error.to_string())),
        );
        assert_eq!(fs::read(&target).unwrap(), corrupt, "{label}");
        assert_eq!(fs::read_dir(generated_dir).unwrap().count(), 1);
    }
}

#[test]
fn generated_image_service_returns_persisted_bytes_to_image_consumer() {
    let fixture = PublicationFixture::new("generated-consumer");
    let generated_dir = fixture.agent_dir().join("generated-images");
    let image = png_2x3();
    let generator = generated_image_service(&generated_dir, &image);
    let (path, encoded) = generator.generate("controlled image fixture", &[])
        .expect("generate and persist through real callback");
    let path = PathBuf::from(path);
    assert_eq!(path, generated_dir.join(format!("{:x}.png", Sha256::digest(&image))));
    assert_eq!(fs::read(&path).unwrap(), image);
    assert_eq!(STANDARD.decode(encoded).unwrap(), image);

    let service = fixture.service();
    let persisted = service.persist_image_bytes(&generated_dir, &image, "image/png")
        .expect("reuse actual generated resource");
    assert_eq!(persisted.absolute_path, path);
    assert!(persisted.file_url.starts_with("file:"));
    assert_eq!(persisted.bytes, image.len() as u64);
    assert_eq!(persisted.width, Some(2));
    assert_eq!(persisted.height, Some(3));
    let read = service.read_image(&path).expect("actual image consumer");
    assert_eq!(read.data_url, format!("data:image/png;base64,{}", STANDARD.encode(&image)));
    assert_eq!(read.width, Some(2));
    assert_eq!(read.height, Some(3));
    let gateway = service.dispatch_gateway(
        "readAttachmentImage", &json!({ "path": path.to_string_lossy() }),
    ).expect("registered image consumer").expect("image consumer result");
    assert_eq!(gateway["dataUrl"], read.data_url);
    assert_eq!(gateway["width"], 2);
    assert_eq!(gateway["height"], 3);
    assert_eq!(fs::read_dir(generated_dir).unwrap().count(), 1);
}

#[test]
fn empty_image_and_attachment_extension_behavior_remain_unchanged() {
    let fixture = PublicationFixture::new("legacy-input-behavior");
    let generated_dir = fixture.agent_dir().join("generated-images");
    let empty = persist_image_bytes(&generated_dir, b"", "image/unknown")
        .expect("legacy empty-image persistence remains valid");
    assert_eq!(empty.absolute_path, generated_dir.join(format!("{:x}.png", Sha256::digest(b""))));
    assert_eq!(empty.bytes, 0);
    assert_eq!(empty.width, None);
    assert_eq!(empty.height, None);
    assert!(fs::read(&empty.absolute_path).unwrap().is_empty());
    assert_eq!(persist_image_bytes(&generated_dir, b"", "image/unknown").unwrap(), empty);

    let agent_dir = fixture.agent_dir();
    assert_eq!(ingest_attachment_bytes(&agent_dir, "empty.txt", b""), Err(SandAttachmentError::EmptyAttachment));
    assert_eq!(ingest_attachment_bytes(&agent_dir, "  ", b"bytes"), Err(SandAttachmentError::EmptyFilename));
    let bytes = b"unchanged canonical filename bytes";
    let expected_hash = format!("{:x}", Sha256::digest(bytes));
    for (filename, extension) in [("UPPER.TXT", "txt"), ("no-extension", "bin")] {
        let result = ingest_attachment_bytes(&agent_dir, filename, bytes).expect("ingest canonical bytes");
        assert_eq!(result.hash, expected_hash);
        assert_eq!(result.bytes, bytes.len() as u64);
        assert_eq!(result.absolute_path, fixture.target(bytes, extension));
        assert_eq!(fs::read(result.absolute_path).unwrap(), bytes);
    }
}
