# Telegram Frozen Source Closure Evidence

Status: P0 evidence; recursive gitlink, tracked generator/packaging roots, and external-acquisition entrypoints discovered; immutable external-resolution/resource-license/behavior closure not complete
Current Stage A authority root: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4` (tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`)  
Reconciled: 2026-10-31; Stage A manual Git/API evidence only, no CI executed

## Root tree

The GitHub recursive tree for the fixed U returned `truncated=false`: 7,194 entries total, including 6,859 blobs, 299 trees, and 36 direct gitlinks. Root `.gitmodules` is blob `2a71e861fab57dfd74d66e787867f2ede203a099`. These are exact fixed-U identity facts, not acceptance or full recursive-source-completeness evidence.

## Fixed-U changed-row semantic reread (2026-10-31)

Starting from checkpoint live HEAD `5c4d02ce42794b7ebc0878e4c7c4654765d706f0`, the 15 parity-ledger rows that still carried historical `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` provenance were reread against fixed `U=f985723b552b30fe1c17d377d57f44e61a15a8c4`. They span seven source blobs. All 15 cited behavior contracts remain supported by the fixed-U sources and are now eligible for provenance re-anchoring; this is provenance reconciliation only and does not change validation/acceptance state.

- `Telegram/SourceFiles/boxes/share_box.cpp`: historical blob `b7bf9f0d1f253d8e1a333ec1daa61d224fb188be` -> fixed-U blob `f4787dbb34cdb5fb2af78a69ae7537e91e06395a`. The GitHub compare reports 85 additions and 0 deletions: new includes plus a new `FastShareImage` function after the cited ShareBox/FastShareLink regions. The existing row-replacement, send-options, destination-policy, formatted-link entity shifting, typed child destination, topic eligibility, forward replay/idempotency mapping, forward privacy, and recipient-eligibility source regions were not removed or rewritten.
- `.agents/shared/codex-cli-leaves.md`: `659da44a8083c433c4e9b59ca28f64ba525bfbf8` -> `11c59b68df43e442e596c3052296fc51a044d46a`. The three-line delta expands model-pin scope for harness review/assessment leaves; availability, read-only sandbox, run/start/continue/recovery and fail-closed provider fallback remain intact.
- `.agents/shared/phase-effort.md`: `dd07a4c611922a29a6892332db98afe76cd58952` -> `8d7b385dbb526e87fc090774ee35191e1619bb08`. Six additive lines extend harness-task model selection; effort classification, evidence preservation, configured-model fallback and host-adapter controls remain intact.
- `.agents/shared/test-loop.md`: `83c4732bf6139f7c6557f18dd87bb35c3d68808d` -> `bb51685e417759939cadc1e2e74b1c0199fcb8f0`. Helper detail was moved to targeted `test/docs/` references and a Windows host-load evidence gate was added; the APPROVED/TEST_FLAW/IMPL_BUG/BLOCKED state machine, unique run directory, pass carry-forward, oracle evidence and one-focused-recovery convergence contract remain present.
- `.agents/skills/code-health/SKILL.md`: `6885938b3d186b55f5d69cb8b9b9a96bbca0e097` -> `101e69bc18d8f37c73d3b077f04d266bd939360f`. Harness lifecycle/report metrics were added; exact `origin/dev` authority, durable ledger/history, atlas, area survey, change check, proposals, audit and report contract remain intact.
- `.agents/skills/code-health/scripts/health.py`: `f08675ef7a03c016d85f91c60b8f29e99454abe9` -> `4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d`. Harness reporting was added to `command_scan`; the cited state, exact-source, detached-worktree and command orchestration functions remain present.
- `.agents/skills/code-health/scripts/health_test.py`: `2176aea672fd9ed16c01342939fbffb41490ae73` -> `6058fa64a5121c966021d8589b2010480aec8794`. Numeric-literal/CRLF/harness regression cases and interpreter portability were added; all cited integrated regression test classes remain present.

The additive fixed-U deltas above stay within the already mapped source-neutral governance/source responsibility boundaries; this reread does not mark any runtime validation gate passed. Coverage counters, `baseline_ready=false`, and `accepted=false` remain unchanged.

### cppgir retry

The parent evidence remains unchanged: `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095` declares `https://gitlab.com/mnauw/cppgir.git` and pins `external/glib/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`. This round retried both the original GitLab API v4 recursive repository-tree URL and the GitLab pinned-commit tree page; the execution network layer rejected both before any repository tree/blob data was returned. Therefore `git_tree_verified` remains false and the recursive cppgir chain remains fail-closed. No mirror, standalone `expected-lite` read, or historical claim is promoted as original-host reachability evidence.

## Direct gitlink recursion

The current `upstream.lock.json` records recursive GitHub-tree reads for all 36 direct gitlinks at fixed U. Three direct gitlinks expose one nested gitlink each; the remaining 33 expose none in those recorded scans. The two GitHub-hosted nested targets remain resolved; the GitLab-hosted cppgir target remains fail-closed because its original-host recursive tree/blob set is not available in the current execution environment.

| Parent | Nested path | Pinned commit | Result |
| --- | --- | --- | --- |
| `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095` | `external/glib/cppgir` | `47cf94f83b54cda59018135601e19d7fb0c77776` | parent `.gitmodules` blob `c92fbea3467f72e188cba4cbfe51c8c68a7e2e77` pins `https://gitlab.com/mnauw/cppgir.git`, and the parent tree pins this exact gitlink commit; original GitLab recursive tree/blob evidence remains unavailable here, so this nested chain is **not** current-closure complete |
| `PJK/libcbor@170bee2b…` | `doxygen-theme` | `46111c61a9f49b7a9886127e679d4317478fab1c` | recursive tree verified, `truncated=false`, no further gitlinks |
| `ericniebler/range-v3@5ba39112d2eafbb3c851a16a537f29b445022998` | `doc/gh-pages` | `2dae74bb693e42d850fb0adcc9045c5b71fbdeae` | recursive tree verified, `truncated=false`, no further gitlinks |

For fixed U, `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095` has `.gitmodules` blob `c92fbea3467f72e188cba4cbfe51c8c68a7e2e77`, which pins `external/glib/cppgir` to `https://gitlab.com/mnauw/cppgir.git`; its recursive GitHub tree pins the gitlink itself to `47cf94f83b54cda59018135601e19d7fb0c77776`. Historical evidence in this document previously asserted a fully paged GitLab tree and an `expected-lite` nested gitlink. The current execution environment cannot retrieve the original GitLab recursive tree/blob set, so those historical claims are not promoted into current fixed-U closure. A standalone GitHub read confirms `nonstd-lite/expected-lite@95b9cb015fa17baa749c2b396b335906e1596a9e` has a non-truncated 55-entry tree with no nested gitlinks, but that does not prove it is reachable from the pinned cppgir commit without the original cppgir tree/.gitmodules evidence.

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

## Source-directory citation coverage audit

A reverse audit against the frozen GitHub tree at `33261535a0e747f125e0ed25486f01e556330677` enumerated all 40 immediate `Telegram/SourceFiles` directories and 130 directory paths through relative depth two. Every one is cited by path or directory identity in the current research dossier set. This is useful negative evidence against an entirely unvisited top-level/second-level source area.

The result is **not** equivalent to behavior completeness: a directory citation can cover multiple state machines and edge cases, deeper files can expose additional responsibilities, and infrastructure/toolkit directories can still affect product lifecycle. It therefore narrows the remaining research question from “is there an untouched major SourceFiles area?” to “are all reachable behaviors inside the cited areas represented by a capability/aggregate row with source-to-behavior evidence?”.

## Build/resource closure checkpoint

The frozen root already proves these non-product-code inputs are reachable and therefore cannot be omitted from provenance closure:

| Area | Frozen evidence / responsibility | Current closure |
| --- | --- | --- |
| generators/codegen | `Telegram/SourceFiles/codegen`, scheme/language/number/MIDL/update-key/DBus/AppStream/model generation references in CMake | source-level input→output map recorded in `research/build-toolchain-resource-provenance.md`; generated outputs are not Fabushi shipping inputs by default |
| resources/assets | `Telegram/Resources` themes, language bundles, emoji, export templates, webview/picker HTML, sounds and updater/platform assets | resource roots identified; copied/derived asset and per-license review still incomplete |
| shaders | `Telegram/shaders` | 35 tracked sources identified; QRhi `qsb` output/QRC generation mapped; copied/derived shader license provenance remains open |
| build/packaging | `Telegram/build`, `Telegram/cmake`, `snap`, Windows/macOS/Linux packaging/update inputs | tracked platform roots and all explicit URL acquisition entrypoints mapped; mutable refs, package-manager resolution and dependency-by-dependency licenses remain open |
| native third parties | direct gitlinks including `tgcalls`, `lib_webrtc`, `lib_webview`, FIDO2 and media/storage libraries | all 36 direct gitlinks have recorded fixed-U recursive scans; two GitHub-hosted nested links are resolved, while `cppgir` remains original-host unresolved; non-gitlink external acquisitions remain separate |

No item in this table is an instruction to ship Telegram dependencies. It is source/provenance research required before declaring capability/source coverage complete.

## Build/resource closure still open

`Telegram/CMakeLists.txt` proves that the product links or generates through codegen, lib_storage, lib_ui, lib_webrtc, lib_webview, tgcalls, FIDO2, ffmpeg, Stripe, MTProto/scheme generators, language generators, update-key generation, platform MIDL on Windows, and Apple Swift runtime. An exact GitHub recursive-tree inventory for frozen root `33261535a0e747f125e0ed25486f01e556330677` (`truncated=false`) records 3,073 files under `Telegram/Resources`, 35 shader sources, eight `.obj` model inputs, 36 files under `Telegram/build`, 30 under `Telegram/cmake`, and the Snap packaging input. `research/telegram-resources-blob-inventory.tsv` binds every resource path to its frozen Git blob SHA and size. `research/build-toolchain-resource-provenance.md` binds the frozen `prepare.py`/Linux Docker blobs and enumerates their explicit external-acquisition entrypoints without upgrading symbolic or mutable inputs to immutable evidence.

A previous CMake-only scan correctly found no CMake `file(DOWNLOAD)`, `FetchContent`, or `ExternalProject_Add`, but that scope was too narrow. The frozen `Telegram/build/prepare/prepare.py` and `Telegram/build/docker/centos_env/Dockerfile` explicitly perform network acquisition using `git clone/fetch`, PowerShell `iwr`, `wget`, and `curl`. Build-time external acquisition therefore remains a real provenance blocker and must be enumerated rather than reported absent.

The current PR #27 changed-files audit finds no Telegram prepare/Docker/Snap invocation, no tracked Telegram resource/shader tree, no `.qsb`/`.binobj`/`.obj` payload, and no Telegram build/resource payload in the Phase 1 production delta. Those upstream build/resource inputs are therefore research/provenance inputs for this branch rather than shipping dependencies. This does not turn mutable historical acquisition into immutable evidence and does not waive source-informed legal review.

P0 must still close:

1. historical immutable build-input evidence only where needed to understand a source-derived behavior/build responsibility, while recording explicit non-adoption for Telegram-only dependencies not used by Fabushi;
2. dependency/license review for dependencies actually adopted by distributed Fabushi, plus source-informed GPL/third-party review;
3. copied/derived provenance for any upstream resource/shader/model/source content that a future or current production change actually copies or adapts; `research/telegram-resources-blob-inventory.tsv` now binds all 3,073 frozen resource blobs by path/SHA/size and `research/build-toolchain-resource-provenance.md` defines the mandatory copied/adapted release-review row, while the current FBCP branch still has no Telegram resource/shader/model payload;
4. mixed-license/per-file provenance needed to interpret researched upstream sources where the selected behavior depends on those files; the previously referenced `research/mixed-license-per-file-provenance.md` and `research/license-maps/*.tsv` artifacts are not present on authoritative PR #27 and must not be treated as completed evidence. Recreate source-backed mappings only for material copied/adapted or behaviorally relied-on inputs, and keep broader resource-level mixed-license cases open;
5. behavior-level source-to-responsibility dossiers until no reachable product capability remains unknown.

Until those close, source research coverage must not be reported as 100%.

## Fixed-U non-code provenance closure (round 30, 2026-10-31)

Stage A manual Git/API reads were performed against fixed `U=f985723b552b30fe1c17d377d57f44e61a15a8c4`; no build, generator, schema validator, test, package, deployment, installation check, or GitHub Actions run/status query was executed.

- **Build-time acquisitions:** the existing inventory references 28 authority source paths. Relative to historical `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`, 24 authority blobs are byte-identical. Four changed: `.github/workflows/canary-bot-api.yml` `7285f29e… -> 3ef5e601…`, Linux Dockerfile `f7216208… -> 3fea9e68…`, `prepare.py` `4063cc76… -> a9f3dd2d…`, and `snapcraft.yaml` `38f0eb36… -> a181d828…`. Their relevant acquisition forms remain the same classes. Fixed-U pins now include `desktop-app/patches@845a383f7abe8e44f92dfdcd566d603066ec2696`, `desktop-app/qt6_windows7_patches@9ce1ffe051161a22f7e78f715b042c773f868e59`, `desktop-app/tg_angle@9fd174a91e6155d1006c5f427366b23006f5d899`, and CI-only `docker/login-action@81c6f93d4bc991e510b342a00b419b3bb4efc5c6`; each GitHub object was independently read at that exact commit.
- **LFS/root external objects:** the fixed-U root has exactly two `.gitattributes` blobs (`d7f1c621…` and `f0af3029…`), neither declaring LFS. All 83 root blobs in the 90–220 byte standard-pointer window were read or identified as native non-UTF8 binary; zero contained the Git LFS pointer signature. This closes the standard **root-tree** pointer question only; it does not substitute for recursive gitlink/external-object evidence.
- **Resources/locales/assets:** `Telegram/Resources/` contains **3134** fixed-U blobs, all bound by path/blob SHA/size in `research/telegram-resources-blob-inventory.tsv`. Of these, `Resources/langs/` has 12 blobs, `Resources/qrc/` has 17, and `Resources/update/` has 4. The fixed root also has 37 shader sources (25 frag / 10 vert / 2 comp) and 5 tracked `.obj` model sources.
- **Generated inputs:** current CMake binds language generation to `Resources/langs/lang.strings`, scheme generation to `api.tl`/`mtproto.tl` plus `SourceFiles/codegen/scheme/codegen_scheme.py` and pinned `lib_tl`, Windows MIDL to two tracked IDLs, update-key generation to `root-public.pem`/`manifest.min.json`/`manifest.sig`, model baking to all tracked `.obj` sources plus `obj2binobj.py`, QRhi baking to all 37 tracked shaders, Linux DBus generation to three root XMLs plus `xdg-desktop-portal@23a76c3…/data/org.freedesktop.portal.Background.xml` blob `1b83fcd1…`, and AppStream changelog generation to tracked changelog/metainfo inputs. Generated outputs were **not executed** in Stage A.
- **Packaging/updater/tests/tooling/docs/licenses:** fixed U has 36 tracked blobs under `Telegram/build/`, 33 under `Telegram/cmake/`, 1 under `snap/`, 20 workflow blobs, 60 agent/tooling blobs, and 228 root test/test-fixture blobs (167 under `SourceFiles/test/`, 55 under `SourceFiles/gram/tests/`, 6 under `SourceFiles/tests/`). Root legal authorities are `LEGAL@09c9e383…`, `LICENSE@d70f4d6a…` and `README.md@15a6194f…`; README identifies GPLv3 with OpenSSL exception plus third-party licenses. This records source identity, not a completed Fabushi adopted-dependency/copied-content legal review.

Durable machine-readable details are in `inventory/fixed-u-non-code-inputs.json`. These facts advance fixed-U provenance closure but do **not** set `baseline_ready`, `verified`, or `accepted`: original-host cppgir remains `git_tree_verified=false`, recursive external-object/toolchain and copied/derived/adopted-dependency license review remain open, and full behavior/source semantic closure is not complete.

## Fixed-U authority and GitHub-readable recursive provenance closure (round 31, 2026-10-31)

Stage A manual Git/API/source reads only; no CI/Actions status query, build, typecheck, lint, schema validator, generator, test, package, deployment, or installation execution occurred.

- **Root mutable authorities:** `inventory/root-mutable-authorities.json` is re-anchored to fixed `U=f985723b552b30fe1c17d377d57f44e61a15a8c4`. The fixed-U source reread collapses the historical split Ada records into one `v3.2.9` authority, leaving exactly **48 authorities / 67 occurrences**. Every occurrence now records the fixed-U source commit and source blob. GitHub-readable commit/root-tree objects were independently read for 47 unique authority identities; symbolic `HEAD`/branch resolutions remain explicitly non-U-pinned. Fixed-U changes include Ada `v3.2.9`, libxcb `1.16.1`, libXtst `1.2.5`, libXv `1.0.13`, libXrandr `1.5.4`, libXrender `0.9.12`, OpenSSL `3.5.9`, Docker xz `5.8.4`, plus exact prepare.py checkouts for OpenAL `00cb9b649c1a6ad1802d88ee4458979164529238` and xz `05af863c3166cddeb4ab935829b14e995c8346b4`.
- **Root tool authorities:** `inventory/root-tool-authorities.json` is re-anchored to fixed U and current source blobs. The candidate total is **16** (14 tool-bootstrap + 2 versioned archive lines), down from 17 because current Snap no longer directly downloads `sh.rustup.rs`; it uses the Rust plugin/channel. Dynamic package/tool URLs remain toolchain inputs rather than immutable product-source authority.
- **Recursive GitHub build children:** exact commit/root-tree closure was read for `abseil/abseil-cpp@d7aaad83...`, `google/crc32c@21fc8ef3...`, `cisco/libsrtp@a566a9cf...` and crc32c's three pinned children (`google/benchmark@e991355c...`, `google/glog@56523194...`, `google/googletest@3b49be07...`). Every listed tree is non-truncated. CRC32C's three children each have zero further gitlinks.
- **Generated inputs:** `desktop-app/codegen@dbd1d531...` -> tree `6b59722e...`, `desktop-app/lib_tl@29746d31...` -> tree `7e5277e0...`, and `flatpak/xdg-desktop-portal@23a76c39...` -> tree `f89cf6df...` were independently read with non-truncated recursive trees and zero gitlinks. Consumed generator/DBus blobs were also read at exact blob identity.
- **License provenance:** fixed-U root `LEGAL`, `LICENSE`, `README.md`, plus the GitHub-readable child LICENSE/COPYING blobs recorded in `inventory/fixed-u-non-code-inputs.json`, were independently blob-read. This is exact source/license identity evidence only; copied/derived-content and actually adopted distributed-dependency legal review remains open.
- **Fail-closed blocker:** `gitlab.com/mnauw/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` has no new original-host tree/blob evidence. `git_tree_verified=false`, `fail-closed-original-host-unread`, `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.

## Fixed-U GitHub build-authority recursive closure (round 32, 2026-10-31)

Stage A remained read-only with respect to execution: no GitHub Actions/status query, CI trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, or installation command was run.

- **Root mutable authorities:** all 48 rows (47 unique GitHub commit/root-tree objects because dav1d is represented twice) now have exact commit -> recorded root tree -> non-truncated recursive-tree reads. `recursive-scan-pending` is zero in `inventory/root-mutable-authorities.json`. Live `HEAD`/branch authorities remain explicitly non-U-pinned.
- **Zero-gitlink roots:** libavif v1.3.0/v1.4.2, FFmpeg n8.1.3/n6.1.6, Boost.Regex 1.83.0, Brotli 1.2.0 and the other ordinary GitHub roots were read through their full non-truncated recursive trees; root license/COPYING identities were blob-read where present.
- **libjxl v0.12.0:** root tree `d56a2e4387ca5bf74ec08dbe682a0cfbedd45818`, `.gitmodules@e64c474a…`, and all ten recursively fetched child commits were independently read. Every child tree is non-truncated and has zero further gitlinks. The historical `glennrp/libpng` submodule URL redirects to `pnggroup/libpng`, where the exact pinned commit `872555f4…` is readable.
- **OpenSSL 3.5.9:** root tree `475b3b8e…` is non-truncated; `.gitmodules@b4c756a6…` identifies 11 direct gitlinks. Fixed-U TDesktop OpenSSL stages do not initialize submodules, so those children are explicitly non-build-reachable rather than recursively fetched.
- **Qt:** Qt 6.11.2 and Qt 5.15.19 superproject trees are non-truncated and their `.gitmodules` blobs were read. Fixed-U TDesktop selects six Qt6 modules and three Qt5 modules with recursive submodule update. All selected child trees are non-truncated; the sole nested gitlink reached by that selected chain, Qt6 `qtdeclarative -> qtdeclarative-testsuites@test262`, is also closed and has zero further gitlinks.
- **xcb-util recursive m4:** libxcb-image, libxcb-util and libxcb-wm each recursively fetch `m4@c617eee…` from `gitlab.freedesktop.org/xorg/util/xcb-util-m4.git`. The exact commit/tree `184bbcb9…` is independently visible in multiple GitHub mirrors and has zero gitlinks, but the original GitLab tree was not readable through the current source path; it remains recorded as original-host-unverified rather than promoted to exact original-host closure.
- **Boost 1.90.0 archive:** the official Boost release index reports `boost_1_90_0.tar.gz` size 210,975,925 bytes; the official sidecar binds it to source commit `1bed2b0712b2119f20d66c5053def9173c8462a5` and SHA-256 `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9`. That GitHub superproject root tree is `b4d30083…` with 171 gitlinks; `libs/regex` is `f439e22a…` and its recursive tree has zero gitlinks. The compressed archive body itself remains unread by the current web connector, and direct source-download attempts in the execution environment failed, so raw archive-member enumeration remains fail-closed.
- **libiconv 1.18 archive:** GNU's release announcement confirms SHA-1 `3402aae249ec56e04114767c8f6bdaafc87a048c`, SHA-256 `3b08f5f4f9b4eb82f151a7040bfd6fe6c6fb922efe4b1659c66ea933276965e8`, detached signature and signing key fingerprint. GNU/mirror indexes report the archive as 5,822,590 bytes and the signature as 833 bytes. The compressed archive body remains unread in the current source-read path, so raw member enumeration remains open.
- **License provenance:** `inventory/fixed-u-non-code-inputs.json` now records 66 exact GitHub-readable license/COPYING blob reads across root authorities and already-closed recursive children. This is source identity evidence only; it does not complete Fabushi adopted-dependency or copied/derived release legal review.
- **Fail-closed state:** cppgir remains `git_tree_verified=false`, `fail-closed-original-host-unread`; `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` are unchanged.

The remaining source-provenance blockers are therefore narrower but still real: raw Boost/libiconv archive member lists, xcb-util-m4 original-host tree identity, cppgir original GitLab tree/blob, adopted/copied-derived legal review, and the broader semantic/implementation closure required by TDRP.

## Fixed-U non-GitHub original-host/toolchain and license closure (round 33, 2026-10-31)

Stage A remained execution-free: no GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, or installation execution was performed.

- **Chromium original-host source objects:** fixed-U Docker Breakpad `9aebd3d8ef5a246deb2c929b5666aaba160ebce6` is now independently read on `chromium.googlesource.com/breakpad/breakpad` at tree `1844a145a10b0463859aa3db4a3a41e4305108a9`, with `.gitmodules@066cbb36c715e82b7658f074e9e458579f30e13f` and `LICENSE@d6435f048b9326dd3bb965e20c68f4f981a0cb78`. The fixed-U Docker stage manually fetches LSS and does not run Breakpad DEPS/gclient or initialize `src/testing`, so upstream optional dependency metadata is not promoted into the build-reachable acquisition set.
- **Linux Syscall Support original-host closure:** Docker's exact LSS `29164a80da4d41134950d76d55199ea33fbb9613` resolves to tree `e1fffd0f09e0d2379e87296f2bef0bc5929b4b7b` with `LICENSE@b66a6b273d6b334c5197222573185b4e320190ea`. The prepare.py short pin `e1e7b0ad8e` is now independently resolved on the original host to `e1e7b0ad8ee99a875b272c8e33e308472e897660`, tree `c2034bc1106379848e2c8c7b68f40d13b3c09097`, `LICENSE@58ab3fba9d4c3318ea545f451bb3ecbfdd5d8f6f`.
- **Mutable GYP toolchain authority:** current Chromium original-host `external/gyp` master still points to `1615ec326858f8c2bd8f30b3a86ea71830409ce4`, tree `ad3372d5851410812eb88c8f477fd716766982d5`, with `LICENSE@ab6b011a109335d7bb8604d49be0d185f34d1744`. Fixed U names `@master`, so this is recorded only as a current original-host observation; it is **not** retroactively treated as an immutable U pin.
- **Boost/libiconv archive accounting:** official Boost 1.90.0 release metadata continues to bind `boost_1_90_0.tar.gz` to SHA-256 `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9` and source commit `1bed2b0712b2119f20d66c5053def9173c8462a5`; GNU libiconv 1.18 remains bound to SHA-256 `3b08f5f4f9b4eb82f151a7040bfd6fe6c6fb922efe4b1659c66ea933276965e8` plus its published signature metadata. Official release/library descriptions are not file-member manifests. The compressed bodies remain unread in the available source-read paths, so raw member enumeration stays fail-closed for both archives.
- **Adopted/copied-derived license source closure:** four non-GitHub original-host license blobs are now recorded. Breakpad's exact LICENSE is composite and contains top-level BSD-style terms plus file-specific Unicode, libunwind/MIT-style, Apple Public Source License, and further notices. LSS and GYP license identities are also exact-read. These facts narrow provenance, but file-by-file copied/derived/adopted mapping and Fabushi release legal acceptance remain open; no compatibility or release-acceptance verdict is inferred from source identity alone.
- **Unchanged fail-closed blockers:** `xcb-util-m4@c617eee22ae5c285e79e81ec39ce96862fd3262f` remains original-GitLab-unverified despite mirror object evidence; `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` remains `git_tree_verified=false / fail-closed-original-host-unread`. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.

Raw Boost/libiconv file-member closure, the two original GitLab blockers, complete adopted/copied-derived legal mapping, and the broader fixed-U semantic/production implementation closure are still outstanding.

## Fixed-U Chromium original-host/object mapping and archive-manifest leads (round 34, 2026-10-31)

Stage A remained execution-free: no GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, installation, benchmark, fuzz, or runtime acceptance was performed.

- **Prepare Breakpad exact pin:** fixed-U `Telegram/build/prepare/prepare.py@a9f3dd2d9e4020df958ab5af942376177d66c78b` directly clones `chromium.googlesource.com/breakpad/breakpad` and checks out short `dfcb7b6799` in both stackwalk and Breakpad stages. Independent Chromium source metadata binds the full original-host object to `dfcb7b6799b7c1e2c8d65e857d8afede185471d8`. The identical content-addressed commit is readable on Chromium's `chromiumos/platform/google-breakpad` host mirror at tree `49b9ded11cf134b7385759bb599616f37f24eb8d`, with `LICENSE@a840b26471e053099ae9cce7507324562915293a`. The direct `breakpad/breakpad` root/tree page was not exposed by the current reader, so this is recorded as original-host full-commit binding plus same-object host-mirror corroboration, not a direct original-repository tree/blob read. These prepare stages explicitly fetch googletest `release-1.11.0` and LSS `e1e7b0ad8e`; they do not invoke Breakpad DEPS/gclient.
- **tg_owt → libyuv official upstream:** the fixed recursive child `gitlab.com/chromiumsrc/libyuv@04821d1e7d60845525e8db55c7bcd41ef5be9406` maps to official Chromium Gitiles `libyuv/libyuv` at the same exact commit. The original-host root is directly read at tree `46b29e13d76928a8423b346d085eda3eb215e4bb`, with `LICENSE@c911747a6b53f063503992c3903d932f54cf6c57` and `PATENTS@64aa5c90d8fd97484876870a6722163468e9c0e7`. This closes the independently readable official-upstream object/license/notice mapping for that content-addressed child while preserving the distinction that the GitLab mirror host itself was not the source of this read.
- **Crashpad → mini_chromium:** fixed-U prepare.py checks out `desktop-app/crashpad@3279fae3f02a0370ef54d4f9d264f52f337b0f1d`, initializes submodules, and explicitly updates `third_party/mini_chromium`. Original-host exact file references at `14b219d5d503e305a6d853e64de201659cfcbe2d` are independently visible. The official `chromium/mini_chromium` GitHub mirror exposes the identical Git commit at tree `655c7a93bdc2126a60bfec45055269986b271f26` and `LICENSE@36980134e0f658a15afe357511883b98d1bdae3b`. Because the current reader did not expose the original Gitiles root/LICENSE page directly, the ledger keeps `direct_original_host_root_tree_license_read=false`.
- **Archive member manifests:** no complete Boost 1.90.0 raw member manifest was found; official Boost pages remain release/package-level evidence and the archive bytes were not retried. For GNU libiconv 1.18, an independent cross-build source page advertises a file list for the exact original `libiconv-1.18.tar.gz`, and Fossies exposes parsed 1.17→1.18 file diffs, but the complete 1.18 list body/target was not readable through the current reader. That is recorded only as a lead; `raw_member_listing` remains `unread`.
- **Legal/source closure:** direct libyuv `LICENSE` and `PATENTS` identities are now explicit, but copied/derived/adopted file-level mapping and release legal review remain open. No source-identity read is treated as release acceptance.
- **Unchanged fail-closed blockers:** `xcb-util-m4@c617eee22ae5c285e79e81ec39ce96862fd3262f` remains original-GitLab-unverified; `cppgir@47cf94f83b54cda59018135601e19d7fb0c77776` remains `git_tree_verified=false / fail-closed-original-host-unread`. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.

## Non-GitHub root readability and archive-manifest search (round 35, 2026-10-31)

Stage A remained zero-execution for validation: no GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance was performed.

- **dav1d original host:** fixed U resolves `code.videolan.org/videolan/dav1d` tag `1.5.4` to `54706fc6bc0cdecab7e9593974a4039cc038fca7`. The public web reader is blocked by robots for the exact repository path, and a single source-only Git probe failed DNS resolution for `code.videolan.org`. No retry was performed. The existing official VideoLAN GitHub mirror verification remains exact at tree `37c883895a0fe6cf3fdf3f975525fe8beefdd9c2` with `COPYING@8d4d6418c1ab4adf0a05bddfc11182216d5238a1`; this is explicitly not relabeled as a direct original-host read.
- **Boost 1.90.0:** the official archive index was reconfirmed. SourceForge also lists the 1.90.0 release archives, and Fossies exposes source-diff pages parsed from the Boost 1.90.0 release tree. None of these readable paths exposed a complete raw member manifest. Archive bytes were not retried; `raw_member_listing=unread`.
- **GNU libiconv 1.18:** the cross-build source page explicitly advertises `Original sources: libiconv-1.18.tar.gz <file list>`, and Fossies exposes a `1.17_vs_1.18` source-diff index. The complete advertised file-list body/target is still not readable, so it is not accepted as a manifest. Independent GNU mirrors reconfirm the exact 5,822,590-byte archive identity. Archive bytes were not retried; `raw_member_listing=unread`.
- **Unchanged fail-closed blockers:** xcb-util-m4 original GitLab remains unread/unverified; cppgir remains `git_tree_verified=false / fail-closed-original-host-unread`. No blocker was promoted from mirror or search metadata alone.

## Current Fabushi adopted/copied-derived file-level mapping and remaining authority search (round 36, 2026-10-31)

Stage A remained execution-free for validation: no GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance was performed.

- **Exact fixed-U root byte-copy check:** the complete non-truncated fixed-U Telegram tree (`91f2b083...`, 7,194 entries) was compared by Git blob identity with the complete checkpoint tree (`6098a14dc30158b463ad637871234f966c3eae29`, 9,545 entries). There is exactly one equal blob: the one-byte newline-only `Telegram/build/docker/centos_env/centos_env/__init__.py` and `source/mahayana/codex-rs/utils/cargo-bin/repo_root.marker` share `8b137891...`. It is classified as incidental/trivial identity, not substantive copied Telegram product source. This exact-blob test does not decide adapted/generated/transcribed/source-informed files.
- **Current lockfile adoption check for historical build authorities:** seven current Cargo/npm lockfiles were manually source-read for Breakpad/LSS/libyuv/mini_chromium/Boost/libiconv/dav1d/libxcb/xcb-util/cppgir/FFmpeg/libavif/OpenH264/libvpx names; no direct name match was found. This narrows direct package adoption only and is not an exhaustive release/legal verdict.
- **TDLib exact copied license:** current Fabushi `providers/telegram-protocol/LICENSE-TDLIB-BOOST-1.0.txt@36b7cd93...` is byte-identical to `tdlib/td@a17f87c4.../LICENSE_1_0.txt@36b7cd93...`. The upstream commit has root tree `27af7989...`, read non-truncated.
- **TDLib schema-derived catalog:** `providers/telegram-protocol/src/generated/schema_ids.rs@ef3e357f...` declares source commit `a17f87c4...`; its checked-in generator `tl-schema-codegen.rs@1408cd1f...` consumes exact `telegram_api.tl@c48d7914...` and `mtproto_api.tl@b2508b47...`. `td_api.tl@4dac6c8b...` is an audit baseline only. The generator was not executed in Stage A.
- **TDLib transcribed/source-informed files:** `providers/telegram-protocol/src/dc.rs@31ca7076...` contains bootstrap DC addresses/ports transcribed from `td/telegram/net/ConnectionCreator.cpp@4bfe3bbd...`; `src/srp.rs@f1368308...` is recorded as a source-informed independent Rust reimplementation using the public protocol plus `td/telegram/PasswordManager.cpp@8c0766d9...`. Both are bound to TDLib commit `a17f87c4...` and the exact Boost-1.0 license blob; neither is promoted to validated/release-accepted state.
- **Remaining non-GitHub authorities:** XQuartz 2.8.6_beta3 independently lists the exact X.Org commits used for `libxcb-keysyms@ef5cb393...`, `libxcb-render-util@5ad9853d...`, and `libxcb-cursor@4929f605...`. Direct freedesktop GitLab/cgit tree/license reads were not available, so this is corroboration only and original-host verification remains open. `xcb-util-m4` and `cppgir` were not retried.
- **Archive manifests remain fail-closed:** Boost 1.90.0 official metadata still binds the 210,975,925-byte tarball and known SHA-256, but no complete member list was exposed and archive bytes were not retried. CrossMiNT advertises an exact `libiconv-1.18.tar.gz <file list>` and Fossies exposes archive-derived 1.18→1.19 per-file diffs, but a complete 1.18 member-list body remains unread. Neither lead is promoted to a manifest.

Concrete file-level mapping is now closed for the four listed current-Fabushi TDLib relationships, but broader current shipping copied/derived/adopted review, release legal acceptance, Boost/libiconv complete member enumeration, original-host xcb-util-m4/cppgir closure, and the wider fixed-U semantic/production implementation remain open. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.

## Current shipping Telegram source provenance and archive-manifest search (round 37, 2026-10-31)

Stage A remained validation-execution-free: no GitHub Actions/CI query or trigger, build, typecheck, lint, schema validator, generator, tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance was performed.

- **Static production-owner wiring:** `mahayana-host/Cargo.toml@cb886151...` depends on `mahayana-telegram`; `mahayana-host/src/lib.rs@fd28ba53...` constructs `TelegramConversationProvider::from_client_id` and registers it through the shared `ConversationProvider` builder when a Telegram client is supplied. `mahayana-telegram/src/lib.rs@cec9d354...` consumes the Telegram core/runtime crates. This is static wiring evidence only, not runtime or packaged acceptance.
- **TDLib exact source/license authority:** current Telegram protocol provenance is anchored to `tdlib/td@a17f87c4...`, root tree `27af7989...`, non-truncated. `LICENSE_1_0.txt@36b7cd93...` is the exact BSL-1.0 authority and the current Fabushi license copy is byte-identical. No separate `NOTICE` or `PATENTS` path was found in the exact pinned TDLib tree; this is source-tree evidence, not legal acceptance.
- **New exact file mappings:** `schema.rs@6e2de8f0...` is bound to the three exact TDLib schema blobs; `api.rs@a5a226ba...` and `updates.rs@05cef74a...` are bound to exact schema declaration IDs/field order; `handshake.rs@e430fcc1...` is bound to exact MTProto schema declarations, `DhCache.cpp@eb9a4f3e...` safe-prime data, `PublicRsaKeySharedMain.cpp@abdbff59...` main/test RSA keys, and TDLib handshake implementation references; `crypto.rs@032b6fcf...` is bound to `KDF.cpp@36daac0b...` and `Transport.cpp@d23173e1...`; `session.rs@bc5dfce3...` maps message-id window behavior to `AuthData.cpp@56c59ae3...`; `transport.rs@f4ce1cd0...` maps intermediate/padded framing to `TcpTransport.cpp@55937def...`. These are generated/transcribed/source-informed provenance classifications, not claims of whole-file byte copying.
- **Legacy pin metadata:** `providers/telegram-core/upstream-pins.json@c3743170...` is retained as 2026-07-10 research metadata and is not treated as fixed-U release authority because its Telegram Desktop pin differs from current U.
- **Archive manifests:** Boost 1.90.0 and libiconv 1.18 remain `raw_member_listing=unread`. Readable release/source pages reconfirm archive identity and CrossMiNT still advertises an exact libiconv `<file list>`, but no complete member-list body was retrieved and archive bytes were not retried.
- **No-spin blockers:** dav1d original-host robots/DNS, xcb-util-m4 original GitLab, and cppgir original GitLab were not retried.

Durable details are in `inventory/fixed-u-non-code-inputs.json`, `inventory/build-time-acquisitions.json`, the updated Telegram third-party notices, and `evidence/round37-shipping-telegram-source-provenance-and-archive-manifests-20261031.md`. Broader shipping-file provenance, release legal acceptance, raw Boost/libiconv member closure, original-host GitLab blockers and fixed-U semantic/implementation closure remain open; `baseline_ready=false`, `accepted=false`, and `validation_status=not-run`.


## Fixed-U wire corroboration and conditional owner injection gap (round 38, 2026-10-31)

Stage A remained execution-free for validation: no GitHub Actions/CI status query or trigger, build, typecheck, lint, schema validator, generator, tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance was performed.

- **Fixed-U dual anchoring:** current protocol `api.rs@a5a226ba...`, `updates.rs@05cef74a...`, `tl.rs@89256ab3...` and `handshake.rs@e430fcc1...` are now cross-anchored not only to the pinned BSL-1.0 TDLib schema/source objects recorded in round 37, but also to fixed `telegramdesktop/tdesktop@f985723b...` schema objects `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df...` and `mtproto.tl@76225965...` as migration-provenance corroboration. Fixed-U legal authorities remain `LEGAL@09c9e383...` and `LICENSE@d70f4d6a...`; those GPL-side objects are not substituted for the TDLib BSL license basis of mapped implementation inputs.
- **Layer/header exact source:** `api.rs` layer 227 is independently anchored to TDLib `td/telegram/Version.h@a8774804...`; invoke-with-layer/init-connection framing is anchored to `td/telegram/net/MtprotoHeader.cpp@a7e7bf97...`.
- **TL core constants:** `tl.rs` uses the schema constants `vector#1cb5c415`, `boolTrue#997275b5`, and `boolFalse#bc799737`; these are mapped to the exact TDLib/fixed-U schema objects while the reader/writer remains Fabushi-owned Rust code.
- **Expanded static owner chain:** `mahayana-conversation/src/lib.rs@bef082ee...` routes `telegram:` IDs to provider key `telegram`; `mahayana-ffi/src/lib.rs@bdff6af5...` force-links Telegram runtime ABI symbols; and `providers/telegram-runtime/src/runtime.rs@787d28c0...` directly composes the mapped protocol/network/storage operations. Combined with the round-37 host/provider registration evidence, this closes more of the static source chain.
- **Injection gap remains explicit:** the inspected desktop app-host construction path `mahayana-app-host/src/lib.rs@3b873243...` creates `HostCreateConfig` with default Telegram fields. The actual platform-login/caller source that supplies `telegram_client_id` has not yet been traced, so unconditional shipping activation is not claimed.
- **Archive member manifests:** a readable Fossies search result exposes archive-derived Boost 1.90.0 diff data but not a complete 1.90.0 member manifest. CrossMiNT still advertises the exact libiconv 1.18 file-list link but the complete list body remains unread. Both remain fail-closed; archive bytes were not retried.
- **No-spin blockers:** dav1d original-host robots/DNS and xcb-util-m4/cppgir original GitLab blockers were not retried.

`baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged. Broader provider semantic/source-informed review, the Telegram client-id injection trace, release legal acceptance, complete archive members, original-host blockers, and full fixed-U implementation closure remain open.


## Canonical Telegram provider activation gap (round 39, 2026-10-31)

Stage A source review found a concrete production-wiring gap without running validation. `mahayana-host/src/lib.rs@fd28ba53...` registers `TelegramConversationProvider::from_client_id` only when `HostCreateConfig.telegram_client_id` is supplied and documents that client as created by the platform login flow. The provider/runtime already exposes real ephemeral/persistent client creation plus transport/auth/update operations, so the missing piece is canonical account/platform ownership and handoff.

The current production app-host construction `mahayana-app-host/src/lib.rs@3b873243...` finishes with `..HostCreateConfig::default()` and leaves Telegram IDs unset. Exact reads of the desktop/mobile/unified wrappers, CLI creation path and generic production FeatureHost path also found no Telegram client-id injection. Therefore the current evidence proves conditional provider code, not canonical shipping activation.

No default ephemeral client, environment-variable client ID, mock login or ABI-only shortcut was added. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.


## Remaining Telegram provider source disposition (round 40, 2026-10-31)

Stage A remained execution-free for validation. Three residual provider files were source-read at exact checkpoint `1caf8ed26becbdad129b41ed70cd353efd58c8d6`:

- `providers/telegram-protocol/src/request.rs@d59aae57...` is a Fabushi-owned request sequencer/JSON-envelope helper. No exact upstream byte-copy, generated, or transcribed relationship was established from the reviewed source.
- `providers/telegram-storage/src/lib.rs@13378d68...` is the Fabushi encrypted-storage module boundary and explicitly assigns key lifecycle to platform credential stores. No exact Telegram/TDLib copied/generated/transcribed relationship was established.
- `providers/telegram-network/src/bin/telegram-auth-key-probe.rs@2854784c...` is tooling rather than a production owner. Its literal `get_future_salts#b921bd04` request and expected `future_salts#ae500895` result identifiers are both exactly present in TDLib `td/generate/scheme/mtproto_api.tl@b2508b47...`; these two schema identifiers are now source/license-mapped. The surrounding probe remains Fabushi-owned diagnostic code.

No relationship was inferred merely from Telegram naming, domain concepts, SQLite, HMAC/SHA-256, or local JSON conventions. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.


## Canonical account-bound embedded Telegram client handoff (2026-11-01)

This Stage-A slice closes the previously recorded **production wiring implementation gap**, not the full source inventory or acceptance gate. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Exact source objects manually read for this responsibility:
- `Telegram/SourceFiles/main/main_account.cpp@2721c7ac12effe454d398610d26d638fadaa1f68`: `Main::Account` owns local account storage, restores serialized authorization/session identity, starts the MTP instance, persists changing keys/config, and tears down/reset local session state on logout.
- `Telegram/SourceFiles/storage/storage_account.cpp@afe6f61af833fd32ba02a3bd8a48f6801cb583d5`: account-local encrypted storage supplies the durable local-state side of that lifecycle.

Fabushi implementation commit `e8a8b7726679c06862205ef21fdfbfc0c5aca4ed` has the following exact target blobs:
- `source/product/fabushi/fabushi-account-service.ts@067b644ee568743892ba8cd0126d8b040eb21d4b`
- `source/mahayana/mahayana-rs/mahayana-product/src/native_desktop_session.rs@d3941bf971ce0d357263ee298e1cdb1f264c50c0`
- `source/host/src/extensions/auth/auth_service.rs@c9fc53ef66e7f1a1c06381a40515edb410f3418b`
- `source/mahayana/mahayana-rs/mahayana-telegram/src/lib.rs@fce2ef0a509d6033697fac29a121b429ee4890bd`
- `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs@86cfa4c38bd535939e9299b79b8d72ce2db6fc70`

The canonical Fabushi account owner now keeps a stable 32-byte Telegram storage key in the existing OS-protected secret lifecycle and projects it only through the private Host credential file. AppHost hashes canonical account identity into an account-scoped database path, restores/creates one encrypted persistent embedded client, retains its process-lifetime lease, and hands only its typed client ID to the existing Host/ConversationProvider composition. This deliberately does not create a Telegram account subsystem, second runtime, sidecar, or source-named state store.

Authored-only test blobs are `source/product/fabushi/fabushi-account-service.contract.test.ts@969eb3cede3060366e1b73c4d4106bcf1f49be89` plus Rust parser/database-path unit contracts in the target blobs above. No test/build/typecheck/lint/schema validator/generator/packaging/runtime acceptance was run. Therefore this slice is `implemented / validation_status=not-run / accepted=false`; global `unknown=16172`, `unread=16145`, and `omitted=0` are unchanged by this mapping.


## Canonical Telegram authorization and self-user identity handoff (round 42, 2026-11-01)

Stage A remained validation-execution-free. Fixed-U account lifecycle remains anchored at `Telegram/SourceFiles/main/main_account.cpp@2721c7ac12effe454d398610d26d638fadaa1f68`; exact fixed-U protocol corroboration is `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`, including `auth.sentCodeSuccess#2390fe44`, `auth.authorization#2ea2c0d4`, `auth.authorizationSignUpRequired#44747e9a`, `userEmpty#d3bc4b7a` and the current full `user` constructor.

Implementation through `d19fc3742b9a2d5076513ec444383d7683a35ea2` closes the previously explicit authorization/self-user settlement slice: successful send-code/sign-in/password/registration responses extract canonical `self_user_id`; authorization/challenge state and MTProto auth-key restore material are encrypted in the existing account-scoped SQLite store; Ready restore without identity/key fails closed; the same `TelegramClientLease` exposes `AuthorizationSnapshot` into `HostCreateConfig`; and the existing non-owning provider resolves identity from that same runtime so same-process login does not create or require a second runtime.

The ledger adds `TDRP-R9-ACCOUNT-AUTHORIZATION-SELF-IDENTITY-001`; `responsibilities_total=54`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged because no whole upstream file is claimed semantically closed by this slice. All authored tests remain unexecuted. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.


## Stage-A canonical Telegram login entry / private application credential (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact fixed-U source reads anchor the applicable login behavior to `intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246` (`auth.sendCode`), `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489` (`auth.signIn`), `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e` (`auth.checkPassword`) and `intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96` (`auth.signUp`). These are behavioral provenance anchors only; no whole intro file is claimed semantically closed and global unknown/unread counts remain unchanged.

Production implementation through `c8109e3412d8c2cb4e7362a0fe4ec4976c07bd22` keeps existing owners: `FabushiAuthService` reads an application-scoped `api_id/api_hash` pair from the existing protected main-process secret store, validates the exact bounded shape, and copies it only through the existing private mode-0600 Native Host credential. The private Rust credential reader retains the pair in process memory; `TelegramApplicationCredentials` redacts `api_hash` from Debug. Renderer-facing methods never accept or return either application credential.

The existing production renderer/Coordinator carrier now exposes typed `feature.auth.telegramStatus`, `telegramSendCode`, `telegramSignIn`, `telegramPassword` and `telegramRegistration` calls. AppHost routes all of them to the same process-lifetime `TelegramClientLease`. First send-code bootstraps transport only when no live/restorable transport exists; restored auth-key material is reused. Every authorization mutation rereads the same runtime `AuthorizationSnapshot` and returns only renderer-safe authorization state/self-user identity while the existing Host/ConversationProvider continues to resolve that same client.

Actual Telegram application credential values remain external product configuration and are intentionally absent from repository and renderer state; missing/malformed configuration fails closed. Authored contracts cover application-credential bounds, private Native parsing and renderer parameter secrecy. Per Stage A policy, no build, typecheck, lint, schema validator, generator, test, package, deployment, installation, workflow/status query or runtime acceptance was executed. State is `implemented / validation_status=not-run / accepted=false`.


## Stage-A Telegram auth lifecycle typed error/backoff contract (round 44, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual exact-source reads covered `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`, `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`, `intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`, `intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246` and `intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96`. These sources establish resend/cancel, password recovery, email verification, flood handling, challenge invalidation and bounded retry/error lifecycle responsibilities; they are behavioral anchors only and do not close the whole intro files.

Production implementation through `a9e9683a3bf8cd05fd79b46213c19f56bc7c606d` plus the manual bounded-number guard correction `e7c0da2a654aad9a65e6c7fc545f9cf7d9dd48a8` keeps the existing owner chain. The retained `TelegramClientLease` now classifies bounded Telegram auth errors, turns `FLOOD_WAIT_*` / `FLOOD_PREMIUM_WAIT_*` into a capped same-process backoff, prevents hot-looped same-lease auth mutations while that deadline is active, and always projects the live `AuthorizationSnapshot/self_user_id` on recognized issues. The shared Coordinator contract now has a discriminated Telegram authorization-state union plus bounded error/backoff validation; the production renderer client consumes that shared type rather than `unknown`. AppHost status remains routed to the same lease. No second account/runtime/provider/state store was added.

The current embedded runtime was also inspected at `bhrumom/fabushi@edde714def34e6eb5ae133dba20790c80a355c85`: it implements send-code, code submit, password submit and registration, but has no protocol command/builder path for `auth.resendCode`, `auth.cancelCode`, `auth.requestPasswordRecovery`, `auth.checkRecoveryPassword`, `account.sendVerifyEmailCode` or `account.verifyEmail`. Those remote lifecycle operations therefore remain explicit fail-closed implementation blockers instead of being fabricated in renderer/AppHost code. The source-specific `SRP_ID_INVALID` challenge refresh likewise remains blocked on a safe runtime `account.getPassword` refresh primitive; the typed projection records the required action without retry-spinning.

Authored-only contracts are the inline Rust auth-error/backoff test and `source/shared/rpc/coordinator.contract.test.ts`; neither was run. Stage A remained execution-free: no GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, packaging, deployment, installation or runtime acceptance occurred. The ledger adds `TDRP-R9-ACCOUNT-TELEGRAM-AUTH-LIFECYCLE-ERROR-BACKOFF-001`; `responsibilities_total=56`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force.


## Stage-A remaining Telegram auth lifecycle primitives (round 45, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact TL authority is `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`; fixed-U intro behavior remains anchored by `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`, `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`, and `intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`.

Canonical shared owner work is committed on `bhrumom/fabushi-platform-core` checkpoint `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: protocol `314e8804ddfc997acdde2c19bff831b5e26b1706`, encrypted storage `bcb8298a4622616f2f8a226899c0e3dc8aa3ba8f`, runtime `ec854e93fb1523d95225203554600858798e4981`, and spec/evidence `a50f33da685506f4e170729a9d11f01f8dda8ec5`. These add auth.resendCode, auth.cancelCode, auth.requestPasswordRecovery/auth.checkRecoveryPassword, account.sendVerifyEmailCode/account.verifyEmail, explicit account.getPassword challenge refresh, and encrypted pending AuthorizationState/AuthContext persistence.

Desktop implementation `169d6cc7798507f0302d6c93fef7deef1a302d28` wires the matching seven commands through the existing `TelegramClientLease`, AppHost `feature.auth`, shared Coordinator contract and production renderer client. No second account/runtime/provider/state store was added, and no api_hash is exposed to renderer code.

The current desktop checkpoint is a sparse overlay and contains no Cargo/dependency manifest, so exact binding of its `fabushi_telegram_runtime` dependency to the platform-core checkpoint cannot be proven from this ref. That remains fail-closed integration evidence, not silently treated as shipping validation.

All protocol/storage/runtime/coordinator tests are authored only. No build, typecheck, lint, schema validator, generator, test, packaging, deployment, installation, workflow/status query or runtime acceptance was executed. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.

## Stage-A Telegram runtime Cargo composition fusion (round 46, 2026-11-01)

The previously recorded sparse-checkpoint composition blocker is now statically closed without executing validation. The actual Mahayana Cargo authority is preserved exactly: `source/mahayana/mahayana-rs/Cargo.toml@144c67c76daaf2229e817f9830e678a3e6d4cf3a` includes `mahayana-telegram` and `providers/telegram-runtime`; `mahayana-telegram/Cargo.toml@f43109356f1f486a82a2d0120f6e78808c431c40` path-depends on `fabushi-telegram-runtime = { path = "../providers/telegram-runtime" }`; and `providers/telegram-runtime/Cargo.toml@0fc8ee357ef5c00074e5fdcd1d7594860e289bd1` retains the sibling protocol/network/storage path dependencies.

Canonical shared-owner fusion is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@13f5458aadf19a87c52bfa904f9942d81c88979a`. It preserves the prior `AuthorizationSnapshot/self_user_id` and resumable MTProto auth-key baseline while retaining resend/cancel/recovery/email-verification/account.getPassword challenge-refresh primitives. Protocol success parsing now carries canonical `auth.Authorization.user.id`; encrypted storage is schema v4 and converges both historical v3 table shapes (`private_metadata` and `secure_value`) without creating a second persistence owner.

Desktop commit `43d9d398801f2885da982d106c17a74efb1f9472` synchronizes those canonical protocol/network/storage/runtime sources into the existing `providers/telegram-*` path dependency chain; authored-only composition/runtime/storage/protocol contracts are saved through `221a661e17f2ad417e349df6604815d3fbfa946f`. The existing checkpoint `mahayana-telegram/src/lib.rs` lease/AppHost/Coordinator wiring is preserved; no second account, runtime, provider, session or state store is introduced.

This is Stage-A static implementation/composition evidence only. No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed. `responsibilities_total=57`, `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged; `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force. Broader fixed-U source/implementation closure, complete Boost/libiconv archive member enumeration, original-host GitLab blockers, legal/release review, Stage-B executable evidence and independent ACCEPT remain open.


## Stage-A MTProto bad-message recovery (round 47, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The next build/shipping-reachable responsibility was manually read at `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, corroborated by `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. Fixed-U `bad_msg_notification` codes 16/17/64 correct trusted server time and resend, while 32/33 reset inconsistent session sequence state; unrelated/fatal codes do not silently recover.

Existing-owner-first implementation is committed in `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@ac9e20217812aa72b92049d4dfb52f76d6380da6` after durable spec correction `47ed28d35dbb6c7dd19bdcf1cd66fbf88dc68f67`. Canonical blobs are `native/telegram-network/src/lib.rs@ba9935b54e9c178c7d9377cd75572f2c21b62144` and `native/telegram-protocol/src/session.rs@ab8fd351a05543854c3f62de5c8332a13b7ff472`. Recovery remains in the unique `EstablishedSession`: 16/17/64 update the live clock offset, clear only the client message-id floor while preserving server replay history, and retry once; 32/33 rotate the session id/reset `SessionSequence` and retry once; mismatched/fatal/repeated recovery fails closed.

Desktop implementation commit `6c714e664d6cd4d66228b313664a6e776a4fd558` synchronizes those exact blobs into the existing Cargo path-dependency chain at `providers/telegram-network` and `providers/telegram-protocol`; no second account, runtime, provider, transport, auth key, session owner or state store was introduced. Inline protocol/network recovery contracts are authored only and were not run.

The ledger adds `TDRP-R9-MTPROTO-BAD-MESSAGE-RECOVERY-001`; `responsibilities_total=58`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged because this is a responsibility slice rather than whole-file semantic closure. No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force. Broader fixed-U closure, archive/legal blockers, Stage-B executable evidence and independent release ACCEPT remain open.


## Stage-A MTProto new-session pending-RPC replay (round 48, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U read of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` lines around `mtpc_new_session_created`, corroborated by `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`, shows that the session adopts the new salt and resends pending requests whose message ids are older than `first_msg_id`.

The canonical shared owner remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `8275952bed7bea3a7c275f1b122c2c9c710ba262` precedes implementation commit `6ec2d64da2d30d78c919bd971e8e0d1953aed587`; the network blob is `7df3dd74ed1cc4fea5273a36152d1ff8bd00ec65`. Fabushi's synchronous `invoke_raw` has one outstanding RPC, so the fixed-U pending-request responsibility maps exactly to that request: if its id is less than `first_msg_id`, update `server_salt` and replay once with a fresh id on the same auth key/transport/runtime; equal/newer requests continue waiting; a repeated replay signal fails closed with `NewSessionReplayLimit`. Ping adopts the salt but does not fabricate content-RPC replay.

Desktop commit `01b78f13e4a500bdb401c080a40f5295d8110779` synchronizes the exact network blob into the existing `providers/telegram-network` path dependency. The ledger adds `TDRP-R9-MTPROTO-NEW-SESSION-REPLAY-001`; `responsibilities_total=59`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Higher-level propagation of `new_session_created` into update/difference recovery remains a separate open responsibility and is not claimed by this slice.

Inline predicate/replay contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, installation or runtime acceptance was performed. `accepted=false`, `validation_status=not-run`.


## Stage-A MTProto detailed-answer resend (round 49, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source authority remains `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` plus `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`: matching `msg_detailed_info#276d3ec6` acknowledges the sent request and requests missing `answer_msg_id` through `msg_resend_req#7d861a08`.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec `9bea551d1e26a4ff5efa522417999388256486ff`, implementation `0536aa2ad9f93e59f1b1e3cdc19aabaf9a104fce`, protocol api/lib blobs `1feaefd062f4e8635bbf002a7dce30b804c9f6ed` / `3e17847045f3be606d8b821c85f7b8136b89474a`, network blob `581156c32289082ad1f53410ab0637bdeef85c66`. The existing protocol owner adds a fixed-U `build_msg_resend_req`; the existing synchronous `EstablishedSession` sends that non-content service request only when `msg_detailed_info.msg_id` equals the current outstanding RPC, then keeps waiting.

Desktop `3f5e2d3a8b2e0c2726a1b1dcd1ab1172a0c5758d` synchronizes those exact blobs into the existing provider path chain. No second request history, runtime, session, provider or transport owner is introduced. `msg_new_detailed_info` and `msgs_state_req/info/all_info` full reconciliation remain explicitly open.

Ledger adds `TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001`; `responsibilities_total=60`, `unknown=16172`, `unread=16145`, `omitted=0`. Contracts are authored only; no Actions/status query or trigger, build, typecheck, lint, validator, generator, tests, packaging, deployment, installation or runtime acceptance occurred. `accepted=false`, `validation_status=not-run`.


## Stage-A MTProto single-RPC state reconciliation (round 50, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U source review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` plus `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` closed the subset of `msg_new_detailed_info`, `msgs_state_req`, `msgs_state_info`, `msgs_all_info`, and `handleMsgsStates` that Fabushi's existing one-outstanding-RPC owner can represent exactly. The upstream low-three-bit rule is preserved: `(state & 0x07) == 0x04` means the server received the request; other states require replay.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `469e46f72d15138aa4fb343a49e06feeaefb1303` defines the single-RPC boundary; implementation commit `404623cad89e0d77ab326e140784b893d677f31e` adds `msgs_state_req` encoding, bounded `msgs_state_info/msgs_all_info` parsing, one current state-query id, one bounded state-driven replay through the existing outer retry loop, and `msg_new_detailed_info` answer reconciliation through the existing server-message replay window. Canonical blobs are protocol api `ed695ed575c5950c99818acc7086a2cf8b1c712d`, protocol lib `21ec771260dd356d4a8e653be6e2e250a32ab5ce`, and network `c24515c2ee15608f51970a5420769e7713e60327`.

Desktop implementation commit `fe17d4bcdea22d1df8e474961aa12b5f04b291db` synchronizes those exact blobs into the existing provider path chain. No second request history, runtime, session, provider, transport or received-message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `_stateAndResendRequests`, simultaneous state queries, queued-resend cancellation, container-wide resend bookkeeping, or durable multi-request history remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-STATE-RECONCILIATION-001`; `responsibilities_total=61`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC acknowledgement (round 51, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next service-message slice that the existing one-outstanding-RPC owner can represent exactly: `mtpc_msgs_ack` plus the `requestsAcked(..., byResponse=false)` rule that a normal ACK does not settle a callback RPC still waiting for a response.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `9f72142c7b4d8ab0e605e95301f4194d964ffa73` defines the single-RPC acknowledgement boundary; implementation commit `3470cd572891222c8846a9e91f98ca621624ec64` adds bounded `msgs_ack` parsing, explicit RPC/ping ACK dispatch, current-request correlation, malformed-payload fail-closed behavior, and callback-RPC non-settlement. Canonical blobs are protocol api `d38546a0f6954bb6a03aaec80968dbf195945edc`, protocol lib `da012dab3b1111f18c318cf0001f2f3d1f32bf6f`, and network `70645dd06426374ae4e20bd9c9545388948f65bb`.

Desktop implementation commit `6804906c0646d75a16b150b4f745476f44e19953` synchronizes those exact blobs into the existing provider path chain. No second request history, sent/resend queue, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, container ACK fan-out, queued-resend cancellation, or per-request fast-clock history remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-ACKNOWLEDGEMENT-001`; `responsibilities_total=62`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-ping pong correlation (round 52, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next adjacent service-message slice that the existing single-outstanding-ping owner can represent exactly: `mtpc_pong` / `pong#347773c5 msg_id:long ping_id:long`. Telegram Desktop additionally consults historical `wasSent`; Fabushi intentionally maps only the current-ping correlation that does not require a multi-request sent history.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `47246159887072673aff6609fb57f9cbc81ff2a2` defines the single-ping boundary; implementation commit `bf6f063b229be43597bddc4231baab24d06a09bd` adds bounded `parse_pong`, strict trailing-byte rejection, explicit `Pong` shape, and `pong_matches_request` correlation requiring both the current request message id and ping id. Canonical blobs are protocol api `32a7610bfc74835734c1e5c4385e4eb4cb2ee1f3`, protocol lib `ffbabc8f66112c2321be75a34928b086d6adf57f`, and network `e0bf1b5f286ad0f8bc45a36e86074e19b7dde560`.

Desktop implementation commit `3e363ccdaf287878f457797442aa97e8b597384b` synchronizes those exact blobs into the existing provider path chain. A valid exact pong settles only the current ping; either id mismatch uses the existing `PingMismatch` failure path, and malformed/trailing payload fails closed as `InvalidServiceMessage`. No second request history, ping history, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, historical sent-message lookup, container fan-out or queued-resend cancellation remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-PING-PONG-001`; `responsibilities_total=63`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC result correlation (round 53, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next adjacent service-message slice that the existing one-outstanding-RPC owner can represent exactly: `mtpc_rpc_result` / `rpc_result#f35c6d01 req_msg_id:long result:Object`. Telegram Desktop additionally consults historical `wasSent` and multi-request acknowledgement state; Fabushi intentionally maps only current-request correlation and leaves that history open/fail-closed.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `6511c5f887c16caa0494918c987fb400774c4349` defines the single-RPC-result boundary; implementation commit `144512ce2bb7366aef57ba76924b2d02852cbce3` adds bounded `parse_rpc_result`, explicit `RpcResultEnvelope`, validation of non-empty word-aligned opaque result bytes, and `rpc_result_matches_request` correlation against only the current request. Canonical blobs are protocol api `b8b9872ec8ce7548817ebbe92615d47aed470e89`, protocol lib `4b6695cac59dc507f76c85f93ff3678d0d5bcbd0`, and network `8ec039d4a12040fcea87af883dba9cd02a2b3efb`.

Desktop implementation commit `7dc33be41568162c2cab398da67b3e6bc9c12dea` synchronizes those exact blobs into the existing provider path chain. A well-formed result settles only when `req_msg_id` equals the current `request_message_id`; unrelated result ids continue waiting, and malformed/empty/non-word-aligned envelopes fail closed as `InvalidServiceMessage`. No second request history, callback registry, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `wasSent/haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, container fan-out, queued-resend cancellation or multi-request callback routing remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-RESULT-001`; `responsibilities_total=64`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC message-container traversal (round 54, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` selected the adjacent `mtpc_msg_container` receive-side branch. Fixed U requires a bounded contained-message count, per-entry `msg_id/seq_no/bytes`, body length of at least one TL word and word alignment, ACK collection for odd `seq_no`, replay registration, and recursive service handling. This round maps only the receive-side subset that Fabushi's existing current RPC/current ping owners can represent; Telegram Desktop's outbound `_sentContainers` and arbitrary historical request fan-out remain open/fail-closed.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `b223407bc9829a2fc14449bb5a7cfe6d7f4b082e` defines the bounded receive-container boundary; implementation commit `98a2ff425407725571966a4ed17b1643fba87b4d` adds `parse_message_container`, `MessageContainer/MessageContainerEntry`, 1024-entry bounding, minimum/aligned body checks, exact full-buffer consumption, and reuses the typed parser in both current-RPC and current-ping network paths. Canonical blobs are protocol api `14c282bfc20a33446c69a47db4fdd6f0b887348a`, protocol lib `f21d38495e5a7f5c6e2a35f28a633ac6e3433d4b`, and network `bab8893bd8a2a0632014d826503a8091cfe64d46`.

Desktop implementation commit `e8091133b85eb7e368839e6c9b09547620be17c2` synchronizes those exact blobs into the existing provider path chain. The existing `MessageIdGuard` still validates every contained server message id before recursion; the RPC path adds only odd-sequence contained ids to its existing acknowledgement accumulator; nested results/pongs/recovery continue through existing exact current-request/current-ping correlation. No second request history, sent-container store, callback registry, runtime, provider, transport, session or message store is introduced.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-MESSAGE-CONTAINER-001`; `responsibilities_total=65`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Parser/sequence-parity contracts are authored only. `gzip_packed` decompression nesting/budget semantics and outbound/multi-request `wasSent/haveSent/_resendingIds/toSend/_sentContainers` remain open and fail-closed. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto new-detailed-info exact wire validation (round 55, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` confirmed the adjacent `mtpc_gzip_packed` path requires `kMaxGzipNesting=64`, a shared `unpackedBudget` initialized to 32 MiB, and zlib/gzip decompression. The canonical `native/telegram-protocol/Cargo.toml@d8a5261034b1eb395d12149a6bcc34cd7e371b73` and `Cargo.lock@97ad9046ce8926764258c978664cd8cd92fce503` contain no approved `flate2`/\`miniz_oxide` decompression owner, so gzip remains explicitly fail-closed rather than introducing an unreviewed dependency.

The next independently implementable responsibility is the complete fixed-U wire contract for `msg_new_detailed_info#809db6df answer_msg_id:long bytes:int status:int` from `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. Canonical spec commit `40d53cfb44f2a495e4bf1a84002f653f555532e5` precedes implementation commit `5bdf993dc1048ed22194d7f1c0439049e820ee5f`. The existing protocol owner now exports `MsgNewDetailedInfo`, `MSG_NEW_DETAILED_INFO_CONSTRUCTOR`, and `parse_msg_new_detailed_info`; the existing `EstablishedSession::scan_rpc_body` uses that parser before the existing detailed-answer ACK/resend reconciliation. Canonical blobs are protocol api `eff20dbc601ca78fa7e6a4662bfeda088706d4f4`, protocol lib `b6ab0397cbad0776b269d965f89427030824ba4c`, and network `7f3b140a9becda2305a2f5e252fcea61108597e0`.

Desktop implementation commit `3b37ff49e7c3b100af12206ab88662f4d83959c2` synchronizes those exact blobs into the existing provider path chain. Truncated, wrong-constructor, or trailing `msg_new_detailed_info` payloads now fail as `InvalidServiceMessage` before an ACK or `msg_resend_req` side effect; a valid message still reuses the current session replay window and existing resend primitive. No second request history, decompression runtime, provider, transport, session or message store was introduced.

Ledger adds `TDRP-R9-MTPROTO-NEW-DETAILED-INFO-WIRE-001`; `responsibilities_total=66`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact-field and malformed-payload contracts are authored only. `gzip_packed`, `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, higher-level new-session update/difference propagation, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto new-session wire validation and high-level difference blocker (round 56, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review bound the low-level service object to `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `new_session_created#9ec20908 first_msg_id:long unique_id:long server_salt:long` in `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. The adjacent high-level chain is `Api::Updates::mtpNewSessionCreated -> _updatesSeq=0 -> getDifference`; full/slice differences are applied before intermediate/final state advancement in `api_updates.cpp@71c929cfe3c779c94719efef32a47a4d029e2c67`.

Existing-owner-first review found the high-level recovery is not yet precisely implementable: canonical `telegram-protocol/src/updates.rs` can build `updates.getDifference` and parse only update state plus terminal empty/too-long differences, while no current protocol/runtime bridge decodes and applies arbitrary `Message/EncryptedMessage/Update/Chat/User` vectors from `updates.difference#f49ca0` or `updates.differenceSlice#a8fb1981` into the existing TelegramEngine owners. A terminal-only recovery would drop missed updates or advance state without applying them, so this responsibility is recorded fail-closed in platform-core spec `telegram-new-session-update-difference-recovery.md` from spec commit `ba9143b660d58728de8b93a49e7c2f97fcd39ed5`.

The next independent responsibility, exact new-session wire validation, is implemented instead. Platform-core implementation `dc66cc7aae7644530fa03160438df5174809beb4` adds `NEW_SESSION_CREATED_CONSTRUCTOR`, `NewSessionCreated`, and `parse_new_session_created` to the existing protocol updates owner, consuming all three fixed-U fields and rejecting wrong constructors, truncation, and trailing bytes. Both `EstablishedSession::scan_rpc_body` and `process_service_body` now parse the complete typed object before server-salt mutation or the existing bounded current-RPC replay decision. Canonical blobs are protocol updates `23a09bb6e4e78ef526823cf01d0a207565fb2a3e`, protocol lib `7b9c641eabccb7dba8c5a425904b6a676b12d866`, and network `234c6c0070a1a92218fe5c974658d16d37fee46a`.

Desktop implementation `23b24ab2db29697071d21eb747f475b2196d891f` synchronizes those exact blobs into the existing provider composition. The provider protocol tree did not previously contain `src/updates.rs` even though its synchronized `lib.rs` declares `pub mod updates`; this round adds the canonical file at that declared existing module slot rather than introducing a second owner.

Ledger adds `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001`; `responsibilities_total=67`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact-field/malformed contracts are authored only and were not executed. High-level full/slice difference recovery, `gzip_packed`, historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed. No Actions/status/check query or trigger, build, typecheck, lint, validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto detailed-info exact wire hardening (round 57, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirmed that `msg_detailed_info#276d3ec6` is a four-field object (`msg_id:long answer_msg_id:long bytes:int status:int`) and Telegram Desktop performs generated TL decoding before using it. The existing Fabushi network branch still read only the first two fields by raw offsets, so malformed/trailing objects could reach the current-session ACK/resend path.

Spec-first canonical work remains on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `8fffe0ca4e7d48d097e9f15f0cf332610e28f4fc`, implementation commit `4c5bea7b847a3e04ee3d0c9b05fab228101acb64`, and source-level compliance commit `46cf745e2c228f0638d2d0afa2856579ddba3d70`. Canonical blobs are protocol api `5da871fc557a49a7e57f1dd0677f307a5ed1bac8`, protocol lib `3f7c363f9fd2eb86e60c8294144d7f1dced9ccca`, and network `2a50068d148b25e710961973315d78fc1b1ee5a8`. The protocol owner now exports `MSG_DETAILED_INFO_CONSTRUCTOR`, `MsgDetailedInfo`, and `parse_msg_detailed_info`; wrong constructor, truncation, and trailing bytes fail before network side effects. The network owner parses the complete object first, then keeps the existing one-outstanding-RPC current-id gate and `reconcile_detailed_answer` behavior.

Desktop implementation commit `45c24752b9d4764768a9eec26ead9fb6b36dc426` synchronizes those three blobs byte-for-byte into the existing provider chain; no second Telegram protocol/runtime/provider, request history, callback registry, or received-message store was introduced. The existing ledger responsibility `TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001` is updated rather than creating a duplicate responsibility; `responsibilities_total=67`, `unknown=16172`, `unread=16145`, `omitted=0` remain unchanged.

Exact-field, wrong-constructor, truncation, trailing-byte, and current-request contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference decoding/propagation, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain explicitly open and fail-closed.


## Stage-A MTProto future-salts exact wire validation (round 58, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Reread of `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms `future_salts#ae500895 req_msg_id:long now:int salts:vector<future_salt>` and nested `future_salt#0949d9dc valid_since:int valid_until:int salt:long`. The existing Fabushi receive branch still recognized `ae500895` locally, read only `req_msg_id` from raw offset 4, and returned the entire body for a matching current request without validating `now`, the TL vector, nested constructors/fields, or trailing bytes.

Existing-owner-first review found this slice is independently representable by the canonical one-outstanding-RPC owner: complete response validation and current-id correlation require no historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state. Platform-core spec commit `69167c52582d67df73c2284bb1e98a47c3846343` defines that boundary. Implementation commit `f631d7cc0c1a21db93ab647d3b6dcb29b42cb74e` adds `FUTURE_SALT_CONSTRUCTOR`, `FUTURE_SALTS_CONSTRUCTOR`, typed `FutureSalt/FutureSalts`, and `parse_future_salts` in the existing protocol owner. It validates top-level/vector/nested constructors and all fields, checks the declared count's fixed-size byte requirement against remaining input before vector allocation, and rejects truncation or trailing bytes. The network branch parses first, then settles only when the typed `request_message_id` equals the one outstanding RPC, preserving the exact validated raw object as the existing `RpcResult.body`. Canonical blobs are api `bdac614fab9ca16ffeb36c7e98a012fe20231597`, lib `6b6bde59a9f5e86ec33b0258e1e6a34f2b1622f9`, network `b2eb7022c6a8d38d7ac9f4a9fd13fe22913621de`; source-level compliance commit is `f604e40e175d6a05ef10d6c4811ec1e363c85529`.

Desktop implementation commit `a3329a8c948cbffab1d6843c78abc8024afa87b4` synchronizes those three blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001`; `responsibilities_total=68`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was added.

Exact-field, wrong top-level/vector/nested constructor, truncation, impossible-count, trailing-byte, and current-request contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto bad-server-salt exact wire validation (round 59, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms that `bad_server_salt#edab447b` is a four-field object: `bad_msg_id:long bad_msg_seqno:int error_code:int new_server_salt:long`. Telegram Desktop first decodes the generated TL object and then consumes all four fields. The existing Fabushi RPC and ping/service branches still used raw offsets for only `bad_msg_id`, `error_code`, and `new_server_salt`, leaving `bad_msg_seqno` untyped and allowing trailing bytes before live salt mutation.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `12bc8a0ce1e453af1b3f2af0ffbd9ce868c92447`, implementation commit `3a850171c59a5f27809e176ba54971c7c0c003dc`, and final source-level compliance commit `515857196fd5fd8688326c7971a60079b67a0b71`. Canonical blobs are protocol api `75959eb82460cb36bf5f43c6e4470ef8814ea2a9`, protocol lib `3540f1a60df0d07f77b6e38349e0c500ec2d12d8`, and network `42be9e9fa78c3acc77367799c1ecb506cdd25ab0`. The protocol owner now exports `BAD_SERVER_SALT_CONSTRUCTOR`, typed `BadServerSalt`, and `parse_bad_server_salt`; it consumes `bad_msg_id`, `bad_msg_seqno`, `error_code`, and `new_server_salt`, and rejects wrong constructors, truncation, and trailing bytes. Both network receive branches parse first, then correlate only the one outstanding request before assigning the exact parsed server salt and preserving the existing bounded `NewSalt` retry outcome.

Desktop implementation commit `131980d0bc3c5b6e8fad3ad2e394715426cd1529` synchronizes those three canonical blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-BAD-SERVER-SALT-WIRE-001`; `responsibilities_total=69`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was introduced.

Exact-field, wrong-constructor, truncation, trailing-byte, both-receive-branch, and current-request contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto bad-msg-notification exact wire validation (round 60, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms `bad_msg_notification#a7eff811 bad_msg_id:long bad_msg_seqno:int error_code:int`; Telegram Desktop first decodes the generated TL object and then reads all three fields. The existing Fabushi RPC and ping/service branches still read only `bad_msg_id` at raw offset 4 and `error_code` at raw offset 16, while `direct_bad_message_time_recovery` also granted its clock-window exception from raw constructor/error-code offsets. This skipped typed `bad_msg_seqno` validation and allowed trailing bytes to reach recovery behavior.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `42d90d6351a31f6d0815c47f2d647ae8162eccf8`, implementation commit `0e569130a80a53b48b74f8e27a96004363b30e7e`. Canonical blobs are protocol api `15e34bbb0e9c5a5d4fdd8b7a9cdedf3d6b123080`, protocol lib `2c4897cfdba4502d8ca880d7a58cfd0e88877bf7`, and network `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. The protocol owner now exports `BAD_MSG_NOTIFICATION_CONSTRUCTOR`, typed `BadMsgNotification`, and `parse_bad_msg_notification`; it consumes `bad_msg_id`, `bad_msg_seqno`, and `error_code`, rejecting wrong constructors, truncation, and trailing bytes. Both network receive branches parse first before current-request correlation or recovery action, and the direct clock-window exception now requires the same fully valid typed object. The decoded sequence number is retained without introducing historical sent/container reconciliation.

Desktop implementation commit `1205f8709995aa4f5fd5dafdc9fb5121fbfce06c` synchronizes those three canonical blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-BAD-MSG-NOTIFICATION-WIRE-001`; `responsibilities_total=70`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was introduced.

Exact-field, wrong-constructor, truncation, trailing-byte, current-request correlation, recovery-code mapping, and malformed-clock-window contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto RpcDropAnswer exact wire validation (round 61, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `rpc_answer_unknown#5e2ad36e`, `rpc_answer_dropped_running#cd78e586`, and `rpc_answer_dropped#a43ad8b7 msg_id:long seq_no:int bytes:int`. The fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `rpc_answer_*` branch; these are schema-defined `rpc_drop_answer` result bodies.

Spec-first canonical work: platform-core spec `0c85884ee491d985e4fbf0d6ec1d1acca1a7d8a4`, implementation `5251ab3eacd8035cb8bb86609d76641233dd22eb`; canonical blobs api `77b9cc0a7d191ef9f197fb8fd718e29631197ddc`, lib `9229c83fa6b484a5af657ee37f0bd827fecdc28d`. The existing protocol owner now has typed `RpcDropAnswer` variants and `parse_rpc_drop_answer`, with exact zero-field/trailing validation for Unknown/DroppedRunning and exact `msg_id/seq_no/bytes` parsing plus truncation/trailing rejection for Dropped. Desktop implementation `4257fc9c875526c4b49a1f3cfbbeec7cd5a3eb0f` synchronizes both protocol blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001`; `responsibilities_total=71`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No outbound `rpc_drop_answer` production call path or historical request registry exists, so interpreting these results as drop/cancel state mutations remains explicitly fail-closed; no `wasSent/haveSent/_resendingIds/toSend/_sentContainers` semantics were invented.

Parser contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.


## Stage-A MTProto DestroySessionRes exact wire validation (round 62, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `destroy_session_ok#e22045fc session_id:long`, `destroy_session_none#62d350c9 session_id:long`, and `destroy_session#e7512126 session_id:long`. The fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `destroy_session_*` receive branch, so wire decoding is separated from any live teardown lifecycle.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `dee7cff150070d7891478510386674970a6217af`, implementation `cfe6a0564c40d2c9b9ea62b686470139a1eeb2ab`; canonical blobs api `d82cca134274b8d939c1140ec44ee00230e2cfac`, lib `655498ce9db2faf8a76f093250d4a1f02f77ff67`. The existing protocol owner now exports typed `DestroySessionResult::{Ok,None}` and `parse_destroy_session_result`; both variants preserve exactly one `session_id:long` and reject unknown constructors, truncation, and trailing bytes. Desktop implementation `b179796344fba3e22bccb1310b74f229d613b531` synchronizes both protocol blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-SESSION-WIRE-001`; `responsibilities_total=72`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No canonical outbound `destroy_session` production path or live teardown owner exists, so interpreting Ok/None as session mutation remains explicitly fail-closed; no `wasSent/haveSent/_resendingIds/toSend/_sentContainers` semantics were invented.

Parser contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.


## Stage-A MTProto http_wait exact wire validation (round 63, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `http_wait#9299359f max_delay:int wait_after:int max_wait:int = HttpWait`. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` constructs `MTP_http_wait(100, 30, 25000)` only when `_connection->usingHttpWait()`; no independent receive-side `mtpc_http_wait` service-message branch was found. The call-site values are therefore recorded as lifecycle evidence, not treated as schema validity constraints.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `03b40ed111c8e64d159ea837848bfae0fa85102c`, implementation commit `96bb0e2cd31cdfc82aaa44e0f8e25acf8c30a204`; canonical blobs api `7b731b6d6bbd2009453c5107a24ebea58cc2ece3`, lib `4d8b7198fbe7aa8cbb71180587970760f6ebd1d9`. The existing protocol owner now exports `HTTP_WAIT_CONSTRUCTOR`, typed `HttpWait { max_delay, wait_after, max_wait }`, and `parse_http_wait`; it consumes exactly constructor + three TL ints and rejects wrong constructors, every-byte truncation, and trailing bytes. The parser is side-effect free and introduces no connection/session/request mutation.

Desktop implementation commit `1495975bbdd2e5d56826a95854def198862e2c5b` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-HTTP-WAIT-WIRE-001`; `responsibilities_total=73`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Telegram Desktop-equivalent `usingHttpWait()` connection scheduling is not represented by a proven canonical owner, so that lifecycle remains explicitly fail-closed; no second HTTP scheduler/runtime/provider or `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history was invented.

Exact-field, wrong-constructor, every-byte truncation, and trailing-byte contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary historical sent/resend/container state, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open/fail-closed.


## Stage-A MTProto help.configSimple exact wire validation (round 64, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares adjacent `ipPort#d433ad73`, `ipPortSecret#37982646`, `accessPointRule#4679b65f`, and `help.configSimple#5a592a6c`. `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no help.configSimple receive/application branch. The actual fixed-U higher-level consumer was reread at `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` / `special_config_request.h@91901489e778089b03187682a9556e929867b94f`: after decrypting a simple config it checks expiry and non-empty rules, applies phone-prefix filtering, converts raw IPv4 values, and emits DC endpoint callbacks. Those lifecycle/policy steps are intentionally not inferred by the wire parser.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `db4e6db92d6ebf6f03442cb3eb2134b9bf0a144e`; implementation commit `cde1297cacd3861764d74a111629d002e74c9d71`; constructor-first fail-closed correction `46db4422aa7221b1c8048f96f1c7c47cd0c01458`. Canonical blobs are api `0c852a5e3bc41a4ac571d713ae3d4edea19b3619` and lib `04e5020e7b1e7ed46b403c151f84a095a1a17e4c`. The existing protocol owner now exports all four fixed-U constructors, typed `IpPort`, `AccessPointRule`, `HelpConfigSimple`, and exact parsers for each object family. Nested vectors use the existing bounded TL reader; raw signed ints, strings, bytes and ordering are preserved; wrong constructors, malformed nested framing, truncation, and trailing bytes fail closed.

Desktop implementation commit `870ec15e7760718ad7ef1eb5e578dfcef8f0e46b` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-CONFIG-SIMPLE-WIRE-001`; `responsibilities_total=74`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Telegram Desktop-equivalent special-config web acquisition/decryption/HTTP-date/expiry/phone-prefix/DC endpoint application is not represented by a proven canonical owner, so that lifecycle remains explicitly fail-closed; no second bootstrap runtime/provider/DC owner or `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history was invented.

Both IpPort variants, nested rule/config preservation, wrong-constructor, every-byte truncation, malformed nested constructor and trailing-byte contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary historical sent/resend/container state, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open/fail-closed.


### Round 64 final authored-contract and compliance readback

After the initial parser synchronization, canonical authored-only negative contracts were expanded at platform-core commit `022b578b8f7823a9af737a2f88fdf4c684fa4e56` to explicitly cover wrong TL vector constructors and invalid UTF-8 `phone_prefix_rules`; no contract was executed. Final canonical blobs are api `0c852a5e3bc41a4ac571d713ae3d4edea19b3619` and lib `04e5020e7b1e7ed46b403c151f84a095a1a17e4c`. Desktop final contract-sync commit `878329a3f54895da0c44ab33e14cf8856444e128` has byte-identical provider api/lib blobs.

Platform-core spec compliance review commit `d1c85e53a4cb162ef02f5344fcadf3c4a3476f6a` updates `docs/specs/telegram-mtproto-config-simple-wire.md@f7dd300a5a5f49fe6958617817f95629dcf95e8f`: AC1-AC7 are satisfied at source/implementation/bookkeeping level; AC8 remains blocked/not-run for Stage B executable validation. Special-config acquisition/decryption/expiry/phone-prefix/DC endpoint application remains fail-closed.


## Stage-A MTProto ping_delay_disconnect exact wire serialization and TLS-template blocker (round 65, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Reread of `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms that the declarations immediately after `help.configSimple` are `tlsClientHello/tlsBlock*`, followed by `ping#7abe77ec` and `ping_delay_disconnect#f3427b8c ping_id:long disconnect_delay:int`. Fixed-U `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16` shows that `tlsClientHello/tlsBlock*` are not inbound MTProto response objects: `PrepareClientHelloRules` constructs a local typed template AST and `Generator::Part::writeBlock` consumes it to generate TLS camouflage ClientHello bytes, including random blocks, X25519 public-key material, HMAC digest/timestamp injection, scoped lengths and permutations. The current canonical `native/telegram-network` owner exposes direct `TcpTransport` only and has no equivalent TLS camouflage socket/generator lifecycle, so no artificial TL parser was introduced; that lifecycle remains fail-closed/open.

The next independently representable responsibility is exact `ping_delay_disconnect` request serialization. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` serializes it on the main connected session using the current ping id and `kPingDelayDisconnect=60`, while session timers decide when to send it. Platform-core spec commit `c98a668a64a25c516065f0dda938200290712018` separates exact wire layout from those timer/session semantics. Implementation commit `cb816c7540ea93717238b3aec96372cdab1c1706` adds `PING_DELAY_DISCONNECT_CONSTRUCTOR=0xf3427b8c` and `build_ping_delay_disconnect` to the existing telegram-protocol owner. Canonical blobs are api `405b04166bea039051833a38ad0a75c62107f70c` and lib `ef7fe73335551e303fa71b02f5cf066d778ae8a4`; the builder emits exactly constructor + little-endian `i64 ping_id` + little-endian `i32 disconnect_delay`, preserves signed boundary values, and performs no timer/socket/session/request-history mutation.

Desktop implementation commit `d38f6bef11a9a4334f8387c73c7b74e7994c0cdb` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-PING-DELAY-DISCONNECT-WIRE-001`; `responsibilities_total=75`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. The Telegram Desktop main-session/timer policy for deciding when to use `ping_delay_disconnect` remains open/fail-closed rather than being invented by a wire-only builder.

Exact-layout and signed-boundary contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. TLS camouflage ClientHello generation/socket lifecycle, special-config acquisition/decryption/expiry/phone-prefix/DC endpoint application, `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.


### Round 65 source-level compliance readback

Platform-core compliance commit `6cfc81d79901493c6b34ec4e174d4fbdae81800e` updates `docs/specs/telegram-mtproto-ping-delay-disconnect-wire.md@aaa7c8b0290264b162405f35285a777fb37df91f`. Source inspection records R1-R10 and AC1-AC6 satisfied at source/implementation/bookkeeping level; AC7 remains Stage-B blocked/not-run. Remote blob readback confirms canonical and desktop provider api/lib remain byte-identical at `405b04166bea039051833a38ad0a75c62107f70c` / `ef7fe73335551e303fa71b02f5cf066d778ae8a4`. No runtime claim is made for the TLS camouflage ClientHello generator/socket lifecycle or the main-session ping timer/scheduling policy.


## Stage-A MTProto destroy_session exact request serialization (round 66, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `destroy_session#e7512126 session_id:long = DestroySessionRes`. The existing `DestroySessionRes` parser remains a separate response responsibility. Platform-core spec amendment `6f1ecac8f9284405e49dfc69af20270f914d2c83` and implementation `5f259ca296432384f5388ffef9539dbbf1f4033a` add only `DESTROY_SESSION_CONSTRUCTOR` plus exact `build_destroy_session(session_id:i64)`; canonical blobs are api `2f0c2893e256064f822082dd2fd7deae394b5763`, lib `05c690a72f5f9b5a547f10486a5289229d7bb0ff`. Desktop implementation `28d36535b4d4d342b72c6a0e35752ce56dbcfd26` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-SESSION-REQUEST-WIRE-001`; `responsibilities_total=76`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 12-byte layout, signed-boundary preservation, and request/result-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live outbound destroy-session/teardown lifecycle remains fail-closed because no precise existing owner currently represents it. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain open/fail-closed.


## Stage-A MTProto get_future_salts exact request serialization (round 67, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `get_future_salts#b921bd04 num:int = FutureSalts`; the existing `FutureSalts` parser/current-request correlation remains a separate response responsibility. Platform-core spec amendment `5a52c2a1c53efbe9ce396919598a934dc732a8d7` and implementation `58a9635f0286283218f6d8b7b2b14dd466bd4b98` add only `GET_FUTURE_SALTS_CONSTRUCTOR` plus exact `build_get_future_salts(num:i32)`; canonical blobs are api `e8026c79b8b14261ef30d39bf6de8b0bc92306a6`, lib `809eeb7aa3256e1aa259e694f413b9c0377f08d6`. Desktop implementation `dd232c52e1b39cdccb409c6d6307f1523e4d538f` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-GET-FUTURE-SALTS-REQUEST-WIRE-001`; `responsibilities_total=77`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 8-byte layout, signed-`i32` boundary preservation, and request/FutureSalts-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live `get_future_salts` scheduling and returned future-salt installation/rotation remain fail-closed because no precise existing owner currently represents those timer/session/history semantics. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain open/fail-closed.


### Round 67 source-level compliance readback

Platform-core compliance commit `eaf1eeefdde1401d820d9fecd15da3b4aa9cfde5` updates `docs/specs/telegram-mtproto-future-salts-wire.md@4b690dcafce939c13540d1ff0a55a66bed1138bb`. Source inspection records R11-R17 and AC9-AC12 satisfied at source/implementation/bookkeeping level; AC13 keeps executable validation blocked/not-run with `accepted=false`. Remote source readback confirms canonical and desktop provider api/lib remain byte-identical at `e8026c79b8b14261ef30d39bf6de8b0bc92306a6` / `809eeb7aa3256e1aa259e694f413b9c0377f08d6`. Live `get_future_salts` scheduling and future-salt installation/rotation remain explicit fail-closed scope.


## Stage-A MTProto ping exact request serialization (round 68, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `ping#7abe77ec ping_id:long = Pong`; the existing strict `Pong` parser/current-ping correlation remains a separate receive responsibility. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` selects ordinary `MTPPing` only from connection/main-session/timer state, so that scheduling/selection lifecycle is not synthesized by this wire-only slice. Platform-core spec amendment `cd40329f3fbc39c06599e3b5d4dda63f09ae6b17` and implementation `d6ef74975d3541ddb4d187bfb5641773dc01a32d` add only `PING_CONSTRUCTOR=0x7abe77ec` plus exact `build_ping(ping_id:i64)`; canonical blobs are api `714c24a5f1727e13693008ea42676831b3e8d3fd`, lib `ef44b1bb7fcac0183f1b83cca8c9d994ecc8ce9c`. Desktop implementation `36b425e547509f334453ac5ebf09033e366fe6ae` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-PING-REQUEST-WIRE-001`; `responsibilities_total=78`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 12-byte layout, signed-`i64` boundary preservation, and request/Pong-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live ordinary-ping scheduling/selection, message-id assignment, send/ACK settlement, timeout/reconnect lifecycle and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain fail-closed. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.


### Round 68 source-level compliance readback

Platform-core compliance commit `bf654c6501776fb46a8e40bad5f9aacea4612514` updates `docs/specs/telegram-mtproto-single-ping-pong.md@4faf7eb9c9a6c7b9e5988c850c9068c9a0a21d79`. Source inspection records request-side R9-R16 and AC-6 through AC-9 satisfied at source/implementation/bookkeeping level; AC-10 keeps executable validation blocked/not-run with `accepted=false`. Remote source identities are canonical/desktop api `714c24a5f1727e13693008ea42676831b3e8d3fd` and lib `ef44b1bb7fcac0183f1b83cca8c9d994ecc8ce9c`. Live ordinary-ping scheduling/selection, message-id assignment, send/ACK settlement, timeout/reconnect, arbitrary historical sent/resend/container state, TLS camouflage, gzip and complete difference decoding remain explicit fail-closed scope.


### Round 69 — rpc_drop_answer request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `rpc_drop_answer#58e4a740 req_msg_id:long = RpcDropAnswer`; reread `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `rpc_drop_answer` branch, so no historical scheduling/correlation lifecycle is inferred.

Platform-core spec-first amendment `fde4b1eadc3d03c1dd943bcb5170e3434ff2771a` and implementation `37fff2cb24658b6e0f738bbbd23cecdf075f4bb1` add only `RPC_DROP_ANSWER_CONSTRUCTOR=0x58e4a740` plus exact `build_rpc_drop_answer(req_msg_id:i64)`. Canonical blobs are api `3ec13cd2248e68f55296ed9bbaffa5c10ffce969`, lib `e4705eb4563a087e0bba811948f1244af8f9f5cb`; desktop synchronization `7c17952b90cc89d575345e192733c2e6c23deb03` copies them byte-for-byte.

Ledger adds independent request responsibility `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-REQUEST-WIRE-001` while retaining `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001` as the response-parser responsibility; `responsibilities_total=79`, `unknown=16172`, `unread=16145`, `omitted=0`. Exact 12-byte layout, signed-`i64` boundary preservation, and request/response-parser separation contracts are authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Historical outstanding-RPC selection/correlation/drop mutation and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state remain fail-closed; gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 69 source-level compliance readback

Platform-core compliance commit `510b75727303147a06b6276fa94f7dcd3a7cb31b` updates `docs/specs/telegram-mtproto-rpc-drop-answer-wire.md@f8408584b8ac997cad97d213fe90eb18155544f1`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `3ec13cd2248e68f55296ed9bbaffa5c10ffce969` / `e4705eb4563a087e0bba811948f1244af8f9f5cb`; `RPC_DROP_ANSWER_CONSTRUCTOR=0x58e4a740`, public `build_rpc_drop_answer(req_msg_id:i64)`, and the pre-existing `parse_rpc_drop_answer` response parser are all present. Request-side R9-R16 and AC7-AC9 are satisfied at source/implementation/bookkeeping level; AC10 executable validation remains blocked/not-run with `accepted=false`. Historical outstanding-RPC selection/correlation/drop mutation, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 70 — destroy_auth_key request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `destroy_auth_key#d1435160 = DestroyAuthKeyRes`. The existing shared `telegram-protocol` api owner can represent this zero-field request independently; auth-key eligibility, result handling, local key deletion, persistence and teardown/reconnect lifecycle remain fail-closed.

Platform-core spec-first commit `529f5c9147f0144b67eb077733b52d61b469f55d` and implementation `8eafd0cc7a84ab9af6b402665670b38f3ad72406` add only `DESTROY_AUTH_KEY_CONSTRUCTOR=0xd1435160` plus exact `build_destroy_auth_key()`. Canonical blobs are api `fbca9b0ebb85d0edd5c01ceea9896d0977294b53`, lib `ed581d8356375e264fabe85a2e5cccbd9b772f21`; desktop synchronization `fe0093d767ad2a12e049b4d0a474c0145d3468ab` copies them byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-REQUEST-WIRE-001`; `responsibilities_total=80`, `unknown=16172`, `unread=16145`, `omitted=0`. Exact four-byte/no-payload contract is authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Auth-key destruction lifecycle, arbitrary sent/resend/container history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 70 source-level compliance readback

Platform-core compliance commit `7643f936cd2b40aa628041880565c88539a1aace` updates `docs/specs/telegram-mtproto-destroy-auth-key-request-wire.md@ca43a6b328e68d68dfbfd7c9bae90c614918a046`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `fbca9b0ebb85d0edd5c01ceea9896d0977294b53` / `ed581d8356375e264fabe85a2e5cccbd9b772f21`; `DESTROY_AUTH_KEY_CONSTRUCTOR=0xd1435160` and public `build_destroy_auth_key()` are present. R1-R5 are satisfied at source/ownership/bookkeeping level; R6 executable validation remains blocked/not-run with `accepted=false`. Auth-key destruction eligibility/result handling/local deletion/persistence/teardown, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.

### Round 71 — req_pq request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_pq#60469778 nonce:int128 = ResPQ`. Fixed-U `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a` confirms `AbstractConnection::preparePQFake` constructs `MTPReq_pq(nonce)` before not-secure packet/message-id wrapping; reread `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no `req_pq` branch. The existing shared `telegram-protocol` owner can independently carry the exact request body; probe scheduling/racing, nonce generation, not-secure framing, response correlation and live auth-key lifecycle remain fail-closed.

Platform-core spec-first commit `c0b1b72f501822023e73f0e971a41bb0bfe38baf` and implementation `48b8dcb9fc7d6f7a79c1ccdce59041493a7a3b7a` add only `REQ_PQ_CONSTRUCTOR=0x60469778` plus exact `build_req_pq(nonce:&[u8;16])`. Canonical blobs are api `54ab072495a4f8f1eca322a7f8f46c5dcd4396ee`, lib `8df4760c6bf6dbeeba3e8d8b72232bac6a410ff0`; desktop synchronization `9e3065eddd2d0497dfb4ea2ed7a38d4ee7acb13f` copies them byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-REQ-PQ-REQUEST-WIRE-001`; `responsibilities_total=81`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 20-byte constructor-plus-raw-int128-nonce contracts, including zero/0xff nonce fixtures, are authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `req_pq_multi`, live preparePQFake scheduling/transport racing, not-secure packet/message-id construction, send/retry/timeout, response correlation, existing ResPQ/auth-key handshake integration, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.

### Round 71 source-level compliance readback

Platform-core compliance commit `893b83679268c9a2e3bd8d867419adb4b5145d26` updates `docs/specs/telegram-mtproto-req-pq-request-wire.md@b90b86e526f24665832326cc1b3ff7e95d0d32c2`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `54ab072495a4f8f1eca322a7f8f46c5dcd4396ee` / `8df4760c6bf6dbeeba3e8d8b72232bac6a410ff0`; `REQ_PQ_CONSTRUCTOR=0x60469778`, public `build_req_pq(nonce:&[u8;16])`, raw int128 serialization and both authored contracts are present. R1-R5 are satisfied at source/ownership/bookkeeping level; R6 executable validation remains blocked/not-run with `accepted=false`. Live connection-probe scheduling/transport racing, nonce generation, not-secure packet/message-id construction, send/retry/timeout, response correlation, `req_pq_multi`, live ResPQ/auth-key handshake integration, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.



### Round 72 — req_pq_multi request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_pq_multi#be7e8ef1 nonce:int128 = ResPQ`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms shipping `DcKeyCreator::pqSend` sets `Stage::WaitingPQ`, generates the attempt nonce, and passes `MTPReq_pq_multi(attempt->data.nonce)` to `sendNotSecureRequest`; `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no adjacent `req_pq_multi` branch.

Platform-core spec-first commit `c5b3021a760892eaf05fcd9b40dc35dff4c2c783` and canonical implementation commits `60e4b9165c0f41804bf68e7270aa52f39e824f14` / `591176a5ba091bb2a00f925d38ed87a1fdae0b11` add public `REQ_PQ_MULTI_CONSTRUCTOR=0xbe7e8ef1` plus pure `build_req_pq_multi(nonce:&[u8;16])` in the existing request-wire owner. Existing `AuthKeyHandshake::begin` delegates only body serialization to that builder while retaining its invalid-state guard, caller message id, `AwaitingResPq` transition, and plaintext-envelope lifecycle. Canonical blobs are api `3e3b250b709ce2f307f896dda6d1c2da55559f7b`, handshake `b1b22af0e83b577d6152beb11a75e44eb76ffdbe`, lib `2d48c2fe1c677b4b4672e7e76d8bd0a29a7ac003`, contracts `bf5aeb85f10baeeb7e645a660fd3bf26259e5989`. Desktop synchronization `59e08389bae4ab2cdcbb3f67808da44dff452ca9` makes its tracked api/lib/contracts byte-identical without introducing a second handshake lifecycle owner.

Ledger adds `TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001`; `responsibilities_total=82`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 20-byte constructor-plus-raw-int128-nonce contracts, including patterned/zero/0xff nonce fixtures and begin-path body equality, are authored only. `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection-probe scheduling/transport racing, nonce/message-id generation, plaintext/not-secure framing, send/retry/timeout, ResPQ correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.


### Round 72 source-level compliance readback

Platform-core compliance commit `492c822360c7dacedef656226386126a41ba5d91` updates `docs/specs/telegram-mtproto-req-pq-multi-request-wire.md@e9cddb463c32ef9c4483a1a7bd57c130fe260f19`. Remote readback confirms public `REQ_PQ_MULTI_CONSTRUCTOR=0xbe7e8ef1`, pure `build_req_pq_multi(nonce:&[u8;16])`, and the existing `AuthKeyHandshake::begin` body-only delegation are present; canonical api/handshake/lib/contracts blobs are `3e3b250b709ce2f307f896dda6d1c2da55559f7b` / `b1b22af0e83b577d6152beb11a75e44eb76ffdbe` / `2d48c2fe1c677b4b4672e7e76d8bd0a29a7ac003` / `bf5aeb85f10baeeb7e645a660fd3bf26259e5989`. Desktop tracked api/lib/contracts remain byte-identical and no second desktop handshake lifecycle owner was introduced.

Spec R1-R7 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; Stage-B executable evidence is intentionally absent. Connection probing/racing, nonce/message-id generation, plaintext/not-secure framing, network send/retry/timeout, ResPQ correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 73 — set_client_DH_params exact request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_DH_params#d712e4be ...` and `set_client_DH_params#f5045f1f nonce:int128 server_nonce:int128 encrypted_data:string = Set_client_DH_params_answer`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms shipping `pqAnswered` sends `MTPReq_DH_params` and `dhClientParamsSend` sends `MTPSet_client_DH_params` after moving to `Stage::WaitingDone`; `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` contains no direct constructor call.

Source review confirms existing canonical `build_req_dh_params` already serializes constructor, nonce, server_nonce, p, q, fingerprint and encrypted_data in fixed-U order, with an authored contract that consumes every field and requires no trailing bytes; that responsibility was left unchanged. `set_client_DH_params` was still inline in `prepare_client_dh`, so platform-core spec-first commit `f3d85c368ace17c864eebf2b4d97a44c9d3d964f` and implementation `a353cdf718a93f8e51f48e58c1be4a9889c3bc32` add pure `build_set_client_dh_params` and delegate only final request-body framing from the existing DH preparation path. Canonical blobs are handshake `2e9acc80d10a87a8633173c88568aef706d4e123`, lib `3a05251d749f0df868e4ebd94fb7e28e31fad8f1`, contracts `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`. Desktop synchronization `85db8d62f8014614234ab2340560367d087ac4e0` mirrors lib/contracts byte-for-byte without introducing a second handshake lifecycle owner.

Ledger adds `TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001`; `responsibilities_total=83`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact constructor/raw-nonce/TL-string-padding and prepare-path body-equality contracts are authored only. `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection probing/racing, message-id generation, plaintext/not-secure framing, send/retry/timeout, server response correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.


### Round 73 source-level compliance readback

Platform-core compliance commit `dd584145d268084545e082c399e9f5fe2e52cba9` updates `docs/specs/telegram-mtproto-set-client-dh-params-request-wire.md@7ec57e2cfa47f62f0ae372e776ec8a1dcb39c04d`. Remote readback confirms canonical `build_set_client_dh_params`, its public facade export, and `prepare_client_dh` body-only delegation are present; canonical handshake/lib/contracts blobs are `2e9acc80d10a87a8633173c88568aef706d4e123` / `3a05251d749f0df868e4ebd94fb7e28e31fad8f1` / `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`. Desktop tracked lib/contracts are byte-identical. Existing `build_req_dh_params` and its complete field-order/trailing contract remain present and unchanged.

Spec R1-R8 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; `responsibilities_total=83`, `unknown=16172`, `unread=16145`, `omitted=0`. Stage-B executable evidence is intentionally absent.

Connection probing/racing, message-id generation, plaintext/not-secure framing, network send/retry/timeout, server response correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 74 — client_DH_inner_data exact wire serializer; temp_dc remains fail-closed

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `client_DH_inner_data#6643b654 nonce:int128 server_nonce:int128 retry_id:long g_b:string = Client_DH_Inner_Data` and adjacent `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` constructs `MTP_client_DH_inner_data` inside shipping `dhClientParamsSend`; its temporary-key branch chooses `MTP_p_q_inner_data_temp_dc` only when `attempt->expiresIn != 0`. `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct construction of either inner object.

Existing canonical `prepare_client_dh` already owned the full DH/auth-key preparation lifecycle but framed `client_DH_inner_data` inline. Platform-core spec-first commit `20049ba38728a50239a900a1c0a684527d427b30` and implementation `b5e5f9cd85ed663b0c2ab35998bc1fbf116df0e9` add pure `build_client_dh_inner_data`, preserving exact constructor/raw nonce/server_nonce/little-endian retry_id/canonical TL bytes framing, and delegate only that inner-object serialization from `prepare_client_dh`. Canonical blobs are handshake `0e5e6c0fa306f75aa1fcb5c26440b1c39f87c9f9`, lib `7273e030bf02c248d9b592b2578cb5dc9716b485`, contracts `7236df6bec3c832363901034d6a7d1c7cd3df3fc`. Desktop synchronization commit `1de1da4bf045c405d26ff44c373ad2dd63bf9ce1` mirrors lib/contracts byte-for-byte and introduces no second handshake owner.

`p_q_inner_data_temp_dc` is deliberately not marked implemented: the canonical `AuthKeyHandshake::prepare_req_dh_params*` currently accepts `dc_id` but has no temporary-key `expires_in` input/state. Adding a dead standalone serializer would not close the shipping temporary-key lifecycle. Ledger adds `TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001`; `responsibilities_total=84`, `unknown=16172`, `unread=16145`, `omitted=0`. New responsibility is `implemented / validation_status=not-run / accepted=false`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection probing/racing, message-id generation, plaintext/not-secure framing, send/retry/timeout, server response correlation/application, temporary-key p_q_inner_data_temp_dc lifecycle, broader auth-key persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.


## Round 74 source-level compliance readback

Platform-core compliance commit `e1b2e5fc59e46b198c07c64845810a73a62b04b4` updates `docs/specs/telegram-mtproto-client-dh-inner-data-wire.md@dc09433a720daa1af5fa7eca4dc641cc542e52cc`. Remote readback confirms canonical `build_client_dh_inner_data`, its public facade export, and `prepare_client_dh` inner-object-only delegation are present; canonical handshake/lib/contracts blobs are `0e5e6c0fa306f75aa1fcb5c26440b1c39f87c9f9` / `7273e030bf02c248d9b592b2578cb5dc9716b485` / `7236df6bec3c832363901034d6a7d1c7cd3df3fc`. Desktop tracked lib/contracts are byte-identical.

Spec R1-R8 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; `responsibilities_total=84`, `unknown=16172`, `unread=16145`, `omitted=0`. Adjacent `p_q_inner_data_temp_dc#56fddf88` remains explicitly fail-closed because current canonical handshake lifecycle does not represent temporary-key `expires_in`; no dead serializer is counted as production closure. Stage-B executable evidence is intentionally absent.

Temporary-key p_q_inner_data_temp_dc lifecycle, connection probing/racing, message-id generation, plaintext/not-secure framing, network send/retry/timeout, response correlation/application, broader auth-key persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 75 — p_q_inner_data_temp_dc exact wire + temporary request preparation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. fixed-U `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int`; `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` records `Attempt::expiresIn` and consumes it only when selecting/serializing the temporary p_q inner object before Req_DH_params. Existing canonical `AuthKeyHandshake` can therefore precisely own the wire/request-preparation slice without a second state machine.

Spec-first commits `93283c35d075a16498b672e548071e16bdafd2c3` and boundary correction `8f09112d7af1875e19eddb7096ef1a0a05347d8d` precede implementation. Canonical commits `af7b37f57be70a88cd10728d79d352c7aade88f5` / `68c20daadfc680e24393c63d0505feb5fee4783e` add `P_Q_INNER_DATA_TEMP_DC_CONSTRUCTOR`, pure `build_p_q_inner_data_temp_dc`, positive lifetime validation, and `prepare_temporary_req_dh_params*` reusing the existing ResPqReceived -> AwaitingServerDh preparation path. Contract commit `bcd516892fa49407a4de757e05235ed585ad4338` is authored only. Desktop commits `aafca841fd745f3aee2deb9b89921f6d34021361` / `bd7c3a1c648107c92a265c76c46bae1da745f7e9` mirror facade/contracts byte-for-byte.

The main telegram-network/runtime establishment remains persistent: current persisted transport state has no temporary-key expiry/binding metadata, so routing a temporary key there would misrepresent lifetime semantics. That broader lifecycle is explicitly fail-closed rather than silently persisted. Ledger adds `TDRP-R9-MTPROTO-P-Q-INNER-DATA-TEMP-DC-WIRE-001`; `responsibilities_total=85`, `unknown=16172`, `unread=16145`, `omitted=0`; new responsibility is `implemented / validation_status=not-run / accepted=false`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. Broader temporary auth-key expiry/binding/persistence, connection scheduling/racing, plaintext/send lifecycle, multi-request history, gzip, complete difference decoding, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.

### Round 76 — DestroyAuthKeyRes exact typed parser + adjacent p_q disposition

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Source readback confirms `DcKeyCreator` requires `protocolDcId` and its shipping pq-answer path selects only `p_q_inner_data_dc#a9f55f95` (persistent) or `p_q_inner_data_temp_dc#56fddf88` (temporary). The adjacent no-dc `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` therefore have no independent fixed-U shipping responsibility in this path and no duplicate builders were added. `bind_auth_key_inner#75a3f765` remains fail-closed because temporary/permanent binding, expiry and session lifecycle require broader state ownership.

The next independent shipping auth-key wire responsibility is `DestroyAuthKeyRes`: fixed-U `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087` sends `destroy_auth_key#d1435160`, parses the response union, and distinguishes `destroy_auth_key_ok#f660e1d4`, `destroy_auth_key_none#0a9f2259`, and `destroy_auth_key_fail#ea109b13`. Spec-first platform commit `791308b5c885b15b4f69b395ae0df0a75f47f634` defines the wire-only boundary. Canonical implementation commit `ebb1e5f859e224c45f9d34beb1aa32b4c18ca4b2` adds the three constructors, `DestroyAuthKeyResult`, strict `parse_destroy_auth_key_result`, facade exports and authored-not-run contracts. Canonical api/lib/contracts blobs are `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`; desktop sync commit `4b152da131b17558fcad7c1aee45fd0cec3269e6` mirrors those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001`; `responsibilities_total=86`, `unknown=16172`, `unread=16145`, `omitted=0`. The new responsibility is `implemented / validation_status=not-run / accepted=false`. Destruction eligibility, local key removal, persistence mutation, transport/session teardown/reconnect, temporary/permanent binding and renewal remain fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

## Round 76 final source-level readback

Platform-core compliance commit `42c7ac4d193f7f1872a9bfd87ce70b2af973c2df` updates `docs/specs/telegram-mtproto-destroy-auth-key-result-wire.md` to blob `da653d871e78a8a97770449034d0ac4db2f71cfe` and records R1-R8 source-level compliance. Canonical api/lib/contracts remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`; desktop tracked provider files are byte-identical.

The ledger responsibility `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=86`, `unknown=16172`, `unread=16145`, `omitted=0`. Adjacent `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` remain dispositioned as non-shipping in fixed-U `DcKeyCreator`; `bind_auth_key_inner#75a3f765` and broader auth-key destruction/binding/persistence/teardown lifecycle remain fail-closed.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.



### Round 77 — Server_DH_Params exact constructor-first union parsing

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping `DcKeyCreator::dhParamsAnswered` consumes fixed-U `server_DH_params_ok#d0e8075c` / `server_DH_params_fail#79cb045d` through the existing canonical `AuthKeyHandshake::receive_server_dh`. The existing parser already owned the wire responsibility, but read nonce/server_nonce before constructor discrimination, so a complete four-byte unknown constructor was misclassified as TL truncation.

Spec-first platform commit `9a6b687e043bae9f7f40e776ce1d5ec98220a168` defines the strict wire-only boundary. Canonical implementation commit `274bdb92bc7e4d5755bb87fd4996398ee6cf768b` hardens `parse_server_dh_params` to reject unknown constructors before field reads while preserving exact nonce binding, OK TL-bytes / FAIL int128 payload decoding and no-trailing validation. Canonical handshake/contracts blobs are `ed9eda7732f4753b442de98ebf510b8a7558b823` / `a19c4a14d113e6c82fe3cc801f1764c555f26f76`. Desktop sync commit `000d38bd2db19052766ccf8062eea4a6fb93c7eb` mirrors the contracts byte-for-byte; the tracked desktop facade remains `abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate desktop `handshake.rs` is introduced.

Ledger adds `TDRP-R9-MTPROTO-SERVER-DH-PARAMS-WIRE-001`; coverage becomes `responsibilities_total=87`, `unknown=16172`, `unread=16145`, `omitted=0`; the new responsibility is `implemented / validation_status=not-run / accepted=false`. Broader auth-key destruction/binding/persistence/teardown, connection scheduling/racing/send lifecycle, multi-request history, gzip, complete difference decoding/application, broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 77 final source-level compliance

Platform-core `8654e737a04900b6cac8c92660fa8a8b84b449bc` / spec blob `b9b0f486aa47071ad46ba4ff5c238cee4b2bbfce` records source-level R1-R9 and AC-1-AC-6 compliance for the constructor-first fixed-U `Server_DH_Params` parser. Canonical handshake/contracts are `ed9eda7732f4753b442de98ebf510b8a7558b823` / `a19c4a14d113e6c82fe3cc801f1764c555f26f76`; desktop contracts remain byte-identical and facade `abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. Ledger stays `implemented / not-run / accepted=false`, coverage `87 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 78 — dh_gen_ok/retry/fail exact result/hash validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::dhClientParamsAnswered` consumes fixed-U `dh_gen_ok#3bcbf734`,
`dh_gen_retry#46dc1fb9` and `dh_gen_fail#a69dae02`. Existing
`AuthKeyHandshake::receive_dh_gen / parse_dh_gen_result` already owned nonce binding,
tag-specific `new_nonce_hash{1,2,3}` authentication, strict trailing validation, OK completion,
handshake-local RETRY and FAIL rejection. The remaining exact-wire gap was discriminator ordering:
nonce/server_nonce/hash were read before constructor matching, so a complete four-byte unknown constructor
was misclassified as TL truncation.

Spec-first platform commit `0c16a7c83a7e511484f5099399acb4c308aecb6c`
(`docs/specs/telegram-mtproto-dh-gen-result-wire.md@a43f4db8123abfcd2334fe49079b413e40c58ea6`)
defines the wire-only boundary. Canonical implementation commit
`5a5c6095d3e049de5d187b89c4942d7f3377ae00` moves three-way constructor discrimination before
all common fields in `handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6`.
Canonical authored-contract commit `39fce63bce58718727541ac9f18dbc258766d159` expands the existing harness to all
OK/RETRY/FAIL variants, every truncation cut, four-byte unknown constructor, nonce/server_nonce mismatch,
wrong variant-tag hash and trailing bytes; contract blob is
`6710d6ef3fd915689792c68d69b3922b446c9bfa`. Desktop sync
`965db14afa7104deae0fc63a76d2cc0df35d81fd` mirrors that blob byte-for-byte; the tracked desktop
facade remains `abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate `handshake.rs` is introduced.

The existing RETRY branch is precisely representable by the current `AwaitingDhGen` owner: it uses the
previous auth-key auxiliary hash as `retry_id`, replaces only prepared client-DH material, and remains in
`AwaitingDhGen`. It does not require or claim temporary/permanent key binding, expiry, persistence,
teardown, connection scheduling, multi-request history, gzip, or complete difference decoding/application.

Ledger adds `TDRP-R9-MTPROTO-DH-GEN-RESULT-WIRE-001`; coverage becomes
`responsibilities_total=88`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck,
lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or
runtime acceptance was performed.


### Round 78 final source-level compliance

Platform-core `3a08f54f8a49c8a8af7a23b252938d1ea4607954` / spec blob
`d846673483e2b165d7fad7463be0d7720ee6a49b` records source-level R1-R9 and AC-1-AC-6 compliance
for the constructor-first fixed-U `dh_gen_ok/retry/fail` parser/hash responsibility.
Canonical handshake/contracts are `87616f4b96a1775455da42b0bbed77711fb3c9f6` /
`6710d6ef3fd915689792c68d69b3922b446c9bfa`; desktop contracts remain byte-identical and facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. The existing RETRY path stays inside
`AwaitingDhGen` and does not claim binding/persistence or connection scheduling. Ledger remains
`implemented / not-run / accepted=false`, coverage `88 / 16172 unknown / 16145 unread / 0 omitted`.
No executable validation was performed.


### Round 79 — resPQ exact constructor/nonce/server_nonce/pq/fingerprint-vector validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::pqAnswered` consumes `resPQ#05162463 nonce:int128 server_nonce:int128 pq:string server_public_key_fingerprints:Vector<long>`.
Remote source review confirms the existing canonical `AuthKeyHandshake::receive_res_pq / parse_res_pq` already owns and exactly implements
this fixed-U wire responsibility: constructor-first discrimination, exact nonce/server_nonce/pq/vector field order, bounded TL Vector
constructor/count validation, strict EOF, active request nonce binding, empty-pq rejection and empty-fingerprint rejection before state mutation.
The server nonce is fresh server-provided handshake material at this phase and is captured only after the response passes validation.

Because production behavior was already exact, no production `handshake.rs` delta was manufactured. Spec-first platform commit
`03dc0f804a8c80d8d60500e6667cc9b091ccc3cf`
(`docs/specs/telegram-mtproto-res-pq-wire.md@ab044dcb2b34d7def042e4dd2eb798f3b658c52a`) records that disposition.
Canonical authored-contract commit `5e1ba90b32ac7c58b7be0d9630f1630a0a764061` extends the existing protocol harness with exact
positive resPQ, every truncation cut, four-byte unknown constructor, wrong Vector constructor, negative/over-limit/short declared vector,
trailing byte, active nonce mismatch, empty pq, empty fingerprints and invalid receiver-state cases; canonical contract blob is
`44d2a5d141e7d7854529379aaaad660853c8cbe3`. Desktop provider sync commit
`c72c7f85c926d05b21842c41a3b11c1313537793` mirrors that contract blob byte-for-byte; the tracked desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate handshake core is introduced.

Ledger adds `TDRP-R9-MTPROTO-RES-PQ-WIRE-001`; coverage becomes
`responsibilities_total=89`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Temporary/permanent key binding/expiry/persistence/teardown,
connection scheduling/racing/send lifecycle, arbitrary multi-request history, gzip, complete difference decoding/application,
broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 79 final source-level compliance

Platform-core `54f1a26231827ac11a1ea0544dc4d1f7c3762c6c` / spec blob
`404bbe591fb514aba732e5b4e8f6481cc2904618` records source-level R1-R9 and AC-1-AC-5 compliance for the
fixed-U `resPQ#05162463` wire responsibility. Canonical production handshake remains
`87616f4b96a1775455da42b0bbed77711fb3c9f6` with no behavior delta; canonical/desktop authored contract blob is
`44d2a5d141e7d7854529379aaaad660853c8cbe3`, and desktop facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. Ledger remains
`implemented / validation_status=not-run / accepted=false`, coverage
`89 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 80 — server_DH_inner_data exact authenticated wire validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::dhParamsAnswered` consumes fixed-U
`server_DH_inner_data#b5890dba nonce:int128 server_nonce:int128 g:int dh_prime:string g_a:string server_time:int`.
Remote source review confirms the existing canonical
`AuthKeyHandshake::receive_server_dh -> decrypt_server_dh_inner_data` already owns and exactly implements this
wire/decryption responsibility: non-empty AES-IGE block framing, temporary key/IV derivation from active
new_nonce/server_nonce, constructor-first exact field decode, active nonce/server_nonce binding, SHA-1 authentication
over exactly the consumed TL object, residual AES alignment padding bounded to 0..15 bytes, signed server_time
preservation, and existing safe-prime/generator/public-value checks before client-DH preparation.

Because production behavior is already exact, no `handshake.rs` behavior delta was manufactured. Spec-first
platform commit `3bdde5a158373e3392cac987c8846d16a6fb3162`
(`docs/specs/telegram-mtproto-server-dh-inner-data-wire.md@5174248bd16e0d125abca66b389d0e49bc2a3f05`)
records the no-production-delta disposition. Canonical authored-contract commit
`d724e29265e53a7ee7efd821c8bb37f6e4994cb2` expands the existing harness with exact signed server_time,
empty/non-block ciphertext, unknown constructor, every serialized-inner truncation cut, nonce/server_nonce mismatch,
SHA-1 mismatch, >15 residual padding, invalid generator, dh_prime length/trust, empty/oversized/unsafe g_a and
rejected-inner state-continuity cases; canonical contract blob is
`256f421f54c41256646388833b634cf1191afd91`. Desktop sync
`4323b97d3b6041731d5b547118fb746d2eba37a6` mirrors that blob byte-for-byte; desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate handshake core is introduced.

Ledger adds `TDRP-R9-MTPROTO-SERVER-DH-INNER-DATA-WIRE-001`; coverage becomes
`responsibilities_total=90`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Auth-key binding/expiry/persistence/teardown,
connection scheduling/racing/send lifecycle, arbitrary multi-request history, gzip, complete difference
decoding/application, broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 80 final source-level compliance

Platform-core `26cc521c1e57944a9f27fa16f469fff870f34517` / spec blob
`1290cb76c59c01343ac7903a83113b28a90b4147` records source-level R1-R13 and AC-1-AC-6 compliance for the
fixed-U `server_DH_inner_data#b5890dba` responsibility. Canonical production handshake remains
`87616f4b96a1775455da42b0bbed77711fb3c9f6` with no behavior delta; canonical/desktop authored contract blob is
`256f421f54c41256646388833b634cf1191afd91`, and desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf`. Ledger remains
`implemented / validation_status=not-run / accepted=false`, coverage
`90 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 81 — msgs_state_req exact request serializer/vector boundary

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`SessionPrivate::tryToSend` path serializes fixed-U
`msgs_state_req#da69fb52 msg_ids:Vector<long>` from non-empty `_stateRequestData`, preserving id order in
`MTP_vector<MTPlong>`. Existing Fabushi `build_msgs_state_req` already owns the independent request-wire
responsibility and is used by the previously implemented single-current-RPC state-reconciliation path.

Remote source review confirms canonical production
`native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`,
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` and
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` already serialize constructor `0xda69fb52`,
TL Vector constructor `0x1cb5c415`, signed i32 count and each signed long as little-endian i64 in caller
order; vector-count conversion fails closed before a partial Vector header can be emitted. Empty Vector remains
schema-valid at the pure builder boundary, while the shipping caller independently sends only non-empty state requests.

Because production behavior is already exact, no production delta was manufactured. Spec-first platform commit
`579b575974b2031e50ffea4910acce8a67289a64`
(`docs/specs/telegram-mtproto-msgs-state-req-request-wire.md@dfa2b21a82d9b4d9f92da32b4cedcc176bab0c3f`)
records the no-production-delta disposition. Canonical authored-contract commit
`97e04d8ee1556c28cd6227f809816f04bcf1f674` adds empty, one-id shipping, multiple signed-long
order/bit-preservation, exact EOF and `i32::MAX+1` vector-overflow/no-partial-header cases; contract blob is
`60a1dfb3ea2051a498d6cfb9d401fe653fde7d75`. Desktop provider sync
`23f023f57cc9c6bc50e519fe89f01cc78a3511d9` mirrors that contract blob byte-for-byte while production
`api.rs` remains `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` and facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged.

Ledger/evidence commit `2b5267d726385e6bebbd9d68555f1ebe607b4a84` adds
`TDRP-R9-MTPROTO-MSGS-STATE-REQ-REQUEST-WIRE-001`; coverage becomes
`responsibilities_total=91`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Arbitrary multi-request
`haveSent/_resendingIds/_stateAndResendRequests` history, simultaneous state queries, connection scheduling,
auth-key binding/persistence, gzip, complete difference decoding/application, broader source/legal closure and
release acceptance remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 81 final source-level compliance

Platform-core `f2593977cbdfe1a09ce811da86029ec824021af3` / final spec blob
`f9b8839b10fa449474a5cfe080f84ca70a08f7a1` records source-level R1-R9 and AC-1-AC-6 compliance for the
fixed-U `msgs_state_req#da69fb52 msg_ids:Vector<long>` request-wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `60a1dfb3ea2051a498d6cfb9d401fe653fde7d75`. Ledger compliance commit
`cba0acd6b39a75d6c605ec0549c372c23c77ffa2` records the responsibility as
`implemented / validation_status=not-run / accepted=false`, coverage
`91 / 16172 unknown / 16145 unread / 0 omitted`. Arbitrary multi-request history and the other deferred
lifecycle/gzip/difference responsibilities remain open/fail-closed. No executable validation was performed.


### Round 82 — msg_resend_req exact request serializer/vector boundary

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`msg_resend_req#7d861a08 msg_ids:Vector<long>` is shipping-reachable in
`SessionPrivate::tryToSend`: a non-empty `_resendRequestData` is moved into
`MTP_vector<MTPlong>` and serialized as `MTP_msg_resend_req`. The adjacent detailed-info receive paths enqueue
an answer id into that vector when the answer was not received.

Remote source review confirms the existing canonical production owner already closes the independent request-wire
boundary with no behavior delta:
`native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines
`MSG_RESEND_REQ_CONSTRUCTOR = 0x7d861a08` and `build_msg_resend_req`;
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` writes TL Vector constructor
`0x1cb5c415`, converts the element count to signed i32 before writing the Vector header, and preserves each signed
long as little-endian i64; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same builder.

Spec-first platform commit `953cd194143ae4cea4b8e842cb6d596b33451b19`
(`docs/specs/telegram-mtproto-msg-resend-req-request-wire.md@56facfa5ba3db5999a42cdcb742086f2c8ae690a`)
records the no-production-delta disposition. Canonical authored-only contract commit
`161d78f68715540e733c43895aba2f1be4619755` adds empty, one shipping-shaped id, multiple signed-long
order/bit-preservation, strict EOF and `i32::MAX+1` vector-overflow/no-partial-header cases; canonical contract blob
is `89692a28620b2d8465703a1335458d5fbd15aee6`. Desktop provider sync
`37299e18e3018e988d541607d5ba7842713d9fd1` mirrors that blob byte-for-byte while desktop production
`api.rs` remains `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` and facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf`.

Ledger/evidence commit `5cd36cdd14e7321b0d96526f334b1f97e4243510` adds
`TDRP-R9-MTPROTO-MSG-RESEND-REQ-REQUEST-WIRE-001`; coverage is
`responsibilities_total=92`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`.

This slice deliberately does not claim Telegram Desktop arbitrary
`haveSent/_resendingIds/toSend/_sentContainers` histories, callback ownership, resend scheduling, container retry,
connection scheduling/racing/send retry, auth-key binding/persistence, gzip, or complete difference decoding/application.
Those remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 82 final source-level compliance

Platform-core `be950b18eca7e3e6c1e9ffb4833a19fcbd224880` / final spec blob
`e85d67b0b3b8c7c3d7bee7c08d72a9978903c325` records source-level R1-R9 and AC-1-AC-6 compliance for the
fixed-U `msg_resend_req#7d861a08 msg_ids:Vector<long>` request-wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `89692a28620b2d8465703a1335458d5fbd15aee6`. Desktop compliance ledger/evidence commit
`19e0d2060b64f78e3a44cbb8cb96c4f46f9ca312` records the responsibility as
`implemented / validation_status=not-run / accepted=false`, coverage
`92 / 16172 unknown / 16145 unread / 0 omitted`.

Arbitrary multi-request `haveSent/_resendingIds/toSend/_sentContainers` history, resend scheduling/cancellation,
connection scheduling/racing/send retry, auth-key binding/persistence, gzip, complete difference decoding/application,
broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.
No executable validation was performed.


### Round 83 — rpc_error exact typed parser/trailing validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`rpc_error#2144ca19 error_code:int error_message:string` is shipping-reachable as the body of
`rpc_result` in `SessionPrivate::handle`. Wider request-history lookup, temporary-key handling and
`gzip_packed` are deliberately outside this isolated wire slice.

Remote source review confirms the existing canonical production owner already closes the independent parser boundary:
`api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines `RPC_ERROR_CONSTRUCTOR`,
`RpcErrorResponse` and `try_parse_rpc_error`; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
owns TL string padding/UTF-8/truncation; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser.
The parser rejects trailing bytes, so production behavior requires no delta.

Spec blob is `c219f35d393bb94e29c30f579ec501bb3d42c9d5`; canonical authored contract commit
`7ad951b93b57fc3c4cf33e43eaa157066d452b31` and desktop sync
`885faac55fc8b7d99e4989b438e1a428ac57c1f8` share contract blob
`d171a3ab9a43955a693c87162f80c2667fc397ce`. Authored cases cover exact signed code/message,
non-RpcError discrimination, truncation, invalid UTF-8, trailing bytes and exact EOF; they were not run.

The responsibility is recorded as `implemented / validation_status=not-run / accepted=false`.
Arbitrary multi-request history, connection scheduling/racing/send retry, auth-key binding/persistence, gzip,
complete difference/differenceSlice decoding/application, broader source/legal closure and Stage-B executable evidence
remain open/fail-closed. No executable validation was performed.


### Round 83 final source-level compliance

Platform-core `e448cb7307ded83d9c6eab34fff0ef31f48e0e79` / final spec blob
`89d572a9ec403835f4e1ec9d538dc3fbbe86a32d` records source-level R1-R10 and AC-1-AC-6 compliance for the
fixed-U `rpc_error#2144ca19 error_code:int error_message:string` response-wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `d171a3ab9a43955a693c87162f80c2667fc397ce`. The desktop ledger records
`TDRP-R9-MTPROTO-RPC-ERROR-WIRE-001` as `implemented / validation_status=not-run / accepted=false`, coverage
`93 / 16172 unknown / 16145 unread / 0 omitted`.

Arbitrary request history, connection scheduling/racing/send retry, temporary/permanent auth-key
binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice decoding/application, broader fixed-U
source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.
No executable validation was performed.


### Round 84 — msgs_state_info exact typed parser / binary TL string

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`msgs_state_info#04deb57d req_msg_id:long info:string` is shipping-reachable in
`SessionPrivate::handle`, where `MTPMsgsStateInfo::read(from,end)` performs typed wire decoding before
`_stateAndResendRequests` lookup and `handleMsgsStates` application.

Remote source review confirms the existing canonical `telegram-protocol` owner already closes the independent
wire boundary with no behavior delta:
`api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines constructor `0x04deb57d`,
`MsgsStateInfo` and `parse_msgs_state_info`; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
owns TL bytes/string short/long framing, length limits, padding and truncation; and
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser. The fixed-U `info` value is
binary state data, so preserving it through `read_bytes()` rather than UTF-8 `read_string()` is intentional.

Spec-first platform commit `41fc75c372bdb818c21c21af6fcc60958bb7974b` /
spec blob `d90260a94d2bc04f2d487614cc4806b819e126ee` records the no-production-delta disposition.
Canonical authored-only contract commit `ae5b7a6f6b6669fe9069d7371b1b4a2acd76e182` and desktop sync
`ae630cf55cb611f9e400b8c5a4bbbbad20654387` share contract blob
`efaa8ac407af5bca430855fbb69c862f9ce793b0`. Authored cases cover signed id preservation,
binary non-UTF8 short-form state, empty and 254-byte long-form state, wrong constructor, truncation,
missing padding, trailing bytes and exact EOF; they were not run.

Ledger adds `TDRP-R9-MTPROTO-MSGS-STATE-INFO-WIRE-001`; coverage becomes
`94 / 16172 unknown / 16145 unread / 0 omitted`, and the responsibility is
`implemented / validation_status=not-run / accepted=false`.

Arbitrary request history/application, connection scheduling/racing/send retry, auth-key binding/persistence,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure and Stage-B executable
evidence remain open/fail-closed. No executable validation was performed.


### Round 84 final source-level compliance

Platform-core `e99655ad3c06858e60fb7a1bf6a5a12cb1a0b2ff` / final spec blob
`4780f4764ae68cbb921177214a05e215b98608fa` records source-level R1-R10 and AC-1-AC-6 compliance for the
fixed-U `msgs_state_info#04deb57d req_msg_id:long info:string` wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `efaa8ac407af5bca430855fbb69c862f9ce793b0`. The desktop ledger records
`TDRP-R9-MTPROTO-MSGS-STATE-INFO-WIRE-001` as
`implemented / validation_status=not-run / accepted=false`, coverage
`94 / 16172 unknown / 16145 unread / 0 omitted`.

Binary `info` preservation is intentional; arbitrary request history/application, connection scheduling/racing/send
retry, temporary/permanent auth-key binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice
decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT
remain open/fail-closed. No executable validation was performed.
