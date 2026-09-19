# Acceptance matrix

| ID | Acceptance criterion | Verification | State |
|---|---|---|---|
| A1 | Production renderer exposes Grok-style agents/workspace only; contacts/MiniApps/payment/Mahayana workbench are not mounted | exact pinned source diff + packaged human check | source PASS / human pending |
| A2 | Agent execution is not a CLI wrapper and uses the reference Agent lifecycle/orchestration | SandAgentRunner + AnysphereAgent runtime contracts | PASS |
| A3 | Each agent targets the installed Mac, with no cloud-computer provisioning requirement | local executor/browser/computer contracts + packaged human tool run | source/runtime PASS / human pending |
| A4 | Plugins/Marketplace visible paths have executable backends | Marketplace HTTP install/uninstall, MCP OAuth/accounts/tool toggles, SKILL.md/workflow contracts | runtime PASS / human UI pending |
| A5 | macOS package builds in GitHub Actions | successful package run + artifacts | PASS |
| A6 | macOS prerelease tag is bound to the exact build SHA | GitHub Release + tag ref | PASS for candidate grok-parity-mac-16 → b3b04a23245345b7d3491eb5859b7ccbbffd8562; latest head will supersede after final gate |
| A7 | Human UX/function testing completed | external tester evidence | external / pending by original request |
| A8 | Every recoverable reference UI/backend source is accounted for without fabricated parity | exact pinned directory diffs + adapter/runtime contracts | SOURCE PASS (308/308 renderer; 1,724/1,724 reference source); runtime adapter PASS for tested local product difference; immutable carrier-only Host audit = EXTERNAL_DEPENDENCY; human packaged acceptance pending |
| A9 | Current exact head passes runtime contracts + pinned source typecheck + renderer production build | GitHub Actions | pending latest exact-head run after account-inference + fixture repair |

## External dependency
The pinned reconstruction's own Host activation script requires immutable carrier artifact `src/app/dist/host/host-main.cjs`. That binary carrier is not present in the Git repository. It is recorded as EXTERNAL_DEPENDENCY; no replacement artifact or fabricated anchor evidence is claimed.

## Product-difference rule
The only intentional runtime product difference is Computer location: Fabushi Agents operate the installed Mac. Cloud Box/VNC provisioning is not required. Reference Agent orchestration, renderer behavior and recoverable source remain pinned to `107877b4e2134fd167d239411386f09e42eadd6d`.
