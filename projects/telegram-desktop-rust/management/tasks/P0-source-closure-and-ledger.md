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
