# Claude perform-task command complete read — strict index 64

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `64`.
- Source: `.claude/commands/perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e`, 42 lines / 2068 characters. The complete source was already read in the uninterrupted pre-gate audit and its SHA was revalidated immediately before this write.
- Direct dependencies previously fully read: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` and `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.
- Transitive dependency closure previously reconciled with no drift: `.agents/shared/phase-effort.md@dd07a4c611922a29a6892332db98afe76cd58952`, `.agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8`, `.agents/shared/codex-delegation.md@ed89d2a14344f50034a9363bd9ffe75f7972e652`, `.agents/shared/test-loop.md@83c4732bf6139f7c6557f18dd87bb35c3d68808d`, `.agents/shared/build-lock-recovery.md@ec75573e7efe74e9c9a344f3e66f2fc43ac7680f`, `.agents/skills/perform-task/references/pipeline.md@46e158ac86ebce97f9dcd010a98b3c581ffc114c`, `.agents/skills/perform-task/references/phase-prompts.md@9e9eac194ffbcf9dd7125876e71dad651a7cabba`, `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`, and `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`.
- Parent Fabushi exact head: `f2d36ac256b9a3b0bedceabce279876940d0b475`, tree `91420da50d8b29f82ca2b2266d9976a170e37283`.
- Parent TDRP run `39718798793` attempt 1 completed success, 61/61 jobs; authority artifact `12400835344`, digest `sha256:dc6aeaa9a0adb7fad5cd0513c6529ab71816f7d9dff325460e36eadbbc6c55ea`; recursive closure artifact `12399754162`, digest `sha256:0addb81be32bbf1e745980ff5da5378f840cd92abba568370e20f7d3b61ca7a9`; recursive reachability artifact `12400271928`, digest `sha256:bb109499178706cc6c4d2cccec540043b6316e7cb9654594e6ff0fbc06561139`.
- Parent Rust desktop runtime run `39718798809` attempt 1 completed success, 6/6 jobs; messaging artifact `12399638673`, digest `sha256:63fca65f3b2f5ca5abcd7b61f7f351d295683c985f4c111f6b7d6b8b35e7a9b0`.
- Parent Official MCP marketplace run `39718798807` completed success.
- Parent evidence becomes historical after this mapping commit; fresh child exact-head Actions remain mandatory.

## Complete responsibility decomposition
The source is a thin Claude host invocation wrapper around the already-audited shared `perform-task` skill and repository development lifecycle. It is not a shipping Agent, Task, Automation, Coordinator, Host or Runner feature and it does not own a new repository task queue.

### One-task selection and authority
The command executes one existing task under current repository/task authority. It inherits exact source-lineage checks rather than trusting chat memory or provider session state. A task that must be split stays split; the wrapper cannot erase task boundaries to make execution convenient.

### Shared lifecycle, not a second engine
Planning, implementation, adaptive verification, independent review, evidence recovery, build-lock recovery, computer-use verification, mechanical-lane qualification and provider leaf delegation all resolve through the already-owned shared skill/reference contracts. The command therefore contributes invocation semantics only. It does not duplicate those dependencies into a Claude-specific workflow engine.

### Commit and stopping boundary
The wrapper preserves the existing conditional `[ai]` commit policy instead of committing merely because the provider command returned. When the selected task reaches its terminal result, the command stops. It does not independently continue the queue, choose another task, route/consolidate findings, or create a scheduler.

## Existing-owner-first disposition
Selected existing owner: Fabushi root `AGENTS.md@4f888d465d29d962542da07bcb6f6efe2b5be897`, source-neutral repository native invocation plus the mandatory task/review/evidence lifecycle. The exact unique target sentence is:

`They must resolve to an existing source-neutral repository workflow owner and preserve that owner's exact-authority, evidence, trust, and routing rules; provider syntax such as $`skill` or slash commands is an adapter detail, not a second scanner, task queue, product runtime, or semantic owner.`

The source's Claude/Codex/Agent spelling and provider mechanics are provenance only. Rejected owners are shipping Agent/Coordinator/Host/Runner, shipping Task/Automation, a Claude/Codex product runtime, and any second repository task queue/router/scheduler/review engine.

Therefore `novel_capability=false`; no new owner or ADR is justified.

## Traceability
- Requirement/responsibility: `TDRP-R9-CLAUDE-PERFORM-TASK-COMMAND-001`
- Oracles: `ORA-TDRP-PERFORM-TASK-EXACT-AUTHORITY-001`, `ORA-TDRP-PERFORM-TASK-SINGLE-TASK-BOUNDARY-001`, `ORA-TDRP-PERFORM-TASK-SHARED-LIFECYCLE-001`, `ORA-TDRP-PERFORM-TASK-CONDITIONAL-COMMIT-001`
- Invariants: `INV-TDRP-PERFORM-TASK-NO-SECOND-QUEUE-001`, `INV-TDRP-PERFORM-TASK-NO-PROVIDER-RUNTIME-001`, `INV-TDRP-PERFORM-TASK-NO-EVIDENCE-BYPASS-001`
- Contracts: `CONTRACT-TDRP-PERFORM-TASK-INVOKE-ONE-001`, `CONTRACT-TDRP-PERFORM-TASK-SPLIT-FENCE-001`, `CONTRACT-TDRP-PERFORM-TASK-STOP-BOUNDARY-001`
- Fault/recovery: `FAULT-TDRP-PERFORM-TASK-STALE-LINEAGE-001`, `FAULT-TDRP-PERFORM-TASK-BLOCKED-SHARED-LIFECYCLE-001`
- Security: `SEC-TDRP-PERFORM-TASK-NO-PROVIDER-PRIVILEGE-BYPASS-001`
- Regression: `REG-TDRP-PERFORM-TASK-NO-AUTO-CONTINUE-QUEUE-001`

Closing only strict index 64 changes counters to unknown=16,239, unread=16,214, omitted=0, responsibilities=82. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root entry: index 65 `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`.
