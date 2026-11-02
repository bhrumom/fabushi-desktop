# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02  
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Current bounded batch: Native startup isolation

Inspected checkpoint base `50b0a2048adce13f2281143e6c3f7449d56344d5` and
live main `5528062177e3865eef60a073a066cca144f9e3e8`. Fixed upstream remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`; no upstream refresh is authorized.
NMR-31–33 remove Native login's legacy secret provisioning and AppHost's
automatic Telegram lease/provider creation. Existing legacy keys, database
files and schema are retained; no user-data migration or deletion is performed.
The real Session/Transcript Human path remains in place. The additional
FeatureHost MessagingService write path and public FFI/default-link legacy
surfaces remain unresolved. The owner inventory and RTM distinguish these.

At commit preparation, independent static review and behavior tests are recorded
in `projects/telegram-desktop-rust/evidence/round14-native-startup-retention.json`;
new exact-H Actions execution remains required. Backend Draft PR #137 has no
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

