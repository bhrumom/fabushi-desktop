# Complete read — fixed-U repository Docker build/publish workflow

Responsibility: `TDRP-R9-REPOSITORY-DOCKER-BUILD-WORKFLOW-001`

## Authority snapshot

- Desktop live checkpoint before spec-first write: `cba7dfef90aaa892c9445b60ce299f24edd24b24` on `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`.
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core authority: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree is non-truncated with 7,194 entries.
- Source entry: `.github/workflows/docker.yml@cba00688d935d50454bfb891fc37e001626cf9a0`, tree index 122 zero-based, mode 100644, Git size/decoded ASCII length 1,121, 42 LF, zero CR, trailing newline.
- Current unique ledger before closure: source_entries_total=16,202; unknown=16,112; unread=16,085; omitted=0; responsibilities_total=166; unmapped_responsibilities=0.
- Historical `upstream.lock.json` and inventory index still carry older rebaseline identities and do not override the current fixed-U ledger/SOURCE_OF_TRUTH queue.

## Trigger and execution admission

The workflow declares only `push` with path filters `.github/workflows/docker.yml` and `Telegram/build/docker/centos_env/**`. It has no trigger-level branches filter. The one `docker` job then applies `if: github.ref_name == github.event.repository.default_branch`. Therefore path-matching non-default pushes can admit the workflow event while the job is skipped; only the runtime default branch is eligible to build/publish. There is no workflow_dispatch, schedule, repository_dispatch, pull_request, workflow_call, concurrency group, environment, timeout or retry declaration.

The single job is named Ubuntu and runs on mutable `ubuntu-latest`. No explicit `permissions` block exists, so effective GITHUB_TOKEN scopes are a repository/host-policy boundary. Source inspection cannot prove package write access.

## Checkout, tool bootstrap and registry authentication

The job declares `IMAGE_TAG=ghcr.io/${{ github.repository }}/centos_env:latest`.

Checkout uses written ref `actions/checkout@v7` with `submodules: recursive`. The v7 tag was observed on 2026-11-01 at `3d3c42e5aac5ba805825da76410c181273ba90b1`; the tag remains mutable. Its metadata defaults to github.token, persisted credentials and fetch-depth 1 unless overridden. Recursive submodule resolution therefore becomes part of the build-source lineage.

First setup runs `sudo apt update`, then `curl -sSL https://install.python-poetry.org | python3 -`. The installer URL is executed directly with no source version or digest in the workflow. Public repository observation of the installer is only mutable context, not a fixed-U identity. It then executes `echo "${{ secrets.GITHUB_TOKEN }}" | docker login ghcr.io -u $ --password-stdin`. The shell username token is literal `$`; actual login outcome depends on registry and token policy. Secret values are never admissible evidence content.

Disk cleanup uses exact commit `jlumbroso/free-disk-space@ceedf095f4ec1a097402bc6bd80831f2e1a6fde6` with `tool-cache: true`. At that pinned revision the input is a deprecated alias for preinstalled-runtimes removal; the action also applies its other default cleanup categories. Cleanup mutates the runner before build and is therefore a toolchain/failure boundary, not a product responsibility.

## Container build and source lineage

The workflow enters `Telegram/build/docker/centos_env`, whose exact relevant fixed-U inputs are:

- `pyproject.toml@8f43692fb217f075ccc588c3b677177e42b5688f`
- `poetry.lock@9d857912774d68b1f8adda1dbbccb87adefda788`
- `gen_dockerfile.py@86d27c96e1f732a1faf3e85397ea3ba38e493a0d`
- `Dockerfile@3fea9e6866e0ea6351f8a13ebb825ebf49cadfc2`
- build/run helper blobs already independently inventoried with that subtree.

`poetry install` consumes the lock after Poetry itself has been bootstrapped mutably. The project pins Jinja2 3.1.6 and MarkupSafe 2.1.5 in the fixed-U lock, while the build-system declaration allows `poetry-core>=1.0.0`; actual bootstrap/build-system resolution remains later evidence.

The generator's boolean helper maps a present environment variable to `bool(len(value))`. Consequently `DEBUG=` and `LTO=` both become false, overriding their true defaults. MINSIZE=false, ASAN=false and JOBS empty remain defaults. The command pipes the rendered Dockerfile into `DOCKER_BUILDKIT=1 docker build -t $IMAGE_TAG -`.

The template starts with mutable `rockylinux:8`, installs distribution/Python tooling, and then acquires a large dependency graph through package managers, git, curl/wget and language toolchains. The repository's existing `build-time-acquisitions.json` already classifies exact-pin and mutable build-source authorities for this Dockerfile and classifies this workflow itself as CI-only/tool-bootstrap. This closure reuses that single inventory rather than creating a second acquisition truth.

## Publication and consumers

Publication is exactly `docker push $IMAGE_TAG`, where the only tag is mutable `centos_env:latest`. The workflow does not emit or persist a digest, immutable version tag, SBOM, provenance/attestation, signature, artifact receipt or rollback pointer.

Fixed-U `.oss-scanner/Dockerfile@0ba260f72ad88b10a0d613ff5027b7a7640be7f2` explicitly states that this workflow publishes the prebuilt dependency environment and begins from `FROM ghcr.io/telegramdesktop/tdesktop/centos_env:latest`. That is a direct downstream consumer and proves that latest-tag mutability is a real source trust boundary. The OSS scanner then copies its checkout, initializes recursive submodules, and builds Telegram Desktop against the image. This source consumer is not imported as a Fabushi runtime.

Related Linux/canary build lanes reuse `gen_dockerfile` and/or local centos_env build behavior as build mechanics; they do not change the ownership conclusion for this source entry. The current source workflow has no reusable-workflow caller surface; its caller is GitHub push event admission plus the runtime default-branch job gate.

## Security, failure and recovery

Security boundaries include mutable ubuntu-latest, mutable checkout tag, persisted checkout credentials, recursive submodule acquisition, implicit GITHUB_TOKEN permissions, apt metadata, curl-to-Python Poetry bootstrap, mutable base image/package sources, Docker daemon/BuildKit, registry credential use and mutable latest publication. Exact secret values must never be recorded.

Failure ordering is fail-closed by normal step dependency: checkout failure prevents setup; setup/login failure prevents cleanup/build; cleanup action failure prevents build unless the action itself swallows a cleanup subcommand failure; poetry/generator/build failure prevents push; push failure prevents source completion. There is no explicit retry, backoff, concurrency exclusion, compensation, digest reconciliation or rollback.

A non-default path-matching push is a skip, not a successful publication. A successful overwrite of `:latest` does not prove continuity with a prior digest. A push interruption or registry-side ambiguity remains non-passing until an immutable manifest digest is obtained and reconciled in Stage B.

## Existing-owner-first disposition

Target owner: existing Fabushi repository container/build/toolchain provenance + release/process governance + repository security/provenance + shared quality/evidence gates.

Rejected owners: copied Telegram docker workflow, TelegramContainerRuntime, Telegram provider/sidecar, second package/release service, product Tasks/Automations, Agent/Coordinator/Host/Runner runtime, platform-core service, or second security/evidence truth.

There is no product UI responsibility and no platform-core production delta. Existing acquisition inventory remains authoritative for the Dockerfile dependency graph.

Authored Stage-A cases: 16 contract, 1 integration, 9 fault/recovery and 9 security/provenance cases listed in root AGENTS/shared TEST_STRATEGY. All are not run and unaccepted.

## Stage-A execution statement

No Actions/check/status/workflow-run query or trigger, source push, action execution, checkout/submodule fetch, apt/curl/Poetry execution, Docker/BuildKit build, registry login/push/pull, token probe, validator, test/E2E, packaging, signing, release, deployment, installation, runtime acceptance or platform-core mutation was performed. Only Git/API/source reads, contract/evidence authoring and safe checkpoint publication are in scope.
