# Microsoft/GSL tests/notnull_tests.cpp complete read — strict166

## Exact authority and source identity

- Fabushi main at audit: `2d915242d67a9ad1c335459b34d8a8c8561605ab`.
- PR #154 branch `tdrp-r9-entry31-includes-r27-20261020`: parent HEAD `9c1778e76f54db28811066127789009615fc356d`, tree `138c726b05df5091b441e6d16783336b03fc7b63`; recursive tree not truncated (9607 entries).
- Live `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` direct gitlink `87f9d768866548b5b86e72be66c60c5abd4d9b37`, child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- `tests/notnull_tests.cpp@df8e5a9acf529a9da9e578da413e955900e91e7d`: 649 lines / 18,407 bytes.
- `include/gsl/pointers@63e193e92478632520eca7e872ac83c643ffd2c0`: 348 lines / 11,854 bytes; `include/gsl/assert@d3e6ccdd53d5a254ed3827cf07711a6e86bb4226`: 134 lines / 3,988 bytes.
- `tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b`, `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`, `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02` were re-read; license is MIT.
- CMake registers `notnull_tests.cpp` in `gsl_tests` at line 206 and CTest registers `gsl_tests` at line 221.

## Behavior, consumers and negative oracles

The tests cover constructors, stream output, casts, assignment, raw-pointer and smart-pointer comparisons, dereference, CTAD, variant emplacement, `make_not_null`, and hashing. `not_null<T>` requires a type comparable to nullptr; its converting/value constructors enforce `Expects(ptr_ != nullptr)` (pointers lines 101-110), while nullptr construction and assignment are deleted (lines 129-131). `make_not_null` forwards into the same wrapper (lines 146-150). Failed `Expects` routes through the noreturn/noexcept termination path in `include/gsl/assert` (lines 111-127), normally `std::terminate()`, with a scoped MSVC no-exceptions workaround.

Negative oracles: null construction must die (test line 178); null-backed wrapper assignment must die (line 307); null CTAD cases must die (492, 500, 506-507); null `make_not_null` cases must die (591, 600, 606-607). If an invalid operation returns, `EXPECT_DEATH` fails. `CONFIRM_COMPILATION_ERRORS` blocks are compile-error probes, not ordinary runtime evidence. Four `GetExpectedDeathString` calls (165, 300, 485, 583) reuse the separately audited test-only matcher. The independent strict165 `no_exception_ensure_tests.cpp` dossier and `gsl_noexcept_tests` target remain separate.

## Existing-owner-first / fail-closed

Reuse the existing TDRP dependency inventory and test-provenance owner; actual production nullability stays with canonical Fabushi domain owners. Do not add GSLNotNullRuntime, a global pointer wrapper, a second assertion/termination owner, or a product composition slot. This row claims no production entrypoint, runtime, persistence, service or UI implementation. Re-read exact refs if gitlink/tree, source/header/helper/build registration or license changes.

Requirement: `TDRP-R9-GSL-NOTNULL-TEST-PROVENANCE-001`.

Oracles: `ORA-TDRP-GSL-NOTNULL-EXACT-SOURCE-001`; `ORA-TDRP-GSL-NOTNULL-CONSTRUCTOR-REJECTS-NULL-001`; `ORA-TDRP-GSL-NOTNULL-ASSIGNMENT-REJECTS-NULL-001`; `ORA-TDRP-GSL-NOTNULL-MAKE-NOT-NULL-REJECTS-NULL-001`; `ORA-TDRP-GSL-NOTNULL-CMAKE-REGISTRATION-001`.

Invariants: `INV-TDRP-GSL-NOTNULL-TESTS-ONLY-001`; `INV-TDRP-GSL-NOTNULL-NO-PARALLEL-RUNTIME-001`; `INV-TDRP-GSL-NOTNULL-NO-BULK-CALLER-CLOSURE-001`.

Counters before strict166: 16,379 source entries; unknown 16,211; unread 16,186; omitted 0; 183 responsibilities; 190 ledger rows. After strict166: 16,379 source entries; unknown 16,210; unread 16,185; omitted 0; 184 responsibilities; 191 ledger rows. Baseline identity acceptance, source closure and independent release acceptance remain open.

## Target anchor

<!-- TDRP_GSL_NOTNULL_TEST_PROVENANCE_001: strict166 source/test-provenance target anchor; documentation only, not a production symbol or runtime owner. -->
