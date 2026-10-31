# Round 82 — MTProto msg_resend_req request wire responsibility

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536
Stage: A — implementation/bookkeeping only; executable validation intentionally not run

## Fixed source

- upstream: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
- mtproto.tl: 76225965bfc794388db241f3f760db8ab79e84f8
- session_private.cpp: 13cbc76ec723386027b5b4dcac79999b791c4614
- constructor: msg_resend_req#7d861a08 msg_ids:Vector<long>

## Shipping source disposition

Fixed-U `SessionPrivate::tryToSend` serializes a resend request only when `_resendRequestData` is non-empty,
moving the vector into `MTP_vector<MTPlong>` and emitting `MTP_msg_resend_req`. The fixed-U detailed-info
receive paths enqueue an answer message id into `_resendRequestData` when that answer was not received.

Existing Fabushi canonical production owner requires no behavior delta:
- platform-core native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43
- platform-core native/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264
- platform-core native/telegram-protocol/src/lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf

`build_msg_resend_req` already writes constructor 0x7d861a08, TL Vector constructor 0x1cb5c415, signed i32
count and each signed long as little-endian i64 in caller order. `TlWriter::write_vector_length` rejects a
length that cannot fit TL int before emitting a vector header.

This isolated wire responsibility does not claim Fabushi parity for Telegram Desktop's arbitrary
`haveSent/_resendingIds/toSend/_sentContainers` histories, callback ownership, resend scheduling, container
retry, connection scheduling, or related multi-request lifecycle.

## Spec and authored-only contracts

- spec commit: 953cd194143ae4cea4b8e842cb6d596b33451b19
- spec blob: 56facfa5ba3db5999a42cdcb742086f2c8ae690a
- canonical authored-contract commit: 161d78f68715540e733c43895aba2f1be4619755
- canonical contract blob: 89692a28620b2d8465703a1335458d5fbd15aee6
- desktop mirror commit: 37299e18e3018e988d541607d5ba7842713d9fd1
- desktop contract blob: 89692a28620b2d8465703a1335458d5fbd15aee6
- desktop production api blob remains: ebfb3670b5cd097b9ce288782c7954f5cfe9fc43
- desktop facade blob remains: abab39e42f100972484191f0e52e9edd9b88c6cf

Authored cases cover empty Vector, one shipping-shaped id, multiple signed longs with exact order/bit
preservation, strict EOF, and i32::MAX+1 vector-count failure with no partial vector header. These contracts were
written and manually reviewed only; they were not executed.

## Ownership boundary

No second Telegram runtime/provider/service-message owner was added. Arbitrary multi-request resend history,
connection scheduling/racing, auth-key binding/persistence, gzip and complete difference decoding/application remain
outside this request-wire responsibility.

## Ledger state

- responsibility: TDRP-R9-MTPROTO-MSG-RESEND-REQ-REQUEST-WIRE-001
- implementation_status: implemented
- validation_status: not-run
- accepted: false
- coverage after this row: responsibilities_total=92, unknown=16172, unread=16145, omitted=0

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.
