# Fabushi 完整通信能力等价重写 — Source of Truth

Status: active  
Project ID: FBCP-001  
Revision: 3  
Date: 2026-10-07  
Repository: `bhrumom/fabushi-desktop`

## Canonical statement

最终产品是一个完整 Fabushi Bot。必须逐文件、逐模块理解 `telegramdesktop/tdesktop`，把全部非 UI 代码职责用最合适语言重写进 current canonical Fabushi；C++ 产品职责默认 Rust；原 UI 改为 Fabushi UI，但 UI 文件的非 UI 逻辑及所有功能必须保留。最终同时实现现有 Bot 全部能力与完整源范围通信能力。

这不是 Telegram 绑定/Provider/外置 runtime，也不是研究后挑选少量功能。禁止“research complete”代替“production complete”。

## Normative documents

- root `AGENTS.md`
- `docs/specs/fabushi-bot-communication-platform.md` — FBCP-001 Revision 3
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` — TDRP-001 Revision 5
- current canonical Bot architecture/spec and exact-head implementation
- `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md` and `module-map.md`
- current task, existing architecture-map, dossiers, ledger, ADR and evidence

以上新 Revision 替代旧 project metadata/研究材料中 research-only、选择性吸收、where accepted 或 blocked 可算最终验收的口径。旧数据保留历史身份，不自动改为 verified；现有 schema/validator 尚需按新合同扩展。

## Hard rules

- 全仓库与递归依赖逐文件对照；文件、责任、capability、target symbol、production entrypoint 与测试双向追溯。
- 除 UI 表现实现外全部产品职责等价重写；mixed UI 文件必须拆出业务，不允许按目录整块忽略。
- current existing owner first；新 owner 只允许批准 ADR 下的最小新增。
- 无第二 Identity/Conversation/Message/Resource 真相、无平行 CommunicationCore/Telegram runtime。
- Fabushi 自有身份/通信/同步/媒体/通话/推送服务；不用 Telegram 网络不是省略服务端功能的理由。
- Human、Agent、群、频道、混合房间使用同一 shell/transcript/composer。
- 微信式结构：左竖栏 + 列表/搜索 + 主工作区；消息、联系人、插件市场均为一级入口，市场复用现有 Plugins/MCP owner。
- 产品名/图标/窗口/文案/安装更新全部 Fabushi；无 Telegram/Grok Bot/Gok Bot 产品品牌；必要法律声明和 provenance 保留。
- 所有可执行验证仅 GitHub Actions，不沿用本项目旧 htch-runtime allowance。
- 完整迁移需要 0 in-scope open blockers、0 omitted/unmapped、0 stub/fake fallback 和 exact-head 生产/发布证据。

## Snapshot, not live truth

Revision 3 读取：main `860f03a8553c779fe006c7826f190c3014a571dc`；PR #20 已合并（merge `ee66bacdf47f36af2ae96c0a8a8ec401460426e7`）。每次执行重新读取 live refs，不能继续把 PR #20 当作未合并工作分支。

上游 discovery HEAD `f23c37857220eb84f8559f0901ea26fb304b564b` 与旧 baseline `33261535a0e747f125e0ed25486f01e556330677` 不同。完整 rebaseline 尚待执行，旧 graph/dossiers/lock 不能证明新基线完整。

## Next task / completion

`management/tasks/P0-product-domain-and-telegram-absorption.md`

先做新基线全量差异与 current owner 对照，再按已明确责任推进真实实现。遇到阻塞记录解除条件并推进不依赖该阻塞的下一项，不能放宽 gate。此次只修订文档，未完成模块迁移或 UI 改造；全产品完成以 FBCP-001 AC-01 至 AC-24 全部通过为准。
