# OpenClaw Current Candidate — Session Lifecycle Delta

**Observed at**: 2026-10-31T08:09:58Z  
**Repository**: `openclaw/openclaw`  
**Exact discovery subject**: `1b870b2c78e81fded72913ecba732b84b611ca12`  
**Tree**: `199fe97259d0226ce0e4d3dcec603252376c9205`  
**Status**: discovery-only; not accepted source authority

## Scope and provenance

The exact compare from the preceding candidate
`db59290c33e1768cce8a461e2697cff7108c5a69` returned seven files. This dossier
records only the three runtime session/cron paths that intersect the existing
OCRM session and automation responsibility boundary:

| Path | Blob at exact subject | Observed delta |
| --- | --- | --- |
| `src/config/sessions/lifecycle.ts` | `7e3c7a6999cb44417ec634b0e77d1299c672f8c1` | Exposes lifecycle-change error classification and terminal-main-session timestamp checks. |
| `src/cron/isolated-agent/session-cleanup.ts` | `5cdeb385daf097dd9d7b87febdc7178dd4a91050` | Deletes cron sessions through guarded `sessions.delete`, passing expected session ID, lifecycle revision and update timestamp; distinguishes changed, retired and survived outcomes. |
| `src/agents/subagents/registry/subagent-session-cleanup.ts` | `1669914996a02fcecd45012811d0a42f18bd32d7` | Retains the captured Gateway/owner context, checks current generation/lifetime before dispatch and maps lifecycle conflicts to `changed`. |

The other four changed files are PDF provider/test support and an assertion/test
allocation file. They are not treated as session-runtime evidence here.

## Producer → owner → state/fence → consumer

The new session cleanup path has an explicit guarded flow:

1. The cron or subagent cleanup producer supplies `sessionId` plus the captured
   lifecycle revision (and, for cron cleanup, the observed update timestamp).
2. The existing `sessions.delete` Gateway owner receives those expected values;
   deletion is conditional on the session still being the captured lifecycle.
3. The Gateway context lifetime/current-owner fence is checked before and after
   the dispatch boundary. A lifecycle conflict becomes the typed `changed`
   outcome rather than a successful deletion.
4. The cron caller chooses `deleted`, `retired`, `survived` or `changed`; the
   subagent caller reports `deleted`, `changed` or `failed`.

This is stronger than a session-key classifier or a local cleanup helper: it
binds destructive session deletion to session identity, lifecycle revision and
the current Gateway owner.

## Fabushi Rust boundary audit

The existing Rust ledger rows cover `classify_session_kind`, session conversation
state and the pending cron session-reaper owner, but the current accepted
baseline does not contain this exact upstream delta. The inspected Rust paths do
not provide current-subject evidence for all of the following as one production
chain:

- a revisioned `sessions.delete` admission contract;
- expected-session/update-time comparison at the destructive owner;
- a current Gateway lifetime/owner fence retained through cleanup;
- distinct cron `changed`/`retired`/`survived` settlement consumers;
- subagent cleanup outside the parent admitted generation while preserving the
  captured child and Gateway authority.

The current Rust boundary is narrower: `source/host/app/src/main.rs:3619-3623`
checks only that `expectedSessionId` equals the derived agent ID, then
`3619-3631` dispatches `deleteAgent` without forwarding
`expectedLifecycleRevision` or `expectedSessionUpdatedAt`. The destructive owner
at `source/host/src/extensions/transcript/agent_lifecycle.rs:601-635` deletes
the session through `store.delete_session` after deletion hooks, and the public
`AgentSummary` currently exposes `updated_at` but no lifecycle revision
(`source/host/src/extensions/session/session_summaries.rs:54-85`). This is direct
evidence of the remaining Rust contract gap; it is not a reason to substitute
`updated_at` for a lifecycle revision without an approved storage/ownership
design.

Therefore this delta does not upgrade
`OCRM-RESP-CRON-SESSION-REAPER-001`, `OCRM-RESP-SESSION-ROUTING-DISPATCH-001`,
or any session responsibility to implemented, wired, verified or accepted. It
also does not authorize a second cleanup owner. The existing Host session,
transcript, Gateway and Runner owners remain the candidate Rust owners pending a
fresh exact-subject read, responsibility mapping and an approved implementation
plan.

## Evidence boundary

This is a read-only exact-source delta audit. No source inventory, dependency
closure, ledger/RTM identity, completion flag, Actions receipt, local OCRM test,
build, package, provider or device acceptance was changed or inferred. The
previous inventory and dependency probe remain bound to their historical
subjects and cannot be used to claim closure for this candidate.

## Implementation checkpoint after the initial audit

The approved lifecycle-fence spec now has a first Rust implementation slice in
the existing owners:

- `source/host/src/extensions/session/agent_db.rs` seeds or CAS-migrates an
  opaque `lifecycleRevision` in the existing metadata row and advances it for
  metadata mutations, checkpoint-root publication, compare-and-set root
  publication, and conversation clearing.
- `source/host/src/extensions/session/session_summaries.rs` and the public Host
  `sessions.list`/`sessions.get` projections expose that revision.
- `source/host/src/extensions/transcript/agent_lifecycle.rs` accepts guarded
  delete fields and rechecks the current session before and after the existing
  deletion hooks.

This later implementation evidence does not alter the exact upstream subject
or promote the responsibility: focused oracles have not been executed,
cron/subagent producers have not been proven to pass the captured fields, and
Actions/package/runtime evidence remains absent. The initial gap statement
above records the pre-implementation state; verification and acceptance remain
pending.

## Current Rust producer audit and oracle checkpoint

The current Rust source tree has no cron isolated-session or subagent cleanup
producer under `source/host/src`; the only matching production entrypoints are
the Host `sessions.delete` adapter and the existing transcript/session
`deleteAgent` owner. The upstream TypeScript producers therefore remain
unimplemented Rust consumers, and no second cleanup owner is introduced to
bridge that gap.

The implementation now rejects an empty lifecycle revision from the public
`sessions.list`/`sessions.get` projections, validates a supplied update-time
guard as a finite number, and includes focused source oracles in
`source/host/tests/agent_lifecycle_contract.rs` for stale revision admission,
stale update-time admission, mutation during the pre-delete drain, and the
matching guarded-success control. These oracles are source evidence only: they
have not been compiled or executed during the development freeze.
Verification remains `not-run`; cron/subagent wiring remains `not-wired`; and
Actions, package, runtime, provider and device evidence remain absent.

The responsibility is now registered as
`OCRM-RESP-SESSION-LIFECYCLE-DELETE-FENCE-001` in `parity-ledger.json` and
linked from the `OCRM-R21` traceability row. This records the implementation
and evidence boundary only; it does not change any completion flag or promote
the upstream discovery subject to accepted authority.

The remaining production-wiring blocker is
`BL-SESSION-LIFECYCLE-PRODUCER-001`: Fabushi Rust has no isolated-cron or
subagent cleanup producer that captures the projection fields and calls this
existing Host lifecycle owner. The blocker is resolved only when a real
producer is found or implemented within the declared owner boundary, its
captured fields reach `sessions.delete` in production, and the corresponding
Actions/runtime evidence is available; helper signatures, fixtures, and static
source oracles do not resolve it.

## Follow-up read at the newer discovery subject

The later upstream subject `openclaw/openclaw@1d2e685b76b2bb7da964a2d6445cec2e07bf5883`
changes `src/cron/isolated-agent/run.ts` (blob
`617257faccf4b8cb7d696e0a8890d77916e83fa5`) but leaves the previously audited
`src/cron/isolated-agent/session-cleanup.ts` blob unchanged
(`5cdeb385daf097dd9d7b87febdc7178dd4a91050`). Direct reading of the changed
run path confirms that the production cron caller passes the captured
`sessionId`, `lifecycleRevision`, and `sessionEntry.updatedAt` into
`cleanupCronRunSessionAfterRun`, along with the existing pre-delete release
hook. This strengthens the upstream producer contract, but it is still not a
Rust producer: the current Fabushi Rust tree has no corresponding isolated-cron
cleanup caller, so no Rust wiring status is promoted.

The same newer-subject read confirms the subagent cleanup source remains blob
`1669914996a02fcecd45012811d0a42f18bd32d7` and requires both captured
`expectedSessionId` and `expectedLifecycleRevision`, while retaining the current
Gateway owner across dispatch. The Rust generated-subagent path currently
materializes child sessions and settles `SubagentRuntime` records, but a source
search finds no completion/cleanup call that dispatches guarded `sessions.delete`
for the child session. This is direct evidence for
`BL-SESSION-LIFECYCLE-PRODUCER-001`, not proof that the Rust path is equivalent.

## 2026-10-31 producer capture slice

The Rust generated-subagent producer now reads the newly materialized child's
non-empty `AgentSummary.lifecycle_revision` and carries it in the existing
runner request into `begin_generated_subagent_with_lifecycle`. The existing
`SubagentRuntime` completion record retains that captured revision, so the
producer-side identity/fence value is no longer dropped between launch and
settlement. This is source-only evidence; no build or test was run.

The cleanup edge remains absent: settlement still only clears the runtime
record/pending wake and invokes completion revival. It does not yet dispatch
the captured `expectedSessionId` and `expectedLifecycleRevision` through the
existing Host Gateway lifecycle owner. Therefore wiring remains `not-wired`,
the blocker remains open, and the capture slice must not be interpreted as
deletion safety or production acceptance.

## 2026-10-31 lifecycle-owner wiring slice

The current Rust source now contains the guarded generated-subagent cleanup
edge. After `SubagentRuntime` settles and the existing completion-revival handoff
is invoked, the worker obtains the injected Gateway-owned cleanup callback. The
callback retains the current `UnifiedGatewayApi`, calls the existing
`ProductionAgentLifecycle` owner with `expectedSessionId` and
`expectedLifecycleRevision`, and performs no retry on a lifecycle conflict.
The callback is installed only after the Gateway is constructed and returns an
explicit error if that owner is no longer current.

This changes the source-level wiring assessment from `not-wired` to
`source-wired`; it does not prove packaged reachability, runtime execution,
Actions verification, or acceptance. The producer blocker is therefore
reframed as an evidence blocker rather than silently removed: exact-head
Actions and real generated-subagent completion/deletion evidence are still
required before the responsibility can be treated as fully production-wired.
