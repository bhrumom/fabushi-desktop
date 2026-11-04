# Round 58 — MTProto future-salts exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 27  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before implementation sync: `8ea4562c58b15051a26f6270282cc11ad4f4b424`.
- Platform-core checkpoint before spec work: `46cf745e2c228f0638d2d0afa2856579ddba3d70`.
- Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- Fixed-U `session_private.cpp` blob: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- Fixed-U `mtproto.tl` blob: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Ledger before this responsibility: `393853f290e1cb9214d165101ac738b0a64aaa71`; coverage `responsibilities_total=67 unknown=16172 unread=16145 omitted=0`.

## Gap confirmed

The canonical network branch had a local `FUTURE_SALTS_CONSTRUCTOR=0xae500895` and read only `read_i64_at(body, 4)`. A matching id immediately produced `RpcOutcome::Result` with the raw body. The canonical protocol owner had no `future_salts` parser. Therefore malformed/truncated `now`, vector metadata, nested `future_salt` objects, or trailing bytes could settle the current RPC.

Fixed-U schema is:

- `future_salt#0949d9dc valid_since:int valid_until:int salt:long = FutureSalt`
- `future_salts#ae500895 req_msg_id:long now:int salts:vector<future_salt> = FutureSalts`
- `get_future_salts#b921bd04 num:int = FutureSalts`

This receive validation is independently representable by the existing single-outstanding-RPC owner and does not require multi-request request history.

## Durable spec and canonical implementation

- Spec commit: `69167c52582d67df73c2284bb1e98a47c3846343`.
- Implementation commit: `f631d7cc0c1a21db93ab647d3b6dcb29b42cb74e`.
- Source compliance commit: `f604e40e175d6a05ef10d6c4811ec1e363c85529`.
- Canonical protocol api blob: `bdac614fab9ca16ffeb36c7e98a012fe20231597`.
- Canonical protocol lib blob: `6b6bde59a9f5e86ec33b0258e1e6a34f2b1622f9`.
- Canonical network blob: `b2eb7022c6a8d38d7ac9f4a9fd13fe22913621de`.

Implementation facts from source inspection:

1. `telegram-protocol` exports `FUTURE_SALT_CONSTRUCTOR`, `FUTURE_SALTS_CONSTRUCTOR`, `FutureSalt`, `FutureSalts`, and `parse_future_salts`.
2. The parser validates the top-level constructor, request id, server `now`, TL vector constructor/count, every nested constructor, and all nested fields.
3. Before `Vec::with_capacity(count)`, it requires `count * 20 <= reader.remaining()`, preventing an impossible declared count from driving allocation.
4. Wrong top-level/vector/nested constructors, truncation, impossible count, or trailing bytes fail parse.
5. `EstablishedSession::scan_rpc_body` calls `parse_future_salts(body)` before request matching and settlement.
6. `future_salts_matches_request` restricts settlement to the one outstanding request.
7. Matching responses preserve the exact already-validated raw TL object through the existing `RpcResult.body`.
8. No gzip, full difference decoder, or historical request/container owner is introduced.

## Desktop synchronization

Desktop implementation commit: `a3329a8c948cbffab1d6843c78abc8024afa87b4`.

Byte-identical synchronized blobs:

- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@bdac614fab9ca16ffeb36c7e98a012fe20231597`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@6b6bde59a9f5e86ec33b0258e1e6a34f2b1622f9`
- `source/mahayana/mahayana-rs/providers/telegram-network/src/lib.rs@b2eb7022c6a8d38d7ac9f4a9fd13fe22913621de`

## Authored-only contracts

The source contains contracts for exact decoding; wrong top-level/vector/nested constructors; truncation; impossible count; trailing bytes; and current-request-only correlation. They were **not run**.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

## Remaining fail-closed scope

- `gzip_packed` bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` historical bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
