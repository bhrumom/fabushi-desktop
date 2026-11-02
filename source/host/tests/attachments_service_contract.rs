use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};

use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentTextPreview, AttachmentsBox, AttachmentsService, LINK_CACHE_DIRNAME,
    LINK_CACHE_TTL_MS, LINK_CACHE_VERSION, LinkMetadata,
};
use mahayana_host_runtime::extensions::attachments::extension::{
    ATTACHMENTS_DEPENDENCIES, ATTACHMENTS_EXTENSION_ID,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
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
