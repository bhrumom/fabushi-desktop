# Complete read — .agents/shared/cross-review/protocol.md

Status: understood; responsibility decomposed; toolchain-equivalent mapping prepared; parent entries exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`  
Path: `.agents/shared/cross-review/protocol.md`  
Blob: `51dabe142d5c3c7247e4050790c4e74b40f35b27`

## Complete responsibility decomposition

This file is provider-neutral AI development review orchestration, not Telegram product runtime. It defines a bounded cross-provider review loop in which a stateful runner owns scope and arbitration while fresh independent reviewers examine one frozen change through general and specialist lenses.

Its responsibilities are:
- freeze the reviewed scope once, including declared uncommitted work or a bounded commit range, and snapshot each round without mutating the source index;
- run one complete general review plus independent lifetime, reuse, structure, performance, and security lenses;
- keep reviewers mutually blind until synthesis so one review does not contaminate another;
- require the general reviewer to validate, deduplicate, or drop specialist findings against current code;
- keep one stateful arbiter responsible for every synthesized finding and record FIX / REJECT / DEFER / COMPROMISE decisions with rationale before mutation;
- prevent review oscillation by preserving prior decisions, requiring a genuinely new concrete failure before reversal, bounding supersession, and escalating repeated design flips to a human;
- after accepted fixes, re-review the actual fix and relevant adjacent callers/consumers rather than restarting unlimited full review;
- stop only on convergence, a bounded round cap, demonstrated non-convergence, or an explicit human product/design decision boundary;
- preserve a durable ledger of scope, rounds, decisions, supersession and final open items.

The protocol explicitly says provider identity is replaceable. Its Codex/Claude mechanics are current host adapters, not product behavior.

## Fabushi applicability and owner audit

The applicable guarantees map to Fabushi's existing development and quality owners:
- `AGENTS.md` owns governed implementation -> verification -> Spec Compliance Review -> Integrate/Deliver and FBCP/TDRP GitHub-Actions-only executable verification.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` owns independent review, evidence-based pass/fail, defect/regression handling, and independent release acceptance.
- `projects/fabushi-communication-platform/quality/evidence-contract.md` owns exact target/run/artifact provenance.

Fabushi does not need a shipped cross-provider reviewer runtime, review ledger service, Telegram review app, or a second Agent/Coordinator/Host/Runner lifecycle. Provider-specific process/session mechanics remain outside product architecture.

The source-neutral equivalent is independent review over a frozen scope, orthogonal risk lenses, evidence-backed synthesis, explicit disposition of every supported finding, bounded reversal/retry, fix re-verification, and fail-closed escalation when review cannot converge or requires a product choice.

## Equivalent contract

`freeze scope -> independent general + applicable lens reviews -> synthesis against code -> explicit arbitration ledger -> accepted fixes -> focused re-review -> converge OR bounded hard stop/human decision -> integrate only with supported pass evidence`

No reviewer vote is automatically authoritative. No unsupported finding is preserved merely because a reviewer emitted it. No real blocking defect in the changed code is ignored merely to save a round.

## Traceability

Requirement: `TDRP-R9-CROSS-REVIEW-PROTOCOL-001`  
Oracle: `ORA-TDRP-CROSS-REVIEW-PROTOCOL-001`  
Invariant: `INV-TDRP-REVIEW-CONVERGENCE-001`

Oracle: Fabushi's canonical quality process preserves a frozen review target, independent review, explicit evidence-backed finding disposition, bounded reversal/retry, re-verification of fixes, and fail-closed escalation when review cannot converge; no provider-specific review orchestration is introduced into the shipping product.

Current parent basis: entry 4 is staged on commit `9643aa50e0ee6c149b4914df23fc83d3d4a1b775`, which descends from PR #126 exact HEAD `f41e63397364778d00db14fc412cd9eaf98915cd`. This dossier is preparation only; coverage counters, ledger target commit and acceptance state must not advance until all parent entries have merged and this mapping receives its own exact-head GitHub Actions evidence.
