# Telegram Desktop → Fabushi：逐文件逐模块等价重写迁移合同

Status: active  
Spec ID: TDRP-001  
Revision: 9  
Last updated: 2026-10-07  
Parent: `docs/specs/fabushi-bot-communication-platform.md` (FBCP-001 Revision 7)  
Project: `projects/telegram-desktop-rust`  
Acceptance status: **not complete; requirements only updated by this revision**

> 本项目不再以“能力研究子项目”或“吸收计划”作为最终交付。必须对 `telegramdesktop/tdesktop` 的全部代码文件逐个理解、逐模块建立行为合同，将全部非 UI 代码职责在现有 Fabushi Bot 架构中用最合适的语言重写并实现等价功能。C++ 非 UI 产品逻辑默认 Rust；UI 使用统一 Fabushi 表现层；原 Bot 全部功能必须保留。

“研究来源”仅表示不运行上游 Telegram 产品/网络，不表示可以研究完不实现，也不表示可以选择性吸收。该含义替代 Revision 4 的 research-only completion 和“decide whether Fabushi needs that responsibility”。本路线 source-informed，不是 clean-room。

## 1. 版本与全量 inventory

### TDRP-SRC-01 — Exact baseline

本 Revision 当前读取并接受的 tdesktop `dev` discovery HEAD 为 `d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`；`f23c37857220eb84f8559f0901ea26fb304b564b` 与更早 `33261535a0e747f125e0ed25486f01e556330677` 均为历史 baseline。accepted identity 不等于 source completeness：递归 source/build authority、逐文件语义读取和责任闭合未完成前，baseline_ready/acceptance.accepted 仍为 false。

下一执行在 GitHub Actions 内重新确认上游 HEAD，并统一更新现有 `projects/telegram-desktop-rust/upstream.lock.json`、inventory、ledger 和 dossiers；记录旧→新差异。每项 evidence 同时绑定 upstream 与 Fabushi target SHA。不得只改文档中的 SHA 就宣称 rebaseline 完成。

### TDRP-SRC-02 — 全仓库、递归依赖、资源和工具链

必须枚举精确 commit 的全部 tracked entries，而非选定几个源码目录：源码、头文件、inline/template、platform branches、schemas、生成器/生成产物、资源/语言/素材、build/package/update/test/tooling/docs/license，以及 `.gitmodules` 中全部 gitlink。递归解析子模块的锁定 commit、嵌套 gitlinks、构建时获取的外部依赖和补丁、LFS/其他外部对象（如存在），记录来源与内容 hash。

API tree 返回 truncated 或某个源/依赖无法读取时，inventory 不完整，必须继续分页/分树处理或标 blocked，不能把可见部分当全集。未知/无法解析数量必须单独报告。

每个非目录项有唯一 source identity：repository + commit + path + blob/content hash；gitlink 既有父引用行，也有递归子项。不得用历史“35 个子模块”或任何旧数量作为新 baseline 的预设完整性证明。

### TDRP-SRC-03 — 不能用 UI 排除业务

逐文件/逐 symbol 分类，不以目录名判断职责：

| 分类 | 必须采取的行动 |
| --- | --- |
| non_ui_production | 全部职责等价重写，记录目标路径/owner/测试 |
| mixed_ui_and_business | 拆出业务/校验/权限/状态/生命周期逐项迁移；UI 改为 Fabushi 实现 |
| pure_ui_presentation | 不照搬 Qt 布局绘制；逐项证明功能、交互、accessibility 的 Fabushi 替代 |
| platform_adapter | 按支持平台映射到系统适配，产品效果与生命周期不得丢失 |
| schema_or_generated | 保留协议/数据职责；生成器/输入/输出均可追溯，不复制生成的 C++ 代替重写 |
| build_test_tooling | 改为 Fabushi 自有构建/测试/打包/更新链，保留对应工程保证 |
| dependency | 递归审查、职责替代、版本/许可证与真实集成证据 |
| resource_brand_asset | Fabushi 合法素材替换；功能性资源与来源审查不可遗漏 |
| documentation_or_license | 研究/更新/保留必要来源与法律记录，不伪装成运行时迁移 |

注释、文档、法律声明、纯视觉绘制不要求生成无意义 Rust 文件，但仍必须逐文件说明处置。非 UI 产品职责没有以“not needed”跳过的通用出口。平台差异必须给出等价替代及证明，不能抹去功能。

## 2. 两层对照：文件与责任

### TDRP-MAP-01 — 文件全覆盖

每个 upstream 文件必须有映射；禁止只写“history → transcript”就宣布几百个文件完成。允许多源文件归并为一个高内聚 Fabushi owner，也允许一个混合文件拆到多个目标；不要求保留上游文件名、目录、类层级、线程模型或 UI 实现。

### TDRP-MAP-02 — 责任可追溯

每个文件中的独立责任有 `responsibility_id`，每个模块有稳定 `module_id`。文件→责任→capability→Fabushi owner/path/symbol→production entrypoint→tests/evidence 必须双向可查。共享实现不能把多个源文件的独特边界行为淹没。

### TDRP-MAP-03 — Ledger 最低合同

在现有账本/schema 上扩展字段，不建立平行真相。本 Revision 定义要求，不宣称现有 validator 已实现这些 gate。

```text
source_repository, source_commit, source_path, source_blob_sha
source_kind, module_id, responsibility_id, source_symbols
capability_ids, dependency_ids, source_analysis_path
behavior_contract, state_machine, side_effects, failure_cases
ui_disposition, non_ui_responsibilities, platform_delta
existing_owner_candidates, existing_owner, rejected_existing_owners
new_owner_proposal, new_owner_adr
fabushi_target_paths, fabushi_target_symbols, target_language, language_rationale
command_event_contracts, state_owner, lifecycle_owner
persistence_and_restart, concurrency_and_cancellation, security_boundary
network_service_dependencies, service_owner, deployment_evidence
implementation_status, coverage_disposition, blockers
production_entrypoints, production_evidence, test_evidence
target_commit, workflow_run, run_attempt, job, artifact_id, artifact_digest
composition_root, composition_slot, capability_component
variation_axes, shared_owner_contracts, duplicate_root_check
novel_capability, owner_absence_evidence, minimal_owner_adr
ui_entry_class, primary_navigation_target, collection_surface, creation_flow
object_action_scope, detail_section, capability_surface, route_contract
search_scope, search_provider, search_result_types, search_filters
search_permission_contract, search_local_remote_contract
responsive_disposition, accessibility_contract, keyboard_contract
state_continuity_contract, ui_acceptance_scenarios
design_system_version, semantic_token_usage, legacy_token_adapter
canonical_component_ids, screen_pattern_id, screen_slots
visual_state_matrix, density_contract, icon_asset_contract, avatar_contract
content_copy_keys, motion_contract, reduced_motion_contract
visual_baseline_ids, visual_evidence, accessibility_evidence
design_exception_adr, design_reviewer
quality_risk, requirement_ids, oracle_ids, invariant_ids
unit_case_ids, property_state_case_ids, contract_case_ids, integration_case_ids
functional_e2e_case_ids, temporal_case_ids, fault_recovery_case_ids
ui_functional_case_ids, visual_case_ids, accessibility_case_ids
performance_case_ids, soak_case_ids, security_case_ids
regression_ids, exploratory_charter_ids, release_gate_ids
test_fixture_ids, test_environment_ids, coverage_matrix_status, flake_status
test_execution_evidence, acceptance_reviewer, release_gate_status
license_and_provenance, reviewer, notes
```

若仍为规划态，target path 可标 proposed，但不能据此提升状态。verified 行不得缺 target symbol、实际生产路径、模块合同、current-head 证据或审查。

### TDRP-MAP-04 — 状态不能冒进

状态顺序：`unreviewed -> understood -> mapped -> implemented -> verified`。blocker 是独立字段，阻塞时停在最后真实达到的状态。批量标 mapped、目录级研究、编译成功不能直接变成 implemented/verified。

`coverage_disposition` 区分 rewritten、fabushi-ui-replacement、platform-equivalent、toolchain-equivalent、dependency-replaced、retained-legal-record 等。只有无产品运行责任的文档/法律/纯来源记录能以有依据的 non-runtime accounted 闭合；任何承载产品行为的文件/子责任都必须有真实实现/替代证据。

不得使用笼统 N/A、not needed、already covered、out of scope 隐藏未迁移业务。安全原因不保留某个具体实现手段时，仍需等价安全产品行为及明确审查；没有实现就是 blocked，不是通过。

## 3. 每个模块必须真正理解的内容

**TDRP-MOD-01:** 每个 module dossier 至少说明：准确源文件/symbol、用户效果、输入/输出、命令/事件、状态机、状态/生命周期 owner、依赖图、调用关系、持久化/迁移、线程/任务与资源所有权、顺序/去重/幂等、重试/backoff、取消/dispose、重启/reconnect、错误/权限/安全/隐私、平台差异、性能与资源预算、许可证、观察到的设计优点/债务。

**TDRP-MOD-02:** 定义行为 oracle：相同输入及对应服务事件，在 Fabushi 应产生什么输出、可见状态、持久化结果和副作用。说明因自有协议、平台、UI 或品牌改变的差异；允许实现不同，但不得把功能缩水包装成架构优化。

**TDRP-MOD-03:** 每个 dossier 必须继续写清 exact-head existing owner、target path/symbol、typed contract、production composition、必要服务端职责、完整测试场景及迁移/回滚。只有研究笔记不算模块完成。

**TDRP-MOD-04 — Test oracle before verified.** 每个 product responsibility 必须绑定 requirement IDs 与一个明确 oracle；涉及排序、状态机、并发、异步、recovery 的责任必须列 invariant 和 event-sequence cases。`eventually visible`、单一 marker、单 screenshot 或单 mock response 不能单独证明等价。
**TDRP-MOD-05 — Escaped defect closure.** 人工/后阶段发现的迁移问题必须在 regression ledger 中记录 affected requirement、missed gate、root cause、最小复现、补加测试层与永久 regression ID；修复后只跑原 happy-path 不能关闭。

## 4. 架构与重写语言

唯一目标是 current canonical Fabushi：Coordinator/Host/Runner 边界、既有 transcript/composer、Shared Room/member、permissions、attachments/artifacts、Plugins/MCP、Computer、Automations、settings/platform 必须保持。

**TDRP-OWN-01:** inspect existing owners -> select/extend owner -> production wiring。只有所有合理 owner 不适合，才用有批准 ADR 的最小新 owner；禁止整体搬入 TelegramSubsystem/CommunicationCore。

**TDRP-COMP-01 — Single root composition.** 对每个 source responsibility，mapping 必须同时标注其 canonical `composition_root` 与 `composition_slot`。同一产品概念不得因为 Human/Agent/Group/Channel/来源不同而产生完整平行 root。Conversation 统一进入一个 ConversationWorkspace；Profile 统一进入一个 Participant/Profile framework；消息列表、Transcript、Composer、Resource viewer/editor、Search、Settings、Marketplace 等同理。

**TDRP-COMP-02 — Typed capability differences only.** 上游专有行为应映射成 typed domain state + typed capability/component，例如 entry renderer、profile section、composer action、side panel、viewer、overlay 或 policy。组件可以有专门实现，但只能拥有自身最小状态/生命周期，不能复制父级 workspace 的 canonical state。

**TDRP-COMP-03 — Novel capability procedure.** 如果 Telegram 源码暴露当前 Fabushi 完全不存在的职责，必须标记 `novel_capability=true`。先完成 exact-head owner-absence evidence；若确实没有 owner，写 rejected-owner analysis 和最小 source-neutral ADR，再新增最小 domain owner。新 owner 必须通过 typed command/event/capability contract 接入统一 product shell，并复用 identity/resource/permissions/navigation。不存在“因为 Fabushi 没有所以 N/A”的出口。

**TDRP-COMP-04 — Dedicated surface is not a second app.** Call、Story、media editor、payments 等能力如果行为需要专门 surface/full-screen overlay，可以新增 capability-specific surface；但它由统一 router/workspace lifecycle 管理，不能自带第二套账号、conversation list、profile truth、message store、settings stack 或插件系统。

**TDRP-LANG-01:** C++ 非 UI 产品代码默认重写为 Rust。不得保留原 tdesktop/desktop-app C++ core 作为业务 owner，不得借 FFI、TDLib、sidecar、subprocess 或“过渡适配器”宣称已迁移。

**TDRP-LANG-02:** React/TypeScript 实现 Fabushi UI/交互投影；最薄系统桥接使用合适平台语言。生成器、schemas、构建工具使用 Fabushi 现有工具链中的 best-fit 实现。每项选择需 rationale，非默认语言需 ADR/审查；不机械把 UI 代码翻成 Rust。

**TDRP-LANG-03:** 通用成熟安全/媒体依赖须登记、锁定和审查，可以替代低层标准能力；这不豁免其 source/dependency inventory，也不允许隐藏原 Telegram C++ 业务模块。不要自行发明密码学来满足表面语言指标。

## 5. 网络与功能等价

不接官方 Telegram 网络是架构约束，不是省略协议/同步职责的许可证。全部 protocol/session/update/storage/media 代码仍须逐文件分析，把 ordering、重放、gap、去重、durability、多设备、安全、续传、限流和恢复等责任重写到 Fabushi 自有服务与 owner。

列出每个客户端能力的远端合同及 Fabushi 等价服务。上游客户端源码不等于拥有其服务器实现、内容库、官方账号/商业资产。对未知或尚无自有服务的部分保留明确 blocker，并推进其他模块；没有真实服务不能验收为全部功能完成。

不能把 Bot/inline callbacks 与 Agent 自主执行混为一谈；必须保留不同 typed intents、权限、用户确认与副作用边界。全量能力域以 FBCP-001 §4 和实际全量源码为准，旧 capability graph 只是历史索引，不是范围上限。

## 6. UI 与品牌的迁移合同

遵循 FBCP-001 §8：微信式信息结构为左侧竖向主导航 + 列表/搜索区 + 主工作区 + 按需详情。消息、联系人、插件市场是竖栏一级入口；保留 Agents、任务、Computer、Automations、账号设置等现有主能力。市场迁移复用原 owner 与权限，不做第二套市场。

UI 文件必须证明每项交互和非 UI 逻辑均被承接。功能不得因不用 Qt 而消失。统一 human/agent/group/channel workspace、草稿、滚动、未读、streaming、任务与产物状态，原 Bot 与通信能力同时可用。

所有 UI 映射都必须标注 canonical root + typed slot。禁止以共享 store 为理由保留两套完整 UI composition，例如 Bot/Human chat window、Agent/Human profile root、Telegram/Fabushi media viewer。允许的是一个 root 里的 typed sections/renderers/actions/overlays。对当前 Fabushi 没有的 UI 能力，先证明没有现有 surface 可承接，再新增最小 capability surface，并保持统一 shell/router、canonical state 与导航。

产品标识统一为 Fabushi，清理 Telegram/Grok Bot/Gok Bot 品牌、Logo、默认素材和独立入口；保留法律要求的 LICENSE/NOTICE/版权与 source provenance。历史研究中的上游名字不能被抹去，否则账本失去可验证性。新 owner 名称使用业务职责，不使用来源品牌。

### TDRP-UI-IA — 每个用户可见责任必须有入口归属

每个上游 UI/交互责任都必须记录 `ui_entry_class` 与 canonical route，按 FBCP-001 §8.1 分类。禁止“上游有一个页面/菜单，所以 Fabushi 也新增一个一级页面”的机械映射。Group/Channel/Topic 等属于 Conversation kinds/filter/views；新建群组、新建频道等必须进入统一 typed ConversationCreation flow。Contact 的添加/邀请归 Participant/contact relation；Contact 的“发消息”进入同一 Direct Conversation/ConversationWorkspace。

Call/Story/media editor/payment 等只有在确实需要专用交互时才可拥有 capability surface，并必须由统一 shell/router 管理且可返回原上下文。

### TDRP-UI-SEARCH — 搜索是一套 domain contract

Telegram 中 dialog/history/people/media/username/filter 等搜索职责必须拆解并升级到 Fabushi canonical Search owner。入口内搜索、对象内搜索、Universal Search、联系人 picker、群成员/管理员 picker 等必须共享 typed query/result/provider 与权限合同；不得把不同页面搜索实现成彼此独立的索引/state truth。

模块 dossier 需要覆盖 scope/filter/result type、local/remote source、ranking/dedupe、cursor/pagination、debounce、cancellation、stale-result fencing、account/privacy/membership/block/retention permission、结果 provenance、点击结果后的 canonical navigation。SearchProvider 只提供领域结果，不拥有独立 Search 产品。

### TDRP-UI-DESIGN — Design-system migration contract

所有 migrated UI responsibility 必须继续映射到 Fabushi design-system version、semantic tokens、canonical component IDs 与 canonical screen pattern/slot。默认不允许 feature-local CSS/JSX 自由创造另一套 visual grammar；需要新 primitive/pattern/token 时，先证明 canonical component/pattern 无法承接，再扩展共享 design system 并取得 review。

当前 exact-head 存在 recovered Sand primitives、runtime theme token generator 及 `cursor-*` aliases。它们是现状兼容事实，不是新 Telegram-derived UI 的公共命名权威。实施应先建立 Fabushi semantic layer/wrappers，再逐步把新功能与受影响旧 surface 接到该层；不得为了重命名而破坏 immutable evidence 或原 Bot 行为。

每个 dossier 还必须覆盖 loading/empty/error/offline/permission/disabled/selected/focus/hover/pressed/data-heavy states、light/dark、locale/long text、responsive collapse、keyboard/focus restore、screen reader、reduced motion、icon/avatar/copy consistency 和 visual baseline。

## 7. 模块实施闭环

每个已批准模块依次执行：

1. 重新确认 target/upstream exact SHA 与 affected dependencies；读完整源文件和相关调用者/被调用者。
2. 建 dossier、状态机、责任拆分及测试 oracle，列出错误/负面/重启等路径。
3. 对每个文件/责任选择现有 Fabushi owner，必要新 owner 经最小 ADR 批准。
4. 在 Fabushi 自有路径用选定语言实现，补齐真实服务端/平台依赖。
5. 接入唯一 shipping composition，迁移数据/路由/权限，移除被替代的重复 ownership。
6. 在 GitHub Actions 按 quality risk 执行 unit/table、property/state-machine、contract、integration、真实 E2E、temporal/recovery、UI functional/visual/a11y，以及适用 performance/security/fault/soak。
7. 对动态用户旅程生成 timeline trace/DOM manifest/screenshots/video，并按 oracle 检查 terminal settlement、ordering、reconnect/restart 后一致性。
8. 更新 RTM、regression ledger、current-head evidence，并由独立 acceptance reviewer 复核后，最后才能把相应责任标 verified。

已批准且无依赖的模块可并行推进；缺权限、账号、服务或某个模块被阻塞时不停止其他安全可推进的任务。仍然不得跳过本模块的必要前置与既有 Bot 硬门。

## 8. Verification gates

以下 gate 必须在实现阶段扩展现有检查并取得真实结果，规范文字不是 gate 已执行的证明：

- **G-INVENTORY:** accepted exact root tree + all recursive gitlinks/external acquisitions；0 unknown/unread/omitted source entries。
- **G-FILE:** 每个文件及 non-UI responsibility 均有处置；0 directory-only placeholders；双向 target trace 无遗漏。
- **G-MODULE:** 每个模块有完整理解、状态机、oracle、existing-owner resolution、language rationale、typed contracts。
- **G-PRODUCTION:** 对应 Rust/best-fit 实现真实接入当前 shipping product；无原 Telegram C++ owner、TDLib 包装或平行 runtime。
- **G-COMPOSITION:** 每个能力都绑定唯一 canonical root + typed slot；无按 Human/Agent/Group/Channel/Telegram 来源复制的完整 workspace/list/profile/composer/resource/settings root；novel capability 只有经 ADR 的最小 owner/surface，并复用 canonical truth。
- **G-BEHAVIOR:** 功能/错误/顺序/重复/取消/销毁/重启/网络恢复等与合同等价；mock 仅作局部测试，不代替真实 E2E。
- **G-TRACEABILITY:** 每个 applicable requirement/responsibility 绑定 risk、oracle/invariants、test IDs、current-head evidence 与 acceptance verdict；0 orphan requirement、0 无依据 test、0 verified-without-evidence。
- **G-PROPERTY-STATE:** 高风险状态机/排序/幂等/merge/replay/cancel/restart 通过 table-driven + property/state-machine scenarios，包括 duplicate/out-of-order/late snapshot。
- **G-TEMPORAL:** 动态流程按完整时间序列验收；Conversation/Agent terminal result 经 settlement quiet window、switch-away/back、reconnect、reload、restart 后保持，不允许 final disappearing/intermediate replacement/completed rollback。
- **G-UI-FUNCTIONAL:** UI route/control/action/state ownership 真实可操作，禁止仅以 screenshot 证明功能；关键 user journey 由 packaged candidate 执行。
- **G-REGRESSION:** escaped/manual defect 都有永久 regression ID、root-cause lower-layer coverage 与 current-head rerun；flaky critical test 不得通过 rerun-until-green。
- **G-EXPLORATORY:** 风险导向 packaged journey/session video/timeline 由独立 reviewer 实际审阅并记录 charter、观察和 verdict。
- **G-FAULT-RECOVERY:** timeout/disconnect/reconnect/restart/cancel/stale/duplicate/out-of-order/permission/service failure 中适用场景 fail-closed。
- **G-PERF-SOAK:** approved performance/resource budgets 与长时运行/large-data soak 通过，无无界内存/CPU/磁盘/DOM/state 增长。
- **G-SECURITY-PRIVACY:** account isolation、permission/secret lifecycle、Mini App/WebMCP、redaction、negative security/privacy 与适用 accessibility gates 通过。
- **G-BOT:** 原 Agent、Coordinator/Host/Runner、MCP/Plugins、Computer、Automations、审批、任务和产物能力无回退。
- **G-UI-BRAND:** 左竖栏/消息/联系人/插件市场与统一 workspace 真实可用，合法来源记录外产品品牌全为 Fabushi。
- **G-UI-IA:** 每个用户可见 capability 有唯一 entry classification/route；Group/Channel/Topic 等不会形成平行 App；统一 ConversationCreation flow、object actions、capability surfaces 与返回路径可验证。
- **G-SEARCH:** 一个 canonical Search owner 支撑 contextual/object/universal 三层搜索和所有 picker；权限、本地/远端合并、排序/去重/分页/取消/stale fencing 及 typed result navigation 全闭合。
- **G-DESIGN-SYSTEM:** migrated UI 只使用 Fabushi semantic token/component/screen contracts；legacy source-named token 只在登记的 compatibility/evidence adapter 中，0 feature-local second visual system。
- **G-COMPONENTS:** canonical primitives/patterns owner 唯一；新增 primitive/pattern 有 design exception review、复用范围和 migration plan。
- **G-VISUAL:** fixed viewport + light/dark + locale + state + keyboard/a11y + reduced-motion visual matrix 由 current-head GitHub Actions 生成并与 approved baseline 比较；非预期 diff fail。
- **G-RELEASE:** 服务部署、隐私安全、许可证、迁移回滚、性能、支持平台签名/安装/更新与 packaged acceptance 全闭合。
- **G-EVIDENCE:** upstream/target/checkout/run/attempt/job/artifact digest/provenance 一致，HEAD 变化使受影响旧证据失效。

所有可执行验证仅 GitHub Actions；本项目不沿用旧 htch-runtime allowance。禁止在用户 Mac、bhrum2 或 assistant container 执行 build/test/lint/generator/schema/benchmark/fuzz/package/acceptance。

## 9. 完成定义

TDRP-001 只有以下同时成立才能 accepted：

1. accepted upstream baseline 全量 inventory 和递归依赖闭合。
2. 逐文件与逐模块都已理解；每个非 UI 产品责任均已重写、接入并 verified。
3. 原 UI 实现已由 Fabushi UI 承接，文件内的非 UI 责任与全部用户功能没有漏掉。
4. 全部 Telegram Desktop 范围能力和现有 Bot 能力在同一 Fabushi 架构/产品中工作。
5. 所有网络/服务端/平台/商业依赖具备真实实现或集成证据；0 in-scope open blocker。
6. 0 omitted/unmapped/unfinished responsibility、0 stub/no-op/fake fallback、0 隐藏功能的 N/A。
7. 无 Telegram/Grok/Gok 产品品牌和第二 owner；法律声明/provenance 保留。
8. 所有 gate 和 FBCP-001 applicable AC 有 current-head GitHub Actions production/release evidence。

研究完整、文件数量一致、owner 映射完成、Rust 编译、某条垂直 slice 或少量 UI 截图均不能单独证明完成。只有 FBCP 全部 AC 通过，才可宣称整个产品迁移完成。

## 10. 本次规范状态

本 Revision 未宣称已逐个阅读全部上游文件、未刷新锁文件/全量账本、未重写产品或运行新 gate。下一任务必须先完成 baseline/ledger/schema/validator 的真实差异审计，再按已有无阻塞 owner 推进实现；旧研究和代码可以复用，但状态必须基于新合同重新证明。

References: https://github.com/telegramdesktop/tdesktop ; upstream README/individual licenses at the accepted exact tree ; FBCP-001 Revision 7 ; current canonical Fabushi/Bot specifications.
