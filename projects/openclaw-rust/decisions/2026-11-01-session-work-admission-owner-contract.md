# ADR: Session work-admission owner contract

Status: bounded owner slices implemented; full responsibility pending
Date: 2026-11-01
Scope: Fabushi Desktop Host session, transcript, and Runner admission
Responsibility: `OCRM-RESP-SESSION-WORK-ADMISSION-001`

## Context

The fixed upstream responsibility is broader than a boolean “agent is busy”
check. `session-agent-work-admission.ts` fences writes while agent deletion
cleanup is pending; `background-session-result.ts` settles a detached result
only against the admitted session generation; `session-archive-restore.ts`
restores an archived generation only after placement and lifecycle rechecks;
and provider-review modules retain terminal review ownership across a turn.

The current Fabushi tree has `ProductionSessionWorkers` and per-agent SQLite
transcript ownership, including a lifecycle revision. It now also has an
owner-local `sessionWorkAdmission` record, a generation-bound local background
result receipt, an authenticated canonical-main archive/restore ingress, a
durable outgoing-media fence, and provider-review admission/acknowledgment
state. These are bounded production slices, not a complete session catalog or
the full upstream detached lifecycle. The existing `deleting_agents` mutex
remains only the deletion-window fence and is not a replacement for the
durable session-row contract.

Boundary recheck on 2026-11-01: the existing background-revival producer now
captures those routing facts, admits work against the current generation,
rechecks the matching terminal event, and commits an idempotent hidden receipt
through the same owner. Duplicate and conflicting payloads are explicit. The
remaining gap is the broader upstream detached-result/session catalog contract,
full media/result replay, native provider-review runtime recovery, and exact
production verification. A bounded provider-review terminal
settlement/consumption path is now wired after continuation dispatch.

## Decision

Keep all work-admission state inside the existing `ProductionSessionWorkers` →
`SandAgentDb` owner chain. Do not create a second workboard, session database,
cloud-agent archive shim, or generic “archive” flag on the agent profile.

The remaining session model must stay in the existing owner-local record rather
than introduce another authority. Its required fields are:

- canonical `sessionKey` and durable `sessionId`;
- `lifecycleRevision` with compare-and-set transitions;
- optional `archivedAt`, `archivedBy`, and `archiveReason`;
- the admitted background-work identity and terminal-result idempotency key;
- provider-review identity and terminal phase when a review is active.

The owner API must provide four guarded operations:

1. admit or continue work only when the captured session generation is current
   and not archived or deleted;
2. commit a detached background result exactly once against that generation;
3. archive and restore with a lifecycle/placement guard at the actual commit;
4. retain or settle provider-review terminal facts without allowing a retry or
   fallback runtime to claim a different physical turn.

Each operation must return an explicit stale, archived, deleted, duplicate, or
   outcome-unknown result. Unknown deletion or transcript outcomes must not be
   retried as if they were rejected. Existing transcript and Runner owners
   remain the only consumers of admitted work and terminal output.

## Producer → owner → state → consumer

`Gateway/Runner/background producer → session admission owner → per-agent
SQLite session record + transcript transaction → existing Runner/transcript
projection and provider-review consumer`.

The current `sessions.delete` path may later publish the deletion fence through
this owner, but it must not bypass the durable generation checks. Cloud-agent
`archive`/`unarchive` actions are not session work-admission producers and must
not be reused for this responsibility.

## Required implementation sequence

The first slice is implemented: `ProductionSessionWorkers` now rejects new
transcript, compaction, and conversation-clear writes while its existing
agent-deletion owner holds the deletion window. `SandAgentDb` also persists a
compare-and-set `sessionWorkAdmission` record carrying session identity,
generation, archive metadata, admitted work, result idempotency, and review
phase fields. The current producer set populates only bounded slices and
fails closed for the remaining lifecycle cases; this is not the complete
session-generation/archive/review implementation.

1. Add the owner-local schema/API and migration-safe transaction to
   `SandAgentDb`/`ProductionSessionWorkers`; preserve existing agent DBs.
2. Extend the existing Runner admission and background-result settlement only
   where the captured generation and durable receipt are available.
3. Complete the detached-result/session catalog lifecycle and provider-review
   terminal recovery/settlement contract through existing authenticated owners; do not add a new
   session authority.
4. Add or retain focused negative contracts for archived admission, deletion pending,
   stale generation, duplicate result, lifecycle change during archive/restore,
   active media/subagent preservation, and terminal review ownership.
5. Reconcile the parity ledger only after non-test producer, durable state, and
   consumer edges are all present. Verification and acceptance remain pending
   until the frozen exact-head lane can run.

The current Rust producer captures the active main-session
`lifecycleRevision` before background routed admission, rechecks it at the
matching `Completed` terminal event, and commits the bounded owner-local
receipt. It still does not close the complete detached-result/session catalog,
media replay, or provider-review terminal lifecycle.

The owner now contains a further bounded slice: the local background-revival
terminal path commits a hidden `background-result-receipt` transcript entry
and the admission-marker transition in one `SandAgentDb` SQLite transaction.
The receipt is idempotent by key and rejects a conflicting payload. The
responsibility remains open because this slice is not yet the complete
upstream detached-result contract and does not provide authenticated
archive/restore, media replay ownership, or provider-review terminal wiring.

The transaction boundary now explicitly rolls back both the receipt insert and
any trigger-side admission mutation when the post-insert lifecycle CAS returns
`RejectedStale`; this prevents an orphan receipt from converting a later stale
retry into `AlreadyCommitted`. The focused negative oracle remains pending
until the frozen exact-head verification lane is available.

The next implementation must extend the same owner-local receipt and session
record to the remaining detached-result and terminal-review cases. Clearing an
admitted-work marker without retaining the original result remains forbidden
because it cannot distinguish duplicate, conflicting-payload, stale-generation,
and outcome-unknown settlement.

The Runner `request_box_help` producer now uses the owner-local
session-admitted transcript append with its already-captured canonical session
key. This closes that producer's archive/identity write edge without creating a
new handoff or transcript owner; the broader detached-result lifecycle remains
open.

The persisted `request_box_help` entry now retains that producer-owned session
key through later supersede/settlement. `resolve_box_request_entry` uses the
same atomic session-admitted update for entries carrying the key, while older
entries without provenance remain on the compatibility path.

The accepted-send session gateway persists its supplied `sessionKey` or
`sessionId` on staged transcript entries and applies the same owner-local
append whenever that selector is present; legacy sends without a session-scoped
selector retain the existing agent-only compatibility path.

The session-admitted transcript update owner also validates the target entry's
persisted `sessionKey` inside its SQLite transaction. Presence of an entry is
not enough, because an out-of-transaction provenance read must not allow one
admitted session to mutate a legacy or differently bound entry in the same
agent database.
The session-admitted append owner likewise requires every inserted entry to
carry the exact admitted `sessionKey` inside the same transaction; it rejects
missing or mismatched provenance instead of silently creating a session-bound
row from an unbound payload.

The bounded outgoing-media settlement now carries the producer's canonical
session key through `ProductionSendMessageSink`, `ProductionTranscriptRuntime`,
`ProductionSessionWorkers`, and `SandAgentDb`; the final receipt/fence
transaction rejects a matching-generation request from a rebound session.
The no-media generated `SendMessage` branch now uses the same session-admitted
append and persists the captured session key on the generated entry, so media
presence no longer determines whether the write is fenced or loses provenance.

Pending-media reopen recovery now canonicalizes each still-present journaled
target against the agent's canonical attachment/assets roots, rejects symlinked
or outside-root paths, and gives only the verified target to the existing
custody rollback. Missing targets are treated as already rolled back; any
inspection or identity uncertainty retains the media fence and remains
outcome-unknown. This tightens recovery safety without claiming the separate
full read-scope capability or descriptor-lifecycle responsibility.

External-memory reopen recovery also compares the current owner-local memory
snapshot with the journaled pre-mutation snapshot before restoring records.
A rebound snapshot now retains the journal and owner fence as outcome-unknown;
the recovery path no longer treats the journal identity alone as sufficient.

The turn-local permission path now carries `worker_session_key` through its
permission scope and event record. Created and settled permission transcript
rows use session-admitted owner transactions when that key is present; legacy
unbound permission infrastructure remains on the explicitly compatibility-only
agent append/update path.

The active Runner's Auto-review controller now has the same producer fact: the
Runner already owns the canonical `worker_session_key` at controller binding,
so approval events can retain it without deriving a selector from `agent_id`.
Created approval cards and settled/expired card updates must use the existing
session-admitted owner transactions when that identity is present; legacy
controller instances remain compatibility-only.

The session preflight also now rejects a non-empty requested selector when an
existing admission record carries a different canonical key. Returning success
for that mismatch was weaker than the owner contract even though later atomic
transcript writes rejected it; the deterministic preflight negative is now
explicit. Missing admission records retain the legacy compatibility behavior
until their producer establishes the session record.

The shared SQLite write predicate now binds an existing admission record's
`session_id` to the owner metadata `agentId` at the same write edge. A malformed,
missing, or rebound metadata identity fails closed; databases without an
admission record retain the legacy compatibility path.

`ProductionSessionWorkers::open_agent_db_owner` performs the same identity
check before either cached-owner recovery or first-open media/profile/memory
recovery, so recovery cannot run against a rebound admission record and only
later fail at the read facade.

The owner-local compare-and-set now applies the same invariant to updates:
advancing a valid lifecycle revision cannot replace the stored `session_key` or
`session_id`. An attempted identity-rebinding CAS is rejected without changing
the prior admission record.

The profile/avatar and external-memory mutation-journal clear transactions
also re-run the shared owner identity-admission predicate before deleting a
journal. Archived, deletion-pending, malformed, or rebound owner state
therefore retains the recovery journal and fails closed instead of treating
cleanup itself as an admitted write.

Owner-open recovery now shares the per-agent filesystem/deletion serialization
boundary with profile/avatar and external-memory producers. The public owner
open path acquires that boundary before identity validation and media/profile/
memory journal recovery; producers already inside the boundary call the
non-reacquiring internal open path. This prevents recovery from interleaving
with directory deletion or another filesystem mutation without creating a
second lifecycle lock or owner.

The same boundary now covers conversation clearing across its external blob
cleanup and owner-local transcript transaction. This is an in-process
deletion-race closure only; the existing cross-storage crash window remains
explicitly pending because no second blob journal or recovery owner was added.

Channel credential/configuration producers that touch the agent directory now
use the same boundary and recheck work admission before writing. Their existing
secret-file and channel-config owners remain separate stores; no cross-store
transaction or new channel authority was introduced.

The public production admission facades now use the same boundary, while
filesystem-backed producers call explicit locked helpers when composing the
check with their mutation. Admission also fails closed when the canonical
session database is gone, preventing post-delete writes from recreating an
agent directory; no recursive mutex acquisition is relied upon.

Production Gateway and Runner settings writes now use the existing session
owner. The Runner's state surface receives an injected settings writer, while
the standalone state fallback remains limited to non-production construction;
no second settings authority was introduced.

The production Runner channel-disconnect command now receives the existing
channel owner as an injected callback; the standalone state fallback remains
compatibility-only and does not establish a second channel authority.

Gateway automation/workflow definition mutations now compose their existing
stores with the admitted per-agent filesystem boundary. This protects the
agent directory during deletion without creating an automation session catalog,
reaper, or alternate lifecycle owner.

The production Runner state surface now delegates automation/workflow mutation
commands through injected callbacks to those existing Session owner methods;
the standalone state fallback remains compatibility-only.

The spend-guard state setter and the atomic session-bound card/state/activity
transaction now perform the same predicate check inside their SQLite
transactions. A facade admission precheck can no longer be the only fence for
direct spend-guard state mutation or card creation.

The generic metadata owner also rejects an `agentId` value that does not equal
the canonical agent-directory identity before entering its CAS transaction.
The clone-specific pre-admission reseed helper remains the only separate
identity initialization path; normal metadata updates cannot create a rebound
owner identity and rely on a later reopen to discover it.

Session maintenance's legacy-blob retirement and repair/stale-root version
markers now use the existing `SandAgentDb` owner. Retirement checks the shared
admission predicate in the same transaction as blob cleanup and its marker;
version markers use the owner KV transaction, so maintenance cannot publish
session-owned cleanup state after archive or deletion custody wins.

The archive/restore owner now receives and checks both canonical
`session_key` and `session_id` inside its transaction, so a producer cannot
pass a valid session id with a different session selector and still mutate
archive state.

Provider-review terminal settlement now clears the matching
`admitted_work_id` in the same owner CAS as the terminal review phase. This
prevents a known terminal, cancellation, failure, or outcome-unknown
continuation from leaving a stale active-work fence after Runner ownership has
ended.

The remaining agent-only transcript mutation producers were rechecked against
the live source. Interactive WidgetResponses updates and stale-card sweeps now
consume a persisted `sessionKey` when a card has one; cards without provenance
remain on the compatibility path. Shared-room restamps now use the same
session-admitted owner when the persisted entry carries `sessionKey`; legacy
entries remain on the compatibility path. New box-request entries retain
`sessionKey` and settle through the session-admitted owner when that provenance
is present. SharedRooms and the local group fanout now apply the same producer
rule to new room-owned entries from local fanout, activity notices, guest
ingress, and mirror ingress: they read the target room agent's existing owner
record, persist its exact key, and use the admitted append when available.
Automation spend-guard card creation now follows the same owner rule, so its
interactive cards persist the existing session key and are rejected when the
owner is archived; unbound legacy producers remain compatibility-only.
Recipient-side agent inbound materialization now follows the same rule when
the target agent already has an owner record: it persists that exact session
key and uses the session-admitted append, while targets without a record stay
on the compatibility path.
Auto-review
creation and settlement already use the event-owned session key when bound;
unbound legacy controller events remain compatibility-only. The generic delete
and conversation-clear wrappers have no production caller in the Host source.
The still-unbound paths remain pending rather than receiving a guessed
`agent:{agent_id}:main` selector. A future change must first make the owning
producer capture and propagate its canonical session identity, then add the
corresponding atomic owner contract and negative stale/archive oracle.
The generic compatibility wrappers now also reject writes against an archived
owner record, so retaining agent-only identity does not bypass the archive
fence.

The Runner `SendToAgent` sender-side transcript producer now persists its
already-owned `worker_session_key` on the outbound entry, carries it through
`ProductionAgentManagementSink` and `ProductionAgentToAgentMessaging` into the
session-admitted append only when the owner-local admission record confirms
that key. A group-member worker
whose group selector has no matching owner record remains on the generic
archive-fenced compatibility path until that record exists. The
recipient wake envelope still carries no canonical target session
selector, so inbound materialization resolves the target's existing owner
record locally rather than deriving an `agent:{id}:main` key. Targets with a
record use the session-admitted append; targets without one remain on the
generic compatibility path. The generic owner fence still rejects
deletion-pending and archived recipient writes.

## Rejected alternatives

- Treating `deleting_agents` or filesystem age as durable work admission.
- Reusing cloud-agent archive/review actions as session lifecycle authority.
- Adding a second workboard/session database or a provider-specific result
  store.
- Marking the responsibility implemented from contract fixtures alone.

## Evidence and references

- Fixed upstream `ad8d12f126d29b0db204187adf5092db80294e70` / tree
  `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.
- `src/sessions/session-agent-work-admission.ts` blob
  `7da9e10faaec1bf94044830f7d39d9f65ef3d68a`.
- `src/sessions/background-session-result.ts` blob
  `a70b7b04de18bdba5216ece63e50928cd2e6eea2`.
- `src/sessions/session-archive-restore.ts` blob
  `45972689946f442e89561da1a26f807b3432b694`.
- `projects/openclaw-rust/docs/specs/session-work-admission.md`.
- `source/host/src/extensions/session/production.rs`.
- `source/host/src/extensions/session/agent_db.rs`.
- `source/host/src/extensions/transcript/production_runtime.rs`.
