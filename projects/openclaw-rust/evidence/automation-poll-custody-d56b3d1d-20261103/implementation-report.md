# OCRM-AUTOMATION-POLL-CUSTODY-v1 — implementation handoff

## Status and exact authority

Implementation and causal test source are complete in the two assigned files. Compilation, test execution, independent review, integration and acceptance remain pending. No executable validation was run by this worker.

- Target: `bhrumom/fabushi-desktop`.
- Sole migration branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Frozen authoring base: `871b033c5b9aabbe322cc7322af92648939db9b5`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`; no upstream source-read or acceptance credit is added.
- Authoring root: `/workspace/scratch/fadc4429f7a8/fusion-automation-poll-custody-20261103`.
- Frozen contract: `docs/specs/sand-completion-ack-safety.md`, read completely before implementation; SHA-256 `40eed97e6ecfbd29748e5f846c1ca1818a17887f775ac6b2b2fed26146415bc9` was checked both before and after the edits. The worker did not edit this spec.
- Root instruction input: `/tmp/automation-poll-custody-contract.json`, complete 36-line read before implementation.
- Root subsequently identified parent `6d65f108476e5e481bc8a9edf28e85da633a743b`. A read-only `ls-tree` of that exact parent confirms both assigned source blobs still equal the authoring base. Root retains and integrates the newer parent tree.
- Phase classification: implementation/evidence design with interacting state and failure semantics; repository default `xhigh` classification. Actual model/effort configuration is inherited from the parent; this worker applied no override and launched no subagents.

## Deliverables and byte identity

Full applicable patch: `/tmp/automation-poll-custody-impl.diff`.

Patch SHA-256: `f9c4a145ca400c781ca6bb61e2f2a1239071fef397c068fd3cf495cca2e4d680`.

| Assigned path | Base blob | Edited blob | Git byte diff |
| --- | --- | --- | --- |
| `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | `7d5f3e50f3a20aa5f142fdfa029cdcc6d40e06c9` | `f7f6789daa21fb21cd09296be049b6e925dfb301` | +24 / -8; 366 to 382 lines |
| `source/host/tests/automations_backend_contract.rs` | `45174d8010a61d631bd4d605ecc1df515a619bbc` | `11a9405442efa9acbbadfcdff879c0a996d1dee0` | +338 / -2; 1077 to 1413 lines |

Only these two repository files were edited. The patch has standard `a/source/...` and `b/source/...` paths and full Git blob IDs. Temporary base materializations used only to generate the byte diff are `/tmp/automation-poll-custody-consumer.base.rs` and `/tmp/automation-poll-custody-tests.base.rs`; their hashes equal the declared base blobs. No real index, Git refs, lock, ledger, schema, workflow, composition file or peer source was modified.

## Actual reading and source identities

All Git source reads below are at the frozen base unless explicitly identified as the authoring spec. File enumeration and symbol searches were discovery only; they are not full-source reading credit. Initial overlong output was not counted as complete reading; the root instruction's truncated middle was reread separately, and applicable source/document ranges were read directly.

| Source | Actual read scope | Base blob / frozen identity |
| --- | --- | --- |
| `AGENTS.md` | Complete 1–395; missing middle explicitly reread at 180–340; nested instruction search found no nearer instruction for the assigned paths | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |
| `docs/specs/openclaw-rust-equivalence-migration.md` | Applicable §17, 637 through EOF, including OCRM-EXEC-02 §§17.1–17.5 | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` |
| Authoring `docs/specs/sand-completion-ack-safety.md` | Complete latest 1–138, including AP01–AP04; before code | SHA-256 above; old base blob `cbe3ceeb072ee9756235ac926f0c09021448ab8c` is not the frozen new contract |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | Direct 1–104 for authority, fixed-U and existing-owner rules; no full historical-document reading claim | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` |
| `projects/openclaw-rust/module-map.md` | Automation/cron/scheduler context from `rg -n -i -C 3 'automations\|cron\|scheduler'`, including M19 and ownership separation; navigation/disposition read, not full | `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` |
| `projects/openclaw-rust/docs/specs/cron-session-reaper.md` | Headings/affected terms, then complete related sections 723–792, 1029–1146, 1399–1475: known Running/alias redelivery, checked backend history, legacy covered-slot custody and retained oracle constraints | `b0a62da2cc86b5479047458019bbb3adfc2b16d1` |
| `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | Complete base 1–366; edited polling function and full resulting diff reread | `7d5f3e50f3a20aa5f142fdfa029cdcc6d40e06c9` |
| `source/host/tests/automations_backend_contract.rs` | Complete base 1–1077 in four contiguous chunks; all added tests and complete resulting diff reread | `45174d8010a61d631bd4d605ecc1df515a619bbc` |
| `source/host/src/extensions/automations/backend_transport.rs` | Complete: actual Reqwest request/auth/status/JSON boundary and Decode error type | `6dc4477c72a38179d71ab33013c9b7e2c3b89991` |
| `source/host/src/extensions/automations/native_backend_transport.rs` | Complete: adjacent `from_value` caller, distinct protocol/account checks and bounded I/O | `9e7a811165610e81719ae26fcf4513a75088074c` |
| `source/host/src/extensions/automations/production.rs` | Complete via 1–260 and 261–EOF: actual runtime construction, tick, pending dispatch, completion/abandon callback, wake/stop lifecycle | `3927e9736bebd27302cd9c4c2139b583e80b7aa4` |
| `source/host/src/extensions/automations/production_lifecycle.rs` | 1–280: shipping Reqwest construction, backend start, error telemetry, adjacent lifecycle | `acc5d67aef35ede93a0f87e317f966013ee2eb8b` |
| `source/host/src/extensions/automations/fire_delivery.rs` | Complete: actual downstream fire shape use, checked history and terminal/refusal interpretation | `17a4466130d516d132db59f2ded772bead34935b` |
| `source/host/src/host_production_extensions.rs` | 1265–1320: sole start slot and inference/schedulability reader passed to lifecycle | `36b5dbac27d346f883150df179372034e0bea223` |
| `source/host/app/src/main.rs` | 4017–4147 actual checked dispatch/launch and completion callback; 17625–17830 shipping fire-dispatch construction/start; 9290–9435 incidental earlier lookup read, not used as fire-wiring authority | `7ec9bd8a22fd595f63cb3f4b632673e298294dfa` |
| `source/host/Cargo.toml` | Complete: existing integration-test crate/dependencies, serde_json features, no dependency changes needed | `e42cc9f56cc98337a49622ae9117ebd9823ef820` |
| `source/host/tests/host_production_extensions_contract.rs` | 465–555, including unchanged single shipping Automations owner assertions | `d8bd79e82e4ae853a121341c8eada15a88467080` |

No independent peer review report was opened. Reports named as provenance inside the root contract JSON were not read or treated as the worker's source review.

## Implemented behavior and owner boundary

The actual shipping path remains App fire-dispatch binding → `ProductionAutomationsLifecycle` → Reqwest final transport → existing `ProductionAutomationsBackendRuntime` → `AutomationFireBackendRuntime::tick_with_state` → the existing callback/checked Host dispatch. No new owner or helper-only alternate path was introduced.

The consumer now requires an object with an `events` array, parses every member through the existing `BackendAutomationFire::from_value`, and checks that each of the three already-parsed identity strings contains a non-whitespace character. It collects the entire typed list into a `Result<Vec<_>, _>` before advancing anything. The `trim()` call is only a predicate: stored, dispatched, completed and acknowledged strings retain their original bytes.

Invalid envelopes return the existing Decode variant with a static shape message. Invalid members return Decode with only a static description and the array index. Neither diagnostic interpolates response contents or identities. The existing outer error branch supplies `AUTOMATION_FIRE_ERROR_BACKOFF_MS`.

The pending notify sequence is captured locally at the original point, but `drained_notify_seq` is committed only after full validation. Clearing `notify_pending`, setting `last_poll_at_ms`, releasing absent Reported entries, dispatching fires, retrying Completed entries, marking the unschedulable drain and applying server delay all remain behind that same checked-list boundary.

For valid responses, the returned-ID set and dispatch iteration now consume the already-parsed list. Existing state ownership, accepted-ID duplicate suppression, false-dispatch removal of only that volatile entry, completion retry semantics, valid-empty acceptance, readiness/notify/stop and delay behavior are retained. No second parse of each event was added in this consumer. The prior whole raw-list clone was removed.

`BackendAutomationFire::from_value` itself is unchanged, preserving optional revision/time/event interpretation and its native transport caller. There is no new wire uniqueness rule, endpoint, auth/provider choice, response receipt field, persistence/account assumption, or clearing of custody on 401/404/409.

## New exact test names and causal oracles

All three tests are in the existing `automations_backend_contract` integration target and use the unchanged `MockTransport` only for final external I/O. They drive the actual consumer's public poll/complete methods and actual dispatch closure. They do not reproduce its parsing or state-transition implementation, inject private state, or substitute mock persistence for the existing real-owner tests.

### `fire_runtime_invalid_poll_preserves_all_custody_until_checked_recovery`

- Declares 53 malformed response cases: all non-object envelope kinds, missing/null/non-array events, non-object members, each missing required field, wrong identity field types, empty and Unicode-whitespace-only identities, invalid unsigned-integer timestamp shapes, invalid already-owed members, a malformed duplicate after a valid prefix and a malformed first member.
- Seeds four entries through an actual valid poll, then real complete calls leave one Running, one Completed after an explicit 409, and two Reported. An unknown completion is also checked to leave transport/state unchanged.
- For every case, performs two bad polls separated by the exact existing backoff boundary. Each requires the exact Decode shape/index diagnostic, unchanged pending count and Running observations, zero added dispatch, and a full transport call vector containing only the repeated exact ACK request. Attempts one millisecond after failure and one millisecond before the backoff deadline must produce no call or dispatch.
- A wholly valid response then permits the actual deferred completion retry with the exact original status/error payload, admits the former valid-prefix fire, retains one returned Reported ID and releases the other absent one. A false dispatch removes only its own entry; a later valid delivery can admit that ID while all already-owned IDs remain suppressed.
- Subsequent exact ACK bodies distinguish retained/released Reported IDs. A final explicit valid empty list releases the remaining Reported entry and retains all three still-Running entries. This intentionally does not equate an empty poll with completion of running work.

### `fire_runtime_invalid_poll_does_not_consume_notify_or_unschedulable_drain`

- Covers an initial failed connected poll while unschedulable and a failed notified poll after an earlier successful drain, with safety polling disabled.
- Observes readiness and stop suppression before polling; actual error backoff; retry eligibility at its boundary without another notification; and unschedulable-drain suppression only after a valid empty response.
- An invalid response advertises a 60-second delay, but cannot apply it. The valid recovery response advertises 5 seconds. Disconnecting the notify channel isolates the server-delay oracle: no poll at 4999 ms and an actual poll at 5000 ms.
- Every observed transport call must be the exact empty-ACK poll, and pending state remains empty.

### `fire_runtime_checked_poll_preserves_opaque_ids_duplicates_and_optional_fields`

- Provides distinct whitespace-surrounded, NFC, NFD and differently-cased run IDs, plus whitespace/Unicode agent and automation identities. Compares all actual dispatched typed envelopes against independently written expected values.
- A valid duplicate with different other fields is accepted and suppressed after the original's single dispatch; no new duplicate rejection or normalization policy is imposed. Repeated returned Reported duplicates stay owed without dispatch.
- Covers absent, null, wrong-type and valid optional fields, an opaque non-null boolean event, unrelated envelope/member fields, and accepted `0`/`u64::MAX` integer boundaries.
- Exact complete request bodies and sorted ACK bodies preserve all identity bytes. A returned opaque ID remains owed until a later valid empty response releases it.

## Existing valid assertions retained

The test diff removes only the two original import lines to add existing constants. No existing test body, assertion, real-owner fixture, fixture cleanup rule or MockTransport method changed. Retained consumer controls include:

- `fire_runtime_polls_delivers_reports_and_acks_on_next_poll`
- `fire_runtime_does_not_poll_until_run_execution_is_ready`
- `fire_runtime_drains_once_while_no_server_schedulable_routines_exist`
- `fire_runtime_matches_frozen_notify_floor_and_safety_poll_gate`
- `fire_runtime_disconnected_and_owed_work_bypass_notify_gate`
- `fire_runtime_config_reset_does_not_impersonate_notify`
- `fire_runtime_successful_completion_honors_next_poll_delay`
- `fire_runtime_never_acks_unconfirmed_completion_or_redispatches_redelivery`
- `reopened_running_original_and_alias_never_execute_complete_or_ack`
- `reopened_backend_reads_hidden_legacy_running_authority_beyond_display_history`
- `reopened_backend_abandons_unreadable_malformed_and_ambiguous_authority_without_ack`
- `reopened_running_history_does_not_fence_a_distinct_event_or_schedule`
- `explicitly_settled_history_replays_terminal_success_or_failure_after_reopen`

All existing relay tests also remain unchanged. App callbacks, checked discovery/history, Running/alias/terminal behavior, production-wiring tests and their separate source files were not edited. Existing controlled transport/owner-reopen results remain bounded local-owner oracles, not actual provider/channel/deployed-service evidence.

## Requirement-to-source assessment and verification state

| Requirement | Source/oracle assessment | Execution/acceptance state |
| --- | --- | --- |
| AP01 | Full checked-list boundary before every specified successful side effect; nonblank exact-byte identities; existing optional and duplicate behavior; first and third new tests | Not executed; exact-H Actions required |
| AP02 | Existing Decode/backoff path; no state mutation, dispatch or completion retry before validation; first and second new tests | Not executed; exact-H Actions required |
| AP03 | Valid retention/release, real dispatch/false-dispatch behavior, exact deferred retry, empty success and lifecycle/delay positives; all old bodies retained | Not executed; old evidence is not relabeled as new-H evidence |
| AP04 | No protocol/auth/account/persistence expansion; consumer remains sole volatile owner and unchanged successful-absence policy is explicitly limited | Source constraint preserved; service/durability acceptance remains unresolved |

Executed operations were source reads, edits, `ls-tree`, non-writing `hash-object`, SHA-256 byte hashes, and `git diff --no-index` byte comparisons. The latter's exit 1 records actual differences, not a test outcome. The complete generated patch was reread; removed lines were separately inspected to establish assertion preservation.

No cargo/rustc/build/test/lint/rustfmt/formatter/generator/schema/project script/validator ran locally, on htch-runtime, or in Actions from this worker. No remote push, ref update or dispatch occurred. No execution receipt or pass verdict is supplied.

Root next owns the complete immutable review scope, review decisions, reconciliation into the newer parent, workflow positive-name/input receipts for the three exact new cases plus retained controls, and final exact-H Actions execution. The required existing integration target is `automations_backend_contract`; use the established workflow rather than a new harness.

## Remaining risks and limits

1. The new Rust source and tests have not been compiled or executed; typing, runtime or integration defects still require the authorized exact-H Actions gates and fresh independent review.
2. A wholly valid successful absence still uses the existing client policy to release Reported IDs. This is not an explicit per-ID service ACK receipt and supplies no deployed protocol/provider/account acceptance.
3. All pending state is still in the existing in-memory consumer. Restart recovery, account/lifetime binding, unbounded retention, terminal-alias history eviction and durable settlement phases remain open as specified by root.
4. A malformed member now intentionally blocks the entire response, including unrelated valid fires and retries, until a wholly valid poll is obtained. This is the frozen fail-closed contract, not a partial-delivery fallback.
5. The controlled final transport proves only the new causal consumer contract when executed. No live HTTP, actual provider/channel, device, package, crash durability or full OCRM completion is claimed. U and all global completion/acceptance flags remain unchanged.
