# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | exact pinned source diff + packaged human check | source PASS / human pending |
| A2 | Agent execution is not a CLI wrapper and uses the reference Agent lifecycle/orchestration | SandAgentRunner + AnysphereAgent runtime contracts | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning requirement | local executor/browser/computer contracts + packaged human tool run | runtime PASS / human pending |
| A4 | Plugins/Marketplace visible paths have executable backends without mandatory external setup | Marketplace + MCP/OAuth/accounts/tool toggles + private-skill/workflow/listener contracts | runtime PASS / human UI pending |
| A5 | macOS package builds in GitHub Actions | Run 35425432753 + artifact 10579089056 | PASS |
| A6 | prerelease tag is bound to exact build SHA | release-branch gate + published tag-SHA verification | PASS — grok-parity-mac-126 → 98ac56f037cb1b0eaa66781087da986334754908 |
| A7 | Human UX/function testing completed | GBR-007 human evidence | EXTERNAL / PENDING |
| A8 | Recoverable reference UI/backend source is accounted for without fabricated parity | exact pinned directory diffs + renderer closure + runtime contracts | SOURCE+RUNTIME PASS; immutable carrier audit = EXTERNAL_DEPENDENCY; human packaged acceptance pending |
| A9 | Product SHA passes runtime contracts + pinned source typecheck + renderer production build | push Run 35425432746 + PR Run 35425434873 | PASS — 109/109, 0 fail |

## Final automated evidence
- Pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`.
- Renderer: 308/308 blobs byte-identical.
- Vendored reference source: 1,724/1,724 blobs byte-identical.
- Product/package SHA: `98ac56f037cb1b0eaa66781087da986334754908`.
- Human-test release alias: `release/grok-parity-alpha4-final` → same SHA.
- Packaging workflow legacy gate ref `release/grok-parity-alpha3-final` also resolved to the same SHA during Run `35425432753`.
- Push source gate Run `35425432746` and PR source gate Run `35425434873` — SUCCESS.
- Runtime contracts: **109/109 PASS, 0 fail**.
- Production runtime coverage includes exact AnysphereAgent/SandAgentRunner lifecycle, tool-call continuation, account-backed inference, reference coordinator closure, local Computer events, approvals, desktop bridges, TrayManager, sharing, Teach and Marketplace closure contracts.
- Mac Run `35425432753` — SUCCESS.
- Artifact `10579089056`: 393058647 bytes; `sha256:6c91562e2d8ede5641c6c4bb9010a1cd3ebad1c0d0744a5cb8cc48b9bd1850d1`.
- Release `grok-parity-mac-126` — tag resolves exactly to the package SHA.
- DMG: `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.dmg`, 196976857 bytes; `sha256:3516c7de4f8052e0fc219f08a3c2234a4648569d63c1277dcc7164a45dfe7320`.
- ZIP: `fabushi-grok-parity-2.0.0-alpha.4-macos-arm64.zip`, 197404010 bytes; `sha256:c70bd2a13c2fabf2cab3a74b959ecead488a2e4df76656fd18ef7b4d52e3a57c`.
- The packaged renderer probe is diagnostic only. It did not produce a UI report and recorded `manualValidationRequired:true`; no automated packaged-UI PASS is claimed.

## External dependency
The pinned reconstruction's Host activation script references immutable carrier `src/app/dist/host/host-main.cjs`, which is not stored in the reference Git repository. It remains EXTERNAL_DEPENDENCY; no substitute/fabricated carrier evidence is claimed.

## Product difference
The only intentional runtime product difference is Computer location: Fabushi Agents operate the installed Mac. Cloud Box/VNC provisioning is not required.

## Remaining acceptance
GBR-007 is the only remaining acceptance stage and is intentionally human/external by the original request. PR #1 remains open until that installed-product evidence is recorded.
