# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | exact pinned source diff + packaged human check | source PASS / human pending |
| A2 | Agent execution is not a CLI wrapper and uses the reference Agent lifecycle/orchestration | SandAgentRunner + AnysphereAgent runtime contracts | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning requirement | local executor/browser/computer contracts + packaged human tool run | runtime PASS / human pending |
| A4 | Plugins/Marketplace visible paths have executable backends without mandatory external setup | default local Marketplace install/uninstall + remote Marketplace + MCP OAuth/accounts/tool toggles + SKILL.md/workflow publication/listener contracts | runtime PASS / human UI pending |
| A5 | macOS package builds in GitHub Actions | successful package run + DMG/ZIP artifacts | PASS — Run 35418603624; artifact 10576404427; DMG/ZIP uploaded |
| A6 | macOS prerelease tag is bound to the exact build SHA | Release/tag/API evidence | PASS — grok-parity-mac-97 → 528ddc8e31c320ca472191cf29a860c2086dabc2 |
| A7 | Human UX/function testing completed | external tester evidence | external / pending by original request |
| A8 | Every recoverable reference UI/backend source is accounted for without fabricated parity | exact pinned directory diffs + official renderer closure + adapter/runtime contracts | SOURCE+RUNTIME PASS for automated scope; immutable carrier-only Host audit = EXTERNAL_DEPENDENCY; packaged human acceptance pending |
| A9 | Current product SHA passes runtime contracts + pinned source typecheck + renderer production build | GitHub Actions Run 35418603612 | PASS — 105/105 runtime tests, 0 fail |

## Verified automated evidence for current human-test candidate
- Pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`.
- Renderer source: 308/308 blobs byte-identical.
- Vendored reference source: 1,724/1,724 blobs byte-identical.
- Push source gate: Run `35418603612` on `528ddc8e31c320ca472191cf29a860c2086dabc2` SUCCESS with 105/105 tests, exact pinned source diff, reference source typecheck and Vite production build.
- PR source gate: Run `35418606285` SUCCESS on the same product SHA.
- Production runtime tests verify the exact AnysphereAgent path, tool-call continuation, account-backed inference, reference coordinator closure, local Computer events, approvals, desktop bridges, TrayManager, sharing and Teach/Marketplace closure contracts.
- macOS package/release: Run `35418603624` SUCCESS on the same SHA.
- Headless renderer probe recorded `manualValidationRequired:true`; no automated packaged-UI PASS is claimed.
- Artifact `10576404427`: 393016719 bytes; digest `sha256:7899c1e47537c4793de59118d9328085038505069e438b255c62b0c0421768b2`.
- Release `grok-parity-mac-97` publishes `2.0.0-alpha.3` DMG/ZIP and its tag resolves exactly to the product SHA.
- Remaining A7 is intentionally external/manual, matching the original request to hand product testing to a human.

## External dependency
The pinned reconstruction's own Host activation script requires immutable carrier artifact `src/app/dist/host/host-main.cjs`. That binary carrier is not present in the Git repository. It is recorded as EXTERNAL_DEPENDENCY; no replacement artifact or fabricated anchor evidence is claimed.

## Product-difference rule
The only intentional runtime product difference is Computer location: Fabushi Agents operate the installed Mac. Cloud Box/VNC provisioning is not required. Reference Agent orchestration, renderer behavior and recoverable source remain pinned to `107877b4e2134fd167d239411386f09e42eadd6d`.


## Final automated delivery evidence
- Product code SHA: `528ddc8e31c320ca472191cf29a860c2086dabc2`.
- Push source gate: Run `35418603612` SUCCESS — 105/105 runtime contracts PASS, pinned reference source typecheck PASS, Vite production build PASS.
- PR source gate: Run `35418606285` SUCCESS on the same SHA.
- macOS package/release: Run `35418603624` SUCCESS.
- Artifact: `10576404427`, 393016719 bytes, digest `sha256:7899c1e47537c4793de59118d9328085038505069e438b255c62b0c0421768b2`.
- Release: `grok-parity-mac-97`; tag resolves exactly to the product code SHA.
- Assets: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.dmg` (196953272 bytes) and `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.zip` (197394188 bytes).
- Human UX/function testing is intentionally not claimed; it is GBR-007.
