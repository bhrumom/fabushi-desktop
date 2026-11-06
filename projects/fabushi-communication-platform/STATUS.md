# FBCP Native Communication Absorption — Status

Date: 2026-11-04
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 4
Complete: false

> 按 FMEP-REPORT-01，本文件不复制 TDRP 轮次正文。逐批细节见
> [`../telegram-desktop-rust/STATUS.md`](../telegram-desktop-rust/STATUS.md)。

## 进度总览

_自动生成：`scripts/tdrp-progress-summary.py`，ledger U=`f985723b552b`_

| 类别 | 行数 | understood | mapped | implemented | verified | release gate passed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 产品职责（Fabushi owner） | 23 | 0 | 6 | 17 | 0 | 0 |
| 领域库实现、桌面未接线（FMEP-STATE-02） | 0 | 0 | 0 | 0 | 0 | 0 |
| 旧栈职责→Fabushi 替代方案（E4，仅设计级 mapped） | 66 | 0 | 66 | 0 | 0 | 0 |
| 旧 Telegram 栈（FMEP-EXIT-01，不计产品） | 2 | 0 | 2 | 0 | 0 | 0 |
| 构建/仓库治理支撑 | 9 | 0 | 8 | 1 | 0 | 0 |
| 上游开发工具（FMEP-TOOL-01，已处置） | 88 | 0 | 71 | 17 | 0 | 0 |
| **合计** | 188 | | | | | |

来源身份 16381；unknown=未统计，unread=未统计，omitted=0；未映射职责 2；source_reviews 16（complete 6 / pending 10）。
分母（全部适用产品职责）尚未完整分解，按 FMEP-STATE-01 不报告总百分比。

## P0 阻塞（FMEP-BLOCK-01 / §14）

| # | 阻塞项 | 解除条件 | 责任人 |
| --- | --- | --- | --- |
| 1 | **后端 PR137 D1 迁移与部署**：PR #137 已合并（后端 main `4e0878d1`，`human-messaging-contracts` 39/39 绿）；生产 D1 迁移与 Worker 部署尚未执行 | 经显式批准后按序执行：1) 对生产 D1 跑一次 `20261031_human_message_mutations.sql`（不可重跑）；2) dispatch `production-worker-delivery`；3) 验证 `afterId=0` 分页与变更流 | 需用户批准部署 |
| 2 | 后端缺乏 Group / Channel / Forum Topic 服务端线协议与 D1 存储表（仅有 1:1 私聊） | 服务端实现群组/频道资源与多人群消息端点；客户端按 [FBCP-ADR-005](../../docs/adr/FBCP-ADR-005-session-group-topic-conversations.md) 先落地桌面 SQLite Schema 与无状态策略接入，远端同步保持 blocked | 后端服务团队 / 迁移线 |

## 当前批次

与 TDRP 当前批次相同（Wave 1~3: 集成至 `integrate/telegram-fusion-w1`，实际 owner: `ProductionSessionWorkers` + `SandAgentDb` + `miniapp-window.ts`；Actions 验证运行 ID: `41827958018`、`41827968182`、`41828142297`、`41860436085`、`41860446111`、`41860459492`），详见 [`../telegram-desktop-rust/STATUS.md`](../telegram-desktop-rust/STATUS.md)。

## Do not report

- Telegram migration percentage as product progress
- Telegram Provider / legacy Telegram stack progress (FMEP-EXIT-01)
- upstream development tooling rows as product progress (FMEP-TOOL-01)
- domain-library code not consumed by the desktop shipping path as implemented (FMEP-STATE-02)
- a new Communication Core as progress
- file-count parity
- source reading as implementation
- local-feedback runs as passes (FMEP-FAST-09)

## Progress dimensions

1. existing-owner inventory
2. Telegram research coverage
3. absorption mapping
4. model evolution designed
5. native network designed
6. native network implemented
7. existing owners production-wired
8. Human + Agent unified UX
9. full capability acceptance
10. packaged/release acceptance

## 历史记录

2026-11-03 及以前的全部轮次叙述原样移至
[`management/status-history-through-20261103.md`](management/status-history-through-20261103.md)。
