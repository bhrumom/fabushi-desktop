# GBR-003 — Local Computer / tool / host execution parity

Status: in-progress

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
- renderer Computer + local-tool-permission surfaces listed in parity-inventory.md

## Current implementation
Commit `5c3480a15003c145d26313f990e3f433cf8d7fdc` provides a first executable local-tool slice: list/read files, foreground terminal execution, and open HTTPS URL. Tool results are persisted in the transcript and the top-level turn can be aborted.

## Acceptance criteria
1. Computer/tool execution is owned by a host/execution boundary rather than a CLI wrapper.
2. Real local Mac Computer actions have explicit capability/error reporting.
3. Read-only vs mutating actions pass through persisted local-tool permission policy and user approval where required.
4. Pending approval can be approved/denied/cancelled; stop cancels approval and active child execution.
5. Tool transcript records queued/waiting-approval/running/done/error/cancelled states.
6. Foreground/background shell lifecycles are cancellable and observable.
7. Browser/Computer mutations use review + state recheck semantics where recoverable.
8. No cloud-computer provisioning path is exposed.

## Verification
Source-level parity inventory + direct runtime tests + exact-head GitHub Actions build/package after A8 closes.

## Evidence
- baseline local tool commit: `5c3480a15003c145d26313f990e3f433cf8d7fdc`
- baseline package run: `35323112810` (success; not final release evidence)

## Blockers / risks
Current target lacks the reference resource registry, controlled/stream execution, background shell, subagent, Computer, browser state, auto-review and local permission/approval model.

## Next action
Implement the host/execution + permission/approval slice, then update this record with commit and direct verification evidence.
