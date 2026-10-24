# Microsoft/GSL tests/assertion_tests.cpp complete read — strict161

## Authority
- Live Fabushi main: fb1189efd9d85c650864d6987ea8fb68c913558d, tree 1a1ef09be4c9fd801354439a5b82cc460f0b4b6a.
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- tests/assertion_tests.cpp@231befc7e415ec53daa09ced61d76505e6be5437, fully read: 61 line slots / 1593 bytes.
- include/gsl/assert@d3e6ccdd53d5a254ed3827cf07711a6e86bb4226, fully read: 134 line slots / 3988 bytes.
- Supporting tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b, fully read: 12 line slots / 380 bytes.
- tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a includes assertion_tests.cpp in the GSL test source list.
- LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02 confirms Microsoft/GSL MIT licensing.
- Parent exact-head 5d72367fcf86a923e8a89c2568dc04b9e6eaa2ec: TDRP 40052761319 completed 61/61 success; authority 12543898833 sha256:395e1b9b93bf1ea56d59ffebaa0c3f208c58965960ce20389d25f9144541e51c; closure 12543919318 sha256:d52234600cc976f6d3911f7ac266f40c1dd8843f24d0588041dcf07093659eb6; reachability 12543953943 sha256:9d641d5636ec50e124991edbd18b2f7cf52755de3aed9444b5b98366e10efffb; Rust 40052761294 completed 6/6 success; artifact 12544086580 sha256:886f9bc2a3852b91e08c71b9011f266ce031575530e2f890bc86aff77afae5c0.
- Accepted tdesktop current dev code search reports 487 files matching `Expects(` and 44 files matching `Ensures(`; representative production consumers span account/domain, mtproto/session, storage/download, data/credits, passport/encryption and UI/layout paths.
- tdesktop repository code search reports zero direct `gsl/assert` and zero `GSL_CONTRACT_CHECK` matches; the macro implementation remains in the separately locked GSL child dependency.
- Next: Microsoft/GSL tests/at_tests.cpp@42b2241bfd472ab384c35267d2042c870edce4a5.

## Exact behavior
`Expects(cond)` and `Ensures(cond)` are semantic precondition/postcondition names over the same `GSL_CONTRACT_CHECK` primitive. The check biases the true branch with `GSL_LIKELY`; a true condition is a no-op. A false condition calls `gsl::details::terminate()`, which is `[[noreturn]]` and `noexcept`. In the ordinary configuration that calls `std::terminate()`. For the explicit MSVC no-exceptions/kernel configuration, the header installs a replacement terminate-handler path whose default ends in `__fastfail`.

The tests define `f(i)` with `Expects(i > 0 && i < 10)` and prove `f(2) == 2` while `f(10)` dies. They define `g(i)`, increment first, then `Ensures(i > 0 && i < 10)`; the suite proves `g(2) == 3` while `g(9)` dies after producing 10. The death tests temporarily install a terminate handler so the abnormal path is deterministic and observable.

The header also defines compiler-facing suppression, likely/unlikely and `GSL_ASSUME` helpers. Those are not additional product state or validation owners. In particular, `GSL_ASSUME` is an optimizer contract and must not be substituted for a checked runtime pre/postcondition.

## Existing-owner-first
Fabushi does not need a `GSLAssertionRuntime`, global assertion service, second validation model or compatibility macro subsystem. Strict161 belongs to the existing TDRP source/dependency inventory and test-provenance owner.

Unlike the prior zero-consumer helper, accepted tdesktop uses `Expects`/`Ensures` broadly in production source. That means the helper/test pair cannot close those callers transitively. Every real condition must be read at its own source file/symbol, classified as that caller's precondition/postcondition responsibility, and migrated into the actual existing Fabushi domain owner with its state, lifecycle, error/recovery and security context intact. A caller-specific Rust implementation must not weaken a required fail-closed invariant into a debug-only check, but this dependency row does not dictate one process-wide replacement mechanism.

## Boundary and fail-closed rules
- A false `Expects` or `Ensures` condition must never proceed as successful execution.
- `GSL_ASSUME` or compiler branch hints are not runtime validation substitutes.
- The ordinary death tests do not by themselves prove every MSVC no-exceptions/kernel execution path; platform-specific caller obligations remain independently open.
- This helper/test provenance does not bulk-close the 487 `Expects(`-matching files, the 44 `Ensures(`-matching files, or any product capability that contains them.
- Each actual caller keeps its existing-owner-first mapping; no global validation owner is introduced.
- Any accepted tdesktop/GSL gitlink, child tree, assertion test/header/license/build-registration or production macro-consumer-set drift requires fresh reconciliation.
- No source test, helper contract or historical artifact substitutes for exact-head production/release acceptance.

## Traceability
Requirement TDRP-R9-GSL-ASSERTION-CONTRACT-TEST-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-ASSERTION-EXACT-SOURCE-001, ORA-TDRP-GSL-ASSERTION-PRECONDITION-DEATH-001, ORA-TDRP-GSL-ASSERTION-POSTCONDITION-DEATH-001, ORA-TDRP-GSL-ASSERTION-CALLER-SCOPE-001.
Invariants: INV-TDRP-GSL-ASSERTION-FALSE-FAILS-CLOSED-001, INV-TDRP-GSL-ASSERTION-NO-GLOBAL-RUNTIME-001, INV-TDRP-GSL-ASSERTION-NO-BULK-CALLER-CLOSURE-001.
Counters after strict161: unknown=16,215 unread=16,190 omitted=0 responsibilities=179.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
