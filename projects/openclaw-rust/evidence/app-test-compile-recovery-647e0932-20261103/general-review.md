# LFA App compile recovery — independent general review

## Verdict

**SOURCE-ONLY APPROVE.** No actionable correctness, oracle, security, or lifetime finding in the frozen two-hunk repair. The edits directly address all 19 compiler diagnostics preserved from H23 without changing the tests' requirements, assertions, production implementation, or workflow. This is a source review of the repair, not evidence that the candidate compiles or that any App case passes.

Reviewer model: configured parent model unchanged, GPT-6 Astra Pro. Effort: **xhigh**, supplied as the actual worker launch setting; the assignment includes failure diagnosis and oracle/security/lifetime judgment. No model fallback or secondary worker was used.

## Frozen authority and inspection

- Repository: `/workspace/scratch/425c55809bda/fabushi-desktop` (`bhrumom/fabushi-desktop`).
- Base commit: `871b033c5b9aabbe322cc7322af92648939db9b5`.
- Base tree: `f641366eea64364f4d029487d04276c4c567790a`.
- Candidate tree: `7dec21739a84f7a238a60ac420a72a3d8e859928`.
- Sole changed path: `source/host/app/src/tests/local_media_file_url_tests.rs`.
- Base blob: `60e6be8a61d446706ac2188366be9bff1f08cb71`.
- Candidate blob: `57a0a3393f1bb9560eb036687688656b1f3fe6c8`.
- Fixed upstream U: `1440bb1771372334bea23d7642bb52deb7db1845`.

Read the root `AGENTS.md`, the complete candidate LFA-v1 spec (`projects/openclaw-rust/docs/specs/local-media-file-url-admission.md`, blob `9249e4a93f353247e97c7ef47339382932451d72`), the provided manifest and complete-scope patch, and all 783 lines of the exact candidate test file. An independent `GIT_NO_LAZY_FETCH=1 git diff --no-ext-diff --binary --full-index --find-renames` between the frozen trees confirmed that the complete scope consists of the explicit serde_json import and the one literal correction. The candidate tree enumeration found no nested `AGENTS.md` under the inspected Host/project subtrees.

Inspected adjacent exact-candidate contracts:

| Source | Inspected contract |
| --- | --- |
| `source/host/src/extensions/transcript/session_runtime.rs`, blob `bf7e3533d2691aaf204888fcdb64e1f548716c10` | `set_window_focused`, its `f64` timestamp, focus update and Session viewed-state call; candidate lines 286–344 |
| `source/host/app/src/main.rs`, blob `7ec9bd8a22fd595f63cb3f4b632673e298294dfa` | Existing imports, the `#[cfg(test)]` module declaration and leaf-module registration, `ProductionSendMessageSink`, `PendingOutgoingMediaPhase`, and the actual `Mutex<HashMap<String, Arc<Mutex<PendingOutgoingMedia>>>>` representation |
| `source/host/app/Cargo.toml`, blob `a2c58167a82657194b79aaf3e1e0d55577b2221c` | App binary entry point, edition 2024, existing normal dependency `serde_json = "1"` and existing test-support dependency configuration |
| `source/host/app/src/tests/channel_host_lane_tests.rs`, blob `f6139af6bde9829f7e9147e6a2028ec77680bf42` | Existing serde_json import precedent, `IdleRunnerOwners`, its disposal path, and `idle_runner_owners` construction of actual Session/Transcript/ForeverBox dependencies |
| `.github/workflows/rust-desktop-runtime.yml`, blob `239c054aaccd42c0e5bc5e7ca0607431bcf400b7` | Existing exact-subject App build capture, all six selected LFA test names, exact test listing, one-pass/zero-failed/zero-ignored case gate, and complete-case capture |

All candidate source reads used the frozen Git tree. The base commit and two dependency/helper blobs were unavailable locally, so their exact identities were read through the GitHub API. The exact commit API confirmed that the base commit points to the stated base tree. No lazy fetch, mutable-worktree source substitution, build, test, lint, formatter, parser, generator, validator, or harness execution was performed. No prior reviewer report or runner-decision artifact was read. The only written file is this report.

## Causal compiler recovery

The original receipt is at `/workspace/scratch/425c55809bda/reviews/plugin-h23-native-evidence/sqlite-final-artifact/automation-app/receipt.json`. It identifies subject `23c2a567d874509f56353b8cae737cdf1437309d`, tree `b10485b13c15cd1cf012a5f51acaf6927aeb322a`, run `41576693105`, attempt 1, Linux job `sqlite-native-path` (assigned job ID `125721731498`). It preserves shipping-check exit **0**, App test-build exit **101**, `cases: []`, and result `failed`.

Read the original `test-build.stdout.log` and `test-build.stderr.log`, plus the rendered raw diagnostic file `/workspace/scratch/425c55809bda/reviews/plugin-h23-native-evidence/APP-H23-19-COMPILER-DIAGNOSTICS.txt`. The original stdout has the 19 error records at lines 832–850 and `build-finished` with `success:false` at line 856; stderr reports failure due to 19 previous errors.

| Original failure | Candidate repair and causal assessment |
| --- | --- |
| Twelve `cannot find macro json in this scope` diagnostics | Candidate line 3 imports `serde_json::json` into the actual leaf test module containing every affected invocation. The existing sibling helper has the same direct import pattern. No invocation or JSON oracle changed. |
| Six E0425 `cannot find type Value in this scope` diagnostics | The same import supplies `serde_json::Value` for the existing delegate/sink signatures, transcript-entry vector and `call_send` signature. This uses the App's existing dependency and the types already used by its production APIs, without introducing a generic parameter or replacement type. |
| E0308 at `LocalMediaAppFixture::open`: expected `f64`, found integer | Candidate lines 200–202 still call the real `set_window_focused(&deps.session_workers, true, ...)` and retain the failing `expect`. `1_000.0` matches the exact API's `now_ms: f64` parameter and preserves the intended numeric timestamp of 1000 milliseconds. The API continues to store/use that timestamp through the same focus and Session paths. |

The historical diagnostic line numbers differ from the candidate because intervening media-custody compatibility changes are present in the base. The diagnosis is matched by the actual symbols and expressions, not an assumption of identical line numbering. Source reading supports removal of these diagnosed causes; it cannot rule out further diagnostics exposed by a subsequent compiler run.

## Oracle, security and lifetime assessment

The complete tree diff changes no assertion, expected value, fixture byte, negative URL, selected test name, ignored status, command, capture gate, production reader, or error path. All six App cases remain registered and selected. Their original refusal, earlier-image rollback, materialization/byte equality, native/Unicode, reopen/repair, AgentManagement batch-refusal and HTTPS controls remain intact.

The incoming Arc/Mutex media compatibility is preserved. The observer still clones the pending entry handle and reads its `media_work_id` through the entry mutex (candidate lines 124–127). `assert_custody_idle` still preserves the durable work-id/path assertions and examines retained terminal entries with `try_lock`, requiring Committed/Aborted phase, no pending owner settlement, no owned media, no media work id and no resolved sources (lines 280–297). It does not require the map to be empty or erase the retained identity fence.

No runtime security or ownership mechanism changes. The new import is inside the existing test module, and the numeric edit preserves the existing actual-owner fixture setup. The fixture's bridge delegation, scope-limited cleanup, owner disposal/reopen, invalid-source refusal and zero outward-effect assertions are unchanged. Neither hunk introduces a network destination, permission/root expansion, resource lifetime change, lock-order change, unsafe operation or new dependency.

## Remaining exact-H requirements

1. Publish the reviewed repair on the reconciled final candidate and bind the existing Actions run/receipt to its exact commit and tree. Any source change during reconciliation requires appropriate focused review; this verdict is bound to the frozen tree above.
2. In GitHub Actions, obtain a successful App test build using the existing locked App binary harness command, and actual execution evidence for all six existing LFA cases. Preserve the workflow's exact-name listing, one-passed/zero-failed/zero-ignored summaries, completeness checks, command exits and raw logs. A successful `--no-run` build alone does not satisfy the runtime requirement.
3. Preserve the H23 shipping-check result as historical positive evidence for H23, together with its test-build failure, empty case list and original logs. It does not establish compilation or execution of this candidate. The existing final-candidate shipping check and incoming custody cases remain governed by their current capture.
4. This focused test-compilation approval does not replace LFA's required native platform evidence, including actual Windows execution of Windows path cases, or the broader product/provider/package/independent acceptance gates. The separate cron candidate is outside this review.

No source revision, unit-verification status, runtime result, or full-media acceptance should be promoted from this source-only verdict.
