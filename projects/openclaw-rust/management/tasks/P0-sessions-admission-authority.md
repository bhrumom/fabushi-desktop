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

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-SESSION-KEY-ROUTING-001` .. `OCRM-RESP-SESSION-WORK-ADMISSION-001`). 35 source files are mapped to reviewed status in Git shards. The session-path owner now provides a canonical public selector resolver consumed by the five public session/chat projections (`sessions.list/get/delete` and `chat.send/abort`), including harness reservation and malformed-identity rejection; it still does not govern the full cron/direct/group/global/spawn-child mutation boundary (`BL-SESSION-KEY-ROUTING-FULL-WIRING-001`). Communication policy now has a bounded authenticated `sessions.patch` mutation and direct agent-message/inbound policy checks through the existing owner; the full responsibility remains pending for approval UI/consumer, broader channel coverage, exact-head verification, and acceptance. Turn correlation and model-override persistence retain explicit missing-production-path blockers (`BL-SESSION-CONVERSATION-TURN-PRODUCTION-WIRING-001`, `BL-SESSION-MODEL-MUTATION-WIRING-001`). The work-admission owner now has bounded Rust slices for deletion/archive/review/media, external memory journals, and profile/avatar before/after recovery; the full detached lifecycle, provider/device/package paths, exact-head verification, and acceptance remain pending. Contract tests are oracle references only for this development freeze and were not executed locally. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.
