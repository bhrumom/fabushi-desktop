# Lock-thread maintenance workflow complete read — strict index 87

## Authority
- Upstream: `telegramdesktop/tdesktop@52a47639d9aa38713098974dd2ce07e631cb3381`, tree `42998681c09cdffcf75c0f38df71f663fc7b7b28`.
- Source: `.github/workflows/lock.yml@7904e79591f79a4ea61c0dc194c189f048bf5c67`, 16 line slots / 274 bytes.
- Provider ref: `dessant/lock-threads@v6 -> 89ae32b08ed1a541efecbab17912962a5e38981c` (package 6.0.2).
- Provider blobs: `action.yml@dd441d19ed1f3fe3d6e5d0c00c77d3b2b767a240`, `package-lock.json@8eb0c7b72beeedceadf9d2bb9df5aaeed288bb88`, `src/index.js@2782ee7dd3137660021496bbef987f649a548d35`, `src/schema.js@6a2850f10abff4dd2be93d156455ea1828741f8e`, `src/data.js@740d1d9a67d344614dacedd6ce606908c4aed7c8`, `src/utils.js@7daaca323090861a6315409c090178b6aff71155`, `LICENSE@a4ac43133b3847adb439a1d5103075a6e9078fa9`.
- Existing defect closure owner: `projects/fabushi-communication-platform/quality/defect-regression-policy.md@b31ecd424cac2abf8a2322e2996bb1b53c561378` / `FQT-DEF-001`.
- Parent exact-head 74d6348d1c42d0020a2154f4c9e7ac6d71178b33 completed TDRP run 39799900496 at 61/61 success (authority 12434433392 sha256:ad24881bfd0eea83cb64e9df5a7470b8023cb909f533484ed471e07e5e0b9bf3; closure 12435227835 sha256:53c1ca96926cf956b46862290d86a5652cfd929a8a3802c1fb770763831b8166; reachability 12434998390 sha256:7d183985c5b597559f2043330e848f0b4f17aeefbfd6a7421334a2f607e935ee) and Rust desktop runtime run 39799900499 at 6/6 success (artifact 12435053136 sha256:b7020766f9d25105ad59d15740b7f68925e944edaa8bcfb05c622e2f1c3bb4e2).

## Complete-read behavior
The workflow runs daily and invokes the provider with the repository GitHub token. It overrides issue and pull-request inactivity to 45 days. With provider defaults, closed and unlocked issues, pull requests, and discussions are considered; discussion inactivity remains the provider default 365 days. The provider searches bounded batches (50 per issue/PR kind and 50 discussions), uses retry/throttling, ignores already locked threads, and applies GitHub's administrative locked state to qualifying closed threads.

No product conversation, message retention, defect state machine, release gate, or runtime persistence is implemented by this workflow. A locked GitHub thread remains an administrative repository state.

## Existing-owner-first disposition
Fabushi already has a canonical defect closure contract: `FQT-DEF-001` requires reproduce -> fix -> targeted regression -> affected regression -> current-head evidence before close. Therefore the applicable Telegram responsibility is repository thread-maintenance provenance only. The workflow is mapped to the existing defect/evidence governance boundary and no second issue state, defect truth, release acceptance owner, or product moderation subsystem is introduced.

## Provider and dependency provenance
The mutable `v6` tag is resolved only as audit-time provenance; exact provider commit/tree and runtime blobs are retained above. The audited package-lock runtime closure contains 33 exact transitive package entries. Provider/network behavior, GitHub token permissions, rate limiting, retry behavior, pagination bounds and tag drift are supply-chain/administrative concerns, not product correctness authority.

## Fail-closed conclusions
- Locking a thread cannot close a Fabushi defect, satisfy a regression gate, or establish release acceptance.
- Provider/tag drift, token denial, rate limiting, partial traversal or schedule failure is non-passing maintenance evidence, not a reason to weaken `FQT-DEF-001`.
- A closed-but-unlocked thread may later become locked; this does not change the underlying defect evidence.
- Exact-head Fabushi acceptance remains GitHub-Actions evidence bound to the current commit.

## Exact traceability
- Requirement: `TDRP-R9-LOCK-THREAD-MAINTENANCE-PROVENANCE-001`
- Oracles: `ORA-TDRP-LOCK-THREAD-EXACT-SOURCE-001`, `ORA-TDRP-LOCK-THREAD-PROVIDER-001`, `ORA-TDRP-LOCK-THREAD-NO-CLOSURE-AUTHORITY-001`
- Invariants: `INV-TDRP-LOCK-THREAD-CLOSED-ONLY-001`, `INV-TDRP-LOCK-THREAD-NO-DEFECT-CLOSURE-001`, `INV-TDRP-LOCK-THREAD-EXACT-HEAD-EVIDENCE-001`
- Contracts: `CONTRACT-TDRP-LOCK-THREAD-MAINTENANCE-001`, `CONTRACT-TDRP-LOCK-THREAD-FABUSHI-CLOSURE-BOUNDARY-001`
- Temporal: `TEMP-TDRP-LOCK-THREAD-ACTION-TAG-DRIFT-001`, `TEMP-TDRP-LOCK-THREAD-INACTIVITY-WINDOW-001`
- Fault/recovery: `FAULT-TDRP-LOCK-THREAD-RATE-LIMIT-001`, `FAULT-TDRP-LOCK-THREAD-PROVIDER-PARTIAL-001`
- Security: `SEC-TDRP-LOCK-THREAD-TOKEN-001`, `SEC-TDRP-LOCK-THREAD-ADMIN-BOUNDARY-001`
- Regression: `REG-TDRP-LOCK-THREAD-DEFECT-OWNER-REUSED-001`, `REG-TDRP-LOCK-THREAD-NO-RELEASE-ACCEPT-001`

Closing strict index 87 changes counters to unknown=16,281, unread=16,256, omitted=0, responsibilities=105. `baseline_identity_accepted=false` and `source_closure_ready=false`. Next strict root is index 88 `.github/workflows/mac.yml@184a94b091d2d7bab19ae67d7f3a5881b18213ba`.
