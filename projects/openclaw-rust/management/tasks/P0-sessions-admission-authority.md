# OCRM-P0-SESSIONS-001 — Session Admission, Keys & Turn Correlation Authority

Status: **Scoped unit admitted into parity-ledger (6 responsibilities mapped, 35 source files read-complete); Rust implementation and wiring in session_paths.rs/agent_session.rs/session_conversation_state.rs/production.rs verified by contract tests; acceptance pending**  
Unit: `OCRM-UNIT-SESSIONS-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R08`, `OCRM-AC08`  
Dossier: `projects/openclaw-rust/dossiers/sessions-admission-scoped.md`  
Pinned source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Session requests enter through canonical Fabushi session API → classify session key kind and enforce harness namespace reservation → check peer-to-peer communication admission policy → track conversation turn correlations and pending turn settlements → apply scoped model overrides → admit work and manage archive/restore lifecycle under single SQLite agent store.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-SESSION-KEY-ROUTING-001` .. `OCRM-RESP-SESSION-WORK-ADMISSION-001`). 35 source files mapped to reviewed status in Git shards. Rust implementation verified in `source/host/tests/openclaw_sessions_and_config_contract.rs`. Full project closure and release acceptance remain false until all 58,750 files are mapped and fully accepted.
