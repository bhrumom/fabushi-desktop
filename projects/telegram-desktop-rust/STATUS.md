# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-10-19  
Spec: TDRP-001 Revision 9  
Parent: FBCP-001 Revision 7  
Complete: false

| Area | Status | Note |
| --- | --- | --- |
| Product role | active | tdesktop is complete source authority, not a runtime/provider |
| Discovery upstream | observed | `0a11537c220df35fe59a96aebe413b1e52bc6e47`; fail-closed rebaseline candidate, not accepted |
| Historical baseline | stale | `b16daff6df63e8a812bc2b52a04ff243f67a7d3f`; predecessor evidence cannot be inherited |
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

At Fabushi `main@adce9f7618e80870bd89a2dee6db7f77d6e066a9`, the committed TDRP lock still targeted `b16daff6df63e8a812bc2b52a04ff243f67a7d3f`. Live `telegramdesktop/tdesktop@dev` is now `0a11537c220df35fe59a96aebe413b1e52bc6e47` (tree `2d737e3da8fd9a36166db27ff56756c99f6c2117`), 31 commits / 63 paths ahead. Root non-directory inventory is 6,703 and recursive non-directory inventory is 15,798. The four changed direct gitlinks are `Telegram/lib_base`, `Telegram/lib_ui`, `Telegram/lib_webview`, and `cmake`; only `cmake` changes recursive cardinality (+1). The 27 previously read-complete source entries retain exact blob identity. Build acquisition authority also changes: `desktop-app/patches` advances to `4811dda9fcabbf9cd8cab585d4df2379d81edf04`, and `prepare.py` adds an exact-pinned `desktop-app/qt6_windows7_patches` input. This rebaseline remains unaccepted until current-head GitHub Actions proves the reconstructed authority.

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
