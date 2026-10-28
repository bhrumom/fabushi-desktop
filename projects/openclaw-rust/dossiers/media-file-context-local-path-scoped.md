# OCRM Technical Dossier — Unit 19: Media File Context, Local Path Resolution, Temp Cleanup & Image Processing Specs (`OCRM-UNIT-MEDIA-006`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media File Context Formatting, Local Media Path Resolution, Conversion Temp Cleanup & Image Processing Contracts)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 9 core media utility, file context rendering, path resolution, temp cleanup, truncation notification, provenance, and image processing contract files (totaling 211 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/file-context.ts` | `fd5986503de5dd96a3ca64f9603d080c374e3419` | 34 | 1..34 (full-file) | `OCRM-RESP-MEDIA-FILE-CONTEXT-001` |
| `part-0042` | `src/media/local-media-path.ts` | `f105ca514b5754849284c429b032af30d1487f1a` | 29 | 1..29 (full-file) | `OCRM-RESP-MEDIA-LOCAL-PATH-001` |
| `part-0042` | `src/media/temp-files.ts` | `0bbafd218ede309730b383c1ae45c77a3db3d409` | 21 | 1..21 (full-file) | `OCRM-RESP-MEDIA-TEMP-CLEANUP-001` |
| `part-0042` | `src/media/document-extraction-metadata.ts` | `988d38f46a181c06a477b9538ac21e12fc0df5c0` | 25 | 1..25 (full-file) | `OCRM-RESP-MEDIA-DOC-TRUNCATION-001` |
| `part-0042` | `src/media/runtime-prompt-image-provenance.ts` | `c0b2fc87db982a0da6478a1cad8e57f915bfd81f` | 36 | 1..36 (full-file) | `OCRM-RESP-MEDIA-PROMPT-IMAGE-PROVENANCE-001` |
| `part-0042` | `src/media/image-optimization-error.ts` | `c427946d91b46885bd0da80d5c488d56b7f2599f` | 10 | 1..10 (full-file) | `OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` |
| `part-0042` | `src/media/image-processor-config.ts` | `96475b37a8e9413a1c54b603a3cf642e978d676c` | 27 | 1..27 (full-file) | `OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` |
| `part-0042` | `src/media/image-processor.types.ts` | `5654ebb6e56847d1143030ea9b7cedda2f4d58cf` | 23 | 1..23 (full-file) | `OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` |
| `part-0042` | `src/media/media-services.ts` | `7e9d2a6226fca81f143e0b21bb1a505e84285c73` | 6 | 1..6 (full-file) | `OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-MEDIA-FILE-CONTEXT-001`**:
   - **Behavior**: Renders sanitized attachment text as a model-visible `<file>` block with attribute XML escaping (`&`, `<`, `>`, `"`, `'`) and escaping of injected `</file>` and `<file` markers in content body, supporting compact single-line mode.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_file_context.rs::render_file_context_block`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway("renderFileContext")` → Owner `render_file_context_block` → Consumer (pending upstream prompt assembler integration).
   - **Oracles**: `OCRM-FILE-CTX-O01`, `OCRM-FILE-CTX-O02`, `OCRM-FILE-CTX-O08`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-FILE-CONTEXT-UPSTREAM-CALLER-001"]`: prompt assembly pipeline and OpenResponses HTTP gateway handlers do not yet route attachment text blocks through `render_file_context_block`).

2. **`OCRM-RESP-MEDIA-LOCAL-PATH-001`**:
   - **Behavior**: Resolves raw media reference string to local filesystem `PathBuf` when not a remote URL or data URL, normalizing single-slash and mixed-case file URLs, rejecting UNC hostnames and encoded traversal separators (`%2f`, `%5c`), and expanding tilde home directory paths.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_file_context.rs::resolve_local_media_path`.
   - **Production Call Chain**: Producer `cross_user_sharing/production.rs::CrossUserSharingService::resolve_attachment` → Entrypoint `AttachmentsService::resolve_channel_attachment` → Resolver `channel_attachment.rs::to_local_channel_attachment_path` → Owner `resolve_local_media_path` → Consumer `cross_user_sharing/production.rs`.
   - **Oracles**: `OCRM-FILE-CTX-O03`, `OCRM-FILE-CTX-O08`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

3. **`OCRM-RESP-MEDIA-TEMP-CLEANUP-001`**:
   - **Behavior**: Best-effort temporary file cleanup helper for optional file paths produced during media conversion and staging flows, gracefully ignoring missing files.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_file_context.rs::unlink_if_exists`.
   - **Production Call Chain**: Producer `video_playback_rendition.rs::create_video_playback_source_with` → Owner `unlink_if_exists` → Consumer `video_playback_rendition.rs` (pending managed image attachment and audio transcode staging pipelines).
   - **Oracles**: `OCRM-FILE-CTX-O04`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-TEMP-CLEANUP-UPSTREAM-001"]`: temporary media conversion cleanup helper is wired into video rendition runner but not yet connected to managed outgoing image attachment and audio transcode staging pipelines).

4. **`OCRM-RESP-MEDIA-DOC-TRUNCATION-001`**:
   - **Behavior**: Formats structured notice for partial document text or page extractions, reflecting automatic selection limits, explicit user limits, and text/image truncation flags.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_file_context.rs::render_document_truncation_notice`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway("readAttachmentText")` with optional extraction metadata → Owner `AttachmentsService::read_text_with_document_metadata` → `render_document_truncation_notice` → Consumer gateway text preview. The standalone `renderDocumentTruncation` gateway remains available for callers that only need the notice.
   - **Oracles**: `OCRM-FILE-CTX-O05`, `OCRM-FILE-CTX-O08`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=not-run` (the current exact-target Actions receipt is still required before verification can change; no global flag is upgraded here).

5. **`OCRM-RESP-MEDIA-PROMPT-IMAGE-PROVENANCE-001`**:
   - **Behavior**: Carries prompt image fact ownership indexes across provider-visible message payload structures without altering provider-visible bytes or schema contracts.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_file_context.rs::finalize_runtime_prompt_images, read_runtime_prompt_image_fact_indexes`.
   - **Production Call Chain**: Producer `finalize_runtime_prompt_images` → Consumer (pending upstream prompt assembler integration).
   - **Oracles**: `OCRM-FILE-CTX-O06`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-PROMPT-IMAGE-PROVENANCE-UPSTREAM-001"]`: prompt assembler does not yet attach runtime fact indices to vision provider message payloads).

6. **`OCRM-RESP-MEDIA-IMAGE-PROCESSOR-SPEC-001`**:
   - **Behavior**: Contracts and error types for image processor limits (`MAX_IMAGE_INPUT_PIXELS=25,000,000`), `ImageProcessorPixelLimits`, `ImageOptimizationLimitError` byte budgeting, and image operations (`encode`, `transparency`, `bmpToPng`).
   - **Rust Owner**: `source/host/src/extensions/attachments/media_file_context.rs::ImageProcessorPixelLimits, ImageOptimizationLimitError`.
   - **Production Call Chain**: Producer `ImageProcessorPixelLimits` → Consumer (pending native image processing worker integration).
   - **Oracles**: `OCRM-FILE-CTX-O07`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-IMAGE-PROCESSOR-RASTERMILL-001"]`: native image worker process not yet connected to Rastermill replacement binary).

---

## 3. Verification & Evidence

- **Contract Test Suite**: `source/host/tests/openclaw_media_file_context_contract.rs`
  - 8 passing test cases, including a live `uploadAttachment` → `readAttachmentText` path that prefixes `[Partial document: ...]` before text when extraction metadata is supplied:
    - `test_ocrm_file_ctx_01_xml_attribute_escaping_and_sanitization`: verifies XML attribute character escaping (`&`, `<`, `>`, `"`, `'`) and filename sanitization against quotes, brackets, slashes, and traversal sequences (`.`, `..`).
    - `test_ocrm_file_ctx_02_render_file_context_block_escaping`: verifies exact parity with upstream `file-context.test.ts` (injection markup stripped, `</file>` replaced with `&lt;/file&gt;`, `<file` replaced with `&lt;file`, compact mode supported, fallback filename and mime attributes applied).
    - `test_ocrm_file_ctx_03_resolve_local_media_path`: verifies remote URL rejection, data URL rejection, file URL traversal rejection (`%2f`, `%5c`), UNC host rejection, valid `file:///` path resolution, absolute path resolution, and `~` home directory expansion.
    - `test_ocrm_file_ctx_04_unlink_if_exists`: verifies graceful best-effort deletion of existing and non-existent temp conversion files.
    - `test_ocrm_file_ctx_05_document_truncation_notice`: verifies notice formatting for explicit page limits, automatic page truncation, explicit omitted pages, text truncation, and image rendering truncation.
    - `test_ocrm_file_ctx_06_runtime_prompt_image_provenance`: verifies parallel fact index bundling and safe retrieval without byte mutation.
    - `test_ocrm_file_ctx_07_image_processor_contracts`: verifies 25M pixel limit constants, pixel limit defaults, optimization error string formatting, and operation JSON serialization.
    - `test_ocrm_file_ctx_08_gateway_service_dispatch`: verifies live production gateway dispatch for `renderFileContext`, `resolveLocalMediaPath`, and `renderDocumentTruncation`.
- **Strict Conservation Invariant**:
  - `mapped_paths`: 176
  - `unclassified_paths`: 58,574
  - `tracked_paths`: 58,750 ($176 + 58,574 = 58,750$)
  - `responsibilities`: 114
  - Active blockers: `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001`, `BL-AGENT-ROSTER-UPSTREAM-CALLER-001`, `BL-MEDIA-UNDERSTANDING-UPSTREAM-CALLER-001`, `BL-MEDIA-FFMPEG-EXEC-UPSTREAM-001`, `BL-MEDIA-AUDIO-TRANSCODE-UPSTREAM-001`, `BL-MEDIA-REF-COMPARISON-UPSTREAM-001`, `BL-MEDIA-PROMPT-IMAGE-INPUT-UPSTREAM-001`, `BL-MEDIA-PROMPT-IMAGE-PROVENANCE-UPSTREAM-001`, `BL-MEDIA-IMAGE-PROCESSOR-RASTERMILL-001`, `BL-MEDIA-FILE-CONTEXT-UPSTREAM-CALLER-001`, `BL-MEDIA-TEMP-CLEANUP-UPSTREAM-001`, `BL-MEDIA-DOC-TRUNCATION-UPSTREAM-CALLER-001`.
  - All 8 completion flags remain `false`.
