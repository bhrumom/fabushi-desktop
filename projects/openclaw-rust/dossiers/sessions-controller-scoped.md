# OCRM Technical Dossier: Unit OCRM-UNIT-SESSIONS-EXT-001 — Session Controller, Transcript Persistence & Telemetry Parity

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-SESSIONS-EXT-001`
- **Module ID:** `OCRM-M07` (Sessions, History, Branches and Transcript)
- **Status:** **Admitted into Parity Ledger; 179 source files mapped to reviewed; 6 responsibilities registered; 6 falsifiable test oracles implemented in openclaw_sessions_and_config_contract.rs; acceptance pending**
- **Requirements Traced:** `OCRM-R04`, `OCRM-R08`, `OCRM-AC08`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

---

## 2. Responsibilities Registered in Parity Ledger

1. `OCRM-RESP-SESSION-CONTROLLER-LIFECYCLE-001`: Session controller lifecycle, state machine, and custody handover.
2. `OCRM-RESP-SESSION-ADMISSION-BARRIERS-001`: Session admission barriers, concurrency limits, and clean execution queueing.
3. `OCRM-RESP-SESSION-TRANSCRIPT-PERSISTENCE-001`: Session transcript journal persistence, page pagination, and durable state storage.
4. `OCRM-RESP-SESSION-MEDIA-ANNOTATIONS-001`: User turn media attachments, image/media metadata, and inline prompt annotations.
5. `OCRM-RESP-SESSION-MUTATION-RECOVERY-001`: Session crash recovery, pending turn reconciliation, and state rollback safety.
6. `OCRM-RESP-SESSION-DIAGNOSTICS-TELEMETRY-001`: Session error telemetry, diagnostic event logging, and status reporting.

---

## 3. Falsifiable Test Oracles

Implemented in `source/host/tests/openclaw_sessions_and_config_contract.rs`:
- `OCRM-SESSION-EXT-O01`: Session controller lifecycle transitions and custody handover to host runner.
- `OCRM-SESSION-EXT-O02`: Session admission barriers and concurrency limits.
- `OCRM-SESSION-EXT-O03`: Transcript persistence, diffing, and page pagination.
- `OCRM-SESSION-EXT-O04`: User turn media attachments and inline prompt annotations.
- `OCRM-SESSION-EXT-O05`: Session crash recovery and upstream link reconnect.
- `OCRM-SESSION-EXT-O06`: Session error telemetry and token usage warnings.

---

## 4. Source Files Mapped & Decomposed

The 179 session controller, barrier, event, and transcript files in `src/sessions/` (part-0046) are classified as `read-complete`, `reviewed`, with exact owner decisions mapping to `source/host/src/extensions/session/` and `source/host/src/extensions/transcript/`.
