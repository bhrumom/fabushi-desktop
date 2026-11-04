# Round 192 evidence — repository canary Bot API image-build workflow

Task round 161. Spec-first commit `b94fe59a8afb3f51a8fc8e92c6774afb5aca7748`. Fixed-U source `.github/workflows/canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9`, recursive-tree index 117 zero-based, mode 100644, 2,364 Git bytes/code points, 68 LF, 0 CR, trailing newline.

Complete semantic read confirms sole manual `workflow_dispatch(ref)`, exact lowercase-40-hex shape validation, workflow `contents: read` + `packages: write`, `depot-ubuntu-latest-32`, `environment: canary`, full-SHA checkout/login/build actions, actor/GITHUB_TOKEN GHCR auth, exact two-file `.github/telegram-bot-api` context tree, sole `TELEGRAM_BOT_API_REF` build arg, owner GHCR ref-shaped tag, build-record suppression and post-push digest output. Ref validation proves syntax only; upstream existence/submodules belong to Dockerfile acquisition.

Digest handoff is explicitly manual: the workflow prints `ghcr.io/<owner>/telegram-bot-api@<digest>` and instructs an operator to set `CANARY_BOT_API_IMAGE`; it does not update the repository variable. Exact fixed-U `canary.yml` consumer validates digest syntax and its win64/mac/linux publish jobs pull that service with package-read credentials and runtime API ID/hash/local mode. Exact downstream `canary_publish.sh` owns bounded readiness/publication. Mutable tag is not accepted as the consumer trust anchor.

Existing-owner-first maps responsibility `TDRP-R9-REPOSITORY-CANARY-BOT-API-BUILD-WORKFLOW-001` to existing repository build/container/toolchain provenance + supply-chain/process/security + shared quality/evidence/release gates. No target source-workflow copy, Bot API sidecar/provider, product runtime, second release/updater service or platform-core delta. Twelve contract, one integration, five fault-recovery and five security cases are authored/not-run; `accepted=false`.

One source entry closes: unknown 16117->16116, unread 16090->16089, responsibilities 161->162, omitted 0, unmapped 0. Next independent source entry is `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947` at index 118.

No Actions/check/status/workflow-run query or trigger, workflow dispatch, checkout, shell/container execution, Docker/BuildKit, registry operation, service start, Bot API request, validator, build/typecheck/lint/test/E2E, packaging, signing, release, deployment, installation or runtime acceptance was executed.
