# Fabushi 统一融合蓝图 05：一级全局任务与自动化中心 + 子代理真实状态机
# Volume 05: Global Tasks, Automations, and Subagents

> **规范编号**: `FBCP-BLUEPRINT-05` / `UFX-BLUEPRINT-05`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 解除左轨 `tasks` 点击依赖当前会话 `!activeIsHuman` 的门控死锁，建立一级全局跨会话任务与自动化中心（Global Cross-Session Tasks Surface）。
> 2. 实现 7 态真实生命周期投影（`pending` -> `running` -> `waiting_auth` / `waiting_connection` -> `running` -> `completed` / `failed` / `cancelled`），严格映射 Host 真实 attempt、更新时间戳与失败诊断信息，彻底杜绝 UI 虚假成功/失败定时器。
> 3. 基于显式元数据（`parentId` / `taskId` / `type`）实现子代理层级归属，彻底废除基于字符串名称（如 `name.includes("executor")`）过滤子代理的错误做法，保障同名用户助手与独立固定历史一级可达。
> 4. 实现 Cron 定时调度（5-field + `@every` + IANA/TZ）、触发器（Webhook / Outside Events）、审批门禁（Approval Gates）与历史产物（Artifacts）的一体化管理。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **一级全局任务视图**<br>(Global Tasks Center) | `ui/src/app-navigation.ts` (tasks route)<br>`src/sessions/subagent-tracker.ts` | `storage/storage_account.cpp` (background progress)<br>`Telegram/SourceFiles/api/api_send_progress.cpp` | `frontend/src/production/ProductionRenderer.tsx:1465,4140`<br>`frontend/src/recovered/features/agent-info/async-tasks/provider.ts`<br>`frontend/src/recovered/features/agent-info/async-tasks/view.tsx` | `GlobalTasksWorkspace`<br>`frontend/src/production/tasks/`<br>`AsyncTasksProvider` (全局聚合) |
| **7 态真实生命周期投影**<br>(Real Lifecycle FSM) | `src/process/worker-state.ts`<br>`src/agents/harness/acp.ts`<br>`packages/worker-runtime/` | `api/api_sending.cpp` (state enum)<br>`history/view/controls/history_view_scheduled_box.cpp` | `frontend/src/recovered/features/agent-info/async-tasks/provider.ts:1-25,87-121`<br>`source/host/src/runner/subagent_runtime.rs:58-86`<br>`source/host/src/runner/stream_attempt.rs` | `AsyncTasksProvider`<br>`source/host/src/runner/background_work.rs` |
| **子代理层级归属**<br>(Subagent Lineage & Attribution) | `src/agents/multi-agent/lineage.ts`<br>`src/routing/subagent.ts`<br>`packages/agent-core/` | `data/data_thread.cpp`<br>`data/data_forum_topic.cpp` | `frontend/src/production/model.ts:145-162,198-202`<br>`frontend/src/production/tdrp/tdrp-folder-filter.ts:7,18,30`<br>`source/host/src/runner/subagent_runtime.rs:10-21` | `model.ts:projectRendererAgent`<br>`tdrp-folder-filter.ts`<br>`source/host/src/runner/subagent_runtime.rs` |
| **任务卡原会话双向联动**<br>(Task Card & Session Jump) | `src/session-cards/task-card.ts`<br>`src/interactive/a2ui.ts` | `history/history_item.cpp` (rich task card)<br>`api/api_rich_tasks.cpp` | `frontend/src/recovered/features/agent-info/async-tasks/view.tsx:75-112`<br>`frontend/src/production/tdrp/TdrpInteractiveMediaCards.tsx` | `TaskCardInTranscript`<br>`GlobalTasksWorkspace` 相互跳转 |
| **定时任务与自动化**<br>(Cron & Routines Scheduling) | `src/cron/schedule.ts:1-367`<br>`src/cron/schedule-identity.ts:1-150`<br>`src/cron/delivery-plan.ts:1-236`<br>`src/cron/session-reaper.ts:1-341` | `history/view/controls/history_view_scheduled_box.cpp`<br>`api/api_sending.cpp` (`SEND-SCHEDULED`) | `frontend/src/recovered/features/automations/routines/`<br>`source/host/src/automations/automation.rs:1-88`<br>`source/host/src/automations/automation_schedule.rs`<br>`source/host/src/automations/automation_trigger.rs` | `source/host/src/automations/`<br>`RoutinesController`<br>`GlobalTasksWorkspace` Tab |
| **心跳与后台 Polling**<br>(Heartbeat & Silent Poll) | `src/auto-reply/heartbeat.ts:1-207`<br>`src/auto-reply/command-detection.ts:1-108` | `data/data_lastseen_status.h`<br>`core/application.cpp` | `source/host/src/runner/heartbeat.rs`<br>`source/host/src/runner/sand_auto_review.rs:51-55` | `source/host/src/runner/heartbeat.rs`<br>`CloudWorkerProjection` |
| **长运行 Worker 与云资源**<br>(Background Work & Cloud Workers) | `src/worker/worker-pool.ts`<br>`packages/worker-runtime/`<br>`extensions/crabbox/` | `storage/storage_account.cpp`<br>`data/data_file_origin.h` | `source/host/src/runner/background_work.rs`<br>`source/host/src/runner/turn_run_shell.rs`<br>`source/node-agent-coordinator/src/cloud_worker_projection.rs` | `source/host/src/runner/background_work.rs`<br>`cloud_worker_projection.rs` |
| **安全审批门禁**<br>(Approval Gates & SandAutoReview) | `src/security/approvals.ts`<br>`src/audit/action-audit.ts` | `window/window_lock_widgets.cpp`<br>`core/sandbox.cpp` | `source/host/src/runner/sand_auto_review.rs:1-60`<br>`source/host/src/runner/auto_review_gate.rs`<br>`contracts/automation/cross-platform-journeys.json:47-57` | `source/host/src/runner/sand_auto_review.rs`<br>`ApprovalGateModal` |
| **任务产物与导出**<br>(Artifacts & Output Receipts) | `src/snapshot/`<br>`src/trajectory/`<br>`packages/workboard-contract/` | `export/export_manager.cpp`<br>`storage/localstorage.cpp` | `frontend/src/recovered/features/conversation/workspace/conversation-outline-view.tsx`<br>`source/host/src/runner/turn_observation.rs` | `TaskArtifactsSection`<br>`source/host/src/extensions/transcript/` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **OpenClaw 侧**：
  - 子代理编排与执行由 `src/agents/multi-agent/lineage.ts` 与 `src/routing/subagent.ts` 管理，生成 `parent_request_id`、`root_parent_request_id` 及 `parent_agent_tool_call_id`。状态机定义在 `src/process/worker-state.ts`，严格区分 `pending`, `running`, `waiting_auth`, `completed`, `failed`, `cancelled` 6 态，重试由 `attempt: number` 驱动，错误附带 `error: { kind, message, recoverable }`。
  - 定时调度由 `src/cron/schedule.ts`（blob `b0e8e7d25211b01204330abd0c5765b355c4582e`）实现标准 5-field cron 表达式、`@every` 间隔以及 `CRON_TZ=`/`TZ=` 时区转换，通过 `compute_next_run_at` 保证调度单调性；`schedule-identity.ts` 产生 7 天运行概览；`session-reaper.ts` 执行基于 `sessionRetention` 的孤儿会话与产物安全清理。
  - 审批门禁通过 `src/security/approvals.ts` 拦截高危工具调用，超时默认 10 分钟（`SAND_AUTO_REVIEW_APPROVAL_TTL_MS = 600,000`）。
- **Telegram Desktop 侧**：
  - 后台任务与文件上传/下载由 `storage/storage_account.cpp`、`storage/file_upload.cpp` 与 `api/api_send_progress.cpp` 管理，通过 `api_sending.cpp` 与 `history_view_scheduled_box.cpp` 维护预定消息（`SEND-SCHEDULED`）；`api_rich_tasks.cpp` 与 `api_todo_lists.cpp` 将任务直接挂载到聊天历史中。

#### Q2（现有链路定位）
- **全局任务导航**：
  `FabushiPrimaryNavRail.tsx` 点击 `tasks` -> 触发 `ProductionRenderer.tsx:4140` 中的 `onSelectDomain`。
- **任务状态加载与订阅**：
  `ProductionRenderer.tsx:1465` `openAsyncTasks(agentId)` -> `AsyncTasksProvider.snapshotsFor(agentId)` (`async-tasks/provider.ts:57`) -> `AsyncTasksCoordinator.getAsyncTasks` -> `source/node-agent-coordinator` -> Host Rust `runner/subagent_runtime.rs` 与 `runner/background_work.rs`。
- **子代理归集过滤**：
  `frontend/src/production/model.ts:155`（`projectRendererAgent`）-> 提取 `parentId`、`taskId`、`origin`、`kind`、`isSubagent` -> 传入 `tdrp-folder-filter.ts:18`（`filterAgentsByFolder`）过滤一级会话列表。
- **定时自动化**：
  `frontend/src/recovered/features/automations/routines/view.tsx` -> `RoutinesController` (`controller.ts`) -> `source/host/src/automations/automation.rs`（`AutomationRecord`, `AutomationRun`, `AutomationConfig`）-> SQLite/文件系统存储。

#### Q3（断点精确诊断）
1. **左轨点击死锁断点**（`ProductionRenderer.tsx:4140`）：
   ```tsx
   else if (domain === "tasks" && activeAgent != null && !activeIsHuman) openAsyncTasks(activeAgent.id);
   ```
   **诊断**：当且仅当用户处于非真人助手会话（`activeAgent != null && !activeIsHuman`）时，点击左轨 `tasks` 才会打开当前助手的右侧抽屉；如果用户当前在看真人私聊、群聊，或刚启动未选中任何会话时，点击左轨 `tasks` **完全没有任何反应**。用户无法在全局层面上看到所有跨会话的任务集合与审批事项。
2. **状态提供者局部孤岛**（`frontend/src/recovered/features/agent-info/async-tasks/provider.ts:57`）：
   `snapshotsFor(agentId: string)` 仅支持单 `agentId` 维度的快照订阅，缺少聚合全工作区所有会话任务的 `globalSnapshotHandle`。
3. **名称过滤字符串硬编码隐患**（历史残留问题）：
   虽然 `model.ts` 已支持元数据判定，但历史上部分分支曾使用 `name.toLowerCase().includes("executor")` 粗暴隐藏会话，导致用户自定义的以 "executor" 命名的正常助手被误杀。必须在蓝图中确立铁律。
4. **Heartbeat 未接线断点**（`cron-automations-heartbeat-scoped.md:58-61`）：
   `source/host/src/runner/heartbeat.rs` 存在纯状态定义，但未在生产 Coordinator 的事件循环中建立调度消费，处于 `wiring=not-wired` 状态。

#### Q4（状态机与安全防护）
- **7 态单调性转移方程**：
  $$\text{State} \in \{\text{Pending}, \text{Running}, \text{WaitingAuth}, \text{WaitingConnection}, \text{Completed}, \text{Failed}, \text{Cancelled}\}$$
  - $\text{Pending} \xrightarrow{\text{host\_spawn}} \text{Running}$
  - $\text{Running} \xrightarrow{\text{tool\_needs\_approval}} \text{WaitingAuth}$
  - $\text{WaitingAuth} \xrightarrow{\text{user\_approve}} \text{Running}$
  - $\text{WaitingAuth} \xrightarrow{\text{user\_deny / ttl\_expired}} \text{Cancelled}$
  - $\text{Running} \xrightarrow{\text{transport\_disconnected}} \text{WaitingConnection}$
  - $\text{WaitingConnection} \xrightarrow{\text{transport\_reconnected}} \text{Running}$
  - $\text{Running} \xrightarrow{\text{attempt\_failed\_recoverable}} \text{Running } (\text{attempt} \leftarrow \text{attempt} + 1)$
  - $\text{Running} \xrightarrow{\text{final\_error}} \text{Failed}$
  - $\text{Running} \xrightarrow{\text{user\_abort}} \text{Cancelled}$
  - $\text{Running} \xrightarrow{\text{turn\_settle}} \text{Completed}$
- **跨账号与跨会话 Fencing**：
  在 `accountScopeGeneration` 改变时，清空当前内存中的所有非持久任务缓存；当切换会话时，正在执行的子任务通过 `taskId` 保持后台执行，但原会话内的 TaskCard 仅响应与当前 `conversationId` 绑定的更新事件；若收到旧账号或跨会话事件，直接丢弃（Fenced）。

#### Q5（UI/UX 极致融合落点）
- **全局任务入口**：左轨一级导航“任务（Tasks）”图标无条件打开中央主舞台的 `GlobalTasksWorkspace`。
- **三段式渐进披露**：
  - 第一层：状态分组标签（需要我处理 Waiting Auth / 进行中 In Progress / 已完成 Terminal / 定时自动化 Routines）。
  - 第二层：结构化任务卡片（发起者头像与名称、任务目标、7 态徽章、尝试次数、实时耗时、主操作按钮“跳转原会话”/“审批授权”/“重试”）。
  - 第三层：展开详情抽屉或行内折叠（详细步骤 Step Progress、子代理树 Lineage Tree、工具调用参数、异常诊断栈、产物下载链接）。
- **设计规范与无障碍**：完全绑定 `--fabushi-*` 语义 Token，支持深浅色无缝自适应；状态徽章包含符合 WCAG AA 标准的对比度文本与 `aria-live="polite"` 读屏支持；键盘支持上下方向键导航与 Enter 键快速审批/跳转。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-05-01** | 点击左轨“任务”图标在真人私聊、群聊或未选会话时毫无反应（任务功能死锁） | `frontend/src/production/ProductionRenderer.tsx:4140` | `onSelectDomain` 中硬编码了 `else if (domain === "tasks" && activeAgent != null && !activeIsHuman) openAsyncTasks(activeAgent.id)`，将全局任务入口劣化为“当前非真人会话的右侧临时抽屉开关”，逻辑互锁。 | **P0 阻断性缺陷** |
| **BUG-05-02** | 任务中心缺乏全局汇总视图，多 Agent 并行时用户必须逐个会话翻找任务进展 | `frontend/src/recovered/features/agent-info/async-tasks/view.tsx:120-142`<br>`provider.ts:57` | `AsyncTasksPanel` 设计初衷仅针对单个 Agent（`agentId: string`），缺少跨会话聚合（Cross-session Aggregator）架构，`AsyncTasksProvider` 未向前端暴露全局查询接口。 | **P1 架构割裂** |
| **BUG-05-03** | 任务卡与会话割裂，会话内缺少统一 TaskCard 投影，无法从任务直接溯源会话 | `frontend/src/recovered/features/agent-info/async-tasks/view.tsx:75-112` | 任务列表中的行组件 `AsyncTaskRow` 仅包含文本和时间，缺少 `conversationId` 导航回执，也缺少向会话主消息流反向发送 TaskCard 的通道。 | **P1 交互断层** |
| **BUG-05-04** | 子代理过滤历史曾存在名称字符串匹配陷阱，威胁同名用户助手可达性 | `projects/fabushi-communication-platform/current-owner-inventory.md`<br>`frontend/src/production/model.ts:155-162` | 早期开发曾有将包含 "executor" 的名称强行视作内部代理的倾向。当前 `model.ts` 虽然已修复为依靠元数据，但尚未在所有分卷蓝图和测试基线中冻结该反作弊准则。 | **P1 规范风险** |
| **BUG-05-05** | 定时任务（Routines）独立孤岛于会话右侧抽屉，未纳入全局任务管理体系 | `frontend/src/recovered/features/automations/routines/view.tsx:18-32`<br>`source/host/src/automations/automation.rs` | Routines（Cron 定时任务与外接事件触发器）被局限在 `RoutinesInfoPane` 抽屉中，与异步执行中的长任务完全脱节，用户无法在一个统一的看板下查看“一次性长任务”与“循环定时任务”。 | **P2 体验割裂** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 架构设计与单一权威归属

为消除上述割裂，本蓝图确立 **一级全局任务与自动化工作区（GlobalTasksWorkspace）** 架构。

```
                       ┌──────────────────────────────────────────────┐
                       │   FabushiPrimaryNavRail (domain === "tasks") │
                       └──────────────────────┬───────────────────────┘
                                              │ (无条件全局导航)
                                              ▼
┌─────────────────────────────────────────────────────────────────────────────────────────────┐
│                          GlobalTasksWorkspace (中央主舞台)                                  │
│  ┌───────────────────────────────────────────────────────────────────────────────────────┐  │
│  │ 分组过滤 Tabs: [全部任务] [需要我处理 (WaitingAuth)] [进行中 (Running)] [已完成] [定时自动化] │  │
│  └───────────────────────────────────────────────────────────────────────────────────────┘  │
│  ┌─────────────────────────────────────────┐  ┌──────────────────────────────────────────┐  │
│  │ 异步任务流卡片列表 (Active Tasks Stream) │  │ 定时与自动化列表 (Routines & Cron Hub)  │  │
│  │ - 7 态真实徽章 (Real Status Badge)      │  │ - 5-Field Cron 表达与 7 天预览           │  │
│  │ - Host Attempt & 错误诊断详情           │  │ - Webhook / Outside Events 触发器     │  │
│  │ - 子代理层级折叠树 (Lineage Tree)       │  │ - 运行历史 (Run History) 与输出产物     │  │
│  │ - [一键跳回原会话] [审批通过/拒绝]      │  │ - [新建/编辑/启用/禁用/删除]          │  │
│  └─────────────────────────────────────────┘  └──────────────────────────────────────────┘  │
└─────────────────────────────────────────────┬───────────────────────────────────────────────┘
                                              │ (双向数据同步与事件订阅)
                      ┌───────────────────────┴───────────────────────┐
                      ▼                                               ▼
┌───────────────────────────────────────────┐   ┌───────────────────────────────────────────┐
│     GlobalAsyncTasksManager (前端状态)     │   │     Host Automations & Runner (Rust 后端) │
│ - 跨会话 Task 聚合订阅 (Cross-Agent Cache)│   │ - source/host/src/automations/            │
│ - Account Scope Generation Fencing 围栏   │   │ - source/host/src/runner/subagent_runtime │
│ - TaskCard 与 Transcript 消息流双向投影   │   │ - source/host/src/runner/sand_auto_review │
└───────────────────────────────────────────┘   └───────────────────────────────────────────┘
```

### 3.2 模块拆解与 200–600 行内聚预算

遵循 `Same Owner != Same File` 原则，所有新增与重构模块均控制在 200–600 行预算内，严禁堆砌巨石文件：

1. **`frontend/src/production/tasks/GlobalTasksWorkspace.tsx`** (~380 行)
   - **职责**: 全局任务与自动化中心的主舞台容器。装配分组 Tabs、快捷统计计数、搜索过滤栏与主列表渲染。
   - **预算**: 350–450 行。
2. **`frontend/src/production/tasks/GlobalTaskCard.tsx`** (~280 行)
   - **职责**: 单个跨会话任务卡片组件。渲染 7 态徽章、尝试次数、开始与更新时间、错误折叠、一键跳转原会话按钮及审批操作按钮。
   - **预算**: 250–320 行。
3. **`frontend/src/production/tasks/global-tasks-store.ts`** (~320 行)
   - **职责**: 客户端跨会话全局任务缓存管理器。汇聚所有已加载 Agent 的 `AsyncTasksSnapshot`，监听后台事件并提供 `useSyncExternalStore` 订阅。
   - **预算**: 280–350 行。
4. **`frontend/src/production/tasks/RoutinesIntegrationTab.tsx`** (~310 行)
   - **职责**: 将既有 `frontend/src/recovered/features/automations/routines/` 中的 `RoutinesController` 引入全局任务中心的“定时自动化”Tab。
   - **预算**: 280–340 行。
5. **`frontend/src/production/ProductionRenderer.tsx` 窄接线** (修改量 < 40 行)
   - **职责**: 修复 `domain === "tasks"` 分支，解除 `!activeIsHuman` 限制，直接挂载 `GlobalTasksWorkspace`。


---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 渐进披露（Progressive Disclosure）层级设计

全局任务与自动化中心采用三层渐进披露架构，防止大量复杂的后台信息和日志噪音直接淹没用户：

```
┌────────────────────────────────────────────────────────────────────────────────┐
│ Level 1: 全局概览看板 (Global At-a-Glance)                                      │
│ - 状态统计胶囊: [待处理 2] [执行中 4] [已完成 18] [定时自动化 6]                 │
│ - 快速筛选器: [全部对象] [仅我的主要助手] [仅高危审批]                          │
├────────────────────────────────────────────────────────────────────────────────┤
│ Level 2: 任务实体卡片 (Task Summary Card)                                       │
│ ┌────────────────────────────────────────────────────────────────────────────┐ │
│ │ 🤖 智能研究员 (Research Agent) · 任务 #task_8921                            │ │
│ │ [执行中 (第 2 次尝试)] 分析 Q3 季报与行业竞争格局                            │ │
│ │ 进度: 正在调用 sand_browser_tools 抓取彭博行情...  (耗时 42s)               │ │
│ │ 动作: [查看原会话 ↗] [查看详情 ▾] [终止任务 ⏹]                              │ │
│ └────────────────────────────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────────────────────────────┤
│ Level 3: 深度上下文与审计抽屉 (Deep Inspector Drawer / Expanded Trace)          │
│ - 完整调用链路 (Trace Steps):                                                  │
│   1. [已完成] 读取本地历史数据库                                              │
│   2. [等待授权] 请求访问外部网络 (https://bloomberg.com) -> [通过]             │
│   3. [执行中] 子代理 #sub_01 (Browser Runner) 正在分析 DOM                    │
│ - 错误诊断与重试信息: (若失败显示精确原因与堆栈)                              │
│ - 产物接收区 (Artifacts): [下载 2026Q3_Report.pdf (2.4MB)]                     │
└────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 响应式断点规范（Responsive Breakpoints）

严格遵循统一体验规范的自适应断点标准：

1. **≥1200px（宽屏三栏模式）**：
   - 最左侧为 52px 导航轨（`FabushiPrimaryNavRail`）。
   - 中间列表区域展示全局任务流，卡片以双列或宽松单列展示，右侧常驻展开选中任务的详细 Execution Inspector 抽屉（宽度 360px–420px）。
2. **900px–1199px（中屏双栏模式）**：
   - 导航轨保持 52px。
   - 任务列表占据主要舞台。点击单个任务卡片上的“查看详情”时，以覆盖式抽屉（Drawer）自右侧滑出，带遮罩与焦点锁定，按 `Esc` 键平滑收回。
3. **<900px（移动/窄屏单栏模式）**：
   - 列表全屏展示。点击任务详情或跳转原会话执行全屏切换路由，顶栏提供清晰的“返回（Back）”按钮并保持状态不重置。

### 4.3 语义设计变量绑定（`--fabushi-*` Design Tokens）

严格禁止在组件中硬编码十六进制颜色或使用外部私有样式（如 `oc-*` 或未收口的 `sand-*`）：

| UI 语义元素 | CSS Variable 绑定 | 浅色模式回退值 (Light) | 深色模式回退值 (Dark) | 无障碍说明 |
|---|---|---|---|---|
| **任务工作区背景** | `var(--fabushi-surface-canvas)` | `#f8fafc` | `#0f172a` | 主页面底色 |
| **卡片背景** | `var(--fabushi-surface-card)` | `#ffffff` | `#1e293b` | 满足容器层级感 |
| **卡片边框** | `var(--fabushi-border-subtle)` | `rgba(0,0,0,0.08)` | `rgba(255,255,255,0.1)` | 区分边缘 |
| **待开始状态** | `var(--fabushi-status-pending)` | `#64748b` | `#94a3b8` | 中性灰色 |
| **执行中强调色** | `var(--fabushi-status-running)` | `#0284c7` | `#38bdf8` | 明晰天蓝色，动态呼吸灯 |
| **等待审批高亮** | `var(--fabushi-status-warning)` | `#d97706` | `#fbbf24` | 琥珀金黄色，引起注意 |
| **等待连接指示** | `var(--fabushi-status-connecting)`| `#ea580c` | `#fb923c` | 橙色警示 |
| **完成成功色** | `var(--fabushi-status-success)` | `#16a34a` | `#4ade80` | 清晰草绿色 |
| **失败严重色** | `var(--fabushi-status-failed)` | `#dc2626` | `#f87171` | 醒目红色，满足对比度 |
| **已取消色** | `var(--fabushi-status-cancelled)` | `#94a3b8` | `#64748b` | 浅灰中性色 |

### 4.4 无障碍（A11y）与输入法（IME）保护规范

1. **ARIA 状态与实时公告**：
   - 任务列表声明 `role="feed"`，每个任务项声明 `role="article"` 与 `aria-labelledby`。
   - 状态变更动态挂载 `aria-live="polite"`（普通进展）与 `aria-live="assertive"`（等待审批通知），确保屏幕阅读器能及时播报。
2. **键盘导航与焦点恢复（Focus Return）**：
   - 任务卡片支持 `Tab` 键依次聚焦各个交互按钮（“跳回会话”、“审批”、“折叠详情”）。
   - 从任务卡片点击“审批”打开模态弹窗后，焦点自动落入“同意”或“拒绝”按钮；弹窗关闭后，焦点精确恢复至原卡片上的触发按钮，防止焦点游离到页面顶端。
3. **中文输入法（IME）保护**：
   - 在 Routines 编辑器修改自动化 Prompt 或名称时，严格挂载 `isComposing` 状态锁。当按下 `Enter` 键确认汉字候选词时，拦截 `onKeyDown` 提交事件，只有非 Composition 状态下的 `Enter` 才允许触发保存。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 核心 TypeScript 数据模型与 Props 契约

```typescript
// 严格 7 态真实生命周期定义
export type GlobalTaskStatus =
  | "pending"
  | "running"
  | "waiting_auth"
  | "waiting_connection"
  | "completed"
  | "failed"
  | "cancelled";

export type GlobalTaskKind = "subagent" | "shell" | "cloud-agent" | "routine";

export interface GlobalTaskItem {
  readonly id: string;
  readonly kind: GlobalTaskKind;
  readonly label: string;
  readonly status: GlobalTaskStatus;
  readonly startedAtMs: number;
  readonly updatedAtMs?: number;
  readonly finishedAtMs?: number;
  readonly agentId: string;
  readonly agentName: string;
  readonly conversationId: string;
  readonly parentTaskId?: string;
  readonly subagentType?: string;
  readonly attempt: number;
  readonly progressPercent?: number;
  readonly currentActivity?: string;
  readonly detail?: string;
  readonly error?: {
    readonly code: string;
    readonly message: string;
    readonly recoverable: boolean;
  };
  readonly artifacts?: readonly {
    readonly id: string;
    readonly name: string;
    readonly sizeBytes: number;
    readonly mimeType: string;
    readonly downloadUrl: string;
  }[];
  readonly requiresApproval?: {
    readonly approvalId: string;
    readonly surface: "hostShell" | "boxShell" | "mcp" | "computer" | "automationWrite";
    readonly command: string;
    readonly riskLevel: "low" | "medium" | "high";
    readonly expiresAtMs: number;
  };
}

export interface GlobalTasksWorkspaceProps {
  readonly currentAccountId: string;
  readonly accountScopeGeneration: number;
  readonly onNavigateToConversation: (conversationId: string, messageId?: string) => void;
  readonly onApproveAction: (approvalId: string, decision: "allow-once" | "allow-always" | "deny") => Promise<void>;
  readonly onCancelTask: (taskId: string) => Promise<void>;
  readonly onRetryTask: (taskId: string) => Promise<void>;
}
```

### 5.2 Rust 后端事件契约（Host -> Coordinator -> UI）

```rust
// source/host/src/runner/subagent_runtime.rs 与 gateway_protocol
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HostTaskStatus {
    Pending,
    Running,
    WaitingAuth,
    WaitingConnection,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostTaskEvent {
    pub account_scope_generation: u64,
    pub task_id: String,
    pub parent_task_id: Option<String>,
    pub agent_id: String,
    pub conversation_id: String,
    pub status: HostTaskStatus,
    pub attempt: u32,
    pub current_activity: Option<String>,
    pub progress_detail: Option<String>,
    pub error_code: Option<String>,
    pub error_message: Option<String>,
    pub updated_at_ms: i64,
}
```

### 5.3 隔离围栏（Fencing Rules）与防竞态规则

1. **`accountScopeGeneration` 围栏**：
   - 前端 `GlobalTasksStore` 维护当前的 `accountScopeGeneration: number`。
   - 当接收到任何推送事件时，核对其附带的 `accountScopeGeneration`：
     ```typescript
     if (event.accountScopeGeneration !== this.currentAccountScopeGeneration) {
       console.warn(`[TasksFencing] Dropping stale task event from generation ${event.accountScopeGeneration}`);
       return; // 坚决丢弃迟到事件
     }
     ```
2. **终态不可逆（Terminal State Monotonicity）**：
   - 任务一旦进入终态（`completed`, `failed`, `cancelled`），除非收到带有递增 `attempt > previousAttempt` 的明确重新启动事件，否则任何网络乱序到达的 `running` 或 `waiting_auth` 事件必须被拒绝更新。
3. **审批时效门禁（Approval TTL Enforce）**：
   - 审批项遵循 `SAND_AUTO_REVIEW_APPROVAL_TTL_MS`（10 分钟）。当本地系统时钟判定 `now > expiresAtMs` 时，本地立即将该审批态标注为已过期（`cancelled`），并向后端发起取消确认，禁止用户点击超期授权。

---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 任务切片划分

| 切片编号 | 切片目标 | 允许修改文件白名单 | 只读参考文件 | 验收准则 |
|---|---|---|---|---|
| **SLICE-05-A** | 解除左轨 Tasks 导航死锁与路由窄接线 | `frontend/src/production/ProductionRenderer.tsx` | `frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx`<br>`projects/fabushi-communication-platform/current-owner-inventory.md` | 点击左轨任务图标无条件切换到任务主舞台，不论当前是在真人聊、群聊还是空状态。 |
| **SLICE-05-B** | 构建全局任务工作区与卡片呈现 | `frontend/src/production/tasks/GlobalTasksWorkspace.tsx`<br>`frontend/src/production/tasks/GlobalTaskCard.tsx` | `frontend/src/recovered/features/agent-info/async-tasks/view.tsx`<br>`frontend/src/production/production.css` | 完整展现 7 态徽章、attempt 计数与错误折叠；每文件保持在 200–400 行。 |
| **SLICE-05-C** | 跨会话任务数据汇聚与状态机管理 | `frontend/src/production/tasks/global-tasks-store.ts`<br>`frontend/src/recovered/features/agent-info/async-tasks/provider.ts` | `source/host/src/runner/subagent_runtime.rs`<br>`source/node-agent-coordinator/src/cloud_worker_projection.rs` | 提供 `useSyncExternalStore` 订阅；实现 `accountScopeGeneration` 熔断隔离。 |
| **SLICE-05-D** | 定时任务 Routines 深度整合接入 | `frontend/src/production/tasks/RoutinesIntegrationTab.tsx` | `frontend/src/recovered/features/automations/routines/view.tsx`<br>`source/host/src/automations/automation.rs` | 在全局任务中心无缝展示定时 Cron 与外接事件触发器，支持就地启停与编辑。 |

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 正反例检查表（Anti-Cheating Checklist）

| 检查项 | 正确实现标准（Acceptance Passing） | 严禁作弊/违规行为（Forbidden Cheating） | 审计判定方式 |
|---|---|---|---|
| **全局任务导航** | 点击左轨 `tasks` 无条件切换到 `GlobalTasksWorkspace` | `if (activeAgent != null && !activeIsHuman)` 限制；点击无反应 | 切换至真人聊天，点击左轨“任务”，验证是否进入全局任务看板。 |
| **任务状态真实性** | 状态完全由 Host WebSocket/RPC 消息驱动，严格遵循 7 态投影 | 使用 `setTimeout` 假装等待 2 秒后自动变成“成功”或“失败” | 静态代码审查是否存在定时修改 status；断网测试是否卡在 waiting_connection。 |
| **重试计数** | 真实展示 `attempt` 字段（如第 2 次尝试）；无重试不展示 | 硬编码文本 `(第 2 次尝试)` 或固定显示动画 | 检查后端无重试时是否仅显示单次执行。 |
| **子代理识别** | 严格基于 `parentId`/`taskId`/`origin`/`kind` 元数据判断 | 使用 `name.includes("executor")` 字符串过滤 | 创建名为 "My Executor Bot" 的用户自定义助手，检查是否依然一级可见。 |
| **原会话跳转** | 点击卡片上的“跳转原会话”精确切换会话并高亮对应 TaskCard | 按钮为静态无回调占位；或点击跳往错误的首页 | 点击跳转，验证 `activeAgentId` 是否更新，Transcript 是否滚动到位。 |
| **账号隔离** | 切换账号后立即清空旧账号的任务卡片与后台通知 | 切换账号后旧账号的任务仍然停留在屏幕上或继续弹窗 | 切换用户账号，观察全局任务列表是否原子清空并重新拉取。 |
| **代码内聚预算** | 新增文件严格控制在 200–600 行以内，结构清晰 | 产生单个超过 1000 行的巨石文件，或直接全部塞入 `ProductionRenderer.tsx` | 统计新增文件行数：`wc -l frontend/src/production/tasks/*`。 |
