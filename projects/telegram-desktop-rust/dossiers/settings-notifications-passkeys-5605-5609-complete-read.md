# Notifications + Reactions — deterministic orders 5605–5609 complete read

Authority: `telegramdesktop/tdesktop@863cf10d9f34fb0b1b35b35da1bda75acfc58d2e`, root tree `5030985204963cbbd362ced7412d231b04ebd0cc`.

Exact recursive order is authoritative: 5608–5609 are `settings_notifications_reactions.cpp/.h`, not Passkeys. This corrects the earlier transient branch accounting error caught by the Revision 9 fail-closed validator. All rows remain mapped-open.

Main Notifications covers app/global notification preferences, multi-account cleanup, native/custom platform manager configuration, privacy preview, badges, events/calls and routes to per-type/reaction policies. Reactions is server-backed policy for message reactions and poll votes with None/Contacts/All sender scope plus preview privacy. Canonical owners remain Notification Policy, Account/Session scope, Privacy, Call authorization and bounded Platform Notification adapters. Native reply/mark-read/open exact-scope action authorization remains open.

Accounting at order 5609: total 16,123; read-through 5,609; unread 10,514; unknown 15,844; omitted 0. First unread is 5610 `settings_notifications_type.cpp` blob `91c391eec2491faa2bec3541cbad4e5cff0a3929`.
