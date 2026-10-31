# Round 85 — MTProto msgs_all_info exact typed parser / vector boundary / binary TL string

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 54

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
The schema blob remains `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`; shipping source remains
`session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`.

Fixed-U declares `msgs_all_info#8cc0d131 msg_ids:Vector<long> info:string = MsgsAllInfo`.
Shipping `SessionPrivate::handle` first calls `MTPMsgsAllInfo::read(from,end)`, then passes the typed
message-id vector and binary `info` bytes to `handleMsgsStates`. Before any `haveSent/_resendingIds`
lookup, `handleMsgsStates` requires `states.size() == ids.size()`. The later request-history lookup,
resend, acknowledgement and scheduling behavior is deliberately outside this isolated wire/state-shape slice.

Remote source review confirms the existing canonical production owner already closes this responsibility:
`native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines
`MSGS_ALL_INFO_CONSTRUCTOR = 0x8cc0d131`, `MsgsAllInfo`, and `parse_msgs_all_info`.
The parser requires the top-level constructor, uses `TlReader::read_vector_length` for canonical/bounded
TL vector framing, reads each `long` as signed `i64`, preserves `info:string` through
`TlReader::read_bytes()`, checks one state byte per id, and requires exact EOF.
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` supplies vector negative/over-limit rejection,
binary TL bytes short/long framing, configured size limits, padding consumption and truncation failure.
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser/type/constructor.

The cardinality check is intentionally fail-closed and matches the fixed-U pre-history `handleMsgsStates`
guard. It does not move arbitrary request history into the parser.

Spec-first platform commit `9f8ed1934fbd35e6a4e6ad9243c686431a84bdee`
(`docs/specs/telegram-mtproto-msgs-all-info-wire.md@8d4b04886141bf16bc36fe045d8997a472fae545`)
records the no-production-delta disposition. Canonical authored-only contract commit
`5e961490ebad8971a44ba91253797408c959ca93` adds signed-id + binary short-form state,
empty vector/state, 254-id/254-byte long-form state, wrong constructor/vector constructor,
negative/over-limit vector counts, truncation, missing-padding, state/id mismatch, trailing-byte and exact-EOF cases.
Contract blob is `d1a86cb35d6e27ed94c911750190c627fc7764cf`. Desktop provider sync
`822ed21752b28b435f55c2c711215fc01b15551c` mirrors that blob byte-for-byte. No authored contract was run.

Disposition: no production delta. Responsibility
`TDRP-R9-MTPROTO-MSGS-ALL-INFO-WIRE-001` is
`implemented / validation_status=not-run / accepted=false`. Coverage becomes
`responsibilities_total=95 / unknown=16172 / unread=16145 / omitted=0`; source-entry unknown/unread counts
remain unchanged because this is an isolated responsibility slice, not whole-file source closure.

Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, resend/ack application,
connection scheduling/racing/send retry, temporary/permanent auth-key binding/expiry/persistence/teardown,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure,
Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final source-level compliance readback

Platform-core compliance commit:
`a6281c8b43aff544acdafca454091fa2856cdd49`.

Final spec blob:
`docs/specs/telegram-mtproto-msgs-all-info-wire.md@0e0baa8fe35f9ef9e446d8a04ac7f95ca1fadba4`.

Remote source readback records R1-R10 and AC-1-AC-6 as source/bookkeeping satisfied while executable Stage-B
validation remains intentionally not run. Canonical production api/tl/lib blobs remain unchanged at
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf`; canonical/desktop authored contract blob remains
`d1a86cb35d6e27ed94c911750190c627fc7764cf`. The responsibility remains
`implemented / validation_status=not-run / accepted=false`, coverage
`95 / 16172 unknown / 16145 unread / 0 omitted`.

The fixed-U `msgs_all_info` vector/TL/cardinality boundary remains intentionally separate from arbitrary request
history/application, connection scheduling/racing/send retry, auth-key binding/expiry/persistence/teardown, gzip,
complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence
and independent release ACCEPT.
