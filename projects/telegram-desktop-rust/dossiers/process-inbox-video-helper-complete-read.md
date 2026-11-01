# Complete read — fixed-U process-inbox supplied-video helper

Date: 2026-11-01  
Task round: 132  
Stage: A / zero executable validation

## Authority snapshot

- Desktop checkpoint parent: `658e769364eca39177baf0453f3f6212263a0438`.
- Desktop main reread: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core exact commit: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs.
- Source entry: `.agents/skills/process-inbox/scripts/video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3`, recursive-tree index 79 zero-based.
- Exact source: 9,024 Git bytes / 9,024 ASCII characters / 292 newline characters, with a trailing newline.

## Complete source contract

The 293-split-line source was read from the module contract through `main()`. It is a command-line repository-planning evidence helper with three commands: `probe`, `extract` and `sheet`. It does not own product media, message attachments, a repository queue, task state, publication or acceptance.

`require_tool` fail-closes missing executables. `run` captures stdout/stderr and converts a non-zero child exit into `VideoError`. `frame_times` invokes ffprobe against video stream 0 for `best_effort_timestamp_time`, rejects an empty decoded-frame set, and normalizes every timestamp to the first frame. This preserves variable-frame-rate timing rather than deriving time from nominal fps. `sha256` streams the local source file in 1 MiB chunks.

`parse_frames` accepts `all`, comma-separated indices and inclusive ranges, deduplicates and sorts them, and rejects negative/out-of-range indices. `parse_crop` accepts only `WxH+X+Y` decimal geometry. `filter_chain` compresses contiguous frame selections into ffmpeg `select` runs, then applies crop and optional scaling; enlargement uses `neighbor`, shrink uses `area`. `decode` writes selected PNGs into a caller-supplied temporary directory with `-fps_mode passthrough` and rejects any requested/decoded count mismatch. `frame_name` binds each durable image name to decoded index, normalized milliseconds and optional label.

`command_probe` emits JSON covering basename, SHA-256, bytes, codec, width/height, nominal rate, frame count, last-frame time and container duration. When intervals exist it adds min/max interval milliseconds and `constant_frame_rate`, true only when every decoded interval is within one millisecond of nominal period.

`command_extract` computes authoritative times before selection, creates the target directory, decodes into `TemporaryDirectory`, then moves selected PNGs to deterministic frame/time names and reports output/count/first/last as JSON. `command_sheet` requires Pillow, samples the selected indices by `--step`, rejects more than 200 cells, decodes through a temporary directory, renders labelled RGB cells with frame/time, creates the parent target directory and saves one contact sheet. Both commands therefore clean helper-owned raw decode intermediates when their temporary-directory scope exits.

The CLI exposes `--frames`, `--crop`, `--scale`, `--out`, extract `--label`, and sheet `--step`/`--columns`. `main` converts `VideoError` into a stderr `error:` and exit status 1. Other Python/filesystem/image exceptions are not converted into transactional output rollback; partial target-side output from such a later failure must not be mistaken for complete evidence.

## Direct callers and downstream consumers

The only fixed-U direct invocation contract is `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`. Its supplied-video section invokes:

- `probe` on the immutable transaction video;
- full `extract` and overview `sheet` into disposable scratch for phase discovery/measurement;
- selected/cropped `extract` and `sheet` outputs into a task's tracked `input/frames`.

That caller makes the durable boundary explicit: frame/time citations use decoded PTS, full extraction is scratch-only and discarded, committed crops/sheets cover every task-owned animated phase, the original video stays local in transaction/backup state, and the receipt records probe facts plus digest so the source can be located again.

Exact fixed-U `workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`, `workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`, Claude `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`, Grok `.grok/commands/process-inbox.md@5fcd9055de9b14a5163dc96a6c3551096898de18`, and root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` contain no direct `video.py`, ffmpeg or ffprobe invocation. `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` is the downstream generic consumer: it treats the external `task.md` and referenced task `input/` files as sufficient inputs.

## Existing-owner resolution

Existing owner remains `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`: Fabushi repository development process authority plus repository quality/test-evidence governance.

The helper is development-process media-input evidence, not shipping product media. Current Fabushi product ownership already places user/runtime resource ingress under composer attachments + Electron attachment manager + Host attachment extensions, and media transfer/viewing under the canonical attachments/artifacts/resource lifecycle. Those are rejected for this source because no user message, blob service, media viewer, product cache or runtime resource is being created. Mapping the helper there would mix repository-planning evidence with product media truth.

Rejected duplicate/different owners:

- shipping attachments/artifacts/resource ingress and media transfer/cache;
- Product Tasks / Automations;
- Agent / Coordinator / Host / Runner runtime;
- platform-core production runtime;
- a Telegram-specific media-input service or copied video-helper runtime;
- a second repository receipt/evidence store.

No new ADR and no platform-core production delta are required.

## Fabushi contract delta

The existing repository inbox/process governance is extended to make supplied-video evidence deterministic and portable: probe/digest identity, PTS-based frame timing, validated frame/crop selection, compact durable frame inputs, local-only raw video/full extraction, performer handoff through tracked task inputs, and fail-closed temporary-decode cleanup.

Authored, not run:

- `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-PROBE-PTS-001`
- `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-FRAME-EXTRACT-001`
- `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-DURABLE-REFERENCE-001`
- `INT-TDRP-PROCESS-INBOX-VIDEO-PERFORMER-HANDOFF-001`
- `FAULT-TDRP-PROCESS-INBOX-VIDEO-CLEANUP-001`

No helper invocation, ffmpeg/ffprobe/Pillow execution, process-inbox transaction, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, packaging, deployment, installation, runtime evidence or acceptance was executed.

## Source-closure disposition

This complete read maps exactly one additional independent fixed-U source entry, `.agents/skills/process-inbox/scripts/video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3`, into the existing repository process responsibility. Responsibility count does not increase.

The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`.
