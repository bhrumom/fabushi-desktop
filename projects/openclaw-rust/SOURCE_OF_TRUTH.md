# OpenClaw 全源码 Rust 迁移 — Source of Truth

Status: active  
Project ID: OCRM-001  
Revision: 2
Last updated: 2026-10-30（Asia/Shanghai）
Target: `bhrumom/fabushi-desktop` current canonical main  
Implementation/acceptance: **未完成**

## 1. 唯一目标

逐文件、逐模块、逐责任理解 `openclaw/openclaw` 的完整源码与必要依赖，将全部产品功能使用 Rust 等价实现到现有 Fabushi 架构，保留现有 Fabushi Bot 的全部能力，交付一个完整、统一、可发布、可维护的 Fabushi Desktop。

本项目必须完成 source completeness、全部产品责任实现、真实生产接线、UI/UX、服务/插件/渠道/模型/节点生态、数据迁移/平台交付以及独立验收。不能以源码盘点、目录映射、概念架构、同名 API、原 OpenClaw wrapper、UI demo、少数模块或单条 end-to-end 路径代替全量完成。

## 2. 权威文件与适用顺序

1. 最新明确用户要求与根 [`AGENTS.md`](../../AGENTS.md)。
2. 主规范 [`docs/specs/openclaw-rust-equivalence-migration.md`](../../docs/specs/openclaw-rust-equivalence-migration.md)（OCRM-001）；本文为入口，不重复定义另一套验收。
3. 当前 canonical Fabushi 真实源码/接线，以及相关 Bot、FBCP、TDRP、插件、Mini App、设备网络领域合同。
4. 本项目 [`module-map.md`](module-map.md)、当前 task、ADR、lock/inventory/ledger/dossier、RTM 和 exact-head evidence。
5. 历史研究与旧版本快照。它们必须重核后才能用于当前结论。

主规范是全部行为、实施和验收要求的单一出处。module map 是候选落点与范围导航，不是批准新架构、实现状态或验收结果。当前工作任务为 [`P0-source-baseline-and-owner-audit.md`](management/tasks/P0-source-baseline-and-owner-audit.md)。

## 3. 不可弱化的约束

- 全源码范围逐文件登记与完整检查，逐函数/状态机/副作用/合同分解责任；UI、tests、scripts、docs、资源、隐藏配置、generated、external package、实验/source-only、companion 文件均不能整类排除。
- 所有迁入业务/策略/协议/调度/状态/持久化/执行控制由 Rust 承担；React/Electron 保留现有产品表现与薄桥接。UI 源文件中的业务也须迁移。
- 先读当前 existing owners；只有无合理承接者时通过 rejected-owner analysis 与 ADR 新增最小 source-neutral owner。
- Coordinator、Host、Runner、box-exec、Electron、renderer、服务端与设备边界保留；无第二 Agent loop、Conversation/Transcript store、scheduler、Marketplace、权限真相或 OpenClaw 运行时。
- 现有 Fabushi Bot 全部功能保留；Human/Agent、资源、插件、工具、自动化、Computer 和通信能力共用 canonical composition。
- OpenClaw 可选外部渠道和 Gateway/SDK/node 兼容由现有 owner 下的 Rust adapters 承接；Fabushi 原生身份/网络/会话不因此依附外部服务。
- `enumerated`、`applicability-assessed`、`dispositioned`、`read`、`mapped`、`implemented`、`wired`、`verified`、`accepted` 独立，按主 Spec §6.4.1 的责任类型完成合同；业务必须 Rust/生产接线，非运行时职责保留实际消费者和处置证据。每条路径必须完成身份、适用性和处置；承载行为、高风险、公开合同、生产 producer/owner/consumer、权限安全、状态迁移、官方插件/渠道/平台和关键上游 oracle 的路径必须深读。普通文档、示例、fixture、重复或研发用途项可用可复核成组证据处置，但 `unread` 不能冒充深读，未知行为必须升级。目录、文件、crate、manifest、mock、旧 CI 不证明全部功能。
- 所有可执行验证只在 GitHub Actions，包括 generator/schema/lint/build/test/E2E/visual/performance/package/device acceptance；不沿用其他项目的 `htch-runtime` 例外。
- 凭据、签名、真实设备/渠道/服务未配置是 blocked；保留解除条件并推进无依赖任务，不改变最终目标。
- 许可证、版权、NOTICE、SBOM 与来源保留；产品统一 Fabushi，外部服务名称和必要兼容字段按合同保留。

## 4. Discovery 快照，不能当作完成状态

| 输入 | 身份 |
| --- | --- |
| OpenClaw main | `e8cde7518c558f1b4e44cb8139a149289b9719ad`（历史 discovery；非当前 source authority） |
| OpenClaw tree | `25c8ee611b72762ae1a0170124fa1fee9e519704`（历史 discovery；非当前 source authority） |
| Fabushi 编辑前 main | `ba9ecce728ef021fee609796c8704d8af6fe4b5a` |
| Fabushi 编辑前 tree | `29e6837e6ea9098d64c5de3e0952b3960abbfc86` |

这些是 2026-10-22 spec 研究输入。文档发布后目标 HEAD 已改变，下一轮必须重新读取 live refs；上游变化需明确 rebaseline。当前未完成全量递归 file inventory/semantic read，不能填造总文件数或覆盖率。

本次已读取主目录、关键文档与部分核心源码。观察到 src 77、extensions 176、skills 50、custodian-skills 4、packages 24、apps 8、crates 2 个直接子目录。plugin inventory 列 175 个插件（73 core / 99 official external / 3 source-only），extensions 另含 test-support。这些数字不是逐文件/功能验收进度。

目标 main 有大量可复用 owner，但外部 channel default 仍是 unregistered seam；Memory、MCP、Human/Call、Browser 源码存在不等于 OpenClaw parity。未合并的 PR #165/#166/#167/#168 仅作协调依赖；正在进行的 TDRP PR #154 及其 evidence 不被本项目改写。

2026-10-30 live 只读核对：Fabushi `origin/main`=`cfef45568293a22deb32a6cdef17916f3ec19648`；当前专用工作分支起点=`074d17dcd8508ef6ca45bc5932553e7429cf62f9`；PR #169/#180/#181 heads 分别为 `b5db35fa272c83833685d70a730b5f8285fa1176`、`2225400ea73392523586a110301edeb87c986001`、`ab4eaa0560a5b767f46d0c91e36eb6e3cfd4344a`，均未合并。OpenClaw live `main`=`87facf07d6b4907846c4a8e4c7ae551f85571420`，tree=`f72fbc794bbb23a04bc5afea875d4877048015c9`；递归 tree API 未截断且有 63,459 entries。该段是较早的 live discovery，已被下方 `c49e073…/fcd6e0…` 观察 supersede，不是当前 source authority。现有 `upstream.lock.json` 已明确将上一轮 `ad8d12f…/04ccce…` 标为 `accepted_for_this_implementation_round=false`，并保存 live candidate；必须在 P0 重新生成 inventory、依赖闭包并取得 Actions provenance 后才可提升为本轮 authority。上述核对不是 CI 或产品验收证据。
随后只读复核发现 OpenClaw `main` 已继续漂移到 commit=`c49e07385f8d17601efbed7746381213d5326ba9`、tree=`fcd6e0b9b42da1e3d39ca89c69ac779294cc3bfb`（`2026-10-30T05:56:33Z`）；递归 tree API 仍未截断，共 63,469 entries（61,231 blobs、2,238 directories）。根 manifest blob identities 与前次一致，`.gitmodules` 仍不存在。这只是最新 live candidate discovery，不能替代 Actions rebaseline，也不能复用旧 inventory 或提升任何 authority/完成 flag。
随后只读获取到更新的 OpenClaw `main` candidate：commit=`d67aa608643f6583a6c6102c522043794aafe064`、root tree=`1999145af25baef58520f40daf89e0be3b39988a0`；限定 `src/node-host/` 与 `crates/openclaw-node-host/` 递归 listing 为 227 条路径。该 candidate 同样只是 discovery，不替代 lock；必须完成同一 exact subject 的递归 inventory、依赖闭包和 Actions provenance 后才能提升 authority。期间不复用 c49 或旧 lock 的未经重核结论。
随后再次只读核对 OpenClaw `main` 漂移到 commit=`c32556b0cea7f63b33db0ac393affd71ecb0015b`、root tree=`e08d361d83ca9cae55aa93d49462d991d0a39efc`；限定 Node Host 范围仍为 227 条路径。该结果仍是 discovery-only，不能替代 lock 或复用 d67/c49/旧 inventory；必须对 c325 exact subject 重新取得完整 inventory、依赖闭包和 Actions provenance。
随后只读核对发现 OpenClaw `main` 再次漂移到 commit=`89734994c6c9d3572e2927be0cdae8d7da1998ca`、root tree=`bcb3914c8418e2f92927dd3e60c635fc4345a858`；递归 tree API 返回 `truncated=false`，共 63,521 条 tree entry（61,281 blobs、2,240 directories）。根 `package.json`、`pnpm-lock.yaml`、`pnpm-workspace.yaml` blob identities 与上一 candidate 相同，但 tree 仍已漂移。该 candidate 仍为 discovery-only；`upstream.lock.json` 已更新其 live-candidate 身份，但未提升 accepted/source-closure-ready，不能复用 5f6 或更早 candidate 的 inventory、依赖闭包或 Actions provenance。

随后只读核对发现 OpenClaw `main` 已漂移到 commit=`b7a537eb6833ea16960c227d5fc223fcd08ccef6`、root tree=`804b4f9c70d2a24a2575cfe68bc682f6c6c98574`（committer=`2026-10-30T09:39:57Z`）；递归 tree API 仍返回 `truncated=false`，共 63,521 条 tree entry（61,281 blobs、2,240 directories）。根 `package.json`、`pnpm-lock.yaml`、`pnpm-workspace.yaml`、`.gitattributes`、`LICENSE` 与 `THIRD_PARTY_NOTICES.md` blob identities 未变。该 candidate 仍为 discovery-only；在同一 exact subject 完成 Actions inventory、依赖闭包、LFS/完整性和独立审核前，不提升 lock authority，也不复用旧 candidate 的 closure 结论。
随后只读核对发现 OpenClaw `main` 再次漂移到 commit=`68fbe4ac429cdac7a54541df40367d66defd0161`、root tree=`a3c122b99571c0c33ff6a02ca4c4e32f05f9433d`（committer=`2026-10-30T09:42:45Z`）。这是新的 discovery-only candidate；未刷新 compare 计数，不复用 `b7a537eb` 或更早 candidate 的 inventory、依赖闭包、源语义结论或 Actions provenance。必须针对该 exact subject 重新取得完整 inventory、依赖闭包、LFS/完整性与独立审核证据后，才能提升任何 authority/完成 flag。
随后只读核对发现 OpenClaw `main` 再次漂移到 commit=`5310270e46cf020ab1e37204075970f3b2679812`、root tree=`acf6bee252bf79ab46e7a4c36ce4cc23e448c885`（committer=`2026-10-30T09:50:47Z`）。该 candidate 仍为 discovery-only；compare 计数未刷新，不复用 `68fbe4a` 或更早 candidate 的 inventory、依赖闭包、源语义结论或 Actions provenance。必须针对该 exact subject 重新取得完整 inventory、依赖闭包、LFS/完整性与独立审核证据后，才能提升任何 authority/完成 flag。

2026-10-31 的独立 live-main 核对又解析到 commit=`2c157ed06dff5032da1093cbb6745efeff2b6f33`、root tree=`3fb582f6a1299daa74446ef001cb7cb4fd3c2270`（committer=`2026-10-30T22:39:38Z`）。该 exact subject 的递归 tree 未截断，共 63,740 entries（61,499 blobs、2,241 directories）；root 输入和 live candidate 已同步写入 `upstream.lock.json`。七个已建立 NodeHost/command blob 身份保持不变，但这仍只是 discovery-only，不能提升 source authority、复用旧 closure 或改变任何 global flag。

本段是当前 Source of Truth 的唯一有效 OpenClaw discovery identity；更早
candidate 段落仅保留历史证据。当前 `upstream.lock.json` 状态仍要求 P0
针对该 exact subject 重新生成递归 inventory、依赖闭包、LFS/完整性与
Actions evidence envelope；在此之前 `baseline_identity_accepted`、
`source_closure_ready` 及后续完成 flags 均保持 false。

## 5. 机器记录的初始化与终态

P0 在 Actions 建立 lock、inventory、parity ledger、RTM、dossiers 和 evidence 索引；本次不提交伪造的完整台账或空清单来充当 baseline。

各 flag 的适用集合、true 谓词、负责 gate 和失效条件以主 Spec §15.1 为唯一规范。`source_closure_ready` 表示完整取得并枚举来源/依赖，不表示已阅读全文；AC02 的全量闭合是“0 未分流、0 未评估适用责任”，而不是把 58,750 个路径强行变成全文深读。全部适用产品责任理解、owner/合同/oracle 双向映射和非产品 disposition 证据归 `responsibility_mapping_complete`。P0 G02 structural 可以保留诚实 unread/unclassified；最终 G02 必须消除未分流/未评估适用责任，但仍要如实报告尚未深读的非适用成组路径。

主 Spec §12.3 区分受验 target 与证据载体：仓库保留 source declarations、已知 baseline 和历史 receipt 引用，当前有效状态由同一 subject target 的不可变 Actions receipt 投影。最终 receipt 发布到 artifact/attestation/check，不为回写自身 SHA 再制造一个提交；后续提交是新 target，旧绿色结果不能自动成为当前验收。

| 标志 | 当前值 |
| --- | --- |
| baseline_identity_accepted | false |
| source_closure_ready | false |
| responsibility_mapping_complete | false |
| rust_implementation_complete | false |
| production_wiring_complete | false |
| ecosystem_compatibility_accepted | false |
| ui_accepted | false |
| release_accepted | false |
| recursive_source_total | 尚未建立，unknown，不是 0 |
| unread / incomplete | 继续如实记录尚未深读/证据不完整的路径；不再作为全量 AC02 的单独硬门槛 |

只有 OCRM-AC01 至 OCRM-AC24 全部取得当前 upstream/target 身份的正面证据并独立接受，才允许宣称完整迁移。当前文档交付不改变任何其他项目的完成标志。

## 6. 执行与交接

按主 Spec 的 P0→P5 依赖推进，阶段不是范围豁免。全局闭包受阻时，满足主 Spec §11.1 的单元可在自身及传递依赖 authority、完整阅读和合同经审查后进入实现/单元验收；单元通过不改变全项目 flags，缺服务/设备仍阻塞对应验收。每个任务记录 source responsibility、existing owner、具体差异、实现与验收证据；进展以真实生产闭合为中心。阻塞时检查已有支持邮件、记录必要动作和解除条件，继续后续可执行项；恢复时重查事实和 lease。

后续接手者先重读本入口、主 Spec、适用 task 和 current code。交接只保留当前请求、确切 refs、已完成且有证据的变化、未闭合项和下一可执行责任，不复制旧会话堆积状态。
