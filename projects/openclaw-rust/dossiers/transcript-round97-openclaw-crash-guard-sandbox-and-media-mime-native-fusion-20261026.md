# OCRM Technical Dossier: Round 97 — OpenClaw Crash Guard, Sandbox & Media Native Fusion

## 1. Executive Summary

This dossier records the full-source native Rust equivalence and fusion for:
1. **`OCRM-M18`**: Sandbox Isolation, Process Crash Guard, Panic Handling & Local Exec (`src/sandbox/`, `src/process/`, `source/host/src/process_crash_guard.rs`, `source/host/src/box/`)
2. **`OCRM-M19`**: Media Processing, MIME Resolution, Audio/Video Codecs & Attachments (`src/media/`, `source/host/src/media_mime.rs`, `source/host/src/attachment_paths.rs`)

All responsibilities from upstream `openclaw/openclaw` in `src/sandbox/`, `src/process/`, and `src/media/` are mapped natively into canonical Fabushi Rust architecture under `source/host/src/process_crash_guard.rs`, `source/host/src/media_mime.rs`, and `source/host/src/attachment_paths.rs`.

---

## 2. Upstream Source to Canonical Fabushi Rust Mapping

### 2.1 Process Crash Guard & Sandbox Execution (`OCRM-M18`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/process/panic.ts` | Process panic capture & thread safety boundary | `ProcessCrashGuard`, `ProcessCrashKind` | Host Crash Guard |
| `src/process/reporter.ts` | Crash event reporting callback | `ProcessCrashReporter`, `set_reporter` | Host Crash Guard |
| `src/process/normalize.ts` | Panic stack & location normalization | `normalize_panic`, `catch_unwind` | Host Crash Guard |
| `src/sandbox/box-exec.ts` | Sandbox subprocess execution boundary | `box_help_tool`, `box_mcp_exec`, `r#box` | Host Box Supervisor |

### 2.2 Media Processing, MIME Resolution & Attachments (`OCRM-M19`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/media/image-mime.ts` | Common image MIME resolution by extension | `image_mime_from_path` (png, jpg, webp, svg, gif) | Host Media MIME |
| `src/media/container-mime.ts` | Container image format resolution | `servable_image_mime_from_path` (heic, heif) | Host Media MIME |
| `src/media/video-mime.ts` | Video container MIME resolution | `video_mime_from_path` (mp4, webm, mov, ogv) | Host Media MIME |
| `src/media/audio-mime.ts` | Audio container MIME resolution | `audio_mime_from_path` (mp3, flac, m4a, aac) | Host Media MIME |
| `src/media/size-limits.ts` | Attachment & video size cap enforcement | `ATTACHMENT_BYTE_LIMIT` (25 MiB), `VIDEO_BYTE_LIMIT` (200 MiB) | Host Media MIME |
| `src/media/paths.ts` | Media storage roots & directory isolation | `get_agent_attachments_dir`, `get_agent_assets_dir` | Host Attachment Paths |
| `src/media/sanitize.ts` | Filename leaf sanitation & traversal prevention | `safe_media_leaf` (160 char cap, ASCII clean) | Host Attachment Paths |
| `src/media/persistence.ts` | Binary media persistence to disk | `persist_agent_media_bytes`, `AgentMediaKind` | Host Attachment Paths |

---

## 3. Falsifiable Test Oracles

1. **`OCRM-CRASH-O01` (Kind String Representation)**: Validates `ProcessCrashKind::Panic.as_str() == "panic"`.
2. **`OCRM-CRASH-O02` (Reporter Registration)**: Validates registration and unregistration of thread-safe crash reporters.
3. **`OCRM-CRASH-O03` (Panic Handling)**: Validates panic capture and safe boundary containment without crashing host process.
4. **`OCRM-MEDIA-O01` (Image MIME Detection)**: Validates standard image extensions (`.png`, `.jpg`, `.jpeg`, `.svg`, `.webp`, `.ico`).
5. **`OCRM-MEDIA-O02` (Servable Containers)**: Validates HEIC/HEIF servable media format detection.
6. **`OCRM-MEDIA-O03` (Video MIME Detection)**: Validates `.mp4`, `.m4v`, `.webm`, `.mov`, `.ogv`.
7. **`OCRM-MEDIA-O04` (Audio MIME Detection)**: Validates `.mp3`, `.flac`, `.m4a`, `.aac`.
8. **`OCRM-MEDIA-O05` (Byte Limits)**: Validates 25 MiB attachment limit and 200 MiB video limit.
9. **`OCRM-MEDIA-O06` (Storage Path Roots)**: Validates attachments and assets directory isolation per agent.
10. **`OCRM-MEDIA-O07` (Safe Media Leaf)**: Validates path traversal mitigation and filename sanitization.
11. **`OCRM-MEDIA-O08` (Media Kind Classification)**: Validates `AgentMediaKind::Image` vs `AgentMediaKind::Attachment`.
12. **`OCRM-MEDIA-O09` (Binary Persistence)**: Validates transactional file writing and directory creation.
