# Microsoft/GSL tests/owner_tests.cpp — strict168

## Exact source lease

- Fabushi PR #154 branch `tdrp-r9-entry31-includes-r27-20261020`: parent exact HEAD `37ac69d26ec41aa45d8a19fa07fc3ff5df42beec`, tree `c532e710f4cf63d0497a13f7439bf74ca4c69cf7`.
- Live Fabushi `main`: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Live `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Direct child gitlink `Telegram/ThirdParty/GSL`: `87f9d768866548b5b86e72be66c60c5abd4d9b37`, tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`; the recursive child tree is not truncated.
- Exact source blob: `Microsoft/GSL:tests/owner_tests.cpp@87c7cec89a5fab4e1ad9ddda56a938766967316d`, 44 lines / 1,285 bytes.
- `include/gsl/pointers@63e193e92478632520eca7e872ac83c643ffd2c0`: `gsl::owner` is an alias constrained by `std::is_pointer<T>::value` (lines 78–79).
- `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`: `owner_tests.cpp` is registered in `gsl_tests` at line 207; CTest registers `gsl_tests` at line 221. The separate `gsl_noexcept_tests` target is not part of this row.
- `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02` confirms the dependency's MIT license.

## Behavior and negative oracle

The source contains two GTest cases:

- `owner_tests.basic_test` allocates `new int(120)` as `owner<int*>`, verifies 120, passes the pointer to `f(int*)` which increments the value to 121, verifies the mutation, and deletes the pointer (lines 24–33).
- `owner_tests.check_pointer_constraint` contains guarded compile-error probes for `owner<int>` and `owner<std::shared_ptr<int>>` under `CONFIRM_COMPILATION_ERRORS` (lines 36–42). These are compile-time probes, not ordinary runtime assertions and not evidence that the guarded cases ran.

The decisive type constraint is the alias template in `include/gsl/pointers`; `gsl::owner<T>` does not introduce runtime ownership, RAII, lifetime tracking, or a second allocation manager. The positive runtime test documents use of a raw owning pointer but does not prove production ownership safety. If the pointer-type constraint is removed, the guarded non-pointer declarations no longer provide their intended compile-error oracle; if `owner_tests.cpp` is removed from CMake, the normal `gsl_tests` suite no longer includes this provenance.

Bounded GitHub code searches against `telegramdesktop/tdesktop` for `gsl::owner`, `owner<`, and `owner_tests.cpp` returned no results. This is a bounded direct-name search only: it does not close indirect callers or any production ownership responsibility.

## Owner disposition

Reuse the existing TDRP source/dependency inventory and test-provenance owner. Actual memory ownership and lifecycle behavior remain with the applicable existing canonical Fabushi Rust/domain owners after their own exact-head audit. No `GslOwnerRuntime`, Telegram-specific pointer wrapper, second allocation manager, or new product owner is warranted.

This dossier is **source-only dependency-test provenance**. It does not claim production implementation, execution of Microsoft/GSL's C++ tests, production entrypoints, closure of downstream callers, or release acceptance.

- Requirement: `TDRP-R9-GSL-OWNER-TEST-PROVENANCE-001`
- Oracles: `ORA-TDRP-GSL-OWNER-EXACT-SOURCE-001`, `ORA-TDRP-GSL-OWNER-POINTER-TYPE-CONSTRAINT-001`, `ORA-TDRP-GSL-OWNER-CMAKE-REGISTRATION-001`, `ORA-TDRP-GSL-OWNER-NO-DIRECT-TDESKTOP-CONSUMER-001`
- Invariants: `INV-TDRP-GSL-OWNER-TESTS-ONLY-001`, `INV-TDRP-GSL-OWNER-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-OWNER-NO-BULK-CALLER-CLOSURE-001`
- Exact-head pre-publication checks: TDRP Revision 9 run `40067725777` success (30/30 jobs), Rust desktop runtime run `40067725696` success (6/6 jobs), Official MCP marketplace run `40067725689` success (1/1 job), all bound to parent HEAD `37ac69d26ec41aa45d8a19fa07fc3ff5df42beec`. Fresh runs on the post-publication exact HEAD are required and remain pending at dossier authoring time.

## Coverage after recording strict168

- Recursive source entries: 16,379
- Unknown: 16,208
- Unread: 16,183
- Omitted: 0
- Responsibility records: 186
- Baseline identity accepted: false
- Source closure ready: false
- Independent release acceptance: not accepted
