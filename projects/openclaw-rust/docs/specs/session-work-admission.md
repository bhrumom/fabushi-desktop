# Session work admission — production boundary

Status: architecture selected; deletion-fence implementation slice in progress
Date: 2026-11-01
Responsibility: `OCRM-RESP-SESSION-WORK-ADMISSION-001`

## Context and goal

Fabushi already owns agent SQLite state, transcript mutation, Runner turn
settlement, and lifecycle revisions. The fixed OpenClaw session work-admission
surface additionally prevents writes during deletion, binds detached results to
the admitted session generation, protects archive/restore transitions, and
retains provider-review terminal ownership. The goal is to implement those
semantics through the existing Fabushi owners with one producer → owner →
durable state → consumer chain.

## Non-goals

- No second session database, workboard, or provider-specific result store.
- No reuse of cloud-agent archive/unarchive as session lifecycle authority.
- No filesystem-age or automation-history-only deletion.
- No acceptance claim from fixtures, static symbols, or a test-only helper.

## Requirements

- `SWA-01`: Every admitted session write carries canonical session identity and
  a durable generation/lifecycle revision. The shared session owner must reject
  an owner-local admission record whose durable `session_id` is not the
  canonical agent identity, non-empty session key, and non-empty lifecycle
  revision before admitting any session-owned operation.
  The SQLite write predicate binds that identity to the owner metadata
  `agentId` at the same transaction edge; missing or malformed owner identity
  is fail-closed when an admission record exists.
  The production owner read facade applies the same validation before exposing
  an admission record to a producer or consumer.
- `SWA-02`: Pending agent deletion rejects new writes and retains any owned
  terminal writes until the existing deletion owner publishes a safe replay or
  failure outcome. The owner-local admission record carries a migration-safe
  `deletionPending` fence while the existing lifecycle owner has deletion
  custody; the in-memory deletion marker remains a fast-path guard, not the
  durable authority. A failed delete clears the durable fence only after the
  existing agent directory remains available; an unknown cleanup outcome stays
  fail-closed.
- `SWA-03`: Detached background results commit at most once against the exact
  admitted session generation and preserve transcript/media idempotency.
- `SWA-04`: Archived sessions reject new work; restore requires the existing
  authenticated session/placement authority and rechecks identity at commit.
  Every actual archive/restore transition rotates the lifecycle revision so a
  worker admitted before the transition cannot resume after restore.
  The first production producer is the authenticated Host Gateway
  `sessions.patch` route for the canonical `agent:{id}:main` session; it must
  carry expected session identity and lifecycle revision and fail closed when
  admitted background work or an active foreground Runner stream is still
  active.
- `SWA-05`: Provider-review terminal facts remain owned by the admitted run;
  retries, fallback providers, and successor runs cannot claim them. The
  session owner must also reject acknowledgment issuance and claim when the
  stored review session identity differs from the owner-local admission
  record.
- `SWA-06`: Stale, deleted, archived, duplicate, cancelled, and
  outcome-unknown results are explicit and fail closed.
- `SWA-07`: Existing `ProductionSessionWorkers`, `SandAgentDb`, transcript
  runtime, and Runner remain the owners; no parallel state owner is introduced.
- `SWA-08`: Profile and avatar filesystem mutations and the existing agent
  directory deletion owner share one per-agent session-owner serialization
  boundary. The mutation must recheck admission while holding that boundary;
  deletion must hold it through directory cleanup. A facade precheck without
  this shared boundary is insufficient.
- `SWA-09`: External per-agent memory deletion and clearing use the same
  session-owner filesystem boundary as directory deletion. The owner captures
  the affected records and snapshot before mutation; if the owner-local
  snapshot clear loses a deletion-fence race, it must compensate by restoring
  the captured records and return a failure/unknown outcome rather than
  reporting success.
- `SWA-10`: Background memory synthesis applies changes through the existing
  `ProductionSessionWorkers` owner. The synthesis target must fail closed
  until that owner gate is bound, hold the per-agent filesystem/deletion lock,
  recheck admission, reject a store path that is not the canonical memory
  store for the agent, and retain the existing fingerprint/CAS stale result.
- `SWA-11`: External per-agent memory mutation must survive a Host crash
  between the file-store mutation and the owner-local SQLite mutation. The
  existing `SandAgentDb` owner must durably record the exact session identity,
  lifecycle revision, mutation kind, and captured memory records plus their
  origin/tombstone metadata before the external store changes. The owner
  transaction that clears the memory
  snapshot must delete that journal atomically. On the next owner open, a
  journal that is still present must be treated as an incomplete/unknown
  mutation: restore only the captured records and metadata through the
  canonical memory store, clear the matching journal with an owner CAS, and
  leave the session
  fenced when any identity, path, restore, or CAS check is uncertain. No
  second journal, memory database, or best-effort timestamp inference is
allowed. Recovery must also compare the current owner-local memory snapshot
with the journal's captured pre-mutation snapshot; a mismatch is rebound or
outcome-unknown state and must retain the journal and fence.
- `SWA-12`: Profile and avatar mutations that cross the agent directory and
  owner-local SQLite must use the existing `SandAgentDb` owner as a durable
  crash-recovery fence. The journal must capture canonical session identity,
  mutation kind, the exact allowlisted before file state, and any owner-local
  profile state changed by the mutation before external files are touched. The
  producer must seal the exact allowlisted after state before commit. Commit
  must clear the journal in the same owner transaction as the
  SQLite profile-pointer update when one is required. Owner reopen must
  restore the before state only when the current files still match the
  journaled after state (or are already at before), otherwise retain the fence
  and fail closed. No path inference, timestamp inference, or second profile
  authority is allowed.
- `SWA-13`: Until a durable per-key session catalog and transcript/Runner
  ownership contract exists, public Gateway operations that consume an
  agent-wide owner must reject `cron`, `group`, `spawn-child`, and `unknown`
  routes before reading, cancelling, or dispatching through that owner. A
  session-kind label must not make an agent-wide transcript, stream registry,
  or send owner appear to be a
  per-session target. Internal producers retain their existing authenticated
  admission path and are not converted into public routing authorities.
- `SWA-14`: Canonical public main-session comparisons must use the normalized
  structural route identity, accepting ASCII case variation in the `agent:`
  wrapper, agent identity, and `main` segment. Opaque legacy selectors remain
  outside the canonical `sessions.patch` mutation path.

## Current and target state

Current Fabushi has per-agent SQLite transcript state and lifecycle revisions.
The owner now also has a migration-safe `sessionWorkAdmission` record,
generation-bound background admission/recheck, and a same-owner idempotent
hidden result receipt for the bounded local background-revival path. There is
still no complete media custody contract or full provider-review terminal
lifecycle; the session owner now has bounded typed refusal recording,
continuation admission, and generation/run-bound terminal settlement, while the
media owner has a durable path fence, settlement receipt, and bounded reopen
recovery consumer.
The authenticated Host Gateway archive/restore producer is now the first
bounded production path, limited to canonical main sessions and fail-closed
while admitted work, active generated-subagent work, or an active foreground
Runner stream is active. Media custody still has no equivalent session-owned
active-work authority and remains an open requirement.
The archive producer now passes the normalized canonical main-session key from
the public route resolver into the owner transition. This preserves SWA-14's
case-insensitive structural selector acceptance without passing an
un-normalized request key into the case-sensitive owner contract.
The same normalized key is now used by `sessions.get` provider-review
projection and by provider-review continuation admission, so case variation
cannot hide an owner-bound review or be mistaken for a different session.
The authenticated provider-review continuation producer also binds its
expected session ID to the canonical route agent before issuing or claiming
the existing owner capability; a mismatch is rejected as a lifecycle change.
`deleting_agents` remains the in-memory fast-path deletion fence; the durable
`deletion_pending` field now backs the same owner-local admission record when
the agent database is available. This closes the durable deletion-window
sub-contract only; it is not the complete detached-result lifecycle.

Target state is one owner-local session admission record containing canonical
`sessionKey`, `sessionId`, lifecycle revision, `deletionPending`, archive metadata, admitted
background-work identity/idempotency, one durable pending outgoing-media work
identity, and structured provider-review facts, continuation run identity, plus
terminal phase. The record
is changed transactionally with transcript/result publication. Pending media
must block archive/restore transitions and be cleared only by the same
session/transcript owner after commit or explicit rollback. When reopening a
pending media fence, the owner must resolve each still-present path against
canonical media roots, reject symlinked or outside-root targets, and hand only
the verified target path to the existing custody cleanup. A missing path is
safe to treat as already rolled back; an identity or canonicalization error
remains outcome-unknown and keeps the fence.

The same existing owner also stores one migration-safe pending external-memory
mutation journal. It contains the canonical session identity and lifecycle
revision, one mutation identity/kind, and the exact pre-mutation memory records
plus origin/tombstone metadata needed for rollback. The journal is written
before the external file-store mutation; a successful owner-local snapshot
clear removes the journal in the same SQLite transaction. A journal found
during owner open is not evidence of success: the owner restores the captured
records and metadata through the canonical memory owner and clears only the
matching journal, otherwise it retains the journal and fails closed. Archive,
restore, deletion-fence, and other lifecycle-revision transitions reject a
pending journal at the owner edge; the production owner may first complete the
same-generation recovery.

The current background-revival producer was rechecked on 2026-11-01. It
constructs a canonical main-session key, a per-run `clientNonce`, and a
`streamId`, captures the admission generation, calls the existing routed-send
admission, waits for the matching runner terminal event, rechecks the generation,
and commits the result through the same `SandAgentDb` owner. Identical retries
return the committed receipt and conflicting payloads fail closed. This bounded
path still does not represent the full upstream detached-result contract or
provide archive/restore, media, or provider-review ownership.

Automation-session identity resolution also rechecks the owner admission fence
before returning the existing owner-admitted main key to upgrade-resume or
automation dispatch. The per-run cron key remains in the durable
`cron_session_runs` projection and is not substituted into the owner admission
record until a real per-key session owner exists. A matching key in an
archived, deleting, or otherwise malformed record is not sufficient evidence
for producer admission.

## Ownership and data flow

`Gateway/Runner/background producer → ProductionSessionWorkers/SandAgentDb →
session admission record + transcript transaction → existing Runner,
transcript, and provider-review consumers`.

The owner must recheck the captured generation and deletion/archive state at
the actual durable commit edge. Unknown mutation outcomes must not be retried
without a committed receipt.

The existing atomic session-admitted transcript append, transcript update,
background-result receipt, and outgoing-media settlement transactions must
also reject the durable `deletion_pending` fence inside their SQLite
transaction; a production precheck alone is not sufficient for a delete race.

The shared direct `SandAgentDb` metadata/KV predicate and the dedicated
session-scoped transcript, media, result, spend-guard, compaction, and
persisted-update transactions must fail closed when an existing admission
record has an empty session key, session ID, or lifecycle revision. Legacy
databases without an admission record retain their compatibility path;
malformed durable admission state is not admitted.

Both owner-local KV writes and deletes use that same transaction-bound
predicate. A session-owned cleanup operation such as clearing an awaiting,
memory-snapshot, or profile-snapshot key must not bypass an archived or
deletion-pending fence merely because it removes a value rather than adding
one.
Receipt replay paths perform the same current-record check before returning an
idempotent result; a stale receipt cannot bypass the live session generation.

The automation spend-guard producer uses the same owner boundary for a
session-bound widget card: card insertion, spend-guard KV state, and activity
metadata commit together, or none commit. Legacy databases without a session
admission record retain their existing compatibility path and remain subject
to the facade admission checks, including direct spend-guard state responses.
Both spend-guard mutation entrypoints also reject before changing the
automation store, so deletion cannot leave a routine-side effect without its
owner-local state transition.

Host turn memory/profile prompt snapshot setters are likewise admitted through
the same owner fence. The memory gateway's paired clear paths now use an
owner-composed mutation path because they mutate the external memory store
before clearing the SQLite snapshot. The selected contract is now an
owner-composed memory mutation path with the
same per-agent filesystem serialization boundary and exact record/metadata
restore on snapshot-clear rejection; the external memory store now has its
own durable journal recovery path, while profile/avatar file recovery remains
the separate atomicity gap addressed by SWA-12.

The file-backed agent profile and avatar mutations remain a separate atomicity
boundary. Profile updates write `profile.json`; avatar replacement or removal
also deletes or writes conventional avatar files and, on legacy removal, may
clear an SQLite avatar pointer. The send-acceptance name seeding path is a real
producer of the profile update. A facade admission check alone cannot make
these filesystem and SQLite effects one owner transaction, so this slice does
not claim their deletion-race closure. The selected closure is a per-agent
session-owner serialization boundary shared by profile/avatar mutation and
directory deletion; the boundary must be held across the complete mutation,
with admission rechecked while held. SWA-12 now extends that same owner
boundary with a durable before/after journal and owner-reopen recovery; its
focused recovery contracts and exact-head verification remain pending.

The selected crash-recovery extension uses one additional pending profile
mutation record in the existing `SandAgentDb`; it is not a second database or
profile owner. The record is prepared under the same per-agent filesystem
serialization lock before profile/avatar files change. It carries exact
allowlisted before/after bytes and the expected owner-local `SandProfile` when
avatar removal changes the legacy SQLite pointer. A successful owner commit
updates that pointer and removes the journal in one SQLite transaction. A
reopen compares every affected file with the recorded before/after states,
restores only an exact after-state, clears an already-before state, and
retains the journal/fence for any mismatch.

Owner-open recovery for media custody, external memory, and profile/avatar
journals runs while holding that same per-agent filesystem/deletion
serialization boundary. Producers that already hold the boundary use an
internal owner-open entrypoint that does not reacquire the non-reentrant lock;
ordinary owner opens acquire the boundary before identity validation and
recovery. This keeps reopen recovery from interleaving with directory deletion
or another external filesystem mutation without introducing a second lock or
owner.

The public production admission facades likewise acquire this boundary before
reading the owner record. Filesystem-backed producers use paired locked
helpers while composing admission with their external mutation, so the
non-reentrant lock is never acquired recursively.

The same fence covers owner-local viewed/unread projections, the persisted
session profile projection, conversation-partner membership, and profile
snapshot clearing. These are session metadata mutations, not alternate
lifecycle owners.

## Failure and edge cases

- missing or malformed session identity: reject before durable admission;
- missing agent/session database after directory cleanup: reject before any
  external store can recreate per-agent state;
- session generation changed: return stale/retryable result without mutation;
- deletion pending: reject new work and retain owned terminal writes;
- archived or restart-tombstone session: reject work until authorized restore;
- duplicate result: return the original committed outcome;
- same idempotency key with different payload: fail closed;
- active media/subagent work: preserve it during archive/reaper decisions;
- cancellation/provider failure: leave prior transcript and generation intact;
- uncertain deletion/transcript result: report outcome-unknown and do not retry.
- pending media recovery: reject symlinked, canonical-identity-mismatched, or
  outside-root targets before cleanup; do not convert a lexical prefix match
  into custody proof. Full descriptor-bound directory identity remains part of
  the separate media read-scope responsibility.
- external memory mutation crash window: a durable journal without a committed
  owner clear is rolled back from its captured records; malformed identity,
  snapshot mismatch, non-canonical store, restore failure, or journal CAS mismatch remains
  outcome-unknown and keeps the owner fence.
- upgrade-resume marker recovery: do not clear the pending set before dispatch;
  clear only the exact marker after a successful accepted resume or an explicit
  terminal-lost disposition (for example, a deleted agent or non-resumable
  group); preserve the marker on identity, dispatch, or unknown failures, and
  never remove a newer marker written for the same agent during recovery.
  A marker with an incomplete automation identity must fail closed and remain
  pending; it must not be reinterpreted as a direct-main resume.
- pending-wake rearm must apply the same exact-marker rule: a rearm attempt
  may clear the observed wake only after successful ownership, while a failed
  attempt must preserve it and must not overwrite a newer marker with the same
  `(agent, kind, work)` key.
- provider-review continuation dispatch failure: restore the review to
  `pending`; Runner cancellation/failure/unknown: retain terminal facts in a
  non-actionable phase and clear the matching terminal work marker; Runner
  success: clear the matching review and work marker.

## Implementation strategy

1. Add migration-safe owner-local session admission schema and compare-and-set
   methods to `SandAgentDb` and expose them through `ProductionSessionWorkers`.
2. Bind the canonical Runner admission boundary to `SWA-01/02`.
3. Add background-result settlement with captured generation and idempotency.
4. Add a durable owner-local pending-media identity around existing
   `OutgoingMediaCustody`; materialization, transcript acceptance, rollback,
   and archive checks must use the same session generation.
5. Add the migration-safe external-memory mutation journal to the existing
   `SandAgentDb` owner, bind remove/clear producers to its prepare/commit/
   abort operations, and run exact-store rollback during owner open.
6. Add the profile/avatar crash-recovery journal to the same `SandAgentDb`
   owner, including exact allowlisted file-state recovery and the atomic
   legacy-avatar-pointer commit.
7. Connect the authenticated Host Gateway archive/restore producer to the
   owner-local transition and then connect the existing provider-review
   terminal owner.
8. Add direct negative contracts, then perform exact-head verification.

## Verification and acceptance

Focused contracts must prove disabled/archived/deletion-pending admission,
stale generation, duplicate and conflicting result, lifecycle change during
restore, active media/subagent preservation, terminal review ownership, and
crash recovery of a journaled external memory mutation.
Profile/avatar recovery must additionally prove before/after matching,
legacy-pointer transaction ownership, malformed/rebound rejection, and
archive/delete exclusion while the journal is retained.
Static presence is insufficient. The parity row remains
`implementation=not-started`, `wiring=not-wired`, `verification=not-run`,
`acceptance=pending` until all production edges and required exact-head
evidence exist.

## Current implementation slice

`ProductionSessionWorkers::assert_agent_work_admission` now fences the
existing transcript append/update/delete, compaction, and conversation-clear
wrappers, activity metadata writes, and awaiting-user-response metadata writes
plus introduction lifecycle metadata while `begin_agent_delete` owns the
deletion window. `SandAgentDb`
also now owns a migration-safe `sessionWorkAdmission` KV record with
compare-and-set lifecycle generation transitions, exposed through
`ProductionSessionWorkers`. New sessions initialize a direct-main record, and
the existing `call_send_prompt` plus local routed producer paths check that
record before admission. Focused contracts cover rejection before durable
write, admission after the fence clears, create-only migration, guarded
archive transition, archived routed-work rejection, and stale transition
rejection. This still does not wire the full detached background-result
lifecycle or native provider-review runtime recovery, so the responsibility
remains open even though the bounded provider-review terminal settlement path
is now wired.

The background revival path now also captures the active main-session
`lifecycle_revision` before routed admission and rechecks the same generation
at the matching `Completed` terminal event. A changed, archived, deleted, or
missing record is rejected before the revival reports success. This closes the
generation-observation guard only; it does not provide the durable result
receipt, or provider-review consumer required by `SWA-03`–`SWA-06`; the
bounded Gateway archive/restore producer is now present.

The current slice adds a same-owner terminal receipt for that revival path:
`SandAgentDb` writes a hidden `background-result-receipt` transcript entry and
clears the admitted work marker in one SQLite transaction. The deterministic
receipt identity is the idempotency key; an identical retry returns
`AlreadyCommitted`, while a different payload for the same key returns an
explicit conflict. Admission is released on pre-run rejection, cancellation,
timeout, disconnect, and terminal failure. This is wired only for the local background-revival
consumer; the existing `ProductionSessionWorkers` transcript/page/thread
projections filter the receipt kind so it cannot enter user-visible context.
It does not yet establish the full detached-result, media, or provider-review
lifecycle required for responsibility completion; the bounded Gateway
archive/restore producer is now present but intentionally rejects active
admitted work.

The provider-review durable transitions now also recheck the owner-local
`deletion_pending` fence at their SandAgentDb edge. Recording a refusal,
releasing a claimed continuation, or settling its terminal outcome cannot
mutate review state after deletion custody begins; the existing generation and
CAS semantics remain unchanged. This is a bounded deletion write-edge
correction, not completion of native continuation identity or the full
detached lifecycle.

The same owner-local transaction fence now covers request-id history and
pending episode-memory writes, including the Runner's direct episode clear
path, as well as the owner-local transient-state clear. A deletion fence
therefore rejects those metadata mutations at the SQLite write edge; this
now also covers metadata CAS writes used by transcript/checkpoint owners.
These remain bounded owner-local corrections and do not close the separate
checkpoint blob or external memory-store atomicity gaps. Profile/avatar
deletion-race serialization is described below, but cross-process crash
recovery remains separate. The
checkpoint transaction now rejects confirmation under the durable fence, while
both confirmed and unconfirmed Runner checkpoint paths use the captured-root
CAS; a stale checkpoint cannot overwrite a successor root or lifecycle
revision. The already-materialized external blob remains a separate
cleanup/recovery responsibility. Session
maintenance now routes its persisted-root CAS through the same `SandAgentDb`
owner instead of maintaining a second direct SQLite mutation path.
Runtime profile-name recovery maintenance now uses the same owner metadata
path as well; clone/materialization initialization remains a separate
pre-admission setup path. Agent cloning now explicitly reseeds the copied
`sessionWorkAdmission` record with the new canonical agent/session identity,
a fresh lifecycle revision, and no inherited work, archive, deletion, media,
or provider-review custody before normal admission resumes. Clone setup also
clears copied pending external-memory and profile/avatar mutation journals from
the target database: those journals are source-agent crash-recovery custody and
must never be replayed or rebound under the new agent identity. This cleanup is
performed in the existing `SandAgentDb` setup owner before normal admission;
the source database is not modified.
Transcript backfill maintenance now also delegates to the same owner append
transaction, so recovery cannot append entries while deletion custody is
active.
Pending-card maintenance fallback updates use the same owner update
transaction and are likewise rejected during deletion custody.
Hidden-entry repair cleanup now delegates deletion to the same owner
transaction and reports no deletion while custody is active.
An admitted transcript deletion returns committed success even when the row
is already absent; closed, busy or fenced attempts remain non-success. An
already-absent replay does not advance the write generation or publish a new
mutation. The boolean acknowledges the admitted delete operation, not a new
row removal, so maintenance does not misclassify an idempotent replay as an
owner refusal.
Transcript identity settlement used by Human handoff/replay also uses the
same transaction-edge fence.
For a Human-conversation projection, `sessionKey` is the source Agent's
provenance, not the Human transcript's admission identity. The Session owner
must validate that exact key against the source Agent admission while holding
the existing per-agent lifecycle boundary through append or provenance
enrichment. The Human conversation keeps its own existing transcript owner
and must not acquire an Agent admission record. Missing, mismatched,
archived or deletion-fenced source admission rejects a bound projection
without changing the Human transcript. Legacy unbound Host-internal results
retain their existing path. Replay preserves stream/content/source identity;
concurrent duplicates do not append twice. Oracles use a real source Agent
and Human owner, test new bound results and legacy provenance enrichment,
then assert refusal and unchanged rows after source archive/deletion or
identity mismatch. This repairs the cross-owner regression found by Actions
`41397585774` on `5f0d548c325998ce4dcbedd1f1f04f91489df323`; it does not
grant renderer access or claim external message delivery.
Conversation clear now uses the same owner transaction and is rejected during
deletion custody. Its external blob cleanup now also runs under the shared
per-agent filesystem/deletion serialization boundary, so it cannot interleave
with directory deletion or profile/memory filesystem mutation. The facade's
pre-cleared external blob store remains a separate cross-storage crash
atomicity gap; this change does not add a second blob journal or claim recovery
after a process interruption between the blob cleanup and owner transaction.
Pruned-placeholder reclamation uses the same boundary through owner/worker
close and directory removal, while retaining an explicit failed-reclaim
outcome for close or removal errors.
The cached `ProductionAgentStore` owner is also retired before the corresponding
`SandAgentDb` owner and agent directory are closed/removed in both deletion and
pruned-placeholder reclamation. The current clone path uses a new agent
identity and does not reuse a source cache key; no successor-cache handoff was
found in the static audit.
Channel credential/configuration producers that write the per-agent channel
directory (`store_connector_credential` and `disconnect_channel`) now acquire
the same boundary and recheck admission while holding it. Secret-file and
channel-config updates remain separate existing stores; this closes the
in-process deletion interleaving only and does not claim cross-store atomicity.
Production Gateway and Runner settings writes now use the same existing
`ProductionSessionWorkers` owner; the Runner state surface injects a settings
writer callback so its `settings.set` command does not write `settings.json`
directly. The standalone `SandAgentState` fallback remains for non-production
construction and is not production acceptance evidence.
Session-scoped model selection now uses the same owner boundary: the
`SandAgentSessionStore` facade delegates evaluation and persistence to
`ProductionSessionWorkers`, which holds the per-agent filesystem/deletion lock,
rechecks admission, and writes the existing `sessionModelSelection` KV through
the canonical `SandAgentDb` generation-CAS API. A missing or malformed
admission record, or a lifecycle change during policy evaluation, fails closed.
The model-picker policy owner remains unchanged; this closes only the
owner-serialization/generation gap and does not create a production mutation
ingress, so model-selection wiring and acceptance remain pending.
The Runner `channels.disconnect` command likewise injects the existing channel
owner instead of deleting the channel directory directly; connector-secret
removal uses the same admission boundary.
The existing Gateway automation/workflow producers now use a shared admitted
filesystem-mutation wrapper around definition, enable/disable, removal, and
workflow import operations. This protects agent-directory writes during
deletion without turning automation history into a session catalog or cron
reaper.
The production Runner state surface now injects automation/workflow mutation
callbacks that delegate to those same existing Session owner methods; its
standalone fallback remains compatibility-only.
The Runner project `join` and `leave` commands now use the same callback
boundary for the agent-local `projects.json` membership file. The callback
delegates through `SandAgentSessionStore` to the existing admitted filesystem
owner; direct `SandAgentState` construction retains its non-production local
fallback. Project-directory creation remains a separate shared project-store
mutation and is not claimed as an atomic transaction with membership or as a
cron/session catalog.
Runner project creation now delegates both the shared project-directory
creation and the agent membership write through the same existing Session
facade and admission boundary. This prevents a deletion-fenced Runner from
creating a new project as a side effect before admission is checked; the
shared project store still has no separate session catalog or cross-agent
transaction owner.
The Runner avatar set/clear commands likewise delegate their validated bytes
through the existing `SandAgentSessionStore` avatar owner and its profile
mutation journal. Direct `SandAgentState` construction retains its local file
fallback; production Runner avatar mutation no longer bypasses the owner.
The shared owner `write_kv` boundary now fences the remaining owner-local KV
setters at the same SQLite edge; external memory-store clear paths remain
separately governed.
The production `TodoWrite` bridge now uses a thin
`ProductionMultitaskTodoState` adapter backed by `ProductionSessionWorkers`.
It no longer retains a raw `SandAgentDb` as its turn-scoped state handle:
reads stay inside the per-agent serialization boundary, and writes use the
admitted owner wrapper before calling the existing `write_kv` predicate. This
preserves the existing `multitaskTodos` KV schema and tool contract while
closing the Host-to-Runner owner bypass; no second todo store is introduced.
The terminal turn-memory path likewise uses a typed `EpisodeProgress` port and
`ProductionEpisodeProgress` adapter. Production `main.rs` no longer opens a
raw agent DB for episode recording; reads and mutations route through the
Session owner, with read serialization and admitted write fencing preserved.
The existing episode KV format and memory extraction behavior are unchanged.

The transcript lifecycle producers likewise consume typed purpose and unread
projections from `ProductionSessionWorkers` rather than opening a general
`SandAgentDb` handle in the transcript layer. The projection methods retain
the existing owner filesystem serialization and preserve the current purpose
and unread schemas; they are an ownership-surface reduction only and do not
change transcript or lifecycle behavior.

`ProductionAgentStore` likewise keeps its existing typed metadata bridge but
does not export a general `SandAgentDb` accessor. The checkpoint owner remains
the only production consumer of the composed database handle; metadata and
blob operations continue through the existing typed AgentStore methods.

Inactive transcript-session retirement now closes the AgentStore owner, DB
owner, and exact conversation-blob worker under the same per-agent
filesystem/deletion serialization boundary used by deletion. This prevents a
retirement close from interleaving with directory cleanup; it does not add a
second lifecycle owner or change the live-session registry semantics.

AgentStore owner opening now retains that same boundary through DB-owner open,
blob-store recovery, checkpoint reset, and cache installation. A deletion
cannot remove the agent directory between those steps and then leave a newly
opened successor owner cached for the deleted identity.
Public DB/AgentStore opens also reject the in-memory deletion fast-path after
acquiring the boundary; deletion's own lifecycle code uses the locked internal
open path so it can publish and consume its durable fence without recursion.
The legacy public owner-close methods remain source-compatible for existing
callers but now acquire that same boundary; deletion/reclaim paths use their
locked variants through the owner retirement helper.

Profile and avatar filesystem mutations now run through a per-agent
`ProductionSessionWorkers` serialization boundary. Profile updates and avatar
replacement/removal recheck the durable admission record while holding that
boundary, and the existing directory deletion owner holds it through cleanup.
The avatar legacy-pointer clear is delegated back to the same `SandAgentDb`
owner rather than opening a second direct SQLite mutation path. The avatar
mutation snapshots conventional files and restores them when the new file
write or legacy-pointer clear fails. The new profile mutation journal also
holds the exact before/after file state until the owner transaction commits;
owner reopen restores a still-pending exact after-state and fails closed on
any mismatch. Focused recovery contracts and exact-head verification remain
pending.

The memory gateway's remove and clear producers now route through the same
owner boundary. They capture the external records and memory snapshot before
mutation, clear the external store, and clear the owner-local snapshot. A
deletion-fence rejection restores captured records before returning failure;
the existing `SandAgentDb` owner now also records a migration-safe pending
mutation journal before the external change and clears the snapshot plus
journal in one owner transaction. Owner reopen restores any still-journaled
records and metadata through the canonical `MemoryService` store and
exact-clears the journal; malformed or uncertain recovery remains fail-closed.
The focused
crash-recovery contract and exact-head verification remain pending.

Profile/avatar mutation remains owner-serialized and now has the selected
cross-process recovery journal and producer wiring. Its focused recovery
contract execution and exact-head verification remain pending.

The active Runner `update_state` agent-memory forget producer now delegates its
agent-scope removal through the same `ProductionSessionWorkers` owner and
journal; user/project memory scopes retain their existing separate stores.
This prevents the Runner's state surface from creating an agent-memory delete
path that bypasses session admission.

The Runner `update_state` profile producer now likewise delegates agent
profile name/description changes to `ProductionSessionWorkers`; it no longer
writes `profile.json` directly when the production owner callback is bound.

The production synthesis target is routed through a late-bound owner gate. A
target without a bound `ProductionSessionWorkers` gate returns a
typed invalid/stale-equivalent failure rather than writing the memory files;
the bound path accepts only the `MemoryService` canonical store for that agent,
then preserves `FileMemoryStore` fingerprint/CAS semantics inside the owner
lock.

The focused deletion-fence contract is recorded in
`source/host/tests/agent_session_contract.rs::deletion_fence_rejects_new_transcript_work_before_durable_write`;
it now also covers profile-file, avatar, and external memory mutation refusal.
The contract is source evidence only until the frozen verification run
executes.

The owner-local admission CAS now preserves the current `deletion_pending`
value as part of its compare predicate. Ordinary work/review/media mutations
cannot clear a fence that appeared after their read; only the deletion owner's
dedicated clear operation may transition `true` to `false`, and it requires
the fence to still be present at the CAS edge.

The receipt transaction also rolls back the inserted receipt when its
post-insert lifecycle compare-and-set loses a race. This preserves the
`RejectedStale` outcome without leaving an orphan receipt that could make a
later retry appear `AlreadyCommitted` while the admission marker still owns
the work. The focused owner oracle is recorded in
`source/host/tests/agent_db_owner_contract.rs::stale_background_result_cas_rolls_back_receipt_insert`;
it remains unrun during the development freeze.

The 2026-11-01 background-result recheck confirms the bounded receipt path is
now present, but the gap remains at the broader lifecycle boundary:
`run_local_background_revival_turn_with_context` has `agent_id`, canonical
`sessionKey`, generated `clientNonce`, `streamId`, a matching terminal event,
generation recheck, and same-owner receipt commit. The Gateway archive/restore
producer now reaches the same owner-local record with expected
identity/revision guards and rejects active admitted work, active generated
subagent work, and active foreground Runner streams before the durable
transition. The media custody integration now includes durable path capture and
bounded reopen recovery, but full detached media/result replay and the
full provider-review terminal lifecycle remains absent. The next implementation
instrument must
extend the existing session/transcript owner with those explicit outcomes; it
must not replace the receipt with a best-effort CAS or add a parallel store.

The Runner's `request_box_help` transcript publication is also an admitted
session write. Because that producer already carries the canonical
`session_key`, it must use
`ProductionSessionWorkers::append_agent_transcript_entries_if_session_admitted`
so archive state and the durable session identity are checked at the write
edge. The generic agent-only append remains reserved for producers whose
existing ownership is not session-scoped; it must not be used to bypass this
session-bound admission boundary.

The same owner boundary applies when a persisted `request_box_help` entry is
superseded or settled after the Runner returns. Such entries retain the
producer-owned `sessionKey` provenance, and `resolve_box_request_entry` must
use the atomic session-admitted update when it is present. Legacy entries
without that provenance remain on the compatibility path and retain the
generic deletion/archive fence.

The session gateway's accepted-send producer now persists the supplied
`sessionKey`/`sessionId` on staged transcript entries and applies the same
owner-local append when that selector is present; only legacy non-session-scoped
fallback sends retain the agent-only append.

Session-admitted transcript updates must validate the target entry's persisted
`sessionKey` inside the same owner transaction as the admission and update.
Entry existence alone is insufficient: a caller must not use one admitted
session key to mutate a legacy or differently bound entry in the same agent
database after an out-of-transaction provenance read.
The corresponding session-admitted append owner applies the same invariant to
every inserted entry; missing or mismatched provenance is rejected in the
append transaction rather than silently converted into a session-bound row.

## Direct media-custody boundary recheck — 2026-11-01

`OutgoingMediaCustody` in `source/host/src/attachment_paths.rs` now rolls back
materialized files when the `SendMessage` tool path fails and commits them
after the existing transcript entry receives an id. `ProductionSendMessageSink`
also captures a matching session key/lifecycle revision and rechecks that
generation immediately before transcript append. This proves a bounded
foreground tool-path fence.

The media settlement owner must retain both captured values at the final
transcript/receipt transaction: `session_key` identifies the canonical session
and `lifecycle_revision` identifies its generation. A matching revision alone
is insufficient evidence when the producer already has a session identity;
settlement must reject a rebound or mismatched session before clearing the
durable media fence.

The same captured `session_key` must be used for the ordinary generated
`SendMessage` settlement path when no media fence exists, and the generated
entry must retain that key as provenance; it must not fall back to the
agent-only transcript append merely because no media receipt is needed.

Turn-local permission requests now carry the existing Runner
`worker_session_key` through the permission scope and event owner. Their
created card append and settled-card update must use session-admitted owner
transactions when that identity is present; permission requests from legacy
unbound infrastructure remain explicitly outside this session-scoped path.

The same producer boundary applies to Auto-review approvals created by an
active Runner. The Runner already has the canonical `worker_session_key` when
it binds the Auto-review controller, so that identity must be carried on the
approval event through the existing Auto-review service. Created approval
cards and settled/expired card updates must use the session-admitted owner
transactions when the key is present; controller instances created by legacy
unbound infrastructure remain on the compatibility path.

The Runner `SendToAgent` producer also already owns the canonical
`worker_session_key`. Its sender-side transcript append must persist that key
on the outbound entry, carry it through the existing agent-management sink, and
use the session-admitted owner transaction only when the owner-local admission
record confirms that key. If
the producer carries a selector that has no matching owner record (for
example, the current group-member compatibility runner), it remains on the
generic agent-only path until that session record is established. The
recipient-side wake envelope does not expose a canonical target session
identity; its later inbound materialization resolves the target's existing
owner record locally and uses the admitted append when available, otherwise it
retains the generic compatibility path. It is still subject to the owner-local
deletion and archive fences. The producer
must not synthesize a target `agent:{id}:main` selector merely from a target
agent id.

The owner now also carries one migration-safe `admitted_media_work_id` and
archive rejects that field while it is present. This closes the durable
archive fence for the bounded SendMessage path: the sink's process-local
`pending_media` remains keyed by `tool_call_id`, but it is paired with the
owner-local media identity and released only after transcript append or
explicit rollback. It still does not satisfy the full `SWA-03`/`SWA-06`
detached-result contract: the owner now writes a durable media settlement
receipt and has a bounded reopen recovery consumer, but full detached
result/media replay and the placement/database identity chain remain open. A
global in-memory registry or a second media lifecycle owner remains
unauthorized by this spec.

## Exact selector preflight — 2026-11-01

The session-owner preflight now fails closed when a producer supplies a
non-empty selector that differs from the existing admission record. A missing
record retains the existing compatibility behavior for callers that have not
yet established a session admission; once a record exists, a rebound or
mismatched `session_key` is an explicit identity error rather than an admitted
preflight. The atomic transcript/media owner checks remain authoritative at
the durable write edge.

The owner-local compare-and-set also preserves `session_key` and `session_id`
across generation updates. A caller may advance lifecycle state and admission
facts, but may not use a valid prior revision to rebind the record to another
session identity; such a CAS returns false and leaves the previous record
intact.

Archive and restore use the same exact identity at their durable owner edge:
the canonical `session_key`, `session_id`, and optional lifecycle revision are
all checked inside the SQLite transaction before archive state changes.

## Agent-only transcript mutation boundary — 2026-11-01

The remaining transcript mutation producers were rechecked against the live
Rust source. `WidgetResponses` now preserves and consumes `sessionKey`
provenance for interactive updates to bound cards; cards without provenance
remain on its generic agent-scoped compatibility path. Its stale-card sweep
now uses the same owner-local transaction for cards carrying provenance, while
legacy cards without it retain the compatibility path. Shared-room restamp
now uses the same owner-local transaction when the persisted entry already
carries a canonical `sessionKey`; legacy entries without provenance remain on
the compatibility path. New room-owned entries from local group fanout,
activity notices, guest ingress, and mirror ingress now read the target
room-agent's owner admission record, persist that exact `sessionKey`, and use
the session-admitted append when the record exists; a missing record retains
the compatibility path. The generic delete and conversation-clear wrappers
currently have no production caller in the Host source. Automation spend-guard
cards now use the same owner record and session-admitted append when available;
unbound legacy card producers remain compatibility-only. Recipient-side agent
inbound materialization likewise reads the target agent's existing owner
record, persists its exact `sessionKey`, and uses the admitted append when
available; legacy targets without a record retain compatibility behavior.

The box-request path is now split by durable provenance: new Runner-created
entries retain their canonical `sessionKey`, and
`resolve_box_request_entry` uses the atomic session-admitted update for those
entries; older entries without provenance retain the compatibility path. The
Auto-review service similarly uses its event-owned session key for bound
creation and settlement, while unbound legacy controller events remain
compatibility-only.

The still-unbound paths remain explicitly pending for `SWA-01`/`SWA-06`. The
implementation must not synthesize `agent:{agent_id}:main`, infer a session
from an unrelated room or approval identity, or route them through the
session-admitted update transaction until their owning producer captures and
propagates the real canonical session identity. The existing agent-only
compatibility boundary is retained for these non-session-scoped producers, but
it is not an archive bypass: generic transcript and compaction wrappers reject
writes when the owner-local admission record is archived. This is incomplete
evidence, not acceptance.

## Checked production checkpoint-root recovery — SWA-ROOT-RECOVERY-v1

At base `81d48636b06d20f8bec551cdc334fe40f31e614b`, production AgentStore
opening ignores the legacy boolean reset outcome. That compatibility reset
turns a missing or damaged persisted root into empty in-memory state; Runner
watermark, prompt-epoch and checkpoint consumers can then mistake an unknown
durable history for a new session. This is a target recovery defect under
SWA-01/06/07 and OCRM-C02/C10, not a claim that the fixed upstream or all
cross-process recovery is accepted.

The existing Session/AgentDb/AgentStore owners retain their boundaries:

- Existing production agent databases open without seeding absent metadata.
  New-session materialization already writes initialized metadata. A checked
  recovery read must consume one durable metadata image, require its object
  and canonical agent identity, and distinguish its explicit empty root string (including the retained
  whitespace normalization) from absent, null, wrong-typed or malformed
  metadata/root state. Invalid
  metadata is rejected before profile/settings repair, maintenance or caching.
  The referenced checkpoint is also validated before pending media, profile
  or memory recovery can mutate files, journals or admission state.
- An explicit empty root is a valid initialized session without a checkpoint.
  A nonempty hexadecimal root must resolve to readable protobuf bytes. Missing
  blobs, malformed protobuf, read failures and poisoned owner state fail
  closed. The canonical zero-byte protobuf is a valid *present* checkpoint,
  distinct from no root. Legacy fixed root IDs remain readable: this slice
  does not impose a new digest/ID-length rule on retained data.
- Checked reset publishes a replacement in-memory snapshot only after all
  reads and decoding succeed. Failure preserves the prior valid snapshot and
  durable metadata/blob/transcript state, returns an error and cannot install
  an empty production AgentStore. Explicit repair can be followed by another
  open; failure is not cached as permanent absence.
- Production re-entry validates an already-cached root and its bytes against
  durable state. Corruption or a changed durable checkpoint refuses re-entry;
  it must not reuse a stale empty snapshot or implicitly overwrite a competing
  checkpoint publication. Explicit checked reload establishes the new cached
  snapshot; ordinary matching-root reuse keeps the same owner.
- The existing legacy boolean reset and lenient display APIs retain their
  contracts. Production owner open and Runner watermark, compaction epoch,
  checkpoint composition, staging and settlement use checked state reads.
  Unknown state cannot become a no-user-turn watermark or successful empty
  checkpoint; the existing turn lease/error path receives the refusal.
  Checkpoint publication validates the durable root against its cached base
  before writing blobs. The existing database root/confirmation CAS must
  require an explicit string root in its compared metadata image, so missing
  or wrong-typed fields cannot race that check and match an empty base.
  Both CAS operations compare decoded byte identity for nonempty hexadecimal
  roots, accepting the case and outer whitespace already accepted by checked
  recovery. They retain the raw metadata CAS/concurrency predicate. Only the
  literal empty string represents an initialized empty root; whitespace-only
  or malformed hexadecimal strings cannot become an empty-base commit, and
  non-ASCII invalid hexadecimal must refuse without panicking.
- Valid empty-root legacy recovery continues through the existing maintenance
  owner and root CAS. If that owner recovers a root, any already-cached
  AgentStore is refreshed through the same checked reset before consumption.
  Archive/deletion serialization and existing generation/root fences remain.

Actions oracles use real Session workers, SQLite and blob owners: initialized
empty state and valid empty checkpoints; absent/malformed metadata and root
fields; missing/corrupt blobs; refused open with unchanged durable state and
no cached AgentStore; explicit repair and reopen; checked reset preserving a
prior snapshot; pending file journals preserved until valid-root recovery;
refused publication without a new blob or user confirmation; and actual
HostRunnerComposition watermark/epoch/sink refusal and recovery. Retain
legacy reset, materialization, maintenance and checkpoint
regression coverage. All executable validation remains in GitHub Actions.
The continuation oracle must reopen normalized ordinary and legacy fixed-root
checkpoints, then advance both the generic root and real user-confirmation
checkpoint paths. Refused raw roots preserve metadata, transcript, generation
and notifications. Separate owned OS-process controls reopen empty, present
empty, user and legacy fixed-root states; six persisted fault classes refuse
repeatedly without rewriting durable bytes and recover after explicit repair.
These process controls cover storage/Host readiness, not provider replay.

This bounded contract does not add a Runner checkpoint load/claim/settle API,
bind upgrade markers to the original stream/model/generation, or authorize
replaying effects with unknown results. Those process-restart responsibilities
and the full OCRM G01–G12/AC01–AC24 matrix remain open.

## Canonical checkpoint readiness — CKR-v1, 2026-11-02

This necessary Runner recovery prerequisite uses the existing
`ProductionSessionWorkers` / `SandAgentDb` / `ProductionAgentStore` and blob
worker. At base `3aa679586d76e446f76a1c94700ff7abab623d42`, Session open ignores
the legacy reset boolean and canonical consumers turn an unavailable checkpoint
into empty bytes. A damaged persisted root can therefore look like a new
conversation. Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`.
This slice adds no provider disk-resume ingress, PARK state, store, schema or
Agent loop, and does not establish full restart acceptance.

- `CKR-01`: Existing-agent DB open must not seed absent metadata. A checked
  root read rejects absent/malformed metadata, missing or non-string root,
  unavailable owner or invalid hex, and checks canonical agent identity in the
  same metadata image. Preserve hex case and whitespace around a nonempty
  ID without UTF-8 slicing panics. An empty root after the existing whitespace normalization is `None`;
  whitespace-only strings still cannot authorize an empty-base CAS. Nonempty
  accepted hex spellings compare decoded identity while preserving the raw
  metadata race fence. Any nonempty fixed legacy ID that names
  a readable zero-length protobuf is a distinct `Some` checkpoint.
  Generic compatibility reads remain available. Both root CAS operations must
  refuse a missing/non-string current field rather than coercing it to empty.
- `CKR-02`: AgentStore returns a fallible root-and-bytes snapshot. Constructor
  and legacy reset do not establish canonical authority. Checked reset alone
  initializes from the strict DB root; failure retains a failed authority,
  never a valid empty replacement. Publish a new Session AgentStore only after
  checked initialization succeeds. Failed opens must remain repeatable failures.
- `CKR-03`: A canonical checked snapshot confirms its captured root still
  matches persisted metadata and a fresh read of its durable root bytes, checks
  the existing protobuf projection, and checks nonempty identity, presence and
  readability of the five required direct-reference groups: root prompts (1), todos (3), summary (6),
  turns (8) and summary archives (13). Deduplicate reads without losing any
  kind-specific maintenance bound. Preserve exact bytes and unknown fields.
  Legacy fixed roots and references remain readable; neither length nor hash
  can identify their addressing mode. New checkpoint writers still generate
  SHA-256 IDs. This is bounded direct-reference availability, not full
  child/schema/graph validation; plan, legacy archive, file-state and subagent graphs remain with
  their current owners. Inline-subagent fallback stays compatible.
- `CKR-04`: Keep expected-root initialization separate from child completeness.
  Both existing checkpoint handlers require initialized expected-root authority
  before blob I/O, preserve their protobuf-only payload acceptance and existing
  DB CAS, and publish the exact successful root. A compatibility write may
  contain missing children; canonical checked reads must then refuse it while
  legacy hydration keeps its contract. Known non-commit preserves prior cache;
  preflight refusal before any write preserves prior authority and is retryable
  after explicit durable repair; uncertain persistence/commit leaves failed
  authority until checked reset.
- `CKR-05`: Use a nonblocking mutation-in-progress state under the existing
  cache mutex for reset/write publication. Never hold that mutex or a new
  blocking operation lock across blob/DB I/O or synchronous callbacks.
  Reentrant checked mutation/read refuses promptly; legacy getters remain
  readable and an overlapping legacy reset returns false without disturbing
  the active operation. An unwind invalidates unfinished authority. A checked
  snapshot must reject if cache identity changes or mutation starts while its
  out-of-lock validation runs. Typed root-metadata mutation must invalidate
  canonical cached authority when its binding changes.
- `CKR-06`: Every DB-owner open/re-entry validates durable metadata and root
  bytes before pending media/profile/memory recovery or file repair. Cached
  AgentStore re-entry checks its durable snapshot before those effects.
  Production preparation validates metadata before legacy root repair,
  permits repair only for explicit empty root, and checks the resulting
  canonical snapshot before readiness or root cleanup. The existing scanner
  selects the candidate; the AgentStore-owned validator checks selected root
  bytes and direct references before repair CAS. Preserve generation fencing
  and existing turn/todo/summary size limits. No candidate or lost CAS is
  `Ok(false)`; a selected missing/corrupt/unreadable candidate is `Err` without
  root publication. Do not skip it to choose another candidate. Scanner-ignored
  malformed orphans are not classified by this contract.
- `CKR-07`: Host checkpoint-sink composition, watermark and prompt epoch, plus
  both checkpoint sink prior-state comparisons, consume checked snapshots.
  Only `Ok(None)` becomes empty initial bytes. The mirror commit uses the root
  returned by the successful durable commit. Propagate errors before provider
  execution; earlier attachment preparation is not claimed side-effect-free.

Acceptance uses real materialization, owner/worker close and fresh Session
owners, including valid empty/nonempty/unknown-field checkpoints; corrupt or
missing metadata/root/blob/reference and read failures; repeated failed open;
repair then retry; no canonical/transcript-mirror publication on refusal; and
bounded reentrant callbacks. Keep lenient hydration/short-root compatibility
tests, including three-byte and 32-byte fixed non-hash roots and references.
Retain existing fixed-ID maintenance fixtures and their lifecycle/CAS assertions. Run affected compilation, owner/maintenance/composition contracts
and existing recovery controls only in Actions at frozen H. No disk format
changes or automatic corruption repair is introduced; rollback retains all
bytes, failed-custody evidence and the existing source/acceptance gates.

## Failed Host takeover refusal — OCRM-HOST-TAKEOVER-REFUSAL-v1

At base `7382badafe5c3a793835b7760bf3ab54e7fb54c9`, the existing Host lock
owner can remove a known Host's PID file even when that process remains alive
after graceful and forced termination polling. Its acquisition-budget fallback
also overwrites the lock without exclusive creation. Both paths can let a
second Host continue into production startup while another execution owner
may still exist. This is a bounded startup admission defect under SWA-06/07;
it does not establish an original-turn recovery contract.

The existing `host_lock.rs` owner and PID-file format remain in place:

- `OCRM-HTR-01`: After both bounded termination polling phases, a known Host
  that still reports alive makes acquisition return `io::ErrorKind::TimedOut`.
  Preserve the lock file and its PID; do not publish a new acquisition handle.
- `OCRM-HTR-02`: Every successful acquisition still requires exclusive
  `create_new`. Exhausting `MAX_ACQUIRE_ATTEMPTS` returns
  `io::ErrorKind::WouldBlock`; it must not truncate or overwrite a contender's
  lock. Recheck the observed PID before stale-lock removal and skip removal
  when the visible holder changed during the liveness/termination probes.
- `OCRM-HTR-03`: Preserve fresh acquisition, dead-holder reclamation,
  successful graceful or forced known-Host takeover, existing same-PID and
  malformed-PID compatibility, and the existing foreign-PID policy. Do not
  signal a foreign PID or add a second lock/runtime owner.
- `OCRM-HTR-04`: The shipping `main.rs` caller already returns on an
  acquisition error, before managed box-daemon and subsequent production
  startup. Keep that existing error propagation; no app-main or public schema
  change is required for this slice.

Actions oracles use real temporary lock files with injected deterministic
liveness, termination and delay callbacks. Cover a known Host that survives
both signals with unchanged bytes, forced termination that succeeds during
polling, repeated visible holder replacement until the acquisition budget
expires without overwriting the last contender, and fresh/dead-holder
controls. Retain the existing successful graceful-takeover and foreign-PID
assertions. Shipping compilation and the affected bootstrap/wiring contracts
must pass on the frozen exact H; no local executable validation is allowed.

This bounded correction does not make PID-file read/remove an atomic
cross-process CAS, solve PID reuse or prove process-tree/tool-effect
termination. Durable Runner claim fencing, unknown-effect reconciliation,
marker preservation and original-stream/provider/model/generation recovery
remain required. There is no state migration; rollback must not erase a
pending recovery marker or assert that an unknown tool result is settled.

## Shipping composition source oracle — 2026-11-02

`SWA-COMPOSITION-ORACLE-v1`: the recovery gate at H
`4fe16f4689e23da2f027d09b2e8d39f56755c6ec` (Actions `41414234389`, job
`125174410868`) misclassified a `cfg(test)` compaction fixture as shipping
Runner construction. This is a test-oracle defect; the real Host still calls
the existing `HostRunnerComposition` owner. Preserve every ownership positive,
negative and ordering assertion. Source probes must inspect the production
prefix before the explicit `cfg(test)` tests module, require the real main
entrypoint and fail closed if that boundary is absent or invalid. Controls
must keep a forbidden production occurrence visible and exclude fixture-only
occurrences from both positive and negative evidence. Retain the existing
Actions target and all gates. Independent review and exact-H Actions are
required; no production behavior, source authority or completion flag changes.
Rollback restores the test-only scope defect and requires no data migration.

Boundary control `SWA-COMPOSITION-ORACLE-02`: a textual opener is insufficient
proof of terminality. A completed inline or indented test module followed by a
production constructor must be refused, including an indented later function;
neither a later brace nor a second test-module marker can hide that code. The
test-only reader uses the already locked `syn=2.0.119` parser (`full`, `parsing`,
default features disabled) to require one final top-level `#[cfg(test)] mod tests`
with a body. Independently parse the selected prefix, require its actual `main`
item and matching item count, then retain every existing source predicate.
Add causal inline/indented/trailing-source, duplicate-marker, missing-close and
test-only-positive controls. No production dependency or owner is introduced.
The existing crate checksum and transitive versions stay locked; only the Host
test dependency edge changes. The pinned upstream Syn manifest and
`parse_file`/ItemMod/Attribute APIs were inspected at its `2.0.119` tag; its
MIT-or-Apache-2.0 license and parser feature boundary remain in force. This is
source-oracle boundary evidence, not a full semantic architecture verifier.

### Group-member admission oracle — SWA-GROUP-ORACLE-01

Run `41416993406` at `f2f2ba00d66b3db79f9cd46690b3ce3737fe296b`
exposed a TEST_FLAW in `routed_send_acceptance_contract`: its positive
group-member fixture omitted the selector required by the existing trusted
route contract, so production correctly returned `MissingSelector` before
the running/badge oracle. The actual `run_local_group_member_turn` supplies
`agent:{member_id}:group:{room_id}` or an existing admitted selector.

Give the positive fixture that canonical group selector and retain every
running, user-turn badge, source, provider and lease-settlement assertion.
Keep absent and mismatched-agent selectors as explicit negative controls:
no persistence callback, queue admission, active source or routed lease.
Do not enable direct-main fallback or change production authorization.
Existing Actions already selects this target. A fresh independent review
and exact-H affected Actions must pass; prior positive results remain
evidence only for their exact subjects and unchanged scope.

## Checked upgrade-marker handoff — SWA-UPGRADE-MARKER-CHECKED-v1

At exact target base `7382badafe5c3a793835b7760bf3ab54e7fb54c9`, an unreadable
or malformed upgrade-marker file is treated as empty, failed writes are
discarded, and replacement can delete the previous file before a failing
rename. This amendment repairs that target defect under SWA-06/07. Fixed U
remains `1440bb1771372334bea23d7642bb52deb7db1845`. The existing
`SandUpgradeResumeStore`, Transcript runtime/manager and HostUpgrade hooks
remain the only owners; no new recovery scheduler or persistence authority is
introduced.

- MARKER-01: checked production discovery distinguishes a missing file from
  malformed JSON/envelope, unsupported version, invalid row identity, I/O
  failure and poisoned owner lock. Only `NotFound` is automatic empty state;
  a valid explicit empty `pending` array is also empty. Current version 1 and
  the legacy versionless `{pending: [...]}` envelope remain accepted when
  every row has a nonempty agent identity, finite numeric timestamp, correctly
  typed optional strings and consistent automation/run identity. Duplicate
  agent rows are ambiguous and rejected. The existing explicitly lenient
  parser/coercion API remains unchanged for compatibility inspection.
- MARKER-02: checked batch upsert validates every existing and incoming row
  under the same store lock, then publishes the whole batch once. It retains
  a later timestamp for the same agent (and the existing row on a tie), never
  publishes a prefix, and preserves original target bytes if replacement
  fails. A unique create-new temporary file is flushed before rename; there
  is no delete-target-then-retry fallback. This is same-owner serialization,
  not a cross-process file lock or a claim that two stores are transactional.
- MARKER-03: exact clear returns `Ok(false)` for stale/absent identity and an
  error for failed reading or mutation; an old recovery pass cannot delete a
  newer marker. Compatibility mutation wrappers, if retained, delegate to
  checked operations and cannot overwrite/delete corrupt or poisoned state.
- MARKER-04: upgrade quiesce persists the complete running-agent batch before
  reporting successful runtime quiescence. The manager issues the registry
  quiesce request only after persistence succeeds: that request cancels live
  tasks and cannot be undone by resetting its flag. Preparation and explicit
  resume are serialized in the same manager. Runtime holds its existing
  admission-state guard across running-agent snapshot, marker write and
  quiesce flag publication. Both actual send admission paths check that flag
  under the same guard before starting new work. Existing in-flight nonces,
  durable replay records and already-accepted queued turns retain their
  coalescing, digest checks, result and lease ownership; quiesce must not
  reject/discard their previously accepted state on wait re-entry. The only
  shipping registry registration path requires that exact accepted lease,
  including group-member turns. No new scheduler or admission state is added.
  A failed batch releases the guard without flag publication or task
  cancellation; subsequent admission remains possible. Kickstart
  marker source normalization to `turn` occurs in that batch; Host composition
  must not consume introduction intent before the durable batch succeeds.
  Failure propagates through the existing fallible HostUpgrade peer hook and
  Gateway response. Previously suspended Automation/sharing owners remain
  suspended for an explicit retry/resume; this slice does not silently undo
  their independent state or claim a cross-owner transaction.
- MARKER-05: startup performs checked discovery before calling any per-marker
  scheduling/HTTP/provider callback. Invalid stored state returns through the
  existing `upgrade_resume_boot_failed` path without dispatch or deletion.
  Per-marker cleanup uses checked exact clear and propagates I/O errors.
  Gateway resume imports markers as one checked batch and consumes a checked
  ID projection before resume/dispatch. Its `resumed` count means accepted
  dispatches, not the number of discovered markers. The prepare response also consumes a
  checked projection. Failure cannot be reported as empty recovery success.

The bounded production scope is the existing marker store, Transcript runtime
and manager, shipping Host marker preparation/discovery/commands, and affected
durable-store, runtime, manager, HostUpgrade service/production, kickstart and
shipping behavior oracles. Tests retain legacy parser, normal quiesce, peer ordering,
exact-marker and pending-wake assertions. New negative cases use real owner
files and state: corrupt/unsupported/invalid rows, lock poison, failed batch
commit with unchanged prior bytes, stale clear, quiesce refusal, checked error
propagation, and zero startup dispatch on failed discovery. An actual marker
writer barrier must prove new routed/execute sends cannot cross a successful
snapshot/write/flag boundary, while failed persistence leaves admission and
existing cancellation tokens live. Existing queued and coalesced turns must
still settle or replay under their original owner. Deterministic
replacement fault injection supplements real file I/O; source wiring checks
supplement actual owner behavior. All executable validation is GitHub Actions
at the integrated exact H, with independent frozen review before acceptance.

Markers remain discovery hints. The existing valid-marker hidden recreation
behavior and clear-after-accepted-dispatch rule are not original-turn recovery.
Original stream/model/generation/checkpoint claim and final settlement remain
open. Asynchronous CompletionRevival marker failure propagation, Automation
observer handoff custody and deletion across pending-wake/marker stores also
remain open where their existing void interfaces cannot return an error. This
batch makes their marker mutation non-destructive and reports failures; it
does not claim those full lifecycles, unknown execution results, restart or
the global OCRM acceptance matrix are closed.

## Release and rollback

Cloud-agent wake rearming does not settle background work. The existing
pending-wake owner retains the exact durable CloudAgent marker while its
watch is armed, including a repeated startup/rearm pass. Only matching
completion settlement or the existing explicit lifecycle/stale disposition
may clear it; an older rearm must preserve a replacement marker. Failure to
arm retains recovery custody. Rearm consumes an already persisted marker and
must not reinsert it after a concurrent terminal or lifecycle owner removes
it; startup/carry/watch admission remains the persistence producer. Tests
must cover settlement after the durable snapshot but before the first armed
check. Recovery watch registration uses an atomic exact-marker replacement
in the existing pending store and skips the fresh-work pending callback;
absence or replacement rejects registration before the completion worker or
pending notifications start. Normal new-work watch admission is unchanged.
Tests cover a second rearm of an already-armed watch, reopening the pending store,
arm failure, replacement races and completion during watch registration. This
is the OCRM-C04/R14 recovery correction for run `41397585774`, not proof of
real cloud service or cross-process Runner acceptance.

The schema must be backward-compatible with existing agent DBs and written
through the existing migration/transaction owner. Rollback removes the new
producer bindings while retaining readable state; no rollback may delete an
outcome-unknown transcript or result.

## References

- `projects/openclaw-rust/decisions/2026-11-01-session-work-admission-owner-contract.md`
- fixed upstream `src/sessions/session-agent-work-admission.ts`
- fixed upstream `src/sessions/background-session-result.ts`
- fixed upstream `src/sessions/session-archive-restore.ts`
- `source/host/src/extensions/session/production.rs`
- `source/host/src/extensions/session/agent_db.rs`
- `source/host/src/extensions/transcript/production_runtime.rs`
