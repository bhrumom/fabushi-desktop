# Microsoft/GSL .github/workflows/compilers.yml complete read — strict176

## Authority

- Parent PR #154 exact HEAD: `869b77142fe13e67b9aa8acd142f55a16a3e00be`, tree `37925468ad83a8ae37dde178b9c8f47d97a0f463`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; Microsoft/GSL child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Exact source `Microsoft/GSL:.github/workflows/compilers.yml@23866a631659aa7d5290e4824b93908de21d42f4`: 104 line slots / 3,164 bytes, fully re-read.
- Direct child action `.github/workflows/cmake/action.yml@0ac09bfa4b7702e768f42ad841563169aef267e0`: 56 line slots / 1,353 bytes, fully re-read.
- Documentation alignment source `README.md@dd61e3fdc4d30614c0ca35f58cce5f58e66dd4df`: 216 line slots / 14,388 bytes; supported-compiler contract at lines 88-99 re-read.
- Accepted tdesktop exact search for `ThirdParty/GSL` returns only `.gitmodules@2a71e861fab57dfd74d66e787867f2ede203a099`; no product/shipping consumer of this child workflow is established.
- Parent exact-head evidence: TDRP `40090073438` 61/61 success; Rust desktop runtime `40090073443` 6/6 success; Official MCP marketplace `40090073449` success.

## Workflow contract

The child workflow is named `Compiler Integration Tests`, triggers on push and pull requests to `main`, and groups concurrency by workflow plus pull-request number/ref with `cancel-in-progress: true`.

It exercises four compiler families:

1. **GCC** on Ubuntu: versions 10, 11 and 12, Debug/Release, C++14/17/20/23, excluding GCC 10 with C++23.
2. **Clang** on Ubuntu: versions 13, 14 and 15, Debug/Release, C++14/17/20/23.
3. **Xcode / Apple Clang** on macOS: Xcode 14.3.1 and 15.4, Debug/Release, C++14/17/20/23; the selected Xcode is activated with `xcode-select`.
4. **Visual Studio** on Windows: VS2019/windows-2019 and VS2022/windows-2022, Debug/Release, C++14/17/20/23, with either the default toolset or `-T ClangCL`; unsupported image/generator and VS2019+C++23 combinations are excluded.

Every compiler job checks out the child and invokes the pinned local composite CMake action. The Visual Studio path additionally provisions MSBuild and overrides build/test commands with `msbuild GSL.sln` and `ctest . --output-on-failure --no-compress-output`.

The README's supported-compiler table mirrors the workflow's GCC 10/11/12, Clang 13/14/15, Xcode 14.3.1/15.4 and Visual Studio 2019/2022 coverage, and states C++14/17/20/23 support where the compiler supports that language level.

## Production boundary

This source is Microsoft/GSL dependency compiler/test CI. It demonstrates the child dependency's supported toolchain matrix; it does not define Fabushi's production compiler support policy, ABI contract, build graph, platform adapter, packaging, signing, updater, deployment or release acceptance.

The accepted tdesktop product owns the child only through the pinned `Telegram/ThirdParty/GSL` submodule. Exact tdesktop search establishes no shipping invocation of this workflow.

No `GslCompilerRuntime`, `TelegramBuildRuntime`, second build graph, second platform owner or copied dependency workflow is introduced.

## Existing-owner-first

Selected owner: existing TDRP recursive dependency/source and CI/test-provenance governance.

Fabushi's shipping build, platform, packaging and release owners remain independent and authoritative. No novel product owner is required.

## Negative oracles

- A compiler matrix job failing does not imply a Fabushi product runtime failure; this source is dependency CI provenance.
- The workflow's compiler versions and hosted runner images are not Fabushi shipping support requirements.
- README/workflow matrix alignment is a child-repository maintenance contract, not a Fabushi release acceptance gate.
- Absence of a tdesktop shipping caller prevents promoting this CI workflow into production implementation evidence.
- Any GSL gitlink/tree, compiler workflow, composite action, README supported-toolchain table or license drift invalidates this exact-ref evidence and requires re-read.

## Fail-closed boundary

This strict entry closes only the exact source identity and CI semantics of `.github/workflows/compilers.yml` inside the pinned Microsoft/GSL child repository. It does not claim:

- that Fabushi executes this Microsoft/GSL compiler workflow;
- that Fabushi supports exactly these compiler or C++ standard versions;
- that any tdesktop production responsibility is implemented by this workflow;
- source closure;
- baseline acceptance;
- production responsibility closure;
- independent release acceptance.

Counters after exactly this source entry are `unknown=16,200`, `unread=16,175`, `omitted=0`, `responsibilities=194`. `baseline_identity_accepted=false` and `source_closure_ready=false`.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-COMPILER-INTEGRATION-WORKFLOW-PROVENANCE-001`
- Target symbol: `TDRP_GSL_COMPILER_INTEGRATION_WORKFLOW_PROVENANCE_001`
- Oracles: `ORA-TDRP-GSL-COMPILER-WORKFLOW-EXACT-SOURCE-001`, `ORA-TDRP-GSL-COMPILER-MATRIX-001`, `ORA-TDRP-GSL-COMPILER-COMPOSITE-CALLER-001`, `ORA-TDRP-GSL-COMPILER-NO-SHIPPING-OWNER-001`
- Invariants: `INV-TDRP-GSL-COMPILER-WORKFLOW-DEPENDENCY-ONLY-001`, `INV-TDRP-GSL-COMPILER-WORKFLOW-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-COMPILER-WORKFLOW-NO-PRODUCT-CLOSURE-001`
- Contracts: `CONTRACT-TDRP-GSL-COMPILER-WORKFLOW-PINNED-SOURCE-001`, `CONTRACT-TDRP-GSL-COMPILER-WORKFLOW-README-ALIGNMENT-001`, `CONTRACT-TDRP-GSL-COMPILER-WORKFLOW-EXACT-HEAD-EVIDENCE-001`
- Fault oracles: `FAULT-TDRP-GSL-COMPILER-WORKFLOW-SOURCE-DRIFT-001`, `FAULT-TDRP-GSL-COMPILER-WORKFLOW-MATRIX-DRIFT-001`, `FAULT-TDRP-GSL-COMPILER-WORKFLOW-PRODUCT-PROMOTION-001`
