# Round 63 — MTProto http_wait exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 32  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before provider sync: `c4329d6c1212d1f6309c0876580e5fe5a10b4ca7`.
- Platform-core checkpoint before spec work: `06582cbc939f597cdcddd249cd0f0510142ecc1b`.
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Ledger before this responsibility: `0510f39bf9645c1ccd27dc0c4b58c6ce3c0db31a`; coverage `responsibilities_total=72 unknown=16172 unread=16145 omitted=0`.

## Gap and scope decision

Fixed U declares `http_wait#9299359f max_delay:int wait_after:int max_wait:int = HttpWait`. The existing canonical/desktop protocol owner had no constructor constant, typed value, or exact parser. Fixed-U `session_private.cpp` constructs `MTP_http_wait(MTP_int(100), MTP_int(30), MTP_int(25000))` only when `_connection->usingHttpWait()`; no independent receive branch was found.

Therefore the exact wire object is independently implementable in the existing protocol owner, while Telegram Desktop-equivalent HTTP connection waiting/scheduling remains a separate lifecycle responsibility. The observed values `100/30/25000` are not used as parser validity constraints.

## Durable canonical implementation

- Spec commit: `03b40ed111c8e64d159ea837848bfae0fa85102c`.
- Implementation commit: `96bb0e2cd31cdfc82aaa44e0f8e25acf8c30a204`.
- Canonical api blob: `7b731b6d6bbd2009453c5107a24ebea58cc2ece3`.
- Canonical lib blob: `4d8b7198fbe7aa8cbb71180587970760f6ebd1d9`.

Source inspection confirms `parse_http_wait` requires constructor `0x9299359f`, preserves `max_delay`, `wait_after`, and `max_wait` as three signed TL ints in schema order, rejects wrong constructor/truncation/trailing bytes, and performs no network/session/request mutation.

## Desktop synchronization

Desktop implementation commit: `1495975bbdd2e5d56826a95854def198862e2c5b`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@7b731b6d6bbd2009453c5107a24ebea58cc2ece3`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@4d8b7198fbe7aa8cbb71180587970760f6ebd1d9`

## Authored-only contracts

Contracts cover exact field preservation, wrong constructor, every-byte truncation, and trailing-byte rejection. They were not run.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

## Remaining fail-closed scope

- Telegram Desktop-equivalent `_connection->usingHttpWait()` connection-wait scheduling/lifecycle.
- Outbound `destroy_session` lifecycle and live teardown semantics.
- Outbound `rpc_drop_answer` lifecycle and historical request/drop mutation semantics.
- `gzip_packed` bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
