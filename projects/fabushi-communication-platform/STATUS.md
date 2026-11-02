# FBCP Native Communication Absorption — Status

> 2026-11-02：下述历史快照不代表当前生产状态；使用前按 FMEP-001 §12 核对当前批次实际入口与 owner，并增量更新。当前已有原生 Human 消息生产接线，旧“未实现”行不能作为新建平行 owner 的依据；完整迁移仍未验收。

Date: 2026-11-02  
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 2  
Complete: false

## Current bounded batch (round 20): recover exact-H shipping validation

Fixed U remains `f985723b552b30fe1c17d377d57f44e61a15a8c4`; inspected base
`151d3ffe6382219dab98fb317e59fe5faa8cea01` and live main
`5528062177e3865eef60a073a066cca144f9e3e8`. Run41465224038 failed after passing
Human33/Agent4/private3, AppHost12/Native6/FeatureHost7/Host6/ABI7/CLI3 and UI16.
Gateway executable was4/5: a shared persistent test file suppressed the second
test's first request. NMR-45 isolates real persistent fixtures and strengthens
duplicate/restart oracles without changing production deduplication.

Electron carrier10/auth8/copy3/metadata15 passed, but the historical installer
experiment rejected a valid cross-platform ZIP blockmap before reaching stale
DMG metadata. NMR-43 now compares complete bounded decoded map content and
records actual sidecar hashes;18 authored cases and the unchanged digest-pinned
real package experiment require containing-H Actions. Gateway lib9, protocol122,
TCP9 and actual-binary package-copy did not execute at the inspected base.

The actual Human path remains existing Composer -> Session/Transcript ->
FabushiNativeMessagingClient -> native HTTP -> the same SandAgentDb/events.
Production+Electron refuses standalone MessagingService before locks/storage.
CLI and gateway now use direct MahayanaHost; the old round16 C ABI consumer
statement is superseded. Retained FFI exports, external callers, old data and
rollback readers are not declared globally retired. No parallel owner or user
data deletion is introduced. This batch changes the existing release verifier
and test fixtures; it does not qualify a new signed installer or update service.

Evidence, exact old-run logs/artifacts, source review and required new gates:
`projects/telegram-desktop-rust/evidence/round20-blockmap-gateway-recovery.json`.
No local executable validation ran. The canonical ledger/RTM keep implementation,
authored tests, scoped execution and final acceptance separate; full source review
and legal disposition remain open with fixed U and unchanged source acceptance.

Backend PR #137 remains open/draft/unmerged at
`06047d3ea69313ec4cfec58bbee36a7afc9cf007`. Both exact-head handler29/SQLite10
runs passed. Historical production artifact12542026010 was inspected and digest
verified; it binds source33a26692ce2f74e3fdca3940dfc4cbfb3ede151b to the recorded
Worker version but only proves health/FDN enrollment checks at deployment time.
Current Worker identity and authenticated afterId=0 pagination/empty/denial/
recovery remain unverified; no backend merge or deployment is authorized.
Full functionality, supported-platform packages, UI/a11y, Native proxy/update
support, complete Bot regression, upgrade/rollback, soak and independent ACCEPT
remain open. Complete remains false.

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



