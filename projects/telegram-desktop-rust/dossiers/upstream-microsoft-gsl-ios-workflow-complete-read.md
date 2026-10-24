# Microsoft/GSL .github/workflows/ios.yml complete read — strict177

## Authority

- Parent PR #154 exact HEAD: `c8edd25411c02a73b843ce38800d419632cd8234`, tree `0a262fda8d3236eab1288fe86e997d376b60c53f`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; Microsoft/GSL child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Exact source `Microsoft/GSL:.github/workflows/ios.yml@a7f2f104f0b75d715102cf0a41b1f9d4a206b485`: 55 line slots / 1,666 bytes, fully re-read.
- Direct action reference `actions/checkout@v4` currently resolves to commit `11d5960a326750d5838078e36cf38b85af677262`; `action.yml@24e73e5a12126edc2adb9e5cd1bf245ce85bde56` was re-read. The child workflow itself pins only the mutable major tag, so this resolution is provenance context, not a frozen product dependency.
- Root build contract `CMakeLists.txt@b1d24e5facfe56b1ae28468a5959797e9f6bacda`: 49 line slots / 1,779 bytes, fully re-read.
- Test build contract `tests/CMakeLists.txt@9de2519b366d6bc6803329a51afe459b0a6a357a`: 318 line slots / 11,090 bytes, fully re-read; `gsl_tests` links `Microsoft.GSL::GSL` and is registered with CTest.
- Documentation `README.md@dd61e3fdc4d30614c0ca35f58cce5f58e66dd4df`: 216 line slots / 14,388 bytes, fully re-read; iOS CI badge/platform-support context at lines 102-110.
- License `LICENSE@aa58667a3fd3784b0dfa11bf17f05f5e8dbd7d02`: MIT, 22 line slots / 1,157 bytes, fully re-read.
- Accepted tdesktop exact search: `ThirdParty/GSL` returns only `.gitmodules@2a71e861fab57dfd74d66e787867f2ede203a099`; `GSL_TEST` and `Microsoft.GSL` return zero direct tdesktop results. No tdesktop/Fabushi shipping iOS invocation of this child workflow is established.
- Parent exact-head evidence: TDRP `40091720209` 61/61 success; Rust desktop runtime `40091720144` 6/6 success; Official MCP marketplace `40091720164` success.

## Workflow contract

The child workflow is named `CI_iOS` and triggers on push and pull requests to `main`. It runs one `iOS` job on `macos-latest`.

The job:

1. checks out the child repository with `actions/checkout@v4`;
2. creates `build/`;
3. configures with CMake's Xcode generator, `CMAKE_SYSTEM_NAME=iOS`, architectures `arm64;x86_64`, deployment target 9, and `CMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY`;
4. sets bundle identifiers/version metadata used by the generated iOS test app targets;
5. builds Release for the `iphonesimulator` SDK with parallelism from `sysctl -n hw.ncpu`;
6. selects the last available iOS simulator runtime/device using `xcrun simctl` plus `jq`, and waits for boot completion;
7. finds every generated `tests/Release-iphonesimulator/*.app`, installs it into the booted simulator, extracts its bundle identifier from `Info.plist` using `plutil` + `jq`, launches it with console output, then uninstalls it.

The root CMake contract exposes the header-only `Microsoft.GSL::GSL` target and enables tests when `GSL_TEST` is true. The test CMake contract builds `gsl_tests` and `gsl_noexcept_tests`; under the iOS Xcode configuration these executable test targets become simulator app bundles that this workflow installs and launches.

## Direct action / toolchain / build / test provenance

- Action: `actions/checkout@v4` only. Its current v4 resolution was inspected, but the upstream workflow uses the mutable major tag.
- Hosted toolchain: `macos-latest`, Xcode generator and Apple iOS simulator SDK/runtime supplied by the GitHub hosted image.
- Configure toolchain: CMake, Xcode generator, `CMAKE_SYSTEM_NAME=iOS`, universal simulator/device architecture list, deployment target 9, static-library try-compile.
- Build: `cmake --build . --parallel ... --config Release -- -sdk iphonesimulator`.
- Test execution: generated test app discovery + `xcrun simctl install/launch/uninstall`; `plutil` and `jq` resolve bundle identifiers.
- No code-signing, physical-device, archive, TestFlight, App Store, production entitlement, platform adapter, updater or Fabushi iOS release path is exercised.

## Production boundary

This source is Microsoft/GSL dependency CI/test provenance for compilation and execution of that child library's tests in an iOS simulator. It is not a Fabushi shipping iOS/platform implementation owner.

It does not establish Fabushi's supported iOS deployment target, architecture policy, Xcode version, signing identity, entitlements, device compatibility, App Store readiness, native platform behavior, lifecycle ownership or release acceptance.

No `GslIOSRuntime`, `TelegramIOSRuntime`, second platform owner or copied dependency workflow is introduced.

## Existing-owner-first

Selected owner: existing TDRP recursive dependency/source and CI/test-provenance governance.

Fabushi shipping iOS/platform, packaging/signing and release owners remain independent and authoritative. No novel product owner is required by this dependency-only workflow.

## Negative oracles

- A failure in Microsoft/GSL's iOS simulator CI does not prove a Fabushi shipping iOS runtime failure.
- A success in Microsoft/GSL's iOS simulator CI does not prove Fabushi iOS/device/App Store acceptance.
- The child workflow's deployment target 9, `arm64;x86_64` architecture list, hosted `macos-latest` image and whichever simulator is selected are not Fabushi production platform requirements.
- `actions/checkout@v4` is a mutable major-tag dependency in the child workflow; its current resolution must not be treated as a permanently pinned product dependency.
- Simulator app install/launch is dependency-test execution only; it is not physical-device, signing, entitlement, archive, TestFlight or App Store evidence.
- Absence of a tdesktop shipping caller prevents promoting this workflow into Fabushi production implementation evidence.
- Any GSL gitlink/tree, ios.yml, checkout tag/action, root/test CMake, README platform table or license drift invalidates this exact-ref provenance and requires re-read.

## Fail-closed boundary

This strict entry closes only the exact source identity and dependency CI semantics of `.github/workflows/ios.yml` inside the pinned Microsoft/GSL child repository. It does not claim:

- that Fabushi executes this Microsoft/GSL iOS workflow;
- that Fabushi's shipping iOS/platform production responsibilities are implemented;
- that simulator success is release/device/App Store acceptance;
- source closure;
- baseline acceptance;
- production responsibility closure;
- independent release acceptance.

Counters after exactly this source entry are `unknown=16,199`, `unread=16,174`, `omitted=0`, `responsibilities=195`. `baseline_identity_accepted=false` and `source_closure_ready=false`.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-IOS-WORKFLOW-PROVENANCE-001`
- Target symbol: `TDRP_GSL_IOS_WORKFLOW_PROVENANCE_001`
- Oracles: `ORA-TDRP-GSL-IOS-WORKFLOW-EXACT-SOURCE-001`, `ORA-TDRP-GSL-IOS-WORKFLOW-BUILD-TEST-CHAIN-001`, `ORA-TDRP-GSL-IOS-WORKFLOW-NO-SHIPPING-OWNER-001`, `ORA-TDRP-GSL-IOS-WORKFLOW-MUTABLE-ACTION-REF-001`
- Invariants: `INV-TDRP-GSL-IOS-WORKFLOW-DEPENDENCY-ONLY-001`, `INV-TDRP-GSL-IOS-WORKFLOW-NO-PARALLEL-PLATFORM-001`, `INV-TDRP-GSL-IOS-WORKFLOW-NO-PRODUCT-CLOSURE-001`
- Contracts: `CONTRACT-TDRP-GSL-IOS-WORKFLOW-PINNED-SOURCE-001`, `CONTRACT-TDRP-GSL-IOS-WORKFLOW-CMAKE-TEST-001`, `CONTRACT-TDRP-GSL-IOS-WORKFLOW-EXACT-HEAD-EVIDENCE-001`
- Fault oracles: `FAULT-TDRP-GSL-IOS-WORKFLOW-SOURCE-DRIFT-001`, `FAULT-TDRP-GSL-IOS-WORKFLOW-TOOLCHAIN-DRIFT-001`, `FAULT-TDRP-GSL-IOS-WORKFLOW-PRODUCT-PROMOTION-001`
