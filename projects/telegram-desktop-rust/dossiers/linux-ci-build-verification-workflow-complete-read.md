# Linux CI build verification workflow complete read — strict index 86

## Authority
- Upstream: `telegramdesktop/tdesktop@5380230173ea60a5222aa9d977f074dd41e76139`, tree `bd6a23621d2a6d69bd986d3a0f01514c2f2cf96b`.
- Source: `.github/workflows/linux.yml@dbedd23593c5ea66ff9b8b880bda6a8bc5a4cc55`, 216 line slots / 7284 bytes.
- Parent exact head/tree: `783610caea5af7a7ae94aad698321cc72934d2e7` / `47c7f89aa510b1ec384960da96797617e07fdeab`.
- Parent exact-head 783610caea5af7a7ae94aad698321cc72934d2e7 completed TDRP run 39788414121 at 61/61 success (authority 12429847473 sha256:59083262c52d76c9c0452e0244236d1987bc927b2c828a114ae6c779686c11cc; closure 12430960879 sha256:9ceedbbab0f9f21be13e63ce726c46e1df18586a1858141808c34e9226e8ee94; reachability 12430561986 sha256:ce804ecae7c497be8f7dd6edb41b704a79d04faa6b45f582937c8449f720a695) and Rust desktop runtime run 39788414127 at 6/6 success (artifact 12429937087 sha256:fc657b28ef7c56bd010fb445a10088d09f58eefb145d2a7b594ecc9202e04ac1).
- Existing owner: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` — `For FBCP/TDRP, executable validation remains exact-head GitHub-Actions-only.`

## Complete-read behavior
The workflow handles Linux-relevant push/pull_request events, excludes public/private canary pushes and broad docs/legal/snap/Windows/macOS/unrelated build paths, while re-including itself and `Telegram/build/docker/centos_env/**`. It recursively checks out source, installs Poetry online, downloads ccache 4.13.6 without an asset checksum, installs the locked Python generator environment, renders the centos_env Dockerfile, frees runner disk, configures Buildx and BuildKit caches, builds+loads a local `tdesktop:centos_env` image, restores ccache, and runs `Telegram/build/docker/centos_env/build.sh` for a Debug/API-test/warnings-as-errors build.

It requires `out/Debug/Telegram`, stages Telegram and Updater, then uploads a one-day Actions artifact. ccache is saved only on the default branch when misses exceed 300. Cache state is performance state only; correctness must survive cache absence/rebuild.

## Provider/dependency provenance
- `actions/checkout@v7 -> 3d3c42e5aac5ba805825da76410c181273ba90b1`.
- `docker/setup-buildx-action@v4 -> f87e5991a6d7451dcb8d9637bfbc97413f497069`; action.yml `1ea4b62fdb83f1818d5ba23e69e5b0df7652b606`; Apache-2.0.
- `actions/cache@v6 -> 55cc8345863c7cc4c66a329aec7e433d2d1c52a9`; root/restore/save action.yml `2606455d4cedc2a06e05c1bf7feb8cc41aaf28f9` / `93352f50541fca3b088080dba6b102051220dda2` / `1b95184507516b6b36a065e21d77e35b3acc4986`; MIT.
- `reproducible-containers/buildkit-cache-dance@5422eac04292c961a382e0f584ea0f03ad9da723`; action.yml `08ad2eef853162f5336e25928967d9a4e64c0c4e`; Apache-2.0.
- `docker/build-push-action@v7 -> c3c9e263c25d99ce0380d002d59b67737d91b0dc`; action.yml `7a1a94d46f66694384cb558b6f164027cdc3b667`; Apache-2.0.
- `jlumbroso/free-disk-space@ceedf095f4ec1a097402bc6bd80831f2e1a6fde6`; action.yml `ed4b963758022e538f41c06283befbbb924d5ad5`; MIT.
- `actions/upload-artifact@v7 -> cf430e030ddbb5b0abf93d22962f4752f3646cd9`; action.yml `7cb4d1e81db55320b41217e1a78a1a46e3d2baef`; MIT.
- ccache `v4.13.6` tag object `100d636a986eb3039880a10ac70869ac187eeee6` resolves to commit `c6f36725ddaa247678ff130cf4349055962ac072`; `LICENSE.adoc@6ac9655f327c4edffb36c52c5d2b96e68024252d` is GPL-3.0-or-later. The release asset URL is not checksum-pinned.
- Independent open inputs remain `Dockerfile@0105fe2123efdc2fc361e546fdbacb6305acbfe6`, `gen_dockerfile.py@86d27c96e1f732a1faf3e85397ea3ba38e493a0d`, `pyproject.toml@8f43692fb217f075ccc588c3b677177e42b5688f`, `poetry.lock@9d857912774d68b1f8adda1dbbccb87adefda788`, `build.sh@00b5c9a2faf5e25fb981325f654d3458ce074a7d`.

## Existing-owner-first disposition
This is CI build-verification provenance, not a Telegram runtime. Fabushi already requires exact-head GitHub-Actions executable validation and keeps source/dependency/container, packaging, security, FQT evidence and release acceptance in existing owners. strict84 mapped publication of the Linux build-environment image; strict86 separately maps source-build verification that locally materializes its build image and checks/uploads its build result. No second build system, Linux product runtime, cache truth, packaging/release owner or acceptance authority is introduced.

## Fail-closed conclusions
Mutable action tags, the online Poetry installer, unchecksummed ccache release asset, provider runner environment, BuildKit/ccache state and one-day Telegram artifacts are provenance only. A cache hit, upstream build success, output-file existence or artifact upload cannot establish Fabushi packaging, signing, release or independent acceptance. Generated Dockerfile/acquisition inputs remain separately open.

## Exact traceability
- Requirement: `TDRP-R9-LINUX-CI-BUILD-VERIFICATION-PROVENANCE-001`
- Oracles: `ORA-TDRP-LINUX-CI-EXACT-SOURCE-001`, `ORA-TDRP-LINUX-CI-BUILD-RESULT-001`, `ORA-TDRP-LINUX-CI-NO-RELEASE-AUTHORITY-001`
- Invariants: `INV-TDRP-LINUX-CI-CACHE-NO-AUTHORITY-001`, `INV-TDRP-LINUX-CI-NO-PRODUCT-RUNTIME-001`, `INV-TDRP-LINUX-CI-EXACT-HEAD-EVIDENCE-001`
- Temporal/fault: `TEMP-TDRP-LINUX-CI-ACTION-TAG-DRIFT-001`, `TEMP-TDRP-LINUX-CI-CACHE-DRIFT-001`, `FAULT-TDRP-LINUX-CI-POETRY-INSTALLER-001`, `FAULT-TDRP-LINUX-CI-CCACHE-ASSET-001`, `FAULT-TDRP-LINUX-CI-BUILDX-CACHE-001`
- Security: `SEC-TDRP-LINUX-CI-SUPPLY-CHAIN-001`, `SEC-TDRP-LINUX-CI-RUNNER-BOUNDARY-001`

Closing strict index 86 changes counters to unknown=16,282, unread=16,257, omitted=0, responsibilities=104. `baseline_identity_accepted=false` and `source_closure_ready=false`. Next strict root is index 87 `.github/workflows/lock.yml@7904e79591f79a4ea61c0dc194c189f048bf5c67`.

## Live build-input rebaseline 52a47639d9aa
The workflow source `.github/workflows/linux.yml@dbedd23593c5ea66ff9b8b880bda6a8bc5a4cc55` is unchanged. Its centos_env input is now `Dockerfile@3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2` and `desktop-app/patches@845a383f7abe8e44f92dfdcd566d603066ec2696`; the repin adds the Qt RHI stale-access guard only. The existing exact-head GitHub-Actions owner, cache non-authority rule, supply-chain boundary and release-evidence separation remain unchanged.
