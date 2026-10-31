# Round 59 — MTProto bad-server-salt exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 28  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before implementation sync: `3b8ed9ee4ca189177f3ea340cd0b4945c23136b5`.
- Platform-core checkpoint before spec work: `f604e40e175d6a05ef10d6c4811ec1e363c85529`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U `session_private.cpp` blob: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Fixed-U `mtproto.tl` blob: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Ledger before this responsibility: `577e49bff2121804e62c24cfe871d1db03a3070d`; coverage `responsibilities_total=68 unknown=16172 unread=16145 omitted=0`.

## Gap confirmed

Fixed-U declares:
- `bad_server_salt#edab447b bad_msg_id:long bad_msg_seqno:int error_code:int new_server_salt:long = BadMsgNotification`.

The canonical network owner had two `BAD_SERVER_SALT` receive branches. Each read `bad_msg_id` at raw offset 4, `error_code` at offset 16, and `new_server_salt` at offset 20. Neither branch typed `bad_msg_seqno` nor rejected trailing bytes before mutating `server_salt`. Telegram Desktop's fixed-U branch decodes a generated TL object first and then reads all four fields.

This validation responsibility is independently representable by the existing one-outstanding-request protocol/network owner and does not require historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state.

## Durable spec and canonical implementation

- Spec commit: `12bc8a0ce1e453af1b3f2af0ffbd9ce868c92447`.
- Implementation commit: `3a850171c59a5f27809e176ba54971c7c0c003dc`.
- Final source-compliance commit: `515857196fd5fd8688326c7971a60079b67a0b71`.
- Canonical protocol api blob: `75959eb82460cb36bf5f43c6e4470ef8814ea2a9`.
- Canonical protocol lib blob: `3540f1a60df0d07f77b6e38349e0c500ec2d12d8`.
- Canonical network blob: `42be9e9fa78c3acc77367799c1ecb506cdd25ab0`.

Implementation facts from source inspection:
1. `telegram-protocol` exports `BAD_SERVER_SALT_CONSTRUCTOR`, `BadServerSalt`, and `parse_bad_server_salt`.
2. The parser consumes all fixed-U fields in order: bad message id, bad message sequence number, error code, and new server salt.
3. Wrong constructor, any truncated read, and trailing bytes fail before network side effects.
4. Both existing network receive branches call the typed parser before request correlation and before `self.server_salt` assignment.
5. `bad_server_salt_matches_request` restricts mutation to the one outstanding request; a valid unrelated id preserves the existing `BadServerMessage` failure.
6. The decoded `bad_msg_seqno` is retained in the typed value but is not used to invent historical sent/container reconciliation.
7. Matching objects preserve the existing exact server-salt assignment and bounded `NewSalt` retry path.
8. No gzip, full difference decoder, or historical request/container owner is introduced.

## Desktop synchronization

Desktop implementation commit: `131980d0bc3c5b6e8fad3ad2e394715426cd1529`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@75959eb82460cb36bf5f43c6e4470ef8814ea2a9`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@3540f1a60df0d07f77b6e38349e0c500ec2d12d8`
- `source/mahayana/mahayana-rs/providers/telegram-network/src/lib.rs@42be9e9fa78c3acc77367799c1ecb506cdd25ab0`

## Authored-only contracts

The source contains contracts for exact four-field decoding; wrong-constructor, truncation, and trailing-byte rejection; current-request-only correlation; and both existing receive paths consuming the same typed parser. They were **not run**.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

## Remaining fail-closed scope

- `gzip_packed` bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` historical bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
