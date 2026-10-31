# Round 30 — fixed-U non-code provenance closure

Date: 2026-10-31  
Stage: A (manual Git/API/source evidence only; no CI/build/test/schema/lint/generator/package/deploy/install execution)  
Checkpoint before write: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1@9a676705570e7feb554513684db14b8728c9a1a0`  
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4` (tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`)

## Build-time acquisitions

The current build-time inventory references 28 authority source paths. Comparing those exact paths from historical `351204f3695b4a39bc3f4d2f230c061b9a01fe2a` to fixed U shows 24 byte-identical blobs and four changed blobs. Source-line review preserved the registered dispositions and exposed the fixed-U pin advances recorded in `inventory/build-time-acquisitions.json`: desktop-app/patches, qt6_windows7_patches, tg_angle, and the CI-only docker/login-action pin. The four GitHub commit objects were independently fetched at their exact full SHA.

## LFS and root external-object check

Fixed U contains two `.gitattributes` files and neither declares an LFS filter. The fixed root has 83 blobs in the 90–220 byte window that covers the standard Git LFS pointer form. Every candidate was read as UTF-8 or rejected as native non-UTF8 binary; zero UTF-8 candidates began with `version https://git-lfs.github.com/spec/v1`. This is root-tree evidence only and does not close recursive gitlink/external-object provenance.

## Resources/locales/assets and generated inputs

The fixed root contains 3134 `Telegram/Resources` blobs. A regenerated TSV binds every resource path to its Git blob SHA and size. Current CMake/source reads additionally bind language, TL scheme, Windows MIDL, update-key, model baking, QRhi shader, Linux DBus, QRC and AppStream generation inputs without executing any generator. The external DBus input is independently bound to `flatpak/xdg-desktop-portal@23a76c392170dbbd26230f85ef56c3a57e52b857:data/org.freedesktop.portal.Background.xml` blob `1b83fcd1f65ac3e34ab6f869fd3bee8244cb9c3a`.

## Packaging/updater/tests/tooling/docs/licenses

The machine inventory records fixed-U root identities for `Telegram/build`, `Telegram/cmake`, Snap, workflows, updater-related paths, test/test-fixture paths, agent/tooling paths and root docs/legal authorities. `LEGAL` is blob `09c9e383db122fe0694e73c48d6224676afe60f6`; `LICENSE` is `d70f4d6a87eece19c4ec184de36a8aba72b4a639`; `README.md` is `15a6194f73ae6084a2ab25d14005dc10262cdfb3`. This does not claim Fabushi dependency/license acceptance: copied/derived content and actually adopted distributed dependencies remain subject to explicit release review.

## Blockers and gates

The cppgir parent declaration remains `desktop-app/cmake_helpers@7ceca3019b4750d7a5acb2de0faffd523ee14095 -> gitlab.com/mnauw/cppgir@47cf94f83b54cda59018135601e19d7fb0c77776`. No new original-host cppgir tree/blob evidence was obtained in this round, so `git_tree_verified=false` and fail-closed status are preserved. `baseline_ready=false`, `accepted=false`, and validation remains not-run.
