# Microsoft/GSL tests/at_tests.cpp complete read — strict162

## Authority
- Live main fb1189efd9d85c650864d6987ea8fb68c913558d/1a1ef09be4c9fd801354439a5b82cc460f0b4b6a; upstream dev a7f453612f4896ac04d0007a36ab0d377e5bcbcf/c539eabfb05e7dcaa84821ac44c35cdfb6551ae5; GSL 87f9d768866548b5b86e72be66c60c5abd4d9b37/6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- tests/at_tests.cpp@42b2241bfd472ab384c35267d2042c870edce4a5, fully read: 174 line slots / 5604 bytes.
- include/gsl/util@26b2f5fb9d0d8c28c3096f50d575eb42a4b89bad, fully read: 172 line slots / 5222 bytes.
- tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b, fully read: 12 line slots / 380 bytes.
- tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a registers at_tests.cpp.
- LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02 confirms MIT.
- Parent exact-head 760f674d7e4f3dabbd40721481133b890d14f24a: TDRP 40053896937 completed 61/61 success; authority 12544532413 sha256:77aba723f1859c68608c5f34c5660f6c1c17882a2465dd8d847c89bdcf102700; closure 12544337666 sha256:7cfbfae914e3041d9ab8c618f4d694abdf4893315ab6d5b8c12c1718b30f73f0; reachability 12544557978 sha256:274037a3db36d9aa539e56a17e616cc2d653303567cc19bc2148d93d2e514ce5; Rust 40053896993 completed 6/6 success; artifact 12544806414 sha256:4a659ee10c2c40e1348f93438c3777f2a079d6f97dd722c21e2972483fd83134.
- Superseded strict161 publication e35c414440048cdecf4489df72f0c5df919f9074: TDRP 40053570607 completed failure with 58 success / 1 failure / 2 skipped; Source authority job 120751406554 failed exactly because strict161 reused the strict160 target symbol without an explicit split. Rust 40053570606 completed cancelled with 5 success / 1 cancelled. Repair commit 760f674d7e4f3dabbd40721481133b890d14f24a changed only parity-ledger.json to a unique target symbol; fresh exact-head TDRP/Rust then became fully green.
- Accepted tdesktop dev code search returned zero direct `gsl::at` and zero `using gsl::at` consumers. Two generic `<gsl/util>` includes exist, but neither establishes an at caller.
- Next: Microsoft/GSL tests/byte_tests.cpp@634dcc90773e7144a127e29a2d49ce94b816e596.

## Exact behavior
`gsl::at` is a bounds-checked indexing helper. Built-in arrays constrain the extent to a `ptrdiff_t`-representable range and require `i >= 0 && i < N` before signed-to-size conversion. Generic containers, initializer lists and optional C++20 `std::span` use the same lower/upper-bound contract.

The tests verify element identity for mutable/const built-in arrays, `std::array`, `std::vector`, and available `std::span`; initializer lists verify selected values. Every family rejects `-1` and the exact end boundary through contract death. The constexpr test proves the covered built-in array, const std::array and initializer-list accesses can participate in constant evaluation on supported compilers.

## Existing-owner-first
No `GSLAtRuntime`, global index service, second container model or compatibility utility layer is introduced. Strict162 belongs to the existing TDRP source/dependency inventory and test-provenance owner. Any independently discovered indexed-access caller keeps bounds, state and failure semantics in its actual existing Rust/domain owner.

Accepted tdesktop has zero direct `gsl::at` consumers. The two generic `<gsl/util>` includes cannot be converted into caller closure because that header also carries unrelated helpers.

## Boundary and fail-closed rules
- Validate signed index before unsigned/size-type conversion.
- `index == size` is outside the valid half-open range.
- Do not substitute unchecked indexing for a real caller's required checked access.
- Generic util-header inclusion is not evidence of `gsl::at` use.
- The helper/test pair does not close future callers or product capabilities transitively.
- Any accepted tdesktop/GSL gitlink, child tree, at test/util header/license/build-registration or consumer-set drift requires fresh reconciliation.
- Historical/source test evidence does not substitute for exact-head production/release acceptance.

## Traceability
Requirement TDRP-R9-GSL-AT-BOUNDS-CHECK-TEST-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-AT-EXACT-SOURCE-001, ORA-TDRP-GSL-AT-VALID-IDENTITY-001, ORA-TDRP-GSL-AT-BOUNDS-DEATH-001, ORA-TDRP-GSL-AT-NO-DIRECT-CONSUMER-001.
Invariants: INV-TDRP-GSL-AT-CHECK-BEFORE-CONVERSION-001, INV-TDRP-GSL-AT-END-EXCLUSIVE-001, INV-TDRP-GSL-AT-NO-PARALLEL-RUNTIME-001.
Counters after strict162: unknown=16,214 unread=16,189 omitted=0 responsibilities=180.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
