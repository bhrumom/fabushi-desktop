# Agent tool-loop detection

Status: implementation and production-wiring slice; verification remains pending
Project: OCRM-001
Responsibility: OCRM-RESP-AGENT-LOOP-DETECTION-001

## Context / goal

The canonical Runner owner already serializes one active turn and owns the
`TurnOwnerToken` cancellation boundary, but it had no stateful detector for a
tool call that repeats without making progress. Add the detector to that same
owner and connect the existing routed-tool producer to it without creating a
second Runner or session state owner.

## Non-goals

- Do not add a provider, tool executor, scheduler, or transcript store.
- Do not infer a loop from tool output text or from retries of a transport
  request; this slice only observes tool name plus arguments.
- Do not claim verification or acceptance from static wiring alone; the exact
  routed-tool execution path still needs the frozen Actions oracle.

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

The detector is a bounded per-turn helper owned by `TurnRunShell`. The
production path binds it in `ProductionTurnAgentOwner`, passes it through
`TurnAgentComposition`, and observes routed calls in
`CheckpointRecordingRoutedToolBridge::call_tool`. A detected loop requests
cancellation through the existing `RoutedProviderCancellation`; the shell
imports that cancellation at its checkpoint boundary. Fixtures and tests are
not used as a substitute for this producer/consumer chain.

## Failure modes

Malformed/unsupported JSON is still an identity value because serde JSON is
already the Runner contract; empty names and stale owners are rejected. The
bounded history evicts the oldest identity and never grows without limit.

## Verification and Definition of Done

- Unit tests prove canonical object ordering, consecutive-repeat threshold,
  reset on a distinct call, history bound, empty-name rejection, and stale-owner
  isolation.
- `git diff --check` and the repository JSON syntax checks pass.
- The ledger records `implementation=implemented` and `wiring=wired`; frozen
  verification and acceptance remain pending.

## References

- `projects/openclaw-rust/parity-ledger.json`
- `source/host/src/runner/turn_run_shell.rs`
- `source/host/src/runner/production_turn_agent_owner.rs`
- OpenClaw source reference recorded by the ledger:
  `src/agents/tool-loop-detection.ts`
