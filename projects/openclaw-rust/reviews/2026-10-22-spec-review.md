# OCRM-001 初版规范独立审读与处置记录

Date: 2026-10-22  
Scope: documentation only; no implementation or product acceptance  
Status: all six findings fixed; focused document re-review complete

## 1. 冻结范围与审读方式

- Target base commit: `ba9ecce728ef021fee609796c8704d8af6fe4b5a`；base tree: `29e6837e6ea9098d64c5de3e0952b3960abbfc86`。
- Upstream discovery commit: `e8cde7518c558f1b4e44cb8139a149289b9719ad`；tree: `25c8ee611b72762ae1a0170124fa1fee9e519704`。
- 初审冻结 candidate tree: `0e479a4eae155d66e42be1a85d8b77b7b190d6fe`。该对象只用于固定审读输入；不代表已更新分支或产品已验收。
- General reviewer 为独立只读 reviewer，使用 xhigh reasoning；审查下表前五个文件。Source reviewer 独立审查第六个文件并核对固定上游来源；保持其实际继承的运行配置，不声称另有 model override。
- 审读期间作者不改动对应冻结文件。两位 reviewer 均未运行测试、构建、生成器或 executable validation；只报告可证伪的实质问题。

| 文件 | 初审 blob |
| --- | --- |
| `AGENTS.md` | `e6a1f21ba5206cdc807b6f9738dad2cec2942100` |
| `README.md` | `eb8ee8ab3de7c1a7e84dabcc0787895950a8292b` |
| `docs/specs/openclaw-rust-equivalence-migration.md` | `e0c489a8838ec6652400bf24d950f8885b622351` |
| `projects/openclaw-rust/SOURCE_OF_TRUTH.md` | `852609ad5e00176d8eafd74ade2e68a3c9ee361b` |
| `projects/openclaw-rust/management/tasks/P0-source-baseline-and-owner-audit.md` | `297d393a7be1540c0ce3d47deb0e68b1d0a52ea4` |
| `projects/openclaw-rust/module-map.md` | `31d2b427dd4e1955962f682c216f728331c8890e` |

## 2. 先决策、后修复

以下决定在修改被审文件前记录。全部选择 FIX；没有通过降低全量范围或把阻塞改成通过来处理发现。

| ID / 级别 | 发现、位置与触发条件 | 决定与理由 | 预定修复 / 聚焦复审 |
| --- | --- | --- | --- |
| OCRM-REV-001 / P1 | 主 Spec §6.3–6.4、AC04/AC05 允许表现、平台桥接、测试、构建、文档与资源处置，却统一要求每责任 Rust 实现和 packaged runtime 路径。合法非运行时责任会永远不能闭合或被错误 N/A。module map §1 也有同类概括。 | FIX。Rust 约束必须覆盖全部业务责任，同时为全部非业务来源保留可执行、可证据化的责任合同。 | 增加 `responsibility_kind` 和按类型的实现/接入/验收合同；维度级非适用必须有依据，整个责任仍在分母；mixed 文件拆出 Rust 业务责任；同步 AC、SOT、P0 和 map 引用。 |
| OCRM-REV-002 / P2 | 主 Spec §11/§12/§15、SOT §5、P0 §1/§6 未给 flags 的 true 谓词，也未定义全局闭包受阻时独立单元的 authority；P0 的诚实 unread 可能被最终 coverage 标准误判。 | FIX。源身份、完整枚举、完整语义理解、实现和验收必须分开；全项目受阻不能冒领完成，也不应阻止已满足依赖单元推进。 | 定义每 flag 的适用集合、gate、正面证据与失效条件；区分 P0 structural coverage 和 final completion；定义可实施单元及其传递依赖的准入证据，禁止单元通过升级全局 flags。 |
| OCRM-REV-003 / P2 | 主 Spec §6.1/§12.3/AC01/AC23 与 P0 机器产物、退出条件未区分受验 revision 和证据保存 revision；为 A 保存结果形成 B 后，exact-head 规则使证据反复自失效。 | FIX。保留严格 exact-target 终态要求，同时提供不需要在提交中嵌入自身 SHA 的实际发布流程。 | 区分 source declarations、受验 target、不可变 Actions receipt 和证据索引载体；最终结果发布到绑定 exact target 的 artifact/check/attestation；后续提交是新 target，不自动继承旧绿灯；Spec/fixture/validator 也属受验输入。 |
| OCRM-REV-004 / P2 | map M07 把 `src/transcripts` 当 Agent 聊天历史；源 provider/store 类型实际承担 meeting/audio capture、participants/utterances、source access、summary/export 与 cleanup。 | FIX。按实际行为归类，避免把 capture 生命周期与会话历史 owner 混同。 | M07 改指向真实 chat/session 来源；`src/transcripts` 分配 M25/M26 并关联 M23，M07 仅消费结果；主 Spec 的泛化同类表述同步修正。 |
| OCRM-REV-005 / P2 | map M21 按名字把 `src/flows` 当 project/board workflow；实际 source 是 channel/provider setup、memory/search setup 和 doctor/repair。 | FIX。配置/维护流程不能被错误分配到 board domain。 | 删除 M21 的该错误归属；按具体 setup/doctor 职责关联 M02/M03/M06/M14/M20/M28/M29/M30；主 Spec 范围与 AC 同步明确全部 flows 仍在范围。 |
| OCRM-REV-006 / P2 | map M19 schedule 枚举遗漏 `stream`，且未明确 source identity/batch/迟到旧流 fencing；主 Spec 已列 stream，但 map 会误导实施分组。 | FIX。事件驱动 stream 是实际独立调度合同，不能被 timer 或普通 webhook 替代。 | 补 `stream`、batch/match、disable/remove/replace/restart 的 source identity 与 ABA/stale fencing；聚焦复审核对 pinned source。 |

## 3. 来源审读的具体证据

- `src/transcripts/provider-types.ts`：blob `66fbe922780368a65337e24271a0b7043127c80e`；`store-types.ts`：blob `f2757f6e71fedad435090e7f1424363974e0f0e4`。[provider 合同](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/transcripts/provider-types.ts)、[store 合同](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/transcripts/store-types.ts)。
- `src/flows/channel-setup.ts`：blob `69c149c85907ccb6463dccd055e16031734b2880`；`provider-flow.ts`：blob `c12d8302a7533e487365879a1231acd47bbc87e5`；`doctor-health.ts`：blob `9cd52c4f2ca6ea9425917d76fcfcd12f458a7ab4`。[channel setup](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/flows/channel-setup.ts)、[provider setup](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/flows/provider-flow.ts)、[doctor](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/flows/doctor-health.ts)。
- `src/cron/schedule.ts`：blob `b0e8e7d25211b01204330abd0c5765b355c4582e`；`types.ts`：blob `0af3dd01381ad3a8e6cbb13bb48110ffcdfd8a65`。[schedule](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/cron/schedule.ts)、[stream identity 类型](https://github.com/openclaw/openclaw/blob/e8cde7518c558f1b4e44cb8139a149289b9719ad/src/cron/types.ts)。

## 4. 复审与交付结果

第一次聚焦复审固定于 candidate tree `32da105af1df1bd0912006e78796e01190fef3f4`。主 Spec blob 为 `d27b56aca4a23185f5e70a0ef0d7f4a1ec028084`，SOT 为 `4b9bdfe745da026ca7ebbca9ea82127d42017f0f`，P0 为 `9757e8cc1c8968c10f50f240d799854ec47357d5`，module map 为 `a4c4282c0fc2c98bb4102111cb21845b59d68123`；AGENTS/README 没有再改动。

- General reviewer 结论：REV-001/002/003 均充分修复；修复及直接依赖未发现提交阻断问题或直接回归。
- Source reviewer 结论：REV-004/005 与 map 的类型合同修复充分；REV-006 的 stream 范围遗漏已修复，但持久 source identity 与 child restart 被并列为身份失效条件，仍需澄清。

**REV-006 聚焦复审残留的决定：FIX，在第二次修改前记录。** 固定 `src/cron/types.ts` 明确 `streamSourceIdentity` 跨 child restart 保留，仅在 disable/remove/replace 原子轮换。M19/C08 必须区分持久 source identity 与 child/process lifetime：前者跨同一 schedule 的重启保留，后者阻止旧实例迟到回调；不能以重启时无条件轮换 source identity 代替实例隔离。此决定细化初次修复说明，未降低 stream/恢复/ABA 合同。

第二次修复只修改 M19 与 C08 的上述规则。最终 semantic candidate tree 为 `37a4a99d8641f783a0c8aa20445112bf03fbcbff`；主 Spec blob 为 `52a0202af8e1ea10e42f8ff8f5913e8ec93a4776`，module map blob 为 `25e4e8258ac45cf72eea704fdceedd0820622a51`；其余语义文件与第一次复审相同。本记录只追加真实审读结论，不修改受审规范。

原 source reviewer 已实际重读 M19 与 C08，确认 source identity 与 child lifetime 区分正确，REV-006 修复充分、无残留实质发现。结合第一次聚焦复审，REV-001 至 REV-006 全部完成文档修正与对应独立确认。

| 结果 | 本次结论 |
| --- | --- |
| General 规范修复复审 | REV-001/002/003 修复充分，未发现相关提交阻断问题或直接回归 |
| Source 事实与直接依赖复审 | REV-004/005/006 修复充分，无残留实质发现 |
| 改动范围 | 主 Spec、项目入口、模块映射、P0 task、README/AGENTS 导航和本审读记录；不修改产品实现或其他项目台账 |
| 本次可执行验证 | 未运行本地或其他环境构建、测试、生成器、validator；只读来源检查、文档编辑、独立审读与 Git/API 操作 |
| 迁移与产品验收 | P0 执行、全文件理解、Rust 实现、生产接线和发布验收仍未完成；全部 OCRM 完成 flags 为 false |

文档审读结论不等于 Rust 迁移、Actions 验证或 packaged release acceptance。后续 gate 的执行与最终受验 target 证据依主 Spec，不能用本记录代替。
