# OCRM-P0-DEVICE-PAIRING-001 — Device Pairing, Node Enrollment & Network Authority

Status: **Scoped candidate admitted into parity-ledger (6 responsibilities mapped, 24 audited source files in reviewed status); local Rust producer-to-owner-to-store wiring is closed for the state-machine, store-persistence, and token-lifecycle responsibilities; executable verification and external-device acceptance remain pending under OCRM-BLOCK-GW-SCOPED-AUTHORITY-001**
Unit: `OCRM-UNIT-DEVICE-PAIRING-001`  
Candidate requirements: `OCRM-R03`, `OCRM-R04`, `OCRM-AC03`, `OCRM-AC11`  
Dossier: `projects/openclaw-rust/dossiers/device-pairing-scoped.md`  
Historical scoped source oracle (not current authority): `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`
Current discovery subject: `projects/openclaw-rust/upstream.lock.json::live_candidate` (discovery-only; do not reuse this unit's historical inventory or provenance without rebaseline).
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Device pairing and network enrollment enter through canonical Fabushi Network Enrollment API (`network_enrollment_owner.rs`) → enforce cryptographic pairing lifecycle state machine (pending -> approved / rejected / revoked) → manage device verification tokens and setup code exchange → maintain node capability registry and device status heartbeats → isolate account scopes and enforce device revocation fencing.

**Current readiness:** Scoped unit admitted into `parity-ledger.json` (`OCRM-RESP-DEVICE-PAIRING-STATE-MACHINE-001` .. `OCRM-RESP-DEVICE-NODE-LIFECYCLE-001`). 24 audited source files mapped to reviewed status in Git shards (`part-0039.jsonl.gz`); remaining paths retain honest unread/unclassified progress state until their applicability and disposition are assessed. The three local pairing responsibilities named above have source-level production wiring; exact-head verification, remote companion transport, physical-device tests, and release acceptance remain pending under `OCRM-BLOCK-GW-SCOPED-AUTHORITY-001`. Full project closure and release acceptance remain false until every path is dispositioned and every applicable product responsibility is reviewed, implemented, wired and accepted.
