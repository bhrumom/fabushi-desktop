# node.invoke Gateway envelope fidelity — 2026-10-30

The typed Rust node-invoke result owner preserves mutually exclusive
`payload`/`payloadJSON` result representations. The public local
`node.invoke(system.run)` projection previously emitted a structured `payload`
and an additional `payloadJSON: null` field, which was not the mutually
selected wire shape.

The projection now publishes the observed structured payload only. The
transport-neutral typed result still owns the canonical result envelope; no
remote transport or second result owner was introduced.

Evidence is limited to source inspection, static contract updates, and
`git diff --check`; local OCRM tests/builds, remote node transport, and live
device acceptance remain pending.
