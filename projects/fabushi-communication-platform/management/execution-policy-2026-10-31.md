# 2026-10-31 — 全量开发零 CI 与集中验证执行补充

Type: user-directed specification/prompt update  
Read base: `ebb902b6262f8b4e92cc233dd1e318a379734dce`  
Product implementation or acceptance: not claimed

## Change

用户要求将 Spec 和总提示词改为“最快最好的方案，迁移前期不进行任何 CI，在全部迁移融合完成后再统一验证”。本次以已有 FBCP Revision 8 / TDRP Revision 10 的 development-first 条款为基础，新增 FMEP-001，不回退已有功能/架构/UI/质量内容。

明确三阶段、实现完成与验收完成分离；一次锁定 upstream、按 owner 成批并行、共享文件单写、增量阅读；前期测试代码随实现编写但不运行；全量实现冻结后统一并行验证；失败分类修复、同 H 复用有效结果，最终 Hfinal 保留完整 required evidence。

消除循环依赖：正式 source/inventory/RTM/编译/部署 gate 不作为阶段 A 开工前置；生成/构建产物属于最终验证。安全云端保存不以触发 CI 为代价，不把 skip commit/draft PR 当全事件禁用，也不要求长期仅在本机保存。

## Files

- 新增 `docs/specs/fabushi-deferred-ci-migration.md`。
- 两个 SOURCE_OF_TRUTH 引用并解释该执行补充。
- 新增 `quality/execution-timing.md`，统一现有质量计划执行时机。
- 新增可直接复制的 `management/prompts/full-migration-deferred-ci.md`，取代旧逐模块 CI 提示词。

## Scope / non-claims

仅修改文档与提示词；不改 production、不改 workflow、不部署、不运行构建/测试/验证器。没有修改 lock/inventory/ledger 的实际源身份或 accepted flags，没有将现有迁移标成已完成。

文档保存提交携带 skip 指令；远端 ref/blob 回读只是保存确认，不是产品测试。是否已存在完整的前期 CI 隔离机制仍须实现会话按实际触发路径确认；本补充不将该能力伪报为已实现。
