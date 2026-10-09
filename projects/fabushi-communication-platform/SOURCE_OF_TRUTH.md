# Fabushi 完整通信能力等价重写 — Source of Truth

Status: active  
Project ID: FBCP-001  
Revision: 7  
Date: 2026-10-07  
Repository: `bhrumom/fabushi-desktop`

## Canonical statement

最终产品是一个完整 Fabushi Bot。必须逐文件、逐模块理解 `telegramdesktop/tdesktop`，把全部非 UI 代码职责用最合适语言重写进 current canonical Fabushi；C++ 产品职责默认 Rust；原 UI 改为 Fabushi UI，但 UI 文件的非 UI 逻辑及所有功能必须保留。最终同时实现现有 Bot 全部能力与完整源范围通信能力。

这不是 Telegram 绑定/Provider/外置 runtime，也不是研究后挑选少量功能。禁止“research complete”代替“production complete”。

## Normative documents

- root `AGENTS.md`
- `docs/specs/fabushi-bot-communication-platform.md` — FBCP-001 Revision 7
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` — TDRP-001 Revision 9
- current canonical Bot architecture/spec and exact-head implementation
- `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md` and `module-map.md`
- `projects/fabushi-communication-platform/ui-information-architecture.md`
- `projects/fabushi-communication-platform/design-system.md`
- `projects/fabushi-communication-platform/ui-component-contract.md`
- `projects/fabushi-communication-platform/canonical-screen-patterns.md`
- `projects/fabushi-communication-platform/visual-acceptance.md`
- `projects/fabushi-communication-platform/quality/README.md` and all normative quality plans/oracles/contracts under `quality/`
- current task, existing architecture-map, dossiers, ledger, ADR and evidence

以上新 Revision 替代旧 project metadata/研究材料中 research-only、选择性吸收、where accepted 或 blocked 可算最终验收的口径。旧数据保留历史身份，不自动改为 verified；现有 schema/validator 尚需按新合同扩展。

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
- 核心 UI 变更必须有 current-head GitHub Actions visual regression、light/dark、locale、responsive、keyboard/a11y、reduced-motion 和状态矩阵证据。
- 功能/QA 使用 requirement→oracle/invariant→test case→execution evidence→independent verdict 的双向 RTM；没有 traceability 不得 verified。
- 动态 Agent/消息/UI 行为必须做 temporal acceptance；final/terminal 在 settlement、切换、reconnect、reload、restart 后不可消失/回滚/被 intermediate 替代。
- 人工/后阶段发现的 defect 必须形成永久 regression case 和 gap analysis；flaky/skipped/blocked/not-run 不算 pass。
- release candidate 必须由独立验收者复核 packaged timeline/screenshots/video/evidence，0 open P0/P1/blocker 才可 ACCEPT。
- 产品名/图标/窗口/文案/安装更新全部 Fabushi；无 Telegram/Grok Bot/Gok Bot 产品品牌；必要法律声明和 provenance 保留。
- 所有可执行验证仅 GitHub Actions，不沿用本项目旧 htch-runtime allowance。
- 完整迁移需要 0 in-scope open blockers、0 omitted/unmapped、0 stub/fake fallback 和 exact-head 生产/发布证据。

## Snapshot, not live truth

Revision 3 读取：main `860f03a8553c779fe006c7826f190c3014a571dc`；PR #20 已合并（merge `ee66bacdf47f36af2ae96c0a8a8ec401460426e7`）。每次执行重新读取 live refs，不能继续把 PR #20 当作未合并工作分支。

上游 discovery HEAD 已 rebaseline 到 `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`（tree `5030985204963cbbd362ced7412d231b04ebd0cc`）；`42f8a36d43b8c805bc821905bea4cfeb3af1d41d` 与更早 authority 仅作历史证据。当前 recursive denominator 16,123；read-through 5,585；unread 10,538；unknown 15,844；source closure 仍 open，predecessor Actions 不得证明新 baseline。

## Next task / completion

`management/tasks/P0-product-domain-and-telegram-absorption.md`

先做新基线全量差异与 current owner 对照，再按已明确责任推进真实实现。遇到阻塞记录解除条件并推进不依赖该阻塞的下一项，不能放宽 gate。此次只修订文档，未完成模块迁移或 UI 改造；全产品完成以 FBCP-001 AC-01 至 AC-50 全部通过为准。

Current live authority (2026-10-09): `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (root tree `5030985204963cbbd362ced7412d231b04ebd0cc`), three commits ahead of historical `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`; 15 root paths changed (12 modified, 3 added), recursive denominator is 16,123, read-through is 5,585, unread is 10,538, unknown is 15,844, omitted is 0, and source closure remains open.
