# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Round5 caption and covered-history refresh — exact-H execution pending

The existing picker now hides its caption control for plain text and media
without caption text, sends false for that flag on new intents, and keeps sender
hiding independently available. Captioned-media coupling and frozen retries
remain intact through the real transcript projection. Original16 Chromium cases
are preserved with3 new contrasts, requiring19. No new owner/wire field or data
migration is introduced. Retained reaction/credential qualification and shared
checkpoint integration remain required; no pending or old-H result is a pass.

NMR-R5-04 also addresses old messages outside the latest200: a separate persisted
refresh cursor with a fixed covered upper bound receives one of the existing32
history requests, preserving new-message coverage and all existing message
owners. Row persistence precedes cursor CAS; failures and restart cannot skip
an unfinished page. Row and cursor admission follow SQLite write-lock acquisition
and recheck the original Native credential and current owner scope. This adds
no background owner or visual read-receipt UI.
Six added real Host cases bring the required Human gate to56; private5,
resource3, journal12 and all retained gates still require containing-H Actions.

Exact25/run41518902093 job125529507403 now has complete-log success for
Human50/resource3/sharebox1/journal12/Agent4/private4/forwardHTTP1. This scoped
prerequisite includes the R5 atomic reaction cases; it does not execute the
new refresh/caption code. Exact25 shipping was still queued at observation.

## Current round5 reaction recovery qualification — NMR-R5-01/02

Existing Session/Transcript remains the only Human message owner. This batch
atomically settles an acknowledged reaction and its exact durable intent,
checks the full Native credential fingerprint after SQLite write-lock
acquisition, and publishes only changed committed rows. It also shares the
existing persisted-journal directory scan and propagates non-NotFound errors.
Current known-message refresh/history coverage, reaction serialization,
current94 attachment snapshots/journal12, concurrent forward UI and six source
review records, and NMR-49 pending-parent reply work are preserved.

Two real HTTP/SQLite/restart cases, one private SQLite wait/credential case and
stronger ordinary-sync permission assertions are authored. Required gates are
Human50, journal12 and private credential4; new containing-H execution is
pending at preparation. Exact68/run41402071662 and exact59/run41508537631 are
actually passed prerequisites, not evidence for unrun changes. Receipt:
projects/telegram-desktop-rust/evidence/round5-recovery-resume-20261102.json.

No user data was deleted and no local executable validation was run. Full
fixed-U source/legal and product closure, deployed authenticated service,
original-byte legacy recovery, power-loss and multi-AppHost qualification,
signed platforms, full UI/a11y/security/performance/soak/upgrade/Bot and
independent ACCEPT remain open. Backend PR137 remains read-only.

Concurrent936 parent/child reply repair and dispatching projection remain intact, including all three added Human cases and Transcript18. This batch adds only its two Human reaction cases.

Qualification branch: checkpoint/tdrp-stage-a-msg-round5-reaction-atomic-76a84e8. Parent exact76a84e8 retains current service25MiB resource bounds/resource3 and real CLI/gateway process4. Shared checkpoint integration is pending; existing workflows qualify this frozen branch without disabling any gate.

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




