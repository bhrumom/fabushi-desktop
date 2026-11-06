# config.set empty-target fidelity — 2026-10-30

The adapted `config.set` Gateway handler previously used the entire request as
the target when `raw`, `config`, and `settings` were all absent. A request
containing only a concurrency guard could therefore reach the settings owner
as an empty/no-op target and still be reported as a successful write.

The boundary now requires one of the supported configuration representations
and requires the selected target to be a JSON object before entering the
existing Host settings owner. It preserves the existing compatibility forms
without creating a second configuration store.

Evidence is limited to source inspection, static contract updates, and
`git diff --check`; current OpenClaw config snapshot parity and local OCRM or
Actions acceptance remain pending.
