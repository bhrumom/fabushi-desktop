# OCRM-M04 Gateway control-plane ingress — scoped source/owner dossier

Status: **SCOPED G01/G02 NOT YET ACCEPTED — read-only admission research**  
Dossier ID: OCRM-DOS-GW-001  
Unit ID: OCRM-UNIT-GW-INGRESS-001  
Linked task: \`projects/openclaw-rust/management/tasks/P0-gateway-rpc-authority-and-auth.md\`  
Spec: OCRM-001 §6.3/6.4, §7, §11.1, §12.2/12.3  
Source: \`openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70\` (tree \`04ccce62f1d74dc8e0c88290ae8ded027b7b0f16\`)  
Target investigated: \`bhrumom/fabushi-desktop@8fdc06b2e3dd698ae7ea84032bae53655f6230cd\` (tree \`2352d5b7a5d29465977f76c4377de8888bb620e2\`)

## 1. Fixed source anchors and current reading truth

| Source path | Frozen blob | Direct observation / outstanding work |
| --- | --- | --- |
| \`src/gateway/server.ts\` | \`fa45b53cc44a8d6747b3724d577fc1aaae614c86\` | 188 lines; public lazy startup, database-engine admission, SQLite process role, owned Gateway lock, owner-loss handling, spawn broker and close propagation. Full cross-module semantic/dependency review is **not** complete. |
| \`src/gateway/server-methods.ts\` | \`5bebd34bd2b3e5765947fb58a5b71be98588c10a\` | 604 lines; plugin/core registry, write rate limit, session authority, predispatch authorization, unknown-method error, handler and work-continuation dispatch. Only selected call sites and declarations inspected. |
| \`src/gateway/auth.ts\` | \`9769ec1697b1fb81812b987ca59a840c07fc0233\` | 571 lines; connect authorization, trusted proxy/origin and rate-limit dependency surfaces. Selected imports/public methods inspected, not all branches/oracles. |
| \`src/gateway/AGENTS.md\` | \`c619077958b3ec95fcbdb096a550a5144c973c3c\` | 59-line upstream governance source studied for its observable product/lifetime claims; **not** target execution authority. |
| \`packages/gateway-protocol/src/index.ts\` | \`034f25aa1e5e50d9da65ba680941eb17e182a103\` | 103-line export surface; numerous schema, node permissions, session, computer and worker re-exports. Export graph and validators still unresolved. |
| \`packages/gateway-protocol/package.json\` | \`3cb52a3f3cb2d51faa276313aaa8e0337328abce\` | Version 2026.10.5; gateway WebSocket protocol schemas/validators; dependencies and package exports must be resolved. |
| \`src/gateway/server-methods/request-authorization.ts\` | \`a6230b631de50d0ad282e108d0af5e62717cfcd2\` | Referenced as required authority dependency; full content pending. |
| \`src/gateway/server-start.ts\` | \`be8c432d28ccea2eb701ca735dcc91a337f306f1\` | Lazy startup implementation dependency; full content pending. |
| \`packages/gateway-protocol/src/schema.ts\` | \`68b28dd05d1a830c6db8d1d29e3ef928b6f74921\` | Protocol shape/validators transitive dependency; full content pending. |

These are **read/inspection anchors**, not read-complete claims. The Actions recursive manifest contains 5,080 paths under \`src/gateway/\` and 318 under \`packages/gateway-protocol/\` including tests/assets. Those prefix counts alone are **not** the scoped dependency closure or review denominator; do not equate a directory subtree with the actual transitive source set.

## 2. Current Fabushi existing-owner-first candidates

| Target path at investigated target | Blob | Current responsibility / gap |
| --- | --- | --- |
| \`source/host/src/gateway_server.rs\` | \`1ce506ae0bc75ee96418e55eb64a935dd7865ac0\` | Rust HTTP/TLS gateway listener, \`start_gateway_server\`, request/response and event transport. Must inspect all producer/consumer callsites and role/security guards before widening methods. |
| \`source/host/src/host_gateway_api.rs\` | \`e93659285baf4903d3c97cf85629864341ad4126\` | Existing method→Host owner table and the frozen legacy public methods; probable dispatch/admission seam, but no OpenClaw method equivalence claimed. |
| \`source/host/src/gateway_protocol.rs\` | \`13b6a5bad55c133ed67e39bc94b7e500808a82a5\` | Typed/payload Gateway compatibility façade for the current Fabushi surface; must distinguish legacy Grok contract and new protocol version. |
| \`source/host/src/lib.rs\` | \`a58b26c439aec69215e3f125995c8230d3d012b5\` | Canonical Host Rust modules include Gateway/Runner/other owners. Actual product entrypoint and clients still require callgraph proof. |

No new generic Gateway process, Node fallback, second store or generic \`OpenClawCore\` is approved. The Gateway listener/transport and Host method routing are initial candidates, not a completed producer→owner→consumer proof. Adjacent \`gateway_command_error.rs\`, \`gateway_config.rs\`, Host composition, Coordinator clients, renderer/CLI adapters, session/agent authorization and plugin registry must be traced.

## 3. Observed source behaviors and falsifiable oracle questions

- Lazy startup must not mutate process/database identity on a lightweight type-only import; failure/owner loss must terminate or recover the same owner generation, not spawn a second competing owner.
- Connection authentication, origin/trusted-proxy assertions and rate limits must fail closed before side effects; unauthorized/plugin method calls must not dispatch.
- Method registry must preserve core/plugin ownership, request observation vs mutation, role/scope, unknown-method error semantics and session sharing checks.
- A request's operation identity, accepted-write outcome and cancellation must stay attached to the original account/session/owner across reconnect/restart; timeout is not proof of rolled-back mutation.
- Typed protocol schemas, validation errors, version negotiation and compatibility contracts require independent wire oracles against the frozen upstream in the GitHub Actions oracle lane.

These are **future test obligations**, not passing tests or assertions of complete source equivalence.

## 4. Explicit scoped §11.1 admission decision

\`unit_authority_ready=false\`: relevant transitive imports/dynamic imports, generator/schema and production dependency closure are not yet enumerated/locked as a scoped G01 set; all required files are not fully read; source oracles and existing-owner wiring have not passed independent review. Therefore scoped G02 cannot yet be marked \`reviewed\`; Rust implementation **must not start** on this Gateway unit. No \`OCRM-RESP-*\` IDs are upgraded from \`unclassified\` by this dossier.

Exit when the linked task records exact source/target dependency closure and all read ranges, caller/consumer/owner election, verified invariants, an applicable ADR if necessary, and a passing **exact-head GitHub Actions** scoped G01/G02 receipt. Separately preserve current FBCP/TDRP contract version/reachability; do not touch parallel implementation branches.

## 5. Dependency frontier and next non-blocked work

1. Expand \`src/gateway/server.ts\` dynamic dependencies (\`server-start.ts\`, database engine, lock, spawn broker, shutdown) and \`server-methods.ts\`/auth/registry dependencies into immutable source IDs and complete full-text reading, not just import graph enumeration.
2. Read existing Rust \`gateway_server.rs\` dispatch branches, \`gateway_config.rs\`, \`gateway_command_error.rs\`, \`host_gateway_api.rs\`, callers and UI/CLI consumers, including only canonical main semantics.
3. Define stable OCRM-GW source oracles and candidate responsibilities for authorization/dispatch/version/lifecycle; obtain independent review and write scoped G01/G02 Actions gate.
4. Once authority is positively accepted, implement one coherent Rust ingress behavior end-to-end under current owner with negative/timeout/unknown-result and packaged path proofs.

Global source closure and G02 final remain open regardless of any future scoped success.
