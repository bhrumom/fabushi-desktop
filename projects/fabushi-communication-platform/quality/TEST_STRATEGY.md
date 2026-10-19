# Fabushi Test Strategy

Status: active / normative  
Strategy ID: FQT-001  
Revision: 1  
Parent: FBCP-001 Revision 7  
Updated: 2026-10-07

## Quality model

验证 functional suitability、reliability、performance efficiency、usability/accessibility、security/privacy、compatibility、maintainability/observability 与 release/recovery。UI 好看、build 绿色或单一路径可用均不是完整质量。

## Risk classification

- P0 critical: 数据/权限/账号隔离、消息丢失/串线、terminal result 错误、安全/隐私、不可恢复 corruption。
- P1 high: ordering/reconciliation、Agent/tool settlement、reconnect/restart、发送/附件、关键 UI route/controls、升级/回滚。
- P2 medium: 次要交互、非关键 settings、低频 presentation states。
- P3 low: cosmetic/minor。

P0/P1 默认需要多层测试，不能只靠 E2E。

## Required test levels

1. Unit/table-driven：纯函数、validation、projection、formatting、state transitions。
2. Property/state-machine：ordering、dedupe、idempotency、replay、merge、cancel/restart、组合 event sequences。
3. Contract：Rust/Host/bridge/renderer/service schemas、IDs、terminal/error/cancel semantics。
4. Integration：real owners/stores/reducers/projections 注入复杂时序。
5. Functional E2E：完整 Electron shipping composition 真实 user journey。
6. Temporal/recovery E2E：关键状态点、quiet settlement、switch/reconnect/reload/restart。
7. UI functional + visual + a11y：control/routes/state + layout/style/accessibility/responsive/locale。
8. Fault/security/performance/soak：适用风险域。
9. Independent exploratory acceptance：独立 reviewer 审阅 packaged timeline/video/evidence。

## Test design techniques

使用 equivalence partitions、boundary values、decision tables、state-transition tests、pairwise/combinatorial、property/model-based、error guessing（补充）和 risk-based exploratory。关键状态机至少有合法/非法 transition cases。

## Standard flow

Discover test basis -> risk -> oracle/invariants -> RTM/test design -> fixture/environment -> implement tests -> GitHub Actions -> evidence review -> defect/regression -> independent acceptance -> release summary。

## Pass/fail semantics

Pass = expected behavior + required evidence。Fail = oracle violation。Blocked/skipped/not-run = no pass evidence。Flaky = unresolved nondeterminism；critical gate 视为 block/fail。禁止 rerun-until-green。

## Cross-review semantics

Independent cross-review covers the complete declared diff, not a convenient subset. General and specialist reviewers remain read-only and report concrete correctness, integration, lifetime, reuse, structure, performance, security, data-safety or maintainability failures rather than preference-only suggestions. Specialist NOT_APPLICABLE requires changed-path/hunk-specific proof; uncertainty remains applicable.

Synthesis rechecks findings against current code, merges duplicates, drops unsupported findings, and rejects unsupported NOT_APPLICABLE decisions. A NEEDS_CHANGES-style gate requires at least one supported blocking failure. After fixes, focused re-review confirms every accepted fix and checks regressions in edited functions plus relevant callers/consumers without reopening unrelated unchanged code.

### Arbitration semantics

Cross-review scope is frozen once per session and snapshotted per round so reviewers judge the same declared change, including explicitly in-scope pre-existing uncommitted work. Round 1 uses fresh blind independent general + applicable specialist reviewers in parallel; later rounds use a fresh focused general reviewer and only the specialist lenses that originated accepted findings. Reviewer reports remain mutually blind; persistent state belongs to the source-neutral runner/ledger, not to a provider process.

Every synthesized finding is arbitrated before fixes as `FIX`, `REJECT`, `DEFER`, or `COMPROMISE`; `NON_BLOCKING` never starts a fix round alone. Reversals must reference the earlier decision and add a previously unweighed concrete failure before reconsideration. One bounded supersession is allowed; a second attempted reversal is a human-decision stop. After any FIX, focused re-review must confirm the previous finding and check the touched functions plus adjacent callers/consumers against the complete frozen scope.

### Focused re-review and stopping

The loop terminates only on convergence (no FIX after valid reversal handling), configured round cap (default 10), non-convergence/scope expansion into another subsystem, or a required human product/arbitration choice. Open real defects in changed code remain blocking regardless of round pressure.

## Independence and environment

Implementer 可写 tests，但 release reviewer 必须独立复核。FBCP/TDRP 所有 executable verification 只在 GitHub Actions；fixture/locale/theme/viewport/network/fault/account identity 必须进入 manifest。
