# config Gateway projection frontier — 2026-10-30

The current OpenClaw candidate's `config.get` handler delegates to the
configuration snapshot owner and publishes that snapshot (including its
path/existence/validity and raw-config hash context). `config.set` requires a
non-empty `raw` replacement plus an optional `baseHash`, then publishes the
persisted config result and write metadata.

Fabushi's current `UnifiedGatewayApi` handlers instead read and write the
existing Host settings owner. `config.get` serializes that settings model and
returns both `settings` and `config` aliases with a locally computed hash;
`config.set` accepts the local settings/config forms in addition to `raw` and
returns the adapted model. The additive `configSource` projection now also
reports the canonical settings path, file existence, supported-version
validity, and raw-byte hash without returning the raw file. This is a
deliberate compatibility projection, but it is not yet evidence of full
OpenClaw config snapshot/schema parity.

The next implementation decision must bind the projection to the existing
Fabushi settings authority and current config consumers, then preserve
base-hash, redaction, validation, write-result, restart and failure semantics.
Do not add a second config store or silently replace the Host settings owner.

Evidence is limited to current candidate source inspection, local source
inspection, and `git diff --check`. No config runtime mutation, local OCRM
test/build, Actions run, or acceptance claim was made in this round.
