# Automation poll custody — round 1 structure, reuse and performance review

## Verdict

**No blocking findings in the complete declared delta.** The checked-response boundary remains in the existing `AutomationFireBackendRuntime`, uses the existing event decoder and error/backoff path, and does not introduce a second scheduler, custody store, protocol owner, or production state machine. I found no concrete structural or performance regression requiring a fix in this snapshot. Preference-only extraction, naming, allocation micro-optimizations, and broader pre-existing resource-policy work are not blockers.

This is an independent, blind, source-only review. It is not a compilation, test, benchmark, GitHub Actions receipt, deployed service verdict, or OCRM acceptance. Required exact-head executable evidence remains pending.

## Immutable subject and review execution identity

- Repository: `bhrumom/fabushi-desktop`.
- Read-only Git subject: `/tmp/ocrm-source-84120837.git`.
- Frozen base / starting HEAD: `b15e58957a597997363e1273f2f32a222365c310`.
- Snapshot commit: `bd3dd6919c9bb6e71ba54ddf67919b9f3d324a1a`.
- Snapshot tree: `e9b2613c22285207c5c061592909f69bc830a8de`.
- Manifest: `/tmp/automation-poll-custody-r1-snapshot.json`.
- Complete diff: `/tmp/automation-poll-custody-r1.diff`, 31,602 bytes, SHA-256 `71f03b5df18591280d9477fe99b8fd2e0c9f9613f0c0bbb301285e89acddf15e`.
- Fixed upstream U retained by the source/spec/lock: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`. The lock continues to mark implementation-round acceptance and baseline identity acceptance false; this review does not promote them.
- Review lenses requested: structure, reuse, performance. Requested model/effort: `gpt-6.1-sol` / `xhigh`, justified by a stateful poll/complete/ACK consumer, its causal oracles, and CI receipt acceptance.
- Observable adapter setting: the worker's provided context contains the requested model/effort but does not expose an independently inspectable effective model or reasoning-effort field. I did not launch another CLI, change global settings, or create another worker to tune the setting. Prompt wording is not treated as proof of enforcement; parent launch metadata must establish the applied setting or record the adapter fallback.

Read-only Git and Python standard-library content checks confirmed that base/snapshot are commits, the snapshot resolves to the manifest tree, the complete diff has its declared byte length/hash, the Git changed-path set is exactly the five paths below, and every declared blob, byte length and SHA-256 matches the immutable snapshot. These are source-identity checks, not project executable validation. No real index or reviewed source was modified.

| Declared path | Snapshot blob | Bytes |
| --- | --- | ---: |
| `docs/specs/sand-completion-ack-safety.md` | `5d1fe5c453e51445ac9c563191e3c6d571d81182` | 10,482 |
| `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | `f7f6789daa21fb21cd09296be049b6e925dfb301` | 12,243 |
| `source/host/tests/automations_backend_contract.rs` | `11a9405442efa9acbbadfcdff879c0a996d1dee0` | 63,156 |
| `.github/workflows/rust-desktop-runtime.yml` | `37f90cfcddef5fbc917d07b5fa5468707a4bc452` | 452,895 |
| `projects/openclaw-rust/docs/specs/cron-session-reaper.md` | `978c7521d46f4dc95edb5e5347bcb890ea2a2582` | 100,390 |

## Whole-diff reading, policy and adjacent source

I read every hunk in the complete five-file diff, including all three added test functions, the test helper/import changes, all workflow acceptance/input/receipt additions, and both documentation additions. The diff was read in bounded complete ranges through EOF after an earlier combined tool output truncated. No omitted range is being claimed as read.

Applicable instructions and authority read directly from the snapshot:

- Entire root `AGENTS.md`, including independent review, frozen immutable scope, model/effort, existing-owner, source-depth and Actions-only requirements. The tracked-tree instruction inventory contains no nested `AGENTS.md` governing any of the five paths; the other tracked nested instruction file is under the unrelated Codex TUI bottom-pane subtree.
- `docs/specs/spec-first-ai-development.md` discovery, lifecycle, review and fail-closed requirements, lines 30–138.
- Entire `docs/specs/sand-completion-ack-safety.md`, including the original completion-error rules and AP01–AP04 continuation.
- Relevant authority and constraints in `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, particularly current fixed-U identity and precedence, existing-owner boundaries, independent progress states and Actions-only execution.
- `docs/specs/openclaw-rust-equivalence-migration.md` existing-owner architecture, resource/fault constraints, exact-subject evidence contract, and the full latest §17 OCRM-EXEC-02. I did not claim a full reread of unrelated domains in this global spec.
- Current `projects/openclaw-rust/upstream.lock.json` and the Automations responsibility entry OCRM-M19 in `module-map.md`.
- `cron-session-reaper.md` responsibility/context and history-ACK boundary, relevant custody/history/admission/deferred-legacy section locations, and the complete new checked-poll prerequisite. The unrelated historical reaper/cancellation detail was not treated as a new scope requirement, and the full 100 KB historical document was not claimed as deeply reread.

Production and test integration read:

- Entire changed fire-consumer file, including `BackendAutomationFire::from_value`, state representation, `complete`, readiness/notify/unschedulable gates, `tick_with_state`, `retry_completions`, and `report_completion`. The base implementation's affected tick section was also read directly for allocation/behavior comparison.
- Entire `backend_transport.rs`: HTTP status/JSON decoding is a separate boundary from event-shape acceptance, and `Decode` is already an existing error category.
- Entire `native_backend_transport.rs`, the other actual `from_value` caller: its protocol/size/receipt checks remain unchanged; the consumer does not acquire or redefine native-account identity.
- `production_lifecycle.rs` composition through construction of the Reqwest transport, the fire poll error sink, and existing backend runtime startup; `production.rs` actual mutex-held tick, pending-fire collection, deferred dispatch/completion/abandon callbacks, notify subscriptions, suspend/resume, stop and join paths.
- `fire_delivery.rs` preparation/history/revision/optional-event consumers, lines 1–260; the optional event body's deeper provider-specific parsing was not changed and was not claimed as completely reread.
- App `dispatch_checked_production_backend_fire`, immediate Complete/Abandon/launch branches, completion mapper entry, and production `fire_dispatch` lifecycle registration. The full large App main file was not reread.
- Existing `MockTransport` implementation and existing success/ACK/delay, readiness, initial unschedulable drain, notify floor/safety, disconnected/owed-work bypass, config-reset, and unconfirmed completion-error tests. Also read the real-owner test fixture's poll/complete/abandon path and reopened Running/alias/hidden-history custody tests. Test enumeration and adjacent production-wiring references were checked; unrelated unchanged test bodies were not all deeply reread.
- `source/host/Cargo.toml` default test target/dependency declaration, workflow triggers and exact-subject binding, complete affected Automation command/parser/receipt block, final artifact upload/failure fixture capture, and the downstream same-subject receipt reuse consumer. The entire 452 KB workflow was not claimed as reread.

I did not read root runner decisions, implementation worker reports, peer reports or prior review reports. No local or htch-runtime build/test/lint/schema/generator/project-helper/benchmark/acceptance command was run. No network/live-service or Actions result was used.

## Structure and reuse assessment

The changed code is placed at the actual owner of `Running`, `Completed(FireCompletion)` and `Reported`, `states: BTreeMap<...>` in `sand_automation_fire_consumer.rs:115–133`. The transport remains the existing `AutomationsBackendTransport`, and `production_lifecycle.rs:136–175` still selects the shipping Reqwest path. `production.rs:157–202` still calls the same tick under the existing mutex and then invokes the actual dispatch callback outside that lock. No public interface, callback type, endpoint, provider choice, dependency, module, schema or persistence path is added.

`tick_with_state` now builds a fallible temporary `Vec<BackendAutomationFire>` at lines 274–295. It delegates required and optional field interpretation to the existing `BackendAutomationFire::from_value`; the added filter only enforces the specified non-whitespace identities without rewriting them. Every member, including already-owed IDs and duplicates, is checked before `drained_notify_seq`, `notify_pending`, `last_poll_at_ms`, Reported retention, dispatch or completion retry can advance at lines 296–338. The vector is a per-response acceptance product, not another canonical state truth.

For an invalid envelope or member, `?` leaves the closure before those transitions. The existing outer error branch at lines 342–344 applies the same 30-second error backoff. Completion POST errors retain the existing `Completed` payload; successful reports alone yield `Reported`. Valid empty arrays still reach the existing per-ID absence release policy, and Running/Completed entries still survive absence. Duplicate admission still relies on the same state-map membership check; this patch does not assert a new server uniqueness rule or change failed-dispatch retry semantics.

The old pre-poll assignment to `drained_notify_seq` becomes a local captured value, committed after validation. Since production serializes tick and notify mutations on the same mutex, this does not introduce a concurrent alternate sequence owner or require a new callback/protocol mechanism. Keeping the acceptance sequence in this function is proportionate to the bounded change; extracting a separate parser module or schema would not remove duplicated domain policy here.

The other `from_value` caller in `native_backend_transport.rs:100–105` remains an existing transport-level protocol guard. Its current decode-before-consumer layering predates this delta. The patch neither reimplements that protocol nor moves its native credential/receipt checks into the legacy/Sand consumer.

## Performance and resource assessment

The change removes `.as_array().cloned()` from the base implementation. Previously the response retained its original JSON array, a complete clone of every raw event (including unrelated fields) was held for iteration, and `from_value` cloned the known strings/optional event again for each dispatch candidate. The new code borrows the original array, constructs known typed fields once, and reuses those objects for the returned-ID set and dispatch. It does not deserialize or parse the accepted event a second time in the consumer.

Whole-list validation necessarily keeps parsed event data until the acceptance boundary; that temporary storage is proportional to the received list and carried optional event payload. The returned `BTreeSet<String>` still clones IDs as before, preserving the existing roughly O(n log n) membership preparation and O(m log n) Reported-state check for n returned events and m pending states. Whitespace checks add scans of the three identity strings, and a malformed response releases the temporary parsed prefix on error. Neither path creates persistent additional retention, threads, timers, I/O, or an additional network request.

Borrowing IDs for the temporary set or tuning vector reservation could be considered later if a measured workload warranted it. I found no concrete failure or realistic material cost introduced here that justifies making either a blocking requirement. The existing Reqwest response-size policy and pre-existing in-memory custody growth remain broader open resource work; this review does not claim a new quota or completion of OCRM-R21/OCRM-AC21 budgets. No latency, allocation count, CPU, power, or soak measurement was executed or inferred from Rust alone.

## Oracle and CI receipt sensitivity

The first new test (`automations_backend_contract.rs:242–403`) seeds actual Running, failed/unconfirmed Completed, and two Reported states using real `tick`/`complete`, rather than writing private state or testing only a standalone parser. Each malformed envelope/member is tried twice. It compares exact full transport call sequences and ACK bodies, checks no prefix dispatch or premature completion retry, checks the backoff boundary, then uses valid recovery to observe the exact deferred completion body, retained/released Reported IDs, failed dispatch release, later redispatch and valid-empty release. A mutation that treats malformed success as an empty list, skips a bad member, validates only unseen IDs, dispatches a valid prefix, retries completion before validation, or alters the pending completion body would contradict these assertions. This is source-level sensitivity reasoning; no mutation run is claimed.

The second test (lines 407–472) uses the existing readiness/stop/start/tick entry points and clock arguments to distinguish a first unschedulable drain from a requested notify drain after a prior success. Invalid `nextPollAfterMs` is not consumed as a successful delay. It checks the exact error-backoff edge, a later valid retry, post-success suppression and the actual valid server delay. It does not introduce sleeping fixtures or another lifecycle harness.

The third test (lines 476–566) compares complete dispatched structs, exact completion and sorted ACK identities for whitespace-bearing opaque IDs, NFC/NFD distinctions and case distinctions, including valid duplicates with differing payloads. It preserves the existing optional revision/time/event handling, u64 boundaries, unknown fields and explicit valid empty lists. Existing test assertions are retained; the delta adds tests/imports/helper only.

The affected workflow block (`rust-desktop-runtime.yml:3833–3957`) already invokes `--test automations_backend_contract` through the same nine-target command. The new three names are counted from exact positive `test ... ok` lines, each must occur once, and `poll_custody_passed` is conjoined with existing command/capture exits, successful nonempty summaries, memory/legacy-slot predicates and exact `FABUSHI_TARGET_SHA`. The added input hashes include the bounded spec and actual transport/lifecycle sources, alongside the already-hashed consumer, tests and production source. It therefore cannot record a passing new poll-custody predicate merely from invocation or suite presence. The existing downstream same-subject receipt reader still requires `result == passed`, matching subject/tree/upstream/log hash and successful exits; it accepts the added metadata without a second evidence format owner.

The receipt text accurately limits this to the local in-memory consumer with a controlled final transport. It does not turn absence into an explicit service ACK receipt, prove restart/account recovery, or claim real provider/deployed idempotency. The existing workflow step executes on its declared OCRM config dispatch or canonical working-branch push conditions; a generic PR event does not alone establish this step ran. A fresh exact-H Actions receipt is still necessary.

## Findings and remaining evidence boundary

- Blocking findings: **none**.
- Concrete nonblocking defects introduced by this delta: **none identified**.
- Speculative cleanup or preferences: no action required for this round.

AP01–AP04 have matching source structure and behavior-sensitive test assertions in this snapshot. Execution outcomes, source closure, deployed service semantics, account binding, durable restart/settlement, terminal-alias retention, numeric resource budgets and the complete G01–G12/AC01–AC24 acceptance remain outside this source-only verdict and are not promoted.
