# OpenClaw live-candidate drift frontier

Status: read-only compare evidence; not source closure or acceptance evidence.

- base: `ad57b6135a55a6792b6d703bc1750954d854d6aa`
- subject: `d27a8c9d9fec49a81084993bf16c3fefc4efd508`
- subject tree: `e378a64af5f7409473636e6758cca43f4a204fcc`
- compare: ahead 1, behind 0, 16 changed files

The changed set includes `package.json`, Plugin SDK subpath documentation and
entrypoint registries, extension package-boundary/XAI configuration, package
distribution inventory, plugin SDK aliasing and contracts, target-resolution
test support, and audit documentation. The root package manifest changed while
the lock/workspace/license identities remained stable.

This drift reopens package/workspace classification and Plugin SDK/plugin
consumer reads on the new exact subject. It does not prove dependency closure,
source responsibility mapping, Rust equivalence, production wiring, Actions
verification, or acceptance. No ledger status or completion flag changes.
