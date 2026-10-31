# Round 42 — canonical Telegram authorization and self-user identity

Date: 2026-11-01
Checkpoint base before implementation: `b53a154100c5bf77a255cb064f698f0c7cbc11c2`
Implementation commits: `1b4f803fb7bc06d19fad936689fcb6cd5c3a52a4`, `d19fc3742b9a2d5076513ec444383d7683a35ea2`
Fixed Telegram Desktop U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Stage A guard

This round used Git/GitHub source/blob reads, manual semantic review, production code authoring, authored test contracts and ledger/P0/source-closure authoring. It did **not** query or trigger GitHub Actions/CI and did not run build, typecheck, lint, schema validation, generators, unit/property/contract/integration/E2E tests, packaging, deployment, installation, benchmark, fuzz, performance or runtime acceptance.

## Source and production ownership

Fixed-U `Telegram/SourceFiles/main/main_account.cpp@2721c7ac12effe454d398610d26d638fadaa1f68` remains the account/session lifecycle anchor. Exact fixed-U `Telegram/SourceFiles/mtproto/scheme/api.tl@8319b7df634d145390c299404129fd161a5dbd2a` confirms the wire contracts used by this slice: `auth.sentCodeSuccess#2390fe44`, `auth.authorization#2ea2c0d4`, `auth.authorizationSignUpRequired#44747e9a`, `userEmpty#d3bc4b7a`, and the current full User constructor.

The existing Fabushi account/platform-login lifecycle remains the outer canonical identity/credential owner. The existing embedded Telegram runtime remains the Telegram authorization/session owner. The existing AppHost `TelegramClientLease` remains the only runtime client owner; Host/ConversationProvider stays non-owning.

## Implemented settlement

The runtime now:
- extracts `auth.Authorization.user.id` as canonical Telegram `self_user_id` after sent-code success, sign-in, password and registration success;
- persists authorization/challenge state and phone/code-hash/SRP context under the existing encrypted SQLite storage key;
- persists MTProto auth-key/DC/salt/server-time restore material under the same encrypted store;
- rejects restored `Ready` without both canonical self identity and auth-key material;
- lazily reconnects transport using the persisted auth key rather than creating a second Telegram client/runtime;
- reports `AuthorizationSnapshot { state, self_user_id }` from the one runtime client.

`TelegramClientLease::handoff` now reads that live snapshot and supplies `client_id + authorization_state + self_user_id` to `HostCreateConfig`. The existing non-owning `TelegramConversationProvider` dynamically reads the same runtime snapshot for identity-sensitive operations: non-Ready is fail-closed, and completing authorization in the same process does not require another provider/client or Host recomposition.

## Exact implementation blobs remotely reread

- `providers/telegram-protocol/src/api.rs@c383b3ebb1eade1dd1b4eb1686cec04d6856867e`
- `providers/telegram-protocol/src/lib.rs@46d1209267da19fcf7dd52190768d715c6f78453`
- `providers/telegram-protocol/src/srp.rs@e233548be79625983f506ff77cb3a1024f715c21`
- `providers/telegram-network/src/lib.rs@520bf4e0e2094e74c878ad1f32f3eb32b6edae32`
- `providers/telegram-storage/src/sqlite.rs@84387005dc1d51962980a4cb4c6ce69e0b308695`
- `providers/telegram-runtime/src/runtime.rs@95d27ee01f5cf1da15f595ab54e3660b24321128`
- `providers/telegram-runtime/src/lib.rs@7f1dd714d6a615ed2af67e48a52633fc7b7250b4`
- `providers/telegram-runtime/Cargo.toml@0fc8ee357ef5c00074e5fdcd1d7594860e289bd1`
- `mahayana-telegram/src/lib.rs@0efb6827e18ea6a722e298c72b61c89195482118`
- `mahayana-app-host/src/lib.rs@928adee9dcac0d471411519e43aacc70655ae6ee`
- `mahayana-host/src/lib.rs@b2a519e44630e08f7e96bd8436c4a377bb8e9d4e`

## Authored tests and remaining state

Authored but unexecuted contracts cover authorization user-id parsing, sent-code-success nested authorization, encrypted private metadata, Ready restore, fail-closed restore without auth-key material, and Host authorization/self-user handoff consistency.

No executable gate has run. Broader fixed-U source-semantic/implementation closure, Boost/libiconv complete member enumeration, original-host GitLab blockers, release legal acceptance, Stage-B executable evidence and independent release acceptance remain open. `baseline_ready=false`, `accepted=false`, `validation_status=not-run`.
