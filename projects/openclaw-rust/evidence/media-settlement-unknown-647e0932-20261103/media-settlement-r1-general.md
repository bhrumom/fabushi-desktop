# MSU-v1 round 1 — independent general, security and lifetime review

## Verdict

**SOURCE-ONLY BLOCKED**, for **F1**, a concrete defect in the new canonical evidence references. The frozen ledger/RTM names two reports that do not exist in its tree. Correct those references before accepting this source scope.

I found **no additional blocking correctness, security or lifetime defect in the changed runtime implementation** after tracing its actual producer, transaction owner, cleanup consumers and Runner uncertainty consumers. This is not an executed approval. No compilation, test, test listing, parser, linter, schema, generator or capture was run locally or through htch. The selected checks remain unverified until the required exact-H GitHub Actions campaign produces positive evidence.

Reviewer: `/root/media_settlement_general_r1`. Actual configured model: **GPT-6 Astra Pro**, inherited without a model override. Actual launch effort: **xhigh**, supplied through `collaboration.spawn_agent` with `fork_turns="none"`; the parent confirmed the launch metadata. This scope requires xhigh because it combines transaction uncertainty, destruction authority, concurrency, FFI lifetime, recovery, and evidence design. No subordinate workers were started.

## Immutable scope and independence

| Identity | Reviewed value |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Branch | `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Task / round | `647e0932-bea4-4113-9a61-65ae3db58955` / `1` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Review base and starting HEAD | `d70f86f33b5e3c67545954b270373e1548809f4b` |
| Base tree | `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c` |
| Frozen candidate tree | `69f77b9dad3f6e42a7d7c53aa2df8f67c1f872a1` |
| Scope manifest | `/workspace/scratch/7d27ea239237/media-settlement-r1-manifest.json` |
| Candidate / base contents | `/workspace/scratch/7d27ea239237/media-settlement-r1/` and `/workspace/scratch/7d27ea239237/media-settlement-r1-base/` |
| Complete core diff | `/workspace/scratch/7d27ea239237/media-settlement-r1-complete.patch`, all 4,608 lines |
| Diff SHA-256, independently read back | `6c78d91836e62a3ae34a8ed8edea30a31660461bb72e7ffa37fbc8b8ab3e82db` |
| Current instruction authority | `media-settlement-authority/AGENTS.md`, blob `e4682748087759b2d4bb2b3a47916b6148e8a6f2` |

The root supplied the verified, untruncated recursive-tree acquisition and full-blob readback facts. I read that manifest and the complete nonhistorical diff, then inspected the immutable changed contents and the adjacent sources listed below. This report does not independently reperform the root's GitHub acquisition.

The later publication notice identifies commit `71b039ae860c45e982dfbc8b1f401143d49d2454`, parent `30088609db15f562a8f97f545d11bc4f9718113a`, tree `3c30ec8ac78d0b8b549267a4b794a7cb177b8b23`, with unchanged reviewed core blobs. That notice did not change my frozen base, candidate or review scope, and is not execution evidence. The root also confirmed that the eight publication-parent-only files do not supply either missing F1 path.

I did not read the mutable `media-settlement-work/` tree, other review reports, root adjudications, previous work reports, or the contents of the ten historical evidence reports. Historical filenames and object identities were read only as manifest metadata, including the existence comparison needed for F1. The root separately owns their substantive audit. No reviewed source was edited. This report is the only file I wrote.

## Finding F1 — canonical evidence references name absent reports

**Severity: P2 / medium. Classification: source/evidence-link defect. Blocking for this frozen source scope.**

**Affected locations and symbols:**

- `projects/openclaw-rust/parity-ledger.json`, `OCRM-RESP-MEDIA-FINAL-CUSTODY-001.evidence_refs`, candidate lines 36788–36789.
- `projects/openclaw-rust/requirements-traceability-matrix.json`, the evidence references for `OCRM-R23`, `OCRM-R24`, `OCRM-AC23` and `OCRM-AC24`, candidate lines 1396, 1553, 2916 and 3096.

The ledger adds references to these two filenames under `projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/`:

| Referenced filename, absent from frozen tree | Existing manifest filename | Existing blob |
| --- | --- | --- |
| `media-settlement-fixed-u-contract.md` | `fixed-u-contract.md` | `b143ea8d36f84c02793263bbadf39a2e138619bf` |
| `media-settlement-unknown-current-audit.md` | `current-owner-audit.md` | `01b724729a97f33ad1ef63b937360022ee5e48b9` |

All four RTM rows repeat the first absent filename. Neither absent path appears in the full d70 base-tree inventory; neither appears among the complete candidate additions/changes in the scope manifest. The actual filenames and identities above are explicitly present in manifest lines 189–203. This comparison requires no access to report contents.

**Concrete trigger and consequence:** a reviewer or evidence consumer follows the newly registered responsibility's fixed-U contract/current-owner audit reference, or any of the four RTM references, against this exact candidate. There is no file to retrieve. The canonical responsibility-to-evidence chain therefore does not resolve, despite the actual reports being published under different names. This is a present defect in the submitted canonical metadata, rather than a preference about naming.

The row is honestly `mapped`, `implementing`, `partial`, `not-run`, and `pending` at ledger lines 36913–36919. I do **not** claim that the current non-final structural gate necessarily fails: the existing `source_review.py` evidence-existence checks apply when completed source-review metadata is validated, and this row does not claim such completion. Nor does the broken reference demonstrate a runtime failure. It nevertheless needs correction before this batch's canonical evidence registration is accepted; carrying an unresolvable reference forward is not traceability.

**Minimal fix:** after the root confirms the intended report identities in its separate historical audit, replace the six incorrect JSON reference strings with the already published `fixed-u-contract.md` and `current-owner-audit.md` paths. Alternatively, publish the intended exact report objects at the referenced paths and bind those identities explicitly. The first option is smaller and avoids duplicated reports. Keep the responsibility's pending/not-run statuses; fixing links must not promote verification or acceptance.

**Falsification / focused review:** inspect the next immutable tree and prove that each of the six references resolves to its intended existing blob. Review the two edited canonical metadata fields and their adjacent responsibility/RTM rows. No runtime-code change is required by this finding. I did not execute a validator to discover it, and do not report an invented validator result.

## Runtime assessment

### Transaction outcome and exact owner retirement

`SandAgentDb::append_transcript_entries_and_release_media` retains the original admission, exact replay-content, receipt and fence-CAS body inside its existing `BEGIN IMMEDIATE` transaction. I read the whole candidate method, including all three well-formed admission checks, transcript entry validation, receipt replay checks, insertion count check, receipt insert and exact admission update (`agent_db.rs`, 4799–5104).

The new outcome handling has the necessary distinction:

| Observed owner state | Source behavior |
| --- | --- |
| Invalid arguments, closed owner, or clean BEGIN refusal before writes | Ordinary prewrite error |
| Poisoned connection lock or a pre-existing live transaction | Retire the exact held connection; typed unknown |
| BEGIN failure that leaves a transaction active | Attempt cleanup; retire; typed unknown |
| Body failure before COMMIT | Ordinary error only after successful ROLLBACK and observed autocommit |
| Body failure with failed ROLLBACK or a remaining active transaction | Retire; typed unknown |
| Any attempted COMMIT error | Typed unknown and retirement, even if subsequent rollback succeeds or the connection is already in autocommit |
| COMMIT success without autocommit | Attempt cleanup; retire; typed unknown |
| COMMIT success with autocommit | Release the connection guard, then publish existing successful DB notifications and return success |

The shared retirement helper sets `closed`, takes/drops the exact `Connection`, and releases that owner's live-handle registration (`agent_db.rs`, 4576–4585). Every newly classified unknown exits before successful transcript/admission notification. The method does not open another connection, retry the append, synthesize a settlement receipt after failure, or infer non-commit from a successful cleanup. The ordinary refusal branch remains usable, as required by the positive and refusal fixtures.

### Typed propagation and downstream publication

The Session facade returns the inner `Result<usize, AgentDbProjectionError>` through its existing filesystem mutation boundary. Its outer `String` errors arise from acquiring/admitting/opening the owner before the operation; `Ok(owner.append_…(...))` keeps the DB error typed (`production.rs`, 4063–4084). The filesystem helper simply acquires the established boundary and calls the operation; it adds no fallible post-operation transformation that could mislabel a committed result as a known refusal (2294–2313).

The Runtime media method maps history/planning failures to ordinary typed variants before append and propagates the Session result with `?` (`production_runtime.rs`, 1797–1837). The App classifies `TranscriptCommitUnknown` by enum variant, not error text. Its compatibility path conservatively retains owned media on a legacy `String` failure after entering the append, without claiming a new compatibility receipt. Untracked text keeps its prior behavior (`main.rs`, 1138–1198).

The App seals custody before the potentially committing Runtime call. It does not restore cleanup for typed unknown, or for the impossible successful-result-without-ID case. On confirmed success it commits local custody and releases the entry guard before the App's channel dispatch, Gateway event, ACK fulfillment and activity update (`main.rs`, 1133–1273). The counting wrapper increments only when the underlying send returns `Ok` (`send_message_tool.rs`, 168–172).

The DB's own successful notification occurs before its result returns to the App; local custody is already sealed during that interval. The map lock is not held across that notification, and reentry for the same media identity refuses through the entry's `try_lock`. I found no introduced path that re-enables destructive cleanup after a committed/unknown owner result.

### Cleanup authority, busy identities and destruction

`OutgoingMediaCustody` now carries explicit rollback-eligible, owner-settlement-pending and committed dispositions. Ordinary rollback and Drop drain only eligible custody. New path tracking and absorption refuse a sealed destination without reopening its prior ownership. Only the App's confirmed precommit-refusal branch invokes the explicit restore-and-rollback operation. First-open recovery constructs a separate fresh eligible custody after its durable checks (`attachment_paths.rs`, complete file; `production.rs`, 1757–1865).

The App's single pending-media index stores stable `Arc<Mutex<PendingOutgoingMedia>>` values. The map lock is used only to find/create and clone the entry; the code then uses nonblocking entry acquisition. Same-ID busy operations return typed unknown promptly. Poison recovery may disarm cleanup but never resume materialization or SQL. Terminal entries are retained, preventing an old Arc from referring to one attempt while the index installs a new entry for the same ID (`main.rs`, 725–896).

Collection caches the exact source string, reuses its resolved value without a second capture, and permits distinct image sources until sealing. All collection/send checks use the retained session/revision identity before new work. Explicit abort does not remove or release pending/unknown/committed entries. A clean refusal clears only the current attempt's eligible files; a cleanup failure leaves the durable recovery fence. The sink is constructed inside the existing run composition, rather than creating a process-wide new registry (`main.rs`, 1004–1224 and 11697–11719).

The Tool build-error change is necessary for the busy case: an unknown resolution error is returned without a destructive abort callback. Ordinary construction errors retain abort behavior. The two facade controls deliberately use a synthetic typed error to test that Tool decision only; they do not claim to establish SQL uncertainty or Drop/recovery behavior (`send_message_tool.rs`, 405–413; `send_message_tool_contract.rs`, 569–688).

### Fault support and FFI lifetime

The default-off feature and App dev-dependency expose only the narrow fault lease. The lease is bound to one owner and exact canonical session, revision and work strings; the slot is claimed once under the connection boundary before BEGIN. A mismatch does not consume it. The connection-to-slot lock order is consistent, SQL runs after releasing the slot, and dropping an old lease checks Arc identity before clearing a still-unconsumed slot (`agent_db.rs`, 879–948 and 4723–4794).

The COMMIT adapter first executes the real SQLite COMMIT and uses `?` before applying the lost-ack fault. A real COMMIT error therefore cannot be replaced by the test fault. The rollback adapter installs a scoped SQLite authorizer and executes the real ROLLBACK. The callback denies only `SQLITE_TRANSACTION` with the operation `ROLLBACK`, marks the observation, and otherwise returns `SQLITE_OK`. The guard retains the state Arc and borrowed connection and unregisters the callback before returning to the retirement decision (`agent_db.rs`, 6243–6340). Product classification does not use `was_claimed`/`was_applied` observations. I found no exposed general SQL executor, persistent raw handle, cross-owner slot, or new production retry authority.

### Runner uncertainty and first-open recovery

The unchanged tool guard marks `DownstreamOutcomeUnknown` before completing the receipt for a returned `ToolOutcomeUnknown`. Stream-attempt custody remains sticky, disallows checkpointing/retry while unknown, and is consumed by the existing shell/generated-history owners. The existing recorder validates actual observed SendMessage results and does not invent success on an unknown result. I traced these adjacent consumers through the exact Git objects listed below.

Cached Session access does not turn a retired owner into media recovery: the cached branch validates the existing owner and skips first-open media reconciliation. Fresh opening, after safe disposal, uses the existing receipt/fence identity logic. Receipt-confirmed media is retained; uncommitted captured paths are cleaned only by the recovery owner after its checks. Malformed identities remain errors/fenced. The compatibility fallback has no newly invented durable recovery guarantee. These boundaries are explicit in MSU-05 rather than being silently treated as complete media equivalence.

## Causal tests and retained guards — source assessment only

I read the complete eight-case App test module, all thirteen new DB cases, all five custody cases, the complete nineteen-case SendMessage contract file, and the twenty-seven selected `commit` test bodies/source oracles. The latter selection includes the existing configuration CAS race whose name also matches the filter. Counts here describe the authored/selected source, not successful execution.

| Surface | What the source establishes as the intended oracle |
| --- | --- |
| 13 DB cases | Real COMMIT followed by lost acknowledgement; real inserted send prefix followed by receipt refusal and actual rollback denial; deferred foreign-key COMMIT failure with clean/refused cleanup; real error not replaced by lost-ack injection; poison/prior transaction; clean busy BEGIN; exact replay; slot mismatch/owner/lease isolation; authorizer scope; independent durable rows, receipt/fence, owner closure and notification checks |
| 8 App cases | Same real command faults through the private production sink, clean refusal control, compatibility retention, exact source-cache/multiple-image behavior, busy winner/loser ordering, poison/sealed unwind, and an actual generated Runner consuming lost-ack uncertainty |
| Drop and reopen | Weak Arc assertions ensure observers do not keep the sink alive; independent file-backed readers distinguish actual committed state from lenient closed-owner reads; disposal is checked separately from two fresh-owner recovery observations; source/unrelated sibling bytes are preserved |
| ACK and success consumers | Actual persisted ACK obligation/token is installed and compared byte-for-byte on refusal/unknown; a normal positive send proves it can be fulfilled; counters, channel requests/effects and publications are checked separately |
| Generated Runner | Real generated SendMessage uses the exact fault lease; downstream-unknown custody remains, success/retry/checkpoint authority is absent, actual error wire is observed, and canonical root/blob/journal/mirror/provider-checkpoint bytes remain unchanged; owned child and repeated fresh-owner observations are scoped to their actual lifecycle |
| 5 custody + 19 Tool cases | Direct file cleanup/retention/absorb tests plus existing wire/schema/counting/media facade cases and the two new construction-error classifications |

The receipt-refusal trigger proves the inserted send prefix by selecting the send ID from the incoming receipt and refusing only if that actual row exists in the transaction (`media_settlement_unknown_tests.rs`, 242–257). Both unknown App command cases require the lease to be claimed **and applied**, inspect independent durable state, verify exact connection retirement, then prove retry/abort/Drop do not change it (354–418). These are causal controls in their source design, not mock unknowns being counted as transaction evidence.

The busy-case scheduling wrapper is observation-only: it pauses the loser after the production sink returns busy, releases the winner before the bridge handles the error, and makes an erroneous Tool abort observable. The generated case uses a controlled Responses producer and the actual Runner path. It does not claim live-provider or external-delivery compatibility.

The source-COMMIT extension proves the exact `MediaSettlementCommands` methods and the sole media owner's prefix, directly invoked transaction body and returned handling tail, then removes only that proven method/call from the oracle-local representation. The existing channel exception and global raw-COMMIT prohibition continue to run. Negative cases cover extra/aliased/raw-identifier/macro callers, absent/duplicate/conditional helpers or owners, changed commit/rollback/retirement handling, discarded results, and hidden raw-COMMIT owners elsewhere. The original eight-owner admission oracle still expects eleven actual calls, including three in media settlement; dormant closures/futures and conditional attributes do not supply missing calls (`openclaw_sessions_and_config_contract.rs`, 723–1145 and 1568–2699).

The LFA fixture now accepts compact Committed/Aborted markers instead of raw map emptiness, while retaining empty durable work/path assertions and requiring each marker to have no pending custody, owned media, work ID or source cache. A busy or poisoned marker fails the idle check. I read all six retained LFA cases; their actual source/file/durable assertions remain present. The three SGF case selectors remain required.

## Build and capture assessment

Host `test-support = []` is absent from default features. The ordinary App dependency remains feature-free; its dev-dependency enables the narrow support feature. No locked dependency version change is in this scope.

The changed workflow retains Cargo JSON from the actual ordinary App `cargo check` and App test build. It binds one Host library artifact by target name/kind plus exact manifest/source paths, requires `profile.test == false`, requires support absent for shipping and present for the App test build, and preserves the artifact fields. The actual App executable is then selected from the build output and required to belong to its owned target directory. This is an observation of compiler artifacts, not an inference from a dependency tree (`rust-desktop-runtime.yml`, 4132–4181).

Eight MSU names are added to the existing exact App listing/execution loop alongside the existing six LFA and three SGF names. A missing/ambiguous exact test, nonzero command outcome, or a summary other than one passed/zero failed/zero ignored refuses acceptance (4295–4335). I found no unchanged hardcoded seventeen-test gate missed by the new nineteen-test SendMessage count: the portable capture updates its exact summary tuple and required names, while the earlier macOS integration receipt checks named positives rather than that old count.

The media cleanup capture selects the 13/5/27/19 source suites, records exact subject/input identities and command/log outcomes, requires every expected positive exactly once with zero ignored/failed results, and retains negative qualification controls. It does not claim native Host execution proves the private App path. The portable capture retains the existing contract sets and adds the eight media source-oracle positives and two Tool positives. The ledger remains pending/not-run and keeps both exact-H evidence and full-equivalence blockers. F1 concerns its unresolved references, not a false passing-state promotion.

## Read depth and ranges

All twenty declared nonhistorical paths were covered through **every changed hunk** in the complete diff. The table records additional whole-file or targeted implementation reads. “Deep” means the changed responsibility and necessary surrounding control flow were read; it does not falsely claim every unrelated line of the approximately one-megabyte App file, ten-thousand-line DB file, large workflow or global ledgers was read.

| Changed path, relative to repository | Additional read scope / depth |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Deep changed capture/build/selection flow; 1625–1690, 1800–1990, 3944–4185, 4280–4375, 4540–4595, plus every changed hunk |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | Complete 1–207 |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | Complete 1–207 |
| `projects/openclaw-rust/parity-ledger.json` | All changed hunks; complete new responsibility 36764–36942 and adjacent status/upstream fields; exact evidence-reference searches |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | All changed hunks/affected requirement rows; exact evidence references at 1396, 1553, 2916, 3096 and surrounding identities |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | Complete 1–560 |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | All changed hunks; subject/input setup 1–65, selection 310–384 and 625–698, final acceptance 1073–1125; targeted exact-name/count searches |
| `source/host/Cargo.toml` | Complete declarations |
| `source/host/app/Cargo.toml` | Complete declarations |
| `source/host/app/src/main.rs` | Deep materialization/sink/owner body 605–1335, construction 11650–11755, all changed hunks; targeted producer/call-site searches |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | All changed hunks; fixture/ownership 1–135, 400–700, 830–890; shared producer/child helpers 1695–1838 |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | Complete 1–782 |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | Complete 1–1035 |
| `source/host/src/attachment_paths.rs` | Complete 1–393 |
| `source/host/src/extensions/session/agent_db.rs` | Deep errors/support/opening 1–81 and 875–1035, retirement 4552–4585, full fault admission/transaction 4715–5104, close 6091–6115, full adapters/FFI 6237–6340, complete new test block 9077–9638; every changed hunk |
| `source/host/src/extensions/session/production.rs` | Reads/recovery 1270–1340 and 1610–1885, filesystem boundary 2250–2345, admission 2450–2545, capture/release 2640–2808, append consumers 3945–4115, shutdown 6320–6384; all changed hunks |
| `source/host/src/extensions/transcript/production_runtime.rs` | Full affected planning/media/legacy methods and adjacent boundary 1710–1885; all changed hunks |
| `source/host/src/runner/tools/send_message_tool.rs` | Complete 1–484 |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | Deep admission guards/oracles 710–1145; transaction/source-COMMIT region 1320–2720; selected existing CAS test 178–266; all changed hunks |
| `source/host/tests/send_message_tool_contract.rs` | Complete 1–806 |

### Adjacent immutable inputs

The other checkout was used solely as a Git object source. Before each adjacent content read, the `fdb0f6acb06165b6f2796961186e06a2de98c229` blob identity was compared with the exact d70 base-tree entry. No mutable files there were read. Relevant acquired adjacent identities and actual reads include:

| Path | Verified blob / inspected scope |
| --- | --- |
| `docs/specs/openclaw-rust-equivalence-migration.md` | `69c9667af2328e20a1fba09ebf80c5aab4d5c336`; §17, 671–720, and relevant authority/gate/owner searches |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `3748d734b10c9ed5b2d948467fbaa9483f1dd306`; 1–85 |
| `projects/openclaw-rust/module-map.md` | `15b93d08a4426d1f09c5a5a66dd3e62a288bb3b3`; relevant Session/Runtime/media/Tool ownership and §17 precedence entries |
| `source/host/src/runner/tools/mcp_meta_tools.rs` | `923288198b898083cbfe5da9a1f76e9f309ee516`; 1–240, real guarded tool operation and downstream-unknown handoff |
| `source/host/src/runner/stream_attempt.rs` | `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f`; 1–62, 138–220, 396–485 |
| `source/host/src/runner/generated_agent_turn_stream.rs` | `4617051c700665287008c8cec225ae333af02387`; 330–430, 580–670 |
| `source/host/src/runner/production_turn_run_shell_adapter.rs` | `d0fe2714923338dd717284f08f05eef438db69d9`; 165–240, 344–578, 775–832 |
| `source/host/src/runner/turn_agent_composition/tool_call_recorder.rs` | `8078fbc13ccc66ae70a479d9d18fb6380db2767e`; 1–388 |
| `source/host/src/extensions/session/production_agent_store.rs` | `e8afdca4bb983d6c3f7f0b062720db142739e576`; checked cache/durable-root validation 513–596 and relevant owner searches |
| `source/host/src/extensions/transcript/ack_obligations.rs` | `f105486efdbf0c1bef78903d9846e7da6d0e3074`; 294–380 and token-check searches |
| `source/host/src/transcript_mutation_events.rs` | `842d4d06d8f0d788b0b730ea3aef942d75f673a2`; complete 1–58 publication/registration body |
| `projects/openclaw-rust/scripts/validate_p0.py` | `2bf4350bf6f4220ee3be1b82a21a65ea4ee52014`; 1–190 and 210–370, relevant mapping/evidence validation |
| `projects/openclaw-rust/scripts/source_review.py` | `143e310f263d474a3a711a8b992013f5ded74b85`; targeted identity, mapping and evidence/completion validation, especially 189–343 |
| `projects/openclaw-rust/scripts/expand_inventory.py` | `49b7313151e9d5602a9edf9de4c82050b65cc98d`; targeted source mapping/evidence flow, especially 90–164 |

The object-only symbol inventory for `OutgoingMediaCustody` and the two changed facade methods finds the same four existing production files covered above. The source review did not discover an additional unmodified caller that would require adapting the changed signatures. Object identities of the existing lockfiles and relevant schema/storage declarations were checked as additional dependency context; this does not claim an independent full dependency implementation audit.

## Execution gaps and disposition

1. **No executable result exists from this reviewer.** Cargo compilation, feature resolution in the compiled artifacts, SQLite fault application, concurrency scheduling, child-process cleanup, test listing/counts, source-oracle parsing and JSON/schema validation remain exact-H Actions obligations. I have not promoted a test source assertion into a result.
2. **The required positive set remains intact in source:** 8 actual App MSU cases, 13 DB cases, 5 custody cases, 27 selected source-COMMIT cases and 19 SendMessage cases, plus the preserved admission/channel/SGF/LFA and other affected gates. The Actions receipts must show exact named positives, accepted command/capture exits, input/log identities and the fault application assertions. Missing/ignored/flaky results remain non-passing under the existing governance.
3. **Feature separation is authored, not observed here.** The new Cargo artifact observer must actually accept the shipping Host library without support and App test dependency with support. I did not run either build or substitute a test-support-enabled build for shipping evidence.
4. **Acceptance is narrower than migration completion.** Controlled local responses/channel ports, file-backed fixtures and source oracles do not establish live-provider/channel acceptance, hardware crash durability, packaged-platform acceptance, original interrupted-turn restart authority, or complete fixed-U final-media/read-scope equivalence. The submitted spec and pending ledger statuses retain those gaps.
5. **F1 is independently actionable from source now.** Resolve the six canonical references without changing product behavior or verification status, then review that exact fix delta. Runtime test execution can follow the existing campaign on its newly frozen H; no local execution or speculative redesign is needed to fix this finding.

The source judgment therefore remains **BLOCKED for F1**, with the runtime assessment otherwise free of demonstrated blocking findings and all execution acceptance explicitly outstanding.
