# Session work admission — production boundary

Status: architecture selected; deletion-fence implementation slice in progress
Date: 2026-11-01
Responsibility: `OCRM-RESP-SESSION-WORK-ADMISSION-001`

## Context and goal

Fabushi already owns agent SQLite state, transcript mutation, Runner turn
settlement, and lifecycle revisions. The fixed OpenClaw session work-admission
surface additionally prevents writes during deletion, binds detached results to
the admitted session generation, protects archive/restore transitions, and
retains provider-review terminal ownership. The goal is to implement those
semantics through the existing Fabushi owners with one producer → owner →
durable state → consumer chain.

## Non-goals

- No second session database, workboard, or provider-specific result store.
- No reuse of cloud-agent archive/unarchive as session lifecycle authority.
- No filesystem-age or automation-history-only deletion.
- No acceptance claim from fixtures, static symbols, or a test-only helper.

## Requirements

- `SWA-01`: Every admitted session write carries canonical session identity and
  a durable generation/lifecycle revision.
- `SWA-02`: Pending agent deletion rejects new writes and retains any owned
  terminal writes until the existing deletion owner publishes a safe replay or
  failure outcome.
- `SWA-03`: Detached background results commit at most once against the exact
  admitted session generation and preserve transcript/media idempotency.
- `SWA-04`: Archived sessions reject new work; restore requires the existing
  authenticated session/placement authority and rechecks identity at commit.
  The first production producer is the authenticated Host Gateway
  `sessions.patch` route for the canonical `agent:{id}:main` session; it must
  carry expected session identity and lifecycle revision and fail closed when
  admitted background work or an active foreground Runner stream is still
  active.
- `SWA-05`: Provider-review terminal facts remain owned by the admitted run;
  retries, fallback providers, and successor runs cannot claim them.
- `SWA-06`: Stale, deleted, archived, duplicate, cancelled, and
  outcome-unknown results are explicit and fail closed.
- `SWA-07`: Existing `ProductionSessionWorkers`, `SandAgentDb`, transcript
  runtime, and Runner remain the owners; no parallel state owner is introduced.

## Current and target state

Current Fabushi has per-agent SQLite transcript state and lifecycle revisions.
The owner now also has a migration-safe `sessionWorkAdmission` record,
generation-bound background admission/recheck, and a same-owner idempotent
hidden result receipt for the bounded local background-revival path. There is
still no complete media custody contract or full provider-review terminal
lifecycle; the session owner now has bounded typed refusal recording,
continuation admission, and generation/run-bound terminal settlement, while the
media owner has a durable path fence, settlement receipt, and bounded reopen
recovery consumer.
The authenticated Host Gateway archive/restore producer is now the first
bounded production path, limited to canonical main sessions and fail-closed
while admitted work, active generated-subagent work, or an active foreground
Runner stream is active. Media custody still has no equivalent session-owned
active-work authority and remains an open requirement.
`deleting_agents` remains an in-memory
deletion fence and is insufficient as the complete contract.

Target state is one owner-local session admission record containing canonical
`sessionKey`, `sessionId`, lifecycle revision, archive metadata, admitted
background-work identity/idempotency, one durable pending outgoing-media work
identity, and structured provider-review facts, continuation run identity, plus
terminal phase. The record
is changed transactionally with transcript/result publication. Pending media
must block archive/restore transitions and be cleared only by the same
session/transcript owner after commit or explicit rollback.

The current background-revival producer was rechecked on 2026-11-01. It
constructs a canonical main-session key, a per-run `clientNonce`, and a
`streamId`, captures the admission generation, calls the existing routed-send
admission, waits for the matching runner terminal event, rechecks the generation,
and commits the result through the same `SandAgentDb` owner. Identical retries
return the committed receipt and conflicting payloads fail closed. This bounded
path still does not represent the full upstream detached-result contract or
provide archive/restore, media, or provider-review ownership.

## Ownership and data flow

`Gateway/Runner/background producer → ProductionSessionWorkers/SandAgentDb →
session admission record + transcript transaction → existing Runner,
transcript, and provider-review consumers`.

The owner must recheck the captured generation and deletion/archive state at
the actual durable commit edge. Unknown mutation outcomes must not be retried
without a committed receipt.

## Failure and edge cases

- missing or malformed session identity: reject before durable admission;
- session generation changed: return stale/retryable result without mutation;
- deletion pending: reject new work and retain owned terminal writes;
- archived or restart-tombstone session: reject work until authorized restore;
- duplicate result: return the original committed outcome;
- same idempotency key with different payload: fail closed;
- active media/subagent work: preserve it during archive/reaper decisions;
- cancellation/provider failure: leave prior transcript and generation intact;
- uncertain deletion/transcript result: report outcome-unknown and do not retry.
- provider-review continuation dispatch failure: restore the review to
  `pending`; Runner cancellation/failure/unknown: retain terminal facts in a
  non-actionable phase; Runner success: clear the matching review.

## Implementation strategy

1. Add migration-safe owner-local session admission schema and compare-and-set
   methods to `SandAgentDb` and expose them through `ProductionSessionWorkers`.
2. Bind the canonical Runner admission boundary to `SWA-01/02`.
3. Add background-result settlement with captured generation and idempotency.
4. Add a durable owner-local pending-media identity around existing
   `OutgoingMediaCustody`; materialization, transcript acceptance, rollback,
   and archive checks must use the same session generation.
5. Connect the authenticated Host Gateway archive/restore producer to the
   owner-local transition and then connect the existing provider-review
   terminal owner.
6. Add direct negative contracts, then perform exact-head verification.

## Verification and acceptance

Focused contracts must prove disabled/archived/deletion-pending admission,
stale generation, duplicate and conflicting result, lifecycle change during
restore, active media/subagent preservation, and terminal review ownership.
Static presence is insufficient. The parity row remains
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`,
`acceptance=pending` until all production edges and required exact-head
evidence exist.

## Current implementation slice

`ProductionSessionWorkers::assert_agent_work_admission` now fences the
existing transcript append/update/delete, compaction, and conversation-clear
wrappers while `begin_agent_delete` owns the deletion window. `SandAgentDb`
also now owns a migration-safe `sessionWorkAdmission` KV record with
compare-and-set lifecycle generation transitions, exposed through
`ProductionSessionWorkers`. New sessions initialize a direct-main record, and
the existing `call_send_prompt` plus local routed producer paths check that
record before admission. Focused contracts cover rejection before durable
write, admission after the fence clears, create-only migration, guarded
archive transition, archived routed-work rejection, and stale transition
rejection. This still does not wire the full detached background-result
lifecycle or native provider-review runtime recovery, so the responsibility
remains open even though the bounded provider-review terminal settlement path
is now wired.

The background revival path now also captures the active main-session
`lifecycle_revision` before routed admission and rechecks the same generation
at the matching `Completed` terminal event. A changed, archived, deleted, or
missing record is rejected before the revival reports success. This closes the
generation-observation guard only; it does not provide the durable result
receipt, or provider-review consumer required by `SWA-03`–`SWA-06`; the
bounded Gateway archive/restore producer is now present.

The current slice adds a same-owner terminal receipt for that revival path:
`SandAgentDb` writes a hidden `background-result-receipt` transcript entry and
clears the admitted work marker in one SQLite transaction. The deterministic
receipt identity is the idempotency key; an identical retry returns
`AlreadyCommitted`, while a different payload for the same key returns an
explicit conflict. Admission is released on pre-run rejection, cancellation,
timeout, disconnect, and terminal failure. This is wired only for the local background-revival
consumer; the existing `ProductionSessionWorkers` transcript/page/thread
projections filter the receipt kind so it cannot enter user-visible context.
It does not yet establish the full detached-result, media, or provider-review
lifecycle required for responsibility completion; the bounded Gateway
archive/restore producer is now present but intentionally rejects active
admitted work.

The receipt transaction also rolls back the inserted receipt when its
post-insert lifecycle compare-and-set loses a race. This preserves the
`RejectedStale` outcome without leaving an orphan receipt that could make a
later retry appear `AlreadyCommitted` while the admission marker still owns
the work. The focused owner oracle is recorded in
`source/host/tests/agent_db_owner_contract.rs::stale_background_result_cas_rolls_back_receipt_insert`;
it remains unrun during the development freeze.

The 2026-11-01 background-result recheck confirms the bounded receipt path is
now present, but the gap remains at the broader lifecycle boundary:
`run_local_background_revival_turn_with_context` has `agent_id`, canonical
`sessionKey`, generated `clientNonce`, `streamId`, a matching terminal event,
generation recheck, and same-owner receipt commit. The Gateway archive/restore
producer now reaches the same owner-local record with expected
identity/revision guards and rejects active admitted work, active generated
subagent work, and active foreground Runner streams before the durable
transition. The media custody integration now includes durable path capture and
bounded reopen recovery, but full detached media/result replay and the
full provider-review terminal lifecycle remains absent. The next implementation
instrument must
extend the existing session/transcript owner with those explicit outcomes; it
must not replace the receipt with a best-effort CAS or add a parallel store.

The Runner's `request_box_help` transcript publication is also an admitted
session write. Because that producer already carries the canonical
`session_key`, it must use
`ProductionSessionWorkers::append_agent_transcript_entries_if_session_admitted`
so archive state and the durable session identity are checked at the write
edge. The generic agent-only append remains reserved for producers whose
existing ownership is not session-scoped; it must not be used to bypass this
session-bound admission boundary.
The session gateway's accepted-send producer now applies the same rule when
its request carries `sessionKey`/`sessionId`; only legacy non-session-scoped
fallback sends retain the agent-only append.

## Direct media-custody boundary recheck — 2026-11-01

`OutgoingMediaCustody` in `source/host/src/attachment_paths.rs` now rolls back
materialized files when the `SendMessage` tool path fails and commits them
after the existing transcript entry receives an id. `ProductionSendMessageSink`
also captures a matching session key/lifecycle revision and rechecks that
generation immediately before transcript append. This proves a bounded
foreground tool-path fence.

The media settlement owner must retain both captured values at the final
transcript/receipt transaction: `session_key` identifies the canonical session
and `lifecycle_revision` identifies its generation. A matching revision alone
is insufficient evidence when the producer already has a session identity;
settlement must reject a rebound or mismatched session before clearing the
durable media fence.

The same captured `session_key` must be used for the ordinary generated
`SendMessage` settlement path when no media fence exists; it must not fall back
to the agent-only transcript append merely because no media receipt is needed.

Turn-local permission requests now carry the existing Runner
`worker_session_key` through the permission scope and event owner. Their
created card append and settled-card update must use session-admitted owner
transactions when that identity is present; permission requests from legacy
unbound infrastructure remain explicitly outside this session-scoped path.

The same producer boundary applies to Auto-review approvals created by an
active Runner. The Runner already has the canonical `worker_session_key` when
it binds the Auto-review controller, so that identity must be carried on the
approval event through the existing Auto-review service. Created approval
cards and settled/expired card updates must use the session-admitted owner
transactions when the key is present; controller instances created by legacy
unbound infrastructure remain on the compatibility path.

The owner now also carries one migration-safe `admitted_media_work_id` and
archive rejects that field while it is present. This closes the durable
archive fence for the bounded SendMessage path: the sink's process-local
`pending_media` remains keyed by `tool_call_id`, but it is paired with the
owner-local media identity and released only after transcript append or
explicit rollback. It still does not satisfy the full `SWA-03`/`SWA-06`
detached-result contract: the owner now writes a durable media settlement
receipt and has a bounded reopen recovery consumer, but full detached
result/media replay and the placement/database identity chain remain open. A
global in-memory registry or a second media lifecycle owner remains
unauthorized by this spec.

## Exact selector preflight — 2026-11-01

The session-owner preflight now fails closed when a producer supplies a
non-empty selector that differs from the existing admission record. A missing
record retains the existing compatibility behavior for callers that have not
yet established a session admission; once a record exists, a rebound or
mismatched `session_key` is an explicit identity error rather than an admitted
preflight. The atomic transcript/media owner checks remain authoritative at
the durable write edge.

## Agent-only transcript mutation boundary — 2026-11-01

The remaining transcript mutation producers were rechecked against the live
Rust source. `WidgetResponses` (`widget_responses.rs`), shared-room restamp
(`shared_rooms.rs`), box-request settlement (`box_request_entries.rs`), and
auto-review settlement (`auto_review_service.rs`) call the generic
agent-scoped update boundary and carry only `agent_id`; their event/request
models do not expose a canonical `session_key`. The generic delete and
conversation-clear wrappers currently have no production caller in the Host
source.

These paths remain explicitly pending for `SWA-01`/`SWA-06`. The implementation
must not synthesize `agent:{agent_id}:main`, infer a session from an unrelated
room or approval identity, or route them through the session-admitted update
transaction until their owning producer captures and propagates the real
canonical session identity. The existing agent-only compatibility boundary is
retained for these non-session-scoped producers; this is incomplete evidence,
not acceptance.

## Release and rollback

The schema must be backward-compatible with existing agent DBs and written
through the existing migration/transaction owner. Rollback removes the new
producer bindings while retaining readable state; no rollback may delete an
outcome-unknown transcript or result.

## References

- `projects/openclaw-rust/decisions/2026-11-01-session-work-admission-owner-contract.md`
- fixed upstream `src/sessions/session-agent-work-admission.ts`
- fixed upstream `src/sessions/background-session-result.ts`
- fixed upstream `src/sessions/session-archive-restore.ts`
- `source/host/src/extensions/session/production.rs`
- `source/host/src/extensions/session/agent_db.rs`
- `source/host/src/extensions/transcript/production_runtime.rs`
