//! Contract test suite for OpenClaw Input Files & Staged Inputs (`OCRM-UNIT-MEDIA-014`).
//!
//! Validates:
//! - Staged input directory prefix, ownership markers, sha256 verification, and path specs.
//! - Staged input alias extraction, duplicate disambiguation, and media path mapping.
//! - Text decoding across UTF-8, Latin-1, Windows-1251, Windows-1252, and UTF-16LE.
//! - BOM-detected UTF-16 precedence over declared charsets.
//! - UTF-16 surrogate-safe truncation and text truncation metadata flags.
//! - Rejection of disallowed URL sources and oversized files/images.
//! - Video byte classification and image allowlist rejection (AVI -> video/x-msvideo).
//! - HEIC / HEIF image normalization to JPEG.
//! - Gateway dispatch integration for input file and staged input methods.
//! - Restrictive URL hostname admission without granting private-host access.

use std::collections::HashSet;
use std::fs;
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentsBox, AttachmentsService,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::GenerateImageAuth;
use mahayana_host_runtime::extensions::attachments::input_files::{
    classify_attachment_bytes, extract_file_content_from_buffer,
    extract_file_content_from_source, extract_image_content_from_source,
    normalize_input_image_buffer, resolve_input_file_limits, slice_utf16_safe,
    truncate_utf16_safe, AttachmentClass, InputFileLimitsConfig, InputFileSource,
    InputImageLimits, InputImageSource, DEFAULT_INPUT_IMAGE_MIMES,
};
use mahayana_host_runtime::extensions::attachments::media_facts::MediaFact;
use mahayana_host_runtime::extensions::attachments::staged_inputs::{
    ensure_staged_input_directory, is_staged_input_path, resolve_staged_input_media_paths,
    staged_input_directories_from_entries, staged_input_directory, staged_input_path_directory,
    StagedManifestEntry, STAGED_INPUT_DIRECTORY_PREFIX, STAGED_INPUT_GITIGNORE,
    STAGED_INPUT_GITIGNORE_SHA256,
};
use serde_json::Value;

struct TempTestDir {
    path: PathBuf,
}

impl TempTestDir {
    fn new(name: &str) -> Self {
        let path = std::env::temp_dir().join(format!("ocrm-input-files-test-{name}-{}", uuid::Uuid::new_v4()));
        let _ = fs::create_dir_all(&path);
        Self { path }
    }
}

impl Drop for TempTestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

struct MockAuth;
impl GenerateImageAuth for MockAuth {
    fn get_access_token(&self) -> Result<String, String> {
        Ok("token".into())
    }
    fn get_machine_id(&self) -> Result<String, String> {
        Ok("machine".into())
    }
}

struct MockBox;
impl AttachmentsBox for MockBox {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        Ok("running".into())
    }
    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        Ok(())
    }
}

fn create_test_service(root: PathBuf) -> AttachmentsService {
    AttachmentsService::new(
        root,
        Arc::new(MockAuth),
        Arc::new(MockBox),
        None,
    )
}

// These ports observe forbidden external work only. Policy admission, limits
// resolution, decoding and extraction still run through AttachmentsService.
struct NoExtractionSideEffects;

impl GenerateImageAuth for NoExtractionSideEffects {
    fn get_access_token(&self) -> Result<String, String> {
        panic!("input extraction must not request provider credentials");
    }

    fn get_machine_id(&self) -> Result<String, String> {
        panic!("input extraction must not request provider identity");
    }
}

impl AttachmentsBox for NoExtractionSideEffects {
    fn run_state(&self, _agent_id: &str) -> Result<String, String> {
        panic!("input extraction must not invoke the Box owner");
    }

    fn upload_file(&self, _agent_id: &str, _path: &str, _data: &[u8]) -> Result<(), String> {
        panic!("input extraction must not upload or stage a file");
    }
}

fn create_input_admission_service(root: PathBuf) -> AttachmentsService {
    AttachmentsService::new(
        root,
        Arc::new(NoExtractionSideEffects),
        Arc::new(NoExtractionSideEffects),
        None,
    )
}

const FILE_EXTRACTION_METHODS: [&str; 2] = [
    "extractFileContentFromBuffer",
    "extractFileContentFromSource",
];

// URL parsing fails before client construction, even if a regressed policy
// unexpectedly enables URL access. No test can reach a live network endpoint.
const INVALID_INPUT_URL: &str = "https://[";

fn file_extraction_args(method: &str, data: &str) -> Value {
    match method {
        "extractFileContentFromBuffer" => serde_json::json!({
            "bufferBase64": data,
            "filename": "policy.txt",
            "mimeType": "text/plain",
        }),
        "extractFileContentFromSource" => serde_json::json!({
            "source": {
                "type": "base64", "data": data,
                "filename": "policy.txt", "mediaType": "text/plain",
            },
        }),
        _ => panic!("unexpected file extraction method: {method}"),
    }
}

fn gateway_file_limits(service: &AttachmentsService, config: Value) -> Value {
    service.dispatch_gateway("resolveInputFileLimits", &config)
        .expect("actual limits resolver dispatch")
        .expect("resolved complete InputFileLimits")
}

fn assert_extraction_left_root_empty(root: &Path) {
    assert_eq!(fs::read_dir(root).expect("inspect extraction fixture").count(), 0,
        "pure input extraction must not create files or staging directories");
}

#[test]
fn test_staged_input_directory_and_ownership() {
    let hex_id = "c".repeat(64);
    let uuid_id = "11111111-1111-4111-8111-111111111111";

    assert_eq!(
        staged_input_directory(&hex_id),
        format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex_id}")
    );

    let hex_dir = staged_input_path_directory(&format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex_id}/input-secret.txt"));
    assert_eq!(hex_dir, Some(format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex_id}")));

    let uuid_dir = staged_input_path_directory(&format!("{STAGED_INPUT_DIRECTORY_PREFIX}{uuid_id}/file.pyc"));
    assert_eq!(uuid_dir, Some(format!("{STAGED_INPUT_DIRECTORY_PREFIX}{uuid_id}")));

    // Rejection of invalid candidate identities
    assert_eq!(staged_input_path_directory("media/inbound/other-dir/file.txt"), None);
    assert_eq!(staged_input_path_directory(&format!("{STAGED_INPUT_DIRECTORY_PREFIX}short/file.txt")), None);
    assert_eq!(staged_input_path_directory(&format!("{STAGED_INPUT_DIRECTORY_PREFIX}{}/file.txt", "C".repeat(64))), None);

    let mut dirs = HashSet::new();
    dirs.insert(format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex_id}"));
    assert!(is_staged_input_path(&format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex_id}/data.bin"), &dirs));
    assert!(!is_staged_input_path(&format!("{STAGED_INPUT_DIRECTORY_PREFIX}{uuid_id}/data.bin"), &dirs));

    // Staged manifest entries verification
    let entries = vec![
        StagedManifestEntry {
            path: format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex_id}/.gitignore"),
            r#type: "file".into(),
            size: Some(STAGED_INPUT_GITIGNORE.len()),
            sha256: Some(STAGED_INPUT_GITIGNORE_SHA256.into()),
        },
        StagedManifestEntry {
            path: format!("{STAGED_INPUT_DIRECTORY_PREFIX}{uuid_id}/.gitignore"),
            r#type: "file".into(),
            size: Some(10), // Wrong size
            sha256: Some(STAGED_INPUT_GITIGNORE_SHA256.into()),
        },
    ];
    let matched_dirs = staged_input_directories_from_entries(&entries);
    assert_eq!(matched_dirs.len(), 1);
    assert!(matched_dirs.contains(&format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex_id}")));

    // Ensure directory creation and ownership enforcement
    let tmp = TempTestDir::new("staged-dir-ownership");
    let target_dir = format!("{STAGED_INPUT_DIRECTORY_PREFIX}{uuid_id}");
    ensure_staged_input_directory(&tmp.path, &target_dir).expect("Directory should be created");

    let ignore_path = tmp.path.join(&target_dir).join(".gitignore");
    assert!(ignore_path.exists());
    assert_eq!(fs::read_to_string(&ignore_path).unwrap(), STAGED_INPUT_GITIGNORE);

    // Idempotent re-entry
    ensure_staged_input_directory(&tmp.path, &target_dir).expect("Idempotent check should succeed");

    // Tampering with .gitignore must fail ownership check
    fs::write(&ignore_path, "tampered content").unwrap();
    let err = ensure_staged_input_directory(&tmp.path, &target_dir).unwrap_err();
    assert!(err.contains("not owned by OpenClaw"));
}

#[test]
fn test_resolve_staged_input_media_paths() {
    let hex_id = "e".repeat(64);
    let dir = staged_input_directory(&hex_id);

    let p1 = format!("{dir}/input-file_analysis.csv");
    let p2 = format!("{dir}/input-file_dup.txt");
    let p3 = format!("{dir}/input-file_dup.json"); // Duplicate alias "file_dup"

    let facts = vec![
        MediaFact {
            path: Some(p1.clone()),
            staged: Some(true),
            ..Default::default()
        },
        MediaFact {
            path: Some(p2.clone()),
            staged: Some(true),
            ..Default::default()
        },
        MediaFact {
            path: Some(p3.clone()),
            staged: Some(true),
            ..Default::default()
        },
        // Not staged
        MediaFact {
            path: Some(format!("{dir}/input-file_not_staged.txt")),
            staged: Some(false),
            ..Default::default()
        },
        // Does not match file_ pattern
        MediaFact {
            path: Some(format!("{dir}/input-custom_name.txt")),
            staged: Some(true),
            ..Default::default()
        },
    ];

    let resolved = resolve_staged_input_media_paths(Some(&facts));
    assert_eq!(resolved.get("file_analysis.csv"), Some(&p1));
    assert_eq!(resolved.get("file_analysis"), Some(&p1));
    // Exact filenames with extensions remain unambiguous
    assert_eq!(resolved.get("file_dup.txt"), Some(&p2));
    assert_eq!(resolved.get("file_dup.json"), Some(&p3));
    // The shared stem "file_dup" is ambiguous and must be removed
    assert_eq!(resolved.get("file_dup"), None);
}

#[test]
fn test_named_attachment_decoding_and_charsets() {
    // UTF-8 decoding
    let utf8_text = "Café notes: résumé et météo pour demain.";
    let limits = resolve_input_file_limits(None);
    let res = extract_file_content_from_buffer(
        utf8_text.as_bytes(),
        Some("notes.txt"),
        Some("text/plain"),
        None,
        &limits,
        None,
    ).expect("UTF-8 text extraction should succeed");
    assert_eq!(res.text.as_deref(), Some(utf8_text));
    assert_eq!(res.metadata.as_ref().and_then(|m| m.text_truncated), None);

    // Windows-1252 decoding ("café €": 0x63, 0x61, 0x66, 0xe9, 0x20, 0x80)
    let w1252_bytes = vec![0x63, 0x61, 0x66, 0xe9, 0x20, 0x80];
    let res_w1252 = extract_file_content_from_buffer(
        &w1252_bytes,
        Some("data.txt"),
        Some("text/plain; charset=\"windows-1252\""),
        None,
        &limits,
        None,
    ).expect("Windows-1252 decode should succeed");
    assert_eq!(res_w1252.text.as_deref(), Some("café €"));

    // Recovery from unclosed quote in charset: text/plain; charset="windows-1252
    let res_unclosed = extract_file_content_from_buffer(
        &w1252_bytes,
        Some("data.txt"),
        Some("text/plain; charset=\"windows-1252"),
        None,
        &limits,
        None,
    ).expect("Unclosed quote recovery should succeed");
    assert_eq!(res_unclosed.text.as_deref(), Some("café €"));

    // Byte-detected UTF-16 with BOM overrides declared Windows-1252
    let mut utf16_bom = vec![0xff, 0xfe]; // UTF-16LE BOM
    for u in "café €".encode_utf16() {
        utf16_bom.extend_from_slice(&u.to_le_bytes());
    }
    let res_utf16 = extract_file_content_from_buffer(
        &utf16_bom,
        Some("utf16.txt"),
        Some("text/plain; charset=\"windows-1252\""),
        None,
        &limits,
        None,
    ).expect("UTF-16LE BOM decode should succeed");
    assert_eq!(res_utf16.text.as_deref(), Some("café €"));
}

#[test]
fn test_input_file_limits_and_truncation() {
    let limits_cfg = InputFileLimitsConfig {
        max_chars: Some(14),
        allowed_mimes: Some(vec!["text/plain".into()]),
        ..Default::default()
    };
    let limits = resolve_input_file_limits(Some(&limits_cfg));
    assert_eq!(limits.max_chars, 14);

    let content = "VISIBLE_PREFIX|INVISIBLE_TAIL";
    let res = extract_file_content_from_buffer(
        content.as_bytes(),
        Some("long.txt"),
        Some("text/plain"),
        None,
        &limits,
        None,
    ).expect("Extraction should succeed with truncation");

    assert_eq!(res.text.as_deref(), Some("VISIBLE_PREFIX"));
    let meta = res.metadata.expect("Metadata should be present when truncated");
    assert_eq!(meta.text_truncated, Some(true));
    assert_eq!(meta.images_truncated, Some(false));

    // UTF-16 safe surrogate slicing
    let emoji_text = "Hello 😀 world";
    assert_eq!(slice_utf16_safe(emoji_text, 0, Some(7)), "Hello "); // Cut before surrogate
    assert_eq!(slice_utf16_safe(emoji_text, 0, Some(8)), "Hello 😀");
    assert_eq!(truncate_utf16_safe(emoji_text, 7), "Hello ");
    assert_eq!(truncate_utf16_safe(emoji_text, 8), "Hello 😀");
}

#[test]
fn test_input_source_and_image_validation() {
    // Disabled URL sources
    let limits = resolve_input_file_limits(Some(&InputFileLimitsConfig {
        allow_url: Some(false),
        ..Default::default()
    }));

    let url_source = InputFileSource::Url {
        url: "https://example.com/file.txt".into(),
        media_type: Some("text/plain".into()),
        filename: Some("file.txt".into()),
    };
    let err = extract_file_content_from_source(&url_source, &limits).unwrap_err();
    assert!(err.contains("URL sources are disabled by config"));

    // File too large rejection
    let big_data = BASE64_STANDARD.encode(vec![0u8; 2048]);
    let big_source = InputFileSource::Base64 {
        data: big_data,
        media_type: Some("text/plain".into()),
        filename: Some("big.txt".into()),
    };
    let tiny_limits = resolve_input_file_limits(Some(&InputFileLimitsConfig {
        max_bytes: Some(1024),
        ..Default::default()
    }));
    let big_err = extract_file_content_from_source(&big_source, &tiny_limits).unwrap_err();
    assert!(big_err.contains("too large"));

    // AVI byte detection and rejection from image allowlist
    let mut avi_bytes = Vec::from(&b"RIFF\x38\x00\x00\x00AVI "[..]);
    avi_bytes.resize(64, 0);

    let classification = classify_attachment_bytes(&avi_bytes, None, None, Some("clip.avi"));
    assert_eq!(classification.class, AttachmentClass::Video);
    assert_eq!(classification.mime.as_deref(), Some("video/x-msvideo"));

    let image_limits = InputImageLimits {
        allow_url: false,
        url_allowlist: None,
        allowed_mimes: DEFAULT_INPUT_IMAGE_MIMES.iter().map(|s| s.to_string()).collect(),
        max_bytes: 1024 * 1024,
        max_redirects: 0,
        timeout_ms: 1000,
    };

    let avi_img_source = InputImageSource::Base64 {
        data: BASE64_STANDARD.encode(&avi_bytes),
        media_type: Some("image/jpeg".into()),
    };
    let avi_err = extract_image_content_from_source(&avi_img_source, &image_limits).unwrap_err();
    assert!(avi_err.contains("Unsupported image MIME type: video/x-msvideo"));

    // HEIC image normalization
    let heic_dummy = vec![0x00, 0x00, 0x00, 0x18, b'f', b't', b'y', b'p', b'h', b'e', b'i', b'c'];
    let normalized = normalize_input_image_buffer(&heic_dummy, Some("image/heic"), &image_limits)
        .expect("HEIC normalization should succeed");
    assert_eq!(normalized.mime_type, "image/jpeg");
    assert!(normalized.buffer.starts_with(b"\xff\xd8\xff"));
}

#[test]
fn test_gateway_dispatch_input_files_and_staged_inputs() {
    let tmp = TempTestDir::new("gateway-dispatch-input-files");
    let service = create_test_service(tmp.path.clone());

    // 1. resolveInputFileLimits
    let limits_res = service.dispatch_gateway(
        "resolveInputFileLimits",
        &serde_json::json!({ "maxChars": 5000, "allowedMimes": ["text/plain", "application/json"] }),
    ).expect("resolveInputFileLimits dispatch");
    let limits_val = limits_res.expect("Value returned");
    assert_eq!(limits_val.get("maxChars").and_then(Value::as_u64), Some(5000));

    // 2. extractFileContentFromBuffer
    let text = "Hello OpenClaw Unit 27 Staged Inputs";
    let b64 = BASE64_STANDARD.encode(text.as_bytes());
    let extract_res = service.dispatch_gateway(
        "extractFileContentFromBuffer",
        &serde_json::json!({
            "bufferBase64": b64,
            "filename": "hello.txt",
            "mimeType": "text/plain",
            "limits": limits_val
        }),
    ).expect("extractFileContentFromBuffer dispatch").expect("extract result");
    assert_eq!(extract_res.get("text").and_then(Value::as_str), Some(text));

    // 3. stagedInputPathDirectory
    let hex = "f".repeat(64);
    let path_res = service.dispatch_gateway(
        "stagedInputPathDirectory",
        &serde_json::json!({ "relativePath": format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex}/item.txt") }),
    ).expect("stagedInputPathDirectory dispatch").expect("directory result");
    assert_eq!(path_res.as_str(), Some(format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex}").as_str()));

    // 4. isStagedInputPath
    let dirs_json = serde_json::json!([format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex}")]);
    let match_res = service.dispatch_gateway(
        "isStagedInputPath",
        &serde_json::json!({
            "relativePath": format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex}/item.txt"),
            "directories": dirs_json
        }),
    ).expect("isStagedInputPath dispatch").expect("bool result");
    assert_eq!(match_res.as_bool(), Some(true));

    // 5. ensureStagedInputDirectory
    let dir_name = format!("{STAGED_INPUT_DIRECTORY_PREFIX}{hex}");
    let ensure_res = service.dispatch_gateway(
        "ensureStagedInputDirectory",
        &serde_json::json!({
            "rootDir": tmp.path.to_str().unwrap(),
            "directory": dir_name
        }),
    ).expect("ensureStagedInputDirectory dispatch").expect("success result");
    assert_eq!(ensure_res.get("success").and_then(Value::as_bool), Some(true));
}

#[test]
fn gateway_file_extraction_rejects_every_malformed_explicit_policy() {
    let tmp = TempTestDir::new("malformed-file-policy");
    let service = create_input_admission_service(tmp.path.clone());
    let complete = gateway_file_limits(&service, Value::Null);
    let data = BASE64_STANDARD.encode(b"policy must not silently fall back");

    let mut missing_field = complete.clone();
    missing_field.as_object_mut().expect("resolved policy object").remove("maxBytes");
    let mut incomplete_pdf = complete.clone();
    incomplete_pdf["pdf"].as_object_mut().expect("resolved PDF policy").remove("maxPixels");
    let mut mistyped_mime = complete.clone();
    mistyped_mime["allowedMimes"] = serde_json::json!(["text/plain", 7]);
    let mut mistyped_allowlist = complete.clone();
    mistyped_allowlist["urlAllowlist"] = serde_json::json!(false);
    let mut mistyped_byte_cap = complete.clone();
    mistyped_byte_cap["maxBytes"] = serde_json::json!(-1);
    let mut null_pdf = complete.clone();
    null_pdf["pdf"] = Value::Null;
    let mut restrictive_with_bad_pdf = gateway_file_limits(&service, serde_json::json!({
        "allowUrl": false, "maxBytes": 1, "maxChars": 0,
    }));
    restrictive_with_bad_pdf["allowedMimes"] = serde_json::json!([]);
    restrictive_with_bad_pdf["pdf"]["maxPages"] = serde_json::json!("invalid");

    // This is the complete positional shape accepted by Serde's struct
    // deserializer. The RPC contract requires an object, even for this shape.
    let positional_policy = serde_json::json!([
        complete["allowUrl"], Value::Null, complete["allowedMimes"],
        complete["maxBytes"], complete["maxChars"], complete["maxRedirects"],
        complete["timeoutMs"], complete["pdf"],
    ]);
    let malformed = [
        ("boolean", serde_json::json!(false)),
        ("number", serde_json::json!(7)),
        ("string", serde_json::json!("default")),
        ("empty sequence", serde_json::json!([])),
        ("complete positional sequence", positional_policy),
        ("empty object", serde_json::json!({})),
        ("partial config", serde_json::json!({"allowUrl": false, "maxChars": 0})),
        ("missing required field", missing_field),
        ("incomplete nested PDF policy", incomplete_pdf),
        ("mistyped MIME member", mistyped_mime),
        ("mistyped URL allowlist", mistyped_allowlist),
        ("negative byte cap", mistyped_byte_cap),
        ("null nested PDF policy", null_pdf),
        ("restrictive policy with invalid PDF field", restrictive_with_bad_pdf),
    ];

    for method in FILE_EXTRACTION_METHODS {
        for (label, limits) in &malformed {
            let mut args = file_extraction_args(method, &data);
            args["limits"] = limits.clone();
            let error = service.dispatch_gateway(method, &args)
                .expect("actual file RPC dispatch")
                .expect_err("malformed explicit policy must not become default extraction");
            assert!(error.starts_with("Invalid InputFileLimits"),
                "{method}, {label}: {error}");
            assert_extraction_left_root_empty(&tmp.path);
        }
    }
}

#[test]
fn gateway_file_policy_errors_precede_base64_and_url_processing() {
    let tmp = TempTestDir::new("file-policy-precedence");
    let service = create_input_admission_service(tmp.path.clone());
    let complete = gateway_file_limits(&service, Value::Null);
    let mut malformed = complete.clone();
    malformed["pdf"]["minTextChars"] = serde_json::json!("invalid");

    for method in FILE_EXTRACTION_METHODS {
        let mut args = file_extraction_args(method, "%%%invalid-base64%%%");
        args["limits"] = complete.clone();
        let payload_error = service.dispatch_gateway(method, &args)
            .expect("actual file RPC dispatch").expect_err("invalid base64 control");
        if method == "extractFileContentFromBuffer" {
            assert!(payload_error.starts_with("Invalid base64:"), "{payload_error}");
        } else {
            assert_eq!(payload_error, "input_file base64 source has invalid 'data' field");
        }

        args["limits"] = malformed.clone();
        let policy_error = service.dispatch_gateway(method, &args)
            .expect("actual file RPC dispatch").expect_err("policy must be checked first");
        assert!(policy_error.starts_with("Invalid InputFileLimits"),
            "{method}: policy refusal must precede {payload_error:?}, got {policy_error:?}");
    }

    let mut url_args = serde_json::json!({
        "source": {"type": "url", "url": INVALID_INPUT_URL, "filename": "policy.txt"},
        "limits": complete,
    });
    let url_error = service.dispatch_gateway("extractFileContentFromSource", &url_args)
        .expect("actual source RPC dispatch").expect_err("invalid URL control");
    assert!(url_error.contains("Invalid URL:"), "{url_error}");
    url_args["limits"] = malformed;
    let policy_error = service.dispatch_gateway("extractFileContentFromSource", &url_args)
        .expect("actual source RPC dispatch").expect_err("policy precedes URL processing");
    assert!(policy_error.starts_with("Invalid InputFileLimits"), "{policy_error}");
    assert_extraction_left_root_empty(&tmp.path);
}

#[test]
fn gateway_file_extraction_preserves_omitted_and_null_default_policies() {
    let tmp = TempTestDir::new("default-file-policy");
    let service = create_input_admission_service(tmp.path.clone());
    let text = "omitted and null limits preserve the default extraction policy";
    let data = BASE64_STANDARD.encode(text);

    for method in FILE_EXTRACTION_METHODS {
        for explicit_null in [false, true] {
            let mut args = file_extraction_args(method, &data);
            if explicit_null {
                args["limits"] = Value::Null;
            }
            let result = service.dispatch_gateway(method, &args)
                .expect("actual file RPC dispatch").expect("default policy extraction");
            assert_eq!(result["filename"], "policy.txt");
            assert_eq!(result["text"], text, "{method}, explicit null: {explicit_null}");
            assert!(result.get("metadata").is_none());
        }
    }

    for explicit_null in [false, true] {
        let mut args = serde_json::json!({
            "source": {"type": "url", "url": INVALID_INPUT_URL},
        });
        if explicit_null {
            args["limits"] = Value::Null;
        }
        let error = service.dispatch_gateway("extractFileContentFromSource", &args)
            .expect("actual source RPC dispatch").expect_err("invalid URL control");
        assert!(error.contains("Invalid URL:"),
            "default URL policy must reach syntax validation: {error}");
    }
    assert_extraction_left_root_empty(&tmp.path);
}

#[test]
fn gateway_file_extraction_preserves_resolved_restrictive_policies() {
    let tmp = TempTestDir::new("restrictive-file-policy");
    let service = create_input_admission_service(tmp.path.clone());
    let complete = gateway_file_limits(&service, Value::Null);
    let data = BASE64_STANDARD.encode(b"abc");

    for method in FILE_EXTRACTION_METHODS {
        let mut args = file_extraction_args(method, &data);
        let mut empty_mimes = complete.clone();
        empty_mimes["allowedMimes"] = serde_json::json!([]);
        args["limits"] = empty_mimes;
        let error = service.dispatch_gateway(method, &args)
            .expect("actual file RPC dispatch").expect_err("empty MIME set must remain empty");
        assert_eq!(error, "Unsupported file MIME type: text/plain", "{method}");

        args["limits"] = gateway_file_limits(&service, serde_json::json!({"maxBytes": 2}));
        let error = service.dispatch_gateway(method, &args)
            .expect("actual file RPC dispatch").expect_err("resolved byte cap must be enforced");
        assert_eq!(error, "File too large: 3 bytes (limit: 2 bytes)", "{method}");

        args["limits"] = gateway_file_limits(&service, serde_json::json!({"maxChars": 0}));
        let result = service.dispatch_gateway(method, &args)
            .expect("actual file RPC dispatch").expect("zero-character policy is valid");
        assert_eq!(result["text"], "", "{method}");
        assert_eq!(result["metadata"]["textTruncated"], true);
        assert_eq!(result["metadata"]["imagesTruncated"], false);
    }

    let no_urls = gateway_file_limits(&service, serde_json::json!({"allowUrl": false}));
    let error = service.dispatch_gateway("extractFileContentFromSource", &serde_json::json!({
        "source": {"type": "url", "url": INVALID_INPUT_URL},
        "limits": no_urls,
    })).expect("actual source RPC dispatch").expect_err("URL denial must remain effective");
    assert_eq!(error, "input_file URL sources are disabled by config");
    assert_extraction_left_root_empty(&tmp.path);
}

#[test]
fn gateway_partial_file_policy_is_resolved_before_extraction() {
    let tmp = TempTestDir::new("resolved-partial-file-policy");
    let service = create_input_admission_service(tmp.path.clone());
    let partial = serde_json::json!({
        "allowUrl": false, "maxBytes": 2, "maxChars": 1, "pdf": {"maxPages": 1},
    });
    let resolved = gateway_file_limits(&service, partial.clone());
    assert_eq!(resolved["allowUrl"], false);
    assert_eq!(resolved["maxBytes"], 2);
    assert_eq!(resolved["maxChars"], 1);
    assert_eq!(resolved["pdf"]["maxPages"], 1);
    assert!(resolved["pdf"]["maxPixels"].as_u64().is_some_and(|value| value > 0));
    let data = BASE64_STANDARD.encode(b"AB");

    for method in FILE_EXTRACTION_METHODS {
        let mut args = file_extraction_args(method, &data);
        args["limits"] = partial.clone();
        let error = service.dispatch_gateway(method, &args)
            .expect("actual file RPC dispatch").expect_err("partial config is not resolved limits");
        assert!(error.starts_with("Invalid InputFileLimits"), "{method}: {error}");

        args["limits"] = resolved.clone();
        let result = service.dispatch_gateway(method, &args)
            .expect("actual file RPC dispatch").expect("actual resolver output is admitted");
        assert_eq!(result["text"], "A", "{method}");
        assert_eq!(result["metadata"]["textTruncated"], true);
    }
    assert_extraction_left_root_empty(&tmp.path);
}

#[test]
fn gateway_padding_only_sources_preserve_upstream_error_precedence() {
    let tmp = TempTestDir::new("padding-only-sources");
    let service = create_input_admission_service(tmp.path.clone());
    let file_limits = gateway_file_limits(&service, serde_json::json!({"maxBytes": 0}));
    let image_limits = serde_json::json!({
        "allowUrl": false, "allowedMimes": ["image/png"],
        "maxBytes": 0, "maxRedirects": 0, "timeoutMs": 1,
    });

    for data in ["=", "==", " \t=\r\n", " \n==\t "] {
        for (method, limits, kind) in [
            ("extractFileContentFromSource", &file_limits, "input_file"),
            ("extractImageContentFromSource", &image_limits, "input_image"),
        ] {
            let args = serde_json::json!({
                "source": {"type": "base64", "data": data}, "limits": limits,
            });
            // A panic fails this case directly; an oversized error also fails
            // the exact assertion. Zero-byte policy must reach invalid data.
            let error = service.dispatch_gateway(method, &args)
                .expect("actual source RPC dispatch").expect_err("padding-only data is invalid");
            assert_eq!(error, format!("{kind} base64 source has invalid 'data' field"),
                "{method}, data: {data:?}");
            assert_extraction_left_root_empty(&tmp.path);
        }
    }

    // Fixed U checks the byte budget before canonical base64 validation.
    // Clamping the one-/two-padding underflow must not replace that ordering
    // with a blanket validation-first rule for longer malformed sources.
    for (data, estimated) in [("====", 1), ("========", 4), (" \t====\r\n", 1)] {
        for (method, limits, label) in [
            ("extractFileContentFromSource", &file_limits, "File"),
            ("extractImageContentFromSource", &image_limits, "Image"),
        ] {
            let args = serde_json::json!({
                "source": {"type": "base64", "data": data}, "limits": limits,
            });
            let error = service.dispatch_gateway(method, &args)
                .expect("actual source RPC dispatch").expect_err("zero-byte budget rejects estimate");
            assert_eq!(error, format!("{label} too large: {estimated} bytes (limit: 0 bytes)"),
                "{method}, data: {data:?}");
            assert_extraction_left_root_empty(&tmp.path);
        }
    }
}

#[test]
fn gateway_valid_padded_file_and_image_sources_preserve_byte_caps() {
    let tmp = TempTestDir::new("valid-padded-sources");
    let service = create_input_admission_service(tmp.path.clone());

    for (text, encoded) in [("A", "QQ=="), ("AB", "QUI=")] {
        assert_eq!(BASE64_STANDARD.encode(text), encoded, "valid padded control");
        for method in FILE_EXTRACTION_METHODS {
            let mut args = file_extraction_args(method, encoded);
            for cap in [text.len(), text.len() + 1] {
                args["limits"] = gateway_file_limits(&service, serde_json::json!({"maxBytes": cap}));
                let result = service.dispatch_gateway(method, &args)
                    .expect("actual file RPC dispatch").expect("valid padded data fits the cap");
                assert_eq!(result["text"], text, "{method}, cap: {cap}");
            }
            let cap = text.len() - 1;
            args["limits"] = gateway_file_limits(&service, serde_json::json!({"maxBytes": cap}));
            let error = service.dispatch_gateway(method, &args)
                .expect("actual file RPC dispatch").expect_err("valid padded data exceeds cap");
            assert_eq!(error, format!("File too large: {} bytes (limit: {cap} bytes)", text.len()));
        }
    }

    // A small PNG exercises the real image path without depending on the
    // unrelated HEIC conversion placeholder or any external image provider.
    const PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mP8/x8AAwMCAO+aVBEAAAAASUVORK5CYII=";
    let png_bytes = BASE64_STANDARD.decode(PNG).expect("valid padded PNG source");
    assert!(PNG.ends_with('='));
    let mut args = serde_json::json!({
        "source": {"type": "base64", "data": format!(" \t{PNG}\r\n"), "mediaType": "image/png"},
        "limits": {
            "allowUrl": false, "allowedMimes": ["image/png"],
            "maxBytes": png_bytes.len(), "maxRedirects": 0, "timeoutMs": 1,
        },
    });
    for cap in [png_bytes.len(), png_bytes.len() + 1] {
        args["limits"]["maxBytes"] = serde_json::json!(cap);
        let result = service.dispatch_gateway("extractImageContentFromSource", &args)
            .expect("actual image RPC dispatch").expect("valid padded image fits the cap");
        assert_eq!(result["type"], "image");
        assert_eq!(result["mimeType"], "image/png");
        assert_eq!(result["data"], PNG, "image data is canonicalized after surrounding whitespace");
    }
    let cap = png_bytes.len() - 1;
    args["limits"]["maxBytes"] = serde_json::json!(cap);
    let error = service.dispatch_gateway("extractImageContentFromSource", &args)
        .expect("actual image RPC dispatch").expect_err("valid padded image exceeds cap");
    assert_eq!(error, format!("Image too large: {} bytes (limit: {cap} bytes)", png_bytes.len()));
    assert_extraction_left_root_empty(&tmp.path);
}

fn input_url_admission_listener() -> (TcpListener, String) {
    // Keep ownership of a real endpoint for the duration of the synchronous
    // RPC. No worker or idle wait is needed: a refused initial request must
    // leave no connection in the listener's queue.
    let listener = TcpListener::bind(("127.0.0.1", 0)).expect("bind owned input admission endpoint");
    listener.set_nonblocking(true).expect("input admission observation must not wait");
    let url = format!("http://{}/input-source", listener.local_addr().unwrap());
    (listener, url)
}

fn assert_no_input_url_connection(listener: &TcpListener) {
    match listener.accept() {
        Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {}
        Ok(_) => panic!("refused input source attempted a connection to the owned endpoint"),
        Err(error) => panic!("cannot observe the owned input endpoint: {error}"),
    }
}

fn gateway_url_source_limits(service: &AttachmentsService, method: &str, allowlist: Value) -> Value {
    let mut limits = if method == "extractFileContentFromSource" {
        gateway_file_limits(service, serde_json::json!({"allowUrl": true, "timeoutMs": 1000}))
    } else {
        assert_eq!(method, "extractImageContentFromSource");
        serde_json::json!({
            "allowUrl": true, "allowedMimes": ["image/png"],
            "maxBytes": 1024, "maxRedirects": 0, "timeoutMs": 1000,
        })
    };
    // urlAllowlist belongs to the complete resolved policy, not the partial
    // InputFileLimitsConfig accepted by resolveInputFileLimits.
    limits["urlAllowlist"] = allowlist;
    limits
}

#[test]
fn gateway_input_url_allowlist_refuses_other_hosts_without_side_effects() {
    let temp = TempTestDir::new("url-off-list-admission");
    let service = create_input_admission_service(temp.path.clone());
    let (listener, url) = input_url_admission_listener();

    for method in ["extractFileContentFromSource", "extractImageContentFromSource"] {
        for allowlist in [
            serde_json::json!(["files.example.com"]),
            serde_json::json!([" *.EXAMPLE.COM... ", "", "*", "*.example.com"]),
        ] {
            let limits = gateway_url_source_limits(&service, method, allowlist);
            let error = service.dispatch_gateway(method, &serde_json::json!({
                "source": {"type": "url", "url": url, "filename": "policy.txt"},
                "limits": limits,
            })).expect("actual file/image source RPC dispatch")
                .expect_err("off-list source must be rejected before fetching");
            // The exact domain-policy cause proves that an ignored allowlist,
            // a later private-host refusal, or a network timeout cannot pass.
            assert_eq!(error, format!(
                "MediaFetchError: Failed to fetch media from {url}: Domain policy: Blocked hostname (not in allowlist): 127.0.0.1",
            ), "{method}");
            assert_no_input_url_connection(&listener);
            assert_extraction_left_root_empty(&temp.path);
        }
    }
}

#[test]
fn gateway_input_url_allowlist_never_grants_private_access() {
    let temp = TempTestDir::new("url-private-trust-separation");
    let service = create_input_admission_service(temp.path.clone());
    let (listener, url) = input_url_admission_listener();

    for method in ["extractFileContentFromSource", "extractImageContentFromSource"] {
        for allowlist in [
            serde_json::json!(["127.0.0.1"]),
            serde_json::json!([" 127.0.0.1... ", "127.0.0.1", "", "*"]),
            serde_json::json!([]),
            serde_json::json!(["", " ", "*", "*."]),
            Value::Null,
        ] {
            let limits = gateway_url_source_limits(&service, method, allowlist);
            let error = service.dispatch_gateway(method, &serde_json::json!({
                "source": {"type": "url", "url": url, "filename": "policy.txt"},
                "limits": limits,
            })).expect("actual file/image source RPC dispatch")
                .expect_err("matching or unrestricted hostname policy must not grant private trust");
            assert_eq!(error, format!(
                "MediaFetchError: Failed to fetch media from {url}: Blocked hostname or private/internal/special-use IP address",
            ), "{method}: a timeout or unrelated failure is not private-host refusal");
            assert_no_input_url_connection(&listener);
            assert_extraction_left_root_empty(&temp.path);
        }
    }
}
