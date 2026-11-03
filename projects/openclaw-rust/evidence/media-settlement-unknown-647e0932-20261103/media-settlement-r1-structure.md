# MSU-v1 round 1 — independent structure, reuse and performance review

**Source verdict: BLOCKED — one P2 traceability defect in the changed OCRM ledger/RTM.** No additional production structure, reuse, lock-topology or performance defect was established in the frozen scope. This is a source-review verdict, not executable verification or final migration acceptance.

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, review round 1. Review is fresh, independent and blind. I did not read `media-settlement-work/`, historical report contents, other reviewers' reports, root arbitration, or prior findings. Historical filenames and object identities were used only from the supplied manifest/tree to check reference resolution.

## Execution identity and authority

- Actual launch metadata supplied by the parent: `collaboration.spawn_agent`, `fork_turns="none"`, model **`gpt-6.1-sol`**, reasoning effort **`xhigh`**. The complete scope combines transaction uncertainty, cleanup lifetime, same-ID concurrency, Runner custody, source oracles and feature/build reach; xhigh is appropriate. No model/effort fallback was reported.
- Repository: `bhrumom/fabushi-desktop`; branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.
- Exact base and starting HEAD: `d70f86f33b5e3c67545954b270373e1548809f4b`; tree `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c`.
- Frozen candidate tree: `69f77b9dad3f6e42a7d7c53aa2df8f67c1f872a1`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`; tree `90c8aab39972da11cfad220361781d49e1bc6700`.
- Input manifest: `media-settlement-r1-manifest.json`; immutable candidates in `media-settlement-r1/`; base inputs in `media-settlement-r1-base/`. The complete 4,608-line, 20-path patch was read. Its byte SHA256 was independently read as `6c78d91836e62a3ae34a8ed8edea30a31660461bb72e7ffa37fbc8b8ab3e82db`. SHA256 reads for all 20 candidate files match the manifest values.
- Current root authority: `media-settlement-authority/AGENTS.md`, Git blob independently read as `e4682748087759b2d4bb2b3a47916b6148e8a6f2`. Its independent-review, existing-owner, effort/model, source-depth and Actions-only rules govern this review. The complete base tree and changed-path manifest show no closer AGENTS file governing these paths.
- Both changed feature specs were read completely: MAC final-assistant custody and MSU-v1, including all MSU-01 through MSU-10 requirements. Current SOT entry/constraints, module-map owner/boundary sections, and OCRM-EXEC-02 §17 were read from immutable Git objects identified below.
- Later checkpoint publication messages from the parent do not change this review's candidate identity or extend its source verdict to any later integration delta.

No build, test, test-list invocation, compiler, parser, lint, schema checker, generator, benchmark or capture was executed locally or through htch. All operations were source/text reads, immutable Git reads, diff reads or byte hashes, except writing this report. No agents were spawned.

## Finding

### SRP-R1-01 — P2: new responsibility evidence references name absent files

**Affected source:** `projects/openclaw-rust/parity-ledger.json:36788-36789`, the new `OCRM-RESP-MEDIA-FINAL-CUSTODY-001` row; and `projects/openclaw-rust/requirements-traceability-matrix.json:1396`, `:1553`, `:2916`, `:3096` for OCRM-R23, R24, AC23 and AC24.

The ledger adds these two evidence paths under `projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/`:

- `media-settlement-fixed-u-contract.md`
- `media-settlement-unknown-current-audit.md`

The exact 30-file candidate manifest supplies `fixed-u-contract.md` (blob `b143ea8d36f84c02793263bbadf39a2e138619bf`) and `current-owner-audit.md` (blob `01b724729a97f33ad1ef63b937360022ee5e48b9`). Neither of the two longer referenced basenames exists in the exact base-tree manifest, and neither is added by this candidate. The four RTM changes repeat the absent `media-settlement-fixed-u-contract.md` path. This conclusion uses filenames/identities only and does not rely on historical report contents.

**Concrete trigger and impact:** following the new responsibility's fixed-U or current-owner evidence reference fails because the target path is absent. The four requirement/acceptance rows also cannot resolve their new fixed-U evidence. The existing `source_review.py::validate_source_reviews` consumer checks `all(evidence_exists(ref) for ref in resp["evidence_refs"])` for responsibility links of completed source reviews (immutable blob `143e310f263d474a3a711a8b992013f5ded74b85`, lines 280–285); `validate_p0.py` supplies a repository-confined actual-file predicate (blob `2bf4350bf6f4220ee3be1b82a21a65ea4ee52014`, lines 239–246). These names therefore cannot support valid source-review closure. The present `mapped`/`partial`/`not-run` state does **not** prove that the current structural Actions run will already fail; I make no such execution claim.

**Minimal fix:** replace the two ledger strings with the actual `fixed-u-contract.md` and `current-owner-audit.md` paths, and replace the fixed-U basename in all four RTM additions. Preserve the existing incomplete statuses and global flags. Do not copy or create duplicate reports to satisfy the erroneous aliases. Recheck the exact corrected reference paths and run the existing applicable authority checks in Actions on the accepted final H.

## Production structure and lifetime assessment

The change extends existing owners and keeps one durable truth. `SandAgentDb::append_transcript_entries_and_release_media` still owns transcript inserts, hidden settlement receipt and media-fence CAS in one SQLite transaction. Session and TranscriptRuntime now propagate the typed DB result through the existing filesystem serialization boundary; they add no new persistence owner. App interprets `TranscriptCommitUnknown` at its existing SendMessage sink, and `attachment_paths` only manages local deletion authority. No new scheduler, runtime, registry, database, retry loop, external dependency or production factory is introduced.

The existing transaction's admission, replay identity/content, inserted-entry and fence-CAS checks remain in the closure. COMMIT errors now retire the exact connection and return unknown even after cleanup; rollback refusal/non-autocommit and a pre-existing transaction also retire it. A clean BEGIN refusal and a confirmed pre-COMMIT rollback remain ordinary refusal. Unknown paths return before DB success publication. The success path drops the connection guard before notifications. The retirement helper is reused rather than duplicated.

The App index lock is held only for lookup/insertion and Arc cloning. Per-ID operations use the entry's `try_lock`, so same-ID reentry reports typed unknown without waiting; an unrelated ID does not wait on that entry. Owner calls happen without the index mutex. The entry guard remains held across preparation/settlement so an operation cannot inspect Collecting state and then perform an unchecked materialization/absorb after another operation seals it. Poison recovery can seal deletion authority but cannot resume SQL or publication. Explicit abort retains unknown/busy/committed entries. The typed Tool build-error branch avoids a delayed destructive abort after a busy result.

The entry is sealed before history allocation and the potentially committing owner call. Only a confirmed ordinary refusal restores rollback eligibility; unknown and unwind keep it sealed. A successful entry-shape check commits local custody before App delivery/event/ACK/activity publication. The App drops the entry guard before its outward callbacks. DB notifications occur inside the admitted owner call; same-ID callback reentry is safe because it uses try-lock, not a blocking reacquisition. No new inverse acquisition order was found.

Terminal entries remain in the single App map, so old Arcs cannot coexist with a replacement entry for the same ID. Committed/Aborted entries clear cached resolved-source contents and active work/custody data; Committed custody also clears paths. Session/revision strings, vector/hash-table capacities and the tool-call key remain until sink destruction. That is a real per-attempt memory cost, but the production sink is allocated inside the routed turn's existing composition (`main.rs:11696` onward), rather than as a process-global registry. Unknown entries intentionally retain exact path/source/identity data needed to prevent unsafe reuse. I found no concrete unbounded process-lifetime cache or performance regression warranting a blocker. The spec requires terminal retention, so deleting those entries as an optimization would be incorrect.

The collecting cache reuses an exact source without rematerialization/capture while admitting multiple distinct images before seal. It rechecks session generation before cache reuse. The legacy no-fence owned-media branch explicitly retains bytes and reports unknown because its String API cannot establish a safe refusal; it does not invent a receipt or claim full compatibility durability.

The fault seam is subordinate to the existing DB owner. It is cfg-gated under `any(test, feature="test-support")`, reserves one exact owner-local slot, matches all three canonical identifiers, claims under the connection boundary before BEGIN, and releases the slot before SQL. Lease Drop takes only the slot and compares Arc identity, so a consumed old token cannot cancel a new arm or reverse product custody. Lock order is connection then fault slot; Drop never takes the connection. Real COMMIT executes before acknowledgment replacement, while a real COMMIT error is preserved. The rollback authorizer has a borrowed connection and retained user-data allocation and is removed synchronously before retirement. I found no new general SQL executor or shipping fault enablement surface.

## Reuse, causal evidence and source-oracle assessment

The new App tests reuse the existing file-backed channel fixture, real Session/Transcript/AgentDb owners, controlled channel port and existing generated Responses/OwnedChild helpers. Helper visibility is widened only within the sibling test module. The changed LFA idle assertion retains its durable work-ID/path checks and now inspects terminal entries with try-lock; pending, busy and poisoned entries cannot pass as idle. Existing SGF behavior is preserved while its Responses fixture delegates event selection to the new narrow callback.

Source inspection establishes that the 13 DB cases include real deferred-COMMIT failure with clean/refused rollback, real successful COMMIT/lost ACK, receipt-trigger failure after an inserted send, prior live transaction, poisoned connection, exact replay, slot isolation, lease lifetime and authorizer cleanup. Separate SQLite readers inspect committed rows and the retained fence. The App cases assert fault claimed **and** applied, retained source/sibling/owned bytes, zero channel/ACK/count effects, same-ID refusal and retained Arc identity, actual last-Arc destruction, two fresh owner observations and positive new work after safe recovery. The generated Runner case reaches the real tool guard and requires DownstreamOutcomeUnknown custody, no retry, no successful completion hook, no fabricated canonical history and fresh-owner persistence. The native mock-error tests are correctly scoped to Tool build-error classification; they are not presented as proof of SQL uncertainty.

The new media source oracle separately proves the exact AST-owned COMMIT adapter, rollback method, owner signature, transaction prefix and returned outcome tail. It leaves the admission closure in scope and requires its three actual guard calls. It removes only the proven method/call from an oracle-local syntax image before invoking the original channel oracle. The retained channel helper method, returned handling tail, unique caller rule and macro/alias rejection logic remain intact. The global raw-COMMIT prohibition is retained. Negative controls cover extra/aliased/macro callers, duplicate/foreign/conditional helpers and owners, discarded/altered handling, changed unknown classification/retirement and raw owners elsewhere. This is legitimate composition of two bounded adapters, not a larger global allowed-call count.

The selected capture declarations retain the original media, LFA, SGF and channel controls. The native capture adds 13 DB, 5 local-custody and 27 transaction/retained-COMMIT named positives; its existing assessment rejects missing, duplicate, extra, failed, ignored, zero-test, incomplete-log and nonzero-capture evidence. It continues through all 19 declared command groups and hashes inputs before/after. The portable capture adds media source controls and two Tool controls and adjusts the whole SendMessage target count from 17 to 19 without reducing the other two target counts. The existing workflow invokes native capture on macOS, Linux and Windows. The actual private App eight-case selection and retained LFA6/SGF3 run in the existing Linux App step; no claim of full App behavior on other platforms is supported by that step.

Cargo adds no dependency package: the Host feature is default-off, and App enables it only as a dev-dependency feature on the same Host package. The Linux step reuses its ordinary shipping check and App test build. Its Cargo JSON observer binds exactly one ordinary Host library artifact to both actual manifest and source paths, requires `profile.test == false`, and checks feature absence/presence for shipping/test builds respectively. This is materially stronger than a dependency-tree approximation, but the actual artifacts remain pending execution.

The existing Runner consumer was checked directly: the tool worker marks DownstreamOutcomeUnknown before completing its local receipt; StreamAttempt keeps that result sticky and refuses checkpoint/retry admission; generated failure history requires settled custody; the invocation recorder rejects duplicate SendMessage IDs and refuses unobserved unknown-result publication. The new App state does not replace those existing authorities.

## Exact changed-source coverage and read depth

Every changed hunk across the 20 paths was read. “Deep” below means the changed behavior and its complete owning functions/tests plus relevant adjacent callers and consumers were read; it does not claim a line-by-line read of the unrelated portions of the pre-existing 996 KB App or 426 KB DB files. Large ledgers/workflows were reviewed at every changed block and owning consumers/configuration. All 20 complete files were byte-hashed against the manifest.

| Candidate path | Candidate blob | Read depth / disposition |
| --- | --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | `239c054aaccd42c0e5bc5e7ca0607431bcf400b7` | Deep changed command selection, JSON feature proof, receipt logic, inputs, Linux guard and all three native capture call sites. |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | `bab905e889a472027a906b551a874081131cc85e` | Complete spec; correction and retained acceptance limits. |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `e42ea96e12c4647c23d98154fabd4a87749ea5c7` | Complete 207-line spec, all MSU requirements. |
| `projects/openclaw-rust/parity-ledger.json` | `9e56c9f65259420b5cdde83e24bfbd1b35064cad` | Every changed block; complete new responsibility, owner/dependency/source/evidence/status contract. Finding SRP-R1-01. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1921355c3643650b835fe1cefce8d653758492b4` | Every changed row/block; R23/R24/AC23/AC24 linkage and incomplete state. Finding SRP-R1-01. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | `d5376d94f569b112c5778efb529eb64d2b863766` | Deep changed selections/input declarations and complete input/assessment/qualification/run/main consumers. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `172485b8437151256fb5058fcec010f9a0665cc8` | Deep changed input/selection/count/receipt logic and assessment consumer. |
| `source/host/Cargo.toml` | `e42cc9f56cc98337a49622ae9117ebd9823ef820` | Complete manifest, feature default and dependency direction. |
| `source/host/app/Cargo.toml` | `a2c58167a82657194b79aaf3e1e0d55577b2221c` | Complete manifest, ordinary versus dev dependency feature. |
| `source/host/app/src/main.rs` | `7ec9bd8a22fd595f63cb3f4b632673e298294dfa` | Deep sink model/state, all changed methods, complete send tail, resolver context and production construction/composition lifetime. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | `f6139af6bde9829f7e9147e6a2028ec77680bf42` | All changes; fixture boundary and complete shared Responses/OwnedChild methods; callback behavior/reuse. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | `60e6be8a61d446706ac2188366be9bff1f08cb71` | All changes with inspection/idle assertion contexts and unchanged durable checks. |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | `f525710de5833559de608e1cb8aaa89a8d58e2b8` | Complete 1,035-line causal fixture/test source. |
| `source/host/src/attachment_paths.rs` | `6e1a2b8eb5752e005bbce30ed0b6eaaacfda0cda` | Complete changed custody model/methods/Drop/tests, persistence and root interfaces. |
| `source/host/src/extensions/session/agent_db.rs` | `6c27e9bb9f665b1160457b9acc29dd5d64fba289` | Deep complete media transaction/fault lease/command/FFI guards and all 13 new tests; adjacent retirement/close/publication helpers. |
| `source/host/src/extensions/session/production.rs` | `c5912cab4f95d3a75fc13cb264a21b1deef9a733` | Deep typed facade plus owner cache/open/identity, first-open media recovery and capture/release consumer functions. |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` | Complete changed typed append and adjacent legacy append contracts. |
| `source/host/src/runner/tools/send_message_tool.rs` | `86a9e0e394b0145173bc033ceb757b7fb10835e5` | Complete file: construction/abort, result observation, counting and return consumers. |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `ba7dde9c12eb14cb0a0615a49a013a2e126b860d` | Complete added media oracle and changed original channel tests; adjacent admission and channel parser/normalization/unique-call consumers. |
| `source/host/tests/send_message_tool_contract.rs` | `3027419ca36d652ae2ab8b2c1fff16197903fa6c` | All additions and adjacent fixture/count contracts; narrow returned-error classification versus real DB scope. |

Additional sources were read only via `git show fdb0f6acb06165b6f2796961186e06a2de98c229:<path>` after comparing their blob identities to the exact d70 base-tree manifest:

| Unchanged source | Verified blob | Read scope |
| --- | --- | --- |
| `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` | Current §17, including risk-depth, single-ledger and Actions/final-gate requirements. |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `3748d734b10c9ed5b2d948467fbaa9483f1dd306` | Current entry, authority, fixed-U and existing-owner constraints; no historical status promoted. |
| `projects/openclaw-rust/module-map.md` | `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3` | Entry/owner boundaries and relevant Session/media/storage mappings. |
| `projects/openclaw-rust/scripts/source_review.py` | `143e310f263d474a3a711a8b992013f5ded74b85` | Actual evidence/reference validation consumer, lines 188–295 and relevant evidence rules. |
| `projects/openclaw-rust/scripts/validate_p0.py` | `2bf4350bf6f4220ee3be1b82a21a65ea4ee52014` | Ledger/inventory/RTM and evidence-file consumer, lines 196–270. |
| `source/host/src/runner/tools/mcp_meta_tools.rs` | `923288198b898083cbfe5da9a1f76e9f309ee516` | Actual guarded worker result/unknown/receipt path, approximately lines 180–234. |
| `source/host/src/runner/stream_attempt.rs` | `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f` | Custody snapshot/retry reason, sticky result tracker, effect admission/completion and checkpoint fences. |
| `source/host/src/runner/generated_agent_turn_stream.rs` | `4617051c700665287008c8cec225ae333af02387` | Failed-history custody checks and final custody capture consumers. |
| `source/host/src/runner/production_turn_run_shell_adapter.rs` | `d0fe2714923338dd717284f08f05eef438db69d9` | Attempt admission and retry custody barriers. |
| `source/host/src/runner/turn_agent_composition/tool_call_recorder.rs` | `8078fbc13ccc66ae70a479d9d18fb6380db2767e` | Receipt lifetime, unknown qualification, duplicate-ID admission and completion. |
| `source/host/src/extensions/inference/provider_session.rs` | `b2513672ca563e72efe0f4b56440b9aaf8f98c60` | Typed unknown definition and relevant conversion/cancellation consumers. |

## Remaining execution gaps and disposition

Fresh exact-H Actions results were pending at review time and were not supplied as successful evidence. Source inspection supports the declared command/test reach; it does not establish that compilation, exact names/counts, feature observations, FFI command faults, App child processes, recovery behavior or platform timing actually passed.

Before verified batch closure, obtain the accepted final-H native capture on the declared platforms (including all 13 DB, 5 custody and 27 retained transaction controls), the complete retained media/Tool/native groups, portable oracle controls, ordinary Host feature-off shipping artifact, feature-on ordinary Host dependency artifact in the App test build, and all eight actual Linux App cases plus retained LFA6/SGF3. Bind actual exits, named positives, log/input hashes and compiler artifacts to that H. Broader live providers/channels, arbitrary process crash points, full read-scope/physical identity, no-fence durability, packaged platform delivery and final OCRM G01–G12/AC01–AC24/independent acceptance remain separate unaccepted work as the specs state.

The only requested source correction from these three lenses is SRP-R1-01. It is a traceability repair; no architecture refactor, terminal-index eviction, new owner or additional fault framework is justified by this review.
