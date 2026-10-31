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
