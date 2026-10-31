# OpenClaw current candidate probe — 2d5703f9 (2026-10-31)

## Evidence identity

- Repository: `openclaw/openclaw`
- Ref: `refs/heads/main`
- Commit: `2d5703f93ee06f45320039323eb6c5ab8733bf15`
- Root tree: `a8760296458322a23cf41c8ab0061813fdb8a225`
- Observed: `2026-10-31T13:09:57Z`
- Parent: `470fe9c2981238977a56c3b8769e530ca54f2c99`
- Commit message: `fix: preserve in-flight work across unrelated auth reloads (#189349)`
- GitHub commit verification: `verified=true`, reason `valid`
- Discovery source: read-only GitHub commit, contents, and recursive-tree API

## Recursive tree

The GitHub recursive-tree response was not truncated and contained 64,106
entries: 61,865 blobs, 2,241 directories, and 0 gitlinks.

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

## Exact commit diff scope

The GitHub commit API reports 21 changed paths relative to parent
`470fe9c2981238977a56c3b8769e530ca54f2c99`:

| Status | Path |
| --- | --- |
| modified | `config/test-lines-allocations/config-reload-inflight-work.txt` |
| modified | `docs/gateway/configuration/hot-reload.md` |
| modified | `scripts/perf/session-event-auth-reload-measure.ts` |
| modified | `scripts/perf/session-resource-loader-fixture.mjs` |
| modified | `src/agents/prepared-model-runtime.configured-refresh.ts` |
| modified | `src/agents/prepared-model-runtime.hot-reload-dispatch.test.ts` |
| modified | `src/agents/prepared-model-runtime.ts` |
| modified | `src/agents/prepared-model-runtime.types.ts` |
| modified | `src/auto-reply/reply/session-event-context.ts` |
| modified | `src/auto-reply/reply/session-event-handoff.ts` |
| modified | `src/auto-reply/reply/session-event-policy.test.ts` |
| modified | `src/auto-reply/reply/session-event-policy.ts` |
| modified | `src/cron/session-run.session-model-strictness.integration.test.ts` |
| modified | `src/gateway/server-reload-hot.ts` |
| modified | `src/gateway/server-reload-model-runtime-scope.ts` |
| modified | `src/gateway/session-row-prepared-read.test.ts` |
| modified | `src/gateway/session-row-prepared-read.ts` |
| modified | `src/gateway/session-row-projection-ancestors.ts` |
| modified | `src/gateway/session-row-projection-materialize.ts` |
| modified | `src/gateway/session-row-projection.reload.test.ts` |
| modified | `src/gateway/session-row-projection.ts` |

The change is a live-source drift observation affecting auth reload, prepared
model runtime refresh, session-row projection, and cron session-model
strictness. It is not proof of a Rust owner, production wiring, or acceptance.

## Disposition

This is discovery-only evidence and supersedes the prior live candidate in
`upstream.lock.json`. It does not promote source authority. The fixed
implementation subject remains
`ad8d12f126d29b0db204187adf5092db80294e70` /
`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`; no inventory, ledger, authority,
completion flag, or acceptance state is promoted by this probe. The prior
candidate's inventory, dependency closure, SBOM, LFS/integrity results, notice
audit, and semantic reads are not reused. No CI, push, PR update, local OCRM
executable verification, build, package, release, or runtime/device
acceptance was run.
