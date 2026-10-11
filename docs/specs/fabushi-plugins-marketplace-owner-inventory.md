# Fabushi Plugins / Marketplace — Current Owner Inventory and Registry Decision

Status: active Phase 0 authority record  
Parent: FPM-001, OMA-001  
Date: 2026-10-11  
Desktop authority: `bhrumom/fabushi-desktop@d575b4f77b6895eea4ecaced96e5c512964bf3ed`  
Observed Desktop MCP branch: PR #49 exact head `72abccaa377fa026e4ebec113d76a1ad31ceabbf` (**not merged/current-main authority**)  
Backend authority: `bhrumom/fabushi-backend@b55320f655584b999c08ca01ebff182fd04328d9`  
Platform Core authority: `bhrumom/fabushi-platform-core@3a125c4d3164db7119b8022f16062a502d54d28a`  
iOS observed implementation: PR #3 `8dbb92513600f046cfd178c15c9c94118dc5ec62`  
Android observed implementation: PR #3 `40e3a91cdce2146421c89b4dc023f6a2cb446d0f`

> This document is an owner/source inventory. It does not mark FPM-001 complete. Historical CI or code on unmerged PRs is observation-only until integrated and re-verified on its accepted exact HEAD.

## Registry architecture decision

### Decision: choose **B with selective C-style reuse**, not full Open VSX server adoption

Fabushi will implement the canonical typed Extension Registry inside the **existing Fabushi Backend Plugins/MiniApp Marketplace domain**, while reusing Open VSX/VS Code/Theia ideas and, where a later source/license/security review proves useful, narrowly reusable components or algorithms.

Do **not** deploy a second Open VSX Java registry as the product authority.

Reasons grounded in current source:

1. Fabushi Backend already owns a real registry-like domain in `ai-backend/src/miniapp_marketplace*.js`: strict manifests, immutable artifact metadata, review states, search/browse, publisher draft/submit/review/yank, account install projection and MCP exposure.
2. Fabushi needs three first-class kinds—`mcp-connector`, `skill`, `web-miniapp`—whose runtime/security contracts differ from VSIX/Node extensions.
3. Existing Desktop, Host/Coordinator and mobile code already own OAuth, account, permission and execution lifecycle. A second registry/runtime stack would create conflicting install/trust truth.
4. Open VSX is still the principal open registry benchmark for namespace ownership, publication, scanning, revocation and self-hosting. Those mature behaviors should be adopted where they fit, without coercing Fabushi artifacts into VSIX.
5. VS Code/Theia remain client/runtime architecture references, not mobile execution models.

This decision can be revisited only with an ADR that proves the existing owner cannot satisfy a specific new responsibility.

## Source-to-owner responsibility inventory

| Responsibility ID | Existing owner | Shipping entrypoint / production path | Data authority | Contract | Platform | Current status | Required change | Test oracle | Evidence / blocker |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| FPM-OWN-001 Marketplace shell | `ProductionRenderer.tsx` + recovered Plugins overlay | Desktop production renderer -> Plugins surface | renderer state + Host projections | typed desktop bridge | Desktop | partial | converge dialog/overlay into canonical first-level Marketplace route while keeping compatibility entry | route opens same installed/catalog state; no second shell | Desktop main source; top-level route not yet complete |
| FPM-OWN-002 Legacy catalog read | shared Node MCP marketplace/catalog flow + Cursor-compatible client | `mcp.catalog` / effective plugins projection | legacy remote catalog + local cache | existing Dashboard-compatible API | Desktop | implemented legacy / migration source | label source as legacy; adapt to canonical Extension Registry; preserve IDs | old IDs resolve to stable identity without changing installs | current main owner; final Fabushi Registry migration pending |
| FPM-OWN-003 Official MCP catalog | PR #49 shared catalog + Electron MCP service (**unmerged**) | Plugins UI -> Electron main -> Coordinator/Host/MCP | official provider definitions + account connection state | Streamable HTTP MCP + OAuth | Desktop | partial/unmerged | rebase/reconcile against current main and canonical registry identity | add/connect -> OAuth -> initialize -> tools/list -> tools/call | PR #49 head diverged from main; old CI not current evidence |
| FPM-OWN-004 MCP OAuth broker | Platform Core + Backend gateway + Desktop adapters | service auth start/callback/consume -> encrypted local vault | server OAuth transaction + account-scoped client vault | state/PKCE + broker delivery | Server/Desktop | partial | retain existing owner; connect catalog identity to stable Extension ID; re-run real provider acceptance | state replay/expiry/account switch/revoke and real tool call | Core #3 / Backend #13 merged; real provider/package acceptance still separate |
| FPM-OWN-005 MCP execution | existing Coordinator/Host MCP and local tool permission owners | Agent -> Coordinator -> Host -> MCP server tool | Host effective policy + MCP session | MCP initialize/list/call | Desktop | implemented baseline | ensure canonical registry install never bypasses Host approval; disable/revoke fences calls | disabled tool cannot execute; uncertain write is not auto-retried | existing owner must remain single runtime |
| FPM-OWN-006 Skills lifecycle | existing Plugins/Agent/Host Skill paths | existing plugin/skill APIs and Agent invocation | installed Skill config + team/private source | existing Plugin/Skill schema | Desktop/mobile | partial | map to `kind=skill`, stable identity/version/origin without granting higher privilege | create/install/enable/use/update/team publish/unpublish/resync | Backend canonical Skill catalog source still to be identified/wired |
| FPM-OWN-007 Mini App registry | Backend `ai-backend/src/miniapp_marketplace*.js` | existing `/v1/marketplace/*`, MiniApp MCP routes | `MiniAppMarketplace` + `AccountSyncStore` | `fabushi.miniapp.marketplace.v2`, manifest/release/install protocols | Server | implemented for current miniapp domain | extend owner to canonical typed Extension Registry, do not add parallel service | strict kind/trust identity, stable pagination, old routes regress green | Backend main exact source proves mature existing owner |
| FPM-OWN-008 Unified Extension Registry | same Backend Marketplace owner | new canonical `/v1/extensions` projection, then client adapters | server catalog metadata; Host remains install authority | `fabushi.extension-registry.v1` | Server/all clients | in progress | add typed read model first, then publisher/release/security write side | mcp/webminiapp projection; no fake skill/metrics; fail closed | Backend PR #15 is implementation-in-progress; not main evidence |
| FPM-OWN-009 Publisher/release governance | Backend Marketplace domain | publisher/admin APIs | server persistent publisher/release/audit store | future versioned registry write API | Server | partial | namespaces, roles, immutable versions, key rotation, scan/review/revoke/report/appeal | impersonation, typosquat, digest replacement, scan timeout all rejected | current MiniApp draft/review/yank exists; complete public registry governance missing |
| FPM-OWN-010 Install truth | existing Host/account install owners; MiniApp account sync compatibility | client install plan -> Host authorization -> durable local/account record | Host local installed ledger + explicit account sync as applicable | typed install/update/uninstall | all | partial | one canonical InstalledRecord adapter across old IDs and new Extension ID | crash/retry/account switch preserve one truth | must not move final authorization into Registry |
| FPM-OWN-011 Mini App permission contract | Platform Core `native/mahayana-messaging/src/miniapp.rs` + native bridge owners | native Mini App session -> Host permission dispatch | Host grants/session | MiniAppPermission/Grant/Session/Request | shared + native | implemented baseline | extend only where OMA needs typed package/origin grants; keep trust separate from permission | request requires exact effective grant; revoke/expiry fail closed | Core main source |
| FPM-OWN-012 iOS Marketplace | `MarketplaceModel.swift` and current native parity owners | SwiftUI Marketplace -> iOS Coordinator/Host | iOS account/install projection | feature.marketplace / plugin actions | iOS | partial | adapt to canonical identity/compatibility; preserve native owner | browse/detail/install gating/account switch on PR exact head | PR #3 observed only; final acceptance pending |
| FPM-OWN-013 iOS Mini App web runtime | `MiniAppWebMcpSurface.swift` + package verifier | WKWebView native surface | native session/grants/local package state | WebMCP bridge nonce/grants | iOS | partial | OMA package origin/storage isolation, offline local resource loader, local-author trust | forged origin/nonce/cross-app storage rejected; airplane-mode cold launch | verifier exists on PR #3; policy/device acceptance pending |
| FPM-OWN-014 Android Marketplace | `MarketplaceViewModel.kt` / MarketplaceContent | native Android Marketplace flow | Android account/install projection | existing native Host actions | Android | partial | adapt canonical identity/compatibility and installed truth | browse/detail/install/update/account switch | current path is under `frontend/src/main/kotlin/...`, not old guessed mobile path |
| FPM-OWN-015 Android Mini App web runtime | `MiniAppWebMcpSurface.kt` + `MiniAppPlatformBridge.kt` | Android WebView + preload bridge | native grants/local app data | native Web bridge | Android | partial | OMA WebViewAssetLoader-style isolated origins/package loader, quotas/update rollback | cross-origin/cookie/storage/native bridge isolation + offline cold start | PR #3 observed; full OMA acceptance pending |
| FPM-OWN-016 AI private Mini App creation | existing Agent/Runner artifacts -> native Host verifier | mobile Agent -> draft -> preview -> explicit install | draft digest + Host local-author key/approval | OMA-AI logical contracts | iOS/Android | missing/partial prerequisites | implement draft workspace, zero-priv preview, exact-digest approval, private-local-author signing and normal installer | generated multi-file app preview/install/offline/update/rollback; malicious output denied | policy and device acceptance are separate blockers, not reason to stop other work |
| FPM-OWN-017 Ratings/install counts | no proven canonical trustworthy source | Marketplace detail/list | must be server audited metrics | future metrics API | all | missing | hide until real source + anti-fraud/privacy exists | absent data => field absent, never fake | FPM-R09 |
| FPM-OWN-018 Team/private policy | existing team/private plugin policy owners | effective plugins / Host authorization | team policy + Host effective grants | existing policy APIs | all | partial | map kind/publisher/version/trust/platform into canonical policy without resetting grants | required/disabled survives migration; forbidden install/call rejected | must preserve current behavior |
| FPM-OWN-019 Release/packaging evidence | each platform release workflow | signed packaged Desktop / iOS distribution / Android artifact | release provenance + exact SHA | Actions/release contracts | all | partial | add Extension Registry/OMA E2Es and bind final artifacts to same exact source | E2E-01..08 and release artifacts/digests | old HEAD success cannot prove new HEAD |
| FPM-OWN-020 Security incident/revocation | Registry governance + Host policy | signed trust/revocation feed -> Host quarantine | registry trust epoch + Host local effective state | versioned signed metadata | all | missing/partial | trust epoch, revoke/quarantine, offline stale policy semantics, audit/appeal | revoked plugin cannot activate/call; replay old grant rejected | future production slice |

## Immediate implementation order derived from the inventory

1. Finish Backend unified read contract and exact-HEAD CI.
2. Add persistent publisher namespace/release model and security/audit write path in the same Backend owner.
3. Map existing Skills and legacy catalog IDs into the unified identity contract.
4. Reconcile Desktop PR #49 onto current canonical main, then consume unified Registry without replacing Host/OAuth owners.
5. Add canonical first-level Desktop Marketplace route and native client adapters.
6. Implement OMA package installer/runtime per platform, followed by AI private creation.
7. Complete governance/security, real provider/device/store acceptance and release evidence.

Blocked device/store/provider work does not block independent registry, contracts, UI, package validation or security work.
