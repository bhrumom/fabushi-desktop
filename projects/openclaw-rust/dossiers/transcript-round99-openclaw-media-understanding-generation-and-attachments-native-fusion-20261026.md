# OCRM Technical Dossier: Round 99 — OpenClaw Media Understanding, Generation & Attachments Native Fusion

## 1. Executive Summary

This dossier records the full-source native Rust equivalence and fusion for:
1. **`OCRM-M23`**: Attachments, Media Understanding, Byte Quotas & Resource Storage (`source/host/src/extensions/attachments/`, `source/host/src/attachment_paths.rs`, `source/host/src/media_mime.rs`)
2. **`OCRM-M24`**: Image Generation, Generation Providers, Box Accessors & Tool Routing (`source/host/src/runner/tools/sand_generate_image_tool.rs`, `source/host/src/runner/host_generate_image_dependencies.rs`)

All upstream responsibilities from `openclaw/openclaw` across `src/media`, `src/media-understanding`, `src/image-generation`, and `src/media-generation` are realized natively in canonical Fabushi Host Rust architecture without external node runtime or unmonitored daemon loops.

---

## 2. Upstream Source to Canonical Fabushi Rust Mapping

### 2.1 Attachments & Media Understanding (`OCRM-M23`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/media/limits.ts` | 25 MiB attachment cap & 200 MiB video limit | `ATTACHMENT_BYTE_LIMIT`, `VIDEO_BYTE_LIMIT` | Host Media MIME |
| `src/media/paths.ts` | Attachments & assets directory isolation | `ATTACHMENTS_DIRNAME`, `ASSETS_DIRNAME` | Host Attachment Paths |
| `src/media/previews.ts` | Text preview and binary metadata inspection | `AttachmentTextPreview` (`Text`, `Binary`) | Host Attachments Service |
| `src/media/extensions.ts` | Extension dependency registration & service start | `ATTACHMENTS_EXTENSION_ID`, `ATTACHMENTS_DEPENDENCIES` | Host Extensions Engine |

### 2.2 Media Generation & Tool Execution (`OCRM-M24`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/image-generation/accessor.ts` | Media storage strict containment & path validation | `SandGenerateImageResourceAccessor` | Host Attachments Extension |
| `src/image-generation/config.ts` | Configured generation model identifier resolution | `configured_generate_image_model_id` | Host Attachments Extension |
| `src/image-generation/service.ts` | Image generation service & transactional persistence | `SandGenerateImageService`, `SandGenerateImageError` | Host Attachments Extension |
| `src/image-generation/tool.ts` | Routed tool definition and executor trait dispatch | `GENERATE_IMAGE_TOOL_NAME`, `GenerateImageToolExecutor` | Runner Tools |

---

## 3. Falsifiable Test Oracles

1. **`OCRM-ATTACH-O01` (Byte Limits)**: Validates `ATTACHMENT_BYTE_LIMIT == 26_214_400` (25 MiB) and `VIDEO_BYTE_LIMIT == 209_715_200` (200 MiB).
2. **`OCRM-ATTACH-O02` (Directory Names)**: Validates `ATTACHMENTS_DIRNAME == "attachments"` and `ASSETS_DIRNAME == "assets"`.
3. **`OCRM-ATTACH-O03` (Resource Containment)**: Validates `SandGenerateImageResourceAccessor` rejection of paths escaping the agent's media store.
4. **`OCRM-ATTACH-O04` (Model Identifier)**: Validates `configured_generate_image_model_id` returns a valid, non-empty model identifier.
5. **`OCRM-ATTACH-O05` (Successful Generation)**: Validates end-to-end `SandGenerateImageService` generation and transactional file persistence.
6. **`OCRM-ATTACH-O06` (Persistence Failure)**: Validates fail-closed handling returning `SandGenerateImageError::Persist` on persistence rejection.
7. **`OCRM-ATTACH-O07` (Invalid Base64)**: Validates fail-closed handling returning `SandGenerateImageError::InvalidBase64` on malformed payload.
8. **`OCRM-ATTACH-O08` (Tool Registry Name)**: Validates `GENERATE_IMAGE_TOOL_NAME == "GenerateImage"`.
9. **`OCRM-ATTACH-O09` (Executor Dispatch)**: Validates `GenerateImageToolExecutor` execution with prompt descriptions and reference images.
10. **`OCRM-ATTACH-O10` (Base64 Roundtrip)**: Validates standard base64 encoding and decoding integrity for binary media payloads.
11. **`OCRM-ATTACH-O11` (Preview Variants)**: Validates `AttachmentTextPreview::Text` and `AttachmentTextPreview::Binary` enum representations.
12. **`OCRM-ATTACH-O12` (Extension Dependencies)**: Validates extension dependencies on Auth, ForeverBox, and Telemetry.
