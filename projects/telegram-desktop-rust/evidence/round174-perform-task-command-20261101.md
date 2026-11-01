# Round 174 — fixed-U Claude perform-task command adapter

- Task round: 143
- Date: 2026-11-01
- Stage: A / zero executable validation
- Desktop checkpoint parent: `ce410ff4f35bdfdfcf1a2418e254a7fe9d437c2d`
- Spec-first desktop commit: `827e7761fb9c5bf040c0e1a9206c58e7b8467618`
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, complete, 7,194 entries / 6,859 blobs
- Closed source: `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e`
- Required Claude adapter: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`
- Required shared performer: `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`

Complete read confirms a thin Claude Code command wrapper around the existing single-task performer. The source is 2,068 bytes / 2,068 decoded characters / 42 newlines with trailing newline, recursive-tree index 93 zero-based. It owns no task state or queue: one named task is resolved/started/resumed and driven through shared implementation/review/evidence/publication, then the command stops.

Wrapper-specific contracts preserve the source-lineage pre/post-Phase-1 split, split-required retained-state stop, foreground Agent delegation, Codex/native review-provider choice without semantic drift, and conditional per-commit `[ai] ` subject rule. Direct host consumer is Claude Code/user `/perform-task`. The `continue` scheduler consumes the shared performer directly; Grok and OpenAI perform-task surfaces are peer adapters, not callers of this wrapper.

Existing-owner-first reuses the canonical performer + Claude host-adapter + quality/evidence owners and introduces no shipping Product Tasks/Automations owner, Agent/Coordinator/Host/Runner runtime owner, platform-core service, duplicate performer pipeline or second task/review/evidence truth.

Authored/not-run: seven contract cases, one integration case and two fault-recovery cases. Unique ledger after closure: `source_entries_total=16202 / unknown=16134 / unread=16107 / omitted=0 / responsibilities_total=144 / unmapped_responsibilities=0`; implementation `implemented`, validation `not-run`, accepted `false`.

Next fixed-U tree blob: `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no workspace helper, Agent, Codex leaf, source-lineage command, build, typecheck, lint, validator, reviewer, test/evidence campaign, UI driver, publication simulation, packaging, deployment, installation, runtime evidence or acceptance was performed.
