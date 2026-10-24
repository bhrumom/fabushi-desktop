# Claude AI workflow adapter complete read — strict index 58

## Authority
- Upstream: `telegramdesktop/tdesktop@5c674f1f6a0f6ace55b67f8bc295138680fe588b`, tree `09a421891474be0da279f44a5bf8e0dcf3bc270a`.
- Strict zero-based root index: `58`.
- Source: `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`, 216 lines, fully read.
- Same-upstream dependencies re-read: `.agents/shared/phase-effort.md@dd07a4c611922a29a6892332db98afe76cd58952`, `.agents/shared/codex-cli-leaves.md@659da44a8083c433c4e9b59ca28f64ba525bfbf8`, `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`, `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, and `.agents/skills/process-inbox/scripts/workspace.py@5d74a11083d66f2ebe9bbf0fcf2e991fa2c6c1c5` for source-lineage execution.
- Parent Fabushi exact head: `33019a2459f3ed8dbd2b62cbe6d9dcf863a69ac5`, tree `4b3bbe0dc44fa29405e33198fe3e304ff1bc74ef`.
- Parent TDRP source-authority run `39701315580` completed success; authority artifact `12393500020`, digest `sha256:6e6c051c7a9bc97362a2fcd5a690b7baf28401d51618ab2d49ee3ed70eb3a321`; recursive reachability artifact `12392714851`, digest `sha256:69c605f4baa12bfd5c8e646f5c9e10d011ab2942c0027faac5ca7c1895b0bb06`; root-source-archive artifact `12392529584`, digest `sha256:17d303d2b2d8245e9ba42e011ef7c7b4c639a6731e689943cf865a8e3670cff2`.
- Parent Rust desktop runtime run `39701315674` completed success, 6/6 jobs; artifact `12392659959`, digest `sha256:021ada7f5f287c13796a2f70ce3263241f41c3b7928aa96ce95484c15e5b7c36`.
- Parent evidence is historical-only after this mapping commit; fresh child exact-head Actions remain mandatory.

## Complete responsibility decomposition
This file is an explicitly loaded Claude Code host adapter for the shared repository-development workflow, not Telegram product runtime and not a second workflow authority. The shared `.agents/skills/` workflow remains authoritative.

### Delegation and lifecycle
Claude `Agent` calls substitute for shared phase workers. Disposable phase leaves receive self-contained prompts, do not delegate and do not commit. Publication-owning orchestrators keep the shared single-writer and one-stateful-performer boundaries. Ordinary leaves run synchronously in the foreground; return is the completion signal and required artifacts/repository state are validated immediately. Background polling, watcher/Monitor substitutes, SendMessage-as-return, nested `claude` processes and duplicate stateful performers are forbidden. A missing artifact permits one bounded fresh retry only after the prior writer is proved finished; resumable stateful work is resumed by identity.

Independent leaves in one step may run concurrently where the shared workflow already allows it. The mandatory general reviewer remains isolated before specialist retirement and again for synthesis. Long builds may use the host's background shell completion signal, not sleep/poll loops.

### Effort/model truth
The shared phase-effort policy remains authority. Claude maps only host-supported model controls and never claims an unavailable reasoning-effort field was applied. Whole-assignment classification, escalation on unexpected complexity and durable recording of the actually applied setting/fallback remain required.

### Review and assessment provider
The adapter may use Codex CLI leaves for the enumerated independent review/assessment phases. User opt-out or failed Codex availability/authentication switches those phases to the native provider and records why. Detached start is never completion; the sanctioned foreground wait and terminal transcript/final-result adjudication remain required before phase artifacts are validated.

### Model self-reporting
Repository task state records the actual running Claude model identity, normalized to the canonical lowercase family name, never a model copied from queue/task metadata or another checkout.

### Text handling
Claude skips dedicated line-ending/BOM normalization work, including normalization-only Phase 7, but this narrow host exception does not weaken content validation, owned-path rules, build, review, test or publication gates.

### Source lineage and branch routing
The parent scheduler retains source-lineage decisions. Before batch freeze a mismatch is a human routing boundary; after freeze only the parent may perform the shared safe switch to an existing compatible local branch after cleanliness/recovery/worktree checks. A mismatch first established after Phase 1 becomes a clean task-local Block so unrelated batch work may proceed. Informal cross-task claims still must be checked on the current branch.

### UI-driver capability
A Codex-specific Computer Use driver is unavailable unless an equivalent driver is actually exposed in Claude. `auto` may use the planned overlay fallback; `required` reports the exact unverified interaction. Driver absence cannot waive runtime, account-safety or evidence requirements.

### Concurrent routing lane
Routing and pending-task consolidation are the narrow background-Agent exception. At most one of each may be in flight; both use the existing routing worktree/helper publication contract and must land and be validated before the invocation ends. Ordinary performers and phase leaves remain foreground. Recovery resumes the existing publication rather than creating a competing writer.

## Existing-owner-first disposition
Selected owner: Fabushi root `AGENTS.md` source-neutral repository AI development/review/evidence lifecycle, specifically:
- `Worker/phase effort and model inheritance`
- `Out-of-process AI worker lifecycle safety`
- `Out-of-process AI worker adapter execution`

These contracts already own truthful effort/model inheritance, terminal worker adjudication, no duplicate writers, immutable launch identity, availability fallback, sanctioned waiting and separate phase-artifact validation.

Rejected owners: shipping Agent/Coordinator/Host/Runner, shipping Task/Automation/Workflow, a Claude/Codex product runtime, a second repository scheduler/task queue/review owner, or a provider-specific evidence store. This source entry governs developer harness mechanics and creates no product UI, account, conversation, resource, search or persistence owner.

## Traceability
- Requirement/responsibility: `TDRP-R9-CLAUDE-AI-WORKFLOW-ADAPTER-001`
- Oracles: `ORA-TDRP-CLAUDE-ADAPTER-SHARED-AUTHORITY-001`, `ORA-TDRP-CLAUDE-ADAPTER-WORKER-LIFECYCLE-001`, `ORA-TDRP-CLAUDE-ADAPTER-PROVIDER-FALLBACK-001`, `ORA-TDRP-CLAUDE-ADAPTER-SOURCE-LINEAGE-001`
- Invariants: `INV-TDRP-CLAUDE-ADAPTER-NO-PRODUCT-RUNTIME-001`, `INV-TDRP-CLAUDE-ADAPTER-NO-DUPLICATE-WRITER-001`, `INV-TDRP-CLAUDE-ADAPTER-HOST-TRUTH-001`
- Contracts: `CONTRACT-TDRP-CLAUDE-ADAPTER-EXPLICIT-LOAD-001`, `CONTRACT-TDRP-CLAUDE-ADAPTER-FOREGROUND-LEAF-001`, `CONTRACT-TDRP-CLAUDE-ADAPTER-CODEX-FALLBACK-001`, `CONTRACT-TDRP-CLAUDE-ADAPTER-SOURCE-LINEAGE-001`
- Regression: `REG-TDRP-CLAUDE-ADAPTER-NO-DUPLICATE-WORKER-001`

Closing only strict index 58 changes counters to unknown=16,245, unread=16,220, omitted=0, responsibilities=76. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root entry: index 59 `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`.
