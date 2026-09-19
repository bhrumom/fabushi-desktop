# GBR-003 — Local Computer / tool / host execution parity

Status: **runtime complete / packaged human acceptance pending**

## Objective

Recover the reference Computer/tool/host behavior while adapting the Computer target to the Mac where Fabushi is installed. The adaptation changes location only; permission, cancellation, transcript, lifecycle, audit and error semantics remain required.

## Actual result

- Files, foreground/background Terminal, Browser, Computer, MCP and Subagent paths are executable.
- Per-Agent Browser windows expose pinned-reference action breadth and keep CDP scoped to the active tab.
- macOS Computer supports screenshot/click/move/drag/scroll/type/key/wait through the packaged native helper.
- Local-tool permission, approval/deny/cancel, auto-review and Stop are persisted and tested.
- Turn observation, provider usage, action audit, output spilling and bounded transient retry are executable.
- Local Teach recording creates real sessions, fails closed on native startup failure, supports discard, and saves into the Agent learning workflow.
- Cloud Box/VNC provisioning is intentionally absent; the installed Mac is the Computer target.

## Verification

Final source gate Run `35417698506` at shipped SHA `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd` passed 103/103 tests. Relevant contracts include local Browser reference actions, executor routing, Computer identity, approvals, Teach recording, MCP and delegated subagents.

Final Mac Run `35417698507` passed packaging and packaged-app smoke. Physical macOS permission dialogs and real-device interaction remain GBR-007.
