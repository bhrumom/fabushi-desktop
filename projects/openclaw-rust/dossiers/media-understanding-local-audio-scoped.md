# OCRM Dossier: Media Understanding Local Audio & OpenAI Compatible Audio/Video

- **Unit ID**: `OCRM-UNIT-034`
- **Module ID**: `OCRM-M24` (`media-understanding`)
- **Responsibility IDs**:
  - `OCRM-RESP-MEDIA-UND-LOCAL-AUDIO-001`
  - `OCRM-RESP-MEDIA-UND-OPENAI-COMPAT-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 9 upstream files in `src/media-understanding/` (1,092 lines total) are audited and mapped:
1. `src/media-understanding/local-audio.ts` (388 lines) -> `OCRM-RESP-MEDIA-UND-LOCAL-AUDIO-001`
2. `src/media-understanding/local-audio.test.ts` (273 lines) -> `OCRM-RESP-MEDIA-UND-LOCAL-AUDIO-001`
3. `src/media-understanding/local-audio.test-support.ts` (25 lines) -> `OCRM-RESP-MEDIA-UND-LOCAL-AUDIO-001`
4. `src/media-understanding/openai-audio-api.ts` (15 lines) -> `OCRM-RESP-MEDIA-UND-OPENAI-COMPAT-001`
5. `src/media-understanding/openai-compatible-audio.ts` (80 lines) -> `OCRM-RESP-MEDIA-UND-OPENAI-COMPAT-001`
6. `src/media-understanding/openai-compatible-audio.test.ts` (85 lines) -> `OCRM-RESP-MEDIA-UND-OPENAI-COMPAT-001`
7. `src/media-understanding/openai-compatible-audio.pin-dns.test.ts` (58 lines) -> `OCRM-RESP-MEDIA-UND-OPENAI-COMPAT-001`
8. `src/media-understanding/openai-compatible-auth.test.ts` (99 lines) -> `OCRM-RESP-MEDIA-UND-OPENAI-COMPAT-001`
9. `src/media-understanding/openai-compatible-video.ts` (69 lines) -> `OCRM-RESP-MEDIA-UND-OPENAI-COMPAT-001`

## 2. Invariants & Implementation Details

- **Local Audio Detection & Selection**:
  - `command_id`: Normalizes command paths by stripping directory paths, executable extensions (`.exe`, `.bat`, `.cmd`, `.com`), and lowercasing.
  - `resolve_requested_local_audio_backend`:
    - `sherpa-onnx-offline`: inspects `--provider <val>` or `--provider=<val>`, defaulting to `"cpu"`.
    - `whisper-cli`: checks for `-ng` / `--no-gpu` -> `"cpu"`, or `-dev <n>` / `--device <n>` / `--device=<n>` -> `"device:<n>"`.
  - `record_local_audio_backend_observation`:
    - Parses runtime output for backend acceleration notices.
    - Matches `failed to initialize (MTL\d+|Metal|CUDA\d*) backend` -> `"cpu"`.
    - Matches `using (MTL\d+|Metal) backend` -> `"metal"`.
    - Matches `using CUDA\d* backend` -> `"cuda"`.
    - Matches `using CPU backend` or `no GPU found` -> `"cpu"`.
  - Candidate Ranking (`rank_local_audio_candidate`):
    - `whisper-cli` with observed `metal` or `cuda`: Rank 0
    - `sherpa-onnx-offline`: Rank 1
    - `whisper-cli` (default/unobserved/cpu): Rank 2
    - `parakeet-mlx`: Rank 3
    - `whisper` (Python CLI): Rank 4
  - `format_local_audio_selection`: formats selected runner, backend details (`capable`, `requested`, `observed`), and fallback candidates.
- **OpenAI-Compatible Audio/Video Integration**:
  - `OPENAI_AUDIO_TRANSCRIPTIONS_API`: defined as `"openai-audio-transcriptions"`.
  - `resolve_openai_audio_auth_model_api`: resolves model API contract only for audio capability and `"openai"` provider.
  - AAC remapping: `build_audio_transcription_form_data` automatically remaps `.aac` uploads to `.m4a` filenames to conform with upstream OpenAI `/audio/transcriptions` ingestion constraints.
  - `build_openai_compatible_video_request_body`: structures chat completion message containing user prompt text and `video_url` (`data:{mime};base64,...`).
  - `coerce_openai_compatible_video_text`: parses completion choice message content (string or array of text parts) or falls back to `reasoning_content`.

## 3. Rust Native Equivalence

Implemented in `source/host/src/extensions/attachments/media_understanding_service.rs`:
- `OPENAI_AUDIO_TRANSCRIPTIONS_API`, `resolve_openai_audio_auth_model_api`
- `resolve_media_understanding_string`, `coerce_openai_compatible_video_text`
- `build_openai_compatible_video_request_body`, `AudioTranscriptionFormData`, `build_audio_transcription_form_data`
- `command_id`, `resolve_requested_local_audio_backend`, `record_local_audio_backend_observation`, `rank_local_audio_candidate`
- `LocalAudioCandidate`, `LocalAudioSelection`, `format_local_audio_selection`

## 4. Verification Evidence

- Contract test suite: `source/host/tests/openclaw_media_understanding_contract.rs`
  - `test_ocrm_media_und_o19_openai_audio_api_and_transcription_form`: verifies API contract resolution, AAC to M4A remapping, and whitespace normalization.
  - `test_ocrm_media_und_o20_openai_compatible_video_request_and_coercion`: verifies fallback strings, chat completion video request payload formatting, and message content / reasoning coercion.
  - `test_ocrm_media_und_o21_local_audio_command_and_backend_detection`: verifies command stem normalization, requested backend resolution (sherpa provider, whisper-cli device/no-gpu), and output log observation (Metal, CUDA, CPU fallback).
  - `test_ocrm_media_und_o22_local_audio_candidate_ranking_and_selection_formatting`: verifies priority ranking and selection formatting with fallbacks.
- All 22 tests in `openclaw_media_understanding_contract.rs` pass cleanly.
