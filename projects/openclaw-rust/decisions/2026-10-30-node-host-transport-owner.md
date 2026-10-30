# ADR: Node Host transport owner boundary

Status: accepted for implementation design; production wiring pending exact-source closure and a legitimate endpoint caller
Date: 2026-10-30
Scope: Fabushi Desktop Host / OCRM-M13 Node Host

## Context

The current Fabushi Rust NodeHost modules provide endpoint validation, Origin
scoping, candidate failover policy, frame admission, stream splice decisions,
session generation fencing, and invocation correlation. The new
`gateway_transport.rs` slice now creates the single blocking Gateway socket,
performs the challenge-first handshake, and owns bounded request/event reads;
it still has no shipping production caller and does not own the product session
registry or side-effect executors.

The existing WebSocket implementation in `native/mahayana-messaging` and
`source/host/src/gateway_server.rs` is an inbound Fabushi messaging server,
not an OpenClaw NodeHost client. Reusing it as if it were the NodeHost
transport would mix protocols and create false production reachability.

The current OpenClaw discovery candidate requires a NodeHost client contract
with a request surface, negotiated server capabilities, hello-gated manifest
publication, connection-generation retirement, and cleanup that prevents late
callbacks from mutating a replacement connection. The current candidate also
contains worker/environment responsibilities that must remain behind their
existing owners.

## Decision

Create one narrowly scoped NodeHost client transport owner at the Host
transport boundary. It is an extension of the existing Host transport
ownership area, not a second pairing store, RPC session registry, Computer
executor, worker environment owner, or generic Agent loop.

The owner is solely responsible for:

1. candidate URL selection and WebSocket/TLS lifecycle;
2. credential/challenge signing through injected existing credential APIs;
3. hello/protocol/capability admission before publishing live presence;
4. bounded request correlation and inbound event dispatch;
5. one connection generation and ordered disconnect/replacement cleanup;
6. forwarding admitted facts to `NodeHostRuntimeState`, manifest owner, and
   invocation runtime without retaining parallel product state.

The owner must expose a transport-neutral request/event interface to the
existing runtime slices. It must reject or remain disconnected when the
credential signer, challenge contract, protocol version, TLS policy, or
server hello is unavailable. Pairing approval remains identity/approval
input only; it cannot activate the owner or make an RPC projection connected.

The credential input is an existing-owner extension: Host now has a local
identity/signing slice and an origin-scoped token store with rotation-fenced
reads/writes, while `DevicePairingOwner` continues to own remote
pairing/public-key and inbound role-token state. The transport owner must
depend on this explicit credential-owner interface for installation-local
identity, raw public-key encoding, challenge signing, and token reads. It must
not create ephemeral keys or inspect pairing storage directly. The producer
that receives/approves a Gateway-issued NodeHost token is still not wired.
The current upstream Cloudflare Access contract is likewise SecretInput-based;
Fabushi must resolve and redact those values through an approved secrets owner
before they can become transport headers. Box secret synchronization and
unrelated process environment variables are not valid substitutes.

### Secret materializer boundary revalidation (2026-10-30)

The existing Rust redaction surface is pure formatting only: it masks or
rewrites values already supplied to a diagnostic boundary and does not maintain
a registration/retirement lifecycle. `HostSecretsExtension` owns Box secret
application and persistence, not arbitrary `SecretInput` resolution. The
required NodeHost owner is therefore a separate injected contract: paired,
origin-scoped opaque references in; ephemeral credentials out only at connect
time; registration with the canonical redaction lifecycle before use; no
plaintext persistence, session/RPC projection, arbitrary environment fallback,
or raw-header logging. This is a design/unblock condition, not a production
implementation or acceptance claim.

## Caller reachability revalidation (2026-10-30)

A read-only search of the current checkout found no legitimate production
caller for the OpenClaw NodeHost client. `DevicePairingOwner::handle_pair_setup_code`
validates an optional `gatewayUrl` and echoes it in a short-lived setup result;
it does not persist an endpoint or start a NodeHost transport. The Electron
`GatewayDescriptorStore`/box connector owns the Fabushi Sand box/backend
connection, while `gateway_server.rs` and `native/mahayana-messaging` own the
inbound Fabushi messaging server. None is an OpenClaw NodeHost client config
owner. Pairing rows therefore remain approval/identity input only, and no
startup caller may infer a NodeHost endpoint from them or from an unrelated
Sand descriptor.

## Consequences

- A WebSocket dependency may be added only to this owner and only with the
  exact source/protocol contract recorded for the accepted upstream subject.
- `gateway_server.rs` and `native/mahayana-messaging` remain unchanged as
  separate inbound messaging owners.
- `node.register`, `node.list`, `node.status`, and `node.invoke` remain
  projections/consumers; they cannot open transport or synthesize live state.
- Production startup remains fail-closed until the owner has a real caller,
  credential binding, and exact-head/real-device evidence.
- The owner must be removable behind the existing host/runtime gate, leaving
  approved remote devices paired-but-disconnected on rollback.

## Required implementation evidence

- exact upstream source and dependency closure for the connection/challenge/
  hello/request/event contract;
- secure credential-owner contract and persistence/rotation evidence for the
  local device identity and origin-scoped node token;
- focused Rust contract tests for generation fencing, hello gating, request
  correlation, malformed/oversized frame rejection, and cleanup;
- a production caller that feeds the canonical session registry and runtime;
- exact-head Actions evidence plus packaged and real Gateway/device transport
  acceptance before any completion flag changes.

## References

- `docs/specs/node-host-runtime-embedding.md`
- `projects/openclaw-rust/dossiers/node-host-scoped.md`
- `source/host/src/extensions/node_host/`
- `source/host/src/gateway_server.rs`
- `native/mahayana-messaging/src/gateway.rs`
- OpenClaw discovery candidate `5f6f15fed0d6ba618cf781c9686a8b93f499564b`
