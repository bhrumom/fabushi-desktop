# OpenClaw 全源码 Rust 迁移 — Source of Truth

> 执行修订（2026-11-04）：主 Spec Revision 3（`§17 OCRM-EXEC-02`、`§18 OCRM-EXEC-03` 与 `§18.1 OCRM-MODULAR-01`）优先于历史漂移快照。本轮实现严格锁定上游基线 `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`（tree `90c8aab39972da11cfad220361781d49e1bc6700`），禁止在功能实施与生产接线批次中轮询上游浮动 `main`。所有实现严格遵守模块化解耦与反空壳压缩红线。

Status: active  
Project ID: OCRM-001  
Revision: 3  
Last updated: 2026-11-04（Asia/Shanghai）  
Target: `bhrumom/fabushi-desktop` current canonical main (`integrate/openclaw-fusion-w1`)  
Implementation/acceptance: **进行中（230/239 职责已实现 Rust 代码，89/239 已完成生产接线，141 项待生产接线，9 项待启动）**

## 1. 唯一目标

逐文件、逐模块、逐责任理解 `openclaw/openclaw` 的完整源码与必要依赖，将全部产品功能使用 Rust 等价实现到现有 Fabushi 架构，保留现有 Fabushi Bot 的全部能力，交付一个完整、统一、可发布、可维护的 Fabushi Desktop。

本项目必须完成 source completeness、全部产品责任实现、真实生产接线、UI/UX、服务/插件/渠道/模型/节点生态、数据迁移/平台交付以及独立验收。不能以源码盘点、目录映射、概念架构、同名 API、原 OpenClaw wrapper、UI demo、少数模块或单条 end-to-end 路径代替全量完成。

## 2. 权威文件与适用顺序

1. 最新明确用户要求与根 [`AGENTS.md`](../../AGENTS.md)。
2. 主规范 [`docs/specs/openclaw-rust-equivalence-migration.md`](../../docs/specs/openclaw-rust-equivalence-migration.md)（OCRM-001 Revision 3，含 `§18.1 OCRM-MODULAR-01`）与批量裁决规范 [`docs/specs/ocrm-bulk-subtrees-disposition.md`](../../docs/specs/ocrm-bulk-subtrees-disposition.md)（OCRM-BULK-01）。
3. 当前 canonical Fabushi 真实源码/接线，以及相关 Bot、FBCP、TDRP、插件、Mini App、设备网络领域合同。
4. 本项目 [`module-map.md`](module-map.md)、[`upstream.lock.json`](upstream.lock.json)、[`inventory/index.json`](inventory/index.json)、[`parity-ledger.json`](parity-ledger.json)、[`requirements-traceability-matrix.json`](requirements-traceability-matrix.json) 及 `inventory/shards/manifest.json`。
5. 自动化校验与进度汇总脚本：
   - `python3 projects/openclaw-rust/scripts/verify_audit_integrity.py`
   - `python3 projects/openclaw-rust/scripts/ocrm-progress-summary.py`
   - `python3 projects/openclaw-rust/scripts/ocrm-bulk-disposition.py`

## 3. 不可弱化的约束

- 全源码范围登记与风险分层审查：62,130 个跟踪对象 100% 纳入 `inventory/shards/part-0000..0060.jsonl.gz`；41,834 个非产品路径按 `OCRM-BULK-01` 成组裁决（`grouped-evidence`），承载产品行为的路径逐文件/状态机/副作用深读并映射至 `parity-ledger.json`。
- 所有迁入业务/策略/协议/调度/状态/持久化/执行控制由 Rust 承担；React/Electron 保留现有产品表现与薄桥接。
- 先读当前 existing owners（Phase 0 五问侦察 + `Trust Boundary Table`）；只有无合理承接者时通过 rejected-owner analysis 与 ADR 新增最小 source-neutral owner。
- Coordinator、Host、Runner、box-exec、Electron、renderer、服务端与设备边界保留；无第二 Agent loop、Conversation/Transcript store、scheduler、Marketplace、权限真相或 OpenClaw 运行时。
- `enumerated`、`applicability-assessed`、`dispositioned`、`read`、`mapped`、`implemented`、`wired`、`verified`、`accepted` 状态正交独立；单独 helper/测试通过不等于 `wired`。
- **模块化解耦与反空壳压缩红线（`OCRM-MODULAR-01`）**：`Same Owner != Same File`（共享状态 Owner 扩展必须物理拆分为 `ocrm_*.rs` 子模块，严禁污染 `production.rs` / `agent_db.rs` / `main.rs` 根文件）；单文件代码行数控制在 200–600 行（上限 800–1,000 行，>1,200 行必须解耦为 `<domain>/mod.rs` 目录模块）；严禁将 >10 个上游源文件机械压缩为 <150 行空壳 stub。

## 4. 锁定上游基线与机械进度快照（2026-11-04）

| 维度 | 指标 | 数值 | 占比 / 状态 |
| --- | --- | ---: | --- |
| **上游锁定基线** | `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845` | 62,130 项 | tree `90c8aab39972da11cfad220361781d49e1bc6700`, `accepted_for_this_implementation_round = true` |
| **Inventory 核算** | 已完成适用性判定 (`applicability_assessed`) | 62,130 / 62,130 | 100.00% (`omitted_known = 0`) |
| **Inventory 处置** | 已映射深读路径 (`deep-read`) | 817 | 1.31% |
| **Inventory 处置** | 非产品成组裁决 (`grouped-evidence`) | 41,834 | 67.33% (`OCRM-BULK-01`) |
| **Inventory 处置** | 已完成处置合计 (`dispositioned`) | 42,651 / 62,130 | **68.65%** |
| **Inventory 前沿** | 待深读产品源码前沿 (`deep-read-required`) | 19,479 | 31.35% (`src/` + `extensions/`) |
| **Parity Ledger** | 已登记职责总数 (`responsibilities`) | 239 | `reviewed=180`, `mapped=59` |
| **代码落地状态** | `implementation = implemented` | 230 / 239 | `not-started=9` |
| **宿主生产接线** | `wiring = wired` | 89 / 239 | **`implemented + not-wired = 141`**（Wave 1–2 主战场） |
| **契约验证状态** | `verification` | 239 | `verified=8`, `not-run=231` |
| **RTM 门禁矩阵** | `OCRM-R01..R24` + `OCRM-AC01..AC24` | 48 | `in-progress=31`, `not-started=17` |
| **模块解耦红线** | `OCRM-MODULAR-01` 巨石文件 (`> 1,200` 行) | 13 处 | 待解耦目录化模块（Wave 1 目标降至 0） |
| **模块解耦红线** | `OCRM-MODULAR-01` 多合一空壳压缩 (`>= 15` 源映射但 `< 150` 行) | 3 处 | 待拆分子模块补齐状态机（Wave 1 目标降至 0） |

## 5. 全局完成标志（Completion Flags）

各 flag 的适用集合、true 谓词、负责 gate 和失效条件以主 Spec §15.1 为唯一规范：

| 标志 | 当前值 |
| --- | --- |
| `baseline_identity_accepted` | `false`（待 exact-head Actions 权威验印） |
| `source_closure_ready` | `false` |
| `responsibility_mapping_complete` | `false` |
| `rust_implementation_complete` | `false` |
| `production_wiring_complete` | `false` |
| `ecosystem_compatibility_accepted` | `false` |
| `ui_accepted` | `false` |
| `release_accepted` | `false` |

## 6. 纵向生产接线切片路线图（Wave 1–2：`OC-S1`..`OC-S5`）

针对当前 **141 项 `implemented` + `not-wired`** 职责与 **9 项 `not-started`** 职责，后续批次严格按以下 5 个纵向切片闭合生产调用链（`entrypoint -> producer -> owner -> storage/service -> consumer`）：

1. **Slice `OC-S1` — Gateway WS / HTTP / RPC Control Plane Wiring**：
   - 将已实现的 Gateway WS 协议鉴权、OpenResponses HTTP、Chat/Node/Session 方法及 Webhook 限流预算接入 `source/host/app/src/main.rs` 与 `source/host/src/http_api/mod.rs` 真实路由。
2. **Slice `OC-S2` — Agent Turn / Context Compaction / Model Pricing & ACP Closure**：
   - 闭合 9 项 `not-started` 核心职责（`OCRM-RESP-AGENT-CONTEXT-COMPACTION-001`、`OCRM-RESP-SESSION-WORK-ADMISSION-001`、`OCRM-RESP-SESSION-AUTH-PROFILE-LIFECYCLE-001`、`OCRM-RESP-AGENT-MODEL-PRICING-001`、`OCRM-RESP-AGENT-ACP-COMMANDS-001`、`OCRM-RESP-AGENT-ACP-POLICY-001`、`OCRM-RESP-PLUGINS-BUNDLE-MANIFEST-001` 等）并接入 `ProductionSessionWorkers` 与 `SandAgentDb`。
3. **Slice `OC-S3` — Multi-Channel Inbound/Outbound & Webhook Ingress Wiring**：
   - 将 Discord、Slack、Telegram、Feishu、Line、WhatsApp、Signal/iMessage/Matrix、BlueBubbles、GoogleChat、MSTeams 等渠道适配器及入站策略门控、打字指示器、出站分块投递管线接入宿主 `ChannelRuntime`。
4. **Slice `OC-S4` — Plugin SDK / Skills / Exec Security / Sandbox & SSRF Enforcement**：
   - 将 Plugin SDK 表面、Skills 配置变更、Exec Approvals、命令混淆检测、Docker/SSH 沙箱与 SSRF 策略接入工具执行与插件加载主路径。
5. **Slice `OC-S5` — Cron / Subagent / Memory QMD / Media / Node Host & Companion Closure**：
   - 将 Cron Session Reaper、Subagent Spawn Queue、Memory QMD 混合检索、Media Understanding Runner、TTS/Talk 控制器与 Node Host 重连/Computer 动作派发接入生产后台服务与协调器。

## 7. 模块化解耦与反多合一压缩红线（OCRM-MODULAR-01）

针对模块膨胀与过度压缩两大架构隐患，所有切片开发必须严格执行以下三条红线：

1. **Rule 1: 共享状态 Owner 与物理文件解耦（`Same Owner != Same File`）**：
   - 复用现有规范 Owner（`ProductionSessionWorkers`、`SandAgentDb`、`HostApp`、`ChannelRuntime` 等）**绝不直接向根物理文件追加代码**。
   - 所有 OpenClaw 扩展必须通过 Rust `impl OwnerStruct` 分块实现，安置于独立的内聚领域子模块中（如 `session/ocrm_*.rs`、`app/src/ocrm_handlers/*.rs`）。
   - 保持根 Owner 物理文件整洁，与并行 Telegram（`tdrp_*.rs`）等其他分支零冲突。
2. **Rule 2: 单文件内聚与体积预算红线（`Cohesive File Size Budget`）**：
   - 领域实现文件目标行数为 **200–600 行**，新建模块严格上限为 **800–1,000 行**。
   - 超出 **1,200 行** 的多能力巨石文件（如 `capability_catalog.rs`、`media_understanding_service.rs`、`device_pairing_owner.rs`、`agent_db.rs`、`production.rs`、`main.rs` 等当前共 13 处）必须解耦拆分为目录化模块（`<domain>/mod.rs` + `<domain>/<sub_capability>.rs`）。
3. **Rule 3: 禁止多合一空壳压缩（`Anti-N-to-1 Stub Compression`）**：
   - 严禁将 **>10 个** 独立上游产品源码文件映射到单一 **<150 行** 的空壳 target 文件（当前存量 3 处：`command_dispatch.rs` 96 源 101 行、`session_turns_controller.rs` 40 源 73 行、`state_store.rs` 19 源 126 行）。
   - 多文件上游子系统必须拆分为具名子模块，完整补齐状态机、事件流转与生产接线，并在 Wave 1 迭代周期内将空壳压缩与巨石文件指标全部归零。
