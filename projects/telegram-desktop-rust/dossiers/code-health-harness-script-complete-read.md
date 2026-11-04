# Code-health harness.py complete read

## Source authority

- Repository: `telegramdesktop/tdesktop`
- Fixed commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries / 6,859 blobs)
- Path: `.agents/skills/code-health/scripts/harness.py`
- Blob: `652a9ee7ed6a690705cad45aeff06568210db6b1`

## Exact responsibility

The script is the executable helper-inventory/consumer-evidence instrument behind the separately owned retention policy. It discovers `Telegram/SourceFiles/test/test_*.(h|cpp|mm)` modules and helper docs from the exact scanned tree; derives current-path birth/task provenance; classifies API/internal/self-test helpers; strips comments/literals before identifier evidence; suppresses ambiguous symbol ownership; finds direct source callers, helper-to-helper dependents and declarative workflow references; and reads canonical archived task state conservatively.

Archive evidence records queue identity/stability, approved tasks, saved/missing patches, missing dates, unknown task classification and read failures. Saved final C++ patch context is a lower bound, not execution telemetry. Uses are de-duplicated by task and separated into independent product, uncertain, harness-only, self-test-only and creation-task observations. Internal dependency closure makes a dependent helper's archived use indirect evidence for what it requires. Shallow source history suppresses age; unavailable archive coverage suppresses product-consumer claims rather than manufacturing zero use.

`inspect` combines those facts with source lines, documentation bytes/index gaps, open tasks, workflow references, direct callers, internal dependents and downstream review-reason fields. `markdown` renders the same coverage limitations for human review.

## Direct callers/consumers

- `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` directly calls `harness.inspect`, aggregates helper totals and converts retention/docs signals into decision findings.
- `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` writes `harness.json` / `harness.md` and keeps retention findings unverified when coverage is unavailable.
- `harness_test.py@5d6b2537c9289cb3e1db0904b9f8e1b99dec4b74` covers inventory/coverage/consumer semantics.
- `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` covers orchestration persistence and exact scanned commit.
- `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` schedules inventory consumption; `harness-lifecycle.md@f18c847326d2cfbdc8bf168e804ecbd5343ee61c` remains the separate decision-policy owner.

## Existing-owner-first

The applicable owner is the existing Fabushi repository quality/test-evidence governance in Test Strategy and Evidence Contract. No platform-core production delta, Telegram-specific Python scanner, second findings ledger, helper store, task queue or new ADR is required.

## Traceability

- Requirement: `TDRP-R9-CODE-HEALTH-HARNESS-INVENTORY-INSTRUMENT-001`
- Oracles: `ORA-TDRP-CODE-HEALTH-HARNESS-INVENTORY-REVISION-001`, `ORA-TDRP-CODE-HEALTH-HARNESS-INVENTORY-CONSUMERS-001`, `ORA-TDRP-CODE-HEALTH-HARNESS-INVENTORY-COVERAGE-001`
- Contracts authored, not run: `CONTRACT-TDRP-CODE-HEALTH-HARNESS-INVENTORY-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-ARCHIVE-COVERAGE-001`, `CONTRACT-TDRP-CODE-HEALTH-HARNESS-CONSUMER-CLASSIFICATION-001`, `INT-TDRP-CODE-HEALTH-HARNESS-REPORT-CONSUMPTION-001`
- Fault controls authored, not run: `FAULT-TDRP-CODE-HEALTH-HARNESS-SHALLOW-HISTORY-001`, `FAULT-TDRP-CODE-HEALTH-HARNESS-DIRTY-QUEUE-001`, `FAULT-TDRP-CODE-HEALTH-HARNESS-MISSING-PATCH-001`

Stage A disposition: `implemented / validation_status=not-run / accepted=false`.
