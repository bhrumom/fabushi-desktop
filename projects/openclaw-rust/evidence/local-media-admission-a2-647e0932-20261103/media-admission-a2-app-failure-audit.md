# A2 Linux App failure evidence audit

## Decision and scope

The two audited App paths fail **while compiling the LFA test module**, before an App test harness runs. Their common cause is 19 compiler diagnostics in `source/host/app/src/tests/local_media_file_url_tests.rs`: twelve unresolved `json!` macro uses, six unresolved `Value` type uses, and one integer argument where `set_window_focused` requires `f64`. The ordinary shipping App `cargo check` succeeds. The raw records do not identify an environment failure as the cause of these two App failures.

The permitted editable fixture is exactly immutable R1 plus the two specified edits: `use serde_json::{Value, json};` and `1_000` → `1_000.0`. All other source bytes, including all assertion bodies and R1's custody observer changes, are retained. This is a source-byte comparison, **not evidence that the repaired fixture compiles or that its cases pass**.

This is a supplemental evidence audit, not terminal independent approval of A2, R1, MSU, the Linux job, or the whole product. No project parser, validator, test, test listing, build, formatter, capture script, local/htch validation, workflow dispatch/rerun, commit, or push was executed. No production, test, workflow, ledger, capture, or existing evidence file was modified. The only outputs are this report and its prefixed data companion.

## Exact subject and custody of the evidence

| Identity | Value and source |
|---|---|
| Repository | `bhrumom/fabushi-desktop` |
| Tested H (A2) | `a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74` |
| Tested tree | `a0dceefeae75fccd7ab5aec143f6d651f7cf2fd3` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Upstream tree recorded in both receipts | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Runtime run / attempt | `41576543683` / `1` |
| Linux job | `sqlite-native-path`, numeric ID `125721250552` |
| Artifact | `13205207341` |
| Workflow path recorded as an input | `.github/workflows/rust-desktop-runtime.yml` |
| Recorded workflow SHA-256 | `6f10da0e15d1463e144e58c3e8a63ca4c028623ba77fce7e2cc7f036a616b52e` |

The parent supplied the completed/failure job status, numeric job ID, artifact download, and immutable R1 designation. This audit did not fetch new remote job metadata. Both audited raw receipts independently contain the exact H/tree/U/run/attempt, `job: "sqlite-native-path"`, and `runner_os: "Linux"` shown above. The local inventory identifies artifact `13205207341`; it is not a substitute for new GitHub terminal job metadata.

Evidence root:

`/workspace/scratch/7d27ea239237/media-admission-a2-actions/final-linux-20261103T0439Z/`

The raw archive is `artifact-13205207341.zip`, **601,501 bytes**, SHA-256 **`3dcc2d70b22a7b236eb2b2159ac2f8cb74cf9ca95584c9495922948223380586`**. Its byte length and hash match `inventory.json`. It contains **79 file members with 79 unique names**. Their name set matches the inventory; every decompressed member equals its extracted file byte for byte and has the inventory's length and SHA-256. These are archive/data comparisons only; no archived executable or repository script was run.

Every output hash cited by the four automation command streams and the link-preview execution logs agrees with the corresponding raw file bytes, including the reused automation-admission log. The data companion records all 79 member hashes and these comparisons.

## Automation App: shipping check succeeds, test compilation fails

Primary receipt: `members/automation-app/receipt.json`.

| Recorded command | Exit | Raw outcome |
|---|---:|---|
| `cargo check --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host` | `0` | Empty stdout; stderr line 1108 reports `Finished dev profile [unoptimized] target(s) in 1m 38s`. |
| `cargo test --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host --no-run --message-format=json` | `101` | Compiler diagnostics in JSON stdout; stderr line 136 reports failure to compile the App test binary due to 19 previous errors. |

The first command is an ordinary App check. It is not a successful linked test harness, an App case execution, a packaged build, or live delivery evidence. Its five App warnings and the later future-incompatibility warning are not the recorded fatal failure.

The second command is explicitly `--no-run`; even a successful result would only build the harness. Here it fails before producing an App compiler artifact. Its complete stdout is **856 JSON lines**, all decodable as JSON data:

| Cargo JSON record type | Count |
|---|---:|
| `compiler-artifact` | 648 |
| `build-script-executed` | 70 |
| `compiler-message` | 137 |
| `build-finished` | 1 |

The 137 compiler messages comprise 116 warnings, 19 errors, and two failure notes. There is **no `compiler-artifact` record whose target name is `mahayana-app-host`**. JSON line 856 is `{"reason":"build-finished","success":false}`. Stderr lines 135–136 identify `mahayana-app-host-runtime` and terminate with failure to compile `(bin "mahayana-app-host" test)`.

The receipt records exactly these two commands, `cases: []`, `result: "failed"`, and a capture traceback ending with `RuntimeError: command failed: test-build`. It does not record a separate numeric capture-exit field. That traceback describes the failed build command; it does not establish an independent environment or App behavioral-test failure. No subsequent automation/LFA App case execution is recorded. The two compiler streams contain no test-harness start or test-result summary.

### Complete fatal diagnostic classification

All 19 primary diagnostics are attributed to `src/tests/local_media_file_url_tests.rs` in the App test binary target. Source coordinates below are the **tested A2 coordinates**, not the later R1/repaired coordinates.

| Diagnostic | Count | A2 source positions | Automation JSON lines | Link-preview `app.log` range |
|---|---:|---|---|---|
| `cannot find macro json in this scope` | 12 | `743:37`, `714:57`, `690:42`, `650:59`, `643:55`, `576:37`, `565:50`, `545:39`, `536:50`, `475:41`, `431:36`, `398:40` | 832–843 | 811–881 |
| `E0425: cannot find type Value in this scope` | 6 | `35:16`, `37:17`, `152:18`, `244:30`, `342:11`, `344:13` | 844–849 | 883–932 |
| `E0308: mismatched types` | 1 | `199:42`: `1_000`, expected `f64`, found integer | 850 | 934–950 |

The timestamp diagnostic identifies `source/host/src/extensions/transcript/session_runtime.rs:315:12` as the defining method and suggests the literal `1_000.0`. This API location is compiler evidence; the current task did not re-read the Session source or infer another API change.

JSON lines 854–855 identify the E0308/E0425 failure notes; line 856 reports failure. The complete nineteen rendered diagnostics and primary spans are preserved as data in the companion JSON.

### Environment classification

The automation receipt records Linux, `rustc 1.97.1 (8bab26f4f 2026-07-14)`, host `x86_64-unknown-linux-gnu`, LLVM 22.1.6, and `CARGO_BUILD_JOBS=1`. It records a runner-owned temporary directory and the shared Host target directory. The inspected App logs contain the source compiler failures above and no fatal disk-full, permission-denied, out-of-memory, or SIGKILL diagnostic. This supports classifying these App failures as test-source compilation defects. It does not certify every aspect of the runner environment or every other Linux command.

## Link preview: the App filter is never reached

Primary receipt: `members/link-preview-native-gateway/receipt.json`.

The App execution uses:

```text
cargo test --locked --target-dir source/host/target --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host tests::link_preview_gateway_tests::
```

The receipt records `cargo_exit: 101`, `capture_exit: 0`, `execution_error: null`, `oracle_accepted: false`, and an empty `observed_passed_test_names` list for this App execution. Its three required positive names are all listed as missing:

- `tests::link_preview_gateway_tests::link_preview_gateway_cached_native_call_bypasses_closed_host_lane`
- `tests::link_preview_gateway_tests::link_preview_gateway_native_errors_and_private_cache_preserve_contract`
- `tests::link_preview_gateway_tests::link_preview_gateway_other_methods_preserve_host_lane_request_and_reply`

The raw `app.log` is **981 lines**. Line 810 begins App compilation; lines 811–950 show the same nineteen LFA-module diagnostics as automation-app. Lines 978–981 identify E0308/E0425 and terminate with failure to compile the test binary. There is no `Running ...` harness invocation, `running N tests` line, positive test line, or `test result:` summary in this App log.

Thus the filter for link-preview tests did not isolate compilation from the faulty sibling LFA module. These are **unexecuted link-preview App cases because the binary did not compile**, not three failing link-preview assertions. A successful capture process (`capture_exit: 0`) means the failing Cargo command's output was captured; it does not turn Cargo exit 101 into a pass. The overall link-preview receipt is `failed-or-incomplete`.

The Host and Coordinator paths are separate and have positive runtime evidence:

| Link-preview execution | Cargo / capture exits | Raw evidence inspected | Scope of conclusion |
|---|---|---|---|
| Host | `0` / `0` | `host.log` lines 754–816: 9 protocol tests and 43 server tests, all passed; 52 positive lines; both summaries have zero failed and zero ignored. | The Host execution ran and passed; all ten required names are recorded without missing/nonunique results. |
| Coordinator | `0` / `0` | `coordinator.log` lines 74–192: 58 contract tests, 14 inference boundary tests, and one production protocol test, all passed; 73 positive lines; all three summaries have zero failed and zero ignored. | The Coordinator execution ran and passed; all four required names are recorded without missing/nonunique results. |
| App | `101` / `0` | `app.log` compiler failure, no harness starts/results. | No App case ran in this invocation. |
| Reused MCP composition | `0` / `0` in receipt | Receipt records reuse of automation-admission evidence, 94 observed names and two required names; its raw referenced log hash matches. | No additional semantic audit of those 94 tests was performed; they do not supply App execution evidence. |

The conclusion “zero App cases ran” here applies to the two audited App paths. This report does not infer a whole-job execution inventory from unrelated receipt scopes.

## Input hashes: one map is not before/after evidence

The two receipts have different input evidence shapes:

| Receipt | Actual fields | Meaning and limit |
|---|---|---|
| Automation App | `inputs_sha256`, 255 entries | One recorded input-hash map. `inputs_before_sha256`, `inputs_after_sha256`, `inputs_sha256_before`, and `inputs_sha256_after` are all absent. No before/after immutability claim is made. |
| Link preview | `inputs_before_sha256`, 31 entries; `inputs_after_sha256`, 31 entries | The maps are equal for their 31 listed paths. `inputs_sha256` is absent. The LFA test file and `session_runtime.rs` are absent from both maps; equality therefore supplies no before/after claim for those files. |

Automation's single map lists the tested LFA test file SHA-256 as **`f2097d754193bd21cc1b7911829f37ba395f59750680e1e560d682e6a8f49d80`**. Both receipts list the App main source SHA-256 as **`36605022e09e91269ee17770a48fa9897e5ba4adcf5bbc7b21b6f382e3cdd4fa`** in their respective map shapes. These recorded A2 identities must not be replaced with the later R1 or editable repair hashes below.

## Two-line repair compared with immutable R1

Only the two explicitly permitted source files were read for this comparison:

- `/workspace/scratch/7d27ea239237/media-settlement-r1/source/host/app/src/tests/local_media_file_url_tests.rs`
- `/workspace/scratch/7d27ea239237/media-settlement-work/source/host/app/src/tests/local_media_file_url_tests.rs`

| Snapshot | Bytes / lines | SHA-256 | Git blob SHA-1 computed from bytes |
|---|---|---|---|
| Immutable R1 | 36,297 / 782 | `9b279aba96ba879332b45aeddcc9c8d156526917725a2c40b6e684b123915a76` | `60e6be8a61d446706ac2188366be9bff1f08cb71` |
| Editable repair at audit read | 36,330 / 783 | `5bc24a39d3bbc77eab971ee0e58221f2a9c0ae6a226e1a35f311a69f80662106` | `280c68503daadecb5aa4a1c6eebec5744bc3b81d` |

The full text diff has only these two hunks:

```diff
@@ -1,5 +1,6 @@
 use super::super::*;
 use super::channel_host_lane_tests::{IdleRunnerOwners, idle_runner_owners};
+use serde_json::{Value, json};
 use std::collections::BTreeSet;
 use std::path::{Path, PathBuf};
 use std::sync::atomic::{AtomicUsize, Ordering};
@@ -197,7 +198,7 @@
         ));
         deps.transcript_manager.bind_roster_emit(roster).expect("actual roster owner");
         deps.transcript_runtime.session_runtime().set_window_focused(
-            &deps.session_workers, true, 1_000,
+            &deps.session_workers, true, 1_000.0,
         ).expect("actual focused desktop state");
         let sink = Arc::new(ProductionSendMessageSink {
             host_tx,
```

Applying precisely those two byte substitutions in memory to the immutable R1 bytes produces the complete editable file byte for byte. Each original anchor occurs once. This establishes retention of the rest of the file without relying on an AST parser or a selected subset of assertions. Both files contain the same 134 lines with an `assert!`, `assert_eq!`, or `assert_ne!` macro; that text count supplements the stronger whole-file equality proof and is not an independently executed oracle.

All six named test functions remain unchanged:

- `local_media_file_url_app_rejects_invalid_before_materialization`
- `local_media_file_url_app_rolls_back_earlier_image_on_later_refusal`
- `local_media_file_url_app_commits_native_and_unicode_media_across_reopen`
- `local_media_file_url_app_repeated_refusal_then_repair_survives_reopen`
- `local_media_file_url_app_agent_management_refuses_image_before_outward_send`
- `local_media_file_url_app_preserves_https_without_local_custody`

R1's custody-observer adaptations are retained exactly, including the repaired file's `assert_custody_idle` at lines 280–297. That helper still requires the durable work ID to be absent and durable paths empty; it clones indexed entries, uses `try_lock`, requires `Committed | Aborted`, and checks that owner settlement is not pending, media ownership is false, the media work ID is absent, and resolved-source state is empty. The two-line repair does not weaken or remove those R1 checks or revert the terminal-entry handling to an empty-map assumption.

The explicit serde_json import addresses the eighteen recorded unresolved macro/type diagnostics, and the float literal addresses the recorded E0308 argument mismatch. This is a direct source-to-diagnostic correspondence. A successful compile and actual execution of the repaired candidate remain unobserved by this audit; another compilation issue could become visible only after these errors are removed.

## Raw byte hash ledger

Paths below are relative to the evidence root. SHA-256 values are recomputed from the stored raw bytes, without text normalization.

| Path | Bytes | SHA-256 |
|---|---:|---|
| `inventory.json` | 14,643 | `4071afb29ed9aa3d52524de241e23c1aa2046657c3ce93f36d1b2323c833ea37` |
| `artifact-13205207341.zip` | 601,501 | `3dcc2d70b22a7b236eb2b2159ac2f8cb74cf9ca95584c9495922948223380586` |
| `members/automation-app/receipt.json` | 40,751 | `8dc4617e2eb5625bae2e6fc5b2f4f1ed03ffe883cda0bc15930421e115507f56` |
| `members/automation-app/shipping-check.stdout.log` | 0 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `members/automation-app/shipping-check.stderr.log` | 50,559 | `0f26e720e6859bbd2fe4dc8ee95e113094ebb488d70e3950f0c8b945ed0c7231` |
| `members/automation-app/test-build.stdout.log` | 862,225 | `6cf48c5357cc6251ed60792e70b9d5fa1ab7439e38331ac2f70fd7847d57657c` |
| `members/automation-app/test-build.stderr.log` | 8,890 | `f820b69a230316781cc6f05ca77bcea0263ff840fb35dbda10534aee25c61590` |
| `members/link-preview-native-gateway/receipt.json` | 34,005 | `640fe265fec818858aa605a7f7bd01d036be9d452f1c30b1a0503e9f70ee7331` |
| `members/link-preview-native-gateway/app.log` | 43,694 | `3f25586d29dea987e7c6c6f1b33e372d39f178a479aa2da8604b08db69ea7087` |
| `members/link-preview-native-gateway/host.log` | 31,892 | `93eee4c99f9f76a5b0ff011fec2ccee10c4b74f306dbd916ebad983fb69178f4` |
| `members/link-preview-native-gateway/coordinator.log` | 12,771 | `4b0bed5cf8b5c6a04fde44d2e5f4c29f3d410f9894c70503a1cc6e6765caa3fd` |

The data companion is `/workspace/scratch/7d27ea239237/media-admission-a2-app-failure-audit-data.json`, **56,157 bytes**, SHA-256 **`9c8db5f1fd5aa08029ae3799ca6c2df4f48101e0d4a151e79ec6942ee4741115`**. It contains the archive/member accounting, exact receipt identity fields, command results, nineteen diagnostic records, input-map presence/absence accounting, raw log summaries, and the complete two-hunk diff.

## Remaining concrete failure and acceptance limit

Frozen A2 still has two failed App compilation commands and supplies no positive App case evidence from either audited route. The later two-line source repair does not change A2's recorded result. Its exact source diff preserves the R1 assertions and addresses the reported compiler sites, but the repaired candidate's App harness compilation, the six LFA cases, the three link-preview App cases, and the other automation App cases still require evidence from the authorized Actions workflow on the actual candidate. No test/runtime defect in those unexecuted cases has been demonstrated by these logs.

No additional unresolved environment cause was found in the inspected App streams. This audit leaves all unrelated job failures, other platform outcomes, MSU implementation acceptance, and the final candidate's complete required-input/oracle accounting outside its conclusion.
