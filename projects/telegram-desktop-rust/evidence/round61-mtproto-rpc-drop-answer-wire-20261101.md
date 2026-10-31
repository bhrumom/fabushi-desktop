# Round 61 — MTProto RpcDropAnswer exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 30  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before provider sync: `878c2d671d00f5d50dc149647d038913a53ea3ea`.
- Platform-core checkpoint before spec work: `38efe918eb2a6592b039cf9f2ee25c434aca2686`.
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- Ledger before this responsibility: `137b7e787898a3b411333c684730ec9ad6d28a48`; coverage `responsibilities_total=70 unknown=16172 unread=16145 omitted=0`.

## Gap and scope decision

Fixed U declares the three `RpcDropAnswer` result constructors plus `rpc_drop_answer#58e4a740 req_msg_id:long`. Canonical Fabushi had no typed parser for the three results and no outbound `rpc_drop_answer` command/call site. Therefore exact wire decoding is independently implementable in the existing protocol owner, while request-history cancellation/drop semantics depend on a missing historical request lifecycle and remain fail-closed.

## Durable canonical implementation

- Spec commit: `0c85884ee491d985e4fbf0d6ec1d1acca1a7d8a4`.
- Implementation commit: `5251ab3eacd8035cb8bb86609d76641233dd22eb`.
- Canonical api blob: `77b9cc0a7d191ef9f197fb8fd718e29631197ddc`.
- Canonical lib blob: `9229c83fa6b484a5af657ee37f0bd827fecdc28d`.

Source inspection confirms `parse_rpc_drop_answer` dispatches all three constructors. Unknown and DroppedRunning consume exactly the constructor and reject trailing bytes. Dropped consumes `msg_id:long`, `seq_no:int`, and `bytes:int`, rejecting truncation and trailing bytes. Unknown constructors fail closed. No network/session mutation or historical request state was added.

## Desktop synchronization

Desktop implementation commit: `4257fc9c875526c4b49a1f3cfbbeec7cd5a3eb0f`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@77b9cc0a7d191ef9f197fb8fd718e29631197ddc`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@9229c83fa6b484a5af657ee37f0bd827fecdc28d`

## Authored-only contracts

Contracts cover all three variants, field preservation, unknown constructor, truncation, and trailing-byte rejection. They were not run.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

## Remaining fail-closed scope

- Outbound `rpc_drop_answer` lifecycle and historical request/drop mutation semantics.
- `gzip_packed` bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
