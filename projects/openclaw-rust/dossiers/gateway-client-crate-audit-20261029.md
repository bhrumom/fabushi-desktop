# OCRM technical dossier — locked Rust Gateway client crate audit

**Status**: source audit and owner mapping only; no product runtime imported
**Project**: OCRM-001
**Audit date**: 2026-10-29 (Asia/Shanghai)
**Upstream authority**: `openclaw/openclaw@ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`)

## Scope and exact source reads

The following locked files were read against their exact GitHub blob identities:

| Path | Blob | Lines | Reading result |
| --- | --- | ---: | --- |
| `crates/openclaw-gateway-client/src/lib.rs` | `bf865be5f9c26e7b4a329645553a79f7e690a761` | 338 | TLS trust/pin/native certificate-policy boundary, bounded recovery metadata and reconnect backoff |
| `crates/openclaw-gateway-client/src/session.rs` | `c0df3638c8b4b7cef97cee5d84eff1277bc9d0f5` | 1701 | WebSocket connect/challenge/hello, bounded request session, event retention, deadlines, cancellation and close behavior |
| `crates/openclaw-gateway-client/Cargo.toml` | `8cb256da3f6b3640eef33b9bafd5886872ce9548` | 24 | Dependency and runtime feature boundary; not a product owner |

The two crate integration tests and workspace lock/config files remain separate
P0 closure items until their complete contents and dependency consumers are
reviewed; no test-only evidence is promoted to runtime acceptance here.

## Responsibility and owner mapping

The crate is a reusable transport/session implementation, not an authority that
can be copied into Fabushi as a second Gateway client. Its semantic behavior
maps to the existing `OCRM-RESP-NODE-HOST-CONNECTION-001` connection boundary:

- TLS pins and native certificate policy are transport admission inputs. They
  must remain behind Fabushi's existing credential/SecretInput and transport
  owners; raw credentials or certificate evidence must not enter the live
  session projection.
- The challenge, connect envelope, bounded WebSocket session, request timeout,
  event retention, cancellation and close fence belong to the future single
  node-host Gateway transport/session owner described by
  `docs/specs/node-host-runtime-embedding.md`.
- `DevicePairingOwner` remains identity/approval storage, and
  `NodeHostSessionRegistry` remains the only live presence projection. This
  crate does not authorize changing paired devices to connected devices.
- The existing Host Gateway server and typed protocol owners remain the
  product-facing RPC boundary; they are consumers/projections, not a second
  client-session store.

## Falsifiable review points

1. A configured fingerprint requires `wss://`; mismatches fail before the
   upgrade and use constant-time fingerprint comparison.
2. Native certificate policy is evaluated on the peer certificate captured from
   the same TLS connection before application headers/credentials are sent;
   TLS handshake signature verification remains mandatory.
3. Connect challenge timeout, request/write deadlines, frame/message bounds,
   in-flight semaphore capacity, event retention bytes and event lag are
   explicit bounded states.
4. Protocol fallback is one-shot and only follows a structured connect
   `PROTOCOL_MISMATCH`; unrelated errors do not trigger a second connection.
5. Request and cancellation traffic share an ordered bounded session path, so
   timeout/close cannot silently replay or settle a replacement request.

These are source-level contract facts. They do not prove a Fabushi caller,
real Gateway/device transport, packaged acceptance, or OCRM verification.

## Open integration boundary

The current Fabushi branch has transport-neutral node-host seams and local
`system.run` wiring, but no shipping remote Gateway caller. Therefore the
connection responsibility remains `wiring=not-wired` and `verification=not-run`
under `BL-NODE-HOST-CONN-WIRING-001`. The next implementation must designate
one transport/session owner and adapt existing credential, policy, local-exec,
Computer, worker and stream owners without importing this crate wholesale.

## References

- `docs/specs/node-host-runtime-embedding.md`
- `projects/openclaw-rust/dossiers/node-host-scoped.md`
- `projects/openclaw-rust/parity-ledger.json`
- `source/host/src/extensions/node_host/gateway_connection.rs`
- `source/host/src/extensions/node_host/session_registry.rs`
- `source/host/app/src/main.rs`
