# Forward options authority read — source completeness evidence

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f23c37857220eb84f8559f0901ea26fb304b564b`

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/ui/chat/forward_options_box.h` | `65db5fbe067e340e0259a23ae57cda59096ccda0` | complete, 682 bytes | product-state contract plus presentation declaration | open |
| `Telegram/SourceFiles/ui/chat/forward_options_box.cpp` | `708ae65e92dc435bf798f8c0220b0547f35ab2bf` | complete, 1,704 bytes | product interaction/state constraint plus presentation wiring | open |
| `Telegram/SourceFiles/history/view/controls/history_view_forward_panel.h` | `4e01059c1b0f0703f99817b8ec6028133ffd3728` | complete, 2,672 bytes | forward-draft option/lifecycle contract plus presentation declaration | open |
| `Telegram/SourceFiles/history/view/controls/history_view_forward_panel.cpp` | `ea88a830e327705ff29c67514fea5b2215844831` | complete, 13,764 bytes | privacy eligibility/normalization, draft lifecycle and preview semantics | open |
| `Telegram/SourceFiles/chat_helpers/share_message_phrase_factory.h` | `86865db47439dbce848fc812df2554335d3e004c` | complete, 911 bytes | forward-success feedback contract | open |
| `Telegram/SourceFiles/chat_helpers/share_message_phrase_factory.cpp` | `1437bbfbd86b9d6649faae0bed7eed755e06337e` | complete, 2,331 bytes | destination-count/self-aware success feedback and navigation behavior | open |

## Transferable responsibilities

The upstream `ForwardOptions` contract carries four pieces of state:
- sender count;
- caption count;
- whether sender provenance/name is dropped;
- whether captions are dropped.

The interaction contract couples the privacy options:
- when sender names are turned back on while captions are hidden, captions are restored;
- when captions are hidden while sender names are still shown, sender names are also hidden;
- option changes are emitted through one canonical state object rather than independent UI-only booleans.

The checkbox/widget implementation itself is presentation and is not ported. The transferable responsibility is the option dependency/privacy semantics and their effect on the canonical forward operation.

The complete ForwardPanel dependency read adds further non-presentation requirements:
- privacy options are normalized against source-item eligibility; rich-page/forced-forward-info cases can force `PreserveInfo` instead of trusting an arbitrary UI toggle;
- forward draft state is tied to a typed destination thread and is cleared when the topic/sublist destination is destroyed;
- item removal mutates the live forward draft rather than forwarding stale source items;
- the option cycle is PreserveInfo -> NoSenderNames -> NoNamesAndCaptions (when captions exist) -> PreserveInfo.

The complete share-message phrase factory read adds terminal feedback semantics:
- saved/self, one destination, exactly two destinations, and many destinations have distinct success projections;
- saved-message success can navigate to the canonical self conversation;
- feedback is a post-settlement projection and must not be emitted before all applicable destination sends settle.

## Fabushi disposition

- keep forwarding in the existing canonical Message/MessagingService/MessagingEngine owners;
- do not create a Telegram/ShareBox forwarding owner;
- represent forward privacy/options as typed canonical command/message policy, with native revalidation rather than trusting renderer state;
- ensure multi-recipient fan-out applies the same option snapshot to every destination or settles failures explicitly;
- ensure retries are idempotent by actor-scoped client message identity.

Current production progress:
- `b598bd9f98903f245842ac81547bd6d5bfdf0acc` makes forward retries use the canonical actor-scoped stable message identity and service-level replay/conflict handling;
- `e4494def67715379e619ba5882078347c00c08e8` adds focused replay and option-conflict regression coverage;
- `633d54b0a393b9ee042266d85e1ccbd9fc661300` adds source-neutral `ForwardPrivacy` and canonical media-caption clearing;
- `7b52b7218a0d34aa64d5203af7a8dbfced304e48`, `08b7a6f6cc14606d04f2e1c242179948a12e384e`, and `1430809599ab2f5a5e20c993ed84541b1d9d895c` carry that policy through protocol/service/engine, normalize `dropCaptions => dropSenderNames`, materialize hidden captions/provenance in the canonical destination Message, and include privacy in replay conflict checks;
- `24523945ecf607e9546cebabb1b98bfb651494ec` and `21ab7870a3ba3129172942d722befb83c321f413` add focused coupled-privacy and changed-privacy replay tests;
- machine-readable responsibility `TDRP-R9-SHARE-FORWARD-PRIVACY-001` is implemented, not verified. The newly read ForwardPanel authority shows that source-item eligibility can force PreserveInfo and that destroyed topic/sublist destinations or removed source items must invalidate the live draft; the share-message phrase authority also requires destination-count/self-aware terminal feedback. Those eligibility/lifecycle/feedback semantics, exact shipping forward-options controls, multi-recipient settlement, paid-send revalidation, and duplicate-submit fencing remain open.

## Coverage accounting

Durable movement supported by the exact source reads recorded in this dossier:
- `unread: 15779 -> 15775` for the four newly completed dependency entries (global movement; the original forward-options pair had already been credited earlier)
- `unknown: 15788` unchanged
- `omitted: 0` unchanged
- `baseline_ready: false`
- `acceptance.accepted: false`

All six entries recorded here remain unknown until all applicable forward-option, eligibility, lifecycle, feedback and settlement responsibilities have bidirectional traceability, shipping wiring, exact-head tests/evidence, and release acceptance.
