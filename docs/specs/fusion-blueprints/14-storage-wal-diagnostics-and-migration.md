# Fabushi 统一融合蓝图 14：存储引擎、WAL 崩溃自愈、诊断遥测与迁移升级
# Volume 14: Storage, WAL Recovery, Diagnostics, Migration, and Auto-Update

> **规范编号**: `FBCP-BLUEPRINT-14` / `UFX-BLUEPRINT-14`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游只读锁定基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. **SQLite 存储底座与 WAL 崩溃自愈**: 统一本地会话与配置持久化底座，落地 `store_db.rs` 与 `sqlite_recovery.rs` 的 WAL Checkpoint 截断与隔离自愈流程；严禁前端双写会话真相，保障 ACID 事务完整性。
> 2. **历史记录全量无损导出与格式迁移**: 融合 Telegram Desktop 数据导出引擎（HTML/JSON）与 OpenClaw 导入器（Claude/Hermes/JSON），提供包含多媒体附件、消息因果关系（DAG）的归档与断点续传导入。
> 3. **Doctor 诊断自愈与遥测事件环形缓冲区**: 统一 Host 端 `host_diagnostics.rs`、CLI `doctor.rs` 与前端诊断视图，建立带敏感凭据自动脱敏（Redaction）的事件环形队列（Ring Buffer）与系统体检流。
> 4. **客户端跨平台自动升级与安全空闲重启门禁**: 对齐 `sand-update-service.ts` 与 `safe-relaunch-gate.ts`，支持 macOS (Squirrel) 与 Windows (NSIS) 签名验签、空闲状态检测（Confirmed Idle）与故障回滚标记。
> 5. **CLI / TUI 与 GUI 操作同权对齐**: 确保命令行/TUI 终端与图形客户端调用相同的 Rust 领域服务，消除断头调用链与伪状态展示。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码全量对照矩阵

| 责任域 / 能力项 | Telegram Desktop 锁定源码锚点 | OpenClaw 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **SQLite 存储与 WAL 事务**<br>(OCRM-M29, TCG-N34) | `Telegram/SourceFiles/storage/storage_databases.cpp`<br>`storage/storage_account.cpp` | `src/storage/sqlite.ts`<br>`src/state/`<br>`packages/storage-core/` | `source/host/src/storage/store_db.rs:1-140`<br>`source/host/src/storage/sqlite_recovery.rs:1-213`<br>`source/host/src/extensions/session/agent_db.rs` | `HostStorageEngine`<br>`source/host/src/storage/` |
| **数据库损坏隔离与数据打捞**<br>(OCRM-M29, TCG-N34) | `Telegram/SourceFiles/storage/storage_databases.cpp:180-240`<br>`core/crash_reports.cpp` | `src/storage/recovery.ts`<br>`src/storage/blob-cache.ts` | `source/host/src/storage/sqlite_recovery.rs:95-151`<br>`source/host/src/storage/sqlite_recovery.rs:184-212` | `SqliteCorruptionQuarantine`<br>`source/host/src/storage/sqlite_recovery.rs` |
| **历史记录全量导出与结构序列化**<br>(TCG-N33, Data Export) | `Telegram/SourceFiles/export/export_controller.cpp`<br>`export/view/export_view_panel_controller.cpp` | `src/compat/`<br>`src/storage/export.ts` | `frontend/src/recovered/features/access/cover/`<br>`source/host/src/extensions/transcript/` | `HistoryExportController`<br>`source/host/src/extensions/export/` |
| **第三方会话格式迁移与导入**<br>(OCRM-M29, Migration Runner) | `Telegram/SourceFiles/storage/storage_user_photos.cpp`<br>`data/data_session.cpp` | `extensions/migrate-claude/`<br>`extensions/migrate-hermes/`<br>`src/compat/importer.ts` | `source/mahayana/codex-rs/state/src/migrations.rs`<br>`source/electron-main/startup/startup-data-root-migration.ts` | `MigrationPipelineEngine`<br>`source/host/src/extensions/migration/` |
| **系统体检与 Doctor 诊断套件**<br>(OCRM-M30, Doctor CLI) | `Telegram/SourceFiles/core/crash_reports.cpp`<br>`window/window_main_menu.cpp` | `src/cli/doctor.ts`<br>`src/status/report.ts`<br>`src/logging/diagnostic.ts` | `source/host/src/host_diagnostics.rs:1-81`<br>`source/mahayana/codex-rs/cli/src/doctor.rs:1-120`<br>`source/host/src/extensions/cloud_workers/doctor.rs` | `SystemDoctorEngine`<br>`source/host/src/host_diagnostics.rs`<br>`source/mahayana/codex-rs/cli/src/doctor.rs` |
| **错误脱敏与遥测环形缓冲区**<br>(OCRM-M30, Telemetry Buffer) | `Telegram/SourceFiles/core/sandbox.cpp`<br>`core/crash_reports.cpp` | `src/logging/reporter.ts`<br>`extensions/diagnostics-otel/` | `source/host/src/host_diagnostics.rs:24-63`<br>`source/host/src/error_diagnostics.rs`<br>`source/node-agent-coordinator/src/telemetry/` | `HostDiagnosticsReporter`<br>`source/host/src/host_diagnostics.rs` |
| **自动更新检测与多轨道分发**<br>(OCRM-M31, TCG-N40, Update Feed) | `Telegram/SourceFiles/core/update_checker.cpp:1-260`<br>`boxes/update_box.cpp` | `desktop/package.json`<br>`apps/desktop/src/updater.ts` | `source/electron-main/update/sand-update-service.ts:1-108`<br>`source/electron-main/update/update-feed.ts` | `SandUpdateService`<br>`source/electron-main/update/` |
| **二进制验签与空闲安全重启门禁**<br>(OCRM-M31, TCG-N40, Safe Relaunch) | `Telegram/SourceFiles/platform/win/windows_update.cpp`<br>`platform/mac/mac_update.mm` | `packages/launcher/`<br>`src/infra/relaunch.ts` | `source/electron-main/update/safe-relaunch-gate.ts:1-6`<br>`source/electron-main/update/idle-relaunch-apply.ts`<br>`source/electron-main/update/apply-marker.ts` | `SafeRelaunchGate`<br>`source/electron-main/update/safe-relaunch-gate.ts` |
| **CLI / TUI 运维与配置平滑演进**<br>(OCRM-M28, M01, CLI Parity) | `Telegram/SourceFiles/main.cpp`<br>`core/launcher.cpp` | `src/cli/`<br>`src/tui/`<br>`packages/terminal-core/` | `source/mahayana/codex-rs/cli/src/doctor.rs`<br>`source/mahayana/codex-rs/tui/src/external_agent_config_migration.rs` | `MahayanaCliTuiHost`<br>`source/mahayana/codex-rs/cli/` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **SQLite 存储与损坏打捞（Telegram `storage_databases.cpp` & OpenClaw `sqlite.ts`）**:
  - Telegram Desktop 通过 `Storage::Databases` 严格管理键值与文件哈希索引，在捕获到 `SQLITE_CORRUPT`（错误码 11）时执行隔离并重命名数据文件为 `.corrupt-{timestamp}`，保留原始日志以便分析，随后重新初始化空白 schema。
  - OpenClaw 维护 WAL（Write-Ahead Logging）模式，定期触发 `PRAGMA wal_checkpoint(TRUNCATE)` 压实日志；在连接受损时利用 `open_sqlite_for_salvage` 逐行扫描打捞未损坏记录。
- **数据迁移与导入导出（Telegram `export_controller.cpp` & OpenClaw `importer.ts`）**:
  - Telegram 提供完整 HTML 与可解析 JSON 格式导出，支持选择指定会话、时间跨度、最大文件体积上限（0–4000MB），并包含 CSS 离线渲染模板。
  - OpenClaw 支持从 Claude JSON 导出文件及 Hermes 运行时结构导入历史会话，导入时对消息 UUID 重新分配命名空间，杜绝覆盖现有数据。
- **诊断遥测与自动更新（OpenClaw `doctor.ts` & Telegram `update_checker.cpp`）**:
  - OpenClaw Doctor 检查 Node 运行时、SQLite 锁状态、网络连通性及磁盘剩余空间；遥测上报前对 Bearer Token、私钥及敏感 URL 执行基于正则的脱敏（Redaction）。
  - Telegram 更新器采用 Ed25519 签名验证增量包，支持后台静默下载并在应用关闭或用户显式点击时原子替换二进制文件。

#### Q2（现有链路定位）
- **SQLite 写入与 WAL 维护链路**:
  前端会话更新 -> IPC 桥接 -> Host `source/host/src/storage/store_db.rs`（`bump_db_write_generation`）-> 执行事务操作 -> 定期调用 `checkpoint_sand_agent_db` 截断 WAL。
- **损坏检测与隔离链路**:
  `source/host/src/storage/sqlite_recovery.rs`（`open_sqlite_for_salvage` -> 若失败则触发 `quarantine_corrupt_sqlite_db` -> 移动损坏文件与 `-wal`/`-shm`/`-journal` sidecars 至隔离路径 -> `copy_salvageable_sqlite_rows` 打捞数据至新库）。
- **客户端自动更新链路**:
  `source/electron-main/update/sand-update-service.ts`（`checkForUpdates`）-> 下载安装包至临时目录 -> Windows 触发 `verifySignature` / macOS 委托 Squirrel.Mac -> 触发 `safe-relaunch-gate.ts`（`isSafeToRelaunchForUpdate` 检查屏幕锁定、屏保及系统空闲）-> 原子落地 `apply-marker.ts` 并在退出时替换安装。
- **Doctor 诊断与遥测上报链路**:
  `source/host/src/host_diagnostics.rs`（`report_diagnostic_event` -> `sanitize_diagnostic_fields` 递归脱敏 -> `pin_host_diagnostics_reporter` 分发给已注册消费端）。

#### Q3（断点精确诊断）
1. **SQLite 崩溃打捞未与前端通知总线接线（`wiring=not-wired`）**:
   `source/host/src/storage/sqlite_recovery.rs:120-150` 成功执行了隔离与文件重命名，但隔离结果 `QuarantineCorruptSqliteResult` 仅作为内部返回值，没有向 Electron 渲染层派发 `STORAGE_CORRUPTION_QUARANTINED` 事件，导致前端无法向用户展示“数据库已自愈恢复，打捞了 N 条记录”的提示通知。
2. **多源历史迁移缺乏前端进度投影与事务边界**:
   `source/mahayana/codex-rs/state/src/migrations.rs` 拥有后端迁移逻辑，但在 UI 侧没有统一的“导入与迁移向导（Migration Wizard）”，用户无法选择导入 Claude 或 Hermes 导出的历史会话，底层能力处于孤岛状态。
3. **安全重启门禁（Safe Relaunch Gate）空闲判定缺少 Host 任务执行锁**:
   `source/electron-main/update/safe-relaunch-gate.ts:2-4` 仅检查了 `screenLocked`、`screensaverActive` 与 `systemIdleSeconds`，未检查当前 Host 是否有正在运行的子代理任务（`hasActiveRunningTasks()`），若用户锁屏但后台代理正在执行耗时任务，自动重启会导致任务被强制中断！
4. **诊断日志脱敏规则覆盖面不全**:
   `source/host/src/error_diagnostics.rs` 对常见的 Bearer Token 和 Basic Auth 进行了脱敏，但缺少针对 OpenAI / Anthropic API Key（`sk-ant-*`, `sk-proj-*`）及 Telegram Bot Token（`\d{9,10}:[a-zA-Z0-9_-]{35}`）的特定模式脱敏，存在偶发日志凭据泄露风险。

#### Q4（状态机与安全防护）
- **WAL 检查点与损坏打捞 5 态流转方程**:
  $$\text{StorageState} \in \{\text{Healthy}, \text{Checkpointing}, \text{Degraded}, \text{Quarantining}, \text{Salvaging}\}$$
  - $\text{Healthy} \xrightarrow{\text{periodic\_wal\_tick}} \text{Checkpointing} \xrightarrow{\text{checkpoint\_ok}} \text{Healthy}$
  - $\text{Healthy} \xrightarrow{\text{sqlite\_corrupt\_error}} \text{Degraded} \xrightarrow{\text{initiate\_recovery}} \text{Quarantining}$
  - $\text{Quarantining} \xrightarrow{\text{rename\_sidecars\_ok}} \text{Salvaging}$
  - $\text{Salvaging} \xrightarrow{\text{copy\_salvageable\_rows\_done}} \text{Healthy}$
- **自动更新生命周期 6 态流转方程**:
  $$\text{UpdateState} \in \{\text{Idle}, \text{Checking}, \text{Available}, \text{Downloading}, \text{Staging}, \text{Ready}\}$$
  - $\text{Idle} \xrightarrow{\text{trigger\_check}} \text{Checking}$
  - $\text{Checking} \xrightarrow{\text{offer\_newer\_version}} \text{Available} \xrightarrow{\text{start\_download}} \text{Downloading}$
  - $\text{Downloading} \xrightarrow{\text{verify\_sha256\_ok}} \text{Staging}$
  - $\text{Staging} \xrightarrow{\text{signature\_verified}} \text{Ready}$
  - $\text{Ready} \xrightarrow{\text{safe\_idle\_gate\_pass} \lor \text{explicit\_restart}} \text{Idle (relaunched)}$
- **多账号与会话 Fencing 机制**:
  数据迁移与历史导出严格按 `accountScopeGeneration` 与 `session_key` 隔离；打捞操作必须在持有独占写锁时执行，阻断一切在途写事务。

#### Q5（UI/UX 极致融合落点）
- **存储与数据管理**: 收拢于“设置 -> 数据与存储（Settings -> Data & Storage）”。展示当前数据库大小、WAL 积压状态、一键手动压实（Checkpoint Now）与全量备份导出按钮。
- **系统诊断面板（Doctor）**: 提供全局快捷键（`Cmd/Ctrl+Shift+D`）或通过“设置 -> 高级 -> 运行体检（System Health）”拉起，以指标卡片展示 SQLite 延迟、网络握手、磁盘用量并支持一键导出已脱敏诊断报告。
- **更新就绪状态卡片**: 当版本处于 `Ready` 时，在应用顶栏右侧及设置入口展示优雅的 `--fabushi-status-info` 呼吸徽标，提示“更新已就绪，将在您离开时自动应用”。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

1. **缺陷 1：SQLite 损坏自愈未向前端通知，状态存在黑盒断点**
   - **源码定位**: `source/host/src/storage/sqlite_recovery.rs:108-151` 与 `source/host/src/storage/store_db.rs:109-139`。
   - **现象**: 当底层 SQLite 检测到损坏并触发 `quarantine_corrupt_sqlite_db` 时，虽然成功将受损文件转储为 `.corrupt-{stamp}`，但整个过程没有任何结构化事件暴露给 Coordinator 或 Electron 渲染进程。UI 无法向用户解释为何部分历史记录丢失或已被打捞。
2. **缺陷 2：历史数据导出与多源迁移能力割裂**
   - **源码定位**: `source/electron-main/startup/startup-data-root-migration.ts:1-80` 与 `source/mahayana/codex-rs/state/src/migrations.rs:1-150`。
   - **现象**: 现有的迁移仅限于启动时的数据目录移动（Data Root），缺少像 Telegram 原生那样导出为离线查看 HTML 格式和 JSON 结构体的高级导出器；同时对 Claude/Hermes 历史数据的导入仍停留在 Rust 内部 crate 单元，前端无接入界面。
3. **缺陷 3：更新重启门禁忽视后台长任务执行状态（Task Collision）**
   - **源码定位**: `source/electron-main/update/safe-relaunch-gate.ts:4`。
   - **现象**: 自动重启门禁仅考量操作系统层级的输入空闲，未串联 Host 端的活跃任务状态池。当子代理正在执行长时间的代码编译或远程控制时，若恰逢用户锁屏超过 10 分钟，应用将执行暴力重启，造成任务执行状态丢失和数据损坏。
4. **缺陷 4：诊断事件脱敏在复杂嵌套结构中存在漏网隐患**
   - **源码定位**: `source/host/src/host_diagnostics.rs:45-63`。
   - **现象**: `sanitize_diagnostic_fields` 虽递归处理了 Object 与 Array，但底层 `sanitize_error_diagnostic` 仅过滤了固定格式的 Bearer Token，对于包含在多行日志堆栈（Stack Trace）中的密钥及 URL Query 参数未能全面清除。

### 2.2 根因分析（Root Cause Analysis）

1. **孤岛架构设计**: 存储层（`store_db.rs`）、升级服务（`sand-update-service.ts`）与体检服务（`doctor.rs`）在初期实现时均为独立子系统，各自拥有独立的数据模型，缺乏统一的事件广播总线与 UI 投影合同。
2. **缺乏原子事务协同保护**: 客户端更新与重启逻辑没有与宿主任务调度器的“任务租赁（Task Lease）”机制握手，使得运维层的动作凌驾于业务运行状态之上，破坏了系统运行的终态单调性。

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 架构分层与责任归属

```
┌────────────────────────────────────────────────────────────────────────┐
│               Frontend Presentation: Settings & Diagnostics Drawer     │
│       (StorageMaintenanceCard, DoctorHealthModal, UpdateNotification)  │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ IPC / Typed Bridge
┌───────────────────────────────────▼────────────────────────────────────┐
│              Electron Main: Update & Relaunch Coordinator              │
│       - sand-update-service.ts (Multi-track distribution & feed)       │
│       - safe-relaunch-gate.ts (Task Lease & OS Idle handshake)         │
│       - startup-data-root-migration.ts (Atomic directory switch)       │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │ Coordinator RPC / Stdio
┌───────────────────────────────────▼────────────────────────────────────┐
│                    Host Engine (Rust Canonical Owners)                 │
│  ┌───────────────────────┐ ┌──────────────────────┐ ┌────────────────┐ │
│  │  HostStorageEngine    │ │  SystemDoctorEngine  │ │ MigrationEngine│ │
│  │  - store_db.rs        │ │  - host_diag.rs      │ │ - importer.rs  │ │
│  │  - sqlite_recovery.rs │ │  - error_diag.rs     │ │ - exporter.rs  │ │
│  └───────────────────────┘ └──────────────────────┘ └────────────────┘ │
└────────────────────────────────────────────────────────────────────────┘
```

### 3.2 模块职责表与代码内聚预算

| 模块名称 | 目标文件路径 | 核心职责与设计约束 | 代码行数预算 |
|---|---|---|---|
| **存储底座与 WAL 控制** | `source/host/src/storage/store_db.rs`<br>`source/host/src/storage/sqlite_recovery.rs` | 唯一管理 SQLite 连接池、WAL 截断、错误隔离与逐行打捞 | 200–450 行 |
| **诊断遥测与脱敏引擎** | `source/host/src/host_diagnostics.rs`<br>`source/host/src/error_diagnostics.rs` | 环形缓冲区日志维护、高级正则脱敏、健康指标收集 | 200–350 行 |
| **自动更新与安全门禁** | `source/electron-main/update/sand-update-service.ts`<br>`source/electron-main/update/safe-relaunch-gate.ts` | 跨平台版本拉取、数字签名校验、任务感知型空闲重启 | 300–500 行 |
| **数据导出与迁移核心** | `source/host/src/extensions/export/history_exporter.rs`<br>`source/host/src/extensions/migration/session_importer.rs` | HTML/JSON 结构化历史归档、Claude/Hermes 数据无损清洗迁移 | 250–500 行 |
| **前端存储与诊断组件** | `frontend/src/production/settings/StorageMaintenanceView.tsx`<br>`frontend/src/production/diagnostics/SystemDoctorModal.tsx` | 存储维护仪表盘、体检报告展示、更新控制卡片 | 200–400 行 |

---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 存储与数据维护交互（Storage Maintenance & Data Hub）

- **标准界面位置与渐进披露**:
  - 位于全局“设置 -> 数据与存储（Settings -> Data & Storage）”中的核心卡片。
  - **一级常规视图**: 展示当前数据库实际占用空间（MB/GB）、附件缓存大小、当前 WAL 待回写日志帧数（WAL Log Frames）。提供两个高频动作按钮：“立即优化压实（Optimize Now）”与“备份导出数据（Export Backup）”。
  - **二级展开视图（高级运维）**: 点击“高级存储与修复（Advanced & Repair）”展开折叠区域，展示各会话表的行数分布、最后一次 Checkpoint 时间戳，以及“数据库完整性自检（PRAGMA integrity_check）”触发器。
- **历史记录导出向导（Export Wizard）**:
  - 点击“备份导出数据”弹出标准模态视窗（Modal Dialog），提供分步导引：
    1. **选择导出格式**: 离线可读 HTML 静态站点（含 CSS/JS 资源打包）或机读结构化 JSON（包含完整元数据与父子关联树）。
    2. **选择范围**: 全部会话、最近 30 天，或自定义勾选指定频道/私聊。
    3. **媒体文件上限**: 滑块选择单个附件最大限制（无限制 / 100MB / 10MB / 仅文本）。
    4. **进度反馈**: 导出过程中展示实时进度条（`--fabushi-status-info`），并展示当前正在导出的会话名称与已写入字节数。
- **第三方会话导入向导（Migration Flow）**:
  - 支持拖拽 Claude 导出的 `conversations.json` 或 Hermes 数据包到导入区域。
  - 预检阶段扫描冲突 ID 并展示预览摘要卡片（包含检测到的会话数、消息总数、附件引用数），点击“确认导入”后在后台事务中安全写入，提供取消（Abort）操作并支持安全回滚。

### 4.2 System Doctor 诊断套件与体检模态视窗（System Doctor Modal）

- **调出入口与快捷键**:
  - 全局快捷键 `Cmd+Shift+D`（macOS）或 `Ctrl+Shift+D`（Windows/Linux），亦可在应用帮助菜单或设置页面底部的“系统体检（Run System Doctor）”直接拉起。
- **体检项指标看板**:
  - 采用网格卡片布局展示 5 项核心指标体检结果：
    1. **SQLite 存储健康度**: 检查数据库连接响应延迟、WAL 锁等待状态、Schema 版本一致性。
    2. **网络与 RPC 通信**: 测量 Coordinator 端口连通性、Host 进程握手心跳、外部网关延迟。
    3. **磁盘与内存配额**: 监控工作区可用磁盘剩余空间（低于 500MB 标红警告）、进程 Resident 内存占用。
    4. **任务调度器负载**: 检查正在运行的子代理任务数、挂起队列长度与超时租赁计数。
    5. **日志与错误缓冲区**: 展示最近 50 条已脱敏的警告与错误事件。
- **状态徽标与一键自愈**:
  - 每个指标依据结果呈现 3 种语义状态徽标：
    - `Normal`（绿色 `--fabushi-status-success`）：各项指标处于健康阈值。
    - `Degraded`（黄色 `--fabushi-status-warning`）：存在轻度积压或非致命异常（如 WAL 帧数超过 1000）。
    - `Critical`（红色 `--fabushi-status-danger`）：检测到数据库损坏或磁盘即将耗尽。
  - 若存在可自愈项（如清理临时缓存、强制 WAL Checkpoint、重启假死子代理连接），顶部显式呈现高亮操作条“一键自愈修复（Fix All Recoverable Issues）”。
  - 提供“复制脱敏报告（Copy Sanitized Report）”按钮，数据格式符合 JSON/Markdown，确保所有 Token 与路径均被遮蔽。

### 4.3 客户端升级提示条与空闲重启交互（Update Pipeline & Idle Relaunch）

- **非侵入式就绪通知（Subtle Staging Toast）**:
  - 升级包后台下载与哈希校验（Ed25519 / SHA-256）完全静默执行，绝不弹出阻塞式弹窗打断用户工作流。
  - 当版本升级包就绪并进入 `Ready` 状态时，应用顶栏右侧展示更新就绪徽标；点击展开轻量气泡卡片：
    - 展示新版本号（如 `v1.4.2`）与主要更新摘要日志（Changelog）。
    - 提供操作：“立即重启并安装（Restart Now）”与“稍后在离开时自动更新（Update When Idle）”。
- **空闲重启保护指示（Safe Idle Guard Indicator）**:
  - 在设置中提供开关“空闲时自动应用更新（Auto-update when idle）”。
  - 若用户启用，当系统锁屏且系统空闲超过 10 分钟、且 Host 没有任何正在执行的 Agent 任务时，更新器安全触发重启，杜绝任务半途中断。

### 4.4 响应式断点、设计 Token 与无障碍规范

- **响应式断点设计**:
  - **宽屏（≥1200px）**: 诊断面板以右侧可停靠抽屉（Docked Drawer）或居中模态视窗（宽 880px）展现；存储视图呈现三列网格布局。
  - **中屏（900–1199px）**: 诊断面板居中宽度收敛为 680px；存储视图自适应折叠为两列，指标图表精简展示。
  - **窄屏（<900px）**: 诊断面板全屏遮罩展开，底部停靠固定操作栏；存储卡片采用垂直单列流式排列。
- **设计语义变量（Design Tokens）**:
  - 严格采用 `--fabushi-*` 语义 Token，严禁任何硬编码十六进制颜色：
    - 容器背景：`var(--fabushi-surface-canvas)` 与 `var(--fabushi-surface-panel)`。
    - 边框与分割线：`var(--fabushi-border-subtle)`。
    - 状态色调：`var(--fabushi-status-success)`、`var(--fabushi-status-warning)`、`var(--fabushi-status-danger)`、`var(--fabushi-status-info)`。
    - 交互按钮：`var(--fabushi-interactive-primary)` 与 `var(--fabushi-interactive-hover)`。
- **无障碍（a11y）与焦点管理**:
  - 模态视窗打开时自动捕获焦点至首个可交互按钮（Focus Trap），按 `Esc` 安全退出并返还焦点至触发元素（Focus Return）。
  - 进度指示器配置完整的 `role="progressbar"`，并带有 `aria-valuenow`、`aria-valuemin="0"`、`aria-valuemax="100"` 与 `aria-label="数据库维护进度"`。
  - 键盘 Tab 键顺序符合自然阅读顺序，屏幕阅读器可完整朗诵诊断报告各卡片的判定状态与错误说明。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 TypeScript 接口契约定义

```typescript
/** 存储底座健康与维护状态快照 */
export interface StorageMaintenanceStatus {
  readonly databaseSizeBytes: number;
  readonly walSizeBytes: number;
  readonly walFramesUncheckpointed: number;
  readonly lastCheckpointTimestampMs: number | null;
  readonly totalConversationCount: number;
  readonly totalMessageCount: number;
  readonly isCompacting: boolean;
  readonly corruptionStatus: {
    readonly isDegraded: boolean;
    readonly lastQuarantinedPath: string | null;
    readonly salvagedRowsCount: number | null;
  };
}

/** 历史数据导出配置请求 */
export interface HistoryExportRequest {
  readonly format: "html" | "json";
  readonly scope: "all" | "selected-conversations" | "time-bounded";
  readonly conversationIds?: readonly string[];
  readonly fromTimestampMs?: number;
  readonly toTimestampMs?: number;
  readonly maxMediaSizeLimitBytes: number; // 0 表示不限制
  readonly outputDirectoryPath: string;
}

/** 数据导出/导入实时进度流事件 */
export interface DataLifecycleProgressEvent {
  readonly phase: "scanning" | "dumping" | "media-copying" | "compressing" | "completed" | "failed";
  readonly processedItems: number;
  readonly totalItems: number;
  readonly processedBytes: number;
  readonly totalBytesEstimate: number;
  readonly currentSubjectName: string;
  readonly errorCode?: string;
  readonly errorMessage?: string;
}

/** System Doctor 体检报告契约 */
export interface DoctorHealthCheckReport {
  readonly overallStatus: "healthy" | "warning" | "critical";
  readonly checkedAtMs: number;
  readonly items: readonly DoctorCheckItem[];
  readonly autoRemediableCount: number;
}

export interface DoctorCheckItem {
  readonly id: string;
  readonly category: "storage" | "network" | "system" | "task-queue" | "telemetry";
  readonly name: string;
  readonly status: "pass" | "warn" | "fail";
  readonly message: string;
  readonly latencyMs?: number;
  readonly details?: Record<string, unknown>;
  readonly remediable: boolean;
}

/** 更新门禁判定参数输入与输出 */
export interface SafeRelaunchCheckPayload {
  readonly optInEnabled: boolean;
  readonly gateEnabled: boolean;
  readonly updateStaged: boolean;
  readonly screenLocked: boolean;
  readonly screensaverActive: boolean;
  readonly systemIdleSeconds: number;
  readonly idleThresholdSeconds: number;
  readonly activeRunningTaskCount: number; // 必须为 0 方可通过
}
```

### 5.2 Rust 核心数据结构与 RPC Payload 定义

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// 数据库损坏隔离与数据打捞综合报告
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuarantineCorruptSqliteReport {
    pub original_db_path: PathBuf,
    pub quarantine_path: Option<PathBuf>,
    pub copied: bool,
    pub rename_error_code: Option<String>,
    pub salvaged_messages_count: usize,
    pub salvaged_sessions_count: usize,
    pub timestamp_ms: u64,
}

/// 数据库维护控制命令
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "action", content = "params")]
pub enum StorageMaintenanceAction {
    TriggerCheckpoint { truncate: bool, busy_timeout_ms: u64 },
    RunIntegrityCheck,
    QuarantineAndSalvage { backup_sidecars: bool },
    ClearAttachmentCache { older_than_days: u32 },
}

/// 脱敏后的诊断事件结构体
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HostDiagnosticEvent {
    pub kind: String,
    pub level: DiagnosticLevel,
    pub timestamp_ms: u64,
    pub trace_id: Option<String>,
    pub fields: serde_json::Map<String, serde_json::Value>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DiagnosticLevel {
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

/// 历史导入数据包格式声明
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "source_format")]
pub enum MigrationSourcePacket {
    ClaudeExport { file_path: PathBuf, namespace_prefix: String },
    HermesRuntime { base_dir: PathBuf },
    FabushiJsonDump { manifest_path: PathBuf },
}
```

### 5.3 严格数学状态转移方程与 Fencing 规则

#### 1. SQLite 存储崩溃自愈状态机（Storage Healing FSM）
定义状态集 $S_{\text{storage}} = \{\text{Healthy}, \text{Checkpointing}, \text{Corrupted}, \text{Quarantining}, \text{Salvaging}, \text{Recovered}, \text{TerminalFault}\}$：

$$\begin{aligned}
(\text{Healthy}, \text{tick\_wal}) &\longrightarrow \text{Checkpointing} \\
(\text{Checkpointing}, \text{ok}) &\longrightarrow \text{Healthy} \\
(\text{Checkpointing}, \text{busy}) &\longrightarrow \text{Healthy} \quad (\text{保持 WAL 待下次处理}) \\
(\text{Healthy} \lor \text{Checkpointing}, \text{SQLITE\_CORRUPT}) &\longrightarrow \text{Corrupted} \\
(\text{Corrupted}, \text{lock\_exclusive}) &\longrightarrow \text{Quarantining} \\
(\text{Quarantining}, \text{rename\_ok}) &\longrightarrow \text{Salvaging} \\
(\text{Quarantining}, \text{rename\_fail}) &\longrightarrow \text{Quarantining} \quad (\text{回退至带重试拷贝模式}) \\
(\text{Salvaging}, \text{rows\_salvaged}) &\longrightarrow \text{Recovered} \\
(\text{Recovered}, \text{init\_new\_schema}) &\longrightarrow \text{Healthy} \\
(\text{Any}, \text{unrecoverable\_io\_error}) &\longrightarrow \text{TerminalFault}
\end{aligned}$$

#### 2. 安全空闲升级重启门禁判定方程（Safe Idle Relaunch Gate Equation）
令重启许可判定函数为 $\Phi(\text{State}) \in \{0, 1\}$，由 6 个布尔判据构成合取公式：

$$\Phi = \text{optIn} \land \text{gateEnabled} \land \text{isStaged} \land (\text{screenLocked} \lor \text{screensaverActive}) \land (\tau_{\text{idle}} \ge \tau_{\text{threshold}}) \land (N_{\text{active\_tasks}} = 0)$$

- **Fencing 规则 1（任务排他性锁）**: 当 $N_{\text{active\_tasks}} > 0$ 时，$\Phi$ 恒为 $0$，严禁发起任何进程退出与安装启动（`quitAndInstall` / `installOnQuit`）。
- **Fencing 规则 2（多账号作用域代际隔离）**: 数据导出与历史迁移任务严格绑定当前的 `accountScopeGeneration`。若用户在导出或迁移在途期间切换了当前账号，正在执行的 IO 管道必须通过 `AbortSignal` 立即中止，已生成的未完成碎片必须原子清理，严禁将 A 账号的数据写入 B 账号的数据库中。

---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 阶段施工切片划分（Stage 3 Implementation Slices）

为确保后续多智能体（Workers）协同施工时互不干扰，本蓝图拆解为 5 个高内聚、单向依赖的独立施工切片：

- **切片 A（Storage Core & Recovery Slice）**:
  - **核心目标**: 强化 `store_db.rs` 与 `sqlite_recovery.rs`，实现 WAL 定期截断、损坏检测、`.corrupt-{stamp}` 隔离打捞，并向上抛出 `QuarantineCorruptSqliteReport`。
  - **交付边界**: 仅限 Rust Host 存储层，禁止修改 Electron 主进程与前端展示代码。
- **切片 B（Diagnostics & Sanitization Slice）**:
  - **核心目标**: 完善 `host_diagnostics.rs` 与 `error_diagnostics.rs`，实现 100 条容量的有界环形事件队列（Bounded Ring Buffer），补全针对各类 API Key（OpenAI、Anthropic、Telegram Bot）的高级脱敏引擎。
  - **交付边界**: Host 诊断模块与 CLI `doctor.rs`，输出标准 `HostDiagnosticEvent`。
- **切片 C（Update Pipeline & Safe Relaunch Slice）**:
  - **核心目标**: 在 `sand-update-service.ts` 与 `safe-relaunch-gate.ts` 中接入 Host 活跃任务计数，实现任务感知型（Task-Aware）安全重启阻断门禁，固化更新标记。
  - **交付边界**: Electron Main 进程 update 目录。
- **切片 D（Export & Migration Pipeline Slice）**:
  - **核心目标**: 落地 `history_exporter.rs`（HTML 离线包与 JSON 序列化）与 `session_importer.rs`（Claude/Hermes/JSON 导入解析与 UUID 重新映射），绑定事务与 AbortSignal。
  - **交付边界**: Host 端 `extensions/export/` 与 `extensions/migration/`。
- **切片 E（UI Presentation & Doctor Modal Slice）**:
  - **核心目标**: 实现前端“设置 -> 数据与存储”维护视图与 `SystemDoctorModal.tsx` 体检弹窗，收束 `--fabushi-*` 语义 Token 与无障碍属性。
  - **交付边界**: 前端生产组件目录。

### 6.2 允许修改的文件白名单（Permitted Write Whitelist）

```text
source/host/src/storage/store_db.rs
source/host/src/storage/sqlite_recovery.rs
source/host/src/storage/mod.rs
source/host/src/host_diagnostics.rs
source/host/src/error_diagnostics.rs
source/host/src/extensions/export/history_exporter.rs
source/host/src/extensions/export/mod.rs
source/host/src/extensions/migration/session_importer.rs
source/host/src/extensions/migration/mod.rs
source/electron-main/update/sand-update-service.ts
source/electron-main/update/safe-relaunch-gate.ts
source/mahayana/codex-rs/cli/src/doctor.rs
frontend/src/production/settings/StorageMaintenanceView.tsx
frontend/src/production/diagnostics/SystemDoctorModal.tsx
```

### 6.3 只读参考文件清单（Read-Only Reference Baselines）

- **上游 Telegram Desktop 只读基准**:
  - `Telegram/SourceFiles/storage/storage_databases.cpp`（损坏检测与隔离）
  - `Telegram/SourceFiles/export/export_controller.cpp`（HTML/JSON 全量导出）
  - `Telegram/SourceFiles/core/update_checker.cpp`（更新包验签与状态机）
  - `Telegram/SourceFiles/core/crash_reports.cpp`（崩溃捕获与上报沙箱）
- **上游 OpenClaw 只读基准**:
  - `src/storage/sqlite.ts` 与 `src/storage/recovery.ts`（WAL Checkpoint 与打捞）
  - `src/logging/diagnostic.ts` 与 `src/status/report.ts`（诊断事件结构）
  - `src/cli/doctor.ts`（健康体检 CLI）
  - `extensions/migrate-claude/` 与 `extensions/migrate-hermes/`（第三方迁移规范）
- **Fabushi 既有协议与资产**:
  - `projects/openclaw-rust/dossiers/transcript-round100-openclaw-storage-migration-and-diagnostics-observability-native-fusion-20261026.md`
  - `source/electron-main/update/apply-marker.ts`
  - `source/electron-main/update/update-feed.ts`

### 6.4 与现有生产壳的窄接线契约（Narrow Wiring Contracts）

- **单一装配入口规范**:
  - 严禁在 `ProductionRenderer.tsx` 中插入长达数百行的存储或诊断组件定义。
  - 前端仅通过单一行窄接线（Narrow Wiring）引入抽屉与全局模态：
    ```tsx
    // In ProductionRenderer.tsx or SettingsDialog.tsx
    <StorageMaintenanceView 
      status={storageStatus} 
      onOptimize={() => desktopBridge.storage.compact()} 
      onExport={() => setExportWizardOpen(true)} 
    />
    <SystemDoctorModal 
      isOpen={isDoctorOpen} 
      onClose={() => setDoctorOpen(false)} 
    />
    ```
- **IPC 桥接窄接线规范**:
  - 所有存储与体检指令通过既有 `desktopBridge` 或 Coordinator RPC 通道传输，严禁前端直接调用 Node.js 原生 `fs` 或执行 Shell 命令。

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 正反例判定检查表（Positive & Negative Oracles）

| 审查维度 | 违规作弊反例（FAIL / REJECT） | 合规真实正例（PASS / ACCEPT） |
|---|---|---|
| **SQLite 损坏自愈** | 1. 捕获到 SQLite 异常时直接 `catch` 吞没错误，返回假空结果。<br>2. 仅在内存中新建临时库，未执行 `.corrupt-{stamp}` 文件归档与 sidecars 转移。<br>3. 前端展示“数据库已恢复正常”，实际既有历史数据被无差别抹除。 | 1. 严格将受损文件及 `-wal`、`-shm` 重命名至隔离目录，失败时安全复制。<br>2. 调用 `open_sqlite_for_salvage` 逐行扫描打捞有效记录并插入新库。<br>3. 产生结构化 `QuarantineCorruptSqliteReport` 并通知上层。 |
| **自动更新安全门禁** | 1. 忽略 Host 活跃任务，仅通过 `setTimeout` 或系统锁屏即发起强制重启。<br>2. 前端进度条使用假 `Math.random()` 模拟下载百分比。<br>3. 绕过数字签名验证，直接执行下载的二进制可执行文件。 | 1. 合取判定公式严格检查 $N_{\text{active\_tasks}} == 0$ 方发放重启许可。<br>2. 真实根据网络流 `receivedBytes / totalBytes` 计算进度。<br>3. Windows 下经 Authenticode 验签，macOS 经 Squirrel 校验，落盘 `apply-marker`。 |
| **诊断日志安全脱敏** | 1. 仅对固定测试字符串（如 `Bearer secret-token`）做硬编码替换。<br>2. 日志中泄露完整的 OpenAI / Anthropic 密钥（如 `sk-ant-api03-...`）。<br>3. 错误堆栈直接原样写入磁盘未做参数遮蔽。 | 1. 采用通用正则递归深度遍历 Map、Array、String，消除所有已知 Token 模式。<br>2. 敏感 URL 中的 Query 参数（如 `?token=...`）被统一替换为 `[REDACTED]`。<br>3. 单元测试覆盖多层嵌套结构与恶意畸变凭据注入。 |
| **历史数据迁移与导出** | 1. 导出的 HTML 仅是空白骨架，缺失 CSS 样式与消息结构体。<br>2. 导入 Claude 会话时直接沿用原始 UUID，发生碰撞时直接覆盖本地已有会话。<br>3. 导入过程中断导致数据库处于损坏的半插入状态。 | 1. HTML 导出为自包含完整静态应用，离线可检索消息与媒体预览。<br>2. 导入时对外部 ID 添加统一命名空间前缀（`claude:conv:...`）并映射新 UUID。<br>3. 全程包裹在 SQLite 事务中，遭遇错误立即触发原子回滚（Rollback）。 |
| **跨账号隔离与 Fencing** | 1. 在账号 A 正在导出时切换至账号 B，数据管道继续向磁盘写入且混入 B 账号数据。<br>2. 缓存的诊断事件跨账号共享，导致 B 账号可窥视 A 账号的请求日志。 | 1. 每次账号切换递增 `accountScopeGeneration`，在途导出立即被 `AbortSignal` 强阻断。<br>2. 诊断缓冲区与临时目录严格基于 `accountScope` 进行物理路径与内存实例隔离。 |

### 7.2 独立自动化验收测试用例清单（Automated Verification Oracles）

1. **`ORACLE-WAL-001`（WAL Checkpoint 截断与帧收敛测试）**:
   - 验证在向 SQLite 写入 500 条消息后，调用 `checkpoint_sand_agent_db`，验证返回值 `folded == true`，且 `-wal` 侧边栏文件体积归零或被截断。
2. **`ORACLE-CORRUPT-002`（数据库坏块隔离与有效记录打捞测试）**:
   - 人为写入损坏字节至 SQLite 文件头，调用 `open_sqlite_for_salvage`，断言其正确触发隔离重命名，并在隔离路径生成带有毫秒时间戳的 `.corrupt-*` 文件，且未损坏的数据表记录被 100% 成功提取转移。
3. **`ORACLE-UPDATE-003`（任务在途阻断空闲重启门禁测试）**:
   - 模拟操作系统锁屏且空闲 15 分钟，但向系统注入 $N_{\text{active\_tasks}} = 1$；断言 `isSafeToRelaunchForUpdate` 严格返回 `false`；仅在任务结束且 $N_{\text{active\_tasks}} = 0$ 时返回 `true`。
4. **`ORACLE-DIAG-004`（多层嵌套高敏感凭据深度脱敏测试）**:
   - 构造深度嵌套 5 层的 JSON 对象，内嵌 Anthropic Key、Telegram Bot Token 及包含私钥的 URL；输入 `sanitize_diagnostic_fields`，断言输出中绝对不包含任何明文敏感子串，且保留原有数据结构。
5. **`ORACLE-MIGRATE-005`（第三方迁移数据幂等与事务回滚测试）**:
   - 导入包含相同会话 ID 的 Claude 导出包，断言第二次导入产生冲突告警且不产生重复脏数据；在写入第 50 条消息时人为注入 IO 错误，断言底层数据库回滚至导入前状态，0 残留碎片。
