# Release-support canary image workflow complete read — strict index 79

## Authority
- Upstream: `telegramdesktop/tdesktop@e14f1566d277d2204f4c8e553bf7cc157047e88b`, tree `241f8ab614292cc90e395ef0c966e289fdb3e985`.
- Source: `.github/workflows/canary-bot-api.yml@7285f29ed2a09695a02b8f7821c10df5aa50bc44`, 69 line slots including trailing newline / 2364 bytes; complete read and SHA revalidated immediately before publication.
- Existing owner: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` — Repository release-support container provenance governance.
- Direct build inputs: `Dockerfile@a361602441040028a3332c3732c4902d9c880122`, `entrypoint.sh@d40a7372efd8e5862e00f9b214b36a840688797e`.
- Direct downstream consumer: `canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`.
- Parent exact head/tree: `ea1ba98cdc69863e5ca595d488c9da903a3b678d` / `b9d9762180af86b2ec6433d96056de695a10a981`.
- Parent TDRP run `39756506131`: 61/61 success; authority `12415958133` / `sha256:1a523f19d54911965cd5b962c930586445f2b930cc9135ac30a7388b2dd5b92a`; closure `12415549488` / `sha256:e2aff7a4690ca8b58974dc958adb5e073028ca98a71069719f21e2d4b171c2c9`; reachability `12416800901` / `sha256:3cf4628f5080b7d778cfa5c639d6594a258a80ee8b9a4de727006130789793a3`.
- Parent Rust run `39756506145`: 6/6 success; artifact `12415568854` / `sha256:57ffc4fb1e896b99ae9f3c4d62a58dc94d2549164408d929ed9fb95e3c54e1c0`.

## Complete-read responsibility decomposition
The workflow is an image-provenance adapter, not a product runtime. Its dispatch accepts an external source ref, rejects anything except full lowercase 40-hex, uses only `contents: read` and `packages: write` under the `canary` environment, pins checkout/login/build-push actions to full commits, builds the reviewed local `.github/telegram-bot-api` context with `TELEGRAM_BOT_API_REF`, pushes a source-ref tag for discovery, and emits the actual pushed image digest as a digest-qualified GHCR reference.

The source-specific image/repository/GHCR names remain provenance. The applicable responsibility is the chain `exact external commit -> reviewed build context -> immutable pushed digest -> exact downstream digest consumption`.

## Consumer and security audit
Exact upstream code search for `CANARY_BOT_API_IMAGE` at the locked commit returns only this workflow and `canary.yml`. The latter shape-validates `ghcr.io/...@sha256:<64hex>` and starts the same configured image in the three platform publish jobs, where API credentials, bot token and release files cross the container boundary.

The upstream build workflow prints its own repository-owner digest, but the downstream regex alone does not prove that a later configured digest came from this reviewed run. Fabushi's existing owner is deliberately stronger: downstream release jobs must consume the exact digest-qualified image that was actually built and reviewed. A digest-shaped third-party or unrelated image is therefore non-passing even if its string matches the upstream regex.

## Existing-owner-first disposition
Strict77 already established the source-neutral owner for exact external refs, immutable image provenance, bounded permissions/secrets and exact digest handoff. Strict79 extends no ownership surface and creates no Telegram Bot API runtime, GHCR release authority, updater, messaging service or storage root. `novel_capability=false`.

## Traceability correction
While tracing the strict79 build argument into the strict77 Dockerfile, the live ledger's strict77 `source_symbols` was found to say `ARG TELEGRAM_BOT_API_VERSION`. The exact Dockerfile says `ARG TELEGRAM_BOT_API_REF`, and strict79 passes `TELEGRAM_BOT_API_REF=${{ inputs.ref }}`. This commit corrects only that stale source-symbol spelling; strict77 responsibility, owner, disposition and evidence remain unchanged.

## Exact traceability IDs
- Requirement: `TDRP-R9-RELEASE-SUPPORT-IMAGE-PUBLISH-PROVENANCE-001`
- Oracles: `ORA-TDRP-CANARY-IMAGE-REF-001`, `ORA-TDRP-CANARY-IMAGE-ACTIONS-PINNED-001`, `ORA-TDRP-CANARY-IMAGE-DIGEST-001`, `ORA-TDRP-CANARY-IMAGE-REVIEWED-PROVENANCE-001`
- Invariants: `INV-TDRP-CANARY-IMAGE-NO-MUTABLE-REF-001`, `INV-TDRP-CANARY-IMAGE-NO-TAG-IDENTITY-001`, `INV-TDRP-CANARY-IMAGE-NO-SECOND-RUNTIME-001`
- Contracts: `CONTRACT-TDRP-CANARY-IMAGE-REF-001`, `CONTRACT-TDRP-CANARY-IMAGE-DIGEST-HANDOFF-001`
- Fault/recovery: `FAULT-TDRP-CANARY-IMAGE-BUILD-PUSH-001`, `FAULT-TDRP-CANARY-IMAGE-DIGEST-DRIFT-001`, `FAULT-TDRP-CANARY-IMAGE-UNREVIEWED-SUBSTITUTION-001`
- Security: `SEC-TDRP-CANARY-IMAGE-LEAST-PERMISSION-001`, `SEC-TDRP-CANARY-IMAGE-SECRET-BOUNDARY-001`
- Regression: `REG-TDRP-CANARY-IMAGE-NO-TELEGRAM-RUNTIME-001`, `REG-TDRP-CANARY-IMAGE-EXACT-HEAD-GATES-001`, `REG-TDRP-STRICT77-ARG-REF-SYMBOL-001`

Closing strict index 79 changes counters to unknown=16,224, unread=16,199, omitted=0, responsibilities=97. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 80 `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`.
