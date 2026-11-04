# Fixed-U `.github/telegram-bot-api/Dockerfile` complete read

- Upstream repository: `telegramdesktop/tdesktop`
- Fixed upstream commit: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6` (`truncated=false`, 7,194 entries)
- Source path/blob: `.github/telegram-bot-api/Dockerfile@a361602441040028a3332c3732c4902d9c880122`
- Tree index: 114 zero-based; mode `100644`; Git size 1,697 bytes
- Decoded content: 1,697 code points, 41 LF, 0 CR, trailing newline, 42 split lines
- Triage: `full-read`
- Target responsibility: `TDRP-R9-REPOSITORY-CANARY-BOT-API-CONTAINER-001`
- Task round: 159
- Source evidence round: 190
- Desktop checkpoint parent: `a95a00949d08c0c857f11dae5fe1edfc21d24beb`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core checkpoint: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`

## Container and base-image contract

The file is a two-stage Linux image definition for the Telegram Desktop canary publication Bot API service. Both stages use the same immutable source base reference, `ubuntu:22.04@sha256:2edbbc5dc405e9612ba3584ce95480277e3eb374407b5505fe26f17df77c7dbc`; the source comment identifies this as Ubuntu 22.04 as observed on 2026-08-20. The base digest fixes the selected base-image content, but it does not freeze later `apt-get update/install` results.

The build stage installs `ca-certificates cmake g++ git gperf libssl-dev make zlib1g-dev` and removes the apt list cache. The runtime stage separately installs `ca-certificates libssl3 zlib1g`, removes apt lists, creates system user `botapi` with UID 999 and a home directory, creates `/data`, and transfers ownership of `/data` to that user.

## Build/acquisition and source-lineage contract

Build input is `ARG TELEGRAM_BOT_API_REF`. The Dockerfile itself requires only that it be non-empty. It then:
1. clones `https://github.com/tdlib/telegram-bot-api.git` into `/src`;
2. runs `git -C /src checkout "$TELEGRAM_BOT_API_REF"`;
3. runs recursive submodule initialization/update;
4. prints the resolved superproject HEAD and recursive submodule status;
5. configures CMake from `/src` to `/build` with `CMAKE_BUILD_TYPE=Release` and `CMAKE_INSTALL_PREFIX=/out`;
6. builds/install target in parallel.

The source therefore has explicit external network acquisition for Ubuntu repositories, the GitHub clone, and whatever remotes recursive submodules require. A digest-pinned Ubuntu base does not make the apt packages immutable. A textual Git ref is not provenance by itself; resolved checkout/submodule object identities are the meaningful source objects.

Exact direct build caller is fixed-U `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9` (2,364 characters, 68 LF, trailing newline). It is manual `workflow_dispatch` only and requires input `ref`. Before building it rejects any input not matching exactly 40 lowercase hexadecimal characters. It checks out tdesktop with `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1`, logs into GHCR with `docker/login-action@81c6f93d4bc991e510b342a00b419b3bb4efc5c6`, and uses `docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a`; these action dependencies are commit-SHA pinned in the fixed source. Build context is exactly `.github/telegram-bot-api`, build arg is `TELEGRAM_BOT_API_REF=${{ inputs.ref }}`, the pushed tag is `ghcr.io/${{ github.repository_owner }}/telegram-bot-api:${{ inputs.ref }}`, and the workflow prints the resulting digest-pinned reference for deliberate assignment to `CANARY_BOT_API_IMAGE`. No image build or external Bot API ref is fixed by the Dockerfile alone.

## Runtime entrypoint, network and I/O contract

The runtime stage copies `/out/bin/telegram-bot-api` to `/usr/local/bin/telegram-bot-api` and exact build-context `.github/telegram-bot-api/entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e` to `/usr/local/bin/entrypoint.sh`. The entrypoint source is mode `100755`, 322 bytes/code points, 11 LF, 0 CR, trailing newline.

The final image runs as `USER botapi`, uses `WORKDIR /data`, declares `EXPOSE 8081`, and uses exec-form `ENTRYPOINT ["/usr/local/bin/entrypoint.sh"]`. The entrypoint uses `set -e` then replaces itself with `telegram-bot-api`, making the server the container's PID 1. It supplies `--http-port=8081`, `--dir=/data`, `--temp-dir=/data/temp`, conditionally appends `--local` when `TELEGRAM_LOCAL` is non-empty, and forwards all caller arguments. `TELEGRAM_API_ID` and `TELEGRAM_API_HASH` are intentionally read by the server process from runtime environment; they are not Docker build args.

Inputs are therefore: pinned base reference, mutable package repositories, validated external Bot API commit input from the caller, recursive submodule remotes, exact two-file build context, and runtime API credentials/local-mode/argv. Outputs are an installed server image, runtime `/data` working state, and an HTTP Bot API endpoint on 8081. The Dockerfile declares no `VOLUME`, healthcheck, restart policy, privileged capability, secret mount, registry credential, persistent database, or retry loop.

## Direct consumers

Fixed-U `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947` is the direct image consumer. Before publish, its version job requires `CANARY_BOT_API_IMAGE` to match a digest-pinned `ghcr.io/...@sha256:<64 hex>` reference. The win64, mac, and linux publish jobs each instantiate a `telegram-bot-api` service from that digest, authenticate to GHCR with the workflow actor/GitHub token, inject `CANARY_API_ID` and `CANARY_API_HASH` only as runtime environment, set `TELEGRAM_LOCAL=1`, and map 8081:8081. Each publish job sets `BOT_API=http://localhost:8081` and `BOT_TOKEN` for exact `Telegram/build/canary_publish.sh@4eb2d790063bb8f11efc7ffa06977fe1ef79a270`.

The publish script waits up to 30 iterations with two-second sleeps for `getMe` success because GitHub service-container startup alone does not prove tdlib readiness. It then verifies exact update/portable filenames, posts update and portable files through the local Bot API, and rewrites the per-platform metadata message. Those publication semantics are direct source consumers of the container but remain separate canary/release responsibilities.

## Security and failure/recovery

Security positives:
- both base stages are digest-pinned;
- the build workflow validates a full commit-shaped input;
- workflow actions used to checkout/login/build-push are SHA-pinned;
- the canary consumer requires a GHCR sha256 image digest;
- API credentials are runtime inputs rather than build args;
- the service runs as non-root UID 999;
- the image is source-owned rather than a third-party prebuilt Bot API image.

Residual or negative boundaries:
- Ubuntu package indexes/packages are mutable network acquisition;
- clone/submodule remotes remain external acquisition edges;
- the Dockerfile itself only checks non-empty ref, so full-commit validation belongs to its exact caller;
- a 40-hex input is not proof of object availability until checkout succeeds;
- no image-level healthcheck/restart policy exists;
- `EXPOSE 8081` is metadata, not an access-control rule;
- no build/push success or source-side image digest independently establishes Fabushi release acceptance.

Any apt, clone, checkout, submodule, CMake/build/install, context-copy, image-push, registry, container-start, credential, or readiness failure is non-passing. Recovery is explicit and provenance-preserving: reread exact tdesktop source/caller, select/verify a concrete external commit input, rebuild only in an authorized later stage, bind the resulting image digest, and requalify the exact consumer. Never substitute a third-party image, mutable tag, root/privileged fallback, baked secret, or stale digest.

## Existing-owner-first disposition

Use:
- root repository build/development-process and source-provenance governance;
- repository dependency advisory / supply-chain quality governance;
- repository security governance;
- existing development-container/toolchain provenance as the nearest container/toolchain domain, without confusing development and publication containers;
- shared `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`;
- shared `projects/fabushi-communication-platform/quality/evidence-contract.md`;
- existing release entry/exit gates.

Reject:
- Telegram Bot API sidecar/provider/runtime inside Fabushi Desktop;
- Telegram-specific container/image owner;
- product updater or second release/publication service;
- Agent / Coordinator / Host / Runner product runtime;
- Product Tasks/Automations/Computer;
- platform-core service or authorization owner;
- shipping product Settings/Permissions as container credential owner;
- second supply-chain/security/evidence truth.

No ADR, target Dockerfile/workflow, shipping production code, product UI, database, deployment, or platform-core delta is required for this repository/process provenance responsibility.

## Authored validation — not run

- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-BASE-IMAGE-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-BUILD-ACQUISITION-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-BUILD-INSTALL-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-RUNTIME-IDENTITY-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-ENTRYPOINT-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-NETWORK-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-INPUT-OUTPUT-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-SOURCE-LINEAGE-001`
- `CONTRACT-TDRP-CANARY-BOT-API-CONTAINER-CONSUMER-001`
- `INT-TDRP-CANARY-BOT-API-CONTAINER-REPOSITORY-PROCESS-001`
- `FAULT-TDRP-CANARY-BOT-API-CONTAINER-REF-UNRESOLVED-001`
- `FAULT-TDRP-CANARY-BOT-API-CONTAINER-ACQUISITION-FAILURE-001`
- `FAULT-TDRP-CANARY-BOT-API-CONTAINER-SERVICE-READINESS-001`
- `FAULT-TDRP-CANARY-BOT-API-CONTAINER-DIGEST-DRIFT-001`
- `SEC-TDRP-CANARY-BOT-API-CONTAINER-NONROOT-001`
- `SEC-TDRP-CANARY-BOT-API-CONTAINER-SECRET-BOUNDARY-001`
- `SEC-TDRP-CANARY-BOT-API-CONTAINER-MUTABLE-ACQUISITION-001`
- `SEC-TDRP-CANARY-BOT-API-CONTAINER-NO-THIRD-PARTY-IMAGE-001`

All remain Stage-A authored-only: no Docker/BuildKit, apt, git/submodule, CMake/build/install, registry push/pull, container/service start, Bot API request, workflow, validator, test/E2E, package, signing, release, deployment, installation, runtime evidence or acceptance was executed.

## Queue continuity

Exactly one independent fixed-U source entry is represented by this dossier. `.github/telegram-bot-api/entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e` at recursive-tree index 115 was read here only as the Dockerfile's exact build-context/runtime dependency and remains the next independently unclosed source entry.
