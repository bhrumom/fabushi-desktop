# Fabushi 完整通信能力等价重写 — Source of Truth

Status: active  
Project ID: FBCP-001  
Revision: 8
Date: 2026-10-31
Execution addendum: FMEP-001 Revision 1 (2026-10-31)
Repository: `bhrumom/fabushi-desktop`

## Canonical statement

最终产品是一个完整 Fabushi Bot。必须逐文件、逐模块理解 `telegramdesktop/tdesktop`，把全部非 UI 代码职责用最合适语言重写进 current canonical Fabushi；C++ 产品职责默认 Rust；原 UI 改为 Fabushi UI，但 UI 文件的非 UI 逻辑及所有功能必须保留。最终同时实现现有 Bot 全部能力与完整源范围通信能力。

这不是 Telegram 绑定/Provider/外置 runtime，也不是研究后挑选少量功能。禁止“research complete”代替“production complete”。

## Normative documents

- root `AGENTS.md`
- `docs/specs/fabushi-deferred-ci-migration.md` — FMEP-001 Revision 1；本迁移执行时机、并行批次、零 CI 保存及集中验证的权威补充
- `docs/specs/fabushi-bot-communication-platform.md` — FBCP-001 Revision 8
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` — TDRP-001 Revision 10
- current canonical Bot architecture/spec and exact-head implementation
- `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md` and `module-map.md`
- `projects/fabushi-communication-platform/ui-information-architecture.md`
- `projects/fabushi-communication-platform/design-system.md`
- `projects/fabushi-communication-platform/ui-component-contract.md`
- `projects/fabushi-communication-platform/canonical-screen-patterns.md`
- `projects/fabushi-communication-platform/visual-acceptance.md`
- `projects/fabushi-communication-platform/quality/README.md` and all normative quality plans/oracles/contracts under `quality/`
- `projects/fabushi-communication-platform/quality/execution-timing.md`
- current task, existing architecture-map, dossiers, ledger, ADR and evidence

以上新 Revision 替代旧 project metadata/研究材料中 research-only、选择性吸收、where accepted 或 blocked 可算最终验收的口径。旧数据保留历史身份，不自动改为 verified；现有 schema/validator 尚需按新合同扩展。

## Execution timing authority

FMEP-001 细化 FBCP-QA-19/TDRP-EXEC-01，并优先解释所有关联任务、质量计划与旧提示词的验证时机。阶段 A 全量实现与融合期间不进行任何迁移 CI，不做本机验证；测试代码、oracles、fixtures、workflow 可编写但不执行。完整实现闭合后冻结 H，阶段 B 才集中验证、修复与重验；阶段 C 独立接受后发布。

FBCP §2 中“在 GitHub Actions 确认基线”是阶段 B 的正式证据，不是阶段 A 获取源码或开始实现的前置条件。阶段 A 以只读 Git/API 锁定 U、枚举并理解来源，最终机器验证不提前运行。P0 的可执行 gate 也依此延期，不以尚无 CI 结果阻塞无依赖实现。

实现/测试源码检查点应通过已证明不会触发 CI 的持久云端保存机制保留；未证明无触发前禁止更新实现 ref/PR。不能仅靠 `[skip ci]` 假定所有事件已隔离，也不能长期只存本机。具体约束按 FMEP-001 §5；本次只更新文档，未宣称已改好 workflow 触发配置。

## Hard rules

- 全仓库与递归依赖逐文件对照；文件、责任、capability、target symbol、production entrypoint 与测试双向追溯。
- 除 UI 表现实现外全部产品职责等价重写；mixed UI 文件必须拆出业务，不允许按目录整块忽略。
- current existing owner first；新 owner 只允许批准 ADR 下的最小新增。
- 无第二 Identity/Conversation/Message/Resource 真相、无平行 CommunicationCore/Telegram runtime。
- Fabushi 自有身份/通信/同步/媒体/通话/推送服务；不用 Telegram 网络不是省略服务端功能的理由。
- Human、Agent、群、频道、混合房间使用同一 shell/transcript/composer。
- 所有同类产品表面遵守 single-composition：统一 ConversationWorkspace/Profile/列表/资源/设置等 canonical root；Human/Agent/Group/Channel/新能力差异只能是 typed capability/section/renderer/action/panel/overlay。
- Telegram 中当前 Fabushi 完全没有的能力不得丢弃：先证明 existing owner 不存在，再以 ADR 新增最小 source-neutral owner，并接入统一 shell/canonical identity/resource/permission；专用 surface 不得成为第二应用。
- 微信式结构：左竖栏 + 列表/搜索 + 主工作区；消息、联系人、插件市场均为一级入口，市场复用现有 Plugins/MCP owner。
- UI 入口按作用域而非源码来源分层：一级领域 / collection / creation / object action / detail / capability surface / global command / system surface；每个可见能力只有一个主要入口与 canonical route。
- Group/Channel/Topic 等是统一 Conversation 体系的 kind/filter；新建私聊/群组/频道/Agent/混合房间复用一个 typed ConversationCreation flow，不产生独立 Group/Channel App。
- Search 只有一个 canonical owner：入口内、对象内、Universal Search 与所有 Participant/member/admin picker 复用 typed query/result/provider/permission 合同；本地/远端索引位于同一 owner 后。
- UI 必须使用 Fabushi semantic design tokens、canonical components 与 canonical screen patterns；AI/开发者不得按模块自由发明新的 visual grammar。
- 当前 recovered `sand-*` / `cursor-*` 可作为兼容实现细节，但新迁移 UI 不直接把来源命名当公共 API；目标层为 Fabushi-owned semantic aliases/wrappers。
- 核心 UI 变更的 current-head GitHub Actions visual regression、light/dark、locale、responsive、keyboard/a11y、reduced-motion 和状态矩阵证据在阶段 B 取得。
- 功能/QA 使用 requirement→oracle/invariant→test case→execution evidence→independent verdict 的双向 RTM；没有 traceability 不得 verified。
- 动态 Agent/消息/UI 行为必须在阶段 B 做 temporal acceptance；final/terminal 在 settlement、切换、reconnect、reload、restart 后不可消失/回滚/被 intermediate 替代。
- 人工/后阶段发现的 defect 必须形成永久 regression case 和 gap analysis；flaky/skipped/blocked/not-run 不算 pass。
- release candidate 必须由独立验收者复核 packaged timeline/screenshots/video/evidence，0 open P0/P1/blocker 才可 ACCEPT。
- 产品名/图标/窗口/文案/安装更新全部 Fabushi；无 Telegram/Grok Bot/Gok Bot 产品品牌；必要法律声明和 provenance 保留。
- 所有可执行验证仅 GitHub Actions，不沿用本项目旧 htch-runtime allowance。
- 阶段 A 按固定 U、现有 owner 与依赖图成批并行实现，变更范围增量审读，单一集成人合入公共接口和账本；不重复开修复线，不等待 CI。
- 完整迁移需要 0 in-scope open blockers、0 omitted/unmapped、0 stub/fake fallback 和最终冻结候选的生产/发布证据。

## Snapshot, not live truth

Revision 3 读取：main `860f03a8553c779fe006c7826f190c3014a571dc`；PR #20 已合并（merge `ee66bacdf47f36af2ae96c0a8a8ec401460426e7`）。执行前读取 live authority，不能继续把 PR #20 当作未合并工作分支；开工固定本轮 U 后，不因上游不断前进而自动重开范围。

此前记录的上游 discovery HEAD 为 `72b3b71c3d6e450e5ef94a3112dd750a0168aa0b`（tree `b4095fab77f923fca2a4b99d710ec333aa6757ab`）；`f23c37857220eb84f8559f0901ea26fb304b564b` 与 `33261535a0e747f125e0ed25486f01e556330677` 为历史证据。当前实际 U 必须读取同一 source revision 下的 lock/inventory/ledger/dossiers 后确认；本执行补充不重新建立上游基线、不改变机器账本状态，旧 Actions 不证明新 baseline 完整。

## Next task / completion

`management/tasks/P0-product-domain-and-telegram-absorption.md`

完整执行提示词：`management/prompts/full-migration-deferred-ci.md`。

先做所需基线差异与 current owner 对照，再按依赖批量推进真实实现。遇到阻塞记录解除条件并推进无依赖任务。全量代码融合完成只可进入候选验证；全产品完成仍以 FBCP-001 全部适用 AC 和独立 ACCEPT 为准。
