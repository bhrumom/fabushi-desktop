# P0 — Revision 9 source authority, inventory and traceability closure

Status: active / fail-closed  
Project: TDRP-001 Revision 9  
Parent: FBCP-001 Revision 7  
Execution: executable validation only in GitHub Actions.

## Goal

Establish one accepted exact `telegramdesktop/tdesktop` baseline and prove complete source accounting before any global migration-complete claim.

The accepted scope is recursive and includes tracked files, direct and nested gitlinks/submodules, build-time acquisitions, patches, LFS/external objects, generated-input contracts, functional resources/locales/assets, build/packaging/updater, tests, tooling, docs and licenses.

## Hard counters

P0 cannot exit until the accepted baseline proves:

- `unknown = 0`
- `unread = 0`
- `omitted = 0`

Tree enumeration is not semantic reading. Directory citation is not responsibility understanding. A generated inventory is not permission to bulk-mark rows `understood`, `mapped`, `implemented` or `verified`.

## Required machine authorities

1. `projects/telegram-desktop-rust/upstream.lock.json` — exact candidate/accepted source authority and recursive pins.
2. `projects/telegram-desktop-rust/contracts/parity-ledger.schema.json` — Revision 9 row contract.
3. a ledger instance covering every non-directory source entry and every independently owned responsibility.
4. module dossiers containing symbols, state machines, lifecycle, failure semantics, ownership and behavior oracles.
5. `projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md` — requirement/oracle/invariant/test/evidence/reviewer chain.
6. GitHub Actions evidence from the same exact Fabushi HEAD.

## Bidirectional gate

Every applicable responsibility must resolve both directions:

`source file -> source symbol -> responsibility -> capability -> existing Fabushi owner -> target path/symbol -> shipping entrypoint -> tests/evidence`

and

`shipping entrypoint/target symbol -> owner -> capability/responsibility -> source symbol/file -> requirement/oracle/invariant -> current-head evidence`.

The validator must fail on orphan source entries, orphan target symbols, duplicate canonical roots, verified rows without exact-head evidence, or any row that bypasses existing-owner-first.

## Canonical composition gates

Revision 9 additionally requires:

- one ProductShell/navigation composition;
- one canonical Conversation/Message/Participant model family;
- one canonical Search owner for contextual/object/universal scopes and pickers;
- one typed ConversationCreation flow;
- existing Plugins/MCP/Marketplace/Computer/Automations/Task owners;
- Fabushi semantic design tokens and canonical component/screen contracts;
- no source-named second visual or product architecture.

## Current candidate

On 2026-10-07, live upstream `dev` was observed at `f23c37857220eb84f8559f0901ea26fb304b564b` with tree `782e61795af00688671ce3a03d3b646504e3cd5f`.

That observation is **candidate discovery only**. It does not become accepted because a human or connector saw it. GitHub Actions must independently re-read and validate the exact commit/tree, direct and nested gitlinks, changed build-time pins, root inventory and Revision 9 contracts before promotion.

Historical `33261535a0e747f125e0ed25486f01e556330677` evidence is historical only and cannot satisfy Revision 9.

## No-stall rule

A service/account/platform blocker blocks only its dependent responsibility. Continue all independent source reading, owner resolution, implementation, tests and service contracts. Do not hide blockers with mocks, fake local responses, disabled assertions or skipped release gates.

## Exit

P0 passes only when the accepted exact baseline, full recursive inventory, full semantic-read/responsibility coverage, owner resolution, schema/ledger validators and Revision 9 traceability/composition gates all pass on one exact Fabushi HEAD in GitHub Actions.

P0 exit is not product completion; production responsibilities, existing Bot regression, real services, signed packaged temporal acceptance and independent release `ACCEPT` remain separate gates.


## Stage-A account / embedded-client handoff implementation (2026-11-01)

Fixed upstream authority remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source review of `Telegram/SourceFiles/main/main_account.cpp@2721c7ac12effe454d398610d26d638fadaa1f68` and `Telegram/SourceFiles/storage/storage_account.cpp@afe6f61af833fd32ba02a3bd8a48f6801cb583d5` confirms the applicable source responsibility: account lifecycle owns restored authorization/session state and encrypted local persistence while the network/session instance remains tied to that account lifecycle.

Fabushi production commit `e8a8b7726679c06862205ef21fdfbfc0c5aca4ed` implements the source-neutral equivalent through existing owners: canonical Fabushi account/platform login -> existing secure secret store -> private Host credential -> AppHost `TelegramClientLease` -> typed `HostCreateConfig.telegram_client_id` -> existing `TelegramConversationProvider::from_client_id`. The lease is the only embedded client owner; ConversationProvider is explicitly non-owning. No source counts were decremented by this responsibility slice and no whole upstream file is claimed semantically closed.

State remains Stage A: implementation only, tests/contracts authored but not executed, `coverage_matrix_status=pending`, `release_gate_status=pending`, no runtime/package evidence and no acceptance.


## Stage-A Telegram authorization / canonical self-user settlement (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Production implementation through `d19fc3742b9a2d5076513ec444383d7683a35ea2` extends the **same** account-bound `TelegramClientLease` created by the canonical Fabushi account/platform-login composition. No second Telegram account, runtime, provider registry, message store, sidecar, subprocess or Web wrapper was introduced.

The embedded runtime now parses `auth.Authorization.user.id` after `auth.sentCodeSuccess`, sign-in, password and registration success and owns it as the canonical Telegram `self_user_id` only while authorization is `Ready`. Authorization/challenge state, phone-code/SRP context and MTProto auth-key/DC/salt restore material are stored through the existing account-scoped encrypted SQLite store. Restore fails closed when `Ready` lacks either self identity or auth-key material; transport reconnect reuses the persisted auth key instead of manufacturing a new Telegram runtime.

`TelegramClientLease::handoff` now reads the live runtime `AuthorizationSnapshot` and passes `client_id + authorization_state + self_user_id` into the existing `HostCreateConfig`. The existing non-owning `TelegramConversationProvider` remains attached to that one client and dynamically resolves `Ready+self_user_id`; before authorization it rejects identity-sensitive operations, while same-process authorization completion does not require a second client or Host recomposition.

Authored-only contracts cover authorization user-id parsing, encrypted private metadata, Ready restore/fail-closed restore, and Host handoff consistency. Per Stage A policy, no build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed or queried. This slice is `implemented / validation_status=not-run / accepted=false`; broader fixed-U semantic closure, Boost/libiconv member enumeration, original-host GitLab blockers, release/legal acceptance, Stage-B evidence and independent ACCEPT remain open.


## Stage-A canonical Telegram login entry / private application credential (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact fixed-U source reads anchor the applicable login behavior to `intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246` (`auth.sendCode`), `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489` (`auth.signIn`), `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e` (`auth.checkPassword`) and `intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96` (`auth.signUp`). These are behavioral provenance anchors only; no whole intro file is claimed semantically closed and global unknown/unread counts remain unchanged.

Production implementation through `c8109e3412d8c2cb4e7362a0fe4ec4976c07bd22` keeps existing owners: `FabushiAuthService` reads an application-scoped `api_id/api_hash` pair from the existing protected main-process secret store, validates the exact bounded shape, and copies it only through the existing private mode-0600 Native Host credential. The private Rust credential reader retains the pair in process memory; `TelegramApplicationCredentials` redacts `api_hash` from Debug. Renderer-facing methods never accept or return either application credential.

The existing production renderer/Coordinator carrier now exposes typed `feature.auth.telegramStatus`, `telegramSendCode`, `telegramSignIn`, `telegramPassword` and `telegramRegistration` calls. AppHost routes all of them to the same process-lifetime `TelegramClientLease`. First send-code bootstraps transport only when no live/restorable transport exists; restored auth-key material is reused. Every authorization mutation rereads the same runtime `AuthorizationSnapshot` and returns only renderer-safe authorization state/self-user identity while the existing Host/ConversationProvider continues to resolve that same client.

Actual Telegram application credential values remain external product configuration and are intentionally absent from repository and renderer state; missing/malformed configuration fails closed. Authored contracts cover application-credential bounds, private Native parsing and renderer parameter secrecy. Per Stage A policy, no build, typecheck, lint, schema validator, generator, test, package, deployment, installation, workflow/status query or runtime acceptance was executed. State is `implemented / validation_status=not-run / accepted=false`.


## Stage-A Telegram auth lifecycle typed error/backoff contract (round 44, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual exact-source reads covered `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`, `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`, `intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`, `intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246` and `intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96`. These sources establish resend/cancel, password recovery, email verification, flood handling, challenge invalidation and bounded retry/error lifecycle responsibilities; they are behavioral anchors only and do not close the whole intro files.

Production implementation through `a9e9683a3bf8cd05fd79b46213c19f56bc7c606d` plus the manual bounded-number guard correction `e7c0da2a654aad9a65e6c7fc545f9cf7d9dd48a8` keeps the existing owner chain. The retained `TelegramClientLease` now classifies bounded Telegram auth errors, turns `FLOOD_WAIT_*` / `FLOOD_PREMIUM_WAIT_*` into a capped same-process backoff, prevents hot-looped same-lease auth mutations while that deadline is active, and always projects the live `AuthorizationSnapshot/self_user_id` on recognized issues. The shared Coordinator contract now has a discriminated Telegram authorization-state union plus bounded error/backoff validation; the production renderer client consumes that shared type rather than `unknown`. AppHost status remains routed to the same lease. No second account/runtime/provider/state store was added.

The current embedded runtime was also inspected at `bhrumom/fabushi@edde714def34e6eb5ae133dba20790c80a355c85`: it implements send-code, code submit, password submit and registration, but has no protocol command/builder path for `auth.resendCode`, `auth.cancelCode`, `auth.requestPasswordRecovery`, `auth.checkRecoveryPassword`, `account.sendVerifyEmailCode` or `account.verifyEmail`. Those remote lifecycle operations therefore remain explicit fail-closed implementation blockers instead of being fabricated in renderer/AppHost code. The source-specific `SRP_ID_INVALID` challenge refresh likewise remains blocked on a safe runtime `account.getPassword` refresh primitive; the typed projection records the required action without retry-spinning.

Authored-only contracts are the inline Rust auth-error/backoff test and `source/shared/rpc/coordinator.contract.test.ts`; neither was run. Stage A remained execution-free: no GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, packaging, deployment, installation or runtime acceptance occurred. The ledger adds `TDRP-R9-ACCOUNT-TELEGRAM-AUTH-LIFECYCLE-ERROR-BACKOFF-001`; `responsibilities_total=56`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force.


## Stage-A remaining Telegram auth lifecycle primitives (round 45, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact TL authority is `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`; fixed-U intro behavior remains anchored by `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`, `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`, and `intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`.

Canonical shared owner work is committed on `bhrumom/fabushi-platform-core` checkpoint `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: protocol `314e8804ddfc997acdde2c19bff831b5e26b1706`, encrypted storage `bcb8298a4622616f2f8a226899c0e3dc8aa3ba8f`, runtime `ec854e93fb1523d95225203554600858798e4981`, and spec/evidence `a50f33da685506f4e170729a9d11f01f8dda8ec5`. These add auth.resendCode, auth.cancelCode, auth.requestPasswordRecovery/auth.checkRecoveryPassword, account.sendVerifyEmailCode/account.verifyEmail, explicit account.getPassword challenge refresh, and encrypted pending AuthorizationState/AuthContext persistence.

Desktop implementation `169d6cc7798507f0302d6c93fef7deef1a302d28` wires the matching seven commands through the existing `TelegramClientLease`, AppHost `feature.auth`, shared Coordinator contract and production renderer client. No second account/runtime/provider/state store was added, and no api_hash is exposed to renderer code.

The current desktop checkpoint is a sparse overlay and contains no Cargo/dependency manifest, so exact binding of its `fabushi_telegram_runtime` dependency to the platform-core checkpoint cannot be proven from this ref. That remains fail-closed integration evidence, not silently treated as shipping validation.

All protocol/storage/runtime/coordinator tests are authored only. No build, typecheck, lint, schema validator, generator, test, packaging, deployment, installation, workflow/status query or runtime acceptance was executed. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.

## Stage-A Telegram runtime Cargo composition fusion (round 46, 2026-11-01)

The previously recorded sparse-checkpoint composition blocker is now statically closed without executing validation. The actual Mahayana Cargo authority is preserved exactly: `source/mahayana/mahayana-rs/Cargo.toml@144c67c76daaf2229e817f9830e678a3e6d4cf3a` includes `mahayana-telegram` and `providers/telegram-runtime`; `mahayana-telegram/Cargo.toml@f43109356f1f486a82a2d0120f6e78808c431c40` path-depends on `fabushi-telegram-runtime = { path = "../providers/telegram-runtime" }`; and `providers/telegram-runtime/Cargo.toml@0fc8ee357ef5c00074e5fdcd1d7594860e289bd1` retains the sibling protocol/network/storage path dependencies.

Canonical shared-owner fusion is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@13f5458aadf19a87c52bfa904f9942d81c88979a`. It preserves the prior `AuthorizationSnapshot/self_user_id` and resumable MTProto auth-key baseline while retaining resend/cancel/recovery/email-verification/account.getPassword challenge-refresh primitives. Protocol success parsing now carries canonical `auth.Authorization.user.id`; encrypted storage is schema v4 and converges both historical v3 table shapes (`private_metadata` and `secure_value`) without creating a second persistence owner.

Desktop commit `43d9d398801f2885da982d106c17a74efb1f9472` synchronizes those canonical protocol/network/storage/runtime sources into the existing `providers/telegram-*` path dependency chain; authored-only composition/runtime/storage/protocol contracts are saved through `221a661e17f2ad417e349df6604815d3fbfa946f`. The existing checkpoint `mahayana-telegram/src/lib.rs` lease/AppHost/Coordinator wiring is preserved; no second account, runtime, provider, session or state store is introduced.

This is Stage-A static implementation/composition evidence only. No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed. `responsibilities_total=57`, `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged; `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force. Broader fixed-U source/implementation closure, complete Boost/libiconv archive member enumeration, original-host GitLab blockers, legal/release review, Stage-B executable evidence and independent ACCEPT remain open.


## Stage-A MTProto bad-message recovery (round 47, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The next build/shipping-reachable responsibility was manually read at `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, corroborated by `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. Fixed-U `bad_msg_notification` codes 16/17/64 correct trusted server time and resend, while 32/33 reset inconsistent session sequence state; unrelated/fatal codes do not silently recover.

Existing-owner-first implementation is committed in `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16@ac9e20217812aa72b92049d4dfb52f76d6380da6` after durable spec correction `47ed28d35dbb6c7dd19bdcf1cd66fbf88dc68f67`. Canonical blobs are `native/telegram-network/src/lib.rs@ba9935b54e9c178c7d9377cd75572f2c21b62144` and `native/telegram-protocol/src/session.rs@ab8fd351a05543854c3f62de5c8332a13b7ff472`. Recovery remains in the unique `EstablishedSession`: 16/17/64 update the live clock offset, clear only the client message-id floor while preserving server replay history, and retry once; 32/33 rotate the session id/reset `SessionSequence` and retry once; mismatched/fatal/repeated recovery fails closed.

Desktop implementation commit `6c714e664d6cd4d66228b313664a6e776a4fd558` synchronizes those exact blobs into the existing Cargo path-dependency chain at `providers/telegram-network` and `providers/telegram-protocol`; no second account, runtime, provider, transport, auth key, session owner or state store was introduced. Inline protocol/network recovery contracts are authored only and were not run.

The ledger adds `TDRP-R9-MTPROTO-BAD-MESSAGE-RECOVERY-001`; `responsibilities_total=58`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged because this is a responsibility slice rather than whole-file semantic closure. No GitHub Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was executed. `baseline_ready=false`, `accepted=false`, and `validation_status=not-run` remain in force. Broader fixed-U closure, archive/legal blockers, Stage-B executable evidence and independent release ACCEPT remain open.


## Stage-A MTProto new-session pending-RPC replay (round 48, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U read of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` lines around `mtpc_new_session_created`, corroborated by `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`, shows that the session adopts the new salt and resends pending requests whose message ids are older than `first_msg_id`.

The canonical shared owner remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `8275952bed7bea3a7c275f1b122c2c9c710ba262` precedes implementation commit `6ec2d64da2d30d78c919bd971e8e0d1953aed587`; the network blob is `7df3dd74ed1cc4fea5273a36152d1ff8bd00ec65`. Fabushi's synchronous `invoke_raw` has one outstanding RPC, so the fixed-U pending-request responsibility maps exactly to that request: if its id is less than `first_msg_id`, update `server_salt` and replay once with a fresh id on the same auth key/transport/runtime; equal/newer requests continue waiting; a repeated replay signal fails closed with `NewSessionReplayLimit`. Ping adopts the salt but does not fabricate content-RPC replay.

Desktop commit `01b78f13e4a500bdb401c080a40f5295d8110779` synchronizes the exact network blob into the existing `providers/telegram-network` path dependency. The ledger adds `TDRP-R9-MTPROTO-NEW-SESSION-REPLAY-001`; `responsibilities_total=59`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Higher-level propagation of `new_session_created` into update/difference recovery remains a separate open responsibility and is not claimed by this slice.

Inline predicate/replay contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, test, package, deployment, installation or runtime acceptance was performed. `accepted=false`, `validation_status=not-run`.


## Stage-A MTProto detailed-answer resend (round 49, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source authority remains `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` plus `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`: matching `msg_detailed_info#276d3ec6` acknowledges the sent request and requests missing `answer_msg_id` through `msg_resend_req#7d861a08`.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec `9bea551d1e26a4ff5efa522417999388256486ff`, implementation `0536aa2ad9f93e59f1b1e3cdc19aabaf9a104fce`, protocol api/lib blobs `1feaefd062f4e8635bbf002a7dce30b804c9f6ed` / `3e17847045f3be606d8b821c85f7b8136b89474a`, network blob `581156c32289082ad1f53410ab0637bdeef85c66`. The existing protocol owner adds a fixed-U `build_msg_resend_req`; the existing synchronous `EstablishedSession` sends that non-content service request only when `msg_detailed_info.msg_id` equals the current outstanding RPC, then keeps waiting.

Desktop `3f5e2d3a8b2e0c2726a1b1dcd1ab1172a0c5758d` synchronizes those exact blobs into the existing provider path chain. No second request history, runtime, session, provider or transport owner is introduced. `msg_new_detailed_info` and `msgs_state_req/info/all_info` full reconciliation remain explicitly open.

Ledger adds `TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001`; `responsibilities_total=60`, `unknown=16172`, `unread=16145`, `omitted=0`. Contracts are authored only; no Actions/status query or trigger, build, typecheck, lint, validator, generator, tests, packaging, deployment, installation or runtime acceptance occurred. `accepted=false`, `validation_status=not-run`.


## Stage-A MTProto single-RPC state reconciliation (round 50, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U source review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` plus `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` closed the subset of `msg_new_detailed_info`, `msgs_state_req`, `msgs_state_info`, `msgs_all_info`, and `handleMsgsStates` that Fabushi's existing one-outstanding-RPC owner can represent exactly. The upstream low-three-bit rule is preserved: `(state & 0x07) == 0x04` means the server received the request; other states require replay.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `469e46f72d15138aa4fb343a49e06feeaefb1303` defines the single-RPC boundary; implementation commit `404623cad89e0d77ab326e140784b893d677f31e` adds `msgs_state_req` encoding, bounded `msgs_state_info/msgs_all_info` parsing, one current state-query id, one bounded state-driven replay through the existing outer retry loop, and `msg_new_detailed_info` answer reconciliation through the existing server-message replay window. Canonical blobs are protocol api `ed695ed575c5950c99818acc7086a2cf8b1c712d`, protocol lib `21ec771260dd356d4a8e653be6e2e250a32ab5ce`, and network `c24515c2ee15608f51970a5420769e7713e60327`.

Desktop implementation commit `fe17d4bcdea22d1df8e474961aa12b5f04b291db` synchronizes those exact blobs into the existing provider path chain. No second request history, runtime, session, provider, transport or received-message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `_stateAndResendRequests`, simultaneous state queries, queued-resend cancellation, container-wide resend bookkeeping, or durable multi-request history remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-STATE-RECONCILIATION-001`; `responsibilities_total=61`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC acknowledgement (round 51, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next service-message slice that the existing one-outstanding-RPC owner can represent exactly: `mtpc_msgs_ack` plus the `requestsAcked(..., byResponse=false)` rule that a normal ACK does not settle a callback RPC still waiting for a response.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `9f72142c7b4d8ab0e605e95301f4194d964ffa73` defines the single-RPC acknowledgement boundary; implementation commit `3470cd572891222c8846a9e91f98ca621624ec64` adds bounded `msgs_ack` parsing, explicit RPC/ping ACK dispatch, current-request correlation, malformed-payload fail-closed behavior, and callback-RPC non-settlement. Canonical blobs are protocol api `d38546a0f6954bb6a03aaec80968dbf195945edc`, protocol lib `da012dab3b1111f18c318cf0001f2f3d1f32bf6f`, and network `70645dd06426374ae4e20bd9c9545388948f65bb`.

Desktop implementation commit `6804906c0646d75a16b150b4f745476f44e19953` synchronizes those exact blobs into the existing provider path chain. No second request history, sent/resend queue, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, container ACK fan-out, queued-resend cancellation, or per-request fast-clock history remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-ACKNOWLEDGEMENT-001`; `responsibilities_total=62`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-ping pong correlation (round 52, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next adjacent service-message slice that the existing single-outstanding-ping owner can represent exactly: `mtpc_pong` / `pong#347773c5 msg_id:long ping_id:long`. Telegram Desktop additionally consults historical `wasSent`; Fabushi intentionally maps only the current-ping correlation that does not require a multi-request sent history.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `47246159887072673aff6609fb57f9cbc81ff2a2` defines the single-ping boundary; implementation commit `bf6f063b229be43597bddc4231baab24d06a09bd` adds bounded `parse_pong`, strict trailing-byte rejection, explicit `Pong` shape, and `pong_matches_request` correlation requiring both the current request message id and ping id. Canonical blobs are protocol api `32a7610bfc74835734c1e5c4385e4eb4cb2ee1f3`, protocol lib `ffbabc8f66112c2321be75a34928b086d6adf57f`, and network `e0bf1b5f286ad0f8bc45a36e86074e19b7dde560`.

Desktop implementation commit `3e363ccdaf287878f457797442aa97e8b597384b` synchronizes those exact blobs into the existing provider path chain. A valid exact pong settles only the current ping; either id mismatch uses the existing `PingMismatch` failure path, and malformed/trailing payload fails closed as `InvalidServiceMessage`. No second request history, ping history, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, historical sent-message lookup, container fan-out or queued-resend cancellation remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-PING-PONG-001`; `responsibilities_total=63`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC result correlation (round 53, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` selected the next adjacent service-message slice that the existing one-outstanding-RPC owner can represent exactly: `mtpc_rpc_result` / `rpc_result#f35c6d01 req_msg_id:long result:Object`. Telegram Desktop additionally consults historical `wasSent` and multi-request acknowledgement state; Fabushi intentionally maps only current-request correlation and leaves that history open/fail-closed.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `6511c5f887c16caa0494918c987fb400774c4349` defines the single-RPC-result boundary; implementation commit `144512ce2bb7366aef57ba76924b2d02852cbce3` adds bounded `parse_rpc_result`, explicit `RpcResultEnvelope`, validation of non-empty word-aligned opaque result bytes, and `rpc_result_matches_request` correlation against only the current request. Canonical blobs are protocol api `b8b9872ec8ce7548817ebbe92615d47aed470e89`, protocol lib `4b6695cac59dc507f76c85f93ff3678d0d5bcbd0`, and network `8ec039d4a12040fcea87af883dba9cd02a2b3efb`.

Desktop implementation commit `7dc33be41568162c2cab398da67b3e6bc9c12dea` synchronizes those exact blobs into the existing provider path chain. A well-formed result settles only when `req_msg_id` equals the current `request_message_id`; unrelated result ids continue waiting, and malformed/empty/non-word-aligned envelopes fail closed as `InvalidServiceMessage`. No second request history, callback registry, runtime, provider, transport, session or message store is introduced. Telegram Desktop semantics requiring arbitrary `wasSent/haveSent`, `_resendingIds`, `toSend`, `_sentContainers`, container fan-out, queued-resend cancellation or multi-request callback routing remain explicitly open and fail-closed.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-RESULT-001`; `responsibilities_total=64`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Protocol/network contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto single-RPC message-container traversal (round 54, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` selected the adjacent `mtpc_msg_container` receive-side branch. Fixed U requires a bounded contained-message count, per-entry `msg_id/seq_no/bytes`, body length of at least one TL word and word alignment, ACK collection for odd `seq_no`, replay registration, and recursive service handling. This round maps only the receive-side subset that Fabushi's existing current RPC/current ping owners can represent; Telegram Desktop's outbound `_sentContainers` and arbitrary historical request fan-out remain open/fail-closed.

Canonical shared ownership remains `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`. Spec commit `b223407bc9829a2fc14449bb5a7cfe6d7f4b082e` defines the bounded receive-container boundary; implementation commit `98a2ff425407725571966a4ed17b1643fba87b4d` adds `parse_message_container`, `MessageContainer/MessageContainerEntry`, 1024-entry bounding, minimum/aligned body checks, exact full-buffer consumption, and reuses the typed parser in both current-RPC and current-ping network paths. Canonical blobs are protocol api `14c282bfc20a33446c69a47db4fdd6f0b887348a`, protocol lib `f21d38495e5a7f5c6e2a35f28a633ac6e3433d4b`, and network `bab8893bd8a2a0632014d826503a8091cfe64d46`.

Desktop implementation commit `e8091133b85eb7e368839e6c9b09547620be17c2` synchronizes those exact blobs into the existing provider path chain. The existing `MessageIdGuard` still validates every contained server message id before recursion; the RPC path adds only odd-sequence contained ids to its existing acknowledgement accumulator; nested results/pongs/recovery continue through existing exact current-request/current-ping correlation. No second request history, sent-container store, callback registry, runtime, provider, transport, session or message store is introduced.

Ledger adds `TDRP-R9-MTPROTO-SINGLE-RPC-MESSAGE-CONTAINER-001`; `responsibilities_total=65`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Parser/sequence-parity contracts are authored only. `gzip_packed` decompression nesting/budget semantics and outbound/multi-request `wasSent/haveSent/_resendingIds/toSend/_sentContainers` remain open and fail-closed. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto new-detailed-info exact wire validation (round 55, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` confirmed the adjacent `mtpc_gzip_packed` path requires `kMaxGzipNesting=64`, a shared `unpackedBudget` initialized to 32 MiB, and zlib/gzip decompression. The canonical `native/telegram-protocol/Cargo.toml@d8a5261034b1eb395d12149a6bcc34cd7e371b73` and `Cargo.lock@97ad9046ce8926764258c978664cd8cd92fce503` contain no approved `flate2`/\`miniz_oxide` decompression owner, so gzip remains explicitly fail-closed rather than introducing an unreviewed dependency.

The next independently implementable responsibility is the complete fixed-U wire contract for `msg_new_detailed_info#809db6df answer_msg_id:long bytes:int status:int` from `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. Canonical spec commit `40d53cfb44f2a495e4bf1a84002f653f555532e5` precedes implementation commit `5bdf993dc1048ed22194d7f1c0439049e820ee5f`. The existing protocol owner now exports `MsgNewDetailedInfo`, `MSG_NEW_DETAILED_INFO_CONSTRUCTOR`, and `parse_msg_new_detailed_info`; the existing `EstablishedSession::scan_rpc_body` uses that parser before the existing detailed-answer ACK/resend reconciliation. Canonical blobs are protocol api `eff20dbc601ca78fa7e6a4662bfeda088706d4f4`, protocol lib `b6ab0397cbad0776b269d965f89427030824ba4c`, and network `7f3b140a9becda2305a2f5e252fcea61108597e0`.

Desktop implementation commit `3b37ff49e7c3b100af12206ab88662f4d83959c2` synchronizes those exact blobs into the existing provider path chain. Truncated, wrong-constructor, or trailing `msg_new_detailed_info` payloads now fail as `InvalidServiceMessage` before an ACK or `msg_resend_req` side effect; a valid message still reuses the current session replay window and existing resend primitive. No second request history, decompression runtime, provider, transport, session or message store was introduced.

Ledger adds `TDRP-R9-MTPROTO-NEW-DETAILED-INFO-WIRE-001`; `responsibilities_total=66`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact-field and malformed-payload contracts are authored only. `gzip_packed`, `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, higher-level new-session update/difference propagation, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto new-session wire validation and high-level difference blocker (round 56, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review bound the low-level service object to `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `new_session_created#9ec20908 first_msg_id:long unique_id:long server_salt:long` in `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8`. The adjacent high-level chain is `Api::Updates::mtpNewSessionCreated -> _updatesSeq=0 -> getDifference`; full/slice differences are applied before intermediate/final state advancement in `api_updates.cpp@71c929cfe3c779c94719efef32a47a4d029e2c67`.

Existing-owner-first review found the high-level recovery is not yet precisely implementable: canonical `telegram-protocol/src/updates.rs` can build `updates.getDifference` and parse only update state plus terminal empty/too-long differences, while no current protocol/runtime bridge decodes and applies arbitrary `Message/EncryptedMessage/Update/Chat/User` vectors from `updates.difference#f49ca0` or `updates.differenceSlice#a8fb1981` into the existing TelegramEngine owners. A terminal-only recovery would drop missed updates or advance state without applying them, so this responsibility is recorded fail-closed in platform-core spec `telegram-new-session-update-difference-recovery.md` from spec commit `ba9143b660d58728de8b93a49e7c2f97fcd39ed5`.

The next independent responsibility, exact new-session wire validation, is implemented instead. Platform-core implementation `dc66cc7aae7644530fa03160438df5174809beb4` adds `NEW_SESSION_CREATED_CONSTRUCTOR`, `NewSessionCreated`, and `parse_new_session_created` to the existing protocol updates owner, consuming all three fixed-U fields and rejecting wrong constructors, truncation, and trailing bytes. Both `EstablishedSession::scan_rpc_body` and `process_service_body` now parse the complete typed object before server-salt mutation or the existing bounded current-RPC replay decision. Canonical blobs are protocol updates `23a09bb6e4e78ef526823cf01d0a207565fb2a3e`, protocol lib `7b9c641eabccb7dba8c5a425904b6a676b12d866`, and network `234c6c0070a1a92218fe5c974658d16d37fee46a`.

Desktop implementation `23b24ab2db29697071d21eb747f475b2196d891f` synchronizes those exact blobs into the existing provider composition. The provider protocol tree did not previously contain `src/updates.rs` even though its synchronized `lib.rs` declares `pub mod updates`; this round adds the canonical file at that declared existing module slot rather than introducing a second owner.

Ledger adds `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001`; `responsibilities_total=67`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact-field/malformed contracts are authored only and were not executed. High-level full/slice difference recovery, `gzip_packed`, historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed. No Actions/status/check query or trigger, build, typecheck, lint, validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `accepted=false` and `validation_status=not-run` remain in force.


## Stage-A MTProto detailed-info exact wire hardening (round 57, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirmed that `msg_detailed_info#276d3ec6` is a four-field object (`msg_id:long answer_msg_id:long bytes:int status:int`) and Telegram Desktop performs generated TL decoding before using it. The existing Fabushi network branch still read only the first two fields by raw offsets, so malformed/trailing objects could reach the current-session ACK/resend path.

Spec-first canonical work remains on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `8fffe0ca4e7d48d097e9f15f0cf332610e28f4fc`, implementation commit `4c5bea7b847a3e04ee3d0c9b05fab228101acb64`, and source-level compliance commit `46cf745e2c228f0638d2d0afa2856579ddba3d70`. Canonical blobs are protocol api `5da871fc557a49a7e57f1dd0677f307a5ed1bac8`, protocol lib `3f7c363f9fd2eb86e60c8294144d7f1dced9ccca`, and network `2a50068d148b25e710961973315d78fc1b1ee5a8`. The protocol owner now exports `MSG_DETAILED_INFO_CONSTRUCTOR`, `MsgDetailedInfo`, and `parse_msg_detailed_info`; wrong constructor, truncation, and trailing bytes fail before network side effects. The network owner parses the complete object first, then keeps the existing one-outstanding-RPC current-id gate and `reconcile_detailed_answer` behavior.

Desktop implementation commit `45c24752b9d4764768a9eec26ead9fb6b36dc426` synchronizes those three blobs byte-for-byte into the existing provider chain; no second Telegram protocol/runtime/provider, request history, callback registry, or received-message store was introduced. The existing ledger responsibility `TDRP-R9-MTPROTO-DETAILED-ANSWER-RESEND-001` is updated rather than creating a duplicate responsibility; `responsibilities_total=67`, `unknown=16172`, `unread=16145`, `omitted=0` remain unchanged.

Exact-field, wrong-constructor, truncation, trailing-byte, and current-request contracts are authored only. No Actions/status query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference decoding/propagation, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain explicitly open and fail-closed.


## Stage-A MTProto future-salts exact wire validation (round 58, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Reread of `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms `future_salts#ae500895 req_msg_id:long now:int salts:vector<future_salt>` and nested `future_salt#0949d9dc valid_since:int valid_until:int salt:long`. The existing Fabushi receive branch still recognized `ae500895` locally, read only `req_msg_id` from raw offset 4, and returned the entire body for a matching current request without validating `now`, the TL vector, nested constructors/fields, or trailing bytes.

Existing-owner-first review found this slice is independently representable by the canonical one-outstanding-RPC owner: complete response validation and current-id correlation require no historical `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state. Platform-core spec commit `69167c52582d67df73c2284bb1e98a47c3846343` defines that boundary. Implementation commit `f631d7cc0c1a21db93ab647d3b6dcb29b42cb74e` adds `FUTURE_SALT_CONSTRUCTOR`, `FUTURE_SALTS_CONSTRUCTOR`, typed `FutureSalt/FutureSalts`, and `parse_future_salts` in the existing protocol owner. It validates top-level/vector/nested constructors and all fields, checks the declared count's fixed-size byte requirement against remaining input before vector allocation, and rejects truncation or trailing bytes. The network branch parses first, then settles only when the typed `request_message_id` equals the one outstanding RPC, preserving the exact validated raw object as the existing `RpcResult.body`. Canonical blobs are api `bdac614fab9ca16ffeb36c7e98a012fe20231597`, lib `6b6bde59a9f5e86ec33b0258e1e6a34f2b1622f9`, network `b2eb7022c6a8d38d7ac9f4a9fd13fe22913621de`; source-level compliance commit is `f604e40e175d6a05ef10d6c4811ec1e363c85529`.

Desktop implementation commit `a3329a8c948cbffab1d6843c78abc8024afa87b4` synchronizes those three blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001`; `responsibilities_total=68`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was added.

Exact-field, wrong top-level/vector/nested constructor, truncation, impossible-count, trailing-byte, and current-request contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto bad-server-salt exact wire validation (round 59, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms that `bad_server_salt#edab447b` is a four-field object: `bad_msg_id:long bad_msg_seqno:int error_code:int new_server_salt:long`. Telegram Desktop first decodes the generated TL object and then consumes all four fields. The existing Fabushi RPC and ping/service branches still used raw offsets for only `bad_msg_id`, `error_code`, and `new_server_salt`, leaving `bad_msg_seqno` untyped and allowing trailing bytes before live salt mutation.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `12bc8a0ce1e453af1b3f2af0ffbd9ce868c92447`, implementation commit `3a850171c59a5f27809e176ba54971c7c0c003dc`, and final source-level compliance commit `515857196fd5fd8688326c7971a60079b67a0b71`. Canonical blobs are protocol api `75959eb82460cb36bf5f43c6e4470ef8814ea2a9`, protocol lib `3540f1a60df0d07f77b6e38349e0c500ec2d12d8`, and network `42be9e9fa78c3acc77367799c1ecb506cdd25ab0`. The protocol owner now exports `BAD_SERVER_SALT_CONSTRUCTOR`, typed `BadServerSalt`, and `parse_bad_server_salt`; it consumes `bad_msg_id`, `bad_msg_seqno`, `error_code`, and `new_server_salt`, and rejects wrong constructors, truncation, and trailing bytes. Both network receive branches parse first, then correlate only the one outstanding request before assigning the exact parsed server salt and preserving the existing bounded `NewSalt` retry outcome.

Desktop implementation commit `131980d0bc3c5b6e8fad3ad2e394715426cd1529` synchronizes those three canonical blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-BAD-SERVER-SALT-WIRE-001`; `responsibilities_total=69`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was introduced.

Exact-field, wrong-constructor, truncation, trailing-byte, both-receive-branch, and current-request contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto bad-msg-notification exact wire validation (round 60, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `Telegram/SourceFiles/mtproto/scheme/mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms `bad_msg_notification#a7eff811 bad_msg_id:long bad_msg_seqno:int error_code:int`; Telegram Desktop first decodes the generated TL object and then reads all three fields. The existing Fabushi RPC and ping/service branches still read only `bad_msg_id` at raw offset 4 and `error_code` at raw offset 16, while `direct_bad_message_time_recovery` also granted its clock-window exception from raw constructor/error-code offsets. This skipped typed `bad_msg_seqno` validation and allowed trailing bytes to reach recovery behavior.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `42d90d6351a31f6d0815c47f2d647ae8162eccf8`, implementation commit `0e569130a80a53b48b74f8e27a96004363b30e7e`. Canonical blobs are protocol api `15e34bbb0e9c5a5d4fdd8b7a9cdedf3d6b123080`, protocol lib `2c4897cfdba4502d8ca880d7a58cfd0e88877bf7`, and network `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. The protocol owner now exports `BAD_MSG_NOTIFICATION_CONSTRUCTOR`, typed `BadMsgNotification`, and `parse_bad_msg_notification`; it consumes `bad_msg_id`, `bad_msg_seqno`, and `error_code`, rejecting wrong constructors, truncation, and trailing bytes. Both network receive branches parse first before current-request correlation or recovery action, and the direct clock-window exception now requires the same fully valid typed object. The decoded sequence number is retained without introducing historical sent/container reconciliation.

Desktop implementation commit `1205f8709995aa4f5fd5dafdc9fb5121fbfce06c` synchronizes those three canonical blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-BAD-MSG-NOTIFICATION-WIRE-001`; `responsibilities_total=70`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No second Telegram runtime/provider/request history was introduced.

Exact-field, wrong-constructor, truncation, trailing-byte, current-request correlation, recovery-code mapping, and malformed-clock-window contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain explicitly open/fail-closed.


## Stage-A MTProto RpcDropAnswer exact wire validation (round 61, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `rpc_answer_unknown#5e2ad36e`, `rpc_answer_dropped_running#cd78e586`, and `rpc_answer_dropped#a43ad8b7 msg_id:long seq_no:int bytes:int`. The fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `rpc_answer_*` branch; these are schema-defined `rpc_drop_answer` result bodies.

Spec-first canonical work: platform-core spec `0c85884ee491d985e4fbf0d6ec1d1acca1a7d8a4`, implementation `5251ab3eacd8035cb8bb86609d76641233dd22eb`; canonical blobs api `77b9cc0a7d191ef9f197fb8fd718e29631197ddc`, lib `9229c83fa6b484a5af657ee37f0bd827fecdc28d`. The existing protocol owner now has typed `RpcDropAnswer` variants and `parse_rpc_drop_answer`, with exact zero-field/trailing validation for Unknown/DroppedRunning and exact `msg_id/seq_no/bytes` parsing plus truncation/trailing rejection for Dropped. Desktop implementation `4257fc9c875526c4b49a1f3cfbbeec7cd5a3eb0f` synchronizes both protocol blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001`; `responsibilities_total=71`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No outbound `rpc_drop_answer` production call path or historical request registry exists, so interpreting these results as drop/cancel state mutations remains explicitly fail-closed; no `wasSent/haveSent/_resendingIds/toSend/_sentContainers` semantics were invented.

Parser contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.


## Stage-A MTProto DestroySessionRes exact wire validation (round 62, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `destroy_session_ok#e22045fc session_id:long`, `destroy_session_none#62d350c9 session_id:long`, and `destroy_session#e7512126 session_id:long`. The fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `destroy_session_*` receive branch, so wire decoding is separated from any live teardown lifecycle.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `dee7cff150070d7891478510386674970a6217af`, implementation `cfe6a0564c40d2c9b9ea62b686470139a1eeb2ab`; canonical blobs api `d82cca134274b8d939c1140ec44ee00230e2cfac`, lib `655498ce9db2faf8a76f093250d4a1f02f77ff67`. The existing protocol owner now exports typed `DestroySessionResult::{Ok,None}` and `parse_destroy_session_result`; both variants preserve exactly one `session_id:long` and reject unknown constructors, truncation, and trailing bytes. Desktop implementation `b179796344fba3e22bccb1310b74f229d613b531` synchronizes both protocol blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-SESSION-WIRE-001`; `responsibilities_total=72`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. No canonical outbound `destroy_session` production path or live teardown owner exists, so interpreting Ok/None as session mutation remains explicitly fail-closed; no `wasSent/haveSent/_resendingIds/toSend/_sentContainers` semantics were invented.

Parser contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.


## Stage-A MTProto http_wait exact wire validation (round 63, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `http_wait#9299359f max_delay:int wait_after:int max_wait:int = HttpWait`. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` constructs `MTP_http_wait(100, 30, 25000)` only when `_connection->usingHttpWait()`; no independent receive-side `mtpc_http_wait` service-message branch was found. The call-site values are therefore recorded as lifecycle evidence, not treated as schema validity constraints.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `03b40ed111c8e64d159ea837848bfae0fa85102c`, implementation commit `96bb0e2cd31cdfc82aaa44e0f8e25acf8c30a204`; canonical blobs api `7b731b6d6bbd2009453c5107a24ebea58cc2ece3`, lib `4d8b7198fbe7aa8cbb71180587970760f6ebd1d9`. The existing protocol owner now exports `HTTP_WAIT_CONSTRUCTOR`, typed `HttpWait { max_delay, wait_after, max_wait }`, and `parse_http_wait`; it consumes exactly constructor + three TL ints and rejects wrong constructors, every-byte truncation, and trailing bytes. The parser is side-effect free and introduces no connection/session/request mutation.

Desktop implementation commit `1495975bbdd2e5d56826a95854def198862e2c5b` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-HTTP-WAIT-WIRE-001`; `responsibilities_total=73`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Telegram Desktop-equivalent `usingHttpWait()` connection scheduling is not represented by a proven canonical owner, so that lifecycle remains explicitly fail-closed; no second HTTP scheduler/runtime/provider or `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history was invented.

Exact-field, wrong-constructor, every-byte truncation, and trailing-byte contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary historical sent/resend/container state, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open/fail-closed.


## Stage-A MTProto help.configSimple exact wire validation (round 64, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares adjacent `ipPort#d433ad73`, `ipPortSecret#37982646`, `accessPointRule#4679b65f`, and `help.configSimple#5a592a6c`. `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no help.configSimple receive/application branch. The actual fixed-U higher-level consumer was reread at `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` / `special_config_request.h@91901489e778089b03187682a9556e929867b94f`: after decrypting a simple config it checks expiry and non-empty rules, applies phone-prefix filtering, converts raw IPv4 values, and emits DC endpoint callbacks. Those lifecycle/policy steps are intentionally not inferred by the wire parser.

Spec-first canonical work is on `bhrumom/fabushi-platform-core checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: spec commit `db4e6db92d6ebf6f03442cb3eb2134b9bf0a144e`; implementation commit `cde1297cacd3861764d74a111629d002e74c9d71`; constructor-first fail-closed correction `46db4422aa7221b1c8048f96f1c7c47cd0c01458`. Canonical blobs are api `0c852a5e3bc41a4ac571d713ae3d4edea19b3619` and lib `04e5020e7b1e7ed46b403c151f84a095a1a17e4c`. The existing protocol owner now exports all four fixed-U constructors, typed `IpPort`, `AccessPointRule`, `HelpConfigSimple`, and exact parsers for each object family. Nested vectors use the existing bounded TL reader; raw signed ints, strings, bytes and ordering are preserved; wrong constructors, malformed nested framing, truncation, and trailing bytes fail closed.

Desktop implementation commit `870ec15e7760718ad7ef1eb5e578dfcef8f0e46b` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-CONFIG-SIMPLE-WIRE-001`; `responsibilities_total=74`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Telegram Desktop-equivalent special-config web acquisition/decryption/HTTP-date/expiry/phone-prefix/DC endpoint application is not represented by a proven canonical owner, so that lifecycle remains explicitly fail-closed; no second bootstrap runtime/provider/DC owner or `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history was invented.

Both IpPort variants, nested rule/config preservation, wrong-constructor, every-byte truncation, malformed nested constructor and trailing-byte contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary historical sent/resend/container state, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open/fail-closed.


### Round 64 final authored-contract and compliance readback

After the initial parser synchronization, canonical authored-only negative contracts were expanded at platform-core commit `022b578b8f7823a9af737a2f88fdf4c684fa4e56` to explicitly cover wrong TL vector constructors and invalid UTF-8 `phone_prefix_rules`; no contract was executed. Final canonical blobs are api `0c852a5e3bc41a4ac571d713ae3d4edea19b3619` and lib `04e5020e7b1e7ed46b403c151f84a095a1a17e4c`. Desktop final contract-sync commit `878329a3f54895da0c44ab33e14cf8856444e128` has byte-identical provider api/lib blobs.

Platform-core spec compliance review commit `d1c85e53a4cb162ef02f5344fcadf3c4a3476f6a` updates `docs/specs/telegram-mtproto-config-simple-wire.md@f7dd300a5a5f49fe6958617817f95629dcf95e8f`: AC1-AC7 are satisfied at source/implementation/bookkeeping level; AC8 remains blocked/not-run for Stage B executable validation. Special-config acquisition/decryption/expiry/phone-prefix/DC endpoint application remains fail-closed.


## Stage-A MTProto ping_delay_disconnect exact wire serialization and TLS-template blocker (round 65, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Reread of `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms that the declarations immediately after `help.configSimple` are `tlsClientHello/tlsBlock*`, followed by `ping#7abe77ec` and `ping_delay_disconnect#f3427b8c ping_id:long disconnect_delay:int`. Fixed-U `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16` shows that `tlsClientHello/tlsBlock*` are not inbound MTProto response objects: `PrepareClientHelloRules` constructs a local typed template AST and `Generator::Part::writeBlock` consumes it to generate TLS camouflage ClientHello bytes, including random blocks, X25519 public-key material, HMAC digest/timestamp injection, scoped lengths and permutations. The current canonical `native/telegram-network` owner exposes direct `TcpTransport` only and has no equivalent TLS camouflage socket/generator lifecycle, so no artificial TL parser was introduced; that lifecycle remains fail-closed/open.

The next independently representable responsibility is exact `ping_delay_disconnect` request serialization. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` serializes it on the main connected session using the current ping id and `kPingDelayDisconnect=60`, while session timers decide when to send it. Platform-core spec commit `c98a668a64a25c516065f0dda938200290712018` separates exact wire layout from those timer/session semantics. Implementation commit `cb816c7540ea93717238b3aec96372cdab1c1706` adds `PING_DELAY_DISCONNECT_CONSTRUCTOR=0xf3427b8c` and `build_ping_delay_disconnect` to the existing telegram-protocol owner. Canonical blobs are api `405b04166bea039051833a38ad0a75c62107f70c` and lib `ef7fe73335551e303fa71b02f5cf066d778ae8a4`; the builder emits exactly constructor + little-endian `i64 ping_id` + little-endian `i32 disconnect_delay`, preserves signed boundary values, and performs no timer/socket/session/request-history mutation.

Desktop implementation commit `d38f6bef11a9a4334f8387c73c7b74e7994c0cdb` synchronizes both protocol blobs byte-for-byte into the existing provider chain. Ledger adds `TDRP-R9-MTPROTO-PING-DELAY-DISCONNECT-WIRE-001`; `responsibilities_total=75`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. The Telegram Desktop main-session/timer policy for deciding when to use `ping_delay_disconnect` remains open/fail-closed rather than being invented by a wire-only builder.

Exact-layout and signed-boundary contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`. TLS camouflage ClientHello generation/socket lifecycle, special-config acquisition/decryption/expiry/phone-prefix/DC endpoint application, `gzip_packed`, complete updates difference/differenceSlice decoding/application, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.


### Round 65 source-level compliance readback

Platform-core compliance commit `6cfc81d79901493c6b34ec4e174d4fbdae81800e` updates `docs/specs/telegram-mtproto-ping-delay-disconnect-wire.md@aaa7c8b0290264b162405f35285a777fb37df91f`. Source inspection records R1-R10 and AC1-AC6 satisfied at source/implementation/bookkeeping level; AC7 remains Stage-B blocked/not-run. Remote blob readback confirms canonical and desktop provider api/lib remain byte-identical at `405b04166bea039051833a38ad0a75c62107f70c` / `ef7fe73335551e303fa71b02f5cf066d778ae8a4`. No runtime claim is made for the TLS camouflage ClientHello generator/socket lifecycle or the main-session ping timer/scheduling policy.


## Stage-A MTProto destroy_session exact request serialization (round 66, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `destroy_session#e7512126 session_id:long = DestroySessionRes`. The existing `DestroySessionRes` parser remains a separate response responsibility. Platform-core spec amendment `6f1ecac8f9284405e49dfc69af20270f914d2c83` and implementation `5f259ca296432384f5388ffef9539dbbf1f4033a` add only `DESTROY_SESSION_CONSTRUCTOR` plus exact `build_destroy_session(session_id:i64)`; canonical blobs are api `2f0c2893e256064f822082dd2fd7deae394b5763`, lib `05c690a72f5f9b5a547f10486a5289229d7bb0ff`. Desktop implementation `28d36535b4d4d342b72c6a0e35752ce56dbcfd26` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-SESSION-REQUEST-WIRE-001`; `responsibilities_total=76`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 12-byte layout, signed-boundary preservation, and request/result-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live outbound destroy-session/teardown lifecycle remains fail-closed because no precise existing owner currently represents it. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain open/fail-closed.


## Stage-A MTProto get_future_salts exact request serialization (round 67, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `get_future_salts#b921bd04 num:int = FutureSalts`; the existing `FutureSalts` parser/current-request correlation remains a separate response responsibility. Platform-core spec amendment `5a52c2a1c53efbe9ce396919598a934dc732a8d7` and implementation `58a9635f0286283218f6d8b7b2b14dd466bd4b98` add only `GET_FUTURE_SALTS_CONSTRUCTOR` plus exact `build_get_future_salts(num:i32)`; canonical blobs are api `e8026c79b8b14261ef30d39bf6de8b0bc92306a6`, lib `809eeb7aa3256e1aa259e694f413b9c0377f08d6`. Desktop implementation `dd232c52e1b39cdccb409c6d6307f1523e4d538f` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-GET-FUTURE-SALTS-REQUEST-WIRE-001`; `responsibilities_total=77`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 8-byte layout, signed-`i32` boundary preservation, and request/FutureSalts-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live `get_future_salts` scheduling and returned future-salt installation/rotation remain fail-closed because no precise existing owner currently represents those timer/session/history semantics. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain open/fail-closed.


### Round 67 source-level compliance readback

Platform-core compliance commit `eaf1eeefdde1401d820d9fecd15da3b4aa9cfde5` updates `docs/specs/telegram-mtproto-future-salts-wire.md@4b690dcafce939c13540d1ff0a55a66bed1138bb`. Source inspection records R11-R17 and AC9-AC12 satisfied at source/implementation/bookkeeping level; AC13 keeps executable validation blocked/not-run with `accepted=false`. Remote source readback confirms canonical and desktop provider api/lib remain byte-identical at `e8026c79b8b14261ef30d39bf6de8b0bc92306a6` / `809eeb7aa3256e1aa259e694f413b9c0377f08d6`. Live `get_future_salts` scheduling and future-salt installation/rotation remain explicit fail-closed scope.


## Stage-A MTProto ping exact request serialization (round 68, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` declares `ping#7abe77ec ping_id:long = Pong`; the existing strict `Pong` parser/current-ping correlation remains a separate receive responsibility. Fixed-U `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` selects ordinary `MTPPing` only from connection/main-session/timer state, so that scheduling/selection lifecycle is not synthesized by this wire-only slice. Platform-core spec amendment `cd40329f3fbc39c06599e3b5d4dda63f09ae6b17` and implementation `d6ef74975d3541ddb4d187bfb5641773dc01a32d` add only `PING_CONSTRUCTOR=0x7abe77ec` plus exact `build_ping(ping_id:i64)`; canonical blobs are api `714c24a5f1727e13693008ea42676831b3e8d3fd`, lib `ef44b1bb7fcac0183f1b83cca8c9d994ecc8ce9c`. Desktop implementation `36b425e547509f334453ac5ebf09033e366fe6ae` synchronizes those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-PING-REQUEST-WIRE-001`; `responsibilities_total=78`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 12-byte layout, signed-`i64` boundary preservation, and request/Pong-parser separation contracts are authored only. No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `validation_status=not-run`, `accepted=false`.

Live ordinary-ping scheduling/selection, message-id assignment, send/ACK settlement, timeout/reconnect lifecycle and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history remain fail-closed. TLS camouflage lifecycle, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.


### Round 68 source-level compliance readback

Platform-core compliance commit `bf654c6501776fb46a8e40bad5f9aacea4612514` updates `docs/specs/telegram-mtproto-single-ping-pong.md@4faf7eb9c9a6c7b9e5988c850c9068c9a0a21d79`. Source inspection records request-side R9-R16 and AC-6 through AC-9 satisfied at source/implementation/bookkeeping level; AC-10 keeps executable validation blocked/not-run with `accepted=false`. Remote source identities are canonical/desktop api `714c24a5f1727e13693008ea42676831b3e8d3fd` and lib `ef44b1bb7fcac0183f1b83cca8c9d994ecc8ce9c`. Live ordinary-ping scheduling/selection, message-id assignment, send/ACK settlement, timeout/reconnect, arbitrary historical sent/resend/container state, TLS camouflage, gzip and complete difference decoding remain explicit fail-closed scope.


### Round 69 — rpc_drop_answer request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `rpc_drop_answer#58e4a740 req_msg_id:long = RpcDropAnswer`; reread `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct `rpc_drop_answer` branch, so no historical scheduling/correlation lifecycle is inferred.

Platform-core spec-first amendment `fde4b1eadc3d03c1dd943bcb5170e3434ff2771a` and implementation `37fff2cb24658b6e0f738bbbd23cecdf075f4bb1` add only `RPC_DROP_ANSWER_CONSTRUCTOR=0x58e4a740` plus exact `build_rpc_drop_answer(req_msg_id:i64)`. Canonical blobs are api `3ec13cd2248e68f55296ed9bbaffa5c10ffce969`, lib `e4705eb4563a087e0bba811948f1244af8f9f5cb`; desktop synchronization `7c17952b90cc89d575345e192733c2e6c23deb03` copies them byte-for-byte.

Ledger adds independent request responsibility `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-REQUEST-WIRE-001` while retaining `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001` as the response-parser responsibility; `responsibilities_total=79`, `unknown=16172`, `unread=16145`, `omitted=0`. Exact 12-byte layout, signed-`i64` boundary preservation, and request/response-parser separation contracts are authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Historical outstanding-RPC selection/correlation/drop mutation and arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` state remain fail-closed; gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 69 source-level compliance readback

Platform-core compliance commit `510b75727303147a06b6276fa94f7dcd3a7cb31b` updates `docs/specs/telegram-mtproto-rpc-drop-answer-wire.md@f8408584b8ac997cad97d213fe90eb18155544f1`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `3ec13cd2248e68f55296ed9bbaffa5c10ffce969` / `e4705eb4563a087e0bba811948f1244af8f9f5cb`; `RPC_DROP_ANSWER_CONSTRUCTOR=0x58e4a740`, public `build_rpc_drop_answer(req_msg_id:i64)`, and the pre-existing `parse_rpc_drop_answer` response parser are all present. Request-side R9-R16 and AC7-AC9 are satisfied at source/implementation/bookkeeping level; AC10 executable validation remains blocked/not-run with `accepted=false`. Historical outstanding-RPC selection/correlation/drop mutation, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 70 — destroy_auth_key request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `destroy_auth_key#d1435160 = DestroyAuthKeyRes`. The existing shared `telegram-protocol` api owner can represent this zero-field request independently; auth-key eligibility, result handling, local key deletion, persistence and teardown/reconnect lifecycle remain fail-closed.

Platform-core spec-first commit `529f5c9147f0144b67eb077733b52d61b469f55d` and implementation `8eafd0cc7a84ab9af6b402665670b38f3ad72406` add only `DESTROY_AUTH_KEY_CONSTRUCTOR=0xd1435160` plus exact `build_destroy_auth_key()`. Canonical blobs are api `fbca9b0ebb85d0edd5c01ceea9896d0977294b53`, lib `ed581d8356375e264fabe85a2e5cccbd9b772f21`; desktop synchronization `fe0093d767ad2a12e049b4d0a474c0145d3468ab` copies them byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-REQUEST-WIRE-001`; `responsibilities_total=80`, `unknown=16172`, `unread=16145`, `omitted=0`. Exact four-byte/no-payload contract is authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Auth-key destruction lifecycle, arbitrary sent/resend/container history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


### Round 70 source-level compliance readback

Platform-core compliance commit `7643f936cd2b40aa628041880565c88539a1aace` updates `docs/specs/telegram-mtproto-destroy-auth-key-request-wire.md@ca43a6b328e68d68dfbfd7c9bae90c614918a046`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `fbca9b0ebb85d0edd5c01ceea9896d0977294b53` / `ed581d8356375e264fabe85a2e5cccbd9b772f21`; `DESTROY_AUTH_KEY_CONSTRUCTOR=0xd1435160` and public `build_destroy_auth_key()` are present. R1-R5 are satisfied at source/ownership/bookkeeping level; R6 executable validation remains blocked/not-run with `accepted=false`. Auth-key destruction eligibility/result handling/local deletion/persistence/teardown, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.

### Round 71 — req_pq request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_pq#60469778 nonce:int128 = ResPQ`. Fixed-U `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a` confirms `AbstractConnection::preparePQFake` constructs `MTPReq_pq(nonce)` before not-secure packet/message-id wrapping; reread `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no `req_pq` branch. The existing shared `telegram-protocol` owner can independently carry the exact request body; probe scheduling/racing, nonce generation, not-secure framing, response correlation and live auth-key lifecycle remain fail-closed.

Platform-core spec-first commit `c0b1b72f501822023e73f0e971a41bb0bfe38baf` and implementation `48b8dcb9fc7d6f7a79c1ccdce59041493a7a3b7a` add only `REQ_PQ_CONSTRUCTOR=0x60469778` plus exact `build_req_pq(nonce:&[u8;16])`. Canonical blobs are api `54ab072495a4f8f1eca322a7f8f46c5dcd4396ee`, lib `8df4760c6bf6dbeeba3e8d8b72232bac6a410ff0`; desktop synchronization `9e3065eddd2d0497dfb4ea2ed7a38d4ee7acb13f` copies them byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-REQ-PQ-REQUEST-WIRE-001`; `responsibilities_total=81`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 20-byte constructor-plus-raw-int128-nonce contracts, including zero/0xff nonce fixtures, are authored only. `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. `req_pq_multi`, live preparePQFake scheduling/transport racing, not-secure packet/message-id construction, send/retry/timeout, response correlation, existing ResPQ/auth-key handshake integration, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.

### Round 71 source-level compliance readback

Platform-core compliance commit `893b83679268c9a2e3bd8d867419adb4b5145d26` updates `docs/specs/telegram-mtproto-req-pq-request-wire.md@b90b86e526f24665832326cc1b3ff7e95d0d32c2`. Remote readback confirms canonical and desktop provider api/lib blobs are byte-identical at `54ab072495a4f8f1eca322a7f8f46c5dcd4396ee` / `8df4760c6bf6dbeeba3e8d8b72232bac6a410ff0`; `REQ_PQ_CONSTRUCTOR=0x60469778`, public `build_req_pq(nonce:&[u8;16])`, raw int128 serialization and both authored contracts are present. R1-R5 are satisfied at source/ownership/bookkeeping level; R6 executable validation remains blocked/not-run with `accepted=false`. Live connection-probe scheduling/transport racing, nonce generation, not-secure packet/message-id construction, send/retry/timeout, response correlation, `req_pq_multi`, live ResPQ/auth-key handshake integration, arbitrary sent/resend/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B evidence and independent release ACCEPT remain open.



### Round 72 — req_pq_multi request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_pq_multi#be7e8ef1 nonce:int128 = ResPQ`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms shipping `DcKeyCreator::pqSend` sets `Stage::WaitingPQ`, generates the attempt nonce, and passes `MTPReq_pq_multi(attempt->data.nonce)` to `sendNotSecureRequest`; `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no adjacent `req_pq_multi` branch.

Platform-core spec-first commit `c5b3021a760892eaf05fcd9b40dc35dff4c2c783` and canonical implementation commits `60e4b9165c0f41804bf68e7270aa52f39e824f14` / `591176a5ba091bb2a00f925d38ed87a1fdae0b11` add public `REQ_PQ_MULTI_CONSTRUCTOR=0xbe7e8ef1` plus pure `build_req_pq_multi(nonce:&[u8;16])` in the existing request-wire owner. Existing `AuthKeyHandshake::begin` delegates only body serialization to that builder while retaining its invalid-state guard, caller message id, `AwaitingResPq` transition, and plaintext-envelope lifecycle. Canonical blobs are api `3e3b250b709ce2f307f896dda6d1c2da55559f7b`, handshake `b1b22af0e83b577d6152beb11a75e44eb76ffdbe`, lib `2d48c2fe1c677b4b4672e7e76d8bd0a29a7ac003`, contracts `bf5aeb85f10baeeb7e645a660fd3bf26259e5989`. Desktop synchronization `59e08389bae4ab2cdcbb3f67808da44dff452ca9` makes its tracked api/lib/contracts byte-identical without introducing a second handshake lifecycle owner.

Ledger adds `TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001`; `responsibilities_total=82`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact 20-byte constructor-plus-raw-int128-nonce contracts, including patterned/zero/0xff nonce fixtures and begin-path body equality, are authored only. `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection-probe scheduling/transport racing, nonce/message-id generation, plaintext/not-secure framing, send/retry/timeout, ResPQ correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.


### Round 72 source-level compliance readback

Platform-core compliance commit `492c822360c7dacedef656226386126a41ba5d91` updates `docs/specs/telegram-mtproto-req-pq-multi-request-wire.md@e9cddb463c32ef9c4483a1a7bd57c130fe260f19`. Remote readback confirms public `REQ_PQ_MULTI_CONSTRUCTOR=0xbe7e8ef1`, pure `build_req_pq_multi(nonce:&[u8;16])`, and the existing `AuthKeyHandshake::begin` body-only delegation are present; canonical api/handshake/lib/contracts blobs are `3e3b250b709ce2f307f896dda6d1c2da55559f7b` / `b1b22af0e83b577d6152beb11a75e44eb76ffdbe` / `2d48c2fe1c677b4b4672e7e76d8bd0a29a7ac003` / `bf5aeb85f10baeeb7e645a660fd3bf26259e5989`. Desktop tracked api/lib/contracts remain byte-identical and no second desktop handshake lifecycle owner was introduced.

Spec R1-R7 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-REQ-PQ-MULTI-REQUEST-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; Stage-B executable evidence is intentionally absent. Connection probing/racing, nonce/message-id generation, plaintext/not-secure framing, network send/retry/timeout, ResPQ correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 73 — set_client_DH_params exact request wire serializer

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `req_DH_params#d712e4be ...` and `set_client_DH_params#f5045f1f nonce:int128 server_nonce:int128 encrypted_data:string = Set_client_DH_params_answer`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms shipping `pqAnswered` sends `MTPReq_DH_params` and `dhClientParamsSend` sends `MTPSet_client_DH_params` after moving to `Stage::WaitingDone`; `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` contains no direct constructor call.

Source review confirms existing canonical `build_req_dh_params` already serializes constructor, nonce, server_nonce, p, q, fingerprint and encrypted_data in fixed-U order, with an authored contract that consumes every field and requires no trailing bytes; that responsibility was left unchanged. `set_client_DH_params` was still inline in `prepare_client_dh`, so platform-core spec-first commit `f3d85c368ace17c864eebf2b4d97a44c9d3d964f` and implementation `a353cdf718a93f8e51f48e58c1be4a9889c3bc32` add pure `build_set_client_dh_params` and delegate only final request-body framing from the existing DH preparation path. Canonical blobs are handshake `2e9acc80d10a87a8633173c88568aef706d4e123`, lib `3a05251d749f0df868e4ebd94fb7e28e31fad8f1`, contracts `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`. Desktop synchronization `85db8d62f8014614234ab2340560367d087ac4e0` mirrors lib/contracts byte-for-byte without introducing a second handshake lifecycle owner.

Ledger adds `TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001`; `responsibilities_total=83`, while `unknown=16172`, `unread=16145`, and `omitted=0` remain unchanged. Exact constructor/raw-nonce/TL-string-padding and prepare-path body-equality contracts are authored only. `implementation_status=implemented`, `validation_status=not-run`, `accepted=false`.

No Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection probing/racing, message-id generation, plaintext/not-secure framing, send/retry/timeout, server response correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.


### Round 73 source-level compliance readback

Platform-core compliance commit `dd584145d268084545e082c399e9f5fe2e52cba9` updates `docs/specs/telegram-mtproto-set-client-dh-params-request-wire.md@7ec57e2cfa47f62f0ae372e776ec8a1dcb39c04d`. Remote readback confirms canonical `build_set_client_dh_params`, its public facade export, and `prepare_client_dh` body-only delegation are present; canonical handshake/lib/contracts blobs are `2e9acc80d10a87a8633173c88568aef706d4e123` / `3a05251d749f0df868e4ebd94fb7e28e31fad8f1` / `3cef7f7532cf98c46f9f9eb06f9b0ff2026dfcb4`. Desktop tracked lib/contracts are byte-identical. Existing `build_req_dh_params` and its complete field-order/trailing contract remain present and unchanged.

Spec R1-R8 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-SET-CLIENT-DH-PARAMS-REQUEST-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; `responsibilities_total=83`, `unknown=16172`, `unread=16145`, `omitted=0`. Stage-B executable evidence is intentionally absent.

Connection probing/racing, message-id generation, plaintext/not-secure framing, network send/retry/timeout, server response correlation/application, broader auth-key creation/persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 74 — client_DH_inner_data exact wire serializer; temp_dc remains fail-closed

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `client_DH_inner_data#6643b654 nonce:int128 server_nonce:int128 retry_id:long g_b:string = Client_DH_Inner_Data` and adjacent `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int`. Fixed-U `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` constructs `MTP_client_DH_inner_data` inside shipping `dhClientParamsSend`; its temporary-key branch chooses `MTP_p_q_inner_data_temp_dc` only when `attempt->expiresIn != 0`. `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no direct construction of either inner object.

Existing canonical `prepare_client_dh` already owned the full DH/auth-key preparation lifecycle but framed `client_DH_inner_data` inline. Platform-core spec-first commit `20049ba38728a50239a900a1c0a684527d427b30` and implementation `b5e5f9cd85ed663b0c2ab35998bc1fbf116df0e9` add pure `build_client_dh_inner_data`, preserving exact constructor/raw nonce/server_nonce/little-endian retry_id/canonical TL bytes framing, and delegate only that inner-object serialization from `prepare_client_dh`. Canonical blobs are handshake `0e5e6c0fa306f75aa1fcb5c26440b1c39f87c9f9`, lib `7273e030bf02c248d9b592b2578cb5dc9716b485`, contracts `7236df6bec3c832363901034d6a7d1c7cd3df3fc`. Desktop synchronization commit `1de1da4bf045c405d26ff44c373ad2dd63bf9ce1` mirrors lib/contracts byte-for-byte and introduces no second handshake owner.

`p_q_inner_data_temp_dc` is deliberately not marked implemented: the canonical `AuthKeyHandshake::prepare_req_dh_params*` currently accepts `dc_id` but has no temporary-key `expires_in` input/state. Adding a dead standalone serializer would not close the shipping temporary-key lifecycle. Ledger adds `TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001`; `responsibilities_total=84`, `unknown=16172`, `unread=16145`, `omitted=0`. New responsibility is `implemented / validation_status=not-run / accepted=false`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed. Connection probing/racing, message-id generation, plaintext/not-secure framing, send/retry/timeout, server response correlation/application, temporary-key p_q_inner_data_temp_dc lifecycle, broader auth-key persistence/session lifecycle, arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.


## Round 74 source-level compliance readback

Platform-core compliance commit `e1b2e5fc59e46b198c07c64845810a73a62b04b4` updates `docs/specs/telegram-mtproto-client-dh-inner-data-wire.md@dc09433a720daa1af5fa7eca4dc641cc542e52cc`. Remote readback confirms canonical `build_client_dh_inner_data`, its public facade export, and `prepare_client_dh` inner-object-only delegation are present; canonical handshake/lib/contracts blobs are `0e5e6c0fa306f75aa1fcb5c26440b1c39f87c9f9` / `7273e030bf02c248d9b592b2578cb5dc9716b485` / `7236df6bec3c832363901034d6a7d1c7cd3df3fc`. Desktop tracked lib/contracts are byte-identical.

Spec R1-R8 are satisfied at source/ownership/bookkeeping level. Ledger responsibility `TDRP-R9-MTPROTO-CLIENT-DH-INNER-DATA-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; `responsibilities_total=84`, `unknown=16172`, `unread=16145`, `omitted=0`. Adjacent `p_q_inner_data_temp_dc#56fddf88` remains explicitly fail-closed because current canonical handshake lifecycle does not represent temporary-key `expires_in`; no dead serializer is counted as production closure. Stage-B executable evidence is intentionally absent.

Temporary-key p_q_inner_data_temp_dc lifecycle, connection probing/racing, message-id generation, plaintext/not-secure framing, network send/retry/timeout, response correlation/application, broader auth-key persistence/session lifecycle, arbitrary sent/resend/container history, gzip, complete difference decoding/application, broader fixed-U source/legal closure and independent release ACCEPT remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 75 — p_q_inner_data_temp_dc exact wire + temporary request preparation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. fixed-U `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` defines `p_q_inner_data_temp_dc#56fddf88 ... dc:int expires_in:int`; `details/mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` records `Attempt::expiresIn` and consumes it only when selecting/serializing the temporary p_q inner object before Req_DH_params. Existing canonical `AuthKeyHandshake` can therefore precisely own the wire/request-preparation slice without a second state machine.

Spec-first commits `93283c35d075a16498b672e548071e16bdafd2c3` and boundary correction `8f09112d7af1875e19eddb7096ef1a0a05347d8d` precede implementation. Canonical commits `af7b37f57be70a88cd10728d79d352c7aade88f5` / `68c20daadfc680e24393c63d0505feb5fee4783e` add `P_Q_INNER_DATA_TEMP_DC_CONSTRUCTOR`, pure `build_p_q_inner_data_temp_dc`, positive lifetime validation, and `prepare_temporary_req_dh_params*` reusing the existing ResPqReceived -> AwaitingServerDh preparation path. Contract commit `bcd516892fa49407a4de757e05235ed585ad4338` is authored only. Desktop commits `aafca841fd745f3aee2deb9b89921f6d34021361` / `bd7c3a1c648107c92a265c76c46bae1da745f7e9` mirror facade/contracts byte-for-byte.

The main telegram-network/runtime establishment remains persistent: current persisted transport state has no temporary-key expiry/binding metadata, so routing a temporary key there would misrepresent lifetime semantics. That broader lifecycle is explicitly fail-closed rather than silently persisted. Ledger adds `TDRP-R9-MTPROTO-P-Q-INNER-DATA-TEMP-DC-WIRE-001`; `responsibilities_total=85`, `unknown=16172`, `unread=16145`, `omitted=0`; new responsibility is `implemented / validation_status=not-run / accepted=false`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. Broader temporary auth-key expiry/binding/persistence, connection scheduling/racing, plaintext/send lifecycle, multi-request history, gzip, complete difference decoding, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open/fail-closed.

### Round 76 — DestroyAuthKeyRes exact typed parser + adjacent p_q disposition

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Source readback confirms `DcKeyCreator` requires `protocolDcId` and its shipping pq-answer path selects only `p_q_inner_data_dc#a9f55f95` (persistent) or `p_q_inner_data_temp_dc#56fddf88` (temporary). The adjacent no-dc `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` therefore have no independent fixed-U shipping responsibility in this path and no duplicate builders were added. `bind_auth_key_inner#75a3f765` remains fail-closed because temporary/permanent binding, expiry and session lifecycle require broader state ownership.

The next independent shipping auth-key wire responsibility is `DestroyAuthKeyRes`: fixed-U `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087` sends `destroy_auth_key#d1435160`, parses the response union, and distinguishes `destroy_auth_key_ok#f660e1d4`, `destroy_auth_key_none#0a9f2259`, and `destroy_auth_key_fail#ea109b13`. Spec-first platform commit `791308b5c885b15b4f69b395ae0df0a75f47f634` defines the wire-only boundary. Canonical implementation commit `ebb1e5f859e224c45f9d34beb1aa32b4c18ca4b2` adds the three constructors, `DestroyAuthKeyResult`, strict `parse_destroy_auth_key_result`, facade exports and authored-not-run contracts. Canonical api/lib/contracts blobs are `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`; desktop sync commit `4b152da131b17558fcad7c1aee45fd0cec3269e6` mirrors those blobs byte-for-byte.

Ledger adds `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001`; `responsibilities_total=86`, `unknown=16172`, `unread=16145`, `omitted=0`. The new responsibility is `implemented / validation_status=not-run / accepted=false`. Destruction eligibility, local key removal, persistence mutation, transport/session teardown/reconnect, temporary/permanent binding and renewal remain fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

## Round 76 final source-level readback

Platform-core compliance commit `42c7ac4d193f7f1872a9bfd87ce70b2af973c2df` updates `docs/specs/telegram-mtproto-destroy-auth-key-result-wire.md` to blob `da653d871e78a8a97770449034d0ac4db2f71cfe` and records R1-R8 source-level compliance. Canonical api/lib/contracts remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `a53c90f09ec18f8e99d8cfb36eb8c8842f27635e`; desktop tracked provider files are byte-identical.

The ledger responsibility `TDRP-R9-MTPROTO-DESTROY-AUTH-KEY-RESULT-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=86`, `unknown=16172`, `unread=16145`, `omitted=0`. Adjacent `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` remain dispositioned as non-shipping in fixed-U `DcKeyCreator`; `bind_auth_key_inner#75a3f765` and broader auth-key destruction/binding/persistence/teardown lifecycle remain fail-closed.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.



### Round 77 — Server_DH_Params exact constructor-first union parsing

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping `DcKeyCreator::dhParamsAnswered` consumes fixed-U `server_DH_params_ok#d0e8075c` / `server_DH_params_fail#79cb045d` through the existing canonical `AuthKeyHandshake::receive_server_dh`. The existing parser already owned the wire responsibility, but read nonce/server_nonce before constructor discrimination, so a complete four-byte unknown constructor was misclassified as TL truncation.

Spec-first platform commit `9a6b687e043bae9f7f40e776ce1d5ec98220a168` defines the strict wire-only boundary. Canonical implementation commit `274bdb92bc7e4d5755bb87fd4996398ee6cf768b` hardens `parse_server_dh_params` to reject unknown constructors before field reads while preserving exact nonce binding, OK TL-bytes / FAIL int128 payload decoding and no-trailing validation. Canonical handshake/contracts blobs are `ed9eda7732f4753b442de98ebf510b8a7558b823` / `a19c4a14d113e6c82fe3cc801f1764c555f26f76`. Desktop sync commit `000d38bd2db19052766ccf8062eea4a6fb93c7eb` mirrors the contracts byte-for-byte; the tracked desktop facade remains `abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate desktop `handshake.rs` is introduced.

Ledger adds `TDRP-R9-MTPROTO-SERVER-DH-PARAMS-WIRE-001`; coverage becomes `responsibilities_total=87`, `unknown=16172`, `unread=16145`, `omitted=0`; the new responsibility is `implemented / validation_status=not-run / accepted=false`. Broader auth-key destruction/binding/persistence/teardown, connection scheduling/racing/send lifecycle, multi-request history, gzip, complete difference decoding/application, broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 77 final source-level compliance

Platform-core `8654e737a04900b6cac8c92660fa8a8b84b449bc` / spec blob `b9b0f486aa47071ad46ba4ff5c238cee4b2bbfce` records source-level R1-R9 and AC-1-AC-6 compliance for the constructor-first fixed-U `Server_DH_Params` parser. Canonical handshake/contracts are `ed9eda7732f4753b442de98ebf510b8a7558b823` / `a19c4a14d113e6c82fe3cc801f1764c555f26f76`; desktop contracts remain byte-identical and facade `abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. Ledger stays `implemented / not-run / accepted=false`, coverage `87 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 78 — dh_gen_ok/retry/fail exact result/hash validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::dhClientParamsAnswered` consumes fixed-U `dh_gen_ok#3bcbf734`,
`dh_gen_retry#46dc1fb9` and `dh_gen_fail#a69dae02`. Existing
`AuthKeyHandshake::receive_dh_gen / parse_dh_gen_result` already owned nonce binding,
tag-specific `new_nonce_hash{1,2,3}` authentication, strict trailing validation, OK completion,
handshake-local RETRY and FAIL rejection. The remaining exact-wire gap was discriminator ordering:
nonce/server_nonce/hash were read before constructor matching, so a complete four-byte unknown constructor
was misclassified as TL truncation.

Spec-first platform commit `0c16a7c83a7e511484f5099399acb4c308aecb6c`
(`docs/specs/telegram-mtproto-dh-gen-result-wire.md@a43f4db8123abfcd2334fe49079b413e40c58ea6`)
defines the wire-only boundary. Canonical implementation commit
`5a5c6095d3e049de5d187b89c4942d7f3377ae00` moves three-way constructor discrimination before
all common fields in `handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6`.
Canonical authored-contract commit `39fce63bce58718727541ac9f18dbc258766d159` expands the existing harness to all
OK/RETRY/FAIL variants, every truncation cut, four-byte unknown constructor, nonce/server_nonce mismatch,
wrong variant-tag hash and trailing bytes; contract blob is
`6710d6ef3fd915689792c68d69b3922b446c9bfa`. Desktop sync
`965db14afa7104deae0fc63a76d2cc0df35d81fd` mirrors that blob byte-for-byte; the tracked desktop
facade remains `abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate `handshake.rs` is introduced.

The existing RETRY branch is precisely representable by the current `AwaitingDhGen` owner: it uses the
previous auth-key auxiliary hash as `retry_id`, replaces only prepared client-DH material, and remains in
`AwaitingDhGen`. It does not require or claim temporary/permanent key binding, expiry, persistence,
teardown, connection scheduling, multi-request history, gzip, or complete difference decoding/application.

Ledger adds `TDRP-R9-MTPROTO-DH-GEN-RESULT-WIRE-001`; coverage becomes
`responsibilities_total=88`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`.

No executable validation was performed. No GitHub Actions/status/check query or trigger, build, typecheck,
lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or
runtime acceptance was performed.


### Round 78 final source-level compliance

Platform-core `3a08f54f8a49c8a8af7a23b252938d1ea4607954` / spec blob
`d846673483e2b165d7fad7463be0d7720ee6a49b` records source-level R1-R9 and AC-1-AC-6 compliance
for the constructor-first fixed-U `dh_gen_ok/retry/fail` parser/hash responsibility.
Canonical handshake/contracts are `87616f4b96a1775455da42b0bbed77711fb3c9f6` /
`6710d6ef3fd915689792c68d69b3922b446c9bfa`; desktop contracts remain byte-identical and facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. The existing RETRY path stays inside
`AwaitingDhGen` and does not claim binding/persistence or connection scheduling. Ledger remains
`implemented / not-run / accepted=false`, coverage `88 / 16172 unknown / 16145 unread / 0 omitted`.
No executable validation was performed.


### Round 79 — resPQ exact constructor/nonce/server_nonce/pq/fingerprint-vector validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::pqAnswered` consumes `resPQ#05162463 nonce:int128 server_nonce:int128 pq:string server_public_key_fingerprints:Vector<long>`.
Remote source review confirms the existing canonical `AuthKeyHandshake::receive_res_pq / parse_res_pq` already owns and exactly implements
this fixed-U wire responsibility: constructor-first discrimination, exact nonce/server_nonce/pq/vector field order, bounded TL Vector
constructor/count validation, strict EOF, active request nonce binding, empty-pq rejection and empty-fingerprint rejection before state mutation.
The server nonce is fresh server-provided handshake material at this phase and is captured only after the response passes validation.

Because production behavior was already exact, no production `handshake.rs` delta was manufactured. Spec-first platform commit
`03dc0f804a8c80d8d60500e6667cc9b091ccc3cf`
(`docs/specs/telegram-mtproto-res-pq-wire.md@ab044dcb2b34d7def042e4dd2eb798f3b658c52a`) records that disposition.
Canonical authored-contract commit `5e1ba90b32ac7c58b7be0d9630f1630a0a764061` extends the existing protocol harness with exact
positive resPQ, every truncation cut, four-byte unknown constructor, wrong Vector constructor, negative/over-limit/short declared vector,
trailing byte, active nonce mismatch, empty pq, empty fingerprints and invalid receiver-state cases; canonical contract blob is
`44d2a5d141e7d7854529379aaaad660853c8cbe3`. Desktop provider sync commit
`c72c7f85c926d05b21842c41a3b11c1313537793` mirrors that contract blob byte-for-byte; the tracked desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate handshake core is introduced.

Ledger adds `TDRP-R9-MTPROTO-RES-PQ-WIRE-001`; coverage becomes
`responsibilities_total=89`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Temporary/permanent key binding/expiry/persistence/teardown,
connection scheduling/racing/send lifecycle, arbitrary multi-request history, gzip, complete difference decoding/application,
broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 79 final source-level compliance

Platform-core `54f1a26231827ac11a1ea0544dc4d1f7c3762c6c` / spec blob
`404bbe591fb514aba732e5b4e8f6481cc2904618` records source-level R1-R9 and AC-1-AC-5 compliance for the
fixed-U `resPQ#05162463` wire responsibility. Canonical production handshake remains
`87616f4b96a1775455da42b0bbed77711fb3c9f6` with no behavior delta; canonical/desktop authored contract blob is
`44d2a5d141e7d7854529379aaaad660853c8cbe3`, and desktop facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged. Ledger remains
`implemented / validation_status=not-run / accepted=false`, coverage
`89 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 80 — server_DH_inner_data exact authenticated wire validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`DcKeyCreator::dhParamsAnswered` consumes fixed-U
`server_DH_inner_data#b5890dba nonce:int128 server_nonce:int128 g:int dh_prime:string g_a:string server_time:int`.
Remote source review confirms the existing canonical
`AuthKeyHandshake::receive_server_dh -> decrypt_server_dh_inner_data` already owns and exactly implements this
wire/decryption responsibility: non-empty AES-IGE block framing, temporary key/IV derivation from active
new_nonce/server_nonce, constructor-first exact field decode, active nonce/server_nonce binding, SHA-1 authentication
over exactly the consumed TL object, residual AES alignment padding bounded to 0..15 bytes, signed server_time
preservation, and existing safe-prime/generator/public-value checks before client-DH preparation.

Because production behavior is already exact, no `handshake.rs` behavior delta was manufactured. Spec-first
platform commit `3bdde5a158373e3392cac987c8846d16a6fb3162`
(`docs/specs/telegram-mtproto-server-dh-inner-data-wire.md@5174248bd16e0d125abca66b389d0e49bc2a3f05`)
records the no-production-delta disposition. Canonical authored-contract commit
`d724e29265e53a7ee7efd821c8bb37f6e4994cb2` expands the existing harness with exact signed server_time,
empty/non-block ciphertext, unknown constructor, every serialized-inner truncation cut, nonce/server_nonce mismatch,
SHA-1 mismatch, >15 residual padding, invalid generator, dh_prime length/trust, empty/oversized/unsafe g_a and
rejected-inner state-continuity cases; canonical contract blob is
`256f421f54c41256646388833b634cf1191afd91`. Desktop sync
`4323b97d3b6041731d5b547118fb746d2eba37a6` mirrors that blob byte-for-byte; desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf` and no duplicate handshake core is introduced.

Ledger adds `TDRP-R9-MTPROTO-SERVER-DH-INNER-DATA-WIRE-001`; coverage becomes
`responsibilities_total=90`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Auth-key binding/expiry/persistence/teardown,
connection scheduling/racing/send lifecycle, arbitrary multi-request history, gzip, complete difference
decoding/application, broader source/legal closure and release acceptance remain fail-closed/open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 80 final source-level compliance

Platform-core `26cc521c1e57944a9f27fa16f469fff870f34517` / spec blob
`1290cb76c59c01343ac7903a83113b28a90b4147` records source-level R1-R13 and AC-1-AC-6 compliance for the
fixed-U `server_DH_inner_data#b5890dba` responsibility. Canonical production handshake remains
`87616f4b96a1775455da42b0bbed77711fb3c9f6` with no behavior delta; canonical/desktop authored contract blob is
`256f421f54c41256646388833b634cf1191afd91`, and desktop facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf`. Ledger remains
`implemented / validation_status=not-run / accepted=false`, coverage
`90 / 16172 unknown / 16145 unread / 0 omitted`. No executable validation was performed.


### Round 81 — msgs_state_req exact request serializer/vector boundary

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The shipping
`SessionPrivate::tryToSend` path serializes fixed-U
`msgs_state_req#da69fb52 msg_ids:Vector<long>` from non-empty `_stateRequestData`, preserving id order in
`MTP_vector<MTPlong>`. Existing Fabushi `build_msgs_state_req` already owns the independent request-wire
responsibility and is used by the previously implemented single-current-RPC state-reconciliation path.

Remote source review confirms canonical production
`native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`,
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` and
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` already serialize constructor `0xda69fb52`,
TL Vector constructor `0x1cb5c415`, signed i32 count and each signed long as little-endian i64 in caller
order; vector-count conversion fails closed before a partial Vector header can be emitted. Empty Vector remains
schema-valid at the pure builder boundary, while the shipping caller independently sends only non-empty state requests.

Because production behavior is already exact, no production delta was manufactured. Spec-first platform commit
`579b575974b2031e50ffea4910acce8a67289a64`
(`docs/specs/telegram-mtproto-msgs-state-req-request-wire.md@dfa2b21a82d9b4d9f92da32b4cedcc176bab0c3f`)
records the no-production-delta disposition. Canonical authored-contract commit
`97e04d8ee1556c28cd6227f809816f04bcf1f674` adds empty, one-id shipping, multiple signed-long
order/bit-preservation, exact EOF and `i32::MAX+1` vector-overflow/no-partial-header cases; contract blob is
`60a1dfb3ea2051a498d6cfb9d401fe653fde7d75`. Desktop provider sync
`23f023f57cc9c6bc50e519fe89f01cc78a3511d9` mirrors that contract blob byte-for-byte while production
`api.rs` remains `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` and facade
`abab39e42f100972484191f0e52e9edd9b88c6cf` remains unchanged.

Ledger/evidence commit `2b5267d726385e6bebbd9d68555f1ebe607b4a84` adds
`TDRP-R9-MTPROTO-MSGS-STATE-REQ-REQUEST-WIRE-001`; coverage becomes
`responsibilities_total=91`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`. Arbitrary multi-request
`haveSent/_resendingIds/_stateAndResendRequests` history, simultaneous state queries, connection scheduling,
auth-key binding/persistence, gzip, complete difference decoding/application, broader source/legal closure and
release acceptance remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 81 final source-level compliance

Platform-core `f2593977cbdfe1a09ce811da86029ec824021af3` / final spec blob
`f9b8839b10fa449474a5cfe080f84ca70a08f7a1` records source-level R1-R9 and AC-1-AC-6 compliance for the
fixed-U `msgs_state_req#da69fb52 msg_ids:Vector<long>` request-wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `60a1dfb3ea2051a498d6cfb9d401fe653fde7d75`. Ledger compliance commit
`cba0acd6b39a75d6c605ec0549c372c23c77ffa2` records the responsibility as
`implemented / validation_status=not-run / accepted=false`, coverage
`91 / 16172 unknown / 16145 unread / 0 omitted`. Arbitrary multi-request history and the other deferred
lifecycle/gzip/difference responsibilities remain open/fail-closed. No executable validation was performed.


### Round 82 — msg_resend_req exact request serializer/vector boundary

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`msg_resend_req#7d861a08 msg_ids:Vector<long>` is shipping-reachable in
`SessionPrivate::tryToSend`: a non-empty `_resendRequestData` is moved into
`MTP_vector<MTPlong>` and serialized as `MTP_msg_resend_req`. The adjacent detailed-info receive paths enqueue
an answer id into that vector when the answer was not received.

Remote source review confirms the existing canonical production owner already closes the independent request-wire
boundary with no behavior delta:
`native/telegram-protocol/src/api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines
`MSG_RESEND_REQ_CONSTRUCTOR = 0x7d861a08` and `build_msg_resend_req`;
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` writes TL Vector constructor
`0x1cb5c415`, converts the element count to signed i32 before writing the Vector header, and preserves each signed
long as little-endian i64; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same builder.

Spec-first platform commit `953cd194143ae4cea4b8e842cb6d596b33451b19`
(`docs/specs/telegram-mtproto-msg-resend-req-request-wire.md@56facfa5ba3db5999a42cdcb742086f2c8ae690a`)
records the no-production-delta disposition. Canonical authored-only contract commit
`161d78f68715540e733c43895aba2f1be4619755` adds empty, one shipping-shaped id, multiple signed-long
order/bit-preservation, strict EOF and `i32::MAX+1` vector-overflow/no-partial-header cases; canonical contract blob
is `89692a28620b2d8465703a1335458d5fbd15aee6`. Desktop provider sync
`37299e18e3018e988d541607d5ba7842713d9fd1` mirrors that blob byte-for-byte while desktop production
`api.rs` remains `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` and facade remains
`abab39e42f100972484191f0e52e9edd9b88c6cf`.

Ledger/evidence commit `5cd36cdd14e7321b0d96526f334b1f97e4243510` adds
`TDRP-R9-MTPROTO-MSG-RESEND-REQ-REQUEST-WIRE-001`; coverage is
`responsibilities_total=92`, `unknown=16172`, `unread=16145`, `omitted=0`; the responsibility is
`implemented / validation_status=not-run / accepted=false`.

This slice deliberately does not claim Telegram Desktop arbitrary
`haveSent/_resendingIds/toSend/_sentContainers` histories, callback ownership, resend scheduling, container retry,
connection scheduling/racing/send retry, auth-key binding/persistence, gzip, or complete difference decoding/application.
Those remain open/fail-closed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 82 final source-level compliance

Platform-core `be950b18eca7e3e6c1e9ffb4833a19fcbd224880` / final spec blob
`e85d67b0b3b8c7c3d7bee7c08d72a9978903c325` records source-level R1-R9 and AC-1-AC-6 compliance for the
fixed-U `msg_resend_req#7d861a08 msg_ids:Vector<long>` request-wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `89692a28620b2d8465703a1335458d5fbd15aee6`. Desktop compliance ledger/evidence commit
`19e0d2060b64f78e3a44cbb8cb96c4f46f9ca312` records the responsibility as
`implemented / validation_status=not-run / accepted=false`, coverage
`92 / 16172 unknown / 16145 unread / 0 omitted`.

Arbitrary multi-request `haveSent/_resendingIds/toSend/_sentContainers` history, resend scheduling/cancellation,
connection scheduling/racing/send retry, auth-key binding/persistence, gzip, complete difference decoding/application,
broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.
No executable validation was performed.


### Round 83 — rpc_error exact typed parser/trailing validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`rpc_error#2144ca19 error_code:int error_message:string` is shipping-reachable as the body of
`rpc_result` in `SessionPrivate::handle`. Wider request-history lookup, temporary-key handling and
`gzip_packed` are deliberately outside this isolated wire slice.

Remote source review confirms the existing canonical production owner already closes the independent parser boundary:
`api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines `RPC_ERROR_CONSTRUCTOR`,
`RpcErrorResponse` and `try_parse_rpc_error`; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
owns TL string padding/UTF-8/truncation; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser.
The parser rejects trailing bytes, so production behavior requires no delta.

Spec blob is `c219f35d393bb94e29c30f579ec501bb3d42c9d5`; canonical authored contract commit
`7ad951b93b57fc3c4cf33e43eaa157066d452b31` and desktop sync
`885faac55fc8b7d99e4989b438e1a428ac57c1f8` share contract blob
`d171a3ab9a43955a693c87162f80c2667fc397ce`. Authored cases cover exact signed code/message,
non-RpcError discrimination, truncation, invalid UTF-8, trailing bytes and exact EOF; they were not run.

The responsibility is recorded as `implemented / validation_status=not-run / accepted=false`.
Arbitrary multi-request history, connection scheduling/racing/send retry, auth-key binding/persistence, gzip,
complete difference/differenceSlice decoding/application, broader source/legal closure and Stage-B executable evidence
remain open/fail-closed. No executable validation was performed.


### Round 83 final source-level compliance

Platform-core `e448cb7307ded83d9c6eab34fff0ef31f48e0e79` / final spec blob
`89d572a9ec403835f4e1ec9d538dc3fbbe86a32d` records source-level R1-R10 and AC-1-AC-6 compliance for the
fixed-U `rpc_error#2144ca19 error_code:int error_message:string` response-wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `d171a3ab9a43955a693c87162f80c2667fc397ce`. The desktop ledger records
`TDRP-R9-MTPROTO-RPC-ERROR-WIRE-001` as `implemented / validation_status=not-run / accepted=false`, coverage
`93 / 16172 unknown / 16145 unread / 0 omitted`.

Arbitrary request history, connection scheduling/racing/send retry, temporary/permanent auth-key
binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice decoding/application, broader fixed-U
source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open/fail-closed.
No executable validation was performed.


### Round 84 — msgs_state_info exact typed parser / binary TL string

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`msgs_state_info#04deb57d req_msg_id:long info:string` is shipping-reachable in
`SessionPrivate::handle`, where `MTPMsgsStateInfo::read(from,end)` performs typed wire decoding before
`_stateAndResendRequests` lookup and `handleMsgsStates` application.

Remote source review confirms the existing canonical `telegram-protocol` owner already closes the independent
wire boundary with no behavior delta:
`api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines constructor `0x04deb57d`,
`MsgsStateInfo` and `parse_msgs_state_info`; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
owns TL bytes/string short/long framing, length limits, padding and truncation; and
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser. The fixed-U `info` value is
binary state data, so preserving it through `read_bytes()` rather than UTF-8 `read_string()` is intentional.

Spec-first platform commit `41fc75c372bdb818c21c21af6fcc60958bb7974b` /
spec blob `d90260a94d2bc04f2d487614cc4806b819e126ee` records the no-production-delta disposition.
Canonical authored-only contract commit `ae5b7a6f6b6669fe9069d7371b1b4a2acd76e182` and desktop sync
`ae630cf55cb611f9e400b8c5a4bbbbad20654387` share contract blob
`efaa8ac407af5bca430855fbb69c862f9ce793b0`. Authored cases cover signed id preservation,
binary non-UTF8 short-form state, empty and 254-byte long-form state, wrong constructor, truncation,
missing padding, trailing bytes and exact EOF; they were not run.

Ledger adds `TDRP-R9-MTPROTO-MSGS-STATE-INFO-WIRE-001`; coverage becomes
`94 / 16172 unknown / 16145 unread / 0 omitted`, and the responsibility is
`implemented / validation_status=not-run / accepted=false`.

Arbitrary request history/application, connection scheduling/racing/send retry, auth-key binding/persistence,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure and Stage-B executable
evidence remain open/fail-closed. No executable validation was performed.


### Round 84 final source-level compliance

Platform-core `e99655ad3c06858e60fb7a1bf6a5a12cb1a0b2ff` / final spec blob
`4780f4764ae68cbb921177214a05e215b98608fa` records source-level R1-R10 and AC-1-AC-6 compliance for the
fixed-U `msgs_state_info#04deb57d req_msg_id:long info:string` wire responsibility. Canonical production
`api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `efaa8ac407af5bca430855fbb69c862f9ce793b0`. The desktop ledger records
`TDRP-R9-MTPROTO-MSGS-STATE-INFO-WIRE-001` as
`implemented / validation_status=not-run / accepted=false`, coverage
`94 / 16172 unknown / 16145 unread / 0 omitted`.

Binary `info` preservation is intentional; arbitrary request history/application, connection scheduling/racing/send
retry, temporary/permanent auth-key binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice
decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT
remain open/fail-closed. No executable validation was performed.


### Round 85 — msgs_all_info exact typed parser / vector boundary / binary TL string

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`msgs_all_info#8cc0d131 msg_ids:Vector<long> info:string` is shipping-reachable in
`SessionPrivate::handle`: `MTPMsgsAllInfo::read(from,end)` first performs typed wire decoding, then
`handleMsgsStates(ids, states)` validates `states.size() == ids.size()` before consulting
`haveSent/_resendingIds` or performing resend/ack application.

Remote source review confirms the existing canonical `telegram-protocol` owner already closes the independent
wire/state-shape boundary with no production delta:
`api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines constructor `0x8cc0d131`,
`MsgsAllInfo` and `parse_msgs_all_info`; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`
owns canonical vector constructor/count validation, signed `i64` reads, TL bytes/string short/long framing,
64MiB bytes limit, 1,000,000-element vector bound, padding consumption and truncation; and
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser/type/constructor.
The parser preserves `info` as binary bytes, requires one state byte per id, and rejects trailing bytes.

Spec-first platform commit `9f8ed1934fbd35e6a4e6ad9243c686431a84bdee` /
spec blob `8d4b04886141bf16bc36fe045d8997a472fae545` records the no-production-delta disposition.
Canonical authored-only contract commit `5e961490ebad8971a44ba91253797408c959ca93` and desktop sync
`822ed21752b28b435f55c2c711215fc01b15551c` share contract blob
`d1a86cb35d6e27ed94c911750190c627fc7764cf`. Authored cases cover signed-id extremes,
non-UTF8 binary short-form state, empty and 254-byte long-form state, wrong top-level/vector constructors,
negative and over-limit vector counts, truncated vector/id/TL body, missing padding, state/id mismatch,
trailing bytes and exact EOF; they were not run.

Ledger adds `TDRP-R9-MTPROTO-MSGS-ALL-INFO-WIRE-001`; coverage becomes
`95 / 16172 unknown / 16145 unread / 0 omitted`, and the responsibility is
`implemented / validation_status=not-run / accepted=false`.

Arbitrary `wasSent/haveSent/_resendingIds/toSend/_sentContainers` history, resend/ack application,
connection scheduling/racing/send retry, temporary/permanent auth-key binding/expiry/persistence/teardown,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure and Stage-B executable
evidence remain open/fail-closed. No executable validation was performed.


### Round 85 final source-level compliance

Platform-core `a6281c8b43aff544acdafca454091fa2856cdd49` / final spec blob
`0e0baa8fe35f9ef9e446d8a04ac7f95ca1fadba4` records source-level R1-R10 and AC-1-AC-6 compliance for the
fixed-U `msgs_all_info#8cc0d131 msg_ids:Vector<long> info:string` wire/state-shape responsibility.
Canonical production `api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `d1a86cb35d6e27ed94c911750190c627fc7764cf`. The desktop ledger records
`TDRP-R9-MTPROTO-MSGS-ALL-INFO-WIRE-001` as
`implemented / validation_status=not-run / accepted=false`, coverage
`95 / 16172 unknown / 16145 unread / 0 omitted`.

Vector constructor/count bounds, signed ids, binary `info`, state/id cardinality and exact EOF are isolated from
arbitrary request history/application. `wasSent/haveSent/_resendingIds/toSend/_sentContainers`, resend/ack application,
connection scheduling/racing/send retry, temporary/permanent auth-key binding/expiry/persistence/teardown, gzip,
complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain open/fail-closed. No executable validation was performed.


### Round 86 — msg_detailed_info exact typed parser / fixed field widths

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`msg_detailed_info#276d3ec6 msg_id:long answer_msg_id:long bytes:int status:int` is shipping-reachable in
`SessionPrivate::handle`: `MTPMsgDetailedInfo::read(from,end)` performs typed wire decoding before any
bad-time recovery, `requestsAcked`, `_receivedMessageIds` lookup, or ack/resend enqueueing.

Remote source review confirms the existing canonical `telegram-protocol` owner already closes the independent
wire boundary with no production delta. `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines constructor
`0x276d3ec6`, `MsgDetailedInfo { i64, i64, i32, i32 }`, and `parse_msg_detailed_info`;
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` reads signed little-endian i64/i32 values with fail-closed
truncation; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser/type/constructor.
The parser requires exact EOF. The exact wire object is 28 bytes: 4 + 8 + 8 + 4 + 4.

Spec-first platform commit `38c54a94dbec4ac82118b8f2b6fd4c611719d064` /
spec blob `a9285d6df90d78133b488d29b8eee79d97b5d8d8` records the no-production-delta disposition.
Canonical authored-only contract commit `688ccb1acd5afb4e3d8c76c1c898148f721e3d48` and desktop sync
`1d82ac67b4a86f1fd8b0584cbf8b76fb9252de2a` share contract blob
`d1d4ed680d297b778516ec52d3b3746fe9687135`. Authored cases cover signed extremes, exact 28-byte layout,
wrong constructor, truncation at constructor/each 8/8/4/4 field boundary, trailing bytes and exact EOF; they were not run.

Ledger adds `TDRP-R9-MTPROTO-MSG-DETAILED-INFO-WIRE-001`; coverage becomes
`96 / 16172 unknown / 16145 unread / 0 omitted`, and the responsibility is
`implemented / validation_status=not-run / accepted=false`.

`requestsAcked`, bad-time recovery, `_receivedMessageIds` history, ack/resend application,
connection scheduling/racing/send retry, temporary/permanent auth-key binding/expiry/persistence/teardown,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure and Stage-B executable
evidence remain open/fail-closed. No executable validation was performed.


### Round 86 final source-level compliance

Platform-core `d6fd206df4b71689d74dc5a5e0b9b3a82b9a6e3c` / final spec blob
`5afa9a228f9d5983e80747d12b355ffa85b0b44e` records source-level R1-R7 and AC-1-AC-6 compliance for the
fixed-U `msg_detailed_info#276d3ec6 msg_id:long answer_msg_id:long bytes:int status:int` wire responsibility.
Canonical production `api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `d1d4ed680d297b778516ec52d3b3746fe9687135`. The desktop ledger records
`TDRP-R9-MTPROTO-MSG-DETAILED-INFO-WIRE-001` as
`implemented / validation_status=not-run / accepted=false`, coverage
`96 / 16172 unknown / 16145 unread / 0 omitted`.

Exact signed 8/8/4/4 field widths and strict EOF are isolated from `requestsAcked`, bad-time recovery,
`_receivedMessageIds` history, ack/resend application and arbitrary request-history semantics. Connection scheduling,
temporary/permanent auth-key binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice
decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT
remain open/fail-closed. No executable validation was performed.


### Round 87 — msg_new_detailed_info exact typed parser / fixed field widths

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U
`msg_new_detailed_info#809db6df answer_msg_id:long bytes:int status:int` is shipping-reachable in
`SessionPrivate::handle`. That branch first ignores the service message when `info.badTime` is set; otherwise
`MTPMsgDetailedInfo::read(from,end)` performs typed wire decoding before `_receivedMessageIds` lookup or
ack/resend enqueueing.

Remote source review confirms the existing canonical `telegram-protocol` owner already closes the independent
wire boundary with no production delta. `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines constructor
`0x809db6df`, `MsgNewDetailedInfo { i64, i32, i32 }`, and `parse_msg_new_detailed_info`;
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` reads signed little-endian i64/i32 values with fail-closed
truncation; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same parser/type/constructor.
The parser requires exact EOF. The exact wire object is 20 bytes: 4 + 8 + 4 + 4.

Spec-first platform commit `ba585c9d3c0d66fd07f254dac0e0ac97f0cd8c92` /
spec blob `1295bed353dfd9b5f26518dadfc68ddb6e1bf56b` records the no-production-delta disposition.
Canonical authored-only contract commit `9154bcf48a0d27efe2fa459a9670100c99d29e25` and desktop sync
`efeb7cef34a4582deb0a3631856ded581889f6bd` share contract blob
`feedf5e25c739e0deca7a26f356a25a9d1a9f46b`. Authored cases cover signed extremes, exact 20-byte layout,
wrong constructor, truncation at constructor/each 8/4/4 field boundary, trailing bytes and exact EOF; they were not run.

Ledger adds `TDRP-R9-MTPROTO-MSG-NEW-DETAILED-INFO-WIRE-001`; coverage becomes
`97 / 16172 unknown / 16145 unread / 0 omitted`, and the responsibility is
`implemented / validation_status=not-run / accepted=false`.

Bad-time handling, `_receivedMessageIds` history, ack/resend application, arbitrary request history,
connection scheduling/racing/send retry, temporary/permanent auth-key binding/expiry/persistence/teardown,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure and Stage-B executable
evidence remain open/fail-closed. No executable validation was performed.


### Round 87 final source-level compliance

Platform-core `329c3e89bb5dcf5dbd6d27153fd160d891adbe2f` / final spec blob
`6681bade8a147afff7e18e9cdfe49fcd179f7436` records source-level R1-R7 and AC-1-AC-6 compliance for the
fixed-U `msg_new_detailed_info#809db6df answer_msg_id:long bytes:int status:int` wire responsibility.
Canonical production `api.rs/tl.rs/lib.rs` remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` with no behavior delta. Canonical/desktop authored contract
blob remains `feedf5e25c739e0deca7a26f356a25a9d1a9f46b`. The desktop ledger records
`TDRP-R9-MTPROTO-MSG-NEW-DETAILED-INFO-WIRE-001` as
`implemented / validation_status=not-run / accepted=false`, coverage
`97 / 16172 unknown / 16145 unread / 0 omitted`.

Exact signed 8/4/4 field widths and strict EOF remain isolated from bad-time handling, `_receivedMessageIds`
history and ack/resend application. Arbitrary request history, connection scheduling/racing/send retry,
temporary/permanent auth-key binding/expiry/persistence/teardown, gzip, complete difference/differenceSlice
decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT
remain open/fail-closed. No executable validation was performed.


## Stage-A MTProto bad_msg_notification adjacent wire no-production-delta (round 88, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`,
with `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and
`session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`. The live upstream schema is
`bad_msg_notification#a7eff811 bad_msg_id:long bad_msg_seqno:int error_code:int`; the shipping
`mtpc_bad_msg_notification` branch completes `MTPBadMsgNotification::read(from, end)` before it enters resend,
clock correction, request-history or session-reset decisions.

The exact wire responsibility had already been closed by the existing shared owner in round 60. Current remote
readback confirms canonical `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`,
`tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264`,
`lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf`, and
`telegram-network/src/lib.rs@1fc1b1f69e3e37377400d078f916599e0c7b20bc`. The protocol parser owns constructor
`0xa7eff811`, signed i64/i32/i32 decoding, truncation failure, and strict EOF. The network recovery consumers remain
pre-existing application semantics and were not modified in this round.

Spec-first no-production-delta record is platform-core commit
`9984ca1b13185693961ff0735156e55ab035e62a` / spec blob
`7155c589c1e4090a534060e42fc9ebce1f8c404d`. Canonical authored-only external contracts are commit
`bc5f9410a520e5e35d40a1c5a1e6a8f2031d6030` / blob
`184e5bfa7832e9bc7c7d4f594011b032382e11d3`; desktop mirror commit
`58237abd9bcb64671d1bbfe88a3cdc8bdca5a2a4` has the byte-identical contract blob. Desktop production
api/lib/network remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` /
`1fc1b1f69e3e37377400d078f916599e0c7b20bc`.

The existing ledger row `TDRP-R9-MTPROTO-BAD-MSG-NOTIFICATION-WIRE-001` is reused rather than duplicated and
remains `implemented / validation_status=not-run / accepted=false`. Coverage remains
`responsibilities_total=97`, `unknown=16172`, `unread=16145`, `omitted=0`. The round88 contracts lock the
exact 20-byte layout, signed extremes, constructor and each field truncation boundary, wrong constructor, trailing
bytes and exact EOF; they were authored only and not run.

Bad-time/time-correction recovery, request correlation/history, retry/session-reset application,
acknowledgement/resend application, connection scheduling/racing/send retry, auth-key lifecycle, gzip, complete
difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and
independent release ACCEPT remain outside this isolated wire-only no-production-delta claim.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 88 final source-level compliance

Platform-core final compliance commit `21ab4e3c4b98a93761c8df525656c09f9b4c404d` / final spec blob
`8b12f64f4a102296195365833b145d5329056b69` records the isolated
`bad_msg_notification#a7eff811 bad_msg_id:long bad_msg_seqno:int error_code:int` wire responsibility as
source-level closed with no production delta. Canonical production api/tl/lib/network blobs remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` /
`89256ab35184b6dc06a095ab34abb9f44e95a264` /
`abab39e42f100972484191f0e52e9edd9b88c6cf` /
`1fc1b1f69e3e37377400d078f916599e0c7b20bc`.

Canonical and desktop authored-only external contract blob is
`184e5bfa7832e9bc7c7d4f594011b032382e11d3`; contracts cover exact 20-byte layout, signed i64/i32/i32
extremes, wrong constructor, constructor/field truncation boundaries, trailing-byte rejection and exact EOF and
remain not run. The existing ledger row stays
`implemented / validation_status=not-run / accepted=false`; coverage remains
`97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`.

Bad-time/time-correction recovery, current-request correlation, retry/session reset, arbitrary sent-request/container
history, acknowledgement/resend application, connection scheduling/racing/send retry, auth-key lifecycle, gzip,
complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain outside this wire-only closure.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 89 bad_server_salt adjacent revalidation (task round 58, 2026-11-01)

Fixed-U `bad_server_salt#edab447b bad_msg_id:long bad_msg_seqno:int error_code:int new_server_salt:long` was reread against `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`. The existing ledger row `TDRP-R9-MTPROTO-BAD-SERVER-SALT-WIRE-001` is reused; no duplicate responsibility is created and coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`.

Existing canonical `telegram-protocol` is already exact: `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` exposes constructor/type/parser; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` enforces signed little-endian 8/4/4/8 fixed-width reads with fail-closed truncation; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same owner. `telegram-network@1fc1b1f69e3e37377400d078f916599e0c7b20bc` consumes that parser in both shipping receive branches before current-request correlation and salt mutation. Exact object size is 28 bytes and strict EOF rejects trailing bytes.

Disposition: **no production delta**. Platform-core spec commit/blob is `bf4f241f9ceddc9b032b23d1f154a8883b636e92@757e919f5328bd004fcf2e9a22b49a05e4a80f21`; canonical authored-only contract commit/blob is `67e66a56a2ab2bf3c85b27a5d2bfac0d5682321f@b3e535525ffa97920816c9b8f316afddbf46f6ac`; desktop contract mirror commit is `8caa269f72d98f79624ef73bd3ea901d0d0a6f86` with the same blob. Contracts cover exact 28-byte layout/EOF, signed i64/i32/i32/i64 extremes, wrong constructor, constructor/field truncation boundaries and trailing bytes; they remain not run.

Salt adoption/persistence, bad-time/time correction, current/historical request correlation, binding/reconnect, resend/application behavior, arbitrary sent/container history, gzip, complete difference decoding, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 89 final source-level compliance

Platform-core final compliance commit `3db5c55a946a466d157e37b95d2fce0eb5706595` / spec blob `4247be931d826a80ef4cc10c8c4b84de96804991` records `bad_server_salt#edab447b` as source-level closed with **no production delta**. Canonical production api/tl/lib/network blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `89256ab35184b6dc06a095ab34abb9f44e95a264` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. Canonical and desktop authored-only external contract blob is `b3e535525ffa97920816c9b8f316afddbf46f6ac` and remains not run.

The existing ledger row stays `implemented / validation_status=not-run / accepted=false`; coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`. Salt adoption/persistence, bad-time/time correction, request-history lookup, binding/reconnect, resend/application, arbitrary sent/container history, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

### Round 90 future_salt element-boundary adjacent revalidation (task round 59, 2026-11-01)

Fixed-U `future_salt#0949d9dc valid_since:int valid_until:int salt:long` was reread as the nested element of `future_salts#ae500895` against `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and the adjacent authenticated service-message boundary in `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`. The existing ledger row `TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001` is reused; no duplicate responsibility is created and coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`.

Existing canonical `telegram-protocol` is already exact: `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` defines `FutureSalt { i32, i32, i64 }` and parses every nested element after `FUTURE_SALT_CONSTRUCTOR`; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` enforces fixed-size signed little-endian 4/4/8 reads with fail-closed truncation; `parse_future_salts` budgets 20 bytes per vector element (constructor 4 + fields 4/4/8), checks `count * 20 <= remaining` before allocation, validates each nested constructor, and requires strict EOF. `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same owner, while `telegram-network@1fc1b1f69e3e37377400d078f916599e0c7b20bc` consumes `parse_future_salts` before current-request matching/settlement.

Disposition: **no production delta**. Platform-core spec commit/blob is `9797db85df1aa286511296887463ef44451c133b@16dd5b50297449b467108767cf70bc4c8243310a`; canonical authored-only external contract commit/blob is `8b9d113599b12d0d0a7fdd250b75aab87f881821@0f99a2c99b377920792316de433c2229bc781e60`; desktop contract mirror commit is `b5891b2907d6ca72fb1a5385f27f29ad4b0e215e` with the byte-identical contract blob. New contracts cover two exact 20-byte nested elements, signed i32/i32/i64 extremes, exact element offsets, wrong nested constructor, constructor/each-field truncation boundaries, impossible vector count and trailing-byte rejection; they were authored only and not run.

Future-salt adoption/installation/rotation, refresh scheduling, persistence, retry, arbitrary request history, broader application lifecycle, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain outside this isolated nested wire claim. Fixed-U `session_private.cpp` has no dedicated `future_salts` application branch, so no salt-lifecycle behavior is inferred from this parser revalidation.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.

### Round 90 final source-level compliance

Platform-core final compliance commit `b6c7f4ad9a7e40bc4ffe24b6ee7a5e33ae134b83` / spec blob `c9617bb2c8e40f3ff4d070121ccb95c4d9ddff16` records nested `future_salt#0949d9dc` wire validation as source-level closed with **no production delta**. Canonical production api/tl/lib/network blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `89256ab35184b6dc06a095ab34abb9f44e95a264` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. Canonical and desktop authored-only external contract blob is `0f99a2c99b377920792316de433c2229bc781e60` and remains not run.

The existing `TDRP-R9-MTPROTO-FUTURE-SALTS-WIRE-001` row stays `implemented / validation_status=not-run / accepted=false`; coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`. Future-salt adoption/installation/rotation, refresh scheduling, persistence, retry, arbitrary request history/application, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.



### Round 91 RpcDropAnswer result adjacent revalidation (task round 60, 2026-11-01)

Fixed-U `rpc_answer_unknown#5e2ad36e`, `rpc_answer_dropped_running#cd78e586`, and `rpc_answer_dropped#a43ad8b7 msg_id:long seq_no:int bytes:int` were reread against `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`. The existing ledger row `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001` is reused; no duplicate responsibility is created and coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`.

Existing canonical `telegram-protocol` is already exact: `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` dispatches the two exact zero-field constructors and the exact Dropped constructor; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` enforces signed little-endian fixed-width i64/i32/i32 reads with fail-closed truncation; strict EOF rejects trailing bytes; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same owner. Fixed-U `session_private.cpp` has no direct `rpc_answer_*` branch, so request-history drop/cancel semantics are not part of this parser-only claim.

Disposition: **no production delta**. Platform-core spec commit/blob is `930a6839e8f64fafab836e43080c485507d4a4a6@8e8c3376dd39b93ae50d8490aedceed8f947ba12`; canonical authored-only external contract commit/blob is `cf51ff32eb217bc2084b8f2cbf9c886eb2a588ec@3c69527d941fad9eddb8c264e695666ad3e404f9`; desktop contract mirror commit is `314b0b90fa96efe492fd3785e141a0d1390646d3` with the same byte-identical contract blob. Contracts cover exact 4-byte zero-field objects, exact 20-byte Dropped layout/offsets, signed i64/i32/i32 extremes, unknown constructor, every truncation cut, and trailing bytes; they remain not run.

Historical request drop/cancel scheduling and arbitrary request mutation, connection scheduling, auth-key binding/persistence, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 91 final source-level compliance

Platform-core final compliance commit `df3feac4618636cdbc863b19e05106e19ba10446` / spec blob `2a6a5adae6b2f8bb1ebf06faa29dba88f35c958e` records `RpcDropAnswer` response wire validation as source-level closed with **no production delta**. Canonical production api/tl/lib blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `89256ab35184b6dc06a095ab34abb9f44e95a264` / `abab39e42f100972484191f0e52e9edd9b88c6cf`. Canonical and desktop authored-only external contract blob is `3c69527d941fad9eddb8c264e695666ad3e404f9` and remains not run.

The existing `TDRP-R9-MTPROTO-RPC-DROP-ANSWER-WIRE-001` row stays `implemented / validation_status=not-run / accepted=false`; coverage remains `97 responsibilities / 16172 unknown / 16145 unread / 0 omitted`. Historical request drop/cancel scheduling and arbitrary request mutation, connection scheduling, auth-key binding/persistence, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 92 DestroySessionRes result adjacent revalidation (task round 61, 2026-11-01)

Fixed-U `destroy_session_ok#e22045fc session_id:long` and `destroy_session_none#62d350c9 session_id:long` were reread against `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`. The existing ledger row `TDRP-R9-MTPROTO-DESTROY-SESSION-WIRE-001` is reused; no duplicate responsibility is created.

Existing canonical `telegram-protocol` is already exact: `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` parses both variants as exactly constructor 4 + signed `session_id:i64` 8 = 12 bytes; `tl.rs@89256ab35184b6dc06a095ab34abb9f44e95a264` provides bounded little-endian fixed-width reads; strict EOF rejects trailing bytes; `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` exports the same owner. Fixed-U `session_private.cpp` has no direct `destroy_session_*` branch, so no live teardown, request-history mutation, scheduler behavior, auth-key lifecycle, resend/container state, or persistence is inferred.

Disposition: **no production delta**. Platform-core spec commit/blob is `dbcec4027f96605c86c8b1197a5477cc21eebfb4@87f5657bcb413ffbd94ae11d4bb3a42ae5b30daa`; canonical authored-only external contract commit/blob is `d3d8635c69186631396886b086133b7f455ce33b@e414104c28784a1735f7d8b5c1a5cfeb23fe2192`. Contracts cover both exact 12-byte variants, constructor/session-id offsets, signed i64 minimum/maximum, every truncation cut, unknown constructor, and trailing bytes; they were authored only and not run.

The row remains `implemented / validation_status=not-run / accepted=false`. Live teardown/outbound destroy-session lifecycle, arbitrary multi-request history, connection scheduling, auth-key binding/persistence, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain outside this isolated wire claim or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 92 final source-level compliance

Platform-core final compliance commit `c32cc58ffe44214a102e7c455fe73c00f90e1aed` / spec blob `1502e49a46298c1bf4e393e37c018009cf33fcca` records `DestroySessionRes` result-wire validation as source-level closed with **no production delta**. Canonical production api/tl/lib blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `89256ab35184b6dc06a095ab34abb9f44e95a264` / `abab39e42f100972484191f0e52e9edd9b88c6cf`. Canonical and desktop authored-only external contract blob is `e414104c28784a1735f7d8b5c1a5cfeb23fe2192` and remains not run.

The existing `TDRP-R9-MTPROTO-DESTROY-SESSION-WIRE-001` row stays `implemented / validation_status=not-run / accepted=false`. Live teardown/outbound destroy-session lifecycle, arbitrary multi-request history, connection scheduling, auth-key binding/persistence, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


## Stage-A MTProto NewSession exact wire adjacent revalidation (round 93, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Live adjacency was reread from `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`: `new_session_created#9ec20908 first_msg_id:long unique_id:long server_salt:long = NewSession` is the next service-message type after `DestroySessionRes`.

The existing owner already closes this exact wire responsibility, so this round is **no production delta**. Canonical protocol `updates.rs@23a09bb6e4e78ef526823cf01d0a207565fb2a3e` parses constructor plus three signed 64-bit fields and strict EOF. Current canonical/desktop network blob `1fc1b1f69e3e37377400d078f916599e0c7b20bc` invokes that parser before salt mutation or the existing one-current-RPC replay decision. Desktop protocol updates/lib/network blobs remain `23a09bb6e4e78ef526823cf01d0a207565fb2a3e` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`.

Platform-core spec-first revalidation is commit `8d86554fcc4afc8d02b7ed38bfd5ca93221fede7`, spec blob `9e5ee768418dddfbde2f4534d9ff9c26d70ef5ba`. Authored-only external contracts are commit `0db067351aef9365ada9e168f041599983dc1693`, blob `32a60800c34d53223fed2f1e4aa521e300b510d7`, and are mirrored byte-identically into the desktop provider in this round. They pin exact 28-byte length, field offsets 0..4 / 4..12 / 12..20 / 20..28, signed i64 extremes, every truncation cut, wrong-constructor rejection and strict EOF. They were not run.

The existing ledger row `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001` is reused; no duplicate responsibility is created. It remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=97`, `unknown=16172`, `unread=16145`, `omitted=0`. Arbitrary sent/resend/container history, connection scheduling, auth-key binding/persistence, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open or outside this isolated wire claim.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 93 final source-level compliance

Platform-core final compliance commit `3e24ea732acd2bcb2fe5bf2e5257f275643384de` / spec blob `298cbba2b593aa17502be752dd406718d745de93` records the fixed-U `NewSession` result-wire revalidation as source-level closed with **no production delta**. Canonical production protocol updates/lib/network blobs remain `23a09bb6e4e78ef526823cf01d0a207565fb2a3e` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. Canonical and desktop authored-only external contract blob is `32a60800c34d53223fed2f1e4aa521e300b510d7` and remains not run.

The existing `TDRP-R9-MTPROTO-NEW-SESSION-WIRE-001` row stays `implemented / validation_status=not-run / accepted=false`. Arbitrary multi-request history, connection scheduling, auth-key binding/persistence, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


## Stage-A MTProto Pong exact wire adjacent revalidation (round 94, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Live adjacency was reread from `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`: after `new_session_created#9ec20908`, the shipping service branch handles `pong#347773c5 msg_id:long ping_id:long = Pong`.

The existing owner already closes this exact wire responsibility, so this round is **no production delta**. Canonical protocol `api.rs@ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` exposes `PONG_CONSTRUCTOR`, typed `Pong`, and `parse_pong`, consuming constructor 4 + signed `msg_id` 8 + signed `ping_id` 8 = exactly 20 bytes and rejecting wrong constructor, truncation, and trailing bytes. Current canonical/desktop network blob `1fc1b1f69e3e37377400d078f916599e0c7b20bc` calls the typed parser before exact current-ping two-ID correlation.

Platform-core spec-first revalidation is commit `8680085af3ef5ba80fa63933a7ce22c36dfc031d`, spec blob `231a89e30087370eb0cf29dc422f47e0322266d6`. Authored-only external contracts are commit `a559782b390f43d5caa28d8eaba1f99a9292059b`, blob `7672b1908bdf8810592e86f8427fd85c49089d7f`, and are mirrored byte-identically into the desktop provider in this round. They pin exact 20-byte length, offsets `0..4 / 4..12 / 12..20`, signed i64 extremes, every truncation cut, wrong-constructor rejection and strict EOF. They were not run.

The existing ledger row `TDRP-R9-MTPROTO-SINGLE-PING-PONG-001` is reused; no duplicate responsibility is created. It remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=97`, `unknown=16172`, `unread=16145`, `omitted=0`. Fixed-U `wasSent/requestsFixTimeSalt/requestsAcked` and arbitrary `haveSent/_resendingIds/toSend/_sentContainers` history, connection scheduling, auth-key binding/persistence, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open or outside this isolated wire claim.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 94 final source-level compliance

Platform-core final compliance commit `1542bb5e40d2b3c9d4b21ac59f99cb420d7fdbb7` / spec blob `979c5340ce456542e0a4f7dc40a13e85135102f3` records the fixed-U `Pong` wire revalidation as source-level closed with **no production delta**. Canonical production protocol api/lib/network blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf` / `1fc1b1f69e3e37377400d078f916599e0c7b20bc`. Canonical and desktop authored-only external contract blob is `7672b1908bdf8810592e86f8427fd85c49089d7f` and remains not run.

The existing `TDRP-R9-MTPROTO-SINGLE-PING-PONG-001` row stays `implemented / validation_status=not-run / accepted=false`. Historical `wasSent/requestsFixTimeSalt/requestsAcked`, arbitrary multi-request sent/resend/container state, connection scheduling, auth-key binding/persistence, gzip, complete difference decoding/application, broader source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


## Stage-A MTProto http_wait exact wire adjacent revalidation (round 95, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Reread of `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` confirms `http_wait#9299359f max_delay:int wait_after:int max_wait:int = HttpWait` is shipping-reachable only where `SessionPrivate::tryToSend()` serializes `MTP_http_wait(100, 30, 25000)` under `_connection->usingHttpWait()`; there is no independent receive-side branch. The observed values remain call-site lifecycle evidence, not protocol validity constraints.

Existing canonical `telegram-protocol` already closes the independent wire object exactly, so disposition is **no production delta**. Canonical production blobs remain api `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43`, lib `abab39e42f100972484191f0e52e9edd9b88c6cf`, tl `89256ab35184b6dc06a095ab34abb9f44e95a264`. `parse_http_wait` consumes exactly constructor 4 + three signed TL ints 4/4/4 = 16 bytes and rejects wrong constructor, every truncation, and trailing bytes.

Platform-core revalidation spec commit/blob is `63743d4c188402e812e2f9ba9e7a46ea9ba3ff6c@8084780823adc82a02c5f7227bcabcd0706e0cfa`. An intermediate tree-construction mistake created a temporary single-file tree; it was immediately repaired without force by fast-forward commit `50f3ef5f748e0008c70f990e8dbd0e3896a66455`, restoring the prior complete tree plus the new spec blob. Current branch readback contains the complete repository tree.

Authored-only external contracts are platform-core commit `e3c19abe76b58742e80ef97713049b121e5831c1` / blob `71449041d7a7ae60be5b13379b7b8e7f417dfb8f`, mirrored byte-identically into desktop at commit `e556ba8ee47dfad52aeaab37b622b90b92cf87b2`. They pin exact 16-byte length, offsets `0..4 / 4..8 / 8..12 / 12..16`, independent signed i32 boundaries, every cut `0..15`, wrong constructor, and strict EOF. They were authored only and not run.

Platform-core final source-level compliance commit is `a9cca9a441ada6738bf3418f389c58293b3ec54e` / spec blob `20dee6cba28ad8dd044314f0bdf4d5f4d1e1c54d`. The existing ledger row `TDRP-R9-MTPROTO-HTTP-WAIT-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=97, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

`usingHttpWait()` transport scheduling, timeout/retry/backoff, arbitrary request history, auth-key binding/persistence, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


## Stage-A MTProto help.configSimple adjacent revalidation (evidence round 96; task round 65, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fresh readback confirms `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` still declares `ipPort#d433ad73`, `ipPortSecret#37982646`, `accessPointRule#4679b65f`, and `help.configSimple#5a592a6c` immediately after `http_wait#9299359f`; `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` has no simple-config receive/application branch. The actual fixed-U higher lifecycle remains `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` / `special_config_request.h@91901489e778089b03187682a9556e929867b94f`, which owns web acquisition/decryption, HTTP-derived time, expiry/non-empty policy, phone-prefix filtering, IPv4 conversion, and DC endpoint callbacks.

Existing-owner-first source review confirms **no production delta**. Canonical production api/lib blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf`, and desktop provider api/lib read back byte-identically. The existing parser still preserves raw signed ints, strings, binary secrets and nested vector order, uses bounded TL framing, rejects malformed/wrong/truncated/trailing objects, and performs no expiry/phone/DC routing mutation.

Spec-first revalidation is platform-core commit `99bb755e64a1fb96889acf0b4b673f855f5e48b6` / blob `a80fb4dd9dd7a6855fb28162ba03716daa000b09`. An external public-API regression contract was authored only at canonical commit `2d78cb1aa4ad7e7edb3a31bb596b9eb79e610d32` / blob `bc5c49ee5be958a49f16d7c14617e51e47c3961b`, then mirrored byte-identically into desktop at commit `231337450087a652b678747b98359914c20c8058`. It independently pins empty wire-level `rules` acceptance, signed nested value/secret preservation, every truncation boundary, wrong top-level/nested/vector constructors, and strict EOF. The contract was not run.

Platform-core final source-level compliance is `070ff5136f550a3c79f3839d91667857f71c9f14` / spec blob `ac08d9b91e06af12d8d794d9b146d2aaba829d60`. The sole ledger row `TDRP-R9-MTPROTO-CONFIG-SIMPLE-WIRE-001` remains `implemented / validation_status=not-run / accepted=false`; coverage remains `responsibilities_total=97, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Special-config acquisition/decryption/expiry/phone-prefix/DC endpoint application, arbitrary request/container history, connection scheduling, auth-key binding/persistence, `gzip_packed`, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain outside this wire-only closure or open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


## Stage-A MTProto TLS ClientHello template adjacent disposition (evidence round 97; task round 66, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fresh readback of `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms the complete `tlsClientHello/TlsBlock` family immediately after `help.configSimple`. Fixed-U `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16` confirms these generated TL types are a local template AST: `PrepareClientHelloRules()` builds nested scopes/permutations and `Generator::Part::writeBlock` interprets them into TLS camouflage ClientHello bytes. They are not independently shipped as MTProto request/response objects.

Spec-first platform-core disposition is `d2efec8b167386ad70da5607e2fd5833fc6387b0` / spec blob `bc0363cec846718b2fb0d54bf89943fe292714e4`. Existing-owner-first review finds no truthful independent telegram-protocol production closure: canonical api/lib remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf`, desktop mirrors are byte-identical, and canonical telegram-network still lacks the TLS camouflage ClientHello generator/socket lifecycle. Therefore this round is **no production delta** and no dead parser/serializer is introduced.

The sole ledger adds `TDRP-R9-MTPROTO-TLS-CLIENT-HELLO-TEMPLATE-001` as `mapped / coverage_disposition=blocked / validation_status=not-run / accepted=false`, not implemented. Coverage becomes `responsibilities_total=98`; `unknown=16172`, `unread=16145`, `omitted=0`, and `unmapped_responsibilities=0` remain unchanged. Real closure requires a source-neutral extension of the existing canonical telegram-network transport/TLS lifecycle; randomness/grease/X25519/HMAC/permutation/padding and connection scheduling remain outside telegram-protocol.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 97 final source-level compliance

Platform-core final compliance is `06ac74bc56fcdb362e4e004980d6c60917311bf6` / spec blob `d40d0a5f6d61f4bf16cbc58e303660350885756a`, referencing desktop bookkeeping checkpoint `918837f0456ea3745bd24d8bf11fd72e379b5069`. The unique TLS-template responsibility remains `mapped / coverage_disposition=blocked / validation_status=not-run / accepted=false`; production disposition remains **no production delta**. Canonical/desktop telegram-protocol api/lib blobs remain `ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf`.

Real implementation remains blocked on a shipping canonical telegram-network TLS camouflage ClientHello generator/socket lifecycle. No dead parser/serializer was added, and no executable validation was performed.


## Stage-A MTProto msgs_ack request-wire adjacent revalidation (evidence round 98; task round 67, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fresh readback of
`mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms
`msgs_ack#62d6b459 msg_ids:Vector<long> = MsgsAck`. Fixed-U
`session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` confirms the shipping send path serializes
`MTP_msgs_ack(MTP_vector<MTPlong>(base::take(_ackRequestData)))` only when `_ackRequestData` is non-empty.

Existing-owner-first review finds **no production delta**. Canonical and desktop production api/lib remain
`ebfb3670b5cd097b9ce288782c7954f5cfe9fc43` / `abab39e42f100972484191f0e52e9edd9b88c6cf`;
canonical TL remains `89256ab35184b6dc06a095ab34abb9f44e95a264`. Existing `build_msgs_ack` already writes the exact
constructor, canonical TL Vector framing/count, and caller-ordered signed i64 values.

Spec-first platform-core commit/blob is `5759adce64e495f85dbc068fb0d74d027dee8004@6fd5961fa70b86417d7bb4e71b937c97edacbd08`.
Authored-only exact boundary contracts are platform-core commit `95508fda3420d4829eb1fb76a20bc6f589f02635` /
blob `7314ad06c48f9ca87dc08c0bd3b7d08a47579797`, mirrored byte-identically into desktop at
`4677ae986ea7f9e3156c57883ecfbb6fca7e2f49`. They cover empty, one-element, multiple signed values,
exact order/EOF, and vector-count overflow/no-partial-header. They were not run.

The unique ledger adds `TDRP-R9-MTPROTO-MSGS-ACK-REQUEST-WIRE-001` as
`implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes
`responsibilities_total=99, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Arbitrary ACK scheduling/history/application, request settlement/callback history, connection/TLS scheduling,
auth-key binding/persistence, `gzip_packed`, complete difference/differenceSlice decoding/application, broader
fixed-U source/legal closure, Stage-B executable evidence, and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator,
unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


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


## Stage-A MTProto p_q_inner_data adjacent builder disposition (evidence round 99; task round 68, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fresh readback of `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and shipping `mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms `DcKeyCreator::pqAnswered` always carries `protocolDcId`: persistent attempts use `p_q_inner_data_dc#a9f55f95`, temporary attempts use `p_q_inner_data_temp_dc#56fddf88`. The adjacent no-dc `p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` therefore have no independent shipping call site in this fixed-U auth-key path.

Existing-owner-first review finds **no production delta**. Canonical `handshake.rs@87616f4b96a1775455da42b0bbed77711fb3c9f6` already exposes exact `build_p_q_inner_data_dc`, and `AuthKeyHandshake::prepare_req_dh_params*` selects it before existing RSA_PAD/`req_DH_params` framing. Existing `lib.rs@abab39e42f100972484191f0e52e9edd9b88c6cf` already exports it. The separately closed temporary `*_temp_dc` path remains unchanged.

Spec-first platform-core commit/blob is `1b78659027387846526d325056204020ec287220@e143d380d8a40cbef115ad7e211ebac67464de73`. A focused contract was authored only at canonical commit `88cee9084231b22c3d575726c39634884a45f270` / blob `9c5ce9630c3865f19869b372d1f8f88d2d665487`, then mirrored byte-identically into desktop at `8c1866bf054281a4518d605ecbdd811300b1e8c4`. It pins exact constructor/field order/signed dc_id/EOF and factor-mismatch rejection. It was not run.

The unique ledger adds `TDRP-R9-MTPROTO-P-Q-INNER-DATA-ADJACENT-WIRE-001` as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=100, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories, `gzip_packed`, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 99 final source-level compliance

Platform-core final compliance is `05b06d5f2f7d39280c0502cffcd091e6d67b0ed8` / spec blob
`d28224576e5d9571c4e2b74e0798e42450af1519`, referencing desktop bookkeeping checkpoint
`798965239cb0a72176de3106e4878809b4548bca`. The unique `TDRP-R9-MTPROTO-P-Q-INNER-DATA-ADJACENT-WIRE-001`
responsibility is `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Production disposition remains **no production delta**. Canonical production handshake/lib blobs remain
`87616f4b96a1775455da42b0bbed77711fb3c9f6` / `abab39e42f100972484191f0e52e9edd9b88c6cf`.
The canonical/desktop authored contract blob is `9c5ce9630c3865f19869b372d1f8f88d2d665487`.
Fixed-U `DcKeyCreator::pqAnswered` always supplies `protocolDcId`, so the no-dc
`p_q_inner_data#83c95aec` and `p_q_inner_data_temp#3c6a84d4` remain no-independent-shipping-reachability
dispositions and no dead builders were introduced.

Coverage is `responsibilities_total=100, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.
Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain open. No executable validation was performed.


## Stage-A MTProto req_DH_params request wire (evidence round 100; task round 69, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fresh readback of `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` and shipping `mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03` confirms `DcKeyCreator::pqAnswered` factors the active resPQ, selects the server RSA fingerprint, encrypts the corresponding p_q inner data and sends `req_DH_params#d712e4be` with nonce/server_nonce/p/q/fingerprint/encrypted_data.

Existing-owner-first review selected canonical `AuthKeyHandshake::prepare_req_dh_params*` / `build_req_dh_params`. A small production hardening was required: the exported pure builder previously checked the offered fingerprint and non-empty encrypted payload but did not itself reject p/q factors inconsistent with the active resPQ. Platform-core spec-first commit is `005bf696ba89b32a8965e510069f5867ed921b73` / spec blob `759586ea8e7b07cd9818da7620577512031ae553`; production/authored-contract commit is `39f9ba32d0ae6bcb03d7f957f708d690ec5fb74e`, with `handshake.rs@b6572c91a450a240c56c15ca80585afb97c19839` and contract blob `071b72be6772117fb174daf86d5c9a19ce372b73`.

The production delta is limited to fail-closed `PqFactorsDoNotMatch` validation before serializing `req_DH_params`. Existing field order, fingerprint membership check, empty-encrypted-data check, request preparation and `ResPqReceived -> AwaitingServerDh` lifecycle remain unchanged. The focused contract also records unoffered-fingerprint and empty-encrypted-data failures. Contracts were authored only and not executed.

Desktop has no duplicate tracked `handshake.rs` at this checkpoint, so no second owner was created. The existing tracked provider contract mirror was synchronized byte-identically at desktop commit `0e0188f5d1ebf96d2ee21444e1f7b65994c88ee2` / blob `071b72be6772117fb174daf86d5c9a19ce372b73`; existing desktop lib remains `abab39e42f100972484191f0e52e9edd9b88c6cf`.

The unique ledger adds `TDRP-R9-MTPROTO-REQ-DH-PARAMS-REQUEST-WIRE-001` as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=101, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories, `gzip_packed`, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 100 final source-level compliance

Platform-core final compliance is `8832c2ce26819033ea7d06c113ab97345ceda290` / spec blob
`ac617d4c87a6fe199c1afd5f541f5f12e1ddafc9`, referencing desktop bookkeeping checkpoint
`cb082b48228fbb2ee43021f41592b605d072bfb9`. The unique
`TDRP-R9-MTPROTO-REQ-DH-PARAMS-REQUEST-WIRE-001` responsibility is
`implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `handshake.rs` is `b6572c91a450a240c56c15ca80585afb97c19839`; unchanged canonical/desktop lib is
`abab39e42f100972484191f0e52e9edd9b88c6cf`. Canonical and desktop authored contract blob is
`071b72be6772117fb174daf86d5c9a19ce372b73`. The builder now fails closed on inconsistent active resPQ factors while preserving the existing exact req_DH_params layout, fingerprint membership and non-empty encrypted-data guards.

Coverage is `responsibilities_total=101, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.
Auth-key binding/persistence/expiry/renewal, connection scheduling/racing/send retry, arbitrary request histories,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain open. No executable validation was performed.


## Stage-A MTProto plaintext auth-key handshake envelope wire (evidence round 101; task round 70, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fixed-U `prepareNotSecurePacket` writes auth_key_id=0, message_id, message_data_length and the request plus 0..63 random 32-bit padding words, with padding included in the declared length; `DcKeyCreator::sendNotSecureRequest` is the real handshake caller.

Existing-owner-first selected canonical `PlaintextEnvelope` plus `TcpTransport::exchange_plaintext`. Spec-first platform-core checkpoint is `acfc281740f3a31a17d7a50ec26327f5ddab0f8b` / `d999d6349b64729783695704464b89ae706698b6`; production/authored-contract checkpoint is `7b25a2cf1d690c1f441f3f45fcc92b7d0a86f057` with handshake `301554c60fe318a621726cacdd37f0d5d1de2a3e`, network `54f509474391bb5118ba4678ba5c566ab3e74ec8`, contract `a7c10e88172ccd664bad0370acea8fac7a3e2f3d`. Desktop mirrors network/contracts byte-identically at `d2aaaf1a4979ad4f825532ec14e6648fd5bbc8eb`; no duplicate handshake.rs was created.

The unique ledger adds `TDRP-R9-MTPROTO-PLAINTEXT-HANDSHAKE-ENVELOPE-WIRE-001` as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=102, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation, or runtime acceptance was performed.


### Round 101 final source-level compliance

Platform-core final compliance is `4a3f7c2c6954d1ce69e9373b9ce058b73549e31d` / spec blob
`28ec5dea119e20220481f672e3c5bc00d1a0ddf7`, referencing desktop bookkeeping checkpoint
`0d8b5b19d9bab4b509db550f8b6ce57441b80ed5`. The unique
`TDRP-R9-MTPROTO-PLAINTEXT-HANDSHAKE-ENVELOPE-WIRE-001` responsibility is
`implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `handshake.rs` is `301554c60fe318a621726cacdd37f0d5d1de2a3e`; canonical/desktop
telegram-network blob is `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is
`a7c10e88172ccd664bad0370acea8fac7a3e2f3d`. The existing shipping plaintext auth-key exchange now emits
fixed-U-equivalent 0..63 random 32-bit padding words with inclusive message-data length and fails closed on invalid
body/padding alignment, non-zero auth_key_id, invalid declared length and declared/actual mismatch. MessageIdGuard
remains the server message-id semantic owner.

Coverage is `responsibilities_total=102, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.
Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories,
gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable
evidence and independent release ACCEPT remain open. No executable validation was performed.


## Stage-A MTProto encrypted external envelope wire (round 102, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reads selected the next independently closable responsibility adjacent to the already-closed plaintext handshake envelope: fixed-U `AbstractConnection::prepareSecurePacket` in `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a` and `SessionPrivate::sendSecureRequest/handleReceived` in `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`. The exact external encrypted layout is two 32-bit words of little-endian `auth_key_id`, four words / 16 bytes of `msg_key`, then AES-IGE ciphertext. The manual MTProto envelope is not a TL constructor; `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` starts TL bodies only inside that boundary.

Existing-owner-first found no production delta: canonical `native/telegram-protocol/src/crypto.rs@7882db6b9f3f5e9643fb963526ccc21dd0862d73` already exposes `EncryptedEnvelope::to_bytes/from_bytes` with exactly the 8/16/ciphertext layout and `encrypt_message/decrypt_message` performs active-key and authenticated message-key checks. Existing shipping `native/telegram-network/src/lib.rs@54f509474391bb5118ba4678ba5c566ab3e74ec8` sends and receives through that owner. Durable spec is platform-core `bb84071c99141f457748825cb51ed6679ebd4c56` / `telegram-mtproto-encrypted-envelope-wire.md@78964da9df4b91a32d077d6f608d93dc81686bb9`. Focused contracts were authored at platform-core `89af115ada9770a1e21e920c9e30a5f5587833c1` / contract blob `b82069d5b7f42919e087efe053549ebddbe6e278` and mirrored to desktop at `e940ec506ea2909dc0f550d533c04f3af7b70d40`; they were not run.

The unique ledger adds `TDRP-R9-MTPROTO-ENCRYPTED-ENVELOPE-WIRE-001` as `implemented / coverage_disposition=no-production-delta / validation_status=not-run / accepted=false`. Coverage is now `responsibilities_total=103, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. The desktop network provider remains byte-identical at `54f509474391bb5118ba4678ba5c566ab3e74ec8`; the sparse desktop overlay has no duplicate `crypto.rs`, so canonical crypto ownership remains in platform-core.

Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 102 final source-level compliance

Platform-core final compliance is `e449b65cbd68a287c98221d2d84da824b53707ca` / spec blob `806c1bf1f850eec6e346d5dd47f90db6f24c6501`, referencing desktop bookkeeping checkpoint `a6541989393bb6d1e44e76c542192af0cef2c3c0`. The unique `TDRP-R9-MTPROTO-ENCRYPTED-ENVELOPE-WIRE-001` responsibility is `implemented / coverage_disposition=no-production-delta / validation_status=not-run / accepted=false`.

Canonical production `crypto.rs` remains `7882db6b9f3f5e9643fb963526ccc21dd0862d73`; canonical/desktop telegram-network blob remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is `b82069d5b7f42919e087efe053549ebddbe6e278`. Existing `EncryptedEnvelope` preserves exact LE64 auth-key id / 16-byte message-key / ciphertext external framing and the existing shipping decrypt path fails closed on active-key mismatch, empty/non-block-aligned ciphertext and authenticated message-key failure before body dispatch.

Coverage is `responsibilities_total=103, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No executable validation was performed.


## Stage-A MTProto secure inner message wire (round 103, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. The next independently closable responsibility after the already-closed encrypted outer envelope is the authenticated plaintext inside it. Fixed-U `SessionPrivate::sendSecureRequest/handleReceived` in `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` establishes the exact `LE64 server_salt | LE64 session_id | LE64 msg_id | LE32 seq_no | LE32 msg_len | body | padding` boundary and rejects `msg_len > 16 MiB`, non-4-byte body length, padding outside 12..1024 bytes, session mismatch and invalid inbound server message-id parity before body dispatch. `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a` remains the already-closed external envelope boundary; `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms this secure inner message is manually framed, not a TL constructor.

Existing-owner-first found one minimal production gap. Canonical `native/telegram-protocol/src/crypto.rs` already owned the full 32-byte internal header, body/padding, authenticated decode, expected-session validation and server-message-id parity. Platform-core `7ffc264ab3a2f7a2997c82a03e6f8f45ce4b5745` adds only the fixed-U 16 MiB body ceiling in early decode plus shared PlainMessage validation; resulting crypto blob is `32d69573acc6ef4ce7bcc80073f5cdd7e2bea7ad`. Shipping `native/telegram-network/src/lib.rs@54f509474391bb5118ba4678ba5c566ab3e74ec8` remains unchanged and continues to send/receive through `encrypt_message/decrypt_message`. Durable spec is platform-core `2f93a5c5dbfce33f3ea5ee85352f6f9b967ed042` / `telegram-mtproto-secure-inner-message-wire.md@00843c11318f7f3f4d5dd025d78ba0527c68abf2`.

Focused contracts were authored in platform-core and mirrored to desktop without execution. Canonical and desktop contract blob is `f8e109c74a82ebe860b15c22766f403998e85b81`; desktop mirror commit is `125891339237613d9f9813c32c55684f3b5b2ecd`. They pin exact internal offsets/endianness, authenticated round-trip, wrong-session rejection, invalid inbound server-message-id parity and the >16 MiB fail-closed boundary.

The unique ledger adds `TDRP-R9-MTPROTO-SECURE-INNER-MESSAGE-WIRE-001` as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage is now `responsibilities_total=104, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. The desktop network provider remains byte-identical at `54f509474391bb5118ba4678ba5c566ab3e74ec8`; no duplicate desktop `crypto.rs` is introduced.

Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 103 final source-level compliance

Platform-core final compliance is `d6ef0db52ef2cabb480959d358386bedd2bb080e` / spec blob `b0fd5be4e048a0b6ddf737c683c21c76dbf73372`, referencing desktop bookkeeping checkpoint `fa7a222978ce3760ddb78cf5bd4c9a381434fca8`. The unique `TDRP-R9-MTPROTO-SECURE-INNER-MESSAGE-WIRE-001` responsibility is `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `crypto.rs` is `32d69573acc6ef4ce7bcc80073f5cdd7e2bea7ad`; canonical/desktop telegram-network remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is `f8e109c74a82ebe860b15c22766f403998e85b81`. The canonical owner now rejects secure-inner bodies above 16 MiB both on outbound validation and early authenticated inbound decode while retaining the exact 8/8/8/4/4 header, padding, session and inbound message-id checks in the existing owner.

Coverage remains `responsibilities_total=104, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No executable validation was performed.


## Stage-A MTProto abridged transport frame wire (round 104, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. The next independently closable exact wire responsibility is fixed-U `TcpConnection::Protocol::Version0` abridged framing in `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`: 1..126 four-byte words use one length byte; 127+ words use `0x7f` plus a 24-bit little-endian word count; the fixed-U decoder rejects the extended form when the decoded count is below 127. `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` confirms the frame is not a TL constructor.

Existing-owner-first found one exact production gap. Platform-core spec-first commit `6f0d7e3cfc1d8d90fa47360b21ae57900df2d33d` / spec blob `a5ff2064fc9386cbfa4dc428b633291ff6d9623f` isolates only abridged frame wire responsibility. Production commit `97c38a1bde61edc53ed9d888ffb3cb541e4ff68f` changes only canonical `native/telegram-protocol/src/transport.rs@7f981b96f2f8a17823ba9c84bf2e2324d408be31` so `decode_abridged` fails closed on a noncanonical extended count below 127. Shipping `native/telegram-network/src/lib.rs@54f509474391bb5118ba4678ba5c566ab3e74ec8` and the desktop network provider with the same blob remain unchanged and continue to use `TransportFrameCodec`.

Focused contracts were authored only, not run. Canonical and desktop mirror contract blob is `00eff72062ab8d3285234428bd09be3fccb27993`; desktop mirror commit is `f2405ce2ea29785fbc218840d3e9bf1ff4048576`. The contracts pin 126-word short framing, 127-word canonical extended framing, noncanonical extended-126 rejection and fragmented extended-header behavior.

The unique ledger adds `TDRP-R9-MTPROTO-ABRIDGED-TRANSPORT-FRAME-WIRE-001` as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=105, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Connection-start obfuscation/AES-CTR prefix, Version1/VersionD secret handling, quick-ACK lifecycle, auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 104 final source-level compliance

Platform-core final compliance is `5329e70f7456b0a6d3de0071ab6576e45bbeeab2` / spec blob `4d7f42773ce37be5c911a64328c4c04684eeab0a`, referencing desktop bookkeeping checkpoint `505eeb190d50a90d955be27f432b3a60f7cf5757`. The unique `TDRP-R9-MTPROTO-ABRIDGED-TRANSPORT-FRAME-WIRE-001` responsibility is `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `transport.rs` is `7f981b96f2f8a17823ba9c84bf2e2324d408be31`; canonical/desktop telegram-network remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is `00eff72062ab8d3285234428bd09be3fccb27993`. The existing owner now rejects the fixed-U noncanonical `0x7f + LE24` abridged length form when the decoded word count is below 127 while retaining the exact 1..126 short and 127+ extended forms.

Coverage is `responsibilities_total=105, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Connection-start obfuscation/AES-CTR prefix, Version1/VersionD secret handling, quick-ACK lifecycle, auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No executable validation was performed.


## Stage-A MTProto padded-intermediate transport frame wire (round 105, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual adjacent review of `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381` selected `TcpConnection::Protocol::VersionD` frame body as the next independently closable wire responsibility. It is deliberately separated from the 64-byte connection-start nonce/AES-CTR prefix, Version1/VersionD secret derivation, protocol/DC-id obfuscation, quick-ACK lifecycle and connection scheduling.

Fixed U writes `LE32(payload_bytes + padding_bytes)`, requires the underlying MTProto payload to be a non-empty sequence of 32-bit words, appends 0..15 random padding bytes, treats incomplete headers/bodies as partial input, and accepts only total packet lengths from 8 bytes through one byte below `0x04000000`. The VersionD parser does not independently implement generic intermediate quick-ACK token semantics.

Existing-owner-first implementation stays in canonical `telegram-protocol::TransportFrameCodec`. Platform spec `ed28746a014682767de9866966eb0b4bb2953618` / initial spec blob `2ecb3e305e850ffd67c7dcc4740bb4b283831566` precedes production commit `3a3d850139082e32a783b7d03b6ea2fd31b00ee9`, which updates only `native/telegram-protocol/src/transport.rs@5d6abd61f130b25baae10a6269eebd20fd025b3a`. Shipping `native/telegram-network/src/lib.rs@54f509474391bb5118ba4678ba5c566ab3e74ec8` is unchanged and still delegates socket frames to the shared codec.

Canonical authored-only contracts are `cbc042f54b5b2bb7ec035ad68aaad5bd40263b4a` / `cd53d0b1b546aaceaf5ad46aaf6957dab5b32bf9`; desktop mirror commit `551d641ba59ae6f2ef7e3c08bfbf884db8f7f2a3` is byte-identical. Platform implementation-record commit is `440da6a1b0f24907e77f1c1a94761adb9d17ae1d` / spec blob `b6ea2c2df755fe632d624e6621e5b89379a47b0e`.

The unique ledger adds `TDRP-R9-MTPROTO-PADDED-INTERMEDIATE-TRANSPORT-FRAME-WIRE-001` as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=106, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Contracts cover explicit padding 0/15, alignment, fragmented header/body, 0..3 declared-body rejection, fixed-U upper packet boundary, random default padding domain, VersionD quick-ACK rejection and preservation of generic Intermediate quick-ACK behavior.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. Connection-start obfuscation/AES-CTR prefix, Version1/VersionD secret handling, quick-ACK lifecycle, auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.


### Round 105 final source-level compliance

Platform-core final compliance is `21c433fb6b1a37b91fa0757b018868c21aa511dc` / spec blob `430fb4257e87965e465474a30c60230e9c8e6618`, referencing desktop bookkeeping checkpoint `44da95b4b754433990a48b23cbc323bc8ca44030`. The unique `TDRP-R9-MTPROTO-PADDED-INTERMEDIATE-TRANSPORT-FRAME-WIRE-001` responsibility is `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Canonical production `transport.rs` is `5d6abd61f130b25baae10a6269eebd20fd025b3a`; canonical/desktop telegram-network remains `54f509474391bb5118ba4678ba5c566ab3e74ec8`; canonical/desktop authored contract blob is `cd53d0b1b546aaceaf5ad46aaf6957dab5b32bf9`. The existing owner now matches fixed-U VersionD frame-body semantics for random 0..15 outbound padding, unpadded word alignment, exact receive floor/strict packet ceiling, fragmented input, and separation from generic Intermediate quick-ACK forms.

Coverage is `responsibilities_total=106, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Connection-start obfuscation/AES-CTR prefix, Version1/VersionD secret handling, protocol/DC id obfuscation, quick-ACK lifecycle, auth-key binding/persistence/expiry/renewal, broader connection scheduling/racing/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open. No executable validation was performed.


### Round 106 MTProto obfuscated connection-start wire source-level closure

Fixed-U authority remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. The newly isolated responsibility `TDRP-R9-MTPROTO-OBFUSCATED-CONNECTION-START-WIRE-001` binds `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381` to its adjacent CTR/nonce/secret/DC-id definitions in `connection_tcp.h`, `mtproto_auth_key.*`, `mtproto_tcp_socket.cpp`, `mtproto_dc_options.cpp` and `session_private.cpp`.

Existing-owner-first implementation is platform-core spec `17d0c7e803d710f98cfcc2d7dcd64829819d5241`, production `0f93e212d5151c598f19a67b0a65ad19774dc1c3`, and authored-only contracts `8b04666c4e1d0c8c986432bcf272a6679f96566d`. Canonical `telegram-protocol/src/transport.rs@74189488dc3ec1015edc59989f7aced8ad65f5c3` now owns exact secret classification, direct nonce filtering, Version1/VersionD SHA-256 key derivation, 64-byte prefix assembly and persistent AES-256-CTR direction state. Canonical `telegram-network/src/lib.rs@54245116969f53ee53ccc3f979bbd9fb7382cc97` threads that state through the existing shipping Main/test direct-TCP path without creating a second runtime or scheduler.

Desktop provider synchronization is `76e467b5a1b3aecceffc4cdb6d34dc12461f12d3`: network and protocol-lib mirrors are `54245116969f53ee53ccc3f979bbd9fb7382cc97` and `d5727367035cef2adbba0a6a6a2f6b95e2716450`; desktop authored protocol contract mirror is byte-identical to canonical at `4de139a0f98c2d2b784145aa1995a67458af9c97`. Canonical network contract is `1a94aaa3f79e8108e04bc5418657fd4e641d0a4a`.

The unique ledger records this responsibility as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=107, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Unknown/unread do not decrease because the same fixed-U connection family still contains explicitly open proxy/TLS camouflage, media/temporary DC semantics, quick-ACK lifecycle and broader connection scheduling/retry responsibilities.

Contracts are authored only and were not executed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed. Auth-key binding/persistence/expiry/renewal, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.


### Round 106 final source-level compliance

Platform-core final compliance is `5b7ed34df4c8063fe3db9627552bc673dda9c92f` / spec blob `65dc6e1d9e611ad5428fc7c9c85551ac6d868081`, referencing desktop bookkeeping checkpoint `3b59512feb6794c235f03a3e83006abb94f11bdb`. The superseding source-reviewed protocol contract is platform `ecf143c6edec0c859e2c19a0f94aa1c9dca215ae` and desktop mirror `6f950d056170a232d22ee08c6975cb051592594f`, byte-identical at `307c973117a96fa901662d021ac79f5ec76b28cf`; canonical network contract remains `1a94aaa3f79e8108e04bc5418657fd4e641d0a4a`.

Canonical production remains `transport.rs@74189488dc3ec1015edc59989f7aced8ad65f5c3`, protocol export `lib.rs@d5727367035cef2adbba0a6a6a2f6b95e2716450`, and shipping `telegram-network/src/lib.rs@54245116969f53ee53ccc3f979bbd9fb7382cc97`. Desktop provider mirrors the network/protocol export blobs exactly. The unique `TDRP-R9-MTPROTO-OBFUSCATED-CONNECTION-START-WIRE-001` responsibility remains `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`.

Coverage is `responsibilities_total=107, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Proxy/Web/TLS camouflage, media-cluster negative/temporary protocol DC-id semantics, quick-ACK lifecycle, broader connection scheduling/racing/reconnect/send retry, auth-key binding/persistence/expiry/renewal, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No executable validation or GitHub Actions/status/check query was performed.


### Round 107 MTProto protocol DC-id wire source-level closure

Fixed-U authority remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, `facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256`, `core_types.h@9d025d8ae265e3fe2eef3f32fc609e79a0c4de7f`, `mtp_instance.h@eeb56d29cc8316bd6600c26b6f8edf3a912601eb`, `connection_abstract.h@217b684fa222c42e968dfddeaa62f16a0054017f` and `mtproto_dc_options.*` isolated `TDRP-R9-MTPROTO-PROTOCOL-DC-ID-WIRE-001`: shifted ids reduce to the bare directory identity, temporary bare ids >=1000 normalize only for the protocol wire, test mode adds 10000, and an explicit MediaCluster wire role negates the resulting signed 16-bit id.

Existing-owner-first canonical implementation is platform-core spec `9d0f25b21b46ef74f4589dec1d3459117d4bdbd6` / spec blob `725069760d50242874de30ddc5a2045895c23103`, production `e9cfc7fbd06002c6510d3bf45df9710e55ff1d57`, and authored-only contracts `ceacfa08800950103c6118b6c579ef582b1e1458`. Canonical `telegram-protocol/src/dc.rs@a9582e4ece93590c322febbff391ba3ff7635575` now owns the pure identity transform; `telegram-network/src/lib.rs@3034acab752ca37531033bb8a1d106003b20265d` remains the socket/endpoint carrier and consumes the returned directory id and wire id separately. No second DC owner, runtime, scheduler, or auth-key lifecycle was introduced.

Desktop provider synchronization is `385e5a261da3620733053503027c9a28c71cc221`: desktop `telegram-protocol/src/dc.rs@a9582e4ece93590c322febbff391ba3ff7635575`, protocol facade `daf27c48a422ed6ed663ca0021ff138436d99c0b`, network `3034acab752ca37531033bb8a1d106003b20265d`, and protocol contract `adc51856cebcdb6e539828e3d4abb9952fc32c29` are byte-identical to canonical. Canonical network contract is `4adbcabf66dc0ca35861f46e40e9a8ca77fb73c5`.

The unique ledger adds this responsibility as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=108, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Unknown/unread intentionally do not decrease because the broader fixed-U session/connection family still contains proxy/TLS camouflage, quick-ACK lifecycle, currentDcType/auth-key lifecycle, connection scheduling/retry, arbitrary request histories, gzip and complete difference decoding responsibilities.

Contracts were authored only and were not executed. No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


### Round 107 final source-level compliance

Platform-core final source-level compliance is `0da9e826633817068f06159d90d46eaf664aa51b` / spec blob `cdb4a4565257b7919f510244ec4a11a4c7fcf32f`, referencing desktop bookkeeping checkpoint `dc671558c60f4c292e2a8bf6de8faf1000da6c89`.

Canonical production remains `e9cfc7fbd06002c6510d3bf45df9710e55ff1d57`: `telegram-protocol/src/dc.rs@a9582e4ece93590c322febbff391ba3ff7635575`, protocol facade `daf27c48a422ed6ed663ca0021ff138436d99c0b`, and shipping network `3034acab752ca37531033bb8a1d106003b20265d`. Authored-only protocol/network contracts remain `adc51856cebcdb6e539828e3d4abb9952fc32c29` and `4adbcabf66dc0ca35861f46e40e9a8ca77fb73c5`; desktop provider mirrors the production/protocol contract blobs exactly at checkpoint `385e5a261da3620733053503027c9a28c71cc221`.

The unique `TDRP-R9-MTPROTO-PROTOCOL-DC-ID-WIRE-001` responsibility remains `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=108, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No executable validation or GitHub Actions/status/check query was performed. Proxy/Web/TLS camouflage, quick-ACK lifecycle, broader currentDcType/auth-key lifecycle, connection scheduling/racing/reconnect/send retry, arbitrary request histories, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.


### Round 108 quick-ACK wire boundary

Fixed-U review: `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`, `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a`, and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`. Fixed-U Version0/VersionD shipping framing does not request quick-ACK; `parsePacket/socketPacket` explicitly leaves the newer quick-ACK receive form unsupported. Existing canonical `telegram-protocol` already owns generic history-free token derivation and Abridged/Intermediate marker/token decoding.

Spec-first checkpoint: platform-core `1a09a5674ec16af4228bbf0aed0fb302cb52fba0` / spec blob `6cbc5fdc140759e66c3c14ac853bdf18c8ce4fba`.

Minimal canonical production: platform-core `605b8928de5fb64bf235196093ef804e350994f2`, `native/telegram-network/src/lib.rs@7c442299aedd4aa7444ba53c7ab8212542dfc2d1`. Shipping `send_payload` still uses ordinary `TransportFrameCodec::encode`; `receive_payload` now returns typed `NetworkError::UnexpectedQuickAck(token)` instead of silently discarding an uncorrelated token. No request-history/correlation owner was added.

Authored-only canonical contracts: platform-core `d1cec8b3952a4be1b02c6f1dae1ab008991a0342`, network contract blob `9dfb554bae0dac9ea68ebd6b2fe7cab664cc1240`. They pin a shipping abridged send with the quick-ACK request bit clear and an unsolicited abridged token failing closed. They were not run.

Desktop provider synchronization: `f24b51ec5ba2d959f8c08fe44c6af706f38d7224`; mirrored network production `7c442299aedd4aa7444ba53c7ab8212542dfc2d1` and network contract `9dfb554bae0dac9ea68ebd6b2fe7cab664cc1240` are byte-identical to canonical.

The unique ledger adds `TDRP-R9-MTPROTO-QUICK-ACK-WIRE-001` as `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=109, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Unknown/unread remain intentionally unchanged because broader connection scheduling/racing/reconnect/send retry, arbitrary request histories, proxy/Web/TLS camouflage, auth-key lifecycle, gzip and complete difference decoding/application remain open.

No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E execution, GitHub Actions/status/check query, package, deployment, installation or runtime acceptance was performed.


### Round 108 final source-level compliance

Platform-core final source-level compliance is `b48b9a92c45f7954322946fb7640b53affc5d7bc` / spec blob `2fb3554e7c6d320ff8d8009f78f04b6436ef1c19`, referencing desktop bookkeeping checkpoint `da6bbfcafabb66681ba8614cf2d97c9e9e43ca5f`.

Canonical production remains `605b8928de5fb64bf235196093ef804e350994f2`: `native/telegram-network/src/lib.rs@7c442299aedd4aa7444ba53c7ab8212542dfc2d1`; the existing protocol quick-ACK wire primitives remain unchanged at `crypto.rs@32d69573acc6ef4ce7bcc80073f5cdd7e2bea7ad` and `transport.rs@74189488dc3ec1015edc59989f7aced8ad65f5c3`. Authored-only network contracts remain `9dfb554bae0dac9ea68ebd6b2fe7cab664cc1240`; desktop provider production/test mirrors are byte-identical from checkpoint `f24b51ec5ba2d959f8c08fe44c6af706f38d7224`.

The unique `TDRP-R9-MTPROTO-QUICK-ACK-WIRE-001` responsibility remains `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`. Coverage remains `responsibilities_total=109, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No executable validation or GitHub Actions/status/check query was performed. Quick-ACK request-to-request correlation/arbitrary request history, proxy/Web/TLS camouflage, broader auth-key lifecycle and connection scheduling/racing/reconnect/send retry, gzip, complete difference/differenceSlice decoding/application, broader fixed-U source/legal closure, Stage-B evidence and independent release ACCEPT remain open.


## Stage-A MTProto TLS camouflage exact wire closure (evidence round 109; task round 78, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fresh reread separates the earlier blocked TLS camouflage lifecycle into an exact scheduler-independent wire responsibility and still-open transport lifecycle work. `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16` plus `mtproto.tl@76225965bfc794388db241f3f760db8ab79e84f8` define local template-to-ClientHello generation, authenticated ServerHello boundaries and TLS application records; adjacent `mtproto_web_proxy_socket.cpp@9aa05e7034031310de4aa9d410fbb15d21c63ee8` owns Web transport stream registration, buffering/window grants and lifecycle and remains outside this closure.

Spec-first platform-core is `7313529ae77d1abb99f678e8105ec4525474fe55` / spec blob `ba258901a12ba33fbf1f89b31867a87ef9fd88bc`. Production is `e4fb391fc4f9f6e151f35469e0ded7326be39749`: canonical `telegram-protocol/src/tls.rs@d15b24a350cc69960efa6f84c509245178917ba9` owns exact wire/crypto and `telegram-network/src/lib.rs@241cd8ec792070a8de8af9ec6ca5b317e34376d1` consumes it in the existing synchronous `0xEE` TcpTransport path without new scheduler/request-history ownership. Authored-only contracts are `9b43bb1bb0bb557a9c1289370f4fdc30a571ad1c`, protocol test blob `212728aacd277b38821d110379215f75c2b4b675`, network test blob `71f9732b1c81d7bf782ff1eccb2852b1401cfa6f`; none were run. Desktop provider sync `6b914b3089a5d3593c276aa37120f9c2f53db198` is byte-identical for all mirrored production/test blobs.

The sole ledger responsibility `TDRP-R9-MTPROTO-TLS-CLIENT-HELLO-TEMPLATE-001` is upgraded to `implemented / coverage_disposition=rewritten / validation_status=not-run / accepted=false`; totals remain `responsibilities_total=109, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0` because this refines an existing row. Proxy/Web lifecycle, broader scheduling/racing/reconnect, arbitrary request history, auth-key binding/persistence, gzip, complete difference decoding/application, remaining fixed-U unknown/unread, broader source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deployment, installation or runtime acceptance was performed.


Platform-core final source-level compliance was then recorded at `c0f2a6c4098cc5b54f3ee580db078e55c61982ab` / spec blob `e266a7805ac552bc23614d7344b5e2e223ac1b3a`. It records R1-R12 as source-review passed where applicable while explicitly retaining executable validation as not-run. No Stage-B gate state changed.

## Stage-A WebProxy relay stream/window/nonce boundary (evidence round 110; task round 79, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Fresh exact-source reads cover `web_proxy_frame.h@a5ba491fed67d70aee88e6ec2e3ac47c991e78f3`, `web_proxy_frame.cpp@bd1d7b310f40203f9da914efc8bdd6711035884b`, `web_proxy_transport.h@f58850d2bc2ceabd93113d7725903e1c0d92d71b`, `web_proxy_transport.cpp@e25ad970e84686e9aca061d02c274048793b5ad4`, `mtproto_web_proxy_socket.cpp@9aa05e7034031310de4aa9d410fbb15d21c63ee8`, `mtproto_web_proxy_socket.h@02ed27b77d4ff606b3e1cb548a60e4f2c2ca11f0`, `web_proxy_webview.cpp@8a2efc8fb15e400366f199cd956effc8ad362da8` and adjacent `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`.

The exact independent wire facts are now durable in platform-core spec-first commit `bde283f60d5413426649737a47912b5ec4c3899a` / spec blob `b7de819a9594f74c6aab7f75112c6876f2b21e87`: an 8-byte relay header (type / BE24 stream id / BE32 payload length), 1 MiB payload cap, 4096-frame batch cap, fixed frame-type set, and a BE32 nonzero WINDOW amount. Per-stream send/receive credit starts at 4 MiB; DATA consumes credit, incoming WINDOW saturates send credit at u32::MAX, and `WebProxySocket::read` grants receive credit back only for bytes actually drained. Fixed-U coalesces returned credit at 256 KiB or 20 ms and caps DATA chunks at 64 KiB. The hidden-WebView bootstrap nonce is 32 secure-random bytes, URL-safe base64 without padding in `#android=`; only exact `tproxy-android-init` version 1 + nonce match queues `Hello(0,[1])`.

Existing-owner-first resolves the relay multiplexing/window state to canonical `telegram-network`, not `telegram-protocol`. Live canonical network production remains `native/telegram-network/src/lib.rs@241cd8ec792070a8de8af9ec6ca5b317e34376d1` and authored external contract `tcp_transport.rs@71f9732b1c81d7bf782ff1eccb2852b1401cfa6f`; desktop mirrors are byte-identical. Neither tree contains a shipping WebProxy carrier/logical-stream implementation or selectable caller. Therefore this round intentionally makes no production/provider code delta: an isolated frame codec would be dead/unwired production API, while the WebView nonce is inseparable from navigation/origin/adoption/health/retry carrier lifecycle. The unique ledger adds `TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001` as `mapped / coverage_disposition=pending / validation_status=not-run / accepted=false`. Coverage becomes `responsibilities_total=110, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; global unknown/unread are not decremented by this partial semantic closure.

Authored source-level contract vectors are recorded in the durable platform spec and were not executed. No hypothetical test-only codec was added because there is no production WebProxy symbol to exercise. Production implementation remains blocked until the existing `telegram-network` shipping call graph can actually select/instantiate a WebProxy carrier. Broader connection scheduling/racing/reconnect/send retry, arbitrary request history, auth-key binding/persistence, gzip, complete difference decoding/application, remaining fixed-U source/legal closure, Stage-B executable evidence and independent release ACCEPT remain open.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.

### Round 110 final source-level compliance

Platform-core final source-level compliance is `0fe86b6d40d1e7ad05b7f4a72ce2c8945cf06a6a` / spec blob `4c8faf410867a0784623b9e00e73d7a9f46bc3cf`, referencing desktop bookkeeping `88cb56ab51df6e00907800871d49cd98294b756a`. AC-4 is now passed for source bookkeeping; R16/AC-6 remain explicitly blocked because the current canonical `telegram-network` call graph still has no real shipping WebProxy carrier entrypoint. The unique responsibility remains `mapped / coverage_disposition=pending / validation_status=not-run / accepted=false`; no production/provider/test blob changed and no executable validation was performed.



## Stage-A WebProxy bridge capability/path/link-secret boundary (evidence round 111; task round 80, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact-source review isolates `mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef` / header `d8deebac2359936f72901ba0eec0cbfdfb186e90` from the separately blocked relay carrier. The pure boundary preserves the root v1 and path v2 domain-separated HMAC-SHA256 contexts, unpadded base64url capability, bounded canonical base path, exact bridge path/URL, and 0x70 path-link-secret marker. Consuming fixed-U sources are `web_proxy_webview.cpp@8a2efc8fb15e400366f199cd956effc8ad362da8`, `web_proxy_transport.cpp@e25ad970e84686e9aca061d02c274048793b5ad4`, and `connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2`.

Spec-first platform commit is `4da2078cae694e8aadc45a5a03d8f206701d1ead` / initial spec blob `cae43055ab17dd8c24aed6ebf06e0c8873efb16a`. Canonical production is source-reviewed through `de586fc446d7ac36d7542c5b23736964de70019b`: `native/telegram-protocol/src/web_proxy.rs@4c91f6e4e6b8d37d84afd40aae6af9fbbe7d80d4`, export lib `42601393105e0ad49b8da97ee8cfd2acc6cf031e`. Authored-only contracts are `d87483eae204ba18aac503e2fa1f4e059b9f784f` / `protocol_contract.rs@2f4fed5e9d6f9103d3598ef94089dbede6d226af`; they were not executed.

Desktop source sync is `7ebdbd7cd9ea4edb0b793be4cd7c616e15d80f71` and contract sync is `3d6489da4056a65d5dedc7a4094474863cbe3af6`; source/lib/test blobs are byte-identical to canonical. Canonical and desktop telegram-network production remain unchanged at `241cd8ec792070a8de8af9ec6ca5b317e34376d1`, so relay/window/bootstrap/scheduler code is not smuggled into this closure.

The unique new responsibility is `TDRP-R9-MTPROTO-WEB-PROXY-BRIDGE-CAPABILITY-001`: `implemented / rewritten / validation_status=not-run / accepted=false`. Coverage becomes `source_entries_total=16202, responsibilities_total=111, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; global unknown/unread are not decremented by this focused semantic closure. `TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001` remains `mapped/pending` until a real shipping carrier exists.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 111 final source-level compliance

Platform-core final source-level compliance is `129d232fc891074ec11cd67a4235b78521bcee7c` / spec blob `87405d6f79801a891d1837ebdd3a3606a38a6b09`. It records R1-R14 and AC-1 through AC-7 as passed by source review, while executable validation remains intentionally `not-run`. Desktop bookkeeping checkpoint `8e81014db9f01831e6a3b91816f8278280b660b6` is referenced by that compliance record.

The unique bridge-capability responsibility remains `implemented / rewritten / validation_status=not-run / accepted=false`. The separate relay carrier/window/WebView bootstrap responsibility remains `mapped/pending`; no telegram-network production blob changed.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.

## Stage-A WebProxy canonical address/settings boundary (round 112, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`, `mtproto_proxy_data.h@d8deebac2359936f72901ba0eec0cbfdfb186e90` and `connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2` closes the semantic read for WebProxy canonical user address plus import/share settings: Unicode host input is converted with `QUrl::toAce`, lowercased and DNS/IP-validated; the address field strips only HTTPS, stores canonical `host[/base-path]`; sharing emits `webproxy?server=<encoded webAddress>&secret=<path-aware secret>` without a port; import uses `server` with legacy `host` fallback, forces 443 and applies path-aware secret decode.

Spec-first platform checkpoint is `a9fe9aa38c67102fda5b568140183857e776e58a` / initial spec blob `27d16a8bedf20c51860889602f67cedb305a205b`. Existing canonical `native/telegram-protocol/src/web_proxy.rs@4c91f6e4e6b8d37d84afd40aae6af9fbbe7d80d4` already owns canonical ASCII host validation plus base-path/bridge/link-secret wire, but `Cargo.toml@a62d99f23d370cd75590c7b3186307c1689554ed` and `Cargo.lock@97ad9046ce8926764258c978664cd8cd92fce503` have no reviewed IDNA/URL dependency. The live platform and desktop checkpoint trees expose no unified Fabushi proxy/settings production owner for the user-input/import/share/persistence/selection lifecycle. Creating a Telegram-specific settings subsystem or hand-written/approximate IDNA would violate existing-owner-first and exact-equivalence requirements; executing dependency resolution/lock regeneration is prohibited in Stage A.

Therefore `TDRP-R9-MTPROTO-WEB-PROXY-CANONICAL-ADDRESS-SETTINGS-001` is added as `mapped / pending / validation_status=not-run / accepted=false` with no production delta. Authored contract vectors live in the durable spec and include the fixed-U Unicode/ACE positives (`bücher.example -> xn--bcher-kva.example`, `bücher.de -> xn--bcher-kva.de`), numeric/IP/delimiter negatives, HTTPS-only address parsing, and exact import/share field semantics; no executable contract was run and no fake API test was added. Canonical/desktop WebProxy production remains `4c91f6e4e6b8d37d84afd40aae6af9fbbe7d80d4`; canonical/desktop telegram-network remains `241cd8ec792070a8de8af9ec6ca5b317e34376d1`.

Coverage becomes `source_entries_total=16202, responsibilities_total=112, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; global unknown/unread are unchanged by this focused responsibility review. Existing bridge capability/path/link-secret remains implemented; relay carrier/window/bootstrap remains mapped/pending.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.

### Round 112 final source-level compliance

Platform-core final source-level compliance is `2682d35b46d5521d71ea82896557466c3e07a58f` / spec blob `eb924c6492ffb4f545b9ccc2eec404284c41335d`. It records exact fixed-U address/import/share semantics, source-review passes for the understood boundary, and explicit blockers R10/R12: no live unified Fabushi proxy/settings production owner and no reviewed exact IDNA dependency/dependency-lock path available under Stage-A zero-execution rules.

The unique `TDRP-R9-MTPROTO-WEB-PROXY-CANONICAL-ADDRESS-SETTINGS-001` row remains `mapped / pending / validation_status=not-run / accepted=false`. No protocol/network production blob changed; bridge capability remains implemented and relay carrier/window/bootstrap remains separately mapped/pending.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.



## Stage-A proxy settings state/persistence boundary (round 113, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99`, `core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26`, `core_settings.h@7192f0db82ddc214489f4c8bd5c247b8911b8a07`, `core_settings.cpp@6ef88cd5f0b946b0b92512d1a94e42a0f5c1fabb`, `localstorage.cpp@d71385083c0e940580398612fb65e522afc768b1`, `connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2`, `application.cpp@7172206f5c1e540e19c90798e9a9e5721276cc53` and `proxy_rotation_manager.cpp@72723d4db408be654dbf68e1ac40bba21c59d4e6` establishes one durable source owner: `Core::SettingsProxy` is embedded once in `Core::Settings` and owns proxy mode, selected proxy, ordered list, IPv6/call flags, IP-warning bit, rotation toggle/timeout and preferred indices. `connection_box` keeps only an edit/view projection; `ProxyRotationManager` owns transient checks/timers and writes preferred order or selection back through the same Settings owner.

Fixed-U persistence is likewise one chain: `SettingsProxy::serialize()` -> `Core::Settings::serialize()` -> encrypted `dbiApplicationSettings` in `Local::writeSettings()`; restore reverses that chain through `SettingsProxy::setFromSerialized()`. Exact list/preferred-index mutation, timeout normalization, proxy type mapping and append-compatible legacy decode semantics are recorded in the durable contract.

Spec-first platform checkpoint is `5724aee519adcd3990bbf98f88a97581602bc313` / initial spec blob `e31a0d4f4d9f9286f7af35562c7d9edac6687487`. Current Fabushi candidates do not supply the required owner: `mahayana-app-host/src/lib.rs@ed69b2b7aa0d87ebd34210e9a13d9c8a8b3e3087` persists generic feature client/UI values in `feature-host/runtime/runtime.sqlite3` table `ui_state`; Telegram storage `sqlite.rs@e6cc314ec749f8a258bfe0125a25beb3613eb2ca` is encrypted Telegram runtime/auth/transport private persistence; `fabushi-account-service.ts@8e26fd5eab70d27ee553791ca30a8d6147358bf5` owns account/session plus protected Telegram storage-key material. None is the canonical product Settings/application state owner. Telegram network and rotation runtime are also rejected because they are consumers/schedulers, not durable settings owners.

Therefore `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` is added as `mapped / pending / validation_status=not-run / accepted=false` with no production delta. The state responsibility is independent of WebProxy carrier, but implementing it now would require either promoting UI/provider storage into the wrong owner or creating a parallel Telegram settings subsystem. Authored contract vectors live in the durable spec; no executable contract was run and no fake test against a nonexistent Settings API was added.

Coverage becomes `source_entries_total=16202, responsibilities_total=113, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; global unknown/unread remain unchanged by this focused source review.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 113 final source-level compliance

Platform-core final source-level compliance is `1cff6f7176de9d879576e0630503626636240622` / spec blob `76411ecf397323d4a3760c4a1c444c672d833105`. It records exact fixed-U proxy settings state/mutation/persistence semantics and the unique `Core::SettingsProxy` source ownership chain. R1-R12/R15 and AC-1..AC-8 are source-review passed; R13-R14 remain explicitly blocked because the current Fabushi checkpoints have no canonical product Settings/application state owner. Existing AppHost `ui_state`, Telegram encrypted runtime storage, account-service, telegram-network and rotation runtime remain rejected as authoritative settings owners.

The unique `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` row remains `mapped / pending / validation_status=not-run / accepted=false`. No production blob changed; the responsibility is carrier-independent but cannot be shipped without the canonical unified Settings owner.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A proxy non-UI consumers boundary (round 114, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source review of `external_control.cpp@43327bc03c528b65b41ce72add5e7cca584ea371`, `calls_call.cpp@a67723f5827e56be3f36033f3ad59bc37d0acf7b`, `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`, `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`, `core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26` and `mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef` splits three non-UI SettingsProxy consumers without reopening durable ownership.

External proxy automation list/add/remove/use/next/toggle/undo is a command adapter over the same SettingsProxy and Local::writeSettings persistence. The fixed-U call projection is deliberately inert because `ProxyData::supportsCalls()` returns false unconditionally, so no tgcalls proxy descriptor is reachable. MTProto projects proxy-enabled state into ConfigLoader/special-config suppression and maintains transient resolved-IP ordering/expiry; those fields are not serialized, and selected writeback/refresh remains gated through the Settings owner.

Current Fabushi still has no canonical product Settings/application owner. Desktop telegram-network `241cd8ec792070a8de8af9ec6ca5b317e34376d1` and telegram-runtime `10031c5261bb0810ad2df2fca3de2e2fe4f65301` contain no shipping proxy/config-loader/custom-resolution consumer, and the tracked desktop tree has no shipping call/proxy consumer to extend. Therefore three exact responsibilities are recorded as mapped/pending with no production delta: `TDRP-R9-PROXY-EXTERNAL-MUTATION-CONSUMER-001`, `TDRP-R9-PROXY-CALL-CONSUMER-001`, and `TDRP-R9-PROXY-MTPROTO-CONSUMER-001`.

Spec-first platform commit is `2e155bb7be49b9e745f94cdc8b40fd831e052af7` / initial spec blob `b4220a6ac80b210cc8a969925ea0b180405486d7`. Coverage becomes `source_entries_total=16202, responsibilities_total=116, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; focused semantic review does not decrement global unknown/unread.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 114 final source-level compliance

Platform-core final source-level compliance is `de2460e31dfa5d47df0a1993580ef11a1cfa1378` / spec blob `438b64863eab0cc6fc746954cff8513de2fdd247`. It references desktop initial bookkeeping commit `a23452a00e9c4d395f3f97bf74ec27fd82747fb2` and the synchronized ledger/P0/source-closure/evidence identities.

The three exact proxy non-UI consumer rows remain `mapped / pending / validation_status=not-run / accepted=false`. External mutation is blocked on the missing canonical Fabushi Settings/application owner; the fixed-U call consumer intentionally has no runtime proxy effect because `ProxyData::supportsCalls()` is unconditionally false; MTProto enabled/config-loader and transient DNS projections are blocked on the missing canonical Settings event source plus a real shipping selected-proxy/config-acquisition consumer. No production blob changed.

R1-R8 and AC-1 through AC-8 are satisfied at source/bookkeeping review level. R9 production implementation remains explicitly blocked for the reasons above, and R10 executable validation remains intentionally not-run.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A MTProto special-config payload boundary (round 115, 2026-11-01)

## Fixed-U source read

`config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194` suppresses `SpecialConfigRequest` when proxy use or keys-destroyer mode forbids it. Outside that gate, `SpecialConfigRequest` acquires compact special-config text, emits source DC endpoint candidates, and `ConfigLoader::sendSpecialRequest` maps a source DC to a temporary DC id, inserts a transient `tcpo_only` endpoint, sends `help.getConfig` through that temporary session, then applies only the returned `dc_options` via `DcOptions::setFromList`.

`special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` splits cleanly into a pure payload boundary and a carrier scheduler. The pure boundary normalizes to 344 base64 bytes, decodes 256 bytes, performs pinned raw RSA public recovery, uses recovered bytes 0..32 as AES-256 key and 16..32 as IV, CBC-decrypts bytes 32..256, checks the first 16 bytes of SHA-256 over the first 208 plaintext bytes, validates a positive aligned declared length <=208, reuses `help.configSimple`, rejects only `now > expires` plus empty rules, applies ordered comma-separated phone-prefix rules where matching negative tokens immediately reject, and projects dc/ip/port/secret candidates. `mtproto_rsa_public_key.cpp@312c8b1ddb3145d5ec3180323e8d72d50c35e526` confirms no-padding raw RSA public operation. `mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a` confirms temporary endpoint insertion and whole-directory `setFromList` are runtime application behavior; temporary IDs are excluded from serialization. `facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256` confirms the temporary-id mapping.

## Existing-owner-first result

The pure payload responsibility is independently representable in the existing `telegram-protocol` owner and does not require a carrier scheduler. Platform spec-first commit `e4344c0c9b68efd54dc44c88f9da87b29acf8bbf` created `docs/specs/telegram-special-config-payload-boundary.md@7ed218ea6f52aac59f5461f8706c5c772d0e0f8b`. Production implementation commit `ba7ca87b27c3fbf7cb07ea5fa01b8d5caf67d1b4` added `special_config.rs@03ea4149703942befe59132a2019bef71e92938b` and exported it through `lib.rs@02853449d52bc713d6db826e88875b94c2a9c70f`. It reuses the crate's existing `aes`, `num-bigint-dig`, `sha2` and `subtle` dependencies and the existing `parse_help_config_simple`; no dependency or lock change was made.

Authored-only public contracts were added at platform commit `0ac3bdc2b3bcb47871970b654bc50dc9cefbf21d`, `protocol_contract.rs@cd51c5db0c2d92cd713eac5ddb80f589775ead43`. Desktop source/contract mirror commit `eba569bf912a89fd02f2731edb3da262f1be962d` has byte-identical special-config, lib and contract blobs. Internal authored-only contracts also cover the NIST AES-256-CBC vector, a synthetic recovered payload through the existing TL parser, expiry/rule/prefix semantics, endpoint bit preservation, no-match behavior and malformed input. None were executed.

The separate carrier/application responsibility remains mapped/pending with no production delta: fixed-U direct no-proxy DoH/Firestore request racing, HTTP Date global-time mutation, temporary-DC session/help.getConfig scheduling/cancel/timeout/kill, and runtime `DcOptions::setFromList` replacement require a shipping telegram-network/runtime bootstrap lifecycle that is absent at the reviewed checkpoints. No second bootstrap runtime/provider/DC owner was invented.

Ledger rows: `TDRP-R9-MTPROTO-SPECIAL-CONFIG-PAYLOAD-001 = implemented / rewritten / validation_status=not-run / accepted=false`; `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001 = mapped / pending / validation_status=not-run / accepted=false`. Coverage is `source_entries_total=16202, responsibilities_total=118, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; focused semantic review does not decrement global source-entry unknown/unread.



### Round 115 final source-level compliance

Platform-core final source-level compliance is `cbfb785ef23f33fbcebda0950b457da2f0578c66` / spec blob `0be79934e196946c3b0d11c232fbbd161d5aef94`. It references desktop initial bookkeeping commit `651c199fee8b3cad83e9f97b6fd92c59cd38f996` and records R1-R14 plus AC-1 through AC-8 as passed at source/implementation/bookkeeping review level. Executable validation remains intentionally `not-run`; no acceptance or release claim is made.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-PAYLOAD-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`. `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false` because the reviewed Fabushi network/runtime checkpoints have no shipping special-config HTTP acquisition / temporary-DC help.getConfig scheduler / DcOptions application lifecycle to extend without inventing a parallel bootstrap runtime.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A DoH / special-config response normalization boundary (round 116, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual fixed-U review of `mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9`, `mtproto_domain_resolver.h@cb1979bbbf0454135c8668ed1e5013da85d1dedc` and `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` identifies a carrier-independent response codec inside the broader pending special-config/domain-resolution carrier lifecycle.

`ParseDnsResponse` accepts Google and Mozilla/Cloudflare DoH JSON through the same `Answer` array contract: malformed top-level structure becomes empty, malformed entries are skipped, optional numeric DNS type filtering is applied, accepted `data` strings preserve source order, and missing/non-number TTL becomes zero. `DomainResolver::requestFinished` then computes an effective cache TTL with a 10-second per-entry floor and 300-second overall ceiling. The cache write/callback remain lifecycle behavior and were not moved into the codec.

For special config, Google/Mozilla responses reuse the same parser with type 16 (TXT). `ConcatenateDnsTxtFields` uses `QMultiMap<int, QString>` keyed by `INT_MAX - QString::size()`, so longer UTF-16 strings precede shorter ones and equal-length later insertions precede earlier ones; concatenated text is converted with source `QString::toLatin1` replacement semantics. Firestore projects `fields.data.stringValue` and applies the same Latin-1 projection. Fixed-U `RemoteConfig` and `Realtime` helper branches remain outside this responsibility because the reviewed constructor does not schedule those attempt types.

Existing-owner-first outcome: the carrier scheduler still belongs to existing `telegram-network`, but the deterministic response codec is implemented in existing `telegram-protocol`, which already owns pure codecs and already depends on `serde_json`; this avoids a new network dependency/lock delta and does not create a second carrier. Platform spec-first commit is `64bdbc8521a33b0c16d1ca51680c7ea22e482f2d` / initial spec blob `f39d0e7ec80b46944e3c50fd7dec78193ac6549d`. Platform implementation/authored-contract commit is `3d27d1aa1272bb15d39c1e2031872a1f2266c62e`: `response_normalization.rs@9371f73669b4e6b211608f043119ca4eca56e185`, `lib.rs@27543d45f3825321e26c8a0b664e37c8488169e2`, `protocol_contract.rs@259f457eeaacce0ed09f1acf1f0fd821a0269733`. Desktop mirror commit `4070ca2be5db0d22bd54c27857ccf84748ce8763` has the same three blob identities. Platform source-level compliance commit is `39db402654e3157d3e9b8694d9d91d89ea2c7f82` / spec blob `8379890d8ee17db235037c42cd9fde0cac00c014`; its bookkeeping AC remains pending until this desktop synchronization is recorded.

Ledger row `TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001` is `implemented / rewritten / validation_status=not-run / accepted=false`. `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`, narrowed to request construction/racing, HTTP Date mutation, temporary-DC lifecycle and dc-options application. Coverage becomes `source_entries_total=16202, responsibilities_total=119, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; focused semantic review does not decrement global source-entry unknown/unread.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 116 final source-level compliance

Platform-core final source-level compliance is `2585e75e6586480378f07d9daab39889e4111339` / spec blob `63f2b715e733cc1fe851b151fdf83355e3f2582b`. It references desktop initial bookkeeping commit `4f91cd629df720e79743885807c5da7db97042c8` and records R1-R11 plus AC-1 through AC-8 as passed at source/implementation/bookkeeping review level.

`TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`. The canonical and desktop response module/lib/contract blobs remain byte-identical at `9371f73669b4e6b211608f043119ca4eca56e185`, `27543d45f3825321e26c8a0b664e37c8488169e2`, and `259f457eeaacce0ed09f1acf1f0fd821a0269733`.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false` because request construction/racing, HTTP Date mutation, temporary-DC help.getConfig scheduling/cancel/timeout/kill and dc-options application still require a shipping existing network/runtime lifecycle. Pure response parsing/normalization is no longer part of that blocker.

Coverage remains `source_entries_total=16202, responsibilities_total=119, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


#### Round 116 HTTP Date parser refinement

Final fixed-U reread of `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` identified one additional pure response responsibility inside `handleHeaderUnixtime`: `ParseHttpDate` itself is deterministic parsing, while `base::unixtime::http_update` is the side effect that must remain in the pending carrier/runtime lifecycle.

The durable spec was amended first at platform commit `fd34ce2eff1cbc5e626aee71dd4f63415e09ff90` / spec blob `0b813e499c4dcb7566a5a0f17a228ec824489ca9`. Platform implementation commit `1d5d02d02abea178c9f91c37e41d24bee3171eca` adds `parse_http_date_unix_seconds` without a regex/time dependency: it reproduces the source weekday/date/month/year/time/GMT shape, source month set, civil-date/time validity and unanchored match behavior, then returns UTC Unix seconds without mutating a clock. New platform blobs are response module `8d2d70ec883ea8969be68b280e994e2bb35baf71`, lib `8ab2cd39719dbeb3725c1a52c8632b6f97cfdf0f`, authored contract `7e2536f7b4bcd13f895ef12993aea2ecebb74ef1`. Desktop mirror commit `bcb34c647362932be0c98e7d7caef918991bf11f` has the same three blob identities.

The same `TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001` row remains `implemented / rewritten / validation_status=not-run / accepted=false`; no second responsibility or owner was created. The carrier row remains pending only for actual requests/racing, global HTTP time mutation, temporary-DC lifecycle and dc-options application.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 116 final HTTP-Date-inclusive source compliance

Platform-core final source-level compliance is `58afebfe62e1c81d4f03d084fbafc4dc19f2a4c7` / spec blob `929cfccd65ec2b009a183ccb052c16817a8ca2a1`. It records R1-R12 plus AC-1 through AC-8 as passed at source/implementation/bookkeeping review level after adding pure `ParseHttpDate` equivalence.

Final canonical and desktop response module/lib/authored-contract blobs are byte-identical at `8d2d70ec883ea8969be68b280e994e2bb35baf71`, `8ab2cd39719dbeb3725c1a52c8632b6f97cfdf0f`, and `7e2536f7b4bcd13f895ef12993aea2ecebb74ef1`. `TDRP-R9-MTPROTO-RESPONSE-NORMALIZATION-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`: only actual direct request construction/racing, global HTTP-derived time mutation after the pure Date parse, temporary-DC help.getConfig scheduling/cancel/timeout/kill, and dc-options application remain blocked on a shipping existing network/runtime lifecycle.

Coverage remains `source_entries_total=16202, responsibilities_total=119, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 117 request-shaping normalization

Fixed-U reread of `mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9` and `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` separated deterministic carrier request shaping from the still-blocked scheduler lifecycle.

`TDRP-R9-MTPROTO-REQUEST-NORMALIZATION-001` is now `implemented / rewritten / validation_status=not-run / accepted=false`. The existing `telegram-network` owner now provides the exact fixed DNS camouflage-domain list, exact UA, source-equivalent 13..=128 rejection-sampled base62 random padding, Google/Mozilla A/AAAA and special-config ANY/TXT request components, and direct/camouflaged Firestore URL-host/path/Host components. It returns semantic request descriptors only and reuses the existing `getrandom` dependency; no manifest/lock delta, HTTP client, timer, proxy state, reply lifecycle or temporary-DC state was added.

Platform spec-first commit is `997c7b12cdc8b22368b6f1a04490ea7b602294bc` / spec blob `0983f2ae91648e1d8a48b553df6bd98c67dd3553`. Platform implementation/authored-contract commit is `9751072e735630accbbb921178c737b609dbb039`, followed by source-review contract-vector correction commit `a65278227c78fac5fd5a6ac41c841b4fc509fcb5`: network lib `3743331b0b1453048a98a21ae4ac2d2b91a2b2a5`, request-normalization module `2f3ea627054779296102f13c70be17a3655ddcb9`, authored contract `55d55d5de956cad3d809d64efc97993d270dcd97`. Desktop initial mirror commit `688ba8dff5fb415736c8cee9d397d97bb0fe1837`, followed by source-review contract-vector correction commit `712bde77f6f232efafe775ff9390faa47e9595fe`; final lib/module/contract blobs are byte-identical to platform at `3743331b0b1453048a98a21ae4ac2d2b91a2b2a5`, `2f3ea627054779296102f13c70be17a3655ddcb9`, and `55d55d5de956cad3d809d64efc97993d270dcd97`.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`, narrowed to actual direct no-proxy HTTP dispatch, 800 ms staggered racing/reply cancellation/finalization, global HTTP-derived time mutation, temporary-DC help.getConfig lifecycle, and dc_options replacement/change notification.

Coverage becomes `source_entries_total=16202, responsibilities_total=120, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; the two source blobs were already inventoried, so focused semantic reread does not change source-entry unknown/unread counts.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 117 final source-level compliance

Platform-core final source-level compliance is `445f49beed303bd7dbef2f8b998bfb22d8aaadce` / spec blob `94c6b75c471167009aacec4bdb3fd4d00cb2bb27`. It records R1-R13 plus AC-1 through AC-8 as passed at source/implementation/bookkeeping review level.

Final canonical and desktop telegram-network lib/request-normalization/authored-contract blobs are byte-identical at `3743331b0b1453048a98a21ae4ac2d2b91a2b2a5`, `2f3ea627054779296102f13c70be17a3655ddcb9`, and `55d55d5de956cad3d809d64efc97993d270dcd97`. `TDRP-R9-MTPROTO-REQUEST-NORMALIZATION-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`: remaining scope is actual direct no-proxy HTTP dispatch of normalized descriptors, 800 ms racing/reply cancellation/finalization, global HTTP-derived time mutation, temporary-DC help.getConfig lifecycle and dc_options application.

Coverage remains `source_entries_total=16202, responsibilities_total=120, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 118 direct HTTP carrier / staggered reply lifecycle owner audit

Fixed-U carrier reread covered `mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9`, `mtproto_domain_resolver.h@cb1979bbbf0454135c8668ed1e5013da85d1dedc`, `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88`, and `special_config_request.h@91901489e778089b03187682a9556e929867b94f`.

Both source owners use a dedicated `QNetworkAccessManager` with `QNetworkProxy::NoProxy` and an 800 ms stagger. The lifecycle is not one generic winner policy: DomainResolver binds delayed attempts to a per-key weak guard and erases that guard after the first non-empty parsed DNS result, while SpecialConfigRequest binds delays to the request object and does not internally stop remaining attempts after one successful response. `ServiceWebRequest::destroy()` performs disconnect+abort+deleteLater only when replacing an owned reply; ordinary destruction only schedules deleteLater, and each finished reply is read then removed even when it carries error metadata.

Existing-owner-first audit at platform-core found no reusable first-party HTTP carrier: `native/telegram-network/Cargo.toml@9ab75b9fa7d8a97f49ef4d627008727826337162` and its lock have no HTTP/Web-PKI/timer stack, `native/telegram-network/src/lib.rs@3743331b0b1453048a98a21ae4ac2d2b91a2b2a5` is synchronous TcpStream/MTProto plus pure normalization, and `native/telegram-runtime/Cargo.toml@a0d5aee63c886b11c3ad8fbf37885e0a3c5b6053` has no HTTP/timer owner. Vendored `third_party/mahayana/codex-rs/http-client` is explicitly Codex-owned and brings reqwest/rustls/tokio plus Codex proxy/logging policy, so it is rejected as an ownership and NoProxy-policy mismatch rather than reused.

Platform durable carrier-lifecycle spec is commit `2eade7264ce40c53e18eee75a21aa6c2a214ef05` / blob `91c92cadaf5df2db84eacb1111616af942851b67`. Because actual HTTPS dispatch would require a new native HTTP/TLS/task dependency graph and lock inputs, this round records a blocked/no-production-delta instead of adding an unreachable facade, thread-sleep surrogate, system curl subprocess, or fake executable contract.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`. Remaining scope is a first-party direct HTTPS carrier/lifecycle, then global HTTP-time mutation, temporary-DC help.getConfig, and dc_options application. Coverage remains `source_entries_total=16202, responsibilities_total=120, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A global HTTP-derived network time owner (round 88 / source evidence round 119, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source review split the clock mutation from the still-missing HTTP carrier. `special_config_request.cpp@fdb65506b0a886bc2daeccb61090bc44be7e2d88` exposes a successful parsed Date to `base::unixtime::http_update` before the time-only completion callback; pinned `Telegram/lib_base@95a543e426e35938dd632139d0bb828b268c3b4c/base/unixtime.cpp@b39b788a8a125d29d06f637c57c9b52ac24b138e` owns the process-global base/HTTP shifts and validity. `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087` suppresses duplicate time acquisition while valid/in-flight, while `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16` consumes `http_now()`. The separate `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194` temporary-DC `help.getConfig` / `dc_options` application lifecycle remains pending.

Existing-owner-first implementation is in canonical `bhrumom/fabushi-platform-core`: durable spec source-compliance commit `1d8c8bd07de0c9a2a970610c39830a77eeff150c`, final cross-repo evidence commit `39aa2275f68f168752d9cf24b52dbf56c3716d02` / final spec blob `cc27f7ae6eca445aa6902f0af48619da28ab83f8`; source correction commit `7c822678107d592c34380fe20683a2ecea66acd2`; `native/telegram-network/src/network_time.rs@19b77a6960d6059b7b708ceb4ec13643a1ec0944`; `native/telegram-network/src/lib.rs@e0dc33a728916fb7be916956f44cfb0b670486c6`. The owner uses thread-safe process-global atomics, preserves fixed-U non-forced MTProto global update semantics (<3-second candidates do not lock future updates; the first material update fixes the base), exposes carrier-independent HTTP update/valid/now/invalidate operations, and wires the real TLS camouflage handshake to HTTP-adjusted network time. Existing per-session message-id clock correction remains intact.

Desktop provider mirror correction commit `53e1a4b22e94a2d688d47e681ef41e6e4bf4247e` carries byte-identical network-time and network-lib blobs. Eight focused contracts are authored in the module and were not executed. No manifest/lock delta was made.

The unique ledger adds `TDRP-R9-MTPROTO-HTTP-DERIVED-NETWORK-TIME-001` as `implemented / validation_status=not-run / accepted=false`; responsibilities_total is now 121. The existing `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending`: direct NoProxy HTTPS/Web-PKI/800 ms reply lifecycle is still blocked on a first-party carrier, and temporary-DC `help.getConfig` / `dc_options` application is still not implemented. Coverage remains `source_entries_total=16202, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed. `baseline_ready=false`, `accepted=false`, and executable validation remains deferred.


## Stage-A special-config temporary-DC application owner audit (round 89 / source evidence round 120, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source review covered `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`, `mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a`, `facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256`, `mtp_instance.h@eeb56d29cc8316bd6600c26b6f8edf3a912601eb`, `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, and `mtproto_dc_key_creator.cpp@13c57454f14b814b5923ea253048fb881ad8cf03`.

The exact carrier-independent chain is now pinned: selected real DC d becomes transient routing/session id 1000+d; `constructAddOne` inserts a tcpo_only endpoint plus optional secret into the existing DC options map, while serialization/config enumeration skip temporary ids; `help.getConfig` is sent on the temporary shifted session; `terminateSpecialRequest` independently cancels the request and kills that session; the attempt timer is 6000 ms; successful `specialConfigLoaded` rejects empty dc_options and applies only `setFromList`; overwrite builds a fresh endpoint map and publishes per-DC change events only on differences.

Existing-owner-first audit found that `telegram-protocol::DcDirectory` already owns temporary protocol identity normalization, but current `telegram-runtime@10031c5261bb0810ad2df2fca3de2e2fe4f65301` owns only one synchronous `Option<EstablishedSession>` and one server directory. It has no shifted-session registry, separately cancellable request handle, total 6000 ms attempt owner, or endpoint-only replacement/change-publication lifecycle. Therefore `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`; adding a bootstrap-only parallel runtime or treating socket read timeout as source-equivalent cancellation was rejected.

One exact prerequisite exposed by this review was independently implementable: `establish_auth_key` passed the raw routing id to `p_q_inner_data_dc`. Canonical platform-core spec commit is `df6d6f913c9edd800487bd5e794a9b34faee8315`; implementation commit `1d4ce62cb6a298ddaaec7cc29d123edf16e8e955` changes `native/telegram-network/src/lib.rs` to blob `6023c96c4ee96a7cae37dbd6f9be077cf88779cb`, deriving `ProtocolDcRole::Regular` wire identity before auth-key inner-data emission. Four contracts for production/test real/temporary ids are authored but not run. Final platform source-compliance commit is `2a154c789b11f4826e27926489affdb5219d0401` / spec blob `a2fd80e0d1ced093ae872b1bd5ec4db33629eb6f`.

Desktop provider mirror commit `dc4e1e654ad2627f5f29554e6db1e3a2c3e20aa6` carries the byte-identical network lib blob `6023c96c4ee96a7cae37dbd6f9be077cf88779cb`. The existing `TDRP-R9-MTPROTO-PROTOCOL-DC-ID-WIRE-001` row is extended with this shipping auth-handshake consumer; responsibilities_total remains 121. Coverage remains `source_entries_total=16202, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

The direct NoProxy HTTPS/Web-PKI/800 ms carrier remains separately blocked. No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A canonical MTProto multi-session owner (round 90 / source evidence round 121, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread covered `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`, `mtp_instance.h@eeb56d29cc8316bd6600c26b6f8edf3a912601eb`, `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, `session_private.h@62b01e38dcb21832125b67dcd35fba3194a26829`, `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`, and `mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a`.

The general fixed-U lifecycle is now separated from special-config policy: `_sessions` is the shared ShiftedDcId session map; `getSession/startSession` own generic creation/reuse; `getDcById` maps temporary logical 1000+d back to real Dcenter/auth identity d; `cancel(requestId)` clears request ownership separately from `killSession/scheduleSessionDestroy`; and ordinary `ConfigLoader::enumerate` also creates config-shifted sessions from sorted non-CDN `configEnumDcIds()`. This proves a canonical source-neutral multi-session owner is useful without the HTTP carrier and must not be special-config-only.

Spec-first platform commit `bff221c3fe8ab597c3b26e081d620b450ac0abc0` adds `docs/specs/telegram-runtime-multi-session-owner.md@30c250a3ce6aab3765db2491d76beacde33562fb`. Implementation commit `8c57554ade7ad4bc75e3692a0e5dc57099162206` changes only `native/telegram-runtime/src/runtime.rs` to blob `d531db9176a09e973690b3a44da52208e09e33b5`: RuntimeClient now owns one SessionOwner instead of a standalone Option<EstablishedSession>; SessionRoute records logical DC separately from canonical auth DC; existing shipping `take_transport/put_transport` and therefore all current RPC consumers use the registry primary slot. Logical 2 and temporary 1002 both derive auth DC 2. No manifest, lock, storage schema, public ABI, HTTP carrier, request scheduler or second runtime changed.

Desktop mirror commit `8f04c5f35cc6674b33f27edb09fe66fb1db7d6f0` carries the byte-identical runtime blob at the shipping provider path. Two focused internal contracts are authored but not run. The unique ledger adds `TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001 = implemented / rewritten / validation_status=not-run / accepted=false`; responsibilities_total becomes 122 while source_entries_total=16202, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains mapped/pending/not-run/accepted=false. The newly implemented owner does not yet create/resume an actual secondary EstablishedSession against a transient endpoint, expose independently cancellable in-flight request ownership or the 6000 ms total-attempt deadline, or perform dc_options-only replacement/change publication. The direct NoProxy HTTPS/Web-PKI carrier remains separately blocked.

Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


### Round 121 final source-level compliance

Platform-core final source-level compliance is `e1d158cc9ba3457da6bc565fdb4b8353df28c0d1`; the canonical multi-session spec blob is `901d5e07c92a538870e6cf87343bab13a58f2558`, and the narrowed special-config temporary-DC boundary spec blob is `33b009b272822bbc4f8a77e027b6cecc8574cbc7`.

The canonical/runtime and desktop shipping provider runtime blobs remain byte-identical at `d531db9176a09e973690b3a44da52208e09e33b5`. `TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001` remains implemented / rewritten / validation_status=not-run / accepted=false. `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains mapped / pending / validation_status=not-run / accepted=false with its blocker narrowed to actual secondary endpoint session/auth material, independently cancellable request + 6000 ms total-attempt lifecycle, dc_options-only application, and the separately blocked direct HTTP carrier.

No GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A per-real-DC auth material and secondary EstablishedSession (round 91 / source evidence round 122, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread added `details/mtproto_dcenter.h@651dcf171544745efc0b5d53afd8af4a57fa5a88`, `details/mtproto_dcenter.cpp@ef0df612db798338d166f47087bb35c1d29ff72b`, `session.h@d6f197d233fa4dcc6d2b19745e6d2787c5227911`, and `session.cpp@40d7048c3057c7c5523b786048670524c8bebe4c`, together with the already pinned `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` and `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`.

The fixed-U key ownership boundary is explicit: `getDcById` maps temporary logical 1000+d to real Dcenter d; SessionData/Session delegate persistent/temporary key access and key-creation arbitration to that Dcenter; SessionPrivate first reuses the Dcenter key and only creates when absent. Dcenter lifetime is broader than one shifted Session, so killing a logical session does not imply discarding real-DC auth material.

Existing-owner-first implementation stays in the single canonical RuntimeClient. Platform spec addendum commit `2e395fbd90447b3df1c1f30a128bcdc1a4c7d11e` was followed by implementation commits `87fb77a6b7220e88f3d376b7543934ddd0644fc5` and `292f41096e554872ee6b85d43e519a9237dcb26f`; final source-compliance is `dbde13a6d2be2dcf9ab5f170507f10e1a69fe301` / spec blob `085f6d82d9c351614ffab83baa8e6af4da84d172`. Runtime blob `4983b262346b73c3dc7270bf5feac31780c28496` adds SessionOwner.auth_materials keyed by real/auth DC, seeds it from unchanged PersistedTransportState, rejects production/test environment mismatch, retains material across logical-session removal, and introduces generic `open_logical_session`. That production path reuses an existing logical EstablishedSession, otherwise calls existing `resume_auth_key` with shared real-DC material, or existing `establish_auth_key` when material is absent. The existing primary restore path now consumes the same generic opener.

Desktop mirror commit `8cebdbe5634d092773cfda8b4a0b059bd552ccbb` carries the byte-identical runtime blob `4983b262346b73c3dc7270bf5feac31780c28496`. Three focused contracts are authored but not run: persisted primary DC 2 shares auth material with temporary logical 1002, temporary logical removal retains real-DC material, and test/prod mismatch fails closed. No public ABI, manifest, lock, storage schema, HTTP carrier, timer/task runtime, second RuntimeClient, provider, account, directory, or persistence namespace was added.

`TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`, now including actual per-real-DC auth-material reuse/creation and generic secondary EstablishedSession open/reinsert capability. `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`: remaining scope is independently cancellable request ownership, fixed 6000 ms total-attempt lifecycle, a wired bare help.getConfig temporary-session attempt, dc_options-only replacement/change publication, and the separately blocked direct NoProxy HTTPS/Web-PKI carrier.

Coverage remains `source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; the reread source blobs were already inventoried and this round extends existing responsibilities rather than adding another row.

Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A independently addressable request ownership and 6000 ms total-attempt teardown (task round 92 / source evidence round 123, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread pins `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`, `config_loader.h@13a80b1b049f28866c3db110df5c662e8e74dede`, and `session.cpp@40d7048c3057c7c5523b786048670524c8bebe4c`. Fixed-U `sendSpecialRequest()` terminates the prior request/session, stores an independent request id plus temporary logical DC, then starts `kSpecialRequestTimeoutMs = 6000`; `terminateSpecialRequest()` independently cancels the request and kills the temporary session. `Session::cancel` removes request ownership independently of session destruction.

Spec-first platform commit is `89805c2b3f24fc187dac975b67e222b0ab44dee1`. Canonical source-level implementation/fixes are `dbf24c6113fbe681e9e3fdcd912e14351e1bc588`, `f8acfdde4020ee6d27d2a9a7c118ba759802dfd7`, `3881f5f022942d3cbef9aa1caed51ebc518960f6`, `3e5b4f9686e1182b85a809bb4224b66c684de614`, and `1fc3e07d4b1e0cc88c5df148de03909dd6b8feeb`; final source-compliance is `61651c3843a93633e101f38901c3e8b14ef27fbd` / spec blob `a190c2cf7bf305cdf21508a36752d8fd65fdb35d`. Canonical telegram-network/runtime blobs are `f2b364a4344f07315c6581a3052a134319aa210d` and `6c3688025bdcfd5bbcc58e9d4e0095ae41baca4c`.

Existing-owner-first implementation remains inside the single RuntimeClient/SessionOwner. SessionOwner now owns owner-local request ids and logical-DC-scoped OwnedRequest values; NetworkAttemptControl is only the lower transport primitive. A controlled attempt uses one absolute deadline through connection, TLS camouflage blocking I/O, auth-key establish/resume, and synchronous `invoke_raw`; a cloned control can mark cancellation and shut down the registered socket. The special-config-derived policy constant is exactly 6000 ms. The owned invoke derives logical identity from request ownership, fences session reinsertion on request survival, always finishes successful transient attempts and cancels failed/open-failed attempts, removes the temporary logical session, and retains shared real-DC AuthMaterial.

Desktop source mirrors are `a71764d91662574851b82ba7f38a24484d68c21b` followed by final runtime mirror `e472b4c9e90ea3672d432db5583bab7bf7636e97`; desktop telegram-network/runtime blobs are byte-identical to canonical at `f2b364a4344f07315c6581a3052a134319aa210d` and `6c3688025bdcfd5bbcc58e9d4e0095ae41baca4c`.

Two network contracts and four runtime contracts were authored but not run. `TDRP-R9-MTPROTO-MULTI-SESSION-OWNER-001` remains `implemented / rewritten / validation_status=not-run / accepted=false`. `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false`: request ownership/6000 ms teardown is no longer a blocker, while the wired bare help.getConfig special-endpoint orchestrator, dc_options-only replacement/change publication, and separately blocked direct NoProxy HTTPS/Web-PKI endpoint-acquisition carrier remain open.

Coverage remains `source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; this round extends existing rows and rereads already inventoried source blobs.

Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A resolved special endpoint -> bare help.getConfig -> dc_options-only publication (task round 93 / source evidence round 124, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread pins `scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`, `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`, `facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256`, `mtproto_dc_options.cpp@d8499fc1df2ed7fdb88da687a04b824d243ff49a`, and `mtproto_dc_options.h@4e0b979c4a84661f3365196ab8702dc8cb60fd03`. Fixed-U declares bare `help.getConfig#c4f9186b = Config`; ConfigLoader maps a resolved real DC to temporary logical `1000+d`, adds a transient `tcpo_only` endpoint with optional secret, sends bare help.getConfig with the existing 6000 ms lifecycle, and applies only non-empty `dc_options`. DcOptions overwrite normalizes duplicate same dc/IP/port entries, compares per-DC IP+port count/membership only, replaces storage only when that difference is non-empty, and publishes each changed DC id.

Spec-first platform history starts at `0198d87b8ced0d811e11499e1bf2d22c6c408339`; implementation and source-review fixes culminate at `000d8cef06238e49e048028cb0e298a17996ffed`; final source-compliance is `afc4379871a397bf4180fb74dd1f7729fbb86341` / spec blob `4109c09f3f92932ac79aa1f6d15d1825a6788359`. Canonical protocol api/dc/contracts/runtime blobs are `e032f4dbba09c0a4d912566b4e1c51311b677002`, `f9aecd0370b43317f81b437ee308fb88d141a302`, `91a74d91214a06fcef5fcb265204f4deec82c30a`, and `b502e32e821433a14cc47861e17c10e106fd5af8`.

Existing-owner-first production wiring remains inside telegram-protocol DcDirectory and the single RuntimeClient/SessionOwner. Protocol now exposes exact bare `build_help_get_config`, a `parse_config_dc_options_prefix` that does not require response `this_dc` to be routable, and fixed-U-equivalent `DcDirectory::replace_endpoints`. Runtime exposes `apply_resolved_special_config_endpoint`: it derives temporary `1000+d`, constructs one transient tcpo_only endpoint preserving the secret, begins the existing exact 6000 ms owned attempt before controlled open, invokes the bare request, treats empty dc_options as a successful no-op, applies non-empty options only to canonical `server_directory`, and returns changed real DC ids. Temporary request/session state is torn down while per-real-DC AuthMaterial remains reusable.

Desktop mirror commit `820418b875f227755f6060a5d531dc1596e5c0a6` carries byte-identical protocol/runtime blobs. Focused protocol and runtime contracts were authored for exact bare bytes, response-this-dc independence, empty options, dc/IP/port dedupe, IP+port-only change publication, secret-only no-op, 2->1002 mapping, transient tcpo_only endpoint/secret preservation, and dc-options-only state mutation; none were run.

`TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains `mapped / pending / validation_status=not-run / accepted=false` because the composite responsibility still lacks the separately blocked first-party direct NoProxy HTTPS/Web-PKI endpoint-acquisition carrier and Stage-B executable evidence. The prior shipping bare-help.getConfig orchestrator and dc_options-only publication blockers are closed. Coverage remains `source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.

## Round 125 / task round 94 — direct HTTPS/Web-PKI carrier lifecycle

- Fixed U remains f985723b552b30fe1c17d377d57f44e61a15a8c4.
- Existing-owner-first direct carrier implemented in platform-core native/telegram-network; final platform source-compliance head 5dd792399fe3c17d1256c80e0eb9fb2c1c2aa94f.
- Desktop carrier mirror commit 623936831ac1831022a35b22471a58bd96036104; direct_https.rs and lib.rs blobs are byte-identical to platform.
- Exact fixed-U NoProxy, 800 ms stagger, DomainResolver winner fencing, SpecialConfig continued stagger, and ServiceWebRequest explicit-abort-vs-ordinary-finalization semantics are recorded in round125 evidence.
- TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001 remains mapped/pending, validation_status=not-run, accepted=false. Carrier lifecycle is implemented; automatic acquisition-to-RuntimeClient composition and Stage-B executable evidence remain pending.
- Stage A zero-CI discipline preserved: tests/contracts authored but not run; no executable validation or Actions/status/check query performed.

## Round 126 / task round 95 — special-config acquisition → RuntimeClient composition

- Fixed U remains f985723b552b30fe1c17d377d57f44e61a15a8c4.
- Fixed-U SpecialConfigRequest/ConfigLoader was reread for Google/Mozilla/Firestore group-local shuffle, Date-before-finalize body adaptation, decode/select callbacks, pending/tried dedupe, randomized endpoint dequeue and temporary help.getConfig application.
- Platform final source-compliance head is 0b24613beea74301bf974dad4e10c56988096c40. Key blobs: special_config_acquisition.rs 9c8ef0ee0176a78eaa98266b5f92db1757b5db7c; telegram-network lib.rs b073f01ab05e692b46afa4fc931cd3e8dd0893cf; telegram-runtime runtime.rs df8ed7c3cebed632f6591c5809c28753ede9bba0; runtime lib.rs 9acba52c16de3085a3dbfa61dc3e9f6ba5cdb689; composition spec 62ec9f061745b1388728c2d60dc061ebaf86e920.
- Desktop final production mirror before bookkeeping is 0c81381ed8b5d3862c9ced13bc7e2320b0e5d510 and the mirrored production/spec blobs are byte-identical.
- Existing-owner-first production path is now request seeding/shuffle -> request_normalization -> DirectHttpsRace(SpecialConfig) -> successful Date/time adapter -> response_normalization -> special-config decode/select -> pending/tried randomized endpoint queue -> existing RuntimeClient apply_resolved_special_config_endpoint. Queue draining begins as decoded endpoints arrive while later HTTP stagger attempts remain independently live.
- TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001 advances to implementation_status=implemented / validation_status=not-run / accepted=false. Stage-B executable evidence remains pending; this is not verified or accepted.
- Contracts were authored but not run. Stage A zero-CI discipline was preserved: no Actions/status/check query, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
- Coverage counters remain source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0.



## Stage-A proxy-enabled special-config suppression (task round 96 / source evidence round 127, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread pins `config_loader.cpp@5d07404ba07223016c6728b986dcba10628d7194`, `config_loader.h@13a80b1b049f28866c3db110df5c662e8e74dede`, `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`, `core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99` and `core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26`. Fixed-U `SettingsProxy::isEnabled()` is exactly Enabled mode; its connection-type stream projects that boolean into ConfigLoader. Enabled causes `refreshSpecialLoader()` to reset the special loader and `sendSpecialRequest()` to terminate/refuse special endpoint work. System/Disabled do not assert this gate.

Existing-owner-first implementation remains in the single RuntimeClient and does not create a proxy settings store. Platform spec-first commit is `cb9b53d2574c012fe90e03df81b8b1336859ee2b`; implementation is `1c315c155eb0ba0c49871c08547bb7e67977aa87`; final source-compliance is `ef4edd0df2a9b74e3ac2cc3e088493e08ef75432`. Canonical runtime/spec blobs are `871ebe40ecbd284828b0c16ab64547eefeb3c3a3` and `eeacc5daa397a487a4f9c6ffcb4f18005fbdeda9`. Desktop production mirror commit `c7fa1f1b373d34d07eedc1aa4b407bd032b0070d` carries those byte-identical blobs.

`telegram.refreshSpecialConfig` now requires an explicit boolean `proxyEnabled` projection. When true it returns a `suppressedByProxy=true` zero-work report before phone resolution, attempt seeding, request normalization, DirectHttpsRace construction, special-config decode/application, or temporary MTProto request/session creation. The public Rust `refresh_special_config` function carries the same explicit projection. RuntimeClient persists no durable proxy mode/list/selection.

Three focused RuntimeClient contracts were authored for direct suppression, strict boolean parsing, and zero-work command suppression; none were run. `TDRP-R9-MTPROTO-SPECIAL-CONFIG-CARRIER-APPLICATION-001` remains implemented / validation_status=not-run / accepted=false. `TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains mapped/pending overall: its special-config suppression consumer is implemented, but the canonical product Settings/application owner that must publish authoritative proxy state and the remaining selected-proxy/custom-resolution consumers are still absent. Coverage counters remain `source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`.

Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A canonical product Settings owner -> proxyEnabled publication (task round 97 / source evidence round 128, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread pins `core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26`, `core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99`, `application.cpp@7172206f5c1e540e19c90798e9a9e5721276cc53` and `boxes/connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2`. `SettingsProxy::setSettings` only mutates mode; `connectionTypeChangesNotify` only publishes the connection-type event. `Application::setCurrentProxy` is the fixed-U authoritative mutation/publication chain: selected + mode update -> global proxy refresh -> proxyChanges -> connectionTypeChanges -> rotation notification. The connection box edits that one application Settings object and persists it; it is not a second owner.

Existing-owner-first re-review at desktop@36bbe14d8f0fd7cb6008149e5fe9931f9e06891a and platform-core@ef4edd0df2a9b74e3ac2cc3e088493e08ef75432 again found no existing typed canonical product Settings owner. AppHost `feature.persistence.*` is backed by `ui_state`; telegram-storage is provider-private auth/transport state; account-service is account/session state; telegram-network/runtime are consumers. The tracked frontend checkpoint has no shipping Settings implementation to absorb durable state. Under the explicit fallback rule, platform-core spec-first commit `206fa124fec1512bba4fca7f19602a1b691732b2` establishes the minimal source-neutral owner; final source-compliance is `8064784fcbc22b54d5aeb47f186e9e89f7fbaef2`, owner-spec blob `dd97a47106dda5d5a82d5af691fd102dc7fa4034`, proxy-state spec blob `4099833bd7899e3e68ab0848bf7087658bab149f`.

Desktop production commit `b1b5dd027497c39a355e9e64e64de0b6c190fd66` composes one `ProductSettingsOwner` into the existing AppHost (blob `5b74723e12884ea218e1f9de634b88d4357b2099`). It owns versioned source-neutral `network.proxyEnabled`, persists `feature-host/product-settings.json` separately from `ui_state` and Telegram storage, fails closed on malformed/unsupported state, and persists before downstream publication. `settings.snapshot` and `settings.setNetworkProxyEnabled` are source-neutral typed host methods. The latter republishes the authoritative boolean on same-value retry.

The same production commit extends the existing `TelegramClientLease` (blob `e4410652fe500144d93ff227b10d149d2c0fc2c7`) with a narrow `refresh_special_config` adapter that forwards exactly `{"@type":"telegram.refreshSpecialConfig","proxyEnabled":<bool>}` to the already-existing RuntimeClient. No RuntimeClient proxy setting, Telegram-specific Settings store, second provider/runtime or second Settings root was created.

Contracts were authored for ProductSettingsOwner persistence/reopen, malformed-state fail-closed, source-neutral command naming and exact boolean refresh request shape; none were run. `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` remains mapped/pending because full fixed-U proxy mode/list/selected/rotation/migration/UI semantics are still open. `TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains mapped/pending because selected-proxy/custom-resolution consumers are still open; its canonical proxyEnabled publisher blocker is closed. Both remain `validation_status=not-run / accepted=false`.

Coverage remains `source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Stage-A typed product proxy Settings state + v1 migration (task round 98 / source evidence round 129, 2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread pins `core_settings_proxy.h@1f2ec5c516e03ae49e80c15bb6083bffa7ec6d99`, `core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26`, `core_settings.cpp@6ef88cd5f0b946b0b92512d1a94e42a0f5c1fabb`, `boxes/connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2`, and `mtproto_proxy_data.h@d8deebac2359936f72901ba0eec0cbfdfb186e90`.

Fixed-U `SettingsProxy::serialize/setFromSerialized` owns mode, selected/list durable proxy records, tryIPv6, useProxyForCalls, IP-warning state, rotation enabled/timeout/preferred indices, including append-compatible tail reads, None/unknown list-entry skipping and preferred-index remapping. `Core::Settings::serialize` embeds that proxy byte array and restores it only after the outer stream is valid. `ProxiesBoxController` binds directly to the one application SettingsProxy; its delayed save timer calls `Local::writeSettings`, and list/mode/rotation edits mutate that same owner rather than a second list store.

Platform spec-first head is `c90e9150914f741b2ec3fbf81011da4005688d61` with owner-spec blob `1a582fc7adc6682364c6857219dee79c6204d50f` and proxy-state spec blob `ee9bc5e4666ee48b9aaa74162823a89f418e4306`.

Desktop production commit `b6f491d2364684111ebfc8ddbdae67ca3adaff6b` upgrades the same AppHost `ProductSettingsOwner` to version 2 (AppHost blob `3078e50bbfa0583383b3449818a79ba8465b064a`). Canonical `network.proxy` now stores typed System/Enabled/Disabled mode, optional selected endpoint, ordered endpoint list, tryIpv6, useProxyForCalls, checkIpWarningShown and rotation { enabled, timeoutSeconds, preferredIndices }. Endpoint persistence contains only fixed-U durable type/host/port/user/password fields; resolved IPs, expiry, sockets, checkers and timers remain outside Settings.

The owner implements fixed-U mutation invariants: whole-list replacement clears preferred indices; insert clamps and shifts preferred indices; remove drops/remaps preferred indices; non-positive timeout normalizes to 10; preferred indices remove negative/out-of-range/duplicates preserving first order; valid preferred promotion is front/idempotent. `setSelectedAndMode` provides one atomic owner mutation. Existing Fabushi v1 `network.proxyEnabled` migrates in place to v2 (true -> Enabled, false -> System) with fixed-U defaults for new fields. `proxyEnabled` is no longer stored; it is derived as `mode == Enabled` and remains the only projection sent through the existing `TelegramClientLease -> telegram.refreshSpecialConfig` boundary. No selected endpoint is prematurely connected to a carrier.

Historical raw TDesktop `Core::Settings` byte import is not guessed in this round: there is no shipping Fabushi import source/version boundary that can safely extract the nested proxy QByteArray. The fixed-U legacy decoder remains the oracle for a future explicit import adapter, which must feed ProductSettingsOwner instead of becoming a store.

Contracts for defaults, v1->v2 migration, typed persistence/reopen, list/preferred mutation invariants, timeout/promotion behavior, malformed fail-closed and source-neutral methods were authored in the AppHost source but not run. `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` advances to implemented / validation_status=not-run / accepted=false for the canonical state/persistence responsibility; unified Settings UI projection and Stage-B executable evidence remain open. `TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains mapped/pending because selected-proxy/custom-resolution carrier consumers are intentionally unwired.

Coverage remains `source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Stage A remained execution-free: no GitHub Actions/status/check query or trigger, build, typecheck, lint, validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


## Round 99 — unified Settings network-proxy projection/edit surface (source evidence round 130, 2026-11-01)

Shipping main `9993ca8e107f7e019a8201e5ab74fa54e82159f2` was re-read for ProductShell/Settings/IPC ownership before implementation. ProductionRenderer routes the primary Settings entry to the existing SettingsDesktopSurface; GeneralSettingsPanel uses existing SettingsGroup and Sand primitives. Renderer/Main IPC is constrained by the shared MAIN_METHOD_TABLE and app-window top-frame trust.

The same ProductSettingsOwner v2 is projected into that unified Settings surface through `getNetworkProxySettings`/`mutateNetworkProxySettings`, with MainData forwarding only `settings.snapshot`/`settings.mutateNetworkProxy`. The UI edits mode, selected/list, tryIpv6, useProxyForCalls, checkIpWarningShown and rotation preferences/timeouts. No Telegram-specific store/UI/runtime was introduced; selected proxy state is not connected to a network carrier.

UI/IPC/contracts were authored but not executed. Stage A remains `validation_status=not-run / accepted=false`. `TDRP-R9-PROXY-SETTINGS-STATE-PERSISTENCE-001` is implemented for canonical state/persistence plus unified UI projection; Stage-B executable evidence remains outstanding. `TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains mapped/pending for selected-proxy/custom-resolution carrier consumption.


### Round 99 shipping-tree integration identity

The checkpoint tree inherited a sparse-tree history that no longer contained the complete current desktop renderer/Electron dependency graph. The round-99 production integration commit `a9cece6754710ab4f91dd9089e3d4eee9ffb4e57` repairs that composition non-destructively: it uses current shipping main `9993ca8e107f7e019a8201e5ab74fa54e82159f2` as the complete tree base, overlays every blob present in checkpoint `695758b14d101e850651e1389861088e16c55f92` plus the network-proxy Settings changes, and records both histories as parents. This preserves checkpoint migration work while restoring the current shipping dependencies required by `ProductionRenderer -> SettingsDesktopSurface -> Main RPC -> Coordinator MainData -> AppHost`. No executable validation was run.


## Task round 100 / source evidence round 131 — selected proxy MTProto carrier consumer (2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Source reread pins the selected-proxy publication and consumption chain across `application.cpp@7172206f5c1e540e19c90798e9a9e5721276cc53`, `main_account.cpp@2721c7ac12effe454d398610d26d638fadaa1f68`, `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`, `session.cpp@40d7048c3057c7c5523b786048670524c8bebe4c`, `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`, `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a`, `connection_resolving.cpp@d319e632655bbfe28956c8ced981a5cdf0299d1d`, `mtproto_domain_resolver.cpp@eeac8f6a8e2df67967ea7b536fb165e2f8d983e9` and `mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`.

Existing-owner-first implementation is present. Platform-core final source-compliance head is `fb44f2b7170558560833a4f2e6cea0533b8e8d56`; desktop production composition is `2f0798e8709b3a3ea6b200f092060db6c1dccb62`. ProductSettingsOwner v2 remains the only durable owner. Runtime/network receive only a transient carrier projection. Enabled MTProto selection uses the existing obfuscated/TLS carrier; hostnames use direct NoProxy DoH with bounded TTL and transient successful-IP promotion. Projection changes cancel owned requests/drop live sessions while preserving reusable auth material. Enabled SOCKS5/HTTP/Web fail closed and remain separate carrier responsibilities.

`TDRP-R9-PROXY-MTPROTO-CONSUMER-001` is now `implemented / validation_status=not-run / accepted=false`. Contracts are written but not run. No build/typecheck/lint/validator/generator/dependency resolution/unit/contract/integration/E2E/package/deploy/install or GitHub Actions/check/status operation was performed in this round.


Round-100 manual source review correction: desktop `2f0798e8709b3a3ea6b200f092060db6c1dccb62` / AppHost `1fe55278a834b828b6ffdd8e2ca933fd2eaa3ef9` narrows startup restore to `apply_network_proxy` only. This avoids requiring authorization phone context through `refreshSpecialConfig(false)` during AppHost construction. Platform source-compliance was correspondingly updated at `fb44f2b7170558560833a4f2e6cea0533b8e8d56` / spec blob `d95fcbf209c2c00b75918aec3de3eb18921d57c0`. No executable validation accompanied this correction.


## Task round 101 / source evidence round 132 — selected WebProxy relay stream/window/bootstrap carrier (2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread pins `web_proxy_frame.h@a5ba491fed67d70aee88e6ec2e3ac47c991e78f3`, `web_proxy_frame.cpp@bd1d7b310f40203f9da914efc8bdd6711035884b`, `web_proxy_transport.h@f58850d2bc2ceabd93113d7725903e1c0d92d71b`, `web_proxy_transport.cpp@e25ad970e84686e9aca061d02c274048793b5ad4`, `web_proxy_webview.cpp@8a2efc8fb15e400366f199cd956effc8ad362da8`, `details/mtproto_web_proxy_socket.cpp@9aa05e7034031310de4aa9d410fbb15d21c63ee8`, `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381` and `mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`.

The prior “no selectable Web carrier call path” blocker is closed by the existing source-neutral ProductSettingsOwner v2 -> TelegramClientLease -> RuntimeClient -> telegram-network projection. Spec-first platform commit `c9aafb3332bea046bc73de00e781213ddd064c1b` / spec blob `76d0fe82f2d7da37326ad9f350674264af5f8576` reauthorizes production implementation in existing owners only. Platform final lifecycle head is `9a8e1c552e3ca9a611e5e0e7b845b845b22854ac`: network lib `450d6754908647b3c0e039ce95da94bb1a2fdf8f`, relay owner `8f6297bace3318c099588e0d6549b65fdeeb3fd2`, runtime `3a3da0eb4bf004569a3989e05e55c8b86a199c41`.

The existing telegram-network owner now carries exact 8-byte relay framing, 24-bit stream ids, 1 MiB frame limit, 4096-frame batch limit, incremental parsing, 4 MiB per-stream send/receive windows, 64 KiB Data chunks, consumer-driven Window return, 256 KiB/20 ms Window coalescing, 4096 closed-stream tombstones, Welcome/Open gating, Ping/Pong, carrier generation fencing and fail-closed teardown. `TransportSocket::Web` supplies the logical byte stream beneath the existing MTProto obfuscation/framing and reuses `NetworkAttemptControl` cancellation. Web selection does not fall back to direct TCP.

The same owner implements an external-browser fallback using a loopback-only one-shot capability, strict Host/Origin WebSocket adoption, hidden remote bridge iframe + MessageChannel, Hello(0,[1]) / Welcome gating, bounded bootstrap deadlines and secret-free runtime projection. Desktop production `8f5e43b02c5cbd81f55f04f218ac7909763fdb6a` plus lifecycle correction `324add75e62fef572a68b038ac7d3a5a04a93451` mirrors the provider and keeps MainEdge as the only platform browser-opening adapter. MainEdge blob `d9ffb976fcdfb7b6741e47e237f6e11e7110628d` accepts only strict `http://127.0.0.1:<port>/#<capability>` bootstrap URLs before calling the existing system-browser shell.

Manual source review before bookkeeping corrected four lifecycle issues without executable validation: incomplete relay frames now accumulate across browser binary messages; outbound WebSocket writes are serialized; relay registry/listener lifetime is explicit rather than accidentally held by blocking accept; carrier replacement and same-endpoint disabled->enabled/IPv6-only projection changes are generation-fenced so a newly prepared relay is not retired as the old carrier.

Contracts were authored in relay/runtime/MainEdge but were not run. `TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001` advances to `implemented / validation_status=not-run / accepted=false`. The adjacent process-restart handoff remains explicit: AppHost startup restoration prepares the Web relay but does not yet hand its one-shot bootstrap URL to the existing Electron shell for automatic browser reopening. Stage B executable validation, SOCKS5/HTTP carriers and the broader fixed-U migration remain open.

Stage A remained execution-free: no GitHub Actions/check/status/workflow query or trigger, build, typecheck, lint, validator, generator, dependency resolution, test execution, package, deployment, installation or runtime acceptance was performed.

## Task round 102 / source evidence round 133 — restored WebProxy restart re-adoption (2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Desktop started at `142d8768ca41fef10f8f4e980fbc78bc4df3635c`; platform-core was reread at `9a8e1c552e3ca9a611e5e0e7b845b845b22854ac` and required no production delta because the existing RuntimeClient/telegram-network projection already returns the fresh one-shot `webProxyBootstrapUrl`.

Existing-owner-first desktop production commit `57b1699fd047648fef1a8fb4daee81e6527e4836` closes the explicit round-132 restart handoff gap. Restored ProductSettingsOwner v2 state is still applied through the single TelegramClientLease -> RuntimeClient -> telegram-network owner chain. AppHost now retains only the transient returned bootstrap URL as a one-shot source-neutral `native-shell/web-proxy-bootstrap` event. UnifiedAppHost and the existing mahayana-app-host-desktop unsolicited stdout event transport carry it without a new runtime/provider/settings owner. Coordinator routes this Host-local event to existing MainData, retaining at most the latest undelivered event until MainData Ready and bypassing SSE-live suppression only for this non-SSE event family. Electron MainData now consumes server-direction events; the production composition accepts only the exact native-shell Web-bootstrap event and reuses the same MainEdge strict `http://127.0.0.1:<port>/#<capability>` parser/system-browser shell boundary used by mutation-time adoption.

Production blobs: AppHost `0c055afd43a292c22a647108a126876d50917814`; UnifiedAppHost `4f0b2d72fc07817de401bb5d378bf5312554e731`; desktop Host `9b6b359c5ca35e9e4ccd66e0487ec741b7a662dc`; Coordinator `279dc93569effbba8f49918d9293325dbb0c4ac8`; MainData client `6c581e0d6c0c644e186190b45e191725295c1a6e`; MainData legs `b32b8efcbe3c9477cb40f6cfbbf790a3f33b57be`; MainEdge `e4f060662317644ddaad8632d5745f3ad2b42f5b`; main production composition `f9b474db6c062f631c8bda693cbf45ee4930c454`.

Restart/re-adoption contracts were authored in AppHost and `source/electron-main/network-proxy-settings.contract.test.ts@b1ea8d2aef7fca34fc5f1d7385d94262a03394f0` but were not executed. `TDRP-R9-MTPROTO-WEB-PROXY-RELAY-STREAM-001` remains `implemented / validation_status=not-run / accepted=false`. The restart browser-reopening blocker is closed; Stage B executable validation, SOCKS5/HTTP carriers and broader fixed-U migration remain open.

Stage A remained execution-free: no GitHub Actions/check/status/workflow query or trigger, build, typecheck, lint, validator, generator, dependency resolution, test execution, package, deployment, installation or runtime acceptance was performed.


## Task round 103 / source evidence round 134 — SOCKS5/HTTP selectable proxy carrier closure (2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread pins `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`, `mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`, `connection_resolving.cpp@d319e632655bbfe28956c8ced981a5cdf0299d1d`, `details/mtproto_abstract_socket.cpp@653033365b543e554104c99a963ee194ddb830d9`, and `core_settings_proxy.cpp@18e7ab8ce60865409dbaff267ee192e272622d26`. Fixed-U keeps the Telegram DC endpoint as the destination for SOCKS5/HTTP, supplies the selected proxy host/port/user/password to the native socket proxy carrier, custom-resolves SOCKS5 proxy hostnames through the existing resolver, and then reuses the same TCP/TLS/MTProto transport.

Spec-first platform-core implementation is `0a8d8f9b1830e989793da934210f699ea5d39d1f`; canonical network blob is `b2ce7b4f018494cc806520f0fd241f5264ba2bbd` and spec blob is `17e42e6671a2bd553bf6a04610348815124b7e26`. The existing telegram-network owner now establishes RFC1928/RFC1929 SOCKS5 or HTTP CONNECT tunnels to the selected Telegram DC endpoint before applying the existing endpoint secret, obfuscation/TLS camouflage, framing and auth/session lifecycle. SOCKS5 reuses the existing direct-NoProxy transient DoH proxy-host candidates and successful-IP promotion; HTTP proxy hostnames use the ordinary socket resolver. Proxy negotiation, authentication, resolver, cancellation and timeout failures fail closed and never create a direct Telegram connection.

Desktop production mirror is `e2fa2dc75f52763e126b5f3f7db8492ae26b92b5`, with byte-identical network/spec blobs. ProductSettingsOwner v2 remains the only durable settings owner; TelegramClientLease/RuntimeClient remain the existing transient projection boundary; RuntimeClient projection changes continue to cancel/drop live request/session transport while preserving reusable auth material. No runtime/provider/settings/session owner was added.

Focused SOCKS5 request-framing and HTTP CONNECT/Basic-auth source contracts were authored in the existing telegram-network test module, and existing RuntimeClient session-fencing contracts remain applicable. None were executed. `TDRP-R9-PROXY-MTPROTO-CONSUMER-001` remains `implemented / validation_status=not-run / accepted=false`; its SOCKS5/HTTP carrier blocker is closed and only Stage-B executable validation remains for this responsibility. Global coverage counters remain `source_entries_total=16202, responsibilities_total=122, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`; this round reread already-mapped fixed-U blobs and therefore does not fabricate source-entry counter decrements.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


Round103 ledger hygiene: historical missing-canonical-Settings blockers were removed from `TDRP-R9-MTPROTO-WEB-PROXY-CANONICAL-ADDRESS-SETTINGS-001` and `TDRP-R9-PROXY-EXTERNAL-MUTATION-CONSUMER-001` because ProductSettingsOwner v2 is now authoritative. Neither row was advanced to implemented: exact reviewed IDNA/QUrl::toAce support and a verified shipping source-neutral external local-automation command surface remain open respectively. No executable validation was run.


### Round 104 / evidence round135 — MTProto TCP socket lifecycle fencing

Fixed-U authority remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual reread of `Telegram/SourceFiles/mtproto/details/mtproto_tcp_socket.cpp@b3130fd706b7cbfa33f7c9f54ac5e87abe9c3e10` and adjacent `mtproto_abstract_socket.cpp@653033365b543e554104c99a963ee194ddb830d9`, `connection_abstract.cpp@3baabf3f72c7c7edba3151814681fbf167a5ca0a`, `connection_tcp.cpp@d721a9058e1885d2964ded55bd4e4c7c2ce9e381`, and `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614` isolated `TDRP-R9-MTPROTO-TCP-SOCKET-LIFECYCLE-FENCING-001`. Fixed-U applies the selected QNetworkProxy before connect, forwards connected/disconnected/readyRead/error through the socket owner, logs detailed socket errors before collapsing them to the connection error lifecycle, destroys the connect-only lifetime after a connection becomes Ready, destroys all socket lifetimes when finished, and fences SessionPrivate callbacks by exact connection identity before adoption/restart.

Existing-owner-first/spec-first implementation is platform-core spec commit `550c2e170f172ab7c96ba9d11c46234121b21c11` and production commit `dc16f388756d681ef4ee6683b98525edef869e1f`. Canonical `native/telegram-network/src/lib.rs@63f9c4a26385a776fbdd42af3d0de44a37857691` now releases the connect-only `NetworkAttemptControl` active-stream/deadline ownership after a successful controlled establish/resume handoff while keeping the adopted transport alive. Existing `EstablishedSession::invoke_raw_controlled` continues to reattach the owned request control for bounded RPC I/O and clear it on settlement. The same source review found and corrected `flush_bounded` recursion plus `send_payload`'s unbounded flush bypass so flush now uses the active request/attempt write bound.

Desktop production mirror commit is `e4fe918b8e8ba5589d1ab3d2cdae888087933719`; its telegram-network and spec blobs are byte-identical to canonical at `63f9c4a26385a776fbdd42af3d0de44a37857691` and `a463412a99805ebf38e035db3dde4f29964be206`. Existing RuntimeClient fencing remains unchanged at `native/telegram-runtime/src/runtime.rs@3a3da0eb4bf004569a3989e05e55c8b86a199c41`: projection changes cancel all owned request controls and clear live sessions before replacing the transient proxy projection while reusable auth material remains separately owned.

Two focused loopback socket lifecycle contracts were authored in the existing telegram-network test module and were not executed: successful connect handoff releases the active-stream registration without cancelling the control, and request control can re-own then release the same adopted transport. Stage A executable validation remains `not-run`; `accepted=false`. Coverage becomes `responsibilities_total=123, source_entries_total=16202, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Unknown/unread do not decrease because this round isolates another responsibility from an already-read fixed-U connection family rather than claiming a newly closed source-entry inventory item.

The adjacent fixed-U `protocolForFiles` responsibility is deliberately not claimed here: `TcpSocket` sets `SendBufferSizeSocketOption=kFilesSendBufferSize` and `ReceiveBufferSizeSocketOption=kFilesReceiveBufferSize` only for file protocol connections. That file-transfer socket tuning remains the next explicit open adjacency for source/owner review.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.


Round104 final source-level compliance: platform-core `1138e34377d441a97d72b23279dacdb0bc310d2e` / spec blob `449cfa61c2b3b20c5fd42838a627a8e5330f8bb4` records R1-R7 and AC-1..AC-7 source-review passed for `TDRP-R9-MTPROTO-TCP-SOCKET-LIFECYCLE-FENCING-001`; executable validation remains `not-run`, accepted remains false, and fixed-U `protocolForFiles` socket-buffer tuning remains open.


## Task round 105 / source evidence round 136 — file-transfer native socket-buffer tuning (2026-11-01)

Fixed U remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`. Manual source reread pins `mtproto_abstract_socket.h@4283a353dd00db4b12044ef5a1d46cfbde7f659b`, `mtproto_tcp_socket.cpp@b3130fd706b7cbfa33f7c9f54ac5e87abe9c3e10`, `mtproto_tls_socket.cpp@e7562e3ca1a1da03f5d368085d5e61d887ab3f16`, `session_private.cpp@13cbc76ec723386027b5b4dcac79999b791c4614`, `mtp_instance.cpp@d25fe3b72298fab371854f80186ee8e4777ea087`, `facade.h@39e2f732c7df32af37ebf8222e887aec7e11b256`, `file_upload.cpp@a19470a24b6a62b58c63d8cc21d831641d970260`, and `download_manager_mtproto.cpp@d0007eb17fb0bcfbdffd89b86d8efdb900fcd363`.

Fixed-U defines both file socket buffers as exactly 2 MiB. `protocolForFiles` is not equivalent to all upload/download file-thread sessions: SessionPrivate enables it for MediaCluster or CDN, while the adjacent `isUploadDcId` predicate is explicitly commented out. MediaCluster includes downloads plus group-call stream/export-media/updater shifts. Existing-owner-first therefore maps the exact responsibility to the existing telegram-network native carrier owner, using the already-shipping `DcPurpose::{Main,Media,Cdn}` boundary; telegram-media remains the transfer scheduling/progress/retry/integrity owner.

Spec-first platform checkpoint `c284ba77b7a18477cf8900e6766787986a4c5aee` created `docs/specs/telegram-file-transfer-socket-buffer-tuning.md`. Canonical platform implementation `0ac50a1545c95681213b329e0589683cb11404a7` changes `native/telegram-network/src/lib.rs` to blob `6dece8da8f9a1512dcd17bc76fd1cc9ce43c1ef6`; final spec blob is `6fca16349ff56f3493854a4a6a78f37ee932a339`. It derives a private Files profile for Media/Cdn and Default for Main, requests 2 MiB SO_SNDBUF/SO_RCVBUF immediately after native TCP connect, threads that profile through direct, SOCKS5/HTTP and MTProto-proxy native carriers, and propagates OS option failures through existing `NetworkError::Io`. WebProxy relay/browser streams are not falsely claimed as native TcpStream tuning. No dependency or second owner was added.

Desktop production/spec mirror checkpoint `23d73b32e301408f9c8aed151cb1f9e442516e13` is byte-identical at network blob `6dece8da8f9a1512dcd17bc76fd1cc9ce43c1ef6` and spec blob `6fca16349ff56f3493854a4a6a78f37ee932a339`. Two contracts are authored in the existing telegram-network test module for exact 2 MiB values and Main-vs-Media/Cdn profile selection; they were not executed.

`TDRP-R9-MTPROTO-FILE-TRANSFER-SOCKET-BUFFER-001` is `implemented / validation_status=not-run / accepted=false`. The prior lifecycle row's protocolForFiles blocker is closed. Coverage becomes `source_entries_total=16202, responsibilities_total=124, unknown=16172, unread=16145, omitted=0, unmapped_responsibilities=0`. Unknown/unread are intentionally not decremented because this round isolates another exact responsibility from already-reviewed adjacent source families rather than claiming whole-file source-entry closure.

Stage A remains execution-free: no GitHub Actions/check/status/workflow query or trigger, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E execution, visual/a11y, package, deployment, installation or runtime acceptance was performed.


Round105 final source-level compliance: platform-core `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` / final spec blob `fd493a83b465c985ec4f8999655e29a7f3090e17` records R1-R8 and AC1-AC7 source-review passed for `TDRP-R9-MTPROTO-FILE-TRANSFER-SOCKET-BUFFER-001`. Desktop bookkeeping checkpoint is `77e9dc1a45e9486ea8cf0ebe912b1a15c83c17a7`; executable validation remains `not-run`, accepted remains false.

## Round 106 / source-entry closure — repository helper-retention lifecycle (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`). The complete fixed-U recursive tree was reread with `truncated=false`. Comparing its blob order against the unique parity ledger's independent `source_path/source_paths` anchors selected the first remaining unrepresented entry: `.agents/skills/code-health/references/harness-lifecycle.md@f18c847326d2cfbdc8bf168e804ecbd5343ee61c`.

The exact reference and direct code-health consumers were read. It defines conservative helper-retention/documentation decision signals, 90-day no-recent-use / <=2-lifetime-consumer review triggers, unknown/open-consumer holds, lower-bound archive coverage, at-most-weekly canonical decisions, and pre-retirement caller/dependency/build/workflow/open-work review. Direct consumers are fixed-U `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f`, `harness.py@652a9ee7ed6a690705cad45aeff06568210db6b1`, `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d`, and `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12`; those consumer reads do not bulk-close their independent source entries.

Existing-owner-first maps this responsibility to the existing Fabushi repository quality-health / Test Strategy / Test Evidence owner. Source-neutral helper-retention qualification/evidence contracts were first saved at desktop spec checkpoint `9837dd5b6b35ec38ad177c3e39be36e8a1ab7adf`; the integration row is `TDRP-R9-CODE-HEALTH-HELPER-RETENTION-LIFECYCLE-001`. No platform-core product/runtime change is required and no second scanner/findings/task owner is created.

Exactly one source entry is closed by this round: `unknown 16172 -> 16171`, `unread 16145 -> 16144`, `omitted=0`; `responsibilities_total 124 -> 125`. Authored contracts remain `not-run`; no executable verification or acceptance is claimed.



## Round 107 / source-entry closure — repository helper-inventory instrument (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`). From desktop parent `c55a04ad94a43424df161670b639705f7b1dda9b`, the next independent unrepresented fixed-U entry is `.agents/skills/code-health/scripts/harness.py@652a9ee7ed6a690705cad45aeff06568210db6b1`.

The exact script and direct consumers were read. `harness.py` owns source-neutral helper inventory/consumer-evidence mechanics: exact-tree helper/doc discovery; current-path birth/task provenance; API/internal/self-test classification; uniquely attributable symbols; direct callers and transitive internal dependents; workflow/open-task references; conservative archived-patch coverage; product/uncertain/harness/self-test/creation separation; distinct-task de-duplication; documentation/index/source-weight facts; explicit coverage limits; and JSON/Markdown rendering. `report.py` calls `harness.inspect`; `health.py` persists `harness.json/harness.md` and preserves unverified retention state when coverage is unavailable; `harness_test.py` / `health_test.py` are direct executable-contract consumers.

Existing-owner-first maps this to the current Fabushi repository quality/test-evidence owner. Spec-first checkpoint `606f8b0f71335967f574d3ca94d1d4bec7cb0b48` adds helper-inventory qualification/evidence plus the complete-read dossier. No upstream Python scanner, second findings ledger/task queue/product runtime/platform-core owner or new ADR is introduced. The prior `TDRP-R9-CODE-HEALTH-HELPER-RETENTION-LIFECYCLE-001` remains the separate decision-policy owner.

`TDRP-R9-CODE-HEALTH-HARNESS-INVENTORY-INSTRUMENT-001` is `implemented / validation_status=not-run / accepted=false`. Contracts were authored but not executed. Exactly one source entry closes: `unknown 16171 -> 16170`, `unread 16144 -> 16143`, `omitted=0`; `responsibilities_total 125 -> 126`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 108 / source-entry closure — helper-inventory regression contract suite (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`). From desktop parent `f80ad4b1d1b82deea3b09be33d9b42acca0d7415`, the next independent unrepresented fixed-U entry was `.agents/skills/code-health/scripts/harness_test.py@5d6b2537c9289cb3e1db0904b9f8e1b99dec4b74`.

The exact 10,145-byte test blob was completely read. It defines 13 helper-inventory regression tests plus disposable Git-repository fixture helpers. The contracts explicitly cover the 89/90-day threshold boundary; low-lifetime distinct-consumer counts; absent/no-patch/dirty archive remaining unavailable rather than known zero; creation/harness/self-test/uncertain/product separation; final saved-patch context while excluding deleted lines and helper edits; task-level de-duplication; comments/string/raw-string/commented-include suppression; internal-helper dependency/open-work propagation; exact-ref versus worktree document authority; namespace-safe symbol attribution; and stable finding identity/unverified semantics when current evidence is unavailable.

Direct dependencies are `harness.py@652a9ee7ed6a690705cad45aeff06568210db6b1`, `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12`, `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`, and the disposable `Repo` fixture imported from `health_test.py@6058fa64a5121c966021d8589b2010480aec8794`. The module has its own `unittest.main` entry and ordinary `*_test.py` discovery shape; fixed-U repository search found no separate product/runtime caller by file name.

Existing-owner-first maps this test responsibility to the current Fabushi repository quality/test-evidence owner, following the already-established `health_test.py` mapping. Spec-first checkpoint `f00a86d5e96a3584efe621f471a1046d1536da22` adds the source-neutral regression-suite qualification/evidence contracts and complete-read dossier. No upstream Python unittest harness, second scanner/findings/task owner, platform-core production delta, or new ADR is introduced.

`TDRP-R9-CODE-HEALTH-HARNESS-INVENTORY-REGRESSION-001` is `implemented / validation_status=not-run / accepted=false`. Contracts are authored but not executed. Exactly one source entry closes: `unknown 16170 -> 16169`, `unread 16143 -> 16142`, `omitted=0`; `responsibilities_total 126 -> 127`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 109 / source-entry closure — repository include graph and build-cost proxy (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`). From desktop parent `bbd9c803755a14e5ac45f9fa5ae4894f3aede06c`, comparing the complete fixed-U blob order against every independently represented ledger `source_path@blob` / `source_paths[]` anchor selects `.agents/skills/code-health/scripts/includes.py@2c02b4e7eb821f123588e0a80056edef86fc084c` as the first remaining unrepresented source entry, immediately after the already-represented `history.py`.

The exact 5,150-byte script was read completely. It owns the repository-quality include graph/build-cost proxy: quoted C/C++ include/import and `.style` import edges; deterministic local/own-root/other-root plus generated-style resolution; explicit unresolved counts; cycle-safe transitive closure; target PCH closure; TU non-PCH header-line weight; header reach/rebuild accumulation; total build-work proxy including distinct PCH closures; and target-layering violations against transitive link dependencies. These are source-line/static-structure proxies, not configured-build or wall-clock compiler evidence.

Direct consumers and adjacent providers were read at fixed U: `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` consumes costs/PCH/layering for snapshot findings; `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` consumes resolution, graph edges, reach/closure/cost and layering for diff-time ratchets; `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` constructs the graph for atlas generation; `atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` consumes graph edges for cross-area dependency counts; `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` exercises target/PCH, upward-layering and reach/weight contracts. `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4`, `cmake_model.py@e40ec0fc152958f96bd58d795f70d1a3a24ab1eb` and `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b` provide parsing/build-model/source-tree inputs. No independent product/runtime consumer was found.

Existing-owner-first maps the responsibility to current Fabushi repository quality/test-evidence governance. Spec-first commit `32178dfb8a7ea859f21870eeb92df6f66a37e831` extends `quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/code-health-includes-script-complete-read.md`. No platform-core production delta, upstream Python port, second include scanner/build model/findings/task/runtime owner, or ADR is introduced.

`TDRP-R9-CODE-HEALTH-INCLUDE-GRAPH-BUILD-COST-001` is `implemented / validation_status=not-run / accepted=false`. Authored contracts are `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-RESOLUTION-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-CLOSURE-PCH-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-BUILD-COST-001`, `CONTRACT-TDRP-CODE-HEALTH-INCLUDE-LAYERING-001`, `CONTRACT-TDRP-CODE-HEALTH-STYLE-IMPORT-001`, and `INT-TDRP-CODE-HEALTH-INCLUDE-GRAPH-001`; none was executed. Exactly one source entry closes: `unknown 16169 -> 16168`, `unread 16142 -> 16141`, `omitted=0`; `responsibilities_total 127 -> 128`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 110 / source-entry closure — repository source metrics, unused candidates, and hotspot composition (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries / 6,859 blobs). From desktop parent `324ffc521c2cddaad476e80894b49a144753c680`, the unique ledger reported `source_entries_total=16202 / unknown=16168 / unread=16141 / omitted=0 / responsibilities_total=128 / unmapped_responsibilities=0`. Complete tree-order comparison confirms `.agents/skills/code-health/scripts/metrics.py@42f42b1100f88bd85977b66dfb6bf13e9dc88e46` is the first independent unrepresented entry after the represented `includes.py`; `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` remains separately unrepresented.

The exact 4,296-byte `metrics.py` blob was read completely. It provides fallback source-area grouping, per-file line/function/class shape metrics, a 150-line long-function collection pre-filter, a rough methods+fields>=40 class pre-filter, and conservative lexical unused-function candidates. Unused candidates are limited to non-test `Telegram/SourceFiles/*.cpp|*.cc|*.cxx`, exclude the language resource from global identifier counts, retain exemptions/constructor filtering, and require declaration-only remaining mentions. The source explicitly treats these as candidates; deletion safety is not proven by the scanner.

Direct fixed-U consumers were read: `report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` consumes file metrics/unused candidates/area fallback and composes `hotspot=round(churn*lines/1000,1)`, while separately applying 300-line long-function and 120-method/60-field large-class finding thresholds; `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` reuses `declares` and `UNUSED_EXEMPT` in orphan-reference review; `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` uses area fallback; `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` covers declared-never-called and initializer-list-use controls. `cpp_text.py@366f173248add43c34317c8fe17d9ca31d46b1d4`, `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b`, and `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df` supply parser/source/churn inputs.

Existing-owner-first maps the responsibility to current Fabushi repository quality/test-evidence governance. Spec-first checkpoint `68e34c7c7ee1459d2ec06f54fc1f4d29583a29e6` extends `quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/code-health-metrics-script-complete-read.md`. No platform-core production delta, upstream Python port, second metrics/dead-code scanner/findings/task/performance/runtime owner, or new ADR is introduced.

`TDRP-R9-CODE-HEALTH-METRICS-UNUSED-HOTSPOT-001` is `implemented / validation_status=not-run / accepted=false`. Authored contracts are `CONTRACT-TDRP-CODE-HEALTH-METRICS-AREA-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-FILE-SHAPE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-CANDIDATE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-UNUSED-NEGATIVE-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-HOTSPOT-COMPOSITION-001`, `CONTRACT-TDRP-CODE-HEALTH-METRICS-FINDING-THRESHOLDS-001`, and `INT-TDRP-CODE-HEALTH-METRICS-REPORT-001`; none was executed. Exactly one source entry closes: `unknown 16168 -> 16167`, `unread 16141 -> 16140`, `omitted=0`; `responsibilities_total 128 -> 129`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 111 / source-entry closure — repository snapshot, findings lifecycle, and report projection (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries / 6,859 blobs). From desktop parent `075b9005e76023479151e1fd5659bbc718861db0`, the unique ledger reported `source_entries_total=16202 / unknown=16167 / unread=16140 / omitted=0 / responsibilities_total=129 / unmapped_responsibilities=0`. Tree-order comparison confirms `.agents/skills/code-health/scripts/report.py@40d9512e3be2e075154230e7fa214d4b52e2bc12` is the next independent unrepresented source entry after represented `metrics.py`; `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` remains separately unrepresented.

The exact 19,769-byte `report.py` blob was read completely. It owns repository-quality aggregation and stable finding/report semantics: multi-provider exact-tree snapshot assembly; stale nested map-reference detection; stable `kind:key` finding identities, class and impact×churn priority; declared threshold projections for file/class/function/build/clone/layering/unused/map/atlas/hot-area/helper signals; historical `first_seen` retention; open/resolved/ignored/unverified merge behavior with explicit unavailable-evidence fencing and a 180-day resolved-history window; and deterministic Markdown totals/deltas/hotspots/findings/build-cost/helper coverage.

Direct consumers were read at fixed U. `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` calls `snapshot -> findings -> merge -> markdown` for scans and owns ignored overrides, daily idempotence, temporary-vs-durable run placement, atomic ledger publication and history append; it also reuses snapshots for atlas/area and finding verification. `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` covers map references, atlas classification and durable/temporary state behavior. `harness_test.py@5d6b2537c9289cb3e1db0904b9f8e1b99dec4b74` directly covers merge semantics for unavailable helper evidence and stable identity. `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` imports shared report limits/file-limit policy. Existing provider modules remain separate source entries and are not re-claimed.

Existing-owner-first maps the portable responsibility to current Fabushi repository quality/test-evidence governance and existing repository quality-health command orchestration. Spec-first commit `1195399417cf08502eda2c5e149ffad8536d2cad` extends `quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/code-health-report-script-complete-read.md`. No platform-core production delta, upstream Python port, second scanner/findings ledger/task queue/product telemetry/publication runtime, or ADR is introduced.

`TDRP-R9-CODE-HEALTH-REPORT-SNAPSHOT-FINDINGS-LIFECYCLE-001` is `implemented / validation_status=not-run / accepted=false`. Authored contracts are `CONTRACT-TDRP-CODE-HEALTH-REPORT-SNAPSHOT-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-FINDINGS-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MERGE-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MAP-REFERENCES-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-MARKDOWN-001`, `CONTRACT-TDRP-CODE-HEALTH-REPORT-UNAVAILABLE-EVIDENCE-001`, and `INT-TDRP-CODE-HEALTH-REPORT-HEALTH-COMPOSITION-001`; none was executed. Exactly one source entry closes: `unknown 16167 -> 16166`, `unread 16140 -> 16139`, `omitted=0`; `responsibilities_total 129 -> 130`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 112 / source-entry closure — diff-time repository review ratchet and move-only proof (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries / 6,859 blobs). From desktop parent `fb2f415ce862ec8890878814eded232d436c3b2d`, the unique ledger reported `source_entries_total=16202 / unknown=16166 / unread=16139 / omitted=0 / responsibilities_total=130 / unmapped_responsibilities=0`. Tree-order comparison confirms `.agents/skills/code-health/scripts/review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` is the next independent unrepresented source entry after represented `report.py`; `sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b` remains next in the script tree.

The exact 21,693-byte `review.py` blob was read completely. It owns a change-scoped repository review ratchet and separate move-only proof: explicit base/staged/worktree/head subjects; in-scope tracked/untracked diff acquisition; shared file/class thresholds; hard new-file and novel cross-target include errors; normal/strict severity for excessive oversized growth and copied-from additions; warnings for known target crossings, class growth, material fresh-header source-build-cost growth, stranded removed definitions and last-use orphan candidates; and a conservative move proof that permits declared scaffolding/declaration normalization while rejecting substantive novelty/loss or reordered statements.

Direct fixed-U consumers were read. `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` imports the module, reconstructs staged subjects, calls `review.check` and `review.moves`, renders warning/error/move diagnostics, and owns exit-code composition. `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` directly covers target-crossing severity, oversized growth tolerance/strictness, copied-from warning, untracked oversized additions, platform reference liveness, orphan styles/language keys, deletion/move negatives, CRLF stability and move-only positive/negative cases. `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` defines the pre-review check/moves contract. Clone/CMake/text/include/metrics/report/source modules remain independently accounted providers.

Existing-owner-first maps the responsibility to current Fabushi repository quality/test-evidence governance with the existing quality-health command orchestration remaining the command/exit-code owner. Spec-first commit `467f75c86e3558d7c15454a1cd66f22e2ea8e6c5` extends `quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/code-health-review-script-complete-read.md`. No platform-core production delta, upstream Python port, second scanner/findings/task queue/product telemetry/configured-build/review runtime, or ADR is introduced.

`TDRP-R9-CODE-HEALTH-DIFF-REVIEW-RATCHET-MOVE-ONLY-001` is `implemented / validation_status=not-run / accepted=false`. Nine contracts/integration cases are authored but not executed. Exactly one source entry closes: `unknown 16166 -> 16165`, `unread 16139 -> 16138`, `omitted=0`; `responsibilities_total 130 -> 131`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 113 / source-entry closure — repository source authority and filtering (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; its recursively reread root tree is `truncated=false` with 7,194 entries. From desktop parent `839f2313ed93e8e723e053f6df00982d19ee13e9`, the unique ledger reported `source_entries_total=16202 / unknown=16165 / unread=16138 / omitted=0 / responsibilities_total=131 / unmapped_responsibilities=0`. Tree-order comparison confirms `.agents/skills/code-health/scripts/sources.py@749ccc1f503e203d4a651c9ac793381e0e0a314b` was independently unrepresented after represented `review.py`; it is the last blob in the code-health scripts directory. The next independent unrepresented blob in fixed-U tree order is `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5`.

The exact 6,085-byte `sources.py` blob was read completely. It owns deterministic code-health source filtering; exact-ref `ref^{commit}` resolution; recursive tracked Git-tree/blob acquisition; top-level `Telegram/lib_*` gitlink pin capture and exact pinned-submodule reads when locally available; mutable WORKTREE acquisition with HEAD/Telegram dirty/submodule-status provenance; checked/timeout-bounded Git execution; text/non-text decoding and CRLF normalization; and the read-only `SourceTree` carrier.

Direct fixed-U consumers were read. `health.py@4fd8d8013d0efb7103a57e4ff7b1d395c448eb0d` selects exact-ref versus worktree subjects and composes scan/check/verify/detached-worktree operations. `harness.py@652a9ee7ed6a690705cad45aeff06568210db6b1` reuses exact tree/blob/Git access; `harness_test.py@5d6b2537c9289cb3e1db0904b9f8e1b99dec4b74` and `health_test.py@6058fa64a5121c966021d8589b2010480aec8794` create exact/worktree carriers. `review.py@89f15913714c54467ef40d6f42feb5c526cb0f32` consumes `LANG_FILE/git/wanted`; `metrics.py@42f42b1100f88bd85977b66dfb6bf13e9dc88e46` consumes `LANG_FILE`; `atlas.py@6680fa747a1ddc520f4396949f3c84ab25cd8c43` and `history.py@7c12f5b091936723c6ad005dda4bb8e2b1e549df` consume the Git runner. Fixed-U `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f` requires durable scans to bind to an exact ref, explicitly names missing pinned submodule objects as skipped, and treats failed fetch as a failed run.

Existing-owner-first maps the responsibility to current Fabushi repository authority plus repository quality/test-evidence governance, with the existing quality-health command remaining the orchestration/lifecycle owner. Spec-first commit `7702790394cb5848c10f821e663d0c5cf4c638fc` extends `quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/code-health-sources-script-complete-read.md`. No platform-core production delta, upstream Python source-loader port, second source index/scanner/findings store/task queue/product runtime, or ADR is introduced.

`TDRP-R9-CODE-HEALTH-SOURCE-AUTHORITY-FILTERING-001` is `implemented / validation_status=not-run / accepted=false`. Eight contracts/integration cases are authored but not executed. Exactly one source entry closes: `unknown 16165 -> 16164`, `unread 16138 -> 16137`, `omitted=0`; `responsibilities_total 131 -> 132`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.

## Round 114 / source-entry closure — repository continue task/process authority (2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` (root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries). From desktop parent `304c467453dcea45be3ed7897444256a8f41a44e`, the unique ledger reported `source_entries_total=16202 / unknown=16164 / unread=16137 / omitted=0 / responsibilities_total=132 / unmapped_responsibilities=0`. Tree-order comparison confirms `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5` is the next independent unrepresented blob after represented code-health `sources.py`.

The exact `continue/SKILL.md` blob was read completely: 39,712 characters / 722 lines. It owns checkout-level repository-development continuation semantics: explicit batch specification; resume-first exclusive active/split state; frozen initial batch; single-task ownership; source-lineage and source-preparation gates; one performer at a time; genuine approved/blocked/split-required/hard-stop boundaries; result-attributable discovery routing; atomic split replacement with at most one retained-work carrier; compatible-unclaimed consolidation with durable old-to-new aliases/no-repeat completion; CAS publication; recoverable race/conflict handling; and implementation/validation/acceptance separation.

Direct fixed-U host consumers were read: `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8` and `.grok/commands/continue.md@1365e5a7dc4c167468f7b6f57ec92ff89389b32e` both require the shared continue skill completely and only adapt host mechanics; `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61` is an invocation/default-prompt surface with no durable-state ownership. Composed workflow boundaries were also read in `process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`, `perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, split reference `552ad09b2fa022b7df94481746a51167305bf846`, and consolidation reference `bd301d93f8f670ad041b3db72330e17cbb50eb5c`.

Existing-owner-first maps this responsibility to current Fabushi repository development process authority plus repository quality/test-evidence governance. Root `AGENTS.md` already owns the development lifecycle, current source/spec authority and task context/handoff; the existing quality strategy/evidence contracts own source-neutral qualification/evidence. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner and platform-core runtime are rejected as different state domains. Exact platform-core commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` contains no repository task/split/consolidation authority surface, so there is no platform-core production delta and no new ADR.

Spec-first commit `56d0e8a62c2dd94fe5a558d0296054be3d86e453` extends `AGENTS.md`, `quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/continue-skill-complete-read.md`. Authored, not run: `CONTRACT-TDRP-CONTINUE-BATCH-FREEZE-001`, `CONTRACT-TDRP-CONTINUE-TASK-STATE-AUTHORITY-001`, `CONTRACT-TDRP-CONTINUE-OWNERSHIP-001`, `CONTRACT-TDRP-CONTINUE-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-TRANSACTION-001`, `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-001`, `CONTRACT-TDRP-CONTINUE-RECOVERY-001`, and `INT-TDRP-CONTINUE-HOST-ADAPTER-COMPOSITION-001`.

`TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` is `implemented / validation_status=not-run / accepted=false`. Exactly one independent source entry closes: `unknown 16164 -> 16163`, `unread 16137 -> 16136`, `omitted=0`; `responsibilities_total 132 -> 133`. Adjacent continue `agents/openai.yaml` and reference blobs remain independently unclosed.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.

## Round 115 / source-entry closure — OpenAI continue host adapter (2026-11-01)

From desktop checkpoint parent `d620e371dbeab832fc6983462fc2466c5abe111c`, the live branch was reread unchanged before spec publication. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` was verified in `bhrumom/fabushi-platform-core`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` is `truncated=false` with 7,194 entries / 6,859 blobs. Before closure the unique ledger reported `source_entries_total=16202 / unknown=16163 / unread=16136 / omitted=0 / responsibilities_total=133 / unmapped_responsibilities=0`.

The exact `.agents/skills/continue/agents/openai.yaml@e981285ee10cbed9e27e358d60ce596c9e0b3d61` blob was read completely: 291 characters / 4 lines. It is declarative OpenAI host metadata only: `display_name`, `short_description`, and a `default_prompt` that invokes `$continue`, places active work first, freezes exactly the caller's batch specification behind it, and processes one task at a time. Repository search found no second in-repository parser/state owner. Adjacent OpenAI adapters for process-inbox, perform-task and code-health use the same declarative schema, confirming host discovery/invocation rather than durable scheduling ownership.

Existing-owner-first maps the entry into the already implemented `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` owner: Fabushi repository development process authority plus repository quality/test-evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a copied OpenAI skill runtime and a Telegram-specific scheduler remain rejected. No platform-core production delta and no ADR are required.

Spec-first commit `fc8bf28953a3f3e100e55663ea3c86f7a51dbb44` extends existing qualification/evidence and adds `projects/telegram-desktop-rust/dossiers/continue-openai-host-adapter-complete-read.md`. Authored, not run: `CONTRACT-TDRP-CONTINUE-OPENAI-HOST-METADATA-001` and `INT-TDRP-CONTINUE-OPENAI-HOST-INVOCATION-001`.

Exactly one independent source entry closes under the existing responsibility: `unknown 16163 -> 16162`, `unread 16136 -> 16135`, `omitted=0`; `responsibilities_total` remains 133 and `unmapped_responsibilities=0`. The next independent unrepresented fixed-U blob in tree order is `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.

## Round 116 / source-entry closure — continue pending-task consolidation (2026-11-01)

From desktop checkpoint parent `b1ecb2452f4fe8c9d215c6e2731573cac735a95f`, the live migration branch was reread unchanged before spec publication; the spec-first CAS then published `f999edc84561fe4d7d97b0eff8cebe068ad8369f`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` was verified in `bhrumom/fabushi-platform-core`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread `truncated=false` with 7,194 entries / 6,859 blobs. Before closure the unique ledger was `source_entries_total=16202 / unknown=16162 / unread=16135 / omitted=0 / responsibilities_total=133 / unmapped_responsibilities=0`.

The exact `.agents/skills/continue/references/consolidate-pending-tasks.md@bd301d93f8f670ad041b3db72330e17cbb50eb5c` blob was read completely: 14,614 characters / 285 lines. Direct consumers were reread at fixed U: `continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c`, `process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`, shared project-context `53c8657eb7949a5a2fba7e7adc8d1073f3079ead`, concrete workspace helper `3cd99d173fcc3a6e293cfe36058a5fd9e492887d`, and workspace tests `79f943a99809bc6acf173872d9a780346648b5cf`.

Existing-owner-first keeps the source under `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`: Fabushi repository development process authority plus quality/test-evidence governance. No new responsibility, owner, ADR, Telegram-specific queue, Product Tasks/Automations owner, Agent/Coordinator/Host/Runner owner, or platform-core production delta is introduced. Spec-first commit `f999edc84561fe4d7d97b0eff8cebe068ad8369f` extends the existing policy with fail-closed pristine-todo eligibility, same-project/frozen-batch partitioning, reverse-dependent/cycle simulation, merge/no-merge criterion preservation, durable aliases and retained-content fingerprints, final reread, explicit-path transaction scope, CAS publication, and generic-publish revalidation.

Authored but not run: `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-ELIGIBILITY-001`, `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-MERGE-NO-MERGE-001`, `CONTRACT-TDRP-CONTINUE-CONSOLIDATION-ALIAS-FINGERPRINT-001`, `INT-TDRP-CONTINUE-CONSOLIDATION-PUBLISH-001`, and `FAULT-TDRP-CONTINUE-CONSOLIDATION-RECOVERY-001`. Pre-commit drift is `RACED` with no tracked write and the pending marker retained; a committed semantic/publication failure is recoverable `BLOCKED`, preserves the commit, and never force-pushes.

Exactly one independent source entry closes under the existing responsibility: `unknown 16162 -> 16161`, `unread 16135 -> 16134`, `omitted=0`; `responsibilities_total=133`, `unmapped_responsibilities=0`. The next fixed-U tree entry at index 54, and next independent unrepresented continuation reference, is `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.

## Round 117 / source-entry closure — continue split-required routing (2026-11-01)

From desktop checkpoint parent `7700fd41ec787c08bb945a6f6c3660f9cde22930`, the live migration branch was reread unchanged before spec publication; the spec-first CAS then published `b87ed018628e6693e790a29d21815b8a8665da5f`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` was verified in `bhrumom/fabushi-platform-core`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread `truncated=false` with 7,194 entries / 6,859 blobs. Before closure the unique ledger was `source_entries_total=16202 / unknown=16161 / unread=16134 / omitted=0 / responsibilities_total=133 / unmapped_responsibilities=0`.

The exact `.agents/skills/continue/references/split-required-task.md@552ad09b2fa022b7df94481746a51167305bf846` blob was read completely: 5,813 characters / 112 lines, fixed-U tree index 54. Direct consumers were reread at fixed U: `continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c`, `perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`, shared project-context `53c8657eb7949a5a2fba7e7adc8d1073f3079ead`, concrete workspace helper `3cd99d173fcc3a6e293cfe36058a5fd9e492887d`, and workspace tests `79f943a99809bc6acf173872d9a780346648b5cf`.

Existing-owner-first keeps the source under `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001`: Fabushi repository development process authority plus quality/test-evidence governance. No new responsibility, owner, ADR, Telegram-specific queue, Product Tasks/Automations owner, Agent/Coordinator/Host/Runner owner, or platform-core production delta is introduced. Spec-first commit `b87ed018628e6693e790a29d21815b8a8665da5f` extends the existing policy with fail-closed canonical split eligibility, independently shippable/testable successors, exclusive first retained-work carrier semantics, deferred source-ref transfer at canonical carrier Start, exact dependency/project rewrite, sealed multi-target history, explicit-path CAS publication and fail-closed recovery.

Authored but not run: `CONTRACT-TDRP-CONTINUE-SPLIT-ELIGIBILITY-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-REPLACEMENT-001`, `CONTRACT-TDRP-CONTINUE-SPLIT-CARRIER-HANDOFF-001`, `INT-TDRP-CONTINUE-SPLIT-PUBLISH-001`, and `FAULT-TDRP-CONTINUE-SPLIT-RECOVERY-001`. Retained implementation remains source-state-retained until the first carrier is canonically started and its seal revalidated; changed seal, incoherent carrier, semantic conflict or remote outage remains recoverable and never force-published.

Exactly one independent source entry closes under the existing responsibility: `unknown 16161 -> 16160`, `unread 16134 -> 16133`, `omitted=0`; `responsibilities_total=133`, `unmapped_responsibilities=0`. The next independent fixed-U blob in tree order is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.



## Round 118 / source-entry closure — repository dependency advisory governance (2026-11-01)

From desktop checkpoint parent `099d79fc5c50bf61ae08298150f9a5186e8ca23c`, live authority was reread before this work. Desktop main is `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread `truncated=false` with 7,194 entries / 6,859 blobs. Before closure the unique ledger was `source_entries_total=16202 / unknown=16160 / unread=16133 / omitted=0 / responsibilities_total=133 / unmapped_responsibilities=0`.

The exact `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8` source was read completely (8932 characters / 150 lines), fixed-U tree index 56. Adjacent direct/supporting consumers were read: OpenAI invocation metadata `a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`, patched-fork contract `29a7aa587ae26118a0611a2e4037a69141a3ffde`, release-trust contract `81a68395f9ad54ed651a84464a933ed7e57c9d32`, snapshot/compare helper `231c5a13e8d7e3cd337dbbad08e8f4434a943a38`, and helper regressions `d3107c617be3e8780a99bebaeba19a4f235ae6ea`.

Existing-owner-first creates no product/runtime owner. The source is represented as `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` under existing Fabushi repository quality/test-evidence governance. The shipping Electron updater is rejected because it owns product update/install/rollback rather than source dependency release/security advisory state; build-time acquisition inventory is evidence input rather than decision ownership. No platform-core production delta or ADR is introduced.

Spec-first CAS `d66c0d3bfa0e83148804b1bfb14036fd8176421e` extends root dependency-advisory governance, Test Strategy qualification and Evidence Contract requirements and adds the complete-read dossier. It preserves fresh exact snapshot identity, full dependency-input union, platform/configuration series, release/security/fork/backport applicability, release-trust assessment, stable pending findings, partial-failure preservation, prior run history and atomic latest projection while forbidding dependency mutation/build/test/backport/product-updater/release/deployment authority.

Authored but not run: `CONTRACT-TDRP-DEPENDENCY-SNAPSHOT-COVERAGE-001`, `CONTRACT-TDRP-DEPENDENCY-RELEASE-TRUST-001`, `CONTRACT-TDRP-DEPENDENCY-SECURITY-FORK-001`, `CONTRACT-TDRP-DEPENDENCY-FINDING-LIFECYCLE-001`, `FAULT-TDRP-DEPENDENCY-PARTIAL-RECOVERY-001`, and `INT-TDRP-DEPENDENCY-ADVISORY-OWNER-001`.

Exactly one independent source entry closes: `unknown 16160 -> 16159`, `unread 16133 -> 16132`, `omitted=0`; `responsibilities_total 133 -> 134`, `unmapped_responsibilities=0`. The next independent fixed-U blob in tree order is `.agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 119 / source-entry closure — OpenAI Dependency Watch host adapter (2026-11-01)

From desktop checkpoint parent `46241ddf29ca947ce1dae3fafae56e27ba13bf97`, the live migration branch was reread unchanged before spec publication; spec-first CAS published `832ce8cdbfba0fc24522f2e9bc001600d4fa6b20`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core exact commit `e92b51e23b39dae5719f033fa1f7a20d6579dc8c` was verified. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread `truncated=false` with 7,194 entries. Before closure the unique ledger was `source_entries_total=16202 / unknown=16159 / unread=16132 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`.

The exact `.agents/skills/dependency-watch/agents/openai.yaml@a61c118c41e3b50d6bc4bde12dc6d4c1b3756b5d` blob was read completely: 260 characters / 4 lines, fixed-U tree index 58. It is declarative OpenAI host metadata only: `display_name="Dependency Watch"`, `short_description="Track dependency releases and missing upstream fixes"`, and a `default_prompt` that invokes `$dependency-watch`, audits freshly fetched `origin/dev`, and orders pressing pending updates before the review-someday list. Repository search found no second in-repository parser/state owner; code-health, continue, perform-task and process-inbox OpenAI adapters use the same declarative schema.

Existing-owner-first attaches this source to the already implemented `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` owner under Fabushi repository quality/test-evidence governance. Shipping Electron updater, platform-core runtime, Product Tasks/Automations, Agent/Coordinator/Host/Runner, build-time inventory as decision authority and a copied Telegram/OpenAI dependency monitor remain rejected. No new responsibility, product/runtime owner, dependency pin, updater state, platform-core production delta or ADR is introduced.

Spec-first commit `832ce8cdbfba0fc24522f2e9bc001600d4fa6b20` extends existing qualification/evidence and adds `projects/telegram-desktop-rust/dossiers/dependency-watch-openai-host-adapter-complete-read.md`. Authored, not run: `CONTRACT-TDRP-DEPENDENCY-OPENAI-HOST-METADATA-001` and `INT-TDRP-DEPENDENCY-OPENAI-HOST-INVOCATION-001`.

Exactly one independent source entry closes under the existing responsibility: `unknown 16159 -> 16158`, `unread 16132 -> 16131`, `omitted=0`; `responsibilities_total=134`, `unmapped_responsibilities=0`. The next independent unrepresented fixed-U blob in tree order is `.agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 120 / source-entry closure — Dependency Watch patched-fork provenance (2026-11-01)

Live desktop parent `5fac5bd06a06de45410f6173c0d444b9eb08602a` was reread unchanged before publication; spec-first CAS published `5c8958895b5596bdab43a0d635e1b69bee125c74`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16158 / unread=16131 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`.

The exact `.agents/skills/dependency-watch/references/forks.md@29a7aa587ae26118a0611a2e4037a69141a3ffde` blob was read completely: 4,581 bytes / 74 lines, fixed-U tree index 60. Its direct workflow consumer is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`; fixed-U `watch.py` and `watch_test.py` contain no direct fork-reference parser/state ownership.

The contract separates consumed fork pin, current fork branch and original upstream/import provenance; treats tg_owt/tg_angle M123/M115 import subjects as discovery clues only; requires verified comparable ancestry or import mapping before lag counts; and classifies missing-fix candidates as confirmed applicable-and-missing, already present/backported, not applicable, or needs-investigation using actual fixing diffs, prerequisites, equivalent patch/semantic evidence, backend/build-flag/reachability and platform scope.

Fixed-U consumption was reread: prepare/Docker/Snap pin `desktop-app/tg_owt@07fb35bc33b3e79125a1beab7690e830f183f5d8`; Windows prepare pins `desktop-app/tg_angle@9fd174a91e6155d1006c5f427366b23006f5d899`, with explicit static ANGLE linkage in the Qt<6 branch. Fork-history anchors were verified as `tg_owt@5b6a5c00017c115f6a54b90b109354abc1444a01` ("M123+patches") and `tg_angle@3e2cbd93cb60a4f677af68bac4e3849176f688fa` ("M115"); the consumed pins are respectively 102 and 8 fork commits ahead of those anchors. These in-fork counts are not original-upstream lag metrics. Existing `build-time-acquisitions.json@e9c14ce86f8e8be5fa1231fabd47147c94ee887f` contains tg_owt but no tg_angle identity; that is an evidence-input gap, not proof of non-applicability.

Existing-owner-first reuses `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` under Fabushi repository quality/test-evidence governance. No product/runtime owner, second fork monitor, dependency mutation, backport application, updater state, platform-core production delta or ADR is introduced. `CONTRACT-TDRP-DEPENDENCY-FORK-IMPORT-HISTORY-001` and `CONTRACT-TDRP-DEPENDENCY-FORK-FIX-APPLICABILITY-001` were authored only and not run.

Exactly one independent source entry closes: `unknown 16158 -> 16157`, `unread 16131 -> 16130`, `omitted=0`; responsibilities remain 134 and unmapped remains 0. Next independent fixed-U blob in tree order is `.agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 121 / source-entry closure — Dependency Watch release-trust reference (2026-11-01)

Live desktop parent `b3b27a3e0975734b325fe637b352473a688dbc47` was reread unchanged before spec publication; spec-first CAS published `5358a6e9a978d2c2540086244f68d729f702142c`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16157 / unread=16130 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`.

The exact `.agents/skills/dependency-watch/references/release-trust.md@81a68395f9ad54ed651a84464a933ed7e57c9d32` blob was completely read: 5,849 bytes / 95 lines, fixed-U tree index 61. Its direct workflow consumer is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`; repository code search found no second direct consumer, and fixed-U `watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38` / `watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea` own no second release-trust parser/state lifecycle.

The release-trust contract requires publication date, canonical source, exact tag object plus peeled commit (or package/artifact identity), consumed source form, changed-tag/asset comparison, independent advisory/regression evidence, and explicit unavailable checks. Signatures, attestations, reproducibility evidence and checksums count only when actually verified; they establish integrity/attribution under their assumptions, not benign code. Unsigned releases are not automatically compromised and same-surface checksums are not independent proof. Relevant source/package deltas include install hooks, generated scripts, binary/test blobs, downloads, unexpected dependency changes, and publisher/signer/repository/release-process changes.

Decision states remain `update | backport | hold | skip | track`. Routine bugfix releases use a seven-day observation window by default, but elapsed time is not a safety oracle; confirmed applicable security urgency may justify immediate reviewed adoption/backport with rationale. Holds preserve missing evidence/review condition; skips name the exact rejected candidate and alternative rather than blacklisting the project; backports receive the same trust review. Durable fields are `decision`, `decision_reason`, `release_published_at`, `source_identity`, `trust_checks`, `regression_checks`, `unknown_checks`, and `review_after` or `review_condition`.

The XZ standing case is treated as a malicious-release regression fixture, not a permanent project ban. The reference's historical initial-snapshot example names prepare 5.4.5 and Docker 5.8.1; current fixed-U was re-established independently: prepare clones XZ v5.4 and checks out `05af863c3166cddeb4ab935829b14e995c8346b4`, while CentOS Docker clones `v5.8.4`. Current upstream `v5.8.4` is annotated tag object `9151b328e76bbc468aa64f85c741886b6227cec1` peeling to `d3e650e63c110e830fd5391e7f8b45df0b91d3da`; GitHub reports the tag signature valid, recorded only as provenance evidence and not safety clearance.

Existing-owner-first reuses `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` under Fabushi repository quality/test-evidence governance. No product/runtime owner, second release monitor, dependency mutation, updater state, platform-core production delta or ADR is introduced. `CONTRACT-TDRP-DEPENDENCY-RELEASE-PROVENANCE-001` and `CONTRACT-TDRP-DEPENDENCY-RELEASE-DECISION-001` were authored only and not run.

Exactly one independent source entry closes: `unknown 16157 -> 16156`, `unread 16130 -> 16129`, `omitted=0`; responsibilities remain 134 and unmapped remains 0. Next independent fixed-U blob in tree order is `.agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 122 / source-entry closure — Dependency Watch snapshot/compare helper (2026-11-01)

From desktop checkpoint parent `4431051c55093cf31669fb49aaa11f3fed223c5c`, the live migration branch was reread unchanged; spec-first CAS published `98ca5286b5fe0c2b63e8da0c4e9bb39ad48b176e`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was reread complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16156 / unread=16129 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`.

The exact `.agents/skills/dependency-watch/scripts/watch.py@231c5a13e8d7e3cd337dbbad08e8f4434a943a38` helper was read completely: 6,655 bytes / 191 logical lines, fixed-U tree index 63. The direct workflow caller is `.agents/skills/dependency-watch/SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8`, which invokes `snapshot` and `compare`; the direct code/test consumer is `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`. Current repository code search found the helper CLI path through the skill and direct `watch.snapshot`/`watch.compare` calls through that helper test; no second durable dependency-advisory state owner was found.

The helper's snapshot contract canonicalizes `--show-toplevel`; online mode fetches only origin `dev` to `refs/remotes/origin/dev` with no tags and no recursive submodules, records time only after a successful fetch, resolves `origin/dev^{commit}` once, enumerates the exact commit tree with `ls-tree -rz`, and reads selected bytes by blob OID with `cat-file`. Required manifests are `.gitmodules`, prepare.py, qt_version.py, the CentOS Dockerfile and snapcraft.yaml, each required as a readable regular file. Selected inputs also cover `Telegram/build/`, `snap/`, `.github/` and configured manifest/lock basenames; gitlinks are recorded separately for recursive inspection.

The storage contract resolves `--git-common-dir` and writes only under its shared `dependency-watch/runs` root, so linked worktrees converge on common report storage. Each run preserves exact source copies beneath `sources/`, explicit skipped reasons and run-local `snapshot.json`/`candidates.json`; non-UTF-8/NUL/non-regular content cannot silently count as readable required coverage. Candidate matches are line-oriented navigation aids only. Stable version comparison supports only optional `v`, `n`, or `openssl-` plus exactly three numeric components; prereleases, hashes, milestones, four-component versions and legacy-letter forms remain `unknown`.

Freshness/failure is fail-closed: `--no-fetch` is offline test mode only and records `fresh_fetch=false` with `audit_status=snapshot-only`. Git failure, 180-second timeout, OS error, missing required manifest or unreadable required manifest returns failure and reports that no fresh audit is available. A fetch failure happens before helper report storage creation, preventing stale remote-ref reuse as a fresh result; later readability failure may leave only run-local partial artifacts and cannot publish successful snapshot/candidate JSON.

Existing-owner-first reuses `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` under Fabushi repository quality/test-evidence governance. No product updater/runtime owner, copied Telegram Python monitor, ADR, dependency mutation or platform-core production delta is introduced. Spec-first commit `98ca5286b5fe0c2b63e8da0c4e9bb39ad48b176e` extends the existing Test Strategy/Evidence Contract and adds the helper complete-read dossier.

Authored but not run: `CONTRACT-TDRP-DEPENDENCY-WATCH-SNAPSHOT-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-STORAGE-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-CANDIDATES-001`, `CONTRACT-TDRP-DEPENDENCY-WATCH-VERSION-COMPARE-001`, and `FAULT-TDRP-DEPENDENCY-WATCH-FRESHNESS-001`.

Exactly one independent source entry closes: `unknown 16156 -> 16155`, `unread 16129 -> 16128`, `omitted=0`; responsibilities remain 134 and unmapped remains 0. Next fixed-U independent blob in tree order is `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.


## Round 123 / source-entry closure — Dependency Watch upstream regression fixture (2026-11-01)

From desktop checkpoint parent `e4259b9b7b1fabab51a9216632e067bf315cef78`, the live migration branch was reread unchanged before spec publication; spec-first CAS published `d6562479b20149b06fc9980baad5ea2cbefc2375`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive source tree is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16155 / unread=16128 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`.

The exact `.agents/skills/dependency-watch/scripts/watch_test.py@d3107c617be3e8780a99bebaeba19a4f235ae6ea` regression fixture was read completely: 4,347 bytes / 99 logical lines, fixed-U tree index 64. Its code dependency is sibling `watch`; executable consumer boundary is standard Python `unittest` discovery / `unittest.main()`. Production Dependency Watch `SKILL.md@482b3a8d0c939d4bb7bd32ab146e0cc988d29fd8` invokes `watch.py`, not this test file, so the source owns no advisory runtime or durable findings state.

`VersionTests.test_stable_ordering_and_prefixes` freezes ten comparison cases: patch/minor/major/older/same for supported stable three-component forms and `unknown` for prerelease, hash, milestone, four-component, and legacy-letter OpenSSL forms. `SnapshotTests` builds disposable remote/checkout fixtures and covers fresh remote bytes despite unfinished local edits, exact remote commit identity, synthetic gitlink capture, byte-stable checkout status/HEAD, no `latest.md`, fetch-failure refusal before helper storage, required-manifest failure, offline unverified freshness, and linked-worktree common storage.

Existing-owner-first reuses `TDRP-R9-REPOSITORY-DEPENDENCY-ADVISORY-001` under Fabushi repository quality/test-evidence governance. No new responsibility, owner, ADR, product runtime, shipping updater, copied Telegram test harness, dependency mutation, or platform-core production delta is introduced. The ledger's previously inaccurate `DependencyWatchTests` case aliases are corrected to exact `VersionTests` / `SnapshotTests` symbols.

Authored but not run: `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-VERSION-REGRESSION-001` and `CONTRACT-TDRP-DEPENDENCY-WATCH-UPSTREAM-SNAPSHOT-REGRESSION-001`.

Exactly one independent source entry closes: `unknown 16155 -> 16154`, `unread 16128 -> 16127`, `omitted=0`; responsibilities remain 134 and unmapped remains 0. The next fixed-U independent blob in tree order is `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation or runtime acceptance was performed.

## Round 124 / source-entry closure — repository perform-task single-task performer (2026-11-01)

From desktop checkpoint parent `d06d2df14f36de9990541f4a21af67ab3b2084bb`, the live migration branch was reread unchanged before spec publication; spec-first CAS published `c18b66863b7d94262e6e6b81385f67b82d89c073`. Desktop main was reread as `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; its recursive tree is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16154 / unread=16127 / omitted=0 / responsibilities_total=134 / unmapped_responsibilities=0`.

The exact `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` source was read completely: 12,740 bytes / 234 logical lines, fixed-U recursive-tree index 66. Its direct engine contracts were traced through `pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`, `phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c`, `mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`, `test-loop.md@bb51685e417759939cadc1e2e74b1c0199fcb8f0`, phase effort/delegation/Codex-leaf/build-lock/UI-driver references, and canonical `workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`. Direct invocation/consumer boundaries were read in root fixed-U `AGENTS.md`, Claude/Grok perform-task commands, OpenAI adapter metadata, `continue/SKILL.md`, and code-health skill/classes.

The portable responsibility is a source-neutral single-task repository performer: one canonical selected task; fail-closed Start/Retry/Resume ownership; source-lineage gating; resumable context/plan/assessment/owned implementation phases; split-required handoff without queue mutation; bounded behavior-preserving Q0/Q1/Q2 mechanical lane; independent general/risk-lens review with focused invalidation-based carry-forward and explicit convergence; claim-specific falsifiable evidence with TEST_FLAW/IMPL_BUG/UNRECOVERABLE separation and directness recovery; already-satisfied no-change proof; and canonical approved/genuine-blocked/split-required final publication. A source commit, worker completion, run cap, ambiguous evidence or interruption is not completion/acceptance.

Existing-owner-first reuses the current Fabushi repository development process authority plus repository quality/test-evidence governance established by the prior continue/task-process closure. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Telegram-specific performer/queue and a second review/evidence store are rejected as different/duplicate state domains. No new ADR and no platform-core production delta are required.

Spec-first commit `c18b66863b7d94262e6e6b81385f67b82d89c073` extends root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/perform-task-skill-complete-read.md`.

Authored but not run: `CONTRACT-TDRP-PERFORM-TASK-STATE-AUTHORITY-001`, `CONTRACT-TDRP-PERFORM-TASK-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-RESUME-001`, `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-LANE-001`, `CONTRACT-TDRP-PERFORM-TASK-REVIEW-CONVERGENCE-001`, `CONTRACT-TDRP-PERFORM-TASK-EVIDENCE-RECOVERY-001`, `CONTRACT-TDRP-PERFORM-TASK-PUBLICATION-001`, and `INT-TDRP-PERFORM-TASK-HOST-COMPOSITION-001`. FMEP-001 keeps every executable validation step deferred to Stage B.

Exactly one independent source entry closes: `unknown 16154 -> 16153`, `unread 16127 -> 16126`, `omitted=0`; `responsibilities_total 134 -> 135`, unmapped remains 0. The next fixed-U independent blob in tree order is `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation, runtime evidence campaign or acceptance was performed.



## Round 125 / source-entry closure — OpenAI perform-task host adapter (2026-11-01)

From desktop checkpoint parent `208dba1916b0ff233e7cadce2d033ce4a9472101`, the live migration branch was reread unchanged before closure publication. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive source tree remains complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16153 / unread=16126 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

The exact `.agents/skills/perform-task/agents/openai.yaml@f238b707da7f39a4293b17d112ddd344dcb8253a` source was read completely: 229 bytes / 4 logical lines, fixed-U recursive-tree index 69 (1-based including tree entries). It contains only `interface.display_name = "Perform AI Task"`, `interface.short_description = "Implement or resume one Telegram AI task"`, and a `default_prompt` that delegates to `$perform-task` to implement, adaptively review/test, and publish one named ai-tdesktop task.

Direct fixed-U consumers/boundaries were re-read in `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, Claude `perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e`, Grok `perform-task.md@313e94a3202c39aa10944b832b45ee80a762f509`, `continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c`, `code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f`, and root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`. They confirm that host adapters invoke exactly one canonical performer while queue selection, split routing, discovery/consolidation, task/review/evidence state and final publication remain owned elsewhere.

Existing-owner-first extends `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. No new responsibility, owner, ADR, shipping Product Tasks/Automations runtime, Agent/Coordinator/Host/Runner owner or platform-core production delta is introduced.

Spec-first CAS commit `208dba1916b0ff233e7cadce2d033ce4a9472101` extends root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/perform-task-openai-host-adapter-complete-read.md`.

Authored but not run: `CONTRACT-TDRP-PERFORM-TASK-OPENAI-HOST-METADATA-001` and `INT-TDRP-PERFORM-TASK-OPENAI-HOST-INVOCATION-001`.

Exactly one independent source entry closes: `unknown 16153 -> 16152`, `unread 16126 -> 16125`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next fixed-U independent blob in tree order is `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger and no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, visual/a11y, package, deployment, installation, runtime evidence campaign or acceptance was performed.

## Round 126 / source-entry closure — perform-task Computer Use testing adapter (2026-11-01)

From desktop checkpoint parent `ba5bcf2beafaaa8264ce46bda31eda36512f9af1`, the live branch was reread unchanged, then spec-first CAS commit `4e40c0b0653a1d81c117e17e48c16d0f6604610b` was published before this ledger closure. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16152 / unread=16125 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

Exact `.agents/skills/perform-task/references/computer-use-testing.md@b7fc92d604a55975f75047828d476e64667aa259` was read completely: 14,178 bytes / 14,166 decoded characters / 231 newline-delimited lines, recursive-tree index 71. Its complete contract covers driver policy (`auto | overlay-only | required`), capability gating, locked-session fallback, interrupted driver sessions, exact-app/process fencing, one-step hybrid handshake, unattended safety envelope, unavailable-result mapping, and durable accessibility/screenshot evidence.

Fixed-U direct references were verified in `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c` and `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`; fixed-U code search found no other direct reference. The shared `.agents/shared/test-loop.md@bb51685e417759939cadc1e2e74b1c0199fcb8f0` remains authoritative for task-derived scenarios, oracle, watchdog/crash and adaptive evidence assessment.

Existing-owner-first extends `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` under the Fabushi repository development process authority plus existing quality/UI-functional/visual/accessibility evidence governance. Portable behavior retained is driver selection only when physical interaction itself is the acceptance subject, exact isolated subject fencing, single desktop ownership, bounded ready/action/refresh handshake, local/disposable safety, contamination/recovery classification, safe deterministic fallback, and evidence precedence. Telegram portable-account, `-testagent`, Qt overlay/helper paths, provider-specific Computer Use API details and Telegram app naming remain source instrumentation provenance rather than a new Fabushi runtime.

Spec-first CAS commit `4e40c0b0653a1d81c117e17e48c16d0f6604610b` extends root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/perform-task-computer-use-testing-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SELECTION-001`, `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-SAFETY-001`, `CONTRACT-TDRP-PERFORM-TASK-UI-DRIVER-EVIDENCE-001`, and `FAULT-TDRP-PERFORM-TASK-UI-DRIVER-RECOVERY-001`.

Exactly one independent source entry closes: `unknown 16152 -> 16151`, `unread 16125 -> 16124`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, Computer Use/UI-driver run, visual/a11y execution, package, deployment, installation, runtime evidence campaign or acceptance was performed.

## Round 127 / source-entry closure — perform-task mechanical lane (2026-11-01)

From desktop checkpoint parent `f7d19783760cf870f07e9efd6567a9402109f0df`, the live migration branch was reread unchanged and spec-first CAS commit `4cccc0e70af8f219621cb19a523773e267f916dc` was published before this closure. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core checkpoint remains `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16151 / unread=16124 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

Exact `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef` was read completely: 9,933 bytes / 9,929 decoded characters / 194 newline-delimited lines, recursive-tree index 71. Q0 deletion, Q1 move and Q2 merge-identical-copies remain eligible only while behavior is unchanged and the class/path bounds remain exact. The lane rechecks stale findings against the exact task base, declares owned paths before edits, uses class-specific build/health/move/verify proof, restores or drops an item on class drift or failed proof, takes one fresh full-diff review with at most one focused fix/resume, and publishes either retained change evidence or an all-dropped already-satisfied result.

Direct fixed-U composition was re-read in `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`, `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f`, and `.agents/skills/code-health/references/classes.md@27847cae96838be7ddf68d3759dba89279f7cf0e`. They compose one repository performer/quality authority; code-health routes candidates but does not own product edits, and Q3-Q5 remain on the full pipeline.

Existing-owner-first extends `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Telegram-specific performer/queue and a second review/evidence store remain rejected duplicate/different domains. No new responsibility, ADR or platform-core production delta was introduced.

Spec-first CAS commit `4cccc0e70af8f219621cb19a523773e267f916dc` extends root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/perform-task-mechanical-lane-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-ELIGIBILITY-001`, `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-OWNED-EDIT-001`, `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-PROOF-DROP-001`, and `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-REVIEW-PUBLICATION-001`; the earlier umbrella `CONTRACT-TDRP-PERFORM-TASK-MECHANICAL-LANE-001` remains valid.

Exactly one independent source entry closes: `unknown 16151 -> 16150`, `unread 16124 -> 16123`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, code-health command, mechanical reviewer execution, visual/a11y run, package, deployment, installation, runtime evidence campaign or acceptance was performed.


## Round 128 / source-entry closure — perform-task phase prompts (2026-11-01)

From desktop checkpoint parent `f46416dd12f12839e5221217b3c8f2ec2b4e5909`, the live migration branch was reread unchanged and spec-first CAS commit `947aa8df76e857adaa681ce8520cf3e9dfa633a3` was published before this closure. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core checkpoint remains `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16150 / unread=16123 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

Exact `.agents/skills/perform-task/references/phase-prompts.md@f62400b00933fe3757d90b809e456b200389c65c` was read completely: 64,311-byte Git blob / 64,245 decoded characters / 1,215 newline-delimited lines, recursive-tree index 72. The source defines the exact prompt/result artifact and completion protocol for context/plan, independent assessment, implementation units, selected pre-review validation, initial general/specialist review, focused review, convergence, evidence-flaw recovery/campaign-cap assessment, Windows normalization, and host-specific prompt delivery.

Worker semantics remain subordinate to one stateful performer. A leaf never delegates, commits, or publishes canonical task state; its BLOCKED result is only a report. Delegation is selected only after the first real spawn succeeds. Immediate host depth/capacity/policy rejection may select the documented same-session fallback. Once delegated, incomplete Phase 1/3/4/6 work follows the bounded disposable-worker retry contract; retry exhaustion becomes a recoverable hard stop to the caller rather than a silent local rerun or task Block. Resume carries completed validated artifacts forward and restarts at the first incomplete boundary.

Review/evidence semantics were also closed exactly: initial reviewers remain independent; fixes invalidate only exact findings/invariants/lenses/evidence; focused review explicitly carries unaffected approvals; convergence selects exactly `CONTINUE | REPLAN_CURRENT | RESCOPE_REQUIRED | HARD_STOP` and cannot approve code. Evidence recovery records prior proof, failed assumption, forbidden repeated technique, a more direct instrument, reachability and oracle independence; it keeps retained implementation unless a sound check proves `IMPL_BUG`. Campaign caps are assessment boundaries, never approval or Block by count.

Direct fixed-U consumers/composition were verified in `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`, `.grok/ai-workflow-adapter.md@db447f204316d3991f6d20db3d8f097fcc021154`, and `.agents/shared/cross-review/prompts.md@4e155d2bd7e647c12df7b008c279969bfd65e44d`; shared phase-effort, Codex delegation and test-loop contracts remain dependencies.

Existing-owner-first extends `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. No new responsibility, ADR, Telegram-specific performer/worker pool, shipping Product Tasks/Automations runtime, Agent/Coordinator/Host/Runner owner or platform-core production delta was introduced.

Spec-first CAS commit `947aa8df76e857adaa681ce8520cf3e9dfa633a3` extends root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/perform-task-phase-prompts-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PERFORM-TASK-PHASE-PROMPT-ARTIFACT-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-ASSESSMENT-001`, `CONTRACT-TDRP-PERFORM-TASK-PHASE-REVIEW-FOCUS-001`, `FAULT-TDRP-PERFORM-TASK-PHASE-WORKER-RECOVERY-001`, and `FAULT-TDRP-PERFORM-TASK-EVIDENCE-CAMPAIGN-001`.

Exactly one independent source entry closes: `unknown 16150 -> 16149`, `unread 16123 -> 16122`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, code-health helper, reviewer execution, UI-driver, visual/a11y run, package, deployment, installation, runtime evidence campaign or acceptance was performed.
## Round 129 / source-entry closure — perform-task end-to-end pipeline (2026-11-01)

From desktop checkpoint parent `289f1f4d22d51f6874bfe8ca772fed287ca474f2`, live checkpoint authority was reread unchanged and spec-first CAS commit `18362bcb3da25c92e6820b4f4ed715c312b5b116` was published before this closure. Desktop main authority remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core checkpoint was independently reread as `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; recursive tree is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16149 / unread=16122 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

Exact `.agents/skills/perform-task/references/pipeline.md@3f897572e10438024edd2e316bd0be7eb4171493` was read completely: 64,933 Git bytes / 64,829 decoded characters / 1,145 newline-delimited lines, recursive-tree index 73. The source defines the authoritative end-to-end single-task performer composition: exact task/input/host resolution; canonical task-state and source-lineage preflight; safe source/submodule preparation and read-only preflight; task-local recovery refs/artifacts; context/plan and independent assessment; owned implementation phases; selected direct pre-review validation; adaptive independent/focused review and convergence; claim-driven evidence selection/recovery; already-satisfied no-change proof; exact final result schema; non-force canonical publication; and failure routing.

Direct fixed-U composition/callers were verified in `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, `.agents/skills/perform-task/references/mechanical-lane.md@e5e3cc88ba80ece6a20df7c4dd6002491d66a4ef`, and `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c`. Shared `project-context`, `phase-effort`, Codex delegation, test-loop, UI-driver and process-inbox workspace contracts remain dependencies. The performer alone owns task/commit/final-publication state; source-specific Telegram build/account/overlay/mirror mechanics are workflow instrumentation, not a Fabushi shipping runtime owner.

Existing-owner-first keeps `TDRP-R9-REPOSITORY-PERFORM-TASK-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. No new responsibility, ADR, Telegram-specific performer/worker pool, shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner owner or platform-core production delta was introduced.

Spec-first CAS commit `18362bcb3da25c92e6820b4f4ed715c312b5b116` extends root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/perform-task-pipeline-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-PREFLIGHT-001`, `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-RESUME-001`, `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-EVIDENCE-ADAPTER-001`, `CONTRACT-TDRP-PERFORM-TASK-PIPELINE-FINAL-PUBLICATION-001`, `FAULT-TDRP-PERFORM-TASK-PIPELINE-RECOVERY-001`, and `INT-TDRP-PERFORM-TASK-PIPELINE-END-TO-END-001`.

Exactly one independent source entry closes: `unknown 16149 -> 16148`, `unread 16122 -> 16121`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, code-health helper, reviewer execution, UI-driver, visual/a11y run, package, deployment, installation, runtime evidence campaign or acceptance was performed.

## Round 130 / source-entry closure — process-inbox repository intake (2026-11-01)

From desktop checkpoint parent `cfbdce82ae388fa763205e74590f73f52dcfe029`, live checkpoint authority was reread unchanged before the spec-first publication, and spec-first CAS commit `3d416de7a4813dad5b6ac973dc5962b884fc695c` was then reread as the live checkpoint before this closure. Desktop main authority remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; the recursive source tree is complete (`truncated=false`, 7,194 entries). Before closure the unique ledger was `source_entries_total=16202 / unknown=16148 / unread=16121 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

Exact `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d` was read completely: 22,475 Git bytes / 22,467 decoded characters / 441 newline-delimited lines, recursive-tree index 75. Its complete contract covers checkout-bound inbox acquisition; isolated publication worktree; immutable transaction snapshot/digest and resume; selective project/task lineage discovery; durable superseded/split history resolution; project-continuity and concrete independence routing; receipt-only branch-history requests; independently testable split/cohesion rules; collision-safe dated task ids; exact pristine `todo/type: implement` state without reservation or completion model; durable input/video-frame policy; source-lineage dependencies; explicit-path publication; digest-bound receipt/finalization; and pre/post-commit recovery.

Direct fixed-U callers/consumers were verified in root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32`, Claude `.claude/commands/process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21`, Grok `.grok/commands/process-inbox.md@5fcd9055de9b14a5163dc96a6c3551096898de18`, adjacent OpenAI adapter `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`, `.agents/skills/continue/SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c`, `.agents/skills/perform-task/SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, split reference `552ad09b2fa022b7df94481746a51167305bf846`, consolidation reference `bd301d93f8f670ad041b3db72330e17cbb50eb5c`, Claude/Grok workflow adapters, and shared helper `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`. The helper code was read selectively across repository identity/worktree isolation, transaction snapshot/digest, alias/split resolution, task/source-lineage state, explicit-path inbox/split/consolidation publication and finalize/abort recovery; it was not executed.

Existing-owner-first extends `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Telegram-specific queue/scheduler and a second receipt/ownership/acceptance store remain rejected duplicate/different domains. No new responsibility, ADR or platform-core production delta was introduced.

Spec-first CAS commit `3d416de7a4813dad5b6ac973dc5962b884fc695c` extends root `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/process-inbox-skill-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PROCESS-INBOX-ACQUISITION-SNAPSHOT-001`, `CONTRACT-TDRP-PROCESS-INBOX-ROUTING-LINEAGE-001`, `CONTRACT-TDRP-PROCESS-INBOX-TASK-STATE-SCHEMA-001`, `CONTRACT-TDRP-PROCESS-INBOX-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-PROCESS-INBOX-SPLIT-CONSOLIDATION-INTEROP-001`, `INT-TDRP-PROCESS-INBOX-PUBLICATION-FINALIZE-001`, and `FAULT-TDRP-PROCESS-INBOX-RECOVERY-001`.

Exactly one independent source entry closes: `unknown 16148 -> 16147`, `unread 16121 -> 16120`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, helper execution, inbox transaction, source-lineage command, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y run, package, deployment, installation, runtime evidence campaign or acceptance was performed.


## Round 131 / source-entry closure — Process AI Inbox OpenAI host adapter (2026-11-01)

From desktop checkpoint parent `a3a83766535bf3470f86eaefd17db67c9ff4ab0a`, live checkpoint authority was reread unchanged before spec publication, and spec-first CAS commit `4f48929a8f594ca3e1aaffb528aade87280deacb` was reread as the live checkpoint before this closure. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was recursively verified `truncated=false` with 7,194 entries / 6,859 blobs. Before closure the unique ledger was `source_entries_total=16202 / unknown=16147 / unread=16120 / omitted=0 / responsibilities_total=135 / unmapped_responsibilities=0`.

Exact `.agents/skills/process-inbox/agents/openai.yaml@b664f98ad4440de7a8074dfe790d80f3d2201546` was read completely: 234 Git bytes / 234 ASCII characters / 4 newline-delimited lines, recursive-tree index 77 (zero-based; 78th entry). It contains only `interface.display_name="Process AI Inbox"`, `interface.short_description="Route inbox notes into project-aware AI tasks"`, and `interface.default_prompt="Use $process-inbox to route and plan the local AI inbox while preserving existing project lineage."`.

Direct semantic target is `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`; the default prompt delegates through `$process-inbox` rather than embedding planner/router logic. Fixed-U Claude/Grok process-inbox wrappers `3bd1c22b06c735ff92cf2260bd893c9de2653f21` / `5fcd9055de9b14a5163dc96a6c3551096898de18` invoke the same parent skill with host-specific substitutions. Adjacent OpenAI adapters use the same declarative three-field schema, and repository-local search found declarative `default_prompt` definitions rather than a second in-repository host parser or durable state owner.

Existing-owner-first attaches this source to `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a copied OpenAI skill runtime, Telegram-specific inbox scheduler and a second queue/receipt/ownership/project-lineage/acceptance store remain rejected. No new responsibility, ADR or platform-core production delta was introduced.

Spec-first CAS commit `4f48929a8f594ca3e1aaffb528aade87280deacb` extends `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/process-inbox-openai-host-adapter-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PROCESS-INBOX-OPENAI-HOST-METADATA-001` and `INT-TDRP-PROCESS-INBOX-OPENAI-HOST-INVOCATION-001`.

Exactly one independent source entry closes: `unknown 16147 -> 16146`, `unread 16120 -> 16119`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/scripts/video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, OpenAI adapter invocation, process-inbox helper, inbox transaction, source-lineage command, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y run, package, deployment, installation, runtime evidence campaign or acceptance was performed.

## Round 132 / source-entry closure — process-inbox supplied-video helper (2026-11-01)

From desktop checkpoint parent `658e769364eca39177baf0453f3f6212263a0438`, spec-first CAS commit `910abf5ffd6f60f86d65ccde5c76d2ea99b37867` was published after rereading the live checkpoint. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` is recursive `truncated=false`, 7,194 entries / 6,859 blobs.

Exact `.agents/skills/process-inbox/scripts/video.py@977d75aca7aea0b6c74471d533a5ee0e2a156ac3` was read completely: 9,024 Git bytes / 9,024 ASCII characters / 292 newline characters with trailing newline, recursive-tree index 79 zero-based. The helper exposes `probe`, `extract`, and `sheet`. It binds a local reference by SHA-256/probe metadata, uses decoded `best_effort_timestamp_time` normalized to the first frame, validates sorted unique frame/range selection and `WxH+X+Y` crop syntax, extracts via ffmpeg with passthrough pacing and requested/decoded count equality, names frames by decoded index plus normalized milliseconds, and builds bounded labelled contact sheets through Pillow. Temporary raw decode paths are scoped to `TemporaryDirectory`; missing tools/dependencies, malformed selection/crop, no decoded frames, subprocess failure and count mismatch are fail-closed. Target-side move/save failures are not transactionally rolled back by the helper, so partial target output is explicitly non-authoritative.

The sole fixed-U direct invocation is the supplied-video section of `.agents/skills/process-inbox/SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`. Exact fixed-U `workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`, `workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`, Claude/Grok process-inbox wrappers and root `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` contain no direct `video.py`/ffmpeg/ffprobe invocation. The downstream perform-task pipeline consumes tracked `task.md` and referenced task `input/` files. The original video remains local in ignored transaction/backup state; full extraction is scratch-only; tracked durable reference is compact phase-complete crops/sheets plus key/context frames and task citations.

Existing-owner-first keeps `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. Shipping attachments/artifacts/resource ingress, media transfer/cache, Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core and any Telegram-specific video service remain rejected duplicate/different domains. No new responsibility, ADR or platform-core production delta was introduced.

Spec-first commit `910abf5ffd6f60f86d65ccde5c76d2ea99b37867` extends `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/process-inbox-video-helper-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-PROBE-PTS-001`, `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-FRAME-EXTRACT-001`, `CONTRACT-TDRP-PROCESS-INBOX-VIDEO-DURABLE-REFERENCE-001`, `INT-TDRP-PROCESS-INBOX-VIDEO-PERFORMER-HANDOFF-001`, and `FAULT-TDRP-PROCESS-INBOX-VIDEO-CLEANUP-001`.

Exactly one independent source entry closes: `unknown 16146 -> 16145`, `unread 16119 -> 16118`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no build, typecheck, lint, validator, generator, dependency resolution, video helper/ffmpeg/ffprobe/Pillow execution, process-inbox transaction, source-lineage command, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was performed.

## Round 133 / source-entry closure — process-inbox workspace helper (2026-11-01)

From desktop checkpoint parent `fbc9949492825cad7b959bfa2750a07f53973396`, live authority was reread before the Stage-A spec write, and spec-first CAS commit `3d3d38156ad6327e37408013745b77c003ce4e02` was published with `force=false` and `expected_sha=fbc9949492825cad7b959bfa2750a07f53973396`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` is recursive `truncated=false`, 7,194 entries / 6,859 blobs.

Exact `.agents/skills/process-inbox/scripts/workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d` was read completely: 185,107 Git bytes / decoded characters, 6,122 newline characters with a trailing newline, 6,123 split lines and 221 class/function declarations, recursive-tree index 80 zero-based. The helper concretely implements checkout/machine identity; isolated slot/inbox/routing/code-health linked worktrees; canonical task state with Start/Retry ownership transitions; approved-source ancestry and local base/green/run recovery refs; recursive source/submodule/worktree safety; explicit owned-path source commits; content-digested immutable inbox snapshots; explicit-path refreshed/rebased publication with post-rebase semantic validation; approval-ref recovery; and digest-checked finalize/abort behavior. Its optional force-updated private `ai-<checkout-tag>` mirror is recovery-only and is not canonical publication authority.

Direct fixed-U consumers were traced to process-inbox `SKILL.md@73bd3c360232679a0ebfe3b990073ecc4f322b7d`, perform-task `SKILL.md@590e6660cfd8782b37dbc3c6959944f53bb4541c`, continue `SKILL.md@aeb81dd57018abbc8f03ea3dbd34f91952128db5c`, code-health `SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f`, performer pipeline `3f897572e10438024edd2e316bd0be7eb4171493`, split `552ad09b2fa022b7df94481746a51167305bf846`, consolidation `bd301d93f8f670ad041b3db72330e17cbb50eb5c`, shared build-lock recovery `ec75573e7efe74e9c9a344f3e66f2fc43ac7680f`, Claude workflow adapter `370ff84a1ab4c080b15b601a54162b0630c4f4df`, and direct regression consumer `workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`. The test blob remains independently unread/unclosed in this round.

Existing-owner-first keeps `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, shipping Storage/Recovery/resource owners, platform-core production runtime, a Telegram-specific workspace scheduler, and any second task/receipt/lineage/evidence store remain rejected. No new responsibility, ADR or platform-core production delta was introduced.

Spec-first commit `3d3d38156ad6327e37408013745b77c003ce4e02` extends `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/process-inbox-workspace-helper-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-IDENTITY-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKTREE-ISOLATION-001`, `CONTRACT-TDRP-PROCESS-INBOX-TRANSACTION-DIGEST-001`, `CONTRACT-TDRP-PROCESS-INBOX-TASK-STATE-TRANSITION-001`, `CONTRACT-TDRP-PROCESS-INBOX-SOURCE-LINEAGE-REFS-001`, `INT-TDRP-PROCESS-INBOX-WORKSPACE-PUBLICATION-001`, and `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-RECOVERY-001`.

Exactly one independent source entry closes: `unknown 16145 -> 16144`, `unread 16118 -> 16117`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no workspace helper/test execution, build, typecheck, lint, validator, generator, dependency resolution, source-lineage command, Git publication simulation, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was performed.



## Round 134 / source-entry closure — process-inbox workspace regression suite (2026-11-01)

From desktop checkpoint parent `24d2801162c92226483a578e2b1f2ce84cfa0967`, live checkpoint authority was reread unchanged before spec publication. Spec-first CAS commit `d03b8c4896ce268b14f32c5def49ed9705b325e5` was then published with `force=false` and `expected_sha=24d2801162c92226483a578e2b1f2ce84cfa0967`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was recursively reread `truncated=false` with 7,194 entries / 6,859 blobs.

Exact `.agents/skills/process-inbox/scripts/workspace_test.py@79f943a99809bc6acf173872d9a780346648b5cf` was read completely: 214,649 Git bytes / decoded characters, 6,447 newline characters with a trailing newline, 6,448 split lines, 254 class/function declarations, 10 class declarations and 140 `test_*` methods, recursive-tree index 81 zero-based. It directly imports `workspace` and terminates in `unittest.main()`; it is a repository regression consumer, not a shipping runtime owner.

Regression coverage directly pins inbox/task-slot isolation, linked-worktree/source-preparation safety, narrow publication paths, deletion/non-ASCII path preservation, retryable publication races, canonical-master receipt/finalize fencing, atomic Start/Retry transitions, source-lineage ancestry, batch race/no-preclaim semantics, owned-path source commits, split carried-work handoff and approval-ref recovery. The exact suite directly calls prepare 6 times, finalize 3 times, Start 11 times, Retry 2 times, inbox-publish 3 times, generic publish once and source_lineage_report 3 times. It contains zero direct `workspace.command_abort` calls and consumes prepare-produced digests without directly invoking the digest algorithm, so abort and digest implementation remain explicit separately authored obligations rather than inferred upstream coverage.

Direct composition is `workspace.py@3cd99d173fcc3a6e293cfe36058a5fd9e492887d` via `import workspace`, plus the file's own unittest entrypoint. Exact fixed-U process-inbox, perform-task, continue and code-health skills, performer pipeline, split/consolidation references and shared build-lock document surveyed for this closure contain no direct `workspace_test` / `workspace_test.py` reference. Existing-owner-first therefore keeps `TDRP-R9-REPOSITORY-CONTINUE-TASK-PROCESS-AUTHORITY-001` under Fabushi repository development process authority + repository quality/test-evidence governance; no new responsibility, ADR, shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner owner or platform-core production delta is introduced.

Spec-first commit `d03b8c4896ce268b14f32c5def49ed9705b325e5` extends `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/process-inbox-workspace-regression-suite-complete-read.md`. Authored but not run: `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-IDENTITY-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-WORKTREE-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-TRANSACTION-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-TASK-STATE-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-FINALIZE-ABORT-001`, `INT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-PUBLICATION-001`, and `FAULT-TDRP-PROCESS-INBOX-WORKSPACE-REGRESSION-RECOVERY-001`.

Exactly one independent source entry closes: `unknown 16144 -> 16143`, `unread 16117 -> 16116`, `omitted=0`; responsibilities remain 135 and unmapped remains 0. The next independent fixed-U blob in tree order is `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no workspace helper/test execution, source-lineage command, Git publication simulation, build, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was performed.


## Round 135 / source-entry closure — intent-aware rebase skill (2026-11-01)

From desktop checkpoint parent `6e8822013bbfc6bd7222381525eb802133d70317`, live checkpoint authority was reread unchanged before spec publication. Spec-first CAS commit `2ced9915727beb46da1bf42c1d278b98f751f17f` was published with `force=false` and `expected_sha=6e8822013bbfc6bd7222381525eb802133d70317`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was recursively reread `truncated=false` with 7,194 entries / 6,859 blobs.

Exact `.agents/skills/rebase/SKILL.md@0660dc78b26614e40d7151d1da59241b1bad5430` was read completely: 15,114 Git bytes / 15,036 decoded characters, 321 newline characters with trailing newline, 322 split lines and 12 headings, recursive-tree index 83 zero-based. It defines one current-checkout intent-aware rebase: resolve ONTO/REPLAY_BASE/OLD_TIP and actual replay list; refuse sibling-worktree ownership, dirty or ambiguous state; keep remote targets as local snapshots unless fetch is explicitly authorized; resolve conflicts from both histories instead of ours/theirs; review rerere and empty commits explicitly; preserve reflog recovery and unknown resume state; verify the whole replay with range-diff/log/status; and never push or force-push.

Direct fixed-U host callers are Claude `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120` and Grok `.grok/commands/rebase.md@dc095e3c837aba1b30ff35e1bb92952cef5f1780`; both read the shared skill completely and explicitly bypass their normal AI workflow/task-publication adapters. Fixed-U `AGENTS.md@b3ce86744f7d889a5dee63dbaefa3353801c4c32` and `REVIEW.md@e65be8231a8e0f50ee86b20ef16120d58ea40376` are repository/style dependencies. Performer pipeline `3f897572e10438024edd2e316bd0be7eb4171493` is a boundary consumer that forbids backport/cherry-pick/rebase/merge work inside canonical task performance. No fixed-U `.agents/skills/rebase/agents/openai.yaml` exists.

Existing-owner-first adds responsibility `TDRP-R9-REPOSITORY-INTENT-AWARE-REBASE-AUTHORITY-001` to existing Fabushi repository development process authority + repository quality/test-evidence governance. Continue/perform-task/process-inbox canonical state machines, shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Telegram-specific rebase runtime and any second task/publication store remain rejected. No new owner, ADR or platform-core production delta is introduced.

Spec-first commit `2ced9915727beb46da1bf42c1d278b98f751f17f` extends `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/rebase-skill-complete-read.md`. Authored but not run: `CONTRACT-TDRP-REBASE-SCOPE-LINEAGE-001`, `CONTRACT-TDRP-REBASE-CONFLICT-INTENT-001`, `CONTRACT-TDRP-REBASE-RERERE-EMPTY-001`, `CONTRACT-TDRP-REBASE-RESUME-ABORT-RECOVERY-001`, `CONTRACT-TDRP-REBASE-NONINTERACTIVE-HOST-001`, `INT-TDRP-REBASE-HOST-COMPOSITION-001`, `INT-TDRP-REBASE-PUBLICATION-BOUNDARY-001`, `FAULT-TDRP-REBASE-UNKNOWN-CONTRADICTION-001`, and `SEC-TDRP-REBASE-NO-FORCE-PUBLISH-001`.

Exactly one independent source entry closes: `unknown 16143 -> 16142`, `unread 16116 -> 16115`, `omitted=0`; responsibilities `135 -> 136` and unmapped remains 0. The next independent fixed-U blob in tree order is `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no rebase/fetch/range-diff/source-lineage operation, Git publication simulation, build, compile, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was performed.


## Round 136 / source-entry closure — root clang-tidy static-analysis policy (2026-11-01)

From desktop checkpoint parent `52591d7976c5ec113ff9bcb8089a739e767c23a7`, live checkpoint authority was reread unchanged before spec publication. Spec-first CAS commit `43415c040400f2c36daf33cf43b34ab7b45b0a3b` was published with `force=false` and `expected_sha=52591d7976c5ec113ff9bcb8089a739e767c23a7`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` was recursively reread `truncated=false` with 7,194 entries / 6,859 blobs.

Exact `.clang-tidy@5610bd13664ca77549993def241eb3edaffeff7a` was read completely: 1,264 bytes / 1,264 decoded ASCII characters, 38 newline characters with trailing newline, 39 split lines, recursive-tree index 84 zero-based. It enables only `bugprone-unsafe-functions`, makes that check warning-as-error, defines the complete Qt-helper `CustomFunctions` replacement table including `qBound -> std::clamp(low, value, high)` and `qRound -> int(base::SafeRound(...))`, and sets empty `HeaderFilterRegex` because header findings repeat and submodules are not owned by this policy.

Direct fixed-U policy consumer `tools/rules/hook.py@287b36c948566659970651226c92f7b8b056b434` opens root `.clang-tidy` and derives the banned helper/replacement pairs for edit and staged-diff checks. The clang-tidy execution path is documented by `tools/clangtidy/README.md@2e64a408cb2d76aed0425706e321a24bb58f350f` and compile database preparation `tools/clangtidy/compile_db.py@16a99ac4ad860cc37ee7c84bfb82cf899fb1cced`; the README requires clang-tidy 20+ because older versions silently ignore `CustomFunctions`. Secondary callers are `.claude/settings.json@d47520dc0e3b7e8d6067456e33062140c75f7c79` and `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361`; `tools/rules/test_hook.py@dce99f3507e4ec9f71c8bbd5360e9dde9b62d45f` is regression coverage only.

Existing-owner-first adds responsibility `TDRP-R9-REPOSITORY-STATIC-ANALYSIS-POLICY-001` under existing Fabushi repository quality/test-evidence governance. A Telegram-specific scanner, second findings ledger/task queue, shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner and platform-core runtime remain rejected. No new owner, ADR or platform-core production delta is introduced.

Spec-first commit `43415c040400f2c36daf33cf43b34ab7b45b0a3b` extends `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/clang-tidy-quality-policy-complete-read.md`. Authored but not run: `CONTRACT-TDRP-STATIC-ANALYSIS-CONFIG-SCOPE-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-CUSTOM-FUNCTIONS-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-HOOK-PARITY-001`, `CONTRACT-TDRP-STATIC-ANALYSIS-TOOLCHAIN-VERSION-001`, `INT-TDRP-STATIC-ANALYSIS-QUALITY-COMPOSITION-001`, and `FAULT-TDRP-STATIC-ANALYSIS-CONFIG-DRIFT-001`.

Exactly one independent source entry closes: `unknown 16142 -> 16141`, `unread 16115 -> 16114`, `omitted=0`; responsibilities `136 -> 137` and unmapped remains 0. The next unrepresented fixed-U blob in tree order is `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no clang-tidy, hook/test execution, configure/build, compile, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was performed.


## Round 137 / source-entry closure — Claude AI workflow host adapter (2026-11-01)

From desktop checkpoint parent `8705bf5da06690626271f1cf7dc5ea8fdb5888b9`, live checkpoint authority was reread unchanged before spec publication. Spec-first CAS commit `e16e4f418eb95718cc9f52fbbb252e8be5ca4066` was published with `force=false` and `expected_sha=8705bf5da06690626271f1cf7dc5ea8fdb5888b9`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` is recursive `truncated=false` with 7,194 entries / 6,859 blobs.

Exact `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` was read completely: recursive-tree index 86 zero-based, 13,188 UTF-8 bytes / 13,166 decoded characters, 215 newline characters with trailing newline, 216 split lines. It is explicit-load only and leaves shared `.agents/skills/` task selection, source changes, evidence, commits, resumability and publication authoritative while adapting Claude Agent delegation, foreground completion/recovery, Codex/native review-provider selection, model reporting, text handling, source-lineage timing, UI-driver capability and the bounded routing/consolidation lane.

Direct fixed-U loaders are `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8`, `perform-task.md@98ec6ed4aa51d3cac043a70a23adc6834b147f8e`, `process-inbox.md@3bd1c22b06c735ff92cf2260bd893c9de2653f21` and `code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`. `.claude/commands/rebase.md@df14e750067aeac1258a92068886cd2b33f83120` explicitly does not load the adapter because branch integration is outside AI queue work. Existing shared continue/perform-task/process-inbox/phase-effort/Codex-leaf contracts remain the durable process authorities.

Existing-owner-first adds cross-cutting responsibility `TDRP-R9-REPOSITORY-AI-WORKFLOW-HOST-ADAPTER-001` under existing Fabushi repository development process authority + repository quality/test-evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Claude-specific queue/performer/review store and a second publication/evidence truth remain rejected. No new state owner, ADR or platform-core production delta is introduced.

Spec-first commit `e16e4f418eb95718cc9f52fbbb252e8be5ca4066` extends `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/claude-ai-workflow-adapter-complete-read.md`. Authored but not run: `CONTRACT-TDRP-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CLAUDE-DELEGATION-SINGLE-WRITER-001`, `CONTRACT-TDRP-CLAUDE-LEAF-COMPLETION-RECOVERY-001`, `CONTRACT-TDRP-CLAUDE-REVIEW-PROVIDER-001`, `CONTRACT-TDRP-CLAUDE-SOURCE-LINEAGE-001`, `CONTRACT-TDRP-CLAUDE-TEXT-HANDLING-001`, `INT-TDRP-CLAUDE-HOST-PROCESS-COMPOSITION-001`, `INT-TDRP-CLAUDE-PUBLICATION-BOUNDARY-001`, and `FAULT-TDRP-CLAUDE-DUPLICATE-WRITER-RECOVERY-001`.

Exactly one independent source entry closes: `unknown 16141 -> 16140`, `unread 16114 -> 16113`, `omitted=0`; responsibilities `137 -> 138` and unmapped remains 0. The next unrepresented fixed-U blob in tree order is `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no Claude Agent/Codex leaf, workspace/source-lineage/routing helper, build, compile, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was executed.


## Round 138 / source-entry closure — Claude code-health command adapter (2026-11-01)

From desktop checkpoint parent `be1db0ca4d4e30a9a5e32b2b288d333d7518ffa8`, live checkpoint authority was reread unchanged before spec publication. Spec-first CAS commit `9bbd2014883783dd7fab15af05d97e94fd46bf2e` was published with `force=false` and `expected_sha=be1db0ca4d4e30a9a5e32b2b288d333d7518ffa8`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains exact `bhrumom/fabushi-platform-core@e92b51e23b39dae5719f033fa1f7a20d6579dc8c`. Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree `91f2b083ab2877c0a17d07868eabec362a5cfca6` is recursive `truncated=false` with 7,194 entries / 6,859 blobs.

Exact `.claude/commands/code-health.md@bcabd4bd5b5a1a2ed09b40a7f1fbdd502042329a` was read completely: recursive-tree index 88 zero-based, 598 ASCII/UTF-8 bytes / 598 decoded characters, 19 newline characters with trailing newline, 20 split lines. Its front matter exposes the Claude host command capability envelope; its body requires complete reads of the generic Claude workflow adapter and shared code-health skill, maps no-arg/`scan`, `survey [area]`, and `check [base]`, passes raw `$ARGUMENTS`, and fences proposal routing to invocations explicitly containing `route`; the scheduled form is `scan route`.

Direct dependencies are `.claude/ai-workflow-adapter.md@370ff84a1ab4c080b15b601a54162b0630c4f4df` and `.agents/skills/code-health/SKILL.md@101e69bc18d8f37c73d3b077f04d266bd939360f`. Direct consumer is Claude Code command discovery/user `/code-health` invocation; the shared skill documents scheduled `/code-health scan route` and manual `/code-health survey <area>`. Repository search found no fixed-U file-level import/reference to this command wrapper by path. `.agents/skills/code-health/agents/openai.yaml@59bb6304445f21e2a3ee1a97a69a8e231f934d30` is a peer `$code-health` host adapter, not a caller; root `CLAUDE.md@f8564d18cef4b9c545767025995d31deb3fa7155` only points to canonical `AGENTS.md`.

Existing-owner-first adds responsibility `TDRP-R9-CODE-HEALTH-CLAUDE-COMMAND-ADAPTER-001` under existing repository quality-health command orchestration + repository AI workflow host-adapter authority + repository quality/test-evidence governance. Shipping Product Tasks/Automations, Agent/Coordinator/Host/Runner, platform-core runtime, a Claude-specific scanner/findings ledger/task queue/approval store/publication store and any second workflow authority remain rejected. No new durable state owner, ADR, service, shipping entrypoint or platform-core production delta is introduced.

Spec-first commit `9bbd2014883783dd7fab15af05d97e94fd46bf2e` extends `AGENTS.md`, `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`, `projects/fabushi-communication-platform/quality/evidence-contract.md`, and adds `projects/telegram-desktop-rust/dossiers/claude-code-health-command-complete-read.md`. Authored but not run: `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-HOST-AUTHORITY-001`, `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-INVOCATION-001`, `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-DELEGATION-001`, `CONTRACT-TDRP-CODE-HEALTH-CLAUDE-ROUTING-001`, `INT-TDRP-CODE-HEALTH-CLAUDE-COMPOSITION-001`, and `FAULT-TDRP-CODE-HEALTH-CLAUDE-ROUTE-FENCE-001`.

Exactly one independent source entry closes: `unknown 16140 -> 16139`, `unread 16113 -> 16112`, `omitted=0`; responsibilities `138 -> 139` and unmapped remains 0. The next unrepresented fixed-U blob in tree order is `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8`.

Stage A zero-CI discipline remained intact: no GitHub Actions/check/status/workflow query or trigger; no code-health scan/survey/check, Claude Agent/Codex leaf, workspace/source-lineage/routing helper, build, compile, typecheck, lint, validator, generator, dependency resolution, unit/contract/integration/E2E, reviewer, UI-driver, visual/a11y, package, deployment, installation, runtime evidence or acceptance was executed.


## Round 139 / source-entry closure — Claude continue command adapter (2026-11-01)

From desktop checkpoint parent `2c24fdb00934402bc6ac204d6856fe1e1a41a5c7`, spec-first CAS commit `f90fd0fc1ef26cc7bd26215c6e11c2e09e2a5c5b` was published with `force=false` and `expected_sha=2c24fdb00934402bc6ac204d6856fe1e1a41a5c7`. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`; fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` with recursive tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, `truncated=false`, 7,194 entries / 6,859 blobs.

Exact `.claude/commands/continue.md@e81c86031f2e501f30d2b6ed908fb03d648324c8` was read completely: tree index 89 zero-based, 3,700 ASCII/UTF-8 bytes / 3,700 decoded characters, 67 newline characters, trailing newline, 68 split lines. It is a thin Claude host wrapper over the shared continue scheduler: no-spec is read-only queue/report/question; specified runs freeze exactly one batch with active-task precedence and admit later work only through canonical split replacements or follow-ups produced by this invocation.

Direct dependencies are the exact generic Claude adapter and shared continue skill; direct host consumer is Claude Code command discovery/user `/continue`. No fixed-U file-level import/reference to the wrapper was found. OpenAI `$continue` and Grok `/continue` are peer host adapters, not callers. Existing-owner-first adds `TDRP-R9-CONTINUE-CLAUDE-COMMAND-ADAPTER-001` over existing continue scheduler + Claude host-adapter + quality/test-evidence governance; no new durable state owner, ADR, service, shipping runtime or platform-core production delta.

Source-lineage timing stays parent-owned at startup/pre-Phase-1, post-Phase-1 mismatch stays task-local, one stateful performer is active, split routing is fresh/foreground, routing/consolidation stay in the existing bounded lane, and review provider is selected once (explicit Codex opt-out => native, otherwise codex) without changing canonical semantics.

Spec-first commit `f90fd0fc1ef26cc7bd26215c6e11c2e09e2a5c5b` extends `AGENTS.md`, `TEST_STRATEGY.md`, `evidence-contract.md`, and adds the complete-read dossier. Authored but not run: six contract cases, one integration case and one fault-recovery case for host authority, batch freeze, source lineage, delegation, review provider, routing lane, composition and duplicate-scheduler recovery.

Exactly one independent source entry closes: `unknown 16139 -> 16138`, `unread 16112 -> 16111`, `omitted=0`; responsibilities `139 -> 140`, unmapped remains 0. Next fixed-U tree blob: `.claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5`.

Stage A zero-CI discipline remained intact: no Actions/check/status/workflow query or trigger; no queue/helper, Agent/Codex leaf, source-lineage/routing/consolidation execution, build, compile, typecheck, lint, validator, generator, dependency resolution, test, reviewer, UI-driver, visual/a11y, packaging, deployment, installation, runtime evidence or acceptance was executed.


## Round 140 / source-entry closure — crash-report process command (2026-11-01)

From desktop checkpoint parent `e35d300f6e915d88bf7788cf6851bab530a4518a`, spec-first CAS `b996f372412e78272a180c8ed8dc878e73565aa7` authored repository process/quality contracts and the complete-read dossier. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`; fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` with complete recursive tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, 7,194 entries / 6,859 blobs.

Exact `.claude/commands/crash-reports-process.md@5ac555406670721694e78c4e1e6b2a96b3e98fb5` was read completely at index 90 zero-based: 13,431 UTF-8 bytes, 13,379 decoded characters, 268 newlines, trailing newline. Direct helper `.claude/scripts/crash_triage.py@2db6cc14d8e529c4e61c5dcf169aade31fd35c58` was read for claim, grouping, ledger, sweep and recovery semantics. Direct consumer is Claude Code command discovery/user `/crash-reports-process`; repository search found no path import. The generic Claude workflow adapter is not imported because this command never explicitly loads it.

Existing-owner-first adds `TDRP-R9-CRASH-REPORT-PROCESS-CLAUDE-COMMAND-ADAPTER-001` under repository development-process plus quality/test-evidence governance. It preserves complete-triple claim, incomplete-report retention, resumable processing markers, move-not-delete sweep, summary-only parent context, waves of at most ten, isolated one-group investigations, adversarial verification, exact verdict routing, common-path performance fencing, serial current-branch application, per-wave record/reconcile and dry-run no-mutation. No shipping crash runtime, Product Tasks/Automations owner, Agent/Coordinator/Host/Runner owner, platform-core service, ADR or duplicate durable state is introduced.

Seven contract cases, one integration case and two fault-recovery cases are authored only and not run. Exactly one source entry closes: `unknown 16138 -> 16137`, `unread 16111 -> 16110`, `omitted=0`; responsibilities `140 -> 141`, unmapped remains 0. Next fixed-U tree blob: `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361`.

Stage A zero-CI discipline remained intact: no Actions/check/status/workflow query or trigger and no crash helper, Agent/Workflow, build, test, validator, packaging, deployment, installation, runtime evidence or acceptance was executed.


## Round 141 / source-entry closure — Claude cross-review command adapter (2026-11-01)

From desktop checkpoint parent `f21130c8dd374d4dc5101ea8ca92f229224e90c6`, spec-first CAS `91a8a6a5134c44bfd10441a0b7719c9822322972` authored the source-neutral host-adapter/quality contracts and complete-read dossier. Desktop main remains `9993ca8e107f7e019a8201e5ab74fa54e82159f2`; platform-core remains `e92b51e23b39dae5719f033fa1f7a20d6579dc8c`; fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4` with complete recursive tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`, 7,194 entries / 6,859 blobs.

Exact `.claude/commands/cross-review.md@7769d03de798cde66987cd41303d2ea529321361` was read completely at tree index 91 zero-based: 3,446 UTF-8 bytes / 3,446 decoded characters, 74 newline characters, trailing newline. Required complete-read dependencies are protocol `51dabe142d5c3c7247e4050790c4e74b40f35b27`, prompts `4e155d2bd7e647c12df7b008c279969bfd65e44d`, and Codex-leaf contract `11c59b68df43e442e596c3052296fc51a044d46a`; direct helpers are `codex_leaf.py@313c4941c61a4ec0e3b3b65cdaa59cc213916fdd` and `cross_review_scope.py@d9fafdf9fa0857cc83f0ccb49d15d4fad8782768`. Generic `.claude/ai-workflow-adapter.md` is a negative dependency because it is explicit-load-only and this command does not load it.

Direct host consumer is Claude Code command discovery/user `/cross-review`; the shared protocol's reference to the command names its current runner, not a second invocation consumer. Existing-owner-first adds only `TDRP-R9-CROSS-REVIEW-CLAUDE-COMMAND-ADAPTER-001` over the already-closed source-neutral cross-review protocol/scope/prompts, repository quality/evidence ledger, root host-command authority and AI-worker lifecycle. No shipping review runtime, Product Tasks/Automations owner, Agent/Coordinator/Host/Runner owner, platform-core service, ADR or duplicate session/findings/arbitration store is introduced.

The command-specific provider fence is explicit: Codex admission failure stops instead of falling back to native/Claude review. Round 1 uses fresh read-only general plus five lens reviewers; exact foreground wait is completion; a failed/verdict-incomplete leaf gets at most one inactive-proven replacement; twice-failed lens becomes an explicit gap, twice-failed general stops; synthesis resumes the completed general thread. Every finding is arbitrated in the durable ledger before optional edits, review-only forbids edits, later rounds focus on accepted fixes, and resume continues the existing frozen session from its first incomplete step. Source-requested build/comment-hook qualification is deferred by FMEP-001.

Eight contract cases, one integration case and two fault-recovery cases are authored only and not run. Exactly one source entry closes: `unknown 16137 -> 16136`, `unread 16110 -> 16109`, `omitted=0`; responsibilities `141 -> 142`, unmapped remains 0. Next fixed-U tree blob: `.claude/commands/icon.md@479e23eceb019477fd7f2ffe4e2c13aadcfc556e`.

Stage A zero-CI discipline remained intact: no Actions/check/status/workflow query or trigger; no Codex check/leaf, scope helper, reviewer, synthesis, source fix, Debug build, comment hook, test, validator, packaging, deployment, installation, runtime evidence or acceptance was executed.
