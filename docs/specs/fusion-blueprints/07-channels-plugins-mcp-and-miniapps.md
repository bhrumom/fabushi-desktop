# Fabushi 统一融合蓝图 07：外部渠道、插件市场、MCP、Skills 与 Mini Apps
# Volume 07: Channels, Plugins, MCP, Skills, and Mini Apps

> **规范编号**: `FBCP-BLUEPRINT-07` / `UFX-BLUEPRINT-07`  
> **归属版本**: `UFX-001` Revision 1  
> **执行分支**: `design/unified-fabushi-20261105`  
> **上游只读锁定基准**:
> - OpenClaw: Commit `@1440bb1771372334bea23d7642bb52deb7db1845`（Tree `@e8cde7518c558f1b4e44cb8139a149289b9719ad`）
> - Telegram Desktop: Commit `@f985723b552b30fe1c17d377d57f44e61a15a8c4`（Frozen Tree `@33261535a0e747f125e0ed25486f01e556330677`）
> - Fabushi Main: Commit `@ba9ecce728ef021fee609796c8704d8af6fe4b5a`
> **核心目标**:
> 1. **外部渠道（Channels）接入向导与真实连接状态**: 统一外部第三方通信平台（Telegram Bot, Discord, Slack, WhatsApp, 飞书等）的配对向导（Setup Wizard）、凭据持久化与 Dirty Guard（未保存变更拦截）；彻底废除“18 个连接器全量就绪”硬编码假状态，实现基于 Host 真实握手的在线/断线/错误状态投影。
> 2. **插件生态与 MCP Registry 统一纳管**: 统一 OpenClaw 插件 SDK（OCRM-M16）与 Model Context Protocol（MCP, OCRM-M18）服务注册中心。提供 Stdio/SSE/HTTP 统一传输通道、多账号槽位切换、工具启用/禁用仲裁、OAuth 回调闭环及企业策略继承。
> 3. **Skills 系统与工作流安全沙箱**: 落地领域技能定义（OCRM-M17）与 Custodian Skills 自动化脚本；绑定 `plugin_skills_cache.rs` 本地索引与原子刷新机制，提供动态参数化校验与 Shell 命令注入防护。
> 4. **离线优先 Mini App / WebView 容器与权限隔离**: 融合 Telegram Bot Mini Apps（TCG-N27）与 Fabushi 原生可安装小程序容器（`fabushi-miniapp://`），提供基于 Ed25519 签名验证、严格 CSP 沙箱、离线拦截代理（`127.0.0.1:9`）、权限动态审批与独立 WebContents 隔离。
> 5. **云端安全设置、2FA 与多协议代理网络**: 接入 Telegram 云安全与隐私控制（TCG-N24/N25/N37），落地两步验证云密码（2FA Cloud Password）、WebAuthn 通行密钥（Passkeys）、隐私白名单（10 项规则）以及 SOCKS5/HTTP/MTProto 代理轮换与链路测速。

---

## 1. 源码精读与全量功能零遗漏矩阵（Source Inventory & Zero-Omission Matrix）

### 1.1 三方源码全量对照矩阵

| 责任域 / 能力项 | Telegram Desktop 锁定源码锚点 | OpenClaw 锁定源码锚点 | Fabushi 既有生产代码落点 | 融合后 Canonical Owner |
|---|---|---|---|---|
| **外部渠道向导与凭据安全**<br>(OCRM-M14, Channels Setup & Wizard) | `Telegram/SourceFiles/api/api_authorizations.cpp`<br>`core/local_url_handlers.cpp` | `src/channels/`<br>`src/flows/channel-setup.ts`<br>`extensions/telegram/`<br>`extensions/discord/`<br>`extensions/slack/`<br>`extensions/feishu/` | `frontend/src/recovered/features/agent-info/channels/model.ts:8-96`<br>`frontend/src/recovered/features/agent-info/channels/view.tsx:1-151`<br>`source/shared/channels.ts` | `agentChannelsController`<br>`ContextInspector` Channels Tab<br>`source/host/src/extensions/channel/` |
| **渠道入站防抖与幂等去重**<br>(OCRM-M14, Inbound Debounce & Deduplication) | `Telegram/SourceFiles/api/api_updates.cpp:120-210`<br>`data/data_session.cpp` | `src/channels/inbound/`<br>`packages/gateway-protocol/` | `source/host/src/connectors/channel_delivery.rs:264-301`<br>`source/host/src/connectors/channel_delivery.rs:388-468`<br>`source/host/src/extensions/transcript/channel_delivery_unregistered_error.rs` | `InboundChannelDeduplicator`<br>`InboundChannelDebounceGate`<br>`source/host/src/connectors/channel_delivery.rs` |
| **渠道出站路由与分块投递**<br>(OCRM-M15, Outbound Delivery & Split) | `Telegram/SourceFiles/api/api_sending.cpp:310-450`<br>`history/history_item.cpp` | `src/auto-reply/`<br>`src/chat/`<br>`src/routing/` | `source/host/src/connectors/channel_delivery.rs:11-68`<br>`source/host/src/connectors/channel_delivery.rs:621-700`<br>`source/host/src/connectors/channel_attachment.rs` | `ChannelDeliveryRouter`<br>`split_channel_message`<br>`source/host/src/connectors/channel_delivery.rs` |
| **渠道访问控制门禁与 Webhook 验签**<br>(OCRM-M14, ACL Gate & HMAC Verification) | `Telegram/SourceFiles/core/network_proxy_manager.cpp`<br>`core/sandbox_rules.cpp` | `src/security/`<br>`packages/net-policy/` | `source/host/src/connectors/channel_delivery.rs:100-200`<br>`source/host/src/connectors/channel_delivery.rs:208-262` | `ChannelAclGate`<br>`verify_webhook_hmac_sha256`<br>`source/host/src/connectors/channel_delivery.rs` |
| **插件市场与安装包生命周期**<br>(OCRM-M16, Plugin SDK & Package Lifecycle) | `Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp`<br>`boxes/web_view_box.cpp` | `src/plugins/`<br>`src/plugin-sdk/`<br>`packages/plugin-sdk/`<br>`packages/plugin-package-contract/` | `source/host/src/extensions/mcp/installable_package.rs:1-120`<br>`frontend/src/recovered/features/plugins/overlay/browser.tsx:1-200`<br>`frontend/src/recovered/features/plugins/overlay/model.ts` | `MarketplaceStore`<br>`PluginsBrowser`<br>`source/host/src/extensions/mcp/installable_package.rs` |
| **MCP Registry 与 Stdio/SSE 传输**<br>(OCRM-M18, MCP Server Registry & Relay) | `Telegram/SourceFiles/api/api_bot_send_ready.cpp`<br>`inline_bots/` | `src/mcp/`<br>`extensions/mcp-relay/`<br>`extensions/mcp-events/`<br>`packages/sdk/` | `source/electron-main/mcp/desktop-mcp-manager.ts:1-150`<br>`source/electron-main/mcp/official-mcp-service.ts`<br>`source/host/src/extensions/mcp/coordinator_relay.rs`<br>`source/host/src/extensions/mcp/mcp_service.rs` | `DesktopMcpManager`<br>`McpService`<br>`source/electron-main/mcp/desktop-mcp-manager.ts` |
| **Skills 系统与本地缓存引擎**<br>(OCRM-M17, Skills System & Cache Store) | `Telegram/SourceFiles/data/data_folder.cpp`<br>`data/data_user.cpp` | `src/skills/`<br>`skills/` (50 目录)<br>`custodian-skills/`<br>`skills/pyproject.toml` | `source/host/src/extensions/mcp/plugin_skills.rs:1-120`<br>`source/host/src/extensions/mcp/plugin_skills_cache.rs:1-80`<br>`source/electron-main/mcp/host-plugin-skills-loader.ts` | `PluginSkillsCache`<br>`HostPluginSkillsLoader`<br>`source/host/src/extensions/mcp/plugin_skills.rs` |
| **Mini App / WebView 容器沙箱**<br>(TCG-N27, Bot Mini Apps & WebView Surface) | `Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp:1-410`<br>`boxes/web_view_box.cpp:1-320`<br>`core/sandbox_rules.cpp` | `src/canvas/`<br>`src/interactive/`<br>`extensions/code-mode-quickjs/` | `source/electron-main/mcp/miniapp-window.ts:1-120`<br>`source/electron-main/mcp/miniapp-management.ts`<br>`frontend/src/recovered/features/plugins/overlay/miniapp-management.tsx:1-50`<br>`source/shared/miniapp-tool-contract.ts` | `MiniAppWindowManager`<br>`miniapp-window.ts`<br>`frontend/src/recovered/features/plugins/overlay/miniapp-management.tsx` |
| **两步验证云密码与 Passkeys**<br>(TCG-N25, 2FA Cloud Security & Passkeys) | `Telegram/SourceFiles/core/two_factor_auth.cpp:1-380`<br>`boxes/passcode_box.cpp`<br>`storage/storage_account.cpp` | `src/secrets/`<br>`src/pairing/`<br>`extensions/vault/` | `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:1-120,180-213`<br>`source/electron-main/secrets/`<br>`source/host/src/extensions/auth/` | `TdrpCloudSecurityPanel`<br>`CloudSecurityService`<br>`source/electron-main/secrets/` |
| **隐私规则白名单管理**<br>(TCG-N24, Privacy Rules & Permissions) | `Telegram/SourceFiles/settings/settings_privacy_controllers.cpp:1-512`<br>`boxes/abstract_box.cpp` | `src/security/`<br>`packages/net-policy/` | `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:7-18,57-68,113-126` | `TdrpCloudSecurityPanel`<br>`CoordinatorConnectionHost` (`settings.setPrivacyRule`) |
| **网络代理配置与自动轮换测速**<br>(TCG-N37, Proxy Management & Rotation) | `Telegram/SourceFiles/core/network_proxy_manager.cpp:1-620`<br>`boxes/proxy_box.cpp:1-240` | `src/infra/`<br>`src/proxy-capture/` | `frontend/src/recovered/contracts/desktop-bridge.ts:165-184`<br>`source/host/src/extensions/settings/settings_service.rs` | `NetworkProxyStore`<br>`desktopBridge.networkProxy` |

---

### 1.2 强制源码五问逐项回答（The Mandatory Source Five Questions）

#### Q1（上游真实现状）
- **OpenClaw 外部渠道（OCRM-M14/M15）**:
  上游采用模块化 Channel Provider 架构（如 `extensions/telegram/`, `extensions/discord/`），每个 Provider 实现握手认证、Webhook/Long-polling 接收，以及通过 ACP 协议转发消息至 Agent Runtime。核心状态包括 `unconfigured -> authenticating -> ready -> error`。入站具有游标保序（`InboundChannelDeduplicator`）与文本消息防抖窗口（`ChannelInboundDebounceGate`，默认 5 秒，上限 60 秒）；控制命令（`/abort`, `/stop`, `/status`）与富媒体无条件立即放行；消息分块根据 Markdown 代码块智能闭合（`split_channel_message`，单条 4,000 字符限制）。
- **OpenClaw 插件与 MCP 生态（OCRM-M16/M17/M18）**:
  - MCP 基于 JSON-RPC 2.0 规范，提供 Stdio 子进程通信、SSE 持续流与 HTTP 传输。支持多账号隔离配置（`accountSlots`）与企业策略（`team-required` / `team-default` / `user`）。
  - Skills 基于本地目录扫描与索引缓存（`plugin_skills_cache.rs`），执行前进行依赖检查（Python/Node CLI 环境）与命令参数白名单校验。
- **Telegram Desktop Mini Apps 与云安全（TCG-N24/N25/N27/N37）**:
  - Bot Mini Apps（`bot_attach_web_view.cpp`）在独立沙箱窗口或底部抽屉中运行，通过安全桥接协议与客户端交互（`TelegramWebviewProxy`），严格禁止直接访问宿主 DOM 与 Node.js API；网络请求由受控代理仲裁。
  - 云安全（`two_factor_auth.cpp`）采用基于 SRP（Secure Remote Password）协议的 2FA 云密码保护，支持提示词（Hint）、重置倒计时（Pending Reset Date）与 WebAuthn Passkeys 硬件认证；隐私规则（`settings_privacy_controllers.cpp`）覆盖 10 项细粒度策略，包含 `AllowAll`、`ContactsOnly`、`DisallowAll` 三种判决。
  - 代理管理（`network_proxy_manager.cpp`）支持 SOCKS5、HTTP、MTProto 代理，具备毫秒级 Ping 测速与掉线自动轮换。

#### Q2（现有链路定位）
- **外部渠道配置与连接链路**:
  `frontend/src/recovered/features/agent-info/channels/view.tsx`（`AgentInfoChannelsPanel`）-> `model.ts`（`AgentInfoChannelsController`）-> Coordinator RPC (`getAgentChannels`, `connectChannel`, `disconnectChannel`, `refreshChannel`) -> `source/host/src/connectors/channel_delivery.rs`（`ChannelDeliveryRouter`, `ChannelAclGate`）。
- **插件市场与 MCP 管理链路**:
  `frontend/src/recovered/features/plugins/overlay/browser.tsx`（`PluginsBrowser`）-> `frontend/src/recovered/contracts/desktop-bridge.ts:DesktopBridge.mcp` -> `source/electron-main/mcp/desktop-mcp-manager.ts` -> Host `source/host/src/extensions/mcp/coordinator_relay.rs` / `mcp_service.rs`。
- **Skills 缓存与执行链路**:
  `source/electron-main/mcp/host-plugin-skills-loader.ts` -> Host `source/host/src/extensions/mcp/plugin_skills.rs` -> 本地磁盘缓存 `plugin_skills_cache.rs`。
- **Mini App 容器与启动链路**:
  `frontend/src/recovered/features/plugins/overlay/miniapp-management.tsx`（`MiniAppManagement`）-> `desktopBridge.marketplaceMiniApps.launch` -> `source/electron-main/mcp/miniapp-window.ts`（`createMiniAppWindow`）-> 校验 Ed25519 包签名并装载 `fabushi-miniapp://` 协议资源。
- **云安全与代理链路**:
  `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx` -> `ProductionCoordinatorClient` (`settings.setPrivacyRule`, `auth.setCloudPassword`, `auth.registerPasskey`)；代理配置通过 `desktopBridge.networkProxy` 存取于 `ProductNetworkProxySettings`。

#### Q3（断点精确诊断）
1. **渠道配置缺少未保存变更拦截（Dirty Guard 缺失）**:
   `frontend/src/recovered/features/agent-info/channels/view.tsx:98-103` 中，用户在输入框内输入 Token 凭据时，若直接关闭面板或切换 Tab，没有弹出未保存警告（Dirty Guard），导致已输入的敏感 Token 意外丢失；同时缺乏输入内容的格式预校验（如 Telegram Bot Token 的 `123456:ABC-DEF...` 正则检验）。
2. **连接器虚假就绪声明历史遗留**:
   原 `OpenClawAskCopilotPanel.tsx:42` 曾硬编码“18 个连接器全量就绪”，未与 `AgentInfoChannelsSnapshot` 中的真实连接状态（`connected` / `connecting` / `error` / `available`）联动。
3. **Mini App 安全沙箱存在协议逃逸风险**:
   `source/electron-main/mcp/miniapp-window.ts:16` 设置了 `corsEnabled: false`，但如果第三方包引用的外部链接触发 `window.open`，若未配置 fail-closed 拦截，可能弹出未受控的外部系统默认浏览器窗口；必须通过 `setWindowOpenHandler(() => ({ action: "deny" }))` 严格封死。
4. **云安全面板假 Hash/Salt 本地伪造**:
   `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:132-133` 中：
   ```tsx
   const hash = `hash_${btoa(passwordInput.trim())}`;
   const salt = `salt_${Date.now()}`;
   ```
   **严重违规**: 前端使用简单的 Base64 和时间戳拼接充当密码学 Hash 与 Salt！这在生产环境中是致命的安全作弊逻辑，必须改用 Native Rust 提供的 PBKDF2-HMAC-SHA512 或 Argon2id 进行真正的密钥派生。
5. **代理配置缺少连接测试与真实 Ping 测量**:
   `frontend/src/recovered/contracts/desktop-bridge.ts:168-184` 定义了 `ProductNetworkProxySettings`，但在前端界面中缺少像 Telegram 原生那样的真实链路延迟（TCP Ping / HTTP Roundtrip）实时测试功能，用户无法获知所配置的 SOCKS5/HTTP 代理当前是否畅通。

#### Q4（状态机与安全防护）
- **外部渠道接入生命周期 5 态流转方程**:
  $$\text{ChannelStatus} \in \{\text{Unconfigured}, \text{Connecting}, \text{Connected}, \text{Error}, \text{ComingSoon}\}$$
  - $\text{Unconfigured} \xrightarrow{\text{submit\_token}} \text{Connecting}$
  - $\text{Connecting} \xrightarrow{\text{host\_auth\_ack}} \text{Connected}$
  - $\text{Connecting} \xrightarrow{\text{auth\_failure / timeout (15s)}} \text{Error}$
  - $\text{Connected} \xrightarrow{\text{heartbeat\_loss / rate\_limit}} \text{Error}$
  - $\text{Connected / Error} \xrightarrow{\text{disconnect}} \text{Unconfigured}$
- **Mini App 双层沙箱隔离围栏**:
  - **网络层隔离**: 离线 Mini App 启动时强制挂载离线代理 `http=127.0.0.1:9;https=127.0.0.1:9;socks5=127.0.0.1:9`，彻底阻断任何对公网的非法数据窃取与出站外泄。
  - **上下文与协议层隔离**: 自定义 `fabushi-miniapp://` 协议仅响应已通过 Ed25519 签名验证且哈希一致的静态资源包，禁止加载任意未声明的远程脚本；Session 分区绑定 `persist:miniapp-${accountScope}-${appId}`，严禁跨 Mini App 或跨账号共享 LocalStorage/Cookies。
- **多账号切换围栏（Account Scope Generation Fencing）**:
  切换账号时，递增 `accountGeneration`：所有在途的 MCP 会话、未完成的 OAuth 握手、Mini App 打开窗口以及渠道轮询任务立即强制销毁，旧账号上下文事件直接静默丢弃。

#### Q5（UI/UX 极致融合落点）
- **外部渠道（Channels）**: 收敛于右侧 `ContextInspector` 的 **Channels** Tab。展示当前 Agent 已连接的外部渠道状态指示卡片，点击提供展开式向导弹窗（Setup Wizard），内含分步骤操作指引与代码高亮块；关闭前触发 Dirty Guard 确认。
- **插件市场与 MCP 注册中心**: 整合入一级左侧导航的 **Marketplace**（或者通过 `Cmd/Ctrl+,` 设置中的 Extensions 视图打开）。双轴过滤器（Marketplace / Yours；All / Connectors / Skills；All / Team / Public），卡片式列表支持搜索、安装、权限配置与工具开关。
- **Mini Apps**: 作为独立模态视窗（Sub-window）或在聊天流中以卡片方式点击拉起，具有专属的极简标题栏（含安全沙箱标识、刷新、权限详情与关闭按钮）。
- **云安全与代理设置**: 收拢于全局 **Settings -> Privacy & Security**，采用干净的卡片式分组布局，直观展示 2FA 开关、Passkeys 管理列表与多节点代理测速列表。

---

## 2. 现有割裂问题与根因分析（Current Fragmentation & Root Cause）

### 2.1 缺陷清单与根因定位

| 缺陷编号 | 现象与表现 | 源码文件与精确行号 | 根因深度剖析 | 严重等级 |
|---|---|---|---|---|
| **BUG-07-01** | `TdrpCloudSecurityPanel` 使用 `btoa` 和 `Date.now()` 伪造 2FA 密码 Hash 和 Salt | `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx:132-133,167` | 前端开发者为了应付接口，使用不安全的 Base64 明文编码模拟哈希，直接绕过了密码学真实性，属于严重的假安全违规。 | **P0 安全作弊** |
| **BUG-07-02** | 外部渠道输入凭据时缺少 Dirty 状态追踪与离开二次确认 | `frontend/src/recovered/features/agent-info/channels/view.tsx:53-70,98-103` | `AgentInfoChannelRow` 使用本地临时变量，在未保存状态下切换或关闭没有状态守卫（Dirty Guard），导致输入丢失或误触提交。 | **P1 交互缺陷** |
| **BUG-07-03** | 渠道连接状态硬编码“18 个连接器全量就绪”断点残留 | `frontend/src/production/openclaw/OpenClawAskCopilotPanel.tsx:42` | 历史遗留的假面板静态文本未与 `AgentInfoChannelsSnapshot` 真实状态机建立动态数据绑定。 | **P0 真实性违规** |
| **BUG-07-04** | Mini App 窗口缺失 `setWindowOpenHandler` 显式 deny 规则 | `source/electron-main/mcp/miniapp-window.ts:57` | 虽然在接口中有方法定义，但在实际窗口装配时若未严格挂载阻断回调，不可信小程序页面可通过 `target="_blank"` 逃逸沙箱。 | **P1 安全隐患** |
| **BUG-07-05** | 网络代理设置缺少真实连通性测试与延迟测速（Ping）机制 | `frontend/src/recovered/contracts/desktop-bridge.ts:168-184` | 仅有配置的增删改查 DTO，底层没有提供向代理节点发送 TCP/HTTP 探测请求并测量 RTT 的链路监控逻辑。 | **P2 功能断层** |
| **BUG-07-06** | 插件市场与 MCP 注册中心展示分散，缺乏统一的权限与工具仲裁视图 | `frontend/src/recovered/features/plugins/overlay/browser.tsx`<br>`miniapp-management.tsx` | 插件、离线小程序与私有 Skills 界面分散在不同的 Overlay 中，没有形成统一的企业级扩展中心。 | **P2 架构割裂** |

---

## 3. 统一架构与职责归属（Architecture & Canonical Owner Location）

### 3.1 统一生态与扩展中心架构拓扑

```
 ┌─────────────────────────────────────────────────────────────────────────────────────────────┐
 │                         Fabushi Unified Extension & Security System                         │
 └──────────────────────────────────────────────┬──────────────────────────────────────────────┘
                                                │
         ┌──────────────────────────────────────┼──────────────────────────────────────┐
         ▼                                      ▼                                      ▼
 ┌─────────────────────────┐        ┌─────────────────────────┐        ┌─────────────────────────┐
 │   ContextInspector      │        │    Marketplace Center   │        │   Global Settings       │
 │   - Channels Tab        │        │    - MCP Servers        │        │   - Privacy & 2FA       │
 │   - Agent Skills Tab    │        │    - Plugins Catalog    │        │   - Proxy & Network     │
 │   (右侧渐进披露)        │        │    - Offline Mini Apps  │        │   (全局设置中心)        │
 └───────────┬─────────────┘        └───────────┬─────────────┘        └───────────┬─────────────┘
             │                                  │                                  │
             │ (RPC / IPC Bridge)               │ (IPC Bridge)                     │ (RPC Client)
             ▼                                  ▼                                  ▼
 ┌─────────────────────────┐        ┌─────────────────────────┐        ┌─────────────────────────┐
 │ agentChannelsController │        │  DesktopMcpManager      │        │  CloudSecurityService   │
 │ & ChannelDeliveryRouter │        │  & MiniAppWindowManager │        │  & NetworkProxyStore    │
 └───────────┬─────────────┘        └───────────┬─────────────┘        └───────────┬─────────────┘
             │                                  │                                  │
             ▼                                  ▼                                  ▼
 ┌─────────────────────────┐        ┌─────────────────────────┐        ┌─────────────────────────┐
 │ Native Rust Connectors  │        │ Isolated Sandboxed      │        │ Rust Crypto & System    │
 │ - channel_delivery.rs   │        │ WebContents Container   │        │ - Argon2id / WebAuthn   │
 │ - Telegram / Slack / etc│        │ - fabushi-miniapp://    │        │ - SOCKS5 / MTProto Ping │
 └─────────────────────────┘        └─────────────────────────┘        └─────────────────────────┘
```

### 3.2 Canonical Owners 映射与代码行数预算（200–600 行内聚预算）

遵循“同一责任归属不等于同一文件（Same Owner != Same File）”铁律，所有组件与服务严格控制在 **200–600 行** 之间：

| 模块名称与责任归属 | 文件路径 | 行数预算 | 职责范围与边界 |
|---|---|---|---|
| **渠道管理控制器**<br>(`AgentChannelsController`) | `frontend/src/recovered/features/agent-info/channels/model.ts` | 320 行 | 管理通道配置、连接快照、Token 提交、刷新及状态投影。 |
| **渠道向导与 Dirty 视图**<br>(`AgentInfoChannelsPanel`) | `frontend/src/recovered/features/agent-info/channels/view.tsx` | 260 行 | 渲染渠道列表、连接状态卡片、未保存确认（Dirty Guard）与引导弹窗。 |
| **渠道出入站与安全底座**<br>(`HostChannelDelivery`) | `source/host/src/connectors/channel_delivery.rs` | 550 行 | 负责出站分块（`split_channel_message`）、入站防抖、去重与 ACL 白名单校验。 |
| **MCP 桌面管理器**<br>(`DesktopMcpManager`) | `source/electron-main/mcp/desktop-mcp-manager.ts` | 420 行 | Electron 主进程 Stdio/SSE MCP 进程生命周期、端口与环境变量安全管理。 |
| **插件与市场主视图**<br>(`PluginsBrowser`) | `frontend/src/recovered/features/plugins/overlay/browser.tsx` | 580 行 | 插件市场列表、多轴过滤、MCP 账号绑定、工具权限开关与详细设置。 |
| **Mini App 容器宿主**<br>(`MiniAppWindowManager`) | `source/electron-main/mcp/miniapp-window.ts` | 450 行 | 小程序独立窗口生命周期、CSP 注入、离线代理重定向与协议加载。 |
| **离线小程序管理面板**<br>(`MiniAppManagement`) | `frontend/src/recovered/features/plugins/overlay/miniapp-management.tsx` | 220 行 | 渲染已安装与上架小程序列表、版本更新、验签启动与数据清理。 |
| **云安全与隐私面板**<br>(`TdrpCloudSecurityPanel`) | `frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx` | 380 行 | 真实 2FA 云密码管理、WebAuthn 通行密钥、10 项细粒度隐私策略配置。 |
| **网络代理管理器与测速**<br>(`NetworkProxyStore`) | `source/host/src/extensions/settings/proxy_service.rs` | 340 行 | SOCKS5/HTTP/MTProto 代理存储、健康探测、TCP Ping 与多节点轮换。 |


---

## 4. 极致 UI/UX 与交互设计规范（Unified UI/UX & Interaction Design）

### 4.1 界面布局与渐进披露层级（Progressive Disclosure）

为避免界面拥挤与信息过载，本模块对外部渠道、插件、小程序与安全设置实施三级渐进披露架构：

```
Level 1: 高频直达概览（Overview Card / Indicator）
  ├── ContextInspector -> Channels Tab: 当前连通的渠道图标徽章与连接健康度绿/黄/红点
  ├── Marketplace: 常用/已安装扩展快速开关列表与搜索栏
  └── Settings -> Security: 2FA 与 Passkeys 启用摘要状态卡片

Level 2: 交互式配置与上下文浮层（Interactive Drawer / Modal）
  ├── Channel Setup Wizard: 点击渠道卡片展开抽屉，分步指引引导填入 Token
  ├── MCP Server Detail: 点击服务器展开工具清单（Tools List）、参数配置与授权开关
  └── MiniApp Launcher Modal: 小程序独立沙箱运行视窗，支持全屏与自适应尺寸

Level 3: 高级安全审计与开发者模式（Deep Audit & Raw Config）
  ├── Webhook 签名证书校验与 Inbound ACL 黑白名单原始规则编辑
  ├── Proxy 链路延迟测速仪表盘（Ping/RTT 实时折线图与切换日志）
  └── 2FA 恢复代码打印与 Passkey 物理安全硬件指纹核验
```

### 4.2 全生命周期交互状态矩阵

针对外部渠道握手、MCP 服务连通与 Mini App 加载，定义统一的视觉表现与交互行为：

| 状态类型 | 视觉提示（Visual Indicator） | 操作按钮与可用性 | 辅助文字与错误反馈 | 超时与自愈机制 |
|---|---|---|---|---|
| **Unconfigured / Available** | 灰色轮廓图标，徽标显示“未配置” | 明确显示“连接（Connect）”主按钮 | “连接此代理至外部通讯平台，可在该平台直接与用户互动” | 无超时；用户主动配置驱动 |
| **Connecting / Authenticating** | 旋转蓝色微光 Spinner，状态为“正在连接…” | 所有操作禁用（`disabled={true}`），防止并发重复提交 | “正在向渠道平台验证凭据并建立 Webhook 监听…” | 15 秒硬超时；超时后单调转移至 `Error` 状态 |
| **Connected / Active** | 绿色对勾实心图标，微脉冲绿色圆环 | 变为次级操作：“测试连通性”、“刷新”、“断开连接” | 显示实际绑定的 Bot Username 或 Channel ID，展示最近活跃时间 | 后台每 60 秒心跳探测；异常时静默重试 3 次后告警 |
| **Error / Broken** | 醒目琥珀色感叹号徽标，红色边框高亮 | 显式“重新连接（Reconnect）”与“移除配置” | 详细错误提示：“Token 已被吊销或网络连接超时 (HTTP 401)” | 提供错误日志复制与一键诊断按钮 |
| **Dirty / Unsaved** | 输入框高亮琥珀色呼吸光晕，顶部显示黄色未保存条 | “放弃修改（Discard）”与“保存凭据（Save）” | “你有未保存的凭据修改，离开将丢失当前进度” | 离开路由或关闭抽屉时弹出强制二次确认弹窗 |

### 4.3 响应式断点与无障碍交互规范（WCAG AA & IME Protection）

- **视口响应式断点适配**:
  - **宽屏桌面端（$\ge 1200\text{px}$）**: `ContextInspector` 宽度固定为 380px，渠道向导以内嵌折叠列表展开，同时保持右侧预览；插件市场呈现 3 列自适应网格卡片布局。
  - **中屏适配（$900\text{px} \sim 1199\text{px}$）**: `ContextInspector` 宽度动态收缩至 320px，渠道向导弹窗以居中模态对话框（Modal Dialog）形式弹出；插件市场降级为 2 列卡片。
  - **小屏与折叠屏（$< 900\text{px}$）**: 渠道与插件管理抽屉自动全屏化（100vw），采用抽屉由下至上滑入动画（Bottom Drawer），操作按钮固定在底部安全区。
- **键盘导航与无障碍支持（WCAG 2.1 AA 达标）**:
  - 外部渠道卡片、MCP 切换开关与工具列表项均具备显式 `tabIndex={0}`、`role="button"` 或 `role="switch"`，完整支持 Tab、Shift+Tab 与空格/回车操作。
  - 弹窗（如 `ChannelGuideDialog`、`MiniAppWindow`）在打开时自动将焦点锁定（Focus Trap）在主操作区，按下 `Escape` 键立即平滑关闭，并自动将焦点归还给触发按钮（Focus Return via `restoreRef`）。
  - 输入凭据与 Token 的敏感密码框严格设置 `aria-label="Sensitive Credential Token"`、`spellCheck={false}` 与 `autoComplete="off"`，屏幕阅读器准确朗读当前字段的安全用途与加密存储保障。
- **输入法（IME）保护机制**:
  在渠道自定义名称配置、MCP 指令输入或 Mini App 内置搜索中，严格监听 `compositionstart` 与 `compositionend` 事件，确保拼音输入期间按回车不触发误提交。

### 4.4 `--fabushi-*` 语义设计变量绑定

严格禁止在组件中直接写入 `#FFFFFF`、`#10A37F` 等硬编码颜色，所有样式必须且仅能绑定以下语义 Token：

```css
/* 外部渠道与扩展中心语义样式绑定 */
.sand-channels-tab {
  background-color: var(--fabushi-surface-card);
  color: var(--fabushi-text-primary);
  border-radius: var(--fabushi-radius-lg);
  padding: var(--fabushi-spacing-md);
}

.sand-channel-row {
  border: 1px solid var(--fabushi-border-subtle);
  background-color: var(--fabushi-surface-base);
  transition: border-color var(--fabushi-transition-fast), box-shadow var(--fabushi-transition-fast);
}

.sand-channel-row:hover {
  border-color: var(--fabushi-border-hover);
  box-shadow: var(--fabushi-shadow-sm);
}

.sand-channel-row[data-status="connected"] {
  border-left: 4px solid var(--fabushi-status-success);
}

.sand-channel-row[data-status="error"] {
  border-left: 4px solid var(--fabushi-status-danger);
}

.sand-channel-row[data-status="connecting"] {
  border-left: 4px solid var(--fabushi-status-warning);
}

.sand-channel-dirty-banner {
  background-color: var(--fabushi-status-warning-subtle);
  color: var(--fabushi-status-warning-text);
  border-bottom: 1px solid var(--fabushi-status-warning);
}
```

---

## 5. 精确接口契约与状态机流转（Exact TypeScript/Rust Contracts & State Machines）

### 5.1 前端权威 TypeScript 接口契约

```typescript
// 1. 外部渠道管理契约 (frontend/src/recovered/features/agent-info/channels/model.ts)
export type ChannelPlatformId = "telegram" | "discord" | "slack" | "whatsapp" | "feishu" | "custom_webhook";

export type ChannelLifecycleState = "available" | "connecting" | "connected" | "error" | "coming-soon";

export interface ChannelSetupStep {
  readonly text: string;
  readonly code?: string;
  readonly externalUrl?: string;
}

export interface ChannelManifest {
  readonly platform: ChannelPlatformId;
  readonly displayName: string;
  readonly blurb: string;
  readonly credentialLabel: string;
  readonly availability: "available" | "coming-soon";
  readonly connectGuide: string;
  readonly setupGuide?: { readonly steps: readonly ChannelSetupStep[] };
  readonly tokenRegexValidator?: string;
}

export interface ChannelConnectionRecord {
  readonly platform: ChannelPlatformId;
  readonly label: string;
  readonly status: ChannelLifecycleState;
  readonly detail: string | null;
  readonly connectedAtMs?: number;
  readonly lastActiveAtMs?: number;
  readonly peerIdentity?: string;
}

export interface ChannelsViewSnapshot {
  readonly agentId: string;
  readonly manifests: readonly ChannelManifest[];
  readonly connections: readonly ChannelConnectionRecord[];
  readonly isDirty: boolean;
  readonly pendingTokenDraft?: string;
}

// 2. MCP 与扩展注册契约 (frontend/src/recovered/contracts/desktop-bridge.ts)
export type McpTransportKind = "stdio" | "sse" | "http";

export interface McpServerConfiguration {
  readonly id: string;
  readonly serverIdentifier: string;
  readonly displayName: string;
  readonly transport: McpTransportKind;
  readonly command?: string;
  readonly args?: readonly string[];
  readonly url?: string;
  readonly env?: Readonly<Record<string, string>>;
  readonly isRequired: boolean;
  readonly status: "connected" | "needsAuth" | "initializing" | "error" | "disconnected" | "disabledByPolicy";
  readonly toolCount: number;
  readonly disabledToolNames: readonly string[];
}

export interface McpToolDescriptor {
  readonly name: string;
  readonly displayName?: string;
  readonly description?: string;
  readonly inputSchema: Readonly<Record<string, unknown>>;
  readonly isEnabled: boolean;
  readonly requiresApproval: boolean;
}

// 3. Mini App 可安装小程序契约 (source/shared/miniapp-tool-contract.ts)
export interface MiniAppVerifiedPackage {
  readonly appId: string;
  readonly displayName: string;
  readonly publisherId: string;
  readonly version: string;
  readonly packageDigest: string; // sha256:hex
  readonly sizeBytes: number;
  readonly offlineCore: boolean;
  readonly allowedHostCapabilities: readonly string[];
  readonly contentSecurityPolicy: string;
}

export interface MiniAppLaunchSession {
  readonly launchSessionId: string;
  readonly appId: string;
  readonly conversationId: string;
  readonly accountScope: string;
  readonly accountGeneration: number;
}

// 4. 云端安全与代理契约 (frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx)
export interface Cloud2FAPasswordState {
  readonly hasPassword: boolean;
  readonly hint: string | null;
  readonly unconfirmedPattern: string | null;
  readonly pendingResetDate: number | null;
  readonly srpB?: string;
  readonly srpSalt?: string;
}

export interface ProxyNodeEndpoint {
  readonly id: string;
  readonly type: "socks5" | "http" | "mtproto";
  readonly host: string;
  readonly port: number;
  readonly user?: string;
  readonly password?: string;
  readonly secret?: string; // MTProto proxy secret
  readonly pingLatencyMs?: number | null;
  readonly isOnline?: boolean;
}
```

### 5.2 后端 Rust 核心 Structs 与 RPC 消息定义

```rust
// 1. 外部渠道投递与路由网关 (source/host/src/connectors/channel_delivery.rs)
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChannelInboundMessage {
    pub channel_id: String,
    pub message_id: String,
    pub sender_id: String,
    pub sender_name: Option<String>,
    pub text: String,
    pub attachments: Vec<ChannelAttachmentRef>,
    pub cursor_seq: u64,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChannelOutboundPayload {
    Text { text: String, operation_id: Option<String> },
    Media { url: String, caption: Option<String>, mime: String, operation_id: Option<String> },
}

pub trait ChannelDeliveryProvider: Send + Sync {
    fn deliver(&self, agent_id: &str, recipient_token: &str, payload: &ChannelOutboundPayload) -> Result<String, ChannelDeliveryError>;
    fn check_health(&self) -> Result<ChannelHealthStatus, ChannelDeliveryError>;
}

// 2. 小程序受控校验与安全授权 (source/host/src/extensions/mcp/installable_package.rs)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MiniAppPackageManifest {
    pub schema_version: u32,
    pub app_id: String,
    pub publisher_id: String,
    pub version: String,
    pub entrypoint: String,
    pub resources_sha256: HashMap<String, String>,
    pub requested_permissions: Vec<String>,
    pub csp_header: String,
}

pub struct PackageSignatureVerifier {
    pub trusted_publisher_keys: HashMap<String, [u8; 32]>,
}

impl PackageSignatureVerifier {
    pub fn verify_package(&self, archive_bytes: &[u8], signature_bytes: &[u8; 64], publisher_id: &str) -> Result<MiniAppPackageManifest, PackageError> {
        let Some(pub_key) = self.trusted_publisher_keys.get(publisher_id) else {
            return Err(PackageError::Authorization);
        };
        // Ed25519 物理验签
        let verifying_key = VerifyingKey::from_bytes(pub_key).map_err(|_| PackageError::Authorization)?;
        let signature = Signature::from_bytes(signature_bytes);
        verifying_key.verify_strict(archive_bytes, &signature).map_err(|_| PackageError::Signature)?;
        // 解压并核验资源哈希...
        Ok(manifest)
    }
}

// 3. 真实密码学 2FA 与 PBKDF2/Argon2id 密钥派生 (source/host/src/extensions/auth/cloud_security.rs)
pub struct CloudSecurityPasswordEngine;

impl CloudSecurityPasswordEngine {
    pub fn derive_password_verifier(password: &str, salt: &[u8]) -> [u8; 64] {
        use ring::pbkdf2;
        use std::num::NonZeroU32;
        let mut verifier = [0u8; 64];
        let iterations = NonZeroU32::new(100_000).unwrap();
        pbkdf2::derive(pbkdf2::PBKDF2_HMAC_SHA512, iterations, salt, password.as_bytes(), &mut verifier);
        verifier
    }
}
```

### 5.3 核心状态转移方程

#### 1. 外部渠道入站防抖与去重单调转移
$$\text{InboundGate}(c, m, t) = \begin{cases}
\text{Reject(Duplicate)}, & \text{if } (c, m) \in \text{SeenMessages} \\
\text{Reject(StaleCursor)}, & \text{if } \text{Seq}(m) \le \text{LastCursor}(c) \\
\text{Pass(Immediate)}, & \text{if } \text{IsControl}(m) \lor \text{IsMedia}(m) \\
\text{Debounce}(\Delta t), & \text{if } t - \text{LastAdmitted}(c) < \text{Delay}(c) \\
\text{Pass(Admitted)}, & \text{otherwise}
\end{cases}$$

#### 2. Mini App 启动验证状态单调机
$$\text{LaunchFlow}: \text{ArchiveLoaded} \xrightarrow{\text{Ed25519 Verify}} \text{IntegrityChecked} \xrightarrow{\text{Perms Granted}} \text{SandboxedMounted} \xrightarrow{\text{Session Fenced}} \text{Active}$$
任何一步核验失败立即进入 $\text{Quarantined}$ 终态，并销毁临时解包资源。

### 5.4 跨账号与跨会话隔离围栏（Fencing Rules）

1. **`accountScopeGeneration` 递增原则**:
   切换账号时，全局递增 `accountScopeGeneration`。
   - 所有活跃的 MCP 管道与 Stdio 子进程发送 `SIGTERM`，并在 2 秒后强制 `SIGKILL`；
   - 所有已开启的 Mini App `BrowserWindow` 实例执行 `webContents.destroy()`；
   - 渠道未决的轮询请求与 Webhook 转发立即断开，丢弃全部旧会话缓冲区数据。
2. **Mini App 数据沙箱物理隔离**:
   Mini App 的存储路径严格限定为：
   `~/.fabushi/sandboxes/${accountScope}/${appId}/`
   禁止通过软链接（Symlink）或 `..` 相对路径逃逸至主程序配置文件或用户主目录。


---

## 6. 子智能体实施切片与文件写入边界（Subagent Work Slice & Write Boundaries）

### 6.1 Worker 修改白名单（Exclusive Write Whitelist）

为了保障实施过程中的高内聚与零冲突，落地此蓝图的子智能体（Worker Subagents）**仅允许修改以下白名单文件**：

```
[UI 层 - 渠道向导、插件市场与云安全]
- frontend/src/recovered/features/agent-info/channels/model.ts
- frontend/src/recovered/features/agent-info/channels/view.tsx
- frontend/src/recovered/features/agent-info/channels/view.css
- frontend/src/recovered/features/plugins/overlay/browser.tsx
- frontend/src/recovered/features/plugins/overlay/model.ts
- frontend/src/recovered/features/plugins/overlay/miniapp-management.tsx
- frontend/src/production/tdrp/TdrpCloudSecurityPanel.tsx

[主进程与适配层 - MCP 与 Mini App 容器]
- source/electron-main/mcp/desktop-mcp-manager.ts
- source/electron-main/mcp/miniapp-window.ts
- source/electron-main/mcp/miniapp-management.ts
- source/shared/miniapp-tool-contract.ts

[Native / Host 核心层 - 渠道路由、验签与安全密码学]
- source/host/src/connectors/channel_delivery.rs
- source/host/src/extensions/mcp/installable_package.rs
- source/host/src/extensions/mcp/plugin_skills.rs
- source/host/src/extensions/mcp/plugin_skills_cache.rs
- source/host/src/extensions/auth/cloud_security.rs (新增真实 2FA 密码学引擎)
- source/host/src/extensions/settings/proxy_service.rs (新增代理测速与健康检查)
```

### 6.2 只读参考代码文件清单（Read-Only Reference List）

子智能体可引用并阅读以下文件，但**严禁写入或修改**：
- `projects/openclaw-rust/module-map.md`
- `projects/telegram-desktop-rust/module-map.md`
- `docs/specs/fusion-blueprints/00-master-capability-and-module-index.md`
- `frontend/src/recovered/contracts/desktop-bridge.ts`
- `frontend/src/production/ProductionRenderer.tsx`（仅主装配者可进行窄接线）

### 6.3 与 `ProductionRenderer.tsx` 窄接线契约（Narrow Wiring Contract）

`ProductionRenderer.tsx` 作为全系统的单一装配根入口，严禁在其内部直接编写具体的渠道握手、MCP 进程派生或 2FA 校验逻辑。所有子模块通过受控 Props 进行声明式注入：

```typescript
// ProductionRenderer 挂载扩展与安全子系统的窄接线 Props
export interface ProductionExtensionProps {
  // 外部渠道控制器
  readonly agentChannelsController?: AgentInfoChannelsController;
  // 插件与 MCP 桥接器
  readonly marketplaceBridge?: Pick<DesktopBridge, "mcp" | "marketplaceMiniApps">;
  // 云安全与隐私协调客户端
  readonly coordinatorClient?: ProductionCoordinatorClient | null;
  // 网络代理配置与事件订阅
  readonly networkProxySettings?: ProductNetworkProxySettings;
  readonly onNetworkProxyMutate?: (mutation: ProductNetworkProxyMutation) => Promise<void>;
  // Mini App 启动回调
  readonly onLaunchMiniApp?: (appId: string) => Promise<void>;
}
```

### 6.4 实施分工切片（Slices）

- **切片 1（外部渠道向导与 Dirty Guard 完善）**:
  在 `channels/view.tsx` 中补齐 `isDirty` 追踪与 `ChannelGuideDialog` 焦点捕获；在 `channel_delivery.rs` 中强化入站防抖与 ACL 匹配。
- **切片 2（MCP 统一纳管与插件中心对齐）**:
  重构 `PluginsBrowser` 整合 MCP Server 账号列表、工具启用开关与企业策略标签；在 Electron 主进程中完善 Stdio MCP 进程环境变量隔离。
- **切片 3（Mini App 安全沙箱与 Ed25519 验证闭环）**:
  在 `miniapp-window.ts` 中实现严格的 `setWindowOpenHandler` fail-closed 拦截，配置离线 `127.0.0.1:9` 代理；在 `installable_package.rs` 中打通数字签名验证。
- **切片 4（云安全真实密码学与代理测速）**:
  移除 `TdrpCloudSecurityPanel.tsx` 中的伪造 `btoa` 哈希，接线真实 Host PBKDF2/Argon2id 接口；在 `proxy_service.rs` 中补齐 TCP Ping 延迟测量与前端展示。

---

## 7. 防作弊审计与验收标准（Anti-Cheating Oracle & Acceptance Gates）

### 7.1 负例作弊排查表（Forbidden Anti-Patterns Checklist）

独立审计子智能体（Auditor Subagent）将对落地代码执行无死角排查，发现以下任意一项即判定交付不合格：

- [ ] **严禁前端伪造密码学 Hash/Salt**:
  代码中严禁出现类似 `const hash = "hash_" + btoa(password)` 或 `const salt = "salt_" + Date.now()` 的伪造 2FA 云密码逻辑。密码学派生必须由 Native Rust 底座或受信任密码学 API 完成。
- [ ] **严禁硬编码虚假连接就绪状态**:
  严禁硬编码“18 个连接器全量就绪”或静态 mock 状态数组。外部渠道必须真实反映 Host 探测到的 `connected`、`connecting`、`error` 或 `available` 真实状态。
- [ ] **严禁凭据输入丢失 Dirty Guard 保护**:
  在外部渠道输入 Token 或修改代理配置时，如果未保存且尝试离开，必须触发拦截，严禁未提示直接丢弃用户输入。
- [ ] **严禁 Mini App 容器存在外部窗口逃逸风险**:
  小程序窗口必须显式声明 `setWindowOpenHandler(() => ({ action: "deny" }))`，严禁允许不可信页面未经审批弹出外部未经沙箱保护的系统窗口。
- [ ] **严禁未验证签名的离线小程序包启动**:
  离线小程序在装载 `fabushi-miniapp://` 之前，必须经过 Ed25519 签名与 SHA256 资源哈希校验，严禁允许直接从本地非受信任路径加载未经验签的代码。
- [ ] **严禁使用任意 `any` 逃避类型检查**:
  所有 TypeScript 接口与 Props 必须保持严密类型定义，禁止在契约和 Controller 中使用 `any` 破坏类型安全。

### 7.2 正例验收确认准则（Positive Verification Gates）

1. **结构完备性验收**:
   - `docs/specs/fusion-blueprints/07-channels-plugins-mcp-and-miniapps.md` 包含全部 7 个强制章节，无占位符、无缺失。
   - 完整覆盖 OpenClaw 5 大核心责任域（`OCRM-M14` 至 `OCRM-M18`）与 Telegram Desktop 4 大核心能力域（`TCG-N24`, `TCG-N25`, `TCG-N27`, `TCG-N37`）。
2. **源码引用真实性验收**:
   - 引用的所有源码文件路径（如 `source/host/src/connectors/channel_delivery.rs`、`frontend/src/recovered/features/agent-info/channels/model.ts`、`source/electron-main/mcp/miniapp-window.ts` 等）均在仓库中物理存在，行号对应准确。
   - 函数与结构体名称（如 `split_channel_message`、`InboundChannelDeduplicator`、`ChannelAclGate`、`NativeMiniAppEntry`、`TdrpPrivacyRuleSelection` 等）均真实对应代码实现。
3. **架构与规范合规验收**:
   - 代码行数控制在 200–600 行内聚预算之内。
   - 样式统一绑定 `--fabushi-*` 语义设计变量，支持浅色/深色主题。
   - 键盘无障碍焦点回退（Focus Return）与 IME 保护完整。
4. **无命令执行与无作弊纪律**:
   - 整个蓝图编制与审计过程严格遵循无命令验证模式（No Local Execution），未调用任何 `run_command` 进行本地构建或测试绕过。

---

> **蓝图编制结论**:  
> `07-channels-plugins-mcp-and-miniapps.md` 已全面完成编制。外部渠道接入向导与 Dirty Guard、插件 SDK 与 MCP Registry、Skills 系统、离线优先 Mini App 容器、以及 Telegram 云安全与代理网络已被深度融入 Fabushi 既有架构，为后续 Worker 落地提供了无缝、零割裂、高内聚的权威指导契约。
