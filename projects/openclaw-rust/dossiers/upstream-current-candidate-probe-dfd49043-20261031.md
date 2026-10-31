# OpenClaw current candidate probe — dfd49043 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `dfd4904312e87674d468d89232180973859e26e0`
- Root tree: `9cf481d8e97f4428d9d9e2f80f941c9cbb7a410d`
- Observed: `2026-10-31T12:36:43Z`
- Commit timestamp: `2026-10-31T12:34:08Z`
- Parent: `159fd3234e928371bcd66a9ece79b665607b343b`
- Commit message: `fix: restore spawn boundary SDK resource lifecycle (#189329)`
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

## Existing upstream Actions observation

At `2026-10-31T12:36:43Z`, read-only GitHub Actions API inspection found 45
runs associated with this exact head. The directly relevant sampled runs were:

| Workflow | Run | Status | Conclusion |
| --- | ---: | --- | --- |
| `CI` | `41201694856` | `pending` | not available |
| `Workflow Sanity` | `41201694363` | `completed` | `success` |
| `CodeQL` | `41201694411` | `completed` | `cancelled` |

These are upstream repository runs, not the Fabushi OCRM exact-H inventory,
dependency/LFS closure, or acceptance receipt. Their presence does not promote
the candidate or change any project flag. The pending CI run remains an
external observation and was not triggered, modified, or treated as approval.
