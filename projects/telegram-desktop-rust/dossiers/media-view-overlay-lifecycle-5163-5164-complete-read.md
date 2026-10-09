# Media view overlay lifecycle 5163-5164 complete read

Status: exact-source read complete; mapped-open responsibility; no unknown or verification promotion
Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted root tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`

## Exact files
- 5163 `Telegram/SourceFiles/media/view/media_view_overlay_widget.cpp` — blob `03138e6eeb429ca38a1b8edafa0a9a7d9742cb1a`, 9,064 lines, 254,132 bytes.
- 5164 `Telegram/SourceFiles/media/view/media_view_overlay_widget.h` — blob `40d67d4c05ba47710d8cdd3d6f8e844ffa1a05e3`, 912 lines, 27,968 bytes.

The cpp was read directly from the accepted blob in contiguous ranges 1-900, 901-1800, 1801-3600, 3601-5400, 5401-7200 and 7201-9064. The header was read in ranges 1-480 and 481-912.

## Responsibility decomposition
The pair is the media-view product controller rather than a renderer-only detail. It owns typed source context, shared-media/user-photo/collage navigation, static and streamed content, player creation/update/error handling, seek/frame-step/speed/quality/fullscreen/PiP controls, system media controls, call interruption, music coexistence, power-save blocking, save/share/delete/copy/recognition actions, keyboard/touch/wheel/native gestures, TTL/single-view burn, screenshot protection and deterministic hide/session/resource cleanup.

Specific failure-sensitive contracts observed include:
- save eligible playback position before streaming teardown;
- only resume after a call when this viewer itself paused the stream;
- quality changes preserve current frame/time and reinitialize without duplicating player ownership;
- stream errors distinguish non-streamable/open-failed and update the canonical media failure state;
- native window recreation reapplies screenshot protection;
- close/hide burns eligible single-view media and cancels TTL/speed/recognition/transient state;
- controls and viewer shortcuts arbitrate input rather than both owning the same gesture/key;
- PiP continuation shares the stream and transfers ownership instead of creating a second authoritative player.

## Existing-owner-first disposition
Fabushi already has the canonical MediaViewer under ConversationWorkspace and the Electron/Chromium media platform. This cycle therefore does not create an OverlayWidget equivalent. The native playback change made the existing video element usable and fenced replacement/unmount/input ownership, but it does not satisfy the whole controller contract above.

Persistence, call interruption, speed/quality policy, system media controls, PiP geometry, TTL/single-view, screenshot protection and recognition remain mapped-open and require implementation through existing Fabushi owners plus exact-head tests.

## Accounting
After the exact-source read, deterministic read-through is 5,164/16,120; unread is 10,956; unknown remains 15,841; omitted remains 0. `baseline_ready=false` and `acceptance.accepted=false`.
