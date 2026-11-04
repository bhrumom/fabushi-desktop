# Round 193 evidence — fixed-U canary release/publication workflow

Task round: 162
Responsibility: `TDRP-R9-REPOSITORY-CANARY-RELEASE-WORKFLOW-001`
Stage: A
Validation: not run
Accepted: false

## Authority

- Desktop checkpoint at round start: `4c36032efb2ebff2402d4ce055f850982c600053`
- Spec-first commit: `48dd946972afdaab1a1221bb50b02db7b7cee05d`
- Desktop main observed: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`
- Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
- Fixed-U recursive tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
- Source: `.github/workflows/canary.yml@0d460c617c2a3910a0d11ac793d07533ac966947`, tree index 118 zero-based, mode 100644, tree size 77,724 UTF-8 bytes, decoded 77,716 code points / 1,707 LF / 0 CR / trailing newline
- Read classification: full-read; earlier Bot API rounds read only consumer ranges.

## Verified source contracts

The workflow is push-only for `public-canary` and `private-canary`, serializes publication with `canary-publish` and does not cancel an in-progress run. The version job verifies the event/ref, current remote branch tip, repository-visibility/lane match, three-component version and nonnegative counter base. Publication is token-gated; when enabled it requires channel id, three platform metadata ids and a digest-pinned `ghcr.io/...@sha256:<64 lowerhex>` Bot API image. Sibling fixed-U platform/Snap workflows merely exclude canary branches; no workflow-call caller was found.

All jobs use `environment: canary`. Source permissions are contents-read globally, actions-read only for the version lookup, OIDC id-token-write only for Windows/macOS-universal/Linux update-signing jobs, and packages-read only for publisher jobs. Runtime environment protection/approvals and Depot runner trust were not inferred from source.

Windows, macOS and Linux build/sign/package/artifact/publication chains were read completely, including exact helper identities, platform signing/notary boundaries, Azure Key Vault v2 update-envelope signing, transient artifact producer/consumer names and retention, explicit upstream Linux update-trust checks, digest-pinned Bot API service consumption and exact downstream `canary_publish.sh` bounded readiness/publication ownership. All ten external Action references resolve at the exact full SHA recorded in the dossier/quality contracts.

Source limitations are preserved as limitations: Breakpad symbol R2 uploads are TODO comments only; Windows KeyLocker tool installation is TODO; the Linux-side mac publish witness checks only `_CodeSignature` directory presence while stronger codesign/notary/staple/spctl checks happen build-side. Full-SHA Actions do not freeze apt/Homebrew/NuGet/submodule/cppgir-mirror/custom-runner/remote-service inputs.

## Target owner disposition

Applicable guarantees map to existing `desktop/package.json@3d500622ce9b26e3452d98d0c4a7e7dcc019ac84`, `.github/workflows/release-macos-main.yml@b0a3a4d5503e6104c083f85e31475523a2b30643`, `desktop/scripts/notarize-after-sign.cjs@5f140c6ff8587b4969e1611ec9030a8c5dc5a6f1`, existing updater/release composition, root repository process/security governance and shared Test Strategy/Evidence Contract. No source workflow/service copy or platform-core delta is required.

## Authored-only tests

- Contract: 22
- Integration: 1
- Fault/recovery: 9
- Security: 9
- Execution evidence: none
- `workflow_run=null`
- `validation_status=not-run`
- `accepted=false`

## Closure

Unique ledger coverage advances from unknown 16116 / unread 16089 / responsibilities 162 to unknown 16115 / unread 16088 / responsibilities 163. Omitted remains 0; unmapped responsibilities remain 0. Next independent fixed-U entry is `.github/workflows/cant-reproduce.yml@8e20e07349b5b43277f5371a4e931066bece4186` at tree index 119.

No Actions/check/status/workflow-run query or trigger, source canary push, build, test, package, signing, notarization, service start, Bot API request, publication, deployment, installation or runtime acceptance occurred in this round.
