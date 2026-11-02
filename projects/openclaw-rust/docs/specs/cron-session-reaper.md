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

### Strict history admission — AUTO-HISTORY-ADMISSION-v1

Decision **AHA-01 — FIX / IMPL_BUG**, before implementation on
`1501be999f195e7204da8b095a0ab64d5bc3a5aa`. The existing file-store settlement
reader validates the complete history, but `FileAutomationStore::begin_run`
still uses the fail-soft display reader. A new run can therefore overwrite
malformed, mixed or duplicate-ID history and enter execution after discarding
the prior evidence. The current run path also requires a returned history ID
only for background runs; a previously selected manual definition removed
before `begin_run` can reach its executor without a history record.

- AHA-01: reuse one private strict mutation reader in the existing
  `FileAutomationStore` for admission and both settlement projections. Validate
  the entire JSON image, every row with the existing parser and all unique
  run IDs before applying the existing sorting/history window. Invalid JSON,
  non-array roots, malformed/mixed rows, duplicate IDs, invalid UTF-8 and
  non-NotFound I/O must refuse before any history write. Do not salvage a subset
  or let a malformed row outside the display window escape validation.
- AHA-02: missing history remains a valid first-run input; missing definition
  remains `None` without creating a directory. Valid legacy normalization,
  current duplicate-return behavior, ordering and the current history-window
  policy remain unchanged. Ordinary listing remains a fail-soft display API.
  This does not establish durable original-run dedupe or retained-run pruning.
  Decision **AHA-EMPTY-ID-01 — FIX**: an explicitly empty new run ID must return
  `InvalidInput` before writing history, because the existing row parser rejects
  that very row on its next read. Add a raw-byte-preservation control; retain
  all other existing whitespace/legacy ID semantics without new normalization.
- AHA-03: every manual, schedule and event run must obtain a nonempty actual
  history identity before prompt execution or cron registration. Missing
  identity is a pre-dispatch error; preserve the existing background error
  contract. Refusal must publish no terminal run observation, enter no executor
  and release current in-flight ownership so a later independently admitted
  run can proceed after the storage input is repaired.
- AHA-04: keep existing Session filesystem/deletion admission, Runtime mutation
  ownership, Spend Guard, notice/lastRunAt bookkeeping order and ACK-01–07
  terminal settlement behavior. Earlier metadata bookkeeping is not rolled
  back by a later history refusal, and this is not a cross-file transaction.
  Do not add a second parser, store, scheduler, retry or permission authority.

Actions oracles use the actual file store and existing
`AutomationRuntime` → admitted Session adapter → run path. At the store,
retain raw malformed bytes and configuration while refusing admission, include
an invalid tail beyond the current window, and retain missing/valid/legacy/
duplicate positive controls. In the production path cover manual, schedule
and event inputs with malformed and ambiguous histories, zero executor calls,
zero new cron rows/observations and released occupancy; repair the input and
prove one later independent run reaches execution and the existing settlement.
Keep existing missing-background-ID and all ACK failure/next-run assertions;
add the manual missing-ID counterpart through the real run path/file store.
Use the existing bounded child/fixture ownership rather than new global state.

Acceptance requires independent source reviews and exact-H Actions in the
already selected store, run-path and production-wiring targets. This is a
target-owner corruption/admission correction, not an additional fixed-U
whole-file-reading claim. Durable pre-dispatch execution receipts, aliases,
sticky unknown effects, backend ACK suppression, original-turn recovery,
real service compatibility and full OCRM acceptance remain open. No stored
format, dependency, global acceptance flag or fixed upstream U changes.

### Automation oracle fixture custody — AUTO-FIXTURE-CUSTODY-v1

Decision `AUTO-FIXTURE-CUSTODY-01 — FIX / TEST_FLAW`, before implementation.
The failed subject is `007c07f7a84f1dee106ae40734369b6259d8f201`, Runtime
`41439491960`, job `125258277125`, attempt 1. The actual
`real_background_history_failures_preserve_terminal_owner_rows_and_allow_a_new_run`
child failed while creating its Session, before Automation history assertions,
with `agent store filesystem error: Invalid argument (os error 22)`; the parent
then reported that same child failure. The error prefix identifies the
`init_configured_db` parent-directory `create_dir_all` call. Recovery can reenter
that helper, so this does not establish that no earlier SQLite open was tried.
It also does not identify the individual kernel operation or why it failed.

Independent fixed-H source assessment found a separate, definite fixture
ownership defect: parallel parent cases name their root using only the same
process ID and a wall-clock timestamp, and `create_dir_all` admits an existing
root. Equal clock values can therefore give two child guards the same cleanup
root. The log has no explicit PID-to-path mapping, so this failure is **not**
claimed as a demonstrated collision or an environment-only fault. Preserve
007c's failed outcome and all its positive results; do not repeat the unchanged
instrument until it happens to pass. The implementation base is now
`2c32127539bba5e797eaef89a02096bb2cb24f5d`, preserving its concurrent Session,
credential-context and Skills fixes. Fixed U is unchanged.

- FIXTURE-01: reserve every fixture leaf with exclusive `create_dir`. Add a
  checked process-local sequence to the existing process/time identity, so
  concurrent callers at one fixed clock value receive distinct paths. An
  already-existing candidate is a refusal, never ownership of that directory;
  do not remove or alter its sentinel. No project dependency is added.
- FIXTURE-02: retain the actual bounded child, parallel parent cases, 45-second
  timeout, exact selected test and non-empty completion receipt. Only that
  child's guard can stop its process. Remove its root only after successful
  exit and a matching completion receipt. Failed or timed-out cases retain
  their owned root; an unwinding real Automation fixture still disposes its
  runtime and shuts down workers before retaining files. Do not kill by name,
  share a cleanup root or treat a timeout as approval.
- FIXTURE-03: print explicit fixture-root, case and OS process identities at
  allocation/launch and preserve failure artifacts in a dedicated per-run,
  per-attempt Actions directory. The recovery step alone receives this fixture
  destination, and failure upload is confined to that destination. No user
  directories, credentials, environment dump or unrelated `/tmp` tree enter
  the artifact. Production materialization rollback may remove its failed
  UUID directory; the retained fixture is therefore not a pre-rollback or
  syscall trace. Do not add synchronous production diagnostics under the
  Session mutation locks merely to capture this test failure.
- FIXTURE-04: use a fixed-time concurrent allocation control, an occupied-root
  refusal with byte-preserved sentinel, and deletion of one owned sibling
  while the other remains usable. Keep every existing real DB, history,
  terminal-row, admission, callback, subsequent-run and no-replay assertion.
  The existing `automation_run_production_wiring_contract` target remains
  selected by the existing scoped workflow; do not serialize it or skip its
  failing case. Required proof is the new controls plus all original cases
  and the shared gates on the resulting exact H in Actions.

The change is confined to this contract, the existing Automation test fixture
and its scoped workflow evidence path. No production retry, Session/Automation
semantics, schema, source lock, ledger flag or final acceptance requirement
changes. Freeze and independently review the complete scope before delivery.
Rollback restores the old fixture isolation weakness. A later filesystem
failure remains non-passing and must be assessed from its new owned-path
evidence; neither this repair nor a later successful run rewrites the original
EINVAL cause as proven. Real backend and full recovery acceptance stay open.

Decision `AUTO-FIXTURE-EVIDENCE-02 — FIX / TEST_FLAW`, before the follow-up:
materialization rollback can leave only empty fixture directories, which a
file artifact cannot preserve. Each exclusively claimed leaf therefore writes
a non-sensitive allocation record with its process ID, case and root before
owner work starts. This is allocation evidence only, never a completion
receipt. The existing occupied-root and sibling controls also preserve that
record's bytes and prove allocation creates no `completed` receipt. Failed
fixtures can then retain useful metadata even when their Session UUID directory
was removed; the original pre-rollback/syscall evidence limit still applies.

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
- CANCEL-01a: preserve the actual `requestSource=connector` identity through
  `classify_send_dispatch` and routed lease admission, so superseded channel
  work reaches the existing DM-preempted wake/redrive consumer. Keep its
  established `RunLane::User` priority, existing trusted Session selection,
  other known sources and unknown-source fallback. A runtime admission/source/
  settlement oracle must accompany the unchanged shipping cancellation test.
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

That cancellation-publication patch leaves the separate Automation 30-minute
deadline's fabricated interrupted execution terminal open. The bounded deadline
correction below addresses that in-process wait only. Event-loss and
unknown-outcome durable custody, load/claim/settle reconciliation, restart and
backend redelivery policy remain open. Neither patch establishes full
cancellation, restart, cron/reaper or OCRM acceptance.

Definition of Done: the exact integrated H passes the selected shipping app
tests and registry/terminal/outline contract targets in GitHub Actions, with
independent review against CANCEL-01--05 and all pre-existing assertions
retained. Until that evidence exists, authored tests and source review are
implementation evidence only; U and all global acceptance flags are unchanged.

Provenance: the canonical Host Gateway/finalizer, TranscriptRunnerRegistry,
turn_runtime terminal projection and actual Automation/Coordinator/outline
consumers at the exact target base above, under OCRM-001 Revision 2 §17.
This target repair does not add an upstream full-file read or acceptance claim.

### CANCEL-06 — Connector source recovery after exact-H failure

Decision `CANCEL-SOURCE-01`: **FIX / IMPL_BUG**, recorded before implementation.
Runtime run `41411715447`, job `125166533584`, on exact H
`b3a3fd398d3057b264ac673fae1b352aaecf1c92` failed the retained
`routed_provider_cancel_request_waits_for_worker_terminal` wake-marker
assertion. The worker's later `Disconnected` panic is secondary to the parent
assertion dropping its release sender. Independent direct source/log assessment
confirmed the real channel producer passes `connector` through
`background_revival_admission_args` into `accept_routed_send`, but
`classify_send_dispatch` projects it to the default `turn`. The scheduler then
cannot report `connector` to the existing cancellation/wake owner. This is not
a missing fixture seed, environment fault or permission to weaken the oracle.

Repair only the existing classifier's exact `connector` source projection;
retain its User lane, every other recognized source and unknown-source `turn`
fallback. The shipping fixture must use the real admission-argument helper and
assert actual source/lane before cancellation. Retain every held-worker,
registry/lease, terminal, telemetry and delivery assertion. Add ordinary and
unknown-source superseding controls plus a nonsuperseding connector cancellation
control: none may receive the connector preemption marker. The real routed
admission contract must independently observe source and lane before settlement.

Scope extends to `extensions/transcript/production_runtime.rs`, the existing
`turn_runtime_recovery_contract.rs`, shipping app tests and existing scoped
workflow/receipt. Concurrent commit
`4fe16f4689e23da2f027d09b2e8d39f56755c6ec` supplied the equivalent production
repair and real-admission oracle before this shipping control batch was
published. Adopt that exact implementation and oracle; do not duplicate them.
This successor adds the real shipping helper/preconditions and cancellation
negatives. No new owner, state/schema, lane policy or dependency is introduced.
U and all global flags stay unchanged; concurrent Host-lock, canonical-root and
compaction changes are preserved. Reverting the controls does not revert the
preceding production fix; reverting that fix restores the lost-source defect.

The b3 Host build, Settings/inference checks, app test compilation, other eight
selected app cases and renderer's fifteen tests remain positive evidence for
b3 only. Its affected integration recovery step did not execute. Recovery must
run the repaired shipping assertion and source controls at the new exact H,
then the previously unreached shared contracts. Do not rerun unchanged b3 or
label queued/cancelled successor runs passing. A further repeated signature
requires a fresh independent causal/exhaustion assessment under root AGENTS.

### Real-owner fixture failure capture — ACK-FIXTURE-DIAGNOSTIC-01

Exact `007c07f7a84f1dee106ae40734369b6259d8f201`, run `41439491960`,
job `125258277125` failed while the background-history case created its Session,
before injecting a history fault. The error identifies `AgentDbRecoveryError::Io`
from `init_configured_db` creating the database parent directory (EINVAL), but
the log omits the scenario and private fixture path/state. Cause remains unknown;
neither a production defect nor an environment exemption is established.

Decision `ACK-FIXTURE-DIAGNOSTIC-01 — FIX / TEST_FLAW` applies only to that
insufficient evidence capture. Retain the exact real-owner creation and every
history, cron, ownership, observation and independent-next-run assertion. On a
setup failure report the selected trigger/fault and metadata/error classes for
the owned fixture root and agents directory. Do not retry creation, change the
storage location, substitute an in-memory owner, or classify a later pass as
proof of the original failure's cause. Keep bounded child custody and cleanup.

## Automation deadline and terminal supervision — 2026-11-02

Contract: `AUTOMATION-DEADLINE-TERMINAL-v1`. Exact target base:
`f2f2ba00d66b3db79f9cd46690b3ce3737fe296b`; fixed upstream U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. This target-owner correction
preserves the shipped history-acknowledgment and Runner terminal-publication
contracts above. It adds no upstream source-read or acceptance claim.

The Host's `run_local_automation_turn` currently returns `Interrupted` when
its 30-minute deadline requests cancellation. The request does not prove that
the registered Runner or its routed lease has ended. That return nevertheless
allows `AutomationRunPath` to settle the guarded cron row and strict history,
publish a terminal observation, release its in-flight key, and return a backend
completion. It can also replace an already-queued genuine success with an
interruption. Repair the existing Host waiter rather than changing these
downstream owners or weakening their settlement acknowledgments.

- DEADLINE-01: the existing deadline requests cancellation at most once for
  the same stream, with the existing timeout reason. Cancellation acceptance,
  rejection or an already-cancelled token is not an execution terminal. Keep
  the original event subscription, synchronous execution waiter and Automation
  in-flight/history ownership until a matching genuine Runner terminal arrives.
- DEADLINE-02: check the event receiver before applying deadline cancellation;
  a matching terminal from that receive wins. Only the existing
  stream/channel-qualified terminal mapper
  determines completed, failed or interrupted execution. Preserve waiting-user
  projection, upgrade-quiescing annotation and final fence attachment. A frozen
  or queued terminal cannot be rewritten by the deadline.
- DEADLINE-03: each receive remains bounded by the existing 250 ms polling
  interval. Before expiry it may shorten to the remaining deadline; after the
  single cancellation request it uses a positive polling interval, avoiding
  a zero-timeout busy loop. Continued supervision until the actual terminal is
  deliberate: the 30-minute boundary is a cancellation deadline, not a second
  terminal authority or a hard limit on this synchronous wait.
- DEADLINE-04: extract a private waiter/clock seam used by the shipping path.
  Deterministic tests reuse the real registry, event hub, routed lease and
  terminal snapshot/finalizer seams. A channel-held worker must remain owned
  after the waiter crosses the deadline; nonterminal or foreign-stream events
  cannot settle it. Release publishes the actual terminal after existing owner
  retirement. A queued-success-at-deadline control must preserve success without
  requesting cancellation. Retain all pre-existing cancellation/terminal tests.
- DEADLINE-05: exercise the same waiter inside the real `AutomationRuntime` /
  admitted Session store / `AutomationRunPath` boundary. While the worker is
  held, history and the cron row remain running, the in-flight key remains
  claimed, a duplicate scheduled admission cannot execute, and no terminal
  observation or backend-completion projection is produced. After the genuine
  terminal, strict history and cron settlement,
  observation, in-flight release and the existing completion mapping proceed
  normally. This is an in-process ownership oracle, not real backend delivery
  or restart acceptance.

Implementation is confined to the existing Host application and tests in
`source/host/app/src/main.rs`, this spec, and the parent-owned scoped Actions
selection/evidence record. No persisted format, public outcome, event schema,
permission, dependency or runtime owner changes are required. Existing Runner
finalizer ordering, cancellation policy, queue admission and row fences remain
authoritative. Rollback is the bounded patch and restores the premature deadline
settlement defect; there is no data migration.

Channel disconnection still follows the existing error return, and Host loss
can still leave durable running/unknown state. Event-loss reconciliation,
durable unknown-outcome custody, load/claim/settle recovery and backend redelivery
remain open. A Runner that never publishes its real terminal keeps this waiter
supervising; this patch does not add a recovery policy or synthesize completion
to release that ownership. Those limits must remain explicit in delivery claims.

Definition of Done: independent source/spec review and exact-integrated-H
GitHub Actions results for the new shipping waiter oracles and retained
Automation/history/Runner contracts. Authored tests are not execution evidence;
no local executable validation is permitted, and all global acceptance flags
and U remain unchanged pending their own gates. The existing canonical Host,
Automation and Session owners at the exact target base provide source authority
for this narrow repair; unavailable real backend/service acceptance stays
separate and cannot be promoted by these local-owner oracles.

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

## Known unfinished backend-run redelivery — 2026-11-02

Contract: `OCRM-AUTOMATION-KNOWN-RUN-REDELIVERY-v1`. Decision
**AUTO-REDELIVERY-01 — FIX / IMPL_BUG**, specified from `f3b8105864a0e215e7918311b2d39cb789e350a4` and revalidated
before implementation on `73cf0269c5ccbbd1dff946e0949824a1bbbeffd4`. Fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`.

The existing delivery lookup finds an original UUID or coalesced alias in
persisted history, but returns the same absence value for a matching `running`
row and for no matching row. A fresh Host can therefore admit another event or
revisioned schedule execution. Legacy covered-slot, disabled-definition and
changed-definition checks can instead fabricate a failed completion for that
unfinished original. Existing volatile in-flight keys, per-run cron projections
and terminal supervision do not repair this delivery decision after reopen.

- REDELIVERY-01: distinguish absent, known unfinished and known terminal
  history in the existing `fire_delivery` owner. A matching `running` original
  UUID or persisted coalesced alias returns the existing
  `PreparedBackendFire::Abandon` with reason `existing_run_unsettled`.
- REDELIVERY-02: keep this decision before enabled, legacy-slot, definition
  revision, event and trigger checks. The shipping Host's existing refusal
  path returns false without invoking completion or spawning an execution
  worker. The backend owner removes only its volatile poll entry; it may
  re-evaluate a later delivery but must not report completion or include this
  UUID in `ackRunUuids` based on the unfinished row.
- REDELIVERY-03: preparation does not begin, refresh, finish or rewrite history
  or cron state and emits no run observation. Preserve the actual file bytes
  and unfinished row. A different original UUID retains existing eligibility;
  this is not an automation-wide execution fence.
- REDELIVERY-04: retain stored terminal-success/failure replay, schedule/event
  validation, the completion-error retention contract and ACK-04/05. A later
  explicit terminal history settlement can be reported by the existing path
  without executing that original again. Do not reinterpret a terminal error
  as proof that every external effect was reconciled.
- REDELIVERY-05: test original and coalesced IDs across valid event/revisioned
  schedule, legacy covered-slot, disabled and changed-definition cases. Use
  actual reopened Session/Automation file owners and the actual backend poll
  consumer with controlled transport; verify unchanged bytes, zero execution,
  zero completion requests and no ACK on repeated polling and fresh backend
  owner construction. Preserve distinct-ID and terminal-history positive
  controls. Owner reopen is not a claim of full Host process recovery.

Production scope is `source/host/src/extensions/automations/fire_delivery.rs`;
its existing Host and backend consumers remain wired. Tests belong to existing
Automation contract targets. The integrator owns this spec and scoped workflow
selection/receipt. No new store, schema, scheduler, registry, transport protocol
or Session admission rule is authorized. Source authority is the current actual
target call chain, with the fixed-U receipt/queue/finalization review retained
in the existing cron dossier; this does not promote a full-source read claim.

Missing, corrupt, pruned or ambiguously duplicated history, lenient projection
normalization, durable execution claims, unknown effects, account changes,
cross-store reconciliation and actual backend-service semantics remain open.
The batch proves refusal only from positively visible unfinished history.
Rollback removes this interpretation guard and restores the replay/false-ACK
risk; no data migration is involved. Completion requires independent immutable
source/spec review and final exact-H Actions for delivery, actual backend,
history/cron and retained deadline controls. U, OCRM responsibility acceptance
and all eight global flags remain unchanged. No local validation is permitted.

This interpretation guard is independent of the pending Session JSON precision
repair: its authority is positively present string UUID/alias and `running`
status, before the existing numeric slot comparison. Existing shared execution
at `fdfdfb2d6a5635fc9c79da51b9fc4b09486b553a` passed its selected owner contracts;
this new guard still requires its own final-H gates. Implement it in parallel
while the precision baseline waits in Actions, keep it out of that frozen
baseline commit, and do not infer permission to settle unknown tool outcomes.
The separate frozen terminal-custody consumer and original-turn recovery remain
required; this slice alone does not close Automation settlement.
