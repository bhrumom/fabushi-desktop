# Round 98 — MTProto msgs_ack request-wire adjacent revalidation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 67  
Stage: A / zero executable validation

## Source readback

- fixed U: telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4
- mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8:
  msgs_ack#62d6b459 msg_ids:Vector<long> = MsgsAck
- session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614:
  SessionPrivate::tryToSend serializes MTP_msgs_ack(MTP_vector<MTPlong>(base::take(_ackRequestData)))
  only when _ackRequestData is non-empty.

## Existing-owner review

Canonical telegram-protocol already owns the exact request serializer:
- api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43
- tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264
- lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf

Desktop production api/lib are byte-identical. No production behavior change is required.

## Spec and authored-only contracts

- platform spec commit: 5759adce64e495f85dbc068fb0d74d027dee8004
- platform spec blob: 6fd5961fa70b86417d7bb4e71b937c97edacbd08
- platform authored contract commit: 95508fda3420d4829eb1fb76a20bc6f589f02635
- canonical contract blob: 7314ad06c48f9ca87dc08c0bd3b7d08a47579797
- desktop mirror commit: 4677ae986ea7f9e3156c57883ecfbb6fca7e2f49
- desktop mirror blob: 7314ad06c48f9ca87dc08c0bd3b7d08a47579797

Contracts cover zero, one, multiple signed i64 values, exact constructor/vector/count/order/EOF, and
vector-length overflow with no partial vector header. They are authored only and were not executed.

## Disposition

Responsibility: TDRP-R9-MTPROTO-MSGS-ACK-REQUEST-WIRE-001

Status: implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false.

This slice closes only exact request-wire serialization. Arbitrary ACK scheduling/history/application and broader
request/container lifecycle remain open/fail-closed. Stage B executable validation and independent release ACCEPT
remain outstanding.


### Round 98 final source-level compliance

Platform-core final compliance is `db0fc2f7db9871bedf7076b12b1fa8ab9c9e430d` / spec blob
`4960f9aba348caf9dcd520a26a9bcc8952faa1a3`, referencing desktop bookkeeping checkpoint
`a4a75364cb754e6d62ea4c02fc76dc4dc99163eb`. The unique `TDRP-R9-MTPROTO-MSGS-ACK-REQUEST-WIRE-001` responsibility is
`implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Production disposition remains **no production delta**. Canonical/desktop telegram-protocol api/lib blobs remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf`,
TL remains `89256ab35184b6dc06a095ab34abb9f44e95a264`, and canonical/desktop authored contract blob is
`7314ad06c48f9ca87dc08c0bd3b7d08a47579797`.

Coverage is `responsibilities_total=99, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.
Arbitrary ACK scheduling/history/application, connection/TLS scheduling, auth-key binding/persistence, gzip,
complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence
and independent release ACCEPT remain open. No executable validation was performed.
