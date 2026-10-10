# Exact-source dossier: orders 5074-5084 — media player controls, float, canonical state and listen telemetry

- Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`
- Root tree: `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`
- Accounting after batch: read-through 5,084; unread 11,036; unknown 15,841; unknown-closed 279; omitted 0.
- Read method: complete direct exact-blob semantic reading of all eleven sources.

## 5074-5078 — canonical player controls and menus

The style source defines one coherent player visual system for playback, repeat/order, speed, volume, next/previous, close, progress, panel/list and floating preview. Fabushi must express these through its existing design-system primitives, including hit areas, active/disabled/hover states, light/dark contrast and responsive geometry; the source style/type names are not public-component candidates.

PlayButton morphs play/pause/cancel with reversible in-flight animation rather than snapping stale intermediate state. Speed and Settings buttons project non-default speed and quality badges and animate hover/active state. Dropdown lifetime is explicitly fenced: hover crossing, macOS deactivate, temporary hide/show and adjacent-dropdown pointer movement cannot leave a stale menu owner.

Speed supports a continuous 0.5–2.5 range in 0.1 steps, snap points, realtime change events, one-second debounce and final-save events. Order toggles default/reverse/shuffle. Quality exposes automatic/original/manual heights and keeps a live active check without allowing selection of the already chosen item.

Canonical owners: **Media player Toolbar/IconButton controls** and **Menu/Popover settings**.

## 5079-5080 — floating round-video player

The float binds to the exact round-video HistoryItem and detaches on item removal or account/session change. It paints streamed circular frames plus playback progress, toggles pause/resume on click, handles double-click through the owning section, and begins drag only after the platform drag threshold.

Visibility is the conjunction of stream readiness, source-message visibility and drag state. Dragging outside the parent fades opacity; release either docks to the nearest enumerated canonical column/corner or closes when more than half outside. Column/corner are persisted. Delegate replacement reparents existing widgets, wheel input is routed to the exact section, and destruction removes the item before destroying its QWidget to avoid reentrant history repaint observing a half-destroyed entry. Closing stops the voice track and emits the exact message id.

Canonical owner: **Media floating preview/player**, subordinate to the existing ProductShell/Conversation sections.

## 5081-5082 — player canonical state/playlist owner

One Instance owns separate Song and Voice domains and one current streamed player per domain. Context is not just a message id: it carries history migration, topic root, monoforum sublist, scheduled/saved-music domain and rich-page multiple tracks.

Playlist slices are requested around the current id, invalidated when the requested key moves too far, and complemented by the opposite boundary only for repeat-all. Track movement first handles multiple tracks inside one message, then playlist entries. TTL media is excluded from playlist jumps.

Shuffle has explicit played/non-played history, a 10,000-item load limit, only 16 remembered old-order items, migrated-id normalization, deleted-message removal and restart when externally selecting a track outside the remaining shuffled set. Scheduled and Saved Music use their proper media viewers rather than pretending all playlists are normal history.

Playback opens shared streaming data, uses Voice/Video `Both` mode for video messages, applies source-specific speed, restores saved long-media position once, marks voice/video media read, and emits exact start/stop/track/switch/seeking events. Seeking pauses, computes a known duration from current stream info or document fallback, restarts streaming at the clamped position, and can preserve paused state during drag.

Calls pause active voice/song playback and only resume tracks explicitly paused by the call. Repeat-one restarts the same track; autoplay-next may be disabled. Streaming NotStreamable/OpenFailed degrade into the canonical save-to-file paths. Long playback positions are saved only for videos >=1 minute or music >=20 minutes and cleared on stopped/end states. Active audio prevents app suspension; active video also prevents display sleep. Round-video playback toggles the global GIF pause reason only while actually playing. Teardown finalizes telemetry and clears blockers/streaming state.

Canonical owner: **Media playback state/playlist**. This is a core state machine, not UI-only parity.

## 5083-5084 — music listen telemetry

The tracker accumulates wall-clock listened time only while actually playing. A pause starts a 60-second report timer; resume cancels it; stop/switch/finalize settles the current segment. Durations below three seconds are not sent. Report is message-context scoped; on `FILE_REFERENCE_*` 400 it refreshes the exact origin and retries only if the document's file reference changed.

Canonical owner: **Media playback telemetry/service**.

## Acceptance

Orders 5074-5084 are exact-tree/path/blob bound and read-complete only. Production behavior, canonical UI implementation/a11y, service wiring, integration/E2E, artifact provenance and release acceptance remain open. Current exact-head Source authority must attest this shard/manifest.
