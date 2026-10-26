# OCRM Technical Dossier — Unit 18: Audio Transcoding, FFmpeg Limits, Voice Compatibility & Media Reference Normalization (`OCRM-UNIT-MEDIA-005`)

**Project ID**: OCRM-001  
**Module**: OCRM-M24 (Audio Transcoding, FFmpeg Execution Limits, Voice Compatibility & Media Reference Comparison)  
**Upstream Target Authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
**Target Repository**: `bhrumom/fabushi-desktop`  
**Working Branch**: `ocrm/round20-snapshot-lifecycle-audit-20261023`  
**Reviewer**: `xuyao@ombhrum.com`  

---

## 1. Upstream Source Audit & Line-by-Line Reading

The following 7 core audio transcoding, FFmpeg execution limit, voice compatibility, comparison, and prompt image input files (totaling 442 lines) were audited line-by-line from the locked upstream repository and mapped into canonical inventory shard `part-0042`:

| Shard | Source File | Blob SHA | Lines | Scope | Mapped Responsibility |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `part-0042` | `src/media/ffmpeg-limits.ts` | `930e010eebfac3c00aedf618e1cfb7e6e99a9c2f` | 8 | 1..8 (full-file) | `OCRM-RESP-MEDIA-FFMPEG-LIMITS-001` |
| `part-0042` | `src/media/ffmpeg-exec.ts` | `264a0b3bac20cbaf23178353003bf3a0ffc5f713` | 97 | 1..97 (full-file) | `OCRM-RESP-MEDIA-FFMPEG-EXEC-001` |
| `part-0042` | `src/media/audio.ts` | `9b6c0d7ba0a740137519b1f609931159471ca6f9` | 39 | 1..39 (full-file) | `OCRM-RESP-MEDIA-VOICE-AUDIO-COMPAT-001` |
| `part-0042` | `src/media/audio-transcode.ts` | `ca6927f3285866b27418eccce344e548e59fea63` | 196 | 1..196 (full-file) | `OCRM-RESP-MEDIA-AUDIO-TRANSCODE-001` |
| `part-0042` | `src/media/media-reference-comparison.ts` | `424dfbe15df6fc4c69d6cb23051773affd966325` | 72 | 1..72 (full-file) | `OCRM-RESP-MEDIA-REF-COMPARISON-001` |
| `part-0042` | `src/media/prompt-image-input.ts` | `4179c24c0f155d2ffc935831aa2b8407e607963f` | 28 | 1..28 (full-file) | `OCRM-RESP-MEDIA-PROMPT-IMAGE-INPUT-001` |
| `part-0042` | `src/media/prompt-image-order.ts` | `f20eab0ac1c55230f232325a60db29d010c39282` | 2 | 1..2 (full-file) | `OCRM-RESP-MEDIA-PROMPT-IMAGE-INPUT-001` |

---

## 2. Decomposed Runtime Responsibilities

The audited source code was decomposed into 6 discrete Rust runtime responsibilities registered in `parity-ledger.json`:

1. **`OCRM-RESP-MEDIA-FFMPEG-LIMITS-001`**:
   - **Behavior**: Shared timeout, buffer cap (10MB), and max audio duration (20m) constraints for FFmpeg helper invocations.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_audio_transcode.rs::MEDIA_FFMPEG_MAX_BUFFER_BYTES`.
   - **Production Call Chain**: Producer `video_playback_rendition.rs::run_video_tool` → Enforce Buffer and Timeout → Process Execution.
   - **Oracles**: `OCRM-AUDIO-O06`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

2. **`OCRM-RESP-MEDIA-FFMPEG-EXEC-001`**:
   - **Behavior**: Compact ffprobe stdout parsing for audio codec and positive sample rate.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_audio_transcode.rs::parse_ffprobe_codec_and_sample_rate`.
   - **Production Call Chain**: Probe output → `parse_ffprobe_codec_and_sample_rate` → Audio Transcode Validation (pending active audio transcoding pipeline).
   - **Oracles**: `OCRM-AUDIO-O02`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-FFMPEG-EXEC-UPSTREAM-001"]`: ffprobe audio codec/sample rate parser not yet invoked in active audio transcoding pipeline).

3. **`OCRM-RESP-MEDIA-VOICE-AUDIO-COMPAT-001`**:
   - **Behavior**: Classification of voice-message compatible audio formats by MIME type and filename extension.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_audio_transcode.rs::is_voice_message_compatible_audio`.
   - **Production Call Chain**: Inbound voice attachment → `connectors/channel_attachment.rs::is_voice_message_attachment` → Channel Dispatch.
   - **Oracles**: `OCRM-AUDIO-O01`.
   - **Status**: `implementation=implemented`, `wiring=wired`, `verification=verified`.

4. **`OCRM-RESP-MEDIA-AUDIO-TRANSCODE-001`**:
   - **Behavior**: Audio transcode parameters normalization and FFmpeg Opus command construction (mono 48kHz, 64k bitrate, duration limit).
   - **Rust Owner**: `source/host/src/extensions/attachments/media_audio_transcode.rs::build_ffmpeg_opus_args`.
   - **Production Call Chain**: Input audio stream → `build_ffmpeg_opus_args` → Native FFmpeg Transcode Pipeline (pending active audio transcode caller).
   - **Oracles**: `OCRM-AUDIO-O05`, `OCRM-AUDIO-O06`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-AUDIO-TRANSCODE-UPSTREAM-001"]`: Opus transcode arguments builder not yet connected to audio message transcode caller).

5. **`OCRM-RESP-MEDIA-REF-COMPARISON-001`**:
   - **Behavior**: Canonicalization of equivalent local media references without filesystem resolution (traversal protection, `%2f` preservation).
   - **Rust Owner**: `source/host/src/extensions/attachments/media_audio_transcode.rs::normalize_media_reference_for_comparison`.
   - **Production Call Chain**: Attachment URL/path → `normalize_media_reference_for_comparison` → Inbound Cache Key (pending active channel deduplication caller).
   - **Oracles**: `OCRM-AUDIO-O03`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-REF-COMPARISON-UPSTREAM-001"]`: Media reference comparison normalization not yet hooked into channel attachment deduplication).

6. **`OCRM-RESP-MEDIA-PROMPT-IMAGE-INPUT-001`**:
   - **Behavior**: Detection of prompt images requiring vision model inputs vs hydration-suppressed facts.
   - **Rust Owner**: `source/host/src/extensions/attachments/media_audio_transcode.rs::has_prompt_image_input`.
   - **Production Call Chain**: Turn input message facts → `has_prompt_image_input` → Model Selector / Capabilities (pending active inference provider prompt selector).
   - **Oracles**: `OCRM-AUDIO-O04`.
   - **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run` (`blocker_ids=["BL-MEDIA-PROMPT-IMAGE-INPUT-UPSTREAM-001"]`: Prompt image input detection helper not yet wired to inference provider prompt selector).

---

## 3. Verification & Evidence

- **Contract Test Suite**: `source/host/tests/openclaw_audio_transcode_contract.rs`
  - 6 passing test cases:
    - `test_ocrm_audio_o01_voice_message_compatible_audio`: verifies compatible extensions (`.oga`, `.ogg`, `.opus`, `.mp3`, `.m4a`) and compatible MIMEs, rejecting incompatible formats like WAV or FLAC.
    - `test_ocrm_audio_o02_ffprobe_codec_and_sample_rate_parsing`: verifies multi-delimiter grouping (`\r\n`, `,`) and sample rate parsing.
    - `test_ocrm_audio_o03_media_reference_comparison_normalization`: verifies path normalization, literal retention of `..` segments, `file://` decoding, and `%2f` separator preservation.
    - `test_ocrm_audio_o04_prompt_image_input_detection`: verifies image detection, suppression index exclusions, and hydration suppression skipping.
    - `test_ocrm_audio_o05_audio_extension_and_temp_normalization`: verifies extension normalization, prefix sanitization, and output filename security.
    - `test_ocrm_audio_o06_build_ffmpeg_opus_args`: verifies FFmpeg Opus CLI construction with `-c:a libopus`, `-b:a 64k`, `-ar 48000`, `-ac 1`, and `-t` duration capping.
- **Strict Conservation Invariant**:
  - `mapped_paths`: 167
  - `unclassified_paths`: 58,583
  - `tracked_paths`: 58,750 ($167 + 58,583 = 58,750$)
  - `responsibilities`: 108
  - Active blockers: `BL-CURSOR-BACKEND-IMAGE-RESOLUTION-001`, `BL-MEDIA-FFMPEG-EXEC-UPSTREAM-001`, `BL-MEDIA-AUDIO-TRANSCODE-UPSTREAM-001`, `BL-MEDIA-REF-COMPARISON-UPSTREAM-001`, `BL-MEDIA-PROMPT-IMAGE-INPUT-UPSTREAM-001`.
  - All 8 completion flags remain `false`.
