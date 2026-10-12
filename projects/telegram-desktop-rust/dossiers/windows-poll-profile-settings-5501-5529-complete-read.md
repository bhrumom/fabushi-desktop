# Windows / poll / profile / settings responsibilities 5501-5529 — complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`
Accepted tree: `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`
Range: deterministic orders 5501-5529 (29 exact blobs).

## Findings

- 5501-5514 finish the Windows platform tail: Windows Hello provider registration is delayed until the main queue exists; WebAuthn preserves exact RP/user/challenge/credential result data and cancellation while routing native platform authenticators vs cable/libfido2; AppUserModelID and shortcut validation bind shell/toast identity to the actual executable file; StartupTask distinguishes policy/user states; safe dynamic symbol resolution bounds optional native APIs; Quiet Hours informs notification policy; taskbar media controls derive from the single player state and delay theme-driven updates for five seconds to avoid Explorer shared-handle races; COM toast activation is parsed then forwarded to the main queue.
- 5515-5520 define poll option-link editing and poll media. Poll media accepts one photo/video/document, prepares metadata/thumbnails, token-fences every asynchronous preparation/upload callback, reports progress/failure, cancels prior active uploads on clear, and never lets stale completions replace a newer choice. No existing Fabushi poll-media owner was found; this is a production gap, not an omission.
- 5521-5527 are profile presentation primitives: design tokens, responsive/elided header/back affordance, state-saveable profile blocks and animated cover drop area. These must reuse ProfileSection/Toolbar/IconButton/ListRow/resource-drop primitives rather than create source-named UI.
- 5528-5529 expose legacy global settings responsibilities including launch/update/tray/file-dialog path state, secure working-directory permission setup, passcode retry throttling, display scale validation and recent-input state. Fabushi must keep those semantics under canonical typed settings/persistence/security owners rather than global mutable duplicates.

## Closure

All rows are read/decomposed, `unknown_closed=false`, `omitted=false`. Source reading alone does not grant production/test/release credit. The poll domain gap, Windows WebAuthn/toast/taskbar equivalence and settings/security semantics require same-head evidence before closure.
