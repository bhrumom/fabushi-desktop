# node.invoke Gateway envelope fidelity — 2026-10-30

The typed Rust node-invoke result owner preserves the transport result
representations. The public Gateway compatibility projection is a separate
boundary: the current OpenClaw Gateway handler emits a decoded structured
`payload` together with `payloadJSON: null` for this response shape.

The local Rust projection retains that observed Gateway response shape, while
the transport-neutral typed result remains the canonical node-host envelope;
no remote transport or second result owner was introduced.

The legacy public `params` adapter now serializes the object once at the
boundary and records both the canonical `paramsJSON` text and its byte count
on `NodeInvokeRequest`. This preserves the source representation for the
existing node-host owner instead of presenting a fabricated zero-byte,
unobserved request.

The same adapter applies the node-host parser's 256 KiB `paramsJSON` bound;
the public legacy entry point cannot bypass the wire-level resource limit.

Evidence is limited to source inspection, static contract updates, and
`git diff --check`; local OCRM tests/builds, remote node transport, and live
device acceptance remain pending.
