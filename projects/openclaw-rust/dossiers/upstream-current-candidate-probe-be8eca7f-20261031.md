# OpenClaw current candidate probe — be8eca7f (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `be8eca7f92b2d7c9acb4043debef785906af6188`
- Root tree: `6b24eed3a7d79dcb099ad4c17aee7f85c0ea3842`
- Observed: `2026-10-31T12:33:09Z`
- Commit timestamp: `2026-10-31T12:31:49Z`
- Commit message: `fix(ci): remove leftover internal exports (#189345)`
- Discovery source: read-only GitHub branches/commit/tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,088
entries: 61,847 blobs, 2,241 directories, and 0 gitlinks.

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

Root identity stability is not recursive dependency, archive, license, LFS,
semantic-read, or Actions closure evidence.

## Disposition

This is discovery-only evidence and supersedes the prior live candidate in
`upstream.lock.json`. It does not promote source authority. The fixed
implementation subject remains `ad8d12f126d29b0db204187adf5092db80294e70` /
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; no inventory, ledger, authority,
completion flag, or acceptance state is promoted by this probe. The prior
candidate's inventory, dependency closure, LFS/integrity results, and semantic
reads are not reused. No CI, push, PR update, local OCRM executable
verification, build, package, release, or runtime/device acceptance was run.

The next permitted source-authority step is a fresh exact-subject Actions
inventory/dependency-closure run for this commit/tree. Until then, all global
flags remain false and the implementation freeze remains in force.
