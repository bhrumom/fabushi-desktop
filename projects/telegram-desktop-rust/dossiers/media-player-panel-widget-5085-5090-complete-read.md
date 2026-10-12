# Exact-source dossier: orders 5085-5090 — player playlist panel, volume and responsive top bar

Accepted upstream: `telegramdesktop/tdesktop@3a15bf1fe34b6950916215a11b50eadfce4bbb41`, tree `b031c2cd84aa0cbbb149a7e5693c41ef14d643f3`. Complete direct exact-blob reads. Accounting after batch: read-through 5,090; unread 11,030; unknown 15,841; unknown-closed 279; omitted 0.

## 5085-5086 — playlist panel

The panel is lazy and binds to the exact active Song context. It refuses another account, distinguishes Saved Music, topic root, monoforum sublist and migrated peer, destroys the old list when context identity changes, restores around the current id with a 32-id bound, and keeps scroll/visible-range state attached to that list. Hover cancels hiding; normal leave delays 300 ms; list-requested hide can wait three seconds; `preventAutoHide` can fence hiding. Hiding destroys the list and refresh lifetime rather than leaving a hidden playlist owner.

Canonical owner: **Media player playlist Popover/DetailPanel** backed by the existing canonical media/playlist state.

## 5087-5088 — volume

The slider supports horizontal/vertical geometry and routed wheel input. Live movement updates mixer and setting; finished movement remembers only nonzero volume and persists settings. External volume changes update the slider only while the user is not actively changing it, avoiding feedback races. Mute therefore preserves the last audible value.

Canonical owner: **Media player volume/settings control**.

## 5089-5090 — responsive accessible player toolbar

This widget projects the canonical player state. Voice/video-message playback takes UI priority while voice remains active; after voice finishes an already-active Song becomes the projected type. Closing Voice while Song remains active stops Voice only instead of closing the whole player.

Controls are play/pause/cancel, previous/next, volume, repeat, order, speed, close, progress, title and time. Accessible names are maintained for speed, volume, repeat, order, close, play/pause, previous and next. Loading projects Cancel, percentage and disabled seeking. Song seeking previews bounded time then commits through the canonical player; round-video seek is intentionally not exposed by this UI.

Mute restores remembered nonzero volume. Repeat cycles None/One/All; order and speed persist. Narrow layouts fade right controls unless hovered. Volume/order/speed dropdowns mutually dismiss, and shell animation can hide then restore shadow, slider and dropdowns. Song label hover requests the playlist; label click navigates to the exact item only for Voice or a Song owned by another session. Previous/next buttons exist only when navigation is available and expose independent disabled visuals/cursors plus accessible names.

Canonical owner: **Media player Toolbar** using source-neutral canonical components.

## Acceptance

All six entries are exact tree/path/blob bound and `read_complete=true` only. Production implementation, a11y/responsive verification, service wiring, integration/E2E, artifacts and release acceptance remain open. Current exact-head Source authority must attest this shard and manifest.
