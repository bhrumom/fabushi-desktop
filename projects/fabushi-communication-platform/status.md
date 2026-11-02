# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02  
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Current bounded batch (round 17): Native auth, Settings recovery and actual AppHost copy

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; inspected base
`23f737faba69b4f60eecb6ec1a031602f87f60dc` and live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Run `41451598911` is terminal
success, attempt 1. Complete logs confirm Human33, Agent4, private credential3,
AppHost composition11, Native6, FeatureHost7, Host6, C ABI7, normalization6,
future-salts2, protocol122, TCP9 including live, proxy4, Native credential1,
UI16 and Electron carrier10/auth8/package-copy3. The required jobs are
`125297875803` and `125297875447`; these results qualify the exact base only.

The same-H shipping artifact contains the actual Mach-O arm64 AppHost with
SHA256 `7d6dd995eb01fb4dd53c5e4195742d7294b8c0fd0c313d39fd6cdee848811818`.
Its normal/build dependency graph, defined C symbols and linked-library
inventory pass the legacy-boundary negatives. The actual CLI **and gateway**
consume the generic RuntimeHandle C ABI; direct old Telegram exports remain
force-linked in `mahayana-ffi`. No global ABI retirement is inferred.

Round 17 fills NMR-AC-28 in the existing six production subprocess scenarios:
125 refusal requests per legacy-auth matrix, all 15 typed Settings mutations,
full snapshot/disk restart oracles, invalid input, persistence failure and
recovery. NMR-41 fixes the inherited proxy-removal ordering defect that lost
an adjacent surviving preference; an identity-based 30-case owner table and
public dispatch/restart cases cover the deliberate fixed-U correctness change.
Composition now requires 12 tests. NMR-40 copies the freshly built actual
AppHost through the locked production FileSet and records exact bytes, mode,
provenance, exclusions and copied-binary symbols/libraries in Actions.

These round-17 changes are implemented/tests-authored, with containing-commit
execution pending at preparation. No local executable validation was run.
The single owner inventory, ledger and RTM distinguish source review,
implementation, production wiring, authored tests, exact-H execution and
independent acceptance. Evidence: `evidence/round17-auth-settings-actual-package.json`.

Backend PR #137 stays open/draft at `06047d3ea69313ec4cfec58bbee36a7afc9cf007`,
unmerged and without reviews. Its exact-H 29 handler/source and 10 SQLite
contracts passed, but current production Worker provenance and authenticated
afterId=0 pagination/denial/recovery remain unverified. Current backend main
source still rejects an explicit afterId=0. No backend mutation, merge or
deployment was performed. Full source/legal closure, all product paths,
signed/installed supported-platform packages, external FFI callers, UI/a11y,
Native proxy transport, complete Bot regression, upgrade/rollback, soak and
independent ACCEPT remain open. Complete remains false.

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



