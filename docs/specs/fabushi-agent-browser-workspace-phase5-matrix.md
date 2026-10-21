# Fabushi Agent Browser Workspace — Phase 5 Capability Matrix

This matrix is an evidence ledger, not a feature promise. Statuses use the four required values:
`implemented-and-tested`, `partially-implemented`, `blocked-with-evidence`, and `not-applicable-with-rationale`.
A capability is not upgraded to `implemented-and-tested` until the shipping code path, focused negative tests, and exact-HEAD GitHub Actions evidence all agree. The dialog/event slice is newly added and its exact-HEAD CI is pending at authoring time.

## Evidence rules

- Product authority: the native Electron `WebContentsView` Browser Workspace in `source/electron-main/browser-workspace.ts`; Main owns tab identity, session, permissions, download paths, dialogs, CDP command allowlist, event capture, and the final workspace/control/generation checks.
- Agent route: `source/shared/local-browser-agent.ts` -> `source/host/src/runner/tools/local_browser_tools.rs` -> `CoordinatorToolRelay` -> Main. `target=local` never silently falls back to Box.
- Box Browser remains a separate Playwright/Host path in `source/host/src/runner/tools/sand_browser_tools.rs` and `source/host/src/runner/tools/sand_browser_driver_source.rs`.
- Tests present in source are not proof of execution. Attach run ID, tested `head_sha`, job ID, real test counts, artifact ID/name/digest, and downloadable ZIP SHA-256 before promoting a row.
- Latest code parent at matrix drafting: `9f78a665435085ee622b6280a5dc1f34af9b6af9`. Dialog/event code and this matrix require a new exact-HEAD Actions run; no run is claimed here.

## Capability matrix

| Capability | Status | Shipping path and current evidence | Gap / promotion gate |
|---|---|---|---|
| Browser TaskSpace / Page API | partially-implemented | Conversation-owned runtime workspace and WebContents tab registry: `browser-workspace.ts`; action identity: `local-browser-agent.ts`; contract suite: `browser-workspace.contract.test.ts`. | Prove current exact-HEAD Windows/Linux packaged executable and signed macOS candidate; separate task-space/profile semantics remain narrower than a standalone Page API. |
| Multiple simultaneous Bot tasks | partially-implemented | Main tracks per-workspace task state, bounded concurrency, task IDs and control epochs; real Electron E2E in `desktop/e2e/mahayana-agent-workbench.spec.ts`. | Require terminal exact-HEAD Chat Parity plus packaged Windows/Linux run with two simultaneous production Host tasks and takeover negative assertion. |
| Structured snapshot | partially-implemented | `captureLocalBrowserSnapshot`, bounded elements/frames/text and generation-bound refs; focused Main contract tests. | Exact-HEAD Electron E2E must confirm actual page elements and no stale-ref side effects. |
| Deep iframe discovery | partially-implemented | `local-browser-agent-page.ts` captures bounded frames and resolves frame-scoped locators. | Add/retain real Chromium nested-frame cases and report pass/fail/skip counts on final candidate. |
| Element references and lifecycle | partially-implemented | Opaque generation refs in `RuntimeTab.agentRefs`; navigation/crash increments generation and clears refs. | Exact-HEAD Host and real Chromium stale-generation negative tests must pass. |
| Navigation and waits | partially-implemented | Main navigation policy is HTTP(S) plus loopback; bounded text wait and generation checks. | Current candidate must prove redirects, failures, cancellation, and bounded event waits in real Chromium. |
| Mouse, keyboard, form, scroll and drag | partially-implemented | Fixed action union and Host schema; Main validates target, refs and bounds. | Packaged executable and Host relay tests pending on final candidate. |
| Screenshots | partially-implemented | Main captures the actual Chromium page with bounded screenshot size. | Verify packaged evidence does not contain private page content and the screenshot comes from the real executable. |
| Download and trusted upload | partially-implemented | Main-owned download paths, 64 MiB cap, SHA-256; upload uses trusted OS picker and bounded bytes. | Verify download bytes/hash in exact-HEAD Chat Parity and both Windows/Linux packaged runs; retain no local path in Agent result. |
| JavaScript dialogs | partially-implemented | New `dialog` action uses only Main-owned `Page.handleJavaScriptDialog`; accept/dismiss and prompt text are schema-bounded and generation/control-epoch fenced. | Focused negative tests and exact-HEAD Electron CI must pass; verify alert/confirm/prompt and takeover behavior on real Chromium. |
| Page / load / network-idle events | partially-implemented | New bounded 64-event ring and 32-event read batch; page/load/network events carry sequence, timestamp and origin only; network-idle is a 500 ms quiet window with a 15 s max wait. | Exact-HEAD tests must prove cursor behavior, timeout, bounded buffer, no URL query/header/body leakage, and packaged network observation. |
| Errors and recovery | partially-implemented | Load failures, crash/destroy state, bounded descriptions and explicit action errors. | Verify crash/restart and failed-navigation behavior in the installed candidate. |
| Cancellation and user takeover | partially-implemented | Main revokes task/control epochs and serializes workspace mutations; stale task/generation operations are rejected. | Exact-HEAD concurrent Bot and takeover tests must pass; prove queued operations have no post-takeover side effects. |
| Reusable experience / workflow memory | not-applicable-with-rationale | The current Browser Workspace has no approved durable cross-task workflow-memory product or source API. | Do not infer this capability from history/bookmarks; a separate product and privacy decision would be required. |
| Browser extension lifecycle | partially-implemented | Electron session extension APIs support inspected unpacked MV3 install/enable/update/disable/remove/restore with integrity and permission checks. | Compatibility is intentionally narrower than Chrome; exact packaged lifecycle test and explicit unsupported API matrix required. |
| Login state and profile strategy | partially-implemented | Account-scoped persistent Electron session partition; account revocation destroys runtime tabs. | Verify persistence and account-switch fencing from packaged Windows/Linux and final macOS candidate; shared login state is not shared Agent authority. |
| Privacy and security isolation | partially-implemented | Remote pages have no Node integration; trusted IPC sender checks; default-deny permissions; Main-only fixed CDP methods; event logs omit request URL, headers and bodies. | Require exact-HEAD negative tests for forged sender, arbitrary CDP, stale lease, extension privilege, and event/dialog leakage. |
| Windows NSIS packaging | partially-implemented | `.github/workflows/browser-workspace-packaged-ci.yml` packages NSIS and binds the installed executable path. | Current run `39531306353` is still running on earlier exact HEAD; require new exact-HEAD terminal success, test counts, SHA-256 manifest and artifact digest. |
| Linux AppImage/deb packaging | partially-implemented | Same workflow packages AppImage and deb, installs the deb and binds the installed executable. | Current run `39531306353` is still running on earlier exact HEAD; require new exact-HEAD terminal success, test counts, SHA-256 manifest and artifact digest. |
| macOS signing, notarization and installed candidate | partially-implemented | `.github/workflows/release-macos-main.yml` accepts `source_sha`, signs/notarizes, staples, installs the exact candidate, runs Playwright acceptance and uploads SHA-256 manifests. | Do not dispatch until the final merge-candidate SHA and its prerequisite gates are known; release must remain non-publishing (`publish_release=false`) for acceptance. |

## Exact-HEAD acceptance record

- Dialog/event implementation SHA: pending atomic commit.
- Desktop Chat Parity run/job: pending for the dialog/event SHA.
- Browser Workspace CI run/job: pending for the dialog/event SHA.
- Rust desktop runtime run/job: pending for the dialog/event SHA.
- Packaged Windows/Linux run/jobs and artifact manifests: pending for the dialog/event SHA.
- macOS release run/jobs, artifact digests, and downloaded ZIP SHA-256: pending for the final merge-candidate SHA.

Do not change this matrix to a passing status on the basis of source inspection alone. Record terminal Actions evidence after each exact-HEAD change, and keep PR #165 Draft until the final candidate satisfies all required gates.
