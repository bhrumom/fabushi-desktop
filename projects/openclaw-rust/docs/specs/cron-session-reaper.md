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
  **ACK-06-FIXTURE-01 — FIX / TEST_FLAW (2026-11-02):** exact
  `09df841142a378436c490c38b375ac842ac3f419`, Runtime `41513854566`,
  Host job `125512243925` failed while `prepare_away_agent` marked unread
  activity, before the card assertions, because the expected Agent DB path
  was not recognized as a regular file. The preserved command artifact contains
  no fixture-root identity; the actual cause of that unavailable path is not
  established. Source inspection does
  establish that the two bounded parent tests can choose the same PID/timestamp
  path, accept it through non-exclusive `create_dir_all`, and each later remove
  the shared tree. Repair this invalid ownership assumption without changing
  any Spend Guard, Session, insertion, rollback or listener assertion. An owned
  child root is acquired only by successful exclusive directory creation;
  collision chooses another candidate and never adopts, writes or deletes the
  existing candidate. A deterministic same-candidate control must keep a live
  sibling's contents intact through the other fixture's cleanup. Keep the exact
  child exit/kill-and-wait boundary before cleanup and the completion receipt;
  a child selecting no test remains failure. Include case, parent/child and root
  identities in diagnostics, retain failed owned roots under the existing
  `FABUSHI_AUTOMATION_CONTRACT_ARTIFACT_ROOT` when configured, and clean successful
  owned fixtures. Reuse the already-selected whole contract target in Actions.
  The later successful `3c4bf6963cb30ecf01eb34b905315074886e1972` run does not
  erase this earlier failure or prove its hypothesized collision occurred.
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

### Recovery authority prerequisite — AUTO-RECOVERY-AVAILABILITY-v1

Decision **ARA-01 — FIX / IMPL_BUG**, before implementation at
`be07ae46b199d712f689fb0b0a67594f327efcad`. Session corruption recovery salvages
kv/blobs/transcript but can reconstruct an empty cron table. That absence is
not evidence that earlier external effects settled. Extend the existing
`agent_db_recovery` owner; preserve healthy open and configured foreground
recovery, including salvage and reindex. This necessary public contract does
not by itself close Automation execution/settlement or original-turn recovery.

- ARA-01: retain a stable adjacent `<db-path>.automation-custody.guard` file.
  Recovery acquires its exclusive fs2 lock before destructive work. A checked
  availability probe holds its shared lock, validates the supplied existing
  connection/path and rejects any filesystem or KV unavailability marker.
  Use try-lock; propagate contention/errors and never unlink the guard in
  normal operation. Availability is not a run claim or permission grant.
- ARA-02: before quarantine, create a permanent adjacent
  `<db-path>.automation-custody-unavailable` marker, or safely reopen/resync an
  existing regular marker without changing its bytes. Missing/invalid parents,
  symlink/reparse/nonregular objects, including every existing SQLite sidecar,
  and real open/write/sync failures refuse
  before moving or deleting the DB/sidecars. Leave partial markers on error.
  Any marker contents, including empty/malformed, deny checked availability;
  no automatic clearing or timeout release is added.
- ARA-03: Linux checks marker `sync_all` and parent-directory `fsync`; macOS
  additionally syncs the marker after parent metadata submission; Windows
  uses a writable `WRITE_THROUGH | OPEN_REPARSE_POINT` marker and checked
  `sync_all`. Preserve recovery when these real APIs succeed. Do not add a
  blanket Windows failure or ignore an actual unsupported/error result.
- ARA-04: configure the reconstructed DB, then publish schema/migrations and
  `automationCustodyAuthorityUnavailable` KV together in one immediate
  transaction before salvage. Salvage cannot overwrite that marker. The
  sibling marker also covers a concurrent ordinary opener's empty-table view
  and old handles. Successfully unlock the new filesystem guard explicitly
  before external recovery callbacks/reindex; unlock failure is an error and
  cannot publish recovery success. The private shared/exclusive lock resource
  also attempts explicit unlock on Drop, without unlinking the stable guard.
  A duplicate reference must not keep a successful recovery's lock active.
  Keep callback-failure and salvage semantics unchanged.
- ARA-05: existing configured-recovery and kv/blob/transcript assertions stay.
  Add real-owner checks for healthy availability, required marker refusal,
  repeated recovery over valid/empty/malformed markers, wrong connection,
  real lock contention, malformed main/guard/marker/sidecar path objects, and original-byte preservation
  on refusal. A bounded owned child must demonstrate cross-process exclusion
  and actual exit before recovery admission. New availability APIs must reject
  the KV marker even if only that copy survives. Actions executes the existing
  recovery target and shared affected gates; also select private
  `agent_db_recovery::physical_release_tests` for actual Unix descriptor
  duplication at shared-guard Drop and exclusive recovery completion. These
  Unix controls do not claim Windows duplicate-handle equivalence. Source
  inspection is not a pass. ARA-MAIN-ORACLE-01 is **FIX / TEST_FLAW**:
  direct main-path refusal must exercise an actual directory and, on Unix,
  an existing-target and dangling symlink through `recover_corrupt_store_db`.
  Require the original path, target/sentinel and sidecar bytes to remain,
  with no guard, permanent marker or quarantine created. This supplements
  the existing valid recovery and other malformed-object assertions without
  changing production behavior or claiming execution before exact-H Actions.
- ARA-06 (ARA-PATH-01): preflight and the existing SQLite quarantine/removal
  consumer must address the same native path bytes. Derive sidecars and the
  default quarantine name by appending to `OsString`, never from `display()`.
  Preserve the existing utility's rename/copy/salvage and retry policies.
  Real Unix byte-path controls must leave independently named U+FFFD twins and
  their sentinels untouched through quarantine, sidecar cleanup and Session
  reconstruction. Cover a non-UTF-8 filename and a non-UTF-8 parent. Select the
  existing `host_storage_contract` and `conversation_blob_db_recovery_contract`
  alongside Session recovery in Actions. This narrowly extends the batch to
  `storage/sqlite_recovery.rs` and its existing storage contract; it does not
  claim every other owner's path construction or whole-platform acceptance.
  ARA-PATH-PLATFORM-01: run the five new raw-byte oracles on Linux in the
  existing runtime workflow, requiring their actual named positive results.
  They require invalid-UTF-8 entries; the
  [APFS creation contract](https://developer.apple.com/library/archive/documentation/FileManagement/Conceptual/APFS_Guide/FAQ/FAQ.html)
  does not supply that fixture prerequisite. Keep ordinary macOS shared-owner
  recovery tests. No runtime skip on a failed file creation, empty test filter,
  or platform inference supplies acceptance; retain actual exits, logs and
  failed owned fixtures, and keep all other platform obligations open.
- ARA-07 (ARA-PATH-02): checked availability must also admit a healthy main
  database whose Unix filename or parent contains non-UTF-8 bytes. The pinned
  rusqlite 0.39.0 `Connection::path` projects SQLite's filename through UTF-8
  and returns `None` for that supported input; it cannot decide file identity.
  Read the existing connection's main-database filename without lossy text
  conversion, keep the existing canonical-path comparison, and retain the
  parent, physical guard and both marker checks. An attached matching file
  cannot authorize a different or in-memory main database. Add real healthy
  filename/parent positives, replacement-character twin/wrong-connection and
  attached-file negatives, then recheck successful authority and marker refusal.
  The existing Linux receipt must require these two additional named cases;
  retain all five earlier raw-byte controls and ordinary macOS owner checks.
  This corrects the existing availability contract only; it does not wire a
  durable Automation claim or turn a successful probe into execution permission.

This is a target-owner recovery correction, not a new fixed-U full-read claim.
The checked barrier/order, process-crash behavior and real filesystem/physical
power-loss acceptance are distinct. Current Rust 1.97.1 source at
`8bab26f4f68e0e26f0bb7960be334d5b520ea452` maps `sync_all` to Unix fsync,
Apple F_FULLFSYNC or Windows FlushFileBuffers; Microsoft documents write-through
metadata behavior for NTFS. Other filesystems/devices and all-platform actual
execution remain unaccepted. No mock or killed process establishes power-loss
equivalence. Existing marker resync permits foreground reconstruction only.
Original effects, unknown-result reconciliation, durable Automation claims,
terminal/report propagation and the actual complete/retry/ACK consumers still
require the following production batch. Older writers and external path/marker
replacement are not protected participants; rollback cannot discard markers
or treat reconstructed history as permission to replay.

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

## Automation settlement from frozen Runner custody — AUTO-CUSTODY-01

Contract: `OCRM-AUTOMATION-TERMINAL-CUSTODY-v1`, together with
`OCRM-AUTOMATION-KNOWN-RUN-REDELIVERY-v1`. Decision
**AUTO-CUSTODY-01 — FIX / IMPL_BUG**, assessed initially against
`800f166b8e30aa79e55435804dd514f915728332` and revalidated on
`4a2e43e6ce4e6948bccd25bce5eb7d9612a3f826` before implementation. Fixed U
remains `1440bb1771372334bea23d7642bb52deb7db1845`. Bind implementation and
execution evidence to the final immutable subject after the shared gate.

REC-11/12/13 publishes the actual Runner invocation observations in the existing
terminal's `toolEffectCustody` companion. The Automation-specific waiter drops
that event and the run owner treats every local terminal as settlement. A
cancelled or failed local Runner can therefore finish the history and cron row
and authorize backend completion/ACK while a tool outcome remains unknown.
The companion Running redelivery interpretation is already implemented at
`2c8a19cf6c036e799d92e50b979af414b7a43701` and retained in this base. Reuse
that existing guard; the missing boundary is the initial terminal consumer.
Current source authority is the actual Host finalizer/waiter, typed Runner
custody, AutomationRuntime/run path, Session/DB/file owners and backend callback.
This target-owner correction adds no upstream full-file reading claim.

- ACU01: Interpret the existing typed companion only in
  `wait_for_automation_terminal`. Keep shared terminal recognition and the
  background-revival waiter behavior intact. Deserialize
  `ToolEffectCustodyReport`, bind it to the expected stream with its existing
  `for_terminal`, and use `requires_retention`. Missing, malformed, empty,
  foreign/blank-stream, zero-generation, Unavailable, pending or unknown
  evidence requires retention. Earlier uncertainty cannot be erased by a
  later clear observation. Do not infer custody from an error string or from
  a local cancelled/completed tag.
- ACU02: Carry a distinct internal retained-unsettled execution result through
  the existing `AutomationRunPath`, preserving its original local terminal
  value. This is a non-serialized Rust result extension, not a new wire or
  storage format. Before any cron finish/fence publication, history finish,
  failure-counter change or run observation, return `Ok(None)` for retention.
  Do not return an error or fabricate a failed outcome: those paths currently
  permit terminal settlement and backend completion. Preserve ordinary
  Completed/Interrupted/Failed and their fence semantics when evidence is safe.
- ACU03: The existing Runtime, event coalescer and production callback carry
  None to backend abandon. Preserve that path: no completion POST or poll ACK
  for the retained original or any coalesced alias. Release only the existing
  volatile execution key after the actual local terminal, as the current
  outer run owner already does. Keep the admitted Running history and cron row
  unchanged from their post-admission baseline; no new retention registry,
  scheduler, durable claim or store is introduced.
- ACU04: Apply the companion known-running redelivery contract before enabled,
  revision, legacy-slot, event or trigger checks. Positively present Running
  history for the original/coalesced UUID returns
  `Abandon { reason: "existing_run_unsettled" }`, without executing, writing,
  observing or completing. Existing terminal-history replay and a distinct
  eligible UUID retain their behavior. A mere missing/corrupt history is not
  a durable admission guarantee.
- ACU05: Known uncheckpointed work with no pending or unknown result remains
  settlement-eligible; retry eligibility is a different policy. A genuine
  pre-entry NotStarted observation may settle its existing refusal. An entered
  uninstrumented invocation remains Unavailable. Keep visible terminal content,
  cancellation/waiting precedence, delivery facts, upgrade annotation and
  once-only publication. The shared mapper cannot create the Automation-only
  retention result; any necessary defensive exhaustive arm in background
  settlement must not commit or release admission from that marker.

Actions oracles must combine the actual finalizer, Automation waiter, Runtime,
Session/file/DB owners, counted guarded tool and controlled backend transport.
Hold or lose an actual tool result, publish the real terminal, require unchanged
Running rows, no observation/completion/ACK and one effect; a late external
return must not settle or repeat it. Cover the typed evidence matrix and safe
success/failure/cancellation, genuine NotStarted, and the existing real guarded
uncheckpointed positive. Keep the original deadline and queued-terminal
assertions, providing qualified clear observations to their positive fixtures;
add missing/empty evidence as retention negatives. Preserve history-write and
completion-POST failure assertions.

Reopen actual Session/Automation owners and reconstruct the volatile backend;
repeated original and coalesced deliveries must remain unexecuted and unacked
with unchanged files and cron rows. Exercise the real event coalescer's None
broadcast and every actual production backend completion callback for the admitted
original and aliases. Assert the inner Runtime result is exactly `Ok(None)`
before the coalescer broadcasts; an error or disposal also becoming None is not
a valid retention oracle. Use the real in-flight coalescer batch as the causal
enqueue gate, without timing sleeps as an ordering oracle. A distinct UUID and stored terminal success/failure are positive controls.
The existing scoped workflow selects all affected targets and retains the
Runner custody and shared recovery controls; final exact-H Actions and frozen
independent review are mandatory.

Production scope is the existing Host Automation-specific waiter and
`extensions/transcript/automation_run_path.rs`. The already implemented
`extensions/automations/fire_delivery.rs` remains unchanged. Existing Host
app (`tests/automation_terminal_custody_tests.rs`, following its existing child-test
module placement) and `automation_run_production_wiring_contract.rs` test homes supply the
producer and real coalescer/backend/owner-reopen oracles. Root integrates Host, specification and
workflow changes serially. No dependency or persisted schema change is needed.

This slice begins with a matching actual terminal and positively present run
history. Event loss/disconnection, earlier dispatch errors, pruning/corruption,
cross-process/concurrent durable claims, physical remote termination, original
turn restoration and external-effect reconciliation remain open. Manual and
distinct-UUID invocations remain independent under current policy; this is not
an automation-wide uncertainty fence. Existing legacy terminal rows are not
rewritten or certified. Real backend/service/account acceptance, all remaining
product responsibilities and final OCRM gates remain required.

Rollback removes this interpretation and restores premature settlement/ACK of
unknown effects; there is no persisted format migration. Retained work remains
visible through the existing Running history/cron projection and the unchanged
local Runner terminal. It is not automatically retried or silently resolved.
The pre-existing reaper, event-loss, external reconciliation, real-service and
final acceptance requirements above remain mandatory.

### AUTO-CUSTODY-ORACLE-01 — upgrade already owns cancellation

Decision **FIX / TEST_FLAW** against H
`36e06f0fbfe58a0a3ce1852db11699446e7f4ed3`, Actions run `41465568818`,
job `125346503908`: the safe-outcome/upgrade test requests actual upgrade
quiesce, then incorrectly requires a second `cancel_stream` to be a first
cancellation. `request_quiesce_for_upgrade` already cancels registered tasks;
the cancellation owner returns false for repeats and preserves its first reason.

Keep the production owner unchanged. The fixture must prove its initial handle
is uncancelled, the non-upgrade explicit cancellation succeeds, and upgrade
quiesce itself cancels with the upgrade reason. A subsequent cancellation must
return false without changing that reason. Preserve all original terminal kind,
sent/reacted facts, safe outcomes, upgrade annotation, retained-unsettled and
real NotStarted assertions. The same exact app selector remains required in
Actions; this diagnosis and authored repair are not candidate execution evidence.

Cancellation alone must also retain the real registered task and admitted routed
lease; the fixture checks both owners before terminal publication. The other
two new app custody cases passed on the failing H; recovery and Coordinator
checks did not execute. The integrated repair still requires new exact-H Actions.

## Retained history authority — AUTO-RETAINED-HISTORY-v1

Decision **AUTO-RETAINED-HISTORY-01 — FIX / IMPL_BUG**, source-identified at
`18ef48cf1b72c045141f56b227be79226056113b` and revalidated before edits on
`85fcd3d3034d908029a6bc3770755f90f3bc1e63`. No executed failing baseline is
claimed. Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`.

The actual retained-unsettled path leaves Running file history and cron rows.
Twenty later successful manual runs can evict that original from the current
20-row mutation/display window. A valid revisioned backend redelivery then
appears absent; the existing legitimate same-identity Running cron refresh
allows another executor entry. Existing owners must retain the unsettled fact
and consult its complete checked history before delivery decisions. This closes
the pruning/projection gap explicitly left open above, without changing cron
refresh semantics or introducing another registry, store or execution owner.

- ARH01: The existing strict history reader validates the entire array and
  unique primary IDs, preserving all current legacy normalization and errors,
  and returns every valid row before exact lookup. Ordinary fail-soft display
  reads remain newest-first and bounded by `AUTOMATION_MAX_RUN_HISTORY` (20).
- ARH02: Existing-ID begin remains idempotent without a write, before capacity
  checks. A new run refuses when at least 20 existing rows are Running; preserve
  their bytes and enter no cron/executor. Otherwise retain every Running row,
  including a backdated new run, and use remaining slots for newest terminal
  rows only. Do not stop independent work merely because one run is retained.
- ARH03: Settlement finds the exact primary ID in complete checked history.
  Preserve every remaining Running row and pin the just-settled target while
  retaining newest other terminal rows within the capacity. Return a bounded
  `AutomationRecord.runs` containing that exact committed target even when its
  old timestamp would put it outside the ordinary display. Preserve genuine
  absence/None, atomic-write errors and whole-input corruption refusal.
- ARH04: Valid legacy input may already contain more than 20 Running rows.
  Refuse new growth, keep authoritative lookup available, and permit non-growing
  settlement without deleting another Running row. If the protected set plus
  this exact terminal receipt still exceeds 20, preserve that set without other
  terminal history; compact back to 20 when it fits. Display and returned
  receipt projections stay bounded. This is draining existing excess, not a
  new unlimited capacity or silent quarantine.
- ARH05: Add checked original-or-coalesced-ID lookup to FileAutomationStore,
  returning one existing `AutomationRun`, true checked absence, or an error.
  Multiple matching rows are ambiguous and must error; repeated aliases within
  one row are one owner. Reuse the existing strict parser and safe folder-ID
  boundary. Missing history keeps existing first-run semantics. Do not use a
  display projection or a swallowed read error to establish absence.
- ARH06: Expose that read through the existing AutomationRuntime/Session store
  boundary. `prepare_backend_fire` must require a checked resolver for the
  selected entry and requested UUID, before enabled, revision, legacy-slot,
  trigger or event decisions. Errors/ambiguity return existing Abandon without
  invoking completion. Known Running retains `existing_run_unsettled`, terminal
  replay remains existing Complete, and only checked absence proceeds. The
  shipping Host supplies the real resolver; there is no permissive default.
- ARH08: Checked history parsing must reject duplicate JSON object members
  recursively before Value normalization. Duplicated `status`, `id`,
  `coalescedRunIds` or an unknown nested key could otherwise hide Running
  identity or permit a false terminal ACK. Checked delivery lookup, new/existing
  begin and finish all fail without writing; normal fail-soft display stays
  unchanged. Valid unique-key legacy rows and legitimate terminal receipts pass.
- ARH07: Preserve Session deletion/filesystem admission, Runtime mutation and
  in-flight ownership, Spend Guard, all ACK/custody/deadline predicates and the
  existing backend completion/ACK path. AHA-04 still applies: lastRunAt may
  change before a later begin refusal; this is not a cross-file transaction.

Production scope is the existing `automation_store.rs`,
`extensions/transcript/automation_runtime.rs`, `extensions/automations/fire_delivery.rs`
and the actual Host `dispatch_production_backend_fire` caller. The integrator
owns the public resolver contract and Host composition. Reuse existing selected
store, Runtime, run-path, delivery/backend, production-wiring and app custody
test homes; update every caller of the required resolver. No schema, dependency,
AgentDb duplicate semantics or new permission/scheduling truth changes.

The shared Rust contract is `FileAutomationStore::read_run_for_delivery(id,
run_id) -> Result<Option<AutomationRun>, AutomationStoreError>` and
`AutomationRuntime::read_run_for_delivery(agent_id, automation_id, run_id)
-> Result<Option<AutomationRun>, String>`. Extend `prepare_backend_fire` with
a required fourth argument `Fn(&AgentAutomationEntry, &str)
-> Result<Option<AutomationRun>, String>`, after the existing revision resolver.
The shipping callback passes the Runtime read. An error maps to
`Abandon { reason: "run_history_unavailable" }`; retain the existing known-run
reason strings. Reuse existing Session filesystem/deletion serialization for
the read without nested acquisition or new mutation; all existing history
writers remain under their current owner. Test-only fixture resolvers must be
explicit and cannot become a production default.

Preserve valid CRUD/cap assertions using settled rows, then add the 20-Running
capacity negative. Replace only the obsolete expectation that a still-present
old Running row must be treated as absent: retain a genuinely absent no-write
negative and add exact old-row settlement/disk/returned-receipt positives.
Keep malformed tails, duplicates, legacy normalization, write failures,
cancellation, unknown custody, alias/identity and completion/ACK assertions.

Actions must exercise actual RetainedUnsettled Runtime output, twenty real
successful manual runs, unchanged original file/cron identity, reopened
Session/Automation owners and a fresh controlled backend. Repeated original
and alias delivery must enter no executor and send no completion/ACK. Include
hidden legacy Running, malformed/ambiguous authority, full-capacity refusal,
backdated admission, legacy excess drain, distinct-run and terminal positives.
Controlled transport is a protocol fixture, not real-service acceptance.
Freeze the complete batch, independently review and obtain exact-H Actions.

Already deleted history cannot be reconstructed here. Concurrent/cross-process
execution claims, the lookup-to-begin race, direct reuse of an existing UUID,
evicted terminal receipts, full Spend Guard reconciliation and unknown external
effects remain open. Rollback reintroduces unsettled eviction/replay risk; no
stored format changes. Preserve full OCRM scope and all eight global flags.

## Checked backend delivery custody — AUTO-BACKEND-HISTORY-v2

Decision **AUTO-BACKEND-HISTORY-01 — FIX / IMPL_BUG**, before implementation,
against H `3c4bf6963cb30ecf01eb34b905315074886e1972`; fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. The existing delivery reader uses
legacy normalization that can turn missing/unknown status into `ok`, discard
invalid finishes or aliases, and miss cross-row identity conflicts. Separately,
the shipping Host discovers targets through a fail-soft display list: an
unreadable or invalid matching definition is omitted, so preparation can report
`automation_missing` and the backend can POST/ACK an unsettled original or
alias without consulting history. These are target-owner implementation gaps;
this batch adds no upstream reading credit or broader acceptance claim.

### Required behavior and shared interfaces

- BH-01: `FileAutomationStore::read_run_for_delivery` validates the complete raw
  unique-key history before legacy normalization. Require a safe local ID,
  nonempty literal requested run ID and a readable valid matching definition.
  Only a genuinely missing history file under that definition is first-run
  absence. An unreadable object, dangling link, invalid row or bad unrelated
  tail refuses both matching and absent queries. Never create paths, truncate,
  sort away authority or rewrite history during this read.
- BH-02: Every raw row requires a nonempty literal primary ID, finite startedAt,
  explicit `running`, `ok` or `error`, and absent/null finishedAt for Running
  versus finite finishedAt for terminals. Aliases are absent/null or at most
  `MAX_EVENTS_IN_AUTOMATION_WAKE` nonempty literal strings. Preserve identity
  bytes, self aliases and repetitions within one row. A primary or alias owned
  by different rows invalidates the entire image, even for an unrelated query.
  Metadata retains its existing normalization. Legacy display and mutation
  APIs retain their contracts, including `begin_fresh_run` and its regression.
- BH-03: The required resolver in `prepare_backend_fire` remains before all
  enabled/revision/slot/event policy. Also validate a returned typed run's
  identity match, times, status and alias constraints. Error or invalid typed
  authority returns `Abandon { reason: "run_history_unavailable" }`. Known
  Running retains `existing_run_unsettled`; valid terminals replay stored
  success/failure; checked absence preserves existing eligibility behavior.
- BH-04: Checked target discovery is separate from display listing. Add
  `FileAutomationStore::find_for_backend_delivery(agent_id, backend_id)
  -> Result<Option<AutomationRecord>, AutomationStoreError>`. Enumerate the
  existing directory with checked errors, compare the existing stable backend
  hash of safe local folder identities, and then read the matching definition.
  Read/entry/metadata/UTF-8/JSON/definition failures are unknown, not absence.
  An existing matching non-directory or folder with missing config is refusal.
  Detect duplicate JSON members before the existing definition parser; retain
  valid legacy definition semantics. Do not parse unrelated definitions. The
  returned target is definition-only with an empty runs projection; only the
  required checked history resolver supplies custody. Reuse record construction
  without invoking display history. Refuse nonregular config/history objects
  before reading, so a FIFO cannot block this admission boundary. Ordinary
  display/mutation reads retain their compatibility behavior.
  A missing automation directory or completed enumeration with no matching safe
  identity is confirmed absence; a dangling directory link is not absence.
  Preserve valid root-directory links after checked directory metadata, as the
  existing directory reader does. A matching child link is refused: ordinary
  discovery excludes it, so this change cannot newly admit execution through it.
- BH-05: Expose the target read through existing owners only:
  `ProductionSessionWorkers::find_automation_for_backend_delivery(agent_id,
  backend_id) -> Result<Option<AutomationRecord>, String>`, and the same name
  on `SandAgentSessionStore` and `AutomationRuntime` returning
  `Result<Option<AgentAutomationEntry>, String>`. Runtime holds its per-agent
  mutation boundary; Session holds its existing filesystem/deletion boundary
  exactly once. Validate the agent identity and positively establish directory
  absence; a malformed/unreadable existing agent or pending deletion is an
  error. For a present agent, perform existing durable admission before reading
  FileStore. Do not recursively reacquire the public Session admission lock.
  Preserve live Session root/agent directory links after checked metadata;
  dangling links and other path uncertainty remain refusal.
  Ordinary database owner opening/recovery remains permitted and unchanged.
- BH-06: The shipping Host uses checked targeted discovery, never the fail-soft
  list, before preparation. Discovery error takes the existing false/Abandon
  path with no executor or completion callback; no completion POST or ACK is
  authorized. Only checked absence can retain the documented missing-target
  completion. The shared production dispatch helper used by the actual Host
  must own this preparation and immediate completion/refusal branch, so tests
  exercise that branch rather than manually reproducing its match statement.
  Existing scheduling/event launch, telemetry for known dropped fires, provider
  checks and backend Complete/Reported/ACK state machine remain their owners.

### Verification, integration and limits

Keep ARH capacity/retention, fresh-admission, duplicate-key, legacy mutation,
Session deletion, terminal custody, coalescer None, deadline and failed-POST
controls. Add full raw-history and typed-return negatives, cross-row aliases,
exact producer bound, valid Running/terminal/absence and explicit repair.
Use actual Runtime/Session/FileStore and backend owners with controlled external
transport. Through the shipping dispatch helper, repeatedly deliver original
and alias with malformed/missing/unreadable definitions, unknown directory
enumeration, then reopen owners: require no execution, callback, POST, ACK or
observation and unchanged history/config/cron in clean fixtures. Repair must
restore Running refusal, terminal replay and a distinct new fire; genuine
missing targets remain a positive documented completion control. This does not
assert zero database opens or suppress Session's unrelated recovery.

Allowed implementation paths are the existing FileStore, fire preparation,
Session production/facade, AutomationRuntime and root-owned Host composition;
tests stay in existing store/fire/backend/runtime/production-wiring and app
custody homes. The existing Host composition contract also checks that the
shipping entry returns this helper's result with its real Runtime/fire/callback,
including wrong-owner, ignored-result and test-only/dead-call negative controls.
Root serializes this spec, workflow selectors/receipts and the
existing OCRM ledger/RTM references. No schema, dependency or persistent owner
is added. Preserve all uncommitted predecessor work; transplant only reconciled
useful deltas, never overwrite the newer fresh-admission implementation.

Freeze the integrated candidate, obtain independent source review and exact-H
Actions compile, contracts, shipping wiring and high-risk negative evidence.
Reuse the existing Linux native-path job for FileStore's invalid-UTF-8 name
oracle: APFS cannot supply that filename fixture, so a macOS-only pass cannot
cover it. Bind its exact nonzero positive result in the existing job receipt.
The app step must pass the existing owned Automation artifact root and the
existing failure upload must also run when that earlier app step fails, even
if the later integration step never starts. Keep exact child receipts/deadlines.
Local or htch-runtime validation is prohibited. Rollback restores the previous
reader and can reintroduce false completion; disk formats are unchanged. Keep
all eight completion flags false. Real service compatibility, previously
normalized/deleted history, lookup-to-begin races, cross-process claims, event
loss/unknown effects and the full reaper/final OCRM gates remain open.

## Backend admission refusal — AUTO-BACKEND-ADMISSION-v1

Decision **AUTO-BACKEND-ADMISSION-01 — FIX / IMPL_BUG**, before implementation
against H `4df79ca181c8daa98080dbea0c8e8143c0407e05`; fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. BH checked preparation is separate
from Runtime's history precheck and the Session-admitted fresh begin. A refused
attempt currently returns a String error which the scheduled Host turns into
failed completion and eventual ACK despite no executor or admitted run for this
attempt. Twenty retained Running rows with a distinct UUID demonstrate a
deterministic trigger. Competing claims between the existing boundaries expose
the same issue. This contract repairs the disposition of refused admission;
it does not make those boundaries atomic or claim all late input is refused.

- BA-01: Add an in-memory `AutomationRunAttempt` in the existing run-path owner:
  `Outcome(Option<FireAutomationOutcome>)` or
  `AdmissionRefused { stage: AutomationAdmissionStage, detail: String }`.
  Stages are `RuntimeAdmission`, `HistoryPrecheck`, `SpendGuard`,
  `NoticeBookkeeping`, `RunBookkeeping`, and `FreshHistory`.
  A refusal means this invocation has not received a nonempty fresh history
  identity and has not entered its executor. It does not promise zero earlier
  bookkeeping writes, absent prior ownership, or no write after an IO error.
  No persisted/wire schema, new state owner, string classification or blanket
  `From<String>` conversion is permitted.
- BA-02: Factor the existing run-path body into typed
  `fire_automation_attempt_with_on_duplicate` and
  `fire_automation_attempt_with` entry points with one in-flight owner and
  one execution/settlement body. At the explicit notice, record-run and
  begin-run operations, return the corresponding refusal on error; missing or
  empty returned history ID is FreshHistory with the existing exact detail.
  Existing `fire_automation_with[_on_duplicate]` lower Outcome unchanged and
  AdmissionRefused to the original String error, preserving manual/UI callers.
  Duplicate, paused/suspended and RetainedUnsettled results remain Outcome(None).
  Release volatile occupancy on every normal typed return, including refusal.
- BA-03: Factor Runtime's background execution into
  `run_background_automation_attempt_with`; the legacy background wrapper
  lowers typed refusal to the same String error. Explicit store/Session setup
  errors use RuntimeAdmission; checked-ID/Session errors or an already claimed
  primary/alias use HistoryPrecheck; Spend Guard errors use SpendGuard.
  Preserve current missing-definition/suspended/paused None behavior and
  bookkeeping/lock order. Feed the typed run-path core through the current
  AdmittedAutomationRunStore and `begin_fresh_run`; no lock is broadened and
  no Session public admission call is nested inside its held boundary.
  Expose the backend scheduled facade
  `run_server_scheduled_automation_attempt_with` with the same arguments as
  its legacy counterpart, returning `Result<AutomationRunAttempt, String>`.
  Keep the legacy scheduled/manual/event interfaces and error details.
- BA-04: The actual scheduled Host worker uses the typed scheduled facade and
  its actual completion mapper accepts the typed attempt. Only
  `Ok(AdmissionRefused { .. })` newly maps to None; existing outcomes and outer
  errors retain their completion mappings. After an accepted launch, invoke
  the real callback with None so the existing backend owner abandons volatile
  Running without POST/ACK. Merely dropping this callback is not sufficient.
  Event enqueue results are explicitly wrapped as Outcome before that mapper;
  the event coalescer retains its current inner-error-to-None reduction and
  outer receiver-error behavior.
- BA-05: Once a nonempty fresh history ID has been received, preserve cron
  registration rejection and its history cleanup/error behavior. Preserve all
  executed Completed, Failed, Interrupted and raw executor Err settlement,
  reporting and failure accounting; preserve ACK-05's outer post-execution
  settlement-error completion. Never blanket-map every error to None, infer
  admission from an executor boolean, or replace fresh begin with legacy
  idempotent begin.
- BA-06: Causal oracles cover actual RunPath, Runtime/Session/FileStore, Host
  mapper and backend callback/POST/ACK. Cover early precheck and later fresh
  admission refusal, primary and alias claims, terminal claim appearing after
  preparation, malformed input, deletion admission, full Running capacity and
  no returned ID. Early precheck refuses before later bookkeeping; later
  refusal preserves competing history/cron while allowing already-authorized
  notice/lastRunAt/guard writes. Require typed stage, zero executor/terminal
  observation, one None callback after accepted launch, no completion POST/ACK,
  released volatile state, repeated poll/reopen and repair/distinct-run positive
  controls. Keep exact legacy String errors, real execution failure and ACK-05
  positives; None alone cannot prove the new typed branch.
  The existing Host source oracle must bind the scheduled worker's actual
  returned typed facade result to the real completion callback, with wrong
  owner, ignored/replaced result and dead-call negative controls.

Use existing deterministic seams, not sleeps or new production phase hooks:
BH's actual shipping helper launch closure can place a claim after preparation;
the synchronous existing lifecycle reporter can place a claim after Runtime's
precheck and before fresh begin. In the latter fixture the Session closure has
returned but Runtime's mutation mutex remains held: the reporter may call
Session/FileStore directly, never re-enter Runtime or nest Session locks.
Seed and deliberately stale the existing definition baseline, do not attach
the directory watcher, and assert the one-shot event that causes the fault.
Reuse existing bounded owned-child deadlines, receipts and failure artifacts.
The original BA repeated-redelivery oracles use the actual current definition
revision to isolate admission refusal. At that batch's publication, the old
no-revision covered-slot policy remained unchanged: a refused attempt's allowed
lastRunAt write could cause that later legacy policy to complete a subsequent
fire. The follow-on AUTO-LEGACY-SLOT-CUSTODY-v1 contract below repairs that false
terminal inference while preserving refusal to replay the ambiguous slot.

Implementation remains in `automation_run_path.rs`,
`automation_runtime.rs` and root-owned Host composition; tests stay in their
existing run-path, production-wiring and app custody homes. Root serializes the
spec, composition/wiring oracle, workflow selectors/receipts and additive OCRM
references. Freeze the integrated source and obtain independent review plus
exact-H Actions for affected compilation/contracts and causal negatives.
Reuse the existing Linux Host cache/job for typed RunPath and real Runtime
production-wiring contracts with nonzero named-result evidence and retained
owned failure fixtures. Library/source-wiring results do not establish actual
App/macOS execution or whole-platform acceptance. While macOS remains queued without an
assigned runner, also use the same Linux job/cache to check the actual App
binary with cfg(not(test)), compile its actual test harness from the committed
App manifest/lock, bind the emitted executable to H/tree/source/binary hashes,
and execute the four BH, four BA and completion-mapper cases by exact names.
Require one selected outer test with no failed/ignored case, bounded child
receipts, complete command logs and success-only fixture cleanup. This direct
Linux App evidence supplements the pending macOS gate; it cannot replace it.
No new runtime, copied test-module carrier or local native-dependency probe.
No local or htch-runtime validation.
No disk migration; rollback restores the
old false-completion risk. Existing manual behavior is a compatibility gate.

Strict-delivery-invalid history introduced after precheck can still be accepted
by the intentionally legacy mutation parser. Late nonregular mutation inputs,
cross-process claims, unknown write/execution effects, already-admitted cron
cleanup uncertainty, event loss, real external backend compatibility, full
reaper and all final OCRM gates remain separate open work. This batch earns no
new upstream reading credit or global flag promotion.

### AUTO-ADMISSION-ERROR-ORACLE-01 — trigger-specific reader diagnostics

Decision **FIX / TEST_FLAW**, from exact H
`f890002148f122aacdfc78fe39530a7e9c1bef6e`, runtime run `41527502935`, Linux
job `125558911916`: the nine-target command has 114 passed and two failed
outer tests, Cargo exit 101/capture exit 0. Scheduled/event pre-admission
controls stop at the MixedRows error assertion. The existing background
precheck already uses `read_run_for_delivery`, which reports invalid delivery
custody and competing row identity; the manual fresh-history mutation reader
reports an invalid row and duplicate IDs. These distinct readers and the stale
shared expectation also exist in parent `1b9803de2283322a7809dc626a5ca35a52eb834e`.
Keep both production contracts unchanged. Correct only the pre-admission
fixture's trigger-specific MixedRows/DuplicateIds messages and include the
actual error in assertion diagnostics. Preserve all zero-execution, unchanged
bytes, no-observer/no-cron, released-occupancy and explicit-repair positives,
all corrupt-JSON cases, and the settlement helper's original mutation-reader
expectations. Fresh focused source review and new exact-H execution remain
required; the skipped actual-App step is not evidence of compilation or pass.

## Fresh history custody — AUTO-FRESH-CUSTODY-v1

Decision **AUTO-FRESH-CUSTODY-01 — FIX / IMPL_BUG**, before implementation
against H `f890002148f122aacdfc78fe39530a7e9c1bef6e`; fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. BA preserves a refused delivery,
but `begin_fresh_run` still uses the legacy mutation parser. For example, an
existing row's 26th alias can equal the pending fire ID: normalization truncates
that alias before collision detection, then admission overwrites the history
without its prior custody. Unknown status, incompatible finish fields, mixed
aliases and cross-row aliases have the same strict-versus-normalized gap.
An observed FIFO installed after checked preparation can also block an earlier
display/config read before the fresh admission boundary is reached.

The producer remains manual/scheduled/event ingress, the unique state owner is
FileAutomationStore inside the existing Session-admitted mutation, and the
consumer remains the existing RunPath/Runtime and BA Host callback. No new
schema, scheduler, receipt store, execution loop or authority is introduced.

- FC-01: Extract the existing full raw delivery-custody validation into one
  owner-local checked-history reader returning all validated rows. Both delivery
  lookup and fresh mutation use it. Validate every row before lookup, capacity,
  retention or publication, including tails beyond the display window. Retain
  recursive duplicate-member rejection; explicit running/ok/error status;
  finite timestamps; Running with absent/null finish and terminal with finite
  finish; nonempty literal primary/aliases; at most 25 aliases; and no identity
  owned by different rows. Repeats/self-aliases in one row and literal whitespace
  remain valid. Metadata normalization may follow custody validation. Do not
  use a fabricated lookup ID or strict precheck followed by a legacy reread.
- FC-02: Fresh admission consumes that same checked image for all requested
  primary/alias collision checks, Running capacity, retention and its one
  history write. Retain every Running row, reject growth at 20 Running rows,
  retain the existing terminal window and preserve settlement of legacy excess
  Running rows. Legacy `begin_run`, `finish_run`, display parsing and their
  ordinary-file normalization remain compatible. A fresh validation refusal
  must leave the exact existing history bytes/object unchanged.
- FC-03: Validate the proposed fresh row before publication: supplied time must
  be finite, an explicit primary ID must be nonempty, and supplied aliases must
  contain at most 25 nonempty literal strings. Do not silently drop or truncate
  requested custody. Generated IDs, whitespace, repeated/self aliases and
  exactly 25 aliases remain valid. Missing definition still returns None before
  these checks and must never create a definition/history. Trigger/detail/event
  normalization is unchanged. The legacy API keeps its existing input behavior.
- FC-04: Checked fresh history establishes a local child automation directory,
  follows valid regular-file links, and distinguishes genuine missing history
  from dangling links, nonregular objects and unavailable paths. Preserve BH's
  root-directory link policy. Reuse one owner-local regular-file content-read
  primitive on config/display/raw-history reads that precede admission; reject
  an observed nonregular object before attempting a blocking content read.
  Preserve fail-soft display None/empty results and real run counts for regular
  inputs: `list_definitions().runs` feeds Spend Guard and lifecycle snapshots.
  Do not replace those histories with empty projections or broaden locks.
- FC-05: Errors at fresh mutation propagate through BA's existing FreshHistory
  stage and the actual backend callback None/abandon path. Earlier config
  disappearance/nonregular reads can legitimately preserve Outcome(None) or an
  earlier refusal stage. Assert the actual reached stage; do not relabel all
  failures. Zero executor/new cron/terminal observer activity is required for
  these refusals. Previously authorized notice, lastRunAt and Spend Guard
  writes remain; exact-byte no-write claims apply to refused history, not the
  entire attempt. Post-ID execution/cron/settlement and ACK-05 remain unchanged.
  Diagnostic expectations follow the reader actually reached: background
  HistoryPrecheck already returns strict-custody errors; this contract makes
  manual FreshHistory use that same checked reader. String compatibility
  wrappers preserve the resulting detail verbatim. Legacy FileStore begin/
  finish errors remain separate, as do post-admission settlement diagnostics.
- FC-06: Store oracles distinguish strict-only-invalid from malformed JSON and
  from still-supported legacy normalization. Cover alias 26 equal to pending
  ID and an unrelated ID, unknown/missing status, invalid finish/alias forms,
  cross-row primary/alias and alias/alias ownership, invalid hidden tails,
  nonfinite proposed time, alias overflow/empty values, literal valid controls,
  true absence, capacity and links. Reuse existing owner test homes and retain
  every valid earlier assertion. Production oracles inject strict-invalid
  history in the existing synchronous lifecycle seam after Runtime precheck,
  assert unchanged custody/zero execution, explicitly repair it, and prove a
  distinct run succeeds through the real Session owner.
- FC-07: Exercise stable config/history FIFOs with no peer after checked
  preparation and after the lifecycle precheck, using the existing exact owned
  child/deadline/receipt harness. The child must return the declared outcome,
  not just avoid executor entry. Snapshot FIFO metadata/inode and other regular
  state without reading the FIFO. Preserve regular-file/link and Spend Guard
  run-count positive controls. Scope is observed nonregular input rejection;
  metadata-to-open replacement, arbitrary filesystem latency and writer staging
  liveness are not proved by these checks.
- FC-08: Actual App coverage uses checked shipping preparation, typed scheduled
  Runtime, the real completion mapper/callback and backend consumer. A late
  strict-only-invalid image must yield FreshHistory, one None callback after
  launch, zero completion POST/ACK, retained bytes and released volatile state.
  Reopen the real owners/consumer, re-poll the revisioned fire, then explicitly
  repair history and prove a distinct valid fire completes. Keep the legacy
  no-revision covered-slot limitation explicit and preserve admitted failure
  controls. No mocks may substitute for actual provider/channel acceptance.

Implement in `source/host/src/automations/automation_store.rs`; extend existing
store, Runtime production-wiring and App terminal-custody tests. Root alone
integrates the existing workflow's named positive controls, spec and additive
ledger/RTM references. Freeze a complete source tree, review the changed owner
and adjacent callers independently, and run affected Store/Runtime/Guard/event/
lifecycle/contracts and actual App controls on exact-H GitHub Actions. Preserve
earlier valid evidence and classify a failed oracle separately from a product
failure. No local/htch execution; no acceptance or source-coverage promotion.

There is no persisted format migration. Rollback restores the lossy fresh
admission risk; do not repair existing history automatically. Manual inspection
and explicit repair are required for refused invalid history. Existing String
error detail/typed stage provide diagnostics. This contract does not solve
cross-process writes, path replacement, unknown write/cron outcomes, config
revision races, post-admission normalization, historical data already lost,
legacy covered-slot reconciliation, crash durability, full reaper or final OCRM
acceptance. The fixed-U receipt/queue sources and exact existing owner chain
are reconciled in this task's recorded source handoff; imported SQL/state-worker
closure and global recursive/legal source closure remain open.

## Legacy covered-slot custody — AUTO-LEGACY-SLOT-CUSTODY-v1

Decision **AUTO-LEGACY-SLOT-01 — FIX / IMPL_BUG**, before implementation on
H `74484facdac03260af2b5ca1bd7963dc2ce5c1c2`; fixed U remains
`1440bb1771372334bea23d7642bb52deb7db1845`. The actual RunPath writes
`lastRunAt` before fresh history admission. A capacity or other typed admission
refusal can therefore leave that timestamp without a history identity for the
requested fire. On a later no-revision, no-event delivery, checked history can
establish that UUID's absence while the existing covered-slot branch treats
the timestamp as failed completion. The shipping helper then calls completion,
and the existing backend consumer posts and ACKs a run that was never admitted.
The source chain establishes the defect; exact-H execution is still required.

- ALS-01: keep the existing checked history lookup and its precedence. When
  exact primary/coalesced authority is absent and the existing legacy covered-
  slot predicate matches, return `PreparedBackendFire::Abandon` with the stable
  reason `legacy_slot_custody_unresolved`. A timestamp is neither an execution
  receipt nor sufficient terminal evidence. Do not reset the timestamp, invent
  history, retry execution, or infer that another run did or did not execute.
- ALS-02: use the existing shipping helper and backend consumer's abandoned
  admission path. Preparation returns false, starts no worker and invokes no
  completion callback; the existing consumer releases its volatile Running
  state without a completion POST or ACK. Every later poll and reopened owner
  must preserve that disposition until real authority changes. No new queue,
  completion store, scheduler, callback type or wire field is introduced.
- ALS-03: preserve checked Running/invalid-history refusal and real terminal
  primary/coalesced success/failure replay before definition/slot checks.
  Genuine first-run legacy schedules, revisioned schedules, event matching,
  missing/disabled/changed-definition policy and already-admitted failure
  settlement retain their current behavior. An old covered timestamp still
  prevents automatic re-execution; only its unsupported terminal inference is
  removed. The prior unit expectation of `Complete(slot_already_covered)` is
  superseded; retain and strengthen its valid no-execution assertion.
- ALS-04: extend the real checked-backend App harness with a no-revision fire
  whose first actual Runtime attempt meets full Running capacity. Observe
  FreshHistory, one None callback, zero executor/terminal effects and the real
  newly written lastRunAt. Re-poll and reopen owners/consumer: require no extra
  launch/callback, no POST/ACK and unchanged config/history/cron. Repair capacity
  and show the same ambiguous legacy slot remains held. A distinct revisioned
  fire and actual stored terminal replay provide liveness/settlement controls.
  Keep the existing revisioned capacity oracle and every valid BA/BH/FC assertion.
- ALS-05: add bounded pure-owner controls for the timestamp boundary, absent
  authority, current revision/event bypass and stored Running/terminal priority.
  Select the new exact App case and required portable positives in the existing
  Actions workflow/receipts; preserve executable, source and command binding.
  Controlled backend/provider I/O is not real service compatibility evidence.

Implementation is confined to the existing
`extensions/automations/fire_delivery.rs` owner; tests stay in
`automations_fire_delivery_contract.rs`, `automations_backend_contract.rs`, and the shipping App's existing
`automation_terminal_custody_tests.rs`. Root integrates this spec and existing
workflow. All paths already belong to the current OCRM Automation responsibility;
no new owner, persisted schema, source-review status or acceptance flag is added.
Reuse the recorded fixed-U receipt/finalization/queue review bounds, without
claiming their protocol directly defines Fabushi's legacy wire policy. Freeze
the complete batch for independent review and exact-H Actions only.

This change deliberately retains an ambiguous delivery; it does not establish
automatic legacy-slot reconciliation, reconstruct pruned/missing history,
undo an admitted external effect, or resolve cross-process claims and unknown
write/settlement outcomes. Those lifecycle, real-backend, reaper and full
G01–G12/AC01–AC24 gates remain open. There is no data migration; rollback restores
the unsupported completion/ACK risk without changing user data.

### ALS-REVIEW-01 — preserve the reopened backend custody oracle

Decision **FIX / TEST_FLAW**, before the review repair: the complete consumer
review found `automations_backend_contract.rs::reopened_running_original_and_alias_never_execute_complete_or_ack`
still requiring `Complete(slot_already_covered)` for its absent distinct-ID
control. Its actual persisted lastRunAt and legacy fire timestamp are both 100,
so ALS-01 correctly returns `Abandon(legacy_slot_custody_unresolved)`. Change only
that superseded outcome. Preserve both reopened generations, every Running
original/coalesced no-execution/no-callback/no-POST/no-ACK assertion, exact stored
bytes/cron rows, and all disabled/revision/event controls. Require this existing
case's positive result in the same portable receipt; no production gate is
weakened and no executed failure is claimed from this source review.

