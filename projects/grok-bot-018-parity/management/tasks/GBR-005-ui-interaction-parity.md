# GBR-005 — Reference UI / interaction / backend parity

Status: automated source + runtime complete / packaged human visual-interaction acceptance delegated to GBR-007

## Objective
Use the pinned Grok Bot 0.18 reconstruction as the sole production UI/interaction contract, with only the installed-local-Mac Computer adaptation.

## Exact source state
- Reference commit: `107877b4e2134fd167d239411386f09e42eadd6d`.
- Renderer: 308/308 `frontend/src` blobs byte-identical to `desktop/src`; 0 missing / 0 different.
- Reference backend source: 1,724/1,724 blobs byte-identical under `reference/grok-bot-0.18/source`; 0 missing / 0 different.
- Production renderer is the exact pinned `ProductionRenderer`.
- Legacy Contacts/Telegram/payment/MiniApp/Mahayana renderer surfaces are not mounted.

## Executable claim closure
- Official coordinator calls: 58/58 executable local paths.
- Coordinator subscription families: 10/10 event paths.
- Desktop bridge claims: 95/95 classified with 0 unknown.
- Pinned TrayManager and Sand sharing adapters are bundled.
- Cross-user sharing fails closed when no backend is configured.
- Three account/subscription dashboard claims remain external-account-service classifications rather than fabricated local behavior.

## Verification
Run `35419150457` on released package SHA `1f27c239aa8931b3266b9b83c84ff234332785d9` passed exact pinned directory diff, external Host-carrier classification, pinned reference runtime bundles, CommonJS syntax, 105/105 runtime tests, reference source typecheck and TypeScript/Vite production build.

## External evidence boundary
The reference Host activation script requires immutable carrier `src/app/dist/host/host-main.cjs`, absent from the pinned Git repository. It remains EXTERNAL_DEPENDENCY and is not replaced with invented evidence.

## Acceptance
Automated recoverable source/runtime scope: PASS. Actual packaged visual fidelity, macOS interaction behavior and signed-in end-to-end provider behavior remain GBR-007 human acceptance.
