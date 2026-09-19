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
Run `35425432746` on released package SHA `98ac56f037cb1b0eaa66781087da986334754908` passed exact pinned source parity, reference runtime bundle build, 109/109 runtime tests, pinned reference source typecheck and TypeScript/Vite production build.

Runtime tests include reference-Agent model/tool continuation, Browser/MCP/Subagent routing, local mutation approval/cancellation, Computer actions, local Browser actions, Teach recording and coordinator/desktop bridge closure.

## Acceptance
- Non-CLI runtime: PASS.
- Reference Agent lifecycle/orchestration: PASS.
- Installed-Mac Computer execution: automated runtime PASS / PRODUCT_DIFFERENCE.
- Permission/approval/cancellation/transcript lifecycle: PASS.
- Packaged end-user Computer/permission behavior: GBR-007 human acceptance pending.

## Current immutable delivery evidence
- Human-test product/package SHA: `98ac56f037cb1b0eaa66781087da986334754908`.
- Push source gate Run `35425432746` and PR source gate Run `35425434873`: SUCCESS, **109/109 PASS, 0 fail**.
- macOS package/release Run `35425432753`: SUCCESS.
- Release `grok-parity-mac-126` / `2.0.0-alpha.4`; tag and `release/grok-parity-alpha4-final` resolve exactly to the product SHA.
- Packaged renderer UI remains manual acceptance under GBR-007; the hosted probe recorded `manualValidationRequired:true`.
