# Telegram Frozen Source Closure Evidence

Status: P0 evidence; recursive gitlink, tracked generator/packaging roots, and external-acquisition entrypoints discovered; immutable external-resolution/resource-license/behavior closure not complete
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

## Source-directory citation coverage audit

A reverse audit against the frozen GitHub tree at `33261535a0e747f125e0ed25486f01e556330677` enumerated all 40 immediate `Telegram/SourceFiles` directories and 130 directory paths through relative depth two. Every one is cited by path or directory identity in the current research dossier set. This is useful negative evidence against an entirely unvisited top-level/second-level source area.

The result is **not** equivalent to behavior completeness: a directory citation can cover multiple state machines and edge cases, deeper files can expose additional responsibilities, and infrastructure/toolkit directories can still affect product lifecycle. It therefore narrows the remaining research question from “is there an untouched major SourceFiles area?” to “are all reachable behaviors inside the cited areas represented by a capability/aggregate row with source-to-behavior evidence?”.

## Build/resource closure checkpoint

The frozen root already proves these non-product-code inputs are reachable and therefore cannot be omitted from provenance closure:

| Area | Frozen evidence / responsibility | Current closure |
| --- | --- | --- |
| generators/codegen | `Telegram/SourceFiles/codegen`, scheme/language/number/MIDL/update-key/DBus/AppStream/model generation references in CMake | source-level input→output map recorded in `research/build-toolchain-resource-provenance.md`; generated outputs are not Fabushi shipping inputs by default |
| resources/assets | `Telegram/Resources` themes, language bundles, emoji, export templates, webview/picker HTML, sounds and updater/platform assets | resource roots identified; copied/derived asset and per-license review still incomplete |
| shaders | `Telegram/shaders` | 35 tracked sources identified; QRhi `qsb` output/QRC generation mapped; copied/derived shader license provenance remains open |
| build/packaging | `Telegram/build`, `Telegram/cmake`, `snap`, Windows/macOS/Linux packaging/update inputs | tracked platform roots and all explicit URL acquisition entrypoints mapped; mutable refs, package-manager resolution and dependency-by-dependency licenses remain open |
| native third parties | direct gitlinks including `tgcalls`, `lib_webrtc`, `lib_webview`, FIDO2 and media/storage libraries | all direct/nested gitlink recursion closed, including `cppgir`→`expected-lite`; non-gitlink external acquisitions remain separate |

No item in this table is an instruction to ship Telegram dependencies. It is source/provenance research required before declaring capability/source coverage complete.

## Build/resource closure still open

`Telegram/CMakeLists.txt` proves that the product links or generates through codegen, lib_storage, lib_ui, lib_webrtc, lib_webview, tgcalls, FIDO2, ffmpeg, Stripe, MTProto/scheme generators, language generators, update-key generation, platform MIDL on Windows, and Apple Swift runtime. An exact frozen-tree inventory on htch-runtime records 3,073 files under `Telegram/Resources`, 35 shader sources, eight `.obj` model inputs, 36 files under `Telegram/build`, 30 under `Telegram/cmake`, and the Snap packaging input. `research/build-toolchain-resource-provenance.md` now maps the tracked generator inputs/outputs, shader baking, model baking and Windows/macOS/Linux packaging roots.

A previous CMake-only scan correctly found no CMake `file(DOWNLOAD)`, `FetchContent`, or `ExternalProject_Add`, but that scope was too narrow. The frozen `Telegram/build/prepare/prepare.py` and `Telegram/build/docker/centos_env/Dockerfile` explicitly perform network acquisition using `git clone/fetch`, PowerShell `iwr`, `wget`, and `curl`. Build-time external acquisition therefore remains a real provenance blocker and must be enumerated rather than reported absent.

The current PR #26 adoption audit found no Telegram prepare/Docker/Snap invocation, no tracked Telegram resource/shader tree, no `.qsb`/`.binobj`/`.obj` payload, and no binary addition in the FBCP production delta. Those Telegram build/resource inputs are therefore research-only for the current distributed branch rather than shipping dependencies. This does not turn mutable historical acquisition into immutable evidence and does not waive source-informed legal review.

P0 must still close:

1. historical immutable build-input evidence only where needed to understand a source-derived behavior/build responsibility, while recording explicit non-adoption for Telegram-only dependencies not used by Fabushi;
2. dependency/license review for dependencies actually adopted by distributed Fabushi, plus source-informed GPL/third-party review;
3. copied/derived provenance for any upstream resource/shader/model/source content that a future or current production change actually copies or adapts; the current FBCP branch has no such Telegram resource/shader/model payload;
4. mixed-license/per-file provenance needed to interpret researched upstream sources where the selected behavior depends on those files; the floating `desktop-app/legal` reference itself is now historically bounded across the frozen Toolkit commit date range;
5. behavior-level source-to-responsibility dossiers until no reachable product capability remains unknown.

Until those close, source research coverage must not be reported as 100%.
