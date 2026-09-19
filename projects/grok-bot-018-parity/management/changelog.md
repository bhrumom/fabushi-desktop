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

- Added production `sand-media://` streaming with registered attachment Range reads and packaged media helpers.
- Added recovered clock-skew/video-container runner contracts and tests.
- Added authenticated experiments provider/cache/override lifecycle and account token refresh; kept external Statsig parity explicitly partial.
- Added per-agent runner owner and routed coordinator turns through it.
- Run 35360378372 passed on audited code head 910f65e42b0dd096e9c969eba03666f46ae764c9.
- Refreshed machine parity counts without claiming A8 completion.


## 2026-09-19
- Cut production Agent orchestration over to the pinned reference SandAgentRunner + AnysphereAgent; removed the hand-written legacy model/tool loop from production.
- Enforced byte-exact pinned renderer/source parity and official renderer coordinator/desktop bridge closure.
- Closed local Mac Computer/Browser action breadth, default executable Marketplace/private-skill/MCP paths, exact TrayManager, Sand cross-user sharing relay and real local Teach screen recording.
- Raised final runtime contract coverage to 103/103 PASS on shipped SHA `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`.
- Final automated Mac delivery: source Run `35417698506` SUCCESS; macOS Run `35417698507` SUCCESS; artifact `10576418103` digest `sha256:6d3bf39199c312e695a6ab59cb7eb7cf408df6ab2ff34dee1a7076eb91406b2c`.
- Published prerelease `grok-parity-mac-92` with `2.0.0-alpha.2` arm64 DMG/ZIP; release tag verified to exact shipped SHA.
- Transitioned to GBR-007 external human installation/interaction acceptance; PR #1 intentionally remains open and unmerged.

- Corrected the final headless packaged-smoke wording: Run `35417698507` published successfully, but the runner did not emit the UI probe report and recorded `manualValidationRequired:true`; packaged UI acceptance remains GBR-007 human work.
- Synchronized GBR-003/004/005 to current automated runtime/source evidence and created GBR-007 as the durable manual acceptance task.
- Superseded the alpha.2 human-test candidate with reproducible `2.0.0-alpha.3` at immutable product SHA `528ddc8e31c320ca472191cf29a860c2086dabc2`.
- Committed and bound the desktop dependency lock for reproducible `npm ci` source/package verification.
- Final alpha.3 source gates: push Run `35418603612` and PR Run `35418606285` SUCCESS; runtime coverage is **105/105 PASS, 0 fail**.
- Final alpha.3 macOS delivery Run `35418603624` SUCCESS; artifact `10576404427` digest `sha256:7899c1e47537c4793de59118d9328085038505069e438b255c62b0c0421768b2`.
- Published `grok-parity-mac-97` with `2.0.0-alpha.3` arm64 DMG/ZIP and verified the release tag resolves exactly to the product SHA.
- The bounded GitHub-hosted renderer probe still produced no UI report and recorded `manualValidationRequired:true`; human installed-product acceptance remains GBR-007 and PR #1 stays open.
