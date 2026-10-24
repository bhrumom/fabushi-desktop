# Microsoft/GSL docs/headers.md complete read — strict147

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- docs/headers.md@11dd0cdfd5840cbcb7d8538e49efa2ccd5ac6a68, fully read: 861 line slots / 35147 bytes
- Parent exact head 21aa2a6c633245efef91442416a258d722b72b4d, tree 35006931a3e084e77d4477e10e33d8214c7e41f9
- Parent TDRP 40029290433 61/61 success; authority 12533987923 sha256:f0c3941c88f811557891ceba97d52171de15eceeeac45cab636c61dad9621372; closure 12534656490 sha256:364939e0f0ed335223e671177680eed84abdfc3b1906d33949e44033571057d0; reachability 12534401635 sha256:a46e5272413e3a55b6cc4da2be3ae320b78bba64c31efa52154c995445777755; Rust 40029290426 6/6 success; artifact 12534755123 sha256:b3c3bb865feae2d24a65cc8c23bb5d525270c465f89146ad76906f93f65892ad
- Next: Microsoft/GSL include/CMakeLists.txt@fe4eed9190a40c3fc0b182112956a4957653a5f2

## Exact source and disposition
The locked document is the complete public-header contract guide for GSL. It covers algorithms/copy destination-size preconditions; contract macros where failed Expects/Ensures terminate; byte representation and GSL_USE_STD_BYTE cross-standard configuration; the exception-gated aggregate <gsl>; throwing narrow/narrowing_error versus non-throwing narrow_cast; owner/not_null/strict_not_null ownership and non-null constraints including deleted pointer arithmetic/indexing; span as a non-owning fixed/dynamic-extent view with strict runtime bounds checks and terminating violations; span_ext dynamic_extent/comparisons/make_span/at/ssize/iterators; zero-terminated string annotations; final_action/finally scope-exit cleanup; and bounds-checked at overloads.

These are behavior and failure-semantics source contracts, not proof of Fabushi implementation. The later include/gsl files remain the executable source authority that must be read and mapped independently.

## Existing-owner-first
Reuse TDRP-SRC-03 documentation/dependency provenance, TDRP-MOD-01 responsibility decomposition, module-map file-level traceability, and the existing native Fabushi Rust/TypeScript/platform owners when each executable behavior is later implemented or proved equivalent. Do not add a GSL runtime, C++ compatibility layer, second memory-safety subsystem, or parallel dependency graph.

## Fail-closed
The documentation row cannot close include/CMakeLists.txt, any include/gsl implementation, tests, packaged dependency evidence, or runtime/release acceptance. In particular, C++ std::terminate/exception/annotation mechanisms are source semantics to translate through native Fabushi owners and tests, not architecture to preserve verbatim.

## Traceability
Requirement TDRP-R9-GSL-HEADERS-API-BEHAVIOR-DOCUMENTATION-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-HEADERS-EXACT-SOURCE-001, ORA-TDRP-GSL-HEADERS-FAILURE-SEMANTICS-001, ORA-TDRP-GSL-HEADERS-OWNERSHIP-BOUNDS-001, ORA-TDRP-GSL-HEADERS-DOCUMENTATION-NOT-IMPLEMENTATION-001.
Invariants: INV-TDRP-GSL-HEADERS-NO-OMISSION-001, INV-TDRP-GSL-HEADERS-NO-RUNTIME-INFERENCE-001, INV-TDRP-GSL-HEADERS-NO-TRANSITIVE-CLOSURE-001, INV-TDRP-GSL-HEADERS-EXACT-PROVENANCE-001.
Counters after strict147: unknown=16,229 unread=16,204 omitted=0 responsibilities=165.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
