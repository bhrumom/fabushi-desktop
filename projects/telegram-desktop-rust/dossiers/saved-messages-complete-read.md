# SavedMessages parent-scoped child collection — complete read dossier

Status: mapped-open / implementation incomplete  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`  
Root tree: `5db05afa460bb030ac36316beb6496df03742c59`

## Exact source authority

| path | blob | read status |
| --- | --- | --- |
| `Telegram/SourceFiles/data/data_saved_messages.h` | `82b9aed5cb2e1250fc0f4c5c2013bd58c689a344` | complete, 154 lines |
| `Telegram/SourceFiles/data/data_saved_messages.cpp` | `2be216670b644eb012e7a241af78b3adda639619` | complete, 651 lines |
| `Telegram/SourceFiles/data/components/recent_peers.h` | `5d8b1187d8db1138003cc406d86cbdec6626468a` | complete, 59 lines |
| `Telegram/SourceFiles/data/components/recent_peers.cpp` | `bf676e4407fcfa4bf93b2bf7e468b8f61c6ffd05` | complete, 184 lines |

These four source entries are semantically read-complete. The `recent_peers` pair is a direct cleanup/navigation dependency and distinguishes ordinary recent Search suggestions from recent-open Thread history. This dossier does **not** claim the SavedMessages responsibility is production-complete. The existing Fabushi parent-access primitive is only one prerequisite; server/native relation feed, canonical membership source, child-list picker composition and several lifecycle responsibilities remain open.

## Responsibility decomposition

### TDRP-R9-SAVED-MESSAGES-PARENT-SCOPE-001
Source symbols: `SavedMessages`, constructor/parent access, monoforum parent acquisition/revocation paths.  
Oracle: ORA-TDRP-SAVED-MESSAGES-PARENT-SCOPE-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-EXPLICIT-PARENT-001, INV-TDRP-SAVED-MESSAGES-REVOKE-CLEANS-CHILDREN-001.  
Behavior: SavedMessages children exist only under canonical self SavedMessages or an explicit authoritative parent relation. Generic community identity is insufficient. Revoking the relation invalidates child runtime owned under that parent.
Target owner: canonical MessagingState/Conversation parent-access state; current primitive `reconcile_saved_sublist_parent_access`.  
Status: implemented primitive, shipping authoritative feed still open.

### TDRP-R9-SAVED-MESSAGES-REQUEST-BATCH-001
Source symbols: request/coalescing/stale refresh state and callbacks.  
Oracle: ORA-TDRP-SAVED-MESSAGES-REQUEST-BATCH-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-ONE-BATCH-001, INV-TDRP-SAVED-MESSAGES-STALE-FENCE-001.  
Behavior: duplicate/stale child metadata requests coalesce, stale settlement cannot overwrite a newer generation, cancellation/disposal retires pending work.
Target owner: existing Conversation child request lifecycle/controller plus native messaging service projection.  
Status: mapped; shipping composition incomplete.

### TDRP-R9-SAVED-MESSAGES-PAGINATION-001
Source symbols: list/page loading, offsets/cursors, pinned refresh.  
Oracle: ORA-TDRP-SAVED-MESSAGES-PAGINATION-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-PAGE-ORDER-001, INV-TDRP-SAVED-MESSAGES-PINNED-REFRESH-001.  
Behavior: child collection pagination is ordered, duplicate-safe and boundary-aware; pinned refresh does not silently invent membership.
Target owner: Conversation child pagination state + authoritative membership feed.  
Status: mapped/open.

### TDRP-R9-SAVED-MESSAGES-UNSUPPORTED-001
Source symbols: unsupported/error path around SavedMessages availability.  
Oracle: ORA-TDRP-SAVED-MESSAGES-UNSUPPORTED-001.  
Invariant: INV-TDRP-SAVED-MESSAGES-FAIL-CLOSED-001.  
Behavior: when the parent/server capability is unsupported or membership authority is unavailable, Fabushi must fail closed rather than infer a child from visible parent messages.
Target owner: canonical messaging authorization/service boundary.  
Status: mapped/open.

### TDRP-R9-SAVED-MESSAGES-PIN-EMPTY-RESTORE-001
Source symbols: pinned child refresh, empty child transitions, restoration.  
Oracle: ORA-TDRP-SAVED-MESSAGES-PIN-EMPTY-RESTORE-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-PIN-SURVIVES-TEMP-EMPTY-001, INV-TDRP-SAVED-MESSAGES-DESTROY-CLEARS-PIN-001.  
Behavior: temporary emptiness does not erase a still-authoritative pin; actual child destruction clears the child and pin.
Target owner: `ConversationChildRuntime::note_locally_empty` / destroy lifecycle.  
Status: partially implemented at primitive level; shipping picker evidence open.

### TDRP-R9-SAVED-MESSAGES-ACTIVE-SUBSECTION-001
Source symbols: active sublist/subsection selection.  
Oracle: ORA-TDRP-SAVED-MESSAGES-ACTIVE-SUBSECTION-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-ONE-ACTIVE-CHILD-001, INV-TDRP-SAVED-MESSAGES-DESTROY-DESELECTS-001.  
Behavior: one canonical active child per parent scope; destroying the selected child clears selection without falling back to a different implicit parent.
Target owner: existing `ConversationChildSelection`.  
Status: primitive implemented; shipping composition open.

### TDRP-R9-SAVED-MESSAGES-CLEANUP-001
Source symbols: notifications/shared-media/forward-draft cleanup on child removal/destruction.  
Oracle: ORA-TDRP-SAVED-MESSAGES-CLEANUP-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-DESTROY-CLEANS-NOTIFICATIONS-001, INV-TDRP-SAVED-MESSAGES-DESTROY-CLEANS-DRAFT-RESOURCE-001.  
Behavior: destroying the exact child retires child-scoped notification projections, forward draft references and shared-media/resource projections without deleting parent-global state.
Target owner: Conversation child runtime + canonical draft/resource/notification owners.  
Status: mapped/open; unread notification primitive exists, draft/resource shipping cleanup still open.

### TDRP-R9-SAVED-MESSAGES-RECENT-ORDER-001
Source symbols: recent child ordering/update paths.  
Oracle: ORA-TDRP-SAVED-MESSAGES-RECENT-ORDER-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-RECENT-STABLE-001, INV-TDRP-SAVED-MESSAGES-RECENT-NO-STALE-CHILD-001.  
Behavior: recent children reflect authoritative current children, are stable/deduped and remove stale destroyed children.
Target owner: canonical Conversation child list projection.  
Status: mapped/open.

### TDRP-R9-SAVED-MESSAGES-UNREAD-RECONCILE-001
Source symbols: unread aggregate/reconciliation paths.  
Oracle: ORA-TDRP-SAVED-MESSAGES-UNREAD-RECONCILE-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-UNREAD-SERVER-AUTHORITY-001, INV-TDRP-SAVED-MESSAGES-UNREAD-DESTROY-001.  
Behavior: unread state derives from authoritative child/message relation and canonical read cursors; unknown server state is preserved as unknown, not converted into a guessed count.
Target owner: `ConversationChildRuntime::reconcile_unread_things` plus authoritative membership/sync feed.  
Status: primitive implemented; server sync caller/membership authority open.

### TDRP-R9-SAVED-MESSAGES-MEMBERSHIP-001
Source symbols: child message ownership and child lookup/cleanup paths.  
Oracle: ORA-TDRP-SAVED-MESSAGES-MEMBERSHIP-001.  
Invariants: INV-TDRP-SAVED-MESSAGES-MEMBERSHIP-AUTHORITY-001, INV-TDRP-SAVED-MESSAGES-MEMBERSHIP-FAIL-CLOSED-001.  
Behavior: a non-empty SavedSublist may accept/project a message only when the canonical server/native source proves that message belongs to that exact child. Parent-message visibility is not membership evidence.
Target owner: canonical MessagingState/Conversation membership relation populated by a single server/native authoritative sync/feed caller.  
Status: open; current code intentionally fails closed.

## Direct dependency: RecentPeers

The accepted header/cpp split two responsibilities that must not be conflated:

- Ordinary recent participant/conversation suggestions use stable identity bump/remove/clear semantics. Their serialized projection is capped at **48** entries. Empty/malformed data or any per-entry decode failure clears the whole projection rather than retaining a partially trusted prefix.
- Recent-open Thread history is a separate weak projection capped at **32** live entries. Push prunes dead weak references, moves an existing identity to the front without duplication, evicts the oldest when full, and `chatOpenRemove` removes the exact Topic/SavedSublist/Thread. Cached userpic state is presentation projection only.
- `RecentPeers::serialize` serializes the ordinary `_list`, not `_opens`; this read therefore does **not** justify inventing durable recent-open persistence.
- In Fabushi, ordinary suggestions map to canonical Search + existing persistence; recent-open history maps to canonical Conversation/typed-child navigation lifecycle. Neither can mint SavedSublist parent access or message membership.

Current implementation remains incomplete: `ConversationChildDestroyed` removes canonical child runtime state, but the bounded recent-open projection/removal, ordinary recent-suggestion persistence/recovery, shared-media/resource cleanup and full cross-owner destruction composition still require production wiring and exact-head evidence.

## Required shipping closure

1. Connect `saved_sublist_parent_access` to one authoritative server/native sync/feed caller; renderer/client cannot mint access.
2. Add one canonical authoritative message-to-SavedSublist membership relation and populate it from the same trusted sync/feed boundary.
3. Make child-list pagination/picker consume only canonical child membership/list authority.
4. Keep legacy-topic fallback explicit and typed; never reinterpret arbitrary community/topic state as SavedMessages.
5. Close draft/resource/notification cleanup, recent ordering and unread reconciliation through existing owners.
6. Wire exact child destruction to remove the canonical recent-open destination without parent fallback; keep it separate from ordinary recent Search suggestions.
7. Do not delete a canonical parent message/resource merely because one SavedSublist projection is destroyed unless authoritative ownership proves that exact child owns it.
8. Verify all of the above on the exact target HEAD in GitHub Actions; no local/htch execution counts for FBCP/TDRP.

## Accounting

All four source files in this dossier are read-complete. The two newly recorded `recent_peers` entries reduce `unread_minimum` by exactly two more, from 15,746 to 15,744. `unknown` remains 15,788 because responsibility/production closure is not complete. `omitted` remains zero.

## Ledger invariant aliases

The current machine-readable ledger rows use the following stable invariant aliases in addition to the richer invariants above:

- `INV-TDRP-SAVED-MESSAGES-PARENT-SCOPE-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-UNSUPPORTED-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-REQUEST-BATCH-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-PAGINATION-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-PIN-EMPTY-RESTORE-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-ACTIVE-SUBSECTION-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-CLEANUP-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-RECENT-ORDER-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-UNREAD-RECONCILE-001-AUTHORITY`
- `INV-TDRP-SAVED-MESSAGES-MEMBERSHIP-001-AUTHORITY`
