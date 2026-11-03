# OpenClaw current candidate probe — eb0183de (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `eb0183deb6891f07686a1f3cefeaf063c119aaa7`
- Root tree: `6c66d83fbeba3e74a7142073a2980bd2433722a8`
- Observed: `2026-10-31T12:53:51Z`
- Commit timestamp: `2026-10-31T12:49:42Z`
- Parent: `485eb3c19d670884b41c6abe59879bd041eaaa95`
- Commit message: `chore: speed up state lease and preservation tests (#189366)`
- GitHub commit verification: `verified=true`, reason `valid`
- Discovery source: read-only GitHub branches/commit/tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,097
entries: 61,856 blobs, 2,241 directories, and 0 gitlinks.

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

The GitHub commit API reports two changed paths relative to parent
`485eb3c19d670884b41c6abe59879bd041eaaa95`:

| Status | Path |
| --- | --- |
| modified | `src/state/openclaw-agent-db-maintenance.schema-renewal.test.ts` |
| modified | `src/state/openclaw-state.worker.settlement.test.ts` |

This is a test-only scope observation, not proof of a production runtime change
or a Rust owner/wiring change. It does not authorize relabeling fixed-subject
inventory rows, responsibility mapping, implementation, or acceptance state.
