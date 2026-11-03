# MSU R5 independent directness and campaign-budget assessment

## Terminal recommendation

**The R5 oracle needs new exact-subject execution. A bounded new-input verification is technically justified, but a third campaign is not authorized automatically by the changed commit or by this assessment.** The normal H74 campaign and focused Hfc recovery already consume the default budget in `AGENTS.md`. Before a publication that starts another Actions campaign, the task owner must preserve an explicit exhaustion/budget decision adopting the narrowly bounded plan below under its existing execution authority. If that authority does not permit an exception, retain the R5 change as unverified and stop at that execution boundary. Do not report an environmental `BLOCKED/UNRECOVERABLE` outcome: the necessary instrument exists, and the outstanding App/macOS work was pending at the observation cutoff.

The recommended exception, if authorized, is **one new immutable R5 subject, one existing writer-push Actions execution set, no duplicate dispatch, and no automatic recovery or rerun allowance**. It addresses a changed oracle input; it does not repeat H74 or Hfc in hope of a different result. Record it as the next campaign after H74 and Hfc, not as a reset normal campaign. An exception decision must acknowledge that the unchanged workflow executes an inseparable superset of the newly unmet checks. This report supplies the independent directness/exhaustion assessment, not a waiver of the default cap, an execution result, a source-review verdict, or final acceptance.

## Scope, authority, and fixed observation boundary

Task: `647e0932-bea4-4113-9a61-65ae3db58955`, round 1. Repository: `bhrumom/fabushi-desktop`. Branch: `codex/openclaw-rust-fusion-chat-20261101-writer`.

The current repository-wide authority was read from `/workspace/scratch/7d27ea239237/media-settlement-authority/AGENTS.md`, including Adaptive verification and evidence recovery. Its SHA-256 is `3fb67d6b83f0f29436b9299d341b7b0171c712c2314fe995ec719221c2c19c87`. The governing rule preserves passing evidence unless an input change invalidates it, requires causal classification, and requires independent exhaustion assessment at one normal plus one recovery campaign. A cap or timeout alone is neither approval nor proof of an unreachable subject. The OCRM rule and the assignment prohibit executable validation outside GitHub Actions, including `htch-runtime`.

The fixed execution-status cutoff is **2026-11-03 05:23:43 UTC**. No new API observation, wait, or polling loop was made. The assessment reads preserved source, JSON, raw logs, diffs, and hashes. It does not read other R5 source-review reports, alter source/workflows/gates, or perform local tests, parsers, validators, generators, captures, builds, or enumeration. The sole write is this report. The configured parent model and inherited effort were retained; the assignment selected xhigh for the judgment required here, with no independent setting override claimed.

| Subject | Immutable identity | Evidence boundary |
| --- | --- | --- |
| H74 normal campaign | Commit `74f11d3d31c6245ac5886dc0181460b2d4349744`; tree `aca91e1cfbedeecb039d442fb3528caac89873a3` | Source run `41580539540` / attempt 1; runtime run `41580539491` / attempt 1 |
| Hfc recovery | Commit `fc187ecba404e2fa95d65871301aa8d83c7f0314`; tree `410414bcf3ad042cf49aeed86ce55836f4fbc1cd` | Source run `41582614099` / attempt 1; runtime run `41582614116` / attempt 1 |
| Materialized source carrier | Commit `14faff5d9605c99df1494303f1aa0ce31bcfd420`; tree `518e019acc61861af909fcfd48a5c103a7ac7fe3` | Preserves Hfc's Actions-generated source accounting; it is not the subject of the Hfc runtime receipts |
| Frozen R5 candidate | Tree `fc752f889ce0d402f6f5af9911d9777c7cab7999` | Review/source snapshot, not a tested commit or execution |
| Fixed upstream U | Commit `1440bb1771372334bea23d7642bb52deb7db1845`; tree `90c8aab39972da11cfad220361781d49e1bc6700` | Unchanged source-accounting authority |

R5's manifest and tree-readback proof are `/workspace/scratch/7d27ea239237/media-settlement-r5-manifest.json` and `/workspace/scratch/7d27ea239237/media-settlement-r5-tree-proof.json`. Their SHA-256 values are `1a33405b8920f79a0f2967e0caa9aa00cfd8dcb0c0f25cb52cdd31a4bbe272f6` and `2702341dd3f1225774b78a41ba6df61c643c2a0b7c5a6fd015c0faf5ad141ea8`. The publication diff is `/workspace/scratch/7d27ea239237/media-settlement-r5-publication.patch`, SHA-256 `34d5daa754d98cd20ad36b2dab321757d89c8b92980bfe5041ed443e946576a0`.

### Incoming carrier update reported during assessment

After the frozen-input assessment, root reported a fresh branch head `f41a6d3c00150ba59453d14887b5b4914801d4f4`, parent `14faff5d9605c99df1494303f1aa0ce31bcfd420`. It includes another task's Automation poll-custody component and a 23-line addition to an existing backend-contract receipt in the runtime workflow, whose new blob is `37f90cfcddef5fbc917d07b5fa5468707a4bc452`. Root reports that its complete diff inspection found the media/App selectors unchanged and that none of the three R5 publication paths changed. This is an attributed integration update, not an independent source review or execution claim by this assessor; that component's own reviews remain with root.

Preserve that incoming component. A final integrated commit necessarily needs a newly frozen commit/tree and its actual workflow blob recorded; it must not be called tree `fc752f889ce0d402f6f5af9911d9777c7cab7999`. The R5 oracle gap and cap decision remain unchanged. Statements below that the workflow is unchanged describe the original R5 snapshot versus Hfc; they do not erase the later incoming workflow identity. Before launching, bind the selected media/App steps to the preserved final workflow and retain the separately governed component's identity. A provenance difference in the whole workflow input map does not by itself refute unchanged media commands, but the old receipt cannot be relabeled as having executed the new workflow. No additional Automation source-review scope is assumed here.

## What failed, and why repeating it is not recovery

The historical failure evidence is retained under `/workspace/scratch/7d27ea239237/media-h74-ci/`, with its durable copy under `media-settlement-work/projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/execution-h74/`. The following judgments use raw failures and the relevant immutable source, not another reviewer's conclusions.

| Failed assumption | Direct evidence and causal classification | Forbidden repetition and appropriate correction |
| --- | --- | --- |
| Source expansion would repair undeclared missing reverse links after the pre-expansion gate. | Source job `125734077461` stopped at `verify_audit_integrity.py --before-expansion` with `Missing reverse source/responsibility link: OCRM-RESP-MEDIA-FINAL-CUSTODY-001`. The gate detected an actual declarative-input defect; it was not an unavailable runtime or a transient failure. | Do not rerun unchanged H74, hand-edit generated historical rows, or relax the gate. The existing pending-mapping declaration and Actions materialization path is the causal instrument. Hfc has since passed that path. |
| A canonical DB getter returns only the presentation send row. | On both Linux and Windows, DB13 produced 11 passes and two failures. Raw failures show the returned send **and hidden settlement receipt**, while the fixture expected only the send. The H74-to-Hfc DB diff changes only fixture expectations and reuses the exact send-plus-receipt payload/order already required by its independent SQL assertion. This is `TEST_FLAW`, not proof that production should hide its receipt. | Do not delete/filter the receipt or alter production behavior to satisfy the old fixture. Check exact canonical rows, durable fence/receipt state, and real owner reopen through the corrected fixture. Hfc DB13 now supplies that evidence. |
| Every implementation with a matching self type is an additional inherent owner, and literal source fixtures always have LF bytes. | H74 COMMIT27 reported Linux 21 passed/6 failed and Windows 20 passed/7 failed. The actual-source media baseline failed; the Windows owner-mutation case additionally failed at `actual media signature`. H74's source selector counted `impl Drop for SandAgentDb` before imposing the inherent-only condition, and its fixture read `include_str!` without newline normalization. The raw failure is a failing oracle baseline; the source explains the trait/line-ending assumptions. These are `TEST_FLAW` in the evidence mechanism. | Do not rerun the same faulty oracle, remove real trait source, widen the global raw-COMMIT allowance, or treat negative cases as proven merely because a broken baseline rejected everything. Keep trait bodies in the globally checked residual source and exercise both logical line endings with a proven baseline. Hfc establishes its repaired assertions; R5 adds further explicit qualification. |

The H74 oracle source used for this assessment is the matching R3 snapshot: its SHA-256 `e61629a709e4154dda1297d3f5fa9fd13efc38c00d67c08edbc93fb70a27d32e` equals the H74 receipt's oracle input. The DB snapshot similarly matches receipt SHA-256 `9a8ad4aa952f11a3d7595eebf636f7d20c450381120416c00667b93ef0149c7b`. This avoids diagnosing one revision with another revision's source.

Some H74 controls genuinely passed and remain historical positives. In contrast, a media mutation's `None` result while its actual-source baseline also failed does not establish mutation sensitivity. Preserve those logged test results without promoting their intended claim. Hfc's successful baseline and controls provide the stronger subsequent evidence.

## What remains valid at the cutoff

The completed Hfc evidence is documented in `/workspace/scratch/7d27ea239237/media-fc-ci-report.md` and `/workspace/scratch/7d27ea239237/media-fc-ci-windows-report.md`, and retained in `/workspace/scratch/7d27ea239237/media-fc-ci/`. The DB13 and COMMIT27 raw named-result sections, receipts, relevant input identities, source receipt, and fixed cutoff job snapshot were read directly for this assessment.

| Claim | Existing direct positive | Carry-forward limit |
| --- | --- | --- |
| Source structural/history gates | Hfc source run `41582614099` succeeded. Pre-expansion integrity passed with five declared forward links; post-expansion integrity passed with zero pending links. The canonical inventory digest is `d0049087bdbcc8eecdc8d0a69bd48bfc7d70688f8917122198a6e15c80863d8a`. | Generated index/manifest/ledger bytes must retain their Actions provenance; they are not retroactively called unchanged Hfc Git inputs. R5 preserves the carrier's materialized accounting. Full source closure, final G02, independent acceptance, and all eight completion flags remain open/false as recorded. |
| Native Linux | Artifact `13206394654`: all 19 selected groups, **210 passed, 0 failed, 0 ignored**, including DB13 and COMMIT27. | Evidence belongs to Hfc and its executed input map. It remains evidence for unchanged implementation/test claims, not execution of the new R5 assertions. |
| Native Windows | Artifact `13206860961`: all 19 selected groups, **205 passed, 0 failed, 0 ignored**, including DB13 and COMMIT27. | Windows-specific counts stay 205. Native logs contain LF; the receipt contains CRLF. Windows execution does not itself prove CRLF stdout or every new R5 source mutation. |
| DB settlement behavior | Hfc DB13 passes on both native platforms using DB blob `87186bc4b246d88a5ce51ebda0623b76f4425d3d`. | R5 preserves this exact DB blob. Its production behavior and fixture have not been invalidated by editing a separate source-contract oracle. |
| Existing COMMIT oracle assertions | Hfc COMMIT27 passes on both platforms with oracle blob `35719fc77edb1fea8f7acf0b22524c6eff7b29aa`. | Those executed assertions stay positive. The same 27 names in a future receipt are insufficient to identify R5: R5 changes assertions inside those names. |
| Capture qualification | Hfc accepts positive and CRLF-positive synthetic logs and rejects the eleven negative capture controls. | This qualifies log capture only. It is not another set of product behavior tests or proof of R5 mutation sensitivity. |

The Hfc native receipts bind before/after maps of 62 paths, with unchanged input maps and raw-log hashes. The reports document exact-tree binding. The R5 manifest preserves the Hfc DB, production App, capture scripts, and runtime workflow identities. Retaining those positives is appropriate; relabeling the complete Hfc receipt as a new-H execution is not. If publication changes any additional source, fixture, environment, or dependency, reconsider only the claims affected by that actual delta.

At the fixed cutoff, Linux `sqlite-native-path` job `125740667771` was still executing portable controls; the actual App step was pending. No completed actual App feature-mode proof or MSU8/LFA6/SGF3/cron5 receipt had been collected. macOS `rust-host` job `125740668001` was queued. These remain **unmet/pending**, with no inference of success, failure, inability to run, or acceptance. Native tests cannot substitute for the actual App's Session first-open recovery or generated Runner behavior.

## Why the changed R5 oracle requires a new check

The three-path publication contains two specification edits and one oracle edit. Its new executable test input is:

| Oracle revision | Git blob | SHA-256 of LF source |
| --- | --- | --- |
| Executed Hfc | `35719fc77edb1fea8f7acf0b22524c6eff7b29aa` | `a12ea0167dd7e82d8c4982d9e9ac9741466b23e6c90236a4650056fb90cfa8fc` |
| Frozen R5 | `c0162f99b896c00d0cd100ba59ad700eb01ad802` | `30a8daf3750f6997221b1efb0e0d089b92a174e16ac37b05e0dcdffaf499b172` |

R5 adds an explicit accepted actual-source baseline before counterfactuals, exactly-one-site/no-op checks for textual replacements, full LF/CRLF mutation checks, and additional residual/trait preservation checks. These alter the experiment's oracle and controls. A source review can establish what the assertions request, but cannot establish that the compiled tests exercise and pass all those branches. No runtime artifact in the cutoff evidence names R5's oracle blob.

The same-named COMMIT27 suite must therefore be executed against the new frozen subject. This is a concrete input invalidation, not unexplained dissatisfaction with Hfc's green result. The new test obligations do not erase Hfc's DB, cleanup, admission, and other still-valid positives. The two spec edits clarify canonical getter rows, source-accounting declaration, inherent/trait ownership, line endings, and DB-listener versus App-local publication order; they do not create a new runtime owner or justify broader production reimplementation.

## Most direct existing instrument and its actual minimum scope

The direct executable claim is the existing `transaction-commit-owners` command in `capture_media_cleanup.py`:

```text
cargo test --locked --manifest-path source/host/Cargo.toml --test openclaw_sessions_and_config_contract commit -- --test-threads=1
```

This command is quoted from the workflow's capture source; it was **not executed here**. It selects the complete fixed set of 27 required names, including the changed media cases plus unchanged channel and global raw-COMMIT controls. A single changed-test positive without the coupled negative/residual controls would be less causal. No mocked App or source-only assertion can replace this execution.

The exposed workflow does not offer a COMMIT-only dispatch or capture group selector. `capture_media_cleanup.py` constructs all 19 cases unconditionally. The only runtime dispatch scopes are `full` and `ocrm-config`. The actual minimum existing carrier is consequently:

1. **Prefer the normal push-triggered runtime run for the final R5 commit on the existing writer branch.** Its conditions already select scoped OCRM behavior and skip broad unrelated work. Do not add a manual dispatch when the push has supplied an exact-subject run.
2. If no runtime execution exists for that exact frozen commit and a launch is explicitly authorized, the existing fallback is `validation_scope=ocrm-config` with `expected_subject` set to that full **commit** SHA. R5's tree SHA is not a valid substitute for a commit SHA. Do not dispatch `full`: it expands unrelated work and does not enable the Linux App step guarded by the scoped condition.
3. The unchanged native capture reaches COMMIT27 early on Linux and in the dedicated Windows job. These supply the new oracle proof, including synthesized LF/CRLF inputs and real Windows checkout/compilation. All 19 groups will run as an inseparable carrier; the extra groups are not evidence that their prior approvals were lost.
4. The source-authority workflow also triggers from this writer publication. Preserve its exact-subject results as the related execution set; do not launch a second source campaign solely to repeat the already recovered Hfc mapping. Its generated publication is a subsequent carrier, not a replacement tested subject.

This selection preserves the existing instrument's exact-HEAD binding, locked Cargo inputs, raw command and capture exits, required-name checks, input hashes, and artifact capture. In the assessed R5 snapshot, the runtime workflow blob is `673d6270d3c2a7f4b9e9f640acf88927f2f7b154` and the native capture blob is `d5376d94f569b112c5778efb529eb64d2b863766`. The final integration must instead retain and record the incoming workflow identified in the carrier update above. No workflow or assertion weakening is proposed.

For actual App requirements, the existing causal instrument remains the Linux `automation_app` step. It records the ordinary Host library from the shipping check with `test-support` absent, the ordinary Host dependency in the App test build with it present, the actual App executable identity, exact test selection, and one-pass/zero-failure/zero-ignored results for MSU8 and retained regressions. At this cutoff it has not run. Keep the already-owned Hfc run intact and collect a later terminal artifact when available; do not create a duplicate run to get around its pending portable predecessors. Any later Hfc App result must first be assessed at Hfc and then compared against the final candidate's relevant inputs before carry-forward is considered. This report grants no prospective pass.

## Required cap decision and stop rules

The default budget has already been consumed by H74 plus Hfc even though some Hfc work is still in progress. A commit change, oracle-only label, automatic push trigger, or separate workflow run ID does not erase that history. Publishing a triggering change is an execution decision for campaign accounting, even without a manual dispatch.

The recommended one-off decision must explicitly record:

- H74 and Hfc as the prior normal and recovery campaigns, with their preserved positives and failures.
- R5's new oracle blob and final commit/tree; any divergence from the frozen R5 source must be reconciled before execution.
- The new obligation: prove the stricter actual-source baseline, site-bound mutations, both line endings, and retained global/trait/channel controls through COMMIT27.
- The chosen existing carrier and its unavoidable 19-group/native and scoped-workflow superset. If the governing authority insists on executing **only** unmet checks with no inseparable superset, no such launch scope exists in the supplied workflows; hold at that authorization boundary instead of inventing an input or changing the workflow.
- Exactly one new-subject execution set, no duplicate push-plus-dispatch, no unrelated `full` run, no unchanged-H retry, and no automatic fourth-campaign entitlement.

This is a bounded recommendation to the existing task owner. Whether prior user/task authority already permits adopting it is outside this assessment's supplied record; do not invent permission or demand redundant permission when it already exists. Without an applicable recorded exception, **do not start the third campaign under the default budget**. The honest state is “required new-oracle evidence outstanding; campaign authorization decision outstanding,” not “approved,” “unrecoverable environment,” or “coverage waived.”

If the bounded run is authorized, acceptance of the new oracle requires the exact final commit/tree, R5 oracle identity, actual 27 required positive names, zero failed/ignored cases, both command and capture success, complete raw logs, and before/after input bindings. Counts alone cannot distinguish Hfc from R5. Preserve all original artifacts without normalization or replacement.

A failure then receives causal adjudication from its retained evidence: bad baseline/fixture/capture is `TEST_FLAW`; a sound oracle proving an implementation violation is `IMPL_BUG`; unfinished or ambiguous execution stays non-passing. No new run is automatic. The next decision must identify the failed assumption and a materially more direct or corrected input, or state the concrete required capability/authority that is missing. Queue delay, timeout, repeated signatures, and the budget itself do not establish an unreachable subject. Do not cancel unrelated or already-owned runs, perform broad cleanup, weaken COMMIT assertions, manufacture source closure, or promote missing actual App/macOS evidence.

## Evidence references

- Authority: `/workspace/scratch/7d27ea239237/media-settlement-authority/AGENTS.md`, Adaptive verification and evidence recovery; OCRM execution rule.
- Applicable requirements: frozen R5 `projects/openclaw-rust/docs/specs/media-settlement-unknown.md`, especially MSU-06, MSU-09, MSU-10 and Implementation/verification/exit; `final-assistant-media-custody.md` MSU correction.
- Immutable changed-input scope: `media-settlement-r5-manifest.json`, `media-settlement-r5-tree-proof.json`, and the three-path publication patch identified above.
- H74 failures: `media-h74-ci/logs/source-job-log-125734077461.log`; `media-h74-ci/artifacts/linux-media-13205686727/{receipt.json,media-settlement-owner.log,transaction-commit-owners.log}`; corresponding Windows artifact `windows-media-13206046601`.
- Hfc positives: `media-fc-ci/artifacts/source-authority-13206770678/p0-structural-receipt.json`; Linux artifact `linux-media-13206394654`; Windows artifact `windows-media-13206860961`; their raw DB/COMMIT logs and receipts.
- Fixed pending-state observation: `/workspace/scratch/7d27ea239237/media-fc-ci/windows-completion-jobs-api.json`; Windows addendum cutoff 05:23:43 UTC.
- Runtime instrument: `/workspace/scratch/7d27ea239237/media-settlement-r5/.github/workflows/rust-desktop-runtime.yml`, dispatch/push/concurrency, native media, Linux portable/App, Windows media, and scoped job conditions; frozen `capture_media_cleanup.py` command/name tables and unconditional cases.
- Source instrument: `/workspace/scratch/7d27ea239237/media-fc-ci/inputs/exact-h/.github/workflows/ocrm-p0-authority.yml`, SHA-256 `15e7cd4e6402dd515bbd023c2b421162fb854e13c8744ba6abec8f7fc33fc86e`, retained as Hfc workflow source. R5's declared publication does not change it.

No R5 execution or final acceptance is asserted.
