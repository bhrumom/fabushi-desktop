# Stories, notifications, privacy, credential security, Bots and Mini Apps

Status: FBCP P0 source-informed owner-resolution dossier  
Frozen Telegram source: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`  
Canonical Fabushi architecture snapshot: PR #20 `c2767eac1383fd8db7be9b536e5acaf4a4d2b7f0`

This dossier records product behavior learned from the frozen Telegram source and resolves it into the current Fabushi product architecture. Telegram wire types, MTProto, Telegram Bot APIs and Telegram WebApp runtime are research inputs only; none becomes a Fabushi runtime dependency or product truth.

## STORY-LIFECYCLE

### Frozen behavior evidence

Representative source includes `Telegram/SourceFiles/data/data_story.cpp`, `data_stories.*`, `media/stories` and `info/stories`. The frozen implementation treats a Story as independently identified, audience/privacy-scoped publication state with expiry, pinning/highlight state, view/reaction aggregates and authoritative updates. Story media is still resource/media state rather than a second media subsystem.

### Owner analysis

Plausible existing owners: product shell/navigation, conversation workspace, Transcript, attachments/artifacts/resource lifecycle, Human identity/account, Shared Room/member/permissions, Session durable state, or a new Story subsystem.

Selected existing owners: **existing product shell + Session durable state + attachments/resource lifecycle + existing identity/permissions policy**. The shell owns presentation/navigation, Session owns durable canonical publication projection, attachments own binary resources, and identity/permissions own audience policy. Transcript is explicitly rejected as canonical Story truth because expiry/highlight/feed lifecycle is not message-history lifecycle. A broad new Story runtime is therefore not justified at P0.

Exact owner evidence paths:
- `frontend/src/production/ProductionRenderer.tsx`
- `frontend/src/recovered/features/conversation/workspace/*` for shared navigation/workspace integration only
- `source/host/src/extensions/session/*`
- `source/host/src/extensions/attachments/*`
- `source/electron-main/attachments/*`
- `source/host/src/extensions/local_tool_permission/*` and existing account/member policy boundaries

Absorption: add a typed Fabushi publication/story projection under the existing shell, referencing canonical Human/Agent identity and existing resources. Do not add a Telegram sidebar, Telegram Story model, or a second identity/media store.

Persistence/native-network requirements: stable publication ID, author ID, resource references, created/expiry timestamps, audience policy revision, pin/highlight state, view/reaction counters, tombstone/expiry semantics, idempotent publication/update/delete operations, ordered update sequence and reconnect/gap recovery.

Focused acceptance: expiry across restart/sleep; privacy audience changes; duplicate/out-of-order view/reaction updates; delete/expiry races; missing resource recovery; mixed Human/Agent author projection; accessibility/keyboard navigation; no transcript duplication. Native publication/sync transport remains unimplemented.

## NOTIFICATION-TRAY

### Frozen behavior evidence

Representative source includes `Telegram/SourceFiles/platform/platform_notifications_manager.h`, platform notification implementations, tray/badge sources and notification settings. Notification delivery is a projection of unread/activity and policy; dismissal/activation routes back to the addressed conversation, while tray/badge state is lifecycle/platform state rather than message truth.

### Owner analysis

Plausible owners: Electron main notifications, Host notifications, Host trays, sidebar unread projection, settings, or a new notification service.

Selected existing owners: **existing Electron notification/platform lifecycle + Host notifications/trays**, with future native push infrastructure below them.

Exact owner evidence paths:
- `source/electron-main/notifications/os-notification-manager.ts`
- `source/electron-main/notifications/dock-badge-manager.ts`
- `source/host/src/extensions/notifications/extension.rs`
- `source/host/src/extensions/notifications/mobile_push_notifier.rs`
- `source/host/src/extensions/trays/trays_service.rs`
- existing sidebar/conversation unread owners

Absorption: Human/Agent/Group/Channel activity feeds the same notification policy and activation route. No Human-only tray or Telegram notification stack.

Persistence/native-network requirements: push subscription/device token lifecycle, notification event identity/dedupe, mute/silent policy, unread/badge convergence, action/deep-link identity, restart and multi-device clear/read convergence.

Focused acceptance: duplicate push, notification-before-history gap, mute/silent, action routing, stale badge, restart, logout/device revocation, Windows/macOS/Linux notification behavior, sleep/wake. Native Fabushi push service is still absent.

## PRIVACY-BLOCKING

### Frozen behavior evidence

`Telegram/SourceFiles/api/api_user_privacy.cpp` maintains reload/save rule state with allow/deny exceptions, while `api/api_blocked_peers.cpp` exposes explicit block/unblock and paged blocked-peer state. These are identity policy operations, not transcript deletion.

### Owner analysis

Plausible owners: settings, permissions, account/Human identity, Shared Room/member policy, Transcript, or a new safety core.

Selected existing owners: **existing settings + account/Human identity + permissions/member policy**. Transcript may project consequences but cannot own block/privacy truth.

Exact owner evidence paths:
- `frontend/src/recovered/features/settings/overlay/*`
- `source/host/src/extensions/settings/*`
- `source/electron-main/account/human-identity.ts`
- `frontend/src/production/group-members-root.ts`
- `source/host/src/extensions/local_tool_permission/*`
- `source/host/src/extensions/session/session_roster.rs`

Absorption: add typed contact/member privacy and block policy to existing settings/account/member flows. Blocking must not fork a second contact store and must not silently delete canonical history.

Persistence/native-network requirements: account-scoped policy revision, block operation idempotency, stale-write rejection, paged blocked identities, server/device convergence, membership/send admission enforcement, audit/security observability.

Focused acceptance: block/unblock replay, stale privacy write, blocked send/mention/invite, unblock history continuity, account switch, multi-device convergence, abuse/resource-limit interaction. Native identity-policy sync remains unimplemented.

## LOCAL-CREDENTIAL-WEBAUTHN

### Frozen behavior evidence

Representative source includes `Telegram/SourceFiles/api/api_cloud_password.cpp`, `webauthn/*`, platform WebAuthn and passkey data. The frozen behavior separates credential/password recovery lifecycle from WebAuthn ceremony state and rejects stale or mismatched request completion.

### Owner analysis

Plausible owners: Electron account/auth, secrets, OnePassword bridge, Host auth, Host WebAuthn proxy, settings, or a new security runtime.

Selected existing owners: **existing account/auth + secret storage + Host WebAuthn proxy**, with settings as configuration UI. A broad second security core is unnecessary.

Exact owner evidence paths:
- `source/electron-main/account/account-authorization.ts`
- `source/electron-main/account/production-account-authorization.ts`
- `source/electron-main/auth/auth-callback-registration.ts`
- `source/electron-main/secrets/secret-store.ts`
- `source/electron-main/secrets/user-secrets-store.ts`
- `source/host/src/extensions/auth/*`
- `source/host/src/extensions/webauthn_proxy/extension.rs`
- `source/host/src/extensions/webauthn_proxy/webauthn_proxy_bridge.rs`

Absorption: local app lock, passkey/WebAuthn, recovery and credential revocation extend the current account/security boundaries. Secrets never become transcript or renderer canonical state.

Persistence/native-network requirements: device credential identity, secure-at-rest secret storage, challenge/request generation fencing, replay prevention, recovery/revocation state, device-session binding, logout wipe/cutover.

Focused acceptance: stale ceremony result, cancel/retry, restart mid-ceremony, credential corruption, revoked device, account switch, platform unavailable = not-configured, biometric/WebAuthn user-cancel. Full local-lock UX and Fabushi-native device-session service remain open.

## BOT-INLINE

### Frozen behavior evidence

Representative source includes `Telegram/SourceFiles/api/api_bot.cpp`, `inline_bots/*` and bot-command/menu/callback flows. The useful product behavior is typed command/suggestion discovery, addressed invocation, callback/result correlation and inline result insertion; Telegram Bot API identity/wire semantics are not required.

### Owner analysis

Plausible owners: Agent model, Composer, transcript cards, Plugins/MCP, Coordinator/Host/Runner, or a Telegram Bot runtime.

Selected existing owners: **existing Agent + Composer + typed transcript-card interaction + Coordinator/Host/Runner**, with Plugins/MCP for external capabilities. A Telegram Bot runtime is explicitly rejected.

Exact owner evidence paths:
- `frontend/src/recovered/features/conversation/workspace/composer.tsx`
- `frontend/src/recovered/features/conversation/cards/transcript-card/*`
- `frontend/src/recovered/features/plugins/overlay/*`
- `source/host/src/extensions/mcp/*`
- existing Coordinator/Host/Runner production composition

Absorption: inline Agent/Plugin suggestions and callbacks remain typed Fabushi actions in the same Composer/transcript, preserving Agent-native events instead of flattening them into Telegram-style text.

Persistence/native-network requirements: invocation/result identity, cancellation, permission/approval context, idempotent callback settlement, stale-result fencing, reconnect recovery for durable actions.

Focused acceptance: query supersession, stale inline result, callback duplicate, permission denial, Agent/tool failure, restart around durable callback, keyboard/a11y. No Telegram bot network dependency is allowed.

## MINIAPP-WEBVIEW

### Frozen behavior evidence

`Telegram/SourceFiles/inline_bots/bot_attach_web_view.cpp` shows WebView request/session identity, query/result correlation, close/cancel behavior and bridge-delivered data. The reachable `lib_webview` gitlink confirms a native embedded-web surface is part of upstream behavior research.

### Owner analysis

Plausible owners: Plugins/MCP, existing plugin browser overlay, Computer/Web capability, Electron shell/security, Transcript cards, or a new Mini App runtime.

Selected existing owners: **existing Plugins/MCP + plugin browser/Web surface + Electron security boundary**. A separate Mini App product shell is rejected.

Exact owner evidence paths:
- `frontend/src/recovered/features/plugins/overlay/browser.tsx`
- `frontend/src/recovered/features/plugins/overlay/desktop-surface.tsx`
- `source/host/src/extensions/mcp/*`
- `source/host/src/extensions/browser_ua/extension.rs`
- existing Electron navigation/window security boundaries

Absorption: plugin-provided interactive web content opens inside the existing plugin/web capability with explicit Agent/conversation context and typed result events. It must not create a second sidebar, conversation truth or provider runtime.

Persistence/native-network requirements: session/query ID, origin/capability allowlist, permission grants, bridge message schema/version, cancellation/close, bounded storage, restart policy, external-navigation policy and CSP/sandboxing.

Focused acceptance: untrusted origin, navigation escape, duplicate/stale result, close-before-result, account/conversation switch, permission revoke, restart, clipboard/file access policy, a11y/focus. Security hardening and durable web-session policy remain open.
