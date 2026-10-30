# chat.abort status fidelity — 2026-10-30

`chat.abort` previously fabricated `run-agent-id` when no explicit run id was
provided, even when the runner had no active stream. The Gateway owner now
captures the existing runner registry's active stream ids before cancellation
and returns that observed set; an inactive agent therefore returns an empty
`runIds` array and `aborted: false`.

The existing `TranscriptRunnerRegistry` remains the sole cancellation owner.
Static source review and `git diff --check` are the only development evidence;
local OCRM tests/builds and live cancellation acceptance remain pending.
