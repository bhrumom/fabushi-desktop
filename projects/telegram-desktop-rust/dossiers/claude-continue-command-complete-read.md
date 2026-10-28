# Claude continue command complete read — strict index 60

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `60`.
- Source: `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8`, 68 lines / 3,700 bytes, fully read.
- Same-upstream semantic owner fully read: `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`, 722 lines / 39,726 bytes.
- Same-upstream host adapter revalidated: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`, 216 lines / 13,188 bytes.
- Parent Fabushi exact head: `227e16a89b602bee9874eff993f767f9cb471f84`, tree `4f2c23b8c115fa77545105f717f1fbdf38c07e92`.
- Parent TDRP source-authority run `39704746797` completed success, 61/61 jobs; authority artifact `12394586968`, digest `sha256:20344001bdba23c547af8a6cd5b3c6b49f845405d76f6c44f717e15f32d95cc5`; recursive reachability artifact `12394547713`, digest `sha256:1fad31fe06f322d97bcb3f5563bcf5ca24b450d36aea1dbf59ad65a60d51427c`; recursive closure artifact `12394377730`, digest `sha256:f45a6c3ec282e8543082f98f4f63cbb6fd65136903a229de492b9dd8e795c485`.
- Parent Rust desktop runtime run `39704746955` completed success, 6/6 jobs; messaging artifact `12395030136`, digest `sha256:8aefb21bdea01817bfbc4c50bae53d82f75dcd2afc2c3bb6eb2d83f9569f4e11`.
- Parent evidence is historical-only after this mapping commit; fresh child exact-head Actions remain mandatory.

## Complete responsibility decomposition
This file is a Claude host command for continuing a selected repository-development batch. It is not a Fabushi product runtime, end-user task system, second repository queue, or independent scheduler. The shared `.agents/skills/continue/SKILL.md` remains the semantic authority and the Claude adapter supplies only host mechanics.

### Batch admission and freeze
Without a batch specification the command must stop after the shared read-only queue call, report the current checkout state and ask for a specification; it must not mutate queue/source state. With a specification it chooses exactly one shared mode and freezes only the selected startup task ids, with the checkout's exclusive active task first. An inbox request permits one inbox transaction. “All pending tasks” freezes the pending startup snapshot. Ordinary tasks observed later do not silently join.

Only follow-ups causally routed from results produced by this invocation may extend the frozen batch, including transitive follow-ups. Source-project inheritance remains the default and detachment requires the shared affirmative independence test.

### Stateful execution, split and lineage
The shared scheduler owns task state. One stateful performer runs at a time. A split-required result is handled by one fresh split worker; ordered replacements replace the source id, and a retained implementation carrier is started first so sealed state transfers without reset/checkpoint loss.

Source-lineage decisions remain fail-closed. Before freeze, a missing approved prerequisite is a human routing boundary after the exact branch/compatible-branch report. After freeze but before Phase 1, only the parent may perform the shared safe switch to an already-existing compatible local branch after the required cleanliness/recovery checks. A mismatch first proven after Phase 1 becomes a task-local Block and unrelated independent batch work may continue. Branch integration is never manufactured as a queue task.

### Routing and consolidation
Discovered follow-ups publish through the canonical routing worktree and only causally created task ids append to this invocation. Pending consolidation is recovered or run in a fresh dedicated worker through the shared helper publication contract, with at most one consolidation lane in flight. A normal stop is not valid while required routing/consolidation publication is still unlanded.

### Provider and host mechanics
Every worker is told to load the Claude adapter before its shared skill/reference. Workers use the Claude Agent surface rather than nested Claude subprocesses. Review provider is selected once at startup: native only for an explicit Codex opt-out, otherwise Codex. This provider choice does not alter task selection, lineage, publication, evidence or completion semantics.

## Existing-owner-first disposition
Selected owner: Fabushi root `AGENTS.md` source-neutral repository AI development/review/evidence lifecycle. Existing exact-head contracts already require live repository authority rather than chat memory, selective task-lineage context, scope preservation, immutable worker/run identity, sanctioned completion waits, and separate artifact/effect validation.

Rejected owners:
- shipping Agent/Coordinator/Host/Runner: product execution, not repository-development scheduling;
- shipping Task/Automation/Workflow: end-user work, not developer queue batches;
- a Claude-specific scheduler/queue/batch store: duplicates the shared repository lifecycle;
- a second source-lineage/split/consolidation owner: creates competing durable task state.

No new owner or ADR is justified. The host command maps into the existing source-neutral lifecycle.

## Traceability
- Requirement/responsibility: `TDRP-R9-CLAUDE-CONTINUE-COMMAND-001`
- Oracles: `ORA-TDRP-CLAUDE-CONTINUE-FROZEN-BATCH-001`, `ORA-TDRP-CLAUDE-CONTINUE-SOURCE-LINEAGE-001`, `ORA-TDRP-CLAUDE-CONTINUE-SPLIT-CONSOLIDATION-001`
- Invariants: `INV-TDRP-CLAUDE-CONTINUE-NO-UNRELATED-DRAIN-001`, `INV-TDRP-CLAUDE-CONTINUE-NO-SECOND-SCHEDULER-001`, `INV-TDRP-CLAUDE-CONTINUE-SINGLE-PERFORMER-001`
- Contracts: `CONTRACT-TDRP-CLAUDE-CONTINUE-NO-SPEC-001`, `CONTRACT-TDRP-CLAUDE-CONTINUE-FROZEN-BATCH-001`, `CONTRACT-TDRP-CLAUDE-CONTINUE-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CLAUDE-CONTINUE-SPLIT-001`, `CONTRACT-TDRP-CLAUDE-CONTINUE-CONSOLIDATION-001`
- Regression: `REG-TDRP-CLAUDE-CONTINUE-NO-UNRELATED-TASK-001`

Closing only strict index 60 changes counters to unknown=16,243, unread=16,218, omitted=0, responsibilities=78. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root entry: index 61 `.claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5`.
