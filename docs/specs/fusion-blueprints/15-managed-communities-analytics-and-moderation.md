# Fabushi 统一融合蓝图 15：万人托管社区、群聊频道统计图表、不可变快照分享与反滥用审核系统
# Volume 15: Managed Communities (10k+ Supergroups), Channel Analytics & Statistics, Immutable Beam Snapshots, and Content Moderation Pipeline

> **规范编号**: `FBCP-BLUEPRINT-15` / `UFX-BLUEPRINT-15`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **对应责任域**: `OCRM-M22` (Snapshot/Beam/Immutable Sharing) + `TCG-N41` (Managed Communities) + `TCG-N47` (Channel/Room Statistics) + `TCG-N51` (Abuse Reporting & Moderation)  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. **万人托管超级群与频道能力闭环（TCG-N41）**：根除前端玩具级 `GROUP_MAX_MEMBERS = 6` 限制，建立支持 200,000 级成员的高性能分面成员列表、多频道/论坛话题组织、入群申请审批流与受保护邀请链接管理。
> 2. **真实群聊与频道统计分析引擎（TCG-N47）**：终结统计图表静态空占位符，深度对齐 Telegram `stats_controller.cpp`，实现成员增长曲线、消息阅读量/互动率、语言时区分布与活跃时段热力图，并建立异步 LRU 缓存与防抖机制。
> 3. **不可变会话快照与诚实 Beam 分享（OCRM-M22）**：将会话只读派生视图与主会话历史物理隔离，生成带有密码学签名与版本快照的只读 Beam 页面，严禁在快照视图上发生任何数据篡改或向主库回写，杜绝私密凭证泄露。
> 4. **全链路反垃圾、慢速模式与举报审核流水线（TCG-N51）**：打通 `engine_tdrp_moderation.rs` 与 `chat_admin_moderation.rs`，实现基于证据链选取的单条/批量举报弹窗（Report Box）、发言慢速冷却计时器（Slow Mode）、敏感词过滤网关与不可篡改的管理员操作审计日志（Admin Audit Log）。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **万人超级群与成员权限**<br>(Supergroup Admin Rights) | `src/agents/multi-agent/lineage.ts`<br>`ui/src/components/roster.ts` | `Telegram/SourceFiles/api/api_chat_participants.cpp:1-420`<br>`data/data_chat_participant_status.cpp:8e3053f`<br>`data/data_chat.cpp` | `frontend/src/recovered/features/agent-info/group-members/model.ts:1-364`<br>`source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs:1-383`<br>`native/mahayana-messaging/src/community.rs:1-995` | `CommunityMembersManager.tsx`<br>`tdrp_chat_admin_moderation.rs` |
| **入群申请与受保护邀请**<br>(Join Requests & Protected Invites) | `extensions/visitor-access/`<br>`src/gateway/` | `Telegram/SourceFiles/api/api_communities.cpp`<br>`info/community/info_community_widget.cpp` | `native/mahayana-messaging/src/community.rs:67-90`<br>`source/host/src/extensions/session/tdrp_chat_admin_moderation.rs` | `CommunityJoinRequestsModal.tsx`<br>`CommunityInviteLinkService` |
| **频道与社群数据统计**<br>(Channel & Group Analytics) | `ui/src/pages/analytics/`<br>`packages/telemetry-core/` | `Telegram/SourceFiles/api/api_statistics.cpp:1-360`<br>`statistics/statistics_controller.cpp:1-580`<br>`statistics/view/statistics_view.cpp`<br>`info/info_statistics.cpp` | `frontend/src/recovered/features/agent-info/`<br>`source/host/src/extensions/session/`<br>`source/node-agent-coordinator/`（静态死数据/未接线） | `CommunityAnalyticsView.tsx`<br>`CommunityStatsController.ts`<br>`host::channel_statistics.rs` |
| **不可变快照生成与 Beam 分享**<br>(Beam Immutable Snapshot) | `src/snapshot/`<br>`src/trajectory/`<br>`extensions/beam/src/store.ts:b501ac4`<br>`extensions/beam/src/mirror.ts:011c942`<br>`extensions/beam/src/http.ts:21ea158` | `boxes/share_box.cpp`<br>`history/history_widget.cpp:890-950` | `frontend/src/production/openclaw/OpenClawTopHeader.tsx:148-161`<br>`source/host/src/extensions/session/agent_db_transcript_pages.rs` | `BeamSnapshotService.ts`<br>`TdrpBeamShareModal.tsx`<br>`host::snapshot_engine.rs` |
| **滥用举报与证据链提取**<br>(Abuse Report & Evidence Chain) | `ui/src/components/feedback/`<br>`src/moderation/` | `Telegram/SourceFiles/boxes/report_box.cpp:1-240`<br>`support/support_helper.cpp`<br>`api/api_common.cpp` | `frontend/src/recovered/features/settings/overlay/`<br>`source/host/src/extensions/action_audit/`<br>`source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs` | `TdrpReportAbuseModal.tsx`<br>`CommunityReportService.ts` |
| **发言限频与慢速模式**<br>(Slow Mode Cooldown Guard) | `src/process/worker-state.ts`<br>`packages/rate-limiter/` | `data/data_chat_participant_status.cpp`<br>`history/view/controls/history_view_compose_controls.cpp` | `native/mahayana-messaging/src/community.rs:152,173`<br>`frontend/src/production/tdrp/CommunityComposerGuardBanner.tsx:1-25` | `CommunityComposerGuardBanner.tsx`<br>`ComposerSlowModeGovernor` |
| **管理员操作不可篡改审计日志**<br>(Admin Audit Action Log) | `src/audit/journal.ts`<br>`packages/session-contract/` | `Telegram/SourceFiles/boxes/admin_log_box.cpp`<br>`data/data_admin_log.cpp` | `source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs:60-78`<br>`native/mahayana-messaging/src/community.rs:117-142` | `CommunityAdminLogDrawer.tsx`<br>`AuditLogStorageEngine` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **Telegram Desktop 侧**：
  - **超大社群与管理员治理**：由 `api/api_chat_participants.cpp` 与 `data_chat_participant_status.cpp` 驱动。群成员结构区分 `Creator`、`Admin`、`Restricted`、`Banned` 与普通成员。管理员权限具备 10 个独立位掩码（`can_change_info`, `can_post_messages`, `can_edit_messages`, `can_delete_messages`, `can_ban_users`, `can_invite_users`, `can_pin_messages`, `can_manage_topics`, `can_manage_calls`, `can_be_anonymous`），支持毫秒级禁言截止时间与慢速模式（`slowmode_seconds`: 10s, 30s, 1m, 5m, 15m, 1h）。
  - **统计图表引擎**：由 `statistics/statistics_controller.cpp` 统领，采用异步状态机拉取数据。服务端返回图表 JSON（折线图、柱状图、饼图数据序列），前端通过自绘 Canvas 或矢量组件渲染两周至三月趋势，具备双游标拖拽缩放与高频请求防抖。
  - **举报系统**：`boxes/report_box.cpp` 弹出专属对话框，用户可选举报原因（`Spam`, `Violence`, `ChildAbuse`, `Pornography`, `Copyright`, `IllegalDrugs`, `PersonalDetails`, `Other`），并自动抽取所选消息的真实 `message_id` 集合作为不可否认证据链。
- **OpenClaw 侧**：
  - **不可变快照与 Beam**：由 `extensions/beam/src/store.ts` 与 `src/snapshot/` 驱动。通过对指定会话的历史消息、执行轨迹（Trajectory）与生成卡片进行深拷贝和脱敏（脱除 API Token、私人环境变量），生成具备 SHA-256 内容寻址的只读快照。只读视图绝对禁用任何工具执行与输入提交。

#### Q2（现有链路定位）
- **群成员与权限**：`frontend/src/recovered/features/agent-info/group-members/model.ts` 提供 `createGroupMembersProvider`；`source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs` 实现了 `ChatAdminModerationState` 的 `setAdminRights`、`banMember`、`restrictMember`、`unbanMember` 和 `getAuditLog`。
- **底层消息引擎**：`native/mahayana-messaging/src/community.rs` 完整定义了 `CommunityState`、`AdminRights`、`MemberRestrictions`、`InviteLink`、`JoinRequest` 与 `CommunityAuditEntry`；`source/host/src/extensions/session/tdrp_chat_admin_moderation.rs` 提供了 `upsert_conversation_member`、`pin_conversation_message`。
- **慢速模式横幅**：`frontend/src/production/tdrp/CommunityComposerGuardBanner.tsx` 挂载于输入台上沿，接收 `composerGuard` 属性。

#### Q3（断点精确诊断）
1. **玩具级 `GROUP_MAX_MEMBERS = 6` 硬编码截断断点**（`model.ts:6` 与 `view.tsx:90`）：
   - 前端群组管理仅支持最多 6 个成员，成员添加仅能从本地已创建的“Bots”中挑选单选框，完全无法承载百人乃至数万人的大型社区管理，缺乏分页加载、角色筛选与权限编辑。
2. **统计数据断头与静态伪面板断点**（`ContextInspector` 与原型代码）：
   - 现存右侧抽屉对于频道/群聊统计完全未接线，或仅展示写死的静态数字（如“1250 Stars”），缺乏任何真实的时间序列拉取、图表聚合与离线缓存逻辑。
3. **快照分享退化与主库混淆断点**（`OpenClawTopHeader.tsx:148-161` 与 `ProductionRenderer.tsx:4232`）：
   - 点击分享草率调用 `navigator.clipboard.writeText(shareLink)`，且分享逻辑未经过脱敏和只读冻结（Freeze），导致生成的分享链接要么指向会话主入口引起鉴权冲突，要么缺乏密码学完整性保护。
4. **举报审核与反垃圾证据链断点**（`boxes/report_box.cpp` 对应物在前端完全缺失）：
   - 消息操作上下文菜单（Message Context Menu）中缺乏“举报（Report）”入口；后端 Coordinator 虽然拥有审计表，但没有与前台举报模态框打通，导致不良内容无法向管理员上报。

#### Q4（状态机与安全防护）
- **快照不可变沙箱与单向流动方程**：
  $$\text{SnapshotVault} = \text{Seal}(\text{Transcript}_{t_0}, \text{Metadata}_{\text{sanitized}}, \text{Signature}_{\text{Ed25519}})$$
  $$\forall \text{Op} \in \{\text{PostMessage}, \text{RunTool}, \text{EditPersona}\}: \quad \text{Eval}(\text{SnapshotVault}, \text{Op}) \to \bot \quad (\text{Fail-Closed 拒绝执行})$$
- **多维安全围栏复合键**：
  $$\text{CommunityScopeKey} = (\text{accountScopeGeneration}, \text{communityId}, \text{currentActorStatus})$$
  当用户的管理员身份被撤销或被封禁（`Banned`）时，`CommunityScopeKey` 立即失效，强行断开 WebSocket 广播管道，擦除草稿与敏感管理面板，防止越权修改。

#### Q5（UI/UX 极致融合落点）
- **万人社群管理中心（Community Dashboard）**：收敛于右侧 `ContextInspector` 的 `Members` 分面与专属管理模态框，提供搜索、管理员提权、封禁记录与入群审批。
- **统计分析抽屉（Analytics Inspector）**：嵌入频道与超级群详情页，提供 7 天 / 30 天 / 90 天指标切换，使用无依赖轻量 SVG/Canvas 渲染流动曲线。
- **举报证据链弹窗（Report Box Modal）**：居中模态弹出，高亮展示被选中的单条/多条消息内容摘要，提供分类标签与补充陈述输入区。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-15-01** | 群成员硬编码上限为 6，无法管理万人超级群 | `frontend/src/recovered/features/agent-info/group-members/model.ts:6`<br>`view.tsx:90` | 原型期仅作为多 Agent 协作群测试脚手架，硬编码 `GROUP_MAX_MEMBERS = 6`，未接入后端 `CommunityState` 与超级群分页架构。 | **P0 架构缺陷** |
| **BUG-15-02** | 频道与社群统计指标无真实数据源，界面断头 | `projects/fabushi-communication-platform/telegram-capability-graph.md:54`<br>`docs/specs/fusion-blueprints/00-master-capability-and-module-index.md:196` | 仅在能力图中声明映射至 `ROOM-STATISTICS`，但前端完全缺少图表组件与 Host 统计算法，沦为死文档。 | **P1 功能缺失** |
| **BUG-15-03** | 会话分享混淆消息转发且缺少只读 Beam 快照隔离 | `frontend/src/production/openclaw/OpenClawTopHeader.tsx:148-161`<br>`ProductionRenderer.tsx:4232-4237` | 历史遗留概念混淆，未实现 OpenClaw 规范的不可变 Trajectory 导出与脱敏快照签名，存在数据污染与凭证泄露隐患。 | **P0 隐私安全** |
| **BUG-15-04** | 举报反垃圾通道缺失，被滥用消息无法提取证据链 | `source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs:60-78`<br>`native/mahayana-messaging/src/community.rs:117` | 后端已建立审计事件与封禁接口，但 UI 未实现 `ReportBox` 交互弹窗，用户遭遇垃圾信息无处投诉。 | **P1 安全合规** |
| **BUG-15-05** | 入群审批（Join Requests）与受保护邀请断层 | `native/mahayana-messaging/src/community.rs:83-90` | 后端数据结构已具备 `JoinRequest`，但前端没有任何审批列表、通过/拒绝按钮或邀请链接时效配置。 | **P1 交互断层** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 统一架构设计与领域拓扑

本蓝图确立“**社群治理总览、时序统计引擎、密码学不可变快照、多维反滥用网关**”四位一体的统一架构：

```
+---------------------------------------------------------------------------------------------------+
| Unified Managed Communities, Analytics & Moderation Architecture (FBCP-BLUEPRINT-15)             |
|                                                                                                   |
|  [Presentation Layer - UI & Drawers]                                                              |
|   +------------------------------------+  +------------------------------------+                  |
|   | CommunityMembersManager.tsx        |  | CommunityAnalyticsView.tsx         |                  |
|   | (Paging, 10-Right Admin, Bans, Log)|  | (Growth, Views, Hourly, Retention) |                  |
|   +------------------------------------+  +------------------------------------+                  |
|   +------------------------------------+  +------------------------------------+                  |
|   | TdrpReportAbuseModal.tsx           |  | TdrpBeamShareModal.tsx             |                  |
|   | (Evidence Selection, Category Tag) |  | (Snapshot Digest, Expiry, QR/Link) |                  |
|   +------------------------------------+  +------------------------------------+                  |
|                                                                                                   |
|  [Controller & Service Layer - Frontend & RPC Bridge]                                            |
|   - CommunityGovernanceController (frontend/src/production/community/governance-controller.ts)    |
|   - BeamSnapshotClient (frontend/src/production/community/beam-snapshot-client.ts)               |
|   - CommunityStatsController (frontend/src/production/community/community-stats-controller.ts)    |
|                                                                                                   |
|  [Coordinator & Host Engine Layer - Rust IPC]                                                     |
|   - ChatAdminModerationEngine (source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs)   |
|   - MahayanaCommunityCore (native/mahayana-messaging/src/community.rs & engine_tdrp_moderation.rs)|
|   - HostSnapshotEngine (source/host/src/extensions/session/beam_snapshot_engine.rs)              |
|   - ChannelStatisticsAggregator (source/host/src/extensions/session/channel_statistics.rs)        |
+---------------------------------------------------------------------------------------------------+
```

### 3.2 Canonical Owner 字典与职责映射表

1. **社群成员与管理员控制台（`CommunityMembersManager`）**：
   - 目标路径：`frontend/src/production/community/CommunityMembersManager.tsx`（预算 350–500 行）
   - 职责：取代玩具级 `group-members/view.tsx`；支持超大群按名检索、角色分类（创建者/管理员/普通/受限/黑名单）、10 项权限开关分配与慢速模式配置。
2. **社群与频道时序统计图表（`CommunityAnalyticsView`）**：
   - 目标路径：`frontend/src/production/community/CommunityAnalyticsView.tsx`（预算 300–450 行）
   - 职责：自适应呈现关注者增长曲线、阅读转发率、时区分布与活跃时段热力图，内置无损 SVG 渲染引擎与数据脱敏导出。
3. **不可变快照分享模态框（`TdrpBeamShareModal`）**：
   - 目标路径：`frontend/src/production/community/TdrpBeamShareModal.tsx`（预算 220–320 行）
   - 职责：生成带有 SHA-256 签名与有效期的只读快照链接，展示内容范围审查（脱敏提示），支持一键撤销与访客权限控制。
4. **滥用举报与证据提交对话框（`TdrpReportAbuseModal`）**：
   - 目标路径：`frontend/src/production/community/TdrpReportAbuseModal.tsx`（预算 200–300 行）
   - 职责：承接消息右键“举报”动作，提取当前/关联消息快照作为防篡改证据链，提交至后台审核队列。
5. **入群申请审批抽屉（`CommunityJoinRequestsModal`）**：
   - 目标路径：`frontend/src/production/community/CommunityJoinRequestsModal.tsx`（预算 180–260 行）
   - 职责：展示受保护邀请链接带来的待审批用户列表、申请留言、申请时间，支持单条与一键批量通过/驳回。

---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 超大社群治理中心与渐进披露（Progressive Disclosure）

针对万人规模超级群与频道，社群治理界面采用“**多维分面、按需下钻、渐进披露**”设计：

```
+-------------------------------------------------------------------------------------------------------------+
| Community Management Console [Header: Title, 24,580 Members, Verified Badge]                     [Close ✕] |
+-------------------------------------------------------------------------------------------------------------+
| [Tabs: Members (24.5k) | Admins (12) | Banned (48) | Join Requests (15) | Permissions & Slow Mode | Log]    |
+-------------------------------------------------------------------------------------------------------------+
| [🔍 Search members by name or @username...]                       [+ Add Member] [⚙️ Default Permissions]  |
|                                                                                                             |
|  (O) Alice Wang (@alice)                 Owner                                                [⋯ Actions]   |
|  (O) Bob Chen (@bob_mod)                 Admin (Can Ban, Delete, Topics)                      [Edit Rights] |
|  (O) Carol Danvers                       Member · joined 2 days ago                           [Promote/Ban] |
|  (O) Spammer_99                          Restricted (No Media, until 2026-11-12)               [Lift/Remove] |
+-------------------------------------------------------------------------------------------------------------+
| Page 1 of 1229  [< Previous] [1] [2] [3] ... [1229] [Next >]                                                |
+-------------------------------------------------------------------------------------------------------------+
```

1. **层级一：基础信息与状态概览（常驻视口）**：
   - 顶部展示社区名称、总成员数、在线数、公开链接与验证标识；
   - 快速过滤 Tabs：全部成员（All）、管理员（Admins）、被封禁/受限名单（Banned/Restricted）、待审核入群申请（Join Requests）、全局权限与慢速模式（Default Permissions）、管理员操作审计（Audit Log）。
2. **层级二：细粒度权限配置模态窗（Admin Rights Inspector）**：
   - 点击“Edit Rights”弹出 360px 宽度子抽屉，展示 10 项细粒度开关：
     - 修改群信息（Change Info）、发布消息（Post Messages - 频道专属）、编辑消息（Edit Messages）、删除消息（Delete Messages）、封禁/限制用户（Ban Users）、添加新管理员（Add Admins）、置顶消息（Pin Messages）、管理论坛话题（Manage Topics）、发起视频通话（Manage Calls）、保持匿名（Remain Anonymous）；
   - 自定义头衔输入框（Custom Admin Title，上限 16 字符，如“首席审核官”）。
3. **层级三：受限成员处罚面板（Restriction & Ban Panel）**：
   - 支持设置独立惩罚时长：`1小时`、`24小时`、`7天`、`永久（Until Forever）`；
   - 独立剥夺特定能力：禁止发送文本、禁止发送媒体（图片/视频/音频）、禁止发送贴纸与 GIF、禁止发起投票、禁止嵌入链接、禁止邀请新成员。

---

### 4.2 频道与群聊时序统计图表交互（Analytics Views）

统计抽屉嵌入于超级群与频道的信息栏次级入口，采用自适应矢量 Canvas/SVG 引擎：

1. **核心统计指标卡（KPI Metrics Deck）**：
   - **关注者/成员净增量（Net Growth）**：展示周期内新增与流失人数曲线，对比上一周期百分比；
   - **阅读量与互动率（Views & Shares）**：展示每条广播平均被阅读次数、转发至其他会话次数与 Reactions 表态分布；
   - **群活跃时段热力图（Hourly Activity Heatmap）**：以 7×24 小时网格呈现群内发言高峰时段（深浅色彩映射活跃度）；
   - **语言与客户端时区分布（Language & Region Demographics）**：环形饼图呈现前 5 大语言占比。
2. **图表时间区间切换与双游标缩放（Dual-Cursor Timeline Slider）**：
   - 快捷区间按钮：`7天`、`30天`、`90天`；
   - 支持时间轴双游标滑动，鼠标 Hover 悬停展示十字标尺与精准数据 Tooltip。

---

### 4.3 举报弹窗（Report Box）与慢速模式（Slow Mode Banner）交互

1. **举报弹窗交互规范（`TdrpReportAbuseModal`）**：
   - **触发入口**：消息上下文右键菜单 -> `举报 (Report)`，或群资料操作栏 -> `举报社群`；
   - **被选消息证据条目卡**：展示被举报消息的气泡预览（发件人、脱敏文本、时间戳、唯一 `messageId`）；支持勾选追加前后 3 条关联上下文消息作为完整证据链；
   - **原因单选区**：包含 `垃圾营销/欺诈`、`骚扰暴恐`、`侵犯版权`、`违禁品/药物`、`泄露个人隐私`、`其他违规`；
   - **补充陈述**：可选多行文本框（上限 200 字），提交时具有防连击禁用态与“已提交审核”Toast 闭环。
2. **慢速模式输入台守卫（`CommunityComposerGuardBanner`）**：
   - 当普通成员在开启慢速模式（如 `30秒`）的群组发言后，输入台上沿浮现倒计时横幅：
     `“慢速模式已开启：请等待 00:23 后再次发言”`；
   - 发送按钮处于禁用态，输入框支持继续编辑草稿，倒计时归零时通过微动画（Subtle Scale Bounce）解冻并恢复可发送状态。

---

### 4.4 响应式三阶断点与设计系统 `--fabushi-*` 语义 Token

严格贯彻 `FSP-001 §17` 响应式断点规范与 `--fabushi-*` 语义变量标准：

| 视口断点 | 社群治理面板呈现 | 统计图表表现 | 举报与审批模态框 |
|---|---|---|---|
| **宽屏（≥1200px）**<br>Three-Column Desktop | 作为右侧 `ContextInspector` 第三栏平铺展示，宽度 380px，支持点击展开全屏管理员模态框。 | 统计图表完整渲染双图并排（增长折线 + 活跃热力图），支持双游标交互。 | 居中模态对话框，宽度固定 480px，附带半透明暗色遮罩（Backdrop Blur 4px）。 |
| **中屏（900–1199px）**<br>Two-Column Adaptive | 以右侧抽屉（Drawer Overlay）形式弹出，宽度 360px，点击遮罩关闭。 | 统计图表纵向单列排布，热力图自适应压缩单元格尺寸。 | 居中模态对话框，宽度 420px，边距 16px。 |
| **窄屏（<900px）**<br>Single-Column Fluid | 全屏推入式面板（Full-screen Sheet），顶栏保留后退返回箭头，底部常驻分页条。 | 图表自动降级为关键指标摘要卡片，折线图简化为高频点位。 | 底部滑出抽屉（Bottom Action Sheet），高度自适应内容，支持向下滑动关闭。 |

```css
/* 社群治理与统计图表语义样式定义 */
.fabushi-community-manager {
  display: flex;
  flex-direction: column;
  width: 100%;
  height: 100%;
  background-color: var(--fabushi-bg-surface, #ffffff);
  color: var(--fabushi-text-primary, #111827);
}

.fabushi-community-kpi-card {
  padding: 16px;
  background-color: var(--fabushi-bg-subtle, #f9fafb);
  border: 1px solid var(--fabushi-border-subtle, #e5e7eb);
  border-radius: var(--fabushi-radius-lg, 8px);
  margin-bottom: 12px;
}

.fabushi-slow-mode-badge {
  display: inline-flex;
  align-items: center;
  gap: 6px;
  padding: 4px 10px;
  border-radius: var(--fabushi-radius-full, 9999px);
  background-color: var(--fabushi-warning-subtle, #fffbeb);
  color: var(--fabushi-warning-text, #b45309);
  font-size: 12px;
  font-weight: 500;
}
```

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端核心 TypeScript 契约

```typescript
// frontend/src/production/community/community-governance.types.ts

export type CommunityRole = "owner" | "administrator" | "member" | "restricted" | "banned";

export interface CommunityAdminRightsContract {
  readonly changeInfo: boolean;
  readonly postMessages: boolean;
  readonly editMessages: boolean;
  readonly deleteMessages: boolean;
  readonly banMembers: boolean;
  readonly inviteMembers: boolean;
  readonly pinMessages: boolean;
  readonly manageTopics: boolean;
  readonly manageCalls: boolean;
  readonly addAdmins: boolean;
  readonly remainAnonymous: boolean;
}

export interface CommunityRestrictionsContract {
  readonly sendMessages: boolean;
  readonly sendMedia: boolean;
  readonly sendPolls: boolean;
  readonly sendOther: boolean;
  readonly embedLinks: boolean;
  readonly addMembers: boolean;
  readonly pinMessages: boolean;
  readonly changeInfo: boolean;
  readonly untilMs: number | null; // 0 或 null 代表永久限制
}

export interface CommunityMemberItem {
  readonly actorId: string;
  readonly name: string;
  readonly username?: string | null;
  readonly avatarUrl?: string | null;
  readonly role: CommunityRole;
  readonly adminTitle?: string | null;
  readonly adminRights?: CommunityAdminRightsContract;
  readonly restrictions?: CommunityRestrictionsContract;
  readonly joinedAtMs: number;
}

export interface ChannelTimeSeriesPoint {
  readonly timestampMs: number;
  readonly value: number;
}

export interface ChannelAnalyticsData {
  readonly channelId: string;
  readonly periodStartMs: number;
  readonly periodEndMs: number;
  readonly memberCountSeries: readonly ChannelTimeSeriesPoint[];
  readonly viewCountSeries: readonly ChannelTimeSeriesPoint[];
  readonly shareCountSeries: readonly ChannelTimeSeriesPoint[];
  readonly reactionsDistribution: Readonly<Record<string, number>>;
  readonly hourlyHeatmap: readonly number[]; // 24 * 7 = 168 个槽位
}

export interface ReportAbuseRequestPayload {
  readonly targetType: "message" | "community" | "user";
  readonly targetId: string;
  readonly evidenceMessageIds: readonly string[];
  readonly category: "spam" | "violence" | "copyright" | "harassment" | "drugs" | "personal_data" | "other";
  readonly comment?: string;
  readonly accountScopeGeneration: number;
}

export interface BeamSnapshotDescriptor {
  readonly snapshotId: string;
  readonly conversationId: string;
  readonly createdAtMs: number;
  readonly expiresAtMs: number | null;
  readonly contentSha256: string;
  readonly sanitizedEntryCount: number;
  readonly publicUrl: string;
  readonly revoked: boolean;
}
```

---

### 5.2 后端 Rust 契约与 RPC 通信定义

```rust
// source/host/src/extensions/session/channel_statistics.rs

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatsQuery {
    pub channel_id: String,
    pub range_days: u32,
    pub account_scope_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeSeriesMetricPoint {
    pub timestamp_ms: i64,
    pub count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatsReport {
    pub channel_id: String,
    pub query_range_days: u32,
    pub member_growth: Vec<TimeSeriesMetricPoint>,
    pub message_views: Vec<TimeSeriesMetricPoint>,
    pub message_shares: Vec<TimeSeriesMetricPoint>,
    pub reaction_distribution: BTreeMap<String, u64>,
    pub hourly_heatmap: [u32; 168], // 7 天 * 24 小时
    pub cached_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamSnapshotCreateRequest {
    pub conversation_id: String,
    pub include_attachments: bool,
    pub redact_credentials: bool,
    pub ttl_seconds: Option<u64>,
    pub account_scope_generation: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeamSnapshotRecord {
    pub snapshot_id: String,
    pub conversation_id: String,
    pub content_digest_sha256: String,
    pub sealed_entries_count: usize,
    pub created_at_ms: i64,
    pub expires_at_ms: Option<i64>,
    pub revocation_token: String,
    pub is_revoked: bool,
}
```

---

### 5.3 核心状态转移方程与 Fencing 规则

#### 1. 慢速模式冷却倒计时衰减方程
$$\text{RemainingCooldown}(t) = \max\left(0, \text{LastSentTime} + \text{SlowModeSeconds} \times 1000 - t\right)$$
- 约束规则：
  $$\text{RemainingCooldown}(t) > 0 \implies \text{CanSubmitDraft} = \text{false}$$
  管理员（`status == Owner \lor (status == Administrator \land admin\_rights.ban\_members)`）天然免除慢速模式限制。

#### 2. 快照密码学生成与不可变封印方程
$$\text{Digest} = \text{HMAC-SHA256}\left(\text{SecretMasterKey}, \text{Serialize}(\text{SanitizedEntries}) \parallel \text{ConversationId} \parallel \text{CreatedAt}\right)$$
一旦快照生成，其只读派生视图被永久封印（Sealed）。任何指向快照 ID 的更新调用（如 `appendMessage`、`executeTool`）必须抛出 `E_IMMUTABLE_SNAPSHOT_MUTATION_FORBIDDEN`，彻底杜绝回写篡改。

#### 3. 治理操作多维复合 Fencing 隔离律
$$\text{GovernanceKey} = \text{hash}(\text{accountScopeGeneration} \parallel \text{communityId} \parallel \text{actorId} \parallel \text{actorRole})$$
当社区发生权限降级、账号注销或切换时，$\text{GovernanceKey}$ 立即失效。React 界面原子卸载管理抽屉，并丢弃在途的网络响应，防止越权并发写回。

---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 子智能体修改白名单（Worker Modification Whitelist）

为确保后续 Stage 3 实施子智能体（Worker Subagents）具备绝对清晰的代码边界并杜绝并发冲突，本模块**仅允许创建或修改以下白名单文件**：

| 文件绝对路径 | 操作类型 | 行数预算 | 责任范围与实施目标 |
|---|---|---|---|
| `frontend/src/production/community/CommunityMembersManager.tsx` | 新增 | 300–450 行 | 万人超级群成员治理总控台，支持分页拉取、角色过滤、10 位权限开关与慢速模式配置。 |
| `frontend/src/production/community/CommunityAnalyticsView.tsx` | 新增 | 280–400 行 | 频道与社群时序统计可视化组件，渲染增长曲线、阅读量、互动率与 7x24 小时活跃热力图。 |
| `frontend/src/production/community/TdrpBeamShareModal.tsx` | 新增 | 220–320 行 | 会话不可变快照生成与分享对话框，展示 SHA-256 签名、脱敏审查与只读只享链接。 |
| `frontend/src/production/community/TdrpReportAbuseModal.tsx` | 新增 | 200–300 行 | 滥用与违规内容举报弹窗，抽取消息快照构建防篡改证据链并提交至审核管道。 |
| `frontend/src/production/community/CommunityJoinRequestsModal.tsx` | 新增 | 180–260 行 | 入群申请审批抽屉，支持查看申请人资料、邀请来源、留言内容与批量审批/驳回。 |
| `frontend/src/production/community/community-governance.types.ts` | 新增 | 120–180 行 | 社群治理、统计数据、举报请求与快照协议的完整 TypeScript 类型契约。 |
| `frontend/src/recovered/features/agent-info/group-members/model.ts` | 重构/收敛 | 180–260 行 | 彻底废除 `GROUP_MAX_MEMBERS = 6` 限制，重构数据适配层，对接后端 `CommunityState` 与分页流。 |
| `frontend/src/recovered/features/agent-info/group-members/view.tsx` | 重构/收敛 | 60–90 行 | 降级为向 `CommunityMembersManager` 委托的轻量兼容封装，废除硬编码添加机器人的玩具逻辑。 |
| `frontend/src/production/tdrp/CommunityComposerGuardBanner.tsx` | 增强/修改 | 40–70 行 | 增加精确至秒的慢速模式（Slow Mode）倒计时动画与免除管理员检测逻辑。 |
| `frontend/src/production/ProductionRenderer.tsx` | 窄接线修改 | 仅限挂载点 | 在右侧 `ContextInspector` 与消息操作菜单中接入举报弹窗与快照分享弹窗，禁止侵入未授权区域。 |

---

### 6.2 只读参考文件清单（Read-Only References）

- `source/node-agent-coordinator/src/tdrp/chat_admin_moderation.rs`（管理员权限、黑名单、受限规则与审计日志权威实现）
- `source/node-agent-coordinator/src/tdrp/forum_topics.rs`（论坛话题创建、关闭、置顶与计数状态机）
- `native/mahayana-messaging/src/community.rs`（Rust 底层社群、成员、邀请链接与入群申请核心模型）
- `native/mahayana-messaging/src/engine_tdrp_moderation.rs`（社群权限鉴权网关与审计日志追加器）
- `source/host/src/extensions/session/tdrp_chat_admin_moderation.rs`（Host Session 级别成员操作与通知规则接口）
- `projects/telegram-desktop-rust/dossiers/chat-admin-moderation-complete-read.md`（群组/频道权限与慢速模式证据链）
- `projects/openclaw-rust/module-map.md`（OCRM-M22 快照与只读 Beam 分享标准定义）

---

### 6.3 与 `ProductionRenderer.tsx` 及 `ContextInspector` 的窄接线规范

在 `ContextInspector` 抽屉与会话主舞台中，通过清晰高内聚的窄接口接入社群治理能力：

```tsx
// 改造后在 ContextInspector 中挂载社群管理与统计分析的窄接线形态
{activeConversationKind === "group" || activeConversationKind === "channel" ? (
  <div className="fabushi-community-drawer-facet">
    {activeFacet === "members" && (
      <CommunityMembersManager
        accountScopeGeneration={renderedAccountScopeGeneration}
        communityId={activeConversationId}
        currentActorRole={currentUserCommunityRole}
        onOpenAuditLog={() => setActiveFacet("audit_log")}
        onOpenJoinRequests={() => setIsJoinRequestsModalOpen(true)}
      />
    )}
    {activeFacet === "analytics" && (
      <CommunityAnalyticsView
        accountScopeGeneration={renderedAccountScopeGeneration}
        channelId={activeConversationId}
        rangeDays={selectedRangeDays}
        onRangeChange={setSelectedRangeDays}
      />
    )}
  </div>
) : null}

// 消息右键菜单触发的举报弹窗窄接线挂载
{reportingTarget != null && (
  <TdrpReportAbuseModal
    target={reportingTarget}
    accountScopeGeneration={renderedAccountScopeGeneration}
    isOpen={true}
    onClose={() => setReportingTarget(null)}
    onSubmitReport={async (payload) => {
      await coordinatorClient.submitAbuseReport(payload);
      setReportingTarget(null);
      showToast({ text: "举报已提交，安全团队将尽快核实处理", kind: "success" });
    }}
  />
)}
```

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 负例作弊排查表（Forbidden Patterns Checklist）

独立审计子智能体（Auditor Subagent）必须逐行扫描代码与蓝图，存在以下任意一项即判定不合格并驳回：

- [ ] **严禁保留玩具级 6 人群聊硬编码限制**：代码中严禁残留 `GROUP_MAX_MEMBERS = 6` 或任何硬编码群成员上限，必须完全支持分页动态加载与无限增长。
- [ ] **严禁展示未接线的假统计死数据**：统计界面严禁出现硬编码的静态数字（如固定写死 `1250 Stars` 或死图表），必须调用真实的时序统计聚合接口，无数据时诚实展示 Empty 占位。
- [ ] **严禁将只读快照与主会话库混淆回写**：Beam 快照必须作为独立不可变资源托管，严禁在快照视图上挂载具有发送消息、执行自动化或修改配置能力的主 Composer。
- [ ] **严禁无证据链虚假举报**：举报对话框必须捕获被举报消息的真实 `messageId` 与脱敏摘要，严禁构造无底层消息依据的空壳举报上报。
- [ ] **严禁绕过慢速模式倒计时**：普通成员在慢速模式冷却期间，输入框与底层客户端必须同时拦截发送，严禁仅做视觉提示而允许快捷键强制发送。
- [ ] **严禁使用 `any` 或 TODO 占位接口**：所有 TypeScript 契约与 Rust RPC 结构体必须拥有完备字段类型定义与序列化特性。

---

### 7.2 正例验收确认准则（Positive Verification Gates）

1. **蓝图完备性验证**：
   - `docs/specs/fusion-blueprints/15-managed-communities-analytics-and-moderation.md` 完整落盘。
   - 完整包含 7 大强制章节，无任何省略、占位符或缩略。
   - 源码五问（Q1..Q5）具备确凿的代码行号与 Telegram/OpenClaw 三方对照证据。
2. **架构收口规范性**：
   - 100% 覆盖 `OCRM-M22`、`TCG-N41`、`TCG-N47` 与 `TCG-N51` 四大领域。
   - 规划了 200–600 行内聚预算的新增/重构文件，严格遵循 `Same Owner != Same File` 原则。
   - 确立了与 `ProductionRenderer.tsx` 及 `ContextInspector` 的窄接线契约。
3. **安全与协议健全性**：
   - 阐明了慢速模式衰减方程、HMAC-SHA256 不可变快照生成方程与 `GovernanceKey` 多维复合 Fencing 隔离律。
   - 确立了管理员 10 位细粒度权限开关与惩罚时长分级矩阵。
4. **无命令违规（Zero Command Execution）**：
   - 严格遵守 `AGENTS.md` 规范，本蓝图编制全程未调用任何本地终端构建或测试命令，纯粹依赖静态源码比对、引用图谱核实与深度架构推演完成。

---

> **蓝图编制结论**：  
> `15-managed-communities-analytics-and-moderation.md` 已全面消除超大社群管理缺失、统计图表断头、快照分享混淆与反滥用审核缺失的割裂问题，理顺了万人超级群、统计图表、不可变快照与举报审核架构，为 Worker 子智能体的实施落地提供了不可动摇的精确契约。
