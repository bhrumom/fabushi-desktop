# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-02  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Current bounded batch (round 19): gateway, proxy preferences and macOS metadata

Fixed U=f985723b552b30fe1c17d377d57f44e61a15a8c4; inspected checkpoint
H=5281b87d878c9223dc9cd1c100c0f56186c81173 and live main
5528062177e3865eef60a073a066cca144f9e3e8. Main and backend are unchanged.
Run41460570824 completed failure: reconciliation Human33/Agent4/private3,
UI16, Electron carrier10/auth8/copy3, shipping composition12/Native6/
FeatureHost7/Host6/ABI7 passed. Shipping then failed at gateway E0004 for
RuntimeEvent::TurnStateChanged. CLI/gateway executable/approval tests and
all subsequent shipping gates did not execute.

NMR-44 fixes the exhaustive lifecycle match in the existing v1 projection,
with3 behavioral cases in full gateway lib11. NMR-42 approval tests now
record real calls:8 valid inputs/four canonical values,9 invalid inputs.
The existing proxy test covers16 ordered subsets,48 deletions/48 repeated
no-ops including all6 full permutations. NMR-43 reconciles final post-staple
ZIP/DMG hashes, sizes and external blockmaps using the locked builder.
Metadata15 behavior tests and a digest-pinned historical installer-copy
experiment are authored. Containing-H Actions execution remains pending.

Historical package source9d16483f70df9af06db97b17bb02a24dc507e459 has a2143-byte
DMG metadata size discrepancy and failed20-minute packaged acceptance.
Current-H signed installer acceptance, production updater feed/consumer and
external C ABI callers remain open. Full evidence and log digests:
projects/telegram-desktop-rust/evidence/round19-gateway-settings-macos-metadata.json.
No local executable validation ran. Fixed-U inventory remains16381 identities/
183 responsibilities, unknown/unread unestablished, source_reviews absent and
acceptance false. Existing schema/validator enforces FMEP-SOURCE-01, including
negative accepted mode.

Backend PR137 remains open/draft/unmerged at
06047d3ea69313ec4cfec58bbee36a7afc9cf007; handler29/SQLite10 CI passed.
Latest observed formal deployment has historical source33a26692ce2f74e3fdca3940dfc4cbfb3ede151b
and Worker version3ad1a889-4c28-4344-8472-8c8adc225a3c, not proven live now.
Live identity/authenticated afterId=0 behavior remain open; no merge/deploy.
Full source/legal and functional closure, supported platforms, UI/a11y, Bot
regression, upgrade/rollback, soak and independent ACCEPT remain incomplete.

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



