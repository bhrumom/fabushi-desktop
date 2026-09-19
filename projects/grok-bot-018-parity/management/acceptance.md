# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | exact pinned source diff + packaged human check | source PASS / human pending |
| A2 | Agent execution is not a CLI wrapper and uses the reference Agent lifecycle/orchestration | SandAgentRunner + AnysphereAgent runtime contracts | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning requirement | local executor/browser/computer contracts + packaged human tool run | runtime PASS / human pending |
| A4 | Plugins/Marketplace visible paths have executable backends without mandatory external setup | Marketplace + MCP/OAuth/accounts/tool toggles + private-skill/workflow/listener contracts | runtime PASS / human UI pending |
| A5 | macOS package builds in GitHub Actions | Run 35419150459 + artifact 10577580224 | PASS |
| A6 | prerelease tag is bound to exact build SHA | immutable release branch + tag lookup | PASS — grok-parity-mac-101 → 1f27c239aa8931b3266b9b83c84ff234332785d9 |
| A7 | Human UX/function testing completed | GBR-007 human evidence | EXTERNAL / PENDING |
| A8 | Recoverable reference UI/backend source is accounted for without fabricated parity | exact pinned directory diffs + renderer closure + runtime contracts | SOURCE+RUNTIME PASS; immutable carrier audit = EXTERNAL_DEPENDENCY; human packaged acceptance pending |
| A9 | Product SHA passes runtime contracts + pinned source typecheck + renderer production build | push Run 35419150457 + PR Run 35419154451 | PASS — 105/105, 0 fail |

## Final automated evidence
- Pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`.
- Renderer: 308/308 blobs byte-identical.
- Vendored reference source: 1,724/1,724 blobs byte-identical.
- Product/package SHA: `1f27c239aa8931b3266b9b83c84ff234332785d9`.
- Immutable release branch: `release/grok-parity-alpha3-final` → same SHA.
- Push source gate Run `35419150457` and PR source gate Run `35419154451` — SUCCESS.
- Runtime contracts: **105/105 PASS, 0 fail**.
- Production runtime coverage includes exact AnysphereAgent/SandAgentRunner lifecycle, tool-call continuation, account-backed inference, reference coordinator closure, local Computer events, approvals, desktop bridges, TrayManager, sharing, Teach and Marketplace closure contracts.
- Mac Run `35419150459` — SUCCESS.
- Artifact `10577580224`: 393011773 bytes; `sha256:1c70dad6d99bc327a27165767636e47d1c4a0131db21e3b796b07982d7ccd4e1`.
- Release `grok-parity-mac-101` — tag resolves exactly to the package SHA.
- DMG: 196950473 bytes; `sha256:695ffe13980e185ac8f4852635147d5ddf45eefebac6903294dbb4962960371f`.
- ZIP: 197394273 bytes; `sha256:8db709695da85299c10e1155df2f798dd4d9325422f2f0c927101b199fb2052a`.
- Headless packaged renderer probe recorded `manualValidationRequired:true`; no automated packaged-UI PASS is claimed.

## External dependency
The pinned reconstruction's Host activation script references immutable carrier `src/app/dist/host/host-main.cjs`, which is not stored in the reference Git repository. It remains EXTERNAL_DEPENDENCY; no substitute/fabricated carrier evidence is claimed.

## Product difference
The only intentional runtime product difference is Computer location: Fabushi Agents operate the installed Mac. Cloud Box/VNC provisioning is not required.

## Remaining acceptance
GBR-007 is the only remaining acceptance stage and is intentionally human/external by the original request. PR #1 remains open until that installed-product evidence is recorded.
