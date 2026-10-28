# Microsoft/GSL .github/workflows/cmake/action.yml complete read — strict174

## Authority

- Parent PR #154 exact HEAD: `65351f0d75afd5b27a1a36d4fa6a026388b8192d`, tree `97a92b56dcd527f64fc1928bfce4d8b0564264ea`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; Microsoft/GSL child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Exact source `Microsoft/GSL:.github/workflows/cmake/action.yml@0ac09bfa4b7702e768f42ad841563169aef267e0`: 56 line slots / 1,353 bytes, fully re-read.
- Exact direct child caller `Microsoft/GSL:.github/workflows/compilers.yml@23866a631659aa7d5290e4824b93908de21d42f4`: 104 line slots / 3,164 bytes, fully re-read; four `uses: ./.github/workflows/cmake` calls.
- Parent exact-head evidence: TDRP `40087585670` 61/61 success; Rust desktop runtime `40087585716` 6/6 success; Official MCP marketplace `40087585579` success.

## Composite-action contract

The pinned child file is a GitHub composite action named `Composite CMake`. Its inputs cover CMake generator, build type, C++ compiler, GSL C++ standard, extra CMake arguments, build command, test command and shell.

Its ordered contract is:

1. create `build`;
2. configure CMake in `build` with the supplied generator/build type/compiler/standard plus `CI_TESTING=ON`, `CMAKE_VERBOSE_MAKEFILE=ON` and `-Werror=dev`;
3. execute the supplied build command;
4. execute the supplied test command.

The pinned `compilers.yml` child workflow is the only workflow in the exact Microsoft/GSL child tree that directly invokes this local action, once each from its gcc, clang, Xcode and Visual Studio jobs.

## Production boundary

This file is Microsoft/GSL dependency CI/test orchestration. Its direct caller is another child-repository CI workflow, not a tdesktop shipping source entrypoint. The accepted tdesktop product owns the child only through the pinned `Telegram/ThirdParty/GSL` gitlink.

Therefore the generator/compiler/test matrix and composite-action shell contract must not be promoted into Fabushi shipping build architecture, platform ownership or release acceptance. Existing Fabushi build, packaging, platform and release owners remain authoritative for product behavior.

No `GslCMakeRuntime`, `TelegramBuildRuntime`, second build root, copied child action or source-named production owner is introduced.

## Existing-owner-first

Selected owner: existing TDRP recursive dependency/source and CI/test-provenance governance.

The child compiler workflow remains an independent source entry even though it is read here as the direct caller oracle. Actual Fabushi build and platform owners are not reassigned to this dependency-only action.

No novel owner is required.

## Fail-closed boundary

This strict entry closes only the exact source identity and CI semantics of `.github/workflows/cmake/action.yml` inside the pinned Microsoft/GSL child repository. It does not claim:

- that Fabushi executes this Microsoft/GSL composite action;
- that its compiler/generator/C++ standard matrix is a Fabushi release requirement;
- that the child `compilers.yml` source entry is itself closed by this caller read;
- that any tdesktop product caller is implemented;
- source closure;
- baseline acceptance;
- production responsibility closure;
- independent release acceptance.

Counters after exactly this source entry are `unknown=16,202`, `unread=16,177`, `omitted=0`, `responsibilities=192`. `baseline_identity_accepted=false` and `source_closure_ready=false`.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-CMAKE-ACTION-PROVENANCE-001`
- Target symbol: `TDRP_GSL_CMAKE_ACTION_PROVENANCE_001`
- Oracles: `ORA-TDRP-GSL-CMAKE-ACTION-EXACT-SOURCE-001`, `ORA-TDRP-GSL-CMAKE-ACTION-DIRECT-CALLER-001`, `ORA-TDRP-GSL-CMAKE-ACTION-NO-SHIPPING-OWNER-001`
- Invariants: `INV-TDRP-GSL-CMAKE-ACTION-DEPENDENCY-ONLY-001`, `INV-TDRP-GSL-CMAKE-ACTION-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-CMAKE-ACTION-NO-PRODUCT-CLOSURE-001`
- Contracts: `CONTRACT-TDRP-GSL-CMAKE-ACTION-PINNED-SOURCE-001`, `CONTRACT-TDRP-GSL-CMAKE-ACTION-EXACT-HEAD-EVIDENCE-001`
- Fault oracles: `FAULT-TDRP-GSL-CMAKE-ACTION-SOURCE-DRIFT-001`, `FAULT-TDRP-GSL-CMAKE-ACTION-CALLER-DRIFT-001`, `FAULT-TDRP-GSL-CMAKE-ACTION-PRODUCT-PROMOTION-001`
