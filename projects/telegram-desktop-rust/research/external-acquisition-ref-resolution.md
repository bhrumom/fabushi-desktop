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

## Explicitly unresolved mutable/bootstrap surfaces

- `Telegram/build/prepare/prepare.py`: NuGet `/latest/nuget.exe`; Chromium gyp `@master`; default-branch clones including `desktop-app/lzma` and `FFmpeg/gas-preprocessor`; rustup installer/bootstrap URLs; pip/package-manager resolution; any short commit must be expanded/verified before release provenance.
- `Telegram/build/docker/centos_env/Dockerfile`: rustup bootstrap plus package-manager repository state. Its explicit full-SHA `git fetch` inputs are already immutable source identities; its tag/branch clones are only observationally resolved above.
- `snap/snapcraft.yaml`: `source-commit` entries are immutable identities, while `source-tag`/`source-branch` and build/stage packages require the actual Snap build resolution or digest.
- Direct archive URLs such as versioned Boost/jom/MSYS2/libiconv inputs still require content digests and license records; a version-bearing URL is not a content digest.

## Closure rule

A row can leave the provenance blocker only when the distributed Fabushi build either (a) does not use that Telegram build dependency at all and records that non-adoption, or (b) records the actual immutable commit/content digest, license, patches/derivation and platform scope. Symbolic tags, branches, `latest`, package names and bootstrap endpoints never satisfy that rule by themselves.
