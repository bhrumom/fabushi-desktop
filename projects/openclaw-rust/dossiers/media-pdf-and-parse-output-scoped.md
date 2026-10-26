# OCRM Technical Dossier — Unit 28: PDF Document Content Extraction, Metadata Validation & Media Output Directive Parsing (`OCRM-UNIT-MEDIA-015`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (PDF Content Extraction via Plugin Extractors, Bounded Metadata Validation, Tool/Stdout MEDIA Directive Parsing, Markdown Image Separation & Audio Tags)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 21 media extraction, parsing, and media test suite files (totaling 4,261 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/pdf-extract.ts` | `3066ee4156d91c77f0f009a397152f44a4801932` | 104 | 1..104 (full-file) | `OCRM-RESP-MEDIA-PDF-EXTRACT-001` |
| `part-0042` | `src/media/pdf-extract.test.ts` | `1a3b7bbd3613f1f4f3ddf81f22771fe13459dd0b` | 219 | 1..219 (full-file) | `OCRM-RESP-MEDIA-PDF-EXTRACT-001` |
| `part-0042` | `src/media/parse.ts` | `b7a0a1a7342ca851e1ec253b94d8496d8350eeb7` | 30 | 1..30 (full-file) | `OCRM-RESP-MEDIA-PARSE-OUTPUT-001` |
| `part-0042` | `src/media/parse-output.ts` | `edbef5245a218e39ca44985d4035a69f9c3997af` | 708 | 1..708 (full-file) | `OCRM-RESP-MEDIA-PARSE-OUTPUT-001` |
| `part-0042` | `src/media/parse.test.ts` | `b931d5bff0ad44cbc7cade781d0e969e84e8832e` | 1156 | 1..1156 (full-file) | `OCRM-RESP-MEDIA-PARSE-OUTPUT-001` |
| `part-0042` | `src/media/configured-max-bytes.test.ts` | `c24aad840a1ac2453e8cd74c60a9dcf04d6879fe` | 88 | 1..88 (full-file) | `OCRM-RESP-MEDIA-BYTE-CAPS-001` |
| `part-0042` | `src/media/audio.test.ts` | `35e0280917bf488a19e7eee68b2726381b204e77` | 67 | 1..67 (full-file) | `OCRM-RESP-MEDIA-VOICE-AUDIO-COMPAT-001` |
| `part-0042` | `src/media/audio-transcode.test.ts` | `7a8536f731a3c7582fa3c63bf22141b406f4e9f8` | 266 | 1..266 (full-file) | `OCRM-RESP-MEDIA-AUDIO-TRANSCODE-001` |
| `part-0042` | `src/media/ffmpeg-exec.test.ts` | `db7ba0dd81186c91f7580a0fd56bfdf69121a224` | 94 | 1..94 (full-file) | `OCRM-RESP-MEDIA-FFMPEG-EXEC-001` |
| `part-0042` | `src/media/file-context.test.ts` | `0ebddc609a8e0abc5e1b3b95c58254e17ab5acae` | 45 | 1..45 (full-file) | `OCRM-RESP-MEDIA-FILE-CONTEXT-001` |
| `part-0042` | `src/media/local-media-path.windows.test.ts` | `1aef7d957f8e243203ccec8dd92742ca3e795761` | 98 | 1..98 (full-file) | `OCRM-RESP-MEDIA-LOCAL-PATH-001` |
| `part-0042` | `src/media/media-facts.persisted.test.ts` | `cccbd2e2309586a751b1ac99c3db3e3252c0b6f2` | 390 | 1..390 (full-file) | `OCRM-RESP-MEDIA-FACTS-001` |
| `part-0042` | `src/media/media-probe.test.ts` | `98313b886a4caf2cbfd12d30e3e07437c2f03e83` | 384 | 1..384 (full-file) | `OCRM-RESP-MEDIA-PROBE-METADATA-001` |
| `part-0042` | `src/media/media-reference-comparison.test.ts` | `de547cf68f2c3fb09901cc00d5609ef9b832bf4f` | 75 | 1..75 (full-file) | `OCRM-RESP-MEDIA-REF-COMPARISON-001` |
| `part-0042` | `src/media/media-reference.test.ts` | `6de3eeb45abbb6f731c860904e2825425828f657` | 154 | 1..154 (full-file) | `OCRM-RESP-MEDIA-REFERENCE-RESOLUTION-001` |
| `part-0042` | `src/media/media-retirement-non-goals.test.ts` | `504335d18dc4d8a7fe38b1c9eef83e6835d1e609` | 18 | 1..18 (full-file) | `OCRM-RESP-MEDIA-OUTBOUND-ATTACHMENT-001` |
| `part-0042` | `src/media/native-process-runtime.test-support.ts` | `1a0200a41bae96e6550c35e6cbeb4e3cd551a338` | 12 | 1..12 (full-file) | `OCRM-RESP-MEDIA-STORE-TEST-SUPPORT-001` |
| `part-0042` | `src/media/outbound-attachment.test.ts` | `34132ef7fce77af32d741aee44200bbc4cc1e174` | 104 | 1..104 (full-file) | `OCRM-RESP-MEDIA-OUTBOUND-ATTACHMENT-001` |
| `part-0042` | `src/media/qr-image.test.ts` | `af1438b732f077577bca5cf815263bcf45153c48` | 112 | 1..112 (full-file) | `OCRM-RESP-MEDIA-QR-IMAGE-001` |
| `part-0042` | `src/media/qr-terminal.render.test.ts` | `2e539a23deac9a727976e0822df004b58d17d951` | 91 | 1..91 (full-file) | `OCRM-RESP-MEDIA-QR-TERMINAL-001` |
| `part-0042` | `src/media/sniff-mime-from-base64.test.ts` | `816dbc94f43bcc06bde3467cb71a0329e24cf572` | 46 | 1..46 (full-file) | `OCRM-RESP-MEDIA-SNIFF-PREVIEW-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

1. **`OCRM-RESP-MEDIA-PDF-EXTRACT-001` (NEW)**:
   - **Behavior**: Native PDF document content extraction via plugin extractors, request composition (buffer, maxPages, maxPixels, minTextChars, password, pageNumbers), and trusted metadata validation. Enforces strict metadata invariants: automatic vs explicit page selection consistency, `maxPages` clamping, positive non-zero page numbers, uniqueness, page <= total bounds, and truncation consistency (`pages.truncated == has_omitted_pages || (has_omitted_pages && text_truncated)`). Untrusted metadata is safely stripped before rendering prompt text. Dispatches clear error when PDF extraction is disabled or plugins fail.
   - **Rust Owner**: `source/host/src/extensions/attachments/pdf_extract.rs`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `pdf_extract` → Consumer: *Inbound document extractor plugins and turn prompt context composition*.
   - **Oracle**: `OCRM-PDF-EXTRACT-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-PDF-EXTRACT-CHAT-WIRING-001"]`.

2. **`OCRM-RESP-MEDIA-PARSE-OUTPUT-001` (NEW)**:
   - **Behavior**: Text output splitting and media directive parsing. Scans tool and stdout prose for `MEDIA:` directives, unquotes payloads, parses whitespace-split or bracketed comma-separated media references, validates local paths vs remote URLs, enforces SSRF hostname guards (rejecting loopback, private, link-local, and `.internal` addresses), guards against directory traversal (`..`, `~`), isolates code fences (preserving `MEDIA:` inside markdown code blocks), extracts markdown images `![alt](url)` with optional allowlists, strips inline `[[audio_as_voice]]` tags, and projects ordered text/media segments.
   - **Rust Owner**: `source/host/src/extensions/attachments/parse_output.rs`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `parse_output` → Consumer: *Runner tool execution stdout processor and chat timeline projection*.
   - **Oracle**: `OCRM-PARSE-OUTPUT-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-PARSE-OUTPUT-CHAT-WIRING-001"]`.

3. **`OCRM-RESP-MEDIA-ATTACHMENT-PROCESSOR-001` (CALIBRATED & WIRED)**:
   - **Behavior**: Attachment payload size verification and validation.
   - **Rust Owner**: `source/host/src/attachment_paths.rs::validate_and_prepare_attachment_payload`.
   - **Production Consumer**: Actively wired into `source/host/src/extensions/transcript/inline_image_materialization.rs::materialize_one` guarding every inline image before filesystem materialization.
   - **Status**: Calibrated to `implementation=implemented`, `wiring=wired`, `verification=verified`, `blocker_ids=[]` (eliminating `BL-MEDIA-ATTACHMENT-PROCESSOR-CONSUMERS-001`).

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Mapped paths**: **256** (increased by 21 from 235).
- **Unclassified paths**: **58,494** (decreased by 21 from 58,515).
- **Omitted paths known**: **0** (strictly 0).
- **Sum invariant**: `mapped_paths (256) + unclassified_paths (58,494) == tracked_paths (58,750)` (100% exact parity).
- **Total active responsibilities**: **137** (135 existing + 2 newly registered `OCRM-RESP-MEDIA-PDF-EXTRACT-001` and `OCRM-RESP-MEDIA-PARSE-OUTPUT-001`).
- **Global completion flags**: All 8 flags remain strictly `false`.

---

## 4. Verification Evidence & Contract Test Suite

A dedicated contract test suite was authored and verified passing:
- **Test File**: `source/host/tests/openclaw_pdf_and_parse_contract.rs`
- **Results**: 8/8 tests passing:
  1. `test_pdf_extract_disabled_when_no_extractor`: Verifies clear disablement error when no document extractors are available.
  2. `test_pdf_extract_metadata_validation`: Verifies automatic vs explicit selection, max_pages bounds, unique positive page IDs, and truncation consistency.
  3. `test_pdf_extract_mock_extractor_success_and_error`: Verifies end-to-end execution with mock plugin extractor and error aggregation.
  4. `test_parse_output_media_directives`: Verifies single and bracketed `MEDIA:` lines parsed into media URLs with clean prose projection.
  5. `test_parse_output_traversal_and_ssrf_guards`: Verifies rejection of traversal (`..`, `~user`), insecure HTTP, and private/internal IP hostnames.
  6. `test_parse_output_audio_directive_tag`: Verifies `[[audio_as_voice]]` stripping and `audio_as_voice: true` flag.
  7. `test_parse_output_code_block_isolation`: Verifies `MEDIA:` inside markdown code fences is strictly ignored as media directive.
  8. `test_parse_output_markdown_images`: Verifies markdown image extraction with allowlists and segment ordering.
  9. `test_gateway_dispatch_pdf_and_parse_output`: Verifies gateway RPC calls for `extractPdfContent`, `validatePdfExtractionMetadata`, `splitMediaOutput`, and `splitMediaFromOutput`.
