# Round 41 — canonical account-bound Telegram embedded client handoff

Date: 2026-11-01
Checkpoint base before this round: `0db0728915ad431970e34dbd0176b477f043da0a`
Production implementation commit object: `e8a8b7726679c06862205ef21fdfbfc0c5aca4ed`
Fixed Telegram Desktop U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Stage A guard

This round used Git/GitHub source/tree/blob reads and manual source review plus production/test/ledger authoring. It did **not** query or trigger Actions/CI and did not run build, typecheck, lint, schema validation, generators, unit/contract/integration/E2E tests, packaging, deployment, installation, benchmark, fuzz, or runtime acceptance.

## Canonical account and source responsibility

The shipping Electron product owner is `source/product/fabushi/fabushi-account-adapters.ts` + `fabushi-account-service.ts`. It settles the Fabushi account before Coordinator/Host startup, stores the durable product session through the existing secure secret owner, atomically writes the private `0600` Host credential, refreshes/replaces that credential under an account epoch fence, and removes it on logout.

Fixed-U `Telegram/SourceFiles/main/main_account.cpp@2721c7ac12effe454d398610d26d638fadaa1f68` shows the applicable upstream lifecycle: `Main::Account` owns local account storage, restores serialized authorization/session identity, starts the MTP instance, persists key/config changes, and tears down/reset local state on logout. `Telegram/SourceFiles/storage/storage_account.cpp@afe6f61af833fd32ba02a3bd8a48f6801cb583d5` supplies the adjacent encrypted account-local persistence context.

## Implemented single-owner handoff

Production commit `e8a8b7726679c06862205ef21fdfbfc0c5aca4ed` contains these exact blobs:

- `source/product/fabushi/fabushi-account-service.ts@067b644ee568743892ba8cd0126d8b040eb21d4b`
- `source/mahayana/mahayana-rs/mahayana-product/src/native_desktop_session.rs@d3941bf971ce0d357263ee298e1cdb1f264c50c0`
- `source/host/src/extensions/auth/auth_service.rs@c9fc53ef66e7f1a1c06381a40515edb410f3418b`
- `source/mahayana/mahayana-rs/mahayana-telegram/src/lib.rs@fce2ef0a509d6033697fac29a121b429ee4890bd`
- `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs@86cfa4c38bd535939e9299b79b8d72ce2db6fc70`
- authored TypeScript contract: `source/product/fabushi/fabushi-account-service.contract.test.ts@969eb3cede3060366e1b73c4d4106bcf1f49be89`

The existing Fabushi account owner derives a stable account/device-scoped secret name and retains the actual 32-byte Telegram storage key in the existing OS-protected secret store. The key is projected only through the existing private Host credential. The Rust native credential reader validates/decodes it to `[u8; 32]`; the shipping Host auth parser accepts the private field solely so its deny-unknown schema remains compatible and does not expose the key in the public credential object.

AppHost hashes the canonical account identity to an app-data database path, opens/restores one persistent embedded Telegram runtime client through `TelegramClientLease`, retains that lease for process lifetime, and injects only `TelegramClientHandoff.client_id` into `HostCreateConfig.telegram_client_id`. Existing `MahayanaHost` then installs the existing `TelegramConversationProvider::from_client_id`, which is non-owning. AppHost field order ensures the provider/FeatureHost drops before the client lease closes.

This is one production composition, not a new Telegram account/runtime stack. No `TelegramAccount`, second login truth, sidecar, subprocess, web wrapper, second ConversationProvider registry, or second message store was created. Generic FeatureHost/CLI paths without the canonical platform-account storage owner remain fail-closed and do not invent an ephemeral Telegram identity.

## Authored tests and remaining state

Authored but unexecuted contracts cover:
- stable/distinct account-device-issuer secure secret scoping;
- strict private `telegramStorageKey` decoding;
- stable/distinct account-scoped Telegram database paths.

No executable gate has run. Telegram network authentication/self-user settlement, broader fixed-U semantic/implementation closure, Boost/libiconv member enumeration, original-host GitLab blockers, release legal acceptance, Stage-B runtime/package evidence and independent release acceptance remain open. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
