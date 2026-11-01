# Round 156 — fixed-U OpenAI perform-task host adapter

Task round: 125  
Stage: A / zero executable validation  
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Source: `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`

Live authority was reread before work: desktop checkpoint `56a503be59d4fe1378e7475708d7002dc8f7ec09`, then spec-first checkpoint `208dba1916b0ff233e7cadce2d033ce4a9472101`; desktop main `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed-U recursive tree is complete (`truncated=false`, 7,194 entries); the target is recursive-tree index 69 (1-based including tree entries), 229 bytes / 4 logical lines. Before closure the unique ledger was `source_entries_total=16202 / unknown=16153 / unread=16126 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

The exact YAML was read completely. Its portable contract is limited to OpenAI host discovery/invocation metadata: `display_name = "Perform AI Task"`, one-task short description, and a default prompt that invokes `$perform-task` to implement, adaptively review/test and publish one named ai-tdesktop task.

Direct consumers/boundaries were re-read from fixed-U `perform-task/SKILL.md`, Claude/Grok perform-task commands, continue scheduler, code-health skill and root AGENTS. They establish that host adapters only invoke the canonical performer. Task state, ownership, source-lineage, phase/resume, review/convergence, evidence recovery, queue/routing/consolidation and final publication remain under the shared repository process/quality authorities.

Existing-owner-first extends `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001`. Spec-first CAS commit `208dba1916b0ff233e7cadce2d033ce4a9472101` extends existing root governance/Test Strategy/Evidence Contract and adds `perform-task-openai-host-adapter-complete-read.md`. No new responsibility/ADR/platform-core production owner or second performer/task/evidence store was introduced.

Authored, not run: `CONTRACT-TDRP-PERFORM-TASK-OPENAI-HOST-METADATA-001` and `INT-TDRP-PERFORM-TASK-OPENAI-HOST-INVOCATION-001`.

After closure: `source_entries_total=16202 / unknown=16152 / unread=16125 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`. Exactly one independent source entry closes. Next independent fixed-U blob: `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`.

Zero-CI attestation: no Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation, runtime evidence campaign or acceptance.
