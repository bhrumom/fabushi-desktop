# OCRM-P0-DEVICE-PAIRING-001 — Device Pairing, Node Enrollment & Network Authority

Status: **Scoped candidate admitted into parity-ledger (6 responsibilities mapped, 24 audited source files in reviewed status); production Rust wiring blocked by OCRM-BLOCK-GW-SCOPED-AUTHORITY-001; implementation/wiring/verification/acceptance pending**  
Unit: `OCRM-UNIT-DEVICE-PAIRING-001`  
Candidate requirements: `OCRM-R03`, `OCRM-R04`, `OCRM-AC03`, `OCRM-AC11`  
Dossier: `projects/openclaw-rust/dossiers/device-pairing-scoped.md`  
Pinned source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Device pairing and network enrollment enter through canonical Fabushi Network Enrollment API (`network_enrollment_owner.rs`) → enforce cryptographic pairing lifecycle state machine (pending -> approved / rejected / revoked) → manage device verification tokens and setup code exchange → maintain node capability registry and device status heartbeats → isolate account scopes and enforce device revocation fencing.

**Current readiness:** Scoped unit admitted into `parity-ledger.json` (`OCRM-RESP-DEVICE-PAIRING-STATE-MACHINE-001` .. `OCRM-RESP-DEVICE-NODE-LIFECYCLE-001`). 24 audited source files mapped to reviewed status in Git shards (`part-0039.jsonl.gz`), with remaining 39 unread files kept in unclassified state. Real production wiring and external companion device tests remain pending under `OCRM-BLOCK-GW-SCOPED-AUTHORITY-001`. Full project closure and release acceptance remain false until all 58,750 files are mapped and fully accepted.
