# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | exact pinned source diff + packaged human check | source PASS / human pending |
| A2 | Agent execution is not a CLI wrapper and uses the reference Agent lifecycle/orchestration | SandAgentRunner + AnysphereAgent runtime contracts | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning requirement | local executor/browser/computer contracts + packaged human tool run | runtime PASS / human pending |
| A4 | Plugins/Marketplace visible paths have executable backends without mandatory external setup | default local Marketplace install/uninstall + remote Marketplace + MCP OAuth/accounts/tool toggles + SKILL.md/workflow publication/listener contracts | runtime PASS / human UI pending |
| A5 | macOS package builds in GitHub Actions | successful package run + DMG/ZIP artifacts | PASS — Run 35419150459; artifact 10577580224; DMG/ZIP uploaded |
| A6 | macOS prerelease tag is bound to the exact build SHA | Release/tag/API evidence | PASS — grok-parity-mac-101 → 1f27c239aa8931b3266b9b83c84ff234332785d9 |
| A7 | Human UX/function testing completed | external tester evidence | external / pending by original request |
| A8 | Every recoverable reference UI/backend source is accounted for without fabricated parity | exact pinned directory diffs + official renderer closure + adapter/runtime contracts | SOURCE+RUNTIME PASS for automated scope; immutable carrier-only Host audit = EXTERNAL_DEPENDENCY; packaged human acceptance pending |
| A9 | Current product SHA passes runtime contracts + pinned source typecheck + renderer production build | GitHub Actions Run 35419150457 | PASS — 105/105 runtime tests, 0 fail |

## Verified automated evidence for current human-test candidate
- Pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`.
- Renderer source: 308/308 blobs byte-identical.
- Vendored reference source: 1,724/1,724 blobs byte-identical.
- Push source gate: Run `35419150457` on `1f27c239aa8931b3266b9b83c84ff234332785d9` SUCCESS with 105/105 tests, exact pinned source diff, reference source typecheck and Vite production build.
- Production runtime tests verify the exact AnysphereAgent path, tool-call continuation, account-backed inference, reference coordinator closure, local Computer events, approvals, desktop bridges, TrayManager, sharing and Teach/Marketplace closure contracts.
- macOS package/release: Run `35419150459` SUCCESS on the same SHA.
- Headless renderer probe recorded `manualValidationRequired:true`; no automated packaged-UI PASS is claimed.
- Artifact `10577580224`: 393011773 bytes; digest `sha256:1c70dad6d99bc327a27165767636e47d1c4a0131db21e3b796b07982d7ccd4e1`.
- Release `grok-parity-mac-101` publishes `2.0.0-alpha.3` DMG/ZIP; immutable release branch and tag both resolve exactly to the package SHA.
- Remaining A7 is intentionally external/manual, matching the original request to hand product testing to a human.

## External dependency
The pinned reconstruction's own Host activation script requires immutable carrier artifact `src/app/dist/host/host-main.cjs`. That binary carrier is not present in the Git repository. It is recorded as EXTERNAL_DEPENDENCY; no replacement artifact or fabricated anchor evidence is claimed.

## Product-difference rule
The only intentional runtime product difference is Computer location: Fabushi Agents operate the installed Mac. Cloud Box/VNC provisioning is not required. Reference Agent orchestration, renderer behavior and recoverable source remain pinned to `107877b4e2134fd167d239411386f09e42eadd6d`.


## Final automated delivery evidence
- Package SHA: `1f27c239aa8931b3266b9b83c84ff234332785d9`.
- Push source gate: Run `35419150457` SUCCESS — 105/105 runtime contracts PASS, pinned reference source typecheck PASS, Vite production build PASS.
- macOS package/release: Run `35419150459` SUCCESS.
- Artifact: `10577580224`, 393011773 bytes, digest `sha256:1c70dad6d99bc327a27165767636e47d1c4a0131db21e3b796b07982d7ccd4e1`.
- Release: `grok-parity-mac-101`; tag resolves exactly to the package SHA.
- Assets: `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.dmg` (196950473 bytes; `sha256:695ffe13980e185ac8f4852635147d5ddf45eefebac6903294dbb4962960371f`) and `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.zip` (197394273 bytes; `sha256:8db709695da85299c10e1155df2f798dd4d9325422f2f0c927101b199fb2052a`).
- Release assets additionally expose SHA-256 digests: DMG `sha256:695ffe13980e185ac8f4852635147d5ddf45eefebac6903294dbb4962960371f`; ZIP `sha256:8db709695da85299c10e1155df2f798dd4d9325422f2f0c927101b199fb2052a`.
- Human UX/function testing is intentionally not claimed; it is GBR-007.
