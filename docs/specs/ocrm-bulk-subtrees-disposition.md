# OCRM-BULK-01 — OpenClaw 非产品子树与外围资产批量分类裁决规范

- **Status**: Active（Normative addendum to OCRM-001 Revision 3 §18）
- **Spec ID**: OCRM-BULK-01
- **Applies to**: `projects/openclaw-rust`（`OCRM-001`）
- **Locked Upstream Authority**: `openclaw/openclaw@1440bb1771372334bea23d7642bb52deb7db1845`（tree `90c8aab39972da11cfad220361781d49e1bc6700`，共 **62,130** 个跟踪对象：62,128 blobs、2 symlinks、0 gitlinks）
- **Date**: 2026-11-04（Asia/Shanghai）

## 1. 背景与目标

依据 `OCRM-001` Revision 2 §17 与 Revision 3 §18（对齐 `FMEP-001` Revision 4 §15 `FMEP-BULK-01`），固定上游基线 `1440bb1771372334bea23d7642bb52deb7db1845` 的 **62,130** 个跟踪对象必须 100% 保留精确 `(path, mode, git_type, sha, blob_sha, size_bytes)` 身份核算，但**严禁对非产品运行时文件（测试套件、QA 夹具、上游研发配置、文档示例、构建发布脚本、纯前端/移动端视图壳与静态资源）逐个手写长篇 Dossier**。

本规范建立确定性的子树与文件类型分类规则，由 `projects/openclaw-rust/scripts/ocrm-bulk-disposition.py` 机械执行并写入 `projects/openclaw-rust/inventory/shards/part-*.jsonl.gz`，由 `verify_audit_integrity.py` 与 `validate_p0.py` 执行双向守恒校验。

## 2. 62,130 个上游对象的风险分层与批量处置矩阵

在排除已由 `projects/openclaw-rust/parity-ledger.json` 显式映射并完成深读的 **817** 个源路径后，其余路径按以下 5 类非产品成组处置规则（`applicability_status: "non-product"`, `disposition_status: "grouped-evidence"`，合计 **41,834** 项）与产品待深读前沿（`applicability_status: "product"`, `disposition_status: "deep-read-required"`，合计 **19,479** 项）精确划分：

| 分类 ID / `disposition` 值 | 匹配范围与判定准则 | 对象数量 | 架构处置结论与 Fabushi 替代方案 |
| --- | --- | ---: | --- |
| `test-qa-fixture-suite` | `test/`、`qa/`、`extensions/test-support/`、`src/test-{fixtures,helpers,utils}/`、包含 `/test/`、`/tests/`、`/__tests__/`、`/__mocks__/`、`/fixtures/`、`/mocks/` 的路径，以及 `*.test.*`、`*.spec.*`、`*.e2e.*`、`*.test-*`、`*.snap` | **27,963** | 上游 Vitest/Playwright 测试、QA 实验室夹具与 Mock 支撑文件不进入 Fabushi 生产构建；核心业务状态机与故障边界由 `source/host/tests/*_contract.rs` 与 `source/node-agent-coordinator/tests/*` 的 Rust 契约测试等价验证。 |
| `upstream-ui-and-platform-shell-assets` | `ui/`、`apps/`、`Swabble/`、`assets/`、`vendor/` 以及静态视觉/媒体资源（`.svg`、`.png`、`.jpg`、`.jpeg`、`.gif`、`.webp`、`.ico`、`.icns`、`.woff*`、`.ttf`、`.otf`、`.css`、`.scss`、`.html`、`.mp3`、`.wav`、`.ogg`、`.pdf`） | **6,003** | 上游 Control Web UI 与各平台原生壳由 Fabushi 现有 `frontend/src/production/ProductionRenderer.tsx`、Electron 主进程壳、`source/host`、`source/node-agent-coordinator` 及既有 `fabushi-ios`/`fabushi-android` 容器替代（`presentation-replaced-with-behavior-preserved`）。 |
| `upstream-build-release-ops-scripts` | `scripts/`、`tools/`、`config/`、`deploy/`、`patches/`、`security/`，以及构建/打包与声明配置（`tsconfig*`、`vitest*`、`Dockerfile*`、`docker-compose*`、`Makefile`、`*.gradle*`、`*.podspec`、`*.plist`、`*.entitlements`、`*.pro`、`*.d.ts`、`*.json`、`*.yml`、`*.yaml`、`*.toml` 等） | **4,653** | 上游 Node/pnpm 构建、Docker 部署、补丁与静态扫描脚本由 Fabushi 的 Cargo workspace、`desktop/package.json` 及 `.github/workflows/` CI 门禁替代；不随桌面客户端分发。 |
| `upstream-documentation-examples` | `docs/`、`examples/` 以及技能目录之外的 Markdown 文档（`*.md`、`*.mdx`） | **2,142** | 上游使用文档、架构说明与示例工程仅作为只读研究参考，不构成运行时代码职责。 |
| `upstream-governance-ci-devconfig` | `.agents/`、`.claude/`、`.openclaw/`、`.vscode/`、`.github/`、`.pi/`、`git-hooks/`、`CHANGELOG/`、`skills/`、`packages/` 以及根级研发配置 | **1,073** | 上游 AI 代理提示词、研发工作流与发版日志不取得 Fabushi 仓库治理权威，不创建第二套任务或治理运行时。 |
| **已映射并深读（`mapped-in-ledger`）** | `projects/openclaw-rust/parity-ledger.json` 中 239 条职责的 `source_refs` 精确绑定的上游路径 | **817** | 已完成 `read_status: "read-complete"`、`disposition_status: "deep-read"` 及精确行号区间与 Rust 目标落点映射。 |
| **产品待深读前沿（`product-frontier`）** | `src/` 与 `extensions/` 中尚未映射的非测试产品源码 | **19,479** | 诚实标为 `applicability_status: "product"`、`disposition_status: "deep-read-required"`、`read_status: "unread"`、`responsibility_status: "unclassified"`，按纵向切片批次持续深读与闭合。 |
| **总计（守恒分母）** | `inventory/shards/part-0000.jsonl.gz` .. `part-0060.jsonl.gz` | **62,130** | `omitted_known = 0`，`applicability_assessed = 62,130`（100%），`dispositioned = 42,651`（68.65%）。 |

## 3. 机器校验不变量

1. **分母守恒**：`tracked_entries_total (62,130) == blob_entries_total (62,128) + symlink_entries_total (2) + gitlink_entries_total (0)`。
2. **诚实阅读计数**：成组处置的 41,834 个 `non-product` 路径保持 `read_status: "unread"`（不伪造全文行号区间），同时携带 `disposition_status: "grouped-evidence"`、`reviewer: "xuyao@ombhrum.com"` 与 `evidence_refs: ["docs/specs/ocrm-bulk-subtrees-disposition.md"]`。
3. **跨版本内容身份复用**：当 `expand_inventory.py` 处理 inventory 时，只要 `(path, blob_sha)` 完全一致，即保留既有深读或成组处置记录，严禁因 `source_commit` 字段变化抹除已审记录。
