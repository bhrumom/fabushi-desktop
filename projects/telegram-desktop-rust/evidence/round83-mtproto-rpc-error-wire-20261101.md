# Round 83 — MTProto rpc_error exact typed parser wire

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 52

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`.
The schema blob remains `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`; shipping source remains
`session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`.

Source review confirms `rpc_error#2144ca19 error_code:int error_message:string` is carried as an
`rpc_result` response body in the fixed-U shipping service-message path. The wider request lookup,
temporary-key special handling and `gzip_packed` branch are outside this isolated wire responsibility.

Canonical `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` already defines
`RPC_ERROR_CONSTRUCTOR`, `RpcErrorResponse`, and `try_parse_rpc_error`. It discriminates the constructor,
reads signed i32 + TL string, relies on the shared TL reader for padding/UTF-8/truncation, and rejects trailing
bytes. `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` and
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` remain unchanged.

Spec-first history: initial spec commit `b0c192a196afbe1f5a1a4c8c105c6c6125590c82` accidentally used an
empty base tree because the connector wrapper omitted a tree SHA; this was immediately repaired by
`095584238be5fad6a1365fb27d5cc993727d7be3`, restoring the full previous tree plus the spec. Remote readback
after repair confirmed the production api blob unchanged and spec blob
`c219f35d393bb94e29c30f579ec501bb3d42c9d5`.

Canonical authored-only contract commit `7ad951b93b57fc3c4cf33e43eaa157066d452b31` produces contract blob
`d171a3ab9a43955a693c87162f80c2667fc397ce`. It adds non-RpcError discrimination, truncation, invalid UTF-8,
trailing-byte rejection, signed extreme code and exact EOF assertions. Desktop sync commit
`885faac55fc8b7d99e4989b438e1a428ac57c1f8` mirrors that blob byte-for-byte; desktop production api/lib remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf`.

Disposition: no production delta. Responsibility
`TDRP-R9-MTPROTO-RPC-ERROR-WIRE-001` is implemented / validation_status=not-run / accepted=false.
Arbitrary request history, connection scheduling/racing/send retry, auth-key binding/persistence, gzip and complete
difference decoding/application remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


## Final source-level compliance readback

Platform-core compliance commit:
`e448cb7307ded83d9c6eab34fff0ef31f48e0e79`.

Final spec blob:
`docs/specs/telegram-mtproto-rpc-error-wire.md@89d572a9ec403835f4e1ec9d538dc3fbbe86a32d`.

Remote source readback records R1-R10 and AC-1-AC-6 as source/bookkeeping satisfied while executable Stage-B
validation remains intentionally not run. Production api/tl/lib blobs remain unchanged at
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf`; canonical/desktop authored contract blob remains
`d171a3ab9a43955a693c87162f80c2667fc397ce`. The responsibility remains
`implemented / validation_status=not-run / accepted=false`. Arbitrary request history, connection scheduling,
auth-key binding/persistence, gzip and complete difference decoding remain open/fail-closed.
