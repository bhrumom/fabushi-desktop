# Fabushi Quality & Test Governance

Status: active / normative  
Parent: FBCP-001 Revision 7 / TDRP-001 Revision 9  
Execution: all executable verification in GitHub Actions only  
Updated: 2026-10-07

## Purpose

本目录把产品 Spec 转成可执行、可追溯、可独立验收的质量体系。原则：Requirement -> Oracle/Invariant -> Test Design -> GitHub Actions Execution -> Evidence -> Defect/Pass -> Independent Acceptance。

方法参考 ISO/IEC/IEEE 29119 family、ISTQB risk-based testing、ISO/IEC 25010、WCAG 2.2 AA 与 OWASP verification 思路；不声称获得认证。

## Normative documents

- `TEST_STRATEGY.md` — 总体质量模型、风险、测试层、角色、流程。
- `requirements-traceability-matrix.md` — requirement/AC 到 oracle/test/evidence 的双向映射。
- `evidence-contract.md` — current-head passing evidence 合同。
- `release-entry-exit-criteria.md` — RC entry/exit 与 ACCEPT 条件。
- `defect-regression-policy.md` — defect/escaped-defect/flaky/regression。
- `exploratory-test-plan.md` — 独立风险导向 packaged journey/video review。
- `plans/functional-test-plan.md`、`plans/ui-test-plan.md`、`plans/temporal-reliability-test-plan.md`、`plans/performance-soak-test-plan.md`、`plans/security-privacy-test-plan.md`。
- `oracles/conversation-turn-oracle.md`、`oracles/transcript-ordering-oracle.md`、`oracles/reconciliation-oracle.md`。
- `release-acceptance-report-template.md` — 独立 release verdict 模板。

规范存在不代表 gate 已实现。verified 仍要求 current-head GitHub Actions evidence。
