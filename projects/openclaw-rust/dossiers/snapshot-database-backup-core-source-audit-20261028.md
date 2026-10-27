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

## Responsibility/evidence decision

`OCRM-RESP-SNAPSHOT-BACKUP-CODEC-001` remains `read=partial`,
`implementation=implemented`, `wiring=not-wired`, and `verification=not-run`.
The local envelope/admission tests are local evidence only and do not upgrade the
responsibility or any global completion flag. Required next work is a spec-led
Rust design for the missing database-port/restore boundary, followed by a
real producer→owner→storage consumer path and falsifiable exact/portable,
tamper, generation-change, namespace-custody, and artifact-admission tests.

