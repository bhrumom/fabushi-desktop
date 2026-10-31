# OCRM P0 — 全量 source baseline 与 existing-owner 审计

Status: in-progress / global authority pending
Task ID: OCRM-P0  
Spec: [OCRM-001](../../../../docs/specs/openclaw-rust-equivalence-migration.md)  
Project: [Source of Truth](../../SOURCE_OF_TRUTH.md)  
Last updated: 2026-10-31（Asia/Shanghai）

## 1. 本任务要交付什么

建立可以驱动全部后续 Rust 等价实现的真实 source authority、无遗漏 inventory、逐责任 ledger/RTM、当前 owner 解析和首批可实施合同。任务结束后必须能回答任一上游文件“读到哪里、承担什么、由谁实现、还缺什么、如何验收”，不能只产生目录清单和 migration proposal。

本任务不声称一轮已完成所有文件语义深读。P0 必须完成全量集合与身份校验，并为每条路径建立适用性与 disposition；对下一实施单元完成逐文件阅读、依赖/owner/oracle 合同。承载行为、高风险、公开合同、生产 producer/owner/consumer、权限安全、状态迁移、官方插件/渠道/平台和关键上游 oracle 的路径必须深读；普通文档、示例、fixture、重复或研发用途项可以用可复核成组证据处置。未知行为保持未知并升级深读，未深读路径真实保持 unread。全部源码语义阅读、Rust 实现与验收由后续任务持续闭合，最终标准不降低。

完整枚举/取得来源由 `source_closure_ready` 表达；0 未分流路径和 0 未评估适用责任由 OCRM-AC02/G02 final 表达；适用责任理解、映射和 disposition closure 由 `responsibility_mapping_complete` 表达，正面谓词见主 Spec §15.1。`unread` 是深读进度维度，不是把普通非产品路径强行全文阅读的全局门槛。P0 全局未退出时，满足 §11.1 的单元及其全部相关传递依赖仍可获得 scoped authority 并推进 Rust 实现；不把该单元结论升级为全项目通过。

2026-10-31 governance amendment: P0 structural may preserve honest
unread/unclassified progress. P0 final and AC02 close disposition and
applicability sets, not a fabricated zero-unread count; this does not promote
source closure or any project completion flag.

## 2. 前置读取与并行工作保护

1. 读取 live main、实际工作 PR/head/ref/tree、根 AGENTS、OCRM Spec/SOT/module map、当前相关 owner 源码和领域合同。
2. 读取 live `openclaw/openclaw` default branch/ref/commit/tree。Spec discovery `e8cde7518c558f1b4e44cb8139a149289b9719ad` 仅作起点；若变化，先记录 delta 与明确新 baseline，不能静默替换。
3. 查现有 OpenClaw 任务/分支/lock，避免重复项目或覆盖另一执行者；TDRP PR #154 与 #165/#166/#167/#168 等并行工作独立保留。
4. 目标状态只按实际合并源码判断，旧 FBCP inventory 和尚未合并 PR 的代码均需标明来源，不能冒领 main 能力。

此前 live ref 只读核对曾解析到 OpenClaw `main`=`b7a537eb6833ea16960c227d5fc223fcd08ccef6`，root tree=`804b4f9c70d2a24a2575cfe68bc682f6c6c98574`；递归 tree=`truncated=false`、63,521 entries（61,281 blobs、2,240 directories）。这是历史 discovery-only rebaseline 输入，不是 Actions source authority；该 identity 与更早 candidate 的 inventory/依赖闭包不得直接复用。
随后只读核对发现 OpenClaw `main` 再次漂移到 `68fbe4ac429cdac7a54541df40367d66defd0161`、root tree=`a3c122b99571c0c33ff6a02ca4c4e32f05f9433d`（committer=`2026-10-30T09:42:45Z`）。该 exact subject 仍是 discovery-only；compare 计数未刷新，旧 candidate 的 inventory、依赖闭包和 Actions provenance 不得复用。

所有测试/构建/生成器/schema/可执行盘点验收只在 GitHub Actions。外部只做只读调查、文档/代码编辑和 Git/API 操作。

## 3. 任务分解与交付合同

| ID | 工作 | 可复核交付 |
| --- | --- | --- |
| OCRM-P0-01 | 精确来源/目标锁定 | repo/ref/commit/tree、目标实际 checkout、source archive/hash、上游变化 delta、工作分支 lease |
| OCRM-P0-02 | 递归 enumerate + dependency closure | 全部 tracked blobs/symlinks/gitlinks/LFS/root/hidden/generated、包 integrity/patch/外部组件关系、truncation 完整性证明 |
| OCRM-P0-03 | 定义并验证 file/责任/RTM schema | 主 Spec 第 6 节字段、responsibility_kind/类型合同、独立状态、维度处置、集合守恒、禁止冒领状态的正反例 |
| OCRM-P0-04 | 创建完整 file inventory | 每个源路径一条记录，未读保留 unread；root/非运行时职责不得省略；未知数量不填 0 |
| OCRM-P0-05 | 逐模块依赖和当前 owner resolution | OCRM-M01..M32 扩展为真实 file/symbol→behavior→owner 图；新发现能力自动加入，不能按分组遗漏 |
| OCRM-P0-06 | 生态/平台/服务/许可证矩阵 | 全部插件/skills/provider/channel/node/CLI/部署/外部包，来源、版本、owner、权限、配置、验收 lane 与 blocker |
| OCRM-P0-07 | 首批模块全文阅读和 dossier | 完整 read ranges、责任/状态机/错误边界、source oracle、target symbol、production call path、差异与待实现项 |
| OCRM-P0-08 | GitHub Actions authority/coverage gate | G01 identity/closure 和 G02 structural/final 可运行且 fail-closed；final AC02 使用 `validate_p0.py <evidence-dir> --final-ac02`，global/scoped 集合明确；真实 run/attempt/job/日志/artifact ID/digest 与实际 subject target 一致 |
| OCRM-P0-09 | 既有能力回归与首批预算定义 | current Bot/FBCP/Computer/Plugins/Automation 的必保留行为、受影响 gate；性能 workload/数值预算和测量计划 |
| OCRM-P0-10 | 形成首批 Rust 实施任务并交接 | 每任务有 responsibility ID、被选 owner、差异实现、UI/服务/平台 consumer、测试和退出条件；不以 P0 代替项目完成 |

## 4. 机器产物布局

以下为本任务要建立的产物，当前 Spec 发布不宣称它们已经存在：

- `projects/openclaw-rust/upstream.lock.json`：唯一 source/dependency identity。
- `projects/openclaw-rust/inventory/index.json` 与可复核分片：完整逐路径集合、类型和阅读状态。
- `projects/openclaw-rust/parity-ledger.json`：逐责任 source/owner/implementation/wiring/acceptance。
- `projects/openclaw-rust/requirements-traceability-matrix.json`：requirement/AC/oracle/test/evidence/verdict。
- `projects/openclaw-rust/dossiers/`：模块行为、完整阅读、owner 选择和兼容差异。
- `projects/openclaw-rust/decisions/`：确需新 owner、语言/协议/平台适配与变更的 ADR。
- `projects/openclaw-rust/evidence/`：来源和 Actions artifacts 索引；大产物留在受控证据存储，不把密钥、账号数据或构建目录提交仓库。
- 本项目当前任务/状态记录：保存 blockers、首个 strict responsibility 与可并行单元，复用仓库普通 task 生命周期。

复用现有 source-neutral authority/review/evidence 组件；必要扩展应有清晰任务，不复制一套 TDRP 源码扫描器或把 OpenClaw 条目塞入 Telegram ledger。

按主 Spec §12.3 先在 Actions 生成候选机器文件，再将它们与 Spec/validator 等受验输入提交为 H、云端 read-back，随后对 H 执行 authority/coverage 验证。最终 `acceptance-receipt`、effective ledger/RTM、flags、artifact ID/digest 和 reviewer verdict 作为绑定 H 的不可变 Actions artifact/attestation 发布；不要求把 H 的结果写进 H 自身。未来证据索引提交须标明原 subject H 与新 carrier H2，并重新为 H2 取得适用证明，不自动继承历史通过。

## 5. 首批阅读与接线顺序

基于依赖先处理 package/schema/entrypoint/config/lifecycle，再进入当前 owner 对照与真实能力，而不是长期停留在无产品逻辑的文档计数：

1. `package.json`/workspaces/locks、根 entrypoints、`packages/gateway-protocol`/`gateway-client`、实际 dispatch/roles/permissions、`src/config`/`state`/`storage`/`secrets`。
2. `src/agents`、`packages/agent-core`/`llm-core`、session/transcript/context、provider/harness/tool/approval 与 Fabushi Runner/Inference/Session/Transcript 边界。
3. `src/plugins`、`src/plugin-sdk`、`packages/plugin-sdk`/`plugin-package-contract`、skills、MCP/registry，先形成版本/权限/生命周期 conformance。
4. `src/cron`/hooks/worker、memory、channels/routing 与当前 Automations/Memory/connectors 的逐责任差异；尤其验证 HostChannelDelivery default unregistered 的真实接线需求。
5. 继续所有剩余模块、全部插件/skills、UI、平台/设备/服务、quality/build/release。按 file/symbol 稳定顺序与 dependency DAG 保留未完成 frontier。

这是实施依赖顺序，不是缩小源范围或按优先级豁免低频功能。某一来源/渠道/平台阻塞时，按主 Spec §11.1 的 scoped G01/G02、完整阅读和合同条件确定其他单元能否进入实现；依赖未满足的事项可继续只读研究/合同设计。缺账号/设备可以继续有 authority 的实现和其他验证，相关真实验收仍 blocked，不能虚构通过。

## 6. 本任务退出标准

| ID | 退出标准 |
| --- | --- |
| OCRM-P0-AC01 | 本任务 receipt 的 upstream、subject target/ref/tree、实际 checkout 与所有机器输入 provenance 一致；baseline/subject/carrier 明确区分，上游 drift 有显式处置。 |
| OCRM-P0-AC02 | 全量路径集合与必要依赖闭包经过 Actions 校验，无静默截断/过滤；若外部闭包未获证据，P0 整体仍 blocked，已完成子任务可保留。 |
| OCRM-P0-AC03 | 每路径已入 inventory 且身份、类型、适用性和 disposition 状态诚实；未读项不冒领 read-complete，未知行为升级深读，全部未分流/未评估集合明确。 |
| OCRM-P0-AC04 | 下一实施单元满足主 Spec §11.1：相关文件/传递依赖已取得并完整阅读，scoped authority、类型化责任/owner/oracle 与合同经审查；不宣称其他模块同步完成。 |
| OCRM-P0-AC05 | G02 structural 接受诚实 unread/unclassified/not-started，拒绝漏项、断引用、错误类型/维度处置、身份漂移、伪 verified/accepted；仅已声明实施或验收状态要求对应真实 target/evidence。final 模式要求 0 未分流、0 未评估适用责任、0 适用 orphan/unmapped；不把普通非产品路径的 unread 伪装成深读或把它单独作为 AC02 硬门槛。 |
| OCRM-P0-AC06 | 外部服务、账号、设备、包、许可证和平台矩阵有逐项 owner/证据/解除条件。 |
| OCRM-P0-AC07 | 针对已提交 H 的 Actions 完整日志与不可变 receipt/artifact ID/digest 被读取，独立 reviewer 检查 P0 结论和首批任务；结论作为 H 的 receipt 发布，不以新增 carrier 提交冒领 H2 通过。 |
| OCRM-P0-AC08 | 受验输入 H 已云端原子提交/read-back，随后完成 H 的验证及 receipt 发布；没有变更无关实现分支或他人文件；后续满足准入的任务可继续真实 Rust 实现。 |

P0-AC 全通过也只完成基线任务；可按主 Spec §15.1 为 baseline identity/source closure 提供正面证据。未分流/未评估适用责任或适用 orphan/unmapped 使 `responsibility_mapping_complete` 保持 false；诚实存在的非适用 unread 不单独改变该 flag。`rust_implementation_complete`、`production_wiring_complete`、`ecosystem_compatibility_accepted`、`ui_accepted`、`release_accepted` 不因 P0 通过而改变。若 P0-AC02 受阻，项目级 closure 仍 false，满足 scoped authority 的单元按 §11.1 推进，不改变这一全局结论。

## 7. 阻塞与支持请求

使用稳定 blocker ID，记录影响范围、已知事实、根因/未知点、owner、解除条件、最后检查、既有通知 thread 和可继续任务。需要用户操作时按用户指定 Gmail 地址通知；先检索是否已发送相同未解决请求，避免重复。地址、token 和账号敏感详情不进入公开仓库。

每次继续执行时检查回复和真实环境变化；已解决即恢复，未解决继续下一可行工作。缺运行器、凭据、设备、签名不是通过，也不是产品功能 N/A。失败按实际日志分清实现缺陷、测试缺陷或环境不可达，不能无限原样重跑。

## 8. 本次状态

P0 仍未退出：此前只读 live candidate 曾推进到 OpenClaw
`68fbe4ac429cdac7a54541df40367d66defd0161` / tree
`a3c122b99571c0c33ff6a02ca4c4e32f05f9433d`；该 identity 仅保留为历史
discovery evidence。此前 candidate 曾以 `upstream.lock.json` 的
`10d93efc…/d33ed7c3…` 为准，但现已被后续 live candidate supersede；该段仅作
历史记录。当前 `002f25a…/553796f…` 尚未取得针对 exact subject 的 GitHub Actions
inventory、依赖闭包和 evidence envelope。既有 inventory/shards 仍绑定上一轮
候选，不能编辑性重标为当前来源；对应 drift decision 见
`decisions/2026-10-31-upstream-live-drift.md`。

2026-10-31T04:20:22Z 的只读重核对确认该 candidate 的递归 tree 未截断，共
63,880 entries（61,639 blobs、2,241 directories）；根依赖/许可证输入与上一
candidate 相同。该事实仍不构成 Actions authority、source closure 或任何
completion flag 的正面证据。

已完成的工作包括 schema/门槛修订、live candidate 记录、若干 scoped
node-host owner 合同与 Rust 实现切片；这些不构成 P0-AC01–AC08、全项目
source closure 或任何 global flag 的正面证据。OCRM Actions、本机 OCRM
验证、push 与 PR 更新仍按开发期冻结要求未执行；本文件的退出标准保持
pending，允许满足主 Spec §11.1 的 scoped 单元继续推进。

本轮新增 `dossiers/upstream-node-host-candidate-20261030.md`，仅记录历史候选
`5f6f15fed0d6ba618cf781c9686a8b93f499564b` 的 NodeHost package/lock/workspace
依赖前沿与待验证闭包条件；它不改变 `upstream.lock.json` 的 discovery-only
状态，也不替代 exact-subject Actions inventory、LFS、integrity、license/
NOTICE/SBOM 或 omitted/orphan negative evidence。

随后对当前 discovery-only candidate
`68fbe4ac429cdac7a54541df40367d66defd0161` / tree
`a3c122b99571c0c33ff6a02ca4c4e32f05f9433d` 完成 NodeHost 关键来源的精确
blob 读取与责任增量记录（见 `dossiers/node-host-scoped.md` §192）。该读取
明确补充了当前 command catalog、manifest availability、optional publication
与 invoke responder 的 mapping，但不提升 source authority、不重标旧
inventory/closure，也不改变任何 global flag。

随后 live main 漂移至 `5310270e46cf020ab1e37204075970f3b2679812` /
`acf6bee252bf79ab46e7a4c36ce4cc23e448c885`；此前 exact blob 读取与 SecretInput
结论仅保留为历史 discovery evidence，不能直接作为新 subject 的 source
closure 或 implementation authority。

本轮又只读核对了 Fabushi 当前 secret/redaction owner：
`openclaw_secret_redactor` 仅提供纯脱敏函数，没有 redaction registry；
`HostSecretsExtension` 仅负责 Box secret application/persistence，不是通用
`SecretInput` materializer。故 NodeHost Cloudflare Access 仍需独立的注入式
credential owner contract（成对、origin-scoped opaque refs；connection-time
materialize；canonical redaction registration；不持久化 plaintext、不读任意
环境变量、不进入 session/RPC），在该 owner 与真实 caller 出现前保持
transport fail-closed；这不改变 P0 或任何 global flag。

2026-10-30T10:03:53Z 的独立 `git ls-remote` 只读核对仍返回同一
`5310270e46cf020ab1e37204075970f3b2679812`；对应 tree 仍为
`acf6bee252bf79ab46e7a4c36ce4cc23e448c885`。该核对只刷新 identity freshness，
不替代 exact-subject Actions inventory、依赖闭包或验收 receipt。

2026-10-31 的 live-main 只读核对又发现上游推进到
`2c157ed06dff5032da1093cbb6745efeff2b6f33` / tree
`3fb582f6a1299daa74446ef001cb7cb4fd3c2270`；递归 tree 未截断，计数为
63,740 entries、61,499 blobs、2,241 directories。七个已映射 NodeHost/command
blob 身份保持不变，但该新 subject 仍仅为 discovery-only；P0 不得复用旧
inventory/依赖闭包或将其提升为 authority。

本段 exact subject 是本轮治理文件的唯一当前 discovery reference；较早段落
保留为历史 identity 记录，不代表当前 source authority。当前 lock 状态仍为
`p0-source-authority-rebaseline-required`，因此不得从本段 discovery identity
提升 `baseline_identity_accepted` 或 `source_closure_ready`。

2026-10-31T05:51:22Z 的最新只读核对将当前 discovery reference 更新为
OpenClaw `bcf23b2ea0eee92c8c497c6b1c4d486588e4d5ab` / tree
`5d9b96f3208eb2daf4c566fc60865a1850e4c77f`；递归 tree 未截断，共 63,946
entries（61,705 blobs、2,241 directories）。它已同步写入
`upstream.lock.json` 的 `live_candidate`，仍不构成 Actions authority、依赖
闭包或任何 completion flag 的正面证据；以上较早 current-candidate 段落仅作
历史记录保留。

随后在 2026-10-31T02:34:32Z 重新核对到 OpenClaw
`298546d88934bf05bc7e208d13bf6ce2c538e3f2` / tree
`120f973ab7da856b4bc1c545c437dd94d35ef9eb`。递归 tree 未截断，计数为
63,847 entries、61,606 blobs、2,241 directories；根输入 blob 身份与 lock
一致。该 subject 仍 discovery-only，旧 inventory、依赖闭包和 Actions
provenance 不得复用；P0-AC02、source closure 与所有 global flags 继续保持
未通过/未闭合。

2026-10-31T07:55:14Z 的最新只读核对将当前 discovery reference 更新为
OpenClaw `77b08d1345a8e5734ada19dc95e77f4c5135bd0a` / tree
`a52b727858379d14b1b1ed4bde71837326c127f7`；递归 tree 未截断，共 63,993
entries（61,752 blobs、2,241 directories）。该 subject 已同步写入
`upstream.lock.json` 的 `live_candidate`，仍不构成 Actions authority、依赖闭包
或任何 completion flag 的正面证据；此前 candidate 段落仅作历史记录保留。
