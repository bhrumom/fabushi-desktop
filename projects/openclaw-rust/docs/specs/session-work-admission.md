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
  failure outcome. The owner-local admission record carries a migration-safe
  `deletionPending` fence while the existing lifecycle owner has deletion
  custody; the in-memory deletion marker remains a fast-path guard, not the
  durable authority. A failed delete clears the durable fence only after the
  existing agent directory remains available; an unknown cleanup outcome stays
  fail-closed.
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
`deleting_agents` remains the in-memory fast-path deletion fence; the durable
`deletion_pending` field now backs the same owner-local admission record when
the agent database is available. This closes the durable deletion-window
sub-contract only; it is not the complete detached-result lifecycle.

Target state is one owner-local session admission record containing canonical
`sessionKey`, `sessionId`, lifecycle revision, `deletionPending`, archive metadata, admitted
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

The existing atomic session-admitted transcript append, transcript update,
background-result receipt, and outgoing-media settlement transactions must
also reject the durable `deletion_pending` fence inside their SQLite
transaction; a production precheck alone is not sufficient for a delete race.

The automation spend-guard producer uses the same owner boundary for a
session-bound widget card: card insertion, spend-guard KV state, and activity
metadata commit together, or none commit. Legacy databases without a session
admission record retain their existing compatibility path and remain subject
to the facade admission checks, including direct spend-guard state responses.
Both spend-guard mutation entrypoints also reject before changing the
automation store, so deletion cannot leave a routine-side effect without its
owner-local state transition.

Host turn memory/profile prompt snapshot setters are likewise admitted through
the same owner fence. Their paired clear paths are intentionally not widened
here because one production memory-clear caller mutates the external memory
store before clearing the SQLite snapshot and needs a separate atomic contract.

The file-backed agent profile and avatar mutations remain a separate atomicity
boundary. Profile updates write `profile.json`; avatar replacement or removal
also deletes or writes conventional avatar files and, on legacy removal, may
clear an SQLite avatar pointer. The send-acceptance name seeding path is a real
producer of the profile update. A facade admission check alone cannot make
these filesystem and SQLite effects one owner transaction, so this slice does
not claim their deletion-race closure. They remain pending until the existing
session owner has a deletion-custody protocol that can either serialize the
whole mutation or define and verify an explicit rollback/unknown-outcome
contract.

The same fence covers owner-local viewed/unread projections, the persisted
session profile projection, conversation-partner membership, and profile
snapshot clearing. These are session metadata mutations, not alternate
lifecycle owners.

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
  non-actionable phase and clear the matching terminal work marker; Runner
  success: clear the matching review and work marker.

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
wrappers, activity metadata writes, and awaiting-user-response metadata writes
plus introduction lifecycle metadata while `begin_agent_delete` owns the
deletion window. `SandAgentDb`
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

The provider-review durable transitions now also recheck the owner-local
`deletion_pending` fence at their SandAgentDb edge. Recording a refusal,
releasing a claimed continuation, or settling its terminal outcome cannot
mutate review state after deletion custody begins; the existing generation and
CAS semantics remain unchanged. This is a bounded deletion write-edge
correction, not completion of native continuation identity or the full
detached lifecycle.

The same owner-local transaction fence now covers request-id history and
pending episode-memory writes, including the Runner's direct episode clear
path, as well as the owner-local transient-state clear. A deletion fence
therefore rejects those metadata mutations at the SQLite write edge; this
now also covers metadata CAS writes used by transcript/checkpoint owners.
These remain bounded owner-local corrections and do not close the separate
checkpoint blob, profile/avatar, or external memory-store atomicity gaps. The
checkpoint transaction now rejects confirmation under the durable fence, while
the standalone latest-root CAS is fenced as well. The already-materialized
external blob remains a separate cleanup/recovery responsibility. Session
maintenance now routes its persisted-root CAS through the same `SandAgentDb`
owner instead of maintaining a second direct SQLite mutation path.
Runtime profile-name recovery maintenance now uses the same owner metadata
path as well; clone/materialization initialization remains a separate
pre-admission setup path.

The owner-local admission CAS now preserves the current `deletion_pending`
value as part of its compare predicate. Ordinary work/review/media mutations
cannot clear a fence that appeared after their read; only the deletion owner's
dedicated clear operation may transition `true` to `false`, and it requires
the fence to still be present at the CAS edge.

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

The same owner boundary applies when a persisted `request_box_help` entry is
superseded or settled after the Runner returns. Such entries retain the
producer-owned `sessionKey` provenance, and `resolve_box_request_entry` must
use the atomic session-admitted update when it is present. Legacy entries
without that provenance remain on the compatibility path and retain the
generic deletion/archive fence.

The session gateway's accepted-send producer now persists the supplied
`sessionKey`/`sessionId` on staged transcript entries and applies the same
owner-local append when that selector is present; only legacy non-session-scoped
fallback sends retain the agent-only append.

Session-admitted transcript updates must validate the target entry's persisted
`sessionKey` inside the same owner transaction as the admission and update.
Entry existence alone is insufficient: a caller must not use one admitted
session key to mutate a legacy or differently bound entry in the same agent
database after an out-of-transaction provenance read.
The corresponding session-admitted append owner applies the same invariant to
every inserted entry; missing or mismatched provenance is rejected in the
append transaction rather than silently converted into a session-bound row.

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
`SendMessage` settlement path when no media fence exists, and the generated
entry must retain that key as provenance; it must not fall back to the
agent-only transcript append merely because no media receipt is needed.

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

The Runner `SendToAgent` producer also already owns the canonical
`worker_session_key`. Its sender-side transcript append must persist that key
on the outbound entry, carry it through the existing agent-management sink, and
use the session-admitted owner transaction only when the owner-local admission
record confirms that key. If
the producer carries a selector that has no matching owner record (for
example, the current group-member compatibility runner), it remains on the
generic agent-only path until that session record is established. The
recipient-side wake envelope does not expose a canonical target session
identity; its later inbound materialization resolves the target's existing
owner record locally and uses the admitted append when available, otherwise it
retains the generic compatibility path. It is still subject to the owner-local
deletion and archive fences. The producer
must not synthesize a target `agent:{id}:main` selector merely from a target
agent id.

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

The owner-local compare-and-set also preserves `session_key` and `session_id`
across generation updates. A caller may advance lifecycle state and admission
facts, but may not use a valid prior revision to rebind the record to another
session identity; such a CAS returns false and leaves the previous record
intact.

Archive and restore use the same exact identity at their durable owner edge:
the canonical `session_key`, `session_id`, and optional lifecycle revision are
all checked inside the SQLite transaction before archive state changes.

## Agent-only transcript mutation boundary — 2026-11-01

The remaining transcript mutation producers were rechecked against the live
Rust source. `WidgetResponses` now preserves and consumes `sessionKey`
provenance for interactive updates to bound cards; cards without provenance
remain on its generic agent-scoped compatibility path. Its stale-card sweep
now uses the same owner-local transaction for cards carrying provenance, while
legacy cards without it retain the compatibility path. Shared-room restamp
now uses the same owner-local transaction when the persisted entry already
carries a canonical `sessionKey`; legacy entries without provenance remain on
the compatibility path. New room-owned entries from local group fanout,
activity notices, guest ingress, and mirror ingress now read the target
room-agent's owner admission record, persist that exact `sessionKey`, and use
the session-admitted append when the record exists; a missing record retains
the compatibility path. The generic delete and conversation-clear wrappers
currently have no production caller in the Host source. Automation spend-guard
cards now use the same owner record and session-admitted append when available;
unbound legacy card producers remain compatibility-only. Recipient-side agent
inbound materialization likewise reads the target agent's existing owner
record, persists its exact `sessionKey`, and uses the admitted append when
available; legacy targets without a record retain compatibility behavior.

The box-request path is now split by durable provenance: new Runner-created
entries retain their canonical `sessionKey`, and
`resolve_box_request_entry` uses the atomic session-admitted update for those
entries; older entries without provenance retain the compatibility path. The
Auto-review service similarly uses its event-owned session key for bound
creation and settlement, while unbound legacy controller events remain
compatibility-only.

The still-unbound paths remain explicitly pending for `SWA-01`/`SWA-06`. The
implementation must not synthesize `agent:{agent_id}:main`, infer a session
from an unrelated room or approval identity, or route them through the
session-admitted update transaction until their owning producer captures and
propagates the real canonical session identity. The existing agent-only
compatibility boundary is retained for these non-session-scoped producers, but
it is not an archive bypass: generic transcript and compaction wrappers reject
writes when the owner-local admission record is archived. This is incomplete
evidence, not acceptance.

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
