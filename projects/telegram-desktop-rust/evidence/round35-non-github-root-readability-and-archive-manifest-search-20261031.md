# Round 35 — non-GitHub root readability and archive-manifest search

Date: 2026-10-31
Project: TDRP-001
Upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Starting live checkpoint: `34e028b990ab20b36d7e3cef2535249887f57f0b`

## Constraints

This round performed only public source-host reads/searches, one source-only Git reachability probe, and ledger/evidence edits. No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, installation, benchmark, fuzz, or runtime acceptance was executed. No archive bytes were downloaded or retried.

## dav1d

Fixed-U authority:
- repository: `https://code.videolan.org/videolan/dav1d`
- tag/ref: `1.5.4`
- resolved commit: `54706fc6bc0cdecab7e9593974a4039cc038fca7`

Direct original-host access:
- public web reader: blocked by robots for the exact dav1d repository path;
- one source-only Git probe: `git ls-remote https://code.videolan.org/videolan/dav1d.git refs/tags/1.5.4`;
- result: DNS resolution failed for `code.videolan.org`;
- retry policy: do not retry without a reader/network state change.

Existing exact official-mirror evidence remains:
- mirror: `https://github.com/videolan/dav1d`;
- exact commit: `54706fc6bc0cdecab7e9593974a4039cc038fca7`;
- root tree: `37c883895a0fe6cf3fdf3f975525fe8beefdd9c2`;
- `COPYING` blob: `8d4d6418c1ab4adf0a05bddfc11182216d5238a1`.

The mirror evidence is retained as content-addressed verification only; `code.videolan.org` is not marked direct-read verified.

## Boost 1.90.0 archive-manifest search

Readable evidence:
- official archive index: `https://archives.boost.io/release/1.90.0/source/`;
- official `boost_1_90_0.tar.gz` identity remains size 210,975,925 and SHA-256 `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9`;
- SourceForge release listing independently lists Boost 1.90.0 archive variants;
- Fossies exposes source-diff pages involving the Boost 1.90.0 release tree, including `boost_1_89_0.tar.bz2` vs `boost_1_90_0.tar.bz2`.

None of the readable pages exposes the complete raw archive member list. A diff report is not a manifest. `raw_member_listing` remains `unread`.

## GNU libiconv 1.18 archive-manifest search

Readable evidence:
- existing official identity: size 5,822,590; SHA-256 `3b08f5f4f9b4eb82f151a7040bfd6fe6c6fb922efe4b1659c66ea933276965e8`;
- independent GNU mirrors reconfirm the exact 5,822,590-byte archive;
- `https://tho-otto.m68k.eu/crossmint.php` explicitly advertises `Original sources: libiconv-1.18.tar.gz <file list>`;
- `https://fossies.org/diffs/libiconv/` exposes a `1.17_vs_1.18` source-code changes report.

The complete advertised file-list body/target is not exposed by the current reader, and the Fossies diff index is not a full manifest. Therefore `raw_member_listing` remains `unread`.

## Blockers unchanged

- xcb-util-m4 exact original `gitlab.freedesktop.org` tree/blob identity remains unverified/fail-closed.
- cppgir exact original `gitlab.com` tree/blob identity remains `git_tree_verified=false / fail-closed-original-host-unread`.
- copied/derived/adopted file-level legal mapping and release legal review remain open.
- broader fixed-U semantic/source production implementation closure remains open.

`baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.
