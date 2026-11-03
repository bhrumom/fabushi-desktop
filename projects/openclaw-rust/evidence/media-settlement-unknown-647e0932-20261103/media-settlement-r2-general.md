# MSU-v1 R2 general, correctness, security and lifetime review

## Terminal verdict

**APPROVED — source-only focused re-review.** No blocking or non-blocking defect was established in the R1-to-R2 fixes or their inspected integration context. The four-path delta is coherent with the actual source contracts. This is an independent source-review verdict, **not** a compilation, test, feature-isolation, schema, runtime, recovery, platform or release acceptance result.

- Reviewer: `/root/media_settlement_general_r2`.
- Configured model retained: **GPT-6 Astra Pro**. Applied/inherited effort: **xhigh**. No model substitution, effort fallback, subordinate reviewer or external AI process was used.
- Whole-scope effort reason: the unchanged context includes typed transaction uncertainty, connection retirement, file-deletion authority, lock ordering, callback reentry, process-local terminal identities, restart recovery and a changed App fixture's compilation boundary.
- Review mode: fresh and blind; read-only for all reviewed source.
- Findings: **0 blocking; 0 non-blocking**. No speculative finding or preference-only cleanup is proposed.
- Executable verification: **NOT RUN**. Required exact-candidate evidence remains pending.

## Immutable subject and authority

Repository: `bhrumom/fabushi-desktop`; task `647e0932-bea4-4113-9a61-65ae3db58955`, task round 1, review round 2.

| Identity | Value |
| --- | --- |
| Frozen full-scope base commit | `d70f86f33b5e3c67545954b270373e1548809f4b` |
| Base tree | `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c` |
| Starting publication head | `871b033c5b9aabbe322cc7322af92648939db9b5` |
| Starting publication tree | `f641366eea64364f4d029487d04276c4c567790a` |
| Prior R1 review tree | `69f77b9dad3f6e42a7d7c53aa2df8f67c1f872a1` |
| Reviewed R2 candidate tree | `9892b15758904c23f3888ba8af1b5e429f29b59f` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Root authority blob | `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |

The reviewed local source is `/workspace/scratch/7d27ea239237/media-settlement-r2/`. The manifest retains the complete frozen 20-path scope, each candidate/base/prior blob identity, the exact four-path parent delta, and the acquisition owner's readback that all other parent blobs are preserved. I independently read the manifest and checked the focus files and patch bytes. I did not reperform the acquisition owner's entire remote-tree identity audit.

Pure byte/Git-object checks performed in this review:

| Input | Observed SHA-256 |
| --- | --- |
| `media-settlement-r2-manifest.json` | `58f73d55ff011d786c43e6ef1b2c766512632803b24ae4d7291f68d4dcc0c3fa` |
| Complete scope patch | `d813dbc3a88e2b685c9ee5bb50747d342edee75bb1cb0214ade9f61769ff4f22` |
| Four-path focus patch | `4d9d40a81134e67f5eaaf72e30ef150852bfafe6cde33beb475c30f9fcacf90a` |
| `final-assistant-media-custody.md` | `5fbdb25aec5808c0cf4121de18b6ccad8ae8e9c9194e69fc2d54c419c1b4eeed` |
| `parity-ledger.json` | `9355df10efd537cd95ad7fc3307adae86e7b4c38d1d0b6606fcabdd9588809e3` |
| `requirements-traceability-matrix.json` | `19acd0cae8995979888ab66674dbf019ff8ba1deb1f80ee89bd51e9b79be5e06` |
| `local_media_file_url_tests.rs` | `5bc24a39d3bbc77eab971ee0e58221f2a9c0ae6a226e1a35f311a69f80662106` |

`git hash-object` of the supplied root `AGENTS.md` returned the declared authority blob above. These operations establish byte identity only; they are not executable validation of the product or its checks.

## Focused fix adjudication

### 1. Explicit JSON imports in the actual App fixture

**Source disposition: correct.** `source/host/app/src/tests/local_media_file_url_tests.rs:3` now imports `serde_json::{Value, json}` explicitly. The fixture and its six consumers use `Value` in trait method signatures, stored transcript values and observation helpers, and use `json!` to construct real tool arguments and assertions. The App already declares `serde_json = "1"` in its normal dependencies. The explicit import fits the file's module boundary and requires no new runtime dependency or production export.

I read this fixture file in full, including `ObservedProductionSendSink`, `LocalMediaAppFixture::{new,open,bridge,reopen,dispose_owners,close}`, `assert_custody_idle`, all six tests, and the `#[cfg(test)]` module declaration in `main.rs`. I also inspected the actual shared `IdleRunnerOwners` construction/disposal and the workflow's exact six LFA test names. The two added/import-adjusted identifiers are consistently used. This is a source resolution assessment, not proof that the entire App test target compiles.

### 2. Focus timestamp type

**Source disposition: correct.** `local_media_file_url_tests.rs:201` changes the timestamp passed by the complete `LocalMediaAppFixture::open` function to `1_000.0`. The actual callee, `SessionRuntime::set_window_focused`, accepts `now_ms: f64` at `source/host/src/extensions/transcript/session_runtime.rs:315`. Its full body stores that floating-point timestamp and, when appropriate, passes it to `mark_agent_viewed`.

That adjacent source was read from immutable head `871b033c5b9aabbe322cc7322af92648939db9b5`, blob `bf7e3533d2691aaf204888fcdb64e1f548716c10`. The existing channel fixture calls the same API with its floating-point `FOCUS_AT_MS`. The correction preserves the intended numeric timestamp and the fixture's existing focus behavior; it changes no custody, file or admission oracle.

### 3. Notification-order clarification

**Source disposition: correct and materially more precise.** `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md:141` now distinguishes App publication/ACK/success counting from the existing DB listener callbacks.

The inspected ordering is:

1. `ProductionSendMessageSink::send_message` acquires the existing per-ID entry, checks collecting/admission state and seals local cleanup before history preparation and the potentially committing owner call.
2. The typed Runtime and Session facades reach `SandAgentDb::append_transcript_entries_and_release_media` without converting its inner uncertain result to `String`.
3. A successful DB `COMMIT` and autocommit check precede `drop(guard)`, transcript mutation publication and admission metadata notification (`agent_db.rs:5066` onward). These callbacks execute before the method returns to the App.
4. During those DB callbacks, the App still holds the per-ID entry guard. Same-ID resolution/send uses `try_pending_media` and returns typed unknown on contention before entering materialization/history/append. The index mutex has already been released. An abort cannot take that held entry and cannot delete its bytes.
5. Once the owner returns success and the entry ID is usable, the App calls `custody.commit()` at `main.rs:1220`, drops its entry guard, and then dispatches channel delivery, publishes the App event, fulfills ACK and updates activity. `CountingSendMessageSink` counts success only after its delegate returns `Ok`.

The revised sentence describes these two publication boundaries accurately. It does not introduce a new callback-order guarantee or authorize cleanup of an unknown attempt. The remaining MSU text still requires pending/unknown retention and terminal identity preservation. The inspected DB notification implementation catches listener unwinds, and the App's sealed custody remains conservative if a surrounding operation unwinds.

### 4. Evidence reference spelling

**Source disposition: correct.** The two ledger strings at `parity-ledger.json:36788` and `:36789` now name `fixed-u-contract.md` and `current-owner-audit.md`. The four RTM references at `requirements-traceability-matrix.json:1396`, `:1553`, `:2916` and `:3096` now use `fixed-u-contract.md` consistently for R23, R24, AC23 and AC24.

The frozen manifest's candidate-tree evidence-reference readback records the exact existing blobs:

- `fixed-u-contract.md`: `b143ea8d36f84c02793263bbadf39a2e138619bf`.
- `current-owner-audit.md`: `01b724729a97f33ad1ef63b937360022ee5e48b9`.

I read the entire affected responsibility row and all four affected RTM rows. The responsibility remains `implementation=implementing`, `read=partial`, `verification=not-run`, `acceptance=pending`, with the exact-H and full-equivalence blockers retained. The RTM rows remain `in-progress`. The changes repair navigation/provenance references without manufacturing execution or acceptance status.

I also read the relevant ledger/RTM consumer in immutable `projects/openclaw-rust/scripts/validate_p0.py`, blob `2bf4350bf6f4220ee3be1b82a21a65ea4ee52014`: its responsibility-reference, wiring/evidence, RTM and completion-flag checks remain separate. I did not execute that validator or claim that path existence establishes the evidence documents' semantic conclusions. Historical evidence contents were deliberately not read.

## Retained full-scope source context and read depths

Every manifest core path is retained below. “Scoped deep” means complete affected behavior and the necessary neighboring definitions/callers were read; it does not claim a full read of a large unrelated file. Test source establishes designed controls only.

| Frozen core path | Read depth and disposition |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Contract review of every changed hunk; scoped deep read of Cargo artifact feature observation, shipping/App build selection, exact LFA/MSU/SGF selection and result checks. No Actions result was read or inferred. |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | Full document; every focus line and the historical/current-contract distinction checked against source. |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | Full document; MSU-01 through MSU-10, non-goals, fault contract and evidence boundaries retained. |
| `projects/openclaw-rust/parity-ledger.json` | Complete affected final-custody responsibility row, focus references and separate read-scope risk correction; no whole-ledger semantic certification. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Complete affected R23/R24/AC23/AC24 rows, their statuses, blockers, references and responsibility links. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | Every changed hunk and scoped source of input binding, required names, assessment, execution selection and retained evidence envelope. Not executed. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | Changed contract inputs, media/Tool oracle selections, acceptance-count adjustment and declared support-only scope. Not a full-file review of unrelated capture campaigns. |
| `source/host/Cargo.toml` | Full manifest; default-off `test-support`, dependency boundary and existing test dependencies. |
| `source/host/app/Cargo.toml` | Full manifest; normal JSON dependency, same-package Host dev dependency with `test-support`, normal shipping Host dependency and target declarations. |
| `source/host/app/src/main.rs` | Scoped deep read of media resolver/persistence callers, pending-entry state, index/entry locking, poison handling, rollback, typed dispatch, success/unknown paths and post-commit consumers; test module registration. Full changed main diff also compared. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | Shared fixture creation/disposal and focused-window precedent; changed Responses/OwnedChild helper functions and visibility changes; selected neighboring observation helpers. Other channel scenarios were not reread wholesale. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | Full file and every R2 edit; all six actual fixture consumers. Durable idle/work-path and source-byte assertions preserved. |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | Full file; all eight designed App controls, real transaction-fault claims, exact durable images, ACK/channel/counter negatives, busy/poison/sealed-unwind handling and generated-child teardown/reopen assertions. No controls executed. |
| `source/host/src/attachment_paths.rs` | Complete custody state/methods/Drop plus all five custody tests and owning-root API context. Other path utilities were not reread in full. |
| `source/host/src/extensions/session/agent_db.rs` | Scoped deep read of complete media append/replay/admission transaction, command/fault lease and scoped FFI authorizer, retirement, notifications and all 13 media fault tests; relevant changed diff. Unrelated DB owners were not fully reread. |
| `source/host/src/extensions/session/production.rs` | Complete typed facade, filesystem mutation wrapper, first-open media recovery and path-verification/clear-fence helpers with owner-open context. Unrelated Session functionality excluded from deep-read claims. |
| `source/host/src/extensions/transcript/production_runtime.rs` | Complete media append plus adjacent ordinary/compatibility append contracts and imports; existing generated-entry preparation boundary. |
| `source/host/src/runner/tools/send_message_tool.rs` | Full file; construction/refusal handling, unknown-vs-known abort classification, counting delegate and emitted success/error outcomes. |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | Complete added media adapter/owner source oracle and eight media-oracle tests, including positive and mutation controls; changed existing channel/raw-COMMIT consumer functions inspected. No AST parser or oracle executed. |
| `source/host/tests/send_message_tool_contract.rs` | Complete added prepared-custody refusal helper and two controls, import change and adjacent persistence assertions. No execution claim. |

Additional authority/context reads were bound to the immutable starting head, whose unchanged blobs are recorded in the manifest: OCRM `SOURCE_OF_TRUTH.md` current authority/constraints sections (`3748d734b10c9ed5b2d948467fbaa9483f1dd306`), the main OCRM spec's applicable requirement/acceptance entries and complete current §17 (`69c9667af2328e20a1fba09ebf80c5aab4d5c336`), `module-map.md` ownership/placement and relevant Session/media domains (`15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3`), and the full current batch prompt (`5ff7894d245588ed539ab075b09215a2f1a82716`). Historical live/discovery identities in those documents were not treated as the reviewed source authority. No new fixed-U semantic reading or full upstream-equivalence conclusion is claimed in this focused round.

## Correctness, security and lifetime assessment

The inspected focus changes introduce no new runtime deletion operation, transaction adapter, mutable ownership index, public API, wire format or persistence schema. Their integration context retains the relevant safeguards: every attempted COMMIT error stays typed unknown; unconfirmed rollback and poisoned/prior active transactions retire the exact connection; Session preserves the inner typed result; App pending custody retains bytes through ordinary abort and Drop; same-ID entries cannot be replaced while older Arcs survive; committed/aborted entries retain terminal identity; and known refusal can clean only captured paths.

The local entry lock serializes the actual per-ID operation, while the global index lock is released before Session/DB or callbacks. The revised documentation accurately accounts for the DB-listener interval inside that operation. The fault lease keeps its callback state alive, claims an exact owner/identity once, and removes the authorizer before connection retirement. The source defaults keep this fault port out of ordinary shipping compilation; the declared workflow must still provide actual compiled-feature evidence.

Broader physical-file identity/read-scope, no-fence durability, arbitrary crash points, full upstream finalizer behavior, and real provider/channel/platform acceptance remain explicitly open in the supplied contract. This review does not promote them or issue speculative new blockers in their place.

## Execution gaps and completion boundary

No build, test, test listing, lint, parser, schema check, generator, capture, benchmark or acceptance run was executed locally, on `htch-runtime` or through Actions by this reviewer. No commit, push, dispatch or source edit was made. Only source/Git reads, byte hashes and this report write were performed.

Consequently the following remain unproved for R2: whole-App compilation and name/type resolution beyond source inspection; shipping feature-off/App-test feature-on Host artifacts; the selected eight MSU App, six LFA and three SGF cases; the native DB/custody/transaction/Tool suites and negative oracle controls; real rollback denial and lost-ACK fault application; race scheduling/teardown/reopen behavior; metadata schema/source-accounting execution; and all platform/provider/package/final-acceptance gates. Historical execution, if any, retains its original subject and was not used as current-R2 proof.

Blindness was preserved: no other reviewer report, prior review conclusion, root adjudication, worker report, historical evidence report content or mutable `media-settlement-work` source was read. Historical evidence filenames/blob identities were used only to assess corrected reference existence. No source file was modified, and no subordinate worker was started.

**Final source-review result: APPROVED, with zero established findings. Exact-candidate executable verification and wider acceptance remain pending.**
