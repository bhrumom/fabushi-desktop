# OCRM-P0-CONFIG-001 — Configuration, Schema Normalization & Single-Writer Settings Authority

Status: **Scoped unit admitted into parity-ledger (6 responsibilities mapped, 8 source files read-complete); source-level contract assertions are present for settings_service.rs/agent_paths.rs, but no current exact-head Actions receipt proves implementation, production wiring, or acceptance; acceptance pending**
Unit: `OCRM-UNIT-CONFIG-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R09`, `OCRM-AC05`, `OCRM-AC17`  
Dossier: `projects/openclaw-rust/dossiers/config-settings-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Current discovery subject: `projects/openclaw-rust/upstream.lock.json::live_candidate` (discovery-only; do not reuse this unit's historical inventory or provenance without rebaseline).
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Configuration reads and mutations enter through canonical Fabushi settings API → check immutability / deployment mode guards → acquire serial writer mutex → traverse or update properties along safe dot notation paths with prototype pollution defense → persist atomically via temporary staging files and sync → notify subscribers of effective field changes.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-CONFIG-PATH-MUTATION-001` .. `OCRM-RESP-CONFIG-AGENT-WORKSPACE-DIRS-001`). 8 source files mapped to reviewed status in Git shards. `source/host/tests/settings_service_contract.rs` contains source-level contract assertions; this is not a current exact-head Actions receipt and does not by itself prove production wiring. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.
