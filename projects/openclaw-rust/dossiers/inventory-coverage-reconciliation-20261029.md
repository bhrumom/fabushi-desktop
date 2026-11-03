# OCRM inventory coverage reconciliation — 2026-10-29

## Scope

This dossier records a read-only reconciliation of the locked OpenClaw source inventory and the derived parity-ledger coverage totals. It does not accept any completion flag, implementation, wiring, release, or runtime behavior.

## Exact authority

- Upstream commit: `ad8d12f126d29b0db204187adf5092db80294e70`
- Upstream tree: `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
- Canonical rows: `projects/openclaw-rust/inventory/shards/part-*.jsonl.gz`
- Summary: `projects/openclaw-rust/inventory/index.json`
- Derived coverage: `projects/openclaw-rust/parity-ledger.json`

## Reconciled facts

Streaming all canonical shard rows and counting `responsibility_status` produced:

- tracked rows: `58,750`
- responsibility-mapped rows: `1,092`
- responsibility-unclassified rows: `57,658`
- omitted-known rows: `0`
- invariant: `1,092 + 57,658 = 58,750`

The inventory summary already reported `unclassified: 57,658`. The parity ledger alone was stale at `mapped_paths: 1,090` and `unclassified_paths: 57,660`. This reconciliation changes only those two derived counters; it does not alter canonical rows, source authority, or any completion flag.

## Evidence and limits

The counts were obtained by read-only `gzip`/`jq` streaming over every tracked shard and by comparing the current summary with the prior exact-head ledger state. No local build, test, validator, generator, CI workflow, push, or release action was run. This is bookkeeping consistency evidence only and is not proof of semantic reading, implementation equivalence, production wiring, or acceptance.
