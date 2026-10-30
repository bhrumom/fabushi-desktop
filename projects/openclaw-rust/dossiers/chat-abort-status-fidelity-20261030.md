# chat.abort status fidelity — 2026-10-30

`chat.abort` previously fabricated `run-agent-id` when no explicit run id was
provided, even when the runner had no active stream. The Gateway owner now
captures the existing runner registry's active stream ids before cancellation
and returns that observed set; an inactive agent therefore returns an empty
`runIds` array and `aborted: false`. An explicitly supplied `runId` is now
returned only when it matches a stream observed as active before cancellation;
the request parameter is not treated as proof that a run existed.

The same boundary rejects a non-empty session key that parses to an empty agent
identity, such as `agent::main`, before reaching the cancellation owner.

When a caller supplies `runId`, cancellation is scoped to that exact observed
stream instead of cancelling every active stream for the agent and only
filtering the response. Without `runId`, all observed active streams are
cancelled through the existing per-stream owner, including routed and group
member streams.

An explicitly supplied `runId` must also be a non-empty string; an empty or
non-string value is rejected rather than widening into an all-stream abort.

The existing `TranscriptRunnerRegistry` remains the sole cancellation owner.
Static source review and `git diff --check` are the only development evidence;
local OCRM tests/builds and live cancellation acceptance remain pending.
