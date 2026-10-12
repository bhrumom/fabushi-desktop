# Media view rendering 5151-5160 complete read

Accepted upstream: `telegramdesktop/tdesktop@42f8a36d43b8c805bc821905bea4cfeb3af1d41d`.

This batch directly reads recursive orders 5151-5160 and closes source-reading/decomposition only. It does not claim implementation or verification.

- 5151 `media_view_group_thumbs.h`: grouped thumbnail cache, animation, activation and repaint lifecycle contract.
- 5152-5153 `media_view_metal_texture.{h,mm}`: macOS CoreVideo/Metal NV12 texture-cache bridge, device rebinding, Y/UV plane sizing and flush/release lifetime.
- 5154-5155 `media_view_open_common.{cpp,h}`: typed media-open contexts, streaming resume/start options, video timestamp extraction, and quote/preformatted entity normalization.
- 5156-5157 `media_view_overlay_opengl.{cpp,h}`: GPU media overlay rendering, shader/texture/cache lifecycle, YUV/NV12/static/story surfaces, controls/fades/rounded geometry.
- 5158-5159 `media_view_overlay_raster.{cpp,h}`: software fallback renderer, clipping/rotation/story crop/transparency and painter-state ownership.
- 5160 `media_view_overlay_renderer.h`: backend-independent media overlay renderer contract shared by GPU/software implementations.

Canonical disposition: reuse current Fabushi MediaViewer/player/platform rendering owners. No Telegram-specific product root or component is introduced. All rows remain mapped-open until current shipping owners are audited, gaps are implemented, and focused exact-head production evidence is green.
