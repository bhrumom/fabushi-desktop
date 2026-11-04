# Round 43 — canonical Telegram login entry and private application credential

Date: 2026-11-01
Checkpoint base: `bf72e29187f6932a56462295b8c63cee0005f2ed`
Production implementation: `c8109e3412d8c2cb4e7362a0fe4ec4976c07bd22`
Fixed upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Exact fixed-U behavioral sources

- `Telegram/SourceFiles/intro/intro_phone.cpp@ee14ffa7c6f31e171cdcbdfaf627ea9868af1246`: `PhoneWidget` sends `MTPauth_SendCode` and settles the sent-code result.
- `Telegram/SourceFiles/intro/intro_code.cpp@2bed39cf0d696ee642e7d18f3f17bec05e8e4489`: `CodeWidget` sends `MTPauth_SignIn`, handles authorization/password/registration branching and settles authorization.
- `Telegram/SourceFiles/intro/intro_password_check.cpp@d08bf88408b29cb5ec7ed31c4e925074bfb8825e`: `PasswordCheckWidget` computes the password check and sends `MTPauth_CheckPassword`.
- `Telegram/SourceFiles/intro/intro_signup.cpp@1ba7a38379a6ae2febe615783333fd0822e0bb96`: `SignupWidget` sends `MTPauth_SignUp` and settles authorization.

No C++ UI implementation was copied. These exact reads support only the mapped login responsibility and do not close the whole source files.

## Existing-owner implementation

- `source/product/fabushi/fabushi-account-service.ts@8e26fd5eab70d27ee553791ca30a8d6147358bf5`: application-scoped secret name, bounded `apiId/apiHash` validation, protected-secret read, existing mode-0600 Host credential handoff.
- `source/mahayana/mahayana-rs/mahayana-product/src/native_desktop_session.rs@8202caa3317d57def98d84b7a7f793edcd612d6c`: private pair validation/getter; pair participates in credential binding and is not serialized/projected.
- `source/mahayana/mahayana-rs/mahayana-telegram/src/lib.rs@d90bbca9b452aca09957b8b59142180cdfe6a5dc`: redacted `TelegramApplicationCredentials`; send-code/sign-in/password/registration on one lease; live/restorable transport reuse; post-mutation `AuthorizationSnapshot` handoff.
- `source/mahayana/mahayana-rs/mahayana-app-host/src/lib.rs@165abd51b30f4237cf441191854bcd1f0d652a40`: canonical `feature.auth.telegram*` AppHost routing to the retained lease.
- `source/shared/rpc/coordinator.ts@9f2cd3c91663ef7c26cf17c211cfb2de0e977362` and `frontend/src/production/coordinator-client.ts@c09dfd0e8130de3f63875184f661823d324c5b00`: typed renderer/Coordinator methods. Their parameters contain phone/code/password/name only; no `apiId` or `apiHash`.

## Security and lifecycle result

The application pair is main-process configuration, not user/renderer state. The renderer cannot provide the application identity, so an arbitrary renderer cannot swap Telegram API credentials. The same account-bound client lease retains the pair only for its process lifetime and continues using the encrypted account storage/AuthorizationSnapshot from round 42. If no application pair is configured, send-code fails closed. If a persistent transport can be resumed, send-code does not replace it with a new auth key.

No second Telegram account owner, runtime, provider registry, sidecar, subprocess, message store or Web wrapper was introduced.

## Authored-only verification contracts

- `source/product/fabushi/fabushi-account-service.contract.test.ts@676136367fb9758e982488e9520febe04f507b1e`: bounded application credential and exact private secret key.
- inline Native credential tests in `native_desktop_session.rs`: pair completeness/format and private getter.
- inline AppHost contract in `mahayana-app-host/src/lib.rs`: renderer parameter contract excludes `apiId/apiHash`.

They were authored but not run. Stage A remained zero-execution: no GitHub Actions/status query, build, typecheck, lint, validator, generator, test, package, deploy, install or runtime acceptance.

## Open gates

The concrete Telegram `api_id/api_hash` value must be provisioned through the protected product secret source outside the repository. Broader fixed-U semantic/implementation closure, Boost/libiconv raw member enumeration, original-host GitLab blockers, release/legal acceptance, Stage-B executable evidence and independent ACCEPT remain open. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
