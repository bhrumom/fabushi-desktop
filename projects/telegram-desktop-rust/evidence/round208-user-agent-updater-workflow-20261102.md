# Round 208 evidence — fixed-U repository user-agent updater workflow

Date: 2026-11-02
Task round: 177
Desktop input checkpoint: `b23ea7be65bf659e8830cd0f92bf3de24c4df2fe`
Desktop main reread: `6f770fb00e9f74f787a49a05b82eadebe585d75b`
Platform-core: `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Fixed-U tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`, non-truncated, 7,194 entries
Source: `.github/workflows/user_agent_updater.yml@1e3dae8182aefc4e4c76df065f547a43a1e50437`
Tree index: 133 zero-based
Responsibility: `TDRP-R9-REPOSITORY-USER-AGENT-UPDATER-WORKFLOW-001`

Exact source and observed Action lineage were reread. The workflow admits repository dispatch and monthly `0 0 1 * *` only; no manual dispatch; mutable `ubuntu-latest`; no explicit permissions/environment/concurrency/timeout/retry/checkout; mutable `desktop-app/action_code_updater@master` type `user-agent`. All 20 fixed-U workflow blobs were checked and no sibling carries this dispatch/type contract.

Observed Action master is `0a8d3ebb437e5c77cf5d066d4a54cc93d34b534a`, unprotected/unsigned, with action/runtime blobs recorded and no LICENSE/package manifest/lock. It installs unpinned npm dependencies and passes token on argv. UserAgent gets runner Chrome version, derives major.0.0.0 with unescaped-dot regex semantics, clones current default branch, scans broad text excluding substring-.git paths/binaries, mutates matching Chrome tokens, pushes a derived branch and best-effort PR. Partial publication and mutable runtime identities remain fail-closed.

Fixed-U product consumer linkage is `DnsUserAgent()` in `mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9` plus DomainResolver and `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` header use. Existing-owner-first keeps source maintenance under repository process/security/quality and runtime DNS behavior under canonical networking; no duplicate updater/runtime or platform-core delta.

Authored only and not run: 28 contract, 1 integration, 15 fault/recovery and 12 security/provenance cases. Ledger closure: source entries 16,202; unknown 16,101 -> 16,100; unread 16,074 -> 16,073; omitted 0; responsibilities 177 -> 178; unmapped 0. Next entry: `.github/workflows/waiting-for-answer.yml@0cccca6b2d434303da915b12a13e7620cf1b8bc4`, index 134. Existing upstream.lock/inventory identity remains older `351204f...`. No CI/Actions/check/status query or executable validation was performed.
