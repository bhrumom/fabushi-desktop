# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-03
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Current bounded batch (round 27): durable forward intent and restart recovery

Fixed U=`f985723b552b30fe1c17d377d57f44e61a15a8c4`; base checkpoint
H=`2a00708897e17c76194e0811a083ef6cd315d674`, live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Baseline run41500770749
reconciliation125467351049 passed Human33/Agent4/private3/Gateway1.
Electron125467350610 passed carrier10/account8/resources3/source16/
publisher28/metadata18 and actual builds. Windows125467351150 passed
native Authenticode11. UI125467351076 compiled the renderer and passed
Transcript16/settlement5, but all Chromium12 timed out before fixture mount
and behavior assertions. That TEST_FLAW is retained; run41500770749 is now
completed/failure. Shipping125467351072 was cancelled without runner steps
or a log blob; cancellation cause is unverified and shipping did not execute.

NMR-47 stages the complete account/issuer-bound intent in the existing source
SandAgentDb before the first destination append. The immutable submitted
snapshot, ordered destination set and deterministic nonces survive process
termination; original subsets retry without expanding or replacing the plan.
Progress writes fail closed. Ordinary source/startup/destination sync uses the
same Session/Transcript append, native HTTP, resources and event path. New
batch-owned pending rows cannot bypass their originating intent; legacy rows
remain readable. Completed receipts use current permitted canonical state and
cannot recreate deleted messages. No new message writer/store, backend change
or deletion of user data is introduced.

Five new real Host/SQLite/HTTP integration cases cover a killed owner process,
previously unsubmitted destination recovery, ambiguous accepted-first replay,
immutable subset retries, scope/permission/corruption denials, concurrency and
actual SQLite staging/progress failures. Human expected count is38; five
forward-owner unit cases include malformed-state and permission matrices.
The UI fixture now uses a Vite-root module entry and preserves browser/network/
Vite diagnostics; the actual query-change and keyboard/focus cases plus four
theme/viewport screenshots remain scoped evidence. New containing-H execution
is pending at commit preparation; no local executable validation was run.
Exact commands, baseline log hashes and follow-up artifact bindings are in
`projects/telegram-desktop-rust/evidence/round27-forward-batch-recovery.json`.

Source identity/depth accounting remains unaccepted:6895 root/16381 recursive
identities,41 repositories/40 edges,184 responsibility rows; source_reviews
absent and unknown/unread undetermined. Current source-review schema/validator
already implement FMEP-SOURCE-01. Correcting the descriptive183→184 count does
not promote semantic reading or acceptance. Remote forward provenance,
attachment retention/recovery under cache loss, real production service/paging,
full source/legal closure, full Bot regression, supported signed packages,
upgrade/rollback, performance/soak and independent ACCEPT remain blocked.

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



