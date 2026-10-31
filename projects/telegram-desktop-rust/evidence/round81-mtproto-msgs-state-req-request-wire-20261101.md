# Round 81 — MTProto msgs_state_req request wire responsibility

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536
Stage: A — implementation/bookkeeping only; executable validation intentionally not run

## Fixed source

- upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
- mtproto.tl: 76225965bfc794388db241f3f760db8ab79e84f8
- session_private.cpp: 13cbc76ec723386027b5b4dcac79999b791c4614
- constructor: msgs_state_req#da69fb52 msg_ids:Vector<long>

## Shipping source disposition

Fixed-U `SessionPrivate::tryToSend` serializes a state request only when `_stateRequestData` is non-empty,
preserves id order in `MTP_vector<MTPlong>`, and emits `MTP_msgs_state_req`. The later
`mtpc_msgs_state_info` path parses that previously sent request before applying message states.

Existing Fabushi canonical production owner requires no behavior delta:
- platform-core native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43
- platform-core native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264
- platform-core native/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf
- shipping Fabushi path remains existing single-current-RPC state reconciliation -> build_msgs_state_req([current_request_message_id])

The builder already writes constructor 0xda69fb52, TL Vector constructor 0x1cb5c415, signed i32 count and each
signed long as little-endian i64 in caller order. `TlWriter::write_vector_length` rejects a length that cannot
fit TL int before emitting a vector header.

## Spec and authored-only contracts

- spec commit: 579b575974b2031e50ffea4910acce8a67289a64
- spec blob: dfa2b21a82d9b4d9f92da32b4cedcc176bab0c3f
- canonical authored-contract commit: 97e04d8ee1556c28cd6227f809816f04bcf1f674
- canonical contract blob: 60a1dfb3ea2051a498d6cfb9d401fe653fde7d75
- desktop mirror commit: 23f023f57cc9c6bc50e519fe89f01cc78a3511d9
- desktop contract blob: 60a1dfb3ea2051a498d6cfb9d401fe653fde7d75
- desktop production api blob remains: ebfb3670b5cd097b9ce288782c7954f5cfe9fc43
- desktop facade blob remains: abab39e42f100972484191f0e52e9edd9b88c6cf

Authored cases cover empty Vector, one shipping id, multiple signed longs with exact order/bit preservation,
strict EOF, and i32::MAX+1 vector-count failure with no partial vector header. These contracts were written and
manually reviewed only; they were not executed.

## Ownership boundary

No second Telegram runtime/provider/service-message owner was added. Arbitrary multi-request
`haveSent/_resendingIds/_stateAndResendRequests` history, simultaneous state-query scheduling, connection
scheduling/racing, auth-key binding/persistence, gzip and complete difference decoding/application remain outside
this request-wire responsibility.

## Ledger state

- responsibility: TDRP-R9-MTPROTO-MSGS-STATE-REQ-REQUEST-WIRE-001
- implementation_status: implemented
- validation_status: not-run
- accepted: false
- coverage after this row: responsibilities_total=91, unknown=16172, unread=16145, omitted=0

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
