# OCRM SQLite physical identity, schema/read lifecycle, cache and error boundary — 2026-10-22

> **PARTIAL READ-ONLY DOSSIER ONLY.** No scoped G01/G02 acceptance, no independent ACCEPT, no OCRM Spec §11.1 production admission. No claims of full recursive dependency closure, complete Gateway compatibility, or an implemented Rust writer.

## Authority and exact P0 Actions evidence

- Target subject H: `bhrumom/fabushi-desktop@0126248602f5f16e2a5a5aeeda0d4b88206b469f`, tree `bfd28d7191864901dbb1ec259607d49bf765120f`, parent `b93056fc52264986f6abcad9803ff31bb7edf0aa`; PR #169, branch `ocrm/p0-source-authority-20261022`, base branch `main`.
- GitHub Actions run `39768724706`, attempt 1, completed/success (2026-10-22 11:13:10 UTC); source-authority job `119790586117` and persist-verified-inventory job `119791125244`, both completed/success. Complete decoded logs inspected. Both actually `git checkout --force 0126248602f5f16e2a5a5aeeda0d4b88206b469f` in detached HEAD, **not a synthetic merge checkout**.
- Canonical tracked-source candidate = `58,750` paths; canonical inventory SHA-256 `586cadaa199916ac9593782b59059bfdcf8c72a717e534a72225e7e883977153`; committed 58 shard reconstruction PASS. Structural G02 negative probes rejected duplicate source path, falsified unread denominator, unbacked accepted verdict and orphan responsibility. This is NOT semantic G02, scoped permission/writer acceptance or LFS/external dependency closure.
- Immutable Actions artifact ID `12421237005`, name `ocrm-p0-authority-0126248602f5f16e2a5a5aeeda0d4b88206b469f`, size `4,479,293` bytes, digest `sha256:9900fe74eacdf3c3d9480f767f1915f092962882c15a18ac5b9f09b11d2ffbe5`. Upload log, GitHub artifact metadata and downstream Actions download digest match; additionally retrieved artifact via GitHub connector. No executable verification was run outside Actions.
- Frozen source **unchanged**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. Observed live upstream `main@698a6f83b46beae08b9e63dc06138c5fccfb66b1`, ahead 121 commits by GitHub compare; compare returned 300 paths at its response cap, **not** a complete change-set. No rebaseline or denominator change. Observed target live main `7394f6dc7687f9b986ee5589d40aa680ce23b90a`, tree `fbeb0bb32d944d0c1053880c471fe5c52279f052`, different from PR's recorded base `f6eb1758c59d65aba93e5608d7aba445e0bad42b`; no automatic rebase or acceptance inheritance.

## Full-text frozen blob reads performed in this continuation

All rows identify the actual frozen-ref Git blob and inclusive line interval; for text, the final empty line (if any) comes from a trailing newline. These reads do **not** close recursive imports or external dependency sources.

| Frozen source path | Git blob SHA | Full inspected range |
| --- | --- | --- |
| `src/infra/sqlite-worker-identity.ts` | `9d6fef1eea5eec6901ce9a454e7ab6a7001d7d2d` | 1–243 |
| `src/infra/sqlite-schema-facts.ts` | `6125196b7a8e256066d251b2557cf54716fa3219` | 1–737 |
| `src/infra/sqlite-reader-lifecycle.ts` | `ff40ee42e0b58f04aee9c24ef7e69e8ebb013112` | 1–270 |
| `src/infra/kysely-sync-cache-state.ts` | `29f87af9b2d2837db28cb2e1f26c6e8428de3108` | 1–272 |
| `src/infra/sqlite-error-diagnostics.ts` | `5972bcdbf6d72065b26bd96db90933aa762753b7` | 1–235 |
| `src/infra/errno.ts` | `606390f2baca5010799b2e4ec5eaefe7d60a61d0` | 1–19 |
| `src/infra/path-guards.ts` | `ba21e274745fb36452650dd6e709a5d7c2a29bd7` | 1–51 |
| `src/shared/global-singleton.ts` | `92c598b9c0e835ac80c39b48c55f97a062e6929c` | 1–124 |
| `src/shared/listeners.ts` | `498c0e5816d55286af9cb326b8c6446304b60863` | 1–38 |
| `src/infra/sql-connection.ts` | `7e32009a4bb407f5641ce09bf1d408f145c65429` | 1–63 |
| `src/infra/sqlite-schema-scope.ts` | `2515f5433a5c831f02d80fe389742432745383bd` | 1–107 |
| `src/infra/sqlite-schema-lifetime.ts` | `b4dd4eaae8b77832070f755c33bc6c0e332d9461` | 1–54 |
| `src/infra/sqlite-pinned-read-snapshot.ts` | `1473068dd221c3454b3d3ac59ad01c887017655c` | 1–43 |
| `src/infra/sqlite-schema-admission.ts` | `7f42a547d19b63e461abd411ed9bc770227235ad` | 1–149 |
| `src/infra/sqlite-schema-mutation.ts` | `d4c7c2b2429ff63a6b42b622f601e0ab9377882e` | 1–197 |
| `src/infra/sqlite-schema-sql.ts` | `4af1f61125b13527ae5c0c953efcaff24bbf002f` | 1–229 |
| `src/infra/sqlite-native-observer.ts` | `497406e1e356d9f624c64abe3be73dc244e48780` | 1–309 |
| `src/infra/sqlite-temp-generation-schema.ts` | `7b727f8837a5b81ff57218e1abe99e1f9cea11ab` | 1–149 |
| `src/infra/database-engine/physical-facts.ts` | `defc74cffa6d36ecf46e98d250cbaf164d7c6fc9` | 1–121 |
| `packages/normalization-core/src/error-coercion.ts` | `8e53d4bc60b192f8fa3a67db29a5b14fd1f8eb5b` | 1–379 |
| `packages/normalization-core/src/record-coerce.ts` | `ec7988eabf919a0e2f594854b9c33341ab4bed29` | 1–74 |
| `packages/normalization-core/src/promise-like.ts` | `a3e79119633e52022eaef35f61ca27567ccad02b` | 1–14 |
| `src/infra/node-sqlite.ts` | `7775b8c32b06bb76ad1ff844f2f132127c7a5193` | 1–127 |
| `src/infra/kysely-sync.ts` | `ed72c6bd8ad4e6d35fd3be19b48e79a2cfc3ff45` | 1–358 |
| `src/infra/sqlite-worker-identity.test.ts` | `4842787872950679beb6abe25f987f3da805cf97` | 1–125 |
| `src/state/openclaw-schema-metadata.test.ts` | `7a114daf10b29521a7d960060f211663f2df83cc` | 1–174 |
| `src/infra/sqlite-wal-checkpoint.test.ts` | `0f6bf6f537991ec8d601d303ac045d54702a9365` | 1–313 |

Previously completed deep reads of `sqlite-transaction.ts`, `sqlite-database-admission.ts`, StateDB, post-commit, Gateway pairing store, server methods and related handlers are recorded with their own blob SHA/ranges in `gateway-sqlite-admission-registry-continuation-20261022.md` and preceding dossiers. This note does not silently upgrade any partially read inherited file or unenumerated dependency.

## Source behavior, source-to-target contract, ownership implications

1. **Physical SQLite identity is not authority** (`sqlite-worker-identity.ts`). Existing regular files bind `dev:ino` and (except Linux/Android) birthtime; Linux/Android deliberately pin birthtime to `0` rather than treating ctime as creation time. Missing path keys resolve the nearest existing realpath ancestor, normalizing aliases without rewriting the persisted locator. File replacement, non-file target and races around canonical realpath must deny stale open. Linux sole-descriptor examination is only a conservative observation, not durable permission or cross-process writer ownership.
2. **Schema facts require tracked admission and native observation** (`node-sqlite.ts`, `sqlite-schema-facts.ts`, `sqlite-schema-admission.ts`). `trackSqliteSchema` runs before statement-cache authorizer wrapping. Cold admission validates the version within a pinned native snapshot and derives schema objects/encoding. Main DDL invalidates siblings; TEMP DDL and isolated TEMP tables remain connection-local. Rollback must discard transaction-local admissions. Historical pinned snapshots/transactions may retain local catalog proof but must not republish old facts as current committed ownership. Database admission/write revision and schema revision are distinct.
3. **Read revision is a scoped cache receipt, not business authorization** (`sqlite-schema-scope.ts`, `sqlite-schema-lifetime.ts`). Read depth, mutation depth, transaction snapshot, authorizer and unmanaged iterators decide reusability. `getSqliteReadOperationRevision` excludes open SQLite transactions/pins; PostgreSQL path requires matched namespace/store/admission ID, mutation and write revisions. A previously valid schema, query snapshot or cache key is never a substitute for live pairing credential/scope checking at the writer.
4. **Native callback and iterator lifecycle requires exact settlement** (`sqlite-native-observer.ts`, `kysely-sync.ts`). The observer classifies SQL batches outside quoted/comment spans, tracks operation-depth and step/return/reset/close separately, and marks abandoned iterator finalization without claiming COMMIT. Kysely holds private statements for lazy iteration and explicitly releases `retainSqliteReader` leases on success/error/return. Reader diagnostics are *current-thread only*, bounded to eight displayed readers/connections, show `blockingOwner: unknown` and `nativeStatements: unobserved`; they cannot identify a real WAL lock owner or grant custody.
5. **Statement cache has an explicit upper bound and revocation** (`kysely-sync-cache-state.ts`). At most 64 retained SQL entries at 64 KiB each after second sighting; oversized SQL/bindings bypass, candidates limited to 64, reentrant active SQL prepares another statement. `setAuthorizer` transition (including removing a policy), database `deserialize`, replacement and close invalidate/dispose dependent state. Failed disposal cannot be misreported as successful resource settlement; native handle lifetime and proof of authorization remain owned elsewhere.
6. **Storage errors preserve execution/provenance distinctions** (`sqlite-error-diagnostics.ts`, `normalization-core`). Native/typed SQLITE_BUSY(5), LOCKED(6), READONLY(8), IOERR(10/14), FULL(13) are classified before exact legacy messages; EIO, ENOSPC and CANTOPEN map to storage failures. CORRUPT(11)/NOTADB(26) are positive damage categories, not the same as transient lock/busy. Snapshot/native-open failure scopes preserve original cause and operation tagging; diagnostics do not change permissions or establish whether a mutation committed. When COMMIT may have completed and reply was lost, retain `outcome-unknown` and reconcile rather than retry mutation.
7. **Fabushi has no demonstrated OpenClaw pairing writer**. Target PR head still has `source/host/src/host_gateway_api.rs@e93659285baf4903d3c97cf85629864341ad4126` (256 lines), `source/host/src/gateway_server.rs@1ce506ae0bc75ee96418e55eb64a935dd7865ac0` (1,244), `frontend/src/production/ProductionRenderer.tsx@af8a83a8dd7cf1444474889ce3e64fe796a7d458` (4,457), `frontend/src/production/coordinator-client.ts@f120b9f600861560409d9a9cac6487cb5ec79e63` (194), `source/node-agent-coordinator/src/renderer_port_server.rs@ede3e482d6d29847fb56ac1d0f2d77524331f55e` (191), CLI device agent `@10029b40dcccd21c920b760b5deb234b93af90e1`, and official device registry `@461ec329caf850c5b494db38c62a2ed88ef5f140` as in previous dossiers. Renderer → Coordinator transport → Host API/dispatch is partial reachability, NOT a proven `device.pair.*`/`device.token.*` Rust handler-to-durable-row chain. Official device `wss://.../agent` and the Mahayana stdio gateway are distinct authorities. Host pairing-method registry/permissions, Companion and full caller closure remain unresolved. No second writer is authorized.

## New falsifiable scoped oracles (NOT RUN, NOT ACCEPTED)

- **OCRM-GW-SQL-O71:** swapping symlink/regular file between path admission and native open, pathname rename, non-file or inode/birthtime mismatch rejects old physical handle; Linux/Android ctime writes alone do not falsely revoke while replacement still does.
- **O72:** missing path and its canonical parent aliases share an admission slot; observed absence cannot adopt unexpected replacement; no path or Linux sole-descriptor heuristic confers durable write permission.
- **O73:** schema admission is cold/tracked and validates user_version before potentially misleading catalog errors, with physical version/integrity and migration preflight instead of raw cache authority.
- **O74:** DDL, schema_version/user_version change, trigger-side writes, TEMP isolation, outer rollback and partial native failures invalidate only the correct local/shared facts and never publish a rolled-back schema.
- **O75:** sibling DDL and write revision changes invalidate current readers; pinned historical SQLite/PostgreSQL snapshots cannot overwrite newer committed writer facts; new reads after snapshot end observe new owner.
- **O76:** changes to authorizer/role or removal of authorizer evict native cached prepared statements; denied reads remain denied even if statement was previously compiled; deserialize/close must retire dependents.
- **O77:** 64-entry/64-KiB cache limit and second-use admission hold under reentrant callback, buffer bindings and repeated SQL; active outer statement cannot be reset or resurrected after eviction.
- **O78:** native iterator next/return/reset/GC/close and `retainSqliteReader.release()` leave no phantom active cursor; forgetting weak diagnostic metadata does not settle an explicit lease.
- **O79:** WAL busy diagnostic describes only local observations; bounded eight-entry sampling neither names a foreign lock owner nor asserts no readers merely because `activeReaders` is empty.
- **O80:** BUSY/LOCKED, READONLY/IOERR/FULL, CORRUPT/NOTADB and nested/cyclic error causes keep typed classification and original failure; native-open vs inspection/probe failure is not silently conflated.
- **O81:** committed pairing row/token/role/scope/generation always checked at the true Host writer at execution, despite valid schema/file/cache/reader facts; stale token/role and RPC retry cannot bypass revocation.
- **O82:** simultaneous Renderer, CLI, Gateway RPC, companion/device path and Coordinator routing converge on one durable, auditable Rust pairing writer; official Fabushi device enrollment does not mutate separate OpenClaw pairing truth.
- **O83:** COMMIT succeeded but response was lost, canceled reader, failed close or delayed post-COMMIT publication produce explicit outcome reconciliation and resource cleanup, never unguarded mutation replay.

## Open dependency, consumer and review frontier

- Recursive import/test/codegen closure **open**: `sqlite-native-runtime.ts`, `bun-sqlite-library.ts`, `sqlite-wal*`, `sqlite-maintenance*`, full `sql-query-compiler.ts`, database-engine `contract.ts`, `postgres-{catalog,schema}.ts`, remaining native-runtime/worker admission/receipt source, dynamic gateway handlers, test helpers, Kysely runtime, plugin/package/LFS manifests, and actual platform/companion registry. The upstream `packages` tree contains no `fs-safe` directory: frozen root package.json lists external `@openclaw/fs-safe` version `0.26.8`; obtain its exact published source/lock and licensing before claiming the path/errno helper closure.
- User-specified five files and the named supporting files above were read in full; globally effective tracked inventory still reports `unread=58,750`, `unclassified=58,750`, `source_closure_ready=false`. Do not edit file statuses or infer global read completion from this separate research note.
- Existing-owner-first pairing state election and actual producer → owner → storage → consumer verification still lack full Host/Coordinator/Renderer/UI/CLI/companion proof. No independent reviewer tool was available for this session; no independent ACCEPT invented. Scoped G01/G02 must be implemented/verified on an exact subject through GitHub Actions before OCRM Spec §11.1 permits production Rust edits.
- **No production Rust, schema, workflow, shared contract, ledger, RTM, acceptance flag, or other project's evidence was changed by this dossier.** This dossier's commit H2 is a *new* subject: H's two green jobs and immutable artifact are historical subject-H evidence only; H2 needs independent exact-head Actions proof.

