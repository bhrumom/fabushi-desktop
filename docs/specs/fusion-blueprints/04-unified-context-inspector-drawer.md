# Fabushi 统一融合蓝图 04：唯一右侧上下文详情栏（合并 7 个竞争抽屉，废除假面板）
# Volume 04: Unified Context Inspector Drawer, Eradication of Competing Drawers, and Dynamic Contextual Inspector

> **规范编号**: `FBCP-BLUEPRINT-04` / `UFX-BLUEPRINT-04`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 彻底根治 7 个竞争抽屉互斥并存的架构割裂，建立唯一生产级右侧上下文详情栏（Unified Context Inspector）。
> 2. 彻底废除 `OpenClawAskCopilotPanel`：移除本地假在线状态（`isGatewayOnline={false}`）、假连接器提示（“18 个连接器全量就绪”）、以及快捷 Action Pill 点击覆盖主输入台草稿（`composerDraftStore.setDraft`）的破坏性逻辑。
> 3. 根治 `TdrpProfileDetailsDrawer` 的硬编码假星标余额（1250 Stars）与静态死数据占位，整合真实星币账本与媒体检索。
> 4. 按活跃对象类型（`Human` 真人 / `Group` 普通群 / `Channel` 广播频道 / `Agent` 智能体 / `Hybrid` 人机混合协同群）动态组织分面 Tabs：身份资料、群成员与细粒度管理员权限、共享媒体与附件、智能体模型与技能、Cron 定时 Routines、外部渠道连接器以及会话大纲与子任务。
> 5. 严格遵循 200–600 行内聚预算架构（`Same Owner != Same File`），在 `frontend/src/recovered/features/conversation/workspace/context-inspector/` 下建立清晰模块拓扑。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **右侧上下文根容器**<br>(Root Inspector Container) | `ui/src/components/layout/right-sidebar.ts`<br>`ui/src/pages/chat/context-inspector.ts` | `info/info_controller.cpp`<br>`info/info_content_widget.cpp`<br>`window/window_controller.cpp` | `frontend/src/production/ProductionRenderer.tsx:906-926,4319-4347,4375-4463` | `ContextInspector`<br>`frontend/src/recovered/features/conversation/workspace/context-inspector/index.tsx` |
| **网关诊断与伪面板消灭**<br>(AskCopilotPanel Deprecation) | `src/gateway/client.ts`<br>`ui/src/components/gateway-status.ts` | `core/network_lifecycle.cpp`<br>`api/api_common.cpp` | `frontend/src/production/openclaw/OpenClawAskCopilotPanel.tsx:1-269`<br>`ProductionRenderer.tsx:4319-4331` | 彻底移除假面板；真实网关诊断下沉至 `SkillsChannelsTab`，禁止篡改输入台草稿 |
| **对象详情与星标余额**<br>(Profile Hero & Stars Ledger) | `ui/src/pages/identity/profile.ts`<br>`src/agents/identity.ts` | `info/profile/info_profile_widget.cpp`<br>`api/api_credits.cpp`<br>`data/data_user.cpp` | `frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx:1-271`<br>`ProductionRenderer.tsx:4337-4347` | `ProfileTab.tsx`<br>挂载真实 ADR-002 星币余额订阅与真实呼叫入口 |
| **群成员管理与权限细化**<br>(Group Members & Admin Rights) | `src/agents/multi-agent/lineage.ts`<br>`ui/src/components/roster.ts` | `info/members/info_members_widget.cpp`<br>`data/data_chat.cpp`<br>`api/api_chat_participants.cpp` | `frontend/src/recovered/features/agent-info/group-members/view.tsx:1-94`<br>`ProductionRenderer.tsx:4417-4431` | `MembersTab.tsx`<br>统一支持普通群/超级群成员列表、封禁、禁言与管理员提权 |
| **共享媒体与历史归档**<br>(Shared Media & Attachments) | `src/media/storage.ts`<br>`ui/src/pages/chat/shared-media.ts` | `info/media/info_media_widget.cpp`<br>`info/media/info_media_provider.cpp` | `frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx:232-266`（静态占位） | `MediaTab.tsx`<br>真实验收图片、视频、文件、链接、音频分面虚拟列表与全量导出 |
| **智能体设置与提示词**<br>(Agent Persona & Model Settings) | `src/agents/configuration.ts`<br>`ui/src/pages/agent/settings.ts` | `info/profile/info_profile_badge.cpp`<br>`data/data_user_business.cpp` | `frontend/src/recovered/features/agent-info/settings/view.tsx:1-82`<br>`ProductionRenderer.tsx:4391-4416` | `AgentSettingsTab.tsx`<br>收拢名称、身份描述、头像编辑器与思考强度配置 |
| **渠道连接与 MCP 技能**<br>(Channels & Skills Dock) | `src/connectors/`<br>`src/plugins/mcp/`<br>`ui/src/pages/channels/` | `api/api_authorizations.cpp`<br>`core/local_url_handlers.cpp` | `frontend/src/recovered/features/agent-info/channels/view.tsx:1-151`<br>`ProductionRenderer.tsx:4445-4453` | `SkillsChannelsTab.tsx`<br>集成真实渠道连接器状态与 MCP 工具白名单 |
| **定时自动化与历史回溯**<br>(Routines & Automation Triggers) | `src/cron/schedule.ts`<br>`ui/src/pages/automations/` | `history/view/controls/history_view_scheduled_box.cpp` | `frontend/src/recovered/features/automations/routines/view.tsx:1-309`<br>`ProductionRenderer.tsx:4432-4444` | `RoutinesTab.tsx`<br>按会话过滤展示 Cron 任务、执行触发器与历史运行结果 |
| **会话大纲与子任务跟踪**<br>(Session Outline & Async Tasks) | `ui/src/components/outline.ts`<br>`src/sessions/subagent-tracker.ts` | `data/data_forum_topic.cpp`<br>`data/data_thread.cpp` | `frontend/src/recovered/features/conversation/workspace/conversation-outline-view.tsx:1-199`<br>`frontend/src/recovered/features/agent-info/async-tasks/view.tsx:1-143` | `OutlineTasksTab.tsx`<br>合并会话大纲条目与后台 7 态异步任务执行树 |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - 上下文详情由 `Telegram/SourceFiles/info/info_controller.cpp` 统领，采用 `Info::Key`（用户 Peer、频道 Channel、话题 Topic 或设置 Settings）驱动。
  - 内容区挂载单一 `Info::ContentWidget`，内部派生 `Info::Profile::Widget`（详情）、`Info::Members::Widget`（成员列表与管理员权限分配）、`Info::Media::Widget`（多媒体分面，涵盖照片、视频、文件、音频、共享链接、语音消息六大分片），具有独立的按月聚合时间轴与高速分块缓存。
  - 所有操作具备原子权限校验（`data_chat->canAddMembers()`, `data_chat->canBanMembers()`），网络错误具备 fail-closed 提示。
- **OpenClaw 侧**：
  - 侧边栏由 `ui/src/pages/chat/context-inspector.ts` 与 `src/agents/configuration.ts` 驱动，分为智能体配置、ACP 协议参数、技能工具列表（MCP Tools）、外部渠道连接向导及运行日志。
  - 会话大纲（Outline）实时捕获用户 Prompt、思考链（Thinking Steps）及工具调用结果；异步子代理通过 `subagent-tracker.ts` 呈现执行树。

#### Q2（现有链路定位）
- **7 个孤立抽屉的挂载现状（`ProductionRenderer.tsx`）**：
  - 伪网关抽屉：`copilotPanelOpen` 控制 `OpenClawAskCopilotPanel.tsx:4320`。
  - 静态资料抽屉：`tdrpDetailsOpen` 控制 `TdrpProfileDetailsDrawer.tsx`（通过 `TdrpConversationOverlay.tsx:4337` 间接挂载）。
  - 会话大纲浮层：`conversationOutlineAgent` 控制 `ConversationOutlinePanel:4375`。
  - 任务详情浮层：`asyncTasksAgent` 控制 `AsyncTasksPanel:4382`。
  - 智能体设置抽屉：`agentSettingsOpen` 控制 `AgentSettingsPanel:4391`。
  - 群成员管理抽屉：`groupInfoPaneOpen` 控制 `GroupMembersPane:4417`。
  - 定时 Routines 抽屉：`routinesInfoPaneOpen` 控制 `mountRoutinesInfoPane:4432`。
  - 渠道配置抽屉：`channelsInfoPaneOpen` 控制 `mountAgentInfoChannels:4445`。
  - 运行时详情抽屉：`runtimeDetailPaneOpen` 控制 `FabushiAgentRuntimeDetailSections:4454`。
- **现状结论**：所有抽屉彼此互斥，存在 9 个独立状态变量（`setCopilotPanelOpen`, `setTdrpDetailsOpen`, `setAgentSettingsOpen`, `setGroupInfoPaneOpen`, `setRoutinesInfoPaneOpen`, `setChannelsInfoPaneOpen`, `setRuntimeDetailPaneOpen`, `setConversationOutlineAgentId`, `setAsyncTasksAgentId`），点击一个抽屉必须手工清空其余抽屉，导致极度混乱的竞态。

#### Q3（断点精确诊断）
1. **假在线与草稿擦除断点**（`OpenClawAskCopilotPanel.tsx:25,326-329`）：
   - `isGatewayOnline={false}` 被直接写死，却在状态卡片中渲染假绿色红色的未连接状态；
   - 点击快捷操作药丸（Pill）时直接调用 `composerDraftStore.setDraft(activeAgent.id, { prompt: actionText, attachments: [] })`，**直接抹除用户正在输入的输入台草稿与附件**，严重违反数据完整性原则。
2. **假星标余额与无接线模态框断点**（`TdrpProfileDetailsDrawer.tsx:29,207-230`）：
   - `starsBalance = 1250` 硬编码；
   - “群组管理”、“导出聊天记录”、“云密码与隐私”三个条目仅有占位 `onClick` 回调，弹出的模态框脱离上下文；媒体/投票/待办 Tabs 仅有静态“暂无”文字。
3. **群成员权限控制缺失断点**（`GroupMembersPane/view.tsx:78-83`）：
   - 仅支持简单的 `Remove` 和 `Add Member`，缺乏管理员权限任命、封禁时长、只读禁言（Mute）与反垃圾设置（TCG-N15）。
4. **抽屉堆叠与视口抢占断点**：
   - `ConversationOutlinePanel` 与 `AsyncTasksPanel` 采用浮动面板（`useMovablePanel`），在小屏或全屏工作区中与右侧抽屉直接重叠撞车。

#### Q4（状态机与安全防护）
- **统一抽屉状态机**：
  $$\text{InspectorState} = \langle \text{isOpen}: \text{boolean}, \text{activeTab}: \text{TabId}, \text{targetId}: \text{string}, \text{accountGen}: \text{number} \rangle$$
  - 当切换当前会话（`activeAgent.id`）时，`targetId` 立即同步切换；若原 Tab 在新对象类型中不支持，自动降级为 `profile` Tab。
  - 当触发账号切换（`accountScopeGeneration` 单调递增）时，物理清空缓存中的成员、任务、渠道敏感凭据快照，严禁跨账号残留。
- **输入法与表单安全**：
  - 智能体设置及成员搜索中的输入控件，全面受 IME 保护（`isComposing` 时按 Enter 绝不触发变更提交或关闭抽屉）。

#### Q5（UI/UX 极致融合落点）
- 唯一入口：集成在统一直顶栏（`ConversationHeader`）右侧的“详情”图标（三点或侧边栏切换按钮），点击展开唯一 `ContextInspector`。
- 渐进披露（Progressive Disclosure）：默认展示首屏资料卡片与高频操作（呼叫、搜索、静音、成员列表）；高级设置（MCP 绑定、Cron 触发器、导出日志）按分面 Tab 懒加载，杜绝单屏信息过载。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-04-01** | AskCopilotPanel 假状态与草稿破坏性覆盖 | `frontend/src/production/openclaw/OpenClawAskCopilotPanel.tsx:25,326-329`<br>`ProductionRenderer.tsx:4320-4331` | 原型期遗留的未接线调试面板。硬编码 `isGatewayOnline={false}`，且快捷按钮直接覆写 `composerDraftStore`，导致用户输入的数据丢失。 | **P0 严重逻辑与数据损坏** |
| **BUG-04-02** | 7 个平行抽屉争抢右侧视口，状态变量爆炸 | `frontend/src/production/ProductionRenderer.tsx:906-926,4238,4319-4463` | 缺乏统一的 Context Inspector 架构。9 个独立的 boolean 状态在各自的触发器中互相 `setOther(false)`，维护成本极高且极易漏关。 | **P0 架构严重割裂** |
| **BUG-04-03** | TdrpProfileDetailsDrawer 硬编码假星标与空动作 | `frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx:29,139-158,207-266` | 假数据静态打桩。`starsBalance = 1250` 未接入真实 ADR-002 账本，媒体 Tabs 未连接消息附件存储服务。 | **P1 虚假与断头交互** |
| **BUG-04-04** | GroupMembersPane 缺乏现代超级群管理能力 | `frontend/src/recovered/features/agent-info/group-members/view.tsx:68-93` | 仅支持固定名单增删，未与 Telegram TCG-N15（管理员角色、禁言、封禁权限）深度结合，无法管理大规模群组。 | **P1 功能未闭环** |
| **BUG-04-05** | Routines 与 Channels 抽屉孤岛化脱离上下文 | `frontend/src/recovered/features/automations/routines/view.tsx:18-32`<br>`frontend/src/recovered/features/agent-info/channels/view.tsx:29-49` | 本应作为当前智能体的属性面板，却作为独立的全局弹层挂载，无法在查看智能体资料时顺畅联动。 | **P1 交互断层** |
| **BUG-04-06** | Outline 与 Tasks 使用浮动面板遮挡聊天流 | `frontend/src/recovered/features/conversation/workspace/conversation-outline-view.tsx:7`<br>`frontend/src/recovered/features/agent-info/async-tasks/view.tsx:14` | 依赖 `useMovablePanel` 悬浮在主舞台上方，在 1200px 以下窗口中频繁挡住主输入框和最新消息行。 | **P2 布局干扰** |
| **BUG-04-07** | 抽屉跨账号未清空，存在敏感凭据串扰风险 | `frontend/src/production/ProductionRenderer.tsx:4391-4463` | 抽屉挂载未绑定 `accountScopeGeneration` 密钥，换号登录后抽屉仍保留上一个账号的智能体配置与频道凭据草稿。 | **P1 安全与隔离漏洞** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 架构设计与单一权威归属

本蓝图确立 **唯一右侧上下文检查器（Canonical Context Inspector）** 架构。彻底废除 `OpenClawAskCopilotPanel` 和独立分散的抽屉，全部由位于 `frontend/src/recovered/features/conversation/workspace/context-inspector/` 的模块统一接管：

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   ProductionRenderer (Root Shell)                                      │
├───────────────────────────────┬──────────────────────────────────┬─────────────────────────────────────┤
│ 1. 一级左轨 (52px)            │ 2. 二级侧边栏 (260-400px)        │ 3. 三级主舞台与唯一上下文详情栏     │
│ (Primary Rail)                │ (Collection Filter & Dialogs)    │ Stage + ContextInspector (320-420px)│
└───────────────────────────────┴──────────────────────────────────┴─────────────────────────────────────┘
                                                                                      │
                                                                   ┌──────────────────┴──────────────────┐
                                                                   │       ContextInspector (唯一抽屉)   │
                                                                   ├─────────────────────────────────────┤
                                                                   │ 顶部状态栏: 头像/标题/关闭/Tab 胶囊 │
                                                                   ├─────────────────────────────────────┤
                                                                   │ 动态分面渲染 (根据 EntityType 动态过滤)│
                                                                   │ ├─ ProfileTab (资料/呼叫/星币/安全) │
                                                                   │ ├─ MembersTab (群成员/超级群管理)   │
                                                                   │ ├─ MediaTab (共享媒体/文件/链接/音频)│
                                                                   │ ├─ AgentSettingsTab (模型/设定/头像)│
                                                                   │ ├─ SkillsChannelsTab (MCP技能/渠道) │
                                                                   │ ├─ RoutinesTab (Cron自动化/触发器)  │
                                                                   │ └─ OutlineTasksTab (大纲/子任务跟踪)│
                                                                   └─────────────────────────────────────┘
```

### 3.2 模块拆分与 200–600 行预算明细表（Same Owner != Same File）

| 模块文件路径 | 预估行数 | 核心内聚职责与边界 | 替代的原有割裂组件 |
|---|---|---|---|
| `context-inspector/index.tsx` | 280–350 行 | 检查器主框架、拖拽宽度调整（320–420px）、响应式抽屉折叠、Tab 路由与无障碍焦点控制 | `OpenClawAskCopilotPanel.tsx`<br>`TdrpConversationOverlay.tsx`（抽屉部分） |
| `context-inspector/model.ts` | 180–240 行 | 检查器状态机、EntityType 判定规则、Tab 可见性计算、TypeScript 契约与事件接口 | 分散在各组件的 9 个状态变量 |
| `context-inspector/tabs/ProfileTab.tsx` | 220–300 行 | 统一资料展示卡片、在线状态指示、真实星币余额展示、音视频快捷呼叫、隐私拉黑与举报入口 | `TdrpProfileDetailsDrawer.tsx`（资料与星币部分） |
| `context-inspector/tabs/MembersTab.tsx` | 320–420 行 | 普通群与超级群成员检索、管理员权限分配、封禁/禁言（Mute/Ban）、添加成员弹窗与安全审计 | `group-members/view.tsx` |
| `context-inspector/tabs/MediaTab.tsx` | 280–360 行 | 共享多媒体（照片/视频/文档/链接/音频）分面虚拟列表、按月分组时间轴、大附件下载与导出 | 原 `TdrpProfileDetailsDrawer.tsx` 中的静态假 Tabs |
| `context-inspector/tabs/AgentSettingsTab.tsx` | 260–340 行 | 智能体名称/标题/系统提示词（System Prompt）、头像生成/裁剪、Provider 模型参数与思考强度选择 | `agent-info/settings/view.tsx`<br>`AvatarEditorView` |
| `context-inspector/tabs/SkillsChannelsTab.tsx` | 280–380 行 | OpenClaw 外部渠道（Telegram/Discord/Slack/WhatsApp）真实连接状态、MCP Tool 权限配置 | `agent-info/channels/view.tsx`<br>`OpenClawAskCopilotPanel.tsx`（网关诊断部分） |
| `context-inspector/tabs/RoutinesTab.tsx` | 280–360 行 | 会话关联的定时 Cron 自动化、执行计划编辑、手动运行触发、历史成功/失败日志 | `automations/routines/view.tsx` |
| `context-inspector/tabs/OutlineTasksTab.tsx` | 250–320 行 | 当前会话大纲（Turn 结构与工具执行摘要）与异步 7 态任务进度树（Subagents 执行追踪） | `conversation-outline-view.tsx`<br>`async-tasks/view.tsx` |


---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 渐进披露（Progressive Disclosure）与分层信息架构

为解决原 7 个抽屉要么信息匮乏（如仅有静态文本）、要么信息过载且布局冲突的问题，`ContextInspector` 建立四级渐进披露架构：

```
┌────────────────────────────────────────────────────────────────────────┐
│ Layer 1: 顶部状态栏与核心高频动作 (Top Hero & Quick Actions)            │
│  - 实体头像 (Avatar)、名称、在线状态徽章 (Online/Offline/Ready)        │
│  - 一键快捷动作栏: 语音呼叫 (Phone) / 视频呼叫 (Video) / 搜索 (Search) / │
│    群组语音房 (Group Call) / 静音通知 (Mute)                           │
│  - 官方星标余额徽章 (Stars Balance，接真实 ADR-002 账本)               │
├────────────────────────────────────────────────────────────────────────┤
│ Layer 2: 动态分面 Tab 胶囊选择器 (Contextual Pill Segmented Bar)       │
│  - 根据当前 EntityType (Human/Group/Channel/Agent/Hybrid) 动态呈现 Tabs:│
│    [资料 Profile] [成员 Members] [媒体 Media] [设置 Settings]          │
│    [技能与渠道 Skills] [自动化 Routines] [大纲任务 Outline]            │
├────────────────────────────────────────────────────────────────────────┤
│ Layer 3: 分面内容渲染与内联交互 (Tab Content & Inline Controls)        │
│  - Members: 虚拟成员列表、搜索过滤、管理员徽章、一键禁言/封禁/提权     │
│  - Media: 照片/视频/文件/链接/音频分面网格与按月时间轴聚合             │
│  - Settings: 提示词内联编辑、模型推理参数与思考强度滑块                │
│  - Routines: 定时 Cron 规则列表、一键触发执行、最新运行日志状态        │
│  - Skills: MCP 工具启停门禁、外部通讯渠道连接状态与引导向导           │
├────────────────────────────────────────────────────────────────────────┤
│ Layer 4: 沉浸式高级联动 (Deep Dives & Modals)                           │
│  - 媒体全屏查看器 (MediaViewer)、全量会话数据导出、2FA/云安全设置       │
└────────────────────────────────────────────────────────────────────────┘
```

#### 实体类型与动态 Tab 映射矩阵

| 实体类型 (`EntityType`) | 判定逻辑 (`activeAgent` 属性) | 可用 Tabs 集合 | 默认激活 Tab | 专属操作栏动作 |
|---|---|---|---|---|
| **Human（真人私聊）** | `isHuman === true && !isGroup` | `[profile, media, outline]` | `profile` | 语音呼叫、视频呼叫、搜索、赠送礼物、加密安全设置 |
| **Group（普通群组）** | `isGroup === true && !isChannel` | `[profile, members, media, outline]` | `profile` | 群组语音房、添加成员、搜索、群组管理与权限 |
| **Channel（广播频道）** | `isChannel === true` | `[profile, media, outline]` | `profile` | 频道静音、分享邀请链接、搜索、统计图表 |
| **Agent（智能体助手）** | `!isHuman && !isGroup` | `[profile, settings, skills, routines, outline, media]` | `profile` | 发起新会话、导出对话、调整思考强度、MCP工具管理 |
| **Hybrid（人机混合协同群）** | `isGroup === true && hasAgentMembers` | `[profile, members, skills, routines, media, outline]` | `profile` | 群组语音房、添加Bot/成员、工作流排期、群管理 |

---

### 4.2 响应式三阶布局断点与视口管理

严格遵循 `FSP-001 §17` 与 `openclaw-theme.css §7` 规范，实现无缝断点适配：

1. **宽屏工作区（视口宽度 $\ge 1200\text{px}$）**：
   - `ContextInspector` 作为三栏布局的第三列常驻挂载，宽度默认 340px，允许用户通过左边缘的拖拽手柄（Resize Handle）在 **320px 至 420px** 之间自由拖动调整，调整后的宽度持久化至 `localStorage`（键名受账户隔离保护）。
   - 主舞台消息流（Stage）自适应弹性压缩，虚拟滚动列表视口绝对稳定。
2. **中屏工作区（$900\text{px} \le \text{视口宽度} < 1200\text{px}$）**：
   - 抽屉默认处于收起状态（Collapsed）。
   - 用户点击顶栏详情按钮时，抽屉以 **平滑滑动动画（`transform: translateX(0)`，耗时 220ms，`cubic-bezier(0.16, 1, 0.3, 1)`）** 从视口右侧滑出，作为抽屉浮层（Drawer Overlay）覆盖于主舞台上方，背景附带半透明遮罩（`rgba(0, 0, 0, 0.32)`）。
   - 按 `Escape` 键或点击外部遮罩立即收起抽屉，并将焦点平滑归还至顶栏触发器。
3. **窄屏移动端（视口宽度 $< 900\text{px}$）**：
   - 抽屉展开时占据视口 100% 宽度（Full Stack Sheet），顶栏左侧呈现明显的“返回（Back Chevron）”图标按钮，点击直接退回会话主舞台。

---

### 4.3 语义设计系统变量（Design Tokens）与深浅色模式（WCAG AA）

彻底收口并绑定 `--fabushi-*` 官方语义 Token，100% 杜绝裸写十六进制色值与平行 `--oc-*` 类名：

```css
/* ContextInspector 样式规范映射 */
.fabushi-context-inspector {
  background-color: var(--fabushi-bg-surface, #ffffff);
  border-left: 1px solid var(--fabushi-border-weak, rgba(128, 128, 128, 0.15));
  color: var(--fabushi-text-primary, #1c1c1e);
}

.fabushi-inspector-hero__avatar {
  border: 2px solid var(--fabushi-border-subtle, rgba(0, 0, 0, 0.08));
  box-shadow: 0 4px 16px var(--fabushi-shadow-elevation-1, rgba(0, 0, 0, 0.06));
}

.fabushi-inspector-pill--active {
  background-color: var(--fabushi-accent-primary, #1084fe);
  color: var(--fabushi-text-on-accent, #ffffff);
}

.fabushi-inspector-stars-badge {
  background: linear-gradient(135deg, rgba(234, 179, 8, 0.14), rgba(249, 115, 22, 0.08));
  border: 1px solid var(--fabushi-badge-border-gold, rgba(234, 179, 8, 0.32));
  color: var(--fabushi-text-primary, #1c1c1e);
}
```
- **WCAG AA 达标规范**：所有文本标签对背景色对比度 $\ge 4.5:1$（次要文本 `--fabushi-text-muted` 对比度 $\ge 4.5:1$）；状态圆点（在线绿色 `#27AE60`、离线灰色 `#8E8E93`、错误红色 `#E74C3C`）附带 `aria-label` 与语义文本双重指示，绝不依赖纯色彩传意。

---

### 4.4 全键盘无障碍（A11y）与中文输入法（IME）保护

1. **ARIA 标注规范**：
   - 根容器声明 `role="complementary"` 与 `aria-label="会话与对象详情"`；
   - 标签栏声明 `role="tablist"`，每个胶囊声明 `role="tab"`, `aria-selected={activeTab === id}`, `aria-controls={`tabpanel-${id}`}`；
   - 内容面板声明 `role="tabpanel"`, `id={`tabpanel-${id}`}`, `aria-labelledby={`tab-${id}`}`。
2. **键盘导航闭环**：
   - 在 Tab 列表中支持键盘方向键左右（`ArrowLeft` / `ArrowRight`）无缝切换焦点；`Home` / `End` 快速跳转首尾 Tab；
   - 在中窄屏抽屉浮层打开时，激活焦点陷阱（Focus Trap），`Tab` 键循环限制在抽屉内部；按 `Escape` 立即关闭并恢复触发器焦点（Focus Restoration）。
3. **中文输入法（IME）保护**：
   - 在智能体提示词编辑、名称修改、成员搜索框中，监听 `onCompositionStart` 与 `onCompositionEnd`。
   - 当 `isComposing === true` 时，回车键（Enter）确认选词严格被拦截，绝不误触设置保存、表单提交或导致抽屉误关闭。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端核心 TypeScript 类型契约

```typescript
/** 实体分类枚举 */
export type InspectorEntityType = "human" | "group" | "channel" | "agent" | "hybrid";

/** 上下文检查器分面 Tab 标识 */
export type ContextInspectorTabId =
  | "profile"
  | "members"
  | "media"
  | "settings"
  | "skills"
  | "routines"
  | "outline";

/** 共享媒体分片类型 (对标 Telegram TCG-N19/TCG-N20) */
export type SharedMediaSection = "photos" | "videos" | "files" | "links" | "audio" | "voice";

/** 统一上下文检查器根 Props */
export interface ContextInspectorProps {
  readonly isOpen: boolean;
  readonly onClose: () => void;
  readonly activeAgent: {
    readonly id: string;
    readonly name: string;
    readonly title?: string;
    readonly description?: string;
    readonly isHuman?: boolean;
    readonly isGroup?: boolean;
    readonly isChannel?: boolean;
    readonly raw?: Record<string, unknown>;
  };
  readonly accountScopeGeneration: number;
  readonly client?: ProductionCoordinatorClient | null;
  readonly starsBalance?: number;
  readonly onStartCall: (isVideo: boolean) => void;
  readonly onStartGroupCall: () => void;
  readonly onOpenSearch: () => void;
  readonly onOpenExportModal: () => void;
  readonly onOpenSecurityModal: () => void;
  readonly onOpenStarsModal: () => void;
  readonly initialTab?: ContextInspectorTabId;
}

/** 分面 Tab 状态契约 */
export interface InspectorTabDescriptor {
  readonly id: ContextInspectorTabId;
  readonly label: string;
  readonly icon: React.ComponentType<{ size?: number; className?: string }>;
  readonly badgeCount?: number;
  readonly isVisible: (entityType: InspectorEntityType) => boolean;
}
```

```rust
// Rust Host 侧 Context Inspector 聚合快照与成员权限更新契约 (source/host/src/extensions/session/production.rs)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InspectorPeerKind {
    HumanDirect,
    Supergroup,
    BroadcastChannel,
    AutonomousAgent,
    HybridRoom,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextInspectorSnapshot {
    pub peer_id: String,
    pub peer_kind: InspectorPeerKind,
    pub account_scope_generation: u64,
    pub stars_balance: i64,
    pub shared_media_counts: std::collections::BTreeMap<String, u32>,
    pub active_channel_bindings: Vec<String>,
    pub enabled_skill_ids: Vec<String>,
}
```

---

### 5.2 状态转移方程与降级策略

检查器状态转移由以下确定性有限状态机（FSM）驱动：

$$\mathcal{M}_{\text{inspector}} = \langle S, \Sigma, \delta, s_0 \rangle$$

- **状态空间 $S$**：$\{\text{Closed}, \text{Open}(\text{entityId}, \text{tabId})\}$
- **输入事件 $\Sigma$**：
  1. $\text{OPEN\_INSPECTOR}(\text{targetAgent}, \text{preferredTab}?)$
  2. $\text{CLOSE\_INSPECTOR}$
  3. $\text{SWITCH\_TAB}(\text{newTabId})$
  4. $\text{CONVERSATION\_CHANGED}(\text{newAgent})$
  5. $\text{ACCOUNT\_LOGOUT\_OR\_SWITCH}(\text{newGeneration})$
- **状态转移函数 $\delta$**：
  - $\delta(\text{Closed}, \text{OPEN\_INSPECTOR}(A, T)) \to \text{Open}(A.\text{id}, \text{ClampTab}(A, T))$
  - $\delta(\text{Open}(A, T), \text{SWITCH\_TAB}(T')) \to \text{Open}(A, T')$（若 $T' \in \text{VisibleTabs}(A)$）
  - $\delta(\text{Open}(A, T), \text{CONVERSATION\_CHANGED}(A')) \to \text{Open}(A'.\text{id}, \text{SanitizeTab}(A', T))$
  - $\delta(\text{Open}(A, T), \text{CLOSE\_INSPECTOR}) \to \text{Closed}$
  - $\delta(S, \text{ACCOUNT\_LOGOUT\_OR\_SWITCH}) \to \text{Closed}$（清空所有挂起状态）

其中 Tab 净化函数 $\text{SanitizeTab}(A', T)$ 定义为：
$$\text{SanitizeTab}(A', T) = \begin{cases} T & \text{if } T \in \text{VisibleTabs}(\text{EntityType}(A')) \\ \text{"profile"} & \text{otherwise} \end{cases}$$
确保用户从拥有 `settings` Tab 的智能体会话切换到真人私聊时，**绝不崩溃或显示空白面板，而是平滑自愈回退到 `profile` Tab**。

---

### 5.3 跨账号、跨会话与并发 Fencing 围栏

1. **`accountScopeGeneration` 单调递增安全隔离**：
   - 当用户注销、切换登录账号或会话凭证失效时，系统派发递增的 `accountScopeGeneration`。
   - `ContextInspector` 根容器监测到该世代标识变更时，立即物理销毁各子 Tab（`MembersTab`, `SkillsChannelsTab`, `RoutinesTab`）的本地内存快照与输入草稿。
   - 所有在途的异步查询（如群成员列表拉取、MCP 技能状态检测、频道 Webhook 探测）携带发出时的 `generation`。当响应返回时若 `req.generation !== currentGeneration`，立即静默丢弃（Fail-closed Drop），防止旧账号的敏感联系人或自动化任务串入新账号。
2. **`activeAgent.id` 会话级 Fencing**：
   - 切换当前会话时，抽屉内所有基于会话粒度的订阅（如会话大纲流、子代理异步任务监听）立即取消前向订阅（Unsubscribe），新会话加载完成前呈现受控骨架屏（Skeleton），杜绝内容闪烁和旧会话数据残留。
3. **真实 ADR-002 星币余额与礼物账本流转**：
   - 彻底废除 `starsBalance = 1250` 硬编码。通过 `CoordinatorClient.getCreditsSnapshot()` 订阅权威账本。
   - 礼物赠送与星币消费使用 CAS（Compare-And-Swap）乐观锁，网络抖动或并发赠送时服务端原子仲裁，余额不足时给出精准错误提示，杜绝本地虚假成功回显。


---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 实施切片拆解与派工清单（Work Slices）

为确保后续子智能体（Worker Subagents）在落地修改代码时高效并发且无代码冲突，本蓝图将 `ContextInspector` 的落地拆分为 5 个相互解耦的执行切片：

```
┌────────────────────────────────────────────────────────────────────────┐
│ 切片 04-A: 基础契约与主框架容器 (Model & Shell Frame)                   │
│   - 创建 context-inspector/model.ts 与 context-inspector/index.tsx      │
│   - 搭建 320–420px 宽度拖拽骨架、三阶响应式断点与 Tab 胶囊导航条       │
├────────────────────────────────────────────────────────────────────────┤
│ 切片 04-B: 身份资料、真实账本与多媒体分面 (Profile & Media Tabs)        │
│   - 创建 tabs/ProfileTab.tsx 与 tabs/MediaTab.tsx                       │
│   - 接入真实 ADR-002 星币余额订阅，实现呼叫/搜索/全量导出快捷动作       │
│   - 实现共享媒体按月时间轴与照片/视频/文档/链接分片虚拟列表             │
├────────────────────────────────────────────────────────────────────────┤
│ 切片 04-C: 超级群管理与智能体模型设定 (Members & Agent Settings)       │
│   - 创建 tabs/MembersTab.tsx 与 tabs/AgentSettingsTab.tsx               │
│   - 升级群成员列表，集成 TCG-N15 管理员任命、禁言（Mute）与封禁权限     │
│   - 整合名称、提示词（System Prompt）、思考强度滑块与头像编辑器        │
├────────────────────────────────────────────────────────────────────────┤
│ 切片 04-D: 技能门禁、Cron 自动化与会话大纲 (Skills, Routines & Outline)│
│   - 创建 tabs/SkillsChannelsTab.tsx, RoutinesTab.tsx, OutlineTasksTab.tsx│
│   - 接入真实外部渠道探测（Telegram/Discord）、MCP 工具授权门禁          │
│   - 嵌入会话级别 Cron 调度列表、执行大纲与后台 7 态异步任务追踪树       │
├────────────────────────────────────────────────────────────────────────┤
│ 切片 04-E: 生产壳窄接线与旧抽屉收口 (ProductionRenderer Narrow Wiring) │
│   - 在 ProductionRenderer.tsx 中移除 9 个分散状态，接入单一状态机       │
│   - 彻底废除 OpenClawAskCopilotPanel 并收口 TdrpConversationOverlay    │
└────────────────────────────────────────────────────────────────────────┘
```

---

### 6.2 文件修改白名单与只读参考范围

严格限定子智能体的写操作权限，防止跨界破坏其他模块：

#### 允许修改的文件白名单（Writable Whitelist）
1. `frontend/src/recovered/features/conversation/workspace/context-inspector/**`（新建单一权威目录全部文件）
2. `frontend/src/production/openclaw/OpenClawAskCopilotPanel.tsx`（清空其假在线/假连接器逻辑与草稿覆写，标记为 `@deprecated` 并重定向至 `ContextInspector`）
3. `frontend/src/production/tdrp/TdrpProfileDetailsDrawer.tsx`（清空假数据，委托至 `ContextInspector` 的 `ProfileTab`）
4. `frontend/src/production/tdrp/TdrpConversationOverlay.tsx`（移除脱离上下文的孤立抽屉包装代码）
5. `frontend/src/production/ProductionRenderer.tsx`（仅限执行窄接线：用单一 `<ContextInspector ... />` 替换原有分散在 4319–4463 行的 7 个独立抽屉与浮层挂载）

#### 只读参考范围（Strictly Read-Only References）
- `source/host/**`（Rust 宿主服务与 SandAgentDb 存储层）
- `frontend/src/recovered/features/agent-info/**`（旧有组件逻辑参考，禁止在原目录新增散落文件）
- `frontend/src/recovered/features/automations/routines/**`（Controller 与 RunHistory 协议参考）
- `frontend/src/recovered/features/conversation/workspace/conversation-outline-provider.ts`（大纲数据源参考）
- `frontend/src/recovered/features/agent-info/async-tasks/provider.ts`（异步任务快照参考）
- `contracts/**`（桌面 Bridge 契约与跨平台规范）

---

### 6.3 窄接线架构纪律（Narrow Wiring Disciplines）

在更新 `ProductionRenderer.tsx` 时，子智能体必须遵循三项铁律：
1. **单一状态收敛**：将原有的 `copilotPanelOpen`, `tdrpDetailsOpen`, `agentSettingsOpen`, `groupInfoPaneOpen`, `routinesInfoPaneOpen`, `channelsInfoPaneOpen`, `runtimeDetailPaneOpen`, `conversationOutlineAgentId`, `asyncTasksAgentId` 9 个分散状态，统一收敛为：
   ```typescript
   const [inspectorState, setInspectorState] = useState<{
     isOpen: boolean;
     initialTab?: ContextInspectorTabId;
   }>({ isOpen: false });
   ```
2. **零内联业务逻辑**：`ProductionRenderer.tsx` 仅作为组装插座，传递 `activeAgent`, `accountScopeGeneration`, `client` 等已有上下文，不得在 `ProductionRenderer.tsx` 中编写超过 20 行的抽屉内部事件处理逻辑。
3. **保持单向数据流**：抽屉内部状态（如当前激活的 Tab、选中的媒体过滤条件、成员搜索关键词）由 `ContextInspector` 内部状态机自管理，不向父级 `ProductionRenderer` 溢出。

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

为保障融合质量，独立审计子智能体（Auditor/Reviewer）在审查实施代码时，必须执行以下正反例对照核验：

### 7.1 绝对禁止的反模式检查清单（Negative Anti-Patterns — Fail-Closed）

- [ ] **严禁假定时器逻辑**：源码中不得出现任何使用 `setTimeout` 或 `setInterval` 模拟异步响应、虚假发送或假装网络就绪的逻辑。
- [ ] **严禁硬编码伪造在线状态**：严禁出现 `isGatewayOnline={false}` 或假称“18 个连接器全量就绪”的静态文本。未连接真实网关时必须如实显示“未连接 (未知)”。
- [ ] **严禁破坏用户输入台草稿**：严禁在点击快捷操作（如 Pill）时调用 `composerDraftStore.setDraft` 清空或覆盖用户正在主输入框键入的文字与附件。
- [ ] **严禁硬编码假星标余额**：严禁在资料面板中写死 `starsBalance = 1250` 等假数值，必须订阅权威 ADR-002 账本数据源或在无数据时展示优雅空状态。
- [ ] **严禁实例化第二个聊天输入框**：`ContextInspector` 内部绝对不允许渲染独立的 `<textarea>` 或并行的聊天输入台，所有消息交互必须通过中央唯一的 `ConversationComposer` 进行。
- [ ] **严禁跨账号敏感数据残留**：在递增 `accountScopeGeneration` 时，抽屉必须彻底重置，严禁显示前一账号的群成员、提示词草稿或私密渠道凭据。
- [ ] **严禁新增平行全局样式系统**：严禁新写带有 `--oc-*` 前缀的全局平行变量或未封装的十六进制硬编码色彩，必须 100% 消费 `--fabushi-*` 设计变量。

---

### 7.2 必须达成的正例验收标准（Positive Acceptance Gates）

- [ ] **单一抽屉统一性**：原 7 个分散竞争抽屉（`OpenClawAskCopilotPanel`、`TdrpProfileDetailsDrawer`、`GroupMembersPane`、`AgentSettingsPanel`、`mountRoutinesInfoPane`、`mountAgentInfoChannels`、`FabushiAgentRuntimeDetailSections`）100% 收口至唯一 `ContextInspector`。
- [ ] **动态 Tab 智能自愈**：
  - 针对真人（Human），正确呈现 `Profile`, `Media`, `Outline`；
  - 针对群组（Group），正确呈现 `Profile`, `Members`, `Media`, `Outline`；
  - 针对智能体（Agent），正确呈现 `Profile`, `Settings`, `Skills`, `Routines`, `Outline`, `Media`；
  - 跨会话切换时，若旧 Tab 在新会话中不可见，平滑自动降级为 `Profile` Tab，无报错无白屏。
- [ ] **超级群权限闭环**：`MembersTab` 完整支持群成员检索、移除、管理员任命、禁言（Mute）与封禁（Ban）动作，且权限操作具备 fail-closed 保护。
- [ ] **响应式自适应断点达标**：
  - $\ge 1200\text{px}$：右侧常驻展开，支持 320–420px 鼠标拖拽平滑调宽；
  - $900\text{–}1199\text{px}$：作为浮层抽屉（Drawer Overlay）滑出，附带遮罩与 Escape 一键退出；
  - $< 900\text{px}$：全屏堆叠自适应，提供清晰返回按钮。
- [ ] **无障碍（A11y）与输入法保护**：
  - 根容器包含 `role="complementary"` 与语义 ARIA 标签；
  - 浮层状态下支持焦点循环（Focus Trap）并在关闭后恢复顶栏触发器焦点；
  - 输入框支持中文输入法（IME）`isComposing` 组合状态保护，按 Enter 键选词绝不触发保存或面板关闭。
- [ ] **单文件内聚预算合规**：`context-inspector/` 目录下每个新建实现文件代码行数严格控制在 **200–600 行** 之间，严格践行 `Same Owner != Same File` 原则。

---

### 7.3 独立审计核验流程（Auditor Verification Protocol）

```
┌────────────────────────────────────────────────────────────────────────┐
│ 第一步：静态引用链审查 (Static Import Graph Audit)                      │
│   - 核查 ProductionRenderer.tsx 确认仅挂载单一 ContextInspector        │
│   - 确认无废弃抽屉的残留 import 与多余 boolean state                   │
├────────────────────────────────────────────────────────────────────────┤
│ 第二步：代码反模式扫描 (Anti-Pattern Regex Scan)                        │
│   - 扫描 context-inspector/ 目录，确认无 setTimeout / setInterval 假回复 │
│   - 扫描确认无硬编码 1250 或 isGatewayOnline=false 假逻辑              │
│   - 确认无 composerDraftStore.setDraft 强行覆写草稿的调用              │
├────────────────────────────────────────────────────────────────────────┤
│ 第三步：类型与接口完整性校验 (TypeScript Contract Check)                │
│   - 核验 ContextInspectorProps 属性与 ProductionRenderer 传递一致性    │
│   - 验证 EntityType 与 Tab 可见性状态转移矩阵的确定性                  │
├────────────────────────────────────────────────────────────────────────┤
│ 第四步：设计变量与无障碍审查 (Design Tokens & A11y Audit)               │
│   - 校验 CSS 变量全部遵循 --fabushi-*，无直接 oc-* 类名依赖            │
│   - 验证 ARIA 角色、Tablist 结构与 IME isComposing 保护代码            │
└────────────────────────────────────────────────────────────────────────┘
```
