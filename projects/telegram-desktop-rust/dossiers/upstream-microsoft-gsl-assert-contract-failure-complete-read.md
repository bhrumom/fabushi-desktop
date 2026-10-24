# Microsoft/GSL include/gsl/assert complete read — strict150

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/gsl/assert@d3e6ccdd53d5a254ed3827cf07711a6e86bb4226, fully read: 134 line slots / 3988 bytes
- Parent exact head a8d5c1f797cac10749db1608743d2eba13d5a139, tree 255b122d1358d7d4c0c4a833bcc542ed6b059989
- Parent TDRP 40034178153 61/61 success; authority 12535384410 sha256:c96eef6b8937dd374dc901728030841e78a309e57a23c56c73296063c62e20e6; closure 12537015768 sha256:f8ac01dba730448bfbea88979d1b5f9f91616cc2564fcfd2ae36a0a7c2e8bd94; reachability 12536726927 sha256:48efbf7c3b63a8ce8ad2d43e786238ac02a47fd05d0c745cad2e642e07ddec20; Rust 40034178058 6/6 success; artifact 12536755908 sha256:ccdf141f91d920d6a993a4982b58b41a23627340870e21dc93419e225c5c58aa
- GSL focused tests read: tests/assertion_tests.cpp@231befc7e415ec53daa09ced61d76505e6be5437 (61 line slots / 1593 bytes); tests/no_exception_ensure_tests.cpp@5fde41cf2f307830c07d592819c7baed5d59d40d (56 line slots / 1644 bytes); tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b (12 line slots / 380 bytes)
- tdesktop integration override: Telegram/SourceFiles/stdafx.h@252ab2c2aa698e3337894ab6d50acef697789036 includes base/assertion.h before <gsl/gsl>; Telegram/lib_base@95a543e426e35938dd632139d0bb828b268c3b4c base/assertion.h@05194235168d883f34ce7da9972856d824bf1f1f redefines Expects/Ensures to log, crash for a report and abort as fallback
- Next: Microsoft/GSL include/gsl/byte@87d007ef94d197632872e96d91525a013c665da8

## Exact behavior
The locked GSL header provides compiler-specific suppression, likely/unlikely and assume helpers; a special MSVC kernel/no-exceptions termination workaround; gsl::details::terminate; and Expects/Ensures contract checks that do not permit execution to continue after a failed contract. The focused tests prove normal pre/postconditions and death on violation, including the no-exceptions path.

tdesktop deliberately changes the observable failure path: its precompiled-header composition loads desktop-app/lib_base assertion support before the aggregate GSL header, and that owner undefines/redefines Expects/Ensures so failed contracts are logged with file/line, deliberately crash to generate a crash report, and abort as a fallback. This integration fact is source authority, not a reason to reproduce the C++ deliberate-null-write mechanism.

## Existing-owner-first
Reuse TDRP-LANG-03 low-level dependency replacement plus the canonical Fabushi behavior/fault-recovery contracts and each actual Rust domain owner's validation/invariant/error/diagnostic lifecycle. Recoverable external input is validated by its domain contract before an invariant boundary; an impossible internal invariant breach remains fail-closed and diagnosable. Do not add a GSL/assert runtime, C++ macro layer, parallel crash subsystem or second error owner.

## Fail-closed
This row closes only the GSL assert dependency implementation. It does not close Telegram/lib_base/base/assertion.h as its own recursive source entry, any domain caller, byte/narrow/pointers/span/util headers, crash/recovery acceptance, or release acceptance. Optimizer GSL_ASSUME is not a substitute for a runtime validation boundary, and the source's deliberate crash technique is not target architecture.

## Traceability
Requirement TDRP-R9-GSL-ASSERT-CONTRACT-FAILURE-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-ASSERT-EXACT-SOURCE-001, ORA-TDRP-GSL-ASSERT-FAIL-CLOSED-001, ORA-TDRP-GSL-ASSERT-TDESKTOP-OVERRIDE-001, ORA-TDRP-GSL-ASSERT-NATIVE-REPLACEMENT-001.
Invariants: INV-TDRP-GSL-ASSERT-NO-OMISSION-001, INV-TDRP-GSL-ASSERT-NO-GSL-RUNTIME-001, INV-TDRP-GSL-ASSERT-DIAGNOSTIC-NONCONTINUATION-001, INV-TDRP-GSL-ASSERT-EXACT-PROVENANCE-001.
Counters after strict150: unknown=16,226 unread=16,201 omitted=0 responsibilities=168.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
