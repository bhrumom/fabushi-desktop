# Fabushi Bot Communication Platform — Source of Truth

Status: active  
Project ID: FBCP-001  
Date: 2026-10-02  
Repository: `bhrumom/fabushi-desktop`

## Product statement

**Fabushi Bot is the product. Telegram is a complete communication capability source and network provider, not a separate product.**

最终目标是在现有 Bot / Agent 产品基础上，拥有 Telegram 的全部适用功能，并让 Human、Agent、Computer、Plugins/MCP、Automations、Tasks、Artifacts 与通信网络成为一个产品。

## Authority

1. latest explicit user requirement
2. `docs/specs/fabushi-bot-communication-platform.md`
3. applicable PR #20 / canonical Grok Bot architecture spec
4. TDRP-001 Telegram capability research/provider sub-spec
5. approved ADRs
6. exact-head implementation/evidence
7. historical chats/docs

## Must read

- root `AGENTS.md`
- `docs/specs/fabushi-bot-communication-platform.md`
- `docs/specs/grok-bot-018-runtime-product-parity-recovery.md` when working on Bot architecture
- `docs/specs/telegram-desktop-rust-equivalence-migration.md`
- `projects/telegram-desktop-rust/SOURCE_OF_TRUTH.md`
- current task / ADR / evidence

## Non-negotiable architecture

- one Fabushi product shell
- no final Telegram-only workspace
- existing Agent Runtime remains independent from Communication Core
- Telegram Provider does not own Agent execution
- Communication Core does not call models directly
- cross-plane data goes through InteractionGateway
- unified product Identity / Participant / Conversation / Message model
- provider-specific Telegram semantics preserved
- all Telegram/desktop-app C++ production logic ultimately Rust-owned
- non-C++ boundaries use best-fit language via ADR
- no duplicate conversation/state owner
- all tests/builds only GitHub Actions or htch-runtime

## PR #20 integration

At this revision, observed PR #20 HEAD:

`7823c596712b674661e40925b1426d945d0e2e55`

It already contains Human/Agent Shared Room concepts and Bot features such as Computer and Automations. FBCP is an approved product extension of that Bot foundation.

Always reread the live exact HEAD before implementation.

## Current next task

`management/tasks/P0-product-domain-and-telegram-absorption.md`

P0 is architecture/research work, not a Telegram demo implementation.

## Data boundary

Telegram communication data is not automatically Agent/model context.

Any communication → Agent transfer must be explicit, policy-checked, provenance-preserving and compatible with current Telegram terms and user permissions.
