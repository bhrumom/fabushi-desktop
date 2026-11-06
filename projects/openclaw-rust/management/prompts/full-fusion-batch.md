# OpenClaw → Fabushi 全量融合执行入口（Revision 3 / OCRM-EXEC-03 + OCRM-MODULAR-01）

先读根 [`AGENTS.md`](../../../AGENTS.md)、[`projects/openclaw-rust/SOURCE_OF_TRUTH.md`](../../SOURCE_OF_TRUTH.md)、主规范 [`docs/specs/openclaw-rust-equivalence-migration.md`](../../../docs/specs/openclaw-rust-equivalence-migration.md) Revision 3（`§17`、`§18 OCRM-EXEC-03` 与 `§18.1 OCRM-MODULAR-01`）以及当前批次相关生产源码。

1. **锁定上游与分支唯一性（`OCRM-LOCK-01`）**：
   - 严格使用已锁定的 `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`（tree `90c8aab39972da11cfad220361781d49e1bc6700`，`accepted_for_this_implementation_round = true`）。
   - **严禁在功能实现与生产接线批次中轮询上游 `main` 或新建 `upstream-*` 漂移探测报告**。
   - OpenClaw 融合在独立工作树 `fabushi-ocrm-integrate`（分支 `integrate/openclaw-fusion-w1`）推进，严禁触碰并行 Telegram 工作树。
2. **Phase 0 五问侦察与信任边界表（Reconnaissance Gate）**：
   - 在动手写任何代码前，针对当前纵向切片完成五问侦察：
     1. 上游入口、状态机与不变量是什么？
     2. Fabushi 现有 canonical owner 是哪个（`source/host` / `source/runner` / `source/node-agent-coordinator` / `source/memory`）？
     3. 当前为何处于 `implemented` + `not-wired`（或 `not-started`）？具体断在 `entrypoint -> producer -> owner -> storage/service -> consumer` 的哪一环？
     4. 跨组件信任边界、权限/沙箱门控与锁顺序是什么（输出 `Trust Boundary Table`）？
     5. 错误码、取消信号、超时代价与重启恢复如何闭环？
3. **纵向生产接线切片优先（`OC-S1`..`OC-S5`）**：
   - 围绕当前 **141 项 `implemented` + `not-wired`** 职责与 **9 项 `not-started`** 职责，按 `OC-S1`（Gateway 控制面）、`OC-S2`（Agent/Session/ACP/模型定价）、`OC-S3`（多渠道入站/出站与 Webhook）、`OC-S4`（Plugin SDK/Skills/安全沙箱/SSRF）、`OC-S5`（Cron/Subagent/Memory QMD/Media/Node Host）逐批推进端到端生产接线。
   - 实现、宿主生产接线（`source/host/app/src/main.rs`、`http_api`、`ProductionSessionWorkers` 等）与契约测试必须在同一批次闭环交付。
4. **模块化解耦与反多合一压缩红线（`OCRM-MODULAR-01`）**：
   - **`Same Owner != Same File`（共享状态 Owner 与物理文件解耦）**：复用现有 canonical owner（`ProductionSessionWorkers`、`SandAgentDb`、`HostApp`、`ChannelRuntime` 等）**绝不直接向 `production.rs`、`agent_db.rs`、`source/host/app/src/main.rs` 堆叠代码**。必须在独立子模块文件（`session/ocrm_*.rs`、`app/src/ocrm_handlers/*.rs` 等）中通过 Rust `impl OwnerStruct` 分块实现，确保根 Owner 文件整洁，与 Telegram（`tdrp_*.rs`）等并行开发分支零冲突。
   - **`Cohesive File Size Budget`（单文件内聚与体积预算）**：领域模块代码量以 **200–600 行** 为内聚目标，严格上限 **800–1,000 行**。超过 **1,200 行** 的多能力巨石文件（如 `capability_catalog.rs`、`media_understanding_service.rs`、`device_pairing_owner.rs`、`agent_db.rs`、`production.rs`、`main.rs` 等）必须重构解耦为目录化模块（`<domain>/mod.rs` + `<domain>/<sub_capability>.rs`）。
   - **`Anti-N-to-1 Stub Compression`（反多合一空壳压缩）**：严禁将 **>10 个** 独立上游产品源码文件映射到单一 **<150 行** 空壳 target 文件（典型如 96 个上游文件映射到 101 行 `command_dispatch.rs` 或 40 个映射到 73 行 `session_turns_controller.rs`）。多文件上游子系统必须按职责拆分为具名 Rust 目录化子模块，完整补齐状态机流转与生产接线。
5. **账本卫生与两阶段验证（`OCRM-HYGIENE-01` + Stage A / Stage B）**：
   - **Stage A（本地快速反馈）**：每次修改后在本地运行：
     - `python3 projects/openclaw-rust/scripts/verify_audit_integrity.py`
     - `python3 projects/openclaw-rust/scripts/ocrm-progress-summary.py`
     - `cargo test --manifest-path source/host/Cargo.toml --test <contract_test>`
   - **Stage B（GitHub Actions 权威验印）**：切片闭环后推送分支并在 GitHub Actions 运行权威门禁，凭真实 artifact receipt 晋升 `verification` / `acceptance` 状态。
