# Microsoft/GSL include/gsl/narrow complete read — strict153

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/gsl/narrow@7b1a7912163185e2b2cbecc34ad64a539d566173, fully read: 83 line slots / 3408 bytes
- tests/utils_tests.cpp@5b64e207eda9ef0e8935701647d6d4bed567960e, fully read: 166 line slots / 4211 bytes
- Parent exact head c7fa56b4f39289fc96af421fe5c083d29d6be3d4, tree 14dc20a036aa83d5b8c9ead28d84c5bef09138bd
- Parent TDRP 40039556310 61/61 success; authority 12539230287 sha256:1a8ac6428c98b6588082c37ca2121bf731c318f942fb8557ba981b519761b708; closure 12538956659 sha256:1a74b1df6d9bc8d379d834577e34a79d1c2c9e35609329f735b11c5bc3d6ceb5; reachability 12538732607 sha256:131bc8a8ef338cea7827b09a68bd68ad15e3f9e174a597297fb78ac7252fd88c; Rust 40039556304 6/6 success; artifact 12538885653 sha256:7a606bb1d4e9ecb5015670296e88914873c5d0b109bb34533310b4aa9f30d9a2.
- Accepted exact-head tdesktop code search found no direct gsl::narrow, narrow_cast or narrow<...> use.
- Next: Microsoft/GSL include/gsl/pointers@63e193e92478632520eca7e872ac83c643ffd2c0.

## Exact behavior
narrow<T>(u) first performs narrow_cast<T>(u), then casts the result back and rejects any value loss. Arithmetic conversions additionally reject sign changes when source and destination signedness differ. NaN necessarily fails the equality check. The source failure type is narrowing_error. The focused tests distinguish unchecked narrow_cast truncation from checked narrow failure, and cover signed/unsigned boundaries plus lossy complex conversion.

No direct accepted tdesktop callsite was found. That means this recursive dependency entry has no proven product caller at this exact source boundary; it does not mean the source item can be omitted or that future/indirect callers are closed.

## Existing-owner-first
Reuse TDRP-LANG-03 and each actual Rust/domain validation boundary. Use TryFrom/TryInto, Result, checked ranges or explicit sign validation. Do not add a GSL exception runtime, generic conversion service or second validation truth.

## Fail-closed
This row closes only the generic checked-narrowing dependency responsibility. It does not close util, numeric domain validation, protocol/storage/crypto semantics, future/indirect callers, later GSL headers, source closure or release acceptance. Silent `as` truncation is not an equivalent substitute where checked narrowing semantics apply.

## Traceability
Requirement TDRP-R9-GSL-CHECKED-NARROWING-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-NARROW-EXACT-SOURCE-001, ORA-TDRP-GSL-NARROW-ROUNDTRIP-001, ORA-TDRP-GSL-NARROW-SIGNEDNESS-001, ORA-TDRP-GSL-NARROW-NATIVE-ERROR-001.
Invariants: INV-TDRP-GSL-NARROW-NO-LOSSY-SUCCESS-001, INV-TDRP-GSL-NARROW-NO-GSL-EXCEPTION-RUNTIME-001, INV-TDRP-GSL-NARROW-NO-DUPLICATE-VALIDATION-OWNER-001, INV-TDRP-GSL-NARROW-EXACT-PROVENANCE-001.
Counters after strict153: unknown=16,223 unread=16,198 omitted=0 responsibilities=171.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
