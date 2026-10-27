# OCRM Technical Dossier: Unit OCRM-UNIT-DEVICE-PAIRING-001 — Device Pairing, Node Enrollment & Network Authority

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-DEVICE-PAIRING-001`
- **Module ID:** `OCRM-M03` (Identity, Credentials, Pairing and Account Scope)
- **Status:** **Scoped candidate admitted into Parity Ledger; 24 audited source files mapped to reviewed; 6 responsibilities registered with implementation/wiring pending; production wiring blocked by OCRM-BLOCK-GW-SCOPED-AUTHORITY-001; acceptance pending**
- **Requirements Traced:** `OCRM-R03`, `OCRM-R04`, `OCRM-AC03`, `OCRM-AC11`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)
- **Audit Lineage:** PR #169, `projects/openclaw-rust/dossiers/state-read-snapshot-worker-custody-pairing-round20.md`, `pairing-unknown-reconcile-postgresql-provenance-round19.md`, `transcript-round84-gateway-import-closure-protocol-codegen-pairing-worker-20261024.md`, `transcript-round89-openclaw-device-pairing-sessions-and-config-native-wiring-20261026.md`.

---

## 2. Responsibilities Registered in Parity Ledger

1. `OCRM-RESP-DEVICE-PAIRING-STATE-MACHINE-001`: Device pairing cryptographic state machine (pending -> approved / rejected / revoked).
2. `OCRM-RESP-DEVICE-PAIRING-APPROVAL-POLICY-001`: Operator approval policy, admin privilege gate, and rejection reason logging.
3. `OCRM-RESP-DEVICE-PAIRING-TOKENS-LIFECYCLE-001`: Cryptographic verification token generation, single-use invalidation, and rotation.
4. `OCRM-RESP-DEVICE-PAIRING-JOIN-CODE-001`: Device setup / join code verification, expiration window, and format validation.
5. `OCRM-RESP-DEVICE-PAIRING-STORE-PERSISTENCE-001`: SQLite durable pairing store, row facts mapping, and write generation.
6. `OCRM-RESP-DEVICE-PAIRING-NODE-LIFECYCLE-001`: Node enrollment lifecycle, capability registration, heartbeat pruner, and revocation fence.

---

## 3. Blockers & Frontiers

- `OCRM-BLOCK-GW-SCOPED-AUTHORITY-001`: Remaining pairing worker communication and background daemons not fully closed; external physical companion device communication unverified.
- `OCRM-BLOCK-GW-INDEPENDENT-REVIEW-002`: Independent review receipt pending.
- Current Rust status in `parity-ledger.json` is set to `implementation: pending`, `wiring: pending`, `verification: pending`, and `acceptance: pending`.

---

## 4. Itemized Audited Source Files (24 Files)

| Upstream Source Path | Blob SHA | Lines | Language | Read Status | Disposition | Target Fabushi Owner |
|---|---|---|---|---|---|---|
| `src/infra/device-pairing-access.ts` | `2d544ce97feaa9c9bbad4a1cf2ef7fb7d377b21e` | 38 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-admission.types.ts` | `a391cb09f83ea0e74fec9faef2ca799793bf81bb` | 24 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-approval.kernel.ts` | `da7eb9310c14c51aa6cb17b4c6b840e6df29ba85` | 134 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-approval.ts` | `e99f6ba3a79d06bdfce4cf557b7f94bb40e34f66` | 89 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-cloud-worker.ts` | `704c382103f56e9273c52e46ca79dca6d09bb097` | 172 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-core.kernel.ts` | `9b5634fc9b360aa4f395ca0aa097368cebe987c2` | 215 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-core.types.ts` | `f6a479b1adbf9bcceeb74b32a76f28a7d3635ce5` | 98 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-dispatch.worker.ts` | `5c7329aa8a1bb0f7c2448df790ebca5f6c8dca46` | 84 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-identity.ts` | `5c34523a7bb14b9cdb79ab4a3b74ca752f9a7442` | 64 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-migration.test.ts` | `4f38699aa567301c238b97d2e956bc52ec01d0a5` | 110 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/network_enrollment_owner_contract.rs` |
| `src/infra/device-pairing-mutation.worker.ts` | `88c9fa3080aa9b0bfae99e90aaec665d911b3bc2` | 96 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-node.records.ts` | `37803a6bcde6d44af0f779d71a815a13c9e6bb07` | 145 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-node-state.ts` | `ab7329bf7ca5c8894eb5df98baec09bbfa4020a1` | 168 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-publication.ts` | `4ca6984e3aa520bd76df3a2fbcc0c8b6d499bf0e` | 76 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-read.types.ts` | `6b974523d46a8bcddfa0342ab7cd09fe7a353cde` | 42 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-row-mapping.ts` | `e0bcda461794b15099cb2cae89b2ccab932588c1` | 190 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-state.kernel.ts` | `7cc3623f79fa5df9460a80eecb96efbb122a6ea0` | 112 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-store-cache.ts` | `8da77bc542da630f9a2638843c080b00a5cf8705` | 148 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-store-readonly.ts` | `3d67965aaec968df72944b1c7849cbdfba332b50` | 92 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-store.ts` | `7845ba0ceeb28017c6e00b3d5b035a22bb0b54ac` | 230 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-token-utils.ts` | `6022e3ea81335bdfa91bca7f7ea4cb20bf05bb65` | 74 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-worker.test.ts` | `9b360a3a7fca58aa7be03577adccbb98ba03cf8d` | 165 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/network_enrollment_owner_contract.rs` |
| `src/infra/device-pairing-worker.ts` | `6643ae7b9e6e8bb81ca7c6f0e95cb5c90b63afbb` | 210 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing.types.ts` | `37803a6bcde6d44af0f779d71a815a13c9e6bb07` | 55 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
