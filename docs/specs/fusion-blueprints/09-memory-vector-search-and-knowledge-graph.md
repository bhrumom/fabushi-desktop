# Fabushi 统一融合蓝图 09：长期记忆、QMD 向量检索与 Wiki 知识图谱
# Volume 09: Long-Term Memory, QMD Vector Search, and Knowledge Graph Engine

> **规范编号**: `FBCP-BLUEPRINT-09` / `UFX-BLUEPRINT-09`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 终结检索孤岛与跨会话记忆割裂：深度融合 Telegram Desktop 高性能全文检索（SQLite FTS5）与 OpenClaw 长期记忆银行、QMD 语义向量检索与 Wiki 实体知识图谱。
> 2. 构建混合检索流水线（Hybrid Retrieval Pipeline）：结合 BM25 稀疏倒排索引与 Dense Vector 向量近似近邻（ANN），应用倒数排名融合算法（RRF, $k=60$）输出统一排名结果。
> 3. 闭环记忆全生命周期管理：提供显式事实捕获（`- (YYYY-MM-DD) <fact>`）、无损记忆梦境合成（Dreaming Consolidation）、CAS 乐观锁校验、物理级原子遗忘（Atomic Forget & Durable Tombstone）以及 30 天半衰期时间衰减。
> 4. 落地精准溯源引用（Citation Provenance）：实现 `#L<start>-L<end>` 真实代码/文本行号格式化与群组/频道的自动模式抑制（Auto-suppression），杜绝虚拟推断行号。
> 5. 统一 UI 交互入口：联动全局指令面板（`CommandPalette`，支持消息/文件/链接/行动全量过滤）、会话内查找（`FindInChatBar`，高亮脉冲动画与匹配定位）以及右侧检查器记忆抽屉（`ContextInspectorMemoryTab`）。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **长期记忆存储与事实解析**<br>(Long-term Memory Store) | `src/memory/` (tree `04ccce62f1`)<br>`extensions/memory-core/src/memory-tool-contract.ts:1-181`<br>`extensions/memory-core/src/memory-forget.ts:1-508` | `storage/storage_account.cpp:120-250`<br>`data/data_user.cpp:310-380` | `source/host/src/extensions/memory/memory_service.rs:1-921`<br>`source/host/src/extensions/memory/memory_service_ocrm.rs:1-420`<br>`source/host/src/runner/sand_memory.rs:1-602` | `source/host/src/extensions/memory/memory_service.rs`<br>(`FileMemoryStore`) |
| **记忆合成与梦境固化**<br>(Memory Synthesis & Dreaming) | `extensions/memory-core/src/short-term-promotion.ts:1-229`<br>`src/memory/synthesis.ts:1-310` | N/A<br>(依赖服务端漫游状态 `data/data_drafts.cpp`) | `source/host/src/extensions/memory/memory_synthesis_service.rs:1-520`<br>`source/host/src/extensions/telemetry/memory_synthesis_telemetry.rs:1-180` | `source/host/src/extensions/memory/memory_synthesis_service.rs`<br>(`MemorySynthesisService`) |
| **QMD 语义向量索引与嵌入**<br>(QMD Vector Store & Embeddings) | `extensions/memory-lancedb/src/index.ts:1-350`<br>`extensions/memory-core/src/vector-store.ts:1-280`<br>`src/embeddings/cache.ts:1-190` | `storage/storage_search.cpp:1-450`<br>(仅支持分词倒排) | `source/host/src/storage/vector_store.rs`<br>`source/host/src/extensions/content_search/search_index_db.rs:15-28` | `source/host/src/storage/vector_store.rs`<br>(`QmdVectorStore`) |
| **全文消息与媒体索引**<br>(SQLite FTS5 Fulltext Index) | `extensions/memory-core/src/tools.ts:1-587`<br>`packages/memory-host-sdk/` | `storage/storage_search.cpp:1-450`<br>`data/data_search_messages.cpp:1-620`<br>`Telegram/SourceFiles/api/api_messages_search.cpp:1-380` | `source/host/src/extensions/content_search/search_index_db.rs:1-249`<br>`source/host/src/extensions/content_search/search_index_service.rs:1-310`<br>`native/mahayana-messaging/src/search.rs:253-350` | `source/host/src/extensions/content_search/search_index_db.rs`<br>(`SearchIndexDb`) |
| **精确引用行号与群聊抑制**<br>(Citation Provenance & Scoping) | `extensions/memory-core/src/tools.citations.ts:1-61`<br>`packages/memory-citations/` | `history/history_item_components.cpp:140-210`<br>(消息跳转与引用高亮) | `source/host/src/extensions/content_search/memory_citations.rs:1-115`<br>`projects/openclaw-rust/docs/specs/memory-citations-format.md:1-59` | `source/host/src/extensions/content_search/memory_citations.rs`<br>(`format_citation`) |
| **Wiki 知识图谱与双向链接**<br>(Wiki Graph & [[wikilinks]]) | `extensions/memory-wiki/src/vault.ts:1-340`<br>`extensions/memory-wiki/src/graph.ts:1-220` | `data/data_peer_values.cpp:80-160`<br>(Peer mention/hashtag 解析) | `source/host/src/extensions/memory/wiki_graph.rs`<br>`source/host/src/extensions/memory/root_memory_provenance.rs` | `source/host/src/extensions/memory/wiki_graph.rs`<br>(`WikiKnowledgeGraph`) |
| **全局与局部搜索交互表面**<br>(Global Palette & Find in Chat) | `ui/src/pages/search.ts`<br>`ui/src/components/search-bar.ts` | `history/history_search.cpp:1-380`<br>`dialogs/dialogs_search_from_controllers.cpp:1-210` | `frontend/src/production/CommandPalette.tsx:1-383`<br>`frontend/src/production/command-palette-message-provider.ts:1-195`<br>`frontend/src/production/command-palette-search-provider.ts:1-197`<br>`frontend/src/recovered/features/conversation/workspace/sidebar-search-trigger.ts:1-54` | `frontend/src/production/CommandPalette.tsx`<br>`frontend/src/production/conversation/FindInChatBar.tsx` |
| **记忆检查器与知识库可视化**<br>(Memory Inspector Tab) | `ui/src/components/memory/memory-inspector.ts:1-290`<br>`ui/src/components/vault/` | `profile/profile_media.cpp:1-420`<br>(按多媒体分类检索与过滤) | `frontend/src/production/inspector/tabs/ContextInspectorMemoryTab.tsx`<br>`frontend/src/recovered/features/conversation/workspace/conversation-outline-view.tsx` | `ContextInspectorMemoryTab.tsx`<br>(唯一记忆抽屉标签页) |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **OpenClaw 侧**：
  - 核心记忆库在 `src/memory/` 与 `extensions/memory-core/` 中维护。事实分为持久概况（`profile.md`）与时间日志（`log/YYYY-MM-DD.md`），单行格式严格约束为 `- (YYYY-MM-DD) <fact>`。
  - 检索采用混合架构：`extensions/memory-lancedb/` 基于 LanceDB/SQLite 维护文本与 QMD 语义分块嵌入向量，`short-term-promotion.ts` 结合相关性、时间戳与强化权重在梦境阶段（Dreaming）提升候选事实。
  - 物理遗忘由 `memory-forget.ts` 驱动，生成不可逆墓碑标记（Tombstone），防止已被抹除的记忆在后续向量合成或会话回放中复活。
  - 引用格式化由 `tools.citations.ts` 控制：显式输出 `#L<start>` 或 `#L<start>-L<end>`，并在会话键包含 `channel` 或 `group` 时强制静默。
- **Telegram Desktop 侧**：
  - 搜索分为会话级与全局级，底层由 `storage/storage_search.cpp` 与 `data/data_search_messages.cpp` 统管。
  - 会话内搜索（`history/history_search.cpp`）支持正向/反向滚动、搜索词高亮（Highlight Pulse）、匹配计数器（`search_results_count`）以及跳转定位（`jumpToItem`）。
  - 全局搜索支持按发起者（`from_user`）、日期跨度（`min_date`/`max_date`）以及 8 类媒体类型（照片、视频、文件、音频、链接、语音、圆视频、GIF）进行多条件交集查询。

#### Q2（现有链路定位）
- **全文检索生产链路**：
  - 前端 `CommandPalette.tsx` 唤起 -> 驱动 `CommandPaletteMessageProvider`（`command-palette-message-provider.ts:26-34`）与 `CommandPaletteFileProvider`（`command-palette-search-provider.ts:42-50`）-> IPC 桥接 `searchAgents` / `searchMedia` -> Rust 服务 `source/host/src/extensions/transcript/roster_search.rs:search_agents` -> 最终由 `source/host/src/extensions/content_search/search_index_db.rs:search_messages` 执行 SQLite FTS5 查询。
- **长期记忆注入与提取链路**：
  - 模型执行推理时，`source/host/src/runner/sand_memory.rs:resolve_frozen_memory_prompt` 读取 `source/host/src/extensions/memory/memory_service.rs`（`FileMemoryStore`），将事实注入 System Prompt（遵循 `MEMORY_USER_PROFILE_CHAR_BUDGET = 4,000` 与 `MEMORY_RECENT_PROMPT_LIMIT = 30` 字符预算）。
  - 回合结束时触发 `sand_memory.rs:MemoryExtraction` 提取新增事实，经由 `memory_synthesis_service.rs` 提交 CAS 乐观锁写入磁盘。

#### Q3（断点精确诊断）
1. **语义向量与文本检索孤岛断点**（`search_index_db.rs:15-28` vs `vector_store.rs`）：
   - `search_index_db.rs` 仅建立了 `messages_fts` 与 `media_fts` 两张 FTS5 虚拟表，只支持精确分词匹配，对近义词、语义概念检索与自然语言知识问答完全无能为力；QMD 向量嵌入索引处于分离状态，缺乏统一的 RRF 倒数排名融合聚合层。
2. **记忆引用真实行号未贯通断点**（`memory_citations.rs:1-115` 与 `roster_search.rs:search_agents`）：
   - `source/host/src/extensions/content_search/memory_citations.rs` 虽已实现严格的引用规则，但生产端 `roster_search.rs` 投影的 `MessageSearchResult` 仅包含 `agent_id`、`entry_id`、`role`、`timestamp_ms` 与扁平 `snippet`，未附带物理文件的真实起始行号，导致引用格式化器始终处于 `not-wired` 挂起状态。
3. **会话内查找缺失上下文脉冲高亮断点**（`sidebar-search-trigger.ts:27-53`）：
   - 现有的会话内查找仅有侧边栏触发接口，未实现 Telegram Desktop 式的顶栏悬浮查找条（`FindInChatBar`），缺少匹配项总数显示、上一条/下一条切换以及消息定位后的黄色脉冲动画（Yellow Pulse Highlight）。
4. **群聊与多账号沙箱串扰断点**（`sand_memory.rs:12-25` 与 `memory_service.rs:42`）：
   - `FileMemoryStore` 中的记忆路径未绑定显式 `accountScopeGeneration`，在多账号切换时可能加载旧账号的个人记忆；群组会话在未设置显式模式时可能误暴露私有 User Profile 记忆。

#### Q4（状态机与安全防护）
- **混合检索倒数排名融合状态机**：
  $$RRF(d) = \sum_{m \in \{FTS, Vector\}} \frac{1}{k + r_m(d)}, \quad k = 60$$
  - 当检索请求到达时，并行派发 FTS5 文本查询与 Vector 余弦近邻查询，限制各分支最大召回 50 项，经 RRF 公式归一化后截断输出 Top 20。
- **跨会话与账号隔离 Fencing 规则**：
  - 记忆命名空间严格由 `account_id`、`agent_id` 与 `session_key` 联合签名：
    $$\text{MemoryScope} = \text{HMAC-SHA256}(\text{account\_id}, \text{agent\_id} \parallel \text{session\_key})$$
  - 当用户在群聊或公开频道（`session_key` 包含 `group` 或 `channel`）执行记忆检索时，`MemoryCitationsMode::Auto` 强制触发自动抑制（Auto-suppression），隐藏具体文件物理路径，防止敏感路径外泄。
- **CAS 乐观锁防冲突状态转移**：
  - 记忆合成提交必须携带 `fingerprint`：
    $$\text{Fingerprint}_{new} = \text{SHA256}(\text{MemoryFacts} \parallel \text{Version})$$
    若磁盘指纹不匹配则返回 `SynthesisApplyResult::Stale`，彻底杜绝多并发 Agent 写入时的覆写丢失。

#### Q5（UI/UX 极致融合落点）
- **全局搜索（Command Palette）**：通过快捷键 `Cmd/Ctrl + K` 居中唤起，预置全部（All）、消息（Messages）、文件（Files）、链接（Links）、动作（Actions）等 8 大过滤器，支持拼音首字母与模糊高亮。
- **会话内搜索（Find in Chat）**：通过快捷键 `Cmd/Ctrl + F` 在当前会话舞台顶栏下方平滑滑出，右侧附带“第 3 / 共 42 条”、上箭头/下箭头、媒体类型筛选胶囊与关闭按钮；选中结果时光标平滑滚动至对应气泡，并播放 1.5 秒淡出高亮外框。
- **记忆检查器（Memory Inspector）**：集成于右侧上下文抽屉（`ContextInspector`）的“记忆与知识”Tab，采用卡片流形式展示“持久概况”（Profile）与“日志事实”（Logs），支持关键词检索、一键复制引用、物理删除（Forget）及 Wiki 实体网状拓扑预览。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-09-01** | 全文与向量检索两极孤岛，语义知识问答无法召回 | `source/host/src/extensions/content_search/search_index_db.rs:15-28`<br>`projects/openclaw-rust/SOURCE_OF_TRUTH.md:83-84` | FTS5 虚拟表仅索引扁平分词，未集成 QMD 语义向量索引与嵌入模型，无 RRF 倒数排名融合算法，导致同义词、意图理解类查询完全失效。 | **P0 架构孤岛** |
| **BUG-09-02** | 记忆引用格式化策略已实现但生产未连通，行号虚设 | `source/host/src/extensions/content_search/memory_citations.rs:21-37`<br>`projects/openclaw-rust/docs/specs/memory-citations-format.md:8-13` | `roster_search.rs` 投影的 `MessageSearchResult` 缺少文件行号元数据，`memory_citations.rs` 处于 pure policy not-wired 状态，UI 无法渲染真实 `#L<start>-L<end>`。 | **P1 链路悬空** |
| **BUG-09-03** | 会话内查找缺乏沉浸式高亮与匹配计数交互 | `frontend/src/recovered/features/conversation/workspace/sidebar-search-trigger.ts:27-53`<br>`frontend/src/production/CommandPalette.tsx:31-33` | 仅有侧边栏与全局命令面板触发器，未实现 Telegram 级的会话级悬浮查找栏（`FindInChatBar`），缺乏命中总数反馈与目标消息平滑滚动锚定。 | **P1 交互缺失** |
| **BUG-09-04** | 记忆合成提交缺乏跨账号作用域隔离，可能导致串扰 | `source/host/src/runner/sand_memory.rs:16-25,127-145`<br>`source/host/src/extensions/memory/memory_service.rs:42` | `FileMemoryStore` 仅以固定目录 `memory/` 与 `profile.md` 存储，未显式注入 `accountScopeGeneration`，在同一客户端切换不同登录身份时存在数据泄漏隐患。 | **P0 安全隔离** |
| **BUG-09-05** | 遗忘指令（Forget）缺乏跨存储引擎的原子物理清除 | `source/host/src/extensions/memory/memory_service.rs:86-99,108-112`<br>`extensions/memory-core/src/memory-forget.ts:1-508` | 当前只对 Markdown 文本行追加注释或删除，未能同步在 SQLite 搜索索引与向量库中标记持久墓碑（Tombstone），导致后续会话中记忆事实容易被错误召回。 | **P1 数据不一致** |
| **BUG-09-06** | 媒体与消息搜索缺少流式加载取消（AbortController） | `frontend/src/production/command-palette-search-provider.ts:42-50`<br>`frontend/src/production/command-palette-message-provider.ts:26-34` | 连续快速键入搜索词时，先前的异步查询请求未能被有效中止，容易产生乱序返回（Race Condition）并导致界面结果闪烁。 | **P2 稳定性隐患** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 统一检索与记忆系统架构全景

```
┌───────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                 Fabushi Search & Memory Architecture                              │
├─────────────────────────────────────────────────┬─────────────────────────────────────────────────┤
│ 1. UI 交互层 (Presentation Surfaces)           │ 2. 状态与适配层 (Controllers & Providers)       │
│ - CommandPalette.tsx (全局综合命令面板)          │ - CommandPaletteMessageProvider.ts (消息过滤)   │
│ - FindInChatBar.tsx (会话内悬浮查找栏)          │ - CommandPaletteFileProvider.ts (媒体文件过滤)  │
│ - ContextInspectorMemoryTab.tsx (记忆知识抽屉)  │ - MemoryInspectorController.ts (记忆编辑/遗忘)  │
├─────────────────────────────────────────────────┴─────────────────────────────────────────────────┤
│ 3. 核心协调与服务层 (Host Rust Core Services)                                                    │
│ ┌───────────────────────────────────────────────┬───────────────────────────────────────────────┐ │
│ │ HybridRetrievalEngine (混合检索中枢)          │ MemoryLifecycleCoordinator (记忆生命周期管理) │ │
│ │ - BM25 Sparse Search (FTS5)                   │ - Explicit Fact Parser (profile.md / log)     │ │
│ │ - Vector Dense Search (QMD ANN)               │ - Dreaming & Consolidation (CAS Optimistic)   │ │
│ │ - RRF Rank Fusion Normalizer (k=60)           │ - Atomic Forget & Durable Tombstone Store     │ │
│ │ - Citation Provenance Formatter (#L<start>)   │ - Memory Decay Calculator (30-day Half-life)  │ │
│ └───────────────────────────────────────────────┴───────────────────────────────────────────────┘ │
├───────────────────────────────────────────────────────────────────────────────────────────────────┤
│ 4. 底层持久层与安全沙箱 (Persistence & Storage Engines)                                           │
│ ┌───────────────────────────┬───────────────────────────┬───────────────────────────────────────┐ │
│ │ SQLite FTS5 (search-index)│ QmdVectorStore (LanceDB)  │ Markdown Vault (Wiki & Profile)       │ │
│ │ - messages_fts            │ - qmd_embeddings.lance    │ - memory/profile.md                   │ │
│ │ - media_fts               │ - semantic_chunk_cache    │ - wiki/[[concepts]].md                │ │
│ └───────────────────────────┴───────────────────────────┴───────────────────────────────────────┘ │
│ 安全沙箱保障: Account Scope Fencing | Session Type Suppression | Path Traversal Protection        │
└───────────────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 Canonical Owners 职责分布与代码行数预算（Same Owner != Same File）

遵循单一职责与 200–600 行紧凑内聚原则，严禁新增巨石文件：

| 模块名称与责任域 | 归属文件物理路径 | 职责范围与边界 | 行数预算 |
|---|---|---|---|
| **会话内悬浮查找条** | `frontend/src/production/conversation/FindInChatBar.tsx` | 会话内查找输入框、结果计数（"X of Y"）、上/下翻动、媒体类型筛选胶囊与高亮联动 | 260–340 行 |
| **记忆上下文抽屉面板** | `frontend/src/production/inspector/tabs/ContextInspectorMemoryTab.tsx` | 右侧检查器记忆标签页，展示持久概况卡片、日志流、Wiki 实体跳转与遗忘确认对话框 | 320–420 行 |
| **混合检索融合服务** | `source/host/src/extensions/content_search/hybrid_retrieval.rs` | 执行 FTS5 与 Vector ANN 并行查询，应用 RRF 倒数排名融合算法并截断 Top-K | 240–320 行 |
| **QMD 语义向量索引引擎** | `source/host/src/storage/vector_store.rs` | 语义分块（Chunking）、向量嵌入持久化、余弦相似度近邻计算与缓存更新 | 350–480 行 |
| **Wiki 知识图谱解析器** | `source/host/src/extensions/memory/wiki_graph.rs` | 解析 Markdown 双向链接 `[[wikilinks]]`、提取实体关系边拓扑、输出知识节点子图 | 280–380 行 |
| **记忆事实与墓碑存储** | `source/host/src/extensions/memory/memory_service.rs` | 维系 `FileMemoryStore`，支持 CAS 乐观锁事实更新、物理遗忘与持久墓碑防复活 | 380–520 行 |
| **真实引用格式化器** | `source/host/src/extensions/content_search/memory_citations.rs` | 纯策略行号格式化器，实现 `#L<start>-L<end>` 组装与群组/频道会话自动屏蔽 | 160–220 行 |

---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 核心交互组件与界面布局结构

本模块在统一前端体系中由三大交互表面构成：**全局综合命令面板（CommandPalette）**、**会话内悬浮查找栏（FindInChatBar）** 以及 **右侧上下文记忆检查器（ContextInspectorMemoryTab）**。

#### 4.1.1 全局综合命令面板（CommandPalette）
- **唤起与层级**：全局快捷键 `Cmd/Ctrl + K` 居中弹出，外层配备高斯模糊遮罩（`backdrop-filter: blur(8px)`，层级 `z-index: 3000`）。
- **渐进披露（Progressive Disclosure）**：
  - *未键入状态*：展示“最近检索（Recent Searches）”以及“常用快捷动作（Actions）”，避免空白尴尬；
  - *输入状态*：头部提供横向 Pill 胶囊过滤器，支持 8 域切换：全部（`all`）、消息（`messages`）、智能体（`agents`）、群组（`groups`）、文件（`files`）、链接（`links`）、自动化例程（`routines`）、动作（`actions`）。支持快捷键 `Cmd/Ctrl + 1..8` 秒级直达；
  - *长列表渲染*：采用虚拟滚动窗口（`commandPaletteVirtualWindow`），行高 52px，无论 10 万条历史记录均保持 60 FPS 滚动流畅度。

#### 4.1.2 会话内悬浮查找栏（FindInChatBar）
- **唤起与布局**：在具体会话舞台中通过快捷键 `Cmd/Ctrl + F` 唤起。浮动置顶于会话顶部（高度 44px），使用绝对浮层（Floating Overlay）模式，不破坏消息视口的滚动位置几何结构。
- **匹配计数与导航**：
  - 右侧显示实时匹配位置：“`3 / 42`”；若无命中则显示柔和红字“`0 / 0`”；
  - 配备“上一条”（`Shift + Enter` 或 `Cmd + G` 后退）、“下一条”（`Enter` 或 `Cmd + G` 前进）按钮；
  - 提供媒体过滤器快速切片：全部文本、图片、文件、链接；
  - 点击“关闭”或按 `Esc` 键时平滑向上收起，并将输入焦点无缝归还至底部会话 Composer。
- **黄色脉冲定位动画（Highlight Pulse）**：
  - 选中目标消息时，消息视口自动调用 `scrollIntoView({ behavior: 'smooth', block: 'center' })`；
  - 目标消息气泡激活 CSS 动画类 `.fabushi-search-pulse`，产生 1.5 秒从黄色微光高亮至透明的平滑衰减：
    ```css
    @keyframes fabushi-search-pulse {
      0% { box-shadow: 0 0 0 4px var(--fabushi-search-highlight-border); background-color: var(--fabushi-search-highlight-bg); }
      100% { box-shadow: 0 0 0 0 transparent; background-color: transparent; }
    }
    ```

#### 4.1.3 记忆与知识图谱检查器（ContextInspectorMemoryTab）
- **装配位置**：位于右侧统一上下文抽屉（`ContextInspector`）的第 4 个标准标签页。
- **事实卡片流（Fact Card Stream）**：
  - 卡片顶部显示事实类型徽章：绿色 `Profile`（长期稳定事实）、蓝色 `Log`（时间日志记录）、紫色 `Synthesized`（梦境固化提炼）；
  - 卡片右上方提供“复制带行号引用”图标按钮（输出 `MEMORY.md#L42` 或 `log/2026-11-05.md#L12-L16`）；
  - 提供“遗忘（Forget）”物理垃圾桶图标，点击调起带有不可逆告警的二级确认弹窗；
- **Wiki 双向链接图谱卡片**：
  - 提取当前会话或智能体相关的 `[[WikiEntities]]`，以关系网状图标签呈现，悬停展示实体卡片摘要，点击可直接在右侧抽屉内查看实体百科全文。

### 4.2 响应式三阶布局断点（Responsive Breakpoints）

严格遵循系统统一的三阶断点策略：
1. **宽屏工作台（≥1200px）**：
   - 全局命令面板宽度固定为 640px，最大高度 560px；
   - 会话内查找条固定宽度 460px 居右上角吸顶；
   - 右侧记忆检查器与主舞台常驻并列展示（宽度 360px），支持拖拽边缘调整至 420px。
2. **中屏协作态（900–1199px）**：
   - 命令面板宽度缩窄为 540px；
   - 会话内查找条自适应撑满会话顶栏；
   - 右侧记忆检查器自动折叠为滑出抽屉浮层（Drawer Overlay），背景增加 `rgba(0,0,0,0.3)` 半透明遮罩层。
3. **窄屏移动/单栏态（<900px）**：
   - 命令面板占满屏幕宽度（留白 16px）；
   - 会话查找条折叠辅助开关，仅保留文本输入框、结果指示器与前进/后退/关闭按钮；
   - 记忆检查器作为独立全屏视图推进（Push Transition），并提供返回导航按钮。

### 4.3 语义设计系统 Token 绑定与无障碍、IME 防护

- **设计变量（CSS Tokens）严格收口**：
  - 背景与边框：`--fabushi-bg-base`, `--fabushi-bg-surface`, `--fabushi-border-subtle`；
  - 文本对比度：`--fabushi-text-primary`（对比度 ≥ 7:1），`--fabushi-text-secondary`（对比度 ≥ 4.5:1，达成 WCAG AA）；
  - 检索与脉冲色：`--fabushi-search-highlight-bg: rgba(255, 214, 10, 0.28)`, `--fabushi-search-highlight-border: #ffd60a`；
  - 主题强调色：`--fabushi-accent-primary: #1084fe`。
- **无障碍（A11y）标准**：
  - 所有输入框显式标注 `aria-label="搜索消息或记忆"`、`role="search"`；
  - 查找结果切换时，通过隐藏的 `aria-live="polite"` 区域播报“已定位至第 3 条，共 42 条结果”；
  - 焦点恢复（Focus Return）：按 `Esc` 退出查找或命令面板后，焦点 100% 可预测地返还给触发前的控件。
- **中文输入法（IME）保护**：
  - 在 `compositionstart` 与 `compositionend` 事件周期内，输入框绝对屏蔽 `Enter` 按键的立即执行行为，防止中文拼音确认上屏时误触发“跳转至下一条”或“执行命令”，保障东亚多语言输入的极致流畅。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 TypeScript 接口契约定义

```typescript
// 1. 会话内悬浮查找栏输入与状态契约
export interface FindInChatBarProps {
  readonly isOpen: boolean;
  readonly activeConversationId: string;
  readonly totalMatches: number;
  readonly currentMatchIndex: number;
  readonly isSearching: boolean;
  onClose(): void;
  onSearchQueryChange(query: string): void;
  onNextMatch(): void;
  onPreviousMatch(): void;
  onFilterMediaTypeChange?(mediaType: "all" | "media" | "files" | "links"): void;
}

// 2. 长期记忆检查器快照与条目契约
export type MemoryKindTag = "profile" | "log" | "synthesized";

export interface MemoryFactItem {
  readonly id: string;
  readonly factDate: string; // YYYY-MM-DD
  readonly content: string;
  readonly kind: MemoryKindTag;
  readonly provenancePath: string; // e.g. "memory/profile.md"
  readonly startLine: number;
  readonly endLine: number;
  readonly citationText: string;   // e.g. "memory/profile.md#L42"
  readonly isTombstoned: boolean;
  readonly lastModifiedMs: number;
}

export interface MemoryInspectorSnapshot {
  readonly agentId: string;
  readonly accountScope: string;
  readonly facts: readonly MemoryFactItem[];
  readonly relatedWikiConcepts: readonly {
    readonly term: string;
    readonly snippet: string;
    readonly linkUrl: string;
  }[];
  readonly status: "idle" | "loading" | "ready" | "error";
  readonly errorMessage?: string;
}

// 3. 混合检索结果契约
export interface HybridSearchMatch {
  readonly entryId: string;
  readonly conversationId: string;
  readonly senderId: string;
  readonly timestampMs: number;
  readonly role: "user" | "assistant" | "system";
  readonly snippet: string;
  readonly citationProvenance?: {
    readonly path: string;
    readonly startLine: number;
    readonly endLine: number;
    readonly formatted: string;
  };
  readonly ftsRankScore: number;
  readonly vectorSimilarityScore: number;
  readonly fusedRrfScore: number;
}
```

### 5.2 Rust RPC 消息定义与存储引擎 Schema

```rust
// 1. 混合检索请求与响应 RPC Payload
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HybridRetrievalQuery {
    pub account_id: String,
    pub session_key: String,
    pub text: String,
    pub limit: usize,
    pub vector_weight: f32, // 默认 0.5
    pub fts_weight: f32,    // 默认 0.5
    pub citations_mode: MemoryCitationsMode,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HybridRetrievalResult {
    pub entry_id: String,
    pub session_key: String,
    pub body_snippet: String,
    pub citation: Option<String>,
    pub score: f32,
    pub source_kind: RetrievalSourceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RetrievalSourceKind {
    FtsOnly,
    VectorOnly,
    HybridFused,
}

// 2. 物理遗忘命令负载
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryForgetPayload {
    pub account_id: String,
    pub agent_id: String,
    pub fact_id: String,
    pub reason: String,
    pub expected_fingerprint: String,
}

// 3. QMD 语义向量索引表与 SQLite 扩展 Schema
pub const HYBRID_SEARCH_MIGRATION_SQL: &str = r#"
-- 语义分块元数据表
CREATE TABLE IF NOT EXISTS semantic_chunks (
    chunk_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    session_key TEXT NOT NULL,
    provenance_path TEXT NOT NULL,
    start_line INTEGER NOT NULL,
    end_line INTEGER NOT NULL,
    content_hash TEXT NOT NULL,
    body TEXT NOT NULL,
    created_at INTEGER NOT NULL
) STRICT;

CREATE INDEX IF NOT EXISTS idx_chunks_agent_session 
ON semantic_chunks(agent_id, session_key);

-- 记忆物理擦除持久墓碑表
CREATE TABLE IF NOT EXISTS memory_tombstones (
    fact_id TEXT PRIMARY KEY,
    agent_id TEXT NOT NULL,
    fact_hash TEXT NOT NULL,
    erased_at INTEGER NOT NULL,
    erased_by TEXT NOT NULL
) STRICT;
"#;
```

### 5.3 数学状态转移方程与 Fencing 隔离规则

#### 1. 混合检索倒数排名融合（RRF）状态转移
对于文档集合 $\mathcal{D} = \mathcal{D}_{\text{FTS}} \cup \mathcal{D}_{\text{Vec}}$，每个候选文档 $d \in \mathcal{D}$ 的最终得分由两路召回结果的排名倒数加权求和决定：
$$S_{\text{RRF}}(d) = \frac{w_{\text{fts}}}{k + \text{rank}_{\text{fts}}(d)} + \frac{w_{\text{vec}}}{k + \text{rank}_{\text{vec}}(d)}$$
其中常数平滑项 $k = 60$，$w_{\text{fts}} = 0.5$，$w_{\text{vec}} = 0.5$。若 $d \notin \mathcal{D}_{m}$，则令 $\frac{1}{k + \text{rank}_m(d)} = 0$。排序后截取前 $N$（$N \le 20$）个最高分候选者。

#### 2. 记忆合成 CAS 乐观锁状态转移方程
设当前存储指纹为 $\mathcal{F}_{\text{disk}}$，内存快照指纹为 $\mathcal{F}_{\text{snap}}$，操作集合为 $\Delta = \{\text{Create}, \text{Update}, \text{Remove}\}$：
$$\text{Apply}(\mathcal{F}_{\text{snap}}, \Delta) = \begin{cases}
(\mathcal{F}_{\text{new}}, \text{Committed}) & \text{if } \mathcal{F}_{\text{snap}} == \mathcal{F}_{\text{disk}} \land \text{TombstoneCheck}(\Delta) == \text{Pass} \\
(\mathcal{F}_{\text{disk}}, \text{Stale}) & \text{if } \mathcal{F}_{\text{snap}} \ne \mathcal{F}_{\text{disk}} \\
(\mathcal{F}_{\text{disk}}, \text{Invalid}) & \text{if } \text{PathTraversal}(\Delta) \lor \text{EmptyContent}(\Delta)
\end{cases}$$
只有当指纹完全一致且无墓碑冲突时，才允许写入新指纹并持久化到磁盘。

#### 3. 跨账号与跨会话自动抑制 Fencing 规则
令 $\mathcal{S}$ 为当前执行会话标识，$\mathcal{M}_{\text{mode}} \in \{\text{On}, \text{Off}, \text{Auto}\}$：
$$\text{EmitCitation}(\mathcal{S}, \mathcal{P}) = \begin{cases}
\text{FormatCitation}(\mathcal{P}) & \text{if } \mathcal{M}_{\text{mode}} == \text{On} \\
\text{None} & \text{if } \mathcal{M}_{\text{mode}} == \text{Off} \\
\text{None} & \text{if } \mathcal{M}_{\text{mode}} == \text{Auto} \land (\text{"group"} \in \mathcal{S} \lor \text{"channel"} \in \mathcal{S}) \\
\text{FormatCitation}(\mathcal{P}) & \text{if } \mathcal{M}_{\text{mode}} == \text{Auto} \land \text{"group"} \notin \mathcal{S} \land \text{"channel"} \notin \mathcal{S}
\end{cases}$$
任何群组、公共频道或多人会话在默认 `Auto` 模式下，一律拦截物理路径输出，杜绝敏感文件路径在公开讨论中泄漏。

---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 子智能体修改白名单（Modifiable File Whitelist）

下一阶段专项实施子智能体（Worker Subagent）在进行代码落地时，严格受限于以下文件白名单，禁止越界修改任何未授权文件：

1. **会话内查找组件**：
   - `frontend/src/production/conversation/FindInChatBar.tsx`（新建文件，内聚预算 260–340 行）
2. **记忆检查器标签页**：
   - `frontend/src/production/inspector/tabs/ContextInspectorMemoryTab.tsx`（新建文件，内聚预算 320–420 行）
3. **混合检索融合服务**：
   - `source/host/src/extensions/content_search/hybrid_retrieval.rs`（新建文件，内聚预算 240–320 行）
4. **QMD 语义向量索引引擎**：
   - `source/host/src/storage/vector_store.rs`（新建文件，内聚预算 350–480 行）
5. **Wiki 实体图谱拓扑服务**：
   - `source/host/src/extensions/memory/wiki_graph.rs`（新建文件，内聚预算 280–380 行）
6. **记忆引用格式化与生产接线**：
   - `source/host/src/extensions/content_search/memory_citations.rs`（微调现有策略模块，将真实行号元数据透传至输出结果，约 30–50 行微调）
7. **记忆事实持久层与墓碑表管理**：
   - `source/host/src/extensions/memory/memory_service.rs`（微调 `FileMemoryStore` 擦除逻辑，接入 `memory_tombstones` 持久化，约 40–60 行微调）
8. **全文索引数据库 Schema 演进**：
   - `source/host/src/extensions/content_search/search_index_db.rs`（追加 `semantic_chunks` 与 `memory_tombstones` 建表 DDL，约 30–40 行微调）

### 6.2 只读参考与禁止修改边界（Read-Only References & Prohibitions）

- **只读参考文件（Read-Only）**：
  - `native/mahayana-messaging/src/search.rs`：仅供查阅消息授权判定矩阵（`authorize_destination`），严禁修改其枚举定义；
  - `source/host/src/runner/sand_memory.rs`：仅供读取记忆 System Prompt 字符预算上限常量（`MEMORY_RECENT_PROMPT_LIMIT` 等）；
  - `frontend/src/production/CommandPalette.tsx`：仅供对接外部数据 Provider，不得重构其内部渲染模型；
  - `projects/openclaw-rust/docs/specs/memory-citations-format.md`：权威引用格式设计依据，只读核对。
- **严苛禁止项（Prohibitions）**：
  - ❌ **严禁在根渲染壳体进行大面积侵入**：禁止直接重构 `ProductionRenderer.tsx` 的主体结构，仅允许整合者通过窄属性（`isFindInChatOpen`）将 `FindInChatBar` 挂载在会话舞台顶栏下方；
  - ❌ **严禁创建平行设计系统样式**：严禁新建任何 `.oc-*` 或 `.tdrp-*` 样式类名，所有新组件必须 100% 消费 `--fabushi-*` 语义 CSS 变量；
  - ❌ **严禁伪造或虚构文件行号**：绝对禁止从文本片段长度、时间戳或数组索引反向推导假行号；无真实行号记录时必须返回 `None`；
  - ❌ **严禁本地执行构建或测试命令**：严格遵循 `AGENTS.md`，严禁在本地调用 `run_command` 执行 `cargo build` 或 `npm test`。

### 6.3 窄接线契约（Narrow Wiring Contracts）

实施子智能体向主应用接入时，必须通过以下极简、强类型的窄契约接口暴露能力：

```typescript
// 1. ProductionRenderer 窄接线属性
export interface FindInChatMountProps {
  readonly isFindInChatOpen: boolean;
  readonly activeConversationId: string | null;
  onCloseFindInChat(): void;
  onNavigateToMessage(messageId: string): void;
}

// 2. ContextInspector 抽屉挂载属性
export interface ContextInspectorMemoryMountProps {
  readonly activeAgentId: string;
  readonly accountScope: string;
  readonly isVisible: boolean;
  onOpenWikiConcept(conceptUrl: string): void;
}
```

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

为杜绝任何偷工减料、占位伪造或假逻辑充数，独立审计子智能体（Auditor Subagent）将严格依据以下**正反例检查表**进行逐项核验：

### 7.1 正反例判定检查表（Positive & Negative Inspection Oracles）

| 审计维度 | 判定准则（Pass Oracle ✅） | 违规作弊判定（Fail Violation ❌） |
|---|---|---|
| **混合检索真实性** | 并行执行 SQLite FTS5 与 Vector ANN 查询，通过数学公式 $RRF(d) = \sum \frac{w}{60 + r}$ 融合输出真实相关度分数，保留得分明细。 | 使用 `setTimeout` 模拟延迟后返回硬编码的固定消息；仅调用普通 LIKE 查询；伪造静态假分数。 |
| **真实行号引用溯源** | 仅当文档物理分块记录中包含明确的 `start_line` 与 `end_line`（$\ge 1$ 且 $\text{end} \ge \text{start}$）时，才组装输出 `#L<start>-L<end>`。 | 根据搜索摘要在字符串中的偏移量（Offset）随意除以行宽伪造行号；在摘要前无脑拼写 `#L1-L10`。 |
| **物理遗忘与墓碑防复活** | 触发 Forget 时，同步从 Markdown 文本删除、在 SQLite `memory_tombstones` 写入哈希并使向量索引失效；后续检索与合成阶段对墓碑执行 fail-closed 拦截。 | 仅在前端将事实卡片设为 `display: none`；仅在内存数组中 `splice` 删除但在刷新后被磁盘历史重新复活。 |
| **多租户与群聊安全隔离** | 记忆存储路径与 SQLite 查询显式携带 `account_id`；在群聊与公开频道环境下，`Auto` 模式强制拦截记忆引用路径输出。 | 记忆路径写死为全局单例；在群聊中向所有成员明文吐出用户的私有本地配置文件物理路径。 |
| **会话查找定位体验** | 真实遍历当前会话消息列表，计算精准命中总数与当前序号；触发平滑滚动（`scrollIntoView`）并在目标行播放黄色高亮脉冲动画。 | 悬浮条常驻显示“1 of 5”但无法跳转；点击上下箭头仅在控制台 `console.log` 而无任何 UI 定位联动。 |
| **中文 IME 输入保护** | 输入框在 `isComposing`（即拼音拼写阶段）按下 `Enter` 时，仅完成中文上屏，不触发任何跳转、提交或关闭查找栏动作。 | 用户键入拼音未上屏按回车选词时，直接触发了“跳转至下一条”或关闭了命令面板，产生严重误操作。 |
| **设计系统 Token 纯度** | 100% 使用 `--fabushi-*` 语义设计变量，文本对比度实测达到 WCAG AA 规范（$\ge 4.5:1$）。 | 出现未经别名映射的任意硬编码 Hex 色值（如 `#fff`）或残留的 `--oc-*` / `--tdrp-*` 变量。 |

### 7.2 验收门禁与静态合规基线（Acceptance Gates）

本蓝图通过以下 4 道不可逾越的验收门禁：

1. **章节完备性门禁（Gate 1: Chapter Completeness）**：
   - 必须 100% 覆盖全部 7 大强制章节，无省略、无合并、无 `TODO`/`TBD` 占位符。
2. **源码五问确证门禁（Gate 2: Source Five Verification）**：
   - Q1 至 Q5 中引用的每一个三方源码文件（Telegram C++、OpenClaw TS、Fabushi Rust/TSX）及其类名、方法名必须与物理仓库当前代码树绝对吻合，无任何凭空臆造。
3. **接口契约可编译门禁（Gate 3: Contract Strictness）**：
   - TypeScript `interface` 与 Rust `struct`/`enum` 必须具备严格类型定义，禁止出现 `any`、`unknown as any` 或空壳 trait。
4. **代码预算防巨石门禁（Gate 4: Single File 200–600 Lines Budget）**：
   - 规划的所有 5 个新建文件与 3 个修改文件严格遵守单文件 200–600 行内聚预算，杜绝产生超过 800 行的不可维护巨石文件。
