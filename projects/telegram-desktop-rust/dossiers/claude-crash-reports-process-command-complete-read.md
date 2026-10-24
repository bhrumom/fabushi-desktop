# Claude crash-reports-process command complete read — strict index 61

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `61`.
- Source: `.claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5`, 269 lines, fully read.
- Dependency read for behavior decomposition only: `.claude/scripts/crash_triage.py@2db6cc14d8e529c4e61c5dcf169aade31fd35c58`, 1031 lines. This dependency remains its own future source entry and is not closed here.
- Parent Fabushi exact head: `5d5b7ad28d9ce001578a547fb0b6ccf14fb4bd2a`, tree `24db35bbb4e8fb92c21f80ad2a5f73cdcade2bcf`.
- Parent TDRP source-authority run `39706276648` attempt 1 completed success, 61/61 jobs; authority artifact `12395232437`, digest `sha256:64c55ca07b34561a8218161dbc8606b76a6d459a803087eefa3344f5c8dcd3a4`; recursive closure artifact `12395228247`, digest `sha256:6ab03035e82e5930511ead62416a59652b7e3fa0eb1ac62bed5b1e37c9953407`; recursive reachability artifact `12394844059`, digest `sha256:48917eb61dd96e4244b56b366bd79333e31db40b2e4cfa0f38f92732c1f8cc4e`.
- Parent Rust desktop runtime run `39706276680` completed success, 6/6 jobs; messaging artifact `12395237458`, digest `sha256:0da9c6e3436bd76170bcffde068378bccd9e4d0574989090d80a7754b351fa48`.
- Parent evidence becomes historical after this mapping commit; fresh child exact-head Actions remain mandatory.

## Complete responsibility decomposition
This source is a developer crash-corpus triage/fix command. It is not a shipping Telegram crash runtime and does not justify a second Fabushi telemetry, crash store, defect database, task queue or product Host/Runner owner.

### Intake, version and corpus ownership
The command binds the run to one target branch and one release version before investigation. Complete crash triples are claimed into a unique run folder while half-written triples remain pending. Old-version, no-frame and unsymbolized reports are moved aside by default rather than deleted. Explicit deletion is an opt-in destructive action. These mechanics preserve provenance, resumability and data-loss safety.

### Stable grouping and bounded work
Mechanical parsing/classification/grouping belongs to the helper, while the orchestrator consumes summaries. Equivalent crashes are clustered by stable version-independent signatures and prior ledger decisions are skipped. Fresh groups are ordered by report count and processed in waves of at most ten, with `--limit` leaving named remainder rather than silently dropping scope.

### Independent diagnosis and refutation
Each fresh crash group receives its own read-only investigation context. The investigator must bind source-line reasoning to the crashed release/tag, prove the object/state/lifetime path, distinguish known noise/already-fixed cases, and return exactly one of `not-actionable`, `needs-discussion`, or `fixed`. A merely plausible story is not fixed.

Every `fixed` proposal gets an independent adversarial verifier. Refutation includes correctness, invariant compatibility and common-path performance cost. A refuted proposal is demoted to `needs-discussion`. This maps directly to Fabushi's existing independent evidence/review lifecycle; it does not create a crash-specific review authority.

### Serial repair, durable regression record and reconciliation
Only the main writer applies confirmed small fixes, against current target-branch code, after rechecking healthy-path cost. Each selected group receives a durable verdict and optional fix SHA. Reconciliation after each wave and before finalization proves no selected group fell through between diagnosis, edit and record. Dry-run performs investigation only and owns no publication.

Fabushi already has the normative Defect & Regression Policy: defect records bind SHA/environment, steps, expected/actual, frequency, evidence, affected requirement/oracle, root cause, fix SHA and regression IDs; escaped defects require gap analysis and lower-layer regression coverage; closure is reproduce -> fix -> targeted regression -> affected regression -> current-head evidence. That owner subsumes the applicable responsibility.

### Verification adaptation
The upstream command's final local CMake build is source-specific. For FBCP/TDRP, executable build/test verification remains GitHub-Actions-only under root `AGENTS.md`. The existing adaptive evidence loop forbids rerun-until-green, classifies sound-check implementation failures separately from test flaws, preserves prior positive evidence, treats flake as non-passing, and classifies timeout/cancellation/crash by causality. Therefore strict 61 maps into the existing quality/evidence lifecycle rather than copying upstream build mechanics.

## Existing-owner-first disposition
Selected owner: Fabushi source-neutral defect/regression and verification-evidence lifecycle:
- `projects/fabushi-communication-platform/quality/defect-regression-policy.md`: `Record`, `Escaped defect`, `Closure/flaky`.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`: `Adaptive evidence loop and bounded recovery`.
- root `AGENTS.md`: `Adaptive verification and evidence recovery` and `Build and test execution environment`.

Existing shipping crash telemetry, Host crash marker, panic guard, unclean-exit/Sentry and platform recovery owners remain the product-side observation/recovery subjects; they are not duplicated by this developer workflow.

Rejected owners:
- a Telegram/Fabushi crash-triage shipping runtime;
- a second crash/defect ledger;
- shipping Task/Automation/Agent/Coordinator/Host/Runner as the repository triage scheduler;
- a local-build acceptance path for FBCP/TDRP.

No new owner or ADR is justified.

## Traceability
- Requirement/responsibility: `TDRP-R9-CLAUDE-CRASH-REPORTS-PROCESS-001`
- Oracles: `ORA-TDRP-CRASH-TRIAGE-IMMUTABLE-SOURCE-001`, `ORA-TDRP-CRASH-TRIAGE-INDEPENDENT-VERIFY-001`, `ORA-TDRP-CRASH-TRIAGE-RESUMABLE-LEDGER-001`
- Invariants: `INV-TDRP-CRASH-TRIAGE-NO-SECOND-RUNTIME-001`, `INV-TDRP-CRASH-TRIAGE-ONE-GROUP-PER-AGENT-001`, `INV-TDRP-CRASH-TRIAGE-NO-RERUN-UNTIL-GREEN-001`
- Contracts: `CONTRACT-TDRP-CRASH-TRIAGE-FUNNEL-001`, `CONTRACT-TDRP-CRASH-TRIAGE-WAVE-001`, `CONTRACT-TDRP-CRASH-TRIAGE-VERIFY-001`, `CONTRACT-TDRP-CRASH-TRIAGE-RECONCILE-001`, `CONTRACT-TDRP-CRASH-TRIAGE-BUILD-001`
- Regression: `REG-TDRP-CRASH-TRIAGE-NO-DROPPED-VERDICT-001`

Closing only strict index 61 changes counters to unknown=16,242, unread=16,217, omitted=0, responsibilities=79. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root entry: index 62 `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361`.
