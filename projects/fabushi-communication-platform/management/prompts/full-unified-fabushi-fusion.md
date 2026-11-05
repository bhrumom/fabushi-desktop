# Fabushi 终极全量融合执行提示词（`UFX-FLOW-01`：全源码精读 $\rightarrow$ 全景能力索引与分模块蓝图 $\rightarrow$ 子智能体严格落地）

你现在担任 **Fabushi 首席产品架构师与总控编排者（Lead Architect & Orchestrator）**，工作区位于 `/Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105`（分支 `design/unified-fabushi-20261105`）。

你的目标是：严格按照根 [`AGENTS.md`](../../../../AGENTS.md) 中的 **`UFX-FLOW-01` 四阶段开发流程**，对 **Telegram Desktop（`TDRP-001`）**、**OpenClaw（`OCRM-001`）** 与 **Fabushi（`FBCP-001` / `UFX-001`）** 的全部源码进行深度阅读，**不遗漏任何一个 Telegram 和 OpenClaw 的产品功能**，编写全景能力索引并按模块制定包含职责、架构位置、状态机与极致 UI/UX 的详细融合方案（落盘到 `docs/specs/fusion-blueprints/`），再将方案拆解派发给子智能体严格实施并独立审计，打造出体验浑然一体的 Fabushi 产品。

---

## 阶段一：全量三方源码精读与零遗漏盘点（Exhaustive Tri-Source Reading）

**严禁在未通读源码前直接改代码或凭印象写方案！** 主控架构师必须借助只读调研子智能体（Research Subagents）严格按照**“四步精读法”**与**“源码五问”**执行：

### 1. 源码精读四步法
1. **宏观地图与账本核验（Macro Atlas & Ledger Audit）**：
   - 梳理三方全景索引，建立完整心智模型：
     - **Telegram 侧**：[`projects/telegram-desktop-rust/module-map.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/telegram-desktop-rust/module-map.md)、[`telegram-capability-graph.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/fabushi-communication-platform/telegram-capability-graph.md)（覆盖 `Telegram/SourceFiles` 40 个顶层目录与 50+ 个能力节点 `TCG-N01`..`TCG-N50+`）、`source-closure.md` 与 `dossiers/`；
     - **OpenClaw 侧**：[`projects/openclaw-rust/module-map.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/openclaw-rust/module-map.md)（覆盖 `OCRM-M01` 至 `OCRM-M32` 全部 32 个责任域）、`parity-ledger.json`（聚焦 141 项 `implemented` + `not-wired` 与 9 项 `not-started`）与 `dossiers/`；
     - **Fabushi 侧**：[`current-owner-inventory.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/fabushi-communication-platform/current-owner-inventory.md)、`architecture-map.md`、`canonical-screen-patterns.md`、`ui-information-architecture.md` 以及 `frontend/src/production/ProductionRenderer.tsx`。
2. **三方源码并排精读（Tri-Source Side-by-Side Reading）**：
   - 针对具体领域，同时打开并排阅读三方具体实现文件：
     - **Telegram Desktop 锁定源码（`@f985723b552b30fe1c17d377d57f44e61a15a8c4`）**：C++/Qt 实现中的数据结构、消息模型、事件过滤、UI 状态机与错误处理；
     - **OpenClaw 锁定源码（`@1440bb1771372334bea23d7642bb52deb7db1845`）**：TypeScript/Lit/Node 中的 Gateway RPC 协议、ACP 交互、Agent 调度、记忆检索、自动化与权限门禁；
     - **Fabushi 生产源码**：`frontend/src/production/`、`frontend/src/recovered/features/`、`source/host/`、`source/node-agent-coordinator/`、`native/mahayana-messaging/`、`contracts/` 中的现有实现与断点。
3. **端到端全链路调用追踪（End-to-End Call-Chain Tracing）**：
   - 追踪每一项能力从前端到后端的完整闭环：
     `UI Entrypoint (Click/Key/Drag) -> Producer -> Controller/Store -> IPC/RPC Bridge -> Host/Coordinator Rust Service -> Storage/Network Provider -> Event/Subscription Loop back to UI Consumer`
   - 必须记录真实的类名、函数名与文件路径，拒绝凭空臆造 API。
4. **强制回答“源码五问”（The Mandatory Source Five Questions）**：
   - 对任何被研究的模块或功能，必须用确凿的代码行号证据回答以下 5 个问题：
     - **Q1（上游真实现状）**：该功能在上游是如何实现的？核心状态机、数据模型与边界条件（超时、取消、重试、错误码）是什么？
     - **Q2（现有链路定位）**：Fabushi 现有 canonical owner 是谁？现有的生产调用链路走到哪一步？
     - **Q3（断点精确诊断）**：为什么该功能当前不可用？是 `implemented` 但 `not-wired`、是硬编码了假数据/假状态（如 `isGatewayOnline={false}` 或 `setTimeout` 假回复）、还是缺少状态传递或类型对接？（精确到文件和行号）
     - **Q4（状态机与安全防护）**：跨组件信任边界如何？断网、重连、切换账号（`accountScopeGeneration`）、切换会话（`session_key`）时如何隔离，防止迟到事件和跨会话污染？
     - **Q5（UI/UX 极致融合落点）**：在统一界面中它应当落在哪个标准位置？高频交互如何直达？低频配置如何通过渐进披露（Progressive Disclosure）收敛到唯一右侧 `ContextInspector`？如何绑定 `--fabushi-*` 语义 Token 与无障碍标准？

---

## 阶段二：全景能力索引与动态蓝图落盘（`docs/specs/fusion-blueprints/`）

严禁直接拍脑袋预设少数几个模块而删减功能！主控架构师必须在 `docs/specs/fusion-blueprints/` 目录下完成两层落盘：

### 1. 全景能力总索引（`00-master-capability-and-module-index.md`）
- 必须首先编写该文件，以二维矩阵严格对齐：
  - **OpenClaw 全量 32 个责任域（`OCRM-M01` 至 `OCRM-M32`）**（涵盖启动/引导、配置/Schema、认证/凭据/配对、安全沙箱、Agent执行、模型推理、会话ACP协议、上下文引擎、多Agent拓扑、Worker池、Webhook、通道路由、交互协议、适配器、浏览器/DOM控制、Cron自动化、Canvas/A2UI、节点网关、任务流编排、记忆QMD检索、项目看板、快照版本、媒体生成、语音TTS、会议Bot、插件MCP、技能包、终端CLI、存储DB、Doctor诊断自愈、遥测日志、桌面交付）；
  - **Telegram Desktop 全量 50+ 个能力域（`TCG-N01`..`TCG-N50+`）**（涵盖应用生命周期、多账号/多设备会话隔离、联系人/搜索、会话列表/文件夹/归档/角标、Stories发布与查看、消息历史/Gap恢复、发送排队、引用/回复/转发、Reactions、置顶横幅、论坛话题、草稿与IME保护、富文本编辑器、投票/待办/账单直发、音视频通话/屏幕共享/群语音房、群/频道管理权限与封禁、贴纸/GIF/Emoji、媒体查看器与缓存、Mini Apps容器、云安全与多协议代理等）；
  - **Fabushi 现有与新增 Canonical Owners 映射**。
  - 确保 100% 能力被分配到具体的模块蓝图分卷，**0 遗漏、0 孤岛**。

### 2. 不设数量上限的高内聚分卷蓝图（`docs/specs/fusion-blueprints/01..NN-*.md`）
- **严禁预设固定的模块数量上限（如生搬硬套 8 个模块）或将十几个独立子系统塞进同一份文件充数！**
- 必须严格按照 `docs/specs/fusion-blueprints/README.md` 规定的 **4 条硬性拆分与颗粒度标准** 动态拆分分卷（符合标准拆出多少份就是多少份）：
  1. **单卷领域密度上限（Anti-Cramming Cap）**：每份分卷蓝图最多只允许承载 **1～4 个强内聚能力域**，严禁把超过 4 个松散相关的能力域大杂烩式地塞进同一份文件。
  2. **独立状态机与独立 Schema 必拆准则**：凡拥有独立持久层表结构、独立网络/信令/加密协议、独立后台执行引擎（如 Cron、QMD 向量检索与 Wiki、STT/TTS 实时语音与会议 Bot、AI 多模态生成与媒体编辑器、大文件分片传输与 Instant View、收银台支付与星币礼物账本、端到端加密与本地安全锁/2FA/Passkeys、SQLite 迁移修复与全量记录导出、社群统计图表与反滥用举报等）或独立复杂交互表面的子系统，**必须独立成卷**！
  3. **单能力契约深度下限**：每个被覆盖的子系统在蓝图中都必须拥有完整的数学状态转移方程、完整的 TypeScript `interface`/Props、完整的 Rust `struct`/`enum`/RPC 契约与精确到行号的三方源码映射。凡因多域拥挤导致某个子系统缺少独立完整契约与状态机的，Auditor 必须判定为 `Crammed / Under-specified` 并强制拆出独立新卷。
  4. **1 Blueprint = 1 Focused Worker Slice**：每份蓝图精确对应 Stage 3 中单个专项 Worker 一次施工的 1～4 个高内聚代码模块（每模块 **200–600 行**）。
- 每一份分卷蓝图必须严格包含 `README.md` 规定的 **7 大强制章节**（1. 源码精读与全量功能零遗漏矩阵；2. 现有割裂问题与根因；3. 统一架构与 Canonical Owner 位置；4. 极致 UI/UX 与交互设计；5. 精确 TS/Rust 接口契约与状态机；6. 子智能体实施切片与写入边界；7. 防作弊审计与验收标准）。

---

## 阶段三：按蓝图拆解派发给子智能体执行（Blueprint-Driven Subagent Execution）

只有当对应模块的蓝图文件编写完毕后，才允许调用 `invoke_subagent` 派发实施子智能体（Worker Subagents）：

1. **精准投喂蓝图**：每个 Worker Subagent 的任务提示词必须明确指定其负责阅读和执行的具体 `docs/specs/fusion-blueprints/0X-*.md` 蓝图路径、允许修改的文件白名单（遵守 `Same Owner != Same File`，新增内聚子模块控制在 **200–600 行**）、以及精确的 TypeScript/Rust 接口定义。
2. **串行收口根文件**：各子智能体在各自领域目录内独立开发，禁止多个子智能体同时覆写 `ProductionRenderer.tsx`。所有子模块就绪后，由总控或唯一整合智能体对 `ProductionRenderer.tsx` 进行干净的窄接线（严禁为凑行数删减既有业务功能）。
3. **红线铁律检查**：
   - 彻底消灭 `activeIsHuman` 导致的双顶栏分裂；
   - 彻底删除 `OpenClawAskCopilotPanel`，坚决消除硬编码在线和 `isGatewayOnline={false}` 假离线；
   - 7 个右侧竞争抽屉收敛为唯一 `ContextInspector`；
   - 解除左轨“任务（Tasks）”的门控限制，使其无条件进入全局任务与自动化中心；
   - 修复投票/待办弹窗用 `setTimeout(..., 50)` 覆盖草稿的 Bug；
   - 修复分享按钮回退复制 `window.location.href` 的 Bug。

---

## 阶段四：独立取证审计与闭环验收（Independent Audit & Verification）

每个模块落地后，必须派发**独立的审查/挑战/审计子智能体（Reviewer / Challenger / Auditor Subagent）**，对照 `docs/specs/fusion-blueprints/0X-*.md` 与 `docs/specs/unified-fabushi-experience-20261105.md` 进行逐行取证审计：

- **查功能完整性**：是否完整保留了 Telegram 与 OpenClaw 的全部对应能力？有无空壳压缩（`Anti-N-to-1 Stub Compression`）或偷偷删减入口？
- **查 UI/UX 统一性**：是否彻底消除了 `activeIsHuman` 导致的双顶栏分裂？右侧 7 个抽屉是否已真正合入唯一 `ContextInspector`？左轨“任务”在真人聊天时点击是否能正常打开全局任务中心？
- **查真实性与零作弊**：是否彻底清除了 `OpenClawAskCopilotPanel`、`setTimeout` 假回复、硬编码在线、以及 `isGatewayOnline={false}` 硬编码假离线？投票/待办提交是否已不再覆盖用户草稿？分享按钮是否已不再回退复制 `window.location.href`？
- **查语法与契约安全**：TypeScript 类型、Props 契约、JSX 标签闭合、跨账号 Fencing 是否 100% 严丝合缝？

凡有任一审计项未通过，立即打回对应 Worker 修复，直至全部模块均获独立审计 `CLEAN / PASS`。最后同步更新 `docs/specs/unified-fabushi-execution-status-20261105.md`，如实报告所有代码改动、调用链证据与远端 Actions 验证状态。

现在，请立即从【阶段一：全量三方源码精读】与【阶段二：编写全景能力索引与分模块蓝图】开始执行！
