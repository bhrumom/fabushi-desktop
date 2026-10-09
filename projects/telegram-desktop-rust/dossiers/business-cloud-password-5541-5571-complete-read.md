# Business / cloud-password responsibilities 5541-5571 — complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`

## Business 5541-5549

Quick Replies enforce premium access, server-provided shortcut/message limits and strict shortcut-name validation. Recipient helpers model all-except vs selected-only plus Contacts/NonContacts/NewChats/ExistingChats and keep include/exclude scopes mutually consistent. Shortcut Messages are reusable persisted message templates. Working Hours normalize seven-day intervals, preserve at least minute-level ordering, support next-day closing ranges, enable/disable days, and bind a timezone with closest-timezone fallback after asynchronous timezone loading. These are applicable business messaging/availability responsibilities and remain production gaps.

## Account security 5550-5571

The cloud-password flow carries transient current/new password, hint, recovery email and checked recovery-code state across typed steps. Password entry validates confirmation and routes create/check/change/recovery flows. Recovery email setup/confirmation serializes requests and distinguishes flood, invalid/expired code/email and remote password-change errors. Password recovery supports pending reset, cancel reset, ready reset and recovery-code-based replacement/removal. Manage requires a verified current password, auto-exits after ten minutes idle, and clears transient credentials when leaving. `PASSWORD_HASH_INVALID` or `SRP_PASSWORD_CHANGED` is treated as remote revocation: the entire 2SV stack is invalidated and StepData cleared. Login-email verification binds expected code length and account state. Success animation is presentation only.

Current Fabushi repository searches did not prove an equivalent complete 2SV/SRP/recovery-email state machine. These rows therefore remain critical mapped-open security responsibilities. No secret value was copied into evidence. All 31 rows remain `unknown_closed=false`, `omitted=false`.
