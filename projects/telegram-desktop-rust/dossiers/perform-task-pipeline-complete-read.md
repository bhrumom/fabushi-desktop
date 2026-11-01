# Complete read — fixed-U perform-task end-to-end pipeline

## Authority snapshot

- Desktop checkpoint parent: `289f1f4d22d51f6874bfe8ca772fed287ca474f2`.
- Desktop main merge-base/current authority read in this round: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core checkpoint: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Recursive tree: `truncated=false`, 7,194 entries.
- Source: `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`, recursive-tree index 73.
- Exact blob read completely: 64,933 Git bytes / 64,829 decoded characters / 1,145 newline-delimited lines.

## Complete contract

This source is the authoritative end-to-end runner contract beneath the existing single-task performer. It begins by resolving one selected task, source/spec/project context, task-local work paths, recovery refs, host kind and available build/evidence instruments. Before Phase 1 it reads the applicable repository/task instructions, verifies canonical in-progress ownership, gates approved prerequisites through source lineage, safely prepares fresh source/submodules only when there is no carried implementation, records a read-only preflight report, initializes/resolves task recovery refs, and refuses dirty paths outside the proven task-owned set. It expressly forbids stash/reset/restore/stage/commit/delete of unexpected paths.

Task-local context, plan, visual contract when applicable, split proposal, review artifacts, test design/report, result, owned-path set, prompt/result logs and compact evidence are resumable state, not canonical ownership. A resumed performer reads those artifacts and restarts at the first incomplete validated boundary; completed work is not repeated merely because the prior agent disappeared. Checkpoints are task-scoped and do not publish canonical state.

The performer is the only stateful task owner. Phase workers do not delegate or commit. Assessment independently validates files/APIs, approach, repository precedent, evidence design and intrinsic scope before source edits. It may approve bounded phases, reject a fast path or over-engineered approach for a fresh plan, or stop as split-required. Implementation is limited to explicit owned write sets. Selected pre-review validation is the direct assessed check, followed by independent general plus standard-lens review (with the explicit harness narrowing), focused re-review carrying unaffected approvals, and convergence dispositions `CONTINUE | REPLAN_CURRENT | RESCOPE_REQUIRED | HARD_STOP`.

Every task enters the same claim-driven evidence loop. A selected check states changed surface, instrument, oracle, window, control, falsifier and durable output. Static/artifact, unit/probe/component, Telegram runtime and interaction/visual instruments are selected by directness rather than ceremony. The result of one check/campaign is exactly `APPROVED`, `TEST_FLAW`, `IMPL_BUG` or `UNRECOVERABLE`. Test-flaw recovery removes a failed assumption or moves closer to the changed surface while carrying prior valid proof; implementation bugs return to a retained implementation attempt plus focused review. Campaign caps are assessment checkpoints, never approval or Block. The already-satisfied path requires direct proof and no manufactured source edit.

The upstream commit helper, private mirror, portable Telegram account, overlay, executable cleanup and platform-specific build commands are source workflow mechanics. Their portable responsibility is commit-boundary ownership, exact task provenance, safe recovery, direct evidence and final publication; they do not justify a Telegram-specific performer or Fabushi shipping runtime owner.

Final state is explicit. The result schema records `STATUS`, outcome, verdict, blocker type, implementation retention, split proposal, attempts, test runs, UI-driver disposition, touched paths, test report, evidence, exact `Unverified` behavior, checkout condition and discovered follow-ups. Approval requires complete positive evidence; genuine Block has its recovery/exhaustion proof; split-required preserves retained work for the scheduler. Final publication occurs only through canonical non-force repository state publication after the required source boundary is safe; semantic conflict or remote outage preserves recoverable state instead of pretending completion.

Failure handling preserves the same ownership model: pre-Phase-1 lineage mismatch is a clean routing stop; a mismatch discovered later preserves/reports owned state rather than integrating branches inside the performer; disposable worker retry is bounded; a blocked attempt remains unfinished; a campaign cap is not itself Block; exact-path file-lock recovery never kills unrelated processes; locked macOS does not reduce deterministic in-process evidence; missing optional art does not block; and every unverified shipped behavior is surfaced precisely.

## Direct fixed-U callers, consumers and dependencies

- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` declares `references/pipeline.md` the authoritative end-to-end runner and instructs the performer to execute it.
- `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` imports all pipeline responsibilities except its explicitly narrowed code-health review/evidence exceptions.
- `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c` routes selected work to perform-task and relies on pipeline rules for `Unverified` and split-required outcomes.
- Shared dependencies remain `.agents/shared/project-context.md@53c8657eb7949a5a2fba7e7adc8d1073f3079ead`, `.agents/shared/phase-effort.md@8d7b385dbb526e87fc090774ee35191e1619bb08`, `.agents/shared/codex-delegation.md@ed89d2a14344f50034a9363bd9ffe75f7972e652`, `.agents/shared/test-loop.md@bb51685e417759939cadc1e2e74b1c0199fcb8f0`, `references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`, and the existing process-inbox workspace helper. These compose the performer; none becomes a second owner.

## Existing-owner resolution

Existing-owner-first keeps `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001`: Fabushi repository task/process authority plus existing Test Strategy/Test Evidence governance. The current owner already owns one-task state, source-lineage, resumable phases, owned paths, independent/focused review, evidence recovery and non-force publication. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core, a Telegram-specific performer/worker pool and a second review/evidence store are rejected duplicate/different domains.

No new responsibility, ADR, shipping runtime owner or platform-core production delta is required.

## Stage-A delta

Spec-first source-level governance extends:
- root `AGENTS.md` with the fixed-U end-to-end pipeline composition boundary;
- the existing Test Strategy with preflight/resume, evidence-adapter, final-publication and recovery qualification;
- the existing evidence contract with exact pipeline preflight/resume/end-to-end/final-publication evidence;
- this dossier with the complete source/consumer mapping.

Authored/not-run:
- `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-PREFLIGHT-001`
- `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-RESUME-001`
- `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-EVIDENCE-ADAPTER-001`
- `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-FINAL-PUBLICATION-001`
- `FAULT-TDRP-PERFORM-TASK-PIPELINE-RECOVERY-001`
- `INT-TDRP-PERFORM-TASK-PIPELINE-END-TO-END-001`

Existing performer umbrella contracts remain authoritative. Nothing is executed in Stage A.

## Source-closure disposition

Exactly this independent fixed-U source entry is ready to close after the spec-first checkpoint is published: `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`. No broader fixed-U tree closure or Stage-B validation is implied. The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`.
