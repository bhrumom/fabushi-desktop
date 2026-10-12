# notifications_manager.h/.cpp complete read

- Accepted upstream: `telegramdesktop/tdesktop@d346b42a1d30ef60dc989b6e5191bb8e571f6bd5`
- Header: `Telegram/SourceFiles/window/notifications_manager.h@cd36b671795afa49b282b1609380cf7e5c3473ed` — complete, 484 lines
- Implementation: `Telegram/SourceFiles/window/notifications_manager.cpp@e5b9eb949e180667952172d338cbcf3e552798d8` — complete, 1700 lines
- Status: fully read/decomposed; mapped/open, not implemented or verified

## Complete responsibility decomposition

The pair owns far more than toast rendering. It defines the system/manager/native-adapter boundary; account/session and peer/topic/SavedSublist notification context identity; mute/block/privacy and lock-screen redaction; reaction/poll duplicate suppression; cross-device timing delays; scheduling queues, waiters, alerts and timers; forwarded/album grouping; custom/default sound and flash/bounce; settings-driven manager replacement; native activation/open routing; native reply with typed topic/SavedSublist draft identity; callback actions; multi-account title disambiguation; and display-target selection.

The lifecycle is explicit. `registerThread` watches Topic and SavedSublist destruction. `clearFromSublist` clears the native manager projection, the sublist's queued notifications, all timing/waiter maps, the destruction watch, cancels the timer and recomputes the next delivery. `clearIncomingFromSublist` is separately scoped. This is the notification-clear responsibility invoked by SavedMessages deletion; deleting only `ConversationChildRuntimeState.pending_incoming_notification_message_ids` cannot claim parity.

## Existing-owner-first mapping

Fabushi already has the correct source-neutral owners:
- `native/mahayana-messaging/src/notification.rs`: `NotificationPolicy`, `NotificationCandidate`, `NotificationDecision`, `ConversationNotificationRule` for canonical policy.
- `source/electron-main/main-production-services.ts`: one `ProductionNotificationsService` shipping service boundary.
- `source/electron-main/notifications/os-notification-manager.ts`: one `SandOsNotificationManager` for OS instance lifecycle.
- `source/electron-main/production-binding-providers.ts`: `createProductionNotificationsBinding` for production composition.
- `source/shared/os-notification.ts`: existing `SandOsNotificationDecider` transition/throttle machinery.

No Telegram/source-shaped notification root is allowed.

## Open production parity

The existing production notification service currently consumes Agent roster/event feeds. That is not evidence for Human messaging notifications. Still open:
- canonical Human/Group/Channel/Topic/SavedSublist messaging event feed into the existing notification service;
- exact keyed OS-instance/queue clear by Conversation/Topic/SavedSublist;
- destroy/account-reset stale-callback fencing for those messaging notifications;
- native reply/action composition with canonical Human messaging authorization/revalidation;
- SavedSublist child identity from the authoritative service relation/membership feed.

Therefore both rows remain `mapped` with no production or verification credit.

## Traceability anchors

### `TDRP-R9-NOTIFICATION-MANAGER-CONTRACT-001`
- Requirement: `TDRP-R9-NOTIFICATION-MANAGER-CONTRACT-001`
- Oracle: `ORA-TDRP-R9-NOTIFICATION-MANAGER-CONTRACT-001`
- Invariant: `INV-TDRP-R9-NOTIFICATION-MANAGER-CONTRACT-001-CANONICAL`
- Release gates: `G-FILE`, `G-TRACEABILITY`, `G-NOTIFICATION`, `G-EVIDENCE`

### `TDRP-R9-NOTIFICATION-MANAGER-LIFECYCLE-001`
- Requirement: `TDRP-R9-NOTIFICATION-MANAGER-LIFECYCLE-001`
- Oracle: `ORA-TDRP-R9-NOTIFICATION-MANAGER-LIFECYCLE-001`
- Invariant: `INV-TDRP-R9-NOTIFICATION-MANAGER-LIFECYCLE-001-CANONICAL`
- Release gates: `G-FILE`, `G-TRACEABILITY`, `G-NOTIFICATION`, `G-EVIDENCE`

## Accounting

Only these two exact accepted blobs receive new read credit in this commit:
- `unknown`: 15788 (unchanged)
- `unread`: 15739 -> 15737
- `omitted`: 0

No `history.*` or other notification dependency is decremented by this dossier.
