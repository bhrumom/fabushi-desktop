# OCRM-P0-STORAGE-001 — Storage, Locations & Persistence Core Parity

Status: **Scoped unit admitted into parity-ledger (7 responsibilities mapped, 10 source files read-complete); source-level contract assertions are present for folder_id.rs/store_db.rs/sqlite_recovery.rs/agent_paths.rs, but no current exact-head Actions receipt proves implementation, production wiring, or acceptance; acceptance pending**
Unit: `OCRM-UNIT-STORAGE-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R15`, `OCRM-AC19`  
Dossier: `projects/openclaw-rust/dossiers/storage-persistence-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Current discovery subject: `projects/openclaw-rust/upstream.lock.json::live_candidate` (discovery-only; do not reuse this unit's historical inventory or provenance without rebaseline).
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Storage access enters through canonical Fabushi storage paths → validates portable keys and root boundaries without directory traversal → verifies initialization marker and cryptographic authenticity → executes durable atomic mutations via temp files and directory synchronization → exposes consistent object streams.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-STORAGE-KEY-VALIDATION-001` .. `OCRM-RESP-STORAGE-STREAMING-ENCRYPTION-001`). 10 source files mapped to reviewed status in Git shards. `source/host/tests/openclaw_storage_diagnostics_contract.rs` contains source-level contract assertions; this is not a current exact-head Actions receipt and does not by itself prove production wiring. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.
