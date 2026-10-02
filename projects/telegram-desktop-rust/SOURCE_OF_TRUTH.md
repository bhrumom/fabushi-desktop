# Telegram Desktop Source-Informed Rearchitecture — Source of Truth

Status: active  
Date: 2026-10-02  
Project ID: TDRP-001  
Repository: bhrumom/fabushi-desktop

## 最新要求

研究固定 Telegram Desktop 源码，完整理解功能、协议、状态机、失败语义和平台差异；不要逐文件、逐模块照抄目标结构。

所有进入最终产品的 C++ 产品逻辑必须由更好的 Rust 实现替代。

非 C++ 部分在理解职责后选择最合适的架构、语言和生态，不做“所有东西都强制 Rust”。

完成标准是完整 capability/behavior/protocol/platform parity 和更清晰的目标架构，不是 source-file parity。

## 必须先读

1. 根 AGENTS.md。
2. docs/specs/telegram-desktop-rust-equivalence-migration.md Revision 2。
3. upstream.lock.json。
4. module-map.md。该文件现在是 source research/capability map，不是目标 module map。
5. contracts/parity-ledger.schema.json。该 schema 现在描述 capability ledger，而不是逐文件 port ledger。
6. STATUS.md。
7. 当前任务与 ADR。

## 关键不变量

- 上游源码可以深入阅读和研究。
- 上游源码结构不是目标架构。
- 不要求 source file → target file、class → struct、module → crate 一一对应。
- C++ production logic 最终必须为零，由 Rust production owner 替代。
- 非 C++ 使用 best-fit language，但业务状态 owner 必须唯一。
- tdesktop/Qt/TDLib/原 C++ helper 不能成为最终 fallback。
- 研究覆盖和实现完成度分开统计。
- 这是 source-informed 路线，不是 clean-room，不得声称换成 Rust 自动摆脱 GPL。
- 现有 Grok/Agent 继续由自身 Spec 管理；Telegram 项目不授权破坏其边界。
- 所有 build/test/benchmark/fuzz/package/acceptance 仅允许 GitHub Actions 或 htch-runtime。

## 固定上游

telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677

dev 只用于发现 upstream drift。实现和验收不能使用浮动 dev。

## 当前工作方式

Discover source → capability research dossier → architecture alternatives → ADR → implementation → behavior/differential tests → production wiring → packaged acceptance → independent review。

禁止：

Discover file → create same-name Rust file → mark ported。

## 下一步

执行 P0 source understanding：

- 完整递归 source/dependency/resource inventory；
- 标出所有 C++ production logic；
- 建 source → capability research coverage；
- 建 capability graph；
- 写第一批 research dossiers；
- 建 architecture risk/ADR backlog；
- 建 capability ledger。

P0 不做逐文件 target mapping。

首个实现 vertical slice：

authentication → MTProto/session → updates → storage → dialogs/history → compose/send → server update → restart recovery。

## 执行环境

所有构建和测试只能在 GitHub Actions 或 htch-runtime。缺环境一律 blocked/not-configured，不能移到本地电脑替代执行。
