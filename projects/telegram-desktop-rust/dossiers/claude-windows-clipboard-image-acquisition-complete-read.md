# Fixed-U complete read — Windows clipboard-image acquisition host adapter

- Source repository: `telegramdesktop/tdesktop`
- Fixed source commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Source path: `.claude/grab_clipboard.ps1`
- Blob: `bece778c6038820ca4d16bbf4224548de1e385a8`
- Recursive fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`
- Tree position: index 97 zero-based
- Identity: 306 Git bytes, 306 decoded ASCII characters, 11 newline characters, trailing newline, 12 split lines
- Stage: A; executable acquisition/validation deferred
- Desktop authority parent: `b165387f233904fce062458385307e34a5d51482`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core authority: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Complete source read

The complete PowerShell file was read. It declares one string parameter, `outPath`; loads `System.Windows.Forms`; reads the current Windows clipboard image with `[System.Windows.Forms.Clipboard]::GetImage()`; and branches only on whether an image object was returned.

When an image exists, the script calls `$img.Save($outPath, [System.Drawing.Imaging.ImageFormat]::Png)`, writes `Saved to $outPath`, and exits 0. When the clipboard does not contain an image, it writes `No image on clipboard` and exits 1. The source has no loop, background watcher, network access, retry policy, directory creation, canonical naming, resource registration, chat-image fallback, credential handling, or product state.

The source does not catch assembly-load, clipboard-access, invalid/missing destination, directory, permission, encoder or file-write exceptions. Existing-owner adaptation therefore treats any exception/non-success host result as fail-closed acquisition failure; it does not infer success merely because the no-image branch was not taken.

## Direct consumer and peer adapter

Fixed-U `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e` is the direct file-level consumer. In Phase 0, when a user message has an attached image, the command generates a temporary icon path and selects the macOS shell helper or this PowerShell helper by host OS. On Windows it invokes `powershell -ExecutionPolicy Bypass -File .claude/grab_clipboard.ps1 ".ai/icon_<hash>.png"`.

The command treats helper failure as terminal for that source attempt, asks the user to re-copy the icon area, and explicitly refuses to substitute chat pixels. After capture it reads the saved PNG and compares it with the pasted/requested image; a clear mismatch is deleted and stops. Thus helper success is only acquisition, not source approval.

Fixed-U `.claude/grab_clipboard.sh@80c0c6d05c99a8f1fca6c57b325f561440e7b22e` is the macOS peer. It is not a caller of the PowerShell helper and does not create a second durable source truth.

## Input/output, failure and recovery contracts

Input is one caller-selected path plus ambient Windows clipboard image state. Output is a transient PNG at that exact path only when acquisition and save succeed, accompanied by a success message and exit 0. The explicit no-image outcome is message plus exit 1. Other host/runtime/file failures remain non-success and must not advance the icon generation pipeline.

Recovery is caller-driven: re-copy the intended image and retry, or correct the destination/path/permissions/environment and retry. A retry creates a new acquisition result; prior failure must not be rewritten as success. The helper itself does not retry, search other sources, use chat pixels, fetch network images, persist a clipboard history, or choose a canonical asset target.

The transient PNG lifetime belongs to the calling icon workflow. It may be moved/copied into the existing icon scratch lineage and must still pass the source-match gate. It never becomes a Fabushi product Resource, attachment or canonical icon merely because it was saved.

## Existing-owner-first mapping

Responsibility: `TDRP-R9-WINDOWS-CLIPBOARD-IMAGE-ACQUISITION-HOST-ADAPTER-001`.

Existing owners:
- root repository development-process authority — repository-local host adapter scope and caller ownership;
- already-qualified `TDRP-R9-ICON-GENERATION-CLAUDE-COMMAND-ADAPTER-001` source-input contract — source match, scratch lifetime, generation routing and recovery;
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

- `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-HOST-AUTHORITY-001`
- `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-IO-001`
- `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-PNG-001`
- `CONTRACT-TDRP-WINDOWS-CLIPBOARD-IMAGE-FAIL-CLOSED-001`
- `CONTRACT-TDRP-WINDOWS-CLIPBOARD-ICON-CONSUMER-001`
- `CONTRACT-TDRP-WINDOWS-CLIPBOARD-RECOVERY-001`
- `INT-TDRP-WINDOWS-CLIPBOARD-ICON-COMPOSITION-001`
- `FAULT-TDRP-WINDOWS-CLIPBOARD-NO-IMAGE-001`
- `FAULT-TDRP-WINDOWS-CLIPBOARD-SAVE-FAILURE-001`
- `SEC-TDRP-WINDOWS-CLIPBOARD-INPUT-BOUNDARY-001`

All remain `validation_status=not-run`, `accepted=false`. No clipboard read, PowerShell execution, PNG write, vectorization, renderer/build, validator, test, visual/a11y execution, CI, packaging, deployment, runtime evidence or acceptance was performed.
