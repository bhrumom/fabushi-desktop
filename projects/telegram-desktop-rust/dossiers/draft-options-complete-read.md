# history_view_draft_options.h/.cpp complete read

- Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`
- Header: `Telegram/SourceFiles/history/view/controls/history_view_draft_options.h@d42c352af7f6949e546585427ece86987563def2` — complete, 48 lines
- Implementation: `Telegram/SourceFiles/history/view/controls/history_view_draft_options.cpp@9dabfb109125f82f5b78b0103a2fe65b79e27f8d` — complete, 1516 lines
- Status: fully read/decomposed; mapped/open, not implemented or verified

## Complete responsibility decomposition

This pair is not presentation-only. It owns one staged options state machine spanning reply/quote, forwarding, and link-preview edits. It exposes only valid sections, closes when all staged inputs disappear, normalizes forwarding options while displaying and again on submit, and rebuilds when source items disappear or their views refresh.

Forward options preserve sender/caption, hide sender names, or hide names+captions. Availability is constrained by source items, rich-message/premium policy, and sender/caption counts. A premium-required hide-author operation is not silently accepted. Changing recipient exports the exact surviving source message ids and normalized options into the canonical recipient flow.

Reply transfer is typed. `ShowReplyToChatBox` resolves the chosen destination `Data::Thread`, rewrites both `topicRootId` and `monoforumPeerId`, preserves destination text/cursor state, writes the local draft, clears only the matching edit draft, emits `LocalDraftSet`, then schedules `clearOldDraft` on the session owner. Topic and SavedSublist identities therefore cannot be conflated, and old-destination cleanup happens only after the new typed draft exists.

The box also owns live lifecycle convergence: exact source-item removal clears the reply or prunes only that forward item; when no reply/link/forward remains it closes. Enter/Return submits through the same normalization path. Link preview mutations are staged and committed only through the explicit finish callback.

## Existing-owner-first mapping

Reuse canonical `ConversationChildRuntimeState`, `MessagingEngine`, ConversationWorkspace/Composer, and ShareBox/Search recipient authorization. No source-shaped DraftOptions/ForwardPanel store or second Composer owner is allowed.

## Open production parity

Still open: one destination-scoped forward/reply staging model across Human/Group/Channel/Topic/SavedSublist; exact source-message removal and destination-child destruction convergence; sender/caption normalization including rich-message/premium restrictions; paid-send/revalidation for every destination kind; typed reply transfer/old-draft cleanup preserving unrelated draft payload; renderer keyboard/accessibility evidence; and the authoritative SavedSublist server relation/membership feed.

PR #46 remains only narrow Human-direct submit-time revalidation evidence and does not satisfy this pair.

## Traceability anchors

- `TDRP-R9-DRAFT-OPTIONS-CONTRACT-001` / `ORA-TDRP-R9-DRAFT-OPTIONS-CONTRACT-001` / `INV-TDRP-R9-DRAFT-OPTIONS-CONTRACT-001-CANONICAL`
- `TDRP-R9-DRAFT-OPTIONS-LIFECYCLE-001` / `ORA-TDRP-R9-DRAFT-OPTIONS-LIFECYCLE-001` / `INV-TDRP-R9-DRAFT-OPTIONS-LIFECYCLE-001-STALE-FREE`
- Gates: `G-FILE`, `G-TRACEABILITY`, `G-PRODUCTION`, `G-COMPOSITION`, `G-EVIDENCE`

## Accounting

Only these two exact accepted blobs receive new read credit:
- unknown: 15788 (unchanged)
- unread: 15735 -> 15733
- omitted: 0

No `history.h/.cpp`, `data_types.h`, `data_changes.h`, `mainwidget.*`, or recipient-controller dependency receives read credit.
