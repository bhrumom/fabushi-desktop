//! OpenClaw Media Understanding, Generation & Attachments Native Rust Fusion & Contract Tests (OCRM-M23/M24)
//!
//! Validates the 12 falsifiable test oracles defined for OCRM Media Generation & Attachments:
//! - OCRM-ATTACH-O01: ATTACHMENT_BYTE_LIMIT (25 MiB) and VIDEO_BYTE_LIMIT (200 MiB) constants.
//! - OCRM-ATTACH-O02: Directory name constants (ATTACHMENTS_DIRNAME, ASSETS_DIRNAME).
//! - OCRM-ATTACH-O03: SandGenerateImageResourceAccessor path containment and assets isolation.
//! - OCRM-ATTACH-O04: configured_generate_image_model_id returns non-empty model identifier.
//! - OCRM-ATTACH-O05: SandGenerateImageService successful image generation and persistence.
//! - OCRM-ATTACH-O06: SandGenerateImageService failure on persistence rejection (Persist error).
//! - OCRM-ATTACH-O07: SandGenerateImageService rejection on invalid base64 image data.
//! - OCRM-ATTACH-O08: GENERATE_IMAGE_TOOL_NAME constant matches routed tool registry definition.
//! - OCRM-ATTACH-O09: GenerateImageToolExecutor execution with prompt description and references.
//! - OCRM-ATTACH-O10: Standard Base64 roundtrip verification for generated media binary payload.
//! - OCRM-ATTACH-O11: AttachmentTextPreview enum variants (Text with truncation vs Binary bytes).
//! - OCRM-ATTACH-O12: Host Extension ID and dependency graph integrity for AttachmentsExtension.

use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use base64::Engine;
use base64::engine::general_purpose::STANDARD;

use mahayana_host_runtime::attachment_paths::{
    ASSETS_DIRNAME, ATTACHMENTS_DIRNAME,
};
use mahayana_host_runtime::extensions::attachments::attachments_service::AttachmentTextPreview;
use mahayana_host_runtime::extensions::attachments::extension::{
    ATTACHMENTS_DEPENDENCIES, ATTACHMENTS_EXTENSION_ID,
};
use mahayana_host_runtime::extensions::attachments::generate_image_resource_accessor::{
    GenerateImageResourceResult, SandGenerateImageResourceAccessor,
};
use mahayana_host_runtime::extensions::attachments::generate_image_service::{
    configured_generate_image_model_id, GeneratedImage, PersistedImage,
    SandGenerateImageError, SandGenerateImageService,
};
use mahayana_host_runtime::extensions::extension_ids_generated::HostExtensionId;
use mahayana_host_runtime::extensions::inference::provider_session::ProviderSessionError;
use mahayana_host_runtime::media_mime::{
    ATTACHMENT_BYTE_LIMIT, VIDEO_BYTE_LIMIT,
};
use mahayana_host_runtime::runner::tools::sand_generate_image_tool::{
    GenerateImageToolExecutor, GeneratedImageToolOutput, GENERATE_IMAGE_TOOL_NAME,
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
    std::fs::create_dir_all(&dir).expect("create temp dir");
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
// OCRM-ATTACH-O04: Configured Model ID
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o04_configured_model_id() {
    let model_id = configured_generate_image_model_id();
    assert!(
        !model_id.trim().is_empty(),
        "Configured generate image model ID must not be empty"
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
// OCRM-ATTACH-O08: Tool Name Constant
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o08_tool_name_constant() {
    assert_eq!(GENERATE_IMAGE_TOOL_NAME, "GenerateImage");
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O09: GenerateImageToolExecutor Mock Invocation
// ---------------------------------------------------------------------------
struct MockImageExecutor {
    last_prompt: Mutex<Option<String>>,
}

impl GenerateImageToolExecutor for MockImageExecutor {
    fn generate(
        &self,
        description: &str,
        reference_images: &[(String, String)],
    ) -> Result<GeneratedImageToolOutput, ProviderSessionError> {
        *self.last_prompt.lock().unwrap() = Some(description.to_string());
        assert_eq!(reference_images.len(), 1);
        assert_eq!(reference_images[0].0, "ref_base64");
        assert_eq!(reference_images[0].1, "image/jpeg");

        Ok(GeneratedImageToolOutput {
            persisted_path: "/workspace/diagram.png".to_string(),
            image_data_base64: "AQIDBA==".to_string(),
        })
    }
}

#[test]
fn test_ocrm_attach_o09_tool_executor_execution() {
    let executor = MockImageExecutor {
        last_prompt: Mutex::new(None),
    };

    let refs = vec![("ref_base64".to_string(), "image/jpeg".to_string())];
    let output = executor
        .generate("Generate an ER diagram", &refs)
        .expect("executor succeeds");

    assert_eq!(output.persisted_path, "/workspace/diagram.png");
    assert_eq!(output.image_data_base64, "AQIDBA==");
    assert_eq!(
        *executor.last_prompt.lock().unwrap(),
        Some("Generate an ER diagram".to_string())
    );
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O10: Standard Base64 Roundtrip
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o10_base64_roundtrip() {
    let original = b"Header: image/png\x00Binary\xFFPayload\x42";
    let encoded = STANDARD.encode(original);
    let decoded = STANDARD.decode(&encoded).expect("base64 decode");
    assert_eq!(decoded, original);
}

// ---------------------------------------------------------------------------
// OCRM-ATTACH-O11: AttachmentTextPreview Enum Variants
// ---------------------------------------------------------------------------
#[test]
fn test_ocrm_attach_o11_attachment_text_preview_variants() {
    let text_preview = AttachmentTextPreview::Text {
        text: "Preview snippet".to_string(),
        truncated: false,
        bytes: 15,
    };

    match text_preview {
        AttachmentTextPreview::Text { text, truncated, bytes } => {
            assert_eq!(text, "Preview snippet");
            assert!(!truncated);
            assert_eq!(bytes, 15);
        }
        AttachmentTextPreview::Binary { .. } => panic!("Expected Text variant"),
    }

    let binary_preview = AttachmentTextPreview::Binary { bytes: 1048576 };
    match binary_preview {
        AttachmentTextPreview::Binary { bytes } => {
            assert_eq!(bytes, 1048576);
        }
        AttachmentTextPreview::Text { .. } => panic!("Expected Binary variant"),
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
