# MSU R6 — independent general, security and lifetime source review

## Terminal verdict

**SOURCE APPROVED for the frozen R6 candidate tree `4b94e525d6790d6f355ea6e2113db02cb5e5a46f`.** I found no actionable source defect in the complete publication delta, the complete focused fix delta, or the affected owner/caller boundaries inspected below. There are no blocking or non-blocking findings to request a source fix from this review.

This is a terminal source-review verdict, not an executed-test, Actions, runtime, release or full OCRM acceptance verdict. Current-candidate executable evidence remains pending. The historical failure logs retain their original identities and results.

Task: `647e0932-bea4-4113-9a61-65ae3db58955`; task round 1; focused review round R6. Lenses: general correctness, security and lifetime/concurrency. These lenses are applicable because the changed source oracles constrain transaction uncertainty, cleanup authority and the actual App owner boundary. The configured model was retained. The assignment selected `xhigh` for the interacting transaction/cleanup oracles and actual owner types; this worker inherited its session configuration and did not launch another worker or alter model/effort settings.

## Exact review authority and identities

| Subject | Exact identity |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Existing writer branch | `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Frozen candidate tree | `4b94e525d6790d6f355ea6e2113db02cb5e5a46f` |
| Publication parent | `4e80e161c6cc539f4c120d64f91421619a57def1` |
| Publication-parent tree | `efb2a28c92d2f24cd4f6bbbd84d86709c0b2f028` |
| Original complete-scope base | `d70f86f33b5e3c67545954b270373e1548809f4b` |
| Original-base tree | `4c4c9cfd3f5067fef57dca530042d149a8e2fc5c` |
| Prior-round tree | `fc752f889ce0d402f6f5af9911d9777c7cab7999` |
| Fixed OpenClaw U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed-U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Applicable master specification blob | `69c9667af2328e20a1fba09ebf80c5aab4d5c336` |
| Scope manifest SHA-256 | `893fd4fd0b5ef6787935956ea5156a6a0bcd9ed7dcda0f49ffe1f8d0f9cabe88` |
| Complete original-base patch SHA-256 | `4b4acb024bf58910a71ac019d0ee50f61fd07186140b3a43779046d2a663dcae` |
| Focused fix patch SHA-256 | `dd598fdce68ca64f590fa2dd77324115e4164e2061580cd4e06e4e5c5129c9ef` |
| Publication patch SHA-256 | `f73ad17effa380bfc1bd8ab3224d306284e47aa9f1467958b896a3ad6ded43d7` |

The immutable source root is `/workspace/scratch/7d27ea239237/media-settlement-r6/`. The scope manifest is `/workspace/scratch/7d27ea239237/media-settlement-r6-manifest.json`; the supplied tree proof is `/workspace/scratch/7d27ea239237/media-settlement-r6-tree-proof.json`. I independently hashed the manifest, the three patch files and all 22 scoped snapshot files; their SHA-256 values agree with the supplied manifest. I also obtained the four principal changed Rust/master-spec blob identities with read-only `git hash-object`. The root-tree binding and preservation of all other publication-parent leaves are supplied by the immutable tree proof; I did not reconstruct a full repository tree locally.

The exact load-bearing blobs are:

| Path | Blob |
| --- | --- |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | `32a7218874c4b03e27a9872adf339186fab50f28` |
| `source/host/tests/background_wakes_production_contract.rs` | `1c38f0bfde3e23cb0d227d1c4dfb4bd5b4d0b713` |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | `fc1f056a318fee390e0c8e67cdda361d94f0c3a5` |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | `e89c27654614752fd4700e6506735bee3bde25a8` |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | `8235f203e5ac0797f73cb2d9002b64eeb1f69110` |
| `.github/workflows/rust-desktop-runtime.yml` | `25e80da72d20d4dbfed8ab6f2a50a3eff62b53af` |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | `f5f3d5624568cb553441b8ca0c0f63aff76ebeae` |
| `projects/openclaw-rust/parity-ledger.json` | `285f9ca672947d9ad054832557c0b4abf38a0d68` |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | `1bdbd1ca17b2ff334f365183c51bcd15c7ca0871` |
| `source/host/app/src/main.rs` | `ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df` |
| `source/host/src/extensions/session/agent_db.rs` | `87186bc4b246d88a5ce51ebda0623b76f4425d3d` |
| `source/host/src/extensions/transcript/production_runtime.rs` | `9f15ddbdebe0ec7fe612975e28e9a7d5d0c3beb6` |
| Context `source/host/src/extensions/transcript/send_pipeline.rs` | `12f58e2923cd909b9564b99e0a6393955a0945e0` |
| Context inventory shard manifest | `911019b1c4c3b629a4425610bf941caf88a97e18` |

The inventory context retains canonical digest `d0049087bdbcc8eecdc8d0a69bd48bfc7d70688f8917122198a6e15c80863d8a` and the same fixed U. The review does not replace that inventory authority with source-text counts.

## Scope and actual reading depth

I first read the supplied repository `AGENTS.md`, including its independent-review and Actions-only execution rules. I read applicable master-spec §17 and the complete current MSU-v1 and MAC-v1 documents. I also inspected the exact-parent Source of Truth's authority/execution material and the module map's relevant existing-owner and media/lifecycle contracts through read-only GitHub file APIs. Historical project snapshots were not treated as current runtime or acceptance facts.

I read **all five paths' publication delta** and **all eight paths' focused fix delta**. The 372-line shared background-wakes patch section was read in the publication delta and compared byte-for-byte against its focused-delta copy with `diff`; it is identical. The full original-base 22-path patch was available, hashed and used for context. I do not claim a full-file deep read of the approximately 1 MB App entrypoint, the large DB/Session owners, the approximately 456 KB workflow, or the approximately 1.46 MB ledger.

Every scoped source identity was dispositioned as follows:

| Scoped path | Actual depth and disposition |
| --- | --- |
| `.github/workflows/rust-desktop-runtime.yml` | Complete focused integration hunks; original MSU App/build/input additions and adjacent feature-artifact, exact-case and result gates. Preserved parent workflow, including incoming fresh-authorization and poll-custody additions; not a whole-workflow review. |
| `projects/openclaw-rust/docs/specs/final-assistant-media-custody.md` | Complete document and publication qualification. |
| `projects/openclaw-rust/docs/specs/media-settlement-unknown.md` | Complete document and both relevant deltas. |
| `projects/openclaw-rust/inventory/index.json` | Complete current file, prior/parent comparison and original-base hunk. No R6 inventory regeneration or completion promotion. |
| `projects/openclaw-rust/parity-ledger.json` | Complete focused hunks; global flags/coverage header, original MSU registration, read-scope risk qualification and adjacent changed references. No whole-ledger reading claim. |
| `projects/openclaw-rust/requirements-traceability-matrix.json` | Complete original/focused changed hunks, including MSU requirement/oracle/blocker references and the incoming component references. No whole-file reading claim. |
| `projects/openclaw-rust/scripts/capture_media_cleanup.py` | MSU name selections, input identities, exact-result assessment and relevant command assembly. Existing script source only; not executed. |
| `projects/openclaw-rust/scripts/capture_portable_contracts.py` | Complete focused hunks plus affected required-name/input/result consumers. Existing script source only; not executed. |
| `source/host/Cargo.toml` | Complete file; default-off feature and existing `syn` contract reviewed. |
| `source/host/app/Cargo.toml` | Complete file; ordinary dependency versus dev-dependency feature activation reviewed. |
| `source/host/app/src/main.rs` | Deep read of the complete pending-media state, index/entry locking, guarded cleanup, resolution, abort and send methods at approximately lines 708–1275. Original-base adjacent cron changes read as inherited context, not an R6 production change. |
| `source/host/app/src/tests/channel_host_lane_tests.rs` | Complete original-base changed hunks for the nested MSU module and reused Responses/child/read helpers. Relevant actual fixture consumers read through the complete MSU fixture. Not a whole SGF fixture reread. |
| `source/host/app/src/tests/local_media_file_url_tests.rs` | Complete changed hunks and idle/custody assertions; durable empty work/path requirements remain. Not a whole-file reread. |
| `source/host/app/src/tests/media_settlement_unknown_tests.rs` | Complete current 1,035-line file, including the changed acceptance field and actual generated Runner path. |
| `source/host/src/attachment_paths.rs` | Complete file, including cleanup state transitions, Drop and custody tests. |
| `source/host/src/extensions/session/agent_db.rs` | Deep read of the exact fault lease/state, retirement, command adapter/authorizer, entire media transaction and relevant getter/receipt boundaries; fixture helpers plus lost-ACK and other-owner reopen assertions. Not a whole 426 KB owner read. |
| `source/host/src/extensions/session/production.rs` | Typed media append, cached versus first-open owner/recovery, path verification, fence clear and presentation filtering. Not a whole 290 KB owner read. |
| `source/host/src/extensions/transcript/production_runtime.rs` | Typed media append and its adjacent compatibility methods; actual `RoutedSendAcceptance` declaration and returning branches. Not a whole-file read. |
| `source/host/src/runner/tools/send_message_tool.rs` | Changed build-error classification and complete relevant send/result branch, with generated producer context. Not a whole-file read. |
| `source/host/tests/background_wakes_production_contract.rs` | Complete new helper/mutation block, its enclosing shipping test and adjacent unconditional/source-selection helpers; unchanged declaration names compared against prior snapshot. Not a whole 191 KB test-file read. |
| `source/host/tests/openclaw_sessions_and_config_contract.rs` | Complete focused/publication changes; deep read of media method/owner proofs, normalization, fixture/negative controls and the composed original channel parser/call-count/raw-COMMIT boundary. Unchanged declaration names compared against prior snapshot. Not a whole 171 KB test-file read. |
| `source/host/tests/send_message_tool_contract.rs` | Complete added custody-classification fixture/tests and affected producer/counter behavior. Not a whole-file read. |

Additional unchanged exact-parent source was read directly from `4e80e161c6cc539f4c120d64f91421619a57def1`: `source/host/src/runner/tools/mcp_meta_tools.rs`, blob `923288198b898083cbfe5da9a1f76e9f309ee516`, lines 1–250; and relevant custody/receipt/admission/checkpoint sections of `source/host/src/runner/stream_attempt.rs`, blob `6eb8a3eae5556eb6d6a6c03d00059d7c1180190f`, especially lines 17–130, 155–201 and 396–485. These reads introduced no source writes. Outside-scope incoming component implementation was not independently re-reviewed; its preservation follows the supplied exact-parent tree proof, and this verdict does not claim new acceptance of that component.

## Review conclusions with source evidence

### 1. The COMMIT oracle union retains the original prohibition

The media exception remains bounded to the actual inherent command method and its actual media-owner consumer. `media_commit_inherent_impl` distinguishes inherent implementations from trait implementations without deleting trait bodies from the residual source (`openclaw_sessions_and_config_contract.rs:1693–1737`). `media_commit_owner_is_handled` checks the unique typed method, exact entry classification, synchronous immediately invoked write closure with its three admission checks, and exact returned settlement tail (`:1740–1862`). The rollback command is independently proved but left in the residual source (`:1871–1886`).

The media proof substitutes only its already-proved call and removes only its already-proved command method before composing the original channel proof. The original argument-taking caller count remains `Some(1)` in the channel proof (`:2413–2486`); the final global raw-COMMIT prohibition remains at `:2489–2494`. This does not convert the new legitimate adapter into a larger global count allowance or a whole-implementation exclusion.

The R6 union preserves both sides of the required qualification: logical CRLF normalization is centralized for oracle images only (`:1865–1868`); each destructive media substitution requires a proven actual-source baseline, one intended site and changed input (`:1894–1921`); positives retain the actual `Drop` and complete harmless trait bodies; negatives cover helper/owner ambiguity and conditional exclusion, command/rollback/retirement changes, additional direct/aliased/macro callers, trait callers and hidden raw COMMITs (`:1924–2231`). The raw negative requires exactly one residual forbidden direct call, rather than merely checking for some residual text. The incoming channel no-op assertions remain in the focused delta.

**Conclusion:** no weakened exception or lost control identified. The LF/CRLF positive and mutation outcomes still require execution in Actions.

### 2. The background-wakes repair checks the actual guarded cleanup delegation

The raw Hfc log reports the old failure at `background_wakes_production_contract.rs:2930`, specifically `SHIPPING_HOST.contains("for cleanup_error in custody.rollback()")`. The current production abort delegates into `rollback_pending_media`; those obsolete literals did not describe the current owner.

The replacement helper binds the unconditional shipping `SendMessageSink` implementation and the unique inherent cleanup helper (`background_wakes_production_contract.rs:2853–3070`). Its abort pattern requires lookup without creation, the same entry's nonblocking lock and `confirmed_precommit_refusal = false`. Its cleanup pattern checks the Collecting branch, the guarded pending-refusal branch, the retain/return branch, terminal-state cleanup, real returned-error diagnostic, failure return before fence release and checked release diagnostics. These shapes correspond to the actual methods at `main.rs:856–895` and `:1087–1093`.

The same existing shipping test retains its unrelated delivery/failure/context checks. The new block first applies the actual-source positive for LF and CRLF, then contains 16 destructive controls with unique-site and changed-input assertions and both physical line endings (`background_wakes_production_contract.rs:3152–3279`). The controls include dead delegation, false refusal authority, entry creation, blocking lock substitution, removal of the pending guard, cleanup of retained phases, error loss, premature release, failed-cleanup fallthrough, entry replacement and conditional/ambiguous owners.

**Conclusion:** the repair preserves cleanup/error-reporting responsibility and strengthens its connection to the actual methods. Its comment correctly limits the helper to source wiring; it does not pretend the AST check proves real SQL, file deletion or concurrency behavior.

### 3. The App acceptance-field correction matches the real type and preserves the assertion

The changed assertion at `media_settlement_unknown_tests.rs:805` now reads `accepted.context.accepted_durably && !accepted.duplicate`. The actual `RoutedSendAcceptance` owns `duplicate` alongside `context: PersistedSendContext` (`production_runtime.rs:65–69`); the context owns `accepted_durably` (`send_pipeline.rs:49–64`). The actual duplicate and newly admitted returning branches set the former field on `RoutedSendAcceptance` (`production_runtime.rs:833–851`, `:1032–1035`).

Both intended predicates remain. No production field or type is added, and no assertion is deleted. The surrounding fixture still requires a real claimed/applied lost-ACK fault, retired owner, durable send plus hidden receipt, retained media/ACK/identity, zero delivery/success/retry, no fabricated canonical history and two fresh owners (`media_settlement_unknown_tests.rs:793–965`).

**Conclusion:** the source cause of the supplied E0609 failure is corrected at the actual owner field. This is not a claim that the App test build or its eight cases have now passed.

### 4. The publication qualification agrees with the unchanged ownership chain

MAC-v1 now precisely says local custody settles before **App** publication/ACK/success counting, while DB listeners run after COMMIT and before the App-local terminal transition (`final-assistant-media-custody.md:134–152`). The DB releases its connection guard before publishing listeners (`agent_db.rs:5082–5091`); the App commits its custody and releases the entry guard before delivery/event/ACK/activity consumers (`main.rs:1211–1273`). Reentrant same-ID resolution/send reaches `try_lock` before another capture or append and returns typed unknown while the existing entry is held (`main.rs:795–835`, `:1009–1013`, `:1115–1137`). The documentation no longer overstates the DB/App ordering.

Unknown result retention is consistent across the inspected chain. The exact DB connection is retired on attempted-COMMIT error or unconfirmed rollback (`agent_db.rs:4568–4584`, `:5064–5103`). Session preserves the inner typed result through its outer filesystem boundary (`production.rs:4063–4084`), and Runtime propagates it (`production_runtime.rs:1797–1836`). App keeps OwnerSettlementPending on typed unknown; ordinary abort and Drop do not restore deletion authority (`main.rs:856–895`, `:1165–1208`; `attachment_paths.rs`, custody state/Drop implementations). Terminal entries remain in the existing map, preventing a detached older Arc from coexisting with a replacement for the same ID.

The actual Tool bridge preserves an unknown build error without abort and returns a sink error without fabricating success (`send_message_tool.rs:405–413`, `:449–481`). The unchanged Runner guard marks `DownstreamOutcomeUnknown` before completing the worker receipt (`mcp_meta_tools.rs:185–213`); custody remains sticky, blocks subsequent admission/checkpoint qualification and exposes a retry-block reason (`stream_attempt.rs:35–50`, `:195–201`, `:407–485`). The fault authorizer retains its state Arc and borrowed connection until synchronous removal before retirement (`agent_db.rs:6280–6340`). No new production owner, retry loop or schema is introduced by R6.

The MSU owner-getter qualification also matches the implementation: the DB getter uses the owner parser and retains receipts (`agent_db.rs:5843–5856`, `:7777–7788`), while the Session projection filters them (`production.rs:41–47`). The independently specified ordered send-plus-receipt fixture remains used for raw storage, lost-ACK reopen and exact-other-owner checks (`agent_db.rs:9189–9207`, `:9364–9374`, `:9552–9589`).

**Conclusion:** the documentation changes describe the actual existing boundary and do not relax unknown retention or the receipt oracle.

### 5. Integration and source-count preservation remain explicit

Read-only textual comparison against the immutable prior snapshot found identical test declaration/name lists: **48** background-wakes `#[test]` declarations and **63** session-contract `#[test]` declarations. These are source declarations, not executed counts. No test harness, test listing command, parser or validator was run to obtain them.

The existing native capture retains the 27-name transaction-commit selection, including the same eight media-oracle names, and its exact named-result/no-extra/zero-failure/zero-ignore assessment (`capture_media_cleanup.py:228–256`, `:358–387`, `:519–521`). The App workflow still requires the eight existing MSU cases and an exact case selection followed by one-pass/zero-failure/zero-ignore completion (`rust-desktop-runtime.yml:4355–4377`). The ordinary shipping Host library must lack `test-support`; the App test dependency library must contain it, with exact manifest/source binding and `profile.test == false` for both dependency artifacts (`:4166–4199`). Source declarations and feature settings alone do not satisfy these compiled-artifact observations.

The adopted workflow/capture/ledger/RTM have the incoming identities listed above. They are not the old R5 workflow identity. Their complete focused integration delta adds the incoming fresh-authorization/poll-custody requirements and scope qualifications without removing the existing MSU gates. The R6 publication delta does not modify those parent leaves. The ledger's eight completion flags remain false; the MSU responsibility remains `read: partial`, `implementation: implementing`, `verification: not-run`, `acceptance: pending`. MSU's pending-mapping declaration rule and immutable inventory digest are preserved as the stated Actions authority; this review neither materialized mapping rows nor audited the complete compressed shard population.

## Findings

**None.** No concrete failure scenario in the reviewed R6 changes and adjacent MSU boundaries warrants a source-fix request from this lens. The existing no-fence durability limitation, broader read-scope/file-identity contract, arbitrary crash windows and real provider/device/package acceptance remain explicitly open; R6 does not purport to close them.

## Execution boundary and outstanding evidence

Only source reads, exact-ref GitHub reads, text searches, diffs and hashes were used. I did not run Cargo, rustc, tests or test listing, linters, generators, schema checks, parsers, validators, captures, local/htch verification, an Actions dispatch or a branch mutation. I did not read prior/current review reports, implementation reports, diagnosis reports, root adjudications or mutable working source. Only this report file was written.

The raw evidence consulted was:

- `/workspace/scratch/7d27ea239237/media-fc-ci-app/linux-native-terminal-13207615800/channel-pending-recovery/integration.log:766–832`: the historical target reports 47 passed and one failed, with the stale cleanup literal as the explicit failure.
- `/workspace/scratch/7d27ea239237/media-fc-ci-app/linux-native-terminal-13207615800/link-preview-native-gateway/app.log:1064–1070,1098–1100`: E0609 on `accepted.context.duplicate` prevented the historical App test build.

Neither result is relabeled as current-candidate success. The root-owned authorized exact-H Actions set still must establish the current AST positives and mutations, preserved transaction/channel controls, actual App compilation and eight cases, real DB fault application and recovery, and compiled feature separation with the existing named-result, exit/log/input identity receipts. The final published commit/tree must remain correctly bound to those receipts. Full G01–G12/AC01–AC24 and independent migration acceptance remain separate.

**Final source verdict: APPROVED. Current-candidate Actions/runtime acceptance: PENDING.**
