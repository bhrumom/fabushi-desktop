# GBR-003 — Local Computer / tool / host execution parity

Status: in-progress — local-Mac execution model is broad and executable; exact runner/Computer parity still open

## Objective
Recover the reference Computer/tool/host execution model while adapting the Box/Computer target to the Mac where Fabushi is installed. The adaptation changes location only. Permission, cancellation, state identity, transcript, lifecycle, observation and error semantics remain required.

## Source references
- `source/packages/agent-exec/resource-provider.ts`
- `source/packages/agent-exec/remote.ts`
- `source/packages/agent-exec/background-shell.ts`
- `source/packages/agent-exec/request-context.ts`
- `source/packages/agent-exec/subagent.ts`
- `source/host/runner/turn-agent-composition.ts`
- `source/host/runner/tools/turn-toolset.ts`
- `source/host/runner/sand-computer-auto-review.ts`
- `source/host/runner/sand-browser-auto-review.ts`
- `source/host/runner/stream-attempt.ts`
- `source/host/runner/turn-observation.ts`
- renderer Computer shell/overlay surfaces in the generated inventory

## Current executable implementation
- Ownership: `grok-agent-coordinator.cjs` → `grok-host-runtime.cjs` → resource registry/request context → local/browser/external/subagent executors. The old runtime file is composition-only, not a CLI shell.
- Permission: persisted `always | ask | never`, approval allow/deny, cancellation and enforce/shadow/off auto-review.
- Auto-review: canonical target/fingerprint, classifier, user allow/ask-first instructions, screenshot/browser state recheck, fail-closed fallback.
- Local files: list/read/write/mkdir/move.
- Foreground shell: streaming stdout/stderr, abort propagation.
- Background shell: spawn/status/stdin/stop.
- Browser: hidden local browser with snapshot/stateId, navigate, click, type, key, scroll and screenshot.
- Computer on macOS: screenshot, click, mouse move, drag, scroll, type, key and wait. Mutating actions require the reviewed screenshot stateId; native pointer/scroll actions use the packaged CoreGraphics helper.
- Computer renderer: status/preview plus expandable local Computer shell.
- Turn communication: SendMessage progress/final delivery, ReactToMessage, tool-silence reminders.
- Turn execution: bounded transient retry before visible stream output, Retry-After handling/backoff.
- Observation/audit: turn/tool/retry/outcome observations, provider token usage, scrubbed action audit, site visit host tracking and bot-block classification.
- Durable state: memory/update_state/workflows/routines are host tools rather than CLI escape hatches.
- Cloud Box/VNC resources are not provisioned.

## Acceptance status
1. Independent host/execution boundary, not CLI wrapper — **PASS**.
2. Real installed-Mac Computer execution — **PASS for screenshot/click/move/drag/scroll/type/key/wait**.
3. Persisted local permission + approval — **PASS for current target tools**.
4. Approval allow/deny/cancel + Stop — **PASS**.
5. Tool transcript lifecycle — **PASS for current target tools**.
6. Foreground/background shell — **PARTIAL**; exact reference rewatch/restart/watch semantics remain.
7. Browser state/screenshot/action model — **PARTIAL**; executable local browser exists, exact reference subagent/audit breadth remains.
8. Auto-review classifier + target fingerprint/state recheck — **PARTIAL**; core behavior exists, exact escalation/ceiling modules remain.
9. Turn observation/usage/action audit — **PARTIAL**; core audit exists, TTFT/await/stall/CDP probe breadth remains.
10. Exact Computer shell/teach-recording — **PARTIAL**.
11. No cloud provisioning — **PASS / PRODUCT_DIFFERENCE constraint satisfied**.

## Objective evidence
- last confirmed source check before the newest changes: Run `35345930555` SUCCESS.
- Computer move/drag/scroll/wait: commits `5413245d5efc43ce36f0633fbda53e1f6d1cfe7f`, `8c089b5ca7222cb71197e916e684c0fe7453413a`, native helper `d784cd088ac3e69535dfa9023bdbf521e7670067`.
- transient retry: `9e957f244cd48434f3379b59fac3487a2b8fd1ae`, integration `b23b74f80a2ede92d3b0582e6a7b289312c4577e`.
- SendMessage/ReactToMessage + interaction semantics: `5a14905d4a11cfd68bd6996fff837e44ed7d2472` and later interactive-card commits.
- turn usage/action audit: `34d61a15dab0fddbfd620b2be87eaf90a579c494`, `303b7e4a3e44df2f1d5884da0d0f60fdaa8dc028`, host integration `71a774f5aae8a6d1837e8ad3d68b1402223843cd`.
- historical package mechanism only: Run `35323112810`; not exact-head package or release evidence.

## Remaining blockers
The authoritative blockers are the PARTIAL/FAIL runner and renderer rows in `../parity-inventory.generated.json`: exact stream checkpoint/resume, start-of-turn acknowledgement timing, async task/await/MCP stall observations, shell rewatch/restart depth, request-box-help/local replacement, Computer teach/recording, exact browser/Computer subagent composition, remaining auto-review escalation modules, and full UI/state equivalence.
