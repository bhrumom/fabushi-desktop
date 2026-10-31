# Agent tool-loop detection

Status: implementation slice; production wiring remains pending
Project: OCRM-001
Responsibility: OCRM-RESP-AGENT-LOOP-DETECTION-001

## Context / goal

The canonical Runner owner already serializes one active turn and owns the
`TurnOwnerToken` cancellation boundary, but it has no stateful detector for a
tool call that repeats without making progress. Add the detector to that same
owner so a future production tool producer can report a loop without creating a
second Runner or session state owner.

## Non-goals

- Do not add a provider, tool executor, scheduler, or transcript store.
- Do not infer a loop from tool output text or from retries of a transport
  request; this slice only observes tool name plus arguments.
- Do not claim production wiring or acceptance until a real tool producer calls
  the owner and the cancellation reaches the active executor.

## Requirements

- LD-01: canonicalize JSON arguments deterministically, including object key
  ordering, before identity comparison.
- LD-02: retain at most 64 recent identities per active turn.
- LD-03: classify the third identical call as a loop and request intentional
  cancellation with a stable reason; distinct arguments reset the consecutive
  run.
- LD-04: reject empty tool names and stale owners without changing state.
- LD-05: keep all state inside `TurnRunShell`; no parallel loop owner.

## Architecture and flow

`tool producer -> TurnRunShell::observe_tool_call -> active run loop state ->
TurnRunShell::checkpoint_boundary -> existing executor cancellation path`.

The detector is a bounded per-turn helper owned by `TurnRunShell`. It is not
invoked by fixtures or tests as a substitute for a producer; production
integration remains a separate follow-up at the existing tool-call boundary.
The live source audit located that boundary in
source/host/src/runner/routed_provider_runtime.rs::execute_tool and the
RoutedToolBridge::call_tool chain. That discovery does not yet prove the
callback can safely borrow the mutable TurnRunShell across the provider
execution lifetime, so no wiring claim is made.

## Failure modes

Malformed/unsupported JSON is still an identity value because serde JSON is
already the Runner contract; empty names and stale owners are rejected. The
bounded history evicts the oldest identity and never grows without limit.

## Verification and Definition of Done

- Unit tests prove canonical object ordering, consecutive-repeat threshold,
  reset on a distinct call, history bound, empty-name rejection, and stale-owner
  isolation.
- `git diff --check` and the repository JSON syntax checks pass.
- The ledger records implementation progress while leaving wiring,
  verification, and acceptance pending.

## References

- `projects/openclaw-rust/parity-ledger.json`
- `source/host/src/runner/turn_run_shell.rs`
- `source/host/src/runner/production_turn_agent_owner.rs`
- OpenClaw source reference recorded by the ledger:
  `src/agents/tool-loop-detection.ts`
