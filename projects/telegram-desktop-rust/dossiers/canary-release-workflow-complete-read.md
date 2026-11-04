# Fixed-U complete read — .github/workflows/canary.yml

Status: Stage A source responsibility closed to implemented / validation not-run / accepted false
Responsibility: `TDRP-R9-REPOSITORY-CANARY-RELEASE-WORKFLOW-001`
Source: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4:.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`
Recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
Tree index: 118 zero-based
Mode: 100644
Git tree size: 77,724 UTF-8 bytes
Decoded text: 77,716 code points, 1,707 LF, 0 CR, trailing newline
Read classification: full-read; prior rounds read only consumer ranges for the Bot API chain, not this independent source responsibility.

## Disposition

This file is source repository release/build/sign/package/artifact/publication provenance. It is not copied into Fabushi and it does not authorize a Telegram-specific runtime, provider, Bot API sidecar, channel publisher, second updater/release service, second credential owner or platform-core delta. Existing-owner-first maps its applicable engineering guarantees to current Fabushi repository/package/release/process/security/quality owners: `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`, `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`, `desktop/scripts/notarize-after-sign.cjs@5f140c6ff8587b4969e1611ec9030a8c5dc5a6f1`, existing updater/release composition, root process/security governance, shared Test Strategy and Test Evidence Contract. Fabushi platform-core remains unchanged at `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`.

## Trigger, lane and admission

The sole trigger is push to `public-canary` or `private-canary`; no dispatch/call/schedule/PR/repository-dispatch trigger exists. Fixed-U reads of every other workflow show no workflow-call of this file. Windows, macOS, macOS-packaged, Linux and Snap workflows contain only comments plus `branches-ignore` routing that reserves these canary branches for this workflow. Concurrency is `canary-publish` with `cancel-in-progress: false`.

All jobs name GitHub environment `canary`. The version job also gates event/ref explicitly before secrets are requested. Lane defaults from repository visibility unless overridden, allows only public/private, refuses private lane in a public repository, and requires branch name `<lane>-canary`. Actual GitHub environment protection/ref rules, approvals and Depot runner trust are not in the source and remain Stage-B infrastructure evidence.

## Permissions and version state

Global permission is contents-read. Version adds actions-read; Windows/macOS-universal/Linux add id-token-write; macOS thin builds remain contents-read; each publish job has contents-read/packages-read. OIDC is therefore scoped to jobs that sign update envelopes.

The version job checks out full history and refuses stale reruns unless `GITHUB_SHA` remains the current remote branch tip. It reads exact `Telegram/build/version@2e6d6a21a175c02188038f4f9e3acae4300ddff7`, validates three-component `AppVersionStr`, exposes a seven-character commit id, validates `CANARY_COUNTER_BASE` as nonnegative, and computes counter = run number + base. Public/private suffix and portable folder are derived once.

`CANARY_BOT_TOKEN` presence alone turns publication on. If absent, platform build/package paths may proceed but publication is skipped. If publishing, channel id, three platform metadata ids and digest-pinned `CANARY_BOT_API_IMAGE` are mandatory. `CANARY_ALLOW_UNSIGNED=1` is an explicit bring-up override for platform signatures; it does not erase the v2 update-envelope signing distinction.

Previous successful branch tip is looked up through GitHub Actions REST with actions-read and excludes the current run. The command has `|| true`; network/API failure can yield empty previous-tip/changelog context. This is bounded degradation only.

## External Action lineage

Every external Action is full-SHA pinned and was independently resolved at the exact commit:
- actions/checkout `3d3c42e5aac5ba805825da76410c181273ba90b1`
- actions/cache restore/save `55cc8345863c7cc4c66a329aec7e433d2d1c52a9`
- Azure/login `7ddb5af1ef8758cf1353cf3b42f940aee27ba21c`
- actions/download-artifact `9000827ccba6bdab643e8b6fd33ac0654aef8333`
- jlumbroso/free-disk-space `ceedf095f4ec1a097402bc6bd80831f2e1a6fde6`
- docker/setup-buildx-action `37fe631027851001ddb9b187196cc803df7f5f0e`
- reproducible-containers/buildkit-cache-dance `5422eac04292c961a382e0f584ea0f03ad9da723`
- docker/build-push-action `53b7df96c91f9c12dcc8a07bcb9ccacbed38856a`
- actions/upload-artifact `043fb46d1a93c77aae656e7c1c64a875d1fc6a0a`
- Eden-CI/msvc-dev-cmd `1bd71f95d6f3d1b2b3395b335cc63bcdc90cf223`

Immutable Action identity does not freeze custom runner images, package indexes or source dependency acquisition.

## Windows lane

Windows uses depot-windows-latest-32. It constructs TBuild, masks and forwards a GitHub auth extraheader across the symlink checkout, initializes direct submodules non-recursively, derives lane-specific update key/channel metadata, derives Windows SDK and cache keys, ensures matching VS ATL, pins the msvc-dev-cmd action, uses NuGet network resolution, and explicitly restores/saves ThirdParty/Libraries/Qt caches.

The build creates throwaway v1 DesktopPrivate keys only because the Packer target expects those headers; v2 canary packing does not use them. Release x64 is built with LTO, Telegram API id/hash, auto-update/crash reporting, canary counter/commit and lane defines. Breakpad symbols are generated.

If KeyLocker credentials are absent, a publishing run fails unless unsigned bring-up is explicitly allowed. The source still has a TODO to install KeyLocker tools. If available, Telegram.exe and Updater.exe are signed and verified. Publishing uses Azure login and exact `sign_update.py@9e3e2e9e1b716db6a15a36b949bf53d3f650777d` to sign the Packer signing input before embedding the v2 signature. Build-only mode keeps an unsigned envelope. Update and portable are uploaded separately for seven days. The publisher rechecks portable Telegram.exe with osslsigncode unless unsigned mode is active.

## macOS lane

`macos-build` runs x86_64 and arm64 in parallel on Depot macOS with fail-fast. It uses system Xcode, Homebrew tooling, records exact Xcode/clang/swift/SDK/deployment-target data, hashes that record into cache identity, prepares libraries, builds thin Release canary bundles without LTO, emits Breakpad symbols, strips known binaries and transports each thin app/Packer/toolchain record as a one-day tar artifact.

`macos` requires both halves. It verifies toolchain records match, lipo-assembles Packer and the universal application, includes required Swift runtimes and rejects executable Mach-O files that lack x86_64. If signing credentials are missing, publishing fails unless unsigned bring-up is explicit. With credentials, all thin/universal bundles are Developer-ID signed, codesign verified, notarized, stapled/validated and assessed with spctl. Update envelopes are Azure-Key-Vault-signed when publishing. Final update and portable artifacts have seven-day retention.

The Linux publisher's macOS platform witness only checks that the extracted portable contains `Contents/_CodeSignature`; the source explicitly marks full rcodesign verification as TODO. Do not equate that weak publish-side check with the stronger build-side notarization witness.

## Linux lane

Linux uses depot-ubuntu-latest-32. It checks out submodules, recursively initializes the cmake tree and may rewrite the cppgir GitLab source through operator-provided `CANARY_CPPGIR_MIRROR`. It installs Jinja2 with apt, generates the centos environment Dockerfile via `gen_dockerfile.py@86d27c96e1f732a1faf3e85397ea3ba38e493a0d`, creates a local Buildx image with explicit caches, builds through `build.sh@00b5c9a2faf5e25fb981325f654d3458ce074a7d`, emits symbols/minidebug, and compiles update verification tests.

The workflow itself then runs `out/Release/test_update_verify` and exact `canary_test_fixtures.sh@88ae0c43b8dc24538f6347819f5f4f904c0a0a49` against the built Packer. This is an upstream executable quality guarantee only; Stage A does not run it. Azure Key Vault signing and update/portable artifact behavior matches the v2 publication model.

## Artifacts and publication

mac thin transport artifacts live one day. Final Windows/macOS/Linux update and portable artifacts live seven days. They are CI transport between source jobs, not immutable long-term release evidence by themselves.

Each platform publisher needs version plus only its own completed platform build, so a failing/slow unrelated platform does not gate already-satisfied independent publisher dependencies. Publishers run on Depot Ubuntu with contents-read/packages-read, consume the exact digest-pinned Bot API service image, authenticate GHCR with actor/GITHUB_TOKEN, inject API id/hash and local mode at runtime, map port 8081 and use localhost HTTP. They download only their named update/portable artifacts.

The final publication owner is exact `Telegram/build/canary_publish.sh@4eb2d790063bb8f11efc7ffa06977fe1ef79a270`, already completely read/qualified under the previous exact blob. It owns bounded readiness and actual Telegram Bot API publication. Each platform writes only its own fixed metadata message. The service image is built separately by exact `canary-bot-api.yml@3ef5e601948c8f3256464d0d31816eebf4ba41a9`; current workflow requires its digest, never the mutable builder tag.

## Secrets, network and mutable provenance

Actual source references cover Telegram API id/hash, bot token, GitHub token, Azure client/tenant/key-vault inputs, Windows KeyLocker inputs, Apple signing/notary inputs. R2 credentials/bucket are documented as infrastructure placeholders but no executable R2 upload is present. API id/hash are build inputs and also runtime service environment in publisher jobs; registry, signing and notary secrets remain job-specific boundaries.

Network/supply-chain edges include GitHub checkout/API/artifacts, GHCR, NuGet, apt, Homebrew, submodule hosts/cppgir mirror, Azure login/Key Vault, Apple notary and downstream Bot API/Telegram. Full-SHA Action pins do not freeze these mutable services/packages. Caches improve throughput but are not release evidence.

## Recovery and fail-closed boundaries

Hard failures include wrong trigger/lane/repository visibility, stale branch tip, malformed version/counter/config, unavailable required build inputs, platform build/assembly/package failure, required signing failure, malformed/missing Bot API digest, missing/expired artifacts and failed bounded service readiness. Missing bot token deliberately disables publication. Explicit unsigned bring-up is a modeled exception, not an implicit missing-secret fallback. Previous-run lookup failure may only reduce changelog history.

There is no general source job retry/recovery workflow. Recovery means a fresh qualified run with identities rebound. `cancel-in-progress=false` preserves the older in-progress publication rather than cancelling it.

## Source limitations

R2 symbol uploads are TODO comments only on all platforms. KeyLocker installation is TODO. macOS Linux-side publish verification is a signature-directory presence check with full rcodesign marked TODO. GitHub environment policies and Depot runner trust are out-of-file. None is marked implemented by this read.

## Target ownership

Existing target owner candidates:
- `desktop/package.json` for canonical Fabushi package/build targets and electron-builder platform outputs.
- `.github/workflows/release-macos-main.yml` for exact-source candidate/package/release gating.
- `desktop/scripts/notarize-after-sign.cjs` for current macOS signing/notarization.
- existing updater/release composition for update lifecycle.
- root repository process/security/provenance governance.
- shared TEST_STRATEGY/evidence contract/release gates.

Rejected: Telegram canary workflow copy, Telegram Bot API sidecar/provider, Telegram channel publisher, second updater/release/package service, product Agent/Coordinator/Host/Runner runtime, platform-core service, second credential/security/evidence truth.

No production source code or platform-core change is required for this repository-tooling source entry; the applicable engineering requirements are integrated into existing target governance and authored Stage-B qualification contracts.

## Authored-only cases

Contracts (22):
- CONTRACT-TDRP-CANARY-WORKFLOW-TRIGGER-LANE-001
- CONTRACT-TDRP-CANARY-WORKFLOW-CONCURRENCY-001
- CONTRACT-TDRP-CANARY-WORKFLOW-ENVIRONMENT-001
- CONTRACT-TDRP-CANARY-WORKFLOW-PERMISSIONS-001
- CONTRACT-TDRP-CANARY-WORKFLOW-TIP-FENCE-001
- CONTRACT-TDRP-CANARY-WORKFLOW-VERSION-COUNTER-001
- CONTRACT-TDRP-CANARY-WORKFLOW-PUBLISH-GATE-001
- CONTRACT-TDRP-CANARY-WORKFLOW-UNSIGNED-OVERRIDE-001
- CONTRACT-TDRP-CANARY-WORKFLOW-BOT-IMAGE-DIGEST-001
- CONTRACT-TDRP-CANARY-WORKFLOW-ACTION-PINS-001
- CONTRACT-TDRP-CANARY-WORKFLOW-WINDOWS-BUILD-SIGN-001
- CONTRACT-TDRP-CANARY-WORKFLOW-MACOS-BUILD-UNIVERSAL-001
- CONTRACT-TDRP-CANARY-WORKFLOW-MACOS-SIGN-NOTARY-001
- CONTRACT-TDRP-CANARY-WORKFLOW-LINUX-BUILD-TRUST-001
- CONTRACT-TDRP-CANARY-WORKFLOW-UPDATE-ENVELOPE-001
- CONTRACT-TDRP-CANARY-WORKFLOW-ARTIFACT-HANDOFF-001
- CONTRACT-TDRP-CANARY-WORKFLOW-PUBLISH-JOB-ISOLATION-001
- CONTRACT-TDRP-CANARY-WORKFLOW-BOT-API-SERVICE-001
- CONTRACT-TDRP-CANARY-WORKFLOW-PUBLISH-SCRIPT-001
- CONTRACT-TDRP-CANARY-WORKFLOW-SOURCE-LINEAGE-001
- CONTRACT-TDRP-CANARY-WORKFLOW-SECRETS-NETWORK-001
- CONTRACT-TDRP-CANARY-WORKFLOW-SYMBOL-PROVENANCE-001

Integration (1):
- INT-TDRP-CANARY-WORKFLOW-REPOSITORY-RELEASE-001

Fault/recovery (9):
- FAULT-TDRP-CANARY-WORKFLOW-LANE-REJECT-001
- FAULT-TDRP-CANARY-WORKFLOW-STALE-TIP-001
- FAULT-TDRP-CANARY-WORKFLOW-VERSION-CONFIG-001
- FAULT-TDRP-CANARY-WORKFLOW-BUILD-PLATFORM-001
- FAULT-TDRP-CANARY-WORKFLOW-SIGNING-REQUIRED-001
- FAULT-TDRP-CANARY-WORKFLOW-BOT-IMAGE-DIGEST-001
- FAULT-TDRP-CANARY-WORKFLOW-ARTIFACT-HANDOFF-001
- FAULT-TDRP-CANARY-WORKFLOW-SERVICE-READINESS-001
- FAULT-TDRP-CANARY-WORKFLOW-PREVIOUS-RUN-LOOKUP-001

Security (9):
- SEC-TDRP-CANARY-WORKFLOW-LEAST-PRIVILEGE-001
- SEC-TDRP-CANARY-WORKFLOW-ACTION-PINS-001
- SEC-TDRP-CANARY-WORKFLOW-RUNNER-ENVIRONMENT-TRUST-001
- SEC-TDRP-CANARY-WORKFLOW-OIDC-SCOPE-001
- SEC-TDRP-CANARY-WORKFLOW-SECRET-BOUNDARY-001
- SEC-TDRP-CANARY-WORKFLOW-DIGEST-TRUST-001
- SEC-TDRP-CANARY-WORKFLOW-UNSIGNED-OVERRIDE-001
- SEC-TDRP-CANARY-WORKFLOW-MUTABLE-ACQUISITION-001
- SEC-TDRP-CANARY-WORKFLOW-NO-SOURCE-PUBLISHER-IMPORT-001

No case was executed. No workflow run/status/check was queried. No build, test, packaging, signing, notarization, service, Bot API, publication, deployment, installation or runtime acceptance was performed.
