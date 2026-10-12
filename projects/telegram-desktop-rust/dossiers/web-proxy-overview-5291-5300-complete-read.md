# Web-proxy carrier and shared-media overview 5291-5300 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`)

Recursive orders 5291-5300 were read directly and completely from their exact upstream blobs. Credit is limited to source read-through, responsibility decomposition and an existing-owner audit. `unknown` intentionally remains unchanged and `omitted=0`.

## Responsibilities

- **5291 — `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_transport.cpp` @ `e25ad970e84686e9aca061d02c274048793b5ad4`**: bounded generation-fenced browser/WebView relay transport with loopback capability authentication, strict HTTP/WebSocket/origin validation, multiplexed stream flow control, backpressure/fair flushing, write-progress watchdogs, carrier retry/failover and deterministic teardown.
- **5292 — `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_transport.h` @ `f58850d2bc2ceabd93113d7725903e1c0d92d71b`**: typed relay lifecycle and stream API contract: Idle/WaitingForBrowser/Connecting/Connected/Failed state publication, stream registration/data/window/close callbacks and global bounded pending accounting.
- **5293 — `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_webview.cpp` @ `8a2efc8fb15e400366f199cd956effc8ad362da8`**: restricted hidden-WebView carrier with exact HTTPS navigation/source and nonce binding, versioned bridge handshake/welcome, health/write timeouts, bounded single-inflight sequenced delivery, ACK validation and generation-bound failure.
- **5294 — `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_webview.h` @ `57d8a5d6e91e275759774ad7a32961ce14406fd9`**: typed hidden-WebView carrier ownership contract for ready/payload/written/failed callbacks, pending/in-flight frames, handshake/health/probe/write timers and close/failure lifecycle.
- **5295 — `Telegram/SourceFiles/overview/overview.style` @ `d51f1de1d1cd811c6c5d3679ec9bec73fdfccd0f`**: shared-media overview semantic visual states for photo/video/file/voice/song/link rows, selection controls, play/pause/download/cancel feedback, status badges and story pinned state.
- **5296 — `Telegram/SourceFiles/overview/overview_checkbox.cpp` @ `9722aa21fed56c788bceed284f7438e9074c51ab`**: shared-media selection control interaction: active/pressed/selected/selecting state projection, pressed feedback animation, deterministic finish and cache invalidation.
- **5297 — `Telegram/SourceFiles/overview/overview_checkbox.h` @ `2da919e232b94811d6991bb8c641536b9c589f6c`**: typed shared-media checkbox state/animation contract with update callback and active/pressed/checked lifecycle.
- **5298 — `Telegram/SourceFiles/overview/overview_layout.cpp` @ `ceae87bc8148b76426a3e1e9d427f99f04659429`**: shared-media item lifecycle for photo/video/voice/document/link/GIF: media load/unload ownership, bounded thumbnail/crop work, async generation fencing, sensitive/spoiler reveal, story pinned/hidden state, download/upload/play progress, link safety, GIF visibility/power-saving and selection/click behavior.
- **5299 — `Telegram/SourceFiles/overview/overview_layout.h` @ `c6aba8cac455f3e01805ab835fc6968d5b8c142c`**: typed shared-media layout/item contracts for selection, radial progress/status, media options, heavy-resource lifecycle, external loading/cancel and photo/video/voice/document/link/GIF item state.
- **5300 — `Telegram/SourceFiles/overview/overview_layout_delegate.h` @ `0d016974b102bedf989f9a65a10a48f0587bed62`**: shared-media layout delegate boundary for heavy-item register/unregister, visibility/repaint, media retention policy and typed photo/document open actions.

## Existing-owner audit

The WebProxy files do **not** justify a second MTProto/WebProxy runtime. Their applicable product invariants are capability/authentication expiry, exact source/origin binding, bounded pending work, stream/request flow control, write/handshake health, retry/failover, generation fencing and deterministic cleanup. Those belong to the existing Fabushi Coordinator/Host/backend transport and desktop/browser/plugin/MCP bridge/security boundaries. The Telegram relay wire protocol, local browser page and source-specific WebView bridge are platform/protocol details and are not copied.

The overview files likewise do **not** justify Telegram/Overview UI owners. Current canonical owners already include `native/mahayana-messaging/src/search.rs` (Media/Files/Links scopes), `native/mahayana-messaging/src/media_cache.rs` (bounded physical cache/eviction), `source/shared/rpc/coordinator.ts` (`searchMedia` entrypoint), and `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx` (media open/playback/navigation/resource projection). The canonical design-system `Checkbox` / `ResultRow` / collection semantics must be reused for selection; new source-derived UI must not expand legacy `sand-*` naming.

Order 5298 adds important open behavior beyond drawing: heavy-resource register/unregister and visibility ownership, async thumbnail generation fencing, bounded image crop strategy for extreme/huge inputs, spoiler/sensitive reveal, story pinned/hidden state, download/upload/play status, safe-link dispatch, GIF visibility/power-saving and exact click-vs-selection behavior. These remain mapped-open until canonical collection/resource owners implement and prove the applicable subset.

## Evidence and closure status

Predecessor PR head `a5c76249c4f79f9d3355b8c78f0a99e9ce6763bf` passed Rust desktop runtime run `37917685269`, including Source authority / recursive inventory job `113777882511` with artifact `11610602313` / `sha256:32c47f1a0a30d0d19c1986f21bb2e3afea4a7728e9062ee463979699c04c46e2`, and Desktop Chat Parity CI run `37917685329` with Playwright artifact `11609977618` / `sha256:0f52772bcffe306e6b266b63b632e65a2df13dda6fd3d042253a8ce9c05b19a4`.

Those runs prove the predecessor head only. The commit that records orders 5291-5300 creates a new PR head, so all current-head gates/evidence must run again. No responsibility in this batch is promoted to implemented/verified from read evidence alone.
