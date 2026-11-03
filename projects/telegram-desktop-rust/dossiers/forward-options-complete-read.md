# Forward options authority read — source completeness evidence

## Fixed-U forward intent review — 2026-11-02

U stays `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4`, tree
`91f2b083ab2877c0a17d07868eabec362a5cfca6`. Current integration input is
`94dcf7a2213808a8ded6baddaf069562d365c6d7`; the unchanged picker and shared
review contracts were reviewed against `59f5fa24827812f78336c464995d161f403dedfb`
and `43d70a62d6fd47785f15b85a70e9e3350a17cef4` respectively. New Host attachment
snapshots and journal recovery are preserved. Historical counters, upstream and
owner statements below are retained as history, not current acceptance.

All six complete file bodies below were read at fixed U. The unique ledger
records two complete source reviews with an existing exact responsibility
mapping and four pending reviews whose additional business obligations have
not been fully decomposed/mapped. Complete reading is not complete product
implementation, legal review, source closure or independent acceptance.

| Path relative to `Telegram/SourceFiles/` | Exact blob | Current review record |
| --- | --- | --- |
| `ui/chat/forward_options_box.h` | `65db5fbe067e340e0259a23ae57cda59096ccda0` | complete: `ForwardOptions` state/callback contract maps to FORWARD-PRIVACY-001 |
| `ui/chat/forward_options_box.cpp` | `708ae65e92dc435bf798f8c0220b0547f35ab2bf` | complete: both privacy transition directions and callback lifetime map to the same responsibility |
| `history/view/controls/history_view_forward_panel.h` | `b5847bfe6e651f5e3bc52480192afd664a37c8b8` | pending: entire file read; full draft/thread/reply responsibility mapping remains open |
| `history/view/controls/history_view_forward_panel.cpp` | `eaa0108dd32dbadc21727ed4583c329d3badc85e` | pending: entire file read; forced provenance, source removal, typed destination, reply/suggestion/cloud-save obligations remain open |
| `chat_helpers/share_message_phrase_factory.h` | `86865db47439dbce848fc812df2554335d3e004c` | pending: entire file read; precise terminal feedback responsibility remains open |
| `chat_helpers/share_message_phrase_factory.cpp` | `1437bbfbd86b9d6649faae0bed7eed755e06337e` | pending: entire file read; saved/self/premium/one/two/many feedback and navigation are not fully mapped |

`FillForwardOptions` emits one state snapshot: hiding captions also hides names;
showing names restores captions. Names/captions controls share the supplied
lifetime. Fabushi already reuses `ForwardPrivacy::normalized` in the actual
Session path. ORA-TDRP-FORWARD-INTENT-UI-001/FWD-UI-01..05 adds the missing
reverse UI transition and freezes this normalized snapshot across partial or
ambiguous retries in the existing picker. Remote source provenance and source
eligibility equivalence remain separate open requirements.

The panel read establishes further obligations which this UI batch does not
close: item removal filters the live draft; topic/sublist destruction clears
it; options normalize against forced forward-info, rich pages and author
capability. `ClearDraftReplyTo` preserves the exact thread address, clears the
matching reply and suggestion, removes a null draft or saves the surviving
draft, and schedules cloud save only when its thread exists. These obligations
remain explicitly pending instead of being credited to the narrow privacy row.
The phrase factory varies settled feedback for saved/self/premium, one, two and
many recipients; saved-message navigation resolves the session's window.
Generic recipient counts do not establish complete equivalence.

Scoped callers/consumers were also read, without claiming complete-file reads:
`boxes/share_box.cpp` (`f4787dbb34cdb5fb2af78a69ae7537e91e06395a`) default-forward
and submit callbacks; `apiwrap.cpp` forwarding requests/random IDs and settlement;
`history/history.cpp` forward-draft resolution; `history/history_item_helpers.cpp`
source restrictions; compose controls, draft options and peer-menu callsites.
Those reads identify source fences; Fabushi's durable journal/restart behavior
is independently qualified by the existing Host contracts, not inferred from
upstream random IDs.

Current production: Transcript → ProductionRenderer/ForwardMessageDialog →
Session gateway → `ProductionSessionWorkers::forward_human_message` →
source-conversation `HumanForwardBatch` journal → destination pending/settled
SandAgentDb rows → authenticated Fabushi native service. Existing account
lifetime, resource IDs, permissions, Agent machinery and legacy-data retention
are preserved. FeatureHost and old Telegram paths cannot be treated as this
desktop action's message authority. The previous interrupted local snapshot
was not overlaid onto the newer remote Host/schema/lock implementations.

Execution and remaining gates are recorded in
`projects/telegram-desktop-rust/evidence/task20bf-round1-forward-intent-ui.json`.
All executable validation is GitHub Actions at the containing H.

## Historical source review

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@0a11537c220df35fe59a96aebe413b1e52bc6e47`
Rebaseline inheritance: source entries cited below retain the same blob/content hashes on `0a11537c220df35fe59a96aebe413b1e52bc6e47`; read status is inherited by blob identity, while execution evidence must be reacquired on the current Fabushi HEAD.

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


## Exact-U forward callgraph supplement — 2026-11-03

The four pending source files above have now been read in full again at the fixed U, and their production consumers have been read at the same U. The source-review records are marked complete for these exact file identities and mapped to existing responsibilities; the feature-level gaps below remain open.

### ForwardPanel owner and state transitions

- `ForwardPanel::update(to, draft)` compares destination thread, source item list and options; when changed it destroys the prior reactive lifetime, installs the new draft, subscribes to source-item removal, and clears the draft when its typed topic or sublist destination is destroyed. It publishes `itemsUpdated` after the update.
- `itemRemoved` removes only the matching source item, refreshes the derived preview and publishes the update. It does not itself dispatch a send.
- `applyOptions` first calls `NormalizeForwardOptions`, writes the normalized options and exact item IDs back to the owning History forward draft, then repaints. `editToNextOption` cycles PreserveInfo → NoSenderNames → NoNamesAndCaptions when captions exist → PreserveInfo.
- `updateTexts/checkTexts` derives sender labels from original sender/hidden-sender metadata, de-duplicates known senders, and selects a version from the name/caption options and sender-name versions. It uses the first source item for the single-message preview, hides captions when requested, and drops disallowed custom emoji when provenance is hidden.
- `CanHideForwardAuthor` refuses hiding for forced-forwarded information and for rich pages when the account cannot author rich messages; rich-page hiding also depends on premium. `HideForwardAuthorPremiumRequired` identifies the case where the control should show a premium prompt rather than allow a privacy change. `NormalizeForwardOptions` is the final source eligibility clamp.
- `ClearDraftReplyTo` is a separate existing reply-draft operation: it only clears the matching reply identity, resets suggestion state, preserves the rest of the draft, removes a now-null local draft or writes the surviving draft, then schedules cloud save for the same thread.

### Actual caller and send path

- `boxes/share_box.cpp`: selected recipient rows are resolved to typed `Data::Thread` objects. The default forward callback reloads source items, refuses empty sources/targets, normalizes rich-page options, checks send eligibility and paid-send approval, then maps the one normalized privacy snapshot to MTProto `f_drop_author` and `f_drop_media_captions` flags for every destination. Each destination can have its own request set; the final toast is delayed until all requests settle. Any destination failure suppresses the all-success toast. Comment text is dispatched through the existing ordinary send path separately from forwarded messages.
- `apiwrap.cpp`: the shared `ApiWrap::forwardMessages` owner normalizes options again before building forward requests. It maps the canonical option to the same author/caption flags and only creates a local optimistic message for the single-message PreserveInfo case. Saved-music items use the existing document send path rather than the forward request.
- `history_view_draft_options.cpp`: the forward selector preview normalizes before projecting local preview messages and carries the resulting privacy bits into the preview. The forward/reply/link draft editor re-normalizes on entry and before committing a selected option; the hide/show controls respect source eligibility and premium restrictions.
- `window_peer_menu.cpp`: direct Forward-to-Saved-Messages resolves the existing forward draft, dispatches through `ApiWrap::forwardMessages`, then uses the shared phrase factory and saved-message navigation filter after success.
- `history_view_top_peers_selector.cpp`: quick-forward completion uses the same phrase factory and saved-message navigation filter.
- `history_view_self_forwards_tagger.cpp`: self-forward tag feedback deliberately sets `toSelfWithPremiumIsEmpty=false`, so its UI does not inherit the ordinary premium self-toast suppression.
- `chat_helpers/share_message_phrase_factory.cpp`: one destination names the peer, two destinations name both peers, larger fan-out uses a count, and self targets link to Saved Messages. The optional premium flag suppresses the normal self-success phrase; the icon and click filter are separate projections.

### Source-to-responsibility mapping and remaining gaps

- `history_view_forward_panel.h/.cpp` map to existing send-options, typed thread-destination, topic-eligibility and privacy responsibilities. They are source review complete; they do not prove Fabushi shipping parity for every topic/sublist, source-removal or reply-draft/cloud-save edge.
- `share_message_phrase_factory.h/.cpp` map to the existing FBCP forwarding and send-options responsibility. They are source review complete; the exact saved/self/premium feedback, one/two/many destination wording and session-window navigation still need product-side responsibility and exact-H behavior acceptance.
- The source UI and API normalize privacy at multiple boundaries, but source reads do not prove live service behavior, partial fan-out recovery, destination authorization freshness, or that a presentation toast is only shown for the exact settled destination set.
- The Fabushi production owner remains the existing Session/Transcript + durable HumanForwardBatch journal + authenticated Native service. Do not recreate Telegram's ShareBox, MTProto request state, or toast owner. Reuse current Host policy and durable batch recovery, then project truthful settled feedback through the existing renderer.

### Exact fixed-U caller identities read

| Caller | Blob |
|---|---|
| `Telegram/SourceFiles/boxes/share_box.cpp` | `f4787dbb34cdb5fb2af78a69ae7537e91e06395a` |
| `Telegram/SourceFiles/window/window_peer_menu.cpp` | `864a8103df594c634cb1017c7bf2733da13885f2` |
| `Telegram/SourceFiles/apiwrap.cpp` | `342e501177996be3c083ba7985d3720a53100b51` |
| `Telegram/SourceFiles/history/view/controls/history_view_draft_options.cpp` | `5bdc9891bb130433a30cf4add1c72bb0cfb973b9` |
| `Telegram/SourceFiles/history/view/history_view_top_peers_selector.cpp` | `2c452f7258228e319c58bdc703f8883253c77981` |
| `Telegram/SourceFiles/history/view/history_view_self_forwards_tagger.cpp` | `e3cb5d4e920f205c6b0d553359eb2c9995c94943` |

These are exact-U caller blobs and scoped reads, not claims that every caller file was fully read. The four reviewed source files are complete at their exact recorded blobs; recursive source, build-time, generated, resource, tooling and legal closure remain independently unaccepted.
