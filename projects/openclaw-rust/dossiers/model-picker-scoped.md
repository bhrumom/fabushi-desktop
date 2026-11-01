# OCRM Dossier: Model Picker Capabilities, Placement Fences & Session Selection

- **Unit ID**: `OCRM-UNIT-040`
- **Module ID**: `OCRM-M06` (`model-picker`)
- **Responsibility IDs**:
  - `OCRM-RESP-MODEL-PICKER-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `implementation=implemented`, `wiring=not-wired`, `verification=not-run`, `acceptance=pending`

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
- **Historical contract record**: the dossier records four contract tests as passed in an earlier run. That record is not current exact-head Actions evidence and does not establish production wiring or acceptance during the active development freeze.

## 4. Current production reachability audit (2026-10-31)

The current Host source exposes `SandAgentSessionStore::apply_session_model_selection`
and persists an applied result in the agent-scoped SQLite `sessionModelSelection`
key. A non-test caller search found no production session mutation route invoking
that method. Separately, `runner.startRoutedProvider` reads the persisted
selection and projects it into `SandSessionOptions`, so the read-side Runner /
Inference consumer edge is present. The missing authenticated mutation producer
keeps `BL-MODEL-PICKER-EXEC-WIRING-001` at `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`.

## 5. Fixed-source producer comparison and safe-wiring boundary

The pinned OpenClaw subject has two concrete production mutation paths for
this responsibility. The Telegram callback router resolves a callback against
the live provider/model catalog and invokes `applySessionModelSelection` at
`extensions/telegram/src/bot-handlers.callback-router.ts:546`; the general
auto-reply directive path invokes the same service at
`src/auto-reply/reply/get-reply-directives-apply.ts:355`. Both paths construct
their request from resolved catalog, default-model, session-entry and operator
context rather than accepting visibility or allowed-model policy from an
untrusted request body.

The Fabushi target currently has only the portable
`MessagePresentationAction::ModelPicker` schema/fallback renderer and the
Rust persistence/evaluation owners. The current gateway surface exposes
`sessions.*` and `chat.*`, but no model-selection mutation method; a non-test
search also found no caller for `SandAgentSessionStore::apply_session_model_selection`.
Therefore the model-picker envelope is not evidence of a usable UI or
production mutation path. A route that accepts `visibilityAllow` or
`operatorAllowedModels` from request parameters would violate the existing
policy owner and is explicitly out of scope. Wiring remains pending until a
canonical model catalog/operator-authority producer and an authenticated
session-action caller are available in existing owners.

The policy owner additionally requires an exact canonical-catalog match for
every non-default selection. A request that passes visibility and operator
allowlists but names a `(provider, model)` absent from the catalog must return
the existing `UnknownProvider` rejection and leave the session entry
unchanged. This is an owner-local safety invariant, not production ingress or
wiring evidence.

## 2026-10-31 current-source correction

The consumer portion of the preceding audit is superseded by the current
checkout. `start_routed_provider_task` reads the persisted entry at
`source/host/app/src/main.rs:7528-7530`, and the worker projects its
`model_override` and `thinking_level` into `SandSessionOptions` at
`source/host/app/src/main.rs:8424-8440`. The existing Runner/Inference
consumer edge is therefore confirmed.

The missing edge is the producer: no non-test caller of
`SandAgentSessionStore::apply_session_model_selection` was found. The model
picker and SQLite owners remain valid state/mutation owners, but no
authenticated session-action or gateway/UI mutation ingress reaches them.
`OCRM-RESP-MODEL-PICKER-001` and
`OCRM-RESP-SESSION-MODEL-OVERRIDES-001` remain `not-wired`; this correction
narrows the gap and does not promote verification or acceptance.

## 2026-11-01 exact ingress audit

The negative producer result was rechecked against the current checkout with
the following source boundaries:

- `source/host/app/src/main.rs::call_openclaw_gateway_command` dispatches
  `sessions.list/get/create/delete` and `chat.send/abort`, but no model-picker
  or session-model mutation method.
- `source/host/src/extensions/session/gateway.rs` owns session/profile/transcript
  gateway operations; its exposed mutation cases do not call
  `apply_session_model_selection`.
- `source/host/src/extensions/interactive/payload_service.rs` validates and
  fallback-renders `MessagePresentationAction::ModelPicker`, but has no action
  executor or authenticated session context.
- A repository search for
  `SandAgentSessionStore::apply_session_model_selection` and
  `apply_session_model_selection(` found only the owner definition and the
  inference implementation; contract tests are not production ingress.

This keeps `BL-MODEL-PICKER-EXEC-WIRING-001` and
`BL-SESSION-MODEL-MUTATION-WIRING-001` open. The safe next ingress must be an
existing authenticated session-action producer that derives catalog,
visibility, operator authority, placement, and generation context from their
canonical owners. Accepting those policy inputs from request JSON, or adding a
thin model-selection gateway route solely to reach the existing method, remains
out of scope under the ADR.

## 2026-11-01 exact-head evidence refresh

The preceding negative producer audit was revalidated against commit
`e4117e2bc0a6ce2e12f0b2641e0d45bf30e94837`, tree
`6a6c6f38cbbd453725a5e7f7814449dc0908462b`. The gateway dispatch at
`source/host/app/src/main.rs:3647-3658` exposes session listing, creation,
deletion, chat send/abort, and an authenticated-context-only `sessions.patch`,
but no model-selection mutation. The `sessions.patch` implementation at
`:4144-4255` accepts only archive lifecycle fields and calls the session
archive owner.

The sole non-test owner entry point remains
`source/host/src/extensions/session/agent_session.rs:266-287`; an exact
repository search for `apply_session_model_selection(` finds that definition,
the inference owner, and contract tests only. The model-picker presentation
validator at `source/host/src/extensions/interactive/payload_service.rs:271-277`
still has no action executor or authenticated session context. Therefore the
consumer edge is wired, while `BL-MODEL-PICKER-EXEC-WIRING-001` and
`BL-SESSION-MODEL-MUTATION-WIRING-001` remain `wiring=not-wired`,
`verification=not-run`, and `acceptance=pending`.
