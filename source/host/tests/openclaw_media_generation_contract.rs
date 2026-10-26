//! OpenClaw Media Understanding, Generation & Attachments Native Rust Fusion & Contract Tests (OCRM-M23/M24)
//!
//! Validates the 12 falsifiable test oracles defined for OCRM Media Generation & Attachments:
//! - OCRM-ATTACH-O01: ATTACHMENT_BYTE_LIMIT (25 MiB) and VIDEO_BYTE_LIMIT (200 MiB) constants.
//! - OCRM-ATTACH-O02: Directory name constants (ATTACHMENTS_DIRNAME, ASSETS_DIRNAME).
//! - OCRM-ATTACH-O03: SandGenerateImageResourceAccessor path containment and assets isolation.
//! - OCRM-ATTACH-O04: resolve_default_media_model and resolve_auto_media_key_providers capability & priority resolution.
//! - OCRM-ATTACH-O05: SandGenerateImageService successful image generation and persistence.
//! - OCRM-ATTACH-O06: SandGenerateImageService failure on persistence rejection (Persist error).
//! - OCRM-ATTACH-O07: SandGenerateImageService rejection on invalid base64 image data.
//! - OCRM-ATTACH-O08: normalize_image_generation_resolution normalization (1K, 2K, 4K) and invalid value rejection.
//! - OCRM-ATTACH-O09: SandGenerateImageToolBridge tool call resolution validation and rejection.
//! - OCRM-ATTACH-O10: sniff_mime_from_base64 prefix-bounded detection and magic byte detection.
//! - OCRM-ATTACH-O11: AttachmentTextPreview real filesystem truncation (>64 KiB) and binary classification.
//! - OCRM-ATTACH-O12: Host Extension ID and dependency graph integrity for AttachmentsExtension.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use serde_json::{Value, json};

use mahayana_host_runtime::attachment_paths::{
    ASSETS_DIRNAME, ATTACHMENTS_DIRNAME, get_agent_attachments_dir,
};
use mahayana_host_runtime::extensions::attachments::attachments_service::{
    AttachmentTextPreview, read_attachment_text_with_root,
};
use mahayana_host_runtime::extensions::attachments::extension::{
    ATTACHMENTS_DEPENDENCIES, ATTACHMENTS_EXTENSION_ID,
};
use mahayana_host_runtime::extensions::attachments::generate_image_resource_accessor::{
    GenerateImageResourceResult, SandGenerateImageResourceAccessor,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::{
    GenerateImageAuth, GenerateImageBackendWithOptions, GeneratedImage, PersistedImage,
    SandGenerateImageError, SandGenerateImageExecutionOptions, SandGenerateImageService,
};
use mahayana_host_runtime::runner::host_generate_image_dependencies::ProductionGenerateImageToolExecutor;
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::inference::provider_session::{
    ProviderSessionError, RoutedToolDefinition,
};
use mahayana_host_runtime::media_mime::{
    ATTACHMENT_BYTE_LIMIT, BASE64_SNIFF_PREFIX_CHARS, MimeSniffHints, VIDEO_BYTE_LIMIT,
    inspect_canonical_base64, sniff_mime_from_base64, sniff_mime_from_base64_with_hints,
    sniff_mime_from_buffer,
};
use mahayana_host_runtime::media_understanding_defaults::{
    MediaUnderstandingCapability, MediaUnderstandingProviderMetadata,
    resolve_auto_media_key_providers, resolve_default_media_model,
};
use mahayana_host_runtime::runner::box_tool_access::{
    RunnerBoxReadRequest, RunnerBoxResourcePort, RunnerBoxShellRequest, RunnerBoxWriteRequest,
};
use mahayana_host_runtime::runner::routed_provider_runtime::RoutedToolBridge;
use mahayana_host_runtime::runner::shell_terminal_watch::{
    ShellTerminalPollRead, TerminalReadResult,
};
use mahayana_host_runtime::runner::tools::sand_generate_image_tool::{
    GenerateImageToolExecutor, GenerateImageToolOptions, GeneratedImageToolOutput,
    GENERATE_IMAGE_TOOL_NAME, ImageGenerationResolution, SandGenerateImageToolBridge,
    normalize_image_generation_resolution,
};

fn temp_agent_root(label: &str) -> std::path::PathBuf {
    let suffix = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock")
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "fabushi-media-contract-{label}-{}-{suffix}",
        std::process::id()
    ));
    fs::create_dir_all(&dir).expect("create temp dir");
    dir
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O01: Attachment and Video Byte Limits
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o01_byte_limits() {
    assert_eq!(
        ATTACHMENT_BYTE_LIMIT, 26_214_400,
        "Attachment limit must be exactly 25 MiB"
    );
    assert_eq!(
        VIDEO_BYTE_LIMIT, 209_715_200,
        "Video limit must be exactly 200 MiB"
    );
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O02: Directory Name Constants
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o02_directory_names() {
    assert_eq!(ATTACHMENTS_DIRNAME, "attachments");
    assert_eq!(ASSETS_DIRNAME, "assets");
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O03: SandGenerateImageResourceAccessor Path Containment
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o03_resource_accessor_containment() {
    let root = temp_agent_root("accessor");
    let accessor = SandGenerateImageResourceAccessor::new(&root);

    // Attempt writing outside agent media store (e.g. system root)
    let outside_path = std::env::temp_dir().join("malicious_escape.png");
    let result = accessor.write(&outside_path, b"fake png data").expect("write call");

    match result {
        GenerateImageResourceResult::Error { error, .. } => {
            assert!(
                error.contains("outside the agent's media store"),
                "Writing outside media store must be rejected: {error}"
            );
        }
        GenerateImageResourceResult::Success { .. } => {
            panic!("Expected rejection for path outside media store");
        }
    }
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O04: Media Understanding Default Models & Auto Priority Order
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o04_media_understanding_default_models() {
    let mut registry = HashMap::new();

    let mut google_defaults = HashMap::new();
    google_defaults.insert("image".to_string(), "gemini-2.5-flash".to_string());
    let mut google_prio = HashMap::new();
    google_prio.insert("image".to_string(), 30);
    registry.insert(
        "google".to_string(),
        MediaUnderstandingProviderMetadata {
            id: "google".to_string(),
            capabilities: vec![MediaUnderstandingCapability::Image],
            default_models: google_defaults,
            auto_priority: google_prio,
        },
    );

    let mut minimax_defaults = HashMap::new();
    minimax_defaults.insert("image".to_string(), "MiniMax-M2.7".to_string());
    let mut minimax_prio = HashMap::new();
    minimax_prio.insert("image".to_string(), 40);
    registry.insert(
        "minimax".to_string(),
        MediaUnderstandingProviderMetadata {
            id: "minimax".to_string(),
            capabilities: vec![MediaUnderstandingCapability::Image],
            default_models: minimax_defaults,
            auto_priority: minimax_prio,
        },
    );

    let mut openrouter_defaults = HashMap::new();
    openrouter_defaults.insert("image".to_string(), "auto".to_string());
    let mut openrouter_prio = HashMap::new();
    openrouter_prio.insert("image".to_string(), 50);
    registry.insert(
        "openrouter".to_string(),
        MediaUnderstandingProviderMetadata {
            id: "openrouter".to_string(),
            capabilities: vec![MediaUnderstandingCapability::Image],
            default_models: openrouter_defaults,
            auto_priority: openrouter_prio,
        },
    );

    let mut configured_models = HashMap::new();
    configured_models.insert(
        "openrouter".to_string(),
        vec!["google/gemini-2.5-flash".to_string()],
    );

    // 1. Configured image models take precedence when include_configured_models is true
    let model = resolve_default_media_model(
        "openrouter",
        MediaUnderstandingCapability::Image,
        &registry,
        Some(&configured_models),
        true,
    );
    assert_eq!(
        model.as_deref(),
        Some("google/gemini-2.5-flash"),
        "Configured model override must be preferred"
    );

    // 2. When include_configured_models is false, manifest default is returned
    let default_model = resolve_default_media_model(
        "openrouter",
        MediaUnderstandingCapability::Image,
        &registry,
        Some(&configured_models),
        false,
    );
    assert_eq!(
        default_model.as_deref(),
        Some("auto"),
        "Should fallback to registry default model"
    );

    // 3. Auto discovery priority order matches priorities (30, 40, 50)
    let auto_providers = resolve_auto_media_key_providers(
        MediaUnderstandingCapability::Image,
        &registry,
        None,
    );
    assert_eq!(
        auto_providers,
        vec!["google", "minimax", "openrouter"],
        "Providers must be sorted in ascending order of autoPriority"
    );
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O05: Successful Generation and Persistence
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o05_generation_and_persistence() {
    let persisted_records = Arc::new(Mutex::new(Vec::new()));
    let capture = Arc::clone(&persisted_records);

    let raw_bytes = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR";
    let encoded_b64 = STANDARD.encode(raw_bytes);

    let service = SandGenerateImageService::new(
        Arc::new(move |desc, refs| {
            assert_eq!(desc, "futuristic city at night");
            assert!(refs.is_empty());
            Ok(GeneratedImage {
                image_data: encoded_b64.clone(),
                mime_type: "image/png".to_string(),
            })
        }),
        Arc::new(move |bytes, mime| {
            capture.lock().unwrap().push((bytes.to_vec(), mime.to_string()));
            Ok(Some(PersistedImage {
                absolute_path: "/store/agents/a1/assets/city.png".to_string(),
            }))
        }),
    );

    let (path, returned_b64) = service
        .generate("futuristic city at night", &[])
        .expect("generation succeeds");

    assert_eq!(path, "/store/agents/a1/assets/city.png");
    assert_eq!(STANDARD.decode(returned_b64).expect("decode"), raw_bytes);
    assert_eq!(persisted_records.lock().unwrap().len(), 1);
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O06: Persistence Failure Handling
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o06_persistence_failure() {
    let service = SandGenerateImageService::new(
        Arc::new(|_, _| {
            Ok(GeneratedImage {
                image_data: STANDARD.encode(b"valid bytes"),
                mime_type: "image/png".to_string(),
            })
        }),
        Arc::new(|_, _| Ok(None)), // Simulating persistence rejection
    );

    let error = service
        .generate("prompt", &[])
        .expect_err("should fail when persistence returns None");

    assert_eq!(error, SandGenerateImageError::Persist);
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O07: Invalid Base64 Handling
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o07_invalid_base64() {
    let service = SandGenerateImageService::new(
        Arc::new(|_, _| {
            Ok(GeneratedImage {
                image_data: "not-valid-base64-!@#$%^&*".to_string(),
                mime_type: "image/png".to_string(),
            })
        }),
        Arc::new(|_, _| {
            Ok(Some(PersistedImage {
                absolute_path: "/dummy/path".to_string(),
            }))
        }),
    );

    let error = service
        .generate("prompt", &[])
        .expect_err("should fail on invalid base64 data");

    assert_eq!(error, SandGenerateImageError::InvalidBase64);
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O08: Image Resolution Normalization and Rejection
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o08_resolution_normalization() {
    // Valid values (case-insensitive, trimmed)
    assert_eq!(
        normalize_image_generation_resolution(Some("1K")).unwrap(),
        Some(ImageGenerationResolution::R1K)
    );
    assert_eq!(
        normalize_image_generation_resolution(Some(" 2k \t")).unwrap(),
        Some(ImageGenerationResolution::R2K)
    );
    assert_eq!(
        normalize_image_generation_resolution(Some("4k")).unwrap(),
        Some(ImageGenerationResolution::R4K)
    );

    // Empty or omitted resolution returns None
    assert_eq!(normalize_image_generation_resolution(None).unwrap(), None);
    assert_eq!(
        normalize_image_generation_resolution(Some("")).unwrap(),
        None
    );
    assert_eq!(
        normalize_image_generation_resolution(Some("   ")).unwrap(),
        None
    );

    // Invalid resolutions must be rejected with the exact upstream error
    let err = normalize_image_generation_resolution(Some("8K")).unwrap_err();
    match err {
        ProviderSessionError::Tool(msg) => {
            assert!(
                msg.contains("resolution must be one of 1K, 2K, or 4K"),
                "Unexpected error message: {msg}"
            );
        }
        other => panic!("Expected ProviderSessionError::Tool, got {other:?}"),
    }

    let err2 = normalize_image_generation_resolution(Some("1080p")).unwrap_err();
    match err2 {
        ProviderSessionError::Tool(msg) => {
            assert!(msg.contains("resolution must be one of 1K, 2K, or 4K"));
        }
        other => panic!("Expected ProviderSessionError::Tool, got {other:?}"),
    }
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O09: Tool Call Resolution Validation & Rejection
// ---------------------------------------------------------------------------
struct TestToolDelegate;
impl RoutedToolBridge for TestToolDelegate {
    fn list_tools(&self) -> Result<Vec<RoutedToolDefinition>, ProviderSessionError> {
        Ok(Vec::new())
    }
    fn call_tool(
        &self,
        _tool: &RoutedToolDefinition,
        _args: Value,
        _tool_call_id: &str,
    ) -> Result<Value, ProviderSessionError> {
        Err(ProviderSessionError::Tool("unexpected delegate invocation".into()))
    }
}

struct TestImageExecutor {
    last_received_options: Mutex<Option<GenerateImageToolOptions>>,
}

impl GenerateImageToolExecutor for TestImageExecutor {
    fn generate_with_options(
        &self,
        _description: &str,
        _references: &[(String, String)],
        options: &GenerateImageToolOptions,
    ) -> Result<GeneratedImageToolOutput, ProviderSessionError> {
        *self.last_received_options.lock().unwrap() = Some(options.clone());
        Ok(GeneratedImageToolOutput {
            persisted_path: "/workspace/assets/test-image.png".to_string(),
            image_data_base64: STANDARD.encode(b"\x89PNG\r\n\x1a\n"),
        })
    }
}

#[derive(Default)]
struct TestBoxPort {
    writes: Mutex<Vec<(String, Vec<u8>, String)>>,
}
impl RunnerBoxResourcePort for TestBoxPort {
    fn execute_shell(
        &self,
        _request: RunnerBoxShellRequest,
    ) -> Result<Value, ProviderSessionError> {
        unreachable!()
    }
    fn execute_read(
        &self,
        _request: RunnerBoxReadRequest,
    ) -> Result<Value, ProviderSessionError> {
        Ok(json!({"kind": "fileNotFound"}))
    }
    fn execute_write(
        &self,
        request: RunnerBoxWriteRequest,
    ) -> Result<(), ProviderSessionError> {
        self.writes.lock().unwrap().push((
            request.path,
            request.data,
            request.tool_call_id,
        ));
        Ok(())
    }
    fn poll_background_shell_terminal(&self, _shell_id: &str) -> ShellTerminalPollRead {
        ShellTerminalPollRead::Snapshot {
            output_path: "/tmp/test-out".into(),
            result: TerminalReadResult::FileNotFound,
        }
    }
}

#[derive(Clone)]
struct TestGenerateImageAuth;

impl GenerateImageAuth for TestGenerateImageAuth {
    fn get_access_token(&self) -> Result<String, String> {
        Ok("test-access-token".into())
    }

    fn get_machine_id(&self) -> Result<String, String> {
        Ok("test-machine-id".into())
    }
}

#[test]
fn test_ocrm_attach_o09_tool_resolution_validation_and_rejection() {
    let box_port = Arc::new(TestBoxPort::default());
    let executor = Arc::new(TestImageExecutor {
        last_received_options: Mutex::new(None),
    });
    let bridge = SandGenerateImageToolBridge::new(
        Arc::new(TestToolDelegate),
        executor.clone(),
        box_port,
    );

    let tool_def = RoutedToolDefinition {
        name: GENERATE_IMAGE_TOOL_NAME.into(),
        provider_identifier: "test".into(),
        tool_name: GENERATE_IMAGE_TOOL_NAME.into(),
        description: None,
        input_schema: json!({}),
    };

    // 1. Call with invalid resolution must be rejected before executor
    let invalid_call = bridge.call_tool(
        &tool_def,
        json!({
            "description": "drawing of a cat",
            "resolution": "8K"
        }),
        "call_1",
    );
    assert!(invalid_call.is_err(), "Invalid resolution must fail");
    let err_str = invalid_call.unwrap_err().to_string();
    assert!(
        err_str.contains("resolution must be one of 1K, 2K, or 4K"),
        "Error must mention 1K, 2K, or 4K: {err_str}"
    );
    assert_eq!(
        *executor.last_received_options.lock().unwrap(),
        None,
        "Rejected call must never invoke executor"
    );

    // 2. Call with valid resolution must succeed, propagate options to executor, and echo in output
    let valid_call = bridge.call_tool(
        &tool_def,
        json!({
            "description": "drawing of a cat",
            "resolution": "2K"
        }),
        "call_2",
    ).expect("valid call succeeds");

    assert_eq!(
        valid_call["success"]["resolution"].as_str(),
        Some("2K"),
        "Success payload must reflect normalized resolution"
    );

    let received_options = executor.last_received_options.lock().unwrap().clone();
    assert_eq!(
        received_options,
        Some(GenerateImageToolOptions {
            resolution: Some(ImageGenerationResolution::R2K),
            aspect_ratio: None,
        }),
        "Executor must receive the parsed and normalized resolution in options"
    );

    // 3. Positive Oracle: Provider that supports options receives resolution parameter end-to-end
    let captured_provider_resolution = Arc::new(Mutex::new(None));
    let cap_res = Arc::clone(&captured_provider_resolution);
    let sample_png_b64 = STANDARD.encode(b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR");
    let provider_backend: GenerateImageBackendWithOptions = Arc::new(move |_desc, _refs, opts| {
        *cap_res.lock().unwrap() = opts.resolution.clone();
        Ok(GeneratedImage {
            image_data: sample_png_b64.clone(),
            mime_type: "image/png".to_string(),
        })
    });
    let persist_callback = Arc::new(|_bytes: &[u8], _mime: &str| {
        Ok(Some(PersistedImage {
            absolute_path: "/workspace/assets/out.png".to_string(),
        }))
    });
    let service_with_options = SandGenerateImageService::new_with_options(
        provider_backend,
        persist_callback.clone(),
    );
    let (persisted_path, _) = service_with_options
        .generate_with_options(
            "render landscape",
            &[],
            &SandGenerateImageExecutionOptions {
                resolution: Some("4K".to_string()),
            },
        )
        .expect("generation succeeds");
    assert_eq!(persisted_path, "/workspace/assets/out.png");
    assert_eq!(
        *captured_provider_resolution.lock().unwrap(),
        Some("4K".to_string()),
        "Provider backend must receive the 4K resolution parameter"
    );

    // 4. Negative Oracle: Unsupported provider explicitly fails rather than silently dropping options
    let legacy_backend = Arc::new(|_desc: &str, _refs: &[(String, String)]| {
        Ok(GeneratedImage {
            image_data: STANDARD.encode(b"\x89PNG\r\n\x1a\n"),
            mime_type: "image/png".to_string(),
        })
    });
    let unsupporting_service = SandGenerateImageService::new(legacy_backend, persist_callback.clone());
    let unsupported_err = unsupporting_service
        .generate_with_options(
            "render landscape",
            &[],
            &SandGenerateImageExecutionOptions {
                resolution: Some("2K".to_string()),
            },
        )
        .unwrap_err();
    match unsupported_err {
        SandGenerateImageError::Generate(msg) => {
            assert!(
                msg.contains("Backend provider does not support resolution parameter: 2K"),
                "Must explicitly report unsupported resolution: {msg}"
            );
        }
        other => panic!("Expected SandGenerateImageError::Generate, got {other:?}"),
    }

    // 5. Production executor wiring: ProductionGenerateImageToolExecutor propagates options to service
    let prod_executor = ProductionGenerateImageToolExecutor::new(
        Arc::new(TestGenerateImageAuth),
        persist_callback,
    );
    let prod_err = prod_executor
        .generate_with_options(
            "render landscape",
            &[],
            &GenerateImageToolOptions {
                resolution: Some(ImageGenerationResolution::R2K),
                aspect_ratio: None,
            },
        )
        .unwrap_err();
    let prod_err_msg = prod_err.to_string();
    assert!(
        prod_err_msg.contains("does not support resolution parameter: 2K"),
        "Production executor must pass resolution to service and report provider rejection: {prod_err_msg}"
    );
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O10: Prefix-bounded Base64 MIME Sniffing, Canonical Pad Bits & Hints
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o10_sniff_mime_from_base64() {
    // 1. Valid magic byte payloads
    let png_b64 = STANDARD.encode(b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR");
    assert_eq!(
        sniff_mime_from_base64(&png_b64).as_deref(),
        Some("image/png")
    );

    let jpeg_b64 = STANDARD.encode(b"\xff\xd8\xff\xe0\x00\x10JFIF");
    assert_eq!(
        sniff_mime_from_base64(&jpeg_b64).as_deref(),
        Some("image/jpeg")
    );

    let webp_b64 = STANDARD.encode(b"RIFF\x24\x00\x00\x00WEBPVP8 ");
    assert_eq!(
        sniff_mime_from_base64(&webp_b64).as_deref(),
        Some("image/webp")
    );

    let pdf_b64 = STANDARD.encode(b"%PDF-1.4\n%header");
    assert_eq!(
        sniff_mime_from_base64(&pdf_b64).as_deref(),
        Some("application/pdf")
    );

    // 2. Tail corruption test: valid PNG prefix, but trailing garbage
    let corrupted_tail = format!("{png_b64}invalid_tail_chars!@#$");
    assert_eq!(
        sniff_mime_from_base64(&corrupted_tail),
        None,
        "Tail corruption must cause the entire payload to be rejected"
    );

    // 3. Non-canonical pad bits tests (RFC 4648 canonicalBase64 inspection)
    // 2-pad ('=='):
    // 'AA==' (data byte 0, pad bits 0) -> valid
    assert!(inspect_canonical_base64("AA==").is_some());
    // 'AB==' (value 1, pad bits 1 != 0) -> non-canonical
    assert!(inspect_canonical_base64("AB==").is_none());
    assert_eq!(sniff_mime_from_base64("AB=="), None);
    // '/w==' (value 48 = 0b110000, lowest 4 bits = 0) -> canonical
    assert!(inspect_canonical_base64("/w==").is_some());
    // '//==' (value 63 = 0b111111, lowest 4 bits = 0b1111 != 0) -> non-canonical
    assert!(inspect_canonical_base64("//==").is_none());
    assert_eq!(sniff_mime_from_base64("//=="), None);

    // 1-pad ('='):
    // '//8=' (char 3 is '8' = 60 = 0b111100, lowest 2 bits = 0) -> canonical
    assert!(inspect_canonical_base64("//8=").is_some());
    // '///=' (char 3 is '/' = 63 = 0b111111, lowest 2 bits = 0b11 != 0) -> non-canonical
    assert!(inspect_canonical_base64("///=").is_none());
    assert_eq!(sniff_mime_from_base64("///="), None);

    // 4. Slice length < 8 chars returns None (too short for magic bytes)
    assert_eq!(sniff_mime_from_base64("AQIDBA=="), None);
    assert_eq!(sniff_mime_from_base64(""), None);

    // 5. Non-magic bytes without hints returns None
    let text_b64 = STANDARD.encode(b"Hello world, this is plain text!");
    assert_eq!(sniff_mime_from_base64(&text_b64), None);

    // 6. Hints fallback: non-magic payload falls back to header/path hints
    let hints_header = MimeSniffHints {
        header_mime: Some("image/svg+xml"),
        file_path: None,
        additional_hints: &[],
    };
    assert_eq!(
        sniff_mime_from_base64_with_hints(&text_b64, &hints_header).as_deref(),
        Some("image/svg+xml"),
        "When buffer lacks magic bytes and file path, headerMime hint must be used"
    );

    // 7. Path extension takes precedence over headerMime hint (matching upstream detectMime)
    let hints_path_precedence = MimeSniffHints {
        header_mime: Some("image/svg+xml"),
        file_path: Some(Path::new("recording.mp4")),
        additional_hints: &[],
    };
    assert_eq!(
        sniff_mime_from_base64_with_hints(&text_b64, &hints_path_precedence).as_deref(),
        Some("video/mp4"),
        "File path extension must take precedence over header hint when magic bytes are absent"
    );

    // 8. Hints conflict: magic bytes take precedence over conflicting header hint and file path
    let conflicting_hints = MimeSniffHints {
        header_mime: Some("image/jpeg"),
        file_path: Some(Path::new("document.pdf")),
        additional_hints: &[],
    };
    assert_eq!(
        sniff_mime_from_base64_with_hints(&png_b64, &conflicting_hints).as_deref(),
        Some("image/png"),
        "Magic bytes must take precedence over conflicting header hint and path"
    );

    // 9. Prefix bound: longer payloads only inspect up to BASE64_SNIFF_PREFIX_CHARS
    assert_eq!(BASE64_SNIFF_PREFIX_CHARS, 256);
    assert_eq!(sniff_mime_from_buffer(b"\x89PNG\r\n\x1a\n"), Some("image/png"));
    assert_eq!(sniff_mime_from_buffer(b"\xff\xd8\xff"), Some("image/jpeg"));
    assert_eq!(sniff_mime_from_buffer(b"%PDF-"), Some("application/pdf"));
    let mut large_payload = b"\x89PNG\r\n\x1a\n\x00\x00\x00\rIHDR".to_vec();
    large_payload.resize(10_000, 0x55);
    let large_b64 = STANDARD.encode(&large_payload);
    assert_eq!(
        sniff_mime_from_base64(&large_b64).as_deref(),
        Some("image/png")
    );
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O11: Attachment Text Preview Real Truncation and Binary
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o11_attachment_text_preview_truncation_and_binary() {
    let sand_root = temp_agent_root("sand_preview");
    let agent_dir = sand_root.join("agent_test");
    let attachments_dir = get_agent_attachments_dir(&agent_dir);
    fs::create_dir_all(&attachments_dir).expect("create attachments dir");

    // 1. Large text file (> 64 KiB) should be read and marked truncated = true
    let large_text_path = attachments_dir.join("large_log.txt");
    let large_content = "Line of log entry text to exceed preview bounds.\n".repeat(1500);
    assert!(large_content.len() > 64 * 1024);
    fs::write(&large_text_path, &large_content).expect("write large text file");

    let preview = read_attachment_text_with_root(&sand_root, &agent_dir, &large_text_path)
        .expect("text preview available");

    match preview {
        AttachmentTextPreview::Text { text, truncated, bytes } => {
            assert!(truncated, "Text exceeding 64 KiB must have truncated = true");
            assert_eq!(bytes, large_content.len() as u64);
            assert!(
                text.len() <= 64 * 1024,
                "Returned preview text must be bounded by byte cap"
            );
        }
        AttachmentTextPreview::Binary { .. } => panic!("Expected Text preview variant for text file"),
    }

    // 2. Binary file (containing binary / null bytes) must return Binary variant
    let binary_path = attachments_dir.join("data.bin");
    let binary_content = vec![0u8, 159u8, 255u8, 0u8, 128u8, 32u8, 0u8, 0u8];
    fs::write(&binary_path, &binary_content).expect("write binary file");

    let bin_preview = read_attachment_text_with_root(&sand_root, &agent_dir, &binary_path)
        .expect("binary preview available");

    match bin_preview {
        AttachmentTextPreview::Binary { bytes } => {
            assert_eq!(bytes, binary_content.len() as u64);
        }
        AttachmentTextPreview::Text { .. } => panic!("Expected Binary preview variant for binary file"),
    }

    // 3. Small text file (< 64 KiB) should have truncated = false
    let small_text_path = attachments_dir.join("small_notes.txt");
    fs::write(&small_text_path, "Hello world").expect("write small text file");

    let small_preview = read_attachment_text_with_root(&sand_root, &agent_dir, &small_text_path)
        .expect("small text preview available");

    match small_preview {
        AttachmentTextPreview::Text { text, truncated, bytes } => {
            assert!(!truncated, "Small text file must not be truncated");
            assert_eq!(text, "Hello world");
            assert_eq!(bytes, 11);
        }
        AttachmentTextPreview::Binary { .. } => panic!("Expected Text preview variant"),
    }
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O12: Attachments Extension ID and Dependency Graph
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o12_extension_id_and_dependencies() {
    assert_eq!(ATTACHMENTS_EXTENSION_ID, HostExtensionId::Attachments);
    assert!(ATTACHMENTS_DEPENDENCIES.contains(&HostExtensionId::Auth));
    assert!(ATTACHMENTS_DEPENDENCIES.contains(&HostExtensionId::ForeverBox));
    assert!(ATTACHMENTS_DEPENDENCIES.contains(&HostExtensionId::Telemetry));
}
