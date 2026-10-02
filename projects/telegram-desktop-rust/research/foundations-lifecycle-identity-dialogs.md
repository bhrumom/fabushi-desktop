# Telegram foundations: lifecycle, account, identity/presence, and dialog-list research dossier

Status: researched and owner-resolved for the four earliest FBCP P0 domains; implementation/acceptance remains open
Frozen upstream: `telegramdesktop/tdesktop@33261535a0e747f125e0ed25486f01e556330677`
Canonical Fabushi architecture snapshot: PR #20 `bbc7b34a5f6dad46e3d4ca88fe21cc4f7932ce09`
Captured: 2026-10-02

This dossier records behavior learned from Telegram Desktop without adopting its account model, network, protocol, or product architecture. Each capability is resolved into the current Fabushi owner before any infrastructure proposal.

## APP-LIFECYCLE — startup, single-instance, window, quit/restart

### Frozen upstream evidence

- `Telegram/SourceFiles/main.cpp` creates `Core::Launcher` and makes launcher execution the process-lifetime root; startup failure returns a non-success exit rather than manufacturing a partially initialized UI.
- `Telegram/SourceFiles/main/main_domain.cpp` separates durable domain startup from active account/session activation and finishes the active domain before teardown.
- The broader frozen `core`, `window`, `platform`, update and storage areas are lifecycle dependencies; platform behavior is part of the product contract, not a separate chat subsystem.

### Current Fabushi candidates inspected

- Electron main application lifecycle: `source/electron-main/main.ts`.
- Startup migration/move/bootstrap owners: `source/electron-main/startup/*`.
- Window placement and persistence: `source/electron-main/window-state-persistence.ts`, `window-state-store.ts`, `window-chrome.ts`.
- Update/install owners: `source/electron-main/update/*`.
- Host lifecycle/telemetry and Coordinator supervision were considered but rejected as the product app-lifecycle owner: they are downstream process/runtime responsibilities.

### Selected existing owner

`existing_owner = Electron main/platform lifecycle`.

Shipping `source/electron-main/main.ts` already owns pre-single-instance bootstrap, single-instance fencing, deep-link activation, `app.whenReady`, service initialization, BrowserWindow creation, macOS reactivation, window-all-closed behavior and before-quit disposal. `startup-data-root-migration.ts` already fail-closes on unsafe/conflicting durable roots and live writers.

### Absorption / model / persistence / native-network requirements

- Communication startup must register beneath the existing service/bootstrap phase and must not create a second application root.
- Reconnect/sleep/wake/network-transition behavior may wake native sync, but lifecycle truth remains Electron/platform-owned.
- Durable communication state must settle before destructive migration/quit and resume from the same Session/Transcript state after restart.
- Update/install/rollback must preserve schema compatibility or fail closed with explicit recovery/migration.

### Focused tests / blockers

Single-instance activation; startup migration conflict/live-writer fencing; service-init failure before window; quit with durable send/sync pending; sleep/wake/network transition; update/restart persistence; Windows/macOS/Linux lifecycle. Native communication resume wiring is not implemented yet.

## ACCOUNT-AUTH-MULTI — account authorization, Human identity, sessions, revocation

### Frozen upstream evidence

- `Telegram/SourceFiles/main/main_account.cpp` gives each account its own durable storage/session lifecycle, saves session state before destruction, and restarts network state on proxy changes.
- `Telegram/SourceFiles/main/main_domain.cpp` stores multiple accounts, persists/chooses an active account, exposes ordered accounts and active-session changes, and cleanly finishes all accounts.
- `Telegram/SourceFiles/api/api_authorizations.cpp` models current/other device authorizations, refreshes authoritative sessions, terminates one/all sessions and carries authorization TTL metadata.

### Current Fabushi candidates inspected

- Electron account authorization/binding: `source/electron-main/account/account-authorization.ts`, `production-account-authorization.ts`, `cursor-auth-wiring.ts`.
- Fabushi Human identity: `source/electron-main/account/human-identity.ts`.
- Host Session durable state: `source/host/src/extensions/session/*`.
- Coordinator account transition/revoke wiring: `source/electron-main/coordinator/account-transition-cleanup.ts`, `account-revoke.ts`, `coordinator-account-runtime.ts`.
- Shared Room member and Agent session owners were considered but rejected as canonical account/auth owners.

### Selected existing owner

`existing_owner = existing Electron account/auth owner + existing Host Session durable owner`, with `human-identity.ts` as the current local Human identity seed. No Telegram account/session model is introduced.

The current Human identity store is account-scoped, durable, atomically replaced and fails closed on corrupt state. It is adequate as a local identity precursor, but it is not yet a complete multi-device identity/auth/session protocol.

### Absorption / model / persistence / native-network requirements

- Stable Fabushi account and Human identity IDs must be independent of Telegram peer/account IDs.
- Account switch must fence renderer/account-scoped state, native sync cursors, outgoing queues and presence publication by account generation.
- Device/session credentials, revocation and multi-device list belong below existing account/auth boundaries; revocation must terminate future send/sync authorization without deleting canonical local history by accident.
- Multi-account storage must remain partitioned; cross-account nonce/message/reference reuse fails closed.

### Focused tests / blockers

Corrupt identity state; stable identity across restart; two-account isolation; switch while send/sync is pending; device revoke; revoke-current-device; stale credential/session; logout/re-login; account-generation fencing. Native Fabushi device/session service and multi-device authorization are not implemented.

## CONTACT-ID-PRESENCE — contacts, participant identity, online/typing state

### Frozen upstream evidence

- `Telegram/SourceFiles/data/data_lastseen_status.h` separates exact online-until from privacy-hidden `recently/within-week/within-month/long-ago` states and serializes the presence abstraction independently of message history.
- `Telegram/SourceFiles/data/data_peer_values.cpp` derives visible presence text and next refresh timing from status, while also applying participant/peer send restrictions.
- Frozen peer/user/search/member areas show that identity/profile, membership and short-lived presence are related but distinct state families.

### Current Fabushi candidates inspected

- Local Human identity: `source/electron-main/account/human-identity.ts`.
- Shared Room/member projection: `frontend/src/production/group-members-root.ts`, `frontend/src/recovered/features/agent-info/group-members/*`, Host group/session owners.
- Host Agent roster: `source/host/src/extensions/session/session_roster.rs`.
- PR #20 c276 outline roster projection: `source/host/src/extensions/transcript/roster_projection.rs` + `roster_emit.rs`.
- Transcript, renderer-only presence, and a new broad CommunicationCore were considered and rejected as canonical contact/presence owners.

### Selected existing owner

`existing_owner = existing Human identity/account + Shared Room/member model for durable participant identity/contact projection; minimal ephemeral presence infrastructure below conversation/room projection where required`.

There is no dedicated current canonical Fabushi presence owner in the inspected exact HEAD. The c276 `RosterProjection` specifically owns Agent outline stream identity/coalescing and must not be misused as Human contact/presence truth.

### Absorption / model / persistence / native-network requirements

- Extend current participant/member identity semantics to durable Human profile/contact references; do not create `TelegramUser` or another identity database.
- Presence/typing is ephemeral, sequence/timestamped, expiring and privacy-aware; it does not become transcript history.
- Reconnect drops stale presence and republishes only current local state; backpressure/throttle is required.
- Durable contact/profile changes use canonical Fabushi identity IDs and account scope; presence can be rebuilt after restart.

### Focused tests / blockers

Contact/profile update convergence; account isolation; hidden/unknown presence; expiry; typing cancel; stale/out-of-order presence; reconnect stale-drop; room member removed while presence inflight; privacy policy. Contact service and ephemeral native presence channel remain unimplemented.

## DIALOG-FOLDER-ARCHIVE — conversation list, pinning, folders, archive

### Frozen upstream evidence

- `Telegram/SourceFiles/dialogs/dialogs_main_list.cpp` maintains per-filter ordered/pinned lists, loaded/cloud sizes and unread aggregates rather than treating the sidebar as a static view.
- `Telegram/SourceFiles/data/data_chat_filters.cpp` represents filter inclusion/exclusion, peer classes, pinned peers and archive/read/mute policies over canonical histories.
- `Telegram/SourceFiles/data/data_folder.cpp` treats archive as a folder/list with unread and recent-history projection; archive is not a second conversation/message store.

### Current Fabushi candidates inspected

- Existing sidebar: `frontend/src/recovered/features/conversation/workspace/sidebar.tsx`.
- Current pin ordering: `frontend/src/production/sidebar-model.ts`.
- Section projection/persistence: `frontend/src/recovered/features/conversation/workspace/sidebar-section-projection.ts`, `sidebar-sections-state.ts`.
- Existing conversation workspace/Session transcript owners were considered for list data, but rejected as a separate sidebar-state owner; they supply canonical conversations, not list organization policy.

### Selected existing owner

`existing_owner = existing sidebar / conversation-list owner`.

Current sidebar sections are still Agent-shaped (`agentIds`, synthetic `__agents__`) but already have account-scoped durable section state, Host hydration, retry and stale-generation fencing. FBCP should generalize this owner to conversation IDs/kinds and folder/archive/pin policy rather than add a Human or Telegram sidebar.

### Absorption / model / persistence / native-network requirements

- One sidebar row model must project Human, Agent, Group, Channel and Topic conversations.
- Pin/folder/archive is metadata over canonical conversation identity; moving/archiving never duplicates transcript truth.
- Persist account-scoped ordering/filter metadata with migration from current Agent-only section schema.
- Native sync must version/reconcile remote list metadata and unread/read state without letting stale remote order overwrite newer local changes.
- Archive/folder/filter policy should preserve bounded list loading and unread aggregates for large conversation sets.

### Focused tests / blockers

Agent-only schema migration; Human+Agent mixed ordering; duplicate IDs; pin reorder; archive/unarchive; folder inclusion/exclusion; unread aggregation; account switch; stale remote list update; restart; large-list bounded projection; accessibility/keyboard. Current sidebar schema remains Agent-specific and remote list sync does not exist.

## Owner-resolution conclusion

All four domains have a selected existing product owner at PR #20 `c2767eac...`; none requires a broad new product architecture. The missing native identity/session, presence and list-sync responsibilities are infrastructure beneath those owners and remain blockers, not permission to create a second Conversation/Message/Identity truth.
