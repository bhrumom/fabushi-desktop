# Microsoft/GSL CMakeLists.txt complete read — strict139

## Authority
- Accepted tdesktop upstream: telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Root gitlink: Telegram/ThirdParty/GSL resolves exactly to Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37.
- GSL child tree: 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- Atomic child source: CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda, fully read: 49 line slots / 1779 bytes.
- Parent Fabushi exact head: 61a87fd19adcf62aa9ca9ca11587e430cd9d1e61, tree 4a6c913de2cd35aa223589acf7f88c8db3d46200.
- Parent exact-head evidence: TDRP 40011193081 61/61 success; authority 12525698776 sha256:eec8e94f1ab42a2f74c5c2ea19906b6cf639ed2fe61e8508ca6ea3b2b25a1449; closure 12526831881 sha256:8ab394d8e7c8cc30041901ed306b7c4aa64cdcb36043ae6ce73db30fc5716430; reachability 12526916734 sha256:8f26c3ee654a063fe90496cc1cfb602afe99d2b7c2a0850f22ca57ab08361b5c; Rust 40011193189 6/6 success; artifact 12526048848 sha256:1e3a92abd847175d1efccb76a3e1893b2e5e867e6e02a9b7e462d9857dc15b42.
- Submission lease: live main 7394f6dc7687f9b986ee5589d40aa680ce23b90a; live upstream a7f453612f4896ac04d0007a36ab0d377e5bcbcf/c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Next GSL root non-directory entry: CMakeSettings.json@2e1ad24c582b1db241a9cee2553935095e77e80c.

## Exact source
GSL 4.1.0 is declared as a CXX project with header-only INTERFACE target GSL and alias Microsoft.GSL::GSL. PROJECT_IS_TOP_LEVEL controls default GSL_INSTALL/GSL_TEST values; cxx_std_14 is required; include is always composed; GSL.natvis is a build-interface source; GSL_TEST conditionally enables testing and tests; GSL_INSTALL conditionally declares header installation, target export, Microsoft.GSLConfig, SameMajorVersion ARCH_INDEPENDENT version config, and natvis installation.

The include/ and tests/ child contents remain independent unread/unknown source responsibilities; this parent read does not close them transitively.

## Existing-owner-first / applicability
Fabushi reuses desktop/package.json canonical build/package composition, FQT-EVID-001 Repository source-derived build-model evidence, FQT-EVID-001 Configured-build graph provenance evidence, .github/workflows/rust-desktop-runtime.yml exact-head executable validation, and existing packaging/release acceptance. No Microsoft/GSL/Telegram CMake runtime, second build graph, second package owner, C++ runtime or release authority is introduced.

## Fail-closed
Source CMake text cannot prove actual evaluation, configured target membership, compiler definitions, generated inputs, successful compilation, linked artifacts, runtime reachability, installed package identity, signing or release acceptance. PROJECT_IS_TOP_LEVEL-sensitive defaults cannot be generalized. Child subgraphs remain open. Any gitlink/tree/blob drift, partial read, unsupported inference or cancelled publication is non-passing.

## Traceability
Requirement: TDRP-R9-GSL-CMAKE-BUILD-PACKAGE-COMPOSITION-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-CMAKE-SOURCE-001, ORA-TDRP-GSL-CMAKE-TARGET-001, ORA-TDRP-GSL-CMAKE-OPTIONS-001, ORA-TDRP-GSL-CMAKE-OWNER-001, ORA-TDRP-GSL-CMAKE-CONFIGURED-TRUTH-SEPARATION-001.
Invariants: INV-TDRP-GSL-CMAKE-NO-SECOND-BUILD-ROOT-001, INV-TDRP-GSL-CMAKE-SOURCE-TEXT-NO-BUILD-PASS-001, INV-TDRP-GSL-CMAKE-CHILD-SUBGRAPHS-STAY-OPEN-001, INV-TDRP-GSL-CMAKE-INSTALL-NOT-RELEASE-ACCEPTANCE-001, INV-TDRP-GSL-CMAKE-EXACT-PROVENANCE-001.

Closing only strict139 changes counters to unknown=16,237, unread=16,212, omitted=0, responsibilities=157. baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain mandatory.

Next strict140: Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37 CMakeSettings.json@2e1ad24c582b1db241a9cee2553935095e77e80c.
