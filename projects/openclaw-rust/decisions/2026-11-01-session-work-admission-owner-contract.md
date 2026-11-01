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

The accepted-send session gateway applies the same owner-local append whenever
its request carries `sessionKey` or `sessionId`; legacy sends without a
session-scoped selector retain the existing agent-only compatibility path.

The bounded outgoing-media settlement now carries the producer's canonical
session key through `ProductionSendMessageSink`, `ProductionTranscriptRuntime`,
`ProductionSessionWorkers`, and `SandAgentDb`; the final receipt/fence
transaction rejects a matching-generation request from a rebound session.
The no-media generated `SendMessage` branch now uses the same session-admitted
append, so media presence no longer determines whether the write is fenced.

The turn-local permission path now carries `worker_session_key` through its
permission scope and event record. Created and settled permission transcript
rows use session-admitted owner transactions when that key is present; legacy
unbound permission infrastructure remains on the explicitly compatibility-only
agent append/update path.

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
