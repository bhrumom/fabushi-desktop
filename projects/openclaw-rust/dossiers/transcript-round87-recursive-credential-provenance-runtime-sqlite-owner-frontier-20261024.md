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


## 19. Round81 — shared-state async lifecycle wrapper and close-page exact-ref closure

<!-- Round81 shared-state async lifecycle wrapper closure marker -->

### 19.1 Direct lifecycle wrapper discovery sets

The moving-default discovery sets for `registerOpenClawStateDatabaseAsyncResource`, `captureOpenClawStateDatabaseReadAdmission`, `captureOpenClawStateIntegrityAdmission`, and `publishOpenClawStateDatabaseWorkerAdmission` were reconciled path-by-path against frozen `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`. The returned set sizes were 64, 41, 2, and 9 respectively. Relative to the cumulative Round87 dossier, 42, 28, 1, and 7 returned paths required frozen re-read; overlaps were deduplicated. No newly missing wrapper candidate in these four sets was accepted as frozen evidence without an exact-ref read.

New production/direct wrapper identities:

- `src/infra/push-web-store.scope.ts@23d1db86930961e1c612f694701a14c24c27fc67`
- `src/state/openclaw-state-lease-cleanup.ts@dbd2944cb7b78b2a670cc65dd0b6e8b6df900290`
- `src/agents/sandbox/registry-currentness.ts@6ca2c97cea15b8c5c658e7384b7a08fcf46bd4d9`
- `src/infra/device-pairing-tokens.ts@5e2f2662f1401e0f1f40f419a95557b458875027`
- `src/proxy-capture/store.async.ts@8573b9411bb0d71f06b868feb57a499372551ead`
- `src/infra/exec-approvals-cron-policy.ts@f47a9e9b7e3fe9ef3d40b5a8a14df7bb42038662`
- `src/channels/message/ingress-queue-health-cache.ts@1371ec2378d2b0c3ff4d52a0edc728f82ba190a1`
- `src/skills/lifecycle/upload-store-install-owner.ts@70b2aafd1cbe560909f94a6bda0341fd8e194eac`
- `src/cron/store/read-only.ts@30f151b914918092a7015380cd3de5e90b513251`
- `src/infra/device-pairing-publication.ts@a8d9e7bcbde564ebf8d08cd00ca7fd8e6cf97666`
- `src/agents/auth-profiles/file-state.ts@63839e1f215b44b474b52195c5979342ddbc2a61`
- `src/gateway/session-utils-store-sources.ts@f88d78469cb680a60d7498e3d6bddfdffb0abdb4`
- `src/state/openclaw-state-db-current-reader.ts@0550cc0d4e91cf296149dc051a698b377b42421c`
- `src/state/user-profile-events.ts@18e2a7e6982576f8009096680925833fae9515dd`
- `src/agents/subagents/registry/subagent-registry-queued-registration-wait.ts@c109d8566f55f90cdec2400ba86bc63d11251682`
- `src/gateway/github-oauth-lifecycle-helpers.ts@22938008c08fa66c9ce2d5882ebcbb17d60360fb`
- `src/state/openclaw-state-read-worker.ts@d247c009cf515b8d42623512fd400efc792fbe5e`
- `src/state/openclaw-agent-execution-owner.ts@a96b04c0fd188ef1c3f621ba246a6cdb4ae009d1`
- `src/config/sessions/session-accessor.sqlite-canonical-worker-pool.ts@c9a493148783df0e8f9042301f22ab821a12c2e1`
- `src/config/sessions/session-transcript-worker-resources.ts@ffd341bd00fad8380d81181124bac3ab6aa5d454`
- `src/config/sessions/session-accessor.sqlite-reclamation-worker-lifetime.ts@47b74b249e5c398c908c84180b097e048b0f9712`
- `src/state/openclaw-state-worker-context.ts@5600eb0b0734e01b4b09fae5fa90c404dc376c7f`
- `src/plugin-state/plugin-state-operation-epochs.ts@ce11d357cee6dba6765f83ec40be7763becfd650`
- `src/state/openclaw-state-db-readonly-reuse.ts@6e007de1d9f17b9b3e0b4f2d1baf48858c56eb02`
- `src/plugin-state/plugin-state.worker.ts@99beb7ae2ddc3dc7c003221e197d980e0521f442`
- `src/agents/subagents/registry/subagent-registry-read-cache.ts@9e2d4502a94394e2d39b5a6090a60db0609e9e38`
- `src/state/openclaw-state-db-schema-runtime.ts@353f481db2749a49c8c2a42df4327b2894733dd7`
- `src/state/openclaw-agent-execution-native.ts@16712ac15dc69461baf6f0b4ff0367294578bd96`

New focused test/support identities:

- `src/state/openclaw-state-lease-async.cancel.test.ts@9b2f1a60c067a96e972c0059a623a6c8811a5c43`
- `src/trajectory/runtime-retention.test.ts@b3259ab6127d839b23fe535827d5fed050f7d8f9`
- `src/state/openclaw-state-worker-lease.test.ts@6c19da4c4546dc2b6f3bff8633ad36d7a4fcbc44`
- `src/projects/project-registry.authority.test.ts@6dc1bef97b2e3fe243bbcbc4c6737450a2cb9886`
- `src/state/openclaw-agent-worker-store.env-capture.test.ts@9b1b4eee583263b2b1180839f5d01f331e73f911`
- `src/config/sessions/session-accessor.sqlite-worker-request.lifetime.test.ts@df3e54b6e737088ccda55446005ac52755b40dc9`
- `src/state/openclaw-state-lease-async.test-mocks.ts@9898e5d97158ac9f66401b9c4eb8ac368683c7cf`
- `src/cron/store.readonly-worker-cleanup.test.ts@59dc81b5450bdf45ebe0ff86f26f1366a8ba56f2`
- `src/cli/runtime-cleanup-scope.retirement.test.ts@e9bc88b426b29a4d727ea791d0a816e4879d94f1`
- `src/test-utils/temp-home.test.ts@5e4a0dfaabb8e8da2829c366a8392d3341be8e0b`
- `src/infra/device-bootstrap.maintenance.test.ts@7c874285d01f13e75d4b45371881a7e38e1a7050`
- `src/state/openclaw-state-db-async-lifecycle.test.ts@3dbf4c77279d7759d2cc3ae9b8d8773e1fd8550a`
- `src/config/state-corpus-fixture-lifetime.test.ts@d34dcebbf14e651dd0dc59d364d7e4c36144ad9f`
- `src/gateway/worker-environments/service-inventory-recovery.test.ts@79806d4aed5c98a371df8aa6ba708c08e9711721`
- `src/gateway/test-helpers.server-storage.test.ts@46ae1a2ebc72a52a0adec8fcd9c5d42806af2b22`
- `src/agents/subagents/registry/subagent-registry-pressure.test.ts@3b0f95e9d1706a4747722da51fb3e5a8b23ff4b7`
- `test/non-isolated-runner.sqlite-fixtures.ts@77b3ff3d6da36579dea3ebc390588554e8a814be`
- `src/state/openclaw-state-read-worker.test.ts@42ddc8320e9b4b2dbae3d2c03e7310c3cc82710d`
- `src/agents/subagents/registry/subagent-restart-recovery.test-support.ts@c3e18a056eae89d74d568a975842e9f8c448bd5c`
- `src/agents/sessions/session-manager-static-notes.test.ts@30502cd8694e2e4eeb0b2bb9986c7137fb1ffed8`
- `src/commands/onboard-non-interactive/local/auth-choice.setup-replacement.test.ts@084e194c63c278a450595355bc3dc6f0db835073`
- `src/infra/push-apns-store.worker-read.test.ts@9b41a0805094c8d05c659d07ee0b3cb2a46badd7`
- `src/projects/project-environment.test.ts@b7ebb07f4d15426f7654f083203a48c824298805`
- `src/state/openclaw-state-db.windows.test.ts@2be01639a8d4971e80fe3208663433066dce6694`
- `src/state/openclaw-agent-registration.test-support.ts@7b9d9ff76d59a5a96298ac47ff860c7eb1272556`
- `src/agents/subagents/swarm/swarm-scheduler.claim-close.test.ts@37e4dc31a42e21fa5959898a87280a75797937ae`
- `src/state/openclaw-state-read-worker.bun.test-support.ts@56cb545ce563e2eb16a80edece30332092481857`
- `src/gateway/talk/relay/index.test.ts@3e81170284ac02f43771c008cc1308d9fc0de0e5`
- `src/gateway/event-web-push.storage-scope.test.ts@fc17384ddb6df1a475046de59f890f6558575398`
- `src/state/openclaw-state-db-read-admission.test.ts@67e35c387465f743492e6ff1002dbf197b7ba271`
- `src/config/sessions/session-accessor.sqlite-reclamation-worker.test.ts@5d8d81d14161325d3375bf54aee94affe57218cd`
- `src/agents/subagents/registry/subagent-registry-state.async.test.ts@42099257d92b70ee2f487ab37f7ff92465c34380`
- `src/infra/sqlite-writer-admission-service.test.ts@e6dd9c9a644cf710d1cffe8b381ffa5bff3d4da4`

### 19.2 Shared-state async lifecycle production path

`openclaw-state-db-async-lifecycle.ts@fefa1e8c9dffbac0967263c91dbe74ede6d29fb9` and `openclaw-state-db-cache.ts@adcc28173d50c16683e776dcf6bbd056022dd35a` remain the process-wide shared-state lifecycle owner. The frozen source establishes these separate responsibilities rather than a generic close helper:

- **Read admission generation:** `capture` binds a read admission to one physical identity record and generation. `assertCurrent` fails after sealing, identity replacement, generation replacement, or admission replacement. Warm capture may reuse a retained admission only while the same physical record remains current.
- **Worker publication:** `publishOpenClawStateDatabaseWorkerAdmission` binds worker-created storage to the captured database path/physical identity and immediately rechecks the original admission. Publication does not mint fresh authority for a revoked requester and does not publish a host native handle.
- **Async-resource custody:** `registerOpenClawStateDatabaseAsyncResource` adds accepted async owners to the process-wide resource set and to any close attempt already draining. Identity-bound resources settle only for the matching physical database. `phase: after-resources` finalizers are ordered behind ordinary accepted work.
- **Drain versus native retirement:** path/global close seals and invalidates reads first, drains accepted resources, then calls the native retirement callback. Failed ordinary drainage retains failed resources; for a non-superseded database it releases the read seal so service may resume, and the explicit retry seals/invalidate again. A failed native retirement keeps the seal and close custody because the native owner is still live.
- **Replacement fencing:** replacing a physical file invalidates the old generation. If an earlier close has failed, its record stays sealed/custodied even after the logical path publishes a successor. Alias/path equality alone does not transfer the old admission to replacement bytes.
- **Removal exclusion:** offline state removal uses `holdExclusion`, awaits path async close, rechecks maintenance authority, then retains a native removal exclusion until the maintenance owner releases it. A failed close releases only the temporary removal admission; it does not pretend native retirement succeeded.

These wrapper consumers preserve domain-specific owners: web push, device pairing, exec approvals, cron reads, auth-profile observations, plugin-state epochs, user-profile authority, GitHub OAuth inventory, subagent waits, state readers, agent execution, and session reclamation all enter the shared lifecycle through an explicit admission/resource contract rather than becoming second state writers.

### 19.3 Focused negative oracles frozen in this cluster

- `openclaw-state-db-async-lifecycle.test.ts` distinguishes failed resource drainage from failed native close: drainage failure retains the failed resource but can reopen reads, retry reseals; native close failure keeps the seal. It also verifies first-creation targeting, replacement identity fencing, alias normalization, native-reader join, retained after-resource finalizers, close coalescing, and maintenance exclusion.
- `openclaw-state-worker-lease.test.ts` closes fresh callback admission while accepted callbacks/terminal writes finish, drains commands accepted before cold acquisition, cancels queued callbacks without abandoning the actor, and joins accepted native opening before retiring an abandoned actor.
- `cron/store.readonly-worker-cleanup.test.ts` refuses new reader admission while canonical close drains an earlier reader; `trajectory/runtime-retention.test.ts` retains failed cleanup custody and exposes the original failure to lifecycle joiners.
- `test/non-isolated-runner.sqlite-fixtures.ts` preserves a failed native handle/lease across teardown, allows unrelated independent cleanup to proceed, rebinds read pools to a successor generation, and prevents a retired generation from installing an owner against the current cache.
- `openclaw-state-read-worker.test.ts`, corpus-fixture lifetime, worker-environment recovery, web-push storage scope, read-admission, reclamation-worker, and writer-admission-service tests preserve source identity, join accepted child/native work, reject stale descriptors, and keep shutdown/revocation final.
- Project, update, OAuth, device, and session tests in the identity list remain source/oracle evidence only; their negative paths do not turn a successful SQLite operation into proof that cleanup, lease release, or publication authority also succeeded.

### 19.4 First capped `closeOpenClawStateDatabaseAsync` page

The moving-default search returned exactly 100 paths, so the result is explicitly treated as a capped page, not repository-global closure. Three page paths were already pinned before this subsection (`src/state/openclaw-state-db-cache.ts`, `src/state/openclaw-state-db.ts`, and the Round18 `src/cli/update-cli/progress.observation.test.ts` identity). The other 97 paths were individually queried at the frozen ref. Nine moving-default-only hits are absent at the frozen source and are excluded:

- `scripts/perf/plugin-state-polling.mts`
- `scripts/perf/managed-git-maintenance-measure.ts`
- `scripts/perf/session-lifecycle-boot.ts`
- `scripts/perf/database-page-cache-measure.ts`
- `scripts/perf/adoption-candidates-measure.ts`
- `scripts/perf/session-catalog-selection-measure.ts`
- `scripts/perf/managed-attachment-cleanup-measure.mjs`
- `scripts/perf/subagent-maintenance-capture-measure.ts`
- `src/commands/doctor-capture-repeat.test.ts`

That leaves 91 frozen paths on this first page. Eighty-eight were not previously in the dossier before this round; `src/state/openclaw-state-db.windows.test.ts@2be01639a8d4971e80fe3208663433066dce6694` is already listed in §19.1 because it also belongs to the read-admission wrapper set, leaving 87 additional identities below.

Production/tooling/documentation/E2E identities:

- `src/plugin-sdk/sqlite-runtime-testing.ts@2d234e0ac3a3ee7fad62394f6e21bec26ec8584e`
- `src/cli/runtime-cleanup-scope.ts@931bbb9c668e8e80c9286823c2e6ef103c6d8375`
- `src/acp/server.ts@4d63e0c08b9a18eac1825b3ead16cc669e84c8ff`
- `docs/help/testing/fixtures.md@d2670a0fa0ac4f79201b447b96260a8007ec935d`
- `scripts/perf/README.md@c4f41f25a487aa9bca7e188898c6581bb49286c5`
- `scripts/bench-channel-ingress-writer.ts@7fa4f259b0241b6447074adfc130b89bf00fda17`
- `scripts/bench-usage-refresh-memory.ts@f0c4851b146e902982e3a1f5ba83b98d03cd69f8`
- `scripts/bench-session-history.ts@56c8f77d5bcb53a11f7f5407d4838b82890ca7c7`
- `scripts/perf/transcript-hydration-headroom.ts@85b444d8117b4015f65b558d141b06f16a9116f2`
- `scripts/perf/team-shape-fixture.ts@e6b34df3baae232551400c3134eae7ca7f8387ae`
- `scripts/perf/seal-version-restart.ts@9c72a7864495350081cf7eb1dec3c9ff33186e6d`
- `scripts/bench-gateway-startup.ts@913c8760a25cddba01b35d9691cb2bf7ca476ae1`
- `scripts/perf/history-diff-measure.ts@21dcc30a8845af2f2ce769dd64f7f0b82ac864a0`
- `src/infra/update-migrated-finalize.worker.ts@3a700fab13573ccb836b8c437240ae6c8da8edf8`
- `scripts/bench-agent-concurrency-worker.ts@9f826c96de174beac4c959327b976af6fa2a2a75`
- `scripts/perf/session-profile-reference-measure.ts@2417f5f0e45dbf767cb508017347d2e0a432f2a9`
- `scripts/e2e/lib/upgrade-survivor/dreaming-cron.mjs@748b6eaabbcdc268857b6503169bc6ce140d0728`
- `scripts/perf/transcript-broadcast-measure.ts@0903d6cba90b6b1366c98494a5b82aaf4eb55fd1`
- `scripts/perf/worker-placement-reconcile-shutdown-measure.ts@0095614d2c7f0f4e7b87d9658e1073e68c9f686a`

Test/support identities:

- `src/acp/server.startup.test.ts@79cd50262a5f48ddef8493c2d05906aa8c11b6bf`
- `extensions/codex/src/app-server/run-attempt-session-owners.test.ts@62c24d304632c145ea37cf871919809e5e28db4b`
- `src/gateway/server.placement-abandonment.lifecycle.test.ts@bca95735a0d06a2eb403060080c6d2847ba4984f`
- `src/node-host/config.test.ts@8b01a2f1158a4076f3bb0b9f58b2293077345570`
- `src/node-host/local-id.test.ts@98ca5bff8b3c48a1ffa9868c9d0ecd53cb6f0547`
- `src/system-agent/audit.test.ts@1e27c9238eddc758e6062381bdbd0700a702fcbe`
- `src/cli/node-cli/identity.test.ts@cc81bd509309cfe29fcb514e8e536f988cf9f197`
- `src/cli/local-state-owner.test.ts@cf391331f417aa8f275a054107a08144db72f2e9`
- `src/test-utils/database-cleanup.ts@4add142b952b9e414bb5f2b4f55f1e99650fdce5`
- `extensions/feishu/src/dedup.test.ts@d9648c3b0d53c2bed944b13a54bbdd68a4dea59c`
- `src/agents/mcp-oauth-writes.test.ts@1bc09ded77017fda460dcb840d8880caf1b78f4d`
- `src/config/io.runtime-models.test.ts@5758a21d9f60e3ed0e26395699537263b4595c62`
- `src/acp/event-ledger.test-support.ts@47329984e0f2851c96d036bafb22b64d718c5dff`
- `src/agents/mcp-oauth.test-harness.ts@6bb3efbb36219b0743af85c307958229bcf127bd`
- `src/config/io.worker.test-support.ts@9bf5959f3ac6b5cccfc05514e1a63259d41e704a`
- `src/gateway/board-store.test.ts@bc98b31164ffdf8faac7b4584cba9b2fc519369d`
- `src/canvas/widget-tool.test-support.ts@40fc60df77aa691b10299aa2ffc17ffb1085c4f9`
- `src/system-agent/rescue-policy.test.ts@7138f1cd61c67f4c0114b4e0eb6ea0c07db27a8d`
- `src/boards/board-store.test-support.ts@2556cf54c7a603f80f7ad18257ce41669876307e`
- `src/commands/doctor-lint.crabbox.test.ts@9cb877df525fa0f9ebfbf0c4c83d59e3a53288d5`
- `src/gateway/auth-token-store-ref.test.ts@aeffeee1ca24b80513c9eb1209fc43ed5b8c8584`
- `src/claws/provenance-runtime-read.test.ts@a671344f387b6d417744880fe8998f0e3d774806`
- `src/infra/update-run-windows-user.test.ts@6355ba3053f8133e1a93018631b236ea521b82c3`
- `src/infra/sqlite-worker-claw-read.test.ts@7979970ebb9622dbcc95ace5a9726cacb3e82951`
- `extensions/twitch/src/oauth-owner.test.ts@4a49b14a7dc651b86be8b742def05f277af99d36`
- `src/gateway/session-utils.test-support.ts@cd212f0d7f9fd2cba3991883e8fd21bd8c9ab415`
- `extensions/acpx/src/process-lease.test.ts@c6a3da52a01852d9ee93adfc490ac9bee5e77ef2`
- `src/system-agent/transcript-store.test.ts@801d535c392b4b17cb6a9386b13e273565d2ab6e`
- `src/acp/runtime/session-meta-list.test.ts@fa6c496c84531e13a91259b09503907ef797728f`
- `extensions/reef/src/state.test-support.ts@4a5379e1c7b0ff827d62f0e08c2c13782ca7d5b8`
- `src/test-utils/state-database-temp-dirs.ts@7e59ba84ddc3080ce3763283e515c59c2457ac0c`
- `src/infra/device-bootstrap.pruning.test.ts@f866ab8a4f96f06066ffe1f0a5e3a4b807234452`
- `src/agents/live-cache-test-support.test.ts@2e99b290c9dd84a2774bd272c40c14e3c2577bdc`
- `src/cli/update-cli/status.readonly.test.ts@c928d068bf1cfd1a91968af9172da4471091b395`
- `src/agents/mcp-oauth-provider.read.test.ts@4a51687e9ae404d8d0408dae43879150d8380936`
- `src/cli/database-ownership.process.test.ts@ee28c016dd6acb9e88820c32fa541835af9c1dbf`
- `src/infra/update-run-admission-hold.test.ts@f3136576d240c2c8c12e16dc439c85f158212f41`
- `src/commands/doctor-db-bloat.worker.test.ts@dad5b3fc7755438944a59edad67eabfa7432d8c4`
- `extensions/acpx/doctor-contract-api.test.ts@b6988f76f196727d248224c2a68c720b23282556`
- `src/state/openclaw-state-lease-async.test.ts@d51b248bd6ce43eeb46e44d634136837e8636c8c`
- `src/agents/worktrees/run-lease-store.test.ts@ba8dfd88c6413bba1ba9bedbda137a42fd7ccb8e`
- `extensions/google/vertex-adc-refresh.test.ts@f38061a9f284becf9f99b14544d3dfb8bc72cab9`
- `src/acp/runtime/session-meta.test-support.ts@f6411636df8c273572f543d8ab8930fe8416fb78`
- `src/agents/mcp-oauth-refresh-boundary.test.ts@89f0312bd692227d68dac73743485ee839a07d06`
- `test/helpers/stage-live-auth-profiles.test.ts@2891756bd19dea7f73766dd3f37afed4535e3a05`
- `src/cli/runtime-cleanup-scope.workers.test.ts@6c8896bb3ba7034f51485203f7940a7d990f6f84`
- `extensions/diffs/src/test-helpers.ts@2368786f614ef6c81cb092d1473209bc112d40b9`
- `src/infra/device-auth-store.client.test.ts@153b0503430a5493091c918353911cd6a8300afb`
- `extensions/msteams/src/sso-token-store.test.ts@61c6a90fab391a54c6cf12d71631661bebef13b1`
- `src/cron/store/run-finalization.worker.test.ts@7b053d0451dd5a46d744eceeeb17d021ccb774e1`
- `src/node-host/invoke-system-run.socket.test.ts@f7450e117949318e5967855d59483b4b282b34cc`
- `extensions/google-meet/src/oauth-owner.test.ts@e5fffed4e7158e7a8becdca72e67e55e18ae9abf`
- `src/commands/doctor/cron/schema-safety.test.ts@d82ebc128c772daa46c97114e3ed202fa67a7553`
- `src/gateway/openresponses-session-store.test.ts@103fe0a268e22ab1d3deee20753203dfc75e8580`
- `src/infra/update-run-interruption-store.test.ts@f4bff27824e4649e2a6b264de16ee612f4c92f5f`
- `src/state/openclaw-state.worker.imports.test.ts@fd99991ffcbd7e0586aea2545cee1020240ab8f3`
- `src/infra/sqlite-worker-existing-schema.test.ts@c50283c555ee4d2d0131bd0a89809e08d2c153e7`
- `src/state/user-profiles-fixture.test-support.ts@3e62ef20c5cb11010cd403f2da8ddc713a151c6a`
- `src/boards/board-store.fixtures.test-support.ts@1f778f2d810c8199f0d6b67c288ce6665c45d015`
- `extensions/imessage/src/test-support/runtime.ts@25cded6db858b37e9ba184e5c72820c2c7a21198`
- `src/gateway/session-groups.registration.test.ts@90ccf302446c6f3b8576ea9417d174fd2ac1f7e3`
- `src/infra/client-credential-state-owner.test.ts@f398b95f8d8923ff848a499a15e6f3bd6ebac54b`
- `src/snapshot/git-backup-command.test-support.ts@6ab73d8859cce0fcd767af85b21d250d3383bf0d`
- `src/agents/mcp-oauth-requesters.test.ts@211a2ebec9c6e5a08c913c928ac92a975c445a76`
- `src/infra/update-immutable-protection.test.ts@519edd3144543280617259061ab1f55b6a53b0d9`
- `src/agents/mcp-oauth-refresh.test.ts@57d6edd89264389d9d362332ad13fa01db444d16`
- `src/state/openclaw-state-db.index-repair.test.ts@420c6fb2ef8cc0d50b65a37756a4a5ee85147482`
- `src/infra/message-tool-run-outcome-store.test.ts@e1fcb15157bbf17c57f169a004b6aa1fcdc078e3`

The close-page production path has real shipping entrypoints in addition to validation harnesses: CLI process cleanup dynamically imports the async state close, ACP shutdown closes shared state after transport teardown, and update finalization includes async state retirement. The plugin SDK file is a testing export surface; benchmark/perf/E2E files are validation consumers and are not production ownership evidence. The testing fixture documentation explicitly requires awaiting async close before raw SQLite access, copying, hashing, or snapshotting because synchronous test close may merely start worker retirement and leave WAL/native ownership live.

Close-page negative oracles include ACP transport teardown before state close, retained seeded/session ownership when maintenance or agent drain fails, cancelled-startup join before replacement work, state-directory identity separation, fail-closed identity minting, durable OAuth/token replacement across reopen, writer release before delayed retry, lease expiry before effect, accepted callback drain during canonical close, caller cancellation while another lease/HTTP exchange settles, worker join after CLI executable cleanup, Doctor schema/WAL fail-closed behavior, no mutation replay after close, generation-replacement refusal, and prevention of stale refresh/logout resurrection.

### 19.5 Status and remaining frontier

The four direct lifecycle-wrapper discovery sets returned in this round are frozen-ref reconciled, and the first 100-result `closeOpenClawStateDatabaseAsync` page is frozen-ref reconciled with all nine moving-default-only paths excluded. This is not repository-global close-caller closure: GitHub code search is capped and indexed against the moving default branch, so additional production-entry clusters and indirect/dynamic wrappers can remain outside this page.

Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, OCRM final acceptance, and release acceptance remain absent. No Actions, build, test, generator, package, production code, ledger, RTM, or flags were run or modified in this round. The next source-only frontier is to split global-close/lifecycle consumers by production entry and exact-ref reconcile paths beyond the capped page, then close remaining indirect lifecycle listeners/removal/cache wrappers without upgrading implementation or acceptance state from source-only provenance.


## 20. Round81 continuation — gateway capped caller clusters

<!-- Round81 gateway capped-cluster continuation marker -->

### 20.1 `sqlite-worker-operation-admission`: gateway cluster beyond the broad page

Moving-default discovery for `createSqliteWorkerOperationAdmission repo:openclaw/openclaw path:src/gateway` returned 37 paths, below the 100-result cap. Fourteen were already frozen in the cumulative Round87 dossier; the other 23 were individually re-read at frozen `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` and are new source/oracle identities:

- `src/gateway/worker-environments/prepared-pool-presence-store.test.ts@208749210e5bbe6b8ff82f4c503b748bca0bf993`
- `src/gateway/worker-environments/placement-session-tool-operations.worker.test.ts@e0bed91c1631c2a6f5cd0ff79f517052fd481b5f`
- `src/gateway/server.sessions.create.firstbirth.test.ts@c1aa45b1eea4d3f39dabecfa40f7f0746c2c4e21`
- `src/gateway/session-sharing-groups.test.ts@e2cc6f1f46a8b78fb82658449a56e03b143248dd`
- `src/gateway/server-methods/users-preferences.test.ts@98b6354efbfcb5ab5a02687cc1b95c423feb8bfb`
- `src/gateway/server.pending-input-media-custody.test.ts@51fa5ac2d69a5f56bfee86e78cace5e01a5c3281`
- `src/gateway/worker-environments/prepared-environment-store.test.ts@46e3fd870eced6da680eb82a8aa7f7d1d87a053e`
- `src/gateway/session-pr-workspace-adoption.test.ts@b4b4b1fb84519c6f6b087037feaf79f900062c2d`
- `src/gateway/conversation-route-ownership.worker.test.ts@72e3ba95a3ca07e54f45f0a0652852c9d3baf6ac`
- `src/gateway/worker-environments/transcript-commit-store.test.ts@4af4b77815eaa57c4420cd78880eae3b730ea4db`
- `src/gateway/server-methods/users-enumeration.worker.test.ts@395b7a2cb08881c800f15c24aa5d2bed60b17848`
- `src/gateway/talk/relay/agent-consult.registration.test.ts@6a2228fda07c6a9ad317a50fcc21a5d7c4464e62`
- `src/gateway/talk/handlers/client-consult-authority.test.ts@d37b2b44e52449c54fb8c3b1cfd0781086308023`
- `src/gateway/worker-environments/placement-read-cache.worker.test.ts@ad5e8faea2aa391d7f08abfbf6e24c1b6b2a4e63`
- `src/gateway/worker-environments/placement-workspace-journal.test.ts@3eef43664df68eee3b8dc2e2d34226b936c0ec66`
- `src/gateway/talk/handlers/client-create-cross-store-authority.test.ts@3ef2a9b19eda86a4e83bd70bf2e312fa0b11ff92`
- `src/gateway/worker-environments/placement-store.move.test.ts@8d32b917dea7b24ed31e5e8b38c61049ed6c7163`
- `src/gateway/server-methods/agent.visitor-access.test-utils.ts@9701a48b89670e75403c5a8b9767938461b84714`
- `src/gateway/worker-environments/placement-turn-claims.worker.test.ts@91d8d0f434d299063952f2db15ca6419b5b43b48`
- `src/gateway/managed-image-attachments.sqlite-visibility.test.ts@427054bf20a2ecbffaa7869e7e3430cca885b620`
- `src/gateway/server-methods/chat-send-pending-input-withdrawal.test.ts@8bb4629f8866bff348a43d596dc0355ffd01938d`
- `src/gateway/worker-environments/destination-authority.e2e.test.ts@6061f7dc2dfa1c4d5a83421eedb3ef24c23acc76`
- `src/gateway/server-methods/chat-metadata-runtime.personal-accounts.test.ts@27c459378c207118dca4a392553c2153f48933a6`

These are test/oracle callers, not new product owners. The production gateway writers already frozen earlier in Round87 remain the owner path: mention inbox, automatic update cleanup, managed-image record store, prepared-pool presence worker, session-group catalog, placement worker mutation, GitHub publication source, and worker-environment store.

The new negative oracle set materially closes admission semantics. Prepared-pool and prepared-environment writes rollback if host/placement authority changes at transaction or commit. Placement tool operations seal admission before teardown, drain committed receipts, make an indeterminate settlement require restart recovery, and prevent a stale close or delayed uncertain outcome from fencing a reopened same-byte successor. Transcript-commit storage preserves exact tuple/idempotency ordering, rechecks transaction/commit authority, and retains only committed state. PR workspace adoption reports `outcome-unknown` after a native commit rather than undoing the committed binding. Workspace journal and managed-media tests distinguish committed custody from refused commit, unknown settlement, rollback, and post-commit revocation. Pending-input withdrawal keeps the input/adoption available after a refused commit and only aborts/retire-custody after the withdrawal itself commits. Conversation-route and Talk tests prevent platform effects or cross-store voice publication when route/source/caller authority changes before the relevant grant. Destination and placement-move tests bind mutation to exact generation/source and refuse stale or policy-revoked transitions.

### 20.2 Gateway state-close caller clusters: capped page split to non-capped families

The capped gateway search was split by production surface. The returned moving-default candidate counts are `server-methods=32`, `worker-environments=29`, `talk=1`, and `server=3`. Every candidate was checked at the frozen ref. Two moving-default-only paths are absent at the frozen source and are excluded from evidence:

- `src/gateway/server-methods/sessions-create-chat-type.test.ts`
- `src/gateway/server-methods/cron.inference-binding.test-support.ts`

Frozen server-methods identities newly reconciled in this continuation include:

- `src/gateway/server-methods/audit.trim.store.test.ts@38eb7e42caf5c2a54121362cb94d387aed208dff`
- `src/gateway/server-methods/session-goal-request.test.ts@ba1ead40225eeb3a165a268d58a5951bbe618d7e`
- `src/gateway/server-methods/chat-webchat-acp-binding.test.ts@5775ef578798c0800d7f3759bc836e0b6800f5d6`
- `src/gateway/server-methods/worktrees.authorization.test.ts@725a774c0d5c248437ca77e24de11da5bb100be9`
- `src/gateway/server-methods/system-agent-reset-boundary.test.ts@88a5f6734373e60a38d6415cc445f0bd50ce41db`
- `src/gateway/server-methods/skills-upload.test.ts@50f57aa5c6ec1a75fc79cde10cb759aa3230f11b`
- `src/gateway/server-methods/sessions.messages-subscribe-perf.test.ts@b69b6ec86fc7aa092230af5ed84342509e967a22`
- `src/gateway/server-methods/worktrees.test.ts@2ad6c7358d1b59026fd3ba553f565a7edf57960a`
- `src/gateway/server-methods/board.test.ts@7f01288c3908b8bac40368689af0305753a99114`
- `src/gateway/server-methods/sessions-create-category.test.ts@acf45773e843695059b3e6fa002c14e1bec02f8c`
- `src/gateway/server-methods/exec-approvals.test.ts@0c75172a137f0f2c5aa9cd94992d8c9d3e3c535c`
- `src/gateway/server-methods/themes.test.ts@d5cf83a2c191067b816a5ea26d724f2b2377a2f0`
- `src/gateway/server-methods/transcripts.test.ts@f106096c1546b0393ba22bc1a805a7c1be8171cb`
- `src/gateway/server-methods/system-agent.audit-worker.test.ts@2f8990a7480c0ff84b070f7148549b56fc101477`
- `src/gateway/server-methods/approval.test.ts@aa75b413f3f2f50adc4654228f9aad0da6ac78e9`
- `src/gateway/server-methods/system-changes.test.ts@8af820097dfbb8284c098ca135f70e3c170738ff`
- `src/gateway/server-methods/system-agent-setup-concurrent.test.ts@02b1b61818d92261708c9a82d2c29d89b2731af9`
- `src/gateway/server-methods/sessions-files.incognito.test.ts@0cf0a6bbca83e1e9ec78bce0728f663175779706`
- `src/gateway/server-methods/artifacts.request-authority.test.ts@c1092397afb691fedd0abcff2a13cae1299bf96c`
- `src/gateway/server-methods/sessions.initial-transfer-custody.test.ts@e08e3603e0c3562dbfccf90db6be96f23da5fe5b`
- `src/gateway/server-methods/sessions-diff.test.ts@12f1fd74b02f54285a086c529fd9f8a3e0dc9f93`
- `src/gateway/server-methods/sessions.send-yield-resume.test.ts@071fb004fae5e5a54d72e92d85f07399ddb8dc84`
- `src/gateway/server-methods/update-report-run.test.ts@60e7439390470f1136219f97306c68bd742afcba`
- `src/gateway/server-methods/mcp-app.view-grant.test.ts@96852c8834a114d48bdf951d932d6147d7b1ce39`
- `src/gateway/server-methods/board.runtime-boundaries.test.ts@5cb56e4dabeeb698859ed5f33966333d9164bf6e`
- `src/gateway/server-methods/sessions-create-worktree-base.test.ts@dcdb31e8f68749fbeb0686b0ceffb1db0352ac19`
- `src/gateway/server-methods/session-controls.incognito.test.ts@bba61e496256e6626b7bfdf0f6998f5a3b912c6b`
- `src/gateway/server-methods/sessions-rewind.storage.test.ts@ce08c4f7301aeda0f899a6e7ab8519091332187d`
- `src/gateway/server-methods/agent.visitor-access.test-utils.ts@9701a48b89670e75403c5a8b9767938461b84714`
- `src/gateway/server-methods/system-agent-nested-inference.integration.test.ts@46b41178149bfd1fdd9ea11df4a4e261d96a9f19`

Frozen worker-environment identities newly reconciled include:

- `src/gateway/worker-environments/prepared-pool-presence-store.test.ts@208749210e5bbe6b8ff82f4c503b748bca0bf993`
- `src/gateway/worker-environments/store.test.ts@372e28939af3ed8b2d78bf0dfe12a6fc9cc60bea`
- `src/gateway/worker-environments/service-inventory-recovery.test.ts@79806d4aed5c98a371df8aa6ba708c08e9711721`
- `src/gateway/worker-environments/store-retention.test.ts@6d756c6507a5c85a4ddc7e4a47026292583298e5`
- `src/gateway/worker-environments/bundle.test.ts@f1c08a870f2c3d21509d164409aed1afb6ce707e`
- `src/gateway/worker-environments/placement-change-snapshot.test.ts@76a109464c6ad96010b2238900eb7ea19c723e85`
- `src/gateway/worker-environments/placement-session-read.test.ts@25a0bb2f437576c6ae082128fad732f7c310fc00`
- `src/gateway/worker-environments/store-worker.test.ts@9d9aa15ffbd40e21962a15b1b8d860225dd3018a`
- `src/gateway/worker-environments/local-workspace-incremental.test-support.ts@2d7fba74547c7b835de3357d54ebf256b9476e8b`
- `src/gateway/worker-environments/prepared-pool.test-support.ts@a7955c719f93856b733d5cef00f93a81eb10d075`
- `src/gateway/worker-environments/store-recovery.worker.test.ts@f392a257d68a1b3cd918d680971ef8d97ba79c38`
- `src/gateway/worker-environments/store-runtime-refresh.test.ts@9102f4f52931e384ab3058f4bce333dfdcdca691`
- `src/gateway/worker-environments/inference-store.test.ts@2da2b6ac97710f4759a794db96f708404194706b`
- `src/gateway/worker-environments/workspace-result-ref-mutation.test.ts@f5e3f59953814bc8c5c01aa80e745f140f47039b`
- `src/gateway/worker-environments/local-workspace-projection.test.ts@f8775b34e79ce43e1ea75e9fd98c0c0139dad82f`
- `src/gateway/worker-environments/service.test-support.ts@e559e40dd481c1f9247bccdac4380aa861941b03`
- `src/gateway/worker-environments/node-enrollment.test.ts@e6ed8536fd9ab0d45f91f7bce2c6fb131d0bd65d`
- `src/gateway/worker-environments/store-node-enrollment.test.ts@65bb5e51adcde97d0a6ea18b880f1a483eaabe92`
- `src/gateway/worker-environments/placement-read-cache.worker.test.ts@ad5e8faea2aa391d7f08abfbf6e24c1b6b2a4e63`
- `src/gateway/worker-environments/local-workspace-generated-paths.test-support.ts@09f59bc07042f234985bb10b54df18242ebd52a2`
- `src/gateway/worker-environments/environment-access.test.ts@5da412dfd2d3b8cad267dacd334afec127b82c38`
- `src/gateway/worker-environments/placement-read-projection.worker.test.ts@a17ab5f9c2fda6ce09ca801590e853a14a503924`
- `src/gateway/worker-environments/local-workspace-store.test.ts@a2fe8026eb0656a2e714f2267bf8ec97ebbc686e`
- `src/gateway/worker-environments/placement-move-abandon.test.ts@eb8f611681b33c8ee93b6cd07f820db0b6df1d5e`
- `src/gateway/worker-environments/workspace-result-repository.test-support.ts@b6b0ea54f83d830b130f4342a723cd34f66a9423`
- `src/gateway/worker-environments/placement-dispatch-authority.test.ts@1985e27e8201c07911428e0009b42aa4e46dfc7d`
- `src/gateway/worker-environments/worker-turn-launcher.test-support.ts@9b6977a86c46576cceb286ecf3e1872f11d4352e`
- `src/gateway/worker-environments/node-workspace-transfer-revocation.test.ts@cfa33c7126f6af8fa5c677b2ac3745b66664d6f1`
- `src/gateway/worker-environments/placement-dispatch-staged-results.test.ts@bd695a7789e1e95590a6bb2b7e43d908320b3894`

The remaining state-close gateway split identities are `src/gateway/talk/session-registry.test.ts@c8b1908b5fc0ef5ab0f4b8edef32b9da725b80e1`, `src/gateway/server/ws-connection.startup.test.ts@2ad61ffb0b66184fb3a4708f2c995bd2bcab00e5`, `src/gateway/server/ws-connection/message-handler.worker.test.ts@196a5c3f0a36d3ec9f586557fc90d445bd162e1c`, and `src/gateway/server/ws-connection/update-report-authority.server.test.ts@3b6580f26ec8812fabb162fdedd7493a85571793`.

These tests preserve the close contract instead of making close itself a business owner: read admission becomes invalid across close/replacement; accepted writes, callbacks, transport/authentication setup, workspace transfer, and result publication settle before teardown; failed or unknown native/worker outcomes retain recovery custody; and stale generation, config, role, connection, request, or source authority cannot be revived by reopening a cache.

### 20.3 Gateway agent-database close clusters

The broad gateway discovery for `closeOpenClawAgentDatabases` was likewise split below the cap: `server-methods=37`, `worker-environments=9`, `talk=5`, `server=1`. Re-reads confirmed the previously listed overlap plus these additional frozen identities:

- `src/gateway/server-methods/environments.session.test.ts@98e34d67fd78e5d71deb0d0e4cae2eb5fe26281e`
- `src/gateway/server-methods/sessions-suggestions.test-support.ts@9e37b62d6ae9ed53c831f71f1906534d8bbce30c`
- `src/gateway/server-methods/sessions-public-share.test.ts@e4d8ca755e9081a9203515ecef225429e7dfb952`
- `src/gateway/server-methods/sessions-read-fixture-lifetime.test.ts@4e756675280e49f1ce8d402013b372f9ca6aec04`
- `src/gateway/server-methods/sessions-read-metadata.test.ts@012b872a8d0080b5335bf6c8db67336c6a88a6e6`
- `src/gateway/server-methods/task-suggestions.owner-recovery.test.ts@f12c65e645ef54ad9cc0fdac0d8a362f6313d8ae`
- `src/gateway/server-methods/sessions.abort-durability.test.ts@d832fca26f98f0a43f6602b006891f526d1076db`
- `src/gateway/server-methods/chat-history-ownership.test.ts@fba84c428fe37610c6d8853e569f5dc0f948782a`
- `src/gateway/server-methods/sessions-abort.test.ts@8102c26babcc6711220706a07ba166a9f41f8398`
- `src/gateway/server-methods/sessions-mutations.perf.test.ts@095cea2618c29fb51c3dec5e31e30462c64bc00b`
- `src/gateway/server-methods/task-suggestions-session-start.test.ts@c3ed225919b9091b40e5db2be9a49f78f03e668d`
- `src/gateway/server-methods/chat-history-delta.test.ts@8ffdb1c7492c9e09231906c6696ea0cd56839f53`
- `src/gateway/server-methods/chat.abort-live-proof.test.ts@cdd8d788eb98b1619425993b04f5c145c3b52efb`
- `src/gateway/server-methods/approval.handlers.test-support.ts@4350cdccec752ed5e40aa071cef2fc1c706a0719`
- `src/gateway/server-methods/chat-restart-recovery.worker.test.ts@9ad0e62447bed47b0591656ca983750d6ba5f050`
- `src/gateway/server-methods/chat-send-reply-session.test.ts@4608b20fc8953c3816caa5d4b4e2fd5c9bb826c2`
- `src/gateway/server-methods/sessions-search.test.ts@3fb2cfe103994c422d3980adfacfd25ba432abbc`
- `src/gateway/server-methods/sessions-typing.test.ts@104d4e6ac15495c6f6645c68e18188dd21f12c7f`
- `src/gateway/server-methods/models-session.test.ts@5f5cc82c5b275680d740fe89d214a90af40924eb`
- `src/gateway/server-methods/sessions-sharing.patch-worker.test.ts@61403471db58f75e682e8ca1f30c32c3538302c8`
- `src/gateway/server-methods/task-suggestions.test.ts@41c6e7538a1ed4a5646f381aea41595af60713d7`
- `src/gateway/server-methods/sessions-sharing.test.ts@7a421543b7d44d64d3a3f8270ef669609272acce`
- `src/gateway/server-methods/sessions-rewind.test.ts@33ae5ded6db8d22e368a1dcc2173054e01fda41d`
- `src/gateway/server-methods/chat-send-admission-authority.test.ts@edbad03b9b8435e1aa3c87807e8636f8b8c95ae2`
- `src/gateway/server-methods/sessions-read-visibility.test.ts@9301f1162e06969552ef440e94b80b589973a17d`
- `src/gateway/server-methods/chat-metadata-session-writes.test.ts@66c85f1123f5aaa983d599092ef6163ac28de5b7`
- `src/gateway/server-methods/chat-send-global-identity.test.ts@2244a525a8bdbc450a3454e23abc154904578062`
- `src/gateway/server-methods/requester-cron-authority.test-support.ts@35aec092c6a94600c32236b16c4f4fa3f9042d0e`
- `src/gateway/server-methods/sessions-files.repository.test.ts@f5e6f19b9927412c7f062fef8526cd76575ede3d`
- `src/gateway/server-methods/sessions-mutations.sticky-model.test.ts@e59b5ed8cf8e0764a7ba11cbf87a20b32ce105c8`
- `src/gateway/server-methods/sessions-mutations.catalog-queue.test.ts@a2e9b3b69f0fe3846f62e8425c107a67edd814b5`
- `src/gateway/server-methods/sessions-mutations.archive-attribution.test.ts@2d213eb0cc13a0f4936669a77cd9ab6e80c9c594`
- `src/gateway/worker-environments/provider-provisioning-node.test.ts@cc1f319082bbdf73e38aa5f2591588be738b40a1`
- `src/gateway/worker-environments/session-attachment-service.test.ts@fd8144b4571a0f17702a44be7e3f04d7e3b9bac0`
- `src/gateway/worker-environments/live-events.media.test.ts@23d9de654b91da327cfd9a8c16139a6cbdc36399`
- `src/gateway/worker-environments/transcript-commit.fixture.test-support.ts@c4020d5319c57a441976fa0ec734d47c3db3e535`
- `src/gateway/worker-environments/device-placement-demand.test.ts@81fbcb3afde4e6d987f5b4c406710f1020f30df3`
- `src/gateway/worker-environments/live-events-settlement.test.ts@e762963cab2ef0672315880c2515613950aaad11`
- `src/gateway/worker-environments/live-events.test.ts@da268393f94434bd52808dad15abb2564a8efbf6`
- `src/gateway/worker-environments/transcript-commit.test.ts@93e8e002d54756bf8ab59d31c956ca981cd66dd4`
- `src/gateway/talk/handlers/client-create-source-authority.test-support.ts@909099345bcfadfc2fd462ffaca1c6fc0ac77556`
- `src/gateway/talk/handlers/client-create-authority.test.ts@bf797ae1a86d5c86390592f9885c4dc24074dc17`
- `src/gateway/talk/relay/agent-consult.registration.test.ts@6a2228fda07c6a9ad317a50fcc21a5d7c4464e62`
- `src/gateway/talk/handlers/client-consult-authority.test.ts@d37b2b44e52449c54fb8c3b1cfd0781086308023`
- `src/gateway/talk/client-gateway-control.agent-consult.test.ts@cf23bb44196b357a57d8a8d3bddf7f82d275260d`
- `src/gateway/server/__tests__/server.sessions.reclamation-admission.test.ts@7b320996686fdef3b7d60fc25f7ed1153e1a9670`

The same missing `cron.inference-binding.test-support.ts` moving-default path is excluded here as well. The agent-close negative oracles keep native close separate from logical session/run settlement: active writes and accepted events join before close, restart recovery does not tombstone successor/source facts, stale public-share/session generations are rejected, late abort cannot undo an already committed terminal reply, and caller/source/role/config replacement must be revalidated before mutation or presentation.

### 20.4 Status after gateway cluster split

This continuation removes the 100-result ambiguity for the gateway subfamilies above and records the exact frozen identities and negative oracles. It does **not** establish repository-global closure: `extensions` remains a capped family for state close (160 moving-default hits) and must be split by extension family; additional indirect/dynamic lifecycle wrappers outside the searched symbols can still exist. Direct `withSqliteSnapshotSource` discovery under `src/gateway` and `extensions` returned zero, so no gateway/extension snapshot-source caller is inferred from those queries.

Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, OCRM final acceptance, and release acceptance remain absent. No Actions, build, test, generator, package, production code, ledger, RTM, or flags were run or modified. This remains source/oracle provenance only.

## 21. Round81 continuation — extension page-2 and snapshot-source closure

This section appends only evidence absent from the live Round87 dossier at branch head 343f5235216bff71bb1172598974d602a6dd793a. It does not repeat the gateway cluster identities added by the concurrent Round81 writer. All source identities below were re-read at frozen upstream openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70.

### 21.1 Shipping close owner and exact shutdown order

New production identities absent from the prior dossier:
- src/gateway/server-close.ts@7d579c6f40580d2d1f90d3910bea7d969fffac60
- src/shared/global-singleton.ts@92c598b9c0e835ac80c39b48c55f97a062e6929
- src/gateway/server-close.agent-databases.test.ts@d7f1c8468451d65ba2e7c5117ad9ceb68840d
- src/gateway/server-plugin-reload.recovery.test.ts@3b2d747bf5ff73a48acfa47e753ae2c35404ef69

server-close.ts first drains accepted Gateway, channel, runner, transport, provider, socket, request, transcript-worker, plugin and cleanup work. Its terminal retirement sequence then closes agent databases with closeOpenClawAgentDatabasesAsync and only afterward calls drainGlobalSingletonLifecycleState. The already-anchored state DB cache registers the state database under the shared-state singleton lifecycle, so the later singleton drain is the shipping shared-state close. This proves the product ordering directly: agent DB native close and durable lease release occur while shared state is still available; process-wide shared state retires afterward. Close errors are accumulated and surfaced, not silently collapsed into success.

server-close.agent-databases.test.ts proves accepted work joins before actor/database retirement and that new work is refused while the root drains. server-plugin-reload.recovery.test.ts proves a stale lifecycle getter cannot mutate a replacement scheduler. This is production owner evidence; extension direct-close imports below remain test cleanup/oracle evidence.

### 21.2 Extension capped callers exact-ref reconciled by family

The moving-default closeOpenClawStateDatabaseAsync extension query returned 160 results and later 163 while the audit was running. That drift is why search remained candidate discovery only. The broad set was split by extension family below the 100-result cap and each positive path was exact-ref read. No extension production module was found directly owning process-wide state/agent DB close.

Codex exact identities:
- extensions/codex/src/session-catalog-cold-native.test.ts@d1f1c7c4a397e603f33341490be729bab6d5ceef
- extensions/codex/src/app-server/run-attempt.reasoning-effort.test.ts@be05d1485870c93ccb9921df87d504ee463c3473
- extensions/codex/src/session-catalog-ephemeral.test.ts@46dd707d2630fe68c1d6656770bf84783bfd4172
- extensions/codex/index.test.ts@fe825a0b262a61e8ab53d56f2324bf278c963064
- extensions/codex/src/app-server/run-attempt-session-owners.test-support.ts@7b80baef553aa67fe26ce8da4eb6010c1a9d5f2f
- extensions/codex/src/app-server/dynamic-tool-build.test-support.ts@57f50d8fbb854171e36dca2d249cdf9c190bfba1
- extensions/codex/src/app-server/thread-lifecycle.native.test.ts@4072bdff8c267ae69d1924881af9d6f2c4628090
- extensions/codex/src/commands.test-support.ts@29b216181090a5b8ddeb7e3be5eea847137fb97a
- extensions/codex/src/session-catalog-resident-state.test.ts@31cf26106b9e60fd62fbcf08cf36be957567fbb5
- extensions/codex/src/migration/provider.test-support.ts@1b38b06093dafa6be523fe27258bd7d189eab7d5
- extensions/codex/doctor-contract-api.test.ts@cd20cfcbe6c5a4865ed019fcc20a191330ad2d1e
- extensions/codex/src/app-server/session-binding.test.ts@fbb707403ba16acf836c920cd5266a39f3b396b8
- extensions/codex/src/session-catalog-adoption.test.ts@445279dbc8a2c8a1fee85861122479c0f7434dbc
- extensions/codex/src/session-catalog.test-helpers.ts@9a3f63c30cda494fa5525a880dd149c5f836ccc4
- extensions/codex/src/app-server/upstream-session-fork.test.ts@cba794860ab22a2f3b9b74b5a77b35405738aef5
- extensions/codex/src/app-server/session-binding.reads.test.ts@dc290cd57fedef6965634c2b10c65968e79b547f
- extensions/codex/doctor-contract-api.native-assignments.test.ts@f1b16a56aaa02ae5def67e3970e227d476057057
- extensions/codex/src/app-server/native-subagent-submission-store.test.ts@c08015f4104120a0eadcd38188d694dd0842bd83
- extensions/codex/src/app-server/thread-lifecycle.warm-cleanup.native.test.ts@7d25c11bfbb2387f183c0e255e828ef8becffc85
- extensions/codex/src/native-thread-tool.test-helpers.ts@98dad6a5f57a4a36b95dd745f6d0f098295f2256
- extensions/codex/src/app-server/transcript-mirror.test-harness.ts@29e1bde685c6cc0eeecbab2acb608d9810e7cd0b
- extensions/codex/src/app-server/session-history.test-support.ts@55c7187b61bd5e1e03541eab3b74d7765c349fdc
- extensions/codex/src/app-server/event-projector.test-harness.ts@2ac58cb8ccc1832f34e234038e70075dad5a5b03
- extensions/codex/src/native-thread-tool-selection.test.ts@20bb85e234ad8a46ab19f0762c78dc51e10734c5
- extensions/codex/src/app-server/session-warnings.test.ts@5f707cada8f357f975abe6ca979d74b7bb6d26e6
- extensions/codex/src/app-server/native-execution-policy.authority-chain.test.ts@14f52efccaaa85806606310add9e2729c70f6490
- extensions/codex/src/app-server/upstream-fork-import.test.ts@47182bbe56b1f8d55d0265e006d83fd680a1844a
- extensions/codex/src/conversation-binding.test.ts@89839aead6ecf2a72feb8fff5a9118be211c0f68

Moving-default-only Codex exclusion: extensions/codex/src/app-server/reasoning-effort.native.test.ts was NOT_FOUND at the frozen commit.

memory-core exact identities:
- extensions/memory-core/doctor-contract-api.test-support.ts@a9278eb62da2c6b71b9c9ccc27b6b212e638a313
- extensions/memory-core/src/memory-search-operation.test.ts@310b0dc2375d7e92f94acf629f0308db89910f42
- extensions/memory-core/src/memory/manager.unicode-query.test.ts@4a8c34edf91726d381be732eef04b788c83a5d0a
- extensions/memory-core/src/test-helpers.ts@5878a057aceb86d59b6683b6c1d3e74d5430c161
- extensions/memory-core/src/dreaming-state.test.ts@69b46797546e5662b84fafa8e9edc9561967d291
- extensions/memory-core/src/cli.test.ts@dcd7458521d2a5c7d0c5d1ae8921faadc7b46726
- extensions/memory-core/src/standing-intents.test.ts@fe7c92902bd5d801c4ff61fc26e4045214503d11
- extensions/memory-core/src/memory-mutation-authority.test.ts@56394bf21d80e6029d1d3bc12b777bb40857960e
- extensions/memory-core/src/memory-entry-origins.test.ts@25c6cff5f240710c6d28ce17bd6f6c4712b813be
- extensions/memory-core/src/chunking-upgrade-real-transport.test.ts@ef2d48a2a035cf839c802cbedcd7c9965b22574f
- extensions/memory-core/src/standing-intents.async.test.ts@13e891b9ec50df9105155cedd0efa7cb6262b92b
- extensions/memory-core/src/memory/manager.fts-only-reindex.test.ts@60503c0e50661f62ec4e664030bb576026c8a47c
- extensions/memory-core/src/memory/manager.reindex-recovery.test.ts@fff81b380cc6afbe89f84ef8726ed2374aeac2ad
- extensions/memory-core/src/memory/manager-search-upgrade.test.ts@0bc3f6215dbd392721ecc327ef0a6fc1009f6d70
- extensions/memory-core/src/memory-forget.sources.test.ts@4aa2b95ab84edd42ec18911aa864b54a10528acf
- extensions/memory-core/src/tools.index-upgrade.test.ts@a3386af074318b5ad08baa646843b2e9432ead10
- extensions/memory-core/src/memory/manager-compatible-index.test-support.ts@187764b89fbf162365d714cabb903026ef1154bb
- extensions/memory-core/src/tools.real-manager.test.ts@9c89f1cbc4188418e28512d110e45b64803b540b
- extensions/memory-core/src/session-backfill.test.ts@b42a2788ff9364bd523351ce4ec94e8c5aa89d98
- extensions/memory-core/src/memory/manager-embedding-cache-dimensions.integration.test.ts@5cd5b8ad1e8da40817f6593d9323f7f1e322ca84
- extensions/memory-core/src/memory/manager-publication-admission.test.ts@0267dd2742b03d474483aef99739d24ccd22d3bd
- extensions/memory-core/src/memory-forget-consolidation.test.ts@20c4d5216d9e4e0d486d3db5e1d5de53e1357c16
- extensions/memory-core/src/memory/index.test.ts@238264c6ad174c34d2dab1455a9c7822cb005373
- extensions/memory-core/src/memory/manager-publication-settlement.test.ts@faad70b9ae6e3dee07df60685ccd24ec8710477e
- extensions/memory-core/src/memory/manager.borrowed-connection.test.ts@f7796dea6cfcb83ae74b3ee0b3d2b0c7268c83cf

Matrix exact identities:
- extensions/matrix/src/approval-config.storage.test.ts@108376672e42cd90c71a5f99c2c5c4d53586f017
- extensions/matrix/src/matrix/account-state.postgres.test.ts@202a2e32af8430f3aaa3dc0931844ed4c4f93757
- extensions/matrix/src/test-runtime.ts@326a990f8e10eb13309ed39a0ced0d528a1e5380
- extensions/matrix/src/matrix/client/sync-store.test-support.ts@9bc15ca6ae9a5350197fef1203169404f2951801
- extensions/matrix/src/matrix/send.authority.integration.test.ts@c7df7b0caab0519ba179032d4dee8461676d010b
- extensions/matrix/src/matrix/client/create-client.storage.test.ts@c2c1f08322298ba86ff0af3e08c352078a9edcd5
- extensions/matrix/src/matrix/monitor/handler.restart.test.ts@777ceb7c16527c4947613ecb935f1ff52dbdb7da
- extensions/matrix/src/matrix/monitor/index.credentials.test.ts@4b0ba4b7f382b79f6a377cd25bc3aa75f33b1e89
- extensions/matrix/src/matrix/sdk/idb-persistence.test.ts@016ee7a586258b5582de9e0b4a4591243f1d3167
- extensions/matrix/src/direct-encryption.test.ts@6401c118ab8dc2089ebd787601f2598dbb3987b5
- extensions/matrix/doctor-contract-api.import.test.ts@a36380d03f1d08138f06a7890e78fa83c00a314d
- extensions/matrix/src/matrix/credentials.test.ts@0ea1a107f0bda3790374f9805c86016b7e702fe1
- extensions/matrix/src/matrix/sdk/client-discovery-recovery.test.ts@6028acb9bafc12e3d94ca0fa3f66a73afd0ccba1
- extensions/matrix/doctor-contract-api.test.ts@13c2368e5e0bfd68079301bda8d7ed904ab911a2
- extensions/matrix/src/matrix/client/storage.test.ts@60793120f45d52b63245535fa5de9a37f0fd08b5
- extensions/matrix/src/matrix/crypto-unsafe-state-doctor.test.ts@88074301f475c75448ba8f5a2782b2871a3e1d04
- extensions/matrix/src/matrix/sdk.test.ts@0d57b822b34d689cad14b76403197c71298a55c2
- extensions/matrix/src/matrix/client/file-sync-store.test.ts@f14014482f0144e1914335903c42285ceaef36e2
- extensions/matrix/src/matrix/conversation-route-owner.test.ts@8bbd96e43129b9ab3d308aae229eca1c843f9bc9
- extensions/matrix/src/matrix/sdk/client-real-snapshot-custody.test.ts@bcea02f581e9d3fac624be385e4a4cb5e1a5a630
- extensions/matrix/src/matrix/sdk/recovery-key-store.test.ts@a2988556cdd75415c785b66952302a1b2c7e01c7
- extensions/matrix/src/session-route.test.ts@16728785143a5588f80e1401ba5adcb941cddf1c
- extensions/matrix/src/exec-approvals.test.ts@a127eefe534fec21ca002af4b5d7c7c5a986f831

Microsoft Teams exact identities:
- extensions/msteams/src/token-refresh-boundary.test.ts@e6a91c2dc1104bb72894313fa8589a2a691aeb86
- extensions/msteams/src/token.test.ts@13e583cfd61b70d01e0cf5c73c4d6a5bb56d38b4
- extensions/msteams/src/sent-message-cache.state.test.ts@e0a20e32666a44a90b1ecdeaf25d6a4d1cf6a017
- extensions/msteams/src/conversation-store.shared.test.ts@8c68b5472e43b2a8213d59095c390532579bacad
- extensions/msteams/src/pending-uploads-fs.test.ts@7f21502349e229570f0a0a46374fcb734c2e07c3
- extensions/msteams/src/polls.test.ts@7f28558f92b66708064f243c9a4445be5edd94a4
- extensions/msteams/src/conversation-store-state.test.ts@ac8ff99591ab22e33e076d73910101694b59ff05
- extensions/msteams/src/msteams-ingress.test.ts@6a4034179eeef55d8dfc02d760e4a09926b4507b
- extensions/msteams/src/monitor-sso.ingress.test.ts@f8f639a852ee52319d4a8df6f725ace39289785a
- extensions/msteams/doctor-contract-api.test.ts@3f29edb66c066a183343606add421314a5157758
- extensions/msteams/src/monitor-handler/message-handler.ingress-lifecycle.test.ts@e955d7d52144bc43bce7e741e65c15a1bc6b6dd6

Telegram exact identities:
- extensions/telegram/src/bot.callback-retry.test.ts@2095d968ce7b42f467ab3841e4a43be125fada03
- extensions/telegram/src/monitor.webhook-identity.test.ts@5a7ff2347a0f0e4859d7142959ad1b0cddebe548
- extensions/telegram/src/telegram-ingress-spool.test.ts@e86e3611e9131b1f5af4d5a9dd78d1004152221b
- extensions/telegram/src/send.history.test.ts@4f4f9afe94500f70bfeda4353303a7d69217437b
- extensions/telegram/src/bot.test.ts@5bca5184a06682a0d72856c2a37b8e7242ae364b
- extensions/telegram/src/polling-session.test.ts@cad1d8c4caf8e938d40ad7a1b06e65daa9f1bfb9
- extensions/telegram/src/webhook.test.ts@8025ba7ac60784cb14a85174ab22e104be334232
- extensions/telegram/src/bot.media.e2e.test-harness.ts@baeb0b533d47bf59ac8f01cb4e791c0f316fc22f
- extensions/telegram/src/telegram-ingress-drain.test.ts@6eaa9d8885163c6e3c371110880094fdfa02a385
- extensions/telegram/src/bot-message-context.dm-session.test.ts@25c69c4c1b21b2322ed1f759386cb8a6fe79e3e0
- extensions/telegram/src/bot.create-telegram-bot.test.ts@12e87146785aa4d7c39555292c4606664026edc5
- extensions/telegram/src/telegram-ingress-coalescing-fixture.test-support.ts@93a0d6dbc1e77c648ea2420188cbb6c457462e90

These family oracles are consistent: old borrower/finalizer release cannot close a replacement; terminal revocation requires re-admission; lost final seal/guard and unknown commit outcome fail closed; queued maintenance cannot reopen retired source; committed prune or mutation reply loss is not replay authority; SDK/sync/credential generations cannot be deleted or resurrected by stale owners.

### 21.3 Extension state-close page-2 tail outside the core families

Additional exact identities beyond the former first-100 page:
- extensions/memory-wiki/doctor-contract-api.test.ts@4bb640c56807a480da821868a4a7339c2f594d4d
- extensions/feishu/src/monitor.reaction.test.ts@7eddee3cd0d25ea764b6ab7f21d8cde8eb061c1a
- extensions/imessage/src/monitor/catchup.test-support.ts@6dd10e8a351abf0bf47943b6d48fac3c72a00f61
- extensions/agentsapi/agentsapi-harness.persistence.test-helpers.ts@f5cd356ca7d37b336b462ea5e2463efd9afedd95
- extensions/slack/src/monitor.test-helpers.ts@4e21721cd694d28258c72b39832a0b33d894eddc
- extensions/migrate-hermes/secrets.test.ts@a7be6d4f342834373fd10af3009944b3c3351996
- extensions/zalo/src/test-support/monitor-mocks-test-support.ts@3d73b586a71b376504020a66c4d8cdc64d2044c1
- extensions/active-memory/index.test.ts@b74453be4b0769daeeb298493c52e76ea3586fe7
- extensions/qa-lab/src/runtime-parity.test.ts@d4cf45a9f6497819e9666b55307eac8b8f7cb513
- extensions/voice-call/src/manager/store.test.ts@00cbfc1349f21fe93b1ee49875e7c6ae62b2c29b
- extensions/migrate-hermes/files-and-skills.test.ts@eaa3a7222d9157e5e89794b43690b73b2e0e027b
- extensions/synology-chat/src/outbound-media.test.ts@a677f10822ef89262bcbbc21962a84dfd852bd36
- extensions/qa-lab/src/live-transports/matrix/scenarios/scenario-runtime-state-files.test.ts@8be42ee9f24bff6e082e83a4cbbaedd2ee4eecb6
- extensions/crabbox/src/crabbox-worker-warm-image.test-support.ts@16bda6158d6bbb1e6a25039f4b4bb179ffd829fb
- extensions/imessage/src/monitor/catchup-startup.test.ts@3d723bafb92e03c38be39e8cfe7abb5e67a5d9b4
- extensions/zalouser/src/zalo-js.credentials.test.ts@1f44050e1fe98ba0d86d404e4a1a68392d7fef41
- extensions/buzz/src/gateway.cold-start-recovery.test.ts@3234123e959cae88b1761753c64f1f36385b930c
- extensions/nextcloud-talk/src/inbound.behavior.test.ts@75833c8e0e94c0c85326f757ea7012cd4f1a33f1
- extensions/voice-call/src/manager.async-store.test.ts@e3d1a853fc044aebf1abd08baeac2c4e45d57471
- extensions/feishu/src/test-support/lifecycle-test-support.ts@5d60acbae519cec963657594cbfa91e6fad4952d
- extensions/feishu/src/monitor.vc-meeting-invited-handler.test.ts@597abc244a9e1ad73eeab73d5ca4b5fe09d3325f
- extensions/memory-wiki/src/compiled-cache.integration.test.ts@7b3db4fc008931f21b80d4a2e502a730e9e27211
- extensions/feishu/src/monitor.message-handler.ingress.test.ts@7516a634c07c1333185a901922202a5f946919dd
- extensions/voice-call/src/webhook.hangup-once.lifecycle.test.ts@ffc8ecc016201a315c43fca222a18416be8ae9d4
- extensions/signal/src/monitor/event-handler.reply-session-conflict.test.ts@25088991d8b08b42aecacafcd7c610e222f9f2ac
- extensions/clickclack/src/discussions/service-state-persistence.test.ts@9b5527419a1207564b457a9388bc4a79ddd1b48d
- extensions/crabbox/src/crabbox-worker-warm-image-maintenance.test.ts@dc5e22a5f1e7290cbaa6a5bda1ddd65e8f34de73
- extensions/mattermost/src/mattermost/monitor-posts.thread-history.test.ts@8f0d2bf75c68bfad71ea3dd2b374a12fb5f5b881
- extensions/crabbox/src/crabbox-worker-warm-image-allocation.test.ts@89af0fe6b67238f2605fb960287d56c2e7f3c811

These are test/test-support direct close callers, not production close owners. New negative oracles include unknown settlement refusing authority restoration, old finalization preserving a successor, aborted ingress preventing late dispatch, uncertain capture remaining fenced after restart, and old borrower retention until exact release.

### 21.4 sqlite-snapshot-source cap closure and production chain

Frozen source: src/infra/sqlite-snapshot-source.ts@14931e709dbccb2daab535c15201dfc671994904.

Export-level moving-default candidate counts were split as follows: openSqliteReadOnlyDatabase=22, prepareSqliteReadOnlyLocation=101, startSqliteReadOnlyLocationAsync=22, prepareSqliteReadOnlyLocationAwaited=4, forkSqlitePrivateSnapshot=11, prepareSqliteReadOnlyLocationSync=64, withSqliteSnapshotSource=2, readSqliteSourceContentVersionSync=6. Thus the only actual 100-result overflow for this module was prepareSqliteReadOnlyLocation; its unique page-2 candidate is now frozen as src/cli/update-cli/update-command-terminal-outcome.test.ts@8f91387479cafce091df40b22a24422809999622.

Previously anchored production consumers were re-read rather than duplicated in this dossier: openclaw-state-read-snapshot-source, openclaw-state-read-operation, openclaw-database-preflight, sqlite-snapshot, sqlite-maintenance-inspection, state-ownership-preflight, and plugin-sdk/sqlite-runtime. The re-read confirms one retained snapshot-preparation owner across native reader/worker close and cleanup. Artifact-preserving reads capture expected physical identity before publishing private bytes. Awaited/private-fork paths recheck cancellation and current authority after native work returns and before adoption. Cleanup failures remain first-class and can aggregate with the triggering error. withSqliteSnapshotSource retries only after discovering a rollback-journal-backed source and then operates on the prepared immutable copy; cleanup remains in finally. A retired copy child is recovered with a new scoped worker plus explicit staging reconcile/removal rather than abandoned custody.

### 21.5 Status

The requested broad capped caller frontier is now materially narrower: gateway capped pages have been split and reconciled by the concurrent Round81 section 20, extension close callers have now been split and exact-ref reconciled through their hidden page-2 tail, and sqlite-snapshot-source has been decomposed by export with its sole 101st caller frozen. No new production DB-close owner emerged; the shipping owner proof converges on Gateway shutdown, agent DB retirement, and later shared-state singleton drain.

Independent ACCEPT, scoped G01/G02 positive evidence, unit_authority_ready, executable/package proof, OCRM final acceptance, and release acceptance remain absent. No Actions, build, test, generator, package, production code, ledger, RTM, or flags were run or modified. This remains source/oracle provenance only.


## 22. Round82 continuation — extension state-close family closure

This addendum records only evidence absent from the full live Round87 dossier at the lease re-read immediately before this write. Frozen upstream remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. No Actions/build/test/generator/package execution occurred, and no production code, ledger, RTM or flags are changed.

### 22.1 Capped-family decomposition and frozen-ref resolution

A moving-default candidate search for `closeOpenClawStateDatabaseAsync` under `extensions/` returned 163 paths. The broad result was decomposed by extension family, with every family below 100 results; the named high-volume families were Codex=21, Matrix=21, memory-core=13, Microsoft Teams=12, Telegram=11, Feishu=10, Discord=2. Candidate discovery is not authority: every path below was re-read at the frozen commit and compared against the complete Round87 dossier before inclusion.

The full-dossier dedupe left 48 moving-default candidates not yet recorded. Frozen exact-ref resolution produced 47 real files and one moving-default-only exclusion: `extensions/codex/src/app-server/reasoning-effort.native.test.ts` is NOT_FOUND at `ad8d12f...` and remains excluded. After full-dossier dedupe, Codex/memory-core/Matrix/Microsoft Teams/Telegram contributed no new positive direct-close identity; their earlier Round87 identities remain authoritative. New frozen direct-close/test-support identities are:

- Feishu: `extensions/feishu/src/bot.cleanup.test-support.ts@dae93e77f2149163bdb07553a5594b83f1c6060c`; `extensions/feishu/src/monitor.cleanup.test-helpers.ts@504e86edc35f97f63d35f2530b58f22e53746441`; `extensions/feishu/src/monitor.post-replay.test.ts@b6e738fda113188e2cffcf9b13c68e5e664223be`; `extensions/feishu/src/feishu-ingress.test.ts@25d0ac0a7fae121377d6be3e3086b57c2c0140cf`; `extensions/feishu/src/bot.broadcast.test-support.ts@64ff60c446668a027a69bd9d80100984eaea356f`.
- Discord: `extensions/discord/src/monitor/thread-bindings.worker.test.ts@e07b74249cbf64f8a41e3e31964714c84ac0ed9b`; `extensions/discord/src/monitor/model-picker-preferences.test.ts@015be7f89a6baac830699f66f8fb683aaa7a18e9`.
- Crabbox: `extensions/crabbox/src/crabbox-warm-scenarios.test-support.ts@bfde5645c97a8bc6a998e4574345a646333165ea`; `extensions/crabbox/index.test.ts@61f0f7dcaeb614c3908c49d78d7cf3f6c78039c1`; `extensions/crabbox/doctor-contract-api.test.ts@9f3da80afa60bff792a22805419bdff8166ff89c`; `extensions/crabbox/src/crabbox-worker-provider.test-support.ts@02920557a4f9ac08bcf1957caae43196a36a4a43`; `extensions/crabbox/src/doctor.test.ts@79dbcd0f1dd7069c4a9d676d4d005723ead947b8`; `extensions/crabbox/src/crabbox-gateway-methods.test.ts@4b65a6064431dfc96316c99fa2b8589f6e7d2921`.
- Voice Call: `extensions/voice-call/src/manager.test-harness.test.ts@aad73e393b9f6fc3d984a6d5e6737144d8325af2`; `extensions/voice-call/doctor-contract-api.test.ts@c9d7cb43f3a85938f271827570ac5fff6ae09333`; `extensions/voice-call/src/manager.test-harness.ts@f5727d1b146f14485d01c52e8d6fc6702f47ab35`.
- Reef: `extensions/reef/src/flow.test-helpers.ts@9908d383ebc360067ed8b3e6e01dda111f4de4e9`; `extensions/reef/src/replay-state.test.ts@985725beaf1ff0051978666fad58d561106cdf76`; `extensions/reef/src/state.test.ts@161ca19ecadfe13ac86ab264bbd1e2c9f466598d`; `extensions/reef/src/trust-store.test.ts@e89a0d689db8945dbd95287f798c0ceba8f8dde1`.
- QA Lab: `extensions/qa-lab/src/codex-plugin-lifecycle.test.ts@7e9411440d8341da22361fb5f63a52ecd199ab45`; `extensions/qa-lab/src/providers/shared/auth-store.test.ts@efea9aa0e93141fc03d2e1ccd5f49770e56f6dca`; `extensions/qa-lab/src/gateway-child-auth-profiles.test.ts@b61a834d99d3e0e598bc87981256475a393fbcb6`.
- iMessage: `extensions/imessage/src/monitor/catchup-bridge.test-support.ts@624d6db41ca64f9741d152854dbd8173d8a5f401`.
- ACPX: `extensions/acpx/src/service.test.ts@bdae46d9585b4ac9882e68ca2b388032c06f2523`.
- Zalo User: `extensions/zalouser/doctor-contract-api.test.ts@4c9e1cb45f1983b50fdc609109b5b4dd3458af15`; `extensions/zalouser/src/zalo-js.listener.test.ts@0c588ead2154970a961009b10c2f2d1a7a7bfa33`.
- Google Meet: `extensions/google-meet/participation-runtime-registration.test.ts@01e56c4c695318654921b38e6e6d35d8545fb109`.
- 1Password: `extensions/onepassword/src/tool-list.test.ts@e5d7745b7cddf2e559dc78c3cd69dd2ea060d418`; `extensions/onepassword/index.worker.test.ts@157e81d4973c43a4ba87e7aa803d8ddb0b1a1730`.
- Signal: `extensions/signal/src/reply-authors.test.ts@1aec4aeddcac044c96fdb88a4100b0ad7643c842`.
- Slack: `extensions/slack/src/conversation-bindings.test.ts@871ec9fa728e2a9701586a977640be05ed032d6e`.
- Zalo: `extensions/zalo/src/outbound-media.test.ts@1407b1caac2a729d4021a0912736ffcad7c4ceb7`.
- Buzz: `extensions/buzz/src/buzz-bus.socket.test.ts@b7808ea165947a699cd36e9ec22f76d38880033b`.
- Copilot: `extensions/copilot/test/bench-attempt-transcript-journal.ts@4a4d544944a70888dcbbbc22206296162b66afcb`; `extensions/copilot/harness.test.ts@c45431e9fc39abe3f1a05d60c2b9dcf30bf8c7c6`.
- WhatsApp: `extensions/whatsapp/src/state-migrations.test.ts@b9e2ee1614b0ecc263f64b0c30ac898b1ba7625a`; `extensions/whatsapp/src/auto-reply.test-harness.ts@bc676918409d2ac6a0901ba44577d6a38647ca15`.
- X: `extensions/x/src/client.test.ts@e9ffcd5c4e903c5140351aa653a596e468662bfb`.
- OpenAI: `extensions/openai/binary-transport.test.ts@4359d209e048de5a759616d64e804e2eca3d4538`.
- Shared extension test support: `extensions/test-support/debug-proxy-env-test-helpers.ts@470076a03951c81d0889ccbd046d328206ad802e`.
- LINE: `extensions/line/doctor-contract-api.test.ts@c76e26224fef7a310975098594678d93a1a665b3`.
- Beam: `extensions/beam/src/beam.test-support.ts@244f610d0915f7409c0c93c04dc8cd4a77538a29`.
- IRC: `extensions/irc/src/monitor.test.ts@e3c0fe43a6e28fb8aed551c231de86f5ff3b2987`.
- Nostr: `extensions/nostr/src/nostr-state-store.test.ts@e5eb60406f19d834fc73e9ff044fbb08f9131f5c`.
- Device Pair: `extensions/device-pair/notify.test.ts@fed70562f105286cb7cb7715120dea273ce6c990`.
- Visitor Access: `extensions/visitor-access/index.test.ts@54c4cc6ee1550059e54bd3b09e8ca5c844c8203f`.

### 22.2 Exact production-path anchors for the named families

The direct close callers above remain test/test-support evidence, not shipping close owners. The exact frozen production modules that those tests exercise were also read:

- Microsoft Teams SSO state: `extensions/msteams/src/sso-token-store.ts@f273e99054969107f6f3eccfd7005900d8e40039` → account-scoped namespace → runtime `state.openKeyedStore` → bounded SHA-256 token key → SQLite mutation lock for save/remove. The already-recorded SSO test proves default-account migration survives reopen while named-account updates/removal stay isolated and long identifiers stay bounded.
- Discord thread bindings: `extensions/discord/src/monitor/thread-bindings.manager.ts@b601cb5b400d3e8a8f09ffa517d4f3cf966e19e6` → current manager generation → persisted thread-binding store → sweeper/current-binding recheck → unbind/stop. The worker test proves restoration, mutation and shutdown SQL stay off the process main thread; a stale probe cannot remove a replacement binding.
- Discord model preferences: `extensions/discord/src/monitor/model-picker-preferences.ts@741c1c1ba1ab719d9b7bf7977296ac0620b49c08` → scoped plugin-state store → normalized model key → bounded recency/prune. Store failures are optional misses, concurrent selections are retained, and retirement of the rebuildable legacy command cache must not erase canonical SQLite preferences/bindings.
- Feishu dedupe: `extensions/feishu/src/dedup.ts@92f6f7a3084a5ad054736689f1291b8ae701bb97` → claimable guard → account namespace → commit/hasRecent/warmup. Pending claims do not become committed across restart; committed marks do; TTL expiry reopens only after expiry; persistence failure may log while in-memory duplicate fencing remains.
- Feishu durable ingress: `extensions/feishu/src/feishu-ingress.ts@8ab2d929de6227d17041d814a9d4afca8678ef65` → durable channel ingress queue → adoption/abandonment lifecycle → replay-claim release/commit. Durable append must settle before a 2xx acknowledgement; append failure must not dispatch; transport adoption failure releases logical claims; logical dedupe-commit failure does not reopen an already adopted transport claim; overlapping abandonment wins over later adoption; terminal finalization precedes durable completion.
- Feishu monitor and bot: `extensions/feishu/src/monitor.account.ts@70948ddfd1282ea75abc487a5697620651a486f3` → bot identity + dedupe warmup → durable ingress/transport → message handler, and `extensions/feishu/src/bot.ts@0dfbc99d6603b8185b4203bfe89514e2087e74d4` → live config reauthorization → dedupe/finalize or supplied adoption lifecycle → channel inbound/reply path. Duplicate delivery cannot start a second turn, and bot messages fail closed when local bot identity is unavailable.

Feishu test-support consumers were exact-ref read so helper cleanup is tied to real product paths rather than treated as an owner: `extensions/feishu/src/bot.test.ts@5a36d7e980426ccbc811573dbb05f819b2ae71d5`, `extensions/feishu/src/bot.bot-threads.test.ts@258b33476681bbae50f05cffa6761d968288e85d`, `extensions/feishu/src/monitor.cleanup.test.ts@ba25d1e127824041f29726666ca90b6750e20673`, `extensions/feishu/src/monitor.startup.test.ts@758d483fa319501aea506c7e1466f27501e40f5d`, `extensions/feishu/src/monitor.webhook-routing.test.ts@55cbaa6d005528014fbb1b603a6fc14d1e99679d`, `extensions/feishu/src/monitor.webhook-e2e.test.ts@bcc3e1f4e96e9c8ba2e909aae47f471215ae5c4b`, `extensions/feishu/src/bot.broadcast.routing.test.ts@4e02f5eb1d3882a73cde2d27d7ea2a775ca39d4f`, and `extensions/feishu/src/bot.broadcast.test.ts@349182e7dc7e3f53873a2633924f14d0640436eb`.

### 22.3 Caller → product path → negative oracle for the remaining families

- Crabbox callers map to `index.js`, warm-image store/provider, worker process/command, Gateway snapshot methods and doctor. Negative oracles: malformed Gateway mutations fail before owner access; pending retirement is preserved; owner refusal is distinct from storage failure; recovery targets only the acknowledged capture; replacement provider generations remain usable; abort/stop joins in-flight work; doctor detection does not mutate unsupported/future or paid ownership.
- Voice Call callers map to manager/context/store and doctor migrations. Fixture shutdown must release timers, database workers and temp directories; absent stores stay absent; retired call logs are refused without creating SQLite state; schema repair is plugin-local rather than a reason to resurrect legacy call-log ownership.
- Reef callers map to flow/state/replay-store/trust-store. Stale completion/cleanup cannot overwrite a successor; conflict retry must recheck lease eligibility; nonce-preparation failure preserves current custody; borrowed worker results after authority revocation are rejected; malformed approvals never authorize; delivered-marker persistence failure is visible; peer-key changes/revocation fence trust commits and cleanup.
- QA Lab callers map to Codex plugin fixture, isolated auth-store staging and Gateway child auth profiles. Repairing a missing Codex plugin cannot leak into API-key fallback; inherited host shared state remains unchanged during isolated staging; target auth DB refusal and pending legacy-source refusal fail closed; Codex API-key handoff stays in the Gateway env rather than being persisted as a profile.
- iMessage catchup helper maps to catchup bridge/runtime/session cursor state. Per-run limits are bounded and oldest-first; a dispatch failure holds the cursor; per-chat/query failures remain explicit; invalid persisted cursor timestamps do not crash the recovery lane.
- ACPX maps to service/process-lease/state. Legacy sessions remain preserved; lifecycle publication/retraction is exact; stale process leases are reaped without closing unidentifiable pending descendants; skip mode avoids probe/health; catalog-only disposal must not retire another active runtime lease.
- Zalo User maps to doctor/session-state plus monitor/ZCA listener. Credential migration persists into SQLite before archive and never restores an explicitly cleared profile; an unconfigured install avoids unrelated agent-session inspection; listener cleanup permits a replacement, while handshake timeout settles and reconnects the same monitor profile.
- Google Meet maps to plugin participation runtime registration. Persisted unsupported outcomes may replay, but replay must not restore authority for an ended participation session.
- 1Password maps to broker/tool and pending authorization worker paths. Corrupt/expired/null grants do not become active; callers without an agent do not enumerate grants; competing executions hand off approvals without parent-thread SQL.
- Signal reply-author state preserves merged reply context across reopen; a failed update cannot erase already-retained memory, and parent-thread SQL is not the fallback.
- Slack conversation-binding tests map to conversation-route owner, installation identity and session-binding runtime. Workspace binding capability/mutations are preserved, while detected org installs cannot advertise, select or mutate unsupported bindings.
- Zalo outbound-media tests map to hosted-media preparation/runtime. Proxy-aware fetch is preserved; HEAD does not consume media; invalid expiry/token/id fails before disk or delivery side effects.
- Buzz bus tests map to bus/gateway signed roster state. Stale snapshots cannot reauthorize removal; a confirmed rejoin is required; silent-presence recovery does not replay pre-activation messages; an older query completing after a push cannot downgrade the signed role.
- Copilot callers map to attempt transcript journal and harness/attempt/runtime. Cancellation or revoked ownership prevents further SDK dispatch; late sessions are disconnected after deadline; disposal errors remain aggregate failures; reset cannot delete another OpenClaw session; deferred cleanup blocks premature reuse; durable bindings survive restart only when compatibility/auth fingerprints still match.
- WhatsApp callers map to state migration and inbound/auto-reply harness. Original credential bytes are preserved on conflict/incomplete/logout; orphaned keys remain until original identity returns; root ownership is not silently reassigned; the auto-reply harness close is teardown evidence, not a production DB-close owner.
- X client tests map to OAuth client/runtime plugin state. A retired seed cannot overwrite a replacement grant; retry/reopen is fenced; held provider responses recheck current account config; commit failure recovery cannot exchange the completed rotation twice; an aborted caller may detach while the joined owner persists the late rotation.
- OpenAI binary transport maps to speech-provider/proxy capture. Invalid audio is rejected, byte/time budgets remain bounded, persisted capture survives finalization, and rejected upstream sockets are closed.
- LINE doctor tests map to doctor state migration. No pre-drain rows means no migration; configured and removed named accounts are handled; lack of the required ingress-queue host capability fails visibly rather than silently skipping.
- IRC monitor maps to IRC ingress/monitor/runtime. Delayed replies cannot cross a reconnect generation; only the stopped account settles; unavailable credentials do not open a connection; outbound receipt-time self echoes are not reclassified as inbound activity.
- Nostr state-store maps to the account-scoped Nostr state store/runtime. Account IDs remain isolated, legacy key bytes remain readable, profile publish state survives reopen, and since-timestamp selection uses the maximum valid watermark.
- Device Pair maps to notify-state/notify. Persisted one-shot requests deliver once; service recreation cannot duplicate a slow send; subscriber changes during delivery are preserved; same-millisecond re-arm is not consumed by the previous delivery; malformed/cross-type Telegram thread IDs still compare by the persisted canonical identity.
- Visitor Access maps to plugin lifecycle/visitor grants/tools. Discovery is side-effect free; person-wide revocation is distinct from exact pending-invite cancellation; owner authority is rechecked after provider reads; expired/revoked grants cannot renew; startup/sweep remains restart-safe and provider cleanup is aborted on stop.
- Pure support-only direct callers (`extensions/test-support/debug-proxy-env-test-helpers.ts`, `extensions/beam/src/beam.test-support.ts`, and similar harness files above) remain test cleanup infrastructure. Their presence is negative production evidence: they do not establish a shipping state-DB close owner and cannot upgrade implementation/wiring status.

### 22.4 Owner conclusion and open gates

The expanded extension closure still finds no extension production module directly owning process-wide `closeOpenClawStateDatabaseAsync`. The new positive evidence is caller/oracle provenance around existing plugin-state, durable-ingress, worker and lifecycle owners. Shipping shared-state retirement remains the already-recorded Gateway/global-singleton path; extension cleanup calls are test isolation or bounded feature-state recovery evidence.

Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, OCRM final acceptance and release acceptance remain absent. No Actions/build/test/generator/package was run. No production code, ledger, RTM or flags were modified.


## 23. Round83 continuation — exact production owners behind extension close callers

This continuation starts from the Round82 direct-close/test-support frontier and follows those callers through their frozen production imports. Every identity below was fetched at `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`; the frozen upstream tree remains `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. These are source/owner/oracle mappings only: no Actions/build/test/generator/package execution occurred, and no production code, ledger, RTM or flags are changed.

### 23.1 Crabbox: warm-image state, worker provider, snapshots, Gateway and doctor

Frozen production identities:
- `extensions/crabbox/index.ts@f7b32bbed65069e3a0fcad1d01592f8c154ef487`
- `extensions/crabbox/src/crabbox-worker-warm-image-store.ts@1031bf93f52dc15f65eaacd4e7a0a6197ace60bd`
- `extensions/crabbox/src/crabbox-worker-provider.ts@a0ef59bbd42ef07b88d3bfb5a5a73fdc507b1b11`
- `extensions/crabbox/src/crabbox-worker-snapshot-actions.ts@78a412b39bfd4b54b7935049d4a8c3f08e8b4f57`
- `extensions/crabbox/src/crabbox-gateway-methods.ts@b55bdd1f79ee96963561a3806c19dc8366251cba`
- `extensions/crabbox/src/doctor.ts@aa26ac1aab70a3f199ab0a8698ad866b4ce9341e`
- `extensions/crabbox/src/crabbox-process.ts@0ccfe03b9db815de9ea466891396eacc76597203`
- `extensions/crabbox/src/crabbox-worker-command.ts@36a37ad3f476e4baa7ace04e72202942a6c4481d`

The warm-image store owns bounded canonical records through a V2 compare-and-apply store with `reject-new`. Only explicit comparison conflicts are retried; uncertain writes remain with their owner. Provision replay is fenced to the recorded profile/project/runtime identity. Manual capture recovery requires explicit provider-cleanup acknowledgement and an unchanged selector; changed or absent selectors leave state untouched. Gateway mutation methods validate complete parameter shape before owner access and distinguish invalid-request refusal from unavailable/storage failure. Provider disposal aborts the provider generation and joins heartbeat, maintenance, snapshot and binary-resolver lifetimes; maintenance calls `context.assertCurrent()` before and after asynchronous resolution. This is the production path exercised by the six Round82 Crabbox direct-close/test-support callers, not a new shared-state close owner.

Negative oracles retained from the exact tests: pending retirement cannot be erased by a stale cleanup; a replacement provider generation remains usable after the old generation stops; malformed mutation input performs no state access; recovery mutates only the acknowledged exact capture; future/unsupported/paid ownership is preserved; abort/stop joins owned work rather than publishing a false successful release.

### 23.2 Voice Call: manager lifecycle and durable call-event state

Frozen production identities:
- `extensions/voice-call/src/manager.ts@7a69169734f13bf408d502cd5e3f248c130e8e97`
- `extensions/voice-call/src/manager/context.ts@ce51fd51b75eb04008e922ff811e293ebbb0b7ec`
- `extensions/voice-call/src/manager/store.ts@20c0beb20d5d9fb78fb9d5295c873eea9a688d5a`
- `extensions/voice-call/doctor-contract-api.ts@835c7cb2ffc16beaa2afc9803f4e1d6fd4302f4f`

`CallManager` is the lifecycle owner for active calls, provider aliases, replayed event IDs, timers, pending admissions/work and end-call operations. Once stopping begins, new operations reject; transcript waiters and timers are explicitly settled, and restored calls are revalidated before becoming live. Durable call snapshots are written as chunked event records; guarded registration keeps the final await under caller authority, while ordinary writes preserve the historical publish-before-prune recovery contract. Active-call restoration rebuilds call/provider/replay identity and excludes terminal records. Doctor migration repairs the plugin-local SQLite schema without resurrecting the retired call-log owner.

Negative oracles: shutdown must release timers/database workers/directories and refuse new work; unavailable history is not equivalent to an absent call; a stale or replaced call record cannot regain automatic-response authority; terminal calls are not restored as active; legacy call-log absence/refusal cannot create a second state authority.

### 23.3 Reef: replay claim owner, trust operations and flow state

Frozen production identities:
- `extensions/reef/src/flow.ts@e559f42e2c4311fee5420b7cec8247befb1fbff0`
- `extensions/reef/src/state.ts@d8b6d43c03c6a00b699e88cab1a74f97fa6ae520`
- `extensions/reef/src/replay-store.ts@477603189a3054766209c9c4be502b1f06ab1a7c`
- `extensions/reef/src/trust-store.ts@210bcdce7369e11da1d059b7fbccea6ac69728b1`
- `extensions/reef/src/transport.ts@1182e2dcbcc2fb54706461388868160d7794dc79`
- `extensions/reef/src/commands.ts@2dd636fea24c477e2bda9b6f90c2f915d9157506`
- `extensions/reef/src/friends.ts@923882cfe58307584aa7814fed33520b35629d59`

`ReefSqliteReplayStore` serializes durable replay mutations, stores a generated claim owner, requires matching peer/id/envelope hash, and only completes/refreshes/releases the currently published claim owner. Conflicting live claims return `in_flight`; expired claims can be reacquired with a new owner. `ReefTrustStore` builds multi-store operations with `assertCurrent` and optional owner revalidation, keeps removal/request settlement one-shot, and binds trust/autonomy mutations to revision and identity scope. These owners are the production paths behind the Reef close-caller tests.

Negative oracles: stale completion/cleanup cannot overwrite a successor; a conflict retry must re-read eligibility; nonce/preparation failure preserves current custody; malformed approval never authorizes; borrowed worker results after revocation are rejected; request/removal settlement cannot be consumed twice; peer-key/revision changes fence later trust commits; durable delivery markers cannot be silently synthesized after persistence failure.

### 23.4 QA Lab, ACPX and Copilot runtime ownership

QA Lab frozen identities:
- `extensions/qa-lab/src/providers/shared/auth-store.ts@d3e2ae91bc8800966cd712c8a4fd2e7ba8c9921a`
- `extensions/qa-lab/src/codex-plugin.fixture.ts@b81238b6758697e5047709bb95f7cad6fd66f4fc`
- `extensions/qa-lab/src/gateway-child-lifecycle.ts@fbfd3784b675a84dfce4baaba39af3a4698074e6`
- `extensions/qa-lab/src/gateway-child-setup.ts@a45a9f46523ce4aab792ad29bdb06ef35e915c0f`
- `extensions/qa-lab/src/providers/live-frontier/auth.ts@071c9b5ab1c9544472f05ee8468d7cd7aef49fc8`

The QA paths are isolated child/fixture authority: staged auth must not mutate inherited host shared state; target auth DB refusal and pending legacy-source refusal fail closed; Codex API-key handoff stays in the child Gateway environment rather than becoming a persisted profile; plugin repair cannot silently fall back to a different credential path.

ACPX frozen identities:
- `extensions/acpx/src/service.ts@84b89ceaf35fd7910b2e3f2fa416344a877ca394`
- `extensions/acpx/src/process-lease.ts@ddb075ac0dd9b29b2dfb487880724eead13b3a33`
- `extensions/acpx/src/state.ts@c3ae7f58ef5f9206cc65554ae8e7f22483486e4f`

The service persists a Gateway instance identity, snapshots legacy session ownership under the service owner, and reaps only matching persisted process leases. Recovery is fenced by a lifecycle revision and owner assertion. Process identity is encoded as explicit lease-id + Gateway-instance-id arguments; closing/lost terminal lease state removes that exact lease record. A pending wrapper with no proven process identity cannot be treated as proof that detached descendants are gone. Tests therefore require exact publication/retraction, no probe/health side effects under skip mode, and no disposal of another active runtime lease.

Copilot frozen identities:
- `extensions/copilot/harness.ts@4b65bef752744a8280437b269a7956f4f2f268a8`
- `extensions/copilot/src/attempt.ts@30fa89f2a1fc2a9fba195733fe8d69f721c79f44`
- `extensions/copilot/src/attempt-transcript-journal.ts@2ec5b3c88f66c7d19643f7cc65c45015fdeae218`
- `extensions/copilot/src/runtime.ts@5b31fee1560e71ad37a7350e828b38b221f0aa0b`

The harness owns pool lifetime, in-flight attempts, per-session binding serialization, deferred-compaction cleanup and tracked native sessions. New attempts reject after disposal. Reuse requires a compatible session binding; settled-tool finalization fails closed without the exact compatible SDK session. Compaction requires host source authority and rechecks it before/inside the SDK operation. Durable binding cleanup failure does not authorize deletion/reuse of a different native session. Negative oracles from the close-caller harness include: revoked ownership/cancellation cannot dispatch further SDK work; late sessions disconnect after deadline; dispose errors surface; deferred cleanup blocks reuse; reset cannot delete a session owned by another OpenClaw session; auth/model/cwd compatibility changes prevent reuse.

### 23.5 Visitor Access and channel/runtime families

Visitor Access production identities:
- `extensions/visitor-access/index.ts@fbb00cb9b7e53f8901be0e38dac98e28991a5341`
- `extensions/visitor-access/src/visitors.ts@f86cc538256012ee846c6e5ab56511842af00962`
- `extensions/visitor-access/src/tool-results.ts@c09f283ab4a4edc18cd81a4c87768fcacaa81188`

`VisitorAccessService` serializes grant mutations, publishes live grant authorities with abort controllers, refuses authorization before initialization/after close, and aborts every live grant when stopping. Resume uses the exact durable grant id instead of inventing authority when the grant map is unavailable. Invite/revoke mutation paths take caller `assertCurrent`; selector forms are mutually exclusive. Negative oracles: post-provider-read identity changes must be rechecked; expired/revoked grants never renew; person-wide revocation is distinct from exact invite cancellation; stop aborts provider cleanup and removes live authority.

Additional frozen production anchors behind the remaining Round82 close callers:
- iMessage catchup: `extensions/imessage/src/monitor/catchup-bridge.ts@66a55bba5d35dedfd5836d213584b6bf3cf2a1dd`, `extensions/imessage/src/monitor/catchup.ts@6eb6ab1633030fd5ce011cb615bc12bb3d7d825c`. Oldest-first bounded catchup advances its cursor only through the defined dispatch/high-watermark contract; dispatch/query failure cannot silently skip custody.
- Zalo User: `extensions/zalouser/src/session-state.ts@c5bd529cecb715c96520c951802eccd01610ae36`, `extensions/zalouser/src/monitor.ts@f7b9dcc7b5912d052e6403fbde3ff4bfea0d79b5`, `extensions/zalouser/src/zalo-js.ts@7306d88d7c45786b667f2bda1c832bfcde1e4e4b`. Credential identity remains account/profile scoped; listener cleanup may permit a successor only after the old listener settles.
- Google Meet: `extensions/google-meet/index.ts@643dcd738d66f40daaa52948a8765692237ddead`, `extensions/google-meet/src/plugin-registration.ts@b13d2c9fb8511f4eaf58b2699d5aac70115741fb`, `extensions/google-meet/src/runtime.ts@875f106ac2e3e61fc29fb0ad55588dd035a1af25`, `extensions/google-meet/src/transports/chrome.ts@d052dde1e38198f0f5043d02b78c13177ccf9328`. Persisted unsupported participation outcomes can replay diagnostically but cannot restore authority to an ended participation session.
- 1Password: `extensions/onepassword/src/broker.ts@34e151d4917f83379a46e2cc11d85e4f4dc51999`, `extensions/onepassword/src/tool.ts@01da2e6faf38b0828c2c94cd9ff5d72d8e731b3f`. Grant/authorization selection stays with the broker; corrupt/null/expired rows remain inactive and competing tool executions cannot turn one approval into two owners.
- Signal reply authors: `extensions/signal/src/reply-authors-state.ts@fd2a1eae1b967dcaece99e0b0bc6b58d3a3306dc`, `extensions/signal/src/reply-authors.ts@e5fbfc0a6e979354601ddaf1f5c9922d18c4f49a`. Persisted merged context survives reopen; a failed mutation cannot erase already-retained memory or fall back to parent-thread SQL.
- Slack bindings: `extensions/slack/src/conversation-route-owner.ts@b582fecf04cc6fb1ca73b3726d0b21d2c7b8bc33`, `extensions/slack/src/installation-identity-state.ts@5d10ae91d250360cc9dc534a7be80ac69607cfdb`, `extensions/slack/src/channel.ts@69edb5c0f9c64bf7380472faedf7bf9ddb280ec1`. Workspace binding capability is distinct from org-install identity; unsupported org installs cannot advertise/select/mutate workspace-only bindings.
- Zalo hosted media: `extensions/zalo/src/outbound-media.ts@67f5e65d602c0b91029cbfd85f6d59766a5b1d42`, `extensions/zalo/src/runtime.ts@e92ba6643449f8693da8fde08d4ae504dac438c3`. Token/expiry/id validation happens before serving bytes; HEAD does not consume media; proxy-aware fetch identity is retained.
- Buzz: `extensions/buzz/src/buzz-bus.ts@0f8865f0f9eba10747f01fdf3af844347a428eef`, `extensions/buzz/src/gateway.ts@df46f8dd8b31b22a69ba5d6a774ab09e551ab444`. Signed roster/current presence is authoritative; stale snapshots cannot authorize removal or downgrade a newer push role.
- WhatsApp: `extensions/whatsapp/src/state-migrations.ts@789857bcaf986319acb9a91821fd15754ef22776`, `extensions/whatsapp/src/auto-reply/monitor.ts@68302af6ee2f6364cc75db2603801c1b429d21c2`, `extensions/whatsapp/src/runtime.ts@ba058a32940198ba7a2aa51ee50f8bdb78d90ac3`. Credential migration preserves conflicting/incomplete/orphaned originals until their exact authority can be reconciled; auto-reply close remains harness teardown, not the shared DB owner.
- X OAuth: `extensions/x/src/client.ts@11f7bdfa3cf3168dfc8c54bdb1e7c1d6891127dd`, `extensions/x/src/runtime.ts@ea1d378ffb2dd290d09bd237d178238b95f44766`. Grant rotation is fenced across reopen/retry; retired seed state cannot overwrite the replacement; completed rotation is not exchanged twice after uncertain commit.
- OpenAI speech: `extensions/openai/speech-provider.ts@b1e50a3dde970c3cbac6cfdc2ee47d5ed9717567`, `extensions/openai/index.ts@2cb4ed48ef19dd9d79f6f8575bd2d06a86aa5cb9`. Invalid audio and byte/time-budget violations fail before accepting capture; rejected upstream sockets close without manufacturing a successful transcript.
- LINE doctor: `extensions/line/doctor-contract-api.ts@60a16182e6b8d660399424f2b32776316dbf5c09`, `extensions/line/index.ts@3bb6e17f1620046fab38be5d1b6cb74ff2ece001`. Pre-drain migration is conditional on real rows/host capability and fails visibly if the channel ingress queue is unavailable.
- IRC: `extensions/irc/src/monitor.ts@4fb0671e956c6f512c0a173909e822e05fb924c2`, `extensions/irc/src/irc-ingress.ts@fbc921ac020adce4ab701d46d488df6425b43a52`. A delayed reply from an old connection cannot cross into a reconnected client generation; only the stopped account settles.
- Nostr: `extensions/nostr/src/nostr-state-store.ts@35372930926095f28634536e1a18fa4d9e3954dd`. Bus/profile state is account scoped, survives reopen and advances its since-watermark monotonically from valid persisted timestamps.
- Device Pair: `extensions/device-pair/notify-state.ts@496b212e0207adb67652d34ab3e17ab6ee89809a`, `extensions/device-pair/notify.ts@d83b9cd3c595fcbe803b9b51ed9613d8421b06c0`, `extensions/device-pair/index.ts@5aca2651ad0687ffc33c04903ad1cabe8a34c1c4`. One-shot notification custody survives service recreation and cannot be consumed twice by a slow delivery or same-millisecond re-arm.
- Beam: `extensions/beam/src/store.ts@b501ac4a7948988366e771cd411bec6425159fc9`, `extensions/beam/src/mirror.ts@011c9422ddded974f5c4317251e81115dfbaf036`. The close caller remains test-support cleanup around the Beam store/mirror owners; it is not a process-wide DB close path.

### 23.6 Closure result for this frontier

The Round82 extension direct-close frontier is now linked through frozen exact production owners for the high-risk lifecycle/state families rather than only through test filenames. No inspected extension production module imports or owns process-wide `closeOpenClawStateDatabaseAsync`; direct extension calls remain test/test-support/bench teardown or deliberate reopen-oracle boundaries. The shipping shared-state retirement owner therefore remains the already-recorded Gateway shutdown → agent DB retirement → global singleton drain sequence. This does not upgrade Rust implementation, production wiring, scoped G01/G02, executable/package proof or independent acceptance.

Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, OCRM final acceptance and release acceptance remain absent. No Actions/build/test/generator/package was run. No production code, ledger, RTM or flags were modified.


## 24. Recovery continuation — reverse consumer and lifecycle-test closure

This continuation reverse-walks the frozen Section 23 production owners into their direct consumers and focused lifecycle tests. All exact identities below were fetched from `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`) and deduplicated against the live Round87 dossier before publication. Entries already present in Round87 are referenced as prior evidence rather than counted again.

### 24.1 Crabbox warm-image consumers and generation/retirement oracles

New frozen identities:
- `extensions/crabbox/src/crabbox-worker-warm-image-cli.ts@28f1b4d2cca8bf199145de123290f680b9fd54dc`
- `extensions/crabbox/src/crabbox-worker-warm-image-allocation.ts@27427bcb41602ad97b157f7f6e736b20b44b1221`
- `extensions/crabbox/src/crabbox-worker-warm-image.ts@af4b96a5fab3adf751864b784981e01c61fb8be5`
- `extensions/crabbox/src/crabbox-worker-warm-image-capture.ts@12cc2cbcd40a33502927bc1a284216cc66428912`
- `extensions/crabbox/src/crabbox-worker-warm-image-lifecycle.test.ts@ce9907a7e82397be46e50e51a2ba9c1d032799c5`
- `extensions/crabbox/src/crabbox-worker-warm-image-retirement.test.ts@37e850c507874009ab6a62dbf0900ef08de365f0`
- `extensions/crabbox/src/crabbox-worker-warm-image-recovery.test.ts@66ddbed3b5941e17fab48ffa662cdb31a690d296`
- `extensions/crabbox/src/crabbox-worker-node-enrollment.test.ts@b6f534ec508c97ee0cf3624b05a668570eb20dab`

Production path: CLI recovery requires the inspected exact selector and explicit provider-cleanup acknowledgement; allocation rechecks `assertCurrent` around generation-bound store updates; the warm-image manager records retirement as durable deletion debt and only clears it through the compare-and-apply owner; capture claims the operation before source preparation/scrub and rechecks owner authority across async boundaries. If capture ownership changes before publication, the returned checkpoint is not published as the current generation and is instead surfaced for reconciliation.

Focused negative oracles: provider disposal aborts its heartbeat generation; explicit recovery clears only the selected abandoned reservation; failed retirement remains durable across restart; old retirement completion cannot erase a newer generation; recovered scrub completion cannot overwrite a replacement capture; project authority closing during scrub prevents native capture. The already-recorded `crabbox-worker-warm-image-allocation.test.ts@89af0fe6b67238f2605fb960287d56c2e7f3c811` remains prior Round87 evidence and is not counted as new here.

### 24.2 Voice Call stop/restart/delivery consumers

New frozen identities:
- `extensions/voice-call/src/runtime.ts@19cb120156df6983b446d29056722c32f3b8d7c4`
- `extensions/voice-call/src/webhook.ts@e9ee3f612a5441d6a0a53eb13cafaf9c3e924458`
- `extensions/voice-call/src/call-delivery-runtime.ts@a6a5a20590683645b00814111d1a8137cc65da10`
- `extensions/voice-call/src/manager/lifecycle.ts@8f7ae8937f0bf75a55c5b672fac6e7f2d9abd5db`
- `extensions/voice-call/src/manager/mutations.ts@c1ef7d69d7205791eb4000cd4fca3b56e6334fb5`
- `extensions/voice-call/src/manager/events.ts@e25232482a124c88f3c57c6b2b6a48d29f0366c7`
- `extensions/voice-call/src/webhook/stale-call-reaper.ts@3eb34f6c1c5d1245fb1d9af70663888233ff68c3`
- `extensions/voice-call/src/manager.restore.test.ts@6fb5eb6139777903874497a8344985df24a74f87`
- `extensions/voice-call/src/webhook.auto-response.lifecycle.test.ts@e3e35986ddc19adc5558285f716b601a8e0193c7`
- `extensions/voice-call/src/webhook/realtime-handler.lifecycle.test.ts@5036337d8d5e044ce2211803cb9e88b8ba697158`
- `extensions/voice-call/src/call-delivery-runtime.test.ts@55578bcc97f59329592b470d962c3ba0ac6bedb5`
- `extensions/voice-call/src/runtime.test.ts@1446dd483f86dc2b30a798e86c068fc04d9fe190`

Production path: runtime shutdown is memoized and joins transport/webhook/delivery/manager shutdown; webhook shutdown joins stale-call reaping, upgraded media/realtime bridges and server close; delivery aborts new work, clears timers and waits until all tracked pending work settles. Manager lifecycle persists terminal state before clearing max-duration/notify timers and transcript waiters. Event processing preserves durable replay identity and treats terminal retained calls as replayable/ignored rather than reactivating them.

Focused negative oracles: a replayed webhook performs no duplicate side effect; shutdown cannot report complete while carrier admission or provider bridge cleanup is still pending; a failed startup remains nonterminal until manager-owned carrier termination succeeds; a disconnected speech generation cannot invalidate its replacement; restored terminal calls stay terminal; interrupted pending delivery is persisted as failed after restart without describing or sending it. `webhook.hangup-once.lifecycle.test.ts@ffc8ecc016201a315c43fca222a18416be8ae9d4` was already present in Round87 and is not duplicated.

### 24.3 Reef replay/trust production consumers and authority oracles

New frozen identities:
- `extensions/reef/src/channel.ts@5ea1c4dd1ffafed3fdb88f3a4c563b336fbfdd7b`
- `extensions/reef/src/cli.ts@1fe237e18a7baa005b7e7c79026a3212cf584768`
- `extensions/reef/src/flow.source-authority.test.ts@0fe25561b21aaed26bf72bd0fb176361b887a603`
- `extensions/reef/src/transport-capacity-recovery.test.ts@f0275f58df9c3f848e85a08662df824eeac53075`
- `extensions/reef/src/flow.worker-outbound.test.ts@acc24e5ab41bb51a8e597dcfadc3a5efdb875fbb`
- `extensions/reef/src/friends.test.ts@28cafce404cebcad0875b286fd934f88411739b1`

Production path: the channel creates one runtime authority, opens identity/replay/review/trust stores under that authority, and rechecks it before handing accepted ingress/rejection notices to the host. Friend requests persist owner intent before the relay side effect; ambiguous relay failure or authority loss therefore keeps custody instead of inventing success or silently dropping the request. CLI registration similarly keeps relay claim/session provenance so a retry can recognize a previously completed remote claim rather than double-claiming.

Focused negative oracles: source authority revocation plus routing change fences later outbound/delivery effects; parked delivered-capacity entries do not advance the durable cursor, while persisted replay allows bounded redispatch only when confirmation is absent; trust/key changes during classification fence dispatch; a durable approval cannot be reused for another recipient; accepted friend intent survives ambiguous authority loss, while queued commands after owner revocation are rejected and do not create new intents. The previously recorded `replay-state.test.ts@985725beaf1ff0051978666fad58d561106cdf76` and `trust-store.test.ts@e89a0d689db8945dbd95287f798c0ceba8f8dde1` remain the direct replay/trust owner oracles.

### 24.4 ACPX lease consumers and successor-safe reaping

New frozen identities:
- `extensions/acpx/src/runtime-process-cleanup.ts@1c36d65b5b5248df0875600f37b397fe540d85f1`
- `extensions/acpx/src/runtime-session-store.ts@fd323e75bff5f39eb477765e2938f3766fb07866`
- `extensions/acpx/src/runtime.ts@6161d771645741126de0be8240b9b654a185be3c`
- `extensions/acpx/src/process-reaper.ts@b88c6e111285dbff908d2f589a55a9243b09e634`
- `extensions/acpx/src/session-owner-migration.ts@b5f0de55523c1f953890bc869c56953137945aa3`
- `extensions/acpx/src/process-reaper.test.ts@c3d0d79073bf33e17803563db0618ddc11d65f62`
- `extensions/acpx/src/runtime-process-cleanup.process.test.ts@48d69f460a1930e9c410bb36cf40d9f78487f8ee`
- `extensions/acpx/src/runtime.test.ts@092d6f027f0aca882a4074152a76a466f60a8dd3`

Production path: cleanup snapshots the exact lease/gateway identity before backend close can yield, selects by PID when available, marks only that lease closing, and maps ambiguous/unverified pending cleanup to `lost` rather than false `closed`. The session store retains the exact old close record even when publication is fenced, while the runtime refuses command reuse/reconnect for another Gateway instance. The reaper validates wrapper ownership plus lease identity before killing and rechecks caller authority around awaited inspection/termination. Doctor migration refuses repair when process/lease ownership is live, uncertain, mismatched or incomplete.

Focused negative oracles: process-list timeout fails closed without kill; lease mismatch prevents tree termination; owner revocation during awaited cleanup blocks later SIGKILL; delayed close of an old record leaves a newly leased successor wrapper alive; a foreign Gateway lease cannot be reused or reconnected. `process-lease.test.ts@c6a3da52a01852d9ee93adfc490ac9bee5e77ef2` and `service.test.ts@bdae46d9585b4ac9882e68ca2b388032c06f2523` were already present in Round87 and are not duplicated.

### 24.5 Copilot attempt consumers and finalization/session-binding oracles

New frozen identities:
- `extensions/copilot/src/attempt-finalize.ts@8d257b2a7a4a9a4f90558d191ea1266c88457f60`
- `extensions/copilot/src/attempt-active-run.ts@122eb7e078ab6986393421f32e958c1ae5d35180`
- `extensions/copilot/src/attempt-execution.ts@259d933af3d0b6d5d3f696caab622bc595b3908f`
- `extensions/copilot/src/attempt-transcript-journal.test.ts@36748323ee0922606550fc0d22d8a0f091a73b51`
- `extensions/copilot/src/attempt-active-run.test.ts@60562ba36a7e695542b8ffe7b8a1727f62c4c0c0`
- `extensions/copilot/src/attempt-session.integration.test.ts@0be7e9b43b37fd047b097b68ff60859f771e0ff6`

Production path: finalization consumes the caller-owned transcript journal and explicitly refuses to reconstruct the deleted dual-write transcript mirror. Active-run source preparation may await, so it is completed before live-run checks and no send occurs after settled/aborted state; cancel/abort also settles pending Gateway user input before aborting the active SDK session. Attempt execution distinguishes freshly created native sessions from resumed sessions so incomplete cleanup cannot authorize deletion of an older retained session.

Focused negative oracles: tool transcript persistence must join before finalization completes; journal failure/replay invalidity is preserved instead of manufacturing a validated transcript; source steering cannot dispatch after the run settles or aborts; provider failure→retry→resume→settled-tool finalization keeps the compatible host session binding and resumes only the expected SDK session. `harness.test.ts@c45431e9fc39abe3f1a05d60c2b9dcf30bf8c7c6` was already present in Round87 and is not duplicated.

### 24.6 Visitor Access grant consumers and expiry/revocation authority

New frozen identities:
- `extensions/visitor-access/src/runtime.ts@b81f16ae1df91e107ea0dae76de625708c03a25b`
- `extensions/visitor-access/src/visitors.test.ts@8afcb1046f0305df76d8b094f8b5f1747a507338`
- `extensions/visitor-access/src/visitors.authority.test.ts@a8a23f162e5a24a91a7cbf83fecd0728942b08e4`

Production path: the runtime store exposes a single Gateway-published VisitorAccessService slot so tools share the same mutation queue and lifetime; the already-recorded `index.ts` owner initializes the service, publishes only after initialization/current-lifetime checks, runs serialized sweeps, and on stop closes the service, aborts lifetime authority and joins any sweep.

Focused negative oracles: legacy/incompatible state refuses invite/revoke before provider or grant effects; caller authority is rechecked after durable cleanup is recorded and before provider access is granted; if authority closes after the final durable/provider effect, the already-successful operation settles instead of being rolled back; expiry or revocation aborts the old live authority; restart resumes only the exact durable grant id, and an expired grant never regains authority; closing while a grant read is in flight prevents starting a new durable revocation.

### 24.7 Frozen-ref exclusions and closure result

Default-branch discovery also surfaced `extensions/crabbox/src/crabbox-worker-warm-image-reconciliation.ts`, `extensions/copilot/harness.detached.integration.test.ts`, `extensions/visitor-access/src/welcome.ts`, and `extensions/visitor-access/src/visitors.onboarding.test.ts`, but each returned `NOT_FOUND` at the frozen upstream SHA and is therefore excluded as moving-default-only evidence rather than backfilled from a different source identity.

This reverse closure adds consumer/test provenance without changing the prior owner conclusion: no newly inspected extension production consumer owns process-wide `closeOpenClawStateDatabaseAsync`. Shipping shared-state retirement remains the already-recorded Gateway shutdown → agent DB retirement → global singleton drain path; extension direct-close calls remain test/test-support/bench teardown or deliberate reopen/lifecycle oracle boundaries.

Independent ACCEPT, scoped G01/G02 positive evidence, `unit_authority_ready`, executable/package proof, OCRM final acceptance and release acceptance remain absent. No Actions/build/test/generator/package was run. No production code, ledger, RTM or flags were modified.


## 25. Recovery continuation — remaining extension owner consumers

This continuation completes the reverse consumer/focused-test pass for the remaining Section 23 extension owner families at frozen `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Every identity below was fetched at that exact ref and deduplicated against the live Round87 dossier before publication. Seventeen candidate identities were already present and are retained as prior evidence rather than duplicated.

### 25.1 iMessage and Zalo User consumers

New frozen identities:
- `extensions/imessage/src/monitor/monitor-provider.ts@b25f117cbdfe26fc10c396f170b5079e7b2afd2e`
- `extensions/zalouser/src/tool.ts@988a8896097a4e2069790cb8ba8d42b0f4a44cb0`
- `extensions/zalouser/src/credential-persistence.ts@d23e8544a0e75cde0e1794b402eb6236589cbf5a`
- `extensions/zalouser/src/send.handoff.test.ts@f93f9350fa510de619a7296faf37a40c880e490c`
- `extensions/zalouser/src/send.media-handoff.test.ts@19bb3781cc885257d71268a0e4fdec52b9f3f9ec`

iMessage production wiring is now traced from the previously locked catchup owner into `monitor-provider.ts`: catchup rows re-enter the same durable live-admission path, oldest-first, while the recovery/catchup cursor advances only through durable enqueue/high-watermark rules. Query or dispatch failure holds the cursor; malformed raw rows can contribute only a raw watermark so parsing failure does not manufacture delivery. The prior `catchup-bridge.test-support.ts@624d6db41ca64f9741d152854dbd8173d8a5f401` remains the focused oracle and is not duplicated.

Zalo User production consumers keep credential identity scoped to the selected profile, serialize credential refresh/revocation, and recheck current authority before/after durable writes. Monitor stop is idempotent and joins ingress retirement before returning. Send handoff tests prove preparation precedes visible dispatch, caller cancellation after dispatch does not rewrite already-visible custody, chunk progress is retained, and guarded media downloads use the runtime dispatcher and close owned transport. Previously recorded listener/credential lifecycle tests remain prior evidence.

### 25.2 Google Meet, 1Password, Signal and Slack consumers

New Google Meet identities:
- `extensions/google-meet/index.create-ownership.test.ts@9a50bf01cc61387930c256314ed59411e41c6395`
- `extensions/google-meet/participation-registration.test.ts@7e4c793521c61d5e895181a181d27ec01d3d58b8`
- `extensions/google-meet/lazy-import.test.ts@2a561de949b6dffcd52acb3bc1781c69d4594cd0`
- `extensions/google-meet/src/transports/chrome.test.ts@655414213cd43ac2517cb7f72f29827bbcd70030`
- `extensions/google-meet/src/realtime.process.test.ts@68def97f3b893d9b5fab85d71126bac7c1dd54b1`

The already locked Google Meet runtime/registration/Chrome owners preserve browser-tab ownership across create/leave, reject malformed participation identity before runtime dispatch, preserve unsupported participation as a replayable durable outcome without restoring ended-session authority, and re-resolve recovery against current node inventory rather than a stale session pin. Realtime audio shutdown is idempotent, waits for resistant child processes, and retries only unsettled teardown phases. The previously recorded participation-runtime and OAuth-owner tests remain prior evidence.

New 1Password identities:
- `extensions/onepassword/index.ts@bceb6880924bff295d2b04d7d0477bb3ba0737d8`
- `extensions/onepassword/src/cli.ts@f6da6819f0a1598ff20d92ce8f63869f6508181f`
- `extensions/onepassword/src/op-client.ts@8182d81dfb163ec53a6afe09f24a119b4704f9d6`
- `extensions/onepassword/src/broker.test.ts@7509e443fc49f4449e6526d5bf4d123dde26fb20`
- `extensions/onepassword/index.test.ts@3eaa3aee6cecf68b44238a93b132b3c8de277c5b`

The plugin composition opens bounded audit/grant/pending stores and exposes the broker to both tools and pre-tool authorization. Service-account material is read from the dedicated token file and is not exposed by CLI readiness. Approval tests bind pending authorization to the actual call/agent/slug/nonce, join competing approval writes and consume one-use authority once. Existing worker/list grant tests already prove no main-thread SQL, corrupt/null/expired grants fail closed, and rejected bulk reads are not silently retried.

Signal produced no additional nonduplicate identity in this pass: its focused persistent reply-author test was already present in Round87. That prior test remains the oracle that failed persistence cannot erase retained memory and reopen does not fall back to parent-thread SQL.

New Slack/core identities:
- `extensions/slack/src/conversation-route-owner.test.ts@7bba749195b9f87d82d1cd0eb5e9bce744eaa916`
- `src/gateway/conversation-route-ownership.ts@eb9920c31651b11d5dee2f7d85400fe1c22a8b49`

Slack route ownership is now traced into the core conversation-route owner resolver. Thread ownership outranks parent ownership without touching liveness; workspace identity is rechecked after async binding inspection; released/degraded identity fails closed. Core routing composes configured bindings, plugin claims and conversation authority rather than creating a Slack-local second writer. Existing Slack binding and core worker tests already cover org-install capability removal and stale preparation/worker-authority fencing.

### 25.3 Zalo hosted media, Buzz and WhatsApp consumers

New Zalo identities:
- `extensions/zalo/src/monitor.ts@dd42f66fe0f183e895db397e7c12b8309a616a98`
- `extensions/zalo/src/monitor.polling.media-reply.test-support.ts@e03f5b8df1d3b16ad2e0f3b25767a67099b67c73`

The monitor reaches the already locked hosted-media owner only when hosting is enabled, carrying the account proxy and byte budget into preparation. Partial provider sends retain accepted provider identities so recovery cannot duplicate visible text/media. The prior outbound-media test remains the direct oracle: HEAD returns metadata without consumption, GET consumes once, wrong tokens/malformed ids fail before disk access, proxy-aware fetch is preserved, and invalid expiry fails before download.

New Buzz identities:
- `extensions/buzz/src/inbound.ts@95b757484319c8b207c9c55b0efd856114cbe24e`
- `extensions/buzz/src/buzz-bus.history-catchup.test.ts@1eb1311c86c20813aea4d89067daee4164756ef7`
- `extensions/buzz/src/buzz-bus.test.ts@7477be17d1bd8054d63a431b507caa196a1d2b79`
- `extensions/buzz/src/buzz-bus.lifecycle.test.ts@4830acd19877ec2f0225b22d8ebf7ca202c62528`
- `extensions/buzz/src/gateway.lifecycle.test.ts@69f0dd4838a08e75e2d4b205a22397864aea5c73`

Inbound policy waits for the current signed directory before dispatch and consumes only the history snapshot actually used by that reply. Catchup failure/timeout does not claim successful recovery. Bus close joins active presence publication; stopped buses cannot hand off new effects; removed members cease mention authority before a later signed-roster refresh; stalled lookup hands control back to the reconnect owner. Gateway lifecycle keeps only the live account bus and uses standalone sends only after retirement rather than reusing a retired owner.

New WhatsApp identities:
- `extensions/whatsapp/src/connection-controller.ts@0c2cacb79d9880f5c38448457973b39dc9e99e66`
- `extensions/whatsapp/src/connection-controller.test.ts@d159847825beb529957b39225578ea4b48f5c7f8`
- `extensions/whatsapp/src/monitor-inbox.socket-lifecycle.test.ts@416e88c66fd7d846c591f44c171a0a325bc10eb9`
- `extensions/whatsapp/src/auto-reply.web-auto-reply.connection-and-logging.e2e.test.ts@e243cf0ef5b514ce4b9e4cf358f632209018e29f`

Connection ownership is explicit: one controller owns the connection id/socket/listener/watchdog/reconnect attempt and acquires a Gateway connection-owner lease. Failed open closes the socket; login restart replaces the socket instead of mutating the old generation; socket-session replies created before reconnect use or wait for the replacement socket. The web monitor widens catchup only after an actually active reconnect, drains pending deliveries only while connected, and stops drain/retry work after close. The previously recorded state-migration test remains the credential-provenance oracle: missing/expired/conflicting/incomplete/orphaned originals are preserved until exact authority is recoverable.

### 25.4 X OAuth, OpenAI speech, LINE and IRC consumers

New X identities:
- `extensions/x/src/oauth.ts@13dee996eacb81d1455a84d931c7d346ffc44f21`
- `extensions/x/src/api.ts@11fa4387f0c9b94db37757410c6b2682d2b4edde`
- `extensions/x/src/api.test.ts@5ace0f7419f525a03706683674cf4f982156bc9c`
- `extensions/x/src/authority.test.ts@d247c071415cdd7671198118cf8eab6d95b2893f`
- `src/gateway/x-publication-owner-composition.e2e.test.ts@7c88828257c8db77bc806445d9bdbb42a15a0953`

OAuth has one in-flight refresh owner. Rotation persists the replacement refresh token before later requests; response loss is explicitly unknown rather than safe replay; current account authority is checked after async preparation and before final fetch. API mutation distinguishes explicit provider rejection from ambiguous 5xx/transport outcomes. Gateway composition binds public creation to the live publication owner and rejects revocation/owner retirement before commit. The prior client ownership test remains the direct reopen/retired-seed fencing oracle.

New OpenAI speech identities:
- `extensions/openai/speech-provider.test.ts@18c623ebe51bca85a3ee0a12c873cbf1cba2b28e`
- `extensions/openai/tts.optional-capture.test.ts@8954a2a7bb24655e4f9dc6703caf90e6fa090974`
- `extensions/openai/speech-runtime.contract.test.ts@86529ece3d9f404ba0305515cdff7573228d829d`
- `src/tts/tts-runtime-fallbacks.test.ts@f634af298723b01b706f635ac28b445082e53766`

The OpenAI provider is exercised through the public TTS runtime rather than only its local helper. Provider configuration/voice/model validation precedes dispatch; media byte caps reject oversized responses; hanging responses are deadline-bounded and socket teardown precedes watchdog settlement. Optional diagnostic capture cannot turn successful synthesis into failure, and fallback selection remains owned by the TTS runtime rather than by the OpenAI plugin.

New LINE identities:
- `extensions/line/src/webhook-spool-migration.ts@95ddb2e6603bfb872d83a05d28e291d4502e4674`
- `extensions/line/src/webhook-spool-migration.test.ts@6e381b11db0bad69474188c802f75913bd94da72`

The doctor-facing migration now maps to its actual queue transformer: stale legacy claims are recovered only when their payload is the recognized legacy shape, legacy decode dead letters are distinguished from identity-fence/delivery failures, canonical enqueue precedes retiring the legacy row, and one failed rewrite leaves that row pending for an idempotent later retry. Canonical failures are never resurrected. The prior doctor-contract test remains the host-capability/retired-account discovery oracle.

New IRC identities:
- `extensions/irc/src/irc-ingress.test.ts@34f914368af28749afb000ba86648054b42d8cea`
- `extensions/irc/src/inbound.behavior.test.ts@98d2d06f1b20002850d7fdcb0ce8d53fb3fa17c4`

Durable ingress append failure propagates instead of dispatching; interrupted queued work recovers after restart; pause/stop owns drain retirement; stop winning an async prune race cannot create a new drain. Inbound reply completion is transferred to the reply-lane adoption lifecycle rather than prematurely settled in the adapter. The prior monitor test remains the reconnect oracle that only the stopped account settles and a delayed reply cannot cross into the replacement socket generation.

### 25.5 Nostr, Device Pair and Beam consumers

New Nostr identities:
- `extensions/nostr/src/nostr-bus.ts@27f254eabc33ebbe9d7747d6f094af5b2384b43d`
- `extensions/nostr/src/nostr-bus.inbound.test.ts@1b274ce774341dfb9b79667dddaf586660564412`
- `extensions/nostr/src/nostr-bus.outbound.test.ts@569bc4c7e4f3d66bc40959e3864dd956bc96dfad`

The bus consumes account-scoped persisted state to derive a monotonic startup cursor, persists transient replay cursor before retiring relays on durable-admission failure, and clears legacy event-id seed only after tombstones exist. Close is memoized and joins relay subscription/pool plus ingress retirement. Outbound tests distinguish preparation from relay handoff, do not retry explicit refusal as if it were unknown, and settle an already handed-off acknowledgement without letting cancellation fabricate non-dispatch. The prior state-store test remains the account isolation/reopen oracle.

New Device Pair identity:
- `extensions/device-pair/notify-cas.test.ts@0b5a74077d979ee77420fd78b939d547a7ad64d1`

The CAS oracle proves a delivered one-shot arm is removed only if its exact observed arm is still current; changed metadata, a newly armed successor, or removal during delivery cannot be consumed by the stale completion. The previously recorded notify test remains the restart/service-recreation and persisted-seen-receipt oracle.

New Beam identities:
- `extensions/beam/index.ts@f1990dab6215e40ce1f26a0091a4c6e6a09487a3`
- `extensions/beam/src/mirror.test.ts@3aabb4d1269305e75dddf1b0134edd4eea98e095`
- `extensions/beam/src/mirror-retry.test.ts@84c6bfc44a4b95173652d5bd0a2e3d281f7622bc`
- `test/plugins/beam-mirror-secrets.test.ts@b249485bcd9cbcfaa8cec6c929f951ea28ba1b98`

Plugin composition wires one store/catalog service/HTTP receiver/mirror service. Mirror upload does not follow redirects to another origin, restart rechecks current endpoint/config, stop aborts in-flight work and waits for guarded response cleanup, and terminal retry rotates behind later sessions rather than monopolizing the queue. Receiver-retained retry state has independent clocks and eventually settles the exact target session. SecretRef preparation is resolved through the runtime snapshot; mirror tokens are not recovered from arbitrary ambient state.

### 25.6 Closure result

The remaining Section 23 extension families are now reverse-linked to production consumers and focused lifecycle/oracle evidence at the frozen source ref. No newly inspected production consumer imports or owns process-wide `closeOpenClawStateDatabaseAsync`; the direct calls encountered remain test/test-support teardown or explicit reopen/lifecycle oracles. Shared-state retirement therefore remains the previously established Gateway shutdown → agent DB retirement → global singleton drain sequence.

This evidence does not establish scoped G01/G02, Rust implementation/wiring, executable/package proof, independent ACCEPT, OCRM final acceptance, or release acceptance. No Actions/build/test/generator/package was run, and no production code, ledger, RTM or flags were modified.


## 26. Recovery continuation — Section 25 recursive owners, scheduler and SQLite-worker dependencies

This continuation stays on frozen `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`) and expands only identities that were absent from the live Round87 dossier immediately before this write. The already-recorded Section 25 identities are not duplicated.

### 26.1 Google Meet shared participation owner and effect boundary

New frozen identities:
- `extensions/google-meet/src/create.ts@2c8e8af24a8092ea6207af047ae463fc4e1a56c0`
- `src/meeting-bot/participation.ts@0aecefc1c4d1a76e34df18e8aa929df4df1358c4`
- `src/meeting-bot/participation-browser.ts@7eb54eab22be98eed4b1aff9633f96f71bfbf724`
- `src/meeting-bot/participation.capacity-race.test.ts@131901ce685fa0c94c9df751d828f03b9c55fd84`
- `src/meeting-bot/participation.test.ts@e5094c9174bd0143d681517f627911a79ac07628`
- `src/meeting-bot/session-runtime.startup.test.ts@d572f98a9c4fc2d80737a85ddc305799963f97df`

`create.ts` is a direct production consumer of the Google Meet runtime: create-and-join routes browser creation through the current `GoogleMeetRuntime` rather than opening a second meeting-session owner. The shared `MeetingParticipation` owner snapshots caller arguments before awaiting, claims a durable request row before effect dispatch, binds observed source authority to the current live session, and treats a pre-existing unfinalized claim as `uncertain` rather than replay-safe. Source action authority is single-use per action type and is invalidated by corrected/revised/expired/own-echo sources or by the live owner becoming non-current.

The browser participation executor rechecks current authority before and after tab discovery/preparation/evaluation and distinguishes pre-dispatch authority loss from post-dispatch uncertainty. Its negative oracle is therefore asymmetric by design: before dispatch, stale tab/session authority must reject without a visible action; after dispatch, an unreadable or interrupted result may be `uncertain` and must not authorize retry.

The focused tests add shared-owner evidence that was not present in Section 25: a capacity cleanup race cannot drop a sibling's newly admitted claim; durable request claims survive reopen and suppress duplicate effects; caller mutation after claim does not change the captured action; leave, browser-tab replacement, node/state changes, caption epoch changes, and source corrections revoke previously issued source authority. These tests also prove that replayed history cannot evict newer live source authority merely because the bounded source set is full.

### 26.2 WhatsApp connection ownership, active sender and transport-confirmed close

New frozen identities:
- `extensions/whatsapp/src/active-listener.ts@108272560277e1cb5a14687c95011b32d965fa3c`
- `extensions/whatsapp/src/send.ts@07eed1516361d772502685f37d55ed2ae007ae66`
- `extensions/whatsapp/src/connection-owner.ts@12636af7fffed1588262309c33bc06706090a72a`
- `extensions/whatsapp/src/connection-owner.test.ts@42ee95125522f66fca90f5e88152ebe50e372d16`
- `extensions/whatsapp/src/socket-close.ts@33c808561ad7f2afe347b293f628e9d739891b6f`
- `extensions/whatsapp/src/socket-close.test.ts@681058e61b18e1b558a0501949d0645d1efe468c`
- `extensions/whatsapp/src/directory-config.ts@a7273e11ab6a2d75a5029681e07d0f99409bb79b`
- `extensions/whatsapp/src/agent-tools-call.ts@8b213e2c6a88436a1c82a18f3372d441e334709d`

The direct outbound path resolves the current `WhatsAppConnectionController` and refuses sends when there is no active listener. Multipart sends report each accepted native send before the next fallible send, so later failure retains already accepted provider identities instead of pretending the whole mutation was rolled back. Agent-tool call identity resolution also reads the current controller/socket generation rather than caching a detached socket.

Connection ownership is serialized twice: a process-local owner prevents same-process re-entrant use of one auth directory, and a file lock prevents another process from opening the same auth-backed Baileys connection. Canonicalized real paths make symlink aliases the same owner. Gateway acquisition can wait for a bounded standalone lookup; standalone acquisition fails fast if the gateway already owns the directory. Shutdown aborts pending cross-process owner acquisition rather than allowing a late socket to appear after retirement.

`directory-config.ts` is the production standalone-consumer path: it acquires the standalone owner, opens the temporary directory socket, confirms socket close, verifies credential persistence has drained, and only then releases the owner lease. `socket-close.ts` treats Baileys `end()` completion as insufficient when the underlying WebSocket is still open; it waits for the close handshake or actively closes the transport and fails if closure cannot be confirmed. Focused tests prove process-local exclusion, symlink alias exclusion, shutdown cancellation, asynchronous close-handshake waiting, and failure when transport closure remains unconfirmed.

Negative oracle: an old socket generation or an incompletely closed standalone directory socket must never hand ownership to a replacement merely because a higher-level `end()` promise resolved.

### 26.3 X durable OAuth token owner — exact-ref recheck, no duplicate identity

The frozen ref was re-read for `extensions/x/src/client.ts@11f7bdfa3cf3168dfc8c54bdb1e7c1d6891127dd`, `extensions/x/src/oauth.ts@13dee996eacb81d1455a84d931c7d346ffc44f21`, and `extensions/x/src/api.ts@11fa4387f0c9b94db37757410c6b2682d2b4edde`; all three identities were already present in Round87/Section 25 and are therefore not re-registered as new rows.

The recheck confirms the prior owner conclusion: `client.ts` publishes the complete runtime owner before opening the `x.oauth` keyed store, then uses observe/compare-and-apply to claim a refresh token lineage, reconcile a committed write whose response was lost, and fail closed if the account owner changes before refresh or settlement. `oauth.ts` serializes one in-flight refresh and commits a rotated refresh token before publishing the access token. `api.ts` consumes only that owner and treats post-dispatch unreadable transport/provider outcomes as unknown rather than replay-safe.

### 26.4 Nostr durable ingress, cursor recovery and relay retirement

New frozen identities:
- `extensions/nostr/src/nostr-ingress.ts@470bf27e070efaef68620993bc28aa66dec74350`
- `extensions/nostr/src/nostr-cursor.ts@96f4a3a36d8e8110d4c0ab61e4a7b854de195ceb`
- `extensions/nostr/src/nostr-cursor.test.ts@4563daa18a6e4f56e97c89d39caaaef094b1eff4`
- `extensions/nostr/src/nostr-relay-subscription.ts@56376ca6d2efdba3bda6b7ef6cf25eb495cb5dfe`

`createNostrIngress` owns the durable channel ingress queue and its monitor. It validates/serializes before admission, maps permanent decode/identity failures to non-retryable outcomes, and on stop first prevents new receives, pauses delivery, then joins the monitor's accepted durable admissions. Legacy migration completion cannot restart a drain after stop.

The cursor owner separates durable append progress from transient replay safety. A transient event that failed durable admission caps the next replay cursor so later durable events cannot skip it. Cursor writes are serialized through one write tail, safety rewinds land after older progress writes, and `flushUntilSuccess` keeps retrying a failed safety write during retirement until the rewind is durable. The focused cursor tests prove a failed higher progress write cannot overtake a later rewind and that failed safety persistence remains pending until durable.

The relay subscription wrapper distinguishes actual EOSE frames from timeout/close synthesis. Caller-requested close marks local retirement so the same close callback is not misclassified as an external relay failure; incomplete relays cannot falsely confirm backfill completion.

Negative oracle: relay shutdown or transient admission failure must not advance a durable replay cursor beyond an event that was never durably admitted.

### 26.5 Beam receiver/store authority and catalog persistence

New frozen identities:
- `extensions/beam/src/http.ts@21ea1585f0333be2292f3e9158008d90c2c1a701`
- `extensions/beam/src/http-authority.test.ts@14e5807e815849dd5de54389ce6d97d7b2121674`
- `extensions/beam/src/session-catalog.ts@5584a34df69f8037044223e6d95af7ec6fd0a6ae`
- `extensions/beam/src/store.test.ts@63eb157151e43b4c1561d65324f2ec3607488c83`
- `extensions/beam/src/beam.test.ts@a8685c940f6ddbf5016a74dea6d74eec8a092d7d`
- `test/plugins/beam-http-identity.test.ts@01cfa4aedaa9baa66a66e6e9d56742f419e0e1a5`

The already-recorded Beam store is now traced through its direct HTTP receiver and catalog consumers. The receiver checks the current Gateway request scope before reading/committing the upload and revalidates publisher authority immediately before store mutation. Publisher profile identity comes from the authenticated request scope, never from the Beam payload. The store's compare-and-apply loop decides replacement against the currently observed revision; a competing writer causes re-observation rather than blind overwrite, and an anonymous replacement does not inherit a prior publisher identity.

Catalog reads expose the persisted uploader identity, use revision-bound transcript cursors, and reject stale cursors after content changes. Store/service tests prove background inventory refresh observes foreign writers, service stop prevents a late in-flight refresh from repopulating retired inventory, and competing upload revision must be rechecked. Receiver tests prove forged payload publisher identity is rejected, stale/equal snapshots do not refresh retained state, and operator-write authorization is required before reading the request body. The cross-server identity test closes/reopens plugin SQLite state and proves the uploader identity comes from the saved snapshot rather than a retained request context or the reader's current identity.

Negative oracle: a stale receiver request, competing upload, or reopened reader must never acquire authority to rewrite uploader identity or resurrect an older Beam revision.

### 26.6 Device Pair notifier scheduler recursion

The Device Pair notifier production identities themselves were already present in Round87 and are not duplicated. Its scheduler dependency adds these previously unregistered frozen identities:
- `src/plugins/service-scheduler.ts@c99f2b7ce3b2c81fcdbea8dcae6090a753762675`
- `src/plugins/service-scheduler-binding.ts@180c6cd40567b9e790c682217c5aed5d236d9ffe`
- `src/plugins/services.scheduler-context.test.ts@0dd382665e6b27d2ed1c805560b0d6ce997678e4`

Plugin service scheduling is scoped to a concrete service/account/CLI owner. Each scope has an abort signal, child scopes, and a close path that first begins close for parent/children and then joins scheduled work. The async binding refuses unbound scheduling and refuses a retired scheduler. The scheduler-context tests prove callbacks keep service ownership after the startup/reload caller is gone, old scheduler generations are aborted on reload, scheduling on a retired scope throws, siblings survive scoped stop, and scheduled callbacks settle before the disposal that stops the service.

This closes the Device Pair notifier's recursive scheduler ownership: its 10-second poll cannot continue on a retired service generation merely because a timer callback was already queued.

### 26.7 Memory-core SQLite-worker production entrypoints

New frozen identities:
- `extensions/memory-core/src/standing-intents.worker.ts@a27ce6a395c9292e6c4767f4f9ca3c71522242ca`
- `extensions/memory-core/src/memory/manager-publication.worker.ts@e87d19f14adc17c249659906172ca5ce74410de7`
- `extensions/memory-core/src/memory/manager-cpu-entrypoints.ts@d5900dfbb1050f92a385d5b1344cd6e70d7ded8b`
- `extensions/memory-core/src/memory/manager-publication-transfer.test.ts@309feec35e1efc223fc9cb4720c05c001fb9c2e9`
- `extensions/memory-core/src/memory/manager-publication-crash.test-support.ts@fbfffc4483e08210acfde77727f9962f2d59b539`

The CPU entrypoint map proves these are packaged production worker identities, not test-only filenames: `manager-publication.worker` maps to `memory-publication.worker.js`, while `standing-intents.worker` maps to `standing-intents.worker.js`. Standing intents binds the canonical agent executor's SQLite connection and explicitly leaves retention/close ownership with that executor; commands run under admitted worker search/schema/transaction boundaries instead of a second main-thread writer.

Memory publication has two distinct ownership modes. Agent publication borrows the executor connection and does not own its close; private shadow publication opens its own SQLite connection, configures WAL maintenance, validates file identity/connection policy, and owns stop/close for that private shadow. Every operation is wrapped by SQLite worker admission and scoped write-token checks. The transfer tests prove connection policy survives scalar and staged transfers, declined preparation can reuse a healthy publication owner, concurrent large transfers remain isolated by publication scope, and an original failed publication outcome is preserved even when close itself also fails. Crash support deliberately kills the process before publication commit to exercise recovery rather than treating process death as a successful mutation.

Negative oracle: worker retirement, publication failure, or a failed cleanup must not transfer a borrowed executor connection into a second owner, and private-shadow cleanup failure must not erase the original uncertain/failed publication outcome.

### 26.8 Extension shared-state close / agent-close / snapshot-cleanup dedupe result

A fresh 100-cap extension scan of `closeOpenClawStateDatabaseAsync` and `closeOpenClawAgentDatabasesAsync` found the previously recorded Codex, Feishu, Twitch, ACPX, Reef, Google Vertex, Diffs, Microsoft Teams, Google Meet, iMessage, QA-lab, and memory-core test/support identities; every direct close identity from those result sets was already present in Round87. Extension searches for `cleanupSnapshotOperations`, `sqlite-snapshot-source`, and `sqlite-readonly-location-cleanup` produced no extension production caller at the frozen ref. No duplicate close identity is added here.

The new SQLite-worker evidence above therefore changes caller coverage, not process-wide close ownership. No newly inspected extension production consumer owns process-wide `closeOpenClawStateDatabaseAsync`. Shared-state retirement remains the previously established Gateway shutdown → agent DB retirement → global singleton drain sequence.

### 26.9 Closure status

This pass adds nonduplicate production-consumer, scheduler, durable-ingress and SQLite-worker evidence only. It does not establish scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, independent ACCEPT, OCRM final acceptance, or release acceptance. No Actions/build/test/generator/package was run, and no production code, ledger, RTM or flags were modified.
