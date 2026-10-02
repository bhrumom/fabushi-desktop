# External acquisition ref resolution

Frozen Telegram source: `33261535a0e747f125e0ed25486f01e556330677`
Remote-ref observation date: 2026-10-02
Scope: provenance research for build dependencies only; none of these are authorized Fabushi runtime dependencies.

## Version/tag refs observed

The frozen build scripts use the refs below. `git ls-remote` was used on htch-runtime to record what each remote ref resolves to on 2026-10-02. For annotated tags, the peeled commit is recorded. This converts a symbolic ref into a concrete research identity for the observation date, but **does not prove the ref was immutable at the frozen source commit date**. Release provenance must retain an immutable digest/commit from the actual build input.

| Remote | Ref in frozen build input | Observed commit | Evidence strength |
| --- | --- | --- | --- |
| `https://github.com/tukaani-project/xz.git` | `v5.8.1` | `a522a226545730551f7e7c2685fab27cf567746c` | remote tag/ref observed 2026-10-02 |
| `https://github.com/tukaani-project/xz.git` | `v5.4.5` | `49053c0a649f4c8bd2b8d97ce915f401fbc0f3d9` | remote tag/ref observed 2026-10-02 |
| `https://github.com/mm2/Little-CMS.git` | `lcms2.15` | `d0752309ff93f62be246ee9bf18338107f30d485` | remote tag/ref observed 2026-10-02 |
| `https://github.com/mm2/Little-CMS.git` | `lcms2.16` | `453bafeb85b4ef96498866b7a8eadcc74dff9223` | remote tag/ref observed 2026-10-02 |
| `https://github.com/google/brotli.git` | `v1.2.0` | `028fb5a23661f123017c060daa546b55cf4bde29` | remote tag/ref observed 2026-10-02 |
| `https://github.com/google/highway.git` | `1.4.0` | `2607d3b5b0113992fe84d3848859eae13b3b52c1` | remote tag/ref observed 2026-10-02 |
| `https://github.com/mozilla/mozjpeg.git` | `v4.1.5` | `6c9f0897afa1c2738d7222a0a9ab49e8b536a267` | remote tag/ref observed 2026-10-02 |
| `https://github.com/xiph/opus.git` | `v1.5.2` | `ddbe48383984d56acd9e1ab6a090c54ca6b735a6` | remote tag/ref observed 2026-10-02 |
| `https://github.com/videolan/dav1d.git` | `1.5.4` | `54706fc6bc0cdecab7e9593974a4039cc038fca7` | remote tag/ref observed 2026-10-02 |
| `https://code.videolan.org/videolan/dav1d.git` | `1.5.4` | `unresolved` | unresolved |
| `https://github.com/cisco/openh264.git` | `v2.6.0` | `652bdb7719f30b52b08e506645a7322ff1b2cc6f` | remote tag/ref observed 2026-10-02 |
| `https://github.com/webmproject/libwebp.git` | `v1.6.0` | `4fa21912338357f89e4fd51cf2368325b59e9bd9` | remote tag/ref observed 2026-10-02 |
| `https://github.com/AOMediaCodec/libavif.git` | `v1.4.2` | `c5240fc79fe5c2407e10afd35f5505ef6333ea49` | remote tag/ref observed 2026-10-02 |
| `https://github.com/AOMediaCodec/libavif.git` | `v1.3.0` | `1aadfad932c98c069a1204261b1856f81f3bc199` | remote tag/ref observed 2026-10-02 |
| `https://github.com/libjxl/libjxl.git` | `v0.12.0` | `a7a9c787341cf703dede03c2009fa460cae5e5df` | remote tag/ref observed 2026-10-02 |
| `https://github.com/xiph/rnnoise.git` | `v0.2` | `904a876dce1f9ab8860c0a5000ed151f9f6eef58` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/xcbproto.git` | `xcb-proto-1.16.0` | `98eeebfc2d7db5377b85437418fb942ea30ffc0d` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxcb.git` | `libxcb-1.16` | `cc4b93c9cd93bad15b7106747b0213e4b9c53a1c` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxext.git` | `libXext-1.3.5` | `67e82cc395f44faddd83ec3d0d21624fe91c6fc0` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxtst.git` | `libXtst-1.2.4` | `99b89c3bcb0ebb0b6dd86bfdc9d276715eaea889` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxfixes.git` | `libXfixes-5.0.3` | `84df9cb81cc31bbed27ba241a23ae04f61da57db` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxv.git` | `libXv-1.0.12` | `d419928942dbf1897c9627475aa4a2828a81240f` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxrandr.git` | `libXrandr-1.5.3` | `3387129532899eaeee3477a2d92fa662d7292a84` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxrender.git` | `libXrender-0.9.11` | `e5e23272394c90731debd7e18dd167e8c25b5c15` | remote tag/ref observed 2026-10-02 |
| `https://github.com/gitlab-freedesktop-mirrors/libxdamage.git` | `libXdamage-1.1.6` | `1f20df4fd7c132f55e924dc5ec3f270cd54704d0` | remote tag/ref observed 2026-10-02 |
| `https://github.com/FFmpeg/nv-codec-headers.git` | `n12.1.14.0` | `1889e62e2d35ff7aa9baca2bceb14f053785e6f1` | remote tag/ref observed 2026-10-02 |
| `https://github.com/FFmpeg/FFmpeg.git` | `n8.1.3` | `1041abdc962f4cc4f394aa8de9dc5236c0c3b9e7` | remote tag/ref observed 2026-10-02 |
| `https://github.com/FFmpeg/FFmpeg.git` | `n6.1.6` | `f1e3a2bf7a2f2cde936d1ed97f09a26853d20125` | remote tag/ref observed 2026-10-02 |
| `https://github.com/strukturag/libheif.git` | `v1.23.5` | `413e2a87e6a70b3eccc3a3adc5801179dd2d9e00` | remote tag/ref observed 2026-10-02 |
| `https://github.com/PipeWire/pipewire.git` | `0.3.62` | `3a443b4e1c9730675c7de0453a6279ab9ee263fd` | remote tag/ref observed 2026-10-02 |
| `https://github.com/kcat/openal-soft.git` | `1.25.2` | `b2c48f7718ef3fcf67921a8b6534c4914e328970` | remote tag/ref observed 2026-10-02 |
| `https://github.com/kcat/openal-soft.git` | `1.24.3` | `dc7d7054a5b4f3bec1dc23a42fd616a0847af948` | remote tag/ref observed 2026-10-02 |
| `https://github.com/openssl/openssl.git` | `openssl-3.2.1` | `a7e992847de83aa36be0c399c89db3fb827b0be2` | remote tag/ref observed 2026-10-02 |
| `https://github.com/xkbcommon/libxkbcommon.git` | `xkbcommon-1.6.0` | `d2a08f761c796733e42fac4099f5c38d443e88e1` | remote tag/ref observed 2026-10-02 |
| `https://github.com/qt/qt5.git` | `v6.11.2` | `713a36536903d172f9e6737584d428753c119496` | remote tag/ref observed 2026-10-02 |
| `https://github.com/ada-url/ada.git` | `v3.2.4` | `010f7c45aeaff1205452e7da2df8702cf725fb3e` | remote tag/ref observed 2026-10-02 |
| `https://github.com/boostorg/regex.git` | `boost-1.83.0` | `4cbcd3078e6ae10d05124379623a1bf03fcb9350` | remote tag/ref observed 2026-10-02 |
| `https://github.com/google/googletest` | `release-1.11.0` | `e2239ee6043f73722e7aa812a459f54a28552929` | remote tag/ref observed 2026-10-02 |

## Mutable/default refs and direct payloads observed on 2026-10-02

The following observations were captured on `htch-runtime` from the URLs and refs present in the frozen build scripts. They improve reproducibility of the research snapshot, but mutable endpoints remain historical-provenance blockers unless an actual build record proves the same content was consumed.

### Default/master Git refs

| Frozen input | Observed identity | Strength / limitation |
| --- | --- | --- |
| `desktop-app/lzma` default branch | `455a368eec2ac5d94de4de71bbf7a8a0fa0d72b7` | remote HEAD observed 2026-10-02; frozen script does not pin it |
| `FFmpeg/gas-preprocessor` default branch | `ac1836309c2e77023c228b7184485597286289d3` | remote HEAD observed 2026-10-02; frozen script does not pin it |
| Chromium `external/gyp@master` | `1615ec326858f8c2bd8f30b3a86ea71830409ce4` | master observed 2026-10-02; mutable branch |
| `desktop-app/gyp` short checkout `5e2425c47b` | `5e2425c47ba62aea63b20887ce545058b2dddec6` | short commit expanded to full immutable identity |
| `desktop-app/rnnoise` short checkout `d8ea2b0` | `d8ea2b0ec6a88f8a68d6904d75e33b4f09cb2987` | short commit expanded to full immutable identity |

### Downloaded payload observations

| Frozen URL/input | Bytes | SHA256 observed 2026-10-02 | Strength / limitation |
| --- | ---: | --- | --- |
| `https://sh.rustup.rs` | 29,915 | `7d0ea0f8eba7fa1ebfe998091cd7ec4501e33ec5ca6b884eb4d894d7da5170af` | mutable bootstrap endpoint; current-content observation only |
| `https://static.rust-lang.org/rustup/dist/x86_64-pc-windows-msvc/rustup-init.exe` | 12,721,664 | `6f4bef66261261fcb43131be8720bab817d403a09edec7455c371974b90bdb7e` | mutable bootstrap path; current-content observation only |
| `https://dist.nuget.org/win-x86-commandline/latest/nuget.exe` | 8,695,632 | `992d70cac5b06c38efec91806caba64cdcc07e6d963a0959dbbbaf264d33b800` | explicit `latest`; current-content observation only |
| `https://master.qt.io/official_releases/jom/jom_1_1_3.zip` | 1,213,852 | `128fdd846fe24f8594eed37d1d8929a0ea78df563537c0c1b1861a635013fff8` | versioned archive content observed |
| `https://ftp.gnu.org/pub/gnu/libiconv/libiconv-1.18.tar.gz` | 5,822,590 | `3b08f5f4f9b4eb82f151a7040bfd6fe6c6fb922efe4b1659c66ea933276965e8` | versioned archive content observed |
| `https://archives.boost.io/release/1.90.0/source/boost_1_90_0.tar.gz` | 210,975,925 | `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9` | versioned archive content observed |
| `https://github.com/msys2/msys2-installer/releases/download/2026-09-27/msys2-base-x86_64-20260927.sfx.exe` | 43,117,824 | `ad336cccfda47758b5e15cda993fbba421115cb0b126697daef1ee4dfe37209f` | versioned release asset content observed |

The frozen acquisition entrypoint files themselves were also re-fetched by exact Telegram commit and hashed: `prepare.py` = `98099d5a1c1ca530bf906ba4b29e54696640dd3a58ca155c0a3762fba6a10571`, CentOS `Dockerfile` = `2bd57a70b63c08afc7ccf7fcd48992a4ee1a75499bf531bb52e268b208282516`, and `snapcraft.yaml` = `5640e9f7b31b0464bcd0fa2dc66b44065d40d42ae8b893bb3b6a99ae8530bd4b`. These hashes bind the research inventory to the exact frozen source inputs.

## Explicitly unresolved mutable/bootstrap surfaces

- `prepare.py`: the observed NuGet `latest`, Chromium gyp `master`, `desktop-app/lzma` default branch, `FFmpeg/gas-preprocessor` default branch and rustup payloads now have dated identities above, but the frozen script still lacks immutable pins for those mutable inputs. Historical build provenance therefore remains unresolved unless an actual build record supplies the consumed commit/digest.
- `prepare.py` and the CentOS Dockerfile still use pip and platform package managers without a repository snapshot/lock that identifies the exact package artifacts consumed.
- `snap/snapcraft.yaml`: `source-commit` entries are immutable identities, while `source-tag`/`source-branch` and build/stage packages still require actual Snap build resolution or digest.
- `https://code.videolan.org/videolan/dav1d.git` tag `1.5.4` remains unresolved from the original remote in this evidence set; the GitHub mirror observation is not substituted for the source URL.
- License records are still required for every external acquisition that would enter a distributed Fabushi artifact. The hashes above are provenance identities, not license clearance.

## Closure rule

A row can leave the provenance blocker only when the distributed Fabushi build either (a) does not use that Telegram build dependency at all and records that non-adoption, or (b) records the actual immutable commit/content digest, license, patches/derivation and platform scope. Symbolic tags, branches, `latest`, package names and bootstrap endpoints never satisfy that rule by themselves.
