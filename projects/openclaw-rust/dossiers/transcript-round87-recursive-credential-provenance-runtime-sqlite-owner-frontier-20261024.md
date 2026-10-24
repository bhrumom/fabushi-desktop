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
