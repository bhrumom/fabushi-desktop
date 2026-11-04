# Complete read — fixed-U repository Linux CI build/artifact workflow

- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source: `.github/workflows/linux.yml@dbedd23593c5ea66ff9b8b880bda6a8bc5a4cc55`
- Tree index: 124 zero-based
- Mode/size: 100644 / 7,284 ASCII bytes/code points
- Line endings: 215 LF, 0 CR, trailing newline
- Responsibility: `TDRP-R9-REPOSITORY-LINUX-WORKFLOW-001`
- Stage A: authored only; `validation_status=not-run`; `accepted=false`

## Authority note

The active semantic queue/ledger/closure continues fixed U above. `upstream.lock.json` and `inventory/index.json` still carry older rebaseline identity `351204f3695b4a39bc3f4d2f230c061b9a01fe2a`; that pre-existing cross-document mismatch is not silently treated as resolved and is not broadened into this one-entry closure. Formal baseline reconciliation remains a whole-project open item.

## Trigger, runner and permissions

Push excludes `public-canary`/`private-canary` and uses the source path-ignore list with exceptions for this workflow and `Telegram/build/docker/centos_env/**`; pull_request uses the same path rules. No schedule/manual/repository-dispatch/workflow_call, explicit permissions, environment, concurrency, timeout or retry exists. The only matrix value is empty defines. PR/tag-like/nightly expressions select `depot-ubuntu-latest-16`; other admitted refs select `ubuntu-latest`. Exact host admission and runner identity are later evidence.

## Source lineage and toolchain

Observed mutable tag resolutions on 2026-11-01: checkout v7 `3d3c42e5aac5ba805825da76410c181273ba90b1`; setup-buildx v4 `f87e5991a6d7451dcb8d9637bfbc97413f497069`; actions/cache v6 `55cc8345863c7cc4c66a329aec7e433d2d1c52a9`; build-push v7 `c3c9e263c25d99ce0380d002d59b67737d91b0dc`; upload-artifact v7 `cf430e030ddbb5b0abf93d22962f4752f3646cd9`. Free-disk-space and cache-dance are exact-commit pins; cache-dance action metadata is `08ad2eef853162f5336e25928967d9a4e64c0c4e` and defaults its helper image to mutable `ghcr.io/containerd/busybox:latest`.

Bootstrap runs apt metadata refresh, unpinned curl-to-Python Poetry installer, and a versioned ccache 4.13.6 tarball with no asserted checksum. Exact Poetry inputs are pyproject `8f43692fb217f075ccc588c3b677177e42b5688f` and lock `9d857912774d68b1f8adda1dbbccb87adefda788`. Generator `86d27c96e1f732a1faf3e85397ea3ba38e493a0d` receives empty DEBUG/LTO and renders both false; Dockerfile template is `3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2`. Existing build-time acquisition inventory remains the detailed authority for that template graph.

## Cache and build

Libraries cache keys Buildx/cache-mount state by runner OS and hash of the centos_env subtree, with OS-wide restore fallback. Exact cache-dance injects/extracts mounts. Build-push creates and loads local `tdesktop:centos_env`, reads an old local Buildx cache and writes a new max-mode cache; subsequent rm+mv replacement can be partial. Ccache restore uses unique run-id primary key with OS/defines fallback. Only default-branch runs with parsed misses >300 evict older-than-1d entries and save; malformed miss output is coerced to zero.

Build uses `build.sh@00b5c9a2faf5e25fb981325f654d3458ce074a7d` -> `configure.sh@c550d988cff5fe3047311cc330aeee943ee328e5` -> configure.py/CMake. The container runs as host uid with repo and ccache read-write mounts, Debug config, ccache sloppiness, O0/pch flags, warnings-as-errors, API-test build option and auto-update/crash-report disable flags set OFF. No test command is present.

## Quality, artifact and publication

`Check.` accepts any regular `out/Debug/Telegram` file and only prints size, so zero-byte output is source-accepted. Updater is not checked separately before `mv {Telegram,Updater} artifact/`; failure can partially stage Telegram. Upload artifact is named `Telegram` for the sole matrix value, path `out/Debug/artifact/`, retention one day. No digest is captured by source, and there is no packaging, signing, installation, SBOM, attestation, release or updater publication.

## Callers/consumers

Direct callers are GitHub push/pull_request event admission; dependencies/consumers are checkout/submodules, Poetry/ccache/network bootstrap, Docker/BuildKit/cache services, source build scripts, GitHub cache/artifact stores, maintainers/reviewers and manual artifact users. All 19 sibling fixed-U workflow blobs were checked for direct reusable/workflow-run coupling; none calls linux.yml. Canary contains a prose reference and independent artifact/download steps, not a caller of this workflow.

## Security and failure boundaries

No explicit token permissions plus checkout credential persistence, recursive submodules, PR-controlled source execution and a full read-write repo mount are a credential/trust boundary. Mutable runners/actions, curl-to-Python, no-digest ccache archive, broad cache restore prefixes, mutable cache helper image and Docker acquisition graph are provenance boundaries. Unsigned Debug output is not release authority. There is no workflow-level retry, concurrency lock or rollback. Cache rotation/artifact staging can be partial; ccache parse anomalies degrade silently; later run success cannot establish earlier identities.

## Ownership and authored verification

Existing-owner-first maps to current Fabushi repository build/toolchain + package/release/process + security/provenance + shared quality/evidence governance. No Telegram Linux workflow, second package/release truth, product runtime, Agent/Coordinator/Host/Runner change or platform-core delta is created.

Authored only: 28 contract, 1 integration, 16 fault/recovery and 14 security/provenance cases; none executed.
