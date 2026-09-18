# GBR-003 — Local Computer / tool / host execution parity

Status: in-progress — executable host/permission/Computer slice implemented

## Objective
Recover the reference Computer/tool/host execution model while adapting the Box/Computer target to the Mac where Fabushi is installed. The adaptation changes location only; permission, cancellation, transcript, lifecycle and error semantics remain required.

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
- renderer Computer + local-tool-permission surfaces listed in `../parity-inventory.md`

## Implemented this round
- Runtime ownership is now `grok-agent-coordinator.cjs` → `grok-host-runtime.cjs` → `local-tool-executor.cjs`; the old runtime file is only a composition facade.
- Persisted local-tool permission: `always | ask | never`.
- One-shot approval request/allow/deny/cancel with renderer card and coordinator wait state.
- Agent Stop aborts the active turn and pending approval; foreground child execution accepts the AbortSignal.
- Tool transcript states: queued, waiting-approval, running, done, error, cancelled.
- Files: list/read/write/create-directory/move.
- Foreground shell plus background spawn/status/stdin/stop.
- Browser: executable HTTPS open.
- Computer on macOS: screencapture plus Accessibility-backed click/type/key actions with explicit capability errors.
- No cloud-computer provisioning path.

## Acceptance status
1. Host/execution boundary, not CLI wrapper — PASS.
2. Real local Mac Computer actions with errors — PASS for screenshot/click/type/key slice.
3. Persisted local permission + approval — PASS for current local tools.
4. Approval allow/deny/cancel + Stop — PASS for current turn.
5. Tool transcript lifecycle — PASS for current tools.
6. Foreground/background shell observable — PARTIAL; reference stream/watch/resource semantics remain.
7. Browser/Computer auto-review classifier + display-state recheck — FAIL/open.
8. No cloud provisioning — PASS.

## Evidence
- baseline local tool commit: `5c3480a15003c145d26313f990e3f433cf8d7fdc`
- local Computer executor: `f3a7b05b998a3d8a5dd9b0f9f31b4b6be8ba5dd1`
- host runtime: `d9fc48b7e737612b9f09489eaa69e0ab49862bcb`
- coordinator/approval ownership: `1eb5016baea8b2702b874ec454449ef529f84e16`
- runtime facade refactor: `f080cdaf5dd1e36d7b675e7a7ab56af6c40abb6b`
- renderer approval/settings/stop lifecycle: `399182167e154653893d348bfc9eceec023b02de`
- source check: Run `35333859439` PASS (CJS syntax + TypeScript/Vite build)
- baseline package run: `35323112810` success; not final release evidence

## Remaining blockers
Reference resource-provider/remote/controlled/stream execution, request-context propagation, subagents, browser snapshot/action state, classifier-backed auto-review, state identity recheck, richer Computer shell/overlay and exact runner behavior remain open in the generated per-module inventory.
