# OpenClaw current candidate probe — 02b28c9d (2026-11-01)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `02b28c9d0b5ae329b07a1e758c9cfca8f1798bca`
- Root tree: `76940ff14f9d37a1729a6b0a2d8e466026d4a554`
- Observed: `2026-11-01T01:18:36Z`
- Commit message: `perf(ui): reduce shared agent selector setup and focus work (#189843)`
- Parent: `4b709b8565dc42aaa3d4aea76eed7c7bc5d36fb4`
- Discovery source: read-only `git ls-remote` plus GitHub commit/tree API

The GitHub commit signature is verified (`reason=valid`, verified at
`2026-11-01T01:18:37Z`).

## Exact commit delta

The commit API reports nine changed paths relative to parent
`4b709b8565dc42aaa3d4aea76eed7c7bc5d36fb4`:

- `scripts/bench-control-ui-engines.ts`
- `scripts/lib/control-ui-page-observer.ts`
- `test/scripts/control-ui-page-observer.test.ts` (added)
- `ui/src/components/markdown-file-links.ts`
- `ui/src/components/solid/menu-machine.ts`
- `ui/src/components/solid/menu-runtime.ts`
- `ui/src/components/solid/menu.tsx`
- `ui/src/pages/new-session/new-session-page.session-facts.test.ts`
- `ui/src/pages/new-session/new-session-page.tsx`

This bounded delta is discovery evidence only. It does not establish that any
other path is unchanged in semantic responsibility, and it is not production
Rust wiring, dependency closure, or acceptance evidence.

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,445
entries: 62,202 blobs, 2,243 directories, and 0 gitlinks.

## Root inputs

| Path | Blob |
| --- | --- |
| `package.json` | `6989c0618742d3c2b93cc336cbe1d9c3c4f0a657` |
| `pnpm-lock.yaml` | `ba24cee97daeeb709109eb7d2b6a9efdfac0f310` |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` |
| `.gitmodules` | absent |
| `AGENTS.md` | `6eedd536b6f5145d5d158a48415a8c0e1cd8db7e` |

Root identity stability is not recursive dependency, archive, license, LFS,
semantic-read, or Actions closure evidence.

## Disposition

This is discovery-only evidence. It supersedes the prior live candidate in
`upstream.lock.json` but does not promote source authority. The fixed
implementation subject remains `ad8d12f126d29b0db204187adf5092db80294e70` /
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; no inventory, ledger, authority,
completion flag, or acceptance state is promoted by this probe. Prior candidate
inventory, dependency closure, LFS/integrity and semantic reads are not reused.

