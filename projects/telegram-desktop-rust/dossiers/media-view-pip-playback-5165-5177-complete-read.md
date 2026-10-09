# Media view PiP and playback 5165-5177 complete read

Status: exact-source read complete; mapped-open product responsibilities; no unknown/verification promotion
Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`

## Exact source
Orders 5165-5177 were fetched directly from accepted upstream in deterministic order. The range covers `media_view_pip.{cpp,h}`, GL/raster/RHI PiP renderer backends and abstraction, `media_view_playback_controls.{cpp,h}`, and `media_view_playback_progress.{cpp,h}`. The attestation manifest binds every path to its accepted blob and byte size.

## Product responsibilities
The PiP controller persists/clamps geometry, shares rather than duplicates the streaming document, transfers ownership on enlarge/close, preserves seek pause/resume state, volume, quality, speed, power-save blocking and deterministic teardown. GL/raster/RHI files are platform renderer implementations and are replaced by Chromium/Electron compositor primitives rather than copied.

Playback controls expose play/pause, seek, downloaded/available progress, volume, speed, quality, fullscreen, PiP, timestamp/chapter markers, fade and drag/input ownership. PlaybackProgress separates authoritative current position from buffered/available progress and loading transitions.

## Existing-owner audit
Fabushi's canonical `MediaViewer` now exposes Chromium native controls, pauses outgoing video on replacement/unmount, protects native media input from gallery zoom/pan/navigation and lets browser fullscreen consume Escape. That is real partial implementation, not closure of the full range.

Still open: durable volume/playback-position/speed policy, explicit quality selection semantics, deterministic PiP geometry/stream handoff, buffered/loading projection beyond browser-native rendering, Media Session integration and packaged cross-platform evidence.

## Accounting
After this complete read: recursive 16,120; read-through 5,177; unread 10,943; unknown 15,841; omitted 0. Unknown remains unchanged because these responsibilities are not yet fully implemented and verified. `baseline_ready=false`; `acceptance.accepted=false`.
