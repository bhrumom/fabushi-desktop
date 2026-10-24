# Microsoft/GSL tests/strict_notnull_tests.cpp — strict167

## Exact source lease
- Fabushi PR #154 parent HEAD `5ce86b88df3e6e37dcf3ec7b1adf0df2dac7cfcc`, tree `988829a9a0e98c141e6aae25816a80de92033d56`.
- tdesktop dev `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- GSL gitlink `Telegram/ThirdParty/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37`, child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- `tests/strict_notnull_tests.cpp@c1b1f54c97abe1c2cf825e81e713bee2fb8a888f`: 305 lines / 7,437 bytes.
- `include/gsl/pointers@63e193e92478632520eca7e872ac83c643ffd2c0`, `include/gsl/assert@d3e6ccdd53d5a254ed3827cf07711a6e86bb4226`, shared `tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b`, `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`, MIT `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`.
- CMake includes this source in `gsl_tests` line 211; CTest registers `gsl_tests` line 221.
- Next candidate `tests/owner_tests.cpp@87c7cec89a5fab4e1ad9ddda56a938766967316d` remains open for strict168.

## Behavior and oracle
Two GTest cases: `TestStrictNotNull` covers explicit construction, const-correctness, conversions, comparisons and hashes; `TestStrictNotNullConstructorTypeDeduction` is gated to C++17+ and uses the existing matcher at line 225. Null CTAD death oracles are lines 278, 286, 292 and 293. `CONFIRM_COMPILATION_ERRORS` sections are compile-time probes, not runtime evidence.

The existing `strict_not_null<T>` wrapper derives from `not_null<T>`; explicit constructors are at pointers lines 270/274, inherited `Expects(ptr_ != nullptr)` at 104/110, deleted nullptr construction/assignment at 297/298, and the C++17 deduction guide at 332. `Expects` dispatches to a noreturn termination path in assert lines 111-127. The shared death-test matcher is reused; no parallel runtime is justified.

Exact-name tdesktop searches for `strict_not_null` and `strict_notnull_tests.cpp` returned zero direct matches. This bounded direct-name result does not close indirect callers or production nullability responsibilities.

## Owner disposition
Existing TDRP source/dependency inventory and test-provenance owner. Product nullability remains with canonical Fabushi Rust/domain owners. No GSL runtime, second assertion owner, parallel C++ pointer wrapper, or Telegram-named product owner. This is source-only provenance: no production entrypoint, production evidence, or test-execution evidence is claimed.

Requirement: `TDRP-R9-GSL-STRICT-NOTNULL-TEST-PROVENANCE-001`.
Oracles: `ORA-TDRP-GSL-STRICT-NOTNULL-EXACT-SOURCE-001`, `ORA-TDRP-GSL-STRICT-NOTNULL-CTAD-NULL-TERMINATES-001`, `ORA-TDRP-GSL-STRICT-NOTNULL-CMAKE-REGISTRATION-001`, `ORA-TDRP-GSL-STRICT-NOTNULL-NO-DIRECT-TDESKTOP-CONSUMER-001`.
Invariants: `INV-TDRP-GSL-STRICT-NOTNULL-TESTS-ONLY-001`, `INV-TDRP-GSL-STRICT-NOTNULL-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-STRICT-NOTNULL-NO-BULK-CALLER-CLOSURE-001`.
Contract cases: `CONTRACT-TDRP-GSL-STRICT-NOTNULL-REGISTERED-TESTS-001`, `CONTRACT-TDRP-GSL-STRICT-NOTNULL-EXACT-HEAD-EVIDENCE-001`.
Fault cases: `FAULT-TDRP-GSL-STRICT-NOTNULL-SOURCE-DRIFT-001`, `FAULT-TDRP-GSL-STRICT-NOTNULL-BUILD-REGISTRATION-001`, `FAULT-TDRP-GSL-STRICT-NOTNULL-BULK-CALLER-CLOSURE-001`.

Parent exact-head only: TDRP run 40065985915 succeeded 61/61; Rust 40065985904 succeeded 6/6; Official MCP marketplace 40065985932 succeeded. These repository gates do not claim that Microsoft/GSL C++ tests ran. Fresh exact-head Actions are required after publication.

Counters before strict167: unknown=16,210; unread=16,185; omitted=0; responsibilities=184. After one source-only provenance row: unknown=16,209; unread=16,184; omitted=0; responsibilities=185. Baseline identity, source closure, and independent release acceptance remain open.

<!-- TDRP_GSL_STRICT_NOTNULL_TEST_PROVENANCE_001: source-only anchor; no production symbol or runtime owner. -->
