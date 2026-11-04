# Complete read — fixed-U canary Bot API image-build workflow

Responsibility: `TDRP-R9-REPOSITORY-CANARY-BOT-API-BUILD-WORKFLOW-001`

## Authority snapshot

- Desktop live checkpoint before write: `b3e104612212c4b27c8fcbc7a18247390ec581e7` on `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`.
- Desktop main: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core authority: `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`; commit tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`; recursive tree read non-truncated with 7,194 entries.
- Source entry: `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9`, recursive index 117 zero-based, mode 100644, Git size 2,364, ASCII code points 2,364, 68 LF, 0 CR, trailing newline.
- Context tree: `.github/telegram-bot-api@050b40e726444074e504dadfa3701eedb58bc3e3`, exactly Dockerfile `a361602441040028a3332c3732c4902d9c880122` plus entrypoint `d40a7372efd8e5862e00f9b214b36a840688797e`.

Current `parity-ledger.json` is already rebound to fixed U with `source_entries_total=16202`, `unknown=16117`, `unread=16090`, `omitted=0`, `responsibilities_total=161`, `unmapped_responsibilities=0`. Historical `upstream.lock.json` / `inventory/index.json` still describe older rebaseline identities; they are not used to override the fixed-U current ledger/SOURCE_OF_TRUTH queue in this round and are not silently rewritten outside this source-entry scope.

## Complete source semantics

The file declares only `workflow_dispatch` and one required input named `ref`. No push, pull request, schedule, repository dispatch, reusable-workflow trigger, or automatic repository-variable write exists. The selected runner is `depot-ubuntu-latest-32`; the job binds GitHub environment `canary` and workflow permissions `contents: read`, `packages: write`.

Validation assigns the input to environment variable `REF` and executes Bash regex `^[0-9a-f]{40}$`. This accepts only exactly forty lowercase hex characters. It rejects uppercase and abbreviated SHA forms, but it proves syntax only; it neither resolves nor verifies a commit in `tdlib/telegram-bot-api`. A syntactically valid nonexistent ref proceeds until the Dockerfile's `git checkout` fails.

Checkout is immutable-action pinned at `actions/checkout@3d3c42e5aac5ba805825da76410c181273ba90b1`. GHCR login is pinned at `docker/login-action@81c6f93d4bc991e510b342a00b419b3bb4efc5c6`, with registry `ghcr.io`, username `github.actor`, password `secrets.GITHUB_TOKEN`. Build/push is pinned at `docker/build-push-action@53b7df96c91f9c12dcc8a07bcb9ccacbed38856a`. Public GitHub commit reads verified all three commit identities; comments `# v7`, `# v4.7.0`, `# v7` remain descriptive labels, not floating refs.

The build action receives local context `.github/telegram-bot-api`, `push: true`, the single build argument `TELEGRAM_BOT_API_REF=${{ inputs.ref }}`, and one ref-shaped mutable tag `ghcr.io/${{ github.repository_owner }}/telegram-bot-api:${{ inputs.ref }}`. The context tree contains only the already-read Dockerfile and entrypoint. `DOCKER_BUILD_RECORD_UPLOAD=false` disables the action's default build-record upload. This YAML does not declare cache, platform, Dockerfile override, target, secret build inputs, explicit SBOM/provenance setting, timeout, concurrency, retry, or fallback.

On success, step id `push` exposes `outputs.digest`; the last shell step prints a digest-pinned reference `ghcr.io/${{ github.repository_owner }}/telegram-bot-api@${{ steps.push.outputs.digest }}` and instructs the operator to set repository variable `CANARY_BOT_API_IMAGE`. There is no API call or workflow command here that mutates the repository variable. Manual handoff is therefore an explicit lifecycle/security boundary.

## Callers and consumers

Declared caller is a manual GitHub `workflow_dispatch` event. Repository source search for `canary-bot-api.yml` found only the Dockerfile provenance comment and `canary.yml` consumer/provenance comments, not a programmatic `workflow_call`/dispatch wrapper. Any out-of-repository API/UI dispatch remains external operator authority.

Direct build-context consumers are Docker BuildKit through build-push-action plus exact Dockerfile/entrypoint files. Dockerfile consumes `TELEGRAM_BOT_API_REF`, clones `https://github.com/tdlib/telegram-bot-api.git`, checks out that ref, recursively initializes submodules, builds and installs the binary, then composes the non-root runtime image.

Exact fixed-U image consumer is `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`. Its version job reads `vars.CANARY_BOT_API_IMAGE` and, when publishing, rejects anything not matching a digest-pinned GHCR regex. Its win64/mac/linux publish jobs have `packages: read`, use the variable as a `telegram-bot-api` service image, authenticate to GHCR with actor/GITHUB_TOKEN, inject `CANARY_API_ID`, `CANARY_API_HASH`, `TELEGRAM_LOCAL=1`, publish 8081, and address the service at `http://localhost:8081`. The next independent fixed-U source entry remains this full `canary.yml` file; reading its relevant consumer ranges here does not close its independent ledger entry.

Exact downstream publication consumer is `Telegram/build/canary_publish.sh@4eb2d790063bb8f11efc7ffa06977fe1ef79a270`, which owns bounded Bot API readiness and publication. The image-build workflow owns neither service readiness nor publish retries.

## Failure, recovery, security, quality

Step dependency is fail-closed by default success ordering: validation failure prevents checkout; checkout failure prevents login/build; login failure prevents build/push; build/push failure prevents the digest-print step. There is no explicit retry/backoff or recovery branch. A new deliberate dispatch is the recovery mechanism represented by source structure. If digest handoff is omitted/stale, the consumer's digest validation or service startup must fail rather than fall back to a tag or third-party image.

`contents: read` and `packages: write` bound the builder token surface. The workflow passes GITHUB_TOKEN to the login action but not as a Docker build argument. The custom Depot runner and external `canary` environment configuration are privileged infrastructure boundaries because they receive a package-write token and perform registry publication; their actual protection/ref admission is not encoded in this file and is deferred to frozen-candidate evidence. The source comment's intended environment restriction is recorded as intent, not accepted as runtime proof.

The ref-shaped GHCR tag is mutable even though it names a source commit. The digest emitted by the successful push is the immutable consumer identity and must be the only publication trust anchor. Same-ref rebuilds can legitimately yield a different digest because the Dockerfile includes build-time package/acquisition inputs; therefore tag equality never substitutes for digest equality.

## Existing-owner-first disposition

Target owner: existing Fabushi repository build/container/toolchain provenance + dependency/supply-chain quality + repository process/security governance + shared quality/evidence/release gates. Rejected owners: Bot API sidecar/provider/runtime, Telegram-specific workflow owner, second package/release service, product Settings/Permissions credential owner, Coordinator/Host/Runner runtime, platform-core service, or second security/evidence truth. There is no product UI responsibility and no platform-core production delta.

Stage-A authored cases: CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-DISPATCH-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-REF-VALIDATION-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-PERMISSIONS-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-ENVIRONMENT-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-CHECKOUT-PIN-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-GHCR-AUTH-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-BUILD-CONTEXT-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-BUILD-ARGS-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-IMAGE-TAG-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-DIGEST-HANDOFF-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-SOURCE-LINEAGE-001, CONTRACT-TDRP-CANARY-BOT-API-WORKFLOW-CONSUMER-001, INT-TDRP-CANARY-BOT-API-WORKFLOW-REPOSITORY-PROCESS-001, FAULT-TDRP-CANARY-BOT-API-WORKFLOW-INVALID-REF-001, FAULT-TDRP-CANARY-BOT-API-WORKFLOW-CHECKOUT-OR-LOGIN-001, FAULT-TDRP-CANARY-BOT-API-WORKFLOW-UPSTREAM-REF-UNRESOLVED-001, FAULT-TDRP-CANARY-BOT-API-WORKFLOW-BUILD-PUSH-001, FAULT-TDRP-CANARY-BOT-API-WORKFLOW-DIGEST-HANDOFF-001, SEC-TDRP-CANARY-BOT-API-WORKFLOW-TOKEN-SCOPE-001, SEC-TDRP-CANARY-BOT-API-WORKFLOW-ACTION-PINS-001, SEC-TDRP-CANARY-BOT-API-WORKFLOW-INPUT-BOUNDARY-001, SEC-TDRP-CANARY-BOT-API-WORKFLOW-RUNNER-TRUST-001, SEC-TDRP-CANARY-BOT-API-WORKFLOW-DIGEST-TRUST-001. All remain not run and unaccepted.

## Stage-A execution statement

No Actions/check/status/workflow-run query or trigger, workflow dispatch, checkout, shell/container execution, Docker/BuildKit, registry login/push/pull, Bot API call, validator, build, test/E2E, package, signing, release, deployment, installation, runtime acceptance, or executable source-lineage verification was performed. Only Git/API/source reads plus contract/evidence authoring and safe checkpoint publication are in scope.
