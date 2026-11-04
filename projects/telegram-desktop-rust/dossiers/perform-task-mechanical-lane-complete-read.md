# Complete read — fixed-U perform-task mechanical lane

## Authority snapshot

- Desktop checkpoint parent: `f7d19783760cf870f07e9efd6567a9402109f0df`.
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core checkpoint: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Recursive tree: `truncated=false`, 7,194 entries.
- Source: `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`, tree index 71, 9,933 bytes / 9,929 decoded characters / 194 newline-delimited lines.

## Complete contract

This source defines the bounded behavior-preserving lane for code-health Q0 deletion, Q1 move and Q2 merge-identical-copies. Q0 removes only unreferenced code plus exclusively-used declarations/includes/CMake/style/language residue. Q1 moves code without logic edits except declaration/include/namespace/CMake scaffolding. Q2 consolidates copies identical except names/literal values into one already-linkable shared function with differences passed explicitly. New behavior, strings/styles/settings/logs, side-effect reorder, templates, header-defined functions or variant dispatch are disallowed.

The lane rechecks each finding against the exact task base because findings may be stale. Present remains actionable; unknown requires reading the named source locations and is dropped only when gone or no longer class-conformant. Owned paths are declared before edits and are limited to task-named files, the legal receive/shared file, their CMake lists, direct-user include lines, relevant area-map lines and exclusively-used style/language entries. Unrelated defects or nearby cleanup are recorded and left alone.

Proof requires changed-target Debug/build coverage, repository health, Q1 move-only proof when applicable, and candidate verify/metric. A failing proof is fixed only within bounds or causes restore/drop. Every edit/drop invalidates affected proof and the remaining diff is reproved. If nothing remains, the checkout returns to base and the task publishes already-satisfied.

One fresh reviewer reads the task/spec, area map, complete diff, every changed file and all proof outputs; it checks behavior, completeness, bounds, proofs and placement. A first NEEDS_CHANGES may receive one bounded fix and focused resume; a second NEEDS_CHANGES drops the named items. There is no third review. Retained work publishes source commit + test proof + final review; all-dropped publishes already-satisfied with drop evidence and no review. Result records Resolved, Dropped and Metric.

## Direct callers/consumers

- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` selects this lane exactly for canonical Q0/Q1/Q2 tasks.
- `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` names it as the only class-bounded exception to the ordinary adaptive review/evidence path.
- `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` routes Q0-Q2 findings to perform-task while remaining non-editing itself.
- `.agents/skills/code-health/references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e` defines the Q0/Q1/Q2 no-behavior-change and proof classes; Q3-Q5 stay on the full pipeline.

## Existing-owner resolution

Existing-owner-first keeps `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001`: Fabushi repository development process authority plus existing Test Strategy/Test Evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Telegram-specific performer/queue and a second review/evidence store are rejected as duplicate/different state domains. No new responsibility, ADR or platform-core production delta is required.

## Stage-A delta

Authored/not-run: `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-ELIGIBILITY-001`, `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-OWNED-EDIT-001`, `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-PROOF-DROP-001`, `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-REVIEW-PUBLICATION-001`. The previous umbrella mechanical-lane contract remains valid. FMEP-001 defers all executable proof/review to Stage B.
