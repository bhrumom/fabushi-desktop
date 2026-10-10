Live TDRP Revision 9 upstream authority: telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941 (root tree 4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6); recursive total 16,125; read-through 7,028; first unread 7,029 Telegram/ThirdParty/TooManyCooks::.clang-format@b484c8323870e494c8031a35ddac49aa114052b3; unread 9,097; unknown 15,845; omitted 0. Orders 6,977-7,028 exact-read the QR generator component. Existing RemoteControl pairing, SharedRoom invite and DeepLink owners retain domain state; QR remains a source-neutral representation/codec responsibility with no duplicate owner. Fresh descendant exact-head GitHub Actions evidence is required.


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

Historical predecessor authority: 本 Revision 当前读取并接受的 tdesktop `dev` discovery HEAD 为 `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`（root tree `5030985204963cbbd362ced7412d231b04ebd0cc`）；`42f8a36d43b8c805bc821905bea4cfeb3af1d41d` 与更早 authority 仅作历史证据。当前 recursive denominator 16,123；read-through 5,773；unread 10,350；unknown 15,844；source closure 仍 open，predecessor Actions 不得证明新 baseline。

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

Historical predecessor authority: Current live authority (2026-10-09): `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` (root tree `5030985204963cbbd362ced7412d231b04ebd0cc`), three commits ahead of historical `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`; 15 root paths changed (12 modified, 3 added), recursive denominator is 16,123, read-through is 5,773, unread is 10,350, unknown is 15,844, omitted is 0, and source closure remains open.


### Revision 9 live read-through note: 5600-5604

Exact-source decomposition now includes Local Storage and Settings Main. Local cache clearing/quota/retention must bind canonical Storage/Cache owners; Settings routing/profile/account/wallet/business/help composition must bind canonical product owners. Source-read credit does not reduce unknown. Current counters: 16,123 total; 5,604 read-through; 10,519 unread; 15,844 unknown; 0 omitted. First unread: order 5,605 `settings_notifications.cpp`.


### Revision 9 live read-through note: 5605-5613

Revision 9 fail-closed validation caught a transient deterministic-order mistake at 5608 and rejected the branch evidence. Correct exact order is: 5605–5607 Notifications main/header/style; 5608–5609 Reactions; 5610–5611 per-type notification policy; 5612–5613 Passkeys. The corrected rows are all mapped-open. Notification responsibilities map to canonical policy/account/privacy/call/platform owners; Account Passkeys map to canonical Account Auth plus bounded WebAuthn. Existing remote Agent WebAuthn proxy infrastructure is not product Account Passkeys proof. Reading does not close native action authorization or credential lifecycle gaps. Counters: 16,123 total; 5,613 read-through; 10,510 unread; 15,844 unknown; 0 omitted. First unread: 5,614 `settings_premium.cpp`.

## Revision 9 deterministic source batch 5614–5616 — Premium entitlement/commerce

Exact blobs 5614 `settings_premium.cpp@60ac3cbb…`, 5615 `settings_premium.h@894e07c3…`, 5616 `settings_premium.style@4949348b…` are read/decomposed. Applicable contracts map to canonical account entitlement/subscription, commerce/payment routing, Settings and design-system owners. Purchase activation must be account-fenced, idempotent, cancel/failure safe and reconciled after settlement/reload/restart; invalid external routes fail closed. No Telegram Premium runtime or UI family may be introduced. Accounting: read-through **5,616**, unread **10,507**, unknown **15,844**, omitted **0**.

### Read-through 5617–5622

Privacy/Security composition, editable keyboard shortcuts, and server-backed Website/Bot authorization sessions are exact-blob read-complete and mapped-open to existing canonical Account Security/Privacy, Authorization, Command Registry, Bot/Blocked-Peer, Payments/Retention and Settings owners. Exact authorization hash, account/session fencing, destructive confirmation/retry/idempotency and keyboard/a11y behavior remain production gates. Accounting: read-through 5,622; unread 10,501; unknown 15,844; omitted 0. First unread: 5623 `settings/settings_builder.cpp`.

### Read-through 5623–5632

Settings Builder/Common/Search/section-factory infrastructure, hidden diagnostic codes, and Credits/Gift commerce graphics are exact-blob read-complete. UI infrastructure maps to canonical Settings/UniversalSearch/design-system owners; diagnostic codes require controlled development/support policy rather than product copying; Credits/Gift actions remain mapped-open under canonical Wallet/Payments/Credits + Gift/Commerce with exact-account/id, cancel/retry/idempotency/rollback/settlement/restart gates. Accounting: read-through 5,632; unread 10,491; unknown 15,844; omitted 0. First unread 5633 `settings_experimental.cpp`.

### Read-through 5633–5642

Experimental flags/support, FAQ suggestion loading, Settings layer/focus/keyboard navigation, notification-common presentation and Power Saving are exact-blob read-complete. They map to canonical feature-flag policy, Help/Support, Settings/design-system navigation, Notification Settings and Performance/Power owners. No source-named product UI or parallel state owner is introduced. Accounting: read-through 5,642; unread 10,481; unknown 15,844; omitted 0. First unread 5643 `settings_power_saving.h`.

## Deterministic source-order correction at 5,623 and live read-through 5,653

Accepted recursive tree `5030985204963cbbd362ced7412d231b04ebd0cc` proves `Telegram/SourceFiles/settings/settings.style@8b9e56dd…` is blob order **5,623**, between `sections/settings_websites.h` (5,622) and `settings_builder.cpp` (5,624). The earlier 5,623–5,642 labels were therefore off by one and are superseded. Their shard/attestation artifacts are replaced rather than treated as parallel authority.

The corrected contiguous batches are 5,623–5,633 (settings.style through Credits/Gift graphics), 5,634–5,643 (Experimental/FAQ/Settings shell/navigation/Power Saving implementation), and 5,644–5,653 (Power Saving interface, privacy controllers, recent Settings search, scale preview, Settings search/type). Current accounting is **5,653 / 16,123 read**, **10,470 unread**, **15,844 unknown**, **0 omitted**. Reading alone closes no unknown. First unread is 5,654 `Telegram/SourceFiles/statistics/chart_lines_filter_controller.cpp@40c3b612…`.

Revision 9 requires accepted-tree blob rank as the source of deterministic order. A directory listing is insufficient because root style blobs such as `settings/settings.style` can precede root `.cpp` files.

## Deterministic statistics batch 5,654–5,665

Exact blobs for line-filter animation, ruler generation, interactive chart widget, range min/max segment tree, statistics style/common types and chart JSON deserialization are read/decomposed. They map to canonical Analytics/Insights data projection + reusable Chart/design-system owners; business/account truth must not live in the chart. JSON parse errors, empty columns and column-length mismatches fail closed. Hover/zoom/filter/footer state is derived and lifetime-bounded. Accounting: **5,665/16,123 read; 10,458 unread; 15,844 unknown; 0 omitted**. First unread: 5,666 `statistics_format_values.cpp`.


## Deterministic statistics formatting/export batch 5,666–5,674

Exact blobs for locale/timezone analytics labels, TON/Credits chart graphics, analytics-to-sheet projection, 64-bit chart values, and XLSX/OpenXML serialization are read/decomposed. They map to canonical Analytics/localization, Wallet/Credits visual projection and Export owners. Export must preserve inline-string formula-injection safety, XML/control escaping, legal unique sheet names, empty/short-series behavior, numeric/date boundaries, ZIP failure, cancellation/account fencing and cross-platform save behavior. No Telegram-derived statistics/export runtime or UI is introduced. Accounting: **5,674/16,123 read; 10,449 unread; 15,844 unknown; 0 omitted**. First unread: 5,675 `statistics/view/abstract_chart_view.cpp@4b5f2929…`. Reading alone closes no unknown.


## Deterministic statistics chart-view batch 5,675–5,684

Exact blobs for chart-view abstraction, double-line ratios, bar/stack bars, animated dual-axis/currency rulers, closed chart-type dispatch and linear/DPR cache + pointer hit-testing are read/decomposed. They remain mapped-open to one canonical Analytics/Chart design-system owner. The Fabushi port must strengthen empty/one-point, zero-range/max, all-disabled, mismatched-series, invalid-type and pointer-endpoint cases fail closed; rendering state owns no analytics truth. Accounting: **5,684/16,123 read; 10,439 unread; 15,844 unknown; 0 omitted**. First unread: 5,685 `statistics/view/stack_chart_common.cpp@82a1143f…`.


## Deterministic statistics stack/pie batch 5,685–5,690

The remaining statistics/view blobs are read/decomposed: stack geometry/index mapping, filtered pie-percentage rounding, and stack-linear local zoom/pie transition/selection. They remain mapped-open to the canonical Analytics/Chart owner. Canonical code must fail closed for <2 points, zero-width ranges/denominators, invalid indices/line lengths, zero totals and stale/racing zoom/filter/hover state. Accounting: **5,690/16,123 read; 10,433 unread; 15,844 unknown; 0 omitted**. First unread: 5,691 `statistics/widgets/chart_header_widget.cpp@b8754ceb…`.


## Deterministic statistics widgets batch 5,691–5,696

Statistics widgets are exact-blob read/decomposed: responsive chart header, line-filter controls with last-visible-line guard, and localized/currency point details with zoom/cache/ripple state. They map only to canonical source-neutral Analytics/Chart + existing design-system controls. Empty/mismatched data, invalid indices, RTL/long text, currency precision, a11y/focus/theme/DPR and teardown remain production gates. Accounting: **5,696/16,123 read; 10,427 unread; 15,844 unknown; 0 omitted**. Next order 5,697 must be rebound from the accepted recursive tree before recording its path.


## Deterministic PCH + storage security/migration batch 5,697–5,701

Order 5,697 maps C++ PCH/platform composition to canonical cross-platform build evidence. Orders 5,698–5,701 map critical durable local-file protocol, Argon2id/scrypt/legacy passcode KDFs, encryption/signature/version checks and versioned settings/session migrations to canonical Persistence + Local Security/Key Protection + Settings Migration owners. Invalid/over-cost KDFs, corrupt/truncated/unknown blocks, interrupted writes, wrong passcodes and legacy/account/restart recovery must fail closed. This does not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. Accounting: **5,701/16,123 read; 10,422 unread; 15,844 unknown; 0 omitted**. First unread: 5,702 `storage/download_manager_mtproto.cpp@d0007eb1…`.


### TDRP Revision 9 source read 5702-5711 — storage transfer lifecycle

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,702-5,711 cover adaptive remote download scheduling, generic cache/file loader lifecycle, part-based download/resume, HTTP(S)-only web transfer and multipart media upload. Canonical mapping is the existing source-neutral messaging attachment/file-transfer owner plus platform download/save adapters; MTProto/DC/CDN details are protocol-specific and must not create a Telegram runtime. Required parity includes cancel/teardown/account-switch stale-result fencing, partial resume, integrity/reference refresh, redirect downgrade/scheme protection, TLS/auth/disk failures, adaptive concurrency, removed-session resend, transcode/archive cancellation, upload progress/finalization and restart/recovery. Accounting: 5,711/16,123 read, 10,412 unread, 15,844 unknown, 0 omitted; reading alone closes no unknown. First unread: 5,712 `Telegram/SourceFiles/storage/localimageloader.cpp`.

### TDRP Revision 9 source read 5712-5721 — media preparation and durable serialization

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,712–5,713 cover source-neutral media-send preparation and worker/result lifecycle; 5,714–5,717 cover local settings/theme/language/update persistence, legacy encrypted migration and bounded serialization primitives; 5,718–5,721 cover versioned document/media and user/chat/channel projection persistence with legacy location migration and fail-closed corruption handling. Canonical mapping stays with existing messaging Attachment/media-preparation, Persistence/Settings/Local Security/Updater/Theme/Localization, messaging data persistence, and Identity/Profile/Conversation owners. Telegram/MTP file formats and flags are migration inputs only and must not become a parallel runtime or product UI. Required closure includes cancel/account-switch/teardown stale-result fencing, malformed media/encode/archive failures, wrong-key/tamper/truncation/future-version handling, interrupted atomic writes, legacy migration, cache-vs-authority reconciliation, reload/restart and canonical UI/a11y evidence. Accounting: **5,721/16,123 read, 10,402 unread, 15,844 unknown, 0 omitted**; reading alone closes no unknown. First unread: **5,722 `Telegram/SourceFiles/storage/storage_account.cpp@5888ffa1164327b7170a2a3d8df7793a3fa467da`**.

### TDRP Revision 9 source read 5722-5729 — account/domain storage and local security

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,722–5,723 cover encrypted account-scoped drafts/cache/settings/trust/payment/webview/bot/wallet persistence; 5,724–5,725 cover bounded cloud-blob download/extraction; 5,726–5,727 define passcode wraps, bounded memory-hard derivation, secret cleansing, fresh verification tokens, App Lock mutation, crash-safe committed generations, legacy migration and wallet keyring protection; 5,728–5,729 route shared-media/profile-photo cache queries and invalidation. The local-security rows strengthen but do not close the existing Local Passcode/App Lock/Wallet key-protection responsibility. All remain mapped-open to canonical source-neutral owners; no Telegram storage/security runtime or UI family is permitted. Production evidence must cover wrong/unsupported KDF, stale verification, passcode/App Lock mutation, crash/restart migration, wallet corruption/absence, account fencing, delayed-write teardown, cloud/archive failure, cache invalidation and canonical UI/a11y. Accounting: **5,729/16,123 read, 10,394 unread, 15,844 unknown, 0 omitted**. First unread: **5,730 `Telegram/SourceFiles/storage/storage_file_lock.h@181b5bc063827458c4e59a0daa167355f835ae0b`**.

### TDRP Revision 9 source read 5730-5740 — file locking, archive/media preparation and sparse indexes

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,730–5,732 define exclusive storage locking on POSIX/Windows, including conflict-process handling; 5,733–5,734 define folder/multi-file ZIP preparation with symlink exclusion, progress/cancel, size/disk failure and stale temporary cleanup; 5,735–5,736 cover Composer/editor media drag/drop, MIME/limit/decode and asynchronous preview preparation; 5,737–5,740 define shared-media categories and sparse message-id range merge/query/count/invalidation. All remain mapped-open to canonical Persistence/platform adapters, Attachment/Media Editor and Shared Media/index owners. Required evidence includes cross-platform lock conflict/retry, archive cancellation/limit/disk failure/cleanup, malformed media and stale async results, and sparse pagination/count consistency. Accounting: **5,740/16,123 read, 10,383 unread, 15,844 unknown, 0 omitted**. First unread: **5,741 `Telegram/SourceFiles/storage/storage_user_photos.cpp@232befde40c5edea34286105374058fcc66ab658`**.

### TDRP Revision 9 source read 5741-5754 — user-photo cache, streamed download and operator support

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,741–5,742 cover profile-photo history cache list/count/paging; 5,743–5,744 cover bounded aligned streaming-part download with duplicate suppression, cancellation and completion; 5,745–5,754 cover operator/support template autocomplete/search, contact confirmation, shortcut navigation, occupation leases, support-info editing, history preload, fast-button preferences and template update/reload. Support-specific code is conservatively mapped-open pending explicit applicability evidence rather than silently omitted. Required evidence includes profile count/paging consistency, part failure/cancel/staleness, support account/history fencing, edit request replacement, occupation expiry/race, preload retry and malformed/update-failed template recovery plus canonical keyboard/focus/a11y. Accounting: **5,754/16,123 read, 10,369 unread, 15,844 unknown, 0 omitted**. First unread: **5,755 `Telegram/SourceFiles/tde2e/tde2e_api.cpp@efe4edc5eef735c954b39d7a927722a7e8f9214c`**.

### TDRP Revision 9 source read 5755-5758 — call end-to-end encryption

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,755–5,758 own call E2EE temporary/private key lifecycle, one-shot ECDH envelopes, encrypt/decrypt callbacks, participant/state blocks, inbound/outbound queues, subchain reconciliation, failure terminality and emoji verification hash, with protocol conversion as an adapter. They remain mapped-open to canonical Calls E2EE/Crypto + Call Session owners; transport TLS is not a substitute. This implementation batch also hardens the canonical Wallet ledger so reuse of an idempotency request id must match operation kind, accounts, normalized currency, amount and reference; conflicting credit/transfer/refund reuse fails closed with `RequestConflict` and unit coverage. Broader wallet rows remain open pending full production/evidence closure. Accounting: **5,758/16,123 read, 10,365 unread, 15,844 unknown, 0 omitted**. First unread: **5,759 `Telegram/SourceFiles/test/README.md@429b4f3d83ec6fa6aa71203624a894d1d7973d35`**.

### TDRP Revision 9 source read 5759-5771 — professional task-test harness

Accepted upstream `863cf10d9f34fb0b1b35b35da1bda75acfc58d2e` / tree `5030985204963cbbd362ced7412d231b04ebd0cc`. Orders 5,759–5,771 define the upstream task-test authority: pure bounded readiness; exact object/action identity; assertions outside waits; diagnostic timeouts; precondition-only N/A; packed-scenario teardown; protected fixture secrets; deterministic activation/focus and animation timing; exact dialog-shell interaction; visual capture of real paint owners with DPR/scale boundaries; and privacy-safe clipboard classification/retry without logging foreign clipboard contents. These map to Fabushi's professional cross-platform GitHub Actions test/evidence harness, not to production Telegram UI. The same descendant repairs current-head authority failures by correcting order 5,739 to accepted blob `cc0e3ed6b2c763326c2c16635a7671a747ef506b` and refreshing the strict Grok semantic-adaptation target for `source/electron-main/attachments/attachments.ts` to production blob `9fb1ed46b2ae623e7fa781d6d3ab47f465f4a332`; no gate is weakened. Orders 5,772-5,773 cover the Windows console-lock professional-test contract: the implementation reads process-session/WTS metadata and classifies locked/unlocked/unknown, while the header fixes the public NotApplicable/Unlocked/Locked/Unknown state model, makes `locked()` true only for Locked, and requires `ConsoleLockGate` to remain empty for Unknown and NotApplicable. Only a positive locked reading gates; documented legacy-Windows reversed flags remain unknown; diagnostics never read or print user/domain/window-station fields; non-Windows is explicit N/A. This maps to the canonical professional cross-platform test/evidence harness and creates no production Telegram UI. Accounting: **5,773/16,123 read, 10,350 unread, 15,844 unknown, 0 omitted**. Reading alone closes no unknown. First unread: **5,774 `Telegram/SourceFiles/test/test_corner_patch.cpp@a34785a050cdeea405c3948345704d2e19395843`**.



### cf478d37 live-authority rebaseline

Live Revision 9 authority (2026-10-10): telegramdesktop/tdesktop@cf478d37c8f57df831cdedb5621e1cf2ef069a0f (root tree fa901c0c44fb1f94fd38de0d5edbcfef23a9f0ad). Root non-directory=6,653; recursive non-directory=16,125; read-through=5,793; first unread=5,794 Telegram/SourceFiles/test/test_log.cpp@17c72a8ae1bc672e7c380b855887b859b78fabec; unread=10,332; unknown=15,846; omitted=0. Changed accepted-prefix orders 11/30/31/92/4599/5759/5764/5765 were explicitly re-read; lib_ui is now order 6,589; runs 37958691718/37958689647 and artifacts 11630066443/11631221092/11630211787 are historical-only for 863cf10d9f34fb0b1b35b35da1bda75acfc58d2e + b4d9f7f8cf580eefd553c5ce105f1d3e682de87f.

Direct delta: channel_earn.style replaces static negative placeholder margins with floating placeholderShiftLeft; lib_ui 91ff446 implements horizontal interpolation in InputField and MaskedInputField. Canonical TextField remains the sole product owner. Orders 5,774-5,793 were identity/order reconciled; reading alone closes no unknown.


### Revision 9 live read-through note: 5794-5801

Orders 5,794-5,797 preserve professional evidence-log one-line/completion-forgery integrity plus an independent raw-byte oracle. Orders 5,798-5,799 preserve complete mapped-target/viewport capture readiness. Orders 5,800-5,801 preserve the bounded reversible not-marking-read evidence lever. These are test/evidence responsibilities only, create no second product owner, and close no unknown. The new 811b83a1 readability delta is order 5,951 and remains unread/unknown.


### Revision 9 live read-through note: 5802-5805

Orders 5,802-5,803 close the exact-blob read/decomposition of popup/context-menu professional evidence semantics: same-turn fresh-menu identity, explicit refusal taxonomy, isolated QAction queued-callback delivery, prepared-frame capture and lock/teardown behavior. Orders 5,804-5,805 close the SentMessageWatcher client→server id reconciliation contract with stable-history/candidate fencing and a five-second diagnostic probe throttle. These are test/evidence responsibilities only and close no unknown. First unread is now 5,806 `test_notify_override.cpp`.

### Revision 9 live rebaseline/read-through note: b0d1fe5e / 5166 / 5806-5813

The 811b83a1→b0d1fe5e upstream delta changes only order 5,166 `media_view_overlay_widget.cpp`: the old `forbidsSaving()` control-refresh branch hid controls and could call `DocumentSaveClickHandler::Save(...ToCacheOrFile)` when media was not loaded; b0d1fe5e removes that viewer-driven automatic persistence path. The new blob `e35f67119323850a3cea450548566186c8a2231c` was re-read and the mapped-open MediaViewer lifecycle responsibility now explicitly requires protected/forbids-saving media not to be auto-persisted as a rendering/control-refresh side effect. Path order and total counts are unchanged. Separately, orders 5,806–5,813 are read-complete/responsibility-decomposed professional test harnesses for reversible notify overrides/no-server-write locality, document-open handoff inspection without OS launch, SeparatePanel pointer/liveness evidence, and post-paint temporal sampling. They add no product owner, fake fallback, unknown closure or release credit. First unread is 5,814 `test_probe.cpp`; read-through is 5,813/16,125, unread 10,312, unknown 15,846, omitted 0.

### Recursive component read-through 6,654-6,697 — Microsoft/GSL

Orders **6,654-6,697** exact-read the pinned `Microsoft/GSL@87f9d768866548b5b86e72be66c60c5abd4d9b37` component mounted at `Telegram/ThirdParty/GSL`. The 44 entries cover repository/build/CI metadata, license/security/provenance, header-only safety contracts (bounds and extent checks, non-null ownership, checked narrowing, byte/string boundaries, scope cleanup), and focused fail-closed tests. Applicable semantics map to existing Fabushi Build/Quality/Provenance and Rust/TypeScript/platform-boundary owners; no Telegram/GSL-specific runtime is introduced. Reading does **not** reduce `unknown`; production/shipping evidence remains required. Accounting: **6,697/16,125 read; 9,428 unread; 15,845 unknown; 0 omitted**. Next: **6,698 `Telegram/ThirdParty/MicroTeX::.github/workflows/ubuntu-gtk.yml@9863f6100fd12d94b21e81e1c5f4859aad3c1764`**.
## 14. Math/rich-content absorption contract

**TDRP-MATH-01 — Existing canonical owner.** Telegram/MicroTeX math responsibilities map into the existing Fabushi ConversationWorkspace/Transcript rich-content math owner (`frontend/src/recovered/features/conversation/workspace/math-runtime.ts` + `math.tsx`). MicroTeX/Telegram-specific runtime, C++ parser, sidecar or second transcript renderer is prohibited.

**TDRP-MATH-02 — Pre-parse resource bound.** Untrusted assistant/transcript math input larger than **32 Ki UTF-16 code units** must be rejected before loading/invoking the shipped KaTeX parser. The UI must fail closed to source-text fallback; oversized content must not enter parser work merely because renderer/layout caps exist later in the pipeline. This preserves the applicable upstream denial-of-service responsibility while remaining source-neutral.

**TDRP-MATH-03 — Bounded shared render cache.** The shared async math-markup cache is an LRU with a default **32 MiB approximate byte budget**, including a fixed per-entry overhead so tiny/empty entries cannot grow the map without bound. Hits refresh recency; rejected work is removed; eviction/invalidation must not let late promise completion resurrect an evicted entry. Streaming prefixes and adversarial unique formulas therefore cannot create unbounded renderer memory/state growth.

**TDRP-MATH-04 — Platform adaptation and verification.** Telegram's bitmap width/height/device-pixel-ratio allocation caps are not mechanically copied into Fabushi's DOM/KaTeX renderer. Their applicable product effect is enforced by the pre-parse/cache bounds above plus canonical transcript overflow/visual acceptance. Focused production tests must prove over-limit rejection occurs before loader/parser invocation, cache sharing, bounded LRU eviction, loader isolation, invalidation and retry-after-rejection. All executable evidence is GitHub Actions exact-head evidence.

**TDRP-MATH-05 — Rendered-markup and configuration fail-closed bound.** Every strict or recovery KaTeX render is capped at **8 MiB approximate UTF-16 markup bytes before transcript DOM injection/cache retention**. A strict-render output over the cap terminates that render attempt without a second parser pass and projects an escaped source/error fallback; a recovery output over the cap does the same. Caller-supplied expression/markup/cache budgets that are negative, NaN or infinite normalize fail-closed to zero rather than disabling a guard. This output cap complements the 32 KiB input, expansion and size limits; it does not claim a synchronous wall-clock deadline. Focused exact-head tests must cover over-cap output and non-finite budget inputs.

**TDRP-MATH-06 — Per-render macro/parser isolation.** Untrusted formula-defined macros/environments must never mutate the macro namespace used by a later transcript entry. Every strict render attempt receives a fresh macro scope, and a failed strict attempt must not share a mutable macro scope with its recovery render. Built-in/runtime trust policy remains source-neutral and immutable at the wrapper boundary: `trust=false`, bounded expression length, bounded expansion count/size and bounded returned markup apply to both attempts. This absorbs MicroTeX's built-in snapshot/reset, user-macro cleanup and parser-state restoration responsibilities without retaining its process-global C++ registry. Focused GitHub Actions contracts must prove fresh macro-scope identity across strict/recovery and across separate render calls, including simulated mutation before a strict failure; exact grammar compatibility for individual commands remains mapped-open rather than being inferred from parser isolation.


**TDRP-MATH-07 — Bounded structural group depth.** Before invoking the shipped math runtime, the canonical wrapper must reject untrusted formula input whose actual TeX brace-group nesting exceeds **48** after accounting for escaped braces and comments. This is a source-neutral preflight against native/JS parser stack exhaustion and complements the existing expression-length/maxExpand/maxSize/output budgets. The guard must run before both strict and recovery attempts, must not count escaped literal braces or braces inside comments, and must not claim closure for MicroTeX's separate brace-less recursive command-chain guard; that grammar-dependent responsibility remains mapped-open until the shipped KaTeX dependency is independently evidenced. Focused GitHub Actions tests must prove depth 48 is accepted, depth 49 is rejected before runtime invocation, and escaped/comment braces do not consume depth.
### Recursive component read-through 6,698-6,714 — MicroTeX entry/build contracts

Orders **6,698-6,714** exact-read the pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f` build/entry contract under `Telegram/ThirdParty/MicroTeX`. The applicable product behavior maps into the existing ConversationWorkspace/Transcript math owner and existing Build/Resource/Release provenance owners. Current production work adds a 32 KiB pre-parse formula bound and bounded 32 MiB LRU markup cache without retaining the C++/Qt/GDI runtime. Reading does **not** reduce `unknown`; current exact-head GitHub Actions evidence is still required. Accounting: **6,714/16,125 read; 9,411 unread; 15,845 unknown; 0 omitted**. Next: **6,715 `Telegram/ThirdParty/MicroTeX::readme/example_bw_false.svg@9e812c823d3cbe06ab43ee77f10b6122bd638dd`**.


### Recursive component read-through 6,831-6,852 — MicroTeX bounded layout/parser safety

Orders **6,831-6,852** exact-read the pinned MicroTeX unit/color, box/layout, formula/environment and glue implementation contracts. Applicable responsibilities are source-neutral: invalid unit/atom/style/table indices fail closed; NaN/Infinity color/scale/rotation/size values cannot poison renderer transforms; delimiter/arrow/box/matrix growth is bounded; partial parse keeps a non-null fallback; exceptional paths restore graphics/font/parser state; predefined formula/glue semantics remain deterministic. Build/config files preserve only dependency/platform provenance. These map to the existing canonical ConversationWorkspace/Transcript KaTeX owner plus Build/Release provenance; no MicroTeX C++/Qt runtime is retained. The canonical owner now also fail-closes non-finite custom budgets and has focused contracts for rendered-markup bounds, cache sharing/LRU eviction, loader isolation, invalidation and retry-after-rejection. Reading does **not** reduce `unknown`. Accounting: **6,852/16,125 read; 9,273 unread; 15,845 unknown; 0 omitted**. Next: **6,853 `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83`**.

### Recursive component read-through 6,853 — MicroTeX localized decimal input

Order **6,853** exact-reads `Telegram/ThirdParty/MicroTeX::src/core/localized_num.cpp@4f8e61fcc0f6f3c6d361cd835b4ceaa3a4ae5b83` from pinned `desktop-app/MicroTeX@61aaa7cc354de91d5898ffb0b2a6c62628d9a76f`. Its applicable responsibility is deterministic numeric input normalization: U+066B becomes the ASCII decimal point and the twenty accepted Unicode decimal blocks map digit-for-digit to ASCII while every unrelated code point remains unchanged. The existing canonical ConversationWorkspace/Transcript KaTeX owner performs that normalization before both strict and recovery renders, and the focused Desktop Chat Parity contract covers all twenty blocks, unchanged non-target text, and identical strict/recovery input. No MicroTeX parser/runtime is retained. Reading does **not** reduce `unknown`; descendant exact-head GitHub Actions remain required. Accounting: **6,853/16,125 read; 9,272 unread; 15,845 unknown; 0 omitted**. Next: **6,854 `Telegram/ThirdParty/MicroTeX::src/core/macro_def.cpp@0ccf9fe8b4028f97c62834555d2a3d344d0f8726`**.

### Live rebaseline 28ac5769 → 6fed91ff

Accepted upstream advanced by four commits to `6fed91ffab9861771a75f29df65031b3c80b6941` / root tree `4cf9e0e1830ff8569e3264d07062d8ff94dbb4f6` without adding or removing a root path. Ten root blobs changed at deterministic orders **3,155, 3,201, 3,202, 3,766, 4,521, 4,522, 5,166, 5,943, 6,541, 6,627**. Each changed blob was re-read against its predecessor object and reconciled through the explicit source-disposition rebaseline record. Version/resource/build changes preserve source-neutral release-version coherence. The history-photo delta adds owned enlarge eligibility/hit-testing plus press/release ripple lifecycle and keeps sponsored/editor/small-media exclusions; it maps to the existing Transcript/media interaction owner rather than a new media root. The media-viewer delta explicitly claims Save/SaveAs at the application event-filter boundary so competing shortcuts cannot consume Ctrl+S; it maps to the existing canonical media-viewer shortcut owner. The new style opacity is presentation-only under the Fabushi design system. The 7.3.1 changelog adds wallet Windows Hello/passcode, copy-restricted download, custom-theme date-color and photo-editor Link regression cues; those cues remain open until their existing Fabushi owners have shipping evidence. Recursive totals/order are unchanged, so read-through remains **6,853/16,125**, first unread remains **6,854**, unread **9,272**, unknown **15,845**, omitted **0**.
