# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | exact pinned source diff + packaged human check | source PASS / human pending |
| A2 | Agent execution is not a CLI wrapper and uses the reference Agent lifecycle/orchestration | SandAgentRunner + AnysphereAgent runtime contracts | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning requirement | local executor/browser/computer contracts + packaged human tool run | runtime PASS / human pending |
| A4 | Plugins/Marketplace visible paths have executable backends without mandatory external setup | default local Marketplace install/uninstall + remote Marketplace + MCP OAuth/accounts/tool toggles + SKILL.md/workflow publication/listener contracts | runtime PASS / human UI pending |
| A5 | macOS package builds in GitHub Actions | successful package run + DMG/ZIP artifacts | PASS mechanism; final 2.0.0-alpha.2 exact-head package pending |
| A6 | macOS prerelease tag is bound to the exact build SHA | GitHub Release + tag ref | PASS for prior candidate grok-parity-mac-16 → b3b04a23245345b7d3491eb5859b7ccbbffd8562; final 2.0.0-alpha.2 candidate pending |
| A7 | Human UX/function testing completed | external tester evidence | EXTERNAL / pending by original request |
| A8 | Every recoverable reference UI/backend source is accounted for without fabricated parity | exact pinned directory diffs + official renderer closure + adapter/runtime contracts | SOURCE+RUNTIME PASS for automated scope; immutable carrier-only Host audit = EXTERNAL_DEPENDENCY; packaged human acceptance pending |
| A9 | Final frozen code SHA passes runtime contracts + pinned source typecheck + renderer production build | GitHub Actions | pending final 2.0.0-alpha.2 frozen SHA |

## Verified automated evidence before final version freeze
- Pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`.
- Renderer source: 308/308 blobs byte-identical.
- Vendored reference source: 1,724/1,724 blobs byte-identical.
- Push source gate on `ded4645501db434753de38b174412145b3fc32dc`: Run `35416952328` SUCCESS with 80/80 tests, reference source typecheck and Vite production build.
- Production runtime tests explicitly verify the exact AnysphereAgent path, tool-call continuation, account-backed inference, reference coordinator closure, local Computer events, approvals and desktop bridges.
- Default Marketplace now has executable local entries for Custom MCP Server and Private Skill; remote Grok-compatible provider support remains available.

## External dependency
The pinned reconstruction's own Host activation script requires immutable carrier artifact `src/app/dist/host/host-main.cjs`. That binary carrier is not present in the Git repository. It is recorded as EXTERNAL_DEPENDENCY; no replacement artifact or fabricated anchor evidence is claimed.

## Product-difference rule
The only intentional runtime product difference is Computer location: Fabushi Agents operate the installed Mac. Cloud Box/VNC provisioning is not required. Reference Agent orchestration, renderer behavior and recoverable source remain pinned to `107877b4e2134fd167d239411386f09e42eadd6d`.
