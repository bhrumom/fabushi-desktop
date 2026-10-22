# Linux build-environment image publish workflow complete read — strict index 84

## Authority
- Upstream: `telegramdesktop/tdesktop@42e2af80cdd20a99edecb4255470f0aa6bae6bcb`, tree `19740740209ea84bf3768f3f63ca92ea8d861087`.
- Source: `.github/workflows/docker.yml@cba00688d935d50454bfb891fc37e001626cf9a0`, 43 line slots / 1121 bytes.
- Parent exact head/tree: `712b92e0503adb72bbbd64d996568dbe64d763c0` / `a7680931afb9e2144ef4957dd0e4da25c9626a31`.
- Parent exact-head 712b92e0503adb72bbbd64d996568dbe64d763c0 completed TDRP run 39771425432 at 61/61 success (authority 12421549778 sha256:8c1b5525bc43eca99a0f308ff1ae3e5aa5944762ac165e36cd3a74f3a15f8e41; closure 12423575035 sha256:6ad8318faefdff18fffab56d24b2c49100b2ee8409141a5d14919b5627dbea52; reachability 12423330472 sha256:f9082755de4aecc8020e28ab03913c173ac9a1d7073ae704b35239cf7436bde0) and Rust desktop runtime run 39771425437 at 6/6 success (artifact 12423225318 sha256:9285ee0266657d4b587fa584a56810cf4c258e343e0e2cd25e148cde4d5474ef).
- Live Fabushi main rechecked at `7394f6dc7687f9b986ee5589d40aa680ce23b90a`.
- Existing owner: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` — Repository release-support container provenance governance.

## Complete-read workflow behavior
The workflow is path-filtered to pushes that touch the workflow itself or `Telegram/build/docker/centos_env/**`, and the job is additionally gated by `github.ref_name == github.event.repository.default_branch`. It checks out the repository recursively, runs `apt update`, downloads and executes the Poetry installer, authenticates Docker to GHCR with `GITHUB_TOKEN`, frees hosted-runner disk space, runs `poetry install`, streams the reviewed Jinja-generated Dockerfile directly into BuildKit, builds the image, and pushes `ghcr.io/${{ github.repository }}/centos_env:latest`. The workflow does not contain a Docker installation step.

The workflow publishes a mutable build-support pointer. It does not digest-bind the consumer-facing `:latest` tag and does not declare an explicit permissions block in this source file.

## Recursive inputs and consumers
The generator and lock inputs are anchored at:
- `gen_dockerfile.py@86d27c96e1f732a1faf3e85397ea3ba38e493a0d`
- `pyproject.toml@8f43692fb217f075ccc588c3b677177e42b5688f`
- `poetry.lock@9d857912774d68b1f8adda1dbbccb87adefda788`
- generated template `Dockerfile@3135794864044f21d2a0092b32cff787c86704bf`

The Dockerfile begins from mutable `rockylinux:8` and contains a mixture of exact commits, version tags, online archives and online installers. Those source-acquisition entries remain separate unread inventory responsibilities and are not falsely closed by strict84.

A direct upstream consumer exists at `.oss-scanner/Dockerfile@0ba260f72ad88b10a0d613ff5027b7a7640be7f2`, which starts from `ghcr.io/telegramdesktop/tdesktop/centos_env:latest`. Local Linux build preparation also generates `tdesktop:centos_env` via `Telegram/build/prepare/linux.sh@e4d2f920e4becdff6292d296efe6f21455a19fcb`; `docs/building-linux.md@fbdf25355734f35617849224da106f1b9fafb3c9` documents that consumer path.

## Existing-owner-first disposition
Fabushi already has source-neutral release-support container provenance governance. Strict84 therefore adds no Linux build-container product runtime, registry owner, release owner, packaging owner or acceptance authority.

The selected canonical target symbol is:
`Release-support containers and canary-only service images are bounded build/test/release adapters over existing source, dependency, security, packaging and evidence owners; they are not shipping Fabushi product runtimes and do not create a second service or protocol owner.`

The workflow remains a provenance adapter under that owner. Exact source, dependencies and digest-qualified artifacts are authoritative; mutable provider tags are not.

## Fail-closed and security conclusions
- `actions/checkout@v7` is a mutable major tag; its audit-time resolved commit is provenance, not a future immutable guarantee.
- The Poetry installer is executed from a network URL without a digest in the workflow.
- `rockylinux:8`, many Dockerfile dependency tags and `centos_env:latest` are mutable pointers.
- The workflow token authenticates GHCR; registry credentials remain inside CI/provider boundaries.
- No explicit concurrency group is declared. Multiple qualifying default-branch runs may race on the same `:latest` tag; later successful pushes may retarget it.
- A successful push does not prove what digest a later consumer resolves.
- A successful image build/push cannot satisfy Fabushi production, packaging, release or migration acceptance by itself.
- The direct OSS Scanner consumer makes mutable-tag drift a real build-environment provenance risk, not merely cosmetic CI metadata.

## Exact traceability IDs
- Requirement: `TDRP-R9-LINUX-BUILD-ENV-IMAGE-PUBLISH-PROVENANCE-001`
- Oracles: `ORA-TDRP-LINUX-BUILD-IMAGE-EXACT-SOURCE-001`, `ORA-TDRP-LINUX-BUILD-IMAGE-DOWNSTREAM-CONSUMER-001`, `ORA-TDRP-LINUX-BUILD-IMAGE-NO-RELEASE-AUTHORITY-001`
- Invariants: `INV-TDRP-LINUX-BUILD-IMAGE-MUTABLE-TAG-NO-AUTHORITY-001`, `INV-TDRP-LINUX-BUILD-IMAGE-NO-PRODUCT-RUNTIME-001`, `INV-TDRP-LINUX-BUILD-IMAGE-SECRETS-BOUNDARY-001`
- Contracts: `CONTRACT-TDRP-LINUX-BUILD-IMAGE-PROVENANCE-001`, `CONTRACT-TDRP-LINUX-BUILD-IMAGE-CONSUMER-HANDOFF-001`
- Temporal: `TEMP-TDRP-LINUX-BUILD-IMAGE-LATEST-DRIFT-001`
- Fault/recovery: `FAULT-TDRP-LINUX-BUILD-IMAGE-INSTALLER-001`, `FAULT-TDRP-LINUX-BUILD-IMAGE-REGISTRY-001`, `FAULT-TDRP-LINUX-BUILD-IMAGE-TAG-DRIFT-001`
- Security: `SEC-TDRP-LINUX-BUILD-IMAGE-GITHUB-TOKEN-001`, `SEC-TDRP-LINUX-BUILD-IMAGE-SUPPLY-CHAIN-001`
- Regression: `REG-TDRP-LINUX-BUILD-IMAGE-NO-PRODUCT-RUNTIME-001`, `REG-TDRP-LINUX-BUILD-IMAGE-NO-RELEASE-ACCEPT-001`

Closing strict index 84 changes counters to unknown=16,219, unread=16,194, omitted=0, responsibilities=102. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 85 `.github/workflows/issue_closer.yml@355405165590712b1826b2f92a8f867c8d568dd9`.

## Source-semantic correction
The first strict84 publication used inaccurate shorthand for the trigger and runner preparation. The locked workflow has no `master`/`dev` branch list and no Docker installation step: it path-filters `push`, fences the job to the repository default branch, and streams `gen_dockerfile` output to `docker build -`. This correction changes no source disposition, owner, coverage count, product code, or acceptance flag.

## Live build-input rebaseline cade9c628cc62
The workflow source `.github/workflows/docker.yml@cba00688d935d50454bfb891fc37e001626cf9a0` is unchanged. Its generated build-input dependency is now `Telegram/build/docker/centos_env/Dockerfile@0105fe2123efdc2fc361e546fdbacb6305acbfe6`, with `desktop-app/patches` advanced linearly from `97b2b514c76666b24b43b31e2aacfa703b6aa6b2` to `1f43f0f1d5ba8643ea45bbd8404782d4d840e421`. Build-container provenance/owner disposition is unchanged; the mutable published image remains non-authoritative and fresh exact-head Actions are required for the new upstream lease.
