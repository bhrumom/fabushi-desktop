# Complete read — fixed-U perform-task phase prompts

## Authority snapshot

- Desktop checkpoint parent: `f46416dd12f12839e5221217b3c8f2ec2b4e5909`.
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core checkpoint: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed upstream U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Recursive tree: `truncated=false`, 7,194 entries.
- Source: `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c`, recursive-tree index 72.
- Exact blob read completely: 64,311 bytes / 64,245 decoded characters / 1,215 newline-delimited lines.

## Complete contract

This source is the exact phase-worker prompt/completion protocol beneath the canonical single-task performer. It does not own canonical task state or queue routing. The performer writes each phase prompt before execution, chooses phase effort through the shared policy, gives each delegated worker a unique leaf name, and writes a result record with STATUS/ARTIFACTS/TOUCHED/BLOCKER/NOTES after validating the required artifacts. Leaf workers do not delegate and do not commit; their BLOCKED status is a report to the performer, not authority to publish a task Block.

Delegation is selected only after the first real phase spawn succeeds. If delegation is unavailable immediately because of depth/capacity/policy, the source-defined same-session checklist path is selected rather than counted as a delegated retry. Once delegated, an incomplete Phase 1, Phase 3, Phase 4 or Phase 6 leaf is retried at most once in a fresh worker with more specific instructions after the original worker is confirmed stopped. If that retry still fails, the performer returns a recoverable hard stop to its caller rather than replaying locally. Grok depth-one `/continue` uses the same-session fallback by design; Claude uses synchronous foreground Agent calls or the declared Codex-leaf provider; Codex uses native subagents and the shared wait/recovery contract.

Phase 1 writes task-specific context plus a bounded plan and, for visual work, the derived visual contract. Phase 3 independently verifies correctness, completeness, repository precedent, containment, evidence design, phase sizing and intrinsic scope. It may accept/refine, reject an undersized fast path, reject an over-coupled approach for a fresh Phase 1 rerun, or return `split-required` with a proposal and no source edit. Phase 4 implements only one assessed phase inside its owned write set. Phase 5 uses the assessed direct pre-review validation rather than broad validation by ceremony.

Initial Phase 6 review uses the mandatory general reviewer plus the standard independent lifetime/reuse/structure/performance/security lenses, except the explicit harness narrowing. Initial reviewers do not see each other's findings. After a blocking fix, focused re-review verifies the repaired boundary, carries unaffected approvals, reruns only invalidated specialists and reconciles only invalidated evidence. Repeated/cycling findings are diagnosed by a fresh convergence assessor with exactly one `CONTINUE | REPLAN_CURRENT | RESCOPE_REQUIRED | HARD_STOP` disposition; a convergence assessment cannot approve code and a round count never resolves findings.

Evidence-flaw recovery preserves retained implementation unless a sound check proves `IMPL_BUG`. Before changing an evidence check it records prior proof, the failed assumption, forbidden repeated technique, the new direct instrument, reachability and oracle independence. Recovery moves toward the changed surface, keeps prior passing checks unless invalidated and never reimplements product behavior in the test. Campaign-cap assessment preserves prior proof and unmet checks; only one focused recovery campaign may follow the normal cap, while an exhausted or non-converging focused campaign records the exact exhausted/human boundary rather than manufacturing approval or Block.

Native-Windows normalization is a current-session final phase over the exact touched textual file set only; prompt delivery/logging is canonical across hosts. Completion/error handling preserves phase artifacts, bounded retries and resumable performer state. The fixed-U source's ordinary build/test/review commands are executable workflow behavior, but FMEP-001 Stage A defers all such execution for this Fabushi migration.

## Direct fixed-U callers and consumers

- `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` declares this file the exact leaf-prompt and retry-rules authority.
- `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` delegates context/plan, assessment, implementation, build, review, test-design and Windows-normalization prompt semantics to this file.
- `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154` requires a depth-one Grok child to execute every phase leaf as a same-session checklist from this file.
- `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d` imports the specialist lens definitions from this file for cross-review without importing performer task state.

The prompt protocol itself composes the shared phase-effort, Codex delegation/CLI-leaf, test-loop and build-lock contracts plus Claude/Grok host adapters; those remain peer dependencies rather than new product owners.

## Existing-owner resolution

Existing-owner-first keeps `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001`: Fabushi repository development process authority plus existing Test Strategy/Test Evidence governance. The current AGENTS performer section already owns canonical single-task state, Start/Retry/Resume, resumable phase boundaries, focused review/convergence, evidence recovery and non-force publication. Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Telegram-specific performer/worker pool and a second review/evidence store are rejected duplicate/different domains.

No new responsibility, ADR, shipping runtime owner or platform-core production delta is required.

## Stage-A delta

Spec-first source-level governance extends:
- `AGENTS.md` with the fixed-U phase-prompt/worker contract;
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` with phase-prompt qualification;
- `projects/fabushi-communication-platform/quality/evidence-contract.md` with durable phase-prompt evidence;
- this dossier with the complete source/consumer mapping.

Authored/not-run:
- `CONTRACT-TDRP-PERFORM-TASK-PHASE-PROMPT-ARTIFACT-001`
- `CONTRACT-TDRP-PERFORM-TASK-PHASE-ASSESSMENT-001`
- `CONTRACT-TDRP-PERFORM-TASK-PHASE-REVIEW-FOCUS-001`
- `FAULT-TDRP-PERFORM-TASK-PHASE-WORKER-RECOVERY-001`
- `FAULT-TDRP-PERFORM-TASK-EVIDENCE-CAMPAIGN-001`

Existing umbrella `CONTRACT-TDRP-PERFORM-TASK-PHASE-RESUME-001`, `CONTRACT-TDRP-PERFORM-TASK-REVIEW-CONVERGENCE-001` and `CONTRACT-TDRP-PERFORM-TASK-EVIDENCE-RECOVERY-001` remain authoritative. Nothing is executed in Stage A.

## Source-closure disposition

Exactly this independent fixed-U source entry is ready to close after the spec-first checkpoint is published: `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c`. No broader fixed-U tree closure or Stage-B validation is implied.
