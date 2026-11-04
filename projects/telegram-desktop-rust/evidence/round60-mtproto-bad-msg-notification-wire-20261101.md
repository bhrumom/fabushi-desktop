# Round 60 — MTProto bad-msg-notification exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 29  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before implementation sync: `c964a738554352c457daf347b3d68b1b39b0db32`.
- Platform-core checkpoint before spec work: `515857196fd5fd8688326c7971a60079b67a0b71`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U `session_private.cpp` blob: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Fixed-U `mtproto.tl` blob: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Ledger before this responsibility: `f60911da80e70dc4bcdc049e14ebdd901e632648`; coverage `responsibilities_total=69 unknown=16172 unread=16145 omitted=0`.

## Gap confirmed

Fixed U declares `bad_msg_notification#a7eff811 bad_msg_id:long bad_msg_seqno:int error_code:int = BadMsgNotification`. The canonical network owner had two receive branches reading only `bad_msg_id` from raw offset 4 and `error_code` from raw offset 16; neither branch typed `bad_msg_seqno` or rejected trailing bytes before recovery behavior. `direct_bad_message_time_recovery` also recognized only raw constructor/error-code offsets before relaxing the incoming message-id time window. Telegram Desktop's fixed-U branch decodes a generated TL object before reading all three fields.

This validation responsibility is independently representable by the existing one-outstanding-request protocol/network owner and does not require historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state.

## Durable canonical implementation

- Spec commit: `42d90d6351a31f6d0815c47f2d647ae8162eccf8`.
- Implementation commit: `0e569130a80a53b48b74f8e27a96004363b30e7e`.
- Canonical protocol api blob: `15e34bbb0e9c5a5d4fdd8b7a9cdedf3d6b123080`.
- Canonical protocol lib blob: `2c4897cfdba4502d8ca880d7a58cfd0e88877bf7`.
- Canonical network blob: `1fc1b1f69e3e37377400d078f916599e0c7b20bc`.

Source inspection confirms the protocol owner now exports the constructor, typed value and exact parser; all three fields are consumed and trailing bytes rejected. Both existing network receive paths parse before current-request correlation or recovery mutation. The direct clock-window relaxation now accepts only a fully valid typed notification with the already-supported direct-time recovery codes 16/17/64. The existing bounded recovery mapping remains 16/17/64 => time correction/retry, 32/33 => session reset/retry, other codes => fail-closed error. No historical request/container state was added.

## Desktop synchronization

Desktop implementation commit: `1205f8709995aa4f5fd5dafdc9fb5121fbfce06c`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@15e34bbb0e9c5a5d4fdd8b7a9cdedf3d6b123080`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@2c4897cfdba4502d8ca880d7a58cfd0e88877bf7`
- `source/mahayana/mahayana-rs/providers/telegram-network/src/lib.rs@1fc1b1f69e3e37377400d078f916599e0c7b20bc`

## Authored-only contracts

Protocol contracts cover exact three-field decode plus wrong-constructor, truncation and trailing-byte rejection. Network contracts cover current-request-only correlation, bounded error-code mapping, and malformed/trailing notifications not receiving the direct time-window exception. They were not run.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

## Remaining fail-closed scope

- `gzip_packed` bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` historical bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
