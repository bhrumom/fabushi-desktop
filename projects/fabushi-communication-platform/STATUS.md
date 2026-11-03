# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Human Retry partial upload recovery — taskbfafe567 round1 continuation

Inspected base `cae1537f44e5e6401cc7bbd770b73565a8028b63`; live main
`5528062177e3865eef60a073a066cca144f9e3e8`; fixed Telegram U remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`. Existing Retry/Session/Transcript
owners now persist qualified upload prefixes before later resource I/O and retry
only the remaining suffix. Partial history echoes cannot settle missing
attachments. Real HTTP/SQLite restart, corruption and source-preservation cases
are authored; containing-H Actions and independent source review are pending.
The cold-store failure at base run41591226893/job125768179597 is a test-oracle
side effect, repaired without weakening byte assertions or changing product data.
Actual results and review/publication identities are recorded in
`projects/telegram-desktop-rust/evidence/taskbfafe567-round1-human-retry.json`.
Source6 reviews/2 complete, unknown/unread null and acceptance false are unchanged.
Full service, supported signed packages, complete quality/Bot/upgrade/rollback
and independent ACCEPT remain open; this is not full migration completion.

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

Current task20bf execution is bound to `7f8abefc6020d5fc16d7d34cfd0dc0f3b6ec2b18`,
which preserves frozen shared `217d1499dbd541274ba11f8f1fdb092a85e8bc7b`.
Renderer132/strict inventory, Electron100, Windows11 and source171/Python37/current
schema/candidate checks passed. Native UI passed its first114 cases; actual
Renderer11 then had1 pass/10 initial-history activation failures, and Transcript8
was not executed. Human65 did not run because a test used an unsupported error
Display. Social Feature10 passed but its named-output oracle failed; Runtime9/HTTP6
were not executed. These failures retain their artifacts and are not combined success.

The current repair retains every gate: structured Human error matching and captured
expected Social poison output repair the two test defects. Existing Session/native
owners now skip active forward batches during automatic recovery, scan once per
roster sync and bind the friends response to the entire credential snapshot.
Four real HTTP/SQLite/barrier/reopen cases bring the authored Human selection to69.
The same Transcript now reevaluates its near-top rule after committed rows, with
one new no-scroll activation case and the original Renderer11/Transcript8 retained.
These changes require the new containing-H Actions; full Human Retry, same-H
shipping, source/legal, service/platform/quality/Bot/upgrade/rollback and independent
ACCEPT remain open. No new state owner or data deletion is introduced.

The following paragraphs retain the original H73/H531 authoring history.

Product H=`73cc6e1cc9fa871e5f80931979606ac9ee810551` was pushed and its commit,
tree and changed file contents were reread through GitHub. Source run41514576605
passed Node157/schema37/current-ledger shape/fixed candidate. Native run41514576570
passed Chromium16/Transcript16/settlement5, Electron83 and Windows11. Six actual
light/dark/wide/narrow/retry screenshots were viewed without visible clipping.
H73 reconciliation subsequently passed Human44/sharebox1/journal12/Agent4/
credential3/Gateway1, 65 selected checks in total. Shipping remained queued at
that readback and is not credited as a pass. Full acceptance remains open.

Continued work found and repaired NMR-48d: the existing journal load/store
validator now rejects three valid-JSON privacy contradictions before recovery,
preserves original intent, and admits text-only bodies with both hide options.
The new cold/cached/repaired-intent integration and journal tests require
Human49/journal13 after preserving the concurrent NMR-49 cases. These new
tests are not-run at authoring; H73 does not execute the new extension.

Inspected migration H=`94dcf7a2213808a8ded6baddaf069562d365c6d7` and main
`5528062177e3865eef60a073a066cca144f9e3e8`; fixed U remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`. Production predecessor
`59f5fa24827812f78336c464995d161f403dedfb` passed run41508537631 attempt1;
this does not execute the new extension. Concurrent Host journal,
account lifetime, Native/legacy isolation, source acquisition and checker work
is preserved, including the later accepted acquisition-closure and recursive
responsibility-membership guards, subsequent immutable attachment snapshots,
stale-completion fences, complete journal scans and runtime source acquisition.
The concurrent reply batch retains Human44 and adds three cases. With the
privacy recovery extension and native upload case, the containing H requires Human49 and journal13.
The interrupted old local snapshot was not published over this current work.

The existing shipping ForwardMessageDialog now freezes original nonce/targets/
normalized privacy, retries only unconfirmed targets, rejects ambiguous
settlement success, bounds selection to32, restores captions when showing the
sender, and allows presentation-only close during a durable send. Existing12
Chromium cases are retained and expanded to16; execution is bound to H73 above.
No local validation or independent ACCEPT.

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
