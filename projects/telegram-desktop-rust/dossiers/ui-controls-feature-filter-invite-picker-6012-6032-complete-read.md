# UI controls feature/filter/invite/picker 6012-6032 complete read

Status: exact-source read complete; mapped-open; no global unknown reduction.
Accepted upstream: `65e23ba7137ea4129b6bc1b2616104a1f59495ef`; tree `6b616494f3465324e749a04dcd1c9d508657a998`.

Orders 6,012-6,032 cover feature/detail rich rows; filter/share preview, badge and CTA state; invite copy/share/reactivate/delete and joined-count projection; invite link ContextMenu activation; jump-to-latest unread projection; labeled emoji Tabs with drag-threshold scrolling and active reveal; location Picker WebView/geolocation/address/venue search-cache-cancellation state; participant LoadingState skeletons; and screen-clamped Popover lifecycle.

All UI maps to source-neutral canonical `ProfileSection`/`ListRow`/`DetailPanel`, `Dialog`, `Button`, `Badge`, `Tabs`, `ContextMenu`, `IconButton`, `Picker`, `SearchField`, `ResultRow`, `LoadingState`, `ErrorState` and `Popover`. Location remains a service boundary: precise-location permission/tokens, scoped WebView storage, request cancellation, privacy and teardown remain explicit requirements under existing Host/WebView owners.

The descendant Human-call production slice also advances media command ownership: camera and screen-share mutations share one source-neutral `video-media` fence, project canonical pending state, roll back local sender/track state on authoritative failure, restore camera when sharing stops, and stop the owned screen capture on failure/decline/hangup/unmount. ForceMuted/RaisedHand/scheduled/audio-reactive call presentation remains mapped-open.

Accounting: **6,032 / 16,125** read; **10,093** unread; **15,846** unknown; **0** omitted. First unread: **6,033** `Telegram/SourceFiles/ui/controls/round_video_recorder.cpp@b52b73bd0ea351ee1ba3b76b5ed9834af2f4451c`. Reading alone grants no implementation, verification, baseline or release credit.
