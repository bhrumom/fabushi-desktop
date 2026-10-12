# Hime Qt input test, distro lifecycle, and output transforms — orders 7,743–7,768

Accepted authority: `telegramdesktop/tdesktop@6fed91ffab9861771a75f29df65031b3c80b6941` → Hime `9b3e6f9ab59d1fe4d9de73d3bf0fed7789f921c5`.

Orders 7,743–7,747 are a deliberately minimal Qt5 line-edit harness. Qt widgets are not migration targets; the applicable responsibility is a focused standard-editable-field compatibility test for the platform input path.

7,748 makes release archives from ChangeLog version authority, rejects conflicting outputs, strips VCS metadata, normalizes archive ownership, and creates a DFSG variant excluding the separately licensed Dayi input table. 7,749 creates an isolated input-method test environment: locale, module/table paths and unique XIM identity are explicit; GTK module registries are regenerated; a non-executable output filter is rejected; generated registries, socket state and shell environment are cleaned after the run.

7,750–7,757 cover Fedora/FreeBSD/Ubuntu distribution behavior: deterministic version/source generation, declared dependencies, install/upgrade/uninstall hooks, GTK input-module cache refresh, alternatives/config registration, idempotent Unity systray registration and cleanup. Distro/Hime branding and commands are replacement-only; lifecycle symmetry and fail-closed registration belong to canonical Build/Release + platform lifecycle/settings owners.

7,758 is GNU FDL 1.2 provenance. 7,759 installs/uninstalls the declared output-filter set symmetrically.

7,760–7,768 are optional single-input/single-output transforms. They preserve unmapped text, apply ordered character mappings or explicit representability conversions, and must not introduce a second draft/message truth. The random Hime-branded image notification in `nobopomofo` is replacement-only; any user feedback must use deterministic canonical Fabushi feedback/accessibility behavior.

All 26 rows are read and responsibility-decomposed but remain mapped-open. Reading closes no unknown.

Next unread: **7,769** `Telegram/ThirdParty/hime::git-hooks/pre-commit`.
