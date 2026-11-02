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
