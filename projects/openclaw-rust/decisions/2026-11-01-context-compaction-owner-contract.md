# ADR: Context-compaction owner contract

Status: owner contract and Runner producer wiring implemented; verification pending
Date: 2026-11-01
Scope: Fabushi Desktop Host Runner, transcript, and inference integration
Responsibility: `OCRM-RESP-AGENT-CONTEXT-COMPACTION-001`

## Context

The fixed OpenClaw compaction facade and its agent-core implementation define
cut-point selection, retained-history rules, summary fitting, split-turn
context, file-operation metadata, and no-summary fallback. Fabushi already has
the production summarization binding and a canonical transcript owner, but no
caller commits a compaction result or makes the next turn replay it.

The canonical state boundary is:

`ProductionSessionWorkers -> SandAgentDb -> transcript_entries + metadata`

`SandAgentDb` already provides `BEGIN IMMEDIATE` write transactions, metadata
compare-and-set, and a durable `lifecycleRevision`. Its public transcript API
only supports independent append, update, and single-entry delete operations;
composing those calls in Runner would permit a partially committed compaction.

## Decision

Compaction state remains owned by `SandAgentDb`, exposed through
`ProductionSessionWorkers`. A future implementation must add one owner-local,
transactional compaction operation and one replay projection; Runner and
Inference may call that operation but may not manipulate transcript rows or
metadata directly.

The operation must accept and validate, in one call:

- the expected `lifecycleRevision` and source turn identity;
- the fixed-source `firstKeptEntryId` cut point;
- the bounded summary and `tokensBefore`;
- optional parsed compaction details and file-operation metadata;
- an idempotency/compaction-entry identity.

The owner must atomically reject stale lifecycle state, malformed or missing
cut points, and conflicting duplicate identities. A successful commit must
advance the lifecycle revision and publish one replay-visible compaction entry
whose schema contains the retained-entry boundary, summary, token count,
details, source turn, and resulting revision. A duplicate retry may return the
already committed identical result; a different payload with the same identity
must fail closed. Any failure leaves the previous transcript and metadata
unchanged.

Replay must be implemented by the same transcript owner and must expose the
compacted context exactly once to a newly constructed Runner turn. Existing
generic append/update/delete methods, prompt `compaction_epoch` snapshots,
contract-only `ContextWindowTracker`, and a second transcript database are not
valid substitutes.

## Current implementation evidence

`SandAgentDb::commit_compaction` now implements the guarded transactional
commit and `SandAgentDb::get_compacted_transcript_entries` implements the
owner-local replay projection. `ProductionSessionWorkers` exposes both
operations. The focused contract test is present in
`source/host/tests/agent_db_owner_contract.rs`, but remains unrun during the
development freeze.

The non-test routed-provider completion path now prepares a fixed-source cut
point from canonical transcript rows, invokes the existing summarization
binding, and calls the guarded owner commit. The next routed turn already
reads the owner projection through typed provider serializers. This remains
partial implementation evidence: exact upstream file-operation detail
merging is now wired for the fixed `readFiles`/`modifiedFiles` contract, and
split-turn prefix planning plus its second summarization call are wired.
Bounded sampled summary-input parity, live-turn proof, and frozen verification
are still open.

## Required implementation sequence

1. Add the owner-local schema/API and migration-safe transaction, with focused
   tests for stale revision, duplicate commit, malformed cut point, rollback,
   and replay.
2. Adapt the existing Runner turn owner to estimate context, prepare the fixed
   cut point, invoke the existing summarization binding, and call the owner
   operation only after the provider result is complete. (Implemented in the
   routed-provider completion boundary; file-operation metadata remains a
   deliberate fail-closed gap.)
3. Ensure cancellation/provider failure leaves the prior transcript intact and
   that a newly constructed turn consumes the committed projection.
4. Reconcile the parity ledger only when a non-test producer and next-turn
   consumer are present. Verification and acceptance remain pending until the
   frozen exact-head lane is available.

## Rejected alternatives

- Calling `append_transcript_entries` followed by deletes from Runner.
- Treating `compaction_epoch` as conversation-history compaction.
- Wiring the test-only `ContextWindowTracker` directly into production.
- Creating a parallel summary/session store or provider-specific transcript.

## Evidence and references

- `projects/openclaw-rust/docs/specs/context-compaction-production-boundary.md`
- `source/host/src/extensions/session/production.rs:1152-1189`
- `source/host/src/extensions/session/production.rs:1711-1748`
- `source/host/src/extensions/session/agent_db.rs:428-452`
- `source/host/src/extensions/session/agent_db.rs:1266-1414`
- `source/host/src/extensions/session/agent_db.rs:1561-1620`
- fixed upstream `ad8d12f126d29b0db204187adf5092db80294e70`

