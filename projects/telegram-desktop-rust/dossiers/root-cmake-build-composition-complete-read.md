# Root CMake build composition complete read — strict index 103

## Exact source and parent evidence
- Upstream: telegramdesktop/tdesktop@a7f453612f4896ac04d0007a36ab0d377e5bcbcf, tree c539eabfb05e7dcaa84821ac44c35cdfb6551ae5.
- Source: CMakeLists.txt@59477e251bbfea8d5efaf0bcda53798f92695449, fully read, 68 line slots / 2015 bytes.
- Parent Fabushi HEAD: 20e1eca26a610ee9fdcc7af38317a856a59340ff, tree 8a31ee1af96f430a093c81ad31529b9a6185f93d.
- Parent TDRP run 39907325317 attempt 1: 61/61 success.
- Authority artifact 12481591530, sha256:b498b25adc98e012698dbc3590318376dc4082e89f1670696e93125d51fda6b5.
- Closure artifact 12481567383, sha256:d519fab676cb70d1bba675b16bf999827072f164627c5dfad21d75f979f30b14.
- Reachability artifact 12481741705, sha256:5d6ab1354c5b55333b500f14bf111ba0f26a3d8ba8bb2cd98b8a791913396062.
- Parent Rust desktop runtime run 39907325306 attempt 1: 6/6 success; artifact 12481044536, sha256:d560e16bda0dc22a343409b767de42026be0c1a62dfb35f0a9c29bd7f9322c23.

## Complete-read responsibilities
The root CMake file is the top-level Telegram Desktop build-composition definition. It binds the supported CMake range, project/version/languages, Apple Objective-C/Objective-C++ variation, multi-config MinSizeRel, Visual Studio startup target, ThirdParty/Telegram/CMake-helper roots, bundled Python selection for non-packaged Windows, common variables/source/compile/link/framework/target-generation/NuGet/D3D/qrc/options/Qt helpers, skip-list entries glibmm/variant, and the cmake + Telegram subgraphs.

Every referenced helper, acquisition path and child subgraph remains an independent strict-source responsibility. This row does not mark them read or implemented transitively.

## Existing-owner-first disposition
Applicable source-neutral responsibilities already belong to:
- desktop/package.json canonical scripts.build / scripts.package;
- existing component Cargo/package manifests;
- FQT-EVID-001 Repository source-derived build-model evidence and Configured-build graph provenance evidence;
- .github/workflows/rust-desktop-runtime.yml exact-head executable validation;
- FQT-REL-001 package/release acceptance.

No Telegram/CMake/Qt build root, second dependency graph, second package owner, local executable-validation path, or shipping runtime is introduced.

## Existing-owner / validator uniqueness preflight
At parent exact HEAD:
- source_path=CMakeLists.txt rows: 0;
- source blob rows: 0;
- responsibility rows: 0;
- desktop/package.json selected-target rows: 0;
- canonical scripts.build occurrence: 1;
- canonical scripts.package occurrence: 1;
- Repository source-derived build-model evidence heading: 1;
- Configured-build graph provenance evidence heading: 1;
- Rust desktop runtime workflow name: 1.

The desktop/package.json build/package blob is identical on the PR branch and live main; live-main FQT-EVID-001 retains the same source-derived/configured-build evidence headings.

## Fail-closed boundary
Source CMake text cannot prove evaluated configured membership, compiler definitions, generated inputs, build execution, artifact identity, runtime reachability, package identity, signing, or release acceptance. Direct helpers/acquisitions/subgraphs stay open. Executable validation remains exact-head GitHub-Actions-only. Branch or upstream lease drift stops publication.

## Traceability
Requirement: TDRP-R9-ROOT-CMAKE-BUILD-COMPOSITION-PROVENANCE-001.
Selected existing owner: desktop/package.json canonical shipping build/package composition, with FQT-EVID-001 source-derived/configured-build evidence and rust-desktop-runtime exact-head validation.
Coverage disposition: toolchain-equivalent / mapped-open.
Closing strict103 yields unknown=16,273, unread=16,248, omitted=0, responsibilities=121.
Next strict root index 104: GROK.md@b1ae452d94cf48971c6d7ea17cac6b45cabee08d.
Fresh child exact-head TDRP/Rust evidence is required.
