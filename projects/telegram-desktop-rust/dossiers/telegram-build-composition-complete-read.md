# Telegram shipping build composition — complete source read

Accepted upstream: `telegramdesktop/tdesktop@aac515c5408015231a273c80ab4c4b33815e63ab`  
Source: `Telegram/CMakeLists.txt@23997fd6503cd46389c2c13803a83b45b40c21f7`  
Read status: complete, 2,672 lines.

This manifest is a shipping build owner, not boilerplate. It composes the Telegram executable across API/data/dialog/history/media/settings/storage/platform/UI modules, resources and generated code; wires update-key verification/tests; generates Windows MIDL; selects platform integrations; configures macOS resources, entitlements and Crashpad; generates Linux portal/notification D-Bus bindings; links platform dependencies; constructs Updater/Packer/StartupTask; and installs Linux desktop/AppStream/icon/service integration.

Fabushi mapping is responsibility/effect based, not file-list copying: canonical Electron/npm + Rust Host/Coordinator composition replaces the C++/Qt CMake topology. This entry is therefore mapped and unknown-closed, but it does **not** grant implementation credit to any production source merely because that path appears in the source list. Each `Telegram/**` source/resource still requires its own exact-blob read, responsibility decomposition and shipping/test evidence.

Known platform artifact gaps remain independently open: cross-platform signed release/update channel (54), Linux full build qualification (60), Snap/Linux distribution (66), Windows multi-arch full build qualification (71), and WinGet publication (72).

Accounting:
- full-read: `148`
- unread: `15,664`
- unknown: `15,730`
- omitted: `0`
- read/decomposed through deterministic entry: `88`
