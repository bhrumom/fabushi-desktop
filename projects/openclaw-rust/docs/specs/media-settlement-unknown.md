# Outgoing media settlement uncertainty — MSU-v1

Task `647e0932-bea4-4113-9a61-65ae3db58955`, round 1. This is the narrow
production correction to MAC-v1, not a replacement media/session owner or a
claim that the complete media responsibility is accepted.

Canonical responsibility is `OCRM-RESP-MEDIA-FINAL-CUSTODY-001` in the existing
OCRM ledger/RTM, under `OCRM-UNIT-MEDIA-008`. MAC-v1 already named this identity
but the prior ledger had no row. Register it against the existing Session/DB
owner; the separate `OCRM-RESP-MEDIA-STORE-READ-SCOPE-001` stays incomplete.
Scoped source reads do not alter historical full-reading facts or global flags.
The existing source-authority Actions workflow derives canonical inventory
mapping/counts from these exact-U references; no local generator is run.
New responsibility links must first be declared through the existing
`inventory/index.json::pending_mapping_additions`, anchored to the unchanged
canonical inventory digest. Actions consumes only the exact declared missing
links; historical rows, reading facts and completion flags are not hand-edited.

## Authority, current problem and scope

Fixed OpenClaw U is `1440bb1771372334bea23d7642bb52deb7db1845`, tree
`90c8aab39972da11cfad220361781d49e1bc6700`. Target base is the published LFA
checkpoint `fdb0f6acb06165b6f2796961186e06a2de98c229`, tree
`abeb6ffdf09b676279462fea3b9e1f6d72abc1b6`, on the existing migration branch.
The three adjacent source/design reports in
`../evidence/media-settlement-unknown-647e0932-20261103/` bind the actual
producer, owner, consumers, fault edges, exact blobs and honest read ranges.
They are specification inputs, not executable evidence.

The existing AgentDb media transaction inserts the send, inserts its hidden
settlement receipt and clears the durable media fence atomically. Its error
path currently discards ROLLBACK results; Session and Runtime erase typed
outcomes into String; the App then deletes owned media on every error, also
through Drop. A real COMMIT with a lost acknowledgment can therefore leave a
durable transcript referencing deleted bytes. An unconfirmed rollback can
leave an active cached transaction whose rows are not safely reusable.

U's final-media and delegated-file custody explicitly retain files on canonical
unknown outcomes. Its separate native receipts can establish commitment despite
a lost ordinary reply. Its transient cleanup is not proof of indefinite unknown
retention. The target uses its existing durable receipt/fence and Session
first-open recovery, with no TTL-based inference that uncertainty has ended.

Entry is the actual generated SendMessage tool, through its existing App sink,
TranscriptRuntime, ProductionSessionWorkers and SandAgentDb. `attachment_paths`
owns only process-local file cleanup authority. The existing Runner tool guard
owns downstream-unknown execution custody. React/Electron and wire schemas do
not change. No new scheduler, session database, runtime, registry or retry loop
is introduced. No-fence compatibility sends, generic filesystem publication
durability, later A2A persistence, and arbitrary process crash points remain
separate applicable work; they are not waived by this batch.

## Required behavior

- **MSU-01 / owner outcome:** retain all existing media admission, replay,
  exact transcript, receipt and fence-CAS guards in the same transaction.
  Confirmed COMMIT is success. Any attempted COMMIT error is
  `AgentDbProjectionError::TranscriptCommitUnknown`, even if subsequent cleanup
  succeeds. A pre-COMMIT error is ordinary refusal only after rollback and
  autocommit are confirmed. Unconfirmed rollback, a pre-existing live
  transaction, or an unsettled BEGIN failure is unknown. Retire the exact DB
  connection on every unknown result before returning; do not reopen, retry,
  publish successful transcript/admission notifications or fabricate a receipt.
  A clean BEGIN refusal remains an ordinary prewrite error. Existing known
  refusal behavior and byte-for-byte receipt/admission formats are preserved.
- **MSU-02 / typed chain:** the existing Session media append returns
  `Result<usize, AgentDbProjectionError>` and the existing Runtime media append
  returns `Result<Value, AgentDbProjectionError>`. Preserve the inner DB error
  through Session's established nested-result filesystem boundary. Map only
  before-operation lock/admission/open/history/entry-planning errors to their
  existing ordinary variants. Runtime is the sole Session caller, and App is
  the sole Runtime caller at this base. Existing non-media String APIs stay
  unchanged. No string matching may classify unknown.
- **MSU-03 / cleanup authority:** replace the custody boolean with explicit
  local rollback-eligible, owner-settlement-pending and committed dispositions.
  Seal the owned media before invoking the admitted media append. Pending
  includes an active call, an unwound call and a returned unknown result;
  ordinary abort and Drop retain bytes in all three cases. Only the same
  attempt's confirmed precommit refusal restores cleanup eligibility. Only a
  confirmed successful append commits custody. Never call commit to disguise
  uncertainty. Recovery continues to create its own fresh eligible custody
  after it has established the existing durable recovery facts.
- **MSU-04 / App and concurrency:** return existing
  `ProviderSessionError::ToolOutcomeUnknown` for the typed unknown and preserve
  the exact pending entry/session/revision/work identity. Same-tool-ID
  resolution and send must refuse before materialization, capture, history
  allocation or another append while that entry is pending/unknown. Explicit
  abort cannot remove it or release its durable fence. Serialize or reserve
  same-ID operations under the current pending-media owner; a preflight-only
  check followed by an unchecked absorb is insufficient. Never hold its map
  mutex across Session/DB or external callbacks. An impossible post-success
  result-shape failure must retain media and report uncertainty. Confirmed
  success settles custody before delivery/event/ACK/success-count consumers.
  Keep the existing map as the sole index, with each value an
  `Arc<Mutex<PendingOutgoingMedia>>`. Release the map lock before taking an
  entry's `try_lock`; busy or poisoned entries refuse with typed unknown and
  abort retains them. Seal before a potentially committing call. Retain compact
  Committed/Aborted terminal entries so old Arcs cannot coexist with a newly
  created entry for the same ID. Collecting entries may resolve distinct images;
  an exact repeated source reuses its resolved value without another capture.
  The Tool build-error branch must not abort an unknown result. The existing
  per-invocation recorder still supplies its narrower duplicate-ID guarantee;
  these local locks do not claim authentication of unrelated non-overlapping
  calls that reuse one string.
- **MSU-05 / terminal and recovery:** the actual tool bridge/guard must retain
  downstream-unknown custody and suppress automatic replay and fabricated
  SendMessage success. The sink's unknown tombstone is process-local, not a
  durable replay registry. Existing Runner recovery and durable media
  receipt/fence remain authoritative across restart. Lost-ACK committed rows
  and bytes survive sink destruction and fresh owners; the uncommitted case
  keeps its fence and bytes until safe owner retirement followed by existing
  first-open recovery proves cleanup. Cached reads are not recovery. Unknown
  or malformed recovery identity remains fenced, with unrelated/source files
  unchanged. No automatic owner replacement is introduced.
  An owned-media compatibility append whose legacy String API cannot prove a
  clean refusal is conservatively unknown after the call begins; it retains
  bytes without claiming a new durable compatibility receipt. Untracked text
  sends retain their existing API behavior. This is a retention fallback, not
  acceptance of the still-open no-fence durability contract.

## Causal test access and oracles

- **MSU-06 / test support:** add default-off Host `test-support = []`, enabled
  only by the App's dev-dependency on that same package. Ordinary shipping
  App compilation must also run without this feature. Test-only types, fields,
  methods and observations use `cfg(any(test, feature = "test-support"))`.
  Reuse the existing shipping check and App test build, retaining their Cargo
  JSON compiler artifacts. Bind the exact Host library by manifest and source
  path; require one artifact with test-support absent for the shipping check
  and present for the App test build. Both Host dependency artifacts must be
  ordinary libraries, not cfg(test) Host test harnesses. This observes the
  actual compilation rather than treating a dependency-tree approximation as
  a compiled-feature proof; no second shipping build is introduced.
  Keep the pinned channel command adapter unchanged. Keep conditional test
  statements outside the media method body so its existing three admission
  guard oracles still apply.
  The existing global raw-COMMIT oracle assumed only one argument-taking
  command adapter. MSU adds a second legitimate adapter. Extend that oracle
  with a separately proven exact MediaSettlementCommands method and its sole
  existing media-owner consumer, preserving the complete channel method,
  returned error-handling tail, unique caller and macro/alias negative cases.
  Do not merely raise a global call count or delete the raw-COMMIT prohibition.
  New negative controls must reject an extra/aliased media caller, ambiguous
  or conditionally excluded helper/owner, altered COMMIT/rollback/retirement
  classification, discarded handling and a hidden raw-COMMIT owner elsewhere.
  Distinguish inherent owner implementations from the actual `Drop` trait
  implementation when selecting and parsing the one media owner. Trait bodies
  remain in the global unsafe-call/raw-COMMIT scan and can never receive the
  bounded inherent-method exception. Keep duplicate/foreign/conditional inherent
  owner and trait-only owner refusals. For source comparison only, normalize
  CRLF checkout newlines to LF before literal substitutions and token comparison;
  raw input receipts retain their original bytes and hashes. Require both LF and
  CRLF actual-owner positives, changed-branch and trait-owned unsafe negatives,
  and evidence that each destructive substitution really changes its input.
  Each media mutation starts from a proven actual-source baseline and binds
  exactly one intended replacement site; every media mutation negative exercises
  both line endings. These repair the exact-H74 oracle failures without changing
  product behavior or weakening the global prohibition.
- **MSU-07 / exact fault port:** the existing SandAgentDb exposes a hidden
  `MediaSettlementTestFault::{LoseCommitAcknowledgement, RefuseRollback}` and
  `arm_media_settlement_fault_for_test(session, revision, work, fault)` returning
  a borrowed `MediaSettlementFaultLease` with `was_claimed()` and
  `was_applied()` observations. One slot per exact owner, exact nonempty
  canonical identifiers, one unique lease token and one use. Reject occupied,
  closed or already-unsettled owners. Mismatch does not consume the slot;
  claim it under the connection boundary before BEGIN; later calls cannot
  reuse it. Lease Drop clears only its own still-unconsumed token and never
  changes product custody. Lock order is connection before slot; do not keep
  the slot locked during SQL or publication. Access is via existing Session
  owner-open, without raw connection/SQL executors, new factories, environment
  flags or a general fault framework.
- **MSU-08 / real command edges:** after a real successful media COMMIT, the
  lost-ACK port may replace only its acknowledgment; an actual COMMIT error
  remains unchanged. The helper must not manufacture an unknown enum or
  durable rows. ROLLBACK refusal uses SQLite's real authorizer on the owning
  connection, denying only that invocation's actual ROLLBACK, with a scoped
  guard removed before retirement. Observations never drive product outcome
  classification. Both applied-fault assertions are mandatory.
- **MSU-09 / DB controls:** exercise actual deferred-COMMIT refusal with clean
  and refused rollback, real COMMIT/lost ACK, a real inserted prefix followed
  by receipt failure/ROLLBACK refusal, prior active transaction, ordinary
  successful settlement/replay, known refusal and fault-slot isolation/lease
  lifetime. Inspect committed rows, receipt/fence, owner usability and
  publication effects through the real owner and separate file-backed reader.
  The owner getter includes the hidden settlement receipt; only the Session
  presentation projection filters it. The lost-ACK reopen and exact-other-owner
  controls must therefore compare the complete ordered send-plus-receipt image,
  using the fixture's independently specified receipt identity, session,
  lifecycle, work, media paths and transcript IDs. Preserve the complete raw-row
  equality and every retirement, no-publication, fence, retained-file and repeated
  reopen check. This repairs the two test expectations observed at exact H
  `74f11d3d31c6245ac5886dc0181460b2d4349744` in Runtime `41580539491`
  attempt 1; it changes no product projection or settlement behavior.
- **MSU-10 / actual App controls:** use existing file-backed App fixtures and
  real production sinks/Session/Runtime. Required cases are actual COMMIT/lost
  ACK; receipt refusal after a proven inserted send with actual ROLLBACK
  denial; the same receipt refusal with normal rollback; same-ID retry/abort/
  sink Drop and two fresh owner observations; and a generated Runner producer
  consuming unknown through the existing tool guard. Prove zero channel send,
  ACK, success count and fabricated send result on unknown, retained bytes and
  exact durable state, and ordinary recovery/positive behavior after safe
  retirement. Preserve all existing SGF/LFA assertions and counts. A returned
  mock unknown error is insufficient evidence for these transaction faults.
  Existing LFA idle checks continue to require empty durable work identity and
  paths. Replace only their raw map-emptiness implementation assertion with
  all entries Committed/Aborted, no active custody/admission; pending, busy or
  poisoned entries are never idle. Terminal marker retention is now required
  by MSU-04 and does not relax any durable or file-preservation oracle.

## Implementation, verification and exit

Root serially integrates public signatures, Cargo, composition, capture,
OCRM ledger/RTM and specification. A bounded DB worker may edit only
`source/host/src/extensions/session/agent_db.rs`, including its private tests,
against the above base/blob and shared MSU-v1 contract. App/local custody and
facades are separate allowed paths. No other writer's worktree is edited.

Every executable check runs only in GitHub Actions on the newly frozen H.
Reuse the existing workflow/captures and select the changed DB contracts,
admission-guard oracles, actual App cases, normal feature-off shipping check
and affected high-risk regressions. Preserve original failed evidence and
classify implementation/oracle/environment failures. Missing current-H
execution, named positives, exit/log/input hashes or real fault application
keeps the batch unverified. Review each changed owner and actual consumers;
fresh focused review follows accepted fixes. No ledger/global promotion occurs
from writing this contract, source inspection, mocks or unrelated green runs.

No persisted format migration is planned. A rollback to old code cannot be
claimed safe for unresolved media attempts; preserve existing durable recovery
data and finish owner reconciliation before resuming a known-unsafe version.
Error reporting must not expose media bytes or URL credentials. Real provider,
channel/device compatibility, all platform packages and final G01–G12 /
AC01–AC24 plus independent ACCEPT remain required for full migration delivery.
