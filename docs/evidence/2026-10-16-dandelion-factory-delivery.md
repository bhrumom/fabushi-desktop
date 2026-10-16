# Global Dandelion factory-backed Desktop delivery ledger

Date:2026-10-16. Status: incomplete; this record does not approve a public Desktop
release or full native Global Dandelion replacement. All executable verification
below runs in GitHub Actions, never the local desktop.

## Accepted immutable package and historical runtime

- Approved marketplace release: org.fabushi.global-dandelion0.1.0, proprietary,
  Desktop offline packet-workbench preview only; no native LAN/WAN/hotspot claim.
- Original signed fabapp453093bytes, SHA256
  de10e02115415908f827723457817e51cb4f39aa5ca68798da824fecc483118a.
- Existing public proof:
  https://api.ombhrum.com/v1/extensions/install/web-miniapp/bhrumom/global-dandelion/0.1.0
- Historical installed production lifecycle run38830168249 succeeded with binary
  source e62ea0f0f335a5acbbc90852e1f2ba75cab03424 and harness
  4cf6215d7dbdbcc440f18a32fd76e13c84bc2f66. Actual Wasm/report, OS-network-disabled
  whole-app cold restart, preference retention, remove-code/window close and
  online reinstall were observed. This does not accept later product source.
- Historical evidence artifact12016473349, archive SHA256
  1db619af4df126759670b7cd390e677a776db505d5cc0dad027d602bb1b35939.

## Factory consumer integration

Backend PR105 merged as f2c60fc282e8c6a0457856b467515479b965d12b.
Its reviewed head c826b7f94bd3c48d7f4dc0503fda56f6af388668 passed Actions
38833916527:83 tests,83 passed,0 failed,0 skipped, exit0. Main factory-contract
run38856031216 also succeeded. These are contracts/build validation, not the
actual signed Desktop consumer run.

The existing authorized main-only cloud workflow has a consumer mode. No OIDC
allowlist, factory Worker, issuer, production credentials or cloud deployment
secret changed. Explicit evidence allowlist, job-scoped artifact-read permission,
two delivered sessions' independent401 cleanup checks and RAM/FIFO env alignment
were reviewed. Factory deployed source remains4ccf4f31ce035b19a0b697828f091d320e04b77b,
Core c7d0e24b7561c6f5d1bb3c0e975f599643252504; no new runtime acceptance is inferred.
Canonical Account Worker Runtime run38856031221 also succeeded on Backend main;
this is build/runtime CI, not a new leased signed-Desktop acceptance.

## Current Desktop source and unresolved checks

PR64 merged canonical main c33a9495fc1e640557d65f3e2daedc1449be7fcb into its
branch at6fe39a6cacff96ae03bb0c1f180b7ae0e52b4c0d, preserving FDN/Computer and
TDRP main-tree changes. New native issuer qualification/RAM storage, factory UI
harness and immutable same-byte promotion are implemented, not finally accepted.

Exact6fe source passed renderer, native intents, publisher and promotion/Apple
contracts. Builds failed resolving withdrawn zip2.4.2; verifier also reported
formatting. Follow-up2735a21887dc39e20a169c12cc6e802a775682ca pinned Host zip9.0.3
and applied formatter diagnostics. Actions then exposed the same withdrawn
dependency in the Mahayana plugin workspace, Linux native Computer prerequisites,
and zip9's fallible name API. Follow-up097209df21416551da266ce5ce5f19db9d065e46
addresses those without relaxing archive policy. Its exact-source checks are
in progress: all three OS pressure jobs and renderer/native/publisher/promotion
checks passed, while verifier found missing test-module imports (four E0425
errors), now repaired locally. No zero-test issuer filter is accepted: its
focused command must name the actual native_account_scope_tests module.
Private candidate38856357296 failed before packaging due that workspace
dependency; it is not a signed/accepted artifact.

Actions38856292706 ran the real signed Dandelion offline codec successfully, but
the broader actual Electron isolation case timed out after intentional renderer
crash. The whole job failed; do not report sandbox acceptance as passing.

Independent review additionally required explicit CI issuer provenance and
logout/current-status race fencing. These repairs must pass new exact-source
contracts before integration; historical green checks do not cover them.

## Requirement-to-evidence review

| Criteria | Status and evidence |
| --- | --- |
| DFC-01 | implemented/reviewed; PR105 and83-test CI; actual consumer not yet run |
| DFC-02 | contract-verified atc826; canonical-main signed candidate still required |
| DFC-03 | existing accepted factory pinned; new lease consumer runtime blocked |
| DFC-04 | RAM/FIFO delivery and child-env contracts pass; signed runtime unproven |
| DFC-05 | blocked: actual signed A/B/A UI run required |
| DFC-06 | both-session cleanup contracts pass; actual remaining0/revocation required |
| DFC-07 | explicit safe upload reviewed and contract-verified |
| DFC-08 |83-test Actions evidence; not a packaged acceptance substitute |
| MFA-01 | Backend integrated; authorized cloud consumer runtime still required |
| MFA-02 | issuer/scope implemented; explicit-bootstrap repair and current CI pending |
| MFA-03 | fixed public catalog/no bearer and issuer transport denial implemented; current CI pending |
| MFA-04 | normal storage owners on RAM implemented; actual signed path proof pending |
| MFA-05 | blocked: genuine logout, A/B isolation and A retention on final binary |
| MFA-06 | factory cleanup implementation reviewed; actual zero-residue proof pending |
| MFA-07 | source/artifact binding implemented; final canonical-main binary missing |
| MFA-08 | same-byte promotion contracts pass; actual production/factory gates missing |
| DCI-01..04 / AC-1 | blocked for current source: new ZIP dependency requires complete adversarial/interoperability CI |
| DCI-05..08 / AC-2 | blocked: current Host/actual sandbox, account and installed lifecycle gates required |
| DCI-AC-3 | marketplace approved; final installed canonical-main Desktop acceptance missing |
| MFA-AC-1..4 / DFC-AC-1..3 | incomplete; no public Desktop promotion authorized by evidence yet |

Next required sequence: fix current-source findings; exact-source CI; integrate
Desktop to canonical main; private signed main-push package with production chat
and MU-07; existing Backend factory consumer on that same source/archive; finally
zero residue and both-token rejection; verify and promote original accepted bytes.
No rebuild may substitute for the accepted artifact. Native hardware/hotplug,
network transfer and independent-human gates remain separate and unclaimed.
