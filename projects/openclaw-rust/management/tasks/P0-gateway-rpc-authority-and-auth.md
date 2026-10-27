# OCRM-P0-GW-001 — Gateway ingress authorization and dispatch parity

Status: **P0 scoped authority/read + existing-owner audit in progress; Rust implementation blocked until §11.1**  
Unit: \`OCRM-UNIT-GW-INGRESS-001\`  
Candidate requirement: \`OCRM-R04\`, \`OCRM-AC11\`, \`OCRM-AC17\`, \`OCRM-AC18\` (crosscheck normative requirement wording before linking a ledger responsibility)  
Dossier: \`projects/openclaw-rust/dossiers/gateway-ingress-scoped.md\`  
Pinned source: \`openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70\` / tree \`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16\`  
Investigated target: \`bhrumom/fabushi-desktop@8fdc06b2e3dd698ae7ea84032bae53655f6230cd\`

## Atomic closed behavior

A connection/request enters the canonical Gateway → authenticates/authorizes exactly once against current account/role/scope/session generation → the existing Host method owner dispatches a typed request → exactly one success/error/unknown-result settlement is exposed to the correct consumer. The final implementation must retain current Fabushi native dispatch and support the complete frozen source-scope compatibility rather than an illustrative RPC demo.

**Current readiness:** NOT READY. Full transitive dependencies and source behavior/oracles not read/approved, scoped G01/G02 not run. No Rust mutation or implemented/wired/verified status is authorized by this task yet.

## Input, state, authority, error and lifecycle contract to establish

- Inputs: frozen upstream version/method definitions, connect origin/auth material, role/scope, request identity, session/account/owner generation, cancellation and deadline.
- Preconditions: authenticated canonical account and method/capability authority; plugin descriptor approved; no read/write authority guessing from the UI.
- Effects: Host/Gateway existing-owner routing, typed response and durable side-effect outcome; no independent OpenClaw state writer.
- Negative/error: malformed frame, unsupported version, unknown method, forbidden origin/proxy, rate limit, revoked scope, stale generation, plugin name collision, cancelled/disconnected request, partial/unknown accepted side effect.
- Recovery: maintain request identity and owner generation on reconnect/restart; reconcile possible accepted mutations rather than blind retry; terminal and cleanup separately observable.
- Acceptance: applicable G01/G02 scoped authority, G03 owner/reachability, G04 Rust contract, G05 frozen oracle, G06 protocol method/version compatibility, G08 lifecycle/security, G07 packaged consumer plus independent review and exact-head receipt.

## Ownership and write scope

**Proposed existing Rust owners, not yet approved:** \`source/host/src/gateway_server.rs\` for transport/admission; \`source/host/src/host_gateway_api.rs\` for method routing; \`source/host/src/gateway_protocol.rs\` for typed surface; existing auth/config/error owner as determined after full audit. Canonical integrator alone changes shared schemas, method tables, composition/lock, ledger and workflows. No writer owns production files until unit admitted; source read-only work may proceed independently.

## Dependency DAG and next executable tasks

1. \`GW-AUTH-01\` fully read imports/dynamic imports and lock package/schema, auth/proxy/roles, registry and source oracles; build exact scoped file+dependency closure and read ranges.
2. \`GW-AUTH-02\` trace target product producer→Host Gateway→actual consumer, permissions, persistence and lifecycle with explicit candidate-owner rejection/selection.
3. \`GW-AUTH-03\` review a scope-specific Gateway contract/oracle and publish scoped G01/G02 Actions validator; failure remains blocked with full logs.
4. \`GW-AUTH-04\` after all §11.1 gates are green, implement actual Rust differential behavior in the existing owner and wire all applicable consumers.
5. \`GW-AUTH-05\` verify G03–G08, preserved Fabushi/FBCP/TDRP behavior, independent read-only review, current target receipt and delivery.

Blocker \`OCRM-BLOCK-GW-SCOPED-AUTHORITY-001\` is **internal missing source/contract evidence**; no end-user intervention currently required. It does not block independent global P0 schema/inventory work or other scoped reading.
