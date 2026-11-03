use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use serde_json::json;

use mahayana_host_runtime::extensions::attachments::anthropic_inline_images::{
    normalize_anthropic_inline_content_blocks, AnthropicContentBlock,
};
use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentsBox, AttachmentsService,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
use mahayana_host_runtime::extensions::attachments::host_read_media_policy::HOST_READ_DECLARED_TEXT_ERROR;
use mahayana_host_runtime::extensions::attachments::remote_workspace_file::{
    read_bounded_remote_workspace_file, RemoteWorkspaceFileError, RemoteWorkspaceFileParams,
};
use mahayana_host_runtime::extensions::attachments::web_media::{
    assert_no_windows_network_path, file_url_to_path, load_web_media,
    optimize_image_buffer_for_web_media, strip_legacy_media_directive_prefix,
    WebMediaError, WebMediaOptions,
};

const WEB_MEDIA_SOURCE: &str = include_str!("../src/extensions/attachments/web_media.rs");

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ocrm-web-media-test-{name}-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
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
struct TestBox;
impl AttachmentsBox for TestBox {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        Ok("running".into())
    }
    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

fn test_attachments_service(temp_dir: &std::path::Path) -> AttachmentsService {
    AttachmentsService::new(
        temp_dir.to_path_buf(),
        Arc::new(TestAuth),
        Arc::new(TestBox::default()),
        None,
    )
}

#[test]
fn test_ocrm_anthropic_inline_content_blocks_normalization() {
    // 1. Text blocks preserved
    let blocks = vec![
        AnthropicContentBlock::Text {
            text: "Hello Anthropic".to_string(),
        },
        AnthropicContentBlock::Image {
            // One red pixel uncompressed 24-bit BMP
            data: "Qk06AAAAAAAAADYAAAAoAAAAAQAAAAEAAAABABgAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAD/AA==".to_string(),
            mime_type: "image/bmp".to_string(),
        },
    ];

    let normalized = normalize_anthropic_inline_content_blocks(&blocks)
        .expect("normalize content blocks succeeded");

    assert_eq!(normalized.len(), 2);
    match &normalized[0] {
        AnthropicContentBlock::Text { text } => assert_eq!(text, "Hello Anthropic"),
        _ => panic!("Expected text block"),
    }
    match &normalized[1] {
        AnthropicContentBlock::Image { data, mime_type } => {
            assert_eq!(mime_type, "image/png");
            use base64::Engine as _;
            let png_bytes = base64::engine::general_purpose::STANDARD
                .decode(data)
                .expect("valid png base64");
            assert_eq!(&png_bytes[0..8], b"\x89PNG\r\n\x1a\n");
        }
        _ => panic!("Expected image block"),
    }

    // 2. Reject oversized input (> 10MB) before decode
    let huge_data = "A".repeat(14 * 1024 * 1024);
    let oversized_blocks = vec![AnthropicContentBlock::Image {
        data: huge_data,
        mime_type: "image/jpeg".to_string(),
    }];
    let err = normalize_anthropic_inline_content_blocks(&oversized_blocks).unwrap_err();
    assert!(err.contains("10 MB decoded safety limit"));
}

#[test]
fn test_ocrm_web_media_animated_webp_preservation() {
    // Valid animated WebP buffer with VP8X animation flag bit set
    let mut webp = Vec::new();
    webp.extend_from_slice(b"RIFF");
    webp.extend_from_slice(&32u32.to_le_bytes());
    webp.extend_from_slice(b"WEBP");
    webp.extend_from_slice(b"VP8X");
    webp.extend_from_slice(&10u32.to_le_bytes());
    webp.push(0x02); // animation bit 1 set
    webp.extend_from_slice(&[0u8; 15]);

    let res = optimize_image_buffer_for_web_media(
        webp.clone(),
        Some("image/webp"),
        Some("animated.webp"),
        Some(1024),
        None,
    )
    .expect("animated WebP preserved");

    assert_eq!(res.content_type.as_deref(), Some("image/webp"));
    assert_eq!(res.kind.as_deref(), Some("image"));
    assert_eq!(res.buffer, webp);

    // Over-budget animated WebP rejection
    let over_budget_err = optimize_image_buffer_for_web_media(
        webp.clone(),
        Some("image/webp"),
        Some("animated.webp"),
        Some(10), // cap = 10 bytes, smaller than buffer length
        None,
    )
    .unwrap_err();

    match over_budget_err {
        mahayana_host_runtime::extensions::attachments::web_media::WebMediaError::OptimizationLimit(e) => {
            assert!(e.message.contains("Animated WebP"));
            assert_eq!(e.max_bytes, 10);
        }
        other => panic!("Expected OptimizationLimit error, got: {other:?}"),
    }
}

#[test]
fn test_ocrm_web_media_file_url_and_traversal_guards() {
    // 1. strip_legacy_media_directive_prefix
    assert_eq!(
        strip_legacy_media_directive_prefix("MEDIA: file:///path/to/img.png"),
        "file:///path/to/img.png"
    );
    assert_eq!(
        strip_legacy_media_directive_prefix("media://inbound/asset-123.jpg"),
        "media://inbound/asset-123.jpg"
    );

    // 2. assert_no_windows_network_path
    assert!(assert_no_windows_network_path(r"\\attacker\share\evil.png", "test").is_err());
    assert!(assert_no_windows_network_path("//attacker/share/evil.png", "test").is_err());
    assert!(assert_no_windows_network_path("/safe/path/img.png", "test").is_ok());

    // 3. file_url_to_path
    let native_path = std::env::temp_dir().join("café image.png");
    let native_url = url::Url::from_file_path(&native_path).expect("native absolute URL");
    let single_slash_url = format!("FILE:{}", native_url.as_str().strip_prefix("file://").unwrap());
    let decoded_path = file_url_to_path(&single_slash_url)
        .expect("decode file URL");
    assert_eq!(decoded_path, native_path);

    // Traversal rejection in file URL
    assert!(file_url_to_path("FILE:///C:/safe/folder%2Fsecret.png").is_err());
    assert!(file_url_to_path("FILE:///C:/safe/folder%5Csecret.png").is_err());
    assert!(file_url_to_path("FILE:////attacker/share/evil.png").is_err());
}

#[test]
fn test_ocrm_web_media_file_url_rejects_nonlocal_and_invalid_references() {
    for reference in [
        "file://files.example/share/image.png",
        "FiLe://localhost.example/share/image.png",
        "FILE://127.0.0.1/share/image.png",
        "file://[::1]/share/image.png",
        "file://localhost./share/image.png",
        "file://[broken/share/image.png",
        "file://user:secret@localhost/share/image.png",
        "file://localhost:8080/share/image.png",
        "FILE:////files.example/share/image.png",
        "file:////C:/safe/image.png",
        r"file:/\/\C:/safe/image.png",
        r"file:\/\\C:/safe/image.png",
        r"file:\\\/C:/safe/image.png",
        "file://localhost//C:/safe/image.png",
        r"FiLe://LOCALHOST/\C:/safe/image.png",
        r"file:\\localhost\/C:\safe\image.png",
        "file:/\t/\r/\n/C:/safe/image.png",
        "file://local\thost/\r/\nC:/safe/image.png",
        "file://files.example/C:/safe/image.png",
        "FiLe://localhost.example/C:/safe/image.png",
        "FILE://127.0.0.1/C:/safe/image.png",
        "file://[::1]/C:/safe/image.png",
        "file://files.exa\tmple/C:/safe/image.png",
        "file://files.example/C|/safe/image.png",
        r"file:\\files.example\C:\safe\image.png",
        "file:///C:/safe/folder%2fimage.png",
        "FILE:///C:/safe/folder%5Cimage.png",
        "https://localhost/image.png",
    ] {
        match file_url_to_path(reference).expect_err("invalid reference must not become a path") {
            WebMediaError::LocalAccess { code, .. } => assert_eq!(code, "invalid-file-url"),
            other => panic!("Expected invalid-file-url for {reference:?}, got {other:?}"),
        }
    }
    #[cfg(windows)]
    for reference in [
        "file:///drive-less/image.png",
        "file:",
        "file:/",
        "file://localhost/",
        "file:C:",
        "file:/C:",
        "file://C:",
        "file:///C:",
        "file:///C|",
        "file:///C%3A",
        "file:///C%3a",
        "FiLe://localhost/C:",
        "file:///C:?ignored=/",
        "file:///C:#ignored=/",
    ] {
        match file_url_to_path(reference).expect_err("native conversion failure") {
            WebMediaError::LocalAccess { code, .. } => assert_eq!(code, "invalid-file-url"),
            other => panic!("Expected invalid-file-url for {reference:?}, got {other:?}"),
        }
    }
    #[cfg(windows)]
    for reference in ["file:///C:/", "file://C:/", "file:///C|/", "file:///C%3A/"] {
        assert_eq!(file_url_to_path(reference).expect("rooted drive remains valid"), PathBuf::from(r"C:\"));
    }
}

#[test]
fn test_ocrm_web_media_file_url_native_bytes_service_and_cap_recovery() {
    let temp = TempTestDir::new("file-url-native");
    let service = test_attachments_service(&temp.path);
    let name = "café 100% report.html";
    let path = temp.path.join(name);
    let bytes = b"<!doctype html><html><body>native URL bytes</body></html>";
    fs::write(&path, bytes).unwrap();
    let native_url = url::Url::from_file_path(&path).expect("native fixture URL");
    let references = [
        native_url.to_string(),
        format!("FILE:{}", native_url.as_str().strip_prefix("file://").unwrap()),
        format!("fIlE:{}", native_url.as_str().strip_prefix("file:").unwrap()),
        format!("file://localhost{}", native_url.path()),
        format!("file://local\thost{}", native_url.path()),
        format!("FILE:{}", native_url.path().replace('/', r"\")),
        format!("file:///{}", native_url.path().trim_start_matches('/').replacen('/', "//", 1)),
    ].into_iter();
    #[cfg(windows)]
    let references = references.chain([
        format!("file://{}", native_url.path().trim_start_matches('/')),
        format!("file://{}", native_url.path().trim_start_matches('/').replacen('/', "//", 1)),
    ]);
    for reference in references.flat_map(|reference| {
        let with_components = format!("{reference}?next=%2F%5C#part=%5C%2F");
        [reference, with_components]
    }) {
        assert_eq!(file_url_to_path(&reference).unwrap(), path);
        let mut options = WebMediaOptions {
            max_bytes: Some(bytes.len()),
            local_roots: Some(vec![temp.path.clone()]),
            optimize_images: true,
            html_publication: Some("explicit".to_string()),
            ..Default::default()
        };
        let loaded = load_web_media(&reference, &options).expect("native local URL inside allowed root");
        assert_eq!(loaded.buffer, bytes);
        assert_eq!(loaded.file_name.as_deref(), Some(name));
        assert_eq!(loaded.content_type.as_deref(), Some("text/html"));
        assert_explicit_html_loads(&service, &reference, bytes, name);

        options.max_bytes = Some(bytes.len() - 1);
        assert!(matches!(load_web_media(&reference, &options), Err(WebMediaError::TooLarge(_))));
        let error = service.dispatch_gateway("loadWebMedia", &json!({
            "url": reference, "maxBytes": bytes.len() - 1, "htmlPublication": "explicit"
        })).expect("service handles loader").expect_err("service retains byte cap");
        assert!(error.starts_with("Media exceeds "), "{error}");
        assert_explicit_html_loads(&service, &reference, bytes, name);
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn test_ocrm_web_media_file_url_service_refusal_keeps_public_admission_closed() {
    let temp = TempTestDir::new("file-url-service-refusal");
    let service = test_attachments_service(&temp.path);
    // These fail URL parsing without any possible remote native path conversion.
    // Remote Windows authority controls use the private no-I/O boundary instead.
    for reference in ["file://[broken/share/image.png", "file://user:secret@localhost/share/image.png"] {
        let error = service.dispatch_gateway("loadWebMedia", &json!({ "url": reference }))
            .expect("direct service arm exists").expect_err("service propagates invalid URL");
        assert!(error.starts_with("Local media access error [invalid-file-url]: "), "{error}");
        assert!(!error.contains("secret"), "refusal must not echo URL credentials");
    }
    // This checks the real finite registry, not an HTTP or renderer execution.
    assert!(!mahayana_host_runtime::gateway_protocol::is_grok_gateway_command("loadWebMedia"));
    let path = temp.path.join("recovered.html");
    let bytes = b"<!doctype html><html>valid after refusal</html>";
    fs::write(&path, bytes).unwrap();
    let reference = url::Url::from_file_path(&path).unwrap();
    assert_explicit_html_loads(&service, reference.as_str(), bytes, "recovered.html");
}

#[cfg(unix)]
#[test]
fn test_ocrm_web_media_file_url_authority_alias_is_not_read() {
    let temp = TempTestDir::new("file-url-alias");
    let service = test_attachments_service(&temp.path);
    let path = temp.path.join("alias.html");
    let bytes = b"<!doctype html><html>owned alias bytes</html>";
    fs::write(&path, bytes).unwrap();
    let local = url::Url::from_file_path(&path).unwrap();
    let alias = format!("file://{}", local.path().trim_start_matches('/'));
    assert!(url::Url::parse(&alias).unwrap().host_str().is_some());
    for sandbox_validated in [false, true] {
        let options = WebMediaOptions {
            local_roots: Some(vec![temp.path.clone()]),
            sandbox_validated,
            html_publication: Some("explicit".to_string()),
            ..Default::default()
        };
        match load_web_media(&alias, &options).expect_err("remote authority must not alias an allowed local file") {
            WebMediaError::LocalAccess { code, .. } => assert_eq!(code, "invalid-file-url"),
            other => panic!("Expected invalid-file-url, got {other:?}"),
        }
        assert_eq!(load_web_media(local.as_str(), &options).unwrap().buffer, bytes);
    }
    let error = service.dispatch_gateway("loadWebMedia", &json!({ "url": alias, "htmlPublication": "explicit" }))
        .expect("service handles loader").expect_err("service refuses authority alias");
    assert!(error.starts_with("Local media access error [invalid-file-url]: "), "{error}");
    assert_explicit_html_loads(&service, local.as_str(), bytes, "alias.html");
    assert_eq!(fs::read(&path).unwrap(), bytes);
}

#[test]
fn test_ocrm_web_media_local_loading_and_roots_boundary() {
    let temp = TempTestDir::new("web-media-roots");
    let allowed_dir = temp.path.join("allowed");
    let forbidden_dir = temp.path.join("forbidden");
    fs::create_dir_all(&allowed_dir).unwrap();
    fs::create_dir_all(&forbidden_dir).unwrap();

    let safe_file = allowed_dir.join("test.png");
    // Minimal 1x1 PNG bytes
    let png_bytes = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
        0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
        0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    fs::write(&safe_file, png_bytes).unwrap();

    let forbidden_file = forbidden_dir.join("secret.png");
    fs::write(&forbidden_file, png_bytes).unwrap();

    let opts = WebMediaOptions {
        local_roots: Some(vec![allowed_dir.clone()]),
        optimize_images: false,
        ..Default::default()
    };

    // Allowed read succeeds
    let res = load_web_media(&safe_file.to_string_lossy(), &opts).expect("load safe file");
    assert_eq!(res.buffer, png_bytes);

    // Forbidden read outside allowed local_roots fails with path-not-allowed
    let err = load_web_media(&forbidden_file.to_string_lossy(), &opts).unwrap_err();
    match err {
        mahayana_host_runtime::extensions::attachments::web_media::WebMediaError::LocalAccess { code, .. } => {
            assert_eq!(code, "path-not-allowed");
        }
        other => panic!("Expected path-not-allowed LocalAccess, got: {other:?}"),
    }
}

#[test]
fn web_media_local_reads_are_bounded_before_buffer_acceptance() {
    assert!(WEB_MEDIA_SOURCE.contains("read_outbound_media_file_bounded("));
    assert!(!WEB_MEDIA_SOURCE.contains("let buffer = fs::read(&file_path)"));
}

#[test]
fn test_ocrm_remote_workspace_file_transfer() {
    let temp = TempTestDir::new("remote-ws");
    let test_file = temp.path.join("data.bin");
    let data = b"Chunked remote workspace file transfer verification data bytes!";
    fs::write(&test_file, data).unwrap();

    let params = RemoteWorkspaceFileParams {
        path: test_file.clone(),
        max_bytes: 1024,
        workspace_root: Some(temp.path.clone()),
        timeout_ms: Some(5000),
    };

    // Transfer with small cap (1024) to exercise chunking
    let transferred = read_bounded_remote_workspace_file(&params, 1024, || Ok(()))
        .expect("remote transfer succeeded");
    assert_eq!(transferred, data);

    // Reject oversized file
    let small_cap_params = RemoteWorkspaceFileParams {
        path: test_file.clone(),
        max_bytes: 10,
        workspace_root: Some(temp.path.clone()),
        timeout_ms: Some(5000),
    };
    let err = read_bounded_remote_workspace_file(&small_cap_params, 1024, || Ok(())).unwrap_err();
    match err {
        RemoteWorkspaceFileError::FileTooLarge(limit) => assert_eq!(limit, 10),
        other => panic!("Expected FileTooLarge, got: {other:?}"),
    }

    // Closed owner cancels transfer
    let closed_err = read_bounded_remote_workspace_file(&params, 1024, || {
        Err(RemoteWorkspaceFileError::OwnerClosed)
    })
    .unwrap_err();
    match closed_err {
        RemoteWorkspaceFileError::OwnerClosed => {}
        other => panic!("Expected OwnerClosed, got: {other:?}"),
    }
}

#[test]
fn test_ocrm_web_media_gateway_service_dispatch() {
    let temp = TempTestDir::new("gateway-web");
    let service = test_attachments_service(&temp.path);

    // Minimal 1x1 PNG file in temp dir
    let img_path = temp.path.join("dot.png");
    let png_bytes = &[
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
        0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
        0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    fs::write(&img_path, png_bytes).unwrap();

    let res = service
        .dispatch_gateway(
            "loadWebMedia",
            &json!({
                "url": img_path.to_string_lossy(),
                "optimizeImages": false
            }),
        )
        .expect("dispatch handled")
        .expect("result ok");

    assert_eq!(res["contentType"], "image/png");
    assert_eq!(res["fileName"], "dot.png");

    // Gateway normalizeAnthropicInlineContentBlocks
    let blocks_res = service
        .dispatch_gateway(
            "normalizeAnthropicInlineContentBlocks",
            &json!({
                "blocks": [
                    { "type": "text", "text": "ping" }
                ]
            }),
        )
        .expect("dispatch handled")
        .expect("result ok");

    assert_eq!(blocks_res, json!([{ "type": "text", "text": "ping" }]));
}

fn assert_explicit_html_loads(
    service: &AttachmentsService,
    url: &str,
    bytes: &[u8],
    file_name: &str,
) {
    let options = WebMediaOptions {
        max_bytes: Some(bytes.len()),
        optimize_images: true,
        html_publication: Some("explicit".to_string()),
        ..Default::default()
    };
    let expected = json!({
        "buffer": bytes,
        "contentType": "text/html",
        "kind": "document",
        "fileName": file_name,
    });
    let loaded = load_web_media(url, &options).expect("explicit HTML loader result");
    assert_eq!(serde_json::to_value(loaded).unwrap(), expected);

    let dispatched = service
        .dispatch_gateway(
            "loadWebMedia",
            &json!({
                "url": url,
                "maxBytes": bytes.len(),
                "htmlPublication": "explicit",
            }),
        )
        .expect("loadWebMedia is handled")
        .expect("explicit HTML service result");
    // Exact object equality also requires generated trust to stay absent.
    assert_eq!(dispatched, expected);
}

fn assert_explicit_html_rejected(service: &AttachmentsService, url: &str) {
    let options = WebMediaOptions {
        optimize_images: true,
        html_publication: Some("explicit".to_string()),
        ..Default::default()
    };
    let policy_message = format!("HostReadMediaTypeError: {HOST_READ_DECLARED_TEXT_ERROR}");
    let error = load_web_media(url, &options).expect_err("invalid explicit HTML must fail");
    match error {
        WebMediaError::LocalAccess { code, message } => {
            assert_eq!(code, "invalid-html-publication");
            assert_eq!(message, policy_message);
        }
        other => panic!("Expected an HTML policy refusal, got: {other:?}"),
    }

    let error = service
        .dispatch_gateway(
            "loadWebMedia",
            &json!({ "url": url, "htmlPublication": "explicit" }),
        )
        .expect("loadWebMedia is handled")
        .expect_err("service must preserve the HTML policy refusal");
    assert_eq!(
        error,
        format!("Local media access error [invalid-html-publication]: {policy_message}")
    );
}

#[test]
fn test_ocrm_web_media_explicit_html_loader_and_service_preserve_bytes() {
    let temp = TempTestDir::new("explicit-html-success");
    let service = test_attachments_service(&temp.path);
    let cases = [
        ("report.html", "report.html", "<!doctype html><html><body>report</body></html>"),
        ("report café.HTM", "report café.HTM", "<HTML lang=\"en\"><body>café</body></HTML>"),
        (
            "report---123e4567-e89b-42d3-a456-426614174000.HTML",
            "report.HTML",
            "\u{feff}\u{00a0}<!DOCTYPE\u{00a0}HtMl><p>original bytes</p>",
        ),
    ];
    for (stored_name, original_name, html) in cases {
        let path = temp.path.join(stored_name);
        fs::write(&path, html.as_bytes()).expect("write HTML fixture");
        assert_explicit_html_loads(&service, &path.to_string_lossy(), html.as_bytes(), original_name);

        let file_url = url::Url::from_file_path(&path).expect("owned file URL");
        let uppercase_file_url = format!(
            "FILE:{}",
            file_url.as_str().strip_prefix("file:").expect("file scheme")
        );
        assert_explicit_html_loads(&service, &uppercase_file_url, html.as_bytes(), original_name);
    }
}

#[test]
fn test_ocrm_web_media_explicit_html_unicode_byte_boundary() {
    let temp = TempTestDir::new("explicit-html-byte-boundary");
    let service = test_attachments_service(&temp.path);
    for (label, scalar) in [("two-byte", "é"), ("three-byte", "界"), ("four-byte", "😀")] {
        let html = format!("<html>{}{scalar}</html>", "x".repeat(8191 - "<html>".len()));
        assert!(!html.is_char_boundary(8192), "fixture must cross byte 8192");
        let file_name = format!("{label}.html");
        let path = temp.path.join(&file_name);
        fs::write(&path, html.as_bytes()).expect("write Unicode HTML fixture");
        assert_explicit_html_loads(&service, &path.to_string_lossy(), html.as_bytes(), &file_name);
    }
}

#[test]
fn test_ocrm_web_media_explicit_html_utf16_shape_window() {
    let temp = TempTestDir::new("explicit-html-utf16-window");
    let service = test_attachments_service(&temp.path);
    // No opening HTML header: only a complete closing token inside the window admits these.
    let cases = [
        ("bmp-at-limit", format!("\u{feff}\u{00a0}{}</html>", "界".repeat(8185)), true),
        ("bmp-after-limit", format!("{}</html>", "界".repeat(8186)), false),
        ("pair-at-limit", format!("{}x</body>", "😀".repeat(4092)), true),
        ("pair-after-limit", format!("{}</body>", "😀".repeat(4093)), false),
        ("split-pair-at-limit", format!("{}😀</html>", "x".repeat(8191)), false),
    ];
    for (label, html, allowed) in cases {
        let file_name = format!("{label}.htm");
        let path = temp.path.join(&file_name);
        fs::write(&path, html.as_bytes()).expect("write UTF-16 window fixture");
        if allowed {
            assert_explicit_html_loads(&service, &path.to_string_lossy(), html.as_bytes(), &file_name);
        } else {
            assert_explicit_html_rejected(&service, &path.to_string_lossy());
        }
    }
}

#[test]
fn test_ocrm_web_media_explicit_html_rejects_malformed_and_nontext() {
    let temp = TempTestDir::new("explicit-html-invalid");
    let service = test_attachments_service(&temp.path);
    let mut invalid_utf8 = b"<html>".to_vec();
    invalid_utf8.extend_from_slice(&[0xff; 64]);
    invalid_utf8.extend_from_slice(b"</html>");
    let mut nul_text = b"<html>".to_vec();
    nul_text.extend_from_slice(&[0; 64]);
    nul_text.extend_from_slice(b"</html>");
    let cases = [
        ("empty", Vec::new()),
        ("ordinary-text", b"a,b,c\nthis is not an HTML document\n".to_vec()),
        ("header-word", b"<htmlish><p>no closing document token</p>".to_vec()),
        ("header-underscore", b"<html_><p>no closing document token</p>".to_vec()),
        ("doctype-digit", b"<!doctype html5><p>no closing document token</p>".to_vec()),
        ("doctype-no-space", b"<!doctypehtml><p>no closing document token</p>".to_vec()),
        ("header-long-s", "<htmlſ><p>no closing document token</p>".as_bytes().to_vec()),
        ("header-kelvin", "<htmlK><p>no closing document token</p>".as_bytes().to_vec()),
        ("non-ecmascript-space", "\u{0085}<html><p>this leading character is not ECMAScript whitespace</p>".as_bytes().to_vec()),
        ("invalid-utf8", invalid_utf8),
        ("nul-text", nul_text),
    ];
    for (label, bytes) in cases {
        let path = temp.path.join(format!("{label}.html"));
        fs::write(&path, bytes).expect("write invalid HTML fixture");
        assert_explicit_html_rejected(&service, &path.to_string_lossy());
    }
}

#[test]
fn test_ocrm_web_media_explicit_html_rejects_binary_signature_conflicts() {
    let temp = TempTestDir::new("explicit-html-signatures");
    let service = test_attachments_service(&temp.path);
    let signatures: [(&str, &[u8]); 3] = [
        ("png", b"\x89PNG\r\n\x1a\n"),
        ("pdf", b"%PDF-1.7\n"),
        ("zip", b"PK\x03\x04"),
    ];
    for (label, signature) in signatures {
        let mut bytes = signature.to_vec();
        // PDF is printable; ZIP's two control bytes are below the text threshold.
        // Both otherwise satisfy the closing-tag alternative, so byte sniffing matters.
        bytes.extend_from_slice("x".repeat(256).as_bytes());
        bytes.extend_from_slice(b"</body>");
        let path = temp.path.join(format!("{label}.HTML"));
        fs::write(&path, bytes).expect("write conflicting signature fixture");
        assert_explicit_html_rejected(&service, &path.to_string_lossy());
    }
}

#[test]
fn test_ocrm_web_media_html_publication_requires_exact_mode() {
    let temp = TempTestDir::new("explicit-html-mode");
    let service = test_attachments_service(&temp.path);
    let path = temp.path.join("ordinary.html");
    let bytes = b"plain text retains the ordinary loader behavior";
    fs::write(&path, bytes).expect("write ordinary loader fixture");
    let expected = json!({ "buffer": bytes.as_slice(), "kind": "image", "fileName": "ordinary.html" });
    for publication in [None, Some(""), Some("EXPLICIT"), Some(" explicit"), Some("explicit "), Some("generated")] {
        let options = WebMediaOptions {
            optimize_images: true,
            html_publication: publication.map(str::to_string),
            ..Default::default()
        };
        let loaded = load_web_media(&path.to_string_lossy(), &options).expect("ordinary mode remains unchanged");
        assert_eq!(serde_json::to_value(loaded).unwrap(), expected);
        let mut args = json!({ "url": path.to_string_lossy() });
        if let Some(mode) = publication {
            args["htmlPublication"] = json!(mode);
        }
        let dispatched = service.dispatch_gateway("loadWebMedia", &args)
            .expect("loadWebMedia is handled")
            .expect("non-explicit service mode remains unchanged");
        assert_eq!(dispatched, expected);
    }
    for publication in [json!(null), json!(false), json!(true), json!(17), json!({ "mode": "explicit" }), json!(["explicit"])] {
        let dispatched = service.dispatch_gateway("loadWebMedia", &json!({
            "url": path.to_string_lossy(),
            "htmlPublication": publication,
        }))
        .expect("loadWebMedia is handled")
        .expect("non-string publication does not opt in");
        assert_eq!(dispatched, expected);
    }
    assert_explicit_html_rejected(&service, &path.to_string_lossy());
}

#[test]
fn test_ocrm_web_media_explicit_mode_preserves_non_html_loading() {
    let temp = TempTestDir::new("explicit-non-html");
    let service = test_attachments_service(&temp.path);
    let png = vec![
        0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A, 0x00, 0x00, 0x00, 0x0D, 0x49, 0x48,
        0x44, 0x52, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x02, 0x00, 0x00,
        0x00, 0x90, 0x77, 0x53, 0xDE, 0x00, 0x00, 0x00, 0x0C, 0x49, 0x44, 0x41, 0x54, 0x08,
        0xD7, 0x63, 0xF8, 0xCF, 0xC0, 0x00, 0x00, 0x03, 0x01, 0x01, 0x00, 0x18, 0xDD, 0x8D,
        0xB0, 0x00, 0x00, 0x00, 0x00, 0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82,
    ];
    let cases = [
        ("bytes.txt", vec![0, 0xff, 1, 2], None),
        ("markup.bin", b"<html><body>ordinary bytes</body></html>".to_vec(), None),
        ("dot.png", png, Some("image/png")),
    ];
    for (file_name, bytes, content_type) in cases {
        let path = temp.path.join(file_name);
        fs::write(&path, &bytes).expect("write non-HTML fixture");
        for optimize_images in [false, true] {
            let options = WebMediaOptions {
                optimize_images,
                html_publication: Some("explicit".to_string()),
                ..Default::default()
            };
            let mut expected = json!({ "buffer": bytes, "kind": "image", "fileName": file_name });
            if let Some(mime) = content_type {
                expected["contentType"] = json!(mime);
            }
            let loaded = load_web_media(&path.to_string_lossy(), &options).expect("non-HTML loader remains unchanged");
            assert_eq!(serde_json::to_value(loaded).unwrap(), expected);
            let dispatched = service.dispatch_gateway("loadWebMedia", &json!({
                "url": path.to_string_lossy(),
                "htmlPublication": "explicit",
                "optimizeImages": optimize_images,
            }))
            .expect("loadWebMedia is handled")
            .expect("non-HTML service load remains unchanged");
            assert_eq!(dispatched, expected);
        }
    }
}

#[test]
fn test_ocrm_web_media_explicit_html_keeps_byte_cap_boundary() {
    let temp = TempTestDir::new("explicit-html-cap");
    let service = test_attachments_service(&temp.path);
    let valid = b"<html><body>bounded actual bytes</body></html>";
    let valid_path = temp.path.join("valid.html");
    fs::write(&valid_path, valid).expect("write valid HTML");
    assert_explicit_html_loads(&service, &valid_path.to_string_lossy(), valid, "valid.html");

    for (file_name, bytes) in [("valid.html", valid.as_slice()), ("invalid.html", b"not HTML but still bounded".as_slice())] {
        let path = temp.path.join(file_name);
        fs::write(&path, bytes).expect("write cap fixture");
        let options = WebMediaOptions {
            max_bytes: Some(bytes.len() - 1),
            optimize_images: true,
            html_publication: Some("explicit".to_string()),
            ..Default::default()
        };
        let error = load_web_media(&path.to_string_lossy(), &options).expect_err("byte cap must precede HTML admission");
        assert!(matches!(&error, WebMediaError::TooLarge(_)), "wrong cap error: {error:?}");
        let service_error = service.dispatch_gateway("loadWebMedia", &json!({
            "url": path.to_string_lossy(),
            "maxBytes": bytes.len() - 1,
            "htmlPublication": "explicit",
        }))
        .expect("loadWebMedia is handled")
        .expect_err("service must preserve the existing byte cap");
        assert_eq!(service_error, error.to_string());
    }
}

#[test]
fn test_ocrm_web_media_explicit_html_keeps_local_access_guards() {
    let temp = TempTestDir::new("explicit-html-access");
    let service = test_attachments_service(&temp.path);
    let allowed = temp.path.join("allowed");
    let forbidden = temp.path.join("forbidden");
    fs::create_dir_all(&allowed).expect("allowed root");
    fs::create_dir_all(&forbidden).expect("outside allowed root");
    let bytes = b"<html><body>root-constrained HTML</body></html>";
    let allowed_path = allowed.join("report.html");
    let forbidden_path = forbidden.join("report.html");
    fs::write(&allowed_path, bytes).expect("write allowed HTML");
    fs::write(&forbidden_path, bytes).expect("write forbidden HTML");
    let options = WebMediaOptions {
        local_roots: Some(vec![allowed.clone()]),
        workspace_dir: Some(allowed.clone()),
        html_publication: Some("explicit".to_string()),
        ..Default::default()
    };
    let loaded = load_web_media("report.html", &options).expect("resolved workspace HTML inside root");
    assert_eq!(loaded.buffer, bytes);
    assert_eq!(loaded.content_type.as_deref(), Some("text/html"));
    assert_eq!(loaded.kind.as_deref(), Some("document"));
    assert!(loaded.trusted_generated_html_source.is_none());
    for source in [forbidden_path.to_string_lossy().to_string(), "../forbidden/report.html".to_string()] {
        match load_web_media(&source, &options).expect_err("explicit mode cannot bypass roots") {
            WebMediaError::LocalAccess { code, .. } => assert_eq!(code, "path-not-allowed"),
            other => panic!("Expected path-not-allowed, got: {other:?}"),
        }
    }

    let missing = allowed.join("missing.html");
    let directory = allowed.join("directory.html");
    fs::create_dir(&directory).expect("directory is not a file");
    let options = WebMediaOptions {
        html_publication: Some("explicit".to_string()),
        ..Default::default()
    };
    let cases = [
        (missing.to_string_lossy().to_string(), "not-found"),
        (directory.to_string_lossy().to_string(), "not-found"),
        (r"\\attacker\share\report.html".to_string(), "network-path-not-allowed"),
        ("//attacker/share/report.html".to_string(), "network-path-not-allowed"),
        ("file://attacker/share/report.html".to_string(), "invalid-file-url"),
        ("file:///safe/folder%2Freport.html".to_string(), "invalid-file-url"),
    ];
    for (source, expected_code) in cases {
        match load_web_media(&source, &options).expect_err("existing access guard must still refuse") {
            WebMediaError::LocalAccess { code, .. } => assert_eq!(code, expected_code),
            other => panic!("Expected local access error, got: {other:?}"),
        }
        let error = service.dispatch_gateway("loadWebMedia", &json!({
            "url": source,
            "htmlPublication": "explicit",
            "hostReadCapability": true,
            "sandboxValidated": true,
            "localRoots": [temp.path.to_string_lossy()],
            "trustedGeneratedHtmlSource": true,
        }))
        .expect("loadWebMedia is handled")
        .expect_err("caller fields cannot bypass an existing access guard");
        assert!(error.starts_with(&format!("Local media access error [{expected_code}]:")));
    }
}

#[test]
fn test_ocrm_web_media_explicit_html_retry_reads_fresh_bytes() {
    let temp = TempTestDir::new("explicit-html-retry");
    let service = test_attachments_service(&temp.path);
    let path = temp.path.join("retry.html");
    fs::write(&path, b"not an HTML document").expect("write initial invalid bytes");
    assert_explicit_html_rejected(&service, &path.to_string_lossy());

    let first = b"<html><body>first report</body></html>";
    let second = b"<html><body>other report</body></html>";
    assert_eq!(first.len(), second.len());
    for bytes in [first, second] {
        fs::remove_file(&path).expect("replace prior file");
        fs::write(&path, bytes).expect("write fresh HTML bytes");
        assert_explicit_html_loads(&service, &path.to_string_lossy(), bytes, "retry.html");
        assert_eq!(fs::read(&path).unwrap(), bytes, "loading must preserve the file");
        fs::write(&path, vec![b'x'; bytes.len()]).expect("same-length invalid replacement");
        assert_explicit_html_rejected(&service, &path.to_string_lossy());
    }
    assert_eq!(fs::read_dir(&temp.path).unwrap().count(), 1, "no staged file or provenance sidecar");
}

#[test]
fn test_ocrm_web_media_service_maximum_cap_preserves_actual_bytes() {
    let temp = TempTestDir::new("maximum-cap");
    let service = test_attachments_service(&temp.path);
    // Header-only bytes exercise the existing metadata probe, not a full decoder.
    let png = [
        0x89, b'P', b'N', b'G', 13, 10, 26, 10,
        0, 0, 0, 13, b'I', b'H', b'D', b'R',
        0, 0, 0, 1, 0, 0, 0, 1, 8, 6,
    ];
    let html = b"<html><body>maximum cap still reads real bytes</body></html>";
    for (name, bytes, publication, mime, kind) in [
        ("limit.png", png.as_slice(), None, "image/png", "image"),
        ("limit.html", html.as_slice(), Some("explicit"), "text/html", "document"),
    ] {
        let path = temp.path.join(name);
        fs::write(&path, bytes).expect("write actual file");
        let result = service.dispatch_gateway("loadWebMedia", &json!({
            "url": path.to_string_lossy(),
            "maxBytes": u64::MAX,
            "optimizeImages": false,
            "htmlPublication": publication,
        }))
        .expect("existing service handles the request");
        if usize::try_from(u64::MAX).is_err() {
            assert_eq!(result.expect_err("unrepresentable cap must be refused"),
                "loadWebMedia.maxBytes exceeds the supported byte-count range");
        } else {
            let value = result.expect("representable maximum cap must preserve the file");
            assert_eq!(value, json!({
                "buffer": bytes,
                "contentType": mime,
                "kind": kind,
                "fileName": name,
            }));
            assert!(value.get("trustedGeneratedHtmlSource").is_none());
        }
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }
}

#[test]
fn test_ocrm_web_media_service_reader_zero_exact_and_default_caps() {
    let temp = TempTestDir::new("reader-cap-boundaries");
    let service = test_attachments_service(&temp.path);
    let path = temp.path.join("bytes.bin");
    for (bytes, cap, succeeds) in [
        (b"".as_slice(), 0usize, true),
        (b"actual bytes".as_slice(), 0usize, false),
        (b"actual bytes".as_slice(), 12usize, true),
        (b"actual bytes".as_slice(), 11usize, false),
    ] {
        fs::write(&path, bytes).expect("write bounded input");
        let result = service.dispatch_gateway("loadWebMedia", &json!({
            "url": path.to_string_lossy(), "maxBytes": cap, "optimizeImages": false,
        }))
        .expect("existing service handles the request");
        if succeeds {
            let value = result.expect("EOF within cap is success");
            assert_eq!(value["buffer"], json!(bytes));
        } else {
            let error = result.expect_err("over-cap input must not publish a prefix");
            assert!(error.starts_with("Media exceeds "), "wrong error: {error}");
        }
        assert_eq!(fs::read(&path).unwrap(), bytes);
    }

    let bytes = b"ordinary default behavior";
    fs::write(&path, bytes).unwrap();
    for cap in [json!(null), json!(-1), json!("1"), json!(1.5), json!(false)] {
        let value = service.dispatch_gateway("loadWebMedia", &json!({
            "url": path.to_string_lossy(), "maxBytes": cap, "optimizeImages": false,
        }))
        .expect("existing service handles the request")
        .expect("non-u64 options retain the established default");
        assert_eq!(value["buffer"], json!(bytes));
    }
}

#[test]
fn test_ocrm_web_media_service_reader_failures_and_fresh_retry() {
    let temp = TempTestDir::new("reader-failure-retry");
    let service = test_attachments_service(&temp.path);
    let path = temp.path.join("retry.bin");
    let directory = temp.path.join("directory.bin");
    fs::create_dir(&directory).unwrap();
    for refused in [&path, &directory] {
        let error = service.dispatch_gateway("loadWebMedia", &json!({
            "url": refused.to_string_lossy(), "maxBytes": 1024, "optimizeImages": false,
        }))
        .expect("existing service handles the request")
        .expect_err("missing and nonregular files remain failures");
        assert!(error.starts_with("Local media access error [not-found]:"), "wrong error: {error}");
    }
    for bytes in [b"first bytes".as_slice(), b"replacement bytes".as_slice()] {
        if path.exists() {
            fs::remove_file(&path).unwrap();
        }
        fs::write(&path, bytes).unwrap();
        let value = service.dispatch_gateway("loadWebMedia", &json!({
            "url": path.to_string_lossy(), "maxBytes": bytes.len(), "optimizeImages": false,
        }))
        .expect("existing service handles the request")
        .expect("fresh admitted file is read after refusal or replacement");
        assert_eq!(value["buffer"], json!(bytes));
        assert_eq!(fs::read(&path).unwrap(), bytes);
        assert!(value.get("trustedGeneratedHtmlSource").is_none());
    }
    assert!(directory.is_dir(), "a failed read cannot replace the directory");
}

#[cfg(unix)]
#[test]
fn test_ocrm_web_media_reader_aliases_keep_actual_root_checks() {
    use std::os::unix::fs::symlink;

    let temp = TempTestDir::new("reader-alias-roots");
    let allowed = temp.path.join("allowed");
    let outside = temp.path.join("outside.bin");
    fs::create_dir(&allowed).unwrap();
    let target = allowed.join("target.bin");
    let bytes = b"admitted alias bytes";
    fs::write(&target, bytes).unwrap();
    fs::write(&outside, b"outside bytes").unwrap();
    let symbolic = allowed.join("symbolic.bin");
    let hard = allowed.join("hard.bin");
    let escaping = allowed.join("escaping.bin");
    symlink(&target, &symbolic).expect("actual ordinary-file symlink");
    fs::hard_link(&target, &hard).expect("actual ordinary-file hardlink");
    symlink(&outside, &escaping).expect("actual outside-root alias");
    let options = WebMediaOptions {
        max_bytes: Some(usize::MAX),
        local_roots: Some(vec![allowed]),
        optimize_images: false,
        ..Default::default()
    };
    for path in [&target, &symbolic, &hard] {
        let loaded = load_web_media(&path.to_string_lossy(), &options)
            .expect("existing admitted ordinary aliases remain readable");
        assert_eq!(loaded.buffer, bytes);
        assert!(loaded.trusted_generated_html_source.is_none());
    }
    match load_web_media(&escaping.to_string_lossy(), &options)
        .expect_err("same reader must not bypass its caller's root check") {
        WebMediaError::LocalAccess { code, .. } => assert_eq!(code, "path-not-allowed"),
        other => panic!("wrong root refusal: {other:?}"),
    }
    assert_eq!(fs::read(&outside).unwrap(), b"outside bytes");
}
