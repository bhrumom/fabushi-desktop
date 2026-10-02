# Telegram Frozen Source Closure Evidence

Status: P0 evidence; recursive gitlink closure complete; resource/license/behavior closure not complete  
Frozen root: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
Captured: 2026-10-02

## Root tree

The GitHub recursive tree for the frozen root returned `truncated=false`. The root contains 35 direct gitlinks pinned in `upstream.lock.json`. The recursive SourceFiles inventory exposed 118 directories at depth <= 4, including product areas that were not explicit in the initial hand-authored capability list.

## Direct gitlink recursion

Every one of the 35 direct GitHub-hosted gitlink commits was queried with the GitHub recursive-tree API. None of those tree responses was truncated.

Thirty-two direct gitlinks contain no nested gitlinks. Three contain one nested gitlink each:

| Parent | Nested path | Pinned commit | Result |
| --- | --- | --- | --- |
| `desktop-app/cmake_helpers@7a6abdae…` | `external/glib/cppgir` | `47cf94f83b54cda59018135601e19d7fb0c77776` | recursive GitLab tree fully paged: 126 entries (103 blobs, 22 trees, 1 gitlink); sole nested gitlink `expected-lite@95b9cb015fa17baa749c2b396b335906e1596a9e`; that exact GitHub tree is `truncated=false`, 55 entries, 0 further gitlinks |
| `PJK/libcbor@170bee2b…` | `doxygen-theme` | `46111c61a9f49b7a9886127e679d4317478fab1c` | recursive tree verified, `truncated=false`, no further gitlinks |
| `ericniebler/range-v3@a8147793…` | `doc/gh-pages` | `2dae74bb693e42d850fb0adcc9045c5b71fbdeae` | recursive tree verified, `truncated=false`, no further gitlinks |

The `cppgir` parent `.gitmodules` pins its URL to `https://gitlab.com/mnauw/cppgir.git`. On 2026-10-02, htch-runtime read the GitLab repository-tree API at exact ref `47cf94f83b54cda59018135601e19d7fb0c77776` through all pages: 126 entries total, with exactly one mode-`160000` entry, `expected-lite@95b9cb015fa17baa749c2b396b335906e1596a9e`. The exact `cppgir/.gitmodules` maps it to `https://github.com/martinmoene/expected-lite.git`; GitHub's canonical repository is now `nonstd-lite/expected-lite`, whose exact recursive tree at `95b9cb0…` returned `truncated=false`, 55 entries and no gitlinks. The external `cppgir` recursive gitlink chain is therefore closed; this does **not** close build/resource/license provenance.

## Newly exposed capability domains

The recursive frozen root and `Telegram/CMakeLists.txt` exposed additional reachable domains that are now explicit FBCP research inputs:

- communities / managed-community membership and administration;
- Compose AI and AI tone creation/preview;
- rich tasks and todo lists;
- ringtones / notification sound resources;
- self-destruct / expiry;
- sensitive-content policy;
- statistics / channel analytics;
- usernames, websites and deep-link handling;
- media/photo/video editor;
- Instant View / rich document rendering;
- support/moderation/report flows;
- TDE2E security/protocol behavior.

These are capability inputs only. They do not authorize Telegram-prefixed product owners.

## Build/resource closure checkpoint

The frozen root already proves these non-product-code inputs are reachable and therefore cannot be omitted from provenance closure:

| Area | Frozen evidence / responsibility | Current closure |
| --- | --- | --- |
| generators/codegen | `Telegram/codegen`, MTProto/scheme and language-generation references in CMake | source roots identified; generated-output-to-input map still incomplete |
| resources/assets | `Telegram/Resources` themes, language bundles, emoji, export templates, webview/picker HTML, sounds and updater/platform assets | resource roots identified; copied/derived asset and per-license review still incomplete |
| shaders | `Telegram/shaders` | root identified; shader build inputs/outputs and license provenance still incomplete |
| build/packaging | `Telegram/build`, `Telegram/cmake`, root `cmake`, `snap`, platform-specific packaging/update inputs | roots identified; build-time downloads and complete platform matrix still incomplete |
| native third parties | direct gitlinks including `tgcalls`, `lib_webrtc`, `lib_webview`, FIDO2 and media/storage libraries | GitHub direct trees recursively checked; external cppgir leaf remains partial |

No item in this table is an instruction to ship Telegram dependencies. It is source/provenance research required before declaring capability/source coverage complete.

## Build/resource closure still open

`Telegram/CMakeLists.txt` proves that the product links or generates through codegen, lib_storage, lib_ui, lib_webrtc, lib_webview, tgcalls, FIDO2, ffmpeg, Stripe, MTProto/scheme generators, language generators, update-key generation, platform MIDL on Windows, and Apple Swift runtime. An exact frozen-tarball inventory on htch-runtime recorded 3,073 files under `Telegram/Resources`, 35 shader files, 36 files under `Telegram/build`, 30 under `Telegram/cmake`, and the Snap packaging input. The generator chain explicitly references scheme, language, numbers, MIDL, DBus, update-key, AppStream changelog and QRhi-shader generation. A targeted scan of Telegram/root CMake inputs found no `file(DOWNLOAD)`, `FetchContent`, `ExternalProject_Add`, URL-download, curl or wget primitives; this narrows the build-time-download search but does not prove every external package/license is closed.

P0 must still enumerate:

1. build-time downloads and non-gitlink external packages;
2. generated outputs and their source inputs;
3. patches/shaders/platform packaging inputs;
4. dependency license expressions and copied/derived asset provenance;
5. third-party dependency/license expressions and copied/derived asset provenance across the now-closed gitlink tree;
6. behavior-level responsibility dossiers for each reachable product capability.

Until those close, source research coverage must not be reported as 100%.
