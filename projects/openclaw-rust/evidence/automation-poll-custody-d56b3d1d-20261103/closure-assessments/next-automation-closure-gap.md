# OCRM next Automations batch — read-only production-gap investigation

Date: 2026-11-03 UTC. Subject: one possible next coherent product batch while the separate auth-recovery batch runs. This report is investigation and design input, not implementation authorization, executable verification, independent acceptance, or an OCRM completion record.

## 1. Result and exact subject

**One concrete source-confirmed gap remains: an already executed, safely settled coalesced backend event can execute again after normal history eviction and owner reconstruction while its remote completion/ACK is still unresolved.** The decisive delivery is an alias of the original coalesced execution. The primary execution has a durable cron row; the alias does not. Once the single terminal history row containing all aliases is pruned, checked lookup truthfully returns absence, fresh admission accepts the alias, and the primary-only cron projection does not stop it. The local Runner receives a new stream and nonce. This can repeat a non-idempotent effect. [F01:488–503; F05:259–305; F06:493–551,623–636; F07:885–900; F10:2348–2378; F14:6092–6098]

This is a deterministic conclusion from the pinned source and actual call chain. No reproducer was executed, no live backend incident was observed, and no claim is made that a particular deployed service currently redelivers this exact sequence. The counterexample uses an unacknowledged delivery that the current client explicitly treats as still owed. It does **not** require arbitrary replay after an acknowledged delivery has been discharged. The current completion-safety spec separately marks deployed service semantics and restart durability unresolved. [S05:12–22,33–50]

| Subject | Exact identity |
| --- | --- |
| Target repository | `bhrumom/fabushi-desktop` |
| Target H / investigation base | `bb5a7f6098f952df6d388a672c4fb01045bc645c` |
| Target tree | `5ad5f0e4a774947e22d6c3956775590503edac0c` |
| Read-only source store | `/tmp/ocrm-source-84120837.git` |
| Fixed upstream repository | `openclaw/openclaw` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |

H/tree were checked with Git object reads. U objects were not available in the bare repository, so exact commit/tree/file API reads were used. The U commit/tree association was checked through the exact Git commit API. No floating branch, latest-upstream discovery, or rebaseline was used. The current SOT fixes this U; its older ad8d paragraphs are historical. [S02:26–43,55–71]

**Decision for the integrator:** this is a valid bounded candidate for spec repair and a subsequent existing-owner Rust batch. It is not ready to be called verified. A permanent pin of every terminal run, a second journal owner, a different scheduler, or removal of existing Abandon behavior would not be an appropriate repair. Sections 7–10 describe the necessary contract decisions and distinguish the smallest safety correction from complete restartable outbound settlement.

## 2. Authority read and scope

The governing order used here is the explicit read-only assignment, current root AGENTS, current project SOT, main Spec §17, current task/M19 map, and the applicable Automations/Runner domain contracts. Main §17 permits relevant-source, risk-based investigation and implementation batches without waiting for global P0 completion, but requires the real producer → owner → storage/service → consumer chain and failures/restart to be bound to exact U/H. All executable OCRM validation remains GitHub Actions only. [S01:5–112; S02:18–24,73–82; S03:671–720]

The complete current `cron-session-reaper.md` domain spec and the complete `agent-loop-runner.md` were read. The former contains the earlier history ACK, fresh admission, owner-recovery, cancellation/deadline, typed tool-custody, checked-history, backend discovery, admission-result, fresh-custody and legacy-slot repairs. Those requirements remain valid. The latter's historical baseline/acceptance passages are not promoted to current-H evidence. Main §17 governs execution timing. The current task summaries and M19 row were reviewed as navigation and status, not as proof of a new implementation authority. Precise read extents and content identities are in §12.

The finding is confined to normal healthy storage, a stable same-account execution context, a valid enabled event automation, safe known execution settlement, an unresolved backend obligation, ordinary local run-history retention, and reconstruction of the existing owners. It does not depend on corruption, a malformed fire, a deletion race, an unknown tool effect, an incorrectly normalized history record, or a cancellation being mistaken for settlement.

## 3. The actual production chain

| Stage | Current producer / owner / consumer | Load-bearing behavior |
| --- | --- | --- |
| Backend ingress | `ProductionAutomationsLifecycle` constructs `ReqwestAutomationsBackendTransport`; `ProductionAutomationsBackendRuntime` owns `AutomationFireBackendRuntime` | Shipping lifecycle uses the Sand transport. The separate strict native adapter is not wired here. [F04:119–169; F15; S06:43–46] |
| Poll and volatile claim | `AutomationFireBackendRuntime::tick_with_state` | A new event ID is inserted as Running, then dispatched. IDs still present in any state suppress redispatch in that runtime. [F03:240–307] |
| Shipping composition | Main startup → `dispatch_production_backend_fire` → `dispatch_checked_production_backend_fire` | Actual checked target discovery and checked history precede launch. This is the same helper used by the App backend oracle. [F14:3893–4002,5868–5912,17549–17556] |
| Preparation | `prepare_backend_fire` | Unreadable/ambiguous custody abandons; known Running abandons; exact terminal primary/alias replays completion; only checked absence reaches definition/event policy. [F02:41–145,162–197] |
| Event batching | `AutomationRuntime::enqueue_event_automation_fire_with` → `AutomationEventFires` | Actual queue/debounce produces one primary UUID and remaining coalesced UUIDs. Every item receives the same batch outcome and each original backend fire has its own completion callback. [F05:97–155,259–305; F07:981–1035] |
| Fresh durable execution claim | `AutomationRunPath` through `AdmittedAutomationRunStore` | Current Session filesystem/deletion admission wraps `FileAutomationStore::begin_fresh_run`. It checks the whole retained custody image and all requested primary/alias IDs. It persists one Running history record. [F01:204–296; F06:493–543; F07:348–368,885–900] |
| Session cron projection | `ProductionSessionWorkers` → `SandAgentDb::begin_cron_session_run` | One cron row is registered for the admitted **primary** run ID. The SQL key is `agent:<agent>:cron:<automation>:<primary>`. Neither the API nor the schema records alias IDs or completion/ACK state. [F06:546–551; F09:2928–2987; F10:2318–2385; F11:30–48] |
| Actual execution | Main `run_local_automation_turn` → existing routed Transcript/Runner composition | It admits the canonical main session and creates a fresh UUID stream and fresh UUID nonce. It starts the existing provider task and waits for an actual typed terminal. This path supplies no durable backend-alias nonce to deduplicate a later new run. [F14:6056–6223] |
| Execution settlement | Typed terminal waiter → RunPath | Missing/unknown tool-effect custody retains Running and returns None. Safe Completed settles cron first, then file history; only after settlement does the observation consumer receive a result. [F14:5934–6049; F06:607–636,733–759] |
| Remote completion | Main callback → production callback → `complete` | Some completion becomes volatile Completed; success of the report advances to Reported. An error keeps the same pending completion. [F14:3947,3990–3992,5901,5914–5931; F08:180–202; F03:187–204] |
| ACK and retirement | Next backend poll | Only Reported IDs are sent in `ackRunUuids`. After a successful poll, Reported IDs absent from the returned event IDs are removed from the in-memory map. There is no FileAutomationStore or Session release receipt. [F03:263–307] |
| Reconstruction | New production backend plus reopened Automation/Session owners | New backend state is an empty BTreeMap. The checked history file is the persisted primary/alias lookup authority. Healthy cron reopen has no erased alias or remote phase from which to rebuild this relationship. [F03:115–151; F08:79–95; F09:1666–1720; F11:30–48] |

The neighboring listener/webhook route is not a second execution owner. `dispatch_production_listener_event` sends local listener fires into the same event coalescer with `run_uuid: None` and the same `run_local_automation_turn`; it has no Sand fire-completion callback. The normal Run Now command and workflow-to-Automation Run Now command use the same manual Runtime/RunPath and can legitimately generate the later history rows. [F14:3821–3862,13944–14017] For the counterexample a valid GitHub event definition is server schedulable; `is_server_schedulable` excludes the specific Slack `@` listener case, not all event automations. [F20:105–109]

## 4. Concrete trigger and causal failure

Use P for the original coalesced execution's primary backend UUID and A for a coalesced alias. All definitions, principals and storage remain valid throughout.

| Step | Real action and resulting state |
| --- | --- |
| 1 | The backend poll supplies two valid matching events P and A. The actual coalescer queues both behind a controlled existing batch, then flushes them together. It selects P as primary and A as alias. No hand-written alias history is required. [F05:259–294; T03:1357–1485] |
| 2 | The real Runtime freshly admits the batch. History contains one Running row `id=P, coalescedRunIds=[A]`; cron contains one primary row P. A controlled executor performs one externally observable fixture effect and returns safe `CompletedWithFences`. In the shipping application the same executor position is `run_local_automation_turn`. [F06:517–551,593–636; F07:1002–1024] |
| 3 | Both normal execution stores settle successfully. The history row is terminal `ok`, and the cron row P is terminal. The callbacks attempt remote completion for each original backend fire. Hold A's completion in an unresolved response class, such as a transport lost response; the current consumer retains A as Completed and sends no ACK for A. This matches existing completion safety, not an invented server-error-as-success rule. [F03:187–204,332–364; S05:12–22; T02:468–522] |
| 4 | Execute twenty later ordinary manual runs of the same automation, with later start times. Each goes through fresh admission, actual executor, and terminal settlement. After the twentieth admission/settlement, the old terminal P row no longer fits the twenty-row retention window. Its A alias disappears with it. No file tampering or corruption is involved. `retain_run_history` protects Running and the just-settled target only. [F01:274–296,315–324,488–503; F18:9; F07:773–821] |
| 5 | Stop/reconstruct the actual backend and Automation/Session owners from those same persisted files. This healthy reconstruction does not activate a corruption/recovery-unavailable fence. The new backend map has no P/A state; the valid pruned history has no P/A row; the healthy cron table still has P only. [F03:135–151; F08:79–95,264–302; F09:1666–1720; F11] |
| 6 | Redeliver the still-unacknowledged A with the same valid event. Preparation performs checked history lookup and obtains real absence. Because this is an event, the legacy covered schedule-slot refusal does not apply. Current definition and event checks pass. [F02:55–145] |
| 7 | Runtime's precheck and authoritative `begin_fresh_run` also see no retained A owner. They admit a new history row whose primary ID is A. The cron insert for A succeeds because the original cron key contains P, not A. [F07:885–900; F01:251–296; F10:2348–2378] |
| 8 | The existing executor is reached a second time. The shipping Runner gets a fresh stream/nonce, and a non-idempotent operation may happen again. A fixture append/atomic counter in the real admitted executor gives a causal duplicate-effect oracle. [F14:6092–6154] |

The source already establishes each transition. The proposed oracle must execute them together in Actions before the batch is called verified. The twenty-run pressure must be produced by the real manual Runtime, not by injecting a truncated history file. Reconstruction must reopen existing owners without create/upsert/admission seeding.

Why choose the alias? If the primary P alone is redelivered after its terminal history was evicted, its existing terminal cron row rejects a fresh cron admission, so that is not the clean duplicate-executor counterexample. The alias has no corresponding row, which makes the causal bypass definite. The primary case is related behavior of the same missing receipt relationship; it is not a second finding in this report.

Why earlier fixes are still correct: checked reads prevent corruption or display truncation from masquerading as absence, but cannot recover a row that the normal writer really removed. Fresh-ID admission checks all **retained** IDs, but does not create eternal tombstones. Running/unknown tool custody is protected and remains Abandon. The legacy timestamp guard intentionally holds an ambiguous schedule slot, but does not apply to a valid event. These are complementary safeguards, and none should be weakened to fix this case. [S04:723–900,924–1027,1029–1145,1288–1475]

## 5. Execution settlement, completion and ACK are distinct

The word “completed” refers to different responsibilities here. The table states what the current client actually does; it does not promote the deployed backend's unversioned response semantics into a proven contract.

| Observed fact | Current client meaning | Custody that must still be retained |
| --- | --- | --- |
| Runner publishes a safe terminal and cron/history writes succeed | Local execution has settled; exact terminal history can prevent another execution | Backend completion/ACK may still be owed for P and each alias independently |
| Tool effect pending/unknown or custody companion unavailable | Execution settlement is not safe | Existing Running row and aliases remain; None/Abandon, no synthesized terminal/ACK |
| `/sand/automation-runs/complete` returns transport `Ok(Value)` | `Completed → Reported` for that one fire ID | Delivery ACK has not yet been established; Reported is still retained |
| Completion returns 401, 404, 409, other status failure, decode failure, timeout or lost response | Completed remains pending; direct `complete` propagates the error; retry retains the same completion | Both exact local execution receipt and remote obligation remain needed; a lost response can conceal server acceptance |
| ACK-bearing poll fails or its response is lost | The retain/remove block does not run | Reported remains; sending ACK is not by itself local retirement evidence |
| Successful poll still returns a Reported ID | The map retains it and suppresses redispatch | Client continues to send the ACK on later polls |
| Successful poll omits a Reported ID | Current client removes that ID from memory | This is the client's discharge observation; no persisted per-ID discharge is written today |
| Only P has been discharged, A remains pending | P's remote delivery can retire independently | The one execution row must still preserve A's ownership until A's outstanding obligation is resolved |

Sources: F03:187–204,263–307,332–364; F06:607–636; F14:6015–6049; S05:12–22. The transport reports success only for a successful HTTP status with a JSON-decodable body; an empty 204 fails JSON decoding. `report_completion` does not validate an application receipt. The poll parser currently defaults a missing/non-array `events` member to an empty list. Thus “successful poll omits an ID” above precisely describes current client behavior; it is **not** proof of an authenticated, typed deployed ACK receipt. [F03:270–291,351–364; F15:61–101; S05:22,33–38]

The independent Native adapter has explicit native protocol, account/issuer identity and receipt validation, but shipping `ProductionAutomationsLifecycle` instantiates the Reqwest Sand adapter. Its unused stronger contract cannot supply evidence for this path. Historical reconstructed server dossiers likewise are not current deployed authority. This report does not infer that 404/409 are idempotent success, nor that the absence of a fire from an arbitrary poll establishes ACK. [F04:136–139; F16:1–119; S06:43–46]

### Every actual completion callback path

| Caller / registration | What it invokes |
| --- | --- |
| Main startup, F14:17553–17556 | Registers the production `fire_dispatch` closure that calls `UnifiedGatewayApi::dispatch_production_backend_fire` |
| Production backend worker, F08:180–194 | Sole production creator of the `Arc<dyn Fn(Option<FireCompletion>)>` callback; captures original `event_id` and the actual fire-state owner |
| Scheduled worker, F14:3931–3947 | Calls `completion(automation_fire_completion(result))` after the actual scheduled Runtime attempt |
| Event worker, F14:3983–3992 | Waits on `enqueue_event_automation_fire_with`, then calls the same mapping; coalescer's one result wakes every original item, so each original backend ID gets its own callback |
| Checked immediate branch, F14:5897–5902 | Calls `completion(Some(terminal))` for exact stored terminal replay and existing known policy-completion branches, without executor entry |
| Callback Some branch, F08:188–190 | Calls actual `AutomationFireBackendRuntime::complete`; the callback ignores the returned error, while the consumer retains Completed for retry |
| Callback None branch, F08:192 | Calls `abandon`; removes only the volatile claim. Existing Running history/cron remains the later replay fence |
| Preparation/launch refusal, F14:5904–5908 and F08:195–200 | Returns false with no completion callback; production backend abandons the volatile claim |
| Later poll, F03:307,332–348 | Calls `retry_completions` directly on retained Completed entries; it does not run an application completion callback again |

Repository searches for `FireDispatch`, direct `completion(automation_fire_completion...)`, and background Runtime callers were used to cross-check this inventory. Test callbacks were separated from production callbacks. The local relay and manual/workflow Run Now paths above do not own a backend completion callback.

The existing mapper also deliberately converts post-admission outer `Err` to a failed completion, while typed AdmissionRefused and `Outcome(None)` map to None. A repair must not globally turn errors into None or require every legacy immediate/policy/error callback to manufacture a durable terminal receipt. ACK05 and BA distinguish those cases already. [F14:5914–5931; S04:129–237,1147–1267]

## 6. Fixed-U comparison and dependency depth

The relevant current-U implementation was read directly rather than treating an old dossier as current evidence. U's cron run receipt is an execution owner with job/store/config revision/agent/process identity. Admission claims through an existing SQL owner and unique conflict handling; finishing updates the exact active receipt. A late execution settlement can retain status Running after a recorded timeout. Terminal pruning retains a bounded 64 receipts and prioritizes the pending receipt supplied by the owner. It does not prescribe eternal retention of every historical terminal. [U01:26–67,81–129; U02; U03:26,76–164,207–241]

The timer finalizer sends terminal state through `cron.finalizeRuns`; execution still settling is recorded separately, and a late settlement uses receipt-only finalization. Its lost-reply recovery explicitly refreshes views without replaying the run. Queue execution rechecks the live receipt/job/owner conditions before launch. This supports reviewing receipt identity, publication, and remaining obligations as distinct from executing again. It does not define Fabushi Sand completion/ACK wire behavior. [U04:47–259; U05:30–215; U06]

The immediate dependencies reviewed for this bounded comparison were the receipt row decoder/current-read snapshot, receipt write-admission requirements, and the relevant SQL query/transaction/publication APIs. The write-admission module requires an owning write transaction. Transaction settlement accounts for the possibility that COMMIT succeeded before a later failure and keeps conservative fencing. Query publication APIs require the owning publication scope. These are bounded dependency-contract reads, not a claim that all of SQLite, Kysely, PostgreSQL, worker publication, process recovery, or the dependency/legal closure has been deeply reviewed. [U07; U08; U09:349–370,421–498,558–623; U10:282–325]

No upstream implementation is proposed as a second Fabushi owner or runtime dependency. The analogous Rust responsibility belongs in existing AutomationRuntime/FileAutomationStore/Session/backend composition. Full U coverage, remaining source/dependency dispositions, legal records, platform delivery and OCRM acceptance remain open.

## 7. Spec gap that must be repaired before code

There is an explicit durable-spec gap, not permission to silently broaden the earlier repair. The current cron spec says evicted terminal receipts remain open. ARH02/03 intentionally protects Running and the just-settled row; its capacity rules do not include terminal remote obligations. The completion-safety spec explicitly says pending state remains in memory and restart recovery/account binding/retention/server idempotency remain unresolved; its original scope excludes a new persistent writer/schema. [S04:944–984,1023–1027; S05:18–22]

Before product code or tests change, the integrator should repair the existing applicable spec and current task under the same authority, with a narrow new requirement set. Necessary decisions are:

1. **Protected obligation.** For a newly admitted backend execution, its exact primary and coalesced delivery identities remain execution-owned while any completion or delivery ACK remains unresolved. Safe local terminal status alone is not eviction authority. Known Running/unavailable history still abandons.
2. **Settlement evidence.** Define separately safe local terminal, completion report success, and ACK discharge. Use the actual deployed protocol or explicitly retain a scoped client-contract limitation. 404/409/401/lost responses cannot become success. Do not release on merely sending a request or on a poll that did not carry the corresponding ACK.
3. **Per-delivery versus per-execution identity.** One execution receipt can own several backend fire IDs. Discharging one ID cannot release another alias. Namespace/agent/local automation/backend automation and any existing account-generation authority must remain bound; an unqualified global UUID is not sufficient for cross-account persistence.
4. **Publication order.** Register the obligation before executor entry and preserve it across terminal history writes. Define the completion-payload publication and the reported/ACK transitions, including failure of each local write after a remote response.
5. **Bounded retention/admission.** Choose an explicit protected-capacity rule and refusal behavior. Preserve display limit 20, existing valid legacy excess drain, non-growing settlement and ordinary independent work when one old run is held. Do not quietly turn the history into an unlimited outbox or evict an outstanding obligation to make room.
6. **Recovery.** Distinguish no-ACK completion retry, reported-but-not-ACKed recovery, and ACK-success/local-clear-unknown. State exactly which work the existing owner restores at startup and which ambiguity blocks. Never reconstruct “not executed” from absence of an erased alias.
7. **Compatibility and rollback.** Specify new-format validation/preservation, old rows with no custody field, and old binaries that discard unknown fields. Already-evicted aliases cannot be reconstructed by this change. No transparent rollback promise while protected obligations remain unless an enforceable guard or compatible writer is supplied.
8. **Preserved behavior and evidence.** Keep existing ACK05/BA error mapping, immediate policy completions, Running/unknown-effect/legacy-slot refusal, terminal positive controls, checked corrupt-input refusal, and existing queue/clock semantics. Bind the new causal oracle and exact-H Actions selection before a verified claim.

This is a repair to the current owner contract, not a new guarantee of deduplication forever after a confirmed ACK. Once a delivery's obligation is actually discharged, normal bounded terminal retention may apply. Historical arbitrary replay after that point remains outside this proposed batch.

## 8. Minimal persistence candidates in existing owners

The smallest reasonable storage location is the canonical execution history row in `FileAutomationStore`, reached through `AutomationRuntime` and the existing `SandAgentSessionStore::with_admitted_filesystem_mutation` boundary. Current code already uses that boundary for begin and finish; the new writes must reuse it and must not reacquire the same non-reentrant Session lock through a public facade inside the closure. The RunPath remains the execution owner and the backend runtime remains the sole outbound completion/ACK consumer. [F07:348–384,632–648,1272–1291; F19:902–915]

### Candidate A — minimum safety pin, with its recovery limit stated

Add checked, versioned per-backend-delivery ownership metadata to the existing run row and publish it in the same fresh history write that creates the Running row. Propagate the explicit backend origin through the existing args/batch path; manual runs and local listener events must not acquire fictitious backend obligations just because all local runs have an ID. Protect any row with unresolved backend IDs in `retain_run_history`, in addition to all Running rows and the just-settled target. All current writers must preserve the metadata. [F01:204–296,315–324,488–512; F05; F06; F07]

Clear only the exact acknowledged ID after the existing backend consumer observes its ACK-discharge transition and the existing Session-authorized mutation commits. Pass a typed local receipt binding back to that owner; the backend transport must not open or write history files directly. No new scheduler, database, independent journal service or cron execution table is needed.

A compatible bounded policy candidate is to count terminal rows with unresolved deliveries in the existing protected-capacity limit, sharing the current 20-slot admission rule. One held row still permits later manual runs; when all protected slots are used, new execution refuses before its side effect rather than deleting custody. Legacy excess continues to drain without growth. This is a candidate requiring explicit spec adoption, because it changes what counts toward admission capacity.

**Limit:** a pin plus a release callback alone does not close outbound restart recovery. If the backend accepts ACK and the process stops before the local clear commits, a pinned row can remain even if the server no longer redelivers it. Loading only on redelivery will not resolve that case. Candidate A can close the identified duplicate-execution safety gap but must not be sold as complete completion/ACK liveness or crash recovery.

### Candidate B — smallest coherent durable settlement extension

Use the same row and owners as Candidate A, and retain the minimal per-ID outbound phase needed by the existing consumer: completion owed, then reported/ACK owed, then discharged. Persist a frozen, bounded completion payload or otherwise define an exactly reproducible payload for the stored safe terminal before the phase is consumed. No secrets belong in the record. A source-account/generation binding must use the existing account owner, not a newly invented identity store.

The conceptual durable facts are: existing execution primary ID; exact coalesced delivery IDs; explicit backend origin and existing identity namespace; unresolved per-ID phase; and the safe terminal completion value when available. These are design facts, **not an approved schema or API spelling**. The precise contract must be frozen in the existing spec before coding.

On owner reconstruction, use the existing Automation/Session owner to enumerate/read checked outstanding records and seed the existing backend consumer with completion/ACK work. It must not call RunPath to execute those terminals again. Recovered completion-owed work reports the same result; recovered reported work retries ACK. Unknown source-account binding blocks that outbound use. This avoids a second recovery loop or independent source of truth.

The order should be reviewed explicitly:

- Fresh admission commits the ownership marker before side effects. Safe terminal settlement preserves it in the same existing row.
- For a tracked safe terminal, completion success is durably reflected before permitting its ACK. If that local write fails after remote success, retain unresolved custody and retry/reconcile the same delivery; do not execute again.
- After a successful ACK-bearing poll's discharge observation, persist release of that exact ID before forgetting the corresponding locally tracked obligation. If the release write fails, retain reported/ACK-owed work.
- A crash after remote ACK but before local release reopens as reported/ACK-owed. Repeating ACK must be justified by the actual protocol, or recovery must remain explicitly blocked. Client mocks cannot establish server idempotency.

This extension is more complete than a pin-only fix, but requires a checked startup enumeration, identity binding and a versioned interpretation of ACK success. Those are material design dependencies, not details to hide inside a utility. This investigation has not deeply reviewed the auth-generation owner at H or established the deployed Sand contract; the integrator must resolve those affected dependencies before implementing this broader candidate. The same-account source defect above does not depend on their resolution.

### Preserve the existing callback distinctions

The current callback surface handles more than safely settled executions: known missing/disabled/mismatched-definition policy outcomes, stored legacy terminal replay, and post-admission outer failures also reach Some completion. Candidate B must carry or resolve a typed durable execution receipt for the newly tracked safe-terminal path; it must not impose an invented durable terminal on every callback. In particular, do not replace `Err → failed completion` with a blanket None, and do not manufacture `ok` history to satisfy the new persistence adapter. Existing ACK05 errors and Running/unknown-custody holds require separate explicit treatment in the repaired spec. [F14:5897–5931; S04:129–237,1147–1267]

For the concrete successful-terminal counterexample, the payload is the stable succeeded completion, so no ambiguity about generic versus detailed failure text is required. A general terminal implementation must also account for the existing initial callback's generic failure/interruption text versus terminal history replay's stored detail. Claiming byte-identical durable completion without deciding that distinction would be unsupported. [F14:5918–5930; F02:162–197]

### Why not the superficially smaller alternatives?

| Alternative | Reason not to select it as this repair |
| --- | --- |
| Retain every terminal forever | Unbounded behavior and a new guarantee not authorized by the current contract or U's bounded retention |
| Use the in-memory backend map to protect history | Reconstruction loses that map, exactly the gap being investigated; file writers also need a durable ownership fact |
| Store a disconnected sidecar/outbox with its own writer | Creates a second custody truth and adds cross-file publication/recovery problems; existing row/Session writer can carry the fact |
| Insert a cron row for every alias only | Cron is a reaper projection, not the existing backend receipt owner. It does not preserve the original terminal payload or remote phase and changes schema/owner meaning without closing settlement |
| Replay or complete known Running after restart | Violates the already correct uncertain-effect custody requirements; there is no safe terminal authority |
| Treat 404/409 or a sent ACK as discharge | Reintroduces the exact unsafe inference removed by the completion-safety contract |
| Stop backend event coalescing | Changes valid product batching behavior, still leaves terminal receipt/completion recovery incomplete, and does not repair the ownership relationship |

### Likely bounded implementation surfaces, after spec repair

| Existing path | Bounded responsibility |
| --- | --- |
| `source/host/src/automations/automation.rs` | Internal typed run custody representation, retaining existing display shape/limits unless explicitly changed |
| `source/host/src/automations/automation_store.rs` | Strict parsing, all-writer preservation, admission/retention and exact per-ID phase/release mutations in the same run history |
| `source/host/src/extensions/transcript/automation_run_path.rs` | Publish backend ownership with fresh admission; preserve it through actual safe execution settlement without changing unknown/cancel semantics |
| `source/host/src/extensions/transcript/automation_runtime.rs` | Session-authorized custody methods, origin propagation and checked recovery/read surface; no second Session or scheduler |
| `source/host/src/extensions/transcript/automation_event_fires.rs` | Preserve explicit backend provenance and all original delivery identities through one real batch if the selected interface requires it |
| `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | Existing completion/ACK phase consumer and per-ID durable transition integration |
| `source/host/src/extensions/automations/production.rs` | Existing callbacks, durable phase/release adapter and owner reconstruction wiring |
| `source/host/app/src/main.rs` | Actual startup and all three callback-producing branches; typed association to the existing AutomationRuntime |
| `production_lifecycle.rs`, `host_production_extensions.rs`, `session/agent_session.rs` | Only if the frozen interface requires passing the existing owner/identity bridge; narrow facade/wiring changes, not new owners |
| Existing store/backend/production-wiring/App test files | Causal oracle and retained controls; no unrelated harness or new workflow required |

No Runner business change or AgentDb schema change appears necessary for the narrow receipt-retention repair. If the design cannot remain inside these existing owners without a parallel authority, revisit the spec/owner decision instead of expanding silently. This is a candidate path budget, not an authorized patch list.

## 9. Crash, cancellation, unknown-result and rollback analysis

| Boundary | Required behavior / remaining limitation |
| --- | --- |
| Before fresh durable claim | No executor entry; typed AdmissionRefused remains None and later preparation remains checked |
| After claim, before safe terminal | Existing Running ownership blocks replay. Shutdown, requested cancellation, deadline expiry, missing terminal or unknown tool effects do not authorize completion or re-execution |
| Safe terminal committed, before callback/report | Protected terminal must survive ordinary history pressure. Reconstruction may consume only the stored safe result; no new execution |
| Report request accepted remotely, response lost | Completion remains unresolved locally. Retain exact identity and payload; no ACK based on the lost response |
| Completion Ok, durable reported-phase write fails | Candidate B retains unresolved work and forbids releasing the execution receipt; idempotent report/reconciliation needs actual protocol evidence |
| Reported persisted, before ACK | Reopen the same existing backend consumer as ACK-owed; preserve all remaining aliases independently |
| ACK sent, poll response lost | Keep ACK-owed. Do not infer remote non-acceptance or reopen execution |
| ACK success observed, local release fails/crashes | Keep/reconstruct ACK-owed; repeat/reconcile under the real protocol. A redelivery-only pin design cannot guarantee eventual local discharge |
| History/identity unreadable during any recovery decision | Existing unavailable-history Abandon/blocked state; no empty-history fallback and no fabricated terminal |
| One alias discharged, another unresolved | Keep the one execution row for the unresolved alias; do not release the whole batch on primary success |
| All obligations discharged | Normal bounded retention may resume. No guarantee of arbitrary historical replay protection after discharge is introduced |

For cancellation, current source already waits for the actual typed terminal and checks tool-effect custody. The existing integration test demonstrates that the routed local worker/lease can retire while external custody remains unknown; automation history/cron remain unchanged and the reopened backend still abandons. That distinction is a required control, not a bug to replace with automatic retry. [F14:5970–6049; T04:375–491]

The current file writer uses a temporary file followed by rename, with a remove-and-rename fallback; it does not show fsync of file/directory. Reusing it provides the current application's filesystem-write contract, not proof of power-loss durability or universal atomic replacement on every platform. The deterministic new oracle should first establish healthy completed-write owner reconstruction. Any stronger abrupt-process-stop/write-interruption or power-loss claim requires corresponding source contract and Actions/platform evidence; do not label a graceful reopen test as those guarantees. [F17:154–177]

**Rollback is not automatically safe.** At H, `AutomationRun` and `run_to_json` only preserve their known fields. An older binary would ignore added custody metadata and rewrite history without it; old retention would then evict a terminal again. A new optional JSON field alone is not backward-compatible custody. The new spec must choose a supported downgrade policy: compatible old writer support, an enforceable existing-format/version admission guard, or rollback only after protected obligations are reconciled and the data is explicitly made compatible. An operator instruction without an enforceable product condition is not a demonstrated rollback guarantee. No particular migration/guard has been implemented here. [F18:16; F01:511–512]

Existing rows without metadata cannot be labeled “already ACKed” from status alone. Conversely, marking every historical terminal permanently pending would invent work and unbounded retention. The smallest prospective scope is newly admitted backend deliveries after the upgraded publication contract. Handling still-present legacy terminal receipts on upgrade needs an explicit compatibility decision; already-erased aliases are irrecoverable from these stores. Account changes, definition deletion and old-client coexistence also need explicit decisions before a generalized durable settlement feature is accepted.

## 10. Causal meaningful oracle, using existing production test homes

No test has been added or run in this investigation. The following is the proposed acceptance oracle for the bounded behavior, not a report of results.

The existing `real_retained_run_survives_twenty_manual_settlements_and_reopened_backend_redelivery` already has the correct real-owner scaffolding: actual production backend, real coalescer scheduling gate, real Runtime/Session/file/cron owners, manual executions, and reconstruction without reseeding. It deliberately returns `RetainedUnsettled`, so the original row stays Running. Its successful retention assertion is valid and must remain. The new case must be a separate case with safe terminal settlement and a failed remote completion. [T03:1350–1625]

The existing App checked-backend harness also forwards the **actual** production callback and uses the same shipping checked-dispatch helper. Its current controlled launch handles Schedule; extend the relevant existing test composition to Event if that is the chosen oracle home. Do not present a helper with a hand-coded completion/ACK policy as the production consumer. [T04:1011–1071,1115–1163]

### Required failing-before / passing-after sequence

1. Create a valid enabled GitHub event automation and real healthy Session/Automation owners. Start the actual production backend against a controlled, recorded transport. Readiness and reply availability are controlled; no credentials or external notifications are needed.
2. Hold an actual coalescer batch as the existing test does, then enqueue backend fires P and A through the shipping checked-dispatch boundary. Observe both actual enqueues before releasing the gate. Require one target batch, P primary, A alias, and zero target execution before release. Do not use a sleep to hope that the items coalesce.
3. Enter the real RunPath once. Record the admitted history and cron row before the controlled effect. Append one line to a fixture-owned effect receipt (and count executor entries), then return a safely settled typed result. If the App typed terminal adapter is in scope, use the existing real terminal publication/waiter path with a known-empty custody report rather than fabricating a missing companion as success.
4. Allow the actual completion callbacks. Force A's completion to stay in a specified unresolved transport response. Record exact POST bodies and ACK arrays. Assert one execution/effect and no A ACK. A partial-settlement variant may allow P to report/ACK while A remains pending.
5. Run twenty later real manual Runtime executions of that same automation, each with a safe terminal, separate from the original effect count. Capture resulting file bytes, checked primary/alias reads and cron state. Before the repair, source predicts P/A will be absent from history; after the repair, exact pending custody must remain. Manual calls must actually enter and settle; do not mutate history directly to simulate pressure.
6. Close the backend, dispose AutomationRuntime, shut down Session workers, and reconstruct from the same data without create_session/upsert/begin/cron seeding. Create a fresh production backend consumer. Transport redelivers **only unacknowledged A** with the same valid event/identity.
7. Assert A causes no new executor/effect, no new alias-primary cron row and no new execution observation. It consumes the preserved terminal receipt and offers the original successful completion as appropriate. Repeated redelivery/reconstruction must preserve that result until real obligation settlement.
8. Permit completion success, then perform ACK-bearing poll(s) with precise expected ID sets. Verify per-ID persisted phase/release according to the selected spec and that eventual all-ACKed history becomes ordinarily evictable. Include the crash window between observed ACK and local clear if Candidate B claims closure there.
9. Preserve a distinct valid new event/manual run positive so the fix cannot pass by disabling execution globally. Keep the original Running/unknown-custody oracle unchanged.

Evidence should include the actual batch, post-admission and post-terminal history/cron snapshots, preserved bytes at refusal boundaries, real callback counts, exact transport request bodies, effect receipt, execution observations, reconstructed-owner snapshot and the exact test binary/command subject on H. The oracle should fail if it never admitted the original batch, if the executor was never reachable, if the failed response was not actually consumed, if manual pressure never occurred, or if reconstruction accidentally reused the old consumer map. These causal assertions distinguish the intended state from a timeout or admission failure that merely happens to produce zero replay.

Additional bounded controls for the selected metadata design should cover malformed/ambiguous new fields, missing/failed persistence, exact alias release, protected-capacity refusal, all-ACKed eviction, same-identity restoration and account/owner refusal. Use controlled barriers/owned channels and existing bounded harnesses; all execution belongs in the existing Actions workflow on the frozen implementation H. Mock service behavior cannot close deployed service acceptance. [S03:702,710–716; S04:1014–1021]

## 11. Valid assertions to retain

The following are retained requirements and source-level oracle assets. “Present/read” below does not mean “passed during this investigation.”

| Existing assertion / source | Why it remains valid |
| --- | --- |
| `fire_runtime_never_acks_unconfirmed_completion_or_redispatches_redelivery`, T02:468–522 | 401/404/409/lost-response completion failures retain Completed, do not ACK, and do not redispatch in the same runtime; eventual true success enables ACK |
| `explicitly_settled_history_replays_terminal_success_or_failure_after_reopen`, T02:1003–1077 | Still-present exact primary/alias terminals replay success/failure without new execution or mutated history/cron; changing definition policy does not override the original receipt |
| `begin_keeps_running_identity_and_aliases_across_settled_history_eviction`, T01:942–967 | Running ownership and aliases survive later terminal pressure, display/history remain bounded under the existing policy |
| `finish_preserves_valid_legacy_normalization_order_and_projection`, T01:901–940 | Valid legacy normalization/projection cannot be silently removed while adding new custody-field preservation |
| Actual retained coalescer/no-follow-up and twenty-manual/reopen cases, T03:1342–1625 | Unknown external effects keep Running history/cron exactly intact; no completion/ACK or new executor on original/alias replay |
| App typed tool-custody test, T04:200–494 (relevant execution/reopen ranges read) | Lease/worker retirement does not prove effect settlement; actual typed terminal can still require retention after local cancellation |
| App checked discovery terminal/new-fire positives, T04:1498–1603 | Exact terminal success/failure, genuine checked missing targets, repaired unrelated valid targets and distinct valid fresh execution still work |
| App harness callbacks/traffic assertions, T04:1011–1185 | Refusal has no completion/ACK; Some callbacks go through the real consumer; correct terminal replay performs zero executions |
| Cron contract ACK/AHA/ARA/AC/DC/AUTO-CUSTODY/ARH/BH/BA/FC/ALS, S04 | Strict owner admission and recovery fences, no false normalization-derived success, mandatory fresh identity, safe cancellation/deadline behavior, Running/unavailable/legacy-slot Abandon, typed admission result and post-admission error distinctions all remain |

Additional source searches located the existing strict fresh-history, duplicate primary/alias, malformed-tail, duplicate JSON member, legacy excess/capacity, finish-missing/write-failure and definition-custody tests. Their existence was not converted into a full-body read or execution claim where only searched. The new batch should retain their actual assertions, not rewrite them as generic “no panic” or all-error-to-None checks.

## 12. Source identity and read ledger for this investigation

This is a report-local evidence index, not a replacement project ledger. All target citations F/T/S refer to H above. A citation such as `F01:488–503` means those lines of that exact H path/blob. Target permalinks can be formed as `https://github.com/bhrumom/fabushi-desktop/blob/bb5a7f6098f952df6d388a672c4fb01045bc645c/<path>#L<n>`. Upstream citations similarly refer only to fixed U. Full content identity is recorded here so the next phase can independently re-read the authoritative source.

### Authority and contracts

| Ref | Path | Blob | Read extent / classification |
| --- | --- | --- | --- |
| S01 | `AGENTS.md` | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` | Root instructions reviewed, including 1–115 and subsequent policy/authority sections through end; current applicable instruction read, not product verification |
| S02 | `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` | Current authority 1–85 and 453–510 plus heading search; older historical body not claimed full-read |
| S03 | `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` | Full §17 671–720; relevant ranges 170–198,347–356,371–395,414–427; not whole-spec full-read |
| S04 | `projects/openclaw-rust/docs/specs/cron-session-reaper.md` | `b0a62da2cc86b5479047458019bbb3adfc2b16d1` | Full 1–1475, including all appended custody/settlement requirements |
| S05 | `docs/specs/sand-completion-ack-safety.md` | `cbe3ceeb072ee9756235ac926f0c09021448ab8c` | Full 1–60 |
| S06 | `docs/specs/fabushi-cursor-service-independence.md` | `b857b20ee4fbf92610caac7ae8b19034e559f53f` | Bounded 31–55, especially shipping/native adapter status 43–46; not full native-service review |
| S07 | `projects/openclaw-rust/docs/specs/agent-loop-runner.md` | `254d7c375ceb7dab3ec008aa7b859ea395e04745` | Full 1–190; historical execution evidence remains historical |
| S08 | `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md` | `1f0ecfb239f840df5e2ce5237029a78d945b4b9e` | Current task and recent batch summaries 1–381; historical remainder not full-read |
| S09 | `projects/openclaw-rust/module-map.md` | `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` | Current navigation 1–64 and M19 row at 80; not all module responsibilities audited |

### Current production code

| Ref | Path | Blob | Read extent |
| --- | --- | --- | --- |
| F01 | `source/host/src/automations/automation_store.rs` | `57f39e545e525a36de3c7e9830970623862ee1fd` | Full 1–515, read in ranges; strict parser/legacy parser/retention/write consumer all reviewed |
| F02 | `source/host/src/extensions/automations/fire_delivery.rs` | `17a4466130d516d132db59f2ded772bead34935b` | Full 1–541 |
| F03 | `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | `7d5f3e50f3a20aa5f142fdfa029cdcc6d40e06c9` | Full 1–366 |
| F04 | `source/host/src/extensions/automations/production_lifecycle.rs` | `acc5d67aef35ede93a0f87e317f966013ee2eb8b` | 110–250, transport/startup/lifecycle composition; not full file |
| F05 | `source/host/src/extensions/transcript/automation_event_fires.rs` | `0cab169c66da397a9bb2777995c67178461edc43` | Full 1–337 |
| F06 | `source/host/src/extensions/transcript/automation_run_path.rs` | `a80ca919c1d05c30be29ad60e109f3a54f324e19` | 1–861; remaining prompt helpers not claimed read |
| F07 | `source/host/src/extensions/transcript/automation_runtime.rs` | `044f53c8eb94924168cc7dd953f3732cd70fa668` | 301–494,571–654,773–1085,1152–1185,1268–1310, with key custody ranges reread |
| F08 | `source/host/src/extensions/automations/production.rs` | `3927e9736bebd27302cd9c4c2139b583e80b7aa4` | Full 1–310 |
| F09 | `source/host/src/extensions/session/production.rs` | `f585fae13720e08f34323bed82759253590a3753` | 1648–1738,2928–2990,3083–3096 plus relevant caller searches; not full Session recovery audit |
| F10 | `source/host/src/extensions/session/agent_db.rs` | `78740768f7f1b0307dfd6756c4d50c368a91b721` | 2318–2385,2581–2650; cron admission/list projection and relevant searches, not whole DB |
| F11 | `source/host/src/extensions/session/agent_db_schema.rs` | `279583c0e0ccb37a7188bfa29b5467f8a1e24b3e` | 1–58, exact cron schema/index |
| F12 | `source/host/src/extensions/session/extension.rs` | `f34b1d871a4d635e0f83b7103a0bbe1150388c4b` | 1–130 startup/maintenance boundary |
| F13 | `source/host/src/host_production_extensions.rs` | `36b5dbac27d346f883150df179372034e0bea223` | 1260–1335, actual Automations start/stop composition |
| F14 | `source/host/app/src/main.rs` | `4831ae8cf002ce64e6409f2d695b48aaa1914c45` | 3820–4022,5839–6223,13944–14028,17520–17635 and exact caller searches; not full app or Runner implementation read |
| F15 | `source/host/src/extensions/automations/backend_transport.rs` | `6dc4477c72a38179d71ab33013c9b7e2c3b89991` | Full 1–101 |
| F16 | `source/host/src/extensions/automations/native_backend_transport.rs` | `9e7a811165610e81719ae26fcf4513a75088074c` | Full 1–119; separate unused stronger adapter, not shipping authority |
| F17 | `source/host/src/watched_directory.rs` | `eb7bb0106966e9a8b5adb7205667ff81678dce80` | 1–55,125–205, especially write publication contract 154–177 |
| F18 | `source/host/src/automations/automation.rs` | `4b27d9f5a7c4174a26e1244c50bef683d2c01482` | Full 1–87 |
| F19 | `source/host/src/extensions/session/agent_session.rs` | `0a54da8349bc3edc9ef21c5f1c1862e13a9e3039` | 875–935, exact external history mutation facade |
| F20 | `source/host/src/extensions/automations/sand_automation_cloud_sync.rs` | `184e4398e5fc0f2dbe3ac7556ba7d03c0c57877e` | 98–120, schedulability predicate only |

### Existing oracle source

| Ref | Path | Blob | Actual read / limit |
| --- | --- | --- | --- |
| T01 | `source/host/tests/automation_store_contract.rs` | `81720b74a8db66bbcd90dbb19c7fcd624599089b` | 901–967 full relevant test bodies; function-name/custody searches elsewhere. No whole-suite/full-body claim |
| T02 | `source/host/tests/automations_backend_contract.rs` | `45174d8010a61d631bd4d605ecc1df515a619bbc` | 468–522,1003–1077 full relevant tests; other matching test names searched |
| T03 | `source/host/tests/automation_run_production_wiring_contract.rs` | `fcf496dd0dd8c20810d064b98a31c67a310b82ad` | 1140–1275,1350–1635; actual coalescer, manual pressure/reopen oracle and callback helper read; remaining matches searched only |
| T04 | `source/host/app/src/tests/automation_terminal_custody_tests.rs` | `391c685d143e7e38b66b54e5b60ef104201d0f12` | 200–270,375–505,900–1185,1490–1605 and relevant searches; unknown-custody and backend callback/terminal-positive ranges, not whole file |

### Fixed-U directly read implementation and dependency contracts

All eight cron blobs below were fetched at U and read fully. Tree membership/blob IDs were checked through the exact U root tree and its `src`, `cron`, `service`/`store` descendants. The connector's Contents endpoint returned source bodies rather than metadata, so those bodies were not misreported as SHA metadata; exact Git tree entries supplied the IDs.

| Ref | Fixed-U path | Blob | Review |
| --- | --- | --- | --- |
| U01 | `src/cron/service/run-receipts.ts` | `6e9e5216c04e6511d5715b3eeb9ec7b606149148` | Full 156 lines; execution identity binding / owner / terminal mapping |
| U02 | `src/cron/store/run-receipt.types.ts` | `8b1ad063d6ca57dd758dc9b632ca6ed76ffacb9c` | Full 64 lines; durable identity/status/current-read contract |
| U03 | `src/cron/store/run-receipt-store.ts` | `1a79e5505cc4aaca67ea797885fe2058193a8fbc` | Full 241 lines; claim/finish/retention/schema dependency |
| U04 | `src/cron/service/run-queue-execution.ts` | `42045bf23ec71b0bd970f03acf113011be4ddd83` | Full 259 lines; live admission/execution/finalization chain |
| U05 | `src/cron/service/timer-outcome-finalization.ts` | `a346d651c99c6a74228869bc218e4e6bb87f16bf` | Full 215 lines; commit response, late settlement, lost-reply recovery |
| U06 | `src/cron/service/run-history.ts` | `586db842216dd537f3dcafeb19e11573ab9524d5` | Full 116 lines; exact run history consumer |
| U07 | `src/cron/store/run-receipt-read.ts` | `3a851f9480f8f342fb26a28c8ce8b03509e40e49` | Full 138 lines; row decoder/status and shared current-facts snapshot |
| U08 | `src/cron/store/run-receipt-write-admission.ts` | `9bbfc93987aa9cc82b045703ba8e1b433527a3bd` | Full 20 lines; owning transaction/schema admission contract |
| U09 | `src/infra/sqlite-transaction.ts` | `c87a3b05d3e569cd54a2318db2045b357cce8ae1` | Contract review 1–80,349–379,421–510,558–623; 764-line module not full-read |
| U10 | `src/infra/kysely-sync.ts` | `056d7368bc59b9c000ee374a2c29a40e15202c18` | Contract review 1–87,140–230,282–326; 404-line module not full-read |

The exact root tree also identifies U `package.json` blob `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657` and `pnpm-lock.yaml` blob `83b4975eb96942ec007a2eb9847ccfe98d380bda`. Their bodies/transitive licenses/build patches were **not** reviewed in this investigation and no dependency-closure claim is made. They are recorded only to prevent accidental use of a floating dependency baseline. No dependency addition is proposed.

The previous `cron-automations-heartbeat-scoped.md` dossier's 249–320 ranges were consulted for history/navigation and independently checked against current fixed-U source where used. Historical independent reconstructed-backend references were encountered in searches but not promoted to deployed behavior. No other agent's private current review report was accessed.

## 13. Work performed and remaining gates

Only read-only Git/source/API inspection and this report write were performed. No production, spec, test, task, ledger, workflow, inventory or index file was changed. No builds, tests, lints, generators, schema validators, helper validation scripts, local/htch execution, pushes or workflow dispatches were run. No subagents were spawned.

The new finding is source-confirmed, not executed or accepted. The next step belongs to the integrator: independently re-read the relevant current contracts/source, decide the narrow phase/retention/recovery contract, repair the durable spec, and only then implement the selected existing-owner batch with its real causal oracle and exact-H Actions evidence. If the broader durable settlement candidate depends on unreviewed identity or unavailable deployed protocol facts, keep those portions blocked rather than claiming that a pin-only change closes them.

All full OCRM responsibilities, source/dependency closure, actual production wiring coverage beyond this narrow chain, UI/ecosystem/provider/channel/node/platform acceptance, data migration/upgrade/rollback evidence, final AC01–AC24/G01–G12 and independent acceptance remain open. None of the eight global completion flags is advanced by this investigation. [S02:12–16,38–53; S03:714–720]
