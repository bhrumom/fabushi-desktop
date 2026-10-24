# Microsoft/GSL CMakeSettings.json complete read — strict140

## Authority
- Accepted tdesktop upstream: telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Root gitlink: Telegram/ThirdParty/GSL resolves exactly to Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37.
- GSL child tree: 6d9ffc01c1dd5b232fc2bdd634c8f95fa80f4f41.
- Atomic child source: CMakeSettings.json@2e1ad24c582b1db241a9cee2553935095e77e80c, fully read: 18 line slots / 545 bytes.
- Parent Fabushi exact head: b407927178603f43898336356cd0efd5aa0fbca6, tree de960785bb242165ba81202df277ec51687c3c02.
- Parent exact-head evidence: TDRP 40013908183 61/61 success; authority 12528325394 sha256:05d6a65f6c05981b20c2185177d170b5f593a5f20da3771c5929d9e9207d8f2d; closure 12527742108 sha256:76ee38dc72b02df7dd034c93e76d637990615c42b7ed790e6f3aca3b481d3b01; reachability 12527822233 sha256:03cbd2ac9df1a883807f447c1241cb230aba54d4858d1365f5e6372525bdde1a; Rust 40013908184 6/6 success; artifact 12527158542 sha256:0c8f7b2952d68e77bc3e887aefd591bc31389978d8ec7a11da3a7cff2c69d3d5.
- Submission lease: live main 7394f6dc7687f9b986ee5589d40aa680ce23b90a/tree fbeb0bb32d944d0c1053880c471fe5c52279f052; live upstream a7f453612f4896ac04d0007a36ab0d377e5bcbcf/tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Next GSL root non-directory entry: CONTRIBUTING.md@e28f534e31b73b6ce1ac658944887015907c37d1.

## Exact source
The locked file declares exactly one Visual Studio CMake development configuration: `x64-Debug`, generator `Ninja`, `configurationType=Debug`, inheriting `msvc_x64_x64`. It keeps build and install outputs outside the source tree under `${env.USERPROFILE}\\CMakeBuilds\\${workspaceHash}\\build|install\\${name}`, passes `-DGSL_CXX_STANDARD=17` to CMake, passes `-v` to the build command, supplies an empty extra CTest argument string, and names `CppCoreCheckRules.ruleset` as the code-analysis ruleset.

A recursive read of the exact GSL tree found no repository entry named `CppCoreCheckRules.ruleset`. Therefore this source proves only the ruleset reference string; it does not prove whether Visual Studio resolves that name from an external installation, what exact rule contents/version would be selected, or whether analysis actually ran.

## Existing-owner-first / applicability
Fabushi reuses FQT-EVID-001 Configured-build graph provenance evidence, the existing repository language/toolchain quality owner described by TEST_STRATEGY's Repository source static-analysis migration qualification, and exact-head GitHub Actions executable validation. No Microsoft/GSL, Telegram, Visual Studio or CMake-specific second build graph, package owner, analyzer owner, C++ runtime or release authority is introduced.

## Fail-closed
This JSON does not prove that an IDE loaded the profile; that Ninja/MSVC were present; that `msvc_x64_x64` resolved; that path templates expanded; that `GSL_CXX_STANDARD=17` was consumed by a configured graph or compiler; that tests executed; that `CppCoreCheckRules.ruleset` resolved or was applied; or that any build, artifact, runtime, package, signing or release acceptance succeeded. The x64-Debug source profile cannot be generalized to Release, other architectures or non-Windows platforms. Any source/gitlink/tree drift, partial read, unsupported inference or cancelled publication remains non-passing.

## Traceability
Requirement: TDRP-R9-GSL-CMAKESETTINGS-WINDOWS-DEVELOPMENT-PROFILE-PROVENANCE-001.
Oracles: ORA-TDRP-GSL-CMAKESETTINGS-SOURCE-001, ORA-TDRP-GSL-CMAKESETTINGS-PROFILE-001, ORA-TDRP-GSL-CMAKESETTINGS-OWNER-001, ORA-TDRP-GSL-CMAKESETTINGS-CONFIGURED-TRUTH-SEPARATION-001, ORA-TDRP-GSL-CMAKESETTINGS-RULESET-RESOLUTION-001.
Invariants: INV-TDRP-GSL-CMAKESETTINGS-NO-SECOND-BUILD-ROOT-001, INV-TDRP-GSL-CMAKESETTINGS-IDE-PROFILE-NOT-BUILD-PASS-001, INV-TDRP-GSL-CMAKESETTINGS-RULESET-UNRESOLVED-STAYS-OPEN-001, INV-TDRP-GSL-CMAKESETTINGS-X64-DEBUG-NOT-RELEASE-ACCEPTANCE-001, INV-TDRP-GSL-CMAKESETTINGS-EXACT-PROVENANCE-001.

Closing only strict140 changes counters to unknown=16,236, unread=16,211, omitted=0, responsibilities=158. baseline_identity_accepted=false, source_closure_ready=false and independent release acceptance remain mandatory.

Next strict141: Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37 CONTRIBUTING.md@e28f534e31b73b6ce1ac658944887015907c37d1.
