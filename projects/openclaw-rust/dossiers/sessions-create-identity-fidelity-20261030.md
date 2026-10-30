# sessions.create identity fidelity — 2026-10-30

`call_openclaw_sessions_create` delegates creation to the existing
`AgentLifecycle` owner. It previously treated a successful lifecycle result
without `agent.id` as success and constructed the fabricated key
`agent::main`.

The Gateway projection now requires a non-empty durable agent identity before
publishing `ok`, `key`, or `sessionId`; missing identity is an internal error.
No lifecycle or session owner was duplicated.

Evidence is limited to source inspection, a focused source-contract assertion,
and `git diff --check`; local OCRM tests/builds and live sessions.create
acceptance remain pending.
