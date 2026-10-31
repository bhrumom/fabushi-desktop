# OpenClaw current candidate probe — f8e492f2 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `f8e492f2c9d1cc3768c222b909a82bd3e4f97455`
- Root tree: `89257a1e739896c29fbc427e43e2c59d95ab02a1`
- Observed: `2026-10-31T12:42:39Z`
- Commit timestamp: `2026-10-31T12:39:04Z`
- Parent: `ed19b276e1630a14c66ddf8cc4594d2915ba8c56`
- Commit message: `test(ui): expose reconnect service worker policy (#189374)`
- GitHub commit verification: `verified=true`, reason `valid`
- Discovery source: read-only GitHub branches/commit/tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,090
entries: 61,849 blobs, 2,241 directories, and 0 gitlinks.

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

## Exact commit diff scope

The GitHub commit API reports two changed paths relative to parent
`ed19b276e1630a14c66ddf8cc4594d2915ba8c56`:

| Status | Path |
| --- | --- |
| modified | `scripts/README.md` |
| modified | `scripts/bench-control-ui-reconnect.ts` |

This is a benchmark/script scope observation, not proof of a production runtime
change or a Rust owner/wiring change. It does not authorize relabeling
fixed-subject inventory rows, responsibility mapping, implementation, or
acceptance state.

The exact-tree declaration-input count for this subject is recorded separately
in `dossiers/upstream-current-candidate-declaration-frontier-f8e492f2-20261031.md`;
that frontier remains structural evidence and does not establish dependency
closure.
