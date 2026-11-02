# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-02  
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 2  
Parent: FBCP-001 Revision 8  
Complete: false

## Round 20 peer mutation boundary — NMR-PEER-DURABILITY-01

The fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
The existing peer registry now stages each change from the latest shared file
under the same OS lock and commits before replacing live state or returning a
frame/resolution. The real stdio publisher and both direct/compatibility
approval response routes propagate storage failures before output or Host calls.
Same-file approval deduplication and stale-instance merges use the locked latest
state. Both existing writers retain one advisory lock inode and the only backup;
no new store, owner, wire format or user-data migration is introduced.

Production was published at9bb1c85da1bdff22c8f49fbbe69e2edaa07c119a;
the canonical ledger append and temporary-helper removal were published at
5c285452de88dcfba78bd5085f9833712eb3a459. Both non-forced expected-head CAS
updates and complete remote file contents were verified. Exact9bb/run41528207637
passed peer23 on each of Linux and Windows, retained C15 with one explicitly
ignored live-DC case, Social10/9/6, Electron metadata18/build/historical-DMG repair
and Windows publisher11. Its shipping job125561310442 remains queued, so macOS
peer23, gateway executable9/library13 and subsequent shipping checks are not-run.
The same9bb has actual renderer2/11 and damaged-store creation11/12 failures.
Concurrent9b9 repairs subsequently passed reconciliation105 total (Human63,
creation12 and storage3 included) and renderer10/11; the last renderer timeout
was an anchor-hover TEST_FLAW. The newer SQLite probe and expanded corrupt-file
tests actually passed creation12/storage3 and all105 reconciliation cases at
52f6bf6f07580fd7a8967c8ab018d386428df723/run41529735914. Hover-fix221d then
stopped before Chromium at a distinct TS2741 production telemetry-map defect;
34/19/40 precompile checks passed, browser gates did not run. Concurrent7201
adds only the missing retryHumanMessage mapping; exact UI125570311430 remains
pending. Neither those failures nor pending gates grant overall native approval.
Exact5c/run41529547932 passed Node158/schema37, candidate consistency and the
expected accepted-mode rejection:184 rows/14 source reviews, only2 complete,
unknown/unread null and every source/product/independent acceptance false. All
current Human/Transcript, CLI Native Host, protocol/TCP and package gates remain.
No local executable validation has run. The original NMR48 attachment contract
and all concurrent source reviews and responsibility records are preserved.

The original151/run41465224038 failures were the shared gateway fixture and
compressed blockmap comparison; later exact59f5/run41508537631 fully executed the
corrected bounded gates, including immutable historical DMG repair. Those results
do not qualify this new peer implementation. Recent afb/run41522324008 failed to
compile a reaction restart test (E0382); existing106bb95 corrected its shutdown
receiver without weakening restart. This batch preserves that remote repair.

The independent reviews are source reviews, not ACCEPT. Peer-to-Host crash
handoff, replay publication/cross-writer epochs, power-loss, old/new binaries and
same-question clarify conflict semantics remain unqualified. Backend137 is
read-only/open/draft/unmerged at52fbf874d5fde1ad518e8f0e6497fe9cc1d3ba23:
both exact-head63-fixture runs passed; both production readiness jobs skipped.
Current Worker identity and authorized afterId=0 service pagination remain open.
Full source/legal/features, signed/installed platforms, Bot/UI/a11y/performance/
soak/upgrade/rollback and independent ACCEPT are still required.

Spec: `docs/specs/fbcp-gateway-peer-durability.md`.
Evidence: `projects/telegram-desktop-rust/evidence/round20-peer-durable-mutations.json`.
Execution: `projects/telegram-desktop-rust/evidence/round20-peer-durable-execution.json`.


## Round31 — canonical reply identity recovery

Production H `4cbd7ddbe58ae789db9d2c674e8c9fa73146af40` uses the existing
strict message-ID parser before direct or historical reply staging/HTTP. Existing
Session/Transcript owners and valid reply semantics remain. Run41524748572 /
reconciliation125549482711 actually passed Human62/resource3/sharebox1/journal12/
Agent4/credential3/forwardHTTP1, including the28-scenario historical reply matrix,
17 direct-target scenarios, dual SQLite CAS, mixed stale progress and corrupt
journal restart. UI29/cursor17/forward5/Chromium16, Electron, Windows native11
and retained C wrappers15-pass/1-live-DC-ignore also passed at this H.

Same-H shipping125549483258 is queued at observation. Originalff6 shipping and
containing106bb recovery are separate historical evidence. Full migration, real
services, supported signed packages, complete source/legal/Bot/quality/upgrade/
rollback and independent ACCEPT remain blocked. See
`projects/telegram-desktop-rust/evidence/round31-reply-identity-recovery.json`.

## Concurrent round16 — pinned package inputs and retained C wrapper qualification

The original requested run41447812811 passed at exact
`adf15caadac5775fdd8582227cad3bbf9c93cf0f`; actual round16 NMR38/39
delivery and all artifact bytes were reconciled at23f737f and its successful
run41451598911. Later59f5fa/run41508537631 supplies separately scoped actual
AppHost resource-copy and owner evidence; it does not qualify later changes.

Input capture run41514329200 at565a4639f1b7efbd15937aff93f4e14168322389
succeeded. The two independent Cargo locks and Go manifest/checksums match
artifact13175461007 exactly; capture Mac jobs skipped. Concurrent e13c34a5 already committed those
inputs, disabled capture and locked the normal builders. This batch preserves
that work, completes the remaining CI/release input guards and adds five
direct C wrapper/temporary-store recovery cases while retaining the original
ignored live-DC case. Pinned-H actual Mac binaries/owner tests and the Ubuntu
C-wrapper gate are **not run at preparation**. Original data/readers, concurrent
Session/UI work, the fixed U and all source/release acceptance blockers remain.
Receipt: `evidence/round16-apphost-package-boundary.json`.


## Round 18 scoped result — native CLI Host and installer artifact boundary

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
Production H=`1b0e674b3e39292ae715a3eb50435d83e2cb8577` restores the existing CLI Host's
desktop-full/local-only features after its FFI dependency was removed. Both real
CLI/gateway adapters now require native MCP inventory in their existing private
tests. The shipping workflow separately builds the two real executables, records
normal/build features, bytes, symbols and linked libraries, and requires four
isolated process controls. The existing release workflow keeps final-byte metadata
in its acceptance artifact so installer members retain their flat promotion
contract. No new owner, public schema or data migration was introduced.

Run41517540804 at exact H has completed UI18/settlement5/Chromium16,
Electron10/8/fixture-copy3/update16/Windows-policy28/metadata18/promotion6, and
native Windows11. Full logs and all three actual ZIP digests/target-sha receipts
were inspected. As of the recorded20:24 UTC observation, reconciliation125524808072
and shipping125524807567 were queued with no runner or steps. Human47/journal12,
native MCP, actual CLI/gateway binaries and process4 are not yet executed at H;
queued work is neither an implementation failure nor a pass.

The requested original99c49cebfcb76aee5731da78b12e88e09d7baff8 run41456371872
completed successfully. It actually exercised composition12, Native6,125 auth
requests (119 known-method refusals plus6 unknown-method refusals),17 typed
Settings calls covering15 variants with persistence failure/recovery and a fresh
process restart,30 proxy deletion scenarios, actual AppHost copy4 and2 expected
rejecting controls. Later qualified59/run41508537631 extended proxy coverage to
48 deletions plus48 repeated no-ops/all6 full permutations and3 copy negatives.
These older results remain separate source-qualified evidence.

Ledger commit `1a2b864aee290db21548262c983033d184cab356` updates two existing rows and
retains every concurrent NMR49/50/51 record, all184 rows,6 source_reviews and
fixed-source/legal identities; its temporary transport files were removed.
Source-contract run41518863601 at that exact ledger H passed Node157/Python37
and fixed-source-candidate checks. Both actual ZIP digests and target SHAs were
verified; accepted-mode correctly rejected the unaccepted candidate. All source,
product and independent acceptance flags remain false.

Exact-H FFI audit read1780 selected non-Rust executable-path instances/
1769 unique blobs with zero failed reads; target ABI strings only occur in two
workflow guards and one negative fixture. Actual desktop transport is Electron
utilityProcess -> JS carrier -> Coordinator -> AppHost through MessagePorts/
stdio/HTTP. This is bounded lexical coverage plus reviewed entrypoints, not
deep reading of every source or proof of zero external FFI consumers. Keep all
legacy ABI, credentials, SQLite and rollback readers.

The current signed-package workflow's nonpublishing mode still logs into
production and executes real Agent/MiniApp acceptance; no current-H signed
installer or artifact-only route was observed. No such dispatch was performed.
Backend PR137 remains read-only; current deployed Worker provenance and real
authenticated afterId=0 acceptance remain open. Full source/legal closure,
supported signed/installed platforms, external clients, full Bot/UI/a11y/timing/
security/performance/soak, upgrade/rollback and independent ACCEPT remain open.

Evidence: `projects/telegram-desktop-rust/evidence/round18-current-host-features-and-delivery.json`;
caller index: `projects/telegram-desktop-rust/evidence/round18-ffi-caller-reference-index.json`.

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

Execution correction: exact `59f5fa24827812f78336c464995d161f403dedfb`,
run41508537631 attempt1 is completed/success. All five full job logs were
read: reconciliation125494011040 passed Human38/sharebox1/journal3/Agent4/
private3/Gateway1, including the five actual process/SQLite cases. UI125494010641
passed renderer/Transcript16/settlement5/Chromium12; its four actual light/dark
wide/narrow images were opened. Shipping125494011097 passed the locked actual
AppHost build, gateway5/library11, protocol122/TCP9 and actual resource-copy4
plus all three required refusal controls. Electron125494011030 and
Windows125494011118 succeeded. These results qualify only that exact source.

After concurrent migration changes, the same existing journal test is now
strengthened through real valid batches and the actual observed-value
merge/write path. The reviewed e8b48cb journal keeps12 cases, later attachment
and completion protections, and the independently strengthened concurrency
test. The new oracle is not-run at preparation and requires containing-H
Actions; historical Human38 and journal3 do not qualify later code. No local
executable validation was performed. Complete IDs, digests and scope are in
`projects/telegram-desktop-rust/evidence/round28-forward-crash-qualification.json`.

At the inspected65253740 base, inventory recorded6895 root/16381 recursive
Git identities and184 responsibility rows, with source_reviews absent and
unknown/unread undetermined. Later concurrent source reviews are preserved;
these historical counts do not state current semantic coverage. FMEP-SOURCE-01 schema/checker contracts are present; no source or
acceptance state is promoted. Remote forward provenance/policy, attachment
power-loss/filesystem qualification, simultaneous multi-AppHost cross-source uniqueness,
real production service/pagination, full source/legal closure, Bot regression,
signed supported-platform packages, upgrade/rollback, performance/soak and
independent ACCEPT/release remain open.

Backend PR #137 was read only: open/draft/unmerged,
baseline H=`06047d3ea69313ec4cfec58bbee36a7afc9cf007`; later readback observed
H=`52fbf874d5fde1ad518e8f0e6497fe9cc1d3ba23`, still open/draft/unmerged.
This round made no Backend changes, merge or deployment.

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

## Round27 resumed: source-owned forwarding pending recovery

At exact936115b0aa578c340c74dd6aa7c71c362572bd72, renderer/Transcript18/
settlement5/Chromium16, Electron, Windows11 and source157/schema37 passed;
Human47 and shipping were still queued at the recorded audit. Exact59's five
message jobs passed earlier; those results do not qualify later attachment,
reply or upload-limit code. Original2a/run41500770749 remains a failed fixture
run with cancelled shipping. Full log identities are retained in
`projects/telegram-desktop-rust/evidence/round27-forward-pending-ownership.json`.

NMR-47g now keeps damaged/missing forwarding context under the original
source journal owner. An independent pending marker is stored before effects;
strict request/target-nonce lookup prevents generic replay after source denial.
One actual child-kill/SQLite/HTTP matrix preserves the current reply and pinned
attachment paths, with a prefixed legacy positive. New containing-H execution
is not-run at preparation; the current gate adds one Human case (53 after the
52-case pagination base). Concurrent NMR-50 work is retained.

The canonical ledger receives only the two reviewed existing-row changes and
current shipping-owner correction. Actions prepares the large blob with an
exact-base guard and no ref writes; integration uses non-forced expected-head
CAS. Six source reviews/two complete at936 are scoped historical counts; later
source records remain preserved. Complete source/legal/product/platform/Bot/
real-service and independent ACCEPT/release remain blocked. Backend PR137 is
not modified, merged or deployed.

## Round23 resumed: current Social boundary qualification

Fixed U stays `f985723b552b30fe1c17d377d57f44e61a15a8c4`. Exact original
H44f5203b8475afbeaa37bb049f6c72351ecb76d8 native41484767335 failed only
Windows fixture setup (11 hook failures, no executed native assertions);
source41484767424 succeeded. Later exact59f5fa24827812f78336c464995d161f403dedfb
native41508537631 succeeded including Windows11, controlled28, shipping
AppHost-copy positive and valid-format wrong-digest negative. Those runs retain
their own source/input scope. Preparation head106bb95 already contained the concurrent
reopened-runtime compile fix and later forwarding/reply/pagination work.

This batch contributes three missing Social public-command boundary cases,
raises only the matching FeatureHost oracle7→10, and uses named Social spec
IDs without replacing numeric forwarding requirements. Published code H=`83c9e3ffbf059556f3a8bce9d84d946c97433345` was remotely
read back with all 17 file contents. Exact f2903f53ded5b07faea73e7f07d780eda0423283
run41527613857/job125559290571 now passes Feature10/Runtime9/expanded HTTP6,
all with zero failed or ignored; Windows11 and controlled28 also pass at that H.
The f75 caught-panic log oracle failure and f290 UI/SQLite preservation defects
remain in the evidence. Exact9b9's reconciliation now passes Human63,
creation12/storage3 and every following group; its UI passes10/11, with the
remaining reaction hover-entry oracle and previously blocked Node8 still pending. Exact1c0bd07f85c4f17ee465509c766eead878879185 integrates
the reviewed ledger additions and removes the temporary helper; source41527855423
passes158/schema37/ledger-errors0. Later ledger additions retain their own source
input scope; no source or independent acceptance flag is promoted. Backend PR137 exact52fb remains draft/open/unmerged; both
63-case CI jobs passed and production readiness skipped. No merge/deploy or
production request occurred. Full source/legal/product/platform/Bot, formal
publisher/real signed packages and independent ACCEPT remain incomplete.

The existing workflow now also runs these same Feature10/Runtime9/HTTP6 behavioral
harnesses on Ubuntu while retaining the queued macOS shipping requirement. This
focused job is not a signed-package, live-service or independent acceptance gate.


## Round8 durable draft recovery integration (2026-11-02)

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`.
This batch reuses the existing ProductionRenderer/Session and account Composer
draft owner. It adds durable author/nonce-to-original-draft IDs, an awaited
pre-send write barrier, canonical re-observation after restore, scoped settlement,
field-level restore merging and retired-instance fencing. The process-owned
SandClientPersistenceStore must propagate non-ENOENT I/O errors and retain failed
removals; its existing queue supplies admitted-write/read ordering. No new
message, draft or persistence authority is introduced.

The current checkpoint's original-nonce Human retry, partial attachment commits,
opaque history, pointer corridor, named forward feedback and all Agent owners
are preserved. Exact808a8605b876f979427ebac11ca7e512aeffeefa/run41543732585
actually passed UI193: projection42/draft14/persistence5/reply5/cursor19/retry40/
Forward9+17/history13/Transcript8/author21, with compiled Renderer and no
failed/skipped/retried cases. Its Host reconciliation passed116 including
Human72 and actual6400/32-page, concurrency and restart cases; peer Linux23
and Windows23, carrier100, Windows11 and Social37 also passed.

Exact808 Mac run41543732546 failed Plugins0/2 and Human0/2 before Human
message behavior. Exact487083725db93109b0d3d2b4bdddd483d148ba30 follow-up
retains the existing Renderer and fixes background roster/restore/clear closing
Plugins. Its actual Mac run41546866898 passed Plugins2/2 and reached both
Human first-lifetime business, video/trace and real shutdown checks. Both Human
cases then failed in the fixture's post-close app.process() evidence access;
no second-process recovery passed. The fixture now captures the same
ChildProcess before close and retains all original process-exit and reopen
assertions. Its containing-H Mac run is required. Author22 was still running
in run41546866976 at23:46:30 UTC; exact808 and487 shipping remained queued.

The shared authority trace check and actual canonical NMR checkout test
passed at exact487 in run41546866900: Node161, Python37 and zero schema errors.
All55 target paths,60 symbols and158 trace entries were checked;183 other
ledger rows and source-review/candidate counters stayed unchanged. Actual
accepted entrypoint still refuses the candidate; full source acceptance is false.
Exact487 run41546866976 also passed Host116 (including Human72), Social37,
carrier100, peer Linux23/Windows23 and Windows publisher11. Retained FFI had
15 passed and1 ignored; that ignored live-DC case is not a pass.

Original H5d534898/run41413368942 message and shipping gates passed. Review
H375936fb compiled and passed pure41/cursor17/Forward5+16, but historical
browser11 was 10 pass/1 fail and dependent groups skipped; Electron H608f0d49
compiled native inputs but failed Renderer TS2698 before browser execution.
Current batch fixes retain those failed records. The two existing round8
evidence receipts record full source/file/run identities and subsequent results.

Canonical inventory remains 16,381 recursive entries/184 responsibility rows,
unknown/unread null and two unmapped responsibilities at the observed input.
There are 16 source-review records (2 complete/14 pending), not complete
source coverage. Backend compatible active deployment, full source/legal,
supported signed packages, complete Bot/UI/a11y/security/performance/soak and
independent ACCEPT remain open; this is not full migration acceptance.
