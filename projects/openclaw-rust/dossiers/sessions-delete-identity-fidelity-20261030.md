# sessions.delete identity fidelity — 2026-10-30

The session-delete Gateway projection previously rejected only an entirely
empty key. A key such as `agent::main` passed that check, parsed to an empty
agent identity, and was forwarded to the existing lifecycle owner.

The projection now rejects empty parsed identities before dispatching
`deleteAgent`. This keeps malformed routing at the Gateway boundary and does
not add another lifecycle or session owner.

Evidence is limited to source inspection, a focused source-contract assertion,
and `git diff --check`; local OCRM tests/builds and live sessions.delete
acceptance remain pending.
