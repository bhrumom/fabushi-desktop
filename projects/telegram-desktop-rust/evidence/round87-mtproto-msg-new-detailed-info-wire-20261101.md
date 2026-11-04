# Round 87 — MTProto msg_new_detailed_info wire closure

Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.

Fixed source identities:
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`

## Source/shipping readback

The schema declares
`msg_new_detailed_info#809db6df answer_msg_id:long bytes:int status:int = MsgDetailedInfo`.

Shipping `SessionPrivate::handle` first checks `info.badTime`; a bad-time `msg_new_detailed_info` is ignored
before typed decoding. Otherwise `MTPMsgDetailedInfo::read(from,end)` performs the typed wire decode. Only after
that read succeeds does the branch consult `_receivedMessageIds` and enqueue acknowledgement or resend work for
`answer_msg_id`. The bad-time gate and those history/application effects are outside this wire-only responsibility.

## Existing owner disposition

No production delta is required.

Canonical production blobs remain:
- `native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`
- `native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
- `native/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`

`parse_msg_new_detailed_info` requires constructor `0x809db6df`, then reads one signed little-endian `i64`
and two signed little-endian `i32` values, and rejects trailing bytes. `TlReader` fails closed on truncated reads.
The exact encoded object is 20 bytes: 4 + 8 + 4 + 4.

## Stage-A writes

- platform spec commit: `ba585c9d3c0d66fd07f254dac0e0ac97f0cd8c92`
- spec blob: `1295bed353dfd9b5f26518dadfc68ddb6e1bf56b`
- canonical authored-only contract commit: `9154bcf48a0d27efe2fa459a9670100c99d29e25`
- canonical contract blob: `feedf5e25c739e0deca7a26f356a25a9d1a9f46b`
- desktop mirror commit: `efeb7cef34a4582deb0a3631856ded581889f6bd`
- desktop contract blob: `feedf5e25c739e0deca7a26f356a25a9d1a9f46b`
- ledger commit: `0d5c346d108434ed02faaa542a17ff18bac48f4d`
- ledger blob: `88d76e27ad2ab0c7cfa60eb3f6fed1c8ed8d5989`

Authored contracts cover exact signed 8/4/4 widths, field order, wrong constructor, truncation at constructor and each
field boundary, trailing input and exact EOF. They were written but not executed.

Responsibility: `TDRP-R9-MTPROTO-MSG-NEW-DETAILED-INFO-WIRE-001` =
`implemented / validation_status=not-run / accepted=false`.

Coverage: `97 / 16172 unknown / 16145 unread / 0 omitted`.

Bad-time handling, `_receivedMessageIds` history, ack/resend application, arbitrary request history, connection
scheduling/racing/send retry, auth-key lifecycle, gzip, complete difference/differenceSlice decoding/application,
broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

## Final source-level compliance readback

Platform-core compliance commit:
`329c3e89bb5dcf5dbd6d27153fd160d891adbe2f`.

Final spec blob:
`docs/specs/telegram-mtproto-msg-new-detailed-info-wire.md@6681bade8a147afff7e18e9cdfe49fcd179f7436`.

Remote source readback records R1-R7 and AC-1-AC-6 as source/bookkeeping satisfied while executable Stage-B
validation remains intentionally not run. Canonical production api/tl/lib blobs remain unchanged at
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf`; canonical/desktop authored contract blob remains
`feedf5e25c739e0deca7a26f356a25a9d1a9f46b`. The responsibility remains
`implemented / validation_status=not-run / accepted=false`, coverage
`97 / 16172 unknown / 16145 unread / 0 omitted`.

The exact-width/trailing boundary remains intentionally separate from bad-time handling, received-message history,
ack/resend application, arbitrary request history, connection scheduling/racing/send retry,
auth-key binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice decoding/application,
broader source/legal closure, Stage-B executable evidence and independent release ACCEPT.
