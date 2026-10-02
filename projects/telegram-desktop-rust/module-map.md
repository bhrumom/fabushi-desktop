# Telegram Desktop → Rust 初始模块对应表

Status: planned, not implemented  
Date: 2026-10-02  
Source: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
SourceFiles tree: `6e29a19ac5e3f19bf72842c29dc1447c842ef97f`

必须同时阅读[主 Spec](../../docs/specs/telegram-desktop-rust-equivalence-migration.md)与[执行约束](SOURCE_OF_TRUTH.md)。本表根据已读取的真实目录结构提出迁移责任划分，**不是对所有源函数完成分析的证明，也不是逐叶文件/符号的验收台账**。列出的职责须在 P0/P1 读取源码后精化；任何未覆盖责任仍在完整范围内。

表中 `fd-*` 均指规划路径 `telegram-rs/crates/fd-*`。`UI/domain/API` 的拆分只在职责确实不同并有符号映射时成立，每个状态/用例/副作用仍只能有一个生产 owner。多列 crate 不授权多个实现共同写同一状态。

## A. Telegram/SourceFiles 的真实顶层模块

| ID | 上游相对目录 | 规划 Rust 责任/路径 | 主要门禁 |
| --- | --- | --- | --- |
| TDM-001 | `_other/` | P0 逐文件分类；schema/辅助素材进入 `fd-tl`、`fd-resources` 或 `tools/xtask`，不整目录忽略 | FR-02/17；生成输入、文件格式和引用闭包 |
| TDM-002 | `api/` | `fd-api/src/api/`；业务 RPC、返回结果及领域更新的唯一适配入口 | FR-02/03/05；请求取消、错误、去重和更新顺序 |
| TDM-003 | `boxes/` | `fd-ui-pages/src/boxes/`；弹窗/表单交互；业务命令交由 application | FR-05/13/15；焦点、验证、确认/取消和辅助技术 |
| TDM-004 | `calls/` | `fd-calls/src/`；通话状态/协商，UI 投影在 `fd-ui-pages/src/calls/` | FR-09；真实互通、设备/权限、断网与挂断清理 |
| TDM-005 | `chat_helpers/` | `fd-ui-pages/src/chat_helpers/` 与明确分离的 domain helpers | FR-05/08/15；输入、建议、emoji/贴纸及生命周期 |
| TDM-006 | `codegen/` | `tools/xtask/src/codegen/`；项目自有生成器逻辑迁移 Rust | FR-17；同输入/参数的输出及错误合同 |
| TDM-007 | `core/` | `fd-core/src/core/`，组合/启动监督进入 `fd-app` | FR-01/13/16；启动关闭、配置、恢复、资源释放 |
| TDM-008 | `countries/` | `fd-resources/src/countries/` 与账号输入相关纯映射 | FR-01/15；语言、号码显示、数据版本一致 |
| TDM-009 | `data/` | `fd-domain/src/data/`；实体、索引、领域状态及变更 | FR-03/06/11；字段、identity、顺序和订阅一致性 |
| TDM-010 | `dialogs/` | `fd-domain/src/dialogs/` 管理列表状态；`fd-ui-pages/src/dialogs/` 渲染 | FR-04/15；分页、文件夹、排序、滚动及实时更新 |
| TDM-011 | `editor/` | `fd-media/src/editor/` 处理变换，`fd-ui-pages/src/editor/` 处理交互 | FR-07/15；编辑结果、撤销、坐标/DPI、取消 |
| TDM-012 | `export/` | `fd-application/src/export/` 编排；独立格式 writer 与 storage/media 端口 | FR-14；输出格式、完整性、权限、取消/续作和空间不足 |
| TDM-013 | `ffmpeg/` | `fd-media/src/codec/`；项目包装/调度逻辑 Rust-owned | FR-07；格式/seek/坏帧/线程；FFmpeg 依赖本身单独处置 |
| TDM-014 | `history/` | `fd-domain/src/history/` 与 `fd-ui-pages/src/history/` 保留子域形状 | FR-05/15；全部消息变体、view/model 生命周期、selection |
| TDM-015 | `info/` | `fd-ui-pages/src/info/`；实体资料和导航；状态来自 domain | FR-06/15；分页、权限变化、导航返回/焦点 |
| TDM-016 | `inline_bots/` | `fd-application/src/inline_bots/`；结果/发送由 API，界面由 UI | FR-10；查询竞争、旧响应取消、结果与权限 |
| TDM-017 | `intro/` | `fd-application/src/auth/` 为状态 owner；`fd-ui-pages/src/intro/` | FR-01；完整认证、失效、恢复、账号切换 |
| TDM-018 | `iv/` | `fd-ui-pages/src/iv/` 与受限内容模型/渲染层 | FR-10/15；页面内容、链接、导航、安全/资源边界 |
| TDM-019 | `lang/` | `fd-resources/src/lang/`；语言加载/选择与本地化合同 | FR-15；复数、参数、RTL、缺失回退和动态更新 |
| TDM-020 | `layout/` | `fd-ui/src/layout/`；布局算法与命中/测量合同 | FR-15；字形/行高/缩放、RTL 和尺寸边界 |
| TDM-021 | `main/` | `fd-application/src/main/`；账号/session 领域组合；程序入口在 fd-app | FR-01/03；session epoch、唯一 owner、事件生命周期 |
| TDM-022 | `media/` | `fd-media/src/media/`；传输/播放/录制/缓存/设备策略 | FR-07/08；吞吐、取消、续传、流式播放与清理 |
| TDM-023 | `menu/` | `fd-ui-pages/src/menu/`；菜单语义，原生菜单适配在 platform | FR-13/15；可见性/权限、快捷键、焦点与触发次数 |
| TDM-024 | `mtproto/` | `fd-mtproto/src/`；协议会话、transport、鉴权和请求状态 | FR-02；wire 字节、ack/salt/时钟/重试/DC/proxy |
| TDM-025 | `overview/` | `fd-ui-pages/src/overview/`；媒体/消息集合投影 | FR-04/07/15；分页、过滤、更新、选择与导航 |
| TDM-026 | `passport/` | `fd-security/src/passport/` 敏感数据 owner；专用 application/UI | FR-12；密钥/数据保护、上传授权、失败/撤销 |
| TDM-027 | `payments/` | `fd-application/src/payments/`；支付操作单 owner，服务端结果权威 | FR-11/12；幂等/不确定结果、取消、权益/收据，不可假成功 |
| TDM-028 | `platform/` | `fd-platform/src/{windows,macos,linux}/`，具体子目录以源码为准 | FR-16；每个平台分支、最低系统和实际系统服务 |
| TDM-029 | `poll/` | `fd-domain/src/poll/` 与 `fd-ui-pages/src/poll/` | FR-05；创建/投票/撤销/关闭及可见性权限 |
| TDM-030 | `profile/` | `fd-ui-pages/src/profile/`；资料编辑/展示和导航 | FR-06/13/15；本人/他人资料、编辑状态、权限 |
| TDM-031 | `settings/` | `fd-application/src/settings/` 存取用例与 `fd-ui-pages/src/settings/` | FR-13；全部设置、即时/重启生效和失败恢复 |
| TDM-032 | `statistics/` | `fd-application/src/statistics/` 与 `fd-ui-pages/src/statistics/` | FR-06；角色、数据/图表/时间范围、加载失败 |
| TDM-033 | `storage/` | `fd-storage/src/` 和 `fd-tdata/src/` 明确区分事务 owner/外部格式 | FR-14；加密、崩溃、schema、清理、备份/回滚 |
| TDM-034 | `support/` | `fd-application/src/support/`；实际角色/功能待逐源审查 | FR-13/17；特定账号配置不能默认为无用删除 |
| TDM-035 | `tde2e/` | `fd-security/src/tde2e/`；先读取协议用途/调用者，不据名字推断 Secret Chats | FR-09/12；与原实现相同的安全合同和真实适用场景 |
| TDM-036 | `test/` | 对应 `telegram-rs/tests/` 的原用途，P0 识别 fixture/工具/测试入口 | FR-17；测试注册和实际执行，零测试不能成功 |
| TDM-037 | `tests/` | `telegram-rs/tests/`；逐测试迁移语义并增加边界反例 | FR-17；原断言含义不弱化，测试列表/计数可核对 |
| TDM-038 | `ui/` | `fd-ui/src/` 及业务控件对应 `fd-ui-pages/src/ui/` | FR-15；渲染/输入/可访问性/动效，不保留原 Qt 业务实现 |
| TDM-039 | `webauthn/` | `fd-security/src/webauthn/` 与 platform 认证器适配 | FR-01/12；origin/challenge/权限、取消、设备和凭据保护 |
| TDM-040 | `window/` | `fd-ui-pages/src/window/`；窗口导航/视图策略，OS 句柄由 platform | FR-15/16；多窗/多屏/恢复/关闭/托盘与系统生命周期 |

所有行当前状态为 `planned`（模块规划状态，不是文件台账状态），没有行 `accepted`。P0 应自动对比 SourceFiles 实际顶层目录集合与本表，发现新增/遗漏必须显式补表，不静默跳过。

## B. SourceFiles 顶层文件

`.h` 与 `.cpp` 必须分别登记独立 file ID，以下分组仅便于阅读，不能合并源文件计数。

| ID | 源文件 | 规划责任与专项验收 |
| --- | --- | --- |
| TDF-001 | `apiwrap.h`, `apiwrap.cpp` | `fd-api/src/apiwrap/`；巨大协调文件须按符号追溯拆分，保持原 RPC/领域更新职责，不建立第二个全局 owner |
| TDF-002 | `config.h` | `fd-core/src/config/`；常量、feature 和条件编译逐项锁定 |
| TDF-003 | `logs.h`, `logs.cpp` | `fd-diagnostics`；错误分类、轮转、线程/脱敏和关闭合同 |
| TDF-004 | `main.cpp` | `fd-app/src/main.rs`；启动参数/退出码/平台入口，不调用原可执行文件 |
| TDF-005 | `mainwidget.h`, `mainwidget.cpp` | `fd-ui-pages/src/mainwidget/`；UI 与用例拆开，所有响应分支和 ownership 显式映射 |
| TDF-006 | `mainwindow.h`, `mainwindow.cpp` | `fd-ui-pages/src/mainwindow/` + platform ports；原生窗体生命周期 |
| TDF-007 | `settings.h`, `settings.cpp` | `fd-core/src/settings/` 或已定义 settings owner；避免与 `settings/` 创建双份存储 |
| TDF-008 | `stdafx.h` | include/precompiled-header 依赖闭包与 Rust 构建依赖映射；不是 `.rs` 空文件迁移 |
| TDF-009 | `tray.h`, `tray.cpp` | platform tray adapter 与 Rust 托盘策略；通知/恢复/关闭 |
| TDF-010 | `tray_accounts_menu.h`, `tray_accounts_menu.cpp`, `tray_accounts_menu_dummy.cpp` | 多账号菜单与平台 dummy 条件分别验收；不能因文件名 dummy 删除必要回退行为 |

## C. 子模块与共享库

下列每个目录都按锁文件中的 repository + commit 递归展开，不把 gitlink 当作已完成的一个文件。子模块可能自带嵌套 gitlink、构建下载物和许可差异，必须继续展开。

| ID | 冻结挂载点 | 目标责任 / 处置约束 |
| --- | --- | --- |
| TDL-001 | `Telegram/lib_base` | `fd-core` 基础算法/类型/容器与 platform 边界；错误/时间/内存语义差分 |
| TDL-002 | `Telegram/lib_crl` | Rust 调度/线程/任务生命周期；不只将所有 API 换成 spawn |
| TDL-003 | `Telegram/lib_rpl` | Rust typed reactive layer；事件顺序、订阅销毁、同步/延迟、reentrancy 对照 |
| TDL-004 | `Telegram/lib_ui` | `fd-ui` 基础组件；原生 UI/backend 门通过后逐控件迁移 |
| TDL-005 | `Telegram/lib_lottie` | Rust-owned 动画和缓存/生命周期；原生第三方解码依赖无默认豁免 |
| TDL-006 | `Telegram/lib_tl` | `fd-tl`、生成器/codec；构造器和 flags 等协议合同 |
| TDL-007 | `Telegram/lib_spellcheck` | Rust 拼写检查策略、系统适配、语言/词典/权限；第三方引擎另审 |
| TDL-008 | `Telegram/lib_storage` | Rust 存储/缓存行为；与上层 storage 的唯一提交边界一致 |
| TDL-009 | `Telegram/lib_qr` | Rust QR 编解码/显示/输入合同；原始第三方 QR 实现独立入账 |
| TDL-010 | `Telegram/lib_webrtc` | `fd-calls`/`fd-media` 的 Rust 协商/设备包装；底层 RTC 依赖及 patches 逐项审计 |
| TDL-011 | `Telegram/lib_webview` | `fd-webview` 的 Rust 导航/权限/bridge 策略；允许 OS 系统 WebView，不允许整体业务 UI WebView 化 |
| TDL-012 | `Telegram/lib_translate` | Rust 翻译能力接口、身份/错误/重试/缓存/用户开关；数据流与当前 API 条款审查 |
| TDL-013 | `Telegram/codegen` | Rust 生成工具；输入 schema/resources → 输出的版本化可重建合同 |
| TDL-014 | `cmake` | 原构建选项/平台/依赖闭包 → Cargo/xtask/打包配置；逐脚本审查，不要求将纯声明内容强改 `.rs` |
| TDL-015 | `Telegram/ThirdParty/tgcalls`, `Telegram/ThirdParty/libprisma` | Telegram-specific 逻辑不得按 ThirdParty 目录位置自动豁免 Rust；先确认源责任再迁移 |
| TDL-016 | `Telegram/ThirdParty/MicroTeX`, `cmark-gfm`, `QR`, `TooManyCooks`, `cld3`, `libcbor` | 分别锁定算法/语法/格式/输出/限额；Rust 库替代须逐符号合同验证，不是 crate 存在即完成 |
| TDL-017 | `Telegram/ThirdParty/GSL`, `expected`, `range-v3` | Rust 类型/结果/迭代能力替代；生命周期、边界行为、复杂度和错误逐源映射 |
| TDL-018 | `Telegram/ThirdParty/lz4`, `xxHash` | 格式/哈希黄金向量、流式与溢出/极值，不能随意更换磁盘或网络兼容格式 |
| TDL-019 | `Telegram/ThirdParty/fcitx5-qt`, `hime`, `nimf`, `hunspell` | Rust 输入法/拼写系统边界；IME composition、焦点、词典与平台差分，不能永久保留 Qt UI |
| TDL-020 | `Telegram/ThirdParty/kcoreaddons`, `kimageformats` | 所用平台/图像能力逐件对应；未用文件也需证据处置，不默认免清点 |
| TDL-021 | `Telegram/ThirdParty/libfido2` | 认证器协议/OS 集成与安全能力；Rust替代或明确批准依赖，不能跳过真实认证器验收 |
| TDL-022 | `Telegram/ThirdParty/xdg-desktop-portal` | Linux portal 协议与受限系统能力；沙箱/权限/取消/设备恢复 |

上表是 35 个直接 gitlink 的分组导航；确切路径、URL、SHA 以 `upstream.lock.json` 为准。Qt、OpenSSL、WebRTC、FFmpeg、字体等**非根 gitlink 下载依赖**须在 P0 由构建脚本和补丁另行锁定；不在上述表里不代表不在项目范围。

通用原生依赖保留 FFI 的最终批准清单目前为空。所有例外遵守主 Spec RUST-03/04：明确范围/理由、版本许可、安全与 ABI、测试、维护者批准，单列非 Rust 部分；整块原客户端或项目业务逻辑不得成为例外。

## D. 非 SourceFiles 范围

| ID | 源范围 | 对应 / 完成条件 |
| --- | --- | --- |
| TDX-001 | `Telegram/Resources/` | `fd-resources` 索引及 assets；语言、主题、图标、字体、声音、动画等逐叶 hash/许可/转换与加载验证 |
| TDX-002 | `Telegram/shaders/` | 目标渲染后端着色器/变换；输入输出、颜色空间、像素/性能/软件回退；不强求 GLSL 文本改为 Rust |
| TDX-003 | `Telegram/Telegram/` | 实际资源/平台包结构逐源分析；不能因目录名重复忽略 |
| TDX-004 | `Telegram/Telegram.plist` | 自有身份下的 macOS bundle/权限/URL 等声明，逐 key 对照 |
| TDX-005 | 根/Telegram `CMakeLists.txt`、`Telegram/cmake/`、`Telegram/configure.*`、`Telegram/create.bat`、`Telegram/build/` | Cargo/xtask/packaging 逐选项和平台对应；生成器、下载/patch、版本/发布/更新等实际用途不遗漏 |
| TDX-006 | 根 `lib/` | 全叶清点并依据实际内容分类；不能未经检查视为外部依赖且排除 |
| TDX-007 | 根 `tools/`、`tasks/` | 开发/测试/构建/维护工具逐文件处置；可执行逻辑等价、文档保留/更新 |
| TDX-008 | `.github/`、`snap/` | CI矩阵/构建发布/沙箱打包等合同；独立目标身份、签名与源代码交付，不能直接复用上游凭据 |
| TDX-009 | `LICENSE`、`LEGAL`、`README.md`、`changelog.txt`、`docs/` | 许可原文/署名/对应源码文档；更新目标说明并保留来源；历史文本不能算作迁移逻辑 |
| TDX-010 | 其余根配置、dotfiles、`.agents/`、`.claude/`、`.grok/`、`AGENTS.md`、`CLAUDE.md`、`GROK.md`、`REVIEW.md` | 全部入账；参考开发材料可 `upstream-reference-only`，不把上游 agent 指令自动变成本仓库权威 |

## E. 跨目录的功能台账

不能从顶层目录缺少某个名字推断功能不存在。Stories、Stars/gifts、Premium/business、sponsored messages、自定义 emoji、文件夹、下载管理和隐私等能力可能分散在 data/api/history/ui/settings/boxes。P0/P1 必须建立跨文件的 `feature → source symbols → Rust owner → scenario tests` 图。它与逐文件清单互为补充；两者都不能单独声明全覆盖。

## F. 每个文件/符号在开工前必须回答

- 精确源 repo/挂载路径/commit/blob/符号/行区间是什么，哪些平台/feature 会编译或触达？
- 输入输出、状态读写、事件先后、权限、错误、取消、清理、重启和资源上限是什么？哪些是继承/模板/宏/生成内容？
- 原始调用者和最终副作用在哪里？Rust 唯一 owner、目标路径/符号、依赖方向是什么？
- 哪些行为可用 golden/差分验证，哪些必须真实 OS、账号、设备或安装包？如何证明生产入口确实调用它？
- 如何安全回滚？使用了什么许可/资产？是否产生临时 bridge 或未批准依赖？

答案与源片段链接进入文件/符号台账后才进入 `analyzed/mapped`。本表的相似名称、预计 crate 或阶段编号均不自动赋予上述状态。
