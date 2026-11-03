use std::fs;
use std::path::PathBuf;
use std::sync::Arc;
use serde_json::json;

use mahayana_host_runtime::extensions::attachments::attachments_service::{AttachmentsBox, AttachmentsService};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
use mahayana_host_runtime::extensions::attachments::image_ops::{
    build_image_resize_side_grid, convert_bmp_to_png, is_animated_webp_buffer,
    is_still_png_buffer, normalize_anthropic_inline_image, prepare_attachment,
    read_image_metadata_from_header, read_image_probe_from_header, AttachmentInput,
    ImageMetadata, ImageProbe, IMAGE_REDUCE_QUALITY_STEPS, WITHIN_BYTES_QUALITY_STEPS,
};
use mahayana_host_runtime::extensions::attachments::web_media::{
    load_web_media, WebMediaError, WebMediaOptions,
};

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ocrm-image-ops-test-{name}-{}", uuid::Uuid::new_v4()));
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
fn test_ocrm_image_ops_build_resize_side_grid() {
    let grid1 = build_image_resize_side_grid(1200, 900);
    assert_eq!(grid1, vec![1200, 1000, 900, 800]);

    let grid_zero = build_image_resize_side_grid(0, 0);
    assert!(grid_zero.is_empty());

    let grid2 = build_image_resize_side_grid(1600, 1800);
    assert_eq!(grid2, vec![1600, 1400, 1200, 1000, 800]);
}

#[test]
fn test_ocrm_image_ops_quality_steps_ladder() {
    assert_eq!(IMAGE_REDUCE_QUALITY_STEPS, &[85, 75, 65, 55, 45, 35]);
    assert_eq!(WITHIN_BYTES_QUALITY_STEPS, &[92, 85, 78, 70, 62, 52, 42]);
}

#[test]
fn test_ocrm_image_ops_animated_webp_detection() {
    // Valid VP8X animated WebP buffer: RIFF (4) + len (4) + WEBP (4) + VP8X (4) + chunk_len (4) + flags (1) + ...
    let mut webp = Vec::new();
    webp.extend_from_slice(b"RIFF");
    webp.extend_from_slice(&32u32.to_le_bytes());
    webp.extend_from_slice(b"WEBP");
    webp.extend_from_slice(b"VP8X");
    webp.extend_from_slice(&10u32.to_le_bytes()); // chunk len >= 10
    webp.push(0x02); // animation bit 1 set
    webp.extend_from_slice(&[0u8; 15]); // padding to length 40

    assert!(is_animated_webp_buffer(&webp));

    // Clear animation bit
    webp[20] = 0x00;
    assert!(!is_animated_webp_buffer(&webp));

    // Truncated buffer
    assert!(!is_animated_webp_buffer(&webp[..20]));
}

#[test]
fn test_ocrm_image_ops_still_png_and_apng_detection() {
    // 1x1 PNG: signature + IHDR chunk + IDAT chunk + IEND chunk
    let mut png = Vec::new();
    png.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    // IHDR chunk: len=13, type="IHDR", data (13 bytes), crc (4 bytes)
    png.extend_from_slice(&13u32.to_be_bytes());
    png.extend_from_slice(b"IHDR");
    png.extend_from_slice(&1u32.to_be_bytes()); // width
    png.extend_from_slice(&1u32.to_be_bytes()); // height
    png.extend_from_slice(&[8, 2, 0, 0, 0]); // 8-bit truecolor
    png.extend_from_slice(&[0u8; 4]); // crc placeholder
    // IDAT chunk: len=4, type="IDAT", data, crc
    png.extend_from_slice(&4u32.to_be_bytes());
    png.extend_from_slice(b"IDAT");
    png.extend_from_slice(&[0u8; 4]);
    png.extend_from_slice(&[0u8; 4]);
    // IEND chunk
    png.extend_from_slice(&0u32.to_be_bytes());
    png.extend_from_slice(b"IEND");
    png.extend_from_slice(&[0u8; 4]);

    assert!(is_still_png_buffer(&png));

    // APNG: insert acTL chunk before IDAT
    let mut apng = Vec::new();
    apng.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    apng.extend_from_slice(&13u32.to_be_bytes());
    apng.extend_from_slice(b"IHDR");
    apng.extend_from_slice(&1u32.to_be_bytes());
    apng.extend_from_slice(&1u32.to_be_bytes());
    apng.extend_from_slice(&[8, 2, 0, 0, 0]);
    apng.extend_from_slice(&[0u8; 4]);
    // acTL chunk (animation control)
    apng.extend_from_slice(&8u32.to_be_bytes());
    apng.extend_from_slice(b"acTL");
    apng.extend_from_slice(&[0u8; 8]);
    apng.extend_from_slice(&[0u8; 4]);
    // IDAT
    apng.extend_from_slice(&4u32.to_be_bytes());
    apng.extend_from_slice(b"IDAT");
    apng.extend_from_slice(&[0u8; 4]);
    apng.extend_from_slice(&[0u8; 4]);

    assert!(!is_still_png_buffer(&apng));
}

#[test]
fn test_ocrm_image_ops_display_metadata_orientation_swap() {
    // Construct JPEG with SOF0 (640x480) and EXIF App1 with orientation 6
    let mut jpeg = Vec::new();
    jpeg.extend_from_slice(&[0xFF, 0xD8]); // SOI

    // App1 with EXIF
    let mut exif_payload = Vec::new();
    exif_payload.extend_from_slice(b"Exif\0\0");
    exif_payload.extend_from_slice(b"II"); // little endian
    exif_payload.extend_from_slice(&42u16.to_le_bytes()); // magic 42
    exif_payload.extend_from_slice(&8u32.to_le_bytes()); // offset to IFD0 (8 bytes from TIFF start)
    // IFD0: 1 entry
    exif_payload.extend_from_slice(&1u16.to_le_bytes()); // 1 entry
    // Entry: tag 0x0112 (Orientation), type 3 (SHORT), count 1, value 6
    exif_payload.extend_from_slice(&0x0112u16.to_le_bytes());
    exif_payload.extend_from_slice(&3u16.to_le_bytes());
    exif_payload.extend_from_slice(&1u32.to_le_bytes());
    exif_payload.extend_from_slice(&6u16.to_le_bytes());
    exif_payload.extend_from_slice(&0u16.to_le_bytes()); // padding

    let app1_len = (exif_payload.len() + 2) as u16;
    jpeg.extend_from_slice(&[0xFF, 0xE1]);
    jpeg.extend_from_slice(&app1_len.to_be_bytes());
    jpeg.extend_from_slice(&exif_payload);

    // SOF0: height=480, width=640
    jpeg.extend_from_slice(&[0xFF, 0xC0]);
    jpeg.extend_from_slice(&11u16.to_be_bytes()); // length
    jpeg.push(8); // precision
    jpeg.extend_from_slice(&480u16.to_be_bytes()); // height
    jpeg.extend_from_slice(&640u16.to_be_bytes()); // width
    jpeg.push(3); // components
    jpeg.extend_from_slice(&[1, 0x11, 0, 2, 0x11, 1, 3, 0x11, 1]); // components data

    let probe = read_image_probe_from_header(&jpeg).expect("probe success");
    assert_eq!(probe.width, 640);
    assert_eq!(probe.height, 480);
    assert_eq!(probe.orientation, Some(6));

    let display_meta = read_image_metadata_from_header(&jpeg).expect("metadata success");
    // Orientation 6 must swap width and height for display!
    assert_eq!(
        display_meta,
        ImageMetadata {
            width: 480,
            height: 640,
        }
    );
}

#[test]
fn test_ocrm_image_ops_bmp_to_png_conversion_and_pixel_limit() {
    // Two 24-bit BMP pixels: red (0x00, 0x00, 0xFF) then green (0x00, 0xFF, 0x00) with row padding
    let hex_bmp = "424d3e00000000000000360000002800000002000000010000000100180000000000\
                   08000000000000000000000000000000000000000000ff00ff000000";
    let bmp = (0..hex_bmp.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_bmp[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>();

    let png = convert_bmp_to_png(&bmp).expect("BMP to PNG conversion succeeded");
    assert_eq!(&png[0..8], b"\x89PNG\r\n\x1a\n");

    let probe = read_image_probe_from_header(&png).expect("probe PNG");
    assert_eq!(probe.width, 2);
    assert_eq!(probe.height, 1);
    assert_eq!(probe.format, "png");
}

#[test]
fn test_ocrm_image_ops_prepare_attachment() {
    use base64::Engine as _;
    let hex_bmp = "424d3e00000000000000360000002800000002000000010000000100180000000000\
                   08000000000000000000000000000000000000000000ff00ff000000";
    let bmp_bytes = (0..hex_bmp.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_bmp[i..i + 2], 16).unwrap())
        .collect::<Vec<u8>>();
    let b64 = base64::engine::general_purpose::STANDARD.encode(&bmp_bytes);

    let input = AttachmentInput {
        base64: b64,
        label: "test.bmp".to_string(),
        mime: "image/bmp".to_string(),
        image_storage: None,
    };

    let prepared = prepare_attachment(&input).expect("prepare attachment succeeded");
    assert_eq!(prepared.size_bytes, bmp_bytes.len());
    assert_eq!(prepared.mime, "image/bmp");
    assert_eq!(prepared.width, Some(2));
    assert_eq!(prepared.height, Some(1));
    assert!(prepared.buffer.is_some());

    // Test invalid base64 rejection
    let bad_input = AttachmentInput {
        base64: "not valid base64!!!".to_string(),
        label: "corrupt.png".to_string(),
        mime: "image/png".to_string(),
        image_storage: None,
    };
    let err = prepare_attachment(&bad_input).unwrap_err();
    assert!(err.contains("invalid base64 content"));
}

#[test]
fn test_ocrm_image_ops_normalize_anthropic_inline_image() {
    use base64::Engine as _;
    // One red pixel in uncompressed 24-bit BMP (from OpenClaw image worker test)
    let b64_bmp = "Qk06AAAAAAAAADYAAAAoAAAAAQAAAAEAAAABABgAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAD/AA==";

    let (norm_data, norm_mime) = normalize_anthropic_inline_image(b64_bmp, "image/bmp")
        .expect("normalize BMP inline image succeeded");

    assert_eq!(norm_mime, "image/png");
    let decoded_png = base64::engine::general_purpose::STANDARD
        .decode(&norm_data)
        .expect("valid base64 png");
    assert_eq!(&decoded_png[0..8], b"\x89PNG\r\n\x1a\n");

    let probe = read_image_probe_from_header(&decoded_png).expect("probe decoded png");
    assert_eq!(probe.width, 1);
    assert_eq!(probe.height, 1);
    assert_eq!(probe.format, "png");
}

#[test]
fn test_ocrm_image_ops_gateway_dispatch() {
    let temp = TempTestDir::new("gateway");
    let service = test_attachments_service(&temp.path);

    // 1. buildImageResizeSideGrid
    let res = service
        .dispatch_gateway(
            "buildImageResizeSideGrid",
            &json!({ "maxSide": 1200, "sideStart": 900 }),
        )
        .expect("dispatch handled")
        .expect("success");
    assert_eq!(res, json!([1200, 1000, 900, 800]));

    // 2. convertBmpToPng gateway dispatch
    let b64_bmp = "Qk06AAAAAAAAADYAAAAoAAAAAQAAAAEAAAABABgAAAAAAAQAAAAAAAAAAAAAAAAAAAAAAAAAAAD/AA==";
    let res_bmp = service
        .dispatch_gateway("convertBmpToPng", &json!({ "base64": b64_bmp }))
        .expect("dispatch handled")
        .expect("success");
    assert_eq!(res_bmp["mimeType"], "image/png");

    // 3. normalizeAnthropicInlineImage gateway dispatch
    let res_norm = service
        .dispatch_gateway(
            "normalizeAnthropicInlineImage",
            &json!({ "data": b64_bmp, "mimeType": "image/bmp" }),
        )
        .expect("dispatch handled")
        .expect("success");
    assert_eq!(res_norm["mimeType"], "image/png");
}

// Header-only probe fixture through the color-type byte; no full chunk, CRC, or pixel data.
fn png_probe_header(width: u32, height: u32, color_type: u8) -> Vec<u8> {
    let mut header = Vec::new();
    header.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    header.extend_from_slice(&13u32.to_be_bytes());
    header.extend_from_slice(b"IHDR");
    header.extend_from_slice(&width.to_be_bytes());
    header.extend_from_slice(&height.to_be_bytes());
    header.extend_from_slice(&[8, color_type]);
    header
}

#[test]
fn test_ocrm_image_ops_png_probe_rejects_every_prefix_before_color_type() {
    let header = png_probe_header(640, 480, 6);
    assert_eq!(header.len(), 26);

    // Direct calls include the former 24- and 25-byte panic cases.
    for len in 0..=25 {
        let prefix = &header[..len];
        assert_eq!(read_image_probe_from_header(prefix), None, "prefix length {len}");
        assert_eq!(read_image_metadata_from_header(prefix), None, "prefix length {len}");
    }
}

#[test]
fn test_ocrm_image_ops_png_probe_preserves_required_header_metadata() {
    for (color_type, has_alpha) in [(0, false), (2, false), (3, false), (4, true), (6, true)] {
        for extra_bytes in [0, 3, 7] {
            let mut header = png_probe_header(640, 480, color_type);
            header.resize(26 + extra_bytes, 0);
            assert_eq!(
                read_image_probe_from_header(&header),
                Some(ImageProbe {
                    width: 640,
                    height: 480,
                    format: "png".to_string(),
                    has_alpha,
                    orientation: None,
                    bytes: 26 + extra_bytes,
                }),
                "color type {color_type}, extra bytes {extra_bytes}"
            );
            assert_eq!(
                read_image_metadata_from_header(&header),
                Some(ImageMetadata { width: 640, height: 480 })
            );
        }
    }
}

#[test]
fn test_ocrm_image_ops_png_probe_rejects_zero_dimensions() {
    for (width, height) in [(0, 480), (640, 0), (0, 0)] {
        let header = png_probe_header(width, height, 6);
        assert_eq!(read_image_probe_from_header(&header), None);
        assert_eq!(read_image_metadata_from_header(&header), None);
    }
}

#[test]
fn test_ocrm_image_ops_png_prefix_prepare_attachment_and_retry() {
    use base64::Engine as _;
    let header = png_probe_header(640, 480, 6);

    for len in [24, 25] {
        for mime in ["image/png", "application/octet-stream"] {
            let mut input = AttachmentInput {
                base64: base64::engine::general_purpose::STANDARD.encode(&header[..len]),
                label: "header.png".to_string(),
                mime: mime.to_string(),
                image_storage: None,
            };
            let prepared = prepare_attachment(&input).expect("short PNG header remains an attachment");
            assert_eq!(prepared.size_bytes, len);
            assert_eq!(prepared.mime, "image/png");
            assert_eq!(prepared.buffer.as_deref(), Some(&header[..len]));
            assert_eq!(prepared.width, None);
            assert_eq!(prepared.height, None);

            input.base64 = base64::engine::general_purpose::STANDARD.encode(&header);
            let prepared = prepare_attachment(&input).expect("sufficient header provides metadata");
            assert_eq!(prepared.size_bytes, 26);
            assert_eq!(prepared.mime, "image/png");
            assert_eq!(prepared.buffer.as_deref(), Some(header.as_slice()));
            assert_eq!(prepared.width, Some(640));
            assert_eq!(prepared.height, Some(480));
        }
    }
}

#[test]
fn test_ocrm_image_ops_png_prefix_file_loader_and_retry() {
    let temp = TempTestDir::new("png-probe-retry");
    let path = temp.path.join("header.png");
    let header = png_probe_header(640, 480, 6);

    for optimize_images in [false, true] {
        let options = WebMediaOptions {
            max_bytes: Some(26),
            local_roots: Some(vec![temp.path.clone()]),
            optimize_images,
            ..Default::default()
        };
        for len in [24, 25] {
            fs::write(&path, &header[..len]).expect("write short PNG header");
            let loaded = load_web_media(&path.to_string_lossy(), &options)
                .expect("bounded file loader accepts short header bytes");
            assert_eq!(loaded.buffer.as_slice(), &header[..len]);
            assert_eq!(loaded.content_type, None);
            assert_eq!(loaded.kind.as_deref(), Some("image"));
            assert_eq!(loaded.file_name.as_deref(), Some("header.png"));
            assert_eq!(loaded.trusted_generated_html_source, None);
            assert_eq!(read_image_metadata_from_header(&loaded.buffer), None);

            fs::write(&path, &header).expect("replace the same file with a sufficient header");
            let loaded = load_web_media(&path.to_string_lossy(), &options)
                .expect("bounded file loader retries with fresh header bytes");
            assert_eq!(loaded.buffer, header);
            assert_eq!(loaded.content_type.as_deref(), Some("image/png"));
            assert_eq!(loaded.kind.as_deref(), Some("image"));
            assert_eq!(loaded.file_name.as_deref(), Some("header.png"));
            assert_eq!(loaded.trusted_generated_html_source, None);
            assert_eq!(
                read_image_metadata_from_header(&loaded.buffer),
                Some(ImageMetadata { width: 640, height: 480 })
            );
        }
    }
}

#[test]
fn test_ocrm_image_ops_png_file_loader_preserves_read_boundaries() {
    let temp = TempTestDir::new("png-probe-boundaries");
    let allowed_dir = temp.path.join("allowed");
    let forbidden_dir = temp.path.join("forbidden");
    fs::create_dir_all(&allowed_dir).expect("create allowed root");
    fs::create_dir_all(&forbidden_dir).expect("create forbidden root");
    let path = allowed_dir.join("header.png");
    let forbidden_path = forbidden_dir.join("header.png");
    let header = png_probe_header(640, 480, 2);
    fs::write(&path, &header).expect("write header at exact byte cap");
    fs::write(&forbidden_path, &header).expect("write header outside allowed root");
    let options = WebMediaOptions {
        max_bytes: Some(26),
        local_roots: Some(vec![allowed_dir.clone()]),
        ..Default::default()
    };

    let loaded = load_web_media(&path.to_string_lossy(), &options).expect("exact byte cap is accepted");
    assert_eq!(loaded.buffer, header);
    assert_eq!(loaded.content_type.as_deref(), Some("image/png"));

    let mut oversized = header.clone();
    oversized.push(0);
    fs::write(&path, &oversized).expect("write one byte above cap");
    assert!(matches!(
        load_web_media(&path.to_string_lossy(), &options),
        Err(WebMediaError::TooLarge(_))
    ));
    fs::write(&path, &header).expect("restore file to byte cap");
    let loaded = load_web_media(&path.to_string_lossy(), &options).expect("retry after size rejection");
    assert_eq!(loaded.buffer, header);
    assert_eq!(loaded.content_type.as_deref(), Some("image/png"));

    assert!(matches!(
        load_web_media(&forbidden_path.to_string_lossy(), &options),
        Err(WebMediaError::LocalAccess { code, .. }) if code == "path-not-allowed"
    ));
    assert!(matches!(
        load_web_media(&allowed_dir.to_string_lossy(), &options),
        Err(WebMediaError::LocalAccess { code, .. }) if code == "not-found"
    ));
}

#[test]
fn test_ocrm_image_ops_png_prefix_gateway_dispatch_and_retry() {
    use base64::Engine as _;
    let temp = TempTestDir::new("png-probe-gateway");
    let service = test_attachments_service(&temp.path);
    let path = temp.path.join("header.png");
    let header = png_probe_header(640, 480, 6);
    let load_args = json!({ "url": path.to_string_lossy(), "maxBytes": 26 });

    for len in [24, 25] {
        fs::write(&path, &header[..len]).expect("write short header for service load");
        let loaded = service
            .dispatch_gateway("loadWebMedia", &load_args)
            .expect("loadWebMedia handled")
            .expect("short header loaded without metadata");
        assert_eq!(loaded, json!({
            "buffer": &header[..len], "kind": "image", "fileName": "header.png"
        }));
        let loaded_bytes: Vec<u8> = serde_json::from_value(loaded["buffer"].clone())
            .expect("service returns byte buffer");
        let base64 = base64::engine::general_purpose::STANDARD.encode(&loaded_bytes);
        let metadata = service
            .dispatch_gateway("readImageMetadata", &json!({ "base64": &base64 }))
            .expect("readImageMetadata handled")
            .expect("short header metadata is optional");
        assert_eq!(metadata, json!(null));
        let prepared = service
            .dispatch_gateway("prepareAttachment", &json!({
                "base64": &base64, "label": "header.png", "mime": "image/png"
            }))
            .expect("prepareAttachment handled")
            .expect("short header attachment prepared");
        assert_eq!(prepared, json!({
            "sizeBytes": len, "mime": "image/png", "buffer": &header[..len]
        }));

        fs::write(&path, &header).expect("replace the same service file with sufficient header");
        let loaded = service
            .dispatch_gateway("loadWebMedia", &load_args)
            .expect("loadWebMedia retry handled")
            .expect("service reloads fresh file bytes");
        assert_eq!(loaded, json!({
            "buffer": &header, "contentType": "image/png", "kind": "image", "fileName": "header.png"
        }));
        let loaded_bytes: Vec<u8> = serde_json::from_value(loaded["buffer"].clone())
            .expect("service returns fresh byte buffer");
        let base64 = base64::engine::general_purpose::STANDARD.encode(&loaded_bytes);
        let metadata = service
            .dispatch_gateway("readImageMetadata", &json!({ "base64": &base64 }))
            .expect("readImageMetadata retry handled")
            .expect("fresh header metadata read");
        assert_eq!(metadata, json!({ "width": 640, "height": 480 }));
        let prepared = service
            .dispatch_gateway("prepareAttachment", &json!({
                "base64": &base64, "label": "header.png", "mime": "image/png"
            }))
            .expect("prepareAttachment retry handled")
            .expect("fresh header attachment prepared");
        assert_eq!(prepared, json!({
            "sizeBytes": 26, "mime": "image/png", "buffer": &header, "width": 640, "height": 480
        }));
    }

    let error = service
        .dispatch_gateway("loadWebMedia", &json!({
            "url": path.to_string_lossy(), "maxBytes": 25
        }))
        .expect("loadWebMedia byte limit handled")
        .expect_err("service keeps the bounded file read limit");
    assert!(error.starts_with("Media exceeds "), "unexpected error: {error}");
}
