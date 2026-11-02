# Channel SendMessage through the existing Host delivery owner

Contract: `OCRM-CHANNEL-HOST-LANE-DELIVERY-v1`  
Status: implementation and real-owner oracles authored; source review and exact-H execution pending  
Original diagnosed base: `e70a72574c108f8b7704eb45e5109875a24eddb9`  
Integration base: `7a0254d4133080877cbfb2d9d4831acb231e673f`  
Fixed U: `1440bb1771372334bea23d7642bb52deb7db1845`

## Problem and ownership

The shipping `ProductionSendMessageSink` persists the generated message through
Session/Transcript, copies the entry ID to `deliveryOperationId`, then calls
`call_host_lane("deliverToChannel", ...)`. The sole Host-lane receiver invokes
`dispatch_gateway_call`, which does not route that method to the delivery
handler. The handler exists only in `UnifiedGatewayApi::call`. Consequently this
Runner path can persist a channel message and then log UnknownMethod without
reaching the existing delivery/failure owner.

Decision **CH-ROUTE-01 — FIX / IMPL_BUG**, before implementation. Route this exact
internal method to the existing concrete handler. Session remains transcript
truth, `HostChannelDelivery`/`ChannelDeliveryRouter` remain transport registration
and delivery, and existing trays, pending-wake storage and BackgroundWakes remain
failure owners. Add no scheduler, delivery database, adapter catalog or loop.

## Requirements and implementation boundary

- CH-01: one actual Sink call produces one existing transcript append. Preserve
  its entry ID as the failure/delivery operation identity and its admitted
  Session key. The real router preserves its text chunk suffix (`:chunk-1`).
  Delivery processing must not append the original message a second time.
- CH-02: extract the existing handler into one private concrete
  `dispatch_channel_delivery(&ChannelDeliveryDispatchDeps, Value)` function.
  The projection contains only clones of existing delivery, routed Runner,
  roster and wake handles; it has no mutable business state. Both external
  Gateway and internal `dispatch_gateway_call` use that same function. The
  production main receive arm supplies the actual projection.
- CH-03: match only `deliverToChannel` before the existing generic fallback.
  Do not call the full Gateway API from the Host lane: its general fallback
  sends to that same queue and waits. Preserve all unrelated dispatch behavior.
  Internal Runner sends do not manufacture desktop-contact activity; external
  Gateway calls retain their existing contact accounting.
- CH-04: preserve argument validation, outbound projection and Session checks.
  A message without an outbound projection returns `queued: false` and starts
  no delivery worker. Successful thread creation returns `queued: true` before
  transport completes. That response is scheduling acknowledgement, never a
  delivery receipt, Runner terminal or permission to retire pending custody.
- CH-05: transport and failure revival remain on their existing worker threads.
  A held transport cannot block the main Host lane. Preserve actual failure
  humanization, the `Message not delivered` tray, original-ID durable marker,
  and actual queue/revival handling. Missing provider or upgrade quiescence
  retains the existing failure queue and marker, with no additional transcript
  append or fake follow-up run.
- CH-06: do not retry a failed or timed-out transport. The current String error
  cannot prove whether an external effect occurred. An error after an effect
  must produce only the existing failure path, without a second transport call.
  Unsupported/missing real provider registration remains fail-closed.

## Source and changed paths

The integrator owns `source/host/app/src/main.rs`, its private dependency
projection/dispatch/main wiring, the existing Actions workflow, this contract
and the narrow existing channel dossier update. A test worker may own only
`source/host/app/src/tests/channel_host_lane_tests.rs`, included beneath the
existing terminal `#[cfg(test)] mod tests`; it must not edit main concurrently.
No schema, dependency, lockfile or OCRM ledger change is required.

The integration base retains incoming `0ac0a8030dc74bbd0d9832fea684458b4e52da0e`
and its atomic Gateway/A2A fresh-append owner. The shared test-compilation repair
is a separate five-file commit. This batch preserves every incoming selector,
assertion, prepared channel attempt and generation-bound continuation. Its new
tests use the actual Session writer exposed by the integrated source.
The subsequent MCP account-listing retirement commit is also retained. Its
renderer selector additions merge with these app selectors in the existing
workflow; it changes none of this batch's Rust handler or fixture source.

Read the target Sink, lane sender/receiver, both dispatchers, Session append,
outbound projection, router, failure queue/worker and durable marker consumers.
The fixed-U `src/infra/outbound/message.ts`, blob
`5032acef657250cbce6b85435f5215237825163b`, was read in full (lines 1–567) for
the direct/Gateway boundary and explicit delivery-outcome distinction. Its
idempotent Gateway timeout reattachment does not authorize retrying this
target's String-error transport. That file's imported durable-runtime contracts,
all other channels and real providers remain required wider review; this bounded
routing repair does not claim their source or compatibility closure. Historical
dossier reads retain their original identities.

## Falsifiable verification

Use real Session/SQLite, TranscriptManager canonical runtime/Ack/registry/wakes,
actual ProductionSendMessageSink, an actual HostLaneRequest mpsc queue, the
production dispatcher/handler, HostChannelDelivery and ChannelDeliveryRouter.
Only the external transport port is controlled. Construct the actual non-Send
UnifiedAppHost inside the lane thread. Idle dependencies use real explicit-root
constructors, not the full production startup factory or fake dispatch/failure
callbacks. The lane fixture only mirrors the existing receive/reply arm.

Required controls:

1. Held successful transport: exact append/operation/address/session identity,
   one delivery, no extra append or failure, `queued: true`, and a successful
   unrelated `host.platform` request through the same lane before release.
2. Unregistered real router with actual missing inference settings: completed
   delivery and failure workers, actual tray, fresh reopened durable marker,
   retained queue, ended revival and no follow-up admission.
3. Registered failing transport with configured provider and actual runtime
   upgrade quiescence: the distinct quiescence branch retains the same custody.
4. Recorded external effect followed by an error: one transport attempt only,
   original entry/marker retained and no fabricated delivered result.
5. Projection no-op: real append and `queued: false`, no worker/transport/tray/
   marker, and unrelated/unknown fallback behavior preserved.
6. Internal sends preserve the actual Session runtime's prior desktop-focus
   timestamp, proving they did not traverse the external contact boundary.

For these concurrency oracles only, an optional per-fixture `cfg(test)` sender
may report DeliveryFinished after the actual delivery closure returns from all
failure enqueue work, and FailureRevivalFinished after the actual revival
function returns and ends revival. It selects no behavior and is absent from
shipping builds. Failure tests wait for both events, not the early tray event;
timeouts are failures. No destructor emits successful completion on panic.

Freeze the changed scope for independent source/permission/lifetime/oracle
review. Run locked shipping compilation and these exact app selectors, retained
channel/Session/transcript contracts and source authority in the existing
GitHub Actions workflow only. Keep every incoming CA/Skills assertion/selector.
Do not count a controlled transport as real channel acceptance.

Select and index these exact app cases under
`tests::channel_host_lane_tests::`:

- `channel_send_host_lane_reaches_existing_delivery_owner_once`
- `channel_send_host_lane_failure_reaches_tray_and_pending_wake`
- `channel_send_host_lane_unregistered_provider_uses_existing_failure_owner`
- `channel_send_host_lane_returns_before_transport_and_preserves_fallback`
- `channel_send_host_lane_preserves_projection_noop_and_no_retry`

Evidence repair **CH-EVIDENCE-01 — FIX / TEST_FLAW**, before editing: the
inherited workflow selects the real AgentDb channel-append library group but
omits its exact name from `integration_recovery_lib_filters` in the receipt.
Restore that one index member when editing this batch's selections. Execution
still requires the actual log; an index omission is not itself a missed test.
The incoming 0ac workflow already restored this member. Integration adopts that
exact correction and preserves all 22 library-filter index members; this batch
adds only its five app selectors and bounded receipt fields.

Review decision **CH-ORACLE-01 — FIX / TEST_FLAW**, before editing: the initial
fixture passes `auth.service()` by reference, while the existing BoxStore
credential field owns an `Arc<dyn FabushiCoreCredentialProvider>`. Clone the
existing Auth service Arc exactly as the production composition does. This
changes only construction of the idle real owner; it changes no credential,
permission, delivery or assertion behavior. Preserve the first frozen snapshot
and require fresh focused source review of the corrected full scope.

## Release, rollback and remaining limits

No persistent format changes. Rollback removes the route/projection and its
oracles together, restoring the known missing internal route. Thread-spawn or
lane-reply loss still lacks durable dispatch reconciliation. Session key checks
do not add a new full-generation transport fence. Transport registration,
credentials, protocol-specific acknowledgements, original-turn/restart recovery,
autonomous failure rearm, all channels/platforms and full G01–G12/AC01–AC24 remain
open. Capability metadata alone does not prove a concrete provider is registered.

## Exact pending-failure recovery — CWR-v1

Authoring base: `0533ca72870a2794c2db32959b9d01427abf325a`; fixed U is
unchanged. Decision **CWR-01 — FIX / IMPL_BUG**, before implementation:
`UnifiedGatewayApi::rearm_channel_delivery_failure` sends a persisted marker
through the fresh-failure producer, changing its timestamp and optional
metadata. `PendingWakeRearm` then clears the observed ChannelDelivery marker
on enqueue success. With missing inference configuration or upgrade pause,
the worker has only requeued the failure; neither enqueue nor requeue proves
terminal settlement. A settled/replaced snapshot can also be written back by
the fresh producer. The store additionally treats malformed/read-failed state
as empty during mutations, allowing loss of other retained work.

The production chain remains transport failure -> existing pending-wake store
and BackgroundWakes -> canonical Session/Transcript -> existing connector
Runner -> exact marker settlement. These are existing owners; this change
adds no transport retry, scheduler, session database or permission authority.

### Requirements

- CWR-01: recovery uses an already persisted, fully specified ChannelDelivery
  marker. Agent/kind/work identity, matching delivery operation, address and
  failure reason are checked before enqueue. An incomplete marker is retained
  and refused; it cannot become a new operation or direct-main fallback.
- CWR-02: `SandPendingWakeStore::with_matching_marker_checked` reads the whole
  current image under its existing clone-shared mutex, admits only exact marker
  equality, and returns `Result<Option<R>, PendingWakeStoreError>`. Absent or
  replaced is `Ok(None)`; unreadable/invalid state is an error. Admission does
  no disk write. The closure only enqueues under BackgroundWakes; lock order is
  store then wakes. Spawn, Session/provider I/O and notifications run after both
  guards are released. No independent process or separately constructed owner
  serialization is claimed.
- CWR-03: carry the original marker with the internal queued failure so
  settlement preserves its exact timestamp, title, quiet origin and other
  metadata. Repeated rearm coalesces identical queued failures. A subsequent
  worker batch rechecks durable identity before execution, suppressing already
  settled or replaced snapshots. Requeue preserves order without duplicating
  the same retained failure. Existing unpersisted live-failure behavior remains
  distinct from a durable recovery claim.
  Review decision **CWR-LIVE-01 — FIX / IMPL_BUG**: a fresh transport failure
  whose marker cannot be persisted still reaches the existing volatile failure
  queue and tray. Keep its legacy payload without `pendingWakeMarker`; do not
  fabricate durable admission or overwrite unreadable state. Recovery of an
  existing marker continues to require successful checked exact admission.
  A real Host-lane negative must prove the live failure is retained and the
  unknown file bytes are unchanged after the actual workers finish.
  The independently reviewed initial tree was
  `d54f25f27396fcbe799832b38710a10dbb5de01b` against the authoring base above;
  this finding requires a fresh focused review of the fix and adjacent owners.
  Review source evidence does not substitute for exact-H Actions execution.
- CWR-04: successful recovery enqueue never clears ChannelDelivery custody.
  Missing provider, quiescence, spawn/read failure and unsuccessful follow-up
  retain the original marker. Only the existing positively completed follow-up
  with a sent message, or an explicit existing stale-session/deleted/group
  disposition, attempts exact-marker removal. A newer same-key marker survives.
  Failed marker removal is not reported as a successful durable settlement.
  Decision **CWR-AGE-01 — FIX / IMPL_BUG**, before the age-policy edit: the
  existing 48-hour startup prune cannot establish a terminal delivery outcome.
  Exclude ChannelDelivery from age-only pruning while retaining existing stale
  pruning for the other wake kinds. Test an old channel marker beside an old
  shell marker and rearm the channel after the threshold without changing its
  timestamp or metadata.
- CWR-05: add checked whole-image reads/mutations to the existing pending store.
  Missing file is known empty; explicit empty pending array is valid. Accept
  version 1 and the historical versionless envelope. Reject malformed JSON,
  unknown versions/shapes/fields, duplicate JSON members or marker keys,
  malformed required identities/time and mistyped optional fields. Preserve
  all original bytes on refusal and retain literal values without normalization.
  Lenient pure coercion/parsing remains inspection compatibility only. Existing
  mutation wrappers delegate to checked operations and never mutate after an
  unknown read; clear/prune success follows actual successful persistence.
  Host health treats an unreadable pending image as retained custody, so it
  cannot authorize idle teardown by projecting the unknown image as empty.
- CWR-06: write through a unique create-new temporary file, flush it, then
  rename once using the existing upgrade-store precedent. Never remove the old
  target to make a failed rename succeed. Cleanup touches only this operation's
  temporary file. Strict rearm startup propagates store refusal through the
  existing manager `Result`; it must not start a partial prefix from a corrupt
  image. Existing compatibility APIs remain callable.
- CWR-07: retain all existing CH/IH/CA/Session, deletion, quiet-origin and valid
  concurrency assertions. New real-file tests cover absent/versionless/v1,
  corrupt/unreadable/duplicate/mistyped state, write refusal, exact/replaced/
  cleared admission, and retained ChannelDelivery rearm. Real Host-owner tests
  invoke the production recovery enqueue and worker, reopen the store, and
  prove no new append/provider admission with missing provider or quiescence.
  Include incomplete marker, replacement, cleared snapshot, repeated enqueue,
  metadata-preserving exact settlement and store-failure negatives. Deterministic
  completion channels observe actual workers; no sleeps or fake terminal
  acknowledgments establish success.

### Review, verification and remaining scope

Review decision **CWR-ORACLE-02 — FIX / TEST_FLAW** on frozen tree
`cf5dc6b36f8583ffa7915ed4c0edfe0c03526dc1`: the manager's repaired-image
control rearms an ordinary Shell marker. Its successful watch handoff clears
that marker under the retained Shell contract; asserting retained custody
after that handoff contradicts the oracle. Keep the malformed-image refusal
and busy-health assertions, and require both the reopened store and health
projection to be empty after successful ordinary-Shell rearm. ChannelDelivery
retention assertions remain unchanged. No production behavior is changed for
this test correction; exact-H Actions execution is still required.

Allowed owner paths are the pending store/rearm, existing BackgroundWakes and
TranscriptManager if required, shipping main's channel failure helpers, their
existing contract/app tests, this contract, the existing channel dossier and
the relevant OCRM ledger/RTM rows. Shared composition/workflow/ledger edits stay
with the integrator. Select affected existing Rust contracts and shipping app
cases in `rust-desktop-runtime.yml`, and require independent frozen review and
exact-H Actions. Also execute the already-authored paired-device SSE filter
control: prior Runtime selected its library compilation but omitted its test
filter; adding that selection repairs evidence coverage without changing the
permission oracle. Source §17.2 gates and all eight false flags are preserved.

The exact-H macOS Host jobs for CWR publication `3c4bf6963cb30ecf01eb34b905315074886e1972`
and subsequent integration parents remained queued without executed steps at
the 2026-11-02 recovery check. This is missing execution evidence, not a failed
behavioral oracle. Continue the portable part of CWR in the existing Linux
`sqlite-native-path` job/cache: complete `durable_recovery_stores_contract`,
`pending_wake_rearm_contract`, `transcript_manager_contract` and the existing
`cloud_agent_pending_wake_rearm_production_contract` source-wiring supplement;
then the three `checked_pending_wake_tests` library controls and the exact
paired-device SSE filter. Preserve the macOS app selectors and shipping compile
gate. The Linux receipt must bind the new H/tree/U and source hashes, retain
each command's complete log and exit outcome, require each declared positive
name exactly once, and remain failed/incomplete if any selected check is
missing or fails. Preserve owned failed fixtures through the existing artifact
path. A passing Linux receipt covers real store/rearm/manager/locking controls;
controlled runtime-port outcomes do not execute the shipping provider or
quiescence branches. The source-wiring target does not execute the app. Shipping
app and real channel acceptance remain open; Linux app execution is unverified,
not declared unsupported. This is an evidence-carrier amendment with no change
to the CWR product contract, assertions, source-review states or completion flags.

This target integration correction reuses the CH source/owner contract above;
it does not relabel historical source reads or establish scoped source closure.
Real channel credentials/provider compatibility, original-turn unknown-effect
reconciliation, automatic readiness rearm after this bounded startup path,
cross-process writer arbitration, joined disposal and full packaged acceptance
remain open. A follow-up error still cannot prove that external effects were
rolled back; this batch authorizes no automatic transport retry. No schema
version is bumped. Rollback retains readable version-1 bytes and restores the
known recovery defect; no user state may be removed to roll back.
