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


## 9. Worker source and packaging closure evidence (partial)

The branch live ref was re-read immediately before this write. Source is still pinned to the frozen upstream commit above.

| Source | Exact blob SHA | Verified source fact |
|---|---|---|
| `src/config/sessions/session-transcript.worker.ts` | `4d18ec90f10f6cff1a1f2e6823fb939c6c4a9732` | Worker entrypoint is present at the exact manifest-derived source path. Its full 31,859-byte body was fetched. It installs database engine selection from worker data, routes typed worker messages, handles session/transcript/SQLite maintenance kinds, and includes close handling that validates captured physical paths, releases read validation and prunes closed history scopes. Every message-kind branch and all dependencies still need detailed mapping. |
| `scripts/lib/runtime-process-core-build-entries.mts` | `9d1b728dc156a80a1fcf3e22e0a69db03683621d` | Core build entry list consumes `runtimeProcessEntrypoints.sessionTranscript`. This is positive source registration evidence, but the actual generated artifact/package inclusion and npm/portable-deploy inclusion rules are not yet verified. |
| `src/config/sessions/session-transcript-context-read.worker.test.ts` | `b761d295820ba7c1c3d6faf4928bf3385b35ca4c` | 22,820-byte focused concurrency test body pinned; it includes read/close interleaving and checks that disclosure after close is refused. Complete test-case inventory still needs to be mapped. |
| `src/infra/worker-task-host.ts` | `25a837385fd6bbca851c369abf2091cbd798f233` | Host for worker task pool uses MessageChannel, detached async context, DB reader fences and explicit worker retirement state; this is a cross-cutting lifecycle owner. |
| `src/state/openclaw-agent-db-lifecycle.ts` | `581228a0469b553b9ae23beb494374c92c8ec68a` | Agent DB lifecycle closes engine/native handles with explicit finally-path cleanup and exposes async-resource registration. Exact caller chain to transcript worker retirement remains open. |
| `src/state/openclaw-state-db-async-lifecycle.ts` | `fefa1e8c9dffbac0967263c91dbe74ede6d29fb9` | State DB lifecycle keeps failed native retirement under custody for explicit retry; failed drainage does not silently release its seal. Exact consumer and race paths need to be joined to transcript pool closure. |
| `src/infra/runtime-worker-url.ts` | `77701a6697c824f5229b86f634e0c0ca635495bf` | Full source read confirms installed-root/dist/source-sibling resolution and Node TypeScript loader selection for source URLs. |

### 9.1 Source identity → test → production path → negative oracle

- Worker manifest `574271fcb7457427d90fbe9d94a57312f938bafa` → build entry registry `9d1b728dc156a80a1fcf3e22e0a69db03683621d` → `session-transcript.worker.ts` `4d18ec90f10f6cff1a1f2e6823fb939c6c4a9732` → runtime pool `b385c34a0a015ee98fc81c541a7c22d05daa4a5a` / worker facade `ad0c7325b4e81607c49dd0eddfda7dccb615e99f`.
- Negative oracle: build entry must point to the same worker implementation as source/installed runtime URL; worker messages must be validated against captured DB physical paths; close must release validation and prune only closed history scopes; unknown or malformed worker messages must not produce a success result. The exact message-kind table and negative branches remain to be extracted from the worker's full body.
- Pool lifecycle tests `cbb10d47a221f4533fc0739281c425370e39c4a8` and read/close concurrency tests `b761d295820ba7c1c3d6faf4928bf3385b35ca4c` are test-source identities, not executed evidence. The native SQLite test `777bbf05672d062acbe65d0de8fca90261cfe9d4` still requires per-case mapping to close policy, worker retirement and retained snapshot cleanup.
- Package version/lock/toolchain are represented in the frozen root package manifest `af543c4c033b5c5e0c466061ada470feeda777f3`; its presence alone does not prove the transcript worker's published package contents.

Remaining work: enumerate the worker's full message protocol and every import; read the runtime-process build helper's full inclusion/exclusion behavior and relevant package scripts; trace all worker lifecycle callers from DB-close request through worker retirement, native close, Kysely cache eviction, snapshot retirement and retry custody; enumerate all direct/dynamic consumers and negative tests. No Actions, build, package or tests were run. Acceptance remains open.


## 10. Worker message protocol and native cleanup owner frontier

This section follows the previously pinned worker entrypoint through its full source body (710 lines) and pins the exact worker/cleanup identities. It is still a source audit, not executed evidence.

| Source | Exact blob SHA | Newly confirmed scope |
|---|---|---|
| `src/config/sessions/session-transcript.worker.ts` | `4d18ec90f10f6cff1a1f2e6823fb939c6c4a9732` | Full body has distinct request kinds for CLI process history, SQLite target, usage cost, transcript search, cleanup, lifecycle artifact plan, prewarm, eviction candidates, pending archives, transcript stats, transcript instances, memory targets, archive/corpus inventory and pruning, cold metadata/storage inventory, session store target/retirement/exact entries/row facts/current entry/target inventory/topology/identity evidence/entry list/store projection/summary, voice sessions, membership, projection status, members/reactions/suggestions/pending-input history, conversation bindings/rows/delivery, goal receipt, progress card, row presence, session entries, activity summary, transcript hydration/maintenance/current-turn entry/recent active events/latest active message/history page/reset recall, plus a dedicated close request. Some request branches dynamically import specialized worker modules. The full protocol has not yet been individually mapped to every input type and test. |
| `scripts/lib/runtime-process-core-build-entries.mts` | `9d1b728dc156a80a1fcf3e22e0a69db03683621d` | 82-line build helper defines core/standalone runtime-process entry arrays and shared-vs-standalone filtering. Exact downstream package build scripts/artifact manifests remain to be verified. |
| `src/config/sessions/session-transcript-context-read.worker.test.ts` | `b761d295820ba7c1c3d6faf4928bf3385b35ca4c` | Full 534-line body includes real DB admission receipts, read/write interleavings, transcript fence and read-owner close during a scan; verifies detached context is rejected when DB ownership closes, rather than being disclosed as valid. |
| `src/infra/worker-task-host.ts` | `25a837385fd6bbca851c369abf2091cbd798f233` | Worker task host includes MessageChannel, detached async context, deleted-agent DB fences, database engine selection, worker retirement state and live pool tracking. |
| `src/state/openclaw-agent-db-lifecycle.ts` | `581228a0469b553b9ae23beb494374c92c8ec68a` | Agent DB lifecycle includes async close and resource registration; all call edges to worker retirement and reader release still need enumeration. |
| `src/state/openclaw-state-db-async-lifecycle.ts` | `fefa1e8c9dffbac0967263c91dbe74ede6d29fb9` | Failed native retirement retains resource custody for an explicit retry; cleanup/drain failures do not silently release the seal. |
| `src/infra/sqlite-readonly-location-cleanup.ts` | `851b9cb4ed4f6880712bc48fbdf1423087a97e8c` | Full source contains retained-operation tracking, snapshot-reader counts, deferred cleanup after successful reads, async removal, failure reporting, signal-exit finalizer and cleanup custody that is not discharged by a rejected close. |
| `src/infra/sqlite-snapshot-source.ts` | `14931e709dbccb2daab535c15201dfc671994904` | Snapshot scope retains private readers until native close and snapshot disposal settle; worker close/closed waits and recovery mode are explicit. |
| `src/config/sessions/session-transcript-read-pools.test.ts` | `cbb10d47a221f4533fc0739281c425370e39c4a8` | Five selection modes plus queued-read revocation are asserted as documented in §8. |
| `src/infra/bun-sqlite-library.worker.test.ts` | `777bbf05672d062acbe65d0de8fca90261cfe9d4` | Native-close and worker mode test suite identity confirmed; per-case full mapping remains open. |

### 10.1 Production graph and negative oracle

- Entry chain: `runtime-process-manifest.ts.sessionTranscript` → source sibling `config/sessions/session-transcript.worker.ts` / dist path `dist/config/sessions/session-transcript.worker.js` → core build entry list → worker task pool URL resolver → worker runtime message dispatcher.
- Read API chain: worker facade exports `readSessionTranscriptModelContextInWorker`, `readSessionTranscriptContextMessagesInWorker`, `prepareSessionEntryInWorker`, `prepareSessionEntriesInWorker`, `readSessionEntrySnapshotInWorker`, `readSessionResetRecallCutoffInWorker` and `resolveSessionSqliteTargetInWorker`; current production caller confirmed at `session-transcript-context-read.ts`, with incognito manager and memory-host lazy binder as additional consumers. The direct search result set also includes test-only consumers.
- Worker has dynamic imports for cleanup service, lifecycle artifact cleanup, history snapshot, SQLite transaction helpers, retirement reader and entry-current admission. These dynamically loaded modules must be pinned and their source-to-worker protocol mapped; they are not closed merely because the static worker entry exists.
- Cleanup chain spans worker task pool/core and owned-task joining; worker host; database admission turn; agent/state database lifecycle; reader lifecycle registration; Kysely cache/statement disposal; private snapshot retained operations; native close capability; signal-exit finalization; retry custody after failure.
- Negative oracle: invalid/malformed message kinds cannot return a successful unrelated result; each message must use its expected DB admission and captured physical target; a close request requires captured physical paths, releases validation and prunes closed history scope; after DB close, queued and already-scanned context reads must be revoked before disclosure; failed native close/snapshot cleanup must preserve resource custody for explicit retry; unrelated DB closure cannot retire another DB's replacement worker.

### 10.2 Remaining closure

Still not complete: enumerate all worker message input/output types and specialized dynamic imports; trace all consumers of worker exports including `resolveSessionSqliteTargetInWorker` and `prepareSessionEntriesInWorker`; inspect build script/package rules that emit and ship the entrypoint; enumerate native-close test cases; and link each cleanup owner from acquisition to release, cancellation, worker death, retry and final disposal. No Actions/build/test/package execution occurred. No production code or protected project records changed.


## 11. Round79 additive closure: protocol types, dynamic imports, consumers, build packaging and cleanup custody

This section is additive evidence for report round 79. The frozen upstream remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`; no Actions, build, test, generator or package command was executed. No pairing, Box, Gateway production code, ledger, RTM or completion flag was modified.

### 11.1 Worker import closure and typed request/reply surface

The exact `src/config/sessions/session-transcript.worker.ts` blob remains `4d18ec90f10f6cff1a1f2e6823fb939c6c4a9732`. Its complete static import-module set is:

- `node:diagnostics_channel`
- `node:v8`
- `node:worker_threads`
- `../../infra/agent-database-readers.js`
- `../../infra/database-engine/context.js`
- `../../infra/session-cost-usage-worker.types.js`
- `../../infra/worker-task-server.js`
- `../config-env-vars.js`
- `./session-accessor.sqlite-entry-availability.js`
- `./session-history-read-operation-kind.js`
- `./session-history-worker-errors.js`
- `./session-transcript-projection-facts-scope.js`
- `./session-transcript-worker-scopes.js`
- `./session-transcript-worker.types.js`

Its complete literal dynamic-import module set at the frozen blob is:

`../../state/openclaw-agent-db-validation-cache.js`, `../../gateway/cli-session-history.process-held.js`, `./session-sqlite-target.js`, `../../infra/session-cost-usage-worker.js`, `./session-transcript-search.js`, `./session-history-read-operation.worker.js`, `./cleanup-service-read.worker.js`, `./session-accessor.sqlite-lifecycle-artifacts.js`, `../../state/openclaw-agent-db-readonly.js`, `../../gateway/session-history-worker-reader.js`, `../../gateway/server-methods/chat-history-page-kernel.js`, `../../gateway/server-methods/chat-history-response-page.js`, `../../gateway/session-history-snapshot.js`, `../../infra/sqlite-transaction.js`, `./session-history-eviction-candidates.js`, `./session-accessor.sqlite-archive-store-kernel.js`, `./session-accessor.sqlite-read.js`, `./session-history.js`, `./session-memory-targets.js`, `./session-accessor.sqlite-history.js`, `../../../packages/memory-host-sdk/src/host/session-transcript-corpus.js`, `./session-accessor.sqlite-archive-read.js`, `./session-history-archive-pruning.worker.js`, `./session-cold-storage-worker.js`, `./session-store-target-inventory.js`, `./session-retirement-read.worker.js`, `./session-entry-read.worker.js`, `./session-entry-current-admission.worker.js`, `./combined-store-topology.worker.js`, `./session-accessor.sqlite-entry-availability.js`, `./session-canonical-key.js`, `./session-entry-list.worker.js`, `./session-accessor.sqlite-summary.js`, `../../talk/client-voice-session-lookup.worker.js`, `./session-membership-facts.js`, `./session-transcript-index.js`, `./session-sharing-store.kernel.js`, `./session-reaction-store.read.js`, `./session-suggestion-store.kernel.js`, `./session-pending-input-history-read.worker.js`, `./conversation-binding-facts.worker.js`, `./session-accessor.sqlite-conversation-read.js`, `./conversation-delivery-store.kernel.js`, `./goals-operations.js`, `./goals-operation-policy.js`, `../../session-cards/progress-card-store.js`, `./session-accessor.sqlite-exact-read.js`, `./session-memory-entry.worker.js`, `./session-transcript-read-fence.js`, `../../gateway/session-activity-summary-source.worker.js`, `./session-transcript-hydration-read.worker.js`, and `../../../packages/memory-host-sdk/src/host/session-reset-recall-read.js`.

The exact protocol owner `src/config/sessions/session-transcript-worker.types.ts@0fdb634747ccd9325bb4f33d7af4d4bb81c1b26f` was read. It composes `SessionTranscriptWorkerInput` from SQLite-target, history, model-context, context-messages, single-entry, multi-entry and reset-recall inputs, then attaches dispatch-owned optional physical validation and projection receipts. Reply values are keyed by request `kind`, so malformed kind/result pairing is not an accepted successful reply. `src/config/sessions/session-history-read.types.ts@cc8e4aeadb662e9071fb0da836738be827bf9695` confirms `context-messages` carries the resolved runtime target, optional current-turn transcript admission receipt and optional expected physical database identity.

### 11.2 Dynamic cleanup, retirement and entry-current admission modules

New exact identities and semantics:

| Source | Blob | Read-only finding |
|---|---|---|
| `src/config/sessions/cleanup-service-read.worker.ts` | `cdc444dd4cedf30bee92f1e6a19c7edfd64261e8` | Reads cleanup state only through the captured physical database owner and throws if cleanup loses that owner. |
| `src/config/sessions/session-accessor.sqlite-lifecycle-artifacts.ts` | `e13ecf4c541207cbc80c265a4d12b4c7b340b7f1` | Plans lifecycle artifact cleanup under a SQLite read snapshot; planning does not itself perform file writes or row deletion, and uncertain sources remain with writable repair/integrity owners. |
| `src/config/sessions/session-retirement-read.worker.ts` | `73f68d87c4d7deaac62c86457024ac9f2b080c83` | Worker-side retirement read is a read projection over the captured database, not a second retirement writer. |
| `src/config/sessions/session-entry-current-admission.worker.ts` | `2f1bb9520e0a3df029580170e1fa09ff98428a42` | Current-entry reads use canonical session admission. Writer receipts invalidate cached facts; multi-entry admission chains each physical request and does not let one cached result authorize unrelated entries. |
| `src/config/sessions/session-history-read-operation.worker.ts` | `9f9f7609e09f69e223f3bc324541641407f0f37c` | History operations execute beneath transcript read fences and snapshot-aware dispatch; board/history branches preserve the same admission boundary. |
| `src/state/openclaw-agent-db-validation-cache.ts` | `2d760bc4a78dffa9c597025e101e08da6ce2ed35` | Validation proof is bound to live physical admission; replacement retires old proof, and reader cleanup releases local metadata without revoking independent shared proof. |
| `src/infra/sqlite-transaction.ts` | `eac4144f319f561bc786c99d2dd01223dc3b1337` | Transaction failure clears Kysely cache before native close; rollback/native-close failure remains recorded rather than silently clearing failure state. |
| `src/infra/kysely-sync-cache-state.ts` | `29f87af9b2d2837db28cb2e1f26c6e8428de3108` | Native close/replace disposes dependent callbacks and statement cache before deleting the per-database Kysely cache, preventing stale statements from retaining a closed database. |

Together with the previously pinned worker/resource and snapshot owners, this closes the observed order as: resource acquisition/captured physical admission → fenced dispatch → abort/timeout-aware task custody → lane rotation/resource close → database lifecycle retirement → Kysely/statement dependent disposal → native SQLite close → private snapshot cleanup. Failed native close or failed snapshot cleanup remains in explicit retained custody for retry rather than becoming a successful release.

### 11.3 Four requested worker APIs: production consumers and tests

All paths below were re-fetched at the frozen commit; code-search discovery was not treated as frozen evidence until the exact path was fetched at `ad8d12f126d29b0db204187adf5092db80294e70`.

1. `readSessionTranscriptModelContextInWorker`
   - Production: `src/config/sessions/session-transcript-context-read.ts@73a3440268f760a4d491ce753ef3ee00184220a2`.
   - Test/support consumers: `session-manager-fixtures.test-support.ts@500ac75c749c120e38e9f2c14c67f13feefadd8b`, `session-transcript-read-worker-runtime.test.ts@9c3d1e8352c5db65241d87d533efcd59072b6ee5`, `sdk.auth-migration.test.ts@96a44fc1ae2a9b2b6c71f3591ba52bb50130719a`, `session-manual-compact.worker.test.ts@2140f9316ca7c7bc31fb26e3fd8195644e2300a4`, `session-transcript-context-read.worker.test.ts@b761d295820ba7c1c3d6faf4928bf3385b35ca4c`, `worker-turn-detached-context.test.ts@ca58648bddf4fee335f2fb172d186941f093d3ae`, `session-manager-model-context-snapshot.test.ts@b2ef133f85fd3240f5b3be29384412a4fe1af643`, and `bun-sqlite-library.worker.test.ts@777bbf05672d062acbe65d0de8fca90261cfe9d4`.
   - Negative oracle: a read scanned before owner close must still fail disclosure if ownership/admission is revoked before acceptance; detached async context, auth migration and native-close placement must not revive a stale snapshot.

2. `readSessionTranscriptContextMessagesInWorker`
   - Production: `src/agents/sessions/session-manager-incognito.ts@6e1f3abc2959c090035970014f5fbd7216fbb0b9`.
   - Focused test: `session-transcript-context-read.worker.test.ts@b761d295820ba7c1c3d6faf4928bf3385b35ca4c`.
   - Negative oracle: current-turn admission mismatch or physical identity change rejects the read; post-scan close is not accepted as valid detached context.

3. `prepareSessionEntriesInWorker`
   - Production: `packages/memory-host-sdk/src/host/session-files.ts@295fd643799c6ae1c0b90d9be7a17ae26df4361b`; the compatibility binder is `packages/memory-host-sdk/src/host/openclaw-runtime-session.ts@8860e441f3d62f539cf50b9ae4723f662d83fa51`.
   - Focused test: `src/config/sessions/session-memory-working-set.test.ts@2e66effe75408f73431e94ab0827f15e22b533b5`.
   - Negative oracle: bulk preparation preserves input/result cardinality and cannot publish a partially prepared working set after worker failure/cancellation; the production caller keeps snapshots closed before observer work.

4. `resolveSessionSqliteTargetInWorker`
   - Production: dynamic caller `src/config/sessions/session-sqlite-target.ts@0c9153419035016e260523479eab420b67e0e93e`.
   - Focused tests: `session-transcript-events.worker.test.ts@1e47bfb2d8d18deddf8caf650e9cbf8239ba4645`, `session-accessor.sqlite-shared-write-order.test.ts@38c06212a51ec080eebc6a04e6a62bf17075ca97`, and `session-transcript-anchor-read.worker.test.ts@c80fdf9d05f67a303a65a9b80da908a3ee38be63`.
   - Negative oracle: target change during repair/write admission rejects pending work; alias close and replacement cannot be silently converted into a successful target selection.

No additional frozen production consumer for these four exact exported symbols was found beyond the listed production paths and the memory-host compatibility re-export/binder.

### 11.4 Build and package reachability

The frozen build/package chain is now pinned further:

- `scripts/lib/runtime-process-core-build-entries.mts@9d1b728dc156a80a1fcf3e22e0a69db03683621d` includes `runtimeProcessEntrypoints.sessionTranscript`; its standalone output enables code splitting for the transcript worker under `config/sessions/transcript-[name]-[hash].mjs`.
- `scripts/lib/runtime-process-build-entries.mts@f6e8fa68718647a63f9edc36703ec5dd63f8ce77` composes core runtime-process entrypoints into the package build entry map.
- `tsdown.config.ts@81b8af9ce7170a958c4028fafb6b5099941eedb0` injects runtime-process build entries into the root `dist` build and separately applies standalone runtime-process output options.
- `test/scripts/runtime-process-package.test.ts@fbc2150e022759142b28517dbccf95a729fa836a` derives required `dist/<runtime entry>` paths from the same core entrypoint list and checks npm-pack inventory behavior.
- `test/scripts/runtime-process-build-graph.test.ts@faa8c29daa855f26cb8060965b39e7ae606b43d4` checks the standalone runtime-process graph against forbidden dependency classes.

This is source-level packaging evidence only. These tests were not executed in this round, so package emission/distribution remains unverified evidence rather than an ACCEPT condition.

### 11.5 Lifecycle custody and remaining acceptance gap

The worker-resource owner `session-transcript-worker-resources.ts@ffd341bd00fad8380d81181124bac3ab6aa5d454` uses abort-aware preparation, 60-second task timeouts, lane pending counts, idle retirement, per-database close resources and failed-retirement custody. `worker-task-pool.ts@5a8565c4678f35b029c24921a62e14c7f026755b` exposes explicit `retryFailedRetirements`, `rotate`, `startCloseResources` and pool close operations. Kysely cache disposal is tied to database close/replace, while snapshot cleanup separately preserves cleanup errors.

The recursive source path is materially narrower, but scoped G01/G02 positive evidence, unit authority readiness, executable package proof and independent ACCEPT are still absent. Therefore this audit does not authorize production changes, Actions execution, ledger/RTM/flag upgrades, or an implementation-complete claim.


## 12. Round80 additive closure: transcript cleanup, retirement admission, failed-retirement retry, database close custody, and snapshot retirement

Audit mode remained source-only against the frozen upstream commit `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`. No Actions, build, test, generator, package, production-code, ledger, RTM, or flag mutation was performed. Search results were used only for discovery; every identity asserted below was re-fetched at the frozen commit. This matters because one default-branch search hit, `src/infra/sqlite-snapshot-reclamation.ts`, does not exist at the frozen commit and therefore is not counted as frozen evidence.

### 12.1 Cleanup classification kernel and read contract

New frozen source identities:

- `src/config/sessions/cleanup-service-read.kernel.ts@1541d9e53674708ecce430ff45620cfa976c715a`
- `src/config/sessions/cleanup-service-read.types.ts@487662056d8f07d77b3ba0df61fd29479b251981`
- production wrapper `src/config/sessions/cleanup-service-read.ts@9bc97666e28c267b828f8ac7434fbcf97c0c7e8d`
- production consumer `src/config/sessions/cleanup-service.ts@4277ffdb6064f54d13424fdbea79a4a479052be3`
- already-pinned dynamic worker `src/config/sessions/cleanup-service-read.worker.ts@cdc444dd4cedf30bee92f1e6a19c7edfd64261e8`

Source identity -> production path: `readSessionCleanupSnapshotInDatabase` lists session entries and, when `fixMissing` is enabled, classifies positive missing-transcript removals inside the same admitted read snapshot. When the connection is not already in a transaction it uses `runSqliteReadSnapshotSync`; canonical continuation is preserved through `readWithCanonicalSessionReaderContinuation`. The worker opens the captured agent database read-only, captures its read source, refuses loss of an expected physical file owner, and includes deferred-plugin retained artifact paths. `withSessionCleanupStore` captures candidate identities before asynchronous work, binds an initially absent path to its first admitted file, reasserts identity before and after the worker read, and has a separate in-memory incognito owner path. `runSessionsCleanup` consumes this captured snapshot for preview and apply rather than allowing the read worker to become a mutation owner.

Negative oracle: malformed/cold/unavailable transcript rows cannot prove message absence; the kernel catches that read failure and preserves the session. A session is only classified missing after message absence is positively established. The exact frozen tests read for this path are `cleanup-service.fix-missing.test.ts@b1fa957072538826b38b9eb04f8fc2af6b8919dc`, `cleanup-service.readonly.test.ts@678c6364f7f679282392b3f340f59ec9a789819a`, and `cleanup-service.applied-summary.test.ts@103640ee22b942b5c411bd8c5e09de983d029c2a`; additional direct test/support identities pinned are `commands/sessions-cleanup.test.ts@95ca8ae2e4b4a1f691d9f68eaedf2973ad52dd08` and `cleanup-service.test-support.ts@bc46a41f6d780c8f3817a32083a4cd308d7caa28`. The focused assertions establish: malformed rows preserve state; a foreign message appended after classification survives; a rewritten generation is reclassified; physical replacement after classification is refused without overwriting replacement bytes; dry-run preserves database bytes and registry metadata; revoked cleanup authority fails before lifecycle commit; archive collision never overwrites different bytes; lifecycle-delete failure rolls back the canonical archive; and earlier committed store summaries remain distinguished from a later store's pre/post-commit failure.

### 12.2 Retirement-read types and physical-custody production path

New type identity `src/config/sessions/session-retirement-read.types.ts@2cf7d8835780f9e00fbca5f4a2ba9b2005b15bc2` defines only `keys` and `comparison-claims` requests. Its frozen production implementation remains `session-retirement-read.worker.ts@73f68d87c4d7deaac62c86457024ac9f2b080c83`; the production capture/owner is `session-retirement-read.ts@84c7b777272e0c2c26fde4c21c3074ebb87c69e5`.

Source identity -> production path: `captureSessionRetirementReader` captures both the physical store identity and, when distinct, owner-store identity before either scan phase can yield. Every worker read is surrounded by identity assertions. `withSessionRetirementReaders` holds the existing writer FIFO with `runOpenClawAgentWriteAdmissions` and verifies a native mutation witness after the asynchronous scan. The read worker asserts the expected agent-database identity before querying Kysely and returns an empty typed result only for a genuinely absent read-only database.

Negative oracle: path replacement or native mutation during the scan cannot be accepted as the same source; detect-mode retirement discovery must not materialize transcripts or write. `legacy-main-session-migration.test.ts@f6a8357061679a87305728f75d2b10e491493072` proves detect mode issues only targeted `comparison-claims` for the legacy/canonical pair and performs no transcript reads, while doctor-fix uses the streaming generation reader rather than eager materialization.

### 12.3 Session lifecycle transport contract: declaration-only frontier

`src/config/sessions/session-accessor.sqlite-lifecycle-types.ts@fe1a3b19d1b385a18ddafbba2bde4986b9f2dc4e` was read in full. It is a transport/planning contract, not a live authority owner. It deliberately strips live commit guards and environment/native identity from planning-only shapes, and separates entry/history deletion planning, lifecycle projection plan/commit, archive publish prepare/record, maintenance page/statistics/age/plan/finalize, lifecycle-artifact cleanup, history eviction, and historical-generation reclamation.

The frozen direct-reference inventory pinned this round covers the production chain through `session-lifecycle-projection.worker.ts@fc3802154834a866848e69ed78543bf342c18c41`, `session-accessor.sqlite-reclamation-worker.ts@282e66791782b28e143c65398380f8ea9ef774c6`, `session-accessor.sqlite-reclamation.ts@8be0a591987c450bc8fc09821ef39f0e84b2beb4`, `session-accessor.sqlite-deletion-plan.ts@333769cdc0d4da037c32da3df56ae55f214a1048`, `session-accessor.sqlite-reclamation-publication.ts@a940e269d3f11d9778ec57e7ee39d00ede1d03b2`, `session-accessor.sqlite-projection.ts@b062842cf60a679400157366a0cebf43a2333dc1`, `session-accessor.sqlite-lifecycle.ts@24da1958847b8939408046358db6213aeeb5d147`, `session-accessor.sqlite-maintenance.ts@b38fb99b63c96b4b4e4cfbf0b76b35573877389f`, `session-accessor.sqlite-lifecycle-state.ts@b158417ef0a9202d29bd8340060a50fd07c4335d`, `session-accessor.sqlite-archive.worker.ts@ac720312b8dfaa5f76c5cc6601d8466dece0e045`, `session-accessor.sqlite-reclamation-run.ts@955f799e97c3ced8754eb0eb29f0c4ca7463aa5b`, `session-accessor.sqlite-lifecycle-artifacts.ts@e13ecf4c541207cbc80c265a4d12b4c7b340b7f1`, `session-accessor.sqlite-mutation-worker.runtime.ts@ada267318473e3074f694abe183f9ea1447abe84`, and `session-accessor.sqlite-reclamation-worker-lifetime.ts@47b74b249e5c398c908c84180b097e048b0f9712`. Thirteen frozen test/support direct-reference blobs were also pinned, including reclamation, maintenance-read/admission/cap/worker/reuse, schema-probe, lifecycle-publication, cadence, and preparation support.

Negative oracle: a transport plan cannot grant commit/native authority by itself; historical-generation scope cannot authorize whole-entry reclamation; a result discriminant must match its plan kind; maintenance finalization refers to captured plan positions rather than echoing stale entry snapshots; lifecycle-artifact planning remains read-only until the real writer commits.

### 12.4 SQLite worker operation admission

`src/infra/sqlite-worker-operation-admission.ts@90f03a25678f00a88489c13392e19dbb09df6a40` was read in full.

Source identity -> production path: the host owns a private MessageChannel plus shared atomic decision words `REQUESTED/GRANTED/REFUSED/TIMED_OUT`. Failure provenance remains separated as authority/domain/protocol, with physical/protocol loss able to supersede a handled domain refusal. Database-admission relay validates creation facts and deadlines, retains a special bounded cold-open-pending state, and never treats shared format facts as database authority. Native commit receipts are sequenced/idempotent; settlement distinguishes completed from outcome-unknown. `grant` rechecks deadline, closure, and database authority immediately before atomic release. Schema maintenance is constrained to the admitted database and holds a releasable schema lease. `finish()` shuts future authority, services queued receipts, refuses outstanding requests, closes ports, and records lease-release failures separately. Commit receipts require an active retained admission and are deferred to the real database COMMIT edge.

Frozen negative tests read in full:
- `sqlite-worker-operation-admission.timeout.test.ts@7511cb8ea7ab283094fecb5fc6ddcbcc7500738f`
- `sqlite-worker-operation-attachment.test.ts@259eede9d9ca4f00b6365842ce47e6d486d9058c`
- `sqlite-worker-lifecycle-cleanup.test.ts@99877a3f298b483b75a4756a14430c93fcede43e`

They prove revoked broker custody rolls back after a host grant but before native use; explicit deadlines reject delayed transaction/commit admission and preserve only earlier commits; policy preparation that crosses the deadline cannot run its release continuation; thread/fake-clock skew does not create a false timeout; an attachment cannot escape the retained operation scope; and post-grant cleanup failure cannot rewrite an already committed result or drop its queued follower.

Discovery returned the connector's 100-result cap for the `createSqliteWorkerOperationAdmission` symbol, so this round does not falsely claim repository-global frozen caller closure. Those search hits remain a frontier for exact frozen per-file re-fetch rather than evidence by themselves.

### 12.5 Failed worker retirement is explicit retry custody, not implicit restart

Frozen runtime identities read in full:
- `src/infra/worker-task-pool.ts@5a8565c4678f35b029c24921a62e14c7f026755b`
- `packages/worker-runtime/src/worker-task-pool-core.ts@f73ca6fbfb5cf54a9c69143cce14d02fae8c39f9`
- `packages/worker-runtime/src/worker-task-pool-retirement.ts@8f8b1804b1b9538793953231bc1cfb76b1e1fcef`
- transcript production composition `session-transcript-read-pools.ts@b385c34a0a015ee98fc81c541a7c22d05daa4a5a`
- transcript database-resource owner `session-transcript-worker-resources.ts@ffd341bd00fad8380d81181124bac3ab6aa5d454`

Source identity -> production path: `startRetire` first cancels native sections, waits for creation/native sections, treats diagnostic flush as non-authoritative telemetry, joins native stop, then releases task resources and retained artifact cleanup. A native stop failure sets `slot.retirementFailed`, clears only the in-progress retirement handle, and retains native custody plus queued input charges; dispatch excludes that slot. `retryFailedRetirements` retries every failed slot and joins artifact cleanups, collecting all failures. Rotation joins current tasks, every retirement, and artifacts; rotation failure blocks dispatch until a later explicit retry path. Pool close joins owned and unowned retirements and preparation/artifact cleanup.

In the transcript production owner, an idle exact-database close uses `closeResources` only when native-close qualification is positive; on failure the lane is re-added to the database's native custody. Revoked `HistoryDatabaseResource` refuses new admission while its close is unsettled. Per-database retirement/whole-runtime rotation therefore provides the production retry path; the public `retryFailedRetirements` method is not substituted as a second transcript owner.

`worker-task-pool.preparation.test.ts@bd3b8efb8484ead172400901ad70da07da22db4a` was read through its failed-retirement cases. Negative oracle: every retry and its artifacts settle before rejection; multiple retry failures aggregate; a healthy sibling is neither terminated nor waited on while a failed sibling retries; input/response/execution-settlement custody remains held after task+retirement failure until a later explicit `close` or `rotate` joins native exit; the final settlement callback runs in the admitted context exactly once.

### 12.6 Agent database native close and lease custody

`src/state/openclaw-agent-db-lifecycle.ts@581228a0469b553b9ae23beb494374c92c8ec68a` and `src/state/openclaw-agent-db-resources.ts@b99f092118b969d8208db4aa137ad243e1bc3688` were read in full. Production bridge `src/state/openclaw-agent-db.ts@9ef32a1137b92d95af00d1a92086c5e6461fab69` and reclamation worker `session-accessor.sqlite-mutation-worker.runtime.ts@ada267318473e3074f694abe183f9ea1447abe84` were read for the settlement path.

Source identity -> production path: the lifecycle owner retains physical handles, borrowers, idle timers, durable leases, pending opens, and failed-close records. `retainFailedAgentDatabaseClose` keeps the exact path/agent close callback until a later successful retry and registers it with the maintenance/exit owner. Idle close failure refreshes its timer rather than deleting the DB/lease. `settleOpenClawAgentDatabaseWorkerClose` separately converges native handle close and durable lease release, returns all cleanup errors plus `settled`, and does not turn an already committed worker result into a false rollback. Agent resource drainage keeps failed resources in `resources.closing`, fences matching new admissions, closes ordinary dependent resources before `retireAfterResources` native owners, then closes the database handle. The reclamation worker clears its Kysely cache at each request boundary and calls this settlement owner on normal close and failure cleanup; an incomplete cleanup produces explicit warnings/`cleanupIncomplete` or an AggregateError on the failure path.

Focused frozen tests include `session-accessor.sqlite-mutation-worker.runtime.test.ts@37da85166d782fe71c3268fa3acf43ff76962bab` and `openclaw-agent-db.lease-owner.test.ts@76f01089df6f2e19df5a4352c70f2c927f2d8ab5`. Negative oracle: original lease state must be released even after ambient environment mutation; a failed-open native close retains its original store owner for retry; lease-release failure leaves the closed handle's durable lease retryable and a later lifecycle pass clears it rather than retaining a second writer forever.

### 12.7 Shared-state async lifecycle distinguishes failed drainage from failed native retirement

`src/state/openclaw-state-db-async-lifecycle.ts@fefa1e8c9dffbac0967263c91dbe74ede6d29fb9` was read in full. Frozen constructor/cache/test identities pinned are `openclaw-state-db-cache.types.ts@a46066eb3af08e80cd9e62005dcb304e764725e1`, `openclaw-state-db-cache.ts@adcc28173d50c16683e776dcf6bbd056022dd35a`, `openclaw-state-db-pending-admission.test.ts@fbe640859da7b961f5a85bc58fcdb59dcd6d42d6`, `openclaw-state-db-borrow.test.ts@60ca10d5168c75467f21ddab5324b1a289694abc`, `openclaw-state-db-read-admission.test.ts@67e35c387465f743492e6ff1002dbf197b7ba271`, `openclaw-state-db-async-lifecycle.test.ts@3dbf4c77279d7759d2cc3ae9b8d8773e1fd8550a`, and `openclaw-state-db-schema-policy.test.ts@05fc29d2ff7eef75aeb0cfaa9404be5cd87fd5c5`.

Source identity -> production path: maintenance scope drains resources in ordered phases and waits tracked work both before and after each batch. A close attempt seals read admissions, queues current and newly registered resources, retains failed resource owners, and serializes native retirement. Failed resource drainage releases the read seal when the original database can safely continue, but retains resource custody and seals again on each retry. Failed native retirement keeps the seal because the old physical owner remains unretired. Replacement publication during an unsettled path close marks the old record superseded and cannot bind readers to the replacement until old custody retires.

The full `openclaw-state-db-async-lifecycle.test.ts` supplies the negative oracle: an unregistered multipath resource remains retained after failure while unrelated paths can close; an `after-resources` finalizer skipped after ordinary failure is retained for retry; reads reopen after failed drainage but are sealed again for every retry; replacement during or after failed drainage remains inaccessible until the original identity retires; concurrent pending closes coalesce; native close failure retains its seal until retry succeeds; and closing a never-admitted missing/directory target cannot accidentally global-drain an unrelated database.

### 12.8 Snapshot retirement/source keeps file deletion behind nested native lifetime

Frozen source identities:
- `src/infra/sqlite-snapshot-retirement.ts@38ed99613362265377ae6b11a31d1a1ce02f19e2`
- `src/infra/sqlite-snapshot-source.ts@14931e709dbccb2daab535c15201dfc671994904`

Frozen direct retirement-side identities re-fetched include `sqlite-snapshot-staging.ts@45b01f38a5cdd82b6dd2b8dce5872f2c92025685`, `sqlite-readonly-location.worker.ts@c1bcd2687aa4c3d9ce7dbd9310fdc2ca3e90b6c5`, `sqlite-readonly-location-cleanup.ts@851b9cb4ed4f6880712bc48fbdf1423087a97e8c`, `sqlite-snapshot-retirement.test.ts@5597dbd303676e99d66ff29f5b9ac70392329365`, `sqlite-readonly-location.worker.session.test.ts@9bef24be732c0b5d8c3b33c6f30e18ed577fdc93`, and CI inventory `scripts/lib/ci-proof-test-inventory.mts@7721eb00b51b48412b41b6e0e3dfe58c23c01198`. The default-branch-only hit `sqlite-snapshot-reclamation.ts` was explicitly rejected after frozen fetch returned NOT_FOUND.

Source identity -> production path: retirement acquires every nested staging lifetime before selecting payload, validates ownership/layout/activity, keeps the creator token separately retryable, and publishes token retirement before control-file/directory removal. Snapshot source keeps native reader close, token release, and temporary-directory cleanup in one retained owner. Awaited worker snapshots keep a retained retirement operation; a rejected retirement can be replaced by a later retry. Preparation failure plus cleanup failure is preserved as an AggregateError rather than reported as successful cancellation. Focused frozen production consumers pinned include `openclaw-state-read-operation.ts@2f8740a2800aa5190b00621da5b7f74bdd17e4e2`, `sqlite-snapshot.ts@535adc4ecc41b46357daa83d2b7a004d4c50d5c7`, `openclaw-state-db-schema-discovery.ts@ada041a24b45ec995f11d2a58a3aec5119a84e9a`, `deferred-plugin-session-transcript-read.worker.ts@09bcfb86aec221600bd738ef1b8d4b563188612d`, `openclaw-agent-db-readonly-open.ts@1796b0d3d312ddbbda2fc967b1e8454544e92da9`, `session-sqlite-migration-readers.ts@8b884c2ff641d561aec01dda547e7ecc5a84b708`, and `openclaw-state-read-snapshot-source.ts@4d97a8303f5e467e9fb9a09178b0e3e1253940ee`.

Negative oracle: `sqlite-snapshot-retirement.test.ts` proves parent cleanup refuses while a live child exists, unowned/identity-drift retirement preserves readable payload, and owned legacy staging alone is reclaimable. `sqlite-snapshot-source.identity.test.ts@b803b3b760955344ad18eb17363b93c42cf2c596` proves a stale expected identity cannot copy successor bytes while a concurrent current identity succeeds, and all retained preparation closes/cleanup operations must join without staging leftovers. `sqlite-snapshot-source.rehearsal.test.ts@cb3d6118fad7c7c5ae4e775d4b1d8935feafa1bc` proves artifact-preserving rehearsal reads stay read-only and a shared-state write is refused.

### 12.9 Updated lifecycle frontier

The source-only chain is now more specific:

`captured physical/read identity -> admitted worker request -> deadline/custody recheck -> typed read/classification -> task/native-section settlement -> failed-retirement retained slot custody -> exact database resource drainage -> Kysely/statement cache disposal -> native SQLite handle close -> durable lease release -> nested snapshot token retirement -> temporary snapshot removal`.

The important negative distinction is now source-grounded at each layer: task success is not native retirement; native close is not durable lease release; resource drainage failure is not native retirement failure; snapshot token retirement is not directory removal. Each has its own retry custody and fail-closed oracle.

This remains source evidence only. The connector's default-branch code-search cap means repository-global exact frozen caller closure is not yet proven for broad modules such as `sqlite-worker-operation-admission`, `openclaw-agent-db-lifecycle`, `openclaw-state-db-async-lifecycle`, and `sqlite-snapshot-source`. Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, and release acceptance therefore remain open. No status/ledger/RTM/flag upgrade is justified.


## 13. Round81 — exact-ref production-entry clustering beyond the 100-result cap

### 13.1 Frozen-source method and exclusions

This round kept the upstream source frozen at `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` and treated GitHub default-branch code search only as a discovery index. Every identity below was re-fetched by path at the frozen commit before it was admitted as evidence. Two discovery hits demonstrate why this distinction matters: `src/gateway/update-automatic-cleanup.ts` and `src/commands/doctor-a2ui-widgets.ts` both exist in the moving default-branch search surface but return NOT_FOUND at the frozen commit, so neither is counted as frozen evidence.

The pre-write audit-branch lease was `3bdc199de8645bedbdf9cd1162413cb7dd06b058`. No Actions, build, test, generator, package, production-code, ledger, RTM, or flag mutation was performed in this round.

### 13.2 SQLite worker operation admission is a caller-owned capability, not generic write authority

New exact frozen production caller identities were clustered by the product entry that supplies the live authority:

- Session/transcript: `src/config/sessions/session-worker-execution-source.ts@f5c07ec10872ec9335660c0905f22286852810da`, `src/config/sessions/session-accessor.sqlite-worker-coordination.ts@b4e805240b51228188ac7e8ff76b02c6a393a042`, `src/config/sessions/session-accessor.sqlite-replacement-worker.ts@8a13a2a9ffea31c161c88d22d394840153d7e2b0`, `src/sessions/session-upstream-links-runtime.ts@5995346380a9bb408e8a6af53d02c917f770b2af`, and `src/sessions/session-state-events.operation.ts@33a66e8dc73f67bf6352b498f3f04233e056607b`.
- Agent and deletion: `src/state/openclaw-agent-db-worker-admission.ts@0780b01abe79e73d6510ef153cd7d1ed51e73512`, `src/state/openclaw-agent-execution-preparation.ts@16dfe6a62472eb123db20c3cf88cda99a0432f0f`, `src/state/openclaw-agent-execution-cleanup.ts@5553a2e68f51cc6ed1a0f4441ffc50f46160618f`, and `src/agents/agent-deletion-claim.ts@8034afc9b708003e039ab640d5b8c7cef4cea921`.
- Shared-state/domain mutation: `src/claws/state-mutation-write.ts@54e8cc2bde81b3301805d36f43ff96dccb2dfd14`, `src/state/user-model-account-operations.ts@b7a96637e08f49306374ddf9599ed363eb549a16`, `src/state/user-channel-identity-operations.ts@64c1d7790400298871ace4df953ea6fcbbf8247c`, and `src/agents/workspace-state-store.ts@756994343f97f2ef13ed021fe2ad188ce2007a4b`.
- Gateway/device/publication: `src/pairing/pairing-store.ts@f36eb9034f1119721093fe142c10265d383e6dfd`, `src/infra/device-pairing-worker.ts@190f6301ff4a865992d8bd1c5f3ddf4aff912ca5`, `src/infra/push-apns-store.ts@810725ff9451fdab7530edce28c640dc2f267083`, `src/infra/push-web-store.ts@9a26c0bc84c51e04ae31d60b442b4a40b3e01789`, `src/acp/runtime/session-meta-worker-mutation.ts@90adaa80c0e149585748f23bf179e661428eb542`, and `src/gateway/github-publication-source.ts@32b16c77cbe306da84fa1aa3d47a4266a7b1ab54`.
- Shared infrastructure: `src/infra/sqlite-worker-store.ts@9eb422ce572d955d743c94ffce2eed31dcfc4f9b`, `src/infra/sqlite-worker-broker-admission.ts@45133632d6177148977278743c345cee9e5482cf`, `src/infra/sqlite-worker-broker-reply.ts@d53989505d7e36c8075009ac22c0af6e8a03d9a7`, and `src/state/openclaw-state-lease-worker-owner.ts@47668f72bf2b04d69416a530f61cafe590e2fa5f`.

The source-grounded production path is now broader than the core module alone:

`product mutation / session owner / agent owner -> capture physical state and caller authority -> create caller-specific SqliteWorkerOperationAdmission -> native worker requests stage/facts -> revalidate signal + caller/current database + generation/identity -> grant or fail closed -> retain settlement/publication custody`.

The callers do not all authorize the same thing. The canonical write factory in `sqlite-worker-store.ts` enforces ordered transaction/commit phases. ACP session metadata additionally binds a nonce and exact phase, rejecting a callback that differs from its retained transaction. GitHub memory publication permits only a preparation grant. Pairing validates the captured channel and approval facts. Agent/session execution sources first call `binding.authorize(request)`, then recheck the captured execution before granting. Most importantly, `session-accessor.sqlite-worker-coordination.ts` deliberately installs an operation admission that throws `SQLite mutation file admission does not grant transaction authority` outside its reconciliation relay: possession of the file/lifecycle admission is explicitly not mutation authority.

### 13.3 Agent-database lifecycle callers keep close/disposal behind exact physical ownership

New exact frozen production callers of the agent lifecycle include:

- `src/state/openclaw-agent-db-disposal.ts@d4b1cad767943f7ed35d892838973efb7c7288bc`
- `src/agents/agent-delete-databases.ts@9e5551bb6fe09235e3675437132da161393d7a04`
- `src/commands/doctor-session-worktree-workspace.ts@7c4096b2aeec2c3d3e17e463f280419ca0f6ef91`
- `src/commands/doctor-session-sqlite-diagnostics.ts@ff84d6133418519d40a4940a611011febc5416a6`
- `src/commands/doctor-maintenance-state.ts@e453f1f458d897997b14f8ea2dc1b78a7a12160c`
- `src/config/sessions/startup-migration.ts@5dce338ea47b067f6f0447180c71a521b3143660`
- `src/infra/state-migrations.shared-auth-store.ts@c4567d8029e23ff63c2d90267e2e2cec8f1edb5d`
- `src/infra/update-database-restore.ts@ee222da077c9361a5eca4cacd36ed51ef42b0dcf`

The production path is:

`delete / Doctor / startup migration / rollback -> freeze exact registry and path identity -> drain session/model/read-pool or other registered resources -> closeOpenClawAgentDatabaseByPathAsync -> native handle retirement -> registry/durable-lease publication -> only then replacement/removal`.

`openclaw-agent-db-disposal.ts` rejects multiple native owners, rechecks maintenance plus every captured path identity, waits pending asynchronous owners, and aggregates close failures before registry publication. Update rollback separately requires exclusive shared-state ownership and holds removal exclusions while the complete database family is restored. Startup/Doctor helpers preserve pre-existing open owners: they close a database only when their operation opened it.

### 13.4 Shared-state async lifecycle is the canonical resource-drain owner for production readers and workers

The direct constructor remains centralized in `src/state/openclaw-state-db-cache.ts@adcc28173d50c16683e776dcf6bbd056022dd35a`: it exposes `register`, `capture`, `publish`, `holdExclusion`, and `close` through one `createOpenClawStateDatabaseAsyncLifecycle()` instance. New exact frozen production consumers of those wrappers include:

- `src/state/openclaw-state-read-scope.ts@4059cd39015c788ea8ddbca154d14f58718688b5`
- `src/cron/service/scheduler-actor.ts@8d896adbe9bc48f6893b3fe4d1f3da9432e85111`
- `src/secrets/store/secret-store.ts@b2f7e23251568e309d06dafbf3877fa33a2a1545`
- `src/config/sessions/session-accessor.sqlite-commit-context.ts@4363ab314c71fb0624c0f9a479f9eb465d48b21b`
- `src/config/sessions/session-accessor.sqlite-archive-session.ts@5e6940270f525cad2859b93cb6f0caca8b94afa3`
- `src/gateway/mention-inbox.ts@b52705d073f05dcfc6c110f0ba25a1de63b848fb`
- `src/state/openclaw-state-worker-owner.ts@3524b5a1d28ec1421185cd2e3eabd80a00217fc6`
- `src/agents/sessions/session-manager-write-admission.ts@29274ecb1af4dacdd3b2a077e73ad21ed153a001`
- `src/commands/cleanup-utils.ts@71b2404b05f737603953cdcb38d5b23930275bab`
- `src/cli/update-cli/update-command-database-backup.ts@cb3ef46eb0da12997bf344e810518e83a2f4721d`

The shared lifecycle path is:

`read/worker/scheduler/archive owner -> capture exact state admission -> register retained resource -> track accepted async work -> path/global close seals new reads -> drain matching registered resources -> aggregate failures and retain failed custody -> retire native handles -> release or keep seal according to failure class`.

A retained read scope drains its tracked work and child resources before snapshot cleanup. Session commit/archive contexts register against both shared-state and agent-database lifecycles, so either owner can revoke future work while the accepted close is joined. Mention Inbox records unknown mutation outcomes as requiring synchronization/readback and explicitly does not replay the mutation to manufacture a receipt. Cleanup and update backup acquire the removal exclusion before destructive filesystem activity and revalidate authority immediately before dispatch.

### 13.5 Snapshot-source callers preserve source bytes, identity, and cleanup custody across Doctor/update/plugin flows

New exact frozen production consumers include:

- `src/plugin-sdk/sqlite-runtime.ts@6277ffff34afc3e3c85a0cca8c7bedf8369bd6e5`, which exposes the supported plugin snapshot boundary.
- `src/infra/gateway-lock-recovery-lease.ts@ae795de771d0cbbae4e790219b4246d90cb29767`.
- `src/infra/sqlite-maintenance-inspection.ts@982442f494c174774893b9d2dcfbfdc0d2603376`.
- `src/state/openclaw-state-ownership-preflight.ts@b9513d9293e3e3ef5d8c69a61af65334703e3494`.
- `extensions/memory-core/src/doctor-vector-index-provider.ts@05dfed9088eb742ece04a28d7a865e5c11048157`.
- `src/node-host/auto-update-compatibility.ts@11069ad380b737b36684d4be85d3207d9c5e59df`.
- `src/commands/doctor-update-schema-guard.ts@9f21fdb71c44fe1c489e6c2d0d4b4553ad218c75`.
- `src/infra/state-migrations.transcript-directives-archives.ts@9a07116e2a020f3ab318abc7eeff6cbec387b67c`.
- `src/infra/sqlite-index-recovery.ts@cd0a39faf1e253ded17350bd3eb4b520de49bee7`.
- `src/commands/doctor-db-bloat.read.ts@1b5e04bfcc3e9bf0a501b0179750d8c48a06b716`.
- `src/infra/database-engine/physical-store.ts@301389b3ce141f844d89e7cf753069114317c97f`.
- `src/state/openclaw-state-ownership.ts@3910a770050e5d207c6eca47ffa3d5a892fd92af`.

The resulting path is:

`Doctor / ownership / update / plugin read -> artifact-preserving or live-owner snapshot -> immutable/private read location -> read-only consumer -> explicit cleanupAsync/cleanup -> any cleanup failure remains part of the operation outcome`.

Gateway recovery treats a false cleanup result as an error and aggregates inspection/cleanup failures. The memory-core Doctor consumer similarly reports incomplete temporary-snapshot cleanup as a failure. Node auto-update compatibility combines a compatibility failure with cleanup failure rather than losing either error, and the preflight uses a caller signal. State ownership preflight refuses canceled or unauthorized admission before destructive staging and only permits deferred cleanup after a read actually succeeded.

### 13.6 New frozen focused-test identities and negative oracles

These tests were read as source/oracle evidence only; they were not executed in this audit round:

- `src/config/sessions/session-accessor.sqlite-worker-coordination.test.ts@cfe8b31c32e3ef5485fb1938f12cb8554898575b`: accepted cleanup remains valid after the close owner revokes new reads; failed dispatch joins native worker exit; a sibling lease is not over-drained.
- `src/config/sessions/session-accessor.sqlite-archive-session.test.ts@b6f9e4bb4517d578dc867e9df632d8a808bbd7e2`: queued/scoped archive work joins owner close, late scope work is refused, failed publication is recovered from durable state, and physical replacement cannot receive the old source publication.
- `src/gateway/server-methods/channel-pairing.test.ts@0251618b7b9f2f5ecfe6159ce79a687911c85d55`: invalid selectors and stale/non-admin authority are rejected, while dismissal does not accidentally approve the sender.
- `src/node-host/auto-update-compatibility.test.ts@bf6086ee1491e16f8ed710af2994418f9243d68a`: committed WAL rows are consolidated for inspection without modifying live database/WAL families; compatibility/install failures leave current state and the selected runtime intact.
- `src/infra/update-database-restore.test.ts@ba94920c55b67b15f771efbbbe148082339af640`: authority is rechecked after awaited verification; competing native readers and foreign maintenance owners block replacement; allocation plus cleanup failures preserve the current family and both error causes.
- `src/state/openclaw-agent-db-disposal.test.ts@3324167141643be37352cedebaeeaa7f5f3d8163`: absent disposal fences a successor open, only the exact cached owner is disposed, file replacement and duplicate native owners are refused, and a worker-created database retires through its registry owner without host data SQL.
- `src/infra/sqlite-worker-shared-state-idle.test.ts@67e4ace0ddb3c080ba0ddd64cf8f34cedfa3d397`: path close retains pressure-retirement custody until whole drainage, healthy co-users survive unrelated idle retirement, and canonical close joins discharged custody without replaying an earlier operation failure.
- `src/agents/subagents/registry/subagent-registry-worker.test.ts@d959453013967dc64c90339d6e177bdc0bc5b9d5`: canceled restore publishes no partial rows; unknown mutation outcome remains fenced through source close until canonical restoration; native admission revalidates live caller authority; old-source commits cannot notify a replacement database.
- `src/state/openclaw-state-ownership.test.ts@b35231b1bccb0b58be49c85a81f4c7bf834e3a27`: cancellation before staging does not create a snapshot; cancellation after adoption still cleans it; unauthorized Doctor staging is refused; WAL/rollback races do not mutate or leak uncommitted ownership.
- `extensions/memory-core/src/doctor-health.test.ts@3e469f0373ded7be0c536f72ffaf51c10dfbfdef`: plugin Doctor inspection preserves active WAL/database bytes while writes continue and returns a structured non-ready result when inspection cannot be completed.

### 13.7 Round81 frontier

The 100-result cap is materially reduced by production-entry clustering, but repository-global frozen caller closure is still not proven. The broad `createSqliteWorkerOperationAdmission` discovery query itself still reaches the search cap, and GitHub code search indexes the moving default branch rather than providing an exhaustive frozen-ref index. Additional discovered production paths remain to be exact-fetched/classified, and the wrapper consumers of shared-state lifecycle and snapshot source are still broader than the directly closed clusters above.

Therefore the source-only verdict remains unchanged: independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, and release acceptance remain absent. No production implementation, ledger, RTM, or completion flag may be upgraded from this round.


## 14. Round81 continuation after live-lease reconciliation

### 14.1 Concurrent lease reconciliation and evidence de-duplication

While this audit was still reading frozen callers, the audit branch advanced from the previously observed `3bdc199de8645bedbdf9cd1162413cb7dd06b058` to `12b8a93b68f20aea9a36f024b3f6f61a20128bb0`. The live dossier was re-read before this write. Its new section 13 already contained a substantial Round81 caller-clustering update, so this continuation does not repeat those identities. Candidate exact-ref identities were checked against the live dossier text and only still-missing frozen evidence is added below.

The upstream remains frozen at `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`. Default-branch code search remains discovery-only. No Actions, build, test, generator, package, production code, ledger, RTM, or flags were modified or run.

### 14.2 Additional frozen SQLite-operation admission callers

The following production callers were re-fetched at the frozen upstream and were absent from the live Round87 dossier:

- `src/state/agent-deletion-recovery.ts@41b0ab2e1efedf957131c3b5b06ee07cd339488e`
- `src/state/user-background.ts@a149796ff29d5db13c0b496a56870c213ba0e1ca`
- `src/media/generated-html-provenance.ts@d00a635b7ee764126236a7f7c4c32969acf3f893`
- `src/agents/worktrees/capacity-store.ts@0c23a3aa416221870df842a1bb1db1c01840f382`
- `src/agents/harness/native-hook-relay-store.ts@665a74dc20289b403a73c66bd24e9a69a915febc`
- `src/state/user-preferences.ts@a20e4e1ebc2d60624993f75918858751f244dc3b`
- `src/state/user-profiles-avatar.ts@26e19acd261d9e34508019aee57082ab6576f2de`
- `src/trajectory/runtime-retention.ts@82c48e14c7b180632c925629c98f0b216bc67d86`
- `src/infra/client-credential-async.ts@a99b4e68613e621fdd58bd11cc5db55d2b69a2ce`
- `src/gateway/mention-inbox-worker.ts@83ecd55c09c400122702641aa3578047d7353bde`

Their admission behavior is not interchangeable:

- Agent-deletion recovery permits only transaction/commit admission and rechecks both the captured state-database admission and the caller's current recovery authority before grant.
- User-background mutation rechecks database identity, abort signal, and caller-currentness before grant; the publication fence is released only after the worker owner has joined accepted native settlement.
- Generated-HTML provenance never replays an unknown mutation outcome: if the ordinary reply is lost, only a matching committed receipt may recover the numeric result; otherwise the original error is preserved.
- Worktree-capacity exact-token release joins native settlement before worker close and aggregates release/close failures rather than declaring the reservation released early.
- Native-hook relay persistence permits transaction admission only and revalidates both state admission and caller authority.
- User-preference mutation distinguishes transaction from commit admission and validates returned authority facts for profile/coauthor publication before accepting the commit.
- Profile-avatar adoption binds the transaction to one canonical profile identity and refuses a commit whose row/profile identity does not match that captured transaction.
- Trajectory retention passes the existing execution binding through `binding.authorize(request)`, rechecks source authority, and fails when grant custody has expired.
- Client-credential operations bind database authority plus schema lease to the same captured access object; maintenance/read admission is rechecked before grant and throughout schema lease use.
- Mention Inbox rechecks state and caller authority before grant. A lost ordinary reply is not replayed: only a matching committed head receipt can be returned as committed.

This expands the source-grounded path beyond section 13 without changing the owner decision: the product/domain caller supplies narrow live authority, while `sqlite-worker-operation-admission` transports and settles that authority; it is not a generic mutation capability.

### 14.3 Additional agent-database close and snapshot/read-source consumers

New frozen production identities absent from the live dossier were also re-read:

- `src/commands/doctor-session-incognito-key-repair.ts@360ddaf3416bb40b2d767eaa316d50103e1848eb`: Doctor closes the target agent database only when that operation did not inherit an already-open owner, preserving pre-existing ownership.
- `src/infra/state-migrations.meeting-transcripts-detection.ts@ca6bb1efed24a2d0a43dfaf70369f547c89b432a`: migration detection uses artifact-preserving shared-state reads when available, otherwise a read-only SQLite handle with explicit close.
- `src/state/openclaw-quarantine-store.ts@b70e97d4fda291c9495cd866a39d7e1224d44ff8`: quarantine inspection closes its read-only handle explicitly; close failure is surfaced as `OpenClawQuarantineReadCleanupError` and cannot silently replace the read outcome.
- `src/agents/auth-profiles/sqlite.ts@b6aa362a3148b38ee480eb1124909aedaf1b5efd`: auth-profile read paths pass the canonical read-only database opener into the profile read/acquisition helpers rather than introducing a second SQLite owner.
- `src/commands/doctor-database-port-migrations.ts@859e7100d282ebfd2eebc21c5e525ac60cb38dfa`: port-migration inspection opens read-only, admits schema facts, and refuses migration while reserved-incognito-key repair remains pending.
- `src/infra/state-migrations.legacy-sessions.ts@0aa3d8643bb9c6afbe1621cac03db35731153f20`: legacy-session inspection opens read-only and rejects missing/invalid agent schema ownership metadata instead of treating an arbitrary database as a migratable agent source.

The shared-state async-lifecycle constructor query was re-run as part of this clustering. Its direct constructor surface did not produce a new frozen identity beyond the already-pinned cache/type/test set, so no duplicate constructor evidence is added here; the remaining open work is wrapper/consumer closure, not creation of another lifecycle owner.

### 14.4 Newly pinned focused negative-oracle tests

These frozen tests were read as source/oracle evidence only and were absent from the live dossier before this append:

- `src/infra/client-credential-lifecycle.test.ts@0b687741b0feefd8f2ebad97b1d1b609fab5d8c9`: an in-place copied receipt from another process is refused even when the admission key is unchanged; schema-version damage and physical file replacement force re-admission rather than serving cached authority.
- `src/agents/agent-delete-safety.test.ts@8d095a1de69cae7298c251cd1938d1dbeb4c4eb2`: deletion planning rejects a session-store replacement that occurs after the safety admission yields.
- `src/config/sessions/startup-migration.concurrency.test.ts@cde81e35aa49020862d09734403114358592e6a6`: startup drains sibling work and the worker pool before cold database handles close; failed idle-pool retirement retains a cleanup owner and retries through shared-state retirement until native exit succeeds.
- `src/infra/state-migrations.shared-auth-store.test.ts@e7410c88b67f257b1fbff4c2340001f38919947e`: pre-existing source resources are joined before returning migration-read failure; source credential authority is revoked before relocation; copied inspection preserves live source locks.
- `src/state/openclaw-quarantine-store.test.ts@29f6775f8027aca86c3666d80ee86f70b9a41382`: read-handle close failure is preserved as cleanup failure, and ABORT/ROLLBACK failure while clearing quarantine preserves the recorded quarantine instead of publishing a false clean state.
- `src/infra/sqlite-snapshot-source.retained.test.ts@8669140a7e15545aa5cdbfb4e68285c61af3004d`: one retained snapshot owner cannot delete the prepared directory while a joined source still holds it; cleanup completes only after both retained owners and operations close.
- `src/state/openclaw-state-db-readonly.worker-lifecycle.test.ts@f2122951ca0c0299e28ba6045a851c00dcb4eed9`: shutdown closes pending snapshot preparation before joining the read; failed published cleanup retains ownership for canonical retry; read error and owner-retirement error remain distinguishable.
- `src/state/openclaw-state-db-readonly.signal-custody.test.ts@0221e580e19b8c0cd9af744669984592c0891a13`: native-close failure retains prepared bytes; direct readers and transport close must settle before removal; cleanup cannot delete through an enclosing retained callback.
- `src/gateway/mention-inbox.target-capacity.test.ts@656f2c2d32e1c79eb76c3f5099c58a5e5997aa80`: query-only target aliases filling the cache do not make retained mentions disappear, preserving the mutation/read projection contract.

Together these oracles reinforce the same negative boundary: caller cancellation, reply loss, stale cached admission, source replacement, read failure, or cleanup failure cannot manufacture success, silently replay mutation, or release physical custody before the corresponding owner settles.

### 14.5 Round81 continuation frontier

This continuation adds non-duplicate exact-ref evidence after reconciling the concurrent live lease, but it still does not prove repository-global frozen caller closure. The broad `createSqliteWorkerOperationAdmission` discovery surface still hits the 100-result cap, and additional default-branch candidates remain to be exact-fetched and classified at the frozen commit. Wrapper consumers of `openclaw-state-db-async-lifecycle` and `sqlite-snapshot-source` also remain broader than the clusters now pinned.

Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, OCRM final acceptance, and release acceptance remain absent. No production implementation, ledger, RTM, or completion flag upgrade is justified.


## 15. Round81 — first capped admission page exact-ref closure

### 15.1 Exact-ref page accounting

The default-branch discovery query for `createSqliteWorkerOperationAdmission` still returns exactly the connector's 100-result cap. This round classified every path returned on that first capped page against the frozen upstream `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` rather than treating search snippets as source evidence.

- Production paths: 68 returned; 67 exist and were exact-fetched/classified at the frozen commit. `src/gateway/update-automatic-cleanup.ts` is a moving-default-only hit and returns NOT_FOUND at the frozen commit.
- Test/support paths: 32 returned; 31 exist at the frozen commit. `src/gateway/update-recovery-cleanup.authority.integration.test.ts` is a moving-default-only hit and returns NOT_FOUND at the frozen commit.
- Four of the 31 frozen test/support identities were already recorded earlier in this dossier and are not duplicated as new evidence: `src/state/openclaw-agent-db-disposal.test.ts@3324167141643be37352cedebaeeaa7f5f3d8163`, `src/infra/sqlite-worker-lifecycle-cleanup.test.ts@99877a3f298b483b75a4756a14430c93fcede43e`, `src/infra/sqlite-worker-operation-attachment.test.ts@259eede9d9ca4f00b6365842ce47e6d486d9058c`, and `src/infra/sqlite-worker-operation-admission.timeout.test.ts@7511cb8ea7ab283094fecb5fc6ddcbcc7500738f`.

This establishes page-level exact-ref closure for the first 100 search results only. It does not establish repository-global caller closure because the discovery query itself remains capped and indexes the moving default branch.

### 15.2 Additional frozen production caller identities from the capped page

The following 33 frozen production identities were exact-fetched after the prior dossier append and were not previously recorded by exact SHA:

- Lifecycle/fencing: `src/infra/sqlite-source-fence-admission.ts@512a8f93e6c09edfab6c150f6b864b2dbb6f6ddb`; `src/state/openclaw-state-db-cache.wal.ts@8e7ebf8139af85b89ed80ec35774590a156181b2`.
- Product/domain state: `src/skills/library/store-access.ts@783eb79cb551140e0d245f4dcd6519b33753285f`; `src/gateway/managed-image-record-store.ts@8d7b9ee26135bcb856f3876d4c1e00e310871e2a`; `src/state/user-github-connections.ts@d67e92e4592045b6e3b5db97ca5adf969b19e544`; `src/state/user-profile-writes.ts@9a1e6ad80bfb328e317f57147d40bf00742c9dfb`; `src/infra/exec-approvals-store.ts@c3344574bf3451b662fd52eae1bb73b5278a5989`; `src/state/session-repository-workspaces.ts@c0d47b18ad67b1e721f21cbe1692cbe8cf112c11`.
- Restart/publication: `src/infra/restart-handoff.ts@110728a478545450994f93dfd8ece53a6e1235b3`; `src/infra/restart-intent.ts@4c3382cbe14d5394527a1e772a8a266fbe6e7512`; `src/state/github-publication-worker.ts@1855961b43fac5d1ef45c0474230e00002daadc9`; `src/node-host/node-worker-journal-worker.ts@63a3c7510b8e6b16c502d12019dc75795b4cdfb7`.
- Worker-environment/channel surfaces: `src/gateway/worker-environments/prepared-pool-presence-worker.ts@19e059602739d25159eb24ab73e426379830eb22`; `src/gateway/worker-environments/placement-worker-mutation.ts@5d33ef4746d17fd5e7fa82dfde681cd55963648c`; `src/gateway/worker-environments/store.ts@bba0f316959629efb648ab65ab8215a6585c8da2`; `src/gateway/session-group-catalog.ts@ea8d8c61139e1bf435a312fb9c00ae76789377c4`; `src/channels/message/ingress-queue.ts@55c883a8347a472bc08b2b5abee7fde9ecb695e2`; `src/infra/outbound/delivery-queue-storage.ts@943977d59dab5314043d3de1d3c80b9f023706c7`.
- Agent/auth/session/trajectory: `src/agents/worktrees/template-registry-async.ts@60a1fd9693262106e6146cc935ad581ebae14c89`; `src/agents/worktrees/registry-run-end.ts@d448b39cf133d716f22e265dd8d2cbb1b488a960`; `src/agents/auth-profiles/sqlite-read.ts@7ae3540a661bf7b94501bb58c90c94d59b0b8ed5`; `src/agents/auth-profiles/shared-store-bootstrap-async.ts@56ba6a6622b9c3efdf5b41db93e39084523d0015`; `src/agents/auth-profiles/authorization-enrollment-runtime.ts@84e2ae26619ac94ec12a323b8ca1fae2e993acc9`; `src/agents/auth-profiles/store-update.ts@0df87f292ea2d817b6884f22995862cc405c71e5`; `src/infra/session-cost-usage-cache.sqlite.ts@23b9f09f2d7f9207f975e5f3f6c7bc7f736a7aea`; `src/trajectory/runtime-store-writer.ts@3e543ac75e3626a4ccf62902856654633953eae5`; `src/sessions/session-state-events.ts@11f02fcf4ac063897845f1fdbd6d564dd6795fd9`; `src/talk/client-voice-session-write.ts@0012fbb019a8dcce404a2d544e08f126147606c2`; `src/state/openclaw-agent-worker-store.ts@918ab97a3a7142cd3cb3b4fb8c5c6f74e133efab`; `src/config/sessions/conversation-registry.ts@fee9df39bdf9682bf389f88f5187c08377d393e3`; `src/plugin-state/plugin-state-worker-client.ts@d52cf25581ecee6dbb3a4c95affea007840faf76`; `src/agents/subagents/registry/subagent-registry-persistence.ts@dd257d7adffa15d8c1ea4d75ec591b96d41468c5`.
- Explicit non-authority boundaries: `src/state/user-model-account-authority.ts@4e1b9618d1813a7c996fccd5148f8d1a4a46b642`.

The common production invariant is now source-grounded across the entire returned production page: callers bind domain-specific currentness, stage/order, identity, generation, publication, or settlement facts before native admission. Several callers deliberately deny extra authority. `github-publication-worker.ts` throws if bookkeeping unexpectedly requests source admission; `user-model-account-authority.ts` throws if machine-state writes request separate domain admission; `outbound/delivery-queue-storage.ts` creates an observer admission that grants no host transaction authority and retains queue custody when publication cannot be confirmed. `openclaw-agent-worker-store.ts` also restricts publication cleanup admission to prepare-only and refuses any cleanup attempt to open storage or admit a transaction.

Unknown-result handling remains non-replayable at multiple product entries. Restart intent recovers only from the exact committed consume receipt; managed-image writes and restart handoff likewise require matching committed facts; session state events explicitly do not replay an uncertain signal mutation. Ordered multi-phase callers such as plugin-state replacement, auth-profile update, skill-library writes, ingress claims, exec approvals, worker placement, repository workspaces, and subagent persistence fail closed when a transaction/commit sequence or authority fact is wrong.

### 15.3 New frozen test/support identities from the capped page

The following 27 frozen identities were newly pinned after exact-ref re-read:

- `src/infra/update-run-mutation.worker.test.ts@b1854b10220e3d95eac538960ff36619b19a797d`
- `src/infra/update-candidate-canary-mocks.test-support.ts@91ad96a62f8160d8113bfe25dbdb2bedf4557942`
- `src/infra/update-run-reconciliation.worker.test.ts@40af4a66a2bf56c939c2dc663610220fd7fad315`
- `src/gateway/worker-environments/store-retention.test.ts@6d756c6507a5c85a4ddc7e4a47026292583298e5`
- `src/gateway/worker-environments/store-recovery.worker.test.ts@f392a257d68a1b3cd918d680971ef8d97ba79c38`
- `src/gateway/worker-environments/credential-attachment-authority.test.ts@7eea89b0ba8095e9d1eff3d52ba9465ceb151976`
- `src/gateway/server-methods/approval.settlement.test.ts@b15642202cd86bcff6fc8eb8adf4dbaec3d07a03`
- `src/cli/update-cli/update-command-execution-validation.test.ts@971bea1c988950b4d614fd9f346376dcfa23c21e`
- `src/infra/sqlite-worker-open-refusal.test.ts@f5e5587c97e7159f8face94bffb92ee0a9491817`
- `src/infra/postgres-sync/fixture.test-support.ts@0a4930bea687a20acb1a625025edf94f988c594f`
- `src/infra/sqlite-writer-admission-client.test.ts@55aa25fcc67fa4b95bb28c1eb6c2e672c0744772`
- `src/config/sessions/session-worker-receipt.test.ts@60831e427193de2fc26b782b70e2b3cd69ab8ebe`
- `src/infra/sqlite-worker-owner-probe.test-support.ts@d2d5f9c48496b127242b3ef5cec98f96139da6b2`
- `src/channels/message/ingress-queue.authority.test.ts@4b7096359982e4ec9d2337229b1f9f88497a579a`
- `src/infra/sqlite-worker-cold-admission.test.ts@b64c474d7ec8e31a7d5051cb066acdff4041cbf6`
- `src/infra/sqlite-worker-operation-attachment.test-support.ts@7e3a46aa68b534e804a98d79ee4c42c022ff75af`
- `src/config/sessions/session-entry-patch-delivery.test-support.ts@d9ae7e50a03b9f1ce2345aa277cea7985886851f`
- `src/config/sessions/session-entry-patch.authority.test.ts@647ee8854dfd0e76af74c33ccab39c12af1a4bfe`
- `src/state/openclaw-agent-execution.creation-fixture.test-support.ts@e31eff1e9aa57f573ca380f1bf1b38b833fa3bd8`
- `src/infra/database-engine/agent-opening-refusal.conformance.test.ts@42256ba97c9633a0f8063d521ba9df0eec210521`
- `src/config/sessions/session-entry-deletion.test.ts@7f84fb13fb79196c2809edef2b3ea7c32d5e349c`
- `src/cron/service/admission-timeout.test.ts@11b942831cc17df276c17c2f7e95bb0dca1bfff9`
- `src/infra/sqlite-transaction.diagnostics.test.ts@7aa69295af06353f8c0d147291fe6440427daa91`
- `src/state/openclaw-state-lease-worker.test.ts@4f2055fe950166be13ec2534d25c1af212f0e186`
- `src/infra/push-web.test.ts@62441f17abc27dd5725a0e81fd08b09e1bf83ba0`
- `src/state/user-background.test.ts@1717184cf93968c8f1c88bf8b512a13afa3e25ee`
- `src/infra/update-run-contention.test.ts@72e9582bf3ab4d3bd03b5811b315a73c5cc3c62a`

The focused negative oracles include: reconciliation rejects cancellation at commit rather than returning stale history; worker-environment retention defers pruning when live demand changes at commit; unknown revocation/rollback does not manufacture a fence; attachment authority is not lent between queued callers and is rechecked at native commit; pre-native open refusal leaves pooled siblings and recovery intact; prior writer refusal clears only after fresh live authority; unknown session-worker settlement remains outcome-unknown and must not inspect/replay a commit; source-retired ingress is refused without revoking the shared account queue; patch authority is checked at the native mutation grant after asynchronous preparation; a post-commit caller refusal does not destroy another borrower's valid opening lease; session-entry deletion rolls back when cleanup authority is lost at native commit; transaction diagnostics prove admission precedes BEGIN and revoked COMMIT fails; user-background quota/current-authority failures roll back coherent state; and update contention does not replay or discard a lock failure after transaction admission.

Support-only fixtures above are recorded as source identities, not promoted into standalone behavior proof.

### 15.4 Page-level closure does not change the OCRM verdict

The first 100-result discovery page for `createSqliteWorkerOperationAdmission` is now exact-ref classified at the frozen commit, including explicit rejection of both moving-default-only paths. The repository-global caller set is still unknown beyond that page because search remains capped; other symbol families such as `createSqliteDatabaseAdmissionRelay`, `closeOpenClawAgentDatabaseByPathAsync`, shared-state lifecycle wrappers, and snapshot-source entrypoints still require the same frozen page/cluster closure.

No Actions were run. No production code, ledger, RTM, or flags were changed. Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, OCRM final acceptance, and release acceptance remain absent.


## 16. Round81 — database-admission relay exact-ref cluster

### 16.1 Complete 13-result discovery cluster at the frozen ref

The default-branch discovery query for `createSqliteDatabaseAdmissionRelay` returned 13 paths. Every returned path was re-fetched against frozen `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`.

Frozen production identities:
- `src/infra/sqlite-worker-operation-admission.ts@90f03a25678f00a88489c13392e19dbb09df6a40` — relay implementation, already pinned earlier.
- `src/infra/worker-task-host.ts@25a837385fd6bbca851c369abf2091cbd798f233`
- `src/state/openclaw-state-lease-heartbeat.ts@95a67834df4cfc17fed8387a71c7be2d1f4ab096`
- `src/config/sessions/session-accessor.sqlite-worker-coordination.ts@b4e805240b51228188ac7e8ff76b02c6a393a042` — already pinned in section 13.
- `src/config/sessions/session-accessor.sqlite-archive.ts@641db353303686f59e98c30ee46103a9225989d9`
- `src/config/sessions/session-accessor.sqlite-worker-request.ts@b2ce0e07a1134e74e84d994fb57abbcdc96fc4e6`

Frozen test identities:
- `src/state/openclaw-state-lease-heartbeat.test.ts@b4ade28ef5bdc68e5033abdbc187ede6b2a59ac8`
- `src/infra/sqlite-database-admission.retention.test.ts@596649f5c2a405c7bb570a2b9b97dec9f1d6edf5`
- `src/config/sessions/session-accessor.sqlite-worker-coordination.test.ts@cfe8b31c32e3ef5485fb1938f12cb8554898575b` — already pinned in section 13.
- `src/infra/sqlite-worker-operation-admission.test.ts@2ce87b63c700ee3d5f6f2d3487d57ee6dabd5760`

Moving-default-only discovery hits rejected at the frozen ref:
- `src/infra/database-engine/state-read-admission-owners.postgres.test.ts` — NOT_FOUND.
- `src/infra/database-engine/state-read-admission.postgres.test.ts` — NOT_FOUND.
- `src/infra/database-engine/state-read-admission-publication.postgres.test.ts` — NOT_FOUND.

Thus this 13-result discovery cluster is exact-ref closed for the frozen source, while the three PostgreSQL test paths are explicitly excluded as later default-branch additions rather than silently counted.

### 16.2 Production path and owner boundary

`worker-task-host.ts` retains one relay per native worker. The relay is installed on first use, serviced through the host's retained admission service, transferred only once to that worker, and `finish()` is tied to worker retirement. This makes database-admission transport part of the retained worker lifetime instead of a free-standing permission snapshot.

`openclaw-state-lease-heartbeat.ts` requires an established file identity before starting the actor, captures the current state-database admission, and treats sampled diagnostics as non-authoritative. Renewal/startup have explicit bounded timers. Commit publication can precede the ordinary actor reply, so the parent tracks committed expiry separately from reply delivery; outcome-unknown renewal failures remain distinct from an explicit lease loss.

`session-accessor.sqlite-archive.ts` creates a relay for the archive worker and closes it together with the one-shot worker admission service. Cold archive mutations keep their cleanup/exit lifetime instead of borrowing a sweep connection. State-database authority remains a separate captured object and is rechecked by the archive write path.

`session-accessor.sqlite-worker-request.ts` separates caller cancellation, database-owner closure, worker transport settlement, commit requests, and mutation authority. A worker's `admission-request` can create a database relay only after the parent write-admission path has accepted the operation and, when database authority exists, revalidated the captured owner. Parent authority failures take precedence over a generic worker unwind error; transport settlement does not erase a caller refusal.

The resulting production chain is:

`retained worker/session owner -> captured exact database identity/current authority -> per-worker database-admission relay -> on-demand fact exchange -> native transaction/commit path -> explicit relay finish on worker/resource retirement`.

The relay transports current database facts; it does not replace the domain mutation owner or make a stale database identity current.

### 16.3 Negative oracles

`openclaw-state-lease-heartbeat.test.ts@b4ade28ef5bdc68e5033abdbc187ede6b2a59ac8` proves retained lease callbacks stop after completion/closure, activation waits for a contended renewal to commit, and a caller revoked at commit is rejected rather than publishing the attempted write.

`sqlite-database-admission.retention.test.ts@596649f5c2a405c7bb570a2b9b97dec9f1d6edf5` exercises retained cold receipts and write scopes across physical-fact reads, invalidation, deletion, release, worker exchange, and clean close. Its source-level oracle is that a retained scope preserves the exact admitted fact only for its lifetime; invalidation is not suppressed merely because a scope is retained, and released/deleted scopes cannot keep stale authority alive.

`sqlite-worker-operation-admission.test.ts@2ce87b63c700ee3d5f6f2d3487d57ee6dabd5760` covers relay/operation integration and fail-closed lifecycle: finish is terminal and repeatable only with the same failure, late observers cannot reopen a closed admission, elapsed time before host servicing is not itself authority, and a close/revocation before the grant prevents transaction admission.

These are source/oracle reads only; no tests were executed in this audit round.

### 16.4 Frontier

The `createSqliteDatabaseAdmissionRelay` discovery cluster is frozen exact-ref closed. Repository-global SQLite lifecycle closure is still open because `closeOpenClawAgentDatabaseByPathAsync` itself returns a capped 100-result search surface and the broader snapshot/read-only caller families still contain unclassified test/consumer paths. Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, and release acceptance remain absent. No production code, ledger, RTM, or flags were changed and no Actions were run.


## 17. Round81 — agent-database close capped-page closure

### 17.1 Frozen page accounting

The default-branch discovery query for `closeOpenClawAgentDatabaseByPathAsync` returned 99 unique paths: 14 production/helper paths and 85 test/support paths. The frozen upstream tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16` was used as a second exact-ref identity check after the per-file reads.

At frozen `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`:
- 97 of the 99 returned paths exist in the frozen tree.
- `src/commands/doctor-a2ui-widgets.ts` is a moving-default-only production discovery hit and is absent from the frozen tree/fetch.
- `src/infra/database-engine/session-archive-alias.conformance.test.ts` is a moving-default-only test discovery hit and is absent from the frozen tree/fetch.
- The other 97 paths were exact-ref reconciled. Identities already present elsewhere in this dossier are not repeated below; only the 85 exact identities absent from the live dossier before this append are listed.

This is exact-ref closure for this returned search page, not proof that GitHub code search exposed every repository caller.

### 17.2 Newly persisted frozen production/helper identities

- `src/commands/sessions.test-helpers.ts@db1260b349aa0444a91b5621146116fa0eb8ba5f`
- `src/agents/cli-runner.test-helpers.ts@b71515fb78058b9baad2518516a4690aa7d88ae5`

The two helper files are test-fixture lifecycle owners rather than shipping product entrypoints. `sessions.test-helpers.ts` drains suffixed database owners before handing off fixture state, while `cli-runner.test-helpers.ts` joins fixture lifetime, agent/state database closure, auth-profile read pools, and owned resource cleanup. They are recorded as test infrastructure and are not promoted into production-completion evidence.

All shipping production paths returned on this page were either already pinned in sections 12–16 or re-confirmed here. Their closure chain remains:

`captured physical owner / maintenance or deletion authority -> drain registered session/read/worker resources -> closeOpenClawAgentDatabaseByPathAsync -> native handle retirement -> durable lease/registry settlement -> only then file replacement, deletion, migration, or handoff`.

### 17.3 Newly persisted frozen test/support identities

- `src/gateway/server-methods/agents-mutate.test.ts@de5c9342216271e382b352e3c70f9cdbf7e4f4be`
- `src/agents/cli-runner.prepare-fixture.test.ts@e60cca0579153b2b07bd5dcc504edf77017893ca`
- `src/infra/worker-task-pool.sqlite-idle.test.ts@90b587ed15d2388c703f3b7f4b288b8d1c851d91`
- `src/state/openclaw-agent-execution.alias.test.ts@05c79bc0b86539e004145461f23be2e2bccf0e08`
- `src/config/sessions/lifecycle-read.worker.test.ts@21cc03de409cb0e64f99d8ab22fc07307f27763c`
- `test/non-isolated-runner.agent-reader-fixtures.ts@70938a98b6de4d8cab5218cfe9f227aa8165bb49`
- `src/agents/session-maintenance/run.test-support.ts@55dc0da4365874e95b29107be0a5b8e338ad1a6e`
- `src/gateway/assistant-media-policy.lifetime.test.ts@c010cfda4c5769e7279629e537cfc9bff21cb342`
- `src/gateway/server.sessions.face.test.ts@997819ffd7979846c12adf3c81cf880e761dded3`
- `src/agents/cli-runner/session-history.async.test.ts@edacda478a6b6c9b35038f1e332257fe48811ccf`
- `src/config/sessions/session-sqlite-target.worker.test.ts@67f79f15c2b5acae85e812e9d9be23a124e6efc2`
- `src/config/sessions/session-transcript-read-pools.test.ts@cbb10d47a221f4533fc0739281c425370e39c4a8`
- `src/commands/doctor-maintenance.sqlite-reclamation.test.ts@14ad9baa1967664549dc31b3e9ce9ae4169ac4b8`
- `src/config/sessions/session-sqlite-target.ownership.test.ts@f1cc7bab71c30f2943bbb3fd8908486a8e621aad`
- `src/config/sessions/targets.test.ts@cf715aa132832d9c29dc947dee1934e7d3c168da`
- `src/agents/sessions/session-manager-metadata-runtime.test.ts@591ebbb8a9abbeb3ab727dece694db8c9d4db3a0`
- `src/infra/database-engine/receipt-lifecycle.conformance.test.ts@549f090958fc48eec8b6f16b57030ec61675ffb2`
- `src/config/sessions/session-history-eviction.archived.test.ts@0b67ff94fa7b3d36d519279dd48be3a3d01e5560`
- `src/state/openclaw-agent-db.postgres.test-support.ts@b1b6e28f24fa1c600ed710a593f9cb2fab3c87d0`
- `src/agents/sessions/session-manager-model-context-rewrite.test.ts@01b7d5e92a54e912130b6499fc51a999673310d8`
- `src/config/sessions/provider-review-store.test.ts@3c1f13438a85c69e479650ecca7d3f8fd7102e09`
- `src/config/sessions/session-transcript-events.worker.test.ts@1e47bfb2d8d18deddf8caf650e9cbf8239ba4645`
- `src/commands/reset.sessions.test.ts@0d2a55942f8063c58dd221edd1410058db54a0bb`
- `src/infra/database-engine/mixed-agent-readers.conformance.test.ts@c38c6d8ba05128cf5eaca51027428b8a56fb4e82`
- `src/state/openclaw-agent-execution.idle.test.ts@e67dbc4d228341c954eadae735a1c655df38c445`
- `src/gateway/agent-turn/agent-request-routing.session-id.test.ts@6cd4ecb7f4624f88a1d3ccf37f6ee671959161bf`
- `src/acp/runtime/session-meta-read.test.ts@02775e755b71f4b9125e0056e4fcaf006d611615`
- `src/agents/auth-profiles/candidate-stores.test.ts@73fa480a62c910d722fa4722d6c5c875a0cd5a69`
- `src/config/sessions/session-cold-storage.test.ts@ffb69bb7cae5c1a32b3c8620a88d1cf4cb58a572`
- `src/cron/session-reaper.test.ts@ec029bd79195ee186507ffa90da696d13beff3dc`
- `src/gateway/session-history-worker-lifecycle.test.ts@56cb8f4ce748afd244565fc97dc02339fec8f139`
- `src/config/sessions/session-accessor.sqlite-reclamation-preparation.test.ts@f4636ab37018de1b4bfd218fb5f75836cd214e05`
- `src/gateway/conversation-send.test.ts@71997d5a6a20bb83428e9545b90714ea3aeab2c1`
- `src/agents/sandbox/sdk-state-owner.process.test.ts@5761a56a6093d8081fc0711f8e5e0236aaa44a85`
- `src/commands/agent-exec.state-lock.test.ts@0741c51a386a841415f75b543a1e52a52852a70b`
- `src/cron/isolated-agent/session.preparation.test.ts@29511e2e9459899313f5d514336d4f64772a07a2`
- `src/gateway/control-ui-session-prs-staleness.test.ts@20cf075d88859d6c2c2f1dc20cc7c3944bf2e28b`
- `src/gateway/session-utils-read-lifetime.test.ts@da6f97ab4bd168dd85e6eec1b4b2e6817aae0fe0`
- `src/agents/command/session.exact-target.test.ts@ec79bed3d83777b1e2841237b5490a703ff0cc42`
- `src/gateway/server-methods/artifacts-list.benchmark.test.ts@4f70218955f7030a2262913aaada5c87d76bd93a`
- `src/config/sessions/session-transcript-reconcile.admission.test.ts@d3000a9489eedd29c867a88d470d3d9526e2131a`
- `src/gateway/session-sharing-preparation.test.ts@c64ea3b5641a07bc943956d4f36207e8c4d2d880`
- `src/gateway/control-ui-session-pr-access.test.ts@97f93df29ead605c1454b8a4ac66c9756cdfdf61`
- `src/state/agent-deletion-cleanup.test.ts@51cb29c3e0636a3199d30fc7906a364a11c98bda`
- `src/auto-reply/reply/reply-turn-admission.worker.test.ts@854267b887daa094a6c650f9702143b3206733e7`
- `src/agents/sessions/session-manager-hydration.test.ts@4121b78d29adb4405e76e5b1e9fe37038de73b89`
- `src/gateway/session-observer.worker.test.ts@0ddcf38853e6159d70b9e92dc842cffb3a064462`
- `src/gateway/server-worker-placement-provision-cancellation.test-support.ts@9dc341cea26990667195fe27a3f6bc259937f2f8`
- `src/gateway/server-methods.canonical-validation.test.ts@9d15865546806a6474a00846ce51bc31c6687c86`
- `src/state/openclaw-agent-doctor-media.postgres.test.ts@51f9ed1b56fc9fc7e42dfd3d8820a98e1efecd43`
- `src/gateway/conversation-turn.test.ts@5453100b8cbac5beb081e7a81402dbfc1b292927`
- `src/agents/auth-profiles/store-state-owner.test.ts@5d2e7eb9e47577c8321c6780585ce24b0a73eaaf`
- `src/gateway/test-helpers.server-rpc.test.ts@4d6edb57af049c1cb91e23cb6ec5b84204ea4532`
- `src/state/openclaw-agent-db.postgres.test.ts@d0eb281bacee5ddd0c279215d42aee8eeddae571`
- `src/agents/harness/native-session/binding-authority.prepared.test.ts@32f819f5243fcace3d3c61eafb2eff975b01fd1d`
- `src/config/sessions/session-entry-read.worker.test.ts@6edad9b228bcc648c68ead4d466c8acb7baefdc8`
- `src/gateway/session-access-authority.test.ts@4c964168c452e3a0196d5be5c7513ebfb4e3d813`
- `src/config/sessions/session-transcript-search.worker.test.ts@baaa492e0d65375001f6e1b8b285c90b1047a977`
- `src/config/sessions/session-accessor.sqlite-maintenance-close.test.ts@e173a3eadd97e2896aca30b131baaa07f9de50a4`
- `src/agents/auth-profiles/oauth.peer-settlement.test.ts@bab977c7123d1f531b183c84577e368cd7fec501`
- `src/infra/database-engine/gateway-agent-inspection.conformance.test.ts@8b464622de8115e39891e185dfc7b52a8f117ce7`
- `src/gateway/control-ui-session-prs-publication.test.ts@21da3f16e6ac2f323d87b88b3ed5228ea835889b`
- `src/state/openclaw-agent-db.registration-settlement.test.ts@4796fbf08929fe3eb0330dd61f1c025caa0f268c`
- `src/infra/session-cost-usage-cache.admission.test.ts@270c004e73419c5037a9d58de05617a0c24877e0`
- `src/infra/session-cost-usage-worker-env.test.ts@f6eb3d02e41581cd172def5588150b3abd723d99`
- `src/config/sessions/session-actor-worker.test-support.ts@198c816851e8c12d58b1a895469a11928f1036b8`
- `src/config/sessions/session-history-eviction.admission.test.ts@c3e7b798196ee74b851b45872edf36cf0c6b0270`
- `src/gateway/server-methods/models-session.test.ts@5f5cc82c5b275680d740fe89d214a90af40924eb`
- `src/gateway/managed-image-attachments.authority.test.ts@ea06c0e964f7c419c0435f6b0be3104f20b08038`
- `src/config/sessions/session-history-eviction.test.ts@28d55d1c9c4fb4463a232f4191cf3a2dd407dd5e`
- `src/config/sessions/startup-migration.acp.test.ts@e06e747aa5b07528d633617c076f5c4fde02cc0e`
- `src/config/sessions/session-transcript-reconcile.lease.test.ts@b086cf9a05f0ff8788972f7794dd316a9f2b6130`
- `src/gateway/session-row-transcript-backfill.test.ts@803d336f08129f62c2a69a468312475c1254ae8e`
- `src/config/sessions/session-entry-current-check.test.ts@2c36aa79eaad101a78d43daa571b1b74b0cd4bb4`
- `src/boards/sqlite-board-store.worker.test.ts@46a81c493f06715724b8afc526c837ec052598ef`
- `src/state/openclaw-agent-db.checkonce.test.ts@6567ecc7af16b579ec20b093aec949c092291af1`
- `src/config/sessions/transcript.media-identity.test.ts@502c7ce875483307c4efa65bbb1a7bdd0f0e14d9`
- `src/gateway/server-methods/chat-send-reply-session.test.ts@4608b20fc8953c3816caa5d4b4e2fd5c9bb826c2`
- `src/config/sessions/session-history-eviction.lifecycle.test.ts@d32227bebfd6d64f78e41574c324c952dbb166b1`
- `src/auto-reply/reply/reply-turn-admission.database-claim.test.ts@43a32bd3c1f63669aff6e57c121f3099d0d6f6b0`
- `src/config/sessions/session-entry-readonly.worker.test.ts@968061da3df84e21c3c5cc1bf324e34caccbda0e`
- `src/agents/anthropic-transport-stream.live.test.ts@b595596e4153424e4ee86c0aaef99dc2f6c3aa8c`
- `src/gateway/session-row-projection.accepted-facts.test-support.ts@5d20960331a60682501fd1f4e61441df44692725`

The per-file reads group these negative oracles into distinct lifecycle classes:

- **Physical identity and replacement:** alias/reopen tests preserve admitted creator reads on the original physical file, reject successor disclosure, fail closed on mismatched owners, and invalidate same-path/same-row cached reads after physical replacement.
- **Close joins accepted work:** CLI fixture cleanup, transcript/history pools, maintenance finalization, agent deletion, session reconciliation, usage cache, session actor/read workers, and Gateway fixture release all require accepted reader/worker/resource work to settle before file cleanup or owner retirement.
- **Authority at the final native edge:** lifecycle read, session-entry patch/deletion/current checks, Board writes, provider review, channel/reply admission, repository/workspace/session operations, and auth/OAuth flows recheck current authority after asynchronous preparation and reject revoked commit/use rather than publishing stale success.
- **Unknown outcome is not replay:** conversation send/turn, registration settlement, worker-backed session operations, and publication tests preserve unknown/committed receipts separately from ordinary reply delivery and do not resend a mutation merely because a reply or cleanup acknowledgement was lost.
- **Failure custody and recovery:** failed idle eviction, native close, archive/retention cleanup, resource drain, or cleanup publication keeps explicit custody until retry/reopen/close settles; healthy sibling readers/workers remain usable when the failed owner is unrelated.
- **Read-only and schema boundaries:** read workers, target discovery, Doctor/port/mixed-engine conformance, PostgreSQL support tests, and cold-storage tests keep inspection/read authority separate from write authority and reject unavailable/ambiguous ownership instead of manufacturing an owner.
- **Cancellation and deadline:** reply admission, transcript search, session preparation, Gateway provisioning, and live-stream/worker tests distinguish caller cancellation from database-owner closure and ensure cancellation cannot leak or silently convert an accepted native mutation into rollback/success.

These are frozen source/oracle identities only. No test was executed in this audit round.

### 17.4 Remaining closure frontier

The first returned `closeOpenClawAgentDatabaseByPathAsync` page is now exact-ref reconciled, including explicit exclusion of both moving-default-only paths. This does not establish repository-global search closure because GitHub code search remains capped/indexed against the moving default branch. The next exact-ref frontier is to perform the same page/cluster reconciliation for `openSqliteReadOnlyDatabase`, `prepareSqliteReadOnlyLocation` / `startSqliteReadOnlyLocationAsync`, and remaining wrapper references around `openclaw-state-db-async-lifecycle`, then compare their frozen path sets against the cumulative Round87 dossier.

Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, OCRM final acceptance, and release acceptance remain absent. No Actions were run and no production code, ledger, RTM, or flags were changed.


## 18. Round81 — snapshot-source entry clusters exact-ref closure

### 18.1 Combined frozen identity set

The discovery sets for `openSqliteReadOnlyDatabase`, `prepareSqliteReadOnlyLocation`, `startSqliteReadOnlyLocationAsync`, and `withSqliteSnapshotSource` were merged by path and reconciled against frozen tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16` at `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`.

The merged moving-default discovery set contains 126 unique paths; 124 exist in the frozen tree. Two default-branch-only test hits are absent at the frozen ref and are excluded from frozen evidence:
- `src/infra/gateway-lock.hostname.test.ts`
- `src/infra/doctor-capture-reuse.test.ts`

Every frozen identity newly listed below was also re-read at the frozen ref in this round. Exact identities already recorded in earlier Round87 sections are omitted rather than duplicated.

### 18.2 Newly persisted production / documentation / E2E identities

- `src/state/openclaw-state-db-schema-discovery.ts@ada041a24b45ec995f11d2a58a3aec5119a84e9a` (openSqliteReadOnlyDatabase)
- `src/infra/deferred-plugin-session-transcript-read.worker.ts@09bcfb86aec221600bd738ef1b8d4b563188612d` (openSqliteReadOnlyDatabase)
- `src/agents/auth-profiles/store.worker.ts@61521f804456169fa2be9cddcf351a15755f38f1` (openSqliteReadOnlyDatabase)
- `src/state/openclaw-state-db.ts@adb2af202d485fc4d98fb405935921da8efd2689` (openSqliteReadOnlyDatabase, prepareSqliteReadOnlyLocation)
- `src/state/openclaw-agent-db-readonly-open.ts@1796b0d3d312ddbbda2fc967b1e8454544e92da9` (openSqliteReadOnlyDatabase)
- `src/infra/session-sqlite-migration-readers.ts@8b884c2ff641d561aec01dda547e7ecc5a84b708` (openSqliteReadOnlyDatabase)
- `src/infra/sqlite-readonly-location.worker.ts@c1bcd2687aa4c3d9ce7dbd9310fdc2ca3e90b6c5` (openSqliteReadOnlyDatabase, prepareSqliteReadOnlyLocation)
- `src/state/openclaw-state-read-snapshot-source.ts@4d97a8303f5e467e9fb9a09178b0e3e1253940ee` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/infra/sqlite-snapshot.ts@535adc4ecc41b46357daa83d2b7a004d4c50d5c7` (prepareSqliteReadOnlyLocation, withSqliteSnapshotSource)
- `src/state/openclaw-state-db-readonly.ts@5bd95bc9cf5aa3e1dc24a2d9c4dbbc03e46a67f8` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-database-preflight.ts@b826b7c26a13868ea008902bdd37c08d9d123e06` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-agent-deletion-journal-quarantine.ts@b9d245d9edfb55b61a26c07bf516405c422dd515` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-live-snapshot.ts@d5d47ff07eee4e247419c35d108aec99491ec799` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-state-db-fast-path.ts@8747cb713eb57d6dfef6b95cbe415b1dcd244c86` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-state-read-operation.ts@2f8740a2800aa5190b00621da5b7f74bdd17e4e2` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/commands/doctor-lint-state-snapshot.ts@08e304c6d246e644aff84a9399bdf3284524ef0c` (prepareSqliteReadOnlyLocation)
- `src/infra/update-candidate-state.postgres.ts@1807749be33a5f92388906baa2034d984e9c786d` (prepareSqliteReadOnlyLocation)
- `src/infra/update-database-generations.ts@f10a8b0119cb37026a493e2064a9796a67a8638f` (prepareSqliteReadOnlyLocation)
- `src/infra/gateway-owner-lease.ts@c7c94b2ebe1d17db183df1cab2f80c865b795627` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-readonly-location.ts@c0d765a2cea3faf06300bdbc2edcd77e8ee082f5` (prepareSqliteReadOnlyLocation)
- `src/infra/update-candidate-state.ts@0fe5c863f20cb8b7e8a53d8c345eaee681f9e1f4` (prepareSqliteReadOnlyLocation)
- `src/agents/auth-profiles/shared-store-bootstrap.ts@1e17ac08598fe53cc014c6163d8a911b98606a5c` (prepareSqliteReadOnlyLocation)
- `src/infra/database-engine/diagnostics.ts@bc1a7e305b5ed2e9b2f8dc30613b63d47c2675ba` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-agent-db-lease.ts@563373e63e1b0b9aae13f073e74319b3421efe9f` (prepareSqliteReadOnlyLocation)
- `src/infra/database-engine/port-command.worker.ts@4eeb2269476f6d8d3cb5b0492a93dc9b114dcf15` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-state-db-task-delivery-recovery.ts@eaee7cc7b4e4ea88869ff0099a7272ed53758273` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-agent-deletion-journal-restore.ts@b131398df443d85a48e2b70e8256b9b3e5e5d8dd` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor/shared/session-entry-rewrite.ts@83444e017030dbf1c83bd300d957ec322bf8719d` (prepareSqliteReadOnlyLocation)
- `scripts/e2e/lib/upgrade-survivor/project-worktree-startup.mjs@7b32a76dc62ff3f3d7d9835e1883f913b27ae47c` (prepareSqliteReadOnlyLocation)
- `src/agents/test-helpers/prepared-model-catalog-inspection.worker.ts@09c9e891074d7c042f369988aaf3e6aecb3a0162` (prepareSqliteReadOnlyLocation)
- `docs/plugins/sdk-runtime.md@2642713c549589a9ac8a499b552eb9b82e39b8cb` (startSqliteReadOnlyLocationAsync)
- `extensions/browser/src/browser/system-profiles.ts@9ad071baf9cb432b7f20a352d3696abe203253c2` (startSqliteReadOnlyLocationAsync)

The production paths resolve into four owner clusters rather than one generic snapshot service:

1. **State/agent read owner:** state and agent read-only wrappers capture current physical/database ownership before exposing rows. Clean-close fast paths may read the original file; artifact-preserving or live-WAL paths use private snapshot custody. Read-only handles have explicit close/cleanup ownership and cannot grant mutation authority.
2. **Snapshot creation and staging owner:** `sqlite-readonly-location.ts`, `sqlite-live-snapshot.ts`, `sqlite-readonly-location.worker.ts`, and `sqlite-snapshot.ts` pin source identity, validate main/WAL family consistency, stage private bytes, and retain tokens/directories until native readers and cleanup settle. Source identity drift or target replacement aborts publication rather than copying successor bytes.
3. **Doctor/update/maintenance inspection owner:** preflight, Doctor journal restore/quarantine/session rewrite, update candidate/backup/generation, Gateway owner lease, and database-port inspection borrow private read snapshots under the caller's existing maintenance/repair authority. A snapshot is evidence/input, never a write permit; cleanup failure remains separate from the diagnostic or repair result.
4. **Consumer-specific read projection:** auth-profile readers, migration readers, plugin/runtime inspection, Browser profile capture, and test/e2e upgrade-survivor consumers use the same snapshot/read-only contracts but keep their own product identity and publication rules. The Browser cookie path is an external profile snapshot consumer, not an OpenClaw state writer.

The common source path is therefore:

`captured current source identity -> retained preparation/staging owner -> private/read-only SQLite location -> consumer-specific read/inspection -> native reader/transport close -> retained token/directory cleanup -> retry on cleanup failure`.

A successful query is not proof that the physical snapshot can already be removed.

### 18.3 Newly persisted frozen test/support identities

- `src/infra/sqlite-snapshot-source.rehearsal.test.ts@cb3d6118fad7c7c5ae4e775d4b1d8935feafa1bc` (openSqliteReadOnlyDatabase)
- `src/agents/auth-profiles/sqlite-read-pool.test.ts@29cc4655feceb86edad005fd268ccb5208f33c26` (openSqliteReadOnlyDatabase)
- `src/agents/auth-profiles/file-state.test.ts@de667efc7d8dc85a51f67bb3a7c6e82d8a10437f` (openSqliteReadOnlyDatabase)
- `src/agents/auth-profiles/runtime-snapshots.test.ts@59713bb68ba30956006b9b8954dd261336c0913c` (openSqliteReadOnlyDatabase)
- `src/agents/auth-profiles.sqlite-read-pool.test.ts@3fb76511bc6e7a04de2da53c4fd94b63028fde03` (openSqliteReadOnlyDatabase)
- `src/commands/doctor-lint.private-auth-retirement.test-support.ts@7b0a131a6d42acca51bc83bc4f755bc90af1f098` (openSqliteReadOnlyDatabase)
- `src/gateway/server.agent-database-recreation-product-proof.test.ts@607703a7bc9966cc17773dfdebcf651ac12a3dae` (openSqliteReadOnlyDatabase)
- `src/commands/doctor-codex-native-inspection.test.ts@f326dcd691698b494ea850785589c7655ea4cdba` (prepareSqliteReadOnlyLocation)
- `src/infra/database-engine/backup-worker.test-support.ts@b48f57544a5899af165afff52516a340d77e4cd8` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-readonly-location.cancellation.test.ts@531f50de5d30ee42d7c966d42318bcc2cfbaa38d` (prepareSqliteReadOnlyLocation)
- `src/state/database-preflight-fixtures.test-support.ts@5efb168688048078f8d3b8ce76ceef6f9311e167` (prepareSqliteReadOnlyLocation)
- `src/cli/update-cli/update-command-schema-cleanup.test.ts@b4c261836015ee6c788792beaf136c90b84ee78b` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-readonly-location.test.ts@c382e670240d7fff3b3a564b1e1aa1e7749af067` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-database-preflight.snapshot-cleanup.test.ts@46e1ed2a41da8839b004f99850e5368d88140ee4` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-state-db-readonly.test.ts@6a89e2e1cd1953691cacb7089f751b5d2a57f50d` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/state/openclaw-state-db-readonly.retained.test.ts@a2e6383beba2b39a22c6e1a87336c7d03ecb3f52` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/infra/sqlite-readonly-worker.test.ts@895bfdd78d9cafe3c9baa8092b1acc9dfd83e065` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/state/openclaw-database-preflight.lifecycle.test.ts@b9a35df854e4acde9b039ff5bea2118554b86e88` (prepareSqliteReadOnlyLocation)
- `src/infra/database-engine/backup.test.ts@8659fac3aa48c139d10f4262515c2c06345ddb0e` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-database-preflight.test.ts@5487369453c64ddfdf893d46147c80243d56fa9a` (prepareSqliteReadOnlyLocation)
- `src/flows/doctor-health.update-schema.test.ts@6d500994619660e6f16a938537680b3c5c217755` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-lint.sqlite-preservation.test.ts@96efe02be34960eca9c97df4e23594697580a90b` (prepareSqliteReadOnlyLocation)
- `src/gateway/server.roles-allowlist-update.test.ts@7f8078f4334cb8868cc74f45d8c8db5fb681f575` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/state/openclaw-state-db-readonly.source-identity.test.ts@25f210cfbefdc44c03c528915432d90f208c13ac` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/infra/sqlite-readonly-location.cleanup.test.ts@6f0ef3604d6d4939c89e7b84640160e94f6a87c0` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/infra/sqlite-snapshot-staging.cancellation.test.ts@7058c9fe73dc93b24103bdf7340e725a7441fa47` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-update-schema-guard.test.ts@b2c266b1122c4d412e05d4bad4829160249dfa27` (prepareSqliteReadOnlyLocation)
- `src/infra/database-engine/port-command.recovery.integration.test.ts@446f53c8b429aad62aff8271495748c0744a51e7` (prepareSqliteReadOnlyLocation)
- `test/scripts/vitest-worker-artifacts.test-support.ts@2dd0f289bf9fb2ff752616c6d7c91e8098f51e58` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-update-schema-guard.legacy-windows.test.ts@b8ab753a48928d39a2c43f2c95d34963fd0805cd` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-database-preflight.startup-prepared.test.ts@f9a46b807cf672f99c2ef59ba32fa7d3025c51d1` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-database-preflight.startup-integrity.test.ts@c539a447781e5eede600b275b4419bed4b861767` (prepareSqliteReadOnlyLocation)
- `src/state/openclaw-state-db-readonly.snapshot-admission.test.ts@1e3aacff09572c0a154e0fad6bc6a799441999b6` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/commands/doctor-lint.test.ts@b0f65a892617a620dccc444b267cfeb34e273fc1` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-db-bloat.read.test.ts@cb1a50d5e7edd181aeba65fb32638e0cd1627db5` (prepareSqliteReadOnlyLocation)
- `src/infra/update-database-generations.test.ts@ba11ef700f71889b1df4b6cb8e640e5738ee9796` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-snapshot-source.identity.test.ts@b803b3b760955344ad18eb17363b93c42cf2c596` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/config/io.plugin-metadata.snapshot-reuse.test.ts@6cda4cd7b32230cc26c687b46326c59b5b97b912` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-readonly-location.interruption.test.ts@5a9c585586264584275c3256e394fc3417717891` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-snapshot-cow.test.ts@132451cc9ad2eff53124a07c29e1427872d006fd` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-readonly-location.owned-native.test.ts@4317eec0ff2e28ce7b80ee1cc726afaa52f19ae4` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-maintenance.admission-activation.test.ts@44bb9635681e9ac9118389c26fca676c27b70dc2` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-readonly-location.copy.test.ts@46bdc1e90bf4f3bf97ac2abe84c33704245a6b82` (prepareSqliteReadOnlyLocation)
- `src/cli/update-cli/progress.observation.test.ts@f8f2f952bf3709bdcc04e202de8b3a495f7d935c` (prepareSqliteReadOnlyLocation, startSqliteReadOnlyLocationAsync)
- `src/infra/sqlite-readonly-worker.compile-cache-parent.test-support.ts@90e4dbd733eaeea52f24153f6b1a4117269a4a64` (prepareSqliteReadOnlyLocation)
- `src/cli/update-cli/update-command-legacy-finalize.test-support.ts@98930fc7dcce148cddd564fa477de037f2b97a3d` (prepareSqliteReadOnlyLocation)
- `src/infra/update-repair-turn-worker.test.ts@2b00379bf50ad0df96c09fda512932000a547364` (prepareSqliteReadOnlyLocation)
- `src/cli/update-cli/update-command-recovery-cost.test.ts@feb7495c400319aaab7837e95a15dc7d4d2addc1` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-snapshot-staging.ownership.test.ts@0735f38c6fae40bb11364a50cc5a644b6203e0b8` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-workspace-alias.test.ts@ff3041ac94e9b9396e0a3309bcf2f49235dc6ac6` (prepareSqliteReadOnlyLocation)
- `src/daemon/runtime-pin-service.test.ts@fe2bd4b31014940fce253ddc82f65ae77ae3e474` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-readonly-location.worker.test.ts@bb54b1a309a79205b8cf0115665e73101ba0f5e6` (prepareSqliteReadOnlyLocation)
- `src/infra/gateway-owner-lease.test.ts@15bc343fd03cb91e4e7965f05b217aac377d3a4a` (prepareSqliteReadOnlyLocation)
- `src/infra/update-database-backup.test.ts@252e5b9586d24b422ca44bda502e9bc3ae5c8336` (prepareSqliteReadOnlyLocation)
- `src/gateway/server-methods/update-status.test.ts@854f906db046ffa9bec21af33f6e774187d0248a` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-maintenance.admission-warm.test.ts@f304c7db84176c3dd4cbbe77e6dd42926335ecb6` (prepareSqliteReadOnlyLocation)
- `src/infra/database-engine/maintenance-custody.conformance.test.ts@543e27c19f1deaac6464cf9aaf053d259d87546c` (prepareSqliteReadOnlyLocation)
- `src/cli/update-cli/update-command-recovery.test.ts@e61440199969ca2586b79962e195bbd2fc76234f` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-lint.state-isolation.test.ts@1fb7d4989b45ab90f9090e77275e7485fbda7c38` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-maintenance.installation.test.ts@2c3c6d41de482c2a4466b1d876134b62afe096f7` (prepareSqliteReadOnlyLocation)
- `src/cli/update-cli/update-command-terminal-record.test.ts@c97be7dabdcd33f761a2847c2c1c5a3f09f5017b` (prepareSqliteReadOnlyLocation)
- `src/cli/update-cli/update-command-terminal-authority.test.ts@00d8a219fe009728c917a62da88a479efb3d88ff` (prepareSqliteReadOnlyLocation)
- `src/gateway/talk/handlers/client-create-authority.test.ts@bf797ae1a86d5c86390592f9885c4dc24074dc17` (prepareSqliteReadOnlyLocation)
- `src/commands/doctor-maintenance.finish-revalidation.test.ts@910517d0a2034bf8c2013be12005788c7b736c60` (prepareSqliteReadOnlyLocation)
- `src/infra/sqlite-snapshot-staging-owner.test.ts@b2fde4434537132b0f8738b15c41949ea9da4581` (startSqliteReadOnlyLocationAsync)
- `src/commands/doctor-config-preflight.pending-read.test.ts@497e9e7998bd0f1d3004cdb03c79a1706b6f5fd5` (startSqliteReadOnlyLocationAsync)

The frozen negative oracles read in this cluster establish:

- **Identity fencing:** successor bytes substituted after validation are rejected; copied/reopened databases do not inherit prior source authority; identical pathname or matching size is insufficient without the captured physical identity.
- **WAL and source-family consistency:** main/WAL resets, sidecar changes, checkpoint-restored main bytes, and source-family drift are retried or rejected; only a coherent admitted generation is published.
- **Cancellation and shutdown:** cancelled or closed snapshot scopes join child/native work before releasing staging ownership; a child that ignores graceful shutdown is still joined through bounded fallback; no late launch is allowed after owner shutdown.
- **Cleanup custody:** failed native close or directory removal preserves private bytes and the exact cleanup owner for canonical retry; nested/independent owners must all release before parent retirement.
- **Read-only authority:** artifact-preserving inspection refuses shared-state writes, Doctor/update inspection cannot convert a retained snapshot into mutation authority, and maintenance/current admission is revalidated separately.
- **Update/Doctor integrity:** backup, recovery, schema-guard, terminal publication, port migration, and maintenance tests retain verified source/backup identities through the actual mutation edge and refuse changed ownership or missing custody.
- **Performance/lifecycle boundaries:** warm/read-only caches may be reused only while their owner identity remains current; idle close failure retains native custody and cannot open unlimited replacement readers; progress/status readers avoid unnecessary snapshot creation.
- **Publication/result separation:** cleanup or observation failures do not rewrite already committed diagnostic/update results, and an ordinary successful read does not erase deferred cleanup failure.

These are source/oracle reads only. No tests or Actions were executed.

### 18.4 Status after snapshot-source closure work

The combined returned discovery sets for the four snapshot-source entry symbols are now frozen-ref reconciled for this round, with both moving-default-only paths explicitly excluded and all newly persisted frozen identities re-read. This still does not establish repository-global closure for arbitrary indirect/dynamic wrappers, nor does it establish scoped G01/G02 positive evidence.

The remaining source-only frontier is to enumerate indirect wrapper consumers around `openclaw-state-db-async-lifecycle` and the four lifecycle methods it returns (read admission, async-resource registration/drain, native retirement, close/retry), then exact-ref map their production callers/tests and compare against the cumulative Round87 dossier. Independent ACCEPT, `unit_authority_ready`, executable/package proof, OCRM final acceptance, and release acceptance remain absent. No production code, ledger, RTM, or flags were changed and no Actions were run.
