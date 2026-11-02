# Cron session reaper

Status: implementation in progress; bounded registration slice approved
Project: OCRM-001
Responsibility: OCRM-RESP-CRON-SESSION-REAPER-001

## Context

OpenClaw's cron reaper removes expired per-run cron sessions and archives their
unreferenced transcripts. It preserves reusable base sessions, skips active
controller work and pending generated-media/subagent work, and uses the
existing guarded lifecycle owner when a session row has a session identity,
lifecycle revision, and update timestamp.

The current Fabushi Rust automation path records automation runs in the
agent-owned automation store but does not currently materialize or index
per-run cron session rows. `source/host/src/extensions/transcript/mod.rs` also
does not contain the previously recorded `TranscriptCleanup` owner. The old
ledger target and O05 oracle therefore do not establish an implementation
path.

The first implementation slice is now approved for D01--D03 only: scheduled
and event-triggered runs register and settle a durable per-run session row in
the existing agent database through `ProductionSessionWorkers`. This makes the
producer-to-owner-to-state edge explicit without claiming reaper deletion,
transcript archival, media/subagent preservation, or acceptance. D04--D05
remain an architecture and verification gate for the later cleanup slice.

## Goal

Define and implement one existing-owner Rust path for cron run-session
retention only after the per-run session representation, lifecycle authority,
transcript archive semantics, and scheduler/reaper trigger are explicitly
selected and approved.

## Non-goals

- Do not delete automation definitions or historical automation run records.
- Do not delete a session based only on filesystem age, automation-run age, or
  a stale projection.
- Do not add a second session database, cleanup owner, or unauthenticated
  deletion path.
- Do not treat the existing subagent cleanup wiring as cron reaper evidence.
- Do not claim production wiring, verification, or acceptance from this spec.

## Required design decisions

- CRON-REAPER-D01: identify the durable per-run session row/key and its owner;
  if Rust has no equivalent, specify the minimal migration before cleanup.
- CRON-REAPER-D02: define retention parsing, including `false`, invalid values,
  zero duration, default duration, and bounded sweep throttling.
- CRON-REAPER-D03: define the producer trigger and ownership fence for a sweep,
  including agent availability and current Gateway/lifecycle authority.
- CRON-REAPER-D04: define active media/subagent preservation and exact
  transcript artifact cleanup ownership.
- CRON-REAPER-D05: define guarded deletion fields and the changed/failed
  outcomes without retrying unknown deletion outcomes.

The bounded registration slice implements only the durable identity and
terminal-state portions of D01 and the producer/owner handoff in D03. It does
not select a cleanup trigger, perform retention parsing, archive transcript
artifacts, or delete rows. A read-only owner projection of these rows is part
of the same D01 slice so a future sweep cannot read `runs.json` as a substitute
for session state.

The per-run `cron_session_runs.session_key` is a cleanup/reaper projection
identity. Automation execution continues through the existing admitted main
Session owner until a real per-key session catalog and Runner/transcript owner
are introduced; the projection must not be mistaken for, or written into, the
owner-local admission record.

Terminal settlement must record the actual execution-completion timestamp in
the per-run row's `updated_at_ms`. The scheduled/fired timestamp remains an
automation-history fact and must not be reused as the retention anchor for a
run that completed later.

If the owner rejects registration after the compatibility automation history
run has been opened, the producer must settle that history run as `error` and
return the original registration error before invoking the automation. A
failed owner admission must not leave a permanently `running` history record
or execute work without a durable session projection.

The next approved slice is the pure D02 policy seam: retention parsing and a
per-owner five-minute throttle primitive. It has no scheduler caller and does
not decide whether a row is deletable; D03 trigger wiring and D04--D05 cleanup
remain pending.

The bounded D03 bridge may expose a read-only candidate-sweep request from the
existing `AutomationRuntime`. It must parse policy, apply the owner-scope
throttle, and delegate row selection to `ProductionSessionWorkers`; it must not
delete, archive, or create a second scheduler/cleanup owner.

The current-source caller recheck remains open. `run_session_maintenance` is a
startup-only task runner invoked while `start_session_extension` is constructing
the Session extension, before the Host binds the `TranscriptManager` automation
runtime. It is therefore not yet an approved caller for the D03 bridge. No
other recurring maintenance owner was found in the current Host automation,
session, or application startup paths. A future caller must be explicitly
selected and documented before wiring the bridge; generic startup maintenance
must not be treated as a cron scheduler by inference.

The following bounded read slice is also approved: the existing Session owner
may project expired, terminal, non-base rows while excluding active
controller/media/subagent fences. This is candidate selection only; it must not
archive transcript artifacts or delete a row until D04--D05 are approved.
Fence columns are fail-closed: until a real producer publishes a complete
fence snapshot for the row, `fence_state_known` is false and the row is not a
cleanup candidate.
Schema initialization must migrate an already-created registration table by
adding this marker with a default of false; migration errors remain explicit
and non-passing.

The next D04 slice may publish a typed fence snapshot at the existing Host
automation terminal boundary. It may read only the current admitted media
state, runner stream registry, and generated-subagent runtime through their
existing owners; any unavailable or mismatched fact keeps the row unknown.
The failure terminal outcome must use the same typed path so a known provider
failure can retain its row-specific fence snapshot; an unknown or mismatched
read still keeps the row unknown. Snapshot publication is not transcript
archival or deletion authorization.

When a complete typed snapshot is available, fence publication and terminal
state settlement must commit in one existing `SandAgentDb` transaction. This
prevents a future projection from observing a durable `fence_state_known` row
whose terminal outcome has not yet been committed. If the combined owner
transaction cannot commit, the run remains non-cleanup-authoritative and the
failure is explicit.

## Automation history acknowledgment — 2026-11-02

Contract: `OCRM-AUTOMATION-HISTORY-ACK-v1`. Base:
`76095c27539c2be57e3c1ba0e6b33b8158d92766`; fixed upstream U remains
`1440bb1771372334bea23d7642bb52deb7db1845`.
The integration parent is now `3aa679586d76e446f76a1c94700ff7abab623d42`:
its concurrent compaction continuation independently supplies the equivalent
ACK-07 SQL token repair and is preserved. ACK-06 must coexist with that owner's
admission/custody checks; neither branch's valid controls are removed.

The current `FileAutomationStore::finish_run_with` returns `Some` even when
the requested history row is absent. Its fail-soft projection reader can also
drop malformed rows before writing the remaining history. The existing
`AutomationRunPath` interprets `Some` as a committed settlement and may then
publish a terminal observation. This slice repairs that existing owner
boundary for both manual and background execution; it does not add an owner.

- ACK-01: both `finish_run` and `finish_run_definition` acknowledge settlement
  only after finding the requested row and successfully writing its terminal
  state. Missing configuration, missing history or a valid history without the
  requested ID returns `None`, without creating or changing history.
- ACK-02: settlement uses a private strict history read. Invalid JSON, a
  non-array root, any row rejected by the existing row parser, duplicate run
  IDs, invalid UTF-8 and non-NotFound I/O errors return an error without a
  write. Valid legacy row normalization and existing sort/history-window
  policy remain unchanged; projection reads and run admission are not changed.
- ACK-03: write failure remains an error. Known pre-write failure preserves
  history/configuration bytes; the existing filesystem writer's rename and
  crash guarantees are not expanded. No retry follows an unknown write result.
- ACK-04: the real `AutomationRuntime` → admitted Session store → run path →
  file store path propagates a failed acknowledgment and emits no terminal
  `AutomationRunObservation`. In-flight ownership is released, so a later
  independently admitted run can execute after storage is repaired. Successful
  runs still settle and report through the same owner.
- ACK-05: background cron-row settlement remains before compatibility history.
  A history failure may therefore leave a terminal cron row and incomplete
  history; this is explicit failure, not a cross-store transaction or durable
  recovery claim. The Host may still report a failed `FireCompletion` to its
  backend. Suppressed successful history telemetry does not mean no backend
  acknowledgment.

The immutable source review exposed two inherited prerequisites of ACK-04 and
ACK-05. The same batch therefore also repairs these existing owner boundaries:

- ACK-06: a background run must enter Spend Guard and execution without
  recursively acquiring the Session filesystem/deletion mutex. Guard reads,
  card/state mutations and automation-file changes retain existing admission,
  lifecycle and deletion protection. Use the existing Session/Spend Guard
  owners and one lock acquisition at each protected boundary; public facades
  that acquire the mutex must not be called inside an already-held boundary.
  Retain pause, nudge and widget-answer behavior. A bounded real Runtime
  control must observe execution or a causal refusal; a timeout is failure.
  The Session owner provides a crate-private `with_agent_admitted_db_mutation`
  callback with the already-open `&SandAgentDb`. Runtime retains its own
  automation mutation mutex; Spend Guard's apply and answer entries acquire
  the Session callback once. Private guard helpers use that DB owner instead
  of re-locking public facades. Legacy transcript append uses the existing
  locked append seam; a card is acknowledged only after actual insertion.
  A bound card whose ID already exists rolls back without changing guard
  state, unread/activity state, write generation or notifications. Successful
  atomic card publication releases the SQLite owner mutex before synchronous
  listeners, so a listener can read that same committed owner. A bounded real
  Runtime oracle covers both legacy/bound collision refusal and bound callback
  reads; no cross-store rollback of earlier routine disablement is implied.
- ACK-07: cron registration, plain and fenced terminal settlement, fence
  publication and projection must reach valid SQL in the existing `SandAgentDb`
  owner. Preserve all identity/lifecycle/running-state predicates, admission
  checks, BEGIN/COMMIT/ROLLBACK behavior and fail-closed fence semantics. Real
  database positive and stale/terminal/admission refusal controls remain required.

Implement the file-store boundary, these necessary caller/DB repairs and their
real caller oracles. Keep
terminal replay semantics, repeated execution, admission of pre-existing
corrupt history, pruning of running rows, concurrent/process locking, durable
reconciliation, delivery, reaper deletion and backend retry policy as open
work. No new schema, dependency, runtime or storage owner is required.

Exact-H Actions must cover both store entry points: missing/empty/no-match,
malformed and mixed history, duplicates, unreadable input, deterministic
pre-write failure, valid legacy normalization, successful persistence and
definition next-run projection. Production tests alter history after admission
inside the executor, assert no terminal observation and preserved failure,
then repair storage and prove a subsequent run can settle. Exercise manual,
schedule/event and execution-error/interruption branches through their existing
owners. Existing valid assertions remain. Source review and these checks do not
establish real backend delivery, crash recovery or full cron equivalence.

## Runner cancellation request and terminal publication — 2026-11-02

Contract: `OCRM-RUNNER-CANCEL-TERMINAL-v1`. Exact target base:
`d94aeb372c2fe4483bcf60ebbc5aaae4e6db52ef`; integration parent is
`58f1fb359c71e203cb82a6fe43cb52336f4542e5`, preserving the intervening channel,
strict checkpoint, MCP lifecycle and Plugin Skills snapshot changes. Fixed upstream U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. This bounded repair extends the
existing Host Runner composition and terminal projection; it adds no runtime,
registry, scheduler, durable state or event type.

The Gateway currently publishes `runner-inference/cancelled` immediately after
the registry accepts a cancellation token request. The provider task and routed
turn lease can still be active. Automation/background waits, Coordinator and
the outline bridge consume that event as execution terminal. The actual Runner
finalizer also re-reads cancellation after choosing its structured terminal,
so a late request can produce contradictory terminal facts for one turn.

- CANCEL-01: `runner.cancelRoutedProvider` retains its `{streamId,cancelled}`
  response, validation/default reason, first-request cancellation reason,
  superseded connector wake bookkeeping and interrupt telemetry. `cancelled`
  acknowledges a newly accepted token request only. Neither accepted,
  duplicate, missing-stream nor post-finalization requests publish a terminal
  event. Do not add `cancel_requested` to the existing event channel; current
  Coordinator parsing rejects unknown event types.
- CANCEL-02: after final Human handoff persistence has made its last possible
  change to the provider result, capture cancellation, result and delivery
  counters once. The immutable terminal snapshot uses the existing
  `project_turn_terminal` policy: waiting-user before cancellation before
  success/failure. Structured turn status, trace and the terminal event use
  that same frozen kind. A later accepted cancellation cannot rewrite frozen
  completed/failed/waiting facts.
- CANCEL-03: provider-review continuation settlement consumes the same frozen
  inputs while retaining its existing owner policy: cancellation first;
  otherwise waiting-user is explicitly `OutcomeUnknown`; otherwise completed
  or failed. This distinct continuation policy does not change the visible
  terminal priority or imply that waiting is a completed provider review.
- CANCEL-04: the existing finalizer is the sole routed task terminal publisher.
  Preserve activity, computer-use, provider, registry, permission, ack, routed
  lease and live-session cleanup ordering. Publish only after those existing
  boundaries, through a production seam that consumes the frozen event once.
  Retain `sentMessageCount` and `reacted` on every terminal type, empty content
  plus `waitingUser: true` for waiting, known failure details/trays/telemetry,
  and existing successful-run update behavior. Cancellation does not undo
  delivered messages or reactions.
- CANCEL-05: deterministic shipping tests use the real registry, event hub and
  routed lease owner with a channel/barrier held worker. Cancellation leaves
  active ownership and no terminal for the Automation parser; release produces
  one truthful terminal after retirement. Include duplicate/wrong/late-stream
  requests, cancellation after frozen success/failure, waiting priority,
  provider-review `OutcomeUnknown` and cancellation priority, known failure,
  delivery facts and nonterminal deltas. Source wiring checks may supplement
  these behavioral oracles, never replace them; retain all existing tests.

Implementation is confined to `source/host/app/src/main.rs`, this amendment,
the existing `runner_routed_provider_contract.rs` registry and
`turn_telemetry_production_wiring_contract.rs` telemetry oracles, and the
scoped workflow/receipt and current task record. No persisted format or public
event schema changes, migration or new dependencies are
required. Rollback is the bounded patch; a rollback also restores the known
premature-event defect. Existing telemetry remains the observation surface.

The separate Automation 30-minute deadline still fabricates an interrupted
execution terminal after requesting cancellation and remains open. Event-loss
and unknown-outcome durable custody, load/claim/settle reconciliation, restart
and backend redelivery policy also remain open. This patch does not establish
full cancellation, restart, cron/reaper or OCRM acceptance.

Definition of Done: the exact integrated H passes the selected shipping app
tests and registry/terminal/outline contract targets in GitHub Actions, with
independent review against CANCEL-01--05 and all pre-existing assertions
retained. Until that evidence exists, authored tests and source review are
implementation evidence only; U and all global acceptance flags are unchanged.

Provenance: the canonical Host Gateway/finalizer, TranscriptRunnerRegistry,
turn_runtime terminal projection and actual Automation/Coordinator/outline
consumers at the exact target base above, under OCRM-001 Revision 2 §17.
This target repair does not add an upstream full-file read or acceptance claim.

## Verification and Definition of Done

- A source-backed architecture decision names the existing owners and all
  producer → owner → durable row → consumer edges.
- Focused oracles cover disabled/zero retention, invalid fallback, expired
  deletion, base-session preservation, active media/subagent preservation,
  lifecycle change during sweep, and transcript cleanup failure.
- The exact upstream source identity and current Rust source identity are
  recorded; no old O05 auto-review test is reused as reaper evidence.
- Exact-head Actions, runtime, package, provider, and device evidence remain
  required before the responsibility is considered accepted.
- The bounded registration slice is complete only when the run path calls the
  owner before execution and settles the same guarded row after execution;
  verification remains pending during the development freeze.
- The D02 policy slice is complete only when disabled, zero, invalid, finite,
  and throttle outcomes are represented without reading filesystem or run age;
  verification remains pending during the development freeze.

## References

- `projects/openclaw-rust/dossiers/cron-automations-heartbeat-scoped.md`
- `projects/openclaw-rust/dossiers/upstream-current-session-lifecycle-delta-20261031.md`
- OpenClaw `src/cron/session-reaper.ts` at the current discovery subject
- `source/host/src/extensions/transcript/automation_run_path.rs`
- `source/host/src/extensions/transcript/automation_runtime.rs`
- `source/host/src/extensions/session/production.rs`
- `projects/openclaw-rust/decisions/2026-11-01-cron-session-reaper-owner-boundary.md`
