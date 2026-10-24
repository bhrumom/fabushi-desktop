# Microsoft/GSL include/gsl/algorithm complete read — strict149

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/gsl/algorithm@584f5cd12beb15e863b1c32bc887638314261399, fully read: 64 line slots / 2339 bytes
- Parent exact head 5ad8f77a764c4e1abf25b1549e8acb87533aad90, tree fb6e44473dae77a3e01b5799cd1e8c09eff95c09
- Parent TDRP 40032422818 61/61 success; authority 12535009130 sha256:52f5f920d522ea61db5ecd3dd6a37ef915374b5a187ada4f8c4b86d823fcd42f; closure 12535502594 sha256:9efd7ef2c935a1525b39f8327993191cf6211d1f33f2dd6984f94cef2f2b8050; reachability 12535588315 sha256:77d55bddd4361677d0d5cd096378d1f26ff23d8684aa28699e996a2dc8ef3465; Rust 40032422966 6/6 success; artifact 12534964298 sha256:7ee3bfd1cac74e6864184cbf4af4d70abb27c9bd292d4c292cb0b38869599cdc
- Accepted tdesktop exact-head code search returned no direct gsl::copy callsite.
- Next: Microsoft/GSL include/gsl/assert@d3e6ccdd53d5a254ed3827cf07711a6e86bb4226

## Exact behavior
gsl::copy accepts source and destination spans, requires element assignability at compile time, rejects incompatible fixed extents at compile time, enforces dest.size() >= src.size() through Expects, suppresses a known STL analysis warning, and copies exactly src.size() elements with std::copy_n.

## Existing-owner-first and language disposition
This is a generic low-level recursive dependency utility, not a product-specific owner. Reuse TDRP-LANG-03 dependency replacement and native Rust type/slice/collection safety at actual Fabushi callsites. Do not add GSL/C++, a copy subsystem, or a second memory-safety owner. The source's template/pragma mechanics are not target architecture.

## Fail-closed
Dependency replacement must preserve type compatibility, capacity/bounds and exact-count semantics wherever this responsibility is applicable. This row does not close domain callsites, other GSL headers, source closure, security acceptance or release acceptance.

## Traceability
Requirement TDRP-R9-GSL-ALGORITHM-SAFE-SPAN-COPY-DEPENDENCY-001.
Oracles: ORA-TDRP-GSL-ALGORITHM-EXACT-SOURCE-001, ORA-TDRP-GSL-ALGORITHM-TYPE-EXTENT-001, ORA-TDRP-GSL-ALGORITHM-RUNTIME-CAPACITY-001, ORA-TDRP-GSL-ALGORITHM-NATIVE-REPLACEMENT-001.
Invariants: INV-TDRP-GSL-ALGORITHM-NO-OMISSION-001, INV-TDRP-GSL-ALGORITHM-NO-GSL-RUNTIME-001, INV-TDRP-GSL-ALGORITHM-EXACT-COUNT-001, INV-TDRP-GSL-ALGORITHM-EXACT-PROVENANCE-001.
Counters after strict149: unknown=16,227 unread=16,202 omitted=0 responsibilities=167.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
