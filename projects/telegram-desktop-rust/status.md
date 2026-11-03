# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-02  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Current bounded batch (round 17): Native auth, Settings recovery and actual AppHost copy

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; inspected base
`23f737faba69b4f60eecb6ec1a031602f87f60dc` and live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Run `41451598911` is terminal
success, attempt 1. Complete logs confirm Human33, Agent4, private credential3,
AppHost composition11, Native6, FeatureHost7, Host6, C ABI7, normalization6,
future-salts2, protocol122, TCP9 including live, proxy4, Native credential1,
UI16 and Electron carrier10/auth8/package-copy3. The required jobs are
`125297875803` and `125297875447`; these results qualify the exact base only.

The same-H shipping artifact contains the actual Mach-O arm64 AppHost with
SHA256 `7d6dd995eb01fb4dd53c5e4195742d7294b8c0fd0c313d39fd6cdee848811818`.
Its normal/build dependency graph, defined C symbols and linked-library
inventory pass the legacy-boundary negatives. The actual CLI **and gateway**
consume the generic RuntimeHandle C ABI; direct old Telegram exports remain
force-linked in `mahayana-ffi`. No global ABI retirement is inferred.

Round 17 fills NMR-AC-28 in the existing six production subprocess scenarios:
125 refusal requests per legacy-auth matrix, all 15 typed Settings mutations,
full snapshot/disk restart oracles, invalid input, persistence failure and
recovery. NMR-41 fixes the inherited proxy-removal ordering defect that lost
an adjacent surviving preference; an identity-based 30-case owner table and
public dispatch/restart cases cover the deliberate fixed-U correctness change.
Composition now requires 12 tests. NMR-40 copies the freshly built actual
AppHost through the locked production FileSet and records exact bytes, mode,
provenance, exclusions and copied-binary symbols/libraries in Actions.

These round-17 changes are implemented/tests-authored, with containing-commit
execution pending at preparation. No local executable validation was run.
The single owner inventory, ledger and RTM distinguish source review,
implementation, production wiring, authored tests, exact-H execution and
independent acceptance. Evidence: `evidence/round17-auth-settings-actual-package.json`.

Backend PR #137 stays open/draft at `06047d3ea69313ec4cfec58bbee36a7afc9cf007`,
unmerged and without reviews. Its exact-H 29 handler/source and 10 SQLite
contracts passed, but current production Worker provenance and authenticated
afterId=0 pagination/denial/recovery remain unverified. Current backend main
source still rejects an explicit afterId=0. No backend mutation, merge or
deployment was performed. Full source/legal closure, all product paths,
signed/installed supported-platform packages, external FFI callers, UI/a11y,
Native proxy transport, complete Bot regression, upgrade/rollback, soak and
independent ACCEPT remain open. Complete remains false.

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





## Current exact-head reconciliation — 2026-11-04

Current product code head `08f408ac0de6223fb0e7637974bb7cadc8db4927` on `checkpoint/tdrp-forward-privacy-task20bf-r1`. Previous exact H `50bc3d275dff24be1bf60a13810fce93ae568930` has run291 Human73, actual ProductionRenderer13/Transcript10 and shipping compile successful. Run91 source contract/schema checks pass but candidate remains unaccepted; run9650 renderer build/asset/strict architecture passes with conditional jobs skipped. New retry admission and four UI cases await exact-H Actions. Full source closure, product, real service, signed packages, Bot, long-duration and independent ACCEPT remain open.
