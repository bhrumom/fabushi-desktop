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

## Fabushi owner comparison

The current Rust owner is
`source/host/src/extensions/snapshot/git_backup_manager.rs`. It provides
`SnapshotExportEnvelope`, `GitBackupMetadata`, `create_export_envelope`, and
`verify_export_envelope`; it hashes a JSON manifest and rejects a mismatched
version/checksum. The current contract test covers envelope tampering only:
`source/host/tests/openclaw_snapshot_skills_contract.rs::test_ocrm_snapshot_git_backup`.
There is no demonstrated PostgreSQL/SQLite port materialization, artifact
nlink/no-follow admission, generation guard, exact/portable profile split,
session-archive embedding, anchor verification/publication, restore namespace
custody, or repeatable-read fingerprint path in this owner. Therefore the
existing owner is a valid placement candidate but is not behaviorally equivalent
to the exact upstream worker.

## Responsibility/evidence decision

`OCRM-RESP-SNAPSHOT-BACKUP-CODEC-001` remains `read=partial`,
`implementation=implemented`, `wiring=not-wired`, and `verification=not-run`.
The local envelope test is local evidence only and does not upgrade the
responsibility or any global completion flag. Required next work is a spec-led
Rust design for the missing database-port/restore boundary, followed by a
real producer→owner→storage consumer path and falsifiable exact/portable,
tamper, generation-change, namespace-custody, and artifact-admission tests.

