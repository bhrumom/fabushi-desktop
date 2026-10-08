# data_drafts.h complete read

- Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`
- Exact blob: `f610d7ab604fade113d3311f7cc8b19daf946174`
- Source path: `Telegram/SourceFiles/data/data_drafts.h`
- Read status: complete, 268 lines
- Responsibility: `TDRP-R9-DRAFT-IDENTITY-CONTRACT-001`

## Full responsibility decomposition

The file is not just a text-draft struct. It defines the source contract for cloud draft apply/clear; webpage/link-preview draft state; the draft payload (text/tags, reply, suggest options, cursor, webpage, rich page/summary, save request); typed draft identity for local, edit, cloud, scheduled, welcome-message, and business-shortcut scopes; serialized-key compatibility including the old 32-bit encoding; topic-vs-SavedSublist (`monoforumPeerId`) discrimination; draft null/equality helpers; and chat-link draft mutation.

`DraftKey::Local(topicRootId, monoforumPeerId)` and the matching edit/cloud variants reject invalid mixed identities and encode SavedSublist identity separately with `kMonoforumDraftBit`. That is a product responsibility: a SavedSublist draft must never alias the parent/self draft or a topic draft.

## Existing-owner-first mapping

Current Fabushi already has:
- Rust `ConversationDraft`, `TopicDraft`, and `ConversationChildRuntimeState`;
- Rust `DraftChanged`, `TopicDraftChanged`, and `ConversationChildDraftChanged` lifecycle events;
- renderer `ComposerDraft` payload validation;
- account-scoped `createComposerDraftStateStore` persistence/recovery.

These are the correct canonical owners. No Telegram/source-specific draft store is allowed.

## Open parity

This row remains `mapped`, not implemented/verified. Current owners do not yet prove:
- typed namespaces equivalent to local/cloud/edit/scheduled/welcome/business shortcut;
- old serialized draft-key compatibility where migration requires it;
- full Human/Topic/SavedSublist payload parity for cursor/webpage/rich content;
- cross-device/cloud draft sync semantics;
- SavedSublist child scope without the authoritative server relation/membership feed.

The existing Agent composer store is therefore not accepted as the sole product truth for all conversation kinds.

## Deletion implication

This contract explains why clearing `ConversationChildRuntimeState::draft_text` is insufficient to claim the SavedMessages deletion sequence complete. The later `History::_forwardDrafts` store is a separate message-id/options draft keyed through the same typed draft identity and must be cleared independently.

## Accounting

This exact accepted blob is the only new unread closure in this commit:
- `unknown`: 15788 (unchanged)
- `unread`: 15740 -> 15739
- `omitted`: 0

No notification/history file is decremented here because those larger files have only been dependency-focused inspected, not yet fully decomposed across every responsibility.
