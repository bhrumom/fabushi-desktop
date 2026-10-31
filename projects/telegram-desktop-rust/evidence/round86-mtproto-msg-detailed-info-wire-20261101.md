# Round 86 — MTProto msg_detailed_info wire closure

Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Fixed source identities:
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`

## Source/shipping readback

The schema declares
`msg_detailed_info#276d3ec6 msg_id:long answer_msg_id:long bytes:int status:int = MsgDetailedInfo`.

Shipping `SessionPrivate::handle` first calls `MTPMsgDetailedInfo::read(from,end)`. Only after that typed read
succeeds does it perform bad-time recovery, `requestsAcked`, `_receivedMessageIds` lookup, and ack/resend
enqueueing for the answer message. Those application/history effects remain outside this wire-only responsibility.

## Existing owner disposition

No production delta is required.

Canonical production blobs remain:
- `native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`
- `native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
- `native/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`

`parse_msg_detailed_info` requires constructor `0x276d3ec6`, then reads two signed little-endian `i64` values
and two signed little-endian `i32` values, and rejects trailing bytes. `TlReader` fails closed on truncated reads.
The exact encoded object is 28 bytes.

## Stage-A writes

- platform spec commit: `38c54a94dbec4ac82118b8f2b6fd4c611719d064`
- initial spec blob: `a9285d6df90d78133b488d29b8eee79d97b5d8d8`
- canonical authored-only contract commit: `688ccb1acd5afb4e3d8c76c1c898148f721e3d48`
- canonical contract blob: `d1d4ed680d297b778516ec52d3b3746fe9687135`
- desktop mirror commit: `1d82ac67b4a86f1fd8b0584cbf8b76fb9252de2a`
- desktop contract blob: `d1d4ed680d297b778516ec52d3b3746fe9687135`

Authored contracts cover exact signed 8/8/4/4 widths, field order, wrong constructor, truncation at constructor and
each field boundary, trailing input and exact EOF. They were written but not executed.

Responsibility: `TDRP-R9-MTPROTO-MSG-DETAILED-INFO-WIRE-001` =
`implemented / validation_status=not-run / accepted=false`.

Broader request acknowledgement/history/resend semantics, connection scheduling, auth-key lifecycle, gzip,
complete difference decoding, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT
remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
