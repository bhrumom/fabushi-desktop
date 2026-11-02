# Channel inbound worker handoff

Contract: `OCRM-CHANNEL-IDLE-HANDOFF-v1`  
Status: implementation in progress; exact-H verification and full ingress acceptance pending  
Original base: `81d48636b06d20f8bec551cdc334fe40f31e614b`  
Integration parent: `d94aeb372c2fe4483bcf60ebbc5aaae4e6db52ef`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`

## Problem and existing owner

The shipping `wakeForInbound` gateway enqueues an envelope and claims inbound
revival under one `BackgroundWakes` mutex. The worker instead takes an empty
batch and later clears revival under another lock. An ingress between those
operations sees a worker, declines to spawn, and is left pending without one.

Keep the existing queue, mutex, gateway and worker. This is an atomic idle
handoff within one live TranscriptManager lifetime, not a new Agent loop,
scheduler, transport, transcript store or channel registration.

## Requirements and production flow

- IH-01: `BackgroundWakes::take_pending_inbound_or_end_revival(&mut self,
  agent_id)` returns `Some(nonempty_batch)` and retains inbound revival, or
  removes that agent's revival bit and returns `None`. A stored empty vector
  is empty. The operation does not change another agent or queue family.
- IH-02: the actual inbound worker calls this operation under the same mutex
  as enqueue. On `None` it returns immediately, with no later removal of the
  revival bit. A new ingress may already own that bit for a successor worker.
- IH-03: extract the existing queue-drain control into a production-used Host
  function accepting the atomic take-or-end operation, batch processor and
  nonempty-stop cleanup as callbacks for a causal concurrency oracle. The
  processor returns whether draining continues; `None` from take-or-end must
  never invoke stop cleanup. The existing nonempty pause/error exits retain
  their current cleanup. Tests pause after a real idle release before returning
  `None` to this same drain control, admit a successor, then resume the old
  control and assert the successor still owns revival. No
  new background thread, timer, retry policy or state is added.
- IH-04: enqueue-before-idle is consumed by the current worker; idle-before-
  enqueue admits a successor. After idle release, a predecessor's return
  cannot clear successor ownership. Batch order and the existing single-worker
  coalescing decision are preserved.
- IH-05: all Session admission, append, provider selection, Runner dispatch,
  delivery, activity and DM-preemption behavior remains with current owners.

## Verification and limits

Actions tests must execute the real queue owner and production-used drain
control with deterministic channels/barriers: both mutex orderings, successor
admission while the predecessor is returning, exactly one batch claim, empty
vector handling, and cross-agent/queue-family isolation. A fake processing
callback is allowed after a real claim; copying the drain loop into a test or
checking source text alone is not the concurrency oracle. Shipping Host
compilation and existing background-wake/manager contracts remain selected.

At the IH-v1 base, append errors dropped the claimed batch and their String
errors could not distinguish known rollback from unknown commit. The CA-v1
continuation below addresses that live-owner custody boundary. Deferred/provider/quiesce and
spawn-failure batches have no autonomous inbound rearm. Detached workers also
lack a dispose/recreate generation fence. These remain open production work;
this batch must not add blind replay or claim persistence, transport ACK,
restart, stop/recreate, real-channel or full ingress-to-delivery acceptance.

No data or protocol migration is needed. Rollback restores the prior drain
control and does not remove user state. Existing telemetry remains unchanged.
Source review uses the fixed-U ingress monitor's coalesced wake/final-repump
invariant; exact identities and read limits belong to the existing channel
dossier. The pure debounce contract and its unwired status are not promoted.

## Append custody continuation — OCRM-CHANNEL-APPEND-CUSTODY-v1

Implementation base: `febef8ba077fd667ce2c06f4b0cdd3edd1570cde`.
Integration parent: `3dea753726c6e2751252ce4ed2a68b17cfdc97cf`; retain its
genuine background-terminal supervision, Coordinator control retirement and all
locked shipping/regression selectors.
Fixed U and all IH requirements remain unchanged. This is a necessary ingress
contract in the existing Gateway → BackgroundWakes → Session/SandAgentDb →
Transcript → connector Runner chain. The shipping append currently treats a
failed transcript read as empty history and ignores the number of rows inserted
by the generic `INSERT OR IGNORE` owner. A failed/partial append can therefore
discard claimed input or run the Agent without the complete admitted input.

### Contract and state ownership

- CA-01: `BackgroundWakes` retains one head claim per agent while append and its
  Runner handoff are in progress. The claim owns the original ordered envelopes,
  a Host-generated opaque claim ID, a per-acquisition worker token, preparation
  and typed state. Later ingress remains behind that head. Claim and idle release
  use the existing queue mutex; no database I/O, callback or Runner runs under it.
  Token-mismatched or late workers cannot settle the head or clear a successor.
- CA-02: add a channel-specific Session/AgentDb API, leaving all generic append
  semantics and SQL unchanged. The existing per-agent filesystem/admission lock
  encloses owner opening, preparation and initial append. The DB transaction
  checks the actual metadata/admission identity, active/archive/deletion status,
  session key, session ID and lifecycle revision. Capture that binding before
  reading history. A checked history read must reject closed ownership, storage
  or decoding errors; it must never use an empty-history fallback. Prepare exact
  entry IDs, content, provenance and default timestamps once. A retry retains the
  binding and complete prepared entries and never remints IDs from newer history.
- CA-03: the shared Rust API is
  `append_channel_inbound_entries(agent_id, &mut Option<ChannelInboundAppendAttempt>, prepare)`
  on `ProductionSessionWorkers`, with its corresponding existing DB method.
  `prepare(&SessionWorkAdmissionRecord, &[Value])` is a pure, once-per-attempt
  closure returning the complete batch or a typed failure. An attempt contains
  `session_key`, `session_id`, `lifecycle_revision` and `entries: Option<Vec<Value>>`.
  `None` entries means binding captured but preparation not completed; `Some([])`
  is the existing already-displayed DM-redrive case, not missing input.
  `ChannelInboundAppendOutcome` is `Committed { inserted_count }`,
  `NotWritten(ChannelInboundAppendFailure)` or `Unknown(ChannelInboundAppendFailure)`.
  Failure kind distinguishes admission, closed owner, storage, invalid batch,
  identity conflict and unconfirmed rollback; classification never parses text.
- CA-04: Committed requires complete valid input and every strict insertion plus
  a positively acknowledged COMMIT. Validate nonempty, unique IDs, message kind,
  user role and matching session provenance before mutation. An ID conflict or
  later insertion failure rolls back the whole batch; no partial publication.
  NotWritten requires no possible insertion or positively acknowledged rollback
  before any COMMIT attempt. Any failed COMMIT acknowledgement remains Unknown,
  even if a later rollback succeeds. A rollback failure is Unknown; retire the
  exact connection under its owner before reuse if transaction state is not
  proven clean. Retirement does not prove replay is safe. Publish inserted rows
  only after a confirmed commit, outside both the DB and per-agent filesystem
  locks. A notification may re-enter the public Session read API. No new schema
  is required.
- CA-05: only Committed reaches the production activity/prompt/Runner handoff.
  NotWritten retains exact preparation and pauses without spin; a later existing
  wake may retry that same head. Unknown is sticky and retained, blocks automatic
  append and Runner, and prevents later input overtaking it. Never reinterpret
  current row absence, a read failure, local cancellation or a later message as
  permission to retry Unknown. A real committed mutation whose acknowledgement
  was lost must remain intact. Provider absence, checked reads and upgrade
  quiescence retain the head and stop draining. After confirmed commit preserve
  original channels, images, widget context, connector lane and DM-redrive rules.
  The zero-message reply nudge carries the same key/revision into its real
  admission. A displayed DM continuation is atomically retained ahead of raw
  later input by the same queue owner, with a fresh claim identity but the
  original Session binding and `Some([])` entries. It must not re-capture a new
  generation after archive/restore. Keep the existing one-redrive limit.
- CA-06: pause/settlement must compare the current worker token under the queue
  lock. Idle release remains atomic and the prior worker never clears a new
  worker. Expose a concise failure-kind diagnostic without envelope bodies and
  without a retry/log loop. Captured identity cannot adopt a restored/recreated
  Session generation. Claims are volatile within this manager lifetime; durable
  ingress, explicit unknown reconciliation, autonomous rearm, transport ACK,
  duplicate transport IDs, stop/recreate and full channel delivery remain open.

### Verification, release and provenance

Review disposition **CA-COMMIT-ORACLE-01 — FIX / TEST_FLAW** (2026-11-02),
recorded before repair. At `ae8774e38fd04ba07d571dd5c5ea42ddb600f977`, Runtime
`41429187307` job `125225966146` passed locked shipping compilation but failed
`test_agent_db_has_no_direct_commit_error_propagation` in the shared Settings
target (38 passed, one failed); downstream app/recovery/library groups did not
execute. Its whole-file literal prohibition matches the production command
adapter `ChannelInboundTransactionControl::commit`, whose sole caller explicitly
matches the result and forwards errors to `channel_inbound_transaction_failure`
with `commit_attempted = true`. That existing owner retains Unknown and performs
the required cleanup; changing production syntax to avoid the needle would not
repair the oracle. Preserve the prohibition for every unguarded site, exempt
only this precisely bounded, uniquely owned adapter after checking its sole
caller and cleanup path, and add mutation controls for an unguarded propagation
and missing error/rollback handling. Keep all actual SQLite acknowledgement,
rollback/retirement, queue and continuation assertions. A fresh independent
oracle review and final-H Actions must decide the repair; no CA execution pass
or environment failure is inferred from this source assessment.

CA-07 precise-exception continuation on
`e70a72574c108f8b7704eb45e5109875a24eddb9`: preserve the incoming repair and all
of its cleanup/Unknown mutation controls. Independent source assessment found
two further `TEST_FLAW` cases: deleting the entire text interval can hide an
unrelated exact-literal COMMIT owner after a harmless rewrite of the raw method,
and counting only `control.commit(db)` misses a second aliased or qualified
caller. Bind the exception to the unique parsed inherent raw method and remove
only that method, retaining siblings and intervening items. Require the one
argument-taking adapter call to be the actual returned handled match; refuse
extra or unresolved method references. Preserve the original literal assertion
and existing failure-owner checks. Add actual-source and bounded counterfactual
controls for these cases, malformed/ambiguous boundaries and discarded or
changed error handling. Reuse pinned Syn without changing dependencies or
production code. The complete shared target and real SQLite controls remain
required in exact-H Actions; a source review is not an execution pass.

CA-07 independent R1 disposition `FIX / TEST_FLAW`: a method-name forwarding
macro can hide an additional adapter call from a token-only direct-call count.
Refuse unresolved method-name forwarding and bare method identifiers before
granting the exception; add this counterfactual and preserve ordinary parsed
method declarations, zero-argument RAII commits, and all preceding controls.
Integration on `0ac0a8030dc74bbd0d9832fea684458b4e52da0e` preserves its fresh
Session append contract and all existing real SQLite oracles. No production
syntax rewrite or local executable check is authorized by this test repair.

CA-07 R2 dispositions `FIX / TEST_FLAW`: opaque macro arguments can disguise
an adapter call as `.commit()` by forwarding its separator and method; an
unqualified `commit!` invocation can also avoid the direct-call count. Keep
macro invocation inputs unresolved, including apparent zero-argument calls,
and reject the normalized `commit!` name. Distinguish literal macro definitions
so explicit zero-argument RAII controls remain valid; nested definition tokens
cannot lower an already opaque invocation context. Add both concrete cases to
the existing bounded and actual-source controls without removing any positive
or failure-owner assertion. This still does not claim general macro expansion
or semantic name resolution; unsupported recognized shapes revoke the exception.

CA-07 R3 dispositions `FIX / TEST_FLAW`: a substituted invocation separator
can make an apparent literal macro-definition call opaque after expansion.
Refuse that unresolved envelope, while retaining the explicit literal RAII
macro control. Recognize genuine macro path/bang boundaries; ordinary unary
negation, including after Rust control-flow keywords, must not revoke an
otherwise valid zero-argument RAII exception. Add both bounded and actual-source
counterfactuals and preserve every prior positive and failure-owner assertion.
The macro-definition exception is bounded to a complete single literal
receiver-to-zero-argument-commit rule; other definitions remain unresolved.

Use the actual Session/SQLite owner, retained queue and production-used Host
append/handoff control. Required controls include checked read/open/closed-owner
failure; real SQLite later-insert failure with complete rollback; ID conflict
with unchanged prepared IDs; full ordered commit and exactly one dispatch;
real commit followed by lost acknowledgement (rows remain once, head held,
no Runner and no later input overtaking); commit/rollback uncertainty and retired
connection; session key/generation/archive/deletion/provenance negatives; and
  the already-displayed DM redrive with zero new rows. Include a confirmed-append
  observer that re-enters the public Session facade after both locks release.
  Refuse both continuation paths after actual Session archive/restore while
  retaining the original committed rows; unchanged-generation controls still
  reach their existing consumers without reinserting displayed input.
  Preserve existing IH
concurrency oracles, model/checkpoint/custody, upgrade and completion contracts.
Fault seams may replace an acknowledgement after real mutation, not fabricate
rows or successful channel/provider acceptance. Run compilation/contracts only
in the existing Actions workflow, with final-H identity and frozen dependency
inputs. Exact-H runtime evidence and scoped source authority/coverage are still
required before declaring this unit verified; no global flag is advanced.

The allowed production paths are the existing app composition, Session
`production.rs` and `agent_db.rs`, and Transcript `background_wakes.rs`, plus
their existing contracts and workflow selectors. No new loop, queue service,
permission authority, database, upstream runtime or product module is added.
Rollback restores this bounded code change without deleting user rows; runtime
rollback loses volatile held claims and is not a recovery acceptance claim.

The shipping app's real SQLite oracles additionally require a dev-only direct
`rusqlite` edge and its app-lock package edge. Reuse the existing Host `0.39`
bundled configuration and already locked `0.39.0` identity; do not resolve new
versions or change production dependencies. These two build-input paths are
part of the complete reviewed batch. The malformed-JSON app fixture removes
only its two JSON-expression indexes before installing the damaged row, so the
oracle reaches the real checked read; production schema is unchanged.

Necessary fixed-U review: ingress-monitor-admission.ts
`0e83828d0d45af2d3e17d7abb7f8ea470c3f015c` (1–235),
ingress-monitor-tasks.ts `f8c4bdb6c6374a01c282a8cd4ffb24c5416c59ba` (1–80),
ingress-retry-policy.ts `92cb60517f91bd9fa134a44e1953aec6904c3409` (1–179),
ingress-monitor.ts `9d37fc03b0807d9f3b9f06e69662a7ac2d631a41`
(180–390, 535–743), all under `src/channels/message/`; and
`src/plugin-sdk/channel-ingress-runtime.ts`
`1b1817a655b06c0f5d2a3b7438ddf35dcfe82719` (1–140).
These are the recorded read extents, not a claim that partially inspected files
were fully read. The current OCRM inventory/ledger/RTM remain the only source
and responsibility accounts; full recursive ingress dependencies and actual
channel services/devices remain unaccepted.

## Shared test compilation repair — 2026-11-02

Decision **CA-SKILLS-TEST-COMPILE-01 — FIX / TEST_FLAW**, before editing.
At `e70a72574c108f8b7704eb45e5109875a24eddb9`, Runtime run `41432270535`,
job `125233494245`, the recovery step cannot compile two SQLite count
observations (`row.get::<_, usize>`) and the Session live-Skills fixture's
unimported `json!` macro. These same test expressions remain in incoming
`0ac0a8030dc74bbd0d9832fea684458b4e52da0e`. Read SQLite counts as its supported
signed `i64` representation and import the macro inside the affected test
module. Preserve exact zero/one counts, strict history refusal, retired-owner
and publication assertions, and all permission/lifecycle assertions. No
production behavior, transaction policy, test selection or dependency changes.
The earlier successful shipping, Settings/COMMIT and app results remain facts
for e70 only; recovery/lib and later skipped checks require new exact-H evidence.
Independent source review and the existing selected Actions gate are required.

Decision **CA-SKILLS-TEST-COMPILE-02 — FIX / TEST_FLAW**, before editing.
Incoming source review identified the same unsupported SQLite count type in
`session_gateway_contract::fresh_gateway_and_legacy_agent_messages_preserve_opaque_namespace_and_nonce_semantics`.
The integrator confirmed this new 0ac expression and its exact four-physical-row
assertion. Change only its read type to `i64`; retain the opaque-row, unique-ID,
committed-echo, duplicate nonce and no-publication assertions. This is a source
diagnosis of an incoming test defect, not a claim that its 0ac job has executed.
Freeze the five-path repair and its delta from the first four-path snapshot for
fresh focused review before publication.

Decision **CA-SKILLS-TEST-COMPILE-03 — FIX / TEST_FLAW**, before editing.
The first frozen review found that resolving `json!` also exposes the fixture's
unqualified `Vec::<Value>` expression. The integrator confirmed that neither
this module nor its parent imports `Value`; import both `json` and `Value` in
the test module. Keep both blocked/unblocked fixture branches and their exact
payloads. Preserve earlier snapshots and review a new complete five-path
snapshot with its delta; no new execution result is claimed from source review.

## Scoped receipt index repair — 2026-11-02

Decision **CA-APP-INDEX-01 — FIX / TEST_FLAW**, before editing the workflow.
At `f3b8105864a0e215e7918311b2d39cb789e350a4`, static reconciliation finds
41 exact shipping app selections but only 34 corresponding receipt labels.
The seven retained `OCRM-CHANNEL-APPEND-CUSTODY-v1` app cases are selected in
the actual loop; their names are missing from the generated receipt index.
Add their full names beside that contract without changing commands, selection,
assertions or declared acceptance. The omission does not mean zero execution;
the exact job log remains required, and authored receipt fields do not prove
that a pending or failed workflow ran these cases. Preserve earlier artifacts
and inspect the corrected final-H artifact independently.

## Borrowed-image wiring oracle repair — 2026-11-02

Decision **CA-IMAGE-WIRING-01 — FIX / TEST_FLAW**, before editing the oracle.
Exact H `f80b4baca00a999659a8ebfe8ebe92e7d27b6018`, Runtime `41434802065`,
job `125241848734`, fails the background-wake source check for the old literal
`collect_inbound_images(&envelopes)`. The actual retained-append continuation
now receives `envelopes` as a borrowed slice and calls
`collect_inbound_images(envelopes)`, projects every image's data and MIME type,
and passes the result to the existing `BackgroundRevivalContext`. The same
production path remains at the current `f3b8105864a0e215e7918311b2d39cb789e350a4`.
Update the obsolete call-spelling observation; retain the required image call,
context fields, actual two-image payload oracle and all delivery/failure/nonce
assertions. Do not add an unnecessary production borrow to satisfy stale text.
This is not an image-loss implementation finding or real-channel acceptance.

CA-IMAGE-WIRING-02 retains that actual projection requirement using the already
locked test-only parser and the uniquely named unconditional shipping worker.
A test fixture, literal, or cfg/cfg_attr function, local, expression or match arm
must not supply shipping evidence. Independent syntax controls accept slice
borrowing and whitespace variants and refuse missing/wrong input; no fixed
production spelling is required to select a mutation target. Decisions
WF-GEN-02/03 are FIX / TEST_FLAW before correcting this unpublished oracle.
Keep actual image payload, continuation and delivery assertions unchanged.
This remains a bounded syntactic supplement, not a dataflow or provider proof.

## Fresh Transcript commit — `SESSION-FRESH-APPEND-v1`

Original implementation base: `3761bde297377bca80ad7497d1664f6f3d32bc58`;
integration parent: `e70a72574c108f8b7704eb45e5109875a24eddb9`. Fixed U is
unchanged. This follow-up repairs Gateway prompt and legacy direct-agent
inbound allocation in the shared `tNu` / `tNuaM` namespace. Their former
read/allocate/append split could lose a different message to `INSERT OR IGNORE`.
The channel producer now uses the CA-v1 retained-attempt transaction above;
preserve that already-published custody and generation-bound Runner path.
All three producers serialize allocation through the same Session/SQLite
owner. Generic explicit-ID compatibility writes keep their existing semantics.

- FC-01: the existing Session filesystem/deletion boundary and `SandAgentDb`
  connection own one strict `BEGIN IMMEDIATE` read/plan/insert/commit operation.
  Recheck metadata-bound canonical session identity, archive/deletion state,
  selector and a supplied lifecycle revision in that transaction. An explicitly
  legacy no-admission caller remains compatible only while no admission record
  exists; a newly present record requires the caller to bind its selector.
- FC-02: the planner is crate-internal, synchronous and invoked exactly once.
  It receives the complete physical-row snapshot, including occupied IDs that
  presentation decoding would hide. Invalid JSON, nonobject rows or disagreement
  between SQL and embedded IDs refuse the operation. Preserve the existing pure
  `next_entry_id` algorithm and sequential batch allocation. Planners perform no
  I/O, Session/DB calls, listener dispatch, pending-turn mutation or provider work.
- FC-03: validate every proposed row's nonempty identity, kind and canonical
  provenance before accepting the batch. Every fresh insert must affect exactly
  one row; an ignored or conflicting insert rolls the whole batch back. A
  deliberately existing nonce/stable inbound identity is an explicit no-op,
  never success inferred from a fresh zero-row insertion. No recovery helper may
  replay the planner or convert closed/busy/read failure into an empty success.
- FC-04: return producer context only after COMMIT. A changed commit advances
  the existing DB write generation once. Carry the existing entries-upserted
  publication out of both connection and Session filesystem locks before
  synchronously notifying listeners. Failed or duplicate operations publish no
  fresh entries. Notifications remain observations, not transport acknowledgments
  or a new globally ordered event stream.
- FC-05: preserve the actual CA-v1 channel API and shipping handoff. Channel
  preparation is retained across confirmed rollback, Unknown blocks replay,
  and both continuations keep their original Session binding. Do not add a
  parallel String-result channel facade. Channel history retains its stricter
  owner-decoding refusal; Gateway/A2A physical snapshots may reserve opaque
  future IDs without treating them as visible messages. A recognized hidden
  control row supplies the positive shared-namespace control, while separate
  Gateway/A2A controls preserve opaque-ID compatibility.
- FC-06: Gateway recomputes nonce lookup, reply/fork placement, IDs and returned
  echo/recovery context from the transaction snapshot. Prepare attachment file
  facts and perform existing profile/awaiting/activity effects outside owner
  locks. The initial compatibility read may guide those metadata effects but
  cannot authorize a durable echo. This does not make those secondary effects
  atomic with the transcript or turn their failure into proof of rollback.
  Reuse the existing owner-row validity and hidden-receipt projection for
  semantic nonce/thread/recovery decisions; opaque physical rows still reserve
  their IDs but cannot impersonate a visible echo through a matching nonce.
- FC-07: direct-agent inbound recomputes stable-ID duplicate/conflict checks
  and legacy user ID selection in the same commit. Preserve stable
  `agent-inbound:{from}:{message}` IDs and pending reply claim/complete/release
  ordering. A provisional legacy ID used only in the returned local claim is
  not a reservation; replace it with the actual committed ID before settlement.
  No registry mutation runs inside the DB planner. A COMMIT failure is a typed
  unknown outcome: refuse provider continuation and retain an existing reply
  claim, rather than automatically releasing it for another side effect.

The shared crate-internal Session facade is
`commit_agent_transcript_entries(agent_id, session_key: Option<&str>,
expected_lifecycle_revision: Option<&str>, planner)`, with a pure `FnOnce`
planner returning `(producer_result, fresh_entries)`. It returns the DB owner's
typed error, including `TranscriptCommitUnknown`, without a retry. The
channel's distinct prepared-attempt API remains `append_channel_inbound_entries`.
Both operations belong to the same existing owner and connection; the channel
contract needs retained preparation and explicit known/unknown rollback outcomes,
whereas Gateway/A2A also need transactional nonce/no-op and context planning.
No duplicate channel facade, storage or schema owner is introduced.

Actions must exercise real Session/SQLite owners: fresh batch and frozen ID
compatibility, the three producers sharing IDs, same-nonce replay, partial
insert rollback, closed/busy/read/identity/generation/archive/delete refusals,
and a synchronous listener reentering Session after commit. Retain existing
Gateway attachment/reply/fork and A2A correlation/permission controls. The
shipping CA-v1 channel persister and its append-before-provider branch retain
their direct controls; an injected provider boundary is not real provider acceptance.

Allowed implementation paths are the existing Session `agent_db.rs`,
`agent_db_serde.rs`, `production.rs`, `gateway.rs`, Transcript `agent_to_agent_messaging.rs`, shipping
Host `main.rs`, their contract tests, the Session-private
`production_fresh_append_tests.rs` module and existing Actions selectors. Root
integrates shared workflow and OCRM traceability. No data migration, alternative
ID format, new queue, new journal or new runtime is introduced. Reverting the
change restores the collision risk without removing user state.

This contract does not prove the generation of an envelope queued before the
admission read, authenticated/pre-displayed ingress, durable transport ACK or
restart delivery. The channel CA-v1 head now retains failed/unknown append
custody; autonomous rearm/dispose fences, secondary acceptance effects, other
assistant/send-message namespaces and true post-COMMIT crash reconciliation
remain open. The superseded unpublished ID-less facade is omitted: its common
transaction/publication/admission assertions move to the production-used private
Session facade, while user-row validation remains covered by the actual CA-v1
batch validator and Gateway/A2A input contracts. No existing assertion is removed
from the integration parent. Preserve those failures and full acceptance gates.
The fixed-U channel dossier's recorded source reads are reused only for their
unchanged invariant; no historical read, scoped authority or global flag is
promoted by this local production repair. Exact-H evidence remains required.

### Fresh failure settlement

FC-03/FC-07 require an explicit cleanup decision while the existing connection
guard remains held. Pre-COMMIT refusal is an ordinary error only after rollback
acknowledgment and autocommit state establish clean rollback. Otherwise return
typed `TranscriptCommitUnknown` and retain A2A reply custody. Every failed COMMIT
acknowledgment stays unknown even if cleanup succeeds. Retire the exact connection
through the existing owner if it remains unsettled; no later read/write may
consume an uncommitted prefix as accepted transcript. Do not replay the planner.
Transaction destruction is an unwind backstop, not proof that rollback succeeded.
Keep compatibility writes and poison refusal semantics. Required controls perform
real inserts before command-level rollback failure, cover pre-COMMIT and COMMIT
failures separately, and bind actual A2A claim/no-publication/no-generation state.

### Duplicate attachment reads

FC-06 preserves the existing duplicate/coalesced-send resource behavior: when
an existing visible nonce is observed, authorize its returned context in the
Session transaction before reading attachment file facts. If that nonce is no
longer present, perform fresh preparation outside locks and recheck it in the
fresh append transaction. Physical opaque rows cannot authorize this fast path.
A controlled file-facts boundary in the real persister must prove zero reads
for a duplicate and real committed attachment facts for a fresh send; this does
not replace provider, filesystem-platform or performance-budget acceptance.

### Exact echo failure observation — FC-ECHO-OBSERVATION-01

Decision **FC-ECHO-OBSERVATION-01 — FIX / TEST_FLAW (capture)**, before the
focused evidence repair on `fdfdfb2d6a5635fc9c79da51b9fc4b09486b553a`. At
`364cca21ce04cbcf2d7ace69ce607fb4af95a1df`, Actions run `41446276172`, job
`125280031635`, the opaque-namespace Gateway test passed visible row count,
unique IDs and the durable/echo-count checks, then failed its complete echo
equality assertion. The log identifies neither producer order nor differing
field. The original failure stays unresolved; an unchanged green retry is not
an explanation, and no later assertions are counted as passed for that case.

The Gateway returns staged Values after the real SQLite commit. Rows are
written as JSON text and decoded by the existing Session reader. Its default
clock can supply fractional `f64` milliseconds. The locked serde_json 1.0.154
decoder without `float_roundtrip` rounds a decimal significand before scaling;
this permits a numeric round-trip difference, but the old log cannot prove
that it caused this particular failure. This is bounded dependency-contract
review, not a new upstream whole-file reading or an environment diagnosis.

- Keep both existing producer orders, the default-clock case, every full-entry
  equality, physical-ID/opaque-row, nonce/recovery and zero-publication check.
  Before adjudicating each echo, capture its synthetic fixture value, matching
  public row or absence, raw SQL text, producer order and both timestamp bits.
  Compare raw committed text with the serialized echo and its decoded value
  with the public reader, then retain the original full equality assertion.
- Add a real public Gateway/Session/SQLite control using the existing
  `composedAtMs` argument: integral 1000.0 and the representable value
  `1_793_000_000_000.0 + 6.0 / 4096.0`. Close and reopen the existing owners and
  require the same entire returned echo, timestamp bits, durable bytes and
  duplicate-nonce/no-new-echo behavior. No copied persistence implementation,
  rounded expected value, dropped field or approximate equality is permitted.
- Run the existing selected Gateway target in Actions at the new exact H.
  Correct serialized bytes with decoded-only timestamp drift establishes a
  numeric fidelity defect against this exact-echo oracle; absent/changed bytes
  or other fields require the observed owner-path repair. Preserve the failed
  subject and captured values before choosing a production change. Source
  arithmetic is a control design, not executable proof or acceptance.

Scope is this contract and the existing Gateway test. There is no production,
dependency, lockfile, workflow, schema or permission change in this diagnostic
increment. The current bounded credential fixture integration may share the
same frozen H, with its own assertions retained. Full source/ecosystem/shipping
acceptance remains open. Primary reviewed contracts are serde_json tag
`v1.0.154`, `src/de.rs` decimal and f64 conversion paths and `src/number.rs`
equality; the feature declaration in `Cargo.toml` blob
`059ac2957d93da5de73e6203fdbf720b4e417dd0` distinguishes fixed-precision
round trips from arbitrary-precision JSON preservation and documents its
float-parsing cost. No unobserved timing or performance claim follows.

### Exact committed JSON image — `SESSION-JSON-ROUNDTRIP-v1`

Decision `SESSION-JSON-01` — **FIX / production configuration gap**, at
`a514b70ab7901e9c8b4d87a8d29bf8e7d52be507`, with fixed U unchanged.
Runtime 41446276172 / job 125280031635 at
`364cca21ce04cbcf2d7ace69ce607fb4af95a1df` failed the valid Gateway
`entries.contains(&echo.entry)` assertion after the expected three rows and
unique physical IDs were present. That log does not identify the differing
field; preserve that uncertainty and the original assertion. Source review
shows a concrete candidate: Gateway returns the staged JSON image after COMMIT,
SQLite stores its serialization, and public reads decode it again. Gateway's
clock and supported `composedAtMs` carry fractional `f64` milliseconds; nested
valid JSON values can also contain floats. Host currently requests default
serde_json features, which promise only best-effort float parsing.

- JR-01: a successfully committed supported JSON entry must preserve its exact
  finite floating-point values through serialization and decoding. Returned
  producer context, committed publication, raw-row decode and Session read must
  agree. Preserve timestamp precision, nonce behavior, IDs, admission fencing,
  rollback/unknown outcomes and lock-free publication; no post-COMMIT readback
  becomes a new success condition.
- JR-02: enable serde_json's existing `float_roundtrip` production dependency
  feature in Host after a deterministic Actions baseline proves the gap. Do not
  add arbitrary-precision representation, change wire/storage schemas, normalize
  expected fixtures through the faulty parser, truncate timestamps, or replace
  equality with a tolerance. No crate version, lock identity or fixed U changes.
- JR-03: reuse the published FC-ECHO-OBSERVATION-01 Gateway/reopen control
  unchanged, including the default-clock namespace assertions. Supplement it
  through the existing Session fresh-append owner with nested positive/negative
  epoch-millisecond neighbors and small/large finite values built directly from
  fixed bit patterns; never derive expected values from JSON parsing. Require
  exact context/publication/read equality,
  one batch generation advance, unchanged raw rows/generation/publication on
  same-nonce replay and empty append, and retained physical IDs. Keep all prior
  attachment/opaque-row and rejection assertions.
- JR-04: first publish the deterministic controls with the existing production
  dependency and capture their exact-H failure plus Cargo's effective feature
  graph in GitHub Actions. Then publish the production feature change and run
  the same unchanged controls and affected shipping/contracts on that new H.
  A later pass of the original clock-dependent test alone does not resolve the
  earlier failure. Keep the baseline and fixed results separately; if the
  baseline does not expose a mismatch, reassess before claiming causality.

Ownership stays in Session/SQLite and the existing JSON dependency. Allowed
paths are Host Cargo.toml, `session_gateway_contract.rs`, the existing Session
`production_fresh_append_tests.rs`, this spec and the existing runtime workflow.
Both test groups are already selected. A small feature-graph evidence command
must retain ordinary failure propagation and existing selectors. Reviews cover
all these paths and adjacent producer/read/publication consumers.

Pinned dependency review: serde_json 1.0.154
[feature contract](https://github.com/serde-rs/json/blob/20572a6d066958542d79cdd6cf1a7cd9f0c8e1f8/Cargo.toml#L67-L74),
blob `059ac2957d93da5de73e6203fdbf720b4e417dd0`, and
[float parser](https://github.com/serde-rs/json/blob/20572a6d066958542d79cdd6cf1a7cd9f0c8e1f8/src/de.rs#L623-L672),
blob `d353b2d3ecb3146ef7f3ceb5a05a8d3a47144ea9`. The feature adds no dependency
and retains MIT/Apache-2.0 licensing; its documented float-parsing cost is about
twice the default path. Exact existing bytes are not rewritten, and rollback
restores the precision gap. Historical already-rounded in-memory values cannot
be recovered by this change. Final performance/soak, external protocol and
cross-process execution acceptance remain open; no global flags advance.

Integration decision `SESSION-JSON-INTEGRATION-01` reuses the published Gateway
instrument from `73cf0269c5ccbbd1dff946e0949824a1bbbeffd4`; the unpublished
overlapping Gateway instrument is archived rather than duplicated. Its real
owner reopen, raw-text, complete-equality and nonce controls remain intact.
Review decision `JR-03-NONCE-RAW-01 — FIX / TEST_FLAW` additionally requires
ordered physical-row and write-generation snapshots immediately around that
same integral/fractional nonce replay: decoded equality alone cannot detect a
raw-byte rewrite. The additional Session corpus supplies nested-value and signed
zero coverage plus one-generation/empty-append invariants. Every published
Gateway assertion remains unchanged. No production precision repair is claimed
by this baseline increment.


### Observed parser repair — `SESSION-JSON-02`

Decision **FIX / IMPL_BUG** before the production feature edit, on
`04301c80866768e9169ceda274f48f9758232792`. The prior deterministic
[Actions run 41450496585](https://github.com/bhrumom/fabushi-desktop/actions/runs/41450496585),
Host job `125296671159`, attempt 1, at
`73cf0269c5ccbbd1dff946e0949824a1bbbeffd4` failed only the
`gateway_echoes_preserve_fractional_timestamps_through_durable_reopen` case
in its Gateway target (25 passed, one failed). Its initial and reopened raw SQL
text exactly matched the returned echo, containing
`timestampMs` and `sentWhileOfflineAtMs` of `1793000000000.0015`; decoded and
public values were `1793000000000.0017`. The Rust-emitted timestamp bits changed
from `4790166932267466758` to `4790166932267466759`. Integral controls retained
their exact values, both existing namespace tests passed, and SQL/decode errors
were absent. The retained complete Host log SHA-256 is
`10759adc2c6183fa8393581b0adb1502388081d9aec0d1db0d480d0e5dbff54f`.
This proves a production decoding mismatch in that deterministic case; it does
not retrospectively identify the unlogged differing field at `364cca21`.

Enable `float_roundtrip` on Host's ordinary serde_json dependency, so the
shipping app and ordinary consumers use the same faithful finite-float parser.
The existing version/lock identities and raw stored bytes stay fixed. Keep
every Gateway, nested-value, nonce, generation and no-write assertion. This
change introduces no new parser, representation, store or schema. The primary
dependency contract, license, parsing-cost qualification and open acceptance
limits above still apply.

The expanded pre-feature baseline is separately published at
`04301c80866768e9169ceda274f48f9758232792`, runtime run
[41453150181](https://github.com/bhrumom/fabushi-desktop/actions/runs/41453150181).
JR-04 requires its actual effective-feature graph and deterministic nested-value
outcome to be retained before publication of this repair, followed by unchanged
controls and affected shipping/contracts on the resulting exact H. Source
review or the earlier integral/clock-dependent passes cannot substitute for
that execution, and no performance or full-migration acceptance is implied.
## Live inbound readiness — `OCRM-CHANNEL-INBOUND-REARM-v1`

Implementation base: `0ac0a8030dc74bbd0d9832fea684458b4e52da0e`.
Fixed U remains `1440bb1771372334bea23d7642bb52deb7db1845`. The CA-v1 queue
already retains a paused or confirmed-not-written head, but provider-setting
and successful upgrade-resume changes do not wake it. A signal arriving while
the worker is about to pause also needs to survive that exact transition.
Extend the existing `BackgroundWakes` claim; do not add a queue, scheduler,
permission owner or another recovery registry.

- CR-01: supported signals are a committed Settings notification for
  `inferenceProvider`, `inference_provider` or `router`, successful existing
  `TranscriptManager::resume_after_recreate`, and newly enqueued channel input.
  These request a fresh gate check; they do not certify credentials, network
  health or permission. Failed Settings CAS/persistence and unrelated fields
  must not request readiness. A retained head precedes all later raw input.
- CR-02: claim the real existing head and worker token under its mutex before
  spawning a thread. One common shipping launch adapter serves ingress and
  readiness. A launch failure releases only its exact current Active token,
  preserving envelopes and prepared attempt. A later supported signal can
  reacquire it; no timer or no-signal eventual recovery is promised.
- CR-03: a signal observed while a head is Active sets one coalesced latch.
  Pausing or recording NotWritten atomically either consumes the latch and
  returns the latest claim-owned Work with a new token, or releases the old
  token and revival ownership together. A signal after release acquires one
  successor. Retry carries the just-recorded attempt, original IDs, timestamp
  and Session generation; it never reuses an earlier unprepared snapshot.
  If the gate stays closed without a new signal, stop after consuming the latch.
- CR-04: typed transition results distinguish Committed dispatch permission,
  Retry(work), Held and Stale. Unknown clears readiness and remains held;
  neither new input, Settings nor resume may retry its append or dispatch it.
  Committed continues the existing single Runner handoff and generation-bound
  displayed redrive. Readiness must never become Runner permission by boolean
  overloading, and no queue mutex may cover Session I/O or Runner callbacks.
- CR-05: disposal closes this `BackgroundWakes` instance permanently before
  downstream cleanup callbacks. Clear existing pending/held/revival collections
  while retaining the closed fact; do not reset it to an open Default. Late
  callbacks, tickets and old-token cleanup cannot reopen or populate executable
  inbound work. A new manager owns a separate fresh object. This does not join
  workers or retract an append/dispatch that already crossed its last check.
- CR-06: retain exactly one Settings subscription in shipping composition;
  capture a Weak Gateway and reread the actual provider/upgrade gates in the
  worker. Unsubscription cannot retract a snapshotted callback, so the queue's
  closed check under its mutex is authoritative. Emit resume readiness directly
  after manager resume succeeds, before later fallible restoration/recovery.
  Launch failure diagnostics cannot reverse an already committed Settings write.
- CR-07: preserve CA/IH/FC assertions and the original Session generation,
  connector lane/permission policy, Unknown isolation, head/tail ordering,
  displayed redrive and independent agents/wake families. Do not add an active
  user preflight or change provider selection/Settings persistence ownership.
- CR-08: use real BackgroundWakes, Session/SQLite, SettingsService and manager
  entry points with deterministic I/O or launch boundaries. Cover both sides of
  pause, a signal during real confirmed rollback, Unknown plus later signals,
  stale launch failure, actual subscription delivery and failed writes, actual
  resume, disposal versus snapshotted callback and a reserved ticket, and fresh
  successful append/handoff. Downstream counters prove ordering only, not real
  provider execution or external channel delivery. Execute only in Actions.

### Shared Rust interface and composition

The existing owner adds `InboundAppendAction<T>` with `Committed`,
`Retry(InboundAppendWork<T>)`, `Held` and `Stale`. The current
`pause_inbound_append` and `record_inbound_append_outcome` return this type;
production callers must handle it explicitly. Existing negative and positive
assertions are adapted to the same state meanings, not removed.

`enqueue_inbound_and_claim(agent_id, envelope)` returns
`Result<Option<InboundAppendWork<T>>, String>` and rejects a closed object.
`request_inbound_readiness()` returns current `(agent_id, Work)` launch tickets
and coalesces Active heads. `release_inbound_launch(agent_id, work)` retires only
that exact Active ticket after a spawn error. `close()` permanently closes and
clears the existing object; `is_closed()` observes that fact. All are operations
of the existing owner, serialized by its already-held mutex. Readiness inspects
retained claims as well as raw pending input. Reuse the existing atomic
take/idle handoff between successful batches; do not clear a successor's bit.

Allowed implementation paths are `background_wakes.rs`, `transcript_manager.rs`,
their existing production contracts, shipping Host `main.rs`, this existing
specification, a private shipping-app test module at
`source/host/app/src/tests/channel_inbound_readiness_tests.rs`, and existing
Actions/ledger/RTM references. The new module exercises the same shipping
launch, Settings-subscription, readiness gate, append and resume adapters with
real Session/SQLite and TranscriptManager owners; it adds no runtime module.
No Settings storage,
provider policy, wire schema, lockfile or new runtime module changes are needed.
Root integrates composition and shared selectors serially. Revert this bounded
change without deleting durable transcript; volatile ingress and unknown-state
reconciliation are not migration or restart acceptance claims.

The fixed-U ingress monitor/admission/tasks and drain/state source identities
already recorded in the bounded channel dossier support request coalescing,
final owner release and Unknown custody. Reuse only those read extents. The
upstream recovery sweep, provider-health events, joined shutdown, durable
ingress/reconciliation and full delivery remain open. Final-H source authority,
scoped contracts and full G01–G12/AC01–AC24 remain required; no global flag moves.

## Delivery-failure idle handoff — `OCRM-CHANNEL-FAILURE-IDLE-HANDOFF-v1`

Base: `350b87058a4944a58d04b75195fdcf8aa1d042d9`; fixed U is unchanged.
Decision **CFH-01 — FIX / IMPL_BUG**, established by production source review:
`queue_channel_delivery_failure` enqueues and begins revival under one existing
`BackgroundWakes` mutex, but `run_channel_failure_revival_worker` removes an
empty batch before separately clearing revival. A failure enqueued between
those locks observes the old worker, starts none, and remains stranded after
that worker exits. This is independent of the original-turn receipt contract;
its pending exact-H checks are not treated as passing evidence.

The flow remains SendMessage/Host delivery failure → existing failure marker
and `BackgroundWakes` → existing connector follow-up Runner → exact marker
settlement. Reuse IH-01..04's atomic handoff and production drain control.

- CFH-02: `take_pending_channel_failures_or_end_revival(&mut self, agent_id)`
  returns one ordered nonempty batch while retaining that agent's failure
  revival bit, or removes the bit and returns `None` under the same queue mutex
  as enqueue. A stored empty vector is idle. Share the existing take-or-end
  operation with the inbound wrapper; do not change other agents or wake
  families. A closed queue returns no executable work.
- CFH-03: both shipping channel workers use the existing common drain control.
  `None` returns without later cleanup: a successor may already own revival.
  A nonempty failure batch keeps all current Session/provenance checks, prompt,
  provider/quiesce gates, connector lane and exact durable-marker decisions.
  Existing pause/error requeue preserves the original batch before newer input
  and performs its existing cleanup once; no immediate replay or timer is added.
  No Session I/O, provider, callback or telemetry runs while the queue is locked.
- CFH-04: deterministic real-owner controls cover enqueue before idle, idle
  before enqueue, exactly one successor, claim once, ordered batches, stored
  empty vectors, closed owner and cross-agent/family isolation. The actual
  production drain control must be paused after atomic idle release; a new
  failure claims a successor before the predecessor returns, and its ownership
  must survive that return. Preserve the nonempty pause/requeue control and
  supplement with a bounded shipping-worker wiring check. Existing actual
  Host-lane failure/marker tests remain selected; callbacks are not real external
  channel or provider compatibility evidence.
- CFH-05: keep delivery failure distinct from a remote no-effect receipt. No
  transport retry, permission change, new queue, scheduler, state store or
  runtime is introduced. Durable marker errors, unknown remote outcomes,
  failure-readiness rearm, joined disposal/recreate, timeline-event handoff and
  full restart recovery remain required subsequent work. No on-disk migration
  is needed; rollback restores the race without deleting user state.

Allowed paths are this existing specification, Transcript `background_wakes.rs`,
its existing production contract target, shipping Host `main.rs` and its private
tests, and the existing Actions selectors/receipt index. Root integrates shared
composition and workflow; no schema, lockfile, source inventory or acceptance
flag changes. Run affected compilation, real-owner concurrency, shipping drain
and retained channel/Session contracts only in Actions on the final exact H.
The fixed-U ingress-monitor source identities and bounded read extents already
recorded in the channel dossier provide the final-repump/active-owner invariant;
this native race repair does not promote historical reading or scoped authority.

## Failure-notice live custody — `OCRM-CHANNEL-FAILURE-LIVE-CUSTODY-v1`

Authoring base: `09df841142a378436c490c38b375ac842ac3f419`; fixed U
remains `1440bb1771372334bea23d7642bb52deb7db1845`. Decision **CFL-01 —
FIX / IMPL_BUG**, before implementation: the existing failure worker puts both
gate-paused batches and batches that already invoked Runner back into one raw
queue. Settings and successful upgrade resume wake only inbound work. Simply
adding failure readiness would also replay possibly executed work. Close can
clear the queue while its old static enqueue/requeue callbacks populate it again.
Keep the existing failure producer, BackgroundWakes mutex, common drain,
Session/provenance checks, Runner, trays and precise pending-marker owner.

Integration base: `3c4bf6963cb30ecf01eb34b905315074886e1972`, a direct
descendant of the authoring base, adds the existing-owner CWR-v1 exact pending
marker recovery contract. Preserve that complete incoming change. Decision
**CFL-CWR-01 — FIX / IMPL_BUG**, before integration: raw-tail-only recovery
deduplication and token-free worker launch cannot safely compose with a retained
live head. Adapt those entry points to the same claim/token owner, retaining
checked whole-image reads, exact marker metadata/currentness, age retention,
unpersisted live-failure fallback and every valid CWR assertion.

- **CFL-01 — one retained head.** Keep later input in the existing
  `pending_channel_failures`; add only a private head claim in BackgroundWakes,
  containing the original ordered failures, opaque claim ID, per-acquisition
  worker token, phase and one readiness latch. Phases are
  `ActiveBeforeDispatch`, `PausedBeforeDispatch`, `EnteredRunner` and
  `RetainedAfterDispatch`. Claims are live custody in this manager lifetime,
  not another queue service, durable ledger or scheduler.
- **CFL-02 — exact acquisition.** Enqueue and readiness reserve the real head
  and token under the existing mutex before thread creation. An active
  pre-dispatch head coalesces a signal; a paused head gets a fresh token. A
  failed launch releases only its matching pre-dispatch ticket and retains the
  batch for a later supported signal. It does not recursively retry. Idle
  release stays atomic; no returning predecessor clears a successor's revival.
- **CFL-03 — atomic pause.** A pre-dispatch pause records the latest retained
  failure list after existing provenance filtering. Under one lock, either
  consume the latch and return `Retry` with that same head and a fresh token,
  or mark it paused and release revival. A closed gate without a new signal
  stops after this one recheck. Later input cannot overtake the retained head.
- **CFL-04 — one follow-up admission.** Immediately before the actual background
  Runner invocation, compare claim/token/phase and the closed flag under the
  queue lock, store the filtered batch and enter `EnteredRunner`. A mismatch
  makes zero Runner calls. No Session query, marker I/O, provider, callback or
  thread operation runs while holding this lock. Close cannot retract work
  that already crossed this last admission boundary.
- **CFL-05 — no replay inference.** After invoking Runner, all non-successful
  notification outcomes remain `RetainedAfterDispatch`: errors, cancellation,
  quiescence and zero sent messages. Do not parse String errors or infer no
  effects from them. Settings, resume and new failures cannot re-enter this
  head, clear it, combine it with later input or erase its marker. This batch
  introduces no automatic release of retained post-dispatch custody.
- **CFL-06 — preserve precise completion.** Preserve the existing notification
  completion condition: not aborted, not quiesced, and sent-message count above
  zero. Only that branch clears the same exact durable markers and emits the
  existing roster update before finishing the matching head. Existing definite
  no-work decisions (missing/group Session or completely stale provenance)
  may finish a pre-dispatch head with their existing marker handling. Finished
  means this queue head may drain onward; it does not prove external delivery
  or that all additional tool effects physically stopped.
- **CFL-07 — existing readiness signals.** Extend the current single Settings
  subscription and successful `resume_channel_inbound_after_recreate` callback
  to request both inbound and failure readiness. Keep the same committed keys
  `inferenceProvider`, `inference_provider` and `router`, weak shipping Gateway
  capture, and resume-before-later-fallible-restoration ordering. Failed writes,
  failed CAS/resume and unrelated keys request no readiness. Each worker
  re-reads its real provider, upgrade and Session/provenance gates. Preserve
  failure policy; do not adopt inbound's additional remote-room exclusion.
- **CFL-08 — permanent close.** The existing close operation also clears these
  claims/latches. Instance enqueue, take, pause, admission, finish, late launch
  cleanup and snapshotted readiness reject a closed owner or stale token and
  cannot populate executable work. A fresh manager remains a separate object.
  This does not join old workers or grant cancellation of admitted effects.
  Integration decision **CFL-CLOSE-MARKER-01 — FIX / IMPL_BUG**, before the
  correction: an already admitted transport can return failure after this
  manager closes. Reject its executable enqueue, but preserve the CWR-v1
  fresh-failure marker publication before that refusal so the next manager
  can recover the actual delivery failure. An early closed check before marker
  publication would silently discard durable custody. Keep marker I/O outside
  the queue lock and prove the late failure creates no old-owner follow-up.
- **CFL-09 — shared implementation and observations.** Reuse
  `drain_channel_revival_batches<B>` and existing owner constructors. Production
  must use the token transitions, with no generic stop callback that clears a
  possibly newer revival bit. Expose only read-only claim phase/failure views
  needed by existing status/tests; raw mutable state is not dispatch authority.
  The legacy raw failure take refuses while a live head exists, without
  changing that head, its tail or revival; otherwise retain its CFH behavior.
  Keep all existing inbound, CFH, Session and Host-lane behavior assertions.
- **CFL-10 — exact recovery joins live custody.** Keep CWR-v1 marker decoding
  and equality in shipping composition. Recovery deduplicates against both the
  retained head and raw tail; an exact duplicate signals only that agent's
  existing pre-dispatch head, while entered/retained work remains non-replayable.
  Different marker metadata remains distinct ordered input. Preserve the
  checked store-then-wakes lock order, reserve the ticket inside that admission,
  and launch after both guards are released. Each processing pass applies the
  existing checked marker-currentness filter before Session/provenance gates;
  read refusal pauses the original head, and fully stale input finishes without
  Runner. Repeated recovery never re-inserts the same marker behind a held head.

### Frozen shared interface

The existing BackgroundWakes owner adds `ChannelFailureWork<T>` with
`claim_id: String`, `worker_token: String` and `failures: Vec<T>`,
`ChannelFailureState` with the four phases above, and
`ChannelFailureAction<T>` with `Finished`, `Retry(ChannelFailureWork<T>)`,
`Held` and `Stale`. `Finished` permits the existing drain to continue; it is
not a delivery receipt. Use these instance methods:

- `enqueue_channel_failure_and_claim(&mut self, agent_id: &str, failure: T)`
  returns `Result<Option<ChannelFailureWork<T>>, String>`.
- `request_channel_failure_readiness(&mut self)` returns
  `Vec<(String, ChannelFailureWork<T>)>`.
- `request_channel_failure_readiness_for_agent(&mut self, agent_id: &str)`
  returns `Option<ChannelFailureWork<T>>`, checks permanent close, and requests
  only that agent's existing head/tail. Composition uses this for a duplicate
  exact marker already retained by the owner.
- `owns_channel_failure_work(&self, agent_id, &work)` checks a live matching
  active or entered ticket; phase-specific transitions check their own phase.
- `release_channel_failure_launch(&mut self, agent_id, &work)` returns bool
  and only retires a matching pre-dispatch ticket.
- `pause_channel_failure_before_dispatch(&mut self, agent_id, &work,
  retained_failures: Vec<T>)` returns `ChannelFailureAction<T>`.
- `begin_channel_failure_followup(&mut self, agent_id, &work,
  retained_failures: Vec<T>)` returns bool and only admits pre-dispatch work.
- `retain_channel_failure_after_dispatch(&mut self, agent_id, &work)` returns
  bool and only retains an entered ticket.
- `finish_channel_failure_work(&mut self, agent_id, &work)` returns bool for a
  matching pre-dispatch or entered ticket; callers own the definite completion
  decision. Preserve revival for the next atomic take/idle operation.
- `take_channel_failure_work_or_end_revival(&mut self, agent_id)` returns
  `Option<ChannelFailureWork<T>>`, never acquires an entered/retained head and
  never clears another active ticket.
- `channel_failure_state(&self, agent_id)` returns the optional phase;
  `retained_channel_failures(&self, agent_id)` returns `Option<&[T]>`.

### Actions oracles, scope and limits

Use the real BackgroundWakes owner and production-used launch, readiness,
pause/admission/outcome adapters with actual Settings, Session/SQLite,
TranscriptManager and pending-marker fixtures. Only the final provider/follow-up
I/O may be controlled. A counted callback proves one invocation and control
ordering, not real provider or channel acceptance. Required cases cover signals
on both sides of pause, one coalesced recheck without spin, latest filtered head
and tail ordering, successful resume and failed Settings/CAS/unrelated changes,
entered error/cancellation/zero-message retention under every later signal,
successful completion followed by a new head, failed launch versus successor,
close during a gate and late cleanup, fresh-owner independence and wake-family
isolation. Retain the real Host-lane append/operation/marker/no-transport-retry
assertions. Migrate the old worker-shape AST supplement to the new token
delegation, preserving the negative control for cleanup after idle release and
all actual CFH concurrency controls; changed spelling is not a reason to drop
the oracle. Add app selectors and matching receipt labels in the existing
workflow; only exact-H Actions can establish execution evidence.
The CFH app controls now acquire tickets before launch: a second arrival stays
in the tail instead of joining an already reserved head. Preserve exact ordered
payloads and one acquisition per head. In the nonempty pause control that second
arrival is one supported signal, so require exactly one additional gate check,
then a held original head ahead of the tail; do not weaken its no-spin assertion.

Also select the complete `background_wakes_production_contract` target in the
existing Linux pending-failure recovery step/cache. Keep every CWR command and
positive control, and require the ten live-custody owner controls plus both
token-wiring mutation controls exactly once. Bind the CFL spec and test inputs
in that same exact-H receipt. This provides portable queue/state-machine
evidence and explicitly identified source-wiring supplements. Also reuse the
existing Linux shipping-App check and linked test harness to execute all
fourteen failure-readiness, two handoff and eighteen Host-lane/CWR cases by
exact name, retaining their real Settings/Session/marker and controlled final
I/O boundaries. Keep its existing executable identity, exact-list and positive
completion checks. Linux execution does not replace the macOS shipping gate,
real-channel acceptance or any other supported platform's acceptance.

Keep the actual shipping Session/provenance/provider/quiesce/marker/telemetry
decisions in one private `process_channel_failure_work` adapter taking existing
runner deps, roster, wakes, agent and work, plus a final
`FnOnce(RoutedProvider, &[Value]) -> Result<RevivalExecution, String>` delegate.
The worker supplies its real prompt/widget/Runner call; app tests supply only
that final controlled I/O. Both use the same admission/outcome handoff helper.
No test may copy the gate or outcome state machine as its own oracle.

Production changes are limited to existing `background_wakes.rs` and shipping
Host `main.rs`. Tests may extend `background_wakes_production_contract.rs`,
existing private channel readiness/Host-lane fixtures and a private
`channel_failure_readiness_tests.rs`; shared fixture exposure is test-only.
Root integrates this existing spec, composition, workflow and narrow existing
OCRM responsibility references serially. Add no dependency, lock, persistent
schema, permission authority, transport registration or runtime module.

If connector transport cannot carry the large reviewed Host/ledger blobs,
reuse the repository's temporary `ocrm-stage-reviewed-objects.yml` delivery
carrier. Bind its exact parent, fixed changed-path allowlist, before/after blob
identities, patch digest and candidate tree; it may upload only those immutable
Git objects, without committing source, changing refs or claiming validation.
The formal implementation commit removes the carrier and must reproduce the
reviewed candidate tree. Product verification runs against that formal H.

Do not call `PendingWakeRearm::rearm_pending_wakes` as a readiness mechanism.
Incoming CWR-v1 now preserves exact marker identity through bounded restart
enqueue and retains it until the existing settlement path; keep that production
implementation and its tests while integrating the live claims. These claims do
not survive process loss. Original-turn reconciliation, post-dispatch custody
across restart, unknown remote effects, failed marker publication, real services/platforms and
all final G01–G12/AC01–AC24 gates remain open. Background terminal custody is not
upgraded by this change. Rollback loses this live retention fence without
deleting durable user data. Reuse only the existing fixed-U channel dossier's
recorded coalescing/final-repump and no-blind-replay review extents; no historical
full-read fact, scoped verification or global completion flag is promoted.
