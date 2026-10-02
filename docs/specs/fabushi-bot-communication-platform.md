# Fabushi Bot Communication Platform：完整吸收 Telegram 通信能力 Spec

Status: active  
Spec ID: FBCP-001  
Revision: 1  
Last updated: 2026-10-02  
Owner: Fabushi Desktop / Communication Platform  
Canonical project: `projects/fabushi-communication-platform`  
Implementation status: **not implemented / not accepted by this spec**

> **Fabushi Bot is the product. Telegram is a complete communication capability source and network integration, not a separate product, workspace, or architectural root.**
>
> 本项目不是“做一个 Telegram 客户端，再把 Bot 接进去”，也不是“给 Telegram 加一个 AI 按钮”。目标是在现有 Fabushi Bot / Agent 产品基础上，完整吸收 Telegram Desktop 所代表的通信能力，使人、Agent、群组、频道、消息、媒体、通话、任务、Computer、Plugins/MCP 与 Automations 成为同一个产品系统中的一等能力。

## 1. 产品定义

Fabushi Desktop 的唯一产品根是现有 Bot / Agent 产品。

Telegram 提供两类输入：

1. **完整通信产品能力来源**：账号、联系人、会话、消息、群组、频道、Topic、搜索、媒体、通话、通知、Stories、Bots/Mini Apps、支付及其他冻结基线能力；
2. **Telegram 网络 provider**：通过 Telegram API / MTProto / Bot Platform 与真实 Telegram 用户、群组、频道和 Bot 互通。

最终产品不得呈现为两个并列应用：

- 不允许顶层 `Bot Workspace` 与 `Telegram Workspace`；
- 不允许把 Telegram 做成独立子应用后再 iframe/WebView/bridge 到 Bot；
- 不允许用户必须在两套联系人、会话列表、消息模型和设置体系之间切换；
- 不允许把 Agent Runtime 塞入 Telegram 的历史架构并让 Telegram 成为产品根。

最终应该只有一个 Fabushi 产品外壳、一套统一信息架构和一套统一的产品领域模型。

## 2. 与 PR #20 的关系

当前设计读取时，PR #20 `refactor/grok-018-architecture-rebuild` exact HEAD 为：

`59a06a43d20600915e79770d83773522a7c58b0c`

该 PR 提供现有 Bot 产品的关键基础：

- Agent / conversation 产品模型；
- Coordinator / Host / Runner 独立边界；
- Shared Room；
- Human / Agent member distinction；
- Plugins / MCP；
- Computer；
- Automations；
- async tasks / subagents；
- transcript / composer / permissions；
- durable runtime / lifecycle。

这些能力是 **Fabushi 产品基础**，不是要被 Telegram 架构替换的临时层。

从本 Spec 起，完整通信平台是一个明确批准的 Fabushi product extension。PR #20 中“没有 Grok counterpart 的 Telegram/messaging 子系统默认移除”规则，不得用于删除 FBCP-001 明确批准的 communication/Telegram provider 能力。

同时，本 Spec 也不授权破坏 PR #20 的 Coordinator / Host / Runner 等核心架构边界。通信能力应接入现有产品，而不是把 Agent Runtime 重写成 Telegram runtime。

每轮实现仍需重新读取 PR #20 / canonical main 的 exact HEAD；这里记录的 SHA 只是本 Revision 的设计输入，不是永久固定依赖。

## 3. 产品核心模型

### 3.1 Identity

产品中身份是可组合的，不把网络身份和产品身份混成一个 ID。

至少有：

- `FabushiIdentity`
- `HumanIdentity`
- `AgentIdentity`
- `TelegramUserIdentity`
- `TelegramBotIdentity`
- 后续 provider identity

一个 Participant 可以绑定多个 provider identity，但绑定必须显式、可撤销、可审计。

### 3.2 Participant

`Participant` 是 Conversation 中的参与者：

- Human
- Agent
- System / service participant（仅在明确需要时）

Agent 与 Human 在会话层都是一等参与者，但权限、能力、可见数据和执行语义不同。

### 3.3 Conversation

`Conversation` 是产品统一会话抽象，而不是 TelegramChat 或 AgentThread 的别名。

Conversation 可绑定：

- Fabushi-native conversation；
- Telegram private chat；
- Telegram group；
- Telegram supergroup；
- Telegram channel；
- Telegram topic/forum；
- Agent private conversation；
- hybrid human-agent room。

Conversation 只定义产品层共同语义。Telegram 独有的复杂语义必须通过 typed provider extension 保留，不能为了“统一模型”丢掉权限、Topic、Channel、read state、message identity 等真实含义。

### 3.4 Message / Thread / Attachment

统一产品模型至少包括：

- Message
- Thread / Topic
- Attachment
- Reaction
- Reply / Quote
- Forward provenance
- Edit / Delete state
- Read / unread state
- Draft
- Scheduled send
- Media transfer
- Provider-specific extensions

Telegram 原始 ID、random id、peer、topic、pts/qts/seq/date 等不能被通用模型抹掉；它们保存在 Telegram provider 的强类型状态中。

### 3.5 Agent-native entities

现有 Bot 产品的一等对象继续保留：

- Agent
- Task
- Subagent
- Automation
- ComputerSession
- Plugin / MCP capability
- Tool call
- Artifact
- Permission request
- Waiting-user state

这些对象可以与 Conversation/Message 建立明确关联，但不能变成 Telegram message metadata 的附属品。

## 4. 产品体验

### 4.1 Unified Inbox / Sidebar

用户看到的是统一列表，例如：

- Alice
- Product Team
- Research Agent
- Family
- Coding Agent
- News Channel
- My Secretary

列表可以显示 provider/Agent 类型的轻量标识，但不能要求用户先选择“进入 Telegram”或“进入 Agent”。

### 4.2 Unified Conversation Surface

打开任何 Conversation 都使用同一个 Fabushi conversation shell。

能力按参与者/provider 动态出现：

- 人类聊天：消息、媒体、搜索、通话、资料；
- Agent：reasoning、tools、Computer、Automations、Plugins；
- hybrid room：人类通信 + Agent 协作；
- Channel：Telegram channel-specific actions；
- Topic：Telegram topic-specific navigation；
- Bot/Mini App：对应 provider extension。

### 4.3 Unified Composer

Composer 既是通信入口，也是工作入口。

可包含：

- text / rich text
- attachments
- emoji/stickers
- @Human
- @Agent
- commands
- tool / plugin selection
- schedule
- create task
- create automation
- reply / quote
- provider-specific send options

UI 必须按上下文保持简洁，不能把全部高级能力永久堆在主输入框。

### 4.4 Unified Context Panel

一个 Conversation 的上下文可以包含：

- People
- Agents
- Files
- Media
- Tasks
- Automations
- Computers
- Permissions
- Search
- Provider details

Telegram 群资料页、频道信息、成员管理等必须被吸收到该产品信息架构，而不是打开第二套 Telegram 设置应用。

## 5. 产品架构

### 5.1 Product Shell

现有 Fabushi Bot UI / conversation / agent workspace 是产品 shell。

产品 shell 负责：

- navigation
- unified inbox
- conversation presentation
- composer
- task/agent/computer/plugin/automation surfaces
- permission UX

### 5.2 Communication Core

新建独立通信领域，但它属于 Fabushi 产品：

- identity
- participants
- conversations
- messaging
- media
- calls
- contacts
- search
- notifications
- presence
- provider bindings

Communication Core 不直接调用模型，也不拥有 Agent runtime。

### 5.3 Telegram Provider

Telegram 能力进入 provider 边界，而不是成为产品根。

Telegram Provider 负责：

- Telegram authentication/account sessions
- TL / MTProto
- DC/session/update consistency
- peers/contacts
- dialogs/messages
- groups/channels/topics
- Telegram media
- Telegram calls
- Telegram notifications
- Telegram search semantics
- Telegram-specific Bot/Mini App/Stories/Premium/Stars/Business 等能力
- provider-specific persistence/recovery

根据 TDRP-001，Telegram/desktop-app 自有 C++ production logic 最终全部由 Rust production owner 取代。

### 5.4 Agent Runtime

PR #20 / canonical Bot architecture 继续负责：

- Coordinator
- Host
- Runner
- provider/model execution
- tool lifecycle
- Plugins/MCP
- Computer
- Automations
- tasks/subagents
- permissions
- durable turn lifecycle

Communication Core 不复制这些职责。

### 5.5 Interaction Gateway

Communication Plane 与 Agent Plane 之间必须有明确的 `InteractionGateway` / `ContextBroker`。

它负责：

- explicit user invocation
- Agent mention routing
- attachment handoff
- selected message/context handoff
- permission checks
- provider/terms policy
- consent state
- redaction
- audit metadata
- output publish policy

禁止 Telegram sync/update handler 直接调用模型。

## 6. 典型产品流

### FLOW-01 — 普通 Telegram 私聊

Telegram update → Telegram Provider → Communication Core → unified Conversation → UI。

没有 Agent invocation 时，消息不进入 Agent runtime。

### FLOW-02 — 从消息显式调用 Agent

用户选择消息/附件 → Ask Agent / @Agent → Interaction Gateway → Coordinator → Host/Runner → Agent result。

Agent result 可以：

- 保持为 Fabushi task/artifact；
- 显示在本地 Agent context；
- 在用户明确决定且 provider policy 允许时发送回 Telegram。

### FLOW-03 — Hybrid Room

同一个 Conversation 同时存在 Human 与 Agent participant。

Agent 的触发规则必须显式：

- mention
- direct request
- allowed automation
- permitted bot/business integration

不能因为 Agent 在 room 中就默认读取并处理所有消息。

### FLOW-04 — Agent 作为 Telegram Bot Identity

Fabushi Agent 可绑定 Telegram Bot identity。

此时 Agent 可以通过 Telegram Bot Platform 成为真实网络参与者，但：

- identity mapping 必须显式；
- permissions 与 room scope 分离；
- Bot Platform 数据规则单独执行；
- Bot 输出和 Fabushi local Agent output 不能混淆。

### FLOW-05 — Message → Task / Automation

用户可从任意允许的消息创建：

- task
- reminder
- automation
- computer job
- plugin workflow

原消息作为有 provenance 的输入引用，不复制成无来源 memory。

### FLOW-06 — Agent execution → Communication

Agent 完成 Computer / plugin / tool 工作后，可产生 Artifact。

Artifact 可以被用户发送到 Conversation，或在明确允许的自动化中发布。

“Agent 完成”与“已经向 Telegram 网络发送”是两个不同状态。

## 7. Permissions

权限不是“Agent 全局有 GitHub/Computer”。

至少同时考虑：

- Agent identity
- Conversation / room
- Human principal
- provider account
- capability/tool
- data scope
- action class
- duration

例：

`Coding Agent` 在 Product Team 中可以：

- read explicitly routed messages
- receive @mentions
- use project-scoped GitHub
- use Computer

但不能自动：

- read unrelated Telegram chats
- send as the user
- access payments
- invite members
- create persistent automations

高风险动作需要显式 approval 或预先配置的窄权限。

## 8. Telegram 完整能力吸收

“Telegram complete capability integration” 不是只做消息收发。

TDRP-001 必须研究并交付固定基线中适用的全部能力，至少覆盖：

- startup/account/login/multi-account
- contacts/peers
- dialogs/archive/folders
- messages/reply/quote/forward/edit/delete
- drafts/scheduled/silent send
- reactions/polls
- groups/supergroups/channels/topics
- members/admin/permissions/invites
- search
- files/photos/video/audio/voice
- upload/download/cache/streaming
- stickers/GIF/custom emoji
- Stories
- calls/video/screen sharing
- notifications/tray/badges
- settings/privacy/local lock
- Bots/inline bots/Mini Apps/WebView
- Premium/Stars/gifts/business
- payments/passport/webauthn where applicable
- export/local data/recovery
- themes/i18n/RTL/IME/accessibility
- install/update/platform integration

每项能力必须进入 Fabushi 产品域。没有合适通用抽象时，可以保留 `TelegramExtension` typed capability；禁止为了架构漂亮而删除 Telegram 特性。

## 9. Source-informed implementation

Telegram Desktop 固定源码仍然是功能发现和行为研究来源，但不是产品架构模板。

TDRP-001 负责：

- 完整源码闭包；
- capability research；
- C++ production logic inventory；
- behavior oracle；
- Rust replacement；
- Telegram provider 实现；
- provider-specific acceptance。

FBCP-001 负责：

- 产品领域；
- Bot + communication 融合；
- unified UI；
- identity/conversation contracts；
- Agent interaction；
- permissions；
- compliance/data boundaries；
- product acceptance。

## 10. 数据与 AI 边界

Telegram client data 与 Agent/model data 必须默认隔离。

### DATA-01 — No implicit AI ingestion

登录 Telegram、同步消息或打开 Conversation 不等于授权把该数据发送给模型、第三方 AI API、memory、training、retrieval 或 Agent runtime。

### DATA-02 — Explicit routing

只有明确的产品动作或经批准的 automation 才能通过 Interaction Gateway 传递上下文。

### DATA-03 — Terms-aware policy

Telegram Client API、Bot Platform、Business integration 的条款和可处理数据范围不同。

实现和发布时必须重新读取最新官方条款；policy engine 不得把所有来源视为同一种许可。

### DATA-04 — No AI scraping architecture

禁止设计“后台读取所有群/频道/历史 → 建 AI dataset/global memory”的默认路径。

### DATA-05 — Third-party model disclosure

若上下文将发送给第三方模型/provider，必须满足当前条款、产品 privacy policy、用户授权和必要的 redaction。

### DATA-06 — Search is not Agent memory

通信搜索索引与 Agent memory 是不同存储/用途。不得因本地搜索需要而自动把 Telegram 数据纳入 Agent memory。

官方动态参考：

- https://core.telegram.org/api/terms
- https://telegram.org/tos/bot-developers
- https://core.telegram.org/api/bots/ai

## 11. 语言和实现原则

### C++ replacement

所有 Telegram/desktop-app 自有 C++ production logic，最终由 Rust 取代。

不得用：

- C++ helper process
- Qt business UI
- TDLib wrapper
- original tdesktop binary
- Rust façade around original C++ logic

冒充完成。

### Best-fit non-C++

其他边界选择最合适语言：

- Rust：communication core、Telegram protocol/sync/storage、native logic、安全/并发/高性能路径；
- TypeScript/React：现有产品 UI/DOM/Electron 边界在它仍为最佳选择时；
- platform language：仅窄 OS bridge；
- shader/resource/manifest：使用自然格式；
- Python/Node：仅在工具/生态边界有明确 ADR 时。

语言不能制造第二套业务 owner。

## 12. Provider abstraction 原则

不能为了未来多 provider，把 Telegram 降级到“最低公共分母”。

`CommunicationProvider` 只抽象稳定公共行为：

- account/session
- identity resolution
- conversation discovery
- send/receive
- media transfer
- presence/typing where applicable
- notifications
- capability discovery

Telegram-specific：

- channels
- forum topics
- stories
- stars
- business
- specialized message/action types
- MTProto update semantics

必须通过 typed extensions 保留。

## 13. Migration strategy

### P0 — Product domain and absorption design

先完成：

1. PR #20 / canonical Bot architecture inventory；
2. Telegram complete capability graph；
3. existing Bot domain ↔ communication domain collision analysis；
4. unified Identity / Participant / Conversation / Message contracts；
5. Telegram provider boundary；
6. Interaction Gateway / data policy；
7. unified Inbox / Conversation / Composer information architecture；
8. permissions model；
9. persistence ownership；
10. architecture ADR backlog。

### P1 — First product vertical slice

必须以现有 Bot 产品为壳，而不是做 Telegram demo app：

1. Fabushi 启动；
2. Telegram account sign-in；
3. unified sidebar 同时显示 Agent 与 Telegram human conversation；
4. 打开 Telegram private chat；
5. send/receive Telegram message；
6. 从一条消息显式 Ask Agent；
7. Agent 通过现有 Coordinator/Host/Runner 执行；
8. 结果成为 Fabushi task/artifact；
9. 用户明确选择是否发送结果回原 Conversation；
10. restart 后所有 identity/conversation/task linkage 正确恢复。

### P2 — Core communication completeness

contacts、dialogs、groups、channels、topics、search、notifications、settings、permissions、完整 message types。

### P3 — Media / calls

media lifecycle、record/playback、calls/video/screen sharing、device/network recovery。

### P4 — Agent-native collaboration

Agent membership、@Agent、Telegram Bot identity、room-scoped permissions、group workflows、automations、Computer/artifact publish。

### P5 — Long-tail Telegram completeness

Stories、Bots/Mini Apps、Premium/Stars/gifts/business、payments/security/export 等冻结基线能力。

### P6 — Platform/release hardening

Windows/macOS/Linux、install/update/rollback、accessibility、performance/power/security、license/API/brand/signing。

### P7 — Cutover

删除重复 product shell、Telegram-only workspace、C++ fallback、legacy parallel messaging model 和 duplicate state owners。

## 14. Verification

所有构建、lint、schema、generator、test、benchmark、fuzz、package、acceptance 只能在 GitHub Actions 或 `htch-runtime`。

不得使用本地构建作为验收。

至少验证：

- Bot existing capability regression；
- Telegram provider behavior/protocol；
- unified product domain；
- identity mapping；
- no duplicate state owner；
- normal chat never invokes Agent unexpectedly；
- explicit Agent handoff carries only intended context；
- restart/reconnect；
- human + Agent hybrid room；
- packaged application；
- platform UI/accessibility；
- current Telegram terms policy configuration；
- performance/power；
- C++ production dependency absence。

## 15. Acceptance Criteria

**AC-01 — Single product root**  
Fabushi Bot/Agent Desktop 是唯一产品根；不存在独立 Telegram app/workspace 作为最终 UX。

**AC-02 — Bot foundation preserved**  
Coordinator/Host/Runner、Agent、Computer、Plugins/MCP、Automations 等现有 Bot 核心能力继续是 canonical product capability，没有因 Telegram 集成退化。

**AC-03 — Complete Telegram capability absorption**  
TDRP-001 的全部适用 Telegram capability 已进入 Fabushi 产品，零未解释功能缺口。

**AC-04 — Unified product model**  
Identity/Participant/Conversation/Message 等核心模型统一，同时 Telegram-specific semantics 没有被最低公共分母丢失。

**AC-05 — C++ replacement**  
Telegram/desktop-app C++ production logic 为零，全部由 Rust owners 或明确允许的外部系统能力取代。

**AC-06 — Unified UX**  
Inbox、Conversation、Composer、Search、Context、Settings 是 Fabushi 产品体验，不需要切换到 Telegram 子应用。

**AC-07 — Human + Agent collaboration**  
Conversation 可以安全地承载 Human、Agent、Task、Artifact、Automation、Computer 等协作实体。

**AC-08 — Permission correctness**  
Agent 只在明确 principal/room/provider/data/tool scope 内执行；没有静默越权或代表用户隐式发送。

**AC-09 — Data/AI boundary**  
Communication data 与 Agent/model plane 默认隔离；所有跨界路径可解释、可审计、符合最新条款和用户授权。

**AC-10 — Telegram interoperability**  
真实 Telegram 客户端、群组、频道、媒体、通话及其他适用能力互通正确。

**AC-11 — No duplicate architecture**  
没有 Telegram-only product shell、第二套 conversation truth、第二套 identity owner 或 parallel agent runtime。

**AC-12 — Existing Bot regression**  
PR #20 / canonical Bot requirements 的适用能力没有因通信平台加入而退化。

**AC-13 — Exact-head evidence**  
所有验收绑定目标 exact SHA 和 GitHub Actions / htch-runtime 的真实 run/artifact。

**AC-14 — Release/legal provenance**  
Telegram source-informed provenance、GPL、API terms、Bot terms、assets、branding、signing 等均有 release review。

**AC-15 — Independent product acceptance**  
在 packaged build 中，通信、Agent、Computer、Plugins、Automations 和 Telegram 网络能力表现为一个连贯产品，由独立验收确认。

## 16. 当前状态

本 Spec 目前只完成产品方向与架构治理定义。

- PR #20 Bot foundation：进行中；
- Telegram source research：未完成；
- Communication Core：未实现；
- Telegram Provider：未实现；
- Unified product model：未实现；
- Agent/communication Interaction Gateway：未实现；
- Unified UX：未实现；
- Full Telegram absorption：blocked；
- release：blocked。

不得把 Spec 文件本身当作产品完成。

## 17. References

- Latest explicit user direction, 2026-10-02: the product is the existing Bot product with all Telegram functionality absorbed into it, not a Telegram product with Bot integration.
- PR #20: https://github.com/bhrumom/fabushi-desktop/pull/20
- TDRP-001: `docs/specs/telegram-desktop-rust-equivalence-migration.md`
- Telegram API terms: https://core.telegram.org/api/terms
- Telegram Bot Platform terms: https://telegram.org/tos/bot-developers
- Telegram AI bot features: https://core.telegram.org/api/bots/ai
