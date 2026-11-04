# Round 67 — MTProto get_future_salts exact request serialization

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 36  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before provider sync: `0c19c0e94186e36c1170322e42eb86c70e629976`.
- Platform-core checkpoint before request-side spec amendment: `5e4a142d6090bcfd233b9913c1dbb16caa999a7b`.
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Ledger before this responsibility: `ea28d169cd0859cf67a34fa54afcb88a4e6302e0`; coverage `responsibilities_total=76 unknown=16172 unread=16145 omitted=0`.

## Gap and scope decision

Fixed U declares `get_future_salts#b921bd04 num:int = FutureSalts`. Canonical Fabushi already had strict `FutureSalts` response decoding and current-request-only correlation but no request constructor/builder. Exact request serialization is independently representable in the existing telegram-protocol owner. Live scheduling and future-salt installation/rotation are not: current ownership does not precisely encode when to request salts or how returned salts mutate session state, so those lifecycles remain explicit fail-closed.

## Durable canonical implementation

- Spec amendment commit: `5a52c2a1c53efbe9ce396919598a934dc732a8d7`.
- Implementation commit: `58a9635f0286283218f6d8b7b2b14dd466bd4b98`.
- Canonical api blob: `e8026c79b8b14261ef30d39bf6de8b0bc92306a6`.
- Canonical lib blob: `809eeb7aa3256e1aa259e694f413b9c0377f08d6`.

Source inspection confirms `build_get_future_salts` writes only constructor `0xb921bd04` then one signed `i32 num`, with no validation, clamp, scheduling, or salt-lifecycle transformation. Its authored contracts assert exact 8-byte layout, signed boundaries, and that request bytes are not accepted by the separate `parse_future_salts` response parser.

## Desktop synchronization

Desktop implementation commit: `dd232c52e1b39cdccb409c6d6307f1523e4d538f`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@e8026c79b8b14261ef30d39bf6de8b0bc92306a6`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@809eeb7aa3256e1aa259e694f413b9c0377f08d6`

## Stage-A validation boundary

Contracts were authored but not run. No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

Implementation status: implemented. Validation status: not-run. Accepted: false.

## Remaining fail-closed scope

- Live get_future_salts request scheduling and returned future-salt installation/rotation.
- TLS camouflage ClientHello generator/socket lifecycle.
- gzip_packed bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary wasSent/haveSent/_resendingIds/toSend/_sentContainers bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.


## Source-level compliance readback

Platform-core compliance commit: `eaf1eeefdde1401d820d9fecd15da3b4aa9cfde5`.  
Final spec blob: `4b690dcafce939c13540d1ff0a55a66bed1138bb`.

R11-R17 and AC9-AC12 are satisfied by source/blob/bookkeeping inspection for the request-side serializer boundary. Canonical and desktop provider blobs remain api `e8026c79b8b14261ef30d39bf6de8b0bc92306a6` and lib `809eeb7aa3256e1aa259e694f413b9c0377f08d6`. AC13 executable validation remains Stage-B blocked/not-run; `accepted=false`. Live request scheduling and future-salt installation/rotation remain explicit fail-closed.
