# Telegram Desktop → Fabushi 全量等价重写 — Status

Date: 2026-11-04
Spec: TDRP-001 Revision 10 / FMEP-001 Revision 4
Parent: FBCP-001 Revision 8
Fixed U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`
Complete: false

> 本文件按 FMEP-REPORT-01 维护：顶部总览由脚本从 ledger 生成；"当前批次" ≤ 40 行；历史轮次叙述不在此文件中。

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
| 1 | **后端 PR137 D1 迁移与部署**：PR #137 已合并（后端 main `4e0878d1`，`human-messaging-contracts` 39/39 绿）；生产 D1 迁移与 Worker 部署尚未执行 | 经显式批准后按序执行：1) 对生产 D1 执行一次 `20261031_human_message_mutations.sql`（不可重跑）；2) dispatch `production-worker-delivery`；3) 验证 `afterId=0` 分页与变更流 | 需用户批准部署 |

## 当前批次 — Wave 1~3 全量接线闭环与证据同步（2026-11-04）

- **集成分支与生产 Owner**：集成至 `integrate/telegram-fusion-w1`；状态唯一权威归属 `ProductionSessionWorkers` + `SandAgentDb` + `miniapp-window.ts`，领域策略纯函数无状态复用（`fabushi_messaging_core`）。
- **Actions 验证运行**：Wave 1 CI 运行 `41827958018`（Service contracts）、`41827968182`（Review contract）、`41828142297`（Compile guard）；Wave 2 CI 运行 `41860436085`（Service contracts）、`41860446111`（Review contract）、`41860459492`（Compile guard）全部全绿通过。
- **用户行为与切片推进**：
  - **Wave 1（切片收口与批量裁决）**：`W1-A` (S1a) 落地带附言转发，Session 原子投递 `{clientNonce}:comment` 独立文本；`W1-B` (S5) 打通 Telegram Mini App / Bot WebApp 容器与会话桥接，Host 权威签发 `launchContext` 并防会话注入；`W1-C` 完成 `FMEP-BULK-01` 9,486 个子模块批量分类裁决。
  - **Wave 2（多人会话架构与实时策略）**：`W2-ADR005` 确立群组/频道/Topic 架构；`W2-A` 升级 SQLite Schema（user_version=2）并支持 Topic 消息分页；`W2-B` 提取发言权限与 Topic 准入纯策略；`W2-C` 落地群组/频道创建与 Topic 转发；`W2-D` 实现 Typing 3s 节流与 Topic 免打扰静音；`W2-E` 绑定前端 Topic 选择器与 `topicId` 投影；`W2-F` 落地媒体缓存配额清理与附件预检；`W2-G` 完成 Session 宿主接线并推进 S5 为 implemented。
  - **Wave 3（纵向切片闭环与退役迁移）**：`W3-A` 闭合 ShareLink/S1 协议接线；`W3-B` 闭合 S2 权限与 S3 Topic 工作区交互；`W3-C` 扩展 S4 媒体缓存与 S5 Mini App 语音/数据桥；`W3-D` 归档 E3 旧 Telegram 数据只读退役。
- **缺口与下一批**：等待后端 PR #137 生产 D1 迁移与部署批准，进入 S6 真实线上验收。

## 历史记录

2026-11-03 及以前的全部轮次叙述原样移至
[`evidence/status-history-through-20261103.md`](evidence/status-history-through-20261103.md)。
其中的 run/H 结果仍只对应各自的 exact H，不因移动而改变含义。
