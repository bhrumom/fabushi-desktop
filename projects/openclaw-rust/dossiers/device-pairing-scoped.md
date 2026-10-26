# OCRM Technical Dossier: Unit OCRM-UNIT-DEVICE-PAIRING-001 — Device Pairing, Node Enrollment & Network Authority

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-DEVICE-PAIRING-001`
- **Module ID:** `OCRM-M03` (Identity, Credentials, Pairing and Account Scope)
- **Status:** **Admitted into Parity Ledger; 63 source files mapped to reviewed; 6 responsibilities registered; 6 falsifiable test oracles implemented in network_enrollment_owner_contract.rs; acceptance pending**
- **Requirements Traced:** `OCRM-R03`, `OCRM-R04`, `OCRM-AC03`, `OCRM-AC11`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

---

## 2. Responsibilities Registered in Parity Ledger

1. `OCRM-RESP-DEVICE-PAIRING-STATE-MACHINE-001`: Device pairing cryptographic state machine (pending -> approved / rejected / revoked).
2. `OCRM-RESP-DEVICE-PAIRING-VERIFICATION-TOKEN-001`: Cryptographic verification token generation and exchange.
3. `OCRM-RESP-DEVICE-SETUP-CODE-EXCHANGE-001`: Device setup code verification and rotation.
4. `OCRM-RESP-DEVICE-CAPABILITY-REGISTRY-001`: Device capability discovery and registry management.
5. `OCRM-RESP-DEVICE-STATUS-HEARTBEAT-001`: Device health heartbeats, reachability, and disconnect tracking.
6. `OCRM-RESP-DEVICE-NODE-LIFECYCLE-001`: Node enrollment lifecycle and device revocation fencing.

---

## 3. Falsifiable Test Oracles

Implemented in `source/host/tests/network_enrollment_owner_contract.rs`:
- `OCRM-DEVICE-PAIRING-O01`: Pairing state machine transitions and invalid transition rejection.
- `OCRM-DEVICE-PAIRING-O02`: Verification token generation, format validation, and single-use exhaustion.
- `OCRM-DEVICE-PAIRING-O03`: Setup code format, expiration window, and validation.
- `OCRM-DEVICE-PAIRING-O04`: Capability registration, scope filtering, and permission checks.
- `OCRM-DEVICE-PAIRING-O05`: Device heartbeat processing, missed heartbeat timeout, and stale status transition.
- `OCRM-DEVICE-PAIRING-O06`: Device revocation fencing, instant token invalidation, and unauthorized access rejection.

---

## 4. Source Files Mapped & Decomposed

All 63 device-pairing files in `src/infra/` (part-0039, 4,600+ lines) are classified as `read-complete`, `reviewed`, with exact owner decisions mapping to `source/host/src/network_enrollment_owner.rs` and related network infrastructure owners.
