# Telegram Frozen Source Closure Evidence

Status: P0 evidence; recursive gitlink, tracked generator/packaging roots, and external-acquisition entrypoints discovered; immutable external-resolution/resource-license/behavior closure not complete
Current Stage A authority root: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4` (tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`)  
Reconciled: 2026-10-31; Stage A manual Git/API evidence only, no CI executed

## Root tree

The GitHub recursive tree for the fixed U returned `truncated=false`: 7,194 entries total, including 6,859 blobs, 299 trees, and 36 direct gitlinks. Root `.gitmodules` is blob `2a71e861fab57dfd74d66e787867f2ede203a099`. These are exact fixed-U identity facts, not acceptance or full recursive-source-completeness evidence.

## Direct gitlink recursion

The current `upstream.lock.json` records recursive GitHub-tree reads for all 36 direct gitlinks at fixed U. Three direct gitlinks expose one nested gitlink each; the remaining 33 expose none in those recorded scans. The two GitHub-hosted nested targets remain resolved; the GitLab-hosted cppgir target remains fail-closed because its original-host recursive tree/blob set is not available in the current execution environment.

| Parent | Nested path | Pinned commit | Result |
| --- | --- | --- | --- |
| `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095` | `external/glib/cppgir` | `47cf94f83b54cda59018135601e19d7fb0c77776` | parent `.gitmodules` blob `c92fbea3467f72e188cba4cbfe51c8c68a7e2e77` pins `https://gitlab.com/mnauw/cppgir.git`, and the parent tree pins this exact gitlink commit; original GitLab recursive tree/blob evidence remains unavailable here, so this nested chain is **not** current-closure complete |
| `PJK/libcbor@170bee2b…` | `doxygen-theme` | `46111c61a9f49b7a9886127e679d4317478fab1c` | recursive tree verified, `truncated=false`, no further gitlinks |
| `ericniebler/range-v3@5ba39112d2eafbb3c851a16a537f29b445022998` | `doc/gh-pages` | `2dae74bb693e42d850fb0adcc9045c5b71fbdeae` | recursive tree verified, `truncated=false`, no further gitlinks |

For fixed U, `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095` has `.gitmodules` blob `c92fbea3467f72e188cba4cbfe51c8c68a7e2e77`, which pins `external/glib/cppgir` to `https://gitlab.com/mnauw/cppgir.git`; its recursive GitHub tree pins the gitlink itself to `47cf94f83b54cda59018135601e19d7fb0c77776`. Historical evidence in this document previously asserted a fully paged GitLab tree and an `expected-lite` nested gitlink. The current execution environment cannot retrieve the original GitLab recursive tree/blob set, so those historical claims are not promoted into current fixed-U closure. A standalone GitHub read confirms `nonstd-lite/expected-lite@95b9cb015fa17baa749c2b396b335906e1596a9e` has a non-truncated 55-entry tree with no nested gitlinks, but that does not prove it is reachable from the pinned cppgir commit without the original cppgir tree/.gitmodules evidence.

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
| native third parties | direct gitlinks including `tgcalls`, `lib_webrtc`, `lib_webview`, FIDO2 and media/storage libraries | all 36 direct gitlinks have recorded fixed-U recursive scans; two GitHub-hosted nested links are resolved, while `cppgir` remains original-host unresolved; non-gitlink external acquisitions remain separate |

No item in this table is an instruction to ship Telegram dependencies. It is source/provenance research required before declaring capability/source coverage complete.

## Build/resource closure still open

`Telegram/CMakeLists.txt` proves that the product links or generates through codegen, lib_storage, lib_ui, lib_webrtc, lib_webview, tgcalls, FIDO2, ffmpeg, Stripe, MTProto/scheme generators, language generators, update-key generation, platform MIDL on Windows, and Apple Swift runtime. An exact GitHub recursive-tree inventory for frozen root `33261535a0e747f125e0ed25486f01e556330677` (`truncated=false`) records 3,073 files under `Telegram/Resources`, 35 shader sources, eight `.obj` model inputs, 36 files under `Telegram/build`, 30 under `Telegram/cmake`, and the Snap packaging input. `research/telegram-resources-blob-inventory.tsv` binds every resource path to its frozen Git blob SHA and size. `research/build-toolchain-resource-provenance.md` binds the frozen `prepare.py`/Linux Docker blobs and enumerates their explicit external-acquisition entrypoints without upgrading symbolic or mutable inputs to immutable evidence.

A previous CMake-only scan correctly found no CMake `file(DOWNLOAD)`, `FetchContent`, or `ExternalProject_Add`, but that scope was too narrow. The frozen `Telegram/build/prepare/prepare.py` and `Telegram/build/docker/centos_env/Dockerfile` explicitly perform network acquisition using `git clone/fetch`, PowerShell `iwr`, `wget`, and `curl`. Build-time external acquisition therefore remains a real provenance blocker and must be enumerated rather than reported absent.

The current PR #27 changed-files audit finds no Telegram prepare/Docker/Snap invocation, no tracked Telegram resource/shader tree, no `.qsb`/`.binobj`/`.obj` payload, and no Telegram build/resource payload in the Phase 1 production delta. Those upstream build/resource inputs are therefore research/provenance inputs for this branch rather than shipping dependencies. This does not turn mutable historical acquisition into immutable evidence and does not waive source-informed legal review.

P0 must still close:

1. historical immutable build-input evidence only where needed to understand a source-derived behavior/build responsibility, while recording explicit non-adoption for Telegram-only dependencies not used by Fabushi;
2. dependency/license review for dependencies actually adopted by distributed Fabushi, plus source-informed GPL/third-party review;
3. copied/derived provenance for any upstream resource/shader/model/source content that a future or current production change actually copies or adapts; `research/telegram-resources-blob-inventory.tsv` now binds all 3,073 frozen resource blobs by path/SHA/size and `research/build-toolchain-resource-provenance.md` defines the mandatory copied/adapted release-review row, while the current FBCP branch still has no Telegram resource/shader/model payload;
4. mixed-license/per-file provenance needed to interpret researched upstream sources where the selected behavior depends on those files; the previously referenced `research/mixed-license-per-file-provenance.md` and `research/license-maps/*.tsv` artifacts are not present on authoritative PR #27 and must not be treated as completed evidence. Recreate source-backed mappings only for material copied/adapted or behaviorally relied-on inputs, and keep broader resource-level mixed-license cases open;
5. behavior-level source-to-responsibility dossiers until no reachable product capability remains unknown.

Until those close, source research coverage must not be reported as 100%.
