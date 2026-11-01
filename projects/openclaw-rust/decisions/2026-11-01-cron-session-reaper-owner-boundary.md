# Cron session reaper owner boundary

Status: proposed; architecture gate not yet approved
Date: 2026-11-01
Project: OCRM-001
Responsibility: OCRM-RESP-CRON-SESSION-REAPER-001

## Decision summary

Fabushi does not currently have the durable per-run session representation
required by the OpenClaw reaper. The existing automation store records
definitions and a bounded run history in `runs.json`; it does not own a
session row, transcript identity, lifecycle revision, update timestamp, or
per-run work fence. The agent DB owns one canonical agent transcript and one
canonical `agent:{agent_id}:main` work-admission record. Its archive transition
rejects non-main session keys and its deletion API removes the whole agent
directory.

Therefore no cron reaper implementation is approved by this decision. The
minimum viable target is an extension of the existing `SandAgentDb` /
`ProductionSessionWorkers` ownership path, not a second session database,
automation store, transcript owner, or filesystem-age cleanup job. The work
remains architecture-blocked until the migration and archive contract below
are approved and then implemented with focused oracles.

## Source evidence and exact subjects

- Fabushi source subject captured for this decision (before this
  documentation-only provenance publication):
  `codex/openclaw-rust-fusion-chat-20261101-writer`, commit
  `62b395940f42fcbef9dcc54a0a6512ce057698cf`, tree
  `d2b2e71801b40589ed8c6c81ae5d13a62620b1d1`.
- Fixed OpenClaw implementation subject: commit
  `ad8d12f126d29b0db204187adf5092db80294e70`, tree
  `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.
- The fixed OpenClaw subject is the only implementation authority for this
  decision; moving discovery candidates are not substituted for it.
- Upstream behavior: `src/cron/session-reaper.ts` removes expired per-run
  rows, preserves base sessions, skips active media/subagent/controller work,
  and uses guarded lifecycle deletion when identity, revision, and update time
  are present.
- Fabushi run producer: `source/host/src/extensions/transcript/automation_run_path.rs`
  calls `FileAutomationStore::begin_run` and `finish_run`.
- Fabushi automation persistence:
  `source/host/src/automations/automation_store.rs` stores definitions in
  `automation.json` and at most 20 `AutomationRun` records in `runs.json`.
- Fabushi session schema:
  `source/host/src/extensions/session/agent_db_schema.rs` contains `kv`,
  legacy `blobs`, and one `transcript_entries` table; it has no per-run session
  table or session ownership column.
- Fabushi lifecycle owner:
  `source/host/src/extensions/session/agent_db.rs` and
  `source/host/src/extensions/session/production.rs` own lifecycle revision,
  work/media admission, and the guarded archive transition.
- Whole-agent deletion:
  `source/host/src/extensions/session/agent_session.rs::delete_session`
  closes the agent stores and removes the complete agent directory; it is not
  a valid cron-run deletion primitive.
- Session routing already recognizes `agent:<id>:cron:*` in
  `source/host/src/extensions/session/session_paths.rs`, but recognition is
  not durable row ownership or deletion capability.

The exact-subject revalidation on 2026-11-01 found no implementation or
wiring change that closes D01–D05: `AutomationRunPath` still persists only
the existing bounded automation run history, while `SandAgentDb` still owns
the canonical main-session row and has no per-run session table or guarded
transcript archive primitive. This refresh corrects provenance only; it does
not approve the architecture or promote any ledger/global status.

## Decisions

### CRON-REAPER-D01 — per-run row/key and owner

The durable row must be owned by `SandAgentDb`, reached through
`ProductionSessionWorkers`; `FileAutomationStore` remains the owner of
automation definitions and run history only. A row is created for each
background schedule run and must contain at least:

- `agent_id` / database scope;
- `session_key` (`agent:<agent_id>:cron:<automation_id>:<run_id>` or an
  equivalent stable, documented key);
- automation id and run id;
- `session_id` generation identity;
- lifecycle revision;
- `updated_at_ms`;
- base-session flag (false for a generated run);
- durable execution state and terminal outcome;
- active controller, subagent-continuation, and outgoing-media fence state;
- transcript ownership/archive state.

The minimum migration is in the existing agent DB, with deterministic schema
initialization and indexes for `(session_key)`, `(automation_id, run_id)`, and
expiry/update lookup. It must not create a parallel DB or infer a row from
`runs.json` after the fact. Existing canonical transcript entries remain
main-session entries during migration; no row may claim ownership of them
without an explicit backfill/provenance rule.

### CRON-REAPER-D02 — retention parsing and throttle

Retention parsing belongs to the cron reaper policy owner, with the same
fail-closed semantics as the upstream behavior:

- `false` disables retention;
- zero disables retention;
- invalid or non-finite values use the documented default of 24 hours;
- positive values are converted to a bounded millisecond duration;
- sweep execution is throttled to at most once per five minutes per owner
  scope.

The parser must be a pure policy seam. It must not treat automation run age,
directory modification time, or a missing per-run row as evidence that a
session is expired.

### CRON-REAPER-D03 — producer, trigger, and sweep fence

`AutomationRunPath` is the producer boundary: after a scheduled run is
durably admitted, it must create/update the per-run row through
`ProductionSessionWorkers` before execution can be considered reaper-visible.
The reaper consumer is owned by the same session workers and is triggered by:

1. a bounded startup/maintenance sweep for each available agent;
2. a session/automation change notification that requests an early sweep;
3. the five-minute throttle and an owner-scoped non-reentrant lock.

The sweep must first confirm agent availability and the current Gateway/
lifecycle owner. A closed, deleting, rebinding, or otherwise unavailable
agent is skipped with a diagnostic outcome. The lock and throttle are
coordination state, not permission to delete; every candidate still requires
the per-row guards in D05.

### CRON-REAPER-D04 — media, subagent, and transcript archive ownership

Preservation checks belong to the per-run lifecycle row and the existing
session admission owner. A candidate is not archived or deleted while it has
an active controller, admitted work, pending outgoing media, or unsettled
subagent continuation. These checks must use durable row state and lifecycle
revision, not only the in-memory `AutomationRunPath` duplicate map.

Transcript archive and artifact cleanup must be added to the same
`SandAgentDb` / `ProductionSessionWorkers` ownership boundary. The current
`transcript_entries` table and `conversation-blobs.db` path do not expose a
per-run archive/delete API, so implementation must first define one of these
explicitly in the migration:

- add session provenance to transcript entries and move unreferenced run
  entries into an in-database archive with a durable archive receipt; or
- add an equivalently owned, same-database per-run transcript projection whose
  archive and artifact cleanup are guarded by the row identity.

Filesystem cleanup is only a post-archive artifact step. A cleanup error is a
separate failure outcome and must never be reported as complete deletion.
There is no authorization here to reuse whole-agent `delete_session` or to
introduce a new `TranscriptCleanup` owner.

### CRON-REAPER-D05 — guarded deletion and outcomes

For every candidate, the consumer captures `session_key`, `session_id`,
`lifecycle_revision`, and `updated_at_ms` from the same durable read. The
delete/archive transaction succeeds only if all guarded values still match and
the row is still expired, non-base, and free of preservation fences.

Outcomes are distinct and durable:

- `deleted`: row/archive mutation and required owned transcript archive
  completed;
- `changed`: a guard or lifecycle value changed, so no deletion occurred;
- `preserved`: a base row or active work/media/subagent fence prevented work;
- `failed`: a known archive/delete/cleanup error occurred;
- `outcome_unknown`: the mutation result or cleanup ownership is ambiguous.

`outcome_unknown` must not be retried by the same sweep and must not be
converted to `deleted`. It requires a later reconciliation read owned by the
session lifecycle path. A missing or malformed guard is non-deletable, not a
wildcard match.

## Required implementation sequence

1. Approve or amend this ADR and the cron reaper spec.
2. Define the same-database migration and transcript provenance/archive
   contract, including rollback and recovery receipts.
3. Extend the existing lifecycle owner and scheduled-run producer.
4. Add focused oracles for disabled/zero retention, invalid fallback,
   expiration, base preservation, media/subagent preservation, lifecycle race,
   and transcript cleanup failure/unknown outcome.
5. Re-read the exact source identities and run the required verification
   campaign. Production/runtime/package/provider/device acceptance remains a
   separate gate.

## Non-decisions

- `runs.json` is not a substitute for a session catalog.
- `agent:{id}:main` is not a per-run cron session.
- `delete_session` is not a per-run cleanup API.
- Session-kind classification of `agent:<id>:cron:*` is not proof of durable
  session ownership.
- Static source inspection, contract tests, or old O05 evidence cannot establish
  wiring, runtime behavior, or acceptance.
