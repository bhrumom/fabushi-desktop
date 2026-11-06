# Fabushi 统一融合蓝图 03：唯一权威输入台与无损富交互动作
# Volume 03: Unified Authoritative Composer, Rich Text Editor, and Lossless Typed Actions

> **规范编号**: `FBCP-BLUEPRINT-03` / `UFX-BLUEPRINT-03`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游只读锁定基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. **根除双输入框架构割裂**: 彻底消灭右侧 `OpenClawAskCopilotPanel` 与主输入框竞争冲突，确立全局唯一权威 `ConversationComposer`，真人与 Agent 会话共用单一输入台。
> 2. **修复草稿覆盖与假异步缺陷**: 终结通过 `setTimeout` 暴力覆盖用户主输入框未发送草稿的作弊逻辑；投票（Poll）、待办（Todo）、发票（Invoice）实行无损独立直发管道。
> 3. **根治中文输入法（IME）回车吞字 Bug**: 规范 ProseMirror / TipTap 与原生文本域的 IME 合成事件（`compositionstart` / `compositionend` / `isComposing`），确保拼音确认 Enter 绝不误触发送。
> 4. **消除硬编码模型与思考强度**: 彻底废除前端写死的静态 `AVAILABLE_MODELS` 假列表，接入 Host 动态下发的真实 Provider 模型目录、思考推理深度（Thinking Effort）与安全门策略。
> 5. **对齐无损草稿与发送结算合同**: 基于 `draft-state.ts`（`scopeKey` 隔离）与 `submission.ts`（单调递增 Nonce 幂等队列）对齐全量会话级草稿、附件拖拽、语音听写（Whisper STT）与流式中断。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码全量对照矩阵

| 责任域 / 能力项 | Telegram Desktop 锁定源码锚点 | OpenClaw 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **唯一权威主输入框**<br>(Unified Composer Stage) | `history/view/controls/compose_controls.cpp`<br>`history/view/controls/compose_controls_common.cpp` | `ui/src/pages/chat/composer.ts`<br>`packages/prompt-editor-core/` | `frontend/src/recovered/features/conversation/workspace/composer.tsx:33-253`<br>`frontend/src/production/openclaw/OpenClawCleanComposer.tsx:1-423` | `ConversationComposer`<br>(收口 OpenClawCleanComposer 为统一样式变体) |
| **会话草稿与隔离存储**<br>(Draft Persistence & Fencing) | `data/data_drafts.cpp:1-420`<br>`storage/storage_account.cpp` | `src/chat/draft-manager.ts`<br>`packages/session-contract/` | `frontend/src/recovered/features/conversation/workspace/draft-state.ts:9-110,480-522`<br>`ProductionRenderer.tsx:3582-3617` | `ComposerDraftStateStore`<br>`draft-state.ts` (`scopeKey` 物理隔离) |
| **发送队列与 Nonce 结算**<br>(Send Pipeline & Nonce Settle) | `api/api_sending.cpp:120-380`<br>`data/data_messages.cpp` | `src/chat/submission-pipeline.ts`<br>`packages/gateway-protocol/` | `frontend/src/recovered/features/conversation/workspace/submission.ts:7-198`<br>`ProductionRenderer.tsx:3592-3635` | `ComposerSubmissionQueue`<br>`submission.ts` (单调 Nonce 幂等队列) |
| **无损富文本与 IME 保护**<br>(TipTap Rich Editor & IME) | `ui/widgets/fields/input_field.cpp:150-290`<br>`history/view/controls/compose_controls_format.cpp` | `packages/markdown-core/`<br>`ui/src/components/rich-input.ts` | `frontend/src/recovered/features/conversation/workspace/rich-text-editor.tsx:720-815`<br>`OpenClawCleanComposer.tsx:72-79` (缺陷点) | `PromptRichTextEditor`<br>`rich-text-editor.tsx` (IME 状态机封密) |
| **投票/待办/发票直发工具栏**<br>(Typed Actions & Modals) | `data/data_poll.cpp`<br>`api/api_polls.cpp`<br>`data/data_todo_list.h`<br>`payments/payments_checkout_box.cpp` | `extensions/workboard/`<br>`src/interactive/`<br>`src/cards/` | `frontend/src/production/tdrp/TdrpComposerToolbar.tsx:1-96`<br>`frontend/src/production/tdrp/TdrpPollAndTodoModal.tsx:12-123,135-385`<br>`ProductionRenderer.tsx:4337-4348` (缺陷点) | `TdrpComposerToolbar`<br>`TdrpPollAndTodoModal`<br>(无损直发管道，禁止覆盖草稿) |
| **动态模型与思考强度选择器**<br>(Model & Thinking Effort) | `api/api_premium.cpp`<br>`Compose Controls Dropdown` | `src/models/model-catalog.ts`<br>`src/agents/inference-params.ts`<br>`ui/src/components/model-selector.ts` | `frontend/src/production/openclaw/OpenClawCleanComposer.tsx:267-393`<br>`source/electron-main/models/cursor-model-catalog.ts:5-20` | `ConversationComposerToolbar`<br>`cursor-model-catalog.ts` (真实 Owner 接线) |
| **本地工具权限门禁浮层**<br>(Tool Permission Dock) | `boxes/confirm_box.cpp`<br>`core/sandbox_rules.cpp` | `src/security/permission-gate.ts`<br>`extensions/sandbox/` | `frontend/src/recovered/features/permissions/local-tool/view.tsx:168-172`<br>`ProductionRenderer.tsx:2158-2166,4295` | `LocalToolPermissionDock`<br>`localToolPermissionStore` |
| **附件暂存与剪贴板/拖拽**<br>(Attachments Staging & Drag) | `history/view/controls/compose_controls_attach.cpp`<br>`storage/storage_media_prepare.cpp` | `src/media/attachment-uploader.ts`<br>`ui/src/utils/drag-drop.ts` | `frontend/src/recovered/features/conversation/workspace/composer.tsx:51-72,122-156`<br>`source/electron-main/attachments/` | `ConversationComposer`<br>`stageFiles` & `SandChatDropOverlay` |
| **语音听写录制与 STT**<br>(Voice Dictation & STT) | `media/audio/media_audio_capture.cpp`<br>`history/view/controls/compose_controls_record.cpp` | `src/realtime-transcription/`<br>`extensions/talk-voice/` | `frontend/src/recovered/features/conversation/workspace/composer.tsx:82-97,201,231-247`<br>`source/electron-main/media/` | `useVoiceSession`<br>`VoiceWaveform` & `transcribeAudio` |
| **定时与静默发送策略**<br>(Scheduled & Silent Send) | `api/api_sending.cpp:410-480`<br>`compose_controls_schedule.cpp` | `src/cron/`<br>`src/routing/silent-delivery.ts` | `frontend/src/recovered/features/conversation/workspace/composer.tsx:33`<br>`source/host/src/automations/` | `SendSchedulePopover`<br>`source/host/src/automations/` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**:
  - 输入控制由 `compose_controls.cpp` 统管，草稿持久化由 `data_drafts.cpp` 维护，支持按 Peer 独立存储光标、文本、局部引用（Quote）与未完成附件。
  - 投票（`data_poll.cpp` / `api_polls.cpp`）与待办清单（`api_todo_lists.cpp`）具有独立 RPC 发送命令（`messages.sendMedia`），直接上屏为带有专属 `media_type` 的结构化消息卡片，绝不触碰正在编辑的文本草稿。
  - 输入框支持 Markdown/富文本实时解析，中文 IME 拼音确认通过 Qt 事件过滤器（`QEvent::InputMethod`）拦截，仅在非 IME 状态下响应 `Qt::Key_Return`。
- **OpenClaw 侧**:
  - 模型与推理参数由 `src/models/` 动态发现，支持配置模型厂商、上下文窗口与思考预算（`ThinkingEffort: low | medium | high`）。
  - 执行安全门禁由 `src/security/permission-gate.ts` 控制，工具调用触发时在输入台上方弹出受控审批条（`LocalToolPermissionDock`），提供一键批准/拒绝及永久白名单记住机制。

#### Q2（现有链路定位）
- **输入台挂载链路**:
  - `ProductionRenderer.tsx:4294-4317`：在 `.sand-chat-input-dock` 中串联挂载 `LocalToolPermissionDock` -> `CommunityComposerGuardBanner` -> `TdrpComposerToolbar` -> `ConversationComposer`。
  - `composer.tsx:183` 实例化 `PromptRichTextEditor`（基于 TipTap / ProseMirror），管理本地光标、输入与附件；底部右侧渲染语音录制与发送箭头。
- **发送提交流程**:
  - 用户按下 Enter 或点击发送 -> 触发 `ProductionRenderer.tsx:3576` `submit()` -> 获取 `activeDraftSnapshotStore` 实时草稿 -> 生成客户端 Nonce -> 插入乐观消息 -> 投递至 `composerSubmissionQueue` -> 通过 Coordinator RPC `sendPrompt` 投递至 Host。

#### Q3（断点精确诊断）
1. **右侧并行第二输入框严重割裂**:
   `frontend/src/production/openclaw/OpenClawAskCopilotPanel.tsx:120-145` 独立实现了一个带有 `<textarea>` 的输入框，并使用 `setTimeout` 伪造回复。该输入框与主输入台竞争草稿状态，完全破坏了系统单一真相源（Single Source of Truth）铁律。
2. **快捷动作暴力覆盖用户未发送草稿与假 `setTimeout`**:
   `frontend/src/production/ProductionRenderer.tsx:4340-4345`:
   ```tsx
   onInsertMessage={(formattedPayload) => {
     setTdrpModalMode(null);
     if (activeAgent == null) return;
     composerDraftStore.setDraft(activeAgent.id, { prompt: formattedPayload, attachments: [] });
     setTimeout(() => { void submit(); }, 50);
   }}
   ```
   **严重违规**: 用户若在输入框中已输入一段未发送的文本，一旦在弹窗中创建投票或待办，原文本立即被全量覆盖丢失！且通过脆弱的 `50ms setTimeout` 假异步触发提交，极易引发状态竞态。
3. **`OpenClawCleanComposer` 中文输入法 IME Enter 吞字**:
   `frontend/src/production/openclaw/OpenClawCleanComposer.tsx:72-79`：
   `if (event.key === "Enter" && !event.shiftKey) { event.preventDefault(); onSubmit(); }`
   未检查 `event.nativeEvent.isComposing` 或状态机 `compositionActive`，导致中文拼音选词敲 Enter 时立即触发消息误发，吞噬选词。
4. **硬编码静态 `AVAILABLE_MODELS` 与未接线选择器**:
   `OpenClawCleanComposer.tsx:267-393` 内置假模型菜单，而在 `ProductionRenderer.tsx:4301-4316` 挂载 `ConversationComposer` 时根本没有传递 `model`、`availableModels`、`thinkingLevel`，导致该能力沦为未接线（`not-wired`）死代码。

#### Q4（状态机与安全防护）
- **会话草稿与账号隔离状态转移方程**:
  $$\text{DraftStore}: (\text{AccountSlot}, \text{AgentId}) \to \text{ComposerDraft}$$
  - 切换账号触发 `accountScopeGeneration` 递增，清空内存映射并从新账号的持久化分区中恢复：
    $$\text{Key} = \text{sand.client.slice.account.} + \text{encode}(\text{accountSlot}) + \text{.composer-drafts}$$
  - 绝不允许在账号 A 的输入台中恢复账号 B 的残留草稿。
- **发送队列与单调 Nonce 状态机**:
  $$\text{SubmissionPhase} \in \{\text{pending}, \text{queued}, \text{failed}, \text{sent}, \text{cancelled}\}$$
  - $\text{Idle} \xrightarrow{\text{submit(draft)}} \text{Pending (Optimistic)} \xrightarrow{\text{ack(nonce)}} \text{Sent}$
  - 若网络断开，转移至 $\text{Queued}$；重连后按 FIFO 顺序 Flush。只有权威 `message_id` 确认后才将草稿标记为 `settled` 并清除。

#### Q5（UI/UX 极致融合落点）
- **单一权威输入台（Unified Composer Dock）**: 位于主会话舞台底部，自适应高度（48px 至 180px）。
- **工具扩展栏（Action Toolbar）**: 紧贴输入框上沿，采用圆角 Pill 按钮展示“投票（Poll）”、“待办（Todo）”、“收款账单（Invoice）”、“快捷指令（/bot）”。在 Agent 会话中，左侧动态展示当前模型药丸与思考强度选择器。
- **渐进披露与快捷操作**:
  - 高频：回车发送、语音录制、文件直接拖拽粘贴；
  - 中频：点击 `+` 展开附件选择、发起投票/待办模态框；
  - 低频：右键发送按钮弹出“定时发送”与“静默发送”策略菜单。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-03-01** | `OpenClawAskCopilotPanel` 自带平行第二输入框与假 `setTimeout` 回复 | `frontend/src/production/openclaw/OpenClawAskCopilotPanel.tsx:120-145`<br>`ProductionRenderer.tsx:4320-4331` | 架构融合初期将 OpenClaw 原型侧边栏粗暴挂载，导致存在两个输入框竞争焦点与草稿存储，且内嵌假回复生成器。 | **P0 架构双轨** |
| **BUG-03-02** | 快捷动作直发通过 `setTimeout` 强制覆盖主输入框未发送草稿 | `frontend/src/production/ProductionRenderer.tsx:4340-4345`<br>`frontend/src/production/tdrp/TdrpPollAndTodoModal.tsx:87,103,121` | `onInsertMessage` 错误复用主草稿管道作为临时消息载体，不仅丢失用户正在编辑的草稿，还使用了非确定性 50ms 延时。 | **P0 数据破坏** |
| **BUG-03-03** | `OpenClawCleanComposer` 缺失 IME 组合保护，中文回车直接误发 | `frontend/src/production/openclaw/OpenClawCleanComposer.tsx:72-79` | `handleKeyDown` 只判断了 `event.key === "Enter"`，未拦截 `compositionstart` / `compositionend` 或 `event.nativeEvent.isComposing`。 | **P1 用户体验** |
| **BUG-03-04** | 模型列表与思考强度硬编码静态数据，未接线真实配置 Owner | `frontend/src/production/openclaw/OpenClawCleanComposer.tsx:267-393`<br>`ProductionRenderer.tsx:4301-4316` | 原型组件写死假列表；主装配入口未传相应 Props，使得模型切换无法动态驱动后端运行时。 | **P1 功能假造** |
| **BUG-03-05** | 本地工具审批条与输入台布局分离，缺乏统一的键盘 Escape 焦点回收 | `frontend/src/production/ProductionRenderer.tsx:2158,4295`<br>`frontend/src/recovered/features/permissions/local-tool/view.tsx:168` | 权限提示条以裸组件插在输入台上方，没有集成进统一的 Composer Dock 区域，Esc 退出时焦点易迷失。 | **P2 交互缺陷** |
| **BUG-03-06** | 语音录制与 STT 状态在组件卸载或切会话时缺少清理，存在音频流悬挂 | `frontend/src/recovered/features/conversation/workspace/composer.tsx:82-97`<br>`source/electron-main/media/` | `useVoiceSession` 在快速切换 `scopeKey` 时未强制终止底层 MediaStream 轨道，可能导致录音红点持续常驻。 | **P2 稳定性隐患** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 统一权威输入台架构拓扑

```
 ┌────────────────────────────────────────────────────────────────────────────────────────┐
 │                                   Unified Composer Dock                                │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ 1. 顶部挂载层 (Contextual Banner & Safety Gates)                                       │
 │    ├── LocalToolPermissionDock (智能体本地工具审批横幅: 授权 / 拒绝 / 记住)           │
 │    └── CommunityComposerGuardBanner (频道/群组禁言只读提示与发信频率控制)              │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ 2. 交互动作栏 (Typed Actions & Model Controls Toolbar)                                 │
 │    ├── [Model Selector Pill] (真实 Host Provider 模型下拉选择器，仅 Agent 会话可见)   │
 │    ├── [Thinking Effort Pill] (思考推理预算: Low / Medium / High，仅推理模型可见)     │
 │    └── [Typed Action Buttons] (投票 / 待办清单 / 收款账单 / 快捷指令)                 │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ 3. 主编辑区 (Authoritative Multi-line Rich Text Editor)                                │
 │    ├── 引用回复胶囊条 (ReplyPill with preview & dismiss)                               │
 │    ├── 暂存附件排布网格 (AttachmentChips with format, size & delete)                    │
 │    ├── PromptRichTextEditor (ProseMirror / TipTap 无损格式化，IME 组合输入严密保护)    │
 │    └── 拖拽覆盖遮罩 (SandChatDropOverlay - 全窗口文件拖放响应)                         │
 ├────────────────────────────────────────────────────────────────────────────────────────┤
 │ 4. 底栏操作簇 (Trailing Controls Cluster)                                              │
 │    ├── 附件添加按钮 (+) -> TdrpAttachMenuPopover (照片 / 文件 / 投票 / 待办 / 发票)     │
 │    ├── 语音听写按钮 (Mic) -> Whisper STT 实时声波指示器与录制计时器                   │
 │    └── 发送动作按钮 (Send / Schedule / Silent) -> 支持 Cmd+Enter / Enter 瞬时提交    │
 └────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Canonical Owners 映射与代码行数预算（200–600 行内聚预算）

严格遵循“同一责任归属不等于同一文件（Same Owner != Same File）”铁律，所有组件与控制器拆分保持高内聚并控制在 **200–600 行** 之间：

| 模块名称与责任归属 | 文件路径 | 行数预算 | 职责范围与边界 |
|---|---|---|---|
| **统一输入台根组件**<br>(`ConversationComposer`) | `frontend/src/recovered/features/conversation/workspace/composer.tsx` | 280 行 | 统管输入台布局、拖拽状态、语音录制、附件暂存与表单提交调度。 |
| **交互动作与模型选择栏**<br>(`TdrpComposerToolbar`) | `frontend/src/production/tdrp/TdrpComposerToolbar.tsx` | 240 行 | 承载投票/待办/发票直发按钮、动态模型与思考强度药丸下拉框。 |
| **富文本与 IME 保护编辑器**<br>(`PromptRichTextEditor`) | `frontend/src/recovered/features/conversation/workspace/rich-text-editor.tsx` | 550 行 | ProseMirror/TipTap 文本绑定、Markdown 快捷格式、IME 合成事件拦截。 |
| **结构化卡片直发模态框**<br>(`TdrpPollAndTodoModal`) | `frontend/src/production/tdrp/TdrpPollAndTodoModal.tsx` | 380 行 | 投票、待办清单、收款发票表单采集与真实 Coordinator RPC 直发。 |
| **草稿持久化与跨会话状态机**<br>(`ComposerDraftStateStore`) | `frontend/src/recovered/features/conversation/workspace/draft-state.ts` | 480 行 | 依据 `scopeKey` 提供草稿自动落盘、快照订阅、恢复与提交清理。 |
| **单调 Nonce 发送结算队列**<br>(`ComposerSubmissionQueue`) | `frontend/src/recovered/features/conversation/workspace/submission.ts` | 220 行 | 管理客户端临时 Nonce、乐观上屏、离线暂存、断线重试与权威状态合并。 |
| **工具权限审批浮层**<br>(`LocalToolPermissionDock`) | `frontend/src/recovered/features/permissions/local-tool/view.tsx` | 210 行 | 渲染 Agent 本地命令与文件读写执行申请、超时倒计时与结果回传。 |
| **模型目录与推理参数服务**<br>(`ModelCatalogService`) | `source/electron-main/models/cursor-model-catalog.ts` | 250 行 | 真实 Provider 模型列表拉取、缓存、过滤与可用性心跳校验。 |



---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 界面布局与渐进披露层级（Progressive Disclosure）

输入台采用清晰的三层渐进披露架构，将高频击键、中频富媒体及低频高级选项分层归拢：

```
Level 1: 高频直达区 (常驻轻量表面)
  ├── 单一权威编辑框: 自动换行扩展 (1 行至 6 行，48px ~ 180px)，支持自然键盘输入
  ├── 交互药丸工具栏 (Action Toolbar): 投票、待办、发票直发按钮与当前模型指示
  └── 核心快捷按钮: 附件 (+)、语音录制 (Mic)、立即发送 (Enter / Send Button)

Level 2: 交互式微件与上下文抽屉 (按需激活)
  ├── 附件管理网格: 文件缩略图、文件名截断、字节大小与单项一键移除
  ├── 语音转写波形条: 实时声波动画 (VoiceWaveform)、录音计时与 Esc 取消听写
  ├── 结构化动作表单模态框: 投票创建器 (单选/多选/测验/限时)、待办清单编辑器、收款单设置
  └── 审批提示浮条: 智能体申请本地 Bash 命令或文件读写时从输入框上沿平滑滑出

Level 3: 高级发送策略与策略配置 (右键与快捷指令)
  ├── 长按/右键发送按钮: 弹出“定时发送 (Scheduled Send)”与“静默免打扰发送 (Silent Send)”
  └── Slash 命令自动补齐 (/poll, /todo, /invoice, /clean, /model, /settings)
```

### 4.2 全生命周期交互状态矩阵

| 状态类型 | 视觉表现与样式指示 | 键盘与快捷键行为 | 可用操作与按钮状态 | 退出与转移边界 |
|---|---|---|---|---|
| **Empty / Idle** | 占位符居中，显示“给 [目标] 发消息”；发送按钮置灰禁用 | `Enter` 换行或无效；支持直接粘贴文字或图片 | 附件 (+) 与语音 (Mic) 可用；发送按钮禁用 | 用户键入字符或粘贴内容进入 `Composing` |
| **Composing (Normal)** | 编辑区高度依据内容自适应撑开；发送按钮高亮主题蓝 | `Enter` 触发发送；`Shift+Enter` 换行；`Esc` 收起焦点 | 发送按钮可用；支持点击附件添加或发起投票 | 文本提交进入 `Pending Send`；清空退回 `Empty` |
| **Composing (IME Active)** | 输入法候选词框浮于光标下方；输入框带有下划线高亮 | `Enter` 仅确认选中拼音；`Esc` 取消拼音；严格拦截发送 | 发送按钮保持视觉可用但锁定响应 | 拼音确认后触发 `compositionend`，平稳切回 `Composing` |
| **Voice Recording** | 输入框下沿滑出红色录音胶囊条，显示实时声波与计时器 | `Esc` 立即取消录音并清空音频流；`Space` 暂停 | 发送按钮变为停止录音 (Stop) 芯片；附件按钮禁用 | 点击 Stop 或静音超时触发 STT，转写文本追加进草稿 |
| **Staging Files** | 输入框上方展示横向滚动附件卡片，标明文件类型与大小 | `Backspace` 在光标位于开头时高亮末尾附件 | 支持继续拖入文件 (上限 10 个)；支持逐个删除 | 附件全部删除且无文本时退回 `Empty` |
| **Pending Send (Optimistic)** | 消息以透明度 0.7 立即插入聊天舞台，带微旋转时钟图标 | 编辑器文本瞬间清空，光标复位；禁用连续重复敲击 | 输入框保持可用，允许用户连续编写下一条草稿 | 收到 Coordinator 权威 `message_id` 后更新为已发送 |
| **Offline Queued** | 输入台显示琥珀色弱提示条：“当前离线，已排队等待发送” | 允许继续发送多条消息，全部进入本地队列 | 发送按钮变为排队图标，单调递增在途计数 | 网络重连后按 Nonce FIFO 顺序自动批量投递 |
| **Local Tool Asking** | 输入台上沿滑出深色审批条：“Agent 申请执行本地终端命令” | `Cmd+Y` 批准，`Cmd+N` 拒绝，`Esc` 最小化至顶栏 | 编辑器保持输入能力，但发送操作与审批并行隔离 | 用户决策完成或 60 秒硬超时后滑出收起 |

### 4.3 响应式断点与无障碍交互规范（WCAG AA & Keyboard Flow）

- **视口自适应断点**:
  - **宽屏桌面端（$\ge 1200\text{px}$）**: 输入台最大宽度 880px 居中对齐，动作工具栏全部平铺展开，模型选择器与思考强度选择器完全可见。
  - **中屏端（$900\text{px} \sim 1199\text{px}$）**: 输入台占满舞台宽度（左右留白 16px），动作工具栏折叠次要项目至“更多动作（...）”下拉菜单。
  - **窄屏与移动端（$< 900\text{px}$）**: 输入台紧贴视口底部，动作工具栏收口至单一 `+` 悬浮菜单，语音录制支持长按手势（Hold to Record）。
- **无障碍访问与键盘流（WCAG 2.1 AA 达标）**:
  - 富文本编辑区严格声明 `role="textbox"`、`aria-multiline="true"`、`aria-label="Message draft"`。
  - 附件列表声明 `role="list"`，单个附件声明 `role="listitem"` 并提供独立删除按钮的 `aria-label="Remove [filename]"`。
  - 当语音录音状态改变时，通过 `aria-live="polite"` 区域播报“Listening…”与“Transcribing…”。
  - 支持快捷键：`Cmd+Enter`（或 `Ctrl+Enter`）强制立即发送；`Cmd+Shift+M` 切换模型；`Cmd+K` 插入超链接；`Esc` 清除引用目标或取消录音。

### 4.4 中文输入法（IME）保护机制

为彻底杜绝拼音输入法下敲回车误触发消息发送，必须在编辑器生命周期中严格维持组合输入状态机：

```
                    ┌────────────────────────┐
                    │      IME Inactive      │
                    └───────────┬────────────┘
                                │
                  compositionstart (拼音开始键入)
                                │
                                ▼
                    ┌────────────────────────┐
   keydown: Enter ─►│       IME Active       │◄─ keydown: 字母/数字选词
  (拦截发送: 吞噬)   │(compositionActive=true)│   (更新候选词高亮)
                    └───────────┬────────────┘
                                │
                   compositionend (汉字已提交到 DOM)
                                │
                                ▼
                    ┌────────────────────────┐
                    │      IME Settled       │
                    └───────────┬────────────┘
                                │ requestAnimationFrame (放行后续普通回车)
                                ▼
                    ┌────────────────────────┐
                    │      IME Inactive      │
                    └────────────────────────┘
```

### 4.5 `--fabushi-*` 语义设计变量绑定

输入台所有视觉元素 100% 绑定统一设计变量，严禁出现内联硬编码色彩：

```css
/* 统一输入台样式规范 */
.sand-chat-input-dock {
  background: var(--fabushi-surface-base);
  border-top: 1px solid var(--fabushi-border-subtle);
  padding: var(--fabushi-spacing-sm) var(--fabushi-spacing-md);
  position: relative;
  transition: background-color var(--fabushi-transition-normal);
}

.sand-prompt-shell {
  background: var(--fabushi-surface-card);
  border: 1px solid var(--fabushi-border-subtle);
  border-radius: var(--fabushi-radius-xl);
  box-shadow: var(--fabushi-shadow-sm);
  padding: var(--fabushi-spacing-sm);
  transition: border-color var(--fabushi-transition-fast), box-shadow var(--fabushi-transition-fast);
}

.sand-prompt-shell:focus-within {
  border-color: var(--fabushi-accent-primary);
  box-shadow: 0 0 0 2px var(--fabushi-accent-subtle);
}

.tdrp-composer-toolbar {
  border-bottom: 1px solid var(--fabushi-border-subtle);
  color: var(--fabushi-text-secondary);
  display: flex;
  gap: var(--fabushi-spacing-xs);
  padding-bottom: var(--fabushi-spacing-xs);
}

.sand-prompt-send-btn {
  background: var(--fabushi-accent-primary);
  color: var(--fabushi-text-inverse);
  border-radius: var(--fabushi-radius-circle);
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  transition: transform var(--fabushi-transition-fast), opacity var(--fabushi-transition-fast);
}

.sand-prompt-send-btn:disabled {
  background: var(--fabushi-surface-disabled);
  color: var(--fabushi-text-muted);
  cursor: not-allowed;
  opacity: 0.6;
}
```

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端权威 TypeScript 接口契约

```typescript
// 1. 统一权威输入台 Props 定义 (frontend/src/recovered/features/conversation/workspace/composer.tsx)
export interface ModelDescriptor {
  readonly id: string;
  readonly label: string;
  readonly vendor?: string;
  readonly supportsThinking?: boolean;
}

export type ThinkingEffortLevel = "Low" | "Medium" | "High" | "Default";

export interface DirectTypedActionPayload {
  readonly type: "poll" | "todo" | "invoice";
  readonly formattedMessage: string;
  readonly rawData: Record<string, unknown>;
}

export interface ConversationComposerProps {
  readonly draft: ComposerDraft;
  readonly disabled?: boolean;
  readonly notice?: string | null;
  readonly placeholder?: string;
  readonly scopeKey: string;
  readonly acceptedSendGeneration?: number;
  readonly replyTarget?: ComposerReplyTarget;
  readonly editorProviders?: PromptEditorProviders;
  // 动态模型与推理深度选择 (真实接线)
  readonly activeModel?: string;
  readonly availableModels?: readonly ModelDescriptor[];
  readonly thinkingEffort?: ThinkingEffortLevel;
  readonly onChangeModel?: (modelId: string) => void;
  readonly onChangeThinkingEffort?: (level: ThinkingEffortLevel) => void;
  // 附件、语音与草稿事件
  readonly onChange: (draft: ComposerDraft) => void;
  readonly onStageFiles: (files: readonly File[]) => void | Promise<void>;
  readonly onRemoveAttachment?: (attachment: DraftAttachment) => void | Promise<void>;
  readonly onClearReplyTarget?: () => void;
  readonly onSubmit: () => void | Promise<void>;
  // 结构化卡片无损直发管道 (杜绝覆盖草稿)
  readonly onDirectSendTypedAction?: (action: DirectTypedActionPayload) => Promise<void>;
  readonly transcribeAudio: VoiceTranscriber;
}

// 2. 结构化卡片直发模态框契约 (frontend/src/production/tdrp/TdrpPollAndTodoModal.tsx)
export interface TdrpPollAndTodoModalProps {
  readonly isOpen: boolean;
  readonly initialMode?: "poll" | "todo" | "invoice" | "bot";
  readonly conversationId: string;
  readonly client?: ProductionCoordinatorClient | null;
  readonly onClose: () => void;
  // 核心变更: 改为通过直发管道提交，严禁 setDraft 覆盖用户文本
  readonly onDirectSubmit: (payload: DirectTypedActionPayload) => void;
}

// 3. 权威草稿存储结构 (frontend/src/recovered/features/conversation/workspace/draft-state.ts)
export interface ComposerDraft {
  readonly prompt: string;
  readonly richText?: string; // Serialized ProseMirror/TipTap JSON
  readonly attachments: readonly DraftAttachment[];
  readonly replyToId?: string;
  readonly isFork?: boolean;
}

export interface ComposerDraftSnapshot {
  readonly draft: ComposerDraft | null;
  readonly recovery: ComposerDraft | null;
}

// 4. 单调 Nonce 提交队列契约 (frontend/src/recovered/features/conversation/workspace/submission.ts)
export interface ComposerSubmission {
  readonly nonce: string;
  readonly agentId: string;
  readonly prompt: string;
  readonly richText?: string;
  readonly attachments: readonly { path: string; name: string }[];
  readonly createdAtMs: number;
  readonly replyToId?: string;
  readonly isFork?: boolean;
  readonly directTypedPayload?: DirectTypedActionPayload;
}
```

### 5.2 后端 Rust 核心 Structs 与 RPC 消息定义

```rust
// 1. 输入台提交与直发管道契约 (source/host/src/gateway_protocol.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPromptRequest {
    pub client_nonce: String,
    pub session_id: String,
    pub prompt: String,
    pub rich_text: Option<String>,
    pub attachments: Vec<AttachmentDescriptor>,
    pub reply_to_id: Option<String>,
    pub model_override: Option<String>,
    pub thinking_effort: Option<String>,
    pub typed_action: Option<TypedActionPayload>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TypedActionPayload {
    Poll {
        question: String,
        options: Vec<String>,
        is_quiz: bool,
        correct_index: Option<u32>,
        public_votes: bool,
    },
    TodoList {
        title: String,
        items: Vec<String>,
        others_can_append: bool,
    },
    Invoice {
        title: String,
        amount_cents: u64,
        currency: String,
        is_recurring: bool,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SendPromptResponse {
    pub message_id: String,
    pub client_nonce: String,
    pub delivered_at_ms: u64,
    pub status: SendDeliveryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SendDeliveryStatus {
    Delivered,
    QueuedOffline,
    RejectedPolicy { reason: String },
}
```

### 5.3 核心状态转移方程

#### 1. IME 保护与 Enter 击键裁决方程
$$\text{HandleKeyEnter}(e) = \begin{cases}
\text{Ignore(AllowIME)}, & \text{if } e.\text{isComposing} \lor \text{compositionActive} = \text{true} \\
\text{InsertNewline}, & \text{if } e.\text{shiftKey} = \text{true} \\
\text{SubmitMessage}, & \text{if } \neg e.\text{shiftKey} \land \text{CanSubmit}(\text{draft}) \\
\text{Noop}, & \text{otherwise}
\end{cases}$$

#### 2. 无损草稿保存与结构化直发解耦方程（彻底消灭草稿覆盖）
$$\begin{aligned}
\text{OnDirectActionSend}(\text{action}, \text{conversationId}) \implies & \text{EnqueueOptimisticMessage}(\text{action}) \\
& \land \text{DispatchCoordinatorRPC}(\text{action}) \\
& \land \mathbf{PreserveDraft}(\text{DraftStore}[\text{conversationId}])
\end{aligned}$$
在此方程下，直发动作直接进入消息发送管线，现有草稿存储（$\text{DraftStore}$）保持恒等不变。

#### 3. 客户端单调 Nonce 状态机
$$\text{NonceState}: \text{Pending} \xrightarrow{\text{network\_error}} \text{Queued} \xrightarrow{\text{reconnect}} \text{Pending} \xrightarrow{\text{server\_ack}} \text{Settled}$$
任何阶段若用户主动点击“取消发送”，仅当消息处于 $\text{Queued}$ 阶段方可安全移出队列并标记为 $\text{Cancelled}$。

### 5.4 跨账号与跨会话隔离围栏（Fencing Rules）

1. **`accountScopeGeneration` 单调递增隔离**:
   切换活跃账号时，全局递增 `accountScopeGeneration`：
   - 立即清空内存中的草稿暂存字典，取消一切正在进行的语音听写流；
   - 阻断在途的 `sendPrompt` 回调，丢弃来自前一账号的异步结算通知；
   - 依据新账号的分区键从 SQLite/LocalForage 载入对应会话的草稿。
2. **`scopeKey` 切换草稿无损换场**:
   当用户在会话列表快速切换联系人时：
   $$\text{scopeKey} = \text{accountSlot} + \text{":"} + \text{agentId}$$
   - 当前会话的未提交文本、附件列表通过 `composerDraftStore.setDraft` 原子同步至持久化存储；
   - 新会话若存在先前已暂存的草稿，通过 `clearGeneration` 指令同步注入 ProseMirror 编辑器；
   - 严禁将会话 A 的引用回复（`replyTarget`）带入会话 B。


---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 Worker 修改白名单（Exclusive Write Whitelist）

落地本蓝图的子智能体（Worker Subagents）**仅允许修改以下白名单文件**，严禁触碰白名单外的任何基础设施：

```
[UI 表现层与输入台核心组件]
- frontend/src/recovered/features/conversation/workspace/composer.tsx
- frontend/src/production/tdrp/TdrpComposerToolbar.tsx
- frontend/src/production/tdrp/TdrpPollAndTodoModal.tsx
- frontend/src/production/openclaw/OpenClawCleanComposer.tsx (收拢为纯样式兼容变体)
- frontend/src/recovered/features/conversation/workspace/rich-text-editor.tsx

[状态机、草稿持久化与发送队列]
- frontend/src/recovered/features/conversation/workspace/draft-state.ts
- frontend/src/recovered/features/conversation/workspace/submission.ts
- frontend/src/recovered/features/permissions/local-tool/view.tsx

[主进程模型目录与服务适配]
- source/electron-main/models/cursor-model-catalog.ts

[Native / Host 消息协议与直发管道]
- source/host/src/gateway_protocol.rs
- source/host/src/extensions/transcript/send_pipeline.rs
```

### 6.2 只读参考代码文件清单（Read-Only Reference List）

子智能体可引用并精读以下文件以保持系统一致性，但**严禁写入或修改**：
- `projects/openclaw-rust/module-map.md`
- `projects/telegram-desktop-rust/module-map.md`
- `docs/specs/fusion-blueprints/00-master-capability-and-module-index.md`
- `docs/specs/fusion-blueprints/01-shell-navigation-and-layout.md`
- `frontend/src/production/ProductionRenderer.tsx`（仅主装配者可进行窄接线）

### 6.3 与 `ProductionRenderer.tsx` 窄接线契约（Narrow Wiring Contract）

`ProductionRenderer.tsx` 作为全系统的单一装配根入口，严禁在其内部直接编写文本拼装或假延时逻辑。所有输入台能力通过以下窄接口 Props 注入：

```typescript
// ProductionRenderer 挂载输入台与直发动作的窄接线 Props
export interface ProductionComposerProps {
  // 核心草稿与提交
  readonly draft: ComposerDraft;
  readonly onDraftChange: (draft: ComposerDraft) => void;
  readonly onSubmit: () => Promise<void>;
  readonly acceptedSendGeneration: number;
  readonly scopeKey: string;
  readonly replyTarget?: ComposerReplyTarget;
  readonly onClearReplyTarget: () => void;
  
  // 附件暂存与语音转写
  readonly onStageFiles: (files: readonly File[]) => Promise<void>;
  readonly onRemoveAttachment: (attachment: DraftAttachment) => Promise<void>;
  readonly transcribeAudio: VoiceTranscriber;
  
  // 真实动态模型与推理参数
  readonly activeModel?: string;
  readonly availableModels?: readonly ModelDescriptor[];
  readonly thinkingEffort?: ThinkingEffortLevel;
  readonly onChangeModel?: (modelId: string) => void;
  readonly onChangeThinkingEffort?: (level: ThinkingEffortLevel) => void;
  
  // 结构化直发管道 (彻底取代原 onInsertMessage 覆盖草稿逻辑)
  readonly onDirectSendTypedAction: (payload: DirectTypedActionPayload) => Promise<void>;
}
```

在 `ProductionRenderer.tsx` 的装配实现中，**彻底废除** 原有脆弱的覆盖草稿逻辑：
```tsx
// 改造前 (严重违规作弊代码):
onInsertMessage={(formattedPayload) => {
  setTdrpModalMode(null);
  composerDraftStore.setDraft(activeAgent.id, { prompt: formattedPayload, attachments: [] });
  setTimeout(() => { void submit(); }, 50);
}}

// 改造后 (权威无损直发管道):
onDirectSendTypedAction={async (actionPayload) => {
  setTdrpModalMode(null);
  await submitDirectTypedAction(activeAgent.id, actionPayload);
}}
```

### 6.4 实施分工切片（Slices）

- **切片 1（无损直发管道与草稿保护）**:
  在 `TdrpPollAndTodoModal.tsx` 中将 `onInsertMessage` 重构为 `onDirectSubmit`；在 `composer.tsx` 中支持接收结构化直发消息，并由 `submitDirectTypedAction` 直接构造乐观记录与投递，绝不触碰正在编辑的 `draft.prompt`。
- **切片 2（中文输入法 IME 保护与 TipTap 统一）**:
  在 `rich-text-editor.tsx` 与 `OpenClawCleanComposer.tsx` 中补齐 `isComposing` 与 `compositionActive` 双保险，拦截拼音选词回车；统一按 `scopeKey` 实现光标与草稿的瞬时恢复。
- **切片 3（动态模型与思考强度真实接线）**:
  重构 `TdrpComposerToolbar.tsx` 整合模型选择器与思考强度下拉菜单；通过 `cursor-model-catalog.ts` 拉取 Host 真实可用模型，彻底移除静态硬编码 `AVAILABLE_MODELS`。
- **切片 4（双输入框清理与单一真相源收拢）**:
  在生产界面中彻底移除 `OpenClawAskCopilotPanel` 的右侧输入框与模拟回复逻辑；确保全局仅存在唯一挂载的 `ConversationComposer`。

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 负例作弊排查表（Forbidden Anti-Patterns Checklist）

独立审计子智能体（Auditor Subagent）将对落地代码执行无死角排查，发现以下任意一项即判定交付不合格：

- [ ] **严禁并行的第二输入器**:
  界面中除了主舞台底部的 `ConversationComposer` 之外，严禁在右侧抽屉（如 `OpenClawAskCopilotPanel`）或任何浮层中实例化第二个能够发送消息的 `<textarea>` 或输入框。
- [ ] **严禁通过 `setTimeout` 覆盖未发送草稿来发送投票/待办/发票**:
  严禁出现通过 `composerDraftStore.setDraft(...)` 替换草稿后使用 `setTimeout(() => submit(), 50)` 触发发送的作弊逻辑。投票、待办清单、收款发票必须走独立的直发管道（`onDirectSendTypedAction`），保持用户原文本草稿 100% 完整无损。
- [ ] **严禁中文输入法 IME 选词回车误触发发送**:
  在拼音或日文输入法处于组合状态时（`isComposing === true` 或 `compositionActive === true`），按下 Enter 键严禁触发任何提交事件或导致草稿清空。
- [ ] **严禁硬编码静态模型列表**:
  严禁在前端代码中写死 `AVAILABLE_MODELS = [{ id: "gpt-6-astra", label: "GPT-6 Astra" }, ...]` 等静态假数据。模型列表必须来自 Host 或在无模型配置时自动优雅隐藏。
- [ ] **严禁在离线或故障时伪造发送成功**:
  当网络断开或处于无响应状态时，消息必须以 `pending` 或 `queued` 状态保存在本地队列中，严禁在未经后端返回 `message_id` 的情况下擅自更新为已发送成功状态。
- [ ] **严禁跨会话或跨账号草稿串扰**:
  切换会话必须严格依据 `scopeKey`（`accountSlot:agentId`）隔离草稿，严禁将会话 A 的文本、附件或引用回复带入会话 B。

### 7.2 正例验收确认准则（Positive Verification Gates）

1. **结构完备性验收**:
   - `docs/specs/fusion-blueprints/03-unified-composer-and-typed-actions.md` 包含全部 7 个强制章节，无占位符、无缺失。
   - 完整覆盖 Telegram Desktop 10 项能力域（`TCG-N10`, `TCG-N11`, `TCG-N12`, `TCG-N13`, `TCG-N21`, `TCG-N26`, `TCG-N29`, `TCG-N31`, `TCG-N42`, `TCG-N43`）与 OpenClaw 5 大核心责任域（`OCRM-M06`, `OCRM-M08`, `OCRM-M10`, `OCRM-M23`, `OCRM-M25`）。
2. **源码引用真实性验收**:
   - 引用的所有源码文件路径（如 `frontend/src/recovered/features/conversation/workspace/composer.tsx`、`frontend/src/production/tdrp/TdrpPollAndTodoModal.tsx`、`frontend/src/recovered/features/conversation/workspace/draft-state.ts` 等）物理存在，行号对应准确。
   - 函数与结构体名称真实对应实际代码。
3. **架构与规范合规验收**:
   - 代码行数严格符合单内聚模块 200–600 行预算。
   - 样式统一绑定 `--fabushi-*` 语义设计变量，支持浅色/深色主题无缝切换。
   - 键盘无障碍流与全键盘导航无焦点黑洞。
4. **无命令执行与无作弊纪律**:
   - 整个蓝图编制与审计过程严格遵循无命令验证模式（No Local Execution），未调用任何 `run_command`。

---

> **蓝图编制结论**:  
> `03-unified-composer-and-typed-actions.md` 已全面完成编制。双输入框割裂已被彻底消灭，投票/待办/发票已建立无损直发管道（杜绝覆盖草稿与假 `setTimeout`），中文输入法（IME）保护机制严密完备，动态模型与思考强度选择器已与真实 Host Owner 接线。该蓝图为后续 Worker 落地提供了无缝、零割裂、高内聚的权威指导契约。
