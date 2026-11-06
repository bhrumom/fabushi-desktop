# Current OpenClaw Gateway client payload-limit audit

Status: exact-subject source audit; not implementation, verification, or
production acceptance evidence.

## Exact source

- subject: `openclaw/openclaw@ad57b6135a55a6792b6d703bc1750954d854d6aa`
- tree: `24204d2196587105bf3af93a999cea03e8bdfae5`
- changed-from: `b7cc7ebc9b442ed7eb73a92336c2a9886b578f7b`

| Path | Blob | Observed responsibility |
| --- | --- | --- |
| `packages/gateway-client/src/pending-request.ts` | `79bd17722dfd2dcc11a47ad2dce13ce179535737` | Per-request correlation, retry/settlement and negotiated outgoing payload byte guard |
| `packages/gateway-client/src/protocol-client.ts` | `6b9fbb67949cd54440f7ebb32dcf8745980439ed` | One browser-safe socket/handshake/reconnect owner; applies `hello.policy.maxPayload` |
| `packages/gateway-client/src/protocol-request.ts` | `eb42c16dfd9401480e661693edd20afc0d748d02` | Typed local `GatewayPayloadLimitError` distinct from response/timeout errors |
| `packages/gateway-protocol/src/frame-guards.ts` | `815e2ec3f1d83086df6acd3ae36d9855e2cade1f` | Exports the shared pre-auth payload limit with protocol guards |
| `packages/gateway-protocol/src/payload-limits.ts` | `701718e2c5ee07338909d886e68846aa23cd0b71` | `MAX_PREAUTH_PAYLOAD_BYTES = 64 * 1024` |
| `src/gateway/server-constants.ts` | `97d805fa84179787010eb654dc11fd3ca4b0e11b` | Re-exports the same protocol-owned pre-auth limit |

## Behavior observed

Before hello, the client uses a 64 KiB default. After hello, the client
replaces it with `hello.policy.maxPayload`. Every serialized outgoing frame is
measured in UTF-8 bytes before `send`; an oversized frame raises a typed local
error and is not sent. Flushing pending requests restores the pre-auth default.
The protocol package owns the limit; the Gateway server constant is a re-export,
not a second owner.

## Fabushi owner boundary

The current Fabushi search found `source/node-agent-coordinator/src/gateway/
gateway_client.rs` as an existing HTTP/SSE Gateway client and
`source/host/src/gateway_server.rs` as the Host server boundary. These are not
yet evidence of the OpenClaw browser WebSocket client's negotiated
`hello.policy.maxPayload` contract. Existing 64 KiB checks in Coordinator and
Host are separate header/body guards and must not be relabeled as this
per-request UTF-8 pre-send guard.

The likely responsibility remains within the existing Gateway/Coordinator/Host
transport owners, but the exact Rust consumer, hello policy source, pending
request state owner and a falsifiable cross-language oracle require recursive
source/dependency closure and independent review before production code changes.
No new transport owner or second request store is authorized by this audit.

## Open evidence

- Current candidate dependency closure is unresolved: the root manifest and
  lock structure do not correspond, as recorded in the dependency probe.
- The changed source's transitive imports (`frame-guards`, protocol schema,
  normalization, retry, timeout, handshake policy and transport adapters) have
  not been closed at this exact subject.
- No Rust implementation, producer-to-consumer wiring, Actions verification,
  package/runtime evidence, or acceptance status changes from this dossier.
