# Round 34 — Chromium original-host/object mapping and archive-manifest leads

Date: 2026-10-31
Project: TDRP-001
Upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Starting live checkpoint: `a56960a094b5e47721ccde292e86203374a29d45`

## Constraints obeyed

Only Git/GitHub source reads, public source-host text reads, and manual source/provenance review were used. No GitHub Actions/status query or trigger occurred. No build, typecheck, lint, schema validator, generator, test, package, deploy, install, benchmark, fuzz, or runtime acceptance was executed. Archive-byte downloads were not retried. The one source-only Git probe to `chromium.googlesource.com` encountered DNS resolution failure and was not repeated.

## Fixed-U reachability source

`Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b` establishes:

- stackwalk and Breakpad stages clone `https://chromium.googlesource.com/breakpad/breakpad` and checkout short `dfcb7b6799`;
- those stages apply the local Breakpad patch, explicitly clone googletest `release-1.11.0` into `src/testing`, and on mac explicitly clone LSS `e1e7b0ad8e`;
- no Breakpad DEPS/gclient invocation is present in those fixed-U stages;
- the Crashpad stage checks out `desktop-app/crashpad@3279fae3f02a0370ef54d4f9d264f52f337b0f1d`, initializes submodules, and explicitly updates `third_party/mini_chromium`.

## Breakpad `dfcb7b6799` object binding

- Original repository named by fixed U: `https://chromium.googlesource.com/breakpad/breakpad`.
- Full commit: `dfcb7b6799b7c1e2c8d65e857d8afede185471d8`.
- Independent Chromium src DEPS authority: `https://chromium.googlesource.com/chromium/src/+/931c5b62b5caa9d2765246d2f9d17b8c30157a4e/DEPS` binds `/breakpad/breakpad.git` to that full commit.
- Same-host same-content-addressed commit mirror: `https://chromium.googlesource.com/chromiumos/platform/google-breakpad/+/dfcb7b6799b7c1e2c8d65e857d8afede185471d8`.
- Observed tree on that mirror: `49b9ded11cf134b7385759bb599616f37f24eb8d`.
- Observed LICENSE blob: `a840b26471e053099ae9cce7507324562915293a`.
- The LICENSE is composite: a top BSD-style section plus file-specific Unicode, libunwind/MIT-style, Apple Public Source License, and additional legacy notices.
- Caveat: the current reader did not expose the direct `breakpad/breakpad` exact root/tree page. Therefore the ledger records original-host full-commit binding plus identical content-addressed host-mirror tree/license corroboration, not a direct original-repository tree/blob read.

## tg_owt recursive libyuv child

- Fixed child authority: `gitlab.com/chromiumsrc/libyuv@04821d1e7d60845525e8db55c7bcd41ef5be9406`.
- Official upstream: `https://chromium.googlesource.com/libyuv/libyuv/+/04821d1e7d60845525e8db55c7bcd41ef5be9406`.
- Directly read root tree: `46b29e13d76928a8423b346d085eda3eb215e4bb`.
- Directly read LICENSE blob: `c911747a6b53f063503992c3903d932f54cf6c57`.
- Directly read PATENTS blob: `64aa5c90d8fd97484876870a6722163468e9c0e7`.
- The LICENSE is BSD-style; PATENTS is preserved separately as an additional IP-rights notice object.
- This establishes official-upstream provenance for the exact content-addressed child. It does not claim that the GitLab mirror itself was read from its original GitLab host and does not constitute Fabushi release legal acceptance.

## Crashpad recursive mini_chromium child

- Build-reachable child: `https://chromium.googlesource.com/chromium/mini_chromium@14b219d5d503e305a6d853e64de201659cfcbe2d`.
- Original-host exact file references at that commit are independently visible in generated Fuchsia build documentation.
- Official GitHub mirror exact commit: `https://github.com/chromium/mini_chromium/commit/14b219d5d503e305a6d853e64de201659cfcbe2d`.
- Mirror Git tree: `655c7a93bdc2126a60bfec45055269986b271f26`.
- Mirror LICENSE blob: `36980134e0f658a15afe357511883b98d1bdae3b`.
- Caveat: the current reader did not expose the original Gitiles root/LICENSE page directly, so `direct_original_host_root_tree_license_read=false`; the tree/license identities are recorded as identical-Git-SHA official-mirror corroboration only.

## Archive manifest search

Boost 1.90.0:
- official release/package evidence remains bound to `boost_1_90_0.tar.gz`, size 210,975,925 and SHA-256 `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9`;
- official release pages did not provide a complete raw member manifest;
- archive bytes were not retried;
- raw member enumeration remains `unread`.

GNU libiconv 1.18:
- existing official identity remains size 5,822,590 and SHA-256 `3b08f5f4f9b4eb82f151a7040bfd6fe6c6fb922efe4b1659c66ea933276965e8`;
- `https://tho-otto.m68k.eu/crossmint.php` advertises a `<file list>` adjacent to exact `Original sources: libiconv-1.18.tar.gz`;
- Fossies exposes parsed source diffs for `libiconv-1.17.tar.gz` versus `libiconv-1.18.tar.gz`;
- the complete 1.18 file-list target/body was not independently readable through the current reader;
- this is a lead only, not a verified archive member manifest, so raw member enumeration remains `unread`.

## Remaining fail-closed items

- Boost 1.90.0 raw archive member enumeration.
- libiconv 1.18 raw archive member enumeration.
- xcb-util-m4 exact original `gitlab.freedesktop.org` tree/blob identity.
- cppgir exact original `gitlab.com` tree/blob identity.
- complete copied/derived/adopted file-level legal mapping and release review.
- broader fixed-U semantic/source production implementation closure.

`baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.
