# Microsoft/GSL README.md complete read — strict144

## Authority
- telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5
- Telegram/ThirdParty/GSL -> Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37, child tree 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41
- README.md@dd61e3fdc4d30614c0ca35f58cce5f58e66dd4df, fully read: 216 line slots / 14388 bytes
- Parent exact head 1038a9f4297dd44d5ec98941217280d1e706b245, tree 06d4576f053e404794b84c19ab037f30cebdca5c
- Parent evidence: TDRP 40023294574 61/61 success; authority 12532011015 sha256:7379340d3c6f0bf894311f6a33eac0fc27a3f0558df02093082b1adc48fb60c4; closure 12531912187 sha256:d76901bbfd441b272193f5e3260e615a534409ab8490964c032eac644fd6d9e3; reachability 12532216538 sha256:f4b1ef74186adf7671cf81c7565c7ff2aebaeb8df70f5b0d53d75df61a7bc870; Rust 40023294443 6/6 success; artifact 12531881804 sha256:f6ec57fb9ee686dc3803dfdc1f9e176fba04e264c4336e9920251e061837f55d
- Next: SECURITY.md@869fdfe2b246991a053fab9cfec1bed3ab532ab1

## Exact source and disposition
The locked README identifies Microsoft GSL as the Guidelines Support Library implementation, documents an inline/header-only include model, a supported/removed Core Guidelines feature matrix, compiler/toolset and platform-CI claims, test-build instructions, vcpkg/CMake/FetchContent integration, Google Test third-party notice linkage, and GSL.natvis debugger support. These are dependency/build-interface and source-provenance responsibilities. They do not prove that current Fabushi artifacts ship GSL, do not establish Fabushi platform/release support, and do not replace reading the referenced include/gsl implementation.

## Existing-owner-first
Reuse TDRP-SRC-03 dependency/documentation disposition, module-map build/generator/resources/dependencies mapping, and existing Fabushi CI/packaging/dependency review lifecycle. No GSL runtime, second dependency/package owner, parallel CMake/vcpkg graph, Telegram build owner or service is introduced.

## Fail-closed
README feature/support claims cannot replace exact implementation reads, configured-build evidence, package composition, signing, release acceptance or third-party notice closure. The feature matrix does not transitively close include/gsl; the Google Test pointer does not close ThirdPartyNotices.txt; the repository policy link does not close SECURITY.md. Drift, partial read, transitive closure, inferred shipping or cancelled publication is non-passing.

## Traceability
Requirement TDRP-R9-GSL-README-DEPENDENCY-BUILD-CONTRACT-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-README-EXACT-SOURCE-001, ORA-TDRP-GSL-README-DEPENDENCY-CONTRACT-001, ORA-TDRP-GSL-README-BUILD-OWNER-001, ORA-TDRP-GSL-README-SHIPPING-SEPARATION-001.
Invariants: INV-TDRP-GSL-README-NO-OMISSION-001, INV-TDRP-GSL-README-NO-RUNTIME-OWNER-001, INV-TDRP-GSL-README-NO-PARALLEL-BUILD-GRAPH-001, INV-TDRP-GSL-README-NO-SHIPPING-INFERENCE-001, INV-TDRP-GSL-README-EXACT-PROVENANCE-001.
Counters after strict144: unknown=16,232 unread=16,207 omitted=0 responsibilities=162.
baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain open.
