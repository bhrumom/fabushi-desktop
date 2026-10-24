# Microsoft/GSL tests/utils_tests.cpp complete read — strict172

## Authority

- Parent PR #154 exact HEAD: `dcea9a7c52c944d4dff14f45f6506206001a8c1d`, tree `40b9543d49d65314c5c64165b17c7f18dd5397fe`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; Microsoft/GSL child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Exact source `Microsoft/GSL:tests/utils_tests.cpp@5b64e207eda9ef0e8935701647d6d4bed567960e`: 166 line slots / 4,211 bytes, fully re-read.
- Tested headers re-read: `include/gsl/util@26b2f5fb9d0d8c28c3096f50d575eb42a4b89bad` (172 line slots / 5,222 bytes) and `include/gsl/narrow@7b1a7912163185e2b2cbecc34ad64a539d566173` (83 line slots / 3,408 bytes).
- Build/license anchors: `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`, root `CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda`, MIT `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`.
- Parent exact-head Actions: TDRP Revision 9 run `40085527657` is 61/61 success; Rust desktop runtime `40085527748` is 6/6 success; Official MCP marketplace `40085527664` is success.

## Source behavior and negative oracle

This file is dependency-test provenance, not a shipping owner. It contains 10 GoogleTest cases:

1. `sanity_check_for_gsl_index_typedef` verifies `gsl::index == std::ptrdiff_t` with one `static_assert`.
2. Seven `finally_*` cases verify scope-exit execution for lambdas, moved guards, const/mutable lvalue lambdas, bound functions, function pointers and functions.
3. `narrow_cast` proves unchecked cast behavior, including `300 -> unsigned char == 44`.
4. `narrow` verifies checked conversion success and five throw oracles for value/sign loss, including 300-to-char, negative signed-to-unsigned boundaries and lossy complex conversion.

The tested `final_action` stores one callable, invokes it from the destructor only while its `invoke` flag is true, and on move transfers the callable plus the invoke flag while disabling the moved-from guard. `finally(F&&)` only constructs that scope guard. The production-equivalent Rust behavior is ordinary ownership/Drop/RAII or a narrowly scoped guard inside each existing domain owner; it is not a new cross-domain runtime.

`narrow_cast<T>(u)` is explicitly an unchecked `static_cast`. `narrow<T>(u)` performs the cast, casts back, rejects value changes, and for arithmetic types rejects signedness/sign changes. NaN fails the equality check. The source failure is `narrowing_error`. In Fabushi, applicable production behavior belongs to checked `TryFrom`/`TryInto`, explicit range/sign validation, or existing domain-specific conversion APIs rather than a GSL exception runtime.

The `narrow` test is excluded under `GSL_KERNEL_MODE`; this is a dependency-test/build variant and not a Fabushi product platform mode.

## Production consumer boundary

GitHub exact-code search over accepted `tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` for `"gsl::finally"` returned `161` results with `incomplete_results=false`; every result URL was pinned to that exact commit. Classification is:

- 155 production source/header files.
- 5 tdesktop test files.
- 1 repository review/governance document (`REVIEW.md`).

Direct exact-code searches for `gsl::narrow<`, `gsl::narrow_cast<` and `narrow_cast<` returned zero tdesktop files.

The 155 production `gsl::finally` consumers span APIs, conversations/dialog lists, media/FFmpeg, calls, storage, platform adapters, export, settings, wallet, notifications and UI lifecycle code. They remain independent source responsibilities owned by their actual Fabushi domain/platform owners. This strict entry does not mark any of those product behaviors implemented or verified.

A zero direct `narrow` name result also does not authorize omission of conversion responsibilities: indirect helper semantics and each actual domain conversion remain governed by their own source rows. Strict153 already recorded the generic checked-narrowing header responsibility separately; strict172 records the test file itself as an independent source entry.

## Existing-owner-first

Selected owners:

- Source/test provenance: existing TDRP inventory, ledger and dossier governance.
- Scope cleanup: the existing Rust/domain/platform owner that owns the resource, state mutation or lifecycle being guarded, using Rust ownership/Drop or the smallest local scope guard where needed.
- Checked conversion: the existing domain validation boundary using checked conversion/range/sign rules.

Rejected duplicate roots:

- `GslUtilRuntime`
- `GslFinallyRuntime`
- `TelegramScopeGuardRuntime`
- a global conversion service or GSL exception subsystem

No novel product owner is introduced.

## Fail-closed boundary

This strict entry proves only the exact `utils_tests.cpp` source identity, its registered test cases, tested header semantics, build registration, negative oracles and MIT provenance. It does **not** claim:

- execution of Microsoft/GSL C++ tests in Fabushi CI;
- production implementation of the 155 `gsl::finally` consumers;
- closure of indirect conversion callers;
- source closure;
- baseline acceptance;
- production responsibility closure;
- independent release acceptance.

Counters after exactly this source entry are `unknown=16,204`, `unread=16,179`, `omitted=0`, `responsibilities=190`. `baseline_identity_accepted=false` and `source_closure_ready=false`.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-UTILS-TEST-PROVENANCE-001`
- Target symbol: `TDRP_GSL_UTILS_TEST_PROVENANCE_001`
- Oracles: `ORA-TDRP-GSL-UTILS-TEST-EXACT-SOURCE-001`, `ORA-TDRP-GSL-UTILS-TEST-FINALLY-ONCE-001`, `ORA-TDRP-GSL-UTILS-TEST-NARROW-FAILURE-001`, `ORA-TDRP-GSL-UTILS-TEST-CMAKE-REGISTRATION-001`
- Invariants: `INV-TDRP-GSL-UTILS-TESTS-ONLY-001`, `INV-TDRP-GSL-UTILS-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-UTILS-NO-CALLER-CLOSURE-001`
- Contracts: `CONTRACT-TDRP-GSL-UTILS-REGISTERED-TESTS-001`, `CONTRACT-TDRP-GSL-UTILS-EXACT-HEAD-EVIDENCE-001`
- Fault oracles: `FAULT-TDRP-GSL-UTILS-SOURCE-DRIFT-001`, `FAULT-TDRP-GSL-UTILS-BUILD-REGISTRATION-001`, `FAULT-TDRP-GSL-UTILS-CALLER-CLOSURE-001`
