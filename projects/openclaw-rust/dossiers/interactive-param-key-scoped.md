# OCRM Dossier: Interactive Message Presentation, Portable Cards & Parameter Snake-Casing

- **Unit ID**: `OCRM-UNIT-043`
- **Module ID**: `OCRM-M27` (`interactive`)
- **Responsibility IDs**:
  - `OCRM-RESP-INTERACTIVE-PRESENTATION-001`
  - `OCRM-RESP-PARAM-KEY-NORMALIZATION-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 8 upstream files (2,116 lines total) are audited, mapped, and natively implemented:

### `src/interactive/` (3 files, 1,768 lines) -> `OCRM-RESP-INTERACTIVE-PRESENTATION-001`
1. `src/interactive/payload.ts` (1,201 lines)
2. `src/interactive/payload.test.ts` (530 lines)
3. `src/interactive/payload.fallback.test.ts` (37 lines)

### Parameter Normalization & Public Facades (5 files, 348 lines) -> `OCRM-RESP-PARAM-KEY-NORMALIZATION-001`
1. `src/param-key.ts` (31 lines)
2. `src/library.ts` (50 lines)
3. `src/library.test.ts` (79 lines)
4. `src/install-sh-version.test.ts` (89 lines)
5. `src/plugin-activation-boundary.test.ts` (99 lines)

## 2. Invariants & Implementation Details

- **Portable Message Presentation & Component Blocks**:
  - `MessagePresentation` models portable cards with optional title, description, tone (`info`, `success`, `warning`, `danger`, `neutral`), and component blocks (`Buttons`, `Select`, `Text`).
  - Action types support `Command`, `Callback`, `Url`, `Approval`, `Question`, and `ModelPicker`.
  - Question actions enforce strict intent validation: only `custom-input` or non-empty `option_value` are accepted. Unknown intents cause presentation normalization to fail closed (`None`).
- **Safe Fallback Text Rendering**:
  - Renders interactive select menus as markdown lists: `<Placeholder>:\n- <Label>: \`<command>\`\n- <Label>`.
  - **Security Critical**: Opaque callback tokens are intentionally withheld from fallback text transcripts to prevent leaking private operational/authorization tokens to human transcripts or insecure chat channels.
  - Buttons format as clickable labels or formatted commands (`[ Label: \`/command\` ]`).
- **Parameter Key Normalization**:
  - `to_snake_case_key` transforms camelCase and multi-letter acronyms (`parseURL` $\to$ `parse_url`, `modelID` $\to$ `model_id`) to snake_case while preserving surrounding whitespace.
  - `resolve_snake_case_param_key` and `read_snake_case_param_raw` match direct properties first, then evaluate the normalized snake_case variant.

## 3. Rust Native Owner & Verification Evidence

- **Rust Implementation**:
  - `source/host/src/extensions/interactive/payload_service.rs`
  - `source/host/src/extensions/interactive/param_key.rs`
  - Registered in `source/host/src/extensions/interactive/mod.rs` and `source/host/src/extensions/mod.rs`
- **Contract Tests**:
  - `source/host/tests/openclaw_interactive_contract.rs`
    - `test_ocrm_param_key_normalization`
    - `test_ocrm_interactive_fallback_select_does_not_leak_callback`
    - `test_ocrm_interactive_fallback_buttons_and_card`
    - `test_ocrm_interactive_action_normalization`
- **Verification Result**: 4 passed; 0 failed.
