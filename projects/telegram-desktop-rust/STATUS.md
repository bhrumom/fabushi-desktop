# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-02  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Current bounded batch: pending-parent reply recovery (NMR-49 / NMR-AC-40)

Observed 2026-11-02. Fixed U=`f985723b552b30fe1c17d377d57f44e61a15a8c4`.
Integration base is `b25c2003a8e76779f2535fc3dfa90e4fe5bde149`;
concurrent immutable forward UI, source acquisition, candidate-review and
recursive-membership changes are retained.
Runtime prerequisite `59f5fa24827812f78336c464995d161f403dedfb`,
run41508537631, completed/success: Human38, Agent4, settlement-credential3,
sharebox1/journal3/forward-HTTP1, Transcript16/forward5/Chromium12 and the
selected shipping/Electron/Windows jobs. Full logs and five actual artifact
archives were read; no complete installer or independent ACCEPT is implied.

The actual path remains Composer/Reply → Session command →
ProductionSessionWorkers → SandAgentDb/native HTTP → transcript mutations →
production model projection. This batch maps dispatching to existing UI pending,
atomically rekeys a settled local parent and its unsent same-author child reply
references, and recovers older dangling aliases only through a unique canonical
local-author nonce in the same conversation. Replay reloads each durable child.
SQLite rollback, peer/foreign/ambiguous identities and immutable fields are
covered by three new real Host/SQLite/HTTP cases and two real projection cases.

At preparation, new containing-H execution is not-run: require Human47 and
Transcript18, journal12, forward projection5 and Chromium16, plus the retained
credential, Agent and shipping gates in GitHub Actions. This reply batch does
not change the inventory or source-review completion counters. Backend Draft PR #137 at
`52fbf874d5fde1ad518e8f0e6497fe9cc1d3ba23` has exact-head contract runs
41510852280/41510861859 with 29+10+24 passes each; production readiness was
skipped; no formal merge/deployment authorization was observed in the PR or
current-run records. Source/legal closure,
authorized live afterId=0 history and durable service recovery, signed packages,
full Bot regression, performance/soak and independent ACCEPT remain open.

The inherited94 attachment snapshots, strict forwarding source scan,
completion monotonicity and metadata-object checks are retained. Require its
twelve journal cases and five attachment cases, the concurrent73 immutable
forward UI and current source acquisition/candidate-review gates. No journal
implementation or source/release acceptance is added by this reply batch.

Evidence: `projects/telegram-desktop-rust/evidence/round12-pending-parent-reply-recovery.json`.

## Retained concurrent batch — task20bf round1 resumed, immutable forward intent

Inspected migration H=`94dcf7a2213808a8ded6baddaf069562d365c6d7` and main
`5528062177e3865eef60a073a066cca144f9e3e8`; fixed U remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`. Production predecessor
`59f5fa24827812f78336c464995d161f403dedfb` passed run41508537631 attempt1;
this does not execute the new extension. Concurrent Host journal,
account lifetime, Native/legacy isolation, source acquisition and checker work
is preserved, including the later accepted acquisition-closure and recursive
responsibility-membership guards, subsequent immutable attachment snapshots,
stale-completion fences, complete journal scans and runtime source acquisition.
This concurrent base authors Human44 and journal12; the combined reply batch
requires Human47 and retains journal12. The interrupted old local snapshot was
not published over it.

The existing shipping ForwardMessageDialog now freezes original nonce/targets/
normalized privacy, retries only unconfirmed targets, rejects ambiguous
settlement success, bounds selection to32, restores captions when showing the
sender, and allows presentation-only close during a durable send. Existing12
Chromium cases are retained and expanded to16; same-H Actions execution is
pending at authoring. No local validation or independent ACCEPT.

The unique source ledger now has two complete option-source mappings and four
explicitly pending panel/phrase decompositions, despite all six bodies having
been read. Candidate and full-acceptance paths distinguish these states; the
full16381 Git-only inventory,184 responsibility rows, null global unknown/unread
and false acceptance flags remain. Full source/legal, remote forward provenance,
resource retention, live services, supported signed packages, complete visual/
a11y/temporal/Bot/performance/soak, upgrade/rollback and independent ACCEPT are open.
Evidence: `projects/telegram-desktop-rust/evidence/task20bf-round1-forward-intent-ui.json`.

## Historical round 28: forwarding crash and persistence qualification

Fixed U=`f985723b552b30fe1c17d377d57f44e61a15a8c4`; inspected checkpoint
H=`65253740c033508ab08a4b500449a5b56727e940`, live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Run41504939510
reconciliation125481711661 passed Human33, then selected one sharebox test
successfully but asserted three. The two journal tests were not selected;
Agent4/private3/Gateway1 did not execute after that failed count assertion.
UI125481711814 compiled the renderer, passed Transcript16/settlement5, and
executed Chromium12 with eleven passes and one missing-fixture-setTheme failure.
Electron125481711452 and Windows125481711041 completed successfully.
Shipping125481711613 compiled actual AppHost and passed composition12/Native7/
FeatureHost7/Social9/HTTP-owner6/Host6/ABI7/CLI3/gateway-executable5, then failed
because its gateway count assertion expected3. Later protocol/TCP/package-copy
steps did not run. All three failures are TEST_FLAW; run41504939510 is completed/failure.

Correction: the round27 STATUS/RTM claimed five integration tests and five
owner unit tests that did not exist at that H. Its referenced
`round27-forward-batch-recovery.json` is also absent from that exact Git tree.
Those descriptions are withdrawn as authorship/execution evidence; history is
retained in Git. The actual base has Human33, sharebox1 and journal2.

This round adds the five actual process/HTTP/SQLite integration cases in the
existing Session gateway target. They cover parent-killed child process,
complete intent before first POST, never-dispatched-target ordinary-sync
recovery, ambiguous accepted-first nonce replay, immutable text/subset/replay
conflicts, local source/account/issuer denial and real staging/progress SQL
failures. Concurrent replay covers completed batches, not simultaneous initial
dispatch. Human38 and sharebox1+journal3 are required at the containing H.

Two production defects are repaired in the same owners: expected-value
metadata CAS prevents a stale source-journal merge from erasing another batch;
typed source-policy denials persist failed unfinished targets without starving
unrelated inbound history. Database/decoding/write failures still stop work.
The UI fixture installs the existing runtime theme owner and proves actual
light/dark colors before its four screenshots. No public schema, service API,
message owner or data migration/deletion is introduced.

Tests are authored, reviewed and **not-run at commit preparation**. Only the
containing-H Actions logs/artifacts establish execution; no local executable
validation was performed. Exact predecessor evidence, corrections, review
findings and scoped case IDs are recorded in
`projects/telegram-desktop-rust/evidence/round28-forward-crash-qualification.json`.

The single inventory/ledger still has6895 root/16381 recursive Git identities
and184 responsibility rows; source_reviews is absent and unknown/unread are
undetermined. FMEP-SOURCE-01 schema/checker contracts are present; no source or
acceptance state is promoted. Remote forward provenance/policy, attachment
retention under cache loss, simultaneous multi-AppHost cross-source uniqueness,
real production service/pagination, full source/legal closure, Bot regression,
signed supported-platform packages, upgrade/rollback, performance/soak and
independent ACCEPT/release remain open.

Backend PR #137 was read only: open/draft/unmerged,
H=`06047d3ea69313ec4cfec58bbee36a7afc9cf007`. No merge or deployment.

## Historical bounded batch (round 21): explicit approved Electron update source

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

## Resumed round 6 checkpoint (2026-11-02)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`. NMR-PAGING-01/NMR-ACTIVITY-01 extend the existing Human responsibility and actual owner path; concurrent recursive acquisition, six source reviews and all acceptance fields are preserved. The unpushed earlier candidate is superseded by the current-branch integration. New exact-H tests remain required; bounded evidence is in `evidence/task20bf-round6-resumed-stable-pagination.json`. No source coverage or full-migration completion is promoted.

