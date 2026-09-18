# GBR-001 — Production parity entry

Status: implementation-complete / CI-pending

## Actual result
Production now mounts a single Grok-style agent application. Messenger V2, contacts/groups, MiniApps, Telegram/payment, Mahayana workbench, credential vault, OpenBot overlays, and their compatibility bridges are no longer imported or mounted by `desktop/src/main.tsx`.

The standalone desktop no longer depends on missing monorepo `../frontend` or `../third_party/mahayana` roots.

## Evidence
- Implementation commit: 1bce9f070ce789c082e47c1edb07462588d0dbc3
- Source review: pass
- macOS GitHub Actions package: pending
