# ADR: Session work-admission owner contract

Status: architecture selected; implementation pending
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
transcript ownership, including a lifecycle revision, but it does not have a
durable session-entry catalog with archived state, a background-result
generation contract, or a production archive/restore/review ingress. The
existing `deleting_agents` mutex is an in-memory agent-deletion marker, not a
replacement for those session-row contracts.

Boundary recheck on 2026-11-01: the existing background-revival producer does
have the necessary routing facts (`agentId`, canonical main `sessionKey`,
`clientNonce`, `streamId`) and receives a matching terminal event. Its current
consumer is still an in-memory wait plus transcript before/after counting and
telemetry. Because the owner record cannot return a committed terminal receipt
for duplicate or conflicting idempotency keys, this is evidence for the
missing durable consumer, not evidence that the responsibility is wired.

## Decision

Keep all work-admission state inside the existing `ProductionSessionWorkers` →
`SandAgentDb` owner chain. Do not create a second workboard, session database,
cloud-agent archive shim, or generic “archive” flag on the agent profile.

The minimum durable extension required before this responsibility can be
implemented is one owner-local session admission record containing:

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
phase fields. This is still deliberately not the production
session-generation/archive/review implementation described below: no
authenticated producer currently populates or consumes all of those fields.

1. Add the owner-local schema/API and migration-safe transaction to
   `SandAgentDb`/`ProductionSessionWorkers`; preserve existing agent DBs.
2. Add the existing Runner admission call at the one canonical turn boundary,
   plus background-result settlement using the captured generation.
3. Add authenticated archive/restore and provider-review producers only where
   the existing Gateway/session owner already supplies identity, placement and
   lifecycle authority.
4. Add focused negative contracts for archived admission, deletion pending,
   stale generation, duplicate result, lifecycle change during archive/restore,
   active media/subagent preservation, and terminal review ownership.
5. Reconcile the parity ledger only after non-test producer, durable state, and
   consumer edges are all present. Verification and acceptance remain pending
   until the frozen exact-head lane can run.

The current Rust producer now captures the active main-session
`lifecycleRevision` before background routed admission and rechecks it at the
matching `Completed` terminal event. This is a generation-observation guard,
not the required durable result receipt: it still cannot replay an original
committed outcome for a duplicate idempotency key and does not close archive,
restore, or provider-review ownership.

The owner now contains a further bounded slice: the local background-revival
terminal path commits a hidden `background-result-receipt` transcript entry
and the admission-marker transition in one `SandAgentDb` SQLite transaction.
The receipt is idempotent by key and rejects a conflicting payload. The
responsibility remains open because this slice is not yet the complete
upstream detached-result contract and does not provide authenticated
archive/restore, media replay ownership, or provider-review terminal wiring.

The next implementation must bind the captured lifecycle revision and
idempotency key to a durable terminal receipt at the existing session/transcript
owner. Clearing the admitted-work marker without retaining the original result
is insufficient because it cannot distinguish duplicate, conflicting-payload,
stale-generation, and outcome-unknown settlement.

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
