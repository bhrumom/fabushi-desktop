# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-04
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 3
Parent: FBCP-001 Revision 8
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Complete: false

> 本文件按 FMEP-REPORT-01 维护：顶部总览由脚本从 ledger 生成；"当前批次" ≤ 40 行；历史轮次叙述不在此文件中。

## 进度总览

_自动生成：`scripts/tdrp-progress-summary.py`，ledger U=`f985723b552b`_

| 类别 | 行数 | understood | mapped | implemented | verified | release gate passed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 产品职责（Fabushi owner） | 19 | 0 | 6 | 13 | 0 | 0 |
| 领域库实现、桌面未接线（FMEP-STATE-02） | 3 | 0 | 3 | 0 | 0 | 0 |
| 旧栈职责→Fabushi 替代方案（E4，仅设计级 mapped） | 66 | 0 | 66 | 0 | 0 | 0 |
| 旧 Telegram 栈（FMEP-EXIT-01，不计产品） | 2 | 0 | 2 | 0 | 0 | 0 |
| 构建/仓库治理支撑 | 9 | 0 | 8 | 1 | 0 | 0 |
| 上游开发工具（FMEP-TOOL-01，已处置） | 88 | 0 | 71 | 17 | 0 | 0 |
| **合计** | 187 | | | | | |

来源身份 16381；unknown=未统计，unread=未统计，omitted=0；未映射职责 2；source_reviews 16（complete 6 / pending 10）。
分母（全部适用产品职责）尚未完整分解，按 FMEP-STATE-01 不报告总百分比。

## P0 阻塞（FMEP-BLOCK-01 / §14）

| # | 阻塞项 | 解除条件 | 责任人 |
| --- | --- | --- | --- |
| 1 | 后端 PR137 已审查、补修复并合并（后端 main `4e0878d1`，`human-messaging-contracts` 通过）；**尚未部署** | 经显式批准后按顺序执行：先对生产 D1 跑一次 `20261031_human_message_mutations.sql`（不可重跑），再 dispatch `production-worker-delivery`，然后做真实服务 afterId=0 多页/拒绝负例检查 | 需用户批准部署 |
| 2 | 同一消息语义有两个写入者：桌面 Session/Transcript 与移动端 `MessagingService` | **已决并完成 W1 接线**：[FBCP-ADR-004](../../docs/adr/FBCP-ADR-004-single-message-owner.md)（Session 为唯一状态 owner，`fabushi_messaging_core` 为纯领域策略库）。已完成 `authorize_destination` 纯函数提取并接入桌面 `ProductionSessionWorkers`（`TDRP-R9-SHARE-DESTINATION-POLICY-001` → implemented）；剩余 3 项 Share 细粒度协议扩展待后续轮次 | 迁移线 |
| 3 | ~~旧 Telegram 栈退出：E1–E4~~ **已完成**：删除 `mahayana-telegram` 与 `providers/telegram-{runtime,network,protocol,media,wasm}`，仅保留 `telegram-storage`/`telegram-core` 供 E3 只读退役迁移；69 行旧栈职责完成真实替代符号与规范锚点（`docs/specs/fmep-exit-mtproto-disposition.md`）重映射 | 待 E3 退役窗口结束后删除 `telegram-storage`/`telegram-core` | — |
| 4 | ~~Round8/Round9 修复待在包含修复的 H 上执行~~ **已解决**：Round9 @ `c29c22144` 三套核心工作流全绿（`Native service contracts` run `41755914496` 8/8 jobs passed；`TDRP source review contract` run `41755956791` passed；`Mahayana workspace compile guard` run `41755956794` passed）。证据：`telegram-desktop-rust/evidence/round9-requalification-20261104.json` | 之后的 H 仍需各自的 Actions 结果 | — |
| 5 | 来源闭合：unknown/unread 未统计；source_reviews 只有 6/16 complete | 按 FMEP-SOURCE-01 完成身份对照和必需深读 | 迁移线 |

## 当前批次 — FMEP-001 r3 全面收口与账本规范化（2026-11-04）

- **架构与旧栈退出（E1–E4）**：移除 `mahayana-ffi` 对 `fabushi-telegram-runtime` 的依赖（E1）；删除 `mahayana-telegram`（E2）及 `providers/telegram-{runtime,network,protocol,media,wasm}`（E4），仅保留只读退役迁移所需的 `telegram-storage`/`telegram-core`（E3：`docs/specs/fmep-exit-legacy-telegram-data.md`）；建立 `docs/specs/fmep-exit-mtproto-disposition.md` 并将全部旧栈账本行锚定到真实替代代码符号与规范小节。
- **单一消息 Owner 接线（ADR-004 W1）**：在 `native/mahayana-messaging` 提取无状态纯策略 API（`authorize_destination`、`resolve_send_menu_details_from_inputs`、`resolve_paid_restriction_from_capabilities`），并在桌面 `ProductionSessionWorkers`（`source/host/src/extensions/session/production.rs`）转发批次投递与收件人搜索中直接调用 `fabushi_messaging_core::authorize_destination`，推进 `TDRP-R9-SHARE-DESTINATION-POLICY-001` 为 `implemented`。
- **全量编译守卫与 CI 卫生**：修复 `mahayana-web` 与 `mahayana-native-engine` 历史编译断层，新增 `.github/workflows/mahayana-workspace-compile-guard.yml`；清理 `.github/` 下 27 个一次性历史 `round*`/`checkpoint-*` 工作流与脚本文件；将 `native-service-contracts.yml` 中脆弱的硬编码测试条数正则统一升级为 `[1-9][0-9]* passed; 0 failed;`（保留全部具名测试断言）。
- **账本全量符号归一化与 `FBCP-NMR-001` 拆解**：修复全部 32 行历史 `fabushi_target_paths`/`fabushi_target_symbols` 伪修饰符与跨仓前缀，在 `tdrp-source-review-contract.yml` 启用逐行本地路径、字面量符号与单 owner 强校验（187 行 0 错误）；将原 252 KB 的 `FBCP-NMR-001` 拆解为 4 个内聚产品职责行（`FBCP-NMR-001`、`FBCP-NMR-FORWARD-PANEL-001`、`FBCP-NMR-SPARSE-HISTORY-001`、`FBCP-NMR-HISTORY-SEARCH-001`），历史流水归档至 `evidence/fbcp-nmr-001-history-through-20261103.md`。

## 历史记录

2026-11-03 及以前的全部轮次叙述原样移至
[`evidence/status-history-through-20261103.md`](evidence/status-history-through-20261103.md)。
其中的 run/H 结果仍只对应各自的 exact H，不因移动而改变含义。
