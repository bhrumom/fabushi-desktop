# Microsoft/GSL tests/deathTestCommon.h complete read — strict164

## Authority and exact source lease

- Live Fabushi main: `fb1189efd9d85c650864d6987ea8fb68c913558d`, tree `1a1ef09be4c9fd801354439a5b82cc460f0b4b6a`.
- Fixed PR #154 branch `tdrp-r9-entry31-includes-r27-20261020`: exact parent HEAD `584dc11ecd07fd6431e8c876bf9508e1f3436a43`, tree `11f271518e182834bc9d3801d2f1391467f9f728`.
- Live `telegramdesktop/tdesktop` `dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`; recursive tree not truncated.
- `Telegram/ThirdParty/GSL` is a direct gitlink (mode `160000`, type `commit`) at `87f9d768866548b5b86e72be66c60c5abd4d9b37`; exact child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`, not truncated. The source identity follows the pinned gitlink, not a moving dependency branch.
- The fixed PR branch's `upstream.lock.json`, inventory and ledger match live tdesktop dev and the same GSL gitlink/tree. The live main's own lock still records tdesktop `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` / `5151db79560ed40bf3802482fcfb74b950be6c40`, which differs from live dev; this change is intentionally scoped to the fixed PR branch and does not rewrite main or rebase the branch.
- `tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b`, fully read: 12 lines / 380 bytes.
- `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`, fully read: 11,090 bytes.
- `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`, fully read: 1,157 bytes; MIT.
- Parent exact-head evidence for `584dc11ecd07fd6431e8c876bf9508e1f3436a43`: TDRP run [40056047058](https://github.com/bhrumom/fabushi-desktop/actions/runs/40056047058) terminal success, 61/61 jobs; authority artifact 12545735367 `sha256:3250a31b6c5b9664bc3c48d3dee3e3026a19a0100fdba496ea62226e6b9e700b`, recursive closure 12545970920 `sha256:94b580596f0337a21b2c96c0704e3943392c8329c1f8a9b0383c20d92daba2ac`, reachability 12545710914 `sha256:6f9aee92263f40a9bf6bac9ead8e29071c5372a11e8cc91eab2f6d09cd0f2cc8`. Rust run [40056047079](https://github.com/bhrumom/fabushi-desktop/actions/runs/40056047079) terminal success, 6/6 jobs; artifact 12545680534 `sha256:7482a72b5f904ef7b029fa94e518450ca73f8fc79f2e023a4912591d87746b9b`.
- The strict164 publication head must obtain its own fresh TDRP/Rust terminal runs after the CAS push; this dossier does not pre-claim those future results.

## Exact behavior

The helper is a test-only matcher selector, not an assertion or termination implementation. It defines `deathstring = "Expected Death"` and `failed_set_terminate_deathstring = ".*"`. `GetExpectedDeathString(std::terminate_handler handle)` returns the stable expected-death matcher when a terminate handler exists, and the permissive regex only when installing the terminate handler failed. That fallback prevents an environmental setup failure from falsely failing every death assertion; it does not prove the tested operation terminated correctly.

## Seven exact-ref registered test consumers

Every source below was fully read at the pinned GSL commit. Each includes `deathTestCommon.h`, calls `GetExpectedDeathString`, and is listed in the shared `gsl_tests` executable in `tests/CMakeLists.txt`; CTest registers `gsl_tests` at line 221.

| Exact consumer blob | Size | Helper include line | Matcher call lines | Build target |
| --- | ---: | ---: | ---: | --- |
| `tests/algorithm_tests.cpp@e369be0e34dcf786a63631807ebe3cc06fb59544` | 6404 bytes | 23 | 207 | `gsl_tests` |
| `tests/assertion_tests.cpp@231befc7e415ec53daa09ced61d76505e6be5437` | 1593 bytes | 17 | 48, 59 | `gsl_tests` |
| `tests/at_tests.cpp@42b2241bfd472ab384c35267d2042c870edce4a5` | 5604 bytes | 30 | 47, 70, 93, 115, 142 | `gsl_tests` |
| `tests/notnull_tests.cpp@df8e5a9acf529a9da9e578da413e955900e91e7d` | 18407 bytes | 29 | 165, 300, 485, 583 | `gsl_tests` |
| `tests/span_ext_tests.cpp@dfd2725db3612de97c4a3173f5f168131f7341f3` | 11004 bytes | 30 | 38, 213 | `gsl_tests` |
| `tests/span_tests.cpp@d86458bd3bd8e5a1f1739a769436594de2a6b85a` | 36875 bytes | 47 | 121, 169, 655, 778, 822 | `gsl_tests` |
| `tests/strict_notnull_tests.cpp@c1b1f54c97abe1c2cf825e81e713bee2fb8a888f` | 7437 bytes | 20 | 225 | `gsl_tests` |

The helper is not a standalone executable and has no product entrypoint. The separate `tests/no_exception_ensure_tests.cpp@5fde41cf2f307830c07d592819c7baed5d59d40d` (1,644 bytes) is registered as `gsl_noexcept_tests` in the same CMake file (lines 312-317) and uses the no-exception compiler configuration (`-fno-exceptions` on the non-MSVC path and `_HAS_EXCEPTIONS=0` for MSVC). It does **not** include this helper and remains the separate strict165 source responsibility.

## tdesktop direct-consumer boundary

Exact-symbol GitHub code searches on accepted `telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` returned zero matches for both `deathTestCommon.h` and `GetExpectedDeathString`: `tdesktop direct consumer = 0`. The seven consumers above belong to the pinned GSL dependency's test harness, not tdesktop production code. This row closes only the helper's exact source/test provenance; it does not close any GSL production header or tdesktop product responsibility.

## Existing-owner-first and fail-closed disposition

- Reuse the existing TDRP source/dependency inventory and test-provenance owner. No new runtime owner is warranted.
- Unique target symbol: `TDRP_GSL_DEATH_TEST_COMMON_PROVENANCE_001`, scoped to this source dossier only.
- Do not add `GSLDeathTestRuntime`, a global no-exception runtime, a global span runtime, a second assertion/termination owner, or any product composition slot.
- Preserve exact MIT provenance and CMake registration. If the GSL gitlink/tree, helper blob, consumer set, CMake registration, or license changes, re-read exact refs before reusing this record.
- The source-only row has no production entrypoint or production evidence. It remains `mapped-open`; source closure and independent release acceptance remain open.

## Traceability

Requirement: `TDRP-R9-GSL-DEATH-TEST-COMMON-TEST-PROVENANCE-001`.

Oracles: `ORA-TDRP-GSL-DEATH-TEST-COMMON-EXACT-SOURCE-001`, `ORA-TDRP-GSL-DEATH-TEST-CONSUMER-SET-001`, `ORA-TDRP-GSL-DEATH-TEST-COMMON-NO-DIRECT-TDESKTOP-CONSUMER-001`, `ORA-TDRP-GSL-DEATH-TEST-CMAKE-REGISTRATION-001`.

Invariants: `INV-TDRP-GSL-DEATH-TEST-COMMON-TESTS-ONLY-001`, `INV-TDRP-GSL-DEATH-TEST-COMMON-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-DEATH-TEST-COMMON-NO-BULK-CALLER-CLOSURE-001`.

Counters before strict164: source entries 16,379; unknown 16,213; unread 16,188; omitted 0; responsibilities 181. After this one source entry is accounted: source entries remain 16,379; unknown 16,212; unread 16,187; omitted 0; responsibilities 182. `baseline_identity_accepted=false`, `source_closure_ready=false`, and independent release acceptance remain open.

Next: strict165 `Microsoft/GSL tests/no_exception_ensure_tests.cpp@5fde41cf2f307830c07d592819c7baed5d59d40d`, as a separate test-provenance row and without a global no-exception/span runtime.
