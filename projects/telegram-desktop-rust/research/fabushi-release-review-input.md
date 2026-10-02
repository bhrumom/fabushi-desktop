# Fabushi release-review input from the reachable build graph

Status: active P0 evidence; release/legal acceptance remains open  
Project: FBCP-001 / TDRP-001  
Source-review baseline: PR #26 parent `4552932bab6880223872f61a56f88f4f43984545` after absorbing PR #20 `bf36916c80e68737f02217ae85be4d22a6a5f928`  
Main observed: `d9ae2773f2c517a0cb911e7b7bc996905cf3ada4`  
Prepared on: 2026-10-02

## Purpose

This dossier is the release-review input requested by P0. It works backwards from the files and commands that can actually enter a Fabushi desktop package. It does not treat Telegram Desktop's research dependency graph as a Fabushi shipping graph.

The review rule is:

`distributed artifact -> packaging input -> build input -> immutable dependency/resource identity -> license/origin evidence`.

Anything not reachable from the Fabushi package path is research provenance only. Anything reachable but not repository-immutable remains a release blocker rather than being inferred from Telegram or from a later observation.

## 1. Reachable desktop package graph

The canonical desktop package is defined by `desktop/package.json`.

The package command is:

`npm run build:host && npm run build:electron && npm run build:renderer && npm run check:desktop-runtime && electron-builder`.

Electron Builder includes:

- `desktop/dist/**`;
- `desktop/package.json`;
- staged `desktop/resources/bin`;
- staged `desktop/resources/asr`;
- staged `desktop/resources/computer-control`;
- platform metadata and `desktop/resources/icon.png`.

The tracked `desktop/resources` tree at this baseline contains only:

- `desktop/resources/icon.png`;
- macOS entitlements;
- Mac App Store entitlements.

Therefore `bin`, `asr`, and `computer-control` are build/staging products, not pre-existing tracked payload trees at this exact source baseline.

## 2. JavaScript dependency identity

The release workflow installs with:

`npm --prefix desktop ci --prefer-offline --no-audit --no-fund`.

The lockfile is:

- path: `desktop/package-lock.json`
- Git blob: `2f2004c299adc24a7aecf005a3a62897b5b5fc5d`
- lockfile format: v3

The lock records concrete registry tarball URLs, integrity hashes, versions, and package license metadata for npm resolution. This is materially stronger than a floating package-manager install and is the authoritative JavaScript dependency input for release review.

Open review item: produce the final distributed npm-license inventory from this exact lockfile at the release candidate SHA and reconcile it with Electron Builder's actual packaged dependency set. The lockfile itself is provenance input, not a legal compatibility verdict.

## 3. Shipping Rust binary graph

`desktop/scripts/stage-host.mjs` stages three Rust executables into `desktop/resources/bin`:

- `mahayana-app-host`, built from `source/host/app/Cargo.toml`;
- the Node Agent Coordinator Rust binary, built from `source/node-agent-coordinator/Cargo.toml`;
- `box-exec-daemon`, built from `source/box-exec-daemon/Cargo.toml`.

Historical finding (now superseded): the initial audited release path invoked these through `desktop/package.json` without `--locked`, and the three shipping manifest roots did not yet have authoritative adjacent lockfiles. The repository also contained other unrelated lockfiles, including:

- `source/mahayana/mahayana-rs/Cargo.lock` blob `aedcddd756d2434e2eaaca89c2e6105d1611a0e8`;
- `source/mahayana/codex-rs/Cargo.lock` blob `7591a40df797cde1c4cd88e30bfe3cf248ab3f92`;
- `native/mahayana-messaging/Cargo.lock` blob `876b3fe8a6e182d4d53c379dbfad7b80c967d748`.

Those unrelated lockfiles were not substituted for the shipping manifest roots. The current resolution below instead gives each shipping root its own checked-in lock and uses Cargo `--locked` on the actual package path.

### Release blocker RR-RUST-LOCK-01

The shipping Rust roots now carry authoritative checked-in lockfiles: `source/host/app/Cargo.lock`, `source/host/Cargo.lock`, `source/node-agent-coordinator/Cargo.lock`, and `source/box-exec-daemon/Cargo.lock`. Desktop packaging and Rust runtime CI invoke these roots with `--locked`, and CI validates that each lockfile can satisfy `cargo metadata --locked` before the shipping builds/tests run. The Mahayana workspace continues to use its existing `source/mahayana/mahayana-rs/Cargo.lock`.

This closes RR-RUST-LOCK-01 for source resolution at a candidate HEAD; artifact acceptance still has to bind the produced binaries to that exact HEAD and its checked-in lockfile digests.

## 4. Toolchain and workflow identity

The macOS release workflow binds the source SHA before packaging.

Current toolchain declarations include:

- Node `24` via `actions/setup-node@v6`: major-version constrained, not an exact Node patch/content digest;
- Rust `1.97.1` via `dtolnay/rust-toolchain@stable`: Rust toolchain version is explicit, while the GitHub Action ref itself is a mutable tag;
- GitHub Actions such as `actions/checkout@v6`, `actions/setup-node@v6`, `Swatinem/rust-cache@v2`, and `actions/upload-artifact@v7` are tag-addressed rather than commit-SHA-addressed.

These are build-environment provenance inputs. They do not make Telegram's package-manager/default/latest inputs reachable, but they remain independent supply-chain hardening items for a reproducible Fabushi release.

## 5. Offline ASR packaging blocker

Historical finding (now superseded): `.github/workflows/release-macos-main.yml` read `desktop/electron/offline-asr-engine.json` to obtain an exact `whisper.cpp` repository and commit before staging `desktop/resources/asr`, even though that manifest had already been deleted from the production tree.

Git history shows it was deleted by `0d4969452379eb322978b785ac57de17072faf93` ("refactor: remove superseded desktop runtime roots"). The immediately preceding manifest pinned:

- `ggerganov/whisper.cpp` commit `306c88f4d1286aec1bf96e544632897886af5501`;
- model URL `.../resolve/main/ggml-tiny.bin`;
- model SHA-256 `be07e048e1e599ad46341c8d2a135645097a538221678b7acdd1b1919c6e1b21`.

That historical content is evidence of the old design only. It must not be silently restored as current authority after the runtime-root removal.

### RR-ASR-01 resolution

Current-tree ownership is the Electron account transcription path: `source/electron-main/account/cursor-transcribe.ts` (`SandTranscriptionManager`), wired by `source/electron-main/account/cursor-auth-wiring.ts` and exposed through `source/electron-main/main-edge.ts::transcribeAudio`. No current production source references an offline whisper executable or `desktop/resources/asr`.

The macOS release step that read the deleted `desktop/electron/offline-asr-engine.json`, the Electron Builder `resources/asr` staging rule, and the notarization requirement for `whisper-cli` have therefore been removed rather than reviving superseded runtime roots. The stale `getOfflineAsrStatus` / Offline ASR E2E assumptions were removed with that deleted owner. This closes RR-ASR-01 as a stale release path; voice transcription acceptance now belongs to the current Cursor transcription owner.

## 6. Resource and asset provenance

The frozen Telegram `Telegram/Resources` tree has a complete immutable blob inventory in `telegram-resources-blob-inventory.tsv`, but its independent asset-origin/license evidence remains incomplete.

Current Fabushi package reachability provides the decisive boundary:

- no Telegram `Resources`, shader, `.qsb`, or `.binobj` tree is tracked as a Fabushi package input at this baseline;
- the Fabushi package does include its own `desktop/resources/icon.png`;
- the initial audit found no independent license/origin/provenance row for that icon; the resolution below now supplies one.

Therefore Telegram asset-license uncertainty is a research/legal blocker only if a Telegram-derived asset is later copied/adapted. Fabushi's distributed icon is handled independently by RR-ASSET-01 below.

### RR-ASSET-01 resolution

`desktop/resources/ASSET-PROVENANCE.md` now records the app icon's current Git blob, SHA-256, first tracked project commit, package-level proprietary rights basis, package scope, and attribution status. The record explicitly reopens if a later external source or contributor-specific rights constraint is identified.

Generated/staged resources such as `resources/bin` and `resources/computer-control` inherit provenance from the exact build input that produced each payload, not from the destination directory name. The obsolete `resources/asr` payload is no longer a shipping input.

## 7. Telegram external acquisition disposition

The frozen Telegram scripts contain mutable/default/latest/bootstrap and package-manager acquisition surfaces. Their dated observations are recorded in `external-acquisition-ref-resolution.md`.

For the current Fabushi build graph:

- Telegram `prepare.py`, CentOS Docker acquisition, Snap acquisition, NuGet `latest`, Chromium gyp `master`, Telegram's rustup endpoints, Telegram package-manager snapshots, and Telegram resource payloads are not reachable package inputs;
- therefore historical inability to prove what Telegram downloaded at its frozen-source date does not block the current Fabushi package by itself;
- if any such input becomes reachable through copying, vendoring, invocation, or generated output, the row immediately reopens and must obtain immutable identity plus license/origin evidence.

This distinction is intentional: unresolved historical provenance is recorded, not fabricated; unreachable research inputs are not promoted into the Fabushi shipping dependency graph.

## 8. Release-review checklist

A candidate is not provenance-closed until all applicable items below are bound to the same exact candidate SHA and packaged artifact:

- npm lockfile identity and distributed dependency/license inventory;
- shipping Rust resolution locked or build-record pinned;
- exact toolchain/action identities where policy requires reproducibility;
- ASR stale packaging removed and current Cursor transcription owner recorded;
- distributed asset origin/license rows, including the app icon (`desktop/resources/ASSET-PROVENANCE.md`);
- staged helper/binary provenance for `resources/bin` and `resources/computer-control`;
- source-informed copied/adapted-material rows for any Telegram-derived material actually distributed;
- final artifact SHA-256 and exact-head packaged acceptance evidence.

## 9. P0 conclusion from this dossier

This closes the missing distinction between Telegram's research acquisition graph and Fabushi's actual release graph.

It does **not** close P0 or legal review. RR-RUST-LOCK-01, RR-ASR-01, and RR-ASSET-01 are resolved in the current source graph as documented above; candidate acceptance must still bind lockfile/asset digests and the packaged binaries to the same exact HEAD. Mutable Telegram package-manager/default/latest history remains documented as historically unprovable where no build record exists, but it is not currently reachable from the Fabushi package graph and therefore is not misrepresented as a shipping dependency.
