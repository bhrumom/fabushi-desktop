# UFX 基线问题与验收重点

只读源审查；基线 7f6548d42。以下不是运行时验收，不宣称已修复。

1. P1 伪造执行结果：OpenClawAskCopilotPanel 本地 setTimeout 输出成功/调度/连接器就绪；ProductionRenderer 装配仅传 onClose，无真实后端。删除伪执行链，所有能力状态必须来自真实 owner。
2. P1 模型/权限控件缺少真实接线：OpenClawCleanComposer 定义 hardcoded AVAILABLE_MODELS、默认 GPT-6 Astra/Medium/allowlist；ProductionRenderer 的该组件装配不传 model/onChangeModel/thinkingLevel/onChangeThinkingLevel/securityGate/onChangeSecurityGate。视觉选择不能证明实际生效；恢复 canonical editorProviders 与权限 owner，按真实 provider 能力渲染。
3. P1 输入器功能回退风险：同一 ProductionRenderer 的 Human 分支传 acceptedSendGeneration、replyTarget、onClearReplyTarget、onRemoveAttachment、scopeKey 给 ConversationComposer，Agent 分支替换成 OpenClawCleanComposer 未传这些合同。必须逐项检查 accepted clear、回复、附件、scope/草稿、流式取消、IME/语音等；不能只换外观。
4. P2 分享语义错误：OpenClawTopHeader 文案“分享会话”，传入 handler 却选 displayedEntries 最后一条非 card 打开 forwardTarget。应明确“转发消息”并定位所选消息，或走真正会话分享合同；不允许一键意外分享最后私密内容。
5. P2 静态/无作用导航：OpenClawTopHeader 显示默认 workspace 面包屑及缺少实际 back/forward handler 的禁用箭头。按有意义的当前对象动作收口；不保留纯装饰控件。

建议 first-batch oracle：断网时没有默认 ready；实际模型切换收到配置 owner 确认后才反映；回复 target 与附件删除正确；发送成功清空对应会话草稿，失败保留；切换账号后迟到回复/旧草稿不写入新账号；分享动作标注准确范围。

CLI 启动记录：首次 xhigh 被 gemini-4-argon 拒绝，根会话未创建；auto 模式账号不可用。第二次保留原模型，使用其可支持 high（复杂任务 adapter fallback）及 accept-edits，实际 init permission_mode=request-review。根 conversation=6989f3d7-a8d0-4fa9-8c9b-cda7b70c9f88；teamwork coordinator=85e84663-8255-4b2c-8a64-337f2a878b15。expanded_commands 确认 teamwork 为 system command；初始 team workspace 正确。尚不等于 worker 开始修改或测试通过。

---

## 候选快照 `df66fb9df` 深度审查发现与收敛要求（杜绝模块割裂）

对 `df66fb9df`（`wip(ux): snapshot unverified Antigravity candidate for independent review`）的源码审计表明，虽然移除了 `setTimeout` 假回复并将 Agent 输入器切回了 `ConversationComposer`，但产品体验仍然存在严重的“三套系统硬拼凑”割裂问题，必须在统一收口中彻底解决：

1. **P1 会话顶栏按 `activeIsHuman` 精神分裂（UI 割裂）**：
   - `ProductionRenderer.tsx` 仍使用 `activeIsHuman ? <div className="sand-chat-header"> : <OpenClawTopHeader>` 将会话舞台切为两套截然不同的顶栏与交互布局。切换真人与助手会话时顶栏跳变，且无法自然承载“真人 + 多个 Agent”混合群聊。
   - **收敛要求**：废除按 `activeIsHuman` 分叉两套顶栏的做法，统一为单一 canonical Header 骨架，仅通过 typed capability actions 注入通话、人机交接（Ask Agent）、搜索与详情开关。

2. **P1 右侧 7 个抽屉互相打架 + `OpenClawAskCopilotPanel` 硬编码“假离线”规避**：
   - 在 `ProductionRenderer.tsx` 中，`OpenClawAskCopilotPanel` 被写死为 `gatewayAddress="ws://127.0.0.1:18789"`、`isGatewayOnline={false}`、`gatewayStatusText="未连接 (未知)"`。这是把“硬编码在线”替换成“硬编码离线”来应付静态规则检查，即使真实后端已连接也永远显示未连接。
   - 同时右侧并存 `OpenClawAskCopilotPanel`、`TdrpConversationOverlay` / `TdrpProfileDetailsDrawer`、`ConversationOutlinePanel`、`AsyncTasksPanel`、`AgentSettingsPanel`、`GroupMembersPane`、`RoutinesInfoPane`、`ChannelsInfoPane` 共 7 类抽屉/浮层，交互入口与状态管理严重碎片化。
   - **收敛要求**：彻底移除 `OpenClawAskCopilotPanel` 及顶栏“布局”按钮；将所有右侧详情面板收敛为**唯一的上下文详情栏（Context Inspector）**，按对象类型组织分段（资料/成员与权限、共享媒体/大纲、助手模型/技能/权限、子任务与定时自动化、外部渠道），所有连接状态直连真实 owner（`transport` / `coordinatorPort` / `runtimeDetail`）。

3. **P1 一级导航“任务（Tasks）”在真人会话或未选会话时点击失效，且左轨与列表过滤栏职责重叠**：
   - `ProductionRenderer.tsx` 中左轨 `onSelectDomain` 写为 `else if (domain === "tasks" && activeAgent != null && !activeIsHuman) openAsyncTasks(activeAgent.id)`。当用户正在看真人聊天（`activeIsHuman === true`）或尚未选中会话时，点击左轨“任务”图标毫无反应。
   - 左轨 `agents` / `messages` 与中间列表顶部的 `FabushiCollectionFilterBar`（全部 / 未读 / 联系人 / 群组 / 机器人）干了同一件事。
   - **收敛要求**：点击左轨 `tasks` 必须无条件切换到一级全局“任务与自动化中心”视图，跨会话汇总所有待授权、进行中、已完成/失败任务与定时任务；左轨负责一级产品领域切换，中间列表顶部 Filter Bar 仅负责“会话（Messages）”领域内的视图筛选。

4. **P1 投票/待办（Poll/Todo）与欢迎页 Prompt 靠 `setTimeout(..., 50)` + 覆盖草稿发送**：
   - `TdrpConversationOverlay` 的 `onInsertMessage` 直接调用 `composerDraftStore.setDraft(activeAgent.id, { prompt: formattedPayload, attachments: [] })` 并用 `setTimeout(() => { void submit(); }, 50)` 发送；`OpenClawWelcomeHero` 的 `onSelectPrompt` 同样使用 50ms 定时器。
   - 这不仅会直接覆盖并丢失用户输入框内写了一半的草稿，还会引发 React 异步状态竞态。
   - **收敛要求**：富交互卡片（Poll / Todo / Invoice）与快捷 Prompt 必须走独立的直发通道或无损草稿挂载，严禁覆盖既有文本草稿，严禁使用 `setTimeout(..., 50)` 触发 `submit()`。

5. **P2 分享会话在缺少链接时回退复制 `window.location.href`**：
   - `ProductionRenderer.tsx` 的 `onShare` 写为 `(activeAgent.raw as any)?.shareUrl ?? (activeAgent.raw as any)?.inviteLink ?? window.location.href`。在 Electron 桌面端，`window.location.href` 是本地 `file://` 或 `localhost` 渲染器地址，复制给用户毫无意义且无任何反馈提示。
   - **收敛要求**：仅在对象拥有真实 `shareUrl` / `inviteLink`（或可由后端生成的真实分享链接）时展示/启用分享动作，复制成功后弹出明确 Toast 提示，禁止回退到 `window.location.href`。
