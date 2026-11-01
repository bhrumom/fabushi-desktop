# Round 157 — fixed-U perform-task Computer Use testing adapter

Task round: 126
Stage: A / zero executable validation
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Source: `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`

Live authority was reread before work: desktop checkpoint `ba5bcf2beafaaa8264ce46bda31eda36512f9af1`, then spec-first checkpoint `4e40c0b0653a1d81c117e17e48c16d0f6604610b`; desktop main `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed-U recursive tree was re-read complete (`truncated=false`, 7,194 entries); target index 71, exact 14,178-byte blob.

The target source was read completely. Direct fixed-U references are the shared perform-task skill and pipeline; shared test-loop stays authoritative for scenarios/oracles/evidence assessment. Existing-owner-first maps the portable UI-driver selection, exact-subject, bounded operation, safety, recovery and durable-evidence contracts into the existing repository performer + quality/UI-evidence owner, with no new responsibility/ADR/platform-core production delta.

Spec-first commit: `4e40c0b0653a1d81c117e17e48c16d0f6604610b`.
Authored/not-run contracts: `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SELECTION-001`, `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SAFETY-001`, `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-EVIDENCE-001`, `FAULT-TDRP-PERFORM-TASK-UI-DRIVER-RECOVERY-001`.

Coverage after closure: `source_entries_total=16202 / unknown=16151 / unread=16124 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`. Next independent fixed-U blob: `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger and no executable validation, UI-driver operation, build, visual/a11y run, packaging, deployment, installation or acceptance.
