# OCRM-P0-STORAGE-001 — Storage, Locations & Persistence Core Parity

Status: **Scoped unit admitted into parity-ledger (7 responsibilities mapped, 10 source files read-complete); Rust implementation and wiring in folder_id.rs/store_db.rs/sqlite_recovery.rs/agent_paths.rs verified by contract tests; acceptance pending**  
Unit: `OCRM-UNIT-STORAGE-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R15`, `OCRM-AC19`  
Dossier: `projects/openclaw-rust/dossiers/storage-persistence-scoped.md`  
Pinned source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Storage access enters through canonical Fabushi storage paths → validates portable keys and root boundaries without directory traversal → verifies initialization marker and cryptographic authenticity → executes durable atomic mutations via temp files and directory synchronization → exposes consistent object streams.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-STORAGE-KEY-VALIDATION-001` .. `OCRM-RESP-STORAGE-STREAMING-ENCRYPTION-001`). 10 source files mapped to reviewed status in Git shards. Rust implementation verified in `source/host/tests/openclaw_storage_diagnostics_contract.rs`. Full project closure and release acceptance remain false until all 58,750 files are mapped and fully accepted.
