# Round 88 — MTProto bad_msg_notification adjacent wire no-production-delta

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 / user round 57
Stage: A — source implementation/bookkeeping only; executable validation deferred

## Live authority readback

- Desktop checkpoint before bookkeeping: `58237abd9bcb64671d1bbfe88a3cdc8bdca5a2a4`.
- Platform-core checkpoint after spec/contract writes: `bc5f9410a520e5e35d40a1c5a1e6a8f2031d6030`.
- Fixed upstream U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U schema blob: `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`.
- Fixed-U shipping blob: `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`.

## Fixed-U wire and shipping boundary

The schema declares:
`bad_msg_notification#a7eff811 bad_msg_id:long bad_msg_seqno:int error_code:int = BadMsgNotification`.

Shipping `SessionPrivate::handle` first executes `MTPBadMsgNotification::read(from, end)`; only after successful
typed decoding does it inspect the three fields and enter resend, bad-time/time correction, sent-request/container
history, or session-reset behavior. The wire parser and recovery/application semantics are therefore separable.

## Current Fabushi owner readback

The existing canonical owner already closes the exact parser:

- `native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`:
  constructor `0xa7eff811`, typed i64/i32/i32 value and strict trailing-byte rejection.
- `native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`:
  exact signed little-endian 8-byte/4-byte fixed-width reads and fail-closed truncation.
- `native/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`:
  exports the parser/type/constructor.
- `native/telegram-network/src/lib.rs@1fc1b1f69e3e37377400d078f916599e0c7b20bc`:
  pre-existing consumers parse first, then apply current-request/recovery policy.

Desktop production api/lib/network blobs are byte-identical to those canonical current blobs. No production source was
changed in round 88.

## Durable Stage-A writes

- Spec-first no-production-delta commit:
  `9984ca1b13185693961ff0735156e55ab035e62a`;
  spec blob `7155c589c1e4090a534060e42fc9ebce1f8c404d`.
- Canonical authored-only external contract commit:
  `bc5f9410a520e5e35d40a1c5a1e6a8f2031d6030`;
  contract blob `184e5bfa7832e9bc7c7d4f594011b032382e11d3`.
- Desktop byte-identical contract mirror:
  `58237abd9bcb64671d1bbfe88a3cdc8bdca5a2a4`;
  contract blob `184e5bfa7832e9bc7c7d4f594011b032382e11d3`.

The new external contracts cover exact 20-byte layout, signed i64/i32/i32 extremes, wrong constructor,
constructor/each-field truncation, trailing bytes and exact EOF. They were not executed.

## Existing ledger disposition

The same row `TDRP-R9-MTPROTO-BAD-MSG-NOTIFICATION-WIRE-001` is retained. No duplicate responsibility is created.
Status remains `implemented / validation_status=not-run / accepted=false`; coverage remains
`97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`.

## Explicit non-claims

This no-production-delta wire slice does not claim closure for bad-time/time-correction recovery, arbitrary request or
sent-container history, acknowledgement/resend application, connection scheduling/racing/send retry,
temporary/permanent auth-key binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice
decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence or independent release ACCEPT.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
