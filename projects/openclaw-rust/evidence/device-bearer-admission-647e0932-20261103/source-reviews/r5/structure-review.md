# DFA R5 independent structure, reuse, performance and integration review

Decision: **APPROVE — SOURCE ONLY**. No actionable defect was found in the frozen original twelve-path DFA scope or the R5 integration delta. This decision approves the inspected source structure and contracts. It does not claim compilation, test execution, scoped G01/G02 closure, unit verification, media runtime recovery, or release acceptance.

This was a fresh independent review. I decided the findings without reading earlier reviewers' reports or their conclusions. I read the supplied scope manifests, required-case declaration and media identity manifests as scope/identity inputs, not as evidence that behavior passed.

## Immutable subject and acquisition

- Repository: `/workspace/scratch/29e7135d8bfe/fabushi-desktop`.
- Original scope/base and starting HEAD: `fa75f9ccd66e43f3a4d72ab47acdefaa832212f6`.
- R5 candidate tree: `63385ed61d0ef7074dab86baea7fab548224416d`.
- Published checkpoint: `9059d0c4493505851339ab228ec0d58a6023c904`. A direct Git commit read confirms that its tree is exactly the candidate tree and its sole parent is `f41a6d3c00150ba59453d14887b5b4914801d4f4`.
- Previous R4 tree: `f3a5f945195bd82f32537a6740817b5bdbf45857`.
- Publication/integration parent: `f41a6d3c00150ba59453d14887b5b4914801d4f4`.
- Fixed upstream declaration: `1440bb1771372334bea23d7642bb52deb7db1845`, tree `90c8aab39972da11cfad220361781d49e1bc6700`. This review does not establish upstream dependency closure.
- Original complete patch: **104,106 bytes**, SHA-256 `246be0aeb1cdc86153e0a2922f8bc1fca16a1b66b08eef38738f9c2016a3cc85`.
- R5 focused delta: **3,647 bytes**, SHA-256 `c06e34f77857158443dd639494a7c909851ac412649cc2b941d9936b89a3afde`.

I read every changed hunk in the complete twelve-path patch, including inherited integration/traceability hunks, and the complete focused delta. I independently regenerated their read-only Git diffs with `git diff --binary --full-index` against the stated immutable identities; their SHA-256 values match the supplied artifacts. I also hashed the supplied patch files and read their byte counts. Direct `git ls-tree` reads confirm all twelve candidate blobs below. Comparing the original twelve paths between R4 and R5 reports only the workflow as changed. No mutable product file was used as the review subject.

| Original frozen-scope member | R5 Git blob | Review disposition |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `25e80da72d20d4dbfed8ab6f2a50a3eff62b53af` | All complete/focused changed hunks; adjacent subject binding, owner/App commands, exact-name/count acceptance, receipt/input custody and Automation selection read. |
| `projects/openclaw-rust/docs/specs/device-bearer-approval-admission.md` | `f99fe2e81da2822ef7dd871bceba1215d3b0229e` | Entire DFA-v1 spec read, including DFA-01 through DFA-10, production boundaries, recovery, source evidence and acceptance limits. |
| `projects/openclaw-rust/parity-ledger.json` | `285f9ca672947d9ad054832557c0b4abf38a0d68` | All changed hunks and completion flags read; DFA existing-owner responsibility additions and inherited cron/media reference maintenance dispositioned. No full-ledger semantic audit claimed. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1bdbd1ca17b2ff334f365183c51bcd15c7ca0871` | All changed requirement/acceptance evidence-reference hunks and completion flags read; no status promotion in the diff. No full-RTM validation claimed. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` | All changed hunks; adjacent exact-H/input declarations, retained named owner/HTTP/stream selections, command mapping, result parser, negative parser controls and final aggregation read. |
| `source/host/app/src/tests/config_secret_boundary_tests.rs` | `8f3405e6335dac35e6df87674241ad4d2df2dc07` | Entire added DFA adapter/fixture/test/source-binding implementation and existing Settings fixture/helpers read. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `57a0a3393f1bb9560eb036687688656b1f3fe6c8` | Complete narrow diff read: explicit `serde_json::{json, Value}` imports and floating-point timestamp fixture argument only. Existing media assertions are not removed or rewritten by this patch. |
| `source/host/src/device_pairing_owner.rs` | `9825003cf1f7047027aa1b8e496c40f0c1693a23` | Full current file read, lines 1–2489, including all production paths and embedded tests. Deep review of admission, sealed proof, strict parsing, normalization, scope implications, currentness, locks, mutations and recovery boundaries. |
| `source/host/tests/device_pairing_revocation_authority_contract.rs` | `212b1067c4ffed3523aec5ebb2c3104eafe4c86f` | All changed hunks, full real-owner/raw-SQL fixture and adjacent late-admission/implication functions read. New three-case DFA owner suite and its large-authority/corruption/reopen controls read in full. |
| `source/host/tests/device_pairing_rotation_authority_contract.rs` | `a06e0593cf3be3673e91706e1636399dab90d0a8` | All changed hunks plus complete affected rotation assertion helper, empty-baseline case and live-grant narrowing/expansion cases read. |
| `source/host/tests/device_token_rotation_wire_authority_contract.rs` | `7c37e8093b8b7f2975eb031d76dae1b1419ed13c` | All changed hunks plus adjacent genuine-proof acquisition, independent-owner race, primary materialization fault/rollback/reopen and recovery code read. |
| `source/host/tests/device_token_rotation_wire_gateway_contract.rs` | `e7c42f88067d8d4695ff7ee356987e493b1f4224` | All changed hunks and full affected primary-only successor/withdrawal/reopen test read. |

Unchanged portions of the large workflow, capture file, external test files, ledger and RTM were not represented as whole-file deep reads. Complete changed-scope coverage is established by the immutable complete patch and the explicit path dispositions above, consistent with FMEP-SOURCE-01/OCRM §17. No original scope member is omitted.

## Rules, contracts and adjacent source read

I read root `AGENTS.md`, including independent immutable review, no local executable verification, existing-owner-first placement, exact-subject evidence and the source-review amendment. The file read hashes to `e4682748087759b2d4bb2b3a47916b6148e8a6f2`, the same blob in the candidate tree. There is no additional nested AGENTS file governing the reviewed Host, App, workflow or project paths.

I read the applicable current OCRM SOT authority/constraint/completion sections; main migration Spec §§12.2–12.3, §15.1 and current §17; FMEP §13; the current pairing task; the entire DFA spec; DPA's current proof/management contract; DVR's currentness, retained-stream, lifetime, execution and recovery contract; DRT's typed role/marker/management contract; DTR's normalization, performance, selected-primary and decisive-baseline requirements; and the incoming Automation checked-poll custody contract. Historical statements in these documents remain historical rather than evidence for R5.

Additional immutable identities used to assess integration:

| Adjacent authority/input | Candidate blob | Extent relevant to this decision |
| --- | --- | --- |
| `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` | Current evidence, gates, flags and §17 contracts. |
| `docs/specs/fabushi-deferred-ci-migration.md` | `359659cc3125d29ecb626cc74c5de40a9a1081a9` | FMEP source-depth/identity rules. |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` | Current authority, constraints and completion/subject separation. |
| `projects/openclaw-rust/management/tasks/P0-device-pairing-authority.md` | `a172905b4c7eefece4a500b8ef7337f475de963b` | Existing unit/owner and open authority/verification boundary. |
| `projects/openclaw-rust/docs/specs/device-pairing-authenticated-rotation.md` | `51937cd16ea0937cc72123471b05098b5f47f3f7` | DPA proof, role/grant, atomic mutation and no-fallback behavior. |
| `projects/openclaw-rust/docs/specs/device-pairing-revocation-continuity.md` | `0a594c04f26b1a7e82e285e0ad11357b0b478511` | DVR currentness/lifetime/exact-evidence contracts and existing documented recovery boundaries. |
| `projects/openclaw-rust/docs/specs/device-role-token-revocation.md` | `e46aaedf369dcc92f096117e7e601e69ab1cf952` | DRT currentness, strict role types and active marker contracts. |
| `projects/openclaw-rust/docs/specs/device-token-rotation-wire.md` | `c0cde59b089f4b6aef344218bcb1302a11826a86` | DTR credential spelling, membership complexity, selected-primary and decisive-baseline contracts. |
| `source/host/src/gateway_server.rs` | `2cc83cf387ae37c3b1c28663f72d9d0492e84ccd` | Trait, server close/accept-thread ownership, fresh authentication/guard ordering and actual command/report path. |
| `source/host/app/src/main.rs` | `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` | Actual Settings helper/dispatcher and Gateway delegates, durable pairing construction, shared-owner composition and server API injection. |
| `source/host/Cargo.toml` | `e42cc9f56cc98337a49622ae9117ebd9823ef820` | Existing crate/test-support/dependency declarations; no new dependency or target is needed. |
| `source/host/app/Cargo.toml` | `a2c58167a82657194b79aaf3e1e0d55577b2221c` | Existing binary and real Host/rusqlite dev-dependency seam used by the App fixture. |
| `docs/specs/sand-completion-ack-safety.md` | `5d1fe5c453e51445ac9c563191e3c6d571d81182` | Incoming AP01–AP04 checked-poll custody and evidence limitations. |
| `source/host/tests/automations_backend_contract.rs` | `11a9405442efa9acbbadfcdff879c0a996d1dee0` | Existing selected target; three new ordinary `#[test]` cases and real runtime/controlled transport assertions read. Text enumeration gives 21 `#[test]` cases in this target; this is a source count only. |

The incoming Automation receipt additionally binds `backend_transport.rs` blob `6dc4477c72a38179d71ab33013c9b7e2c3b89991`, `production_lifecycle.rs` blob `acc5d67aef35ede93a0f87e317f966013ee2eb8b`, and its already-listed consumer `sand_automation_fire_consumer.rs` blob `f7f6789daa21fb21cd09296be049b6e925dfb301`. These identities were enumerated for input preservation; this review does not claim a fresh full implementation review of that separate Automation batch.

## DFA requirement and structural assessment

| Contract | Source assessment and decisive locations in R5 |
| --- | --- |
| DFA-01 | `device_pairing_owner.rs:611` constructs a candidate from the matched token and returns it only after the shared validator at `:675`. `current_pairing_record` at `:1605` checks issuer, decisive row, complete approved-role parsing, selected role-map entry, exact role, active marker and token fingerprint. `:1572`–`:1599` strictly parses live/approved grants and checks the entire live and originally admitted grants. The private proof, rather than mutable public fields, drives these decisions. |
| DFA-02 | The lookup retains its existing connection mutex, and calls the private connection-taking `validate_current_device_authorization_in`. Public retained validation at `:1559` acquires the mutex once and delegates to the same function. There is no recursive lock, connection creation, parallel authorization owner, new persisted state, or write. |
| DFA-03 | Only the typed `GatewayCommandError::Unauthorized(_)` result becomes `Ok(None)` at fresh lookup. All other errors propagate. The retained public method still returns its typed error. The matched decisive parse/read paths use fixed `stored pairing authority is invalid`; HTTP authentication at `gateway_server.rs:1769` returns its actual error before dispatch. No numeric-status-only catch or corruption-to-empty acceptance was introduced. |
| DFA-04 | Canonical vectors retain ECMAScript scope spelling and first-seen order; role strings retain Rust `str::trim`. The same existing `role_scope_policy_allows` preserves literal nonoperator prefix+membership, explicit operator implications and empty-grant behavior. SQL-NULL-only approval fallback remains in `current_pairing_record`; unused malformed fallback is not decoded when present approval is decisive. Primary labels and opaque token/lifecycle metadata do not become currentness authority. |
| DFA-05 | The fresh and shared currentness code uses SELECT/parse/local collections only. It does not increment revision, update last-use, repair storage, rotate, substitute an unselected credential, or hold the lock through Settings work. Owner and HTTP fixtures compare complete raw paired/pending/machine state, not a reduced projection. |
| DFA-06 | The owner cases at `device_pairing_revocation_authority_contract.rs:1144`, `:1256` and `:1312` cover known durable bearers, complete-grant refusal, role withdrawal/map/entry mismatch, strict decisive types, baseline choice, redacted failures, explicit repair and fresh-owner boundaries. `config_secret_boundary_tests.rs:721` reaches the actual TCP server and actual App Settings helper with both configured-secret and secret-free listeners, refusal before command effects, and positive durable Settings commits. |
| DFA-07 | The runtime fixture delegates authentication to the real owner and configuration to `dispatch_public_settings_command`; it copies no authorization predicate. Separate mutation-sensitive source bindings at `config_secret_boundary_tests.rs:818`/`:840` bind the actual App chain read in `main.rs:1782`, `:4436`, `:4477`, `:4501`, `:13462`, `:15979`, `:17528` and `:17824`. Workflow selections require those cases and affected App compilation. Neither the test nor receipt equates this bounded HTTP/helper/source evidence with full App construction. |
| DFA-08 | Negative retained implication cases acquire an actually valid empty proof before injecting an invalid live grant (`revocation_authority_contract.rs:1106`). Valid implication cases acquire the intended live grant. DPA's empty-baseline special case names exactly `(TARGET, node)` and asserts one such refusal while preserving ordinary all-current behavior for every other caller (`rotation_authority_contract.rs:323`–`:418`, `:619`–`:629`). DTR captures genuine proofs before role withdrawal; after reopen it tests fresh refusal directly instead of using a foreign issuer as the sole oracle (`wire_authority_contract.rs:839`, `:1747`–`:1793`; `wire_gateway_contract.rs:1243`–`:1285`). |
| DFA-09 | The shared validator builds live/approved `BTreeSet<&str>` membership once (`device_pairing_owner.rs:1583`), then calls the existing role-scope predicate through local lookup closures. It does not scan a grant vector for each checked scope. The owner large test uses 20,000 literal node scopes, and the HTTP test uses 20,000 literal operator scopes plus write; both include an unapproved tail, unchanged-state controls, repair and reopen. There is no cap, persistent cache, wildcard shortcut or timing assertion. |
| DFA-10 | `normalized_authority_strings` at `:74` deduplicates via a local borrowed-string `BTreeSet`; the output vector still pushes only the first trimmed nonempty string. Every element passes `as_str()?`, including after an approving role has appeared. The large owner and HTTP controls include 20,000 distinct roles, duplicate/trimmed/empty entries and a final non-string, with fixed-error/no-effect assertions and subsequent repair/reopen. There is no early selected-role exit or role-count cap. |

The implementation preserves the existing ownership split: Pairing owns credentials/currentness; Gateway transports authenticated projections and applies its existing guards; App dispatches to the existing Settings owner; management mutation transactions and retained-connection currentness retain their own later boundaries. In particular, validating fresh admission does not replace the `IMMEDIATE` management transaction checks and does not promise cancellation of an operation already admitted by an unrelated owner.

The change introduces no product dependency, public proof constructor, schema, scheduler, permission store, cache, thread or second runtime. The full owner read also confirms that existing native/direct mutation paths remain separate from the new fresh-lookup absence mapping.

## Performance and fixture lifecycle

Let R be the role-array size, L the selected live-grant size, A the approval size and P the original admitted-grant size. Ignoring variable string comparison cost, role deduplication is O(R log R), and membership construction/checking is O(L log L + A log A + L log A + P(log L + log A)). Scope canonicalization already uses order-preserving set membership. These are source-level complexity conclusions, not measured latency or resource results.

The existing fresh lookup still enumerates paired token rows. The new check reads the selected decisive row and parses local authority, including some repeated selected token-map decoding through the established currentness helper. Those operations are linear in the serialized selected data and do not reintroduce the forbidden per-scope full-grant scan. This is an acceptable reuse tradeoff in the inspected contract; no claim is made that arbitrary database size has constant-time lookup or that quantified soak budgets have passed.

The real HTTP fixture's request and response handling uses owned loopback ports, bounded I/O/deadline and response-size controls, with no sleep-based readiness oracle. `on_command_complete` occurs before the HTTP response in the production Gateway, so the counter observations after response EOF do not race a later completion callback (`gateway_server.rs:2043`). The fixture does not mistake server accept-thread shutdown for closure of every request handler: its API Drop drops the actual pairing owner before signalling acknowledgement (`config_secret_boundary_tests.rs:549`), and stop/reopen awaits that acknowledgement before opening a new owner. Failed teardown retains the owned fixture instead of deleting a potentially live database (`:710`). No global cleanup or unowned process control was added.

Refusal witnesses record lookups/accepted/bound/dispatch/commit/completion/error counters, actual Settings bytes and directory entries, and complete raw pairing state. The reader control requires a real authenticated but insufficient grant and zero dispatch; malformed authority fails during authentication and produces no command report. Positive writer/native/other-device controls use the same real Settings helper and observe a durable value through a fresh Settings owner. These distinguish source-auth refusal from a fake HTTP status or a synthetic downstream mutation.

## R5 workflow union and evidence gates

The R5 focused delta changes only `.github/workflows/rust-desktop-runtime.yml:3898`–`:3959`. It adds three exact successful-name occurrences, conjoins `poll_custody_passed` into the existing acceptance predicate, adds the incoming spec/transport/lifecycle input hashes, and records the contract/list/counts/result/scope in the same receipt. No command, target, job, `if` condition, retry, timeout, concurrency setting, cache or worker is added or changed by R5.

The existing command at `:3844` already runs the complete `--test automations_backend_contract` target without a filter or ignored-test substitution, alongside its existing eight other targets and `--test-threads=1`. The new cases are ordinary unignored `#[test]` functions at `automations_backend_contract.rs:242`, `:407`, and `:476`:

1. `fire_runtime_invalid_poll_preserves_all_custody_until_checked_recovery`
2. `fire_runtime_invalid_poll_does_not_consume_notify_or_unschedulable_drain`
3. `fire_runtime_checked_poll_preserves_opaque_ids_duplicates_and_optional_fields`

Consequently the source command is configured to execute all three. I did not execute it and do not claim they ran. The successful-result regex requires a complete `test NAME ... ok` line, and each new name must occur exactly once. Missing, ignored, failed, truncated or duplicated successful records cannot satisfy the new conjunction. Existing cargo/capture exit-zero requirements, nonempty successful zero-failed/zero-ignored target/owned-child summaries, memory-tool positive, legacy-slot positives and exact HEAD equality remain intact at `:3881`–`:3906`. The receipt retains the actual command argv, run/attempt/job/OS, subject tree, source-input hashes and original log hash at `:3940`–`:3964`.

The original complete-scope workflow and portable-capture diffs only add DFA names and metadata to their relevant existing selections; no required DPA/DVR/DRT/DTR, retained, ordinary Gateway, Settings, media or App assertion is removed. The three owner names declared in the R5 `required-cases.json` are present in both `capture_portable_contracts.py:455`–`:457` and workflow `:2456`–`:2458`. The two fully qualified App names are present in the existing App execution selections at workflow `:1235`/`:1236` and `:4238`/`:4239`.

The portable owner gate still requires exact positive names and a complete single summary with exactly the required count, zero failed/ignored/measured/filtered results for the relevant full targets (`capture_portable_contracts.py:1094`–`:1104`). The workflow management receipt preserves the equivalent strict counts, original command/capture evidence and exact subject/run/attempt binding (`:2602`–`:2683`). The Linux App path still checks the actual ordinary shipping feature-off Host dependency, links its real test-support harness, selects one executable under the owned target root, lists each exact case, then requires one-passed/zero-failed/zero-ignored completion (`:4166`–`:4215`, `:4367`–`:4379`). The new DFA tests are not documentary names with no execution consumer.

The unchanged capture parser's explicit synthetic controls still reject missing/duplicate/ignored/failed/malformed/zero-test and command/capture-refusal evidence. Reading these controls is not execution credit. R5 strengthens the incoming Automation evidence predicate and does not weaken the stricter DFA/management gates. Its new receipt scope correctly limits Automation coverage to the existing volatile consumer and controlled transport; persisted restart recovery, deployed principal/ACK semantics and service acceptance remain open.

## Separate MSU3 preservation

Before making any preservation statement, I directly read the media member blobs in both the R4 and R5 trees and compared them with the original media manifest. All three identities are unchanged:

| Separate media member | Blob in both R4 and R5 |
| --- | --- |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `8aee21c69bab72824025e4304eec38388ba1e9f8` |
| `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `88a80214aa08a1d61b3bd290063069d5aa4d417e` |

This establishes exact source preservation of the separate MSU3 scope. I did not read a previous media review's verdict, re-review that separate implementation, inspect a new media runtime receipt, or transfer a previous pass. The twelve-path workflow's retained media selections are visible in the complete patch/adjacent gate, including the existing eight App media-settlement cases; R5 changes none of them.

## Findings, remaining gates and limits

**Actionable findings: none.** No blocker, must-fix item, or speculative cleanup request is raised. The source decision is APPROVE for this exact immutable scope, including the additive R5 workflow union.

All eight completion flags remain false in both inspected ledger and RTM: `baseline_identity_accepted`, `source_closure_ready`, `responsibility_mapping_complete`, `rust_implementation_complete`, `production_wiring_complete`, `ecosystem_compatibility_accepted`, `ui_accepted`, and `release_accepted`. Scoped G01/G02 authority/coverage and exact-H runtime evidence remain required before any unit-verification claim. No current-source review or blob equality closes those gates.

No project build, test, formatter, linter, generator, schema checker, validator, benchmark, acceptance harness or application was run locally. The review used file/source reads, text location/counts, hashes and immutable Git snapshot/diff operations only. The sole file written by this reviewer is this report. Mutable unrelated MCP work and every reviewed source/schema/workflow were left untouched.

Required subsequent evidence remains the authorized GitHub Actions execution against published H `9059d0c4493505851339ab228ec0d58a6023c904` (or a newly frozen replacement if the source changes), with the actual required names/counts, original logs/artifact custody, source-input hashes and separate bounded App/source-binding claims. No historical subject, manifest, old green status, absent runner capability or source-only approval supplies runtime pass credit. Full OCRM G01–G12/AC01–AC24, real ecosystem/device/platform, package/upgrade/rollback, resource/soak and independent final acceptance remain open.
