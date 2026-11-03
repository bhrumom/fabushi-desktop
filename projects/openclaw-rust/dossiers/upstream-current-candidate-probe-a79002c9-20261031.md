# OpenClaw current candidate probe — a79002c9 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `a79002c971e2231807f3268eb58f20363b5482d6`
- Root tree: `c42693615111af5103940392bb6e318505760f92`
- Observed: `2026-10-31T12:58:01Z`
- Commit timestamp: `2026-10-31T12:55:44Z`
- Parent: `27c5554782e707e06a848c885776c8ed36302e5d`
- Commit message: `fix(test): bind native command fixture SDK hosts (#189386)`
- GitHub commit verification: `verified=true`, reason `valid`
- Discovery source: read-only GitHub branches/commit/tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,101
entries: 61,860 blobs, 2,241 directories, and 0 gitlinks.

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

The GitHub commit API reports three changed paths relative to parent
`27c5554782e707e06a848c885776c8ed36302e5d`:

| Status | Path |
| --- | --- |
| added | `config/test-lines-allocations/l168-command-fixture-hosts.txt` |
| modified | `src/auto-reply/reply/commands-plugin.native-subagents.test.ts` |
| modified | `src/auto-reply/reply/commands-plugin.scheduled-turns.test-support.ts` |

This is a test/fixture scope observation, not proof of a production runtime
change or a Rust owner/wiring change. It does not authorize relabeling
fixed-subject inventory rows, responsibility mapping, implementation, or
acceptance state.
