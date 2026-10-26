# OCRM Technical Dossier — Unit 27: Media Input Files & Staged Inputs (`OCRM-UNIT-MEDIA-014`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Multi-Modal Input File & Image Extraction, Charset Normalization, Staged Task Inputs & Sandbox Directory Scoping)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 5 core input file and staged input files (totaling 1,666 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/input-files.ts` | `c8a8167f53f9547d2f34936d0a7a0b368731ad8c` | 409 | 1..409 (full-file) | `OCRM-RESP-MEDIA-INPUT-FILES-EXTRACT-001` |
| `part-0042` | `src/media/input-files.extract.test.ts` | `8ba1804f8664ecffbb724e5ef6dc972b2c9ad508` | 329 | 1..329 (full-file) | `OCRM-RESP-MEDIA-INPUT-FILES-EXTRACT-001` |
| `part-0042` | `src/media/input-files.fetch-guard.test.ts` | `c41f92e59e19864aa9dbeff64a4ad9cbaebfe411` | 700 | 1..700 (full-file) | `OCRM-RESP-MEDIA-INPUT-FILES-EXTRACT-001` |
| `part-0042` | `src/media/staged-inputs.ts` | `6f92023d537f525bf7845f061ca91307bdfa5e55` | 175 | 1..175 (full-file) | `OCRM-RESP-MEDIA-STAGED-INPUTS-001` |
| `part-0042` | `src/media/staged-inputs.test-support.ts` | `815aa025a1f6a1e360ca690623d24ea2be81a8c0` | 53 | 1..53 (full-file) | `OCRM-RESP-MEDIA-STAGED-INPUTS-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

The audited sources were mapped to 2 discrete Rust runtime responsibilities registered in `parity-ledger.json`. In strict compliance with the project's honest calibration policy, because end-to-end user turn input ingestion and task staging workflows are scheduled for subsequent integration stages, these responsibilities are conservatively marked `wiring=not-wired`, `verification=not-run`, `acceptance=pending` with registered blocker IDs:

1. **`OCRM-RESP-MEDIA-INPUT-FILES-EXTRACT-001` (NEW)**:
   - **Behavior**: Input file and image extraction, MIME classification, WHATWG charset normalization, surrogate-safe UTF-16 truncation, and SSRF guard pipeline. Validates buffer size and image dimensions, detects and verifies container formats, decodes multi-byte charsets with streaming chunk state preservation, truncates safely at UTF-16 surrogate boundaries, converts HEIC/HEIF images to JPEG, and enforces network SSRF policies on remote input sources.
   - **Rust Owner**: `source/host/src/extensions/attachments/input_files.rs`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `input_files` → Consumer: *Pending user turn input processing and multi-modal prompt formatting*.
   - **Oracle**: `OCRM-INPUT-FILES-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-INPUT-FILES-CHAT-WIRING-001"]`.

2. **`OCRM-RESP-MEDIA-STAGED-INPUTS-001` (NEW)**:
   - **Behavior**: Inbound task input staging directory sandboxing, exact `.gitignore` marker digest validation, alias disambiguation, and media fact path mapping. Enforces OpenClaw-owned staging directories with 0o600 permissions and SHA-256 verified `.gitignore` markers (`c2ae48852cf5745c7dcbcdfa46b0141497cb37fda5b011164f75dd3c08358e6f`), extracts stem and extension aliases for turn inputs, and removes ambiguous duplicates to prevent unverified selection.
   - **Rust Owner**: `source/host/src/extensions/attachments/staged_inputs.rs`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `staged_inputs` → Consumer: *Inbound task agent execution sandbox and staged input injectors*.
   - **Oracle**: `OCRM-STAGED-INPUTS-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-STAGED-INPUTS-CHAT-WIRING-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Mapped paths**: **235** (increased by 5 from 230).
- **Unclassified paths**: **58,515** (decreased by 5 from 58,520).
- **Omitted paths known**: **0** (strictly 0).
- **Sum invariant**: `mapped_paths (235) + unclassified_paths (58515) == tracked_paths (58750)`.
- **Total active responsibilities**: **135** (increased by 2 from 133).
- **Global completion flags**: All 8 flags remain strictly `false`.

---

## 4. Verification Evidence & Contract Test Suite

A dedicated contract test suite was authored and verified passing:
- **Test File**: `source/host/tests/openclaw_input_files_contract.rs`
- **Results**: 6/6 tests passing:
  1. `test_staged_input_directory_and_ownership`: Verifies exact candidate regex (64 lower hex or UUID v4), `.gitignore` marker file SHA-256 validation (78 bytes, `c2ae48852cf5745c7dcbcdfa46b0141497cb37fda5b011164f75dd3c08358e6f`), 0o600 file modes, and rejection of tampered directories.
  2. `test_resolve_staged_input_media_paths`: Verifies filename extraction with `.ext` and stem aliases, ambiguous duplicate handle stripping, and non-staged filtering.
  3. `test_named_attachment_decoding_and_charsets`: Verifies text extraction under UTF-8, Latin-1, Windows-1251, Windows-1252 (`café €`), unclosed quote recovery, and UTF-16LE BOM overriding declared charsets.
  4. `test_input_file_limits_and_truncation`: Verifies `maxChars` clamping with surrogate pair safety and `textTruncated` metadata tagging.
  5. `test_input_source_and_image_validation`: Verifies `allowUrl=false` rejection, payload size limit enforcement, AVI video byte classification (`video/x-msvideo`) and rejection from image allowlist, and HEIC image normalization to JPEG.
  6. `test_gateway_dispatch_input_files_and_staged_inputs`: Verifies dynamic gateway RPC methods `resolveInputFileLimits`, `extractFileContentFromBuffer`, `stagedInputPathDirectory`, `isStagedInputPath`, and `ensureStagedInputDirectory`.
