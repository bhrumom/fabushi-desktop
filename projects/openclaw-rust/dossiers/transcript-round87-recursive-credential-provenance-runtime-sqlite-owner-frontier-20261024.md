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


## 7. Continued read-only closure after Round87 delivery

The branch was re-read after the initial Round87 commit before this follow-on evidence write. The frozen upstream identities below extend the worker/manifest and cleanup frontier; they do not establish execution success.

| Dependency / oracle | Exact blob SHA | Evidence and open condition |
|---|---|---|
| `src/infra/runtime-process-entrypoints.ts` | `18071b162512d6975ba7e281d4b2b152cc19f28b` | Eight-line declaration wrapper delegates to `runtime-process-manifest.ts`; exact manifest and `sessionTranscript` mapping must be read next. |
| `src/infra/runtime-worker-url.ts` | `77701a6697c824f5229b86f634e0c0ca635495bf` | Resolves installed `dist` path, stable packaged path, or source sibling URL; has argv resolution helper. Runtime source-vs-packaged path parity and all manifest consumers remain to verify. |
| `src/config/sessions/session-transcript-read-pools.ts` | `b385c34a0a015ee98fc81c541a7c22d05daa4a5a` | Calls `resolveRuntimeWorkerUrl(runtimeProcessEntrypoints.sessionTranscript)`; captures SQLite close policy and creates worker pools. |
| `src/config/sessions/session-transcript-read-pools.test.ts` | `cbb10d47a221f4533fc0739281c425370e39c4a8` | Focused worker-pool test uses real `node:worker_threads`; includes isolation assertion that closing an unrelated DB cannot retire a replacement worker. Remaining assertions must be read in full and mapped to pool retirement behavior. |
| `src/config/sessions/session-memory-working-set.test.ts` | `2e66effe75408f73431e94ab0827f15e22b533b5` | Tests memory working-set construction and imports cleanup/session accessor helpers; still requires full assertion-to-worker-path mapping. |
| `src/infra/bun-sqlite-library.worker.test.ts` | `777bbf05672d062acbe65d0de8fca90261cfe9d4` | 542-line worker/native-close oracle; imports compiled-subprocess preparation and tests explicit SQLite close capability. Full test sections need classification against transcript worker acquisition/retirement and cleanup paths. |
| `src/config/sessions/session-transcript-worker-read-admission.ts` | `91fd8fc2ea73767d5553917e516fd6968b7fce3e` | Admission helper wraps database admission turn, checks schema admission and arms worker idle retirement. |
| `src/config/sessions/session-history-worker-errors.ts` | `d73ba55054ddcbf8803dafffb1e12837a66f05c7` | Typed worker error transport preserves encoded OpenClaw state errors and differentiates database-close/worker-retirement stages. |
| `src/infra/worker-task-pool.ts` | `5a8565c4678f35b029c24921a62e14c7f026755b` | Adapter over `@openclaw/worker-runtime` `WorkerTaskPoolCore`, creates worker task host and exposes close lifecycle. |
| `src/infra/sqlite-database-admission-turn.ts` | `20291bebe683ceed619f83578979a94fcdccef77` | Async-local database admission turn and abort-aware task boundary; exact consumers and release behavior remain to enumerate. |
| `src/infra/sqlite-reader-lifecycle.ts` | `ff40ee42e0b58f04aee9c24ef7e69e8ebb013112` | Registers reader connections by worker-thread/database identity; must be followed through close and reader-owner release. |
| `src/infra/sqlite-readonly-location-cleanup.ts` | `851b9cb4ed4f6880712bc48fbdf1423087a97e8c` | 585-line retained-operation/snapshot cleanup owner, registers signal-exit finalizer, tracks snapshot retirement and explicit native-close capability. |
| `src/infra/sqlite-snapshot-source.ts` | `14931e709dbccb2daab535c15201dfc671994904` | Owns private readers/snapshots until native close and snapshot disposal settle; worker retirement/close/closed waits are explicit source-level lifecycle edges. |
| `src/infra/kysely-sync.ts` | `ed72c6bd8ad4e6d35fd3be19b48e79a2cfc3ff45` | Kysely statement disposal/DB-specific cache cleanup is a separate edge to trace into preflight and transcript reader teardown. |

### 7.1 Revised evidence boundary

- The worker entrypoint chain is now known to be `session-transcript-read-worker-runtime.ts → runtimeProcessEntrypoints.sessionTranscript → runtime-process-manifest.ts → resolveRuntimeWorkerUrl`, with pool construction in `session-transcript-read-pools.ts`. The manifest itself, its exact key-to-file mapping, build output, package files and generated subprocess registration are not yet pinned; do not claim packaging closure.
- Cleanup is a graph of distinct owners: worker pool/core and owned-task join; DB admission turn; reader connection lifecycle; Kysely cache/statement disposal; private SQLite snapshot/retained operation; native close capability; and signal-exit cleanup. A successful call to one cleanup helper cannot stand in for the full graph.
- Test blob identities for read pools, memory working set and SQLite worker/native close are now pinned, but this round has not yet read every assertion or executed them. Source inspection is not a test pass.
- Next immediate read-only step: fetch `src/infra/runtime-process-manifest.ts` in full, enumerate every `sessionTranscript` reference and build/package registration, then read the complete pool/native-close test bodies and trace the close/abort/error paths back to all production consumers.


## 8. Exact manifest and full negative-oracle reads (continued)

The latest live branch ref was re-read before this update; this addendum is applied with a non-force compare-and-swap against that ref. Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`.

| Source | Exact blob SHA | Full-read finding |
|---|---|---|
| `src/infra/runtime-process-manifest.ts` | `574271fcb7457427d90fbe9d94a57312f938bafa` | `sessionTranscript` is explicitly mapped to `config/sessions/session-transcript.worker`; `runtimeProcessEntrypoints` derives source sibling and `dist/config/sessions/session-transcript.worker.js` paths. This is a manifest declaration, not yet proof that the target file is in every distribution. |
| `src/config/sessions/session-transcript-read-pools.ts` | `b385c34a0a015ee98fc81c541a7c22d05daa4a5a` | History pool uses owned worker task pool, read-only DB access, bounded V8 heap (512 MiB old / 32 MiB young), native-close capability policy, and rotation callback; usage lanes use worker pools and report closed DBs. |
| `src/config/sessions/session-transcript-read-pools.test.ts` | `cbb10d47a221f4533fc0739281c425370e39c4a8` | Full body read. Five target selections (exact/family/alias/history-reuse/history-first) verify close joins foreground worker, then pool reuses a replacement; closing unrelated DB cannot retire replacement; state DB close retires it. A pending read waiting on DB admission must reject with “Session transcript database read was revoked” before it reopens the store. The test cleanup explicitly terminates retained workers on failing paths. |
| `src/config/sessions/session-memory-working-set.test.ts` | `2e66effe75408f73431e94ab0827f15e22b533b5` | Full body read. A 72-entry medium transcript workload uses production worker preparation with 512 MiB worker limit; every entry must preserve head/tail markers and exact 524,288 synthetic-word count. Large raw history classification test keeps the intended user message and avoids retaining all noise events as the complete history. |
| `src/config/sessions/session-transcript-read-worker-runtime.test.ts` | `9c3d1e8352c5db65241d87d533efcd59072b6ee5` | Focused tests cover typed context-preparation timeout, closed pool behavior and propagation of unrelated `WorkerTaskError`; exact source read, no execution. |
| `src/infra/bun-sqlite-library.worker.test.ts` | `777bbf05672d062acbe65d0de8fca90261cfe9d4` | Exact 542-line test identity confirmed. The test includes real Worker subprocess cases and native-close capability probes; complete section-to-consumer mapping remains open. |

### 8.1 Direct production consumer search

Frozen-ref search returned these direct call sites for `readSessionTranscriptModelContextInWorker`:
- `src/config/sessions/session-transcript-context-read.ts` — production model-context route.
- `src/agents/sessions/session-manager-fixtures.test-support.ts` — test interception support only.
- `src/infra/bun-sqlite-library.worker.test.ts` — worker/native SQLite oracle.
- `src/agents/sessions/sdk.auth-migration.test.ts` — migration behavior test.
- `src/config/sessions/session-manual-compact.worker.test.ts` — manual compaction worker test.
- `src/config/sessions/session-transcript-context-read.worker.test.ts` — read/close interleaving test.
- `src/gateway/worker-environments/worker-turn-detached-context.test.ts` — detached context mutation/freshness oracle.
- `src/agents/sessions/session-manager-model-context-snapshot.test.ts` — session-manager context snapshot oracle.

Search for `runtimeProcessEntrypoints.sessionTranscript` additionally found `scripts/lib/runtime-process-core-build-entries.mts`, which explicitly registers this entrypoint in the core build entry list, and the pool module above. Separate `sessionTranscriptArchive`, `sessionTranscriptProjectionPublication`, `sessionTranscriptReports` and `sessionTranscriptReconcile` entries belong to adjacent specialized workers; do not conflate those workers with the transcript read runtime.

### 8.2 Updated negative-oracle map

- **Worker close/reuse:** DB close must join the matching foreground reader; an unrelated DB close must not retire a replacement worker; a state DB close must retire that replacement.
- **Admission race:** a read queued behind an admission turn must be revoked when its target DB closes and cannot reopen the database after admission is released.
- **Bounded memory:** multi-entry working-set conversion must preserve exact content and markers without retaining all projected sources and rendered results simultaneously; a large event history must still preserve the selected message-bearing entry.
- **Worker error identity:** timeout remains typed, closed-pool failures stay failures, unrelated worker errors propagate unchanged.
- **Packaging:** the manifest's declared source and dist paths must match the actual worker entry file, build entry registration, and package inclusion/exclusion rules. Until those files and package artifacts are traced, packaged-runtime closure remains unproven.

These are source-level assertions only. No tests, build, packaging or Actions were executed. No implementation was changed, and no acceptance flag was upgraded.
