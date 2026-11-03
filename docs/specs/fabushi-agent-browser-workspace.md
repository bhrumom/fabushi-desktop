# Fabushi Agent Browser Workspace — Product & Architecture Specification

- **Status:** Proposed; architecture/specification only
- **Owner:** Desktop Product / Electron Main / Agent Host & Runner
- **Date:** 2026-10-20
- **Scope:** Fabushi Desktop first; shared contracts must permit future platform adapters without promising identical extension support on every OS
- **Related existing owners:** Electron main/preload, Computer, Agent Exec, Host Runner, Browser tools, MCP/Plugins, user-data bootstrap, desktop packaging and GitHub Actions
- **Reference UX:** User-provided screenshot (ChatGPT/Codex-style split workspace with a browser tab on the right)
- **Reference implementation:** [citrolabs/ego-lite](https://github.com/citrolabs/ego-lite), especially its task-space/page API and browser-facing Agent Skill. Reference ideas and compatible MIT-licensed helper code may be evaluated; do not assume the separate ego lite browser application is included in that repository.

## 1. Executive summary

Fabushi shall gain a first-class, user-visible Chromium browser workspace that is opened from the active Bot/conversation header with a **+** action. The browser appears as a right-hand pane with its own browser tab(s), address/navigation controls, page content, downloads, extension management and an explicit relationship to the invoking Bot. The Bot can operate the page through a narrow, typed, permission-fenced Browser Agent API; the user can observe, pause, stop or take over that work without losing the current page.

This is an extension of Fabushi's existing architecture, not a second product architecture and not a new browser engine:

1. Keep Electron/Chromium as the rendering substrate.
2. Keep the current conversation, Bot identity, Agent orchestration, Host/Runner, Computer, Plugins/MCP and automation owners authoritative for their existing responsibilities.
3. Add a small Browser Workspace owner for browser-window/pane lifecycle, tabs, profiles/sessions, extensions and user-facing browser state.
4. Add a Browser Agent Bridge with explicit adapters. It connects the existing Agent tool dispatch to a selected, authorized browser target; it must not give model/renderer code unrestricted CDP or Node access.
5. Retain the existing managed Box Browser path for remote/isolated execution. It is a distinct execution target and must not be silently reinterpreted as the visible local browser.
6. Use GitHub Actions for build, tests, real Chromium E2E and packaging evidence. Do not require builds on personal workstations.

The goal is to deliver ego-lite-class capabilities where compatible with Electron and platform policy. “All ego-lite functionality” means the capability outcomes described below, not a promise of binary compatibility or exact implementation reuse. Feature parity is accepted only by executable tests and evidence, not by UI resemblance.

## 2. Product outcomes

### 2.1 Primary experience

- Each Bot/conversation header exposes a browser **+** action.
- Clicking **+** opens or reveals the right-side Browser Workspace associated with that Bot. If the Bot already owns a live workspace, reveal it rather than create duplicates.
- The browser pane has normal browser controls: back, forward, reload/stop, address/search input, secure-origin indicator, tab strip, new/close tab, loading/progress state, page title/favicon, downloads, permissions and extension entry points.
- The pane can be resized, collapsed, restored, and opened in a wider/dedicated window when the product layout supports it. Conversation content remains intact.
- The active Bot receives a browser tool surface scoped to the exact workspace and tab granted to it. It can navigate, inspect, click, type, scroll, wait, take screenshots, handle downloads/uploads under policy, and verify postconditions.
- The user sees when a Bot is controlling a page. User interaction takes priority; explicit pause/stop/takeover must fence off further Agent mutations.
- Bot work continues only under the product's normal background-work/lifecycle policy. Closing the pane hides it; it must not silently destroy a running task or keep unbounded work alive.
- Reopening the same Bot restores its browser workspace according to user preferences and profile policy.

### 2.2 Required capability set

1. **Human browsing:** navigation/history, tabs, search/address parsing, links/popups, page find, zoom, copy/paste, downloads, uploads, print where supported, basic permissions, crash recovery and accessible keyboard control.
2. **Agent control:** structured page snapshots, stable scoped element references, navigation, click, fill/type, keyboard, scroll, drag where reliable, wait, screenshot, frame-aware discovery, page events, downloads, bounded uploads and observable postconditions.
3. **Task spaces:** multiple Bots can work in parallel without stealing focus, sharing stale element references, or mutating one another's selected tab. Each task has explicit target identity and lifecycle.
4. **Login continuity:** persistent Fabushi-owned profiles and optional explicit profile selection/sharing. Never claim existing Chrome login state is automatically available. Import/migration is a separately consented, auditable flow.
5. **Extensions:** install, enable/disable, update, inspect permissions, remove and diagnose compatible browser extensions, with version/platform compatibility checks and safe defaults.
6. **Safety:** origin-aware permissions, profile isolation, no unrestricted renderer Node integration, no unrestricted CDP, user-visible operation status, bounded timeouts, cancellation and auditable high-risk actions.
7. **Reliability:** deterministic tab identity and generation fencing; stale references must fail closed; crash/reload/navigation must invalidate affected page references; no blind retries of uncertain writes.
8. **Observability:** privacy-minimizing structured diagnostics, test artifacts, action counts and source/version provenance. Never upload cookies, tokens, raw browser profiles or real page secrets as CI evidence.

## 3. Non-goals and boundaries

- Do not replace Chromium or build a rendering engine.
- Do not turn the existing Fabushi main application window into a generic remote-navigation surface.
- Do not expose Node integration, arbitrary preload APIs, unrestricted CDP, profile directories, cookies, local storage, password stores or extension internals to an Agent.
- Do not treat screenshots as the only source of truth when a safe structured snapshot is available.
- Do not automatically copy Chrome passwords/cookies/extensions or silently reuse a personal browser profile.
- Do not merge the local browser and Box Browser into one implicit security domain. The selected target must be explicit.
- Do not promise every Chrome Web Store extension works in Electron. Compatibility depends on Electron/Chromium version, extension manifest version, API coverage, platform and signing/package policy.
- Do not duplicate Bot state, conversation state, plugin/MCP install state or Agent orchestration into a new browser-owned store.
- Do not make a UI-only demo count as browser, Agent-control, extension or release acceptance.

## 4. Current architecture integration

The following existing code is the starting point and must be re-read at implementation time on the target PR HEAD; paths are ownership clues, not authorization to assume every detail is unchanged.

| Existing area | Current responsibility | Integration rule |
|---|---|---|
| `source/electron-main/main.ts` and Electron main services | App/window lifecycle, navigation hardening, app services | Register Browser Workspace lifecycle through the existing composition/service seams. Keep the primary app origin isolated from arbitrary web content. |
| `source/electron-preload/*` | Narrow renderer-to-main bridge | Add schema-validated browser IPC methods to the existing bridge pattern. No generic IPC send or arbitrary JS/CDP surface. |
| `frontend/src/recovered/features/computer/*` | Existing Computer UI and remote VNC viewer | Reuse established pane/layout patterns where appropriate; do not mislabel VNC as the local browser. |
| `source/packages/agent-exec/computer-use.ts` and generated ComputerUse protocol | Desktop-computer action contract | Keep OS-level Computer Use distinct from DOM/browser actions. Browser tools should not be implemented as fake mouse/keyboard Computer Use if a browser-native adapter is available. |
| `source/host/src/runner/tools/sand_browser_tools.rs` | Existing managed Box Browser tools, target claims, generations, screenshots and action policy | Preserve as the remote/Box adapter. Extract shared protocol semantics only where behavior is genuinely equivalent; do not weaken its existing fences. |
| `source/host/src/runner/tools/sand_browser_driver_source.rs` | Embedded managed-browser driver | Reuse tested contracts/ideas where possible; local browser adapter must not assume Box paths, display numbers or remote CDP endpoints. |
| Existing Agent coordinator and routed tool dispatch | Selects and executes Agent tools | Register browser capabilities through the existing tool registry/dispatch. Bot/workspace identity comes from trusted host-side context, not model-supplied arbitrary IDs. |
| Plugins/MCP owners and `source/electron-main/mcp/*` | Plugin/MCP install and tool lifecycle | Keep Fabushi Plugins/MCP separate from browser extensions. They may interoperate through explicitly granted integrations, but one must not silently install or execute the other. |
| `source/electron-main/startup/*` | User-data root bootstrap/migration | Register profile roots and migration with the existing startup/user-data lifecycle. Do not invent an independent profile directory lifecycle. |
| `desktop/package.json`, desktop workflows | Packaging, signing, notarization and CI | Extension packaging, Chromium tests and platform-specific gates must be included in existing GitHub Actions release paths. |

The architecture must continue to follow the existing rule: one authoritative owner per responsibility, with a new owner only when existing owners cannot reasonably hold the responsibility and the ADR explains why.

## 5. Proposed component architecture

### 5.1 Browser Workspace owner (Electron main)

Create a dedicated, small Browser Workspace service under the existing Electron main composition. Suggested modules (final names may follow repository conventions):

- `BrowserWorkspaceService`: create/reveal/close workspace; bind it to an authenticated local Bot/conversation context; coordinate tabs and lifecycle.
- `BrowserTabRegistry`: authoritative mapping from opaque `workspaceId` + `tabId` to WebContents, profile/session, origin, lifecycle state and generation.
- `BrowserProfileService`: create/select/lock/close persistent profiles, enforce per-profile session ownership and data deletion/migration policy.
- `BrowserExtensionService`: extension inventory, install/update/enable/disable/remove, compatibility checks and permission review.
- `BrowserPermissionBroker`: request, display, persist and revoke origin/extension/Agent permissions.
- `BrowserDownloadService`: safe download destination, user approval policy, size limits, filename normalization and integrity metadata.
- `BrowserAuditService`: bounded structured action records without secrets or page contents by default.

These are logical seams, not a mandate to create seven large independent subsystems. Keep them in one service package until actual complexity justifies separation. Electron main owns privileged WebContents, Session, extension and permission APIs. The renderer never creates privileged browser objects.

### 5.2 Browser surface (renderer)

Add a Browser Workspace surface to the existing conversation layout. Use the repository's existing React/UI and layout state patterns.

- Header plus button: opens/reveals the browser associated with the current Bot.
- Right pane: browser tabs and toolbar. It is a first-class pane, not a child of the website DOM.
- Browser controls are Fabushi-owned UI; website content is a separate WebContentsView.
- Keep chat composer, transcript, selection, keyboard shortcuts and accessibility semantics intact when the pane opens/closes.
- Keep pane visibility separate from Browser Workspace lifetime and Agent task lifetime.
- Never render arbitrary website HTML inside the trusted Fabushi renderer origin.

### 5.3 Browser Agent Bridge (typed contract)

Add a typed browser target interface that can be implemented by multiple adapters:

- `LocalBrowserTarget`: the user-visible Electron Browser Workspace.
- `ManagedBoxBrowserTarget`: existing Box Browser/Runner implementation.
- Future targets (if justified): remote/cloud browser; not part of this initial delivery.

The tool router must receive an explicit target selector produced by trusted host policy. No implicit fallback from local to Box or vice versa. A task may switch targets only through an explicit authorized transition.

Proposed logical interface:

```ts
type BrowserTargetKind = "local" | "managed-box";

interface BrowserTargetClaim {
  workspaceId: string;
  tabId: string;
  targetKind: BrowserTargetKind;
  generation: number;
  origin?: string;
}

interface BrowserAgentTarget {
  listTabs(workspaceId: string): Promise<BrowserTabSummary[]>;
  navigate(claim: BrowserTargetClaim, url: string): Promise<BrowserObservation>;
  snapshot(claim: BrowserTargetClaim, options?: SnapshotOptions): Promise<PageSnapshot>;
  act(claim: BrowserTargetClaim, action: BrowserAction): Promise<BrowserActionResult>;
  screenshot(claim: BrowserTargetClaim): Promise<BrowserScreenshot>;
  wait(claim: BrowserTargetClaim, condition: BrowserWaitCondition): Promise<BrowserObservation>;
  cancel(operationId: string): Promise<CancelResult>;
}
```

This is illustrative. Implementation must define the actual schema in the repository's canonical protocol location, validate all input/output at runtime, and avoid creating a second competing contract. Prefer generated types/protobuf if the existing Host/Runner contract uses them. Never accept a raw CDP command or arbitrary JavaScript evaluation as a normal Agent operation.

### 5.4 Local browser execution

- Prefer Electron `WebContentsView` managed by main process for visible web content and a Fabushi-owned toolbar. Do not use a generic `webview` tag as the new browser architecture.
- Use one explicitly selected Electron `Session` per profile. Never rely on the default session accidentally being the intended profile.
- The browser content's `webContents` is registered in the host-owned tab registry. Renderer-provided tab IDs are untrusted and must be resolved against an authorization claim.
- Browser control must be performed by a main-process/host-owned adapter with narrow APIs. The renderer may request supported actions, but cannot choose arbitrary IPC channels, filesystem paths, CDP methods or privileged objects.
- Structured DOM snapshots should be generated in a controlled context, with frame boundaries and node-reference lifetime recorded. Cross-origin frame support must follow Chromium's security model and only expose the minimum information needed for a requested operation.
- A browser navigation, reload, tab close/reuse, frame/document replacement or renderer crash increments the target generation or invalidates the claim. An action using an old generation must be rejected without side effects.
- The browser agent adapter may reuse the proven semantics from the existing driver (exact tab claim, generation fence, bounded waits, safe download/upload, explicit CDP allowlist) but must be revalidated against the local Electron implementation.

### 5.5 Local vs Box target semantics

- Local browser target: visible to the user; profile and website sessions are local; actions are reflected in the selected visible tab.
- Managed Box target: remains remote/isolated and follows current Box resource, display, generation and authorization semantics.
- The UI and Agent trace must show which target is active.
- Local target availability must not be inferred from a Box browser being reachable.
- Box-specific paths such as `/workspace/.sand-browser`, display numbers and remote CDP ports must never leak into the local target contract.
- Shared behavior may live in a pure contract/policy package only if tests prove the same semantics; transport and privilege boundaries remain adapter-specific.

## 6. Bot-to-browser lifecycle and state model

### 6.1 Identity

Every browser workspace is bound to:
- an opaque workspace ID;
- a stable local account/profile owner;
- a Bot/conversation ID from trusted coordinator context;
- a profile ID and session partition;
- one or more tab IDs;
- per-tab navigation generation and frame/document reference registry;
- current operation/task IDs and lifecycle state.

Do not use a user-visible Bot name as an identity or filesystem path. Do not allow model-generated arguments to bind a workspace to another user's conversation.

### 6.2 State transitions

Workspace states: `closed → opening → ready → hidden → closing → closed`, with `failed`/recovery states. Tab states: `created → loading → ready → navigating → crashed/closed`. Operation states: `queued → running → awaiting-user-approval (optional) → succeeded/failed/cancelled/uncertain`.

- Repeated header plus clicks are idempotent and reveal the existing workspace.
- A tab close invalidates every outstanding claim and reference to that tab.
- If the renderer crashes, restart only within a bounded policy and invalidate stale references before allowing actions.
- If an operation's result is uncertain (for example, a form submission timed out after dispatch), do not blindly retry. Re-observe the page and ask for confirmation where the action is consequential.
- Closing the right pane only hides it by default. Explicit “close workspace” may close tabs after confirming if active Agent work would be interrupted.
- Bot deletion/account sign-out must revoke bindings and apply the existing account-transition cleanup policy to profile/session data.
- The same website may be open in multiple tabs or Bot workspaces; URL equality never implies target identity.

### 6.3 Profile/session policy

Define profile scopes:
1. **Fabushi Default**: persistent profile for human browsing, user-controlled.
2. **Bot Workspace**: default for Agent-owned work; persistent or ephemeral according to user policy and origin grants.
3. **Isolated Task**: ephemeral profile for sensitive or untrusted tasks.
4. **Explicitly Shared Profile**: opt-in access to a human profile for a specific Bot/task and time/permission scope.

Never default all Bots to a single unrestricted personal profile. Human browsing and Bot browsing can share a profile only after explicit selection and with visible activity indicators. Cookie/password/storage APIs are not exposed to Agent tools. Profile export/import and Chrome migration require explicit consent, a preview, cancellation and cleanup; no secrets are included in diagnostic artifacts.

## 7. Extension architecture and acceptance

### 7.1 Extensions vs Fabushi Plugins

Browser extensions (Chrome-style extensions) and Fabushi Plugins/MCP are separate systems:
- Browser extensions execute within Chromium extension contexts and interact with tabs/pages through browser APIs.
- Fabushi Plugins/MCP provide application/Agent capabilities under Fabushi's existing install, OAuth, vault and tool-approval model.
- Extension installation must not implicitly install a Fabushi Plugin, grant MCP tools, or expose extension secrets to an Agent.
- An Agent may propose an extension installation; the user must approve it. Silent installation by web content or an untrusted Agent is forbidden.

### 7.2 Extension manager requirements

- Show installed extensions, version, source, enabled state, required permissions, optional permissions, compatibility status and last update.
- Support install from a user-selected local unpacked directory for development only, and a signed/verified package or approved source for production where supported.
- Support enable/disable/remove and a bounded update path; retain rollback/recovery information when feasible.
- Display a permission review before installation or before new/expanded permissions are granted.
- Apply an allowlist/blocklist policy where required by managed builds. Verify extension origin and integrity; reject path traversal, malformed manifests, symlink escape, oversized packages and unapproved remote code.
- Keep extension data in its declared profile/session scope. Extensions must not silently gain access to every Bot workspace.
- Ensure extension content scripts cannot cross the trust boundary into Fabushi's privileged renderer or Agent bridge.
- Expose extension-specific APIs only through a brokered, auditable capability grant. A page or extension cannot call Browser Agent tools unless independently authorized.
- Provide visible failure reasons for unsupported manifest/API/platform combinations instead of reporting false success.

### 7.3 Compatibility and platform gates

Electron extension support is not identical to full Chrome. Before claiming compatibility, record the exact Electron/Chromium versions and test:
- Manifest V3 service worker lifecycle and supported extension APIs;
- permissions, optional permissions, host permissions and content scripts;
- extension action popup/options page behavior;
- storage and extension-origin persistence;
- extension updates, disable/remove and app restart recovery;
- macOS signed/notarized packaging, Windows installer, and Linux package behavior;
- behavior when an extension API is unsupported.

The release spec must publish a compatibility matrix. Chrome Web Store integration, if pursued, must follow its terms and a supported install mechanism; do not scrape or bypass store protections. No “all Chrome extensions work” claim until the matrix demonstrates it.

## 8. Browser Agent tool contract

The initial tool set should cover these outcomes, reusing existing tool semantics where possible:

- `browser.tabs`: list/create/select/close tabs with exact target claims.
- `browser.navigate`, `browser.back`, `browser.forward`, `browser.reload`: bounded navigation with generation rotation.
- `browser.snapshot`: structured text/roles/labels/visible controls and stable reference handles.
- `browser.click`, `browser.fill`, `browser.type`, `browser.pressKey`, `browser.scroll`, `browser.drag`: operate on a fresh, authorized reference or explicitly supported coordinate action.
- `browser.wait`: bounded element, navigation, page-state or network-idle wait with documented semantics.
- `browser.screenshot`: capture the selected target; redact or avoid sensitive UI where policy requires.
- `browser.download`: track download destination, size, filename and digest; do not auto-open downloaded files.
- `browser.upload`: require explicit file selection/grant, constrain to authorized files and size, reject symlink/path escape.
- `browser.permissions`: read current grant state and request a user-mediated grant; never self-grant.
- `browser.extensions`: inspect inventory and propose installation/enablement; user approval remains required.
- `browser.stop`: cancel the current operation and fence subsequent mutations from that operation.

Rules:
- Every mutation is bound to workspace, tab, generation, account and operation ID.
- Snapshot refs are opaque, frame/document-bound and invalidated on navigation or document replacement; stale refs fail closed.
- Use explicit action schemas and bounded payload sizes/timeouts. No arbitrary `eval`, `Runtime.evaluate`, unrestricted CDP or filesystem paths.
- Action output includes observed result, URL/title, generation, timing, operation state and a safe error code. Return screenshot/image data through the existing approved result channel rather than oversized JSON.
- Tool success means observed postcondition, not merely that a command was dispatched.
- Cancellation must propagate to the underlying operation and prevent queued later actions from running. If in-flight work cannot be cancelled safely, report `uncertain` and re-observe rather than retry.
- High-impact actions (purchase, sending, deleting, publishing, changing security settings, handling sensitive account pages) require the existing Agent approval policy or an explicit user grant. Browser presence alone is not consent.

## 9. Security, privacy and threat model

Treat all websites, downloaded content, extension code, page text, page snapshots, model-provided selectors and browser events as untrusted input.

Required controls:
- Keep `nodeIntegration: false`, `contextIsolation: true`, sandbox enabled for remote content where compatible; isolate trusted Fabushi UI from website origins.
- Do not load arbitrary websites into the privileged main renderer. Use a separate WebContentsView with a restricted preload or no preload, and brokered main-process APIs.
- Validate every IPC request with a strict schema, maximum sizes, allowlisted action names, origin/target ownership and current generation.
- Restrict navigation protocols to approved web schemes by default. Intercept dangerous protocols, unexpected window opens, permission requests, downloads and file chooser actions.
- Do not expose cookies, local/session storage dumps, password-manager APIs, browser profile paths or arbitrary CDP methods to the Agent.
- Use a positive allowlist for CDP methods if CDP is needed; block script evaluation, cookie/storage export, network body capture and browser/profile/target enumeration unless a narrowly reviewed use case is approved.
- Do not use website content as instructions for privileged IPC, extension installation or tool permission grants. Prompt injection must not bypass host policy.
- Require explicit user approval for extension install, profile sharing/import, and sensitive cross-origin capabilities.
- Maintain bounded logs and redact URL query/fragment secrets, form values, auth headers, cookies, page text and screenshots by default.
- On sign-out/account switch, revoke workspace claims, stop operations, close or lock relevant sessions according to policy, and clear in-memory references.
- CI must use synthetic fixtures and disposable profiles only; never use real user accounts or personal workstation profiles.

## 10. UX details

### Browser header / tab strip
- Browser plus action is available in every eligible Bot conversation header.
- First click opens the pane with a new tab; subsequent clicks reveal the current Bot workspace.
- Browser tabs have title/favicon, loading indicator, active state, close action and accessible labels.
- Address bar supports URL navigation and search-engine query fallback; show canonical origin and certificate/security state.
- Show back/forward/reload-or-stop, new tab, downloads, extension manager and overflow menu.
- Opening external links from the trusted Fabushi shell follows existing navigation policy; links inside the browser remain in the browser subject to protocol and popup policy.

### Agent activity / takeover
- Show an explicit “Bot is using this tab” indicator and current high-level operation.
- Offer Pause, Stop and Take over. Takeover must revoke the Agent's mutation lease immediately; the Agent may resume only after a new explicit lease/grant.
- Human navigation or direct interaction can invalidate the Agent's current generation; the next Agent action must refresh state rather than overwrite the user's change.
- Preserve page state when merely switching conversations, but never let one Bot's tool calls mutate another Bot's workspace.
- Accessibility: keyboard navigation, focus restoration, screen-reader names, visible focus, no keyboard trap, and consistent shortcuts with the existing desktop shell.

## 11. Delivery phases

### Phase 0 — Architecture and contract freeze
- Re-read current main, preload, conversation header, layout state, tool registry, Browser Runner, user-data bootstrap, packaging and CI on exact HEAD.
- Produce a responsibility map and ADR confirming whether Browser Workspace can fit in existing owners.
- Define schemas, profile policy, local/Box target boundary, extension support matrix and threat model.
- No implementation starts until owner conflicts and privilege boundaries are explicit.

### Phase 1 — Human-visible local browser
- Add Bot-header plus action and right pane.
- Add WebContentsView-based tabs, address/navigation controls, safe external navigation, download/permission prompts and crash recovery.
- Add explicit persistent Fabushi profile/session handling.
- Add unit and renderer/main contract tests.
- Acceptance: users can browse normal sites, open/close/reorder tabs, navigate history, download through approved flow, restart and recover persisted tabs/profile according to settings; the chat shell remains stable.

### Phase 2 — Browser Agent Bridge
- Add LocalBrowserTarget to the existing Agent tool dispatch.
- Bind tool claims to the selected Bot/workspace/tab and generation.
- Implement snapshot, navigation, click, fill/type, key, scroll, wait, screenshot and cancellation.
- Keep ManagedBoxBrowserTarget unchanged except for shared contract adapters that preserve all existing Box fences.
- Acceptance: a Bot can complete synthetic multi-step workflows in the visible browser, with verified postconditions, stale-generation rejection, cross-Bot isolation and pause/takeover tests.

### Phase 3 — ego-lite-style task spaces
- Allow isolated Agent tabs/workspaces and bounded parallelism.
- Show running tasks and per-task target ownership.
- Support human takeover and safe handback.
- Add task lifecycle, recovery and cancellation tests.
- Acceptance: two Bots operate distinct targets in parallel without stealing each other's tabs, sharing references or changing the user's selected page.

### Phase 4 — Extension manager
- Implement extension inventory/install/permission review/enable-disable/update/remove.
- Validate exact Electron extension API support and production packaging behavior per OS.
- Add synthetic test extensions and adversarial extension fixtures.
- Acceptance: a compatible extension can be installed with consent, survives expected restart/update behavior, can be disabled/removed, and cannot access another profile or the privileged Fabushi bridge without a grant.

### Phase 5 — Parity, hardening and release
- Audit ego-lite's current documented capability set and map every relevant capability to implemented behavior, not just marketing claims.
- Fill missing browser semantics, page/frame edge cases, downloads/uploads, task spaces, extension support and reliability gaps.
- Run real Chromium E2E, Electron packaged smoke tests, signed/notarized distribution tests and platform compatibility suites in GitHub Actions.
- Publish the capability matrix, known limitations, exact version provenance and artifact digests.

## 12. Verification plan (GitHub Actions only)

All required builds/tests run in GitHub Actions on the exact tested commit. Local developer builds are not an acceptance source.

### Unit and contract tests
- Browser workspace lifecycle, idempotent plus action and Bot/workspace binding.
- IPC schema rejects unknown operations, malformed payloads, oversized strings, untrusted IDs and arbitrary CDP/eval requests.
- Session/profile ownership, account transition cleanup, consent and migration cancellation.
- Generation fencing across navigation, reload, frame/document replacement, tab close/reuse, crash and recovery.
- Snapshot ref identity, frame scoping, stale refs and duplicate URLs.
- Cancellation, timeout, uncertain write, no blind retry and operation ordering.
- Extension compatibility, permission changes, installation validation, rollback and profile scoping.
- Existing Computer, MCP/Plugins, Box Browser and main-window navigation/security tests remain green.

### Real-browser integration tests
Use synthetic local fixture sites, disposable profile roots, loopback CDP only and no credentials:
1. Open the right pane from Bot A; repeated plus click is idempotent.
2. Open Bot B; ensure its tab and profile policy are independent from A.
3. Navigate, back, forward, reload and verify actual page state.
4. Snapshot/click/fill/submit a fixture and verify the final server-observed result.
5. Navigate/reload/reuse a tab, then prove old generation/ref calls have zero side effects.
6. Open same URL in two tabs and prove target claims cannot cross.
7. Test nested/cross-origin frames within the supported policy; unsupported access fails explicitly.
8. Pause/stop/takeover during a task; queued actions do not execute after the lease is revoked.
9. Download a synthetic file; verify actual bytes, size limit and SHA-256. Test upload approval, root confinement and symlink escape.
10. Test popup, permission prompt, unsupported protocol, navigation redirect and renderer crash recovery.
11. Test two concurrent Bot tasks and verify no cross-workspace mutation.
12. Test extension install, permission prompt, content script, restart, disable/remove and unsupported API behavior with synthetic extensions.
13. Test profile persistence, ephemeral cleanup, sign-out/account switch and explicit profile sharing.
14. Confirm logs/artifacts contain no cookies, tokens, real profiles or raw secret form values.

### Packaging and release gates
- Build the app and native Host/Runner in existing Actions workflows.
- Exercise packaged Electron on supported OSes; extension support cannot be accepted from dev mode alone.
- macOS: signed/notarized artifacts and extension persistence after install/update.
- Windows: installer, extension directory/profile permissions and recovery.
- Linux: supported package formats, sandbox/runtime behavior and extension lifecycle.
- Artifact provenance records exact commit SHA, Electron/Chromium versions, OS image, test counts, failures, skipped cases, dependency lock hashes and artifact digests.
- A missing test, skipped mandatory case, mismatched HEAD, missing artifact or unverified package is a failed gate, not a pass.

## 13. Acceptance criteria

The project is complete only when all applicable criteria are demonstrated on the exact release candidate:

- **AC-01 UI:** Every eligible Bot header has a plus action that opens/reveals the right-side browser workspace; chat state is preserved.
- **AC-02 Human browser:** Navigation, tabs, history, address/search, downloads, permissions and restart behavior work in the packaged app.
- **AC-03 Agent control:** Bot operations target the exact authorized local tab and are confirmed by observed postconditions.
- **AC-04 Isolation:** Multiple Bots and duplicate-URL tabs cannot cross-mutate; stale refs/generations fail with zero side effects.
- **AC-05 Takeover:** User pause/stop/takeover revokes mutation authority and prevents queued actions.
- **AC-06 Profiles:** Persistent, isolated, ephemeral and explicitly shared profiles follow the documented policy; no silent Chrome import or cross-account leak.
- **AC-07 Extensions:** Install/review/enable/disable/update/remove works for the published compatibility matrix in packaged builds; unsupported extensions fail clearly.
- **AC-08 Security:** No remote page Node access, unrestricted IPC/CDP/eval, secret export or unauthorized profile/extension access; threat-model tests pass.
- **AC-09 Regression:** Existing Computer, Plugins/MCP, Agent lifecycle, main navigation, Box Browser and packaging gates remain green.
- **AC-10 Evidence:** All required exact-HEAD Actions artifacts, real browser observations, counts and hashes are available and reviewed.
- **AC-11 Capability parity:** A feature-by-feature matrix against the relevant current ego-lite capabilities marks each item as passed, intentionally not applicable with rationale, or blocked. No blanket “100% parity” claim with unresolved rows.
- **AC-12 Release:** Signed/packaged smoke tests and supported-platform compatibility matrix are complete.

## 14. Risks and decisions to resolve in Phase 0

1. **Electron extension API coverage:** Establish the actual extension APIs available in the pinned Electron/Chromium build and what survives production signing. This is a compatibility gate, not an assumption.
2. **Profile isolation vs convenience:** Default to Bot-scoped isolation; allow user-approved sharing with a visible indicator and revocable grant.
3. **Browser API ownership:** Prefer a single canonical schema and adapter interface rather than duplicate local and Box tool schemas.
4. **Layout ownership:** Integrate the right pane into the existing conversation/workspace layout; do not introduce a second independent navigation shell or another Bot store.
5. **Agent control channel:** All mutations must flow through trusted Host/Main authorization and generation fencing; the page and renderer must never directly grant themselves control.
6. **Extension trust:** Define approved install sources, package verification and review UX before shipping installation.
7. **Resource bounds:** Set explicit caps for parallel tabs/tasks, operation timeouts, screenshots, downloads and profile disk usage. Closing tasks must clean temporary data without deleting user-owned persistent profiles.
8. **Privacy:** Decide logging defaults and retention before implementation; evidence is metadata-first and synthetic-only.

## 15. Definition of done

- This spec is linked from the desktop architecture/source-of-truth index and has an approved owner/ADR.
- Every new responsibility is mapped to an existing owner or justified by a small ADR.
- Implementation lands in reviewable increments with contract tests before broad UI work.
- All build/test/package verification occurs in GitHub Actions on exact HEAD.
- Each phase has explicit artifacts and acceptance results; no success is inferred from screenshots alone.
- Existing Fabushi product architecture remains canonical. The browser is a new capability of the current desktop app, not a second app or second Agent platform.
