# OCRM Round82 — Gateway ingress dependency, schema-generation, and pairing persistence oracle — 2026-10-24

Task 3dd43a37-9f36-4b7a-adac-dfcc87584523, report round 74. Additive read-only audit evidence. No production, pairing, Box, Gateway, ledger, RTM, flags, or workflow mutation; no Actions, build, generator, or test execution is claimed.

## Exact subjects and leases

- Target PR #169: bhrumom/fabushi-desktop@b5db35fa272c83833685d70a730b5f8285fa1176, tree d01637ba8d83cd6c723077555ccf725887f7156e. Live PR read confirms OPEN, unmerged, base main, zero review submissions and zero review threads.
- Audit branch ocrm/round20-snapshot-lifecycle-audit-20261023 was re-read immediately before writing and still resolves to 5ea083df819a9c59aab358d63ec4f02825248e02, tree 16335a52764f5f1d88fc7dcb0740f6b4c252a6e3. This is the CAS expected SHA; publish only as a non-force child.
- Frozen upstream remains openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70, tree 04ccce62f1d74dc8e0c88290ae8ded027b7b0f16. Do not silently rebaseline to moving upstream.

## New exact source reads

| Frozen source path | Blob SHA | Inspected interval / evidence |
|---|---|---|
| src/gateway/server/ws-connection/connect-admission.ts | f55361c35e36130c420ffa64c42817bedffd5fc2 | 1–374/374. Protocol compatibility, startup gating, role parsing, Remote Control UI signed-device and scope ceiling, browser origin, and post-await recheck of role/auth/shared-session generation. |
| src/gateway/server/ws-connection/connect-session.ts | ec6e6d6064679681d68ffc25684f5a03629c65bc | Whole file fetched (703 lines); semantic review partial. Imports pairing node state, role policy, scopes, profile and plugin-node capability owners. |
| src/gateway/server/ws-connection/authenticated-request-dispatch.ts | 965068f52f6abd5945d0cd5f0e3ac35ae5c31fc4 | Whole file fetched; semantic review partial. Authorization and session mutation fences are part of ingress, not an isolated frame parser. |
| src/gateway/server/ws-connection/message-handler.ts | 9a733a06b3cf7c1099548008feb65a95af24e331 | Whole file fetched; semantic review partial. Parses first connect frame, runs authenticate → device authorize → attach, retains pairing cleanup claim across phases, bounds preauth queue. |
| src/gateway/server-methods/method-authorization.ts | a4dda8be141ad623d9e2e668f51e2e6bf2227cd3 | 1–139/139. Existing method owner applies role/current role policy/admin/registered scope checks; stale clients are denied. |
| src/gateway/server-methods/request-authorization.ts | a6230b631de50d0ad282e108d0af5e62717cfcd2 | Whole-file fetch; semantic review partial. Async authorization preparation, startup readiness, session-sharing/mutation target validation and repeated live-authority checks. |

## Schema, generator and package chain

| Frozen source path | Blob SHA | Observed responsibility |
|---|---|---|
| packages/gateway-protocol/src/schema.ts | 68b28dd05d1a830c6db8d1d29e3ef928b6f74921 | Public barrel for canonical schema modules and protocol registry. |
| packages/gateway-protocol/src/schema-modules.ts | a3a87e9effaa670d5201a9a8e4423b92c0ca00bf | Owner-module barrel includes devices, channel pairing, nodes, sessions, plugin approvals, worker admission and other schemas. |
| packages/gateway-protocol/src/schema-contract.ts | 8498d844b36c395555692351064933edfce8c4a8 | Preserves optional-property type distinctions when deriving JSON contracts. |
| packages/gateway-protocol/src/schema-types.ts | d2f54a4ce852aa4eef8c6310c4a305c940762543 | Type-only schema export separate from runtime registry. |
| packages/gateway-protocol/src/schema/protocol-schema-selection.ts | ecd99693aa825c9555a0a33329df0c357d6bd553 | Derives public schema map from owner modules with explicit exclusions. |
| packages/gateway-protocol/src/schema/protocol-schemas.ts | 3577b4b106bd2a8672b5db0d998a6d396a7ef339 | Composes derived, migration and session-placement registries. |
| packages/gateway-protocol/scripts/native-codegen.ts | 6171dbdde2a202a838e7894d3413985bffce2a45 | Generates Swift/Kotlin models and checks model/version/literal consistency. |
| packages/gateway-protocol/package.json | 3cb52a3f3cb2d51faa276313aaa8e0337328abce | Public exports; build runs workspace build and registry type check; prepack generates protocol.schema.json. |
| scripts/protocol-gen.ts | 73c488c4545ef94c4dd7e84c57eeae0594d6587e | Builds published schema from ProtocolSchemas and core method metadata. |
| scripts/lib/protocol-schema-document.mts | dc52f0a1dcccfaf1fdaffd8046d1e6cb4b4c7f6f | Canonical schema envelope; required definitions and req/res/event frame order/mapping are checked. |
| scripts/protocol-gen-kotlin.ts | 788b21517c4c2a88007febcabb3be047bd70f494 | Consumes ProtocolSchemas, protocol versions, core method names and event extraction. |
| scripts/protocol-gen-swift.ts | identity not separately captured in this round | Retrieved in a combined response, but exact identity/full read must be repeated before closure. |

Important distinction: the runtime registry feeds both published JSON schema and native Swift/Kotlin generation. A generator invocation or existing output is not an oracle pass. Schema owner modules, method descriptors, event coverage, generator helpers, workspace lock/toolchain, generated database schema and downstream native consumers are still mandatory closure edges. No generator was executed.

## Pairing persistence owner evidence

Frozen src/infra/device-pairing-store.ts blob b7592120467b67ab19011d3ce3eea055559adf25 was read in ranges 1–220, 221–440 and 441–649 (complete 649 lines).

- Pairing writes use the existing shared-state transaction/worker context. Paired-row mutations bump config_machine_state[devicePairing.revision] atomically.
- Snapshot reads return paired rows and revision from one statement. Cache and node-snapshot invalidation are staged post-commit; pending transactions do not publish authority.
- Paired-device patches write explicit columns in the existing pairing table; whole-state persistence is transaction-bound and related APNS cleanup is explicit.
- Bootstrap consumption rechecks exact token/device binding and expiry in the authoritative transaction. Setup delivery starts as uncertain; confirmation only updates the exact live setup/device row and retires the matching bootstrap token atomically.
- Required recursive dependencies include additive state schema/migration helpers, generated DB types, shared-state transaction owner, row mapping, cache/publication invalidation, worker completion binding, token expiry, SQLite/Kysely, APNS cleanup and post-commit staging. This list is a closure frontier, not proof all dependency imports are closed.

Owner invariant: authorization must be bound to the current persisted pairing row and revision/generation. Cache, hello payload, browser claim or UI state is not authority. This source does not authorize a second pairing store or writer.

Relevant exact test identities located (not run):
- src/gateway/server/ws-connection/connect-device-pairing.test.ts — e2f67985cd396c649ccc5034bcaf303fde5f1c64 (1023 lines).
- src/gateway/server/ws-connection/connect-policy.test.ts — 9dbb41f3e3ba6296ca474df7d602277b1357b352.
- src/gateway/server-methods/request-authorization.test.ts — 49df8814158949bf3dc14ac57e0c0314db1bbf3a.
- src/infra/device-pairing-store.test.ts — 205d5a9ab1f59b0ec2fc1b6762844f02eecf12cb.
- src/infra/device-pairing-approval-policy.test.ts — b8edee7f829134a211fbb2c91018fad7c72edca8.
- src/infra/device-pairing-publication.test.ts — 6d9f9e74761cfd01825462d3b6824be94d1da9b2.
- src/infra/device-pairing.ingress-approval.test.ts — c04fdbcace777ef0a5c937995292316a1110e259 (100 lines); visible tests reject changed-key/invalid-policy autoapproval and merged node/operator autoapproval.

These tests are oracle candidates, not complete test-by-test review or execution evidence.

## Existing Fabushi owner/consumer conclusion (partial)

The existing target dossier projects/openclaw-rust/dossiers/gateway-ingress-scoped.md blob 783fc5da574b13b3e4229d1c08552a63c2be6387 records exact PR-head anchors: source/host/src/gateway_server.rs@1ce506ae0bc75ee96418e55eb64a935dd7865ac0 (HTTP/TLS, Origin/Host/Bearer guards, /api route whitelist, events/bridge; no upstream WebSocket pairing handshake); source/host/src/host_gateway_api.rs@e93659285baf4903d3c97cf85629864341ad4126 (method-to-Host feature-owner routing); source/host/src/gateway_protocol.rs@13b6a5bad55c133ed67e39bc94b7e500808a82a5 (existing typed façade, not proof of upstream schema parity); source/host/src/gateway_config.rs@64af7f6354047fe18b3e94fe629236ee0ade1942; source/host/src/gateway_command_error.rs@a7827b29c7f53899b1f34e7c989849816f4090fb.

Partial live chain remains Renderer → typed Coordinator port → Coordinator HTTP Gateway dispatcher → Host /api/{method} → existing Host domain owner → SSE/response. Coordinator is transport/reconnect consumer; Host retains Session/Transcript and existing feature writers. Pairing persistence is a distinct durable authority in the upstream source. Existing-owner-first is a candidate only; full CLI/TUI, Renderer, companion, remote node, channel and account-switch consumers remain unclosed. Do not add a universal Gateway process, duplicate pairing store or second business writer.

## Falsifiable oracle and exact-subject Actions gate (proposal only)

Unit candidate: OCRM-UNIT-GW-INGRESS-001; scoped requirements GW-AUTH-01/02/03. These are proposed and NOT executed.

| Oracle | Fault injection | Required assertion |
|---|---|---|
| GW-O01 | Missing import/dynamic dependency, stale blob, missing schema/generator/test input | G01 fails closed with exact path/blob/digest; prefix counts cannot substitute for closure. |
| GW-O02 | Missing/invalid credential, spoofed Host/origin, expired ingress grant, unsigned device, excessive scope | Deny before client registration, credential mint, pairing write, hello-ok or business handler. |
| GW-O03 | Change role/auth/shared-session generation, device key/token/scopes or pairing revision while connect is parked at awaits | Post-await freshness check denies before publishing credentials/client or dispatching requests. |
| GW-O04 | Approval/revoke/token rotation, stale cache, commit response loss | Durable row/revision and related effects are transactional; stale cache cannot revive grants; reconcile unknown outcomes rather than replay. |
| GW-O05 | Perturb schema, method metadata, discriminator order, enum literal or optional/null distinction | Published schema and Swift/Kotlin checks fail on exact mismatch; cached output cannot pass. |
| GW-O06 | Core/plugin method collision, attached registry changes, operator.read invokes write | Same immutable descriptor governs authorization and dispatch; unauthorized path has zero side effects. |
| GW-O07 | Disconnect/cancel after possible mutation or hello write failure | Distinguish rejected/committed/uncertain/confirmed; preserve request identity and clean connection/claims exactly once. |
| GW-O08 | Exercise existing Host API and Renderer→Coordinator path during ingress changes | Existing Host business owners and transport/reconnect behavior remain intact; no second Session/Transcript writer. |
| GW-O09 | Run gate against carrier instead of declared PR subject | Receipt rejects unless exact target commit/tree and all source/schema/generator/workflow inputs are digest-bound. |

Gate inputs must pin upstream commit/tree, PR H/tree, recursive imports/dynamic imports, workspace exports/lock/toolchain, schemas/generators/generated outputs/tests and production consumers. G02 must record one owner per durable state and method dispatch, negative oracle cases, independent reviewer verdict, exact Actions run/attempt/job/logs/artifact digests. Missing dependency, stale SHA, skipped oracle, unreviewed owner or subject/carrier mismatch is non-passing.

## Verdict and next frontier

Current scoped G01 = NOT ACCEPTED; scoped G02 = NOT ACCEPTED; independent owner/oracle verdict = NOT AVAILABLE; unit_authority_ready=false; §11.1 NOT ADMITTED. PR has zero submitted reviews and zero review threads. No Actions run, no production code mutation, no pairing mutation and no ledger/RTM/flag upgrade.

Immediate remaining read frontier: complete semantic/caller reads for connect-session, message-handler, authenticated-request-dispatch, request-authorization, connect-device-metadata, connect-pairing-approval-plan, connect-hello and method registry; recursively lock all schema owner imports, package exports, generator helpers, generated DB schema, workspace lock/toolchain and tests; finish pairing row mapping/publication/worker/schema transaction dependencies; resolve CLI/TUI, companion, remote node and channel consumers.

Keep post-readiness managed Box child death/recovery, typed error preservation, application-error vs transport-down separation, OCRM-AC01..24 and release acceptance open. This dossier does not upgrade any responsibility from read to mapped/implemented/wired/verified/accepted.
