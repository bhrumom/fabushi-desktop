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
