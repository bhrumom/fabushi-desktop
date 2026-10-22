# Canary release workflow complete read — strict index 80

## Authority
- Upstream: `telegramdesktop/tdesktop@e14f1566d277d2204f4c8e553bf7cc157047e88b`, tree `241f8ab614292cc90e395ef0c966e289fdb3e985`.
- Source: `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`, 1708 line slots / 77724 bytes; complete read and SHA revalidated immediately before publication.
- Parent Fabushi exact head/tree: `8f2347f579f86208226bd0c73ded67b16dae8213` / `0fb8661db742d1c8a718e70f973d5163ec873d8e`.
- Existing policy owner: `AGENTS.md@da3e37258c74963970ff5cfc5d54671a37558b97` — Repository release-preparation governance plus Repository release-support container provenance governance.
- Existing execution/evidence owners: `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`, `FQT-REL-001@c5eb9d0897234bb2fdad4f505549b2350c44aa72`, `FQT-EVID-001@2d1d709a6ac3204bd05bedf5b3e98f37856e8c70`.
- Parent TDRP run `39758423395`: 61/61 success; authority `12417620767` / `sha256:0b0bfea8aae8fcf1b840748d5e0134b5ac4ebb54406ab58130e4254c2154222e`; closure `12417496834` / `sha256:03b1f75888572529b62040fb684281bd63669583ce87c96b465ac5585f1e5ead`; reachability `12416569182` / `sha256:6cb8aecf64716c7e006a1e36e7ee8474b8adacc5d26bd94e9b7be302ddd1829c`.
- Parent Rust run `39758423417`: 6/6 success; artifact `12416941871` / `sha256:40b48cb9baddce151bd325b9ae4d04232e8150a57026c946401338be59920ff0`.

## Complete-read responsibility decomposition
The workflow is a repository release adapter, not a shipping Telegram/Fabushi runtime. Its applicable source-neutral responsibilities are:
1. Admit only an eligible canary branch whose checked-out SHA is still the remote branch tip; reject historical reruns before they can become newer versions.
2. Validate public/private lane identity, canonical three-component version metadata and a non-negative counter base; derive monotonic release identity from `GITHUB_RUN_NUMBER + CANARY_COUNTER_BASE`.
3. Use reviewed, commit-pinned Actions and exact source/submodule inputs for platform builds.
4. Build each platform independently, preserve debug/update-trust evidence, require the applicable signing/notarization path, and package update plus portable artifacts.
5. Upload platform outputs as separately named immutable Actions artifacts and let each publish job download only its matching artifacts.
6. Re-check the applicable platform/provenance witness immediately before publication.
7. Consume the strict79 release-support image only by immutable reviewed digest and keep registry/API/bot/signing/notary credentials inside the existing job/environment permission boundary.
8. Preserve per-platform failure visibility: a successful platform cannot convert another platform's failure into whole-release acceptance.
9. Keep final release acceptance with Fabushi's existing release/FQT owners.

The source's Telegram channel IDs, Bot API transport, public/private channel distribution, KeyLocker/Azure provider names, executable/update names and `CANARY_ALLOW_UNSIGNED` bring-up escape hatch are provenance. In particular, Fabushi does not adopt the unsigned escape hatch as a release bypass.

## Direct dependency anchors
The workflow's directly invoked source inputs were re-resolved at the same upstream commit:
- `Telegram/build/version@2e6d6a21a175c02188038f4f9e3acae4300ddff7`
- `Telegram/build/sign_update.py@9e3e2e9e1b716db6a15a36b949bf53d3f650777d`
- `Telegram/build/canary_test_fixtures.sh@88ae0c43b8dc24538f6347819f5f4f904c0a0a49`
- `Telegram/build/canary_publish.sh@4eb2d790063bb8f11efc7ffa06977fe1ef79a270`
- `.github/workflows/canary-bot-api.yml@7285f29ed2a09695a02b8f7821c10df5aa50bc44`

Those scripts remain independent inventory entries. Their SHA anchoring here proves the strict80 dependency edge; it does not mark those later entries read or migrated.

## Build, signing and publish oracles
- The version job runs only for the two declared lane branches, requires the current remote tip, validates lane/version/counter inputs, and requires a digest-qualified GHCR service image before publish.
- Windows signs binaries and immediately verifies them, signs the v2 update envelope through the declared signing path, and uploads separate update/portable artifacts. The publish job re-verifies the portable's Authenticode signature.
- macOS transports two architecture halves, checks toolchain compatibility before universal assembly, codesigns/notarizes/staples, signs update envelopes and uploads separate update/portable artifacts. The publish job witnesses the code-signature directory; the source itself records a TODO for fuller Linux-side verification.
- Linux builds in its declared build image, runs the update verifier and canary fixtures, signs the update envelope when publishing, and uploads separate update/portable artifacts.
- Each publish job is downstream of its matching build, uses `contents:read` + `packages:read`, downloads the exact named artifacts and runs the shared publish script with the exact version/counter/commit/provenance inputs.
- The digest-shaped-image regex is necessary but not sufficient under Fabushi policy: strict79 already requires the configured digest to be the exact image that was built and reviewed, not merely any syntactically valid GHCR digest.

## Existing-owner-first disposition
No new owner is required. Fabushi already has:
- repository release-preparation governance with exact-source, version, mutation and acceptance boundaries;
- an exact-source macOS packaging workflow that binds checkout SHA, requires signing credentials, codesigns/notarizes, proves raw candidate executable SHA-256/size stability across installed copying, runs packaged acceptance and retains immutable Actions artifacts;
- FQT-REL-001 release entry/exit and artifact-identity gates;
- FQT-EVID-001 exact-head/run/job/artifact evidence and copy/transform identity requirements;
- strict77/79 release-support container/image provenance governance.

This strict mapping therefore creates no Telegram canary runtime, updater, Bot API provider, second release manager, signing authority, messaging service, artifact store or acceptance owner. `novel_capability=false`.

Current Fabushi evidence is strongest for macOS. The source workflow's Windows/Linux responsibilities are mapped to the same canonical release owner but are not falsely marked implemented or verified for Fabushi Windows/Linux by this audit. Actual platform release implementation and exact-head packaged evidence remain required independently.

## Fail-closed and security conclusions
Non-passing conditions include stale branch tip, invalid lane/version/counter, mutable/unpinned inputs, dependency/build failure, signing/notary/trust-chain failure, missing/cross-wired artifacts, unreviewed digest substitution, credential-boundary escape, hidden per-platform failure, and any attempt to interpret source canary success or unsigned bring-up as Fabushi release ACCEPT.

The workflow's permissions are bounded by responsibility: source/version reads, OIDC only where signing needs it, and registry package read only in publish jobs. Secret-bearing publication remains an adapter under existing release/security owners.

## Exact traceability IDs
- Requirement: `TDRP-R9-CANARY-RELEASE-PIPELINE-PROVENANCE-001`
- Oracles: `ORA-TDRP-CANARY-EXACT-TIP-001`, `ORA-TDRP-CANARY-MONOTONIC-VERSION-001`, `ORA-TDRP-CANARY-PLATFORM-ARTIFACT-001`, `ORA-TDRP-CANARY-PUBLISH-PROVENANCE-001`
- Invariants: `INV-TDRP-CANARY-NO-ROLLBACK-001`, `INV-TDRP-CANARY-NO-UNSIGNED-FABUSHI-RELEASE-001`, `INV-TDRP-CANARY-NO-SECOND-RELEASE-OWNER-001`
- Contracts: `CONTRACT-TDRP-CANARY-ARTIFACT-HANDOFF-001`, `CONTRACT-TDRP-CANARY-SIGNING-BOUNDARY-001`
- Fault/recovery: `FAULT-TDRP-CANARY-STALE-TIP-001`, `FAULT-TDRP-CANARY-SIGNATURE-001`, `FAULT-TDRP-CANARY-DIGEST-SUBSTITUTION-001`
- Security: `SEC-TDRP-CANARY-SECRET-BOUNDARY-001`, `SEC-TDRP-CANARY-PINNED-ACTIONS-001`
- Regression: `REG-TDRP-CANARY-NO-TELEGRAM-RUNTIME-001`, `REG-TDRP-CANARY-EXACT-HEAD-GATES-001`, `REG-TDRP-CANARY-NO-UNSIGNED-RELEASE-DOWNGRADE-001`

Closing strict index 80 changes counters to unknown=16,223, unread=16,198, omitted=0, responsibilities=98. `baseline_identity_accepted=false` and `source_closure_ready=false` remain mandatory. Next strict root: index 81 `.github/workflows/cant-reproduce.yml@8e20e07349b5b43277f5371a4e931066bece4186`.
