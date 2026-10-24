# Upstream root cmake build-helper/toolchain provenance — strict index 110

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Root source entry: `cmake@7ceca3019b4750d7a5acb2de0faffd523ee14095`, gitlink to `desktop-app/cmake_helpers`, commit tree `f5c386cb34208309aae521187611fac4a2045577`.
- Parent exact HEAD: `79ba8d1e17a8f3736d37a38561e1acec8c82281d`, tree `ee9ad82afa6084eb8f4cd80f698ec0203b81899b`.
- Parent TDRP run `39923903565`: 61/61 success; authority `12488317476` sha256:a41831799280c49980822d6400b0e6578b8361c9a65eaa7faf9d1dc838f2e6f4; closure `12488578130` sha256:0d9c4a59f7e3982bccfd219fc3c79057ece05e3d205fc37928b554eb30198de1; reachability `12489375058` sha256:0823e275fd5a07b29a58370145aa800d45305a5b2b24adb8a488ef4e572229c4.
- Parent Rust desktop runtime run `39923903724`: 6/6 success; artifact `12488971000` sha256:3ce1613fd1d58bc2afce5ba5c1924a1c2396f1201340c5dd26cba0c7711afc8b.

## Recursive source accounting and complete read

The exact `desktop-app/cmake_helpers` tree is non-truncated and contains 170 entries / 102 non-directory entries: 101 blobs totaling 9,243,742 bytes plus one nested gitlink. All 99 text blobs were fetched and read in full. The two remaining blobs are the pinned x64/x86 `d3dcompiler_47.dll` payloads (4,916,840 and 4,127,224 bytes; git blobs `80489b89f29338fd3cb52cb3c58d97f2e39c107b` and `803e8f0a883d38b1176b9c06d46c97e4ad84ffe2`); their exact identities, use sites, version check and SHA-256 validation path were read from the adjacent validator/helper source rather than treating binary bytes as source text.

Its sole nested gitlink is `external/glib/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`, with authority URL `https://gitlab.com/mnauw/cppgir.git`. The same exact commit is mirrored at `repomansez/cppgir`; object IDs match the TDRP GitLab authority inventory, so that mirror was used only as a byte-identical read surface and not as replacement authority. That exact tree is non-truncated: 126 entries / 104 non-directory entries, comprising 103 fully read blobs totaling 759,925 bytes plus `expected-lite@95b9cb015fa17baa749c2b396b335906e1596a9e`.

The exact expected-lite child, now hosted as `nonstd-lite/expected-lite`, is also non-truncated: 55 entries / 43 non-directory entries, all 43 blobs fully read, totaling 265,949 bytes, with no deeper gitlink.

## Responsibility decomposition

The source is a build/toolchain integration layer, not a second product architecture. Responsibilities found across the complete read include:

- target initialization, source grouping, platform filtering, compiler/link option probing, framework linking, resource generation, special-target validation, LTO/ASAN and build-runner selection;
- packaged-versus-bundled dependency selection and imported/static target wiring for media, crypto, compression, text, QR, WebRTC, update and other third-party libraries;
- platform-specific Qt/static-plugin composition, including fcitx5/hime/nimf/kimageformats wiring, while their underlying product/input behavior remains owned by the separately inventoried source authorities;
- GLib/GObject/GIO GIR, D-Bus and C++ wrapper generation, with cppgir supplying the generator, generic GI lifetime/transfer/container/error/object wrappers, GIR overrides, tests and examples;
- expected-lite supplying cppgir's header-only value-or-error abstraction and its build/test/package provenance;
- bundled CLD3/Hunspell/MicroTeX and generated binding inputs, while the actual third-party implementation responsibilities remain separately accountable at their own source entries;
- the single Rust-staticlib constraint used by upstream to avoid duplicate allocator/runtime/panic machinery, and generated UniFFI wallet-engine binding wiring;
- Linux allocation tracing as a diagnostic tool, not shipping product truth;
- Windows DirectX helper resolution with exact DLL-version/hash fencing for special builds;
- upstream version parsing and release-build variation as build provenance only, not Fabushi canonical version authority.

## Existing-owner / validator uniqueness

Fresh parent-head preflight found zero prior `cmake` source-path rows, zero `7ceca3019b4750d7a5acb2de0faffd523ee14095` object rows, and zero `TDRP-R9-UPSTREAM-CMAKE-HELPERS-BUILD-INTEGRATION-PROVENANCE-001` rows. The already-mapped `TDRP-R9-ROOT-CMAKE-BUILD-COMPOSITION-PROVENANCE-001` row occurs exactly once and remains the nearest source-neutral build-composition owner. The configured-build provenance, qualification and usage rows each occur exactly once and remain the stronger validator/oracle for evaluated target/source/PCH/link/definition/generated-input claims.

Fabushi's current shipping composition remains `desktop/package.json` (`build` = Host + Electron + Renderer + desktop-runtime check; `package` adds electron-builder), component Cargo/package manifests and exact-head GitHub Actions. No Telegram/CMake/Qt build root, second dependency graph, second package owner, second configured-build validator, cppgir runtime, GI product owner, or expected result owner is introduced.

## Fail-closed boundaries

- CMake helper text is source provenance. It cannot prove a configured build, executed test, runtime reachability, package identity, signing, deployment or release acceptance.
- Packaged/bundled fallback rules do not by themselves prove which dependency bytes Fabushi ships.
- Child library source named by helper files is not transitively closed by this gitlink disposition; the child's own inventory/read/implementation evidence remains required.
- `version.cmake` does not become Fabushi version/release authority.
- cppgir/expected-lite are build-time/generic C++ support provenance and do not create Fabushi product owners.
- Upstream Qt/static-plugin selection is not a license to retain Qt or create a second platform-input runtime; applicable behaviors must remain mapped to Fabushi's existing source-neutral platform/input owners.
- The pinned DirectX DLLs are exact upstream binary build inputs only. Their presence cannot substitute for current Fabushi Windows dependency, security, package or acceptance evidence.
- Any source/ref/tree/owner drift requires fresh reconciliation before publication.

## Traceability

- Requirement: `TDRP-R9-UPSTREAM-CMAKE-HELPERS-BUILD-INTEGRATION-PROVENANCE-001`.
- Oracles: `ORA-TDRP-CMAKE-HELPERS-EXACT-RECURSIVE-SOURCE-001`, `ORA-TDRP-CMAKE-HELPERS-EXISTING-BUILD-OWNER-001`, `ORA-TDRP-CMAKE-HELPERS-CONFIGURED-TRUTH-SEPARATION-001`.
- Invariants: `INV-TDRP-CMAKE-HELPERS-NO-SECOND-BUILD-ROOT-001`, `INV-TDRP-CMAKE-HELPERS-CHILD-SOURCE-STAYS-INDEPENDENT-001`, `INV-TDRP-CMAKE-HELPERS-NO-RELEASE-ACCEPT-001`.

Closing strict110 yields unknown=16,266, unread=16,241, omitted=0, responsibilities=128. Next strict root index 111 is `docs/api_credentials.md@0d2819bd3cd6b6407ced257edbbd6623b438f262` (695 bytes). Fresh child exact-head TDRP/Rust evidence is required.
