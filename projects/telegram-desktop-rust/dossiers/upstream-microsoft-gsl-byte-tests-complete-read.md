# Microsoft/GSL tests/byte_tests.cpp complete read — strict163

## Authority
- Live Fabushi main: fb1189efd9d85c650864d6987ea8fb68c913558d, tree 1a1ef09be4c9fd801354439a5b82cc460f0b4b6a.
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- tests/byte_tests.cpp@634dcc90773e7144a127e29a2d49ce94b816e596, fully read: 135 line slots / 3518 bytes.
- include/gsl/byte@87d007ef94d197632872e96d91525a013c665da8, fully read: 184 line slots / 5962 bytes.
- tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a registers byte_tests.cpp.
- LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02 confirms Microsoft/GSL MIT licensing.
- Parent exact-head c5c8f81c1dea6d77f90b9f1e0ab31749bf08337a: TDRP 40054914813 completed 61/61 success; authority 12544539280 sha256:8469d3852256dc02e0b38567f6091677eb0f54a9ab8af3778016fef3919621c0; closure 12545345565 sha256:ac23b2c5616a1927b8fecd1f91468b5bc104c70e3acef937bce1ea3410e62ff0; reachability 12544669389 sha256:306da3736447347f18a0ea8c3e7ebbfd8352eb32615c819969ecab4f4be26154; Rust 40054914790 completed 6/6 success; artifact 12544587734 sha256:acd60a3e3a141eb67293204b27b873f820441e11f775e3abe26e0e3a2ad34b81.
- Existing strict151 byte-representation dossier already accounts for the header implementation; strict163 accounts the standalone test-source entry and does not create a second byte owner.
- Accepted tdesktop dev code search is complete (incomplete_results=false): gsl::byte matches 10 files; gsl::to_integer< matches 4 files; to_byte( matches 0 files; the union is 13 production files because calls/calls_call.cpp appears in both sets. Exact consumers span UI effects, MTProto auth/RSA/DC-key/socket paths, Passport encryption, Calls/fingerprints, Storage key derivation and crash reserved memory. This dependency-test row closes none of them.
- Next: Microsoft/GSL tests/deathTestCommon.h@7bf242393f7bcff62c12cd985a87824f48f7bb0b.

## Exact behavior
The locked header prefers `std::byte` and `std::to_integer` when the compiler/library exposes them. Otherwise it defines an unsigned-char-backed byte type with compiler may-alias support where available, integral shift and bitwise operators, integral-only `to_integer`, runtime `to_byte(T)` restricted by static assertion to `unsigned char`, and compile-time `to_byte<I>()` restricted to 0..255.

The four focused tests cover construction, bitwise/shift operations, integral conversion and aliasing. Construction proves explicit byte formation and both compile-time and unsigned-char runtime `to_byte`; guarded compile-error cases reject char/int/unsigned-int runtime inputs. Bitwise tests cover OR/AND/XOR/not and left/right shift, including compound assignment. `to_integer` is exercised across signed and unsigned integral widths while floating-point conversion is intentionally outside the contract. The aliasing test qualifies byte access to object representation without defining endianness, cryptographic semantics or durable format.

## Existing-owner-first
Fabushi does not need a `GSLByteTestRuntime`, generic binary-data root or source-branded buffer subsystem. Strict163 reuses the strict151 native byte-representation mapping and the existing TDRP source/dependency test-provenance owner.

The accepted tdesktop consumer set spans UI effects, MTProto auth/RSA/DC-key/socket paths, Passport encryption, Calls/fingerprints, Storage key derivation and crash reserved memory. Those are unrelated product responsibilities that keep their actual existing Fabushi owner. Rust `u8`, arrays, slices, vectors or narrow domain wrappers are implementation tools inside those owners, not a cross-domain state owner.

## Boundary and fail-closed rules
- Unsupported runtime conversion must not become an implicit truncating cast.
- Compile-time byte constants outside 0..255 must remain rejected.
- Byte alias qualification does not establish endianness, secrecy, zeroization, authorization, cryptographic validity or protocol framing.
- The 13 unique observed production consumers remain independently open; no helper/test row bulk-closes them.
- Any accepted tdesktop/GSL gitlink, child tree, byte test/header/license/build-registration or production consumer-set drift requires fresh reconciliation.
- No source test or historical artifact substitutes for exact-head production/release acceptance.

## Traceability
Requirement TDRP-R9-GSL-BYTE-CONTRACT-TEST-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-BYTE-TEST-EXACT-SOURCE-001, ORA-TDRP-GSL-BYTE-TEST-CONVERSION-001, ORA-TDRP-GSL-BYTE-TEST-BITWISE-001, ORA-TDRP-GSL-BYTE-TEST-CALLER-SCOPE-001.
Invariants: INV-TDRP-GSL-BYTE-TEST-NO-IMPLICIT-TRUNCATION-001, INV-TDRP-GSL-BYTE-TEST-NO-PARALLEL-RUNTIME-001, INV-TDRP-GSL-BYTE-TEST-NO-BULK-CALLER-CLOSURE-001.
Counters after strict163: unknown=16,213 unread=16,188 omitted=0 responsibilities=181.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
