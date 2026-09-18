# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | source review + packaged app human check | source PASS / human pending |
| A2 | Agent execution uses Electron/TypeScript coordinator/runtime design, not a CLI wrapper | source review | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning UI | source review + human tool run | source PASS / human pending |
| A4 | Plugins browse/search/install/enable lifecycle is functional | source review + human flow | baseline PASS / advanced MCP auth pending |
| A5 | macOS package builds in GitHub Actions | successful run + artifact | pending |
| A6 | macOS prerelease published from canonical main | release asset evidence | pending |
| A7 | Human UX/function testing completed | external tester evidence | external |
| A8 | Every recoverable reference UI/backend feature is accounted for and matched | parity inventory | NOT COMPLETE |
