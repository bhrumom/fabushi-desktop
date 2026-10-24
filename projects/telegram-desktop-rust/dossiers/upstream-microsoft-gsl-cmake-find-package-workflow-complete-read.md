# Microsoft/GSL .github/workflows/cmake_find_package.yml complete read — strict175

## Authority

- Parent PR #154 exact HEAD: `cfa73543c24f3d86275bee7f6c8b8b6583a8aa20`, tree `17e250f10593da8cc58c1a7ec0287675014b689f`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; Microsoft/GSL child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Exact source `Microsoft/GSL:.github/workflows/cmake_find_package.yml@fd2f7a812ca21e4e125058e50acadd6b5ccb28ec`: 25 line slots / 807 bytes, fully re-read.
- Root package-install context `CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda` and consumer context `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a` were re-read.
- Parent exact-head evidence: TDRP `40089287644` 61/61 success; Rust desktop runtime `40089287862` 6/6 success; Official MCP marketplace `40089287740` success.

## Workflow contract

The child workflow is named `cmake_find_package`, triggers on push and pull_request to `main`, and runs an Ubuntu/macOS matrix.

Each matrix job:

1. checks out the Microsoft/GSL child repository;
2. provisions CMake 3.14.0 using `lukka/get-cmake@latest`;
3. configures the project with Ninja, `GSL_TEST=OFF`, and a temporary install prefix under `build/install`;
4. builds the `install` target;
5. configures `tests/` as a separate source tree with `CMAKE_PREFIX_PATH` pointing at that install prefix.

The root CMake contract exposes `Microsoft.GSL::GSL`, gates installation with `GSL_INSTALL`, installs the `gsl` headers and exported target namespace, and emits `Microsoft.GSLConfigVersion.cmake`. When `tests/` is configured as its own source root, it requires `find_package(Microsoft.GSL CONFIG REQUIRED)`.

## Production boundary

This is Microsoft/GSL dependency package-discovery CI. It proves that the child project's installed CMake package metadata can be consumed from a fresh configure. It does not define Fabushi application packaging, updater behavior, code signing, package-manager integration, runtime loading, deployment topology or release acceptance.

The accepted tdesktop product owns this child through the pinned `Telegram/ThirdParty/GSL` gitlink; no shipping tdesktop/Fabushi package consumer is established by this workflow.

No `GslPackageRuntime`, `TelegramPackagingRuntime`, second packaging root or copied dependency workflow is introduced.

## Existing-owner-first

Selected owner: existing TDRP recursive dependency/source and CI/test-provenance governance.

Actual Fabushi packaging, updater, platform and release owners remain independent and authoritative. No novel owner is required.

## Fail-closed boundary

This strict entry closes only the exact source identity and CI semantics of `.github/workflows/cmake_find_package.yml` inside the pinned Microsoft/GSL child repository. It does not claim:

- that Fabushi executes this Microsoft/GSL package-discovery workflow;
- that CMake 3.14.0, Ninja, the install layout or package namespace are Fabushi release requirements;
- that any tdesktop product package consumer is implemented;
- source closure;
- baseline acceptance;
- production responsibility closure;
- independent release acceptance.

Counters after exactly this source entry are `unknown=16,201`, `unread=16,176`, `omitted=0`, `responsibilities=193`. `baseline_identity_accepted=false` and `source_closure_ready=false`.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-CMAKE-FIND-PACKAGE-PROVENANCE-001`
- Target symbol: `TDRP_GSL_CMAKE_FIND_PACKAGE_PROVENANCE_001`
- Oracles: `ORA-TDRP-GSL-CMAKE-FIND-PACKAGE-EXACT-SOURCE-001`, `ORA-TDRP-GSL-CMAKE-FIND-PACKAGE-INSTALL-001`, `ORA-TDRP-GSL-CMAKE-FIND-PACKAGE-CONSUMER-001`, `ORA-TDRP-GSL-CMAKE-FIND-PACKAGE-NO-SHIPPING-OWNER-001`
- Invariants: `INV-TDRP-GSL-CMAKE-FIND-PACKAGE-DEPENDENCY-ONLY-001`, `INV-TDRP-GSL-CMAKE-FIND-PACKAGE-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-CMAKE-FIND-PACKAGE-NO-PRODUCT-CLOSURE-001`
- Contracts: `CONTRACT-TDRP-GSL-CMAKE-FIND-PACKAGE-PINNED-SOURCE-001`, `CONTRACT-TDRP-GSL-CMAKE-FIND-PACKAGE-EXACT-HEAD-EVIDENCE-001`
- Fault oracles: `FAULT-TDRP-GSL-CMAKE-FIND-PACKAGE-SOURCE-DRIFT-001`, `FAULT-TDRP-GSL-CMAKE-FIND-PACKAGE-INSTALL-DRIFT-001`, `FAULT-TDRP-GSL-CMAKE-FIND-PACKAGE-PRODUCT-PROMOTION-001`
