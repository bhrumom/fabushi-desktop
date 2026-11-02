# FBCP gateway peer durable mutation boundary

Parent: FBCP-001 / TDRP-001 / FMEP-001; existing native gateway owners.
Status: authored; containing-H Actions and independent acceptance required.
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.

The originally prepared peer ID NMR-48 was never published by this task.
Concurrent checkpoint work assigned NMR-48/NMR-AC-39 to immutable forwarding
attachments. This document uses unique peer IDs and preserves that existing
contract, all later Human/Agent/UI/CLI gates and current canonical source records.

### Gateway peer durable mutation boundary (NMR-PEER-DURABILITY-01 / NMR-AC-PEER-DURABILITY-01)

Current source `59f5fa24827812f78336c464995d161f403dedfb` still mutates the
PersistentServerRequestRegistry in memory, records persistence errors only in
`lastError`, and returns successful frames/resolutions/cancellation. The real
stdio gateway then publishes an approval or calls Host despite that failure.
Its compatibility `approval.respond` path calls Host before clearing the peer
record. A stale registry can also replace another instance's entire peer list.
These are source-observed implementation defects, not pre-fix executed failures.

- NMR-PEER-DURABILITY-01a: Keep the existing gateway peer as pending-request authority and the
  existing version-1 shared gateway document. Each durable mutation must take
  the shared state lock, read and validate the latest document, import into a
  temporary registry, apply the operation and persist before replacing the live
  registry or returning any success/frame/resolution. Persistence errors are
  returned explicitly and retain the prior live snapshot and committed data.
  Startup import is also all-or-nothing. Reject malformed/unknown schema,
  malformed peer data, duplicate persisted IDs and nonregular/symlink state
  inputs without overwriting them. Preserve unrelated replay/root/peer fields.
  An explicitly configured memory-only registry retains that documented mode.
- NMR-PEER-DURABILITY-01b: Approval issue deduplication and compatibility response lookup belong
  inside that same latest-state transaction. The same runtime requestId cannot
  create two pending peer requests through the production issue path, even from
  two independent registry instances. A stale writer cannot erase a new request
  or resurrect a durably resolved request. Generic clarify locking and session
  cancellation receive the same durable-before-success rule. Unknown late
  response IDs remain harmless; storage failures must not be disguised as that
  no-op outcome.
- NMR-PEER-DURABILITY-01c: Both existing shared-document writers use the same persistent lock
  inode and the already locked fs2 0.4.3 advisory lock API with bounded contention
  waiting. Hold the file handle for the whole read/mutate/write transaction;
  never steal a lock by timestamp or unlink it on release. Refuse a nonempty
  legacy sentinel or nonregular/symlink lock file without deleting it. Use
  private owned fixture directories; permissions apply to opened handles.
  Preserve the only backup when the main file is absent until a new main is
  installed, and remove only this operation's temporary file on failure.
  No new dependency version, state database, runtime owner or format migration.
- NMR-PEER-DURABILITY-01d: The actual CLI event path returns peer issuance errors and publishes
  no approval frame/event from that failed batch. The direct response route
  propagates durable resolution errors before any Host-facing approval call.
  Compatibility `approval.respond` reuses the existing parser and canonical
  decision mapping, with a GatewayRuntime adapter performing durable peer
  settlement before invoking the underlying Host. Preserve valid legacy calls
  without a peer record and all existing decision/denial rules. Errors remain
  visible to the transport rather than becoming success acknowledgements.
- NMR-AC-PEER-DURABILITY-01: Add real-file peer tests for issuance, response, clarify lock and
  cancellation failures, unchanged live/committed state, repair and reopen;
  malformed import, only-backup recovery, stale-instance merge/deletion and
  duplicate-approval suppression; lock contention, old timestamps and release
  after a killed child process. Exercise the actual CLI helpers/adapters with
  independent observations of Host-facing calls, including both response paths,
  malformed decisions and storage failure zero-call controls. Add direct peer
  library execution and path triggers to the existing Actions gate, retain the
  actual CLI, full gateway library, Host/Agent/message/UI/package gates, and
  qualify the containing exact H. No local executable validation is permitted.

This closes the peer storage-failure success path and current-writer lost-update
boundary. It does not claim atomicity between a durable peer resolution and a
later Host side effect: Host failure or process exit in that interval still needs
a separate durable handoff/idempotent Host contract. Replay-journal publication
after its own persistence failure, multi-writer replay merge/epoch semantics,
power-loss durability, simultaneous old/new
binaries and coordinated upgrade/rollback remain unqualified. Old create-new
lock writers do not honor advisory locks; stop them before using the new writer
or rolling back, and do not delete their locks without verified ownership.
The live event publisher fence does not make the peer and replay namespaces one
transaction: an approval event can enter replay before peer issuance fails, and
`session.events.since` can return that historical event. Durable handoff/replay
reconciliation and its sequence semantics remain required before full approval
recovery acceptance; this batch does not label the whole approval flow verified.
No backend change, full Bot regression, signed package or independent ACCEPT is
granted by these focused contracts. Fixed U and all final acceptance gates stay.
