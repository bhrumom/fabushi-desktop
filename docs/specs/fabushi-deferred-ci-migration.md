# Fabushi 全量迁移执行规范：先完成融合，后集中验证

Status: active / normative execution addendum  
Spec ID: FMEP-001  
Revision: 2  
Updated: 2026-11-01  
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

**FMEP-FAST-08 — exact-blob 复用分流优先于全文重读。** 对剩余 source queue，读取正文前先以 `repository + fixed commit + path + exact blob SHA` 检查现有 inventory/dossier/ledger/evidence，并核对直接依赖 identity 与行为合同是否变化。每项必须归入三类之一：`reuse-ready`（exact blob、相关依赖与合同均未变化，复用已有完整语义阅读，只做当前 Revision/owner/implementation/wiring 重新绑定）；`delta-read`（source blob 可复用但当前规范字段、依赖、owner 或调用链有变化，只补读变化和受影响范围）；`full-read`（没有可信完整阅读、blob/关键依赖变化或旧结论不足，完整重新语义阅读）。不得把历史“曾打开/曾 inventory”误当成当前 closure，也不得把可复用的 exact blob 无意义从头重读。分流结果写入现有 ledger/evidence，不建立第二套永久进度真相。

**FMEP-FAST-09 — source entry 是最小追踪单位，不是最小执行/提交单位。** 每个 source entry 仍须有精确 identity、责任、处置与最终状态；但 Stage-A 默认执行单位是高内聚 owner-oriented batch。禁止因为账本逐 entry 追踪，就机械采用“一 source entry -> 一完整执行轮 -> 一 spec commit -> 一 closure commit -> 一用户报告”。只有责任真正独立、存在阻塞边界、并发冲突或风险要求隔离时才单 entry 成批。一个责任可以覆盖多个 source entries，一个 mixed source 也可拆给多个 owner。

**FMEP-FAST-10 — 批次按语义和 owner 定界，数量只是上限提示。** repository/process/docs/schema/resource/重复模板等低耦合来源，优先从 fixed-U 有序队列取连续候选窗口并按共同 owner/合同组成约 20–100 entries 的批次；若共同语义更强可更大，若风险/依赖边界更窄则更小。核心产品、协议、状态机、mixed UI 和跨服务代码不按数量硬凑，必须以完整调用链、共享状态生命周期和可独立接线的 owner slice 定界。不得为了批量化跳过 source identity、直接 consumers、failure/recovery、security/privacy 或 shipping wiring。

**FMEP-FAST-11 — reader lane 与 integrator lane 分离。** 有 worker 时，reader/worker 可并行处理互不重叠的 source ranges 或 owner slices，输出 exact source identities、复用分类、responsibilities、依赖、行为/状态/失败合同、owner candidates、tests/oracles 和 blockers；除明确分配的独占 owner path 外，不得同时修改共享 schema、公共 composition、唯一 ledger、P0、source-closure 或共享 evidence authority。单一 integrator 负责 owner-level consolidation、共享生产接线、公共文件和唯一账本写入。无 worker 时也按相同两阶段思路串行工作：先收集一个 batch 的分析/复用结果，再集中实现和闭合，不退化成一文件一事务。

**FMEP-FAST-12 — authority context 按 blob SHA 缓存。** 首次建立当前 revision 的 root/nested instructions、FMEP/FBCP/TDRP/SOT、architecture/owner maps、quality/evidence contracts 后，记录其 exact blob SHA。后续批次只在 authority blob 变化、修改目录命中新的 nested instructions、或当前 owner/contract 受影响时重读对应文档；不得每轮递归重读全部历史 authority。live ref、受影响 blob、当前 ledger rows 和目标 owner 仍须在写入前实时回读。

**FMEP-FAST-13 — batch-level spec-first 与 batch-level closure。** 一个 coherent batch 默认先形成一次 owner-level spec/production/test authoring commit，再形成一次唯一 ledger/P0/source-closure/evidence closure commit；两次保存都在立即写入前读取 live ref，并使用非强制 `expected_sha` CAS。每个 source entry 仍在唯一 ledger 中逐项记录 identity/status/test traceability；batch evidence 汇总共享合同并保留逐 entry 清单。发生并发前移时重新读取并精确融合整个受影响 batch，禁止 force 覆盖。若某批只包含无需生产代码的 repository/process provenance，也仍按同一批次原则闭合，不为每个文件制造独立提交。

**FMEP-FAST-14 — 进度报告区分“历史已读”和“当前闭合”。** `unknown/unread` 是当前 fixed-U/current-revision closure 状态，不能直接解释为“从未阅读过”。阶段 A 报告至少区分：本批 `reuse-ready` 数、`delta-read` 数、`full-read` 数、source entries closed、responsibilities implemented、blockers，以及唯一 ledger 的 unknown/unread/omitted delta。不得仅用 source-entry 百分比代表总体产品实现百分比。

**FMEP-FAST-15 — 加速不得变成浅读或只做账本。** exact-blob 复用只复用已证实且仍有效的语义结论；owner batching 的目标是减少事务开销和重复修改，不是跳过产品实现。只要 source 承载 production responsibility，批次必须实际升级对应 Fabushi owner、接口/数据/UI/服务和 shipping chain，并同步编写测试/oracle；只有纯 repository/process/docs/legal/resource provenance 才可在没有 shipping production delta 的情况下以准确 disposition 闭合。

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

每轮按 batch 报告 exact base/working head、固定 U、reuse-ready/delta-read/full-read 分类数量、已闭合 source entries、已完成责任及接线路径、待实现/阻塞、唯一 ledger 的 unknown/unread/omitted delta、保存位置和下一批。阶段 A 不要求 CI 链接、不重复解释没有 CI；统一标 not-run。source-entry closure 百分比不得冒充总体产品实现百分比；不编造测试结果、并行 worker 或后台持续运行。

## 12. 配套提示词与官方触发说明

完整执行提示词：`projects/fabushi-communication-platform/management/prompts/full-migration-deferred-ci.md`。它取代旧的逐模块运行 CI 总提示词；产品/质量范围以现行 FBCP/TDRP 为准。

触发规则核对来源（2026-10-31）：GitHub 官方 Skipping workflow runs、Disabling and enabling a workflow、Troubleshooting required status checks。跳过 push/pull_request 不覆盖其他事件，跳过 required checks 可能阻塞合并；这正是保留最终合并门而不提前伪造绿色的原因。

- https://docs.github.com/en/actions/how-tos/manage-workflow-runs/skip-workflow-runs
- https://docs.github.com/en/actions/how-tos/manage-workflow-runs/disable-and-enable-workflows
- https://docs.github.com/en/pull-requests/how-tos/merge-and-close-pull-requests/troubleshooting-required-status-checks
