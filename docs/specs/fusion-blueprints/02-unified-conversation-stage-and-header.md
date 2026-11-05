# Fabushi 统一融合蓝图 02：统一会话舞台、单一顶栏与消息流
# Volume 02: Unified Conversation Stage, Single Top Bar, and Transcript Stream

> **规范编号**: `FBCP-BLUEPRINT-02` / `UFX-BLUEPRINT-02`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. **根除 `activeIsHuman` 二元硬编码分化**：彻底消灭 `ProductionRenderer.tsx:4212-4239` 中的条件分叉，真人、智能体、群组、频道与人机混合群共用单一权威顶栏（`UnifiedConversationHeader`）与统一会话舞台（`UnifiedConversationStage`）。
> 2. **彻底解耦“会话分享”与“消息转发”**：废除历史遗留中误将“转发末条消息”当成“分享会话”的严重错误与隐患，规范诚实分享邀请/快照链接与基于 `TdrpForwardMessageModal` 的精准选条/批量消息转发。
> 3. **收敛顶栏静态与装饰性控件**：清除 `OpenClawTopHeader.tsx` 中无真实历史栈支撑的静态前进/后退箭头，整合通话控制、群组管理、人机接力（Handoff）与更多操作菜单。
> 4. **无缝集成论坛话题（Forum Topics）与置顶横幅（Pinned Banners）**：将 `ForumTopicSwitcher` 与 `ConversationPinnedBanner` 深度嵌入主消息流视口，严格遵守话题消息流数据隔离围栏（`INV-TDRP-FORUM-TOPICS-THREAD-ISOLATION-001`）。
> 5. **筑牢虚拟化消息流（ConversationTranscript）核心能力**：保持流式生成、富文本（Markdown/Math/Mermaid）、富媒体（Media Gallery）、交互卡片（A2UI）、表态（Reactions）、引用回复（Quote & Reply）、投递结算（Delivery Settlement）与空洞补齐（Gap Recovery）的零遗漏交付。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **统一会话主舞台**<br>(Unified Stage) | `ui/src/pages/chat/chat-stage.ts`<br>`frontend/src/production/openclaw/OpenClawChatStage.tsx:1-148` | `Telegram/SourceFiles/history/history_widget.cpp:1-520`<br>`window/window_history_hider.cpp` | `frontend/src/production/ProductionRenderer.tsx:4209-4335`<br>`frontend/src/production/openclaw/OpenClawChatStage.tsx` | `UnifiedConversationStage.tsx`<br>(唯一权威舞台容器) |
| **统一会话顶栏**<br>(Unified Header) | `ui/src/pages/chat/session-header.ts`<br>`frontend/src/production/openclaw/OpenClawTopHeader.tsx:1-323` | `history/view/history_view_top_bar_widget.cpp:115-466`<br>`Telegram/SourceFiles/top_bar_widget.cpp` | `frontend/src/production/ProductionRenderer.tsx:4212-4239`<br>`frontend/src/recovered/features/conversation/workspace/chat-header.tsx:1-40` | `UnifiedConversationHeader.tsx`<br>(合并消除 activeIsHuman) |
| **会话诚实分享与快照**<br>(Honest Share Link & Snapshot) | `src/snapshot/`<br>`src/sessions/session-share.ts`<br>`extensions/beam/` | `boxes/share_box.cpp`<br>`window/window_peer_menu.cpp:120-180` | `frontend/src/production/openclaw/OpenClawTopHeader.tsx:148-161`<br>`ProductionRenderer.tsx:4232-4237` | `ConversationShareService`<br>`TdrpShareSessionModal.tsx` |
| **单条/批量消息转发**<br>(Message Forwarding & Provenance) | `src/interactive/a2ui.ts`<br>`packages/session-contract/` | `ui/chat/forward_options_box.cpp:1-85`<br>`history/view/controls/history_view_forward_panel.cpp:1-120`<br>`chat_helpers/share_message_phrase_factory.cpp` | `frontend/src/production/tdrp/TdrpForwardMessageModal.tsx:1-51`<br>`frontend/src/production/ForwardMessageDialog.tsx:1-240`<br>`ProductionRenderer.tsx:4268-4273` | `TdrpForwardMessageModal.tsx`<br>`ForwardMessageDialog.tsx` |
| **论坛话题切换与隔离**<br>(Forum Topics Switcher) | `src/sessions/topic-tree.ts`<br>`packages/agent-core/` | `Telegram/SourceFiles/data/data_forum_topic.cpp:1-210`<br>`data/data_thread.cpp` | `frontend/src/production/tdrp/ForumTopicSwitcher.tsx:1-41`<br>`ProductionRenderer.tsx:4242-4246`<br>`source/node-agent-coordinator/src/tdrp/forum_topics.rs` | `ForumTopicSwitcher.tsx`<br>(挂载于顶栏下方次级导轨) |
| **会话置顶横幅与导航**<br>(Pinned Messages Banner) | `src/sessions/pinned-items.ts` | `history/view/controls/history_view_pinned_bar.cpp:1-190`<br>`history/history_widget.cpp:890-950` | `frontend/src/recovered/features/conversation/workspace/conversation-pinned-banner.tsx`<br>`ProductionRenderer.tsx:4247-4250` | `ConversationPinnedBanner.tsx`<br>(多置顶轮播与平滑跳转) |
| **会话内高亮检索**<br>(Find In Chat Controller) | `ui/src/components/search/`<br>`packages/search-core/` | `history/history_widget.cpp:320-380`<br>`history/view/controls/history_view_search_bar.cpp` | `frontend/src/recovered/features/conversation/workspace/find-in-chat-bar.tsx`<br>`frontend/src/recovered/features/conversation/workspace/find-in-chat-controller.ts`<br>`ProductionRenderer.tsx:4241` | `FindInChatBar.tsx`<br>`find-in-chat-controller.ts` |
| **虚拟化消息流底座**<br>(Transcript Virtual Feed) | `ui/src/pages/chat/transcript-view.ts`<br>`packages/markdown-core/` | `history/view/history_view_list_widget.cpp:1-450`<br>`history/history_item.cpp` | `frontend/src/recovered/features/conversation/workspace/transcript.tsx:1-886`<br>`ProductionRenderer.tsx:4255-4292` | `ConversationTranscript.tsx`<br>(唯一权威高性能消息列表) |
| **消息表情表态与参与者**<br>(Message Reactions Pill) | `src/interactive/reactions.ts` | `Telegram/SourceFiles/data/data_message_reactions.cpp`<br>`Telegram/SourceFiles/api/api_who_reacted.cpp` | `frontend/src/production/reaction-root.ts`<br>`frontend/src/recovered/features/conversation/cards/transcript-card/reaction-actions.ts`<br>`transcript.tsx:53-65,4277-4279` | `reaction-actions.ts`<br>`TranscriptMessageReactionPills` |
| **富媒体与结构化卡片渲染**<br>(Cards, Media, A2UI, Math) | `src/canvas/`<br>`packages/mermaid-renderer/`<br>`packages/media-core/` | `history/history_item_components.cpp`<br>`media/view/media_view_overlay_widget.cpp` | `frontend/src/recovered/features/conversation/workspace/mermaid.tsx`<br>`frontend/src/recovered/features/conversation/workspace/math.tsx`<br>`frontend/src/production/tdrp/TdrpInteractiveMediaCards.tsx` | `TdrpInteractiveMediaCards.tsx`<br>`TranscriptCardRootEntry` |
| **消息发送结算与空洞补洞**<br>(Send Settlement & Gap Repair) | `src/process/worker-state.ts`<br>`packages/retry/` | `Telegram/SourceFiles/api/api_sending.cpp`<br>`Telegram/SourceFiles/data/data_history_messages.cpp`<br>`Telegram/SourceFiles/api/api_updates.cpp` | `frontend/src/recovered/features/conversation/workspace/pagination.ts`<br>`source/host/src/extensions/session/production.rs`<br>`source/host/src/extensions/session/agent_db_transcript_pages.rs` | `ProductionSessionWorkers`<br>`transcriptPaginationSnapshot` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - 顶栏由 `history/view/history_view_top_bar_widget.cpp` 管理，采用单一组件自适应承载用户（`User`）、群组（`Chat`/`Channel`）与机器人（`Bot`）。顶栏具备头像、在线/成员状态、呼叫按键（`call` 306–314）、会话检索、置顶条折叠与弹出操作菜单。
  - 转发消息与会话分享严格物理分离：`ui/chat/forward_options_box.cpp` 与 `history/view/controls/history_view_forward_panel.cpp` 控制转发消息隐私（`drop_author`, `drop_media_captions`），而 `boxes/share_box.cpp` 仅负责群组/频道公开链接复制与邀请。
  - 话题由 `data/data_forum_topic.cpp` 维护，具有独立的话题 ID（`TopicId`）、未读角标、关闭状态（`closed`）与置顶状态（`pinned`）。
- **OpenClaw 侧**：
  - 会话视口由 `ui/src/pages/chat/chat-stage.ts` 呈现，包含顶部 `session-header.ts`、主视口、以及底部输入台；支持空会话时的 `WelcomeHero` 引导提示词（Starter Prompts）；支持实时流式 Markdown 渲染、Diff 视图与工具执行审批通知。

#### Q2（现有链路定位）
- `ProductionRenderer.tsx:4209-4335` 挂载主视口。
- 顶栏逻辑：在第 4212 行执行分支：
  `activeIsHuman ? <div className="sand-chat-header">...</div> : <OpenClawTopHeader .../>`。
- 话题与置顶条：紧接顶栏下方，第 4242 行渲染 `ForumTopicSwitcher`，第 4247 行渲染 `ConversationPinnedBanner`。
- 消息流：第 4255 行渲染 `ConversationTranscript`，承接 `displayedEntries` 并绑定视口事件。

#### Q3（断点精确诊断）
1. **`activeIsHuman` 二元撕裂断点**（`ProductionRenderer.tsx:4212-4239`）：
   - 当 `activeIsHuman === true` 时，强制渲染极简的 `<div className="sand-chat-header">`，丢弃了 `AgentAvatar` 丰富的动态状态、丢弃了重命名/导出/清除历史菜单、丢弃了详情抽屉触发入口；当 `activeIsHuman === false` 时，渲染 `OpenClawTopHeader`，但却缺失了通话入口（`HumanCallControls`）与群管理入口（`TdrpCallAndAdminDrawer`）。
2. **分享会话误转发末条消息及静态 URL 复制断点**（`ProductionRenderer.tsx:4232-4237` 与历史 `OpenClawTopHeader`）：
   - 历史版本在 `OpenClawTopHeader` 中点击分享时错误抓取最后一条消息并调用转发；在修复后则退化为简单 `navigator.clipboard.writeText(shareLink)`，当缺少真实后端 inviteLink 时甚至直接复制无意义的 `window.location.href`。
3. **顶栏无回调假箭头断点**（`OpenClawTopHeader.tsx:99-126`）：
   - 包含 `<` 与 `>` 按钮，当父级未提供真实历史栈路由时沦为静止占位符；缺乏统一的面包屑或上下文指示。
4. **话题隔离防穿透断点**（`ForumTopicSwitcher.tsx:15-38` 与 `ProductionRenderer.tsx:4256`）：
   - `ForumTopicSwitcher` 仅做前端活跃话题 ID 切换，若后端或缓存未建立 `activeTopicId` 的查询隔离围栏，易导致跨话题消息在同一视口串扰。
5. **欢迎屏排斥真人会话断点**（`ProductionRenderer.tsx:4253`）：
   - `displayedEntries.length === 0 && !activeIsHuman` 导致真人新会话只有光秃秃的空白屏幕，缺少联系人信息卡与发起打招呼建议。

#### Q4（状态机与安全防护）
- **多维安全围栏复合键**：
  $$\text{StageKey} = (\text{accountScopeGeneration}, \text{transcriptAccountSlot}, \text{activeConversationId}, \text{activeTopicId})$$
  切换账号、会话或话题时，触发复合键刷新，原子重置视口滚动位置与历史消息缓存，丢弃上一个上下文的所有在途流式 chunk 与迟到异步响应。
- **转发隐私状态转移方程**：
  $$\text{ForwardPrivacy} = (\text{dropSenderNames}: \text{boolean}, \text{dropCaptions}: \text{boolean})$$
  $$\text{dropCaptions} = \text{true} \implies \text{dropSenderNames} = \text{true}$$
  严禁跨越账号边界（`accountScopeGeneration` 不一致）执行消息转发；任何目标会话提交必须在 SandAgentDb 中写入唯一的幂等 `HumanForwardBatch` 日志记录。

#### Q5（UI/UX 极致融合落点）
- **统一顶栏（Unified Top Header）**：固定高度 56px，采用 `--fabushi-bg-surface` 与 `--fabushi-border-subtle` 底边框。
  - 左侧（Leading）：会话对象头像（`AgentAvatar` 统一支持真人、群组、频道、智能体徽标与在途打字动画）、会话主标题（600 权重）与副标题（在线状态、群成员数、执行中状态或渠道来源）。
  - 中间（Context Affordance）：展示论坛话题面包屑或多置顶切换指示器。
  - 右侧（Actions Dock）：按对象属性动态聚拢动作：音视频通话按钮、搜索按钮、分享按钮、更多下拉菜单（重命名、导出、清空）、右侧上下文详情触发按钮（带选中态高亮）。
- **折叠式次级条（Collapsible Sub-Bars）**：
  - 话题切换条（`ForumTopicSwitcher`）采用流式水平滚动的 Pill 标签；
  - 置顶横幅（`ConversationPinnedBanner`）高度 36px，左侧置顶针图标，中间截断预览，右侧多置顶序列指示与快速展开列表。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-02-01** | `activeIsHuman` 硬编码分支导致顶栏二元分化与功能互斥 | `frontend/src/production/ProductionRenderer.tsx:4212-4239` | 早期由不同团队分别对接真人聊天与 Agent 对话，未抽象统一领域模型，导致两类会话在顶栏、详情入口、通话控件上产生人为鸿沟。 | **P0 架构割裂** |
| **BUG-02-02** | 会话“分享”混淆消息“转发”，退化复制无效 `location.href` | `frontend/src/production/openclaw/OpenClawTopHeader.tsx:148-161`<br>`ProductionRenderer.tsx:4232-4237` | 概念混淆。把单条消息的引用转发（Provenance Forward）与公开快照/邀请分享（Link Share）混为一个入口，导致早期错误转发私密末条，后修复又草率降级为 URL 复制。 | **P0 隐私/交互缺陷** |
| **BUG-02-03** | 顶栏存在无动作绑定的静态/伪控制元素 | `frontend/src/production/openclaw/OpenClawTopHeader.tsx:99-126` | 原型期照抄浏览器前进后退 UI，在无主进程历史栈支持时展示无效箭头，破坏用户预期。 | **P2 体验缺陷** |
| **BUG-02-04** | 空会话引导（WelcomeHero）仅限 Agent，真人新对话展现空白 | `frontend/src/production/ProductionRenderer.tsx:4253-4254` | 空状态视图未与统一对象模型对齐，缺乏真人/群组维度的名片展示与打招呼建议模板。 | **P1 视觉/引导缺陷** |
| **BUG-02-05** | 话题切换器与消息视口之间缺少深层单调性 Fencing | `frontend/src/production/tdrp/ForumTopicSwitcher.tsx:1-41`<br>`ProductionRenderer.tsx:4242-4246` | 话题切换仅修改了 React 状态，若网络发生乱序分页加载，旧话题的消息可能插入新话题列表中。 | **P1 数据竞态** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 统一架构设计与领域分层

为彻底终结 `activeIsHuman` 分化，本蓝图确立“**单一舞台、统一顶栏、自适应动作槽、隔离消息流**”的架构原则：

```
+---------------------------------------------------------------------------------------------------+
| UnifiedConversationStage (frontend/src/production/conversation/UnifiedConversationStage.tsx)       |
|                                                                                                   |
|  +---------------------------------------------------------------------------------------------+  |
|  | UnifiedConversationHeader (frontend/src/production/conversation/UnifiedConversationHeader.tsx) |  |
|  | [Avatar] [Title + Subtitle/Presence]   | [Search] [Call] [Handoff] [Share] [More] [Details]  |  |
|  +---------------------------------------------------------------------------------------------+  |
|  | [Sub-Bar 1: Collapsible ForumTopicSwitcher] (frontend/src/production/tdrp/ForumTopicSwitcher)  |  |
|  | [Sub-Bar 2: Collapsible ConversationPinnedBanner] (conversation-pinned-banner.tsx)          |  |
|  | [Sub-Bar 3: FindInChatBar] (find-in-chat-bar.tsx)                                           |  |
|  +---------------------------------------------------------------------------------------------+  |
|  | Main Content Feed Area:                                                                      |  |
|  |   - If empty: UnifiedWelcomeHero (adaptive: Human Profile Card vs Agent Starter Prompts)       |  |
|  |   - If non-empty: ConversationTranscript (frontend/src/recovered/features/.../transcript.tsx)  |  |
|  +---------------------------------------------------------------------------------------------+  |
|  | Unified Composer Dock Area (蓝图 03 Canonical Owner: ConversationComposer)                    |  |
+---------------------------------------------------------------------------------------------------+
```

### 3.2 Canonical Owner 字典与职责映射表

1. **统一会话舞台容器（`UnifiedConversationStage`）**：
   - 目标路径：`frontend/src/production/conversation/UnifiedConversationStage.tsx`（预算 300–450 行）
   - 职责：统一承载顶栏、次级辅助栏、主内容流视口与底部输入底座；统管视口 Resize、Fencing 键重置与全屏/分屏布局。
2. **统一会话顶栏（`UnifiedConversationHeader`）**：
   - 目标路径：`frontend/src/production/conversation/UnifiedConversationHeader.tsx`（预算 350–500 行）
   - 职责：取代原 `OpenClawTopHeader` 与 `chat-header.tsx`。依据会话对象类型（`human | agent | group | channel | hybrid`）自适应装配动作按钮。
3. **单条/批量消息转发模态窗（`TdrpForwardMessageModal`）**：
   - 既有路径：`frontend/src/production/tdrp/TdrpForwardMessageModal.tsx` 结合 `ForwardMessageDialog.tsx`
   - 职责：承接来自消息操作菜单的单条或批量转发，展示接收人过滤搜索、转发隐私切换与持久化投递。
4. **会话诚实分享模态窗（`TdrpShareSessionModal`）**：
   - 目标路径：`frontend/src/production/tdrp/TdrpShareSessionModal.tsx`（预算 200–300 行）
   - 职责：专属管理会话邀请链接复制、只读快照生成与二维码导出，严禁触发任何消息发送动作。
5. **高性能虚拟化消息流（`ConversationTranscript`）**：
   - 既有路径：`frontend/src/recovered/features/conversation/workspace/transcript.tsx`
   - 职责：唯一权威消息流展示器，支持全部 11 种卡片与多模态渲染，严格遵循 Fencing 隔离。


---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 统一顶栏信息架构与渐进披露（Progressive Disclosure）

统一顶栏（`UnifiedConversationHeader`）固定高度为 56px，采用三段式弹性 Flexbox 布局，遵循清晰的视觉层级与渐进披露设计：

```
+-------------------------------------------------------------------------------------------------------------+
| [Leading: 40px Avatar + Title + Status]        [Center: Topic/Pin Indicator]        [Trailing: Actions Dock]|
|  (O) Alice Wang                      Active now                                        [📞] [🔍] [👥] [⋯] [ℹ️] |
+-------------------------------------------------------------------------------------------------------------+
```

1. **左侧标识区（Leading Section）**：
   - **头像与动态指示符（Avatar & Activity Ring）**：尺寸固定 36x36px。复用 canonical `AgentAvatar`，基于 `kind`（`human | agent | group | shared-room | channel`）动态渲染：
     - 真人（Human）：展示用户头像或姓名双字母 Monogram，右下角展示精准在线绿点或灰色离线状态；
     - 智能体（Agent）：展示 Agent 专属形状与颜色，当后台处于思考/执行（`isRunning`）或打字中（`isComposingMessage`）时，外圈环绕 CSS 脉冲光环（`var(--fabushi-accent-primary)`）；
     - 群组/频道（Group/Channel）：展示群组多重叠头像或频道图标，并带有免打扰（Mute）徽标。
   - **标题与副标题（Title & Presence Subtitle）**：
     - 主标题：字体大小 15px，字重 600，颜色 `--fabushi-text-primary`，溢出单行省略号截断（`text-overflow: ellipsis`）；
     - 副标题：字体大小 12px，字重 400，颜色 `--fabushi-text-secondary`。依据对象类型智能呈现：
       - 真人：展示精确的“在线（Online）”或“上次在线时间（Last seen 10 mins ago）”；
       - 智能体：空闲时显示模型与版本（如“Claude 3.5 Sonnet”），运行时显示动态动作（如“正在分析数据...”或“等待用户确认”）；
       - 群组：显示成员数与在线数（如“128 位成员，14 位在线”）；
       - 论坛话题模式：显示当前话题名称与 `#` 标识（如“# 开发讨论 · 34 条更新”）。

2. **中间上下文标尺区（Center Context Indicator）**：
   - 当处于特定话题或存在全局置顶消息时，顶栏居中显示胶囊状轻量指示器（Pill Badge），展示当前位置并支持单机快速折叠/展开话题面板或滚动至置顶消息。

3. **右侧自适应操作坞（Trailing Actions Dock）**：
   - 按钮采用统一的 32x32px 圆角方形规格，图标尺寸 16x16px，间距 6px。按会话对象属性动态显隐：
     - **实时通话（Audio/Video Call）**：针对真人会话、群组或具备语音能力的 Agent 显示；未接通时显示电话图标，通话中时显示绿色脉动通话指示条；
     - **会话内搜索（Find in Chat）**：全局常驻，快捷键 `Cmd/Ctrl + F` 唤出 `FindInChatBar`；
     - **会话接力（Agent Handoff）**：仅在真人会话且系统存在可用智能体时显示，以优雅的弹出卡片（Popover）形式替代粗暴的 `<select>` 下拉框；
     - **会话诚实分享（Share Link）**：复制受鉴权保护的邀请链接或快照，并弹出 Toast 反馈；
     - **更多操作菜单（More Options Menu, `⋯`）**：收敛低频管理动作：重命名会话、导出记录（Markdown/JSON）、免打扰切换、清空历史记录、删除会话；
     - **上下文详情栏触发器（Context Inspector Toggle, `ℹ️`）**：统一触发右侧抽屉，激活态呈现高亮背景（`var(--fabushi-bg-active)`）。

---

### 4.2 响应式三阶断点与布局自适应

严格贯彻 `FSP-001 §17` 响应式断点规范与 `openclaw-theme.css §7` 视口管理：

| 视口断点 | 顶栏呈现行为 | 舞台与消息流布局 | 详情栏与次级栏展现 |
|---|---|---|---|
| **宽屏（≥1200px）**<br>Three-Column Desktop | 顶栏完整展开，显示全部操作按钮（通话、搜索、接力、分享、更多、详情）；主副标题不折叠。 | 消息流最大宽度自适应拉伸，代码块与 Markdown 宽幅卡片支持最大 960px 沉浸展示；两侧边距 24px。 | 话题栏与置顶横幅常驻于顶栏下方；右侧 `ContextInspector` 常驻第三栏展开（320–380px）。 |
| **中屏（900–1199px）**<br>Two-Column Adaptive | 顶栏精简展示：保留通话、搜索、详情按钮；接力与分享动作自动收拢至“更多操作（`⋯`）”下拉菜单。 | 消息流保持中心对齐，卡片与图表最大宽度自适应收缩至 100%；两侧边距 16px。 | 次级栏保持展开；右侧 `ContextInspector` 转换为右侧滑出抽屉（Drawer Overlay），点击遮罩或 `Esc` 关闭。 |
| **窄屏（<900px）**<br>Single-Column Fluid | 顶栏高度降为 48px，最左侧显示“返回会话列表（Back）”箭头；仅保留搜索与更多操作按钮；标题单行紧凑截断。 | 消息流采用移动端推拉动画（Slide Transition），占满全屏宽度；两侧边距 12px。 | 话题栏转换为横向轻量水平滑动条；右侧详情栏全屏覆盖，滑动返回。 |

---

### 4.3 设计系统 `--fabushi-*` 语义 Token 绑定规范

严禁使用硬编码颜色或未经封装的 `oc-*` 平行样式。顶栏与舞台全面绑定 `--fabushi-*` 语义变量：

```css
/* 统一顶栏与舞台语义样式 */
.fabushi-conversation-stage {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  background-color: var(--fabushi-bg-canvas, #ffffff);
  color: var(--fabushi-text-primary, #111827);
  overflow: hidden;
}

.fabushi-conversation-header {
  display: flex;
  align-items: center;
  justify-content: space-between;
  height: 56px;
  min-height: 56px;
  padding: 0 16px;
  background-color: var(--fabushi-bg-surface, #ffffff);
  border-bottom: 1px solid var(--fabushi-border-subtle, #e5e7eb);
  box-shadow: var(--fabushi-shadow-xs, 0 1px 2px 0 rgba(0, 0, 0, 0.03));
  z-index: 20;
}

.fabushi-header-action-btn {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: var(--fabushi-radius-md, 6px);
  border: 1px solid transparent;
  background: transparent;
  color: var(--fabushi-text-secondary, #4b5563);
  cursor: pointer;
  transition: all 150ms ease-in-out;
}

.fabushi-header-action-btn:hover {
  background-color: var(--fabushi-bg-hover, #f3f4f6);
  color: var(--fabushi-text-primary, #111827);
}

.fabushi-header-action-btn--active {
  background-color: var(--fabushi-accent-subtle, #eff6ff);
  color: var(--fabushi-accent-primary, #1084fe);
  border-color: var(--fabushi-accent-border, #bfdbfe);
}

.fabushi-header-action-btn:focus-visible {
  outline: 2px solid var(--fabushi-accent-primary, #1084fe);
  outline-offset: 2px;
}
```

---

### 4.4 无障碍（WCAG 2.1 AA）与输入法保护规范

1. **ARIA 语义层级**：
   - 顶栏采用 `<header role="banner" aria-label="会话顶栏">` 语义包裹；
   - 标题关联 `aria-labelledby="fabushi-conversation-heading"`；
   - 动态更新状态区标记 `aria-live="polite"`；
   - 交互按钮声明精准的 `aria-label`、`aria-haspopup` 与 `aria-expanded`。
2. **全键盘可达性与焦点管理**：
   - `Tab` 键在 Leading、Sub-bars、Actions Dock 与消息流之间保序跳转；
   - 快捷键体系：`Cmd/Ctrl + F`（会话内搜索）、`Cmd/Ctrl + I`（切换详情栏）、`Esc`（关闭搜索/弹窗并恢复焦点至上一个激活元素）；
   - 模态窗（如转发对话框、分享对话框）开启时启用焦点陷阱（Focus Trap），关闭时原子恢复焦点至触发按钮。
3. **中文输入法（IME）保护机制**：
   - 消息流快速回复组件与输入框在 `compositionstart` 至 `compositionend` 期间，绝对拦截 `Enter` 按键发送，防止拼音选词误提交。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端核心 TypeScript 契约

```typescript
// frontend/src/production/conversation/conversation-stage.types.ts

export type ConversationKind = "human" | "agent" | "group" | "channel" | "hybrid";

export interface ConversationParticipantIdentity {
  readonly id: string;
  readonly name: string;
  readonly kind: ConversationKind;
  readonly avatarDataUrl?: string | null;
  readonly avatarShape?: string | null;
  readonly avatarColor?: string | null;
  readonly isOnline?: boolean;
  readonly lastSeenMs?: number | null;
  readonly isRunning?: boolean;
  readonly isComposingMessage?: boolean;
  readonly currentActivityText?: string | null;
  readonly memberCount?: number;
  readonly onlineMemberCount?: number;
  readonly verified?: boolean;
  readonly muted?: boolean;
  readonly raw?: Record<string, unknown>;
}

export interface UnifiedConversationHeaderProps {
  readonly identity: ConversationParticipantIdentity;
  readonly activeTopic?: { id: string; title: string; closed?: boolean } | null;
  readonly onOpenTopicSwitcher?: () => void;
  // 呼叫信令
  readonly canCall?: boolean;
  readonly isCallActive?: boolean;
  readonly onStartCall?: (video: boolean) => void;
  // 检索
  readonly isSearchOpen?: boolean;
  readonly onToggleSearch?: () => void;
  // 人机接力
  readonly availableHandoffAgents?: readonly ConversationParticipantIdentity[];
  readonly onHandoffToAgent?: (agentId: string) => Promise<void>;
  // 诚实分享与更多菜单
  readonly onOpenShareModal?: () => void;
  readonly onRename?: (newName: string) => Promise<void>;
  readonly onExportTranscript?: (format: "markdown" | "json") => void;
  readonly onToggleMute?: (muted: boolean) => void;
  readonly onClearHistory?: () => Promise<void>;
  // 详情抽屉
  readonly isDetailsOpen?: boolean;
  readonly onToggleDetails?: () => void;
  // 响应式单栏返回
  readonly onNavigateBack?: () => void;
  readonly isMobileCompact?: boolean;
}

export interface UnifiedConversationStageProps {
  readonly accountScopeGeneration: number;
  readonly transcriptAccountSlot: string;
  readonly conversationId: string;
  readonly identity: ConversationParticipantIdentity;
  // 话题与置顶
  readonly visibleTopics: readonly { id: string; title: string; closed?: boolean }[];
  readonly activeTopicId: string | null;
  readonly onSelectTopic: (topicId: string | null) => void;
  readonly pinnedBanner?: { pinId: string; text: string; totalPins: number } | null;
  readonly onScrollToPin: (pinId: string) => void;
  // 检索
  readonly findInChatController: unknown;
  // 消息流与输入台
  readonly transcriptProps: unknown;
  readonly composerProps: unknown;
  readonly isDetailsOpen: boolean;
  readonly onToggleDetails: () => void;
}
```

---

### 5.2 后端 Rust 契约与 IPC 通信定义

```rust
// source/host/src/extensions/session/forward_batch_journal.rs

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ForwardPrivacy {
    pub drop_sender_names: bool,
    pub drop_captions: bool,
}

impl ForwardPrivacy {
    pub fn normalize(mut self) -> Self {
        // 铁律：隐藏说明文字强制隐藏发件人姓名
        if self.drop_captions {
            self.drop_sender_names = true;
        }
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardBatchRequest {
    pub source_conversation_id: String,
    pub source_message_ids: Vec<String>,
    pub target_conversation_ids: Vec<String>,
    pub privacy: ForwardPrivacy,
    pub account_scope_generation: u64,
    pub client_batch_nonce: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForwardSettlementRecord {
    pub client_batch_nonce: String,
    pub target_conversation_id: String,
    pub destination_message_id: String,
    pub settled_at_ms: u64,
    pub success: bool,
    pub error_code: Option<String>,
}
```

---

### 5.3 核心状态转移方程与 Fencing 规则

#### 1. 消息投递状态机流转方程
$$\text{DeliveryState} \in \{\text{optimistic}, \text{queued}, \text{pending}, \text{delivered}, \text{failed}\}$$
- 状态跃迁：
  $$\text{Draft} \xrightarrow{\text{Submit}} \text{optimistic} \xrightarrow{\text{Network Offline}} \text{queued}$$
  $$\text{queued} \xrightarrow{\text{Reconnect}} \text{pending} \xrightarrow{\text{Host ACK}} \text{delivered}$$
  $$\text{pending} \xrightarrow{\text{Timeout / Rejected}} \text{failed}$$
- 失败操作分支：
  $$\text{failed} \xrightarrow{\text{onResendFailedSend}} \text{pending}$$
  $$\text{failed} \xrightarrow{\text{onDeleteFailedSend}} \emptyset \quad (\text{物理擦除暂存记录})$$

#### 2. 多重会话复合 Fencing 隔离律
为杜绝网络延迟导致的跨会话或跨话题串场，消息流组件强制绑定唯一样本槽标识：
$$\text{TranscriptKey} = \text{hash}(\text{accountScopeGeneration} \parallel \text{accountSlot} \parallel \text{conversationId} \parallel \text{activeTopicId})$$
当 $\text{TranscriptKey}$ 改变时，React 引擎必须原子卸载旧视口、清除分页游标、销毁临时监听器，并重建全新虚拟列表。旧请求返回的任一数据块均无法命中新 Key，自动 fail-closed 丢弃。


---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 子智能体修改白名单（Worker Modification Whitelist）

为防止多智能体并发修改产生代码冲突，分配至本模块的实施子智能体（Worker Subagents）**仅允许创建或修改以下白名单文件**：

| 文件绝对路径 | 操作类型 | 行数预算 | 责任范围与实施目标 |
|---|---|---|---|
| `frontend/src/production/conversation/UnifiedConversationStage.tsx` | 新增 | 250–380 行 | 统一会话主舞台容器，集成顶栏、次级栏、消息流与输入台布局。 |
| `frontend/src/production/conversation/UnifiedConversationHeader.tsx` | 新增 | 300–450 行 | 统一会话顶栏，消灭 `activeIsHuman` 分支，自适应渲染 5 类会话动作。 |
| `frontend/src/production/conversation/conversation-stage.types.ts` | 新增 | 120–200 行 | 统一会话舞台、顶栏、参与者身份、话题状态的 TypeScript 类型定义。 |
| `frontend/src/production/tdrp/TdrpShareSessionModal.tsx` | 新增 | 180–260 行 | 独立会话诚实分享模态窗，生成邀请/快照链接与二维码，严禁触发转发。 |
| `frontend/src/production/openclaw/OpenClawChatStage.tsx` | 重构/收敛 | 80–120 行 | 降级为向 `UnifiedConversationStage` 转发的轻量兼容封装，废除内置假 Copilot。 |
| `frontend/src/production/openclaw/OpenClawTopHeader.tsx` | 重构/收敛 | 80–130 行 | 降级为向 `UnifiedConversationHeader` 转发的轻量兼容层，清理假箭头与错位转发。 |
| `frontend/src/production/tdrp/ForumTopicSwitcher.tsx` | 修改 | 60–90 行 | 增强主题语义样式与 `INV-TDRP-FORUM-TOPICS-THREAD-ISOLATION-001` 话题切换回调。 |
| `frontend/src/production/tdrp/TdrpForwardMessageModal.tsx` | 修改 | 60–90 行 | 明确仅接收 `forwardTarget` 进行消息转发，切断任何兜底的 `shareUrl` 混淆。 |
| `frontend/src/production/ProductionRenderer.tsx` | 窄接线修改 | 仅限 4209–4335 行 | 移除 `activeIsHuman` 顶栏二元分化分支，以单行 `<UnifiedConversationStage ... />` 完成窄接线挂载。 |

### 6.2 只读参考文件清单（Read-Only References）

- `frontend/src/recovered/features/conversation/workspace/transcript.tsx`（保持高性能虚拟流核心逻辑不变）
- `frontend/src/recovered/features/conversation/workspace/conversation-pinned-banner.tsx`（置顶横幅挂载基准）
- `frontend/src/recovered/features/conversation/workspace/find-in-chat-bar.tsx`（检索栏挂载基准）
- `source/host/src/extensions/session/production.rs`（SandAgentDb 消息与会话存储接口）
- `native/mahayana-messaging/src/engine_tdrp_topics_channels.rs`（话题原生通信协议）
- `projects/telegram-desktop-rust/dossiers/forum-topics-complete-read.md`（话题隔离溯源证据）
- `projects/telegram-desktop-rust/dossiers/forward-options-complete-read.md`（转发隐私溯源证据）

---

### 6.3 与 `ProductionRenderer.tsx` 的窄接线规范

在 `ProductionRenderer.tsx` 中，彻底替换原第 4209 至 4335 行的粗暴二元分叉与多层内联组件拼装，收口为高内聚的窄接线装配：

```tsx
// 改造后 ProductionRenderer.tsx:4209-4250 窄接线目标形态
{activeAgent == null ? (
  showRootEmptyWorkspace ? <RootShellEmptyWorkspace isVisible /> : null
) : (
  <UnifiedConversationStage
    accountScopeGeneration={renderedAccountScopeGeneration}
    transcriptAccountSlot={transcriptAccountSlot}
    conversationId={activeAgent.id}
    identity={projectUnifiedConversationIdentity(activeAgent, activeIsHuman, client)}
    visibleTopics={visibleTopics}
    activeTopicId={activeTopicId}
    onSelectTopic={setActiveTopicId}
    pinnedBanner={pinnedBanner}
    onScrollToPin={(pinId) => transcriptHandleRef.current?.scrollToEntryWithoutHighlight(pinId)}
    findInChatController={findInChatController}
    findInChatOpen={findInChatOpen}
    onCloseFindInChat={closeFindInChat}
    onOpenSearch={openSidebarSearch}
    canCall={client != null}
    isCallActive={client != null && isCallInProgress(client, activeAgent.id)}
    onStartCall={(video) => initiateCall(activeAgent.id, video)}
    availableHandoffAgents={availableHandoffAgents}
    onHandoffToAgent={handoffHumanConversationToAgent}
    onOpenShareModal={() => setIsShareModalOpen(true)}
    isDetailsOpen={tdrpDetailsOpen || runtimeDetailPaneOpen}
    onToggleDetails={() => setTdrpDetailsOpen((open) => !open)}
    transcriptProps={projectTranscriptBindingProps({
      entries: displayedEntries,
      transcriptPaginationSnapshot,
      loadOlderTranscript,
      bindTranscriptViewport,
      cancelQueuedSend,
      removeTranscriptMessage,
      resendFailedSend,
      replyThreadController,
      forwardMessage: (entry) => {
        if (isForwardAccountScopeCurrent(renderedAccountScopeGeneration)) {
          setForwardTarget({ sourceConversationId: activeAgent.id, message: entry, accountScopeGeneration: renderedAccountScopeGeneration });
        }
      },
      reactionHandlers: { renderReactionActions, renderReactionPills },
      attachmentHandlers: { resolveAttachmentMedia, bridge },
    })}
    composerProps={projectComposerBindingProps({
      draft,
      busy,
      isActiveSendPending,
      client,
      composerGuard,
      composerClearGeneration,
      editorProviders,
      notice,
      setDraft: (v) => composerDraftStore.setDraft(activeAgent.id, v),
      clearReplyTarget,
      removeAttachment,
      stageFiles,
      submit,
      replyTarget,
      transcribeAudio,
      openTdrpModal: (m) => setTdrpModalMode(m),
    })}
  />
)}
```

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 负例作弊排查表（Forbidden Patterns Checklist）

独立审计子智能体（Auditor Subagent）必须逐行扫描代码与蓝图，存在以下任意一项即判定不合格并驳回：

- [ ] **严禁保留 `activeIsHuman` 顶栏二元分化分支**：`ProductionRenderer.tsx` 根部严禁存在 `{activeIsHuman ? <div className="sand-chat-header"> : <OpenClawTopHeader>}` 的并列逻辑，必须统一使用 `UnifiedConversationStage`。
- [ ] **严禁分享按钮暗中转发末条消息**：点击分享按钮严禁触发任何 `onForward`、`forwardHumanMessage` 或抓取 `displayedEntries[displayedEntries.length - 1]` 发送的操作。
- [ ] **严禁在分享链接中草率退化为 `window.location.href`**：分享会话链接必须优先读取后端的真实 inviteLink、shareUrl 或由 Session 模块生成的加密快照 URL。
- [ ] **严禁保留无真实历史栈支撑的静态箭头**：顶栏中的前进后退图标必须在有明确 `canGoBack / canGoForward` 且具有主进程历史栈路由回调时方可呈现，严禁保留无交互回调的死图标。
- [ ] **严禁跨会话/话题数据泄露**：切换话题或会话时必须重置 `TranscriptKey`，严禁让上一个话题的未决流式 chunk 写入当前消息视口。
- [ ] **严禁真人新会话展现未处理的空白**：当消息列表为空时，真人会话必须呈现具有头像、用户名与打招呼建议模板的统一名片视图，严禁仅对 Agent 显示欢迎屏。
- [ ] **严禁伪造不存在的 API 或硬编码在线状态**：所有组件属性与接口必须对应代码库中真实存在的类型，禁止在 Props 中传递硬编码的 `isOnline={true}`。

---

### 7.2 正例验收确认准则（Positive Verification Gates）

1. **蓝图完备性验证**：
   - `docs/specs/fusion-blueprints/02-unified-conversation-stage-and-header.md` 完整落盘。
   - 完整包含 7 大强制章节，无任何省略或占位符。
   - 源码五问（Q1..Q5）具有确凿的代码行号与三方对照证据。
2. **架构收口规范性**：
   - 确立 `UnifiedConversationStage` 与 `UnifiedConversationHeader` 为单一权威归属。
   - 彻底解耦会话分享（`TdrpShareSessionModal`）与单条消息转发（`TdrpForwardMessageModal`）。
   - 规划了 200–600 行内聚预算的新增/重构文件，并提供了与 `ProductionRenderer.tsx` 的清晰窄接线契约。
3. **安全与协议健全性**：
   - 阐明了 `accountScopeGeneration` 多维复合 Fencing 隔离律。
   - 阐明了 `ForwardPrivacy` 状态规范化转移方程（`dropCaptions => dropSenderNames`）。
4. **无命令违规（Zero Command Execution）**：
   - 严格遵守 `AGENTS.md` 规范，本蓝图编制全程未调用任何本地终端构建或测试命令，纯粹依赖静态源码比对、引用图谱核实与深度架构推演完成。

---

> **蓝图编制结论**：  
> `02-unified-conversation-stage-and-header.md` 已全面消除 `activeIsHuman` 割裂，理顺了会话顶栏、次级栏、消息流、话题切换与分享/转发架构，为 Worker 子智能体的实施落地提供了不可动摇的精确契约。
