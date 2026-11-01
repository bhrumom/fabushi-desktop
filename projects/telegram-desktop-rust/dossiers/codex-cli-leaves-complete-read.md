# Complete read — .agents/shared/codex-cli-leaves.md

Status: understood, mapped, toolchain-equivalent; independent verification pending  
Upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`  
Path: `.agents/shared/codex-cli-leaves.md`  
Blob: `659da44a8083c433c4e9b59ca28f64ba525bfbf8`

## Complete responsibility decomposition

This file is AI-assisted development/review tooling, not Telegram product runtime. It defines a provider-specific Codex CLI leaf runner used only when a non-Codex host delegates review/assessment phases. Its responsibilities are: provider availability and native fallback; explicit review effort/model provenance; read-only or workspace-limited sandboxing; unchanged source checkout; isolated local run-state/transcript files; concurrent leaf start plus authoritative completion wait; resume only from a completed prior leaf; bounded failure/retry/fallback; no user-side install/login or global git mutation.

## Fabushi applicability and owner audit

Fabushi does not ship or require `codex exec`, `codex_leaf.py`, a provider-specific AI review daemon, or those machine-local run files. No such owner exists in current main, and creating one would make the product/toolchain depend on an upstream development-agent implementation.

The applicable engineering guarantees already belong to source-neutral Fabushi quality owners:
- `AGENTS.md`: Discover -> Spec -> Implement -> Verify -> Spec Compliance Review -> Integrate/Deliver; completion is fail-closed and FBCP/TDRP executable verification is GitHub-Actions-only.
- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`: independent exploratory acceptance, independent release reviewer, no blocked/skipped/flaky result counted as pass.
- `projects/fabushi-communication-platform/quality/evidence-contract.md`: evidence must bind target SHA, run/job/environment/artifact identity and independent acceptance records.

Provider-specific mechanics (Codex installation/login discovery, CLI thread resume IDs, detached local processes, local JSONL/stderr/lock files, Windows local-user sandbox implementation) are non-product implementation details and are not copied. Their applicable guarantees are retained as provider-neutral review independence, sandbox/no-source-mutation policy, provenance, fail-closed result semantics, and independent acceptance.

## Equivalent contract

`current source/spec -> implementation -> GitHub Actions exact-head verification -> evidence review -> independent reviewer/acceptance -> integrate only on pass`

No AI provider availability failure can be presented as passing product evidence. No review provider may authorize local executable verification for TDRP. Review evidence must identify the exact target and remain separate from implementation claims.

## Traceability

Requirement: `TDRP-R9-CODEX-LEAF-REVIEW-001`  
Oracle: `ORA-TDRP-CODEX-LEAF-REVIEW-001`  
Invariant: `INV-TDRP-REVIEW-PROVIDER-NEUTRAL-001`

Oracle: the shipping repository has no Codex CLI runtime dependency; review/acceptance semantics remain source-neutral, independent, exact-head, and fail-closed; executable verification remains GitHub Actions only.

Current evidence basis: main `160703057b24f6339ef2d88732c38dced8bc13aa`; PR #124 exact-head authority run `39272224644@b204fe042a32e975b39ee6d511c6f5b2d8cb2900` and Rust runtime run `39272223926@b204fe042a32e975b39ee6d511c6f5b2d8cb2900` both succeeded. This mapping itself remains pending exact-head verification.
