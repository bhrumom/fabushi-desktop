# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02  
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Current bounded batch: desktop standalone-messaging boundary

Inspected checkpoint base `c15dfbd9631f8b9fb6920cb5f80b9aea35210d07` and
live main `5528062177e3865eef60a073a066cca144f9e3e8`. Fixed upstream remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`; no upstream refresh is authorized.
NMR-31–33 already remove automatic legacy secret/client/provider creation.
NMR-34–35 now refuse FeatureHost's separate messaging execution, blob reads and
access issuance in Production+Electron, including generic typed dispatch.
Session/Transcript remains the Human owner. Retained snapshots/blobs/access
files are not migrated or deleted. The existing Automation owner also excludes
the reserved `_messaging` namespace from loading, owner overrides, writes and
cleanup, preserving unknown/corrupt files during account changes. Test mode and
selected non-Electron surfaces retain existing behavior; their acceptance and
legacy FFI/default dependencies remain open.

Preparation/audit evidence is in
`projects/telegram-desktop-rust/evidence/round15-desktop-messaging-boundary.json`.
Predecessor run 41438907013 binds c15 and ended failure: Electron carrier10/auth8
and UI16 passed; both Rust jobs were cancelled before runner assignment, with
no steps and no downloadable log. No Rust result/count is accepted. New
owner7/AppHost6 and all retained gates require this candidate's exact-H Actions;
authorship/static review is not execution acceptance. Backend Draft PR #137 has no
approval or deployment authorization; current Worker provenance and production
authenticated `afterId=0` pagination are not accepted. Full source/legal,
platform/package/UI/accessibility, Bot regression, upgrade/rollback, soak and
independent ACCEPT remain open.

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


