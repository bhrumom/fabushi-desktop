# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | source review + packaged app human check | in progress |
| A2 | Agent execution uses TS coordinator/host/local-exec design, not a CLI wrapper | architecture/source review | planned |
| A3 | Each agent operates the installed Mac, with no cloud-computer provisioning UI | source review + human tool run | planned |
| A4 | Plugins UI matches reference structure and supports local install/configure/enable | source review + human flow | planned |
| A5 | macOS package builds in GitHub Actions | successful run + artifact | planned |
| A6 | macOS test release/prerelease published | release asset evidence | planned |
| A7 | Human UX/function testing completed | external tester evidence | external |
