# Code-health history.py complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Commit: `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`
- Path: `.agents/skills/code-health/scripts/history.py`
- Blob: `7c12f5b091936723c6ad005dda4bb8e2b1e549df`
- Recursive authority position: array index 29, immediately after `health_test.py` at index 28.
- File history at the accepted authority contains one introducing commit: `0d6570aeb5794b82b071a004cd4745142a21a048` ("[ai] Add a code-health skill measuring and checking code").

## Symbols and behavior

`churn(tree, days)` counts Git commits touching each scanned path in the selected time window. It reads the root repository at the exact scanned commit and each pinned submodule at its recorded commit, prefixes submodule paths back into the unified source tree, ignores unavailable submodule history rather than inventing observations, and finally filters counts to `tree.files`.

`collect(...)` is the Git-log counter used by `churn`: it requests commit separators plus changed paths with rename inference disabled, then increments one count for each reported path occurrence.

`task_files(tree, days)` reads root-repository commits in the window and maps immutable `Task:` trailers to the Telegram source paths changed by those commits.

`agent_cost(tree, queue, days)` is optional repository-development evidence. Missing queue/task state or absence of task-tagged commits yields no measurements. It reads queue history for `Start <task>` and `Approve <task>` lifecycle commits, retains earliest start and latest approval, rejects incomplete or non-positive intervals, records wall-clock hours, derives the highest `reviewN-*` round present under the task work directory, and binds the result to the task's changed source paths and approval date.

`summarize(values)` emits count, median and total for a non-empty measurement set and `None` for no evidence.

## Real callers and responsibility boundary

- `.agents/skills/code-health/scripts/report.py@a1177613670e933694f545f387ca7e7ed8cf0ee0` is the direct code-health caller. Its snapshot composes `history.churn(...)` and optional `history.agent_cost(...)` with file size, include/build cost, clone, unused, tested-file and atlas facts.
- `.agents/skills/code-health/SKILL.md@6885938b3d186b55f5d69cb8b9b9a96bbca0e097` declares churn/hotspots and agent cost (Start-to-Approve hours plus review rounds) as scan measurements, and uses hotspot multiplied by median task hours only to prioritize a survey.
- The measurements do not own task lifecycle, review approval, source authority, atlas ownership, product behavior, release acceptance or a second queue. Missing history is missing evidence, not zero-cost or proof of health.

## Existing-owner decision

Applicable responsibility: source-neutral repository-maintenance history and task-effort measurement for quality reporting/prioritization.

Current Fabushi already owns this responsibility in repository quality/test-evidence governance. The Test Strategy already treats churn, task cost and hotspot signals as investigation/prioritization inputs rather than ownership or acceptance. The Evidence Contract already requires raw churn/task-duration measurements to remain distinguishable from human responsibility judgments. This closure strengthens those existing contracts with exact-revision, incomplete-history, lifecycle-interval and review-round semantics; it does not port Telegram's Python helper or introduce a second task/history owner.

## Failure and lifecycle boundaries

Historical observations remain bound to the exact source revision and measurement window. Root/submodule histories are not silently mixed across revisions. An unavailable submodule history, missing queue, absent Task trailer, missing Start/Approve endpoint, non-positive interval or unrecognized review artifact cannot be fabricated into a successful cost observation. Rename-disabled churn is an explicit measurement convention, not semantic file identity. Churn, hotspot, elapsed hours and review rounds are prioritization evidence only and cannot by themselves approve an atlas/map, route a task, establish product correctness, prove a build, or satisfy release acceptance.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-HISTORY-MEASUREMENT-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-HISTORY-REVISION-001`, `ORA-TDRP-CODE-HEALTH-TASK-COST-001`, `ORA-TDRP-CODE-HEALTH-HISTORY-ABSENCE-001`
- Invariants: `INV-TDRP-CODE-HEALTH-HISTORY-EXACT-REVISION-001`, `INV-TDRP-CODE-HEALTH-HISTORY-NO-FABRICATED-ZERO-001`, `INV-TDRP-CODE-HEALTH-HISTORY-NO-SECOND-OWNER-001`

These identifiers bind only this source-entry responsibility and its repository-quality evidence boundary; they do not claim product/runtime or release acceptance.
