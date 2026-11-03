# R6 actual App evidence collection — pending at bounded cutoff

## Result

**PENDING: the Linux job was still running when the fourth and final permitted metadata observation completed at 2026-11-03 06:25:20 UTC.** Actual App step 19 had completed successfully, but link-preview step 21 was still in progress and the final native SQLite artifact upload was pending.

No complete job log or actual App receipt was acquired in this phase. No log request was attempted while the job remained nonterminal. Consequently this report does not accept background48, the two compiled Host feature modes, the 115 actual App names, MSU8 / LFA6 / SGF3 / cron5, or link-preview App3 on the basis of step status alone.

This is an evidence-collection cutoff, not an implementation failure, a new campaign, or a finding that the capability is unrecoverable.

## Exact subject and configuration

| Identity | Value |
| --- | --- |
| Repository | bhrumom/fabushi-desktop |
| H862 | 8621173a53d1f46f7dc613c1b2e162c3350b94bc |
| H862 tree | 077065fdeeb9aa17213a4f6d2dece8cba168fbf5 |
| Fixed upstream U | 1440bb1771372334bea23d7642bb52deb7db1845 |
| Fixed upstream tree | 90c8aab39972da11cfad220361781d49e1bc6700 |
| Existing runtime run / attempt | 41586608776 / 1 |
| Linux job / ID | sqlite-native-path / 125753467838 |
| macOS job / ID | rust-host / 125753467847 |
| Selected effort | xhigh |
| Model setting | Configured inherited parent model; no override applied |

The effort choice follows the task's need to distinguish actual App execution and compiled feature observations from metadata-only success. The collector did not invent a model identifier that the tools do not expose.

[Runtime run 41586608776](https://github.com/bhrumom/fabushi-desktop/actions/runs/41586608776) remains the sole execution subject. Each collector observation used the exact-attempt jobs endpoint and confirmed H862 and attempt 1 on the Linux job. The exact-H commit/tree authority from the preceding collection was reused; no mutable branch head was substituted.

## Observed job progress

| Selected step | API state at 06:25:20 UTC | Start / completion UTC |
| --- | --- | --- |
| 12 — portable recovery, Memory, pairing, palette, Session, MCP and channel controls | completed / success | 06:01:33 / 06:12:06 |
| 19 — actual Automation, Settings and channel App owners | completed / success | 06:13:31 / 06:18:51 |
| 20 — historical prior-root recovery | completed / success | 06:18:51 / 06:22:26 |
| 21 — native link-preview HTTP, Coordinator and App dispatch | in_progress / no conclusion | 06:22:26 / not completed |
| 22 — preserve failed owned fixtures | pending | not started |
| 23 — upload native SQLite path evidence | pending | not started |

The Linux job is in_progress with no conclusion or completion timestamp. The macOS rust-host job remains queued in the final snapshot.

These API facts establish the recorded step lifecycle. They do not identify every executed test, provide the successful compiler JSON feature observations, or expose the final App executable identity.

## Four observations, then stop

| Collector observation | UTC | State change relevant to collection |
| --- | --- | --- |
| 1 | 06:21:13 | App step 19 success; prior-root step 20 in progress; link-preview pending |
| 2 | 06:22:45 | Prior-root step 20 success; link-preview step 21 in progress |
| 3 | 06:24:18 | Link-preview still in progress |
| 4 | 06:25:20 | Link-preview still in progress; upload pending |

Intervals are **92, 93 and 62 seconds**, all at least 60 seconds. The parent's previously supplied 06:13:59 observation is separately attributed as starting context; it is not represented as a collector-owned API response. The collector made exactly four job metadata observations in this phase and has no remaining observation budget.

No fifth observation, job-log request, artifact polling loop or execution request follows this cutoff.

## Prepared direct evidence requirements

The exact workflow and portable capture were read through immutable Git blob APIs. The repaired background fixture was also read and bound to H862. Source declarations are used only to establish the required comparison inventory, never as execution positives.

| Required subject | Prepared expectation | Direct execution evidence acquired here |
| --- | --- | --- |
| Repaired background target | Original 48 declared test names, including the existing shipping SendMessage test | None; portable step success only |
| Shipping App compile | Exact cargo check command; actual Host library observation without test-support | None; App step success only |
| App test harness compile | Exact cargo test --no-run command; actual Host library observation with test-support and actual App executable identity | None |
| Actual App named cases | Complete 115-name required sequence from exact workflow | None |
| MSU | 8 exact names within that sequence | None |
| LFA | 6 exact names within that sequence | None |
| SGF | 3 exact names within that sequence | None |
| Cron identity | 5 exact names within that sequence | None |
| Link-preview App | 3 exact names in its declared App command | None; owning step still running |

The repaired background positive that must appear in the complete portable integration output is:

`shipping_send_message_reaches_host_owned_delivery_failure_broadcast_and_context_paths`

Its target remains `background_wakes_production_contract`; the expected source inventory contains exactly 48 distinct test declarations. The preserved inventory is not a claim that those 48 executed successfully.

The actual App inventory contains 115 distinct full names, including the 22 focused MSU/LFA/SGF/cron cases. It also preserves the other required Automation, Settings, channel, MCP, device and model/cancellation cases. No existing required name is removed to make the collection easier.

Link-preview App requires these full names:

- `tests::link_preview_gateway_tests::link_preview_gateway_cached_native_call_bypasses_closed_host_lane`
- `tests::link_preview_gateway_tests::link_preview_gateway_native_errors_and_private_cache_preserve_contract`
- `tests::link_preview_gateway_tests::link_preview_gateway_other_methods_preserve_host_lane_request_and_reply`

## Compiled feature proof must be explicit

The exact workflow's successful path runs:

```text
cargo check --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host --message-format=json
cargo test --locked --manifest-path source/host/app/Cargo.toml --bin mahayana-app-host --no-run --message-format=json
```

Its declared feature observer requires one compiler-artifact event for the actual `mahayana_host_runtime` library, the exact Host Cargo manifest and src/lib.rs paths, and a non-test dependency profile. It requires test-support absent in the shipping observation and present in the App-test observation. The App executable selector separately requires the actual shipping binary target and a test profile, then records the owned executable path and SHA-256.

Those are the expected gates read from source. **No observed feature array, compiler event, executable hash or command exit from this App capture has been obtained in this phase.**

The successful workflow suppresses full Cargo JSON stdout in the job console and records its digest plus the extracted feature observations in the final receipt. A later log assessment must distinguish the explicit recorded feature observations from independently rereading the full original compiler JSON artifact. It must not invent that missing artifact verification or use an empty missing-results list as a feature proof.

## Fifteen causal source inputs bound

Fifteen unique source paths were fetched from exact-H immutable blobs and hashed as UTF-8 bytes; the source preparation records their byte counts, blob identities and SHA-256 values. It is a deliberately stated causal subset, not an assertion that every portable or actual App input was independently acquired.

Key current identities:

| Source | Git blob | Canonical SHA-256 |
| --- | --- | --- |
| Runtime workflow | 25e80da72d20d4dbfed8ab6f2a50a3eff62b53af | 68f0f49cc3d6d5f1c96156a4a96e862555accdcc6fe015a36ed5195f2e524803 |
| Portable capture | f5f3d5624568cb553441b8ca0c0f63aff76ebeae | e101cf22f5dbda30f2a024010bff64e13d0ed4bee7b6dbac89afee4c629d8b2e |
| Repaired background fixture | 1c38f0bfde3e23cb0d227d1c4dfb4bd5b4d0b713 | 281cfdaa4b4c28a89dab0474340775c43d17b0b3b01808c9842c26f4a1460529 |
| Repaired App MSU fixture | fc1f056a318fee390e0c8e67cdda361d94f0c3a5 | 92f462fc7c7763b909daabd805bc0f33161f156e842234ae2ea83ea8070fd0e3 |
| Actual App main | ab3cd46cfb869db19d12f2e1ee77a7c18e7b52df | 0ca4e2e81b459ac57c1a9aab9b31b54ef2d421d7131970519aa4f58faaa0011f |
| Link-preview App fixture | 553f0675015cee1d3a7beebd054d69f787536882 | 21e566af4a7c8b8550eaffbc8f0f24d85e82f27029755d03c9fa35d8e2e9a16a |

The remaining prepared paths are the fixed upstream lock, Host/App Cargo manifests and locks, Host src/lib.rs, and the LFA, channel-host-lane and cron App fixtures. The full list is in source-input-preparation.json.

No claim is made that all 377 portable inputs were rebound, that the complete App input set was acquired, or that any runtime receipt hash matched these files; no such receipt has yet been read.

## Evidence preservation and permitted actions

Only immutable evidence blobs under the authorized directory are created:

`projects/openclaw-rust/evidence/media-settlement-unknown-647e0932-20261103/execution-r6/actual-app/`

The collection preserves:

- All four complete exact-attempt jobs API JSON text responses, with explicit timestamps.
- The expected full-name/command inventory, labeled as expectation.
- The 15-path source-input preparation and its limits.
- A pending phase summary and this report.
- A handoff manifest identifying created blob objects and complete readback verification.

Saved API responses are connector-returned JSON text, not raw HTTP response bytes. The derived source/name records are not reconstructed receipts or logs.

**Raw job logs obtained: 0. Log chunks created: 0. Actual App receipts obtained: 0.** A lossless raw-log manifest is not fabricated for absent text.

The executor remained unavailable and was not called. No local or htch project validation, source parser executable, build, test, generator, capture or test enumeration ran. Cloud operations were read-only source/API acquisition, text/JSON/hash accounting and explicitly authorized evidence-blob creation. No tree, commit, branch, production source, workflow, gate or ledger was changed, and no dispatch, rerun or cancellation occurred.

## Handoff

The earlier exact-H Linux and Windows native positives remain valid within their recorded scope. This collection neither repeats those executions nor inflates them into App proof.

The next direct instrument is the complete decoded log of **this same job**, once a terminal state has been separately established. Preserve the exact connector-returned text as UTF-8 chunks no larger than 1 MiB, verify complete readbacks and lossless concatenation, then compare explicit subject/U/workflow/commands, actual feature observations and every required named positive. Retain the distinction between original artifact bytes and console-decoded text.

For this bounded phase, the terminal decision is **PENDING evidence acquisition, no new failure classification and no new execution campaign**. A step success or exhausted observation budget cannot substitute for the missing direct evidence.
