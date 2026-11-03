# Hfc Windows native media evidence addendum

## Result

**The completed Windows media job passed all 19 selected native groups: 205 passed test results, 0 failed, 0 ignored. DB13 and COMMIT27 both passed.**

This result belongs only to Hfc, commit **fc187ecba404e2fa95d65871301aa8d83c7f0314**, tree **410414bcf3ad042cf49aeed86ce55836f4fbc1cd**, on run **41582614116**, attempt **1**. It is not assigned to R5 or to later source edits.

The source/Linux report, assessment, snapshots and bindings remain unchanged. This report and the new windows-prefixed files are additive.

## Exact job and artifact binding

| Identity | Observed value |
| --- | --- |
| Repository | bhrumom/fabushi-desktop |
| Runtime run / attempt | 41582614116 / 1 |
| Tested commit | fc187ecba404e2fa95d65871301aa8d83c7f0314 |
| Tested tree | 410414bcf3ad042cf49aeed86ce55836f4fbc1cd |
| Fixed upstream U | 1440bb1771372334bea23d7642bb52deb7db1845 |
| Fixed upstream tree | 90c8aab39972da11cfad220361781d49e1bc6700 |
| Job ID | 125740667992 |
| Job display name | OCRM media cleanup / Windows |
| Receipt/workflow job key | media-store-windows |
| Runner | windows-2025 |
| Job started | 2026-11-03T05:13:14Z |
| Job completed | 2026-11-03T05:21:29Z |
| Job conclusion | success |
| Artifact ID | 13206860961 |
| Artifact created | 2026-11-03T05:21:01Z |
| Downloaded ZIP bytes | 148,489 |
| ZIP SHA-256 | daeacc2c6573bf636c3a5c1a9ab59a84d84115cc9072a6b5ca01dd4758ba18cc |

[Runtime run 41582614116](https://github.com/bhrumom/fabushi-desktop/actions/runs/41582614116)

The exact attempt-1 jobs response supplies the complete job record, including Hfc, attempt 1 and successful completion. All ten job steps are successful, including exact-HEAD binding, native capture and artifact upload.

The artifact is named ocrm-media-cleanup-Windows-fc187ecba404e2fa95d65871301aa8d83c7f0314-41582614116-1. Its API workflow-run record names Hfc and run 41582614116. The downloaded ZIP hash exactly matches the API digest.

The receipt names Hfc, the exact target and upstream trees, run 41582614116, attempt 1, Windows and job key media-store-windows. The exact Hfc workflow blob 673d6270d3c2a7f4b9e9f640acf88927f2f7b154 maps that job key to this display name, windows-2025, the media capture command and this artifact naming pattern.

## Input and raw-byte integrity

Both receipt input maps contain **62 paths**. Every before and after Git blob and normalized-worktree blob equals its path's entry in the already-preserved, untruncated GitHub exact-Hfc tree response. There are **zero binding mismatches**. The two maps are identical, consistent with the receipt's inputs_unchanged value.

All **19 raw-log SHA-256 values match the receipt**. The archive members were extracted as bytes; no newline conversion, text rewrite or added newline was applied.

The actual line endings are:

- All 19 native logs contain LF, with **zero CRLF and zero bare CR**.
- The raw receipt contains **2,445 CRLF**, with no bare CR.

Windows execution is established by the job and receipt identity. It does not imply that its captured stdout files use CRLF. The differing line endings above are preserved in the saved artifact.

| Key member | Bytes | SHA-256 |
| --- | ---: | --- |
| receipt.json | 121,219 | 0cb5fff9310a3001208309099ee5d09989bcf0b51c87a5323695bb4d2a918a69 |
| media-settlement-owner.log | 30,601 | 59f66a1f5520090a768de665dbc959119ae852346d30fc7144366efd5080a471 |
| transaction-commit-owners.log | 30,839 | 18b59f18e7d6e59e9bc52e8873bd95776dd9a8ca56ef59393033cc86083525e1 |

The receipt separately preserves Windows checkout-byte identities. For example:

| Input | Checkout bytes | Checkout SHA-256 | Git and normalized blob |
| --- | ---: | --- | --- |
| source/host/src/extensions/session/agent_db.rs | 437,038 | 19e73f4aff95d8f5cec8a717bc02682d63799a9eddaef6988643e355b4e4f54e | 87186bc4b246d88a5ce51ebda0623b76f4425d3d |
| source/host/tests/openclaw_sessions_and_config_contract.rs | 169,417 | 6376b945c4fe2ee56ee4ad49534037393c09b77aefb7c10b74a5797d16338804 | 35719fc77edb1fea8f7acf0b22524c6eff7b29aa |

Those are Hfc's executed inputs. They are not substituted with the earlier worker repair, the frozen R5 oracle or a later branch head.

## Actual selected group results

Every group has cargo exit 0, capture exit 0, no execution error, no missing/nonunique required positive, no unexpected positive and an accepted capture oracle. All 19 raw summary lines were read directly.

| Native group | Passed | Failed | Ignored | Raw-log CRLF |
| --- | ---: | ---: | ---: | ---: |
| media-settlement-owner | 13 | 0 | 0 | 0 |
| outgoing-media-custody | 5 | 0 | 0 | 0 |
| transaction-commit-owners | 27 | 0 | 0 | 0 |
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
| **Total selected test results** | **205** | **0** | **0** | **0** |

These are the actual Windows counts, including its platform-specific selection. The earlier Linux total remains 210; it is not used as an expected Windows count.

### DB13 and COMMIT27

The complete named-result portions of both raw logs were read.

DB13 reports all thirteen required settlement cases passing, including actual lost-COMMIT-acknowledgement handling, actual rollback-denial handling, deferred COMMIT failure, ordinary refusal after a real receipt-failure prefix, poisoned/prior transaction retirement, success/replay, exact fault identities and lease lifetime. The raw summary is **13 passed, 0 failed, 0 ignored**, in **0.85 seconds**.

COMMIT27 reports all twenty-seven required names passing. This includes the actual media positive, missing/duplicate/conditional owner and helper negatives, changed COMMIT/rollback handling, alias/macro cases, changed settlement and entry handling, the raw-COMMIT prohibition and all retained channel and ordinary owner checks. The raw summary is **27 passed, 0 failed, 0 ignored**, in **153.05 seconds**.

The receipt's separate synthetic log-oracle qualification also accepts its positive and CRLF-positive controls and rejects all eleven negative controls. Its scope remains synthetic capture qualification, not additional product behavior evidence.

## App and macOS state at completion

After finishing the Windows artifact assessment, one completion observation was made at **2026-11-03 05:23:43 UTC**. No repeated polling or waiting followed.

| Remaining subject | Observed state |
| --- | --- |
| Linux sqlite-native-path / 125740667771 | In progress in portable recovery/Memory/pairing/palette/Session/MCP/channel controls |
| Actual Automation, Settings and channel App step | Pending |
| Actual App feature proof / MSU8 / LFA6 / SGF3 / cron5 | No newly completed receipt or artifact available |
| macOS rust-host / 125740668001 | Queued |

No separate App addendum is created because no newly completed App evidence was available. The native Windows result does not establish App execution, Session first-open recovery, generated Runner behavior, packaged compatibility or final migration acceptance.

## Additive files and preserved prior assessment

New evidence is under /workspace/scratch/7d27ea239237/media-fc-ci/:

- windows-runtime-jobs-api.json and windows-artifacts-api.json: initial metadata at the completed Windows boundary.
- windows-job-record.json: exact job selected from the attempt-1 API response.
- windows-completion-jobs-api.json and windows-completion-artifacts-api.json: the single completion observation.
- windows-media-13206860961.zip and windows-media-13206860961/: raw archive, receipt and all nineteen logs.
- windows-media-13206860961-manifest.json: each member's bytes, SHA-256 and LF/CRLF counts.
- windows-media-binding.json: all commands/results, nineteen raw-log bindings, both sets of sixty-two input bindings and qualification data.
- windows-assessment.json: compact machine-readable addendum.

The unsupported generic single-job endpoint attempt is preserved separately as windows-job-direct-fetch-error.json. The supported exact-attempt jobs endpoint supplied the full required job record; this retrieval detail is not a failed CI result.

The prior files still have their recorded hashes:

| Preserved file | SHA-256 |
| --- | --- |
| media-fc-ci-report.md | 73ea2bd88e5911715faa3ffec33f3ae7d75219f238ca457c2e755182695ce0bf |
| media-fc-ci/assessment.json | 2494f16498240c150de0a5df9446b554408da70a9eea969e0092ff9ae1090dc7 |
| media-fc-ci/linux-media-binding.json | 890f37db1a6876462ad83f7ecb4d48481db010bd481711ee4937f699cff1ba95 |
| media-fc-ci/inputs/source-receipt-input-index.json | 96fca0e925fc0d9910fc832e8b72cb1eceb57be346e2ab0cb7f4d4587e355a85 |

Selected effort remained inherited xhigh for assessment, with the configured parent model. Work was limited to read-only API/source/log inspection, JSON/ZIP decoding, hashing, input binding and additive evidence writes. No source/spec/ledger editing, workflow mutation, local or htch validation occurred. Temporary signed artifact-download URLs were not persisted or exposed.
