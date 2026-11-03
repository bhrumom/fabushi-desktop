# H23 App failure investigation — bounded readback at 2026-11-03 04:38–04:39 UTC

## Confirmed failure and subject

- Repository: bhrumom/fabushi-desktop.
- H23: 23c2a567d874509f56353b8cae737cdf1437309d.
- Tree: b10485b13c15cd1cf012a5f51acaf6927aeb322a.
- Fixed U: 1440bb1771372334bea23d7642bb52deb7db1845.
- Runtime run 41576693105, attempt 1.
- Job 125721731498: https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/job/125721731498
- At the 04:38:00 UTC API snapshot, the job remained in_progress / conclusion null.
- Step 19, "Check and execute the actual Automation, Settings and channel App owners on Linux", started 04:31:41Z and completed 04:36:44Z with conclusion failure.
- Step 20, historical prior-root recovery through current Host and shipping App, was skipped at 04:36:44Z. Exact H23 workflow gates this step on automation_app.outcome == success; this is a dependent skip, not a separately observed failing test.
- Step 21, native link-preview HTTP / Coordinator / app dispatch, was in_progress from 04:36:44Z. Its workflow condition uses always() plus native-path success, so continuing that step does not erase the App failure.
- Step 22 preservation and step 23 final artifact upload were pending.
- macOS rust-host job 125721731668 remained queued, with no steps or assigned runner.

## Original data actually available

The job-log connector was invoked once. It returned a decoded log prefix of 4,256,256 characters, preserved unchanged as a UTF-8 snapshot:
sqlite-native-path-125721731498-043800-snapshot.raw.log

SHA256:
92cb8a60c8929d2512cb80cef1cdb9fe856d4ab220ba713c690d4efbe09671f0

The latest timestamp in those returned bytes is 2026-11-03T04:30:11.7801745Z. The content stops inside the printed portable receipt, before App step 19 started. It contains no App-period command result, compiler diagnostic, assertion failure or failure traceback. No claim is made that this prefix is the complete job log; the cause of this incomplete returned prefix is not established.

The artifacts API still listed seven artifacts, all already observed before this failure. No final ocrm-sqlite-native-path artifact or automation-app failure artifact was published in this snapshot.

The exact-H check-runs API returned the matching check run with:
- status in_progress, conclusion null.
- output title, summary and text null.
- annotations_count 0.

The same insufficient log action was not repeated. Existing portable success evidence remains valid in its own scope and is not recast as App success.

## Exact H23 command sequence and diagnostic destinations

The read-only workflow source .github/workflows/rust-desktop-runtime.yml at H23 defines the failed step beginning at its automation_app entry. It runs a Python capture block; the block was read, not executed locally.

Environment in that step:
- CARGO_TARGET_DIR = GitHub workspace/source/host/target.
- CARGO_BUILD_JOBS = 1.
- TMPDIR = NATIVE_PATH_FIXTURES.
- FABUSHI_AUTOMATION_CONTRACT_ARTIFACT_ROOT = NATIVE_PATH_FIXTURES/automation-app.

Declared sequence:
1. shipping-check:
   cargo check --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host
2. test-build:
   cargo test --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host --no-run --message-format=json
3. Select exactly one real App test executable for source/host/app/src/main.rs under the owned CARGO_TARGET_DIR.
4. For each required name, run the exact executable with --exact <name> --list, then --exact <name> --nocapture --test-threads=1.
5. After all required App cases, startup-admission:
   cargo test --locked --manifest-path source/host/app/Cargo.toml --test startup_admission_contract -- --test-threads=1

The stage includes actual Automation, Settings, channel, MCP catalog, cancellation, device, SendMessage and local-media App cases. The failed subcommand or required case cannot be identified from the available prefix.

The authoritative failure data should be in the final artifact:
- automation-app/receipt.json: commands[].argv, commands[].exit, stdout/stderr paths and digests, accepted cases, result and full error traceback.
- automation-app/shipping-check.stdout.log and shipping-check.stderr.log.
- automation-app/test-build.stdout.log and test-build.stderr.log.
- automation-app/case-N-list.stdout.log / stderr.log and case-N-run.stdout.log / stderr.log for whichever cases were reached.
- automation-app/startup-admission.stdout.log / stderr.log if reached.
- owned-fixtures.tar from the failure-preservation step, if preservation succeeds.

The final expected artifact name is:
ocrm-sqlite-native-path-23c2a567d874509f56353b8cae737cdf1437309d-41576693105-1

Source locations established here are the workflow capture/command definitions and the selected target source/host/app/src/main.rs and tests. A concrete diagnostic source file/line has not yet been exposed; no such location is invented.

## Relation to later commits and current cron candidate

GitHub commit APIs establish this lineage:
H23 -> bb5a7f6098f952df6d388a672c4fb01045bc645c -> 972779ecd290b90e407ffe651aa5c456c31e9f7f -> d70f86f33b5e3c67545954b270373e1548809f4b -> later commits.

- bb5 is the direct child of H23. It changes Rust Plugin Skills/MCP auth-recovery owners, their tests and their workflow/capture selections. Its file list does not change source/host/app/src/main.rs or App Cargo manifests. It can affect future dependent builds via Host code; it did not execute in this H23 run.
- d70 changes four Electron MCP/OAuth TypeScript files, two specs and capture_prior_root_contracts.py. It does not change the App Rust target or Cargo manifests. Its prior-root capture change concerns the following step, which H23 skipped because App failed.
- Canonical ref during the investigation was 871b033c5b9aabbe322cc7322af92648939db9b5. This live state is not substituted for the failed H23 subject.
- The primary integrator's current unpublished cron candidate is outside the H23 checkout. It therefore cannot have introduced this exact-H failure. No diagnostic is available to decide whether that candidate happens to repair the same pre-existing defect, or whether bb5/d70 do.
- The local reference checkout is an older detached object-reading checkout, not the primary integrator's current candidate. No comparison to its working-tree contents is represented as a candidate review.

## Classification and repair handoff

Confirmed result: App step execution failed; dependent prior-root execution was skipped.

Cause classification: pending original diagnostic. The current evidence does not distinguish:
- production or test-compilation failure;
- an actual App assertion/invariant failure;
- exact-selection/result-oracle failure;
- environment/resource/toolchain failure;
- capture or other execution error.

No IMPL_BUG, TEST_FLAW or environment verdict is assigned without the corresponding raw command and error. Five minutes of elapsed step time and a failure conclusion do not identify the cause.

No concrete code repair is justified yet. The next bounded read is the final artifact's automation-app/receipt.json plus the stdout/stderr of its first failed command (or the traceback after successful commands). Compare those recorded input hashes to H23 before assessing subsequent deltas. If all commands completed successfully and the receipt rejects selection or summaries, inspect that exact oracle failure; if a command is nonzero, use its original diagnostic and source location. Preserve the passing portable/plugin/native subsets and do not rerun them solely because the later App step failed.

No builds, tests, lint, product parser/generator/validator/harness or repository scripts were executed locally or on htch. No production/workflow/ledger edits or push occurred. All writes are inside the authorized evidence directory; actual model/effort remain inherited.
