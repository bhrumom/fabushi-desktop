# Round 163 — fixed-U Process Inbox supplied-video helper

- Task round: 132
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `658e769364eca39177baf0453f3f6212263a0438`
- Spec-first desktop commit: `910abf5ffd6f60f86d65ccde5c76d2ea99b37867`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, recursive `truncated=false`, 7,194 entries / 6,859 blobs
- Closed source: `.agents/skills/process-inbox/scripts/video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3`
- Source identity: recursive-tree index 79 zero-based; 9,024 Git bytes; 9,024 ASCII characters; 292 newline characters with trailing newline

The exact helper implements repository-planning video evidence only: ffprobe-based per-frame PTS normalized to the first frame, SHA-256/probe metadata, validated sorted unique frame/range selection, strict crop syntax, contiguous-run ffmpeg selection, crop/scale filtering, passthrough PNG decode, requested/decoded count fencing, deterministic frame-index/time names, and bounded labelled Pillow sheets. Helper-owned raw decode intermediates live in `TemporaryDirectory` and are cleaned when scope exits. Missing dependencies, malformed/out-of-range selection, empty decode, child-process failure and frame-count mismatch fail closed. Target-side move/save failures are not transactionally rolled back and any partial output remains non-authoritative.

The only fixed-U direct invocation is `process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`. It keeps the original supplied video local in transaction/backup state, bulk extraction in disposable scratch, and commits only compact phase-complete crops/sheets plus key/context frames. The later perform-task pipeline consumes the tracked task spec and referenced `input/` files; it has no required local-video dependency.

Existing owner remains `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`. Shipping attachment/resource/media owners are deliberately not selected because this source is repository-development evidence transformation, not product media ingress/runtime. No new responsibility, ADR, product owner or platform-core delta was introduced.

Authored/not-run contracts: `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-PROBE-PTS-001`, `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-FRAME-EXTRACT-001`, `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-DURABLE-REFERENCE-001`, `INT-TDRP-PROCESS-INBOX-VIDEO-PERFORMER-HANDOFF-001`, `FAULT-TDRP-PROCESS-INBOX-VIDEO-CLEANUP-001`.

Unique-ledger closure: `source_entries_total=16202 / unknown=16145 / unread=16118 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`. Implementation state remains `implemented`, `validation_status=not-run`, `accepted=false`.

Next independent fixed-U blob: `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, video helper/ffmpeg/ffprobe/Pillow invocation, process-inbox helper/inbox transaction/source-lineage command, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deploy, install, runtime evidence or acceptance was performed.
