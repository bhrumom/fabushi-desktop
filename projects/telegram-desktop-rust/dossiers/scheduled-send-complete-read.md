# Scheduled-send authority read — source completeness evidence

Status: read-complete / responsibility closure open  
Project: TDRP-001 Revision 9  
Accepted upstream: `telegramdesktop/tdesktop@f23c37857220eb84f8559f0901ea26fb304b564b`

## Exact source identities

| path | blob | read status | classification | mapping status |
| --- | --- | --- | --- | --- |
| `Telegram/SourceFiles/history/view/history_view_schedule_box.h` | `aa97de8344cc82ddae19ccad7be63bf00357359c` | complete, 2,524 bytes | product contract + UI declaration | open |
| `Telegram/SourceFiles/history/view/history_view_schedule_box.cpp` | `86036fc2c171205d0031ec71f8dcf66e1f369a4a` | complete, 12,175 bytes | scheduling eligibility/state/interaction + presentation | open |
| `Telegram/SourceFiles/api/api_common.h` | `82b21c9bcdac121164138c4f007a48058041af67` | complete, 2,284 bytes | send-option protocol/domain contract | open |

## Transferable responsibilities

The accepted upstream carries more than a date/time picker:

- ordinary scheduled send has a default suggestion of current time + 600 seconds;
- "send when online" is a distinct send type, not merely presentation;
- eligibility is limited to an eligible human recipient: not self, not a bot, last-seen not hidden, no per-message Stars requirement, and not the notifications service user;
- the schedule surface carries silent delivery and repeat-period state in the same send-options snapshot;
- holding Ctrl or toggling notify-off makes a scheduled send silent;
- repeating schedule is separately policy-gated;
- the schedule UI can show scheduled-media thumbnails by date while excluding "when online" entries from calendar-date grouping;
- reminder mode for self is distinct from schedule-to-recipient behavior;
- the upstream protocol uses `0x7FFFFFFE` as a transport sentinel for "until online", but that sentinel is an implementation detail and must not leak into Fabushi's canonical model.

## Existing-owner audit

Current Fabushi already has suitable canonical owners:
- `Actor::presence` / `PresenceStatus` for presence;
- `PrivacyKey::LastSeen` and canonical privacy settings for last-seen visibility policy;
- canonical `Message`, `ClientCommand::SendMessage` / `ForwardMessage`, `MessagingService`, and `MessagingEngine` for scheduling/send lifecycle;
- canonical Conversation/Participant owners for direct-recipient eligibility.

No new Schedule/Telegram owner is justified.

Current gap:
- the canonical message contract currently carries only `scheduled_at_ms: Option<i64>`; it does not yet represent a typed presence-triggered schedule without a magic timestamp;
- presence/LastSeen privacy is not yet wired into a durable "when recipient becomes online" trigger lifecycle;
- persistence, restart recovery, multi-device settlement, cancellation/reschedule, and server-side presence-trigger execution therefore remain open;
- UI eligibility, keyboard/accessibility, reminder semantics, repeat policy, and exact-head E2E evidence remain open.

The correct direction is to extend the existing canonical message schedule policy with a typed trigger and reuse Presence/Privacy/Conversation owners, not to copy Telegram's sentinel or create a ShareBox scheduler.

## Coverage accounting

Durable movement supported by this read:
- `unread: 15782 -> 15779`
- `unknown: 15788` unchanged
- `omitted: 0` unchanged
- `baseline_ready: false`
- `acceptance.accepted: false`

These three entries remain unknown until their applicable responsibilities are fully mapped, implemented in shipping composition, tested on exact HEAD, and accepted.
