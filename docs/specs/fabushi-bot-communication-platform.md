# Fabushi Bot 全量通信能力等价重写与统一产品 Spec

Status: active  
Spec ID: FBCP-001  
Revision: 3  
Last updated: 2026-10-07  
Owner: Fabushi Desktop  
Canonical project: `projects/fabushi-communication-platform`  
Companion implementation contract: `docs/specs/telegram-desktop-rust-equivalence-migration.md` (TDRP-001 Revision 5)  
Implementation status: **requirements updated; full migration not accepted**

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
