# Exact-source dossier: orders 5031-5033

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Read method: direct exact-blob semantic reading of the remaining `main/session` declarations and presentation implementation. Read-complete records source understanding only and does not imply production implementation or release closure.
- Accounting after batch: read-through 5,033; unread 11,087; unknown 15,841; unknown-closed 279; omitted 0.

## 5031: Send-as identity declaration and cache/update contract

- Source: `Telegram/SourceFiles/main/session/send_as_peers.h@df09f6b4616cbf31cbf958c0523dea2df74701ff`
- Canonical owner: Canonical message send identity/permissions owner.
- State: message/paid-reaction/video-stream context -> cached eligible identities/default -> TTL/rights refresh -> canonical chosen identity -> mutation/update.
- Lifecycle: one Session-owned cache, last-request timestamps, chosen identity map and update stream share one lifetime.
- Failure boundary: stale rights/default, expired cache, missing peer, premium/paid-reaction eligibility drift or failed server mutation.
- Disposition: `mapped-open-send-identity`; implementation and verification remain open.

## 5032-5033: Session-scoped presentation and frozen-account guard

- Sources:
  - `Telegram/SourceFiles/main/session/session_show.cpp@1f740d441385950d8dbcdab4761e9bf626cec9f7`
  - `Telegram/SourceFiles/main/session/session_show.h@16f8e6f9136725de93615ef54366d67579da54c9`
- Canonical owner: Canonical session-scoped Dialog/Toast presentation owner.
- State: exact Session + live presentation host -> delegated layer/box/toast operations -> host validity -> frozen-session guard -> canonical frozen-info dialog or normal continuation -> teardown.
- Product invariant: presentation must remain bound to the exact active session; the UI layer must not become a second Session owner.
- UI mapping: reuse canonical Dialog/AlertDialog/Toast/Button/IconButton surfaces, preserving focus/a11y and session validity rather than creating Telegram- or SessionShow-named public components.
- Failure boundary: stale host, wrong session binding, frozen guard bypass or post-teardown presentation.
- Disposition: `mapped-open-session-presentation-guard`; production/UI/a11y/integration/E2E verification remains open.

## Accounting and acceptance

All three entries are exact-tree/path/blob bound with `read_complete=true` and responsibility decomposition recorded. `unknown` remains 15,841, `omitted` remains 0, and no implementation or release state is promoted. A current exact-head GitHub Actions Source authority run must attest this new shard and manifest.
