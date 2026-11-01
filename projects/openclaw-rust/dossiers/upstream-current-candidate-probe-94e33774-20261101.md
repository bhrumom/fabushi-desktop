# OpenClaw current candidate probe — 94e33774 (2026-11-01)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `94e33774c5fba3da3d3e521f47a2179d11113405`
- Root tree: `7228383c9d7c5e025def3ae9a69eb1c75d3c1ea0`
- Observed commit time: `2026-11-01T01:34:09Z`
- Parent: `cdf14a50f59a5abe5ce3df008d3141e9e789f2cc`
- Discovery source: read-only `git ls-remote` plus GitHub commit/tree API

The GitHub commit signature is verified. The commit message is:
`refactor(embeddings): decouple usage decoding from memory transport (#189865)`.

## Exact commit delta

The commit API reports five modified paths (25 additions, 26 deletions):

- `packages/llm-core/src/usage-cost.ts`
- `packages/memory-host-sdk/src/engine-embeddings.ts`
- `packages/memory-host-sdk/src/host/embeddings-remote-fetch.ts`
- `src/plugins/embedding-provider-types.ts`
- `src/plugins/openai-compatible-embedding-provider.ts`

This is a fresh live-main subject. The embedding and usage-cost semantic delta
and all transitive source/dependency consequences require a new exact-subject
read; the prior `02b28c9d` probe is not reused as closure evidence.

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,457
entries: 62,214 blobs, 2,243 directories, and 0 gitlinks.

## Root inputs

| Path | Blob |
| --- | --- |
| `package.json` | `864764f48ec4f1a7946b880b10a14d96de4b6d78` |
| `pnpm-lock.yaml` | `ba24cee97daeeb709109eb7d2b6a9efdfac0f310` |
| `pnpm-workspace.yaml` | `32ce4d83de4aac6a1e3665baba09a3aa9582960c` |
| `.gitattributes` | `c6118b226a4a760f7861b6b7b1e7dccd943384e4` |
| `LICENSE` | `ed064819ab537575d0cad902f7e5af4a3488b1d0` |
| `THIRD_PARTY_NOTICES.md` | `0c9c123760188915a384ce7275df4096114e8f90` |
| `.gitmodules` | absent |
| `AGENTS.md` | `6eedd536b6f5145d5d158a48415a8c0e1cd8db7e` |

Root identity stability is not recursive dependency, archive, license, LFS,
semantic-read, or Actions closure evidence.

## Fabushi applicability probe

The five-file delta is an upstream dependency/normalization refactor for
embedding-provider usage reporting. A narrow non-test search of the current
Fabushi host found the existing embedding registration contract in
`source/host/src/extensions/plugins/capability_catalog.rs`, but no production
embedding-vector transport, usage-cost owner, or memory embedding consumer
under `source/host/src`. The existing registry contract therefore remains the
nearest owner candidate for plugin registration only; it is not evidence that
the upstream remote embedding transport is implemented or wired in Fabushi.
This probe is an applicability/disposition lead, not a complete source
closure or owner audit, and no ledger status is promoted from it.

## Disposition

This is discovery-only evidence and supersedes the prior live candidate in
`upstream.lock.json`. It does not promote source authority. The fixed
implementation subject remains `1440bb1771372334bea23d7642bb52deb7db1845` /
`90c8aab39972da11cfad220361781d49e1bc6700`; no inventory, ledger, authority,
completion flag, or acceptance state is promoted by this probe. Prior candidate
inventory, dependency closure, LFS/integrity and semantic reads are not reused.
