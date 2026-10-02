# Telegram Desktop → Fabushi Desktop：Rust 逐文件、逐模块完整等价迁移 Spec

Status: active  
Spec ID: TDRP-001  
Revision: 1  
Last updated: 2026-10-02  
Owner: Fabushi Desktop / Telegram Rust migration  
Related project: `projects/telegram-desktop-rust`  
Implementation status: **not implemented / not accepted by this spec**  
Canonical entry: `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md`

> 本规范落实用户的最新要求：将 `telegramdesktop/tdesktop` 使用 Rust 逐个文件、逐个模块等价迁移到 `bhrumom/fabushi-desktop`，完整实现全部范围。它不是“做一个类似 Telegram 的聊天界面”，不是 TDLib/原 C++ 的 Rust 包装器，不是只迁移后端，也不是将旧 Electron UI 自动视为最终实现。本次交付是规范和迁移治理资料；不得将其表述为客户端已经迁移完成。

## 1. 背景、权威顺序与既有项目关系

### 1.1 经本次读取确认的基线

| 对象 | 固定值 | 作用 |
| --- | --- | --- |
| 上游仓库 | `telegramdesktop/tdesktop` | 唯一首轮产品/源码等价基准 |
| 上游分支 | `dev` | 仅用于发现更新，不作为浮动依赖 |
| 上游 commit | `33261535a0e747f125e0ed25486f01e556330677` | 本次读取时的 dev HEAD；后续所有首轮对照均绑定此 SHA |
| 目标仓库 | `bhrumom/fabushi-desktop` | 唯一交付仓库；不是 `bhrumom/fabushi` |
| 目标调查基线 | `main@3b1f2c5deaf47867629c96b08bf4694f973177e2` | 记录调查起点，不声称它已经具备 Telegram parity |
| 目标基线 tree | `e06f125d1112548966e04a754a42fee3236943f3` | 保留既有代码的基准 |

机器可读冻结记录见 `projects/telegram-desktop-rust/upstream.lock.json`。首次记录已锁定根仓库、重要子树、许可证 blob 和 35 个直接 gitlink；**递归子模块叶文件、外部构建下载物、生成物和平台构建参数仍须在 P0 完整展开验证**。直接 gitlink 清单不等于完整递归源闭包。

### 1.2 权威顺序

在本项目范围内：最新明确用户要求 → 本 Spec → 固定上游源码及对应可复现行为 → 本项目模块/文件/符号台账 → 旧设计与聊天记录。遇到矛盾先修改 Spec/ADR，再实施。不得从旧聊天复制一个旧 HEAD 代替实时读取。

既有 `docs/specs/grok-bot-018-runtime-product-parity-recovery.md` 针对 Grok/Agent 工作域，允许按层选语言并默认移除未批准的 Fabushi 扩展。本规范依据最新用户要求，**明确批准 Telegram Rust 为一个独立扩展项目及独立桌面运行时边界**，不是未批准的 legacy runtime。既有 Grok 的 Coordinator/Host/Runner 边界、规范和证据继续对其自身范围有效。

本项目的 Telegram 应用逻辑、协议状态机、UI 控件与交互、存储、平台策略、工具中的项目自有逻辑，以 Rust 实现为终态。旧“Telegram UI 留在 React/TS、协议/通话永久保留 C++”方案不能作为本项目完成口径。本规范不授权顺带重写、删除或合并无关 Grok/Agent 代码；既有产品须经过独立回归和切换决策。不得向其他仓库的 PR #20 写本项目实现，或借用其 CI 证明本项目通过。

### 1.3 “等价”的可核验定义

对每个固定上游模块，明确输入、前置状态、系统能力、时钟/随机源、服务端响应和操作序列。Rust 实现须在同一条件下产生等价的可观察结果：UI 状态、事件顺序、协议字节或协议语义、持久化结果、取消/重试行为、权限与错误、资源生命周期。

随机数、时间戳、网络时延、平台字体栅格化等非确定项只能通过预先登记的规范化规则比较；不得用“大致相同”吞掉业务字段、顺序或错误。必要的品牌/签名/API 身份差异、经批准的安全修正须单列，不假装字节相同。

有限测试不能证明任意输入下永无缺陷。“完整等价验收”在本项目中严格指第 13 节所有门禁通过、所有范围均有追溯和证据、没有未闭合差异；禁止以“完美”承诺替代验收，也禁止因范围大而悄悄缩小需求。

## 2. 目标

**GOAL-01**：在目标仓库交付可安装、可升级、可实际使用的 Rust 原生 Telegram Desktop 等价客户端，覆盖冻结基线的全部可达功能及平台行为。

**GOAL-02**：逐文件、逐模块、逐符号/状态机建立可审计映射；每个上游叶文件都有处置，每个产品责任都有唯一 Rust owner，每项验收都有 exact-target-SHA 证据。

**GOAL-03**：保持上游的领域分工、可观察行为和兼容性，允许 Rust 惯用的所有权、类型、模块组织，而不是逐行模仿 C++ 对象布局。不得以重新设计为名削减分支、UI 或恢复行为。

**GOAL-04**：同时验证正常路径、错误/中断/重启路径、安全、隐私、性能、功耗、可访问性、国际化及完整安装包。主功能演示或编译成功都不是整体完成。

## 3. 范围、非目标及 Rust 边界

### 3.1 完整范围

- 整个冻结根仓库，不只是 `Telegram/SourceFiles`：所有源码、头文件、平台条件分支、TL/schema、生成器、资源、样式、着色器、构建/更新/安装工具、测试、配置及维护文件均入账。
- 所有递归 submodule，及构建脚本实际拉取的依赖、补丁、字体、翻译和资源。`desktop-app/lib_*`、`Telegram/codegen`、Telegram-specific `tgcalls`/`libprisma` 逻辑不得通过“放在 ThirdParty”绕开 Rust 迁移。
- 所有冻结版本可达特性，包括服务端配置、账号权限、付费权益、地区、平台开关控制的路径；暂时拿不到测试资格不等于不在范围。
- 上游支持的桌面平台、架构、安装渠道和最低系统版本须全部列入平台清单；具体以冻结 README、构建脚本、条件编译和实际发布能力交叉确认。

### 3.2 语言和依赖处置

**RUST-01**：上游项目自有 C/C++/Objective-C++/Python 等程序逻辑须获得 Rust 对应实现；文件格式、声明式资源、着色器语言和平台 manifest 无须伪装成 `.rs`，但必须逐件保留/转换并验证。

**RUST-02**：Rust 可用成熟 Rust 库替代通用算法或框架，只要精确记录覆盖的源文件/符号、版本、许可和语义验证；引入一个 crate 名称本身不是等价证据。

**RUST-03**：操作系统 ABI、驱动及系统 WebView 属于外部平台能力，不要求重写操作系统/浏览器引擎。项目拥有的权限、生命周期、导航、消息桥、会话和业务策略仍必须在 Rust。对通用第三方 C/C++ 库保留 FFI 的任何最终豁免，须有逐依赖 ADR、范围/风险/许可、Rust 替代评估及明确维护者批准；当前批准清单为空。不得以泛化的“原生依赖都允许”偷渡整块上游逻辑。

**RUST-04**：过渡期旧 C++ bridge 必须标为 `temporary-bridge`，有 owner、撤除阶段、运行时开关和依赖影响图；计入未完成。最终 Telegram 生产路径禁止依赖原 tdesktop 可执行文件、Qt 业务控件/原 UI、TDLib 封装或旧 C++ 协议实现来充当已迁移模块。第三方例外只能计为“批准复用依赖”，不能计入“Rust 已迁移”百分比；存在非 Rust 依赖时不得宣称传递依赖全为纯 Rust。

### 3.3 非目标

本项目不实现 Telegram 服务端，不新增上游不存在的移动端/网页版功能，不默认要求增加 tdesktop 没有的 Secret Chats，不绕过 Premium、支付、风控、权限或内容保护。不重写操作系统；不将品牌冒用、复制官方签名密钥或生产凭据作为“等价”。不会因测试账号/服务端功能不可用而删除相应要求。

## 4. 功能要求

所有下列要求均以冻结源码支持的实际范围为准，并向下细化到台账中的具体函数/页面/状态机。功能清单是导航，不是文件清单的替代；未列在表中的源责任仍受完整清单约束。

| ID | 必须迁移的责任 | 最低验收覆盖 |
| --- | --- | --- |
| FR-01 | 启动、单实例、账号选择、手机号/二维码及冻结版本支持的其他认证方式、两步验证、恢复、登出/撤销会话 | 所有认证状态、失效/过期、DC 迁移、取消、多账号隔离、重启 |
| FR-02 | TL 类型/方法生成、MTProto transport/auth/session、请求复用与取消、代理、重连、DC/CDN/文件路由、updates 同步 | codec 黄金向量、真实互通、乱序/重复/缺口、限流/时钟漂移/密钥失效 |
| FR-03 | peer/user/chat/channel、消息/附件/权益等领域数据、订阅和缓存一致性 | 与固定上游同输入重放，字段、排序、更新、删除及事件顺序对照 |
| FR-04 | 会话列表、归档、文件夹/过滤、置顶、未读/提及、草稿、搜索及分页 | 大量会话、实时更新、离线恢复、跨账号、滚动位置和焦点 |
| FR-05 | 文本/富文本/entities、回复/引用/转发、编辑/删除、相册、定时/静音发送、reaction、投票、消息菜单及冻结版本的消息变体 | 发送全生命周期、重复确认、不确定状态、权限边界、跨客户端对照 |
| FR-06 | 私聊、群组、频道、论坛/topic、成员/权限、管理、邀请/链接、封禁、统计与资料页 | 角色矩阵、权限变化、已删除/迁移 peer、访问失效 |
| FR-07 | 图片/视频/音频/语音/文件、缩略图、流式播放、录制、编辑、下载/上传/CDN/cache | 格式/seek/同步、取消/续传、文件替换/过期、坏文件/资源限额 |
| FR-08 | emoji/sticker/GIF、自定义 emoji、反应/动画、lottie、主题/背景/样式资源 | 素材和帧序列、缩放、触摸/鼠标、内存、暂停/后台生命周期 |
| FR-09 | 一对一/群组通话、音视频、屏幕分享和冻结版本支持的通话协商/加密 | 对照客户端互通、拒接/挂断、设备切换、权限拒绝、丢包/重连 |
| FR-10 | Bot/inline bot、键盘/回调、Mini Apps/WebView、Instant View、外部链接 | origin/权限/会话隔离、导航/退出、文件与剪贴板能力、恶意页面 |
| FR-11 | Stories、Premium、Stars/gifts、business、付费媒体/订阅、sponsored messages 及其他冻结版本入口 | 账号权益开/关、服务端配置、付款取消/失败/重复确认，禁止假成功 |
| FR-12 | Payments、Passport、WebAuthn/passkey、`tde2e` 等敏感模块的实际原始责任 | 精确分析源码用途；合法测试夹具、密钥/凭据保护、实际系统认证器能力 |
| FR-13 | 设置、通知/托盘/角标、快捷键/菜单、隐私/屏蔽、本地锁、自动下载/存储策略、账号管理 | 系统权限、锁定、DND、账号切换、撤销和重启保持 |
| FR-14 | 导出、导入/本地数据兼容、缓存管理、清理/迁移、诊断/支持能力 | 数据完整性、编码/格式、取消、磁盘满、损坏、备份/回滚 |
| FR-15 | 全部界面、布局、导航、窗口/浮层、输入法、文本选择/编辑、RTL、翻译、可访问性 | 逐页面/交互图、截图、语义树、键盘、屏幕阅读器、DPI/多屏 |
| FR-16 | Windows/macOS/Linux 平台服务、安装/便携/沙箱渠道、启动/更新/卸载/恢复 | 真正的安装包、签名/更新验证、系统集成及最低版本实测 |
| FR-17 | 构建配置、代码生成、测试基础设施、资源编译、release 工具及条件分支 | 干净检出可构建、确定性输出、所有构建组合、工具错误码/输入输出 |
| FR-18 | 既有 Fabushi 共存与切换 | 无共享密钥/数据库写入，无隐式启动额外服务，无 Grok/Agent 回归 |

## 5. 当前状态与已知缺口

目标 README 和根目录显示现有 Electron `desktop/`、共享 `frontend/`、`contracts/`、`native/mahayana-messaging` 以及 `third_party/mahayana`。存在 Spec-first 与 Grok 项目规范；这些是目标仓库现状，不是 Telegram 源码等价的证据。

本次已读取根/Telegram/SourceFiles/ThirdParty 目录结构、直接 gitlink、许可证/LEGAL、目标治理规范及既有 Grok 的范围约束。本次未宣称通读上游所有函数、构建原应用、展开所有递归依赖或测试目标产品。

`projects/telegram-desktop-rust/module-map.md` 是真实源目录对应的**初始职责规划**；不是逐叶文件已完成台账。`upstream.lock.json` 显式记录闭包待验证状态。所有实现验收初始为 `blocked`，不得继承旧项目的完成百分比。

## 6. 目标目录与模块映射规则

在不改变现有 workspace 的前提下创建独立 Rust workspace：

```text
telegram-rs/
  Cargo.toml / Cargo.lock / rust-toolchain.toml
  crates/
    fd-app/                  # 可执行入口、组合根和监督
    fd-core/                 # 配置、生命周期、基础类型
    fd-domain/               # 账号/peer/消息/附件等纯领域状态
    fd-application/          # 用例、会话/操作编排和端口
    fd-mtproto/              # MTProto 协议状态机
    fd-tl/                   # 生成的 TL 类型/codec
    fd-api/                  # 业务 RPC 与更新映射
    fd-storage/              # 唯一存储 owner 与事务
    fd-tdata/                # 冻结本地格式兼容/迁移
    fd-ui/                   # Rust 控件、scene、文本、输入和可访问性
    fd-ui-pages/             # 原各领域页面及 view-model
    fd-media/                # 媒体调度/录制/播放/上传下载
    fd-calls/                # 通话状态和媒体协商
    fd-webview/              # 受限系统 WebView 适配
    fd-platform/             # 平台端口及 OS 实现
    fd-security/             # 密钥/凭据策略、敏感流程
    fd-resources/            # 样式、语言、资源 manifest
    fd-update/               # 更新校验和原子替换
    fd-diagnostics/          # 脱敏、有界诊断
  tools/xtask/               # Rust 生成器、inventory、parity 与打包驱动
  tests/{unit,contract,differential,integration,e2e,security,performance}/
  fixtures/                 # 合成/脱敏且可公开的夹具
  packaging/{windows,macos,linux}/
projects/telegram-desktop-rust/
  SOURCE_OF_TRUTH.md / upstream.lock.json / module-map.md
  contracts/parity-ledger.schema.json
  inventory/                # P0 生成，不以空目录代替覆盖
  parity/                   # 文件、符号、场景及平台明细
  decisions/                # 显式批准的架构/兼容性决策
  evidence/                 # 指向不可变 CI evidence 的索引
```

路径是目标规划，不宣称当前存在。禁止创建空 crate 后将目录对应项标为已实现。工作域内保留上游相对路径的语义归属，例如 `history/view/*` 对应 `fd-ui-pages/src/history/view/*`，`history` 的领域责任进入 `fd-domain/src/history/*`，通过显式映射连接，不把业务全部塞进 UI 或 `fd-app`。

**MAP-01**：每一个上游叶文件有独立行，`.h` 与 `.cpp` 即使合并到同一 `.rs` 也保留两行，分别映射声明、函数、布局语义和依赖。

**MAP-02**：允许一对多/多对一，但必须有 symbol-level 证据，不允许 `dir/** → crate` 直接结案。重命名必须保留稳定 source ID；内容变化触发重验，不新建 ID 逃避历史。

**MAP-03**：每个目标生产模块必须反向追溯到源责任或获批扩展。避免未追踪新增逻辑、未使用的重复实现及一个上游 owner 对应两个生产 owner。

## 7. 架构、进程和状态所有权

### 7.1 依赖方向

`fd-app` 只做组合/监督。UI 页面调用 application 用例；application 依赖 domain 与定义好的 ports；MTProto、API、storage、platform、media 等实现 ports。domain 不依赖 UI/数据库/网络/系统 API。跨域禁止直接访问对方内部可变状态，禁止跨 crate 通过全局 singleton 绕过接口。

保留 tdesktop 的领域模块和行为边界，而非套入 Grok 的 Agent Coordinator/Host/Runner 来“复用”聊天逻辑。存在 Grok 集成时只能经过独立、版本化的扩展接口。

### 7.2 默认进程模型

默认一个 Rust 桌面应用进程：UI 主线程、异步网络/存储任务、受限工作线程。账号级 `AccountSession` actor 唯一拥有该账号会话和有序更新应用；`StorageOwner` 唯一提交该账号持久化事务；UI 持有只读快照/变更订阅。媒体设备线程遵守平台实时回调约束。

系统 WebView 的子进程由平台引擎隔离。处理不可信媒体的隔离 worker、更新 helper 可作为受监督独立进程，但须通过威胁模型和平台实测证明必要；不得为每个小模块强行创建进程。所有 helper 均有退出/崩溃/取消与资源释放合同，不自动启动旧 Node/远程控制服务。

### 7.3 必须保持的 Rust 语义

- QObject/signal-slot/rpl/crl 的事件顺序、线程归属、订阅销毁、reentrancy、同步与延迟回调须逐项翻译，不可直接以任意异步 channel 替代。
- 生命周期以强类型、所有权和显式 cancellation 实现；析构副作用与清理顺序保留。禁止通用 `Arc<Mutex<Everything>>` 或随意 `Send/Sync`。
- 所有队列、缓存、重试、并发、帧/文件缓冲有上限；更新不允许丢弃，UI 合并事件只在不改变语义时允许。
- 错误以类型表达；没有证据时不吞错、不返回默认成功。网络/用户输入/磁盘数据的生产路径不允许用 `unwrap/expect` 假设有效。
- `unsafe` 限于已审计的 ABI/性能边界，记录不变量、所有权、线程安全、测试和 owner；不得为模仿 C++ 指针大规模引入 unsafe。

### 7.4 原生 UI 技术决策门

本规范确定 UI 的业务、控件、交互和可访问性语义为 Rust-owned，不确定一个未经实测的 GUI crate 能覆盖所有旧系统。P1 必须提交 `ADR-UI-BACKEND`，锁定窗口/渲染/文本/无障碍依赖和最低系统支持，并用第 9 节平台矩阵证明 IME、RTL、screen reader、GPU/软件回退、弹窗/多屏及性能。

默认实施方向是 `fd-ui` 的 Rust 组件/scene/text/input 层与可替换平台渲染后端。可采用现有 Rust UI 基础设施，但不得用 WebView 承载整个 Telegram 业务 UI 或保留 Qt 控件来宣称 Rust UI 已完成。后端选型未通过前不得批量迁移页面；依赖不支持旧平台必须解决或明确阻塞，不能静默删除平台。

## 8. 接口、协议、存储与数据流

### 8.1 身份和命令合同

独立类型：`AccountId`、`SessionEpoch`、`PeerId`、`DialogId`、`MessageId`、`LocalOperationId`、`RpcRequestId`、`UpdateCursor`、`TransferId`、`CallId`、`SubscriptionId`。原协议中的 ID 位宽/有符号性不得随平台改变；不能混用临时消息 ID 和服务器消息 ID。

应用命令信封至少包含版本、account/session epoch、operation ID、取消 token、trace ID 和类型化 payload。`accepted` 仅表示持久化/接纳，不表示服务器已发送或通话已连接。事件流携带 account/session、单调本地 sequence、operation ID 和类型化 outcome。快照+增量订阅有一致性屏障；断开恢复从有效 cursor 重放或全量重建，旧 epoch 事件不得串入新登录账号。

### 8.2 发送与更新流

`UI action → application validation → durable intent → protocol request → server result/updates → atomic domain+cursor commit → UI projection`。

对上游支持的幂等发送方法，原随机/去重 ID 必须在第一次发送前持久化，并跨重试复用。不能对所有 RPC 虚构 exactly-once；方法无幂等保障时发生不确定结果，必须查询/对账或呈现不确定状态，禁止盲重放付款、删除、邀请或其他副作用。

`pts/qts/seq/date` 等更新 cursor 以冻结层实际语义分别处理；乱序、重复、缺口、频道差分、全量差分及 too-long 场景均有状态机和测试。崩溃不能使“已提交 cursor”与“领域变更未提交”分离。恢复不得跨账号套用 cursor。

### 8.3 MTProto/TL

固定 `api.tl`/`mtproto.tl` 等实际 schema 文件及 blob、layer、生成器和参数，生成 Rust 类型/codec。覆盖 constructor/method ID、flags、optional、向量、bytes 对齐、整数宽度、gzip/容器、未知/损坏输入、长度/递归限额。源码中存在但主配置未启用的生成目标仍入账。

认证密钥、server salt、session、消息序号、时间同步、ack、错误重发、授权导入导出、DC/proxy/media 路由均由 Rust 协议模块拥有。加密原语优先采用有维护和审计证据的库，不自行发明算法，不因“迁移等价”复制已知漏洞。涉及安全改正时登记差异、依据和回归，不静默更改协议。

### 8.4 持久化与迁移

原磁盘格式与内部存储引擎分开建模。`fd-tdata` 负责冻结版本格式/加密/版本兼容，`fd-storage` 可选择 Rust 适用引擎，但必须保留外部可观察恢复/缓存/导出语义。具体 schema、迁移版本、锁和事务边界须在实现该模块前落到合同文档。

禁止直接对用户现用 tdata 试验。只在获得明确授权后读取指定副本；先只读检测、版本/空间校验、加密备份、迁移到临时目标、逐对象校验，再原子切换。失败保持源副本与原应用不变。禁止两个客户端并发写同一 profile。密钥、会话、Passport 数据、认证器材料不能进日志、截图、公有 fixtures 或 artifact。

迁移需逐项记录账号、联系人/会话、草稿/本地配置、缓存索引、下载记录及原应用实际保存的其他资料；服务端可重新同步的缓存与不可丢的本地数据分开。不准以“重新登录就好了”关闭数据兼容项。旧版本不可读取的新 schema 必须通过备份/新目录回滚，禁止直接降级打开。

### 8.5 外部/IPC/FFI 边界

默认同进程使用类型化 Rust 接口。helper/可选 Grok bridge 使用版本化长度前缀消息，限定尺寸、权限、request ID、deadline、取消、断连清理、重启 epoch；具体编码须在 ADR 锁定并有兼容性测试。不得把 Rust 内存布局当 wire ABI。

FFI 使用不透明句柄和明确 alloc/free/线程/lifetime 规则；panic 不得跨 ABI；第三方 callback 在销毁后到达必须安全处理。只暴露最小能力。

## 9. 非功能约束

### 9.1 平台兼容

冻结 README 目前声明 Windows 7+（32/64 位）、macOS 10.13+、Linux 64 位静态构建，以及 Snap/Flatpak。它只是第一层基线，不能用 README 代替构建验证。P0 必须将各平台架构、SDK、Rust/toolchain 最低支持、libc、图形栈、沙箱/权限、安装形式、上游编译开关逐项锁定。macOS Intel/Apple Silicon 等实际发行架构须从固定构建配置核实。

现代 Rust 工具链/GUI crate 未必能覆盖旧 OS；这是必须关闭的工程阻塞，不是默认降级授权。上游可达的平台分支在至少一个适用矩阵 cell 执行；不适用必须给源码条件证据。无旧系统 runner、签名证书、音视频设备或账号资格，记录 `not-configured/blocked`，不得记通过。

### 9.2 UI、国际化与可访问性

逐屏记录布局、字体、间距、颜色、阴影、动效、滚动、焦点、快捷键、selection、拖放、context menu、窗口恢复和系统主题行为。覆盖至少 100/125/150/200% DPI、LTR/RTL、长文本/CJK/emoji/组合字符、IME composition、键盘-only、缩放/多屏与系统辅助技术。

同 OS/字体/分辨率/fixture 固定时优先逐像素对照；平台栅格化差异须在迁移前登记掩码/阈值，未经批准的截图差异数为零。动态内容只按稳定规则归一化；不可忽略整页、文本、图标、焦点或可交互区域。截图通过不能代替屏幕阅读器与输入法验证。

### 9.3 性能、功耗与稳定性

先对同一硬件/OS/配置/资料集采集冻结上游 reference 基线，再比较 Rust 安装包。性能数值是项目验收预算，不是声称已测得的上游数据。

默认预算：冷/热启动、会话切换、搜索及输入反馈 p95 不劣于上游 10%；同场景 RSS/CPU/能耗不劣于上游 10%；受控传输吞吐不低于上游 90%；60Hz UI 的 p95 CPU frame time 不超过 16.7ms，卡顿率不高于 reference。稳定空闲无永久 busy loop，后台隐藏后暂停非必要动画；不能通过停收消息来降低功耗。

每一场景至少 10 次独立运行并记录分布/样本/环境与噪声；CPU/功耗稳定窗口至少 10 分钟，近零基线时使用预先登记的绝对噪声界限。若原硬件无法达到绝对帧预算，须在测量前登记硬件适用性并保留相对约束，不得看结果后放宽。最终有 24 小时账号/重连/媒体混合 soak；无未解释内存/线程/handle 增长、死锁或丢操作。

### 9.4 安全、隐私与合规

日志默认不记录消息正文、手机号、访问令牌、auth key、密码、支付/Passport 内容；诊断导出需要清晰用户同意和脱敏/保留期。恶意媒体/TL/URL/HTML 是不可信输入，设解析/解压/分配/递归上限，路径规范化，阻止穿越、任意命令执行和越权 bridge。

使用自行注册的 Telegram `api_id`/`api_hash`；发布不能复用上游示例身份。不得要求或复制 Telegram 官方签名/更新密钥。不自动从用户真实会话发送测试消息、购买/赠送/支付/删除/邀请或发起通话；实时验收使用明确获准的测试账号/收件人/环境。

按源文件许可证处理本次派生改编。根 `LEGAL` 声明 GPL version 3 or later 并有 OpenSSL linking exception；须保留原版权/许可/适用例外，标识修改，为分发提供完整对应源码、生成器、构建/安装材料及适用的安装信息。不得把语言改为 Rust 当作重新许可依据，也不得未经逐文件检查统一重许可第三方依赖。是否存在链接/商店发行等法律问题由发布审查处理，未闭合则不发布。

使用 Fabushi 自有品牌/签名/更新源；保留 Telegram API 使用透明说明，不冒充官方应用。遵守 API 条款对 sponsored messages、阅读/在线状态、自动删除内容及用户知情操作的要求。

Telegram API 当前条款还约束平台数据的 AI/ML 使用。本项目默认禁止将 Telegram 内容、账号数据库、索引或附件送入 Grok/Mahayana/模型、训练/检索管线或诊断服务；仅获得用户同意不足以自动解除平台条款。任何相关扩展须经过独立条款/权利审查和明确批准，未通过前接口 fail closed。

## 10. 失败模式与必须复现的反例

| 场景 | 不变量 / 验收要求 |
| --- | --- |
| 发出前/发出后/ack 前/提交后各崩溃点 | 同一 durable operation 保留身份；不丢意图，不把未知当成功，不盲重发副作用 |
| 更新乱序/重复/缺口/过量 | 对照冻结 cursor 语义恢复；领域状态一致，不跳过 cursor |
| 断网、代理失败、DC 迁移、限流、服务器维护 | 分类、上限/退避、取消、用户状态一致，不忙循环 |
| 账号切换/登出后迟到回调 | epoch 隔离，不泄露旧账号消息/文件/通知 |
| 磁盘满、权限不足、损坏、schema 不兼容 | 明确错误；原数据可恢复；不截断/静默清空 |
| 媒体恶意长度、压缩炸弹、坏帧、设备消失 | 有界资源和清理；故障不拖死整个 UI |
| 录音/屏幕/通知/凭据权限拒绝或撤销 | 不假成功；可恢复并保持上游交互语义 |
| 睡眠/唤醒、时钟跳变、多屏/缩放切换 | 时间语义、重连、窗口和音视频设备恢复 |
| WebView 导航/新窗口/重定向/跨 origin | 受限权限、明确来源、销毁隔离；无任意执行 |
| 下载取消后重启、源文件被替换、CDN token 过期 | 内容校验、进度准确、正确重试/重新授权 |
| 双进程启动/更新中被杀/恶意更新包 | profile 单写，签名验证，原子切换，上一有效包可启动 |
| 页面/对象销毁后异步回调、订阅循环 | 无 use-after-free、死锁、订阅泄漏或重复通知 |
| 未拥有 Premium/业务/付款测试资格 | 功能实现与真实验收分别标记；缺环境保持 blocked |

已知上游安全缺陷不要求故意复刻；必须有安全差异单、威胁模型和替代行为测试，不可把它用作普通功能差异的借口。

## 11. 实施策略与逐文件工作流

### 11.1 P0：冻结与完整清点（先于产品实现）

1. 读取目标当前 exact HEAD 及根/嵌套指令，确认适用 Spec；保存工作起点。
2. 取本规范的固定 upstream commit，不使用浮动 dev；递归初始化每个 gitlink 到所锁 SHA。上游新版本仅写 drift 报告。
3. 用 NUL-safe `git ls-tree -r -z --full-tree <sha>` 等方式枚举主库及每个递归子模块的 blob/gitlink/symlink/mode，登记 repository + mount path + relative path + blob OID。GitHub recursive tree 若 `truncated=true`，按子树展开，不能接受截断结果。
4. 读取构建脚本/patch/下载配置，冻结非 git 来源到 URL/version/hash/license；只扫 Git 文件不够。所有网络下载必须可校验，不能依赖“最新版”。
5. 枚举所有编译配置和生成链。生成物记录输入、生成器、参数、输出摘要，重新生成可核验。header/template/macro 与未默认启用的分支不可丢失。
6. 生成逐叶 inventory、模块聚合、平台矩阵、符号/状态机提取计划、license/资产清单；独立比较本地 Git 与已锁 tree/OID，记录总数/类别/遗漏。
7. 逐文件初始状态 `discovered`，不能因创建映射自动升级。源闭包未完整时整体覆盖率为 `unknown`，不得报告 100%。

### 11.2 每个工作单元的强制流程

`读取源文件及调用者/被调者/条件分支 → 写语义合同与反例 → 指定唯一 Rust owner → reference harness → Rust 实现 → unit/contract/differential → 接入真实生产入口 → 集成/UI/平台测试 → packaged evidence → 独立复核 → 台账结案`。

迁移顺序由依赖拓扑和可交付 vertical slice 决定，不按文件名机械排序。不允许几千个空接口“先铺满”。每个 PR 声明源文件/symbol 集合、源/目标 SHA、依赖、原 owner→新 owner、已证明行为/剩余/风险/回滚。原则上一次只改一个可复核模块或紧密耦合文件组；确需扩大必须先解释依赖。

### 11.3 阶段及退出条件

| Phase | 内容 | 退出证据 |
| --- | --- | --- |
| P0 | 法律/基线/递归 inventory、模块/平台清单、台账和差异检查器合同 | 源闭包可重建；零未分类叶文件；真实总数；license 问题显式登记 |
| P1 | Rust workspace、基础类型/生命周期、UI backend/平台/安全 ADR、reference harness | 依赖方向门；各平台最小原生窗体、IME/辅助技术/图形回退实测 |
| P2 | TL/codegen/MTProto、账号认证、存储和 updates | 字节/状态差分、合法测试账号真实互通、崩溃恢复 |
| P3 | 会话列表→历史→输入→发送→更新→本地恢复纵向闭环 | Rust UI 的真实端到端发送/接收及重启，多账号无串扰 |
| P4 | 群/频道/topic/搜索/设置/语言/通知及全部消息变体 | 角色/平台/页面矩阵通过；无临时 bridge 的已迁移域 |
| P5 | 媒体/动画/录制/通话/屏幕分享/设备与网络异常 | reference 互通和设备/丢包/恢复实测 |
| P6 | Bot/Mini Apps/IV/商业权益/支付/Passport/WebAuthn/tde2e/导出等长尾 | 所有 gated 场景有证据；无账号/外部环境缺口被伪装通过 |
| P7 | 全平台打包/升级/迁移/回滚/性能/安全/无障碍 | exact-HEAD 安装包、SBOM/源码/签名、全矩阵和 soak |
| P8 | 全量独立验收、cutover、撤除所有临时桥接与重复 owner | 第 13 节全通过，canonical main 重验后才允许正式发布 |

阶段可以并行准备；依赖门未过不能宣布依赖它的阶段完成。一个 vertical slice 成功不表示该模块剩余文件完成。原生产数据/默认启动入口在 P8 之前不得无授权切换。

### 11.4 上游漂移与持续维护

每轮生成 `frozen SHA → current upstream HEAD` 差异，仅作为待处理记录。本轮仍验收固定 SHA；不得因 upstream 变化让已完成工作无限重开，也不能冒称已追上新 HEAD。升级基线必须单独 PR/变更单：新 root/submodule/hash、新增/删除/重命名符号与配置、依赖影响图、需重验项、证据重新绑定。删除的源行保留 tombstone/history，不消失以美化分母。

## 12. 验证策略与自动门禁合同

### 12.1 台账及状态机

本项目的台账格式合同见 `projects/telegram-desktop-rust/contracts/parity-ledger.schema.json`。每行至少记录源身份/内容、类别、责任/符号、依赖/条件、处置、目标路径/符号/owner、风险、测试、证据、审查和状态。

状态严格区分：`discovered → analyzed → mapped → implemented → verified → accepted`；`blocked` 可从任何未结案状态进入，修复后从有证据的阶段恢复。`implemented` 仅表示有实际逻辑和生产接线，不代表通过。`accepted` 需要独立审查和满足该行所有适用门；资源/文档/依赖也须有对应的 hash/许可/生成/构建验证。

处置类型：`rust-port`、`rust-library-replacement`、`generated`、`resource-preserved`、`declarative-conversion`、`build-tool-equivalent`、`upstream-reference-only`、`approved-native-dependency`、`temporary-bridge`、`not-applicable`。`temporary-bridge` 不能 accepted。`not-applicable` 只限有源码/平台证据的真实无关项，不允许用于尚未实现/缺资源/不想迁移的功能。`upstream-reference-only` 只允许不进入产品的参考/审查材料，不允许把可达代码划入。

源覆盖率、已实现比例、已验收比例、Rust 自有逻辑比例、获批原生依赖数必须分别展示；零分母/空数组/缺失 inventory 返回失败/unknown，不得返回 100%。统计必须分源码、生成、资源、工具、依赖、平台，而非用大量图片稀释未迁移代码。

### 12.2 未来 xtask 命令合同（本次未实现这些工具）

```text
cargo xtask parity inventory --locked
cargo xtask parity verify --mode progress --locked
cargo xtask parity verify --mode release --locked
cargo xtask parity drift --locked
cargo xtask parity report --target-sha <exact-commit>
```

- inventory：可重现扫描，发现 missing/unexpected/duplicate/改 hash/漏 submodule/未知生成器即失败；不能自动丢行或接受差异。
- progress：验证清单完整性、单调证据、已 accepted 行回归、生产接线/owner 和新增范围；允许诚实的未完成行。没有 inventory 时必须失败，而不是零项成功。
- release：另要求全部适用项 accepted、所有 FR/AC passed、所有平台实际通过、零 temporary bridge/未批准例外/未闭合阻塞。
- drift：只报告差异不改变冻结锁；rebaseline 是独立受审写操作。
- 工具自测必须包含空台账、缺一文件、同名不同子模块、修改 blob、删除测试、错误 SHA、纯 mock、死代码 facade、假 evidence URL、重复 owner、N/A 滥用和子树截断的负例。

### 12.3 测试层级

1. **结构和静态**：fmt、clippy（warnings as errors，有限且有理由的 allow）、依赖方向、unsafe/FFI、license/SBOM、禁止遗留生产路径、实际 feature 组合编译。
2. **单元/属性/状态机**：边界值、确定时钟和随机源、取消/重试/事务/订阅生命周期；property tests 与必要的并发模型检查。
3. **差分**：运行固定上游 oracle 与 Rust 模块，对同一 fixture 比较字段、协议 bytes、事件序列、输出文件和副作用。oracle 单独构建/进程运行，不把 reference 链入最终应用。上游测试迁移后保持原断言含义，并补未覆盖错误路径。
4. **集成**：从真实入口走到 canonical owner/adapter/storage/transport/renderer，验证调用次数/身份/崩溃边界。只有 mapper/facade/字符串存在不算接线证据。
5. **E2E**：真实 Rust 安装包、明确授权测试账号和测试收件人，与固定参考客户端互通；页面、系统权限、音视频、更新、导出和账号完整流程。
6. **安全/鲁棒性**：fuzz TL/本地数据/媒体/URL/导入/IPC，至少登记 seed corpus、执行次数/时长、平台、问题及修复；不以 fuzz 无 crash 证明安全。
7. **非功能/回归**：第 9 节性能功耗、可访问性、平台和 soak；既有 Grok/Agent 的适用回归独立跑，不共享完成状态。

mock/fixture 只证明它覆盖的合同，不代替真实服务端、操作系统、设备、签名/安装包或用户体验。外部服务不可用：单元可 passed，真实互通项仍 blocked。

### 12.4 CI 和 exact-HEAD 原则

实施期间使用主题分支和可审 PR，不强推覆盖他人工作，不放宽原仓库门禁。正式合并/发布以本仓库 GitHub Actions 的 exact target HEAD 为权威；需要设备/旧 OS 的用例可由受控 runner/设备执行并把可验证证据归档到对应 run。

记录 workflow/run/attempt/job/step ID、实际命令/退出码、测试计数、target SHA、upstream lock digest、platform、依赖锁、安装包 artifact ID/name/digest。必须检查 steps，不是只看绿色 workflow 名字。跳过、零测试、continue-on-error、未配置、只上传文件都不能满足产品门。

目标代码变动后，旧运行只作为历史证据；不得用其他分支/仓库的绿灯作为当前 HEAD 通过。合并 main 产生新 commit 时，canonical main 门禁重新运行。正式验收只能运行该 SHA CI 产出的安装包，不用本地重新编译包代替。

证据报告由 CI 在构建后生成并作为不可变 artifact 发布，以避免把“包含自身 commit SHA 的文件”提交造成自引用。仓库 evidence index 引用真实产品 SHA/run/digest；后来的文档提交不能篡改原证据绑定。

本规范的落库可仅运行文档/JSON/链接检查，不能伪装成上述尚未实现的产品 CI。正式 release gate 未实现或未配置时整体为 blocked。

## 13. 验收标准与 Definition of Done

| ID | 通过条件 |
| --- | --- |
| AC-01 | 固定 root、全部递归 gitlink、外部下载/补丁/生成/资源/平台闭包可重建且摘要一致；inventory 无截断/遗漏/重复 |
| AC-02 | 每个源叶文件都有审过的处置；每个可达符号/状态机/平台条件有 Rust 映射或合法且明确批准的处置；无目录级假结案 |
| AC-03 | Telegram 自有逻辑均由 Rust 实际生产 owner 执行；零 temporary bridge、空壳、死代码 facade、重复 owner 或旧 C++/Qt/TDLib 代实现 |
| AC-04 | FR-01…FR-18 全部适用场景完成差分/集成/真实验收；无未解释语义/错误/顺序差异 |
| AC-05 | TL/MTProto/同步/发送/加密安全与跨客户端互通通过；无不确定副作用被重复提交 |
| AC-06 | 完整 UI/交互/语言/主题/IME/可访问性矩阵通过；未经批准视觉差异为零 |
| AC-07 | 所有冻结支持的平台/架构/最低版本/打包渠道实测通过；缺环境不视为通过 |
| AC-08 | 原数据兼容、崩溃恢复、升级/回滚、取消/登出/多账号隔离通过；无不可恢复用户数据损失 |
| AC-09 | 性能/内存/功耗/传输/soak 达到第 9 节预算；证据可复现且没有事后放宽 |
| AC-10 | 威胁模型、fuzz/负例、依赖/unsafe 审计和缺陷处置完成；严重安全/隐私缺陷为零 |
| AC-11 | GPL/例外/第三方/商标/API 条款、API 身份、签名/更新、对应源码/SBOM/构建材料审查闭合 |
| AC-12 | 新 exact HEAD 的必需 CI 全通过，安装包 provenance 校验且真实运行；独立审查者逐 FR/AC 签署；canonical main 重验 |
| AC-13 | 旧产品切换/共存/撤除计划执行，无既有 Grok/Agent 回归，无 Telegram 数据隐式流入 AI 路径 |
| AC-14 | release parity 检查器真实运行，所有适用台账 accepted；零 unknown、未批准豁免、not-configured 或未闭合 blocker |

整体完成是所有条件的交集，不是平均分。不得以阶段完成、代码行数、目录数量、截图或“所有测试通过但没有真实测试”宣布全部完成。

独立验收只能读证据和复核，不把实施者最后回复当作新的验收标准；实施者不能自签独立 review。验收报告明确 `complete: false/true`、本次 target SHA、冻结源、已证实/剩余/阻塞及下一工作单元；验收报告自身不驱动无证据无限新建“已通过”记录。

## 14. 发布、切换、回滚

以独立应用 ID/profile/cache/update channel 构建开发版，避免覆盖现有 Fabushi/Telegram 用户资料。开发版必须明显说明未完成范围，不冒充完整替代版本。凭据由受控 CI/本地授权配置提供，不进源码。

发布顺序：锁定候选 SHA → 全门禁 → 生成可追溯包/源码/SBOM → 实机安装/运行/升级/卸载 → 独立验收 → main 重验 → 正式 release。签名/公证/更新签名材料缺失则 release blocked，不得用未签开发包替代正式发布验收。

更新先校验来源、版本、签名和摘要，后台准备到新目录，原子切换；失败保留上一有效版本。拒绝未授权降级和不兼容 schema。rollback 保留加密备份、必要审计和恢复说明；不能通过静默运行旧 C++ bridge 假装 Rust 产品继续正常。

正式切换后移除临时桥、过渡入口和重复 owner，保留必要的可独立构建测试 oracle/许可记录但不打入产品。无关 Grok/Agent 的撤除不属于本项目授权。

## 15. 可观测性与证据

结构化事件至少关联 source lock digest、target SHA、account 的脱敏本地标识、session epoch、operation/request/subscription、状态转换、耗时、队列/缓存规模及错误类别。密钥/消息正文不入日志。日志轮转、容量/保留期有上限，诊断关闭后不继续上传。

每个 evidence record 含：要求/源行 ID、test ID、test kind、oracle 身份、target SHA、lock digest、平台/配置、run/job/step/artifact 标识、SHA-256、实际结果、覆盖边界、审查者。图片/录像说明环境及数据脱敏，不以截取成功片段掩盖其余失败。

本次基线状态见 `projects/telegram-desktop-rust/STATUS.md`。该文件是诚实起点，不是完整实现结果；新提交只按新证据改状态。

## 16. 参考与出处

1. 用户本轮明确要求：`telegramdesktop/tdesktop` → `bhrumom/fabushi-desktop`，Rust 逐文件逐模块完整等价迁移，并把 Spec 写入目标仓库。
2. 固定上游：https://github.com/telegramdesktop/tdesktop/tree/33261535a0e747f125e0ed25486f01e556330677
3. 上游 README：https://github.com/telegramdesktop/tdesktop/blob/33261535a0e747f125e0ed25486f01e556330677/README.md
4. 上游 LICENSE / LEGAL：https://github.com/telegramdesktop/tdesktop/blob/33261535a0e747f125e0ed25486f01e556330677/LEGAL 及同 SHA 的 LICENSE；原例外文字保留在原文件，不擅自扩展授权范围。
5. 上游 gitmodules：https://github.com/telegramdesktop/tdesktop/blob/33261535a0e747f125e0ed25486f01e556330677/.gitmodules
6. 官方 API 身份：https://core.telegram.org/api/obtaining_api_id （2026-10-02 读取）
7. 官方 API 条款：https://core.telegram.org/api/terms （2026-10-02 读取；发布前须重新核验，不把动态条款冻结为永久有效）
8. 目标规范：`AGENTS.md`、`docs/specs/spec-first-ai-development.md`、`docs/specs/SPEC_TEMPLATE.md`。
9. 既有 Grok 范围：`docs/specs/grok-bot-018-runtime-product-parity-recovery.md` §§1.2–3；本规范 §1.2 明确本次批准的独立扩展边界。

## 17. Spec compliance 起始记录

| 对象 | 状态 | 证据 / 原因 |
| --- | --- | --- |
| 规范范围/责任/合同/阶段/验收定义 | 已在本规范定义；不等于产品验收 | 本文件和项目配套文档 |
| 已读取根 commit、重要树、35 个直接 gitlink 与 LICENSE/LEGAL | 已记录 | upstream.lock.json 的 exact IDs 和 provenance URLs |
| 完整递归 inventory / 全量符号映射 | blocked | P0 尚未执行；禁止报告完整覆盖率 |
| FR-01…FR-18 | blocked | 本规范没有对应 Rust 生产实现/测试/包证据 |
| AC-01…AC-14 | blocked | 必须由后续真实 evidence 逐项闭合 |
| 正式发布 | blocked | 不具备全部 parity、平台、合规和安装包验收证据 |

下一实施动作固定为 P0：在目标实时 HEAD 上读取本规范，完成固定源闭包/平台/license 清点和机器可校验的逐叶台账；**不得从空 UI、已有 Electron 或 C++ 包装开始，随后补写“全部完成”。**
