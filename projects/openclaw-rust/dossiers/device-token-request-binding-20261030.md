# Device-token request binding — 2026-10-30

The Gateway already authenticated durable device tokens through the canonical
pairing owner and enforced role/scope checks. The `node` role, however, was
only prevented from invoking operator-only methods; `node.invoke` and
`node.register` could still carry a different target identity because the
request body was not bound to the authenticated token's `device_id`.

Gateway ingress now calls an authorization hook after decoding the command
arguments. The existing `UnifiedGatewayApi` binds node-token targets for
`node.invoke`, `node.register`, `device.pair.request`, and
`device.pair.status` to the token's durable device identity. Missing or
mismatched targets fail with 403 before the existing method owner runs. The
pairing owner remains the only token identity authority, and operator tokens
retain the existing path.

Evidence is limited to source inspection and `git diff --check`; runtime
gateway tests, device transport, packaged acceptance, and Actions receipts
remain pending.
