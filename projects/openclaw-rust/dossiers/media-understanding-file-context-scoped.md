# OCRM Dossier: Media Understanding File Context & Extraction Limits

- **Unit ID**: `OCRM-UNIT-033`
- **Module ID**: `OCRM-M24` (`media-understanding`)
- **Responsibility IDs**:
  - `OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001`
  - `OCRM-RESP-MEDIA-UND-FILE-CONTEXT-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 8 upstream files in `src/media-understanding/` (913 lines total) are audited and mapped:
1. `src/media-understanding/file-extraction-limits.ts` (45 lines) -> `OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001`
2. `src/media-understanding/file-extraction-limits.test.ts` (44 lines) -> `OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001`
3. `src/media-understanding/extracted-file-images.ts` (14 lines) -> `OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001`
4. `src/media-understanding/media-attachment-outcomes.ts` (49 lines) -> `OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001`
5. `src/media-understanding/file-attachment-outcomes.ts` (180 lines) -> `OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001`
6. `src/media-understanding/file-attachment-outcomes.test.ts` (93 lines) -> `OCRM-RESP-MEDIA-UND-FILE-OUTCOMES-001`
7. `src/media-understanding/file-context.ts` (333 lines) -> `OCRM-RESP-MEDIA-UND-FILE-CONTEXT-001`
8. `src/media-understanding/file-context.test.ts` (155 lines) -> `OCRM-RESP-MEDIA-UND-FILE-CONTEXT-001`

## 2. Invariants & Implementation Details

- **Extraction Limits Clamping**:
  - Clamps byte limit to `INBOUND_FILE_EXTRACTION_MAX_BYTES_CAP` (25MB safety ceiling) when operator `mediaMaxMb` is configured high (e.g. 500MB).
  - Clamps page limit to `INBOUND_FILE_EXTRACTION_MAX_PAGES_CAP` (150 pages) when `pdfMaxPages` exceeds bound.
  - Applies default 20MB and 20 pages when limits are omitted.
  - Honors explicit operator `responses.files` configuration per-field.
- **MIME & Path Sanitization**:
  - Validates MIME against RFC 9110 token specification, stripping parameter tails. Discards invalid MIMEs (such as prompt injections) to prevent model-visible poisoning.
  - Validates local file paths against strict unicode character sets (`MARKER_PATH_SAFE`), forbidding path traversal (`..`), RTL override characters (`\u202e`), shell punctuation, or prompt injections.
  - Absolute POSIX (`/`) and Windows drive (`C:\`) paths are accepted; relative workspace paths are permitted only when explicitly enabled.
- **Untrusted Content Fencing**:
  - Untrusted attachment text and approved local file paths are framed within boundary markers (`<<<EXTERNAL_UNTRUSTED_CONTENT id="...">>>`) using random 16-hex IDs to prevent boundary spoofing.
  - Nested spoof markers and special LLM tokens are sanitized prior to wrapping.
- **Attachment Marker Budget**:
  - Skipped file outcomes (unsupported format, policy rejected, read failure, URL disabled) consume a marker budget capped at 5. Subsequent skipped attachments emit `[N more attachments skipped]` overflow summary.

## 3. Rust Native Equivalence

Implemented in `source/host/src/extensions/attachments/media_understanding_service.rs`:
- `FileExtractionLimitsConfig`, `FileExtractionLimits`, `resolve_file_extraction_limits`
- `DocumentExtractedImage`, `ExtractedFileImage`, `strip_extracted_file_image_metadata`
- `MAX_SKIPPED_FILE_MARKERS`, `render_skipped_file_overflow_summary`, `render_media_attachment_disposition`, `AttachmentContextBlock`, `apply_attachment_marker_budget`
- `sanitize_mime_type`, `marker_safe_mime`, `marker_safe_local_path`, `sanitize_external_content_text`, `wrap_untrusted_attachment_content`
- `FileAttachmentOutcome`, `resolve_file_extraction_outcome`, `is_skipped_file_outcome`, `RenderFileAttachmentOutcomeOptions`, `render_file_attachment_outcome`
- `InboundDocumentContext`, `InboundDocumentItem`, `render_inbound_document_context_from_items`

## 4. Verification Evidence

- Contract test suite: `source/host/tests/openclaw_media_understanding_contract.rs`
  - `test_ocrm_media_und_o15_file_extraction_limits_clamping_and_defaults`: verifies 25MB byte cap clamping, 150-page clamping, defaults, and explicit config precedence.
  - `test_ocrm_media_und_o16_mime_and_path_security_sanitization`: verifies valid MIME stripping, malformed MIME rejection, path traversal prevention, and safe character set filtering.
  - `test_ocrm_media_und_o17_file_attachment_outcomes_and_fencing`: verifies extracted text wrapping, truncation notices, local path metadata fencing, Office XML zip hints, and policy rejections.
  - `test_ocrm_media_und_o18_inbound_document_context_and_marker_budget`: verifies disposition rendering, 5-marker budget overflow, inbound `<file>` block XML generation, and extracted image metadata stripping.
- All 18 tests in `openclaw_media_understanding_contract.rs` passed with 0 failures.
