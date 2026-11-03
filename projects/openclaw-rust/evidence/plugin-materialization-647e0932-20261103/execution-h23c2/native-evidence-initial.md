# H23 native evidence readback — bounded snapshot

## Subject and collection boundary

- Repository: bhrumom/fabushi-desktop.
- Canonical branch: codex/openclaw-rust-fusion-chat-20261101-writer.
- Tested subject H23: 23c2a567d874509f56353b8cae737cdf1437309d.
- Tested tree: b10485b13c15cd1cf012a5f51acaf6927aeb322a.
- Fixed upstream U: 1440bb1771372334bea23d7642bb52deb7db1845; upstream tree in restore receipt: 90c8aab39972da11cfad220361781d49e1bc6700.
- Runtime run: https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105 ; run 41576693105, attempt 1.
- API run identity, head_commit.tree_id and Git commit API tree agree with H23.
- First acquisition occurred before the recorded clock reading 2026-11-03 04:23:37 UTC. The focused job follow-up completed before 2026-11-03 04:24:40 UTC. Preservation finished after 04:25:23 UTC.
- The API run summary reported status queued / conclusion null, while its exact attempt job list showed some jobs completed. This snapshot does not turn the run summary into success.
- Canonical ref at acquisition was carrier 972779ecd290b90e407ffe651aa5c456c31e9f7f, tree a75bb4706831a34b818e459f6be4df920560a2fe. Its API parent is bb5a7f6098f952df6d388a672c4fb01045bc645c. Neither later commit is the tested subject of these receipts.
- No local or htch compilation, testing, linting, parser/generator/validator/harness execution, production/workflow/ledger edit or push occurred. Only API/source/log reads, archive download/extraction, byte identity comparisons and this evidence-directory write occurred.
- Host model and effort were inherited unchanged; no prompt text is represented as a model/effort parameter override.

## Job observations

| Job | Native platform | Observed status | Exact evidence available |
| --- | --- | --- | --- |
| [125721731498 — sqlite-native-path](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/job/125721731498) | ubuntu-24.04 | in_progress, conclusion null | Steps 8–11 success; step 12 portable recovery/Memory/pairing/palette/Session/MCP/channel in_progress. Later palette, Automation, App, prior-root and native-dispatch/upload steps pending. No terminal full-job log/receipt collected. |
| [125721731668 — rust-host](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/job/125721731668) | macos-15 | queued, conclusion null | Exact attempt API: steps empty, runner_id 0, runner_name empty. Its started_at equals creation despite queued state; that is not execution evidence. No completed Rust plugin receipt/log exists in this collected snapshot. |
| [125721731685 — installation key restore / Linux](https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/job/125721731685) | ubuntu-24.04 | completed / success | Started 2026-11-03T04:19:55Z; completed 04:22:16Z. Original decoded job log and artifact 13204611490 read and preserved. |

The wrapper job follow-up returned the same three states. No continuing polling loop was started for the queued macOS job. Queue cause and completion time are not exposed. These are pending native evidence states, not an implementation-failure finding.

## Linux installation-key restore: actual command and results

The original job log and compiled.json record:

```
cargo test --locked --manifest-path source/host/Cargo.toml --test box_store_download_frozen_parity_contract --no-run --message-format=json
```

Compilation exit_code is 0 and compiler_uid is 1001. The actual compiled test executable was then invoked:

1. Ordinary invocation with --nocapture: 31 discovered tests; 29 passed, 0 failed, 2 ignored.
2. sudo -n unshare --mount --propagation private -- <same executable> --exact linux_bind_mounted_descendant_file_restore_boundary --ignored --nocapture: 1 passed, 0 failed, 0 ignored, 30 filtered out; exit 0.
3. Equivalent private mount namespace invocation for linux_bind_mounted_descendant_symlink_restore_boundary: 1 passed, 0 failed, 0 ignored, 30 filtered out; exit 0.

Thus the two ordinary ignored tests have explicit separate positive executions; there are 31 successful test executions across these three invocations, not 33 distinct passing tests. execution.json marks all three oracle_accepted true and result passed, while preserving independent_acceptance pending.

Actual scope: ordinary installation-key restore boundaries and real private-namespace bind-mounted descendant file/symlink replacement boundaries, using synthetic installation keys and owned fixtures. This includes key/staging native aliases, ancestor/descendant hardlinks, symlink routes, protected namespace refusals, ordinary-neighbor restores, staged-key identity limits and unchanged-file controls. It does not establish exclusion of concurrent external namespace mutation, full backup/client compatibility, plugin Rust owner/cancellation/restart, or packaged/global acceptance.

## Artifact and byte/identity readback

Artifact:
- ID 13204611490.
- Name ocrm-config-restore-linux-23c2a567d874509f56353b8cae737cdf1437309d-41576693105-1.
- URL https://github.com/bhrumom/fabushi-desktop/actions/runs/41576693105/artifacts/13204611490 .
- Size 47,203 bytes; created 2026-11-03T04:22:13Z; expired false.
- API digest, original upload log digest and downloaded ZIP SHA256 all equal 2ebad053db95f18274b2124abcdb500caf4094d3b351b1f719e0dac618b45f29.
- subject.json and execution.json both bind H23, tested tree, U, run 41576693105, attempt 1 and Linux.

Comparisons performed without executing repository validation:
- compiler-artifacts.jsonl SHA256: 8bd0a1b8014e8694bbba004517e89ab6615f1a6dab71d13c8597a8c5faf70416; matches compiled receipt.
- ordinary.log: d66095448a69172ee362a419a24dc07af8c5fef4265f19134810e3563547c1bc; matches execution receipt.
- file mount log: 6a4e7e28651c7c2b1c5866aceeccaa4a15c99c363f0d035d26552ed73e1c7bf4; matches execution receipt.
- symlink mount log: 440cf3d7e0e6eace8956a7f34d65489d87be9b68b0d67ae796d9df965585c3b7; matches execution receipt.
- H23 source/host/Cargo.toml bytes: 3fe05f12ecd0aef14035ea61a48b909614188d21b89ab27697bf8b737a84a387; matches compiled receipt.
- H23 source/host/Cargo.lock bytes: 96b7c7e3faa9faf3aa54b800dccb17ae7a0ecebf59ca7e2e1a948b9ae0545eaf; matches compiled receipt.
- The executable SHA256 6e65582d7ecb53d17913ece4d3f1e83299162a4568ccaf7e5f2ee5afe6a371ef is a runner-recorded value. The binary is not in this seven-entry archive and was not independently rehashed here.
- Preserved decoded full job log SHA256: 83ad8a35daed68a2bac907f6a582c97d602cb0d159a6d88e23f05d362a748e49.
- Exact H23 portable capture source was fetched from GitHub after local object content was unavailable; its saved Git blob identity ee404c54ce24a740f6808a8c7bba9650a0f0e2d6 matches the H23 tree entry. No script was executed.

## Plugin Rust owner / cancel / restart coverage boundary

The H23 workflow and portable capture script declare native checks; declarations are not completed execution evidence.

The in-progress SQLite step invokes projects/openclaw-rust/scripts/capture_portable_contracts.py. That exact H23 source includes:
- plugin-uninstall-service: existing MCP service, plugin Skills, Host bundle/cache, workflow, legacy references, Coordinator relay, extension/production and Host production extension integration targets.
- plugin-uninstall-state: extensions::mcp::plugin_skills:: library cases, including pending/confirmed durable refusal, exact-ticket settlement, ordinary-sync no-release/no-prune, old-load fencing, recovery faults and same-root successor settlement refusal.
- plugin-uninstall-binding: actual Host binding recovery and same-root reader custody.
- plugin-uninstall-relay-handoff: one exact Coordinator lifecycle reader-handoff case.
- plugin-cache-publication and plugin-session-consumer library groups.
- Required service positives include close-after-reservation behavior, closed service no replay/new dispatch, stale reply refusal, and owned producer process exit with fresh process consumers.

The queued macOS rust-host workflow declares overlapping plugin integration/library groups and a durable-uninstall-positive receipt step. Neither job has a completed plugin Rust receipt in this collected snapshot. Therefore no passed owner/cancel/restart coverage is asserted here.

Even a later receipt from these H23 jobs would attest H23 only. It could not accept the bb5a7f6098f952df6d388a672c4fb01045bc645c auth-recovery changes or the carrier. Declared plugin scope also leaves account/backend incarnation, remote receipt/quiescence reconciliation, real ecosystem, hardware power-loss and packaged acceptance open.

## Acquisition notes and exclusions

- GitHub generic fetch rejected direct /actions/jobs/125721731498 with INVALID_ARGUMENT because that endpoint is not on its allowlist. The supported workflow-jobs wrapper provided the focused follow-up; this is an acquisition-tool limitation, not a job failure.
- Local git show could not read the portable capture blob, while ls-tree identified its exact object. The exact-H GitHub source read recovered it; its blob identity was then compared as above.
- Renderer 125721731215 and Windows 125721731609 were not recollected.
- Linux media artifact 13204156561 is reserved to the primary integrator and was not downloaded or inspected by this agent.
- This report does not promote all-job completion, unit verification, global verification, or packaged acceptance.
