# Telegram Capability Source Research & Provider Implementation Sub-Spec

Status: active  
Spec ID: TDRP-001  
Revision: 3  
Last updated: 2026-10-02  
Parent product spec: `FBCP-001`  
Parent: `docs/specs/fabushi-bot-communication-platform.md`  
Related project: `projects/telegram-desktop-rust`

> 本规范不再定义“重做 Telegram Desktop”这个产品。它是 FBCP-001 的 Telegram capability source/provider 子规范：深入研究固定 Telegram Desktop 源码，完整发现通信能力和边界条件，然后将这些能力实现为 Fabushi Communication Platform 的 Telegram Provider 与相关产品域能力。

## 1. 最高原则

- **Fabushi Bot is the product.**
- Telegram Desktop 是能力来源、行为 oracle、协议/平台研究对象，不是目标产品架构。
- 不创建最终用户可见的独立 Telegram workspace/app。
- Telegram 能力最终进入 Fabushi 的 Identity / Conversation / Messaging / Media / Calls / Search / Notifications / Settings 等产品域。
- Telegram 独有语义通过 typed provider extension 保留。
- 完整 Telegram 功能范围不能因为产品统一而缩水。
- 所有 Telegram/desktop-app 自有 C++ production logic 最终由 Rust production owner 取代。
- 非 C++ 部分按职责使用最佳架构/语言。
- 本路线 source-informed，不是 clean-room。

## 2. 固定研究基线

Upstream:

`telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`

`dev` 仅用于 drift discovery，不作为浮动实现输入。

机器可读记录：

`projects/telegram-desktop-rust/upstream.lock.json`

## 3. 研究目标

研究单位是 capability，不是 target file。

每个 capability dossier 必须回答：

- 用户可观察行为；
- Telegram network/protocol contract；
- source references；
- state machine；
- ordering/idempotency/retry/cancellation；
- storage/restart；
- permission/security；
- platform variants；
- failure/negative cases；
- performance/resource constraints；
- interaction with other capabilities；
- target FBCP product domain；
- provider-specific semantics that cannot be generalized；
- C++ production regions that require Rust replacement；
- license/provenance。

## 4. Telegram capability scope

至少覆盖：

- account/login/multi-account
- MTProto/TL/session/DC
- updates/difference/recovery
- peers/contacts
- dialogs/archive/folders
- messages and all supported message variants
- reply/quote/forward/edit/delete
- drafts/scheduled/silent
- reactions/polls
- groups/supergroups/channels/topics
- permissions/admin/invites
- search
- media/upload/download/cache/streaming
- voice/video/calls/screen sharing
- stickers/GIF/custom emoji
- Stories
- notifications/tray/badge
- settings/privacy/local lock
- Bots/inline bots/Mini Apps/WebView
- Premium/Stars/gifts/business
- payments/Passport/WebAuthn/tde2e where applicable
- export/local data/migration/recovery
- themes/language/RTL/IME/accessibility
- platform install/update/portable/sandbox

任何固定源码中存在但未在列表中的可达产品能力自动纳入研究范围。

## 5. Target placement

默认不是：

`telegram-next/app/ui/...`

而是进入 FBCP 产品：

```text
product/
  identity/
  conversations/
  messaging/
  media/
  calls/
  search/
  notifications/
  permissions/
  agents/
  automations/
  computer/
  plugins/

communication/
  providers/
    telegram/
      protocol/
      sync/
      storage/
      platform/
      extensions/
```

这只是责任方向，不是预先锁死的物理目录。

Telegram Provider 负责 network/provider truth；Product domains 负责 Fabushi product truth。

## 6. C++ replacement

以下若属于 Telegram/desktop-app 自有 production logic，最终必须 Rust-owned：

- protocol/session/update state machines
- business/domain logic
- storage logic
- UI business behavior
- platform policy
- Telegram-specific media/call orchestration
- build/runtime tools that ship as product logic

不允许：

- original tdesktop binary
- Qt business UI
- TDLib-as-client-core
- C++ helper fallback
- Rust wrapper around old C++ implementation

第三方原生能力需逐依赖 ADR；系统 API 不视为 Telegram C++ logic。

## 7. Source inventory 的用途

仍然要求完整递归 source/dependency/resource inventory，但目的为：

- 防止遗漏 capability；
- C++ production logic closure；
- provenance；
- license；
- build/generator understanding；
- platform matrix。

不计算 source-file port percentage，不创建 source-file → target-file completion gate。

## 8. FBCP domain mapping

每个 researched capability 必须映射到：

- FBCP core domain；或
- Telegram Provider；或
- typed Telegram extension；或
- platform boundary；或
- tool/build/release responsibility。

映射的是**责任**，不是文件。

如果某项 Telegram 能力无法合理进入通用 Conversation/Message 模型，不得删除；必须保留 provider extension。

## 9. Interaction with Agent plane

Telegram sync/data path 不直接调用 Agent Runtime。

所有 communication → Agent 数据流经 FBCP `InteractionGateway`。

本子规范必须提供足够 metadata 让 policy 能判断：

- data origin
- Telegram account
- conversation
- message/attachment provenance
- bot/client/business source
- user action
- provider permissions
- retention constraints

## 10. First vertical slice

本项目第一条实现链路不再是“独立 Telegram app 登录并聊天”。

必须嵌入 Fabushi 产品：

1. Fabushi product shell；
2. Telegram sign-in；
3. Telegram Provider session/sync；
4. unified Identity/Conversation projection；
5. existing Agent entries 与 Telegram private chat 同时出现在 Inbox；
6. private chat send/receive；
7. durable restart recovery；
8. selected message can be explicitly handed to Agent through FBCP InteractionGateway；
9. Agent result remains distinct from Telegram send state；
10. user-controlled publish back to conversation。

## 11. P0

P0 继续完成：

- recursive upstream closure；
- C++ production-logic inventory；
- source → capability research coverage；
- full Telegram capability graph；
- first research dossiers；
- provider architecture questions；
- source/license provenance；
- capability ledger。

新增要求：

- 每项 capability 必须指出 FBCP destination；
- 标出 generic product semantics 与 Telegram-specific extension；
- 标出与 PR #20 现有 Bot model 的冲突/融合点；
- 不创建 Telegram-only product shell 设计。

## 12. Acceptance

TDRP-001 自身 accepted 需要：

- full research coverage；
- all applicable Telegram capabilities implemented；
- Telegram Provider interop；
- all Telegram C++ production logic replaced；
- no unexplained behavior gap；
- all FBCP mappings real production-wired；
- provider-specific semantics preserved；
- packaged FBCP product evidence；
- exact-head GitHub Actions/htch-runtime evidence；
- license/API/provenance review。

TDRP-001 通过不代表 FBCP-001 自动通过；FBCP 还要求 Agent/communication product integration、permissions、data boundary 和 unified UX。

## 13. Execution environment

所有 executable verification 仅允许：

- GitHub Actions
- `htch-runtime`

禁止在本地工作站、用户 Mac/Windows 或助手本地容器运行 build/test/lint/generator/schema/benchmark/fuzz/package/acceptance。

## 14. Dynamic external rules

实现和 release 时必须重新读取：

- https://core.telegram.org/api/terms
- https://telegram.org/tos/bot-developers
- https://core.telegram.org/api/bots/ai

条款变化可能改变 InteractionGateway/Data Policy，但不能通过偷偷删功能来“解决”。

## 15. Current status

- fixed source baseline: recorded
- recursive closure: blocked
- research coverage: blocked
- FBCP destination mapping: blocked
- Telegram Provider: not implemented
- C++ Rust replacement: not started
- packaged product acceptance: blocked

Revision 1/2 中任何“Telegram client 是目标产品”的表述均由 FBCP-001 + 本 Revision 3 取代。
