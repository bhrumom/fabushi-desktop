# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-02  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Current bounded batch (round 20): blockmap verification and persistent approval fixtures

Fixed U=f985723b552b30fe1c17d377d57f44e61a15a8c4; inspected checkpoint
H=151d3ffe6382219dab98fb317e59fe5faa8cea01 and live main
5528062177e3865eef60a073a066cca144f9e3e8. Main and backend are unchanged.
Run41465224038 completed failure. Complete logs establish Human33/Agent4/
private3, UI16, AppHost12, Native6, FeatureHost7, Host6, ABI7 and CLI3 passed.
The proxy matrix actually exercised16 ordered subsets,48 deletions and48
repeated no-ops, including all6 full permutations. Gateway executable ran5:
4 passed and the first-issuance assertion in the repeated-approval case failed.
Its two tests shared the production default persistent file (TEST_FLAW).
Gateway lib9, protocol122, TCP9 and actual package-copy gates did not execute.

Electron carrier10/auth8/copy3/metadata15 passed. The digest-pinned historical
installer experiment failed because comparing compressed blockmap bytes rejected
a valid ZIP sidecar before reaching the real stale-DMG metadata mismatch.
NMR-43 now compares complete bounded decoded map bytes and receipts hash the
actual retained sidecar. Three added cases cover equivalent gzip envelopes,
content tampering and corrupt/oversized input; the required total is18.
NMR-45 gives each approval test an explicit UUID-owned persistent store and
checks duplicate stability, restart restoration and independent-store isolation.
Production registry defaults and approval deduplication are unchanged.

Historical artifact13093546572 was downloaded and its full digest verified;
its original DMG is174156394 bytes versus metadata174154251 (delta2143).
Package bytes remain immutable inputs to the repair experiment. Containing-H
Actions results must qualify both fixes; this preparation snapshot claims no
new-H pass. The receipt records exact prior logs, artifacts, frozen source review
and required gates at projects/telegram-desktop-rust/evidence/round20-blockmap-gateway-recovery.json.
No local executable validation ran. Fixed-U inventory remains16381 identities/
183 responsibilities, unknown/unread unestablished, source_reviews absent and
acceptance false. The existing FMEP-SOURCE-01 schema/validator was inspected;
the narrow ledger refresh also requires its Actions contract/candidate gates.

Backend PR137 remains open/draft/unmerged at06047d3ea69313ec4cfec58bbee36a7afc9cf007;
both exact-head runs passed handler29/SQLite10. Downloaded production provenance
artifact12542026010 binds historical source33a26692ce2f74e3fdca3940dfc4cbfb3ede151b
to Worker version3ad1a889-4c28-4344-8472-8c8adc225a3c. Its health/FDN401 checks do
not prove a current active version or authenticated social-message pagination.
No backend merge, deploy or mutation occurred. Current-H signed installer,
Native update service, external ABI callers, full source/legal and functional
closure, supported platforms, UI/a11y, Bot regression, upgrade/rollback, soak
and independent ACCEPT remain open.

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



