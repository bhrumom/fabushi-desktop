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
