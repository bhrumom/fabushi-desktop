# Microsoft/GSL include/gsl/gsl complete read — strict152

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/gsl/gsl@53030b31747ca25e3d740bd078c0ffd5b71ebf88, fully read: 35 line slots / 1217 bytes
- Parent exact head ce7e372b3bd12e53dcf23b4f7d901e8c371b1703, tree ad4e3ffa7fb0950f049692a4c7e22cec604d6d24
- Parent TDRP 40037884109 61/61 success; authority 12538530472 sha256:aab487be9d0492a8c92ae4ee5f6039dcb22ab21e54e135f3e86d7df360dc9eb7; closure 12538371757 sha256:ea1d81a43aaec065300a9e37c1098de95eaf13039ab7d5ab1048a184e331aad5; reachability 12538327095 sha256:bcc886f931b61eadf35337ca01d8c1e16113c5b4cc6c259db817fbafb83f7e23; Rust 40037884162 6/6 success; artifact 12538311273 sha256:78712bec6ca706d669f7ea1e654f00704779dd29de8d529a17a862126bc6c16b.
- Accepted exact-head direct <gsl/gsl> consumers: Telegram/SourceFiles/stdafx.h, Telegram/SourceFiles/mtproto/core_types.h, Telegram/SourceFiles/ui/passcode_strength.cpp.
- Next: Microsoft/GSL include/gsl/narrow@7b1a7912163185e2b2cbecc34ad64a539d566173.

## Exact behavior
The locked header is an aggregation/export boundary, not a runtime subsystem. It exports algorithm, assert, byte, pointers, span, zstring and util unconditionally, and exports narrow only when __cpp_exceptions is defined. Its include guard and IWYU export pragmas are C++ integration mechanics.

The accepted tdesktop commit includes this umbrella directly from the shared stdafx path and two specific translation units. That broad visibility does not make the umbrella the owner of any constituent algorithm, assertion, byte, pointer, span, string, utility, checked-conversion or downstream product behavior.

## Existing-owner-first
Reuse TDRP-LANG-03 dependency replacement, existing Fabushi build/module dependency boundaries, and each actual constituent/domain owner. Native Rust or existing modules import the capabilities they need. Do not add a GSL umbrella runtime, second dependency root, parallel build graph, or source-branded compatibility subsystem.

## Fail-closed
This row closes only the umbrella/export dependency responsibility. It does not close algorithm/assert/byte/narrow/pointers/span/zstring/util, stdafx transitive consumers, MTProto behavior, passcode-strength behavior, later GSL files, source closure, or release acceptance. Exception-sensitive checked narrowing must remain an explicit target error-model decision at the real owner.

## Traceability
Requirement TDRP-R9-GSL-UMBRELLA-EXPORT-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-UMBRELLA-EXACT-SOURCE-001, ORA-TDRP-GSL-UMBRELLA-EXPORT-BOUNDARY-001, ORA-TDRP-GSL-UMBRELLA-EXCEPTION-NARROW-001, ORA-TDRP-GSL-UMBRELLA-NATIVE-REPLACEMENT-001.
Invariants: INV-TDRP-GSL-UMBRELLA-NO-OMISSION-001, INV-TDRP-GSL-UMBRELLA-NO-GSL-RUNTIME-001, INV-TDRP-GSL-UMBRELLA-NO-CONSTITUENT-CLOSURE-001, INV-TDRP-GSL-UMBRELLA-EXACT-PROVENANCE-001.
Counters after strict152: unknown=16,224 unread=16,199 omitted=0 responsibilities=170.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
