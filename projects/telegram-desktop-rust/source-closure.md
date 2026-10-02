# Telegram Frozen Source Closure Evidence

Status: P0 evidence; not complete  
Frozen root: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
Captured: 2026-10-02

## Root tree

The GitHub recursive tree for the frozen root returned `truncated=false`. The root contains 35 direct gitlinks pinned in `upstream.lock.json`. The recursive SourceFiles inventory exposed 118 directories at depth <= 4, including product areas that were not explicit in the initial hand-authored capability list.

## Direct gitlink recursion

Every one of the 35 direct GitHub-hosted gitlink commits was queried with the GitHub recursive-tree API. None of those tree responses was truncated.

Thirty-two direct gitlinks contain no nested gitlinks. Three contain one nested gitlink each:

| Parent | Nested path | Pinned commit | Result |
| --- | --- | --- | --- |
| `desktop-app/cmake_helpers@7a6abdae…` | `external/glib/cppgir` | `47cf94f83b54cda59018135601e19d7fb0c77776` | URL resolves to GitLab; commit-specific recursive tree still pending |
| `PJK/libcbor@170bee2b…` | `doxygen-theme` | `46111c61a9f49b7a9886127e679d4317478fab1c` | recursive tree verified, `truncated=false`, no further gitlinks |
| `ericniebler/range-v3@a8147793…` | `doc/gh-pages` | `2dae74bb693e42d850fb0adcc9045c5b71fbdeae` | recursive tree verified, `truncated=false`, no further gitlinks |

The `cppgir` parent `.gitmodules` pins its URL to `https://gitlab.com/mnauw/cppgir.git`. The available GitHub connector cannot prove that GitLab commit tree and the web fetch did not expose the commit-specific raw `.gitmodules`. Therefore recursive submodule closure remains **not complete** rather than being inferred from the current default branch.

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

## Build/resource closure still open

`Telegram/CMakeLists.txt` proves that the product links or generates through codegen, lib_storage, lib_ui, lib_webrtc, lib_webview, tgcalls, FIDO2, ffmpeg, Stripe, MTProto/scheme generators, language generators, update-key generation, platform MIDL on Windows, and Apple Swift runtime. `Telegram/Resources` contains themes, language bundles, emoji, export templates, webview/picker HTML, sounds, update assets and platform packaging resources.

P0 must still enumerate:

1. build-time downloads and non-gitlink external packages;
2. generated outputs and their source inputs;
3. patches/shaders/platform packaging inputs;
4. dependency license expressions and copied/derived asset provenance;
5. the external GitLab `cppgir` recursive leaf;
6. behavior-level responsibility dossiers for each reachable product capability.

Until those close, source research coverage must not be reported as 100%.
