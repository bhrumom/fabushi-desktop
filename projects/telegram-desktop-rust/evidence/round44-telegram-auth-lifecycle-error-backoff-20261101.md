# Round 44 — Telegram auth lifecycle typed error/backoff contract

Date: 2026-11-01
Checkpoint base: `ad9b3c5914dd3e34797011df07b1b3716218afeb`
Production implementation: `a9e9683a3bf8cd05fd79b46213c19f56bc7c606d`, `e7c0da2a654aad9a65e6c7fc545f9cf7d9dd48a8`
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Exact fixed-U behavioral sources

- `Telegram/SourceFiles/intro/intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`: pending auth request cancellation, `auth.cancelCode`, `auth.resendCode`, email-code verification, flood/error routing, `SESSION_PASSWORD_NEEDED`, and resend fallback semantics.
- `Telegram/SourceFiles/intro/intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`: password/recovery errors, `SRP_ID_INVALID` bounded refresh behavior, `auth.requestPasswordRecovery`, `auth.checkRecoveryPassword`, and request cancellation.
- `Telegram/SourceFiles/intro/intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`: `account.sendVerifyEmailCode`, EMAIL_NOT_ALLOWED/EMAIL_INVALID/EMAIL_HASH_EXPIRED handling, and cancellation.
- `Telegram/SourceFiles/intro/intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246`: send-code flood/invalid/banned handling and pending-request cancellation.
- `Telegram/SourceFiles/intro/intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96`: registration flood/name/challenge-invalid error routing.

No C++ UI implementation was copied. These reads support only the mapped lifecycle/error responsibility.

## Existing-owner implementation

- `source/mahayana/mahayana-rs/mahayana-telegram/src/lib.rs@07c4bdff3a146dcd96165358e43bf6ca6d4dd137`: bounded Telegram auth issue classification; capped FLOOD_WAIT/FLOOD_PREMIUM_WAIT backoff; same-lease AUTH_BACKOFF_ACTIVE projection; live AuthorizationSnapshot/self-user settlement on success and recognized issue; authored inline test.
- `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs@d22df18ceec568f1a0c75663f664a3a4b0d9a9af`: Telegram status now delegates to the retained lease's canonical authorization projection.
- `source/shared/rpc/coordinator.ts@77aa3c82d6fcf1b7f11996cdd7544551af23bf55`: shared discriminated authorization-state/error/backoff contract and bounded reply validation.
- `frontend/src/production/coordinator-client.ts@9d5cf6bc55c4e769835f21c433cf30a84fb4e314`: renderer client imports the shared projection and validates all `feature.auth.telegram*` replies.
- `source/shared/rpc/coordinator.contract.test.ts@9b92080adb2a3457dfd20b55753b3aae661c723d`: authored-only contract cases for typed challenge/error/backoff and malformed/unbounded rejection.

## Runtime primitive boundary

Manual source inspection of `bhrumom/fabushi@edde714def34e6eb5ae133dba20790c80a355c85`, `native/telegram-runtime/src/runtime.rs@9c82e5b78ed578ffd7ec283d42771333fc4be96c`, confirms the current embedded runtime has commands for sendAuthenticationCode, submitAuthenticationCode, submitAuthenticationPassword and submitRegistration. Searches and source reads found no runtime/protocol builder command path for:

- `auth.resendCode`;
- `auth.cancelCode`;
- `auth.requestPasswordRecovery` / `auth.checkRecoveryPassword`;
- `account.sendVerifyEmailCode` / `account.verifyEmail`.

Those operations remain implementation blockers. This round does not simulate them, create a second Telegram runtime, or route them through a sidecar/provider fallback. `SRP_ID_INVALID` automatic challenge refresh also remains blocked on a safe runtime account.getPassword refresh primitive; the new typed error projection identifies `refreshPasswordChallenge` without recursive retry.

## State/error/backoff contract

Known remote/auth-context failures are converted into bounded `TelegramAuthIssueProjection` values. Flood waits retain a capped process-local deadline (maximum seven days) on the same `TelegramClientLease`; subsequent auth mutations during that deadline return `AUTH_BACKOFF_ACTIVE` without another network authorization mutation. The status/issue response still rereads the same runtime AuthorizationSnapshot and self-user identity. The deadline is not a second durable truth: restart leaves authoritative server rate limiting to Telegram while existing encrypted authorization/auth-key persistence remains unchanged.

## Authored-only verification

The inline Rust classifier/backoff test and `source/shared/rpc/coordinator.contract.test.ts` were written and manually reviewed but not executed. No build, typecheck, lint, schema validator, generator, unit/contract/integration/E2E test, package, deploy, install, workflow/status query or runtime acceptance was run in Stage A.

## Open gates

Resend/cancel/password-recovery/email-verification and explicit SRP challenge-refresh primitives are implemented in the canonical platform-core Stage-A checkpoint and wired through the existing desktop lease/feature.auth contract. Exact desktop dependency-manifest binding to that platform-core checkpoint is not provable from the sparse desktop ref; actual Telegram application credentials remain external protected product configuration; broader fixed-U semantic/implementation closure, Boost/libiconv member enumeration, original-host GitLab blockers, adopted/copied-derived legal closure, Stage-B executable evidence and independent ACCEPT remain open. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.


## Stage-A remaining Telegram auth lifecycle primitives (round 45, 2026-11-01)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact TL authority is `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a`; fixed-U intro behavior remains anchored by `intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`, `intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`, and `intro_email.cpp@dfb5ceeddeb48ab00773add72e75e0aef8ceb074`.

Canonical shared owner work is committed on `bhrumom/fabushi-platform-core` checkpoint `checkpoint/tdrp-telegram-auth-lifecycle-7457eb1c-r16`: protocol `314e8804ddfc997acdde2c19bff831b5e26b1706`, encrypted storage `bcb8298a4622616f2f8a226899c0e3dc8aa3ba8f`, runtime `ec854e93fb1523d95225203554600858798e4981`, and spec/evidence `a50f33da685506f4e170729a9d11f01f8dda8ec5`. These add auth.resendCode, auth.cancelCode, auth.requestPasswordRecovery/auth.checkRecoveryPassword, account.sendVerifyEmailCode/account.verifyEmail, explicit account.getPassword challenge refresh, and encrypted pending AuthorizationState/AuthContext persistence.

Desktop implementation `169d6cc7798507f0302d6c93fef7deef1a302d28` wires the matching seven commands through the existing `TelegramClientLease`, AppHost `feature.auth`, shared Coordinator contract and production renderer client. No second account/runtime/provider/state store was added, and no api_hash is exposed to renderer code.

The current desktop checkpoint is a sparse overlay and contains no Cargo/dependency manifest, so exact binding of its `fabushi_telegram_runtime` dependency to the platform-core checkpoint cannot be proven from this ref. That remains fail-closed integration evidence, not silently treated as shipping validation.

All protocol/storage/runtime/coordinator tests are authored only. No build, typecheck, lint, schema validator, generator, test, packaging, deployment, installation, workflow/status query or runtime acceptance was executed. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
