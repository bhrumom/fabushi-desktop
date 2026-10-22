# OpenClaw 全源码 Rust 等价迁移与 Fabushi 原生融合 Spec

Status: active  
Spec ID: OCRM-001  
Revision: 1  
Last updated: 2026-10-22（Asia/Shanghai）  
Owner: Fabushi Desktop  
Canonical target: `bhrumom/fabushi-desktop` 当前 `main` 及从它推进的已核验实现分支  
Project: [`projects/openclaw-rust`](../../projects/openclaw-rust/SOURCE_OF_TRUTH.md)  
Implementation status: **规范已建立；全量源码闭包、迁移实现和生产验收均未完成。**

> 最终目标：逐文件、逐模块、逐责任理解 `openclaw/openclaw` 的完整源码与必要依赖，把它承载的全部产品功能用 Rust 等价实现到 current canonical Fabushi Desktop；保留并升级现有 Fabushi Bot 的全部能力，形成同一个完整产品、同一套架构和唯一的状态所有权。

本 Spec 是后续实现的规范，不是“已迁移”的证明。模块目录、同名 API、已存在的 crate、写好的映射、CI 工作流名称或本文件提交成功，都不得据此增加 `implemented`、`verified` 或 `accepted` 数量。

## 1. 背景与问题

用户于 2026-10-22 明确要求：把 `https://github.com/openclaw/openclaw` 逐文件、逐模块理解，使用 Rust 等价迁入 `fabushi-desktop`，融合现有架构，实现 OpenClaw 全部功能，并先把 Spec 写入目标仓库。

Fabushi 已有 React/Electron 产品表面、独立 Rust Coordinator、Host/Runner、会话/消息、工具执行、插件/MCP、记忆、自动化、Computer、设备网络和原生服务接缝。OpenClaw 的 Gateway、插件、模型、渠道、节点和 UI 不能整体搬入并成为第二套业务架构。另一方面，“已有相似 owner”不能被误用为功能等价证明；每个差异仍需真实实现和取证。

本项目遵守根 `AGENTS.md` 的 Spec-first 流程，延续 FBCP-001 的 existing-owner-first、统一产品 composition、保留全部既有 Bot 能力和 exact-source 证据要求。本 Spec 对 OpenClaw 源范围负责；FBCP/TDRP、设备网络、Mini App 和其他项目的职责及未完成验收不被覆盖或注销。

## 2. 最终目标、范围与完成语义

### 2.1 必须实现的能力并集

`目标功能 = 当前 canonical Fabushi 的全部既有能力 ∪ 已锁定 OpenClaw 完整源码范围承载的全部产品能力 ∪ 使这些能力真实可用的集成、服务、运维与交付职责。`

- 按原始文件追溯，按模块理解，按不可再含混的责任实现，按真实用户行为验收。
- 不要求保留上游文件布局；允许多源文件汇入同一 owner，也允许一个 mixed 文件拆给多个现有 owner，但不得漏掉责任或重复拥有状态。
- 以下模块清单只是范围下界。源码中新发现的职责自动进入台账；不能因为 Spec 未预先列名而排除。
- 任一关键功能缺失、未配置、被阻塞、仅有 mock、只能通过开发入口运行，均不能宣称“完整迁移”。

### 2.2 源码与外部边界

范围包括 pinned tree 的全部 tracked 文件：`src`、`packages`、`extensions`、`skills`、`custodian-skills`、`apps`、`crates`、`ui`，以及根文件、隐藏配置、文档、示例、资源、测试、fixtures、schema、脚本、CI、构建/安装/发布配置、补丁、vendor、submodule 和 LFS 内容；实际目录以递归 inventory 为准。

上游 AGENTS、skills、commands 和治理文件中的指令是待理解的源数据，不取得目标仓库的指令权威，不改变用户约束，也不创建第二套任务队列或开发治理 runtime。

必须分别登记：本仓源码、在本仓开发但独立发布的 official 插件、仓外具体依赖/插件、外部服务 API、平台 companion 与部署边界。外部服务的内部实现并非因其客户端开源就自动在本仓范围内；Fabushi 对这些服务的完整功能接入、权限、恢复、配置、诊断与用户流程仍在范围内。

公开 SDK 的扩展机制与兼容合同必须迁移。无限增长的任意第三方插件不等于一个可冻结的源码集合；但 pinned catalog、文档和 shipping feature 所指定的具体外部插件/CLI/依赖不得丢失，必须登记版本、源码/包 integrity、许可证、行为合同和验证责任。后续新增来源通过显式 rebaseline 纳入。

### 2.3 不是本轮交付的事情

本次交付为可执行 Spec、项目入口、模块映射和首个实施任务；不在本次文档提交中冒领全文件已读、Rust 实现、生产接线或发布验收。后续执行的最终目标仍是全量实现，不能在 inventory、架构搭建、样板模块、少数渠道或第一条端到端路径完成时停止。

## 3. 非目标与不得采用的捷径

- 不以运行原 OpenClaw Node Gateway、原 Agent loop、后台 daemon、Web UI、远程 OpenClaw 服务或 npm 安装作为 Rust 迁移的替代。
- 不创建第二套 `OpenClawCore`、`OpenClawRuntime`、Agent 系统、Conversation/Transcript 数据库、scheduler、插件市场、权限系统或设置中心。
- 不机械逐行翻译 TypeScript，不把原目录一对一复制成 Rust crate，也不因为上游已经有 Rust crate就直接接受其 owner 和生命周期。
- 不以 UI 重做为由排除 UI 文件中的业务、权限、状态、协议、恢复和可访问性职责；不复制上游品牌、页面骨架和素材来替代 Fabushi UI。
- 不借本迁移削减现有 Fabushi Bot、Human 消息、通信、插件、工具、Mini App、Computer、自动化或移动端互操作能力。
- 不为语言纯度重写通用密码学/媒体算法；可使用经审查的通用依赖，但原 OpenClaw 业务实现不得借 FFI/子进程继续持有生产责任。
- 不恢复 pinned 版本已删除的历史产品设计。升级/导入兼容仍需迁移；不能从旧博客或旧 README 重新引入已退休的第二调度器或 Tasks ledger。

## 4. 已核验的当前状态与权威规则

### 4.1 本次读取快照

| 项目 | 固定身份 | 含义 |
| --- | --- | --- |
| OpenClaw `main` | commit `e8cde7518c558f1b4e44cb8139a149289b9719ad` | 本次 discovery 输入，不是未来永远的 latest |
| OpenClaw tree | `25c8ee611b72762ae1a0170124fa1fee9e519704` | 所有本次上游引用固定于此树 |
| Fabushi `main` | commit `ba9ecce728ef021fee609796c8704d8af6fe4b5a` | 本 Spec 编辑前的目标架构基线；文档提交后须重读 live main |
| Fabushi tree | `29e6837e6ea9098d64c5de3e0952b3960abbfc86` | 本次 owner 研究输入 |
| OpenClaw 根 LICENSE | blob `ed064819ab537575d0cad902f7e5af4a3488b1d0` | MIT，另须审计第三方 notices 与依赖 |
| OpenClaw plugin inventory | blob `e575a2790c20d14b196fd4956619c384e343294c` | 本次文档列 73 core、99 official external、3 source-only 插件 |

本次目录级发现为 `src` 77、`extensions` 176、`skills` 50、`custodian-skills` 4、`packages` 24、`apps` 8、`crates` 2 个直接子目录。`extensions/test-support` 不是第 176 个产品插件；`apps/.i18n` 不是独立平台。上述计数不是递归文件总数，不证明 source completeness。

OpenClaw 当前协议为 v4；authenticated node 和轻量 probe 有 N-1/v3 窗口。包版本与 wire version 不同。当前 automation 文档已经移除 shared Tasks ledger、TaskFlow API/webhooks 和 inferred commitments；旧 heartbeat monitors 迁入普通 automation jobs。实现必须以 pinned 源码和合同重新核对，不凭历史印象设计。

### 4.2 当前 Fabushi 接线事实

- `frontend/src/main.tsx` mount `frontend/src/production/ProductionRenderer.tsx`；`source/electron-main/entry.ts` 装配生产 adapters 和 Coordinator 连接。
- `source/node-agent-coordinator` 与 `source/host/app` 是独立进程边界；`source/host/src/lib.rs` 明确 accepted-turn、stream/retry/checkpoint、cancel/settlement 属于 Host/Runner。
- `source/host/src/host_production_extensions.rs` 装配 Auth、Settings、Inference、Memory、Session、Transcript、Automations、MCP 等已有 owner。
- 当前 main 已有 Human 消息和 CallSession 接缝，不能照抄旧 owner inventory 中的“尚不存在”结论。
- 外部 channel 存在 `connectors/channel_delivery.rs` 接缝，但其 default 是 `UnregisteredChannelDeliveryPort`；Host app 本次读取仍注入 `HostChannelDelivery::default()`。这不能证明任一 OpenClaw channel 已生产可用。
- 当前 main 的 Browser/Memory/Plugins 等源码存在，不等于其已覆盖全部 OpenClaw 行为，也不等于取得当前 HEAD 的 packaged acceptance。

这些是只读源码观察。未在本次执行任何构建、测试或产品验收。

### 4.3 并行工作及 source authority

本次读取时 PR #165（Browser Workspace）、#166（Mini App authoring）、#167（Hermes hardening）、#168（Mini App publication）均未合并；#166→#167→#168 是依赖链。它们是候选实现/协调依赖，不能当作 canonical main 或已验收能力。TDRP PR #154 是独立进行中的工作，不得覆盖其分支、lock、inventory、ledger 或既有 evidence。

每轮工作先重新读取 target main、实际工作 PR/ref/head/tree、相关 source/spec/lock/blob 及上游 ref。优先级为：最新明确用户要求；适用的仓库 instructions；本 Spec 与相关领域规范；exact-head 实际生产源码/合同；本项目 lock/inventory/ledger/dossier/evidence；历史记录。事实冲突先修规范或映射，不能把旧文档中的路径、缺失判断或 CI 状态当成实时事实。

写入前重新读取实际修改文件的 blob 与 branch lease，按 base tree 创建原子 commit；更新 ref 使用 `expected_sha`，不 force 覆盖并行工作。lease 变化时重读差异后重做，不盲目重试旧 tree。上游更新经显式差异审计与 rebaseline；当前基线不在循环中偷偷浮动。

## 5. 核心需求

| ID | 规范性要求 |
| --- | --- |
| OCRM-R01 | 全部 tracked source 与必要递归依赖纳入 inventory，零遗漏；每个文件有明确处置与责任映射。 |
| OCRM-R02 | 每个文件必须阅读全文或完成与其类型匹配的完整检查，逐函数/类型/状态机/副作用分解；目录级摘要不能替代逐文件理解。 |
| OCRM-R03 | 全部产品能力等价实现；功能矩阵是下界，不允许按“暂时用不到”、复杂度、平台、实验标记或外置发布裁剪。 |
| OCRM-R04 | 所有迁入的业务、协议、路由、策略、调度、状态、持久化、恢复和执行控制由 Rust 实现并成为真实 owner。 |
| OCRM-R05 | 保留当前 Fabushi 全部能力，以 existing-owner-first 融合；禁止平行产品/状态/执行真相。 |
| OCRM-R06 | Coordinator、Host、Runner、box-exec、Electron、renderer、服务端与设备边界保持清晰，新增职责不能堆进单个巨型 owner。 |
| OCRM-R07 | 必需且确实缺失的 owner 经 rejected-owner analysis 与 ADR 建立最小 source-neutral owner，不得因此丢功能。 |
| OCRM-R08 | 每项迁移形成 source→requirement→contract/oracle→target symbol/artifact→按责任类型的生产或交付接入→evidence→acceptance 的双向追溯；运行时责任必须含真实 producer/owner/consumer。 |
| OCRM-R09 | 既有 Conversation、Transcript、Participant、Resource、Permission、Task/Automation、Settings 与 Marketplace 各自保持唯一权威。 |
| OCRM-R10 | Gateway/API/SDK、插件、渠道、模型和节点的公开兼容合同完整迁移；兼容入口只做协议投影，不拥有第二份业务状态。 |
| OCRM-R11 | 外部账号、服务、设备、安装/升级和运维闭环完整；真实依赖未就绪必须 blocked，不可伪造成功。 |
| OCRM-R12 | UI 使用 Fabushi 现有 shell、design system 与 typed composition；每个功能有可发现的主要入口和完整状态矩阵。 |
| OCRM-R13 | account/tenant/agent/conversation/device/resource 隔离、工具授权、凭据生命周期和信任边界保持或增强，不能以等价为由复制已知不安全缺陷。 |
| OCRM-R14 | streaming、cancel、timeout、restart、reconnect、stale generation、乱序/重复及 outcome-unknown 都有状态机和真实验证。 |
| OCRM-R15 | 原有数据无损升级；OpenClaw 用户配置/会话/技能/记忆等迁移采用显式映射、可预览导入、事务、备份和可恢复流程。 |
| OCRM-R16 | 全部可执行验证只在 GitHub Actions，证据绑定 exact target/upstream/source closure 与 artifact provenance。 |
| OCRM-R17 | file read、responsibility mapped、implemented、生产/交付接入、verified、按类型 accepted 分开记录，禁止状态跳级；运行时责任仍必须 packaged accepted。 |
| OCRM-R18 | 许可证、第三方 notices、来源和 SBOM 完整保留；产品表面统一 Fabushi，合法第三方服务名和兼容字段不做盲目替换。 |
| OCRM-R19 | 所有官方插件、内置技能、管理技能和本基线指定的外部集成逐项有功能、权限、依赖与验收记录。 |
| OCRM-R20 | 全部支持平台、companion/node、无头部署和服务运维有明确交付矩阵；没有 runner/设备/签名不等于不适用。 |
| OCRM-R21 | 有界资源、性能/功耗/磁盘增长、可观测性和数据保留要求在实现前量化，不能由“Rust 应该更快”代替证据。 |
| OCRM-R22 | 阻塞记录原因、解除条件与支持请求，继续无依赖工作；不得改门槛、虚构结果或无限重复同一失败手段。 |
| OCRM-R23 | 新源码/新能力与缺陷反馈进入同一 canonical 台账和回归链；历史证据保留，受影响项重验。 |
| OCRM-R24 | 完整迁移必须满足源完整性、功能实现、UI、服务/生态、数据/发布与独立验收全部闭合。 |

## 6. 全量源码盘点与逐文件迁移合同

### 6.1 基线锁定

实施 P0 在 GitHub Actions 中生成并校验 `projects/openclaw-rust/upstream.lock.json`、`inventory/index.json` 与分片、`parity-ledger.json`、`requirements-traceability-matrix.json`、`dossiers/`、`evidence/` 索引。它们是本项目审计材料，不能另建产品 runtime。复用现有 source-neutral authority/quality 机制；TDRP 自身的源范围和计数不得被 OpenClaw 混入。

锁定字段至少包括 repo URL、ref、commit/tree、取得时间、各 submodule 的 repo/commit/tree、LFS oid/内容 digest、包/依赖版本与 integrity、patch SHA、平台条件、源码 archive digest、规范 revision、目标 baseline、生成器版本和对应 Actions run。

仓库文件记录已知的 `target_baseline`、输入身份与历史结果引用；本次实际受验 `subject_target_commit/tree` 由 Actions receipt 从真实 checkout 注入。不能要求一个提交在自身内容中嵌入尚未产生的自身 SHA。完整的受验身份、证据载体与最终发布流程见 §12.3。

Git tree API 被截断时必须分子树继续枚举并核对集合；GitHub search 结果、目录前一页、npm 发布文件清单和 `.gitignore` 不能作为完整源码集合。只枚举产品默认安装文件也不合格。对 symlink 分别保存 link blob 与解析关系，拒绝逃逸/循环；gitlink/远端下载物需单独闭合。

### 6.2 文件记录

每个路径至少记录：`source_repository`、`source_commit`、`source_tree`、`path`、`blob_sha`、`mode/type`、`size`、`language/kind`、`module_id`、`dependency_edges`、`read_state`、`read_ranges/content_identity`、`responsibility_ids`、`disposition`、`license/provenance`、`reviewer`、`evidence_refs`。

同一 blob 位于多个路径时，可引用同一内容阅读记录，但路径用途、caller、构建条件与责任仍分别登记。不能重复计算 source coverage。生成文件必须追溯 schema、template、generator、使用方和生成结果；可以重建实现方式，不能省略生成出来的合同。二进制/资源按完整内容身份、用途、交互/版权/替换关系检查，不伪造行级阅读全文。

### 6.3 阅读与责任分解

每次完整阅读必须回答：输入/输出；公开与内部语义；顶层副作用；持有状态；调用者/消费者；异步/线程/生命周期；错误/重试/取消；持久化；权限；平台条件；兼容/版本；依赖/许可证；验证 oracle。对有多职责的文件拆分稳定 `OCRM-RESP-*`，覆盖函数/方法、注册器、事件处理、类型约束、默认值、特殊分支及跨文件关系。

不能以 AST 符号数、注释摘要、读取前若干行、仅测试成功、粗粒度目录 row 或自动生成文本证明理解。大型文件按有记录的完整区间阅读，遗漏区间保持 unread。测试文件中的每个独立行为/边界 oracle 也要映射；fixture 与 mock 的能力边界不能被当作生产行为。

合法处置包括 Rust rewrite、reuse-and-extend-existing-Rust、presentation-replaced-with-behavior-preserved、platform-bridge、schema/build/test responsibility、licensed-asset replacement，以及有逐项依据的 non-product disposition。所有处置仍进入 inventory 分母。`not-applicable` 只能用于经源证据和审查确认无本产品责任的具体事项；不能用于缺账号、缺服务、缺平台、工作量大、source-only 或暂未实现。

### 6.4 责任记录与状态

每条责任至少包含：`id`、`responsibility_kind`、源 path/blob/symbol/range、行为描述、前后置条件、不变量、capability/requirement IDs、source oracle、所有候选 existing owners 与选择原因、target path/symbol/artifact/language、commands/events/schema、state/lifecycle、dependencies、按类型的 integration path、`required_evidence_dimensions`、`required_gate_ids`、测试、差异 ADR、迁移/回滚、风险、阻塞、当前证据。运行时 integration path 必须记录 production entrypoint/producer/owner/consumer。

| 维度 | 可记录状态 | 升级条件 |
| --- | --- | --- |
| 来源读取 | unread / partial / read-complete | 完整内容与责任阅读记录；没有未读区间 |
| 责任分析 | unclassified / mapped / reviewed | owner、行为、依赖、oracle 和具体覆盖经复核 |
| 实现 | not-started / implementing / implemented | 按 §6.4.1 类型合同有真实实现或交付物；业务责任必须 Rust，不是 stub |
| 生产/交付接入 | not-wired / wired | 对应类型的真实 consumer 已接入；运行时必须实际 packaged entrypoint 可到达 owner 和 consumer |
| 验证 | not-run / failed / blocked / verified | exact-target receipt 绑定的 Actions 与类型所需审查证据满足该责任全部 oracle；可执行检查均在 Actions |
| 责任验收 | pending / rejected / accepted | 独立审查类型合同的全部证据；运行时、表现和平台职责必须包含 packaged 用户路径及所需服务/设备/升级证明 |

状态独立维护。旧代码的 `reused` 不等于 `verified`；上游测试移植不等于等价；有测试通过不能抹去未覆盖分支。变更 source/spec/owner/oracle/dependency 后把受影响证明失效，保存历史身份和原因。

### 6.4.1 按责任类型闭合，不从分母删除

| `responsibility_kind` | 实现合同 | 接入与验收合同 |
| --- | --- | --- |
| `business-runtime` | 全部业务、协议、策略、状态、持久化、调度和执行控制由真实 Rust owner 实现 | 真实生产 producer→owner→consumer；Actions 合同/差异/故障测试与 packaged 用户、服务或 CLI 路径 |
| `presentation` | 可使用既有 React/Electron 表现技术；关联 authoritative Rust 业务 owner 和 typed contract，无独立业务真相 | 真实产品 route/view/action 消费该合同；UI 功能、状态连续、visual/a11y 等适用证据 |
| `platform-bridge` | 仅 OS/API 必需的薄桥接可保留平台语言；业务责任另拆为 Rust responsibility，写明调用、权限与生命周期 | 真实 Rust↔平台接线；Actions 管理的对应 OS/device lane、授权与资源释放证明 |
| `schema-generated` | 保留 wire/schema/默认值/约束及生成关系；格式与生成工具可按现有构建体系选择，业务语义仍属 Rust | schema→generator→产物→实际消费者；Actions 生成/兼容检查及适用的分发证明，不能只验生成器退出码 |
| `build-release-operations` | 构建、安装、更新、发布、运维脚本按现有工具链实现；其中运行时业务拆出 Rust 责任 | 真实 workflow/installer/service 消费；Actions 中构建、分发、安装、升级、回滚等适用结果 |
| `test-oracle` | 独立行为 oracle、fixture、负例与故障条件有明确替代；上游 Node oracle 只在隔离 Actions lane | 真实 CI lane 执行且确实检验被映射行为，报告覆盖与失败能力；不能要求测试本身进入生产 package |
| `documentation-governance` | 文档、操作合同、示例或有理由的不转入处置保留准确来源与行为关系 | canonical 文档/配置/开发流程中的实际引用或逐项处置证明；源审读、G02 引用/状态检查及相关产品责任证据，不虚构 runtime 入口 |
| `asset-license` | 二进制、图标、字体、模板、版权与 notices 有内容身份、替换关系及合法 provenance | 应分发内容进入实际 artifact/NOTICE/SBOM；不分发项有经审查处置；Actions 包内容证据与许可审查 |
| `non-product-disposition` | 对具体责任给出完整源证据、适用性分析和独立审查；不能用于未实现或受阻功能 | G02 校验处置及引用；不要求不存在的运行时实现，但该责任仍在分母，审查不通过即未闭合 |

mixed 文件必须拆出所有不同类型的责任，不能把带业务的 UI、脚本、插件、schema 或文档整体改标为非运行时。`dimension-not-applicable` 仅用于上述合同确实不要求的某个维度，记录 `dimension/reason/source_evidence/reviewer`；缺少必要的实现、账号、服务、设备或证据不是维度不适用。所有 responsibility 仍需完成自己的实现/处置、接入/消费、验证和独立验收合同；某维度不适用不等于整个责任已接受。

文档和许可判断可以采用独立只读审查记录；它们的机器身份、映射、分发等可执行检查仍只在 Actions。状态投影由 §12.3 的 exact-target receipt 绑定这些输入和审查记录，不能编造测试或要求一份纯文档被编译成 Rust。

### 6.5 无遗漏守恒与推进顺序

Actions 必须报告完整源集合、逐路径处置、责任集合与双向映射，检查 omitted/orphan/ambiguous/duplicate/unread/unmapped 等集合。所有 totals 由已锁定集合计算，禁止手填让总数“好看”。未建立递归总数时写 `unknown`/`null`，不能写 0。

先按 dependency graph 定义可实施单元，单元内依稳定的 source-root/path/symbol 顺序推进。保留首个未完成的 strict responsibility 和 blocked frontier；可以并行阅读、设计和实施依赖满足的其他单元，不能跳过阻塞项后谎报连续覆盖已完成。

## 7. 目标架构与唯一所有权

### 7.1 Rust 与界面边界

**Rust 承担全部迁入的产品业务语义**，包括 Agent/工具控制、模型路由策略、协议/序列化、渠道状态机、调度、记忆、持久化、权限、恢复与兼容。现有 React/TypeScript 和 Electron 保持视图、交互投影、系统 shell 与必要桥接；不能让新增 renderer TS 再持有业务状态真相。UI 文件里的业务职责必须提取到对应 Rust owner，通过 typed contract 投影。

Swift/Objective-C/Kotlin/Java、平台 SDK、JS/WASM guest 等仅在 OS/API 或被支持的用户扩展确实需要时作为最薄边界，逐项记录原因、权限和生命周期。内置/官方上游业务插件不能因插件形式逃避 Rust 迁移。第三方 guest 执行是受控扩展能力，不是保留原 OpenClaw host 的借口。

### 7.2 现有 owner 的优先落点

具体清单和缺口见 [`module-map.md`](../../projects/openclaw-rust/module-map.md)。下表是本次源观察后的优先检查方向，后续必须对 actual implementation HEAD 重核，不能仅凭此表升级状态。

| 职责 | 首先检查/扩展的现有 owner |
| --- | --- |
| shell、导航、会话/设置/市场界面 | `frontend/src/production`、`frontend/src/recovered/features`、`source/electron-main` |
| 进程监督、typed routing、relay | `source/node-agent-coordinator` |
| accepted turn、推理/工具、子 Agent、settlement | `source/host/src/runner`、`host_runner_composition.rs`、`extensions/inference` |
| 会话、历史、原生消息、call 状态 | `extensions/session`、`extensions/transcript`、`transcript_mirror`、`native/mahayana-messaging` |
| 外部渠道与资源投递 | `source/shared/channels.ts`、`channel-messaging.ts`、`source/host/src/connectors`，经真实注册扩展 |
| Skills/Plugins/MCP/安装与市场 | `extensions/mcp`、`workflows`、既有 Electron/Coordinator MCP relay 与 Plugins surfaces |
| 自动化/事件/后台唤醒 | `automations`、`extensions/automations`、`runner/background_work.rs` 与既有 pending-wake owner |
| 记忆/检索/合成 | `extensions/memory`、`extensions/content_search`、`runner/turn_memory.rs` |
| 审批、沙箱、本地/远程执行 | `extensions/local_tool_permission`、`extensions/auto_review`、`extensions/action_audit`、`extensions/local_exec`、`box`、`source/box-exec-daemon` |
| Browser/Computer/节点 | `runner/tools`、`runner/computer_use.rs`、`remote_computer_target.rs`、`network_enrollment_owner.rs`、`source/fabushi-network-worker` |
| 身份、配置、凭据、存储 | `extensions/auth`、`extensions/settings`、`extensions/secrets`、`storage`、`agent_isolation` 与既有 Electron secure-storage bridge |
| Mini App/Canvas/交互产物 | `extensions/mcp/installable_package.rs`、既有 Mini App host/window、Resource 和 transcript-card owners |
| 服务端、移动 companion、共享协议 | `source/mahayana/mahayana-rs` 中现有 platform worker、mobile host 和 host protocol owners |

所有权不是“一切塞进 Host main”。应扩展对应小模块，维持公开 API、依赖方向和域状态边界。新增 owner 的 ADR 必须列出被拒绝的合理候选、唯一 state、commands/events、storage、lifecycle、process placement、security、consumer、迁移/cutover 和验证。

### 7.3 统一产品与外部互操作

每个 Conversation/Transcript/Composer/Profile/Search/Resource/Settings/Marketplace/Automation 概念使用唯一 canonical composition。保留消息、联系人、插件市场的一级入口和现有左竖栏/列表/主工作区设计；新功能按任务作用域进入现有 section/action/panel/overlay，不按上游模块制造一套 OpenClaw Dashboard。

本项目明确包含 OpenClaw 已提供的可选外部渠道互操作，例如 Telegram Bot API、Slack、Discord、WhatsApp。它们作为现有渠道/插件 owner 的 Rust adapter，不是 Fabushi 原生通信网络，也不能使 Fabushi 登录、Human 消息、身份或存储依赖外部账号。本条明确这次 OpenClaw 全功能要求的互操作范围，不把 FBCP/TDRP 改成 Telegram/MTProto 迁移，也不另建 Telegram 产品。

Gateway 兼容 WS/HTTP、CLI、SDK 或 companion adapter 可以保留必须的外部 wire 名称、字段和 URL 兼容入口，但只翻译/授权/投影到当前 owner；不得再有自己的 session DB、权限真相或调度循环。能力发现不授予权限，外部身份必须映射到明确 scoped principal。

## 8. 必须逐项展开的功能范围

[`module-map.md`](../../projects/openclaw-rust/module-map.md) 的 OCRM-M01 至 OCRM-M32 是实施分组，每组仍必须展开到全部文件/责任；不是任选范围。

### 8.1 Agent、模型与上下文

- Agent 创建、配置、身份/persona、工作区/项目绑定；embedded loop、所有支持 harness、ACP/A2A、native/hosted session、模型与账户选择、custom endpoint、本地模型及 provider fallback。
- streaming text/reasoning/tool、工具参数修复、审批、重试分类、上下文窗口、compaction、截断/溢出、token/cost/usage、模型能力差异、静默回复、stop/cancel/steer、pending user input、最终结果稳定。
- session branch/fork/reset/history/export、多 Agent/group/swarm/subagent、parent lineage、递归/预算限制、并发队列、任务完成唤醒、background continuation、advisor/autoreview 与工具循环/重复输出防护。
- 模型目录、认证/OAuth/credentials、个人/系统/团队模型账户隔离、refresh/revoke、故障转移不重复外部副作用。确定性协议语义与非确定性模型文本分开验收。

### 8.2 Channels、协议与投递

完整覆盖 pinned channel catalog、所有 official 插件及明确外部插件合同。至少包括 A2A、Buzz、ClickClack、Discord、Feishu、Google Chat、iMessage、IRC、LINE、Matrix、Mattermost、Microsoft Teams、Nextcloud Talk、Nostr、QQ、Raft、Reef、Signal、Slack、SMS/MMS、Synology Chat、Telegram、Tlon、Twitch、WebChat、WeChat、WeCom、WhatsApp、X、Yuanbao、Zalo/Zalo ClawBot/Zalo personal；精确 membership 和支持粒度以 manifest/source 为准。

每个渠道分别覆盖账号设置、登录/QR/OAuth、inbound transport、DM pairing/allowlist、群组/线程/提及、成员/角色、编辑/删除/反应/附件/富交互等上游实际支持的动作、分块/流式输出、格式转换、rate limit、重试/幂等/回执、checkpoint/reconnect/logout/撤销，以及 UI 与诊断。各平台能力不同，不能虚构上游不支持的动作，也不能仅因部分动作不支持就排除整个渠道。

保留 bot-loop protection、ambient room events、join-introduction 的触发条件、工具禁用、内容不可信标记、历史可见性、一次性/TTL 去重与重启行为。配置项只在被真实实现的渠道上接受，不能 silently accept 后不生效。一次 Agent 运行成功与 outbound delivery 成功是不同结果。

### 8.3 插件、技能、MCP 与生态

全部 175 个插件的 pinned manifest/entrypoint/SDK/运行与卸载职责、50 个 skills、4 个 custodian skills 均需单列，不由目录分组直接宣称完成。source-only QA 项目承接测试/诊断职责；visitor-access 有真实权限语义，不因 source-only 标签而消失。

覆盖发现、安装、检查、启停、更新、卸载、版本/依赖/锁定、schema/config、权限、exclusive slots、注册工具/commands/routes/services/hooks、动态重载、取消/dispose、缓存失效、故障回滚、市场搜索、发布/校验、CLI/GUI/API 对等操作。静态读取 manifest 不得执行任意 package code；实际执行通过既有审批和隔离边界。

MCP client/server/relay、stdio/SSE/HTTP、工具/resources/prompts、schema、认证、分页、event subscription、signed callbacks、取消/reconnect、账号隔离与工具发现均需逐契约核验。MCP 兼容不等于 OpenClaw plugin SDK 兼容。JS/TS SDK guest 必须有受限 capability bridge、版本适配、跨语言序列化、生命周期和真实插件 conformance；缺失任何 hook 不可冒领兼容。

Skills 中的文案/结构可保留为内容格式；其中脚本、参数、安全策略、OS/CLI 集成与工具调用的业务责任仍要迁移到 Rust 或批准的薄平台适配层。不能只复制 SKILL.md 就把整个技能标完成。

### 8.4 Automation、记忆与工作组织

- at/every/cron、on-exit/stream/event、HTTP webhook、MCP/IMAP 事件、hooks、standing orders、Lobster resumable approval、声明式 job、activeHours/idleOnly、scratch、context/owner/grant、quiet/no-reply、失败通知。
- IANA timezone、系统/用户 fallback、DST gap/fold（含非一小时变化）、cron 边界、clock rollback、sleep/resume、catch-up、schedule edit revision、去重、并发和触发来源 identity。当前 heartbeat 迁为普通 jobs，保留迁移与用户编辑/删除，不再添加第二调度器。
- memory-core/LanceDB/active-memory/memory-wiki、embedding/retrieval/flush、Forget/Remember、来源/有效期/权限/删除传播、wiki/vault 编译、历史修订与 Obsidian 等互操作。群组内容不能泄露到私有记忆或其他 agent/account。
- boards/projects/claws/workboard、workboard-contract、team-reports、logbook、session cards/sharing、trajectory/snapshot、read-only Beam、GitHub previews/publication 及上下文绑定。明确这些当前职责与已移除 TaskFlow/旧 Tasks ledger 的区别。`src/flows` 则按实际 setup/onboarding/doctor/repair 职责分配给配置、认证、模型、渠道、记忆/搜索、CLI 与维护 owner，不按目录名当作 board workflow。

### 8.5 执行、浏览器、Computer、节点与多模态

- exec/PTY、读写/编辑/apply-patch、workspace/worktree、process supervisor、background jobs、输出截断/溢出、filesystem confinement、网络策略、审批、资源 lease、Code Mode/QuickJS/WASM、cloud worker/sandbox provider、结果与资源释放分别结算。
- 浏览器 CDP/扩展/profile/login/session、navigate/read/action/screenshot/download/upload、tab ownership、browser cleanup、remote worker、computer act 和 desktop resources；安全检查、坐标/viewport/generation、页面权限与取消恢复必须接既有 Computer/tool owner。
- macOS/iOS/Android/Linux/headless nodes 与已声明 Windows 边界、pairing/revoke、capabilities/commands、camera/screen/location/notifications/device actions、mobile wake/background/process death；节点宣告与实际可执行授权一致。
- 音频/语音识别、实时转写、TTS、voice wake/Talk、打断、设备切换、image/video/music 生成与理解、document extraction、link understanding、Web Fetch/Search、媒体格式/限额/资源生命周期。
- Voice Call、Google Meet、Teams Meetings、Zoom、Slack Huddles、FaceTime 等上游已有会议能力；加入/退出/撤销、录制与转写权限、speaker/time provenance、断线/后台恢复、通话与 Agent 之间的授权边界。平台或账号缺失是验收阻塞。
- `src/transcripts` 的音频/会议 source、capture/provider、participants/utterances、occupancy、summary revision、export 和 cleanup 归音频/会议及资源职责；其输出可进入统一 Conversation/Transcript，但不能把 capture/source authority 当成聊天历史 owner。聊天历史从实际 Agent/session/config/Gateway 路径逐项追溯。

### 8.6 UI、配置、安全、部署与运维

Control UI/CLI/TUI/wizard/chat/slash command/快捷操作、设置、模型/渠道/节点/插件管理、诊断/doctor、session share、public transcript、team/visitor、权限审批、Canvas/A2UI/交互产物、diff/diagram/markdown 的全部交互能力进入现有 Fabushi 产品入口。

覆盖 JSON/JSON5/config/env/profile precedence、版本迁移、验证、热更新、回滚与原子 generation；credential/SecretRef/Vault/1Password、网络访问策略、sandbox、audit、telemetry consent、OTel/Prometheus、日志脱敏/保留、备份/恢复/导出、SQLite/PostgreSQL/命名存储位置、服务/daemon/headless、安装/update/signing/package/release/rollback。

源码文档/脚本里暴露的运维行为也是责任：可发现的错误、健康探测、启停、迁移、卸载、远程访问、TLS/代理/SSH/设备网络配置不能因不在主 UI 而遗漏。禁止把上游默认开发环境当成生产安全和部署证据。

## 9. 接口、状态机与数据流合同

所有以下名称为语义要求，具体类型/字段优先扩展既有合同，不能为照抄此表创建第二套模型。每个 dossier 写出真实 API/schema/错误码、version、limits、owner 与兼容矩阵。

| ID | 合同 | 必须保持的语义 |
| --- | --- | --- |
| OCRM-C01 | Principal / scope / grant | 明确 account、tenant、human/agent、conversation、device、capability 和授权来源；第三方消息/工具结果/skill 内容不能自行提升权限。 |
| OCRM-C02 | Command / operation identity | 稳定 request/operation/idempotency identity、target/generation、deadline/cancel、schema version；重复请求不重复副作用，未知结果先 reconcile。 |
| OCRM-C03 | Event / snapshot | 单调 revision/sequence、去重/gap recovery、snapshot/live 交接、account/generation fencing；旧 snapshot 不能覆盖新 terminal。 |
| OCRM-C04 | Turn / task / delivery | accepted→running/streaming/tool/approval→terminal→settled；执行、投递、资源释放分开；取消不等于撤销已发生副作用。 |
| OCRM-C05 | Channel adapter | 每个 account 的 ingress cursor、thread/recipient/attachment mapping、retry/receipt/logout、权限与错误；只通过 canonical Conversation/Resource 接口投影。 |
| OCRM-C06 | Provider / harness | capability/model/auth、context/messages/tools、typed stream、usage、retry/fallback/interrupt、session identity；不得把 sessionKey 与 sessionId 混用。 |
| OCRM-C07 | Plugin / skill / MCP | manifest/schema/version、registration、capability grant、owned tasks/timers、start/update/stop/dispose、events、guest serialization 与卸载回收。 |
| OCRM-C08 | Automation | schedule/event/stream source、timezone、next-run、batch/match、job revision、execution attempt、delivery receipt、grant/context/catch-up；同一 stream schedule 的 child restart 保留持久 `streamSourceIdentity`，disable/remove/replace 原子轮换并拒绝旧来源事件；child/process lifetime 或执行 generation 单独隔离旧实例迟到回调，不能用重启时轮换 source identity 代替实例隔离。 |
| OCRM-C09 | Node / worker / device | pairing/device identity、capability manifest、command policy、generation/lease、resource quota、pending wake、revocation、结果/cleanup。 |
| OCRM-C10 | Store / import / migration | 唯一 writer、schema/version、事务/backup、import provenance、conflict/idempotency、rollback、delete/retention、recovery。 |
| OCRM-C11 | Gateway compatibility | v4 及已定义 v3 node/probe 窗口、connect challenge、roles/scopes、method-specific validation、events/errors/limits、无权限不能 advertise 或 invoke。 |
| OCRM-C12 | UI projection | typed route/section/action、loading/empty/error/offline/permission states、accessibility/focus、draft/selection/scroll 与运行中任务连续性。 |

典型路径：external channel/UI/automation/node intent → 现有 account/permission boundary → 当前 domain owner → 必要时 Coordinator→Host/Runner 或既有 tool/box → canonical durable event/resource → 同一个 transcript/consumer → 独立 delivery/settlement。

Gateway compatibility 必须遍历 server dispatch、plugin registration、schema registry 和 role-specific methods；`hello-ok.features.methods` 只是保守发现列表，不是全部 callable methods。未知 additive field 的保留/拒绝由版本合同决定；缺字段、null、空数组、未知 enum、integer range、UTF-8/UTF-16 offset、binary/base64、时间单位均需跨语言 oracle。

对于本地 state owner routing，保留 `OWNER_UNAVAILABLE`、`OWNER_REFUSED`、`OUTCOME_UNKNOWN` 的区别。已分派 mutation 的 timeout/error 不能证明 rollback，不能触发第二 writer fallback 或自动 replay。协议支持不授权停止/替换另一个进程；ownership、lease、真实 process identity 和 readiness 必须独立核验。

## 10. 非功能、安全与故障要求

### 10.1 安全与隐私

- 身份/权限在 Rust authoritative owner 校验，renderer hidden control、模型口头同意和 manifest 声明均不是授权。
- 外部消息、网页、文档、工具输出、仓库文件、技能和插件输入按 trust level 处理；prompt injection 不能改变 granted capabilities 或确认外部发送。
- 工具 approvals 绑定实际 operation、target、参数摘要、scope、有效期和 generation；修改参数或切换账户使旧批准失效。
- 凭据按现有 secure-store/auth owner 的种类边界存储；不把所有 secret 混进通用设置或 UI state。日志、trace、fixture、artifact 和导出脱敏。
- webhook 校验认证/签名、时钟窗口与重放；URL/SSRF、redirect、DNS、loopback/private-network、文件路径/symlink/archive traversal、上传大小、MIME 与 origin 权限有明确合同。
- 插件包的 integrity/provenance、安装脚本、依赖污染、动态重载、撤销/卸载、跨进程 capability、guest 时间/内存/网络限制均需实证。
- upstream 的不安全行为不能无条件复制；修复需登记精确差异、理由、兼容影响与测试，并保留目标功能。

### 10.2 性能、容量与可靠性

P0/P1 为每个高风险模块建立 workload 和明确数值预算，至少含 cold/warm startup、P50/P95/P99 latency、stream first-event/settlement、吞吐/队列长度、memory/CPU、idle/background power、日志/缓存/数据库/磁盘增长、large history/search、附件上限、连接/Agent/任务并发、长时间 soak 与 failure recovery 时间。数值依据 current Fabushi 和 pinned upstream 在同类 Actions runner 的测量与产品要求制定；未测量/未批准就是未满足，不能留空后验收。

bounded queues/cache/retry、backpressure、cancellation-safe async、timer/task ownership、graceful shutdown、资源 quota 与 cleanup 必须完整。资源仍被 native operation 或 preparation 使用时，不可因收到 cancel 提前归还 lease；执行终态与资源可重用时刻分开。

### 10.3 必测故障与边界

| 场景 | 要求 |
| --- | --- |
| accept 前后断网、timeout、reply 丢失 | 区分未执行、明确拒绝、未知结果；稳定 identity，先状态查询/reconcile 后决定重试 |
| duplicate、乱序、late event、snapshot/live race | 唯一 event/order truth，terminal 不回退，不串会话/账户 |
| 用户 cancel/stop、审批拒绝/超时 | 停止后续授权动作，保留已发生副作用和清理责任，可恢复状态真实可见 |
| Host/Runner/Coordinator/插件/节点 crash | generation fencing、唯一恢复者、checkpoint/lease、pending wake 与 outcome-unknown 不丢失 |
| sleep/resume、时钟回拨、DST gap/fold | 不重复触发、不遗漏规定 catch-up、不把事件触发器当时间任务 |
| 账户切换、设备撤销、token refresh | 旧 requests/subscriptions/caches/approval/secret reference 失效；不得跨账户投影 |
| 插件 update/reload/uninstall 失败 | 完整 generation 原子发布；旧任务生命周期可收敛；禁止新旧模型/认证/插件状态混用 |
| storage full/corruption/migration 中断 | 不伪造写入成功，保持可诊断和备份/恢复策略，不丢既有数据 |
| provider/rate limit/service/permission failure | typed error 和退避，保留使用量/副作用/投递的真实结果 |
| 打包缺 binary/schema/native lib 或协议不匹配 | 正确 blocked/fail，不能偷偷回落开发服务器或原 OpenClaw runtime |

## 11. 实施阶段、任务颗粒度与阻塞推进

阶段表达依赖顺序，不减少最终范围；每个实施任务都要有源 responsibility、owner、合同、UI/服务落点、验证和退出条件。

| 阶段 | 主要成果 | 退出条件 |
| --- | --- | --- |
| P0 基线与完整盘点 | Actions 锁定源码/递归依赖，完整 inventory、责任规则、当前 owner 审计、外部/平台/许可证/预算矩阵 | 完整源集合经 G01 接受；G02 structural mode 接受诚实 unread/unclassified；下一实施单元满足 §11.1，不能把仅盘点作为项目完成 |
| P1 统一合同与基础能力 | 既有 Bot 基线、typed cross-language/protocol、storage/identity/config/security、source-oracle 与 CI gates | 不破坏当前架构；最小新 owner 有 ADR；当前责任可独立验证 |
| P2 Agent/会话/工具/调度/记忆 | 实际 Rust 差异实现、生产接线、UI 操作与恢复链 | 每个已迁移责任通过其要求的 Actions 与 production proof |
| P3 插件/模型/渠道/多 Agent/工作组织 | 全部官方插件、skills、external contracts、provider/channel matrix、workboard/team 等 | 全部行真实接入；权限、SDK、外部服务与版本兼容取证 |
| P4 设备/媒体/会议/平台与完整 UX | companion/node/browser/computer、音视频、Canvas、CLI/TUI、安装/运维、统一界面 | 支持矩阵每格有实际 artifact/device/service evidence |
| P5 全量集成与发布 | 现有功能回归、全源 closure、性能/安全/升级/回滚、独立 packaged acceptance | 本 Spec 全部 AC 通过，0 in-scope blocker，才可宣称完整迁移 |

每个模块执行：读完整源 → 拆类型化责任与依赖 → 读 current owner → 定合同/oracle → 按类型实现（业务必须 Rust）→ 真实生产或交付接入 → 适用 UI/服务/平台闭合 → Actions 与必要只读审查 → 独立 review → exact-target receipt 投影台账状态。测试和 ledger 不应单独替代实现工作。

遇到阻塞，记录 `blocker_id`、受影响 requirement/责任、owner、缺什么、解除条件、最后检查、通知 thread 和可继续任务。按 §11.1 推进同阶段或后阶段已经满足单元前置的工作；不能因全局 flags 为 false 而停止所有独立实现，也不能把单元通过升级为全项目通过。不得为等待一个账号/设备停止整个项目，也不得用未通过前置批准后续发布。

需要用户操作时按用户指定 Gmail 渠道通知指定地址；发送前搜索相同未解决请求，避免重复邮件。工作继续时核查回复或真实解除状态，已解除就恢复该任务。联系人地址与 token 不写入公共源码/日志。Gmail 本身不可用则记录该阻塞并在会话告知，继续其他可推进内容。

### 11.1 全局闭包未就绪时的单元准入

`unit_authority_ready` 是带 `unit_id`、责任集合和传递依赖集合的单元结论，不是项目完成 flag。一个单元进入实现必须同时具备：

1. 其源文件及全部影响行为、构建/运行的传递依赖和公共合同已取得并锁定内容身份；Actions 的 scoped G01 证明该依赖闭包完整，没有隐藏在单元边界外的未知依赖。
2. 上述相关文件已完整阅读，责任类型、owner、状态/错误/权限/版本合同、source oracle 和目标接入路径已 reviewed；新 owner 的 ADR 已按规则处理。scoped G02 不能以 unread 或 unclassified 代替这些前置。
3. current target 与相关并行改动已重新核对；实施不会依赖未合并或未取得来源的代码。共享合同仍在变动时锁定 revision，并把变更视为失效条件。
4. 所需 gate/平台/服务矩阵及 blockers 明确。缺真实凭据/设备可以推进已具 authority 的实现和其他验证；相关 `verified/accepted` 仍 blocked，不能以 mock 通过替代。

满足这些条件可进行单元实现、接线和单元验收；其 receipt 必须带 scoped 集合，不能宣称全项目 G01/G02/AC 或发布通过。未知依赖影响某单元时，该单元只继续研究、合同设计与解除阻塞；无依赖单元继续推进。P0 全局退出仍要求完整 inventory/dependency closure，之后也仍需完成全文件语义阅读；阶段状态、单元状态和 §15.1 项目 flags 分别报告。

## 12. GitHub Actions 验证与证据合同

### 12.1 执行环境

本项目的 build、test、lint、generator、schema check、benchmark、fuzz、integration、E2E、visual/a11y、package、install/upgrade/rollback 和验收执行 **只在 GitHub Actions**。本项目不采用仓库其他地方的 `htch-runtime` 例外；不在 bhrum2、用户 Mac/Windows 或 assistant container 运行这些任务。

外部可做只读源码研究、文档/代码编辑、Git/API 操作、review 和证据分析。优先 GitHub 连接器；必要设备编辑按用户要求提交云端并清理本次拥有的临时内容，不能清理他人数据。真实平台/设备验收通过 Actions 管理的受控 runner/device lane，缺配置是 blocked。

### 12.2 必需 gate

下列名称是待实现的验收合同，不表示当前仓库已有同名 workflow。优先扩展既有 source-neutral gate、Rust desktop runtime 和 Desktop Chat Parity；新增工作流先按本 Spec 定义并保证 docs、schema、ledger 和生产路径变化能触发对应 gate。

| Gate | 直接证明的内容 |
| --- | --- |
| OCRM-G01 Authority/closure | upstream/target exact SHA/tree、递归输入与内容身份、无 truncated 漏项、lock/inventory/ledger/dossier provenance 一致、baseline drift；明确 global 或 scoped 集合，不把部分集合通过当全局 closure |
| OCRM-G02 Traceability/coverage | 每文件/类型化责任/capability/requirement/oracle/test/target 双向映射、维度处置证据与状态真实性；structural/final 模式分别按下文判定，禁止虚假升级 |
| OCRM-G03 Architecture/reachability | 唯一 owner/composition、Rust 真实业务路径、真实 producer→owner→consumer，原 OpenClaw core 未进入产物依赖闭包 |
| OCRM-G04 Rust/domain contracts | 各受影响 workspace 的编译、静态检查、unit/property/state-machine、协议/存储/调度/并发/安全合同 |
| OCRM-G05 Source equivalence | pinned source oracle、fixture/record-replay/differential/metamorphic 对照与逐责任差异解释 |
| OCRM-G06 Ecosystem compatibility | Gateway v4/N-1、公开方法/events、plugin SDK/guest、MCP、每个 provider/channel/node、支持版本矩阵与真实注册 |
| OCRM-G07 Product integration | 当前 Fabushi 全能力回归、真实 services/credentials、渠道入站→Agent/tool→投递、UI/CLI/API 实际用户路径 |
| OCRM-G08 Temporal/fault/security | 完整生命周期、乱序/重复、取消、crash/restart、权限/账户/secret、未知结果与恢复 |
| OCRM-G09 UI quality | canonical route/entry、light/dark、语言/RTL、responsive、keyboard/focus/screen-reader、IME、状态连续、timeline visual review |
| OCRM-G10 Resource/soak | 批准数值预算、压力/长运行、CPU/memory/power/storage、quota/cleanup 与回归比较 |
| OCRM-G11 Package/release | 每支持平台实际打包/签名/安装/升级/回滚、binary reachability、SBOM/NOTICE/provenance 与无开发 fallback |
| OCRM-G12 Independent acceptance | 独立阅读 Spec/RTM、审看真实 artifacts/视频/日志、全部 AC 与 release verdict |

G01 可以分别报告根/已声明输入的 identity verdict 与全量枚举/依赖 closure verdict；前者不能推出后者。G02 `structural` mode 用于 P0：验证完整 inventory 集合、schema、引用、责任种类与状态合法性，允许准确列出的 unread/unclassified/not-started，不允许 omitted、断引用或假 accepted。G02 `final` mode 则要求全部路径已读、全部责任分解与映射经审查、所有适用类型合同满足、未完成集合归零。scoped G01/G02 必须显式给出单元和依赖集合；不准复用一个泛化绿色结论抹去阶段与范围差异。

上游 Node/TS 环境只可在隔离的 Actions oracle lane 为对照运行，不能随生产 Fabushi 分发或被生产 fallback 使用。高风险逻辑采用正反例/property/状态机；真实第三方服务用受控账号与清理策略，不把 mock、record-replay 或 SDK sample 当作外部全流程已通过。

LLM 非确定性使用确定的 protocol/state/security/usage invariants 和受控场景，不要求逐字文本相同。上游已知缺陷作为明确差异处理；不为“和上游一样”保留严重漏洞，也不能用任意更改 oracle 掩盖功能缺失。

### 12.3 Evidence envelope

每份 evidence 至少记录：requirement/责任/oracle/test IDs、upstream repo/commit/tree 与 closure digest、`subject_target_commit/tree`、实际 checkout、`subject_inputs_digest`、Spec/contract/fixture/validator hash、workflow/run/attempt/job、runner OS/arch/toolchain/dependency/environment identity、执行动作/命令、开始结束时间、实际测试数/skip/exit/conclusion、artifact ID/name/digest、内部 provenance、reviewer/verdict。仅存放或引用结果的提交另记 `evidence_carrier_revision`，不可把它冒充实际受验 target。

PR 事件需明确实际测试的是 PR head 还是 synthetic merge SHA；不得用错误 SHA 的绿色结果证明 exact-head。阶段内部保留仍有效证据并只重跑失败/失效项；最终验收必须有当前确切 target revision 的聚合证明。artifact 过期或无法验证内容时是缺证据，不是通过。

#### 12.3.1 不产生自引用提交循环的发布流程

1. 把待验代码、Spec/合同、fixture、依赖、workflow/validator 及机器声明提交为 H。P0 中由 Actions 生成的候选 inventory/ledger 等，先从其生成 run 的 artifact 取得再提交为 H；随后检查 H 上这些声明。它们可记录已知 baseline/生成 run，但不要求内嵌 H 的自身 SHA。
2. 对实际 H checkout 执行适用 gates。receipt 从 GitHub/checkout 取得 H 和 tree，并覆盖完整受验输入及 digest。最终全量验收所需 gates 在 H 执行并聚合；release artifact 必须来自该 H。
3. 将 `acceptance-receipt`、effective ledger/RTM、各 flag 与 scope、证据索引和独立 verdict 发布为绑定 H 的不可变 Actions artifact/attestation，以及指向它的 GitHub check。仓库内的声明加该 H 的有效 receipt 是唯一验收投影来源；不再为了把 H 的通过状态或自身 SHA 写回 H 而提交一次。
4. 仓库 `evidence/` 可以在未来提交记录历史 receipt 的 ID/digest；必须同时注明原受验 H 与新的 carrier H2。H2 是新 target，其当前验收尚未成立，必须重新取得自己的适用 gates/聚合证明。任何仅称“docs-only”的提交都不能自动继承 H 的通过；Spec、oracle、fixture、validator 或构建输入变化同样可能使证明失效。

仓库中的静态状态是其标注 revision 的声明或历史摘要，不能压过当前 receipt；没有可验证的 exact-target receipt 时不得把当前 target 的 `verified/accepted` 或全局 flags 投影为 true。最终 receipt 的持久化与内容校验不要求新建代码提交，避免 A→保存 A 的证据成为 B→再保存 B 的证据成为 C 的无限链。该机制是开发验收记录，不创建第二产品运行状态。

工作流绿、artifact 上传成功、配置探针绿、目录计数正确分别只证明自身声明。read-complete、production-wired、full equivalence、packaged-accepted 各自需要对应证明。skipped、not-run、flaky、timeout、cancelled、blocked、缺设备/账号/签名均不计为通过。

## 13. 数据迁移、版本兼容与发布回滚

现有 Fabushi 数据升级先保护账户、Agent、Conversation、Transcript、记忆、附件/产物、工具审批、插件权限、自动化和设备绑定。schema/key/ID 修改必须 versioned、可迁移、可中断恢复；单一 canonical writer，不能创建第二份永远双写的旧新状态。

OpenClaw 导入路径包括配置、workspace instructions、Agent profile、skills/plugins 配置、会话/历史、记忆、自动化、渠道/节点映射以及本基线迁移器承载的相关来源。导入前 preview/dry-run、明确授权和备份；区分 safe configuration 与需重新授权的 token/device/session。保留 provenance、冲突报告、unsupported 暂未迁入项和幂等 identity；不默认复制可执行插件、危险 standing order 或用户凭据。导入无法表达某项功能时保持 open，不能静默丢弃。

本地/无头/团队/远程部署能力和 macOS/Windows/Linux 的支持范围通过平台矩阵固定；companion iOS/Android 等按现有原生项目协同，公共业务/协议优先复用 Rust。每个 cell 写明 owner、repo、artifact、OS/arch/min-version、系统权限、安装/升级策略和验收 lane。不能因目标当前缺某平台发布链就将该能力删掉。

逐责任 cutover 后唯一写入者属于 canonical Fabushi。feature flag 可用于分阶段开启能力，不得有两个生产业务 owner 同时执行副作用。回滚到前一个已验证 Fabushi 版本及兼容 schema/备份路径；不以切回原 OpenClaw Node runtime 作为长期回滚方案。放量条件、回滚触发阈值、数据可逆性和停止发版条件在发布前明确。

## 14. 可观测性、许可证与长期维护

日志/metrics/traces 采用现有 telemetry/audit owner，关联 account-safe operation/turn/job/delivery/device/plugin generation，能解释 accepted/running/terminal、queue/retry、permission denial、outcome-unknown、reconcile 和 resource cleanup。保留独立 execution/delivery 结果；错误不能只剩笼统字符串或在 UI 被吞掉。不得输出消息全文、密钥、授权 headers、设备 secret 或个人数据作为默认 telemetry。

对 telemetry opt-in/out、诊断导出、日志轮转/清理、后台 version check 和离线行为逐项映射。新增模块必须可诊断、可升级、可撤销和可恢复，不能只在开发者机器上工作。

本迁移为 source-informed。OpenClaw 根 MIT 及 `THIRD_PARTY_NOTICES.md` 中 Pi、Hermes、Octicons、SQLite 等来源与实际依赖/资源许可证分别审计；Rust 重写不自动取消适用 attribution/分发义务。保留 source repo/commit/path、改写/复用关系、NOTICE/SBOM 和必要版权。Fabushi 已有 TDRP 等来源的义务继续独立处理，不由 OpenClaw 根 MIT 覆盖。

用户可见产品统一 Fabushi；真实外部服务名、用户内容、合法 notices、协议兼容字段和 migration keys 保留精确用途。品牌清理不可破坏 wire/data 兼容或伪造第三方服务身份。

后续上游变化形成 added/modified/deleted/renamed files、dependencies、contracts、capabilities 的显式 delta；重审受影响 owner、测试和证据，保留删改来源历史。周期性维护须另按用户授权的任务安排，不因本 Spec 自行创建定时服务。

## 15. Acceptance Criteria / Definition of Done

只有下列全部适用 AC 有当前源的正面证据并通过独立验收，才能把 OCRM-001 标为 complete；blocker、未配置或未测项不能以 N/A 消掉。

| ID | 必须满足的结果 |
| --- | --- |
| OCRM-AC01 | upstream exact source/dependency closure 经 Actions 接受；§12.3 receipt 绑定实际 subject target/ref/tree、完整输入及 lock/inventory/ledger/dossiers 的一致 provenance，清楚区分 baseline 与 evidence carrier。 |
| OCRM-AC02 | 全部路径完成类型适配的完整阅读与处置；0 omitted、0 unread/partial、0 未审适用性。 |
| OCRM-AC03 | 全部源 responsibility/capability 双向映射，0 unmapped/orphan/ambiguous owner；非产品处置逐项有证据。 |
| OCRM-AC04 | 所有业务运行时责任在 canonical Fabushi 真实 Rust 实现或明确复用已有 Rust；表现/薄平台桥接及其他责任满足 §6.4.1 类型合同，mixed 文件无业务漏项；无原 OpenClaw core 生产 fallback。 |
| OCRM-AC05 | 每运行时责任 production producer→owner→consumer 可达；表现/桥接有真实业务 owner 与产品入口，非运行时责任有对应构建/测试/文档/资源 consumer 或经审查的具体处置；0 stub/no-op/fake/unregistered 代替要求功能。 |
| OCRM-AC06 | Coordinator/Host/Runner/Electron/renderer/服务/设备边界与唯一 state/composition 成立；所有新 owner 有 ADR。 |
| OCRM-AC07 | 当前 Fabushi 全部既有功能回归通过，未因 OpenClaw 迁移丢失 Bot、Human、插件、自动化或 Computer 能力。 |
| OCRM-AC08 | Agent/模型/harness/上下文/多 Agent/工具与会话生命周期全部源责任等价；terminal 在 switch/reconnect/reload/restart 后保持。 |
| OCRM-AC09 | pinned provider/channel/catalog 的全部功能、认证、权限、恢复和真实投递有逐行 conformance 与服务证据。 |
| OCRM-AC10 | 全部官方插件、skills、custodian skills、SDK/guest/MCP 行为、安装/更新/卸载与权限生命周期通过。 |
| OCRM-AC11 | Gateway v4/N-1、公开/角色/插件方法、schema/errors/events/limits、CLI/TUI/API/companion 兼容矩阵通过。 |
| OCRM-AC12 | Automation 各 trigger、timezone/DST/clock、revision/grant、execution/delivery/unknown、heartbeat migration 与 Lobster 通过。 |
| OCRM-AC13 | Memory/search/wiki/vault、项目/boards/claws/workboard/team/visitor 的数据、权限、删除/恢复与 UI 功能全部闭合；setup/onboarding/doctor/repair flows 按实际配置/维护职责完整接入。 |
| OCRM-AC14 | Browser/Computer/node/device/worker/file/exec/camera/screen/location/notifications 等全部实际命令与 capability/approval 一致。 |
| OCRM-AC15 | 音频/语音/多模态生成理解/会议/通话/Canvas/A2UI/文档与资源生命周期全部闭合。 |
| OCRM-AC16 | 所有功能进入统一 Fabushi route/composition；UI functional/visual/a11y/i18n/IME/responsive/状态连续证据通过。 |
| OCRM-AC17 | 身份/租户/账户/Agent/会话隔离、secret/permission/revocation、prompt/网络/文件/插件安全负例通过。 |
| OCRM-AC18 | cancel/timeout/crash/restart/duplicate/out-of-order/stale/unknown 的适用 fault/recovery 测试通过且无隐式 replay/第二 writer。 |
| OCRM-AC19 | 既有数据升级和 OpenClaw 导入无静默丢失；preview/backup/transaction/conflict/idempotency/recovery/rollback 通过。 |
| OCRM-AC20 | 每个承诺的平台、设备、部署和真实服务矩阵 cell 有 exact-source package/install/operation/upgrade evidence。 |
| OCRM-AC21 | 已批准性能/容量/功耗/磁盘增长/soak 数值预算通过；无无界队列、泄漏或提前资源释放。 |
| OCRM-AC22 | 许可证、来源、NOTICE/SBOM、签名/发布 provenance、Fabushi 品牌与兼容例外完整。 |
| OCRM-AC23 | effective RTM 100% 通过 §12.3 当前 subject target/upstream 的不可变 receipt 绑定 Actions artifact ID/digest 与独立 verdict；不以历史 carrier 提交冒领当前通过；0 critical skipped/flaky/blocked/not-run。 |
| OCRM-AC24 | 0 open in-scope blocker、0 open P0/P1 defect；独立 packaged release acceptance 为 ACCEPT，全部最终能力并集完成。 |

### 15.1 项目完成标志的正面谓词

以下 flags 是全项目集合的结论，本次初始化均为 **false**。它们按 §12.3 的 subject target、source closure 和受验输入投影；单元 receipt 不能把全局 flag 变 true。新 target 需取得自己的证明，不能从旧 summary 自动继承。

| Flag | 适用集合与变为 true 的条件 | 负责 gate / 证据 | 失效或不能升级的条件 |
| --- | --- | --- | --- |
| `baseline_identity_accepted` | pinned 根来源、已声明依赖和当前 target 的身份/内容/provenance 已验证；只表示这些已声明身份可信，不表示枚举已穷尽 | G01 identity verdict 与 exact-target receipt | ref/内容/身份不匹配、缺 digest、证据不可取得；该 flag 本身不能豁免未知依赖 |
| `source_closure_ready` | 全部 tracked 路径、gitlink/LFS、生成输入和必要递归依赖完整枚举并取得；零截断/漏项/未取得依赖；可以仍有诚实 unread | 全局 G01 closure、G02 structural；AC01 的源集合部分 | 未知或缺失来源/依赖、集合漂移、闭包不完整；不要求此时所有语义已读 |
| `responsibility_mapping_complete` | 全部文件完成类型适配阅读、全部责任分解并 reviewed，owner/合同/oracle/依赖/类型处置双向映射完成 | G02 全局语义映射检查；AC02/AC03 | unread/partial/unclassified/unmapped/orphan/owner 歧义或来源/合同变化 |
| `rust_implementation_complete` | 全部业务运行时责任的真实 Rust 实现存在且合同通过，非 Rust 表现/桥接不承载迁入业务；这是实现结论，不代替生产验收 | G03/G04/G05 与 AC04 的 Rust/语言边界部分 | 缺实现、stub、业务留在 TS/原 Node core、相关 source/owner/oracle 变化 |
| `production_wiring_complete` | 全部运行时/表现/平台责任生产接线真实；全部非运行时责任也完成各自类型合同的实际消费/分发或审定处置 | G02/G03、适用 G07/G09/G11；AC05/AC06 | unregistered、不可达消费者、缺产物/资源、未闭合类型合同或接线变化 |
| `ecosystem_compatibility_accepted` | 全部 pinned provider/channel/plugin/skill/SDK/MCP/node 与具体外部合同逐行通过兼容及所需真实服务/设备验收 | G06/G07/G08/G12 与 AC09–AC15 的对应集合 | 缺账号/设备/服务证据、unsupported 未解决、版本/身份/合同变化 |
| `ui_accepted` | 全部产品能力的 canonical UI/CLI/TUI 等适用交互、状态、可访问性和一致性经接受 | G07/G09/G12；AC16 | 缺入口/状态/交互证据、仅开发入口、相关视图/typed contract 变化 |
| `release_accepted` | 其余 flags 均 true；全部类型责任 accepted，全部 AC01–AC24 通过，零 in-scope blocker，当前 H 的完整发布/独立验收 ACCEPT | 全部适用 G01–G12、当前 H 的聚合 receipt、签名/安装/升级/回滚产物 | 任一必需证据失效/过期、未配置/未测、open blocker/严重缺陷或当前 target 已改变 |

P0 通常只能建立前两个 flags 的正面证据；完整语义映射可以在后续逐模块阅读中完成。G02 structural 的绿色不允许把 `responsibility_mapping_complete` 置 true；Rust 实现通过也不允许提升 wiring、ecosystem、UI 或 release。只有全部完成且证据可核验才能对外宣称完整迁移。

## 16. References / provenance

所有 pinned URL 是本次研究输入，不是实时状态或验收结论。后续使用时依第 4 节重新核验。

### 16.1 用户与仓库规范

- 用户 2026-10-22 明确目标：完整源码逐文件/模块理解、Rust 等价实现、融合现有 Fabushi、实现全部功能、先写 Spec。
- [`AGENTS.md`](../../AGENTS.md)、[`spec-first-ai-development.md`](spec-first-ai-development.md)、[`SPEC_TEMPLATE.md`](SPEC_TEMPLATE.md)。
- [`FBCP-001`](fabushi-bot-communication-platform.md)、[`FBCP source of truth`](../../projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md)、[`architecture-map`](../../projects/fabushi-communication-platform/architecture-map.md)。
- [`TDRP-001`](telegram-desktop-rust-equivalence-migration.md)、[`当前 Bot parity spec`](grok-bot-018-runtime-product-parity-recovery.md)、[`设备网络 Spec`](fabushi-same-account-device-network.md)、[`插件生态 Spec`](fabushi-plugins-marketplace-open-ecosystem.md)、[`Mini App 平台 Spec`](offline-first-installable-mini-app-platform.md)。只读取/应用与当前责任有关的合同，历史状态须重核。

### 16.2 OpenClaw 固定来源

- [commit / tree](https://github.com/openclaw/openclaw/commit/e8cde7518c558f1b4e44cb8139a149289b9719ad)。
- [README](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/README.md)、[源码](https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/src)、[packages](https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/packages)。
- [plugin inventory](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/docs/plugins/plugin-inventory.md)、[extensions](https://github.com/openclaw/openclaw/tree/e8cde7518c558f1b4e44cb8139a149289b9719ad/extensions)、[channels](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/docs/channels/index.md)。
- [Gateway architecture](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/docs/concepts/architecture.md)、[protocol package](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/packages/gateway-protocol/README.md)、[versioning / state owner](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/docs/gateway/protocol/versioning.md)。
- [automation / retired surfaces](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/docs/automation/index.md)、[schedule source](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/cron/schedule.ts)、[node runtime manifest](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/node-host/runtime-manifest.ts)。
- [LICENSE](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/LICENSE)、[THIRD_PARTY_NOTICES](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/THIRD_PARTY_NOTICES.md)。

### 16.3 Fabushi 固定来源

- [目标基线](https://github.com/bhrumom/fabushi-desktop/commit/ba9ecce728ef021fee609796c8704d8af6fe4b5a)、[Renderer mount](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/frontend/src/main.tsx)、[Electron composition](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/electron-main/entry.ts)。
- [Coordinator boundary](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/node-agent-coordinator/src/lib.rs)、[Host/Runner boundary](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/src/lib.rs)、[production extensions](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/src/host_production_extensions.rs)。
- [channel port](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/src/connectors/channel_delivery.rs)、[Host app composition](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/source/host/app/src/main.rs)、[packaging scripts](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/desktop/package.json)。
- [Rust Actions](https://github.com/bhrumom/fabushi-desktop/blob/ba9ecce728ef021fee609796c8704d8af6fe4b5a/.github/workflows/rust-desktop-runtime.yml) 是已有验证入口，不是本 Spec 的迁移通过证明。

## 17. Spec compliance 与本次交付记录

| 对象 | 状态 | 说明 |
| --- | --- | --- |
| 需求/范围/架构/接口/实施/验收规范 | 本次已定义，待后续按规范实施 | 对应本文件、module map 与 P0 task |
| discovery baseline 与 owner 候选 | 已有只读来源依据 | 第 4/16 节；没有递归全量已读或产品验收结论 |
| OCRM-R01 至 R24 实现合规 | pending / 尚未验收 | 不因写好 Spec 自动 passed |
| OCRM-AC01 至 AC24 | pending / 尚未验收 | 后续逐条提供 exact-head Actions 证据与独立 verdict |
| 所有完成 flags | false | 递归总量与全量进度尚未建立，不填虚构百分比 |

执行阶段每次更新真实责任与 evidence，最终每个适用 AC 只能 `passed`（有证据）或保持 `blocked`；具体非产品事项的 `not-applicable` 必须有源证据，不能豁免最终功能。下一项工作为 [`P0-source-baseline-and-owner-audit.md`](../../projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md)。
