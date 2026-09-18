# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | source review + packaged app human check | source PASS / human pending |
| A2 | Agent execution uses Electron coordinator → host → execution ownership, not a CLI wrapper | source review | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning UI | source review + human tool run | source PASS / human pending |
| A4 | Every visible Plugins item has an executable backend; local capabilities and stdio MCP server/tool configuration are functional | source review + source build | current visible entries PASS; reference OAuth/accounts/private skills/workflows/marketplace breadth pending under A8 |
| A5 | macOS package builds in GitHub Actions | successful run + artifact | PASS — Run 35323112810, artifact 10538065159, sha256:884a992b9cfa2cf16891ddee05939a5daa52bc2343f8ac92e6fa0c30d9d4adae |
| A6 | macOS prerelease published from canonical main | release asset evidence | pending; deliberately gated until A8 |
| A7 | Human UX/function testing completed | external tester evidence | external / next phase |
| A8 | Every recoverable reference UI/backend feature is accounted for and matched | parity-inventory.md + parity-inventory.generated.json | NOT COMPLETE |
| A9 | Current parity source head passes CJS syntax + TypeScript/Vite build without packaging | GitHub Actions | PASS — Run 35333859439 on 5e447597d4081e8df8dc5d3b5abea59175f7c232 |

A5 is historical package-mechanism evidence only. It does not substitute for the exact-head package required after A8 closes, and it is not release evidence for A6.
