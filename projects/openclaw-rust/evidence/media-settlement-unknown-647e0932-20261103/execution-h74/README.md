# H74 exact-attempt source and native CI evidence

## Result and binding

**H74 has a failed source-authority run and failed native media groups on both Linux and Windows. The actual Linux App stages were skipped.** This collection preserves the failed controls and raw evidence; it does not assign the native failures to production logic, test expectations, or oracle implementation. That diagnosis belongs to the root source assessment.

| Binding | Exact identity |
|---|---|
| Repository | `bhrumom/fabushi-desktop` |
| Tested H | `74f11d3d31c6245ac5886dc0181460b2d4349744` |
| Tested tree | `aca91e1cfbedeecb039d442fb3528caac89873a3` |
| Fixed U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Fixed U tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Source run / attempt | `41580539540` / `1` |
| Runtime run / attempt | `41580539491` / `1` |
| Source workflow | ID `392577148`; `.github/workflows/ocrm-p0-authority.yml` |
| Runtime workflow | ID `362455251`; `.github/workflows/rust-desktop-runtime.yml` |

Both exact-attempt run API records bind H and its tree. The two downloaded native receipts independently bind H/tree/U/U-tree, runtime run `41580539491`, attempt `1`, and their respective job/OS. **These records belong only to H74. They are not evidence for the separate R3 metadata candidate or any later H.** Commit-message claims, scope prose, successful neighboring jobs, and prior-H evidence are not substituted for observed execution.

The initial inventory was read at approximately `2026-11-03T04:56:55Z`. One subsequent runtime inventory at `04:58:26Z` found the Windows job completed and its artifact available. The parent-requested closing macOS status read completed at **`2026-11-03T05:00:00.025Z`**. No failed job was rerun, dispatched, cancelled, or repeatedly queried for unchanged output.

## Closing job inventory

| Workflow / job | Job ID | Observed status and conclusion | Evidence qualification |
|---|---:|---|---|
| Source / `source-authority` | 125734077461 | Completed / failure, completed `04:52:47Z` | Complete decoded raw job log collected; source artifact count is zero. |
| Source / `recover-original-host-log` | 125734078782 | Completed / skipped | Metadata only. |
| Source / `persist-verified-inventory` | 125734407334 | Completed / skipped | Metadata only; no persistence success claimed. |
| Runtime / `rust-host` (`macos-15`) | 125734079336 | **Queued**, conclusion null at closing | No running step or case outcome. API supplies a `started_at` field but status remains queued; it is not execution evidence. |
| Runtime / `renderer` | 125734079487 | Completed / success | Status and artifact inventory only; renderer receipt contents were not audited in this collection. |
| Runtime / `OCRM media cleanup / Windows` | 125734079542 | Completed / failure, completed `04:57:31Z` | Complete native receipt, nineteen logs, ZIP, and final decoded job log collected. |
| Runtime / `OCRM installation key restore boundary / Linux` | 125734079551 | Completed / success | Status and artifact inventory only; no new test qualification made here. |
| Runtime / `sqlite-native-path` | 125734079585 | Completed / failure, completed `04:55:44Z` | Complete native media receipt, nineteen logs, ZIP, and decoded job log collected. Downstream App stages skipped. |
| Runtime / pressure profiler | 125734080804 | Completed / skipped | Metadata only. |
| Runtime / `platform-worker` | 125734080870 | Completed / skipped | Metadata only. |

The initial runtime run-level record returned `status: queued`, conclusion null while some jobs had already completed. This report uses the exact-attempt job records for individual outcomes and does not invent a terminal conclusion for the still-open runtime run.

## Source-authority failure

Source job `125734077461` failed step 9, **“Expand canonical source records and verify P0 G01/G02 structural”**, at `04:52:45Z`. The complete log is:

`logs/source-job-log-125734077461.log`

The relevant command sequence is visible at lines 292–304. `python3 projects/openclaw-rust/scripts/test_inventory_materialization.py` reports **21 tests, OK** at lines 321–326. The next command is:

```text
python3 projects/openclaw-rust/scripts/verify_audit_integrity.py --before-expansion
```

It raises:

```text
AssertionError: Missing reverse source/responsibility link: OCRM-RESP-MEDIA-FINAL-CUSTODY-001
```

The traceback is at log lines 327–337, with the failing link check in `projects/openclaw-rust/scripts/source_review.py:94`, called by `verify_audit_integrity.py:100`. The exact error timestamp is `2026-11-03T04:52:44.7921830Z`; line 338 records **process exit code 1**.

The shell uses `set -euo pipefail`. The subsequently printed `expand_inventory.py`, `shard_inventory.py`, `validate_p0.py --negative-probes`, and final integrity commands are part of the planned script; this log does not show them executing after the failure. Step 10, “Upload immutable authority evidence,” is skipped. The run is completed/failure and its artifact API response contains `total_count: 0`. There is therefore no H74 source structural receipt in this collection and no source G01/G02 acceptance claim.

This is the observed source-index/responsibility-link failure. No ledger, pending-addition record, source index, or capture code was edited during collection.

## Linux and Windows native receipt inventory

| Platform | Receipt job key | Artifact ID | Artifact name |
|---|---|---:|---|
| Linux | `sqlite-native-path` | 13205686727 | `ocrm-media-cleanup-Linux-74f11d3d31c6245ac5886dc0181460b2d4349744-41580539491-1` |
| Windows | `media-store-windows` | 13206046601 | `ocrm-media-cleanup-Windows-74f11d3d31c6245ac5886dc0181460b2d4349744-41580539491-1` |

Each ZIP contains **20 files: one receipt and nineteen Cargo logs**. Every archive member was extracted as bytes and accounted for without executing anything in the archive. Both ZIP byte lengths and SHA-256 values agree with the GitHub artifact API's size and digest.

Both receipts are `result: "failed-or-incomplete"`. Each planned and executed all **19 groups**. All nineteen command captures have `capture_exit: 0` and no recorded `execution_error`. Two groups per platform have `cargo_exit: 101`; the remaining seventeen have `cargo_exit: 0`. The enclosing media capture step exits **1** on both platforms (Linux job log line 17693; Windows job log line 18653). Successful output capture does not turn either failed Cargo command into a pass.

The raw Cargo logs establish **202 passed / 8 failed on Linux** and **196 passed / 9 failed on Windows**, across the requested groups. Every raw summary has zero ignored tests. Those totals are execution counts, not overall native acceptance. The failed named controls remain failed.

### All nineteen command groups

Every command starts with:

```text
cargo test --locked --manifest-path source/host/Cargo.toml
```

The table gives the following selector/flags. Unless marked otherwise, each ends with `-- --test-threads=1`. The exact complete argv arrays, cwd, exits, raw names, summary lines, and hashes are preserved in `accounting/native-receipt-accounting.json` and the original receipts.

| Group | Selector / special suffix | Linux pass / fail | Windows pass / fail | Cargo exit Linux / Windows |
|---|---|---:|---:|---|
| `media-settlement-owner` | `--lib extensions::session::agent_db::media_settlement_tests::` | 11 / 2 | 11 / 2 | 101 / 101 |
| `outgoing-media-custody` | `--lib attachment_paths::tests::` | 5 / 0 | 5 / 0 | 0 / 0 |
| `transaction-commit-owners` | `--test openclaw_sessions_and_config_contract commit` | 21 / 6 | 20 / 7 | 101 / 101 |
| `cleanup-owner` | `--lib extensions::attachments::media_store::cleanup::tests::` | 7 / 0 | 8 / 0 | 0 / 0 |
| `cleanup-consumers` | `--test openclaw_media_store_contract` | 17 / 0 | 16 / 0 | 0 / 0 |
| `html-policy` | `--test openclaw_html_provenance_contract` | 12 / 0 | 9 / 0 | 0 / 0 |
| `host-read-format` | `--test openclaw_channel_and_local_roots_contract test_ocrm_host_read_media_policy_format_admission`; suffix `-- --exact --test-threads=1` | 1 / 0 | 1 / 0 | 0 / 0 |
| `file-url-admission` | `--lib extensions::attachments::web_media::file_url_admission_tests::` | 5 / 0 | 6 / 0 | 0 / 0 |
| `explicit-html-consumers` | `--test openclaw_web_media_contract` | 28 / 0 | 26 / 0 | 0 / 0 |
| `image-probe-consumers` | `--test openclaw_image_ops_contract` | 16 / 0 | 16 / 0 | 0 / 0 |
| `bounded-reader-owner` | `--lib extensions::attachments::media_reference::bounded_read_tests::`; suffix `-- --test-threads=1 --show-output` | 11 / 0 | 8 / 0 | 0 / 0 |
| `bounded-reader-public` | `--test openclaw_media_reference_contract` | 17 / 0 | 17 / 0 | 0 / 0 |
| `bounded-reader-box-consumer` | `--test attachments_service_contract service_upload_read_text_chunk_and_gateway_use_single_agent_media_store`; suffix `-- --exact --test-threads=1` | 1 / 0 | 1 / 0 | 0 / 0 |
| `bounded-reader-image-consumer` | `--test attachments_service_contract image_reads_and_persistence_preserve_content_addressed_dimensions`; suffix `-- --exact --test-threads=1` | 1 / 0 | 1 / 0 | 0 / 0 |
| `bounded-reader-channel-consumers` | `--test channel_attachment_contract` | 7 / 0 | 8 / 0 | 0 / 0 |
| `local-file-url-owner` | `--test local_media_file_url_contract` | 10 / 0 | 11 / 0 | 0 / 0 |
| `file-context-consumers` | `--test openclaw_media_file_context_contract` | 10 / 0 | 10 / 0 | 0 / 0 |
| `send-message-consumers` | `--test send_message_tool_contract` | 19 / 0 | 19 / 0 | 0 / 0 |
| `agent-management-consumers` | `--test sand_agent_management_tools_contract` | 3 / 0 | 3 / 0 | 0 / 0 |

The OS-specific retained positive-name counts are taken from each receipt and checked against its own raw output. They are not assumed equal across platforms. All raw passed-name lists agree with the receipts. There are no extra positive names. For passing groups, all required names occur exactly once; for failing groups the missing positives are precisely the actual failed controls named below.

### DB13: same two failing assertions on both platforms

The exact failed names are:

- `extensions::session::agent_db::media_settlement_tests::media_settlement_fault_matches_three_identities_and_only_its_exact_owner`
- `extensions::session::agent_db::media_settlement_tests::media_settlement_lost_commit_acknowledgement_retires_without_publication`

The first panic is at `src/extensions/session/agent_db.rs:9577:9` (Windows uses backslashes in the printed path). The second is at `:9362:13`.

For both panics on both platforms, the raw assertion shows:

- **Actual left side:** the `media-send` send-message record **and** an `outgoing-media-settlement:media-work` record, with kind `outgoing-media-settlement`, lifecycle revision, the owned media path, work ID, session key, and `transcriptEntryIds: ["media-send"]`.
- **Expected right side:** only the `media-send` send-message record.

The complete raw values, including platform-native owned paths, remain in the logs. No production/test correction is prescribed from this comparison alone.

| Platform | Raw file | Failure detail and summary |
|---|---|---|
| Linux | `artifacts/linux-media-13205686727/media-settlement-owner.log` | Lines 707–720 contain the two panics and values; line 727: **11 passed, 2 failed, 0 ignored**, 719 filtered out. |
| Windows | `artifacts/windows-media-13206046601/media-settlement-owner.log` | Lines 819–832 contain the two panics and values; line 839: **11 passed, 2 failed, 0 ignored**, 706 filtered out. |

These are executed assertion failures after successful compilation and harness startup. They are not compilation errors or a failure to capture output. The named lost-ACK test remains failed: its name and persisted-row diagnostic must not be converted into positive evidence for every lost-ACK/no-publication assertion, or for assertions after the panic.

The other eleven DB names have positive raw lines, including the deferred COMMIT/rollback-denial, real-COMMIT-error, real-rollback-denial, retirement, scope/lease, known-refusal, and exact-replay controls. Their recorded scope remains that of the native DB cases; they do not establish the actual App or generated Runner consumer.

### COMMIT27: six shared failures and one additional Windows failure

All following paths are `tests/openclaw_sessions_and_config_contract.rs` in the tested H74 source. The table records the panic location/message, not a root-cause attribution.

| Failed control | Panic source coordinate | Raw panic message | Platforms |
|---|---|---|---|
| `channel_commit_oracle_rejects_hidden_owner_and_aliased_calls_in_actual_source` | `2619:7` | `prove only the media exception before exercising the unchanged channel oracle` | Linux, Windows |
| `channel_commit_oracle_retains_unsafe_owners_before_after_and_inside_helper_impl` | `2471:7` | Same message as above | Linux, Windows |
| `media_commit_oracle_accepts_actual_owner_and_preserves_admission_guards` | `1889:10` | `the actual media command and consumer must establish the bounded exception` | Linux, Windows |
| `media_commit_oracle_preserves_raw_commit_prohibition_outside_proven_method` | `2097:14` | `the unchanged bounded adapters still prove their own handling` | Linux, Windows |
| `test_agent_db_has_no_direct_commit_error_propagation` | `2360:10` | `both command adapters must return to their individually checked transaction owners` | Linux, Windows |
| `test_commit_error_oracle_rejects_unguarded_propagation_and_missing_cleanup` | `2411:7` | `prove only the media exception before exercising the unchanged channel oracle` | Linux, Windows |
| `media_commit_oracle_rejects_missing_duplicate_or_conditional_owner` | `1945:59` | `actual media signature` | **Windows only**; Linux has a positive line for this exact control. |

Linux `transaction-commit-owners.log` lines 786–815 contain its six panic contexts; line 826 records **21 passed, 6 failed, 0 ignored**, 40 filtered out. Windows lines 834–868 contain seven panic contexts; line 880 records **20 passed, 7 failed, 0 ignored**, 40 filtered out.

The shared failures occur at the printed bounded-media-exception proof expectations. The Windows-only failure occurs at the printed signature expectation. This collection does not decide which implementation or expectation is responsible, and does not classify them as a test flaw. The original failed controls and full raw output remain available for the separate source assessment.

### Custody5 and Tool19: observed positives with bounded scope

Both platforms run and pass all five custody tests. The exact positives include `unknown_owner_settlement_retains_bytes_through_abort_and_drop`, `sealed_custody_refuses_new_paths_without_reopening_retained_media`, `confirmed_refusal_restores_only_its_own_cleanup_authority`, ordinary rollback cleanup, and committed-media preservation.

Both platforms run and pass all nineteen Tool tests. In particular, the raw logs contain one positive result each for:

- `send_message_bridge_preserves_prepared_custody_on_unknown_resolution`
- `send_message_bridge_aborts_prepared_custody_on_known_resolution_error`

These Tool controls establish their recorded error-class behavior, not an actual SQL fault reaching the App. The native custody tests likewise do not replace the App's Session first-open/reopen and consumer evidence.

## Actual App, feature, LFA, and SGF status

Linux job step 8 (“Execute media cleanup owner and real service contracts”) fails after all nineteen native groups are captured. Step 9 successfully uploads the native media artifact. Steps 10–23 then have `conclusion: skipped`, including:

- Step 18: typed Automation admission and actual owners.
- Step 19: **“Check and execute the actual Automation, Settings and channel App owners on Linux.”**
- Step 20: historical prior-root recovery through current Host and shipping App.
- Step 21: native link-preview HTTP, Coordinator, and App dispatch.
- Step 23: native SQLite/App evidence upload.

Consequently this H74 run supplies **no execution observation** of the App shipping check, test-support feature reach, App harness compile/link, **MSU8**, **LFA6**, or **SGF3** through that App stage. Their state is skipped/unobserved, not failed App assertions and not a successful repair confirmation. No App command exit code can be assigned because the stage did not execute. There is no actual App receipt in either downloaded native media ZIP.

The native receipts' scope descriptions mention separate required App evidence. Those descriptions are contract prose, not evidence that the App ran. Native passing tests, the H74 LFA import/type correction, and renderer status do not close this gap.

## Input and raw-byte accounting

Each native receipt has rich **`inputs_before` and `inputs_after` maps containing 62 paths**. This differs from the earlier A2 automation receipt's single `inputs_sha256` map; no earlier shape is assumed here.

For both platforms:

- The before and after maps are equal, and the receipt records `inputs_unchanged: true`.
- Every `normalized_worktree_blob_sha` equals that path's `git_blob_sha`.
- The Linux and Windows path sets and Git blob identities are equal for all 62 paths.
- All 62 raw `working_tree_sha256` values differ across the platform checkouts. The receipt's normalization facts are retained separately from raw-byte hashes; this audit does not use those differences to assign the Windows-only failure's cause.
- All nineteen recorded log SHA-256 values agree with the raw artifact-member bytes.

Both receipts also record the capture oracle's thirteen synthetic qualification controls, all with observed values equal to expected and `accepted: true`. They cover positive/CRLF-positive, missing, duplicate, unexpected, failed, ignored, zero-test, partial-log, duplicate-summary, failed-summary, nonzero-command, and capture-error controls. This is **synthetic capture-oracle qualification**, not product behavior. The two failed native groups are correctly retained as `oracle_accepted: false`.

The failing groups' receipt `summaries_passed_failed_ignored` arrays are empty even though the complete raw logs contain `test result: FAILED` summaries. This collection uses the actual raw failed summaries shown above; it does not mistake the empty accepted-summary arrays for missing/truncated execution or zero failures.

### Raw hash ledger

Paths are relative to `/workspace/scratch/7d27ea239237/media-h74-ci/`. Hashes are SHA-256 over stored bytes.

| File | Bytes | SHA-256 |
|---|---:|---|
| `logs/source-job-log-125734077461.log` | 28,920 | `31f25cb8af36bfb77ebfbd4179ea5728774379f2cf70f7dddebf2e1f2fc9dd1c` |
| `logs/linux-job-log-125734079585.log` | 1,220,138 | `4630d79b0ecbdbd01cf5f0ce4fd2f34a4696fed8e20dbff0501d529f125acf43` |
| `logs/windows-job-log-125734079542-final.log` | 1,281,835 | `44dd336b57476e793afa35134517b0fe888fd0d976eb583ddb5f0991dc7d4abc` |
| `artifacts/h74-linux-media-13205686727.zip` | 142,379 | `284a316258ebf9b60dd29d781c6aceb44ef3d1ca351a3337e0945efd7b2699b5` |
| `artifacts/h74-windows-media-13206046601.zip` | 149,588 | `184296dcafd2b230021ed8523d4e159cc34473d4436a5a9f5198c28c1bcac96f` |
| `artifacts/linux-media-13205686727/receipt.json` | 120,067 | `5fc8dfd24cf2635e253056c12d06c1f67f29e66f5cabe9e3003a1efc0bdfc6e2` |
| `artifacts/windows-media-13206046601/receipt.json` | 121,109 | `c79d4e0d447831aae967a80be6d74b5c39b71c0daa64d18ac0595f151f563ce8` |
| `artifacts/linux-media-13205686727/media-settlement-owner.log` | 30,567 | `26ab3027d0bd03ff450927df814349347f9fb104f0b8ec2677b4126aa838fad8` |
| `artifacts/windows-media-13206046601/media-settlement-owner.log` | 34,262 | `c73d70c1d9829a1b8404f8e888e048c4a5a997c63f4f0ead5221f6e4b3cffcb9` |
| `artifacts/linux-media-13205686727/transaction-commit-owners.log` | 32,185 | `921b7317adf72cfad5ddd22b541f1e366a4d8beddc48214b913f642fbaaf1bc5` |
| `artifacts/windows-media-13206046601/transaction-commit-owners.log` | 33,774 | `2e25b24f4252140b8d4c9f063b62b9d596b62d2ed833fc3b2afef896fa922bf3` |
| `artifacts/linux-media-13205686727/outgoing-media-custody.log` | 25,266 | `12fd5519dac296210ecc857ac42b461c5354c87921570426f54783ad61a6f19c` |
| `artifacts/windows-media-13206046601/outgoing-media-custody.log` | 28,779 | `b721149c77bc329b06c084ecfe2ad5fee214793f7ec3462e2e6de607ce22c623` |
| `artifacts/linux-media-13205686727/send-message-consumers.log` | 28,957 | `3894e838fd5f8259607b1ffa8681b92b3eba3b4c477e30da4cb94b3da82e7df5` |
| `artifacts/windows-media-13206046601/send-message-consumers.log` | 30,212 | `c9cba2ad7f3adc4ab960a3efc7493920408a4a44e272984ba361dc594c980c06` |
| `accounting/native-receipt-accounting.json` | 122,036 | `29d2627a35b315c820847020a51650354d6d30a21864dda09843d01dc9e0dc8f` |

The source/Linux job logs are the connector-decoded text saved as UTF-8 with their leading BOM and LF preserved. The Windows final job log was saved from the connector's JSON text value by UTF-8 encoding, retaining its leading BOM and **18,723 CRLF line endings**. These hashes identify that decoded representation; the ZIP hashes identify original artifact archive bytes. The extracted member hashes identify the original bytes inside each ZIP.

An initial Windows in-progress log request at `04:57:04.966Z` returned `404 BlobNotFound`. That response is preserved under `metadata/windows-job-log-125734079542-initial.json`. No case outcome was inferred from it. After terminal completion, a new final-log request at `04:58:31.455Z` succeeded. The final logs' last actual timestamps are source `04:52:45.4171389Z`, Linux `04:55:41.8347772Z`, and Windows `04:57:30.5180471Z`; no later case outcomes are extrapolated.

The native artifact inventory files record every member hash. `accounting/evidence-files-sha256.json` records the saved collection's file hashes, excluding that manifest itself. Metadata fetch records retain request URLs and fetch times; other runtime artifacts are inventoried without being treated as audited passes.

## Remaining scope and next permitted read

The concrete unresolved H74 failures are the source reverse-link assertion and the named DB/COMMIT controls above. Their production-versus-expectation/oracle cause remains under separate source assessment. The skipped App stage leaves shipping/test feature observation, actual App MSU8/LFA6/SGF3, Session first-open/reopen, and generated Runner consumer execution unobserved here. Passing native custody/Tool controls do not close those dependencies.

At the closing `05:00:00.025Z` read, macOS job `125734079336` is still queued. The next useful read, when independently resumed after a material status change, is the **same runtime attempt-1 jobs endpoint** or that known job's step metadata, followed by its final logs/artifacts only when available. There is no reason to poll the already collected failed source/Linux/Windows jobs for unchanged output, and none were rerun.

This is exact-H evidence collection and data accounting, not unit, platform, MSU, or full acceptance. The only writes are beneath `/workspace/scratch/7d27ea239237/media-h74-ci/`. No production/spec/ledger/shared-work file was read for diagnosis or changed; no project build, test, parser, validator, schema/generator, test listing, capture execution, htch action, commit, push, dispatch, rerun, cancellation, or agent spawn occurred locally. All product/capture command outcomes reported above came from the downloaded GitHub Actions evidence.
