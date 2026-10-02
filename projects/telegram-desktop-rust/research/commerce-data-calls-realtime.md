# Commerce, data lifecycle, shell quality, calls and realtime

Status: FBCP P0 source-informed owner-resolution dossier
Frozen Telegram source: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Canonical Fabushi architecture snapshot: PR #20 `c2767eac1383fd8db7be9b536e5acaf4a4d2b7f0`

This dossier extracts product requirements from the frozen Telegram source without importing Telegram wire, MTProto, payment rails, identity documents, call protocols or UI architecture into Fabushi.

## PREMIUM-ENTITLEMENT

Frozen evidence: `api/api_premium.cpp`, `data/data_premium_limits.*` and subscription-option models separate entitlement state, plan/options and promotional presentation. Requests are cancelable and entitlement changes trigger dependent refresh.

Plausible owners: settings/product shell, account/session, Plugins, a new billing owner. Selected owner: **existing account/session entitlement projection + settings/product shell presentation**. Billing/checkout is not owned here.

Exact paths: `source/electron-main/account/*`, `source/host/src/extensions/session/*`, `frontend/src/recovered/features/settings/overlay/*`, `frontend/src/production/ProductionRenderer.tsx`.

Requirements: durable account-scoped entitlement/version, feature-gate projection, expiry/revocation, device/account switch convergence, idempotent refresh. Tests: stale entitlement, expiry, downgrade while feature active, account switch, offline cache, restart. No Fabushi paid-entitlement backend exists yet.

## CREDITS-GIFTS

Frozen evidence: `api/api_credits.cpp` exposes balance, history/subscriptions and paged tokens; `data/data_star_gift.cpp` exposes owned/resale gift identity, rarity and value. Product lesson: value ledger settlement is distinct from presentation/resource metadata.

Plausible owners: transcript typed events, artifacts/resources, account, payment settlement, new credits/gifts subsystem. Selected owner: **existing transcript/resource/product shell for presentation plus the minimal payment/value-settlement infrastructure proposed in ADR FBCP-ADR-002 for any real value mutation**. No Telegram Stars currency is adopted.

Requirements: Fabushi-defined value/entitlement IDs only if product policy enables them; immutable ledger operation ID, authorization, balance/version reconciliation, gift/resource provenance, fraud/abuse limits. Tests: duplicate debit/credit, stale balance, refund/reversal, transfer replay, account isolation, restart. Product/business applicability remains a blocker; no economic feature is marked implemented.

## BUSINESS-WORKFLOWS

Frozen evidence: `data/business/*` and `settings/business/*` group greeting/away behavior, automated responses, opening/profile controls and chatbot delegation, but these are separable workflows and policies.

Plausible owners: Automations, Agent, settings, permissions, composer/transcript. Selected owner: **existing Automations + Agent + settings/permissions + Composer/Transcript**, capability by capability; no broad Business subsystem.

Requirements: scoped automation identity, schedule/trigger, audience rules, Agent delegation, auditability, cancellation, deterministic once-only action. Tests: overlapping rules, blocked recipient, stale settings, retry/restart, Agent unavailable, permission revoke.

## PAYMENT-SETTLEMENT

Frozen evidence: `payments/payments_checkout_process.cpp`, `payments_form.cpp`, credits APIs and Stripe build dependency show explicit invoice/form validation, authorization, submission, terminal success/failure and retry boundaries.

No current canonical Fabushi owner safely owns money movement. Rejected owners: settings (configuration only), Transcript (projection/history only), Session (generic durable conversation state), Automations (triggering only), Plugins/MCP (external capability boundary, not financial truth), Agent (cannot be authoritative payer).

Resolution: **existing_owner = none; minimal new infrastructure owner proposed by `docs/adr/FBCP-ADR-002-payment-settlement.md`**. This owner is only for Fabushi-approved payment/value settlement if/when commerce is applicable; it is not a Telegram payment provider.

## IDENTITY-DOCUMENT-SECURITY

Frozen evidence: `passport/*`, `passport_encryption.cpp`, cloud-password/WebAuthn/passkey sources distinguish encrypted sensitive document/credential material from UI and from ordinary profile identity.

Plausible owners: account/auth, secrets, attachments/artifacts, settings, new Passport subsystem. Selected owner: **existing account/auth + secrets/security + attachment/resource lifecycle**. Identity-document verification is conditional on a future Fabushi product requirement; Telegram Passport semantics are not adopted.

Requirements: explicit consent/purpose, encrypted-at-rest sensitive resource, least-privilege access, expiration/revocation, redaction/export/delete policy, request-generation fencing. Tests: wrong key, stale request, revoke, account switch, corrupted secret, export/delete policy. Any regulated identity verification backend remains not-configured.

## EXPORT-DATA

Frozen evidence: `export/export_manager.cpp`, controller/settings sources expose a cancelable, progress-bearing export lifecycle over selected data categories.

Plausible owners: settings, Session, attachments/artifacts, product shell, new exporter. Selected owner: **existing settings + Session/data lifecycle + attachments/artifacts**, with a narrow export worker below them if needed.

Requirements: stable export job ID, selected scopes, consistent snapshot/version, progress/cancel/resume policy, path safety, sensitive-data filtering. Tests: cancellation, restart, partial failure, concurrent mutation, large histories/resources, path denial, account switch.

## STORAGE-MIGRATION-RECOVERY

Frozen evidence: `storage/localstorage.cpp`, `storage_account.cpp`, `storage_domain.cpp`, file locks, sparse lists and download state demonstrate versioned persistence, account partitioning, migration, locking and recovery.

Plausible owners: Host Session SQLite/recovery, Electron startup/data migration, attachments/cache, new storage core. Selected owner: **existing Host Session durable-state/recovery + Electron startup migration + existing attachment/cache owners**. No second canonical store.

Requirements: schema/version migration, atomic write/transaction boundaries, corruption detection, file/process lock discipline, account isolation, bounded cache/eviction, crash/restart recovery and observability. Tests: interrupted migration, corrupt DB, stale lock, disk-full, concurrent open, rollback, account switch.

## THEME-I18N-A11Y

Frozen evidence: `data/data_cloud_themes.cpp` uses versioned theme refresh; `lang/lang_cloud_manager.cpp` applies version/difference updates; UI/platform trees carry RTL, keyboard/focus and accessibility behavior.

Plausible owners: existing root shell/theme tokens, settings, product shell/platform, new localization runtime. Selected owner: **existing root shell/theme + settings + platform/accessibility composition**. A localization catalog/runtime may be added as a narrow shell service, not a second product owner.

Exact paths include `frontend/src/recovered/features/runtime-theme-token-installer.ts`, `frontend/src/recovered/features/settings/overlay/*`, `frontend/src/recovered/features/window-chrome/*`, `frontend/src/production/ProductionRenderer.tsx`.

Requirements: locale/version, RTL layout direction, plural/format rules, IME-safe Composer behavior, keyboard/focus order, screen-reader labels/live regions, high DPI/multi-screen/theme persistence. Tests must cover RTL, CJK IME composition, keyboard-only, screen reader semantics, zoom/DPI, theme/locale restart. Full localization/a11y packaged acceptance remains open.

## CALL-REALTIME

Frozen evidence: `calls/calls_call.cpp` has explicit incoming/outgoing/waiting/requesting/key-exchange/active/finish states, ring/hangup timeouts and device bindings; `calls_controller_webrtc.cpp` separates signaling data, media endpoints, encryption key, network type and media-device control.

Current owner analysis: Conversation/Shared Room can own participant/context projection; Computer owns computer handoff/screen-control product behavior, not call signaling truth; Transcript can record typed call events but cannot own active call state; Electron platform can bridge devices but cannot own cross-device session state.

Resolution: **existing_owner = none for call-session/signaling state; minimal new owner proposed by `docs/adr/FBCP-ADR-003-call-session-signaling.md`**. Existing Computer remains the screen-sharing/control integration surface where semantically appropriate; call media/signaling does not move into Computer.

## NETWORK-SYNC-REALTIME

Frozen evidence: MTProto/update machinery, storage sparse histories, session/update handling and call network transitions demonstrate reconnect, sequence/gap detection, duplicate suppression, stale-update rejection, proxy/network change, sleep/wake and multi-device convergence requirements.

Plausible owners: Session/Transcript, Electron platform lifecycle, Shared Room/presence, new CommunicationCore. Selected owner: **existing product owners plus minimal native messaging/sync/presence/media/call infrastructure below them**. A parallel CommunicationCore is explicitly rejected.

Requirements: stable device/session and event IDs, durable outbox, attempt/ack settlement, idempotency, per-conversation order, gap recovery, reconnect/resume, multi-device revision/read state, backpressure/cancel, stale drop, sleep/wake/network transition. This row is the network-requirements aggregation gate for P0; implementation starts only after P0 owner/source closure.

## UPDATE-INSTALL-ROLLBACK

Frozen evidence: updater platform sources, update keys/verify/unpack, `Telegram.plist`, Windows updater resources and signing/package scripts show signed metadata, staged install and restart/rollback boundaries.

Plausible owners: Electron update/platform lifecycle, startup migration, Host lifecycle. Selected owner: **existing Electron update/platform owner**.

Exact paths: `source/electron-main/update/*`, startup migration and existing window/platform lifecycle.

Requirements/tests: signed metadata, interrupted download, apply/relaunch gates, migration compatibility, rollback/failure, pending durable communication state, Windows/macOS/Linux packaging. Packaged exact-head acceptance remains open.
