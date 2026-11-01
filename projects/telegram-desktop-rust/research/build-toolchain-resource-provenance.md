# Frozen Telegram build/toolchain/resource provenance

Status: source-grounded acquisition inventory for frozen upstream `33261535a0e747f125e0ed25486f01e556330677`; not a declaration that Fabushi ships these dependencies.

## Frozen authority

- upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
- recursive root tree: `b806e54584d1db16a7edaef58d9cb0bbcf74540c` (`truncated=false`)
- `Telegram/build/prepare/prepare.py` blob: `8132172acc217b838bfc0d39bbd9a6118ef74795`
- `Telegram/build/docker/centos_env/Dockerfile` blob: `566f6bae924e68f8f446b0e463000edbd6b52546`
- `Telegram/Resources`: 3,073 frozen blobs, separately bound in `telegram-resources-blob-inventory.tsv`.

## Why this exists

The frozen build scripts perform network acquisition outside CMake. Therefore a CMake-only scan cannot close provenance. This inventory records every explicit `git clone/fetch`, `iwr`, `wget`, `curl`, and direct pip-install acquisition line in the two frozen build entrypoints above. Symbolic tags/branches, `latest`, package resolvers, rustup bootstrap URLs, and unpinned clones are intentionally not upgraded to immutable evidence.

## Classification

| Class | Meaning | Count |
| --- | --- | ---: |
| `clone-plus-content-ref` | clone is followed nearby by a content-addressed checkout | 15 |
| `content-addressed-ref` | fetch names a full 40-hex commit | 12 |
| `external-acquisition` | clone/fetch has no immutable pin on the acquisition line | 3 |
| `mutable-or-resolver` | latest/master/package resolver/bootstrap URL remains mutable | 7 |
| `symbolic-tag-or-branch` | clone selects a tag/branch name; symbolic ref can move | 53 |
| `unpinned-clone` | clone/fetch has no immutable pin on the acquisition line | 3 |
| `versioned-url` | URL visibly embeds a version/date but is not content-hash bound | 2 |

## Explicit acquisition entrypoints

| Source | Line | Classification | Frozen command |
| --- | ---: | --- | --- |
| `Telegram/build/prepare/prepare.py` | 521 | `clone-plus-content-ref` | `git clone https://github.com/desktop-app/patches.git` |
| `Telegram/build/prepare/prepare.py` | 526 | `clone-plus-content-ref` | `git clone https://github.com/desktop-app/qt6_highsierra_patches.git qt6_highsierra` |
| `Telegram/build/prepare/prepare.py` | 537 | `versioned-url` | `powershell -Command "iwr -OutFile ./msys64.exe https://github.com/msys2/msys2-installer/releases/download/2026-09-27/msys2-base-x86_64-20260927.sfx.exe"` |
| `Telegram/build/prepare/prepare.py` | 556 | `mutable-or-resolver` | `pip install pywin32 six meson` |
| `Telegram/build/prepare/prepare.py` | 563 | `mutable-or-resolver` | `powershell -Command "iwr -OutFile ./NuGet/nuget.exe https://dist.nuget.org/win-x86-commandline/latest/nuget.exe"` |
| `Telegram/build/prepare/prepare.py` | 568 | `external-acquisition` | `powershell -Command "iwr -OutFile ./jom.zip https://master.qt.io/official_releases/jom/jom_1_1_3.zip"` |
| `Telegram/build/prepare/prepare.py` | 576 | `clone-plus-content-ref` | `git clone https://github.com/desktop-app/gyp.git` |
| `Telegram/build/prepare/prepare.py` | 580 | `mutable-or-resolver` | `python3 -m pip install \\` |
| `Telegram/build/prepare/prepare.py` | 589 | `mutable-or-resolver` | `powershell -Command "iwr -OutFile ./rustup-init.exe https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe"` |
| `Telegram/build/prepare/prepare.py` | 597 | `mutable-or-resolver` | `wget -O rustup-init.sh https://sh.rustup.rs` |
| `Telegram/build/prepare/prepare.py` | 609 | `unpinned-clone` | `git clone https://github.com/desktop-app/lzma.git` |
| `Telegram/build/prepare/prepare.py` | 622 | `symbolic-tag-or-branch` | `git clone -b v5.4.5 https://github.com/tukaani-project/xz.git` |
| `Telegram/build/prepare/prepare.py` | 633 | `clone-plus-content-ref` | `git clone https://github.com/madler/zlib.git` |
| `Telegram/build/prepare/prepare.py` | 671 | `symbolic-tag-or-branch` | `git clone -b v4.1.5 https://github.com/mozilla/mozjpeg.git` |
| `Telegram/build/prepare/prepare.py` | 708 | `symbolic-tag-or-branch` | `git clone -b openssl-3.2.1 https://github.com/openssl/openssl openssl3` |
| `Telegram/build/prepare/prepare.py` | 755 | `symbolic-tag-or-branch` | `git clone -b v1.5.2 https://github.com/xiph/opus.git` |
| `Telegram/build/prepare/prepare.py` | 773 | `clone-plus-content-ref` | `git clone https://github.com/desktop-app/rnnoise.git` |
| `Telegram/build/prepare/prepare.py` | 804 | `external-acquisition` | `wget --timeout=30 --tries=2 -O libiconv.tar.gz ftp://ftp.gnu.org/gnu/libiconv/libiconv-$VERSION.tar.gz \|\| wget -O libiconv.tar.gz https://ftp.gnu.org/pub/gnu/libiconv/libiconv-$VERSION.tar.gz` |
| `Telegram/build/prepare/prepare.py` | 825 | `unpinned-clone` | `git clone https://github.com/FFmpeg/gas-preprocessor` |
| `Telegram/build/prepare/prepare.py` | 833 | `symbolic-tag-or-branch` | `git clone -b 1.5.4 https://code.videolan.org/videolan/dav1d.git` |
| `Telegram/build/prepare/prepare.py` | 898 | `symbolic-tag-or-branch` | `git clone -b v2.6.0 https://github.com/cisco/openh264.git` |
| `Telegram/build/prepare/prepare.py` | 956 | `symbolic-tag-or-branch` | `git clone -b v1.4.2 https://github.com/AOMediaCodec/libavif.git` |
| `Telegram/build/prepare/prepare.py` | 985 | `symbolic-tag-or-branch` | `git clone -b v1.6.0 https://github.com/webmproject/libwebp.git` |
| `Telegram/build/prepare/prepare.py` | 1024 | `symbolic-tag-or-branch` | `git clone -b v0.12.0 --recursive --shallow-submodules https://github.com/libjxl/libjxl.git` |
| `Telegram/build/prepare/prepare.py` | 1067 | `unpinned-clone` | `git clone https://github.com/webmproject/libvpx.git` |
| `Telegram/build/prepare/prepare.py` | 1129 | `symbolic-tag-or-branch` | `git clone -b lcms2.16 https://github.com/mm2/Little-CMS.git liblcms2` |
| `Telegram/build/prepare/prepare.py` | 1164 | `symbolic-tag-or-branch` | `git clone -b n12.1.14.0 https://github.com/FFmpeg/nv-codec-headers.git` |
| `Telegram/build/prepare/prepare.py` | 1168 | `symbolic-tag-or-branch` | `git clone -b boost-1.83.0 https://github.com/boostorg/regex.git` |
| `Telegram/build/prepare/prepare.py` | 1172 | `symbolic-tag-or-branch` | `git clone -b n8.1.3 https://github.com/FFmpeg/FFmpeg.git ffmpeg` |
| `Telegram/build/prepare/prepare.py` | 1355 | `symbolic-tag-or-branch` | `git clone -b v1.23.5 https://github.com/strukturag/libheif.git` |
| `Telegram/build/prepare/prepare.py` | 1417 | `clone-plus-content-ref` | `git clone https://github.com/telegramdesktop/openal-soft.git` |
| `Telegram/build/prepare/prepare.py` | 1447 | `clone-plus-content-ref` | `git clone https://chromium.googlesource.com/breakpad/breakpad stackwalk` |
| `Telegram/build/prepare/prepare.py` | 1452 | `symbolic-tag-or-branch` | `git clone -b release-1.11.0 https://github.com/google/googletest src/testing` |
| `Telegram/build/prepare/prepare.py` | 1453 | `clone-plus-content-ref` | `git clone https://chromium.googlesource.com/linux-syscall-support src/third_party/lss` |
| `Telegram/build/prepare/prepare.py` | 1487 | `clone-plus-content-ref` | `git clone https://chromium.googlesource.com/breakpad/breakpad` |
| `Telegram/build/prepare/prepare.py` | 1492 | `symbolic-tag-or-branch` | `git clone -b release-1.11.0 https://github.com/google/googletest src/testing` |
| `Telegram/build/prepare/prepare.py` | 1517 | `clone-plus-content-ref` | `git clone https://chromium.googlesource.com/linux-syscall-support src/third_party/lss` |
| `Telegram/build/prepare/prepare.py` | 1525 | `clone-plus-content-ref` | `git clone https://github.com/desktop-app/crashpad.git` |
| `Telegram/build/prepare/prepare.py` | 1587 | `clone-plus-content-ref` | `git clone https://github.com/desktop-app/tg_angle.git` |
| `Telegram/build/prepare/prepare.py` | 1599 | `symbolic-tag-or-branch` | `git clone -b v$QT-lts-lgpl https://github.com/qt/qt5.git qt_$QT` |
| `Telegram/build/prepare/prepare.py` | 1668 | `symbolic-tag-or-branch` | `git clone -b """ + branch + """ https://github.com/qt/qt5.git qt_$QT` |
| `Telegram/build/prepare/prepare.py` | 1785 | `clone-plus-content-ref` | `git clone https://github.com/desktop-app/tg_owt.git` |
| `Telegram/build/prepare/prepare.py` | 1883 | `symbolic-tag-or-branch` | `git clone -b v3.2.4 https://github.com/ada-url/ada.git` |
| `Telegram/build/prepare/prepare.py` | 1904 | `clone-plus-content-ref` | `git clone https://github.com/tdlib/td.git tde2e` |
| `Telegram/build/prepare/prepare.py` | 1983 | `clone-plus-content-ref` | `git clone https://github.com/dkaraush/tlottie.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 18 | `external-acquisition` | `lld ccache nasm file which wget perl-open perl-XML-Parser perl-IPC-Cmd \` |
| `Telegram/build/docker/centos_env/Dockerfile` | 30 | `mutable-or-resolver` | `python3 -m pip install meson ninja` |
| `Telegram/build/docker/centos_env/Dockerfile` | 88 | `content-addressed-ref` | `git fetch --depth=1 origin ecf7bb51a92a0fb16834c5b698570ab25f9f1d21` |
| `Telegram/build/docker/centos_env/Dockerfile` | 163 | `content-addressed-ref` | `git fetch --depth=1 origin 593bd3c5ab2ce3294306426ffd1f01c5502caec8` |
| `Telegram/build/docker/centos_env/Dockerfile` | 173 | `content-addressed-ref` | `git fetch --depth=1 origin e3dc0a85b7032e98380dec011bc8f2c2ee0d8fca` |
| `Telegram/build/docker/centos_env/Dockerfile` | 192 | `symbolic-tag-or-branch` | `git clone -b v5.8.1 --depth=1 https://github.com/tukaani-project/xz.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 203 | `symbolic-tag-or-branch` | `git clone -b lcms2.15 --depth=1 https://github.com/mm2/Little-CMS.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 214 | `symbolic-tag-or-branch` | `git clone -b v1.2.0 --depth=1 https://github.com/google/brotli.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 225 | `symbolic-tag-or-branch` | `git clone -b 1.4.0 --depth=1 https://github.com/google/highway.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 236 | `symbolic-tag-or-branch` | `git clone -b v4.1.5 --depth=1 https://github.com/mozilla/mozjpeg.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 252 | `symbolic-tag-or-branch` | `git clone -b v1.5.2 --depth=1 https://github.com/xiph/opus.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 263 | `symbolic-tag-or-branch` | `git clone -b 1.5.4 --depth=1 https://github.com/videolan/dav1d.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 278 | `symbolic-tag-or-branch` | `git clone -b v2.6.0 --depth=1 https://github.com/cisco/openh264.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 292 | `content-addressed-ref` | `git fetch --depth=1 origin 12f3a2ac603e8f10742105519e0cd03c3b8f71dd` |
| `Telegram/build/docker/centos_env/Dockerfile` | 311 | `symbolic-tag-or-branch` | `git clone -b v1.6.0 --depth=1 https://github.com/webmproject/libwebp.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 333 | `symbolic-tag-or-branch` | `git clone -b v1.4.2 --depth=1 https://github.com/AOMediaCodec/libavif.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 348 | `symbolic-tag-or-branch` | `git clone -b v0.12.0 --depth=1 https://github.com/libjxl/libjxl.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 373 | `symbolic-tag-or-branch` | `git clone -b v0.2 --depth=1 https://github.com/xiph/rnnoise.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 385 | `symbolic-tag-or-branch` | `git clone -b xcb-proto-1.16.0 --depth=1 https://github.com/gitlab-freedesktop-mirrors/xcbproto.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 398 | `symbolic-tag-or-branch` | `git clone -b libxcb-1.16 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxcb.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 411 | `symbolic-tag-or-branch` | `git clone -b xcb-util-wm-0.4.2 --depth=1 --recursive --shallow-submodules \` |
| `Telegram/build/docker/centos_env/Dockerfile` | 423 | `symbolic-tag-or-branch` | `git clone -b xcb-util-0.4.1-gitlab --depth=1 --recursive --shallow-submodules \` |
| `Telegram/build/docker/centos_env/Dockerfile` | 437 | `symbolic-tag-or-branch` | `git clone -b xcb-util-image-0.4.1-gitlab --depth=1 --recursive --shallow-submodules \` |
| `Telegram/build/docker/centos_env/Dockerfile` | 452 | `content-addressed-ref` | `git fetch --depth=1 origin ef5cb393d27511ba511c68a54f8ff7b9aab4a384` |
| `Telegram/build/docker/centos_env/Dockerfile` | 467 | `content-addressed-ref` | `git fetch --depth=1 origin 5ad9853d6ddcac394d42dd2d4e34436b5db9da39` |
| `Telegram/build/docker/centos_env/Dockerfile` | 486 | `content-addressed-ref` | `git fetch --depth=1 origin 4929f6051658ba5424b41703a1fb63f9db896065` |
| `Telegram/build/docker/centos_env/Dockerfile` | 501 | `symbolic-tag-or-branch` | `git clone -b libXext-1.3.5 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxext.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 512 | `symbolic-tag-or-branch` | `git clone -b libXtst-1.2.4 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxtst.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 523 | `symbolic-tag-or-branch` | `git clone -b libXfixes-5.0.3 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxfixes.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 536 | `symbolic-tag-or-branch` | `git clone -b libXv-1.0.12 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxv.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 547 | `symbolic-tag-or-branch` | `git clone -b libXrandr-1.5.3 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxrandr.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 558 | `symbolic-tag-or-branch` | `git clone -b libXrender-0.9.11 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxrender.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 569 | `symbolic-tag-or-branch` | `git clone -b libXdamage-1.1.6 --depth=1 https://github.com/gitlab-freedesktop-mirrors/libxdamage.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 580 | `symbolic-tag-or-branch` | `git clone -b libXcomposite-0.4.6 --depth=1 \` |
| `Telegram/build/docker/centos_env/Dockerfile` | 592 | `symbolic-tag-or-branch` | `git clone -b n12.1.14.0 --depth=1 https://github.com/FFmpeg/nv-codec-headers.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 607 | `symbolic-tag-or-branch` | `git clone -b n8.1.3 --depth=1 https://github.com/FFmpeg/FFmpeg.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 760 | `symbolic-tag-or-branch` | `git clone -b v1.23.5 --depth=1 https://github.com/strukturag/libheif.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 791 | `symbolic-tag-or-branch` | `git clone -b 0.3.62 --depth=1 https://github.com/PipeWire/pipewire.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 809 | `symbolic-tag-or-branch` | `git clone -b 1.25.2 --depth=1 https://github.com/kcat/openal-soft.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 825 | `symbolic-tag-or-branch` | `git clone -b openssl-3.2.1 --depth=1 https://github.com/openssl/openssl.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 839 | `symbolic-tag-or-branch` | `git clone -b xkbcommon-1.6.0 --depth=1 https://github.com/xkbcommon/libxkbcommon.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 873 | `symbolic-tag-or-branch` | `git clone -b v$QT --depth=1 https://github.com/qt/qt5.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 903 | `versioned-url` | `curl -sSL https://archives.boost.io/release/1.90.0/source/boost_1_90_0.tar.gz \| tar -xz` |
| `Telegram/build/docker/centos_env/Dockerfile` | 916 | `content-addressed-ref` | `git fetch --depth=1 origin 9aebd3d8ef5a246deb2c929b5666aaba160ebce6` |
| `Telegram/build/docker/centos_env/Dockerfile` | 921 | `content-addressed-ref` | `git fetch --depth=1 origin 29164a80da4d41134950d76d55199ea33fbb9613` |
| `Telegram/build/docker/centos_env/Dockerfile` | 954 | `content-addressed-ref` | `git fetch --depth=1 origin e2d0e88d1bde6cc600da5dc92581dc97e4c1e685` |
| `Telegram/build/docker/centos_env/Dockerfile` | 966 | `symbolic-tag-or-branch` | `git clone -b v3.2.4 --depth=1 https://github.com/ada-url/ada.git` |
| `Telegram/build/docker/centos_env/Dockerfile` | 984 | `content-addressed-ref` | `git fetch --depth=1 origin 51743dfd01dff6179e2d8f7095729caa4e2222e9` |
| `Telegram/build/docker/centos_env/Dockerfile` | 998 | `mutable-or-resolver` | `wget -O rustup-init.sh https://sh.rustup.rs` |
| `Telegram/build/docker/centos_env/Dockerfile` | 1007 | `content-addressed-ref` | `git fetch --depth=1 origin 31f1b542f88e7b4be9a01e749920d857535fc715` |

## Current Fabushi adoption boundary

PR #27 (`work/fbcp-phase1-main-20261005`) currently changes no `Telegram/Resources`, `Telegram/shaders`, `Telegram/build`, or `Telegram/cmake` payload and adds no `.qsb`, `.binobj`, or `.obj` Telegram payload. These upstream acquisitions are therefore research/provenance inputs for source-informed capability analysis, not shipping dependencies of this Phase 1 branch.

This non-adoption finding does not waive legal/source review. Any future copied/adapted upstream source/resource must add a release-review row binding the Fabushi target path to the exact upstream blob, transformation, license basis, and applicable tests before packaging.

## Open closure

- Resolve/license-review dependencies actually adopted by distributed Fabushi, not every Telegram historical build dependency.
- For symbolic/mutable acquisition needed to understand a behavior, resolve the historical object only when materially required; otherwise record explicit non-adoption.
- Keep per-file/mixed-license provenance open until source-backed mappings exist. Do not cite absent license-map artifacts as completed evidence.
- Continue behavior-level source-to-responsibility dossiers until no reachable applicable product capability is unknown.
