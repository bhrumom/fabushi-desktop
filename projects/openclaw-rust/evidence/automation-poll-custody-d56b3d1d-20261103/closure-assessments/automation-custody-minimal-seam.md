# Minimal Automation history-custody seam

## Decision and scope

Use **FileAutomationStore's complete history image through AdmittedAutomationRunStore before execution**. Persist unresolved backend IDs on the admitted row and retain them through terminal settlement and ordinary 20-row pruning. No startup requests or unknown-execution replay.

The safety core needs **two production files** (store, Runtime); actual ACK release adds **five composition files**. Pin-only without a valid release rule has a capacity/liveness limitation, not complete recovery. **No history version/downgrade gate exists in this owner.** Optional metadata is stripped by old writes. A new object envelope can make the inspected old mutation parser reject writes; that is a new, narrowly bounded format decision.

Subject: `bhrumom/fabushi-desktop@71b039ae860c45e982dfbc8b1f401143d49d2454`, tree `3c30ec8ac78d0b8b549267a4b794a7cb177b8b23`, bare source `/tmp/ocrm-source-84120837.git`. Parent reports Automation unchanged at 871; independent source identity here remains H71. No cb2 changes or private peer reports were read. No repository edits, execution, validation, service calls or acceptance.

Authority and detailed original cause/identity analysis are reused from `/tmp/next-automation-closure-gap.md` and `/tmp/automation-durable-settlement-identity.md`. Their exact source indexes and unchanged governing AGENTS/SOT/Spec§17/cron/Runner contracts remain applicable. New reads and exact blobs are indexed at the end.

## 1. The exact protected object

Protect the **existing run row plus its exact backend IDs**, not an automation definition, calendar slot, in-memory lock, or all historical terminal records.

Proposed private stored rows carry the existing AutomationRun projection plus validated pending backend IDs. No tokens or fake run status; public AutomationRun/renderer stay unchanged. A versioned envelope can provide bounded rollback rejection (§5).

Actual backend schedule/event callers supply UUIDs; manual and local listener callers do not. Carry explicit custody IDs on AdmittedAutomationRunStore; do not classify arbitrary explicit store UUIDs as backend work. [R:773–1084; P:376–386; E:279–291; App:3975–3980,4060–4076,4112–4121]

Publish custody **atomically with fresh admission**, before cron registration/execution. A post-terminal pin leaves a pruning gap. Write failure keeps AdmissionRefused/Abandon and zero execution. Later cron-registration failure must preserve the pin through error settlement. [R:348–384; P:517–570]

Validate the whole image: version, duplicate keys, exact bounded IDs, membership in the row's primary/aliases and cross-row ambiguity. Never clamp custody IDs. Existing store aliases allow 25 plus primary; actual event batches allow 25 items total. Preserve same-row alias tolerance. [S:125–171,234–269; A:9–29; E:277–291]

## 2. All relevant write/read boundaries

| Boundary | Current behavior | Required treatment for this batch |
| --- | --- | --- |
| `begin_fresh_run` → `begin_run_checked(..., true)` | Complete checked image; refuses claimed primary/alias, then capacity/retention/write | Publish the row and its backend custody together; protect every existing pinned row. Keep collision precedence and all current invalid-history refusal. |
| Legacy `begin_run` → `begin_run_checked(..., false)` | Complete mutation image, normalizes legacy fields; existing primary returns without write | Preserve new metadata even on ordinary/manual callers. Never turn a projected row into a new backend pin. Existing no-write lookup stays no-write. |
| `finish_run` / `finish_run_definition` → `finish_run_with` | Reads full mutation image; edits target; protects only target for that write; serializes all rows | Preserve all pins on target and other rows. A terminal result does not clear custody. Keep exact committed receipt behavior when the target is outside display capacity. |
| `read_runs_for_custody` | Whole-image validation followed by known-field parse | Decode/validate custody losslessly before generic projection; malformed metadata makes custody unavailable, never absent. |
| `read_runs_for_mutation` | Whole-image parse but only AutomationRun fields survive | Carry a private stored-row/image type so every valid custody field survives normal writeback. Preserve existing legacy normalization for unrelated fields. |
| Public `write_runs(id, &[AutomationRun])` | Blind whole-file serialization; no current-image/version/custody check | Cannot remain a way to overwrite a protected image with a 20-row projection. Reject replacement that loses protected rows/IDs or require a complete checked image through an internal writer. Do not silently merge unknown/corrupt input. |
| `retain_run_history` | Protects Running + just-settled target, then fills remaining slots up to 20 | Protect union of Running, unsettled backend custody, and this write's exact settlement receipt. Ordinary terminal slots use remaining capacity. |
| `read_runs` / `get` / `list` / `list_definitions` / `to_record` | Sort and display at most 20; no history write | Continue a bounded projection. A hidden protected row must remain discoverable by checked delivery lookup. Never use the projection as mutation authority. |
| `record_run` / `record_run_definition` | Writes automation.json lastRunAt only | Preserve current behavior; lastRunAt is not settlement or a pin-release condition. |
| `write_config`, create/update/enable/notice and watcher reconciliation | Config writes or read-only history projection | No custody release. Existing definition edits must leave history custody bytes/semantics intact. |
| New exact-ID release mutation | Absent today | Same complete-image validation and admitted Session write boundary; remove only a confirmed member of the matching row/binding. Retain all remaining aliases. |

[S:103–214,224–385,488–512; R:319–405,650–771,1152–1180]

Production searches find only begin S295 and finish S324 calling `write_runs`. Guard that public replacement API anyway. No display-to-history writeback was found in the searched renderer/Host paths; external file writes remain outside scope.

### Capacity and slot release

Candidate capacity: count protected rows, not aliases. At 20, refuse new admission; permit lookup/settlement/release and drain legacy excess without loss/growth. Specify this extended backpressure. One pin still permits twenty sequential manual runs recycling nineteen terminal slots. [S:274–295,488–503; TS:968–1084]

Do not conflate these releases:

- Run-path `in_flight_automation_keys.remove` releases scheduling suppression after a result; it does not settle remote delivery.
- Event batch `pending.remove` and `dispose` release queue/waiters; they do not clear admitted history custody.
- Consumer `abandon(id)` removes a volatile backend claim. Admission refusal, Running history, unknown-effect None, stop and restart must not clear an existing durable pin.
- Local terminal finish changes ok/error and frees Running status. It still leaves the backend pin.
- Only the exact approved ACK-release boundary can remove a pin; completion POST success alone cannot settle the delivery obligation.

[P:449–481; E:205–220,296–327; C:159–204,262–291; prior reports]

## 3. ACK seam and its deliberately bounded promise

Add a **local release observer** before removing Reported IDs actually ACKed in this poll and absent from its accepted response. Retain original agent/backend-automation/run target; resolve through checked Runtime/Session and require exact row membership, not an ID-only global scan. No new remote operation. [C:262–305; B:157–202; App:5997–6040]

The observer uses Runtime's admitted exact-ID mutation. Deletion/retirement, missing target, wrong alias, Running or unavailable history cannot clear/recreate custody. Local failure keeps the pin and Reported state; successful exact-ID release permits ordinary pruning only after all pins are gone.

Reject malformed poll shape before durable release: current missing/non-array events defaults to empty. Narrow acceptance without inventing an ACK receipt field. Completion 401/404/409/transport/decode errors remain Completed and unACKed. [C:187–204,262–291,332–365]

**Identity limit:** bind the observer to originating backend lifetime/current Session owner; retired/mismatched/unknown authority cannot release. Session identity does not prove the Sand bearer principal. The same-owner and successful-absence ACK conditions require explicit spec approval; if unprovable, retain the pin. No cross-account clear is authorized.

**Restart limit:** no startup pin scan/POST. Existing redelivery keeps its existing terminal replay path. Crash after remote ACK before local clear can strand a pin if the server no longer redelivers. Age/new runs cannot clear it. Durable ACK-owed recovery is separate; retained uncertainty is not an unlimited-retention guarantee.

Persisted completion payloads or Completed/Reported phases would expand this batch into the prior recovery design.

## 4. Deletion, coalescing and existing data

### Deletion must have an explicit scope boundary

Runtime `delete_agent_automation` and Session `remove_agent_automation` both call store `remove` (entire automation directory). Session `delete_session` publishes deletion admission, retires owners and removes the agent directory. A Runtime-only guard misses these other owners. [R:756–770; AS:580–604,1023–1034; SP:2347–2390]

Bound this batch to **ordinary retention while automation/session exists**. Preserve authorized deletion; deletion is not completion/ACK and late release must not recreate files. Delete/recreate is outside this guarantee. Forbidding deletion while pinned would require both owners plus a specified tombstone/repair/product policy, a larger scope.

Conversation clear in the inspected Session method clears conversation blobs/DB through the existing owner; it does not call Automation history removal. Do not use conversation clear, DB cron-row settlement or sweep throttle as an alias-release signal. [SP:4183–4203; P:74–103]

### Coalesced aliases

The coalescer selects first supplied UUID as primary, the rest as aliases; local None items can share the batch. Pin only actual admitted backend IDs, not generated IDs/pre-admission drops. Shared execution outcome does not join ACK ownership: P ACK never clears A. [E:97–150,264–305; App:3975–3980,4112–4121]

Overflow, disposed queue, Spend Guard refusal, duplicate in-flight rejection and failed checked admission do not authorize a new durable run/pin. Conversely a row already admitted before an unknown outcome remains protected when the waiter resolves None. Do not replace existing Running/history-unavailable/legacy-slot Abandon with replay to make a test progress.

### Old data

No metadata means no evidence of an unACKed historical obligation. Do not pin all old terminal rows or reconstruct evicted aliases. Create pins on new admissions; adoption of old terminal redeliveries is a separate spec choice. Preserve existing normalization, duplicate/ambiguity refusal, missing/unavailable distinctions, regular-link behavior and excess-Running drain controls.

## 5. Version and rollback: what is actually possible

Current runs.json is an unversioned array. Mutation decoding requires `Vec<UniqueJson>`; row parse/serialize retain only known fields. Session admission serializes writes/deletion, not format compatibility. Its DB record versions are unrelated. [S:9–18,337–384,511–512; A:15–22; AS:902–915; SP:2291–2344]

| Format strategy | Source-supported consequence | Decision |
| --- | --- | --- |
| Add optional custody field to old array rows | Old H71 parser accepts row, drops field, later begin/finish rewrites without it | Insufficient rollback safety; do not call backward-compatible |
| Put formatVersion only in a row/config | Old known-field parsers can ignore/drop it | No existing enforcement |
| Keep status Running or encode a fake alias to trick old retention | Conflicts with terminal/identity semantics; old finish can overwrite it | Reject; not a valid custody format |
| New object envelope with explicit supported version and rows | Old H71 `Vec<UniqueJson>` mutation decoder rejects the object before either begin/finish reaches write_runs | Viable bounded fail-closed rollback candidate; it is a new format change |
| Reuse a Session DB version or owner lock as global minimum-binary gate | No such Automation check found in inspected composition | Not available as an established solution; do not repurpose unrelated admission versions |

Keep any new `{formatVersion: ..., runs: [...]}` envelope private to FileAutomationStore. Accept legacy arrays and the exact supported version; reject unknown versions/duplicate keys/malformed rows. Display stays unchanged. Conversion back to array requires an explicit no-pins migration rule.

Old-H71 begin/fresh-begin/finish all reach the array-only decoder before history write, so an object envelope makes them error. Old display becomes empty; earlier notice/lastRunAt writes may already occur. This is history-preserving refusal, not functional rollback. Direct public write_runs, remove, agent deletion, external writes/restores and uninspected old binaries remain unprotected.

Any supported rollback target needs exact-source old-writer/readback evidence in Actions. An imitation parser test is insufficient; nothing was executed here.

## 6. Minimum production-file plan

| Production file | Bounded change |
| --- | --- |
| `source/host/src/automations/automation_store.rs` | Private checked complete image; version/pin validation; lossless mutation; atomic admission with custody; retained-capacity policy; guarded replacement; exact-ID release |
| `source/host/src/extensions/transcript/automation_runtime.rs` | Mark actual backend IDs on AdmittedAutomationRunStore; call store admission through existing Session closure; expose checked exact-ID release using same owner/locks |
| `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | Local ACK-release observer before volatile removal, checked poll shape, original target retained; no completion-error reclassification |
| `source/host/src/extensions/automations/production.rs` | Wire observer to the existing fire owner and permanent lifetime; maintain lock order and dispatch-outside-lock behavior |
| `source/host/src/extensions/automations/production_lifecycle.rs` | Pass observer through existing constructor; no startup scan or auth-domain switch |
| `source/host/src/host_production_extensions.rs` | Pass real observer through actual start_automations composition |
| `source/host/app/src/main.rs` | Install observer backed by the same gateway/AutomationRuntime/Session owner used for real backend fire dispatch |

Existing begin_run arguments carry primary/aliases; Runtime can add custody to its admitted wrapper without changing the public trait. This plan needs no public AutomationRun, Renderer, coalescer, run-path, Session production, Auth or native-transport edits. Review their affected contracts. If §3 authority is unavailable, refuse release and state the limitation; Auth/startup recovery would expand scope.

## 7. Real App/Runtime oracle plan

Use existing fixtures, bounded synchronization and retained failure artifacts:

1. In `source/host/tests/automation_run_production_wiring_contract.rs`, add a sibling of `real_retained_run_survives_twenty_manual_settlements_and_reopened_backend_redelivery`. Actual P+A coalescing → safe local terminal → unconfirmed A completion → twenty real manual executions → close/reopen owners → A redelivery → no second execution. Check retained exact terminal/aliases, bounded display and unchanged original cron authority. Preserve the existing test: it returns RetainedUnsettled and proves Running protection, not this terminal case. [TR:1350–1618]
2. In `source/host/app/src/tests/automation_terminal_custody_tests.rs`, extend `checked_backend_delivery`, which invokes the shipping checked dispatcher and forwards its real callback. P release must not clear failed A; later A ACK permits pruning. Include malformed poll, write failure and retired-owner negatives. The current fixture always succeeds completion and launches Schedule only: add a real Event path or schedule companion rather than claiming it covers aliases already. [TA:900–1086,1120–1185]
3. In `automation_store_contract.rs`, cover both begin/finish variants, replacement, legacy/new/unknown format, capacity/excess, no-write refusal and exact-alias release. Keep TS942/968/1022 assertions. Update array-only fixture readers with exact version checks, not permissive fallback.
4. In `automations_backend_contract.rs`, preserve success, 401/404/409/lost-response no-ACK and repeated-ACK controls; add observer-failure retention and only-sent-ACK-ID release. These prove client policy, not service idempotency.

Release must pass through the real ACK consumer. Check admission-write failure gives zero execution; local/manual/None gets no pin; config updates preserve custody; deletion never fabricates ACK/recreates files; normal unpinned pruning remains. Store-only helper tests cannot prove actual production admission.

## 8. Exact source and read extent

Exact H71 path/blob mappings are inherited from §11 of `/tmp/automation-durable-settlement-identity.md` (ID report) and §12 of `/tmp/next-automation-closure-gap.md` (gap report), with this explicit reference map. This avoids repeating the full hashes while retaining immutable attribution.

| Here → prior index | New read in this pass; other evidence explicitly reused |
| --- | --- |
| S → ID R03 (`automation_store.rs`) | 1–225,337–410; prior full 1–515 reused |
| A → ID R04 (`automation.rs`) | Full 1–87 |
| R → ID R01 (`automation_runtime.rs`) | 650–1088,1148–1248; admitted-store/lock/dispose prior ranges reused |
| P → ID R06 (`automation_run_path.rs`) | 20–145,300–386,448–580,700–862; prior 1–861 reused |
| E → ID R02 (`automation_event_fires.rs`) | Full 1–337 |
| C → ID B04 (fire consumer); B → ID B06 (backend production) | Prior full 1–366 / 1–310 reused; no new full read |
| App → ID I08 (shipping main) | 3958–3990 and caller searches; prior H71 callback/startup/stop ranges reused |
| AS → ID R05 (`agent_session.rs`) | 545–615,830–1045 |
| SP → ID I09 (Session production) | 2280–2435,4178–4210 |
| TS → gap T01 (`automation_store_contract.rs`) | 934–1086 and name searches |
| TR → gap T03 (`automation_run_production_wiring_contract.rs`) | 1350–1636 requested, load-bearing 1350–1618 directly reviewed; prior 1350–1635 reused |
| TA → gap T04 (App custody tests) | 900–1086,1120–1185 |

TS/TR/TA H71 blobs were rechecked: `81720b74a8db66bbcd90dbb19c7fcd624599089b`, `fcf496dd0dd8c20810d064b98a31c67a310b82ad`, `391c685d143e7e38b66b54e5b60ef104201d0f12`. Search-only: version/writer references, run-entry callers and direct write_runs consumers. H71 agent_db blob `6c27e9bb9f665b1160457b9acc29dd5d64fba289` was searched, not fully audited. The concrete no-version-gate conclusion concerns the fully read Automation mutation chain, not a global absence inferred from keywords.

## Handoff

Before code, specify retention scope, admitted IDs, capacity, ACK authority, deletion, format/rollback and pin-only limits plus the causal oracle. No implementation/validation/acceptance; incoming H/cb2 and existing Abandon assertions remain intact.
