# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-02  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Current bounded batch (round 16): AppHost dependency and package boundary

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; inspected base
`adf15caadac5775fdd8582227cad3bbf9c93cf0f` and live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Main and backend are unchanged.
Run `41447812811` finished success: complete logs confirm Human33, Agent4,
private credential3, AppHost composition11, Native6, FeatureHost7, Host6,
C ABI7, Host direct-dependency negative, normalization6, future-salts2,
protocol122, TCP9 including live, proxy4, Native credential1, UI16 and
Electron carrier10/auth8. These qualify that exact base only.

NMR-38 removes the impossible AppHost Telegram lease and unreachable forwards,
retaining all twelve explicit refusal routes, registration validation order,
Native auth/Agent state and Settings persistence/publication semantics. The
legacy proxy projection dependency is test-only; the independent shipping lock
prunes only the eight packages made unreachable, with no version upgrades.
NMR-39 restricts Electron `resources/bin` to its four canonical executable
names and Windows forms, with nested-file exclusion. Staging files, original
SQLite/sidecars/keys, retained FeatureHost files and rollback readers are kept.
The CLI is a real generic C ABI consumer; FFI/direct Telegram exports are not
deleted or declared globally retired.

Expanded Native6 behavior tests, three actual locked packager-copy cases and
the shipping dependency/build/symbol gates are authored. Execution at the
containing commit is pending at preparation; no local executable validation
was performed. The single ledger row, owner inventory and RTM retain separate
implementation, execution and final acceptance states. Preparation and prior-H
results are in `projects/telegram-desktop-rust/evidence/round16-apphost-package-boundary.json`.

Backend Draft PR #137 remains open/draft at
`06047d3ea69313ec4cfec58bbee36a7afc9cf007`, without review or formal deployment
authorization. Its handler29/SQLite10 CI passed, but current production Worker
SHA/version and real authenticated afterId=0 pagination/empty/denial/recovery
are unverified. Full source/legal, product scope, supported-platform packages,
UI/a11y, Native proxy support, complete Bot regression, upgrade/rollback, soak
and independent ACCEPT remain open. Complete remains false.

## Historical planning snapshot (2026-10-19)

| Area | Status | Note |
| --- | --- | --- |
| Product role | active | tdesktop is complete source authority, not a runtime/provider |
| Discovery upstream | observed | `1e18da644e37856c9b1d87434bfcd6a2e3d11c57`; fail-closed rebaseline candidate, not accepted |
| Historical baseline | stale | `0a11537c220df35fe59a96aebe413b1e52bc6e47`; predecessor evidence cannot be inherited |
| Recursive source closure | rebaseline required | current lock/inventory must be reconciled to one exact upstream baseline |
| File-level inventory | not accepted | must reach unknown=0, unread=0, omitted=0 |
| Responsibility decomposition | not accepted | every source symbol/responsibility requires traceable destination |
| Exact-head owner inventory | in progress | existing-owner-first remains mandatory |
| UI composition / IA | in progress | one shell, one Conversation/Search/Profile/Marketplace truth |
| Design-system migration | in progress | new migrated UI must use Fabushi semantic tokens/components |
| Native service completeness | in progress / blocked where absent | blocked service is not functional acceptance |
| Production implementation | partial | existing slices do not imply full migration |
| Existing Bot regression | required | Agent/Host/Coordinator/Runner/Plugins/MCP/Computer/Automations/Tasks etc. must remain green |
| Quality/RTM gates | not fully implemented | Revision 9 schema/validator fields and traceability still require executable gates |
| Packaged acceptance | fail-closed required | acceptance failure must block release |
| Independent release acceptance | not accepted | final verdict must be ACCEPT |

## Current live reconciliation

On 2026-10-07, `main` was observed at `e0a7a4221970088beeaf145d0d9be1763e58229f`. That commit weakened the real signed packaged acceptance step with `continue-on-error: true`, which conflicted with FBCP Revision 7 / TDRP Revision 9. The regression was removed on `main` by commit `2115a91eb888aa65702ef8f90f01848042bb65c7`.

This status file records only observed repository state. It is not completion evidence. Any newer `main` supersedes these SHAs and must be re-read before implementation or acceptance.

## 2026-10-19 live authority reconciliation

During the 2026-10-19 PR #126 exact-head reconciliation, the preceding lock targeted `0a11537c220df35fe59a96aebe413b1e52bc6e47`. Live `telegramdesktop/tdesktop@dev` is now `1e18da644e37856c9b1d87434bfcd6a2e3d11c57` (tree `30c1d5cce4c1bde962882c1f2ca4b2e51bb03ae6`), 5 commits / 63 paths ahead. Root non-directory inventory is 6,709: five new `Telegram/SourceFiles` entries and one new Resource entry. No direct gitlink pin changed, so the established recursive dependency expansion rises by the same six entries, from 16,186 to 16,192. The three already closed `.agents` entries plus mapped `share_box.cpp` / `url_auth_box.cpp` retain exact blob identity. Of the prior read-complete set, only `Telegram/cmake/td_ui.cmake` changed; its full new blob was reread and the sole delta is inclusion of `boxes/keys_legend.style`, so all 30 prior reads remain valid. Current fail-closed counters are unknown=16,189, unread=16,162, omitted=0. `snap/snapcraft.yaml` changes only by adding the `login-session-observe` permission; immutable build acquisition pins remain unchanged and current-head Actions must regenerate/validate candidate accounting. This rebaseline remains unaccepted until current-head GitHub Actions proves the reconstructed authority.

## Metrics

Report separately:
- accepted upstream baseline completeness;
- source file read coverage;
- responsibility understanding coverage;
- existing-owner resolution coverage;
- production implementation coverage;
- exact-head test/evidence coverage;
- packaged temporal acceptance;
- independent release acceptance.

Do not report Telegram-provider, Telegram-network, survey, document count, or vertical-slice progress as migration completion.



