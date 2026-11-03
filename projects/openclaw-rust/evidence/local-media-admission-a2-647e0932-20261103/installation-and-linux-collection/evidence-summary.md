# Exact-H Actions evidence collection

## Result at the collection boundary

At the single later read, **2026-11-03 04:28:09 UTC**, installation-key job **125721250213** had completed successfully; Linux `sqlite-native-path` job **125721250552** was still running step 12; its actual App step 19 remained pending. macOS `rust-host` job **125721250429** remained queued with no steps. The six actual App LFA tests therefore have **no execution result in this collection**. This is an evidence collection, not an implementation review or acceptance promotion.

The initial API run/artifact reads were at 04:24:42 UTC, with the successful exact-attempt job read at 04:24:50 UTC. The only later refresh was at 04:28:09 UTC. There was no further polling, rerun, or idle wait.

## Exact subject and primary readbacks

| Identity | Value |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Workflow | `Rust desktop runtime`, `.github/workflows/rust-desktop-runtime.yml` |
| Head H | `a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74` |
| Head tree | `a0dceefeae75fccd7ab5aec143f6d651f7cf2fd3` |
| Upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Run / attempt | `41576543683 / 1` |
| Branch | `codex/openclaw-rust-fusion-chat-20261101-writer` |
| Collector | `/root/media_admission_general_r7`; inherited configured GPT-6 Astra Pro, launch-applied xhigh per parent confirmation; no independent worker introspection |

The [workflow run](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576543683), exact-attempt job API, both collected artifact receipts, and artifact API metadata agree on H/run/attempt. The run API head commit also supplies the tree above. The workflow aggregate was still `queued` with no conclusion; that aggregate is not used as a test result.

Raw complete readbacks and timestamp receipts are saved as `run_initial.json`, `jobs_initial.json`, `artifacts_initial.json`, `run_later.json`, `jobs_later.json`, `artifacts_later.json`, and their receipt files. Their API sources are:

- [Run API](https://api.github.com/repos/bhrumom/fabushi-desktop/actions/runs/41576543683)
- [Attempt 1 jobs API](https://api.github.com/repos/bhrumom/fabushi-desktop/actions/runs/41576543683/attempts/1/jobs?per_page=100)
- [Run artifacts API](https://api.github.com/repos/bhrumom/fabushi-desktop/actions/runs/41576543683/artifacts?per_page=100)

Direct job-detail URLs were rejected by the connector URL allowlist; those error readbacks were preserved. The supported exact-attempt jobs endpoint returned all seven jobs. Renderer job 125721250334 was not investigated by this collector.

## Installation-key restore evidence

[Job 125721250213](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576543683/job/125721250213), `OCRM installation key restore boundary / Linux`, ran from 04:18:07 to 04:20:24 UTC and completed `success`. This observation is supported by full decoded job output and the complete artifact, not just the job conclusion.

The preserved full decoded job log is `installation-job-125721250213.full.log`: 88,584 UTF-8 bytes including its original BOM, SHA-256 `4fa173ea2c5803348a4288f2a6b7bb7ffa35df00588628858bda0583d2d9988f`. The connector returned 88,582 decoded characters and a final newline.

Artifact **13204935336**, `ocrm-config-restore-linux-a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74-41576543683-1`, is saved as `installation-artifact-13204935336.zip`. Its 47,136 bytes have SHA-256 `cdef2325a900c10a1656c86116da7198219920bc4fe84b1ccab3bcad279fbc5e`, matching the GitHub artifact digest. All seven ZIP members are preserved in `installation-artifact-13204935336.members/`, with byte counts, hashes, and ZIP accounting in the accompanying accounting file.

The recorded locked compile command was `cargo test --locked --manifest-path source/host/Cargo.toml --test box_store_download_frozen_parity_contract --no-run --message-format=json`, with exit 0 under ordinary runner UID 1001. The artifact records Rust/Cargo 1.97.1 on `x86_64-unknown-linux-gnu`. Its compiler-artifact JSON identifies the same executable used by all three invocations:

| Invocation | Exit | Raw result |
| --- | ---: | --- |
| Ordinary restore binary, `--nocapture` | 0 | 29 passed; 0 failed; 2 ignored |
| `linux_bind_mounted_descendant_file_restore_boundary`, `--exact --ignored --nocapture`, through `sudo -n unshare --mount --propagation private` | 0 | 1 passed; 0 failed; 0 ignored |
| `linux_bind_mounted_descendant_symlink_restore_boundary`, same private mount namespace command | 0 | 1 passed; 0 failed; 0 ignored |

The two ordinarily ignored mount cases were each explicitly executed and passed. Every positive test line in those three artifact logs is also present in the full job log. All three raw log hashes match their execution receipt; the subject and compiled sections match their standalone receipts.

The exact-H manifest and lockfile were independently fetched through GitHub and preserved as `exact-H-host-Cargo.toml` and `exact-H-host-Cargo.lock`. Their SHA-256 values match the compile receipt:

| Input | SHA-256 |
| --- | --- |
| `source/host/Cargo.toml` | `3fe05f12ecd0aef14035ea61a48b909614188d21b89ab27697bf8b737a84a387` |
| `source/host/Cargo.lock` | `96b7c7e3faa9faf3aa54b800dccb17ae7a0ecebf59ca7e2e1a948b9ae0545eaf` |
| `compiler-artifacts.jsonl` | `eacf949162ddb8a46b57aaef67132b1e8a5fadcb954892f827274060d4e039f1` |

The receipt's executable SHA-256 is `6e65582d7ecb53d17913ece4d3f1e83299162a4568ccaf7e5f2ee5afe6a371ef`. The binary itself is not in the artifact, so that binary hash is receipt evidence rather than an independently rehashed download. Full per-invocation names, commands, summaries, and comparisons are saved in `installation-receipt-accounting.json`.

The receipt scope is ordinary restore plus real private-namespace bind-mounted descendant file/symlink replacements using synthetic installation keys and owned fixtures. It does not claim concurrent external namespace mutation exclusion or full backup/client compatibility. Its `independent_acceptance` remains `pending`.

## Linux completed native media evidence and current limit

[Linux job 125721250552](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576543683/job/125721250552) started at 04:19:00 UTC. At both observed job reads, steps 1–11 had succeeded and step 12, “Test portable recovery, Memory, pairing, palette, Session, MCP and channel controls on the existing Linux Host cache,” was still running. Step 12 began at 04:22:18 UTC. Step 19, “Check and execute the actual Automation, Settings and channel App owners on Linux,” remained pending, as did the later native SQLite evidence upload.

The complete Linux job log was not collected because the job had not completed within this bounded collection. No failure conclusion is attributed to it. Steps 10 and 11 are recorded as successful in the job readback, but their detailed results are not substituted for unavailable final Linux/App evidence.

Completed artifact **13203799823**, `ocrm-media-cleanup-Linux-a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74-41576543683-1`, was uploaded at 04:22:08 UTC and is saved as `linux-artifact-13203799823.zip`. Its 117,699 bytes have SHA-256 `d1a134c702cb70be8e64fb841003d8b8e7161b1427803770dee54b30a20e5116`, matching the GitHub artifact digest. All 17 members are preserved in `linux-artifact-13203799823.members/`.

The native receipt binds exact H/tree/U/run/attempt/job/OS. Its 46 recorded inputs are identical before and after execution, and each normalized input blob identity agrees with its recorded blob. These are receipt consistency observations; only the manifest and lockfile were independently retrieved here from exact H, plus the App LFA source described below.

All 16 native invocations have cargo exit 0, capture exit 0, no execution error, matching raw-log hashes, accepted recorded oracles, exactly one positive line per required name, no unexpected positive names, and summaries with zero failures and zero ignored cases:

| Native invocation | Passing log lines |
| --- | ---: |
| cleanup-owner | 7 |
| cleanup-consumers | 17 |
| html-policy | 12 |
| host-read-format | 1 |
| file-url-admission | 5 |
| explicit-html-consumers | 28 |
| image-probe-consumers | 16 |
| bounded-reader-owner | 11 |
| bounded-reader-public | 17 |
| bounded-reader-box-consumer | 1 |
| bounded-reader-image-consumer | 1 |
| bounded-reader-channel-consumers | 7 |
| local-file-url-owner | 10 |
| file-context-consumers | 10 |
| send-message-consumers | 17 |
| agent-management-consumers | 3 |

These are 163 passing lines across the 16 recorded invocations. Per-case exact names, occurrences, log hashes, exits, and summaries are preserved in `linux-completed-media-artifact-accounting.json`. The receipt's 13 synthetic oracle-qualification controls are explicitly separate from product behavior evidence.

The native artifact's scope explicitly defers actual private Runner/AgentManagement sinks and Session custody to a separate Linux App receipt. Its native LFA successes do not establish the six App outcomes.

## Actual App LFA six-case accounting

The exact-H source [`source/host/app/src/tests/local_media_file_url_tests.rs`](https://api.github.com/repos/bhrumom/fabushi-desktop/contents/source/host/app/src/tests/local_media_file_url_tests.rs?ref=a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74) is preserved as `exact-H-local_media_file_url_tests.rs`, SHA-256 `f2097d754193bd21cc1b7911829f37ba395f59750680e1e560d682e6a8f49d80`, Git blob `5682e951f1eac8f107efd3a3948b0748870912bd`. This establishes the requested input names, not runtime execution.

| Full test name | Observed result |
| --- | --- |
| `tests::local_media_file_url_tests::local_media_file_url_app_rejects_invalid_before_materialization` | Not observed; actual App step pending |
| `tests::local_media_file_url_tests::local_media_file_url_app_rolls_back_earlier_image_on_later_refusal` | Not observed; actual App step pending |
| `tests::local_media_file_url_tests::local_media_file_url_app_commits_native_and_unicode_media_across_reopen` | Not observed; actual App step pending |
| `tests::local_media_file_url_tests::local_media_file_url_app_repeated_refusal_then_repair_survives_reopen` | Not observed; actual App step pending |
| `tests::local_media_file_url_tests::local_media_file_url_app_agent_management_refuses_image_before_outward_send` | Not observed; actual App step pending |
| `tests::local_media_file_url_tests::local_media_file_url_app_preserves_https_without_local_custody` | Not observed; actual App step pending |

None of these six full names occurs in any of the 16 completed native artifact logs. There is no actual App LFA execution receipt, App log, exit code, or case failure in the collected artifacts. Their absence is an evidence gap caused by the pending App step at this read boundary, not a passing result or an inferred testcase failure.

The initial and later artifact lists both contain five artifacts; no later native SQLite/App artifact had appeared. Other artifact families and other heads were not used as replacement evidence.

## macOS and collection limits

[macOS job 125721250429](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576543683/job/125721250429), `rust-host`, was queued at both reads with an empty steps array and no conclusion. Its API `started_at` field alone is not treated as execution evidence.

Only new evidence files in this unique directory were written. Existing source, reports, and evidence bytes were not modified. No local or htch product tests, builds, parsers, linters, generators, or repository validation scripts were executed. Local operations were limited to reading text, JSON/ZIP data handling, exact byte/hash accounting, artifact extraction, and writing these evidence copies. No workflow rerun was requested. Source merging, serial evidence integration, and final conclusions remain with the parent.

## Files and integrity

- `collection-summary.json`: machine-readable bounded observation and six-case accounting.
- `installation-receipt-accounting.json`: complete installation command/name/result/hash accounting.
- `linux-completed-media-artifact-accounting.json`: complete native invocation/name/result/hash accounting.
- `*.zip.accounting.json`: ZIP digest and every extracted member identity.
- `*.receipt.json`: collection provenance and exact-H file source receipts.
- `file-inventory.json`: byte counts and SHA-256 for every preserved file in this directory except the inventory itself.

Collection directory: `/workspace/scratch/7d27ea239237/media-admission-a2-actions/general-evidence-20261103T042507Z`.
