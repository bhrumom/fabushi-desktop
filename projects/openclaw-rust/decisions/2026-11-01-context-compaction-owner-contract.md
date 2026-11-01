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

The compaction commit is guarded by two independent observations captured by
the Runner producer: the session lifecycle revision and the latest canonical
transcript-entry id. Lifecycle CAS protects archive/deletion/owner changes;
transcript-tail CAS protects concurrent ordinary turn appends, which do not
advance the lifecycle revision. A mismatch returns a stale outcome and leaves
the existing transcript unchanged.

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

Provider serializers must also reject duplicate tool-call identities while
building a replay request. A tool result must resolve to exactly one prior
call in the same projection; ambiguity is an unsupported replay shape, not a
reason to stringify or silently drop the row.

## Current implementation evidence

`SandAgentDb::commit_compaction` now implements the guarded transactional
commit, including owner-local non-empty/16,000-character summary validation and
trim-normalized idempotent retry matching,
and `SandAgentDb::get_compacted_transcript_entries` implements the owner-local
replay projection. `ProductionSessionWorkers` exposes both operations. The
compaction transaction also re-reads the existing session-work admission record
and rejects archived or deletion-pending state at the durable write edge,
closing the same owner-local deletion race already covered by the other
session-admitted transcript writers. The
focused contract test is present in
`source/host/tests/agent_db_owner_contract.rs`, but remains unrun during the
development freeze.

The non-test routed-provider completion path now prepares a fixed-source cut
point from canonical transcript rows, invokes the existing summarization
binding, and calls the guarded owner commit. The next routed turn already
reads the owner projection through typed provider serializers. This remains
partial implementation evidence: exact upstream file-operation detail
merging is now wired for the fixed `readFiles`/`modifiedFiles` contract, and
split-turn prefix planning plus its second summarization call are wired.
The producer also bounds large summary inputs with deterministic tail/user/head/
middle selection, gap markers, and head/tail entry elision. The existing
Runner compaction owner now uses the fixed source's provider-independent CJK
character heuristic for its canonical JSON-row projection: ASCII retains raw
UTF-16 weight, common CJK is four weighted characters, rare BMP CJK is twelve,
two-token compatibility ranges are eight, and supplementary ranges use the
corresponding three/four-token weights. This is still an approximation over
canonical rows, not approval of the richer upstream message projection.
Live-turn proof and frozen verification remain open.

## Gemini Argon review record (2026-11-01, Asia/Shanghai)

The plan and resulting diff were discussed in the native Gemini desktop
conversation using the visible `Argon` model. Argon agreed that the slice stays
within the existing Runner → summarizer → `ProductionSessionWorkers` /
`SandAgentDb` → next-turn typed replay chain, and that the historical `ad8d12f`
subject must remain separate from the `1440bb` candidate inventory subject.
The review required duplicate-call registration before emitting provider tool
call blocks in Responses, Chat Completions, and Cursor, with exact
`DuplicateToolCall { tool_call_id }` negative assertions for both same-message
and cross-message duplicates. The post-change review of commit `11f8a78c1`
passed with no blocking defect: registration timing, projection-scoped state,
both duplicate scenarios, and exact-subject separation were accepted. No local
executable verification was requested or run; executable verification and
acceptance remain pending.

The follow-up one-result contract is now implemented in the same inference
owner at Fabushi commit `8c399d0f491d80270609c583adbf29747cc58147`, tree
`7b790903314a9bf6700c5923393980f478a168ef7`. Responses, Chat Completions, and
Cursor serializers use distinct pending/consumed tool-call state: a result must
match one pending call, a second result is rejected as
`DuplicateToolResult`, and an unreconciled call rejects as
`UnsettledToolCall`. This is source and focused-contract evidence only; the
contracts were not executed and live provider/runtime acceptance remains
pending.

A follow-up pairing guard at commit `0fbd29be7edd6f46ad6d52e26f1254b29b6fc7b6`,
tree `6c43ab21acf6aa15c5a5619411c7fa62893474b6`, also retains the expected
tool name with each pending call and rejects a result whose name differs from
that call. The mismatch path restores the pending state and remains retry-safe;
this source change likewise has no local executable verification receipt.

Replay attachment ownership is now narrowed to the existing attachment owner:
it may expose a read-only provider-usable HTTP(S) URL projection for a durable
attachment identity. The inference serializer owns the provider-specific part
shape, while local paths, data URLs, missing URLs, and unsupported schemes stay
fail-closed. This contract does not authorize the compaction path to reuse
outgoing-media custody or to upload historical transcript files; the missing
remote/materialization capability remains an explicit open item.

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

