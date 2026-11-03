# Native Human known-message state during reconnect — fixed-U research

## Identity, scope and status

- Upstream repository: `telegramdesktop/tdesktop`.
- Frozen upstream U: `f985723b552b30fe1c17d377d57f44e61a15a8c4`; root tree: `91f2b083ab2877c0a17d07868eabec362a5cfca6`.
- Target comparison H: `62a3fc19501a5c6f5e25495c833120ae67b973e6` in `bhrumom/fabushi-desktop`. This is the exact integrated parent supplied by the integrator, not a claim about a later branch head.
- Observation date: 2026-11-02. Files were fetched through the GitHub connector at the full U/H; no moving `dev` or default-branch source was substituted.
- Work type: source research and production-call-path inspection only. No production implementation, schema/ledger/lock/STATUS change, executable validation, service call, or source/release ACCEPT is claimed. The message-contract Actions gate remains a prerequisite for implementing its dependent batch.

The bounded finding is that message identity deduplication and message-state refresh are separate responsibilities. At H, the existing Host materializer can durably refresh a known Human message, but reconnect enumeration discards the known rows before reaching that materializer. Reactions and `readAt` can therefore remain stale. A second, independent risk is that an individually settled outbound message can be mistaken for proof that all earlier inbound history was synchronized.

## Exact source identities and reading boundaries

Paths in this table are relative to `Telegram/SourceFiles/`. “Full” means the fetched source text was read in its entirety; “partial” gives the source ranges actually examined for this responsibility. Downloading or symbol-searching a file is not treated as reading its other behavior.

| Path | Git blob at U | Reading boundary |
| --- | --- | --- |
| `data/data_history_messages.cpp` | `412f30c67ae3d39442a38dce66ae123d864ab4ac` | Full, lines 1–518; history slices, local-message merge, anchors, request and notification wiring |
| `data/data_pts_waiter.cpp` | `e0df5a8f8aff2c2667ce11186b20c00ffdf4b1e9` | Full, lines 1–188; PTS ordering, skipped queues, duplicate suppression and recovery timers |
| `data/data_pts_waiter.h` | `e35309aa7b6af5030f52d5e866f04cf660d44bdb` | Full, lines 1–105; state initialization, requesting state and timeout contract |
| `data/data_sparse_ids.cpp` | `1f11a0424294023ee486a87e1ba194cabd9bd87a` | Full, lines 1–429; sparse ranges, unknown counts, merge, limits and insufficient-range requests |
| `api/api_updates.cpp` | `71c929cfe3c779c94719efef32a47a4d029e2c67` | Partial: 1–394, 450–530, 568–723, 888–980, 1216–1890, 2950–2970; remainder only indexed for relevant symbols |
| `data/data_session.cpp` | `baac09c9c320d748d008867dc4ec3ec560c6681b` | Partial: 3020–3125 and 3570–3635; existing/new/edited message admission and application |
| `history/history_item.cpp` | `b753d3ea7bd5fde15cdad206d17ab3eee6d68cab` | Partial: 1880–1950, 2384–2442, 2585–2665, 2780–2882, 3218–3388, 5465–5590, 8438–8478; reaction/read mutation, local-to-server identity transition and adjacent edit/TTL boundaries |
| `history/history.cpp` | `27c0d3c3f9f8668a497c5ab638135972380bfd6b` | Partial: 2255–2335 and 3805–3852; inbox/outbox application and monotonic read watermarks |
| `data/data_message_reactions.cpp` | `ce05de5fadc5caf73bf3cf31f4a7d35a4e0b63dc` | Partial: 1700–1785 and 2110–2340; polling application, unknown unread reactions, pending-send fence, replacement/removal and partial snapshots |
| `apiwrap.cpp` | `342e501177996be3c083ba7985d3720a53100b51` | Partial: 1–120, 3610–3695, 3720–3810; history request wrapper, deduplication and failure cleanup. `PrepareHistoryRequest`/`ParseHistoryResult` implementations were not resolved/read in this investigation. |
| `mtproto/scheme/api.tl` | `8319b7df634d145390c299404129fd161a5dbd2a` | Declaration-only: 344–345, 347, 351, 365, 382, 419, 497–500, 506–507, 1527, 1529, 2595 and 2856. No generated parser or generator execution/qualification claim. |

Ancillary name-resolution lookups do not add responsibility coverage: `api/api_common.h@82b21c9bcdac121164138c4f007a48058041af67` and `api/api_media.cpp@b72ba499edae67e486c7e66685362e3cf2d292c8` were fully read but concern adjacent send/media contracts; `apiwrap.h@ed870e4a57e6b363b3c13fd9d77bb760a03e40fe` and `api/api_messages_search.cpp@ac348199bc358ee9e8d203fb95c5d0c4fdc30354` were inspected only for includes/names. `data/data_messages.h@b5919e53bc7bf07a4e243b114205765a40ed9b18` and `data/data_messages.cpp@218f879233904a806638f92acdc0dc6947992809` were fetched and symbol-searched without a relevant match; their product behavior was not reviewed here.

The upstream file headers retain their existing LEGAL reference. This dossier does not re-audit the complete legal inventory, approve code copying, decrement global unknown/unread counters, or relabel historical-U dossiers as current-U reviews.

## Source behavior and responsibility boundaries

### 1. A known message remains a live state owner

In `api_updates.cpp:1812–1833`, `updateMessageReactions` selects the loaded history by the update's peer, then looks up the existing item by that peer plus `msg_id`. When the item exists it calls `HistoryItem::updateReactions`; identity equality does not discard the update. If the item is absent but its history is loaded, unread-reaction information can request a dialog refresh and register the unknown unread message ID. It does not manufacture an unrelated message or guess a different conversation. [Exact dispatcher](https://github.com/telegramdesktop/tdesktop/blob/f985723b552b30fe1c17d377d57f44e61a15a8c4/Telegram/SourceFiles/api/api_updates.cpp#L1812-L1833).

`HistoryItem::updateReactions` (`history_item.cpp:5482–5499`) compares/applies reaction state, returns on a no-change result, updates unread-reaction membership in both directions, schedules applicable reaction notifications, and notifies the existing item's data consumers. `MessageReactions::change` (`data_message_reactions.cpp:2179–2328`) updates counts and the local selected flag, removes reactions absent from the supplied result, and replaces recent reactors. This is state replacement with explicit partial-data handling, not append-only reaction accumulation.

Two qualifiers matter when translating that contract:

- A `min` reaction snapshot preserves known local selection where appropriate, and a min snapshot cannot blindly replace an item's unread-reaction state (`history_item.cpp:5539–5550`; `data_message_reactions.cpp:2195–2213`).
- While a local reaction request is outstanding, `MessageReactions::checkIfChanged` and `change` return without applying the incoming snapshot; the comment identifies the pending request response as the non-stale source (`data_message_reactions.cpp:2119–2127,2179–2187`). A late history response must not silently undo a newer settled local reaction.

There is also an existing-message application path for incoming new-message envelopes: `applyUpdateNoPtsCheck` calls `Session::updateExistingMessage` before deciding whether insertion is necessary (`api_updates.cpp:1321–1337`); `Session::processMessages` does likewise for unread messages (`data_session.cpp:3076–3099`). `updateExistingMessage` refreshes the existing item and publishes `NewMaybeAdded` (`3037–3051`). Its boolean return also depends on whether the item already has a main view; it is not a simple “this ID has ever existed” predicate. `applySentMessage` was read through its actual body (`history_item.cpp:2780–2872`): it refreshes content and related metadata, but this dossier does **not** attribute reaction refresh to that body. Reaction updates have their explicit dispatcher/edition paths.

### 2. Read progress is distinct from reaction state and local unread marks

Non-channel inbox/outbox read updates pass through PTS admission (`api_updates.cpp:1845–1853`) before applying to the history named by their peer (`1366–1389`). Inbox updates reconcile folder/unread counts; outbox updates repaint the affected outgoing summary and publish an `OutboxRead` history change (`history.cpp:2264–2321`). Both stored read boundaries advance with `accumulate_max` (`3813–3831`), so an older update does not move the boundary backward. [Exact read watermark owner](https://github.com/telegramdesktop/tdesktop/blob/f985723b552b30fe1c17d377d57f44e61a15a8c4/Telegram/SourceFiles/history/history.cpp#L3813-L3831).

Media/mention reads are a different contract: `api_updates.cpp:1340–1364` marks matching loaded items, requests repaint and delegates unknown IDs to unread-things recovery. `HistoryItem::applyMediaContentsRead` can arm or expire self-destructing media (`8452–8470`). Those destructive TTL consequences are outside the proposed Human reaction/read-receipt refresh batch. Likewise, the separate dialog-unread-mark update is not evidence that a remote delivery/read receipt may regress.

The upstream monotonic values above are **message-ID watermarks**, not per-message wall-clock receipt timestamps. They support nonregressing read coverage but do not establish that two valid Fabushi `readAt` values must be reconciled by choosing the maximum instant. Parse validity, retaining an established receipt when a response omits it, and any rule for two conflicting valid timestamps must be explicit Fabushi contract choices with their own tests. The TTL use of an upstream `readDate` does not prove a delivery-receipt timestamp merge policy.

### 3. Recovery applies state changes before publishing the resulting cursor

On a new MTProto session, `mtpNewSessionCreated` resets the sequence marker and requests the difference (`api_updates.cpp:890–896`). While a difference is outstanding, ordinary streamed updates do not enter the normal path; force-logout and group-call exceptions are explicit (`898–907,955–975`). This is an account-wide recovery policy, not a new Human conversation owner.

`feedDifference` processes users/chats, resolves message-ID correlations, processes messages, then applies the other updates (`614–625`). Only afterward does `differenceDone` apply the intermediate/final state; a slice triggers another difference request (`490–526`). `getDifference` uses the established PTS/date/QTS state and prevents duplicate outstanding requests (`693–721`). Failures schedule bounded exponential retry timing (`580–594,628–634,663–691`). The read code does not establish a durable SQLite transaction for this C++ in-memory state; that durability requirement must be satisfied by Fabushi's existing storage owner.

The normal `updates` and `updatesCombined` envelopes reject already-applied sequence ranges and buffer a forward sequence gap before applying its contents (`1502–1546`). `PtsWaiter` separately tracks known-good PTS, cumulative counts, ordered skipped updates and recovery timers. Positive-count updates at/below known-good PTS are skipped; a detected gap is queued, then replayed in PTS/key order when continuity is established (`data_pts_waiter.cpp:81–186`). It is wrong to describe all reaction changes as individually PTS-versioned: the reaction TL declaration has peer/message identity and reaction data, but no PTS or reaction revision field (`api.tl:419`). The containing envelope or refresh operation supplies the surrounding ordering context.

### 4. An existing ID is not proof of a complete history interval

`HistoryBaseMessagesSlice` preserves full count, skipped-before/after information, IDs and the nearest anchor (`data_history_messages.cpp:38–54`). `HistoryViewer` subscribes to slice changes/removal/invalidation and requests insufficient ranges from the existing history owner (`341–413`). Client-side messages are merged without duplicate IDs, with date and server/client tie behavior, while preserving the chosen anchor (`98–225`). Canonical ID changes rewrite the current anchor/slice references (`502–511`).

`SparseIdsSliceBuilder` uses intersection with an explicit no-skip range, optional counts and known/unknown skipped boundaries; it does not establish whole-history coverage from a single present ID (`data_sparse_ids.cpp:170–197,246–297`). Missing before/after limits cause bounded directional requests (`314–365`). `ApiWrap::requestHistory` deduplicates a peer/message/direction request, parses its result and supplies the returned no-skip range to the history owner; failures remove the request marker and finish the owning request, with explicit access-loss handling (`apiwrap.cpp:3636–3683`). The parser's own detailed response validation remains outside this read boundary.

For pending-to-server identity settlement, `HistoryItem::setRealId` exchanges the ID, clears local/sending flags, unregisters the client-side message and publishes identity/data/`NewMaybeAdded` changes (`history_item.cpp:3296–3335`). The history viewer rewrites identity and anchor references (`data_history_messages.cpp:502–511`), while the sparse builder merges server IDs into a sorted set and the client-side merge uses message dates. These excerpts establish an ordered view with identity/anchor continuity; they do not establish a permanent append-slot rule. If Fabushi has `[anchor, pending-P]` and later discovers older inbound A/B after settling P, preserving SQLite slots as `[anchor, P, A, B]` is a target storage policy, not evidence of equivalent server chronology. Exact reorder timing through every upstream consumer was not traced here; broader chronology/viewport acceptance remains explicit.

### 5. Peer scope and malformed-input limits

Reaction/read updates use their peer or channel scope to find an existing owner; missing short-message peer/mention/reply context initiates difference recovery instead of fabricating a complete item (`api_updates.cpp:101–183,1555–1582,1633–1669`). `Session::addNewMessage` refuses an absent peer or `messageEmpty` before routing to a history (`data_session.cpp:3582–3590`).

These functions consume already-decoded MTP types. They do not prove malformed binary-frame handling or generated parser bounds; only the relevant TL inputs and their consumers were read. The account-wide upstream dispatcher also legitimately handles many conversations. Its “loaded peer” condition cannot replace Fabushi's selected direct-conversation participant check, nor authorize arbitrary reactions or read changes from a foreign page.

## Current target wiring and demonstrated static gaps

The following identities were fetched at H. All target files were read selectively, not claimed as full-file product review.

| Target path | Blob at H | Examined path |
| --- | --- | --- |
| `source/host/src/extensions/session/production.rs` | `948db9d624ebc7ad492d962e8f66e006323e60b5` | owner fields; native access; participant/ack admission; resource and message materialization; bounded sync; Human reaction mutation |
| `source/host/src/extensions/session/native_messaging.rs` | `08e0921e9826006e109ed4253d518fad8bf4c68c` | remote message/reaction types, list request and JSON/error handling, reaction request boundary, identity conversion |
| `source/host/src/extensions/session/gateway.rs` | `e77e6bc40d9b12a020b7ad5eec1fcb6d11c68550` | `syncHumanConversation(s)` and `reactHumanMessage` production dispatch |
| `source/host/src/extensions/session/agent_db.rs` | `91ed720449b8e171a94887d032343103be6c829b` | append/update/identity settlement, connection write mutex and publication of `entries-upserted` |
| `source/host/src/extensions/session/agent_db_schema.rs` | `3d381c12c55d8444caac81f61b64bd81efa5e2bd` | transcript identity/sequence table and update SQL |
| `source/host/tests/session_gateway_contract.rs` | `4913a378f626a2763be8e0c80a14712ea9efb28f` | full reaction-toggle, 401-message gap and resource-gap-restart test bodies; remaining NMR names indexed only |
| `frontend/src/production/ProductionRenderer.tsx` | `26dcb2aad353551e6e8f8468f7ce3a63335730ce` | Human reconnect/open calls, authoritative baseline merge, reaction dispatch and feed update callback |
| `frontend/src/production/model.ts` | `a8607a383ec32f6fba7142bd7ce8ae93f6af7dc5` | Human transcript and message/reaction projection |

The existing production route is renderer reconnect/open → `syncHumanConversation(s)` gateway → `ProductionSessionWorkers` → issuer/credential-bound `FabushiNativeMessagingClient` → `SandAgentDb` → existing transcript mutation publication. Renderer reconnect additionally re-reads the canonical transcript and reconciles its existing projection (`ProductionRenderer.tsx:2580–2640,2715–2745`); the feed's update callback replaces a matching existing entry (`2877–2895`). Human reaction actions already use the same Host and DB owner (`1386–1404`; `gateway.rs:403–409`; `production.rs:3241–3334`). No alternate message store, Telegram provider, Coordinator lifecycle or parallel UI is needed for this responsibility.

Specific gaps and constraints, derived from code inspection rather than test execution:

1. **Known rows are discarded.** `sync_human_conversation` finds the newest admitted known ID, sets `unseen_start` to the following index, stores only the suffix, and stops paging on overlap (`production.rs:2332–2346`). A page containing only known messages produces no materialization calls. Earlier known rows in that page are also omitted. Yet `materialize_remote_human_message` already compares a reconstructed entry and uses `update_transcript_entry` for changes (`2180–2202`). [Exact cutoff](https://github.com/bhrumom/fabushi-desktop/blob/62a3fc19501a5c6f5e25495c833120ae67b973e6/source/host/src/extensions/session/production.rs#L2290-L2387).
2. **A newly settled send is not a synchronized range.** Pending-send recovery occurs before the known-ID set is collected (`2268–2298`). If that recovery appends/settles a newest outbound message, the next page can end with that known ID while containing previously unseen inbound messages before it; the suffix rule discards those inbound rows. The same risk exists when a previously settled outbound message sits above a history hole. Capturing the known-ID set before retry is insufficient by itself: after a subsequent GET/resource failure and restart, that settled outbound ID is already persisted and becomes the next run's false anchor. Trusted history coverage must survive failure/restart independently of individual message settlement. A fix limited to refreshing known rows must not claim that this range-completeness defect is solved. Inserting old messages after newer rows also requires preserving the existing sequence/viewport contract, not merely removing the slice operation.
3. **Admission must remain before side effects and overlap use.** The pure validator enforces the local/peer pair, canonical sender consistency and local outgoing nonce/text/reply/resource consistency (`1964–2058`). It must cover known refreshes as well as insertion. It does not validate creation timestamps, read timestamps or reaction counts before a row can become an overlap marker. The materializer parses `createdAt` later, while it only trims/copies `readAt`. A semantic-malformed known row must not silently prove coverage.
4. **Read and reaction freshness need explicit rules.** The materializer rebuilds the entry: an absent/blank `readAt` omits the persisted field, and an omitted `reactions` JSON field deserializes to `[]` (`native_messaging.rs:54–73`; `production.rs:2116–2123,2152–2179`). There is no remote snapshot/reaction revision in the reviewed message type. Simply enabling the old refresh branch can erase known read state or apply a late reaction snapshot over a newer successful toggle. The DB mutex serializes individual writes, not the preceding HTTP read/modify/write interval (`agent_db.rs:1639–1663`).
5. **Malformed count work is unbounded.** A reaction count is `u64`, and both the message materializer and local reaction-result projector expand `peer_reaction_count` into one element per count (`production.rs:2152–2168,3287–3303`). An oversized or inconsistent service response must be rejected/bounded before this loop in any batch that broadens its reach. This is a static risk, not an observed production exploit or executed test result.
6. **Keep the existing ordering and failure boundary.** Pages arrive newest-window-first with oldest-first rows; current replay reverses pages only. A resource or persistence failure stops replay before later rows can create a misleading overlap, while deterministic participant/ack rejection is reported after independent valid progress (`2372–2392`). Resource cache reuse, byte preservation and restart retry remain on the existing owner. The existing 401-message and resource-retry tests must remain actual passing regressions at the new H.
7. **Known updates can preserve sequence.** The transcript table has a stable `seq` and unique `id`; the existing update SQL changes only `entry`, and DB publication sends an upsert after the write. Refreshing the same row through that path need not create a duplicate or reorder history. The materializer's equality check avoids redundant identical updates; its current explicit preservation of `forwardContext` must not be lost.
8. **Read receipt UI and forwarded provenance projection (verified green at `9385fad973fcd1a751af48100e5b410d33f34022` in run `41695632873`).** At `a8607a383ec32f6fba7142bd7ce8ae93f6af7dc5`, `model.ts:231–243,524–539` projected Human rows through a fixed message shape that included reactions but omitted `readAt` and card-level forwarded provenance. `frontend/src/recovered/features/conversation/workspace/model.ts` (`isValidHumanReadAtTimestamp`, `shouldShowHumanForwardedNotice`, `shouldShowHumanReadReceipt`), `frontend/src/production/model.ts` (`projectTranscriptEntry`), `frontend/src/production/transcript-projection.ts` (`monotonicHumanReadAt`), and `frontend/src/recovered/features/conversation/workspace/transcript.tsx` (`ForwardedFromNotice` `role="note"` and `HumanReadReceiptNotice` `role="status"` `data-read-at`) now project monotonic `readAt` and privacy-gated `forwardedFromAuthorName` onto the shared message card, covered by `transcript-projection.contract.test.ts` (`44/44`) and `ProductionRenderer.transcript.behavior.test.mjs` (`15/15`).

## Responsibility mapping for the integrator

These are proposed bounded responsibilities, not newly accepted ledger IDs. The earlier share-box forwarding/privacy rows do not own them.

| Responsibility | Primary source evidence | Existing Fabushi owner and contract |
| --- | --- | --- |
| Apply mutable reaction state to an already-known peer-scoped message | `api_updates.cpp:1812–1833`; `history_item.cpp:5482–5550`; `data_message_reactions.cpp:2179–2328` | Session production admission/materialization → canonical DB update → existing transcript projection; stable ID, no duplicate/order change, removal as well as addition |
| Preserve monotonic remote read progress independently of local unread marks | `api_updates.cpp:1366–1389,1845–1853`; `history.cpp:2264–2321,3813–3831` | Existing Human transcript state and shared message UI if included; stale/null data cannot invent a backwards read transition |
| Preserve pending local reaction results against stale refresh | `data_message_reactions.cpp:2119–2127,2179–2187`; `history_item.cpp:5539–5550` | Existing Session owner serializes/fences mutation and refresh; no new service or second source of reaction truth |
| Distinguish ID overlap from complete synchronized intervals | `data_history_messages.cpp:38–54,341–413`; `data_sparse_ids.cpp:170–197,246–365`; `api_updates.cpp:490–526,614–625` | Existing `MSG-HISTORY-ORDER-GAP` owner anchor; canonical history coverage and ordered durable replay, with separately stated bounds |
| Reject foreign/malformed state before it changes identity, resources or coverage | Upstream scoped dispatch and missing-context recovery; target NMR participant/immutable-send validator | Existing Host admission and network boundary; reapply the same checks for known rows; report precise deterministic rejection versus retryable failure |

This source analysis is sufficient to identify these responsibility slices and their consumers. It is not sufficient to mark entire partially read files reviewed, accept all Telegram reactions/topics/channels/TTL behavior, or grant final source/product acceptance.

## Concrete behavior cases for the next batch

All cases below are **proposed, not yet authored or executed**. Reuse the existing loopback HTTP fixture, real `ProductionSessionWorkers`, canonical DB and gateway. Execute only in exact-H GitHub Actions after the current message-contract prerequisite succeeds.

| Case | Fixture and required observable result |
| --- | --- |
| Known-only refresh | Seed one canonical peer message, return the same identity with a new reaction and valid read receipt. Sync updates that same durable row; reload/restart reproduces it; ID, sequence, content and attachments remain stable. |
| All known rows in a mixed page | Return multiple known rows with distinct changed reactions/read state followed by a new row. Refresh every known row in the fetched scope, append the new row once, preserve the existing row order, and repeat the snapshot without extra changes. |
| Reaction removal | Start with a nonempty authoritative reaction set; return an explicitly complete empty set. Remove the prior reaction, retaining message identity/content. Distinguish an omitted/partial set according to the declared service contract. |
| Foreign known-ID collision | Return a known message ID with a foreign participant or conflicting sender and an attachment sentinel. No resource request, DB update, local-send settlement or false overlap is allowed; independent valid rows still follow the established rejection policy. |
| Invalid known snapshot | Return an invalid creation/read timestamp, invalid identity or malformed/bounded-out reaction data on a known ID. Preserve prior content/state and prevent that invalid row from terminating needed gap recovery; error/rejection classification must be explicit. A structurally invalid JSON envelope must fail before writes. |
| Local send above an inbound gap | Seed an old synchronized anchor plus a pending outbound send. Recover its acknowledgement to a new high remote ID, then return older unseen inbound rows before that known outbound row. Required coverage must not disappear behind the new ID; verify canonical ordering rather than only count. Also exercise a high outbound ID already settled before sync. |
| Existing ordered resource failure | Include a missing older resource followed by newer unseen data and changed known state. The chosen ordered replay policy must stop at retryable failure, preserve prior bytes/content, and converge on restart without skipping the missing row or duplicating later rows. Retain the existing resource-gap regression. |
| Read-state non-regression | Refresh a known read message with an older or null receipt after a newer receipt was applied. Preserve monotonic read state unless a separately specified authoritative transition explicitly permits otherwise. Invalid dates never become trusted read state. |
| In-flight reaction race | Block a history GET, settle a local reaction POST, then release the older GET response. The older response cannot erase the newer successful result; verify DB and renderer-facing projection. A later authoritative refresh still converges. |
| Stable publication | Observe the canonical transcript mutation feed. A changed known row publishes one upsert with its existing identity after durable success; byte-for-byte identical replay does not produce a redundant materializer update. No new append/identity-settlement event is synthesized for a metadata-only refresh. |
| Restart and resource reuse | Repeat a known refresh across shutdown/reopen using a previously materialized attachment. Resource bytes and stable path survive; no redundant download occurs while the cached resource is valid; `forwardContext` and correct author-scoped nonce provenance remain intact. |
| Bounded walk and UI scope | Retain the 200-row/32-page recovery bound and cursor no-progress failure. For visible reactions, verify reconnect refreshes an already-open Human conversation. If read receipts are claimed as UI behavior, add the shared model/card projection and visual/accessibility assertion; DB JSON alone is insufficient. |

The reviewed endpoint accepts a timestamp-only `before` cursor, and the client does not expose a state revision or ID tie-break cursor. Server-side ordering, equal-timestamp pagination, reaction snapshot completeness and any per-message revision guarantees were not re-audited here. They are explicit contract questions for broader history/sync completeness, not permissions to weaken the final acceptance gate.

## Remaining boundary

No whole-project closure changes follow from this dossier. Recursive original-host source provenance, complete source dispositions, legal records, broader destination/topic/channel behavior, live service correctness, supported packages, UI acceptance, Bot regression and independent ACCEPT remain independently required. Current Actions outcomes must be read and bound to the integrator's actual frozen H; this research contains no fabricated run, test count or passing-result claim.

## Round 7 sparse history and parser review

This append records source reading and a production wiring review on 2026-11-02. Fixed upstream remains `telegramdesktop/tdesktop@f985723b552b30fe1c17d377d57f44e61a15a8c4` (tree `91f2b083ab2877c0a17d07868eabec362a5cfca6`). Target comparison is `bhrumom/fabushi-desktop@35fe95331cdb67eb32430e03043ab9246b9b1453`, inspected through exact Git objects without switching the working tree. No executable validation, implementation change or acceptance is claimed.

Earlier sections retain their original target and reading boundaries. In particular, this append resolves the parser previously described as unread; it does not claim that earlier investigation read it. The separate `message-settlement-and-history-gap.md` uses historical U `33261535a0e747f125e0ed25486f01e556330677` and does not establish current-U coverage.

### Exact texts and reading limits

The entire returned UTF-8 text of each file below was read at fixed U. “New” means newly recorded relative to this dossier; “reread” preserves its earlier reading evidence. Text reading does not establish complete responsibility mapping, transitive review or product acceptance. Paths are relative to `Telegram/SourceFiles/`.

| Path | Exact blob | Reading relationship |
| --- | --- | --- |
| `data/data_history_messages.h` | `5c8276867846c861baf208d2e1f059f4edeedf3d` | New |
| `data/data_history_messages.cpp` | `412f30c67ae3d39442a38dce66ae123d864ab4ac` | Reread |
| `storage/storage_sparse_ids_list.h` | `2f892a0e11cc4b92d16d82aa9f61dc471a7c0fe2` | New |
| `storage/storage_sparse_ids_list.cpp` | `cc0e3ed6b2c763326c2c16635a7671a747ef506b` | New |
| `data/data_sparse_ids.h` | `10dbe9037e3221a0d42154d21f84a8af0286c552` | New |
| `data/data_sparse_ids.cpp` | `1f11a0424294023ee486a87e1ba194cabd9bd87a` | Reread |
| `data/data_search_controller.h` | `289a6b7e4e091cbf5ce9424d18a1b5f08396f979` | New |
| `data/data_search_controller.cpp` | `5c1869566dd0551bdf9679aa71c2e1be8392dd93` | New |

Scoped supporting reads were `apiwrap.cpp@342e501177996be3c083ba7985d3720a53100b51`, lines 1–115 and 3628–3699; and `data/data_histories.cpp@8abdb57141996c9b39b5531c9d2ff0c1e1b18adf`, lines 795–831, 1040–1145 and 1231–1342. Their remaining behavior was not reviewed here. Symbol-only lookups of `data_messages.{h,cpp}`, `apiwrap.h` and adjacent media code add no responsibility coverage.

### State, request and lifetime obligations

**Coverage remains separate from cached IDs.** `SparseIdsList` retains explicit `noSkipRange` boundaries and optional count/skipped information. Separate gaps do not become continuous because a later ID exists. An uncovered anchor can yield count-only information. `invalidateBottom` keeps cached rows while invalidating newest completeness; `removeAll` establishes a known complete empty range. `countAfter` requires a covered anchor and newest endpoint (`storage_sparse_ids_list.cpp:69–311`).

**The existing history owner supplies missing ranges.** `HistoryViewer` subscribes to slice/removal/clear/invalidation and routes insufficient ranges through `ApiWrap::requestHistory` (`data_history_messages.cpp:341–413`; `data_sparse_ids.cpp:299–365`). The final viewer suppresses an unresolved empty/count-only snapshot with positive total. Local rows are restricted to their history and respect unknown boundaries. Identity rekey updates anchors, IDs and nearest-to-around; subscriptions and delayed local-row merging are lifetime owned. Migrated-history and sponsored-tail semantics remain separate mapping obligations. This in-memory algorithm proves no Fabushi SQLite restart or opaque-walk behavior.

**The previously unresolved parser is now located.** [`PrepareHistoryRequest` and `ParseHistoryResult`](https://github.com/telegramdesktop/tdesktop/blob/f985723b552b30fe1c17d377d57f44e61a15a8c4/Telegram/SourceFiles/data/data_search_controller.cpp#L294-L347) use a 50-row history limit: Before uses anchor/0, Around anchor/−25, and After anchor+1/−50 for offset/add-offset, with allowed offset bounds. Fabushi’s 200-row/32-page limits are target contracts. `ParseHistoryResult` delegates to `ParseSearchResult:204–292`, which processes response variants and related session data, adds messages through the existing owner, and derives the returned range from IDs. Successful empty responses establish directional terminal ranges. This is not independent proof of Fabushi participant admission before resource effects; malformed or unbound HTTP responses must not become successful empty history.

**Failure and deletion preserve request ownership.** `ApiWrap::requestHistory:3636–3683` deduplicates peer/anchor/direction, removes its marker on success and failure, and reports applicable access loss without inventing an empty-complete slice. `Histories` postpones history requests during deletion, cancels/postpones already active history generators when deletion starts, and resumes them after completion. Cancellation and finish remove owner mappings. By contrast, `SearchController::requestMore:530–573` explicitly removes its key on success but only calls `finish()` on failure; unconditional same-key retry is not established by that code.

### Current production comparison

These four blobs bind the target observations to `35fe95331cdb67eb32430e03043ab9246b9b1453`:

| Target path | Exact blob |
| --- | --- |
| `source/host/src/extensions/session/production.rs` | `fafad99ad86d25562c14644343fda58c8506b530` |
| `source/host/src/extensions/session/native_messaging.rs` | `2c86ddf9c9432334e862a062d72a2de8903368dc` |
| `source/host/src/extensions/session/gateway.rs` | `9e4f1ebc675e7ab3bd0d57b3833448dc72ad979f` |
| `frontend/src/production/ProductionRenderer.tsx` | `5f8ec8e8bf92317d8e0a8d52a5d0be4c49e0b20a` |

This H already persists coverage `version:1`, checks it before this conversation’s reaction/pending-send replay, and restarts valid legacy coverage from origin until qualified persistence. Renderer initial-read deduplication, author/lifetime fencing and clear-triggered reopening are also present. This reading does not requalify their test results.

The relevant production call closure remains:

1. Renderer `refreshRoster:2564–2704` invokes `syncHumanConversations` once. Host `sync_human_conversations:2661–2688` invokes the bounded sync once per friend.
2. Host `sync_human_conversation:2512–2659` normally returns a transcript vector after 32 full pages, retaining `complete:false`. Gateway `323–332` returns arrays; roster `2891–2939` and tail `423–445` expose no coverage.
3. The active cached conversation refresh reads all durable local rows, resets local pagination copies, and schedules no coverage continuation.
4. Renderer `openAgent:2741–2852` returns before syncing a cached conversation. An uncached Human open syncs once; pagination `1055–1070` only reads the local transcript.
5. Feed `2938–3049` updates projections; clear can reopen the active conversation. Ready/connected/focus and explicit or queued retry can refresh again. The connection controller has no coverage poll; Renderer’s interval belongs to teach recording.

**Deterministic static gap (verified green at `1ba6c3680af2237cd6bbfb4b4a08070d03e6644d` in run `41691497048`):** after startup settles, let an already cached active Human conversation have more than 6,400 allowed remote rows beyond its qualified cursor. Previously, one isolated roster refresh received 32 valid full pages without failure and retained `complete:false` without exposing an incomplete-coverage signal to the consumer. `ProductionSessionWorkers` (`metadata_history_coverage_incomplete`, `human_conversation_coverage_incomplete`, `list_human_conversations`) and `gateway.rs` (`getHumanConversationTranscriptTail`) now emit `"historyCoverageComplete": false` whenever `humanHistoryCoverage.complete == false` (omitting the field when complete to preserve backwards-compatible summary and page shapes), and `ProductionRenderer.tsx` (`continueHumanHistoryCoverage`, `humanCoverageContinuationInFlightRef`) drives bounded multi-window continuation (`syncHumanConversation` -> `listHumanConversations` -> `getHumanConversationTranscript`, fenced by account, transport, selection, and conversation generations).

### Disposition and remaining work

At this H the canonical ledger still has six forward source-review records and eight pending sparse/search source-review records. Its schema permits `pending` and `complete`, not `partial`. Pending records preserve this reading while responsibility decomposition and acceptance remain open; publishing them requires real dossier anchors and canonical inventory/relationship checks. Root integration owns that publication. No global unknown/unread counter is reduced.

Global/media search, forwardable permissions, topic/monoforum/migrated scope, query-cache restoration and remaining consumers still require mapping. The 8 fixed-U sparse/search source review records (`data_history_messages.h/.cpp`, `storage_sparse_ids_list.h/.cpp`, `data_sparse_ids.h/.cpp`, `data_search_controller.h/.cpp`) are now linked to `FBCP-NMR-001` (`status: "pending"`) alongside domain-backed multi-term & forward-origin search wiring (`native/mahayana-messaging/src/search.rs`, `ProductionSessionWorkers`, `roster_search.rs`, `frontend/src/production/model.ts`, `find-in-chat-controller.ts`, verified green at `2ac90b3115a280c9a915f3b248b828b20f5ffab3` in run `41686626329`), the multi-window Human history coverage continuation consumer (`ProductionSessionWorkers::human_conversation_coverage_incomplete`, `gateway.rs`, `ProductionRenderer::continueHumanHistoryCoverage`, verified green at `1ba6c3680af2237cd6bbfb4b4a08070d03e6644d` in run `41691497048`), the `NMR-READ-RECEIPT-01` Human read-receipt monotonic projection and privacy-gated forwarded provenance UI (`model.ts`, `transcript-projection.ts`, `transcript.tsx`, verified green at `9385fad973fcd1a751af48100e5b410d33f34022` in run `41695632873`), the `TDRP-MSG-FWD` chained forward-origin attribution and peer display title fallback (`native/mahayana-messaging/src/message.rs` `ForwardPrivacy::resolve_forward_origin_attribution`, `ProductionSessionWorkers::resolve_human_forward_origin`, and `roster_search_contract.rs`, verified green at `c1882a8c8f2af0019d3d1e4abbfaee21a690536b` in run `41700052529`), and the newly authored `TDRP-R9-SHARE-RECIPIENT-ELIGIBILITY-001`, `TDRP-R9-SHARE-DESTINATION-POLICY-001`, and `NMR-UI-ENTRY-REFRESH-01` / `TDRP-R9-SHARE-FORWARD-PRIVACY-001` batches (`ConversationPermissions::can_send_other/can_send_inline/can_send_games`, `MemberRestrictions::send_other/send_inline/send_games`, `MessagingService::actor_can_send_to_recipient`, `MessagingEngine::message_content_uses_other`, `ProductionSessionWorkers::human_recipient_requirements_from_entry` & `human_recipient_authorization_from_metadata`, and `transcript-projection.ts` `isForwardableTranscriptMessage` & `reconcileForwardTargetMessage` wired into `ProductionRenderer::retireOrRefreshForwardTarget`, requiring containing-H Actions evidence). Human retry remains reserved by `fbcp-human-message-retry.md` (HRR); this review proposes no replacement retry API. Full source/legal closure, deployed services, supported packages, UI/a11y, recovery/security/performance/soak/Bot qualification and independent ACCEPT remain open.
