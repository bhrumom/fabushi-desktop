# Round 179 — fixed-U macOS clipboard-image acquisition host adapter

- Task round: 148
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `5b6e9f9c62cca0741e67382882fb0470168a99b4`
- Spec-first desktop commit: `3d64c8ff0627423e0c1f3c61009d4f143776fd66`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries / 6,859 blobs
- Closed source: `.claude/grab_clipboard.sh@80c0c6d05c99a8f1fca6c57b325f561440e7b22e`
- Tree index: 98 zero-based
- Source identity: 565 Git bytes / 563 decoded UTF-8 characters / 28 newline characters / trailing newline / 29 split lines
- Responsibility: `TDRP-R9-MACOS-CLIPBOARD-IMAGE-ACQUISITION-HOST-ADAPTER-001`

## Verified source and composition

The complete fixed-U shell helper was read by exact blob and re-read by fixed-U path identity. It rejects missing/empty `outPath` with usage/exit 1. With a controlled path it invokes `osascript`, requests clipboard PNG flavor, opens the destination with write permission, writes the bytes, closes the handle, returns `ok`, and only shell pipeline success through `grep -q "ok"` becomes `Saved to <path>` with exit 0.

The source-level clipboard conversion error returns `no image`, but the shell suppresses AppleScript stderr and maps every non-`ok` pipeline result to the same outer `No image on clipboard`/exit 1. Therefore missing PNG content, unavailable/failing `osascript`, AppleScript parse/runtime failure and destination/path/permission/open/write/close failures all remain non-success, while the outer message is non-diagnostic and cannot establish root cause.

Direct consumer `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e` was fully reread. Its macOS Phase-0 branch generates `.ai/icon_<random-hex>.png`, invokes the helper before any vectorization, stops on failure, rejects chat-pixel fallback, rereads the saved file, and requires preview/source match. The Windows helper is a peer only.

## Existing-owner-first disposition

The helper is repository-development input acquisition, not a shipping clipboard subsystem. Ownership remains with the existing icon source-input contract, Fabushi design/canonical icon asset authority, repository process, and quality/evidence lifecycle. The controlled temp-path boundary is preserved; the helper is not widened into an arbitrary untrusted path API. Product clipboard/history, attachment/resource/media persistence, Computer, Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core, Telegram/vectosolve input stores, durable `.ai/icon_*` state, and duplicate icon/evidence registries remain rejected.

Spec-first artifacts:
- `AGENTS.md` macOS clipboard host-adapter authority
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` qualification cases
- `projects/fabushi-communication-platform/quality/evidence-contract.md` evidence rules
- `projects/telegram-desktop-rust/dossiers/claude-macos-clipboard-image-acquisition-complete-read.md` complete-read dossier

No platform-core production delta, service, database, shipping UI/runtime surface or ADR is required.

## Deferred Stage-B qualification

Authored only, not run:
- six contract cases;
- one integration case;
- three fault-recovery cases;
- one security/privacy case.

Ledger state is `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`. Coverage moves exactly one independent source entry: unknown `16130 -> 16129`, unread `16103 -> 16102`, omitted stays 0, responsibilities `148 -> 149`, unmapped remains 0.

No Actions/check/status/workflow query or trigger, shell/AppleScript/clipboard access, PNG write, provider/vectorization, renderer/build, typecheck/lint/validator, test, visual/a11y scan, packaging, deployment, installation, runtime evidence or acceptance was performed. The next fixed-U blob after the intervening `.claude/scripts` tree is `.claude/scripts/crash_triage.py@2db6cc14d8e529c4e61c5dcf169aade31fd35c58`.
