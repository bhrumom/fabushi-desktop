# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-02  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Current bounded batch (round 15)

Authoritative lock remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`, tree
`91f2b083ab2877c0a17d07868eabec362a5cfca6`; it is not source-accepted. The older
discovery and rebaseline observations below are historical, not a new request
to follow floating upstream. Current ledger source-review/schema contracts
support FMEP-SOURCE-01 risk/depth categories; this batch does not promote source
coverage, legal status or whole-product acceptance.

From checkpoint `c15dfbd9631f8b9fb6920cb5f80b9aea35210d07`, NMR-34–35 close
the separate FeatureHost messaging execution/blob/access paths for the selected
Production Electron composition, including typed dispatch. The Session/Transcript
owner remains unchanged. Existing snapshots, blobs, access records and unknown
files are retained; the Automation owner now excludes the reserved `_messaging`
namespace during load, owner override, desired writes and stale cleanup.
Seven direct-owner and six real AppHost account/restart cases are authored.
Predecessor run 41438907013 ended failure (Electron10+8/UI16 passed; both Rust
jobs cancelled without runner, steps or downloadable logs); no Rust count is
inferred. The exact scheduling cause is unverified. Execution must
bind the new containing commit through `native-service-contracts.yml` and the
unchanged source-review schema/validator workflow. The receipt
`evidence/round15-desktop-messaging-boundary.json` records preparation and audit,
not a successful future run. Live main was reread at
`5528062177e3865eef60a073a066cca144f9e3e8` and is untouched.

Non-Electron FeatureHost authority, retained data import and public legacy
FFI/default-link paths still require caller/data disposition. Backend #137 remains draft with
no review/production authorization, and production `afterId=0` evidence is
missing. Full migration and independent ACCEPT remain blocked.

Recovery readback: NMR-34/35 were committed as
`98e51b31b5a296065190f665696b2ffe11a3efe1`, run `41443460672` completed success
with all four message jobs, owner7/AppHost6 and the retained Human33/Agent4/
private3/composition11/protocol122/TCP9/UI16/Electron10+8 cases. Source run
`41443460908` passed checker136/schema37 and ledger-schema validation, while
source closure and independent/product acceptance remain false.
NMR-36 now rejects explicit legacy Host/FFI handoff before data access and
removes this Host's old provider registration. Original fields/data/readers
remain for explicit refusal and reviewed rollback. Host6 and full ABI7 cases
are authored and require the new containing-H Actions result. No dependency,
direct legacy-client export, platform-package or full-migration closure is claimed.

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



