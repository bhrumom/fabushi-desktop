# Status log

## 2026-09-18 — Round 1
- Target main confirmed at 12d4aadb4a93a1413ece44aba987162e90b31229.
- Reference main confirmed at 107877b4e2134fd167d239411386f09e42eadd6d.
- Target had 121 files; reference has 2,111 files.
- Target production entry layered Messenger V2, Grok parity patches, Mahayana workbench, credential vault, and MiniApp bridges.

## 2026-09-18 — Round 2
- Project and ADR persisted in commit edfdfdc6aa48e2825068cf75a428896338434b47.
- Core refactor persisted in commit 1bce9f070ce789c082e47c1edb07462588d0dbc3.
- Replaced production entry with one Grok-style agent application.
- Removed production mounting of contacts/groups, Messenger V2, MiniApps/Telegram/payment, Mahayana workbench, credential vault, and prior parity patch layers.
- Replaced monorepo/Cargo build dependency with a self-contained Electron + React + TypeScript desktop path.
- Added context-isolated agent IPC, persistent agent/transcript state, create/delete/rename/send/stop flows, and local-computer semantics.
- Added a functional persisted plugin browser/install/enable baseline.
- Added macOS Actions packaging for DMG/ZIP and automatic prerelease publishing when the change lands on main.
- Full line-by-line equivalence to all 2,111 reference files is NOT claimed; remaining parity gaps are tracked under GBR-003/004/005.
