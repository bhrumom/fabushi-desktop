# Round 84 — MTProto msgs_state_info exact typed parser / binary TL string

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 53

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
The schema blob remains `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`; shipping source remains
`session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`.

Fixed-U defines `msgs_state_info#04deb57d req_msg_id:long info:string = MsgsStateInfo`. Shipping
`SessionPrivate::handle` first performs typed `MTPMsgsStateInfo::read(from,end)`; only after a successful
wire decode does it use `req_msg_id` to query `_stateAndResendRequests` and pass the returned binary state bytes
to `handleMsgsStates`. That request-history/application behavior is deliberately outside this isolated wire slice.

Remote source review confirms the existing canonical production owner already closes the independent parser boundary:
`native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines
`MSGS_STATE_INFO_CONSTRUCTOR = 0x04deb57d`, `MsgsStateInfo`, and `parse_msgs_state_info`; the parser requires
the constructor, reads signed i64, preserves `info:string` as opaque bytes through `TlReader::read_bytes()`, and
rejects trailing data as `InvalidMessageState`. `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
owns short/long TL byte-string framing, configured byte limits, padding consumption and truncation failures.
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser/type/constructor.

The binary distinction matters: fixed-U `info:string` carries compact message-state bytes, not human UTF-8 text.
Therefore the exact existing owner is `read_bytes()`, not `read_string()`; arbitrary non-UTF-8 bytes must survive.

Spec-first platform commit `41fc75c372bdb818c21c21af6fcc60958bb7974b`
(`docs/specs/telegram-mtproto-msgs-state-info-wire.md@d90260a94d2bc04f2d487614cc4806b819e126ee`)
records the no-production-delta disposition. Canonical authored-only contract commit
`ae5b7a6f6b6669fe9069d7371b1b4a2acd76e182` adds signed-id + binary non-UTF8 bytes, empty state,
254-byte long-form TL framing, wrong-constructor, truncation, missing-padding, trailing-byte and exact-EOF cases;
contract blob is `efaa8ac407af5bca430855fbb69c862f9ce793b0`. Desktop provider sync
`ae630cf55cb611f9e400b8c5a4bbbbad20654387` mirrors that blob byte-for-byte. No authored contract was run.

Disposition: no production delta. Responsibility
`TDRP-R9-MTPROTO-MSGS-STATE-INFO-WIRE-001` is
`implemented / validation_status=not-run / accepted=false`. Coverage becomes
`responsibilities_total=94 / unknown=16172 / unread=16145 / omitted=0`; the global source-entry unknown/unread
counts are intentionally unchanged because this is a responsibility slice, not a claim of whole-file source closure.

Arbitrary `_stateAndResendRequests/wasSent/haveSent/_resendingIds/toSend/_sentContainers` history,
`handleMsgsStates` application, connection scheduling/racing/send retry, temporary/permanent auth-key
binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice decoding/application, broader
source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final source-level compliance readback

Platform-core compliance commit:
`e99655ad3c06858e60fb7a1bf6a5a12cb1a0b2ff`.

Final spec blob:
`docs/specs/telegram-mtproto-msgs-state-info-wire.md@4780f4764ae68cbb921177214a05e215b98608fa`.

Remote source readback records R1-R10 and AC-1-AC-6 as source/bookkeeping satisfied while executable Stage-B
validation remains intentionally not run. Canonical production api/tl/lib blobs remain unchanged at
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf`; canonical/desktop authored contract blob remains
`efaa8ac407af5bca430855fbb69c862f9ce793b0`. The responsibility remains
`implemented / validation_status=not-run / accepted=false`, coverage
`94 / 16172 unknown / 16145 unread / 0 omitted`.

The fixed-U `info:string` payload remains intentionally binary at this parser boundary. Arbitrary request
history/application, connection scheduling/racing/send retry, auth-key binding/expiry/persistence/teardown, gzip,
complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence
and independent release ACCEPT remain open/fail-closed.
