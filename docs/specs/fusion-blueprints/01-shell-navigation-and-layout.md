# Fabushi 统一融合蓝图 01：统一应用壳、一级左轨导航、二级过滤栏与响应式布局
# Volume 01: Unified Application Shell, Primary Navigation Rail, Collection Filter Bar, and Responsive Layout

> **规范编号**: `FBCP-BLUEPRINT-01` / `UFX-BLUEPRINT-01`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 根治“双左轨”、“双过滤栏”并存的架构割裂，建立唯一生产级应用壳体（Unified Application Shell）。
> 2. 规范一级左轨导航（`FabushiPrimaryNavRail`，52px 宽）：稳定承载 8 大产品领域（`messages`、`contacts`、`agents`、`tasks`、`computer`、`automations`、`marketplace`、`settings`），角标增量不影响排版几何尺寸，彻底解除 `tasks` 点击死锁。
> 3. 收口二级会话过滤栏（`FabushiCollectionFilterBar`）：消除与 `TdrpFolderFilterBar` 的同义重复，整合会话文件夹筛选（全部/未读/私聊/群组/频道/智能体/自定义分组）与 Telegram 式 Stories 动态横条（`tdrp-sidebar-stories-strip`）。
> 4. 落地严苛的响应式三阶布局断点规范（`FSP-001 §17` 与 `openclaw-theme.css §7`）：≥1200px 三栏常驻展开、900–1199px 两栏互斥滑出、<900px 单栏自适应叠放。
> 5. 全面收口 `--fabushi-*` 语义设计变量，100% 消除未经封装的平行样式系统，严格达成 WCAG AA 无障碍对比度（≥4.5:1）与中文输入法（IME）保护。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **统一应用根壳体**<br>(Unified Root Shell) | `ui/src/app-navigation.ts`<br>`src/entry.ts`<br>`ui/src/components/layout/` | `window/window_main_menu.cpp`<br>`window/window_controller.cpp`<br>`window/window_slide_animation.cpp` | `frontend/src/production/ProductionRenderer.tsx:4130-4180`<br>`source/electron-main/window-state-persistence.ts` | `ProductionRenderer.tsx` (根壳体挂载)<br>`frontend/src/production/layout/` |
| **一级左轨主导航**<br>(Primary Nav Rail) | `ui/src/app-navigation.ts:45-89`<br>`frontend/src/production/openclaw/OpenClawNavigationRail.tsx:1-181` | `window/window_main_menu.cpp:210-340`<br>`ui/widgets/side_bar_button.cpp:1-120` | `frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx:1-211`<br>`ProductionRenderer.tsx:55,4131-4147` | `FabushiPrimaryNavRail.tsx`<br>(唯一权威左轨，收口 OpenClawNavigationRail) |
| **二级集合与文件夹过滤栏**<br>(Collection Filter Bar) | `ui/src/pages/chat/session-sidebar.ts`<br>`src/sessions/folder-filter.ts` | `dialogs/dialogs_widget.cpp:320-410`<br>`data/data_chat_filters.cpp:1-480`<br>`data/data_folder.cpp:1-210` | `frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx:1-359`<br>`frontend/src/production/tdrp/TdrpFolderFilterBar.tsx:1-352`<br>`frontend/src/production/tdrp/tdrp-folder-filter.ts:1-35` | `FabushiCollectionFilterBar.tsx`<br>(废弃并归并 TdrpFolderFilterBar) |
| **Telegram Stories 动态横条**<br>(Stories Avatar Ring Strip) | `src/media/stories/`<br>`ui/src/components/stories/` | `dialogs/dialogs_stories_list.cpp:1-320`<br>`media/stories/media_stories_controller.cpp` | `frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx:312-328`<br>`frontend/src/production/tdrp/TdrpStoryPostModal.tsx` | `FabushiCollectionFilterBar.tsx`<br>`TdrpStoryPostModal.tsx` |
| **侧边栏置顶与排序算法**<br>(Pinned & Roster Ordering) | `src/sessions/pin-order.ts`<br>`packages/session-contract/` | `dialogs/dialogs_main_list.cpp:180-260`<br>`data/data_session.cpp` | `frontend/src/production/sidebar-model.ts:1-39`<br>`frontend/src/production/ProductionRenderer.tsx:1640-1653` | `sidebar-model.ts`<br>(纯函数状态计算) |
| **联系人与智能体专属表面**<br>(Contacts & Agents Surface) | `ui/src/pages/contacts.ts`<br>`src/identity/roster.ts` | `data/data_peer_values.cpp`<br>`data/data_user.cpp:310-450` | `frontend/src/production/tdrp/FabushiContactsSurface.tsx:1-250`<br>`ProductionRenderer.tsx:4148-4164` | `FabushiContactsSurface.tsx`<br>`ProductionRenderer.tsx` |
| **三阶响应式布局断点**<br>(Responsive Breakpoints) | `ui/src/styles/responsive.css`<br>`ui/src/components/split-view.ts` | `window/window_adaptive.cpp:50-180`<br>`window/window_layout.cpp` | `frontend/src/production/openclaw/openclaw-theme.css:726-840`<br>`ProductionRenderer.tsx:4130` | `openclaw-theme.css §7`<br>`ProductionRenderer.tsx` (视口状态机) |
| **设计系统与语义 Token**<br>(Design Tokens & CSS Variables) | `packages/theme-core/`<br>`ui/src/theme/tokens.css` | `ui/style/style_core.cpp:1-500`<br>`ui/colors.palette` | `frontend/src/production/openclaw/openclaw-theme.css:1-120`<br>`frontend/src/recovered/features/runtime-theme-token-installer.ts` | `openclaw-theme.css`<br>(全量收口至 `--fabushi-*`) |
| **无障碍与全键盘导航**<br>(A11y, ARIA & Keyboard Focus) | `ui/src/accessibility/focus.ts`<br>`packages/a11y-contracts/` | `ui/widgets/focus_outline.cpp`<br>`window/window_main_menu.cpp:400-450` | `frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx:119-174`<br>`openclaw-theme.css:41-44,104-106` | 全局 FocusRing、ARIA 读屏与快捷键管理器 |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - 应用主窗口由 `Telegram/SourceFiles/window/window_controller.cpp` 统管，视口自适应通过 `window/window_adaptive.cpp` 实现，窗口宽度依据断点计算 `Adaptive::WindowLayout`（`OneColumn`, `Normal`, `ThreeColumn`）。
  - 会话列表与过滤由 `dialogs/dialogs_widget.cpp`、`dialogs/dialogs_main_list.cpp` 和 `data/data_chat_filters.cpp` 驱动，支持依据群组、频道、机器人、未读、自定义标签对会话进行分片计算，并支持未读角标聚合与置顶会话固定（Pinned Chats）。
  - 动态横条由 `dialogs/dialogs_stories_list.cpp` 渲染，以水平滚动条（Stories Strip）展示联系人最近发布的图片/视频动态，支持已读/未读渐变色彩环高亮。
- **OpenClaw 侧**：
  - 前端导航基于 `ui/src/app-navigation.ts`，定义了工作区（Workspace）、应用（Apps）、会话（Chat）、联系人（Contacts）、网关（Gateway）等主要视图的路由映射。
  - 在早期独立原型中，OpenClaw 通过 `OpenClawNavigationRail.tsx` 自带一套 64px 宽度的深色左轨与红圈新建按钮；右侧配有独立的 `OpenClawTopHeader` 与 `OpenClawAskCopilotPanel`。

#### Q2（现有链路定位）
- **应用壳体渲染链路**：
  - `source/electron-main/entry.ts` 启动 Electron 主窗口 -> 加载 `frontend/src/production/ProductionRenderer.tsx`。
  - `ProductionRenderer.tsx:4130` 声明 CSS Grid 根布局：`gridTemplateColumns: 52px ${renderedSidebarWidth}px minmax(0, 1fr)`。
  - 第一列挂载 `FabushiPrimaryNavRail`（`frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx`）。
  - 第二列顶层挂载 `FabushiCollectionFilterBar`（`frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx`），下方挂载 `ConversationSidebar`。
  - 第三列根据 `activeNavDomain` 与 `activeAgentId` 挂载 `ConversationWorkspace` 或 `FabushiContactsSurface`，并在最右侧根据媒体查询与状态挂载抽屉。

#### Q3（断点精确诊断）
1. **左轨双轨割裂断点**（`OpenClawNavigationRail.tsx` vs `FabushiPrimaryNavRail.tsx`）：
   - 在历史代码中存在 `frontend/src/production/openclaw/OpenClawNavigationRail.tsx`（181 行）与 `frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx`（211 行）两套平行的左轨组件。前者使用独立的 `oc-nav-rail` 类名与硬编码绿色/红色状态指示点，后者承载了 Fabushi 的主品牌 Logo 与 8 大核心领域。必须正式将前者废止并确立 `FabushiPrimaryNavRail` 为唯一 Canonical Owner。
2. **过滤栏双重实现断点**（`TdrpFolderFilterBar.tsx` vs `FabushiCollectionFilterBar.tsx`）：
   - `frontend/src/production/tdrp/TdrpFolderFilterBar.tsx`（352 行）与 `frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx`（359 行）在同一目录下共存，两者的 `customFolders` 分别写入了不同的 localStorage 键（`fabushi_tdrp_custom_folders_v1` 与 `fabushi_collection_custom_folders_v1`）。`ProductionRenderer.tsx:4170` 实际使用的是 `FabushiCollectionFilterBar`，导致 `TdrpFolderFilterBar` 成为未接线（`not-wired`）且状态割裂的僵尸组件。
3. **Tasks 点击门控死锁断点**（`ProductionRenderer.tsx:4140`）：
   ```tsx
   else if (domain === "tasks" && activeAgent != null && !activeIsHuman) openAsyncTasks(activeAgent.id);
   ```
   - 点击左轨的 `tasks` 领域图标时，逻辑被绑定在当前会话是否为非真人 Agent（`!activeIsHuman`）。若用户当前未选中会话或正在真人聊天，点击 `tasks` 毫无反应。必须重构成无条件切换主工作区为 `GlobalTasksWorkspace`。
4. **硬编码响应式 CSS 与内联样式的竞态断点**（`ProductionRenderer.tsx:4130` 与 `openclaw-theme.css:733-838`）：
   - `ProductionRenderer.tsx:4130` 的行内样式直接写死了 `gridTemplateColumns: 52px ${renderedSidebarWidth}px minmax(0, 1fr)`，没有感知窗口媒体查询宽度，导致在宽度小于 900px 时，三列依然被强行挤在视口内，产生水平滚动溢出；而 `openclaw-theme.css` 中的媒体查询仅控制了 `.oc-copilot-drawer` 的绝对/固定定位，外层 Grid 容器未能同步响应。

#### Q4（状态机与安全防护）
- **左轨域切换与账号隔离状态转移方程**：
  $$\text{NavDomain} \in \{\text{messages}, \text{contacts}, \text{agents}, \text{tasks}, \text{computer}, \text{automations}, \text{marketplace}, \text{settings}\}$$
  - 当切换账号或登出时，触发 `accountScopeGeneration` 单调递增：
    $$\text{Generation} \leftarrow \text{Generation} + 1$$
  - 所有局部持久化状态（自定义文件夹 `customFolders`、展开折叠状态 `sidebarCollapseStore`、选中的活跃文件夹 `activeFolderId`）必须带上 `accountScope` 前缀做键名命名空间隔离：
    $$\text{StorageKey} = \text{fabushi\_folders\_v1\_} + \text{sha256}(\text{account.id})$$
  - 禁止在账号 A 下读取或保存账号 B 的会话文件夹或草稿状态。
- **角标计算的原子单调性**：
  - 未读角标必须由纯函数聚合（`unreadFolderCount`），并在后端推送 `message.delivered` 或 `message.read` 事件时原子更新，严禁在前端通过任意 `setTimeout` 或局部计数器产生角标闪烁或负数状态。

#### Q5（UI/UX 极致融合落点）
- **一级左轨（Primary Nav Rail）**：位于界面最左侧，宽度 52px。顶部居中放置 32x32 Monogram 品牌 Logo（Fabushi 渐变）；中间由上至下垂直排布 7 个领域图标（消息、联系人、智能体、任务、Computer、自动化、插件市场）；底部放置账号与设置（Settings）图标。选中态呈现浅蓝色底色（`rgba(16, 132, 254, 0.12)`）与高亮主题蓝（`#1084fe` / `var(--fabushi-accent-primary)`）。
- **二级过滤栏（Collection Filter Bar）**：位于第二列顶部，高度紧凑自适应。第一行横向胶囊（Pill Tabs）支持平滑滚动，并右侧常驻“新建分组（FolderPlus）”与“新建会话（Plus）”两个 24x24 图标按钮；第二行优雅集成 Telegram Stories 动态圈，支持头像环与发布动态入口。
- **自适应视口管理**：
  - 宽屏（≥1200px）：全展开 3 栏常驻；
  - 中屏（900–1199px）：左轨保持 52px，右侧详情栏自动收为滑出浮层（Drawer Overlay），主消息区自适应拉伸；
  - 窄屏（<900px）：左轨紧凑 48px，侧边栏与消息区实行单一工作区推拉（Slide Transition），保障单手触控和移动/小屏体验。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-01-01** | 左轨双组件并存，样式与事件系统两套并行 | `frontend/src/production/openclaw/OpenClawNavigationRail.tsx:1-181`<br>`frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx:1-211` | 架构融合初期遗留的未收口组件。OpenClaw 原型使用 `oc-nav-rail`，Fabushi 生产代码使用 `FabushiPrimaryNavRail`，导致维护者容易引用错误组件，且样式变量未统一。 | **P0 架构双轨** |
| **BUG-01-02** | 会话过滤栏双重实现，LocalStorage 键冲突与功能漂移 | `frontend/src/production/tdrp/TdrpFolderFilterBar.tsx:22,43-100`<br>`frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx:42,80-134` | 存在两个极其相似的过滤器。`TdrpFolderFilterBar` 保存到 `fabushi_tdrp_custom_folders_v1`，而 `FabushiCollectionFilterBar` 保存到 `fabushi_collection_custom_folders_v1`。前者缺少 Stories 动态横条且已被弃用，但仍停留在代码树中。 | **P1 数据割裂** |
| **BUG-01-03** | 根布局 Grid 写死内联样式，导致窄屏视口横向滚动溢出 | `frontend/src/production/ProductionRenderer.tsx:4130` | `gridTemplateColumns: 52px ${renderedSidebarWidth}px minmax(0, 1fr)` 仅由 JS 内联计算，未结合 CSS 媒体查询断点（Media Queries）进行三阶布局降级，窄屏下无法自动隐藏或折叠侧边栏。 | **P1 布局缺陷** |
| **BUG-01-04** | Tasks 导航死锁：仅在当前会话为非真人时方可响应 | `frontend/src/production/ProductionRenderer.tsx:4140` | `else if (domain === "tasks" && activeAgent != null && !activeIsHuman) openAsyncTasks(activeAgent.id)` 将全局一级的任务中心降级为当前 Agent 的从属抽屉，在未选会话或真人会话时点击完全无响应。 | **P0 交互死锁** |
| **BUG-01-05** | 自定义文件夹缺少跨账号隔离，换号登录导致数据串扰 | `frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx:42,82` | `STORAGE_KEY = "fabushi_collection_custom_folders_v1"` 为全局静态常量，未包含 `account.id` 隔离维度。当用户注销并登录新账号时，旧账号创建的分组仍被加载显示。 | **P1 安全与隔离** |
| **BUG-01-06** | 语义 Token 存在局部未收口的 `--oc-*` 变量引用 | `frontend/src/production/openclaw/openclaw-theme.css:46-64,107-120` | 虽然在 `:root` 中将部分 `--oc-*` 别名映射到了 `--fabushi-*`，但在部分组件内仍存在直接消费 `--oc-*` 的情况，缺乏全局一致的 Lint 防护，容易导致浅色模式下对比度失效。 | **P2 规范风险** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 架构设计与单一权威归属

为终结上述双轨并存和割裂问题，本蓝图确立 **唯一应用壳体与分层导航架构（Single-Shell Multi-Tier Navigation）**：

```
┌────────────────────────────────────────────────────────────────────────────────────────────────────────┐
│                                   ProductionRenderer (Root Shell)                                      │
├───────────────┬──────────────────────────────────┬─────────────────────────────────────────────────────┤
│ 1. 一级左轨   │ 2. 二级侧边栏 (Sidebar Column)   │ 3. 三级主舞台与检查器 (Stage & Inspector)           │
│ (Primary Rail)│                                  │                                                     │
│ 52px / 48px   │ 260px - 400px (可拖拽调节)       │ minmax(0, 1fr)                                      │
├───────────────┼──────────────────────────────────┼─────────────────────────────────────────────────────┤
│ Brand Logo    │ FabushiCollectionFilterBar       │ [activeNavDomain 路由分发]                          │
│ ───────────── │ ├─ Tab 栏 (全部/未读/群组/Agent) │ ├─ "messages"    -> ConversationWorkspace           │
│ Messages (💬) │ └─ Telegram Stories 横条         │ ├─ "contacts"    -> FabushiContactsSurface          │
│ Contacts (👥) │                                  │ ├─ "agents"      -> Agent Catalog / Workspace       │
│ Agents   (🤖) │ ConversationSidebar              │ ├─ "tasks"       -> GlobalTasksWorkspace (无条件)   │
│ Tasks    (✅) │ ├─ Search Bar                    │ ├─ "computer"    -> Computer Surface / Banner       │
│ Computer (💻) │ ├─ Pinned Section                │ ├─ "automations" -> Routines & Cron Workspace       │
│ Automations(⚡)│ └─ Roster List (虚拟化渲染)       │ ├─ "marketplace" -> Plugin Marketplace Overlay      │
│ Market   (🛍️) │                                  │ └─ "settings"    -> Settings Modal Overlay          │
│ ───────────── │                                  │ ─────────────────────────────────────────────────── │
│ Settings (⚙️) │                                  │ [右侧上下文抽屉 ContextInspector (≥1200px 自动展开)] │
└───────────────┴──────────────────────────────────┴─────────────────────────────────────────────────────┘
```

### 3.2 权威文件清单与行数预算

遵循 `Same Owner != Same File` 原则，所有新增与重构代码严格控制在内聚预算（**200–600 行**）内：

1. **`frontend/src/production/ProductionRenderer.tsx`**（行数精简与窄接线收口）：
   - 职责：作为顶层 Root Shell 挂载点，仅负责消费 `useResponsiveLayout` 状态，组装外层三栏 CSS Grid，分发 `activeNavDomain`。将冗余的内联样式与抽屉竞争状态收口。
   - 预算管控：下调内联逻辑，下发至独立 Hook。
2. **`frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx`**（唯一左轨 Canonical Owner）：
   - 职责：展示品牌 Logo、8 大领域图标、未读计数角标、底部设置图标、全键盘无障碍导航（Tab / 方向键）。
   - 预算：约 **220–280 行**（高内聚组件）。
3. **`frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx`**（唯一会话过滤与动态栏）：
   - 职责：承载会话分类 Pills、新建分组模态框触发、新建会话触发、Stories 横向动态圈及查看器。集成账号隔离的存储键。
   - 预算：约 **320–380 行**。
4. **`frontend/src/production/layout/useResponsiveLayout.ts`**（新增响应式状态管理 Hook）：
   - 职责：监听 `window.matchMedia`，管理三阶断点（Wide ≥1200 / Standard 900–1199 / Compact <900），导出当前布局模式（`tier: "wide" | "standard" | "compact"`）与侧边栏/抽屉展开折叠状态。
   - 预算：约 **120–180 行**。
5. **`frontend/src/production/openclaw/openclaw-theme.css`**（权威样式与 Token 定义）：
   - 职责：定义全局 `--fabushi-*` 变量、深浅色模式自适应映射、三阶媒体查询响应式规则。彻底废弃独立未别名的 `--oc-*` 选择器。
   - 预算：维持整体结构清晰，集中补强 §7 响应式断点规则。
6. **废弃与清理文件清单**：
   - `frontend/src/production/openclaw/OpenClawNavigationRail.tsx`（标记废弃，移除生产引用）。
   - `frontend/src/production/tdrp/TdrpFolderFilterBar.tsx`（标记废弃，由 `FabushiCollectionFilterBar.tsx` 完全承接）。


---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 界面布局与四级渐进披露层级

根据 `canonical-screen-patterns.md (FSP-001)` 核心规范，统一应用壳体遵循标准的四级渐进披露（Progressive Disclosure）语法：

```
Level 1: 稳定一级左轨 (Primary Navigation Rail, 52px)
  └── 包含应用标识、核心业务领域图标（消息、联系人、智能体、任务、Computer、自动化、插件市场）与全局设置。
Level 2: 二级集合与检索侧栏 (Collection / Sidebar Column, 260px–400px)
  └── 包含顶部胶囊过滤器（全部/未读/私聊/群组/频道/智能体/自定义分组）、Stories 动态横条、实时搜索框与会话列表。
Level 3: 三级主工作舞台 (Canonical Conversation Workspace, minmax(0, 1fr))
  └── 包含唯一会话顶栏（Avatar + Title + Presence + Action Buttons）、消息流虚拟列表与权威统一输入台（Composer）。
Level 4: 四级上下文与能力检查器 (Context Inspector Drawer, 360px–420px)
  └── 包含对象资料（Profile）、群成员列表、共享媒体（Shared Media）、智能体技能/模型设置及子任务跟踪。
```

- **交互设计铁律**：
  - 一级左轨图标必须具备确定的全局动作，严禁将一级导航点击绑定在局部子状态（如必须选中特定类型会话）上；
  - 任何弹出的模态框（如新建会话、新建文件夹、发布 Stories、插件市场）关闭后，焦点必须准确恢复到触发它的原始按钮（Focus Return）；
  - 侧边栏拖拽调宽（Resize Handle）必须带有防抖与范围约束（260px ≤ 宽度 ≤ 400px），并在拖拽过程中添加全局遮罩防止 iframe / Webview 捕获鼠标事件。

---

### 4.2 响应式三阶布局断点规范（Responsive Breakpoints）

严格遵循 `FSP-001 §17` 与 `openclaw-theme.css §7`：

| 布局层级 (Tier) | 视口宽度断点 | 左轨 (Primary Rail) | 侧边栏 (Sidebar) | 主工作区 (Workspace) | 右侧抽屉 (Context Inspector) |
|---|---|---|---|---|---|
| **Tier 1: 宽屏模式 (Wide)** | `W ≥ 1200px` | 52px 常驻展开 | 260px–400px 常驻展开 | 弹性自适应占据剩余宽度 | 360px–420px 常驻内嵌（Docked），无遮罩，不遮挡主消息流 |
| **Tier 2: 标准模式 (Standard)** | `900px ≤ W < 1200px` | 52px 常驻展开 | 260px–340px 常驻展开 | 弹性自适应占据剩余宽度 | 360px 悬浮滑出抽屉（Overlay Drawer），附带阴影与点击半透明遮罩，互斥展示 |
| **Tier 3: 紧凑模式 (Compact)** | `W < 900px` | 48px 紧凑收起或汉堡折叠 | 折叠为滑动视图（Slide Panel） | 单栏全屏展示当前活跃会话 | 100% 满屏滑动弹层（Full-width Modal Sheet），支持滑动返回手势 |

- **断点切换平滑性保障**：
  - 使用 CSS 媒体查询与 `useResponsiveLayout` 共同控制，禁止在 Resize 时触发全树重渲染（Zero Layout Shift）；
  - 侧边栏拖拽尺寸在从 Tier 1 缩放到 Tier 2/3 时自动持久化记忆，待恢复到 Tier 1 时无缝还原用户自定义宽度。

---

### 4.3 状态矩阵与异常反馈

应用壳体必须全面覆盖以下 8 种标准状态：

```
┌──────────────────┬────────────────────────────────────────────────────────────────────────────────────────┐
│ 状态类别         │ 表现与交互反馈                                                                         │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 1. Loading       │ 左轨展示占位骨架或品牌 Logo 呼吸动效（1.5s 循环）；侧边栏展示骨架波纹（Shimmer）；      │
│                  │ 顶栏展示轻量加载进度条（Top Shell Loading Bar，高 2px）。                               │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 2. Empty         │ 当某一文件夹（如“未读”或自定义标签）无匹配会话时，侧边栏显示标准 EmptyState：          │
│                  │ 居中插画 + “暂无相关对话” + “返回全部会话”操作按钮，严禁展示空白死屏。                 │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 3. Ready         │ 正常渲染左轨、会话列表与主工作区；激活态项具备高亮视觉焦点。                           │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 4. Offline       │ 顶部展示全局轻量离线横幅（Offline Banner）：“已断开连接，正在尝试重连...”；           │
│                  │ 左轨与会话列表保留只读缓存，左轨头像状态红点告警，操作进入排队队列（Outbox Queue）。   │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 5. Reconnecting  │ 横幅转为橙色“正在同步最新消息...”，配合旋转指示器，禁止触发多次全量拉取。               │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 6. Waiting Auth  │ 当凭据过期或网关需要授权时，左轨底部设置图标出现黄色告警角标，点击直达认证中心。       │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 7. Error         │ 局部组件崩溃（ErrorBoundary 捕获），仅在对应分栏显示“模块加载失败”及“重试”按钮，        │
│                  │ 严禁整个 Electron 窗口白屏（Root Crash Protection）。                                   │
├──────────────────┼────────────────────────────────────────────────────────────────────────────────────────┤
│ 8. Large Data    │ 会话数量超过 1000+ 时，列表自动激活动态虚拟化（Virtual Scroll），只渲染视口内 20 条，  │
│                  │ 内存占用稳定在 <50MB，滚动帧率保持 60fps。                                             │
└──────────────────┴────────────────────────────────────────────────────────────────────────────────────────┘
```

---

### 4.4 动效与过渡规范

- **左轨选中切换**：背景色与高亮条过渡采用 `transition: all 0.15s cubic-bezier(0.4, 0, 0.2, 1)`；
- **抽屉滑入滑出**：右侧抽屉进入动画为 `transform: translateX(0)`，耗时 `200ms ease-out`；退出为 `transform: translateX(100%)`，耗时 `150ms ease-in`；
- **未读角标跳动**：新消息到达时，角标执行轻微缩放呼吸动效（`scale(1.15) -> scale(1.0)`，耗时 180ms），禁止剧烈抖动打扰用户；
- **减少动效支持**：严格尊重系统级 `prefers-reduced-motion: reduce` 设置，当开启时自动禁用所有位移与缩放过渡，直接离散切换。

---

### 4.5 全键盘导航、焦点恢复与中文 IME 保护

1. **快捷键系统**：
   - `Cmd / Ctrl + 1`: 切换至“消息（Messages）”
   - `Cmd / Ctrl + 2`: 切换至“联系人（Contacts）”
   - `Cmd / Ctrl + 3`: 切换至“智能体（Agents）”
   - `Cmd / Ctrl + 4`: 切换至“任务（Tasks）”
   - `Cmd / Ctrl + 5`: 切换至“Computer”
   - `Cmd / Ctrl + 6`: 切换至“自动化（Automations）”
   - `Cmd / Ctrl + 7`: 切换至“插件市场（Marketplace）”
   - `Cmd / Ctrl + 8` / `Cmd / Ctrl + ,`: 打开“设置（Settings）”
   - `Cmd / Ctrl + K`: 呼出全局命令面板（Command Palette / Universal Search）
   - `Esc`: 关闭当前打开的任何抽屉或浮层，并将焦点恢复至原触发位置。
2. **方向键与 Tab 导航流**：
   - 在左轨中按下 `Tab` 键聚焦首个领域项；
   - 使用 `ArrowUp` / `ArrowDown` 键在各个领域项之间单循环切换焦点，按 `Enter` 或 `Space` 确认激活；
   - 具有正确的 ARIA 属性：`aria-current="page"` 标注当前激活页，未读角标配置 `aria-live="polite"` 读屏支持。
3. **中文输入法（IME）保护机制**：
   - 过滤栏搜索框与新建分组输入框中，必须严格判断 `e.nativeEvent.isComposing`。在输入法候选拼音未确认（按空格选字）之前，严禁触发 `Enter` 快捷提交或过滤重置，彻底杜绝中文拼音中途被打断提交的问题。

---

### 4.6 语义设计变量与深浅色 WCAG AA 映射表

严禁在组件内写死任何十六进制颜色或未封装变量，所有视觉元素统一绑定 `--fabushi-*` 语义 Token：

| 语义 Token | 浅色模式值 (Light) | 深色模式值 (Dark) | 语义用途与对比度保证 |
|---|---|---|---|
| `--fabushi-bg-canvas` | `#ffffff` | `#1c1c1e` | 顶级应用窗口与工作区画布背景 |
| `--fabushi-bg-surface` | `#f7f7f8` | `#252528` | 卡片、工具栏、浮层背景 |
| `--fabushi-bg-sidebar` | `#f0f0f2` | `#18181a` | 侧边栏与左轨主背景 |
| `--fabushi-border` | `rgba(0, 0, 0, 0.08)` | `rgba(255, 255, 255, 0.09)` | 常规分隔线与内嵌描边 |
| `--fabushi-border-strong`| `rgba(0, 0, 0, 0.16)` | `rgba(255, 255, 255, 0.16)` | 悬停态或活动状态边界 |
| `--fabushi-text-primary` | `#1d1d1f` | `#f5f5f7` | 主标题与正文文本（对比度 ≥ 12:1） |
| `--fabushi-text-secondary`| `#555558` | `#a1a1a6` | 副标题、时间戳与标签文本（对比度 ≥ 4.8:1，符合 WCAG AA） |
| `--fabushi-text-muted` | `#8e887e` | `#86868b` | 占位符与禁用文本 |
| `--fabushi-accent-primary`| `#1084fe` | `#388bfd` | 主品牌操作蓝、激活态图标与选中胶囊 |
| `--fabushi-accent-coral` | `#d9534f` | `#e56a66` | 未读告警、红点与删除操作 |
| `--fabushi-focus-ring` | `#0071e3` | `#2997ff` | 全局无障碍焦点外发光环（2px solid） |
| `--fabushi-touch-target-min`| `44px` | `44px` | 移动触控与小屏标准最小点击热区 |

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端核心 TypeScript 类型与 Props 契约

#### 1. 左轨领域定义与组件 Props
```typescript
/**
 * Fabushi 核心一级产品领域
 */
export type FabushiPrimaryNavDomain =
  | "messages"      // 会话与聊天
  | "contacts"      // 通讯录与联系人
  | "agents"        // 智能体工作目录
  | "tasks"         // 全局跨会话任务与审批中心
  | "computer"      // 远程设备与桌面控制
  | "automations"   // 定时任务与工作流
  | "marketplace"   // 插件与 MCP 市场
  | "settings";     // 账号与全局偏好设置

export interface FabushiPrimaryNavRailProps {
  /** 当前激活的产品领域 */
  readonly activeDomain: FabushiPrimaryNavDomain;
  /** 切换领域回调（无条件全局导航） */
  readonly onSelectDomain: (domain: FabushiPrimaryNavDomain) => void;
  /** 全局未读消息聚合计数 */
  readonly unreadMessagesCount?: number;
  /** 打开系统设置模态框 */
  readonly onOpenSettings?: () => void;
  /** 打开插件市场 */
  readonly onOpenMarketplace?: () => void;
  /** 品牌角标文字或头像 Monogram */
  readonly userAvatarText?: string;
  /** 紧凑模式开关（窄屏 <900px 时为 true） */
  readonly isCompact?: boolean;
}
```

#### 2. 二级过滤栏接口与数据模型
```typescript
export interface FabushiFilterFolder {
  readonly id: string;
  readonly name: string;
  readonly icon?: string;
  readonly badge?: number;
  readonly isCustom?: boolean;
}

export interface FabushiCollectionFilterBarProps {
  /** 当前选中的文件夹/标签 ID（如 'all', 'unread', 'direct', 'groups', 'channels', 'agents' 或自定义 ID） */
  readonly activeFolderId: string;
  /** 切换文件夹回调 */
  readonly onSelectFolder: (folderId: string) => void;
  /** 打开新建会话对话框 */
  readonly onOpenNewChat: () => void;
  /** 打开侧边栏搜索框 */
  readonly onOpenSearch?: () => void;
  /** 统计计数汇总 */
  readonly unreadCount?: number;
  readonly directCount?: number;
  readonly groupCount?: number;
  readonly channelCount?: number;
  readonly agentCount?: number;
  /** 协调器客户端（用于同步云端文件夹与动态） */
  readonly client?: ProductionCoordinatorClient | null;
  /** 容器自定义样式 */
  readonly style?: React.CSSProperties;
}
```

#### 3. 响应式视口状态管理契约
```typescript
export type ResponsiveLayoutTier = "wide" | "standard" | "compact";

export interface ResponsiveLayoutState {
  readonly tier: ResponsiveLayoutTier;
  readonly windowWidth: number;
  readonly windowHeight: number;
  readonly isSidebarVisible: boolean;
  readonly isDrawerDocked: boolean; // 是否内嵌展示抽屉
  readonly isDrawerVisible: boolean;
  readonly setSidebarVisible: (visible: boolean) => void;
  readonly setDrawerVisible: (visible: boolean) => void;
}
```

---

### 5.2 后端 Coordinator / Rust RPC 协议契约

前端会话过滤器与 Stories 动态横条通过 Coordinator 客户端与 Rust 后端（`source/host`）保持无缝双向同步：

#### 1. 文件夹列表与创建 RPC
- **`folders.list`**:
  ```json
  // Request
  {}
  // Response
  {
    "folders": [
      {
        "id": "folder_work",
        "name": "工作项目",
        "icon": "💼",
        "colorIndex": 1,
        "flags": 0,
        "alwaysPeers": ["peer_agent_12", "peer_group_8"],
        "neverPeers": []
      }
    ]
  }
  ```
- **`folders.create`**:
  ```json
  // Request
  {
    "folder": {
      "id": "folder_1730812345678",
      "title": "VIP 客户",
      "colorIndex": 2,
      "flags": 0,
      "alwaysPeers": ["peer_human_44"],
      "neverPeers": []
    }
  }
  // Response: { "success": true, "id": "folder_1730812345678" }
  ```
- **`folders.delete`**:
  ```json
  // Request: { "folderId": "folder_1730812345678" }
  // Response: { "success": true }
  ```

#### 2. Stories 动态发布与拉取 RPC
- **`stories.list`**:
  ```json
  // Request: {}
  // Response
  {
    "stories": [
      {
        "id": "story_981",
        "authorPeerId": "peer_human_1",
        "authorName": "Alice",
        "content": "全球法布施新版本发布！",
        "mediaUrl": "sand://resources/stories/story_981.jpg",
        "timestamp": 1730812000000,
        "isRead": false
      }
    ]
  }
  ```
- **`stories.publish`**:
  ```json
  // Request
  {
    "content": "今天探索了新的智能体工作流！",
    "mediaUri": "sand://resources/tmp/snapshot_01.png",
    "privacy": "contacts"
  }
  // Response: { "success": true, "storyId": "story_982" }
  ```

- **Rust Host RPC 契约定义（`source/host/src/extensions/session/production.rs`）**：
  ```rust
  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct FabushiCustomFolderSpec {
      pub id: String,
      pub title: String,
      pub included_peer_ids: Vec<String>,
      pub excluded_peer_ids: Vec<String>,
      pub include_contacts: bool,
      pub include_groups: bool,
      pub include_bots: bool,
      pub account_scope_generation: u64,
  }

  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub enum StoryPrivacyScope {
      Everyone,
      Contacts,
      CloseFriends,
      SelectedPeers(Vec<String>),
  }

  #[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
  #[serde(rename_all = "camelCase")]
  pub struct StorySummaryItem {
      pub id: String,
      pub author_peer_id: String,
      pub author_name: String,
      pub content: String,
      pub media_url: Option<String>,
      pub timestamp_ms: i64,
      pub is_read: bool,
      pub privacy: StoryPrivacyScope,
  }
  ```

---

### 5.3 跨账号 / 跨会话 Fencing 与状态隔离方程

为防止多账号登录或快速切换会话时，产生前一个账号的缓存数据污染新账号的界面（Cross-Account State Leak），系统定义如下 Fencing 机制：

1. **状态机变量定义**：
   - 设当前登录账号代数计数器为 $G_{\text{account}} \in \mathbb{N}$（由 `accountScopeGeneration` 单调递增）；
   - 设当前活跃账号 ID 为 $A_{\text{id}} \in \text{String}$；
   - 设本地存储键函数为：
     $$\text{Key}(\text{domain}, A_{\text{id}}) = \text{"fabushi\_"} + \text{domain} + \text{"\_"} + \text{sha256}(A_{\text{id}})[:16]$$
2. **状态转移与隔离方程**：
   - **当切换账号或用户登出时**：
     $$G_{\text{account}} \leftarrow G_{\text{account}} + 1$$
     $$\text{activeFolderId} \leftarrow \text{"all"}$$
     $$\text{customFolders} \leftarrow \text{Load}(\text{Key}(\text{"custom\_folders"}, A_{\text{id}}^{\text{new}})) \lor [\,]$$
     $$\text{activeNavDomain} \leftarrow \text{"messages"}$$
   - **异步 RPC 响应拦截门控（Generation Fencing Rule）**：
     对于任何在途未完成的 RPC 回调（如 `folders.list.then(res => ...)` 或 `stories.list.then(res => ...)`）：
     $$\text{IF } G_{\text{request}} \neq G_{\text{current}} \implies \text{DROP}(\text{res}) \quad (\text{拒绝写入 React State})$$
     从数学与逻辑层面彻底杜绝已注销账号的慢速网络请求覆盖新账号的会话分组与动态数据。

---

### 5.4 视口断点与布局状态转移机（Layout FSM）

```
                     ┌──────────────────────────────────────────────┐
                     │               Window Initialized             │
                     └──────────────────────┬───────────────────────┘
                                            │
                                            ▼
       ┌────────────────────────────────────────────────────────────────────────┐
       │                 useResponsiveLayout (Viewport Monitor)                 │
       └───────┬────────────────────────────┼───────────────────────────┬───────┘
               │ W ≥ 1200px                 │ 900px ≤ W < 1200px        │ W < 900px
               ▼                            ▼                           ▼
    ┌──────────────────────┐    ┌──────────────────────┐    ┌──────────────────────┐
    │     Tier 1: Wide     │    │   Tier 2: Standard   │    │   Tier 3: Compact    │
    ├──────────────────────┤    ├──────────────────────┤    ├──────────────────────┤
    │ - Left Rail: 52px    │    │ - Left Rail: 52px    │    │ - Left Rail: 48px    │
    │ - Sidebar: 260-400px │    │ - Sidebar: 260-340px │    │ - Sidebar: Overlay   │
    │ - Workspace: 1fr     │    │ - Workspace: 1fr     │    │ - Workspace: 100%    │
    │ - Drawer: 360px Dock │    │ - Drawer: Overlay    │    │ - Drawer: Full Modal │
    └──────────────────────┘    └──────────────────────┘    └──────────────────────┘
```

- **状态转移规则**：
  - 当视口从 Tier 1 缩窄至 Tier 2 时，内嵌的 `ContextInspector` 自动转为收起状态；若用户主动点击右上角详情按钮，则以绝对定位浮层（Overlay）滑出，并带有 `z-index: 25` 与半透明背板；
  - 当视口缩窄至 Tier 3 时，侧边栏与主工作区采用单栏推拉模式，左轨仅保留 48px 图标模式或可被抽屉覆盖；
  - 当视口从小屏恢复至宽屏时，原处于 Docked 状态的抽屉恢复常驻展示，无需用户重复点击开启。


---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 子智能体修改文件白名单与只读参考表

为保障系统重构的确定性与代码整洁，所有落实该蓝图的实施子智能体（Implementation Workers）必须严格遵守以下文件读写边界：

#### 允许修改的文件白名单（Write Whitelist）
1. `frontend/src/production/tdrp/FabushiPrimaryNavRail.tsx`
   - 目标：补齐无障碍 ARIA 属性、全键盘方向键循环支持、确保 8 大领域无条件分发，维持单文件行数在 250 行左右。
2. `frontend/src/production/tdrp/FabushiCollectionFilterBar.tsx`
   - 目标：引入 `account.id` 隔离的 `STORAGE_KEY`，接入 RPC 响应的 `accountScopeGeneration` 门控，彻底废除全局裸键。
3. `frontend/src/production/layout/useResponsiveLayout.ts`（新增独立 Hook）
   - 目标：抽象窗口媒体查询（`window.matchMedia`）状态，统一管理 Tier 1/2/3 视口模式与抽屉/侧边栏展开状态。
4. `frontend/src/production/ProductionRenderer.tsx`
   - 目标：通过极窄接线（Narrow Wiring）引入 `useResponsiveLayout`；解除 `tasks` 领域点击时的 `!activeIsHuman` 门控；彻底移除对 `OpenClawNavigationRail` 的任何潜在引用，收口根 Grid 类名。
5. `frontend/src/production/openclaw/openclaw-theme.css`
   - 目标：强化 §7 三阶媒体查询样式（`.fabushi-shell-grid` / `.fabushi-primary-nav-rail` / 响应式抽屉），确保 `--oc-*` 100% 映射至 `--fabushi-*`。
6. `frontend/src/production/openclaw/OpenClawNavigationRail.tsx` 与 `frontend/src/production/tdrp/TdrpFolderFilterBar.tsx`
   - 目标：标记 `@deprecated` 注解，清空内部私有逻辑，重定向或导出为占位契约，待最终集成阶段安全归档删除。

#### 只读参考文件（Read-Only Reference Files，严禁篡改）
- `projects/fabushi-communication-platform/canonical-screen-patterns.md`（屏幕范式母本）
- `projects/fabushi-communication-platform/current-owner-inventory.md`（责任清单）
- `frontend/src/production/sidebar-model.ts`（置顶与会话列表纯函数模型）
- `frontend/src/production/tdrp/tdrp-folder-filter.ts`（纯函数过滤逻辑）
- `source/electron-main/window-state-persistence.ts`（主进程窗口状态持久化）

---

### 6.2 与 `ProductionRenderer.tsx` 的极窄接线契约

在 `ProductionRenderer.tsx` 中，应用壳体与导航的挂载必须保持极致精简，严禁在顶层组件中堆砌大段复杂的布局与过滤逻辑：

```tsx
// 窄接线示例契约 (Narrow Wiring in ProductionRenderer.tsx)
import { FabushiPrimaryNavRail } from "./tdrp/FabushiPrimaryNavRail";
import { FabushiCollectionFilterBar } from "./tdrp/FabushiCollectionFilterBar";
import { useResponsiveLayout } from "./layout/useResponsiveLayout";

export const ProductionRenderer: React.FC = () => {
  // 1. 响应式布局状态机 Hook
  const { tier, isSidebarVisible, isDrawerDocked } = useResponsiveLayout();

  // 2. 一级导航切换调度（严格解除 tasks 死锁）
  const handleSelectNavDomain = useCallback((domain: FabushiPrimaryNavDomain) => {
    setActiveNavDomain(domain);
    if (domain === "computer") {
      computer.open();
    } else {
      computer.close();
      if (domain === "agents") setActiveFolderId("agents");
      else if (domain === "messages") setActiveFolderId("all");
      // 关键修复：无条件全局任务导航，不再检查 activeAgent 或 !activeIsHuman
      else if (domain === "tasks") setActiveNavDomain("tasks");
    }
  }, [computer]);

  return (
    <div className={`fabushi-root-shell fabushi-tier-${tier}`}>
      <div className="fabushi-shell-grid">
        {/* 一级左轨：唯一挂载点 */}
        <FabushiPrimaryNavRail
          activeDomain={activeNavDomain}
          onSelectDomain={handleSelectNavDomain}
          unreadMessagesCount={unreadFolderCount}
          onOpenMarketplace={() => { setPluginQuery(""); setOverlay("plugins"); }}
          onOpenSettings={() => { setSettingsSection("general"); setOverlay("settings"); }}
          userAvatarText={account?.kind === "logged-in" ? (account.email?.[0] || "F") : "F"}
          isCompact={tier === "compact"}
        />

        {/* 二级侧边栏列 (按领域渲染或在窄屏时受控收起) */}
        {isSidebarVisible && (
          <aside className="fabushi-sidebar-column">
            <FabushiCollectionFilterBar
              activeFolderId={activeFolderId}
              onSelectFolder={setActiveFolderId}
              onOpenNewChat={() => setIsCreateConvModalOpen(true)}
              onOpenSearch={openSidebarSearch}
              unreadCount={unreadFolderCount}
              directCount={humanFolderCount}
              groupCount={groupFolderCount}
              agentCount={agentFolderCount}
              client={client}
            />
            <ConversationSidebar {...sidebarProps} />
          </aside>
        )}

        {/* 三级主工作舞台 */}
        <main className="fabushi-workspace-column">
          {renderActiveDomainWorkspace(activeNavDomain)}
        </main>

        {/* 四级上下文抽屉 (根据 tier 决定内嵌或浮动) */}
        {isDrawerDocked && (
          <aside className="fabushi-context-inspector-docked">
            <ContextInspector {...inspectorProps} />
          </aside>
        )}
      </div>
    </div>
  );
};
```

---

### 6.3 实施切片与工序拆解（Phased Work Slices）

为保证每个实施步骤都可独立编译、独立测试并具备确凿证据，整体改造划分为 4 个清晰工序：

1. **切片 S1：设计变量与 CSS 响应式规则强化（Tokens & Responsive Styles）**
   - 检查 `openclaw-theme.css`，彻底补齐 `.fabushi-shell-grid` 在 `@media (min-width: 1200px)`、`@media (min-width: 900px) and (max-width: 1199px)` 与 `@media (max-width: 899px)` 下的自适应规则；
   - 验证深色模式与浅色模式下的对比度与圆角变量一致性。
2. **切片 S2：左轨与过滤栏组件收口与无障碍增强（Rail & Filter Hardening）**
   - 在 `FabushiPrimaryNavRail.tsx` 中添加全套 ARIA 标签与键盘方向键监听；
   - 在 `FabushiCollectionFilterBar.tsx` 中将 LocalStorage 读写迁移为绑定 `account.id` 的防串扰安全键，并添加在途请求 Generation 检查。
3. **切片 S3：响应式 Hook 抽取与主壳体接线（Hook Extraction & Shell Wire-up）**
   - 编写 `frontend/src/production/layout/useResponsiveLayout.ts`，导出断点与抽屉可见性状态；
   - 在 `ProductionRenderer.tsx` 中替换死锁的 `tasks` 点击逻辑，绑定 `useResponsiveLayout`，消除内联样式写死。
4. **切片 S4：废弃组件标记与回归核验（Deprecation & Regression Check）**
   - 为 `OpenClawNavigationRail.tsx` 与 `TdrpFolderFilterBar.tsx` 添加废弃声明，确认生产引用已被 100% 替换；
   - 执行编译与测试套件，验证无破坏性变更。

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 正反例检查表（Positive vs. Negative Test Oracle）

独立审计子智能体（Teamwork Auditor）与质量保障人员必须依据以下正反例进行逐项穿透核验：

| 审计维度 | 严禁的反例（Negative Case / 视为作弊或严重缺陷） | 必须满足的正例（Positive Case / 生产合格标准） |
|---|---|---|
| **左轨唯一性** | ❌ 在代码中保留 `OpenClawNavigationRail` 的活动挂载，或使用 `isAgentView ? <OpenClawRail> : <FabushiRail>` 条件分叉。 | ✅ 全局唯一定义并挂载 `FabushiPrimaryNavRail`，不论当前选中的是真人、群组还是 Agent 会话，均保持统一样式与几何尺寸。 |
| **过滤栏唯一性** | ❌ 两个过滤器组件各自向 `fabushi_tdrp_*` 和 `fabushi_collection_*` 写入互相不可见的分组数据。 | ✅ 唯一权威使用 `FabushiCollectionFilterBar`，自定义分组统一存取并支持通过 Coordinator RPC 同步至后端数据库。 |
| **Tasks 导航可达性** | ❌ 仅在 `!activeIsHuman && activeAgent != null` 时响应 `tasks` 图标点击，在其他会话时点击静默失效。 | ✅ 任何时刻点击左轨 `tasks` 图标，无条件在主工作区切换呈现全局跨会话任务与自动化中心（`GlobalTasksWorkspace`）。 |
| **响应式视口适配** | ❌ 仅使用行内 `gridTemplateColumns: 52px 300px 1fr`，在窗口缩小到 800px 时页面被挤爆且产生横向滚动条。 | ✅ 窗口缩小到 <900px 时，自动折叠二级侧栏为滑动抽屉，主会话保持全屏可视，输入台与顶栏自适应填充。 |
| **多账号隔离** | ❌ 用户 A 注销并登录用户 B 后，本地依然加载并显示用户 A 创建的自定义会话分组（Cross-account Leak）。 | ✅ 分组存储键严格包含 `account.id` 命名空间，登出或切换账号时单调递增 `accountScopeGeneration` 并清空内存状态。 |
| **设计系统与 Token** | ❌ 直接使用十六进制颜色硬编码（如 `background: #18181c`）或未经别名收口的 `--oc-*` 私有变量。 | ✅ 100% 绑定 `--fabushi-*` 语义 Token，深浅色切换时自然透明适配，WCAG AA 对比度符合 ≥4.5:1。 |
| **无障碍与 IME** | ❌ 中文输入拼音选字时按 Enter 直接触发搜索或新建分组提交；弹窗关闭后焦点丢失到 `body`。 | ✅ 严格校验 `isComposing` 保护中文输入；弹窗关闭后焦点自动归位回触发源（Focus Return）。 |

---

### 7.2 生产代码验收断言门禁（Acceptance Gates G1..G7）

在最终合入主干前，系统必须逐一通过以下 7 大验收门禁（Acceptance Gates）：

- **[GATE-01] 唯一左轨门禁**：
  静态检查生产渲染树中 `FabushiPrimaryNavRail` 的引用计数为 1，`OpenClawNavigationRail` 的渲染引用计数为 0。
- **[GATE-02] 唯一过滤栏门禁**：
  静态检查 `FabushiCollectionFilterBar` 处于生产挂载状态，`TdrpFolderFilterBar` 无任何直接活跃挂载。
- **[GATE-03] 任务中心无条件导航门禁**：
  单元测试断言：在 `activeAgent = null`、`activeIsHuman = true`、`activeIsHuman = false` 三种场景下调用 `onSelectDomain("tasks")`，状态均稳定转移至全局任务视图。
- **[GATE-04] 三阶响应式断点门禁**：
  在 Headless 浏览器中分别设置视口宽度为 1440px、1024px 与 768px，断言 CSS Grid 布局类名与抽屉浮动属性符合 Tier 1、Tier 2、Tier 3 定义，无任何水平溢出滚动。
- **[GATE-05] 跨账号 Fencing 隔离门禁**：
  模拟切换 `account.id`，断言旧账号创建的自定义分组绝不出现在新账号界面，且慢速在途 RPC 响应被 Generation 门控安全拦截丢弃。
- **[GATE-06] 语义设计变量门禁**：
  执行主题样式校验脚本，断言左轨与过滤栏组件中无任何未封装裸色值，深色与浅色模式下主文本对比度均 ≥ 4.5:1。
- **[GATE-07] 全键盘与 IME 保护门禁**：
  模拟输入法处于拼音组合状态（`isComposing = true`），按下 `Enter` 键断言不触发任何提交动作；使用方向键遍历左轨按钮，断言焦点流转无死循环或中断。
