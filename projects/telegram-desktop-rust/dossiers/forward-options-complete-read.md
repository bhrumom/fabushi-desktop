# Forward options authority read — source completeness evidence

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f23c37857220eb84f8559f0901ea26fb304b564b`

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/ui/chat/forward_options_box.h` | `65db5fbe067e340e0259a23ae57cda59096ccda0` | complete, 682 bytes | product-state contract plus presentation declaration | open |
| `Telegram/SourceFiles/ui/chat/forward_options_box.cpp` | `708ae65e92dc435bf798f8c0220b0547f35ab2bf` | complete, 1,704 bytes | product interaction/state constraint plus presentation wiring | open |

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
- machine-readable responsibility `TDRP-R9-SHARE-FORWARD-PRIVACY-001` is implemented, not verified. The renderer action-level reverse transition ("show sender" restores captions), exact shipping forward-options controls, multi-recipient settlement, paid-send revalidation, and duplicate-submit fencing remain open.

## Coverage accounting

Durable movement supported by this read:
- `unread: 15784 -> 15782`
- `unknown: 15788` unchanged
- `omitted: 0` unchanged
- `baseline_ready: false`
- `acceptance.accepted: false`

These two entries remain unknown until all applicable forward-option responsibilities have bidirectional traceability, shipping wiring, exact-head tests/evidence, and release acceptance.
