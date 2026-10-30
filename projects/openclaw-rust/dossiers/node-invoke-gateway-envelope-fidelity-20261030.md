# node.invoke Gateway envelope fidelity — 2026-10-30

The typed Rust node-invoke result owner preserves the transport result
representations. The public Gateway compatibility projection is a separate
boundary: the current OpenClaw Gateway handler emits a decoded structured
`payload` together with `payloadJSON: null` for this response shape.

The local Rust projection retains that observed Gateway response shape, while
the transport-neutral typed result remains the canonical node-host envelope;
no remote transport or second result owner was introduced.

Evidence is limited to source inspection, static contract updates, and
`git diff --check`; local OCRM tests/builds, remote node transport, and live
device acceptance remain pending.
