# Round 159 — fixed-U perform-task phase prompts

Task round: 128
Stage: A / zero executable validation
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Source: `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c`

Live authority: desktop checkpoint parent `f46416dd12f12839e5221217b3c8f2ec2b4e5909`; spec-first checkpoint `947aa8df76e857adaa681ce8520cf3e9dfa633a3`; desktop main `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core checkpoint `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed-U recursive tree is complete (`truncated=false`, 7,194 entries); target tree index 72, exact 64,311-byte Git blob / 64,245 decoded characters / 1,215 newline-delimited lines.

Complete source read covers orchestration and artifact-based completion; Phase 1 context/plan; Phase 3 assessment including fast-path/approach rejection and split-required; Phase 4 implementation owned-write-set discipline; selected pre-review validation; independent initial general/lens review; focused re-review with approval carry-forward; convergence; evidence-flaw directness recovery and campaign-cap assessment; native-Windows normalization; prompt/result logs; and Claude/Grok/Codex leaf execution mechanics.

Retry/resume/worker contract: the performer alone owns task state/publication; leaf workers do not delegate or commit; delegation is selected only after a real spawn succeeds; immediate host rejection may choose the declared same-session fallback; incomplete delegated Phase 1/3/4/6 receives only the bounded disposable retry and then a recoverable hard stop, not a local replay or canonical Block; completed validated artifacts are reused on resume.

Direct fixed-U consumers/composition verified: `perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`, `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154`, and `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d`.

Existing-owner-first: existing `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` + repository quality/test-evidence governance. No new responsibility, ADR, product runtime owner or platform-core production delta.

Spec-first commit: `947aa8df76e857adaa681ce8520cf3e9dfa633a3`.
Dossier: `projects/telegram-desktop-rust/dossiers/perform-task-phase-prompts-complete-read.md`.
Authored/not-run: `CONTRACT-TDRP-PERFORM-TASK-PHASE-PROMPT-ARTIFACT-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-ASSESSMENT-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-REVIEW-FOCUS-001`, `FAULT-TDRP-PERFORM-TASK-PHASE-WORKER-RECOVERY-001`, `FAULT-TDRP-PERFORM-TASK-EVIDENCE-CAMPAIGN-001`.

Closure coverage: `source_entries_total=16202 / unknown=16149 / unread=16122 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.
Next independent fixed-U blob: `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger and no executable validation, code-health helper, build, reviewer execution, test, visual/a11y run, package, deploy, install or acceptance.
