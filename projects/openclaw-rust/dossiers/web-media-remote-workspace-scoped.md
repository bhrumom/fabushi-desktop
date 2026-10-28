# OCRM Technical Dossier — Unit 23: Web Media Pipeline, Remote Workspace File Transfer & Anthropic Inline Content Normalization (`OCRM-UNIT-MEDIA-010`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Web Media Pipeline, SSRF / Windows Network UNC Guards, Animated WebP Preservation, Bounded Remote Workspace File Chunking & Anthropic Inline Blocks)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The locked-source pass re-read `src/media/remote-workspace-file.ts` in full
(blob `991814e47ba2c283c814e097726aa8d8251ad071`, 138 lines). Its bounded
reader pins each chunk to an opened regular file, rejects symlinks and
workspace escapes, enforces output/byte caps, propagates cancellation and
timeouts, and rejects size/revision changes during transfer. The canonical row
maps to the existing Rust remote-workspace owner without claiming live
consumer acceptance.

The locked-source pass re-read `src/media/anthropic-inline-images.ts` in full
(blob `84049330c09c297ed5913a9f33905b7162a83a5d`, 96 lines). Its concrete
behavior is a 10 MiB decoded inbound-image guard, canonical base64 handling,
prefix MIME sniffing, supported MIME preservation, and BMP/unsupported-image
conversion through image-ops before emitting normalized Anthropic blocks. The
canonical row maps this source to the existing Rust owner candidate without
upgrading production verification.

The following 8 core web media loading, SSRF protection, remote workspace file streaming, and Anthropic inline content block files (totaling 2,168 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/anthropic-inline-images.ts` | `f1cf59a9e33ea2cf486b72dbead9b92ae17ce9f3` | 96 | 1..96 (full-file) | `OCRM-RESP-MEDIA-ANTHROPIC-INLINE-001` |
| `part-0042` | `src/media/anthropic-inline-images.test.ts` | `d0498b8cba8c6a0b73c4fca649a43a0889db4c80` | 120 | 1..120 (full-file) | `OCRM-RESP-MEDIA-ANTHROPIC-INLINE-001` |
| `part-0042` | `src/media/web-media.ts` | `52baea5fca06e309cb9f3ef3c7bc379fc70c17be` | 813 | 1..813 (full-file) | `OCRM-RESP-MEDIA-WEB-PIPELINE-001` |
| `part-0042` | `src/media/web-media.test.ts` | `f12cfc7fa689252c8f85f3ba1d86d5cbe9d96859` | 710 | 1..710 (full-file) | `OCRM-RESP-MEDIA-WEB-PIPELINE-001` |
| `part-0042` | `src/media/web-media.animated-webp.test.ts` | `eb5bdfc51659a84a60ea54e38e8e7884ff78816f` | 53 | 1..53 (full-file) | `OCRM-RESP-MEDIA-WEB-PIPELINE-001` |
| `part-0042` | `src/media/web-media.file-url.windows.test.ts` | `c5dd10459cce3d8a5716df0c5b367d30fbe865b4` | 48 | 1..48 (full-file) | `OCRM-RESP-MEDIA-WEB-PIPELINE-001` |
| `part-0042` | `src/media/remote-workspace-file.ts` | `d040aee0f0195a6dd8d689255755ba9b0a1a0110` | 138 | 1..138 (full-file) | `OCRM-RESP-MEDIA-REMOTE-WORKSPACE-FILE-001` |
| `part-0042` | `src/media/remote-workspace-file.test.ts` | `d88b49e29f07a77e112d26f7435f309a4a754167` | 190 | 1..190 (full-file) | `OCRM-RESP-MEDIA-REMOTE-WORKSPACE-FILE-001` |

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

The audited sources were mapped to 3 discrete Rust runtime responsibilities registered in `parity-ledger.json`. In strict compliance with the project's honest wiring policy, because end-to-end inference clients, workspace agent tools, and channel transports are scheduled for subsequent integration stages, these responsibilities are conservatively marked `wiring=not-wired`, `verification=not-run`, `acceptance=pending` with registered blocker IDs:

1. **`OCRM-RESP-MEDIA-ANTHROPIC-INLINE-001` (NEW)**:
   - **Behavior**: Normalization of Anthropic API content blocks by converting unsupported image formats (specifically `image/bmp`) to `image/png` while preserving supported formats (`image/jpeg`, `image/png`, `image/gif`, `image/webp`). Enforces 10MB decoded payload safety ceiling (`ANTHROPIC_INLINE_IMAGE_DECODE_SAFETY_BYTES = 10 * 1024 * 1024`) to guard against base64 memory exhaustion bombs.
   - **Rust Owner**: `source/host/src/extensions/attachments/anthropic_inline_images.rs::normalize_anthropic_inline_content_blocks, estimate_base64_decoded_bytes, AnthropicContentBlock`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `anthropic_inline_images` → Consumer: *Pending Anthropic provider pipeline integration*.
   - **Oracles**: `OCRM-ANTHROPIC-INLINE-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-ANTHROPIC-INLINE-CONSUMERS-001"]`.

2. **`OCRM-RESP-MEDIA-WEB-PIPELINE-001` (NEW)**:
   - **Behavior**: Unified web media loader and optimizer supporting local file paths, file:// URLs with percent-decoding, and remote HTTP/HTTPS resources. Enforces strict SSRF protection by forbidding Windows UNC network paths (`assert_no_windows_network_path`), blocks directory traversal escaping local root allowlists, preserves animated WebP frames against lossy single-frame transcode, and strips legacy media directive prefixes (`@media:`, `media:`).
   - **Rust Owner**: `source/host/src/extensions/attachments/web_media.rs::load_web_media, optimize_image_buffer_for_web_media, assert_no_windows_network_path, file_url_to_path, strip_legacy_media_directive_prefix`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` → Owner `web_media` → Consumer: *Pending inbound/outbound chat media attachment consumers*.
   - **Oracles**: `OCRM-WEB-MEDIA-O01`, `OCRM-WEB-MEDIA-O02`, `OCRM-WEB-MEDIA-O03`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-WEB-PIPELINE-CONSUMERS-001"]`.

3. **`OCRM-RESP-MEDIA-REMOTE-WORKSPACE-FILE-001` (NEW)**:
   - **Behavior**: Bounded remote workspace file reader for remote or sandboxed workspace files with strict monotonic deadline enforcement, adaptive chunking (`output_bytes_cap / 2`), size cap rejection, and file revision `[dev:ino:size:mtimeMs:ctimeMs]` consistency verification.
   - **Rust Owner**: `source/host/src/extensions/attachments/remote_workspace_file.rs::read_bounded_remote_workspace_file, build_file_revision, RemoteWorkspaceFileParams, RemoteWorkspaceFileError`.
   - **Production Call Chain**: Producer `WorkspaceFileTransport` → Owner `remote_workspace_file` → Consumer: *Pending remote agent workspace tool integration*.
   - **Oracles**: `OCRM-REMOTE-WORKSPACE-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-REMOTE-WORKSPACE-CONSUMERS-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Mapped paths**: **211** (increased by 8 from 203).
- **Unclassified paths**: **58,539** (decreased by 8 from 58,547).
- **Conservation Check**: `211 + 58,539 = 58,750` (100% exact parity).
- **Total Ledger Responsibilities**: **128** (125 existing + 3 newly registered responsibilities).
- **Global Completion Flags**: All 8 flags remain strictly `false`.

---

## 4. Contract Verification

All 6 contract tests in `source/host/tests/openclaw_web_media_contract.rs` pass cleanly:
1. `test_ocrm_web_media_animated_webp_preservation` — ok
2. `test_ocrm_web_media_file_url_and_traversal_guards` — ok
3. `test_ocrm_web_media_gateway_service_dispatch` — ok
4. `test_ocrm_remote_workspace_file_transfer` — ok
5. `test_ocrm_web_media_local_loading_and_roots_boundary` — ok
6. `test_ocrm_anthropic_inline_content_blocks_normalization` — ok

All 25 tests across `openclaw_media_store_contract`, `openclaw_image_ops_contract`, and `openclaw_web_media_contract` pass with 0 warnings.
