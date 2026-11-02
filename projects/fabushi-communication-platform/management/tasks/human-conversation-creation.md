# Human conversation creation without destructive races

Status: active / implementation batch, not accepted
Requirement prefix: FBCP-HUMAN-CREATE
Task: 20bf9475-a885-4314-b509-bea53ef8849f, recovery round 1
Discovery source: `bhrumom/fabushi-desktop@936115b0aa578c340c74dd6aa7c71c362572bd72`
Frozen research U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Problem and owner

The shipping renderer's `createHumanConversation` and friend-roster sync both
reach `ProductionSessionWorkers::create_human_conversation`. Its current
file-existence check is separate from SQLite initialization. A competing creator
or a busy initialization can return false; the losing caller then removes the
entire shared conversation directory. That directory can already contain the
winner's metadata, transcript, resources and durable forward snapshots.
`initialize_metadata_if_missing` also checks and upserts in separate statements
under only one handle's mutex. An interrupted creation can leave a schema-only
database which later calls cannot initialize. These are source-observed defects;
no pre-fix execution is claimed.

Keep the current Session/Transcript and SandAgentDb owners, canonical participant
pair digest, gateway command/result, native service and renderer. No new database,
module, service, queue, public schema or Telegram runtime is introduced. The
existing NMR-49 reply and active NMR-50 recovery batches are independent concurrent
work and must be preserved.

## Required behavior

- **FBCP-HUMAN-CREATE-01:** Creation is idempotent for the same canonical pair.
  Concurrent callers, including distinct workers and SQLite handles using the
  same root, cannot replace established metadata or remove any shared path.
  Use SQLite atomic insert/transaction semantics for the absent-metadata
  decision; a per-instance Rust mutex is not cross-handle authority. Preserve
  existing general metadata initialization consumers.
- **FBCP-HUMAN-CREATE-02:** A winner's complete persisted metadata is read and
  validated before another caller returns or caches the owner. Kind, canonical
  conversation ID and exactly the requested two distinct Human participants
  must agree. Conflicting, malformed, foreign or partial existing metadata fails
  closed without rewriting it. Existing valid title, timestamps and ancillary
  metadata remain unchanged.
- **FBCP-HUMAN-CREATE-03:** Resume an interrupted schema-only store through this
  same creation entry. Establish that the metadata is absent and that no
  transcript, blob or other durable KV data exists at one atomic SQLite write
  boundary before seeding Human identity. A missing metadata row in a populated
  store is an integrity failure, not permission to adopt its messages. Empty
  directories and a schema-only database are recoverable; retained user data,
  journals, resources and existing database bytes must never be cleaned up by
  a failed creator.
- **FBCP-HUMAN-CREATE-04:** Busy, storage, decoding and initialization failures
  retain retryable state and return a truthful failure when no valid winner can
  be read. Cache/publication follows durable identity validation. Retrying or
  reopening after the failure converges to the same conversation ID. Existing
  Human/Agent isolation, participant admission, account authority and ordinary
  send/roster/forward recovery remain in force.

## Verification and acceptance for this batch

Run all executable checks only in GitHub Actions against the frozen batch H,
using the existing Session gateway/SQLite tests and native service workflow.

1. Exercise simultaneous same-peer creation through distinct production worker
   instances and real SQLite handles; assert one canonical identity, valid
   persisted metadata and non-destructive retry/reopen. Use barriers or an
   actual held SQLite write lock to establish contested ordering, not
   probabilistic sleep-only claims. A transaction/insert contention test must
   independently prove first-writer metadata preservation.
2. Force a real initialization/busy failure after the database path exists,
   preserve an explicit retained directory sentinel, and recover after release.
   Test both empty-directory and schema-only restart recovery. No data-cleanup
   assertion may be satisfied by deleting and recreating the directory.
3. Cover existing wrong kind/ID/participant cardinality and malformed metadata,
   plus missing metadata with existing transcript/KV/blob data. Assert unchanged
   durable contents and no false successful conversation. Keep real storage
   failures distinct from intentional identity rejection.
4. Reach the repaired creation owner through the shipping Session gateway, then
   use the existing native send/SQLite/event/reopen harness to show that the
   resulting conversation remains usable without duplicate messages. Preserve
   existing Agent and Human contracts. Test authorship is not execution.
5. Complete independent correctness and lifetime/data-integrity review of the
   exact diff; fix demonstrated defects. Record exact H, commands, run/job/log
   references and any unexecuted gates in the unique ledger/RTM and task evidence.

## Integration, recovery and limits

Integrate by expected-head, non-force CAS on the existing migration line after
reconciling concurrent edits. Preserve the suspended local work as historical
recovery material; do not overlay it onto the current branch. Update only this
batch's existing ownership and responsibility records, with acceptance false.
No upstream ref is changed and no source is marked read-complete by this repair.

Rollback is an ordinary code revert and retains all stores and resources;
reintroducing destructive initialization is a known regression, so it cannot
be a safe published rollback without review. No user-data migration is required.
This batch does not prove full source/legal closure, remote service deployment,
cross-platform packaged behavior, all Bot regressions, visual/accessibility,
upgrade/rollback, performance/soak or independent product ACCEPT. Those original
requirements remain open.
