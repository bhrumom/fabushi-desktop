# OCRM Dossier: Media Understanding Runner & Execution Closure

- **Unit ID**: `OCRM-UNIT-037`
- **Module ID**: `OCRM-M24` (`media-understanding`)
- **Responsibility IDs**:
  - `OCRM-RESP-MEDIA-UND-RUNNER-001`
  - `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 23 upstream files in `src/media-understanding/` (6,316 lines total) complete the 100% audit and mapping of `src/media-understanding/`:
1. `src/media-understanding/resolve.ts` (236 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
2. `src/media-understanding/resolve.test.ts` (87 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
3. `src/media-understanding/cli-output.ts` (143 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
4. `src/media-understanding/runner.ts` (839 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
5. `src/media-understanding/runner.attachments.ts` (67 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
6. `src/media-understanding/runner.attachments.test.ts` (28 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
7. `src/media-understanding/runner.entries.ts` (664 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
8. `src/media-understanding/runner.entries.guards.test.ts` (203 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
9. `src/media-understanding/runner.auto-audio.test.ts` (386 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
10. `src/media-understanding/runner.cli-audio.test.ts` (531 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
11. `src/media-understanding/runner.video.test.ts` (168 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
12. `src/media-understanding/runner.vision-skip.test.ts` (755 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
13. `src/media-understanding/runner.local-no-auth.test.ts` (143 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
14. `src/media-understanding/runner.test-mocks.ts` (49 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
15. `src/media-understanding/runner.test-utils.ts` (88 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
16. `src/media-understanding/runner-case.test-support.ts` (17 lines) -> `OCRM-RESP-MEDIA-UND-RUNNER-001`
17. `src/media-understanding/runtime-types.ts` (120 lines) -> `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`
18. `src/media-understanding/runtime.test.ts` (581 lines) -> `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`
19. `src/media-understanding/runtime.audio-authority.test.ts` (156 lines) -> `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`
20. `src/media-understanding/runtime.audio-disposition.test.ts` (89 lines) -> `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`
21. `src/media-understanding/runtime.local-input.test.ts` (35 lines) -> `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`
22. `src/media-understanding/apply.runtime.ts` (2 lines) -> `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`
23. `src/media-understanding/apply.test.ts` (929 lines) -> `OCRM-RESP-MEDIA-UND-APPLY-RUNTIME-001`

With this Unit, all 89 files in `src/media-understanding/` are 100% audited, mapped, and natively implemented.

## 2. Invariants & Implementation Details

- **Model Entry & CLI Resolution**:
  - `resolve_cli_model_entry`: checks command presence and non-empty args; returns explicit error tokens `cli-missing-command` or `cli-missing-attachment-arg`.
  - `resolve_timeout_ms` & `resolve_media_runtime_timeout_ms`: enforces timer-safe conversion clamped within `[1_000ms, 2_147_483_647ms]`.
  - `resolve_entry_run_prompt`: formats per-capability prompts with character limits for vision models while avoiding instruction pollution for audio.
- **Scope & Preference Policies**:
  - `resolve_scope_decision`: evaluates channel, chat-type, and session-key prefix rules in first-match-wins order, falling back to scope default.
  - `preferred_media_model_rank`: prioritizes exact qualified matches (`provider/model`), followed by bare model matches, provider-default matches, and CLI prefix matches.
  - Secret ownership: deterministic format `media-model:shared:{index}`.
- **CLI Output Extraction**:
  - `extract_sherpa_onnx_text`: JSON extraction with reverse-line scan fallback.
  - `resolve_whisper_cpp_output_path`, `resolve_whisper_output_path`, `resolve_parakeet_output_path`: parses CLI flag permutations (`-otxt`, `-of`, `--output_dir`, `--output-format`).
- **Decision Summaries & Dispositions**:
  - `format_decision_summary`: compact string representation showing capability, outcome, success ratio, model, backend observations, and summarized failure reasons.
  - Native media deliverability: validates that local file paths or `media://` URLs can be natively hydrated, flagging unsupported remote URLs.
  - Sticker media rule: strips index 0 when cached stickers are bypassed.

## 3. Rust Native Equivalence

Implemented in `source/host/src/extensions/attachments/media_understanding_service.rs`:
- Constants: `DEFAULT_MEDIA_RUNTIME_TIMEOUT_MS`, `MIN_MEDIA_TIMEOUT_MS`, `MAX_TIMER_TIMEOUT_MS`, `DEFAULT_MEDIA_CONCURRENCY`, `MIN_AUDIO_FILE_BYTES`, `CLI_OUTPUT_MAX_BUFFER`.
- `resolve_cli_model_entry`, `resolve_timeout_ms`, `resolve_media_runtime_timeout_ms`, `resolve_entry_run_prompt`.
- `MediaScopeRuleMatch`, `MediaScopeRule`, `MediaUnderstandingScopeConfig`, `ScopeContext`, `resolve_scope_decision`.
- `preferred_media_model_rank`, `resolve_model_entry_secret_owner_id`.
- `extract_sherpa_onnx_text`, `resolve_whisper_output_path`, `resolve_whisper_cpp_output_path`, `resolve_parakeet_output_path`.
- `normalize_decision_reason`, `summarize_decision_reason`, `format_decision_summary`.
- `resolve_audio_input_budget_for_config`, `is_native_media_deliverable`, `normalize_media_attachments_with_sticker_rule`.

## 4. Verification Evidence

- Contract test suite: `source/host/tests/openclaw_media_understanding_contract.rs`
  - `test_ocrm_media_und_o28_cli_entry_and_timeout_resolution`: tests CLI resolution failures, valid CLI parsing, timeout clamping to `MAX_TIMER_TIMEOUT_MS`, and prompt formatting.
  - `test_ocrm_media_und_o29_scope_decision_and_preference_ranking`: tests scope rule matching (channel, chat-type, prefix) and preference ranking.
  - `test_ocrm_media_und_o30_cli_output_parsing_and_sherpa_onnx`: tests Sherpa ONNX JSON parsing, Whisper.cpp `-otxt`/`-of` paths, and Python Whisper output dirs.
  - `test_ocrm_media_und_o31_decision_summary_and_runtime_disposition`: tests decision reason normalization, formatted summary strings, audio budget, native deliverability, and sticker skipping.
- 31 of 31 tests passed cleanly with 0 failures.
