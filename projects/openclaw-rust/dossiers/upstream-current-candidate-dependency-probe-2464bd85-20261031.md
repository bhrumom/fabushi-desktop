# OpenClaw current candidate root dependency probe

Status: discovery-only; not a dependency-closure or source-authority receipt

## Exact subject

- repository: `openclaw/openclaw`
- ref: `refs/heads/main`
- commit: `2464bd85e0bc2927701c3147bf3dfc6993daaae3`
- root tree: `08b1e6627c310e8640a340be49838e2936b44346`
- observed: `2026-10-31T09:47:18Z`
- recursive tree: 64,009 entries, 61,768 blobs, 2,241 directories, 0 gitlinks
- recursive tree API: `truncated=false`

## Root dependency inputs

- `package.json` blob: `e52ea501bd0e2d749e7f4ab2efce5074b81a2a78`
- `pnpm-lock.yaml` blob: `83b4975eb96942ec007a2eb9847ccfe98d380bda`
- `pnpm-workspace.yaml` blob: `32ce4d83de4aac6a1e3665baba09a3aa9582960c`
- `.gitattributes` blob: `c6118b226a4a760f7861b6b7b1e7dccd943384e4`
- `LICENSE` blob: `ed064819ab537575d0cad902f7e5af4a3488b1d0`
- `THIRD_PARTY_NOTICES.md` blob: `0c9c123760188915a384ce7275df4096114e8f90`
- `.gitmodules`: absent

The exact compare from the previous candidate
`d27a8c9d9fec49a81084993bf16c3fefc4efd508` reports ahead 2, behind 0, and
seven changed files:

- `src/gateway/server-plugin-reload.meeting-transcripts.test.ts`
- `src/gateway/x-publication-owner-composition.e2e.test.ts`
- `ui/src/pages/sessions/agent-scope.ts`
- `ui/src/pages/sessions/session-row.tsx`
- `ui/src/pages/sessions/sessions-page-state.ts`
- `ui/src/pages/sessions/sessions-page.test.ts`
- `ui/src/pages/sessions/view.tsx`

None of those files is a root dependency input. This permits carrying forward
only the narrow root-input identity observation, not inventory, semantic
source reads, LFS results, dependency closure, or Actions provenance.

The root manifest remains OpenClaw 2026.10.9 with integrity-qualified
`pnpm@12.15.0` and 77 runtime, 57 dev, 1 optional and 0 peer dependencies.
The lock remains structurally unresolved against that manifest: one importer,
15 pnpm package/snapshot entries, and no root application dependency closure.
The candidate therefore remains discovery-only with all authority and closure
flags false; prior candidate inventory, dependency, LFS and Actions evidence
cannot be reused.
