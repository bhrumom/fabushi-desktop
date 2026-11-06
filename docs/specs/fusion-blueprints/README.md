# Fabushi 全模块深度融合蓝图目录（`docs/specs/fusion-blueprints/`）

本目录用于存放将 **Telegram Desktop（TDRP，52+ 能力域）** 与 **OpenClaw（OCRM，32 责任域）** 全量产品源码能力深度融入 **Fabushi（FBCP / UFX-001）** 的全景能力总索引与分模块详细设计蓝图。

> **核心铁律（`UFX-FLOW-01`）**：
> 1. **先读透源码再设计**：任何主控智能体（Orchestrator）在派发子智能体修改生产代码之前，**必须先完成对三方源码（Fabushi 生产源码 + Telegram Desktop 锁定源码 + OpenClaw 锁定源码）的全量精读**。
> 2. **绝不限制蓝图数量，严格按高内聚与深度标准拆分（Open-Ended Blueprint Decomposition）**：严禁预设固定的“8 个模块”上限或将十几个独立子系统硬塞进同一份文档充数！必须根据客观的**【分模块蓝图拆分与颗粒度标准】**，该拆多少份就拆多少份（`01..NN-*.md`），直到每一个业务域、每一个状态机、每一条前后端接口都被独立、完整、无省略地写透为止。

---

## 一、 源码精读方法论：四步精读法与源码五问（Mandatory Methodology）

源码精读绝不是粗略扫一眼目录树或只看顶层账本，必须采用**“正向查全景账本 + 反向扫真实源码目录树”**的双向交叉核验，并按以下四步执行：

1. **第一步：宏观账本与真实目录树双向核验（Macro Ledger & Full Directory Tree Cross-Audit）**
   - **Telegram 侧**：对照 [`projects/telegram-desktop-rust/module-map.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/telegram-desktop-rust/module-map.md)、[`telegram-capability-graph.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/fabushi-communication-platform/telegram-capability-graph.md)（`TCG-N01..N52`）及 `/Users/bhrum/.gemini/antigravity/scratch/tdesktop/Telegram/SourceFiles/` 下的全部 40 个一级目录与 130 个二级子目录；
   - **OpenClaw 侧**：对照 [`projects/openclaw-rust/module-map.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/openclaw-rust/module-map.md)（`OCRM-M01..M32`）、`parity-ledger.json`（含 141 项 `implemented` + `not-wired` 与 9 项 `not-started`）及 `/Users/bhrum/.gemini/antigravity/scratch/openclaw/` 下的 77 个 `src/` 子目录、176 个 `extensions/`、50 个 `skills/`；
   - **Fabushi 侧**：对照 [`current-owner-inventory.md`](file:///Users/bhrum/Documents/ChatGPT/fabushi-unified-design-20261105/projects/fabushi-communication-platform/current-owner-inventory.md)、`architecture-map.md`、`canonical-screen-patterns.md`、`ui-information-architecture.md` 以及 `frontend/src/`、`source/host/`、`source/node-agent-coordinator/`、`native/mahayana-messaging/` 全部现有生产文件。
2. **第二步：三方源码并排精读（Tri-Source Side-by-Side Reading）**
   - 针对每个具体子系统，同时打开并排阅读 Telegram C++/Qt 源码、OpenClaw TS/Lit/Node 源码与 Fabushi TSX/Rust 源码，提取全部细分用户功能、边界条件与数据结构。
3. **第三步：端到端全链路调用追踪（End-to-End Call-Chain Tracing）**
   - 顺着真实调用链逐行追踪：
     `UI Entrypoint (Click/Key/Drag) -> Producer -> Controller/Store -> IPC/RPC Bridge -> Host/Coordinator Rust Service -> Storage/Network Provider -> Event/Subscription Loop back to UI Consumer`
   - 必须记录真实存在的类名、函数名、文件路径与行号，严禁凭空编造 API。
4. **第四步：强制回答“源码五问”（The Mandatory Source Five Questions）**
   - 对每一项功能必须用确凿代码证据回答：
     - **Q1（上游真实现状）**：上游如何实现？核心状态机、数据模型与边界条件（超时、取消、重试、错误码）是什么？
     - **Q2（现有链路定位）**：Fabushi 现有 canonical owner 是谁？生产调用链走到哪一步？
     - **Q3（断点精确诊断）**：为什么当前不可用或割裂？是 `not-wired`、是硬编码假数据/假状态，还是缺少类型对接？（精确到文件和行号）
     - **Q4（状态机与安全防护）**：跨组件信任边界如何？断网重连、切账号（`accountScopeGeneration`）、切会话（`session_key`）时如何隔离防串扰？
     - **Q5（UI/UX 极致融合落点）**：在统一界面中落在哪个标准位置？高频操作如何直达？低频配置如何渐进披露？如何绑定 `--fabushi-*` 语义 Token 与无障碍标准？

---

## 二、 分模块蓝图拆分标准（不限数量，按内聚度与深度标准动态生成 `01..NN`）

本目录采用 **“1 份全景总控索引（`00`） + N 份高内聚专项模块蓝图（`01..NN-*.md`，数量不设上限）”** 的架构。

### 1. 全景总控能力对齐索引（`00-master-capability-and-module-index.md`）
- 必须首先创建并持续同步维护该总索引，以二维矩阵 **100% 覆盖 OpenClaw 全部 32 个责任域（`OCRM-M01..M32`）与 Telegram Desktop 全部 52+ 个能力域（`TCG-N01..N52+`）**。
- 每一项能力域及细分功能必须指向其专属的分卷蓝图文件（`01..NN-*.md`），确保 **0 遗漏、0 孤岛、0 大杂烩挤压**。

### 2. 决定拆出多少份分卷蓝图的 4 条硬性标准（Splitting & Granularity Standards）
在将 84+ 个能力域拆解为分卷蓝图（`01..NN-*.md`）时，**严禁固定数量**，必须严格按以下 4 条客观工程标准决定拆分粒度（符合标准能拆出多少份就是多少份）：

1. **标准一：单卷领域密度上限（Anti-Cramming Cap：每卷最多承载 1～4 个强内聚能力域）**
   - 每一份分卷蓝图必须只聚焦于**单一高内聚的产品表面或单一独立的后端/业务子系统**。
   - **严禁大杂烩合并**：绝不允许将超过 4 个松散相关的能力域塞进同一份蓝图（例如：绝不允许把远程桌面 Computer、端到端加密、SQLite 数据库恢复、CLI 终端、自动升级等 10 几个不同领域的子系统塞进同一个“后端杂项”分卷；也绝不允许把 WebRTC 通话、Stories 动态、图片涂鸦编辑器、AI 生图生视频、TTS/STT 实时语音和会议 Bot 塞在同一个分卷里一笔带过）。
2. **标准二：独立状态机与独立数据模型必拆准则（Independent State Machine & Schema Isolation）**
   - 凡是拥有**独立持久层表结构（SQLite Schema / 账本）**、**独立网络/信令/加密协议（如 WebRTC、双棘轮加密、MCP、Webhook、SRP 2FA）**、**独立后台执行引擎（如 Cron 调度器、QMD 向量检索、音视频转写、大文件分片传输、支付结算流水线）** 或 **独立复杂交互表面（如应用壳、会话主舞台、输入台、右侧检查器、全局任务中心、媒体涂鸦编辑器、MiniApp 容器、社群统计图表）** 的子系统，**必须独立成卷**！
3. **标准三：单能力契约深度下限（Minimum Contract Depth per Subsystem）**
   - 在每一份蓝图中，其所覆盖的每一个能力域都必须写出：**完整的状态转移方程、完整的 TypeScript `interface`/Props 定义、完整的 Rust `struct`/`enum`/RPC Payload 定义、精确到行号的三方源码映射与 UI/UX 交互态矩阵**。
   - 如果某个子系统在某份蓝图中因为篇幅拥挤，只分到了泛泛而谈的十几行文字而没有完整的独立状态机与 TS/Rust 接口契约，**审计官（Auditor）必须判定其“颗粒度不达标（Crammed/Under-specified）”，强制将其拆分为独立的专项蓝图文件**！
4. **标准四：子智能体单次施工切片对齐（1 Blueprint = 1 Focused Worker Slice）**
   - 每份蓝图应当恰好对应下一阶段（Stage 3）1 个专项编码子智能体（或 1 个紧密配合的小组）能够一次性读透并写完的 **1～4 个高内聚代码文件（每个代码文件 200–600 行）**。

---

## 三、 每份模块融合蓝图（`01..NN-*.md`）必须包含的 7 个强制章节

无论拆出多少份分卷蓝图，每一份 `.md` 蓝图都必须完整包含以下 7 个章节，缺一不可：

1. **源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）**
   - 列出本模块对应的 Telegram Desktop 锁定源码文件/类/函数、OpenClaw 锁定源码文件/状态机、以及 Fabushi 现有代码文件与精确行号。
   - 包含完整的**“源码五问（Q1..Q5）”**逐项源码取证回答。
2. **现有割裂问题与根因分析（Current Fragmentation & Root Cause）**
   - 精确指出当前 Fabushi 代码中存在的割裂点、硬编码、伪逻辑、断头调用链（`implemented` 但 `not-wired`）或状态竞态（精确到文件路径与行号）。
3. **统一架构与职责归属（Architecture & Canonical Owner Location）**
   - 明确唯一的状态 Owner（Store / Controller / Rust Service）、组件挂载路径、新增/修改文件的精确路径与行数预算（单内聚领域代码模块 **200–600 行**，`Same Owner != Same File`）。
4. **极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）**
   - 详细描述界面布局结构、渐进披露（Progressive Disclosure）层级、所有交互状态（Loading / Empty / Ready / Offline / Waiting Auth / Error / Retry）、动画与过渡、键盘快捷键、中文 IME 行为、焦点恢复（Focus Return）、响应式断点（≥1200 / 900–1199 / <900）以及 `--fabushi-*` 语义 Token 绑定。
5. **精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）**
   - 写出具体的 TypeScript `interface` / Props 定义、Rust `struct` / `enum` / RPC 消息格式、数学状态转移方程、以及跨账号/跨会话的 `accountScopeGeneration` / `session_key` Fencing 规则（严禁 `any` 或 `TODO` 占位）。
6. **子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）**
   - 将该模块的任务定义为无冲突的子智能体执行指令：明确该子智能体**允许修改的文件白名单**、**只读参考文件**、以及与 `ProductionRenderer.tsx` 窄接线的接口约定。
7. **防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）**
   - 列出独立审计子智能体（Auditor/Reviewer）用来逐项核验的正反例检查表（禁止 `setTimeout` 假逻辑、禁止硬编码在线/假离线、禁止覆盖草稿、禁止回退复制无效 URL、禁止多合一空壳压缩）。
