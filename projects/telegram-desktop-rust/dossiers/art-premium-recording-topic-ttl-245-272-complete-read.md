# TDRP Revision 9 complete read: art responsibilities 245-272

Authority: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

All 28 deterministic blobs were read in full. Binary art was fetched from the accepted commit and matched by exact Git blob SHA; SVG/OBJ sources were parsed as complete text rather than inferred from filenames.

## Premium 3D build/runtime chain (246-256)

`Telegram/cmake/generate_models.cmake@b15053479d35fc51b91733c5af6046efae3ef7c1` is included by `Telegram/CMakeLists.txt@7ffda8f085b8917889f5c1b06edbef160d975c5b`. It GLOB_RECURSEs every `Resources/art/**/*.obj`, runs `Telegram/cmake/obj2binobj.py@c6c676c2f209cd880a21aa6ea7d96f294270c676`, generates a build-tree `models.qrc`, and adds `bake_models` as a Telegram dependency. This proves that the tracked OBJ files are human-readable build sources, not runtime lookups.

Runtime consumers:
- `premium_coin_renderer.cpp@38fffe9b207eae148d6806f3c46616fa722456b4` loads generated coin_outer/inner/logo/stars `.binobj`, coin_border.png and flecks.png into QRhi resources.
- `premium_diamond_renderer.cpp@fe0c74c1b8adb9f2d0961cd5a78a93bc37a97146` loads generated diamond_outer_2/diamond_outer/diamond `.binobj`.
- `premium_star_model.cpp@499d751aa9aff7726ef010dafa65750c1c03b24f` loads generated `star.binobj`; `premium_star_renderer.cpp@f1a04861db07ea34a3439eee71332fd9a04d6f02` uses the star border texture and shared flecks texture.
- `premium_3d_mesh.cpp@b8541b310714f1449d7fd2544e562c16453a00cf` validates compact-mesh counts/indices before producing vertices. Coin/Diamond renderers fail closed at the visual-effect layer when model/pipeline creation fails.

The source OBJ files were fully read. Their exact geometry counts were also checked (for example star.obj: 3,889 vertices, 3,889 UVs, 3,896 normals, 6,575 faces), proving this is real model-source acquisition rather than filename credit.

## Recording / video / theme preview (257-261)

`calls_group_recording_box.cpp@04fc207d6e4353099f66c246d16cc34beaeec324` constructs `RecordingInfo` graphics from `:/gui/recording/%1.svg` and maps the UI to AudioOnly, VideoLandscape and VideoPortrait recording types. The three SVGs are embedded by `telegram.qrc@6c966a19f3625c1c78628b8d21820623412ba2e1`.

`round_video_recorder.cpp` uses `round_placeholder.jpg` only as the circular recorder frame fallback when no caller-provided placeholder exists.

`window_theme_preview.cpp@df6755bfa0515414fef2e091f4a4622aade42a79` uses `themeimage.jpg` as a fixed sample photo bubble in the theme preview; it is not theme persistence/state truth.

## Forum topic identity (262-269)

`data_forum_topic.cpp` owns the mapping from canonical topic color IDs to blue/yellow/violet/green/rose/red SVG names, gray as fallback, and the special general icon. It dynamically renders the QRC SVG at device-pixel ratio and may overlay a title letter. This is topic identity presentation attached to the real ForumTopic lifecycle, not an independent topic state store.

## TTL media (270)

`ui/controls/ttl_media.cpp@45703cc78b9bfa23b8500afbe1baaefd8503de38` renders the SVG into TTL countdown/fire and tooltip presentation from the existing destroyAt/ttl state; `history_view_gif.cpp` also uses it for ephemeral video media. The asset does not own expiry/deletion truth.

## Conservative consumer-open assets (245, 271-272)

Exact current-tree code/build/QRC searches found no consumer for `mac_setup.tiff`, `verified_bg.webp`, or `verified_fg.webp`. They remain read-complete but consumer/provenance-review-open. They are not marked omitted or not-applicable based on names alone.

## Rights/provenance

Repository `LICENSE@d70f4d6a87eece19c4ec184de36a8aba72b4a639` and `LEGAL@09c9e383db122fe0694e73c48d6224676afe60f6` were considered. No per-file trademark/art derivation grant was established for this art batch, so direct Telegram artwork reuse is not treated as a Fabushi target without separate rights provenance.

## Accounting

The deterministic complete-read prefix advances from 244 to 272. Unread becomes 15,848; unknown remains 16,033; omitted remains 0. No source-read row is promoted to production-complete merely by understanding the upstream resource.
