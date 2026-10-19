# Complete read — .agents/shared/cross-review/protocol.md

Status: understood; responsibility decomposed; existing-owner implementation wired; exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@1e18da644e37856c9b1d87434bfcd6a2e3d11c57`  
Tree: `30c1d5cce4c1bde962882c1f2ca4b2e51bb03ae6`  
Path: `.agents/shared/cross-review/protocol.md`  
Blob: `51dabe142d5c3c7247e4050790c4e74b40f35b27`

## Complete responsibility decomposition

This source is repository development/quality orchestration, not Telegram product runtime. The complete file defines a bounded independent cross-review state machine:

- freeze the declared scope once, including base revision and any explicitly in-scope pre-existing uncommitted work;
- snapshot the immutable scope before every round without mutating the real index; empty scope is terminal;
- use fresh blind independent reviewers: one complete general reviewer plus lifetime/reuse/structure/performance/security specialist lenses in round 1;
- keep reviewer reports mutually blind, then perform synthesis against current code: confirm/drop findings, merge duplicates, and produce one numbered verdict list;
- keep durable state only in the source-neutral runner/ledger; reviewers do not carry prior arbitration state;
- decide every synthesized finding before fixes as `FIX`, `REJECT`, `DEFER`, or `COMPROMISE`, preserving rationale and change/no-change;
- record `NON_BLOCKING` findings but never let them alone start a fix round;
- treat a request to undo an earlier decision as a reversal: dismiss unless it supplies a concrete failure scenario not previously weighed;
- allow one bounded supersession of a decision while preserving both rationales; a second attempted reversal is a human-decision stop;
- after accepted fixes, perform fresh focused general review over the fix delta, touched functions/callers, and complete frozen scope; rerun only specialist lenses that originated fixed findings;
- do not expose rejected/deferred/compromise decisions to fresh reviewers, preserving independent evaluation;
- stop only on convergence, configured round cap (default ten), demonstrated non-convergence/scope growth into another subsystem, or a required human product/arbitration decision;
- never leave a real defect in changed code open merely to save a round;
- persist the complete decision ledger, per-round reviewer/synthesis reports, counts, reversal handling, fix confirmation, stop reason, and final report.

Provider names and local session-directory mechanics are source implementation details. The applicable product-quality responsibility is a provider-neutral, fail-closed review governance contract. No Claude/Codex review runtime is introduced into Fabushi.

## Existing-owner audit and production-quality wiring

The existing canonical owners remain sufficient:

- `AGENTS.md#Independent review contract` owns frozen declared scope, fresh/blind independence, durable source-neutral arbitration ownership, decision-before-fix, bounded reversal supersession, and hard stop conditions.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md#Cross-review semantics` owns round topology, general/specialist synthesis, focused re-review, FIX/REJECT/DEFER/COMPROMISE semantics, non-blocking behavior, convergence/non-convergence and round-cap quality gates.
- `projects/fabushi-communication-platform/quality/evidence-contract.md#Independent review evidence` owns durable scope/snapshot identity, raw reports/synthesis, arbitration ledger, supersession chain, per-round counts, focused confirmation, cap override, and terminal stop reason.

Rejected owners:
- shipping Agent/Coordinator/Host/Runner: product runtime, not repository quality governance;
- any Telegram/Claude/Codex-specific review runtime: would create a provider-specific second quality owner;
- a new review service/database: unnecessary because repository/GitHub evidence already owns durable quality state.

No novel owner is introduced.

## Equivalent contract

`freeze declared scope -> immutable round snapshot -> fresh blind independent general + applicable specialist review -> synthesis against current code -> durable runner arbitration ledger -> FIX/REJECT/DEFER/COMPROMISE -> accepted fixes -> fresh focused re-review -> bounded reversal/supersession handling -> convergence or explicit hard stop -> durable terminal report/evidence`

The contract is fail-closed: ambiguous scope, missing snapshot identity, missing synthesis, undecided findings, unsupported reversal, unreviewed FIX, open real defect, or missing stop reason cannot advance acceptance.

## Traceability

Parent requirement:
- `TDRP-R9-CROSS-REVIEW-PROTOCOL-001`

Responsibilities:
- `TDRP-R9-CROSS-REVIEW-PROTOCOL-SCOPE-001`
- `TDRP-R9-CROSS-REVIEW-PROTOCOL-ARBITRATION-001`
- `TDRP-R9-CROSS-REVIEW-PROTOCOL-CONVERGENCE-001`
- `TDRP-R9-CROSS-REVIEW-PROTOCOL-EVIDENCE-001`

Oracle:
- `ORA-TDRP-CROSS-REVIEW-PROTOCOL-001`

Invariants:
- `INV-TDRP-CROSS-REVIEW-FROZEN-SCOPE-001`
- `INV-TDRP-CROSS-REVIEW-BOUNDED-REVERSAL-001`
- `INV-TDRP-CROSS-REVIEW-FIX-REREVIEW-001`

This closes exactly one recursive source entry. With recursive inventory fixed at 16,192 for the current upstream candidate, fail-closed counters advance from unknown=16,188 / unread=16,161 / omitted=0 to unknown=16,187 / unread=16,160 / omitted=0. Baseline readiness, source closure, and acceptance remain false.
