# Fabushi Bot 全量通信能力等价重写与统一产品 Spec

Status: active  
Spec ID: FBCP-001  
Revision: 7  
Last updated: 2026-10-07  
Owner: Fabushi Desktop  
Canonical project: `projects/fabushi-communication-platform`  
Companion implementation contract: `docs/specs/telegram-desktop-rust-equivalence-migration.md` (TDRP-001 Revision 9)  
Implementation status: **requirements updated; full migration not accepted**

Live execution authority (2026-10-10): source authority remains `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941`; deterministic read-through is 7,154/16,125, unread=8,971, unknown=15,845, omitted=0. Predecessor exact-head 4028b761 runs 38062398837/38062398841 completed/success, but this descendant source-accounting update requires fresh exact-head Actions. Full migration is not accepted.

> **最终产品是一个完整的 Fabushi Bot：在现有 Fabushi 架构内，逐文件、逐模块理解 `telegramdesktop/tdesktop`，把其全部非 UI 代码职责用最合适的语言等价重写；原 UI 表现层由统一 Fabushi UI 替代，但 UI 中承载的功能与业务逻辑不得遗漏。最终同时具备现有 Bot 的全部能力和 Telegram Desktop 源码所体现的全部产品能力，而不是绑定 Telegram、添加入口、做 Provider 接入、选择性借鉴或局部 Demo。**

本 Revision 落实 2026-10-07 用户要求。它替代 Revision 2 中“研究即可结束”“决定 Fabushi 是否需要再实现”“where accepted”以及把 blocked 当作最终功能验收结果的口径。旧研究、ledger、ADR 和历史证据保留供追溯，但必须按本 Revision 重新核对；不得自动升级为 implemented/verified。

## 1. 目标、范围与非目标

**FBCP-FR-01 — 完整功能并集。** 最终能力集合为：现有 Fabushi Bot 全部能力，加上完整 tdesktop 源码及其依赖所体现的全部通信、内容、媒体、安全、商业及桌面生命周期能力。任务不是机械复制目录，而是逐文件负责、逐模块理解、逐责任等价实现、逐功能取证。功能清单只是下界，源码中新发现的能力自动进入范围。

**FBCP-FR-02 — 完整源文件对照。** 全量文件清单与逐模块行为合同必须相互可追溯。每个源文件都要回答“做什么、状态归谁、迁到哪个 Fabushi owner/目标文件、用什么语言、如何等价验证”。不能用几个顶层模块、目录级勾选、文件数量或空壳文件代替完整对照。具体合同见 TDRP-001。

**FBCP-FR-03 — UI 例外只针对表现实现。** 不迁移 Qt widget/绘制/布局/皮肤的原实现；所有交互能力、校验、权限、消息行为、生命周期、可访问性及混在 UI 文件里的非 UI 逻辑仍在迁移范围。不得整目录排除 `ui`、`boxes`、`history`、`info`、`settings`、`window` 等区域。

**FBCP-FR-04 — 一个 Fabushi。** 不保留 Telegram/Grok Bot/Gok Bot 的产品标识，不建立第二套应用、侧边栏、工作区、消息库、账号体系或运行时。用户只面对 Fabushi 的统一产品。

非目标：官方 Telegram 网络接入、账号绑定、MTProto wire compatibility、运行 Telegram Desktop/TDLib/C++ core 后包一层 UI、嵌入 Telegram Web、复制微信品牌或素材、借此删减现有 Bot 功能。未来互联项目需要独立明确授权，不能替代本迁移。

## 2. 当前事实与版本权威

2026-10-07 本次规范修订读取到：

- Fabushi `main`: `860f03a8553c779fe006c7826f190c3014a571dc`。
- PR #20 已于 2026-10-04 合并；PR head 为 `798cf51d96cb1cb98cf657af212bb47274ccb701`，merge commit 为 `ee66bacdf47f36af2ae96c0a8a8ec401460426e7`。这记录合并事实，不替代原 Bot 的验收证据。
- tdesktop `dev` discovery HEAD: `f23c37857220eb84f8559f0901ea26fb304b564b`。
- 旧迁移文档基线：`33261535a0e747f125e0ed25486f01e556330677`。旧清单与证据不能证明上述新 discovery HEAD 已被覆盖。

上述 SHA 是本次读取快照，不是永远的 latest。每轮执行重新读取 live refs；目标是合并 PR #20 后的当前 canonical Fabushi，而不是回退到旧 PR 分支。TDRP 必须在 GitHub Actions 内把最新确认的上游 exact SHA、完整树和递归依赖锁成一致 baseline，记录与旧 baseline 的新增/删除/修改差异。锁文件、inventory、ledger、行为合同一致前，`baseline_ready=false`。本次仅修订 spec，不宣称已完成 rebaseline。

后续 upstream 更新必须显式 rebaseline，受影响证据失效并重新取得；不得用浮动 `dev` 验收，也不得悄悄继承旧 verified 状态。既有 Bot 完整迁移/合并/验收硬门不能因本规范被绕过。

## 3. 单一架构与 Existing-owner-first

PR #20 演化而来的 canonical Fabushi 是唯一产品架构。优先检查并扩展现有：product shell、navigation、conversation workspace、transcript/cards、composer/drafts、Agent、Shared Room/member、permissions、attachments/artifacts、search、settings、Coordinator、Host、Runner、Plugins/MCP、Computer、Automations、platform/lifecycle、durable state/recovery。

每个责任必须走：

```text
upstream exact file/symbol + module behavior
  -> inspect current Fabushi exact-head owners
  -> select existing_owner
  -> define typed contract + state/lifecycle changes
  -> implement in Fabushi-owned target files
  -> wire shipping composition
  -> equivalent behavior + regression + packaged evidence
```

**ABSORB-01:** 先解析 `existing_owner`，不能因上游有一个模块就照搬一个新架构层。  
**ABSORB-02:** 扩展已有 owner，不复制第二份 Identity/Conversation/Message/Resource/Permission 真相。  
**ABSORB-03:** 只有所有合理 existing owners 都不适合时，记录 rejected owners、原因、唯一状态、生命周期、依赖方向、语言、typed interfaces、测试、迁移方案，并以批准的 ADR 新增最小 owner。  
**ABSORB-04:** 必需的新网络/媒体/同步基础设施支持现有产品 owner，不得升级为平行 CommunicationCore、TelegramRuntime、TelegramProvider 或 MessengerSubsystem。

### 3.1 Unified Product Workspace / Single-Composition Law

本规则适用于**全部**从 Telegram Desktop 吸收的产品能力，不只适用于会话窗口。所谓“统一 workspace”是 Fabushi 唯一 product shell 及其 canonical workspace composition；不同能力可以拥有 typed component、section、panel、card、overlay、viewer 或专门的最小 domain owner，但不得为 Human/Agent/群/频道/某来源能力分别复制一套完整产品表面和状态链。

**FBCP-UW-01 — 一个产品概念只能有一个 canonical root composition。**  
以下核心概念必须各自只有一个 canonical implementation：product shell/navigation、ConversationWorkspace、conversation list、Transcript、Composer、Participant/Profile surface、Resource viewer/editor、Search surface、Settings workspace、Notification surface、Marketplace、Task/Automation surface。不得用联系人种类、conversation kind、来源模块或历史产品名再复制完整 root。

明确禁止把差异实现为完整平行窗口，例如：`BotChatWindow + HumanChatWindow`、`AgentConversationWorkspace + HumanConversationWorkspace`、`GroupChatApp + ChannelChatApp`、`AgentProfileRoot + HumanProfileRoot`、`TelegramMediaViewer + FabushiMediaViewer`。即使这些平行实现暂时共用同一个 store，也属于违反本规则，因为它们会形成重复的 navigation/header/history/composer/draft/selection/lifecycle ownership。

**FBCP-UW-02 — 差异只能通过 typed capability composition 表达。**  
Human、Agent、Group、Channel、Hybrid、Topic 等差异由稳定 typed axes 表达，例如：

```text
conversation.kind
participant.kind
entry.kind
resource.kind
capabilities
policy
permissions
runtime_state
```

统一 workspace 通过明确的 capability/renderer registry 或等价 typed composition 装配差异。允许的差异单位包括 `HumanProfileSection`、`AgentRuntimeSection`、`AgentThinkingEntry`、`PollEntry`、`CallControls`、`StoryCard`、`PaymentReceiptCard` 等；它们不得重新拥有整个 workspace、transcript、composer、identity、resource、permission 或 navigation。

**FBCP-UW-03 — 共享骨架与状态只能有一份。**  
以下能力不能因为类型不同而分叉成第二套实现：header shell、history/transcript container、scroll/pagination、selection、draft、reply/quote、attachment picker/resource lifecycle、read/unread、send settlement、navigation lifecycle、account scoping、permissions projection、loading/error/empty states、accessibility、theme/i18n。类型化组件只能向这些共享 owner 提供数据、行为和扩展 slot。

**FBCP-UW-04 — Profile/联系人同样遵守单一 composition。**  
Human、Agent、Group、Channel 使用统一 Participant/Profile framework 与同一 identity/member truth。Human 可以提供 presence、privacy、phone/username、shared groups 等 section；Agent 可以提供 runtime、model、tools、permissions、tasks 等 section；Group/Channel 可以提供 members、roles、media、moderation 等 section。允许 section 不同，不允许形成互不兼容的第二套 identity/profile 基础设施。

**FBCP-UW-05 — Telegram 中当前 Fabushi 完全没有的能力必须新增“最小能力 owner”，而不是新应用。**  
如果完整 exact-head owner audit 证明某项职责在 Fabushi 中确实没有合理 owner，则该能力仍然在范围内，不能删除、降级或强塞进错误 owner。必须：
1. 记录所有 rejected existing owners 及理由；
2. 定义不可再缩小的 domain responsibility、唯一 state/lifecycle owner、commands/events、persistence、security、service dependencies；
3. 通过 ADR 批准一个 source-neutral 的最小新 owner；
4. 以 typed capability contract 接入统一 shell/workspace/identity/resource/permission/navigation；
5. 复用现有 canonical truth，不重新创建 Conversation/Message/Participant/Resource 等总模型；
6. 取得 production + packaged evidence 后才能 verified。

例如当前架构若没有完整的 CallSession、Story lifecycle、PaymentSettlement、Presence/Sync 等 owner，可以新增相应最小 domain owner；但不得因此出现 CallApp、StoryApp、PaymentsApp 或 TelegramWorkspace 作为第二套产品骨架。某能力确实需要沉浸式界面（例如通话、媒体编辑、Story viewer）时，可以是统一 shell 管理的 capability surface/full-screen overlay，它仍共享 canonical identity/resource/permissions/lifecycle，并由统一 router/composition 打开和销毁。

**FBCP-UW-06 — 新能力不能污染既有 owner。**  
“必须统一”不等于把所有 Telegram 功能塞进 ConversationWorkspace 或 Coordinator。领域状态仍放在最合适 owner；统一的是产品 composition、canonical truth 和 typed contracts。Coordinator/Host/Runner 只负责 Agent 执行职责；普通通信、Call、Story、Payment 等不能为了复用而错误进入 Agent runtime。

**FBCP-UW-07 — 架构 gate 必须检查 composition duplication。**  
实现阶段必须建立可执行 architecture/composition gate：枚举 canonical roots、capability registration、state owner 和 production entrypoints；发现按 Human/Agent/Group/Channel/Telegram 来源复制完整 workspace/list/profile/composer/resource/settings 等 root 时 fail closed。不能仅靠文件名扫描，必须结合 import/composition/state-owner graph 与 shipping route/entrypoint 证明没有平行实现。

下面只指定职责方向，实际 target path/symbol 必须读当前代码后写入 ledger：

| 源码职责 | Fabushi 吸收方向 |
| --- | --- |
| dialogs/history/messages/composer | navigation/list、conversation、transcript、composer/draft 的现有 owner |
| contacts/members/groups/channels/topics | 现有 identity、Shared Room/member、权限与 conversation/thread 模型 |
| reply/quote/forward/edit/delete/reactions/polls | typed transcript、消息关系、provenance、reaction/card owner |
| files/media/cache/transfer/export | attachments/artifacts/resource、存储与 data lifecycle |
| bots/inline/commands/Mini Apps | Agent/typed interaction、Plugins/MCP、既有安全 Web surface |
| scheduled send/business/tasks | Automations、Task、Composer、Agent 与权限 |
| calls/screen sharing | 现有 Computer/媒体/平台 owner，缺失的 call-session/signaling 用最小 ADR 补齐 |
| auth/privacy/notifications/settings | 现有 account/secrets/permissions/settings/platform |
| protocol/sync/session/recovery | 现有 durable state 与 Fabushi 自有网络基础设施 |

## 4. 全功能范围：不得选择性实现

以下领域均必须逐项展开子功能、文件引用、状态机、错误路径、目标 owner 与验收证据，不是任选清单：

- 账号、认证、会话、多账号、多设备、设备撤销、联系人、用户名、身份、在线/输入状态、屏蔽与隐私。
- 会话列表、文件夹、归档、置顶、收藏/个人空间、私聊、群/超级群、频道、广播、社区、话题/论坛、线程、成员角色与管理。
- 完整消息类型、富文本、回复、引用、转发来源、编辑、删除、反应、投票、草稿、定时/静默发送、已读/未读、搜索、分页、消息过期/自销毁、错误恢复。
- 图片、文件、语音、视频、录制、播放、流媒体、上传/下载/续传、缓存、媒体编辑、表情、贴纸、GIF、自定义 emoji、Stories、富文档/Instant View 等内容能力。
- 音视频通话、群通话、屏幕共享、设备选择、权限、信令、断线重连和后台/休眠生命周期。
- Bot 命令、inline/callback/交互消息、Mini Apps、业务工作流、AI 辅助编写、任务/todo，以及现有 Fabushi Agent/Host/Coordinator/Runner、Computer、Plugins/MCP、Automations、任务/子 Agent、审批和产物。
- 订阅/权益、积分/价值单位、礼物、支付、退款/结算、商业功能、身份文档与安全验证等职责；用 Fabushi 品牌和自有服务重实现，不直接把上游商标当作产品名。
- 通知、托盘、角标、铃声、设置、本地锁、凭据与端到端安全职责、举报/审核、敏感内容策略、统计、导出、存储迁移、损坏恢复、安装/升级/回滚、主题、语言/RTL、IME、可访问性及性能资源管理。

源码中发现但上述列表未列出的模块仍必须迁移。开发者不得以“Bot 暂时用不到”“太复杂”“需要服务端”“是 UI 目录”“不是第一阶段”为理由把功能标成已排除。

## 5. 语言与技术选型

**FBCP-LANG-01:** tdesktop/desktop-app 的 C++ 非 UI 产品逻辑，默认用 Rust 在 Fabushi 自有模块中重新实现，包括状态机、存储、同步、传输、任务、生命周期、媒体控制和安全边界。不能把原 C++ 库经 FFI、subprocess 或 wrapper 继续作为真实业务 owner。

**FBCP-LANG-02:** 统一桌面 UI 使用现有 React/TypeScript 与 Electron 边界；TypeScript 负责视图、交互投影与必要的平台桥接，不建立 renderer-side 业务真相。Swift/Objective-C、系统 API、其他语言只用于确有平台要求的最薄适配层，记录理由和边界。

**FBCP-LANG-03:** 非 C++ 文件按职责使用最合适的语言/格式；构建、生成器、schema、资源与打包脚本也要逐项映射到 Fabushi 自有生成/发布链。所有语言偏离默认方案必须有性能、安全、维护成本与平台约束依据，不以迁移省事为理由。

**FBCP-LANG-04:** 通用第三方密码学、编解码或 OS 能力可选用经审查的成熟依赖，但必须全量盘点其依赖/许可证、说明替代关系并验证集成。它们不是保留 Telegram C++ 业务实现的例外；禁止为了“全 Rust”自行发明密码算法或绕过安全审查。

## 6. 模型、控制流与统一状态

扩展既有 Conversation、Participant/Member（Human/Agent）、Message/Transcript、Resource、Permission、Task 等类型。Human、Agent、群、频道、混合房间与话题使用同一产品模型；不另造 Telegram-prefixed 产品类型。

Message 必须保留 stable identity、作者、内容类型、reply/forward provenance、revision、顺序、发送/确认状态与 resource 关系。Agent thinking/tool/task/approval/artifact 继续是 typed entries，不能压成普通文本。Resource 复用附件、Agent 产物、Computer 产物和通信媒体的唯一生命周期及 provenance。

典型消息链：existing composer -> account-scoped typed command -> existing permission/domain owner -> durable outbox/native service -> canonical event/ack -> existing transcript projection。重试、去重、顺序、幂等、gap recovery、取消、撤回、崩溃后恢复都必须有唯一 owner。

Human 消息显式触发 Agent 时：同一 transcript -> 明确的 @Agent/Ask Agent/interaction intent -> 权限边界 -> Coordinator -> Host -> Runner -> typed Agent output/task/artifact -> 同一 transcript。普通消息、第三方内容或 inline interaction 不得隐式获得执行权限。Bot commands/callback 与 LLM conversation 不可仅因都叫 Bot 就混淆语义。

左侧摘要、未读数、右侧消息正文与 Agent 运行状态来自同一 canonical state。不得为 UI 路由切换建立第二套 session/transcript，或因 streaming/settlement 重排、吞掉、替换已确认消息。

## 7. Fabushi 自有通信基础设施与服务端完整性

Fabushi 自己拥有身份、会话、消息、同步、presence、blob/media、call signaling、push 与必要的商业服务。协议为 Human + Agent 设计，支持消息变更、成员/角色、任务、产物、审批、自动化、通话和 Computer handoff 等 typed events，不要求 Telegram wire format。

上游协议代码的每个非 UI 职责仍需逐文件分析和等价重写：ordering、duplicate suppression、offline outbox、reconnect、session failure、clock handling、gap recovery、多设备同步、传输恢复、密钥生命周期等不能因“不用 MTProto”被跳过。

上游是客户端源码来源；客户端调用的远端能力不等于其服务端实现也已开源提供。对每项网络依赖必须明确 client responsibility、remote contract、Fabushi server owner、认证/权限、存储、部署、监控及真实端到端测试。支付、通话中继、推送、同步等所需外部配置未就绪时标 blocked，不能用 mock、空实现或仅本地 UI 证明功能完成。

“不依赖 Telegram 网络”不是删掉依赖服务端的功能，而是补齐 Fabushi 的等价服务。第三方内容库、上游账号/资产/商业权益不会因重写自动转移；必须说明 Fabushi 对应目录、权益与结算机制，阻塞未消除前不得完整验收。

## 8. UI：微信式结构，Fabushi 自有设计

2026-10-07 用户提供的微信桌面截图只作为信息结构参考，不复制截图中的个人数据、商标、图标或视觉素材，也不将原截图提交到公共仓库。

**FBCP-UI-01 — 左侧竖栏是主入口。** 主功能采用固定的窄竖向导航栏，支持图标、选中态、tooltip/可访问名称、未读/状态提示和键盘导航。消息与联系人是一级入口。插件市场必须从原位置迁入左侧竖栏，作为可直接打开的一级入口，而不是只藏在设置、右侧面板或 Bot 会话里。

**FBCP-UI-02 — 三段主体。** 最左为主导航；中左为当前入口的列表/搜索/创建区；右侧为主工作区；成员/媒体/权限/任务详情是按需展开的辅助面板，不是第二个应用。

```text
┌──────────┬──────────────────┬─────────────────────────────────┐
│ Fabushi  │ 搜索 / 新建      │ 当前会话 / 联系人 / 功能标题     │
│ 消息     │                  │                                 │
│ 联系人   │ 会话或联系人列表 │ 消息、Agent 输出、任务与产物     │
│ Agents   │                  │                                 │
│ 任务     │                  │                                 │
│ Computer │                  │                                 │
│ 自动化   │                  │                                 │
│ 插件市场 │                  │ 编辑器 / 附件 / 发送 / Agent操作 │
│ 账号设置 │                  │                                 │
└──────────┴──────────────────┴─────────────────────────────────┘
```

除消息、联系人、插件市场这三个固定要求外，其余已有主功能按实际产品可用能力组织；不能因为示意图未列出某项就删除功能。不得新增名为 Telegram 或 Grok Bot 的入口。

**FBCP-UI-03 — 消息入口。** Human、Agent、群、频道与混合房间共用列表和 workspace；搜索、新建、筛选、置顶、归档、未读与上下文操作属于该入口。会话右侧包含标题/动作、历史和底部 composer。

**FBCP-UI-04 — 联系人入口。** 使用现有 identity/member 模型提供联系人搜索、添加/邀请、详情、发起会话与相关分组/管理；不新建与会话身份不一致的本地通讯录真相。

**FBCP-UI-05 — 插件市场迁移。** 竖栏打开现有 marketplace 的演化页面，保留搜索、分类、详情、安装/卸载、更新、启停、授权和 Plugins/MCP 管理的全部现有能力。旧路由可暂时重定向以兼容链接，但不得产生第二套 marketplace 状态/owner。

**FBCP-UI-06 — 状态连续性。** 切入口/切会话保留草稿、滚动位置、选中对象、未读和运行中任务；支持窗口缩放、最小尺寸、键盘、屏幕阅读器、深浅色、多语言和中文 IME。小窗口可折叠列表，不能把核心入口变得不可达。

**FBCP-UI-07 — 会话 UI 只有一个 canonical composition。** Human、Agent、Group、Channel、Hybrid conversation 必须由同一个 `ConversationWorkspace` composition 构建；不得分别维护完整 Bot/Human/Group/Channel chat window。差异只能通过 typed header/profile section、transcript entry renderer、composer action、capability panel/overlay 和 policy 表达。

**FBCP-UI-08 — 其他产品表面同样只允许 typed differences。** 联系人/Agent/群资料、媒体查看/编辑、搜索、设置、通知、通话、Stories、商业/支付、Mini Apps 等都必须优先接入统一 product shell 和对应 canonical surface。只有当前架构完全缺失且 existing-owner audit 失败时，才可按 FBCP-UW-05 新增最小 capability owner/surface；该 surface 仍由统一 shell/router 管理，不得形成独立应用。

### 8.1 UI Information Architecture / Interaction Contract

本节定义 Telegram 全量能力迁入 Fabushi 后如何决定功能入口、创建路径、搜索范围和专用 surface。目标不是照搬 Telegram 菜单层级，也不是把每个新能力都堆到左栏；目标是让全部能力在一个可学习、可发现、可扩展的 Fabushi 信息架构中工作。详细 companion contract：`projects/fabushi-communication-platform/ui-information-architecture.md`。

**FBCP-UI-09 — 入口按用户任务与作用域分层，不按源码来源分层。** 每个用户可见 capability 必须有且只有一个主要入口类别：`primary-domain`、`collection-action`、`creation-action`、`object-action`、`detail-section`、`capability-surface`、`global-command` 或 `system-surface`。上游存在独立菜单/窗口/模块，不构成新增一级入口的理由。

**FBCP-UI-10 — 一级导航保持稳定且低噪声。** 左侧窄竖栏承载稳定、跨对象、长期独立浏览的产品领域。消息、联系人、插件市场是明确一级入口；既有 Agents、任务、Computer、Automations、账号/设置等能力继续可达。Group、Channel、Topic、Poll、Call、Story、Media、Forward 等不得仅因来源独立而自动升级为一级 App。

**FBCP-UI-11 — Conversation kind 是统一消息领域中的类型，不是独立应用。** Direct Human、Agent、Group、Channel、Hybrid、Topic/Thread 共享同一 Conversation list/workspace。中间列表可提供 `全部 / 未读 / 私聊 / 群组 / 频道 / Agent` 等 filter/view，但 filter 不拥有第二份 list/store/workspace。

**FBCP-UI-12 — 统一创建入口。** 消息入口中左列表顶部的统一 `New / +` action 打开唯一 `ConversationCreationSurface` 或等价 typed creation flow，至少承接 Human 私聊、新建群组、新建频道、新建/选择 Agent 会话、Human + Agent 混合房间以及后续发现的 Conversation-like kind。它们是 `CreateConversationIntent { kind, participants, capabilities, policy }` 的 typed variants；Group/Channel 专有字段以 typed sections 出现，成员选择复用统一 Participant search/eligibility/permission。

**FBCP-UI-13 — Contacts 创建与 Conversation 创建分工。** 联系人入口的 `+` 用于添加/邀请/管理 Participant identity/contact relation；消息入口的 `+` 用于创建 Conversation。联系人详情中的“发消息”必须 find-or-create 同一 Direct Conversation 后进入同一个 ConversationWorkspace；不得打开 ContactChatWindow。

**FBCP-UI-14 — Search 只有一个 canonical domain，UI 有三级作用域。** 入口内搜索用于当前一级领域；对象内搜索用于当前 Conversation/Resource/Task 等对象；Universal Search 通过 `Cmd/Ctrl+K` 与统一可达入口跨整个 Fabushi 搜索。三者只是 scope 不同，不是不同搜索系统。

**FBCP-UI-15 — Search contract 必须统一。** 所有搜索入口最终进入同一 Search owner 的 typed query/result/provider 合同。结果至少可包含 Participant、Conversation、Message、Resource、Agent、Task、Artifact、Plugin、Setting/Command。联系人搜索、群成员/频道管理员 picker、新建会话成员搜索也复用 Participant Search + eligibility policy；模块只能贡献 SearchProvider/filter/result renderer，不能创建第二 Search root、第二索引真相或第二权限判断。

**FBCP-UI-16 — Search 安全与本地/远端组合。** 查询执行前应用 account、membership、privacy、block、retention、resource/task/artifact visibility 等权限，禁止先泄露结果再由 UI 隐藏。local index 与 Fabushi remote historical search 可以并存，但必须位于同一 Search owner 后，统一 ranking、dedupe、cursor/pagination、debounce、cancellation、stale-query fencing 和 result provenance。

**FBCP-UI-17 — 对象动作靠近对象，避免全局入口泛滥。** Reply/Forward/Edit/Delete/Reaction/Pin、Call、查看资料、成员管理、频道权限、媒体/文件操作等优先位于当前 Message/Conversation/Profile/Resource 的 header、context menu、toolbar 或 detail section。只有跨对象且持续独立的领域才成为一级入口。

**FBCP-UI-18 — 专用 surface 必须可返回且不丢上下文。** Call、Story viewer、media editor、payment/settlement、Mini App 等需要沉浸式 UI 时，允许 full-screen/panel/modal/overlay，但必须由统一 router/shell 打开；关闭后恢复原 selection、draft、scroll、search query、running task/call 等上下文，且不建立第二账号、设置、消息列表或资源真相。

**FBCP-UI-19 — 导航、快捷键、可访问与响应式是一套合同。** 一级导航、collection filter、object action、global command 必须有一致 keyboard/focus/accessibility semantics；`Cmd/Ctrl+K` 保留给 Universal Search。窄窗口可折叠中间列表或详情面板，但核心入口、返回路径与活动任务/通话状态必须仍可达。

**FBCP-UI-20 — UI 迁移不是像素复刻。** 微信截图和 Telegram UI 只提供信息架构/行为参考；最终视觉由 Fabushi 自有设计系统决定。验收关注入口合理性、能力可发现性、状态连续、认知负担、键盘/屏幕阅读器可用和完整功能，不逐像素复制外部产品。

### 8.2 Fabushi Design System / Canonical Components / Screen Patterns

以下文档是本 Spec 的 normative UI companion，任何 AI、开发者或迁移模块都必须遵守；不能只看功能需求后自由发挥视觉样式：

- `projects/fabushi-communication-platform/design-system.md` — Fabushi visual language、semantic tokens、theme、typography、spacing、color、icon/avatar、motion、content style。
- `projects/fabushi-communication-platform/ui-component-contract.md` — canonical UI primitives/patterns、状态、可访问性与扩展规则。
- `projects/fabushi-communication-platform/canonical-screen-patterns.md` — 核心 screen/workspace 的结构与 slot。
- `projects/fabushi-communication-platform/visual-acceptance.md` — screenshot/interaction/a11y/responsive/locale/visual-regression 验收合同。

**FBCP-DS-01 — AI 不得自由发明第二视觉语言。** 新能力首先选择既有 design token、canonical component、screen pattern 与 typed slot；只有已有系统无法表达真实产品责任时，才允许扩展 design system，并记录理由、复用范围、兼容迁移和 visual acceptance。

**FBCP-DS-02 — Semantic token only.** 业务/功能 UI 不直接使用任意 hex、rgba、font-size、radius、shadow、z-index、animation duration 或来源品牌 token。颜色、排版、间距、尺寸、圆角、边框、阴影、motion、layering 必须来自 Fabushi semantic token contract。数据可视化、媒体内容、第三方嵌入和明确 legacy adapter 例外必须被精确登记。

**FBCP-DS-03 — Fabushi namespace 是目标公共合同。** 当前 exact-head 存在历史 `sand-*` / `cursor-*` token 与 recovered primitives；它们可在兼容层内保持以避免破坏既有 Bot，但 Telegram-derived 新 UI 不得把这些历史来源命名当作新的产品 API。目标是 `fabushi-*` semantic aliases/components；最终产品表面、DOM/a11y 文案、用户可见 asset 不暴露来源品牌。

**FBCP-DS-04 — Canonical component first.** Button、IconButton、SearchField、Input、Menu、Dialog、Tooltip、Tabs、Avatar、Badge、ListRow、ConversationRow、ParticipantRow、ResultRow、Empty/Loading/Error、Composer、ProfileSection、DetailPanel、Toast/Banner、Picker、Creation flow 等必须有一个 canonical component/pattern owner。功能模块不能复制一套局部 button/list/menu/dialog 体系。

**FBCP-DS-05 — Canonical screens 是结构权威，不是截图。** Messages、Contacts、Conversation、Profile、Creation、Search、Marketplace、Settings、Tasks/Automations、Computer、Call、Story/Media、Mini App 等核心 surface 具有固定结构、slot、action hierarchy 和 state matrix。新 capability 加到既有 slot；确需新 screen pattern 时先扩展 screen contract。

**FBCP-DS-06 — Density 与层级统一。** 默认 desktop density 使用统一 4px rhythm、统一 control/list/header heights、统一 text hierarchy 与 progressive disclosure。高频动作显式，低频/危险动作放入 menu/detail；不能为了“功能完整”把所有动作同时铺在主界面。

**FBCP-DS-07 — Motion 只表达状态变化。** hover/selection/panel/message/task/Agent-thinking/call 等反馈使用统一 duration/easing；`prefers-reduced-motion` 下取消非必要 animation。禁止纯装饰、持续抢注意力或不同页面各自定义的动效语言。

**FBCP-DS-08 — Content design 统一。** 按钮/菜单/错误/空状态/确认/权限提示使用一致动词、时态、破坏性文案与中英文 terminology。产品概念在 UI 中只有一个名称；不得同一对象同时出现“频道/channel/broadcast room”等无规则混称。

**FBCP-DS-09 — Icons / avatars / assets 统一。** 一级导航和通用动作使用一个 canonical icon family 与尺寸体系；不得混用 emoji、Telegram/微信/Grok/Cursor 品牌图标或不同线宽图标作为通用 UI。Human/Agent/Group/Channel 复用统一 Avatar primitive，通过 typed fallback/badge/status 表达类型差异。

**FBCP-DS-10 — Visual change 必须有证据。** 任何影响核心 screen、token、component、layout 或 interaction state 的变更，都必须在 GitHub Actions 生成 current-head visual artifacts，并通过 `visual-acceptance.md` 的 structural、light/dark、locale、responsive、keyboard/a11y 和 state-continuity gates；不能用“看起来差不多”验收。

### 8.3 Quality / Test Governance

以下质量文档是本 Spec 的 normative acceptance companions：

- `projects/fabushi-communication-platform/quality/TEST_STRATEGY.md`
- `projects/fabushi-communication-platform/quality/requirements-traceability-matrix.md`
- `projects/fabushi-communication-platform/quality/evidence-contract.md`
- `projects/fabushi-communication-platform/quality/release-entry-exit-criteria.md`
- `projects/fabushi-communication-platform/quality/defect-regression-policy.md`
- `projects/fabushi-communication-platform/quality/exploratory-test-plan.md`
- `projects/fabushi-communication-platform/quality/plans/*.md`
- `projects/fabushi-communication-platform/quality/oracles/*.md`

该体系采用 ISO/IEC/IEEE 29119 系列的软件测试过程/文档思想、ISTQB 的风险驱动/可追溯/独立性原则、ISO/IEC 25010 质量属性、WCAG 2.2 AA 与 OWASP security verification 思路作为参考；这表示工程方法参考，不构成任何认证声明。

**FBCP-QA-01 — Test basis before test execution.** 每个被测试 capability 先有 stable requirement ID、行为合同、状态机/不变量和可接受差异；测试不能靠“实现看起来合理”自行定义正确性。
**FBCP-QA-02 — Requirement-to-evidence traceability.** 每个 applicable requirement/AC 必须双向追溯到 oracle/invariant、test case、current-head execution evidence 和 acceptance verdict。没有 traceability 的绿色测试不能把责任标 verified。
**FBCP-QA-03 — Risk-based depth.** 依据影响 × 概率 × 时序/并发复杂度标 P0/P1/P2/P3 风险；消息 settlement、transcript ordering/reconciliation、Agent streaming/tool/final、cross-conversation isolation、reconnect/restart、permissions/security 默认属于高风险，要求更深测试层。
**FBCP-QA-04 — Layered verification.** 高风险功能不能只靠 E2E；必须组合 unit、table-driven、property/state-machine、contract、integration、real packaged E2E、temporal/recovery、visual/a11y，以及适用的 performance/security/fault tests。mock 只能证明局部行为。
**FBCP-QA-05 — Temporal correctness is product correctness.** 实时/异步功能必须验证从用户动作到 terminal settlement 的完整时间序列，而不是只等某个 DOM/text 最终出现。turn/message/tool/upload/call/search 等适用能力都必须有 temporal oracle。
**FBCP-QA-06 — Stable settlement.** terminal/final/committed 状态在 quiet window、切换对象后返回、reconnect、reload、restart 后必须保持语义一致；不能出现 final 消失、旧 intermediate 回来、completed 回退为 streaming、重复或重排。
**FBCP-QA-07 — Canonical ordering.** UI DOM order、renderer projection、canonical persisted transcript/collection ordering 必须一致；乱序、duplicate、late snapshot、baseline/live race、optimistic/authoritative merge 均有 deterministic oracle。
**FBCP-QA-08 — Negative and recovery paths.** 每项关键功能至少覆盖权限拒绝、服务错误、timeout、cancel、duplicate、out-of-order、disconnect/reconnect、restart 和 stale data 中适用路径；只测 happy path 不得 verified。
**FBCP-QA-09 — UI functional testing is separate from visual testing.** UI 要同时证明 controls/routes/actions/state ownership 真正可用，以及 layout/style/a11y/locale/responsive 正确；截图漂亮不能代替功能测试，DOM assertion 绿色也不能代替视觉验收。
**FBCP-QA-10 — Timeline visual evidence.** 对 streaming/tool/final、upload、call、sync、reconnect 等动态场景，视觉验收必须在关键状态点和 settlement 后采样 screenshot + DOM manifest + lifecycle trace；整段 session video 必须实际审阅，不能只上传 artifact 就算验收。
**FBCP-QA-11 — Defect becomes permanent regression.** 人工验收、生产或任何后阶段发现的 defect，修复时必须新增最小可复现 regression case，并根据 root cause 补齐更低层测试；没有 regression evidence 不得关闭。
**FBCP-QA-12 — Flaky/blocked/skipped is not pass.** flaky、quarantined、skipped、not-run、blocked 均不是 passing evidence。关键 gate 发现 flaky 必须保留 owner、根因、修复期限并阻止 release，禁止 rerun-until-green 作为验收。
**FBCP-QA-13 — Independent acceptance.** 实现者/实现会话可以写测试并生成证据，但不能作为唯一 release acceptance reviewer。独立验收必须重新读取 spec/RTM、实际检查 evidence，并对 packaged user journey/video 做风险导向审阅。
**FBCP-QA-14 — Exact-source evidence.** 所有执行证据绑定 target SHA、upstream baseline、workflow/run/attempt/job、环境/fixture、test case IDs、artifact ID/digest。相关 HEAD/spec/oracle 改变使受影响旧证据降为历史。
**FBCP-QA-15 — Release is a separate decision.** code complete、tests green、artifact exists 都不是自动 release。只有 release entry/exit criteria、0 blocking defects、RTM 完整、independent acceptance 与 summary verdict 全部通过才可 ACCEPT。
**FBCP-QA-16 — Escaped defect feedback loop.** 人工后来发现但自动测试没发现的问题必须形成 gap analysis：为什么现有 oracle/case/gate 没抓到、在哪一层补 test、是否扩大同类风险扫描；不能只补当前单点。
**FBCP-QA-17 — Test data/environment are controlled.** 关键测试使用版本化 fixture/seed/locale/theme/viewport/network/fault profile，避免随机时间/头像/账号数据让测试失去可重复性；真实 protected account 另按秘密与清理合同管理。
**FBCP-QA-18 — All executable verification GitHub Actions only.** 对 FBCP/TDRP，unit/property/state-machine/contract/integration/E2E/visual/a11y/performance/soak/fault/package/acceptance 全部只在 GitHub Actions 运行。

## 9. 品牌、命名与来源记录

**FBCP-BRAND-01:** 产品名、窗口标题、导航、默认头像/图标、空状态、通知、托盘、设置、安装包、更新界面和用户文案统一使用 Fabushi。不得残留 Telegram、Grok Bot、Gok Bot 的产品品牌、Logo、默认品牌素材或独立产品入口。

**FBCP-BRAND-02:** 新业务类型/模块以 Fabushi domain responsibility 命名，不以 Telegram/Grok 来源命名。旧内部路径、协议字段、持久化 key 或测试 ID 的迁移必须有兼容计划，不能全仓替换导致数据损坏。它们不能被继续当作第二套品牌/业务 owner。

**FBCP-BRAND-03:** 去产品品牌不等于抹去来源。LICENSE、COPYING、NOTICE、第三方鸣谢、源码版权声明、provenance、上游仓库/commit/path 以及历史研究记录必须按适用义务保留。用户内容与真实外部服务/模型名称也不得通过盲目替换被伪造。

本项目是 source-informed，不是 clean-room。C++ 换 Rust 不自动消除源代码派生与许可证问题。发布前审查 GPLv3/OpenSSL exception、各依赖/素材许可证及实际派生关系，履行适用源码提供、告知和分发义务；必要时由合格法务确认。品牌扫描必须对合法来源记录设置有依据的精确例外，不得通过广泛白名单掩盖产品残留。

## 10. 迁移顺序与阻塞推进

先满足既有 Bot 架构/合并硬门，再按依赖推进：

1. P0：最新 exact upstream/target baseline、全量 inventory、逐文件 ledger、逐模块 dossier、current-owner resolution、接口/服务依赖、UI 路由和品牌替换清单。
2. 第一生产闭环：既有 shell + 左竖栏 -> Human identity/contact -> 同一 conversation/composer -> Fabushi durable send/receive -> transcript -> 显式 Agent action -> Coordinator/Host/Runner -> 同一结果/产物 -> restart/reconnect。
3. 按依赖闭合全部模块：群/频道/话题、消息长尾、媒体、通话、安全、Mini Apps/插件、商业、平台等；每条均完成 production wiring 和等价证据。
4. 全量回归、品牌清理、安装/升级/回滚、签名包、独立验收与发布。

阶段是工作顺序，不是范围缩减或完成定义。某项依赖阻塞时记录 owner、原因、解除条件与已请求支持，继续下一项无依赖阻塞的研究、UI/接口设计、已批准模块实现或取证；不能伪造前置通过。需要用户支持时按用户指定通知渠道处理，先检查是否已有相同未解决通知，避免重复催告。

## 11. 验证与证据

本项目所有 build/test/lint/generator/schema/benchmark/fuzz/package/acceptance 等可执行验证只在 **GitHub Actions** 进行；本 Revision 对 FBCP/TDRP 的要求替代旧文档中允许 htch-runtime 验证的口径。不要在用户 Mac、bhrum2 或 assistant container 跑这些任务。只读调查、文档编辑及 Git/API 操作允许；非必要不占用用户设备存储，必要临时改动提交云端后清理。

验证包括：完整 source-tree/recursive dependency 闭合、逐文件/责任覆盖、语言与 owner 约束、真实 shipping wiring、module contract/行为对照、Unit/integration/property/fuzz、顺序/重复/恢复/取消、权限和安全、服务端 E2E、既有 Bot 回归、UI/可访问性、性能/功耗/磁盘增长、签名安装包与升级回滚。

证据绑定 target SHA、upstream SHA、实际 checkout、workflow/run/attempt/job、命令/测试数量/exit code、artifact 名称/ID/digest 和内部 provenance。HEAD 或相关 baseline 改变后，旧结果只是历史，受影响项必须重新取证。绿色 workflow、文件数量、mock 或映射表本身都不证明功能等价。

## 12. 数据迁移、发布、回滚与观测

不破坏已有 Bot 会话、Agent 配置、附件/产物、审批、插件权限或任务。schema/key/identity 改动采用显式版本、备份/事务和回滚策略，验证中断与降级；canonical state 保持单写 owner，禁止无界双写。

生产观测需覆盖 send/ack、重复/丢失/gap、reconnect、outbox、媒体传输、call lifecycle、权限拒绝、后台资源与恢复，使用关联 ID 且脱敏，不泄露消息、密钥或个人数据。性能预算和测试数据规模在模块合同中以实际 baseline 明确并于实现前批准，不能把未测量写成达标。

发布沿用现有 Fabushi 平台矩阵、签名、公证/平台校验、安装、自动更新与回滚合同；至少在声明支持的平台取得 exact-source 包与安装后真实操作证据。不得用开发页面代替正式应用。

## 13. Acceptance Criteria / Definition of Done

以下全部通过才是完整迁移；blocked/partial/planned 不算通过。

| ID | 必须达到的结果 |
| --- | --- |
| AC-01 | canonical Fabushi 是唯一产品骨架，未回退原 Bot 架构 |
| AC-02 | 自有身份、消息、同步、媒体、通话和推送不以 Telegram 网络/Provider 为基础 |
| AC-03 | 每个源责任均有 current-head existing-owner resolution，且实际落地 |
| AC-04 | 无平行 CommunicationCore/Telegram runtime/第二产品 |
| AC-05 | 完整源范围内全部功能等价实现并 verified；困难或服务依赖不能作排除理由 |
| AC-06 | Human、Agent、群、频道、混合房间在同一 shell/workspace 完整工作 |
| AC-07 | Transcript、composer、Shared Room、attachments、Automations、Computer、Plugins/MCP 等原能力无回退 |
| AC-08 | 新 owner 均为最小职责且有批准 ADR、唯一状态与清晰生命周期 |
| AC-09 | 自有协议完整覆盖顺序、去重、离线、同步、恢复及 Agent-native events |
| AC-10 | C++ 非 UI 产品职责已在 Fabushi Rust/best-fit 代码中重写，无原 Telegram C++ 业务 fallback |
| AC-11 | 身份、消息、资源、权限和会话状态无第二真相，UI projections 一致 |
| AC-12 | Coordinator/Host/Runner/Computer/Plugins/Automations 及全部 Bot 基线验收仍通过 |
| AC-13 | 所有功能证据绑定当前 target/upstream exact SHA，来自 GitHub Actions |
| AC-14 | 正式 packaged Fabushi 证明通信与 Agent 工作是一个完整产品 |
| AC-15 | 全量文件与递归依赖双向可追溯；0 omitted/unmapped 文件或职责 |
| AC-16 | 微信式左竖栏/列表/主工作区完成，消息、联系人、插件市场均为一级入口，状态连续且可访问 |
| AC-17 | 产品表面无 Telegram/Grok/Gok 品牌残留；Fabushi 标识一致，合法 attribution/provenance 完整 |
| AC-18 | 远端依赖已有真实 Fabushi 服务/集成和 E2E，不能以本地 mock 宣称全功能 |
| AC-19 | 权限、隐私、密钥、取消/销毁、跨账号隔离及负面路径通过安全验收 |
| AC-20 | 大历史、并发、网络故障、内存/CPU/磁盘/功耗满足批准的真实预算 |
| AC-21 | 既有数据/路由/插件/设置迁移、升级、失败恢复与回滚通过 |
| AC-22 | 0 open in-scope blocker、0 stub/no-op/fake fallback、0 用 N/A 隐藏的产品功能 |
| AC-23 | 最新确认 upstream baseline 及差异审计闭合，无静默跳过新文件/新功能 |
| AC-24 | 独立逐项验收、许可证/分发审查、支持平台包与 release provenance 完整 |
| AC-25 | 所有同类产品表面遵守 single-composition：无 Bot/Human/Group/Channel/来源型平行完整 workspace/list/profile/composer/resource/settings 实现；差异均为 typed capability composition |
| AC-26 | 当前 Fabushi 完全缺失的上游能力均通过 rejected-owner analysis + ADR 建立最小 source-neutral owner，并接入统一 shell/canonical truth；0 因“没有现成架构”而丢失的功能 |
| AC-27 | 每个用户可见 capability 都有唯一 UI entry classification 与 canonical route；无因源码/品牌来源而新增的重复一级入口或独立 App |
| AC-28 | 消息领域使用一个 Conversation list/workspace 与一个 typed ConversationCreation flow；Human/Agent/Group/Channel/Hybrid/Topic 的创建、筛选、详情均不复制状态栈 |
| AC-29 | Search 只有一个 canonical owner；入口内、对象内、Universal Search 三种作用域及所有 picker/search consumer 均复用 typed query/result/provider 合同 |
| AC-30 | Search 的 account/privacy/membership/block/retention/resource/task 权限、本地+远端合并、排序/去重/分页/取消/stale fencing 有真实行为和负面测试证据 |
| AC-31 | 导航、对象动作、专用 surface、响应式折叠、键盘/焦点/屏幕阅读器与状态连续性通过 packaged UI acceptance；无隐藏/不可返回/上下文丢失的迁移功能 |
| AC-32 | Fabushi semantic design tokens 成为 UI 公共合同；新迁移 UI 0 任意颜色/字号/间距/圆角/阴影/motion magic values，legacy/source-named token 仅限登记的兼容层 |
| AC-33 | Button/Input/Menu/Dialog/ListRow/Search/Avatar/Composer/Profile/feedback 等 canonical component owner 唯一；0 功能模块自建重复 primitive/pattern family |
| AC-34 | 所有核心 surface 均匹配 canonical screen pattern、slot、action hierarchy 和 loading/empty/error/offline/permission/data-heavy state matrix |
| AC-35 | Fabushi light/dark visual language、typography、spacing、shape、elevation、icon/avatar、content copy 与 motion 一致，无拼接式多套视觉系统 |
| AC-36 | fixed visual test matrix 覆盖支持的关键 viewport、light/dark、中文/英文、长文本/RTL、keyboard/focus/screen reader 与 reduced-motion |
| AC-37 | current-head GitHub Actions 生成并校验 approved visual baselines；非预期 geometry/style/state diff fail closed，故意变更需明确 review/provenance |
| AC-38 | 大会话/长标题/多附件/多成员/大量搜索结果/Agent tool events 下无裁切重叠、无无界增长，虚拟化/滚动/sticky/ellipsis 行为符合 screen contract |
| AC-39 | 用户可见品牌 asset、图标、头像 fallback、文案、通知和系统 surface 统一 Fabushi；来源品牌只存在合法 provenance/compatibility 记录 |
| AC-40 | 新 UI primitive/pattern/screen/token 只有在 existing design system 无法承接时才可新增，并有 design rationale、owner、复用范围、迁移计划和 visual acceptance evidence |
| AC-41 | applicable requirements/AC 100% 进入 RTM；每项均有 oracle/invariant、test IDs、current-head evidence 和独立 verdict，无 orphan requirement/test |
| AC-42 | 高风险功能同时通过 unit/table/property-state-machine/contract/integration 与 real packaged E2E；只靠 happy-path 或 mock 不得 verified |
| AC-43 | Conversation/Agent turn 的 submitted→accepted→streaming/tool→terminal→settled 全生命周期通过 temporal oracle；terminal final 在 quiet window、switch/reconnect/reload/restart 后不消失、不回滚、不重复、不串线 |
| AC-44 | transcript/collection ordering、optimistic→authoritative merge、late baseline/snapshot、duplicate/out-of-order event 有 property + integration + packaged regression 证据，DOM/projection/persisted order 一致 |
| AC-45 | UI functional + visual + interaction + a11y + responsive + locale + timeline evidence 均通过；session video 被独立 reviewer 实际审阅而非仅存在 artifact |
| AC-46 | 所有 escaped/manual defects 已进入 regression ledger；每个已关闭 defect 有 reproducible test、root-cause lower-layer coverage 与 current-head evidence |
| AC-47 | disconnect/reconnect、timeout、cancel、restart、stale/duplicate/out-of-order、permission/service failure 的适用 fault/recovery scenarios 全通过；0 rerun-until-green 掩盖的 flaky critical gate |
| AC-48 | approved performance/soak budgets 在 large history/roster/search/attachments/Agent tool events 下通过，长会话无无界内存/CPU/磁盘增长或 UI drift |
| AC-49 | security/privacy negative paths、account isolation、secret/permission lifecycle、Mini App/WebMCP boundaries、logs/redaction 与适用 WCAG/OWASP gates 通过 |
| AC-50 | exact-head release candidate 满足 entry/exit criteria、0 open P0/P1/blocker、RTM/evidence bundle 完整、独立 acceptance report 为 ACCEPT；blocked/skipped/flaky/not-run 均不能计为通过 |

## 14. 本次交付与后续任务

本次交付为 spec/项目入口修订，不是全部功能实现。未在本次生成新全量 inventory、重写 production modules、改变现有 ledger 验证状态或完成 UI。既有代码是否满足本 Revision 必须重新逐项审计，不能凭旧 status 断言“都没实现”或“已完成”。

下一任务：`projects/fabushi-communication-platform/management/tasks/P0-product-domain-and-telegram-absorption.md`。TDRP-001 必须执行到真实等价实现与取证，而不能在 research/absorption plan 停止。

## 15. References / provenance

- 用户 2026-10-07 明确要求及微信桌面结构参考（只记录布局要求，不公开原截图）。
- Fabushi repository: https://github.com/bhrumom/fabushi-desktop
- Architecture origin: https://github.com/bhrumom/fabushi-desktop/pull/20
- Full source reference: https://github.com/telegramdesktop/tdesktop
- Upstream README/license: https://github.com/telegramdesktop/tdesktop/blob/f23c37857220eb84f8559f0901ea26fb304b564b/README.md and LICENSE in the same exact tree.
- Companion TDRP-001、当前 canonical Bot spec、原有 module dossiers/ledger/ADR 与 current-head production evidence。

### Revision 9 deterministic read-through — TooManyCooks 7,057–7,154

Pinned authority `tzcnt/TooManyCooks@b86af81982860c96295a7e95e4c60cb335615cef` is now exact-read through all **126** non-directory entries. Orders **7,057–7,154** revalidate the prior temporary 7,057–7,064 read and cover all remaining runtime headers, implementations and package inputs.

Existing-owner-first audit found real shipping candidates: `GatewayHostSupervisor`; `HostRunnerComposition`; the frozen Host→Runner bridge; `RoutedProviderCancellation/RoutedProviderTaskRegistry`; `TurnSettlement`; canonical Mahayana messaging; `PressureCpuProfiler`; and existing GitHub Actions Build/Release owners. Applicable/open behavior includes executor/priority restoration, awaitable/fork/join lifetime, UAF-safe settlement, atomic wait/wake and teardown, foreign callback lifetime/cancellation, queue close/reclamation, CPU quota/topology/work-stealing capacity behavior, and sanitizer/fuzz/coverage/build-option consistency. Exact hwloc pinning is not presumed complete without a dedicated shipping worker-pool owner. vcpkg/CMake/version descriptors are build/provenance inputs, not a new product owner.

No TMC/Telegram runtime/provider or duplicate Identity/Conversation/Message/Search/Settings/Marketplace is introduced. Accounting is **7,154/16,125 read; 8,971 unread; 15,845 unknown; 0 omitted**. Reading closes no unknown. Next deterministic entry is **7,155** `Telegram/ThirdParty/cld3::.github/workflows/main.yml@b26ff5ec0300683c819c0c7f17edfc21bfdd0fc3` in `google/cld3`.
