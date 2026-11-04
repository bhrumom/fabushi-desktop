# Fixed-U complete read — macOS clipboard-image acquisition host adapter

- Source repository: `telegramdesktop/tdesktop`
- Fixed source commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Source path: `.claude/grab_clipboard.sh`
- Blob: `80c0c6d05c99a8f1fca6c57b325f561440e7b22e`
- Recursive fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`
- Tree position: index 98 zero-based
- Identity: 565 Git bytes, 563 decoded UTF-8 characters, 28 newline characters, trailing newline, 29 split lines
- Stage: A; executable acquisition/validation deferred
- Desktop authority parent: `5b6e9f9c62cca0741e67382882fb0470168a99b4`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete source read

The complete shell file was read. It accepts its first positional argument as `outPath`. If that value is missing or empty, it prints `Usage: grab_clipboard.sh <output.png>` and exits 1 without invoking AppleScript.

For a non-empty path, the script invokes `osascript -e` with an AppleScript program. That program converts the caller path to a POSIX file, attempts to coerce the current clipboard to PNG flavor `«class PNGf»`, returns `no image` if that clipboard conversion raises an error, otherwise opens the destination with write permission, writes the PNG-flavor bytes, closes the file and returns `ok`.

The shell suppresses AppleScript stderr and pipes stdout to `grep -q "ok"`. It then branches on the pipeline status. A matching `ok` prints `Saved to <outPath>` and exits 0. Any non-matching/non-success pipeline result prints `No image on clipboard` and exits 1. Therefore the final failure message is not a reliable root-cause discriminator: it can represent true missing/unsupported clipboard image data or another AppleScript/path/open/write/close failure that did not produce `ok`.

The source has no retry loop, background watcher, network access, directory creation policy, canonical asset naming, product Resource registration, chat-image fallback, credential handling, or durable clipboard history. It also does not establish a safe arbitrary-untrusted-path interface: the caller-provided path is interpolated into the AppleScript source. The qualified consumer supplies a repository-generated random-hex temporary path, so existing-owner adaptation keeps that controlled-path boundary instead of widening the helper into a general product file API.

## Direct consumer and peer adapter

Fixed-U `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e` is the direct file-level consumer. In Phase 0, when a user message has an attached image, the command generates a temporary `.ai/icon_<random-hex>.png` path and selects this shell helper on macOS or the PowerShell helper on Windows.

The command treats helper failure as terminal for that source attempt, asks the user to re-copy the icon area, and explicitly refuses to substitute chat pixels. After capture it reads the saved PNG and compares it with the pasted/requested image; a clear mismatch is deleted and stops. Thus helper success is acquisition only, not source approval.

Fixed-U `.claude/grab_clipboard.ps1@bece778c6038820ca4d16bbf4224548de1e385a8` is the Windows peer. It is not a caller of the shell helper and does not create a second durable source truth.

## Input/output, failure and recovery contracts

Input is one controlled caller-selected path plus ambient macOS clipboard image state. Missing/empty path is an explicit pre-acquisition failure. On a successful attempt the helper writes the clipboard PNG flavor to the exact caller path, obtains the `ok` pipeline result, reports the saved path and exits 0.

All other outcomes are non-success. PNG-flavor conversion failure is explicitly converted by AppleScript to `no image`; unavailable/failing `osascript`, AppleScript parse/runtime error, invalid destination, missing directory, permission/open/write/close failure, or any other non-`ok` result is also fail-closed. Because stderr is suppressed and the shell emits the same generic no-image message after the pipeline, caller/evidence logic must not infer the underlying cause from that text alone.

A stale, partial or pre-existing destination is not approval. The helper defines no canonical overwrite/truncation policy; the existing consumer uses a fresh random-hex temp path and owns cleanup/disposition. Recovery is caller-driven: re-copy the intended image and retry, or correct the destination/host environment and retry. A retry is a new acquisition result; prior failure evidence is not rewritten.

The transient PNG lifetime belongs to the calling icon workflow. It may proceed only to the existing source-match gate and scratch lineage. It never becomes a Fabushi product Resource, attachment, canonical icon or visual evidence merely because it was written.

## Existing-owner-first mapping

Responsibility: `TDRP-R9-MACOS-CLIPBOARD-IMAGE-ACQUISITION-HOST-ADAPTER-001`.

Existing owners:
- root repository development-process authority — repository-local host adapter scope and caller ownership;
- already-qualified `TDRP-R9-ICON-GENERATION-CLAUDE-COMMAND-ADAPTER-001` source-input contract — controlled temp path, source match, scratch lifetime, generation routing and recovery;
- Fabushi design system / canonical icon-asset owner — only canonical asset destination;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` and `evidence-contract.md` — deferred qualification/evidence.

Rejected owners:
- product clipboard service/history;
- product attachment/resource/media persistence;
- shipping Product Tasks / Automations;
- Agent / Coordinator / Host / Runner runtime;
- Computer runtime;
- platform-core service;
- a Telegram-specific or vectosolve-specific input store;
- `.ai/icon_*` scratch paths as durable production truth;
- a second icon registry or evidence system.

No platform-core production delta, service, durable database, UI surface, search owner, shipping runtime module or ADR is introduced.

## Authored contracts — not run

- `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-INPUT-PATH-001`
- `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-PNG-PIPELINE-001`
- `CONTRACT-TDRP-MACOS-CLIPBOARD-IMAGE-FAIL-CLOSED-001`
- `CONTRACT-TDRP-MACOS-CLIPBOARD-ICON-CONSUMER-001`
- `CONTRACT-TDRP-MACOS-CLIPBOARD-RECOVERY-001`
- `INT-TDRP-MACOS-CLIPBOARD-ICON-COMPOSITION-001`
- `FAULT-TDRP-MACOS-CLIPBOARD-MISSING-PATH-001`
- `FAULT-TDRP-MACOS-CLIPBOARD-NO-IMAGE-001`
- `FAULT-TDRP-MACOS-CLIPBOARD-SAVE-FAILURE-001`
- `SEC-TDRP-MACOS-CLIPBOARD-CONTROLLED-PATH-001`

All remain `validation_status=not-run`, `accepted=false`. No shell/AppleScript/clipboard access, PNG write, vectorization, renderer/build, validator, test, visual/a11y execution, CI, packaging, deployment, runtime evidence or acceptance was performed.
