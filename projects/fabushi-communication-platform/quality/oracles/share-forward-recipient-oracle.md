# Share forward and recipient authorization oracle

Status: active / normative  
Parent: FQT-RTM-001 / TDRP-001 Revision 9

## ORA-TDRP-SHARE-FORWARD-PRIVACY-001

A forward request has one normalized privacy snapshot before any destination Message is persisted. `drop_captions=true` necessarily implies `drop_sender_names=true`; sender provenance and media captions are removed from canonical destination Message state, not hidden only in UI. Shipping Host code must call the existing source-neutral `fabushi_messaging_core::ForwardPrivacy::normalized()` implementation. The same request/client identity may replay only an identical normalized snapshot; changed privacy fails closed.

Required invariants:
- `INV-TDRP-FORWARD-CAPTION-IMPLIES-NO-SENDER-001`
- `INV-TDRP-FORWARD-PRIVACY-NATIVE-001`
- `INV-TDRP-FORWARD-PRIVACY-IDEMPOTENT-001`

## ORA-TDRP-SHARE-RECIPIENT-ELIGIBILITY-001

A destination can be emitted by canonical Search only after current authenticated Conversation/Community policy authorizes the requested send capability. Message, media and poll requirements are checked before indexing/exposure. Typed send-other, inline and game requests must also be authorized before exposure; if the canonical Conversation owner cannot represent a requested permission axis yet, Search fails closed rather than assuming eligibility. Protected source content is never exposed as forwardable, standard/secret content must target a compatible conversation kind, channel results additionally require current posting authority, and left/banned/restricted community state fails closed. Renderer hiding is never the authorization boundary. Shipping Host recipient discovery must pass the actual source message identity into this same source-neutral authorization function before returning any candidate.

Required invariants:
- `INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001`
- `INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001`
- `INV-TDRP-RECIPIENT-CHANNEL-POSTING-001`

Neither oracle is a release verdict. Exact-head execution evidence and the ledger verdict remain authoritative.

## ORA-TDRP-FORWARD-INTENT-UI-001 — immutable shipping picker intent

Inspected migration input: `59f5fa24827812f78336c464995d161f403dedfb`; fixed U remains
`f985723b552b30fe1c17d377d57f44e61a15a8c4`. This batch extends the existing
ProductionRenderer/ForwardMessageDialog and Session forward-batch journal;
the account-generation fence, durable Host recovery, permission checks and
canonical message/resource owners from the inspected input are preserved.

| Invariant | Required behavior / falsifiable boundary |
| --- | --- |
| FWD-UI-01 | The first dispatch freezes one normalized privacy snapshot, original unique destination set and request nonce. Partial or ambiguous failure retries only unconfirmed original destinations with the identical request identity and privacy. Object-only message refresh and search changes do not replace the intent. Neither new destinations nor changed options can enter an already attempted request. |
| FWD-UI-02 | Each requested destination requires exactly one explicit `sent` settlement before the picker confirms it. Missing, duplicate, conflicting or unexpected settlement rows cannot close the dialog as success or drop an unconfirmed destination. Search completion/failure cannot erase the independent send failure or outstanding retry state. |
| FWD-UI-03 | Before dispatch, selection is limited to the existing Host contract of 32 unique destinations. Rejecting the next selection exposes accessible feedback and leaves selection editable; no oversized immutable request is sent. The count includes selected recipients outside the current search results. |
| FWD-UI-04 | Hiding captions also hides sender names. Choosing to show sender names restores captions. These options remain editable before dispatch and immutable thereafter, matching the fixed-U forward-options state transitions. |
| FWD-UI-05 | Close retires presentation ownership and stale callbacks, including during a pending send; it does not cancel an accepted durable Host intent. Account switch/logout, source change and component unmount preserve the existing lifetime fence and prevent late results from mutating the current account/dialog. Reopening creates a new presentation intent; existing durable work remains recoverable by Session. |

Production path: transcript Forward action → existing picker → Coordinator
Session `forwardHumanMessage` → `ProductionSessionWorkers::forward_human_message`
→ existing source-conversation forward journal → destination pending rows →
authenticated native HTTP settlement. No new message store, public schema,
account authority or data migration is introduced. A normal code revert
preserves the existing journal and user data; it would restore these picker
limitations and is not evidence of final rollback acceptance.

The existing Chromium production-component fixture must cover these transitions
alongside all current account/lifetime/keyboard/light-dark/narrow-width cases.
Run the affected renderer and required exact-H workflow gates only in GitHub
Actions. Existing Host crash/restart/permission contracts remain the service-side
oracle. Browser fixtures, loopback HTTP and build success do not establish real
service, signed-package, complete accessibility/visual, Bot or independent ACCEPT.
