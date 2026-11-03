# Native Runtime provider-dispatch admission

Status: implementation in progress; source review and exact-H Actions required
Project: OCRM-001
Contract: `RDA-v1`
Authoring target: `bhrumom/fabushi-desktop@8c80af7505e7245f8d201943816082fdb8225a1d`
Fixed upstream: `1440bb1771372334bea23d7642bb52deb7db1845` (unchanged)

## Context and authority

The existing shipping Unified/FeatureHost constructs MahayanaRuntime. Its
accepted send creates a LogicalTurn/ExecutionRun, registers the per-conversation
actor and schedules the provider task. In `mahayana-runtime/src/lib.rs`, that
task calls the provider even when `actor.start` fails; it also discards errors
from the Preparing and Thinking transitions. Each transition persists through
the existing RuntimeStore and publishes the typed state event. The SQLite
setters additionally accept an UPDATE affecting zero rows as success.

Decision **RDA-01 — FIX / IMPL_BUG**, based on the actual target source, before
implementation. This is not a diagnosis of an Actions failure. A rejected owner
or failed durable transition cannot authorize a provider call. The existing
architecture contract is `docs/specs/post-1.2.74-agent-architecture-cutover.md`
R3/R4 and its canonical execution flow; OCRM-001 §17, AC05/08/17/18 and the
current recovery requirements remain authoritative. This correction introduces
no new upstream full-file-read claim and no source/acceptance flag promotion.

## Goal, scope and ownership

Close the accepted-send to provider-entry error path in the existing Rust
MahayanaRuntime/ConversationActor/RuntimeStore owners. A failed admission emits
one useful operation failure and retires this operation's volatile ownership,
without starting the provider. Healthy sends and actual provider failures keep
their existing behavior. React/Electron remain typed event consumers.

No new runtime, queue, database, persisted schema, generic fault injector or
production retry owner is introduced. Scope is the current native Runtime
library, existing SQLite state setters, and causal library tests. The existing
Actions workflow selects this package explicitly for the affected scoped batch.
Root integrates workflow/specification changes serially.

This does not certify existing accepted/queued rows as never executed, change
startup recovery policy, enable a provider-checkpoint loader, or close durable
original-turn claim/load/resume/reconciliation. Pre-acceptance registration/
recording rollback, arbitrary late provider events, final persistence after
provider execution, and different profiles sharing app data remain separate
boundaries. They must not be represented as resolved by this admission gate.

## Required behavior

- RDA-01: Under the existing per-conversation gate, require the actual actor's
  start operation to succeed before either state transition or provider entry.
  A refused/poisoned owner cannot be treated as a successful admission.
- RDA-02: Require Preparing and Thinking transitions, including their existing
  RuntimeStore writes and typed event publication, to succeed in order before
  calling `ConversationProvider::send_message`. On the first error, do not call
  the provider and do not continue to the next admission phase. Preserve the
  original failure cause rather than fabricating provider output or success.
- RDA-03: With a real SQLite connection, each existing turn/run state UPDATE
  must affect its one addressed row. Missing rows are errors, including a
  missing run after a turn write. Retain the explicit existing in-memory/wasm
  mode behavior; this is not a new durable guarantee for those modes. No new
  database schema or implicit reconstruction of missing rows is allowed.
- RDA-04: An accepted operation denied at this boundary uses the existing
  operation-failed event, with a distinct `runtime_admission_failed` code and
  useful original cause. It attempts existing failed-state bookkeeping, emits
  no operation-completed event, and retires only its own operation/context and
  actor run through the existing cleanup. A bookkeeping failure cannot turn
  refusal into provider entry or success. Preserve normal provider error codes,
  including usage-limit classification, and normal completion behavior.
- RDA-05: Keep conversation isolation and the actor gate. After a reversible
  persistence fault is removed, a later send in that conversation can run once;
  it cannot inherit the rejected operation's request, state or resource. Other
  conversations remain independently executable. Do not clear whole maps or
  reset unrelated owners to repair one refusal.
- RDA-06: Test actual Runtime/actor/store entry and observable outcomes. Use an
  isolated on-disk SQLite store with real conditional SQL refusal triggers and
  real missing-row cases, plus a counted provider boundary. Cover both state
  phases and both turn/run writes, distinguish zero provider entries from an
  error returned by a provider already entered, and prove the healthy/later-send
  controls. An actor-poison control may use a narrowly scoped cfg(test)-only
  helper for the real mutex; it must not add a production bypass or new owner.
  All original positive and negative assertions remain.

## Oracle and evidence

The test owns its Runtime, per-conversation gate, isolated temporary directory,
SQLite observer and counted provider. Hold the real gate to arrange the fault
after accepted persistence and before asynchronous dispatch, then release it.
Require the exact accepted operation's failure, no completed event, zero provider
entries, and retirement of its operation/context. Successful admission must
reach the counted provider exactly once with the correct operation/message.
Remove the conditional fault and require a later independent operation to
complete, retaining the original refusal evidence. Bound event/cleanup waits;
a timeout or cleanup uncertainty fails, and no broad process-name cleanup is
permitted. These controlled-provider contracts are not real service acceptance.

Only GitHub Actions may compile or execute. Reuse the existing Rust desktop
runtime workflow and committed lock; select the affected `mahayana-runtime`
library contracts plus shipping Host compilation and retained Session/Runner/
Automation/physical-custody controls. Record actual nonzero test execution and
classify failures from their sound oracle; source review alone is not verified.
Fresh independent general/structural source review and final exact-H Actions
are required before accepting this bounded correction. Full G01–G12,
AC01–AC24, source/recursive-dependency/legal closure, real providers/devices,
packages, migrations/rollback and independent ACCEPT remain unchanged.

## Migration, rollback and observability

There is no persisted format migration. The new error code uses the existing
string-valued operation failure envelope; consumers must be checked to retain
its ordinary generic failure handling. Rollback reintroduces provider execution
after failed admission. Preserve exact failing runtime/store cause in the
existing error path without adding credential or prompt logging. No automatic
retry, success receipt or durable original-turn recovery claim is inferred.

