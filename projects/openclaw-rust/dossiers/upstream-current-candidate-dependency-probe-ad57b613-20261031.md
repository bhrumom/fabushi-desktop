# OpenClaw current candidate root dependency probe

Status: discovery-only; not a dependency-closure or source-authority receipt

## Exact subject

- repository: openclaw/openclaw
- ref: refs/heads/main
- commit: ad57b6135a55a6792b6d703bc1750954d854d6aa
- root tree: 24204d2196587105bf3af93a999cea03e8bdfae5
- observed: 2026-10-31T09:37:57Z
- recursive tree: 64,011 entries, 61,770 blobs, 2,241 directories, 0 gitlinks
- recursive tree API: truncated=false

## Root dependency inputs

The exact-commit GitHub reads returned these root identities:

- package.json blob: 3b8d23201d08859353b3ada3915f9e6fe5266dcf
- pnpm-lock.yaml blob: 83b4975eb96942ec007a2eb9847ccfe98d380bda
- pnpm-workspace.yaml blob: 32ce4d83de4aac6a1e3665baba09a3aa9582960c
- .gitattributes blob: c6118b226a4a760f7861b6b7b1e7dccd943384e4
- LICENSE blob: ed064819ab537575d0cad902f7e5af4a3488b1d0
- THIRD_PARTY_NOTICES.md blob: 0c9c123760188915a384ce7275df4096114e8f90
- .gitmodules: absent

The root inputs are identity-stable relative to the preceding candidate. This
probe proves only exact root-input identity and recursive-tree shape. It does
not prove recursive package dependency closure, published tarball integrity,
LFS completeness, Actions inventory, source reads, or authority acceptance.
The lock therefore keeps `accepted_for_this_implementation_round=false`,
`baseline_identity_accepted=false`, and all closure/global flags unchanged.
