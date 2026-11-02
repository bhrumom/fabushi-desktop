# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Round30 — forward feedback and recovery qualification

The actual pointer bridge and ordinary mouse-path regression are committed at
`7d57aa4393c4f6c13bc6129255e2bec8173ed5ed` (run41531186027). Its jobs were still
queued with empty steps at this preparation, so no execution pass is claimed.
Historical9b9/run41528369271 executed Human63/creation12/storage3/journal12 and
Social10/9/6 successfully; UI10/11 failed because its test hovered row whitespace.
The parallel anchor-hover correction is retained. Newer creation14 real external
SQLite writer/WAL controls and Human67 have separate containing-H requirements.

The current authored forwarding batch reuses existing dialog, renderer, composer
and Session owners. It freezes selected destination labels with the first intent,
preserves them across search hiding and retry, shows named one/two-target success
only after validated settlement, retains honest failure/count feedback, and clears
notices across account changes. Names never enter service requests or durable
messages. Actual renderer account/late-reply cases and existing dialog behavior
checks are authored: settlement9/dialog17/renderer13, not yet executed.

Source/ledger/RTM stay canonical; pending phrase-factory Saved/self/premium and
navigation obligations are retained. No Bot/Agent/Host/Runner/Tools/approval,
Plugins/MCP/Computer/Automations capability is replaced. Full source/legal, real
Native service deployment/authenticated pagination, signed platform packages,
UI/a11y/performance/soak/upgrade/rollback/Bot, independent ACCEPT and release remain
blocked. Backend PR137 remains open/draft/unmerged and read-only.
Receipt: `projects/telegram-desktop-rust/evidence/round30-forward-recovery-qualification.json`.


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

## Round7 resumed NMR-22/24/26 qualification

The existing Session/gateway now refuses malformed reaction targets and emoji
before staging or HTTP, preserves current row fields at delayed settlement,
and keeps durable rejected intentions diagnosable. Existing Transcript projection
fences obsolete reaction responses, and pagination waits for the accepted page
before consuming its viewport anchor. Later reply/upload/forward/credential/CLI
changes remain intact. Eight Host and nine production-helper cases are added,
including canonical message-ID and strictly advancing page refusal/recovery;
containing-H Actions execution is required. Initial run41409695104 executed16
Human cases:14passed/2failed; fixture repair30b4/run41410242810 passed16Human/4Agent
and shipping gates.
See `projects/telegram-desktop-rust/evidence/fbcp-native-message-round7-20261102.json`.
Full migration, real service, source/legal, packages/UI/a11y/Bot and ACCEPT remain open.

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

## Historical planning snapshot (2026-10-02)

| Area | Status | Note |
| --- | --- | --- |
| Existing architecture as sole target | specified | PR #20/canonical Fabushi is the skeleton |
| Telegram as research source only | specified | no Telegram Provider/network dependency |
| Existing-owner-first absorption | specified | new owners require ADR |
| Native Fabushi network | specified | protocol/services not yet designed |
| Exact existing-owner inventory | blocked | P0 pending |
| Telegram capability graph | blocked | TDRP research incomplete |
| Capability → owner mapping | blocked | P0 pending |
| Conversation/message model evolution | blocked | P0 pending |
| Native messaging infrastructure | blocked | not implemented |
| Human messaging in existing workspace | blocked | not implemented |
| Human + Agent unified flow | blocked | not implemented |
| Full feature absorption | blocked | not implemented |
| Packaged acceptance | blocked | no artifact |
| Release | blocked | ACs incomplete |

## Do not report

- Telegram migration percentage as product progress
- Telegram Provider progress
- a new Communication Core as progress
- file-count parity
- source reading as implementation

## Progress dimensions

1. existing-owner inventory
2. Telegram research coverage
3. absorption mapping
4. model evolution designed
5. native network designed
6. native network implemented
7. existing owners production-wired
8. Human + Agent unified UX
9. full capability acceptance
10. packaged/release acceptance

## Resumed round 6 — durable Human history traversal (2026-11-02)

The interrupted old round-6 candidate is not applied over later work. On the current checkpoint, NMR-PAGING-01/NMR-ACTIVITY-01 add bounded persistent Human page membership, opaque RPC/cache continuation, committed-update retry overlays and monotonic activity recovery through existing Session/Transcript and renderer owners. Existing full-credential fences, reply/forward journals, resource limits, shipping/source and Agent gates are preserved. Tests are authored at this checkpoint; exact containing-H Actions must establish execution. Historical 849 failure and f68 prerequisite success are separately recorded in `projects/telegram-desktop-rust/evidence/task20bf-round6-resumed-stable-pagination.json`. This is stage-A implementation, not source/real-service/package or independent ACCEPT.

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

## Human message retry — HRR-01–07 (task20bf round1)

The existing Retry action now requests only conversationId + original clientNonce through the existing Coordinator/Host Session gateway. Session uniquely reloads the authorized durable intent, reuses stored reply/resources and returns canonical settlement; original forward targets re-enter the original source batch. Malformed identities/provenance reject without mutation. Pending dispatch errors remain visible after reopen. Account/selection lifetime checks fence the actual request frame and deferred UI changes; Agent retry remains in its existing branch.

Authored gates: four new real HTTP/SQLite/restart cases (Human total67, including64 admission variants),28 UI/controller cases,4 actual Coordinator port cases,8 original queue/pre-Host cases and2 shared contract cases. Execution is not-run at this preparation; the containing-H Actions result must be recorded in `projects/telegram-desktop-rust/evidence/task20bf-round1-human-message-retry.json`. Resource staging retains its existing serialized boundary; only message POST releases the conversation lane. Existing automatic forward recovery may still wait on a batch HTTP lock; concurrent NMR-50 work must be integrated without overwrite. Full source/legal/service/signed packages/UI/a11y/performance/soak/upgrade/rollback/Bot/independent ACCEPT remains open.

HRR-07 integration: failed pre-frame Human submissions resume the original existing Composer queue input with per-attachment commit progress; possibly posted frames switch to identity-only retry. No projected payload, fresh nonce or current-draft mutation is used. Actual authored gates at the current source freeze: Human67; Transcript34; retry helper28 + port4 + pre-Host queue8 =40; shared cursor/RPC19; retained actual Renderer Chromium11. These totals retain concurrent work and remain unexecuted at preparation. Full containing-H evidence is recorded in the same task receipt.

## Native response completion — bounded authoring snapshot

The current checkpoint's earlier forwarding, source-review, renderer and SQLite repairs
are preserved. The actual Native ProductClient transport now retains its dispatch binding
through bounded platform/JSON/multipart response completion and rejects obsolete results
through the existing private credential owner. The change adds no writer, retry or identity
store. Five parent tests/69 real subprocess HTTP scenarios are authored; the containing-H
GitHub Actions result is still required at this snapshot. Original run41493248135 and
qualified exact59f5fa/run41508537631 are recorded separately in
`projects/telegram-desktop-rust/evidence/round25-native-response-binding.json`.
This batch does not merge/deploy Backend PR137, qualify a formal publisher or signed
installation, close complete source/legal review, or grant independent ACCEPT.


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
are preserved. Authored gates are projection42/draft14/persistence5/reply5,
current cursor19/retry40/Forward9+17/history13/Transcript8, author/recovery21,
and two actual Electron Human process cases alongside existing Plugins2.
These counts are requirements until the containing commit's Actions terminal
logs establish actual execution.

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
