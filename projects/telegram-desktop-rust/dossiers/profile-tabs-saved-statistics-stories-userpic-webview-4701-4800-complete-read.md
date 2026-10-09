# Exact-source dossier: orders 4701-4800

- Accepted upstream: `telegramdesktop/tdesktop@36a0c87ca096c48ccf6193aa2c31707fdcafcc7c`
- Root tree: `94e009f981d886ee55cd0450f0a5cc9b38305ba8`
- Read method: direct exact-blob source/symbol/RPC/state/lifecycle reading; read-complete does not imply implementation or release closure.
- Accounting after batch: read-through 4,800; unread 11,320; unknown 15,841; unknown-closed 279; omitted 0.

## Responsibility decomposition

### 4701-4702: Profile related-media navigation/count lifecycle
- Canonical owner: Canonical ProfileSection related-content owner
- State machine: reactive counts -> capability row visibility -> activate canonical related section
- Side effects/ownership: common-groups/similar-peers/stories/saved navigation and counts stay in canonical ProfileSection
- Failure/lifecycle boundary: stale count, unavailable destination, destroyed profile, navigation race
- Disposition: `mapped-open-profile-related-content`; implementation and release verification remain open.

### 4703-4706: Profile status and reactive text lifecycle
- Canonical owner: Canonical ProfileSection owner
- State machine: peer/text updates -> normalized status/text -> conditional selectable presentation -> refresh/actions
- Side effects/ownership: status, hidden/community links and labeled text stay in canonical ProfileSection
- Failure/lifecycle boundary: stale presence/text, timer after destroy, action/theme/a11y regression
- Disposition: `mapped-open-profile-section`; implementation and release verification remain open.

### 4707-4711: Profile top-bar identity/action lifecycle
- Canonical owner: Canonical Profile header owner
- State machine: identity/avatar/story/status/badges -> permission-aware toolbar actions -> animated/theme settlement
- Side effects/ownership: profile identity, media badges and actions stay in one canonical header
- Failure/lifecycle boundary: permission/media drift, animation leak, focus/theme/responsive regression
- Disposition: `mapped-open-profile-header`; implementation and release verification remain open.

### 4712-4715: Profile values and navigation lifecycle
- Canonical owner: Canonical Participant/Profile DetailPanel owner
- State machine: peer/topic/privacy values -> profile projection -> navigation/tabs/scroll -> save/restore
- Side effects/ownership: profile value security/privacy and DetailPanel state stay canonical
- Failure/lifecycle boundary: unsafe link/privacy leak, memento/scroll/tab state loss
- Disposition: `mapped-open-profile-detail`; implementation and release verification remain open.

### 4716-4739: Profile tabs/search/media/member lifecycle
- Canonical owner: Canonical Tabs/Search/ProfileSection owner
- State machine: tab descriptors + visibility -> active body/search/selection/scroll -> switch/save/restore
- Side effects/ownership: shared media, members, peers, polls, saved, stories and tab gesture state reuse canonical owners
- Failure/lifecycle boundary: stale query/provider/tab state, selection/scroll race, a11y/responsive regression
- Disposition: `mapped-open-profile-tabs`; implementation and release verification remain open.

### 4740-4741: Reaction participant list lifecycle
- Canonical owner: Canonical Message reactions owner
- State machine: reaction context -> reaction tabs/participant list -> selection/scroll -> save/restore
- Side effects/ownership: reaction participants stay with canonical message reactions
- Failure/lifecycle boundary: stale reaction/list, state loss
- Disposition: `mapped-open-reactions-list`; implementation and release verification remain open.

### 4742-4743: Join/request participant list lifecycle
- Canonical owner: Canonical Community Requests owner
- State machine: request state -> search/list/moderation rows -> scroll -> save/restore
- Side effects/ownership: request moderation/search stay canonical
- Failure/lifecycle boundary: stale query/request, permission loss, restore mismatch
- Disposition: `mapped-open-requests-list`; implementation and release verification remain open.

### 4744-4753: Saved music and sublists lifecycle
- Canonical owner: Canonical Saved Messages/Music owner
- State machine: saved collections -> list/provider/preload/search/selection/reorder -> navigation -> save/restore
- Side effects/ownership: saved music/sublists reuse canonical Saved/Conversation/Resource owners
- Failure/lifecycle boundary: stale slice, preload/reorder/search/selection/restore race
- Disposition: `mapped-open-saved-content`; implementation and release verification remain open.

### 4754-4755: Settings wrapper/memento lifecycle
- Canonical owner: Canonical Settings owner
- State machine: settings key -> section/wrap/topbar/back/stack -> save/restore
- Side effects/ownership: settings routing/state stays canonical
- Failure/lifecycle boundary: invalid section, stack/scroll/state race
- Disposition: `mapped-open-settings`; implementation and release verification remain open.

### 4756-4757: Similar peers discovery lifecycle
- Canonical owner: Canonical Peer Discovery owner
- State machine: peer + entitlement -> load/paginate similar rows -> open -> save/restore
- Side effects/ownership: related peer discovery stays canonical
- Failure/lifecycle boundary: premium gate drift, stale page, duplicate row
- Disposition: `mapped-open-peer-discovery`; implementation and release verification remain open.

### 4758-4769: Statistics dashboard/export/navigation lifecycle
- Canonical owner: Canonical Analytics/Statistics owner
- State machine: stats/lists/graphs/previews -> render/zoom/expand/export/navigation -> save/restore
- Side effects/ownership: analytics data, chart/list/export/file actions stay canonical
- Failure/lifecycle boundary: load/zoom/export/write/navigation/restore failure, stale graph/list
- Disposition: `mapped-open-statistics`; implementation and release verification remain open.

### 4770-4778: Story albums/provider/section lifecycle
- Canonical owner: Canonical Story capability owner
- State machine: album ids/forms -> create/edit/add-remove/reorder/share/delete -> media preload/selection -> navigation/save
- Side effects/ownership: story album mutations, media provider and section state stay canonical
- Failure/lifecycle boundary: validation/permission/RPC/preload/selection/scroll state failure
- Disposition: `mapped-open-stories`; implementation and release verification remain open.

### 4779-4797: Avatar/userpic builder lifecycle
- Canonical owner: Canonical Avatar editor owner
- State machine: emoji/sticker/colors -> async resources/preview -> gradient/editor/dialog -> save/cancel result
- Side effects/ownership: avatar generation uses canonical Avatar/Picker/Dialog/Resource owners
- Failure/lifecycle boundary: resource/timer/player stale callback, save/cancel race, theme/a11y/responsive regression
- Disposition: `mapped-open-avatar-editor`; implementation and release verification remain open.

### 4798-4799: Bot attach WebView capability/session lifecycle
- Canonical owner: Canonical Plugin/MCP App Surface owner
- State machine: eligible app/source/peer -> permission/install -> web app session/events/storage/download/payment/share -> prolong/close
- Side effects/ownership: app permissions/session/resource actions map to Fabushi Plugin/MCP App Surface; no Telegram transport in shipping runtime
- Failure/lifecycle boundary: origin/permission/session mismatch, stale instance, payment/download/share/cancellation leak
- Disposition: `mapped-open-plugin-app-surface`; implementation and release verification remain open.

### 4800-4800: Prepared app result confirmation lifecycle
- Canonical owner: Canonical Plugin/MCP App Surface + Composer owner
- State machine: prepared content + recipient -> preview -> confirm/send or cancel -> settled conversation
- Side effects/ownership: untrusted app-prepared content requires canonical preview/confirmation before send
- Failure/lifecycle boundary: recipient change, stale preview/media, double submit, cancel/send race
- Disposition: `mapped-open-plugin-prepared-send`; implementation and release verification remain open.

## Exact-source attestation

Every order 4701-4800 is bound to its accepted-tree blob in `inventory/source-attestation-manifests/4701-4800.json` and the matching `source_binary_evidence` row. Current-head GitHub Actions Source authority must regenerate exact blob, consumer trace, and reachability evidence before this batch is accepted as current-head provenance.

No entry in this batch is promoted to implemented/verified solely by this read.
