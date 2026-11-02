# OCRM Technical Dossier — Unit 27: Media Input Files & Staged Inputs (`OCRM-UNIT-MEDIA-014`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Multi-Modal Input File & Image Extraction, Charset Normalization, Staged Task Inputs & Sandbox Directory Scoping)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The locked-source pass re-read `src/media/input-files.ts` in full (blob
`1b8f2241885a1819e930b5b4a1daec91188ab947`, 409 lines) and
`src/media/staged-inputs.ts` in full (blob
`98e3e0cc8d2fec93d0712cef7901961082d36ab0`, 175 lines). Input extraction
normalizes MIME allowlists, bounds base64/URL reads, applies SSRF/redirect/
timeout limits, truncates text with explicit metadata, validates images and
converts HEIC/HEIF; staged inputs require an exact `.gitignore` ownership
marker, sanitized names, digest-bound manifests, ambiguity rejection and
abort-aware creation. These rows map to their existing Rust owners without
upgrading production verification.

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
   - **Production Call Chain**: `AttachmentsService::dispatch_gateway("extractFileContent"|"extractImageContent")` → Owner `input_files`; `input_files::read_input_source` also calls the live `media_fetch` owner for URL sources.
   - **Wiring boundary**: Gateway extraction and URL-source paths are live; complete user-turn multimodal prompt composition is still pending.
   - **Oracle**: `OCRM-INPUT-FILES-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-INPUT-FILES-CHAT-WIRING-001"]`.

2. **`OCRM-RESP-MEDIA-STAGED-INPUTS-001` (NEW)**:
   - **Behavior**: Inbound task input staging directory sandboxing, exact `.gitignore` marker digest validation, alias disambiguation, and media fact path mapping. Enforces OpenClaw-owned staging directories with 0o600 permissions and SHA-256 verified `.gitignore` markers (`c2ae48852cf5745c7dcbcdfa46b0141497cb37fda5b011164f75dd3c08358e6f`), extracts stem and extension aliases for turn inputs, and removes ambiguous duplicates to prevent unverified selection.
   - **Rust Owner**: `source/host/src/extensions/attachments/staged_inputs.rs`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `staged_inputs` → Consumer: *Inbound task agent execution sandbox and staged input injectors*.
   - **Oracle**: `OCRM-STAGED-INPUTS-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-STAGED-INPUTS-CHAT-WIRING-001"]`.

### 2.1 Current exact-head production-path audit (2026-11-01)

The current direct-send path is now recorded precisely for wiring decisions:

- `UnifiedGatewayApi::call_send_prompt` first calls
  `shape_send_prompt_media_args` (`source/host/app/src/main.rs:2123-2139`),
  which classifies attachment paths and prepares selected image/video metadata
  (`source/host/src/extensions/transcript/send_message_shaping.rs:185-213`).
- The accepted send is persisted, then the shaped arguments are passed through
  `prepare_direct_turn_runner_args` and `call_host_lane` (`source/host/app/src/main.rs:2200-2207`).
- The routed turn stages host attachment paths into the agent box
  (`source/host/app/src/main.rs:6512-6529`) and the provider prompt projection
  appends a file-location/read guidance block
  (`source/host/src/runner/prompt_collector_glue.rs:684-712`).
- The production path does **not** call
  `input_files::extract_file_content_from_*` or
  `extract_image_content_from_source`; those remain gateway RPC branches only
  (`source/host/src/extensions/attachments/attachments_service.rs:2085-2128`).

Therefore the existing `BL-MEDIA-INPUT-FILES-CHAT-WIRING-001` and
`BL-MEDIA-STAGED-INPUTS-CHAT-WIRING-001` blockers remain valid. Staging and
prompt guidance are production behavior, but they do not establish extraction,
content injection, or complete multimodal user-turn composition.

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

A dedicated contract test suite was authored; the result below is historical
evidence, not current verification. The suite was not executed in the current
development freeze; exact-head Actions, packaged/runtime, and live input
pipeline acceptance evidence remain pending:
- **Test File**: `source/host/tests/openclaw_input_files_contract.rs`
- **Results**: 6/6 tests passing:
  1. `test_staged_input_directory_and_ownership`: Verifies exact candidate regex (64 lower hex or UUID v4), `.gitignore` marker file SHA-256 validation (78 bytes, `c2ae48852cf5745c7dcbcdfa46b0141497cb37fda5b011164f75dd3c08358e6f`), 0o600 file modes, and rejection of tampered directories.
  2. `test_resolve_staged_input_media_paths`: Verifies filename extraction with `.ext` and stem aliases, ambiguous duplicate handle stripping, and non-staged filtering.
  3. `test_named_attachment_decoding_and_charsets`: Verifies text extraction under UTF-8, Latin-1, Windows-1251, Windows-1252 (`café €`), unclosed quote recovery, and UTF-16LE BOM overriding declared charsets.
  4. `test_input_file_limits_and_truncation`: Verifies `maxChars` clamping with surrogate pair safety and `textTruncated` metadata tagging.
  5. `test_input_source_and_image_validation`: Verifies `allowUrl=false` rejection, payload size limit enforcement, AVI video byte classification (`video/x-msvideo`) and rejection from image allowlist, and HEIC image normalization to JPEG.
  6. `test_gateway_dispatch_input_files_and_staged_inputs`: Verifies dynamic gateway RPC methods `resolveInputFileLimits`, `extractFileContentFromBuffer`, `stagedInputPathDirectory`, `isStagedInputPath`, and `ensureStagedInputDirectory`.

## 5. Fixed-U policy admission correction — MIP-v1

This entry uses current U `1440bb1771372334bea23d7642bb52deb7db1845`
(tree `90c8aab39972da11cfad220361781d49e1bc6700`) and target baseline
`3c4bf6963cb30ecf01eb34b905315074886e1972`. Sections 1–4 retain their
historical source/read/count facts; their old identities are not current-U
full-read evidence. The bounded contract is
[`media-input-policy-admission.md`](../docs/specs/media-input-policy-admission.md).

| Current-U input | Exact blob | Review applied in this batch |
| --- | --- | --- |
| `src/media/input-files.ts` | `64bca417aa7b46c02d11d81f1b8a5b6f906137cd` | Implementation/call-chain review of typed limits, resolver, shared source reads and extraction consumers; independent reader also read the full file. |
| `packages/media-core/src/base64.ts` | `31eead019c4aeecd081ba7836e7ec408a367ae1c` | Full helper read; size arithmetic, canonical validation, allocation and error ordering. |
| `packages/media-core/package.json` | `88490cbd64f5cddd4181f053bb029da78d272885` | Package export contract review. |
| `packages/media-core/src/base64.test.ts` | `337c775bd1723f7bfba87e3d0275a81798857fe9` | Independent full test-file read; invalid dialect and estimator controls. |
| `src/media/input-files.extract.test.ts` | `02cd69d091e93a585a31703a9872e670488d65f1` | Bounded allowlist and truncation oracle review. |
| `src/media/input-files.fetch-guard.test.ts` | `645719e0cc5822b85f4431f1ef55899e2aafb0c2` | Bounded size/validation controls, including base64 lines 612–675. |

The real entrypoints are `AttachmentsService::dispatch_gateway` methods
`extractFileContentFromBuffer`, `extractFileContentFromSource`, and
`extractImageContentFromSource`, reached by the existing generic Host-lane
dispatcher. The earlier `extractFileContent`/`extractImageContent` wording is
shorthand, not a separate implemented method. File RPC admission now shares
one Rust helper: omitted/null policy retains default behavior; explicit
policy must be a complete object or return an error before decoding/source
processing. Valid false/zero/empty restrictions pass unchanged. The existing
file/image decoder clamps the `=`/`==` underflow before normal base64
rejection while preserving size-before-validation for longer malformed data.
Existing serde/base64 dependencies are reused; no new owner or
dependency is introduced.

The existing Host/App manifests select `base64 = "0.22"` with default features;
both locks bind the runtime to registry `base64 0.22.1`, checksum
`72b3254f16251a8381aa12e40e3c4d2f0199f8c6508fbecb9d91f575e0fbb8c6`.
The inspected manifests contain no base64 patch/path/git replacement. Its
official manifest at `marshallpierce/rust-base64@e14400697453bcc85997119b874bc03d9601d0af`
(blob `ea2d4050388e30d6d49e24ef09fee6460fa50997`) declares `MIT OR Apache-2.0`,
default `std`/`alloc`, and no runtime dependencies. The actual consumer keeps
the existing standard decoder and error result; this change neither replaces
the decoder nor establishes broader dialect parity. This is manifest/usage
review, not a claimed download or complete archive/legal-source closure.

Actual-dispatch controls extend `openclaw_input_files_contract` and its full
target is selected in the existing scoped Actions workflow, with source
hashes, command log, exit status and subject receipt. Source-only review is
not execution evidence. At authoring, exact-H Actions for this correction
remain pending; responsibility verification/acceptance and all eight global
flags are not promoted.

Open scope remains concrete: no accepted-turn extractor consumer; placeholder
PDF text and placeholder HEIC image conversion; incomplete URL
hostname-allowlist/redirect/cancellation policy; and broader base64 dialect
differences. Existing staged-input tests and dummy HEIC success are not real
format or live-channel compatibility acceptance. These observations retain
the existing chat-wiring blocker and do not reduce the full media obligation.
