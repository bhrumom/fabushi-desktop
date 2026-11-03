# Hfc Actions evidence collection and assessment

## Result and cutoff

**Hfc source authority and Linux native media evidence are positive. Windows native media, actual App evidence, and macOS Host evidence remain pending at the collection cutoff.**

This is a bounded assessment of commit **fc187ecba404e2fa95d65871301aa8d83c7f0314** only. The last API observation was **2026-11-03 05:19:39 UTC**. Three observations were made while collecting and binding completed source evidence; the third exposed the completed Linux media artifact. That artifact was then downloaded and assessed. No waiting or polling loop was started for the queued macOS job.

| Required subject | Observed state at cutoff |
| --- | --- |
| Source pre-expansion history gate | PASS |
| Source post-expansion integrity gate | PASS |
| Linux native media, all 19 groups | PASS: 210 passed test results, 0 failed, 0 ignored |
| Linux DB settlement group | PASS: 13 required cases |
| Linux COMMIT contracts | PASS: 27 required cases |
| Windows native media, DB13/COMMIT27/19 groups | In progress; artifact not yet published |
| Actual App shipping/test-support feature proof | Pending; owning App step not reached |
| Actual App MSU8 / LFA6 / SGF3 / cron5 | Pending; owning App step not reached |
| macOS rust-host | Queued |

No result is transferred to the distinct frozen R4 source or to the root's later stricter mutation assertions.

## Exact authority and scope

Repository: bhrumom/fabushi-desktop.

Branch attached to the two runs: codex/openclaw-rust-fusion-chat-20261101-writer.

| Identity | Exact value |
| --- | --- |
| Tested Hfc | fc187ecba404e2fa95d65871301aa8d83c7f0314 |
| Hfc tree | 410414bcf3ad042cf49aeed86ce55836f4fbc1cd |
| Fixed upstream repository | openclaw/openclaw |
| Fixed upstream U | 1440bb1771372334bea23d7642bb52deb7db1845 |
| Fixed upstream tree | 90c8aab39972da11cfad220361781d49e1bc6700 |
| Source run / attempt | 41582614099 / 1 |
| Runtime run / attempt | 41582614116 / 1 |

The GitHub exact-commit response confirms the Hfc tree. The exact recursive tree response is untruncated and contains 10,952 target-repository tree entries. Both run responses name Hfc. Both collected receipts independently name Hfc, its tree and the fixed upstream commit/tree.

The current root AGENTS.md was read before the collection. Selected effort is inherited xhigh for source/runtime evidence assessment; the configured parent model is retained.

Allowed work performed: read-only GitHub API GETs, source and log reads, JSON and ZIP decoding, byte hashing, input-identity comparisons and evidence writes. No local or htch builds, tests, source parsers, validators, generators, captures or test enumeration ran. No source/spec/ledger/test file was edited; no dispatch, rerun or cancellation was requested. Prior diagnosis output is preserved.

All new files are confined to /workspace/scratch/7d27ea239237/media-fc-ci/ and this report.

## Source authority: executed positive evidence

[Source run 41582614099](https://github.com/bhrumom/fabushi-desktop/actions/runs/41582614099) is completed/success, attempt 1.

| Job | ID | Actual result |
| --- | --- | --- |
| source-authority | 125740666138 | Success, completed 05:15:18 UTC |
| recover-original-host-log | 125740668183 | Skipped |
| persist-verified-inventory | 125741350050 | Success, completed 05:15:47 UTC |

The complete decoded source-authority and persistence job logs are saved under media-fc-ci/logs. Their exact connector-returned strings were stored as JSON, then encoded to UTF-8 bytes without newline conversion or an added newline.

| Saved log | SHA-256 |
| --- | --- |
| source-authority-job-125740666138.log | 278f277797722bf9f631ab2d89bbf65f7b86ea80abfac1ee9417a3b963be809a |
| persist-inventory-job-125741350050.log | 38ac9c41a258584e674bd7f99ccf3384b5632916085b5c2cfc7d7a503d9c43f4 |

The source-authority log records the exact Hfc checkout and fixed upstream checkout/tree comparison. Its executed gate sequence is:

1. Inventory-materialization tests: **21 tests, OK**.
2. Pre-expansion history integrity at 05:14:29 UTC: **PASS, 62,130 paths, 884 mapped, 62,067 unread/partial, 5 pending forward links**.
3. Canonical inventory expansion and shard reconstruction: **62,130 rows, 61 shards**.
4. G02 negative probes reject duplicate source paths, unread-denominator tampering, an unbacked accepted verdict, and an orphan responsibility.
5. G01 tracked source and G02 structural checks: **PASS**.
6. Post-expansion source integrity at 05:15:15 UTC: **PASS, 62,130 paths, 888 mapped, 62,067 unread/partial, 0 pending forward links**.

The source artifact is ID **13206770678**, named ocrm-p0-authority-fc187ecba404e2fa95d65871301aa8d83c7f0314. Its downloaded ZIP is 4,951,969 bytes and hashes to **8b89f85b1823b9e184bdccb865b15f417bfb8afddcb2d4a08773bf350d7ab393**, exactly the GitHub artifact digest. All eight members are preserved with individual byte counts and hashes.

The canonical inventory artifact, authority summary and structural receipt all agree on SHA-256 **d0049087bdbcc8eecdc8d0a69bd48bfc7d70688f8917122198a6e15c80863d8a**. The inventory contains 62,128 ordinary blobs, 2 symlinks and no gitlinks. All three dependency-root TSV blob identities match their entries in that fixed-U inventory.

The source receipt explicitly retains:

- G01 dependency/LFS closure: open.
- G02 final: not-run.
- Independent review: pending.
- All eight completion flags: false.

The successful source run therefore establishes its recorded structural gates. It does not establish full source closure, semantic equivalence or migration acceptance.

### Source receipt input binding and generated writes

The source receipt contains 26 repository paths plus the generated upstream-recursive-inventory.json artifact. Every repository path was bound to the GitHub exact Hfc tree. Exact bytes were acquired either from GitHub's requested immutable blob or from an existing local snapshot only after its computed Git blob matched that exact tree entry.

**Twenty-three repository-file SHA-256 values match the receipt. Three do not, because the receipt records the later generated state:**

| Generated path | Exact Hfc Git blob | Exact Hfc SHA-256 | Receipt SHA-256 |
| --- | --- | --- | --- |
| projects/openclaw-rust/inventory/index.json | 6905d89a058a59f09a2ec3f5b3b4afc6f751919d | 88bca2d7eab937e28a9197f8576098cc6cffbaae43d9f7f1b050c2060df514e4 | b1605cfab7367390177f63c25d6264b575ace04813bc4a3999f876ec1e5f994a |
| projects/openclaw-rust/inventory/shards/manifest.json | 136989f86fa26513cb73e3c35795cf7b9c5de4e6 | 74e4bbfdbf84435078e674234f6c82a784803f2a285ba547f641a13c295c1ee9 | 9b4dc21b09d80a94259d0688b552c920f41fb93363589235ba4d14a55470b6d3 |
| projects/openclaw-rust/parity-ledger.json | 99fd980c1f8bee47327efc49e59e0d5544dcf46f | e3615f0841279b7056425c0deda42cc2909f21e63447d619e9d60359ae8d01c0 | ff2a0d0dfbd343941a29933d7154dc0bd3e4b5934b2e1c9fdafa8f6b2a886a75 |

This distinction is expected from the executed sequence, but it must remain explicit. The exact Hfc shard_inventory.py writes the manifest, index and ledger before validate_p0.py hashes those paths. The source log directly records the generating command before the receipt and post-expansion gate. No generator was rerun locally and no generated hash is relabeled as an unchanged Hfc blob.

The separately generated inventory artifact matches its receipt hash. Detailed bindings are saved in media-fc-ci/inputs/source-receipt-input-index.json.

The persistence job log records a successful inventory publication from Hfc to carrier commit 14faff5. That observed publication does not change the subjects of either run assessed here.

## Linux native media: completed and bound

Runtime run [41582614116](https://github.com/bhrumom/fabushi-desktop/actions/runs/41582614116), attempt 1, job **sqlite-native-path / 125740667771**, published the completed media capture before continuing with later native and App work.

Artifact **13206394654** is named ocrm-media-cleanup-Linux-fc187ecba404e2fa95d65871301aa8d83c7f0314-41582614116-1. Its ZIP is 141,363 bytes, SHA-256 **7958e1fe247b9b889954728112625cd6bda205444f377a5b92a889107951406e**, matching the GitHub digest. The receipt and all 19 raw logs were extracted as bytes without newline conversion.

The receipt result is passed. Its subject commit/tree, U/tree, run ID, attempt and job match the API records. Both before and after input maps contain 62 paths; every Git blob and normalized-worktree blob matches the corresponding exact Hfc tree entry, and the maps are identical. All 19 raw-log hashes match the receipt.

Every group has cargo exit 0, capture exit 0, no execution error, no missing/nonunique required positive, no unexpected positive and an accepted capture oracle:

| Native group | Passed | Failed | Ignored |
| --- | ---: | ---: | ---: |
| media-settlement-owner | 13 | 0 | 0 |
| outgoing-media-custody | 5 | 0 | 0 |
| transaction-commit-owners | 27 | 0 | 0 |
| cleanup-owner | 7 | 0 | 0 |
| cleanup-consumers | 17 | 0 | 0 |
| html-policy | 12 | 0 | 0 |
| host-read-format | 1 | 0 | 0 |
| file-url-admission | 5 | 0 | 0 |
| explicit-html-consumers | 28 | 0 | 0 |
| image-probe-consumers | 16 | 0 | 0 |
| bounded-reader-owner | 11 | 0 | 0 |
| bounded-reader-public | 17 | 0 | 0 |
| bounded-reader-box-consumer | 1 | 0 | 0 |
| bounded-reader-image-consumer | 1 | 0 | 0 |
| bounded-reader-channel-consumers | 7 | 0 | 0 |
| local-file-url-owner | 10 | 0 | 0 |
| file-context-consumers | 10 | 0 | 0 |
| send-message-consumers | 19 | 0 | 0 |
| agent-management-consumers | 3 | 0 | 0 |
| **Total selected test results** | **210** | **0** | **0** |

Raw DB and COMMIT test lines were read directly in addition to the receipt and all 19 raw summary lines. The DB group includes real lost-COMMIT-acknowledgement, receipt failure with actual rollback denial, deferred COMMIT failure, known refusal, retirement and exact fault-lease controls. All 13 named cases pass. All original 27 selected COMMIT cases pass; their log reports 146.07 seconds.

| Key raw log | SHA-256 |
| --- | --- |
| media-settlement-owner.log | 84fd7774a9434c2039cc87ebf046b4c196141907d13519d4b1d0eb208e09393e |
| transaction-commit-owners.log | cb1c95773d2a99f893473b160fe8680779ac021a2bbfb7367ed4b4cd7ea54112 |

The synthetic capture-oracle qualification reports both positive and CRLF-positive controls accepted and all 11 negative controls rejected. Its own receipt explicitly describes that qualification as synthetic, not product behavior evidence.

The native receipt also expressly excludes the actual App, Session first-open recovery and generated Runner consumer. Those require the separate actual App receipt.

### Exact native source identities

| Relevant Hfc input | Git blob |
| --- | --- |
| source/host/src/extensions/session/agent_db.rs | 87186bc4b246d88a5ce51ebda0623b76f4425d3d |
| source/host/tests/openclaw_sessions_and_config_contract.rs | 35719fc77edb1fea8f7acf0b22524c6eff7b29aa |
| .github/workflows/rust-desktop-runtime.yml | 673d6270d3c2a7f4b9e9f640acf88927f2f7b154 |
| projects/openclaw-rust/scripts/capture_media_cleanup.py | d5376d94f569b112c5778efb529eb64d2b863766 |
| projects/openclaw-rust/scripts/capture_portable_contracts.py | 172485b8437151256fb5058fcec010f9a0665cc8 |
| source/host/app/src/main.rs | ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df |
| source/host/app/src/tests/media_settlement_unknown_tests.rs | f525710de5833559de608e1cb8aaa89a8d58e2b8 |

The Hfc oracle blob differs from this worker's earlier repaired blob 6ec5edbd49fe7fb590b0c52c0f69e38d25a501bf. The positive runtime claim above follows Hfc's executed artifact and input map, not the earlier repair or its review.

Full commands, per-log hashes, both sets of input bindings and qualification records are saved in media-fc-ci/linux-media-binding.json.

## Remaining runtime state

At 05:19:39 UTC, the runtime aggregate remains queued with no conclusion. The job-level facts are more specific:

| Job | ID | Observed state |
| --- | --- | --- |
| sqlite-native-path, ubuntu-24.04 | 125740667771 | In progress; completed media capture; executing portable recovery/Memory/pairing/palette/Session/MCP/channel controls |
| OCRM media cleanup / Windows, windows-2025 | 125740667992 | In progress in media cleanup execution; no Windows media artifact yet |
| rust-host, macos-15 | 125740668001 | Queued, no steps |
| renderer | 125740668051 | Completed/success; several unrelated build/typecheck steps explicitly skipped |
| OCRM installation key restore boundary / Linux | 125740668063 | Completed/success |
| Pressure profiler | 125740668973 | Skipped |
| platform-worker | 125740669639 | Skipped |

The Linux step that checks and executes actual Automation, Settings and channel App owners was pending. Accordingly, shipping feature-off, test-support feature-on Host library identities, MSU8, LFA6, SGF3 and cron5 have no positive evidence in this collection. They are pending, not failed and not skipped. No App receipt was published in the observed artifact inventory.

Windows raw bytes cannot be assessed until its artifact exists. No CRLF log inference is made from Linux or synthetic controls. The queued macOS job is not a platform result.

## Preserved evidence and next acquisition boundary

Evidence root: /workspace/scratch/7d27ea239237/media-fc-ci/

- api/: exact commit/tree plus complete run, attempt-specific job and artifact responses for the three observations.
- logs/: complete decoded source-authority and persistence job logs and their lossless JSON representations.
- artifacts/source-authority-13206770678.zip and extracted eight-member source artifact.
- artifacts/linux-media-13206394654.zip and extracted receipt plus all 19 raw native logs.
- Artifact manifests: per-member bytes, SHA-256 and, for native logs, CRLF counts.
- inputs/: exact source receipt inputs and their Git/SHA-256 binding table, source dependency-root bindings and key Hfc input identities.
- linux-media-binding.json: complete Linux native identity/hash assessment.
- assessment.json: compact machine-readable outcome and pending-state record.

Temporary signed download URLs were neither written to the report nor persisted as artifact-download inputs.

The next useful collection is triggered by a Windows media artifact or a completed actual App step/receipt on the same run and attempt. It should preserve those new raw artifacts and bind their input maps to Hfc before reporting results. This collector stops here under the bounded-observation instruction; it does not promote pending work or launch another execution.
