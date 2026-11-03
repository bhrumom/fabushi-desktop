# Channel closure investigation: durable custody of a delivery-failure notice

Date: 2026-11-03  
Investigator: /root/next_channel_closure_gap  
Status: **ONE CONFIRMED SOURCE GAP; investigation only; no implementation, verification, or acceptance claim.**

## 1. Decision for the parent

The next bounded candidate is **preserving an entered delivery-failure followup across a Host restart**.

At the pinned target, a failure followup can append the required in-app SendMessage and successfully commit its Session result receipt, then fail to clear its exact pending-wake marker. The live BackgroundWakes owner correctly retains that work and refuses a second admission. Its EnteredRunner / RetainedAfterDispatch phase is only in memory. After a fresh manager or process starts, the unchanged durable marker is recovered as a new ActiveBeforeDispatch head, and the failure worker can invoke a second Runner with a fresh stream and client nonce. The first receipt has already released Session admission, so that existing gate does not prevent this particular replay.

The deterministic defect is a second admission of the same notification obligation after it already produced a message and result receipt. With a conforming provider that follows the failure prompt again, the user receives a duplicate in-app failure notice and another inference run. This finding does **not** assert that the original channel transport is retried; the prompt expressly asks for an in-app message and forbids silently retrying the channel. [N1] [N2] [N3] [N4] [N6] [N7]

This is a continuation of an explicitly open frontier, not a regression claim against the current live-only contract. The inbound handoff spec says the claims do not survive process loss and leaves post-dispatch custody across restart open. Root should adjudicate the candidate and amend that existing contract before code. [S3] (lines 963–972)

**Strongest exact trigger:** successful generated message **and committed Session result receipt**, followed by marker-cleanup refusal or process loss before marker cleanup. I do not claim that every crash after a message append will replay. A crash before receipt commit can leave Session admission occupied; that window needs its own existing-owner reconciliation analysis.

## 2. Immutable subjects and operating limits

| Subject | Identity |
| --- | --- |
| Target repository | bhrumom/fabushi-desktop |
| Target H | bb5a7f6098f952df6d388a672c4fb01045bc645c |
| Target tree | 5ad5f0e4a774947e22d6c3956775590503edac0c |
| Target acquisition | Read-only Git object reads from /tmp/ocrm-source-84120837.git; tree identity independently read with git rev-parse |
| Upstream repository | openclaw/openclaw |
| Fixed U | 1440bb1771372334bea23d7642bb52deb7db1845 |
| Fixed U tree supplied by parent | 90c8aab39972da11cfad220361781d49e1bc6700 |
| Upstream acquisition | Exact-U GitHub file reads because U is absent from the local object database; returned file blob identities retained below |
| Model and effort | Configured model and effort inherited; parent explicitly requires xhigh for identity, permission, cancellation, and recovery analysis. No override, additional CLI, or child agent was started. |

There was no floating-branch discovery, source rebaseline, checkout modification, project helper execution, build, test, lint, generator, schema validation, htch validation, push, workflow dispatch, or other agent's private review. Only this report is written. All execution evidence and all global completion flags remain unchanged.

## 3. Authority and owner boundaries

The applicable authority is the root AGENTS spec-first rule, OCRM SOURCE_OF_TRUTH current fixed-source contract, the current P0 task, primary Spec §17, and the current channel domain contracts. Primary §17 permits a source-bounded product batch after its actual owners and failure/permission contracts are known; it does not make source inspection into verification. It requires the batch to bind U/H, allowed paths, real producer/owner/storage/consumer flow, cancellation/restart, and causal oracles. [S1] [S2] [S9] [S10]

Current module ownership remains:

| Responsibility | Existing owner and preservation rule |
| --- | --- |
| M14 external channel ingress and connection identity | Existing channel adapters / connector owners; provider account, allowlist, thread and ingress cursor authority are not moved into failure recovery |
| M15 outbound delivery and failure reporting | Host channel delivery plus the existing Transcript delivery-failure wake path |
| M07 Session and Transcript truth | ProductionSessionWorkers / AgentDbOwner / TranscriptRuntime; the existing per-agent database remains the message and result authority |
| Live deferred-work custody | TranscriptManager's BackgroundWakes under its existing mutex and common drain |
| Durable pending-failure custody | Existing SandPendingWakeStore and PendingWakeRearm |
| Runner and tools | Existing routed provider task and tool/permission boundaries; requestSource remains connector |

The module map's historical “default channel not registered” wording is not a substitute for current composition. Current HostProductionExtensions constructs a real ChannelDeliveryRouter and installs it in HostChannelDelivery. The bounded caller search found its provider register/replace/unregister APIs used in definitions and tests, without establishing a shipping provider registration. A missing real provider is still an actual delivery error through the shipping lane. This report does not propose a new channel runtime or use missing broad transport integration as the selected defect. [S8] (lines 68, 75–76, 95–101) [N9]

Permission and identity facts relevant to this batch:

- The failure is produced from the existing authorized Host/Runner send path; it is not a new external inbound principal. The failure worker still passes the current Session key and lifecycle revision into background admission. Missing/deleted/group/stale Session decisions remain in their current owners. [N1] (lines 7563–7728, 7754–7768)
- Channel credential connect/disconnect continues through the Session gateway and the current credential/write-admission and operator authorization contracts. No token, account, principal, scope, or permission rule needs changing to close this replay window. [S6] [S7]
- The internal wakeForInbound branch accepts an opaque envelope. It is not evidence of a registered, authenticated external provider ingress. The inspected native registry has no wakeForInbound registration; the HTTP Gateway rejects an unregistered command before reaching that branch. No remote unauthorized-ingress exploit is asserted. The thread-binding spec itself requires a typed authenticated adapter event before connecting that behavior. [S5] [N1] (lines 13588–13621)
- No new parser, adapter, transport registration, permission authority, Session database, timer, scheduler, or Agent loop is warranted.

## 4. Actual production chain

### 4.1 Outbound send and failure producer

1. ProductionSendMessageSink appends the generated message through canonical Session/Transcript owners and obtains the durable entry ID. For a channel target it passes that ID as deliveryOperationId, with agentId, addressToken, current sessionKey and message, into the real deliverToChannel Host lane. [N1] (lines 1005–1145)
2. Shipping Host composition installs the extensions-owned HostChannelDelivery; the real lane calls dispatch_channel_delivery. That helper validates/builds the outbound message and starts the existing transport worker. A queued response is only worker acceptance. [N1] (lines 13181–13296, 17393–17394, 17850–17855) [N9]
3. ChannelDeliveryRouter resolves the current provider port. No registered matching port is an error; chunk delivery uses the existing message/delivery identities and does not add an automatic retry loop. [N10] (lines 650–930)
4. A delivery error yields the existing “Message not delivered” tray and calls queue_channel_delivery_failure. This writes a ChannelDelivery marker using the original delivery operation ID as work_id, then enqueues the live failure. If initial marker persistence fails, the existing explicitly volatile fallback remains separate. The selected witness uses a successfully persisted marker. [N1] (lines 13236–13280, 7808–7876)

### 4.2 Live failure work, real message, and settlement

5. The existing BackgroundWakes owner reserves one head with a claim ID and worker token. process_channel_failure_work_observed checks the exact live ticket, current durable marker, Session existence/group/provenance, provider configuration and upgrade gate. It captures the current Session key/lifecycle generation. [N1] (lines 7357–7407, 7535–7728) [N2] (lines 397–545, 704–815)
6. handoff_channel_failure_followup changes the live head to EnteredRunner under the queue mutex, calls the supplied real Runner delegate outside the lock, then checks ownership again. [N1] (lines 7036–7058)
7. The production delegate constructs the channel-failure prompt and calls run_local_background_revival_turn_with_context with source connector. The prompt asks for a SendMessage without a channel target. It does not include a durable followup receipt key. [N1] (lines 7730–7777) [N2] (lines 232–257)
8. That background helper allocates a new stream ID and client nonce, captures Session admission, accepts the hidden routed send, and starts the existing routed provider task. On a positive terminal, wait_and_settle_background_revival computes the generated-send count and calls the current Session result owner. [N1] (lines 6500–6736, 6739–6844)
9. AgentDbOwner commits background-result:<idempotency_key> in the canonical per-agent store.db. The same transaction clears admitted_work_id and sets terminal_result_idempotency_key. The success result follows COMMIT. Receipt equality is scoped to the provided work ID and idempotency key; the next randomly generated nonce is a different receipt identity. [N6] (lines 1993–2146, 6368–6381)
10. Only a nonaborted, nonquiesced result with sent_message_count > 0 enters marker cleanup. Each checked clear uses exact marker equality. Cleanup failure keeps the live head RetainedAfterDispatch and emits the existing failure diagnostic. This is correct live-owner behavior. [N1] (lines 7060–7091, 7425–7458, 7699–7726)

### 4.3 Startup is a real consumer of the persisted marker

11. A fresh TranscriptManager constructs BackgroundWakes::default(). The previous head phase and claim ID do not live in that new owner. [N5] (lines 193–205)
12. After Gateway/Runner resources are live, shipping main binds ProductionPendingWakeRuntime and calls TranscriptManager::rearm_pending_wakes. The runtime delegates ChannelDelivery to UnifiedGatewayApi::rearm_channel_delivery_failure. [N1] (lines 2167–2188, 2276–2284, 2839–2851, 17496–17509) [N5] (lines 411–436)
13. PendingWakeRearm reads the checked image, preserves ChannelDelivery against age-only pruning, and invokes the runtime for an ordinary existing Session. A successful enqueue does not clear the marker. [N3] (lines 528–549) [N4] (lines 224–254, 279–335)
14. queue_recovered_channel_delivery_failure checks exact marker equality under the existing store lock and reserves live work under the wakes lock. It has no durable dispatch phase or followup receipt identity to inspect. In a new owner, the first matching failure becomes a new ActiveBeforeDispatch ticket. The worker then follows steps 5–9 again. [N1] (lines 7878–7904) [N2] (lines 782–815)

The original receipt does not block the second admission: success cleared the old admitted_work_id, while capture_session_work_admission admits a fresh work ID when the Session is active and otherwise eligible. No part of the recovery helper passes the old nonce to that owner. [N6] (lines 2081–2095) [N7] (lines 2603–2652) [N1] (lines 6574–6580, 7878–7904)

## 5. The one concrete defect

### Preconditions

- A real ordinary Session remains eligible, with a current matching sessionKey.
- Inference is configured and upgrade quiescence is off when recovery is allowed.
- The original channel message has a durable deliveryOperationId and a valid persisted ChannelDelivery marker.
- The first failure followup has already produced at least one real generated SendMessage and successfully committed its Session result receipt.
- The exact original pending marker survives because marker cleanup refuses or the process exits after receipt commit and before marker cleanup.
- The pending store is readable at the next startup. For a temporary cleanup failure, use the exact retained image after that condition clears; no fabrication or rewriting of the marker is required by the crash witness.

### Causal sequence

| Moment | Durable state | Live state | Consequence |
| --- | --- | --- | --- |
| First failure is queued | Original operation/address/reason/session marker is present | ActiveBeforeDispatch | One notification obligation |
| First followup starts | Marker unchanged | EnteredRunner | Runner may cause effects |
| First followup succeeds | Actual in-app message and background-result receipt are committed; Session admitted_work_id is cleared; marker still present | EnteredRunner until cleanup | Effects and completion are already real |
| Checked marker cleanup refuses | Exact marker remains | RetainedAfterDispatch | Later same-owner readiness correctly cannot replay |
| Owner/process is recreated | Same marker and message/receipt persist | New empty BackgroundWakes | Entered phase is lost |
| Startup recovery admits marker | Exact marker still matches; old receipt is not consulted | New ActiveBeforeDispatch with new claim | Same notification obligation is executable again |
| Followup starts again | New stream/nonce has no idempotency match with first receipt | EnteredRunner again | Second Runner admission; a conforming second SendMessage duplicates the notice |

**Root cause:** durable pending-failure identity captures the failed transport operation but not the notification attempt's entry/settlement state. Live no-replay custody is being reconstructed from a marker that cannot distinguish “not yet entered” from “already entered and produced effects.” Exact marker equality and exact Session receipt equality each work as specified; there is no durable bridge between them for this recovery decision. [N3] (lines 48–71) [N1] (lines 7486–7533, 7878–7904)

**Storage and consumers:** the marker is in <pending-wake root>/host-pending-wakes.json (SandPendingWakeStore; filename constant in durable_file_policy.rs line 3). Messages and result receipts are in <agents root>/<agent_id>/store.db through the existing AgentDbOwner. The marker's consumer is PendingWakeRearm → production Gateway failure recovery; the receipt's consumer is current background-turn settlement. This recovery path does not correlate the two. Session path ownership is explicit in session_paths.rs lines 17 and 94–99 and production.rs lines 1052–1057.

### Error, cancellation and restart boundaries

| Boundary | Current fact and limit |
| --- | --- |
| Same-manager Settings, resume, or repeated recovery after entry | Correctly held by the claim phase; no second admission for the retained head |
| Provider unavailable / upgrade pause before entry | Correctly retains pre-dispatch work; later readiness or startup recovery is intentional |
| Positive receipt followed by cleanup refusal | Chosen confirmed defect after new-owner recovery; no ambiguous-result assumption is needed |
| Cancellation, zero-send result, terminal or completion error after entry | Live owner conservatively retains. Cancellation is not proof of no effects. The background waiter has distinct existing admission-release branches; the proposed durable continuation must preserve the no-replay distinction rather than parse errors |
| Crash before receipt commit | Not used as the decisive witness: outstanding Session admission may prevent a fresh run until its existing owner resolves it |
| Manager close / old ticket | Correct permanent-close and stale-token refusals apply to the old owner. A new manager is a distinct object; closing the old one is not a durable disposition of its marker |
| Stale/replaced marker | Exact currentness checks already suppress it. A newer same-key marker must continue to survive stale cleanup |
| Old marker age | ChannelDelivery already bypasses age-only pruning. No age-policy defect is asserted |
| Missing initial durable publication | Existing volatile fallback is an explicit separate limitation; this investigation does not claim it becomes durable |

## 6. Existing tests already cover the live half

The test source is strong enough to locate the next oracle precisely; it was not executed in this investigation.

- channel_failure_checked_receipt_cleanup_refusal_retains_head_and_tail uses the real controlled-final-I/O followup fixture. It proves an actual Session result receipt exists before making the pending path unreadable, observes Held, restores the exact original image, then exercises Settings, resume and recovery. All those rearm calls pass the same fixture.wakes Arc. Thus the test proves retention within that manager lifetime, not reconstruction into a new one. [N8] (lines 1087–1179)
- The fixture's real_followup uses actual Session admission, Transcript acceptance, generated message append, routed lease/terminal handling and wait_and_settle_background_revival, then checks the durable background-result row. A counted fake callback alone is not needed to demonstrate the effect boundary. [N8] (lines 284–367)
- channel_failure_dispose_refuses_snapshotted_callback_and_reserved_ticket correctly proves the closed owner rejects stale work, and separately constructs a new manager with a fresh ActiveBeforeDispatch claim. That test's original ticket is pre-dispatch; it does not prove retention of an entered durable obligation across reopen. [N8] (lines 805–856)
- Existing cancellation/lost-terminal/zero-message controls retain an entered live head and do not imply completed external effects were undone. Preserve their assertions. [N8] (lines 1492–1550)

Do not weaken or replace these controls. Add the new-owner continuation to the same production path and fixture. This report makes no claim that every repository test was read or that a search proves every possible test absent.

## 7. Fixed-U comparison and dependencies actually read

The native defect is established by native production sources above. Fixed-U review informs the effect/custody contract; it does not authorize transplanting OpenClaw's queue or runtime.

Relevant source semantics at exact U:

1. deliver-queue-state.ts has a pre-send transition before the effect boundary. Its required queue policy does not proceed when that publication fails; best-effort behavior must safely release the replayable intent or refuse. Its post-send handling records unknown outcome and uses an after-send failure classification rather than treating the work as definitely unsent. [U1]
2. delivery-queue-storage.ts carries exact attempt identity through start, dispatch, unknown and fail-after-send operations. delivery-queue-worker-store.ts routes those operations into the existing state worker, not an independent store. [U3] [U6]
3. delivery-queue-sqlite-claim.kernel.ts compares the exact stored attempt/producer claim inside the existing immediate transaction. Its recovery branch requires matching reconciled attempt identity/time; unknown-after-send replay additionally requires an explicit allowUnknownSendReplay flag. The state type and hasLiveDeliveryQueueClaim dependency were read at the relevant contract. [U2] [U7]
4. delivery-queue-recovery-policy.ts permits ambiguous-final-text replay only with its explicit policy and evidence conditions. Therefore the accurate upstream claim is **identity- and policy-qualified recovery**, not “OpenClaw can never replay any unknown send.” Fabushi's current failure-notice contract independently forbids automatic reentry of its retained live head. [U4] [S3]
5. send.ts distinguishes possible recipient effects (including partial/identity-unknown outcomes) from a proven no-effect result. The inspected ingress monitor ranges similarly maintain handoff/settlement custody rather than freeing an entered receipt merely due to cancellation. Neither observation proves all channels or database crash guarantees. [U5] [U8]

### Exact-U read register

“Full” below is a content-read fact for that file, not global review or acceptance. “Range” is not promoted to full read. All rows use U=1440bb1771372334bea23d7642bb52deb7db1845.

| Upstream path | Returned Git blob | Read extent / use |
| --- | --- | --- |
| src/infra/outbound/deliver-handoff.ts | bf37c6c9456a6535a35aceac19f170d13b08a4f8 | Full; proven-unsent handoff failure boundary |
| src/infra/outbound/message.ts | 5032acef657250cbce6b85435f5215237825163b | Ranges 1–160, 275–485; context forwarding and durable outcomes |
| src/infra/outbound/deliver-types.ts | af31fedd91e1510575da041edb11ab5517857ab5 | Range 1–210; typed no-dispatch/possible-effect contract |
| src/channels/message/runtime.ts | 104207fc00b17cf78373ab4bc5f2963723a3325e | Full; barrel dependency |
| src/channels/message/send.ts | 318f146b5cc3c6a2f7e33a3cd99622d99257c6d0 | Full; real durable-send context, intent and result consumers |
| src/infra/outbound/deliver.ts | 0865ddc6c27b60c23b02280618934efdb4bc8c16 | Full; barrel dependency |
| src/infra/outbound/deliver-queue.ts | 395433da7382f2885b0ab2497385425a5ee3614d | Ranges 1–116, 153–197, 252–275, 330–380, 485–end; queue ownership and effects |
| src/channels/message/ingress-monitor.ts | 9d37fc03b0807d9f3b9f06e69662a7ac2d631a41 | Ranges 239–383, 421–480, 650–end; handoff/stop/re-pump boundaries only |
| src/infra/outbound/deliver-queue-state.ts | 5b4f37f3b8f7372cec97f370cf8c36879fe61033 | Full; pre/post-send state publication |
| src/infra/outbound/delivery-queue-recovery-policy.ts | 33c1e90ca4c206f584063c0e1c8d9b4e48012b2d | Full; explicit ambiguous replay policy |
| src/infra/outbound/delivery-queue-platform-lease.ts | f0a4c24528769450e8e3bb4d9a5e652195589c1b | Full; exact claim API |
| src/infra/outbound/delivery-queue-storage.ts | 97363fd1ec8b894855d75dbf5d086e9da4bbc85c | Full; persisted mutation API and stable intent ownership |
| src/infra/delivery-queue-worker-store.ts | 3d96051f548c7976a9fe3ccfc3039d0aae7881ec | Full; existing state-worker delegation contract |
| src/infra/delivery-queue-sqlite-claim.kernel.ts | bebcfa716d4cb628fa0e58230c4a082d18d162aa | Full; exact claim/recovery transition implementation |
| src/infra/delivery-queue-sqlite.types.ts | f94e91128c92416ba5d9e43accb49eeed7f51088 | Range 86–131; state fields and hasLiveDeliveryQueueClaim contract |

The deeper OpenClaw state-worker broker, SQLite implementation, schema migration, every adapter/provider, real network behavior, and platform shutdown were not fully reviewed here. No crash-durability or ecosystem-wide compatibility assertion rests on an unread dependency. No inventory, source-read status, review status, responsibility mapping, or global flag is changed by this register.

## 8. Narrow spec amendment required before implementation

Root should add a continuation to the existing channel-inbound-worker-handoff contract, with a small cross-reference from the existing CWR recovery section if needed. Suggested working label: **OCRM-CHANNEL-FAILURE-RESTART-CUSTODY-v1**. This is a proposal, not an adopted repository contract.

The amendment should fix the user-visible behavior: **once the exact durable failure-notification obligation crosses the followup effect boundary, creating a new manager must not make it executable again solely because the process-local head is gone.**

Freeze these decisions before code:

1. **Persist the distinction in the existing owner.** Bind the exact original marker(s) to a durable notification-entry/attempt identity or an existing Session turn/receipt identity before Runner may produce effects. Use the current SandPendingWakeStore and Session owners. A replacement must preserve original operation, timestamp, reason, quiet origin, title, address and provenance. Do not add another log, queue database, scheduler, or recovery loop.
2. **Checked publication precedes dispatch.** If the durable entry fence is absent, replaced, unreadable, or cannot be published, do not invoke the corresponding durable followup. Do not obtain execution authority from a successful in-memory transition alone. Keep marker I/O outside the wakes mutex, preserve the existing store→wakes lock order and late ticket rechecks, and make close/replacement races explicit.
3. **Reopen is monotonic.** Startup may recover a positively known pre-dispatch failure. It must retain an entered/unknown failure without a second Runner invocation, even after cancellation, terminal loss, zero-send result or cleanup refusal. Reading a marker, acquiring a queue ticket, observing a success tray, or process death alone is not a disposition.
4. **Keep completion separate.** A matching existing Session result receipt can support exact cleanup only if the new durable binding proves it belongs to this notification obligation and generation. A retention-only first batch can hold unknown entered work without implementing generalized original-turn reconciliation, but it must state that remaining liveness limit and expose it through existing status/diagnostic surfaces. Do not label a held marker “settled.”
5. **Specify a multi-marker head.** One live followup may cover several exact markers. Freeze whether the existing whole-image owner publishes the admitted marker set atomically, or define an equally safe partial-publication state. A crash between per-marker writes must not leave an already covered marker replayable or let one marker's cleanup settle another. SandPendingWakeStore already has replace_if_matches_checked; do not create a redundant generic store. If a set operation is needed, extend that same owner.
6. **Resolve legacy data and rollback explicitly.** Current versionless/v1 markers have no phase or followup identity, so they are intrinsically ambiguous after a restart. The current strict parser rejects unknown fields/versions. Do not silently label every old marker “never entered,” because that recreates the defect. Do not delete legacy data to make rollout work. The spec must choose a fail-closed legacy hold/reconciliation policy, distinguish newly written known-pre-dispatch markers, and state old-reader/rollback behavior. A schema or reader compatibility change is a real shared contract and must be reviewed as such.
7. **Preserve neighboring duties.** Keep exact marker replacement protection, age retention, initial volatile fallback semantics, Session generation/deletion checks, old-owner token refusal, ordered tail behavior within a live head, Bot message behavior, original transport no-retry, credential gates, and unrelated CloudAgent/Subagent/Shell wake behavior. Do not broaden the batch into new ingress transport or delivery registration.

The existing CFL text explicitly forbids a persistent schema addition for that older live-only batch. The new amendment must supersede that narrow restriction only for this chosen durable continuation; it cannot be treated as already authorized by the old implementation scope. [S3] (lines 946–952)

## 9. Proposed non-overlapping implementation boundary

No path is authorized for editing by this investigation. After the spec and shared interfaces are frozen, root can reserve a single serialized channel batch with this bounded candidate set:

| Ownership for the later batch | Exact paths / allowed purpose |
| --- | --- |
| Durable marker owner | source/host/src/extensions/transcript/sand_pending_wake_store.rs — phase/binding representation, strict parsing, exact transition, compatibility tests in the same module |
| Recovery consumer | source/host/src/extensions/transcript/pending_wake_rearm.rs — refuse reentry of durably entered/unknown obligations and preserve existing other-kind handling |
| Shipping composition, root integrator only | source/host/app/src/main.rs — only channel-failure producer, durable admission, recovery, followup context and exact settlement helpers; preserve the current Runner and common drain |
| Live owner, only if needed by the frozen interface | source/host/src/extensions/transcript/background_wakes.rs — expose/reconstruct held phase through the current owner; do not add a new queue |
| Direct causal app oracle | source/host/app/src/tests/channel_failure_readiness_tests.rs — extend the actual receipt/cleanup fixture with a same-root fresh-manager case |
| Existing Host lane and durable recovery controls | source/host/app/src/tests/channel_host_lane_tests.rs; source/host/tests/pending_wake_rearm_contract.rs; source/host/tests/durable_recovery_stores_contract.rs — only selected current owner tests affected by the contract |
| Spec and evidence selection, root integrator only | projects/openclaw-rust/docs/specs/channel-inbound-worker-handoff.md; a narrow cross-reference in channel-outbound-host-lane.md if needed; existing workflow/dossier/ledger/RTM entries only after root freezes exact affected rows and selectors |

A new field in DurablePendingWakeMarker has a finite mechanical compilation surface. A source search at H found literal constructors in the main file and store above, channel_host_lane_tests.rs, and these additional tests:

- source/host/tests/async_task_union_contract.rs
- source/host/tests/run_lifecycle_activity_production_contract.rs
- source/host/tests/transcript_manager_contract.rs
- source/host/tests/durable_recovery_stores_contract.rs
- source/host/tests/pending_wake_rearm_contract.rs

If the chosen schema requires those literals to change, root should enumerate them in the frozen allowlist as **mechanical field initialization only**, preserving every unrelated wake oracle. This is a search result for blast-radius planning, not proof of complete compiler coverage. No other production modules, credentials, ingress SDK helpers, platform adapters, package files, dependency manifests, lockfiles, or native communication schemas are included.

The shared main/spec/schema/workflow/ledger surfaces are not parallel editing lanes. Root retains them; no worker may overlap or widen them based on this report.

## 10. Causal oracle for the later Actions batch

Use the already production-used process_channel_failure_work_observed path and real followup fixture. Only final provider I/O may be controlled. Do not copy the worker state machine into a test or use a counted dummy callback as proof of a generated message.

### Decisive negative/positive pair

1. Produce an actual channel failure through the existing Host lane, or reuse its exact persisted marker fixture together with the existing Host-lane positive. Keep a valid current ordinary Session and configured provider.
2. Execute the actual failure followup fixture so it commits both one generated in-app message and one background-result receipt. Capture their real identities and the original marker.
3. At the already-observed post-receipt/pre-cleanup point, make checked marker cleanup refuse using a real filesystem condition. Observe worker completion as Held, not a timeout. Restore the exact retained image without changing marker metadata. This gives the same durable state as receipt-committed/process-lost-before-cleanup, with no orphan test worker.
4. Dispose the old manager. Reopen the same Session database and pending store through fresh production owner constructors, and exercise the real startup rearm adapter into fresh BackgroundWakes. Preserve the root directory across fixture recreation.
5. Count actual followup admission, generated rows and result receipts. At H the code admits another followup; after the fix, counts must remain one, the entered obligation must remain explicitly held or exactly reconciled under the adopted contract, and no second provider call may occur.
6. In a separate positive control, persist a new marker positively known never to have entered Runner. Recreate owners and prove it is recovered once, produces one real message/receipt, and settles through exact cleanup. This prevents “fixing” the defect by disabling all recovery.

### Required adjacent negatives

- Durable entry publication failure makes zero Runner calls and preserves the original/unknown image.
- A replaced same-key marker survives a stale worker's cleanup or phase publication.
- Partial multi-marker completion and store refusal cannot make the remaining covered markers replayable after reopen.
- Cancellation or terminal loss after an observed real message remains held across reopen; no-effects inference must not be made from an error string or timeout.
- Settings/resume/repeated recovery still cannot reenter the same live or recovered entered obligation.
- Old-manager close and late tickets do not repopulate the old queue or clear a successor's marker.
- Other agents and CloudAgent/Subagent/Shell markers retain their existing recovery and pruning behavior.
- Legacy marker and old-reader compatibility follow the newly frozen spec; malformed/unknown images remain byte-preserving errors.
- The original failed external transport call count does not increase during failure-notice recovery.

Use deterministic barriers and existing completion channels, not sleeps, broad process cleanup, or a fabricated terminal receipt. Add only the affected exact selectors/receipt inputs to the existing Actions workflow. The current workflow's exact-H compile, contract and linked shipping-App checks remain the evidence gate; real provider/channel/platform acceptance remains separate. No such execution occurred here.

## 11. Read ledger and precision limits

### Authority read extent

- Root AGENTS read for repository-wide spec-first, selective-context, effort, canonical native ownership and OCRM Actions-only rules.
- Primary Spec §17 read completely; relevant §7–9 ownership/channel/identity/unknown-outcome clauses read by range. No claim of full primary-spec review.
- SOURCE_OF_TRUTH current identity/authority section and the applicable current channel history read by range; no claim of full historical file read.
- Current P0 task read at its current scope and relevant worker/cancellation/receipt sections by range; historical entries are not current evidence.
- Module map M03/M07/M14/M15 and §3.1 read; its historical production map was checked against current source.
- channel-thread-bindings.md, channel-gateway-credential-authorization.md and channel-credential-write-admission.md read in full.
- channel-outbound-host-lane.md current CH/CWR/chunk contract and limitations read through its sections; no implementation or acceptance status was inferred from the prose.
- channel-inbound-worker-handoff.md read by applicable sections/ranges, including current live custody (767–865, 901–973), checked receipt continuation (996–1079), existing inbound/readiness sections and their stated limitations. This was not a historical full-file completion claim. Root must finish any additional applicable schema/legacy contract reading before adopting the new durable continuation.

### Load-bearing native identities and source ranges

All native rows are at H; “range” never means complete-file review.

| Reference | Path | Git blob | Main read extent for this finding |
| --- | --- | --- | --- |
| N1 | source/host/app/src/main.rs | 4831ae8cf002ce64e6409f2d695b48aaa1914c45 | Sink 1005–1145; runtime 2150–2188, 2276–2284, 2839–2851; background 6500–6845; failure handoff/helpers 7036–7091, 7357–7592, 7594–7904; dispatch 13181–13296; Gateway 13416–13429, 13588–13621; Host wiring 15093–15098, 17370–17398, 17496–17509, 17850–17855 |
| N2 | source/host/src/extensions/transcript/background_wakes.rs | 92ab04a65334be1d83018885e3b269f5628d631d | Ranges 206–273, 380–550, 700–825 |
| N3 | source/host/src/extensions/transcript/sand_pending_wake_store.rs | 50910a5b438b1eedeb980a23426929dbfa9e1df3 | Production implementation 1–647; not all later tests |
| N4 | source/host/src/extensions/transcript/pending_wake_rearm.rs | 60604aec8d774518e8af13dc8f2b13394b8a6ea1 | Range 1–340 |
| N5 | source/host/src/extensions/transcript/transcript_manager.rs | caa2bf73398d4499d51d3152b494ff2d605b4e54 | Ranges 170–220, 386–452; startup/owner symbol search |
| N6 | source/host/src/extensions/session/agent_db.rs | 78740768f7f1b0307dfd6756c4d50c368a91b721 | Ranges 698–724, 1993–2160, 6364–6388; receipt identity symbol search |
| N7 | source/host/src/extensions/session/production.rs | f585fae13720e08f34323bed82759253590a3753 | Ranges 1048–1072, 1380–1450, 1530–1630, 2600–2664 |
| N8 | source/host/app/src/tests/channel_failure_readiness_tests.rs | 6b25aa1e9855e9d6bd76e39295b5c7e9756125cb | Ranges 284–378, 805–857, 1040–1184, 1490–1550; no execution |
| N9 | source/host/src/host_production_extensions.rs | 36b5dbac27d346f883150df179372034e0bea223 | Current catalog/router/composition and provider register/replace/unregister ranges |
| N10 | source/host/src/connectors/channel_delivery.rs | 52782327808aa14330821a6da7091bd4742855d2 | Range 650–930; router/port/Host consumer |

Supporting target channel sources were read without promoting them to shipping adapters:

| Path under source/host/src | Blob | Extent |
| --- | --- | --- |
| extensions/plugin_sdk/channel_ingress.rs | 98a54cd251b61503c7a4084fc6c3d21bfd44a519 | Full |
| extensions/plugin_sdk/channel_inbound.rs | 60d98bc1dd4f09dcbba23f20ed97126f7481e233 | Full |
| extensions/plugin_sdk/channel_outbound.rs | d20eda8687911dfd1f2a203706c38f23595de52f | Full |
| extensions/plugin_sdk/channel_lifecycle.rs | 3add85e7358b559d5481cf5a9d7f2f7c7fcd9772 | Full |
| extensions/plugin_sdk/channel_runtime.rs | 25dbd086526d46e680eda47d5b77deb540b02f24 | Full |
| extensions/plugin_sdk/channel_entry.rs | 6e2b929e415f44c527aa87cb3ba27d211f4a510c | Full |
| extensions/session/channel_store.rs | c1af306ecc5040132913a4eabe31e7447cd00244 | Full |
| connectors/channel_attachment.rs | 723006c2a5b054fc9ddc47f58fdb4ebd8639351e | Full |

Additional bounded supporting reads include Session gateway 750–875; Gateway HTTP authorization/dispatch 930–1045 and 1740–1950; the native method registry 1–210 and targeted symbol searches; filename/path contracts in durable_file_policy.rs and session_paths.rs. No global absence or security acceptance is inferred from those ranges.

## 12. Exit condition and remaining evidence

The investigation goal is met with **one source-confirmed, production-reachable restart gap**, a precise successful-receipt witness, current owners, explicit permission/cancellation boundaries, fixed-source comparison, and a causal next oracle.

Implementation has not begun. The required next decision is root's adjudication and durable spec/interface/legacy-policy freeze. Only then should the bounded existing-owner change and its tests proceed through the shared integration gates. Scoped source coverage, verified behavior, platform acceptance, full OpenClaw parity, and all final G01–G12 / AC01–AC24 claims remain open.

## Source links

Native links bind to the immutable target; upstream links bind to the immutable U.

[N1]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/app/src/main.rs
[N2]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/extensions/transcript/background_wakes.rs
[N3]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/extensions/transcript/sand_pending_wake_store.rs
[N4]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/extensions/transcript/pending_wake_rearm.rs
[N5]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/extensions/transcript/transcript_manager.rs
[N6]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/extensions/session/agent_db.rs
[N7]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/extensions/session/production.rs
[N8]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/app/src/tests/channel_failure_readiness_tests.rs
[N9]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/host_production_extensions.rs
[N10]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/source/host/src/connectors/channel_delivery.rs
[S1]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/AGENTS.md
[S2]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/docs/specs/openclaw-rust-equivalence-migration.md#L671-L720
[S3]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/docs/specs/channel-inbound-worker-handoff.md
[S4]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/docs/specs/channel-outbound-host-lane.md
[S5]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/docs/specs/channel-thread-bindings.md
[S6]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/docs/specs/channel-gateway-credential-authorization.md
[S7]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/docs/specs/channel-credential-write-admission.md
[S8]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/module-map.md
[S9]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/SOURCE_OF_TRUTH.md
[S10]: https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md
[U1]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/outbound/deliver-queue-state.ts
[U2]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/delivery-queue-sqlite-claim.kernel.ts
[U3]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/outbound/delivery-queue-storage.ts
[U4]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/outbound/delivery-queue-recovery-policy.ts
[U5]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/channels/message/send.ts
[U6]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/delivery-queue-worker-store.ts
[U7]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/infra/delivery-queue-sqlite.types.ts


[U8]: https://github.com/openclaw/openclaw/blob/1440bb1771372334bea23d7642bb52deb7db1845/src/channels/message/ingress-monitor.ts
