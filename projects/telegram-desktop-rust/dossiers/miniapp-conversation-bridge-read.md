# Mini App / Bot WebApp conversation bridge — source completeness evidence

Status: read-complete / mapped  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`  
Rebaseline inheritance: source identity cited below represents Telegram Desktop Bot WebApp / Mini App attachment bridge (`inline_bots/bot_attach_web_view.cpp`). Identity verified against recursive upstream inventory artifact 13118749668 (digest `sha256:77501e120e4df95742c028a7b35e1e7e4f570b0a7b2a2d6842999a70be8f9d60`).

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp` | `a99e6c7ec3aae8ab2db77b74ce387f33794d584e` | complete read; 90,208 bytes | Bot WebApp conversation attachment and bridge | mapped |

## Transferable responsibilities

The accepted upstream carries Bot WebApp / Mini App lifecycle and conversation bridging:

- Mini App launched from chat action (`InlineButtonAction::MiniApp`) carries Host-issued immutable `launchContext { launchSessionId, conversationId, startParameter }`.
- Host verifies that the authenticated local human is a participant in the conversation before issuing a launch session.
- Sandboxed desktop container enforces strict CSP and origin isolation; no second WebView container is allowed.
- Guest script in preload accesses read-only launch context via `fabushi.getLaunchContext()`.
- Bridge tools `conversation.sendMessage` and `conversation.sendData` dispatch through `MiniAppToolRouter` to Host gateway method `miniAppPostToConversation`.
- Host enforces pure deterministic authorization via `fabushi_messaging_core::authorize_miniapp_request`: validates session expiry, grant validity/expiry, permissions, and conversation binding against Host-held authoritative records.
- Fail-closed security fences:
  - `SendMessage` requires explicit Host user confirmation before posting on behalf of the user; unconfirmed calls fail closed.
  - `SendData` is data sent to a Bot recipient; calling `SendData` in a Human direct conversation without a Bot participant fails closed.
- Desktop message state belongs solely to `ProductionSessionWorkers`; messages are durably appended via `append_human_message_with_context` with deterministic nonce `miniapp:{launchSessionId}:{requestId}`. Replays are idempotent; content conflicts on the same nonce fail closed.
- Forward batch isolation: MiniApp messages do NOT use `forwardContext`, ensuring the retry/replay path never misidentifies them as forward batches.

## Existing-owner audit

Desktop canonical owners:
- Container: `source/electron-main/mcp/miniapp-window.ts`
- Tool Router: `source/electron-main/mcp/miniapp-tools.ts`
- Preload Bridge: `source/electron-preload/preload-miniapp-tools.ts`
- Management & Launch: `source/electron-main/mcp/miniapp-management.ts`, `miniapp-production.ts`
- Pure Domain Policy: `fabushi_messaging_core::authorize_miniapp_request` (`native/mahayana-messaging/src/miniapp.rs`)
- Message State & Storage: `ProductionSessionWorkers` (`source/host/src/extensions/session/production.rs`, `gateway.rs`)

No duplicate WebView container or secondary message store is introduced.
