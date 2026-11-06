# Quality 执行时机：批次反馈与最终完整验收

Status: active
Updated: 2026-11-04
Authority: FMEP-001 Revision 3，`docs/specs/fabushi-deferred-ci-migration.md`

按 FMEP-001 Revision 2：阶段 A 以功能闭环成批实现，并在 GitHub Actions 执行受影响编译、合同/状态机及必要接线与风险验证；允许必要 push/PR/dispatch。无依赖批次并行，关键公共合同失败先修复。阶段 B 冻结最终候选完成所有适用完整验收，阶段 C 独立 ACCEPT 后发布。所有可执行验证仍仅 Actions；未运行项 not-run，局部通过不等于全产品 verified。

复用现有 TEST_STRATEGY、RTM、oracle 与 harness。按改动风险选择最小必要检查；身份、权限、同步、数据迁移和恢复的关键负例不得留到最后才发现。每批记录 exact H/U、选择理由、run/job/artifact 和失败分类。同 H/输入有效结果复用，修改后只先重验失效与受影响项。最终 Hfinal 仍覆盖全部 required gates、真实环境和独立审阅。

旧“全量完成前禁止 CI”已撤销；全量源验收不能作为任何局部实现的循环前置条件，局部通过也不能宣称全量完成。

Revision 3（FMEP-FAST-09）：阶段 A 可在本机跑聚焦的编译/类型检查/单元/合同命令作为 `local-feedback`，用于尽早发现错误；它不是证据，不推进任何 ledger/RTM/批次门，也不得报告为 pass。上文"所有可执行验证仍仅 Actions"自 2026-11-04 起理解为"所有产生证据的验证仅 Actions"。
