# OCRM Technical Dossier — Unit 29: Video/Audio Playback Transcoding Policy, Cache Ownership & Outbound Generated HTML Provenance (`OCRM-UNIT-MEDIA-016`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Media Playback Transcoding, Store Cache, Single-Flight Inspection Permits, Outbound HTML Report Provenance & Host Read Admission)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 8 media files (totaling 2,344 lines) constitute the final unmapped files in `src/media/`. They were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/playback-transcode.ts` | `d1494a0f2f5e68114552b02b602afa0f4a2e881f` | 598 | 1..598 (full-file) | `OCRM-RESP-MEDIA-PLAYBACK-TRANSCODE-001` |
| `part-0042` | `src/media/playback-transcode.test.ts` | `d8863fe39c1db2561076d559f94042d792f3a49b` | 927 | 1..927 (full-file) | `OCRM-RESP-MEDIA-PLAYBACK-TRANSCODE-001` |
| `part-0042` | `src/media/playback-transcode.staging.test.ts` | `ec5701c2fa59037daf24ba840ff3140e028f6e21` | 225 | 1..225 (full-file) | `OCRM-RESP-MEDIA-PLAYBACK-TRANSCODE-001` |
| `part-0042` | `src/media/playback-transcode.test-support.ts` | `d57dd08b487cf944a492413e017879ecb19165de` | 91 | 1..91 (full-file) | `OCRM-RESP-MEDIA-PLAYBACK-TRANSCODE-001` |
| `part-0042` | `src/media/generated-html-provenance.ts` | `d00a635b7ee764126236a7f7c4c32969acf3f893` | 97 | 1..97 (full-file) | `OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001` |
| `part-0042` | `src/media/generated-html-provenance.test.ts` | `e49c01be45ac77f6275498eeeece0b83feb648a1` | 293 | 1..293 (full-file) | `OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001` |
| `part-0042` | `src/media/generated-html-provenance.worker.ts` | `0dd1b22b82eb26d4c53e00a8bc8bf9ae62b035fa` | 90 | 1..90 (full-file) | `OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001` |
| `part-0042` | `src/media/generated-html-provenance.worker-contract.ts` | `3d1ad1f65212e1b7dd166e647c426c38fe4283d6` | 23 | 1..23 (full-file) | `OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001` |

> [!NOTE]
> With the completion of this unit, **all 115 files in upstream `src/media/`** are completely read, classified, mapped, and accounted for in the canonical shards and parity ledger (100% media coverage).

---

## 2. Decomposed Runtime Responsibilities & Honest Calibration

1. **`OCRM-RESP-MEDIA-PLAYBACK-TRANSCODE-001` (NEW)**:
   - **Behavior**: Media playback transcoding policies, single-flight inspection permits, and deterministic cache store key management (`create_playback_transcode_cache_key`, `playback_cache_relative_path`). Coordinates FFmpeg transcoding arguments for audio (AAC, 128k, ipod container, `-vn`) and video (libx264, yuv420p, scale clamping 1080p, AAC, mp4 faststart) bounded by `PLAYBACK_TRANSCODE_MAX_ALLOC_BYTES` (256MB), `PLAYBACK_TRANSCODE_MAX_DURATION_SECS` (1200s), `PLAYBACK_TRANSCODE_MAX_INPUT_PIXELS` (16MP), 2 transcode threads, and 60s failure cooldowns. Verifies output duration matches source within tolerance window `clamp(ceil(duration * 0.02), 1000, 2000)`.
   - **Rust Owner**: `source/host/src/extensions/attachments/video_playback_rendition.rs`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` / timeline playback renderers → Owner `video_playback_rendition` → Consumer: *Chat timeline video/audio player and desktop media asset server*.
   - **Oracle**: `OCRM-PLAYBACK-TRANSCODE-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-PLAYBACK-TRANSCODE-CHAT-WIRING-001"]`.

2. **`OCRM-RESP-MEDIA-GENERATED-HTML-PROVENANCE-001` (NEW)**:
   - **Behavior**: Outbound generated HTML document provenance tracking and SQLite state store persistence (`outbound_media_provenance`). Validates markers (`trusted-generated-html` v1) with expected SHA256 and byte sizes, enforcing fail-closed host read capability checks (`HostReadHtmlTrust::Outbound { expected_sha256, expected_size }`). Prunes stale provenance entries whose underlying filesystem targets are missing, symbolic links, or hardlinked (`nlink != 1`), executing batched deletions against state storage.
   - **Rust Owner**: `source/host/src/extensions/attachments/host_read_media_policy.rs`.
   - **Production Call Chain**: Producer `AttachmentsService::dispatch_gateway` / outbound attachment staging → Owner `host_read_media_policy` → Consumer: *Host read media capability and web-media pipeline*.
   - **Oracle**: `OCRM-HTML-PROVENANCE-O01`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `blocker_ids=["BL-MEDIA-HTML-PROVENANCE-CHAT-WIRING-001"]`.

---

## 3. Strict Conservation Invariant Verification

- **Total tracked repository paths**: **58,750** (strictly conserved, zero omitted/untracked paths).
- **Cumulative mapped paths**: **264** (increased by +8 from 256; unclassified reduced from 58,494 to 58,486).
- **Active tracked responsibilities**: **139** (increased by +2 from 137).
- **8 Global Completion Flags**: All remain `false` pending full migration and verified evidence gates.

---

## 4. Contract Test Verification

Two dedicated contract test suites were authored and verified passing:

1. **`source/host/tests/openclaw_playback_transcode_contract.rs`**:
   - `test_playback_transcode_deterministic_cache_and_policy`: Verifies deterministic cache key generation from immutable file metadata and cache relative paths.
   - `test_playback_file_extension_replacement`: Verifies stem isolation, fallback to "media", and extension replacement matching Node.js `path.extname` semantics.
   - `test_playback_duration_matching_tolerance`: Verifies tolerance window clamping (1000ms–2000ms) and max duration limit (1200s).
   - `test_build_ffmpeg_playback_args_structure`: Verifies exact FFmpeg argument composition for both audio and video kinds.
   - `test_playback_transcode_resolution_variants`: Verifies enum states (`Passthrough`, `Preparing`, `Fallback`, `Transcoded`).
   - **Result**: 5 passed; 0 failed.

2. **`source/host/tests/openclaw_html_provenance_contract.rs`**:
   - `test_html_provenance_marker_validation`: Verifies `trusted-generated-html` v1 marker extraction and rejection of invalid versions or kinds.
   - `test_html_provenance_filter_stale_rows`: Verifies stale row detection for missing files and symbolic links.
   - `test_html_provenance_host_read_admission`: Verifies outbound HTML trust gating requiring matching SHA256 and byte size.
   - **Result**: 3 passed; 0 failed.
