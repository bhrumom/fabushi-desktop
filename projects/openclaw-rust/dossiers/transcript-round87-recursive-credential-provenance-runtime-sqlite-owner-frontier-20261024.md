# OCRM Round87 — transcript worker runtime, dynamic entrypoint and SQLite lifecycle owner frontier — 2026-10-24

Task: `3dd43a37-9f36-4b7a-adac-dfcc87584523`, report round 78. Additive read-only source audit. Frozen upstream is `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. No Actions/build/test/generator execution; no production code, pairing, Box or Gateway mutation; no ledger/RTM/flags upgrade.

## 1. Target authority and write precondition

- Target repository: `bhrumom/fabushi-desktop`.
- Audit branch: `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- Live ref re-read immediately before this write: `14bb96524b76d30dc1f14b672aa3c2099673454c`; its tree is `a53c2efb75a4f1c9684d96385b7e987cdfea96d4`.
- Round87 dossier was absent at that live ref; Round86 exists as prior evidence and is not overwritten.
- The new dossier is the only intended tree change. Ref update must use `expected_sha=14bb96524b76d30dc1f14b672aa3c2099673454c`, `force=false`. If the lease rejects, re-read the live ref/tree and reconcile before any retry.

## 2. Frozen source identities newly pinned

| Source | Exact blob SHA | Lines | Responsibility |
|---|---|---:|---|
| `src/config/sessions/session-transcript-read-worker-runtime.ts` | `ad0c7325b4e81607c49dd0eddfda7dccb615e99f` | 363 | Canonical worker-facing transcript read facade: context messages/model context/reset recall; worker pool and timeout/error normalization; SQLite engine selection/admission turn; runtime worker URL/entrypoint routing and read fence. |
| `src/config/sessions/session-sqlite-target.ts` | `0c9153419035016e260523479eab420b67e0e93e` | 359 | Resolves SQLite target and registry ownership; dynamically imports worker runtime to resolve targets in worker; checks registered owner/path identity and candidate files. |
| `src/config/sessions/session-transcript-worker-resources.ts` | `ffd341bd00fad8380d81181124bac3ab6aa5d454` | 740 | Owns bounded worker lanes, pending counts, abort joining, idle retirement and async database resource registration for transcript/usage workers. |
| `src/config/sessions/session-transcript-worker-runtime.ts` | `15c75c9536747ed10092d15776a9d01d157b4eef` | 361 | Worker task adapters, database admission turn, actor projection receipt and history read admission. |
| `src/config/sessions/session-transcript-read-worker-runtime.test.ts` | `9c3d1e8352c5db65241d87d533efcd59072b6ee5` | 60 | Focused timeout, closed-pool and unrelated-worker-error behavior oracle. |
| `src/config/sessions/session-transcript-context-read.ts` | `73a3440268f760a4d491ce753ef3ee00184220a2` | 286 | Production model-context read path; fenced actor transcript, writer admission, worker dispatch, and lazy ordered-entry import. |
| `src/agents/sessions/session-manager-incognito.ts` | `6e1f3abc2959c090035970014f5fbd7216fbb0b9` | 224 | Incognito session hydration/context production consumer; carries transcript source and target bindings through worker reads. |
| `packages/memory-host-sdk/src/host/openclaw-runtime-session.ts` | `8860e441f3d62f539cf50b9ae4723f662d83fa51` | 289 | Memory-host runtime facade; lazy-binds transcript worker runtime and inventory runtime while exposing session transcript compatibility methods. |
| `src/config/sessions/session-accessor.sqlite-scope.ts` | `0af27f285fc4362025e6dcd19df34e2f1e80b7f5` | — | Sanctioned SQLite scope/Kysely boundary; uses reader lifecycle and worker-thread context. |
| `src/config/sessions/session-transcript-read-fence.ts` | `f9084d01cb4b3d5db1df8e50844ad98d8a846759` | — | Async-local admission fence binding transcript reads to current user-turn receipt, DB identity and target. |
| `src/config/sessions/session-transcript-read-source.ts` | `c583eafa007d4a68f198db51a870e734916007be` | — | Captures database path identity and read scope before worker dispatch. |
| `src/config/sessions/session-transcript-worker-read.types.ts` | `14bc7c4481b8efe975f9876a8c05083543744064` | — | Typed worker input/output surface with DB identity, transcript admission receipt and host SDK/session types. |
| `src/state/openclaw-database-preflight-cleanup.ts` | `674b0f6b2e4dbad2c26afdef2b949303abc16fee` | — | Existing cleanup owner closes admission, clears Kysely cache, closes native SQLite and retires prepared read-only snapshot; cleanup failures remain explicit. |
| `src/state/agent-database-admission.ts` | `2d9a52db00e9e7f1accf5646683cb4bfe683dafc` | — | Existing admission owner scopes DB access to agent, preparation context and state DB lifetime. |
| `src/state/openclaw-database-preflight.ts` | `b826b7c26a13868ea008902bdd37c08d9d123e06` | — | Startup/restart/Doctor schema readiness and read-only database inspection coordinator (previously pinned; re-confirmed). |
| `src/config/sessions/targets.ts` | `6ebfe0c47d97544d03e55fa584ce697d74748299` | — | Logical/physical session store target and persisted owner resolution (previously pinned; re-confirmed). |

All listed blobs were fetched from the frozen ref in this round; SHA values above are the GitHub API's returned blob identities. Line counts are zero-based file line count minus the terminal newline where applicable and are descriptive only.

## 3. Import and dynamic-entrypoint frontier

### 3.1 Worker runtime direct imports

The complete 363-line worker runtime has these static imports:

- Node path.
- `@openclaw/worker-runtime` `WorkerTaskError`.
- `../../agents/failover/local-preparation-error.js` `ContextPreparationTimeoutError`.
- `../../infra/bun-sqlite-library.js` `ensureSqliteLibrarySelected`.
- `../../infra/database-engine/context.js` `getDatabaseEngineSelection`.
- `../../infra/runtime-process-entrypoints.js` `runtimeProcessEntrypoints`.
- `../../infra/runtime-worker-url.js` `resolveRuntimeWorkerUrl`.
- `../../infra/sqlite-database-admission-turn.js` `runWithSqliteDatabaseAdmissionTurn`.
- `../../infra/worker-task-pool.js` `WorkerTaskPool`; `../../infra/worker-task-pool.types.js` worker options.
- Type-only transcript model-context accessor, runtime target and worker input types.
- `session-history-worker-errors.js`, `session-transcript-read-fence.js`, and `session-transcript-read-pools.js`.

No direct `await import()` occurs in this worker runtime body. Its worker creation is indirect through `runtimeProcessEntrypoints` and `resolveRuntimeWorkerUrl`; therefore the concrete entrypoint map, emitted worker files, package/build manifest and runtime URL mapping are still an open recursive dependency, not presumed resolved by the static imports.

### 3.2 Dynamic edges and callers

- `session-sqlite-target.ts` dynamically imports `./session-transcript-read-worker-runtime.js` inside target resolution. This means worker target resolution and registry-read ordering are part of the same owner/lifecycle contract.
- `session-transcript-context-read.ts` dynamically imports `./session-entry-read-ordered.js` for ordered transcript entry reading and directly calls `readSessionTranscriptModelContextInWorker`; it also uses the target-discovery lane and transcript write context.
- `session-manager-incognito.ts` directly calls worker runtime and context-read/hydration helpers; it binds source/target identity for incognito session reads.
- `openclaw-runtime-session.ts` creates lazy runtime binders for `session-transcript-read-worker-runtime.js` and `session-transcript-inventory-runtime.js`. These binders preserve the host SDK's runtime boundary; the exact lazy binder lifecycle and manifest/packaging registration remain to be traced.
- `session-transcript-worker-resources.ts` owns worker lanes and database-resource registration; `session-transcript-worker-runtime.ts` and the focused test cover adjacent worker-task contracts.
- Existing cleanup helper `cleanupOpenClawStatePreflight` is a distinct startup/schema-preflight cleanup path. It is not by name alone proof that transcript-worker pools, native SQLite handles, snapshots and Kysely cache entries all retire correctly. The resource owners and their teardown callers must be linked explicitly.

## 4. Source identity → test → production path → negative oracle

| Test/source | Production path | Negative oracle (read-only source inspection; not executed) |
|---|---|---|
| `session-transcript-read-worker-runtime.test.ts` `9c3d1e8352c5db65241d87d533efcd59072b6ee5` | worker runtime → worker pool → context preparation timeout/error normalization → model-context read caller | A context preparation timeout must retain its typed timeout identity; a closed worker pool must not be rewritten as success; unrelated `WorkerTaskError` must propagate unchanged. |
| `session-memory-working-set.test.ts` (exact blob not yet pinned in this round) | worker runtime `prepareSessionEntriesInWorker` → memory working-set construction | Large transcript inputs must use worker-backed preparation rather than blocking main-thread work; failure/cancel must not publish a partial working set. Exact test identity and all assertions remain to be fetched. |
| `session-transcript-read-pools.test.ts` (exact blob not yet pinned in this round) | worker resource pool creation → bounded foreground workers and shared worker lane | Concurrent callers must respect pool bounds; idle retirement cannot close resources still in use; abort must join owned tasks and not leak a worker/database. Exact assertions and source SHA remain to be fetched. |
| `bun-sqlite-library.worker.test.ts` (exact blob not yet pinned in this round) | worker runtime → `ensureSqliteLibrarySelected` / SQLite engine selection / worker entrypoint | Wrong SQLite library or target mode must fail closed; worker and main-thread selection must not silently diverge. Exact test SHA and assertion context remain open. |
| `session-transcript-context-read.ts` | user-turn transcript fence + actor read receipt + DB write admission → worker model-context read | Stale or mismatched target/receipt, revoked admission, cancellation, or missing ordered-entry import cannot fall back to an unfenced direct read or publish stale context. Focused test coverage and exact source identity remain open. |
| `session-sqlite-target.ts` + `targets.ts` | physical SQLite candidate → registry snapshot → worker target resolver → persisted owner | Ambiguous/unavailable registry, path escape, wrong agent owner, or conflicting selector must not choose an implicit DB or perform writes before admission. Focused target test remains pinned in Round86 as `cf715aa132832d9c29dc947dee1934e7d3c168da`. |
| `session-transcript-worker-resources.ts` and cleanup/admission modules | worker lane → async resource registration → admission turn → close/cache clear/snapshot retirement | Pool close, abort, database close, snapshot cleanup and cache eviction must settle exactly once; cleanup failure cannot be swallowed or allow stale workers to reuse retired resources. Lifecycle test mapping is not yet complete. |

The above separates observed source contracts from actual test execution. No test was run.

## 5. Existing-owner decision

- Transcript reads remain owned by the existing session accessor/transcript-read and worker-resource modules. No second transcript store, worker pool or DB owner is proposed.
- Physical SQLite identity and persisted logical ownership remain with the existing SQLite target/registry and agent DB owner modules.
- Worker lifecycle belongs to `session-transcript-worker-resources.ts` plus the worker pool/native SQLite/snapshot resource owners; startup database preflight cleanup is not a substitute for this lifecycle.
- Memory-host SDK is a compatibility facade/lazy runtime boundary, not a second authoritative transcript owner.
- Authorization/read freshness stays with the existing transcript read fence, actor receipt and agent DB admission. Worker URL/entrypoint resolution is an execution boundary, not permission to bypass these guards.

## 6. Closure status and next frontier

This dossier records a verified, partial recursive frontier only. It does not assert full dependency closure, independent ACCEPT, scoped G01/G02 positive evidence or unit admission.

Still open and to be read/pinned next:
1. Complete `runtimeProcessEntrypoints` implementation, all references to transcript worker entrypoint keys, `resolveRuntimeWorkerUrl`, package/build manifests, worker source entrypoints and generated/emitted artifacts.
2. Full imports and consumers of `session-transcript-read-pools.ts`, `session-history-worker-errors.ts`, `session-accessor.sqlite-model-context.ts`, `session-history-read.types.ts`, and `session-transcript-worker-resources.ts` dependencies.
3. Exact blob and assertions for `session-memory-working-set.test.ts`, `session-transcript-read-pools.test.ts`, `bun-sqlite-library.worker.test.ts`, plus tests for cancellation, worker death, abort joining, native SQLite close, Kysely cache clear and snapshot cleanup.
4. All consumers of `readSessionTranscriptContextMessagesInWorker`, `readSessionTranscriptModelContextInWorker`, `prepareSessionEntriesInWorker`, `resolveSqliteTargetInWorker` and the host SDK lazy binders; enumerate direct, dynamic and registry/manifest references.
5. Trace every worker/resource owner from acquisition to success, rejection, timeout, abort, caller cancellation, process/worker death, idle retirement and cleanup failure; prove that no stale worker can use a closed database or retired snapshot.

No Actions/build/tests/generator were run. No production/pairing/Box/Gateway code changed. No ledger/RTM/flags changed. OCRM-AC01..24, source closure, mapping, implementation, wiring, ecosystem, UI and release acceptance remain open.
