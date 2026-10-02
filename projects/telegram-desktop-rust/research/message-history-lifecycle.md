# Telegram message/history lifecycle research dossier

Status: researched for the first FBCP message/history slice; broader Telegram capability research remains incomplete
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Target architecture snapshot: PR #20 `c2767eac1383fd8db7be9b536e5acaf4a4d2b7f0`
Capability: private messaging / message history lifecycle
Target rule: absorb behavior into existing Fabushi owners; do not carry MTProto or Telegram product ownership into runtime.

Snapshot refresh note: the exact `556f6308... → c2767eac...` PR #20 delta was inspected; it only moves Agent outline-stream identity/coalescing into shipping `RosterProjection`/`ProductionRosterEmit`, so the owner paths selected in this dossier remain valid at c276.

## Source evidence

The findings below were read from the frozen upstream commit, not inferred from current Telegram behavior.

- `Telegram/SourceFiles/api/api_sending.cpp:70-180`: send path derives reply/silent/scheduled/send-as flags, creates a random request/message correlation id, sends through the history owner, and routes failure to the message failure path.
- `Telegram/SourceFiles/api/api_sending.cpp:180-370`: media send creates a local message id and local optimistic history item before network settlement, registers random-id -> local-id correlation, and retries a stale file reference only after refreshing it and observing that the reference actually changed.
- `Telegram/SourceFiles/api/api_sending.cpp:425-705`: grouped media gives each local item its own local/random identity while retaining a group identity; a batch failure settles every local item; file-reference recovery is bounded to one refreshed retry path before failure.
- `Telegram/SourceFiles/api/api_editing.cpp:620-790`: edits preserve the original request identity across a file-reference refresh/retry and apply authoritative updates only through the completion callback.
- `Telegram/SourceFiles/data/data_messages.cpp:380-530`: history slices merge ordered message positions, maintain skipped-before/skipped-after counts, request missing ranges in both directions, and retain a nearest-to-anchor position for bounded history windows.
- `Telegram/SourceFiles/history/history_item.cpp:900-970`: client-side message ids are explicitly registered as client-side history entries instead of being treated as already-authoritative server ids.
- `Telegram/SourceFiles/history/history_item.cpp:1720-1770`: authoritative message data can trigger content refresh even without a normal edit update, and an edition that resolves to empty removes the item from history.
- `Telegram/SourceFiles/history/history_item.cpp:2340-2420`: edits update the existing history item in place while preserving or replacing media/reactions/views/forward metadata according to explicit flags.
- `Telegram/SourceFiles/storage/storage_account.cpp:1280-1680`: drafts are encrypted durable account state, keyed by conversation/draft scope; malformed or mismatched draft state is cleared instead of trusted; reply identity and cursor state are persisted separately.

## Behavior model extracted for Fabushi

### 1. Local intent precedes remote settlement

Telegram creates a local history identity and optimistic item before the remote send settles. Remote correlation is separate from the local display identity.

Fabushi requirement:
- composer submit creates one durable send intent before transport;
- the visible transcript may project a pending/queued entry immediately;
- transport acknowledgement must reconcile the existing entry, not create a second canonical message;
- a retry must retain the same logical message/client nonce unless the user explicitly creates a new message.

The current Human slice already has a useful seed: `sendHumanMessage` persists by `clientNonce` in the existing Session/Transcript SQLite owner and rejects nonce reuse with different content. This is local durability, not yet a complete network settlement protocol.

### 2. Identity and deduplication are first-class

Upstream separates local ids, random correlation ids and authoritative ids. That prevents retries/acks from becoming duplicate visible messages.

Fabushi requirement:
- `message_id`: stable canonical message identity;
- `client_nonce`: idempotency identity for a user send intent;
- `transport_attempt_id`: per-attempt diagnostic/cancellation identity;
- an eventual server/sync sequence is metadata, not the product identity;
- duplicate delivery of the same accepted event must be harmless;
- same nonce + different immutable send payload must fail closed.

Do not copy Telegram random-id or MTProto semantics directly; preserve the behavior using Fabushi-native identifiers/contracts.

### 3. Failure is state, not disappearance

Upstream retains local items and routes failed sends into a specific failure path.

Fabushi requirement:
- durable send state must distinguish at least pending / queued / accepted / failed;
- retry and delete/cancel operate on the same logical message intent;
- a process crash between local commit and network attempt must resume from durable state;
- failures must retain enough structured reason for UI and retry policy without embedding provider protocol errors as canonical product state.

### 4. Retry must be reason-specific and bounded

The observed media path retries stale file references only after refreshing the reference and proving it changed. It does not blindly replay every failure.

Fabushi requirement:
- retry policy is keyed by failure class;
- retry must be idempotent and cancellation-aware;
- refresh/re-resolve style retry is allowed only when the dependency version changed;
- terminal/permanent failures become a durable failed-send state;
- backoff/backpressure belongs below the existing composer/transcript product owner and must not freeze unrelated conversations.

### 5. History is an ordered, gap-aware projection

Telegram history slices track ordered positions, missing counts on either side and anchored pagination.

Fabushi requirement:
- native sync owns monotonic per-conversation ordering metadata;
- transcript remains the canonical product projection;
- clients detect gaps and request bounded missing ranges around an anchor;
- stale/replayed updates are deduplicated;
- large histories use bounded pagination and must not require full replay to open a conversation.

The existing Fabushi transcript pagination owner is the correct product/UI owner; native sync should feed it rather than replace it.

### 6. Authoritative edits mutate the same message identity

Upstream applies editions to an existing history item and can remove an item when the authoritative edition becomes empty.

Fabushi requirement:
- `MessageEdited` and `MessageDeleted` are native events referencing canonical `message_id`;
- edit/delete are not new chat messages;
- projections update/remove the existing transcript entry;
- edit acknowledgement and retries preserve operation identity and idempotency;
- reply/quote relations must remain resolvable or explicitly project a missing/deleted target.

### 7. Drafts are durable conversation-scoped state with corruption fencing

Upstream persists draft text, reply identity, webpage state and cursor separately and clears malformed/mismatched durable records.

Fabushi requirement:
- keep using the existing composer/draft owner;
- scope drafts by account + conversation/thread;
- preserve reply target and rich composer state;
- malformed state must be rejected/cleared according to an explicit migration policy, never silently reinterpreted as another conversation;
- multi-device draft convergence is a future native-sync responsibility, not a renderer-local second truth.

## Current Fabushi owner resolution

| Responsibility | Plausible owners considered | Selected existing owner | Reason |
| --- | --- | --- | --- |
| visible message/history truth | conversation workspace; Host Transcript; connector channels | existing transcript/message owner | already owns typed Human/Agent/tool/approval/artifact transcript projection and durable transcript storage |
| send intent from UI | composer; Runner send-message tool; connector delivery | existing composer/send action | Human and Agent user sends originate here; Runner tool send is Agent execution output, not Human transport ownership |
| durable local acceptance / idempotency | Session SQLite; Transcript runtime; new messaging store | existing Session/Transcript SQLite owner | current Human slice already uses it and avoids a parallel message store |
| history pagination | transcript pagination; new sync UI | existing transcript pagination owner | already models bounded conversation history in the mounted workspace |
| retry/network settlement | Session/Transcript; connector delivery; Coordinator; new broad CommunicationCore | existing Session/Transcript plus minimal native transport infrastructure below it | connector delivery is external-provider shaped; Coordinator is Agent orchestration; neither should own Human product truth |
| drafts | existing composer draft store; new Telegram-style draft store | existing composer/draft owner | already scoped to the conversation workspace |
| edit/delete projection | transcript/message lifecycle; new Telegram message model | existing transcript/message lifecycle | preserves one canonical message identity and typed Agent entries |

Rejected broad owner: `CommunicationCore`, `TelegramProvider`, `TelegramMessaging`, or a second message/history store. None is necessary for product ownership and each would duplicate existing transcript/conversation truth.

## Absorption changes still required

1. Introduce a Fabushi-native transport/sync contract below Session/Transcript that can accept the existing durable Human send intent without changing the canonical conversation/message model.
2. Persist transport settlement metadata and next-attempt/backoff state durably beside the existing message intent, with one-writer/idempotent transitions.
3. Define native ordered conversation events for `HumanMessage`, `AgentMessage`, `MessageEdited`, `MessageDeleted`, and `ReactionChanged`.
4. Add gap detection/recovery and bounded history-range fetch semantics that feed the existing transcript pagination owner.
5. Add reconnect/crash recovery so accepted-but-not-settled outgoing messages resume without duplicate visible entries.
6. Extend current Human conversation UI to reply/edit/delete/reaction only after those operations are represented by the same transcript/message identity.
7. Keep Agent-native entries (tool calls, approvals, tasks, artifacts, Computer handoff) typed; they are not flattened into generic HumanMessage wire payloads.

## Focused contracts required before this capability can be accepted

- same client nonce + same immutable payload is idempotent across process restart;
- same client nonce + different immutable payload fails closed;
- crash after durable acceptance but before first transport attempt resumes exactly one logical message;
- disconnect after remote acceptance but before local acknowledgement does not duplicate after reconnect;
- out-of-order and duplicate remote events converge to one ordered transcript;
- detected sequence gap requests only the missing bounded range and preserves the anchor;
- permanent send failure remains visible and retry reuses the logical message identity;
- cancellation fences future retry attempts;
- edit/delete retry is idempotent and targets the original message identity;
- corrupted durable send metadata fails closed or is recovered by an explicit migration path;
- large-history pagination does not require loading the full history;
- Human -> Agent handoff remains Coordinator -> Host -> Runner and its result remains a typed Agent-authored transcript entry in the same workspace.

## Current status

Research state for this capability: **researched, not accepted**.

The source evidence above is sufficient to define the first Fabushi message/history behavioral contract and confirms that the existing transcript/session/composer owners are the correct absorption targets. It does **not** prove Telegram-wide research closure, Fabushi native network implementation, sync/reconnect correctness, exact-head CI, packaged acceptance, or independent acceptance.
