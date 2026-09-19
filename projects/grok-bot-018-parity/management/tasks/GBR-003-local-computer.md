# GBR-003 — Local Computer / tool / host execution parity

Status: automated runtime complete / packaged human acceptance delegated to GBR-007

## Objective
Recover the reference Computer/tool/host execution model while adapting the Computer target to the Mac where Fabushi is installed. The product adaptation changes location only; permission, cancellation, transcript, lifecycle, observation and error semantics remain required.

## Actual result
- Production execution is reference-Agent-owned: pinned `SandAgentRunner` + pinned `AnysphereAgent`.
- Fabushi injects local Files, Terminal, Browser, Computer, MCP and Subagent executors through the reference Agent tool boundary.
- Computer targets the installed Mac; cloud Box/VNC provisioning is intentionally absent.
- Local mutation permission persists as always/ask/never with approval allow/deny/cancel and Stop propagation.
- Files support list/read/write/mkdir/move; Terminal supports foreground streaming and background lifecycle.
- Browser uses the local hidden-browser runtime with reference-style snapshot/action semantics.
- macOS Computer supports screenshot, click, move, drag, scroll, type, key and wait through the packaged native helper.
- Tool lifecycle, retry, usage, action audit, bot-block observation and output spill are wired through the production Host.
- Teach recording is a real local Mac recording service; save attaches the recording to the Agent and invokes the Learn from demonstration workflow through the reference Agent runtime.

## Verification
Run `35419150457` on released package SHA `1f27c239aa8931b3266b9b83c84ff234332785d9` passed exact pinned source parity, reference runtime bundle build, 105/105 runtime tests, pinned reference source typecheck and TypeScript/Vite production build.

Runtime tests include reference-Agent model/tool continuation, Browser/MCP/Subagent routing, local mutation approval/cancellation, Computer actions, local Browser actions, Teach recording and coordinator/desktop bridge closure.

## Acceptance
- Non-CLI runtime: PASS.
- Reference Agent lifecycle/orchestration: PASS.
- Installed-Mac Computer execution: automated runtime PASS / PRODUCT_DIFFERENCE.
- Permission/approval/cancellation/transcript lifecycle: PASS.
- Packaged end-user Computer/permission behavior: GBR-007 human acceptance pending.
