# Complete read — .agents/shared/cross-review/prompts.md

Status: understood; responsibility decomposed; toolchain-equivalent mapping prepared; parent entry exact-head verification pending  
Upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`  
Path: `.agents/shared/cross-review/prompts.md`  
Blob: `4e155d2bd7e647c12df7b008c279969bfd65e44d`

## Complete responsibility decomposition

This file is AI-assisted development review tooling, not Telegram product runtime. It defines the prompt contract for independent, read-only reviewers participating in the adjacent provider-neutral cross-review protocol.

Its responsibilities are:
- freeze every reviewer to an explicit source checkout and complete change scope rather than an inferred subset;
- enforce read-only review leaves: no edits, commits, delegation, sandbox bypass, or unrelated session-state inspection;
- require concrete, falsifiable findings with path/line/symbol, severity, triggering failure, and specific fix;
- distinguish BLOCKING failures from NON_BLOCKING preference/speculation;
- require one full general correctness/integration review plus orthogonal lifetime, reuse, structure, performance, and security lens decisions;
- let a lens return NOT_APPLICABLE only with scope/hunk-specific proof; uncertainty remains applicable;
- make synthesis re-check every finding against code, merge duplicates, drop unsupported findings, and reject invalid NOT_APPLICABLE decisions;
- require later focused review to verify accepted fixes and inspect the fix's adjacent callers/consumers without needlessly reopening unrelated unchanged code.

The adjacent `.agents/shared/cross-review/protocol.md` confirms these prompts feed a provider-neutral independent review loop whose runner arbitrates evidence and converges fixes. The prompt file itself does not define any end-user Telegram/Fabushi feature.

## Fabushi applicability and owner audit

Fabushi does not need a shipped cross-review prompt engine or Telegram-specific reviewer roles. The applicable guarantees already belong to source-neutral repository quality owners:
- `AGENTS.md` requires Spec Compliance Review before Integrate/Deliver and fail-closed accounting of every applicable criterion.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md` requires independent exploratory/release review, real pass evidence, and treats blocked/skipped/not-run/flaky as non-pass.
- `projects/fabushi-communication-platform/quality/evidence-contract.md` requires exact target/run/job/environment/artifact provenance.

Shipping `Agent`, `Coordinator`, `Host`, and `Runner` are rejected as owners because they execute Fabushi product behavior, not developer code review. A provider- or Telegram-named review runtime would create a parallel quality owner with no product responsibility.

The source-neutral equivalent is therefore: review the complete declared change, keep reviewers read-only and independent, require concrete failure evidence, explicitly prove lens non-applicability, synthesize/dedupe findings against code, and re-review accepted fixes before integration.

## Equivalent contract

`frozen change scope -> independent read-only general + applicable specialist review -> concrete findings -> synthesis against current code -> supported blocking findings gate integration -> accepted fixes -> focused re-review -> independent acceptance`

A preference, speculative cleanup, unsupported finding, or unproved NOT_APPLICABLE decision cannot masquerade as a blocking defect or completed coverage.

## Traceability

Requirement: `TDRP-R9-CROSS-REVIEW-PROMPTS-001`  
Oracle: `ORA-TDRP-CROSS-REVIEW-PROMPTS-001`  
Invariant: `INV-TDRP-INDEPENDENT-REVIEW-SCOPE-001`

Oracle: Fabushi retains an independent, source-neutral and fail-closed review contract that covers the complete declared change, keeps review activity read-only, requires concrete failure evidence, records specialist applicability, validates synthesis against code, and re-reviews fixes before integration; no provider-specific review runtime is added to the shipping product.

Current parent basis: PR #126 exact HEAD `f41e63397364778d00db14fc412cd9eaf98915cd` is still undergoing GitHub Actions verification. This dossier is staged on a descendant preparation branch only; coverage counters, ledger target commit and acceptance state must not advance until the parent entry merges and this mapping receives its own exact-head verification.
