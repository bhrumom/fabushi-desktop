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

A destination can be emitted by canonical Search only after current authenticated Conversation/Community policy authorizes the requested send capability. Message, media and poll requirements are checked before indexing/exposure; channel results additionally require current posting authority, and left/banned/restricted community state fails closed. Renderer hiding is never the authorization boundary.

Required invariants:
- `INV-TDRP-RECIPIENT-AUTH-BEFORE-EXPOSURE-001`
- `INV-TDRP-RECIPIENT-MEDIA-POLL-ELIGIBILITY-001`
- `INV-TDRP-RECIPIENT-CHANNEL-POSTING-001`

Neither oracle is a release verdict. Exact-head execution evidence and the ledger verdict remain authoritative.
