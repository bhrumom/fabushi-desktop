# Exact-source dossier: orders 5001-5030

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Read method: direct exact-blob semantic reading of translation-provider, layout, diagnostics/bootstrap, account/domain/session/config/settings and send-identity state machines, service boundaries, persistence, failure handling and teardown behavior. Read-complete does not imply implementation or release closure.
- Accounting after batch: read-through 5,030; unread 11,090; unknown 15,841; unknown-closed 279; omitted 0.

## Responsibility decomposition

### 5001-5002: Custom translation URL provider lifecycle
- Canonical owner: Canonical Translation service gateway
- State machine: language/text input -> encoded template expansion -> bounded HTTP request -> JSON path extraction -> translation/error.
- Failure boundary: invalid provider template, unknown language, HTTP/JSON failure or stale response.
- Disposition: `mapped-open-translation-url-provider`; implementation and release verification remain open.

### 5003-5014: Reusable transcript/media layout geometry, mosaic, hit testing and selection
- Canonical owner: Canonical TranscriptEntry layout/media composition owner
- State machine: content/media metadata -> min/max geometry and mosaic rows -> RTL-aware hit testing/selection -> repaint/reuse/invalidation.
- Failure boundary: invalid dimensions, stale reused geometry, selection drift, wrong RTL coordinates or unbounded derived-media lifetime.
- Disposition: `mapped-open-transcript-layout`; implementation and release verification remain open.

### 5015-5016: Desktop diagnostics/logging lifecycle
- Canonical owner: Canonical Desktop diagnostics/logging owner
- State machine: bootstrap buffering -> main/debug/MTP log routing/rotation -> flush/teardown.
- Failure boundary: unwritable path, rotation/flush race, diagnostic leakage or teardown loss.
- Disposition: `mapped-open-desktop-diagnostics`; implementation and release verification remain open.

### 5017: Process/bootstrap crypto lifetime
- Canonical owner: Canonical Desktop process/bootstrap owner
- State machine: process entry -> detached-worker-safe OpenSSL exit policy -> application bootstrap -> shutdown.
- Failure boundary: global crypto cleanup racing detached work or duplicate initialization.
- Disposition: `mapped-open-desktop-bootstrap`; implementation and release verification remain open.

### 5018-5019: Account transport/session ownership and authorization persistence
- Canonical owner: Canonical Account/Session owner
- State machine: account storage/config -> transport start/restart -> authorization/session -> updates -> logout/key destruction/reset.
- Failure boundary: corrupt auth/config, proxy/restart race, stale key write, unauthorized session reuse or incomplete logout persistence.
- Disposition: `mapped-open-account-session-owner`; implementation and release verification remain open.

### 5020-5023: Remote application configuration
- Canonical owner: Canonical AppConfig/settings owner
- State machine: cached config -> remote fetch -> typed normalization -> feature/limit propagation -> retry/refresh.
- Failure boundary: malformed values, stale config, unsupported keys or refresh/network failure.
- Disposition: `mapped-open-app-config`; implementation and release verification remain open.

### 5024-5025: Multi-account domain, unread aggregation, windows and passcode cleanup
- Canonical owner: Canonical Account roster/domain owner
- State machine: domain/account load -> active account/session -> aggregate unread/window routing -> logout/redundant removal -> authorized passcode cleanup.
- Failure boundary: active identity loss, stale unread aggregate, orphan window, unsafe account destruction or premature passcode removal.
- Disposition: `mapped-open-account-domain`; implementation and release verification remain open.

### 5026-5027: Session service composition and persisted runtime lifecycle
- Canonical owner: Canonical Session runtime owner
- State machine: authorized account -> construct session services/data -> deferred load/config/self/terms/link flows -> delayed settings persistence -> upload stop/orderly destruction.
- Failure boundary: stale service after teardown, deferred write loss, upload shutdown race or inconsistent self/config state.
- Disposition: `mapped-open-session-runtime`; implementation and release verification remain open.

### 5028-5029: Session preference serialization and backward compatibility
- Canonical owner: Canonical Session settings persistence owner
- State machine: typed preferences -> versioned binary serialization -> backward-compatible decode/migration -> reactive getters/setters -> persistence.
- Failure boundary: corrupt/truncated stream, invalid enum/range, migration drift, lost thread setting or stale security/auth-review state.
- Disposition: `mapped-open-session-settings`; implementation and release verification remain open.

### 5030: Send-as identity eligibility and refresh
- Canonical owner: Canonical message send identity/permissions owner
- State machine: conversation context -> cached send-as/default identities -> rights/story/premium refresh/TTL -> selection/server mutation.
- Failure boundary: stale rights/default identity, expired cache, missing peer, premium/paid eligibility mismatch or server mutation failure.
- Disposition: `mapped-open-send-identity`; implementation and release verification remain open.

## Accounting and acceptance

All 30 entries are exact-tree/path/blob bound and `read_complete=true`, but `unknown_closed=false`, `omitted=false` and their dispositions remain `mapped-open-*`. This batch therefore lowers only `unread`; it does not claim production implementation, source closure, release acceptance, or independent acceptance. A current exact-head GitHub Actions Source authority run must validate the new shard and attestation manifest.
