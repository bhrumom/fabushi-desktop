# Fabushi 全量迁移执行规范：先完成融合，后集中验证

Status: active / normative execution addendum  
Spec ID: FMEP-001  
Revision: 1  
Updated: 2026-10-31  
Applies to: FBCP-001 / TDRP-001 full migration  
Product requirements: FBCP-001 Revision 8 / TDRP-001 Revision 10 and later applicable revisions  
Implementation/acceptance status: not asserted by this document

## 1. 用户目标与优先级

本规范落实最新要求：**全量迁移融合完成前不进行任何迁移 CI，全部开发完成后冻结候选，再统一验证。** 优化目标是减少重复上下文读取、串行等待、重复构建和无效返工，不承诺延后测试必然带来最短总工期。

本文件只调整执行时机、协作、进度状态和证据获取顺序，不删减完整源责任、现有 Bot 能力、统一架构/UI、服务端职责、安全、许可证或最终验收范围。它细化并优先解释 FBCP-QA-19 / TDRP-EXEC-01；历史任务、质量计划或提示词中与其冲突的“逐模块先跑 CI 才能继续”“每次提交立刻重跑”“P0 必须先取得 Actions 结果才可实现”一律延后至阶段 B。root AGENTS 的无关项目规则不变。

统一流程：`A 全量实现与融合（零 CI） -> B 冻结候选并集中验证/修复 -> C 独立接受与发布`。

## 2. 三种完成概念必须分开

- **实现进度完成**：按源码审读，适用责任已写入实际生产 owner，接口/调用/路由/数据迁移已接入；尚无运行证明。
- **候选可验证**：全范围实现盘点结束、测试/用例/数据/配置已备齐，并冻结提交 H；不要求测试先通过，否则造成循环前置条件。
- **迁移验收完成**：H 对应完整源码范围、生产能力、UI、服务与全部适用验收通过，独立结论为 ACCEPT。

阶段 A 保持每项 `validation_status=not-run` 或明确的既有历史状态，`accepted=false`。不得凭代码审读把 baseline_ready、verified 或 release readiness 改成 true。原有账本沿用；新阶段字段先按既有 schema 兼容扩展，禁止创建第二套状态真相。本文不修改任何机器账本的当前值。

## 3. 阶段 A：全量实现，严格零 CI

允许：读取 Git/API/tree/blob/源码与差异；逐文件语义理解；编辑 production、schema、生成输入、测试、fixture、workflow；人工静态审查；最小 ADR；保存 Git 检查点；更新唯一 inventory/ledger/RTM。

禁止：触发、排队、轮询等待、重跑任何本迁移 CI；workflow_dispatch/repository_dispatch；借 push/PR/标签/评论/定时器间接触发；运行 build、typecheck、lint、schema validator、测试、E2E、截图验收、benchmark、fuzz、打包、签名、部署或安装验证。不得把这些搬到本地、Mac、Windows、bhrum2、htch-runtime、assistant container、隐藏 runner 或其他平台规避。人工源码审查不是运行静态分析器的许可。

前期**写测试，不跑测试；写验证工具，不运行验证工具；写生成输入，不要求预先生成构建产物；审查 UI 代码，不声称 UI 已运行验收。** 编译产物、真实服务部署与安装后证据属于阶段 B，不是阶段 A 开始实现的前置条件。

## 4. 高效迁移的工作组织

**FMEP-FAST-01 — 一次建立范围，增量维护。** 开工读取 live main、现有迁移分支/PR 与 source lock。固定一个 exact upstream U 作为本轮范围；只读解析完整递归来源，不先启动 Actions 做 acquisition。后续只因明确批准的 rebaseline、必要安全修复或事实冲突更新 U，不因上游每天有提交就不断重开全量迁移。

**FMEP-FAST-02 — 以目标 owner 成批工作。** inventory 仍逐文件/逐责任覆盖，但实现按现有 owner 和依赖图分批。相关文件一起读、一起改、一起接线，避免每一个源文件都创建一套目标模块、PR、长报告或 CI。

**FMEP-FAST-03 — 共享接口先定型。** 先落实身份、会话、消息/entry、资源、权限、命令/事件、错误、版本、取消和持久化合同，再并行推进消息/群频道、资料、搜索、媒体通话、插件、UI 和平台。不能以接口未定为由无限写研究报告。

**FMEP-FAST-04 — 并行工作，单一集成人。** 有可用 worker 时按可分离 owner/path 分配任务；每项带 exact base、允许修改路径、输入/输出合同和回传 diff。共享 schema、依赖锁、公共 composition 和账本由单一集成人串行合入；不可同时覆盖同一文件。不复制完整仓库到用户设备，不为每个 worker 开新修复线。无 worker 时按相同队列顺序执行，不虚构并行完成。

**FMEP-FAST-05 — 不重复已完成阅读。** source blob、调用合同和依赖均未变化时复用已有理解与映射；相关变化才补读受影响范围。复用阅读结论不等于复用旧运行证据。

**FMEP-FAST-06 — 文档最小充分。** 单一清单、单一责任账本、单一 RTM，差异记录与批次摘要代替重复复制全文。新增能力必须有足够行为/边界/去向说明，但不能陷入只补 dossier 而不改生产代码的循环。

**FMEP-FAST-07 — 阻塞分流。** 记录访问/账号/签名/服务阻塞及解除条件，立刻推进无依赖任务；需要用户支持时先查 Gmail 已发通知与回复，再向 1315518325@qq.com 发一封必要通知，未解决不重复发送。不得为了绕过阻塞删功能或宣称闭合。

## 5. 云端保存与零 CI 不能互相冲突

默认继续现有单一迁移分支；未验收实现不提前合并或发布 main。不得使用 force push、覆盖他人修改或新开重复修复线。

“前期不推送实现分支”应理解为：**禁止导致 CI 的 ref/PR 更新，不是要求所有代码永远只存本机。** 在写入前核查所有相关触发路径，证明不会触发 CI 的受控云端保存才允许：已有明确 CI 隔离的开发 ref/远端存储；必要时对该迁移范围建立已授权、可恢复的触发隔离。该隔离不能通过先启动一个 CI job 再退出实现，也不能移除最终 required checks。

`[skip ci]` / `skip-checks: true` 只可作为适用 push/pull_request 的额外保险，不能覆盖其他事件。draft PR 也不是零 CI 保证；job-level skipped/success 不能冒充测试通过。不得全库停用无关 workflow、安全机制、ruleset 或正在进行的其他任务。

若暂时无法证明实现 ref 更新不会运行 CI：保留已生成 diff/commit，通过可用且经确认不会触发 CI 的持久云端备份保存；记录远端对象/校验身份与恢复方法。无人引用的 Git blob/commit 不能当成长期备份。没有可用保存通道则记录工具阻塞、按用户通知规则求助，继续可推进的源码/设计工作，不把未保存改动删除。

文档政策提交同样采用零 CI 保存纪律。本文件仅规定机制，不表示仓库已完成自动触发隔离配置。

## 6. 逐责任迁移仍然完整

每一 tracked file、gitlink、递归依赖、构建期下载/patch、LFS、生成输入、资源、工具链和法律记录有精确身份及处置。逐文件完整理解，逐 responsibility 映射到现有 Fabushi owner；允许多对一或一对多，不机械复制目录。

C++ 非 UI 产品逻辑默认 Rust；React/TypeScript 负责统一 UI 投影；平台差异用最薄适配。原 Qt 绘制可替代，但混合 UI 文件的业务/权限/键盘/生命周期不可省略。缺 owner 才以最小 ADR 新增 source-neutral owner；不保留 Telegram C++ runtime/TDLib/sidecar 作为业务 owner。

已有消息、联系人、搜索、资源、群/频道、权限等功能是升级原 owner，不是新增另一套产品。Human/Agent/Group/Channel 使用统一会话框架，差异只在 typed capabilities/sections/renderers/actions。既有 Bot 的 Host/Coordinator/Runner、Tools、审批、任务、产物、Computer、Plugins/MCP、Automations 不得丢失。

## 7. 阶段 A 到 B 的非运行冻结检查

通过人工源码/差异盘点确认：

1. 当前固定 U 的完整来源已获得并阅读/分类，所有适用职责有去向；未解决来源或实现阻塞保持可见，不得宣布全量完成。
2. 所有适用 production responsibilities 已实现并进入真实 shipping 调用链；不是 stub、仅 UI、mock 或未接线 facade。
3. UI 的统一布局、创建、搜索、标准组件/状态、品牌，以及服务、存储、迁移、平台和打包输入已实现；外部配置缺失单列阻塞。
4. 单元/属性/合同/集成/功能/UI/时序/安全/性能/恢复等测试代码与 oracle、fixture、最终运行计划已备齐，执行仍为 not-run。
5. 正式生成/构建/部署由阶段 B 完成；不得为了满足上述清单提前执行。
6. 冻结 exact target H、source U、依赖/工具链锁、测试配置、环境矩阵及内容摘要。阶段声明为 `candidate_ready_for_validation`，不得为 ACCEPT。

此处要求的是可审计的实现闭合，不是运行正确性的证明。`verified=0` 的新实现可以进入集中验证；不得要求先全部 verified 才允许第一次 CI。

## 8. 阶段 B：统一验证，但不是只允许运行一次

全部开发闭合之后，才启动一个绑定 H 的集中验证 campaign。最终测试清单保留所有适用 gate，不因延期删减。

按依赖图组织：先 source/inventory/RTM/编译和低成本合同验证；通过必要前置后，并行运行独立 unit/property/integration、UI/visual/a11y、功能/temporal、security/privacy、fault/recovery、performance/soak 与支持平台打包/真实服务验收。每个平台/构建配置构建一次，在同一 H 和相同输入范围内复用不可变构件，不能拿一个 OS 的包替另一个 OS 验收。

缓存只优化依赖与编译成本，键必须含工具链、依赖锁及 relevant configuration；不得把旧测试结果当新通过。阶段 B 的所有可执行工作仍只在 GitHub Actions。

无需空等 Actions：读取已完成 job 的结果，归类失败，审阅 trace/截图/视频，准备可分离修复；不得偷偷改变正在验收的 H。

## 9. 失败恢复与最终候选证据

区分 production bug、测试/fixture/oracle 缺陷、workflow/environment 故障。修根因，不删断言、不自动接受坏截图、不把失败改成功、不以无解释重跑挑绿结果。

同一 H、同一测试与环境输入的有效已通过结果可以在本 campaign 中保留，仅重跑失败/缺证项。改了 source/test/oracle/environment 后冻结 H2，保留旧失败记录，先做受影响项回归；未受影响旧结果只能附带影响分析作为参考，不能标成在 H2 执行。

修复收敛后，最终候选 Hfinal 必须取得全部适用 required gates 的完整 exact-head 结果。若合并产生新的 shipping SHA，必须对实际 shipping SHA 重新完成必要验证并绑定安装包 provenance，不能把 PR head 结果伪装为 merge SHA 结果。

“统一验证”指开发后进入集中验收阶段，不是只能运行一次、只允许一次修复或失败就放弃。

## 10. 最终正确性与 UI 验收保持不变

必须覆盖复杂 Agent turn（中间段 -> tool -> 最终输出）、消息顺序、optimistic/ack/baseline/reconnect、双会话隔离、取消和重启。至少取 terminal、+2s、+8s、切回、重连/重载及重启的语义/DOM/存储对应证据；有限时间窗不能替代状态机和乱序故障场景。

一个 logical turn 有唯一终态决策；其用户可见结果可由合法的多段消息/产物组成，不能用“只剩一个气泡”掩盖丢内容。最终输出不得因旧 snapshot 回滚；显式授权编辑/删除/保留策略按各自合同验证，不等同于缺陷。

UI 同时验功能、截图、布局、焦点/键盘、IME、RTL、中文/英文、浅深色、响应式、大数据和状态连续性；采用既有设计系统与 oracles。视频/截图必须实际审阅且留结论；无人审阅不记 pass。安全与隐私验收只针对授权测试环境，记录脱敏证据。

## 11. 发布和状态

required tests 未运行、blocked、flaky、缺来源/服务/签名、缺独立评审均不算通过。阶段 A 可以持续前进，但不能发布；阶段 B 失败也不能发布。最终 `source completeness + production implementation + full applicable quality evidence + independent ACCEPT` 才能宣称完整迁移完成。

每轮仅报告 exact base/working head、固定 U、已完成责任及接线路径、待实现/阻塞、保存位置和下一批。阶段 A 不要求 CI 链接、不重复解释没有 CI；统一标 not-run。不编造进度百分比、测试结果、并行 worker 或后台持续运行。

## 12. 配套提示词与官方触发说明

完整执行提示词：`projects/fabushi-communication-platform/management/prompts/full-migration-deferred-ci.md`。它取代旧的逐模块运行 CI 总提示词；产品/质量范围以现行 FBCP/TDRP 为准。

触发规则核对来源（2026-10-31）：GitHub 官方 Skipping workflow runs、Disabling and enabling a workflow、Troubleshooting required status checks。跳过 push/pull_request 不覆盖其他事件，跳过 required checks 可能阻塞合并；这正是保留最终合并门而不提前伪造绿色的原因。

- https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs
- https://docs.github.com/en/actions/how-tos/manage-workflow-runs/disable-and-enable-workflows
- https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks

## 12. 产品责任优先与生产归属收口（2026-11-02）

本节落实用户要求：修正迁移方式并合并为 AI 执行依据。它细化阶段 A 的工作选择与实现判定，优先于旧提示词中“最早未闭合文件优先”的机械顺序；完整来源范围与最终验收标准不变。

### FMEP-OWNER-01 — 当前生产图决定目标

PR #20 是架构沿革，不是永久冻结的目标源码。每批绑定实际 base/working head，从 shipping entrypoint 跟踪 UI → command → owner → state/storage → service → event/projection；在既有 current-owner-inventory 和 owner-resolution-matrix 中增量记录路径、符号、权威状态、持久化、权限与调用者。旧 inventory/STATUS 只作有日期的历史证据；不得因旧表写“无 owner”直接新建。只补查当前批次和公共边界，不把全仓盘点反复作为开工前置。

### FMEP-OWNER-02 — 同一语义只有一个权威写入者

首先审计桌面 Session/Transcript/native_messaging、native/mahayana-messaging、FeatureHost MessagingService 和旧 Telegram Provider/FFI 的实际关系。目录名、Cargo 成员、存在测试、注释“desktop uses”均不能证明 shipping reachability。记录每个支持平台/构建配置的入口、注册条件和依赖路径；区分可调用、实际选用、仅研究/历史。

为 Identity、Conversation、Message、Resource、Permission、Search 明确唯一逻辑权威与写入入口；允许服务端权威、本地 durable outbox、离线状态、缓存和 UI 投影分层，但必须定义 ID/版本、幂等、排序、权限、冲突、重放和重启的合同，不要求所有状态挤进一个进程或文件。禁止仅通过重命名隐藏第二 owner，或让同一行为分别在桌面路径和孤立领域库重复实现。领域库只有经真实调用链复用后才计入产品接线。

### FMEP-OWNER-03 — 旧路径有明确退出条件

Telegram core/runtime/provider/network/storage 不作为本迁移的 shipping owner。先识别所有调用者、平台依赖和存量数据，再将适用行为迁入选定 Fabushi owner，补齐版本化数据迁移、失败恢复与回滚方案，最后移除旧注册、默认依赖和产品入口。不能因名字含 Telegram 就直接删库或用户数据，也不能因“遗留代码”无限保留第二运行路径。法律声明和研究 provenance 保留。若另有明确授权的互操作规范，单独标明适用范围，不把互操作算作原生融合完成。

### FMEP-FAST-08 — 完整核算，产品批次驱动

所有源文件继续完整理解并有处置。产品行为、构建/平台/安全/许可证必要支撑、上游开发辅助工具分别记账；沿用一个 inventory/ledger/RTM，用兼容字段或既有 notes 标注类别，不建第二套真相。

上游 .agents、审查提示词、代码健康工具不是 Fabushi 产品需求的自动来源。理解后应说明复用现有机制、必要适配或经证据说明不适用；不能仅因源仓有该文件就改 root AGENTS、复制治理体系或制造 Rust 对应物。已有明确必迁责任不得偷偷改成不适用；冲突必须在当前规范/ADR 中解决。工具工作仅在解除具体产品、构建、安全或最终验收阻塞时优先，并指出被解锁的能力。

按依赖、用户价值和风险选择下一批，不按文件名字典序：先权威模型与生产链收口，再消息/同步/恢复与统一 UI，再群频道/权限/搜索、资源媒体/通话等可并行域；Stories、商业、平台等全部适用职责仍必须完成。每批交付实际 owner 修改、调用接线、数据/错误恢复合同和已编写用例。纯阅读批次如确有必要，记录它解锁的下一生产批次，不冒充功能实现。

### FMEP-STATE-01 — 进度不得混算

分别报告来源理解、产品职责实现、shipping 接线、测试编写、执行与独立接受。治理文档条数不得进入产品完成率；分母尚未完整分解时不得给全产品百分比。单纯政策说明不能把可执行工具职责标 implemented；独立 helper 或领域模型完成但 UI/服务调用未接入时，保留局部实现说明和 wiring gap，不标整个产品责任 implemented。历史行逐项纠正且保留原记录，不批量升级状态。

每批更新既有 STATUS 和 ledger 的相关项、exact head 与未决问题；计数、表头 revision、行状态不一致时明确修正来源。未执行仍 not-run；本规范变更不修改任何产品验收值。

### 首个生产批次与退出标准

下一实现批次从 OWNER-01/02 的三条消息路径及旧 Telegram 注册开始，复用现有迁移线。交付一份写入既有 owner matrix 的路径/权威/调用者差异，以及相应生产收口实现和用例；禁止再次只扩充治理报告。发现跨仓服务依赖时同步追踪服务 owner 和接口版本，缺少源码或权限明确阻塞，继续其他独立能力。

阶段 A 人工审查必须能回答：实际用户入口在哪里；哪个 owner 决策；什么状态被持久化；谁发送/同步；谁消费事件；失败/重启怎么恢复；旧路径何时退出。缺一项则保留该项缺口。阶段 B 再在 Actions 证明支持配置无旧 Telegram 原生路径、同一消息跨 UI/服务/持久化的 ID 与状态一致、重试重启及数据迁移可恢复、现有 Agent 能力不回退。不得提前运行验证或把政策合并当产品完成。

本次政策交付审查：基于 `6f770fb00`，仅修改 Markdown 规范与入口；人工复核范围、状态分层、生产路径判定、安全退出及阶段 A/B 约束。审读该基线全部 `.github/workflows` 事件：本次 ref/PR/合并可触发的自动事件仅 push/pull_request，未发现 pull_request_target/workflow_run/schedule 链；其他入口为显式 dispatch/call。本次分支提交与最终合并提交均带 `[skip ci]`，与该事件清单共同构成此次文档保存的零 CI 条件，不推广为未来提交的通用保证。不修改 workflow/保护规则，不运行测试或构建，不改变产品实现状态。后续发布前若 base/workflow 变化须重查；文档回滚使用普通 revert 并保留历史证据。
