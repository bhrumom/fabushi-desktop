# OCRM Round88 — Gateway Ingress native Rust implementation, Settings mutation lock, and contract test oracles — 2026-10-26

## 1. Executive Summary and Authority Context

- **Frozen Upstream Authority:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Target Implementation Branch:** `ocrm/round20-snapshot-lifecycle-audit-20261023`.
- **Target Responsibility Unit:** `OCRM-UNIT-GW-INGRESS-001` (Module `OCRM-M04 Gateway & Protocol Ingress` and Module `OCRM-M02 Configuration, Profile & Settings Semantics`).
- **Core Milestone:** Implemented native Rust Gateway Ingress authorization, method dispatch, and settings serialization bridging OpenClaw v4 contracts into Fabushi Host without Node fallback or parallel daemon loops.

---

## 2. Source Equivalence & Exact Upstream Paths

The implementation covers and preserves the contracts established in the following upstream source units:

| Upstream Source Path | Blob SHA | Primary Contract Implemented in Rust |
|---|---|---|
| `src/gateway/auth.ts` | `a32dc48530467a505b22b115ea9fb1354bb07c13` | Dual-token authentication, role/scope classification, timestamp skew, and nonce replay validation |
| `src/gateway/server-methods.ts` | `5bebd34bd2b3e5765947fb58a5b71be98588c10a` | Method routing to Host domain owners (`HostGatewayOwner`) |
| `src/gateway/server-methods/request-authorization.ts` | `a6230b631de50d0ad282e108d0af5e62717cfcd2` | Pre-dispatch authorization guards: role checks (`ROLE_MISMATCH`), scope checks (`INSUFFICIENT_SCOPE`), pairing revision checks (`PAIRING_CHANGED`) |
| `src/infra/device-pairing-node-state.ts` | `4cc23766efb819a207bf3b04c596d974a98d93fa` | Unauthenticated handshake exceptions for `device.pair.request` and `device.pair.status` |

---

## 3. Rust Code Modifications & Architecture Wiring

### 3.1 Method Registry & Grok Parity Isolation (`host_gateway_api.rs` & `gateway_protocol.rs`)
- Preserved historical `FROZEN_HOST_GATEWAY_METHODS` (strictly 122 entries) to prevent regressions on existing Grok 0.18 contract tests.
- Introduced `OPENCLAW_GATEWAY_METHODS` (23 methods) mapping to `HostGatewayOwner`:
  - `DevicePairing`: `device.pair.request`, `device.pair.status`, `device.pair.approve`, `device.pair.reject`, `device.pair.list`, `device.pair.revoke`, `device.pair.rotateToken`, `device.pair.setupCode`, `node.register`, `node.status`, `node.list`.
  - `HostControl`: `node.invoke`, `system.info`.
  - `TranscriptManager`: `chat.send`, `chat.abort`, `sessions.list`, `sessions.get`.
  - `AgentLifecycle`: `sessions.create`, `sessions.delete`.
  - `Automations`: `cron.list`, `cron.trigger`.
  - `Settings`: `config.get`, `config.set`.
- Updated `host_gateway_owner(method)` to check OpenClaw methods first, and exposed `is_openclaw_gateway_command(method)`.

### 3.2 Gateway Error Types (`gateway_command_error.rs`)
- Added `GatewayCommandError::Unauthorized(String)` -> HTTP 401.
- Added `GatewayCommandError::Forbidden(String)` -> HTTP 403.

### 3.3 Gateway Ingress Authorization Engine (`gateway_server.rs`)
- **Unauthenticated Handshake:** Allowed `device.pair.request` and `device.pair.status` to bypass bearer token requirement.
- **Clock Skew Protection:** Added `check_device_timestamp_skew` enforcing a strict ±120s window against system time (`TIMESTAMP_SKEW: clock skew exceeds +/-120s`).
- **Nonce Replay Protection:** Added thread-safe `check_and_record_nonce` backed by `NONCE_CACHE` (`NONCE_REPLAY: nonce has already been consumed`).
- **Pairing Revision Freshness:** Checked `x-sand-pairing-revision` header, returning 401 `PAIRING_CHANGED` if marked stale.
- **Role Isolation:** Enforced role boundaries, returning 403 `ROLE_MISMATCH` if `x-sand-role: node` attempts to invoke operator-only methods (`device.pair.revoke`, `device.pair.approve`, `device.pair.rotateToken`, `config.set`, `cron.trigger`).
- **Scope Isolation:** Enforced scope permissions, returning 403 `INSUFFICIENT_SCOPE` if `x-sand-scope: operator.read` attempts mutating operations.
- **Native Security Invariant:** Preserved browser `Origin` header blocking, returning 403 Forbidden on untrusted origins.

### 3.4 Direct Settings Dispatch & Lost-Update Mutex (`main.rs` & `settings_service.rs`)
- In `source/host/app/src/main.rs`, wired `getHostSettings` and `setHostSettings` directly to `self.settings.get_host_settings()` and `self.settings.set_host_settings(&args)`, eliminating the HTTP 404 defect in `UnifiedGatewayApi`.
- In `source/host/src/extensions/settings/settings_service.rs`, added an internal `write_lock: Mutex<()>` to `SettingsService`, acquiring `_guard` across `set_json_field` and `set_host_settings` to eliminate concurrent write races and lost updates between tool permissions, inference usage counters, and RPC settings updates.

---

## 4. Contract Tests & Falsifiable Oracles

Contract tests were added to:
1. `source/host/tests/gateway_protocol_contract.rs`:
   - `openclaw_gateway_extensions_preserve_frozen_inventory_and_owners`: Verifies all 23 OpenClaw methods are recognized, non-overlapping with Grok frozen methods, and mapped to valid `HostGatewayOwner` variants.
2. `source/host/tests/gateway_server_contract.rs`:
   - `openclaw_gateway_device_pairing_handshake_allows_unauthenticated`: Verifies 200 OK for unauthenticated pairing requests and 401 Unauthorized for unauthenticated operator endpoints.
   - `openclaw_gateway_device_timestamp_skew_and_nonce_replay`: Verifies clock skew rejection, nonce consumption, replay rejection, and stale pairing revision rejection.
   - `openclaw_gateway_role_and_scope_enforcement`: Verifies 403 `ROLE_MISMATCH` for node role on operator methods, 403 `INSUFFICIENT_SCOPE` for read-only scope on mutating methods, and 403 Forbidden for browser Origin headers.

---

## 5. Verification Invariant

All executable tests and builds are reserved strictly for GitHub Actions per OCRM spec §6.4. No local unverified state or fabricated receipts are emitted.
