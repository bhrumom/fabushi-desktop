# Session lifecycle deletion fence

Status: implementation in progress; verification not run
Project: OCRM-001  
Current live discovery subject: `openclaw/openclaw@447e14d02460342af4fe2c98dbd5475b15b938d2`
Current live discovery tree: `3b066eb5f2eddd08ea7a74f124395248fbdcd384`
Status: discovery-only; the historical session delta below was read at
`1b870b2c78e81fded72913ecba732b84b611ca12` / tree
`199fe97259d0226ce0e4d3dcec603252376c9205` and requires exact-head
revalidation before final authority or acceptance.
Owner boundary: existing Host Gateway → Transcript/Session lifecycle owner

## Context and problem

The current OpenClaw session deletion contract binds a destructive delete to the
captured session identity and lifecycle state. Callers pass `expectedSessionId`,
`expectedLifecycleRevision`, and, for cron cleanup, `expectedSessionUpdatedAt`.
The Gateway reads and rechecks the same session row under its lifecycle/storage
owner; a mismatch returns the typed `session-changed` invalid-request outcome.
Subagent cleanup also retains the captured Gateway owner/currentness fence across
async dispatch.

Fabushi's current `sessions.delete` path only compares `expectedSessionId` with
the derived agent ID, then dispatches `deleteAgent`. The Rust agent summary has
`updated_at` but no authoritative lifecycle revision, and deletion does not yet
recheck a captured revision after cleanup/drain work. Replacing the missing
revision with a timestamp would be an unproven contract change.

## Goal

Make the existing Rust Host session deletion owner reject stale destructive
cleanup and preserve the current owner boundary, with an authoritative durable
lifecycle revision exposed to session projections and consumed by cron/subagent
cleanup callers.

## Non-goals

- No second session database, cleanup worker, Gateway owner, or transcript owner.
- No change to public session-key classification or harness namespace policy.
- No automatic retry after a lifecycle conflict or unknown deletion outcome.
- No provider, UI, channel, package, or release acceptance claim from unit-level
  implementation evidence.
- No use of `updated_at` as a substitute for lifecycle revision.

## Requirements

### SLD-01 — Durable revision owner

The existing per-agent/session durable state owner must hold a non-empty,
opaque lifecycle revision. Existing stores without the field receive a
backward-compatible migration/default exactly once. Revision publication is
atomic with the lifecycle mutation that changes the session identity/state.

### SLD-02 — Projection

`sessions.list` and `sessions.get` projections expose the current revision as
`lifecycleRevision`. Projection failure or missing/invalid revision is explicit
incomplete evidence and must not silently produce an empty revision.

### SLD-03 — Guarded delete admission

The `sessions.delete` Gateway owner validates, before destructive work:

- `expectedSessionId`, when supplied, matches the current row;
- `expectedLifecycleRevision`, when supplied, matches the current row;
- `expectedSessionUpdatedAt`, when supplied, matches the current current-row
  activity/update value under the same owner.

Any mismatch returns the existing `SESSION_LIFECYCLE_CHANGED` / `session-changed`
error shape and performs no delete side effect.

### SLD-04 — Post-drain recheck

After active-run cleanup, transcript/media drain, or other asynchronous
pre-delete work, the owner re-reads the same row and revalidates session ID,
lifecycle revision, owner identity, and deletion authority immediately before
the destructive commit. A stale owner cannot delete a successor session.

### SLD-05 — Cleanup consumers

Cron isolated-session cleanup and subagent cleanup pass the captured identity and
revision through the existing Gateway call boundary. They map a lifecycle
conflict to `changed`; they do not retry or report deletion success. Existing
isolated retirement/survival behavior remains owned by the current cleanup
caller for non-lifecycle transport failures.

For generated subagents specifically, launch must capture the child's current
`lifecycleRevision` from the session projection and retain it through the
existing `SubagentRuntime` completion record. Completion cleanup may invoke the
existing Host lifecycle owner only after the child run has settled and the
parent completion/revival handoff has been admitted. The cleanup callback must
be injected from the Gateway owner; `SubagentRuntime` must not acquire a second
session/deletion owner or call storage directly.

### SLD-06 — Ownership and safety

The implementation must preserve the existing Host Gateway, Transcript/Session
storage, Runner generation, and plugin Gateway-context owners. No unauthenticated
fallback, alternate deletion path, or stale callback may bypass the guard.

## Current and target state

| Concern | Current state | Target state |
| --- | --- | --- |
| Session identity | `expectedSessionId` is compared to derived agent ID | Compared to the authoritative current session row |
| Lifecycle revision | Absent from Rust AgentSummary/storage projection | Durable opaque revision in existing session owner |
| Updated-at guard | Accepted upstream but ignored by Rust delete | Compared under the same storage owner when supplied |
| Async cleanup | `deleteAgent` runs after a shallow admission check | Drain/cleanup then immediate guarded commit recheck |
| Cron/subagent consumer | Current upstream carries revision/currentness | Rust callers carry the same guarded contract |

## Architecture and data flow

```text
session list/get projection
        │ captures sessionId + lifecycleRevision (+ updatedAt)
        ▼
cron/subagent cleanup producer
        │ existing Gateway request boundary
        ▼
Host sessions.delete owner
        │ read/authorize → drain/cleanup → re-read/revalidate
        ▼
existing Transcript/Session durable delete owner
        │ commit + lifecycle event
        ▼
existing roster/automation/transcript consumers
```

The revision must be owned by the same durable session state that authorizes
deletion. A projection, cache, filesystem mtime, or test fixture cannot become a
second authority.

## Failure modes and edge cases

- Missing/blank expected revision: reject when the caller declares the guarded
  contract requires it; do not coerce it to a timestamp or generated fallback.
- Session deleted/recreated between admission and commit: return
  `session-changed`; do not delete the replacement.
- Lifecycle revision changes during drain: return `session-changed`; preserve
  the newer row and report no successful deletion.
- Owner/Gateway context closes or is replaced: return the existing owner-current
  error; do not retry under a new owner implicitly.
- Main/global/harness session deletion remains rejected by existing policy.
- Existing stores with no revision require an explicit migration/default path;
  a read-only synthetic value is insufficient for destructive admission.
- Deletion side effect has an unknown outcome: preserve existing unknown-result
  handling and do not replay automatically.

## Implementation strategy

1. Add an approved storage-level revision field and migration/default logic to
   the existing agent/session DB owner.
2. Thread the revision through `AgentMetadataProjection`, `AgentSummary`, and
   `sessions.list/get` without changing the owner boundary.
3. Add a typed guarded-delete input/result helper in the existing lifecycle
   owner; use it for preflight and post-drain commit checks.
4. Forward guarded fields from the Host Gateway handler and existing cleanup
   producers. Keep conflict mapping at the caller boundary.
5. Add focused contract oracles for stale session ID, stale revision, stale
   updated-at, mutation-during-drain, owner replacement, and successful control
   deletion. Do not claim execution during the development freeze.

## Verification and acceptance

Required evidence is proportional and ordered:

- Static source review proves one durable revision owner and all producer →
  Gateway owner → storage commit → consumer edges.
- Contract oracles prove every SLD-03/04 negative case and the successful control
  case; deliberate removal of the post-drain check must make the oracle fail.
- Exact-head Actions receipt is required before `verification=verified`.
- Packaged/runtime and real cron/subagent execution evidence are required before
  `wiring=wired` is interpreted as full production reachability or acceptance.
- No global flag changes follow from this spec or a local implementation.

Definition of Done for this task is SLD-01 through SLD-06 implemented in the
existing owners, focused oracle sources present, exact source identity recorded,
and all required verification/acceptance states honestly represented. Until
those gates exist, the responsibility remains pending.

## Current implementation checkpoint

The first Rust implementation slice now exists in the existing owners:

- `SandAgentDb` seeds or CAS-migrates `lifecycleRevision` in the existing
  metadata row and advances it with metadata mutations, checkpoint-root
  publication, compare-and-set root publication, and conversation clearing.
- `AgentMetadataProjection`, `AgentSummary`, `sessions.list`, and
  `sessions.get` expose the captured revision.
- The existing `deleteAgent` owner accepts the guarded fields and rechecks the
  current summary both before deletion admission and after deletion hooks.

This checkpoint is not yet DoD: focused oracles have not been executed, the
cron/subagent production producers are not yet proven to pass the fields, and
exact-head Actions/package/runtime evidence is absent. Ledger verification and
acceptance states must remain pending until those checks are available.

## Release, migration, rollback, and observability

The migration must be backward-compatible for existing agent DBs and preserve
the previous deletion policy when no guarded caller is used. Rollback removes
the new caller contract only after no producer depends on it; it must not delete
or rewrite stored transcript state. Guard conflicts emit the existing typed
error/diagnostic and identify the session key without logging secrets or full
session content.

## References

- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`
- `projects/openclaw-rust/dossiers/upstream-current-session-lifecycle-delta-20261031.md`
- `source/host/app/src/main.rs:3581-3645`
- `source/host/src/extensions/transcript/agent_lifecycle.rs:601-635`
- `source/host/src/extensions/session/session_summaries.rs:52-85`
- Upstream exact paths: `src/config/sessions/lifecycle.ts`,
  `src/cron/isolated-agent/session-cleanup.ts`,
  `src/agents/subagents/registry/subagent-session-cleanup.ts`
