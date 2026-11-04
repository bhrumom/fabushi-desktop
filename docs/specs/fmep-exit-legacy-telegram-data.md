# FMEP-EXIT-01 E3 — Retained legacy Telegram data: retention, migration and retirement

Status: active (2026-11-04)
Applies to: FBCP-001 / TDRP-001, FMEP-001 r3 §14 FMEP-EXIT-01

## Scope

Data that older Fabushi desktop builds created through the embedded Telegram client:

| Item | Location (per account data root) | Created by |
| --- | --- | --- |
| Encrypted Telegram SQLite + WAL/SHM | `app-host/feature-host/telegram/*.sqlite3{,-wal,-shm}` | `providers/telegram-storage::EncryptedSqliteStore` |
| Telegram storage key | OS secure store entry `telegram-key` (and descriptor fields `telegramClientId`/`telegramSelfUserId`/optional Telegram credential fields in the Native account descriptor) | Electron account adapter (pre-round-14) |

Since round 14 the shipping desktop never opens, upgrades or writes these items; `MahayanaHost::create` rejects any legacy handoff, and AppHost answers `feature.auth.telegram*` with `legacy_telegram_unavailable`.

## Decision

1. **No product data is migrated.** The retained Telegram store holds Telegram-network sessions and caches, not Fabushi conversations. Fabushi Human messages live in the Session owner and the Fabushi service. There is nothing that must be imported into Fabushi owners.
2. **Retention window.** Keep the files and the secure-store entry untouched for **two stable releases** after the release containing FMEP-EXIT-01 E1/E2, so that a user who rolls back to a pre-round-14 build still finds their data.
3. **Retirement step (release N+2):** a one-shot, idempotent, account-scoped cleanup in the existing AppHost startup data-root owner:
   - only runs after the Native credential for the account is valid;
   - moves the `app-host/feature-host/telegram/` directory to `app-host/feature-host/.retired-telegram-<timestamp>/` (rename, not delete) and records a marker;
   - deletes the renamed directory and the `telegram-key` secure-store entry one further release later, or immediately if the user chooses "Remove legacy data" in Settings → Privacy;
   - never touches files outside that directory; failures leave data in place and are retried on the next start.
4. **Descriptor fields.** Optional Telegram fields in the Native account descriptor stay readable (ignored) for compatibility. They are dropped on the next normal descriptor rewrite after the retirement step.
5. **Code consequences.**
   - `providers/telegram-storage` (and its dependency `telegram-core`) remain until the retirement step ships, because the AppHost retention contract test reads the old encrypted store to prove it is untouched.
   - After the retirement release, remove that test dependency and the remaining provider crates (E4 completion).

## Verification (Actions only)

- Retention: existing `native_desktop_account_handoff` tests keep proving the files and key are untouched.
- Retirement: new tests prove that cleanup is idempotent, renames only the exact directory, survives a crash between rename and marker, never runs without a valid Native credential, and that rollback within the window still finds data.
