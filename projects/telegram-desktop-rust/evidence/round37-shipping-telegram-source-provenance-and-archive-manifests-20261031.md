# Round 37 — current shipping Telegram source provenance and archive-manifest search

Date: 2026-10-31  
Repository: `bhrumom/fabushi-desktop`  
Checkpoint branch: `checkpoint/tdrp-stage-a-msg-mutations-7457eb1c-r1`  
Starting live HEAD: `231202ef98f0e93845fcadebae67ebfffd143109`  
Fixed Telegram Desktop upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Execution constraints

This round used Git/GitHub commit/tree/blob/source reads and public release/source-host reads only for provenance. It did not query or trigger GitHub Actions/CI and did not run build, typecheck, lint, schema validation, generators, tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance.

The unchanged dav1d robots/DNS state and original GitLab blockers for xcb-util-m4/cppgir were intentionally not retried.

The requested `docs/specs/fabushi-deferred-ci-migration.md` path is absent at this exact checkpoint (GitHub contents read returned 404). No missing document was inferred from historical chat; this round followed the live root instructions, current SOT/spec sources that were readable, and the explicit Stage A zero-CI task instruction.

## 1. Static shipping owner chain

The current source tree establishes a static production-owner dependency path, without claiming runtime acceptance:

- workspace `source/mahayana/mahayana-rs/Cargo.toml@144c67c76daaf2229e817f9830e678a3e6d4cf3a` includes all `providers/telegram-*` crates and `mahayana-telegram`;
- `mahayana-host/Cargo.toml@cb88615198f0320f01b0cdc65f1dd2cff8047a54` directly depends on `mahayana-telegram`;
- `mahayana-host/src/lib.rs@fd28ba534d82cc928a4c7723ec3f9c50ddd1e280` imports `TelegramConversationProvider`; when `telegram_client_id` is supplied it constructs `TelegramConversationProvider::from_client_id` and registers it through the shared `ConversationProvider` builder;
- `mahayana-telegram/src/lib.rs@cec9d354acc3923ab05e30af55baa4b7c814adf5` is the shared ConversationProvider adapter and directly consumes `fabushi-telegram-core` plus `fabushi-telegram-runtime`;
- `mahayana-ffi/Cargo.toml@8b0d6b7671cacbb52760057d15edd72349716bf0` directly depends on `fabushi-telegram-runtime` and `mahayana-host`;
- `Cargo.lock@5c343a8b1bc0f88a68fd0055d372af55f4dfbf55` independently records the resolved `mahayana-host -> mahayana-telegram -> fabushi-telegram-core/fabushi-telegram-runtime` package dependency.

This is static source wiring only. No packaged or runtime call trace was executed in Stage A.

## 2. Exact TDLib authority and legal objects

Current Fabushi Telegram protocol provenance is anchored to `tdlib/td@a17f87c4cff7b90b278d12b91ba0614383aaee82`, root tree `27af7989c2c65dec54eb7860fb6932d5e422c7b5`, read recursively without truncation (1,714 entries).

The exact TDLib license is `LICENSE_1_0.txt@36b7cd93cdfbac762f5be4c6ce276df2ea6305c2` (Boost Software License 1.0). Current Fabushi `providers/telegram-protocol/LICENSE-TDLIB-BOOST-1.0.txt` is the same Git blob.

An exact recursive path-name read of this pinned TDLib tree did not expose a separate `NOTICE` or `PATENTS` file. That is an exact-tree source-authority observation only; it is not a legal-acceptance verdict.

## 3. Newly closed current-shipping file mappings

### 3.1 Schema audit baselines

Current `providers/telegram-protocol/src/schema.rs@6e2de8f02e61ec47d2e925eb5a2ad06fe16755c9` stores the TDLib pin plus expected SHA-256 and declaration counts for:

- `td_api.tl@4dac6c8b10eeb76dd5d0280c9f374281990db877`;
- `telegram_api.tl@c48d7914bc151a514b8c93c1b514bce48b28cb27`;
- `mtproto_api.tl@b2508b478b9e5dd51af1dde33adcbf87949b9a5c`.

Transformation: derived audit baseline metadata in checked-in Rust. The audit/generator was not executed.

### 3.2 Hand-written API/update wire code

`src/api.rs@a5a226baa675864f142f5c815d5829bf7344fd87` is now bound to exact `telegram_api.tl` and `mtproto_api.tl` blobs for constructor identifiers, field order and result types. Exact declaration reads include `invokeWithLayer#da9b0d0d`, `initConnection#c1cd5ea9`, `help.getConfig#c4f9186b`, `auth.sendCode#a677244f`, `auth.signIn#8d52a951`, `auth.signUp#aac7b717`, `auth.checkPassword#d18b4d16`, `dcOption#18b7a10d`, `config#cc1a241e`, `account.getPassword#548a30f5`, `account.password#957b50fb`, `rpc_error#2144ca19`, and `msgs_ack#62d6b459`.

`src/updates.rs@05cef74aec2f04f4fd73081aba0ed05383101408` is bound to `telegram_api.tl@c48d7914...` for `updates.state#a56c2a3e`, `updates.differenceEmpty#5d75a138`, `updates.differenceTooLong#4afe8f6d`, `updates.getState#edd4882a`, and `updates.getDifference#19c2f763`.

Transformation for both: hand-written Rust TL encoding/parsing from exact schema declarations; no byte-copy claim.

### 3.3 Handshake constants and behavior

`src/handshake.rs@e430fcc1344c44849873487a67bf98fda6b39ec7` is mapped to:

- exact constructor declarations in `mtproto_api.tl@b2508b478b9e5dd51af1dde33adcbf87949b9a5c`;
- the built-in safe-prime in `td/telegram/DhCache.cpp@eb9a4f3ea83fe5223c588f9d304b115a9bcefcf3`;
- the main/test PKCS#1 RSA public keys in `td/telegram/net/PublicRsaKeySharedMain.cpp@abdbff5970e2692f19b27506ebcd358401b29d56`, normalized downstream to modulus/exponent;
- source-informed behavior references `td/mtproto/Handshake.cpp@35c483f6d3ee17c1b56c26bafb6e8fe2c961e55d` and `DhHandshake.cpp@ca410d395f61afd831200ce911a635b15b89ec2b`.

Transformation: schema/data transcription plus key representation normalization plus independent Rust handshake implementation; no whole-file byte-copy claim.

### 3.4 Crypto, session and transport

`src/crypto.rs@032b6fcf384c23337facad55b8f9f11c0023dbbd` is now source-mapped to `td/mtproto/KDF.cpp@36daac0b1538cea89b897cc81ebb889182781eb4` for MTProto 2.0 KDF2 key/IV derivation and `td/mtproto/Transport.cpp@d23173e15db3abdfff6310e8daced15ab42a9761` for message-key/envelope behavior.

`src/session.rs@bc5dfce3361f7a15c26a4c7a878822dd0f0e6756` maps the server message-id -300/+30 second acceptance semantics to `td/mtproto/AuthData.cpp@56c59ae337fbf28a3ec6af2c59eb5c46ca1d4566`; sequence semantics remain public-protocol/source-informed without an asserted one-file copy.

`src/transport.rs@f4ce1cd004e58283864511d25e4ad04d1cfdcbd3` maps intermediate/padded-intermediate magic/framing and quick-ack behavior to `td/mtproto/TcpTransport.cpp@55937def171c8c6aa5d63055beca7403e9507edf`, with `Transport.cpp@d23173e...` as the packet-transport context. The Rust codec also implements full/abridged modes; no exact one-file derivation for all modes is asserted.

All three remain source-informed independent Rust implementations under release review; validation was not run.

## 4. Reviewed provider files without a new copy claim

The current blobs for Telegram core/domain/engine/feature, media, network, protocol auth/TL/module wrappers, runtime, storage and WASM adapters were reviewed for explicit generated/transcribed/upstream-source markers. No additional exact byte-copy/generated/transcribed file relationship was established for those files in this round. They are not silently reclassified as copied content merely because they implement Telegram concepts.

`providers/telegram-core/upstream-pins.json@c3743170d52b3a886eab7296314538cc9d76b8a2` is explicitly treated as legacy research metadata captured on 2026-07-10, not as fixed-U release authority: its Telegram Desktop commit is `e01ee725...`, while the current TDRP fixed U is `f985723b...`.

## 5. Boost 1.90.0 / libiconv 1.18 member manifests

### Boost 1.90.0

The official Boost release index remains readable at <https://archives.boost.io/release/1.90.0/source/> and continues to identify `boost_1_90_0.tar.gz` (210,975,925 bytes; SHA-256 `5e93d582aff26868d581a52ae78c7d8edf3f3064742c6e77901a1f18a437eea9`). The Boost SourceForge release listing is also readable, but neither path exposes a complete archive-member manifest. Archive bytes were not retried. `raw_member_listing=unread`.

### GNU libiconv 1.18

CrossMiNT remains readable at <https://tho-otto.m68k.eu/crossmint.php> and explicitly renders `Original sources: libiconv-1.18.tar.gz <file list>`. The current crawler still does not expose a complete file-list body/target. Fossies remains a per-file/diff corroboration lead, not a complete manifest. Archive bytes were not retried. `raw_member_listing=unread`.

## 6. Fail-closed disposition

Unchanged blockers were not spun again:

- dav1d original-host robots/DNS state;
- xcb-util-m4 original GitLab authority;
- cppgir original GitLab authority, still `git_tree_verified=false / fail-closed-original-host-unread`.

Broader current-shipping source-informed/copied/derived review, release legal acceptance, Boost/libiconv full member enumeration, original-host blockers, and fixed-U semantic/implementation closure remain open. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain unchanged.
