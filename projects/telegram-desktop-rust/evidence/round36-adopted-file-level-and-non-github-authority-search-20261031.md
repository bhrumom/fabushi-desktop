# Round 36 — adopted/copied-derived file-level provenance and remaining non-GitHub authority search

Date: 2026-10-31  
Repository: `bhrumom/fabushi-desktop`  
Checkpoint branch: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`  
Starting live HEAD: `d47ace6e412eb1e72f04b7957d9d8905746c585f`  
Fixed Telegram Desktop upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Execution constraints

This round used Git/GitHub tree/blob/source reads and public source-host reading only. It did not query or trigger GitHub Actions and did not run build, typecheck, lint, schema validation, generators, tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance.

The known unchanged blockers were intentionally not spun again: dav1d original-host robots/DNS, xcb-util-m4 original GitLab, cppgir original GitLab, and direct archive-byte download attempts.

## 1. Exact fixed-U root blob identity versus current checkpoint

Fixed U root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was read recursively without truncation (7,194 entries). The checkpoint tree `6098a14dc30158b463ad637871234f966c3eae29` was also read recursively without truncation (9,545 entries).

Comparing all blob SHAs produced one hit:

| Upstream fixed-U path | Current checkpoint path | Blob | Disposition |
| --- | --- | --- | --- |
| `Telegram/build/docker/centos_env/centos_env/__init__.py` | `source/mahayana/codex-rs/utils/cargo-bin/repo_root.marker` | `8b137891791fe96927ad78e64b0aad7bded08bdc` | both are exactly one newline; incidental/trivial identity, not substantive Telegram product-source copy |

Therefore this exact-blob comparison found **zero substantive byte-identical fixed-U Telegram root payload copies** in the current checkpoint. This is deliberately narrow: adapted, generated, transcribed, and source-informed files require separate mapping.

## 2. Current lockfile direct-name adoption check

The following current lock/package files were fetched by exact blob and inspected for the historical build-authority names Breakpad, Linux Syscall Support, libyuv, mini_chromium, Boost, libiconv, dav1d, libxcb/xcb-util variants, cppgir, FFmpeg, libavif, OpenH264, and libvpx:

- `desktop/package-lock.json@ee0f596ecf8aaf47cb28873316a7c6d148c70de0`
- `native/mahayana-messaging/Cargo.lock@876b3fe8a6e182d4d53c379dbfad7b80c967d748`
- `source/host/Cargo.lock@b872bdfe574c550cf9d74589015c75e106b25760`
- `source/host/app/Cargo.lock@7cd7f844f981ef4219ea37581a7c79445b1f68e8`
- `source/mahayana/codex-rs/Cargo.lock@7591a40df797cde1c4cd88e30bfe3cf248ab3f92`
- `source/mahayana/mahayana-rs/Cargo.lock@5c343a8b1bc0f88a68fd0055d372af55f4dfbf55`
- `source/mahayana/codex-rs/patches/pagable-0.4.1/Cargo.lock@96aaf5bb772c406d5bac790423eef1415d7d1a62`

No direct name match was found. This only narrows direct lockfile adoption for these named historical C/C++ inputs; it is not an exhaustive dependency or legal-acceptance result.

## 3. Current-Fabushi TDLib file-level mappings

Exact upstream authority: `tdlib/td@a17f87c4cff7b90b278d12b91ba0614383aaee82`, root tree `27af7989c2c65dec54eb7860fb6932d5e422c7b5`, non-truncated. Exact license: `LICENSE_1_0.txt@36b7cd93cdfbac762f5be4c6ce276df2ea6305c2` (Boost Software License 1.0).

### 3.1 Exact license copy

Current:
`source/mahayana/mahayana-rs/providers/telegram-protocol/LICENSE-TDLIB-BOOST-1.0.txt@36b7cd93cdfbac762f5be4c6ce276df2ea6305c2`

Upstream:
`tdlib/td@a17f87c4.../LICENSE_1_0.txt@36b7cd93cdfbac762f5be4c6ce276df2ea6305c2`

The Git blobs are identical. This is classified as an exact-byte copied license object.

### 3.2 Schema-derived constructor catalog

Current generated catalog:
`source/mahayana/mahayana-rs/providers/telegram-protocol/src/generated/schema_ids.rs@ef3e357fa81e2aac917d0083015b72c3cc023659`

Checked-in generator:
`source/mahayana/mahayana-rs/providers/telegram-protocol/src/bin/tl-schema-codegen.rs@1408cd1f1c17a97382aa2c4aa991e84f17f74475`

Exact upstream inputs at `a17f87c4...`:

- `td/generate/scheme/telegram_api.tl@c48d7914bc151a514b8c93c1b514bce48b28cb27` — generator input
- `td/generate/scheme/mtproto_api.tl@b2508b478b9e5dd51af1dde33adcbf87949b9a5c` — generator input
- `td/generate/scheme/td_api.tl@4dac6c8b10eeb76dd5d0280c9f374281990db877` — audit baseline only

The generator source reads and parses the first two inputs and emits declaration names, constructor IDs, kinds, and result types. It was **not executed** in Stage A.

### 3.3 Bootstrap data-center transcription

Current:
`source/mahayana/mahayana-rs/providers/telegram-protocol/src/dc.rs@31ca7076014ea12a76796a07f4f7a27b8eb93201`

Upstream:
`tdlib/td@a17f87c4.../td/telegram/net/ConnectionCreator.cpp@4bfe3bbdaaa82a20772a7bc8b437321578d45c7e`

The current Rust constants reproduce the pinned production/test bootstrap IPv4/IPv6 addresses and ports from the upstream function, then place them in Fabushi typed directory/routing logic. `providers/telegram-network/THIRD_PARTY_NOTICES.md@32f3913ecfabdf44f1c607167f67c65ed26fd02c` already names this relationship and the Boost-1.0 license copy.

### 3.4 SRP source-informed reimplementation

Current:
`source/mahayana/mahayana-rs/providers/telegram-protocol/src/srp.rs@f1368308b0ee1d75769dfb6ff559d7caa4e2d57e`

Upstream behavior source:
`tdlib/td@a17f87c4.../td/telegram/PasswordManager.cpp@8c0766d9e80bc7e5976993130a9a7e0f184e8b77`

The existing third-party notice describes the Rust SRP code as independently implemented from the public protocol and this Boost-licensed TDLib source. This round records that as source-informed/reimplemented, **not** as an asserted byte copy.

These four relationships now have exact current target blob(s), exact upstream commit/path/blob(s), transformation class, and license basis. This does not itself constitute release legal acceptance.

## 4. Remaining non-GitHub official-source authorities

The current reader still could not directly read the corresponding freedesktop GitLab commit/tree/license pages or freedesktop cgit pages for the three GitHub-mirror XCB inputs. A separate X.Org distribution release page provides independent exact-commit corroboration:

- XQuartz 2.8.6_beta3: <https://www.xquartz.org/releases/XQuartz-2.8.6_beta3.html>
- `libxcb-keysyms@ef5cb393d27511ba511c68a54f8ff7b9aab4a384`
- `libxcb-render-util@5ad9853d6ddcac394d42dd2d4e34436b5db9da39`
- `libxcb-cursor@4929f6051658ba5424b41703a1fb63f9db896065`

This corroborates commit identities only. It is **not** promoted to original-host tree/blob/license verification.

Per the task's no-spin rule, `xcb-util-m4@c617eee22ae5c285e79e81ec39ce96862fd3262f` and `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` were not retried. dav1d's robots/DNS state was also not retried.

## 5. Boost 1.90.0 / libiconv 1.18 member manifests

### Boost 1.90.0

The official archive index remains readable:
<https://archives.boost.io/release/1.90.0/source/>

It reconfirms the release archive identity/size already recorded. No complete archive-member listing was exposed by the readable official/search paths. Archive bytes were not retried. `raw_member_listing` remains `unread`.

### GNU libiconv 1.18

CrossMiNT's source page is currently readable at:
<https://tho-otto.m68k.eu/crossmint.php>

It explicitly lists `Original sources: libiconv-1.18.tar.gz <file list>`. Fossies also exposes archive-derived 1.18-vs-1.19 per-file diffs, for example:
<https://fossies.org/diffs/libiconv/1.18_vs_1.19/ChangeLog-diff.html>

The complete 1.18 file-list body/target was still not independently retrievable. This is only an archive-member-manifest lead, not a manifest. Archive bytes were not retried. `raw_member_listing` remains `unread`.

## 6. Fail-closed state after this round

Still open:

- broader current-shipping copied/derived/adopted file review and release legal acceptance beyond the four mapped TDLib relationships;
- complete Boost 1.90.0 and libiconv 1.18 member enumeration;
- original-host freedesktop/XCB tree/license closure where only mirror/corroboration exists;
- original-host xcb-util-m4 and cppgir blockers;
- broader fixed-U semantic responsibility and production implementation closure.

No `baseline_ready`, `accepted`, `verified`, or validation gate is advanced by this evidence.
