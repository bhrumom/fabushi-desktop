# Fabushi 统一融合蓝图 08：远程设备控制、多账号安全隔离与 Rust 后端闭环
# Volume 08: Devices, Computer, and Backend Parity

> **规范编号**: `FBCP-BLUEPRINT-08` / `UFX-BLUEPRINT-08`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. 远程设备控制与节点调度闭环（Computer Use & Remote Nodes）：将 `frontend/src/recovered/features/computer/`、`LocalComputerWorkbench.tsx` 与 Host 端 `source/host/src/runner/computer_use.rs`、`remote_computer_target.rs`、`network_enrollment_owner.rs` 权威串接，消除断头链路。
> 2. 多账号与多设备安全隔离（Account Scope Generation Fencing）：严格落地 `accountScopeGeneration` 单调递增隔离，打通 `source/electron-main/account/account-authorization.ts` 与 Coordinator `settings_cloud_passkeys.rs`，消除跨账号迟到事件与会话数据串染。
> 3. 端到端加密与端侧秘钥闭环（E2E Security & Key Closure）：采用 `native/mahayana-messaging/src/secret_chat.rs`（X25519 + HKDF-SHA256 + XChaCha20Poly1305）替换过时私有 MTProto 协议，配合云端密码（CloudPasswordConfig）与本地安全锁。
> 4. Rust 宿主与前端契约 100% 对齐（Backend Parity & Zero-Any）：统领 OpenClaw 底层 12 个责任域（OCRM-M01, M02, M03, M04, M10, M12, M13, M28, M29, M30, M31, M32）与 Telegram 8 个关键安全/同步能力域（TCG-N02, N25, N32, N34, N37, N38, N40, N52），严禁 `any` 逃逸与假离线/假在线状态。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码对照矩阵

| 功能项与能力域 | OpenClaw 锁定源码锚点 | Telegram Desktop 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **守护进程与生命周期**<br>(OCRM-M01 / TCG-N01) | `src/entry.ts`<br>`src/daemon/`<br>`src/infra/` | `Telegram/SourceFiles/main.cpp`<br>`window/window_main_menu.cpp` | `source/electron-main/entry.ts`<br>`source/electron-main/startup/`<br>`source/node-agent-coordinator/src/main.rs`<br>`source/host/app/src/main.rs` | `source/electron-main/startup/`<br>`source/node-agent-coordinator/` |
| **多账号隔离与作用域**<br>(TCG-N02 / OCRM-M03) | `src/secrets/`<br>`src/pairing/`<br>`extensions/device-pair/` | `main/main_account.cpp`<br>`api/api_authorizations.cpp`<br>`storage/storage_account.cpp` | `source/electron-main/account/account-authorization.ts:1-21`<br>`source/electron-main/account/human-identity.ts`<br>`source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs:102-140` | `account-authorization.ts`<br>`settings_cloud_passkeys.rs` |
| **设备配对与入网**<br>(OCRM-M03 / TCG-N38) | `src/pairing/pair-flow.ts`<br>`extensions/device-pair/` | `Telegram/SourceFiles/api/api_authorizations.cpp`<br>`data/data_session.cpp` | `source/host/src/network_enrollment_owner.rs:38-60`<br>`source/host/src/remote_computer_target.rs:25-35`<br>`native/mahayana-messaging/src/device.rs:18-80` | `network_enrollment_owner.rs`<br>`native/mahayana-messaging/src/device.rs` |
| **CUA 远程桌面与 VNC**<br>(OCRM-M13 / Computer) | `src/node-host/`<br>`crates/openclaw-node-host/`<br>`extensions/cua-computer/` | `Telegram/SourceFiles/calls/calls_controller_webrtc.cpp` | `frontend/src/recovered/features/computer/shell/view.tsx:1-80`<br>`frontend/src/recovered/features/computer/shell/model.ts:1-70`<br>`frontend/src/production/LocalComputerWorkbench.tsx:1-60`<br>`source/node-agent-coordinator/src/gateway/box_vnc_proxy.rs` | `ComputerWorkspace`<br>`box_vnc_proxy.rs`<br>`LocalComputerWorkbench.tsx` |
| **Computer 控制租赁与协调**<br>(OCRM-M10 / CUA Lease) | `src/security/approvals.ts`<br>`packages/worker-runtime/` | `window/window_lock_widgets.cpp`<br>`core/sandbox.cpp` | `source/host/src/runner/computer_use.rs:37-105`<br>`source/host/src/remote_computer_target.rs:47-48`<br>`source/host/src/runner/sand_computer_auto_review.rs:20-45` | `source/host/src/runner/computer_use.rs`<br>`sand_computer_auto_review.rs` |
| **无头浏览器与 CDP 控制**<br>(OCRM-M12 / Browser) | `extensions/browser/`<br>`src/web-search/`<br>`src/link-understanding/` | `Telegram/SourceFiles/core/sandbox.cpp` | `source/host/src/runner/tools/sand_browser_tools.rs`<br>`source/host/src/runner/tools/sand_browser_driver_source.rs`<br>`source/host/src/runner/host_web_dependencies.rs` | `source/host/src/runner/tools/sand_browser_tools.rs` |
| **网关协议与 RPC 分发**<br>(OCRM-M04 / Gateway) | `src/gateway/`<br>`packages/gateway-protocol/`<br>`crates/openclaw-gateway-client/` | `Telegram/SourceFiles/apiwrap.cpp`<br>`core/net/` | `source/host/src/gateway_protocol.rs`<br>`source/host/src/gateway_server.rs`<br>`source/node-agent-coordinator/src/renderer_port_server.rs` | `source/host/src/gateway_protocol.rs`<br>`renderer_port_server.rs` |
| **端到端加密与端侧双棘轮**<br>(TCG-N52 / E2E Security) | `src/secrets/vault.ts` | `Telegram/SourceFiles/tde2e/`<br>`storage/storage_encryption.cpp` | `native/mahayana-messaging/src/secret_chat.rs:1-60`<br>`source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs:65-90` | `native/mahayana-messaging/src/secret_chat.rs`<br>`settings_cloud_passkeys.rs` |
| **SQLite 存储恢复与 WAL**<br>(OCRM-M29 / TCG-N34) | `src/storage/sqlite.ts`<br>`src/state/` | `storage/storage_databases.cpp`<br>`storage/storage_account.cpp` | `source/host/src/storage/store_db.rs`<br>`source/host/src/storage/sqlite_recovery.rs`<br>`source/host/src/extensions/session/agent_db.rs` | `source/host/src/storage/store_db.rs`<br>`agent_db.rs` |
| **审计、诊断与遥测自愈**<br>(OCRM-M30 / Doctor) | `src/logging/`<br>`src/status/`<br>`extensions/diagnostics-otel/` | `core/crash_reports.cpp`<br>`Telegram/SourceFiles/api/api_updates.cpp` | `source/host/src/host_diagnostics.rs`<br>`source/host/src/extensions/action_audit/`<br>`source/node-agent-coordinator/src/telemetry/` | `source/host/src/host_diagnostics.rs`<br>`action_audit/` |
| **自动升级与安全验签**<br>(OCRM-M31 / TCG-N40) | `desktop/package.json`<br>`apps/` | `core/update_checker.cpp`<br>`platform/win/windows_update.cpp`<br>`platform/mac/mac_update.mm` | `source/electron-main/update/sand-update-service.ts`<br>`source/electron-main/update/update-wiring.ts`<br>`source/electron-main/update/safe-relaunch-gate.ts` | `source/electron-main/update/` |
| **跨语言契约镜像规范**<br>(OCRM-M32 / Contracts) | `src/shared/`<br>`packages/normalization-core/` | `data/data_session.cpp` | `contracts/automation/cross-platform-journeys.json`<br>`source/shared/`<br>`frontend/src/recovered/features/computer/shell/model.ts` | `contracts/`<br>`source/shared/` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **OpenClaw 侧**：
  - CUA 设备与 Computer Use 执行由 `extensions/cua-computer/` 与 `src/node-host/` 驱动，声明 40 种 V2 标准动作（`screenshot`、`left_click`、`mouse_move`、`scroll`、`type`、`key`、`launch_app` 等），通过单飞锁 `COMPUTER_EXECUTION_ID_PATTERN` 与 `COMPUTER_HOST_BUSY` 保证同一时刻仅有一个 Provider 获得独占控制权。
  - 节点入网（Enrollment）与配对通过 `src/pairing/` 与 `extensions/device-pair/` 生成设备密钥对，并在网关握手时完成签名认证与角色作用域（Roles & Scopes）验证。
- **Telegram Desktop 侧**：
  - 多账号管理由 `main/main_account.cpp`、`core/core_settings.cpp` 与 `storage/storage_account.cpp` 实现，最多支持 3–6 个账号并发登录，各账号的数据目录、数据库、密钥缓存完全独立，切换时向 UI 广播 Session 变更。
  - 会话设备由 `data/data_session.cpp` 与 `api/api_authorizations.cpp` 管理，提供活跃设备列表（Active Sessions）、端侧会话验签与一键强制注销。
  - 端到端加密由 `storage/storage_encryption.cpp` 与本地应用密码锁（Local Passcode / `window_lock_widgets.cpp`）保护。

#### Q2（现有链路定位）
- **Computer Use 执行链路**：
  前端 `LocalComputerWorkbench.tsx` 或 `computer/shell/view.tsx` -> IPC 通信 -> `source/node-agent-coordinator/src/gateway/box_vnc_proxy.rs` -> Host `source/host/src/runner/computer_use.rs`（`ComputerUseCoordination::acquire_control_lease`）-> `remote_computer_target.rs` -> 目标机器端口 8792 上的 `NetworkWorkerProcess`。
- **多账号授权与切换链路**：
  前端账号选择器 -> `source/electron-main/account/account-authorization.ts`（`createDesktopAccountAuthorizer`）-> 计算 `accountCacheScope(slot)` -> 调用 `scopeToAccount` 并淘汰旧缓存镜像 -> 广播给 Coordinator `settings_cloud_passkeys.rs` 更新当前上下文。
- **端到端加密与密钥闭环**：
  `native/mahayana-messaging/src/secret_chat.rs`（`SecretSession::initiate` / `accept`）-> 通过 X25519 协商共享密钥，HKDF-SHA256 派生会话密钥，XChaCha20Poly1305 进行防重放（`REPLAY_WINDOW_BITS = 128`）加密传输。

#### Q3（断点精确诊断）
1. **本地工作台与 VNC 视图 Token 割裂断点**（`frontend/src/production/LocalComputerWorkbench.tsx:53-57` 与 `computer/shell/view.tsx:35-48`）：
   ```tsx
   background: "var(--sand-surface-elevated, var(--cursor-bg-editor, #1e1e24))",
   border: "1px solid var(--sand-border-weak, rgba(128, 128, 128, 0.18))",
   ```
   **诊断**：在已落地的生产组件中残留大量历史遗留样式前缀（`--sand-*`、`--cursor-*`、`sand-kit-button ...`），未收敛至 `--fabushi-*` 语义 Token 系统，导致深浅色主题切换时背景对比度不合格。
2. **Computer 控制租赁未与前端 UI 挂钩（`wiring=not-wired`）**（`projects/openclaw-rust/dossiers/plugins-computer-use-and-config-scoped.md:57-80`）：
   Host 端虽然完整实现了 `ComputerUseCoordination`、`ComputerControlLease` 与 `sand_computer_auto_review.rs`，但前端 `computer/shell/controller.ts` 与 `LocalComputerWorkbench.tsx` 缺少直接反映当前租赁 Generation（`lease.generation()`）的指示条，未能向用户明确呈现当前是否有子代理正在独占控制。
3. **多账号切换旧请求未全面熔断断点**（`account-authorization.ts:10-18`）：
   `account-authorization.ts` 在执行 `store.scopeToAccount(scope)` 时，仅清空了 McpCustomInstructions 缓存，未向正在进行的远程 Computer 长连接与 WebSocket 管道发出显式的 `ACCOUNT_GENERATION_REVOKED` 强阻断信号，存在极短窗口内的跨账号迟到事件混淆隐患。

#### Q4（状态机与安全防护）
- **Computer 设备控制 6 态状态转移方程**：
  $$\text{ComputerState} \in \{\text{Off}, \text{Starting}, \text{Ready}, \text{InUseByAgent}, \text{HumanTakeover}, \text{Error}\}$$
  - $\text{Off} \xrightarrow{\text{worker\_spawn}} \text{Starting}$
  - $\text{Starting} \xrightarrow{\text{vnc\_ready \& handshake\_ok}} \text{Ready}$
  - $\text{Ready} \xrightarrow{\text{acquire\_lease(subagent)}} \text{InUseByAgent}$
  - $\text{InUseByAgent} \xrightarrow{\text{user\_press\_esc / click\_takeover}} \text{HumanTakeover}$
  - $\text{HumanTakeover} \xrightarrow{\text{user\_handback}} \text{InUseByAgent}$
  - $\text{InUseByAgent} \xrightarrow{\text{release\_lease / task\_finish}} \text{Ready}$
  - $\text{Any} \xrightarrow{\text{transport\_crash / heartbeat\_timeout}} \text{Error}$
- **`accountScopeGeneration` 物理隔离围栏**：
  每次账号切换或重新登录，`accountScopeGeneration` 单调递增加 1。所有入站的远程设备事件、截屏帧与 RPC 响应必须携带对应的 generation；若收到不一致数据，一律就地丢弃并触发安全审计警告。
- **高危操作审批门禁（SandComputerAutoReview）**：
  非免检动作（`click`、`doubleClick`、`drag`、`type`、`key`）必须经过 `SandAutoReviewController` 审查，仅 `SAND_COMPUTER_AUTO_REVIEW_BYPASS_ACTIONS`（`screenshot`、`move`、`wait`、`scroll`）免于人机交互式弹窗审批。

#### Q5（UI/UX 极致融合落点）
- **远程设备工作台标准落点**：
  - 在“设置 -> 设备与算力（Devices & Nodes）”中提供全局物理/云端节点资产看板。
  - 在会话中当 Agent 触发 Computer Use 工具时，中央舞台右侧平滑滑出统一的 `ComputerSurface` 抽屉，支持 VNC 画面与本地极速 Shell 命令行终端（`LocalComputerWorkbench`）一键 Tab 切换。
- **紧急安全断开按键（Kill-Switch / Takeover）**：
  在画面顶部显著展示黄色/红色状态条，提供高亮按钮“立即接管（Takeover）”及快捷键 `Esc`，按下后立即向 Host 派发 `release_control_lease` 强行阻断 Agent 输入。
- **设计规范与无障碍**：
  全面接入 `--fabushi-surface-canvas`、`--fabushi-status-warning` 等语义 Token；终端输出与 VNC 视图具备正确的 `aria-label` 与对比度保障。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与源码定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-08-01** | 本地 Computer 工作台与 VNC 视图样式 Token 严重割裂，深浅色对比度失效 | `frontend/src/production/LocalComputerWorkbench.tsx:53-57`<br>`frontend/src/recovered/features/computer/shell/view.tsx:35-48` | 残留历史 `--sand-*`、`--cursor-*` 与私有 `sand-kit-*` CSS 类名，未统一绑定至 `--fabushi-*` 语义设计系统变量。 | **P1 样式割裂** |
| **BUG-08-02** | Computer 控制租赁与前端缺乏双向通信，用户无法感知 Agent 独占状态 | `source/host/src/runner/computer_use.rs:37-53`<br>`frontend/src/recovered/features/computer/shell/model.ts:30-40` | 后端 `ComputerControlLease`（包含 `owner_agent_id` 与 `generation`）未完全上报给前端状态机，前端 `ComputerStatusProjection` 缺少 Lease 字段。 | **P1 状态断链** |
| **BUG-08-03** | 账号切换缺乏全局强阻断广播，在途远程设备连接存在极短竞态污染窗口 | `source/electron-main/account/account-authorization.ts:10-18`<br>`source/host/src/runner/computer_use.rs:76-92` | 切换账号时未向 Coordinator 与 Host 派发 `revoke_all_leases(new_generation)`，未能使旧在途连接立即断开。 | **P0 安全隐患** |
| **BUG-08-04** | 节点配对入网错误提示生硬，断网时缺乏自愈与明确重试指引 | `source/host/src/network_enrollment_owner.rs:38-53`<br>`frontend/src/recovered/features/computer/shell/controller.ts` | `NetworkEnrollmentError` 仅返回 Rust 内部枚举文本，前端未转换为符合体验规范的中文可读指引与重试向导。 | **P2 体验缺陷** |
| **BUG-08-05** | 自动更新服务未接入统一通知中心，缺少静默验证与错误回滚保护 | `source/electron-main/update/sand-update-service.ts:12-40`<br>`source/electron-main/update/safe-relaunch-gate.ts` | 更新服务采用私有定时循环检查，未在下载包损坏或验签失败时将诊断事件发布给前端设置面板。 | **P2 运维断点** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 统一架构拓扑图

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                               Frontend Presentation Layer                              │
│  ┌───────────────────────────────────────────┐  ┌───────────────────────────────────┐  │
│  │ Settings > Devices & Nodes (全局设备看板)  │  │ Conversation Stage > Computer Drawer│  │
│  │ - 物理节点列表与入网二维码 (Enrollment QR)  │  │ - 远程 VNC 画面 (noVNC / Canvas)    │  │
│  │ - 多账号安全会话与活跃设备列表 (Sessions)  │  │ - LocalComputerWorkbench 极速命令行  │  │
│  │ - 云密码设置与端侧 Passkey 管理            │  │ - 独占控制 Lease 横幅与紧急接管按键  │  │
│  └─────────────────────┬─────────────────────┘  └─────────────────┬─────────────────┘  │
└────────────────────────┼──────────────────────────────────────────┼────────────────────┘
                         │ IPC / RendererPortServer                 │ WebRTC / RFB WebSocket
                         ▼                                          ▼
┌────────────────────────────────────────────────────────────────────────────────────────┐
│                       Host & Coordinator Layer (Rust Native)                           │
│  ┌───────────────────────────────────────────┐  ┌───────────────────────────────────┐  │
│  │ source/node-agent-coordinator             │  │ source/host/src/runner/           │  │
│  │ - tdrp/settings_cloud_passkeys.rs         │  │ - computer_use.rs (Lease 调度)    │  │
│  │ - gateway/box_vnc_proxy.rs                │  │ - remote_computer_target.rs:8792  │  │
│  │ - cloud_worker_projection.rs              │  │ - sand_computer_auto_review.rs    │  │
│  └─────────────────────┬─────────────────────┘  └─────────────────┬─────────────────┘  │
│                        │                                          │                    │
│                        ▼                                          ▼                    │
│  ┌──────────────────────────────────────────────────────────────────────────────────┐  │
│  │ native/mahayana-messaging (基础设施层)                                           │  │
│  │ - device.rs (DeviceRegistry, DeviceSession, sync_cursor)                         │  │
│  │ - secret_chat.rs (X25519 + HKDF-SHA256 + XChaCha20Poly1305 双棘轮加密)           │  │
│  │ - network_enrollment_owner.rs (Fabushi 网络授权与设备指纹签名认证)               │  │
│  └──────────────────────────────────────────────────────────────────────────────────┘  │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 3.2 模块拆解与 200–600 行内聚预算清单

遵循 `Same Owner != Same File` 原则，所有相关代码均拆解为职责明确的子模块，严格恪守 200–600 行预算：

1. **`frontend/src/production/devices/DevicesAndNodesWorkspace.tsx`** (~360 行)
   - **职责**: 全局设备与算力中心主视图。展示当前绑定的远程物理主机、云端 Worker、当前本机硬件规格与配对二维码。
   - **预算**: 320–420 行。
2. **`frontend/src/production/devices/ComputerSurface.tsx`** (~420 行)
   - **职责**: 统领会话中的远程桌面抽屉。无缝装配 VNC 画布视窗与 `LocalComputerWorkbench`，集成独占 Lease 呼吸灯指示条与紧急接管按键。
   - **预算**: 380–480 行。
3. **`frontend/src/production/devices/computer-token-styles.css`** (~220 行)
   - **职责**: 彻底收口历史 `--sand-*` 与 `--cursor-*` 样式，全量映射为 `--fabushi-*` 语义设计系统变量。
   - **预算**: 200–260 行。
4. **`frontend/src/production/account/MultiAccountSecurityManager.ts`** (~310 行)
   - **职责**: 客户端多账号安全管理器。监听 `accountScopeGeneration` 变更，执行在途请求原子熔断与本地临时状态物理擦除。
   - **预算**: 280–350 行。
5. **`source/host/src/runner/computer_use_bridge.rs`** (~340 行)
   - **职责**: Rust 后端与 Coordinator 间的 Lease 广播桥接器。将 `ComputerControlLease` 与 `SandComputerAutoReview` 状态实时同步至前端 WebSocket。
   - **预算**: 300–400 行。
---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 渐进披露（Progressive Disclosure）层级设计

远程设备控制与安全管理引入大量底层的系统调用与网络状态。本蓝图设计三层渐进披露架构，将高频操作置于触手可及之处，低频高危配置安全收拢：

```
┌────────────────────────────────────────────────────────────────────────────────────────┐
│ Level 1: 顶栏状态指示与 Lease 呼吸横幅 (Global Status & Active Lease Banner)           │
│ - 状态胶囊: [🟢 本地宿主已就绪] [🟡 节点 Node-02 独占使用中 (Agent: 智能研究员)]       │
│ - 紧急接管按键: [🚨 立即接管控制 (Esc)] [一键静音/断开]                                │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Level 2: 统一 Computer Surface 工作台 (VNC & Workbench Surface)                        │
│ ┌────────────────────────────────────────────────────────────────────────────────────┐ │
│ │ 🖥️ 远程桌面画面 (VNC RFB 60fps 流式回传 / 画布自动缩放 / 真实鼠标轨迹投影)        │ │
│ │ ─── Tab 切换 ───                                                                   │ │
│ │ 💻 本地极速 Shell 命令行终端 (LocalComputerWorkbench, ANSI 高亮, PTY 伪终端交互)   │ │
│ │ 动作栏: [截屏] [发送按键] [剪贴板同步] [分辨率: 1920x1080@2x] [画质自适应]         │ │
│ └────────────────────────────────────────────────────────────────────────────────────┘ │
├────────────────────────────────────────────────────────────────────────────────────────┤
│ Level 3: 深度上下文与权限审计抽屉 (Deep Inspector & Auto-Review Gate)                  │
│ - 动态审批气泡 (Approval Dialog): Agent 正在尝试调用 bash rm -rf / -> [拦截: 拒绝/单次]│
│ - 审计轨迹流 (Audit Trail): 记录每个鼠标点击坐标、键盘敲击事件与窗口聚焦变更          │
│ - 节点状态自愈向导 (Doctor & Diagnostic): 端口连通性、证书有效期、网络延迟往返 RTT     │
└────────────────────────────────────────────────────────────────────────────────────────┘
```

### 4.2 响应式断点规范（Responsive Breakpoints）

严格遵循统一体验规范的自适应断点标准，确保多终端设备下的舒适操作：

1. **≥1200px（宽屏三栏并排模式）**：
   - 导航轨（52px）+ 消息列表/会话（400px–500px）+ 右侧常驻展开 `ComputerSurface` 抽屉（宽度 520px–680px）。
   - VNC 画布支持 1:1 像素映射或等比自适应缩放（Fit to Screen），全功能控制栏横向展开。
2. **900px–1199px（中屏双栏/浮动模式）**：
   - 导航轨保持 52px。
   - `ComputerSurface` 以滑出式侧边栏（Drawer）呈现，带高透毛玻璃遮罩，宽度 420px–500px，支持拖拽边缘调整宽度，按 `Esc` 键自动缩起并保留后台连接。
3. **<900px（移动端/窄屏全屏模式）**：
   - 切换到全屏工作台模式，隐藏左侧一切列表。
   - 顶栏保留固定的“返回会话（Back）”与“紧急接管”按钮，底栏提供虚拟浮动键盘工具条（Ctrl, Alt, Tab, Win/Cmd 快捷键直发）。

### 4.3 语义设计变量绑定（`--fabushi-*` Design Tokens）

全面清理原 `LocalComputerWorkbench.tsx` 与 `computer/shell/view.tsx` 中残留的历史 `--sand-*`、`--cursor-*` 与 `sand-kit-*` 类名，统一映射为语义 Token：

| UI 语义元素 | CSS Variable 绑定 | 浅色模式回退值 (Light) | 深色模式回退值 (Dark) | 无障碍说明 |
|---|---|---|---|---|
| **工作台画布背景** | `var(--fabushi-surface-canvas)` | `#f8fafc` | `#0f172a` | 主页面基底底色 |
| **设备卡片容器** | `var(--fabushi-surface-card)` | `#ffffff` | `#1e293b` | 满足层级深度质感 |
| **终端背景色** | `var(--fabushi-surface-terminal)` | `#18181b` | `#09090b` | 纯黑/深灰终端质感 |
| **控制栏边框** | `var(--fabushi-border-subtle)` | `rgba(0,0,0,0.08)` | `rgba(255,255,255,0.1)` | 区分视图边缘 |
| **Agent 独占警告** | `var(--fabushi-status-warning)` | `#d97706` | `#fbbf24` | 琥珀黄色呼吸灯，警示操作 |
| **正常就绪状态** | `var(--fabushi-status-success)` | `#16a34a` | `#4ade80` | 清晰草绿色，就绪无风险 |
| **连接断开/错误** | `var(--fabushi-status-failed)` | `#dc2626` | `#f87171` | 鲜明警示红色 |
| **终端文本高亮** | `var(--fabushi-text-primary)` | `#f1f5f9` | `#f8fafc` | 符合 WCAG 4.5:1 对比度 |

### 4.4 无障碍（A11y）与安全主动断开（Emergency Takeover / Stop Shortcut）

1. **紧急接管按键（Emergency Takeover / Kill-Switch）**：
   - 当检测到 Agent 正在通过 CUA 操作屏幕时，全局监听 `Esc` 键或组合键 `Ctrl+Alt+K`。触发时，界面毫秒级冻结 Agent 鼠标移动指令，向后端 Host 派发 `release_control_lease`，并弹出“您已接管控制权”的浮动公告。
2. **全键盘导航与焦点管理**：
   - VNC 画布获得焦点时，所有按键均转发给远程系统；按 `Ctrl+Alt+Backquote` 退出画布按键捕获模式，焦点精确返回至外层工具栏。
3. **中文输入法（IME）保护**：
   - 在 `LocalComputerWorkbench` 的输入框与 VNC 文本输入缓冲器中，严格挂载 `isComposing` 状态锁。当用户正在选词（Composition 过程）中敲击 `Enter` 时，拦截向远程 PTY 派发 `\r\n` 换行符，防止未定稿拼音被误提交执行。

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 核心 TypeScript 数据模型与 Props 契约

```typescript
// 远程设备与 Computer Use 核心生命周期
export type ComputerControlPhase =
  | "off"
  | "starting"
  | "ready"
  | "in_use_by_agent"
  | "human_takeover"
  | "error";

export interface ComputerControlLease {
  readonly ownerAgentId: string;
  readonly windowIndex: number;
  readonly generation: number;
  readonly acquiredAtMs: number;
}

export interface RemoteNodeInfo {
  readonly deviceId: string;
  readonly deviceName: string;
  readonly platform: "macos" | "windows" | "linux";
  readonly appVersion: string;
  readonly lastActiveAtMs: number;
  readonly isOnline: boolean;
  readonly syncCursor: number;
  readonly ipAddress?: string;
  readonly port: number;
}

export interface ComputerSurfaceProps {
  readonly currentAccountId: string;
  readonly accountScopeGeneration: number;
  readonly activeAgentId?: string;
  readonly currentLease?: ComputerControlLease | null;
  readonly onTakeoverControl: () => Promise<void>;
  readonly onReleaseControl: () => Promise<void>;
  readonly onExecuteShellCommand: (command: string) => Promise<{ stdout: string; stderr: string }>;
  readonly onRefreshVnc: () => void;
}

export interface MultiAccountSessionDescriptor {
  readonly accountId: string;
  readonly accountScopeGeneration: number;
  readonly username: string;
  readonly displayName: string;
  readonly hasCloudPassword: boolean;
  readonly activeDevices: readonly RemoteNodeInfo[];
}
```

### 5.2 Rust 后端宿主契约（Host Protocol / Remote Computer Target）

```rust
// source/host/src/runner/computer_use.rs 与 remote_computer_target.rs
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HostComputerPhase {
    Off,
    Starting,
    Ready,
    InUseByAgent,
    HumanTakeover,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostComputerControlLeaseEvent {
    pub account_scope_generation: u64,
    pub owner_agent_id: String,
    pub window_index: u32,
    pub lease_generation: u64,
    pub phase: HostComputerPhase,
    pub updated_at_ms: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostAutoReviewGateRequest {
    pub account_scope_generation: u64,
    pub approval_id: String,
    pub action_name: String, // "click", "doubleClick", "drag", "type", "key"
    pub target_window: Option<String>,
    pub input_summary: String,
    pub expires_at_ms: i64,
}
```

### 5.3 隔离围栏（Fencing Rules）与防竞态规则

1. **`accountScopeGeneration` 单调递增与原子熔断（Atomic Revocation）**：
   - 每次账号切换，客户端派发 `scopeToAccount(newSlot)`，递增全局 `accountScopeGeneration`。
   - Coordinator 收到后立即广播 `revoke_all_active_leases(accountScopeGeneration)`。
   - 所有在途的远程桌面截屏、Shell 管道与 WebSocket 事件若携带的 generation 小于当前系统 generation，立即就地丢弃并安全关闭传输套接字，防止跨账号信息串扰。
2. **`lease.generation` 独占锁防脑裂（Single-Flight Concurrency & Split-Brain Prevention）**：
   - `ComputerUseCoordination` 保证全局仅存在一个活跃 `ComputerControlLease`。
   - 当新的子代理请求控制权时，如果上一个 Lease 尚未释放，严格返回 `COMPUTER_HOST_BUSY`。
   - 释放与接管操作均触发递增的 `lease_generation`，任何迟到的上一个 Lease 的输入指令直接失效。
3. **高危操作审批时效门禁（Approval TTL Enforce）**：
   - 任何触发 `SandComputerAutoReview` 的操作，生成 TTL 为 600 秒的 `approval_id`。
   - 若用户在 600 秒内未做出决策，状态机自动将请求置为 `Cancelled`，并向后端下发超时拒绝指令，杜绝挂起任务在用户离开后意外执行。
---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 任务切片划分

为保障多智能体并行实施的绝对隔离与零冲突，本蓝图将设备控制与后端对齐工作细化为 5 个原子实施切片：

| 切片编号 | 切片目标 | 允许修改文件白名单 | 只读参考文件 | 验收准则 |
|---|---|---|---|---|
| **SLICE-08-A** | ComputerSurface 与 LocalComputerWorkbench 样式收敛与统一抽屉装配 | `frontend/src/production/devices/ComputerSurface.tsx`<br>`frontend/src/production/devices/computer-token-styles.css`<br>`frontend/src/production/LocalComputerWorkbench.tsx`<br>`frontend/src/production/ProductionRenderer.tsx` (窄接线) | `frontend/src/recovered/features/computer/shell/view.tsx`<br>`frontend/src/production/openclaw-theme.css`<br>`projects/fabushi-communication-platform/current-owner-inventory.md` | 彻底移除 `--sand-*` 与 `--cursor-*` 样式变量；在 `ProductionRenderer.tsx` 中通过窄接线注入当前 `ComputerSurface` 抽屉。 |
| **SLICE-08-B** | Computer 控制租赁（Lease）双向广播与接管状态机接线 | `source/host/src/runner/computer_use.rs`<br>`source/node-agent-coordinator/src/gateway/box_vnc_proxy.rs`<br>`frontend/src/recovered/features/computer/shell/model.ts`<br>`frontend/src/recovered/features/computer/shell/controller.ts` | `source/host/src/remote_computer_target.rs`<br>`projects/openclaw-rust/dossiers/plugins-computer-use-and-config-scoped.md` | 将 `ComputerControlLease` 的 `owner_agent_id`、`window_index` 与 `generation` 实时同步至前端；支持通过 Esc 键触发毫秒级主动接管。 |
| **SLICE-08-C** | 多账号 `accountScopeGeneration` 熔断与设备会话管理 | `frontend/src/production/account/MultiAccountSecurityManager.ts`<br>`source/electron-main/account/account-authorization.ts`<br>`source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs` | `source/electron-main/entry.ts`<br>`native/mahayana-messaging/src/device.rs` | 切换账号时原子派发熔断广播，旧账号在途网络连接与 WebSocket 订阅立即终止，丢弃所有跨账号事件。 |
| **SLICE-08-D** | 物理节点入网配对向导与自愈状态机 | `frontend/src/production/devices/DevicesAndNodesWorkspace.tsx`<br>`source/host/src/network_enrollment_owner.rs` | `contracts/automation/cross-platform-journeys.json`<br>`native/mahayana-messaging/src/device.rs` | 提供清晰友好的节点二维码配对与端口检测向导；断网时诚实提示“离线”并提供自愈重试通道。 |
| **SLICE-08-E** | 端到端加密与密钥闭环保障 | `native/mahayana-messaging/src/secret_chat.rs`<br>`source/node-agent-coordinator/src/tdrp/settings_cloud_passkeys.rs` | `Telegram/SourceFiles/storage/storage_encryption.cpp`<br>`Telegram/SourceFiles/tde2e/` | 验证 X25519 密钥协商与 XChaCha20Poly1305 加密正确性；实现会话注销时的端侧物理密钥安全擦除（Zeroize）。 |

### 6.2 与 `ProductionRenderer.tsx` 的窄接线规范

在主装配器 `frontend/src/production/ProductionRenderer.tsx` 中，遵循最小侵入式窄接线原则，禁止在装配器内直接书写复杂设备控制状态机：

```tsx
// frontend/src/production/ProductionRenderer.tsx 窄接线示例
{isComputerDrawerOpen && (
  <ComputerSurface
    currentAccountId={currentAccountId}
    accountScopeGeneration={accountScopeGeneration}
    activeAgentId={activeAgent?.id}
    currentLease={activeComputerLease}
    onTakeoverControl={handleTakeoverComputer}
    onReleaseControl={handleReleaseComputer}
    onExecuteShellCommand={handleExecuteShell}
    onRefreshVnc={handleRefreshVnc}
  />
)}
```

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 正反例检查表（Anti-Cheating Checklist）

本蓝图设立严格的独立验收标准，由独立验收子智能体（Auditor / Challenger）对照以下检查表逐项核验。任何一条违反均视为作弊，直接判决交付不合格：

| 检查项 | 正确实现标准（Acceptance Passing） | 严禁作弊/违规行为（Forbidden Cheating） | 审计判定方式 |
|---|---|---|---|
| **VNC 画面真实性** | 画面必须由 Host VNC WebSocket（RFB 协议）真实流式回传，真实反映系统变化 | 使用预录制 GIF、静态截图轮播或本地 `setTimeout` 假动画伪造远程操作 | 检查网络连接面板是否存在真实的 RFB 二进制 WebSocket 流；关闭远程宿主验证是否诚实报连接中断。 |
| **控制租赁单飞锁** | 当子代理持有 Lease 时，其他 Agent 请求必须返回 `COMPUTER_HOST_BUSY` 并排队等待 | 前端不加锁允许多个 Agent 同时向同一窗口发送按键，或前端静默忽略冲突 | 模拟两个 Agent 并发调用 `acquire_control_lease`，验证是否有且仅有一个成功获得当前 generation。 |
| **多账号强隔离** | 切换账号后，旧账号的所有长连接与数据流立即中断，`accountScopeGeneration` 单调递增 | 切换账号后旧账号的截屏继续在屏幕上显示，或在控制台出现迟到数据报错 | 切换账号，断点验证旧账号的 WebSocket `close()` 调用及事件丢弃逻辑。 |
| **紧急接管响应** | 按下 `Esc` 键后 50ms 内向 Host 下发 `release_control_lease` 并冻结 Agent 输入 | `Esc` 键仅在前端隐藏抽屉，后端 Agent 仍在后台无感知继续敲击键盘 | 在 Agent 执行自动化时按下 `Esc`，检查后端日志是否收到立即释放控制权指令。 |
| **设计 Token 规范** | 全面使用 `--fabushi-*` 语义变量，在浅色/深色主题下对比度符合 WCAG AA 标准 | 残留 `--sand-*`、`--cursor-*` 变量，或在组件内直接写死 `#1e1e24` 等十六进制颜色 | 全文搜索检查文件：`grep -E "(--sand-|--cursor-|sand-kit-)" frontend/src/production/devices/*`。 |
| **端到端加密合规** | 严格采用 X25519 + HKDF-SHA256 + XChaCha20Poly1305 双棘轮防重放加密 | 私自回退至明文传输、伪造硬编码 Key，或继续使用废弃的私有 MTProto 加密 | 静态核验 `native/mahayana-messaging/src/secret_chat.rs` 中的算法实现与测试用例。 |
| **代码内聚预算** | 所有新增与重构的单一文件代码行数严格控制在 200–600 行以内 | 产生单个超过 800 行的巨石文件，或直接全部塞入 `ProductionRenderer.tsx` | 统计新增文件行数：`wc -l frontend/src/production/devices/*`。 |
