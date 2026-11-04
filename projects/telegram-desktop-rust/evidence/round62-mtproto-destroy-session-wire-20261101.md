# Round 62 — MTProto DestroySessionRes exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 31  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before provider sync: `28799f961c18d95e1b5786dce23c312390a535b6`.
- Platform-core checkpoint before spec work: `89f4ec41ed7236dddc59b876a791efa67dddf33d`.
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Ledger before this responsibility: `a576fdcc0ba99f18a5ff7f8ecfea0a71ceddedf8`; coverage `responsibilities_total=71 unknown=16172 unread=16145 omitted=0`.

## Gap and scope decision

Fixed U declares `destroy_session_ok#e22045fc session_id:long`, `destroy_session_none#62d350c9 session_id:long`, and `destroy_session#e7512126 session_id:long`. Canonical Fabushi had no typed parser for either result. Fixed-U `session_private.cpp` has no direct `destroy_session_*` receive branch, and current canonical/desktop composition has no established outbound destroy-session lifecycle to safely own teardown semantics. Therefore exact result decoding is independently implementable in the existing protocol owner, while live session teardown remains fail-closed.

## Durable canonical implementation

- Spec commit: `dee7cff150070d7891478510386674970a6217af`.
- Implementation commit: `cfe6a0564c40d2c9b9ea62b686470139a1eeb2ab`.
- Canonical api blob: `d82cca134274b8d939c1140ec44ee00230e2cfac`.
- Canonical lib blob: `655498ce9db2faf8a76f093250d4a1f02f77ff67`.

Source inspection confirms `parse_destroy_session_result` dispatches both fixed-U constructors, preserves one `session_id:long`, rejects unknown constructors/truncation/trailing bytes, and performs no network/session mutation.

## Desktop synchronization

Desktop implementation commit: `b179796344fba3e22bccb1310b74f229d613b531`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@d82cca134274b8d939c1140ec44ee00230e2cfac`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@655498ce9db2faf8a76f093250d4a1f02f77ff67`

## Authored-only contracts

Contracts cover both result variants, session-id preservation, unknown constructor, truncation, and trailing-byte rejection. They were not run.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

## Remaining fail-closed scope

- Outbound `destroy_session` lifecycle and live session teardown semantics.
- Outbound `rpc_drop_answer` lifecycle and historical request/drop mutation semantics.
- `gzip_packed` bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
