# OCRM-M04 Gateway control-plane ingress — scoped source/owner dossier

Status: **SCOPED G01/G02 ADMITTED — 5 responsibilities mapped, 10 source files read-complete**  
Dossier ID: OCRM-DOS-GW-001  
Unit ID: OCRM-UNIT-GW-INGRESS-001  
Linked task: `projects/openclaw-rust/management/tasks/P0-gateway-rpc-authority-and-auth.md`  
Spec: OCRM-001 §6.3/6.4, §7, §11.1, §12.2/12.3  
Source: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)  
Target investigated: `bhrumom/fabushi-desktop` HEAD

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

## 6. GW-AUTH-01/02 exact-source follow-up: 2026-10-22 (read-only, **not** a G01/G02 acceptance)

**Source lease:** openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70, tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16, gateway subtree 0a0b92dd355ab0054af724ea469dd6571c0fa30a. **Target read lease:** bhrumom/fabushi-desktop@2a2c2669b580e40b20f71e2187a0109841e14197, tree 471266e9da4e028df5ac4bf2c04acc33e7ddaf57. These identities describe the examined input; a later documentation carrier does not inherit its parent's Actions verdict. This addendum supersedes the older reading-progress claims for the specific full-text files named here only, not their unresolved imports.

### 6.1 GW-AUTH-01 source anchors and behavior

| Frozen source file | Git blob SHA | Bounded observation |
| --- | --- | --- |
| src/gateway/auth.ts | 9769ec1697b1fb81812b987ca59a840c07fc0233 | Reviewed 571-line auth implementation: token/password/none, trusted proxy and Tailscale pathways, credential timing-safe comparison, proxy/origin provenance, rate limiting and failure behavior. |
| src/gateway/auth-policy.ts | 8fcf9bc3834e4185e69fa530fa68426ac2586b09 | Auth policy snapshot vs grant-generation/currentness: rejected stale grant must not revive across reload. |
| src/gateway/auth-mode-policy.ts | 7186d5401547885993a876ac31cb103aebabe833 | Conflicting token and password without an explicit mode is a configuration error. |
| src/gateway/auth-resolve.ts | 8d1f04f1e2756527769d4146d0d3f5c07629d2b7 | Auth-mode resolution, explicit/implicit credential sources and hot config. |
| src/gateway/auth-surface-resolution.ts | a64a6ae3e284c177d7a1a6cdf6c20fce608a564f | Different passive probe vs interactive-client credential sourcing; unresolved references cannot silently use ambient fallback. |
| src/gateway/auth-token-resolution.ts | d6a58912dc677110e7e8e7e0f4da7ff6d74e3679 | Token override/config/ref/environment precedence; unresolved ref fails closed. |
| src/gateway/auth-token-store-ref.ts | 30af61fbae743028638c8666cab9f457db94bfe0 | Credential-store provisioning/reuse preserves secret lifecycle, avoiding accidental rotation. |
| src/gateway/role-policy.ts | 09247fd9cfeb26a2bf5642d325d41f3d397a709a | Closed operator/node role model and node RPC boundary. |
| src/gateway/method-scopes.ts | 1e5a8343fe315afa88eeb5ebeb3c96b569a670d8 | Admin/read/write/approvals/questions/pairing and session scopes; dynamic methods do not default to ambient write. |
| src/gateway/server-methods.ts | 5bebd34bd2b3e5765947fb58a5b71be98588c10a | Entire 604-line dispatcher read: core/plugin/aux registry, root-work admission, predispatch authority, post-await and pre-invocation guard, response/error settlement. |
| src/gateway/server-methods/request-authorization.ts | a6230b631de50d0ad282e108d0af5e62717cfcd2 | Inspected predispatch access/startup/session-pairing fences, including lines 251-661 in full; source-dependency closure remains open. |
| src/gateway/server-methods/method-authorization.ts | a4dda8be141ad623d9e2e668f51e2e6bf2227cd3 | Role/scope and session-operator authorization before handlers; current operator role-policy rechecked. |
| src/gateway/server-methods/gateway-client-identity.ts | d7bf828d12d055a669ed9bc6257523559bf1c960 | Verified-profile hydration and caller attribution; unresolved profile returns retryable unavailable instead of identity fabrication. |
| src/gateway/methods/core-method-policy.ts | 2b64d1738783b5261d091ef40e7f34606fd1f0f3 | Unclassified core method rejects descriptor creation. |
| src/gateway/methods/registry.ts | e073cd9d480820c47662a786c90de9558a9d379e | Duplicate method name rejected, plugin namespace scope normalization, own-session requires write+profile, sharing restricted to bounded reads. |
| src/gateway/server/ws-connection/connect-auth.ts | 708e34da5847a31d4d839b7917eeb65ae53c2496 | Complete 608-line connect authentication read: signed device ID/public key, nonce, +/-120s clock window, shared/session generation and credential revalidation. |
| src/gateway/server/ws-connection/connect-device-pairing.ts | 57cf297e7d50f725ce657553972c8da17f214712 | Complete 624-line pairing approval read: pending request identity, current durable row, role/scope/key, approval/revocation races, token issuance fences. |
| src/gateway/server/ws-connection/connect-policy.ts | fbcd9f470e079f31d20eef3fc2b332431cb2eea0 | Device-less path limited to specified local/Control-UI authenticated operator cases; no blanket node bypass. |
| src/gateway/server/ws-connection/connect-operator-access.ts | e9e777538b720f424872b8e84fd515d8ac44cae7 | Operator profile access and revocable grant tied to current device-token record. |
| src/gateway/server/ws-connection/connect-device-tokens.ts | 443df70b1e67d60b14c3041d4d2bef2e3b28f6d9 | Tokens issued only for approved live baseline, with scope allowlist / shared-generation. |
| src/gateway/agent-runtime-identity-token.ts | 851d02ddf54b353e450028f78f70a51052369165 | Complete 661-line typed signed identity token: operational run and delegated authority, 60s action/cron claims, current HMAC and run/claim verification. |
| src/gateway/device-revocation.ts | d4b14934c45c5cb63f127846bebc8c49744e36b4 | Complete 304-line retained source capture and revocation of in-flight accepted request; no recapture of later grant. |
| src/gateway/device-scope-upgrade.ts | 4f3311114fd5ce0b7618e5db9bd4e58acfb3b5b3 | Durable pairing-row reconciliation, scope upgrade by same exact device public key, observer cancellation. |
| src/gateway/device-pair-setup-completion.ts | e3c7b362ac0dab6f4d165c2be9138b5e723c765a | Handoff delivery can be confirmed or outcome-uncertain; consuming bearer is not proof peer received it. |

### 6.2 Source oracles directly inspected, **not run or accepted**

| Source test | Git blob SHA | Falsifiable negative case |
| --- | --- | --- |
| src/gateway/server-methods.dispatch-authorization.test.ts | fcc55a7d5efee9b9c3dff52104f3a9bf92f74d39 | Attached registry governs dispatch despite newer global plugin registry; operator.read cannot invoke operator.write; a stale node pairing rejects all node RPCs with PAIRING_CHANGED. |
| src/gateway/server-methods.authorization.test.ts | e0cbbe87dbf58486d824ddc5bf31f72d6d4a9714 | Admin-only setup; profile self-service denies other owner; session replacement/reassignment during await prevents stale mutation/response. |
| src/gateway/server/ws-connection/connect-policy.test.ts | 9dbb41f3e3ba6296ca474df7d602277b1357b352 | auth.mode none only skips operator Control UI pairing, never node or all clients; trusted-proxy exception requires valid matching auth+role. |
| src/gateway/server/connection.test.ts | 269276112f1887b9060e327a2228cfc60b60a253 | Single admitted connect, a single hello-ok and receive registration, cleanup once regardless of hello frame delivery success. |
| src/gateway/server-methods/device-management-authz.test.ts | 7faa48ca9e3e21d94bf50b2c84137eb4e1945fb0 | Self-asserted shared-auth device metadata is untrusted; actual device-token auth can bind self-service device identity. |
| src/gateway/server/ws-connection/auth-messages.test.ts | 69f9be2eb45def8e496fdb70aec8917a72d03b6a | Precise pairing/scope/auth refusal, bounded close-reason and redacted sensitive material. |

### 6.3 Scoped transitive dependencies: explicit non-closure

The gateway ingress graph is larger than the above anchors. Mandatory unread or incompletely audited direct/transitive inputs include src/gateway/server/ws-connection/message-handler.ts, connect-session.ts, connect-admission.ts, connect-device-metadata.ts, connect-pairing-approval-plan.ts, connect-hello.ts, authenticated-request-dispatch.ts, src/gateway/server/connection.ts (first 190/608 inspected), request-authorization.ts supporting session-sharing modules, src/infra/device-pairing* persistence/approvals, src/gateway/operator-role-policy.ts, credential and audit modules, packages/gateway-protocol/src/schema.ts, protocol-codec exports, packages/gateway-client and normalization/retry packages; dynamic plugins, version compatibility, build/generator declarations and tests must all receive exact blob identities and full type-adapted reading. The earlier manifest counts 5,080 src/gateway paths and 318 gateway-protocol paths are prefix inventory only, not a computed transitive closure. Scoped G01 completeness and G02 reviewer verdict remain **false**.

### 6.4 GW-AUTH-02 actual existing production seam (partial caller/consumer proof)

| Fabushi target source at investigated exact H | Git blob SHA | Proven call/writer responsibility |
| --- | --- | --- |
| source/host/src/gateway_server.rs | 1ce506ae0bc75ee96418e55eb64a935dd7865ac0 | Complete 1244-line read. Existing HTTP/TLS listener, Origin denial, loopback Host guard for tokenless mode, timing-safe Bearer check, known-method whitelist, /health, events/bridge routes; no WS role/scope/pairing handshake equivalent exists in this file. |
| source/host/src/host_gateway_api.rs | e93659285baf4903d3c97cf85629864341ad4126 | Complete read: explicit method-to-Host feature-owner mapping; not an OpenClaw plugin descriptor registry. |
| source/host/src/gateway_protocol.rs | 13b6a5bad55c133ed67e39bc94b7e500808a82a5 | Complete read: existing Grok/Fabushi façade; does not prove OpenClaw schema/protocol parity. |
| source/host/src/gateway_config.rs | 64af7f6354047fe18b3e94fe629236ee0ade1942 | Complete read: configured/random auth token, loopback/TLS guards and Host discovery; no role-specific reusable device grant owner proven. |
| source/host/src/gateway_command_error.rs | a7827b29c7f53899b1f34e7c989849816f4090fb | Complete read: existing errors/status classes. |
| source/node-agent-coordinator/src/gateway/gateway_request_dispatcher.rs | 38ca19db76493f9c2bca8862b484d61b7e74f47c | Complete read: Coordinator POST /api/{method}, HTTP auth headers, bounded timeout/error classification; client is not canonical business writer. |
| source/node-agent-coordinator/src/gateway/host_supervisor.rs | ea7d370c0a79948e7e6a0a179405d3aacd9e3a56 | Complete read: Host discovery bearer injection, health/reconnect epochs and endpoint generations. |
| source/node-agent-coordinator/src/gateway/gateway_client.rs | 67c666d4b3ec716aca60763bb53c594f066d0660 | Complete 1064-line read: SSE + bounded retries for sendPrompt/createAgent only after original endpoint and dedupe capability guard; no automatic replay if admission outcome not proven safe. |
| source/host/src/runner_production_bridge.rs | 56b1af467b70d7f2ac8f1f156e1e497441928801 | Complete read: immutable per-turn Host→Runner dependency projection; not a Gateway account/scope owner. |
| source/host/app/src/main.rs | 8e19361b3b278d085927a84ea4c8ce8c3fe3de80 | TARGETED ranges 8920-9060, 10390-10530, 13120-13220: UnifiedGatewayApi impl routes approved existing method into transcript/session, runner acceptance/start; app launches gateway and writes host discovery. Remainder of 13719 lines, related state write/callers not full-read. |
| source/node-agent-coordinator/src/main.rs | c822c07dff3027a22a658d1ece6aefe919691365 | TARGETED ranges 1715-1795, 2030-2180, 2575-2635: Coordinator dispatches via HTTP Host, runner.acceptRoutedPrompt then runner.startRoutedProvider, inference store projection vs Host-owned Transcript. Remainder not fully read. |

Observed partial live chain: Coordinator incoming command → dispatch_gateway_value → CoordinatorGatewayClient / dispatch_http_json_response → Host /api/{method} Bearer boundary → GatewayApi::call_with_context → UnifiedGatewayApi → existing Transcript/Session/Runner services → SSE/HTTP consumer. Host app also calls sendPrompt from native TeachRecording. Renderer/UI/CLI/companion consumers and auth/storage owner call graph remain unread, so **no full G02 assertion**. Candidate disposition: extend existing Host Gateway admission plus typed routing; keep Coordinator as transport/lifetime supervisor and Runner as attempt executor; preserve Host-owned Session/Transcript persistence and reject a second Gateway runtime/second writer. This is a candidate, pending owner/ADR independent review.

### 6.5 Falsifiable scoped G01/G02 oracle table to publish only after positive admission

| Oracle ID | Input / manipulated boundary | Required assertion in Actions oracle lane |
| --- | --- | --- |
| OCRM-GW-AUTH-O01 | Missing/incorrect bearer, remote Host spoof, untrusted browser Origin | No handler invocation, correct denial; current local health and established Fabushi authorized traffic preserved. |
| OCRM-GW-AUTH-O02 | Shared token/password/none, trusted proxy and Tailscale, inactive secretRef | Exact mode authority and origin handling; no ambient credential escalation, config conflict or unverified proxy identity. |
| OCRM-GW-AUTH-O03 | Node vs operator scopes, admin-only method, plugin collides with core method | Scope-bound error before side effects; descriptor selected for policy and execution is the same; default-deny unclassified. |
| OCRM-GW-AUTH-O04 | Signed device nonce mismatch, key substitution, stale timestamp, scope escalation, user-profile absence | Refuse pairing or mutation; never issue reusable credentials from an unverified or revoked principal. |
| OCRM-GW-AUTH-O05 | Approval/revoke/credential rotation while awaiting pairing/auth/session fact | Current durable row and source generation fence after all awaits; late connection fails with PAIRING_CHANGED or appropriate denial. |
| OCRM-GW-AUTH-O06 | Session replaced/reassigned while mutation handler is parked | Mutation guard blocks old write; no stale or unpermitted response content emitted. |
| OCRM-GW-AUTH-O07 | Accepted message/tool turn disconnects, timeout or Host restart | No blind duplicate mutation: stable request id, explicit accepted vs denied vs outcome unknown, reconcile before retry. |
| OCRM-GW-AUTH-O08 | Hello response delivered vs transport write failure, connection close/reconnect | Receive registration once and lifecycle cleanup exactly once, without upgrading socket delivery to peer-confirmed completion. |
| OCRM-GW-AUTH-O09 | Probe/registry/source closure negative changes, altered upstream SHA, missing declared dependency/test | Scoped validator fails closed, exact subject head/tree + dependency digest tied to artifact and independent verdict. |

### 6.6 G01/G02 and §11.1 verdict / blockers

- GW-AUTH-01: **READ PARTIAL**. Many exact blobs now anchored, but indirect and dynamic imports, schema/generator/dependency/package closure, selected source/test ranges and all required oracle inputs are **not** read/locked. Scoped G01 unavailable, not passed.
- GW-AUTH-02: **OWNER TRACE PARTIAL**. Existing Host/Coordinator/Runner route is supported by exact code above; actual account, OAuth, plugin, UI/CLI, remote node and channel consumers plus permanent auth/storage owner remain open. Scoped G02 independent verdict unavailable.
- GW-AUTH-03 / §11.1: **NOT ADMITTED**. No approved source closure, independent reviewer, exact-head scoped G01/G02 Actions receipt, or verified protocol equivalence. Production Rust and ledger/RTM acceptance flags must remain unchanged.
- Follow-up: read complete frozen ws connect/session authorization and infra/device-pairing persistence + app/UI/CLI consumers, expand a real scoped import/manifest graph with blob SHA and negative probes, run only in GitHub Actions once Actions approval is unblocked, then obtain independent review before any production mutation.

**Evidence discipline:** This document is a non-production, unaccepted audit snapshot. File blobs prove identity and visible reading, not tested execution, implementation, wiring, G01/G02 acceptance or global migration. The previous exact-head run 39743564294 (action_required at old target H) does not prove a subsequent documentation carrier H2. No secret/token value is recorded here.


## 7. 2026-10-22 pairing and production-consumer continuation (round 7, unaccepted)

**Evidence type: exact-source read-only investigation, not independent review or scoped G01/G02 receipt.** Frozen upstream `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`); examined target PR #169 parent H=`4730a01f3d210daa5eef3d788de4af5f27a41eba` (tree `b8f93577cd7889fe6ec5ed24874f8602d85a4ede`). Live upstream `main` had moved to `e603eca277ed541e5d8dedefe86eb91587e9a2e7`; this is a drift observation only, **not** a rebaseline. This documentation's later carrier commit is a distinct target and may not inherit H's Actions evidence.

### 7.1 Four specifically requested source files: full text acquired/read in entirety

| Frozen path under `src/infra/` | Exact Git blob SHA | Complete inspected interval | Source responsibility / authority evidence |
| --- | --- | --- | --- |
| `device-pairing-store.ts` | `b7592120467b67ab19011d3ce3eea055559adf25` | **1–649/649** | `devicePairing.revision` increment in paired-write transaction (169–201), coherent row+revision snapshot (204–245), commit-only cache/publication invalidation (114–145), token expiry and same-device recheck during consumption (448–547), completion `uncertain`→`confirmed` with bound token retirement in one transaction (550–585). |
| `device-pairing-node.ts` | `3e032d10f1b2079a22784e8181d14f2eda8e4946` | **1–332/332** | Generation-scoped pending reconnect cleanup claim (42–91, 110–167); exact existing pending revision reused on unchanged reconnect and older claim invalidated (179–209); approved scope/current authority and live cleanup-claim admission enforced (211–261); connect/presence/disconnect mutations carry expected generation (284–319). |
| `device-pairing-node-state.ts` | `4cc23766efb819a207bf3b04c596d974a98d93fa` | **1–78/78** | Published node binding checks identity and generation (17–34), authenticated node requires exact persisted public key + bearer token + effective node role before exposing approved caps/commands/permissions (42–69), generation rechecked following store access (71–77). |
| `device-pairing-approval.kernel.ts` | `7a563c858c078347587d4bfc6e153be59550393a` | **1–382/382** | Existing owner/bootstrap approval provenance cannot be silently downgraded (43–61); node surface and operator label preserved through reapproval (70–110); pairing generation/pending+paired and changed-key APNS cleanup committed together (112–142); silent repair with changed key cannot autoapprove even on trusted ingress (198–237); operator caller-scope ceiling before token issuance (238–302); bounded bootstrap grants (305–381). |

### 7.2 Related direct dependencies read in full at the same frozen source

| Frozen path under `src/infra/` | Git blob SHA | Complete inspected interval | Relevant function |
| --- | --- | --- | --- |
| `device-pairing-worker.ts` | `190f6301ff4a865992d8bd1c5f3ddf4aff912ca5` | **1–240/240** | Serialized worker operation/admission, native post-COMMIT receipts and publication; local retired token-cache cleanup cannot reverse authoritative revoke; an uncertain commit must be rediscovered rather than blindly replayed (78–221). |
| `device-pairing-mutation.worker.ts` | `50face6d5f32ef58cbfe23cef7f19ccdb2348118` | **1–26/26** | Transaction admission and separate commit-stage facts, fail closed outside worker transaction. |
| `device-pairing-identity.ts` | `61869ba59284d2322fa334dde5a5028fdcf0aa85` | **1–199/199** | Tokenless legacy roles deny; active roles bounded by paired approval; key/token-generation + approved-scope recheck; node-specific generation excludes unrelated operator upgrades (63–198). |
| `device-pairing-state.kernel.ts` | `d5714607a7ad69b27fe142cb38fc162f7d734b2a` | **1–70/70** | Pending request TTL/role/scope set normalization; node-surface pending has distinct lifecycle. |
| `device-pairing-token-utils.ts` | `7bb56b799c5945489f60520329d67ea18c06ce75` | **1–63/63** | Fresh role token issuance/rotation and redacted status projection. |
| `device-pairing-tokens.kernel.ts` | `5c61ce1a12a816dd5c147a91f66a0f1f837051d4` | **1–249/249** | Role/approved-scope bounded issue/rotate, revocation timestamp, node-generation invalidation; caller scope check (58–248). |
| `device-pairing-core.kernel.ts` | `3e86654e35d51639500179b66e281a5b17517f6f` | **1–451/451** | Repeated subset request retains original requestId and order while refreshes only TTL (38–189); rejection revokes linked bootstrap (192–210); remove device/pending/APNS (213–235); guarded silent sibling prune excludes owner/bootstrap/legacy/recent/connected (237–317); remove node role drops node surface (319–394); stale presence fencing (430–450). |
| `device-pairing-dispatch.worker.ts` | `26b447c56db8318c92dcc6f47c11ebc7c85d985a` | **1–102/102** | Transactional before/after pairing snapshot, revision and deferred commit receipt (30–101). |

**Source paths fetched but not fully semantically reviewed:** `src/infra/device-pairing-node.records.ts@fdf8b81e2bb38e68a32ecd4df065c6553696e2c1` (267 lines), `device-pairing-store-readonly.ts@a2541c87160e7acfffd96374dde710bba7c186ea` (166), `device-pairing-publication.ts@a8d9e7bcbde564ebf8d08cd00ca7fd8e6cf97666` (261), `device-pairing.types.ts@5a7413d40f0a96fff15da5588ecee53f427b25f7` (164), `device-pairing-row-mapping.ts@8af0a7638a6e2c23300323043935c549cb77739d` (247), `device-pairing-core.types.ts@2cc2a8adfc6b75ad9faf21506e1f8a1dbd07751f` (124), `device-pairing-read.types.ts@d5d06829a860925a467547fd626eb18ed1a3950e` (64), `src/shared/operator-scope-compat.ts@78f8a3f4861c44f447781b32f0fa8aa22761b589` (105), `src/shared/device-bootstrap-profile.ts@bed097a75b02ba623153cd36ca023d44d277b960` (250), `src/shared/device-auth.ts@0fe497d4367619e897bd126e06b91eb42f89f1d8` (35), `src/state/openclaw-state-db.ts@adb2af202d485fc4d98fb405935921da8efd2689` (734), `src/infra/sqlite-worker-operation-admission.ts@90f03a25678f00a88489c13392e19dbb09df6a40` (740), `device-pairing-core.worker.ts@fb60cb2a473a26e1388ea59cc4be840fd33f3110` (103), `device-pairing.ts@ded452595bb22cdd5582d06cc7fe2ff9d4d6b609` (283), plus all other local/third-party direct+recursive imports, schema/generator/tests, `packages/gateway-client`, `packages/gateway-protocol`, and `@openclaw/normalization-core`. Their import/exports survey does **not** earn read-complete or closed authority.

Gateway root blobs remain `src/gateway/server/ws-connection/message-handler.ts@9a733a06b3cf7c1099548008feb65a95af24e331` (749 lines), `connect-session.ts@ec6e6d6064679681d68ffc25684f5a03629c65bc` (703), `connect-admission.ts@f55361c35e36130c420ffa64c42817bedffd5fc2` (374); this round re-fetched exact files and enumerated 29/34/17 import-declaration occurrences respectively, **not** a complete recursive lock or reviewed behavior oracle. Prior §6 source anchors/intervals are not silently upgraded.

### 7.3 Proposed falsifiable source oracle extensions (NOT run/passing)

| ID | Manipulated boundary | Required frozen-source oracle assertion |
| --- | --- | --- |
| OCRM-GW-PAIR-O10 | Pending subset retry vs same deviceId role/scope escalation; queue-order attack | Subset retry preserves requestId/original ts; escalation creates superseding request, old approval cannot apply, TTL refresh does not bump approval rank. |
| OCRM-GW-PAIR-O11 | Same deviceId, different public key and trusted ingress autoapproval | Installation repair/key squat must remain manual; only verified same-key trusted ingress and authorized operator-only automatic branch allowed. |
| OCRM-GW-PAIR-O12 | Caller missing operator scope, bootstrap extra role/scope, fabricated role token | Forbid before token mint/storage; published effective roles and token scopes never exceed durable approved baseline. |
| OCRM-GW-PAIR-O13 | Revoke token/role or rotate node token while connect approval awaits | Live generation/token/approved surface revalidated; stale session/presence/command is denied, node bins/session host cleared, no cache clean-up resurrects authority. |
| OCRM-GW-PAIR-O14 | Begin reconnect cleanup, unchanged pending reconnect, delayed cleanup or approval | Only still-current matching claim may finalize; newer reconnect invalidates older claim; fresh unrelated pending request cannot be deleted. |
| OCRM-GW-PAIR-O15 | Commit succeeds but response disconnects, publish callback fails, cache token cleanup fails | Durable write remains authoritative and is discovered/reconciled; no blind duplicate write or cached regrant; commit receipt/current revision govern publication. |
| OCRM-GW-PAIR-O16 | Bootstrap token expired/replaced/other device; cloud-worker hello lost/confirmed | Exact token+device+live expiry verified inside transaction; uncertain delivery remains retryable only within bound grant; confirmed delivery atomically retires credential; stale setup cannot confirm. |
| OCRM-GW-PAIR-O17 | Owner-approved, bootstrap-approved, legacy and silent records across silent sibling prune | Privileged/unknown-provenance records never silently pruned; protected or just-connected siblings survive and pending/APNS clean only on legitimate removal. |

### 7.4 Fabushi production consumer and state-owner investigation at target parent H

| Exact target path | Git blob SHA | Observed reachability and limit |
| --- | --- | --- |
| `frontend/src/main.tsx` | `ccb3e932cb9b017742c7d6a6ecb77c2faf5814d7` | Real production renderer mount entry. |
| `frontend/src/production/ProductionRenderer.tsx` | `af8a83a8dd7cf1444474889ce3e64fe796a7d458` | Targeted intervals **710–900, 2600–2800 / 4457**: `createCoordinatorClient(coordinatorPort)` (721), typed `client.call('sendPrompt')` (870–872), Host-owned Human transcript refresh and `isAgentNetworkEnabled`; UI projection and client request, not Gateway auth/database writer. Remaining file not fully read. |
| `frontend/src/production/coordinator-client.ts` | `f120b9f600861560409d9a9cac6487cb5ec79e63` | Survey 194 lines: claimed renderer port, typed requests/events and rejection on transport close; does not own business data. |
| `source/electron-preload/coordinator-port-bridge.ts` | `df5be4813dc8068133db14e13916c3c7d31d64ce` | Survey 81 lines: single renderer port claimant and transfer broker; not a separate auth owner. |
| `source/electron-main/entry.ts` | `3c877893f5f6478251587a1130e08328c9cf15f7` | Production composition uses `composeElectronProductionCoordinatorBindings` and `createElectronProductionCoordinatorBinding`. |
| `source/electron-main/coordinator/production-root-provider.ts` | `24e924f3d52d41a0900314aa7c21d98804a9dc19` | Survey 154 lines: account ports + `createProductionCoordinatorGatewayBinding`, not proven OpenClaw pairing. |
| `source/electron-main/adapters/coordinator-gateway.ts` | `f2d6f2577b1500958e98a939c8b65dbad2da09be` | Survey 80 lines: existing Grok/Fabushi coordinator Host gateway descriptor/connector, not full OpenClaw WS auth. |
| `source/node-agent-coordinator/src/main.rs` | `c822c07dff3027a22a658d1ece6aefe919691365` | Targeted callsites around dispatch and `sendPrompt`, Gateway discovery/stream; no claim full 2990-line read. |
| `source/node-agent-coordinator/src/gateway/gateway_request_dispatcher.rs` | `38ca19db76493f9c2bca8862b484d61b7e74f47c` | Previously full reviewed: bounded Coordinator POST `/api/{method}` into existing Host. |
| `source/host/src/gateway_server.rs` | `1ce506ae0bc75ee96418e55eb64a935dd7865ac0` | Previously full reviewed: HTTP/TLS `/api`, Bearer/Origin and events; **not** OpenClaw WS pairing/role/scope equivalent. |
| `source/host/src/host_gateway_api.rs` | `e93659285baf4903d3c97cf85629864341ad4126` | Previously full reviewed: Host method-to-domain owner router; not a device-pairing persistent state owner. |
| `source/host/src/network_enrollment_owner.rs` | `dac2ea3dfa439570de57722c23f583b160f15af9` | Selected review/symbols of 714 lines: existing Rust device-network enrollment account/device credential and durable reconciliation owner, **not proven** OpenClaw pairing owner. |
| `source/product/fabushi/fabushi-official-device-registry.ts` | `461ec329caf850c5b494db38c62a2ed88ef5f140` | **Full 1–330/330**: separate current Fabushi companion App Agent Surface registering over authenticated `wss://fabushi-mcp.ombhrum.com/agent`; per-session `deviceId/sessionId/accessToken` guard before and after calls, six typed `fabushi.app.*` tool names, bounded/redacted results. This external registry is **not** Host `/api` Gateway, not native OpenClaw device pairing; its creation/packaged entry and Rust business authority still require independent tracing. |

Observed **partial** direction: production Renderer → typed transferred Coordinator port → Coordinator HTTP Gateway dispatcher → Host `/api/{method}` → existing Host domain method owner → SSE/response projection. Fabushi App Agent Surface companion is a distinct external integration route, not evidence that all OpenClaw node/pairing contracts have a unique Rust writer. **CLI/TUI real entry/caller still not resolved**; code search was not indexed and a null search result is not proof of absence. No new Rust pairing store or second Gateway process is approved; Host Gateway transport and existing Rust enrollment owner are candidates whose responsibility boundaries need separately reviewed authorization/storage/consumer evidence.

### 7.5 Gate status and explicit remaining blockers

- `GW-AUTH-01`: four requested pairing files and eight tightly related pairing kernel/worker files read in full; additional direct and recursive schema/DB/protocol/Gateway dependencies and tests **unread/unlocked**, so scoped G01=false.
- `GW-AUTH-02`: Renderer and separate companion consumer identified with immutable path/blob anchors; actual CLI/TUI path, complete Host/auth/pairing state ownership, all callers/restart/account-switch semantics and independent reviewer **pending**. Scoped G02=false.
- `GW-AUTH-03`: oracles O10–O17 are proposed, **not executed** or independently accepted; §11.1 unit_authority_ready=false. Do not modify production Rust, advance ledger/RTM status, or upgrade any completion flag.
- Independent subagent tool status check failed with `NOT_FOUND: Tunnel-client has not been seen for 300 seconds` in this round. No independent worker/reviewer launched; continue first-party GitHub read-only work without claiming parallel review.
- The previously verified Actions run `39751237227` and artifact `12413482070` (sha256:`eee0abd63e3cb9e22d2e8144e60883f79676c689033b9ecd1107b11bfb9d9543`) apply to parent H's P0 tracked-tree/structural assertions only. This dossier update is a new evidence carrier H2 requiring its own exact-target proof; neither run nor artifact establishes scoped pairing admission.

**Next read frontier:** full `device-pairing-node.records.ts`, `device-pairing-store-readonly.ts`, `device-pairing-publication.ts`, token-policy, row mapping, pairing state schema, Gateway `message-handler`/`connect-session`/`connect-admission` transitive packages/tests and target CLI/companion registration callsites. Preserve frozen hashes and explicit read intervals; no production code or acceptance mutations until scoped positive source, owner and independent review evidence.
