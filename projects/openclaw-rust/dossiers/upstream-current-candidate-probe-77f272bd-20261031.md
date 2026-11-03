# OpenClaw current candidate probe — 77f272bd (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `77f272bd7e5ad7db4c96e8bc4d440d583ab17e7f`
- Root tree: `a3402ffe65237a9e616e0a287d7b7119c9cd20de`
- Observed: `2026-10-31T11:19:02Z`
- Commit message: `fix(ci): generate published-driver package declarations (#189209)`
- Discovery source: read-only `git ls-remote` plus GitHub commit/tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,057
entries: 61,816 blobs, 2,241 directories, and 0 gitlinks.

## Root inputs

The root tree reports these blob identities:

| Path | Blob |
| --- | --- |
| `package.json` | `e52ea501bd0e2d749e7f4ab2efce5074b81a2a78` |
| `pnpm-lock.yaml` | `83b4975eb96942ec007a2eb9847ccfe98d380bda` |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` |
| `.gitmodules` | absent |

These root identities match the immediately preceding `2464bd85` candidate,
but that limited identity stability does not establish recursive dependency,
archive, license, LFS, semantic-read, or Actions closure.

## Disposition

This is discovery-only evidence. It updates `upstream.lock.json`'s
`live_candidate` and supersedes the prior live candidate. The fixed
implementation subject remains `ad8d12f126d29b0db204187adf5092db80294e70` /
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; no inventory, ledger, authority,
completion flag, or acceptance state is promoted by this probe.
