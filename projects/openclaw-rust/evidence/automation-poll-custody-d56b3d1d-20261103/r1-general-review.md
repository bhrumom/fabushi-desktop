# OCRM-AUTOMATION-POLL-CUSTODY-v1 — round 1 general, lifetime, security and spec review

## Verdict

**APPROVE the frozen source delta for this review scope. No blocking defect or actionable non-blocking finding was identified.** This is a read-only source/spec/oracle assessment, not a build, test, service, platform or release acceptance result. Exact-H GitHub Actions evidence remains required before the implementation is called verified.

## Review identity and independence

- Repository: `bhrumom/fabushi-desktop`; inspected bare object database: `/tmp/ocrm-source-84120837.git`.
- Frozen base / starting HEAD: `b15e58957a597997363e1273f2f32a222365c310`.
- Base tree: `7ebf0e49373bae212ed95be63d7433309d8d02b7`.
- Immutable reviewed snapshot: `bd3dd6919c9bb6e71ba54ddf67919b9f3d324a1a`.
- Reviewed tree: `e9b2613c22285207c5c061592909f69bc830a8de`.
- The snapshot's actual parent is the frozen base above.
- Manifest: `/tmp/automation-poll-custody-r1-snapshot.json`.
- Complete supplied diff: `/tmp/automation-poll-custody-r1.diff`, 31,602 bytes, 571 lines; SHA-256 `71f03b5df18591280d9477fe99b8fd2e0c9f9613f0c0bbb301285e89acddf15e`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`, as declared by the unchanged current lock and current source-of-truth amendment. The lock's acceptance/closure flags remain false; this review does not elevate them.
- Review date: 2026-11-03 UTC.
- Requested effort: **xhigh**, appropriate because the assignment includes custody, completion/ACK ordering, notify/drain timing, lifetime/security boundaries and a CI acceptance oracle. The session identifies the configured assistant as **GPT-6 Astra Pro**; no model override was attempted. An independent runtime model/effort introspection surface was not available, so the requested effort is not represented as independently measured/applied metadata. The configured/inherited session was retained.
- Fresh blind review: I did not read a worker report, root decision ledger, peer report or previous review. Decisions quoted within the required product specifications were read as specification content. The authoring worktree and real/private indexes were not used or modified.

Read-only Git object and standard-library byte inspection established that the supplied diff exactly equals the binary-safe, full-index Git diff between the declared commits for the declared paths. The complete commit-to-commit changed-path set consists of exactly those five paths. Every manifest blob, byte count and SHA-256 matched the corresponding snapshot object.

| Declared path | Snapshot blob | Bytes |
| --- | --- | ---: |
| `docs/specs/sand-completion-ack-safety.md` | `5d1fe5c453e51445ac9c563191e3c6d571d81182` | 10,482 |
| `source/host/src/extensions/automations/sand_automation_fire_consumer.rs` | `f7f6789daa21fb21cd09296be049b6e925dfb301` | 12,243 |
| `source/host/tests/automations_backend_contract.rs` | `11a9405442efa9acbbadfcdff879c0a996d1dee0` | 63,156 |
| `.github/workflows/rust-desktop-runtime.yml` | `37f90cfcddef5fbc917d07b5fa5468707a4bc452` | 452,895 |
| `projects/openclaw-rust/docs/specs/cron-session-reaper.md` | `978c7521d46f4dc95edb5e5347bcb890ea2a2582` | 100,390 |

## Read scope and limits

I read the entire 571-line declared diff; the complete current Sand safety spec, fire consumer, backend contract and cron-session-reaper spec; root `AGENTS.md`; and the complete `docs/specs/spec-first-ai-development.md`. The snapshot contains only one nested `AGENTS.md`, under `source/mahayana/codex-rs/tui/src/bottom_pane/`; it does not govern any changed or relevant inspected path.

For OCRM authority I read the current source-of-truth authority/fixed-U amendment and constraints, current `upstream.lock.json`, applicable module-map ownership/authority entries, the complete latest OCRM-EXEC-02 §17, §12.3 evidence/subject-carrier requirements, and R21/AC21 and relevant gate/flag declarations. Historical discovery identities were not substituted for the fixed U. This is a target-owner response-acceptance correction; no new upstream full-file reading or full-source closure claim is made.

Adjacent shipping source read includes:

- The complete `backend_transport.rs`, `production.rs`, `production_lifecycle.rs`, `fire_delivery.rs` and Automations module declarations.
- `host_production_extensions.rs::start_automations` / `stop_automations`, including the readiness/schedulable reader and unique lifecycle slot.
- The actual App composition at `main.rs:17676–17787`, `UnifiedGatewayApi::dispatch_production_backend_fire` at 4180–4289, `dispatch_checked_production_backend_fire` at 6032–6076, and `automation_fire_completion` at 6078–6096, plus their adjacent reporting/callback context and retained completion-mapper test.
- The complete `automation_lifecycle_production_wiring_contract.rs`; production-source extraction and checked-backend/typed-attempt AST oracles in `host_runner_composition_production_wiring_contract.rs:1–106,1084–1322`; and the existing actual backend App harness's callback, refusal and completion/ACK assertions in `automation_terminal_custody_tests.rs:1060–1235`.
- `source/host/Cargo.toml` and module/build reachability. Adjacent production, App, wiring/delivery/production-wiring contract blobs, Cargo manifest/lock and upstream lock were also compared with the frozen base and are unchanged.
- Workflow triggers, exact-subject binding, Linux job/cache/fixture setup, the entire affected Automation command/receipt step, the adjacent App entry, later same-subject receipt consumer, and failure/artifact upload boundary. I did not claim to read the unrelated remainder of the 452,895-byte workflow.

No project code, test, formatter, linter, validator, generator, build, package, local/htch verification or network service probe was executed. Operations were Git/source reads, standard-library inspection of source bytes/identities, and writing this report outside the repository. No executable Actions result was supplied or reviewed for this snapshot.

## Applicable requirements and source assessment

The governing bounded behavior is AP01–AP04 in `sand-completion-ack-safety.md:82–110`, together with its causal verification and preservation contract at 114–133. Original R1–R4 completion-error retention still applies. The cron amendment at 1478–1489 adds a prerequisite/reference without changing responsibility status, reading history, fixed U or global completion flags.

| Requirement | Source/oracle assessment |
| --- | --- |
| AP01: validate an object with a complete `events` array and every required member before custody/drain/dispatch/retry effects | Satisfied by source ordering in `tick_with_state:262–323`. `as_object` / `events.as_array` and a fallible collection into `Vec<BackendAutomationFire>` must all succeed before the first post-response state mutation. `from_value:83–95` keeps required string/unsigned-integer and optional-field interpretation. Identity admission adds only non-whitespace checking. |
| AP02: malformed envelope/member yields Decode, retains exact custody/payload, applies backoff and does not consume a successful drain | Satisfied by source ordering and the new causal assertions. Failure exits before `drained_notify_seq`, `notify_pending`, `last_poll_at_ms`, `states.retain`, dispatch, completion retry, unschedulable-drain state or server-delay update. The shared error path at 342–344 advances existing backoff. |
| AP03: valid per-ID retention/release, ordinary dispatch/failure, completion-error behavior and timing controls remain | Satisfied in source. Only absent `Reported` entries are released at 306–308. Running/Completed entries remain. Dispatch inserts Running before the callback, and a false callback removes only that fire. Existing completion retry and delay logic is retained. Valid empty arrays are accepted. All previous backend-contract source is retained. |
| AP04: existing owner, opaque identity, auth/endpoint/provider boundaries and honest service/durability limits | Satisfied in the diff and adjacent call chain. No new module/store/schema/dependency/scheduler/bridge or auth selection exists. No error-to-Reported promotion was added. The specs and new receipt explicitly retain the in-memory/deployed-service/account/restart limits. |
| R1/R2: direct and retried completion failures keep Completed; only success permits Reported/ACK | `complete:187–203`, `retry_completions:348–365` and `report_completion:367–380` are unchanged. The existing 401/404/409/transport-loss regression at backend contract 804–843 is retained verbatim. |
| Verification/preservation and OCRM §17 | The selected tests exercise the actual consumer through its final transport trait, and the workflow requires their exact positive names. This provides a sound authored oracle, not executed proof. Source review satisfies only this review phase; exact-H Actions and all remaining applicable final gates remain outstanding. |

### Atomic acceptance and exact identity

In `tick_with_state`, the only pre-response work is eligibility checking, local capture of `notify_seq`, building the ACK request from current Reported IDs and sending the existing poll. None of these releases local custody. Parsing uses an owned, fallibly collected vector; a valid prefix cannot reach the dispatch loop if a later member fails. A malformed duplicate or already-known member is checked before either deduplication or state lookup can skip it. Temporary parsed values are dropped on error without replacing retained completion data.

`trim()` is solely the admission predicate. The accepted strings themselves remain the `to_string()` copies made by `from_value`, and those exact strings remain BTreeMap/BTreeSet keys, dispatch values and completion/ACK identifiers. No case folding, Unicode normalization or trimming is applied to stored or transmitted identities. The parser continues to allow unrelated fields and the existing optional revision/time/event interpretation. Timestamp interpretation remains the existing `as_u64()` contract, including zero and `u64::MAX`.

The new diagnostics contain only a fixed shape message or member index. They do not interpolate an ID, event payload, token or completion error into the Decode detail. The existing lifecycle error sink and log can therefore surface these particular failures without exposing response contents (`production_lifecycle.rs:149–165`; `production.rs:175–177`).

This is local response-acceptance atomicity. The outbound poll still carries ACK IDs before a response exists; source inspection cannot establish or undo server-side effects of that request. Valid absence remains the explicitly unverified existing client release policy, rather than an explicit service ACK receipt.

### Lifetime, concurrency and shipping reachability

The production chain is live source: App composition supplies `fire_dispatch` to `start_automations`; the unique `ProductionAutomationsLifecycle` constructs `ReqwestAutomationsBackendTransport`; `ProductionAutomationsBackendRuntime` constructs this same consumer. Reqwest returns a generic successfully decoded JSON Value after status checking (`backend_transport.rs:65–99`), so this consumer is the relevant poll-shape acceptance boundary.

The backend holds the same `Arc<Mutex<AutomationFireBackendRuntime>>` for poll, notify and completion access. It accumulates pending fires only through the consumer callback while polling under that mutex, then performs actual App dispatch after releasing the guard (`production.rs:157–202`). A malformed list now produces no pending fire. Notify callbacks serialize through the same mutex; this patch adds no lock, reentrant callback, asynchronous task or broader lock scope. Accepted values are owned, so there is no response-buffer borrow crossing dispatch or completion.

The App's checked target/history lookup still precedes execution, and its actual callback still maps Some to `complete` and None to `abandon`. The changed response parsing does not bypass retained Running history, typed admission refusal, terminal replay, account/auth selection or provider controls. Stop/start and owner shutdown are unchanged; no durable restart, account-switch or external-effect reconciliation guarantee is inferred.

### Backoff, notify and delay

All four timing constants remain unchanged: error backoff 30,000 ms, maximum server delay 60,000 ms, notify floor 4,000 ms, safety poll 120,000 ms. The failed attempt no longer records `drained_notify_seq`; `last_poll_at_ms`, pending-notify clearance and `drained_while_unschedulable` are also after successful whole-list validation. A malformed `nextPollAfterMs` envelope cannot impose a successful server delay. Existing absolute/saturating time arithmetic and readiness/stop/owed-work gates remain in the same owner.

This assessment is of the existing supplied-`now_ms` state-machine policy, not wall-clock or network-latency measurement. Independent asynchronous direct completion callbacks retain their existing behavior; the poll's scheduled `retry_completions` stage is suppressed until a wholly valid poll.

## Causal oracle assessment

The new helper only constructs fixture JSON. It does not copy production validation or custody logic. `MockTransport` records actual path/body calls and consumes an explicit response queue, while the tests seed the real state machine through poll and complete methods.

1. **`fire_runtime_invalid_poll_preserves_all_custody_until_checked_recovery`** (`automations_backend_contract.rs:242–404`): source inspection identifies 53 malformed-response scenarios: 11 invalid envelopes, 41 invalid members appended after a valid prefix, and an invalid-first-member control. These include missing required fields, every relevant wrong type, empty/Unicode-whitespace identities, invalid timestamps, an already-owed ID and a malformed duplicate. Each scenario seeds Running, a failed-409 Completed with an exact whitespace/Unicode error payload, and two Reported entries. Two successive failures assert precise Decode diagnostics, unchanged state/dispatch counts, exact repeated ACK bodies and no completion retry. Boundary ticks prove the backoff. Valid recovery proves the exact pending completion is retried, returned Reported custody is retained, absent Reported custody is released, failed dispatch can retry, and valid empty release leaves Running entries intact. Unknown completion also produces no request.
2. **`fire_runtime_invalid_poll_does_not_consume_notify_or_unschedulable_drain`** (`:407–473`): both initial-unschedulable and previously-drained/notify cases exercise not-ready and stopped no-call controls, malformed response refusal, no call before backoff and retry at its boundary. The rejected response's 60-second hint cannot block the 30-second retry. A valid response consumes the drain; disconnecting the notify gate in the second branch isolates the actual 5,000-ms server-delay boundary. The first branch checks that a later unschedulable tick does not re-drain.
3. **`fire_runtime_checked_poll_preserves_opaque_ids_duplicates_and_optional_fields`** (`:476–566`): compares actual dispatched structs and completion/ACK requests for padded identities, distinct NFC/NFD and case, zero/max timestamps, changed-content duplicate IDs, optional missing/null/wrong-type values and unrelated fields. The first accepted duplicate is the dispatched value, Reported duplicates do not redispatch, and later valid-empty release retains exact ACK identity.

These are meaningful consumer-state oracles rather than parser-only assertions. They would fail against the frozen base's absent/non-array-as-empty and malformed-member-skipping behavior. That statement is a source counterfactual, not a claim that a mutation or baseline execution was performed.

### Retained assertions

A standard-library byte comparison removed only the new contiguous helper/three-test insertion from the snapshot and accounted for the expanded import. The result exactly equals the base backend contract. Thus every prior test body, fixture, assertion and expected value is retained; the source test count increases from 18 to 21. This is a count of declarations, not an executed test count.

The retained cases cover normal poll/complete/ACK, readiness, one unschedulable drain, notify floor/safety poll, disconnected/owed-work gates, configuration reset, delay, completion errors/no replay, reopened original/coalesced Running custody, hidden legacy history, unavailable/malformed/ambiguous history, distinct-run liveness and terminal success/failure replay. App, delivery, production-wiring and composition-oracle files are byte-identical to base. Their pertinent callers and callback/source-capture controls were inspected as described above.

## CI oracle and safe capture

The affected `sqlite-native-path` step (`rust-desktop-runtime.yml:3823–3958`) still runs the real nine Cargo contract targets, including all of `automations_backend_contract`, with `--locked --no-fail-fast` and single-threaded test-harness execution. The new three exact function names match the three definitions in that target. Each must occur exactly once in an anchored `test NAME ... ok` line (`:3877–3893`); invocation alone, ignored output, a misspelled name or an absent selector cannot satisfy the new predicate.

The acceptance conjunction adds `poll_custody_passed` without removing cargo exit 0, capture exit 0, the prior required-invocation gates, successful nonempty target/child summaries with zero failed/ignored tests, the actual memory-consumer positive, the legacy-slot positives or exact subject-HEAD equality. The command array is executed directly; both pipeline statuses are captured immediately after Cargo/tee. The quoted Python here-document receives the statuses and command as arguments rather than evaluating captured test text as code. The job disables Cargo color, matching the positive-line capture format.

The added input paths are fixed repository literals: the Sand spec, backend transport and production lifecycle. Existing entries already include the consumer, backend contract, production backend, App and relevant caller/test sources. The receipt hashes these bytes with SHA-256, alongside subject commit/tree, fixed-U lock identity, run/attempt/job/OS, actual command, exits and log digest. It does not emit the raw source files, response payloads, credentials or environment dump. No new shell source interpolation, eval, broad filesystem search or artifact upload scope is introduced.

Workflow paths select this source/workflow/project delta. The exact positive-name addition is in the scoped `ocrm-config` dispatch / dedicated fusion-branch push step; a skipped run of that step is not poll-custody verification. Checkout and the Linux job bind HEAD to the event subject and, when provided, the expected frozen full SHA (`:3352–3368`). The report/fixtures stay under the existing evidence/run-owned roots; the final always-upload path preserves evidence on step failure (`:4554–4569`). The later same-subject MCP receipt consumer still requires the Automation receipt's successful result and matching commit/tree/upstream/log hash (`:4518–4545`).

The retained shipping-source oracle uses compile-time source capture and a syntactically checked, unique terminal cfg(test) boundary; its negative cases reject incomplete/duplicate boundaries and production content hidden after the test module. The typed backend wiring oracle binds the real Runtime, fire, callback, spawned scheduled attempt and returned result, with wrong-owner/dead-call/result-substitution controls. None of that source capture was weakened by this diff.

The receipt's new scope text accurately limits its claim to controlled-transport, in-memory response acceptance. Exact-H CI results, compilation, test exits/counts and complete artifacts have not been observed in this review and must not be inferred from the authored workflow.

## Findings and remaining acceptance boundaries

**Blocking findings: none. Actionable non-blocking findings: none.** No preference-only naming, formatting or speculative cleanup is being promoted into a required fix.

The following are explicit preserved scope limits, not newly discovered defects and not waived gates: deployed completion/ACK semantics and principal binding, durable settlement/restart recovery, terminal-alias history eviction, broader retained-state capacity/soak, real provider/platform/package acceptance and the complete OCRM G01–G12/AC01–AC24 set. The narrow patch neither claims nor implements those outcomes. Local source inspection cannot close them.

**Final verdict: APPROVE for the immutable five-file source review at `bd3dd6919c9bb6e71ba54ddf67919b9f3d324a1a` / tree `e9b2613c22285207c5c061592909f69bc830a8de`, with no required source fix from this lens. Executable verification remains pending and restricted to GitHub Actions at the eventual exact integrated H. No execution pass is claimed.**
