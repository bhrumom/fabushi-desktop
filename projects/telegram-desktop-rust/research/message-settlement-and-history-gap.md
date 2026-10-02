# Telegram send settlement and history-gap research dossier

Status: researched and owner-resolved for Fabushi send/receive settlement and ordered history-gap recovery; implementation/acceptance remains open
Frozen upstream: telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677
Target architecture snapshot: PR #20 de0f1749a675729fb97017085374b96aaa2ca7bb
FBCP source snapshot inspected: PR #26 parent fc2c49a6ca4501444c4ba239f9788d35e46138e1

Snapshot refresh note: the exact `556f6308... → c2767eac...` PR #20 delta was inspected; it only moves Agent outline-stream identity/coalescing into shipping `RosterProjection`/`ProductionRosterEmit`, so the owner paths selected in this dossier remain valid at c276.

This dossier narrows the first message/history research into two executable P0 contracts: one logical outgoing message must survive transport settlement without duplication, and one ordered conversation history must recover gaps without creating a second history owner. Telegram identifiers and MTProto state are evidence only; Fabushi keeps its own identity, protocol, network, and product model.

## Frozen upstream evidence: send settlement

- Telegram/SourceFiles/api/api_sending.cpp creates a client-side message identity and an independent random correlation identity before the remote request settles. Local optimistic history is not remote acknowledgement.
- Telegram/SourceFiles/data/data_histories.cpp:1116-1173, Histories::sendPreparedMessage, serializes send through the existing history owner. Success applies authoritative updates with the correlation identity before the completion callback; failure uses a separate failure callback.
- Telegram/SourceFiles/api/api_updates.cpp:1560-1600, updateShortSentMessage, handles an authoritative message that may already be present before local correlation settles, then feeds the message-ID update and applies sent metadata.
- Telegram/SourceFiles/api/api_updates.cpp:1660-1710, updateMessageID, maps correlation to the local item and authoritative ID. If an authoritative duplicate already exists it is collapsed; otherwise the local item itself receives the real ID. Correlation state is then removed.
- Telegram/SourceFiles/apiwrap.cpp:515-643, sendMessageFail, removes correlation and marks the original local item failed for the normal failure path rather than silently dropping it.
- The existing message-history-lifecycle dossier records bounded, dependency-specific retry behavior for media/file-reference refresh; it is not an unconditional replay loop.

### Fabushi contract

Fabushi separates message_id (canonical logical message), client_nonce (idempotency for immutable user intent), transport_attempt_id (one network attempt/cancellation/tracing identity), and conversation sequence/cursor (ordering metadata only).

The existing Composer may paint a pending/queued entry immediately, but durable Session/Transcript acceptance owns whether the intent exists. Remote acknowledgement must reconcile that same logical entry. Remote echo before acknowledgement and acknowledgement before echo must converge to exactly one canonical transcript message.

Crash after durable local acceptance but before first network attempt resumes the same client_nonce/message_id. Disconnect after remote acceptance but before local acknowledgement must not create a second logical send. Same client_nonce plus a different immutable payload fails closed.

Failure is durable state. Retry keeps the logical message identity and creates a new transport_attempt_id only when policy permits it. Permanent failure stays visible until explicit retry/removal/cancel.

## Frozen upstream evidence: ordering and gap recovery

- Telegram/SourceFiles/data/data_history_messages.cpp:28-54, HistoryBaseMessagesSlice, carries full count, skipped-before, skipped-after, ordered IDs and nearest-to-anchor information.
- Telegram/SourceFiles/data/data_history_messages.cpp:57-193 merges client-side messages into the ordered slice without duplicating existing IDs, respects known skipped boundaries, and orders server/client identities deterministically.
- Telegram/SourceFiles/data/data_history_messages.cpp:300-390, HistoryViewer, requests additional history when a sparse slice reports an insufficient range instead of pretending the local window is complete.
- Telegram/SourceFiles/data/data_history_messages.cpp:430-520, HistoryMessagesViewer, rebuilds a bounded slice and rewrites provisional IDs when authoritative IDs arrive.
- Telegram/SourceFiles/api/api_updates.cpp:1498-1528 buffers update batches whose sequence starts after the expected next sequence and starts skipped-update recovery instead of applying later state across an unknown gap.
- The same updates path falls back to difference recovery when prerequisites are missing or the update stream cannot be safely applied incrementally.

### Fabushi contract

The existing transcript remains product truth and the existing pagination controller remains the workspace history owner. Native sync supplies ordering metadata and missing ranges below those owners.

Per conversation, native sync needs a last contiguous applied sequence/cursor, replay-dedupe identity, bounded detected gap descriptor, and recovery generation/request identity so stale range responses cannot overwrite newer state.

A later event does not silently advance contiguous state across an unresolved earlier gap. Recovery asks for the bounded missing range around the current history window/anchor. Duplicate/stale events are harmless. Restart restores enough sync metadata to continue recovery without replaying the full history.

The product message_id remains stable across local-to-authoritative settlement. Sequence/cursor changes do not rewrite product identity.

## Current Fabushi owner evidence

### Send settlement

Plausible owners inspected: Composer/submission queue, renderer acknowledgement overlay, Host Session/Transcript durable store, Host Transcript send pipeline, Coordinator, and a new message store/CommunicationCore.

Selected owner: existing Session/Transcript durable owner for canonical Human message acceptance; existing Composer/submission and acknowledgement code for optimistic UI projection; only minimal Fabushi-native transport/sync below them.

Exact paths:
- source/host/src/extensions/session/production.rs — append_human_message already validates the authenticated local participant, enforces clientNonce idempotency/conflict fencing, and writes the Human transcript.
- source/host/src/extensions/session/agent_db.rs — durable conversation store used by Session.
- source/host/src/gateway_server.rs — explicit shipping Human command boundary; no broad communication namespace.
- frontend/src/recovered/features/conversation/workspace/submission.ts — pending/queued/failed/sent queue and reconnect flush semantics.
- frontend/src/recovered/features/conversation/workspace/acknowledgement.ts — optimistic pending/queued/dispatching/accepted-awaiting-echo/failed projection and echo reconciliation.
- frontend/src/production/ProductionRenderer.tsx — composes Human send into the same workspace/transcript.
- source/host/src/extensions/transcript/send_pipeline.rs and send_acceptance.rs — existing Agent-send admission/idempotency behavior to reuse where appropriate, not a second Human product truth.

Current gap: append_human_message writes delivery = sent at local durable acceptance. That does not distinguish native remote settlement. Renderer acknowledgement/submission state is not a durable network settlement ledger. Native transport attempt/ack state is absent.

### Ordered history and gaps

Plausible owners inspected: existing transcript pagination, Session SQLite transcript pages, Transcript projection, and a new sync/history store.

Selected owner: existing Session/Transcript durable history plus existing transcript pagination; minimal native sync below them.

Exact paths:
- frontend/src/recovered/features/conversation/workspace/pagination.ts — bounded beforeSeq pages, duplicate-ID merge, stale-request generation fences, and viewport anchor restoration.
- source/host/src/extensions/session/agent_db_transcript_pages.rs — durable seq-backed transcript pages and next_before_seq cursors.
- source/host/src/extensions/session/production.rs — current Human conversation transcript owner.
- frontend/src/production/ProductionRenderer.tsx — mounts the single transcript workspace.

Current gap: the Human workspace hard-disables older-history loading (hasOlder false and loadOlder undefined for active Human conversations), and there is no native receive stream with persistent sequence/gap recovery.

## Minimal durable state required before implementation

Outgoing settlement record:
- conversation_id, message_id, client_nonce, immutable payload digest;
- locally accepted/queued/dispatching/remotely accepted/failed/canceled state as applicable;
- current or last transport_attempt_id;
- bounded retry class/count/next-attempt time;
- remote acknowledgement/event identity;
- UI-safe failure class and timestamps.

Receive/sync record:
- conversation_id;
- last contiguous applied sequence/cursor;
- authoritative checkpoint if the native service contract distinguishes it;
- gap descriptor/missing range;
- recovery generation/request identity;
- dedupe identity for applied remote events.

These records support existing transcript ownership. They are not a second message/history store.

## Focused contracts

Send settlement:
- same client_nonce + same immutable payload replays one logical message across restart;
- same client_nonce + different immutable payload fails closed;
- crash after local acceptance but before first attempt resumes exactly one intent;
- disconnect after remote acceptance but before local acknowledgement does not duplicate;
- echo-before-ack and ack-before-echo converge identically;
- duplicate acknowledgement/event is harmless;
- reason-specific retry changes only transport attempt identity;
- permanent failure stays visible;
- cancel fences future attempts.

Ordering/gap:
- duplicate/stale remote events do not duplicate transcript entries;
- event N+2 before N+1 records a gap instead of silently advancing;
- bounded recovery fills only the missing range and releases later buffered state in order;
- stale recovery response from an older generation is ignored;
- restart with an unresolved gap resumes recovery;
- local optimistic messages remain deterministic in a bounded window;
- provisional-to-authoritative ID settlement rewrites references without duplicate history;
- large-history pagination stays bounded and preserves viewport anchor.

Integration/E2E:
- Human send -> durable local acceptance -> native service attempt -> remote acknowledgement -> same transcript entry;
- forced disconnect between remote acceptance and local acknowledgement -> reconnect -> one message;
- duplicate/out-of-order receive events -> one ordered transcript after gap recovery;
- restart while outgoing settlement or receive gap is unresolved -> resume under the same Session/Transcript owner;
- Human -> Agent handoff remains Coordinator -> Host -> Runner in the same workspace.

## Blockers and non-claims

Owner resolution is complete for these two rows, but implementation is not. Current blockers are no Fabushi-native remote messaging/sync service contract wired to Human Session, no durable Human transport-attempt/ack ledger, no persistent receive cursor/gap recovery, Human older-history loading disabled, and no exact-head packaged evidence.

Rejected ownership: TelegramProvider, TelegramMessaging, CommunicationCore, MessengerRuntime, a second message/history store, and a second Human workspace/sidebar.
