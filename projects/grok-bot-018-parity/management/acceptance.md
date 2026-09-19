# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | exact pinned source diff + packaged human check | source PASS / human pending |
| A2 | Agent execution is not a CLI wrapper and uses the reference Agent lifecycle/orchestration | SandAgentRunner + AnysphereAgent runtime contracts | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning requirement | local executor/browser/computer contracts + packaged human tool run | runtime PASS / human pending |
| A4 | Plugins/Marketplace visible paths have executable backends without mandatory external setup | default local Marketplace install/uninstall + remote Marketplace + MCP OAuth/accounts/tool toggles + SKILL.md/workflow publication/listener contracts | runtime PASS / human UI pending |
| A5 | macOS package builds in GitHub Actions | successful package run + DMG/ZIP artifacts | PASS — Run `35417698507` on exact SHA `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd` |
| A6 | macOS prerelease tag is bound to the exact build SHA | GitHub Release + tag ref | PASS — `grok-parity-mac-92` → `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd` |
| A7 | Human UX/function testing completed | external tester evidence | EXTERNAL / pending by original request |
| A8 | Every recoverable reference UI/backend source is accounted for without fabricated parity | exact pinned directory diffs + official renderer closure + adapter/runtime contracts | SOURCE+RUNTIME PASS for automated scope; immutable carrier-only Host audit = EXTERNAL_DEPENDENCY; packaged human acceptance pending |
| A9 | Final frozen code SHA passes runtime contracts + pinned source typecheck + renderer production build | GitHub Actions | PASS — Run `35417698506`, 103/103 tests, exact source/typecheck/build green |

## Verified automated evidence for final frozen candidate
- Pinned reference: `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`.
- Renderer source: 308/308 blobs byte-identical.
- Vendored reference source: 1,724/1,724 blobs byte-identical.
- Final source gate: Run `35417698506` on `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd` SUCCESS with 103/103 tests, exact pinned source diff, reference source typecheck and Vite production build.
- Production runtime tests explicitly verify the exact AnysphereAgent path, tool-call continuation, account-backed inference, reference coordinator closure, local Computer events, approvals and desktop bridges.
- Default Marketplace now has executable local entries for Custom MCP Server and Private Skill; remote Grok-compatible provider support remains available.
- Final package: Run `35417698507` SUCCESS on the same SHA; packaged smoke PASS.
- Artifact `10576418103`: `393022970` bytes; digest `sha256:6d3bf39199c312e695a6ab59cb7eb7cf408df6ab2ff34dee1a7076eb91406b2c`.
- Release `grok-parity-mac-92` published with `2.0.0-alpha.2` DMG/ZIP; tag resolves exactly to the frozen build SHA.
- Remaining A7 is intentionally external/manual, matching the original request to hand testing to a human.

## External dependency
The pinned reconstruction's own Host activation script requires immutable carrier artifact `src/app/dist/host/host-main.cjs`. That binary carrier is not present in the Git repository. It is recorded as EXTERNAL_DEPENDENCY; no replacement artifact or fabricated anchor evidence is claimed.

## Product-difference rule
The only intentional runtime product difference is Computer location: Fabushi Agents operate the installed Mac. Cloud Box/VNC provisioning is not required. Reference Agent orchestration, renderer behavior and recoverable source remain pinned to `107877b4e2134fd167d239411386f09e42eadd6d`.
