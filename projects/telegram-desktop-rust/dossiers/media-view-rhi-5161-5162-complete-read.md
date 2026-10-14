# Media view RHI 5161-5162 complete read

Status: exact-source read complete; partial production implementation; verification remains open
Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted root tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`
Fabushi base during audit: `6ae1fc21348b4ecd3aa67d4b8196eaabd0e2361e`

## Exact files

- 5161 `Telegram/SourceFiles/media/view/media_view_overlay_rhi.cpp` — blob `fe61046cbc094e7a2cafd7eeba09c42a75284ddf`, 1,895 lines, 54,432 bytes.
- 5162 `Telegram/SourceFiles/media/view/media_view_overlay_rhi.h` — blob `059d22ff5f5bb9b013b79c91c56c9c825062bae1`, 230 lines, 6,349 bytes.

Both files were read directly from the accepted upstream blobs. This advances only deterministic read-through. It does not close unknown responsibility by itself.

## Responsibility decomposition

The pair owns the Qt QRhi implementation boundary for MediaViewer overlay rendering: renderer/device initialization, shader pipelines, bounded vertex/uniform slots, static RGBA content, YUV420/NV12 video planes, macOS native-texture zero-copy binding, story cropping, recognition/control overlays, DPR/theme/story cache invalidation, ordered offscreen/onscreen video-stream paint and deterministic resource release.

The product responsibility is not to copy those Qt/QRhi classes into Fabushi. Fabushi already has one shipping MediaViewer and one Electron/Chromium renderer. Chromium therefore replaces the low-level shader/texture/device layer. Fabushi must still own the application-visible contract around it: media is actually playable, media controls keep input ownership, stale media does not retain playback after source replacement/unmount, full-screen Escape is not mistaken for close, and source changes do not create a second renderer/runtime owner.

## Existing-owner audit

The canonical owner is `frontend/src/recovered/features/conversation/workspace/media-viewer.tsx` composed through the existing ConversationWorkspace. Before this change its video branch rendered a Chromium `<video>` with `controls={false}`, while the viewer's global left/right navigation and zoom/pan handlers could still take input intended for the media element.

No Telegram-specific MediaViewer, GPU renderer, player store, or second UI/runtime owner is introduced.

## Production change in this cycle

The canonical MediaViewer now:
- enables Chromium native video controls and inline playback on the existing video element;
- keeps a single `videoRef` and pauses outgoing video before source replacement and on viewer unmount;
- does not apply image zoom/pan pointer or wheel ownership to videos;
- does not consume left/right gallery shortcuts when focus is inside video/audio/control/slider/editable surfaces;
- lets an active browser fullscreen session consume Escape before the application closes the media dialog.

Focused contract `CONTRACT-TDRP-MEDIAVIEW-NATIVE-PLAYBACK-001` binds these behaviors to the canonical owner. GitHub Actions on the new exact HEAD are still required before verification promotion.

## Still open

This pair does not close recognition UX, story-specific overlays, group-call borrowed rendering, playback speed/quality policy, loading/available progress, sponsored-video semantics, full PiP geometry persistence, system media controls or later media-view source files. Those remain mapped/open and are handled in deterministic source order beginning with 5163.

Accounting after this read: recursive 16,120; read-through 5,162; unread 10,958; unknown 15,841; omitted 0; `baseline_ready=false`; `acceptance.accepted=false`.
