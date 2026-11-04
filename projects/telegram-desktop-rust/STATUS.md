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

## 当前批次 — FMEP-001 r3 迁移纠偏（2026-11-04）

- **用户行为**：无产品 UI 变化。纠正投入方向与状态口径。
- **代码**：`mahayana-ffi` 移除 `fabushi-telegram-runtime` 依赖和 `fabushi_telegram_*` 强制链接（FMEP-EXIT-01 E1）；`Cargo.lock` 同步更新。桌面 AppHost/Host/CLI 的 normal 依赖图本来就不含旧栈，现有 CI 负例守卫保留。
- **账本**：25 行旧栈 implemented → mapped（`[FMEP-EXIT-01 legacy-only]`）；4 行只经桌面被拒的 `MessagingService` 的 implemented → mapped（`[FMEP-STATE-02 domain-only wiring gap]`）；88 行上游开发工具标注 `[FMEP-TOOL-01 upstream-development-tooling]`（状态不变，不计产品）；`rebaseline_candidate` 的 review 计数改为由数据推导（6 complete / 10 pending）。
- **仓库**：删除与 `STATUS.md` 大小写冲突、内容过时的 `status.md`（两处）。
- **检查（local-feedback，不是证据）**：`cargo check --locked -p mahayana-ffi` 通过；ledger 对 `contracts/parity-ledger.schema.json` 的 Draft 2020-12 校验 0 错误。Actions：not-run。
- **续（同日）**：[FBCP-ADR-004](../../docs/adr/FBCP-ADR-004-single-message-owner.md) 已决（Session 为唯一状态 owner，`fabushi_messaging_core` 为纯策略库）；E2 删除 `mahayana-telegram`（`cargo check --locked -p mahayana-app-host --tests` local-feedback 通过），3 行已被硬禁用的 Telegram 账号行和 2 行依赖该 crate 的行改为 mapped；E3 旧数据规范 `docs/specs/fmep-exit-legacy-telegram-data.md`；后端 PR137 安全/迁移审查无阻塞，已补 400/无 body DELETE/发布顺序修复（后端 `ec37941`）。
- **下一批**：ADR-004 提取+Session 接线批次（进行中）→ E4 重映射与删除 provider crate → Round8 在包含修复的 H 上重跑 → PR137 合并后按规范顺序部署（需显式批准）。

## 历史记录

2026-11-03 及以前的全部轮次叙述原样移至
[`evidence/status-history-through-20261103.md`](evidence/status-history-through-20261103.md)。
其中的 run/H 结果仍只对应各自的 exact H，不因移动而改变含义。
