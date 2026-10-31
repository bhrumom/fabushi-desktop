# Round 68 — MTProto ping exact request serialization

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 37  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before provider sync: `4092010441163c8cb21a9c1cf2435751d7a957e8`.
- Platform-core checkpoint before request-side spec amendment: `eaf1eeefdde1401d820d9fecd15da3b4aa9cfde5`.
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Ledger before this responsibility: `ab82efdd07bf4e8e4eecf0f9d31eaf1035b09bfb`; coverage `responsibilities_total=77 unknown=16172 unread=16145 omitted=0`.

## Gap and scope decision

Fixed U declares `ping#7abe77ec ping_id:long = Pong`. Canonical Fabushi already had strict `Pong` response decoding/current-ping correlation and a separate exact `ping_delay_disconnect` builder, but no ordinary-ping request constructor/builder. Exact request serialization is independently representable in the existing telegram-protocol owner. Fixed-U `SessionPrivate::tryToSend` chooses ordinary ping only from live connection/main-session/timer state; message-id assignment, send/ACK settlement and timeout/reconnect are session/network lifecycle and remain explicit fail-closed rather than being invented in the protocol builder.

## Durable implementation

- Platform-core spec amendment: `cd40329f3fbc39c06599e3b5d4dda63f09ae6b17`.
- Platform-core implementation: `d6ef74975d3541ddb4d187bfb5641773dc01a32d`.
- Canonical api blob: `714c24a5f1727e13693008ea42676831b3e8d3fd`.
- Canonical lib blob: `ef44b1bb7fcac0183f1b83cca8c9d994ecc8ce9c`.
- Desktop provider synchronization: `36b425e547509f334453ac5ebf09033e366fe6ae`.

Source inspection confirms `build_ping` writes only constructor `0x7abe77ec` then one signed `i64 ping_id`, for exactly 12 bytes and with no timer/session policy transformation. Authored contracts assert exact layout, signed boundaries, and that request bytes are not accepted by the separate `parse_pong` response parser.

## Stage-A boundary

Contracts were authored but not run. No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

Implementation status: implemented. Validation status: not-run. Accepted: false.

## Remaining fail-closed scope

- Live ordinary-ping scheduling/selection, message-id assignment, send/ACK settlement and timeout/reconnect lifecycle.
- Arbitrary wasSent/haveSent/_resendingIds/toSend/_sentContainers bookkeeping.
- TLS camouflage ClientHello generator/socket lifecycle.
- gzip_packed bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
