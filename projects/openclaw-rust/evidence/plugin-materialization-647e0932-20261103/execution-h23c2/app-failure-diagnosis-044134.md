# H23 App failure: exact diagnosis and minimal fix basis

## Verdict

TEST_FLAW — the local-media App test fixture does not compile. This is a concrete Rust test-target compilation failure, not an observed App business assertion failure, log-result oracle mismatch, timeout, runner resource failure or missing dependency.

The shipping cfg(not(test)) cargo check succeeded. The test build failed before any App case was run; the original receipt has cases: []. This does not establish App behavior acceptance or a linked shipping package.

## Exact execution and artifact

- H23: 23c2a567d874509f56353b8cae737cdf1437309d.
- Tree: b10485b13c15cd1cf012a5f51acaf6927aeb322a.
- Fixed U: 1440bb1771372334bea23d7642bb52deb7db1845.
- Run/attempt: 41576693105 / 1.
- Job 125721731498: https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/job/125721731498
- Final job status: completed / failure at 2026-11-03T04:40:00Z.
- Step 19 failed at 04:36:44Z; step 20 was skipped by the App-success gate; step 21 also failed at 04:39:57Z because its App target encountered the same test compilation defect.
- Step 22 fixture preservation and step 23 evidence upload both succeeded.
- macOS job 125721731668 remained queued in the 04:41:34 UTC snapshot.

Final artifact:
- ID 13204158834.
- Name ocrm-sqlite-native-path-23c2a567d874509f56353b8cae737cdf1437309d-41576693105-1.
- URL https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/artifacts/13204158834
- 601317 bytes; created 2026-11-03T04:39:58Z; 79 archive entries.
- API and downloaded ZIP SHA256 both equal 7d703426b75a1fe53a6f969f8e1fd2ad35c804a1889140c72d7d7f133859122a.

## First failed command

The original automation-app/receipt.json records exactly two commands:

1. cargo check --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host
   - exit 0.
2. cargo test --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host --no-run --message-format=json
   - exit 101.
   - receipt error: RuntimeError: command failed: test-build.
   - cases: [].

The test build's original stderr ends with:
error: could not compile `mahayana-app-host-runtime` (bin "mahayana-app-host" test) due to 19 previous errors; 3 warnings emitted

Its full compiler JSON stdout preserves the concrete diagnostics.

## All 19 diagnostics and H23 locations

All primary error locations are in:
source/host/app/src/tests/local_media_file_url_tests.rs

| Error | Count | H23 lines |
| --- | ---: | --- |
| cannot find macro `json` in this scope | 12 | 398, 431, 475, 536, 545, 565, 576, 643, 650, 690, 714, 743 |
| E0425: cannot find type `Value` in this scope | 6 | 35, 37, 152, 244, 342, 344 |
| E0308: mismatched types — expected `f64`, found integer | 1 | 199:42 |

At H23 line 199 the fixture passes:
&deps.session_workers, true, 1_000,

The compiler points to source/host/src/extensions/transcript/session_runtime.rs:315. Read-only source confirms:
pub fn set_window_focused(..., now_ms: f64) -> Result<bool, String>

The fixture imports super::super::* and other owners but never imports serde_json::json or serde_json::Value. The exact compiler errors confirm these identifiers are not available through that scope.

APP-H23-19-COMPILER-DIAGNOSTICS.json contains all primary locations and the original rendered messages. The corresponding .txt preserves their original rendered text without the unrelated compiler explanation payload.

## Why step 21 also failed

The native-dispatch receipt records:
- Host command: cargo_exit 0 / capture_exit 0.
- Coordinator command: cargo_exit 0 / capture_exit 0.
- App command:
  cargo test --locked --target-dir source/host/target --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host tests::link_preview_gateway_tests::
  cargo_exit 101 / capture_exit 0; no observed App positive names.
- Reused MCP composition command: recorded cargo_exit 0 / capture_exit 0.

The App log repeats the same twelve json macro errors, six Value errors and the same E0308 in the local-media test module. Selecting a link-preview test-name filter does not bypass compilation of the App test target. Thus this second failure is the same fixture compilation blocker; it is not evidence that a link-preview runtime assertion failed.

## Exact source binding and current incoming delta

The App receipt contains 255 input fingerprints. The failing file's recorded SHA256 matches the exact H23 source bytes:

- H23 Git blob: 5682e951f1eac8f107efd3a3948b0748870912bd.
- H23 file SHA256: f2097d754193bd21cc1b7911829f37ba395f59750680e1e560d682e6a8f49d80.

The exact 71b039ae860c45e982dfbc8b1f401143d49d2454 and 871b033c5b9aabbe322cc7322af92648939db9b5 versions of this file are byte-identical:
- Git blob: 60e6be8a61d446706ac2188366be9bff1f08cb71.
- SHA256: 9b279aba96ba879332b45aeddcc9c8d156526917725a2c40b6e684b123915a76.

Their diff from H23 only adapts pending-media reads to Arc/Mutex and strengthens the idle settlement assertions for retained terminal identities. It does not import json/Value or change the integer 1_000; the corresponding current call is at line 200 because the earlier media change inserts a line.

bb5a7f6098f952df6d388a672c4fb01045bc645c and d70f86f33b5e3c67545954b270373e1548809f4b are descendants of H23 and do not change this fixture. The source file at 71/871 still contains the reported issues. No source-comparison claim is promoted to executed acceptance on those later commits.

The unpublished cron R2 candidate cannot have introduced an error in the frozen H23 job. The concrete failure is in the local-media test fixture, not a cron assertion. This investigation did not edit or inspect mutable current cron main/workflow contents as if they were a frozen acceptance subject.

## Minimal repair basis for the integrator

Only two source corrections are indicated by these nineteen diagnostics, to be applied to an isolated copy of the current incoming file:

1. Add an explicit local import:
   use serde_json::{json, Value};
2. In the existing set_window_focused fixture setup, use the f64 literal:
   &deps.session_workers, true, 1_000.0,

Preserve the incoming 71 media changes, the whole existing test body and every assertion. No current main.rs, shared workflow, production owner or cron R2 file needs to be changed to address these particular compiler errors.

This recommendation addresses the observed errors only. No compilation or test has been run on the proposed fix here; subsequent exact-head Actions must establish that the test target builds and that the previously unexecuted App/startup/prior-root/dispatch cases pass. Passing portable and plugin execution remains scoped positive evidence and is not rewritten or rerun by this diagnosis.

## Original logs and comparison records

Root directory:
/workspace/scratch/425c55809bda/reviews/plugin-h23-native-evidence/

- h23-sqlite-native-path-13204158834.zip — original final ZIP.
- sqlite-final-artifact/automation-app/receipt.json.
- sqlite-final-artifact/automation-app/shipping-check.stdout.log.
- sqlite-final-artifact/automation-app/shipping-check.stderr.log.
- sqlite-final-artifact/automation-app/test-build.stdout.log — all original compiler JSON.
- sqlite-final-artifact/automation-app/test-build.stderr.log.
- sqlite-final-artifact/link-preview-native-gateway/receipt.json.
- sqlite-final-artifact/link-preview-native-gateway/app.log — same nineteen diagnostics in plain text.
- sqlite-final-artifact/link-preview-native-gateway/host.log.
- sqlite-final-artifact/link-preview-native-gateway/coordinator.log.
- sqlite-final-artifact/owned-fixtures.tar — retained, not executed or needed to infer this compile failure.
- APP-H23-19-COMPILER-DIAGNOSTICS.json / .txt.
- app-final-original-log-sha256-comparison.txt — all seven original stdout/stderr/dispatch logs match their receipts.
- app-failure-source-identities.txt — source blob and SHA256 comparisons.
- local-media-H23-to-871.diff — source-only incoming comparison.
- run-41576693105-attempt1-jobs-044134-api.json.
- run-41576693105-artifacts-044134-api.json.

No local or htch executable validation, production/workflow/ledger modification or push occurred. All local operations were original evidence acquisition, exact-source reading, byte/identity comparison or writes inside the authorized evidence directory. Actual host model and effort stayed inherited.
