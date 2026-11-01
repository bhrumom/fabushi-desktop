# Quality 执行时机：开发零 CI，冻结后集中验收

Status: active / normative timing addendum  
Updated: 2026-10-31  
Authority: `docs/specs/fabushi-deferred-ci-migration.md` (FMEP-001 Revision 1)

本文件不建立新测试体系，只为现有 TEST_STRATEGY、RTM、oracles、各 test plan、visual acceptance、P0/task 的执行时机提供单一解释。

## 阶段 A

先完成全量 production migration、集成、UI、服务/数据/平台/打包输入，以及所需测试代码、fixture、oracle 和 workflow。所有执行保持 `not-run`，不触发任何迁移 CI，不运行本机测试或验证器。人工审读、源数据获取、文本编辑和不触发 CI 的安全 Git 保存允许。

“先有测试依据”仍成立；“每个模块 verified 后才能继续另一个模块”不成立。开发可以继续至全部实现闭合，再统一取得执行证据。已有复杂测试基础设施可复用，不为本阶段重复实现第二套 harness。

## 冻结

源码/差异盘点确认范围内职责均有真实实现和接线、测试已编写后冻结 H/U/依赖/环境。候选 readiness 不要求 prior CI pass；编译/生成/部署/安装真实性留给阶段 B 证明。存在真正来源或实现缺口则继续阶段 A，不假装全量完成。

## 阶段 B/C

现有全部适用质量门保留。阶段 B 在 Actions 中按依赖图并行、复用同 H 构件、集中执行；失败按 production/test/environment 归因。相同 H 的有效结果保留，修复新 H 只先跑受影响路径，最终候选取得全部 required exact-head evidence。阶段 C 才独立接受和发布。

截图、视频、DOM、存储、运行事件、性能样本与负面结果按 evidence-contract 保存；未运行/阻塞/波动不记 pass。不改变视觉、时序、安全、来源完整性和发布的既有标准。

## 消除循环前置条件

历史文件要求“P0/source rebaseline 必须先跑 Actions”时，分成开发期只读取得与最终正式验证，不阻止源码理解和实现。历史文件要求“全迁移完成后才可测”时，完成指 implementation closure，不是 tests passed。所有与 FMEP 冲突的旧提示词执行顺序停止使用。
