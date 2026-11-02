# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02  
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Current bounded batch (round 26): forward account/lifetime isolation

Fixed U=`f985723b552b30fe1c17d377d57f44e61a15a8c4`; inspected checkpoint
H=`a1196f00230f3307ce0bd9c704ef5c4a7c9a4044`, live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Exact-H run41496846317 is
completed/success, attempt1. Complete logs confirm Social9, HTTP-owner6,
Human33/Agent4/private3 and the authenticated Gateway forwarding HTTP contract1.
Shipping AppHost copy4 passed (three fixtures and one actual Darwin AppHost),
with missing binary input, invalid hash format and valid-but-unequal hash all
rejected. Windows native Authenticode11 passed with zero failures/skips and
remaining owned fixture certificates0; Electron controlled publisher28 and
source16 passed. This is bounded execution, not signed product acceptance.

NMR-46 fixes the existing ForwardMessageDialog and ProductionRenderer path:
account generation plus dialog lifetime fence old search/send callbacks across
logout, A→B→A, close/reopen, source replacement and unmount. Synchronous submit
admission prevents duplicate dispatch; current partial retry retains request
identity and only failed destinations. The same renderer account generation
fences the subsequent roster read, including synchronous layout unmount. The
actual Session/Transcript → FabushiNativeMessagingClient → native HTTP →
SandAgentDb/events writer is unchanged. No new account/message owner, route,
data migration, provider deletion or Backend mutation is introduced.

The new Chromium component/lifecycle12 and roster-boundary5 contracts are
authored; execution at their containing H is pending at this preparation
snapshot. Transcript16 and actual renderer compilation remain required in the
existing UI Actions job. Four theme/viewport screenshots and keyboard/focus
checks are focused evidence; full packaged visual/a11y acceptance remains open.
Evidence and exact job/log/artifact hashes: `projects/telegram-desktop-rust/evidence/round26-forward-account-lifetime.json`.
No local executable validation was run.

Source accounting remains an unaccepted fixed-U candidate: 6895 root /16381
recursive identities, 41 repositories/40 edges and184 responsibilities;
source_reviews has no records, unknown/unread are undetermined. Git-tree
acquisition of cppgir is verified; semantic/external/generated/legal closure
is not. Current FMEP-SOURCE-01 schema and validator are present. Whole-batch
forward intent durability before first destination dispatch and remote forward
provenance remain concrete product gaps. Full Bot regression, real services,
supported-platform signed packages/install/upgrade/rollback, UI/a11y,
performance/soak and independent ACCEPT remain blocked.

Backend PR #137 remains open/draft/unmerged at
`06047d3ea69313ec4cfec58bbee36a7afc9cf007`. Read-only exact-head handler29 and
SQLite10 passes do not prove deployment. Current main still rejects afterId=0;
live Worker identity and authenticated paging/recovery remain unverified.
No merge or deployment is authorized or performed.

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



