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
- Full line-by-line equivalence to all reference modules is NOT claimed; remaining parity gaps are tracked under GBR-003/004/005.

## 2026-09-18 — Round 3
- Re-read the exact reference commit 107877b4e2134fd167d239411386f09e42eadd6d and confirmed it is identical to the reference repository's current main.
- Added a human-readable parity inventory plus a reference-derived machine inventory covering all 275 reachable renderer modules, 70 audited runner modules, and 16 Electron production bindings.
- Added the previously missing GBR-003 and GBR-005 task records.
- Corrected the stale package state: Run 35323112810 succeeded on 5c3480a15003c145d26313f990e3f433cf8d7fdc; artifact 10538065159 has digest sha256:884a992b9cfa2cf16891ddee05939a5daa52bc2343f8ac92e6fa0c30d9d4adae.
- Removed automatic PR packaging. `.github/workflows/macos-grok-parity.yml` now packages only on explicit workflow dispatch or canonical-main push, so parity implementation commits cannot be mistaken for final package evidence.
- Split runtime ownership into `grok-agent-coordinator.cjs` → `grok-host-runtime.cjs` → `local-tool-executor.cjs`; `grok-agent-runtime.cjs` is only a composition facade.
- Added persisted always/ask/never local-tool permission, one-shot approval/deny/cancel lifecycle, waiting/running/error/cancelled transcript states, Stop cancellation, cancellable foreground shell, background shell process lifecycle, and local macOS screenshot/click/type/key execution.
- Removed visible GitHub/Memory placeholder catalog rows. Current built-in Plugin entries map to executable Files/Terminal/Browser/Computer providers.
- Added a real stdio MCP provider with initialize, tools/list, tools/call, server enable/remove, and per-tool enable/disable. OAuth/accounts/private skills/workflows are still open.
- Added functional command palette/shortcuts, agent rename/delete dialogs, backed permission Settings, approval cards, stop control, loading/error states and MCP configuration UI.
- Added non-packaging `Grok parity source check`. Initial generated-newline failure was fixed; Run 35333859439 then passed CJS syntax, dependency install and TypeScript/Vite build on exact source head 5e447597d4081e8df8dc5d3b5abea59175f7c232.
- A8 remains NOT COMPLETE. No merge, exact-head macOS packaging, canonical-main reread or prerelease was performed.

## 2026-09-18 — Round 4
- Audited code head: `910f65e42b0dd096e9c969eba03666f46ae764c9`.
- Recovered and production-wired the reference media path: `sand-media://` privileged scheme registration, attachment chunk reads, HTTP Range semantics, video MIME mapping and package inclusion.
- Recovered exact clock-skew and video-container runner contracts and added contract tests.
- Added an authenticated experiments provider seam with cached snapshots, environment gates, dynamic configs, persisted development overrides, account token refresh and renderer IPC. Exact reference Statsig/bootstrap semantics remain PARTIAL rather than being fabricated.
- Added a per-agent runner owner with run generation, active-run lifecycle, interrupt/quiesce, external cancellation and actual coordinator-to-host dispatch.
- Run `35360378372` passed CommonJS syntax, all runtime contract tests, dependency installation and TypeScript/Vite build on the audited code head.
- Refreshed the generated parity ledger: runner = PASS 10 / PARTIAL 58 / FAIL 0 / PRODUCT_DIFFERENCE 2; Electron bindings = PASS 9 / PARTIAL 7 / FAIL 0. Renderer remains PASS 17 / PARTIAL 196 / FAIL 61 / PRODUCT_DIFFERENCE 1.
- A8 remains NOT COMPLETE. No final macOS packaging, merge or prerelease was performed.


## 2026-09-19 — Round 5 — exact pinned source and reference Agent production cutover
- Pinned reference remains `bhrum/grok-bot-0.18-reconstructed@107877b4e2134fd167d239411386f09e42eadd6d`.
- Exact renderer parity is now enforced by CI: all 308 `frontend/src` blobs map byte-for-byte to `desktop/src` (0 missing / 0 different).
- Full reference source is vendored under `reference/grok-bot-0.18/source`: 1,724 / 1,724 source blobs are byte-identical to the pinned reference.
- The production renderer is the reference `ProductionRenderer`; legacy Fabushi Contacts/Telegram/payment/MiniApp/Mahayana renderer source is not mounted in this Mac parity branch.
- Reference `SandAgentRunner` is built from the pinned source and owns per-agent run/interrupt/quiesce lifecycle.
- Reference `AnysphereAgent` is now the production agent orchestration runtime. The prior hand-written bounded tool loop was removed from production; `FABUSHI_AGENT_ENGINE=legacy` no longer re-enables it.
- Existing local Files/Terminal/Browser/Computer/MCP/Subagent executors are injected through the reference Agent tool boundary, preserving the sole product difference: Computer targets the Mac where Fabushi is installed rather than a provisioned cloud Box.
- Reference conversation state is persisted per agent; runtime tests cover plain turns, tool-call continuation, Browser/MCP/Subagent routing, private provider delta isolation, reference-engine selection and state persistence.
- Fabushi OAuth account tokens are wired as the production inference bearer credential when `FABUSHI_ACCOUNT_INFERENCE_URL` is configured; API-key environment configuration remains only a provider/development fallback.
- The source gate performs exact pinned-directory diffs, runtime contracts, pinned reference source typecheck and renderer production build. The last pre-account-cutover green gate on `97d0bba6fbf65feb59196812a331c6e0126abe1b` passed 66/66 tests; later heads add further reference-runtime/event tests and are being revalidated after a test-fixture syntax repair.
- Mac prerelease `grok-parity-mac-16` was published from exact SHA `b3b04a23245345b7d3491eb5859b7ccbbffd8562`; subsequent packages are superseded by the continuing Agent cutover. The workflow now cancels superseded branch packages so only the latest exact-head candidate remains.
- The reference repository's own Host activation audit depends on immutable carrier artifact `src/app/dist/host/host-main.cjs`, which is not stored in the Git repository. That carrier-only audit is recorded as EXTERNAL_DEPENDENCY and is not substituted with fabricated evidence.
- Final product acceptance remains human/external: packaged UI interactions and model-provider deployment must be validated on the released Mac build.


## 2026-09-19 — Round 6 — final frozen macOS candidate published
- Final shipped candidate code SHA: `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`.
- Exact source gate Run `35417698506` completed SUCCESS on that SHA: exact pinned renderer/source diff PASS, reference runtime bundles PASS, CommonJS syntax PASS, **103/103 runtime contracts PASS**, pinned reference source typecheck PASS, TypeScript/Vite production build PASS.
- macOS package Run `35417698507` completed SUCCESS on the same SHA.
- Artifact `10576418103` (`fabushi-grok-parity-macos`) size `393022970` bytes, digest `sha256:6d3bf39199c312e695a6ab59cb7eb7cf408df6ab2ff34dee1a7076eb91406b2c`.
- Prerelease `grok-parity-mac-92` published with `fabushi-grok-parity-2.0.0-alpha.2-macos-arm64.dmg` and `.zip`.
- Release tag `grok-parity-mac-92` resolves exactly to `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`; branch-SHA and tag-SHA fail-closed checks both passed in Run `35417698507`.
- Automated source/runtime/package scope is complete. Per the original request, PR #1 remains open for external human installation/UX/function acceptance rather than being merged ahead of that test.


## 2026-09-19 — Round 6 — final automated delivery
- Final shipped code SHA is `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`.
- Exact pinned source gate Run `35417698506` completed SUCCESS with 103/103 tests PASS, 0 fail, reference source typecheck PASS and renderer production build PASS.
- PR source gate Run `35417701558` also completed SUCCESS on the same SHA.
- macOS delivery Run `35417698507` completed SUCCESS on the same SHA for package, artifact upload, branch-SHA verification, prerelease creation and published-tag SHA verification. Its deliberately non-blocking headless renderer probe produced no smoke report and recorded `manualValidationRequired:true`; no automated UI-product PASS is claimed.
- Artifact `10576418103` is `393022970` bytes with digest `sha256:6d3bf39199c312e695a6ab59cb7eb7cf408df6ab2ff34dee1a7076eb91406b2c`.
- Prerelease `grok-parity-mac-92` published DMG and ZIP for version `2.0.0-alpha.2`; the lightweight tag resolves exactly to `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`.
- Automated implementation/delivery scope is complete. PR #1 remains open and unmerged because the original requirement delegates final product testing to humans.
- Next stage is GBR-007: install the released DMG and manually exercise the Grok-style UI, reference Agent behavior, local Computer/Browser actions, Marketplace/plugins, Teach recording, sharing, approvals and error/cancellation flows. Any discovered defect must be fixed on the same PR and re-released through the exact-SHA gates.


## 2026-09-19 — Round 7 — evidence reconciliation
- Reconciled final package evidence against Run `35417698507` logs.
- The package/release job itself is SUCCESS and exact-SHA tag verification passed, but the headless renderer probe did not produce its UI report; it recorded `manualValidationRequired:true`.
- Therefore no automated packaged-UI PASS is claimed. This corrects the earlier Round 6 wording that described the packaged smoke as successful.
- GBR-003/004/005 task records were synchronized to the current automated PASS state, and GBR-007 was created as the explicit external human acceptance task.
- Shipped code remains `ccc4e29f75e7057eb6cc3562ab78584ffbd55cdd`; this reconciliation changes project records only.


## 2026-09-19 — Round 8 — alpha.3 exact-head human-test candidate
- Current immutable product candidate SHA: `528ddc8e31c320ca472191cf29a860c2086dabc2`.
- Push source gate Run `35418603612` completed SUCCESS: exact pinned renderer/source diff PASS, reference runtime bundles PASS, CommonJS syntax PASS, **105/105 runtime contracts PASS**, pinned reference `source:typecheck` PASS, TypeScript/Vite production build PASS.
- PR source gate Run `35418606285` also completed SUCCESS on the same product SHA.
- macOS package/release Run `35418603624` completed SUCCESS on the same SHA: dependency install, production build, unsigned DMG/ZIP packaging, artifact upload, release-branch SHA check, prerelease creation and published tag SHA verification all PASS.
- Artifact `10576404427` (`fabushi-grok-parity-macos`) is 393016719 bytes with digest `sha256:7899c1e47537c4793de59118d9328085038505069e438b255c62b0c0421768b2`.
- Prerelease `grok-parity-mac-97` publishes `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.dmg` (196953272 bytes) and `fabushi-grok-parity-2.0.0-alpha.3-macos-arm64.zip` (197394188 bytes).
- Tag `grok-parity-mac-97` resolves exactly to `528ddc8e31c320ca472191cf29a860c2086dabc2`.
- The GitHub-hosted headless renderer probe still produced no UI report and recorded `manualValidationRequired:true`; therefore packaged UI/function PASS is intentionally not claimed.
- This supersedes `grok-parity-mac-92` as the canonical GBR-007 human-test candidate. PR #1 remains open and unmerged for the human acceptance round required by the original request.
