# Complete read — .agents/shared/cross-review/prompts.md

Status: understood; responsibility decomposed; existing-owner implementation wired; exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@1e18da644e37856c9b1d87434bfcd6a2e3d11c57`  
Path: `.agents/shared/cross-review/prompts.md`  
Blob: `4e155d2bd7e647c12df7b008c279969bfd65e44d`

## Complete responsibility decomposition

This source is AI-assisted development review tooling, not Telegram product runtime. The complete file defines a read-only independent reviewer contract:
- freeze review to an explicit source checkout and complete declared change scope;
- forbid reviewer edits, commits, delegation and sandbox bypass;
- require concrete findings with location/symbol, severity, triggering failure and specific fix;
- keep preference-only wording/naming/speculative cleanup non-blocking;
- run a complete general review plus specialist lenses, where NOT_APPLICABLE needs exact changed-path/hunk proof and uncertainty remains applicable;
- synthesize all findings back against current code, merge duplicates, drop unsupported findings and reject invalid NOT_APPLICABLE decisions;
- allow NEEDS_CHANGES only when at least one supported blocking finding exists;
- after accepted fixes, perform focused re-review of each fix plus relevant edited functions, callers and consumers without reopening unrelated unchanged code.

The adjacent cross-review protocol is development orchestration; none of these responsibilities justify a shipped Telegram/Codex review runtime.

## Existing-owner audit and production wiring

The applicable guarantees belong to existing source-neutral Fabushi quality owners:
- `AGENTS.md#Independent review contract` owns complete-scope, independent, read-only review and concrete/falsifiable finding requirements.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md#Cross-review semantics` owns specialist applicability, synthesis, supported-blocker gating and focused fix re-review.
- `projects/fabushi-communication-platform/quality/evidence-contract.md#Independent review evidence` owns durable scope/role/checked-surface/finding/N-A-proof/synthesis/focused-rereview evidence.

Shipping Agent, Coordinator, Host and Runner remain rejected: they execute Fabushi product behavior and are not repository review governance. No novel owner is introduced.

## Equivalent contract

`declared change scope -> independent read-only general + applicable specialist review -> concrete findings / evidenced N-A -> synthesis against current code -> supported blocker gate -> accepted fixes -> focused re-review -> independent verdict`

Unsupported findings, preference-only suggestions, unproved NOT_APPLICABLE, incomplete evidence, and ambiguous review state remain non-pass.

## Traceability

Requirements:
- `TDRP-R9-CROSS-REVIEW-PROMPTS-001` (parent traceability requirement)
- `TDRP-R9-CROSS-REVIEW-READONLY-SCOPE-001`
- `TDRP-R9-CROSS-REVIEW-VERDICT-REREVIEW-001`
- `TDRP-R9-CROSS-REVIEW-EVIDENCE-001`

Oracle: `ORA-TDRP-CROSS-REVIEW-PROMPTS-001`  
Invariant: `INV-TDRP-INDEPENDENT-REVIEW-SCOPE-001`

This closes exactly one recursive source entry. With recursive inventory fixed at 16,192 for the current upstream candidate, fail-closed counters advance from unknown=16,189 / unread=16,162 / omitted=0 to unknown=16,188 / unread=16,161 / omitted=0. Baseline readiness and acceptance remain false.
