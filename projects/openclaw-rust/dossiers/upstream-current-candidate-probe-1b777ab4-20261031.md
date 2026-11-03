# OpenClaw current candidate probe — 1b777ab4 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `1b777ab47da016c998244d7bcb377a72b5b4bba5`
- Root tree: `33cb615b384021957d00cec5c5e8183c05ee18ef`
- Observed: `2026-10-31T12:06:03Z`
- Commit message: `fix(docker): preserve SDK imports for bundled plugin helpers (#181974)`
- Discovery source: read-only `git ls-remote` plus GitHub commit/tree/content API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,072
entries: 61,831 blobs, 2,241 directories, and 0 gitlinks.

## Root inputs

| Path | Blob |
| --- | --- |
| `package.json` | `e52ea501bd0e2d749e7f4ab2efce5074b81a2a78` |
| `pnpm-lock.yaml` | `83b4975eb96942ec007a2eb9847ccfe98d380bda` |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` |
| `.gitmodules` | absent |

The tree/count/root-input stability relative to the prior candidate does not
prove recursive dependency closure, archive bytes, license, LFS, semantic-read,
or Actions closure.

## Disposition

This is discovery-only evidence. It supersedes the prior live candidate in
`upstream.lock.json` but does not promote source authority. The fixed
implementation subject remains `ad8d12f126d29b0db204187adf5092db80294e70` /
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; no inventory, ledger, authority,
completion flag, or acceptance state is promoted by this probe. Prior candidate
inventory, dependency closure, LFS/integrity and semantic reads are not reused.
