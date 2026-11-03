# SG74-MEDIA-v1 — round 1 independent general, lifetime, security, oracle and spec review

## Verdict

**No actionable findings in the frozen three-file delta.** I found no concrete correctness, lifetime, security, oracle-weakening, or scope defect that requires a source change. This is a source-review conclusion, not executable verification or migration acceptance. Snapshot `1cf5d76383b363535ba33864273d06a2bebe7b65` has not been executed in this review.

The two owner assertions now agree with the existing distinction between the complete owner projection and the visible transcript page. Their independently literal durable-row checks remain intact. Both media selectors exclude trait implementations only from inherent-owner candidacy; the subsequent safety checks still inspect the whole retained source. Paired-CRLF normalization is applied before literal mutation construction and at the affected oracle entry points, while the previous rejection cases and their relevant predicates remain present. The fixed 13-owner and 27-commit capture name sets are unchanged.

No code fix is requested. Current-H native Linux and Windows execution, the applicable admission/App/shipping-feature/platform gates, and the remaining MSU/OCRM requirements still need their own evidence. Historical H74 outcomes cannot be relabeled as results for this snapshot.

## Review identity, acquisition, and observability

| Item | Exact identity / observation |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Read authority | Bare repository `/tmp/ocrm-source-84120837.git`; `git rev-parse --is-bare-repository` returned `true` |
| Frozen base and snapshot parent | `b15e58957a597997363e1273f2f32a222365c310` |
| Frozen snapshot | `1cf5d76383b363535ba33864273d06a2bebe7b65` |
| Snapshot tree | `86f7d2de95296a494ac89f4b06f11b85fe16bbcf` |
| Manifest | `/tmp/shared-media-74-r1-manifest.json` |
| Complete delta | `/tmp/shared-media-74-r1.diff`, 531 lines, 32,962 bytes |
| Delta SHA-256 | `4f3ca38e7183442d94dd86c1f6c8a992487bc4b89af9a9b97a0bb74c922c19f6` |
| Scope verification | A fresh read-only `git diff --no-ext-diff --binary --full-index --find-renames BASE SNAPSHOT` was byte-identical to the declared delta. Git reports exactly the three modified paths below. |
| Starting checkout/index context | The handoff and manifest explicitly declare source-copy/private-index acquisition and an unborn bare HEAD, with no real developer checkout/index used. I read exact Git objects directly; I did not open or modify the private index, create a checkout/worktree, or change refs. |

The assigned lens retained the configured model. The session identifies itself as GPT-6 Astra Pro; the parent selected/requested **xhigh** for transaction uncertainty, receipt/recovery projections, and source-oracle acceptance risk. The child has no independent sampler-setting introspection, so the effective backend effort is not independently observable here beyond the supplied launch instruction. No alternate model or effort fallback was used or claimed.

This was a fresh review. I did not open diagnostic, worker, root-adjudication, peer, or prior-review reports. The required specification contains the root's historical TEST_FLAW statement; the conclusions in this report instead follow from the exact source and the original runtime artifacts identified below.

### Frozen file identities

| Path | Blob | Bytes | SHA-256 |
| --- | --- | ---: | --- |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `06b9c5c721aec53449f68684b352ebde89f130a6` | 17,241 | `d4481e0f03074d4c8dae2d0be1051247c58cd25e8db5d4cf7baf04ded042231b` |
| `source/host/src/extensions/session/agent_db.rs` | `8ffec064fad5e220fa1b2dadbb3fe9bc975f0d67` | 427,027 | `5cf63b89b1d2c1d9b4c283b60257376cddc087ac4051aa5936ff4cadf0bd2ea3` |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `85e3552d92e96037ea44a3923dac89a342166179` | 170,216 | `6415e02b5915074acba5e9b3ed5e7d894fb8f02f91d84bb0b4a4e46bebcf0802` |

All three identities were read from the snapshot and matched the manifest. Hashing complete files is identity evidence; it is not a claim that every unchanged line was semantically reviewed.

## Requirements and detailed assessment

### SG74-M01 — complete owner rows, independently checked storage, and visible page

The modified tests are `media_settlement_lost_commit_acknowledgement_retires_without_publication` at `agent_db.rs:9340–9375` and `media_settlement_fault_matches_three_identities_and_only_its_exact_owner` at `9551–9593`.

The existing owner API deliberately retains control records. `SandAgentDb::get_transcript_entries` calls `read_transcript_entries_from_db` (`5843–5857`, `7777–7789`), which decodes with `parse_owner_transcript_entry`. That decoder accepts `outgoing-media-settlement` records (`agent_db_serde.rs:169–187`). In contrast, `get_transcript_page` calls the real page reader (`agent_db.rs:5859–5880`); `LIST_TRANSCRIPT_PAGE_SQL` selects visible message kinds and the page mapper uses `parse_transcript_entry` (`agent_db_schema.rs:128–148`, `agent_db_transcript_pages.rs:45–98`). The Session presentation layer also explicitly hides settlement receipts (`production.rs:41–60`, `1290–1365`). The replacement expectations therefore reflect current production ownership without changing it.

The new `fixture.rows()` comparisons are **not a self-validating expected-value loop**. `Fixture::rows` opens a separate file-backed SQLite reader, selects every row ordered by `seq`, and decodes raw JSON (`9164–9174`). More importantly, unchanged `assert_committed_storage` pins that reader to this independent literal image (`9189–9204`):

| Row / field | Independently expected value |
| --- | --- |
| Entire row vector | Exactly two rows, in send-then-receipt order |
| Send row | Exact fixture entry, including `id = media-send`, kind, session identity, image type, and original path |
| Receipt `id` | Literal `outgoing-media-settlement:media-work` |
| Receipt `kind` | Literal `outgoing-media-settlement` |
| Receipt `sessionKey` | Captured admission session key |
| Receipt `mediaWorkId` | Literal `media-work` |
| Receipt `lifecycleRevision` | Captured admission revision |
| Receipt `mediaPaths` | Exactly the one original media path |
| Receipt `transcriptEntryIds` | Exactly `[media-send]` |
| Durable admission | Exact original admission record with only media work identity removed and media path vector cleared |
| Original file | Exact bytes `retained media original` |

The expected receipt is not built with the production receipt formatter. Extra rows, missing/extra/rebound receipt fields, reordered rows, lost media bytes, or an uncleared/rebound fence still fail the literal assertion. The lost-ACK test makes this assertion before reopening and after each of two independent open/close cycles. The other-owner test makes it before the invoking owner's unknown result and again after the newly added projection/page observations.

The new page calls use the real owner API, an unbounded time range, no cursor, and a limit of 10; the separately pinned two-row database cannot silently overflow that page. Each page must contain exactly `vec![entry.clone()]`, so receipt exposure, send omission, duplication, or send mutation is observable.

All original claim/applied-fault, exact connection retirement, live-handle, no-corruption-recovery, same-ID retry, write-generation, zero-success-publication, and repeated fresh-reopen assertions remain in place. The complete surrounding module retains the known-refusal, actual rollback denial, deferred COMMIT refusal, prior/poisoned transaction, exact fault identity, one-shot lease, stale-lease Drop, and scoped authorizer controls. In the second edited test the surviving owner's `!is_closed()` assertion remains; retirement still targets only the invoking connection.

Lifetime/security assessment: the added operations are read-only owner calls followed by a separately opened read connection. Their methods release the connection mutex before returning. No new guard is held across the independent reader or publication callbacks. No production receipt format, decoder, admission guard, cleanup authority, typed outcome, connection-retirement path, fault-port exposure, or error payload changes in this delta. The `agent_db.rs` modifications are entirely inside `#[cfg(test)] mod media_settlement_tests` (`9079–9080`).

The surrounding first-open recovery consumer remains distinct from a raw DB reopen: Session reads the durable fence/receipt, validates identity, and only cleans admitted paths after establishing its recovery facts (`production.rs:1640–1822`, `1824–1891`). The native DB tests do not by themselves prove execution of that Session/App path, and neither this report nor the revised spec grants them that broader claim.

**Assessment:** source conforms to SG74-M01; current-snapshot runtime evidence remains pending.

### SG74-M02 — inherent selection without hiding traits or widening the exception

At `openclaw_sessions_and_config_contract.rs:1703–1719`, `media_commit_inherent_impl` now excludes `item.trait_.is_some()` before counting candidates. The existing unique candidate, exact unqualified self type, no impl attributes, no defaultness/unsafety, and no generic/where-clause constraints remain. This removes the actual `impl Drop for SandAgentDb` from the candidate set without treating it as missing source.

At `1793–1869`, the second selector similarly skips trait implementations before reparsing an inherent `impl SandAgentDb` boundary. Without that second restriction, the token-boundary parser would still encounter `impl Drop for SandAgentDb` as though `Drop` were an inherent self type. The change therefore addresses both independent stages; it does not rely only on the first AST selector.

The substantive proofs remain unchanged:

- Each command method must be the exact complete method text at one actual inherent-method location. Removal followed by reparsing proves a one-method reduction in that specific owner, preventing a matching string/comment/nested/foreign method from receiving the exemption (`1722–1745`).
- The real owner must have the exact typed public signature, expected entry prefix, one directly invoked synchronous write closure of the expected return type, exactly three supported admission-guard calls, and the exact returned settlement tail (`1748–1870`).
- The rollback method is proved but remains in the residual inspection source. Only the exact media COMMIT method is removed, and only the previously proved sole `commands.commit(db)` call is replaced in the oracle-local image (`1873–1888`).
- The channel proof remains independently bounded. It retains the exact channel command method, real returned handling tail, failure-cleanup requirements, and `Some(1)` argument-taking commit-call rule (`2225–2528`).
- `channel_commit_call_count` still begins at the whole source cursor and recursively scans all token groups (`2315–2397`, `2438–2441`). The selector changes do not feed it a filtered AST or delete any trait block.

The new positive control appends benign trait implementations for both `SandAgentDb` and `MediaSettlementCommands`, requires successful bounded proof, and checks that the entire appended trait text plus the actual `Drop` remains in the returned source (`1953–1962`). Trait-only substitutions for both required inherent owners are rejected (`1986`, `2029`). Added trait-contained alias, raw-identifier alias, associated-call, and method-reference cases cover both types (`2110–2127`). Added trait-contained raw COMMITs cover both types (`2195–2219`).

The raw-COMMIT controls intentionally require the bounded adapters to remain provable and then require the extra forbidden statement to survive in the residual source. That is the correct predicate for this two-stage oracle; unconditional `None` would not prove retention. The final global consumer still asserts absence of the forbidden raw propagation after both proven adapters are removed (`2479–2483`).

**Assessment:** no trait-body inspection gap or broader COMMIT exemption introduced; source conforms to SG74-M02.

### SG74-M03 — one CRLF policy before lookup and mutation

`transaction_commit_source_image` (`1693–1701`) replaces only paired `\r\n` with `\n`. A source without such pairs is borrowed unchanged. It does not trim, fold whitespace, strip comments, edit identifiers, drop trait impls, remove source regions, or change Git configuration. The explicit mixed-line-ending control preserves the lone CR (`1942–1943`).

Normalization is applied at the media oracle boundary (`1873–1876`), both affected channel entry points (`2400–2405`, `2490–2493`), and, critically, before the actual-source fixture's `find`, `matches`, and `replacen` operations (`1896–1914`). Fixing only the parser input would have left the Windows multiline signature lookup and literal replacement fixtures ineffective; the new fixture boundary avoids that problem.

The fixture first requires the actual adapter proof to accept the normalized source, then requires the CRLF version to produce the identical complete residual source. It also requires unique real helper, owner, signature, prefix, and tail text boundaries. Every affected media mutation family now starts from that actual fixture. Its common rejection control requires a changed source, creates a distinct CRLF image, requires normalization to reproduce the entire mutated image exactly, and requires rejection of that image; each caller retains its corresponding LF rejection assertion (`1917–1925`, `1979–2185`). Raw-COMMIT retention controls check both line endings against the residual forbidden statement (`2210–2219`).

No previous media mutation was removed. Missing/renamed/qualified/duplicate/conditional helpers and owners, wrong visibility/return type, comment-only stand-ins, malformed source, changed COMMIT/acknowledgement/ROLLBACK/authorizer cleanup, extra/alias/associated/macro callers, discarded/conditional/changed settlement tails, asynchronous/move write closures, and changed unknown-entry handling all remain. Existing ordinary zero-argument RAII and unary-not positives remain. The complete existing channel negative sets are also retained; the changed standalone channel error-arm and helper-boundary families gain accepted LF/CRLF baselines and corresponding changed-fixture rejection controls (`2693–2751`).

The oracle remains a bounded source contract. This review did not execute `syn`, prove arbitrary Rust name resolution/macro expansion, or convert these fixtures into runtime typechecking claims. The relevant source comparisons and tokens survive the repair, and the new positive-before-negative checks prevent the specific already-rejected-baseline failure from yielding misleading media negative results.

**Assessment:** source conforms to the stated paired-CRLF policy and preserves the meaning of the affected mutations. Execution of the revised positives and negatives remains required.

## Original evidence examined; no new execution claim

Only original runtime artifacts were consulted:

- `/tmp/ar-74-linux-evidence/media-original.zip`
- `/tmp/ar-74-linux-evidence/media-windows-original.zip`

Their selected receipt fields both bind subject `74f11d3d31c6245ac5886dc0181460b2d4349744`, tree `aca91e1cfbedeecb039d442fb3528caac89873a3`, run `41580539491`, attempt `1`. Jobs are `sqlite-native-path` / Linux and `media-store-windows` / Windows. Both receipts remain `failed-or-incomplete`; neither is evidence of success for the new snapshot.

I read the two relevant logs in each archive from the `running N tests` section through EOF. They show the two send-versus-send-plus-receipt assertion failures on both platforms. The Linux commit group has 21 passed / 6 failed. Windows has 20 passed / 7 failed, adding `media_commit_oracle_rejects_missing_duplicate_or_conditional_owner` at its `actual media signature` lookup. These observations corroborate the new spec's factual failure descriptions without consulting its diagnostic reports.

| Original log | SHA-256 |
| --- | --- |
| Linux `media-settlement-owner.log` | `26ab3027d0bd03ff450927df814349347f9fb104f0b8ec2677b4126aa838fad8` |
| Linux `transaction-commit-owners.log` | `921b7317adf72cfad5ddd22b541f1e366a4d8beddc48214b913f642fbaaf1bc5` |
| Windows `media-settlement-owner.log` | `c73d70c1d9829a1b8404f8e888e048c4a5a997c63f4f0ead5221f6e4b3cffcb9` |
| Windows `transaction-commit-owners.log` | `2e25b24f4252140b8d4c9f063b62b9d596b62d2ed833fc3b2afef896fa922bf3` |

The original source blobs are the frozen base's corresponding blobs. Read-only byte hashing independently reproduces the capture's line-ending identities:

| Original source image | Bytes | SHA-256 |
| --- | ---: | --- |
| `agent_db.rs`, LF Git bytes | 426,548 | `9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b` |
| Same bytes with every LF changed to CRLF | 436,762 | `276b6c58a5260c7c0c3fd1561d65249eba307827767c894e149ba0e4bcb9fdb9` |
| Contract test file, LF Git bytes | 162,305 | `e61629a709e4154dda1297d3f5fa9fd13efc38c00d67c08edbc93fb70a27d32e` |
| Same bytes with every LF changed to CRLF | 165,691 | `119cf6ef914b4342719f63cfc7a032c754152b0c6c04a0f9d031d665ef83d946` |

The LF images match the original Linux receipt, and the derived CRLF images match the original Windows receipt exactly. The Git originals contain no CR bytes. This is byte-identity evidence for the reported checkout difference, not a local run of a source oracle.

## Fixed capture compatibility and retained gates

I read `capture_media_cleanup.py` as source; I did not import or execute it. Its fixed names at `204–256`, command selection at `517–521`, and exact result assessment at `358–388` remain compatible with this delta:

| Group | Fixed required names | Snapshot test declarations selected by the group | Removed / added since base | Missing / extra names |
| --- | ---: | ---: | --- | --- |
| `media-settlement-owner` | 13 | 13 | 0 / 0 | 0 / 0 |
| `transaction-commit-owners` | 27 | 27 | 0 / 0 | 0 / 0 |

This was text-based source enumeration, not compiled test discovery or a claim that any test passed. The newly introduced helpers have no `#[test]` attribute, so their names containing `commit` do not introduce extra selected tests. All additional controls are inside existing fixed-name tests.

The capture still requires command/capture exit zero, exactly one positive result for each required name, no unexpected positive name, exactly the expected successful summary, and no failed summary. It records actual working bytes plus normalized Git identities and checks inputs before/after. The workflow still invokes it in the Linux and Windows native jobs and uploads exact-subject evidence (`rust-desktop-runtime.yml:3341–3423`, `4556–4600`). Neither the capture, workflow, Cargo feature policy, nor a ledger/global completion flag is modified by this delta.

The added spec retains current-H Actions execution, original failed evidence, the independent App and ordinary shipping-feature gates, platform requirements, strict inventory acceptance, and final G01–G12 / AC01–AC24 / independent ACCEPT. It does not promote a declared source mapping into committed inventory acceptance or elevate historical negative-test `ok` lines into mutation-sensitivity proof.

## Exact semantic read scope and limits

All line references below are snapshot-relative unless explicitly identified as the original H74 artifacts.

### Complete delta and policy reads

- Entire declared three-file diff, lines 1–531, including every added/deleted/context hunk; independently matched to Git's complete binary/full-index diff.
- Full task spec `projects/openclaw-rust/docs/specs/media-settlement-unknown.md`, lines 1–253.
- Full root `AGENTS.md`, lines 1–395. Repository-wide path enumeration found no nested `AGENTS.md` governing any of the three changed files; the other nested instruction file is under the unrelated Mahayana TUI subtree.
- `docs/specs/openclaw-rust-equivalence-migration.md`, lines 637–720, including historical §17 and the full controlling OCRM-EXEC-02 §17.1–17.5.
- `projects/openclaw-rust/SOURCE_OF_TRUTH.md`, lines 1–64; `module-map.md` preamble/current-owner context at 1–40 and the specific M07/M23/M24/M29 rows at 68, 84–85, 90. Other search hits were navigation only, not asserted as full semantic reads.

### Code and consumer reads

- `source/host/src/extensions/session/agent_db.rs`: imports/outcome context 1–150; fault types, lease Drop, owner state/open/read lifecycle 850–1110; admission read context 1400–1440; retirement and adjacent spend/media/fault/settlement methods 4560–5110; transcript projection/page/window/tail context 5825–5955; Drop, media commands, rollback authorizer, channel commands/admission/history/failure/transaction/receipt code 6220–6740; transcript owner decoder and nearby retained replay 7765–7835; complete edited test module including its cfg boundary at 9079–9645, plus adjacent boundaries 9070–9078 and 9647–9675. The rest of the 10,226-line file was not fully read; whole-file keyword/name searches and identity hashes do not imply that it was.
- `source/host/tests/openclaw_sessions_and_config_contract.rs`: admission guard implementation and negative consumers 715–1149; complete related commit-contract region 1335–2863, including every changed function, all media/channel helpers, and all associated mutation arrays. Other function-name inventory hits were used only for capture enumeration. The remaining portions of the 3,526-line file were not represented as fully read.
- `agent_db_serde.rs`: 130–220, including complete visible/owner decoder predicates.
- `agent_db_transcript_pages.rs`: complete 1–156.
- `agent_db_schema.rs`: 1–165, including the exact owner-list and page SQL.
- `production.rs`: 1–65; 1275–1370; 1635–1895; 4050–4090. These include full receipt hiding, public presentation reads, first-open media recovery, recovery path verification/fence clearing, and the typed media append facade. Other production/App/Runner code was not fully re-reviewed because it is unchanged and outside this repair's source delta.
- `capture_media_cleanup.py`: 204–560, including all relevant fixed names, inputs, result parser, qualification cases, command capture, and invocation logic; read only.
- `.github/workflows/rust-desktop-runtime.yml`: native execution context 3300–3443 and 4545–4615; targeted source-search hits for the App test list and shipping/test-support artifact check. The full workflow was not semantically reviewed.
- Host `Cargo.toml`: 1–63; App `Cargo.toml`: 1–32. The `syn` full/parsing dev dependency and default-off/test-support dependency separation were inspected as declarations only.

### Execution and acceptance limits

No local or htch project build, test, lint, parser, schema check, generator, harness, or project-helper import/run occurred. Tools used for evidence were read-only Git/source reads plus Python standard-library archive/JSON/hash/text enumeration. The only written artifact is this report; reviewed repository objects, source copies, staging state, refs, and original evidence were not changed.

No new compiled syntax/typechecking, mutation execution, SQLite behavior, performance bound, App/Runner scenario, feature-off shipping build, source-inventory gate, or platform acceptance is claimed. No upstream full-source review or global migration completion is claimed. The original results are historical failures; the new snapshot remains unexecuted. These are explicit review limits, not asserted product defects or permission to skip the required current-H gates.
