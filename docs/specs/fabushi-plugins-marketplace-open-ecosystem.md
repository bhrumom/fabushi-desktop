# Fabushi Plugins / Marketplace — 开源成熟生态架构规范（FPM-001）

Status: **active, design/implementation pending**  
Owner: Fabushi canonical Plugins/MCP/Marketplace product domain  
Last updated: 2026-10-11  
Canonical repo: bhrumom/fabushi-desktop  
Related: docs/specs/grok-bot-018-runtime-product-parity-recovery.md; docs/specs/offline-first-installable-mini-app-platform.md (OMA-001); projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md  
Authority snapshot refreshed 2026-10-11: canonical Desktop main **d575b4f77b6895eea4ecaced96e5c512964bf3ed**; Marketplace PR #49 exact head **72abccaa377fa026e4ebec113d76a1ad31ceabbf** is unmerged and diverged from current main, so its old Actions are observation-only. Backend main **b55320f655584b999c08ca01ebff182fd04328d9** and Platform Core main **3a125c4d3164db7119b8022f16062a502d54d28a** were also re-read. See `docs/specs/fabushi-plugins-marketplace-owner-inventory.md` for the current source-to-owner inventory.

## 1. 背景与问题

Fabushi 已拥有「Plugins」入口、Connector/MCP 服务管理、Skills、账号/OAuth、团队策略及部分移动端 Mini App 集成，但**这不等于一个独立、可持续运营且经过供给侧/安全/安装/升级验证的插件生态市场**。近期产品要求新增离线 HTML/JS/Wasm 小程序，以及在手机上由 AI 生成并私有安装小程序（OMA-001）。因此需要统一「发现—信任—安装—授权—运行—更新—禁用—卸载—反馈—发布」，而不是把新能力叠在旧市场列表的特殊分支上。

参考成熟开源生态的**架构与产品行为**，但不机械复制插件运行时或直接移植不兼容的二进制插件：VS Code 客户端成熟的商店 UX 和扩展 manifest，Eclipse Open VSX 的开放注册中心、发布者命名空间、签名/扫描和自托管治理，Eclipse Theia 的扩展边界，以及 Fabushi 自己的 Host/Coordinator/MCP/WebView/AI 能力。

**关键事实**：VS Code 的扩展客户端（Code - OSS）是开源的，但微软 Visual Studio Marketplace **服务端并非开源，也不能默认被 Fabushi 当作自由复用的市场后端**。真正可研究、自建的注册中心首选参考 **Eclipse Open VSX（EPL-2.0）**。Open VSX 的基础包格式是 VSIX / VS Code 扩展，不等于可以把其 Node 扩展直接装入 iOS/Android 或把 Fabushi 所有扩展强制包装成 VSIX。

## 2. 本轮 main 源码事实（截至 authority snapshot）

| 层级 | 真实现有路径 / 行为 | 必须保留与不足 |
| --- | --- | --- |
| Desktop shipping | frontend/src/production/ProductionRenderer.tsx 懒加载 frontend/src/recovered/features/plugins/overlay/desktop-surface.tsx，当前以 **dialog / overlay** 挂载 | 现有功能可用入口必须保留；FBCP 正式产品信息架构已要求「插件市场」成为一级入口，**源码当前形态尚未实现该目标** |
| Desktop UI | frontend/src/recovered/features/plugins/overlay/browser.tsx：Marketplace / Yours 两页签、搜索、Connector/Skill + Team/Public 过滤、插件详情 Add/Remove、配置变量、MCP server 账号操作、单 tool enable/disable、Private Skills | 已有成熟 MCP 安装管理流程；缺少经源码证实的独立商店详情元信息、真实评分/安装量、发布者验证证据、内容审核、版本页、兼容性、预发布/回滚、推荐/报告系统，不可在无后端事实时虚构 |
| Desktop adapter | frontend/src/recovered/features/plugins/overlay/desktop.ts：并行读取 mcp.catalog/effectivePlugins/list，安装时调用 mcp.install；source/electron-main/mcp/mcp-desktop.ts 提供 sand:mcp-* IPC | 复用 typed bridge + 生命周期；不是本地 .fabapp 校验安装器 |
| Catalog authority | source/shared/node/mcp/mcp-marketplace.ts 通过 DashboardService.listMarketplacePlugins 获得 Catalog 与私有市场数据；source/shared/node/mcp/mcp-catalog-flow.ts 30 秒内存缓存、字母排序、install/update 请求经现有 account writer；source/shared/node/marketplace/cursor-marketplace-client.ts 保留 Cursor-compatible DashboardService 协议/元信息 | **现有 adapter 是兼容服务协议，不是已证明为 Fabushi 独立自有的开源注册中心**；不能据此断言生产网络最终由哪个公司运营；要补 Fabushi-owned publisher/catalog/trust authority |
| Desktop compatibility leak | frontend/src/recovered/features/plugins/overlay/desktop.ts 内 private skill Copy Link 指向 cursor.com/marketplace | 新 Fabushi 市场必须有自己的 canonical URI；迁移时保留必要兼容 provenance，不应持续将 Fabushi 用户导往他牌市场 |
| iOS | fabushi-ios/fabushi-ios: mobile/ios/Fabushi/MarketplaceModel.swift 用 feature.marketplace.browse/release、feature.plugin.install、feature.marketplace.add，已有 permissions 与启动；MiniAppWebMcpSurface.swift 使用 WKWebView | 是**另一条 native Host 市场接口路径**；存在平台差异，尚不能由此证明与 Desktop 数据/版本/权限/用户安装真相完全一致；OMA-001 离线包仍待实现 |
| Android | fabu-android/fabushi-android: mobile/android/app/src/main/java/com/ombhrum/fabushi/MarketplaceViewModel.kt 具类似 feature.marketplace.browse/release、feature.plugin.install 和权限流程；有 MiniAppWebMcpSurface.kt | 平台适配已存在，但本 Spec 不将相关能力视为经过独立 exact-head release 验证 |
| Existing SDK | frontend/packages/mcp-app-sdk/src/bridge.ts 的 MessagePort、pluginInstanceId、nonce、grants；source/host/src/extensions/mcp/* 与 local_tool_permission/* | 保留现有 Host/Coordinator/WebMCP 安全与执行 owner；**不能使用扩展市场设计为第三方 JS 开放任意原生 API** |
| Source of truth | FBCP SOURCE_OF_TRUTH.md Rev7 要求微信式左竖栏、列表/搜索、主工作区，「消息/联系人/插件市场」一级入口、统一 design tokens/canonical UI | Marketplace 的实现必须进入现有产品 shell，不能添加第二套商店应用或第二套全局设计语言 |

上述事实来自**源码检查**而非用户体验实机验收：没有证据证明现有 Marketplace 已达到 VS Code/Open VSX 的成熟度；旧 source recovered 名称不自动授权其作为永久公共协议。

## 3. 目标与不做的事

### 目标

**一个** Fabushi Plugins Marketplace，兼容 Desktop/iOS/Android、现有账户与企业策略，支持三类产品条目：**Connectors（MCP 连接器）、Skills（声明式工作流/知识技能）、Mini Apps（在线或已验证可离线 Web 小程序）**。AI 创建的个人小程序是 Mini Apps 的 **private-local-author** 安装来源，与公开发布者来源保持不同信任域，统一出现在「我的小程序」。

向用户提供 VS Code 式的可发现、可验证、可管理体验，向开发者提供 Open VSX 式开放可审计的发布、命名空间、版本、安全检测、审查、密钥轮换和分发体系；对管理员提供团队私有市场及允许/禁止/强制安装策略。保留 Fabushi 原有 Agent/Host/Runner/MCP 和平台边界。

### 明确不做

- 不引入第二个 Marketplace、MiniAppCore、AgentRuntime、CommunicationCore、通用 Node 插件 Host 或新的 shared Fabushi runtime 仓库。
- 不直接宣称 Fabushi 支持所有 VS Code 扩展，不把 VSIX 当作可移植到 iOS 的包，也不下载/执行任意 DEX/JAR/so/原生二进制。
- 不把 VS Code 扩展的广泛宿主权限复制到 Fabushi：官方 VS Code 文档明确说明传统扩展可拥有与 VS Code 相同的权限；这不是移动端安全模型。
- 不把微软 Visual Studio Marketplace 专有服务端代码/数据/商标当开源资产；不得无核查克隆/代理其商店或扩展包。
- 不未经许可证和兼容性核对就 fork/嵌入 Open VSX，或复制其前端布局作为 Fabushi 唯一设计语言。
- 不借本 Spec 改写当前 PR #42/TDRP/FBCP 既有验收门、解除单一 owner 规则，或宣称未完成的 iOS App Review 许可已通过。

## 4. 开源依据与明确借鉴

| 参考 | 成熟做法 | Fabushi 采用方式 | 不采用内容 |
| --- | --- | --- | --- |
| **VS Code extension UX / manifest** | 单一扩展页；搜索、类别与排序；详情、README、Changelog、依赖、发布者、评分、版本、安装/禁用/卸载/更新、配置、推荐；publisher.name + SemVer + engines 兼容声明 | 借鉴用户旅程、状态模型、版本与兼容性信号；设计成 Fabushi typed extension manifest | VS Code 扩展 JS/Node 运行权限、VSIX=Fabushi 包格式假设 |
| **Eclipse Open VSX Registry** | EPL-2.0 开源 registry/portal/CLI；publisher namespace + ownership；发行版本；自托管；发布扫描（凭据、已知恶意哈希、typosquatting）；registry API | 为 Fabushi 注册中心的发布、审查、索引、签名分发、命名空间和治理建立参考合同；评估可复用开源组件与许可证 | 直接把仅面向 VS Code 的 Gallery/VSIX schema 当作 Connectors/Skills/Mini Apps 唯一格式 |
| **Eclipse Theia** | 用户可安装 extension 与编译期产品扩展分开；runtime 通过明确 API 和独立 plugin host 交互 | 产品内核与不可信插件分离、版本化 Host 能力、运行生命周期责任明确 | 将 Theia/VS Code extension host 或 Node 扩展执行器直接植入 iOS |
| **Fabushi OMA-001** | 签名 .fabapp、content-addressed 存储、离线启动、每 app origin、权限和原子更新、AI-created 私有信任域 | 把 Mini App 安装能力作为同一个市场的 **一种 typed kind**；SDK 与安装服务不重复 | 不经 Host 校验的 AI preview→安装捷径或发布者权限漂白 |

开源参考有实际漏洞历史，尤其注册中心上传扫描/身份/归档路径。因此即便决定重用 Open VSX 技术，也必须追踪 upstream security advisories 并锁定受支持的修复版本、做 own threat modeling/CI；“经过多年使用”不等于“零风险”。

## 5. Single-composition 产品信息架构（规范）

Fabushi 左一级导航保留已有消息、联系人等产品入口，**Plugins/插件市场成为既有 shell 的一个一级 route**（而非单独窗口/app；Desktop 老 overlay 可迁移中兼容但最终有明确删除/收敛门）。

~~~text
Fabushi Shell（统一品牌/主题/账号）
 ├─ 消息 / 联系人 / Agent / 其他已有一级领域
 └─ 插件市场（一个 canonical route）
     ├─ 发现 Discover        [精选 / 分类 / 搜索 / 推荐 / 最新]
     ├─ 我的已安装 Installed [全部 / 已启用 / 已禁用 / 更新可用]
     ├─ 我的创建 Created     [本地 AI 小程序 / 私有 Skills]
     ├─ 团队 Team            [组织批准 / 必装 / 禁止 / 私有条目]
     └─ 管理/发布 Publish    [发布者/提交/版本/审查记录 — 有权限者可见]
        主列表                         详情
        图标、名字、发布者、类型、     简介、截图、README、Changelog
        评分证据/安装量（若可信）、    发行版本、OS/API兼容、安全/许可
        离线/权限/更新/团队标记        依赖、所需权限、变更/数据说明
                                       安装、允许、禁用、更新、回滚、卸载
~~~

**移动端** 使用相同导航/对象语义但自适应单栏列表→详情；桌面使用侧栏+列表+详情；复用 FBCP 统一 design tokens、组件、搜索/路由、可访问性与中文/英文多语言规则。不得同时出现一套「插件市场」和一套「小程序商店」，导致重复安装真相。

**一级搜索与筛选**：文字/标签/分类/类型、平台兼容、离线、是否已安装、待更新、已授权、企业范围、安全状态、发布者已验证。支持服务端分页与稳定游标，界面保留焦点、滚动和深链；排序默认 **relevance（可解释）**，另支持 recentlyUpdated / popular / rating / alphabetical，但指标未建成前隐藏而不是假造数值。已安装状态从本地 Host ledger 显示，不取代已授权账号投影。冷网中断仍可浏览本地已安装清单与已验证的缓存元数据，明示缓存时间，禁止把缓存说成实时评级。

**插件详情最低字段**：名称、namespace ID、publisher 和验证状态、类型/运行位置、版本/渠道、变更记录、首页/源码/许可证、兼容平台与最低 Host API、所需能力及变更 diff、是否可离线、数据/隐私说明、适用年龄、问题反馈与安全举报入口、安装与更新证据。展示真实签名/供应链证据而非单独“Verified”图标。用户数据、评分反作弊和隐私遵守产品政策。

## 6. 核心类型与责任分配

| 责任 | 唯一 owner | 禁止 |
| --- | --- | --- |
| Marketplace UX、入口、已安装列表、搜索/过滤和详情 | 现有 frontend Plugins/recovered + shipping ProductionRenderer；native iOS/Android Marketplace views | 新建平行商店 shell、凭 web 页面直接修改全局安装状态 |
| Publisher/catalog/source authority、审核/搜索索引/元信息/评论/举报 | Fabushi-owned Plugins registry **服务端子域**，优先吸收到现有 Fabushi backend/service owner；如无符合 owner 才以单独 ADR 最小新建 | 把 Cursor-compatible API 自动标为 Fabushi 自建注册中心；Catalog 客户端缓存充当发行/权限真相 |
| MCP Connector 运行/连接/账号 OAuth/工具列表及 policy | 既有 Electron/Coordinator/Host MCP + Host permission owner | 连接器自带 JS 任意执行，明文 OAuth token 到 renderer |
| Skill 安装/启停/Team 发布/Agent作用域 | 既有 Plugins/Host workflow/Agent source；Skills 默认声明式，权限需额外批准 | 将 Markdown skill 视为操作系统可执行扩展 |
| Mini App 版本化安装/隔离/WebView/更新/恢复 | OMA-001 的 Plugins + Host package install、native Web surface | 复制第二个 .fabapp 管理器，向脚本直接暴露 Host |
| AI 创建本地 mini app | 现有 Agent/Runner 创作→host verifier→local-author trust domain→已安装清单 | AI 自动公开发布、自动授予 OS/高风险工具权限 |
| 安装审计/授权/策略/锁定和数据所有权 | 既有 account policy、Host durable storage、permission owner（按需求最小扩充 typed records） | 用户帐号、团队政策和本地用户数据在多个服务拥有互相冲突的真相 |
| Desktop/iOS/Android packaging | 各自仓库的原生实现；跨平台 **只共享规范、测试向量和公开协议** | 新建共享运行时/core 仓库或跨平台二进制强耦合 |

服务端需要持久化审核/发布/索引与二进制存储，这不同于客户端 Host 安装状态。将服务端 registry 限制在**现有 Plugins 域**，并通过 API/协议对接原有 Host 而不是让市场成为第二业务核心。

## 7. 统一市场元信息模型 / API

**两个正交维度**：kind 决定运行/权限/分发行为；origin/trustDomain 决定发布者证据与租户可见性。类型字段明确、拒绝未知声明，允许未来扩展但不可把 Connector 伪装为离线 Mini App。

~~~ts
type ExtensionKind = "mcp-connector" | "skill" | "web-miniapp";
type OriginKind = "public" | "team" | "private-created";
type TrustDomain = "catalog-authorized" | "team-authorized" | "private-local-author";

type ExtensionIdentity = {
  namespace: string;              // publisher/org namespace; ownership enforced
  name: string;                   // stable slug
  kind: ExtensionKind;
};
type ExtensionRelease = {
  identity: ExtensionIdentity;
  version: string;                // SemVer; immutable release identity
  channel: "stable" | "preview";
  minHostApi: number;
  supportedPlatforms: ("desktop" | "ios" | "android")[];
  supportedOs?: string[];
  bundleDigest?: string;          // for downloadable verified packages
  manifestDigest: string;
  requestedCapabilities: string[];
  dependencies: ExtensionIdentity[];
  license?: string;
  trustDomain: TrustDomain;
  revoked?: boolean;
};
type MarketplaceEntry = {
  identity: ExtensionIdentity;    // stable id, never displayName key
  origin: OriginKind;
  publisherId: string;
  verifiedPublisher: boolean;    // evidence-backed; not inherited
  displayName: string; description: string;
  categories: string[]; tags: string[];
  latestCompatibleRelease?: ExtensionRelease;
  rating?: { average: number; count: number; fraudChecked: boolean };
  installs?: { count: number; period: string; integrityChecked: boolean };
  contentRating?: string;
  sourceUrl?: string; supportUrl?: string; reportUrl?: string;
};
type InstalledRecord = {
  identity: ExtensionIdentity;
  sourceOrigin: OriginKind;
  installedVersion: string;
  releaseDigest: string;
  accountScope: string;
  activationScope?: string;       // user/team/Agent as applicable
  state: "installed" | "disabled" | "updating" | "quarantined";
  effectiveGrantEpoch: number;    // actual effective grants always Host-owned
};
~~~

**Registry API v1（逻辑合同）**：
- browse/search/list public or approved-team entries + stable cursor, cache/ETag;
- get detail, immutable release metadata, supported targets and dependencies;
- publisher onboarding/namespace claim, scoped signing and CI trusted publishing;
- submit → scan → moderation → publish/reject; author edit metadata or revoke release with immutable audit;
- report/appeal/incident action; release revocation and signed trust epoch feed;
- optional ratings/reviews/installs only after fraud controls, privacy and backend ownership are in place.

**Host/native typed actions（复用已有 bridge）：** listInstalled, planInstall, approveGrants, install, enable/disable, update, pinVersion, rollback, uninstall, listPublisherTrust, revokeGrant, listAvailableUpdates。具体现有方法如 mcp.catalog/effectivePlugins/list/install、feature.marketplace.* 必须通过兼容 adapter **收敛到单一产品合同**；不能让同一操作出现两个不一致的真相。工具/连接器 OAuth 的 setup 值与 secrets 不随公开元数据或日志传播。

Release **依赖解析**必须检测循环、兼容冲突、被封禁发布者、权限变化、宿主 API 不兼容，并向用户展示完整 install plan；不可默默启用新依赖或提升权限。

## 8. 发布者与注册中心的安全治理（借鉴 Open VSX，不照搬运行权限）

~~~text
开发者验证 namespace / 控制权
    → 上传 manifest + 对应 kind 的源/资源/构建包
    → strict schema + API/OS compat + digest/signature + license/source policy
    → secrets / malware / bad hash / typosquat / dependency and supply-chain scan
    → moderation / age / content / privacy / payment review (依分发渠道)
    → immutable release + signed metadata + transparency record
    → 搜索索引/CDN 分发（不拥有运行期用户权限）
    → Host 计划安装，展示版本/平台/能力变更
    → 用户/团队同意 + Host 验证包 & trust epoch
    → Connector / Skill / Mini App 各自受限执行
    → 审计 / 举报 / 禁用 / 版本撤销 / 恢复
~~~

- **namespace 唯一性与审核**：publisher ownership + verified state; 防仿冒、高相似名称、账号劫持、签名密钥泄漏；团队 private registry 的 trust root 与 public 分离。
- **不可变发行**：同 (namespace, name, kind, version, channel) 不能替换 bytes；hash、签名、compat/policy metadata 与 artifact 对齐；更新必须可审查能力 diff，原子切换与 last-good 回滚。
- **权限必须在用户/Host 侧重新判定**；市场签名仅证明来源/完整性，**不等于安全性和所有原生/MCP/Agent 权限**。对所有 tool invoke 独立做调用前校验。Publisher 对 manifest 扩权不能自动继承旧授权。
- **扫描 fail closed**，出现超时、扫描器宕机、结果缺失、归档解析差异或 key/registry 分离不一致必须隔离或拒绝发布/安装。安全审核本身可能遭到竞态/缓存投毒/ZIP 炸弹攻击；参照 Open VSX 公开安全公告独立验证修复。
- **紧急下架**：客户端验证已知恶意 digest / publisher revocation；网络不可用时使用签名的最近可信 policy epoch 和离线模式规则；不能因商店服务暂时不可达擅自删除已安装数据。高风险已撤销包在离线缓存 revocation 命中时停止启动。
- **评论评分**：实名或可追责主体、节流/反刷、内容审核、举报、隐私条款；不以安装数/评分作为安全保证。平台限制发布时仍保留本地用户数据与安全退出路径。
- **开源贡献**：Fabushi 自有 typed schema/API、发布 SDK/CLI 与规范可作为开源项目独立评审；若使用 Open VSX 代码/依赖应明确 EPL-2.0 兼容性、NOTICE、SBOM、版本、补丁及公共 API 边界。

## 9. 严格的三种扩展运行边界

| Kind | 交付/安装 | 执行位置 | 运行时权限 |
| --- | --- | --- | --- |
| **Connector/MCP** | 目录+服务能力声明，账号 setup/OAuth；可无本地二进制 | 现有 Host/Coordinator MCP 路由到被批准的 HTTP/stdio server（平台与政策允许时） | per server/per account/per tool；未授权则拒绝；不因目录审核授予 shell |
| **Skill** | 声明式文本、触发条件、可选 source/bundle hashes；与 Agent/workflow 作用域绑定 | 已有 Agent/Host workflow 与执行批准 owner | Skill 文本是非可信指令，不因来自市场成为管理员，调用高风险工具仍单独批准 |
| **Web Mini App** | OMA-001 已验证 .fabapp 完整资源包；可在线或离线；私有 AI 创建的 bundle 由 local-author 签署 | iOS WKWebView / Android WebView / Desktop sandboxed Electron | app-account/session-origin 隔离；默认无网络/无 native /无 privileged MCP；显式 grant 逐次校验；iOS Policy Gate |

**绝不**为移动端复刻 VS Code/Node 扩展 Host 的系统级访问。将 **gallery + artifact registry** 与 **runtime sandbox** 当成独立威胁面，各自验证。宿主原生权限不能通过 iframe/window bridge 直接暴露。

## 10. 安装与更新状态机

~~~text
catalog-only
   → policy/compatibility/dependency plan
   → user consent + Host trust/grants
   → downloading/staging → verifying → installed-disabled/installed-enabled
   → enable/disable, account connect/disconnect, permission revoke
   → updating → verified new version → atomic activate
                 └─ fail → previous good version preserved
   → quarantined (malware/revocation/policy)
   → uninstall (explicit data-retention decision where relevant)
~~~

**服务端发布状态机独立于客户端安装状态机**：draft → submitted → scanning → review/approved → published；reject/quarantine/revoke 有原因、权限、审计。任何 state change 只由 owner 写入；UI 是可重建 projection。

安装/更新日志 account/device/key 绑定，支持 kill/restart/offline/低存储、重复请求 idempotency、被撤销能力、旧签名、token 失效、重定向、用户快速切换账号；绝不通过一个“install succeeded” RPC 就推断插件工具、Skill 同步、app 启动三者都成功。**评分、热门、已验证**必须有独立来源。

## 11. 同步、企业策略、隐私与平台

- **User / team / system / private-created** 四类安装来源在 UI 中区分，但一个 canonical installed record owner；与 server account slots、多账号 OAuth、Team required/disabled 保持兼容。
- Settings Sync 可同步 **插件清单、已选渠道、显式策略**，而非 OAuth secrets、设备签名私钥、mini app 用户数据、私有 AI 草稿、个人授权；任何跨设备数据迁移须单独授权。
- 企业策略按 kind/publisher/version/trustDomain/platform 设置 allow/deny/required；禁止商店评分、普通发布者声明或 AI 提示绕过 org policy；审计保留谁、何时、用哪版做了什么。
- **iOS** 遵循 App Store Review §§2.5.2 / 4.7 / 4.7.1–4.7.5；可安装脚本内容和暴露原生 API 的许可仍逐条门控；禁止把桌面 VSIX/Node 安装能力间接放到 iOS。
- **Android** 遵循 Google Play 下载代码/解释器例外的适用范围，不以 WebView 可运行 JS 自动推断任何任意动态插件都合法。
- 适配不同平台可用的扩展 kind/feature flag；详情展示明确 **Not supported on iOS/this host**，不要假按钮、fallback 安装或权限降级隐藏失败。

## 12. 与历史源兼容、渐进迁移及技术选型

**后端技术选型结论（2026-10-11 source audit 后定稿）：选择方案 B，并允许方案 C 的窄范围组件复用。** Open VSX 继续作为 marketplace registry 的第一开源 benchmark，但不部署成第二套 Fabushi 产品权威或通用插件运行时。当前 `bhrumom/fabushi-backend` 已有 `ai-backend/src/miniapp_marketplace*.js` 的严格 manifest、发行 artifact/digest、搜索、publisher draft/submit/review/yank、account install projection 与 MCP 路由，因此 canonical typed Registry 必须优先扩展这个 existing owner，统一 `mcp-connector` / `skill` / `web-miniapp`；Open VSX 的 namespace、发布、安全扫描、撤销等成熟做法可在逐项 license/provenance/security/API-fit 核查后选择性吸收。只有 ADR 用 current source 证明 existing owner 无法承担某项责任时，才能引入额外最小服务组件；禁止为“用了开源项目”制造 VSIX 包装或双后端。

**迁移路径**：
1. 保持现有 Desktop mcp.catalog/install/list、有效 Team/Private/OAuth/tool toggle 功能不回退；给 Cursor-compatible 数据明确 legacy source 标记，不自动授予 Fabushi publisher verified。
2. 建立可追溯的原有 pluginId/serverId/workflowId → stable namespace/kind/identity 映射；用户授予的 OAuth 和工具权限不得丢失或悄悄提升。
3. 先让新的 Fabushi-owned registry 提供只读目录/详情，审计与数据结构一致之后逐段改默认查询；逐步增加 publish/signature/update/report；旧 provider 仅作为明确兼容 source，设退休门。
4. 将 Desktop 插件入口由旧 dialog **收敛为 FBCP 指定一级 route**，不能丢掉现有账号、Private Skills、Team policy、OAuth 与单 tool 开关；iOS/Android 同步 typed navigation/visual semantics。
5. 实现离线 .fabapp + AI 私有包时复用 OMA-001 校验、状态机与信任域，不另写第二套安装器；承认 iOS 商店审核限制。
6. 旧 cursor.com/marketplace 用户可见深链在 Fabushi-owned publisher/marketplace route 正式上线后替换；临时保留必须显式标识外部来源，禁止伪称 Fabushi 官方条目。
7. 验证完所有兼容与 release 门后才移除废弃 legacy UI/contract；回滚必须可恢复最后已验证本地状态。

## 13. 功能/安全/可用性要求与验收编号

| ID | 规范性要求 / 验收 |
| --- | --- |
| **FPM-R01** | 插件市场是 existing Fabushi shell 的一个 canonical 一级 route，同一 UI/搜索/设置/账号所有者；手机和桌面统一信息架构，不创建第二商店。 |
| **FPM-R02** | 按 kind 区分 Connector、Skill、Mini App；按 origin/trustDomain 区分 Public、Team、Private-created；不得混淆运行、发布和签名权限。 |
| **FPM-R03** | Browse/search 可分页、筛选、排序，结果支持已安装/平台/API/离线/更新/企业状态；搜索元信息可见来源与缓存时间。 |
| **FPM-R04** | 详情包含 publisher verification evidence、license/source、版本/changelog、API/OS 兼容、请求权限、安装计划、举报及撤销状态；未知证据不可显示安全徽章。 |
| **FPM-R05** | Fabushi-owned publisher namespace、key ownership、immutable release digest、scan/review/moderation、信任与可审核变更记录均有独立权威 owner。 |
| **FPM-R06** | 安装/更新/回滚/撤销/卸载由 Host/registry typed contract 和本地 durable ledger 协作，网络和 crash 不导致半安装或丢数据。 |
| **FPM-R07** | 保留 MCP OAuth、multi-account、per-tool enable/disable、Team policy、Private Skills，且不得在迁移中降级。 |
| **FPM-R08** | 下载代码默认沙箱与权限零授权；移动端不加载 Node/VSIX/本地动态原生插件；AI-created local-author 不获得 public publisher 或系统权限。 |
| **FPM-R09** | ratings/download counts/recommendations 只有在有可审计后端、反作弊和隐私保障时展示；上线前不得假造。 |
| **FPM-R10** | Open VSX/VS Code/Theia 源码/实现复用需先完成 license/provenance/API fit、安全漏洞风险和版本锁定评估；不照搬微软专有市场服务。 |
| **FPM-R11** | 版本/平台/渠道适配与 iOS/Google Play 发行政策单独记录；blocked≠passed，beta/preview 不自动变成 stable。 |
| **FPM-R12** | 全部可执行验证只使用 GitHub Actions exact-HEAD；Desktop packaged + iOS/Android device/market-policy acceptance 是 release 门。 |

**Definition of Done / ACs（任何一种 Preview 只能声明该平台已验证范围）**：
- **FPM-AC01 UX**：对真实 signed-in Desktop/iOS/Android，在新一级入口完成公开搜索、详情、安装/禁用/卸载、团队视图、Private Skills、离线已安装清单；暗/亮/移动响应、中文/英文、键盘/a11y、缩放/Reduced Motion 通过。
- **FPM-AC02 MCP parity**：OAuth 连接/断开/失败恢复、多账户切换、单工具 toggle、团队必装/禁止、private skill 发布及取消、跨账号切换均成功。
- **FPM-AC03 Registry integrity**：namespace 冒名、typosquat、泄漏 secret、恶意 hash、ZIP 路径攻击、扫描超时/并发越过、发行 digest 替换、供应链篡改、撤销/离线 policy 测试均 fail closed。
- **FPM-AC04 lifecycle**：同一版本重复安装、升级中断、宿主 crash/reconnect、kill after journal、低空间、data rollback、不同版本运行中升级正确；无双重 installer truth。
- **FPM-AC05 OMA integration**：.fabapp 与 AI-generated private apps 走相同 Marketplace display/lifecycle，实际设备断网冷启动且没有未经批准的 native/MCP 权限；iOS 合规门独立通过。
- **FPM-AC06 governance**：registry/public+team/private 发布者、版本、签名、公开/私有政策、撤销、审核/举报，安装可审计并有真实可验证证据。
- **FPM-AC07 source & license**：明确 Open VSX upstream commit/license/patch/advisories 或“只借鉴不复用”决策，禁止源码/license 混淆。
- **FPM-AC08 exact-head release**：GitHub Actions 源码 SHA / run/job/测试数/产物哈希，正式安装产物签名、真实 UI/video/device、独立 reviewer 验收，所有适用 AC 通过方能宣布 release-complete。

## 14. CI 取证矩阵（GitHub Actions ONLY）

1. **Contracts/static**：Manifest schema 兼容矩阵、stable identity、versions/signature、source maps、旧 ID migration、所有 kind 权限边界；架构 gate 拒绝 ParallelMarketplace 和 Browser→Host direct privileged call。
2. **Registry integration/security**：namespace claim、publisher roles、dependency solver、signed feeds、scan fail-closed、fraud/ratings verification、owner/admin permission、revocation/rollback；含真实 registry mock + 针对已部署 adapter contract，而非纯 snapshot。
3. **Desktop packaged**：shipping entrypoint 主 shell 进入市场、搜索列表/详情、安装 MCP+Skill，OAuth、team/private account、tool toggles，现有 overlay 迁移兼容及回归。
4. **Mobile device**：iOS/Android 真实 Host 读写、平台兼容 gating、权限、miniapp/WebMCP origin、离线应用及 AI 私有创建，跨账号隔离；缺真实设备/签名/审核时 blocker 记录并推进其他项目。
5. **Visual / a11y / performance**：FBCP canonical screenshots、dark/light、长名字、异常图标、中文/英文、键盘/VoiceOver/TalkBack、reduced motion；搜索/详情/安装延迟与网络缓存有实际 p50/p95 指标，先测量再硬门。
6. **Operational release**：published version 与 signing digest、审计、rollout kill-switch、举报/封禁时延、宕机容灾/缓存过期、真实评分来源和反作弊、CI artifact 指纹。

所有阶段按 requirement → test oracle → Actions run/job/artifact → independent acceptance；缺失=blocked。不得以文档合并或 Open VSX 演示实例成功充当 production evidence。

## 15. 风险、迁移与回滚

- 旧 Catalog 源真实生产依赖/权属须重读 backend 连接配置，不能贸然删旧服务；在新 registry 可用前保持原有安装+OAuth，并标识 legacy risk。重新绑定 Host 版本/权限时需灰度及兼容 mapping。
- VS Code 市场服务闭源、VSIX 格式不兼容和 Apple 禁止暴露任意 native API 是基础限制，不能绕过。
- Open VSX 既有安全公告（2026 年包括 scanner race、路径解析和缓存/host 相关问题）要求安全追踪、补丁验证和独立 threat-model；禁止安装一个旧 registry 当“最安全”。
- 若新 release 违反兼容/商店规则，保留 verified last-good + ledger；阻断该 kind 或平台而非降低规则，继续推进不受影响的业务。
- 个人 AI 创建的小程序不允许悄悄转为公共可搜索发行；来源/签名域/隐私/数据与插件管理状态长期清晰。

## 16. 来源 / licensing / provenance

**Internal (at the authority snapshot; every implementation round refresh exact refs)**:
- Desktop main: https://github.com/bhrumom/fabushi-desktop/tree/67d9892ec19abcbe86e270ae746bba3c808de785
- Current shipping UI: frontend/src/production/ProductionRenderer.tsx; frontend/src/recovered/features/plugins/overlay/{browser.tsx,desktop.ts,desktop-surface.tsx,view.tsx}
- Catalog/backend: source/shared/node/mcp/{mcp-marketplace.ts,mcp-catalog-flow.ts,mcp-manager.ts}; source/shared/node/marketplace/cursor-marketplace-client.ts; source/electron-main/mcp/mcp-desktop.ts
- iOS: https://github.com/fabushi-ios/fabushi-ios/tree/f098639ab178a839690b9dece60fa0d11c5d2e25
- Android: https://github.com/fabu-android/fabushi-android/tree/59f6fc8885ce1cb8d1ad4fc5d4ab36f690fb99a2
- docs/specs/offline-first-installable-mini-app-platform.md OMA-001 including §18 AI creation; FBCP SOURCE_OF_TRUTH.md Rev7; AGENTS.md.

**Official open-source/product references (accessed 2026-10-11)**:
- Open VSX source, license EPL-2.0, server/UI/CLI and deployment: https://github.com/eclipse-openvsx/openvsx
- Open VSX official FAQ: https://www.eclipse.org/legal/open-vsx-registry-faq/
- Open VSX namespace and verification: https://github.com/eclipse-openvsx/openvsx/wiki/Namespace-Access
- Open VSX scanning: https://github.com/eclipse-openvsx/openvsx/wiki/Extension-Scanning
- Open VSX security advisories: https://github.com/eclipse-openvsx/openvsx/security
- VS Code marketplace/install/UX: https://code.visualstudio.com/docs/configure/extensions/extension-marketplace
- VS Code extension manifest: https://code.visualstudio.com/api/references/extension-manifest
- VS Code extension runtime security: https://code.visualstudio.com/docs/configure/extensions/extension-runtime-security
- Theia extensions and plugin host: https://theia-ide.org/docs/extensions/
- Apple review policy: https://developer.apple.com/app-store/review/guidelines/
- Google Play developer policies: https://support.google.com/googleplay/android-developer/answer/16559646

## 17. Specification compliance record

| Item | Status | Evidence / reason |
| --- | --- | --- |
| Read current shipping owner and open-source references | **passed** | Source paths/commits in §§2/16; official VS Code/Open VSX/Theia documentation; no local build/test. |
| FPM-R01 through FPM-R12 implementation | **blocked** | This change defines target architecture; existing code has only partial Marketplace capabilities; no new shipping implementation. |
| FPM-AC01 through FPM-AC08 | **blocked** | Exact-head GitHub Actions, app/device/store and independent release evidence not yet performed for this extension. |

**Implementation precedence**: read AGENTS.md → current live SOURCE_OF_TRUTH/spec → current shipping graph → update design/ADR if necessary → code behind existing owners → GitHub Actions → exact-head compliance review → stage/release. FPM-001 enhances Plugins and OMA-001; it **does not** silently extend PR #42 migration completion criteria or downgrade Grok/FBCP gates.
