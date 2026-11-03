# OpenClaw current candidate probe — af300f44 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `af300f447128e0bec692fa0c96d20f360ee8c8b6`
- Root tree: `601cfe03da8faaac3c1d92972298ec763e4d3fca`
- Observed: `2026-10-31T13:03:16Z`
- Commit timestamp: `2026-10-31T13:01:15Z`
- Parent: `adebbe9aab0f82f7529fa8cdae7f1be794d1e195`
- Commit message: `perf(ui): skip unused settings render work (#189355)`
- GitHub commit verification: `verified=true`, reason `valid`
- Discovery source: read-only GitHub branches/commit/tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,102
entries: 61,861 blobs, 2,241 directories, and 0 gitlinks.

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
candidate's inventory, dependency closure, SBOM, LFS/integrity results, notice
audit, and semantic reads are not reused. No CI, push, PR update, local OCRM
executable verification, build, package, release, or runtime/device acceptance
was run.

## Exact commit diff scope

The GitHub commit API reports five changed UI paths relative to parent
`adebbe9aab0f82f7529fa8cdae7f1be794d1e195`:

| Status | Path |
| --- | --- |
| modified | `ui/src/components/config-form.node.collection-map.tsx` |
| modified | `ui/src/components/config-form.node.collection.tsx` |
| modified | `ui/src/components/config-form.node.tsx` |
| modified | `ui/src/pages/plugins/settings-view.tsx` |
| modified | `ui/src/styles/settings.css` |

This is a UI render-performance scope observation, not proof of Rust owner,
production wiring, or acceptance. It does not authorize relabeling fixed-
subject inventory rows, responsibility mapping, implementation, or acceptance.
