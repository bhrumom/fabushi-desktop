# H74 Linux restore, Windows media and macOS evidence

## Bounded verdict

One fresh attempt-1 job metadata read and one artifact-list read were made. Only the two completed jobs were collected. No status polling, rerun, dispatch or source/ledger/schema/workflow edit was performed.

| Job | ID | Observed result |
| --- | --- | --- |
| Linux installation-key restore | 125734079551 | **Completed success**, 04:51:43–04:54:19 UTC |
| Windows media cleanup | 125734079542 | **Completed failure**, 04:51:43–04:57:31 UTC |
| macOS `rust-host` | 125734079336 | **Queued**, no conclusion, no steps, runner ID 0 |

All belong to runtime run **41580539491, attempt 1**, H74 **`74f11d3d31c6245ac5886dc0181460b2d4349744`**, tree **`aca91e1cfbedeecb039d442fb3528caac89873a3`**. Both original receipts bind fixed U **`openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`**, tree **`90c8aab39972da11cfad220361781d49e1bc6700`**.

No H23 result was imported. No result is transferred to a descendant carrier or another H. macOS's populated API timestamp is not execution proof: the job remains queued without a runner or step history.

## Linux restore: scoped positive evidence

Original artifact **13205531160**, 47,233 bytes:

```text
e3f38fdea40e2264349e23c6c6830d59192c01dfe1525aaa1c1f47bdcb9a4599
```

The ZIP digest matches GitHub metadata. Original files are under `restore-artifact/`: `subject.json`, `compiled.json`, `execution.json`, `compiler-artifacts.jsonl`, and three raw execution logs.

The compile command was:

```text
cargo test --locked --manifest-path source/host/Cargo.toml \
  --test box_store_download_frozen_parity_contract \
  --no-run --message-format=json
```

It exited **0**, using Rust/Cargo **1.97.1**, host `x86_64-unknown-linux-gnu`, compiler UID **1001**. The compiler-artifact stream names exactly the selected test binary and reports `fresh: false`. Its hash matches the compiled receipt:

```text
compiler-artifacts.jsonl:
02395e80a670f4016b312b3c1653d7c984d47362f2e651fe2069a5d3b91856f7
```

The recorded executable hash is `7e94797e865d7e4f1f94ecc8f3dbc3d6633e2e289ded004fdc53123099f0bf24`. The execution workflow checks that hash before invoking the binary. The archive contains the compiler stream and receipts, not executable bytes; this review therefore does not claim an independent binary rehash.

| Invocation | Passed | Failed | Ignored | Exit |
| --- | ---: | ---: | ---: | ---: |
| Selected binary `--nocapture`, ordinary user | 29 | 0 | 2 | 0 |
| Exact mounted-descendant file case | 1 | 0 | 0 | 0 |
| Exact mounted-descendant symlink case | 1 | 0 | 0 | 0 |

The two cases initially ignored in the ordinary invocation were subsequently **actually executed**:

- `linux_bind_mounted_descendant_file_restore_boundary`
- `linux_bind_mounted_descendant_symlink_restore_boundary`

Each used `sudo -n unshare --mount --propagation private -- <selected-binary> --exact <case> --ignored --nocapture` and produced one nonignored positive. The ordinary invocation's two ignored cases are not silently reclassified as ordinary passes; the separate privileged executions supply their own positive evidence.

All three original log hashes match the receipt. The embedded `compiled` object equals original `compiled.json`. The recorded Host manifest and lock hashes match exact H74 Git bytes:

- Manifest: `a6d32331fcb2c0b765ccd562a2b119ae2fb5190fa57039e42ab97bcba1681ddb`.
- Lock: `96b7c7e3faa9faf3aa54b800dccb17ae7a0ecebf59ca7e2e1a948b9ae0545eaf`.

The CS06-ROOT/DELETE source contract and exact workflow were read. This proves the selected ordinary restore and real private-namespace mount controls using synthetic installation keys and owned fixtures. It does not prove exclusion of concurrent external namespace mutation, complete backup/client compatibility, macOS behavior, complete dependency closure or final migration acceptance. The receipt retains `independent_acceptance: pending`.

## Windows media: failed overall, retained executed positives

Original artifact **13206046601**, 149,588 bytes:

```text
184296dcafd2b230021ed8523d4e159cc34473d4436a5a9f5198c28c1bcac96f
```

Its ZIP digest matches GitHub metadata. `windows-artifact/receipt.json` is the unchanged original receipt; the other 19 artifact files are its original per-command logs. Receipt result is **`failed-or-incomplete`**.

Job step 6, **Execute media cleanup owner and real service contracts**, failed. Its capture owner continued through **all 19 planned commands in declared order**: 17 were accepted and 2 failed. All **19 capture exits were 0**; the failed cargo commands returned **101**, and the enclosing job step returned **1**. The always-run artifact upload succeeded. Post-cache saving was skipped; this job had no later App/Session/native-dispatch stages to count as executed.

All commands use `cargo test --locked --manifest-path source/host/Cargo.toml`, their declared `--lib` filter or `--test` target, and `--test-threads=1`; the exact arrays are retained in `windows-execution-readback.json` and the original receipt.

| Selection | Passed | Failed | Ignored | Cargo exit |
| --- | ---: | ---: | ---: | ---: |
| media-settlement-owner | 11 | **2** | 0 | 101 |
| outgoing-media-custody | 5 | 0 | 0 | 0 |
| transaction-commit-owners | 20 | **7** | 0 | 101 |
| cleanup-owner | 8 | 0 | 0 | 0 |
| cleanup-consumers | 16 | 0 | 0 | 0 |
| html-policy | 9 | 0 | 0 | 0 |
| host-read-format | 1 | 0 | 0 | 0 |
| file-url-admission | 6 | 0 | 0 | 0 |
| explicit-html-consumers | 26 | 0 | 0 | 0 |
| image-probe-consumers | 16 | 0 | 0 | 0 |
| bounded-reader-owner | 8 | 0 | 0 | 0 |
| bounded-reader-public | 17 | 0 | 0 | 0 |
| bounded-reader-box-consumer | 1 | 0 | 0 | 0 |
| bounded-reader-image-consumer | 1 | 0 | 0 | 0 |
| bounded-reader-channel-consumers | 8 | 0 | 0 | 0 |
| local-file-url-owner | 11 | 0 | 0 | 0 |
| file-context-consumers | 10 | 0 | 0 | 0 |
| send-message-consumers | 19 | 0 | 0 | 0 |
| agent-management-consumers | 3 | 0 | 0 | 0 |

These command summaries total **196 passing and 9 failing executions**, zero ignored; this is a count of selected executions, not a project-coverage percentage. Filtered-out tests remain outside each selected command's proof.

The two media-settlement failures are:

- `media_settlement_fault_matches_three_identities_and_only_its_exact_owner`, `agent_db.rs:9577`.
- `media_settlement_lost_commit_acknowledgement_retires_without_publication`, `agent_db.rs:9362`.

Both original panic values show the observed transcript includes the additional `outgoing-media-settlement:media-work` row while the asserted expected value contains only the send-message row. The parent owns disposition of those failures.

The seven transaction-owner failures and complete original panic blocks are preserved in `windows-seven-failures-original.log`. Windows has **seven**, not the six supplied by the parent's Linux report. The additional Windows case is `media_commit_oracle_rejects_missing_duplicate_or_conditional_owner`, which panics at `openclaw_sessions_and_config_contract.rs:1945:59` with `actual media signature`.

[WINDOWS-EXTRA-FAILURE.md](WINDOWS-EXTRA-FAILURE.md) gives the source/byte diagnosis: LF-only multiline fixture search versus the actual CRLF Windows source loaded by `include_str!`. That one extra failure is a source-fixture **TEST_FLAW**; the diagnosis does not dismiss the six common failures or the two MSU failures. Only literal byte comparison and official language documentation were used; no local Rust/parser/test execution occurred.

The 13 original synthetic capture-oracle qualification controls are accepted. They prove qualification of the evidence recognizer only, not product behavior.

## Windows exact source identity

The original Windows receipt binds **62 inputs before and after execution**, with unchanged values and normalized Git identities. Every one was independently compared with H74's immutable tree:

- **62/62** Git blob identities match H74.
- **62/62** before/after metadata records match.
- **62/62** Windows working-tree SHA-256 values and byte sizes are reproduced exactly by CRLF representation of the corresponding Git bytes.
- No mismatch was found. Source hash equality is not a claim of semantic acceptance.

The restore manifest/lock comparisons and two supplementary exact-source captures are included in the same `source-input-identities.json`. Its 64 paths do not constitute a complete transitive dependency closure.

All 19 Windows raw-log hashes match the original receipt, and literal successful-test name lists match exactly with no duplicate positives. Failure-summary fields in the receipt are empty because its positive recognizer accepts only successful summaries; the failed **11/2** and **20/7** counts above were read directly from the original failure logs.

Windows scope remains the declared native media/byte/admission/cleanup and file-backed AgentDb selections under controlled faults. Its original receipt explicitly does not prove actual App/Session first-open recovery, generated Runner consumers, live services, hostile Windows open deadlines, full codecs, native 32-bit, packaged or whole-media acceptance.

## Raw-log custody and inventory

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `restore-job-log.log` | 91,557 | `ac08021e574e569a58c21c042b3cae222a3af923def580cc96c9fdfefd05f2f8` |
| `windows-job-log.log` | 1,281,835 | `44dd336b57476e793afa35134517b0fe888fd0d976eb583ddb5f0991dc7d4abc` |

These are the unchanged UTF-8 bytes returned by the GitHub decoded-log connector, preserving BOM and line endings. No authenticated download URLs were saved. `evidence-integrity.json` records archive, original log, compiler and execution comparisons; `FILE-INVENTORY.tsv` inventories all retained files except itself.

All eight global migration flags remain false; none was promoted. Source-authority, renderer and Linux media-job evidence are separately owned reports. The bounded additional collection is complete; macOS receives no passing inference and any later follow-up remains with the parent.

