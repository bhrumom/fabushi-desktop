# Recursive long-tail product capabilities

Status: FBCP P0 source-informed owner-resolution dossier
Frozen Telegram source: telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677
Canonical Fabushi architecture snapshot: PR #20 010ef77aa2502b6301b92b542255cff7a5285f81

All rows below come from the frozen source and are absorbed into the existing Fabushi product architecture. Telegram APIs and wire models are research evidence only.

## COMMUNITY-MANAGED
Sources: api/api_communities.cpp, data/data_community.*, info/community*. Behavior includes create/mutate community, member/admin state and peer/bot limits. Plausible existing owners: Shared Room, member/admin permissions, sidebar/conversation projection, Human/Agent member model, or a new community subsystem. `existing_owner = existing Shared Room + member/admin permissions + sidebar/conversation projection`; community is a room/member policy projection, not a second room truth. Requirements: room/member revisions, idempotent join/invite/remove/role changes, skipped-version refresh, Human+Agent membership, restart and large rosters.

## COMPOSE-AI
Sources: api/api_compose_with_ai.cpp, data/data_ai_compose_tones.*, compose AI boxes. Behavior applies named/custom/default tones to editable text with request/result lifecycle. Plausible existing owners: Composer, Agent/Coordinator/Host/Runner, Plugins/MCP, Automations, or a new AI-compose runtime. `existing_owner = existing Composer + Agent/Coordinator/Host/Runner`; Plugins/MCP is only an external capability boundary. Generated text remains draft state, so no second compose truth is needed. Generated text remains a draft until normal user send. Tests must cover supersession, cancellation, user-edit races, account/conversation switches, IME and accessibility.

## RICH-TASKS-TODO
Sources: api/api_rich_tasks.cpp, api/api_todo_lists.cpp, data/data_todo_list.*. Rich-task toggles are editable typed state; todo creation uses normal send options including reply, schedule and silent policy. Plausible existing owners: Task, Automations, typed Transcript/cards, Composer, Shared Room, or a new todo subsystem. `existing_owner = existing Task/Automations + typed Transcript/cards + Composer`; list/item state extends typed work/message state instead of creating a Telegram-style task product. Requirements: stable task/list/item IDs, revision settlement, duplicate/stale edit fencing, restart and concurrent Human/Agent edits.

## NOTIFICATION-RINGTONE
Sources: api/api_ringtones.cpp, ringtone settings/boxes, Resources/sounds. Upload is resource lifecycle followed by notification-policy selection. Plausible existing owners: notifications/platform lifecycle, settings, attachments/resource lifecycle, call/realtime audio, or a new ringtone owner. `existing_owner = existing notifications/settings + attachments/resource lifecycle`; uploaded audio is a resource selected by notification policy, not a separate media truth. Tests: upload cancel, corrupt/missing asset fallback, mute/silent policy, restart and Windows/macOS/Linux packaged playback.

## SELF-DESTRUCT-POLICY
Source: api/api_self_destruct.cpp. Account TTL and default history TTL are independently loaded/updated and superseded requests are canceled. Plausible existing owners: account lifecycle, settings/permissions, Transcript lifecycle, Automations, or a new expiry service. `existing_owner = existing account lifecycle + settings/permissions + Transcript lifecycle`; scheduling infrastructure may execute expiry, but policy and tombstone semantics remain with existing owners. Requirements: policy revision, expiry scheduling, tombstone/delete convergence, retention/export interaction and safe account deletion.

## SENSITIVE-CONTENT
Source: api/api_sensitive_content.cpp. State is loaded/enabled/can-change, with stale load/save cancellation. Plausible existing owners: settings, permissions/safety policy, Transcript presentation, attachments/resource presentation, or a new safety core. `existing_owner = existing settings + permissions/safety policy`; Transcript/resource surfaces consume the policy and do not become its canonical owner. Transcript/resource presentation consumes policy but does not own it. Tests: fail-closed unknown state, stale save, account switch, media policy, accessibility and audit.

## ROOM-STATISTICS
Sources: api/api_statistics.cpp, statistics/*, channel statistics info. These are derived member/view/share/reaction/story metrics and graphs. Plausible existing owners: Shared Room/room-info projection, search/analytics projection, operational telemetry, or a new statistics store. `existing_owner = existing Shared Room/room-info read-only projection`; operational telemetry is explicitly rejected as product-statistics truth. Requirements: scope/permission, time range/version, pagination/graph loading, privacy aggregation and large-data/export behavior.

## IDENTITY-USERNAME-WEBSITE
Sources: api/api_user_names.cpp and api/api_websites.cpp. Usernames are active/editable aliases; web authorizations have explicit list/terminate/revoke lifecycle. Plausible existing owners: identity/profile, account/auth/session security, deep-link/navigation security, settings, or a new username/web-auth subsystem. `existing_owner = existing identity/profile + account/auth/session security + deep-link/navigation security`; aliases and web authorizations extend existing identity/session state. Tests: uniqueness, stale revoke, malicious URL/origin, account switch and deep-link validation.

## MEDIA-EDITOR
Source: editor/photo_editor.cpp and crop/paint/layer controllers. Plausible existing owners: Composer, attachments/resource lifecycle, media viewer, Artifacts, or a new editor store. `existing_owner = existing Composer + attachments/resource lifecycle + media viewer`; edit state is a non-destructive pre-send resource transformation, not another media store. Edit state is a non-destructive pre-send resource transformation, not a second media store. Tests: derived-resource provenance, undo/cancel, large-media bounds, orientation, draft restart and accessibility.

## RICH-DOCUMENT-VIEW
Sources: iv/*, including cached media, rich-page search/zoom and HTML export. Plausible existing owners: Artifacts, safe Web/rich-content rendering, transcript cards, navigation, Plugins/MCP, or a new rich-document subsystem. `existing_owner = existing Artifacts + safe Web/rich-content rendering + transcript-card/navigation owners`; cached/rendered rich content remains a presentation of canonical artifact/transcript data. Requirements: sanitization, origin/navigation policy, cache bounds, offline behavior, export, search/zoom and accessibility.

## SUPPORT-MODERATION
Sources: api/api_report.cpp, report/moderation boxes and support/*. Report flow has reason/options/comment/terminal settlement; moderation is permission-scoped. Plausible existing owners: feedback/support, permissions/Shared Room admin, Transcript actions, safety policy, or a new moderation product. `existing_owner = existing feedback/support + permissions/Shared Room admin + Transcript action surfaces`; native abuse enforcement may sit below these owners without becoming a second product architecture. Native abuse enforcement may sit below these owners. Tests: idempotent report, report/block interaction, moderation races, offline retry, audit/redaction and resource limits.

## TDE2E-SECURITY
Source: tde2e/* including tde2e_api.cpp. Frozen behavior exposes temporary key generation, participant permissions, scoped encrypt/decrypt and call/channel key lifecycle. Plausible existing owners: identity/device security, native messaging/sync security, ADR-003 CallSession, Shared Room permissions, or a new cryptographic product model. `existing_owner = existing identity/device security + native messaging/sync security + ADR-003 CallSession + Shared Room permissions`; Telegram TDE2E protocol is explicitly not reused. Telegram TDE2E protocol is not reused. Requirements: Fabushi threat model, authenticated device keys, rotation/revocation, replay/order binding, secure erase, device loss, group membership changes, call-key lifetime and independent security review.
