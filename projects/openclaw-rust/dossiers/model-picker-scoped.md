# OCRM Dossier: Model Picker Capabilities, Placement Fences & Session Selection

- **Unit ID**: `OCRM-UNIT-040`
- **Module ID**: `OCRM-M06` (`model-picker`)
- **Responsibility IDs**:
  - `OCRM-RESP-MODEL-PICKER-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 6 upstream files in `src/model-picker/` (1,328 lines total, 100% of the directory) are audited, mapped, and natively implemented:

1. `src/model-picker/capabilities.ts` (14 lines)
2. `src/model-picker/apply-session-model-selection.ts` (466 lines)
3. `src/model-picker/apply-session-model-selection.test-support.ts` (152 lines)
4. `src/model-picker/apply-session-model-selection.test.ts` (565 lines)
5. `src/model-picker/apply-session-model-selection.operator-context.test.ts` (191 lines)
6. `src/model-picker/apply-session-model-selection-placement.test.ts` (152 lines)

## 2. Invariants & Implementation Details

- **Model Picker Capability Profiles**:
  - `ModelPickerCapabilityProfile` encapsulates presentation constraints (markdown support, max text length 4096), callback limits (1024 units, UTF-8 bytes / UTF-16 units), and response interaction capabilities (`supportsEphemeral`, `supportsEdit`, `supportsReplace`).
- **Session Model Selection & Invariants**:
  - `SessionModelSelectionRequest` handles provider, model, explicit `resetToDefault` intent, `alias`, `profileOverride`, and runtime directives (`unchanged`, `clear`, `set`).
  - **Lock Enforcement**: If a session entry is marked `is_locked`, any selection request is immediately rejected with reason `locked` and message `"Model selection is locked for this session."`.
  - **Concurrency & Generation Fence**: Mismatched initial and current session generations trigger a `conflict` rejection with `"Model change was not applied because the session changed. Retry."`.
  - **Operator Authority**: Disallowed selections for operator role are rejected with reason `not-allowed` and `"Model <provider>/<model> is not available for this operator role and agent."`.
  - **Model Visibility Policy**: Disallowed models outside the agent policy allowlist are rejected with `"Model <provider>/<model> is not available for this agent."`.
  - **Active Cloud Worker Placement Fence**: When cloud-worker placement is active (non-local), selecting a runtime without execution mode compatibility is rejected with reason `invalid-runtime`:
    - If execution mode changes: `"Session cannot change cloud placement execution mode while placement is <state>."`.
    - If execution mode is missing: `"Session cannot select a runtime without cloud placement support while cloud worker placement is <state>."`.
  - **Authoritative State Transition**:
    - Default/reset clears `provider_override` and `model_override`.
    - Explicit override updates `provider_override` and `model_override`.
    - Profile overrides set `auth_profile_override`.
    - Any model or runtime change flags `live_model_switch_pending = true` and updates `updated_at`.
  - **Event Formatting**: Switch events formatted as `"Model switched to <alias> (<provider>/<model>)."` or `"Model switched to <provider>/<model>."`.

## 3. Rust Native Owner & Verification Evidence

- **Rust Implementation**:
  - `source/host/src/extensions/inference/model_picker_service.rs`
  - Registered in `source/host/src/extensions/inference/mod.rs`
- **Contract Tests**:
  - `source/host/tests/openclaw_model_picker_contract.rs`
    - `test_ocrm_model_picker_o01_capability_profile_and_event_formatting`
    - `test_ocrm_model_picker_o02_locked_and_conflict_rejections`
    - `test_ocrm_model_picker_o03_operator_and_visibility_policy`
    - `test_ocrm_model_picker_o04_placement_fences_and_successful_mutation`
- **Verification Result**: 4 passed; 0 failed.
