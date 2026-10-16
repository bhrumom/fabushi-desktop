# Desktop isolated Mini App window

Status: implementation in progress; packaged acceptance pending. Date: 2026-10-13.
Owner: existing Plugins/MCP native adapter and Host package verifier/storage owners.
Parent: `desktop-catalog-package-installation.md`, OMA-001 §§7–10.

## Context and goal

The Host package verifier/ledger owns trusted installed package state. This unit
adds the Electron surface that renders only a Rust-Host-reverified immutable
launch snapshot. It does not own install state, authentication, a new app shell,
or a second database. The caller supplies a stable hashed `accountScope` derived
from the confirmed native Fabushi user ID and a monotonically increasing
account `generation`; neither comes from a renderer or access token.

## Requirements

- **DMW-01 Scheme setup:** export `FABUSHI_MINI_APP_PRIVILEGED_SCHEME` for the
  one global `protocol.registerSchemesAsPrivileged(...)` call before
  `app.whenReady()`. The repository already registers `sand-media` in
  `createElectronProductionComposition` calls `services.registerMediaScheme`
  before `mainstart`/Electron readiness. The existing MediaProtocol owner must
  append this declaration to its sole `registerSandMediaScheme()` registration;
  do not add another global registration. WebRTC IP handling is configured on
  each Mini App's `webContents`, never by a process-wide Chromium switch. After
  ready, bind a handler to each exact session obtained from its partition.
- **DMW-02 Verified input:** `open` accepts immutable Host output containing
  app/version/digest/entrypoint, trusted opaque account scope/generation and a
  bounded map of already verified bytes/MIME. It takes no path, URL, key,
  account identity, or package bytes from Web content. Reject malformed IDs,
  digests, resource paths, MIME, size, stale generation, or unavailable entry.
- **DMW-03 Isolation:** one stable origin and persistent session partition per
  account scope + app ID, distinct across apps/accounts. Version updates share
  only that app's user data and never replace a running snapshot. No preload,
  Node, WebView tag, native IPC, MCP bridge, or external windows.
- **DMW-04 Offline boundary:** serve only manifest-listed immutable bytes.
  Deny every non-package request, redirect, external navigation, popup,
  permission, service worker, worker, WebSocket, WebRTC egress, and native
  bridge. The sole download exception is a same-app `blob:` JSON report from
  `org.fabushi.global-dandelion`, with Electron-confirmed user gesture, exact
  `global-dandelion-local-report.json` filename, one active download, and a 1 MiB
  hard cap. Electron's normal save flow chooses the path. CSP permits only
  same-origin resources and scoped Wasm via `wasm-unsafe-eval`; it denies
  cross-origin connect, worker-src and objects. Resource matching compares the
  exact custom scheme and host, not Node's opaque `URL.origin` representation.
- **DMW-05 Lifecycle:** close/crash/account teardown fences pending work before
  destroying the window/session handlers. Stale generations cannot open or
  complete. Version launch snapshots stay immutable. Removing code is delegated
  to Host and never clears user data from this module. The Global Dandelion
  report export is permitted only when Electron reports a genuine user gesture
  for the owning Mini App webContents and the same-origin Blob JSON checks pass;
  all other downloads stay denied.
- **DMW-06 Verification:** GitHub Actions only. Tests must launch actual Electron
  on Linux and assert webPreferences, app/account origin and storage isolation,
  local HTML/JS/Wasm execution, and actual denied fetch, WebSocket, worker,
  WebRTC, navigation, popup, permission and download behavior. CSP assertions
  alone are insufficient. Native account and packaged-client acceptance remains
  a separate gate.
  Ubuntu's runner may ship Electron without a SUID helper: use a runner-local
  AppArmor profile that grants user namespaces to only the exact Electron binary.
  Do not disable AppArmor or its system-wide namespace restriction. Assert actual
  renderer seccomp filtering in addition to sandbox preferences and absence of
  the no-sandbox switch. This setup is only ephemeral Actions configuration.

## Interface and initialization sequence

Before the first `app.whenReady()`, the existing scheme-registration owner
combines `FABUSHI_MINI_APP_PRIVILEGED_SCHEME` with its existing `sand-media`
declaration in the one `registerSandMediaScheme()` call reached from
`createElectronProductionComposition` → `services.registerMediaScheme`.
There is no global WebRTC command-line switch. After ready the existing native
Plugins/MCP owner constructs `createMiniAppWindowFactory` from injected
`BrowserWindow`, `session`, and native account ports. On launch, the existing
Host returns a reverified immutable snapshot bound to the current native account
scope and generation. The factory hashes scope/app IDs for a partition and
origin, registers `session.protocol.handle` on that exact persistent session,
installs deny handlers, creates a sandboxed BrowserWindow with no preload,
awaits `webContents.setWebRTCIPHandlingPolicy("disable_non_proxied_udp")` on
that window only, and loads the fixed scheme origin. Each app session alone
gets a fixed loopback dead-end proxy. Parent account subscriptions call
`closeAccountScope` immediately upon account change; teardown advances the
generation before closing windows.

## Failures, evidence and acceptance

Unknown resource, stale account, digest mismatch, request outside the package,
Electron handler failure or renderer crash fails closed and closes/fences the
  instance. Account teardown force-destroys the untrusted window (beforeunload
  cannot veto it), cancels an active report, and keeps session deny handlers in
  force. Generation is rechecked before any content load. No auto-retry or data deletion.

| Criterion | Evidence/status |
| --- | --- |
| DMW-01–05 implementation contract | Source review; pending GHA integration |
| DMW-06 actual Electron adversarial suite | Pending GitHub Actions run |
| Signed packaged client, real account switch and offline cold launch | Blocked pending parent integration and packaged device run |

This is one platform surface inside existing Plugins/Host ownership. It does not
complete OMA-001 AC-01–AC-10 or public Marketplace approval.

## Provenance

- `AGENTS.md`, `docs/specs/desktop-catalog-package-installation.md`
- `docs/specs/offline-first-installable-mini-app-platform.md` §§7–10, 12–15
- `docs/specs/fabushi-plugins-marketplace-owner-inventory.md` FPM-OWN-010/011
- User task B and follow-up binding: native stable hashed scope and generation
- Electron APIs: [protocol](https://www.electronjs.org/docs/latest/api/protocol),
  [session](https://www.electronjs.org/docs/latest/api/session),
  [WebContents](https://www.electronjs.org/docs/latest/api/web-contents),
  [DownloadItem](https://www.electronjs.org/docs/latest/api/download-item),
  [custom scheme privileges](https://www.electronjs.org/docs/latest/api/structures/custom-scheme),
  [BrowserWindow security options](https://www.electronjs.org/docs/latest/api/structures/browser-window-options),
  [security guidance](https://www.electronjs.org/docs/latest/tutorial/security).
# Signed Dandelion interoperability gate

Actions additionally downloads immutable signed delivery artifact 11797530939,
checks transport SHA-256 and the exact .fabapp digest, and mounts its actual
manifest-listed HTML/JS/CSS/Wasm in the production window factory. It exercises
real file selection, Rust packet verification and JSON export with no Node/preload.
This is package/window interoperability, not a claim of genuine marketplace
installation or native OS file/save dialog acceptance; those remain packaged gates.

## Focused lifecycle verification plan (2026-10-14)

DMW-T-LIFE-01 (DMW-05): exercise the existing factory's renderer-gone callback,
account teardown, explicit instance close and factory disposal with an active
allowed report. Assert forced destruction, exactly-once cancellation, protocol
unregistration, persistent deny handlers, stale-generation rejection and fresh
instance recovery. Deliver late download callbacks after teardown and prove they
cannot affect a replacement instance. Also exercise report size growth and
single-active-report release through injected DownloadItem events.
These are deterministic injected-port lifecycle contracts, not real renderer
crash or native account acceptance. The actual Electron Actions suite additionally
forces a renderer crash and checks teardown/reopen using the production factory.
All execution remains GitHub Actions-only; missing evidence stays blocked. The
signed packaged client, real account switch and offline cold-launch gate remains
independent and mandatory.

