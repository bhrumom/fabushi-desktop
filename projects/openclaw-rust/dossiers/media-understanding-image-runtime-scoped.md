# OCRM Dossier: Media Understanding Image Understanding & Model Runtime

- **Unit ID**: `OCRM-UNIT-035`
- **Module ID**: `OCRM-M24` (`media-understanding`)
- **Responsibility IDs**:
  - `OCRM-RESP-MEDIA-UND-IMAGE-INPUT-001`
  - `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 11 upstream files in `src/media-understanding/` (3,232 lines total) are audited and mapped:
1. `src/media-understanding/image-input-normalize.ts` (89 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-INPUT-001`
2. `src/media-understanding/image-input-normalize.test.ts` (73 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-INPUT-001`
3. `src/media-understanding/image-model-runtime.ts` (299 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
4. `src/media-understanding/image-runtime.ts` (19 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
5. `src/media-understanding/image.ts` (547 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
6. `src/media-understanding/image.test.ts` (551 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
7. `src/media-understanding/image.test-support.ts` (292 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
8. `src/media-understanding/image.clawrouter.test.ts` (180 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
9. `src/media-understanding/image.resources.test.ts` (821 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
10. `src/media-understanding/image.runtime-profile.test.ts` (260 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`
11. `src/media-understanding/image.selected-model.test.ts` (101 lines) -> `OCRM-RESP-MEDIA-UND-IMAGE-RUNTIME-001`

## 2. Invariants & Implementation Details

- **Image Input Normalization & Limits**:
  - `is_heic_image_input`: detects HEIC/HEIF containers via MIME inspection (`image/heic`, `image/heif`, sequences) or `.heic`/`.heif` extension.
  - `effective_image_bytes_cap`: calculates minimum between global max bytes and model-specific byte ceilings.
  - `MEDIA_UNDERSTANDING_MAX_SOURCE_PIXELS`: bounded to 40,000,000 pixels.
- **Model Image Capability Validation**:
  - `require_image_capable_model`: verifies model declarations advertise `"image"` input capabilities. Throws explicit validation failure identifying model provider, model ID, and declared input types (`input: none` or `input: text`).
- **Token Resolution & Reasoning Stripping**:
  - `resolve_image_tool_max_tokens`: bounds requested token budgets against model maximums (defaulting to 4096).
  - `disable_reasoning_for_image_retry_payload`: strips `reasoning`, `reasoning_effort`, and `reasoning.encrypted_content` for empty-text retry payloads. Injects `reasoning: { effort: "none" }` for native OpenAI Responses models.
- **Provider-Specific Prompt Placement & Errors**:
  - `should_place_image_prompt_in_user_content`: returns `true` for `github-copilot` models to prevent HTTP 400 empty-user-message rejection.
  - `is_image_model_no_text_error`: identifies `Image model returned no text` failure signatures.

## 3. Rust Native Equivalence

Implemented in `source/host/src/extensions/attachments/media_understanding_service.rs`:
- `MEDIA_UNDERSTANDING_MAX_SOURCE_PIXELS`, `is_heic_image_input`, `effective_image_bytes_cap`
- `require_image_capable_model`, `resolve_image_tool_max_tokens`
- `disable_reasoning_for_image_retry_payload`, `should_place_image_prompt_in_user_content`, `is_image_model_no_text_error`

## 4. Verification Evidence

- Contract test suite: `source/host/tests/openclaw_media_understanding_contract.rs`
  - `test_ocrm_media_und_o23_image_input_normalization_and_caps`: verifies HEIC/HEIF MIME/extension detection and effective byte cap clamping.
  - `test_ocrm_media_und_o24_image_runtime_model_checks_and_token_resolution`: verifies model image capability validation and token limit resolution.
  - `test_ocrm_media_und_o25_image_reasoning_stripping_and_copilot_placement`: verifies payload reasoning removal, native responses effort manipulation, Copilot user-message placement, and error discrimination.
- All 25 tests in `openclaw_media_understanding_contract.rs` passed with 0 failures.
