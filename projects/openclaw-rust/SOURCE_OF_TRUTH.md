# OpenClaw 全源码 Rust 迁移 — Source of Truth

Status: active  
Project ID: OCRM-001  
Revision: 1  
Last updated: 2026-10-22（Asia/Shanghai）  
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
- `read`、`mapped`、`implemented`、`wired`、`verified`、`accepted` 独立，按主 Spec §6.4.1 的责任类型完成合同；业务必须 Rust/生产接线，非运行时职责保留实际消费者和处置证据。目录、文件、crate、manifest、mock、旧 CI 不证明全部功能。
- 所有可执行验证只在 GitHub Actions，包括 generator/schema/lint/build/test/E2E/visual/performance/package/device acceptance；不沿用其他项目的 `htch-runtime` 例外。
- 凭据、签名、真实设备/渠道/服务未配置是 blocked；保留解除条件并推进无依赖任务，不改变最终目标。
- 许可证、版权、NOTICE、SBOM 与来源保留；产品统一 Fabushi，外部服务名称和必要兼容字段按合同保留。

## 4. Discovery 快照，不能当作完成状态

| 输入 | 身份 |
| --- | --- |
| OpenClaw main | `e8cde7518c558f1b4e44cb8139a149289b9719ad` |
| OpenClaw tree | `25c8ee611b72762ae1a0170124fa1fee9e519704` |
| Fabushi 编辑前 main | `ba9ecce728ef021fee609796c8704d8af6fe4b5a` |
| Fabushi 编辑前 tree | `29e6837e6ea9098d64c5de3e0952b3960abbfc86` |

这些是 2026-10-22 spec 研究输入。文档发布后目标 HEAD 已改变，下一轮必须重新读取 live refs；上游变化需明确 rebaseline。当前未完成全量递归 file inventory/semantic read，不能填造总文件数或覆盖率。

本次已读取主目录、关键文档与部分核心源码。观察到 src 77、extensions 176、skills 50、custodian-skills 4、packages 24、apps 8、crates 2 个直接子目录。plugin inventory 列 175 个插件（73 core / 99 official external / 3 source-only），extensions 另含 test-support。这些数字不是逐文件/功能验收进度。

目标 main 有大量可复用 owner，但外部 channel default 仍是 unregistered seam；Memory、MCP、Human/Call、Browser 源码存在不等于 OpenClaw parity。未合并的 PR #165/#166/#167/#168 仅作协调依赖；正在进行的 TDRP PR #154 及其 evidence 不被本项目改写。

## 5. 机器记录的初始化与终态

P0 在 Actions 建立 lock、inventory、parity ledger、RTM、dossiers 和 evidence 索引；本次不提交伪造的完整台账或空清单来充当 baseline。

各 flag 的适用集合、true 谓词、负责 gate 和失效条件以主 Spec §15.1 为唯一规范。`source_closure_ready` 表示完整取得并枚举来源/依赖，不表示已阅读全文；全部语义理解和责任映射归 `responsibility_mapping_complete`。P0 G02 structural 可以保留诚实 unread/unclassified，最终 G02 不能。

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
| recursive_source_total / unread / incomplete | 尚未建立，unknown，不是 0 |

只有 OCRM-AC01 至 OCRM-AC24 全部取得当前 upstream/target 身份的正面证据并独立接受，才允许宣称完整迁移。当前文档交付不改变任何其他项目的完成标志。

## 6. 执行与交接

按主 Spec 的 P0→P5 依赖推进，阶段不是范围豁免。全局闭包受阻时，满足主 Spec §11.1 的单元可在自身及传递依赖 authority、完整阅读和合同经审查后进入实现/单元验收；单元通过不改变全项目 flags，缺服务/设备仍阻塞对应验收。每个任务记录 source responsibility、existing owner、具体差异、实现与验收证据；进展以真实生产闭合为中心。阻塞时检查已有支持邮件、记录必要动作和解除条件，继续后续可执行项；恢复时重查事实和 lease。

后续接手者先重读本入口、主 Spec、适用 task 和 current code。交接只保留当前请求、确切 refs、已完成且有证据的变化、未闭合项和下一可执行责任，不复制旧会话堆积状态。
