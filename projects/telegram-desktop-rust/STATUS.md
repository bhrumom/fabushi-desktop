# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-03  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Current bounded batch (round 22): Windows publisher trust and exact-base recovery

Fixed U=`f985723b552b30fe1c17d377d57f44e61a15a8c4`; base H=`8f089dd8640dacb4ec39105aa6d43e4c547ac5a2`; staged round-22 code commit `5d271ff0c03d8f01368ca44026d14cd7519ad6bd`, parent exactly base. Production Windows updater no longer trusts historical Anysphere signers implicitly. Only explicit approved main-process policy with exact signer subject and certificate SHA-256 can authorize an embedded Authenticode candidate; installer bytes are rechecked, permission is single-use, and the existing marker, settings, staged recovery and user data are preserved. 28 controlled behavior tests, 11 native Windows/ephemeral CI fixture cases and a well-formed unequal AppHost digest negative are **authored, not yet executed at new H**. Formal Fabushi certificate identity has not been provided or guessed.

At original exact H run41476964752 completed/success after one cancelled unassigned reconciliation and targeted recovery: shipping job125384977191 passed gateway library11, protocol122, TCP9 and actual AppHost resource copy4 (three fixtures plus real Darwin binary); missing-binary and invalid-format hash negatives rejected. New valid-format unequal-hash negative is not covered by this old H. Reconciliation attempt2 job125394283462 passed Human33/Agent4/private3; UI16 and Electron update-source16/carrier10/auth8/copy3/metadata18 passed. Failed unassigned job125384977860 had no steps/runner, not an implementation/oracle failure. Exact run and artifact digests: `projects/telegram-desktop-rust/evidence/round22-windows-publisher-authority.json`.

Backend PR #137 is open/draft at `06047d3ea69313ec4cfec58bbee36a7afc9cf007`, unmerged with zero reviews; no merge/deploy performed and production Worker provenance/afterId=0 authenticated paging remain open. Generic social provider/Host durable recovery authority remains to be audited before removal. Fixed-source 16381 identities / 184 canonical responsibilities have not been falsely promoted: unknown/unread undetermined and full source/legal/packaged acceptance, Bot regressions, performance/soak, signed platform upgrades/rollback and independent ACCEPT are **NOT ACCEPTED**.

## Current bounded batch (round 21): explicit approved Electron update source

Fixed U=f985723b552b30fe1c17d377d57f44e61a15a8c4; reviewed checkpoint
H=8ce0a9a43b7fe94df2ec2085399fe65a57343577 and live main
5528062177e3865eef60a073a066cca144f9e3e8. Main and backend are unchanged.
Exact-base run41471743677 has completed reconciliation Human33/Agent4/private3,
UI16 and Electron carrier10/auth8/synthetic-copy3/metadata18 with actual renderer
and Electron builds passed. The digest-pinned historical ZIP/DMG experiment now
rejects stale metadata, reconciles and verifies it without changing package bytes.
Historical signed packaged acceptance failed and remains unqualified. At this
final baseline observation run41471743677 is completed/failure and shipping125366805190
was cancelled before runner assignment with no steps/log blob; cause unverified. Library11,
protocol122, TCP9, actual AppHost-copy suite4 and two negatives are not passes.
The copy suite is three controlled fixture cases plus one actual compiled binary.

EUS-01–06 removes the inherited default Cursor feed from the actual existing
production updater. Explicit FABUSHI_UPDATE_FEED_BASE_URL is authoritative;
the SAND compatibility value is used only when the primary is absent. The existing
Fabushi origin policy admits the source; missing/invalid values disable all update
entry paths, requests refuse metadata redirects, and status errors omit request
URLs. Production binding, live gate helper, service state, typed IPC projection,
Settings messages and telemetry use the same policy. Startup still settles the
existing persisted apply marker while source-disabled; no marker schema, settings
schema or user-data migration/deletion is introduced. Sixteen focused contracts
are authored, not locally run. Frozen independent source review found no blocking
finding; containing-H Actions execution is pending at this preparation snapshot.

Evidence: projects/telegram-desktop-rust/evidence/round21-update-source-authority.json.
It preserves full-log hashes/URLs, exact source review ranges, backend read-only
facts, implementation/test manifest, independent source review and open gates.
The canonical ledger/RTM distinguish source reading, authored implementation,
execution and release acceptance. Fixed-U inventory remains16381 identities /
184 responsibilities (one new narrow updater discovery mapping); unknown/unread unestablished, source_reviews absent and
acceptance false. Current FMEP-SOURCE-01 schema/validator is present and unchanged.

Backend PR137 remains open/draft/unmerged at06047d3ea69313ec4cfec58bbee36a7afc9cf007;
exact-head handler29/SQLite10 passed. The historical deployed Worker source remains
33a26692ce2f74e3fdca3940dfc4cbfb3ede151b/version3ad1a889-4c28-4344-8472-8c8adc225a3c;
that is not proof of the current active version. Main still rejects afterId=0.
Real authenticated pagination and independent production recovery remain open;
no backend merge, deployment or mutation occurred.

Formal Fabushi update feed service/publisher authority, asset download provenance,
historical Windows signer policy, real signed supported-platform upgrades/rollback,
complete source/legal and all applicable product paths, full Bot regression,
UI/visual/accessibility, performance/soak and independent ACCEPT remain open.

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




