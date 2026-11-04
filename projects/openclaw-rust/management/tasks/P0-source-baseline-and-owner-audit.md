# OCRM P0 — 全量 source baseline 与 existing-owner 审计

> 执行修订（2026-11-04）：主 Spec Revision 3（`§17 OCRM-EXEC-02` 与 `§18 OCRM-EXEC-03`）优先于历史漂移快照。当前实施轮次严格固定上游基线 `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`（tree `90c8aab39972da11cfad220361781d49e1bc6700`，共 62,130 个跟踪对象），禁止在功能实施与生产接线批次中轮询上游浮动 `main`。

Status: in-progress（Wave 0 治理基线与非产品批量裁决已完成；待 exact-head Actions 权威验印与纵向切片闭合）  
Task ID: OCRM-P0  
Spec: [OCRM-001](../../../../docs/specs/openclaw-rust-equivalence-migration.md)  
Project: [Source of Truth](../../SOURCE_OF_TRUTH.md)  
Last updated: 2026-11-04（Asia/Shanghai）

## 1. 本任务要交付什么

建立可以驱动全部后续 Rust 等价实现的真实 source authority、无遗漏 inventory、逐责任 ledger/RTM、当前 owner 解析和纵向切片实施合同。任务必须能回答任一上游文件“读到哪里、承担什么、由谁实现、还缺什么、如何验收”，不能只产生目录清单和 migration proposal。

本任务不声称一轮已完成所有文件语义深读。P0 必须完成全量集合与身份校验，并为每条路径建立适用性与 disposition；对每个纵向实施切片完成逐文件深读、依赖/owner/oracle 合同。承载行为、高风险、公开合同、生产 producer/owner/consumer、权限安全、状态迁移、官方插件/渠道/平台和关键上游 oracle 的路径必须深读；普通文档、示例、fixture、重复或研发用途项按 [`docs/specs/ocrm-bulk-subtrees-disposition.md`](../../../../docs/specs/ocrm-bulk-subtrees-disposition.md)（`OCRM-BULK-01`）成组证据处置。

## 2. 前置读取与并行工作保护

1. 读取根 [`AGENTS.md`](../../../../AGENTS.md)、OCRM Spec/SOT/module map、当前相关 owner 源码和领域合同。
2. 固定本轮实施上游基线 `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`（tree `90c8aab39972da11cfad220361781d49e1bc6700`），`accepted_for_this_implementation_round = true`。
3. 严禁干扰并行 Telegram 融合工作树（`fabushi-tg-integrate`、`fabushi-w2g-s4-host-wiring`）；OpenClaw 融合统一在独立工作树 `fabushi-ocrm-integrate`（分支 `integrate/openclaw-fusion-w1`）推进。

## 3. 任务分解与交付合同

| ID | 工作 | 可复核交付 |
| --- | --- | --- |
| OCRM-P0-01 | 精确来源/目标锁定 (`OCRM-LOCK-01`) | `upstream.lock.json`、`inventory/index.json`、`inventory/shards/manifest.json`、`parity-ledger.json`、`requirements-traceability-matrix.json` 统一锁定 `1440bb1771372334bea23d7642bb52deb7db1845` |
| OCRM-P0-02 | 递归 enumerate + dependency closure | 62,130 个跟踪对象（62,128 blobs、2 symlinks、0 gitlinks）与 252 个声明 manifest 输入完整登记 |
| OCRM-P0-03 | 定义并验证 file/责任/RTM schema | Draft 2020-12 JSON Schema 校验 + `OCRM-HYGIENE-01` 字面符号与路径存在性门禁 |
| OCRM-P0-04 | 非产品子树批量裁决 (`OCRM-BULK-01`) | 41,834 个非产品路径成组裁决（`grouped-evidence`），817 个已映射路径恢复 `read-complete` / `deep-read`，19,479 个产品源码路径标为 `deep-read-required` |
| OCRM-P0-05 | 逐模块依赖和当前 owner resolution | 239 条职责完整绑定 Fabushi 真实目标文件与字面符号 |
| OCRM-P0-08 | GitHub Actions authority/coverage gate | `expand_inventory.py`、`shard_inventory.py`、`validate_p0.py`、`verify_audit_integrity.py` 四处自锁缺陷修复并通过本地与 Actions 双向校验 |
| OCRM-P0-10 | 纵向生产接线切片 (`OC-S1`..`OC-S5`) | 针对 141 项 `implemented` + `not-wired` 与 9 项 `not-started` 职责建立端到端接线批次 |

## 4. 机器产物布局

- [`projects/openclaw-rust/upstream.lock.json`](../../upstream.lock.json)：唯一锁定上游身份。
- [`projects/openclaw-rust/inventory/index.json`](../../inventory/index.json) 与 `inventory/shards/part-0000..0060.jsonl.gz`：62,130 条逐路径集合、类型、适用性、处置与阅读状态。
- [`projects/openclaw-rust/parity-ledger.json`](../../parity-ledger.json)：239 条逐责任 source/owner/implementation/wiring/acceptance。
- [`projects/openclaw-rust/requirements-traceability-matrix.json`](../../requirements-traceability-matrix.json)：48 条 requirement/AC/oracle/test/evidence/verdict。
- [`docs/specs/ocrm-bulk-subtrees-disposition.md`](../../../../docs/specs/ocrm-bulk-subtrees-disposition.md)：非产品子树批量分类裁决规范（`OCRM-BULK-01`）。

## 5. 本任务退出标准

| ID | 退出标准 |
| --- | --- |
| OCRM-P0-AC01 | `upstream.lock.json`、`index.json`、`manifest.json`、`parity-ledger.json`、`requirements-traceability-matrix.json` 身份完全一致，无漂移污染。 |
| OCRM-P0-AC02 | 62,130 条路径集合与依赖输入经校验无截断/过滤，`omitted_known = 0`。 |
| OCRM-P0-AC03 | 每路径已入 inventory 且 `applicability_status` 与 `disposition_status` 诚实闭合（`applicability_assessed = 62,130`，`dispositioned = 42,651`）。 |
| OCRM-P0-AC04 | 下一纵向切片满足主 Spec §11.1 与 §18 Phase 0 五问侦察及 `Trust Boundary Table` 准入。 |
| OCRM-P0-AC05 | `verify_audit_integrity.py` 与 `validate_p0.py` 0 违规通过；所有 `implemented` 职责的目标文件与字面符号真实存在。 |

## 6. 当前状态（2026-11-04）

- **已修复并闭环**：
  1. 统一锁定 `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`（`accepted_for_this_implementation_round = true`），清理 42 份未引用 `upstream-*` 漂移探测报告与 4 份过时漂移 ADR。
  2. 修复 `expand_inventory.py`、`shard_inventory.py`、`validate_p0.py`、`verify_audit_integrity.py` 四处联动自锁缺陷。
  3. 完成 239 条 `parity-ledger.json` 职责的 `OCRM-HYGIENE-01` 规范化（修复 10 处非文件目录/占位路径与 60 处散文符号，修复 `OCRM-RESP-CRON-SESSION-REAPER-001` 源引用结构）。
  4. 执行 `ocrm-bulk-disposition.py` 完成 62,130 条路径的确定性分类（817 `deep-read` + 41,834 `grouped-evidence` + 19,479 `deep-read-required`），`verify_audit_integrity.py` 0 违规通过。
- **下一步**：按 `OC-S1`..`OC-S5` 纵向切片依次将 141 项 `implemented` + `not-wired` 职责与 9 项 `not-started` 职责接入宿主生产主干。
