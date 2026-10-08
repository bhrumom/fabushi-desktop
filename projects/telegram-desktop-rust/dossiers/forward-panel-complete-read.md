# history_view_forward_panel.h/.cpp complete read

- Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`
- Header: `Telegram/SourceFiles/history/view/controls/history_view_forward_panel.h@4e01059c1b0f0703f99817b8ec6028133ffd3728` — complete, 108 lines
- Implementation: `Telegram/SourceFiles/history/view/controls/history_view_forward_panel.cpp@ea88a830e327705ff29c67514fea5b2215844831` — complete, 524 lines
- Status: fully read/decomposed; mapped/open, not implemented or verified

## Complete responsibility decomposition

This pair is not presentation-only. It owns destination-scoped staged-forward state and its lifecycle; live removal of source items; exact Topic/SavedSublist destruction clearing; sender/caption option cycling and normalization; preview author/caption semantics; rich-message/premium constraints on hiding forwarding provenance; typed Topic/SavedSublist reply-draft clearing with delayed cloud save; and explicit link-preview option mutation.

The panel subscribes to the source history item-removal stream and the selected Topic/SavedSublist destruction stream. Source removal prunes only the removed staged item. Destination destruction clears the whole staged destination. Option changes are normalized before being written back through the destination owning history using both topicRootId and monoforumPeerId, so Topic and SavedSublist scopes cannot be conflated.

`ClearDraftReplyTo` clears only the matching reply relation, preserves unrelated draft content, and only deletes the draft when it becomes null; it then schedules canonical cloud draft save for the resolved typed thread. Link-preview options are staged inside the dialog and applied only on explicit Save.

## Existing-owner-first mapping

Fabushi must use the existing canonical owners:
- `ConversationChildRuntimeState` / canonical Conversation child identity;
- `MessagingEngine` / canonical Message lifecycle and authorization;
- existing ConversationWorkspace/Composer UI projection;
- existing ShareBox/Search recipient flow for destination discovery only.

No Telegram/source-shaped ForwardPanel or separate forward-draft store is allowed.

## Open production parity

Still open:
- one canonical destination-scoped forward staging model shared across Human/Group/Channel/Topic/SavedSublist;
- exact source-message removal and child-destruction convergence in shipping composition;
- sender/caption option normalization including rich-message/premium restrictions;
- typed reply-draft clear without deleting unrelated draft payload;
- renderer Composer/forward preview functional + accessibility evidence;
- paid-send/revalidation at the final send boundary;
- SavedSublist authoritative server relation/membership feed.

PR #46's Human-direct submit-time revalidation is valid narrow evidence only and does not satisfy these broader responsibilities.

## Traceability anchors

### `TDRP-R9-FORWARD-PANEL-CONTRACT-001`
- Requirement: `TDRP-R9-FORWARD-PANEL-CONTRACT-001`
- Oracle: `ORA-TDRP-R9-FORWARD-PANEL-CONTRACT-001`
- Invariant: `INV-TDRP-R9-FORWARD-PANEL-CONTRACT-001-CANONICAL`
- Release gates: `G-FILE`, `G-TRACEABILITY`, `G-PRODUCTION`, `G-COMPOSITION`, `G-EVIDENCE`

### `TDRP-R9-FORWARD-PANEL-LIFECYCLE-001`
- Requirement: `TDRP-R9-FORWARD-PANEL-LIFECYCLE-001`
- Oracle: `ORA-TDRP-R9-FORWARD-PANEL-LIFECYCLE-001`
- Invariant: `INV-TDRP-R9-FORWARD-PANEL-LIFECYCLE-001-STALE-FREE`
- Release gates: `G-FILE`, `G-TRACEABILITY`, `G-PRODUCTION`, `G-COMPOSITION`, `G-EVIDENCE`

## Accounting

Only these two exact accepted blobs receive new read credit:
- unknown: 15788 (unchanged)
- unread: 15737 -> 15735
- omitted: 0

No history.h/history.cpp, data_types.h, data_changes.h, mainwidget.*, draft-options.*, or other direct dependency receives read credit from this dossier.
