# Recursive long-tail product capabilities

Status: FBCP P0 source-informed owner-resolution dossier
Frozen Telegram source: telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677
Canonical Fabushi architecture snapshot: PR #20 95995bdf36a9687788e106c8544d292b2bb0877f

All rows below come from the frozen source and are absorbed into the existing Fabushi product architecture. Telegram APIs and wire models are research evidence only.

## COMMUNITY-MANAGED
Sources: api/api_communities.cpp, data/data_community.*, info/community*. Behavior includes create/mutate community, member/admin state and peer/bot limits. Owner: existing Shared Room + member/admin permissions + sidebar/conversation projection. Requirements: room/member revisions, idempotent join/invite/remove/role changes, skipped-version refresh, Human+Agent membership, restart and large rosters.

## COMPOSE-AI
Sources: api/api_compose_with_ai.cpp, data/data_ai_compose_tones.*, compose AI boxes. Behavior applies named/custom/default tones to editable text with request/result lifecycle. Owner: existing Composer + Agent/Coordinator/Host/Runner; Plugins/MCP only for an external capability. Generated text remains a draft until normal user send. Tests must cover supersession, cancellation, user-edit races, account/conversation switches, IME and accessibility.

## RICH-TASKS-TODO
Sources: api/api_rich_tasks.cpp, api/api_todo_lists.cpp, data/data_todo_list.*. Rich-task toggles are editable typed state; todo creation uses normal send options including reply, schedule and silent policy. Owner: existing Task/Automations + typed Transcript/cards + Composer. Requirements: stable task/list/item IDs, revision settlement, duplicate/stale edit fencing, restart and concurrent Human/Agent edits.

## NOTIFICATION-RINGTONE
Sources: api/api_ringtones.cpp, ringtone settings/boxes, Resources/sounds. Upload is resource lifecycle followed by notification-policy selection. Owner: existing notifications/settings + attachments/resource lifecycle. Tests: upload cancel, corrupt/missing asset fallback, mute/silent policy, restart and Windows/macOS/Linux packaged playback.

## SELF-DESTRUCT-POLICY
Source: api/api_self_destruct.cpp. Account TTL and default history TTL are independently loaded/updated and superseded requests are canceled. Owner: existing account lifecycle + settings/permissions + Transcript lifecycle. Requirements: policy revision, expiry scheduling, tombstone/delete convergence, retention/export interaction and safe account deletion.

## SENSITIVE-CONTENT
Source: api/api_sensitive_content.cpp. State is loaded/enabled/can-change, with stale load/save cancellation. Owner: existing settings + permissions/safety policy. Transcript/resource presentation consumes policy but does not own it. Tests: fail-closed unknown state, stale save, account switch, media policy, accessibility and audit.

## ROOM-STATISTICS
Sources: api/api_statistics.cpp, statistics/*, channel statistics info. These are derived member/view/share/reaction/story metrics and graphs. Owner: existing Shared Room/room-info read-only projection; operational telemetry is explicitly not product-statistics truth. Requirements: scope/permission, time range/version, pagination/graph loading, privacy aggregation and large-data/export behavior.

## IDENTITY-USERNAME-WEBSITE
Sources: api/api_user_names.cpp and api/api_websites.cpp. Usernames are active/editable aliases; web authorizations have explicit list/terminate/revoke lifecycle. Owner: existing identity/profile + account/auth/session security + deep-link/navigation security. Tests: uniqueness, stale revoke, malicious URL/origin, account switch and deep-link validation.

## MEDIA-EDITOR
Source: editor/photo_editor.cpp and crop/paint/layer controllers. Owner: existing Composer + attachments/resource lifecycle + media viewer. Edit state is a non-destructive pre-send resource transformation, not a second media store. Tests: derived-resource provenance, undo/cancel, large-media bounds, orientation, draft restart and accessibility.

## RICH-DOCUMENT-VIEW
Sources: iv/*, including cached media, rich-page search/zoom and HTML export. Owner: existing Artifacts + safe Web/rich-content rendering + transcript-card/navigation owners. Requirements: sanitization, origin/navigation policy, cache bounds, offline behavior, export, search/zoom and accessibility.

## SUPPORT-MODERATION
Sources: api/api_report.cpp, report/moderation boxes and support/*. Report flow has reason/options/comment/terminal settlement; moderation is permission-scoped. Owner: existing feedback/support + permissions/Shared Room admin + Transcript action surfaces. Native abuse enforcement may sit below these owners. Tests: idempotent report, report/block interaction, moderation races, offline retry, audit/redaction and resource limits.

## TDE2E-SECURITY
Source: tde2e/* including tde2e_api.cpp. Frozen behavior exposes temporary key generation, participant permissions, scoped encrypt/decrypt and call/channel key lifecycle. Owner: existing identity/device security + native messaging/sync security + ADR-003 CallSession + Shared Room permissions. Telegram TDE2E protocol is not reused. Requirements: Fabushi threat model, authenticated device keys, rotation/revocation, replay/order binding, secure erase, device loss, group membership changes, call-key lifetime and independent security review.
