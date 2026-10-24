# Upstream root REVIEW.md code-review/style provenance — strict index 108

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `REVIEW.md@e65be8231a8e0f50ee86b20ef16120d58ea40376`, 608 lines / 20794 parsed characters, fully read.
- Rule accounting: 25 `##` rules, from class-brace spacing through `No Q_OS_LINUX platform checks in new code`; none omitted.
- Parent exact HEAD: `07972a8c3ffa56e63903215587b1cb196e88c5db`, tree `70051fe9558afef00692415b9802cc849d628af8`.
- Parent TDRP run `39918824228`: 61/61 success; authority `12485614037` sha256:fcbc03dd5505b6369559b6e99fc8fa8836372dc106aef63ba1a3d576e7fc4c32; closure `12486885949` sha256:3b41e4c0d547c20e2c2abd75dc860b17b99877892056bf92f277d772ecf0f4d9; reachability `12485724417` sha256:5a7ef563152ad0b5eed32ba5318f8e3bb570cbc1540c6390226f67bc72f2eb7f.
- Parent Rust run `39918824363`: 6/6 success; artifact `12485338498` sha256:d7ad1e21d7072a98b491ede996b33121219f9cd822b802e2ddf7fe79a86f8ad8.

## Responsibility decomposition

The 25 upstream rules cover: class closing-brace spacing; consecutive blank lines; continuation-operator placement; direct subtype casts; basic-type initialization; `tr::` TextWithEntities projections; multi-line call argument layout; optional access without `.value()`; include ordering; C++17 namespace syntax; identical-branch consolidation; `base::take` read/reset; avoiding `rpl::single` around lang keys; extracting local-class method definitions; avoiding repeated `[[nodiscard]]`; RAII cleanup; extracting substantial lambda logic; data-driven matching; `isHidden()` logic semantics; consolidating `make_state`; avoiding mutable per-subscription `rpl` capture state; trailing-return restraint; data-size/alignment awareness; PascalCase static members; and avoiding new `Q_OS_LINUX` checks.

These are repository review/style responsibilities, not a product runtime. Several express broadly useful quality intent, but many are specifically bound to Telegram's C++/Qt/rpl/tr/base codebase. Fabushi therefore retains the complete exact-source guide as provenance while applying only rules independently adopted by its source-neutral repository policy.

## Existing-owner / validator uniqueness

Fresh parent-head preflight found zero prior `REVIEW.md` path rows, zero `e65be8231a8e0f50ee86b20ef16120d58ea40376` blob rows, and zero `TDRP-R9-UPSTREAM-ROOT-REVIEW-STYLE-PROVENANCE-001` rows. The selected canonical target is the existing `AGENTS.md` source-specific-rule boundary:

> A feedback adapter must not invent source-specific rules that Fabushi has not adopted, silently rewrite user code, widen its scope to unrelated files, weaken the durable spec, or turn a missing/disabled local hook into a passing result. Telegram C++-specific banned-function mappings, comment budgets, `.clang-tidy` parsing, source suffixes, test-harness exemptions, and Claude `PostToolUse` payload shapes are provenance only unless an independent Fabushi rule explicitly adopts the same behavior.

That text occurs exactly once at the parent head, and the exact `AGENTS.md` path + selected symbol has zero ledger owners. Existing supporting owners are the already-mapped repository quality-health change-ratchet/review policy and exact-head quality/evidence contracts. No Telegram review runtime, second style policy owner, second validator, or release gate is introduced.

## Fail-closed boundaries

- Telegram C++/Qt/rpl/tr/base conventions are provenance unless Fabushi independently adopts an equivalent source-neutral rule.
- REVIEW.md does not prove Rust/TypeScript correctness, user-visible behavior equivalence, security, platform support, packaging, deployment, or release readiness.
- The upstream `Q_OS_LINUX` rule is not a Fabushi OS-support contract.
- Review-subagent wording cannot replace GitHub Actions exact-head evidence or independent acceptance.
- Any REVIEW.md identity/rule-set drift, target-owner drift, PR/ref drift, or upstream lease drift requires fresh reconciliation.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-ROOT-REVIEW-STYLE-PROVENANCE-001`.
- Oracles: `ORA-TDRP-REVIEW-EXACT-SOURCE-001`, `ORA-TDRP-REVIEW-RULE-CLASSIFICATION-001`, `ORA-TDRP-REVIEW-NO-AUTOMATIC-ADOPTION-001`.
- Invariants: `INV-TDRP-REVIEW-NO-SECOND-VALIDATOR-001`, `INV-TDRP-REVIEW-NO-SOURCE-SPECIFIC-POLICY-IMPORT-001`, `INV-TDRP-REVIEW-NO-RELEASE-ACCEPT-001`.

Closing strict108 yields unknown=16,268, unread=16,243, omitted=0, responsibilities=126. Next strict root index 109 is `changelog.txt@41b7466f8a11a1866a66ff583809dfe74ad9a3c3`. Fresh child exact-head TDRP/Rust evidence is required.
