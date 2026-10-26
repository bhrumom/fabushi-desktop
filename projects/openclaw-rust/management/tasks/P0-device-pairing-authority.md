# OCRM-P0-DEVICE-PAIRING-001 — Device Pairing, Node Enrollment & Network Authority

Status: **Scoped unit admitted into parity-ledger (6 responsibilities mapped, 63 source files read-complete); Rust implementation and wiring in network_enrollment_owner.rs verified by contract tests; acceptance pending**  
Unit: `OCRM-UNIT-DEVICE-PAIRING-001`  
Candidate requirements: `OCRM-R03`, `OCRM-R04`, `OCRM-AC03`, `OCRM-AC11`  
Dossier: `projects/openclaw-rust/dossiers/device-pairing-scoped.md`  
Pinned source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` / tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`  
Investigated target: `bhrumom/fabushi-desktop` HEAD

## Atomic closed behavior

Device pairing and network enrollment enter through canonical Fabushi Network Enrollment API (`network_enrollment_owner.rs`) → enforce cryptographic pairing lifecycle state machine (pending -> approved / rejected / revoked) → manage device verification tokens and setup code exchange → maintain node capability registry and device status heartbeats → isolate account scopes and enforce device revocation fencing.

**Current readiness:** Unit admitted into `parity-ledger.json` (`OCRM-RESP-DEVICE-PAIRING-STATE-MACHINE-001` .. `OCRM-RESP-DEVICE-NODE-LIFECYCLE-001`). 63 source files mapped to reviewed status in Git shards (`part-0039.jsonl.gz`). Rust implementation verified in `source/host/tests/network_enrollment_owner_contract.rs` (`OCRM-DEVICE-PAIRING-O01` .. `O06`). Full project closure and release acceptance remain false until all 58,750 files are mapped and fully accepted.
