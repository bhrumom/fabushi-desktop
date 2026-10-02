# Frozen Telegram build/toolchain/resource provenance dossier

Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Captured on: 2026-10-02
Purpose: source-closure evidence only. None of these dependencies, assets, build scripts, protocols, or packaging paths are authorized as Fabushi runtime/product architecture.

## Generator input -> output map

The frozen `Telegram/CMakeLists.txt` and `Telegram/cmake/*.cmake` files make the generated-source chain explicit:

| Generator | Frozen input | Build output / consumer |
| --- | --- | --- |
| language | `Telegram/Resources/langs/lang.strings` + `codegen_lang` | `gen/lang_auto.cpp`, `gen/lang_auto.h`, `gen/lang_auto_counts.h`, `gen/lang_auto_keys.h` plus per-source subset headers |
| numbers | `Telegram/Resources/numbers.txt` + `codegen_numbers` | `gen/numbers.cpp`, `gen/numbers.h` |
| TL scheme | `Telegram/SourceFiles/codegen/scheme/codegen_scheme.py`, `lib_tl/tl/generate_tl.py`, scheme files | `gen/scheme.cpp`, `gen/scheme.h`, dump-to-text sources |
| Windows MIDL | `windows_quiethours.idl`, `windows_toastactivator.idl` | generated `_i.c` and `_h.h` files under the build `gen` directory |
| update keys | `Telegram/Resources/update/*` + `generate_update_keys.cmake`/`update_keys_header.cmake` | generated update-key data consumed by `core/update_keys.cpp` |
| Linux DBus | portal/notification XML inputs | generated DBus bindings via `cmake_helpers/external/glib/generate_dbus.cmake` |
| AppStream changelog | root `changelog.txt` + metainfo template | packaged Linux AppStream metainfo output |
| 3D premium models | eight `Telegram/Resources/art/**/*.obj` files + `cmake/obj2binobj.py` | `.binobj` files and generated `models.qrc` |
| QRhi shaders | 35 tracked `Telegram/shaders/*.{vert,frag,comp}` files + Qt `qsb` | `.qsb` binaries and generated `shaders.qrc` |

This closes the previously unknown generator input/output shape at the source-research level. It does not authorize copying generated Telegram outputs into Fabushi.

## Build-time external acquisition

A whole-tree CMake scan found no CMake `file(DOWNLOAD)`, `FetchContent`, or `ExternalProject_Add` primitive. That narrow fact is not equivalent to “no build-time downloads”. The frozen build toolchain contains explicit network acquisition outside CMake:

- `Telegram/build/prepare/prepare.py` clones/fetches source and downloads tools/archive payloads with `git`, PowerShell `iwr`, and `wget`.
- `Telegram/build/docker/centos_env/Dockerfile` clones/fetches a large pinned/tagged Linux dependency set and also uses `curl`/`wget` for archive/tool bootstrap.
- examples visible at this exact ref include desktop-app patches, MSYS2, NuGet, jom, rustup, xz, zlib, mozjpeg, OpenSSL, Opus, FFmpeg, libavif, libjxl, libheif, PipeWire, OpenAL, Qt, Boost, Breakpad, tg_owt, TDLib and other X11/media libraries.

Therefore build-time external acquisition is a real provenance surface. Each fetched dependency must be treated as an input to research/release provenance even when it is not a root gitlink. Mutable refs such as `latest`, `master`, or branch/tag-only fetches are weaker provenance than exact commits and must not be promoted to exact-ref closure without additional evidence.

## Resource and shader inventory

At the frozen root, `Telegram/Resources` contains 3,073 tracked files. `Telegram/shaders` contains 35 tracked shader source files. The premium 3D model generator consumes eight tracked `.obj` model inputs. The resource tree includes themes, language packs, emoji/resource QRCs, export/webview/picker HTML, sounds, Windows resources/manifests, macOS assets and update-key material.

A filename-based search under the Telegram resource/shader/build/cmake/snap roots found no standalone resource `LICENSE`, `COPYING`, `NOTICE` or `LEGAL` file; files named `copyright.png` are UI icons rather than license declarations. That absence is not evidence that assets are license-free. Per-asset provenance remains a release/legal blocker where Fabushi copies or adapts an upstream asset.

## Platform packaging surface

The frozen source contains explicit packaging inputs for all three desktop families:

- Windows: `Telegram/Resources/winrc/Telegram.rc`, `Updater.rc`, `Telegram.manifest`, MIDL generation, `build/setup.iss`, and Windows preparation/signing scripts.
- macOS: `Telegram/Telegram.plist`, normal/Lite/Breakpad entitlements, `Images.xcassets`, icon conversion, framework/helper copying, Swift runtime wiring, and Mac Store upload preparation.
- Linux: XDG desktop/service/metainfo files, generated AppStream changelog, Snap `snapcraft.yaml`, DBus portal/notification generation, Docker/prepare build roots, and packaged install rules for icons/resources.

This closes identification of the tracked platform-packaging roots, but not legal review of downloaded dependencies or copied/derived assets.

## Closure boundary

After this dossier, the source-research blockers are narrower:

1. enumerate and classify all external acquisitions from `prepare.py`, Docker build inputs, Snap/package metadata and any invoked helper scripts, including whether each is exact-commit, immutable archive, tag, branch, `latest`, or other mutable reference;
2. map license/provenance for those non-gitlink external acquisitions;
3. map provenance of any Telegram asset/source actually copied or adapted into distributed Fabushi artifacts;
4. preserve behavior-level capability dossiers and owner resolution independently from build provenance.

The generated-source map, shader source/output shape, tracked packaging-root identification, and recursive gitlink tree are no longer unknowns.
