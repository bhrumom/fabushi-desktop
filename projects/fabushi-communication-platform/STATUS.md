# FBCP Native Communication Absorption — Status

Date: 2026-11-04
Spec: FBCP-001 Revision 8 / TDRP-001 Revision 10 / FMEP-001 Revision 3
Complete: false

> 按 FMEP-REPORT-01，本文件不复制 TDRP 轮次正文。逐批细节见
> [`../telegram-desktop-rust/STATUS.md`](../telegram-desktop-rust/STATUS.md)。

## 进度总览

_自动生成：`scripts/tdrp-progress-summary.py`，ledger U=`f985723b552b`_

| 类别 | 行数 | understood | mapped | implemented | verified | release gate passed |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| 产品职责（Fabushi owner） | 14 | 0 | 5 | 9 | 0 | 0 |
| 领域库实现、桌面未接线（FMEP-STATE-02） | 4 | 0 | 4 | 0 | 0 | 0 |
| 旧 Telegram 栈（FMEP-EXIT-01，不计产品） | 67 | 2 | 65 | 0 | 0 | 0 |
| 构建/仓库治理支撑 | 11 | 0 | 10 | 1 | 0 | 0 |
| 上游开发工具（FMEP-TOOL-01，已处置） | 88 | 0 | 71 | 17 | 0 | 0 |
| **合计** | 184 | | | | | |

来源身份 16381；unknown=未统计，unread=未统计，omitted=0；未映射职责 2；source_reviews 16（complete 6 / pending 10）。
分母（全部适用产品职责）尚未完整分解，按 FMEP-STATE-01 不报告总百分比。

## P0 阻塞（FMEP-BLOCK-01 / §14）

| # | 阻塞项 | 解除条件 | 责任人 |
| --- | --- | --- | --- |
| 1 | 后端 PR137（canonical revisions/tombstones、afterId=0 分页）仍为 draft/未合并；Worker/D1 部署未核验 | PR137 合并到后端 main，部署后用 Actions 核验 Worker 身份与 afterId=0 授权分页 | 待指定（后端） |
| 2 | 同一消息语义有两个写入者：桌面 Session/Transcript 与移动端 `MessagingService` | **已决**：[FBCP-ADR-004](../../docs/adr/FBCP-ADR-004-single-message-owner.md)（Session 为唯一状态 owner，`fabushi_messaging_core` 为纯领域策略库）。剩余：提取纯函数并接入 Session、移动端 snapshot 导入 | 迁移线 |
| 3 | 旧 Telegram 栈退出：E1、E2 已完成，E3 规范已定 | E4：重映射指向 `providers/telegram-*` 的账本行后，删除除 storage/core 外的 provider crate 与专属 CI；storage/core 待 E3 退役版本后删除 | 迁移线 |
| 4 | Round8 修复（UI 209 用例、reconciliation 125 用例、owner 40、actualMac 2+2）尚未在包含修复的 H 上执行 | 在包含修复的 H 上跑 Actions 并全部通过 | 迁移线 |
| 5 | 来源闭合：unknown/unread 未统计；source_reviews 只有 6/16 complete | 按 FMEP-SOURCE-01 完成身份对照和必需深读 | 迁移线 |

## 当前批次

与 TDRP 当前批次相同（FMEP-001 r3 迁移纠偏，2026-11-04），见 TDRP STATUS。

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
