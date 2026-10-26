# OCRM Technical Dossier — Unit 14: Crash Guard, Diagnostics & Process Health (`OCRM-UNIT-CRASH-GUARD-001`)

**Project ID**: OCRM-001  
**Module**: OCRM-M08/M22/M23 (Crash Guard, Diagnostics, Media Attachments & Persistence)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 6 core crash guard, diagnostics, attachment storage paths, media MIME resolution, and persistence files (totaling 525 lines) were audited line-by-line and mapped into canonical shards:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0026` | `src/agents/utils/mime.ts` | `7744266bb3bbeafbbf778b654a9be5831275b4da` | 116 | 1..116 (full-file) | `OCRM-RESP-MEDIA-IMAGE-MIME-DETECT-001` |
| `part-0025` | `src/agents/subagents/subagent-attachment-paths.ts` | `8b8817362005640491d5bb6c40503306e9a804ed` | 30 | 1..30 (full-file) | `OCRM-RESP-SUBAGENT-ATTACHMENT-PATHS-001` |
| `part-0042` | `src/media/attachment-processor.ts` | `0dc184f58bfa76877108d9d57eaa3a3f06e59bd1` | 34 | 1..34 (full-file) | `OCRM-RESP-MEDIA-ATTACHMENT-PROCESSOR-001` |
| `part-0042` | `src/media/outbound-attachment.ts` | `d2c00c2905323763fb58cb61b4e3aeb5bf37a250` | 65 | 1..65 (full-file) | `OCRM-RESP-MEDIA-OUTBOUND-ATTACHMENT-001` |
| `part-0045` | `src/process/spawn-diagnostics.ts` | `1dec1b04c0247e5de0d172e42c32bfe4c4903110` | 235 | 1..235 (full-file) | `OCRM-RESP-PROCESS-SPAWN-DIAGNOSTICS-001` |
| `part-0039` | `src/infra/error-diagnostics.ts` | `226092ad8d0bde17891ebc0310fd7919fd80a9ce` | 45 | 1..45 (full-file) | `OCRM-RESP-ERROR-DIAGNOSTICS-ATTACH-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-MEDIA-IMAGE-MIME-DETECT-001`**:
   - **Behavior**: Deep byte-level image MIME detection matching upstream `detectSupportedImageMimeType` (JPEG byte 4 `0xf7` rejection for lossless JPEG, static PNG magic byte verification, animated PNG rejection via `acTL` chunk inspection before `IDAT` chunk, GIF87a/GIF89a validation, WebP RIFF/WEBP/VP8 signature check, and BMP DIB header structure validation where `pixelDataOffset < declaredFileSize`).
   - **Rust Owner**: `source/host/src/media_mime.rs::detect_supported_image_mime_type`.
   - **Oracles**: `OCRM-MEDIA-O01`, `OCRM-MEDIA-O02`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

2. **`OCRM-RESP-SUBAGENT-ATTACHMENT-PATHS-001`**:
   - **Behavior**: Deterministic resolution of subagent session attachment directory trees mounted under `/openclaw/attachments` using normalized agent ID and SHA-256 session ref, preserving run-level subagent attachment directory trees.
   - **Rust Owner**: `source/host/src/attachment_paths.rs::resolve_subagent_session_attachment_root_dir`.
   - **Oracles**: `OCRM-MEDIA-O04`, `OCRM-MEDIA-O06`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

3. **`OCRM-RESP-MEDIA-ATTACHMENT-PROCESSOR-001`**:
   - **Behavior**: Attachment payload validation and preparation, calculating exact base64 decoded size and enforcing bounds limits before materialization.
   - **Rust Owner**: `source/host/src/attachment_paths.rs::validate_and_prepare_attachment_payload`.
   - **Oracle**: `OCRM-MEDIA-O05`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

4. **`OCRM-RESP-MEDIA-OUTBOUND-ATTACHMENT-001`**:
   - **Behavior**: Resolves outbound attachment buffers into persisted media files inside agent outbound directory with byte bounds checks and MIME sniffing.
   - **Rust Owner**: `source/host/src/attachment_paths.rs::resolve_outbound_attachment_from_buffer`.
   - **Oracles**: `OCRM-MEDIA-O08`, `OCRM-MEDIA-O09`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

5. **`OCRM-RESP-PROCESS-SPAWN-DIAGNOSTICS-001`**:
   - **Behavior**: Child process spawn telemetry tracking, command family classification (git, shell, node, python, rust, docker, curl, etc.), unscoped operation categorization, and atomic sample recording matching upstream `spawn-diagnostics.ts`.
   - **Rust Owner**: `source/host/src/process_spawn_diagnostics.rs::ChildProcessSpawnTracker`.
   - **Oracles**: `OCRM-CRASH-O02`.
   - **Status**: Honest calibration: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-SPAWN-DIAGNOSTICS-DAEMON-WIRING-001"]`: Process spawn diagnostics tracker is implemented and verified in standalone unit contract, pending daemon invocation hooks across background runners).

6. **`OCRM-RESP-ERROR-DIAGNOSTICS-ATTACH-001`**:
   - **Behavior**: Attaches sanitized diagnostic metadata to error objects, masking sensitive credentials (`Bearer <redacted>`, `sk-<redacted-secret>`, `password=<redacted>`) and producing bounded display format strings under 2,048 UTF-16 code units.
   - **Rust Owner**: `source/host/src/error_diagnostics.rs::sanitize_error_diagnostic`.
   - **Oracle**: `OCRM-CRASH-O03`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

---

## 3. Falsifiable Test Oracles (`source/host/tests/openclaw_crash_guard_media_contract.rs`)

1. **`OCRM-MEDIA-O01`**: `test_ocrm_media_o01_deep_image_mime_detection` (verifies JPEG 0xf7 rejection, APNG acTL rejection, and valid JPEG/PNG/GIF/WebP/BMP).
2. **`OCRM-MEDIA-O02`**: `test_ocrm_media_o02_servable_image_mime` (verifies extension-based servable mime mapping).
3. **`OCRM-MEDIA-O03`**: `test_ocrm_media_o03_video_audio_mime_resolution` (verifies audio/video extension and mime sniffing).
4. **`OCRM-MEDIA-O04`**: `test_ocrm_media_o04_subagent_attachment_paths` (verifies sandbox mount `/openclaw/attachments` and sha256 session ref path).
5. **`OCRM-MEDIA-O05`**: `test_ocrm_media_o05_attachment_processor_size_limits` (verifies base64 decoded size calculation and max_bytes rejection).
6. **`OCRM-MEDIA-O06`**: `test_ocrm_media_o06_storage_path_resolution` (verifies agent attachments, assets, and media store roots).
7. **`OCRM-MEDIA-O07`**: `test_ocrm_media_o07_safe_media_leaf` (verifies safe leaf filename sanitization and traversal prevention).
8. **`OCRM-MEDIA-O08`**: `test_ocrm_media_o08_outbound_attachment_staging` (verifies staging buffer into agent outbound directory).
9. **`OCRM-MEDIA-O09`**: `test_ocrm_media_o09_persist_media_bytes` (verifies byte persistence with collision handling).
10. **`OCRM-CRASH-O01`**: `test_ocrm_crash_o01_panic_kind_and_hook` (verifies ProcessCrashKind panic vs error categorization).
11. **`OCRM-CRASH-O02`**: `test_ocrm_crash_o02_spawn_diagnostics_tracker` (verifies child process spawn tracking, command family classification, and counter updates).
12. **`OCRM-CRASH-O03`**: `test_ocrm_crash_o03_error_diagnostics_redaction` (verifies credential sanitization, Bearer/sk- masking, and length truncation).

---

## 4. Invariant and Metric Ledger

- **Inventory Tracking**:
  - `mapped_paths`: 142
  - `unclassified_paths`: 58,608
  - `tracked_paths`: 58,750 (strictly conserved: $142 + 58,608 = 58,750$)
  - `omitted_known`: 0
- **Responsibilities**:
  - Total: 84
  - Active blockers: `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001`, `BL-SPAWN-DIAGNOSTICS-DAEMON-WIRING-001`
- **Global Completion Flags**:
  - All 8 flags remain `false`.
