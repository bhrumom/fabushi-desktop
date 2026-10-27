# OCRM Round89 — OpenClaw Device Pairing Engine, Sessions Lifecycle, and Config Concurrency Native Wiring — 2026-10-26

## 1. Executive Summary and Authority Context

- **Frozen Upstream Authority:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Target Implementation Branch:** `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- **Target Responsibility Units:**
  - `OCRM-UNIT-DEVICE-PAIR-001` (Module `OCRM-M03 Identity, Credentials, Pairing & Account Scope`)
  - `OCRM-UNIT-SESSIONS-LIFECYCLE-001` (Module `OCRM-M07 Sessions, History, Branches & Transcript`)
  - `OCRM-UNIT-CONFIG-OPTIMISTIC-001` (Module `OCRM-M02 Configuration, Profile & Settings Semantics`)
- **Core Milestone:** Implemented the full SQLite-backed `DevicePairingOwner`, wired 15 OpenClaw gateway RPC methods into `UnifiedGatewayApi`, and asserted 17 falsifiable test oracles across two new contract test suites (`device_pairing_contract.rs` and `openclaw_sessions_and_config_contract.rs`).

---

## 2. Source Equivalence & Upstream Contract Mapping

The implementation equivalence directly covers upstream OpenClaw TypeScript implementations without any external Node dependencies:

| Upstream Source Path | Blob SHA | Fabushi Rust Implementation | Responsibility Boundary |
|---|---|---|---|
| `src/infra/device-pairing-store.ts` | `26b38cbb0c74f57ff7491d09e530089f36a537f5` | `source/host/src/device_pairing_owner.rs` | SQLite persistence for pairing requests, approved devices, tokens, and monotonic revisions |
| `src/infra/device-pairing-node-state.ts` | `4cc23766efb819a207bf3b04c596d974a98d93fa` | `source/host/src/device_pairing_owner.rs` | State transitions (`Pending` -> `Approved` / `Rejected` / `Revoked`), in-band vs cross-device token delivery |
| `src/gateway/server-methods/sessions-read.ts` & `sessions-read-by-key.ts` | `1c89f5c4f2bb7a87e5fa92d1fa34c1b26804bb65` | `source/host/app/src/main.rs:call_openclaw_sessions_list` & `call_openclaw_sessions_get` | Sessions projection from `SessionWorkers`, zero-messages 200 OK invariant for non-existent session keys |
| `src/gateway/server-methods/sessions-create.ts` | `5e8697193b2111ce3e8d672cf39735d1f868ad93` | `source/host/app/src/main.rs:call_openclaw_sessions_create` | Direct delegation to `TranscriptManager::dispatch_agent_lifecycle_gateway_call` with agent metadata normalization |
| `src/gateway/server-methods/sessions-delete.ts` | `a108c90b6a386ad2f0c76adbb14a091498c117b3` | `source/host/app/src/main.rs:call_openclaw_sessions_delete` | Root main session (`global`) protection, CAS `expectedSessionId` verification, automation schedule cleanup |
| `src/gateway/server-methods/config.ts` | `41c88e998a4ef7e0cb6cf389658051280387bdf5` | `source/host/app/src/main.rs:call_openclaw_config_get` & `call_openclaw_config_set` | Optimistic concurrency check with SHA-256 `baseHash`, atomic `write_lock` serialization, 409 Conflict rejection |
| `src/gateway/server-methods/system-info.ts` | `ff3a7ce1c49733cf8528990eb5db0d60c1d68378` | `source/host/app/src/main.rs:call_openclaw_system_info` | Zero secret leakage, system architecture/memory reporting, and host diagnostics recording |

---

## 3. Rust Code Modifications & Architecture Details

### 3.1 Device Pairing Authority (`device_pairing_owner.rs`)
- Implemented `DevicePairingOwner` backed by embedded SQLite with schema:
  - `device_pairing_pending`: Pending pairing requests with UUID `request_id`, `device_id`, `public_key`, platform metadata, and expiration.
  - `device_pairing_paired`: Authoritative paired devices with encrypted/bearer tokens, assigned roles, approved scopes, and timestamps.
  - `device_bootstrap_tokens`: Setup tokens and single-use redemption tracking.
  - `device_pair_setup_completions`: Setup completion audit log.
  - `config_machine_state`: Single-writer monotonic sequence tracking (`devicePairing.revision`).
- Implemented all 8 pairing RPC methods:
  1. `device.pair.request`: Records pending request, marks `is_repair` if previously known.
  2. `device.pair.status`: Returns `pending`, `approved`, or `rejected`.
  3. `device.pair.approve`: Atomically removes pending row, upserts paired record, generates operator token, and bumps revision.
  4. `device.pair.reject`: Purges pending request, returns `decision: rejected`.
  5. `device.pair.list`: Aggregates pending and paired rosters.
  6. `device.pair.revoke`: Deletes paired device and increments revision; returns 400 for unknown device.
  7. `device.pair.rotateToken`: Rotates token; if `crossDevice: true`, withholds token in-band (`"tokenDelivery": "withheld-cross-device"`), else returns token in-band (`"tokenDelivery": "in-band"`).
  8. `device.pair.setupCode`: Generates single-use bootstrap code and stores in `device_bootstrap_tokens`.

### 3.2 Unified Gateway Integration (`main.rs`)
- Registered `DevicePairingOwner` in `UnifiedGatewayApi` initialized from `app_data_dir.join("device-pairing.db")` with safe in-memory fallback.
- Added fast-path intercept in `UnifiedGatewayApi::call`:
  ```rust
  if is_openclaw_gateway_command(method) {
      return self.call_openclaw_gateway_command(method, args);
  }
  ```
- Implemented helper methods for `config.get`, `config.set`, `system.info`, `sessions.list`, `sessions.get`, `sessions.create`, `sessions.delete`, and `device.pair.*`.

### 3.3 Diagnostic Event Helper (`host_diagnostics.rs`)
- Added `report_diagnostic_event(kind: impl Into<String>, fields: Value)` to safely construct and emit `HostDiagnostic` reports with JSON metadata.

---

## 4. Contract Tests & Verification Receipts

### 4.1 Test Suites Added
1. `source/host/tests/device_pairing_contract.rs`:
   - `test_ocrm_pair_o01_request_success`: Asserts pending request generation with UUID `pair-*` and expiration.
   - `test_ocrm_pair_o02_request_requires_device_id_and_public_key`: Asserts 400 rejection on missing parameters.
   - `test_ocrm_pair_o03_and_o04_pair_status`: Asserts pending status for valid request and rejected for unknown ID.
   - `test_ocrm_pair_o05_and_o06_pair_approve`: Asserts atomic approval, token creation, revision bump, and 400 on unknown request ID.
   - `test_ocrm_pair_o07_pair_reject`: Asserts pending deletion and transition to rejected status.
   - `test_ocrm_pair_o08_pair_list`: Asserts dual list aggregation of pending and paired devices.
   - `test_ocrm_pair_o09_and_o10_pair_revoke`: Asserts revocation, revision bump, and 400 on unknown device.
   - `test_ocrm_pair_o11_rotate_token_in_band_and_cross_device`: Asserts token rotation delivery policy (in-band vs withheld).
   - `test_ocrm_pair_o12_setup_code`: Asserts setup ID, code prefix `sand-setup-*`, and full access parameters.

2. `source/host/tests/openclaw_sessions_and_config_contract.rs`:
   - `test_config_get_computes_sha256_hash_and_validates_base_hash`: Asserts SHA-256 hash computation and baseHash match assertion.
   - `test_config_set_optimistic_concurrency_rejects_stale_base_hash_with_409_conflict`: Asserts stale update detection and HTTP 409 Conflict rejection.
   - `test_system_info_schema_and_zero_secret_leakage`: Asserts complete system information schema, zero credential exposure, and diagnostic reporting.
   - `test_sessions_get_empty_messages_200_invariant`: Asserts 200 OK empty messages invariant for missing sessions.
   - `test_sessions_delete_guards_root_session_and_cas`: Asserts root session deletion protection (`global`) and CAS `expectedSessionId` verification.

### 4.2 Verified CI Run
- **GitHub Actions Run:** [40218220794](https://github.com/bhrumom/fabushi-desktop/actions/runs/40218220794)
- **Verified Commit:** `3354f6451022ee3c2d6f65d38d3c30576942e624`
- **Matrix Results:**
  - `platform-worker`: `success`
  - `Pressure profiler / macos-15`: `success`
  - `Pressure profiler / ubuntu-24.04`: `success`
  - `Pressure profiler / windows-2025`: `success`
  - `renderer`: `success`
  - `rust-host` (including `device_pairing_contract.rs` and `openclaw_sessions_and_config_contract.rs`): `success`
