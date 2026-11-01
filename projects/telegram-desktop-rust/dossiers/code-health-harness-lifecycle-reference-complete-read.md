# Code-health harness-lifecycle reference complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Fixed commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`
- Path: `.agents/skills/code-health/references/harness-lifecycle.md`
- Blob: `f18c847326d2cfbdc8bf168e804ecbd5343ee61c`
- Fixed-U recursive tree was read with `truncated=false`: 7,194 entries total and 6,859 blobs.
- Queue selection compared recursive blob order with all independently represented parity-ledger `source_path@source_blob_sha` and `source_paths[]` anchors. This is the first independently unrepresented source entry at the round-106 parent.

## Exact behavior

The reference requires helper-lifecycle evidence to come from the same freshly fetched source commit as the parent quality scan. Helper age and sparse archived use are decision signals, not proof of dead code or measured build/runtime cost.

Daily inventory carries stable `harness-retention:test_<name>` findings for reusable APIs at least 90 days old when no independently observed consumer exists in the preceding 90 days or when at most two observed lifetime consumers remain. Unknown consumers count conservatively against an absence-of-use conclusion, and known open consumers hold routing. `harness-docs:Telegram/SourceFiles/test/README.md` is a decision finding when the mandatory guide reaches 8 KiB, is missing, or indexed module references are absent.

The inventory retains current-path age, creation provenance, helper kind, source/reference weight, consumer task IDs and dates, last observed use, direct callers, internal dependents, workflow references and open work. Archived task patches are a lower bound. Missing patches, unclassified consumers, missing dates, read failures, shallow source history and unavailable/dirty canonical task state remain unknown evidence rather than zero use. Self-test/internal helpers remain separately inventoried.

Newly eligible or materially changed candidates are reviewed at most once per UTC calendar week against canonical state. Before retirement is proposed, the actual helper role, callers, internal dependencies, registration/runner entry points, build/workflow references, archived consumers, open work and pending local replacement must be checked. A decision may retain, defer, reduce the reference/API, or propose retirement through the ordinary quality-task lifecycle. Age/count alone never authorizes deletion or task routing.

## Direct callers and consumers

- `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` schedules the daily inventory read and at-most-weekly retention review and defines exact-ref quality state.
- `.agents/skills/code-health/scripts/harness.py@652a9ee7ed6a690705cad45aeff06568210db6b1` computes archive coverage, direct/internal consumers, the 90-day/lifetime triggers and documentation facts.
- `.agents/skills/code-health/scripts/health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` composes the harness snapshot into the exact-revision scan, writes `harness.json` / `harness.md`, and carries unavailable retention coverage into the findings lifecycle.
- `.agents/skills/code-health/scripts/report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` classifies `harness-retention` and `harness-docs` as decision findings and reports helper lifecycle coverage.

Those caller/consumer blobs were read to understand this entry. They remain independent source inventory entries unless separately closed; this dossier claims only `.agents/skills/code-health/references/harness-lifecycle.md@f18c847326d2cfbdc8bf168e804ecbd5343ee61c`.

## Existing-owner decision

The current Fabushi checkpoint already owns repository maintainability through root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, and `projects/fabushi-communication-platform/quality/evidence-contract.md`. This existing quality-health owner already owns exact-revision canonical state, stable finding identity/history, fail-closed missing authority, bounded routing and ordinary task proof.

This round extends that same owner with the source-neutral helper-retention boundary: conservative coverage; review-only 90-day/lifetime thresholds; open-consumer routing hold; weekly duplicate-review fencing; pre-retirement caller/dependency/build/workflow/open-work review; documentation decision signals; and an explicit no-auto-delete rule. No upstream Python scanner, helper store, second findings ledger, second task queue, product runtime or platform-core owner is introduced.

## Stage-A verification disposition

The qualification/evidence contracts were authored but not executed. No build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation, workflow/status query or runtime acceptance was run. State is `implemented / validation_status=not-run / accepted=false`.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-HELPER-RETENTION-LIFECYCLE-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-HELPER-RETENTION-THRESHOLD-001`, `ORA-TDRP-CODE-HEALTH-HELPER-RETENTION-COVERAGE-001`, `ORA-TDRP-CODE-HEALTH-HELPER-RETENTION-WEEKLY-001`
- Invariants: `INV-TDRP-CODE-HEALTH-HELPER-RETENTION-EXACT-REVISION-001`, `INV-TDRP-CODE-HEALTH-HELPER-RETENTION-UNKNOWN-NOT-ZERO-001`, `INV-TDRP-CODE-HEALTH-HELPER-RETENTION-NO-AUTO-DELETE-001`, `INV-TDRP-CODE-HEALTH-HELPER-RETENTION-NO-SECOND-OWNER-001`
- Contracts: `CONTRACT-TDRP-CODE-HEALTH-HELPER-RETENTION-THRESHOLDS-001`, `CONTRACT-TDRP-CODE-HEALTH-HELPER-RETENTION-COVERAGE-001`, `CONTRACT-TDRP-CODE-HEALTH-HELPER-RETENTION-WEEKLY-001`, `CONTRACT-TDRP-CODE-HEALTH-HELPER-RETENTION-NO-AUTO-DELETE-001`
- Coverage after this independent source-entry closure: `source_entries_total=16202 / unknown=16171 / unread=16144 / omitted=0`.
