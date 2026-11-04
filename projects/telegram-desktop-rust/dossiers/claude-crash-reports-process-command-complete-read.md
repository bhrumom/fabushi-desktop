# Fixed-U complete read — crash-report process command

Source: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4:.claude/commands/crash-reports-process.md
Blob: 5ac555406670721694e78c4e1e6b2a96b3e98fb5
Tree: 91f2b083ab2877c0a17d07868eabec362a5cfca6; index 90 zero-based; 13,431 UTF-8 bytes; 13,379 decoded characters; 268 newlines; trailing newline.
Direct helper: .claude/scripts/crash_triage.py@2db6cc14d8e529c4e61c5dcf169aade31fd35c58 (38,738 bytes; 1,030 newlines).
Stage: A; validation not run; accepted false.
Desktop parent: e35d300f6e915d88bf7788cf6851bab530a4518a
Platform-core: e92b51e23b39dae5719f033fa1f7a20d6579dc8c

## Semantics

The command separates mechanical corpus handling from crash diagnosis. Complete report triples are claimed into a per-run folder; incomplete triples remain for a later run. Interrupted claims resume from the processing marker. Old-version, no-frame and unsymbolized reports are moved aside by default rather than deleted. Stable signatures allow prior verdicts to be reused across versions.

Fresh groups are ordered by frequency, processed in waves of at most ten, and limited when requested. Each non-noise group receives an isolated investigation; proposed fixes receive an independent adversarial verification. Investigation is read-only on tracked source. Valid verdicts are not-actionable, needs-discussion, or fixed. A fixed verdict requires a traced path, a small safe change, and no common-path performance cost; a refuted proposal becomes needs-discussion.

Confirmed fixes are applied serially to current target-branch source. Every selected group is recorded after its wave and reconciled against the ledger before the next wave and final report. Dry-run excludes durable mutation. The final build/report step is executable qualification and is deferred to Stage B.

## Ownership

Responsibility: TDRP-R9-CRASH-REPORT-PROCESS-CLAUDE-COMMAND-ADAPTER-001.
Existing owners: repository development process governance; repository quality/test-evidence governance; root host-command authority.
Rejected owners: shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, shipping crash telemetry, and any duplicate crash queue/evidence/publication store.
No platform-core production delta, service, ADR, or shipping state owner is introduced.

Direct host consumer is Claude Code command discovery/user /crash-reports-process invocation. Fixed-U search found no file-level import of the command path. The generic .claude/ai-workflow-adapter.md is not a dependency because this command does not explicitly load it and that adapter is explicit-load-only.

## Authored, not run

CONTRACT-TDRP-CRASH-REPORT-CLAUDE-HOST-AUTHORITY-001
CONTRACT-TDRP-CRASH-REPORT-INVOCATION-001
CONTRACT-TDRP-CRASH-REPORT-CLAIM-RECOVERY-001
CONTRACT-TDRP-CRASH-REPORT-DELEGATION-001
CONTRACT-TDRP-CRASH-REPORT-VERDICT-ROUTING-001
CONTRACT-TDRP-CRASH-REPORT-RECORD-RECONCILE-001
CONTRACT-TDRP-CRASH-REPORT-BUILD-REPORT-BOUNDARY-001
INT-TDRP-CRASH-REPORT-PROCESS-COMPOSITION-001
FAULT-TDRP-CRASH-REPORT-INTERRUPTED-CLAIM-001
FAULT-TDRP-CRASH-REPORT-DROPPED-VERDICT-001
