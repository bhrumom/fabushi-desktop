# Mini App / Bot WebApp conversation bridge — source completeness evidence

Status: read-complete / mapped  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop Bot WebApp / Mini App attachment bridge (`inline_bots/bot_attach_web_view.cpp`). Full recursive upstream blob enumeration is preserved in remote CI artifact 13118749668 and unpersisted in local repository; status is held at `mapped` without fabricated blob SHA pending exact upstream blob binding.

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp` | `unresolved-pending-upstream-inventory` | complete read; local blob unresolved | Bot WebApp conversation attachment and bridge | mapped |

## Transferable responsibilities

The accepted upstream carries Bot WebApp / Mini App lifecycle and conversation bridging:

- Mini App launched from chat action (`InlineButtonAction::MiniApp`) carries immutable `conversationId` and optional `startParameter`.
- Sandboxed desktop container enforces strict CSP and origin isolation; no second WebView container is allowed.
- Guest script in preload accesses read-only launch context via `fabushi.getLaunchContext()`.
- Bridge tools `conversation.sendMessage` and `conversation.sendData` dispatch through `MiniAppToolRouter` to Host gateway method `miniAppPostToConversation`.
- Host enforces pure deterministic authorization via `fabushi_messaging_core::authorize_miniapp_request`: validates session expiry, grant validity/expiry, permissions, and conversation binding.
- Desktop message state belongs solely to `ProductionSessionWorkers`; messages are durably appended via `append_human_message_with_context` with deterministic nonce `miniapp:{sessionId}:{requestId}`. Replays are idempotent; content conflicts on the same nonce fail closed.

## Existing-owner audit

Desktop canonical owners:
- Container: `source/electron-main/mcp/miniapp-window.ts`
- Tool Router: `source/electron-main/mcp/miniapp-tools.ts`
- Preload Bridge: `source/electron-preload/preload-miniapp-tools.ts`
- Pure Domain Policy: `fabushi_messaging_core::authorize_miniapp_request` (`native/mahayana-messaging/src/miniapp.rs`)
- Message State & Storage: `ProductionSessionWorkers` (`source/host/src/extensions/session/production.rs`, `gateway.rs`)

No duplicate WebView container or secondary message store is introduced.
