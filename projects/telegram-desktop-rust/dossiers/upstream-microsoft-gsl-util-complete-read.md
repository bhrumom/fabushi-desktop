# Microsoft/GSL include/gsl/util complete read — strict158

## Authority
- Live Fabushi main: fb1189efd9d85c650864d6987ea8fb68c913558d, tree 1a1ef09be4c9fd801354439a5b82cc460f0b4b6a.
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- include/gsl/util@26b2f5fb9d0d8c28c3096f50d575eb42a4b89bad, fully read: 172 line slots / 5222 bytes.
- tests/utils_tests.cpp@5b64e207eda9ef0e8935701647d6d4bed567960e, fully read: 166 line slots / 4211 bytes.
- tests/at_tests.cpp@42b2241bfd472ab384c35267d2042c870edce4a5, fully read: 174 line slots / 5604 bytes.
- LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02: Microsoft/GSL MIT license.
- Parent exact head 50aeeede7543dd8a189b80eae1a59e16654d5845, tree 563c5aaa05b048fa4496f0104c23029c748b8ecc.
- Parent TDRP 40047699934 completed 61/61 success; authority 12542076361 sha256:8721db2b5177a2365764ee274d323e8f521c695b38e716d173cceabb8b892617; closure 12542710366 sha256:e0cb77a0a10098616898d06c74f9027f126b13eee271a73035e935fb0713585f; reachability 12541154770 sha256:131e6747d41b79e95213c7bd9627aff6b061c64d69f95c618e72ee25c3818a6e.
- Parent Rust desktop runtime 40047699891 completed 6/6 success; artifact 12541908261 sha256:06b0a093bc0e976e2c9ec737893195dd27ca9078fff862119bff72a7f1a4cfba.
- Accepted exact-head tdesktop code search is complete (incomplete=false): 161 unique `gsl::finally` file hits = 155 production SourceFiles + 5 test files + REVIEW.md; direct `gsl::narrow_cast` = 0 and direct `gsl::at` = 0.
- Next: Microsoft/GSL include/gsl/zstring@2cc13d83840afec61e5bd709c77900026ed225f4.

## Exact behavior
`util` is a header-only low-level utility dependency. `gsl::index` aliases `std::ptrdiff_t`. `final_action<F>` owns a callable and invokes it from its noexcept destructor while armed; move construction transfers the callable and disarms the moved-from guard with `std::exchange(other.invoke, false)`, while copy construction and both assignments are deleted. `finally(F&&)` constructs that move-only scope-exit guard. The locked utils tests prove one invocation at scope exit, exactly-once behavior across moves, and support for lvalue/mutable lambdas and functions/function pointers.

`narrow_cast<T>` is intentionally an unchecked named `static_cast`; its focused test demonstrates that lossy conversion is possible. `gsl::at` validates signed indices with `Expects` before builtin-array, container, initializer-list and supported `std::span` access; the locked at tests prove in-range access and fail-closed termination on negative/out-of-range indices.

## Existing-owner-first
Fabushi does not need a `GSLUtilRuntime`, scope-guard service, second resource owner or compatibility API. Rust already supplies deterministic RAII/Drop and checked indexing primitives. Each actual Fabushi resource/lifecycle/state/platform owner must preserve its migrated caller's cleanup, restoration, cancellation and secret-destruction obligations when that caller is independently read and rewritten. This row accounts for the locked helper dependency and its exact semantics only.

The 155 production `gsl::finally` consumer files span media, history, info, platform, UI, wallet, calls, data, dialogs, boxes, storage, IV, settings, window, core, API, export, chat helpers, ffmpeg, MTProto, tde2e, editor and payments. Their product responsibilities remain independently open. Observed uses include native/FFmpeg/OpenSSL handle release, cryptographic key destruction, passcode-buffer cleansing, signal disconnection, completion notification and temporary state restoration. No caller is bulk-closed by this dependency row.

## Boundary and fail-closed rules
- Scope-exit cleanup must remain exactly once on normal and early exits; move/ownership transfer must not duplicate or suppress cleanup.
- Security-sensitive consumers such as passcode/key/crypto cleanup retain their own security acceptance and cannot inherit verification from `util`.
- Because accepted tdesktop has no direct `gsl::narrow_cast` or `gsl::at` match, this row does not invent downstream product consumers for those helpers.
- A GSL-shaped runtime, global cleanup owner, broad unsafe-conversion helper or bulk caller closure would violate existing-owner-first.
- Any accepted tdesktop/GSL gitlink, child tree, util/tests/license blob or consumer-set drift requires fresh reconciliation.

## Traceability
Requirement TDRP-R9-GSL-UTIL-SCOPE-EXIT-CHECKED-INDEX-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-UTIL-EXACT-SOURCE-001, ORA-TDRP-GSL-UTIL-FINAL-ACTION-SEMANTICS-001, ORA-TDRP-GSL-UTIL-CONSUMER-FANOUT-001, ORA-TDRP-GSL-UTIL-NO-BULK-CALLER-CLOSURE-001.
Invariants: INV-TDRP-GSL-UTIL-SCOPE-EXIT-EXACTLY-ONCE-001, INV-TDRP-GSL-UTIL-NO-GSL-RUNTIME-001, INV-TDRP-GSL-UTIL-CALLERS-REMAIN-INDEPENDENT-001.
Counters after strict158: unknown=16,218 unread=16,193 omitted=0 responsibilities=176.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
