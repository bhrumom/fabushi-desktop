# Round 112 — WebProxy canonical address/settings boundary — source evidence

Date: 2026-11-01
Task: 7457eb1c-11a3-4626-8563-a3e495872536 round 81
Stage: A / zero CI

## Live refs at start

- desktop: `80c480076cfc3b34cc9ed96428cabc5f04fdb592`
- platform-core before this round: `129d232fc891074ec11cd67a4235b78521bcee7c`
- fixed U: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`

## Fixed-U source read

- `Telegram/SourceFiles/mtproto/mtproto_proxy_data.cpp@f9748fcb6358dd8ce7429536ec2b999cb2a989ef`
- `Telegram/SourceFiles/mtproto/mtproto_proxy_data.h@d8deebac2359936f72901ba0eec0cbfdfb186e90`
- `Telegram/SourceFiles/boxes/connection_box.cpp@e9fa8a1d8f727042e4cadd632182eee5259f80c2`

Exact adjacent responsibility: `NormalizeWebProxyHost` performs trim + `QUrl::toAce` + lowercase + DNS/IP rejection; `setWebAddress` strips only HTTPS and stores canonical host/base-path; share/import use `webproxy`, URL-encoded `server=host[/path]`, no WebProxy port, path-aware secret encoding/decoding, legacy `host` import fallback, and fixed port 443.

## Existing-owner inventory

- canonical WebProxy pure source: `native/telegram-protocol/src/web_proxy.rs@4c91f6e4e6b8d37d84afd40aae6af9fbbe7d80d4`
- canonical protocol manifest: `a62d99f23d370cd75590c7b3186307c1689554ed`
- canonical protocol lock: `97ad9046ce8926764258c978664cd8cd92fce503`
- canonical network: `241cd8ec792070a8de8af9ec6ca5b317e34376d1`

The protocol manifest/lock contain no reviewed IDNA/URL dependency, and the current platform/desktop trees have no unified Fabushi proxy/settings production owner for shipping WebProxy user-input/import/share/persistence/selection. The Telegram conversation adapter is not such an owner.

## Disposition

Spec-first platform commit: `a9fe9aa38c67102fda5b568140183857e776e58a` / initial spec blob `27d16a8bedf20c51860889602f67cedb305a205b`.

No production code was changed. Exact Unicode/IDNA plus settings lifecycle is recorded as `TDRP-R9-MTPROTO-WEB-PROXY-CANONICAL-ADDRESS-SETTINGS-001 = mapped/pending` because implementing it now would require either a duplicate settings owner, an approximate handwritten IDNA implementation, or an unreviewed dependency/lock delta. Unblock requires a canonical unified settings owner plus a reviewed exact IDNA primitive and Stage-A-safe dependency state.

Authored contract vectors are in the durable spec; they were not executed. No fake contract against a nonexistent production API was added.

Existing bridge responsibility remains `implemented/rewritten`; existing relay responsibility remains `mapped/pending`.

## Stage-A execution statement

No GitHub Actions/status/check query or trigger, build, typecheck, lint, schema validator, generator, dependency resolution, lock regeneration, unit/contract/integration/E2E execution, package, deployment, installation or runtime acceptance was performed.
