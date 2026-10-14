# TDRP Revision 9 complete read: WebView shell, themes and shortcuts 273-278

Authority: `telegramdesktop/tdesktop@22b352e866d0402505c07fa4ed21d75d7e4fb3db` / tree `94ae09469c816b350f60dc9ada1ff049323be8e7`.

All six deterministic resources were read from the accepted commit.

## 273-275 — external Bot WebView shell

`body.html@36ec0adb20c740f65231679e8790b5408106bd7f` defines the shell chrome (back/menu/close controls, title/badge/loading, frame host, footer buttons, pointer shield, menu and resize handles). `page.css@ae74bcc2ab192157e706d78b19891334359920a5` carries the shell's layout/theme/responsive states. `page.js@b7100fb30f28d38175d9987fbac12bd7ea9e58e6` is a real bridge state machine rather than decorative JS.

Shipping path:
- `Resources/qrc/telegram/bot_webview_shell.qrc` embeds the three files and `cmake/td_ui.cmake` includes them in td_ui source ownership.
- `attach_bot_webview_linux_shell.cpp@7dcc697c5830834997a9cb8f524150536cd19af4` injects a per-session shell token into page.js and wraps execution in a shell-origin check.
- `attach_bot_webview.cpp@5d26af74e6e462528b7aac1d72e113738d4b18e4` defines a 64 MiB native-message ceiling and rejects invalid JSON, wrong external sender/type/source, bad or empty shell token, invalid shell/webapp origins and shell-only commands originating from the webapp.
- External shell identity has explicit token generation, generation increment, bootstrap, invalidation and error teardown. page.js separately fences iframe pending events by frame generation, limits pending events to 64, validates message source/origin, forwards viewport/menu/button/chrome state, and ignores untrusted shell-control events where required.

Fabushi responsibility is therefore a session-bound Mini App host/security lifecycle plus canonical shell UI, not a reusable Telegram HTML root. It must converge on Fabushi's existing MCP App Surface/WebMCP bridge and canonical UI rather than creating a second bridge owner.

## 276-277 — embedded Day themes

Both files are QRC embedded and consumed by `window_themes_embedded.cpp@3874c727e77b771bbca48d22d11d7819fae79abe`. Day Blue is a selectable EmbeddedScheme; day-custom-base is the canonical light base file used when deriving accent-colored themes. System accent and stored per-theme accent selection feed the same Colorizer/Theme owner.

## 278 — default custom-shortcut seed

The full JSON template is intentionally empty except for documented examples. `core/shortcuts.cpp@67f248f66b79be8eeb90f0542afcad8f39b87e1c` copies it to `tdata/shortcuts-custom.json` when needed, then the single Shortcuts Manager parses/validates bindings, maps commands to QActions, handles media/support enablement, platform ShortcutOverride behavior and persistence.

## Accounting

Read-through advances to 278; unread becomes 15,842. unknown remains 16,033 and omitted remains 0 because source understanding does not imply Fabushi production implementation or verification.
