# Round 33 — non-GitHub original-host/toolchain and license closure

Date: 2026-10-31
Project: TDRP-001
Upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Starting checkpoint: `68129038c2550793b11775753be4abb52750e04d`

## Constraints obeyed

Only Git/GitHub source reads, public original-host Gitiles reads, official release metadata reads, and archive-content access attempts were used. No GitHub Actions/status query or trigger occurred. No build, typecheck, lint, schema validator, generator, test, package, deploy, install, benchmark, fuzz, or runtime acceptance was executed.

## Original-host Gitiles objects

- Breakpad Docker authority: `https://chromium.googlesource.com/breakpad/breakpad/+/9aebd3d8ef5a246deb2c929b5666aaba160ebce6`
  - commit: `9aebd3d8ef5a246deb2c929b5666aaba160ebce6`
  - tree: `1844a145a10b0463859aa3db4a3a41e4305108a9`
  - `.gitmodules` blob: `066cbb36c715e82b7658f074e9e458579f30e13f`
  - `LICENSE` blob: `d6435f048b9326dd3bb965e20c68f4f981a0cb78`
  - fixed-U Docker explicitly initializes LSS itself and does not invoke Breakpad DEPS/gclient or initialize `src/testing`.

- Linux Syscall Support Docker authority: `https://chromium.googlesource.com/linux-syscall-support/+/29164a80da4d41134950d76d55199ea33fbb9613`
  - commit: `29164a80da4d41134950d76d55199ea33fbb9613`
  - tree: `e1fffd0f09e0d2379e87296f2bef0bc5929b4b7b`
  - `LICENSE` blob: `b66a6b273d6b334c5197222573185b4e320190ea`

- Linux Syscall Support prepare.py short ref:
  - fixed-U short ref: `e1e7b0ad8e`
  - original-host full commit: `e1e7b0ad8ee99a875b272c8e33e308472e897660`
  - tree: `c2034bc1106379848e2c8c7b68f40d13b3c09097`
  - `LICENSE` blob: `58ab3fba9d4c3318ea545f451bb3ecbfdd5d8f6f`

- GYP mutable toolchain observation: `https://chromium.googlesource.com/external/gyp`
  - current `master` observation: `1615ec326858f8c2bd8f30b3a86ea71830409ce4`
  - tree: `ad3372d5851410812eb88c8f477fd716766982d5`
  - `LICENSE` blob: `ab6b011a109335d7bb8604d49be0d185f34d1744`
  - fixed U uses `@master`; this observation is not promoted to an immutable fixed-U pin.

## Archive content accounting

Boost 1.90.0 remains bound by official release metadata to:
- `boost_1_90_0.tar.gz`
- size: 210,975,925 bytes
- SHA-256: `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9`
- source commit: `1bed2b0712b2119f20d66c5053def9173c8462a5`

GNU libiconv 1.18 remains bound by official release metadata to:
- size: 5,822,590 bytes
- SHA-1: `3402aae249ec56e04114767c8f6bdaafc87a048c`
- SHA-256: `3b08f5f4f9b4eb82f151a7040bfd6fe6c6fb922efe4b1659c66ea933276965e8`
- detached signature size: 833 bytes
- signing fingerprint: `9001 B85A F9E1 B83D F1BD A942 F5BE 8B26 7C6A 406D`

The available text readers still do not expose either compressed archive body, and execution-environment direct archive fetches did not produce readable bytes. Official release/library descriptions are not treated as raw tar member manifests. Therefore both raw member lists remain `unread`; no file-level archive content-tree closure is fabricated.

## License/source closure

Original-host license objects added this round:
- Breakpad `LICENSE@d6435f048b9326dd3bb965e20c68f4f981a0cb78`: composite file containing top-level Google BSD-style terms and file-specific Unicode, libunwind/MIT-style, Apple Public Source License, and additional notices.
- LSS `LICENSE@b66a6b273d6b334c5197222573185b4e320190ea` at Docker pin.
- LSS `LICENSE@58ab3fba9d4c3318ea545f451bb3ecbfdd5d8f6f` at prepare pin.
- GYP `LICENSE@ab6b011a109335d7bb8604d49be0d185f34d1744` at the current mutable-master observation.

This is exact source/license provenance only. It does not decide which upstream files will be copied/derived/adopted into Fabushi distribution, does not establish compatibility, and does not constitute release legal acceptance.

## Remaining fail-closed items

- Boost 1.90.0 raw archive member enumeration.
- libiconv 1.18 raw archive member enumeration.
- xcb-util-m4 exact original `gitlab.freedesktop.org` tree/blob identity.
- cppgir exact original `gitlab.com` tree/blob identity.
- adopted/copied-derived file-level legal mapping and release review.
- broader fixed-U semantic/source implementation closure.

`baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.
