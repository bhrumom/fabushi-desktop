# Telegram Desktop → Fabushi Desktop：源码理解驱动的重新架构实现 Spec

Status: active  
Spec ID: TDRP-001  
Revision: 2  
Last updated: 2026-10-02  
Owner: Fabushi Desktop / Telegram rearchitecture  
Related project: projects/telegram-desktop-rust  
Implementation status: not implemented / not accepted by this spec  
Canonical entry: projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md

> 最新要求：不再按 Telegram Desktop 的文件和模块逐一翻译。允许并要求深入阅读固定版本源码，理解完整功能、状态机、边界条件、平台行为、失败语义和历史问题；然后重新设计更好的目标架构。所有进入产品实现的 C++ 代码必须由更好的 Rust 实现取代。非 C++ 部分不强制 Rust，而是在理解职责后，以正确性、安全性、性能、维护性、生态、平台适配和许可证为依据选择最合适的架构与语言。最终要求是完整功能与行为覆盖、协议兼容和可证明的生产实现，不要求目标目录、类、函数或模块与上游一一对应。

## 1. 决策变化

Revision 1 采用“逐文件、逐模块、逐符号迁移”的治理方式。Revision 2 明确废止这种目标架构约束。

仍然保留固定上游源码作为研究和兼容性基准，但用途发生变化：

- 上游源码用于发现功能、理解行为、解释边界条件、查清协议和平台细节、构建测试场景以及发现历史工程问题。
- 上游文件树只作为研究覆盖和 provenance 记录，不再决定目标代码的目录、crate、类、函数、进程或状态 owner。
- 不要求 source file → target file、source module → target module 或 source symbol → target symbol 的机械映射。
- 一个新的目标能力可以综合多个上游目录、源码、协议文档和黑盒行为；一个上游模块也可以被拆到多个更清晰的目标边界。
- 目标内部结构以重新设计后的领域模型、状态所有权、依赖方向和性能边界为准。
- 最终验收按 capability、behavior、protocol、platform 和 product scenario 完成，不按迁移了多少 C++ 文件完成。

本路线是 source-informed reimplementation，不是 clean-room。阅读 GPL 源码后重新实现不能被描述为自动摆脱 GPL 或自动获得独立许可。许可证与派生作品问题必须作为独立发布门审查；若未来目标变成严格 non-GPL clean-room，实现团队必须切换到另一套隔离规范，不能把本路线包装成 clean-room。

## 2. 固定研究基线与权威顺序

第一轮研究基线：

- 上游：telegramdesktop/tdesktop
- 上游 ref：dev，仅用于发现，不作为浮动构建输入
- 固定 commit：33261535a0e747f125e0ed25486f01e556330677
- 目标仓库：bhrumom/fabushi-desktop
- 本 Spec 修订前目标 HEAD：58cb7f5f29ac02440bf1c5e819c43863c1d2d727

upstream.lock.json 继续负责冻结源码、gitlink、资源、下载依赖和 provenance。它不再产生“逐文件迁移百分比”。

本项目的权威顺序：

1. 最新明确用户要求；
2. 本 Spec；
3. Telegram 官方协议/API 文档与实际服务端合同；
4. 固定上游源码及其可复现产品行为；
5. 已批准 Architecture Decision Records；
6. capability/behavior ledger 与 exact-HEAD 验收证据；
7. 历史设计、旧 module map 和聊天记录。

遇到冲突先更新 Spec 或 ADR，不边写代码边默默改变原则。

## 3. 核心目标

### GOAL-01 — 完整能力重建

交付一个可安装、可升级、可长期维护的 Fabushi Telegram 桌面客户端，覆盖固定基线的全部适用产品能力、协议能力和平台行为。

### GOAL-02 — C++ 全面替换

所有进入最终产品依赖闭包的 Telegram/desktop-app 自有 C++ 业务代码、状态机、UI 逻辑、协议逻辑、平台策略和工具逻辑，都必须由 Rust 实现替代。

最终产品不得依赖以下内容来冒充已重建：

- 原 tdesktop 可执行文件；
- Qt 业务 UI；
- TDLib 作为整个客户端逻辑的替身；
- 原 C++ MTProto/session/update 状态机；
- 原 C++ Telegram-specific calls/media/business logic；
- 保留旧 C++ 模块再套一层 Rust facade。

操作系统内部如何实现不属于本项目的 C++ 禁令。例如系统 WebView、系统媒体框架、驱动、系统字体引擎属于外部平台能力。项目自己随包分发或编译进产品的 C++ 库则必须在 dependency decision 中明确处理；默认目标是以 Rust 替代。任何例外都必须单独批准，而且不能包含 Telegram 自有产品逻辑。

### GOAL-03 — 架构优于源码结构

源码的历史结构不是目标。新架构应当：

- 有单一而明确的状态 owner；
- 通过 typed commands/events/ports 表达跨域交互；
- 将领域、应用编排、协议、存储、UI、平台能力隔离；
- 使崩溃恢复、取消、重试和幂等成为一等设计；
- 让并发、资源上限、生命周期和权限可证明；
- 消除不必要的全局状态、隐式 ownership、回调链和跨层依赖；
- 在不破坏产品兼容性的前提下修复可以证明的架构缺陷。

### GOAL-04 — 非 C++ 采用最佳技术

对于 Python、CMake、Shell、JavaScript/TypeScript、声明式资源、shader、平台 manifest、生成器和其他非 C++ 内容，不做“全部强制 Rust”。

每项选择必须根据职责决定：

- Rust：默认用于协议、领域状态、并发 runtime、存储、native UI 业务层、安全边界、性能敏感逻辑、跨平台核心和可执行工具。
- TypeScript/JavaScript：只在 Web/Electron/WebView/浏览器生态边界确实更合适时使用；不能因为现有仓库有 Electron 就强迫 Telegram 新架构使用它。
- Swift、Objective-C、C#、平台脚本等：只允许作为平台 API 的窄适配层，且有明确 ADR；不得重新形成第二套业务 owner。
- Python：仅在生成、发布、研究或生态工具上比 Rust 更合理时使用。
- WGSL/GLSL/Metal shader、JSON、XML、plist、desktop entry、资源文件等：使用其自然格式，不为“统一语言”强行改写。
- 第三方库：优先成熟、安全、维护活跃且许可适配的实现；语言不是唯一标准。

“最佳”必须有 ADR 证据，不是主观口号。

## 4. 非目标

- 不逐行翻译 C++。
- 不保留上游目录结构作为目标架构。
- 不要求每个上游类在 Rust 中存在同名 struct。
- 不为了 source parity 保留糟糕的 ownership、线程或生命周期模型。
- 不重新实现 Telegram 服务端。
- 不绕过 Premium、支付、风控、隐私、权限或平台安全机制。
- 不复制 Telegram 官方签名、凭据或品牌身份。
- 不因为某项功能难测而把它从范围里删除。
- 不宣称 source-informed 路线天然规避 GPL。
- 不授权删除或重构无关 Grok/Agent 生产代码；两套产品边界必须独立治理。

## 5. 研究源码的方法

### 5.1 研究单元不是文件，而是 capability

研究以用户能力或系统能力为单位，例如：

- Authentication and accounts
- MTProto transport/session
- Updates and consistency
- Dialogs and history
- Message composition and send lifecycle
- Media transfers
- Calls and screen sharing
- Notifications
- Search
- Groups/channels/topics
- Stories
- Premium/Stars/business/payments
- Bots/Mini Apps/WebView
- Settings/privacy/local lock
- Export/local data
- Accessibility/i18n
- Window/tray/platform integration
- Update/install/recovery

每个 capability 可以引用任意数量的上游文件、子模块、官方文档和黑盒场景。

### 5.2 源码研究报告必须提取什么

每个 capability 的 research dossier 至少提取：

- 用户可观察行为；
- 对外协议与数据格式；
- 状态机和关键状态；
- 必须保持的 ordering、idempotency、retry、cancellation；
- 异步边界和线程约束；
- 持久化和 crash-recovery 语义；
- 权限和安全边界；
- 平台差异；
- 资源与性能约束；
- 输入错误、断网、损坏数据、迟到回调等反例；
- 上游中值得避免的技术债或耦合；
- 可证明的兼容性要求和测试 oracle。

研究报告可以引用 source path、commit、symbol 和短摘要作为 provenance，但不把 source symbol 当作目标 API。

### 5.3 源文件 inventory 的新用途

仍然做完整递归 inventory，因为需要证明没有漏看重要功能、子模块、生成链和许可证。

但 inventory 状态只有：

- discovered
- classified
- researched
- not-relevant-with-reason

它不包含“ported”或“accepted”字段，也不参与实现完成百分比。

真正的实现完成度来自 capability ledger。

## 6. 目标参考架构

以下是目标方向，不是从 tdesktop 复制的目录结构。P1 允许通过 ADR 调整，只要不破坏本 Spec 的 ownership 原则。

    telegram-next/
      core/
        domain/
        application/
        protocol/
        sync/
        storage/
        security/
        media/
        calls/
      ui/
        shell/
        navigation/
        components/
        feature-surfaces/
        accessibility/
      platform/
        windows/
        macos/
        linux/
      webview/
      tools/
      resources/
      packaging/
      tests/

### 6.1 Domain

纯业务类型、不变量和状态转换。不得依赖 UI、数据库实现、HTTP client 或 OS API。

### 6.2 Application

用例编排、commands、events、ports、operation identity、cancellation 和 lifecycle。禁止直接拥有平台细节。

### 6.3 Protocol / Sync

Rust 实现 TL、MTProto、auth key/session、DC、updates、difference、cursor、ack、retry 和传输策略。

Protocol 负责 wire correctness；Sync 负责把服务端事件可靠地变成 domain state。两者不能被 UI 直接操作内部状态。

### 6.4 Storage

单一事务 owner。持久化 intent、领域状态和更新 cursor 的原子边界必须显式。格式兼容与内部存储设计分开。

### 6.5 UI

C++/Qt UI 被 Rust UI 取代。UI 使用单向状态流：snapshot/state → render，user intent → application command。

GUI backend 必须通过 ADR 选型，至少比较：

- Rust 生态成熟度；
- Windows/macOS/Linux 支持；
- IME、RTL、accessibility、screen reader；
- GPU/software fallback；
- 多窗口、多屏、DPI；
- 文本 shaping、selection、clipboard、drag/drop；
- 性能、内存和功耗；
- 最低系统版本；
- 长期维护风险。

不允许因为 Web 技术开发更快就默认把整个 Telegram UI 放进 WebView。

### 6.6 Platform

平台层只做系统能力：窗口、托盘、通知、凭据、文件选择、媒体设备、screen capture、URL handlers、autostart、sandbox、更新和安装。

平台层不得重新拥有账号、消息、同步等业务状态。

### 6.7 WebView

Mini Apps 和必须嵌入网页的能力使用系统 WebView 或最合适的安全引擎。导航、origin、permissions、bridge、session 隔离和生命周期由目标实现拥有。

## 7. 架构选型门

每个大 capability 开工前必须写 Architecture Decision Record。ADR 至少回答：

1. 上游真正解决了什么问题？
2. 哪些行为必须兼容，哪些内部机制可以改变？
3. 原设计的优点和问题是什么？
4. 至少两个可行目标方案是什么？
5. 为什么选当前方案？
6. ownership 和 failure boundary 在哪里？
7. 为什么选这种语言和库？
8. 性能、功耗、安全、平台、许可证和维护代价是什么？
9. 如何测试它优于或不劣于参考实现？
10. 如何回滚？

没有 ADR 的核心模块不得因“Rust 看起来更现代”直接开写。

## 8. 功能范围

固定基线的全部适用能力仍在范围。最低包含：

### FR-01 Authentication and account lifecycle
手机号、二维码、2FA、session 恢复、多账号、logout、revoke、DC migration、错误/过期/取消。

### FR-02 Protocol and connectivity
TL、MTProto、transport、proxy、session、ack、retry、server salt、time sync、DC/CDN/media routing、rate limit 和断网恢复。

### FR-03 Updates and consistency
pts/qts/seq/date 等实际语义、channel difference、全局 difference、duplicate/out-of-order/gap/too-long、restart recovery。

### FR-04 Dialogs and messaging
dialogs、folders、archive、pin、unread、draft、history、send/edit/delete/reply/forward/quote/entities/reactions/polls/scheduled messages 和实际冻结版本支持的 message variants。

### FR-05 Groups/channels/topics
roles、permissions、members、admin、invite、ban、forum/topic、stats、profile。

### FR-06 Media
image/video/audio/voice/file、thumbnail、upload/download、resume、CDN、cache、streaming、recording、editing、playback。

### FR-07 Calls
1:1/group call、audio/video、device switching、screen sharing、network recovery 和实际加密/协商能力。

### FR-08 Rich product surfaces
stickers/GIF/emoji/custom emoji、stories、bots、inline bots、Mini Apps、Instant View、Premium、Stars/gifts、business、sponsored messages、paid media。

### FR-09 Sensitive flows
payments、Passport、WebAuthn/passkey、tde2e 等固定基线实际能力。

### FR-10 Settings/platform
notifications、tray、badge、shortcuts、menus、privacy、local lock、auto download、storage policy、themes、language、window restore。

### FR-11 Data lifecycle
local data、cache、migration、export、cleanup、corruption handling、backup/rollback。

### FR-12 UI/i18n/accessibility
全部页面、导航、IME、selection、RTL、CJK/emoji、screen reader、keyboard-only、DPI、多屏和系统主题。

### FR-13 Build/update/release
Windows/macOS/Linux、安装、portable/sandbox、update、rollback、signing、packaging、resource/code generation。

### FR-14 Coexistence
与现有 Fabushi/Grok/Mahayana 代码隔离，避免共享写入、隐式启动和数据越界。

任何源码研究发现但以上没有显式列出的可达功能，都自动进入范围，除非经 Spec 变更明确排除。

## 9. 接口与状态原则

### 9.1 强类型 identity

至少区分：

AccountId、SessionEpoch、PeerId、DialogId、MessageId、LocalOperationId、RpcRequestId、UpdateCursor、TransferId、CallId、SubscriptionId。

不得用同一个字符串/整数跨层冒充多个身份。

### 9.2 Durable operation

对有副作用的操作：

user intent → validation → durable operation intent → protocol execution → authoritative result/update → atomic domain commit → UI projection。

支持幂等的 Telegram 方法复用原有 idempotency/random id。对无法证明 exactly-once 的副作用，发生未知结果时必须 reconciliation，不盲目重放。

### 9.3 Actor / owner 原则

账号级 session/sync 只有一个 canonical writer。Storage 只有一个事务 commit owner。UI 持有 projection，不成为第二份业务真相。

媒体、下载和 call 可以有独立 actor，但状态转移通过 typed event 接入 application/domain。

### 9.4 生命周期

所有异步操作必须有 cancellation、deadline 或明确的长期订阅生命周期。页面销毁、账号退出、session epoch 变化、应用退出都必须使过期回调失效。

禁止通用 Arc<Mutex<Everything>> 式所有权。

## 10. C++ → Rust 替代标准

一项 C++ capability 只有同时满足以下条件，才算被 Rust 替代：

- Rust 代码是生产路径真实 owner；
- 原 C++ production path 已从目标产品依赖闭包移除；
- 行为合同和失败语义有测试；
- 状态 ownership 与生命周期比原设计更清晰或至少不更差；
- 没有为了通过测试而保留隐藏 C++ fallback；
- packaged application 证明真实调用 Rust 路径；
- dependency/SBOM 检查证明没有意外重新引入该 C++ 实现。

“写了 Rust facade”“绑定 C ABI”“调用 TDLib”“把 C++ 放到 helper process”都不算替代。

对第三方 C++ 依赖，如果暂时找不到可靠 Rust 等价物，状态保持 blocked 并写 ADR。不能把“生态难题”自动解释成永久例外。

## 11. 非 C++ 技术选择标准

每项技术选择按以下顺序评分，不做政治式或主观式“最先进”判断：

1. Correctness and protocol fidelity
2. Security and memory safety
3. Explicit ownership and failure semantics
4. Performance and power
5. Platform compatibility
6. Accessibility and UX capability
7. Ecosystem maturity and maintenance
8. Testability and observability
9. Build/release simplicity
10. License and distribution fit
11. Team/repository consistency

没有一种语言天然在所有层最好。语言选择必须服务边界，不允许为统一语言牺牲产品质量。

## 12. 验证模型

### 12.1 不再做结构 parity gate

禁止将以下项目作为完成标准：

- C++ 文件数与 Rust 文件数相等；
- source module 和 target crate 同名；
- source class 有同名 Rust struct；
- 逐目录完成百分比；
- grep 到相同 symbol；
- 一个 facade/mapper 存在。

### 12.2 必须做 behavior/capability gate

每个 capability 有：

- research dossier；
- target ADR；
- behavior contract；
- production owner；
- unit/property/state-machine tests；
- differential or golden tests；
- integration tests；
- 真实 packaged E2E；
- 适用的平台、性能、安全和可访问性证据。

### 12.3 Reference oracle

固定上游可以在受控测试环境作为 oracle，用于输出/行为对照，但不链接进最终产品。

差分比较重点：

- protocol bytes/semantic fields；
- state transitions；
- event ordering；
- storage outcome；
- UI visible state；
- cancellation/retry；
- recovery after restart；
- platform observable behavior。

若新实现有意改善上游缺陷，必须在 deviation record 中说明旧行为、新行为、风险、兼容性和批准依据。

### 12.4 完成度

分别报告：

- research coverage
- capability designed
- Rust C++ replacement
- production wired
- behavior verified
- packaged accepted
- platform accepted
- release accepted

禁止合并成一个能被小文件数量稀释的“总体迁移百分比”。

## 13. 性能、安全和产品质量

### 13.1 性能

先在同硬件、同 OS、同账号/fixture 下测固定参考实现，再测目标实现。

默认目标：

- 关键交互 p95 不劣于 reference 10%；
- RSS/CPU/energy 在同场景不劣于 reference 10%；
- 受控传输吞吐不低于 reference 90%；
- 无持久 busy loop；
- 24h soak 无无法解释的内存、线程、handle 或任务增长。

更好的 Rust 架构应尽可能优于 reference，但“优于”必须以测量证明。

### 13.2 安全

不记录 auth key、password、payment/Passport secrets、消息正文等敏感数据。所有网络、媒体、URL、本地数据库、WebView/IPC 输入视为不可信，设置长度、递归、解压、并发和资源上限。

unsafe 限于可审计边界，记录 invariants、owner、线程规则和测试。

### 13.3 UI

目标是功能与交互兼容，不要求内部 widget 架构相同。视觉测试应在固定环境用几何、文本、状态和预先定义的图像阈值比较，不能因字体抗锯齿差异要求无意义的逐像素相同。

IME、RTL、screen reader、keyboard-only 等语义测试不能被 screenshot 替代。

## 14. 许可证与 provenance

本项目明确阅读并研究 Telegram Desktop GPL 源码，因此：

- 这不是 clean-room；
- 不得在 Spec、README、营销或发布材料中宣称“因为换成 Rust 所以自动摆脱 GPL”；
- 不直接复制长段源码、注释、资源或非必要结构到目标实现；
- 对研究过的 capability 记录 upstream commit/path 作为 provenance；
- 对实际复制、翻译、改编或复用的内容记录来源和许可证；
- 第三方依赖逐项保留许可与 notice；
- 发布前由负责许可证/法律审查的人判断最终分发义务；
- 若希望采用与 GPL 不兼容的分发方式，必须先取得明确的权利依据或改走真正隔离的 clean-room 项目。

根 upstream LEGAL 的 GPL-3.0-or-later 和 OpenSSL linking exception 继续作为研究基线记录。语言变化不是重新许可依据。

## 15. 实施阶段

### P0 — Source understanding inventory

目标不是创建逐文件 port ledger，而是建立完整 research coverage：

- 固定 root/submodules/downloads/generators/resources；
- 给每个源文件分类到一个或多个 capability；
- 建 capability graph；
- 建 license/provenance inventory；
- 标出 C++ production logic；
- 输出哪些源码尚未被研究，而不是“哪些文件尚未 port”。

退出条件：没有未分类源叶；所有 C++ 生产区域都归属 capability；coverage 可重建。

### P1 — Architecture foundation

- 完成 core architecture ADR；
- GUI backend ADR；
- storage ADR；
- concurrency/actor ADR；
- protocol/runtime ADR；
- platform boundary ADR；
- language/dependency decision policy；
- 建最小可执行骨架但不做空壳完成宣称。

### P2 — Protocol/auth/sync vertical slice

从登录开始，完成真正 Rust 的 TL/MTProto/auth/session/update/storage/recovery。

### P3 — Messaging vertical slice

dialogs → history → composer → send → server update → durable local state → restart recovery，使用真实 Rust UI。

### P4 — Product breadth

groups/channels/topics/search/settings/notifications/theme/language/privacy 等。

### P5 — Media and calls

上传、下载、播放器、录制、calls、screen sharing、设备和网络异常。

### P6 — Long-tail capabilities

Bots/Mini Apps/Stories/Premium/Stars/business/payments/Passport/WebAuthn/export 等。

### P7 — Platform and release

Windows/macOS/Linux、最低版本、安装/更新/回滚、signing、performance、power、accessibility、security、soak。

### P8 — Cutover

移除全部 C++ Telegram production dependency、temporary bridge 和重复 owner；执行完整 capability acceptance 和独立验收。

## 16. CI 与执行环境

所有构建、生成器、lint、schema、测试、benchmark、fuzz、packaging 和 acceptance 只能运行在 GitHub Actions 或 htch-runtime。

禁止在用户 Mac/Windows、本地工作站或助手本地容器运行这些任务。

缺 runner、旧 OS、设备、账号、证书或服务端资格时，真实场景是 blocked/not-configured，不是假通过。

每个验收证据必须绑定：

- exact target commit；
- fixed upstream commit/lock digest；
- workflow run/job/step 或 htch-runtime device；
- command；
- exit code；
- executed test count；
- platform/config；
- artifact ID/name/hash；
- limitation。

正式 release 验收运行 exact SHA 的 packaged artifact，不用重新现编的包代替。

## 17. Acceptance Criteria

### AC-01 — Research coverage
完整上游源闭包、子模块、资源、生成链和外部下载都已分类到 capability 或有明确不相关理由；不存在未知区域。

### AC-02 — Architecture independence
目标架构由 ADR 定义，dependency direction、state ownership、process/thread boundaries 清晰；不存在为了 source mapping 建的无意义同名模块。

### AC-03 — C++ replacement
最终产品中 Telegram/desktop-app C++ production logic 为零；所有对应能力由 Rust production owner 执行。没有 Qt/TDLib/原 tdesktop/C++ helper fallback 冒充完成。

### AC-04 — Best-fit non-C++ choices
每个非 Rust 生产组件都有职责、语言和 dependency ADR，证明它比强行 Rust 更合适且没有创建第二业务 owner。

### AC-05 — Functional completeness
FR-01…FR-14 全部适用 capability 有真实行为证据，无未解释功能缺口。

### AC-06 — Protocol correctness
MTProto/TL/auth/session/update/send/retry/recovery 与 Telegram 服务和参考客户端真实互通。

### AC-07 — Data safety
多账号、退出、崩溃、磁盘满、损坏、迁移、升级和回滚不会产生不可恢复的用户数据损失或跨账号泄漏。

### AC-08 — UI quality
完整 UI、IME、RTL、accessibility、DPI、多屏、keyboard、system integration 通过目标平台矩阵。

### AC-09 — Media/calls
媒体和通话能力在真实设备/网络异常下通过，不依赖原 C++ 业务实现。

### AC-10 — Performance and power
达到第 13 节基准，不通过削功能或停止后台必要行为作弊。

### AC-11 — Security
威胁模型、fuzz/negative tests、unsafe/FFI/IPC/WebView/dependency 审查完成，无未处理的严重安全/隐私问题。

### AC-12 — Licensing
source-informed provenance、GPL/第三方/asset/API/brand/signing obligations 有明确审查结论；没有“Rust 等于自动重新许可”的错误假设。

### AC-13 — Exact-head evidence
新 exact target HEAD 的 GitHub Actions/htch-runtime 证据和 packaged artifact provenance 完整；不得沿用旧 HEAD 或其他仓库绿灯。

### AC-14 — Independent acceptance
capability ledger 全部 accepted，零 temporary C++ bridge、零未知/未批准 blocker，canonical main 重验通过，由独立验收记录确认。

## 18. 当前状态

本 Revision 只改变实施方法，没有实现 Telegram 客户端。

当前状态：

- source baseline：已冻结根 commit 和首批直接 gitlink，递归闭包仍需 P0；
- source understanding：未完成；
- target architecture ADR：未完成；
- C++ replacement：未开始；
- product capability：blocked；
- packaged acceptance：blocked；
- release：blocked。

不得因为 Spec 从“逐文件迁移”改成“重新架构”而把任何旧 port 百分比转换成新完成度。

## 19. 下一动作

下一轮先再次读取目标 main exact HEAD。若已改变，基于新 HEAD 重建文档基线。

随后执行 P0，但 P0 的产物改为：

1. 完整 source/dependency/resource inventory；
2. C++ production-logic inventory；
3. capability graph；
4. source → capability research coverage；
5. 第一批 research dossiers；
6. architecture questions/risk register；
7. capability ledger；
8. P1 所需 ADR backlog。

P0 不创建 source → target 文件映射，也不以“每个 C++ 文件对应一个 rs 文件”为目标。

首个实现 vertical slice 固定为：认证 → MTProto/session → updates → storage → dialogs/history → compose/send → server update → restart recovery。只有该闭环通过后再扩大页面和长尾功能。

## 20. References

- 用户 2026-10-02 最新要求：阅读源码、理解后重新设计更好的架构；C++ 全部用更好的 Rust 取代；其他部分理解职责后使用最好的架构和语言实现。
- Fixed upstream: telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677
- Upstream LICENSE / LEGAL and .gitmodules at the same commit
- Root AGENTS.md
- docs/specs/spec-first-ai-development.md
- projects/telegram-desktop-rust/upstream.lock.json
- projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md

本 Revision 取代 Revision 1 中所有“逐文件/逐模块目标实现映射”“目录级 source parity”“目标结构镜像上游”的要求。仍保留完整源码研究、功能覆盖、provenance、许可证和行为验证要求。