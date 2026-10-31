# Round 89 — MTProto bad_server_salt exact wire revalidation

Task round: 58  
Date: 2026-11-01  
Stage: A — zero CI

## Authority

- Starting desktop checkpoint: `e0c43de843ae9f843de83001117bc07702d1ff62`.
- Starting platform-core checkpoint: `21ab4e3c4b98a93761c8df525656c09f9b4c404d`.
- Desktop main authority: `9993ca8e107f7e019a8201e5ab74fa54e82159f2`.
- Platform-core main authority: `26c3cc8c69ab89e8fb38952adb8c438ec0b68e05`.
- Fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`.
- `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`.

## Source finding

Fixed-U declares `bad_server_salt#edab447b bad_msg_id:long bad_msg_seqno:int error_code:int new_server_salt:long = BadMsgNotification`. In `SessionPrivate::handleOneReceived`, the `mtpc_bad_server_salt` branch first invokes generated `MTPBadMsgNotification::read(from,end)`; only after successful parsing does it access all four fields and enter `wasSent`, salt adoption, server-time update, temporary-key binding handling, reconnect and resend behavior.

## Existing-owner exactness

- `native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`: `BAD_SERVER_SALT_CONSTRUCTOR`, typed `BadServerSalt`, and `parse_bad_server_salt` decode i64/i32/i32/i64 in order and require EOF.
- `native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`: fixed-size `take_array` backs signed little-endian `read_i64` / `read_i32`, so widths are exactly 8/4/4/8 and truncation fails closed.
- `native/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`: exports the same parser/type/constructor.
- `native/telegram-network/src/lib.rs@1fc1b1f69e3e37377400d078f916599e0c7b20bc`: both receive branches call the typed parser before current-request correlation and salt mutation, proving shipping reachability.

Exact object length is 28 bytes: 4-byte constructor + 8 + 4 + 4 + 8. This is source-level closed with **no production delta**.

## Durable Stage-A writes

- No-production-delta spec: `bf4f241f9ceddc9b032b23d1f154a8883b636e92@757e919f5328bd004fcf2e9a22b49a05e4a80f21`.
- Canonical authored-only external contract: `67e66a56a2ab2bf3c85b27a5d2bfac0d5682321f@b3e535525ffa97920816c9b8f316afddbf46f6ac`.
- Desktop byte-identical contract mirror: `8caa269f72d98f79624ef73bd3ea901d0d0a6f86@b3e535525ffa97920816c9b8f316afddbf46f6ac`.
- Production api/tl/lib/network blobs remain unchanged: `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `89256ab35184b6dc06a095ab34abb9f44e95a264` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`.

The external contracts cover exact signed field widths, exact EOF, wrong constructor, constructor/each-field truncation, and trailing-byte rejection. They were not run.

## Ledger

The existing row `TDRP-R9-MTPROTO-BAD-SERVER-SALT-WIRE-001` is retained at `implemented / validation_status=not-run / accepted=false`. No duplicate row is added. Coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`.

Salt adoption/persistence, bad-time/time correction, request-history lookup, binding/reconnect, resend/application, arbitrary sent/container history, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open or outside this wire-only claim.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 89 final source-level compliance

Platform-core final compliance commit `3db5c55a946a466d157e37b95d2fce0eb5706595` / spec blob `4247be931d826a80ef4cc10c8c4b84de96804991` records `bad_server_salt#edab447b` as source-level closed with **no production delta**. Canonical production api/tl/lib/network blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `89256ab35184b6dc06a095ab34abb9f44e95a264` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. Canonical and desktop authored-only external contract blob is `b3e535525ffa97920816c9b8f316afddbf46f6ac` and remains not run.

The existing ledger row stays `implemented / validation_status=not-run / accepted=false`; coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`. Salt adoption/persistence, bad-time/time correction, request-history lookup, binding/reconnect, resend/application, arbitrary sent/container history, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
