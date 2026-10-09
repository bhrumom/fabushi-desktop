# P0 — 全量源文件/模块与现有 Fabushi 架构对照

Status: active / not accepted  
Project: FBCP-001 Revision 7 / TDRP-001 Revision 9  
Updated: 2026-10-07  
Execution: all executable verification only GitHub Actions

## Goal

为全部 tdesktop 非 UI 代码职责的等价重写建立精确、无遗漏的输入和实施合同，而不是只做 Telegram 功能研究。最终完整实现原 Bot + 全部源范围通信能力，统一 Fabushi 品牌与微信式左竖栏布局。

## A. Re-read canonical architecture and source

重新读取 main、PR #20 状态和 current canonical exact HEAD。PR #20 在本次规范读取时已合并；不能继续从历史 open/draft 认知开始，也不能回退到旧实现分支。既有 Bot 架构/验收硬门继续有效。

重新读取 tdesktop discovery HEAD。当前 accepted discovery HEAD 为 `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`（root tree `5030985204963cbbd362ced7412d231b04ebd0cc`）；`42f8a36d43b8c805bc821905bea4cfeb3af1d41d` 与更早 authority 仅作历史证据。当前 recursive denominator 16,123；read-through 5,599；unread 10,524；unknown 15,844；source closure 仍 open，predecessor Actions 不得证明新 baseline。

## B. Inventory all files and understand all modules

每个源文件/依赖条目有 exact identity/hash、类型和责任，不能只统计目录。按 TDRP-SRC-02/SRC-03 全量覆盖，特别拆出 mixed UI 文件里的非 UI 逻辑。解析未知或 API 截断必须补读，无法访问保留 blocker。

逐模块 dossier 记录完整源文件/symbol、状态机、所有权、调用图、行为 oracle、错误/安全/生命周期/恢复/性能/平台与许可证。现有 capability graph 是旧基线索引，先审计差异，不是最终范围上限。

## C. Exact-head existing-owner inventory

读取真实 shipping entrypoints，而非根据旧聊天推测：

- product shell/左侧导航、conversation/list/workspace、transcript/cards、composer/drafts；
- account/identity、Shared Room/member、permissions、contacts、search；
- attachments/artifacts/resources、storage/recovery、notifications/settings/platform；
- Coordinator/Host/Runner、Agent、Computer、Plugins/MCP/marketplace、Automations/Task；
- 当前 calls/media/network/service owner（若存在）。

每个 source responsibility 填写 owner 候选、选择原因、target paths/symbols、语言、需修改状态/事件/接口、服务依赖、数据迁移、tests/production evidence。目标 owner 尚未合适时做 rejected-owner analysis 与最小 new-owner ADR，禁止整块另建 CommunicationCore/Telegram subsystem。

同时建立 canonical composition inventory：ProductShell、ConversationWorkspace、ConversationList、Transcript、Composer、Participant/Profile、Resource viewer/editor、Search、Settings、Marketplace、Task/Automation 等每类概念只能有一个 root。每个责任写 `composition_root + composition_slot + variation_axes`；禁止 Bot/Human/Group/Channel/来源型完整 UI/状态栈分叉。

如果某项 Telegram 能力当前完全没有 owner，不能标 N/A：记录 `novel_capability`、所有 rejected owners、owner-absence evidence、最小 state/lifecycle responsibility 和 ADR。批准后只能新增 source-neutral capability owner，并通过 typed contracts 接入统一 shell、identity/resource/permissions/navigation。

## D. Language / production contract

C++ 非 UI 产品逻辑默认 Rust；前端表现与交互投影使用现有 React/TS；系统特有职责采用最薄平台适配。所有非默认语言/通用依赖选择写明理由与安全/许可证边界。

不得以 TDLib、原 C++ core、FFI wrapper、sidecar、外置 Telegram Web 或“暂时 Provider”替代源责任重写。已有实现如违反目标，记录 cutover、数据兼容与移除路径，不能因已存在就自动接受，也不在本次文档任务中盲删生产代码。

## E. Native service completeness

对每项客户端远端依赖列 client/server contract、Fabushi owner、认证/权限/持久化、部署与真实 E2E。自有 messaging/sync/presence/media/call/push 和必要商业能力支持现有模型，不形成第二产品。

缺服务、账号、密钥/签名或产品配置时标 blocked；不得以 mock、仅 UI 或“不用 Telegram 网络”当成功能完成/排除。

## F. UI / branding plan

明确同一 shell 内：左侧窄竖栏 -> 当前入口列表/搜索 -> 主工作区 -> 可选详情。消息、联系人、插件市场必须一级可达；插件市场迁移复用原安装/权限/Plugins/MCP owner，旧路由兼容不复制状态。

Agent、任务、Computer、Automations、账号设置等现有主能力仍可达。消息列表/正文、草稿、未读、运行中任务和 Agent 输出维持同一状态。品牌清单覆盖窗口、头像、Logo、导航、默认文案、通知、托盘、安装和更新；统一 Fabushi，法律声明/provenance 例外精确记录。不得把用户原截图或其私人数据提交公开仓库。

UI 计划必须给出 single-composition matrix：Conversation 只允许一个 workspace、Profile 只允许一个 framework、Resource/Search/Settings/Marketplace 等同理。差异只作为 typed section/entry/action/panel/overlay。Call/Story/media editor/payment 等若没有现成 surface，可设计最小 capability surface，但仍由统一 shell/router 管理并复用 canonical truth。

同时必须交付 **UI capability-placement matrix**。每个用户可见 capability 记录：`ui_entry_class`、primary nav owner、collection/filter、creation flow、object action、detail section、capability surface、route、keyboard/focus/accessibility、responsive behavior、state continuity。没有这些字段的 UI responsibility 不能进入 mapped/implemented。

消息领域必须明确一个 `ConversationCreationSurface`：Human private / Group / Channel / Agent / Hybrid / future Conversation kind 以 typed variant 实现；Group/Channel 不新增独立一级 App。Contacts 的 add/invite 与 Messages 的 conversation creation 分开，联系人发消息只 find-or-create canonical Direct Conversation。

搜索必须明确一个 Search owner 和三层 UX：入口内搜索、当前对象搜索、Universal Search（Cmd/Ctrl+K）。Participant/contact/member/admin picker 复用同一 Participant Search + eligibility policy。P0 必须列 local/remote data source、typed result/provider、权限/隐私、ranking/dedupe/cursor/cancel/debounce/stale-result fencing 与 result navigation。

P0 还必须完成 exact-head UI foundation audit：当前 runtime theme tokens、Sand/shared primitives、overlay/focus behavior、command palette、conversation list/workspace、composer、menus/dialogs、settings/plugins surfaces。基于实际 owner 建立 Fabushi semantic token/component adapter 计划，禁止凭空重做设计系统。

必须输出 design-system migration matrix：现有 token/primitive -> Fabushi semantic token/component；哪些 legacy `sand-*`/`cursor-*` 留在 compatibility/evidence adapter；哪些 feature-local magic values 必须收口；每个核心 screen 对应的 canonical pattern 与 protected slots。

必须建立 canonical screen/state inventory：Messages、Contacts、Conversation(Human/Agent/Group/Channel/Hybrid)、Profile、Creation、Search、Marketplace、Settings、Tasks/Automations、Computer、Call、Story/Media、Mini App，逐一列 loading/empty/error/offline/permission/data-heavy/light/dark/locale/responsive/a11y/motion states。

## G. First shipping vertical slice

existing Fabushi shell + 左竖栏 -> Human identity/contact -> 同一 conversation/composer -> Fabushi native durable send/receive -> canonical transcript -> 显式 Agent action/permission -> Coordinator/Host/Runner -> 同一 transcript/artifact -> restart/reconnect。

同时明确消息/联系人导航与插件市场迁移的可执行验收。第一 slice 通过不等于全部模块完成，后续必须继续全部长尾功能。

## H. Ledger and validator update

按 TDRP-001 §2/§8 扩展现有 schema/validator，建立 file/responsibility/module/owner/target/evidence 双向 gate，不能新造平行账本或批量标 verified。schema/代码/测试修改是下一实现任务，须在 GitHub Actions 执行；本文不声称 gate 已实现。

validator 还必须建立 composition graph gate：检查 canonical roots、routes、state owners、capability registrations 与 production entrypoints，发现按 participant/conversation/source 类型复制完整 workspace/list/profile/composer/resource/settings root 时 fail；对 novel capability 强制 owner-absence evidence + ADR + typed integration。

另建立 UI IA/Search gate：所有可见 capability 必须有合法 entry class/canonical route；Conversation creation 不得分叉为 Group/Channel/Bot 独立 root；Search consumer 必须连接 canonical Search owner/provider registry，禁止页面私建第二索引/权限真相。对应可执行检查与 UI acceptance 仅在 GitHub Actions。

另建立 Design System / Visual gate：检查新迁移 UI 是否只消费 Fabushi semantic tokens/canonical components，是否匹配 canonical screen pattern；固定 viewport、light/dark、zh-CN/en、长文本/RTL、keyboard/focus、reduced-motion、large-data states 生成 visual artifacts 并比对 approved baselines。新增 token/primitive/pattern 必须有 design exception review。

另建立 Quality/Test Governance gate：扩展 ledger/schema 记录 quality_risk、requirement/oracle/invariant/test IDs、negative/property/temporal/fault/UI/visual/a11y/performance/security/regression/exploratory evidence。P0 必须生成 RTM，并确保每个 planned capability 在实现前已有 test basis/oracle。

优先建立 Conversation Turn / Transcript Ordering / Reconciliation 三个 oracle，因为它们覆盖此前最容易被 eventual-state E2E 漏掉的 final disappearing、intermediate replacement、message reorder、late baseline 与 optimistic/authoritative merge 类问题。

packaged acceptance 必须扩展为时间序列：不能只等待 marker 出现；需要记录整个 logical turn 的所有 assistant/tool entries，在 terminal 后经过 quiet window、switch-away/back、reconnect/reload/restart，再比较 canonical semantic result。完整 session video 必须由独立验收者实际审阅。

## Exit criteria

P0 只在以下均完成时通过：

- upstream/target exact baseline、一致 lock/inventory 与递归/外部闭合可核验；
- 全部文件和 non-UI responsibilities 有明确对照，无未知范围；
- 全模块行为合同、current owner resolution 和必要最小 ADR 完整；
- 语言、服务端依赖、接口/数据迁移、UI/品牌与发布风险明晰；
- first slice 和后续全部模块均有具体实现/测试路径；
- 新 ledger/validator gate 取得 exact-head GitHub Actions 证据。

P0 通过只是实施输入闭合，不是产品已完成。某项 blocked 时继续无依赖阻塞的研究、合同、已批准实现或证据收集；不能伪造 P0/既有 Bot gate 已通过来启动依赖它的工作。需要用户支持先检查相同通知及回复，按指定渠道通知一次，再推进其他内容。

Current live authority (2026-10-09): `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (root tree `5030985204963cbbd362ced7412d231b04ebd0cc`), three commits ahead of historical `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`; 15 root paths changed (12 modified, 3 added), recursive denominator is 16,123, read-through is 5,613, unread is 10,510, unknown is 15,844, omitted is 0, and source closure remains open.


## Live deterministic source batch 5600-5604

Local Storage and Settings Main are exact-source read-complete and mapped-open. Required implementation owners are the existing canonical Settings/Storage/Cache, Identity/Account, Privacy/Security, Wallet/Payments, Business and platform adapters. The next deterministic source is 5605 `settings_notifications.cpp`. Reading these five files does not close unknown; production, service/platform, UI/UX and exact-head evidence remain required.


## Live deterministic source batch 5605-5613

Fail-closed Revision 9 validation rejected the transient incorrect 5608 path and forced exact-order repair. Corrected source responsibilities are Notifications 5605–5607, Reactions 5608–5609, per-type notification policy 5610–5611 and Account Passkeys 5612–5613. Priority production closure remains native notification reply/mark-read/open exact-scope authorization/cleanup plus canonical Account Passkeys list/create/delete and signed packaged WebAuthn acceptance. Existing remote WebAuthn proxy is infrastructure, not Account Passkeys product completion. Next source: 5614 `settings_premium.cpp`.

## Live deterministic source batch 5614–5616

Premium entitlement/subscription/commerce is exact-source read-complete and mapped-open. Existing canonical Entitlement/Subscription, Wallet/Payments/Commerce, Settings/Identity and design-system owners must absorb it. Purchase/ref routing requires invalid-route fail-closed and idempotent cancel/failure/settlement/account-switch/reload/restart reconciliation. Accounting: 5,616/16,123 read, 10,507 unread, 15,844 unknown, 0 omitted. Next: 5617 `settings_privacy_security.cpp`.

### Read-through 5617–5622

Privacy/Security composition, editable keyboard shortcuts, and server-backed Website/Bot authorization sessions are exact-blob read-complete and mapped-open to existing canonical Account Security/Privacy, Authorization, Command Registry, Bot/Blocked-Peer, Payments/Retention and Settings owners. Exact authorization hash, account/session fencing, destructive confirmation/retry/idempotency and keyboard/a11y behavior remain production gates. Accounting: read-through 5,622; unread 10,501; unknown 15,844; omitted 0. First unread: 5623 `settings/settings_builder.cpp`.
