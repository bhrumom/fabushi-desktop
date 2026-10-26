# OCRM-P0-SESSIONS-CONTROLLER-001 — Session Controller, Barriers, Transcript Persistence & Telemetry Authority

Status: **Scoped unit admitted into parity-ledger (6 responsibilities mapped, 179 source files read-complete); Rust implementation and wiring in agent_session.rs/session_maintenance.rs/agent_db_transcript_pages.rs/conversation_blobs_path.rs/session_recovery.rs/session_diagnostics.rs verified by contract tests; acceptance pending**  
Unit: `OCRM-UNIT-SESSIONS-EXT-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R08`, `OCRM-AC08`  
Dossier: `projects/openclaw-rust/dossiers/sessions-controller-scoped.md`  
Pinned source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Session controller orchestrates conversation execution custody → checks admission barriers and enforces concurrency limits → streams user turn prompts and media attachments to conversation blob store → commits paginated transcript pages into SQLite agent database → handles ungraceful process termination and startup turn recovery → emits structured session telemetry and token warnings.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-SESSION-CONTROLLER-LIFECYCLE-001` .. `OCRM-RESP-SESSION-DIAGNOSTICS-TELEMETRY-001`). 179 source files mapped to reviewed status in Git shards (`part-0046.jsonl.gz`). Rust implementation verified in `source/host/tests/openclaw_sessions_and_config_contract.rs` (`OCRM-SESSION-EXT-O01` .. `O06`). Full project closure and release acceptance remain false until all 58,750 files are mapped and fully accepted.
