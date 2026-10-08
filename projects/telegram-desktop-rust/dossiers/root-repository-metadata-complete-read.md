# Root repository metadata — complete source read

Accepted upstream: `telegramdesktop/tdesktop@6ec5014a92c4580841a043c440d65e8fc605ff78`

Entries 73–87 are fully read and exact-blob dispositioned in `projects/telegram-desktop-rust/inventory/source-dispositions.json`: `.gitignore`, `.gitmodules`, complete `.grok/**`, `AGENTS.md`, `CLAUDE.md`, root `CMakeLists.txt`, `GROK.md`, `LEGAL`, `LICENSE`, `README.md`, and `REVIEW.md`.

Important decomposition rule: engineering guides mention real product rules (serialization compatibility, API callback lifetime, UI scaling, platform branching, localization/reactive semantics), but these documentation entries do not receive shipping-production credit. Those responsibilities remain to be proven against their actual `Telegram/**` source owners. Likewise `.gitmodules` only declares topology; recursive inventory already counts the gitlink contents themselves.

The root CMake file is a real build-composition artifact but its C++/Qt topology is replaced by Fabushi's canonical Electron/npm + Rust/Cargo composition. Existing unresolved cross-platform artifact/distribution gaps remain explicitly open at workflow entries 54, 60, 66, 71 and 72 rather than being hidden here.

Accounting after entry 87:
- full-read: `147`
- unread: `15,665`
- unknown: `15,731`
- omitted: `0`

The next deterministic source entry is under the recursively expanded `Telegram/**` tree.
