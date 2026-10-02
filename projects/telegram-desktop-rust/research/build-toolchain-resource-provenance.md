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

## External acquisition inventory

A tracked-source scan of the frozen build roots records every explicit URL acquisition entrypoint in the three build surfaces that actually fetch dependencies/tools:

| Surface | URL acquisition mentions | Unique URLs | Ref quality observed |
| --- | ---: | ---: | --- |
| `Telegram/build/prepare/prepare.py` | 45 | 41 | mixture of exact commits, version tags, short commit IDs, mutable `master`/default branches, `latest`, versioned archives and bootstrap URLs |
| `Telegram/build/docker/centos_env/Dockerfile` | 44 | 44 | 11 explicit exact-commit fetches, version/tag/branch clones, one versioned Boost archive, and rustup bootstrap |
| `snap/snapcraft.yaml` | 15 | 15 | exact `source-commit` entries plus tag/branch sources and rustup bootstrap |

The exact-commit class includes, among others, Desktop App patches, zlib, libvpx, Breakpad/Linux syscall support, tg_owt, TDLib/TDE2E and tlottie. The version/tag/branch class includes xz, Little-CMS, brotli, Highway, mozjpeg, Opus, dav1d, OpenH264, WebP, libavif, libjxl, rnnoise, FFmpeg/nv-codec, libheif, PipeWire, OpenAL, OpenSSL, XKB/X11 components, Qt and Ada. `prepare.py` additionally contains mutable or bootstrap surfaces including NuGet `latest`, Chromium gyp `@master`, default-branch clones such as desktop-app/lzma and FFmpeg/gas-preprocessor, and rustup installer endpoints. Snap pins some entries by commit but still uses `source-tag`/`source-branch` for others.

Package-manager acquisition is also reachable and is not hidden by the URL inventory: the scripts invoke pip and platform package managers, while Snap declares `build-packages`/stage packages. Those package names and the package repository snapshot are provenance inputs even where no literal download URL appears in the frozen source.

This closes discovery of the tracked external-acquisition **entrypoints**. It does not convert mutable refs/tags/package-manager resolution into immutable provenance. `research/external-acquisition-ref-resolution.md` records the 2026-10-02 remote commit observed for the frozen scripts' version/tag refs while preserving that limitation. For release closure, every reachable acquisition must either resolve to an immutable digest/commit plus license evidence or be proven irrelevant to the Fabushi distributed build.

## Resource and shader inventory

At the frozen root, `Telegram/Resources` contains 3,073 tracked files. `Telegram/shaders` contains 35 tracked shader source files. The premium 3D model generator consumes eight tracked `.obj` model inputs. The resource tree includes themes, language packs, emoji/resource QRCs, export/webview/picker HTML, sounds, Windows resources/manifests, macOS assets and update-key material.

A filename-based search under the Telegram resource/shader/build/cmake/snap roots found no standalone resource `LICENSE`, `COPYING`, `NOTICE` or `LEGAL` file; files named `copyright.png` are UI icons rather than license declarations. That absence is not evidence that assets are license-free. Per-asset provenance remains a release/legal blocker where Fabushi copies or adapts an upstream asset.


### Exact Telegram/Resources blob inventory and license boundary

The frozen `Telegram/Resources` tree is Git tree `04a4a5357d010bf35bda1767c8b04955f64fc2ca`. Its recursive tree response is `truncated=false`: 3,159 total tree entries and exactly 3,073 blobs. `research/telegram-resources-blob-inventory.tsv` records every blob path, Git blob SHA and byte size, so later copied/adapted evidence can be matched to an immutable upstream object rather than a filename alone.

The 3,073 blobs group by top-level resource root as follows: `icons` 2,736; `animations` 113; `art` 60; `export_html` 46; `uwp` 45; `qrc` 17; `emoji` 12; `langs` 12; `sounds` 11; `update` 4; `bot_webview_shell_html` 3; `export_rich` 2; `picker_html` 2; and the remaining root-level theme/config/text files one each.

Five root theme blobs are identity-bounded: `day-blue.tdesktop-theme` = `ccc3dbdf9dc6d1aed5ae02df7d9430334839900f`; `day-custom-base.tdesktop-theme` = `162d90457d8254d3eefcf772c3ca62545a97508a`; `night-custom-base.tdesktop-theme` = `5fc6424ae95aff90aad8763baa1e905cb18b9a13`; `night-green.tdesktop-theme` = `a60d844be5fb5f709e7af7a7b2fc2be9fd8e57dc`; `night.tdesktop-theme` = `96f788d13efeb7264f40f86ea39f1cb573a4dc12`.

All eleven bundled sound blobs are separately identity-bounded in the TSV, including call busy/connect/end/incoming/outgoing, group-call allowed/connect/end/recording-start/start, and message-incoming. The resource tree contains no `.ttf`, `.otf`, `.woff`, or `.woff2` font binary; paths named `icons/menu/fonts*.png` are menu icons, not font payloads. Font behavior/provenance therefore belongs to the actual font/system/build dependency surfaces rather than being falsely attributed to `Telegram/Resources`.

A recursive filename audit found no resource-local `LICENSE`, `COPYING`, `NOTICE`, `README`, `AUTHORS`, `OFL` or equivalent licensing document in this exact tree. The only filename match for “copyright” is `icons/menu/copyright.png`, a UI icon. Consequently the blob inventory closes **identity/inventory**, not origin/license rights. No theme, sound, icon, animation, emoji, artwork or export resource may be labeled with the root source-code license merely by inheritance; its origin/license remains unresolved unless separate upstream evidence establishes it.

### Copied/adapted provenance rule

Any future Fabushi production change that copies, adapts, transforms, embeds, vendors, regenerates from, or distributes a Telegram-derived source/resource must add a release-review provenance row before acceptance containing at least: upstream repository + exact commit; upstream path + immutable blob/content digest; source license/notice evidence and asset-origin evidence where applicable; Fabushi destination path + resulting digest; whether the relation is copied, adapted, translated, regenerated, or behavior-only; a concise transformation description; required notices/attribution/source-offer obligations for review; platform/package scope; and the exact Fabushi commit/artifact that distributes it.

Behavior-only research must stay distinguishable from copied/adapted material. Recording an upstream file as a research source does not make it a Fabushi dependency; conversely, rewriting logic in Rust/TypeScript does not erase source-informed provenance. A release review must be generated from the **actual Fabushi dependency/artifact graph**, then joined against these provenance rows. Unknown or unmatched distributed inputs fail closed as release blockers.


## Platform packaging surface

The frozen source contains explicit packaging inputs for all three desktop families:

- Windows: `Telegram/Resources/winrc/Telegram.rc`, `Updater.rc`, `Telegram.manifest`, MIDL generation, `build/setup.iss`, and Windows preparation/signing scripts.
- macOS: `Telegram/Telegram.plist`, normal/Lite/Breakpad entitlements, `Images.xcassets`, icon conversion, framework/helper copying, Swift runtime wiring, and Mac Store upload preparation.
- Linux: XDG desktop/service/metainfo files, generated AppStream changelog, Snap `snapcraft.yaml`, DBus portal/notification generation, Docker/prepare build roots, and packaged install rules for icons/resources.

This closes identification of the tracked platform-packaging roots, but not legal review of downloaded dependencies or copied/derived assets.

## Current Fabushi adoption audit

The current FBCP branch was compared against the current PR #20 product skeleton. No tracked Fabushi file is a Telegram `Resources` or `shaders` tree, and no tracked `.qsb`, `.binobj`, or `.obj` payload is present. The current production/build roots (`.github`, `desktop`, `frontend`, `source`, root package/Cargo manifests) contain no invocation of Telegram's frozen `prepare.py`, CentOS Docker build, Snap build, or the specific NuGet/MSYS2/Boost/rustup acquisition endpoints inventoried above. The PR adds no binary file.

The only `telegram.sendMessage` string in the changed production/test surface is a negative Host gateway contract asserting that a Telegram-prefixed command is **not** accepted; it is evidence against a Telegram runtime boundary, not adoption of one.

Therefore the Telegram build acquisition graph, baked QRhi shaders, generated `.binobj` models, and Telegram resource tree are research/provenance inputs, not dependencies or assets of the currently distributed Fabushi branch. Their package-manager/archive licenses do not become Fabushi shipping-dependency licenses merely because Telegram used them. If a future Fabushi change copies, adapts, vendors, invokes, or distributes any such input, that change must add immutable input identity, license/asset provenance, derivation, platform scope, and release review before acceptance. Source-informed implementation provenance remains a separate legal-review surface and is not waived by this non-adoption finding.

## Closure boundary

After this dossier, the source-research blockers are narrower:

1. enumerate and classify all external acquisitions from `prepare.py`, Docker build inputs, Snap/package metadata and any invoked helper scripts, including whether each is exact-commit, immutable archive, tag, branch, `latest`, or other mutable reference;
2. map license/provenance for those non-gitlink external acquisitions;
3. map provenance of any Telegram asset/source actually copied or adapted into distributed Fabushi artifacts;
4. preserve behavior-level capability dossiers and owner resolution independently from build provenance.

The generated-source map, shader source/output shape, tracked packaging-root identification, and recursive gitlink tree are no longer unknowns.
