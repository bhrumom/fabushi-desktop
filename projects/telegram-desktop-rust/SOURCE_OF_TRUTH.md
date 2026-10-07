# Telegram 源码 → Fabushi 全量等价重写 — Source of Truth

Status: active  
Project ID: TDRP-001  
Revision: 5  
Date: 2026-10-07  
Parent: FBCP-001 Revision 3

## Role

本项目负责完整 `telegramdesktop/tdesktop` 源码的逐文件、逐模块理解、责任对照、等价重写、生产接入和取证，不再只是 capability research 子项目。

上游为源码/行为来源，不是最终 runtime/network/product。必须把其全部非 UI 产品职责吸收入 current canonical Fabushi，C++ 默认 Rust；UI 使用 Fabushi 原有前端架构按统一左竖栏结构重做，UI 内业务不得遗漏。完整产品保留 Bot 全部能力，统一 Fabushi 品牌。

## Must read

- root `AGENTS.md`
- `docs/specs/fabushi-bot-communication-platform.md` Revision 3
- `docs/specs/telegram-desktop-rust-equivalence-migration.md` Revision 5
- current canonical Fabushi/Bot source and specification
- `module-map.md`, existing ledger/schema, dossiers and active task

新规范优先于旧材料中的 research-only 或“决定是否需要”的选择性实现表述。历史 ledger/schema 不会因为规范更新自动达到新 gate。

## Baseline

2026-10-07 discovery: `telegramdesktop/tdesktop@f23c37857220eb84f8559f0901ea26fb304b564b`。

历史 `upstream.lock.json`/研究引用的基线为 `33261535a0e747f125e0ed25486f01e556330677`。二者不得混用。下一执行需在 GitHub Actions 重新确认上游、全量递归解析并一致更新 lock/inventory/ledger/dossiers，审计差异后才设 `baseline_ready=true`。不能只替换 SHA 或继承 verified。

## Mandatory chain

`every source file -> every non-UI responsibility -> module behavior contract -> existing Fabushi owner -> target path/symbol/language -> shipping composition -> exact-head behavior/release evidence`

允许多对一/一对多文件映射，不要求复制目录或类名，但不允许失去任何源文件/责任对照。无法找到现有 owner 时，记录 rejected owners 与最小 new-owner ADR。

## Non-negotiable

- 全仓库、递归 gitlinks、外部获取、资源、生成器、工具链及 licenses 都有 inventory。
- C++ 非 UI 产品逻辑重写；不以原 Telegram C++、TDLib、FFI 或 sidecar 保留真正业务 owner。
- 协议/同步代码同样逐文件研究并在 Fabushi 自有网络重实现职责；不能把不用 MTProto 当成功能排除。
- 远端服务、媒体/通话、商业等依赖未完成只能 blocked；blocked 不等于 accepted。
- 无 parallel CommunicationCore/Telegram runtime，无第二产品模型。
- 微信式左竖栏中消息、联系人、插件市场为一级入口；全部 Bot/通信能力共用产品 shell。
- 去 Telegram/Grok/Gok 产品品牌，但保留版权/许可证/NOTICE/source provenance；source-informed 不是 clean-room。
- 所有可执行 build/test/lint/generator/schema/benchmark/fuzz/package/acceptance 仅 GitHub Actions；不再使用本项目旧 htch-runtime 例外。

## Status

本次仅修订规格和入口。全部上游文件并未因本文而被读完，inventory/validators/production/UI 未因本文而完成。模块必须经过 unreviewed -> understood -> mapped -> implemented -> verified 的真实证据路径，最终所有 in-scope 责任闭合才可验收。
