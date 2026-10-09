# 2026-10-07 — 全量等价重写与统一 UI 规范修订记录

Type: spec change / user-directed scope clarification  
Product implementation acceptance: not claimed

## User-directed change

对 `telegramdesktop/tdesktop` 逐代码文件、逐模块理解，把全部非 UI 职责以最合适语言重写迁移到现有 Fabushi Bot 架构（C++ 默认 Rust），成为具备 Bot 与完整通信功能的一个产品。UI 按微信桌面信息结构设计，左侧竖栏提供消息、联系人等主入口，插件市场迁入该竖栏。产品品牌统一 Fabushi，不保留 Telegram/Grok Bot/Gok Bot 标识。

原截图仅用于理解布局，不公开其中个人信息或复制品牌素材。

## Durable changes

- FBCP-001 Revision 7：完整功能并集、single-composition、UI information architecture/interaction contract、统一创建与统一 Search、服务端/品牌/发布要求与 AC-01 至 AC-50。
- TDRP-001 Revision 9：全仓库/递归源清单、逐文件和逐责任对照、逐模块行为合同、UI entry/search ledger、实际重写与生产取证；废止 research-only 完成定义。
- 两个项目 SOURCE_OF_TRUTH、module-map 与 P0 task 同步新口径。
- 保留 current-owner-first、Coordinator/Host/Runner、native Fabushi network、无平行 runtime 和 source-informed license/provenance 约束。
- 本项目所有可执行验证收紧为 GitHub Actions only。

## Facts inspected

- Fabushi main: `860f03a8553c779fe006c7826f190c3014a571dc`。
- Desktop PR #20：merged；head `798cf51d96cb1cb98cf657af212bb47274ccb701`；merge `ee66bacdf47f36af2ae96c0a8a8ec401460426e7`。
- tdesktop discovery dev HEAD: `f23c37857220eb84f8559f0901ea26fb304b564b`。
- 历史 migration baseline: `33261535a0e747f125e0ed25486f01e556330677`。

这些只是读取快照。完整 upstream rebaseline/recursive inventory 未在本次执行；旧 graph/ledger/lock/dossiers 不自动升级。

## Verification and remaining work

本次是文档修订；未运行 build/test/generator/schema/benchmark/fuzz/package/acceptance，未改 production code、现有 ledger 状态或 upstream lock，未声称已完成 UI、全文件理解或功能迁移。

本次进一步把 UI 迁移原则从“统一 workspace”细化为可执行的信息架构合同：功能入口按作用域分类；Group/Channel/Topic 留在统一 Messages/Conversation 体系；新建私聊/群组/频道/Agent/Hybrid 复用 typed ConversationCreation flow；Search 统一为 contextual/object/Universal 三层 UX 与一个 canonical Search owner；picker、权限、本地/远端合并、排序/去重/分页/取消/stale fencing 都纳入 migration ledger 与 gate。

本轮继续补齐 AI 可稳定执行的 UI 工程合同：新增 Fabushi design system、canonical component contract、canonical screen patterns 与 visual acceptance；把 typography/color/spacing/shape/motion/icon/avatar/content、legacy token compatibility、component ownership、screen/state matrix、visual regression、keyboard/a11y/locale/responsive/large-data 证据纳入 normative requirements。

本轮再补齐专业 QA/test governance：新增 Test Strategy、RTM、evidence/release/defect/exploratory contracts、功能/UI/temporal/recovery/performance/security plans，以及 Conversation Turn / Transcript Ordering / Reconciliation oracles。质量门从 eventual-state checks 升级为 requirement→oracle→multi-layer tests→temporal/visual evidence→独立 acceptance；人工发现的 defect 必须进入永久 regression ledger。

下一执行以更新后的 P0 task 为准：在 GitHub Actions 建立一致 baseline、扩展既有账本/gate、重新审计实际实现并按无阻塞 owner 持续推进。blocked 是待办，不是完成；完整交付以 FBCP-001 全部 AC 与 current-head 生产证据为准。
