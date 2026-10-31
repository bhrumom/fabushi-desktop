# OpenClaw 全源码 Rust 迁移 — Source of Truth

Status: active  
Project ID: OCRM-001  
Revision: 3
Last updated: 2026-10-31（Asia/Shanghai）
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

2026-10-31T02:34:32Z 的新一轮只读核对发现 OpenClaw `main` 已漂移到
commit=`298546d88934bf05bc7e208d13bf6ce2c538e3f2`、tree
`120f973ab7da856b4bc1c545c437dd94d35ef9eb`；递归 tree 未截断，共 63,847
entries（61,606 blobs、2,241 directories）。根输入 blob 已按该 subject
重读并写入 lock：`package.json`=`5e468c9625b7670af23096df4588fd5e066c58a8`、
`pnpm-lock.yaml`=`df897f3361f5968912f27b31971a60c08a1132b4`、
`pnpm-workspace.yaml`=`32ce4d83de4aac6a1e3665baba09a3aa9582960c`；
`.gitmodules` 仍不存在。该 identity 当时取代 `2c157ed…/3fb582…` 成为 discovery
reference，但不提升 source authority，不使旧 inventory/closure
可复用，也不改变任何 global flag；必须针对新 exact subject 取得 Actions
inventory、依赖闭包、LFS/完整性及 evidence envelope。

2026-10-31T03:34:16Z 的再次只读核对发现 OpenClaw `main` 已漂移到
commit=`ece4ca85af0e0ffa7cc619522b9e8b7e40b08da9`、root tree
`25f9aa8f30eef5b41f8873ddc0a7c4a8a0350772`；递归 tree 未截断，共 63,865
entries（61,624 blobs、2,241 directories）。该 exact subject 的
`package.json` blob 为 `bb155612041150b48dc93246039cb4eac65306ec`，其余
root dependency/license inputs 与上一 candidate 相同。该 identity 已同步
写入 `upstream.lock.json`，但仍是 discovery-only；不得复用旧 inventory、
依赖闭包、LFS/完整性或 Actions provenance，亦不得提升任何 authority 或
global flag。

2026-10-31T03:46:32Z 的再次只读核对发现 OpenClaw `main` 已漂移到
commit=`5ce7a597cbe3f8bd42456cac3b5dbe0e74d9e321`、root tree
`63813e46d6f93598cce22bff02f9def2c098f5ca`；递归 tree 未截断，共 63,872
entries（61,631 blobs、2,241 directories）。该 exact subject 的
`package.json`、`pnpm-lock.yaml`、`pnpm-workspace.yaml`、`.gitattributes`、
`LICENSE` 与 `THIRD_PARTY_NOTICES.md` blob identities 分别为
`bb155612041150b48dc93246039cb4eac65306ec`、
`df897f3361f5968912f27b31971a60c08a1132b4`、
`32ce4d83de4aac6a1e3665baba09a3aa9582960c`、
`c6118b226a4a760f7861b6b7b1e7dccd943384e4`、
`ed064819ab537575d0cad902f7e5af4a3488b1d0`、
`0c9c123760188915a384ce7275df4096114e8f90`；`.gitmodules` 仍不存在。
该 identity 已同步写入 `upstream.lock.json`，但仍是 discovery-only；不得复用
旧 inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升任何
authority 或 global flag。

2026-10-31T03:54:32Z 的再次只读核对发现 OpenClaw `main` 已漂移到
commit=`129583af012bf4177b37ba6d88e8b2e5fa0d532f`、root tree
`84860d166bdddef60cbc3ede3909d1a28696b4d9`；递归 tree 未截断，共 63,874
entries（61,633 blobs、2,241 directories）。该 exact subject 的根输入
blob identities 与上一 candidate 相同，`.gitmodules` 仍不存在。该 candidate
已同步写入 `upstream.lock.json`，但仍是 discovery-only；不得复用旧
inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升任何
authority 或 global flag。

2026-10-31T04:08:46Z 的再次只读核对发现 OpenClaw `main` 已漂移到
commit=`510e1687e965c1a5e2c372f6622704c88448fb5b`、root tree
`8260de46bfdd6ac1a78369ebf2343cb255de20e5`；递归 tree 未截断，共 63,877
entries（61,636 blobs、2,241 directories）。该 exact subject 的根输入
blob identities 与上一 candidate 相同，`.gitmodules` 仍不存在。该 identity
已同步写入 `upstream.lock.json` 的 `live_candidate`，但仍是 discovery-only；
不得复用旧 inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升
任何 authority 或 global flag。

2026-10-31T04:20:22Z 的再次只读核对发现 OpenClaw `main` 已漂移到
commit=`8e61893b6e692b14d4741ccdf681e1f932289f2f`、root tree
`4e9cca115794d90ad9f056871103790101022764`；递归 tree 未截断，共 63,880
entries（61,639 blobs、2,241 directories）。该 exact subject 的根输入
blob identities 与上一 candidate 相同，`.gitmodules` 仍不存在。该 identity
已同步写入 `upstream.lock.json` 的 `live_candidate`，但仍是 discovery-only；
不得复用旧 inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升
任何 authority 或 global flag。

2026-10-31T04:25:12Z 的再次只读核对发现 OpenClaw `main` 已漂移到
commit=`10d93efc9c8730b7a2fad831c20c649adec7176b`、root tree
`d33ed7c344544baf60148a41afab9e9a9547e9f4`；递归 tree 未截断，共 63,881
entries（61,640 blobs、2,241 directories）。该 exact subject 的根输入
blob identities 与上一 candidate 相同，`.gitmodules` 仍不存在。该 identity
已同步写入 `upstream.lock.json` 的 `live_candidate`，但仍是 discovery-only；
不得复用旧 inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升
任何 authority 或 global flag。

2026-10-31T04:40:37Z 的再次只读核对发现 OpenClaw `main` 已漂移到
commit=`44069f9c1ab6d1713c2e558dd631fccf8b5adda0`、root tree
`44b33ff34591447276bb9a42916ce23ed029dead`；递归 tree 未截断，共 63,888
entries（61,647 blobs、2,241 directories）。该 exact subject 的根输入
blob identities 与上一 candidate 相同，`.gitmodules` 仍不存在。该 identity
已同步写入 `upstream.lock.json` 的 `live_candidate`，但仍是 discovery-only；
不得复用旧 inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升
任何 authority 或 global flag。

2026-10-31T05:09:05Z 的只读核对再次发现 OpenClaw `main` 漂移到
commit=`622fa3fab8e154819a329d3c288d13491facea7b`、root tree
`2590235af3db54166cf88801806e2b63b3f4688b`；递归 tree 未截断，共 63,901
entries（61,660 blobs、2,241 directories）。root input blob identities 与上一
candidate 相同，`.gitmodules` 仍不存在。该 exact subject 已同步写入
`upstream.lock.json` 的 `live_candidate`，但仍是 discovery-only；不得复用旧
inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升任何 authority
或 global flag。

2026-10-31T05:14:48Z 的只读核对再次发现 OpenClaw `main` 漂移到
commit=`b924498923dff81a02fb8a89d57828717c6e160d`、root tree
`af9b32fcb093badf0e05912411f5acfa65698e79`；递归 tree 未截断，共 63,902
entries（61,661 blobs、2,241 directories）。root input blob identities 与上一
candidate 相同，`.gitmodules` 仍不存在；`src/acp/commands.ts` 仍不存在，
`src/auto-reply/commands-registry-list.ts` 仍存在。该 exact subject 已同步写入
`upstream.lock.json` 的 `live_candidate`，但仍是 discovery-only；不得复用旧
inventory、依赖闭包、LFS/完整性或 Actions provenance，亦不得提升任何 authority
或 global flag。

2026-10-31T05:51:22Z 的只读核对再次发现 OpenClaw `main` 漂移到
commit=`bcf23b2ea0eee92c8c497c6b1c4d486588e4d5ab`、root tree
`5d9b96f3208eb2daf4c566fc60865a1850e4c77f`；递归 tree 未截断，共 63,946
entries（61,705 blobs、2,241 directories），无 gitlink。`.gitmodules` 仍不存在；
根 package/dependency/license blob identities 与上一 candidate 一致。该 exact
subject 已同步写入 `upstream.lock.json` 的 `live_candidate`，但仍是
discovery-only；不得复用旧 inventory、依赖闭包、LFS/完整性或 Actions
provenance，亦不得提升任何 authority 或 global flag。

2026-10-31T07:55:14Z 的最新只读核对将当前 live discovery reference 更新为
commit=`77b08d1345a8e5734ada19dc95e77f4c5135bd0a`、root tree
`a52b727858379d14b1b1ed4bde71837326c127f7`；递归 tree 未截断，共 63,993
entries（61,752 blobs、2,241 directories）。`upstream.lock.json` 已同步该
`live_candidate`；它仍是 discovery-only，未取得绑定该 exact subject 的 Actions
inventory、依赖闭包、LFS scan、acceptance receipt 或终验记录。

同一 exact subject 的 root dependency input probe 已记录在
`dossiers/upstream-current-candidate-dependency-probe-20261031.md`：确认了
integrity-qualified `pnpm@12.15.0`、六组 workspace pattern、77 个 root runtime
dependencies、57 个 devDependencies，以及同树的 lockfile/workspace blob。该
probe 只证明根输入存在与形状，不等价于递归依赖闭包；所有 closure flags 继续
保持 `false`。

对 `77b08d1345a8e5734ada19dc95e77f4c5135bd0a...2910c0de8d7d290f9a82ca388f2893d067c06cb1`
的 GitHub compare 只读结果为 `ahead_by=1`、`behind_by=0`、`total_commits=1`，
返回的 2 个文件仅为 `scripts/lib/vitest-process-env.mts` 与
`test/scripts/run-vitest.test.ts`；没有 root dependency manifest 或当前已映射
product owner path 变化。该结果支持复用“root input identities unchanged”的狭义
事实，但不支持复用旧 inventory、dependency closure 或 Actions provenance。

2026-10-31T08:02:48Z 的只读远端复核再次发现 `openclaw/openclaw` 的 `main` 已从
`77b08d13…/a52b7278…` 漂移到 commit
`2910c0de8d7d290f9a82ca388f2893d067c06cb1`、tree
`42dc443420e1b5b84e481f170cd8fccfcc00d7be`。递归 tree 仍未截断，63,993 entries
（61,752 blobs、2,241 directories），root dependency/license blob identities
保持不变；lock 已更新 `live_candidate`，但该 subject 仍 discovery-only。此前
dependency probe 绑定旧 candidate，不能直接充当新 subject 的 closure evidence。

2026-10-31T08:07:28Z 的只读远端复核又发现 `openclaw/openclaw` 的 `main` 已漂移到
commit=`db59290c33e1768cce8a461e2697cff7108c5a69`、root tree
`c1e4818f8f877576c02da1573e7dc809d1fdd800`。GitHub compare
`2910c0de…db59290c…` 返回 `ahead_by=2`、`behind_by=0`、`total_commits=2` 和
8 个文件，涉及 UI/Cron windowing、gateway-network 与对应测试；没有 root
dependency input 变化。递归 tree 仍未截断，63,993 entries（61,752 blobs、2,241
directories），根输入 blob identities 保持不变。该 exact subject 已同步到
`upstream.lock.json::live_candidate`，但仍是 discovery-only；旧 candidate 的
inventory、依赖 probe、closure 和 Actions provenance 均不可直接复用。

2026-10-31T08:09:58Z 的只读远端复核又发现 `openclaw/openclaw` 的 `main` 已漂移到
commit=`1b870b2c78e81fded72913ecba732b84b611ca12`、root tree
`199fe97259d0226ce0e4d3dcec603252376c9205`。相对 `db59290c…` 的 GitHub
compare 为 `ahead_by=2`、`behind_by=0`、`total_commits=2`，返回 7 个文件；其中
包含 `src/config/sessions/lifecycle.ts`、`src/cron/isolated-agent/session-cleanup.ts`
与 `src/agents/subagents/registry/subagent-session-cleanup.ts` 等已映射 session/cron
生命周期路径，以及 PDF provider/test 支持文件。根 dependency/license input
identities 保持不变；递归 tree 未截断，为 63,994 entries（61,753 blobs、2,241
directories），无 gitlink。该 subject 已同步到 `upstream.lock.json::live_candidate`，
但 mapped session/cron 路径必须重新读取后才能复用责任证据；旧 inventory、依赖闭包
和 Actions provenance 仍不可复用，authority/global flags 不变。
该 exact-subject 的三条 session/cron runtime delta 已记录在
[`dossiers/upstream-current-session-lifecycle-delta-20261031.md`](dossiers/upstream-current-session-lifecycle-delta-20261031.md)，
仅作为 discovery audit；它不改变 ledger/RTM 状态，也不把 lifecycle revision、
Gateway owner fence 或 cleanup outcome 映射误报为 Rust 生产接线。
随后按已批准的 lifecycle-fence spec 在既有 agent DB、session projection 和
Transcript lifecycle owner 中落下首个实现切片：revision migration/projection 与
delete admission/post-hook recheck 已存在；focused oracle source 已加入
`source/host/tests/agent_lifecycle_contract.rs`，但尚未编译或执行。Rust 侧没有
对应的 cron isolated-session 或 subagent cleanup producer，故 producer 接线、
Actions、packaged/runtime 和 acceptance 仍未闭合；ledger/global flags 不变。

当前 parity ledger 的 9 条 `not-started/not-wired` 责任已在
`dossiers/not-started-responsibility-frontier-20261031.md` 做了逐条 frontier
审计：它们缺少生产 producer/consumer、拥有明确 ACP/plugin blocker，或只能从
contract fixture/metadata 得到间接证据。该 dossier 不改变责任状态，也不把
helper、fixture 或 capability metadata 升级为生产实现。

当前 `projects/openclaw-rust/evidence/` 有开发冻结说明、本地冻结治理说明与历史
upstream drift 记录；尚无绑定 `77b08d13…/a52b7278…` 的 Actions run/attempt/job、
inventory/dependency-closure artifact、不可变 digest、acceptance receipt 或
OCRM-AC01–AC24 终验记录。相关 dossier 仍属于 discovery/static evidence，不能
替代 exact-subject evidence envelope；八项 global flags 继续全部为 `false`。

2026-10-31T08:26:10Z 的只读复核又解析到 `openclaw/openclaw` `main` 为
commit `1d2e685b76b2bb7da964a2d6445cec2e07bf5883`、tree
`a7f33a87af80c203422677be70f83aebb0857666`。递归 tree 未截断，共 63,996
entries（61,755 blobs、2,241 directories），无 gitlink；`package.json`、
`pnpm-lock.yaml`、workspace、license 与 notice 输入身份保持不变。该 subject
仅更新 `upstream.lock.json::live_candidate`，仍是 discovery-only；旧 inventory、
依赖闭包、LFS 与 Actions provenance 不得复用，所有 authority/closure/global
flags 保持不变。

对 `.github/workflows/ocrm-p0-authority.yml` 的只读审阅发现其来源获取步骤使用
lock 的 `upstream` 字段；已加入 fail-closed preflight，要求该字段与
`live_candidate` 的 commit/tree 完全一致且 `accepted_for_this_implementation_round`
为 `true`，否则不得生成或上传 Actions inventory。当前 lock 仍不满足该条件，
因此不会把实现 baseline 的旧来源误报为当前 `002f25a…/553796f…` subject；本轮
没有触发 workflow。

同一 workflow 的 `persist-verified-inventory` job 已在本地提交
`390108a5e` 中移除旧 PR `#169`/旧分支硬编码，改为仅允许同仓库当前 PR
head，并以事件 head ref 做合法性检查和 subject CAS 写入；新的提交仍要求
下一次 exact-head Actions receipt，不能继承旧 receipt。开发冻结期间不触发
该 workflow、不 push，也不把这项治理修订当作 inventory/receipt 正面证据。

同一时间的只读身份审计确认：`inventory/index.json` 与全部 58,750 条分片
记录仍绑定实现 baseline `ad8d12f126d29b0db204187adf5092db80294e70` /
tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`；`parity-ledger.json` 与
`requirements-traceability-matrix.json` 也绑定该 baseline。它们不匹配当前
`77b08d13…/a52b7278…` discovery candidate，未被重标为当前来源；在同一 exact
subject 上重新取得 Actions inventory、依赖闭包和 receipt 前，不得把这些旧
产物当作当前 source closure 或完成证据。

双向责任引用审计还发现旧 baseline inventory 的 responsibility projection 尚未
包含两条已在 parity ledger 中建立的非 P1 责任：
`OCRM-RESP-NODE-HOST-LOCAL-RUNTIME-PROJECTION-001` 与
`OCRM-RESP-SKILLS-CONFIG-MUTATION-001`；其对应路径已有旧责任 ID，且不存在
悬空 ledger/RTM ID。另有四条 Crabbox P1 task-local 责任按既定规则不进入
inventory/RTM 的项目级责任集合。由于分片的压缩摘要和未压缩摘要绑定
canonical inventory 身份，不能手工改写旧分片来伪造 projection closure；这些
差异必须在 `002f25a…/553796f…` exact subject 的新 inventory 生成与独立
验证中闭合。

这里的“四条”仅是该旧 baseline projection 快照中的历史计数，不代表当前
Crabbox task-local responsibility 数量；当前数量和 canonical ID 以
`management/tasks/P1-crabbox-worker-owner.md` 与当前 parity ledger 为准。

上述 baseline projection 审计段落保留其当时的 candidate 快照；截至
2026-10-31T05:51:22Z，当前 discovery identity 已由
`bcf23b2ea0eee92c8c497c6b1c4d486588e4d5ab` /
`5d9b96f3208eb2daf4c566fc60865a1850e4c77f` supersede。旧 baseline inventory、
其 projection 计数和旧 candidate identity 均不得解读为当前 source closure。

2026-10-31T07:55:14Z 的只读远端复核显示 `openclaw/openclaw` 的 `refs/heads/main`
已漂移到 commit `77b08d1345a8e5734ada19dc95e77f4c5135bd0a`、tree
`a52b727858379d14b1b1ed4bde71837326c127f7`；recursive tree 未截断，共 63,993
entries（61,752 blobs、2,241 directories）。这些值已更新到
`upstream.lock.json` 的 `live_candidate`，但仍保持 discovery-only；没有 fresh
GitHub Actions inventory、依赖闭包或 LFS scan，因此没有提升 accepted identity，
也没有改变任何 closure/global flag。

对旧 baseline inventory 的只读计数还显示：58,750 条路径中有 57,331 条仍为
`responsibility_status=unclassified` 且 `disposition=null`；其余 1,419 条已有
read-complete/responsibility-mapping 记录，但当前 applicability/disposition 字段仍为
`null`。该计数是当前 AC02/G02 final 未闭合的直接证据，不是把
unread 强行当作失败门槛，也不能被结构性 inventory 完整或已完成的 scoped
责任行抵消。

随后对 ledger source references 与旧 baseline inventory 的反向 projection 做了
只读核对：22 条被当前责任 source_refs 明确引用的路径→责任边仍未在旧分片中
形成对应的 responsibility projection；其中 12 条路径仍为
`unread/unclassified`，另 10 条虽已 `read-complete/mapped`，但旧分片只投影了
其他责任 ID，包括本轮补齐的 session cleanup producer 文件。该差异是旧
inventory 与后续 scoped source-read 证据之间的 provenance gap；它不证明
source closure，也不允许手工改写旧分片来制造双向映射。必须在同一新 exact
subject 上重新生成 inventory 并由 Actions 验证后，才能闭合这些 reverse edges；
本轮未运行 validator、generator 或 CI。

2026-10-31 的只读生产可达性审计还纠正了 10 条错误的
`implemented/wired` 状态：`OCRM-RESP-AUTO-REPLY-HEARTBEAT-001`、
`OCRM-RESP-CHANNEL-ACCESS-CONTROL-001`、`OCRM-RESP-CHANNEL-INBOUND-DEBOUNCE-001`、
`OCRM-RESP-CHANNEL-STREAMING-CHUNKING-001`、`OCRM-RESP-CHANNEL-THREAD-BINDINGS-001`、
`OCRM-RESP-CRON-SESSION-REAPER-001`、`OCRM-RESP-MEM-CITATIONS-FORMAT-001`、
`OCRM-RESP-TALK-AUDIO-PORT-FENCE-001`、`OCRM-RESP-TALK-EXACT-SPEECH-001`、
`OCRM-RESP-TALK-TURN-CONTEXT-001`。这些目标 symbol 只在 contract tests/文档
中存在，或尚未出现在声明的生产 owner 文件；因此 ledger 已降为
`implementing/not-wired`。测试 oracle 和源阅读证据保留，生产实现、真实
producer→owner→consumer 接线与 packaged acceptance 仍待完成；本审计未运行
OCRM 测试或本机构建。

其中 `OCRM-RESP-CHANNEL-INBOUND-DEBOUNCE-001` 已在既有
`source/host/src/connectors/channel_delivery.rs` owner 内增加
`ChannelInboundDebouncePolicy` / `InboundChannelDebounceGate` 的纯策略实现：
按 normalized channel 解析 bounded delay，`abort`/`stop`/`status` 与 media
立即放行，text 在窗口内返回 `Debounced`。这只是 implementation/source
contract evidence；由于尚无真实 inbound producer 与 turn consumer，ledger
仍保持 `wiring=not-wired`、`verification=not-run`、`acceptance=pending`。

本次 heartbeat 复核还纠正了该责任的 oracle 绑定：`OCRM-PLUGIN-O07` 实际是
web-search/fetch payload bounds contract，不是 heartbeat 测试；ledger 已移除这条
错误的 oracle、test 和未实现的 `HeartbeatService::poll` typed contract。heartbeat
仍保持 `implementing/not-wired`，直到出现真实 producer、timer admission、runner
consumer、response-notification consumer 及独立 heartbeat oracle；没有因此新增
实现或验收证据。

同一轮 memory citations 复核确认：当前 content-search index/result contract 只保留
entry ID、role、timestamp 与 snippet，没有可信的 source-line provenance；ledger 原先
绑定的 `OCRM-MEM-O06/O10` 分别是 synthesis evidence rejection 与 per-agent
match-capping/recency tests，不是 citation-format oracles。该责任因此继续保持
`implementing/not-wired`，不得从 snippet 或 entry 顺序推导 `#L{n}` 行号，直到
provenance contract 与独立 citation oracle 同时存在。

随后对 320 条 wired integration path claim 做了只读存在性检查：5 条 Browser
producer path 纠正为实际的 `source/host/src/runner/tools/` 位置；
`OCRM-RESP-AGENT-ROSTER-AUDIT-001` 的声明 consumer
`source/host/src/security_diagnostics.rs` 不存在，且全仓没有其生产调用者，
因此已将该行降为 `not-wired` 并保留明确 pending consumer。该修订只纠正
接线台账，不创建新的 owner 或替代 packaged/Actions 证据。

重新运行接线路径审计后，316 个仍为 wired 的 integration path claim 全部指向
存在的仓库文件；剩余 28 个复合描述使用 `->`、`plus` 或逗号列举多个调用
步骤，不能按单一 symbol 文本匹配推断缺失。当前未发现新的 wired 不存在路径；
该结果仍不替代调用图、Actions receipt 或 packaged runtime 证据。

随后按 qualified Rust symbol 的最后一段做了当前工作树反向存在性检查：80 条
`implementation=implemented,wiring=wired` 责任的目标文件与目标 symbol 均可定位；
其中带 `Type::method` 的声明按方法名检查，避免把 Rust qualified name 当作字面
源码行。该检查没有发现新的明确缺失目标，但仅证明静态 target presence，不替代
producer→owner→state/storage→consumer 调用图、Actions receipt、packaged runtime
或真实服务/设备验收。

降级后的反向审计确认其余 79 条 `wiring=wired` 责任均可在生产源码范围
（`source/host/src`、`source/node-agent-coordinator/src`、
`source/electron-main` 或 `frontend/src`）定位目标 symbol；未发现第二批仅由
tests 支撑的 wired 行。该结果只证明静态目标可达性，不证明 Actions exact
receipt、packaged runtime reachability、真实服务/设备或最终验收。

RTM 证据完整性审计显示 48 行中 31 行为 `in-progress`，每行至少有 evidence
与 reviewer；17 行为 `not-started`，其中基础/终验行保留空 evidence、责任或
gate，未伪造尚未取得的证明。没有发现 `in-progress` 但完全无 evidence 或无
reviewer 的误标；这只证明 RTM 状态诚实，不证明任何 AC 已接受。

同一只读维度审计通过：1,419 条 `read-complete`/mapped 记录均有 reviewer、
source references、responsibility mapping evidence；57,331 条 `unread`/unclassified
记录没有伪造 applicability/disposition；未发现 mapped-without-responsibility、
unclassified-with-disposition、
缺 reviewer 或缺 evidence 的矛盾。该结果只证明旧 baseline inventory 的内部
状态自洽，不证明当前 candidate 的 source closure 或 AC02 final。

## 5. 机器记录的初始化与终态

2026-10-31 governance amendment: AC02/G02 final requires zero omitted,
undispositioned and unassessed applicable product responsibilities, plus zero
applicable orphan/unmapped responsibilities. Honest unread/unclassified
progress for ordinary non-product grouped evidence remains reportable and is
not an independent AC02 hard gate. This clarification does not change any
global flag or source-closure state.

P0 在 Actions 建立 lock、inventory、parity ledger、RTM、dossiers 和 evidence 索引；本次不提交伪造的完整台账或空清单来充当 baseline。

冻结期本地治理载体见
[`evidence/development-freeze-local-governance-20261031.md`](evidence/development-freeze-local-governance-20261031.md)。
它只记录未推送 local subject、远端 ref 状态和冻结策略，不是 Actions
receipt、source authority 或产品验收证据；其 carrier commit 也不能继承旧
subject 的 receipt。

各 flag 的适用集合、true 谓词、负责 gate 和失效条件以主 Spec §15.1 为唯一规范。`source_closure_ready` 表示完整取得并枚举来源/依赖，不表示已阅读全文；AC02 的全量闭合是“0 适用性/处置未闭合、0 未评估适用责任”，而不是把 58,750 个路径强行变成全文深读。全部适用产品责任理解、owner/合同/oracle 双向映射和非产品 disposition 证据归 `responsibility_mapping_complete`；责任双向映射由 AC03 单独验收。P0 G02 structural 可以保留诚实 unread/unclassified；最终 G02 必须消除适用性/处置未闭合及未评估适用责任，但仍要如实报告尚未深读的非适用成组路径。

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

2026-10-31T09:38:43Z 的只读远端复核发现 OpenClaw main 已再次漂移到
commit=d27a8c9d9fec49a81084993bf16c3fefc4efd508、root tree
e378a64af5f7409473636e6758cca43f4a204fcc；committer 时间为
2026-10-31T09:38:43Z。GitHub recursive tree 未截断，共 64,009 entries
（61,768 blobs、2,241 directories、0 gitlinks）。根 package/lock/workspace/
license/notice blob identities 与上一 candidate 保持不变。该 exact subject 已
同步写入 upstream.lock.json::live_candidate，仍是 discovery-only；本次未
重新运行 compare、Actions inventory、依赖闭包、LFS/完整性或 acceptance
receipt，因此不得复用上一 candidate 的 closure/provenance，也不改变任何
authority 或 global flag。

相对上一 candidate `ad57b6135a55a6792b6d703bc1750954d854d6aa`，只读 compare
显示当前 subject ahead 1、behind 0，共 16 个文件变化；其中 `package.json`
本身发生变化，另外涉及 Plugin SDK subpaths/testing、package boundary、XAI
tsconfig、package-dist inventory、plugin contracts 和审计文档。此前绑定
`ad57b613…` 的 dependency probe、Gateway scoped audit 和 drift mapping 只能
作为历史证据，当前 subject 的受影响文件必须重新读取。

该 exact subject 的根输入 probe 已重新读取：package.json、pnpm-lock.yaml、
pnpm-workspace.yaml、.gitattributes、LICENSE 和 THIRD_PARTY_NOTICES.md 的 blob
身份均与上一 candidate 保持不变；.gitmodules 仍不存在。递归 tree 计数和根输入
身份不等价于依赖闭包，仍需 exact-head Actions inventory、包 integrity/LFS 和
递归 dependency closure 才能提升任何 authority flag。

对该 exact subject 的只读 lock-structure probe 进一步发现：root
`package.json` 声明 77 个 runtime、57 个 dev、1 个 optional dependency，
但 `pnpm-lock.yaml` 的唯一 importer `.` 只记录 `pnpm@12.15.0`；lock 的
`packages`/`snapshots` 各只有 15 条，均为 pnpm 自身平台包且有 integrity，未
出现 root 声明的应用依赖解析记录。这是 dependency closure 的明确负证据，
不是把 manifest 计数或 pnpm integrity 误报为 closure；必须在 exact-head
Actions 中核对生成/提交策略、workspace manifests、patched/optional/peer
关系和完整包来源后才能继续。`dependency_closure_enumerated` 与
`source_closure_ready` 保持 false。

同一 recursive tree 中又发现 206 个 workspace-pattern 下的 `package.json`
候选（root、ui、packages、extensions、examples、tools/solid-lint），而 lock
仍只有一个 importer。该集合差异进一步证明当前 lock 不能直接作为完整
workspace dependency closure；候选 manifest 仍需逐项绑定 workspace/包身份、
patch、optional/peer、外部 tarball integrity 和生成/安装 provenance。

对上一 candidate `b7cc7ebc9b442ed7eb73a92336c2a9886b578f7b` 的只读 compare
显示当前 subject ahead 4、behind 0，共 25 个文件变化。变化覆盖 Gateway
protocol/client handshake 与 pending-request、TTS preference/fallback、UI
Gateway/session/chat history/send queue，以及 Plugins 页面/生成测试；另有
少量 gateway/TTS allocation 配置和文档变更。该 compare 只建立 drift frontier，
不证明新 subject 的递归闭包或语义读取；受影响责任必须绑定
`ad57b6135a55a6792b6d703bc1750954d854d6aa` 重新读取，旧 subject 的对应
inventory、dossier、Actions provenance 不得直接复用。

2026-10-31T09:14:47Z 的最新只读核对将当前 OpenClaw `main` 更新为
`e0f1e0a6af388d18fcfcda210f39f5106c7f7c10` / tree
`63d39e4b31fbb462aab29ff0f8b728597bb3ca20`（committer
`2026-10-31T09:13:56Z`）。递归 tree 未截断，共 64,003 entries（61,762 blobs、
2,241 directories、0 gitlinks）；package/lock/workspace/license 输入 blob
identities 与上一 candidate 保持不变。该 subject 已同步写入
`upstream.lock.json::live_candidate`，仍是 discovery-only；不得复用旧
inventory、依赖闭包、LFS/完整性或 Actions provenance，也不改变任何
authority/global flag。

同一 exact subject 的根 dependency input probe 尚未重新生成；此前的
`dossiers/upstream-current-candidate-dependency-probe-e8dade03-20261031.md`
绑定旧 subject，不能作为当前 closure evidence：
确认了 integrity-qualified `pnpm@12.15.0`、六组 workspace pattern、77 个
root runtime dependencies、57 个 devDependencies，以及同树的
lock/workspace/license blob。该 probe 只证明根输入存在与形状，不等价于递归
dependency closure；closure flags 继续保持 `false`。

当前 exact candidate 的 Gateway drift 已完成一项只读 scoped audit：
`packages/gateway-client` 新增 negotiated `hello.policy.maxPayload`、UTF-8
发送前 payload byte guard、typed `GatewayPayloadLimitError` 和 64 KiB pre-auth
default；协议包是唯一 limit owner，server constants 仅 re-export。Fabushi 当前
`source/node-agent-coordinator/src/gateway/gateway_client.rs` 是 HTTP/SSE client，
`source/host/src/gateway_server.rs` 是 Host server boundary，尚未证明等价的
WebSocket handshake/pending-request negotiated consumer。详见
`dossiers/gateway-client-payload-limit-current-ad57b613-20261031.md`；不提升
任何 ledger/flag，且不创建第二 Gateway transport owner。

2026-10-31T09:47:18Z 的最新只读远端复核将 OpenClaw `main` 更新为
`2464bd85e0bc2927701c3147bf3dfc6993daaae3` / tree
`08b1e6627c310e8640a340be49838e2936b44346`。递归 tree 未截断，共 64,009
entries（61,768 blobs、2,241 directories、0 gitlinks）；commit message 为
`fix(test): bind SDK hosts in direct Gateway fixtures (#189244)`。根
package/lock/workspace/license/notice identities 与上一 candidate 保持不变。
该 subject 已同步写入 `upstream.lock.json::live_candidate`，仍为
discovery-only；不得复用上一 candidate 的 inventory、dependency closure、
LFS/integrity 或 Actions provenance，`baseline_identity_accepted`、
`source_closure_ready` 与所有 global flags 继续保持 false。固定
`ad8d12f...` implementation subject 不变。

对固定 implementation subject `ad8d12f126d29b0db204187adf5092db80294e70` 的
本地 Git object 只读 probe 还发现：精确 `pnpm-lock.yaml` 在同一文本中有两套
`importers`、`packages`、`snapshots` 顶层区段（分别从第 4/13/97 行和第
221/3193/11165 行开始）；前一套只有 pnpm 自身 15 个条目，后一套才包含应用
workspace 图。root manifest 是 75 runtime、58 dev、1 optional、0 peer，树中有
208 个 `package.json` 候选。详见
`dossiers/upstream-fixed-subject-dependency-probe-20261031.md`。这构成固定
subject dependency closure 的直接负证据；在 exact-head Actions 解释/重建同一
subject 的 canonical lock、完整 importer/package/patch/native/optional/peer/
integrity/license provenance 前，`dependency_closure_enumerated`、
`source_closure_ready` 及 authority flags 继续保持 false。

2026-10-31 lock parser correction：上述固定 subject 的 `pnpm-lock.yaml` 并非
同一 YAML 文档中的重复顶层键，而是第 158 行 `---` 分隔的两个合法文档。
第一文档是 pnpm 12.15.0 自身的 1 importer/15 packages/15 snapshots；第二
文档是 OpenClaw 应用 workspace lock，含 200 importers、1,844 packages、
1,846 snapshots、6 patchedDependencies。对 exact Git object 使用
`YAML.load_stream` 后，第二文档 root importer 的 75 runtime、58 dev、1
optional 条目与 root manifest 的 134 个依赖名完全匹配；root resolutions
全部能落到 package/snapshot key，3,396 条 snapshot dependency edge 在处理
pnpm alias 后全部可解析。这纠正了单文档 Psych 读取造成的“只有 pnpm lock”
解释，但不等于 accepted dependency closure；workspace manifest/importer
身份、patch/native/optional/peer 语义、外部 archive bytes、license/notice
provenance、source closure 与 exact-head Actions 仍未验证，故所有 closure/
authority flags 继续保持 `false`。

同一 fixed subject 的 workspace 对齐重核还比较了全部 `package.json` 路径与
应用 lock importer：树中 208 个 manifest 中有 200 个与 importer 一一匹配，且
没有 importer 缺少 manifest；其余 8 个明确位于 `.github`/release、`qa`、
`scripts`、`src/claws/fixtures` 或 `test/fixtures`，不属于声明的 workspace
package roots。该结果消除了此前 workspace count 的歧义，但仍只属于静态
manifest/importer mapping；外部包 archive/integrity/license provenance、
source closure 和 exact-head Actions authority 仍未闭合。

2026-10-31 inventory field audit：当前提交的固定 subject shards 与
`ad8d12f.../04ccce...` 一致，58,750 行路径唯一、`omitted_known=0`，其中
57,331 行仍是 `unread/unclassified`，1,419 行是 `read-complete/mapped`。
但所有行的 `applicability_status` 与 `disposition_status` 仍为旧 materialization
留下的 `null`，不是已分流或已完成 disposition 的证据；因此
`applicability_disposition_closure` 继续为 `false`。现有
Actions 中的 `expand_inventory.py` 会在下一次 exact-subject materialization
时为未复用审查字段发出显式 `unknown/pending`，随后由 `shard_inventory.py`
持久化展开后的记录及 applicability/disposition 聚合投影；开发冻结期间不
手工重写 generated shards。详见
`dossiers/inventory-applicability-field-audit-20261031.md`。

2026-10-31T11:19:02Z 的只读远端复核发现 OpenClaw `main` 又推进到
`77f272bd7e5ad7db4c96e8bc4d440d583ab17e7f` / tree
`a3402ffe65237a9e616e0a287d7b7119c9cd20de`。GitHub recursive-tree API 返回
`truncated=false`，共 64,057 entries（61,816 blobs、2,241 directories、0
gitlinks）；commit message 为 `fix(ci): generate published-driver package
declarations (#189209)`。根 package/lock/workspace/license/notice blob
identities 与上一 candidate 保持不变，但该 exact subject 仍是
discovery-only；旧 inventory、依赖闭包、LFS/integrity、语义阅读和 Actions
provenance 均不得复用。`upstream.lock.json::live_candidate` 已更新，固定
implementation subject 与全部 authority/completion flags 不变。

2026-10-31T11:21:03Z 的只读远端复核发现 OpenClaw `main` 再推进到
`ce908375fed062dd75abab62d57fc840b96a144e` / tree
`f2db36d9a98e23753ce1130a84cdae7a4c57488d`。GitHub recursive-tree API 返回
`truncated=false`，共 64,060 entries（61,819 blobs、2,241 directories、0
gitlinks）；commit message 为 `fix: session deletion delays unrelated agent
writes (#189287)`。根 package/lock/workspace/license/notice blob identities
仍未变化，但该 exact subject 仍是 discovery-only；所有前一 candidate 的
inventory、依赖闭包、LFS/integrity、语义阅读和 Actions provenance 均不得
复用。lock 的 live candidate 已更新，固定 implementation subject 与全部
authority/completion flags 不变。

2026-10-31T11:24:57Z 的只读远端复核发现 OpenClaw `main` 再推进到
`8487c89fbedd53238f42b34bb446723a5629bb16` / tree
`16c359a40e19ea69591704bff7d626677673b112`。GitHub recursive-tree API 返回
`truncated=false`，共 64,062 entries（61,821 blobs、2,241 directories、0
gitlinks）；commit message 为 `perf(ci): skip unrelated PR test changes on
Darwin (#189294)`。根 package/lock/workspace/license/notice blob identities
仍未变化；该 exact subject 仍为 discovery-only，前一 candidate 的
inventory、依赖闭包、LFS/integrity、语义阅读和 Actions provenance 均不得
复用。lock live candidate 已更新，固定 implementation subject、ledger 修复
及全部 authority/completion flags 不变。

2026-10-31T11:29:34Z 的只读远端复核发现 OpenClaw `main` 再推进到
`7293e892728aef27996d8a15626e006e2d537dcc` / tree
`b02727b750a5439df95f2178dd983f2a1bd79f03`。GitHub recursive-tree API 返回
`truncated=false`，共 64,064 entries（61,823 blobs、2,241 directories、0
gitlinks）；commit message 为 `refactor(skills): read the review workspace
from the session entry (#189302)`。根 package/lock/workspace/license/notice
blob identities 仍未变化；该 exact subject 仍为 discovery-only，前一 candidate
的 inventory、依赖闭包、LFS/integrity、语义阅读和 Actions provenance 均不得
复用。lock live candidate 已更新，固定 implementation subject、ledger 修订及
全部 authority/completion flags 不变。

2026-10-31T11:41:30Z 的只读远端复核发现 OpenClaw `main` 再推进到
`55c411df82200a801d790d8d1fe5ad862f79d33f` / tree
`0ab0a51751fb0c9cb3749fb477039b16e5772db8`。GitHub recursive-tree API 返回
`truncated=false`，共 64,068 entries（61,827 blobs、2,241 directories、0
gitlinks）；commit message 为 `fix(ci): settle skill reader descendant
animations (#189235)`。根 package/lock/workspace/license/notice blob identities
仍未变化；该 exact subject 仍为 discovery-only，前一 candidate 的 inventory、
依赖闭包、LFS/integrity、语义阅读和 Actions provenance 均不得复用。lock live
candidate 已更新，固定 implementation subject、ledger 修订及全部
authority/completion flags 不变。

2026-10-31T11:40:24Z 的只读远端复核发现 OpenClaw `main` 再推进到
`70da3265b8bf09e25acb1a14ce246982520a1140` / tree
`b8afd36d3c8d722dbab9a05781605fd79d2d5a50`。GitHub recursive-tree API 返回
`truncated=false`，共 64,067 entries（61,826 blobs、2,241 directories、0
gitlinks）；commit message 为 `refactor(test): share scoped environment
restoration (#189314)`。根 package/lock/workspace/license/notice blob
identities 仍未变化；该 exact subject 仍为 discovery-only，前一 candidate
的 inventory、依赖闭包、LFS/integrity、语义阅读和 Actions provenance 均不得
复用。lock live candidate 已更新，固定 implementation subject、ledger 修订及
全部 authority/completion flags 不变。

对 fixed subject 应用 lock 与 exact Git tree 的进一步只读重核显示：200 个
importer 路径全部匹配 manifest，263 条 `link:` edge 按 importer 相对路径解析
后全部落到现有 workspace manifest；1,844 个 package resolution 全部带声明的
integrity，另有 285 个 platform-qualified package、318 个 optional snapshot、
131 个带 peerDependencies 的 package，6 个 patched dependency key 均有对应
`patches/` 文件。这些是静态一致性和声明 provenance，不证明 archive bytes、
patch 实际应用、native build、peer 环境、license/NOTICE 内容或 exact-head
Actions reproduction；closure 与 authority flags 继续保持 `false`。

2026-10-31T11:34:55Z 的只读远端复核发现 OpenClaw `main` 再推进到
`d04091590ac09ffb743da149ac99e147283eb730` / tree
`1f60bc3eeb1ab36c984d73066392d8bccbf45ed6`。GitHub recursive-tree API 返回
`truncated=false`，共 64,066 entries（61,825 blobs、2,241 directories、0
gitlinks）；commit message 为 `fix: keep background process launches from
stalling the Gateway (#189277)`。根 package/lock/workspace/license/notice
blob identities 仍未变化；该 exact subject 仍为 discovery-only，前一 candidate
的 inventory、依赖闭包、LFS/integrity、语义阅读和 Actions provenance 均不得
复用。lock live candidate 已更新，固定 implementation subject、ledger 修订及
全部 authority/completion flags 不变。
