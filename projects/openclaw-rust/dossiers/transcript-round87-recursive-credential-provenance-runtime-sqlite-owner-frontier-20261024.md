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


## 27. Recovery continuation — worker lifecycle, meeting owner, WhatsApp socket generation and Beam Gateway consumer closure

This continuation remains fixed to `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Immediately before publication, every identity below was absent from the live Round87 dossier. Prior Section 26 identities are referenced only to establish the already-locked parent owner and are not duplicated.

### 27.1 Memory-core worker acquisition, generation replacement and reindex retirement

New frozen identities:
- `extensions/memory-core/index.ts@1870c11d40ffc7fa5bf54b6bbef652927bb2acfd`
- `extensions/memory-core/src/standing-intents.ts@d25cbe9db611b909e33f14a476dd3cde9b9f24ef`
- `extensions/memory-core/src/memory/manager-database-context.ts@3c91b86fcb4ddb4c8e3876ba2609c242ef0a033b`
- `extensions/memory-core/src/memory/manager-publication.ts@10579e9f7daf1eeb7c47101d64c2e49a74c6d304`
- `extensions/memory-core/src/memory/manager-close-observation-lifecycle.test.ts@be49a2659fa3cd23a5e6a34751d2b0331d872439`
- `extensions/memory-core/src/memory/manager.ts@e36781380a027f2fd18f8eeb0acb52c809a79616`
- `extensions/memory-core/src/memory/manager-shadow-owner.test.ts@6e841d20193f5a9a00f4065c008beea8e0b470f2`
- `extensions/memory-core/src/memory/manager-shadow-write.test.ts@fbeb7511aac23d2e780e93880ef4ab5246acc14f`
- `extensions/memory-core/src/memory/manager-publication-generation.test.ts@44afa0a1890cd5cfd71c0d498be1472a8a7ad2ea`
- `extensions/memory-core/src/memory/manager-reindex-ops.ts@b9afc419f8b55c33bb270cabef17aa2ef4f083f5`

The plugin entry lazily loads standing-intent code only for the service/tool/turn hooks that need it. The standing-intent service passes the plugin service scheduler into `startStandingIntentMaintenance`; every create/list/cancel/match/sweep operation is serialized per database path, opens the canonical agent SQLite worker through the packaged standing-intents entrypoint, prepares it, executes under the caller's current-authority assertion, then closes the worker in `finally`. Maintenance restart does an immediate sweep before trusting a retained deadline; service retirement clears the global schedule callback and cancels all outstanding jobs. A retired scheduler cannot mint a new sweep.

`MemoryIndexDatabase` is the publication ownership boundary. Published agent state uses the live agent-database executor/borrow; a private reindex shadow instead opens its own publication worker and read-only native handle. Worker open is fenced by the current database owner and source file identity. A failed open clears only the failed publication-worker promise so a later current owner may retry; it does not hand ownership to a second writer.

Publication calls retain an agent-database execution lease across fallback preparation. No-op/refused preparation can complete without opening a worker. Large source writes retain a bounded staged transfer scope; thrown publication failures are cleaned by the host owner rather than issuing a later command that could obscure the original write outcome.

Manager sync generations explicitly close the current publication worker before replacing provider/reindex generation. Reindex builds against an isolated shadow, closes the shadow writer before publication, enters the index generation write lease, and rechecks that the published database object, file identity, and owner are still current before attaching the completed file. A changed owner makes publication fail rather than publishing the stale shadow.

Focused lifecycle tests add the negative oracles: shadow close joins its retained worker before releasing its read-only handle; a worker close failure does not discard already committed data and remains retryable; manager close drains accepted staging before releasing the database; a publication result failure and generation-cleanup failure are both preserved; a tombstone written after staging but before publication rejects the stale shadow; no-op/refused preparation is SQL-free and does not open a worker; observer/provider retirement failure cannot skip publication-worker join or database-lease retention.

### 27.2 Google Meet shared session owner and browser participation currentness

New frozen identities:
- `src/meeting-bot/session-runtime.ts@07c917cee80539e4c12900044fd6688f35ff2436`
- `src/meeting-bot/participation-browser.test.ts@d4fcfd91e8513c38240c44b2e11b8e41cdd96631`
- `src/meeting-bot/observation-provenance.ts@6df9b018a5e12280f49c1d82bf0a67ba0324114d`

The shared `MeetingSessionRuntime` is the live session owner used by the Google Meet subclass. It holds sessions, cleanup handles, transcript state, and the Section 26 `MeetingParticipation` owner. Participation currentness is captured from the exact session object plus browser tab/node/route identity. Leave and external end close participation immediately before terminal cleanup, so an admitted claim waiting on storage cannot regain authority after retirement.

Transcript capture also snapshots browser ownership. When participation is enabled, capture requires the same tab; after async capture it rechecks the owner before accepting lines. Observation provenance copies only bounded validated scalar facts; invalid envelopes cannot donate participant identity or own-echo authority, and caller-owned mutable provenance is never shared as live authority.

The browser focused tests prove: browser-lock queuing rechecks authority before dispatch; stale authority during tab check or preparation is rejected without a visible effect; authority loss at or after native dispatch is `uncertain`, not safe retry; response loss after dispatch is `uncertain`; and an unavailable browser before dispatch is an ordinary failed non-dispatch. This preserves the distinction between “not sent” and “possibly sent”.

### 27.3 WhatsApp socket-session generation, inbound drain and delivery recovery

New frozen identities:
- `extensions/whatsapp/src/inbound/socket-session.ts@0f62f1651321635159ecdd7548daad17f83d1f44`
- `extensions/whatsapp/src/inbound/monitor.ts@25100d7ed0d43c7b1dcdd5616427595187f69e68`
- `extensions/whatsapp/src/inbound/message-delivery.ts@fd68decd72c04504b8920ac4d9e71294814e650e`
- `extensions/whatsapp/src/directory-config.test.ts@9fc3bd9a5414569ac777a62111a2e525a2f6820b`
- `extensions/whatsapp/src/send.delivery-recovery.test.ts@4671222144775802e8f03e1077d27152eae4d6ab`
- `extensions/whatsapp/src/send.test.ts@8a0dcb7531c6f5f1bff4720a4c9b2636a201e77e`

An attached socket session publishes its socket into the controller-owned socket reference, then resolves a current socket on each operation. A captured reply may adopt a successor only when the successor controller has an authenticated identity overlapping the original session; otherwise it returns no current socket. Native send operations bind an exact current-socket assertion. Disconnect-like failure can clear only the still-current socket ref, and retry waits for a replacement while keeping one logical Baileys message identity across attempts.

Reachout-timelock readiness is checked on the current socket before direct-message dispatch. Only the preflight path can raise `PlatformMessageNotDispatchedError`; after an ambiguous native send, a later readiness failure cannot rewrite the prior attempt as definitely not sent. Late accepted native sends are observed only to suppress self-echo and are not converted into a synthetic retry grant.

`inbound/monitor.ts` composes the socket session, delivery coordinator, group-metadata owner and send API. Close stops intake/listeners/cache ownership, drains accepted inbound work, and only then closes the socket. The delivery coordinator uses the socket session's generation-aware getters for replies/read receipts, retains the first accepted delivery preparation across redelivery, and prevents a successor from sharing an account queue with a still-live owner after close-drain timeout.

Focused tests prove the standalone directory path releases ownership only after socket close and credential persistence drain; owner contention does not open a second socket; setup failure closes the created socket. Send recovery preserves every accepted chunk/provider receipt before a later failure or cancellation, pre-connect failure remains replayable, and the eventual connected recovery sends once. Ordinary transient outbound failure is not automatically retried at the top-level sender, preventing duplicate visible sends.

### 27.4 Beam store to Gateway session-catalog authority and service lifecycle

New frozen identities:
- `src/plugins/session-catalog.ts@7f62db797112b56d9d91e298e6a49e21cd528dac`
- `src/gateway/server-methods/session-catalog.ts@88330393acc726c9df5fe918e1637cc91532bf18`
- `src/gateway/server-methods/session-catalog-gateway-copy.ts@a7059404762308ec2dc4cc948c268a306443e74e`
- `src/gateway/server-methods/session-catalog-visibility.ts@966b3433fdce858d8ed8751c37f762da0a4a15f8`
- `src/gateway/server-methods/session-catalog-read.ts@b1a5d71206baa43244a0c22969bd8ae066b8de7e`
- `src/gateway/server-methods/session-catalog-list-operations.ts@2e8c79f8a4f71841f5646d8416ecaad06546e3ec`
- `src/plugins/services.ts@3c16dd3ce7d522272d6c3c21148eb347a2359368`
- `src/gateway/server-methods/session-catalog-visibility.test.ts@b2495f2584fb8923359e0cea30275af7bcee84f1`
- `src/gateway/server-methods/session-catalog-gateway-copy.test.ts@ed9ebbef716209d4963ab7dac54ddff43cfadd32`

The Beam catalog is consumed through the generic typed session-catalog contract rather than a Beam-specific Gateway bypass. Provider list snapshots are logically frozen for one request; provider list operations carry catalog-owner lifetime separately from a requesting connection's lifetime and can be retired by config/gateway/connection generation. Cached pages never donate new adoption authority: delivery rechecks current visibility and canonical session state.

Gateway catalog methods resolve the registered provider, authorize source visibility before provider dispatch, and recheck current policy after async provider enumeration. For catalog mutation, `operator.write` and session-creation policy are authorization boundaries; a commit guard is propagated through adoption/upstream-link/state-event writes. Provider source IO that outlives a caller's role grant cannot release data unless current policy still permits it.

Beam's `copyToGatewaySession` path therefore becomes a fresh Gateway-owned session, not a continuation of Beam authority. Imported catalog content is explicitly labeled untrusted reference material that cannot authorize actions or inherit source machine/tools. Preferred source-model selection is separately policy checked; model-catalog waiting shares the operator/gateway abort signals, and an unavailable/restricted source model falls back to the Team agent's configured model without granting extra authority.

The generic plugin service owner is also the physical lifecycle owner for Beam's catalog service and mirror service. Service stop begins scheduler/node/control-UI closure before invoking service cleanup, joins scheduler work, and retains cleanup custody across timeout/reload rather than starting a replacement while the old generation still owns resources.

Focused tests prove foreign/unprofiled catalog targets are rejected before provider mutation dispatch, Gateway-published Beam rows obey operator visibility, catalog adoption cannot create a disallowed agent session, and Gateway-copy model assessment is bounded before session creation. Copy tests also show history import and adopted-state publication happen only after the new Gateway session exists and use the current commit guard.

### 27.5 Closure result

The four requested Section 26 recursion fronts are now tied one level deeper into their actual production owners and negative oracles. Memory-core worker packaging now reaches host acquisition/generation/reindex retirement; Google Meet participation reaches the shared live session owner and browser-effect currentness; WhatsApp controller ownership reaches the socket-session/inbound-drain/send-recovery generation rules; Beam store/catalog reaches Gateway authorization, copy and plugin-service retirement.

No new evidence changes the prior process-wide SQLite conclusion: extension-local workers and plugin state owners do not become owners of `closeOpenClawStateDatabaseAsync`. No Actions/build/test/generator/package was run. No production code, ledger, RTM or flags were modified. Scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, independent ACCEPT, OCRM final acceptance and release acceptance remain absent.


## 28. Section 27 next-owner recursion: retirement, durable delivery and catalog custody

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This section records only identities that were absent from the live Round87 dossier when this pass began. Previously registered X OAuth/runtime authority, Device Pair notifier state/CAS, Beam store/mirror/session-catalog, WhatsApp connection-controller/socket-session, and Nostr cursor/bus/state-store identities are intentionally not duplicated.

### 28.1 Memory provider retirement and PostgreSQL staging custody

New frozen identities:
- `extensions/memory-core/src/memory/manager-postgres-reindex.ts@532277877c30c9313c2d3970b3d46f6073e3b6ff`
- `extensions/memory-core/src/memory/manager-provider-lifecycle.ts@1adcaed8d4f4afc840d3e21e6a48f9334491adbb`
- `extensions/memory-core/src/memory/manager-provider-lifecycle-fallback.test.ts@1a07ed9ce76d3f2e0ca3097f3fb1fc1a131e8890`
- `extensions/memory-core/src/memory/manager-provider-lifecycle-leases.test.ts@cbc4baee055e61ea69ca3e8335ac77e7b1582f9a`
- `extensions/memory-core/src/memory/manager-provider-lifecycle.test.ts@9cd748303061328f60bd256517f4cf474359140e`

`MemoryProviderLifecycle` serializes provider retirement through `providerRetirementPromise` and a retained `providersPendingRetirement` set. Replacing the current provider first removes it from new admission, then retirement waits for admitted users through `awaitProviderIdle` before invoking the provider's close operation. A close failure leaves that provider pending, preserves the first retirement error, and prevents a fallback/retry provider from being opened through the same lifecycle until retirement is retried. The focused tests show that fallback activation and retry initialization do not race a previous worker shutdown, an admitted provider user blocks retirement, a leased runtime remains valid for its already-admitted operation, and manager teardown waits admitted search/vector-probe/index-generation work before provider close. Required configured providers fail closed when fallback cannot initialize; optional paths may remain in bounded FTS-only recovery instead of inventing a second embedding authority.

PostgreSQL reindexing keeps the published owner distinct from a private staging database. `withPostgresMemoryReindex` starts the stage only under the still-current published owner, always closes the private stage in `finally`, and discards the stage only while the published database is still open and its currentness assertion passes. `publishPostgresMemoryStage` requires the retained stage identity plus the expected revision. A retired or replaced publication owner therefore cannot donate stale stage writer authority.

### 28.2 Google Meet cleanup, durable transcript owner and dynamic bridge

New frozen identities:
- `src/meeting-bot/session-cleanup-tracker.ts@1793332dde2610754d053d0b9960893fbcf9d8ae`
- `src/meeting-bot/session-durable-transcripts.ts@00527dd670b4c5c38b4bb09924c107530ccba08f`
- `src/meeting-bot/session-runtime-browser-tabs.ts@85dfaf9d230e2f3030b68269259284609bd8d05a`
- `src/meeting-bot/session-runtime.test.ts@7128288366d501d259cfbc20272d62303f9772de`
- `src/meeting-bot/session-transcript-store.ts@b33bed17d154fa71a79d9a01822514516cec6185`
- `src/meeting-bot/session-transcript-store.test.ts@ac13530b720c904dca72d144cb823b37d0ac939b`
- `src/meeting-bot/transcripts-bridge.runtime.ts@7ad966bec17d753f23ed2e3020b2f9197c5468c7`
- `src/meeting-bot/transcripts-bridge.ts@a2c03644b240252a94b48507cfd03656289d2d2b`
- `src/meeting-bot/transcripts-bridge.test.ts@d367ff4bc9ed7e10f0b3b3030497fa74c0d39cb9`
- `src/meeting-bot/transcripts-bridge.provenance.test.ts@0368243142e57e75ebf506f297ae9c22fd3bd55a`

The durable transcript runtime is a recursive dynamic dependency: `MeetingSessionDurableTranscripts` lazily imports `./transcripts-bridge.runtime.js`. The cleanup tracker installs setup custody before invoking callbacks, drains every adopted stop handle even when startup failed, and removes a stop only after that stop succeeds. Terminal cleanup records transport/runtime settlement and browser settlement independently; a successful half is not repeated merely because the other half failed, and the state is released only after both halves settle and no runtime handle remains.

Retained browser tabs are transferable only when transport, meeting URL, node, target ID, and prior plugin-owned-tab provenance match. A retained entry is consumed only after successful adoption or browser release; failed replacement cleanup keeps the remaining entries available for another settlement attempt and does not replace the original join failure.

Durable transcript policy changes are serialized. A failed bridge finalization creates token-scoped retry custody with exponential backoff capped at 60 seconds; a stale retry token cannot retire a successor. The transcript store serializes captures per session and acknowledges durable delivery incrementally, so a later line failure does not erase already delivered lines and pending rows remain flushable. Epoch/gap handling advances the retained cursor without replaying prior source rows.

The dynamically loaded bridge owns the durable `TranscriptsStore`, per-session lifecycle queue, and subscriber delivery queue. Start and stop share the same lifecycle key. Stop marks the exact capture closing, clears polling, drains an in-flight periodic capture, retries `MeetingTranscriptDeliveryError` final delivery up to three times, persists the stopped descriptor and summary under an exact active-capture currentness assertion, and only then retires that capture. Attachment replay rechecks the same active capture and refuses attachment once closing begins. Detach is queued behind the meeting owner and consumes only the same subscriber object, so a delayed detach cannot remove a replacement attachment that reused the same public session ID.

Focused tests prove that failed subscriber delivery detaches only that subscriber while durable rows continue, finalization rejects late attachment, detach drains admitted delivery, terminal status delivery does not block meeting finalization, and provenance is cloned for subscribers so subscriber-local mutation cannot rewrite stored observation authority. Meeting leave/failed-join tests also preserve settled cleanup progress across retries instead of repeating already-settled side effects.

### 28.3 WhatsApp durable inbound drain and reconnect metadata ownership

New frozen identities:
- `extensions/whatsapp/src/monitor-inbox.delivery-and-dedupe.test.ts@b65ca80dfd38ad62a3a61dd444027cc37cd3e4d1`
- `extensions/whatsapp/src/monitor-inbox.metadata-cache.test.ts@3d77e77f9daa8ceff32a2e461354551fca418d4b`

These tests deepen the already-registered connection-owner/socket-session/message-delivery production path rather than introducing another socket owner. The durable queue delays read receipts until accepted inbound handlers settle, keeps a duplicate transport delivery behind the first durable claim, does not redispatch a completed transport-key duplicate, and lets a ciphertext stub be replaced by the later decrypted message without producing a second logical turn. A transient queue persistence failure such as `SQLITE_FULL` is absorbed only by the bounded durable-enqueue recovery path.

Monitor close waits already-admitted same-lane turns and serialized replies before socket close; after a successful drain, durable claims and pending rows are empty. Explicit retryable turn failures and reply-session conflicts retain durable retry custody, while this does not weaken the Section 27 rule that an ambiguous ordinary outbound send is not blindly replayed.

Reconnect metadata is a cache/projection, not a second participant authority. Narrow group facts can survive reconnect, but hydrated participant identities do not outlive provider expiry; membership changes invalidate them. A live metadata fetch completing after the owner closes is never republished, listeners are detached on close, and reconnect entries are bounded. Thus a retired socket/cache generation cannot republish participant authority into its successor.

### 28.4 Nostr ingress admission, relay retirement and generation tests

New frozen identities:
- `extensions/nostr/src/nostr-ingress.test.ts@f5dfd7a39ec1e71ce13ea492a36e441729b19109`
- `extensions/nostr/src/nostr-relay-subscription.test.ts@8d5ae303693a79e15c694b7a11616ac8dc11c114`
- `extensions/nostr/src/channel.outbound.test.ts@46566e04181870ce136760dd97ff0fe46ec5fa79`
- `extensions/nostr/src/nostr-bus.admission.test-support.ts@0d04e7208733718856a51f1fa64ac266a5cf4edc`

The already-registered Nostr cursor/ingress/bus owners are now backed by focused admission and retirement oracles. Durable ingress advances its cursor only after the exact raw event has entered the sender lane. An interrupted, uncompleted relay event is recovered by a fresh drain exactly once. Malformed or cyclic events are isolated from later admissions, while the first durable append failure is latched across already-queued admissions rather than allowing later events to leapfrog an unknown persistence outcome.

Ingress rate/backlog limits are checked before expensive work, and stop quiesces delivery before waiting for an in-flight durable admission; after stop, new receive attempts fail with the stopped-owner error. Relay backfill is confirmed only after every relay reports a real EOSE. Timeout-synthesized EOSE is not treated as confirmation, and same-turn relay close can veto synthetic completion. Local abort/close is not reported as a remote relay failure, while actual relay closure is.

Gateway-account tests retire the old bus before shutdown settlement so new sends cannot target the closing generation, while a newly installed replacement bus remains current even if the prior close completes later. Signed admission tests also show that denied/forged events do not reach the durable queue and a matching literal author string is not sufficient authorization. The state-database close calls appearing in these focused tests remain test cleanup seams; they do not make Nostr a production owner of the process-wide state database.

### 28.5 Generic Gateway session-catalog admission, lifetime and import commit guards

New frozen identities:
- `src/gateway/server-methods/session-catalog-import.ts@94af949aeb008661826747d26603a44ce6c295d6`
- `src/gateway/server-methods/session-catalog-list-admission.ts@dd9c711f965edb11c6758e8403830abf1a266b15`
- `src/gateway/server-methods/session-catalog-list-lifetime.ts@803a5d37f633dd99badf096891657fbca6b6ab80`
- `src/gateway/server-methods/session-catalog-provider-access.ts@25c54973b5fcbbe51dec46d4533df675942f8c4b`
- `src/plugins/session-catalog-history-import.ts@6c11d31cad33beded9c3e3d8f71541dd8e0f2d`
- `src/gateway/server-methods/session-catalog-provider-access.test.ts@b0eb299eecfe052d83b1f77f86acd931b876af15`
- `src/gateway/server-methods/session-catalog-list-admission.test.ts@a1b732e281ad3c57f92d5d66f0423e6476414fc6`
- `src/gateway/server-methods/session-catalog-list-lifetime.test.ts@78b3108b6511731c8c4c8a6240207cdcfcf6e1e0`
- `src/gateway/server-methods/session-catalog-list-retention.test.ts@44c6e815b1605005dcbcb99efed10364391b02cd`
- `src/gateway/server-methods/session-catalog-import.test.ts@f520a40b85e4430af8b0fb138b16c64704c43e80`
- `src/gateway/server-methods/session-catalog-import.integration.test.ts@fc5b66c7ea211f0ab639c2355ae616bc68dd4460`

This is the next generic consumer layer beneath the Section 27 Beam catalog mapping. Listing has a bounded global admission owner (16 active, 32 queued). A stepped provider operation holds its admission slot until that operation ends and checks cancellation/currentness before each next step. Provider access captures the plugin instance consumer, refuses further steps if the instance stops accepting calls or its owner is revoked, closes the stepped operation in `finally`, and releases the retained plugin consumer only after registered `waitUntil` completions settle.

The list lifetime intentionally separates response/delivery lifetime from native-work custody. Connection/gateway/deadline retirement can stop subscriber delivery while an already-admitted native completion still retains Gateway root work until it settles. Focused tests show queued callers retain their original plugin scope and Gateway root; retired queued callers are released without ever starting the provider; an abort lets the already-started page settle but prevents the next page; 1,000 reconnects do not leak provider/listener state; and delivery expiry cannot release native work that ignores cancellation. Pending provider keys are bounded and an evicted operation's late page cannot republish.

Import reads a bounded source history, then reauthorizes the source before destination creation. It composes the source-current assertion with the destination session mutation guard and carries that guard through Gateway session creation, history preservation, and the imported state event. Destination authorization is released in `finally`. History import removes raw provider payload, bounds items/bytes, detects repeated cursors, uses stable idempotency identities, and wraps imported text as external/untrusted content; imported history cannot inherit the source machine or tool authority.

Unit and integration tests prove source sharing/custody revocation at the creation commit or during re-import append blocks the remaining mutation, stalled copy cancellation can leave no durable artifacts and then succeed on a fresh retry, ordinary durable session/projection/transcript owners handle re-import idempotently, and existing-target write plus draft/shared visibility remain enforced.

### 28.6 De-duplication and closure result

A fresh capped search of `closeOpenClawStateDatabaseAsync` found extension test/support callers in Codex, Feishu, Twitch, acpx, Reef, Google, diffs, Teams, Google Meet and iMessage; every returned extension identity was already present in the live Round87 dossier before this append. No new extension production consumer was found that owns the process-wide state-database close. The process-wide ownership conclusion therefore remains unchanged: extension-local workers, durable queues, transcript stores, channel buses, and catalog providers retain their own scoped cleanup custody but do not replace the existing Gateway/agent/global database retirement chain.

X OAuth/runtime authority, Device Pair notifier scheduler/state CAS, and Beam store/mirror/catalog production identities were also rechecked against the live Round87 text and were already registered, so they are not duplicated here.

No Actions, build, test, generator, package, production-code, ledger, RTM, or flags mutation was performed in this pass. Scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, independent ACCEPT, OCRM final acceptance and release acceptance remain absent.


## 29. Section 28 next-owner recursion: generation leases, canonical transcript workers and durable channel ingress

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Every identity below was absent from the live Round87 dossier immediately before this append. Previously registered generic SQLite worker admission/store primitives, Nostr relay-subscription, and Section 28 owner identities are not duplicated.

### 29.1 Memory provider-use and published-index generation ownership

New frozen identities:
- `extensions/memory-core/src/memory/manager-embedding-ops.ts@24277e52f58227107e198d93d238307ea04b265c`
- `extensions/memory-core/src/memory/manager-search-orchestration.ts@0b3ff669ac46a9ed3e28d992b36f6984ad733627`
- `extensions/memory-core/src/memory/manager-sync-ops.ts@1b0d823a9268bd486bebb7f1824dabb12a4f5814`
- `extensions/memory-core/src/memory/manager-index-generation-lease.ts@49844fc63668f63d5454214976cd0a3f8d690e02`
- `extensions/memory-core/src/memory/manager-index-generation-lease.test.ts@bcc739f391c3d1add7fb32a76dfdc029b6ed10ff`

The provider lifecycle from Section 28 is backed by explicit admitted-use custody. `manager-embedding-ops.ts` records active uses per concrete provider, gives each admitted operation an idempotent release, and makes retirement wait `awaitProviderIdle(provider)`. A sync generation captures the exact provider, runtime and provider/index identities and holds one provider-use lease across embedding plus publication; nested sync users share that captured generation and release it only when the final owner exits. Semantic search likewise retains the exact provider during its embed call and releases it in `finally`.

`manager-search-orchestration.ts` separately leases the published index generation. Search releases its read generation before bootstrap, repair or sync can replace the index, then reacquires a fresh generation before continuing. This prevents a search from combining an old database generation with a newly selected provider. Cancellation does not itself degrade the provider, and fallback is attempted only after the old provider-use lease is released.

`manager-sync-ops.ts` closes the same loop from the writer side: fallback activation is serialized, the current provider is retired before a fallback provider is constructed, and a sync that detects provider failure explicitly ends its own sync generation before fallback activation so it cannot deadlock waiting on itself. An explicitly required semantic provider fails closed instead of publishing an FTS-only replacement over an existing semantic index when fallback cannot be established.

The generation lease is keyed by normalized database path. A queued writer waits for all admitted readers; queued readers behind that writer do not leapfrog it. Abort removes only a still-queued waiter, while an already admitted reader continues to own its generation until explicit release. Focused tests prove a pre-aborted reader is refused, a canceled queued reader does not admit its successor ahead of publication, and a signal that aborts after reader admission cannot release the generation early.

### 29.2 Meeting transcript canonical state-worker, summary and capture finalization chain

New frozen identities:
- `src/transcripts/store.ts@edf0bd6a943fd11ed814a0f67eeaba7f6d23903c`
- `src/transcripts/capture-summary.ts@c72d3f11dd320b010b4a4d97d502de78f4511ae7`
- `src/transcripts/store-errors.ts@e8dba1f92e6a29ab9a1e3e90216b55603ab0023a`
- `src/transcripts/store-worker-client.ts@132dfb7fea390d1d28d5bd9123ebad93b86b029e`
- `src/transcripts/store-worker-contract.ts@050c1ccedd8fb0c7435a2dd32b9d3a10ef13c02c`
- `src/transcripts/store-write.worker-contract.ts@17505611211a398b61d255dd7492a9246c0ea347`
- `src/transcripts/store-worker.types.ts@6474f826170169d8b4882ffb96f1547b48d82082`
- `src/transcripts/store-worker-write.ts@cd89a70c5388f10db6dd10a11c80d1ecd1d126c8`
- `src/transcripts/store-worker-read.ts@ece97ebc0e2eec6983b427ea8153f02cd145d124`
- `src/transcripts/store-sqlite-write.ts@49da9ad3d91420c1dacc0d1bd953f42bf54e9dc4`
- `src/transcripts/store-sqlite.ts@23530d17b919be17a4f2a2927a7803f52a7f4b93`
- `src/state/openclaw-state-worker-registry.ts@a29e1cb519ac3dfbcaabb84dbeab64bc395f6025`
- `src/infra/sqlite-worker-transcripts.test.ts@8c520c1d36885eeaaaeafac212c376b7efb3cee8`
- `src/transcripts/capture.ts@10f1d0101a2a62c20bea76fb9e5d894938b2aece`

`TranscriptsStore` is the canonical meeting-capture store; filesystem files are explicit exports, not a second durable source. One `createTranscriptStoreOperation` captures the state-database worker generation across planning and persistence. Ordinary reads and writes go through that captured state worker, while export artifacts additionally require an exact `meeting-transcript.export` lease.

The worker registry dynamically loads both `store-worker-read` and `store-worker-write` under the single `transcripts` worker namespace. Every session, utterance, summary and empty-candidate mutation in `store-worker-write.ts` executes inside `runOpenClawStateWriteTransaction` and rechecks worker admission at both the transaction and commit stages. Export-manifest mutations substitute an exact lease assertion whose scope and session export key must match.

The SQLite writer enforces the durable identity inside that same writer boundary. Session updates check expected input/session revisions and canonical selector ownership; admitted `sessionIdOrigin` cannot be rewritten later. Summary publication validates the current transcript input revision, utterance sequence, prior summary revision and stopped/append state before replacing notes. Utterance append performs exact-ID dedupe and allocates/increments the sequence in the same database operation. A retry owner may delete only its unchanged, stopped, empty admission with no utterance, summary or export ownership.

`capture-summary.ts` adds a per-session summary lane with a generation counter. Replacing a live summary owner first stops the prior owner; retirement aborts active inference, increments the lane generation, and drains the pending lane. A late inference or summary write therefore raises `TranscriptsSummaryChangedError` instead of overwriting a successor.

`capture.ts` is the outer production capture owner. Finalization first drains accepted appends and summary work, then rechecks the exact `activeSessions` owner, writes the stopped descriptor and final summary, and only after persistence settlement releases the provider. Failed startup retains cleanup and restoration custody instead of admitting an unseen replacement provider.

Focused worker tests prove cold and warm transcript access stays off the caller-thread SQLite writer, exact utterance IDs deduplicate with stable sequence order, stale session/input revisions and selector conflicts fail with typed errors, and—critically—retiring the owner at either the worker transaction or commit grant for discard, session update, or summary update rejects the mutation while preserving the previously committed data. Test-only process database close helpers remain cleanup seams, not meeting production ownership of process-wide close.

### 29.3 WhatsApp durable receive through the shared channel-ingress worker

New frozen identities:
- `extensions/whatsapp/src/inbound/durable-receive.ts@a0275d9f0ab71c52e961ea77ece9d93ba19d47a8`
- `extensions/whatsapp/src/inbound/durable-receive.test.ts@70c6ff8c2bd2ddac958921ff1aa73645a120914c`
- `extensions/whatsapp/src/inbound/ingress-lifecycle.ts@1c978aa68060e2afc3ff34c451165e99da8c04e7`
- `extensions/whatsapp/src/inbound/message-debounce.ts@40028399b7d94e2407e0583722a00e908a90deef`
- `src/channels/message/ingress-queue.worker.ts@66e8717cd3b20711681d60fc165e44e8fc25d7f3`
- `src/channels/message/ingress-monitor.ts@08885501ce6b528d87a235bc4b38aa377c080f28`
- `src/channels/message/ingress-monitor-admission.ts@f87da19b2e7574e15bfce9da825214f558819397`
- `src/channels/message/ingress-monitor.test.ts@c4ca63f653e18bd5336ca92f92d1ba570a10b336`
- `src/channels/message/ingress-monitor.admission.test.ts@4e9e75080335799966f2fcd3ebd6a232d415b42d`

WhatsApp derives one durable event identity from the native remote JID plus message ID and one conversation lane from the remote JID. Its account queue is opened through the shared runtime state API; the extension does not own a separate SQLite writer. The private ingress lifecycle is attached to the in-memory message through a symbol rather than being serialized as payload authority.

The shared ingress worker is the durable writer for enqueue/claim/recover/refresh/complete/release/fail/delete/resubmit/prune/purge. Mutations run inside the state-database write transaction and assert the shared SQLite worker admission at transaction and commit. Claim transitions can carry the already-admitted clock attachment, preserving the exact snapshot used to decide the claim.

Admission distinguishes a known append failure from an unknown native result. Known append failure can use the bounded retry schedule. If a worker result is `outcome-unknown`, the monitor does not replay or acknowledge the mutation; if the fused enqueue-and-claim may have committed, it retires the empty drain owner so the ordinary recovery path can discover the durable row instead of generating a duplicate claim.

WhatsApp debounce keeps each durable claim distinct even when several messages fan into one visible turn. Batch ordering is timestamp then receive order, sender changes force a flush before group attribution can mix, and the fan-in lifecycle settles only after the downstream turn succeeds; read receipts occur after settlement. An exception abandons the combined lifecycle instead of marking the messages read.

Focused tests prove missing native message identity is a permanent refusal, pre-adoption dispatch failure releases the claim back to pending, duplicate adoption callbacks are idempotent, old pending rows beyond legacy age/count limits are still recovered, and unknown native admission is neither retried nor acknowledged. Shared monitor tests further prove restart drain returns a claimed event to pending without consuming retry budget and a successor delivers it once; pre-adoption stop releases for retry; a completed result racing stop stays completed; a deferred handoff racing stop remains owned by the deferred owner; and shutdown joins the final claim settlement write before drain disposal. WhatsApp itself selects `admissionMode: "while-running"`, so the generic opt-in append-after-stop mode does not grant a retired WhatsApp monitor late transport admission.

### 29.4 Nostr gateway bus generation and exact Gateway authorization consumer

New frozen identities:
- `extensions/nostr/src/gateway.ts@619ad4ff081212762b2ca84b9acf10fa2d199410`
- `extensions/nostr/src/nostr-gateway.admission.test-support.ts@c4c6ef2b41c4f0ea84541711bf38acf05f2901df`
- `src/channels/direct-dm.ts@4ea0dc80f2caa9f12dc77bedf2d898722e453daf`
- `src/plugin-sdk/channel-inbound.ts@831d6b0c8af9f576c288e44841fd3ac6a5130614`

The Nostr gateway is the production consumer that binds the already-registered bus, relay subscription, durable ingress and outbound owner. `activeBuses` is account-scoped. Shutdown first removes the exact old bus from that map and only then awaits `bus.close()`; it deletes only when the map still points to that bus, so a late old close cannot erase a successor. Every new outbound lookup reads the current map.

Inbound uses a two-stage authorization boundary. The relay-side optimization can reject obvious nonmembers before durable work, but after decryption the gateway resolves current canonical access again. Before dispatch it invokes `dispatchInboundDirectDm` with a route-binding callback that resolves access against the final agent/session/message context. A previously admitted sender whose policy changed therefore cannot use its durable admission as authorization.

`dispatchInboundDirectDm` resolves the final route first, asks the channel for exact ingress authority bound to that route, builds the typed context, and passes any live authority assertion plus turn-adoption lifecycle into the canonical channel turn. The plugin SDK module is only the shipped facade for that existing owner.

Focused Nostr Gateway tests use the real bus, relay, durable queue and Gateway policy resolution while stubbing only downstream model dispatch. A signed outsider burst and forged author do not reach dispatch when a literal allowlist can be optimized; abort/join with a changed allowlist creates a new generation that accepts only the new member; changing the live policy to disabled after initial author matching prevents dispatch; and configurations ineligible for the pre-crypto optimization may still durably admit an event, but that durable admission never grants authorization. The state DB close in this test support is cleanup only.

### 29.5 Session-catalog commit guard into the canonical session transcript writer

New frozen identities:
- `src/gateway/server-methods/session-mutation-guards.ts@6c304c09080c1eee4c10b4c6c4a10f5e381268df`
- `src/gateway/session-sharing-committed-read.test.ts@6660a176caabe9bafce0cb8d4896e1f1b105d5bc`
- `src/gateway/session-control.incognito.test.ts@8ba11f9ae003c7d24db79d375efd57329e039051`
- `src/plugin-sdk/session-transcript-runtime.ts@cd323a123459f3a7cb8f0f4c13d6a3a63777c8fd`
- `src/plugin-sdk/session-transcript-runtime.worker-preparation.test.ts@37abcff81412d5a065689844d6cc6675f042604d`
- `src/plugin-sdk/session-transcript-lock.worker.test.ts@13feb8568590a06b00b0d1ca54242126d8bcaf97`
- `src/plugin-sdk/session-transcript-lock-runtime.ts@3d7d0fb7d8d68110b5cfa9e93dfe5582e73e0b14`
- `src/config/sessions/transcript-write-context.ts@082d652ac7cca1f6cabda7a61f983ed3fcebedd4`
- `src/config/sessions/session-source-authority.ts@4ec91ea30076ea098e96500eb311ae51cdd15544`
- `src/config/sessions/session-accessor.sqlite-transcript-turn.ts@d2afb51165b2556fd5c1578b050ad97a73219481`
- `src/config/sessions/session-accessor.sqlite-transcript-write.ts@003377de6010f04795ba5e8be92a0d13c0169033`
- `src/config/sessions/session-cold-storage.ts@a92697f35b79311c870771bb5768e62cda6441a5`
- `src/config/sessions/session-transcript-locked-write.ts@80972cb5565b55930073aa65be366d9b68be3918`
- `src/config/sessions/session-transcript-event.ts@9960c0344299449ee9b90ca4c276b9e69a5640ed`

The Section 28 catalog import guard reaches the existing session mutation authority rather than inventing a catalog writer. `withSessionMutationCommitGuard` composes operator/profile/source checks onto normal currentness, prepared worker grants, transaction assertions, admitted-input authority and target checks. Already accepted input retains the authority explicitly transferred to it, while new durable effects must continue to pass the live guard.

The session transcript side keeps this source authority through the canonical writer context. `withSessionTranscriptWrite` resolves the exact runtime target and uses ordered optimistic sequence writes. The projected lock captures the selected store/owner across awaits, closes the callback surface when the callback returns, and publishes queued transcript updates only after the callback succeeds; individual message commits and publication settlement therefore remain distinct.

`transcript-write-context.ts` binds the admitted run/session/store/agent identity in AsyncLocal ownership and refuses rebound to another target. Composed source assertions prepare storage-dependent sources, retain them through the scoped operation, check currentness before/after the effect, and release them in reverse order while preserving cleanup failures.

Durable sequence/turn writes delegate to the agent-database canonical writer. Async message preparation is followed by an exact database/transcript identity check before commit; a foreign append or replacement during preparation produces a mutation conflict rather than allowing stale prepared data to overwrite the new generation. The dynamically loaded locked-write owner retains the agent DB generation claim, pending-input custody and prepared source authority through every queued read/append/replace. Cold transcript restoration uses the same exclusive writer and source/generation assertions, so restoring an archive does not create a second transcript writer. Raw guarded events also enter the same canonical worker path and reconcile projection only after a committed result.

Focused tests show physical or logical writer authority revoked during preparation causes the append to fail with no persisted message; an external byte rewrite invalidates the prepared snapshot even when parsed values and watermarks look unchanged; and a callback failure after a successful append leaves that append durable while suppressing its queued publication. Worker-preparation tests reject an async prepared message after a foreign transcript write and preserve idempotent non-append only when the original source remains current. Committed sharing tests show unrelated rows and the same writer's uncommitted changes cannot manufacture or revoke committed authority, while a committed source/session change does revoke it.

### 29.6 Closure and ownership result

This recursion closes the Section 28 fronts through their next authoritative writer layers: Memory provider-use plus index-generation leases; Meeting capture through the canonical state-worker transcript store; WhatsApp through shared channel-ingress admission and state-worker transitions; Nostr through the account bus and exact Gateway ingress route; and catalog history mutation through the existing session-source and transcript writer.

No newly read production file takes ownership of process-wide `closeOpenClawStateDatabaseAsync`. The new state-database consumers use captured worker generations, typed state-worker operations, leases or scoped admissions; process-wide close calls seen in focused tests remain test cleanup. The prior process-wide Gateway/agent/global retirement conclusion is unchanged.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. This evidence is source/oracle closure only: scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, independent ACCEPT, OCRM final acceptance and release acceptance remain absent.


## 30. Section 29 next-owner recursion: ingress claim settlement, routed-turn custody and worker publication fencing

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This section follows only the new production consumers opened by Section 29. The generic channel-ingress queue client, generic SQLite worker admission/store, OpenClaw agent worker store/admission primitives, and previously registered shared-state async lifecycle/owner are not duplicated.

### 30.1 Durable ingress claim settlement owner

New frozen identities:
- `src/channels/message/ingress-drain.ts@0bf1f7025cfef7aa2a69a2518d66775ac52a8018`
- `src/channels/message/ingress-monitor-tasks.ts@f8c4bdb6c6374a01c282a8cd4ffb24c5416c59ba`
- `src/channels/message/ingress-claim-writes.ts@c09988ccc77e5e140236e2fa82d0c67bf93a02ad`
- `src/channels/message/ingress-drain-state.ts@2d0045541d823727f65967f9a71d5f7fbbbdb15f`
- `src/channels/message/ingress-drain-lifecycle.ts@ef3968496031ee744bc74e4ac7025454d1976947`
- `src/channels/message/ingress-claim-owner.ts@a6319397efbdc26671ded13fd5bd98fab94f7bfc`
- `src/channels/message/ingress-drain.test.ts@b3012a623c0b037d5a4398f3bd01c352697a0039`
- `src/channels/message/ingress-drain.watchdog.test.ts@4c5bae5ff0387e09452236e8f63a3f4aebf7ff4d`

The shared drain is the sole claim-lifecycle owner above the durable ingress queue. It assigns a process-instance-bound owner ID, claims per lane, refreshes the claim lease while pre-adoption or terminal settlement is still owned, and routes every terminal transition through one `settleOnce` owner. A state becomes `settled` and leaves the in-memory owner maps only after the complete/release/fail write has actually committed.

Adoption is the durable boundary, not handler return. `onAdopted` commits the completion tombstone; `onDeferred` retains the claim; `onAdoptionFinalizing` suppresses the stall watchdog while a durable handoff is completing; cancellation releases without adding a retry attempt; abandonment reopens the row with its prior retry facts; retryable/nonretryable failures select release or dead-letter through the shared policy. A restart-draining error releases the claim without consuming its failure budget so a successor can recover it.

Claim settlement writes are token-fenced and bounded. Completion returning false means the lease was reclaimed and raises `IngressAdoptionLostError`; release/dead-letter returning false is treated as already settled because another owner may have won the race. Known storage errors retry with bounded backoff. `outcome-unknown` and adoption-loss errors never auto-retry: after visible side effects, ambiguous tombstone status is deliberately held rather than risking double dispatch.

The drain retains Gateway root work for a dispatched claim even when its transport/pump admission frame ends. It refreshes the claim until the tombstone commits. If refresh proves the token was reclaimed, the old handler is guillotined and a late adoption fails loudly instead of mutating the successor's claim. The per-drain owner ID includes PID, process start token and UUID; fresh claims owned by another live process instance are protected, while stale/dead-instance claims can be recovered.

Focused tests prove crash-window recovery dispatches exactly once; duplicate adoption is idempotent only after genuine adoption; a live peer drain cannot steal a fresh claim; a failed tombstone after handler side effects keeps the claim owned and is never redispatched; a reclaimed token rejects late adoption without disturbing the successor; and complete/release wrapped in an unknown SQLite outcome retains custody rather than replaying. Watchdog tests show heartbeats keep valid pre-adoption work alive, silence guillotines it through the same canonical retry owner, and an unknown release outcome cannot let a newer same-lane row overtake an unresolved head.

### 30.2 Nostr passive account lifecycle into canonical routed-turn delivery custody

New frozen identities:
- `src/plugin-sdk/channel-lifecycle.core.ts@45a1d9e08e0cba2a59be528ba259e2e2127e56ae`
- `src/plugin-sdk/channel-lifecycle.test.ts@20771df1ed91e6adc9e4d19c0b0faff92b0bd1e6`
- `src/channels/turn/lifecycle.ts@2ea1e0690375e9b3f6955cfc4013a93f39e67f8d`
- `src/channels/turn/run-channel-turn.ts@a80ee376010064eda13f9be409b2259d12e4cde0`
- `src/channels/turn/run-channel-turn.custody.test.ts@11e1dc049e0b790415bad0ef05ef18bd3ba10492`
- `src/channels/turn/run-channel-turn.delivery.test.ts@26904fe3928b8c9954864d88d448d9838d617fd4`

The Nostr account wrapper from Section 29 uses `runPassiveAccountLifecycle`: startup returns one handle, the task remains pending until its account abort signal, and `finally` invokes stop for that exact handle once. Startup failure never invents a handle to stop. Focused lifecycle tests prove readiness/assertion failures still join startup before teardown, while a startup failure itself performs no synthetic stop.

The direct-DM helper then enters the canonical routed-turn lifecycle. The top-level turn carries the ingress adoption lifecycle into dispatch; a prepared turn is rejected if its internally declared dispatch lifecycle does not own the same top-level adoption object. A skipped dispatch adopts only through the explicit lifecycle hook, so the durable ingress owner has one settlement surface.

Outbound visible delivery is separately custodial. Routed delivery obtains pending-final custody before platform-send dispatch. If the writer identity is replaced, later delivery under the stale custody is rejected. Durable or direct delivery preserves provider receipts and partial-delivery results, and settlement waits all pending attempts before returning the dispatch result. An ambiguous native/platform result remains `unknown`; a retryable pre-dispatch rejection can return custody to `prepared`, but a partial visible send cannot be replayed through the direct path.

Focused custody tests show repeated platform-send dispatch calls on the same retained writer settle once and a replaced writer is rejected. Delivery tests show stale delivery callbacks from a retired owner are refused, prepared payload custody survives hooks, unknown-to-delivered transitions retain one delivery identity, durable partial-send visibility is preserved without direct retry, and provider-finalized content is observed only after deferred delivery settlement.

### 30.3 Session transcript worker commit receipt and publication fencing

New frozen identities:
- `src/config/sessions/session-entry-patch.ts@e0874105f135effcf478c5b7fba0968e4dcd6921`
- `src/config/sessions/session-entry-patch.settlement.test.ts@99417e8402f4846b0106a74e1d2703b75f289537`
- `src/config/sessions/session-entry-patch.test.ts@402ec5d682f704eff1b14df3fe294e430490e262`
- `src/state/openclaw-agent-write-admission.ts@35f9603181d7d8a7ce0000b3f40946bb67d7ecea`

`runSessionEntryWorkerOperation` is the settlement bridge beneath the Section 29 transcript writer. It captures the physical agent database identity, retains both session-entry and transcript publication custody, and requires an exact framed candidate plus matching commit receipt before treating a native write as acknowledged. Transaction facts can validate source authority before the commit grant; publication is begun only from the candidate admitted by that transaction.

The native result and the durable commit receipt are intentionally separate. A confirmed receipt can acknowledge a commit even when the command reply is lost. Conversely, a reply without a trustworthy matching receipt, a lost native settlement, or an ambiguous commit produces `outcome-unknown`; it does not fabricate a rollback and does not re-run the mutation. Confirmed writes settle publication custody even if later acknowledgment/publication callbacks fail, so committed state and projection cleanup are accounted separately.

Prepared source authority is held until the confirmed commit path releases it. Async planning may release the writer FIFO, but the later commit compares the captured preimage and source/currentness again. The worker write admission itself captures the physical database path identity and serializes worker/native writes through the existing per-path queue; worker callbacks cannot reenter an inherited native writer lock.

Focused tests prove two independent commits inside one retained worker operation each receive their own acknowledgment and publication. Patch tests prove revocation at the transaction or final commit grant blocks mutation, source changes during planning refuse the commit without a caller-thread source read, and acknowledged COMMIT publication occurs exactly once even if the command reply is lost. If commit outcome and cleanup both fail, the outward result remains `outcome-unknown` while the already committed row is preserved—cleanup failure does not authorize a retry.

### 30.4 Shared-state worker operation and retained lease boundary

New frozen identities:
- `src/state/openclaw-state-worker-store.ts@2ae5bde9c3498bc124bcd48335ba585214d863f4`
- `src/state/openclaw-state-worker-lease.ts@a037686a4e5adb23a81f49d979d784bc3e666091`
- `src/state/openclaw-state-worker-operation.ts@6b5279b2be6d7bbb9111b909d79c4159769b30ca`

The Meeting transcript and shared ingress state-worker clients from Section 29 terminate at the existing shared-state worker owner. `runOpenClawStateWorkerOperation` rechecks the captured database admission before open and before execution, retains one owner operation for the entire command, and records an open failure only if the same generation is still current. A retired generation therefore cannot publish its refusal into a replacement database owner.

Worker execution also compares the native worker actor's physical path identity with the currently known database identity before each operation. A live alias cannot keep authorizing a worker whose opening path vanished. Captured schema scope and maintenance scope travel with the operation rather than being re-resolved from ambient process state.

A retained worker lease has explicit accepting → draining → closed invocation phases. `release()` seals new operations, runs the optional finalizer under a terminal invocation, joins commands admitted by that finalizer and all previously accepted operations, and only then retires the native store hold. `retire()` invalidates every invocation immediately but still joins already dispatched native commands before releasing the retained operation. If the native store disappears while an active lease is not sealed, command failure triggers retirement and aggregates cleanup failure rather than opening an implicit replacement inside the old lease.

These files lead back to the already registered shared-state worker owner and async database lifecycle; no second shared-state writer or process-wide close owner is introduced.

### 30.5 Scoped closure result

The Section 29 recursion now reaches stable canonical owners for all five active fronts. Shared ingress resolves to one claim settlement owner plus the already registered durable queue/state worker. Nostr resolves through one passive account handle and the canonical routed-turn/delivery custody. Session transcript writes resolve through one physical agent-worker commit receipt/publication fence and the already registered agent worker owner. Meeting/shared-state writes resolve through one captured state-worker operation/lease and the already registered shared-state owner/async lifecycle.

No new dynamic production dependency in these paths establishes an extension-owned `closeOpenClawStateDatabaseAsync`, a second agent-database writer, or a second transcript/ingress state truth. The scoped production-owner chain is therefore closed through this recursion level; further work should move to other still-unregistered Round87 consumers rather than recursively reopening these already resolved generic owners.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## 31. Residual Round87 close/snapshot consumers: agent close, snapshot staging and source fencing

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). After Section 30 closed the prior five recursive owner fronts, this pass returned to the remaining capped Round87 close/worker/snapshot families and recorded only identities absent from the live dossier.

### 31.1 Agent-database native close and maintenance ownership

New frozen identities:
- `src/state/openclaw-agent-db-close.ts@7f8b5f029100efa3fb0bfb99c0c45a4980e60883`
- `src/state/openclaw-agent-db-maintenance-lease.ts@172b5b8b39c55c182da374deaa84f2a60ccb7c4e`
- `src/state/openclaw-agent-db-resources.test.ts@933735d4a8230f2c6b19afa48a20b4fe6f2ff564`
- `src/state/openclaw-agent-db.worker-admission.test.ts@84d1427554f2b531eb3dfd4ec8807829bef7c`
- `src/state/openclaw-agent-db-maintenance.test.ts@7d45f68dbf7c529636052495acb344ba9566e14e`
- `src/commands/doctor-maintenance.agent-lease-wait.test.ts@089c26ba0503afbdac9a7c5a4317451ade5cafad`

The native agent-database closer is a scoped lifecycle owner, not a process-wide state-database closer. It captures the existing database lease and path anchor, prepares a clean-close seal for SQLite, performs WAL checkpoint/maintenance close, and retains runtime proof only when the close is healthy and the physical path is still the current database incarnation. When the exact leased database closes, it revokes admissions for that path, releases the agent-database lease, and then releases deletion-cleanup and creation-claim handles. A stale or unhealthy handle therefore cannot publish clean-close proof for a successor.

Agent maintenance is additionally fenced by the shared state lease. A top-level maintenance lease first closes all agent databases and asserts that no agent-database leases remain before maintenance work begins. Nested maintenance may reuse the same retained owner, but it cannot switch to a different state database. One failed ownership observation latches failure across every nested scope under that owner; renewal or further writes then fail closed. Maintenance and integrity work are joined before the scope releases its fence.

Focused resource tests show a selected close joins that owner's registered resources without retiring unrelated owners, keeps the draining root fenced through every asynchronous settlement, refuses unresolved/new admissions throughout close, drains resources before native close, and leaves a later successor registration untouched. A maintenance close failure is surfaced to ordinary close waiters rather than silently forgotten. Worker-admission and maintenance tests further prove a recreated worker stays private until its creation/native-close claims settle, prepared/delegated worker opens are refused during maintenance, and expiry/replacement/abort/owner/version loss fences worker resume after integrity work. A cancelled integrity worker is joined before its database or maintenance fence is released.

Doctor's foreign-lease wait is deliberately outside state ownership: while another process still owns an agent database, Doctor can wait without holding the state lease that the foreign owner needs in order to publish its own close. Own-process or serving-Gateway leases can instead be refused immediately. This preserves one close authority rather than allowing Doctor to steal a live database.

### 31.2 Snapshot staging, single-flight production and cleanup custody

New frozen identities:
- `src/infra/sqlite-snapshot-staging.ts@45b01f38a5cdd82b6dd2b8dce5872f2c92025685`
- `src/infra/sqlite-snapshot-single-flight.ts@fdd43625346f66a705e9e71774111369e6f9a6cc`
- `src/infra/sqlite-snapshot-staging-owner.ts@251f37366ffac2ed5f9e995c785fc9e30993b245`
- `src/infra/sqlite-snapshot-staging.worker.ts@a2651757b9c915f36aa9f57c15421558260a8b1a`
- `src/infra/sqlite-snapshot-staging-runtime.ts@20e647e8dec40debb26dd56eae0ce32a95ae0e09`
- `src/infra/sqlite-snapshot-staging-allocation.ts@e7cdc6684324e5e55e7a9d04f525653e8b398f79`
- `src/infra/sqlite-snapshot-single-flight.test.ts@ce98e6e19753db7cb8542ed65e0b9505892b4b66`

Snapshot staging has explicit token and cleanup ownership. Read tokens protect staged bytes independently of the child process. Reclamation first acquires retirement authority, retires payload, removes the claimed directory, and only then releases its token; token-close failure keeps cleanup custody visible. Async reclamation and allocation run as retained snapshot work with an abort owner. If an allocation has already produced a directory when cancellation wins, the directory must be removed; failed removal raises `SqliteSnapshotCleanupError` instead of pretending the operation vanished.

The staging owner is keyed by the retained native worker source/runtime generation. One owner tracks active requests and every retained directory. New requests are refused once admission closes. A returned directory remains registered with a native-retirement callback until its exact retirement completes. Owner shutdown joins active requests, preparation cleanup and directory retirement before closing/rotating the worker pool. Cleanup and primary failures are aggregated, so a failed child cleanup cannot be hidden by the request's earlier error.

Inside the staging worker, each published directory has an exact native retire handle. Closing a directory retires native ownership before filesystem removal and reports removal back to the native lifetime owner. Failed preparation attempts cleanup immediately; if that cleanup also fails, the encoded reply carries cleanup failure and retains enough directory identity for the owner to continue custody. The staging runtime shares a native child only across compatible launch context; directory reference counts prevent closing a session while any retained directory exists.

Single-flight snapshot preparation shares one producer without sharing away individual consumer ownership. Each waiter acquires its own reference; only the last release can start base cleanup and producer close. Cancellation withdraws only the canceled demand unless no remaining reference/listener needs the production. Orphan cleanup is tracked and joined before the retained producer is considered settled. A cleanup refusal remains an error rather than deleting the flight and silently allowing an unowned staged image.

Focused single-flight tests prove a synchronously refused producer is not replayed merely because a later call could succeed, one canceled waiter does not abort the surviving waiter, orphan cleanup must finish before tracked production settles, and a snapshot remains retained until a joined waiter actually acquires its lease.

### 31.3 Private/recovery snapshot source identity and cleanup

New frozen identities:
- `src/infra/sqlite-private-snapshot-fork.ts@3ba0c2082b818c846036e5ccee2aec61960a18cb`
- `src/infra/sqlite-private-snapshot-fork.test.ts@42f75557b3ea589d5d7df568e31626be18a34e14`
- `src/infra/sqlite-recovery-snapshot.ts@2945f754048032950f86598e4cd9ae7fdd4c55dd`

A private snapshot fork first acquires a read token for the retained family and revalidates the captured main/WAL fingerprint before copying. A changed source incarnation, WAL removal/replacement, or rollback-journal ambiguity refuses the fork; it does not retry against whatever bytes happen to be current. The destination is a separate writable image, not an alias to the live source. Failure cleans destination work and releases source custody, aggregating cleanup failure rather than replacing the original source error.

Focused tests prove committed WAL rows survive into an independently writable image, retained main/WAL mutation causes `Retained SQLite snapshot changed`, source-WAL removal during copy is refused without retrying live capture, and failed destination work is cleaned while the retained source remains intact.

Recovery snapshots similarly retain the caller's mutation authority and a private directory identity. Directory type, symlink status, permissions and inode/device identity are checked before and after the read and again before cleanup. Cleanup never adopts a successor directory or parent path. Operation and cleanup failures remain distinct and are aggregated when both occur.

### 31.4 Generic SQLite worker client and cross-database source fence

New frozen identities:
- `src/infra/sqlite-worker-client.ts@8cb2f98eb29edde8474997dcaabce19a944ce160`
- `src/infra/sqlite-worker-client.test.ts@1e5efebe35204515992c21cc73e5501bf0dfd79f`
- `src/infra/sqlite-source-fence.ts@dec509c74c343009b74f5c45b700b5194d46929f`
- `src/infra/sqlite-source-fence.test.ts@ed9a57c49b631cbc783d9030ddc1059e85fe9d56`

The generic SQLite client captures maintenance scope, state context and the caller's worker-admission/currentness callbacks at operation creation. Queued reply callbacks execute those assertions in the captured caller context instead of ambient replacement context. A missing, sealed or draining client refuses work before dispatch. Close seals synchronously, waits every accepted scope, and only then releases the owner; idle retirement likewise refuses to run while scopes or commands remain pending.

The source fence is a worker-only multi-database transaction boundary over already admitted durable handles. It binds the destination and all sources to their captured physical identities, requires exclusive operation custody, rejects aliases whose incarnation changed, reserves source transactions inside the worker, validates the complete retained source set, and obtains a destination commit receipt before accepting the result. Empty acquisition attempts may retry, but once reservations/commit custody exist the operation is not replayed.

Destination COMMIT and source cleanup are separately accounted. If the destination has committed, later source rollback/disposal failure cannot erase its commit receipt or authorize a retry; cleanup failure is surfaced after the committed result is owned. Cancellation or source-owner revocation after an accepted commit likewise cannot retroactively roll back the destination. If native execution disappears before/after commit, settlement reports the unknown/native-stopped state while joining source locks; a post-commit row remains preserved and is not replayed.

Focused source-fence tests prove reservations exclude independent writers until destination settlement, alias identities collapse without creating duplicate locks, earlier empty reservations are released when a later physical lock is busy, accepted commit custody survives caller/source/destination cancellation, absent rows or changed incarnations fail closed, committed receipts survive reply/source-cleanup faults, and native loss joins without replay or leaked source locks. Worker-client tests independently prove active work prevents idle retirement and future commands are refused after the eventual sealed close.

### 31.5 Residual-family closure result

A fresh capped search for `closeOpenClawStateDatabaseAsync` returned 99 distinct indexed paths and every one was already present in the live Round87 dossier before this append. The newly registered agent-close, snapshot and generic worker consumers therefore do not reveal another process-wide shared-state close owner.

The separate agent-close family remains scoped to per-agent database handles, leases and maintenance resources. Snapshot staging owns private temporary generations and their cleanup, not the source database. Source fences own only the exact captured multi-database operation and its commit/cleanup settlement. These responsibilities terminate at the existing state/agent database lifecycle and worker authorities already registered by earlier sections; none establish a second durable writer or hidden fallback owner.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## 32. Residual execution-close, worker-broker, read-snapshot and rollback-recovery ownership

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Section 31 closed the native agent-close/snapshot/source-fence primitives; this pass follows the still-unregistered production callers and broker/read/recovery owners exposed by those families. Identities already present in the live dossier, including the SQLite operation-admission core and its timeout tests, are intentionally not duplicated.

### 32.1 Agent execution close and bounded command/doctor cleanup

New frozen identities:
- `src/state/openclaw-agent-execution-close.ts@ac881adcbea18aaf22dff04bf4d1e645c651a58e`
- `src/state/openclaw-agent-execution.worker.ts@a533936aa520fc1770abd779a9fb9baec327ac41`
- `src/state/openclaw-agent-execution.registration.test.ts@28d1588a0e2077d9df8acb495a3a8a9730ef1fb2`
- `src/cli/runtime-cleanup.ts@99c5ff2d45f8a7e838853c2384e6a56bdc11dcae`
- `src/commands/agent-exec.ts@5dcadd1090a8befedf5243a31e784297f23920a1`
- `src/commands/doctor-agent-memory-schema.ts@58461105d06f74d97417b4b7322ea38d1ba8d3b7`
- `src/commands/doctor-session-delivery-state.ts@6ac2f0f7b920ae6f36ce4839f562508265ec036a`

The execution closer keeps the close receipt, preparation releases, domain close, native agent-database close, agent borrow, and shared-state borrow under one retirement owner. Ordinary close first stops WAL maintenance, performs every synchronous cleanup step without letting an earlier failure suppress later releases, then joins the asynchronous shared-state borrow release; all failures are aggregated. Failed-open cleanup uses the same owner but can use the synchronous shared-borrow release because no successful live execution is being handed back.

A close receipt is produced only when the captured physical execution identity still has durable close evidence: an observed WAL checkpoint for SQLite or the captured PostgreSQL connection identity. The worker binds that closer to the same actor that opened and registered the database. Once the execution owner is closed, further commands are refused; registration/open failures retire native custody before surfacing their final error. The focused registration tests preserve the original admission-refusal provenance across retirement, retain typed quarantine cleanup failures, and do not fabricate a successful close receipt from a failed native opening.

CLI cleanup does not become another database owner. Timed-out disposers remain in a process-global pending set and automatic process exit joins them; deferred cleanup runs after already-pending ordinary disposers rather than racing them. `agent-exec` gives temporary runs their own maintenance scope and, only after runtime cleanup is settled, drains the exact temporary agent databases and then the temporary shared-state database before filesystem teardown. If runtime cleanup is uncertain, it records retained ownership instead of pretending the temporary state is safe to remove.

Doctor's memory-schema repair closes only the selected agent handle after acquiring the doctor maintenance lock plus agent maintenance lease and rechecks lease ownership before and after migration. Session-delivery repair similarly closes a target it opened only when that target was not already open and runs its mutation under the current doctor/schema-maintenance authority. These are scoped maintenance consumers, not alternative production close owners.

### 32.2 SQLite worker broker capacity, lifecycle and operation custody

New frozen identities:
- `src/infra/sqlite-worker-broker.ts@3710080e88ac48d321133ccf11d8669234900216`
- `src/infra/sqlite-worker-broker-lifecycle.ts@5245769277deb66b9952597a4d576c6ce980fa7a`
- `src/infra/sqlite-worker-operation-scope.ts@34191016a7a62d8d4eca7196754e590309451b23`
- `src/infra/sqlite-worker-operation-settlement.ts@d1c3b24539d751d33f067e2c7ad83f6485839c26`
- `src/infra/sqlite-worker-input-admission.ts@f8cacdd884aa73f89f7446570c200efa67b7d651`
- `src/infra/sqlite-worker-admission-retry.ts@612d3acfd6018e7c5d27689ec0080c568cc05686`
- `src/infra/sqlite-worker-database-admission-relay.ts@c9255bec0333561c868deac1504d56c30c34b4ed`
- `src/infra/sqlite-worker-broker.capacity.test.ts@00a3a6a2bd5bf765840ecb7794982d35629b4ecb`
- `src/infra/sqlite-worker-broker-lifecycle.test.ts@eca3e4433dd1f5162c294bfe4ef0fd47d4d2e965`

The broker is the host-side owner of actor reuse, physical-path capture, slot capacity and actor retirement. Opening first acquires the input budget and database-admission turn, rechecks caller currentness, captures state-database access and the admitted physical paths, and refuses reuse when the cached actor, module/input identity, runtime generation, or current owner no longer matches. An actor whose retirement or cleanup is still pending cannot be reopened; the broker must join that exact retirement/close first.

Slot exit marks every actor native-stopped before removing the slot. Admission rejection joins slot/actor cleanup and aggregates cleanup failure with the primary refusal. Runtime-generation settlement first requests retirement of all retained actors, waits actor settlement, reports retirement failures, and only then retires empty slots belonging to that generation. Capacity reclamation therefore cannot silently install a replacement over an unresolved native owner.

Input retention is bounded before a job becomes queued work and shared-state/ordinary opens share that owner budget. The operation scope installs only the broker's private admission port; the settlement object carries request custody, execution authority, refusal, committed facts and native settlement separately. The relay transports database-admission facts across the worker boundary under a bounded peer wait instead of re-resolving ambient authority in the child.

Focused capacity tests prove reclamation waits retirement and atomically consumes the released capacity, never reclaims an idle client from a different runtime generation, and propagates retirement failure without admitting a replacement. Shared-state and ordinary cold opens share one input budget and converge on one physical owner. Lifecycle tests prove canceled slot waiters do not release the carrier before worker exit, and FIFO handoff joins accepted work rather than canceling it to manufacture an idle window.

Admission retry remains narrow: only an admission-timeout path can be reconsidered, and the prior transaction/owner must be usable and fully unwound first. It is not a generic retry surface for an ambiguous native commit or cleanup outcome.

### 32.3 Read-only native resource and retained shared-state reader custody

New frozen identities:
- `src/infra/sqlite-readonly-native-resource.ts@d9c025f54f23742666cb80fed1930af39cb15029`
- `src/infra/sqlite-readonly-native-resource.client.ts@6eb4d75a3bd53bb8b4015d3beefa2e7d98918172`
- `src/infra/sqlite-readonly-native-resource.test.ts@7d6ce64c17376398e2cecde1e9091e11d093a582`
- `src/state/openclaw-state-db-read-connection.ts@4a8b104a9e24b720159f953ada3a3b5b61c89184`
- `src/state/openclaw-state-db-read-connection.test.ts@0da080ef8357cbbb20a9e60182a41fbf14f4299f`
- `src/state/openclaw-state-db-read-connection.cache.test.ts@8722ff79dfbccd1620180026ead5bd658c899d89`
- `src/state/openclaw-state-db-read-connection.nesting.test.ts@706e368e85a13c4d6a65432ea40fbaa309f3c7db`
- `src/state/openclaw-state-read.worker.ts@aa7a23ea0a47063dc7f0f5ae7c89312b3ab78d44`
- `src/state/openclaw-state-read.worker.test.ts@f0c0ff4b294e21a6e677d859da3b0d68328af60e`

The read-only native resource keeps child-process/session custody in the host NativeLifetime owner. The client merely transports typed commands: session creation captures its launch environment, close waits even a refused creation to settle, and a failed close leaves the same session sealed for retry rather than permitting a replacement allocation to inherit it. Copy cancellation cancels that request but does not declare the child dead or release unrelated session ownership.

Focused native-resource tests prove an allocation accepted before target-port loss still belongs to cleanup, a host reader fence can refuse retirement while creator custody remains available for retry, loss of the host fence refuses cleanup without releasing creator custody, and a failed close seals creation and retries against the same native session. Outstanding copy work is joined when the owner closes.

Retained shared-state readers are keyed by physical database identity. A new incarnation at the same canonical path retires the prior reader; nested readers never close their caller's retained handle. The host explicitly joins retained-reader close before replacement/deletion, and idle disposal retries when native cleanup fails instead of deleting the retained entry. Read-only snapshot cleanup, read-admission cleanup and native-reader close are independently accounted, with aggregate failure when both the primary read and cleanup fail.

The state read worker uses this same owner and reports native cleanup failure separately from source admission/result. Cache tests prove peer commits can invalidate only the affected physical identity, stale/missing-file readers retire, replacement during worker preparation is refused, and private snapshots stay task-scoped. The nesting test confirms a nested read settles without closing the outer retained handle.

### 32.4 Cold rollback recovery is evidence preparation, not autonomous replay

New frozen identities:
- `src/infra/sqlite-rollback-recovery.ts@957930a6267f407125d6fb54db664b015e65532c`
- `src/infra/sqlite-rollback-recovery.test.ts@226af27291a360c93ecdd347f7d719d2fa0e809a`
- `src/infra/update-managed-service-handoff-database-recovery.ts@6ab3ef93cf3e24f34cd7da0f342b3a3f6ac46287`
- `src/infra/package-update-activation-journal.ts@58904bcddde8640d86eba6eb46827804d98af0e0`
- `src/infra/package-update-activation-immutable-recovery.ts@1aab11dc5566293ecb4a91c6846d74fbfdc82ee7`

Rollback recovery first fingerprints the retained main database and rollback/WAL/SHM sidecars from device/inode/birthtime/size/content hash under caller-provided file-safety checks. Preparation never grants itself authority to replay the source journal. A normal cold read is returned only if that exact file set stays unchanged.

If a hot rollback journal prevents the initial read, preparation rechecks the complete fingerprint, refuses the case if WAL or shared-memory artifacts exist, and reads the expected record through a private verified recovery snapshot. The returned `admit` function then rechecks source identity and caller currentness immediately before touching the live database. Only inside that admitted step may SQLite perform its native rollback-journal recovery, and the resulting record must still equal the prepared evidence. Thus a changed sidecar, owner or record invalidates the recovery instead of adopting successor bytes.

Focused tests prove recovery of a killed writer occurs only after current authority explicitly calls `admit`; replacement of the rollback sidecar makes that admission fail; and a retained WAL is refused without checkpointing, deleting sidecars, or invoking rollback admission. Managed-service handoff recovery adds its installation/file lock and process-identity owner before admission. Package activation and immutable recovery bind the same primitive to their retained publication/control/runtime identities. None turns cold recovery into a background database writer or a replay-on-error fallback.

### 32.5 Closure result

The previously capped search for `closeOpenClawStateDatabaseAsync` still has no unregistered production extension owner: all 99 returned indexed paths were already present in the live Round87 dossier before this append. This pass closes the newly discovered host-side execution-close, broker, read-resource and rollback-recovery layers without changing the process-wide ownership result.

Agent execution close terminates at the existing per-agent database lifecycle plus its captured shared-state borrow. The worker broker owns bounded actor/slot admission and settlement, not database truth. Read-only native resources own private snapshot/reader cleanup, not source writes. Rollback recovery can prepare evidence but requires the retained product owner to re-admit the exact unchanged source before any native recovery.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## 33. Section 32 direct-consumer recursion: state-read streams, bounded admission retry and retained recovery ownership

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This pass follows the direct production consumers opened by Section 32 and records only frozen identities absent from the live Round87 dossier. Existing generic state-read worker, state/agent database lifecycle, SQLite broker/admission primitives, update-run contention tests and transcript store owner identities are referenced but not duplicated.

### 33.1 Shared-state read stream, Cron read-only worker and transcript export cleanup

New frozen identities:
- `src/cron/store/read-only.worker.ts@f51177dc9a0d1277fb71a4c1bea90580f8d7fda7`
- `src/transcripts/store-export.worker.ts@8102f962b6fc6c10a15b7f6e0155a8d92fff2eb1`
- `src/transcripts/store-export-jsonl.ts@36969fc4ec00290594d34b481c2223dbc944df79`
- `src/transcripts/store-export-contract.ts@84d3817f6a445bdbb64bc1aaa93c903b05727a7e`
- `src/state/openclaw-state-read-stream.ts@f136babeb2032f1711ab4dd9e4b21ae6e6c3d3bc`
- `src/infra/gateway-owner-lease.worker.ts@ea565ad7df5705ea8d75ea8d2ab19ea61b057b85`
- `src/cron/store.readonly-compat.test.ts@f2ec84bb5aa90e5f6b62f930486698c1fb2366ae`
- `src/transcripts/store.test.ts@68924ddb1f51ccbd6773ce3c2c5a128b614ff5a7`

The previously registered Cron read-only host facade was re-read at `src/cron/store/read-only.ts@30f151b914918092a7015380cd3de5e90b513251`. It captures the admitted physical state identity, starts the packaged `cronReadOnly` worker through one `WorkerTaskPool`, and, where required, retains a private staged snapshot under the maintenance resource owner. Close aborts the exact operation, joins worker shutdown, releases staged-directory custody, and surfaces deferred snapshot cleanup rather than silently discarding it. The newly registered worker consumes that retained location, opens either the existing shared-state read connection or the admitted private read target, performs only read-side Cron/history work, and closes the exact connection/database in `finally`. The compatibility test proves an older pre-projection Cron schema can be read without writable schema repair: the missing grant-definition columns remain absent after the read.

The state-read stream is a narrow dispatcher, not another database owner. It requires a bounded receiver for both subagent restore and `meetingTranscripts.export`; transcript export is delegated to the newly registered export worker under the already registered shared-state read worker admission. The export worker opens the retained state read connection, streams bounded utterance chunks, reports transcript-library errors as typed results, rolls back any still-open read transaction before release, and aggregates reader-cleanup failure with the primary export error. `store-export-jsonl.ts` consumes that stream for artifact materialization and closes the artifact handle even when the stream or consumer aborts.

The focused transcript store test proves the important cleanup and snapshot oracles: cancel and consumer-error termination both release the retained export snapshot, and an admitted export remains isolated from concurrent state changes until that export completes; a later export observes the newer canonical state. This separates read snapshot custody from source mutation authority and prevents a failed or abandoned reader from becoming a second state owner.

The Gateway owner-lease worker is another direct shared-state reader. It opens the exact retained state-read connection, performs the owner-lease lookup under the admitted schema/engine context, and refuses an ambiguous result when more than one owner row is visible. The already registered `src/state/openclaw-state-read.worker.test.ts@f0c0ff4b294e21a6e677d859da3b0d68328af60e` remains the focused admission/cleanup oracle for this worker path.

The moving-default discovery set also contained `src/agents/subagents/registry/subagent-registry.store.restore.worker.ts`, but an exact-ref read at the frozen commit returned 404. It is therefore excluded from the frozen source set rather than silently imported from moving main.

### 33.2 The bounded SQLite admission-retry family has two production consumers

New frozen identities:
- `src/state/user-channel-identities.worker.ts@236b4e9f0f1c8814f56d7e8a168fbab46cffffca`
- `src/state/user-profiles.worker.ts@8047f64fb1d2e0f745eca1966cca49a017f8b444`
- `src/state/user-profile-read.worker.ts@46c98df38bb615d6e2873bb37e31e62052a70535`
- `src/state/user-channel-identities.worker.test.ts@c9b63cfa8c217d31fb158a6530ef09c1970e8758`
- `src/infra/update-run-mutation.worker.ts@c8c4fec0131df3274ef8241f8db24c4933e945b1`
- `src/infra/update-run-dispatch.worker.ts@d44444d31e9f771cc3613b08e417aae5ae932247`

The frozen direct-consumer reconciliation for `retrySqliteWorkerAdmission` resolves to these two production mutation families plus the already registered helper itself. Channel-identity change uses a bounded three-attempt admission sequence around the existing state writer. Transaction and commit admission are separate; the commit receipt is deferred only after an actual change. The retry helper can clear only its own admission-timeout refusal after the prior transaction is usable and fully unwound. Any non-timeout refusal, revoked authority, still-open transaction or ambiguous native outcome escapes instead of being replayed.

The user-profile worker is the direct product dispatcher for channel-identity change and other profile operations; the profile read worker reuses the same canonical identity rows for list/resolve reads rather than creating a second profile-state store. The focused channel-identity test withholds transaction or commit admission and proves retry, exhaustion and authority revocation all release the WAL before a subsequent attempt. Failed/exhausted cases leave no partial identity row, and the transaction is closed before the final checkpoint. A revocation remains the admission failure instead of being converted into a retry success.

Update-run mutation uses the same narrow timeout-only retry rule for transaction/commit admission. The dispatch worker routes update-run commands through that owner and distinguishes optional bookkeeping from required recovery evidence: an optional progress record may be reported as bookkeeping-skipped after certified rollback, while required mutations surface a concrete failure. The already registered focused tests `src/infra/update-run-admission-hold.test.ts@f3136576d240c2c8c12e16dc439c85f158212f41` and `src/infra/update-run-contention.test.ts@72e9582bf3ab4d3bd03b5811b315a73c5cc3c62a` prove the writer is released while host commit admission is delayed, transient timeout can retry only after rollback, exhaustion is bounded, and a lock failure after transaction admission is neither replayed nor discarded.

This closes the newly exposed admission-retry consumer family without granting the helper independent write authority. The durable writers remain the existing profile and update-run owners, and a timeout retry is legal only after the previous attempt has lost all transaction/commit custody.

### 33.3 Managed-service rollback recovery remains under the retained product owner

New frozen identity:
- `src/infra/update-managed-service-handoff-database-identity.test.ts@563b4559716d2822c68bc73b049a46b5e6c6044e`

The Section 32 production owner `src/infra/update-managed-service-handoff-database-recovery.ts@6ab3ef93cf3e24f34cd7da0f342b3a3f6ac46287` was re-read, not re-registered. It retains the managed-handoff database identity, acquires the installation/file lock and process-identity owner, prepares rollback evidence, and invokes `recovery.admit` only after those current-authority checks still hold.

The newly registered focused identity test proves the retained-owner boundary against real rollback-journal cases. A killed writer's cold rollback journal is repaired only for the original admitted managed-handoff owner; live or unknown foreign executor custody is refused without modifying the row; future/unknown retained schemas are not adopted; and mutating the rollback journal after preparation makes admission fail while the original main/journal bytes remain preserved. The recovery primitive therefore remains evidence preparation plus explicit current-owner admission, not an autonomous replay or background writer.

### 33.4 Direct-consumer closure result

The Section 32 direct-reader frontier is now reconciled through the newly registered Cron, transcript-stream and Gateway owner-lease readers; all terminate in the already registered shared-state read/worker lifecycle and keep cleanup custody separate from source mutation. The bounded admission-retry frontier is reconciled through channel identity and update-run mutation, with focused tests proving retry is timeout-only and occurs only after the prior writer/transaction has unwound. Managed rollback recovery remains owned by the retained update handoff authority.

A recheck of `createAgentDatabaseExecutionCloser` found no additional frozen production consumer beyond `src/state/openclaw-agent-execution.worker.ts@a533936aa520fc1770abd779a9fb9baec327ac41`, which Section 32 already registered. No newly read production file imports or owns process-wide `closeOpenClawStateDatabaseAsync`, and none of these readers/retry helpers introduces a second agent database, transcript, profile, Cron or update-run writer.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## 34. Priority extension lifecycle consumers and residual path-scoped close/broker recursion

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This pass starts from the Section 25/32/33 frontier, exact-ref reads the still-unregistered priority extension consumers and host-side path-scoped close/broker consumers, and omits identities already present in the live Round87 dossier.

### 34.1 Google Meet platform/runtime and node-host boundary

New frozen identities:
- `extensions/google-meet/src/transports/google-meet-platform-adapter.ts@7aa2486d483b2c6776a1142c7928348c7f1d00e5`
- `extensions/google-meet/src/transports/google-meet-platform-adapter.test.ts@c4c389f7e10f515ced5f1402bc39b2f4fd214b63`
- `extensions/google-meet/src/node-host.ts@4fda39fa5203185e4d86bc7c0bc61c5b2b55ce70`
- `extensions/google-meet/node-host.test.ts@1b72bee11a14a05489c5b3ba4419f441d18c531c`
- `extensions/google-meet/src/node-invoke-policy.ts@63fafd3dc1a9cfeeab6c1f330e0e4b7c368742d5`
- `extensions/google-meet/src/node-invoke-policy.test.ts@6cba7787c168a1b9154f2c563f267ec7b09a25f8`

The adapter is the direct Google Meet consumer of the shared meeting runtime contract: it fixes the stable platform id/session prefix, URL normalization/reuse, manual-action categories, caption capture, audio-route readiness and the exact paired-node command. Durable caption observation is enabled independently while live transcript visibility remains gated by `MeetingSessionRuntime`; the capture script refuses a page whose `__openclawMeetAudioSession` no longer matches the current meeting session. The focused adapter test proves stale capture ownership is rejected and realtime readiness remains false until both input and output routing are verified.

The node host consumes the same adapter URL contract and owns only the node-side command-pair bridge processes. Its tests prove non-Meet URLs are refused before Chrome side effects, a retired output process cannot close the replacement bridge, SIGTERM-resistant children are joined and escalated to SIGKILL, concurrent `stopByUrl` calls share one cleanup, and closed bridges disappear from the active list. The invoke policy strips caller-supplied executable fields and reconstructs start/setup parameters from trusted configuration; invalid URL/action input is rejected before node dispatch. These are platform/process consumers of the already registered participation/session owner, not a second MeetingSessionRuntime or participation store.

### 34.2 WhatsApp connection-controller runtime context closes the direct consumer fan-out

New frozen identity:
- `extensions/whatsapp/src/connection-controller-runtime-context.ts@960f8b89bd42079d410601e231fb0d1ce6a299d6`

The facade reads exactly two channel-runtime capabilities, `connection-controller` and `connection-owner-pending`, and exposes only active listener/current socket/self identity for the requested account. The already registered direct consumers were re-read rather than duplicated: `active-listener.ts` and `send.ts` obtain the active listener only through this controller; `agent-tools-call.ts` obtains current socket LID mapping and self identity through it; `directory-config.ts` starts a standalone directory socket only when neither a live controller nor a pending connection owner exists, then closes that exact socket, drains credential persistence and releases the same owner lease; `inbound/socket-session.ts` may adopt a successor only when the authenticated self identity overlaps the retired socket and asserts the chosen socket is still current at each send.

This keeps socket replacement, standalone directory lookup and captured replies under one account connection owner. A stale socket generation cannot silently become the reply owner, and a pending owner prevents the directory path from creating a competing socket.

### 34.3 X ingress/publication recursion reaches the existing durable OAuth owner without creating another token writer

New frozen identities:
- `extensions/x/src/ingress.ts@ba9efc8e5ec8696b29ce85a5808551b0a29cca48`
- `extensions/x/src/monitor.ts@725d04ab00bdb73db5055dffe3a2d8f65475666d`
- `extensions/x/src/send.ts@4d3de2caa63b3bf4697ba951d5071ac5263c17f6`

The previously registered `extensions/x/src/client.ts@11f7bdfa3cf3168dfc8c54bdb1e7c1d6891127dd`, `extensions/x/src/oauth.ts@13dee996eacb81d1455a84d931c7d346ffc44f21` and `extensions/x/src/client.test.ts@e9ffcd5c4e903c5140351aa653a596e468662bfb` were exact-ref re-read to close the dynamic dependency. `getXApi` owns the durable `x.oauth` V2 keyed store, claims a refresh token with CAS, records a claim id, reconciles a committed settlement whose reply was lost, and refuses a retired account/runtime lineage. OAuth refresh itself marks response-loss as unknown and never treats caller cancellation as cancellation of the joined refresh owner.

The new monitor is the production consumer that acquires that API, opens the account cursor/ingress queue and re-resolves ingress policy before dispatch/publication. `ingress.ts` binds publication preparation to the same config, route, allowlist snapshot and runtime generation. `send.ts` obtains the same API owner, constrains visible replies to the retained work origin, rechecks runtime/config/allowlist immediately before media upload and every reply POST, and reports partial visible delivery instead of replaying already posted chunks.

The already registered X ownership tests remain the focused oracle: transport credential rotation joins the active refresh; a retired seed cannot overwrite the replacement grant; lost provider response or invalid rotation remains fenced across SQLite/runtime reopen; pre/post-commit persistence failures do not exchange the completed rotation twice; and aborting one caller detaches it while the joined owner persists a late rotation. Thus these new production consumers terminate at the existing `x.oauth` store rather than introducing a second refresh-token writer.

### 34.4 Nostr runtime and legacy ingress migration terminate at the registered bus/queue owners

New frozen identities:
- `extensions/nostr/src/nostr-ingress-state.ts@0f275170c605fa3b1729335ae156ccf21b2f2906`
- `extensions/nostr/src/runtime.ts@6866046d229c8ce22579b32fcc7b6375351d5b4c`
- `extensions/nostr/index.ts@34d17d1c74f10da78649c24a1d3505b487e9dbf1`

The ingress-state helper validates event id/pubkey and derives the durable direct-message lane key. Its legacy migration converts only the retired recent-event-id seed into completion tombstones in the canonical ingress queue; it completes a marker only when this migration accepted it or observes its own empty legacy payload, and refuses a failed completion rather than silently losing replay protection. The runtime file is only the plugin runtime store. The bundled index installs that runtime and its trusted-operator profile route; it does not own ingress rows or cursor writes.

The already registered `nostr-ingress.ts`, `nostr-bus.ts`, cursor/store and gateway owners were re-read as the direct consumers. Gateway stop removes the exact active bus before awaiting fallible bus close, so new work cannot reacquire a retiring bus. The frozen Beam tree was also enumerated exactly: it contains no separate `receiver` production module; the only relevant store/mirror production identities are the already registered `extensions/beam/src/store.ts@b501ac4a7948988366e771cd411bec6425159fc9` and `extensions/beam/src/mirror.ts@011c9422ddded974f5c4317251e81115dfbaf036`. No duplicate Beam evidence is added.

### 34.5 Device Pair scheduler is gated by command-owner authority and exact-arm CAS

New frozen identities:
- `extensions/device-pair/pair-command-auth.ts@61c041e196b74576a4bb8d5e502ab01f2cc581f4`
- `extensions/device-pair/index.test.ts@3fed39f698278b0080534f36910b19b0800babfa`

The already registered notifier scheduler and CAS test were re-read. The notifier stores a random `armId`, observes the current subscription before visible send, and deletes a one-shot subscription only with compare-and-apply against that observation; a changed/new/removed arm therefore remains owned by its writer. Failed send, observation or settlement does not fabricate a seen receipt, and every durable notify-state write wakes the scheduler so a lost reply can be reconciled. Scheduler abort clears the wake owner.

The newly registered command-authority helper keeps internal Gateway scopes distinct from external command-owner authority. Missing `operator.pairing` or setup-secret authority fails closed; external non-owners gain neither pairing nor setup authority; an admitted external command owner receives only the intended pairing approval scope while full setup remains tied to that owner assertion. The focused index tests prove unauthorized approve paths refuse before reading/approving a request, caller scopes are preserved, requests above the caller session are rejected, and non-Gateway command ownership is rechecked after asynchronous Gateway URL discovery before issuing a bootstrap token. The scheduler therefore remains a consumer of durable notifier state and pairing authority, not an independent pairing writer.

### 34.6 Residual path-scoped state close and worker-broker production consumers

New frozen identities:
- `src/commands/doctor-database-port-state.ts@998ef00cca4636e33b8e91a67f887323c98bcd62`
- `src/entry.ts@a60dc1657d8b48df5ec3ff01527a0902ebb4f570`
- `src/worker/worker.runtime.ts@0b7e727769ff4d45862437859e6792c897c092b0`
- `src/worker/worker-runtime-environment.test.ts@22079b97a86794eb823e66d9e31e0bafcf11408d`
- `src/infra/update-candidate-rehearsal.ts@48b8053aba050aa13af34b8df0361b68161d6ac8`
- `src/wizard/setup.migration-stage.ts@150b853a057b424886e6729eca90393985cb0fc8`
- `src/cli/update-cli/update-command-doctor-child.ts@e7aef7bb0e177ea793eed3c0e9faf882732bd03e`
- `src/agents/auth-profiles/reader-cleanup.ts@86914036169ef4a841626987b48bcd0b95800104`
- `src/infra/database-engine/worker-store.ts@0b2b1a43ac608cf524493aecf272d31b03ea1008`
- `src/gateway/operator-approval-store.ts@ac797094062c7c20988720a6d6e5cdccf93b9342`
- `src/infra/session-cost-usage-worker-scope.ts@400645d764e26be55729117d41510e05b218e693`
- `src/infra/session-cost-usage-worker-runtime.ts@6a672d8ba99b231fb9ce0b0195bd219dc6037360`
- `src/config/sessions/session-transcript-usage-lifecycle.test.ts@9c465cafdcbee63d8a2487a2e6ce7b304bdbdc94`

These consumers use path-scoped close rather than process-wide shutdown. Database-port Doctor runs under the local state owner, follows the physical store during interrupted publication, and closes only its admitted shared-state path. Managed-node CLI startup opens and finalizes the exact managed state path. A worker runtime environment first joins process/exec cleanup, then closes only its temporary state database before restoring ambient state and removing the directory; its focused test proves the worker-owned database closes while an unrelated state database remains open and usable.

Update rehearsal closes only its candidate snapshot database after revalidating directory custody and before removing retained directories. Setup migration disposes the staged agent database and staged shared-state path before promotion/cleanup and explicitly does not promote the live shared-state database. The update Doctor child closes only the child state path in `finally`, then revokes admission and releases the same process custody. None is a new process-wide `closeOpenClawStateDatabaseAsync` owner.

The broker-side consumers likewise preserve existing custody. Auth-profile reader cleanup disposes every captured reader and aggregates cleanup failure with the original read failure. The database-engine worker store seals before close, joins any reopen, refuses replay after an unavailable worker because the prior command may have committed, and routes writes through the existing worker-store admission. Operator-approval storage reserves input/identity/path leases, publishes only committed broker facts, and retains a committed receipt even if the command reply is lost. Session-cost usage keeps physical database resources, native cleanup retries and host effects separately; native worker exit does not settle an admitted host write. The focused usage-lifecycle test proves retained cleanup is drained by captured path before physical retirement, failed cleanup remains retryable and blocks new work, sibling state remains unaffected, queued writes are canceled/revoked without mutation, and changed file identity is refused.

### 34.7 Closure result

The priority extension recursion exposes new direct consumers but no new process-wide shared-state close owner, agent-database writer, OAuth-token writer, Nostr cursor/ingress writer, Beam receiver owner or Device Pair scheduler writer. Google Meet terminates at the shared meeting/participation runtime plus node bridge; WhatsApp at the account connection controller; X at the existing `x.oauth` CAS store and ingress queue; Nostr at the registered bus/cursor/ingress owners; Device Pair at the durable plugin-state stores plus pairing authority. The residual host close paths are explicitly per-path or per-temporary-environment consumers.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## 35. Native worker lifetime, shared-state borrower and integrity-proof owner recursion

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This pass follows the still-unregistered worker-admission/shared-state lifecycle identities exposed after Section 34. Moving-main-only `src/infra/doctor-capture-reuse.ts` was explicitly excluded after exact-ref fetch at the frozen commit returned 404.

### 35.1 Retained native worker source is a process resource owner, not database truth

New frozen identity:
- `src/infra/worker-native-lifecycle.ts@4afc06188377ae708c7a9db36a272b650f719086`

The native-worker lifecycle owns Worker/thread, control-port, spawned native-resource broker and registered domain cleanup. A source is captured against one runtime generation; it refuses new work once closing, and `closeDefaultRetainedNativeWorkerSource` joins registered domain owners before terminating the retained runtime and broker. Control-channel loss rejects callers but does not release native custody: the actual Worker exit is the receipt that resolves the owner join. Runtime close refuses while execution handles remain, and failed domain cleanup is aggregated instead of clearing the source.

The already registered CLI cleanup scope, CLI worker-focused test and state-read worker are direct consumers. Their existing oracle proves ordinary command cleanup preserves borrowed lifetimes, while process-scope cleanup waits adopted plugin-resource disposal and final state reads before the shared database and Worker actually retire. Command return or control-port loss therefore cannot masquerade as native settlement.

### 35.2 Worker broker/store type contracts retain exact admission and cleanup custody

New frozen identities:
- `src/state/openclaw-state-worker-store.types.ts@8025540788b80382ab78f76b2924462da94b5152`
- `src/infra/sqlite-worker-broker.types.ts@dc06ffc41367d79ac9df23cb7373b8099b134aef`

These contracts make the Section 32 broker custody explicit. A shared-state store entry retains its captured runtime source/context, opening promise, exact actor identity, admission cleanup handle, active-operation count and native-loss listeners. Broker jobs separately retain request custody, operation admission, native settlement, maintenance/schema access, input-transfer ownership and caller currentness. Actor state keeps physical database path, state-access identity, close receipt, native-stopped receipt, retirement/settlement promises and failed cleanup state. A failed admission therefore has its own `SqliteWorkerAdmissionCleanup`; pathname cleanup is not allowed to stand in for exact actor custody.

The already registered broker capacity/lifecycle and SQLite admission-retention oracles remain the focused negative evidence: retained source/write-scope receipts survive until their last release, physical mutation invalidates old tokens, stale captured admissions cannot overwrite the newer physical identity, and retirement failure blocks replacement admission. These types carry custody; they do not create another database writer.

### 35.3 Shared-state native borrowers keep retirement behind the last current owner

New frozen identities:
- `src/state/openclaw-state-db-borrow.ts@110fa342365db59e7270a61e98d7ed556e66aa0d`
- `src/state/openclaw-state-db-borrow.test.ts@60ca10d5168c75467f21ddab5324b1a289694abc`

The canonical cache supplies database identity; this layer owns native references and retirement handoff only. Retain rechecks the captured canonical handle and rejects an owner already retiring. Writer release can transfer its retirement request behind outstanding read pins. Async retirement holds a synthetic reference while WAL maintenance joins; cleanup failure keeps the reference and calls `retainFailed` instead of making the database reopenable. A read pin may observe only the same still-open canonical handle and never runs inside an active native transaction.

Focused tests prove the writer retirement request survives when a read pin releases last, concurrent repeated release shares the same asynchronous WAL join, failed WAL stop keeps the handle borrowed and blocks replacement, and a changed owner cancels stale retirement rather than closing the successor. Read-pin tests also reject stale observation and release the original pin exactly once.

### 35.4 Shared-state integrity proof is revocable evidence owned by the physical database lifetime

New frozen identities:
- `src/state/openclaw-state-db-integrity-admission.ts@20b32e52ca5bd5eb363e304b99fbd00339b83356`
- `src/state/openclaw-state-db-existing-schema.ts@003b2ae94cc7482ba3ec5bd7dfd74b4cfe4e5d3f`

Integrity admission binds proof to the tracked physical database identity, schema/user-version tuple and a shared revision epoch. Schema mutation or native-handle replacement invalidates that epoch. A caller using `require-proof` fails closed when physical identity or epoch no longer matches; an ordinary verifier may recompute integrity, but publishes reusable proof only if the same epoch remains current, staging publication behind transaction commit when necessary.

The existing-schema consumer reuses only a current admitted runtime schema plus integrity proof. Otherwise it validates ownership/schema metadata inside a read snapshot, checks canonical runtime shape and absence of retired repair state, and publishes runtime admission only after validation. A warm handle still rebinds revocation custody for the proof it borrows. This remains validation evidence for the existing shared-state owner, not a schema or state writer.

### 35.5 Closure result

This recursion closes the remaining native-worker lifetime, exact broker-admission cleanup, shared-state borrower and integrity-proof identities exposed by the Section 34 scan. None imports or owns process-wide `closeOpenClawStateDatabaseAsync`; none creates a second shared-state writer. Worker exit, actor/native close receipts, WAL retirement and integrity proof are distinct settlement facts, and stale generations fail closed rather than being adopted by replacements.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


### 35.6 Retained native-worker direct consumer closes at plugin-generation custody

New frozen identities:
- `src/plugins/plugin-generation-worker.ts@37e1c105e82b163ed451631136571ebc604d345f`
- `src/plugins/loader-module-runtime.ts@cc0ff80558f8bfe06310f951c4c204a4ccde3c85`
- `src/plugins/plugin-generation-worker.test.ts@4455565818cd53faf3478e27ba35fbdc36386193`
- `src/plugins/loader.async-preparation.test.ts@d88c3e2ab48085dfe5d56ad125078a7d73b53969`

Default-branch search also exposed `src/tts/tts-settings-write-worker.ts`, but exact fetch at frozen `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` returned 404, so it is moving-main-only and remains excluded from this frozen Round87 evidence.

The new frozen production consumer is plugin generation. `preparePluginGenerationArtifact` captures the existing unbound retained-native-worker source and registers a preparation owner with `source.retain`; the owner closes every client, joins worker-pool resource disposal, settles native-admission state and only then releases capture roots. The loader acquires this worker only for async external-source preparation; synchronous CLI or already-retained/cache paths do not create a second capture. This is a direct consumer of the Section 35.1 Worker lifetime owner, not a new Worker or database lifetime authority.

Focused source oracles were read, not executed. They prove that unexpected Worker exit releases host preparation custody only after confirmed pool retirement, terminal native-source shutdown joins retained capture workers, lazy publication is handed back before Worker retirement, sibling captures keep distinct directories, and one plugin acquisition failure does not block its sibling. No new shared-state writer or process-wide close owner is introduced.

### 35.7 Shared-state integrity callers preserve physical-file proof; borrow ownership remains unchanged

New frozen identities:
- `src/state/openclaw-state-db-open.ts@f90ef3094b42fb50d7110b0ee804e74cda4ced96`
- `src/state/openclaw-state-db-startup-checkpoint.ts@309735b2bf2701a104fc1e9c32c2d61de486ebcf`
- `src/state/openclaw-state-db-existing-write.ts@017e5285db360bb3666f76f3a444345749b3f358`
- `src/state/openclaw-state-db-existing-write.test.ts@d39c294c09e346783a8f73f33213ab7a4f345c0f`
- `src/state/openclaw-state-db-checkonce.test.ts@49c56f4ee7ca87e00a0bbf950a70e0023a57a28b`

These are newly registered callers of the already-recorded integrity owner. Normal shared-state open verifies integrity before mutation unless current runtime-schema admission already supplies proof. Startup checkpoint uses a private connection, admits the exact catalog, verifies integrity before initialization, can carry clean-close proof across its temporary handle, and closes that private connection instead of publishing a second runtime owner. Existing-state writes intentionally avoid bootstrap/WAL-timer ownership: they require explicit maintenance/schema authority, verify the exact existing file, retain their own connection, and refuse writes after close.

The focused oracles establish negative behavior: QUICK maintenance proof is not lent to a FULL writer; a replacement appearing during lease acquisition is preserved and the stale writer is refused; WAL identity stays stable across admitted local/sibling writes; failed native close keeps cleanup custody and blocks later writes; startup-checkpoint clean-close proof is reused without repeating integrity checks; tracked header change forces readmission; and replacement bytes at the same pathname are validated as a new generation.

The direct `createStateDatabaseRetainer` caller remains `src/state/openclaw-state-db-cache.ts`, already registered earlier in Round87. No new non-duplicate borrower caller was found in this pass, so no second borrow owner is added.

### 35.8 Agent terminal, transaction, WAL, write and admission are one fenced owner chain

New frozen identities:
- `src/state/openclaw-agent-db-terminal.ts@1fd4b73c372be6fd6b60374720091a8bf0c8aa56`
- `src/state/openclaw-agent-db-transaction.ts@c74066a5425cad4b944db3c9203a0ef5acf919db`
- `src/state/openclaw-agent-db.wal.ts@f0a4ffe298ecf3df3069c2f4087a3e61168aa80a`
- `src/state/openclaw-agent-db-write.ts@e9c1202e550c2931b22c52adb002e7b60db5d5e6`
- `src/state/openclaw-agent-db-admission.ts@51f2477d111c24da91e0128ce090b6def84896d7`
- `src/config/sessions/session-accessor.sqlite-entry.ts@917148f35ae026da7bdfa3d0d56ede893b52426f`
- `src/config/sessions/session-transcript-hydration.ts@f561ef734a46dc8ad5b493695797ee202c734c7c`
- `src/state/openclaw-agent-db-readonly-scope.ts@5f832541a7df9c6f910d23acb464530d1e6569fe`

The terminal helper only reads the lifecycle owner's global terminal latch. A latched `SqliteIntegrityError` can be cleared only after a fresh native integrity check; the async path rechecks caller currentness before and after the check and clears only the same observed failure. Terminal state therefore cannot be silently bypassed by a fresh open.

The yielding transaction helper captures environment, opens and retains the exact canonical database, yields only for BEGIN admission, rechecks that the same handle is still canonical before the write, and never replays an admitted mutation or post-commit publication. The WAL helper captures exact file identity, retains the database while maintenance is in flight, runs maintenance through the existing worker-write/execution authority, and couples cancellation to the registered async resource; release/unregister occurs only after that worker settlement.

The write helper freezes platform-aware environment and resolved pathname before waiting for admission, checks the physical identity again after the wait, rejects a borrowed handle that closed or changed instead of adopting its replacement, and requires a synchronous mutation callback. If a malformed callback returns a promise, it still joins that callback before rejecting so the next writer cannot overtake a hidden tail.

The admission owner is the single driver around the canonical private open generator. Each pending open keeps its own abort controller, promise, operation count and publication borrow. Main-thread cold opens delegate integrity preparation to the Worker, but resume only after terminal/deletion/creation/repair authority is revalidated. Worker admission retains exact execution identity and schema/integrity publication; coalesced callers do not inherit one another's cleanup authority. Last-caller settlement releases the publication borrow; failed cleanup or stale generation remains fenced rather than becoming a replacement admission.

New direct consumers preserve that chain. Session-entry writes use the yielding transaction path while retaining their captured execution/source predicates. Transcript hydration revalidates the terminal latch before reading and carries transcript admission/read fences. Retained read-only scopes assert terminal-open permission on reuse and explicitly close an idle reader before a writable admission can open the same file.

### 35.9 Focused agent oracles fail closed on stale identity, environment drift and close failure

New frozen identities:
- `src/state/openclaw-agent-db-write.test.ts@fc1d523a1d190ecbfbb90a3030bd098c56e45819`
- `src/state/openclaw-agent-db-write.env-capture.test.ts@ba66f38f550c58725c93ed01cd64f3ab3080c1fa`
- `src/config/sessions/session-accessor.sqlite-entry.env-capture.test.ts@84315ad7b372f45c0d90bb55a11726f493f5d7e1`
- `src/config/sessions/session-accessor.sqlite-entry-revalidation.test.ts@a0059329c50cf24720aa3b779a87d37c76df52d7`
- `src/state/openclaw-agent-db.test.ts@1965d916c4b3a3095afb1b1cb6dbacad019ec250`

The already registered `src/state/openclaw-agent-db.worker-admission.test.ts@84d1427554f2b531eb3dfd4ec8807827829bef7c` was re-read as supporting oracle and is not duplicated in the identity list.

The write-admission tests prove FIFO survives aliasing while stale worker reservation is rejected, replacement physical identity cannot receive a queued write, a closed borrowed handle is not rebound, state-directory/environment changes after enqueue cannot retarget the operation, and an accidentally async callback settles before rejection and before the next writer. Environment-capture tests prove Windows case-folded and POSIX case-sensitive state-dir inputs are frozen before admission; canceling one coalesced waiter does not abort the shared physical integrity check.

Session-entry tests prove the captured environment object is the one delivered to the worker/history/maintenance path after later ambient replacement, and commit predicates reject closed/replaced connections, identical-byte physical replacement, changed owner assignment, and stale lifecycle targets. The integrated agent-db oracle additionally proves a failed WAL close retains the durable lease until a successful retry, Worker-close settlement can report the recoverable close error while releasing its lease, creation-claimed handles close when the claim settles, and maintenance fences new writers while prior leases remain live.

Together these tests distinguish terminal latch, operation admission, Worker/native settlement, WAL close and durable lease release. None of those facts alone is treated as proof that another has settled.

### 35.10 Closure result

This pass found new non-duplicate frozen evidence and therefore extends Section 35. The only newly exposed retained-native-worker production consumer at the frozen ref is plugin generation; the default-branch TTS writer is not present in the frozen source. Shared-state integrity gains three newly registered callers, while native borrow still terminates at the already registered cache owner. The remaining agent terminal/transaction/WAL/write/admission family resolves to the existing canonical agent database lifecycle, worker execution admission and exact physical identity; it does not create a second agent writer or a process-wide shared-state close owner.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. The test files above are source-level negative oracles only and were not executed in this read-only round. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## 36. Agent worker operation registry and connection-bound domain recursion

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`. This pass follows the still-unregistered production consumers exposed by Section 35.8/35.9 through the agent worker operation registry, without widening the frozen source or treating ordinary business kernels as new lifecycle owners.

### 36.1 Session entry writer and recovery remain consumers of the admitted agent writer

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-entry-write.ts@530b6a896146a1bdc263f3b6107956105301d2bb`
- `src/config/sessions/session-accessor.sqlite-entry-mutation.ts@d7f771df2fc2b586036355b5a21d364293231153`
- `src/config/sessions/session-accessor.sqlite-maintenance-store.ts@a0b50c2d288a3a602c5eea1f450275669adf13e4`
- `src/config/sessions/session-accessor.sqlite-write-operation.ts@b8ca6e93718adf94daa6d4721629228b5393be62`
- `src/config/sessions/session-accessor.sqlite-recovery.ts@565b7cbfeddcc63668720fb1c0aa4cbf7d9a3f88`
- `src/infra/database-engine/session-entry-write.conformance.test.ts@d7a8dd0bff8c12220bf026657aad81a7bc43cc14`
- `src/config/sessions/session-entry-recovery.worker.test.ts@b3a72c57cabd59362c3faf469bafc87023287c8a`
- `src/config/sessions/session-accessor.recovery.test.ts@474741531b4ae68de0eab6459c600d80c9a1fad5`

`writeSessionEntry` performs canonical-key/lineage validation, same-transaction canonical row reads, scoped node/window/snapshot/conversation mutation and staged cache/publication updates on the already-open `OpenClawAgentDatabase`; it never acquires or closes the database. Entry mutation explicitly documents that the caller owns transaction admission and post-commit publication. Maintenance batches reuse the same writer and revalidate candidate/protection facts before commit. The write-operation union is diagnostic classification only.

Restart-tombstone recovery runs through the existing session-entry worker and retained worker transaction admission. It publishes only after the retained native admission reports completed settlement plus a matching commit receipt. Result loss after native commit is reconciled from that receipt; missing receipt or unknown native settlement becomes `outcome-unknown` rather than a replay. Focused recovery tests prove transaction/commit authority revocation refuses the mutation, committed recovery survives result loss and independent owner retirement, and uncertain native settlement remains uncertain.

The database-engine conformance oracle proves archive batching matches sequential writes, rollback restores the prior state, refused/revoked worker commits do not publish, and actor/patch pairs do not create hidden extra writes. These are writer consumers, not a second writer owner.

### 36.2 Agent operation registry is lazy routing; transaction and admission remain injected authority

New frozen identities:
- `src/state/openclaw-agent-execution-operations.ts@b70693f9b86af6555736bcd9f9e3950939d3ff7b`
- `src/state/openclaw-agent-operation-context.ts@5e6a7e79255f00befca69729c13b5c952986bcba`
- `src/state/worker-operation-registry.ts@84ddd568e26bbb07586389f997d590ebf6bb2762`
- `src/state/openclaw-agent-execution-operations.test-support.ts@860e7501cf11283c0cdf9ae5a3946e1f89cc3d35`
- `src/state/openclaw-state-worker-registry.test.ts@99d892fd9e5c9d5ca001f05e4049d826a7f8cf7b`

The registry lazily loads a command namespace and dispatches only exact prepared operation names. It has no database-open, transaction or close capability. `AgentWorkerOperationContext` carries the canonical executor's `open`, `admit`, `writeTransaction` and optional exact shared-owner settlement callback into handlers. The already registered `openclaw-agent-execution.worker.ts@a533936aa520fc1770abd779a9fb9baec327ac41` constructs this context after checking physical identity, and its transaction wrapper rejects a different native handle before invoking the handler.

The focused registry oracle explicitly supplies context functions that throw if the registry attempts to open a database or transact, proves only the requested domain is loaded, and proves an unprepared exact operation cannot execute. Therefore the large operation-loader map does not itself create another writer, admission service or lifecycle owner.

### 36.3 Maintenance, schema refresh and connection-bound domains keep canonical ownership

New frozen identities:
- `src/state/openclaw-agent-execution-maintenance.ts@acb75cac5a3cd26c1c45107ee7979f82be9998bc`
- `src/state/openclaw-agent-execution-domain.ts@0eb392c200c2b70913a58cad5c47e72ad6ca6eb1`
- `src/state/openclaw-agent-db-schema-refresh.ts@1eaa87398b3aa7032dafcfbc16fe2c6cd99e875e`
- `src/state/openclaw-agent-db-schema-refresh.test.ts@fd11999d7d7ea48f4a7247e861b3aa5a163fb951`
- `src/state/openclaw-agent-db-validation-cache.test.ts@980f2051cab6cf3c6af0ab9aacc03d879be2328b`

Maintenance preparation retains only its bounded snapshot and releases it on replacement/cleanup. Metadata commit opens the executor's writer, refuses a different canonical database, stages a commit receipt, and obtains transaction/commit admission from the injected context. Startup FTS repair additionally requires the one exclusive live agent lease under the shared-state write fence.

Schema refresh explicitly re-admits an already-retained connection without replacing its native generation or lease. It refuses an active transaction, revalidates schema metadata/integrity, and invalidates validation on failure. Focused tests prove derived DDL can be republished without replacing the retained reader, a stale/replaced schema is readmitted, rollback does not certify canonical proof, delayed proof cannot revive after revocation, and replacement receives distinct physical validation.

The domain owner binds only a static local module and borrows `context.assertCurrent()`'s canonical connection. Admission restrictions must dispatch exactly one grant at the original stage. Domain cleanup first revokes its authority, requires synchronous backend close, and cannot admit a transaction after revocation. A failed/unfinished binding is surfaced by `assertSettled`; the domain never owns the database close.

### 36.4 Dynamic operation consumers use injected admission; no new database close owner was found

New frozen identities with direct lifecycle/admission use:
- `src/config/sessions/session-native-binding.worker.ts@db80768739b49d455d5109adecb121372ddde72d`
- `src/config/sessions/session-entry-patch.worker.ts@07b5981995d3c9cf8ea3873ea9159d20974172bc`
- `src/config/sessions/session-turn.worker.ts@4dbf1d36d210b49b3d9ee6bd56e4a5cf19269f17`
- `src/acp/runtime/session-meta-entry.worker.ts@fbc2e983f8c10655ed7082182a086bdcd03c66a7`
- `src/config/sessions/provider-review-store.worker.ts@740126306078995ad62229c7f00196ad4b2e81f0`
- `src/config/sessions/session-pending-input-operations.kernel.ts@61b2f9f4a9d0df2962b8550f91e412d7634e6968`
- `src/config/sessions/session-pending-input-withdrawal.worker.ts@2704dfdff20bd44ed31dd85266f14b53c4626bed`
- `src/config/sessions/session-pending-input-history-reconcile.ts@78914f148d0dbf37a161f18fb849d2f602be111b`
- `src/config/sessions/session-history-archive-pruning.worker.ts@c3acadb82a439b20bcb8745f3269456fdccdffc9`
- `src/agents/auth-profiles/store-agent.worker.ts@6a668b093534d74f2303f7fe7b5eaa2db0d90fbe`

The remaining dynamic modules under `createAgentWorkerOperationLoaders` were also exact-ref read. Ordinary read/replacement-state/conversation/trajectory kernels either have no lifecycle call or consume only the injected `open`/`writeTransaction`/`admit` context. The files above directly request admission or stage commit receipts, but none creates a new worker broker, native database generation or close authority.

Native-binding settlement is the highest-risk direct consumer because it coordinates agent and shared-state writes. It explicitly receives `writeSharedTransaction` from the canonical executor and stages exact transfer receipts. Entry patch/turn, ACP metadata, provider review, pending inputs and archive pruning similarly request transaction/commit grants at the mutation boundary. Auth-profile read uses a preparation MessageChannel only to receive the host-supplied read field and closes those ports after preparation; it does not own the agent database.

### 36.5 Focused negative oracles distinguish commit, delivery, settlement and close

New frozen identities:
- `src/config/sessions/session-native-binding.settlement.test.ts@021b30018cfa25e57de49a05cf30f9beca9e11b4`
- `src/config/sessions/session-native-binding.reclamation.test.ts@f31f2580afc9ed628146c1ac070cb46cdb926faf`
- `src/config/sessions/session-entry-patch.retention.test.ts@c469ae802025ce5821be72b100e589796ff2387d`
- `src/config/sessions/session-entry-cohort.worker.test.ts@4d5d6cb5c882165c2c89aa47cffae7679e9b4037`
- `src/config/sessions/session-pending-input-operations.test.ts@c68bd4996404552d2d9aa91b52e9e00a71aaf904`
- `src/config/sessions/session-pending-input-withdrawal.test.ts@5e5c1b5e15bb21fce840a03bdb86fcd58e829f67`
- `src/config/sessions/session-transcript-append-postimage.test.ts@744127ad72a01b9b7c9121fd42231b94502d3fe5`
- `src/state/openclaw-agent-db-readonly-scope.test.ts@faae659f204b73c4effd8ea78da10e346554b7c0`
- `src/infra/sqlite-readonly-worker.model-auth.test.ts@0f182913529f79f3054423bb5df20b5da637497c`

The already registered `src/state/openclaw-agent-worker-store.test.ts@a8d5a1fd1284a2e5522021e58370bd3c04ea0f86` and `src/state/openclaw-agent-worker-store.env-capture.test.ts@9b1b4eee583263b2b1180839f5d01f331e73f911` were re-read as supporting domain-close oracles and are not duplicated in the identity list.

Native-binding tests prove result loss plus storage-settlement failure is `outcome-unknown`, a mismatched commit receipt cannot finalize custody, worker loss after either shared or agent commit does not replay/compensate, rollback keeps original rows/binding, and final authority is revoked only after acknowledged deletion. Pending-input tests prove stale custody and lifecycle changes refuse completion, a commit refusal rolls back, lost reply/unknown settlement does not duplicate a mutation, and a confirmed committed withdrawal can be recovered from retained native admission. Entry-patch tests distinguish replacement, rollback, revoked lease and retained trajectory cleanup. Read-only auth tests prove admitted reads drain before maintenance close and cannot reuse a closed scope.

The already registered agent-worker-store oracle further proves a publication can retain its database while root close remains pending, failed publication/domain cleanup is retryable, cleanup admission refusal cannot commit a close-time mutation, and a fresh worker is required after terminal failure. These facts keep database close, domain cleanup, native commit, result delivery and operation settlement separate.

### 36.6 Connection-bound backend close only releases domain-local state

New frozen identities read through the domain factory surface:
- `src/config/sessions/session-fork-domain.worker.ts@1a1039e4a281ab8e111b2a48efa3b513b0087b2d`
- `src/session-cards/progress-card-store.worker.ts@b64e91738285cda596e907551ca0da00af2ecd81`
- `src/agents/harness/context-engine-turn-outbox.worker.ts@3734e5aef62ccb42efaca5230692aff0b38b7a23`
- `src/boards/sqlite-board-store.worker.ts@b5def508fa1a35a7f4e625ced9faba64aa40b838`
- `src/agents/auth-profiles/inline-usage.worker.ts@61bf2c6c4bc530cc222397e9d75bc22d55c54097`
- `src/config/sessions/goals-operations.worker.ts@6e051c2e7840113a14b8880e43b63973216ed2a6`
- `src/agents/plugin-model-catalog.worker.ts@df3005e5454613b194aed7413573df08b0a671f0`
- `src/gateway/worker-environments/transcript-commit.worker.ts@9254c2d883d7523e248d6684b76ee0bdb6fa6a85`
- `extensions/memory-core/src/memory-entry-origins.worker.ts@739181547f945eafeed93a14f9427f8ce43fe672`
- `src/config/sessions/session-sharing-store.worker.ts@336463030dd13417c4dec7088f8f13f429ee14f3`
- `src/config/sessions/session-lifecycle-projection.worker.ts@fc3802154834a866848e69ed78543bf342c18c41`
- `src/agents/sessions/session-manager-metadata.worker.ts@9949b7fc7c0f5509713bfc03a148d60aec8dbf04`
- `src/config/sessions/session-message-rewrite.worker.ts@9565b87c2030eee94e3986879a2809df5399c339`
- `src/config/sessions/session-transcript-projection-publication.worker.ts@3de0b42301943511019e076d98d9a970494a91e5`
- `src/config/sessions/session-accessor.sqlite-transcript-reports.worker.ts@aeccb6297d072b0c9464c3246fd43d9f1404e3ce`

`extensions/memory-core/src/standing-intents.worker.ts` and `extensions/memory-core/src/memory/manager-publication.worker.ts` were already present in Round87 and were not re-registered.

These backend factories all receive the canonical connection from the domain owner. Where a backend exposes `close()`, it releases only domain-local prepared state or auxiliary resources: transcript commit and transcript reports clear their prepared plans, while session-manager metadata explicitly documents that it neither opens nor closes the database. The domain owner revokes admission authority before invoking such cleanup. No backend in this scan became an independent agent database close owner.

### 36.7 Closure result

Section 36 closes the new frontier from the Section 35 agent admission chain through session entry writes, restart recovery, the lazy worker-operation registry, maintenance/schema refresh, connection-bound publication domains and their direct admission-consuming handlers. The canonical native database generation, durable lease and physical close remain owned by the existing agent lifecycle/executor. Registry handlers and domain backends borrow that owner; their local cleanup cannot reopen, replace or independently close it.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All test files above are source-level oracles only and were not executed in this read-only round. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 37 — connection-bound domain production consumers and request-class closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This section follows Section 36's connection-bound backend frontier into the direct production consumers and the request-class/worker-store producer path. Previously registered backend, executor and worker-store identities were re-read where needed but are not duplicated below.

### 37.1 `database.domain.bind/publish/close` is produced by the existing worker-store owner; request classification is diagnostic only

New frozen identities:
- `src/infra/sqlite-worker-request-class.ts@38c3192d19125098292285ce9087048085dbc871`
- `src/infra/worker-request-diagnostics.ts@0a3fe3cdb8be124d2c259c5975160c6f5db237c0`
- `src/state/openclaw-agent-db.permissions.test.ts@3f196c01d52084fde30bcd3384bb0eda83b97164`

Re-read without duplicate registration:
- `src/state/openclaw-agent-worker-store.ts@918ab97a3a7142cd3cb3b4fb8c5c6f74e133efab`
- `src/state/openclaw-agent-execution-domain.ts@0eb392c200c2b70913a58cad5c47e72ad6ca6eb1`
- `src/state/openclaw-agent-execution.worker.ts@a533936aa520fc1770abd779a9fb9baec327ac41`

The one-shot producer `executeOpenClawAgentWorkerPublication` emits `database.domain.publish` through an already-admitted `AgentDatabaseExecutionScope`. The multi-command publication path in `openclaw-agent-worker-store.ts` allocates one binding id, sends `database.domain.bind`, serializes exact `database.domain.execute` commands, waits for all accepted command promises, then attempts `database.domain.close` from a fresh cleanup scope whose admission can only grant `prepare`. That cleanup scope cannot open storage or admit a transaction. A refused cleanup admission retires the captured native generation instead of treating cleanup refusal as proof that durable work rolled back.

The request-class table gives `database.domain.bind`, `database.domain.publish` and `database.domain.close` stable diagnostic labels below the database contracts. `worker-request-diagnostics.ts` unwraps the inner command for domain execute/publish metrics and special-cases transcript-projection commands to bounded labels; it never grants admission, opens a database or closes one. The permissions oracle proves that a warm borrowed write still rolls back when commit authority is refused and that backend settlement is checked independently. This classification path therefore does not introduce a second writer or lifecycle owner.

### 37.2 Session sharing is a direct consumer of the canonical execution owner and preserves unknown outcomes

New frozen identities:
- `src/config/sessions/session-sharing-store.async.ts@efce88278bbb5404058722c132194c6b2dc70471`
- `src/config/sessions/session-sharing-store.worker.test.ts@33a6ee577545ceca368dac2159427e988e8a7a89`

The file-backed path captures `OpenClawAgentDatabaseExecution`, joins the existing write-admission FIFO, prepares a replacement only when the canonical execution has no physical identity, and opens the already registered sharing backend through `runtimeProcessEntrypoints.sessionSharingStore`. Host-side admission captures the exact candidate receipt before commit, retained admission settlement is observed separately from the ordinary worker reply, and cache/session-row publication happens only from committed facts that match the admitted candidate. `worker.close()` is in the inner `finally`; `execution.release()` is in the outer `finally`.

A lost ordinary reply after dispatch is not replay permission. Missing/conflicting receipts, a lost result, or a close that happens after native commit remain `outcome-unknown`; publication custody is invalidated rather than synthesizing a second mutation. The focused sharing oracle covers aliased physical owners, stale manager authority, participant/owner publication, and lost-reply cases with valid, conflicting and missing receipts, including host close before the synthetic failure. Only the valid native receipt settles the committed mutation.

### 37.3 Session message rewrite uses a one-shot domain publication; stale generations stay fenced

New frozen identities:
- `src/config/sessions/session-message-rewrite-domain.ts@7653df1ff0b554ef6bb0f5c116cc41f9002e5fb4`
- `src/config/sessions/session-message-rewrite.test.ts@6f758eba67fa5685eeb6c3b8cb520d536da13ed6`
- `src/config/sessions/session-actor-message-rewrite.test.ts@681cc901ae5fb7264de2ff98e080dbfcaa656f7e`

`executeSessionMessageRewriteOperation` does not own a connection. It receives the caller's admitted execution scope, creates a fresh publication id and invokes `executeOpenClawAgentWorkerPublication` against the registered message-rewrite backend. The rewrite oracles prove that the exact prepared payload/generation is committed without an authoritative re-read, that replacement or reset between prepare and commit is refused, stale writer fences cannot publish, and a response lost after commit retains the committed receipt rather than replaying the mutation. Stale-preimage and commit-refused cases remain non-committed.

### 37.4 Transcript projection publication separates native publication, reconcile worker tasks and physical close

New frozen identities:
- `src/config/sessions/session-transcript-reconcile.ts@b2dc846174ea53e9ccdd84824ca6ea8f4572cc4a`
- `src/config/sessions/session-transcript-projection-writer.ts@bafc4b0beef56b8be1b0a1940bb45d4a4b6c6537`
- `src/config/sessions/session-transcript-index-status.test.ts@5cec4ad153d43f934c5b98e144686ac8432131ea`
- `src/config/sessions/session-transcript-reconcile.continuation.test.ts@0d66ab839c64af9dca8da4c3cd6f98639a21f4fe`
- `src/config/sessions/session-transcript-reconcile.publication.test.ts@c7fcdef32fbd4b2580cb951a1fca8e6149944ae9`

When an execution owner is available, reconciliation opens the connection-bound projection backend and uses it only as a `ProjectionPublisher`; the separate reconcile task prepares/parses plans and sends chunks over its own task port. `publicationClient.close()` runs in the pass `finally`, while the captured execution is released by the enclosing operation. When no execution owner exists, the fallback path uses the existing exclusive session-write owner instead of creating a parallel database owner.

Projection status and write helpers reuse the same backend and close/release it after the bounded read/write scope. Dirty state is durable, so retirement or cancellation leaves discoverable pending work rather than authorizing replay. Focused oracles prove admission is reacquired after contention, cancelled admission rolls back, committed finalization is delivered before close allows a physical successor, retired scheduled owners are refused, and an explicit fresh repair can later resume the durable backlog. Worker task completion, projection publication settlement and database close are therefore distinct facts.

### 37.5 Session-manager metadata and unbound transcript append borrow the same canonical worker store

New frozen identities:
- `src/agents/sessions/session-manager-metadata-runtime.ts@49b2c2e318d1863a78a4adf74c26e70edd493220`
- `src/agents/sessions/session-manager-message-runtime.ts@0dc744269642d6bd790eb91385c970cb1437d348`
- `src/agents/sessions/session-manager-maintenance-async.test.ts@de408b9c57ae890f7519bdd2978f2a7a2ec92c09`

The metadata runtime prepares the existing execution through `runOpenClawAgentWorkerWrite`, opens the registered metadata backend, applies host-side admission/publication hooks, and closes the publication worker after the operation. If the operation already completed and cleanup alone fails, the cleanup failure is diagnostic and does not erase the committed receipt; if the operation itself failed, cleanup failure is aggregated with it.

The message runtime keeps the resident actor path separate. Only an unbound SDK append captures `OpenClawAgentDatabaseExecution`, opens the metadata backend, validates the pending-input publication, closes the worker and then releases the execution. Multiple lifecycle failures aggregate, while a committed transcript message is surfaced as committed even if later cleanup/notification fails. The maintenance oracle exercises queued rewrites, stale writer fences and post-commit target rebinding, preserving committed state without granting stale publications.

### 37.6 Worker transcript commit and transcript reports close only their publication scopes

New frozen identities:
- `src/gateway/worker-environments/transcript-commit.runtime.ts@047e3e64748ec70bbb140a39163287d230a647ed`
- `src/gateway/worker-environments/worker-turn-rpc.transcript.test.ts@92b23eed071f0f35cb81fb1c61f21ca196004e71`
- `src/config/sessions/session-accessor.sqlite-transcript-reports.ts@9a6fb6fe2393e290f0f34d6c1142b8860081d22d`
- `src/config/sessions/session-accessor.sqlite-transcript-reports.test.ts@eecc543ac450c5ed30de034c788c35f6ba941d52`

The worker-transcript runtime captures the canonical execution, opens the registered transcript-commit backend, prepares and commits through the bound writer, and closes the worker plus releases the execution after the result. Cleanup failures after a committed batch are logged/aggregated without turning that committed batch into replay permission. The separate transcript commit ledger keeps begin/replay/recover semantics, and the focused RPC oracle preserves committed progress when the worker claim is released or replaced while refusing a mismatched placement.

Transcript reports pin logical writer authority and physical source identity before yielding. The report path captures the canonical execution, prepares replacement under that owner, binds the registered report backend, and nests two settlement wrappers so `worker.close()` and `execution.release()` are both observed. A cleanup-only failure after a committed report cannot replace the committed result with a replayable failure. Focused tests reject byte-identical physical replacement and redirection to another store, fence stale lifecycle claims, preserve committed fallback reports, and retry selection only on a proven no-write revision mismatch.

### 37.7 Closure result

The direct production consumers exposed by Section 36 all converge on the existing canonical agent execution/worker-store owner. Their `close()` calls settle connection-bound publication state; they do not own the physical agent database close. Request-class diagnostics are observational only. Across sharing, rewrite, projection, metadata, worker transcript and report paths, native commit, result delivery, publication settlement, worker/task retirement and physical database close remain separate facts, and uncertain post-commit outcomes are not silently replayed.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. The tests above were read only as frozen source oracles and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 38 — session mutation callers, terminal/WAL infrastructure, and legacy admission compatibility closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). The Section 35 agent terminal/transaction/WAL/write/admission owner files and Section 37 connection-bound backends were re-read at their frozen identities and are not duplicated here.

### 38.1 Session collaboration facades remain consumers of one admitted mutation owner

New frozen identities:
- `src/config/sessions/session-sharing-store.ts@d6251ac650837d218fd862aa8aae1e0fd1b94951`
- `src/config/sessions/session-involvement-store.ts@4416a6969c38aa9134477e4ca851a52b59ced4b9`
- `src/config/sessions/session-metadata-write.async.ts@6b0fd6725c48bd1be81aa85f9120dfc5f83cc344`
- `src/config/sessions/session-group-categories.ts@652f706e9fead5a8e2ab0062c0d167eb5eb06f14`

The sharing read facade uses the existing projection/history reader or resident actor; it does not acquire a writer. Membership, involvement, owner/suggestion and group-category mutations all converge on the already registered `runSessionCollaborationWrite`. Host callbacks only publish committed row facts after the retained native receipt settles. Category preparation keeps prepared rows in the bound backend and crosses the boundary with target keys; target authority is rechecked before apply. No facade opens/closes an agent database or creates a second transaction/admission owner.

### 38.2 Manual compact, rewrite, event and correction share the same session-entry worker/execution owner

New frozen identities:
- `src/config/sessions/session-manual-compact.ts@85b8f3f0a5118d87d0a4af77db79ea08a0af7a51`
- `src/config/sessions/session-message-rewrite.ts@b0f8a466053b518c142f0d25441df66c4602c6c3`
- `src/config/sessions/session-transcript-correction.ts@ecd04b1e7db732a6733d5e0b3771c8a7a8fe4941`

Re-read and deduplicated:
- `src/config/sessions/session-transcript-event.ts@9960c0344299449ee9b90ca4c276b9e69a5640ed`

File-backed manual compaction, prepared message rewrites, event append and transcript correction all use `runSessionEntryWorkerOperation`, which passes its already captured `AgentDatabaseExecutionScope` into `executeSessionMessageRewriteOperation`. Prepare/read work is fenced separately from commit; exact source predicates, lifecycle/writer claims and physical source identity are rechecked before commit. Correction selects exact serialized rows and refuses add/remove semantics; a stale generation cannot overwrite a successor. Event append publishes projection reconciliation only after the committed receipt. The in-worker/actor paths retain their existing owner instead of introducing a host-side fallback writer.

### 38.3 The terminal latch is a lifecycle fact holder, not a database owner

New frozen identities:
- `src/infra/sqlite-terminal-open-latch.ts@8036ceec18321a2184016807905d3e1bade97326`
- `src/infra/sqlite-terminal-open-latch.test.ts@016436c366dccdfbb2f6397e2f78f94a9f8a7aa0`

The generic latch stores one terminal failure per resolved path and invokes the lifecycle owner's `closeByPath` callback when recording damage. Clear operations remove only the recorded fact. The focused lock oracle proves terminal-failure disposal observes and preserves the reopened writer's POSIX lock until the supplied close callback actually closes that writer. Therefore a terminal latch does not itself own a connection and cannot silently dispose an unrelated generation.

### 38.4 WAL write admission delegates maintenance without lending transaction custody

New frozen identities:
- `src/infra/sqlite-wal-write-admission.ts@05c6ebd65a736b07f525aff6eed630ffebddde98`
- `src/infra/sqlite-wal-checkpoint.ts@35a44f96fd509e9f0a1055b50b6fa1874cb325b4`
- `src/infra/sqlite-wal.agent-admission.test.ts@1358bfc022fc03831efed7277f90ec52ec0283c7`
- `src/infra/sqlite-wal.checkpoint-tick.test.ts@fd8a29a69a3f0e72f98d46fb225579456123b5e6`
- `src/infra/sqlite-wal-checkpoint.test.ts@0f6bf6f537991ec8d601d303ac045d54702a9365`

`registerSqliteWalWorkerMaintenance` disables native autocheckpoint and delegates bounded periodic maintenance to the retained worker owner. The scheduler serializes one pending pass, yields between vacuum units and returns both native lock and FIFO custody between units. Worker-side deferred admission stores at most the pending maintenance operation; `flush` must revalidate current authority and verifies that no transaction was left unsettled, while `cancel` removes the pending wake without claiming rollback.

Checkpoint state is diagnostic/maintenance state: native checkpoint outcome is recorded before listeners are called; listener failure cannot change the outcome. Relayed observations keep monotonic native-order facts across worker/host generations, distinguish blocked/error/complete, and never infer ownership from diagnostics.

Focused oracles prove periodic agent maintenance waits behind foreground admission, a closed/replaced generation performs no stale maintenance, retained worker timer writes execute only while the parent grants admission, retire/revoke prevents later timer writes, delegated checkpoint wakeups are cancelled at shutdown, native close joins admitted maintenance, older relayed checkpoint facts cannot replace newer ones, blocked-reader health is preserved, and explicit reader custody remains independent from connection metadata.

The shared-state WAL caller `src/state/openclaw-state-db-cache.wal.ts@8e7ebf8139af85b89ed80ec35774590a156181b2` was already registered earlier in Round87 and is not duplicated.

### 38.5 Agent integrity/lease oracle keeps clean-close certification stricter than handle close

New frozen identity:
- `src/state/openclaw-agent-db.integrity-lease.test.ts@869a7dfa8c668c659b81a4538dc4b2a602d0ff6b`

This source oracle proves clean-close is published only after the final process/lease is gone and WAL state is checkpointed cleanly. A killed process, stale lease, failed checkpoint, native close failure, pinned reader or same-process read lease keeps verification dirty. Final close can certify only when the WAL is empty; stale process cleanup cannot certify a surviving handle. Fresh read-only admission remains blocked by quarantine/schema-version terminal facts until the exact generation is repaired. These negative cases reinforce Section 35's existing lifecycle owner rather than creating a new one.

### 38.6 Legacy SDK access is an explicit reviewed compatibility boundary, not a production ownership fork

New frozen identities:
- `src/plugin-sdk/sqlite-runtime-legacy.ts@5a8de5ba7b16e23efe8c1e636c6d9d6d25476f03`
- `scripts/lib/database-worker-compat.mts@ca295a6c1eccdaa0b97a14336d1d2eeefc7d9c79`
- `test/scripts/database-worker-compat.test.ts@e9ca65d6378dca3530f8b75522d262f628af6163`

The legacy SDK wrappers emit deprecation diagnostics and forward writable access to the same canonical agent database/write-admission APIs; the released `sqlite-runtime.ts` barrel was already registered and re-read. The compatibility scanner treats only exact reviewed deprecated declarations as compatibility boundaries, follows value-reference reachability, and reports a violation if a reviewed native operation becomes reachable through a live or escaped kernel. Its focused source tests reject bundled runtime references, same-file/live forwarding and escaped reviewed APIs while permitting only exact reviewed one-shot compatibility entries. This boundary therefore preserves compatibility without becoming a second owner or a sanctioned fallback around worker admission.

### 38.7 Re-read deduplication of the Section 35 agent owner chain

Re-read at the frozen source and intentionally not re-registered:
- `src/state/openclaw-agent-db-terminal.ts@1fd4b73c372be6fd6b60374720091a8bf0c8aa56`
- `src/state/openclaw-agent-db-transaction.ts@c74066a5425cad4b944db3c9203a0ef5acf919db`
- `src/state/openclaw-agent-db.wal.ts@f0a4ffe298ecf3df3069c2f4087a3e61168aa80a`
- `src/state/openclaw-agent-db-write.ts@e9c1202e550c2931b22c52adb002e7b60db5d5e6`
- `src/state/openclaw-agent-db-admission.ts@51f2477d111c24da91e0128ce090b6def84896d7`

Their Section 35 conclusion remains unchanged: terminal revalidation, yielding transaction admission, WAL maintenance, synchronous write wrappers and private-open admission are one fenced owner chain. Direct production searches did not reveal a new independent agent database close owner, worker broker owner or process-wide shared-state close owner.

### 38.8 Closure result

The newly exposed session mutation callers, terminal latch, WAL admission/checkpoint infrastructure and legacy SDK compatibility boundary all route back to previously established canonical owners. The scan found no second database writer, no new physical close owner, and no replay permission created by lost replies, cancellation, terminal failure, WAL retirement or compatibility forwarding.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All tests above were read only as frozen source oracles and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 39 — WAL-budget consumers, history-reclamation custody, and concrete legacy SQLite compatibility callers

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Section 35's agent database owner chain and Section 38's WAL/terminal primitives were re-read where required and are not duplicated.

### 39.1 Checkpoint observations gate physical-budget retry; they do not grant mutation authority

New frozen identities:
- `src/config/sessions/session-history-budget-state.ts@98f68dae45804fb8f6730809ec400df75addd3c1`
- `src/config/sessions/session-accessor.sqlite-mutation-worker.runtime.ts@ada267318473e3074f694abe183f9ea1447abe84`
- `src/config/sessions/session-accessor.sqlite-mutation-worker.runtime.test.ts@37da85166d782fe71c3268fa3acf43ff76962bab`
- `src/config/sessions/session-history-wal-budget.test.ts@643182d4e74fd253208cddcd19a441f6783225fb`
- `src/config/sessions/session-accessor.sqlite-reclamation-wal.test.ts@4320915885039e6e5fca969d87f8416a8d7b1a2a`
- `src/state/openclaw-state-db-cache.types.ts@a46066eb3af08e80cd9e62005dcb304e764725e1`

The history-budget state keeps only per-store throttle/blocked-checkpoint facts. Its global WAL listener clears a checkpoint gate only when the same physical database reports a newer completed checkpoint after the recorded monotonic boundary. It never opens a database, grants admission or interprets a checkpoint as deletion permission.

The mutation worker retains the canonical agent claim/lease through each cold-mutation or reclamation request. Worker write admission surrounds the native transaction; source identity and lease are checked again before mutation. WAL observations are relayed only when they belong to the current request. Close uses the existing `settleOpenClawAgentDatabaseWorkerClose`, then releases the claim; a pooled task can close shared state only inside the parent/coordinator admission. Refusal is returned as a no-write result only when the current handle has no unsettled transaction.

The focused oracles prove that blocked or older checkpoint facts cannot unblock a physical-budget gate, reclamation resumes only after a newer completed checkpoint, worker result settlement precedes later maintenance admission, buffered admission replies drain before idle collection, and close joins the retained worker rather than treating IPC result delivery as native settlement.

The already registered `src/state/openclaw-agent-execution-close.ts@ac881adcbea18aaf22dff04bf4d1e645c651a58e` remains the direct agent-close checkpoint consumer and is not duplicated.

### 39.2 History eviction is a serialized physical-budget owner over existing lifecycle/reclamation owners

New frozen identities:
- `src/config/sessions/session-history-eviction.ts@3939530661269d5feb886764f602876d24b8473e`
- `src/config/sessions/session-accessor.sqlite-archive.worker.ts@ac720312b8dfaa5f76c5cc6601d8466dece0e045`

The budget owner first measures actual physical bytes and refuses to predict a mutation while its checkpoint gate is blocked. Automatic kicks are throttled per store, serialized through the existing store-writer queue, and detached from the turn that triggered them so caller cancellation cannot silently abandon already-owned cleanup. Before any destructive work it captures the target's physical identity; every archive/reclamation phase reasserts that identity.

Historical eviction materializes the archive before deleting the generation, rechecks live admission/currentness at commit, publishes the canonical archive only after durable deletion, and remeasures physical usage before deciding whether another generation may be removed. Archive pruning is preferred before deleting more searchable history. Archived-entry deletion delegates to the existing lifecycle owner and then performs bounded page reclamation. An incomplete checkpoint returns a deferred result instead of authorizing additional deletion.

The archive worker uses fresh read-only snapshots for materialization and publication verification, bounds staged archive bytes, verifies the canonical stored archive hash before file publication, and owns only its private worker/admission bridge. Failed publication forces the worker toward native exit so cleanup is not inferred from a normal reply.

### 39.3 Direct production callers preserve the captured owner and commit boundary

New frozen identities:
- `src/acp/runtime/session-meta-entry.ts@f51d16826793af36eead9b7cfab05fb9f4d6fc64`
- `src/config/sessions/session-cold-storage-selection.ts@e6bcf6698fc9c2d000eb2df0259542d62c0faee3`
- `src/config/sessions/session-accessor.sqlite-reset.ts@16079f2c60ec85e877428ef6124388ed32501063`
- `src/config/sessions/session-accessor.sqlite-deletion-plan.ts@333769cdc0d4da037c32da3df56ae55f214a1048`
- `src/config/sessions/session-accessor.sqlite-lifecycle.ts@24da1958847b8939408046358db6213aeeb5d147`

ACP session-entry mutation schedules history-budget work only after its retained native receipt settles and the committed publication is known; an unsettled native outcome stays `outcome-unknown`. Cold-storage selection is read-only and excludes all live/admitted generations before producing candidates.

Offline session reset retains the selected read-only database claim, inventories database rows and archive artifacts, verifies path identity plus artifact inode/size/time/hash, refuses reset while any session admission is protected, and executes deletion under the established deletion/archive/write lanes. The deletion planner itself grants no authority: it preflights every historical generation against references and admissions, and the commit path rereads the exact generation fingerprint.

The lifecycle owner schedules a physical-budget kick only after a real commit. Historical generations are materialized outside the short writer lane, then rechecked and committed one at a time; each committed removal is published immediately so a later failure cannot erase an already-durable fact. The final entry deletion uses the captured execution when available, publishes identity invalidation after commit, and only then settles receipts. Disk-budget deletion explicitly reuses the store's captured physical owner and disables recursive rescheduling.

### 39.4 Focused history/budget oracles preserve data under contention, cancellation and owner change

New frozen identities:
- `src/config/sessions/session-history-eviction.memory.test.ts@00fe92fc7899d9c318aa4f989d14415681f5d417`
- `src/config/sessions/session-history-eviction.lifecycle-contention.test.ts@8d061b670dee156d99afabe61240f688cc6c6365`
- `src/config/sessions/session-history-eviction.preservation.test.ts@7e50b4e1e5b00c9f13904df47fc0904bae1cc2fd`
- `src/config/sessions/session-history-eviction.writer-fairness.test.ts@b329489fec6531c56f1d9ac5084cc4c3dcd87142`
- `src/config/sessions/session-history-eviction.protected-cancellation.test.ts@34b9bfafff1c1fb0cc9a3ac0ed485c77b83b5054`
- `src/config/sessions/session-history-budget-owner.test.ts@5de322ca5fc81336015a5d07be64294d347e830a`
- `src/config/sessions/session-history-budget-owner-parity.test.ts@afd0039e3786c2ac05d9b61f6e43e84470fc407c`
- `src/config/sessions/session-history-archive-pruning.admission.test.ts@b678bc386f0d0b17da2fcc76c8dfff88e2ce2f9d`
- `src/config/sessions/session-accessor.sqlite-archive.byte-limit.test.ts@caa8a51ddebed529e9860b3ca1fc90a75d15f3ad`
- `src/config/sessions/session-history-admission-protection.test.ts@d0ef577b88abb402d4c01870cc05a76bdfd91fec`
- `src/config/sessions/session-cold-storage.publication.test.ts@996e12884c413264e4eba9853d5440164c63ddb1`
- `src/config/sessions/disk-budget.physical-usage.test.ts@2df5699b2961ec66599145f616891f503a68738a`

These source oracles establish that incognito memory stores never enter disk cleanup; foreign state-maintenance refusal preserves all history and a later fresh pass can resume; registry/recent/pinned/manual protections preserve every generation and raw UTF-16 identity bytes; page reclamation yields between bounded units so foreground writes and unrelated cold-history reads progress; and protection appearing during planning, materialization, worker admission or archived-entry deletion cancels the destructive path while preserving transcript bytes.

They also prove owner parity across canonical/custom/shared stores, reject a foreign candidate without retargeting cleanup, keep archive materialization before durable deletion and file publication after it, stop page reclamation when the owner is revoked, serialize legacy archive unlink with peer publication, preserve a newly claimed canonical archive, enforce archive/record byte limits with staging cleanup, protect normalized/raw admission keys and every referenced generation, publish a cold-restored key exactly once only after worker settlement, and join disk-budget worker drainage including work admitted while retirement is already in progress.

The previously registered main history-eviction/admission/lifecycle tests were re-read as supporting context where needed and are not duplicated.

### 39.5 Legacy SQLite SDK compatibility is explicitly inventoried and time-bounded

New frozen identities:
- `src/plugins/compat/sqlite-runtime-records.ts@e250f5c619081616fd1d6dc0c5071be33cac951c`
- `src/plugins/compat/registry-records.ts@4cae202259682223aad5bc6f6b069599b78ee9ec`
- `src/plugins/sdk-deprecation.ts@c6250a3cd45900aa3948c07a6dab1d5dc46b3cb0`

The compatibility record labels native SQLite runtime access as deprecated SDK compatibility, owned by the SDK surface and replaced by the V2 worker store/awaited domain operations. The central compatibility registry includes that finite record. Deprecation reporting binds usage to a plugin/runtime registration frame when available and records the SQLite compatibility family with its approved final compatibility day. None of these files owns SQLite state or grants database authority.

### 39.6 Concrete legacy imports are read-only, derived-store, Doctor migration or QA boundaries rather than a live agent-writer fork

New frozen identities:
- `extensions/migrate-hermes/apply.ts@817f26764c2a664559a872a49d082bd105b25046`
- `extensions/browser/src/browser/system-chrome-cookies.ts@eca6d47a466e7c5c727659bf624d93176cd38e6e`
- `extensions/memory-core/src/memory/manager-db.ts@fd6e96a37412df2aa003c2491a9720600634d324`
- `extensions/memory-core/src/memory-forget-index-read.ts@b45cf3257014625ad619e4a48fbb7f6a49ba1a97`
- `extensions/qa-lab/src/live-transports/matrix/scenarios/scenario-runtime-state-files.ts@7e968d48f35a6045309c07b89f64ac6a91fd20b1`
- `extensions/codex/src/migration/native-task-assignments.ts@34068854aae2a68fd8770ce1117f63b728e5bc97`
- `extensions/memory-core/src/migration/doctor-memory-sidecar.ts@7d094a13d24be61df162d97648141651782f459a`

Hermes migration snapshots an existing SQLite source through a read-only native handle and preserves recovery sidecars on failure. Browser cookie import opens only the copied Chromium cookie database read-only. Memory Core's manager uses the V2 worker store for the canonical PostgreSQL/agent-backed publication path; its remaining direct native transaction helper is for the separately owned derived memory index, while canonical agent reads use the read-only agent owner. The forget-index reader uses worker-runtime/in-memory probes plus canonical read-only access and is not a writable legacy consumer.

The QA Matrix state-file helper is a QA scenario fixture, not a shipping database owner. Codex native-task assignment use is a Doctor state migration: it performs read-only inspection and an exact guarded transaction only for historical delivery settlement after rereading the retained task/binding facts. Memory Core sidecar migration is likewise a Doctor migration boundary: it discovers and backs up legacy sidecars, imports derived memory-index data into the selected per-agent database with explicit schema registration, preserves deterministic retry material on uncertainty, and closes the migration handle.

The source compatibility scanner has no path-level exemption for these extension files; its reviewed boundary is the exact deprecated declaration/reachability contract recorded in Section 38. The scanner was read only and was not executed in this round.

### 39.7 Closure result

The WAL-budget, archive/reclamation, direct lifecycle callers and concrete legacy SQLite uses exposed after Section 38 all converge on existing canonical database, lifecycle, session and worker owners. Checkpoint completion is only a maintenance fact; archive materialization is not deletion authority; a worker reply is not native settlement; and deprecated SDK compatibility does not constitute a second online agent database writer.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All tests above were read only as frozen source oracles and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 40 — archive pruning, page reclamation and retained reclamation-worker settlement closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This section follows Section 39 into the pruning/reclamation implementations that actually own bounded physical cleanup. Previously registered agent execution, WAL, history-budget and mutation-worker identities are referenced but not duplicated.

### 40.1 Archive pruning separates measurement, per-item writer custody and WAL/page settlement

New frozen identities:
- `src/config/sessions/session-history-archive-pruning.ts@f22f01b7d945753cc53fb3133038e929043d34cf`
- `src/config/sessions/session-accessor.sqlite-page-reclamation.ts@0961c128bf30f246b7d706c9a365461959ec8256`

`reclaimSqliteFreePages` treats a vacuum pass as creating a new checkpoint boundary. A pre-vacuum checkpoint cannot settle the vacuum commit; an incomplete checkpoint records the blocked fact and stops physical deletion. Canonical archive pruning performs filesystem measurement outside writer admission, then acquires the archive writer only for one selected row/file removal. A canonical row is deleted only after its derived file was removed or proven absent; a failed file removal keeps the canonical recovery row.

Page reclamation captures the requester's physical file identity before any admission can wait. A symlink/alias retarget therefore cannot redirect the operation. Doctor/offline maintenance retains its native owner, while normal runtime work captures the canonical agent execution, uses the existing worker writer for each bounded page/archive mutation, relays WAL observations as maintenance facts, and releases the execution only after the enclosing sweep settles.

### 40.2 Disk-budget measurement and generic artifact cleanup do not become database writers

New frozen identities:
- `src/config/sessions/disk-budget-runtime.ts@3e4f0d7fb694fe76b20051b2fb793a05a8e5ba0d`
- `src/config/sessions/disk-budget.ts@003728432f7a4751aaabb0dd924cb79651cf3577`

Physical usage is measured in a reader-class worker pool. Drain joins every admitted measurement before rotating the pool; a concurrent teardown joins the same rotation instead of retiring a successor worker.

The generic artifact budget protects canonical referenced transcript/trajectory paths, fresh store-temp files and in-flight prompt blobs. Orphan cleanup is cheapest-first. If cap-archived entries must be sacrificed, the reduced index is durably committed before unlinking the victim's artifacts; without that durable commit callback, the artifacts are deliberately retained as recoverable orphans. This filesystem budget layer therefore cannot stand in for the canonical SQLite lifecycle writer.

### 40.3 Reclamation commit authorization is a one-way commit edge

New frozen identity:
- `src/config/sessions/session-accessor.sqlite-reclamation-commit.ts@9aee50916acea9dd4be5ae729be1e5be799913fa`

The shared commit gate moves from WAITING to REQUESTED, then only the captured owner context may atomically accept REQUESTED to COMMITTING after rechecking authority. Revocation can reject a pending request, but cannot undo an already accepted commit. Worker-side wait times out/refuses unless COMMITTING is observed. SETTLED is published only after the transaction ended or the connection successfully closed. A refused request may reuse a connection only after native rollback is established.

### 40.4 Reclamation routing preserves native maintenance, retained claims and worker ownership as separate modes

New frozen identity:
- `src/config/sessions/session-accessor.sqlite-reclamation-run.ts@955f799e97c3ced8754eb0eb29f0c4ca7463aa5b`

Doctor/migration or explicit schema-maintenance authority stays on the native exclusive owner. Ordinary runtime reclamation does not open a new host writer: it first reuses a current retained read-only claim when one exists, otherwise captures the canonical agent execution and prepares the reclamation worker against the exact observed file identity. Maintenance metadata keeps its established executor.

Worker preparation and mutation use separate writer admissions. The request's claim, source identity and caller authority are revalidated at every boundary. After native transaction settlement, session/conversation invalidations and lifecycle effects are published while the writer is still held; result delivery is not used as proof of commit. Archive bytes transferred to the worker are owned transfer buffers, so host mutation does not continue over shared payload state.

### 40.5 Focused commit, binding, retention and admission-race oracles

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-reclamation-commit.test.ts@699bd4d614b872f91821066cfb42bab46301358f`
- `src/config/sessions/session-accessor.sqlite-page-reclamation.binding.test.ts@b8828f29cd1f25ddc2745ae4a3816d425e5b8fdd`
- `src/config/sessions/session-history-archive-retention.test.ts@1518d07fbcdd599a1776e6aebf82a13cc841b92c`
- `src/config/sessions/session-accessor.sqlite-reclamation-admission-race.test.ts@f9ca029371c1a82f9382861a205cb32b7e30ae3f`

The commit oracle proves an accepted reclamation keeps writer admission through native settlement; later revocation cannot abandon it, while authority/revocation before acceptance rolls the transaction back. The binding oracle proves symlinked requester paths retire through either path/root ownership and that retargeting after capture or while write admission is queued fails with an identity change instead of mutating the replacement.

Archive-retention oracles bound selection to 256 stable candidates, roll back a whole deletion batch on a late failure, allow foreground writes during selection and between batches, and recheck delayed candidates for changed file/generation/name/publication/reference facts before deletion. The admission-race oracle proves a reclamation already bound to an opened database is not redirected when an alias changes, and that caller authority closing before admission preserves current plus historical transcript data.

### 40.6 Retained reclamation-worker reuse keeps proof, lease and native exit custody exact

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-reclamation.test.ts@5d5a033678267a20efd19a50e9d45ebab298af3b`
- `src/config/sessions/session-accessor.sqlite-reclamation-reuse.test.ts@7d040123f406d800815829d6060c8618046dd61b`

The reclamation tests show native maintenance checks authority before a cold open can repair/reopen a target, captured aliases remain bound to the original database, one reclamation pass leaves bounded free pages for later maintenance, queued operations reuse a worker only for the same database, and diagnostic/log failures do not lend reclamation metadata to successor writers.

The reuse oracles prove transferred integrity proof is reused only while current, a competing newer parent verification cannot be overwritten by a delayed worker result, explicit database retirement forces a fresh worker/generation, revoked retained claims are refused on the next admission, queued cold reclamation is cancelled without waiting unrelated archive work, and memory-pressure/Gateway/idle retirement releases every exact worker lease before reuse. A crashed worker preserves the uncommitted victim and drops transferable validation; a mismatched retained lease is not silently deleted or retried until its exact receipt is restored.

The already registered reclamation worker/preparation/lifetime and Section 39 WAL/archive oracles were re-read where necessary and are not duplicated.

### 40.7 Closure result

Archive pruning, physical disk measurement, page reclamation, commit authorization and retained reclamation-worker reuse all remain subordinate to existing canonical database/session owners. A checkpoint is not deletion authority, archive file removal is not row commit, IPC delivery is not native settlement, a retained worker is not a cross-database writer, and retirement cannot discard a mismatched lease by assumption.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All tests above were read only as frozen source oracles and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 41 — archive publication, reclamation core, canonical validation, and lifecycle direct-consumer closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Section 40's page-reclamation/commit-gate/reuse identities remain canonical and are not duplicated.

### 41.1 Canonical archive rows remain the recovery source; derived-file export is post-commit

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-archive-store.ts@800fc86deada8588bd6bda377fa5b069b45dc60c`
- `src/config/sessions/session-archive-publication.ts@24e47fda67680f5c7178d4f828f3bc01d099ff44`

`publishSessionStateArchives` enters the existing mutation-worker lifetime and archive session, revalidates the captured source, and either uses the canonical worker owner or the explicit native maintenance owner. An empty requested set still retries previously committed pending archive rows. Worker publication binds the exact physical identity before preparation and verifies the native location again before record publication.

`publishPreparedSessionStateArchives` repeatedly prepares canonical rows, writes derived files outside the SQLite commit, and records successful exports only afterward. A failed/colliding export throws with the canonical row still pending instead of claiming rollback. Retention similarly drops a canonical row only after the derived file is already absent and the row is rechecked under a fresh writer admission; rows are processed in bounded batches so the archive writer is not held across the whole scan.

### 41.2 Physical disk accounting is read-only and cleanup remains admission-scoped

New frozen identities:
- `src/config/sessions/disk-budget-files.ts@f0c5e96d188e4a245f36e057476e411790493b94`
- `src/config/sessions/disk-budget.worker.ts@4707f7a8bb04b81a88174ee68083fd17d643c286`

The disk-budget worker only returns byte totals. The file layer measures canonical session artifacts plus durable SQLite main/WAL paths, excludes SHM/seal metadata and migration rollback artifacts from eviction accounting, canonicalizes/deduplicates paths, and performs bounded directory stats. File removal rechecks that the target is still a file and calls the current database-maintenance admission assertion immediately before unlink; it never grants database write authority.

### 41.3 Retained reclamation workers are keyed by physical database identity and uncertain mutations are never replayed

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-reclamation-worker.ts@282e66791782b28e143c65398380f8ea9ef774c6`
- `src/config/sessions/session-accessor.sqlite-reclamation.ts@8be0a591987c450bc8fc09821ef39f0e84b2beb4`
- `src/config/sessions/session-entry-deletion.worker.ts@9f54e46706976c9a378440c74ceff06fe0f8b019`

The retained worker map is keyed by the physical reader-database path. Requester aliases are registered only as cleanup selectors; they do not own the worker. Before reuse the worker must match both options and expected identity. On failure it is reusable only if the captured source and worker identity are still current. Replacement, revocation or transport uncertainty closes the worker, and the failed mutation is never replayed; a replacement may serve only a later request.

The transaction core routes every plan kind through the established agent/lifecycle transaction owners. Entry and lifecycle-artifact plans reread their prepared snapshots before deleting; historical and budget eviction recheck live protection/recency at commit. Page maintenance calls the database WAL owner with before/on/after commit hooks. Best-effort free-page reclamation after an already durable deletion is explicitly non-authoritative: failure leaves the deletion committed for a later budget pass.

The entry-deletion worker executes the same reclamation transaction through the admitted agent worker context or native-binding transaction. Its publication candidate is built from the transaction result and transferred only through the existing worker admission; it is not an independent writer.

### 41.4 Cleanup, purge, projection, creation and maintenance all compose the same owners

New frozen identities:
- `src/config/sessions/session-agent-purge.ts@0dc25706e6e825eb1c7e4f0dc657bcc9572bd514`
- `src/config/sessions/session-accessor.sqlite-artifact-cleanup.ts@4f178e77c5cf1cb39ce82c36d2d1f1c7210ec4c7`
- `src/config/sessions/session-accessor.sqlite-projection.ts@b062842cf60a679400157366a0cebf43a2333dc1`
- `src/config/sessions/session-accessor.sqlite-maintenance.ts@b38fb99b63c96b4b4e4cfbf0b76b35573877389f`
- `src/config/sessions/session-accessor.sqlite-creation.ts@7de68f8215b37294a87f4cb30a954056adb21f58`
- `src/config/sessions/session-accessor.sqlite-maintenance-kick.ts@7df8c90a8fe20bd324120aec410e64c92c6669ce`
- `src/config/sessions/session-accessor.sqlite-replacement-projection.ts@244cf8bb61598443f18042d770dfe5a4a433c64f`

Agent purge captures the deletion execution, prepares exact removals, materializes archives before commit, commits through the existing deletion/native-binding owner, finalizes maintenance only after the writer is released, and publishes derived archives afterward.

Lifecycle artifact cleanup captures either the incognito actor owner or the exact SQLite file identity. An absent captured store cannot later be silently adopted if startup creates it. SQLite planning runs through the history worker, deletion uses the existing reclamation owner, planner statistics refresh happens after the SQL commit, and archive publication is a separate post-commit boundary.

Lifecycle projection and entry replacement both project first, materialize deletion archives outside the short writer section, capture maintenance preservation, then commit through the existing projection/reclamation/native owner. Post-commit archive publication and retention cleanup can fail independently and may be reported/captured, but cannot erase committed row changes.

Creation retains one source owner across preparation, transcript initialization, entry commit and any pending archive recovery. A worker outcome-unknown is propagated rather than converted to a retry. Maintenance groups shared/historical ownership into indivisible batches, gives writers turns between bounded sizing/finalization work, counts only committed entry removals, and treats archive-publication/planner-statistics failures as later cleanup work rather than transaction rollback.

### 41.5 Canonical-validation readiness certifies rows without becoming a data writer

New frozen identity:
- `src/config/sessions/session-canonical-validation-readiness.ts@247fc8b2ba556ebd4df5c3d6f5f7e1c21e63f2b9`

Readiness retains the existing read-only database claim and current integrity receipt, then certifies pending canonical rows through the reclamation worker in bounded 128-row / 1 MiB batches. Each batch re-enters the existing writer admission and rechecks owner, mutation lifetime, claim and worker identity. Canonical readiness is marked only after the final batch and only while the same physical database remains current. Runtime callers may share one active drain by database incarnation, but success or failure is not cached as a replacement owner.

### 41.6 Direct production consumers pass expected state or live authority back to the lifecycle owner

New frozen identities:
- `src/gateway/boot.ts@43fb88903468039b3f4ebb5b65580c4290efcff0`
- `src/cron/session-reaper.ts@b1d615cf5b6ab55b3f58775aa4251e36eeb7c53a`
- `src/agents/internal-session-effects.ts@39b67f292ce6d3de40d151754f2284ca58b6c211`
- `src/cron/isolated-agent/run-prepare-runtime.ts@2e86cf678a59e642ae1bab0a8576a7002e18eb23`
- `src/config/sessions/session-registry-maintenance.ts@884cc20b6317345891e9f840ef3a69f410f6842d`
- `src/commands/doctor-session-canonical-keys.ts@6b1afad456b3b633c358d19c56beec5a6103f4b1`
- `src/commands/doctor-heartbeat-main-session-repair.ts@645e2c5ab1bdf4882a473b80aeed14d518aa3cf7`
- `src/config/sessions/session-accessor.reset.ts@8ccd9abd1322723534e3b434bea17804d10cba99`
- `src/config/sessions/session-accessor.entry-mutation.ts@2bea4b74ddb1d498d31a89aed868c2ea1bd8f00c`

Gateway boot removes only the session id it created. Cron reaping supplies the exact entry/session plus a continuation-aware commit guard and treats archive retention as separate history policy. Hidden internal-session cleanup supplies its expected session/entry and disables unrelated maintenance. Isolated cron preparation passes the live run guard into lifecycle mutation. Registry maintenance verifies its captured ancestry revision again at commit.

Doctor canonical repair is the explicit exceptional path: `allowCanonicalRepair` plus in-transaction delivery/state rehoming is required, while stale loser stores are removed through exact lifecycle removals and verified archives. Heartbeat repair similarly moves an exact expected main entry. Reset composes its creation binding into the commit guard, and public create-entry mutation enters `createSessionEntryWithTranscriptInScope` through the already prepared lifecycle scope. None of these consumers opens a parallel reclamation writer.

### 41.7 Focused negative oracles close publication, generation, receipt and readiness races

New frozen identities:
- `src/config/sessions/session-canonical-validation-readiness.test.ts@2d2bffbf0dba4db1708c0f0179b4a3c37bb7a274`
- `src/commands/doctor-maintenance.session-workers.test.ts@f11ed762566e2159f943c83fd4b10eb22e783fc6`
- `src/config/sessions/startup-migration.registry-recovery.test.ts@d18732aef16788998da09eaea13801b15c14d8b2`
- `src/config/sessions/session-accessor.sqlite-cleanup-race.test.ts@4a8414f5f0d023e91d752c8e7150733095257508`
- `src/config/sessions/cleanup-service.fix-missing.test.ts@b1fa957072538826b38b9eb04f8fc2af6b8919dc`
- `src/config/sessions/session-accessor.sqlite-deletion-receipts.test.ts@45d24705c0e06a8e9a03d445a71e88ea33633403`
- `src/agents/sessions/agent-session.reclamation-admission.test.ts@14d481e443da13dc89fddc6ca60c0c1e694e03d8`

Canonical-validation tests prove schema drift, replacement, revoked receipts, row changes and superseded startup authority fail closed; a large backlog retains one worker while admitting foreground writes between batches; copied stores require a fresh full certification; shared active drains retain neither success nor failure after completion. Doctor tests show certification and archive/cold-history work drain without retaining worker leases between repairs. Startup registry recovery refuses to create missing configured stores and preserves durable lineage/held deletion state.

Cleanup-race tests prove entry/generation facts are revalidated after async materialization; foreign plugin/agent owners survive; writers progress while archive bytes are produced; maintenance continues only for unchanged batch members; already committed earlier history remains published when a later generation changes; an empty cleanup pass republishes a pending canonical archive; and physical replacement after classification is rejected.

Fix-missing tests prove pending canonical archive rows reconstruct every missing derived file after commit, collision bytes are never overwritten, lifecycle-delete failure rolls back the canonical archive, foreign appends survive classification-to-commit, rewritten generations are reclassified, and retention deletes a canonical row only after its derived file is gone.

Deletion-receipt tests prove rollback retains write scopes/receipts; successful deletion retires only the exact session scopes; stale captured admission state cannot revive retired scopes; alias retarget settles receipts against the original database; batch failure retires only already committed rows; aborted deletion/context reset preserve receipts. Agent-session reclamation admission proves unrelated transcript publication remains queued until reclamation writer admission settles.

The previously registered startup-migration, gateway canonical-validation, native-binding, archive-session, reclamation worker lifetime and Section 40 tests were re-read where necessary and are not duplicated.

### 41.8 Closure result

The newly exposed archive store, reclamation core/worker, disk accounting, lifecycle composition and canonical-validation paths do not introduce a second database writer, a second physical-close owner, archive-file-as-commit semantics, or receipt retirement by inference. Canonical row commit, native transaction settlement, derived-file publication, receipt retirement, worker exit and physical close remain separately evidenced boundaries.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All test files above were read only at the frozen ref and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 42 — recursive archive/reclamation direct-consumer, maintenance-admission and publication-order closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This section continues Section 41 by closing module-path callers and focused tests that are not discoverable from exported-function searches alone. All identities were exact-ref fetched at the frozen source and de-duplicated against the live Round87 dossier before this append.

A moving-main search also exposed `src/commands/doctor-relative-session-store-preservation.ts`, but exact-ref fetch at the frozen source returned 404. It is therefore excluded from this frozen-source evidence rather than being silently imported from moving main.

### 42.1 Disk-budget deletion remains filesystem cleanup under maintenance admission

New frozen identities:
- `src/config/sessions/disk-budget.removal-failures.test.ts@cca23248f471b8d4033e960bd4285fb0aaedfa05`
- `src/config/sessions/disk-budget.test.ts@77401a8e5a5c6549ba175d6334b33baa622b02ed`

The removal-failure oracles show that a revoked maintenance owner prevents unlink, failed transcript/prompt/archive removal does not claim freed bytes, and cleanup continues only over independently removable victims. The main disk-budget oracles preserve referenced primary transcripts, migration recovery archives and fresh/in-flight prompt artifacts; they prove a cap-evicted artifact is retained if the reduced index cannot be durably committed first, and that a file concurrently removed before `rm` is not counted as a successful removal. These tests reinforce Section 41: filesystem accounting/removal is not database mutation authority and cannot manufacture deletion settlement.

### 42.2 Maintenance/reclamation admission retains writer custody through native settlement

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-maintenance-admission.test.ts@303a54062aafcd0c1bd5168a8056c82531bed573`
- `src/config/sessions/session-accessor.sqlite-prepared-admission.test-support.ts@86d91c45311c68990fd251380a056d014028b1f6`
- `src/config/sessions/session-accessor.sqlite-reclamation-maintenance.test.ts@f6c73968c745d899c2d0feb19097e523acdef824`
- `src/config/sessions/session-accessor.sqlite-reclamation-reuse.test-support.ts@8d1f546c82496bcd40ddf2a22fc4bcf8bda14c2d`
- `src/config/sessions/session-accessor.sqlite-maintenance-preparation.test-support.ts@373bc7ce7504028f0ff88c78fb35e7377e6f3af9`
- `src/config/sessions/session-accessor.sqlite-maintenance-worker.test.ts@8ff65e5c6ed5b02fd418fdfcb9fbdd036e7160c0`
- `src/config/sessions/session-accessor.sqlite-maintenance.test-support.ts@d3eda634daae3e35cc706c5baebd942870f5e137`
- `src/config/sessions/session-accessor.sqlite-maintenance-read.test.ts@3da7b2d25dbf64192c76709d33c782e541e9f97d`
- `src/config/sessions/session-accessor.sqlite-maintenance-cadence.test.ts@2621929d9d5aabf0a48c8cd211849cc81ba9da7a`

Maintenance preparation deliberately yields between planning/validation boundaries so foreground writers can progress, but the final commit admission remains held through publication and native settlement. The native maintenance failure matrix proves unsafe rollback/close outcomes keep the database locked, keep the exact worker lease retained and block a following writer until Worker exit; only then is the retained lease released. Prepared-admission support separately verifies integrity work occurs in child/Worker execution rather than caller-thread data SQL, joins those native workers at close, and observes a retained finalizer only against the same current open database.

The Worker maintenance oracles additionally show that post-COMMIT parent predicate retirement does not erase a committed mutation, retained worker reuse stays on the same native worker only while its source remains valid, only committed removal indices are published, and committed statistics survive a later caller predicate failure. Cadence/read oracles keep bounded batches and fresh protection facts authoritative across concurrent writes; stale prepared work is rejected instead of being forced through.

### 42.3 Publication ordering is post-commit and shared across transformed SDK module identity

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-lifecycle-publication.test.ts@bcbc49134c970934a6f60128d7dff87497bd909e`
- `src/state/openclaw-agent-db-module-identity-exports.test-support.ts@7b32e2c6ed27662b9ae854dae24325b679b6f437`
- `src/state/openclaw-agent-db-module-identity.test.ts@f87a90a9137d14c4303f78304731c20feccf2158`

Lifecycle-publication tests distinguish transaction commit, post-commit publication and writer-return settlement. A rollback publishes nothing; a publication failure leaves the committed lifecycle present; and a later writer-return failure occurs only after commit/publication have already settled. Confirmed removals retire prepared sharing/generation/cache facts before identity and row observers can recreate the key.

The module-identity oracle proves transformed plugin SDK imports resolve the same canonical agent database object, the same archive/reclamation FIFO singletons, the same post-commit publication observers and the same worker-store runtime. Host and plugin calls serialize through one queue, plugin borrowing does not physically reopen or repeat integrity validation, explicit owner disposal revokes retained borrowers, and native/transformed publication clients reuse the same native Worker. This is direct negative evidence against a second writer/queue being created by SDK module duplication.

### 42.4 Archive/reclamation transaction helpers still delegate to existing canonical writers

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-archive-transaction.ts@9852b3f7700bb07ec9fdbcb38efc81e0e2061664`
- `src/config/sessions/session-accessor.sqlite-lifecycle-state.ts@b158417ef0a9202d29bd8340060a50fd07c4335d`
- `src/config/sessions/session-accessor.sqlite-replacement-state.ts@acafd2f86f9cd4027fd6bcbf01eeef8a17a48fa8`
- `src/config/sessions/session-entry-read.worker.ts@3e4bef825d05c7f39339641ab2618e5b0a992a73`
- `src/config/sessions/session-accessor.sqlite-maintenance-transaction.ts@ebd8006f34743bef9868db7f89dc00dfb8c25cfd`
- `src/config/sessions/session-accessor.sqlite-maintenance-worker.ts@7368a047111417909ac975eda8c11466434e7ecd`

Archive publication transaction code calls the established `runOpenClawAgentWriteTransaction`; maintenance transaction code uses the established agent write/deletion transaction owners for planning, metadata and finalization. The maintenance Worker refuses a missing/non-captured file database and performs metadata work through the existing session-entry worker mutation path. Lifecycle/replacement state modules produce/archive/project state inside those transactions, while `session-entry-read.worker.ts` remains a read/snapshot worker with physical-identity checks and post-commit publication anchoring. None of these modules introduces an independent host-side writable connection or physical-close authority.

### 42.5 Gateway, sharing and projection consumers preserve commit/lease boundaries

New frozen identities:
- `src/gateway/server.sessions.delete-state-lifecycle.test.ts@23d7e16ab48820eb4d694715ee12d6d3efb98187`
- `src/config/sessions/session-sharing-store.test.ts@665628e7863614b463f797af79f35884a50a8160`
- `src/gateway/session-row-projection.archived.test.ts@e98174fd9f24a76d7de4ec4329c7cb5618d2da56`

Gateway deletion tests recheck exact entry generation, authorization and placement after asynchronous transcript materialization; post-commit placement retirement may happen during archive publication/worker queue without reclassifying the already committed deletion. Sharing-store tests join exited maintenance leases before fixture/store teardown and keep membership publication after the containing transaction. Archived row projection tests prove a committed removal can publish before a reentrant listener recreates the same key; the replacement survives and is what subsequent projection reads observe.

### 42.6 Writer-owner attribution remains bounded to the real commit owner

New frozen identity:
- `src/config/sessions/session-accessor.sqlite-writer-owners.test.ts@aa6a3d6a374b88ba36dd0da83c3de4a54e4ff37f`

The writer-owner oracles attribute lifecycle mutations to their actual inline/split commits, record archive-pruning writer stages separately, and coalesce automatic maintenance through native planning/finalization. Worker outcomes keep their native Worker identity and do not turn archive pruning or maintenance instrumentation into a second data owner.

### 42.7 Recursive direct-consumer closure

Module-path searches for `session-accessor.sqlite-archive-store`, `disk-budget-files`, `session-accessor.sqlite-reclamation-worker` and `session-accessor.sqlite-reclamation.js` were reconciled against the live dossier. Previously registered runtime callers/tests were not duplicated. The newly listed identities above close the remaining frozen-ref production/support/test consumers exposed by those module-level searches. Search-only configuration baselines and inventory scripts are not treated as shipping database owners.

The resulting ownership boundary remains unchanged: canonical row mutation belongs to the existing agent/lifecycle writers; worker admission is retained until native settlement; archive publication and filesystem deletion are later/derived effects; retained workers are keyed and retired by exact database identity/lease; read workers and transformed SDK imports reuse existing ownership rather than creating parallel writers.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All tests above were read only at the frozen ref and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 43 — commit-receipt, retained-publication and native-binding settlement closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Concurrent Section 42 already registered archive-transaction, lifecycle-state and maintenance-transaction helpers; those identities are referenced here only as context and are not duplicated.

### 43.1 Canonical archive rows and reclamation publication are exact committed facts

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-archive-store-kernel.ts@56ca662cf55dfcec2169f054f796813358561ae2`
- `src/config/sessions/session-accessor.sqlite-reclamation-publication.ts@a940e269d3f11d9778ec57e7ee39d00ede1d03b2`
- `src/config/sessions/session-accessor.sqlite-projection-state.ts@3a97da585d9510b9a35efa36741a274f4c228590`
- `src/config/sessions/session-lifecycle-worker-publication.ts@9b91a17eca3b999a319613ead3290ec7dc696500`

The archive kernel stores the canonical archive row inside the owning lifecycle transaction. A duplicate `session_id + generation` is accepted only when archive name, hash, created-at, encoding, reason, session owner and every archive byte match the committed row; otherwise the transaction fails with a conflicting-archive error. Publication planning is bounded to four pending canonical rows and may normalize only unpublished names under that session's write scope. Recording file-publication results only updates the exact matching session/generation row under caller-owned write admission.

Reclamation publication selects only rows the transaction actually removed. Maintenance publication uses `committedEntryIndices`; failed history eviction publishes no changed keys. Identity mutation is prepared only for committed lifecycle/projection/finalization results, never from the existence of a plan or Worker reply.

The lifecycle projection transaction rereads every exact removal/upsert row inside the admitted transaction. A changed row throws stale-lifecycle conflict; materialization failure excludes archive-requiring removals instead of deleting them; reset boundaries and owner assignment are applied only after the exact preimage is validated. The worker lifecycle publication emits progress-card reset and transcript-index reconciliation only for a committed lifecycle-projection result.

### 43.2 Commit receipts describe exact postimages; they are not replay authority

New frozen identities:
- `src/infra/sqlite-commit-receipt.ts@11d3905fc2bde1288d12180c58cdf27250d832d1`
- `src/infra/sqlite-post-commit.ts@a203acefc5d28361a24a620fd132c8f264e8f4e9`

A SQLite commit receipt covers an explicit domain, source identity/incarnation and exact key set. An omitted key is unknown, never proof of absence. The source comments explicitly state that an operation identity is not a durable idempotency key or permission to replay. Worker receipts are validated once for exact coverage; domain-specific code remains responsible for currentness and supersession.

Post-commit state is staged transaction-locally. Immediately before physical COMMIT, transferable readers can be fenced; after COMMIT, committed facts install before fallible public observers. Failure installing committed facts invalidates/fences those facts but does not transform the durable transaction into rollback. Notification failures are logged and remaining committed notifications continue. Rollback discards nested staged state, while an unknown/lost transaction invalidates all staged facts rather than publishing them.

### 43.3 Entry, conversation and transcript publication retain final-grant custody until native settlement

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-entry-worker-publication.ts@1e8ff2d82954b82b68c91a4f7df971a84e87ab78`
- `src/config/sessions/session-accessor.sqlite-conversation-publication.ts@ad85b9c549294b5ae4b73f8c3031356372abd9c9`
- `src/config/sessions/session-accessor.sqlite-entry-cache-publication-state.ts@a13dcafeaf70a301afb18201604fb7e84f46ddbc`
- `src/config/sessions/session-accessor.sqlite-entry-cache-publication.ts@4d6a01611a75e196baf6531a0b6817339b866566`
- `src/config/sessions/session-accessor.sqlite-sharing-acquisition.ts@0adb3ce8e43d5ebaced7c4bf863d9db35c3cf2e8`
- `src/config/sessions/session-transcript-authority.ts@f3f3947a0fc9561f26bd0f321343534c0493eab4`

Entry publication's final-grant owner fences old facts independently of result delivery. Confirmed removals publish absence; incomplete postimages are invalidated. Superseded keys cannot be revived by an older delayed receipt. A COMMIT receipt may survive unknown settlement, but unknown settlement fences retained facts and deliberately does not acknowledge an identity mutation.

Conversation publication uses the same commit-receipt/post-commit phases. The existing commit grant owns the unavailable interval through native settlement; a native SDK write may commit before a queued Worker result arrives. Outcome is `published` only when the exact receipt is installed and settlement is completed; otherwise retained conversation facts become unavailable/unknown.

Entry-cache and sharing-acquisition state bind retained predicates/generations to the physical database identity and original session generation. Newer committed facts supersede delayed older publications. Unknown postimages require a fresh read, and mismatched generation/membership facts invalidate the retained acquisition instead of adopting another lifecycle.

Transcript authority is captured in the existing mutation before COMMIT/savepoint release, produces exact commit receipts, and is published by the native settlement owner while it still holds the writer FIFO. Retained transcript publication checks source identity and supersession; outcome-unknown or a receipt mismatch installs unknown facts rather than pretending a committed transcript postimage.

### 43.4 Native-binding, deletion and maintenance adapters keep one settlement owner

New frozen identities:
- `src/config/sessions/session-entry-deletion.ts@6977a11ded189cd9bbf00524934d86555ae063b7`
- `src/config/sessions/session-maintenance-finalization.ts@75fdd2b8379bbd4a09aea4d0216913b19bca5fd1`
- `src/config/sessions/session-native-binding.ts@0c0cdd1384232ca54fa1e89c43650506b856e3f3`
- `src/config/sessions/session-agent-purge.kernel.ts@aa88b8642535c3ecc27d337430b3dbfe44297028`
- `src/config/sessions/session-maintenance-finalization.worker.ts@eaa8261973d8619a77ee14635bdd503ac32bc5e3`

Entry deletion and maintenance finalization reuse the caller's retained execution and choose the native-binding operation only when exact deletion participants were captured. Maintenance sizing/statistics also stay on that original executor. Finalization publishes only the committed maintenance result.

Native-binding settlement rechecks caller authority, captured deletion authority, shared-state admission and every participant before transaction entry. It waits the retained transaction settlement and the admission settlement, then accepts only a typed closed receipt proving committed or rolled-back phases. If receipt/settlement is absent, transport outcome is uncertain, or plugin-state publication fails, every participant is settled `unknown`, the publication is marked unknown and that native generation is forbidden from reuse. An explicit committed/rolled-back receipt is required for those stronger outcomes.

The purge kernel recomputes the currently owned session-key set and requires it to equal the prepared set before deleting. It rechecks every planned lifecycle artifact preimage, deletes only materialized plans that still match, then applies maintenance. Historical windows not owned by the retired entry remain under retention ownership.

The maintenance-finalization Worker runs the same database finalizer and transfers a candidate through the existing agent admission; native bindings use the established native-binding transaction, otherwise the existing agent `writeTransaction`.

### 43.5 SessionManager and actor transports install only exact native receipts

New frozen identities:
- `src/agents/sessions/session-manager-publication.ts@309970072817c5d5a8160dbf888f7749a8a6d75f`
- `src/config/sessions/session-actor-sqlite.ts@18682b93ee88e4b18c5cae3b1aecf6c0bad1d7f2`

SessionManager parses transcript, entry and conversation publication receipts at commit admission and requires every receipt source identity to equal the admitted database identity. It retains all publication domains under one native receipt boundary. Committed facts must match the admitted postimage exactly before identity publication; otherwise publication is refused. If retained settlement completes without a recognized commit receipt, only a confirmed non-protocol rollback can be classified rolled-back; all other cases settle the retained facts as unknown.

The SQLite session actor similarly keeps admission and settlement with its durable transport. Transaction/commit grants require target-specific admission evidence and live authority. A committed actor result is accepted only after retained native settlement plus the exact transferred receipt matching command id, phase id and phase. Transport timeout or reply loss is not native settlement. Unknown outcomes invalidate the replica and require reconciliation before another command; committed outcomes may run retained follow-up work only after the commit has already been installed.

### 43.6 Focused oracles distinguish commit, rollback, unknown and supersession

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-maintenance-writer.test.ts@1d73fd1a9c779d959b3d37b4f32edb34d2fafbc9`
- `src/config/sessions/session-accessor.sqlite-archive.worker.test.ts@c4bf1c580dce8721a903d161e885f0e09c829bf2`
- `src/config/sessions/session-accessor.sqlite-replacement-publication.test.ts@f0bd876a0eaf6cabc680d769ed87f35f53d662c0`
- `src/config/sessions/session-accessor.sqlite-replacement-receipts.test.ts@67e44bc974192f1e8fba9340876931d818a30df0`
- `src/config/sessions/session-actor.test.ts@1bd4a01192153c103d085afa07d3e3f2ca99ad3c`

Maintenance-writer oracles show finalization returns only exact committed indices, preserves changed/conflicting entries, protects active session-key/session-id identities, does not block foreground ingress behind automatic finalization, and preserves committed planner statistics through later predicate/authorization failures.

Archive-worker oracles prove materialization never creates the derived archive file itself, preserves distinct generations for a restored physical session id, normalizes oversized pending names after deletion commit, routes a logical agent through the exact physical owner, rejects deduped plans with conflicting snapshots, refuses transcript changes between planning/materialization/commit, and leaves lifecycle state untouched when archive materialization cannot be safely published.

Replacement-publication tests explicitly exercise commit, rollback and unknown boundaries, lost/overtaken result delivery, callback/release failure and superseding writes. During a committed-but-not-yet-installed interval retained conversation authority is unavailable; withholding the real COMMIT receipt never invents a postimage. Unknown publication invalidates retained facts without emitting identity mutation.

Replacement-receipt tests verify exact receipt coverage in autocommit and transaction modes, prevent later native writes from certifying older metadata, keep unrelated predicates current, preserve newer successor facts over delayed older receipts, fence all retained readers after native publication installation failure, bind removal receipts to their original physical store across locator rebinding, and preserve durable native commit even when public fact/projection installation fails before notification.

Actor tests recover a transferred native commit after ordinary reply loss without replay, refuse incomplete outcome transfer, preserve FIFO around rolled-back mutations, install append receipts before retained follow-up work, preserve committed state through worker-publication/reply/observer failures, require reconciliation after unknown settlement, wait actual native settlement on release, and refuse new admission while already accepted work finishes its committed reducers.

Previously registered Section 42 transaction helpers, lifecycle-projection Worker, worker-operation settlement and conversation-route ownership tests were re-read only as context and are not duplicated.

### 43.7 Closure result

The transaction/publication foundation now closes as one chain: exact transaction preimage -> physical COMMIT/rollback -> typed commit receipt -> retained settlement -> fact installation -> public observers. None of the newly exposed layers treats operation identity, Worker reply, archive file, pending publication, cache state or transport success as substitute commit proof. Unknown outcomes fence state and force fresh read/reconciliation; they do not replay the mutation or create a second writer.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All test files above were read only at the frozen ref and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 44 — remaining direct publication consumers and writer-owner diagnostics closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Section 43 already registered the archive kernel, maintenance finalization/native-binding adapter, reclamation publication and SessionManager publication core; those identities are context only and are not duplicated below.

### 44.1 Post-COMMIT identity publication is deferred from savepoint success and consumed only after durable commit

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-identity.ts@edcc43bf5ac9e0a1e60210a80ee374ef84696704`
- `src/config/sessions/session-turn-publication.ts@b25538bcda1f2814d5aec0db907793d5a59dc9f2`
- `src/config/sessions/session-actor-memory-publication.ts@33f9854fc9450380f0c3b8bf3c53fdbc1ff5e795`
- `src/config/sessions/session-lifecycle-projection.ts@bca06182a2cdbb47b7c778e0bbb51dc91db4da2b`
- `src/config/sessions/session-turn.ts@4e062e16b59eb1de19001f5afccfc2bded52e41f`
- `src/agents/sessions/session-manager-branching.ts@c377c04a50378bc19a312c6363dc34145fd52b5b`
- `src/agents/sessions/session-manager-persistence.ts@b4b0049d64bf3fc6989df2c709565d3ec382bf05`

The identity owner stages its observer work through `deferSqlitePostCommitPublication(sqlPublicationAnchor(...))`; its source comment explicitly rejects savepoint success as COMMIT. The fallback direct publish path is used only when there is no transaction-local post-COMMIT publication to defer. Lifecycle removals are prepared from exact previous/current facts and the publication binds the already prepared entry-cache facts before emitting identity mutation.

Session-turn publication accepts canonical custody and installs committed transcript message sequences before fallible source/identity observers. Completion callback failure is retained as a committed completion rather than used to undo durable state. Branching likewise treats a committed branch whose later adoption/identity publication fails as a committed error carrying the committed session/version and explicitly forbids replay of that branch.

SessionManager persistence records the committed initial-writer fence and committed header before identity observers can cancel or supersede the run. Identity publication then runs against the admitted database incarnation/physical identity; a failure after that commit invalidates the local view through committed-error handling instead of manufacturing rollback. Actor-memory publication installs the complete committed state/change facts before emitting the identity mutation.

Lifecycle projection keeps the worker commit as the mutation boundary: the acknowledged lifecycle callback is invoked from the committed candidate path, transcript reconcile is scheduled from that committed projection, identity changes publish from the retained worker publication, and `afterCommitted` runs only after those commit/publication steps.

### 44.2 Gateway, patch and actor adapters consume the same committed callback without creating a second writer

New frozen identities:
- `src/gateway/session-create-service.ts@993f5c8eaef6aa89ce721ae27990ec2d40cefd3c`
- `src/gateway/server-methods/sessions-patch-engine.ts@9621c39ca3dc3ccbd15d87c43377460733b59c3e`
- `src/config/sessions/session-actor-entry-adapter.ts@74435cd978d3169ca30dfea0a51d3b60c2f41b78`
- `src/config/sessions/session-accessor.memory-cleanup.ts@22b57206606a8b5be8c35184dccfe368d8d5a0c2`

Gateway creation marks both lifecycle preparation and target creation committed only inside `onLifecycleCommitted`; its finalizer rolls back prepared gateway lifecycle state only when that callback never established commit. Session patch batching records newly created sessions from the group store's committed callback and runs category/effect follow-up through the later `afterCommitted` hook, preserving commit versus side-effect ordering.

The actor entry adapter invokes its lifecycle callback only from the actor's committed outcome and runs the optional post-commit follow-up while holding the same actor storage/currentness authority. Memory cleanup likewise emits the lifecycle callback only for a `committed` actor-storage result, then performs `afterCommitted` with the same authoritative memory owner. These are consumers of the existing commit owner; none opens a parallel SQLite writer.

### 44.3 Writer-owner diagnostics observe settlement; they never become mutation authority

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-reclamation-worker-diagnostics.ts@59ea393e29878f735f12a438a27478202c2c0eec`
- `src/config/sessions/session-history-archive-pruning-diagnostics.ts@8d893c346927ad0b9efd85fa099f055c6e18bc84`

Reclamation diagnostics classify a commit-guard supersession as a caller-owned retry rather than Worker failure, log rejected or >=1s operations, hash the session identity, truncate error/error-frame text, and preserve Worker/main-thread identity only as diagnostics. A fast superseded result is debug-level; slow or real rejection is warning-level. This module does not grant admission, retry a mutation or settle the writer.

Archive-pruning diagnostics wrap the real pruning operation in `try/finally`. They record query/row-deletion/file-removal/measurement/legacy-inventory timing and emit a slow/failure warning only after observing the underlying result. Logger failure is swallowed explicitly so diagnostics cannot replace either the pruning result or its original failure. Instrumentation therefore remains an observer of the real archive/reclamation writers.

### 44.4 Focused writer/admission oracles preserve real owner attribution and off-main-thread reclamation

New frozen identities:
- `src/state/openclaw-agent-write-admission.test.ts@1a1dc7ed58b6354b45479d99ef17ef0bdfde238d`
- `src/config/sessions/session-accessor.sqlite-scope.test.ts@1fbd30c78f08def8561d9cb2ba26948d0cda8d89`
- `src/gateway/server.sessions.reclamation.test.ts@41065c29d3494fd2c3b16bffbce7e5fa342a774b`

Write-admission tests distinguish queue wait from writer execution, attribute reentrant Worker reservations without lending them to foreground callbacks, retain an aborted Worker reservation until actual settlement, avoid reporting a cancelled waiter as holder, and prove diagnostic logging failure cannot replace commit/rollback settlement. The scope diagnostics tests preserve redacted error details, whitelist archive-pruning stage observations, and retain numeric queue/writer timing without leaking payload or raw identity.

The gateway reclamation oracle proves `sessions.delete` reclaims a large session through the reclamation Worker path rather than performing that work on the Gateway thread. This is focused evidence that the diagnostic/worker labels correspond to the existing execution owner and are not a second data owner.

### 44.5 Focused publication oracles distinguish rollback, committed postimage, supersession and lost reply

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-identity-publication.test.ts@22ac9256d80cc77dd96727756aa1fd1fa5fb73ea`
- `src/config/sessions/session-lifecycle-projection.worker.test.ts@eee79401c863055492a42d853c8e748e95000a69`
- `src/config/sessions/session-accessor.sqlite-fresh-upsert-archive.test.ts@8b27d3a2086c754f919e7c05c598fa264143cd42`
- `src/config/sessions/session-accessor.sqlite-creation-worker.test.ts@a13ba394eabc413c3944b51c438c1ad478d98a46`
- `src/config/sessions/session-accessor.sqlite-replacement-worker.test.ts@614fe5392a39bbbc4c4c2c4585ed95470d00d738`

Identity-publication tests fence delivery generation during native writes and restore it only for rollback, classify prepared lifecycle publications without treating copied raw events as authoritative identity changes, and preserve a newer native replacement when an earlier Worker identity listener arrives later.

Lifecycle-projection Worker tests preserve snapshot authority across metadata publication, keep guarded committed follow-up work off the host, reject stale prepared upserts without replacing the concurrent row, publish an acknowledged lifecycle exactly once even when the Worker reply is lost, and report the committed count while a following lifecycle operation observes later writes.

Fresh-upsert/archive tests prove an unrelated failed archive export does not block creation of a fresh session; the pending archive remains recoverable and is retried only after a later adoption commit. Creation-worker tests keep ownership atomic through admitted aliases, settle native finalizers after commit, preserve committed creation/header facts when cleanup fails, and distinguish outcome-unknown from rollback. Replacement-worker tests keep ordinary replacement on the Worker owner rather than a caller-thread write transaction, publish committed sharing/reader invalidation before observers, join post-COMMIT follow-up before successor authority, and preserve unknown-outcome fencing.

### 44.6 Direct-consumer closure and exclusions

Exact-symbol searches for the archive-store kernel exports, maintenance transaction/finalization exports, post-COMMIT identity publication helpers, `onLifecycleCommitted`, reclamation Worker diagnostics and archive-pruning diagnostics were reconciled against the live Round87 dossier. Section 43 identities are not duplicated. `src/commands/doctor-relative-session-store-preservation.ts` remains moving-main-only and was absent at the frozen ref. `scripts/database-worker-inventory.mjs` is inventory/provenance support rather than a shipping database owner.

The resulting boundary is unchanged: durable mutation belongs to the existing transaction/Worker/native owner; COMMIT receipts and retained settlement gate fact installation; public callbacks and diagnostics are later consumers. Publication failure or unknown native settlement fences the derived facts/generation and requires fresh read/reconciliation. It does not authorize mutation replay, convert instrumentation into a writer, or create a second storage truth.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All test files above were read only at the frozen ref and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 45 — committed completion, observer subscriptions, creation publication and committed-error closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). Sections 43–44 already registered the transaction/receipt owners and the first post-COMMIT consumers. This section closes the newly exposed direct import/subscription frontier without recursively treating every downstream business action as a new writer.

### 45.1 Transcript committed completion is an acknowledged-work join, not replay authority

New frozen identities:
- `src/config/sessions/session-transcript-commit-completion.ts@6366ab0021f3f856d58912991c5a8ed1a9008691`
- `src/config/sessions/session-accessor.transcript-turn.ts@042cf4adc51536643fce80e1ae4f37af25339cf2`
- `src/gateway/server-restart-sentinel.test-support.ts@8299aa1f179f9601820ab3ef1205646ad4a9a099`

`completeSessionTranscriptCommit` calls every synchronous committed-message callback while admission is open, captures only the explicit completion promises registered during that callback frame, closes completion admission before yielding, then joins every accepted completion. A synchronous callback failure is preserved after already accepted completion promises settle; an asynchronous completion rejection is surfaced after all accepted completions settle. Late completion registration is rejected. The helper therefore joins follow-up work for an already committed append; it is not a mutation retry path.

The non-SQLite transcript-turn adapter invokes the same helper only after its append result exists and awaits the returned completion before advancing its outer flow. Its multi-message path remembers committed transcript sequences only after the append loop has completed. The frozen direct-import closure also includes the previously registered SQLite transcript-turn owner and Section 44 session-turn publication; those are not duplicated here. Restart-sentinel test support is the only additional frozen direct importer and exercises the same committed-completion boundary in a controlled test harness.

### 45.2 Lifecycle and row-change observers have explicit committed-facts -> projection -> public-notification phases

New frozen identities:
- `src/sessions/session-lifecycle-events.ts@4edec62a6dd698edbda47b6e2f45965efdafa814`
- `src/sessions/session-row-changes.ts@ad63721c00c626b4792b3ea5793cb348ba9fc5a4`
- `src/gateway/session-row-projection-publication.ts@f812c4dfe9f7f79c3cf169fe7342fbd1c507d625`
- `src/gateway/server-runtime-subscriptions.ts@c8113e1c9ab231277dd4826069d59370cbeba09f`

Lifecycle/identity listeners live in close-and-restart global sets. The lifecycle event surface is a notification channel; physical database identity is carried only by the separate identity-mutation payload. Listener registration returns an unsubscribe handle and notification fanout does not turn listeners into storage owners.

`session-row-changes.ts` is stricter: transaction capture is tied to the SQL publication anchor, savepoint rollback removes captured postimages, and committed publication installs private/authority facts first, then resident projection listeners, then public notifications. Pending/indeterminate work uses `invalidate`, which fences facts/projections without announcing a committed public change. Failed committed-fact installation triggers invalidation instead of synthesizing rollback. Row revision advances on both committed fact installation and invalidation so derived reads can detect supersession.

The Gateway row projection subscribes separately to fact, projection, lifecycle and identity channels. Participant writers publish facts before their display-only lifecycle notification; identity mutation advances generation separately. Runtime subscriptions consume lifecycle events for activity/broadcast behavior and preserve the canonical owner tuple rather than becoming row writers.

Focused frozen oracles:
- `src/sessions/session-lifecycle-events.test.ts@9dbf23171d4ab46980bfc5dfda1a5076e55b16b9`
- `src/sessions/session-row-changes.test.ts@3f8ee5ee40b6ae9d39f4918822d08f2a617e2c68`
- `src/gateway/session-row-projection.test.ts@b221068780a1b74ffcaef91e30d5ac91286f72d4`
- `src/gateway/server-runtime-subscriptions.test.ts@63221da7f0a6cf3249a3d0013608c852a299d7a5`

These tests prove unsubscribe semantics and continued fanout after a listener throws; surviving savepoint postimages with a separate transaction anchor; post-COMMIT observers outside the committing Worker; private authority installation before public observers without duplicate presentation notifications; queued-commit projection settlement; row-store ownership for newly committed unknown rows; generation fencing for lifecycle events; and terminal dispatch joining before final settlement.

### 45.3 Remaining frozen direct row-change subscribers consume committed/invalidation facts; none is a second writer

New frozen direct subscriber identities:
- `src/infra/agent-run-activity.ts@2b2817c64e200167b57ecf103207a6d2c056e69f`
- `src/gateway/server-work-metrics.ts@2222f1c0c2b36a4a27b4a6a4e670564ec1afa1ed`
- `src/gateway/control-ui-public-session-cache.ts@2194ed4e5610fe7f7ccec6c1813875bfe2954156`
- `src/auto-reply/reply/reply-turn-recovery.ts@df62cf7f9b37aee68601e387194367adeb05c5a7`
- `src/gateway/github-publication-discovery.ts@99984886eb62756e6f4ca3b22a720bcb95148c8b`
- `src/gateway/session-access-authority.ts@569c9aadf2523081367967a01557716abf2e234e`
- `src/gateway/question-session-access.ts@af0f3e42bcedc0f85fae837192aad69fe3beda16`
- `src/gateway/server-chat-metadata-lifecycle.ts@4f76b4e5ada7e72417891f2fe7ddd5a206a9e8a4`
- `src/agents/tools/sessions-send-followup-custody.ts@e8d23c0d26b20e3f170176e49d6c8e12edbdd93a`
- `src/gateway/cron-stream-sandbox-authority.ts@36817f9d3ef81657007631ed62443b804e69859f`
- `src/config/sessions/session-placement-evidence.ts@3a728041ac55bb39f1f558a46528e8104a8d9e87`
- `src/config/sessions/session-delivery-generation.ts@d636f687f9ca80ca484e32f6b08fe0b67d0f0f4f`
- `src/agents/subagents/registry/subagent-registry-restart-recovery-coordinator.ts@bc52af01ab601b4e7ff0d88c4ab981bd95175387`
- `src/agents/main-session-recovery/main-session-restart-recovery-runtime.ts@91d5ca789b3425de3c1647ba3449b4a914a1d843`
- `src/plugins/runtime/session-changes.ts@cbf20524fdc2c395f6b26f7e7314e2254591134a`
- `src/trajectory/runtime-window-facts.ts@258cb1e7b8a0fe6bc109380ca5c691c2b744cdb6`
- `src/gateway/server-agent-deletion-recovery.ts@2f343842c5c3b1f2e31ca66e94eb9908aa43b411`
- `src/gateway/session-activity-summary-entry-receipts.ts@113bf99274e134a3c57585dac10012661ec23be2`
- `src/cron/maintenance.ts@59a71be5a9f84c4971e08ae845d735d642f07760`
- `src/gateway/session-incognito-lifetime.ts@472fa50f813a87eea8c821741e0e5eadfb291065`
- `src/gateway/session-cost-usage-lifetime.ts@a7612d7ce5ef0d9eb79dea19fca111d2a0f477a1`
- `src/acp/runtime/session-meta-resume-authorization.ts@b74b78ea9c46da28b42e7213d1e9ae4ddce2a473`
- `src/gateway/session-recovery-source.ts@3c6289c6a395bc98312d6979c3b29a89ccc97cf6`
- `src/gateway/server-connection-state.ts@527bb3a278b2e26dbed0a8f29ab517d07c036699`
- `src/plugins/service-mcp-events.ts@717799923ed880afa608024aea7b5a9440dd3f55`
- `src/acp/runtime/session-meta-memory-control.ts@36cbc5d57af2ff7151e632cc254bf12b2f031685`
- `src/agents/worktrees/owner-protection.ts@099db66db3f8ab49359b5bc943c335361e4b9d6e`
- `src/gateway/session-utils-read-lifetime.ts@5e3014e9df59b84ff9d60e62b2a724743682e43e`
- `src/agents/tools/sessions-control-authority.ts@8224ea8d0199d70994e7242dc4a86a4fb41ba70a`
- `src/state/agent-deletion-journal.read.ts@e617f627466d74c50d9a0cb5ed792355a0995eda`
- `src/gateway/session-utils-store-retained.ts@48f14779219dc56e62464f55291b1ed9dd4ff101`
- `src/gateway/session-utils-store-lookup.ts@5f0f621ba6006e77db96759f4300ae5efc079764`
- `src/gateway/session-sharing-preparation.ts@7f015c63748f7099a8d91264451dc525e5494f87`
- `src/config/sessions/session-actor-replica.ts@ca84f4a69573c977df2323bf913c04d348efdef2`
- `src/config/sessions/session-entry-read-ordered.ts@dadefe564992231dc87e9f80394529bccb1ebbaf`
- `src/gateway/control-ui-session-pr-prepared-read.ts@a50a63c2e080acce229eea51211450e825d5a2eb`
- `src/gateway/server-worker-placement-startup.ts@3fbdd8c99aff71b775bd1a6624e3428ff1db1bb1`
- `src/agents/tools/sessions-send-communication.ts@45f1deba220a921dbe0f597a732a4abc7f8736be`
- `src/agents/embedded-agent-runner/run/model-setup.ts@4f630f68032f58c5f86c945c111526583a4a02c7`
- `src/gateway/control-ui-session-pr-subscriptions.ts@cdd25e0c2efd66b1bd7a64fc2b22b3b0167d4667`
- `src/agents/subagents/spawn/subagent-session-store.ts@91681e14d2c2562d9219d0d7aba790eda16f5bee`
- `src/gateway/worker-environments/placement-session-retirement.ts@6ec6f2ce9d7ef562cf916e12bbf5c8a6826f9152`
- `src/agents/subagents/registry/subagent-controller-read.ts@a375e08603789022d7ae5982bf0e10249c080d40`
- `src/auto-reply/reply/dispatch-from-config.harness-defaults.ts@8f590e44c894938b223018b2196f567eb75ff0df`
- `src/gateway/server-worker-placement-dispatch-admission.ts@2eb9696067901e5177d0c958beee019aa6011b92`

These subscribers use committed facts or invalidation as freshness/authority fences for run activity, metrics, Control UI caches, reply recovery, GitHub discovery, access checks, questions/follow-ups, cron/sandbox authority, delivery generation, recovery, plugins, trajectory windows, deletion recovery, maintenance, incognito/usage lifetimes, ACP metadata, MCP events, worktree protection, retained reads, sharing, actor replicas, ordered reads, worker placement, model setup and subagent ownership. Several explicitly ignore runtime-only changes for storage authority; others subscribe to facts/projection rather than public notifications because permission or routing decisions require committed private facts first. Their callbacks invalidate/re-read/recheck retained authority; they do not acquire transaction ownership merely by observing a change.

Previously registered direct subscribers (`cron/service/scheduler-actor.ts`, `cron/session-reaper.ts`, `gateway/mention-inbox.ts`, `gateway/github-publication-source.ts`, `acp/runtime/session-meta-worker-mutation.ts`, `config/sessions/session-registry-maintenance.ts`, `config/sessions/session-transcript-authority.ts`, and `config/sessions/session-accessor.sqlite-maintenance-kick.ts`) were re-read and not duplicated. `src/cron/parent-reset-watch.ts` appears on moving main but is absent at the frozen ref and is excluded.

### 45.4 Entry-creation publication feeds exact generation/membership/projection consumers

New frozen identities:
- `src/config/sessions/session-accessor.sqlite-entry-cache.ts@691f2cde7600892ab1e2612ac74e5fdb3cf06ed1`
- `src/gateway/session-row-projection-generation.ts@74791dc7ed84fbd91cd3184c9c29bea7556b2cac`
- `src/config/sessions/session-reset.ts@20663fede42b8b95e0d9086c0dc00c8afa6e2d6b`
- `src/gateway/session-row-projection-transcript.ts@e204f10af07e982775c1d70509814a7cae705d88`
- `src/config/sessions/session-delivery-generation-memory.ts@2505eb31b67bd4778ffb4983a37883ed98ac65f8`
- `src/gateway/session-row-projection.ts@7864ea9bfb00c04374d6a63d714384c09e54aa1c`
- `src/gateway/session-membership-projection.ts@8fc1989b90e21b2bf9e4beb622e271b63ecadf87`
- `src/gateway/session-row-projection-record.ts@27fa159634bf3cfcc4c4c7b5ebff06e84f8d9daa`

The entry cache re-exports the canonical creation-publication API and treats only complete current metadata as reusable. A generation gap or raw/sibling write forces authoritative reload. Derived cache projection failure invalidates the cache without rolling back the authoritative write. Participant and sharing publication carries prepared source identity/revision when available; missing derived facts require reconciliation rather than invention.

Generation projection consumes committed identity mutations. Reset binds its prepared entry publication to the committed reset event. Transcript projection accepts a prepared source only when its physical identity matches the row's current store identity. Memory delivery generation binds creation only once to the exact memory database incarnation and rejects changed creation admission. Membership projection explicitly states that committed session publications are its only refresh clock and rejects source identity/revision mismatches. Worker placement dispatch retains the prepared entry predicate and distinguishes its own committed publication from a foreign/superseding publication before granting dispatch authority.

Focused frozen oracles:
- `src/config/sessions/session-creation-publication.windows-path.test.ts@34faed9d085964b348ea2c0623030427d1e62c33`
- `src/config/sessions/session-accessor.transcript-turn-initialization.test.ts@cdae09e8239e3f3743b4a1ddb4cd81e0f82aae5b`
- `src/config/sessions/session-accessor.sqlite-entry-admission.test.ts@70abf59fd0e5504f83bb3c2cca047b0092401c52`
- `src/gateway/session-sharing-creation-receipts.test.ts@f4efbcc5e3dbca41be2a9c388862765354aeba77`

The creation oracles cover platform-path publication identity, atomic first-session/header/Goal/input/run creation, rollback of placeholder/header when the receipt cannot commit, refusal to recreate a concurrently created/deleted target, FIFO and physical-owner retention during cold admission, currentness recheck after commit admission, and preservation of acknowledged creation through its committed sharing receipts.

### 45.5 SessionManager committed errors preserve the durable receipt and forbid replay

New frozen identities:
- `src/agents/sessions/session-manager-persistence-error.ts@b639632adcbce2968499e2dcad64fcf9afc4ef8e`
- `src/config/sessions/session-accessor.sqlite-compaction-runtime.ts@47bbf3660095b880380943ee9a96619a876f657b`
- `src/agents/sessions/session-manager-append.ts@f624df73e5463b0b0599cd6287b914d48b020725`
- `src/agents/sessions/session-manager-actor-append.ts@c4fd712adec10532fc84c029ec3a9f2ef5e1b56e`
- `src/agents/sessions/session-manager-memory.ts@a8f21ac67a90ff030b58655c28afec87302c1566`
- `src/agents/sessions/session-manager-entries.ts@b4f935e22d7e628da1553e53febf28a5caf19540`
- `src/agents/sessions/session-manager.ts@1b91fb2ef35d9d21a84a50c42156a0b622839292`
- `src/agents/sessions/session-manager-suffix-persistence.ts@43870f0f020395c96de57dca7118553d2305c3d2`

`SessionManagerActorCommittedError` carries the acknowledged typed reply privately and is named explicitly as a do-not-replay condition. `receiveSessionManagerCommit` unwraps only an error for the exact command and returns the committed reply plus the publication failure; unrelated failures still throw. `SessionEntryCommittedError` similarly carries committed entry id, exact target and committed transcript version and explicitly forbids replay after publication failure.

Compaction converts an acknowledged publication failure into `SessionEntryCommittedError` with the exact committed boundary/version; projection reconciliation is scheduled only for a successful committed receipt without that publication failure. Append adoption treats a later native SDK view as superseding rather than replaying its older receipt, and if an actually appended entry cannot be adopted it invalidates the local view and raises a committed error. Rewrite and suffix operations follow the same rule: after an acknowledged commit, view-publication failure invalidates the view and is surfaced as a committed error; it does not reissue the mutation.

Focused frozen oracles:
- `src/agents/sessions/session-manager-target-capture.test.ts@65e4bcad5492ab3ccae46381e918d8281c78c3ad`
- `src/config/sessions/session-accessor.compaction-boundary.test.ts@2659792ffb09b24e996633495d81ebc5cd60b3be`
- `src/agents/sessions/session-manager-memory.test.ts@830dec6462404cd810fc06f4de2860217586db19`

They retain committed raw-write facts across target changes, capture the target before cwd/read/callback drift, roll back when compaction authority fails before COMMIT, recover from Worker crash before COMMIT, retain a committed receipt when later manager retargeting disrupts publication, and retain a committed initial-writer receipt even when host publication throws.

### 45.6 Closure result

Direct module-import searches for `session-transcript-commit-completion.js`, `session-manager-persistence-error.js`, `session-accessor.sqlite-entry-cache-publication.js`, plus direct `onSessionLifecycleEvent` and `sessionChanges.subscribe*` searches were reconciled against the live Round87 dossier. Existing Section 43/44 identities were not duplicated. The broad `session-transcript-reconcile.js` import surface is downstream reconciliation infrastructure, not a new mutation owner exposed by this frontier; this section records only reconciliation calls directly coupled to the committed-error paths above.

The closed chain is now: durable transaction/Worker commit -> exact receipt -> committed completion join -> fact installation -> resident projection -> public lifecycle/row observers -> downstream authority invalidation/re-read. A callback, observer, projection, logger or reconciliation request cannot retroactively turn a committed mutation into rollback, authorize replay, or become a second writer.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All tests above were read only at the frozen ref and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 46 — transcript reconcile lease/readiness and retained authority closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). The core `session-transcript-reconcile.ts`, entry-cache publication state, projection writer, server-close integration and several lease/admission/publication oracles were already registered in earlier Round87 sections; they were re-read as context and are not duplicated here.

### 46.1 Reconcile pool, readiness and Worker keep lease settlement authoritative

New frozen identities:
- `src/config/sessions/session-transcript-reconcile-pool.ts@7b4d5940afb7c0c00232d22280dc42e4a635da34`
- `src/config/sessions/session-transcript-reconcile-readiness.ts@b918dffc9bd62c4984454ab818d9660851b4e74c`
- `src/config/sessions/session-transcript-reconcile.worker.ts@da7d2c020e9be0f6f384cbc7875f03d94000d4df`
- `src/gateway/server/process-vitals.ts@d60ead9522356487ea432dd03c9d0cf9e1a4224d`

The reconcile pool tracks the complete operation owner, not just the parser task. Each operation has its own abort controller and registered database async resource; revocation stops new work while accepted native settlement remains joined. Pool shutdown closes admission first, waits accepted operations, permits already accepted owners to dispatch lease-release tasks, and only after a five-second grace aborts remaining operations/tasks before closing the worker fleet.

Task finalization joins any in-flight message handler before asking the planner to release its independent lease. If the planner closed without a definitive lease result, the same operation launches a release-only task using the original lease id/path/state directory. A cleanup failure is surfaced as a restart-before-agent-deletion error; if the projection operation also failed, both failures are preserved. Native lease cleanup is therefore never converted into mutation replay or silently treated as success.

The Worker captures the database path identity before admission, requires lifecycle coordination, claims the database lease before opening its read-only database, and rechecks the captured physical identity. Each prepared plan/chunk/finalization step advances only after a parent `continue` acknowledgement. After its terminal `done`/ `failed` message, the Worker waits for the parent's explicit release command before closing its read-only database and releasing the lease. A release-only task can finish the same durable lease after planner retirement.

Readiness is observation-only. It uses the retained database execution when available, otherwise a read-only database check. During database retirement it reports not-ready rather than attempting repair; an explicit abort still propagates cancellation. Process vitals only reports the pool snapshot and does not own reconciliation state.

The previously registered core owner was re-read: one database path has one `runningReconciles` owner; later writes latch `pending` and at most one preferred session onto it, and bounded backoff prevents sustained writers from causing an unbounded tight rebuild loop. Requested-session waits poll only while that session still needs reconcile; revoked work retains its close fence until settlement. A failed background repair is rediscovered from durable dirty state on a later request/restart.

### 46.2 Remaining direct reconcile consumers trigger or await repair without becoming the projection writer

New frozen identities:
- `src/gateway/server-startup-session-migration.ts@0a592a377e459418a7d18f332fbc13b7c64b0ead`
- `src/gateway/server-methods/chat-send-retry.ts@b8c747b507dcd3d03fa69a113bc6312a2b7ae8e7`
- `src/gateway/session-transcript-title-reader.ts@900a4a3bfaf04d266bdcc12d93a2fb292d578a65`
- `src/config/sessions/session-transcript-match.ts@fdf096280ca244be8e70ecd04d947f26150f796e`
- `src/gateway/session-activity-summary-source.ts@47281c135c34421f25d0916e4c4d51011724dd9d`
- `src/gateway/session-transcript-preview.ts@a954447fa1f15e808a8d6414b0b56e9e43376037`
- `src/config/sessions/session-accessor.sqlite-active-projection.ts@75da205498d4cc0cc0779fa370f4694a802df893`
- `src/auto-reply/reply/memory-flush-session.ts@f03f7ebf07b99a48b62e6290183efd440a18b934`
- `src/config/sessions/session-message-cut.ts@91e8cf58f7eeef44cb7dffaa698fc186b8f17e33`
- `src/sessions/user-turn-transcript.persistence.ts@120322db2fe81093573e4123a1cd05d0354f2960`
- `src/config/sessions/session-transcript-delta-read.ts@900f6ad57f8cd30d66e039f9f0c86c1bb0ebd906`
- `src/sessions/user-turn-transcript-annotation.ts@a51e6492cbbdeb18f20af8bc3409af63f1ad632c`
- `src/config/sessions/session-transcript-search.ts@b94a0eee99d85cf64b1192446c08f8f9353628fe`
- `src/gateway/server-methods/chat-send-commentary-media.ts@b09ae96efb906c627f6d689128c47b2843980b49`
- `src/gateway/server-methods/chat-send-session.ts@09a10ca981e72406fbb54b713747002120652207`
- `src/config/sessions/session-accessor.sqlite-transcript-batch.ts@2131cb9e85341263104127b198d85308519a2da4`
- `src/config/sessions/session-history-worker-runtime.ts@ae06c627437aff42327e3bf7da5b1a070edd1978`
- `src/agents/embedded-agent-runner/run-orchestrator.ts@9825cd4682c70d60c68b4f45a028bbc651915860`
- `src/agents/embedded-agent-runner/run/session-prompt-state.ts@822cf185ff10e2bb530706027f9e15b551e83b0a`
- `src/config/sessions/session-accessor.sqlite-transcript-rewrite.ts@f81755105e871166b0a413f7677f12a0e7f56ba4`
- `src/config/sessions/session-accessor.sqlite-active-events.ts@dedccd7add7861eb48ed65efb8d3a902bd0cf9cd`
- `src/agents/embedded-agent-runner/run/attempt-prompt-submit.ts@b9142b6bbb031989a9bf67dba6740973bd373c8d`
- `src/config/sessions/session-accessor.sqlite-transcript-store.ts@0b096873adac63a964887254033d38eeb673181d`
- `src/config/sessions/session-accessor.sqlite-transcript-suffix.ts@5eeb7dbeb1cb923333b37903f4bd25ef48d54510`
- `src/agents/embedded-agent-runner/run/attempt-session-prepare.ts@aa46883ca1da868aa3203ae2de60c16ab6900a67`

These direct importers use the canonical reconcile surface in three bounded ways. Startup/migration code may run explicit reconciliation. Read/search/preview/title/activity/chat/agent paths either start repair after a projection-unavailable result or wait only for the requested projection before retrying the read. Transcript mutation helpers mark/schedule the derived index repair after their owning durable write; they do not parse or write the index from the caller stack.

The previously registered reset, lifecycle publication/projection, turn publication, SessionManager, recovery, branch, message runtime/persistence, locked-write, compaction, suffix and worker transcript-commit paths were re-read and not duplicated. Their `projectionNeedsReconcile` handling is post-commit repair: a committed receipt may request reconciliation, but a publication failure represented as a committed error is not replayed. Dirty projection state remains the durable rediscovery mechanism.

### 46.3 Physical publication scope prevents logical aliases from donating stale row authority

New frozen identity:
- `src/sessions/session-row-facts.ts@604d6215aa0a15e85a19458e099622e9b321cf12`

`prepareSessionRowPublicationScope` records both configured/unsuffixed paths and any selected physical database identity captured before reads. `sessionChangeAffectsStoredRow` rejects display/runtime/auth-only changes and unrelated session keys. When a physical publication identity exists, it wins over path matching; otherwise canonical/resolved paths are used. A logical agent id alone does not fence a physical row publication, preserving cross-owner aliases correctly. Optional topology ignores are explicit.

This is a read/publication fence only. It neither resolves a new owner nor writes storage. It lets downstream readers decide whether their previously selected physical source has been superseded.

### 46.4 Retained entry predicates keep authorization current across pending/superseding publications

The canonical predicate implementation `session-accessor.sqlite-entry-cache-publication-state.ts@a13dcafeaf70a301afb18201604fb7e84f46ddbc` was already registered and was re-read. It retains a physical `databaseIdentity + sessionKey`, tracks pending publications, captures a revision, and accepts refresh only when the retained predicate remains active/current and the revision still matches. Pending policy/postimage work withholds current authority; unknown sharing authority cannot be repaired by an arbitrary fresh snapshot. `prepareRead` joins already admitted publications but queued writers cannot block their owner.

New frozen direct consumer identities:
- `src/agents/harness/node-execution-authority.ts@6c57c74e3ce5253ed8b2ee7e58a6e0b7f5c35d85`
- `src/auto-reply/reply/commands-compact.runtime.ts@8085a2d463298cc96cf168ffcc5871d835308833`
- `src/gateway/worker-environments/session-attachment-session.ts@6e430362c46493660f86e39d4ce0b1ccf0203e3c`
- `src/auto-reply/reply/reply-tool-authority-cohort.ts@cbf106c1f9c9473475144878d566d992a47fee4b`
- `src/gateway/agent-turn/agent-run-final-media.ts@dc7ff02f441189ab579fdf0ef8d37b5cfb1a8d0b`
- `src/gateway/server-methods/chat-metadata-session-read.ts@cf07db4a3d724f8f84a3fa06ef70e8d6027b61ec`
- `src/auto-reply/reply/steering-delivery-preparation.ts@a60a95b151728700ce204b1dd733f3d4e7339ca7`
- `src/gateway/server-methods/session-rewind-repository-entries.ts@ef321f7a81cd8dd3077181ba77d4aa40dc21dde7`
- `src/gateway/approval-channel-custody.ts@34249d9b662959a222973b10edd4c347339c1d51`
- `src/skills/workshop/experience-review.ts@dbd80e1ae4696233bb451108a1d9d4475847684b`
- `src/plugins/runtime/runtime-agent.ts@c24b43e1aa810fcf374b3f8640477251a4b484aa`
- `src/config/sessions/session-entry-source-authority.ts@6c527569f4f5efde086887cf96641db33d0cd12e`
- `src/gateway/node-claude-skill-runtime.ts@4d5e5858c5c90b3b411f3662b5884c763a23b877`
- `src/auto-reply/reply/session-writer-delivery-authority.ts@aba2abffd8a1d620e6436c83d20befad73aa6e69`
- `src/gateway/server-methods/skills-library.ts@8d08bd48cbca629229b37a0eabe2f758fb7bda66`
- `src/gateway/server-methods/chat-reply-media.ts@a713240920bcf31c3c8151acefe45ead906da851`
- `src/gateway/server-methods/sessions-delete.ts@e12aa82b9a33b411d0106184adbaf6a45ef3c7c5`
- `src/agents/main-session-recovery/main-session-restart-recovery-exact-target.ts@7f2a2b9095a9e1d8e751b99de2c1edd8e7c5a2e0`
- `src/gateway/github-publication-availability.ts@9ea701c92c2732de2fbc4b03eb912fa813639a70`
- `src/agents/cli-runner/cli-run-transcript.ts@67a97717f9eee19f30b30a63dd64fa68387ef469`
- `src/config/sessions/session-entry-current-check.ts@b2c4673a568fce1927d914d561da4553d450e07d`
- `src/agents/agent-harness-completion-recovery.ts@fe19a1bc2a5db741ad3516ed33e3d1ebd3d099bb`
- `src/plugins/runtime/runtime-agent-session-create.ts@55bbb43724aa8859008fd3c2eb40da7fb8283182`
- `src/gateway/server-worker-placement-session-target.ts@008732e0a8ef63b65a4eb226a6d2788effe655dd`
- `src/gateway/worker-environments/local-workspace-projection.ts@fed03c97fe7e166c4880e80c34389f0e6fd3d981`
- `src/gateway/worker-environments/worker-turn-transcript-target.ts@e2c5503722ffee38c2bedb3d786d56ccb608a9ca`
- `src/agents/main-session-recovery/main-session-restart-recovery-delivery.ts@88e0dfe7fd57b85f6fa68ad7eb9209308f86fb58`
- `src/agents/main-session-recovery/main-session-restart-dispatch.ts@b762ce0bf29ca4d5d239e906a478b344cfc46d50`

Node execution separately retains session identity and Full-permission predicates; live facts can revoke admitted permission but never create new Full authority. Session deletion joins a pending publication, re-reads through the Worker and only `acknowledge`s the refresh if the original physical source and captured revision still match. Reply media waits retained publication before attachment delivery and rechecks delivery/placement authority. Plugin runtime, approval custody, exact restart recovery and worker transcript preparation all release their retained facts on exit and fail closed when the selected identity/lifecycle/delivery predicate changes. Worker transcript preparation can keep a separate settlement predicate for already completed results, but it remains bound to the same physical source.

Previously registered `session-utils-read-lifetime.ts`, `subagent-session-store.ts` and `server-worker-placement-dispatch-admission.ts` consumers were re-read and not duplicated. `src/gateway/server-methods/chat-inject.ts` appears on moving main search but is absent at the frozen ref and is excluded.

### 46.5 Focused oracles cover scheduling, readiness, native exit, shutdown, lease phases and retained-fact invalidation

New frozen reconcile/lifecycle oracle identities:
- `src/config/sessions/session-transcript-reconcile.scheduling.test.ts@76a3455f1e4a73182e58d63059fd0eff673f8368`
- `src/config/sessions/session-transcript-reconcile.readiness.test.ts@dcc2ec5b073a5c3444a6f9ead11fce887e0fe54a`
- `src/config/sessions/session-transcript-reconcile.native-exit.test.ts@1d06c3bfa80955290690f176c19ea697d9c91637`
- `src/config/sessions/session-transcript-reconcile-pool.test.ts@3f62b0bf9399072f98773b3abfeadc4d1895ce44`
- `src/config/sessions/session-transcript-reconcile.lifecycle.test.ts@3dec9aadc83f5feeae3f7e433f492369e9a137bc`
- `src/config/sessions/session-transcript-reconcile.lifecycle-phases.test.ts@b03b18806589f36249ce5cc61285fde48912794d`
- `src/config/sessions/session-transcript-reconcile.bootstrap.test.ts@0d53d644c933e49afe228ee615bb9079faf32fb5`
- `src/config/sessions/session-transcript-reconcile.backlog-order.test.ts@d9ee41a3b624ed69801e33c71066fe7237dd7993`
- `src/config/sessions/session-transcript-reconcile.test-support.ts@9565c1e3d88140e269d45354941d7bd6e21ae89a`
- `src/test-utils/openclaw-test-state.test.ts@cd05531592bdd4f4a2de3d075d17c8c8dae75f4a`
- `src/config/sessions/session-accessor.sqlite-active-events.test.ts@81b43e4929df8dbb2c77327192ebfdb853becf84`
- `src/agents/embedded-agent-runner/run-orchestrator.projection.test.ts@41da168c82c37473ec00dcc26596c13bcfd376f7`
- `src/agents/embedded-agent-runner/run/attempt-session-boundary.orphan.test.ts@d4c517c9de28ebd37f267c3a98fb95f8c12d58d1`

Scheduling drains deferred work after the caller retires and does not invent a missing store. Readiness can resolve the foreground session while foreign writes continue and preserves cancellation/lifetime. Pool tests reject queued admission after shutdown, drain accepted active/queued work, join lease settlement before agent close, and revoke a scheduled reconcile before its first disk admission. Native-exit tests join queued lifecycle phases after native exit. Lifecycle tests cover reusable fleet workers, state-directory-scoped draining, requested-session readiness before unrelated repairs, pending-pass backoff/coalescing, idle retirement and keeping the operation pending through lease release. Lifecycle-phase tests cover cold/warm cleanup and shared-state close failure after lease deletion. Bootstrap refuses replacement during first canonical creation and joins revocation before dispatch. Backlog ordering yields at a session boundary so another agent gets a turn without introducing a second priority queue.

Previously registered `session-transcript-reconcile.lease.test.ts`, `continuation.test.ts`, `admission.test.ts` and `publication.test.ts` were re-read and not duplicated.

New retained/publication oracle identities:
- `src/config/sessions/session-accessor.sqlite-owner.test.ts@51f9db197f55583486280bc78bb8d7eb2d5d1c7b`
- `src/gateway/session-row-projection.accepted-facts.test.ts@7c3b7c7ccecff99ea74adb3fd3f78f54bd46dee9`
- `src/gateway/session-row-projection.worker-read.test.ts@a89b3e94bf5dc8f8fca0aabc49a9f60fc5c06117`
- `src/config/sessions/session-transcript-worker-readers.test.ts@f84264ce40604f9972f356e94be467539cc15d2d`

These oracles refuse stale session identity/lifecycle before owner assignment, preserve accepted facts only while custody is current, discard invalidated facts after release, keep newer keyed replacements over older Worker replies, fence replacement lifecycles, and preserve exact serialized read/cancellation/result contracts. Previously registered actor/replacement/transcript-report/worker-transcript oracles were re-read and not duplicated.

### 46.6 Closure result

Exact module-path and symbol searches for `session-transcript-reconcile.js`, its pool/readiness/Worker modules, `prepareSessionRowPublicationScope`, `sessionChangeAffectsStoredRow`, `retainPreparedSessionEntryPredicate`, and committed `projectionNeedsReconcile` callers were reconciled against the live dossier. Scripts/benchmarks were treated as support rather than shipping owners. Moving-main-only paths were excluded by exact frozen-ref 404 checks.

The resulting lifecycle remains single-owner: durable transcript writes mark derived projection state dirty; the reconcile owner prepares off-thread and commits through the canonical projection writer; a retained native lease survives planner/owner retirement until exact release; readers can wait for only the projection they require; retained row predicates can revoke or refresh authority but cannot mint permission or replace physical source identity. Failed publication/reconciliation remains repairable from durable dirty state and never authorizes replay of an already committed mutation.

No Actions, build, test, generator or package task was run. No production code, parity ledger, RTM or completion flags were modified. All tests above were read only at the frozen ref and were not executed. Independent ACCEPT, scoped G01/G02 positive evidence, Rust implementation/wiring, executable/package proof, OCRM final acceptance and release acceptance remain absent.


## Section 47 — frozen post-commit publication and writer-owner direct-consumer closure

This increment continues Section 42 against the unchanged frozen source `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`. Discovery searches only enumerated candidate import/symbol users; every identity retained below was fetched again at that exact commit. The moving default branch is not evidence authority.

### 47.1 Archive/maintenance and existing instrumentation dedupe

The direct module-import surfaces for `session-accessor.sqlite-archive-store-kernel.js` (10 discovered paths), `session-accessor.sqlite-maintenance-transaction.js` (5), and `session-accessor.sqlite-maintenance-worker.js` (1) were re-enumerated. Every frozen direct consumer was already present in Sections 41–46, so no duplicate identity is added here. The already-recorded writer anchors `src/config/sessions/session-accessor.sqlite-scope.ts@0af27f285fc4362025e6dcd19df34e2f1e80b7f5`, `src/config/sessions/session-accessor.sqlite-write-operation.ts@b8ca6e93718adf94daa6d4721629228b5393be62`, `src/config/sessions/session-accessor.sqlite-page-reclamation.ts@0961c128bf30f246b7d706c9a365461959ec8256`, and `src/config/sessions/session-accessor.sqlite-reclamation-wal.test.ts@4320915885039e6e5fca969d87f8416a8d7b1a2a` remain canonical.

The frozen post-commit kernel is `src/infra/sqlite-post-commit.ts@a203acefc5d28361a24a620fd132c8f264e8f4e9`. Its direct-symbol searches stayed below the 100-result cap: `deferSqlitePostCommitPublication` 46 paths, `stageSqliteCommittedPublication` 10, `stageSqliteTransactionState` 55, `publishSqliteCommittedState` 9, `withSqlitePostCommitPublications` 42, `hasSqlitePostCommitScope` 11, `discardSqliteTransactionState` 2, and `prepareSqliteTransactionCommit` 3.

### 47.2 Newly frozen direct production consumers of post-commit publication

Previously unrecorded frozen direct consumers/re-exports are:

- memory/SDK/protocol: `packages/memory-host-sdk/src/host/memory-schema.ts@ac17f408d0f41ecfee2015b536aa68a3b2460829`; `packages/memory-host-sdk/src/host/openclaw-runtime-sqlite.ts@b4d37e4203066165ae8adc2bf221e7b683a5ad69`; `src/plugin-sdk/sqlite-worker-runtime.ts@480c150f93bb40c575f62f3c38a97cf63468f6c1`; `src/plugin-state/plugin-state-store.database.ts@2c72678d5174151cd7788374b678b3408b44e0e3`; `src/acp/runtime/session-meta-keys.ts@40933366f2e336044ca47884c53e64ac39281c25`.
- auth/workspace: `src/agents/auth-profiles/credential-write-custody.ts@562d09e33ebeeeaf7fcc1ddf57e773c50ba5fae2`; `src/agents/auth-profiles/runtime-snapshot-publication.ts@ef386ab528b0efb654408afd733bd929e95c6be8`; `src/agents/auth-profiles/store-update-commit.ts@f9d1c34475c7448e5651913a6634338876c19560`; `src/agents/auth-profiles/store.ts@e722a7cf907e7c4b266bf33dd25449fe1770b118`; `src/agents/mcp-oauth-store.worker.ts@5209d34ced328d6e903f67f3069af4136217b1d1`; `src/agents/subagents/registry/subagent-registry.store.worker.ts@092469df750eb5eef58230ed3055972c64ed7321`; `src/agents/workspace-alias-rebind.ts@a478ede563a26622f2b36b18ce3e525793e8a826`; `src/agents/workspace-state-store.worker.ts@f29a3dec88cedd64a4ea51199ca5f0fe78c08c39`.
- session state/cache/read: `src/agents/sessions/session-manager-native-persistence.ts@76b97d470484939d1d570006b91a5ff466d76db1`; `src/config/sessions/session-accessor.sqlite-creation-read.ts@98041eb09f224528eaf7876ddd35cd1150c9b7af`; `src/config/sessions/session-accessor.sqlite-entry-cache-state.ts@ae6f85f6ac9ba2b32dca5175ca2bff5eac9b3cdc`; `src/config/sessions/session-accessor.sqlite-entry-list.read.ts@87a8680ea6bc039bfdb2a205de74445e923245b8`; `src/config/sessions/session-accessor.sqlite-maintenance-age.ts@67e6e5581bda0310e2528a29be771a3edbc4892a`; `src/config/sessions/session-accessor.sqlite-pending-inputs.ts@8c8f542547392049b30f9d3f6438ec94af320790`; `src/config/sessions/session-accessor.sqlite-summary.ts@5b0b4316755e48e940397ac247935a144ae2759f`; `src/config/sessions/session-accessor.sqlite-transcript-suffix-write.ts@f1bb0a9f5b2c3ff796306fa2c2087b1b8c3e8284`; `src/config/sessions/session-accessor.sqlite-transcript-write-snapshot.ts@fceb33b2a25ae2cab9bf012c0ee15a691f468fc1`; `src/config/sessions/session-actor.worker.ts@5a3992b5fc04e2f22dacf6e29fcff602c60c241d`; `src/config/sessions/session-canonical-key.ts@c924c5f37978311ac7ef66ac2601d8e642567ad1`; `src/config/sessions/session-entry-list.worker.ts@1af835e32c4a66daeccf4d69f017621169604ef2`; `src/config/sessions/session-membership-facts.ts@fb43774dd82ee96d207191cf66ebc15c4e12b001`; `src/config/sessions/session-participant-contribution.worker.ts@31f2c21fffcb6271f2729fda7c947b3497b565a4`; `src/config/sessions/session-transcript-index-status.worker.ts@97eeaac39a8fc4df59964e6308eb6eaec031f105`.
- cron/config/doctor: `src/cron/store.ts@29816d89e4697146e775167ecf57407e158b6174`; `src/cron/store/run-history-changes.ts@89cabcc5d10aacf6da62291f13a52cdc800cd69c`; `src/cron/store/save.worker.ts@f7c38d8b90c81e16f90b9f16e92917339b27452b`; `src/cron/store/scheduler-actor.worker.ts@ff7433073a29dad69df5599f7dc689b0d00e9fe2`; `src/commands/doctor/cron/store-repair.ts@e0cce2e46f0542766335f170c6b7899299b84594`; `src/config/io.health-state.ts@4508fb9c768efbbbaa10d42c101ee20c5f30cf6e`.
- gateway/device/outbound: `src/gateway/github-publication-events.ts@037183b5ecab94de10449bd6ec172272e4366a26`; `src/gateway/worker-environments/placement-turn-authority.ts@d599e32f4842f70c94878acf580b49df46a832be`; `src/gateway/worker-environments/placement-turn-claim-events.ts@a4d5e1422aafed267e8f3f5e8184f2930568431f`; `src/gateway/worker-environments/store-native-publication.ts@b2151df835dcda63c3fc81b5e48b4efe79734fb6`; `src/gateway/worker-environments/store-schema.ts@9c8ec983c0d59642f9436e8eb2d1a589541b7cb6`; `src/infra/device-pairing-join-code.worker.ts@9972c1aba9e6e728d75784f377ca8dd33576f68a`; `src/infra/device-pairing-store.ts@b7592120467b67ab19011d3ce3eea055559adf25`; `src/infra/outbound/current-conversation-bindings.publication.ts@fbc5913ddf7d85d12cf8bbb8d4d973ac9c183f5b`; `src/infra/outbound/delivery-queue-enqueue.worker.ts@3895f81d409caa3141833080408e55821144a2df`.
- transaction/admission infrastructure: `src/infra/client-credential-migration.ts@343bfc9838747265c71579ba3dcaf0a43560b79f`; `src/infra/database-engine/open.ts@cb865035bc0f523b7f3acb812dad04fe3f6aba07`; `src/infra/database-engine/search-extension.ts@9ea3e3f2dfcd06673e5d143871f9e7aa37a9e73e`; `src/infra/postgres-sync/connection.ts@77ede3f72c226357652d30aecc58f334e27f09f8`; `src/infra/postgres-sync/transaction.ts@03c26067560006e2eb5ac1de2402ad70e845b618`; `src/infra/sqlite-database-admission.ts@ea267e5c7abccb15466f5b606b896eca28afa585`; `src/infra/sqlite-database-write-receipts.ts@c381431762db0501fa63b90bfdf3112fdfab981b`; `src/infra/sqlite-schema-facts.ts@6125196b7a8e256066d251b2557cf54716fa3219`.
- state/registry/lease/profile: `src/sessions/session-state-events.kernel.ts@92644d06bc3ab9d00b6cda507bad2c6230b32a3c`; `src/state/openclaw-agent-db-registry-listing.ts@dcb30c8923134af01850eb7717f615a9d5d2c9a2`; `src/state/openclaw-agent-db-registry.ts@b44c1ef8e01e8ba41d25913af928bce0a56f35e6`; `src/state/openclaw-agent-db-schema-extension.worker.ts@1a2f7dbf81ad80687575317e7bfa7ce47ee4ec64`; `src/state/openclaw-agent-legacy-acp-schema.ts@b1d605067883ca661eee1fb16c0356f30aba1793`; `src/state/openclaw-state-db-admission.ts@6117b56ab8296c9f798383143416f30f9ed4d1cf`; `src/state/openclaw-state-db-engine.ts@8759a31782d50cdcbf785d2310b14d49d11d50cd`; `src/state/openclaw-state-db-schema-v13-widerow.ts@43c44636b34852126b1063294ddd23ae2aebd60f`; `src/state/openclaw-state-db-transaction.ts@8475c371acfd101d4894d5a1325ffae637846ad9`; `src/state/openclaw-state-lease-worker.ts@9394f8bfefa038c55296444f4a6cb3a504fa059f`; `src/state/openclaw-state-lease.ts@a3781c145848e4b04adb234972f9d39eb82a9c11`; `src/state/state-domain-publication.ts@71ca4b6f0d85d4fbf2e02990da60766886230208`; `src/state/user-channel-identities.ts@ca91e374f8b51ace0e8a8b63f081c9572bb5966e`; `src/state/user-github-connection-events.ts@882c42add1402b47f40ff810d37b9637c5d562ec`; `src/state/user-preferences-publication.ts@7ce3aee7c0578607320c32ee2b3d82b108d3b3c3`; `src/state/user-profile-list.ts@e8d1f2bed73f80dc7b4d200e5c26ebc7262e80ef`; `src/state/user-profile-writes.worker.ts@8bcb8ee3d2b71acc63a17f72d797d2fbdd2e6831`; `src/state/user-profiles-merge.ts@d7798ecececf7f23f8e4eaf7967a7ac557c17e50`; `src/state/user-profiles-owner-migration.ts@20f5e71654ff0a6b94cb51006a552eafd75114b9`; `src/state/user-profiles-schema.ts@81ea28736dd330e5af0f754ea69e40f5da5e9ae8`; `src/trajectory/runtime-retention.sqlite.ts@c21667688c53ab1419c842d6c52bfbf14784a350`.

These paths use transaction-local staged state, commit-time fact installation, post-commit notification, rollback restoration, or unknown-outcome invalidation around their existing domain writer. They do not establish a second durable mutation owner. Credential custody is deliberately fail-closed: unknown native outcome retains exclusive custody until restart rather than risking duplicate credential mutation. Transformed plugin/memory SDK surfaces reuse the same host primitive.

### 47.3 Newly frozen post-commit focused oracle/support identities

- core transaction oracles: `src/infra/sqlite-post-commit.test.ts@a34cd6f289632f4ff4dacfcdd24fc239df69df1b`; `src/state/openclaw-state-db.transactions.test.ts@09efed959c64e123805bcbc4d0603e9c08fc20d4`; `src/infra/sqlite-transaction.test.ts@5f74ac4bbfd7047c6151c09f3a9b5a4617894a23`; `src/infra/sqlite-single-statement.test.ts@1112926169fb5aa39d99de6843331ba4e41c6114`; `src/infra/sqlite-database-write-scopes.test.ts@136d7c3204b6dc9100ee4e2b3032ed00700e6a44`; `src/infra/sqlite-schema-facts.test.ts@080b5a39f1e4111c39333bdfdcc2a7d6fc2784f3`; `src/infra/sqlite-source-fence.test-support.ts@f002e01e4a7d6a6739cc958b0fbd23a8ccf8ac34`.
- domain/session oracles: `src/state/config-machine-state-write.test.ts@b3271d36e7ee9d67ae53a8d4dd46b55bec0fe904`; `src/state/openclaw-agent-legacy-acp-schema.test.ts@5acc44619b1235fc7fe1dc4446df3639c7fd938d`; `src/state/openclaw-agent-standing-intents-schema.admission.test.ts@e2fc0a17b805f1258e9d883488528a0e6ca3d06f`; `src/state/state-domain-publication.test.ts@f8c7391f9f6d0e5760c2ff47466262b431fde390`; `src/config/sessions/session-accessor.sqlite-entry-revision.test.ts@72f4bebdb1560c8889c2120106432a9fb5a028c4`; `src/config/sessions/session-canonical-key-policy.test.ts@47fc740fd375ece8fa6ab72d98837641e44dccb4`; `src/config/sessions/session-canonical-reader-continuation.test.ts@37f862522927211065d0afc3a46fb133be059072`; `src/config/sessions/session-turn-source-custody.test.ts@70b51f9fbc68235b4d6c2feea2d1cfe7b0eb7212`.
- auth/gateway oracles: `src/agents/auth-profiles/credential-authority.test.ts@583e1a54fda822c66c772d089382026ff8971faa`; `src/agents/sandbox/registry-read.test.ts@c8b590395d6e94d202d159d2e3899dde1fb31f29`; `src/agents/worktrees/registry-publication.test.ts@88f9f7147db9e04351c99c649444c9daa8516d6f`; `src/gateway/mention-inbox.worker.test.ts@286475837b60cbb3c96b0ac0065c6468b0b3b226`; `src/gateway/worker-environments/store-projection.test.ts@6259670da93d0807db4772f633830b23c3df5c7c`.
- support-only: `src/gateway/worker-environments/placement-test-fixtures.ts@a495355cbce789f63786bd1f42975c36645e05ec`; `src/test-utils/agent-deletion-journal.ts@b9cff703d060250dbab1a428becc383172c7938e`; `test/helpers/cron/store.ts@d36a5c35e20c79ac9ccb96933a9f657b29bcde9c`; `test/helpers/cron/scratch-store.ts@895ca0f0cfb05a08a554e97f69a1d954e94d2907`.

The core negative oracle is explicit: an observer failure cannot undo a committed database mutation or suppress later observers, while failed fact installation fences/invalidates derived state. Nested savepoint rollback restores staged state and discards its publications; the outer commit owns final installation. Moving-main search also exposed `src/state/openclaw-state-db-schema-additive.test.ts`; frozen `646568eb42592cfb6c693eb5e897fdcc67393549` contains no direct post-commit symbol use, so it is excluded.

### 47.4 Newly frozen direct production consumers of writer-owner instrumentation

`runExclusiveSqliteSessionWrite` discovery returned 66 paths, below the result cap. Thirty-two were absent from the live dossier. Exact-ref reading retained these previously unrecorded production callers:

- `src/config/sessions/session-accessor.sqlite-prepared-write.ts@b3836a00a3dc1da1e8f322d244c9f6f70655022b`: preparation can release the foreground permit, but ordinary commit re-enters the same exclusive writer unless the prepared native worker owns deletion; source currentness is checked again at commit.
- `src/config/sessions/goals-operations.ts@44fcab335c665358ea8011ec4c22a762fa4f2893`: typed Goal mutation uses the existing durable session writer.
- `src/config/sessions/session-accessor.sqlite-transcript-message-rewrite.ts@cb93b088689bfb05cb27a1ac88c8cb7d5f02870a`: transcript rewrite is serialized and revalidates transcript/session generation.
- `src/config/sessions/session-accessor.sqlite-import.ts@af95d61eb4638ca20c2e8ec560b2cef059e68594`: a batch cannot span physical stores and enters the canonical writer before import staging.
- `src/commands/doctor-telegram-general-topic-conversations.ts@eebbb4af64860744c3137bce9281ade2583e5d77`: advisory detection is re-read after BEGIN under the real session writer.
- `src/config/sessions/session-accessor.sqlite-message-cut.ts@f182f54bfbd73528d618dc0fa61e99cdedefb296`: prepared/upstream predicates are rechecked before deletion transaction commit.
- `src/config/sessions/session-accessor.sqlite-canonical-repair.ts@8deb1682e68bcef880873d23dd866b0b69ef7ff2`: repair groups work by database and serializes each group with the existing writer.
- `src/config/sessions/legacy-main-session-migration-operations.ts@9a2f82a845c640280b0843a69ecd4cb9f0b2c84e`: in-place and destination migration apply under the source/destination writer with currentness checks.

The moving-main candidate `src/config/sessions/session-claim-repair-copy.ts` returned 404 at the frozen ref and is excluded.

### 47.5 Newly frozen writer-owner focused tests and diagnostic consumers

- migration/preparation/session: `src/config/sessions/legacy-main-session-migration.race.test.ts@31cd589f3321c0d42fa3fcd7eb242601738f51c3`; `src/config/sessions/session-accessor.sqlite-transcript-write.test-support.ts@ba6373a322c54b4edb40cfbe4379cced453acc54`; `src/config/sessions/session-accessor.pending-inputs.test.ts@0741bdca21edd4cde8df2d6bc8264d62260afdff`; `src/config/sessions/session-accessor.sqlite-lifecycle-admission.test.ts@b58ad13c914a66dc6ad9c8ec0412360524e8ab95`; `src/config/sessions/session-accessor.sqlite-artifact-planning.test.ts@54d08a65f5d27082043159f88b02c0c813dff683`; `src/config/sessions/session-accessor.sqlite-prepared-admission.test.ts@f3ec8cf764a0c822d44cfeb9f9383dbb980a5996`; `src/config/sessions/goals-operations.test.ts@432ff2a30a2b7feb4db4fcc42b520712ef3aaebd`.
- gateway/recovery/authority: `src/gateway/worker-workspace-recovery-binding.test.ts@dd29580605527904c2a8873bc5871e1f56548fad`; `src/gateway/worker-workspace-recovery-transcript.test.ts@e8fd29d86645cd73e7b3f59b8263d95cc8207c55`; `src/gateway/server.sessions.create.spawn.test.ts@43e233e8b81b0b9ce916270022b0df8cc1139758`; `src/gateway/session-activity-summaries.test.ts@84b2f4b6e8f2de12d2fe1654bbbc8e4e074c6922`; `src/gateway/server.agent-input-authority.test.ts@a55c1d881ffd92a7b41c4d747406934cc5c34f81`; `src/gateway/server.sessions.create.authority.test.ts@ecafa5f75a1c1d73f6370eaf9c396eafacffcef7`; `src/gateway/server.sessions.create.lifecycle.test.ts@5beb86483c08e194a4f08e63de4030ec7b3f8c96`; `src/gateway/server.agent-runtime-authority-proof.test.ts@81d71a021cbfb07ad497d1223dc3612b9b1743ee`; `src/gateway/server.sessions.delete-worktree-lifecycle.test.ts@e31764b62e0fa557abf4de105b33fc946ab9785f`; `src/gateway/server.chat.gateway-server-chat-b.test.ts@79a6f0bae810d5138ed28131d05792d7c01cf507`.
- agent/plugin/cron: `src/plugin-sdk/session-transcript-mirror-runtime.test.ts@94ac6a2c516fb5d8b81a8822d61f3598e1008186`; `src/cron/service.remove-session-cleanup.test.ts@32909713421816c0eb0255016ee7f2109351fb51`; `src/sessions/session-agent-work-admission.test.ts@9dbbfa06440dabe644bbe771a9c9f78d0f36f6b0`; `src/agents/harness/host-capability.annotation.test.ts@0c8efd80a1e7ca88d6f1fec578bc063102d5e864`; `src/agents/agent-command.live-model-switch.test.ts@7b942609a17f2b40edbdb89565f5f19156fd566f`; `src/agents/subagents/spawn/subagent-spawn.preparation-authority.test.ts@dd9a8cf450bcf503f21669813642b282ec10ff22`.
- diagnostics: `scripts/lib/gateway-bench-diagnostics.ts@6640e6c74fc4669f407dc7cb1439a68eade90cbf`; `test/scripts/gateway-bench-diagnostics.test.ts@b1ed162735dc0e6b5d434057fb934721fc30413c`; `src/config/sessions/session-entry-snapshots.benchmark.test.ts@cfd1b7d7be0466cc94d196fcf32d799bbe1ca81c`; `src/config/sessions/legacy-main-session-migration.node-handoff.test.ts@225b47b24aa252e26287d700f1404c2ce8a6140a`.

These tests contend the genuine writer, observe queueing, then release it; they do not fabricate durable authority. They cover source/session rebound, prepared-admission stale fencing, lifecycle artifact planning, pending-input custody, delete/create/recovery authority, cron lock release before waiting on the session writer, plugin transcript handoff, and migration races. Diagnostic collection is bounded and removes content-bearing fields; `queueWaitMs`, `writerExecutionMs`, and `completionDelayMs` are observations, not scheduling authority.

### 47.6 Closure/result

This section adds source/provenance only. Post-commit facts install before fallible notifications; nested rollback removes staged state; unknown native outcomes invalidate or retain custody where retry could duplicate mutation. The session writer remains one FIFO authority per physical path, including prepared writes and migration/repair callers.

No GitHub Actions, build, test, generator, package, production source, parity ledger, RTM, or completion flag was modified or executed in this increment. This evidence does not establish independent ACCEPT, scoped G01/G02 positive proof, Rust implementation/wiring, executable/package proof, OCRM final acceptance, or release acceptance.

## Section 48 — frozen downstream publication observers, worker receipts, and remaining writer-consumer closure

Frozen source remains `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`). This section continues Section 47 by following its publication/receipt and writer-owner identities to direct downstream production consumers and focused tests. Every path below was fetched at the frozen commit. Paths already named elsewhere in this Round87 dossier are deliberately not re-counted.

### 48.1 Remaining post-commit domain publication consumers

User preference publication:
- `src/state/user-preferences.store.ts@06a33ea08f4be47d0c638dbc3d46feffb4d67120` is the newly frozen writer-side consumer of `publishUserPreferencesChange`; its database mutations stage the preference revision publication with the owning transaction.
- `src/state/user-background.ts@a149796ff29d5db13c0b496a56870c213ba0e1ca` and `src/state/user-preferences.ts@a20e4e1ebc2d60624993f75918858751f244dc3b` were already registered elsewhere in this dossier and are not duplicated. Their existing read/mutation fences wait through native settlement and reject changed receipts rather than becoming another writer.

Worker-environment publication:
- `src/gateway/worker-environments/placement-row-codec.ts@1e0fd74a3f11d3ace5223c2bbfdaad7b2224dfa2` directly publishes native placement row changes through the existing worker-environment projection owner.
- `src/gateway/worker-environments/placement-lifecycle-store.ts@d4caf456224d13334fd6446d61136ebbb48518c8` reserves publication order under the physical mutation owner, installs the placement lifecycle receipt, then publishes the matching environment patch.
- `src/gateway/worker-environments/placement-turn-claims-store.ts@22c12e10bf59c265cf73b3a17a2ff54df8cceae6` reserves the same publication owner for turn/workspace-result mutations; an unknown durable outcome invalidates the prepared publication and requires recovery rather than replaying a mutation.
- `src/infra/device-pairing-worker.ts@190f6301ff4a865992d8bd1c5f3ddf4aff912ca5` was already registered elsewhere; exact re-read confirms its pairing receipt is installed before fallible callbacks and unknown outcomes return to discovery without reviving authority.
- Focused oracle `src/gateway/worker-environments/store-projection.test.ts@6259670da93d0807db4772f633830b23c3df5c7c` was already registered; it checks rollback discard, ordered post-commit publication, delayed full receipts, and unknown/unsettled authority.

Runtime auth-profile snapshots:
- `src/agents/auth-profiles/store.ts@e722a7cf907e7c4b266bf33dd25449fe1770b118` was already registered and remains the only direct consumer of `publishRuntimeSnapshotsAfterCommit` / `deferRuntimeSnapshotsAfterCommit`. Exact re-read confirms a committed write cannot be rolled back by snapshot publication failure; only the exact derived runtime owner is evicted or refreshed.

SQLite write-settlement observation:
- `src/infra/sqlite-wal.ts@a902d75642928bc57db708c445ea43b6fd6fae1c` is the newly frozen production consumer of `onSqliteDatabaseWriteSettled`; PASSIVE periodic WAL checkpoint scheduling arms only after the shared write receipt is published. The listener is maintenance scheduling, not mutation authority.

### 48.2 GitHub, turn-claim, and conversation-binding downstream observers

GitHub connection/publication:
- `src/gateway/github-personal-oauth.ts@2f55eeadf2e4254f72f7522f4394140ed1103aaf` consumes user-GitHub connection authority and profile-retirement publications. Connection changes invalidate in-flight scans; retirement queues profile cleanup. Neither notification path writes connection authority.
- `src/gateway/github-publication-store.worker.ts@b4f4c4869b19110960ba927b35cde476a7b735bc` stages shared publication receipts and defers `deferSharedGitHubPublicationChanged` inside the existing transaction.
- `src/gateway/github-repository-publication-store.worker.ts@4a8e54ce9027baaa4f2c4c31f5c8db03c8f86b78` does the same for repository publication rows.
- Focused `src/gateway/server-methods/users-github.test.ts@694fa042b95ec846ffd66aa3ab897ee131da6716` observes authority-before-retirement ordering and disconnect cleanup under the native lease.
- `src/state/user-github-connections.ts@d67e92e4592045b6e3b5db97ca5adf969b19e544` and `src/gateway/github-publication-source.ts@32b16c77cbe306da84fa1aa3d47a4266a7b1ab54` were already registered and are not duplicated.

Worker turn-claim closure:
- `src/gateway/server-aux-handlers.ts@e5e1ad41cc224917ce366923e26a31a8e47eff46` registers the placement turn-claim close observer and converts closure into approval-authority settlement/publication, then unregisters it on retirement or failed startup.
- `src/gateway/server-core-runtime.ts@4a2c6b2e5cb895688107343eaf50ca0a2d32dd39` wires the live placement-store registration into the auxiliary handlers.
- `src/gateway/worker-environments/placement-store.ts@4666e152cc68e44e688370ffbad1d4340b53d903` is the direct path adapter for `registerWorkerTurnClaimClosedHandler`.
- Focused `src/gateway/server-aux-handlers.authority.test.ts@9da039058c65854cfde720887ca29e7c471b5bda` exercises the production worker-claim observer and verifies both exec and plugin approvals settle/publish on closure.

Current-conversation binding receipts:
- `src/infra/outbound/prepared-session-binding.ts@0dac5c3432e5eae303de8bb6f6ae87cf9f151a22` subscribes binding facts and waits for native finish to drain committed receipts before declaring a prepared selection unchanged.
- `src/infra/outbound/current-conversation-bindings.ts@495115111612e693cb136ac2aaa24fafc525c7bb` consumes committed facts into retained binding rows; a receipt supersedes an in-flight older read.
- `src/infra/outbound/current-conversation-bindings.kernel.ts@ca378b7b74301c2b6ddab2f9e7058b558cd2028b` stages binding postimages/deletions at the owning synchronous mutation.
- `src/infra/outbound/account-scoped-conversation-bindings.ts@2b6ca244598ae95cc5abf770d45b0eb6c1aafee7` observes publication revision and rejects a selection if facts change during selection.
- `src/infra/outbound/current-conversation-bindings.worker.ts@29b7ec5b54524c1e69d766cd8dcd61d4b0678d20` wraps worker mutations in one binding receipt and commit admission.
- Focused oracles: `src/infra/outbound/current-conversation-bindings.queries.test.ts@620339c758f360a209eeeb10c097a07894dd4c66`; `src/infra/outbound/current-conversation-bindings.worker.test.ts@c2849c78a6ab4f6087c4bbe527ca792d8daff0a1`; `src/infra/outbound/prepared-session-binding.test.ts@ece07abdcb183844142ed6f61cb963c99b761e2f`. They cover worker bind/touch/expiry facts before replies, late receipts not restoring a newer deletion, unknown settlement, and prepared-selection settlement.

### 48.3 Concrete state-domain worker receipt consumers

The generic `state-domain-publication` primitive remains shared staging/notification mechanics. Newly frozen concrete domains and their existing-owner worker receipts are:

- exec approvals: `src/infra/exec-approvals-publication.ts@c9cf172b7656c990da0467d885f742c67d91fb4b`; `src/infra/exec-approvals-mutation.worker.ts@e405c9d955107156f45cd8c1d7b243012c469ae7`; `src/infra/exec-approvals-retirement.worker.ts@8e485084537d5a8dd0c6275733ca88772a4679e9`; `src/infra/exec-approvals-authorization.worker.ts@56b1c8c192f8b00e4540ad91fd3ef1be498db779`; `src/infra/exec-approvals-sqlite.ts@6177e0282c9d7489b10e54080e2f6c83eaaab182`. The workers capture the exact policy facts and defer the bounded receipt after the owning transaction.
- composed operator approval receipt: `src/gateway/operator-approval-store.worker.ts@76f15f1c6d6e35d36056349d82616856397aa501` captures approval, standing-grant, and exec facts into one worker commit receipt. Focused consumers `src/infra/exec-approvals-store.test.ts@be3ebf30b58a0e0c5144a0889af7bb99c58b1ccf` and `src/infra/exec-approvals-store.read-worker.test.ts@b2dfbdd749337c36989d784496ce0655279a15c3` verify committed publication and foreign-commit/read-worker behavior.
- agent provenance/deletion: `src/state/agent-provenance-publication.ts@e60273c9db53b81d7c23e8d31bd5903c2c230ce1`; `src/state/agent-provenance.ts@491886f799d6a8f5e29f8bf45572f2a246004f11`; `src/state/agent-provenance.kernel.ts@e49c89139983f8d9d4920e70d2ad96f8a4d9bf68`; `src/state/agent-provenance.worker.ts@95c5d1476b43abb8eccbac4c75145239b220b05c`; `src/state/agent-deletion-journal-publication.ts@4da4b096dc3653173758af07c1422da1c67cef82`; `src/state/agent-deletion.worker.ts@d1d4882592fe273256d052a84590e2a3da911860`; `src/state/agent-deletion-journal.ts@0ef1cde953c03c9c0eb783de80977f31b71b0591`; `src/agents/agent-lifecycle-binding.ts@6f9e9b6a0a953c661446173c0428888b3a1b98c3`; `src/agents/agent-lifecycle-registry.ts@03eaef33ea1a66ab5c880886948861684285d953`. Lifecycle projections subscribe to committed provenance/journal facts; committed or unknown receipts supersede stale in-flight reads rather than authorizing a replacement write.
- workspace state: `src/agents/workspace-state-publication.ts@fbb6eafdfcdaaa3405d5939cd097c705ac8d79db`; `src/agents/workspace-state-cache.ts@04dfc7f627a94a9241db9f4552f0e0d5541c5eb7`; `src/agents/workspace-state-store.kernel.ts@c29a1d0412285622808586946598ae4d2d7bb834`; focused `src/agents/workspace-state-store.test.ts@1169951d6eafac94f55bcea044b5b6c0c0af433d`. Unknown or alias facts invalidate the cache; the physical database owner remains the writer.
- operator approvals/standing grants: `src/gateway/operator-approval-store.publication.ts@143fbec20655870487a8081b554529f5e80f88cf`; `src/gateway/operator-approval-store.rows.worker.ts@36e60b78b1c45e8ba272cf587450778f4215c70f`; `src/gateway/operator-approval-store.transitions.worker.ts@27291c50008127fa7fec9837fbe0caed10fd0da8`; `src/gateway/operator-approval-store.kernel.worker.ts@d9c305bd9086250f17dcc00697e630a05d08bf75`; `src/gateway/operator-approval-standing-grants.worker.ts@21dee4e00f62a0b403ca2e5108a45eee94977782`; `src/cron/service/config-mutation.ts@47252b7d1a6022dcfcc56de9140fa1813c014428`; `src/cron/store/guarded-mutation.worker.ts@d08a3ed68e35a5966581bb8d129e29c4889f8af2`; `src/cron/store/row-codec.ts@bdf2c8d573141bae3628fbc65632d8af3014df73`; focused `src/gateway/operator-approval-store.worker.test.ts@8b77e33dc56dbd4861bab7b1d50a21e2546ff694`.
- repository workspace: `src/state/session-repository-workspaces.receipts.ts@5c2f53d9a9dd644dea2b9bf8be936a5a102309cd`; `src/state/session-repository-workspaces.worker.ts@d4410ce30091c28c43aa706b6fe21f3f4a045120`; `src/state/session-repository-workspaces.publication.ts@0633fe30c366782cf58a0520f1954270e6bd4e96`; `src/state/session-repository-workspaces.kernel.ts@522f298b8cb66a8af333ad8d3a198b44f4551be1`; `src/gateway/github-publication-shared-read.ts@9ba3e38edaa0815b59adcfefe671a587da1fa366`. The shared GitHub reader clears pending observations when any of its three committed source domains changes.
- node prepared workspace: `src/node-host/node-worker-prepared-workspace-publication.ts@d5e7c55dfabd7c679302001b25663a066e5f45fe`; `src/node-host/node-worker-prepared-workspace-store.kernel.ts@c4c0830044d813e94adbba3a5bff889da9b5fd2b`; focused `src/node-host/node-worker-prepared-workspace-publication.test.ts@6e4c57b0f338187bb14e36fda7227e803db4f9bc`. The already-registered journal worker consumes its bounded receipt and fences unknown settlement.

These domains reuse their existing transaction/worker owner. Publication listeners install or invalidate projections, cache, and authority views only; they are not a second durable writer.

### 48.4 Prepared-write and Goal direct-consumer closure

Prepared write:
- `runPreparedSqliteSessionWrite` has exactly two frozen direct production callers: `src/config/sessions/session-accessor.sqlite-projection.ts@b062842cf60a679400157366a0cebf43a2333dc1` and `src/config/sessions/session-accessor.sqlite-replacement-projection.ts@244cf8bb61598443f18042d770dfe5a4a433c64f`. Both were already registered elsewhere, so no duplicate identity is added. Both revalidate source/currentness at commit and leave durable mutation with the existing session writer.

Goal:
- `src/gateway/server-methods/sessions-goal.ts@2e1a83a2cce09f869df04df8da92359aba35696b` is the frozen Gateway update/clear production entry. It captures external commit guards, calls `mutateSessionGoal`, publishes only the committed projection, and reuses `operationId` as the continuation idempotency key.
- `src/config/sessions/session-turn.kernel.ts@0e950e71ec85f512aeb1120d8e53ee055cc49f98` reads and writes the Goal operation receipt in the same transaction as Goal/turn/lifecycle state.
- `src/config/sessions/goals-management.ts@14bfaf050ce3a02bb10a9ea68bb8b494083bb8b8` is the worker/source-authority adapter and publishes only the admitted worker candidate.
- focused: `src/infra/database-engine/goal-receipts.conformance.test.ts@ae98434e0dd8aaa4c9434fe87cc4389e48e0fbde`; `src/config/sessions/session-actor-memory-continuations.test.ts@4328d55dc9584e412971a5b00f76465614a4527e`; `src/gateway/server-methods/sessions-goal.event-custody.test.ts@21e87e497cb1331abbfbd6d329425a2f1462f04f`. They cover production-worker receipt persistence/replay, newer-edit preservation/conflict, and commit-before-event/projection custody.
- moving-main candidates `src/gateway/server-methods/sessions-goal-replace.ts` and `src/gateway/server.chat-goal-replacement.test-support.ts` return 404 at the frozen ref and are excluded.

### 48.5 Transcript rewrite and import direct-consumer closure

Transcript rewrite:
- The low-level frozen adapter remains `src/config/sessions/session-message-rewrite.ts@b0f8a466053b518c142f0d25441df66c4602c6c3`, already registered. Newly frozen direct production consumers are `src/sessions/user-turn-transcript-updates.ts@6c8d8c5149b413c95bb942ea90f62bfd27b820ac`, `src/gateway/server-methods/chat-transcript-media-enrichment.ts@f3552acdd2a7bc5b4c4f634624a019ed401168ef`, `src/config/sessions/session-accessor.transcript.ts@644efade1b694631d24df408488658f211ce8368`, and `src/gateway/server-methods/chat-transcript-persistence.ts@254cc4b09c39cf9ee1b1a568bd9b60dd8436b876`.
- These callers carry exact anchors/generations and currentness into the worker; rewrites rotate or preserve generation according to the exact-row contract and do not let a stale prepared payload overwrite a later edit.
- focused: `src/sessions/user-turn-transcript.test.ts@0efdc7aef9ac8978bf986bdaa99b1ff5b56f957a`; `src/config/sessions/session-actor-memory-reports.test.ts@1993323f686e415d84b973e06c5ce4249e7de8e7`; `src/gateway/server-methods/chat-send-commentary-media.test.ts@248fffd1c381ac5c720e2c1ef649b9189c9c753b`. They cover commit-only prepared projection adoption, later-append preservation/competing edit refusal, and revocation/unknown-commit media rewrite behavior.

Import:
- `src/commands/doctor-session-sqlite-active.ts@6ff427c6b20e6251a9ec3bcdbb234bbc4cbe33db`, `src/infra/deferred-plugin-session-empty.ts@3e0faf38b932bc327b19ad9e45c34cdee8b5e68b`, and `src/commands/doctor-session-sqlite-import.ts@4eee7b8b249d9b4cd961449bd4216790b515512b` are the remaining frozen production entries to `importSqliteSessionRowsBatch`.
- focused: `src/config/sessions/session-accessor.sqlite-import.test-support.ts@4c26e497765f52814c01b93b9caa5b3b83ede999`; `src/config/sessions/session-accessor.sqlite-import.test.ts@8f0008110efba14b29f0d53a9db8db2e62700641`; `src/commands/doctor-session-sqlite.active-settlement.test.ts@052c1e9a2efc4b68984c04bcdca6ae5dd0225540`; `src/commands/doctor-session-sqlite.failure-reports.test.ts@0fce213dae2491a294b083560c0262701283a4ab`; `src/config/sessions/session-accessor.sqlite-maintenance-kick.test.ts@4ec849ba76d5378c3f297f54a63c3fe7986b56c3`. The exact import tests cover whole-batch rollback, source revalidation after later readers, cross-store refusal, retained originals, and maintenance interaction.

### 48.6 Message-cut, canonical-repair, and legacy-main closure

Message cut:
- `src/config/sessions/session-accessor.ts@10fa871d8e24855a2d9fc5eb2839c20b9b192941` is the frozen public re-export surface for fork/rewind/switch; it adds no writer.
- focused: `src/config/sessions/session-accessor.sqlite-message-cut.test.ts@85251a7bf90042bdad68eb7bba118f689aba8f11`; `src/config/sessions/session-accessor.sqlite-message-cut-native.test.ts@1c2083386d12c2cd323f9e3d9d58e45cb7528c3a`; `src/config/sessions/session-accessor.sqlite-branches.test.ts@b7ab581eed904a603e7474044ba0f7db5494040f`; `src/config/sessions/session-cold-storage.lifecycle.test.ts@aa1b07d18e43a044707f70aa84fca19190974ec7`. These exercise fork/rewind/switch against exact transcript/session state, branch races, and cold-storage lifecycle.

Canonical repair:
- `src/config/sessions/session-accessor.entry.ts@91210793d8aef900a77b7c57bbb3b5ff2eaef774` and `src/config/sessions/session-accessor.ts@10fa871d8e24855a2d9fc5eb2839c20b9b192941` expose the frozen repair helpers; they do not add authority.
- The actual production consumer remains the already-registered `src/commands/doctor-session-canonical-keys.ts@6b1afad456b3b633c358d19c56beec5a6103f4b1`: Doctor groups work by database, normalizes generations, rehomes delivery references, copies owned state, then removes losing aliases under the existing repair writer.
- focused new oracle `src/config/sessions/session-accessor.sqlite-cold-read.test.ts@7c95eac6cf74ae5aec895e0fcc230a739d3c5aa6` exercises cross-store owned-state copy while a cold-read/archive race is committed.

Legacy-main migration:
- `src/config/sessions/legacy-main-session-migration.ts@4b1bc10415dd0561bd68c09c5c6e95d16c8dd2ba` is the frozen wrapper around Section 47's `processIdenticalClaims` / `repairDivergentClaims`.
- newly frozen production callers: `src/commands/onboard-agent.ts@087ad1087e16cef0290fb9ed230c30e9d9a84b45`; `src/agents/agent-create.ts@7302b264648e5aef76d3599f7e5a6ba14835ecff`; `src/commands/doctor-session-transcripts.ts@5e4d7b444ed2d16f2fa4c4b0c902f05b5b84a525`; `src/infra/state-migrations.doctor.ts@2b2d8d3c9aa3f879c266d58b8e974139fd9e0ef3`. `src/config/sessions/startup-migration.ts@5dce338ea47b067f6f0447180c71a521b3143660` was already registered and is not duplicated.
- behavioral focused tests: `src/config/sessions/legacy-main-session-migration.retained.test.ts@8993ce5240e9c67dc19e32429cd602c616d232af`; `src/commands/onboard-agent.persistence.test.ts@c427ab0fb9f813fcd519d3ad4ff44f41c5e176dc`; `src/config/sessions/session-retirement-read.test.ts@aa1a28f9e4322ba6eb536536502d1d78f2eabb22`; `src/agents/agent-create.integration.test.ts@ace041f1e4207b60d57beca34ef1dce2be880a94`; `src/config/sessions/legacy-main-session-migration.test.ts@f6a8357061679a87305728f75d2b10e491493072`; `src/config/sessions/legacy-main-session-migration.input-handoff.test.ts@91215ad3c73205bdac99d2cab26a2177170dcfd4`; `src/config/sessions/legacy-main-session-migration.handoff.test.ts@e632b86741698c97532e9b9e5ad5e69bd9a034fe`.
- mock/support-only identities were read but are not promoted to production evidence: `src/commands/onboard-agent.test.ts@498a0b157f2dda3242814727626ae9d582a1e776`; `src/agents/agent-create.test.ts@3a78f8514b36c88fe81212471def6d14d2016456`; `src/commands/doctor-session-transcripts.sqlite.test.ts@d5b5a3e45691c79bfc80025f36671d58886aa5e0`; `src/agents/main-session-recovery/main-session-terminal-approval-migration.test-support.ts@a810f89621e477ff7921ddfcc5b953efa980d3be`; `src/commands/onboard-config-provenance.integration.test.ts@b38d86576b2909fb56f9acd5ba61e0581ebbae9a`.
- previously registered migration race/node-handoff/startup recovery tests remain de-duplicated.

### 48.7 Closure/result

The new downstream evidence keeps the same owner model as Section 47: a transaction/worker commits durable state; the bounded receipt or post-commit publication installs/invalidate derived facts; downstream observers may refresh caches, projections, OAuth/runtime custody, approval notifications, or maintenance scheduling, but do not acquire a second durable writer. Confirmed rollback discards staged publication. Unknown native outcomes invalidate or retain custody and force recovery/currentness rather than retrying a possibly committed mutation.

No GitHub Actions, build, test, generator, package, production source, parity ledger, RTM, or completion flag was modified or executed in this increment. This evidence still does not establish independent ACCEPT, scoped G01/G02 positive proof, Rust implementation/wiring, executable/package proof, OCRM final acceptance, or release acceptance.

