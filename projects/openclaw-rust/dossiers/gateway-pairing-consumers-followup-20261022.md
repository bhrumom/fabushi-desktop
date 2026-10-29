# OCRM GW-AUTH-01/02: pairing dependencies and production consumers — read-only continuation

**Status: source investigation only; NOT independently reviewed; scoped G01/G02 NOT accepted; §11.1 NOT admitted.**  
**Evidence date:** 2026-10-22. **Scope:** OCRM-M01/M04, `OCRM-UNIT-GW-INGRESS-001`, `GW-AUTH-01` / `GW-AUTH-02`.  
**Frozen upstream:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70`; tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.  
**Investigated target:** `bhrumom/fabushi-desktop` PR #169 at H=`f41cfc74800021cc69160dcd923478987f250870`, tree `0bea6e7811000894c1e9cedeb959c80d1933134a`, base/main `f6eb1758c59d65aba93e5608d7aba445e0bad42b`. H descends directly from `4730a01f3d210daa5eef3d788de4af5f27a41eba`. This document, if committed, is a *later distinct carrier/target* and cannot inherit H's receipts.

This continuation extends `gateway-ingress-scoped.md` §7 without replacing its existing evidence or changing any responsibility flag. Full GitHub text was retrieved across every interval marked complete below, with the frozen path's Git blob SHA. It is NOT a statement that recursive imports, runtime semantics, tests, or dependency closure are complete.

## A. Additional frozen-source files with full-text reads

| Exact path under frozen upstream | Git blob SHA | Inspected lines | Concrete contract |
| --- | --- | --- | --- |
| `src/infra/device-pairing-node.records.ts` | `fdf8b81e2bb38e68a32ecd4df065c6553696e2c1` | 1–150 + 151–267/267 | Pending request/revision identities, normalized approval-surface matching, reconnect metadata equality, public node projection strips auth records |
| `src/infra/device-pairing-store-readonly.ts` | `a2541c87160e7acfffd96374dde710bba7c186ea` | 1–166/166 | Read-only worker snapshot + publication lease; stale publication retries under lock; reads cannot create/migrate shared SQLite |
| `src/infra/device-pairing-read.types.ts` | `d5d06829a860925a467547fd626eb18ed1a3950e` | 1–64/64 | Typed lookup/list/bootstrap reads, per-device binding/commit receipt, before/after revisions |
| `src/infra/device-pairing-publication.ts` | `a8d9e7bcbde564ebf8d08cd00ca7fd8e6cf97666` | 1–145 + 146–261/261 | Cache keyed to DB identity, revision/epoch and COMMIT receipt; blocked or unknown publication cannot authorize |
| `src/infra/device-pairing-row-mapping.ts` | `8af0a7638a6e2c23300323043935c549cb77739d` | 1–127 + 128–247/247 | SQLite reversible optional-field encoding, approval provenance allowlist, unknown setup access → limited, unknown delivery → uncertain |
| `src/infra/device-pairing.types.ts` | `5a7413d40f0a96fff15da5588ecee53f427b25f7` | 1–164/164 | Durable approved roles/tokens and pending node surface revision, setup completion uncertain vs confirmed |
| `src/infra/device-pairing-core.types.ts` | `2cc2a8adfc6b75ad9faf21506e1f8a1dbd07751f` | 1–124/124 | Scope/approval/revoke denials and expected generation types |
| `src/infra/device-pairing-admission.types.ts` | `5ca4d8260d643920adcd0bf10e74c921916774e9` | 1–24/24 | Node-pending revision and typed worker admission facts |
| `src/infra/node-pairing-authz.ts` | `5185bf0088ecf2f915ed0c35971e42a1079ed83e` | 1–31/31 | Node approval requires pairing scope; commandful node needs write; system-run/admin categories require admin |
| `src/infra/node-pairing-surface.ts` | `e7b4fa0c91e4055bfce4cc26edf3474751525191` | 1–57/57 | Declared permissions intersect approved values; caps/commands normalized as sets |
| `src/infra/node-commands.ts` | `1c28c5a03e19203ca40db7faea787c1533563f09` | 1–100/100 | Private worker commands removed from public invocation surface; admin-only browser/file commands |
| `src/gateway/server/ws-connection/connect-node-session.ts` | `51aad1c06c2b8caa84753dfd53c5755767ff10bf` | 1–112 + 113–222/222 | No silent node capability grant merely from trusted CIDR; local opt-out, locality and origin fences; initial-only autoapproval; declared protocol ceiling |
| `packages/gateway-protocol/src/schema-contract.ts` | `8498d844b36c395555692351064933edfce8c4a8` | 1–21/21 | Optional-property schema contract preservation |
| `packages/gateway-protocol/src/schema/devices.ts` | `40aa94ce88d4f64bdc9604f13d491eb4bf4df0f6` | 1–120 + 121–239/239 | Rotate token-delivery facts vs withheld cross-device, separate pairing setup terminal/uncertain semantics, admin-gated full setup capability |

The linked parent dossier independently records full read intervals for `device-pairing-store.ts@b7592120467b67ab19011d3ce3eea055559adf25` (1–649), `device-pairing-node.ts@3e032d10f1b2079a22784e8181d14f2eda8e4946` (1–332), `device-pairing-node-state.ts@4cc23766efb819a207bf3b04c596d974a98d93fa` (1–78), and `device-pairing-approval.kernel.ts@7a563c858c078347587d4bfc6e153be59550393a` (1–382), plus earlier worker/core kernels. Neither this table nor those reads establish closure of all direct/recursive dependencies.

## B. Remaining gateway-source dependency frontier (SHA-locked, **NOT full semantic read**)

- `src/gateway/server/ws-connection/message-handler.ts@9a733a06b3cf7c1099548008feb65a95af24e331` (749 lines): direct imports enumerate `connect-admission`, `connect-auth`, `connect-device-pairing`, `connect-session`, `authenticated-request-dispatch`, node pairing, gateway-protocol, websocket data and diagnostics. Its full recursive closure and oracles are not locked.
- `src/gateway/server/ws-connection/connect-session.ts@ec6e6d6064679681d68ffc25684f5a03629c65bc` (703 lines): direct `connect-node-session`, `connect-hello`, identity, profile, node registry and dynamic `../../server-history-prewarm.js` still require full recursively scoped reading.
- `src/gateway/server/ws-connection/connect-admission.ts@f55361c35e36130c420ffa64c42817bedffd5fc2` (374 lines): direct Origin, role, operator scopes, profile, auth and protocol contracts remain recursively open.
- New enumerated next-stage inputs: `authenticated-request-dispatch.ts@965068f52f6abd5945d0cd5f0e3ac35ae5c31fc4` (568 lines), `connect-hello.ts@45403cf7dac7a9414b05bac43f5d1f0458de8120` (533; targeted 435–533 inspected), `connect-user-profile.ts@72bc221d53cecff121e63be212b992d5650e3b78` (251), `message-handler-types.ts@06ce9ee4bd8c12be664b81de73baff821572a7e1` (207). Source imports of core schema/generators, gateway-client, normalization-core, repository scripts, tests and dynamic registrations are unresolved.
- `connect-node-session.ts` itself has direct dependencies in `node-connect-reconcile.ts`, `node-computer-use-descriptor.ts`, `node-legacy-protocol-filter.ts`, rate-limit policy and plugin surfaces. No false closed flag is assigned.
- No repository-wide or scoped denominator inferred from this partial traversal. All tests/oracles below remain proposals until executed in exact-head GitHub Actions and independently judged.

## C. Fabushi existing producer → bridge → Host ingress (partial production proof)

| Path under target H and blob SHA | Inspected interval | Proven function and ownership limitation |
| --- | --- | --- |
| `frontend/src/main.tsx@ccb3e932cb9b017742c7d6a6ecb77c2faf5814d7` | 1–39/39 | Bootstraps shipping `ProductionRenderer` from preload runtime |
| `frontend/src/production/bootstrap.tsx@63e272d027a0f1f6fb5ea308a2252f121bb46b19` | 1–end of retrieved file | Requires desktop bridge and `coordinatorPort.claim`, mounts React; not permission authority |
| `frontend/src/production/ProductionRenderer.tsx@af8a83a8dd7cf1444474889ce3e64fe796a7d458` | targeted 640–900/4457 | `createCoordinatorClient`; actual `client.call` consumers for agents/create/sendPrompt, automations, search, listeners, plugin auth; broader UI/consumer audit outstanding |
| `frontend/src/production/coordinator-client.ts@f120b9f600861560409d9a9cac6487cb5ec79e63` | 1–194/194 | Request IDs on transferred Coordinator port, pending reject on transport loss; request transport is not durable business owner |
| `source/electron-preload/coordinator-port-bridge.ts@df5be4813dc8068133db14e13916c3c7d31d64ce` | entire retrieved file | One renderer port claim owner; port ownership is not pairing-domain ownership |
| `source/electron-preload/preload.ts@4d4aadb62aacb85cae559c4cb6aa01885a9990b8` | targeted 340–407/407 | Exposes `coordinatorPort`, `sand:coordinator-port-request`/received transferred port |
| `source/electron-main/coordinator/production-provider.ts@3b7591fbaf07cd7a5b13bfda27bca717c8603e01` | targeted 625–682/684 | Trusted webContents/frame assertion precedes port transfer; main IPC owns transfer lifecycle |
| `source/electron-main/coordinator/production-root-provider.ts@24e924f3d52d41a0900314aa7c21d98804a9dc19` | 1–154/154 | Electron coordinator account authorization and port creation composition, not a new Gateway auth store |
| `source/electron-main/entry.ts@3c877893f5f6478251587a1130e08328c9cf15f7` | 1–112/112 | Canonical production service composition calls `startElectronMainProduction` |
| `source/node-agent-coordinator/src/renderer_port_server.rs@ede3e482d6d29847fb56ac1d0f2d77524331f55e` | 1–191/191 | Rust port verifies hello/protocol, rejects duplicate request ids and emits abort on cancel/close; transport state not business completion |
| `source/node-agent-coordinator/src/main.rs@c822c07dff3027a22a658d1ece6aefe919691365` | targeted 2510–2810/2990 | Dispatch of data-port requests either to Coordinator-specialized paths/inference or to `dispatch_to_host`, Host-generation/endpoint fenced HTTP Gateway request, pending reply tracking |
| `source/host/src/gateway_server.rs@1ce506ae0bc75ee96418e55eb64a935dd7865ac0` | earlier linked dossier 1–1244/1244 | Existing Host HTTP/TLS/Bearer ingress; no verified OpenClaw pairing WS admission equivalence |
| `source/host/src/host_gateway_api.rs@e93659285baf4903d3c97cf85629864341ad4126` | earlier linked dossier 1–256/256 | Existing Host method-to-domain owner table; not a plugin protocol method registry |

**Provisional owner election (unaccepted):** existing Host domain writer for Session/Transcript and Host-side method dispatch, Coordinator as session/HTTP supervisor, Electron as trusted IPC adapter, Renderer as presentation-only caller. OpenClaw pairing's worker-owned durable row authorization and revocable generation have **no demonstrated, shipping equivalent Fabushi Rust state owner** from this inspection; thus no new owner or dual writer authorized.

**Cancellation falsifiability:** `ProductionRenderer.callCoordinatorWithAbort` at lines 652–680 rejects its local Promise on AbortSignal; this targeted function does not post a Coordinator Cancel frame. This alone is **not** proof the Host mutation was rolled back (or that every caller requires cancellation). Require an explicit accepted/outcome-unknown + peer-cancellation oracle before any equivalence claim.

## D. CLI, companion and other “Gateway” names are *not* interchangeable

- `source/mahayana/mahayana-rs/mahayana-cli/src/main.rs@a0e0a98f20e7f24eb75dc13ae9bffdf5c4846b6e`, 2305 lines, targeted 515–715: product CLI uses local `with_runtime` / `FeatureHostController` and optional codex-compat commands; no complete inbound Host Gateway client path established.
- `source/mahayana/mahayana-rs/mahayana-cli/src/bin/mahayana-gateway.rs@660eca96a76f1bd0fb2ae05d7b03f60a579f24ca`, 500 lines, targeted 1–180: **distinct stdio JSON-RPC** entrypoint backed by mahayana-runtime, not proof of Host HTTP/WS gateway equivalence.
- `source/mahayana/mahayana-rs/mahayana-cli/src/device_agent.rs@10029b40dcccd21c920b760b5deb234b93af90e1`, 661 lines, targeted 390–525: device agent authenticates to separate remote `wss://fabushi-mcp.ombhrum.com/agent`, carries device generation, lease and heartbeat, rechecks live auth session; not shown as consuming Host Gateway pairing store.
- `source/electron-main/adapters/coordinator-gateway.ts@f2d6f2577b1500958e98a939c8b65dbad2da09be`, 1–80/80: generated Box/GrokBot connector, account-scoped, **not** itself an OpenClaw WS pairing credential owner.
- `native/mahayana-messaging/src/gateway.rs@2c3427851d21d30d9501a28beaf7c0d1c2d6091b` is a separate native messaging WebSocket gateway, not assumed to share OpenClaw pairing semantics. GitHub target `source/` tree `e1b8e684f2646f1c263a9766b3c77c340d1839f3` enumerated 8,708 entries (not truncated), with no **path name containing “companion”** in that prefix; this does NOT rule out differently named, other-repository or dynamically registered companion clients.

## E. Falsifiable oracle obligations (NOT run)

- **GW-PAIR-R01:** stale publication, missing database read identity, older revision or unknown fetch must fail closed (not read-through stale grant).
- **GW-PAIR-R02:** rotate/revoke/approve concurrent with handshake or RPC await must fence on persisted device key, token, role, approved scope, pairing identity/generation and committed revision.
- **GW-PAIR-R03:** CIDR/proxy or matching display metadata alone cannot silently approve node command surface; local-only exception must respect origin, opt-out, first-surface ceiling.
- **GW-PAIR-R04:** node command list/permissions granted are subset of declared and approved current scope, private worker commands never advertised; admin-only commands denied to write-only.
- **GW-PAIR-R05:** reconnect cleanup claim must not delete a newer pending revision; interactive provenance cannot be silently downgraded; changed device key retires previous node generation.
- **GW-PAIR-R06:** cross-device token rotate does not leak bearer; setup delivery `uncertain` remains uncertain without positive confirmation; loss of socket alone never proves refund/revoke.
- **GW-PAIR-R07:** renderer abort/port loss, Host crash or HTTP timeout after an accepted mutation must produce correct accepted/denied/outcome-unknown, reconcile before retry and no duplicate write.
- **GW-PAIR-R08:** target transport, CLI, device agent and other domain transports must be separately verified; shared names cannot be treated as same owner.
- **GW-PAIR-R09:** every positive verdict requires frozen source/recursive dependencies, read intervals, exact H tree, git blob and independent Actions oracle + reviewer verdict. A green structural inventory job cannot satisfy this.

## F. Evidence, blockers and next executable frontier

At target H, GitHub Actions run `39754646399` attempt 1 `completed/success` with `head_sha=f41cfc74800021cc69160dcd923478987f250870`; jobs `source-authority=119743470965` and `persist-verified-inventory=119744023118` success. Artifact `12415028029`, `ocrm-p0-authority-f41cfc74800021cc69160dcd923478987f250870`, GitHub metadata digest `sha256:dee9178d531fecccfc3aed64cc2b94675a75339a0aa597e5d8a4ba012bb1e063`. The run does NOT establish scoped transitive closure, full semantics, G01/G02 independent review, accepted/wired Rust parity, CLI/companion acceptance or release. Any successor commit must be evaluated as a new subject; this note cannot make H green.

Blocker `OCRM-BLOCK-GW-SCOPED-AUTHORITY-001` remains internal missing evidence. Worker inspection via `Chat On Steroids Core agents.status` failed `NOT_FOUND` (tunnel-client inactive >300 seconds); no independent agent or review was started. No user action required on this blocker, no duplicate support email.

**Immediate ready work:** use GitHub frozen SHA to full-read `authenticated-request-dispatch.ts`, `connect-hello.ts`, `connect-user-profile.ts`, `message-handler-types.ts` and follow imports/dynamic sources through worker/schema/generator/test dependencies; expand target packaged Renderer/CLI/companion consumers and explicit credential/persistence owner. Only after complete scoped G01/G02 positive receipt, reviewer acceptance and Spec §11.1 may production Rust be modified. No production code, ledger/RTM, acceptance flags, release or dependency lock is changed by this note.
