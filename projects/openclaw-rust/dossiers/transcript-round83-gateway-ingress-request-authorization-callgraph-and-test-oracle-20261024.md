# OCRM Round83 — Gateway ingress request authorization call graph, generator identity, pairing persistence oracle — 2026-10-24

Task `3dd43a37-9f36-4b7a-adac-dfcc87584523`, report round 75. Additive, read-only source audit evidence. No Actions, build, generator, tests, production edits, pairing/Box/Gateway mutation, ledger/RTM/flag mutation, or claim of ACCEPT.

## Authority and write lease

- Frozen upstream: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`; prior frozen tree identity `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`. All source reads in this dossier use that exact commit.
- Target PR #169 was freshly read: OPEN, unmerged, base `main`, head `b5db35fa272c83833685d70a730b5f8285fa1176`. This is target context only, not the audit branch.
- Audit branch `ocrm/round20-snapshot-lifecycle-audit-20261023` was freshly re-read immediately before publication: `c15810fb3687ac89361603fb0057903a889d6a0c`. Publish as a child with CAS expected SHA, force=false. No force update.
- OCRM source authority/spec and root AGENTS were read at carrier `c15810fb3687ac89361603fb0057903a889d6a0c`: root AGENTS blob `e6a1f21ba5206cdc807b6f9738dad2cec2942100`; SOURCE_OF_TRUTH `4b9bdfe745da026ca7ebbca9ea82127d42017f0f`; OCRM spec `52a0202af8e1ea10e42f8ff8f5913e8ec93a4776`; module map `25e4e8258ac45cf72eea704fdceedd0820622a51`; P0 task `9757e8cc1c8968c10f50f240d799854ec47357d5`. No nested AGENTS was present at `projects/AGENTS.md` or `projects/openclaw-rust/AGENTS.md` at that carrier (404).

## Exact source identities and complete target-file reads

The following three target files were fetched in full (line counts below are the returned complete-file lengths; SHA is the exact Git blob identity):

| Frozen upstream path | Exact blob SHA | Complete lines | Primary owner contract |
|---|---|---:|---|
| `src/gateway/server/ws-connection/connect-session.ts` | `ec6e6d6064679681d68ffc25684f5a03629c65bc` | 703 | `attachAuthenticatedGatewayConnect(context,state)`: attach lifecycle and only publish client after current auth/profile/pairing fences |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.ts` | `965068f52f6abd5945d0cd5f0e3ac35ae5c31fc4` | 568 | `createGatewayAuthenticatedRequestDispatcher(params)`: authenticated wire request lifetime, policy fences, response, scheduling and typed dispatch |
| `src/gateway/server-methods/request-authorization.ts` | `a6230b631de50d0ad282e108d0af5e62717cfcd2` | 661 | `authorizeGatewayRequestPreDispatch(params)`: role/scope/profile/startup/session authorization before handler side effects |

Separately re-acquired exact identities:

| Path | Blob SHA | Size / role |
|---|---|---|
| `scripts/protocol-gen-swift.ts` | `9cc28f4931c069709ad0c63a5f72700e88746e4f` | 873 lines; Swift protocol schema/code generator |
| `src/state/openclaw-agent-db.generated.ts` | `129c39cb3f968efd0d6a9ae28143ce49d842d2c9` | 2 lines; re-export shim to `.artifacts/kysely/openclaw-agent-db.generated.js` |
| `src/gateway/server/ws-connection/message-handler.ts` | `9a733a06b3cf7c1099548008feb65a95af24e331` | 749 lines; WS entry, connect phase owner, preauth queue and cleanup |
| `src/gateway/server-methods.ts` | `5bebd34bd2b3e5765947fb58a5b71be98588c10a` | 604 lines; canonical server-method dispatch |
| `src/gateway/server-methods/method-authorization.ts` | `a4dda8be141ad623d9e2e668f51e2e6bf2227cd3` | 139 lines; method/role/scope authorization helper |
| `src/gateway/server-methods/session-mutation-guards.ts` | `6c304c09080c1eee4c10b4c6c4a10f5e381268df` | 569 lines; mutation authority binding and commit-time fence |
| `src/gateway/server-methods/gateway-client-identity.ts` | `d7bf828d12d055a669ed9bc6257523559bf1c960` | 166 lines; authenticated-profile admission |
| `src/gateway/server-methods/session-sharing-authorization-async.ts` | `f859d49725b47801d5de6bda6fba8ac4283a3945` | 339 lines; async session mutation authorization |
| `src/gateway/session-sharing-preparation.ts` | `7f015c63748f7099a8d91264451dc525e5494f87` | 607 lines; session facts/routing preparation |
| `src/gateway/session-access-authority.ts` | `569c9aadf2523081367967a01557716abf2e234e` | 466 lines; prepared session access authority |
| `src/gateway/session-row-prepared-read.ts` | `1e863e50a03aebd18868366024c38771bd0faf8d` | 235 lines; exact prepared session-row reads |
| `src/gateway/session-row-projection-access.ts` | `2c92f1c9e353f7cca73d989707f9a97188aa5ef5` | 45 lines; existing projection owner access |
| `src/gateway/session-method-policy.ts` | `27af8123618ebbb599b330f7d413a86799926d4f` | 205 lines; target-method classification |
| `src/gateway/session-request-agent.ts` | `ebaec9f437e5997a9853bd03defa1d8547b44a83` | 217 lines; canonical agent/session routing resolution |
| `src/gateway/session-sharing-target-input.ts` | `30abfaae66b37523ce5743760eca47fc7d052bb6` | 351 lines; normalized mutation target extraction |
| `src/infra/device-pairing-node-state.ts` | `4cc23766efb819a207bf3b04c596d974a98d93fa` | 78 lines; capture/current check for pairing generation |
| `src/infra/device-pairing-store.ts` | `b7592120467b67ab19011d3ce3eea055559adf25` | 649 lines; durable pairing/bootstrap store |
| `src/infra/device-pairing-row-mapping.ts` | `8af0a7638a6e2c23300323043935c549cb77739d` | 247 lines; typed row encode/decode for paired/pending/bootstrap/completion |
| `src/infra/device-pairing-publication.ts` | `a8d9e7bcbde564ebf8d08cd00ca7fd8e6cf97666` | 261 lines; publication snapshots and node-snapshot invalidation |
| `src/infra/device-pairing-worker.ts` | `190f6301ff4a865992d8bd1c5f3ddf4aff912ca5` | 240 lines; single mutation/read snapshot worker boundary |
| `src/infra/device-pairing-core.worker.ts` | `fb60cb2a473a26e1388ea59cc4be840fd33f3110` | 103 lines; operation-to-kernel registration |
| `src/infra/device-pairing-dispatch.worker.ts` | `26b447c56db8318c92dcc6f47c11ebc7c85d985a` | 102 lines; mutation dispatch/admission/commit receipt |
| `src/infra/device-pairing-mutation.worker.ts` | `50face6d5f32ef58cbfe23cef7f19ccdb2348118` | 26 lines; mutation admission context |
| `src/state/openclaw-state-db-schema-helpers.ts` | `6ffbaf0e6f53f3dd58f71e3f6ec9f7422f2d2e67` | 202 lines; canonical DB schema inspection/column guards |
| `src/state/openclaw-state-schema.sql` | `140a0771581c95bd193c402bd02b99224730cf6b` | 2,760 lines; canonical shared-state SQL schema |
| `src/infra/sqlite-post-commit.ts` | `a203acefc5d28361a24a620fd132c8f264e8f4e9` | 259 lines; transaction-scoped publication staging |
| `src/infra/sqlite-worker-state-context.ts` | `eb09d9cff326e02502f34b4a89fda0e506fd549b` | 146 lines; worker database identity/schema admission context |
| `src/state/openclaw-state.worker.ts` | `e55cc991563c25cb98d9044089beaeb6f01ec39b` | 528 lines; shared-state worker backend |
| `src/state/openclaw-state-worker-context.ts` | `5600eb0b0734e01b4b09fae5fa90c404dc376c7f` | 105 lines; state worker context capture |
| `src/state/openclaw-state-worker-store.ts` | `2ae5bde9c3498bc124bcd48335ba585214d863f4` | 138 lines; operation execution/lease owner |
| `packages/gateway-protocol/src/schema/protocol-schemas.ts` | `3577b4b106bd2a8672b5db0d998a6d396a7ef339` | registry composition consumed by codegen |
| `packages/gateway-protocol/src/schema/protocol-schema-selection.ts` | `ecd99693aa825c9555a0a33329df0c357d6bd553` | public schema selection and exclusions |
| `packages/gateway-protocol/src/schema.ts` | `68b28dd05d1a830c6db8d1d29e3ef928b6f74921` | canonical schema barrel |
| `packages/gateway-protocol/src/schema-modules.ts` | `a3a87e9effaa670d5201a9a8e4423b92c0ca00bf` | schema owner-module barrel |
| `packages/gateway-protocol/src/schema-contract.ts` | `8498d844b36c395555692351064933edfce8c4a8` | optional/null type contract |
| `packages/gateway-protocol/src/schema-types.ts` | `d2f54a4ce852aa4eef8c6310c4a305c940762543` | type-only schema exports |
| `packages/gateway-protocol/scripts/native-codegen.ts` | `6171dbdde2a202a838e7894d3413985bffce2a45` | native model generator/checker |
| `scripts/protocol-gen.ts` | `73c488c4545ef94c4dd7e84c57eeae0594d6587e` | published protocol schema generator |
| `scripts/lib/protocol-schema-document.mts` | `dc52f0a1dcccfaf1fdaffd8046d1e6cb4b4c7f6f` | schema envelope/required definitions/frame mapping |
| `scripts/protocol-gen-kotlin.ts` | `788b21517c4c2a88007febcabb3be047bd70f494` | Kotlin protocol generator |
| `packages/gateway-client/src/request-error.ts` | `not re-fetched in this round` | Direct import of dispatcher; exact identity remains a closure item |
| `packages/gateway-protocol/src/client-info.ts`, `src/gateway/auth-policy.ts`, `src/gateway/device-revocation.ts`, `src/gateway/expected-profile.ts`, `src/gateway/ingress-attribution.ts`, `src/gateway/operator-access-policy.ts`, `src/gateway/server-request-entry.ts`, `src/gateway/server-shared-auth-generation.ts`, `src/gateway/stale-install.ts`, `src/gateway/server/ws-connection/request-start.ts`, `src/gateway/server/ws-connection/request-diagnostics.ts` | `direct import identities not all re-fetched in this round` | Required recursive dependency closure remains open |

## Caller/owner graph confirmed by complete source reads

1. WebSocket ingress is `attachGatewayWsMessageHandler` in `message-handler.ts`. First frame must be a valid request frame with method `connect` and validated `ConnectParams`; malformed/non-connect first frame fails closed. Pre-auth payload is bounded; pipelined frames are queued under a hard cap and released only after hello delivery. Handshake phases run authenticate → device authorization → attach. The same outer handler owns the pending node-pairing cleanup claim and releases it on exceptional paths.
2. `attachAuthenticatedGatewayConnect` in `connect-session.ts` prepares node session and, for node role, captures authenticated pairing identity/token and current pairing generation before registration. It resolves authenticated profile and identity-derived scopes; intersects operator scopes with role policy; rejects closed sockets, untrusted runtime identity tokens, stale node pairing generation, auth-policy changes, failed receive handoff, operator-access failure, and stale profile before `setClient`. Only after registration/binding does it mark connected, transfer receive handoff, publish presence/node registry state and send hello. Node presence and node registry are downstream projections, not pairing authority.
3. `createGatewayAuthenticatedRequestDispatcher` is the authenticated request consumer. It validates the request frame, rejects stale client/auth policy, captures device revocation and remote ingress source, holds request/client authority through async work, fences shared gateway generation and current committed grants, prepares expected profile binding, suppresses responses when authority becomes stale, applies bounded operator request starts, preserves node lifecycle dispatch ordering, and binds mutation authority into `handleGatewayRequest`. Disconnect cancellation is deliberately method-specific; ordinary mutations may outlive the socket, so disconnect is not proof that side effects rolled back. Credential-invalidating methods are serialized by a barrier; its completion resolves even on thrown/early-return paths.
4. `handleGatewayRequest` in `src/gateway/server-methods.ts` is the actual runtime bridge. It owns request envelope and method registry composition and invokes `authorizeGatewayRequestPreDispatch` before handler dispatch. This is the concrete caller relationship between authenticated transport and request authorization.
5. `authorizeGatewayRequestPreDispatch` first applies registered method role/scope authorization in plugin-registry scope, upload policy, authenticated profile requirements, expected-profile and preparation freshness, then startup-unavailable/agent-database admission. It resolves chat-send routing, session targets and prepared projection facts; obtains asynchronous session-mutation authorization or typed session-access authority; verifies node connection still matches current pairing state; and finally repeats authorization/freshness checks before returning an authority object. Temporary worker/projection reads are facts, not a second writer. Mutation commit fences are carried by `session-mutation-guards.ts`.
6. `device-pairing-store.ts` owns durable paired/pending/bootstrap/setup-completion records and pairing revision. `device-pairing-row-mapping.ts` encodes/decodes typed rows. `device-pairing-worker.ts` is the worker boundary and delegates operation execution through `openclaw-state-worker-store.ts`; `device-pairing-dispatch.worker.ts` gates mutation admission and defers commit receipt; `sqlite-post-commit.ts` stages publication only for successful transaction commit; `device-pairing-publication.ts` provides published pairing snapshots and invalidation. A cache, hello frame, presence row or node-registry entry cannot become an alternate grant writer.
7. Swift generation is derived from `ProtocolSchemas`, error-code definitions and protocol version constants; its own helper imports include `scripts/lib/protocol-codegen-schema.ts`. Generated source `src/state/openclaw-agent-db.generated.ts` is only a re-export shim; the authoritative generated schema file is the hidden `.artifacts/kysely/openclaw-agent-db.generated.ts` (not yet independently fetched in this round). Therefore the shim SHA is not sufficient to prove the generated schema identity.

## Test-to-production-path mapping and negative oracle

These are exact source test blobs discovered/read as source only. No test was executed; these are candidate oracles, not passing evidence.

| Test path / exact blob SHA | Production path under test | Negative oracle / falsifier |
|---|---|---|
| `src/gateway/server/ws-connection/connect-device-pairing.test.ts` — `e2f67985cd396c649ccc5034bcaf303fde5f1c64` (1,023 lines; first 220 lines re-read) | message-handler → authenticate → `authorizeGatewayConnectDevice` → pairing admission/token issuance → `attachAuthenticatedGatewayConnect` | Mutate key/token/pairing revision during an await; stale generation must reject before client registration, presence/node registry, token grant or hello-ok. Invalid device or scope cannot gain a session exemption or widened grant. |
| `src/gateway/server/ws-connection/connect-policy.test.ts` — `9dbb41f3e3ba6296ca474df7d602277b1357b352` | connect admission/profile policy → connect-session final policy check | Change auth policy/origin/profile while parked; no client registration or hello; no stale grant retained. |
| `src/gateway/server-methods/request-authorization.test.ts` — `49df8814158949bf3dc14ac57e0c0314db1bbf3a` (120 lines) | `handleGatewayRequest` → `authorizeGatewayRequestPreDispatch` startup/agent DB admission → handler | Pending/unavailable DB target must return retryable/typed refusal before mutation handler; a valid unrelated agent request must not be blocked by another agent's pending DB. No silent bypass or false handler call. |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.abort.test.ts` — `9e70164d215f2cbc8c0e0e9f1dd96a2d229e3adc` | dispatcher request controller/access signal → request entry → server methods | For disconnect-cancellable methods, abort is propagated and response/authority cleanup settles once; for non-cancellable mutation, socket close must not be misrepresented as rollback or trigger replay. |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.mutation-authority.test.ts` — `e7984eec01af2cc7e1dec386c86974f001386283` | dispatcher → `bindWebSocketRequestMutationAuthority` → handler mutation commit fence | Revoked auth/profile/shared generation between admission and commit must deny or fence mutation; stale commit must not pass using only earlier authorization. |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.policy-close.test.ts` — `e9914732be2214645fc7ddaea1095890fe0a5f77` | policy response registration and client invalidation | Invalidated client/current policy suppresses unauthorized response/mutation continuation and closes with correct policy reason; no stale ingress remains usable. |
| `src/gateway/server/ws-connection/authenticated-request-dispatch.lifetime.test.ts` — `4811660a4a11de4799bcf544965450e825cdc871` | request entry lifetime / client retain/release / disconnect | Every return, throw, timeout and cancel releases entry/client authority exactly once; a released request cannot later start a handler. |
| `src/infra/device-pairing-store.test.ts` — `205d5a9ab1f59b0ec2fc1b6762844f02eecf12cb` (180 lines re-read) | row mapping + DB store + worker context + pairing revision snapshot | Round-trip row values and revisions; paired row and revision are a consistent snapshot; missing schema/column or invalid data cannot be treated as valid authorization. Transaction rollback must not publish cache/snapshot changes. |
| `src/infra/device-pairing-publication.test.ts` — `6d9f9e74761cfd01825462d3b6824be94d1da9b2` (538 lines) | state DB read admission → published pairing binding → cache/node snapshot invalidation | Before commit, readers see only prior committed authority; after commit, invalidation/publication reflects the new revision; a stale cache cannot restore revoked device access. |
| `src/infra/device-pairing.ingress-approval.test.ts` — `c04fdbcace777ef0a5c937995292316a1110e259` (100 lines) | ingress approval policy → durable pairing admission | Changed key/invalid policy/merged node-operator path must not autoapprove. A denied candidate produces zero pairing mutation/token side effect. |
| `src/infra/device-pairing-approval-policy.test.ts` — `b8edee7f829134a211fbb2c91018fad7c72edca8` | pairing approval policy → mutation worker | Policy must not widen grants for untrusted ingress; no autoapproval when source trust/key conditions are false. |
| `src/gateway/server-methods/sessions-placement-authorization.test.ts` — `40ac8b84517a5a986bd0f4f383f5b3dbdb625acd` | request authorization → session mutation target routing/projection | Cross-agent or stale session placement cannot authorize against a display row; prepared exact row and persisted target identity must agree before operation. |
| `src/gateway/server-methods/sessions-mutations.owner.test.ts` — `2e488f070803c33dd7a4e021ede6911d55b81977` | session-sharing authorization → session mutation owner | Owner/profile changes between preparation and invocation invalidate the mutation; no alternate writer fallback. |
| `src/gateway/server-methods/sessions-read-visibility.test.ts` — `9301f1162e06969552ef440e94b80b589973a17d` | authorization scope → session row projection/read consumer | Private/inaccessible session must not leak via list/read projections even when a caller supplies a known key. |

## Scoped oracle set

- `GW-INGRESS-O10` handshake/caller: invalid first frame, invalid connect schema, oversized preauth frame, or preauth queue overflow never reaches authenticated registration.
- `GW-INGRESS-O11` attach fencing: role/scope/profile/auth/pairing generation changes across awaits cause fail-closed refusal before `setClient`, presence, node registry, capability-token publication or hello.
- `GW-INGRESS-O12` dispatch freshness: stale committed grant, device revocation, remote-control ingress, profile binding or shared auth generation denies invocation/response; no unauthorized side effect.
- `GW-INGRESS-O13` lifecycle: cancel/timeout/close does not infer rollback; request entries, retained client refs, pairing cleanup claims and barriers settle exactly once.
- `GW-INGRESS-O14` single pairing writer: DB transaction/revision is authoritative; post-commit publication only, row mapping loss/invalid enum/schema drift fails closed, cache/presence/hello are never authority.
- `GW-INGRESS-O15` generator/schema: changed schema owner, optional/null distinction, enum/error literal, version constant or generated DB schema must invalidate stale generated output and fail exact schema check.
- `GW-INGRESS-O16` source closure: any direct/dynamic/transitive import with missing exact blob SHA, any unresolved `.artifacts/kysely/openclaw-agent-db.generated.ts`, lock/toolchain or consumer is an open dependency; do not mark scoped G01 closed.
- `GW-INGRESS-O17` subject/evidence: run/receipt against anything other than the declared exact target commit/tree and exact source closure is non-passing.

## New evidence vs duplicates; unresolved closure

New this round: complete semantic reads of the three requested target files; exact re-acquisition of the Swift generator and generated DB re-export shim; caller path from message handler through authenticated dispatcher to server-method dispatch and pre-dispatch authorization; explicit test-to-production-path mapping and negative oracles; additional exact identities for session mutation authority and pairing row/publication/worker/schema lifecycle. This is additive to Round82 and does not restate its nine initial oracle rows as newly passed evidence.

Still not closed: every recursive import of the three target files has not yet been fetched and identity-pinned one-by-one; all dynamic imports from the request handler and every transitive helper under those imports remain a frontier. The exact hidden generated file `.artifacts/kysely/openclaw-agent-db.generated.ts`, generator helper, complete schema/migration owner set, workspace lock/toolchain and full tests/callers remain to be fetched/reconciled. Test mappings are source-based candidate oracles; none was run. Direct imports such as client-info, protocol index, retry helpers, auth policy, device revocation, expected profile, ingress attribution, operator access, server request entry, shared generation, stale-install, request diagnostics/start, plugin runtime, node registry, voice-wake, profile and GitHub identity helpers still need exact identities and recursive traversal. No full import closure is claimed.

## Verdict / gate

- scoped G01: NOT ACCEPTED (recursive import/schema/generator/test closure incomplete).
- scoped G02: NOT ACCEPTED (complete responsibility map and independent review not yet available).
- independent owner/oracle verdict: NOT AVAILABLE.
- `unit_authority_ready=false`; OCRM Spec §11.1 NOT ADMITTED.
- No Actions were run, no production code was modified, and no ledger/RTM/flags were upgraded. Existing Host/Coordinator/Session/Transcript owners remain candidates, not approved new authority. Pairing remains with the upstream existing durable DB/worker owner; no second store or writer is justified by this evidence.
- Continue immediately with remaining exact recursive import identities and semantic caller/owner reads; keep post-readiness Box child death/recovery, typed error preservation, application-error vs transport-down, OCRM-AC01..24 and release acceptance open.
