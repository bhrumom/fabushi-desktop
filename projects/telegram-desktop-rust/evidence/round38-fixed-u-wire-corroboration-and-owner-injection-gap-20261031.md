# Round 38 — fixed-U wire corroboration and owner injection gap

Date: 2026-10-31  
Repository: `bhrumom/fabushi-desktop`  
Checkpoint branch: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`  
Starting live HEAD: `a5b44ec9c94768a8b9f6cf5c951541a3ec3f3cf3`  
Fixed Telegram Desktop U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Stage A guard

This round used Git/GitHub source/tree/blob reads and public source-search results only. It did not query or trigger GitHub Actions/CI and did not run build, typecheck, lint, schema validation, generators, tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance.

## 1. Fixed-U schema/legal corroboration

The exact fixed-U root tree is `91f2b083ab2877c0a17d07868eabec362a5cfca6`, previously read recursively without truncation (7,194 entries).

Exact fixed-U schema objects:
- `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`
- `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`

Exact fixed-U legal authorities:
- `LEGAL@09c9e383db122fe0694e73c48d6224676afe60f6`
- `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639`

No separate legal NOTICE/PATENTS file was observed in the exact non-truncated fixed-U tree. These fixed-U schema/legal objects are retained as migration-provenance corroboration; they do not replace the BSL-1.0 TDLib authority used for the mapped implementation inputs and do not constitute release legal acceptance.

## 2. TDLib layer/header source objects

At `tdlib/td@a17f87c4cff7b90b278d12b91ba0614383aaee82`:
- `td/telegram/Version.h@a8774804b611a7fa8b6748d31666dead255c3118` defines `MTPROTO_LAYER = 227`.
- `td/telegram/net/MtprotoHeader.cpp@a7e7bf975bf56696fcaa0da2646dc161a51b5061` supplies invoke-with-layer/init-connection header behavior.

Current `providers/telegram-protocol/src/api.rs@a5a226baa675864f142f5c815d5829bf7344fd87` contains layer 227 plus the corresponding constructor IDs. Round 38 records these exact objects in addition to round 37's TDLib schema mapping.

## 3. TL core constants

Current `providers/telegram-protocol/src/tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` contains:
- `VECTOR_CONSTRUCTOR = 0x1cb5c415`
- `BOOL_TRUE_CONSTRUCTOR = 0x997275b5`
- `BOOL_FALSE_CONSTRUCTOR = 0xbc799737`

These are mapped to exact TDLib `telegram_api.tl@c48d7914bc151a514b8c93c1b514bce48b28cb27` and fixed-U `api.tl@8319b7df...` schema objects. The TL reader/writer remains Fabushi-owned Rust code; no whole-file copy claim is made.

## 4. Expanded static shipping-owner chain

Exact current reads:
1. `mahayana-conversation/src/lib.rs@bef082eee15b811271ca7e62a84cf6221874f427` maps `telegram:` conversation IDs to provider key `telegram`.
2. `mahayana-telegram/src/lib.rs@cec9d354acc3923ab05e30af55baa4b7c814adf5` implements the shared `ConversationProvider` using the embedded Telegram runtime.
3. `mahayana-host/src/lib.rs@fd28ba534d82cc928a4c7723ec3f9c50ddd1e280` conditionally registers that provider when `HostCreateConfig.telegram_client_id` is present.
4. `mahayana-ffi/src/lib.rs@bdff6af543a30dbfb4a2df6893eff0d3322102b5` has `mahayana_runtime_force_link`, which references Telegram create/persistent-create/execute/close/free symbols in the same native artifact.
5. `providers/telegram-runtime/src/runtime.rs@787d28c05944802edfa566512366b7e5898f3a62` directly imports and composes core/network/protocol/storage auth/config/update functions.
6. `mahayana-app-host/src/lib.rs@3b87324329177dc0df00b98c644dd893ae207f27` uses a default-spread `HostCreateConfig` in the inspected desktop app-host path and does not set Telegram IDs there.

Therefore the conditional static composition is source-proven, but the actual platform-login/caller source that populates `telegram_client_id` remains an explicit production-call-chain gap. No runtime validation is inferred.

## 5. Archive member-manifest search

Boost 1.90.0 remains bound to the recorded archive size/SHA-256. Fossies search exposes archive-derived 1.90.0-vs-1.91.0 diff data at <https://fossies.org/diffs/boost/1_90_0_vs_1_91_0/index.html>, demonstrating that the archive has been parsed, but the readable result is not a complete 1.90.0 member manifest. `raw_member_listing=unread`.

GNU libiconv 1.18 remains bound to the recorded archive size/SHA-256. CrossMiNT at <https://tho-otto.m68k.eu/crossmint.php> explicitly renders `Original sources: libiconv-1.18.tar.gz <file list>`, but the complete file-list target/body is not exposed by the current reader. `raw_member_listing=unread`.

Archive bytes were not retried.

## 6. No-spin / fail-closed

dav1d original-host robots/DNS, xcb-util-m4 original GitLab, and cppgir original GitLab were not retried. Their prior fail-closed dispositions remain unchanged.

Still open: broader current-shipping provider semantic/source-informed review, actual Telegram client-id injection source trace, release legal acceptance, complete Boost/libiconv member enumeration, original-host GitLab blockers, and full fixed-U semantic/production implementation closure. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
