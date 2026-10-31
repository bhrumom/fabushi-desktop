# OCRM-P0-SESSIONS-001 — Session Admission, Keys & Turn Correlation Authority

Status: **Scoped unit admitted into parity-ledger (6 responsibilities mapped, 35 source files read-complete); partial Rust implementation exists, while complete session admission/wiring remains pending; acceptance pending**
Unit: `OCRM-UNIT-SESSIONS-001`  
Candidate requirements: `OCRM-R04`, `OCRM-R08`, `OCRM-AC08`  
Dossier: `projects/openclaw-rust/dossiers/sessions-admission-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Current discovery subject: `projects/openclaw-rust/upstream.lock.json::live_candidate` (discovery-only; do not reuse this unit's historical inventory or provenance without rebaseline).
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Session requests enter through canonical Fabushi session API → classify session key kind and enforce harness namespace reservation → check peer-to-peer communication admission policy → track conversation turn correlations and pending turn settlements → apply scoped model overrides → admit work and manage archive/restore lifecycle under single SQLite agent store.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-SESSION-KEY-ROUTING-001` .. `OCRM-RESP-SESSION-WORK-ADMISSION-001`). 35 source files are mapped to reviewed status in Git shards. The session-key classifier and harness namespace guard have real public Host projection/admission call sites, but the classifier does not yet govern the full cron/direct/group/global/spawn-child mutation boundary; the remaining communication, turn, model-override, auth-profile, and work-admission rows remain `not-wired` or `not-started`. Contract tests are oracle references only for this development freeze and were not executed locally. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.
