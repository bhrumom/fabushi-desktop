# Telegram adjacent message lifecycle research dossier

Status: researched for reply/quote/forward, edit/delete, reactions, drafts, scheduled/silent send, and send progress; implementation/acceptance remains open
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Target architecture snapshot: PR #20 `95995bdf36a9687788e106c8544d292b2bb0877f`
FBCP work snapshot inspected: `dc6a14e20d1a9a1f9efcc0b4de59542a3a9969b8`

Snapshot refresh note: the exact `556f6308... → c2767eac...` PR #20 delta was inspected; it only moves Agent outline-stream identity/coalescing into shipping `RosterProjection`/`ProductionRosterEmit`, so the owner paths selected in this dossier remain valid at c276.

This dossier continues the already-recorded message/history research. Telegram is source evidence only; none of the MTProto request types, IDs, or provider ownership below become Fabushi runtime architecture.

## Frozen upstream evidence

### Reply / quote / forward provenance
- `Telegram/SourceFiles/api/api_sending.cpp:91-109, 220-223, 284-307`: reply metadata is attached to the same outgoing local message before remote settlement; invalid local reply targets are normalized rather than producing a second message truth.
- `Telegram/SourceFiles/history/history_item.cpp:680-714`: forwarding preserves origin/saved-from provenance separately from the new message identity.
- `Telegram/SourceFiles/history/history_item.cpp:1020-1058, 1187-1214`: reply dependencies are resolved lazily and updated when the referenced item changes or disappears; relation failure does not require deleting the referencing message.

Fabushi contract: a message owns stable relation fields referencing canonical message IDs. Reply, quote, and forward provenance are relations, not duplicate transcript entries. Missing/deleted targets project an explicit unavailable target while the referencing message remains valid.

### Edit / delete
- `Telegram/SourceFiles/api/api_editing.cpp:57-104`: edit request semantics depend on the existing message identity, including scheduled-message identity mapping.
- `Telegram/SourceFiles/history/history_item.cpp:1228-1247`: an edition refreshes the existing item and dependent projections.
- The earlier message/history dossier records the frozen `history_item.cpp` edition path where authoritative empty content removes the existing item and normal editions mutate it in place.

Fabushi contract: `MessageEdited` / `MessageDeleted` reference the original canonical `message_id`; each operation has its own idempotency identity. A retry may not mint a replacement chat message. Delete must define tombstone versus hard projection removal policy explicitly so replies, audit state, and synchronization remain deterministic.

### Reactions
- `Telegram/SourceFiles/data/data_message_reactions.cpp`: reaction state is message-scoped authoritative state with chosen/current reaction bookkeeping and update application.
- Current Fabushi already has `frontend/src/production/reaction-root.ts`, which performs optimistic reaction projection but rejects pending/queued/failed message targets and is explicitly a transcript observer rather than an alternate transcript store.

Fabushi contract: reaction operations target canonical `message_id`, use an operation/idempotency identity, reconcile optimistic UI with authoritative `ReactionChanged`, and deduplicate replay. Reaction truth remains attached to the transcript message; network transport stays below it.

### Drafts
- `Telegram/SourceFiles/data/data_drafts.cpp` plus the durable account-storage evidence captured in `message-history-lifecycle.md`: draft state is conversation scoped and carries reply/cursor state independently from sent-message settlement.
- The existing FBCP message/history dossier already records corruption fencing from frozen `storage/storage_account.cpp:1280-1680`.

Fabushi contract: the existing composer/draft owner keeps account + conversation/thread scoped text, reply/quote target, attachment/editor state and cursor/selection as applicable. Draft persistence must never create a second message store. Corrupt or cross-scope draft state fails closed or migrates explicitly.

### Scheduled / silent send
- `Telegram/SourceFiles/api/api_sending.cpp:43-69, 126-149, 258-297`: silent/scheduled properties are send options on the same message action; scheduled messages still receive local identity and relation metadata before settlement.

Fabushi contract: scheduled delivery is an Automation-backed future send action that eventually enters the same durable composer/send acceptance path. Silent delivery is notification policy metadata, not a separate message type or transport subsystem. The logical message/client nonce must survive schedule execution retries.

### Typing / upload / send progress
- `Telegram/SourceFiles/api/api_send_progress.cpp:21-25, 33-52, 55-108`: progress is throttled, cancelable, keyed to conversation/thread/type, and expires.
- `api_send_progress.cpp:111-151`: typing/upload/speaking actions are transient signals and typing auto-cancels after a bounded interval.
- `api_send_progress.cpp:154-170`: some targets suppress progress entirely based on context.

Fabushi contract: typing/upload progress belongs to ephemeral presence/progress infrastructure plus conversation projection. It must not be persisted as canonical transcript history. Disconnect/restart drops stale progress; later live signals supersede earlier ones.

## Exact current Fabushi owner evidence

| Capability | Plausible existing owners | Selected existing owner | Exact evidence |
| --- | --- | --- | --- |
| reply / quote / forward | Transcript/message model; conversation workspace; composer | Transcript relation/provenance model, with composer selecting the relation | `frontend/src/production/ProductionRenderer.tsx`; `source/host/src/extensions/transcript/transcript_manager.rs`; Session/Transcript SQLite is already the Human message owner |
| edit / delete | Transcript lifecycle; Session SQLite; new message service | existing Transcript/message lifecycle + Session/Transcript SQLite persistence | `source/host/src/extensions/transcript/transcript_manager.rs`; `source/host/src/extensions/session/production.rs` |
| reactions | reaction UI root; Transcript; Shared Room | existing reaction/transcript owner | `frontend/src/production/reaction-root.ts` and recovered transcript-card reaction actions/feed |
| drafts | composer; renderer local state; Session | existing composer/draft owner | existing production composer in `frontend/src/production/ProductionRenderer.tsx`; durable sync extension to remain scoped below it |
| scheduled send | Automations; composer/send; Transcript | Automations schedules; existing composer/send performs eventual durable admission | `source/host/src/extensions/transcript/transcript_manager.rs` owns AutomationRuntime together with Transcript; existing send pipeline remains the admission path |
| typing/upload progress | Transcript; Shared Room/presence; new network core | existing conversation projection + minimal ephemeral presence/progress infrastructure | no durable transcript owner should be added; network signal feeds existing workspace only |

Rejected for all rows: `CommunicationCore`, `TelegramProvider`, a Telegram-prefixed message model, a second message store, or a second workspace/sidebar.

## Required model/network changes

1. Extend the canonical message shape with optional relation/provenance fields: reply target, quote range/text fingerprint where needed, and forward provenance. All references use Fabushi canonical IDs.
2. Add native idempotent operations for `MessageEdited`, `MessageDeleted`, and `ReactionChanged`; operation IDs are distinct from canonical message IDs and transport attempt IDs.
3. Preserve causal ordering: an edit/reaction/delete arriving before its base message is buffered or triggers bounded gap recovery, never materialized as a phantom second message.
4. Draft sync, when implemented, uses account + conversation/thread versioning and explicit conflict semantics; local renderer state is only a projection/cache.
5. Scheduled sends persist Automation identity plus the eventual logical message/client nonce so crash/retry cannot duplicate a send.
6. Typing/upload progress uses expiring sequence/timestamp semantics and is dropped on stale reconnect; it does not participate in durable history gap recovery.
7. All network/sync events project through existing Session/Transcript/Composer/reaction owners.

## Focused contracts required

- reply/quote references survive restart and a missing/deleted target projects deterministically;
- forward provenance is immutable under resend/retry and does not overwrite author identity;
- same edit operation replay is idempotent; edit with conflicting immutable operation payload fails closed;
- delete replay is idempotent and an edit after an authoritative delete follows an explicit reject/tombstone rule;
- reaction replay and out-of-order authoritative snapshots converge without duplicate reactions;
- reaction against pending/failed unsynchronized messages remains fenced;
- corrupt or cross-conversation draft state fails closed or follows an explicit migration;
- scheduled send restart executes at most one logical message for one schedule occurrence;
- canceled schedule cannot later redrive the send;
- silent-send metadata affects notification delivery only, not canonical message identity;
- typing/progress expires, is cancelable, is not replayed after restart, and stale sequence updates are ignored;
- edit/delete/reaction arriving across a detected history gap causes bounded recovery before projection.

## Blockers / implementation state

These capabilities are **researched and owner-resolved, not implemented for the Human native-network path**. The current Human vertical slice has durable local send acceptance and transcript recovery, but no Fabushi-native remote settlement/sync yet. Therefore reply/edit/delete/reaction network behavior must not be marked production-complete until the underlying durable transport/sync slice exists and exact-head packaged acceptance exercises it.
