# A2 renderer failure evidence supplement

## Outcome

Renderer job **125721250334** failed in the Plugin Skills contract test step. Its complete log reports **121 tests, 117 passed, 4 failed, zero cancelled, zero skipped, zero todo**, followed by exit 1. The four failures are loader assertions about cache publication and filesystem error classification. This is an actual Actions failure on A2; no local execution or reproduced failure is claimed.

The subsequent native attachment binding test step was **skipped**. The always-record evidence step then correctly emitted a **failed-or-incomplete** MAB-v1 receipt and exited 1 because it had no native attachment test result. That secondary evidence failure must not be described as a failing native attachment behavioral test.

This supplement leaves the earlier source/Windows evidence and its timestamped inventory unchanged. It does not refresh queued jobs or assert a complete runtime workflow result.

## Exact subject and collection identity

| Identity | Value |
| --- | --- |
| Repository | `bhrumom/fabushi-desktop` |
| Workflow | `.github/workflows/rust-desktop-runtime.yml`, workflow ID `362455251` |
| Runtime run / attempt | `41576543683` / `1` |
| Job | `renderer`, ID `125721250334` |
| Exact H | `a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74` |
| Exact H tree | `a0dceefeae75fccd7ab5aec143f6d651f7cf2fd3` |
| Fixed upstream U | `1440bb1771372334bea23d7642bb52deb7db1845` |
| Upstream tree | `90c8aab39972da11cfad220361781d49e1bc6700` |
| Log fetched | `2026-11-03T04:24:41.552Z` |
| Step receipt fetched | `2026-11-03T04:24:49.913Z` |
| Runner from raw log | Ubuntu 24.04.5, image `ubuntu-24.04` |

[GitHub job](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576543683/job/125721250334).

The log independently shows exact checkout and HEAD binding: L107/L118 fetch and checkout H; L139 prints H; L146 binds the same expected H. The raw MAB receipt separately records H, its tree, U, run, attempt, and job. The newer remote commit reported by the root is not substituted for this tested subject.

## Primary failure, with exact log locations

All locations below are line numbers in the saved complete `renderer-job-125721250334.log`. Generated JavaScript positions are copied from the runner stack; they are not represented as original TypeScript line numbers.

| Test | Observed mismatch | Raw log | Generated test position |
| --- | --- | --- | --- |
| `failed cache publication removes only its staging directory` | Actual array `['new-version']`; expected `['calendar-slug']` | L947–974 | `desktop/.tmp-grok-electron/ocrm-plugin/loader.contract.test.cjs:16047:30`; assertion `:16073:25` |
| `filesystem EACCES during cache installation is an install failure, not clone authentication` | Actual `'unknown'`; expected `'install'` | L975–998 | Same generated file `:16076:32`; assertion `:16097:27` |
| `filesystem EPERM during cache installation is an install failure, not clone authentication` | Actual `'unknown'`; expected `'install'` | L999–1022 | Same generated file `:16076:32`; assertion `:16097:27` |
| `filesystem EROFS during cache installation is an install failure, not clone authentication` | Actual `'unknown'`; expected `'install'` | L1023–1046 | Same generated file `:16076:32`; assertion `:16097:27` |

The node test command is shown at L740 onward. The final TAP accounting and failure exit appear at L1563–1572. The next loader case, `an older failed install cannot remove a successor published cache directory`, passes at L1047–1048; the log does not support treating the entire loader suite as failed.

**Required root follow-up:** investigate these four exact Plugin Skills loader contract mismatches in the tested A2 implementation/fixtures. The evidence alone does not establish whether the cause is production logic, fault-injection setup, or an obsolete assertion. No source fix, relaxed oracle, or workflow rerun is proposed or performed by this collection.

## Step outcomes and downstream consequence

The purpose-built GitHub job-step receipt records:

| Step | Outcome |
| --- | --- |
| Exact HEAD binding | success |
| Pinned notice inputs and packaging contract | success |
| Desktop dependency install | success |
| Build Grok-shaped Electron runtime | success |
| Test Plugin Skills, model settings bridge and routed MCP carrier | **failure** |
| Record / upload Plugin Skills edge evidence | skipped / skipped |
| Test native attachment account and MainData binding | **skipped** |
| Record exact-subject native attachment binding evidence | **failure** |
| Upload native attachment binding evidence | success |
| Typecheck and build renderer | skipped |
| Later architecture, bundle and desktop runtime gates | skipped |

The raw log at L1775–1779 has `MAB_BUILD_OUTCOME: success`, `MAB_TEST_OUTCOME: skipped`, and `MAB-v1 evidence is failed or incomplete`, then exit 1. The uploaded receipt agrees:

- `contract: MAB-v1`, `result: failed-or-incomplete`.
- `build_outcome: success`, `test_step_outcome: skipped`.
- Node and capture exit fields are null; TAP counters and TAP hash are null.
- Observed positive names are empty; required names are all reported missing.
- Capture-control observed exits are null and their passed flags are false.
- `input_drift` is empty, but that cannot substitute for execution.
- `independent_acceptance: pending`.

The failed receipt was uploaded as artifact **13204725638**, named `ocrm-mab-a2a0c9ef326c1ddeb09b4f9180267fdaeb21db74`. It contains only `receipt.json`, which was preserved verbatim. The local archive SHA-256 matches the artifact-upload SHA-256 printed at raw log L1803.

## Raw files and integrity

All supplement files are under `/workspace/scratch/7d27ea239237/media-admission-a2-actions/`.

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `renderer-job-125721250334.log` | 140527 | `dc67d69790921fac5ac797a448be41908640de366de1ade9f3c275ca8c0acbb7` |
| `renderer-mab-13204725638.zip` | 4878 | `cde93b52fea6d5de3851cc55b34faf1166938f4dad73ad7ffa021bc7b91f7451` |
| `renderer-mab-13204725638-receipt.json` | 20875 | `183a2af40ffe5da5d347eb03037ea7ef83efdb6478d122da7742166088c5213e` |

The log is the complete connector-decoded text encoded as UTF-8, preserving its BOM and line breaks. Its hash identifies that saved representation; it is not claimed to be an independently downloaded raw HTTP byte stream. The ZIP is the raw artifact download; receipt bytes were extracted unchanged.

Supporting raw records:

- `renderer-job-125721250334-log-receipt.json`: complete log string and retrieval observation.
- `renderer-job-125721250334-step-receipt.json`: job-step response.
- `renderer-job-125721250334-log-hash.json`: saved log byte/hash record.
- `renderer-mab-13204725638-download-receipt.json`: artifact/file identity.
- `renderer-mab-13204725638-hashes.json`: archive and member hashes.
- `renderer-supplement-sha256.json`: separate manifest for these new supplement files.

A generic GitHub fetch of the individual jobs endpoint was rejected by the connector as an unsupported endpoint. Its raw error is retained in `renderer-job-125721250334-metadata-fetch-error.json`. The purpose-built job-step/log tools succeeded. This connector limitation is not a CI failure.

## Evidence scope

These are renderer Actions observations for frozen A2. They neither invalidate the separately retained successful Windows native-media job receipt nor upgrade the source-authority run into runtime proof. They do not execute or approve the unpublished MSU App snapshot audited separately. No result for Linux/macOS pending jobs is inferred here.

Collection used GitHub connector reads and one raw artifact download. Local activity was limited to text/JSON/ZIP reading, writing evidence, and hashing bytes. No local or htch executable validation, parser/build/test/test-listing/capture execution, source/capture/ledger/workflow edit, commit/push, Actions dispatch/rerun, or repeated queued-job polling occurred.

