# Exact-source dossier: orders 5034-5037 — MainWidget / MainWindow

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Exact blobs: `mainwidget.cpp@619d6ad925a31424c65dd4d83c1d118376d76f4b`, `mainwidget.h@086a6992cb1cd6b0deb6e145b839b383c7538a6e`, `mainwindow.cpp@4c2279e4c9fa033d7845e93c6291b75b8da1be16`, `mainwindow.h@8f628c4797690ce5e88e61add25d27406110cc16`.
- Read method: complete direct exact-blob semantic reading. These files are mixed responsibility composition owners; decomposition below is behavioral, not a mechanical file port.
- Accounting after batch: read-through 5,037; unread 11,083; unknown 15,841; unknown-closed 279; omitted 0.

## MainWidget responsibilities

1. **Conversation shell and navigation ownership**
   - Composes dialogs/history/main/third sections under one SessionController.
   - Preserves forward/back/clear-stack semantics, duplicate suppression, section mementos, reply-return state, remove/destroy requests, saved-chat restoration and cross-window routing.
   - Routes topics, saved sublists, scheduled/pinned/admin-log sections through existing section identities rather than a second message store.

2. **Send/share/drop permissions and draft transitions**
   - Forward drafts validate send restrictions before navigation.
   - Share URL writes a local draft with exact cursor/reply/thread identity and clears conflicting edit draft.
   - File/drop routing enforces file restrictions, resolves forum/monoforum targets, and only invokes the currently active canonical section.
   - Bot command and single-use keyboard actions delegate to the active section/history owner.

3. **Search routing**
   - Resolves hashtag+username links separately from tags/query search.
   - Selects dialogs/global search versus embedded in-chat search according to window/layout/scope and forwards to the correct window when the canonical chat lives elsewhere.
   - Does not own a second search index.

4. **Auxiliary media/call/export lifecycle**
   - One float-player delegate selects visible sections and fences GIF pause/visibility.
   - Audio player, playlist, call/group-call top bar and export-progress top bar are created/destroyed from authoritative service state and feed geometry offsets into the same shell.
   - Late/delayed destroy callbacks are guarded by current owner state.

5. **Responsive layout, focus and persistence**
   - One/two/three-column geometry derives from SessionController adaptive layout and persisted width ratios.
   - Resize handles persist only settled widths; third-column info/tabbed-selector swaps follow current chat writeability.
   - Focus arbitration follows current visible section and avoids hidden-widget focus theft; Back first retires media/layers/search before navigation.
   - Animation capture hides derived floating overlays while snapshotting and restores them afterward.

6. **Background/resource and session lifecycle**
   - Wallpaper document/media load is asynchronous and guarded against stale generation; ready/default state settles into the canonical theme background.
   - Construction subscribes to message/draft/call/export/player/layout changes and teardown saves the current cloud draft.

**Fabushi mapping:** extend current canonical ProductShell, ConversationWorkspace/navigation, Search, Composer/send-permission, Media/Call and Theme owners. No MainWidget/Telegram runtime or duplicate Conversation/Message/Search store is permitted.

## MainWindow responsibilities

1. **Root window/work-mode lifecycle**
   - Applies initial maximize/minimize/tray policy, tracks position/state, routes close to hide-or-quit based on authenticated domain/window state, and preserves startup-file security gates.

2. **Privacy locks and root composition**
   - Passcode and setup-email lock surfaces replace/hide intro/main content, close session attach-web-view/wallet surfaces where required, and restore the prior root surface through bounded animation.
   - Intro and Main content are mutually composed through one root owner.

3. **Overlay/layer and focus arbitration**
   - One LayerStack owns boxes/special layers/main menu, toggles GIF-pause reason, restores focus on destruction, and is blocked while privacy locks are active.
   - Root focus precedence is theme warning -> layer -> passcode -> setup-email -> main -> intro.

4. **Media preview/theme/read-state lifecycle**
   - Media preview is lazily created, raised with root overlays and hidden on mouse release.
   - Theme test warning is delayed until palette subscribers settle.
   - Mark-as-read is allowed only when main content is visible/not animating, no layer covers it, the OS window is exposed/not minimized, and activity/idle policy permits.

5. **Platform event and geometry bridge**
   - Event filter forwards activity/window-state/position changes; geometry accounts for session filter width and all lock/layer/preview surfaces.

**Fabushi mapping:** current Electron/ProductShell root, Auth/Lock, Overlay/Dialog/Toast, Media preview and platform adapter owners. Qt-specific event/widget mechanisms are replaced, while privacy, focus, read-state, close/tray and lifecycle semantics remain mandatory.

## Acceptance

All four entries are exact-tree/path/blob bound and `read_complete=true`. They remain `mapped-open-*`: no implementation, UI, service, test or release status is promoted by this dossier. Current exact-head Source authority must attest this shard; later production work must close each decomposed responsibility against existing source-neutral Fabushi owners.
