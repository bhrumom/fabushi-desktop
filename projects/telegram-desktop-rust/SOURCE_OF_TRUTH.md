# Telegram Capability Research / Provider — Source of Truth

Status: active  
Project ID: TDRP-001  
Date: 2026-10-02  
Parent product: FBCP-001  
Repository: `bhrumom/fabushi-desktop`

## Role

这个项目不再是独立 Telegram 产品项目。

它只负责为 **Fabushi Bot Communication Platform** 提供：

- Telegram complete capability research；
- fixed source provenance；
- Telegram protocol/provider implementation；
- Telegram-specific product semantics；
- C++ → Rust replacement evidence；
- Telegram interoperability / platform acceptance。

Canonical product spec:

`docs/specs/fabushi-bot-communication-platform.md`

Telegram sub-spec:

`docs/specs/telegram-desktop-rust-equivalence-migration.md` Revision 3。

## Product invariant

**Fabushi Bot is the product. Telegram is a communication capability source and provider.**

禁止：

- 最终独立 Telegram workspace；
- Telegram-only product shell；
- source module mirror as target architecture；
- source-file port percentage；
- Telegram provider owning Agent Runtime；
- Telegram sync directly calling models。

## Source research

固定上游：

`telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`

研究源码时仍需要完整 inventory、capability graph、C++ production logic closure 和 provenance。

每项 capability 必须额外回答：

- 它进入哪个 FBCP product domain？
- 哪部分属于 Telegram Provider？
- 哪部分必须作为 typed Telegram extension 保留？
- 是否会跨 Communication → Agent boundary？
- 对 InteractionGateway/permissions/data policy 有什么要求？

## C++ rule

所有 Telegram/desktop-app 自有 C++ production logic 最终必须由 Rust production owner 替代。

不接受 tdesktop/Qt/TDLib/C++ helper/Rust façade 作为最终替代。

## Execution environment

所有 build/test/lint/generator/schema/benchmark/fuzz/package/acceptance 仅允许：

- GitHub Actions
- `htch-runtime`

## Next work

配合 FBCP P0：

1. complete Telegram capability graph；
2. source → capability research coverage；
3. C++ production-logic inventory；
4. FBCP destination mapping；
5. generic vs Telegram-specific semantics；
6. first research dossiers；
7. provider ADR questions；
8. capability ledger。

首条实现不是 Telegram demo，而是嵌入 Fabushi product shell 的 provider vertical slice。
