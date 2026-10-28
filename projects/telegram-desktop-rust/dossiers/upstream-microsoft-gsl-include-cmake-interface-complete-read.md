# Microsoft/GSL include/CMakeLists.txt complete read — strict148

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- include/CMakeLists.txt@fe4eed9190a40c3fc0b182112956a4957653a5f2, fully read: 14 line slots / 484 bytes
- Parent exact head 215707ebf06ba9cbc5314ff3e5f597deca92375a, tree 6c2044e3a617c37ad441efd745369fdfd7e20658
- Parent TDRP 40030895593 61/61 success; authority 12535136603 sha256:dbe2c3f6d216dc94011571058041b87768d60035feab3d5cc824865113127c7b; closure 12535610595 sha256:2104931dd254e649f35d993372e01659a150f27a1133699cb376238dcf2e5edf; reachability 12535182047 sha256:9b71cc604aa6e0c39ed086ed08b92fdb7ad68b172f264de1862e218c47cc62fc; Rust 40030895592 6/6 success; artifact 12535266019 sha256:99a035190c55154fa7cec5595b532c0102f1b30407d23818877049f702a53f14
- Next: Microsoft/GSL include/gsl/algorithm@584f5cd12beb15e863b1c32bc887638314261399

## Exact source and disposition
The locked build definition exposes the GSL header directory as an INTERFACE BUILD_INTERFACE include path when GSL is the top-level project. When consumed as a nested dependency, it marks the same path SYSTEM INTERFACE so dependency warnings are suppressed for consumers. The directory layout intentionally enables namespaced includes such as <gsl/gsl>.

This is dependency/build-interface provenance. Fabushi must preserve the corresponding engineering guarantees through its existing native build/package/dependency system, not by importing a second GSL/CMake graph.

## Existing-owner-first
Reuse TDRP-SRC-02 build/dependency authority accounting, the canonical Fabushi build/test/package/update chain, and module-map build/generator/resources/dependencies ownership. No GSL build root, package graph, dependency runtime or source-branded toolchain owner is introduced.

## Fail-closed
SYSTEM warning suppression is not security or quality acceptance, and build-interface documentation cannot close any include/gsl implementation behavior. Every implementation header remains separately readable/mappable and exact-head CI remains required after publication.

## Traceability
Requirement TDRP-R9-GSL-INCLUDE-CMAKE-INTERFACE-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-INCLUDE-CMAKE-EXACT-SOURCE-001, ORA-TDRP-GSL-INCLUDE-CMAKE-TOPLEVEL-NESTED-001, ORA-TDRP-GSL-INCLUDE-CMAKE-SYSTEM-ISOLATION-001, ORA-TDRP-GSL-INCLUDE-CMAKE-NO-PARALLEL-BUILD-ROOT-001.
Invariants: INV-TDRP-GSL-INCLUDE-CMAKE-NO-OMISSION-001, INV-TDRP-GSL-INCLUDE-CMAKE-EXISTING-OWNER-FIRST-001, INV-TDRP-GSL-INCLUDE-CMAKE-NO-TRANSITIVE-CLOSURE-001, INV-TDRP-GSL-INCLUDE-CMAKE-EXACT-PROVENANCE-001.
Counters after strict148: unknown=16,228 unread=16,203 omitted=0 responsibilities=166.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
