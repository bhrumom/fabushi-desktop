# Snapshot database backup core — exact upstream source audit

Date: 2026-10-28 (Asia/Shanghai)

## Authority

- Repository: `openclaw/openclaw`
- Commit: `ad8d12f126d29b0db204187adf5092db80294e70`
- Tree: `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
- Source: `src/snapshot/database-backup-core.worker.ts`
- Blob: `f2a914c817028df98d38499abea2e50bcd44578f`
- Size: 8,973 bytes, 229 lines

## Read result

The file is a stateful PostgreSQL-to-private-SQLite export and restore worker,
not only an envelope/checksum codec. `exportPostgresBackup` creates an exclusive
0600 private artifact, opens it with BigInt support, reads source allocations,
materializes the database through the database-port owner, optionally applies a
Git backup row transform, and re-checks the caller's generation guard before
publication. Exact and portable profiles have different semantics: exact mode
renames the raw image, while portable mode creates a verified SQLite snapshot,
sanitizes global state, embeds cold session archives, optionally records redacted
table inspection, and removes the raw staging file.

The manifest records schema/profile/source anchor/active fragments/verification,
checksummed database and anchor artifacts, and sanitization differences. Restore
requires matching descriptor versions, allocates a random `restore_<32 hex>`
namespace, materializes SQLite into PostgreSQL, compares exact verification when
requested, copies and verifies the anchor without overwrite, updates its schema,
and drops only the namespace allocated by this restore owner on failure. The
fingerprint helper uses a repeatable-read, read-only transaction and always rolls
back.

Related exact-head source reads sharpen the remaining contract:

- `src/snapshot/export-envelope.ts` (blob `05b2a8746ced9c7a7fd73590f6b2ac743e308fbc`, 137 lines) defines strict schema-version-2
  Postgres envelopes, identifier/digest bounds, profile/sanitization invariants,
  exclusive 0600 manifest writes, hardlink/symlink-safe reads, held-handle
  anchor inspection, artifact hash/size verification, and mutation-fingerprint
  checks.
- `src/snapshot/postgres-generation.ts` (blob `74ab0aad636829ab16248dafd67d07ef22a2e732`, 166 lines) locks source relations,
  validates owned anchor metadata, fingerprints catalog/routines/table rows and
  sequence allocations without lossy numeric conversion, and exposes a
  deterministic SHA-256 generation identity.
- `src/snapshot/git-backup-codec.ts` (blob `c5cee3153c24d0f7ac1ba6634a67bfd8b05dc789`, 735 lines) adds deterministic SQLite
  schema/table serialization, bounded row batches, safe table identities, secret
  redaction and restore validation; this is substantially broader than a
  manifest envelope.

## Fabushi owner comparison

The current Rust owner is
`source/host/src/extensions/snapshot/git_backup_manager.rs`. It provides
`SnapshotExportEnvelope`, `GitBackupMetadata`, `create_export_envelope`, and
`verify_export_envelope`; it hashes a JSON manifest and rejects a mismatched
version/checksum. This change also adds a narrow, independently testable
`capture_database_artifact`/`assert_database_artifact` admission boundary
(regular-file, Unix `O_NOFOLLOW`/single-link checks, byte hash and size) and a
monotonic `SnapshotGenerationFence`. The focused contract now proves regular
artifact admission, tamper rejection, symlink/hardlink rejection on Unix, and
stale-generation rejection. The local test is
`source/host/tests/openclaw_snapshot_skills_contract.rs::test_ocrm_snapshot_artifact_admission_and_generation_fence`.
There is still no demonstrated PostgreSQL/SQLite port materialization,
schema-v2 envelope parser/writer, held-handle anchor verification, mutation
fingerprint, exact/portable profile split, session-archive embedding, restore
namespace custody, or repeatable-read fingerprint path in this owner. Therefore
the existing owner is a valid placement candidate but is not behaviorally
equivalent to the exact upstream worker.

The current branch now adds format-only Rust validation through
`parse_postgres_export_manifest`: strict serde fields, schema/profile checks,
RFC3339 timestamp, UUID/source identity, safe identifiers, lowercase SHA-256,
positive artifact sizes, sequence allocation bounds, and the exact-profile
sanitization invariant. Its negative/positive oracle is
`test_ocrm_snapshot_postgres_manifest_schema_and_profile_oracles`. This parser
does not read files, inspect anchors, or materialize PostgreSQL/SQLite state.

It also adds SQLite restore-target custody checks matching the exact upstream
`restore-paths.ts` blob `ceb029c57e960c61830de9a6e8269e5579934b60`: base target
and `-wal`/`-shm`/`-journal` sidecars must be absent before restore, and any
existing sidecar is rejected. The focused artifact test exercises the positive
fresh-target and negative-sidecar paths.

The same owner now writes validated schema-v2 manifests with an exclusive
0600/create-new file and `sync_all`, rejecting duplicate publication. The local
manifest oracle reads the resulting bytes back and validates them again; this
still does not prove held-handle anchor inspection or database materialization.

`verify_postgres_export_artifacts` now binds a parsed manifest to its directory
name, required profile, two opened regular artifacts, exact SHA-256/size values,
and sidecar absence. The local oracle passes real temporary database/anchor
files and rejects a newly introduced anchor WAL sidecar; it intentionally does
not claim anchor identity inspection or restore publication.

`verify_postgres_anchor_identity` now checks the admitted SQLite anchor's
`openclaw_engine_anchor` row (`store_id`, `engine=postgres`, and namespace) and
re-captures the artifact after reading to reject mutation. The current Rust
implementation necessarily reopens the path read-only because no fd-backed
SQLite adapter exists yet; therefore this is a local identity oracle, not the
upstream held-FileHandle proof.

`new_restore_namespace` now allocates the upstream-shaped `restore_` plus a
128-bit cryptographically random suffix. The local oracle checks the shape and
independence of two allocations. This only supplies the namespace allocator;
database-owner reservation, materialization, and owned-namespace cleanup are
still absent and remain pending.

`read_postgres_export_manifest` now mirrors the upstream bounded read
admission: it holds an `O_NOFOLLOW` handle, rejects non-regular and hard-linked
files, caps input at 1 MiB, then parses the strict schema. The contract test
passes a real manifest and rejects symlink and hard-link aliases. This is still
an owner-side admission boundary, not proof of database restore or consumer
wiring.

`admit_postgres_export` now composes the bounded manifest read, artifact
custody, anchor identity, and generation fence into one owner admission path.
The contract proves both admission and rejection after a generation advance.
It deliberately stops before PostgreSQL/SQLite materialization and namespace
cleanup, which remain the next production frontier.

`validate_restore_namespace` now mirrors the upstream cleanup guard and only
accepts `restore_` plus exactly 32 hexadecimal characters; the local oracle
rejects `public`, short names, and non-hex suffixes. It is a destructive
operation precondition only; no PostgreSQL DROP/CASCADE is exposed until the
database owner is wired.

`prepare_postgres_restore` now mirrors the upstream pre-materialization gate:
requested profile and family schema versions must match, the generation must
remain current, and a fresh guarded restore namespace is allocated. Positive
and mismatch oracles are local only; no rows are materialized and no namespace
is published to a live PostgreSQL owner.

The exact upstream `git-backup-sanitizer.ts` was also read at the locked tree.
The Rust owner now carries its table/key redaction policy: state versus agent
secret-table selection, protected `config_machine_state` prefixes, and
generated plugin-model catalog credential stripping. Positive and malformed
row oracles are local only; deterministic SQLite/Postgres row materialization
and the live database-port consumer remain unimplemented.

The exact `sqlite-cell-codec.ts` was also read. `encode_sqlite_value` now
matches its portable representation for NULL/text/finite REAL, safe versus
wide integer (`{"$int": ...}`), and BLOB (`{"$hex": ...}`), while rejecting
non-finite REAL values. These are cell-level oracles only; table ordering,
bounded streaming, and materialization are still pending.

`encode_sqlite_row` now builds deterministic JSONL rows from encoded cells,
rejecting duplicate column names and always appending one newline. The local
oracle asserts byte ordering and duplicate-column rejection. It is a row-level
primitive only; table schema extraction, primary-key ordering, bounded batch
flush, and restore consumer wiring remain pending.

`SqliteJsonlBatch` now bounds serialized row buffering at the upstream 1 MiB
threshold, flushes complete rows once reached, permits one oversized row as a
single unit, and retains a final tail batch. The local oracle covers threshold
flush, oversized-row flush, and empty finish. It does not establish table
iteration or end-to-end artifact publication.

`digest_sqlite_rows` now hashes the exact newline-delimited row bytes and
returns the manifest-compatible row count and SHA-256, rejecting empty or
non-terminated lines. The local oracle verifies count, digest shape, and
malformed-line rejection. It remains a producer-side primitive, not evidence
of live database scanning or restore publication.

`validate_sqlite_table_name` now matches the exact codec's filesystem-safe
table-name rule (ASCII letter/underscore first, then ASCII alphanumeric or
underscore). The local oracle rejects JSONL path names and SQL injection-shaped
identifiers before future schema/table publication.

`render_sqlite_schema` now sorts table/index/trigger definitions in the exact
upstream order, normalizes statement terminators, and appends the user_version
pragma marker. Positive byte-level and unsupported-kind negative oracles pass;
the live SQLite schema scanner and restore consumer are still absent.

`read_sqlite_schema_entries` now reads a real SQLite owner's `sqlite_master`
with the exact table/index/trigger filter and ordering, then feeds the
deterministic renderer. The contract creates a temporary table, index, and
trigger and verifies the returned order. This is SQLite-owner evidence only;
it is not PostgreSQL port inspection or restore publication.

`digest_sqlite_table` now reads a real owner SQLite table through `SELECT *`,
converts NULL/integer/REAL/text/blob cells, streams row encoding into SHA-256,
and returns the manifest-compatible count/digest without retaining the full
table. The contract compares it to an independently assembled JSONL digest and
rejects injection-shaped table names. PostgreSQL materialization remains absent.

A parity correction now makes `digest_sqlite_table` order rows by composite
primary-key positions from `PRAGMA table_info`, falling back to `rowid` when
no primary key exists. The real SQLite oracle inserts rows in reverse order yet
matches the independently assembled sorted digest; this closes an actual
insert-order nondeterminism in the SQLite producer path.

`select_sqlite_backup_tables` now mirrors the exact backup projection policy:
it excludes operational projection tables and virtual-table shadow names,
applies explicit exclusions, validates filesystem-safe names, and sorts the
result. The oracle proves selection against manual schema entries; actual
artifact staging and publication remain pending.

`read_sqlite_user_version` now reads and range-checks the real owner's
`PRAGMA user_version`; the temporary SQLite producer oracle sets version 42
and reads it back successfully. This supplies manifest metadata only and does
not prove schema compatibility or migration admission.

## Responsibility/evidence decision

`OCRM-RESP-SNAPSHOT-BACKUP-CODEC-001` remains `read=partial`,
`implementation=implemented`, `wiring=not-wired`, and `verification=not-run`.
The local envelope/admission tests are local evidence only and do not upgrade the
responsibility or any global completion flag. Required next work is a spec-led
Rust design for the missing database-port/restore boundary, followed by a
real producer→owner→storage consumer path and falsifiable exact/portable,
tamper, generation-change, namespace-custody, and artifact-admission tests.

`decode_sqlite_value` now provides the strict inverse for portable restore
cells: JSON NULL/text/number, bounded `$int`, validated hex `$hex`, and
rejection of booleans, arrays, unknown objects, malformed hex, and out-of-range
integers. Round-trip and malformed-input oracles pass; this does not yet insert
decoded rows into a staged SQLite owner.

`restore_sqlite_rows` now supplies a transactional SQLite consumer for decoded
rows: it validates table/column shape, decodes portable cells, inserts via a
prepared statement, hashes the exact restored JSONL bytes, and commits only
after all rows succeed. An independent in-memory database round-trips the
producer digest; row-arity failure is rejected. This is SQLite staging only;
PostgreSQL namespace materialization and publication remain pending.

The restore consumer oracle now injects a row-arity failure after a valid row
and verifies the target count is unchanged, proving transaction rollback rather
than partial commit. This is local SQLite atomicity evidence only; it does not
establish cross-database PostgreSQL cleanup or publication custody.

`parse_git_backup_manifest` now validates the upstream Git backup schema 1/2
manifest shape, global/agent/generic identity scope, optional Postgres source,
safe userVersion, exclusions, table names, row bounds, and SHA-256 entries.
Positive generic-scope and invalid identity/table oracles pass. It is manifest
admission only; filesystem ownership, staging publication, and restore schema
migration remain pending.

`read_git_backup_manifest` now holds a bounded unaliased file handle, rejects
symlink/hardlink/non-regular manifests, caps input at 1 MiB, and delegates to
the strict parser. Real-file, symlink, and hardlink oracles pass; Git repository
ownership/preflight and artifact publication remain outside this owner.

`git_backup_scope_path` now binds admitted backup identities to the upstream
filesystem scope convention: `global`, `agents/<agentId>`, or `stores/<id>`.
It rejects invalid/traversal-shaped identifiers before a caller can construct a
staging or publication path. Generic, agent, and traversal-negative oracles
pass. This closes identity-to-scope derivation only; Git repository ownership,
atomic publication, and cross-process custody remain unproven.

`git_backup_artifact_files` now mirrors the upstream artifact admission list:
schema-v1 yields `manifest.json`, `schema.sql`, and sorted table JSONL paths;
schema-v2 yields the PostgreSQL manifest, anchor, and database bundle. The
contract exercises both profiles. This is deterministic file-set derivation,
not yet Git repository preflight or atomic scope publication.

`admit_git_backup_artifacts` now compares a staged set against the exact
profile-derived artifact paths, rejecting missing/extra/duplicate paths and
absolute or parent-containing escapes. Positive, missing, and traversal
negative oracles pass. This is an owner-side admission boundary; it still does
not perform Git index/HEAD preflight or atomic publication.

The SQLite owner boundary was corrected before further expansion. Digest table
ordering now quotes primary-key identifiers by escaping embedded double quotes;
the contract covers a legal quoted primary-key name. Restore now receives the
admitted expected digest and checks row count/hash before transaction commit;
digest mismatch and row-shape failures leave the target unchanged. This closes
two local integrity bugs, while real production caller wiring remains absent.

## Fabushi production-owner audit

The existing production storage path is `StoreDbSnapshotUpload` under
`source/host/src/extensions/box_store_sync/store_db_snapshot_upload.rs`, started
by `HostProductionExtensions::start_box_store_sync`. It owns the real
producer-to-object-store-to-manifest consumer path and is covered by separate
BoxStore contracts. A repository-wide caller audit finds no production caller
of `digest_sqlite_table`, `restore_sqlite_rows`, `admit_postgres_export`, or
`prepare_postgres_restore`; current uses are confined to the snapshot contract
test. Therefore this OpenClaw codec remains an implemented native module but
is not yet fused into the existing production owner. PostgreSQL port material-
ization and the authorized API boundary are prerequisites for wiring rather
than grounds to fabricate a second storage owner.

The existing owner now exposes `StoreDbSnapshotUpload::upload_artifact_set`:
it preflights all relative artifact paths and duplicates, then routes each
validated file through the established BoxStore blob/manifest consumer. The
owner contract passes for a two-file set and rejects traversal without
mutating the manifest. The method is deliberately documented as non-atomic
because the current object-store API has no batch transaction; a future
OpenClaw caller must supply its own recovery policy and remains unimplemented.

The same owner batch boundary now preflights each source artifact with
`symlink_metadata`, requiring a regular file and (on Unix) a single link before
any upload begins. The production-owner contract rejects a source symlink and
leaves the manifest unchanged. This closes source-alias custody for the
adapter; it does not establish the missing OpenClaw caller or PostgreSQL port.

`export_sqlite_backup_artifacts` now provides the first real SQLite producer:
it reads a live SQLite connection, emits deterministic `schema.sql`, sorted
JSONL table artifacts, and a schema-v1 manifest whose row counts/digests are
derived from the emitted owner data. The contract re-reads the emitted
manifest and bytes and compares them to the live table digest. Publication to
the BoxStore owner is still a separate caller concern; PostgreSQL remains an
explicit database-port blocker.

The producer now performs all reads and row hashing inside one SQLite
transaction, writes into a unique fresh staging directory, syncs each artifact,
commits the read snapshot, and atomically renames staging into a previously
absent target. Any error removes staging; existing targets are rejected before
work begins. The negative contract proves an existing target is not overwritten.
The remaining publication gap is now explicit: the staged directory still
needs a production caller that maps its artifacts into `upload_artifact_set`.

`StoreDbSnapshotUpload::export_and_upload_sqlite_backup` now supplies that
owner-level path: it exports a live SQLite connection, maps manifest/schema/
JSONL artifacts into the existing BoxStore manifest namespace, uploads through
the existing blob consumer, and removes staging afterward. The owner contract
seeds a real SQLite table and verifies all three artifacts reach the fake
object store and the manifest, with staging removed. This is a reachable owner
API and end-to-end contract; the broader BoxStore production cycle still does
not invoke OpenClaw portable export by default, and PostgreSQL remains outside
the available Fabushi database-port boundary.

The BoxStore production cycle now has an explicit opt-in caller controlled by
`FABUSHI_OPENCLAW_PORTABLE_BACKUP=1|true`: each eligible agent `store.db` is
opened read-only, exported, and routed through the existing owner under the
`.../agents/<agent>/portable/` manifest namespace. A production-module test
exercises this path with the real local filesystem owner. The production cycle
resolves the process environment only at its boundary and passes the resulting
boolean through an explicit cycle option; tests do not mutate global process
environment state. The default-off cycle is tested independently, including a
live-handle periodic sweep that proves no portable entries are emitted. The adapter commits
the staged manifest only when every artifact is Uploaded/Unchanged; partial
object uploads return an error, leave the caller manifest unchanged, and clean
staging. External content-addressed blobs may remain as harmless orphaned
objects and require later garbage collection; no false atomicity claim is made.
The production cycle rebuilds the portable prefix from the current export set,
delays copying those entries into the live manifest until the final source
generation check passes, and preserves an existing portable prefix from stale
pruning when a capture/upload fails. A healthy retry therefore publishes a
complete new generation without adopting a mixed or partially uploaded set.

The remaining PostgreSQL boundary is a concrete dependency gap, not an
unverified implementation claim: `source/host/Cargo.toml` currently exposes
`rusqlite` but no PostgreSQL client/port crate, and the host source has no
database-port trait or production PostgreSQL owner. The existing
`BoxObjectStore`/`StoreDbSnapshotUpload` owner only publishes content-addressed
artifacts; it cannot materialize SQLite schema/rows into a PostgreSQL namespace.
解除条件为先引入经审计的 PostgreSQL port owner/API (including namespace
reservation, transaction/rollback, sequence allocation, and generation
revalidation), then wire `prepare_postgres_restore` to that owner and obtain a
real provider-backed integration receipt. Until then PostgreSQL
materialization, restore namespace custody, and exact/portable restore parity
remain pending/not-wired.

