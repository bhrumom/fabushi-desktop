# OpenClaw current candidate root dependency probe

Status: discovery-only; not a dependency-closure or source-authority receipt

## Exact subject

- repository: openclaw/openclaw
- ref: refs/heads/main
- commit: b7cc7ebc9b442ed7eb73a92336c2a9886b578f7b
- root tree: ae1a46e87fdd518f53150ddec74e0948b731c23f
- observed: 2026-10-31T09:33:33Z
- recursive tree: 64,007 entries, 61,766 blobs, 2,241 directories, 0 gitlinks
- recursive tree API: truncated=false

## Root dependency inputs

GitHub raw reads at the exact commit reported:

- package.json blob: 3b8d23201d08859353b3ada3915f9e6fe5266dcf
- pnpm-lock.yaml blob: 83b4975eb96942ec007a2eb9847ccfe98d380bda
- pnpm-workspace.yaml blob: 32ce4d83de4aac6a1e3665baba09a3aa9582960c
- .gitattributes blob: c6118b226a4a760f7861b6b7b1e7dccd943384e4
- LICENSE blob: ed064819ab537575d0cad902f7e5af4a3488b1d0
- THIRD_PARTY_NOTICES.md blob: 0c9c123760188915a384ce7275df4096114e8f90
- .gitmodules: absent

The exact root manifest reports OpenClaw 2026.10.9, integrity-qualified
pnpm@12.15.0, 77 runtime dependencies, 57 dev dependencies, 1 optional
dependency, and 0 peer dependencies. The workspace file declares the root,
ui, packages/*, extensions/*, examples/*, and tools/solid-lint.
The root manifest also reports state schema 36 and agent schema 39.

## Evidence boundary

This probe proves only exact root-input identity and shape. It does not prove
recursive dependency closure, package tarball integrity, LFS completeness,
Actions inventory, source reads, or authority acceptance. The lock therefore
keeps accepted_for_this_implementation_round=false,
baseline_identity_accepted=false, and all closure/global flags unchanged.
