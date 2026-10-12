# Media-editor toggle animation resources — complete source read

Accepted upstream: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

## `photo_editor/trim_shortest.tgs`

The complete gzip/Lottie payload was decoded: 240×240, 60 fps, frames 0..18, one `trim_cut Outlines` layer and no external assets. It is packaged as `/animations/photo_editor_trim_shortest.tgs`.

Its shipping consumer is `TrimShortestButton` in `editor/photo_editor_controls.cpp`, not an orphan decoration. The control is an icon button with a localized `Trim to shortest` tooltip. Inactive/active are the first/last animation frames; normal state changes animate forward or reverse and instant changes jump directly. `PhotoEditor::trimShortestRequests()` toggles the editor content's `durationsLinked` truth and refreshes the video/audio timelines. Availability follows the video/audio timeline/editor state.

Fabushi already resolves Telegram's media editor responsibility to canonical `MEDIA-EDITOR -> Composer + attachments/resources + media viewer`, with non-destructive derived-resource state. No shipping Fabushi trim-shortest/duration-link surface was found, so this entry is read/decomposed but remains an implementation, visual, a11y and focused-test gap. The equivalent UI must reuse canonical `IconButton` and `Tooltip`; it must not introduce a Telegram-specific component or copy the asset merely for parity.

## `sun_outline.tgs`

The complete gzip/Lottie payload was decoded: 100×100, 60 fps, frames 0..35, four top-level layers, one embedded asset. It is packaged as `/animations/sun_outline.tgs`.

The exact shipping consumer is the media-editor link-preview `ThemeButton` in `editor/editor_link_box.cpp`. `false` (light) maps to frame 0; `true` (dark) maps to the last frame; `setDark()` animates from the current frame to the target. The button emits `themeToggles`, is shown for message-source previews, and is hidden for loading/pill/cleared states. The preview renderer and background theme are switched consistently with that state.

This is an applicable media-editor preview-state responsibility, not a generic global theme setting. Fabushi's canonical owner is the same MEDIA-EDITOR capability surface and global light/dark truth; the UI must reuse canonical `IconButton`. No shipping equivalent was found, so production, visual, keyboard/screen-reader and context-preservation evidence remain open.
