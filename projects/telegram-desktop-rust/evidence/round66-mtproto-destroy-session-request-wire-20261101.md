# Round 66 — MTProto destroy_session exact request serialization

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 35  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before provider sync: `0eaf73f76d0412b72c53cb99765021d1420f08d8`.
- Platform-core checkpoint before request-side spec amendment: `6cfc81d79901493c6b34ec4e174d4fbdae81800e`.
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Ledger before this responsibility: `b1ad119730993014e1fc5ad3cc0d353d87a593b9`; coverage `responsibilities_total=75 unknown=16172 unread=16145 omitted=0`.

## Gap and scope decision

Fixed U declares `destroy_session#e7512126 session_id:long = DestroySessionRes`. Canonical Fabushi already had strict `DestroySessionRes` response decoding but no request constructor/builder. Exact request serialization is independently representable in the existing telegram-protocol owner. Live teardown/outbound-session lifecycle is not: current ownership does not precisely encode when a session should be destroyed, so it remains explicit fail-closed.

## Durable canonical implementation

- Spec amendment commit: `6f1ecac8f9284405e49dfc69af20270f914d2c83`.
- Implementation commit: `5f259ca296432384f5388ffef9539dbbf1f4033a`.
- Canonical api blob: `2f0c2893e256064f822082dd2fd7deae394b5763`.
- Canonical lib blob: `05c690a72f5f9b5a547f10486a5289229d7bb0ff`.

Source inspection confirms `build_destroy_session` writes only constructor `0xe7512126` then one `i64 session_id`, with no validation or lifecycle transformation. Its authored contracts assert exact 12-byte layout, signed boundaries, and that request bytes are not accepted by the separate `parse_destroy_session_result` response parser.

## Desktop synchronization

Desktop implementation commit: `28d36535b4d4d342b72c6a0e35752ce56dbcfd26`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@2f0c2893e256064f822082dd2fd7deae394b5763`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@05c690a72f5f9b5a547f10486a5289229d7bb0ff`

## Stage-A validation boundary

Contracts were authored but not run. No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

Implementation status: implemented. Validation status: not-run. Accepted: false.

## Remaining fail-closed scope

- Live outbound destroy_session/session teardown lifecycle.
- TLS camouflage ClientHello generator/socket lifecycle.
- gzip_packed bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary wasSent/haveSent/_resendingIds/toSend/_sentContainers bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.


## Source-level compliance readback

Platform-core compliance commit: `5e4a142d6090bcfd233b9913c1dbb16caa999a7b`.  
Final spec blob: `c2450bb9c97b229392adf2dbc7d6ddab7d3f624b`.

R8-R14 and AC7-AC10 are satisfied by source/blob/bookkeeping inspection for the request-side serializer boundary. Canonical and desktop provider blobs remain expected at api `2f0c2893e256064f822082dd2fd7deae394b5763` and lib `05c690a72f5f9b5a547f10486a5289229d7bb0ff`. Executable validation remains Stage-B blocked/not-run; `accepted=false`. Live teardown/outbound-session lifecycle remains explicit fail-closed.
