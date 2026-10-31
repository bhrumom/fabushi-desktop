# Round 94 — MTProto Pong wire adjacent revalidation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 63  
Stage: A — implementation only, zero CI

## Live authority

- Desktop starting checkpoint: `fba29e00d9fcf2a3f9156caa20e07d909cd08123`
- Platform-core starting checkpoint: `3e24ea732acd2bcb2fe5bf2e5257f275643384de`
- Fixed upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`
- `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`
- `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`

## Selected responsibility

The next shipping service-message branch after `new_session_created` is `pong#347773c5 msg_id:long ping_id:long = Pong`. The existing responsibility is `TDRP-R9-MTPROTO-SINGLE-PING-PONG-001`; no duplicate row is created.

## Source-level finding

Canonical `telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` already owns the exact typed parser. It consumes 20 bytes total: constructor `0..4`, `msg_id` `4..12`, and `ping_id` `12..20`; both fields remain signed `i64`; constructor mismatch, any truncation, and trailing bytes fail closed. Desktop shipping `telegram-network/src/lib.rs@1fc1b1f69e3e37377400d078f916599e0c7b20bc` dispatches `PONG_CONSTRUCTOR` through that parser before matching the current outstanding ping's request message id and ping id.

Therefore the production disposition is **no production delta**. Fixed-U historical `wasSent`, `requestsFixTimeSalt`, `requestsAcked`, arbitrary sent/resend/container history, connection scheduling, auth-key binding/persistence, gzip and complete difference decoding/application remain outside this exact wire responsibility.

## Authored contracts, not run

- Platform spec revalidation: `8680085af3ef5ba80fa63933a7ce22c36dfc031d`, blob `231a89e30087370eb0cf29dc422f47e0322266d6`.
- Canonical external contract: `a559782b390f43d5caa28d8eaba1f99a9292059b`, blob `7672b1908bdf8810592e86f8427fd85c49089d7f`.
- Desktop mirrors the same external contract blob byte-identically.
- Added exact length/offset assertions, `i64::MIN/i64::MAX` preservation, all `0..19` truncation cuts, wrong-constructor rejection, and trailing-byte rejection.

No authored contract was executed.

## Stage-A status

`implementation_status=implemented`; `validation_status=not-run`; `accepted=false`. Coverage remains `responsibilities_total=97, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, installation, or runtime acceptance was performed.
