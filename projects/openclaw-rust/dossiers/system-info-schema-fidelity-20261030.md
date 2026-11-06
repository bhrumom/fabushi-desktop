# system.info schema fidelity — 2026-10-30

The current OpenClaw `SystemInfoResultSchema` requires `nodeVersion` and is a
closed object. The Rust projection previously emitted `version` and `status`
instead, omitted `nodeVersion`, and therefore did not match the current wire
shape.

The Gateway now maps the packaged Rust Host version to the compatibility
`nodeVersion` field and removes the undeclared `version`/`status` fields. The
mapping identifies the serving Host runtime version; it does not claim that a
Node.js runtime exists in Fabushi.

Evidence is limited to current upstream schema inspection, source inspection,
static contract updates, and `git diff --check`; exact-head Actions and live
Gateway compatibility acceptance remain pending.
