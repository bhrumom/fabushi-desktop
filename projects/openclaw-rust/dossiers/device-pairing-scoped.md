# OCRM Technical Dossier: Unit OCRM-UNIT-DEVICE-PAIRING-001 — Device Pairing, Node Enrollment & Network Authority

## 1. Unit Overview & Scope

- **Unit ID:** `OCRM-UNIT-DEVICE-PAIRING-001`
- **Module ID:** `OCRM-M03` (Identity, Credentials, Pairing and Account Scope)
- **Status:** **Scoped candidate admitted into Parity Ledger; 38 audited source files mapped to reviewed; 6 responsibilities registered with implementation/wiring pending; production wiring blocked by OCRM-BLOCK-GW-SCOPED-AUTHORITY-001; acceptance pending**
- **Requirements Traced:** `OCRM-R03`, `OCRM-R04`, `OCRM-AC03`, `OCRM-AC11`
- **Reviewer:** `xuyao@ombhrum.com`
- **Upstream Baseline for this dossier:** `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`), the prior implementation-round lock used for the blob identities below.
- **Latest live candidate:** `openclaw/openclaw@c32556b0cea7f63b33db0ac393affd71ecb0015b` (tree `e08d361d83ca9cae55aa93d49462d991d0a39efc`), discovery-only pending exact-head Actions inventory and dependency closure; this dossier's old blob rows are not relabeled to it.
- **Audit Lineage:** PR #169, `projects/openclaw-rust/dossiers/state-read-snapshot-worker-custody-pairing-round20.md`, `pairing-unknown-reconcile-postgresql-provenance-round19.md`, `transcript-round84-gateway-import-closure-protocol-codegen-pairing-worker-20261024.md`, `transcript-round89-openclaw-device-pairing-sessions-and-config-native-wiring-20261026.md`.

---

## 2. Responsibilities Registered in Parity Ledger

1. `OCRM-RESP-DEVICE-PAIRING-STATE-MACHINE-001`: Device pairing cryptographic state machine (pending -> approved / rejected / revoked).
2. `OCRM-RESP-DEVICE-PAIRING-APPROVAL-POLICY-001`: Operator approval policy, admin privilege gate, and rejection reason logging.
3. `OCRM-RESP-DEVICE-PAIRING-TOKENS-LIFECYCLE-001`: Cryptographic verification token generation, single-use invalidation, and rotation.
4. `OCRM-RESP-DEVICE-PAIRING-JOIN-CODE-001`: Device setup / join code verification, expiration window, and format validation.
5. `OCRM-RESP-DEVICE-PAIRING-STORE-PERSISTENCE-001`: SQLite durable pairing store, row facts mapping, and write generation.
6. `OCRM-RESP-DEVICE-PAIRING-NODE-LIFECYCLE-001`: Node enrollment lifecycle, capability registration, heartbeat pruner, and revocation fence.

The newly read node cluster closes an important owner boundary: `device-pairing-node.ts`
keeps reconnect cleanup claims generation-scoped and refuses stale approval or
cleanup after a newer pending revision; `device-pairing-core.worker.ts` is the
single mutation-operation registry rather than a second pairing store; and
`device-pairing-binding.ts` projects the current identity/generation binding
without owning lifecycle state. The Rust target must preserve these properties
inside `DevicePairingOwner`/the existing enrollment owner and must not move
cleanup claims or mutation dispatch into NodeHost, CloudWorker, or Gateway
handler code.

The corresponding oracle cluster proves the required negative edges at source
level: aged capability reapproval survives reads, writes and database reopen;
changed surfaces supersede older requests; reconnect cleanup claims are
generation-ordered and cannot delete a replacement request; approval remains
blocked while any cleanup claim is active; retired pairing generations cannot
write connection metadata, bins or session-host consent; and stale approval
policy/token issuance is refused while a pairing lock is held. These are frozen
oracle expectations only; they do not replace exact-head Actions execution or
real companion-device acceptance.

The token/migration cluster adds two security and compatibility constraints:
legacy pairing migration imports only validated durable device records, keeps
existing SQLite rows authoritative, excludes transient pending/bootstrap
records, archives legacy files only after the import path, and preserves
retryability on unreadable input. Token verification requires the paired-device
scope baseline, role approval, issuer-generation freshness and explicit
revocation checks; issuance, rotation and revocation clear node-generation
state and record last use through the existing mutation owner. These behaviors
must remain under the existing pairing/enrollment owner, not under CloudWorker
or Gateway transport code.

The low-level pairing-token oracle additionally requires 32 random bytes,
base64url encoding, constant-time equality semantics and rejection of blank
tokens even when both inputs are blank. This is a cryptographic source oracle;
it is not evidence that the shipping Rust path has passed its executable
security gate.

---

## 3. Gateway producer/consumer closure (2026-10-29)

The locked Gateway pairing producer and node-session consumer were read in full
and mapped to the existing owners. `nodes.pairing.ts` owns scoped list/approve/
reject/remove/rename authorization, publishes the committed pairing surface
with expected connection identity and generation fencing, invalidates wakes and
runtime state on removal, and disconnects only the removed device's node role.
`connect-node-session.ts` begins a generation-scoped reconnect claim, derives
the first capability surface from explicit approval provenance, routes reapproval
through the existing pairing owner, and applies declared/effective capability
ceilings before NodeHost admission. `node-registry-pairing.ts` is a narrow
publication predicate: identity and optional generation must still match before
the live session is considered current. Fabushi mapping therefore remains
DevicePairingOwner for pairing mutation/publication and NodeHostSessionRegistry
for live transport/session admission; no Gateway or CloudWorker state owner is
introduced. This closes the source producer → existing owner → consumer map only;
production wiring, executable verification, and physical companion-device
acceptance remain pending.

## 4. Blockers & Frontiers

- `OCRM-BLOCK-GW-SCOPED-AUTHORITY-001`: Remaining pairing worker communication and background daemons not fully closed; external physical companion device communication unverified.
- `OCRM-BLOCK-GW-INDEPENDENT-REVIEW-002`: Independent review receipt pending.
- Current Rust status in `parity-ledger.json` records the scoped state-machine
  and node-lifecycle contracts as implemented/mapped, while production wiring,
  executable verification and acceptance remain pending.

---

## 4. Itemized Audited Source Files (38 Files)

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
| `src/infra/device-pairing-node.ts` | `3e032d10f1b2079a22784e8181d14f2eda8e4946` | 331 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-core.worker.ts` | `fb60cb2a473a26e1388ea59cc4be840fd33f3110` | 102 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-binding.ts` | `5d7b5fc031662b0a9a24466f2f3ea932e7e509b6` | 20 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-node.lifecycle.test.ts` | `d632873d782487012c1d36286a61fd88f87a1f44` | 170 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/network_enrollment_owner_contract.rs` |
| `src/infra/device-pairing-node.test.ts` | `aebbbab2f441e4c952e2094c1a7d8373819c8995` | 1044 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/network_enrollment_owner_contract.rs` |
| `src/infra/device-pairing-approval-policy.test.ts` | `b8edee7f829134a211fbb2c91018fad7c72edca8` | 101 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/network_enrollment_owner_contract.rs` |
| `src/infra/device-pairing-migration.ts` | `7b0b605e63f3ba7ab97ea00bb200cb5672ee4df1` | 145 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-token-policy.ts` | `11044c4dfd3e1a1e4146768b85e217f877c44109` | 142 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-tokens.kernel.ts` | `5c61ce1a12a816dd5c147a91f66a0f1f837051d4` | 248 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/device-pairing-tokens.ts` | `5e2f2662f1401e0f1f40f419a95557b458875027` | 174 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/network_enrollment_owner.rs` |
| `src/infra/pairing-token.test.ts` | `0d7f4087fa156fa0982d996ded64408d7619fd07` | 47 | TypeScript | read-complete | test-conformance-oracle | `source/host/tests/network_enrollment_owner_contract.rs` |
| `src/gateway/node-registry-pairing.ts` | `6ee2e1669f2315ad414383b9153d8ff73f0bffbf` | 59 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/node_host/session_registry.rs` |
| `src/gateway/server-methods/nodes.pairing.ts` | `719d56754faf40644c20c24b44a47a7c79ba369e` | 380 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/device_pairing_owner.rs` |
| `src/gateway/server/ws-connection/connect-node-session.ts` | `51aad1c06c2b8caa84753dfd53c5755767ff10bf` | 221 | TypeScript | read-complete | reuse-and-extend-existing-Rust | `source/host/src/extensions/node_host/session_registry.rs` |

## 5. Gateway websocket auth and pairing admission (2026-10-29)

The locked websocket admission cluster was read in full and mapped to existing
owners. `connect-auth-security.ts` emits redacted, hashed device identifiers
and bounded auth attributes through the canonical security-event owner.
`connect-device-tokens.ts` issues role/scope-bound device grants only when the
approved baseline and issuance-current fence hold, and emits setup-code
handoff grants only through the closed bootstrap profile. `connect-existing-device.ts`
pins platform/device-family metadata, requires pairing for role or scope
upgrades, and keeps ingress grants distinct from the signed device identity.
`connect-node-pairing-ssh.ts` limits SSH auto-approval to fresh scopeless node
requests and rechecks the exact policy immediately before commit. The 1,022-line
pairing oracle covers authority changes during approval, merged-owner repair,
token reuse, role/scope fences, reconnect pause, and Tailscale scope caps.
These responsibilities remain under Gateway auth, DevicePairing approval/token
owners, and NodeHost connection admission; no second pairing or auth owner is
introduced. Rust implementation, exact-head execution, and real-device
acceptance remain pending.

## 6. Setup-code endpoint provenance (2026-10-30)

`DevicePairingOwner::handle_pair_setup_code` no longer emits a fabricated
`http://127.0.0.1:8080` value. A caller may provide `gatewayUrl`, but the
existing `validate_mobile_pairing_url` policy rejects public cleartext `ws://`
and accepts only the declared secure/local forms; an omitted URL is not
replaced with a guessed endpoint. This is a fail-closed producer/owner
boundary: the current Host Gateway binds a runtime port after
`start_gateway_server`, while the inspected Rust ingress is HTTP/TLS and does
not yet provide the upstream websocket pairing/session transport. Therefore no
automatic cross-device URL or live companion reachability is claimed until an
existing transport owner supplies a real endpoint and consumer path.

## 7. Device-token consumer wiring (2026-10-30)

`DevicePairingOwner::authenticate_device_token` now owns lookup of issued
device tokens, constant-time token comparison, and projection of the durable
device identity, role, and scoped grants. `UnifiedGatewayApi` exposes that
owner through the existing `GatewayApi` boundary; Gateway ingress accepts a
device bearer only when the pairing owner returns an authorization result and
enforces the returned role/scope ceiling before dispatch. The configured
Gateway bearer secret remains a separate compatibility credential. Timestamp
and nonce headers remain replay fences and are not credentials. This closes the
local producer → pairing owner → ingress consumer path; websocket challenge,
remote companion transport, executable Actions evidence, and real-device
acceptance remain pending.

## 8. Rotation and revocation invalidation contract (2026-10-30)

The pairing contract now records two negative paths at the owner consumer
boundary: a token issued by `device.pair.rotateToken` authenticates before the
next rotation but the prior bearer is rejected after rotation; a bearer that
authenticates before `device.pair.revoke` is rejected after the paired row is
deleted and `device.pair.status` becomes `rejected`. These checks bind token
lookup to current durable pairing state rather than a cached grant.

The assertions are source-level Rust contract evidence only until the declared
Actions run executes them. They do not prove websocket reconnect fencing,
companion delivery, current OpenClaw source closure, or real-device acceptance.

## 9. Pending-request identity and supersession (2026-10-30)

`DevicePairingOwner::handle_pair_request` now preserves one pending identity
for a compatible retry: the same device/public key/browser origin/role with an
identical or narrower role/scope surface refreshes `refreshed_at_ms` and
returns the original `requestId`. An incompatible public key or browser
origin, or an escalated role/scope surface, supersedes the device's prior
pending rows before creating a new request. The original request then reports
`rejected` rather than remaining a usable approval handle.

This closes the local pending producer → SQLite owner → status consumer
invariant, including the five-minute TTL based on the original or refreshed
timestamp; expired rows are purged before status, approval, rejection, list,
or a new request can reuse them. Approval-worker concurrency,
reconnect-generation fencing, and companion acceptance remain separate open
responsibilities. Approval now also carries the accepted `browserOrigin` from
the pending row into the paired durable identity instead of dropping that
binding at commit, and reapproval clears any retained or pending node surface
so a successor connection must publish a fresh generation-bound capability
surface. Token rotation applies the same node-surface reset atomically with
the bearer replacement, so credential generation and capability generation
cannot silently diverge.
