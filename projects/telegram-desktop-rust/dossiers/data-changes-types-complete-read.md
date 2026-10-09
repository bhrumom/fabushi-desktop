# data_changes.{h,cpp} + data_types.h complete read

Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`.

Fully read exact blobs:
- `Telegram/SourceFiles/data/data_changes.h@f10c987604b21b84626f5fb0288c180cedac9f2c` — 484 lines.
- `Telegram/SourceFiles/data/data_changes.cpp@8b5d73c9a8fa037d05cdb299acff3c4a76f29270` — 395 lines.
- `Telegram/SourceFiles/data/data_types.h@ce41ac4ee1d062ec7b0a2e5a1554fa9ccf0d60b3` — 418 lines.

Status: fully read/decomposed; mapped/open only.

## data_changes responsibility

`Data::Changes` is a source-wide typed mutation notification owner, not UI plumbing. It distinguishes Peer, History, Topic, SavedSublist, Message, Dialog Entry, Story and membership/rank updates.

Each typed manager has two delivery classes:
1. realtime per-flag delivery happens synchronously on `updated()`;
2. ordinary updates coalesce flags by exact object and flush on the main-thread scheduled notification pass.

For Topic, SavedSublist, Message, Dialog Entry and Story, a `Destroyed` update sets `dropScheduled=true`: pending flags are merged, the combined update is fired immediately, and the object is removed from the pending map. No later scheduled callback is allowed to dereference or resurrect that destroyed object. Explicit `topicRemoved`, `sublistRemoved`, and `entryRemoved` also drop pending records.

Relevant flags include:
- Topic: unread, notifications, cloud draft, destroyed.
- SavedSublist: unread, reactions, cloud draft, destroyed, unread poll votes.
- Message: edited, destroyed, new/unread-reaction.
- Entry: `ForwardDraft`, `LocalDraftSet`, `Destroyed`.
- History: unread, cloud draft, streamed drafts.

Fabushi mapping: canonical MessagingEngine command/event boundary, Conversation/child lifecycle events, Message lifecycle, unread/draft projections. No second generic event bus should be introduced merely to mimic Telegram.

Deletion consequence: SavedSublist destruction must publish its exact child-destroy lifecycle before child-owned observers/state are invalidated; notification clear, forward-draft clear, unread reconciliation, Search projection removal and resource cleanup must subscribe/sequence through canonical owners, not infer deletion from renderer disappearance.

## data_types responsibility

This header is a shared value-contract authority. It defines:
- cache/resource key inputs and stable cache tags for image/sticker/voice/video/animation classes;
- upload progress/preparation state;
- stable `MessageGroupId` including scheduled-message discrimination;
- canonical message/media identity aliases and serialization-stable document type values;
- `MessageCursor` position/anchor/scroll state;
- the full `MessageFlag` semantic set, including outgoing/silent/pinned/unread/reaction/scheduled/no-forward/local/fake/sending/failed/history/sensitive/restriction/paid/ephemeral semantics;
- `MediaWebPageFlag`;
- `ForwardOptions = PreserveInfo | NoSenderNames | NoNamesAndCaptions`;
- `ForwardDraft { ids, options }` and `ResolvedForwardDraft { items, options }`.

Fabushi mapping is split across existing canonical Message/Resource/Composer/MessagingEngine owners. The ForwardDraft value contract belongs to destination-scoped canonical Composer/Conversation state; cache/resource semantics stay with Resource/MediaCache; delivery/message flags stay with canonical Message. No monolithic source-shaped `DataTypes` owner is warranted.

## Open parity

Open work includes exact cross-owner destruction sequencing, SavedSublist notification/draft/unread/search/resource cleanup, forward staging persistence/recovery, full message flag applicability mapping, stable resource/cache lifecycle, and paid-send/revalidation. SavedSublist membership/relation truth remains blocked on the authoritative server feed.

## Accounting

These three exact accepted blobs receive read credit only now:
- unknown: 15788 unchanged
- unread: 15733 -> 15730
- omitted: 0


## Traceability and sibling-row audit

The three rows sourced from this dossier are deliberately `mapped/open`; none is claimed implemented or verified:

- `TDRP-R9-DATA-CHANGES-CONTRACT-001` -> `ORA-TDRP-R9-DATA-CHANGES-CONTRACT-001` -> `INV-TDRP-R9-DATA-CHANGES-CONTRACT-001-CANONICAL`. Current exact target bindings are `native/mahayana-messaging/src/engine.rs#MessagingEngine` for typed mutation/destruction sequencing and `native/mahayana-messaging/src/conversation.rs#ConversationChildRuntimeState` for typed Topic/SavedSublist draft/unread child state.
- `TDRP-R9-DATA-CHANGES-LIFECYCLE-001` -> `ORA-TDRP-R9-DATA-CHANGES-LIFECYCLE-001` -> `INV-TDRP-R9-DATA-CHANGES-LIFECYCLE-001-CANONICAL`. Current exact target binding is `native/mahayana-messaging/src/engine.rs#MessagingEngine` for realtime/coalesced/destruction lifecycle sequencing.
- `TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001` -> `ORA-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001` -> `INV-TDRP-R9-DATA-TYPES-VALUE-CONTRACT-001-CANONICAL`. Current exact target bindings are the same existing `MessagingEngine` command semantics and `ConversationChildRuntimeState` destination-scoped child state.

Audit result: `SearchIndex` is a real existing downstream owner, and `native/mahayana-messaging/src/media_cache.rs#MediaCache` is the real canonical media-cache owner with an exact `remove` operation. These dossier rows still do not claim Search/MediaCache target bindings or implementation evidence, because the deletion lifecycle is not yet composed into those owners. Search projection removal and media/resource cleanup therefore remain explicit open downstream responsibilities; future implementation must bind these real paths/symbols and add executable tests/evidence before status can advance.

Current test/evidence state for all three rows remains pending/empty by design. The authoritative SavedSublist relation/membership feed remains blocked/fail-closed and is not replaced by client inference, generic relations, fixtures, or mocks.
