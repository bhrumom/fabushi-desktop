# Round 64 — MTProto help.configSimple exact wire validation

Date: 2026-11-01  
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 33  
Stage: A (implementation only; executable validation deferred)

## Live authority reread

- Desktop checkpoint before provider sync: `60fa3f0ad601bcffaa8acdc6e0f26a7a52bea24e`.
- Platform-core checkpoint before spec work: `3bc59bd8139a42b84f41f3817883b72aabde40cd`.
- Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
- `mtproto.tl`: `76225965bfc794388db241f3f760db8ab79e84f8`.
- `session_private.cpp`: `13cbc76ec723386027b5b4dcac79999b791c4614`.
- `special_config_request.cpp`: `fdb65506b0a886bc2daeccb61090bc44be7e2d88`.
- `special_config_request.h`: `91901489e778089b03187682a9556e929867b94f`.
- Ledger before this responsibility: `b963d8957173e55381240d83cad153b1827fe4e4`; coverage `responsibilities_total=73 unknown=16172 unread=16145 omitted=0`.

## Gap and scope decision

Fixed U declares:
- `ipPort#d433ad73 ipv4:int port:int = IpPort`;
- `ipPortSecret#37982646 ipv4:int port:int secret:bytes = IpPort`;
- `accessPointRule#4679b65f phone_prefix_rules:string dc_id:int ips:vector<IpPort> = AccessPointRule`;
- `help.configSimple#5a592a6c date:int expires:int rules:vector<AccessPointRule> = help.ConfigSimple`.

The live canonical/desktop protocol owner had no constants, typed values, or exact parsers for this family. The existing `dc.rs` owner does own DC endpoint/directory state, but fixed-U special-config consumption additionally requires web acquisition/decryption, expiry/time policy, phone-prefix filtering and endpoint callback/application semantics. That higher lifecycle cannot be precisely synthesized from the existing DC owner.

Therefore exact nested TL decoding is independently implementable in the existing telegram-protocol owner, while special-config acquisition/decryption/policy/application remains fail-closed.

## Durable canonical implementation

- Spec commit: `db4e6db92d6ebf6f03442cb3eb2134b9bf0a144e`.
- Initial implementation commit: `cde1297cacd3861764d74a111629d002e74c9d71`.
- Constructor-first fail-closed correction: `46db4422aa7221b1c8048f96f1c7c47cd0c01458`.
- Canonical api blob: `28bc9f24c7a7e676ecb7a6a8ec0eaa78edb44faa`.
- Canonical lib blob: `04e5020e7b1e7ed46b403c151f84a095a1a17e4c`.

Source inspection confirms:
- unknown IpPort constructors are rejected before variant fields are consumed;
- plain and secret variants preserve raw `ipv4:i32`, `port:i32`, and secret bytes;
- AccessPointRule preserves string, `dc_id`, nested IpPort ordering;
- HelpConfigSimple preserves `date`, `expires`, rule ordering;
- nested vectors use the existing bounded TL reader;
- all public parser entrypoints reject trailing bytes;
- no DC directory, transport, session, auth, clock, phone-policy, or request-history mutation is performed.

## Desktop synchronization

Desktop implementation commit: `870ec15e7760718ad7ef1eb5e578dfcef8f0e46b`.

Byte-identical synchronized blobs:
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/api.rs@28bc9f24c7a7e676ecb7a6a8ec0eaa78edb44faa`
- `source/mahayana/mahayana-rs/providers/telegram-protocol/src/lib.rs@04e5020e7b1e7ed46b403c151f84a095a1a17e4c`

## Authored-only contracts

Contracts cover both IpPort variants, raw field preservation, nested vector/order preservation, wrong top-level and nested constructors, every-byte truncation, and trailing-byte rejection. They were not run.

No GitHub Actions/status/checks were queried or triggered. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E, package, deployment, installation, or runtime acceptance was executed.

## Remaining fail-closed scope

- Telegram Desktop-equivalent SpecialConfigRequest web acquisition/decryption/HTTP-date/expiry/phone-prefix/DC endpoint application lifecycle.
- Telegram Desktop-equivalent `_connection->usingHttpWait()` connection-wait scheduling/lifecycle.
- Outbound `destroy_session` lifecycle and live teardown semantics.
- Outbound `rpc_drop_answer` lifecycle and historical request/drop mutation semantics.
- `gzip_packed` bounded decompression lifecycle.
- Complete updates.difference / updates.differenceSlice decoding and canonical application.
- Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` bookkeeping.
- Remaining fixed-U unknown/unread source responsibilities and source/legal closure.
- Stage-B executable evidence and independent release ACCEPT.
