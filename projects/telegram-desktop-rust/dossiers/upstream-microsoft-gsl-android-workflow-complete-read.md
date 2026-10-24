# Microsoft/GSL .github/workflows/android.yml complete read — strict173

## Authority

- Parent PR #154 exact HEAD: `bf0149a2407f16d2557a8372312b6bd13126b24c`, tree `9e0ec2da112387656f9733d63fbdccf3d2062415`.
- Live Fabushi main: `2d915242d67a9ad1c335459b34d8a8c8561605ab`, tree `1d44087e10f19aaa7c916d42bb67e5c279086d79`.
- Locked `telegramdesktop/tdesktop@dev`: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- `Telegram/ThirdParty/GSL` gitlink: `87f9d768866548b5b86e72be66c60c5abd4d9b37`; Microsoft/GSL child tree `6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41`.
- Exact source `Microsoft/GSL:.github/workflows/android.yml@a2c88fc987a7db248c6bd1719fa6b2c1340732f6`: 60 line slots / 2,052 bytes, fully re-read.
- Parent exact-head evidence: TDRP `40086843574` 61/61 success; Rust desktop runtime `40086843577` 6/6 success; Official MCP marketplace `40086843528` success.

## Workflow contract

The child workflow is named `CI_Android`, triggers on push and pull_request to `main`, and uses concurrency keyed by workflow plus pull request/ref with `cancel-in-progress: true`.

The single `Android` job runs on `macos-latest-large` and:

1. checks out the Microsoft/GSL child repository with `actions/checkout@v4`;
2. creates `build`;
3. configures Java 8 Zulu using `actions/setup-java@v4`;
4. installs the Android API 24 default x86_64 system image, creates `xamarin_android_emulator`, and boots it headlessly;
5. configures CMake with `$ANDROID_NDK_LATEST_HOME/build/cmake/android.toolchain.cmake`, `ANDROID_PLATFORM=16`, `ANDROID_ABI=x86_64`, and Debug;
6. builds with `cmake --build . --parallel`;
7. waits for emulator boot completion and reports ABI;
8. pushes the generated `tests` directory to `/data/local/tmp`, marks test executables executable, and runs every `*_tests` binary on the emulator.

This is Microsoft/GSL dependency CI/test execution provenance. It does not define Fabushi Android architecture, runtime ownership, application packaging, release signing, mobile product behavior, or any tdesktop source responsibility beyond proving how this third-party child project tested itself.

## Production boundary

Exact accepted `tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf` code searches found:

- `GSL_TEST`: zero results.
- `Microsoft.GSL`: zero results.
- `ThirdParty/GSL`: one result, the repository gitlink declaration in `.gitmodules`.

Therefore this child workflow has no proven tdesktop shipping workflow consumer. Its emulator/NDK/test configuration must not be promoted into Fabushi's Android product or CI as if it were a product requirement.

Actual Fabushi Android/mobile responsibilities, platform adapters, release tests, ABI requirements and signing remain owned and evidenced by their existing native product owners and acceptance gates. No `GslAndroidRuntime`, `TelegramAndroidRuntime`, second mobile build root, or copied dependency workflow is introduced.

## Existing-owner-first

Selected owner: existing TDRP recursive dependency/source provenance and test-evidence governance.

Existing production candidates are deliberately not selected because this source file is a third-party dependency workflow, not a shipping product owner. The closest real product/platform owners continue to own any independently derived Android behavior.

No novel owner is required.

## Fail-closed boundary

This strict entry closes only the exact source identity and semantics of `.github/workflows/android.yml` inside the pinned Microsoft/GSL child repository. It does not claim:

- that Fabushi runs Microsoft/GSL Android tests;
- that the Android API/ABI/JDK versions are Fabushi release requirements;
- that any tdesktop product caller is implemented;
- that Fabushi Android acceptance is complete;
- source closure;
- baseline acceptance;
- production responsibility closure;
- independent release acceptance.

Counters after exactly this source entry are `unknown=16,203`, `unread=16,178`, `omitted=0`, `responsibilities=191`. `baseline_identity_accepted=false` and `source_closure_ready=false`.

## Traceability anchors

- Requirement: `TDRP-R9-GSL-ANDROID-CI-WORKFLOW-PROVENANCE-001`
- Target symbol: `TDRP_GSL_ANDROID_CI_WORKFLOW_PROVENANCE_001`
- Oracles: `ORA-TDRP-GSL-ANDROID-CI-EXACT-SOURCE-001`, `ORA-TDRP-GSL-ANDROID-CI-EMULATOR-TEST-001`, `ORA-TDRP-GSL-ANDROID-CI-NO-SHIPPING-OWNER-001`
- Invariants: `INV-TDRP-GSL-ANDROID-CI-DEPENDENCY-ONLY-001`, `INV-TDRP-GSL-ANDROID-CI-NO-PARALLEL-RUNTIME-001`, `INV-TDRP-GSL-ANDROID-CI-NO-PRODUCT-CLOSURE-001`
- Contracts: `CONTRACT-TDRP-GSL-ANDROID-CI-PINNED-WORKFLOW-001`, `CONTRACT-TDRP-GSL-ANDROID-CI-EXACT-HEAD-EVIDENCE-001`
- Fault oracles: `FAULT-TDRP-GSL-ANDROID-CI-SOURCE-DRIFT-001`, `FAULT-TDRP-GSL-ANDROID-CI-PRODUCT-PROMOTION-001`
