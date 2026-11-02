# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02  
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Current bounded batch (round 16): AppHost dependency and package boundary

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; inspected base
`adf15caadac5775fdd8582227cad3bbf9c93cf0f` and live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Main and backend are unchanged.
Run `41447812811` finished success: complete logs confirm Human33, Agent4,
private credential3, AppHost composition11, Native6, FeatureHost7, Host6,
C ABI7, Host direct-dependency negative, normalization6, future-salts2,
protocol122, TCP9 including live, proxy4, Native credential1, UI16 and
Electron carrier10/auth8. These qualify that exact base only.

NMR-38 removes the impossible AppHost Telegram lease and unreachable forwards,
retaining all twelve explicit refusal routes, registration validation order,
Native auth/Agent state and Settings persistence/publication semantics. The
legacy proxy projection dependency is test-only; the independent shipping lock
prunes only the eight packages made unreachable, with no version upgrades.
NMR-39 restricts Electron `resources/bin` to its four canonical executable
names and Windows forms, with nested-file exclusion. Staging files, original
SQLite/sidecars/keys, retained FeatureHost files and rollback readers are kept.
The CLI is a real generic C ABI consumer; FFI/direct Telegram exports are not
deleted or declared globally retired.

Expanded Native6 behavior tests, three actual locked packager-copy cases and
the shipping dependency/build/symbol gates are authored. Execution at the
containing commit is pending at preparation; no local executable validation
was performed. The single ledger row, owner inventory and RTM retain separate
implementation, execution and final acceptance states. Preparation and prior-H
results are in `projects/telegram-desktop-rust/evidence/round16-apphost-package-boundary.json`.

Backend Draft PR #137 remains open/draft at
`06047d3ea69313ec4cfec58bbee36a7afc9cf007`, without review or formal deployment
authorization. Its handler29/SQLite10 CI passed, but current production Worker
SHA/version and real authenticated afterId=0 pagination/empty/denial/recovery
are unverified. Full source/legal, product scope, supported-platform packages,
UI/a11y, Native proxy support, complete Bot regression, upgrade/rollback, soak
and independent ACCEPT remain open. Complete remains false.

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



