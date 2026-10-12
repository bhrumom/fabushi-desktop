# history/history.{h,cpp} complete read

Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`.

Fully read exact blobs:
- `Telegram/SourceFiles/history/history.h@d3a6412fc9d5246164c8bfed777ecf6bb32c31e8` — 795 lines.
- `Telegram/SourceFiles/history/history.cpp@1fabd7d7d9399f8db8141c29d71068d447736595` — 4651 lines, read in contiguous ranges covering 1–4651.

Status: fully read/decomposed; mapped/open only.

## State and child identity

`History` is not presentation-only. It is a root Conversation history state owner with typed Topic/SavedSublist child routing. Drafts and forward drafts are keyed by `DraftKey(topicRootId, monoforumPeerId)`; read cursors, unread counts, notification state, loaded ranges, last/chat-list messages and message indexes share one lifecycle.

Fabushi mapping is existing-owner-first:
- `native/mahayana-messaging/src/engine.rs#MessagingEngine`: authoritative command/event and lifecycle sequencing.
- `native/mahayana-messaging/src/conversation.rs#ConversationChildRuntimeState`: exact Topic/SavedSublist child state, read/unread, notification ids and draft scope.
- `native/mahayana-messaging/src/message.rs#Message`: canonical message identity/content/delivery state.

No Telegram/History runtime or second Message/Search store is introduced.

## Deletion / destruction ordering

The implementation establishes coupled responsibilities:
- `destroyMessage` removes history-entry state, HistoryMessages membership and shared-media membership, applies `itemRemoved`, unregisters the canonical message, clears OS notification references while item identity still exists, then retires storage ownership.
- bulk date/topic/sublist destruction first calls `notifyItemsAboutToBeDestroyed`, then destroys the items.
- `itemRemoved` updates last/chat-list message state, client-side registration, Topic/SavedSublist projections and streamed drafts.
- `itemVanished` removes per-thread notification references and reconciles unread count.
- `clear` / `clearUpTill` notify destruction before retirement, clear notifications/shared media/message indexes and converge last/chat-list state.
- new-item paths add unread things, notifications, Topic/SavedSublist projections and shared-media indexes; deletion must reverse these effects under the same exact scope.

This confirms SavedSublist deletion is a cross-owner lifecycle, not renderer disappearance.

## Drafts and forward staging

Local/cloud/edit drafts and `ForwardDraft` are child-scoped by `topicRootId + monoforumPeerId`. Cloud-draft save fencing prevents stale server drafts from overwriting active/new local state. Forward-draft resolution prunes missing source items and emits an exact thread entry update.

Fabushi must keep these semantics in canonical Composer/Conversation state. Child destruction must clear only the exact destination scope and must not fall back to the parent Conversation.

## Unread / recent / ordering

`History` owns monotonic inbox/outbox read positions, local unread reconciliation, per-Topic and per-SavedSublist interval accounting, unread mention/reaction/poll clearing, and chat-list/recent ordering derived from last surviving authoritative state.

The SavedSublist interval logic explicitly relies on real sublist membership. Where the authoritative server relation/membership feed is absent, Fabushi remains blocked/fail-closed rather than deriving child membership from generic relations or client inference.

## Shared media, Search and resources

History adds/removes shared-media indexes and message indexes as messages enter/leave lifecycle. Fabushi canonical Search projection and resource/media cleanup must converge from the same authoritative deletion boundary.

Current exact-head audit:
- `SearchIndex` exists and exposes `remove_message`, but these read-credit rows do not claim that cross-owner deletion wiring is already composed or evidenced.
- `media.rs` exposes `MediaTransferQueue`; there is no exact-head symbol named `MediaCache`. No nonexistent target symbol is registered to make the ledger look complete.
- Search projection removal, resource/cache retirement and document/message cleanup remain explicit downstream open responsibilities.

## Traceability

- `TDRP-R9-HISTORY-STATE-CONTRACT-001` -> `ORA-TDRP-R9-HISTORY-STATE-CONTRACT-001` -> `INV-TDRP-R9-HISTORY-STATE-CONTRACT-001-CANONICAL`.
- `TDRP-R9-HISTORY-LIFECYCLE-001` -> `ORA-TDRP-R9-HISTORY-LIFECYCLE-001` -> `INV-TDRP-R9-HISTORY-LIFECYCLE-001-CANONICAL`.

Both remain `mapped/open`. Production entrypoints, tests and current-head evidence are intentionally empty/pending.

## Accounting

These two exact accepted blobs receive read credit now:
- unknown: 15788 unchanged
- unread: 15730 -> 15728
- omitted: 0
