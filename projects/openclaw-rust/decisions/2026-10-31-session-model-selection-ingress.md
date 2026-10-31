# ADR: Session model-selection mutation ingress boundary

Status: accepted boundary; implementation deferred pending an existing authenticated producer
Date: 2026-10-31
Scope: Fabushi Desktop Host session/model-picker integration
Responsibilities: `OCRM-RESP-MODEL-PICKER-001`, `OCRM-RESP-SESSION-MODEL-OVERRIDES-001`

## Context

Fabushi already has two valid owner/state slices:

- `source/host/src/extensions/inference/model_picker_service.rs` validates
  model visibility, operator authority, placement/runtime compatibility and
  generation fences.
- `source/host/src/extensions/session/agent_session.rs` persists an applied
  result as the agent-scoped `sessionModelSelection` SQLite KV.

The production turn consumer is also present: `runner.startRoutedProvider`
reads the persisted entry at `source/host/app/src/main.rs:6434-6436` and
projects its model override and thinking level into `SandSessionOptions` at
`:7296-7312`.

What is missing is the mutation producer. A non-test search found no caller of
`SandAgentSessionStore::apply_session_model_selection`. The public Gateway
surface has `sessions.list/get/create/delete`, but no authenticated
model-selection mutation method. The portable `ModelPicker` presentation
shape is not an action ingress and does not supply the catalog, visibility,
operator, placement or generation inputs required by the policy owner.

## Decision

Do not add a new model-selection Gateway method or accept policy inputs from an
untrusted request body during this phase. The canonical producer must be an
existing authenticated session action owner that can obtain all required
policy inputs and pass them to `apply_session_model_selection` without
creating a second model catalog, operator-authority store, session store or
placement owner.

Until that producer is identified and approved:

- `model_picker_service` remains the policy owner;
- `SandAgentSessionStore` remains the sole persistence owner;
- `runner.startRoutedProvider` remains the sole turn consumer of persisted
  session selection;
- both parity rows remain `wiring=not-wired`, `verification=not-run`, and
  `acceptance=pending`.

## Required producer contract

Any future implementation must provide, in one traceable call chain:

`authenticated session action -> canonical model catalog/operator authority ->
model_picker_service -> SandAgentSessionStore/SandAgentDb ->
runner.startRoutedProvider -> ProductionInferenceExtension`.

The producer must preserve session identity, visibility and operator policy,
placement/runtime compatibility, generation conflict handling, reset-to-default
semantics and the negative invariant that agent defaults are not mutated.

## Rejected alternatives

- Adding a thin `sessions.setModel` route that trusts `provider`, `model`,
  `visibilityAllow` or `operatorAllowedModels` from request arguments.
- Reusing `cloud_agents.list_models` as the global model catalog.
- Treating the `ModelPicker` presentation envelope or contract fixtures as a
  production action producer.
- Writing directly to `sessionModelSelection` outside
  `SandAgentSessionStore`.

## Verification and acceptance

Static evidence currently proves the owner/state and consumer edges only.
Focused tests and frozen Actions evidence must be added after the real
producer exists; no local OCRM build/test or CI is run during the development
freeze. The ADR does not promote any completion flag.

## References

- `projects/openclaw-rust/dossiers/model-picker-scoped.md`
- `projects/openclaw-rust/dossiers/sessions-admission-scoped.md`
- `projects/openclaw-rust/dossiers/ocrm-closure-state-audit-20261031.md`
- `source/host/src/extensions/inference/model_picker_service.rs`
- `source/host/src/extensions/session/agent_session.rs`
- `source/host/app/src/main.rs`
