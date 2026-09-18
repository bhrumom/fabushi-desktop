# Changelog

## 2026-09-18
- Created project GBR and recorded reference/target SHAs.
- Accepted ADR-001: TypeScript agent architecture with installed-computer adaptation.
- Replaced layered legacy desktop production entry with one Grok-style agent shell.
- Added context-isolated local agent runtime and persistent transcripts.
- Added plugin browse/search/install/enable baseline.
- Removed build dependency on missing monorepo frontend/Mahayana roots.
- Added macOS GitHub Actions package/prerelease workflow.
- Added `management/parity-inventory.md` and reference-derived `management/parity-inventory.generated.json`.
- Added GBR-003 and GBR-005 task records.
- Gated macOS packaging so PR implementation commits no longer auto-package before A8.
- Refactored runtime ownership to coordinator → host → local execution.
- Added persisted local-tool permission and one-shot approval/deny/cancel UI/runtime lifecycle.
- Added cancellable foreground shell, background shell process control, and local macOS screenshot/click/type/key tools.
- Removed unimplemented GitHub/Memory plugin placeholders.
- Added executable stdio MCP server provider and server/tool configuration UI.
- Added command palette/shortcuts, agent rename/delete, Stop, approval cards, backed Settings and loading/error states.
- Added non-packaging parity source CI; Run 35333859439 passed CJS syntax and TypeScript/Vite build after fixing the first detected syntax issue.
- Corrected A5/GBR-001/GBR-006 package evidence to Run 35323112810 and artifact digest sha256:884a992b9cfa2cf16891ddee05939a5daa52bc2343f8ac92e6fa0c30d9d4adae.
