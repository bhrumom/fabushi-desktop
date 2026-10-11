# Fabushi Offline-first Installable Mini Apps — Product, Architecture & Security Specification

Status: **active (specification only; production implementation and release pending)**  
Owner: canonical Fabushi **Plugins / MCP / Marketplace** product owner; platform adapters owned by Desktop, iOS and Android repositories  
Last updated: 2026-10-11  
Related project: canonical Fabushi (PR #20 architecture), not a second product or runtime project  
Related task: user request, 2026-10-11 — mobile App as host for installable offline HTML/JavaScript/WebAssembly mini apps  
Canonical repository: `bhrumom/fabushi-desktop`; platform implementations: `fabushi-ios/fabushi-ios` and `fabu-android/fabushi-android`

> **Status discipline:** This document authorizes a product direction and defines an implementation contract. It does **not** assert that the installer, offline package runtime, Wasm support, mobile distribution approval, or any acceptance gate is already implemented. Read `AGENTS.md` and current exact-HEAD ownership before coding.

## 1. Context / problem

Today, network-hosted mini apps depend on a remote site even when the user only needs a calculator, notes, document viewer, game, or other local utility. Fabushi should instead let users discover and **install a complete, verified Web mini-app package inside the existing Fabushi App**, then launch it without a network connection. Installation is **content inside Fabushi**, not an iOS IPA, Android APK, dynamically downloaded native library, or second OS-installed application.

The design must address real constraints: browser-origin and native-sandbox isolation, untrusted downloadable code, per-account privacy, durable app data, interrupted installation/update, explicit capabilities, the existing Plugins/MCP and Host/Coordinator ownership, and Apple/Google policy. **WebAssembly is optional:** it runs in a browser engine when supported; it does not confer native privileges or exemption from store policy.

The user expressly requested the best architecture **within the current Fabushi architecture**, not a parallel marketplace or “shared core” repository. This is a newly approved Fabushi product extension; legacy Mini App adapters and their names do not automatically become production owners.

## 2. Goal

A single **Plugins / Mini Apps** experience on Fabushi Desktop, iOS, and Android that supports:

1. downloading or importing an approved, signed, immutable `.fabapp` bundle while online;
2. offline cold launch, navigation, rendering, execution, and local app data **after complete installation**;
3. HTML/CSS/JavaScript as the universal baseline; WebAssembly modules inside the Web sandbox **only where browser capabilities and distribution policy both permit**;
4. app-scoped origin, storage, lifecycle, signing, updates, quota, revocation, and clear user controls;
5. optional WebMCP/Agent integration through existing owners, without granting third-party mini apps Host, shell, account credentials, Computer, unrestricted filesystem, or native APIs;
6. platform-specific production implementations of a shared **versioned contract**, with independent source ownership in each existing repository.

**Acceptance requires production wiring and per-platform exact-HEAD evidence, not merely a valid package parser, mock WebView, test sample, or this spec.**

## 3. Non-goals / out of scope

- No new Fabushi platform, parallel “MiniAppCore”, mini-app-only Marketplace, or separate Agent/Host/Coordinator architecture.
- No user-installed APK/IPA; no downloaded DEX/JAR/`.so`, executable machine code, or JIT engine installation.
- No requirement that every mini app compile to Wasm, or that a Wasm module access native system calls.
- No promise that iOS allows arbitrary post-review Wasm bundles; **iOS Wasm distribution is policy-gated** until documented clearance.
- No unrestricted local-package sideloading on iOS in the baseline release; do not use this architecture to bypass App Review.
- No silent background execution, autonomous Agent permissions, shared per-app cookies/database, unrestricted network proxy, payment bypass, or device-wide permissions.
- No auto-migration of old hosted URL mini apps into trusted offline packages without proven publisher identity and explicit user action.
- No runtime implementation or false “supported” status as part of this docs-only change.

## 4. Stable requirements

| ID | Required observable behavior |
| --- | --- |
| **OMA-PKG-01** | A versioned cross-platform `.fabapp` specification describes complete local Web content, resource hashes, publisher identity, host API compatibility and required capabilities. |
| **OMA-PKG-02** | Installation verifies trust anchor, publisher binding, canonical signed manifest and **every asset digest** before making an app launchable. Rejected bundles never execute. |
| **OMA-PKG-03** | Installer rejects traversal/absolute paths, links, duplicate or normalization-colliding names, oversized/bomb archives, illegal MIME/path mappings, and incomplete downloads. |
| **OMA-OFF-01** | Fully installed offline-class apps cold launch with device radio/Internet disabled; **no CDN, online manifest, hosted entrypoint, remote font, external script, service worker, or login request is required** to render the offline baseline. |
| **OMA-OFF-02** | Package identity/version and account-scoped user data persist across ordinary restarts; OS eviction/low-space and uninstall behavior is explicit and recoverable where possible. |
| **OMA-ISO-01** | Each app has a distinct resource origin, execution instance and effective storage scope; one app cannot read another app, host session, or host source files. |
| **OMA-ISO-02** | Untrusted app code has **no general native bridge**; all external navigation, network requests, browser APIs, WebMCP calls and deep links obey host-enforced policy. |
| **OMA-CAP-01** | Capability grants are deny-by-default, per app/version/account/session as appropriate, explicit to the user, revocable, bounded, auditable, and cannot be reused across a new load nonce. |
| **OMA-CAP-02** | Downloaded apps cannot invoke `fabushi.app.*` semantic-control tools, account secrets, unrestricted MCP server tools, computer control, local shell or filesystem by default. |
| **OMA-LIFE-01** | Install, update, launch, suspend, resume, crash, uninstall and recovery are represented by durable state transitions; stale async results are fenced by app ID/version/account/session generation. |
| **OMA-LIFE-02** | Updates are content-addressed, validated and activated atomically; interrupted or invalid versions leave the last-good permitted version usable offline. |
| **OMA-MKT-01** | The existing Plugins Marketplace owns discovery, reviews, age/content moderation, eligibility, install/update/remove, package source and offline-ready status; Connector/Skill identities do not silently become executable packages. |
| **OMA-DATA-01** | Resource cache and user-generated data have separate ownership, quota, deletion and backup semantics; sign-out/account switch cannot leak another account's app data. |
| **OMA-WEB-01** | The common portable API is standards-based Web content; optional Wasm is a browser-side computation module, no native ABI or hidden OS entitlement. |
| **OMA-MCP-01** | Optional MCP App/WebMCP integration **reuses the existing bridge** and Host permission owner, preserves exact instance/nonce/grants/cancellation, and has no automatic tool registration. |
| **OMA-PLAT-01** | iOS WebKit, Android WebView, and Desktop Electron each have an explicit capability/policy adapter and independent shipping implementation, not a new shared runtime repository. |
| **OMA-POL-01** | Release is blocked where applicable Apple 4.7/2.5.2, privacy, age, payments, catalog/universal-link, or Google Play dynamic-code requirements are not satisfied. |
| **OMA-UX-01** | Users see size/publisher/version/offline capability/data/permission status, progress, update/rollback errors, export/delete options, and accessible offline/error states. |
| **OMA-QA-01** | All executable verification runs in GitHub Actions, with exact source SHA and actual test/artifact evidence; lack of Apple approval, device, signing, or runner is **blocked**, never passed. |

## 5. Current state — exact-source inspection, not implementation evidence

Evidence snapshot for this design:

- Desktop `bhrumom/fabushi-desktop@92fcc2b0f64b6b8f8f8829305b30874b558d6b35` (canonical `main` when inspected); `AGENTS.md` and `docs/specs/spec-first-ai-development.md` enforce spec-first and allowed verification locations.
- Existing Plugins UI and catalog/install composition: `frontend/src/recovered/features/plugins/overlay/browser.tsx`, `desktop-surface.tsx`, `desktop.ts`, and `frontend/src/production/ProductionRenderer.tsx`. The visible Plugins browser currently models plugin/server/workflow entries; this is **not evidence of an offline `.fabapp` package installer**.
- Existing SDK: `frontend/packages/mcp-app-sdk/src/bridge.ts` binds RPC over a transferred private MessagePort, `pluginInstanceId`, per-load `nonce` and `grants`; `webmcp.ts` exposes registration and local fallback; `app-surface.ts`/`app-surface-webmcp.ts` define first-party `fabushi.app.*` semantic UI tools. **SDK / Agent App Surface is not an installable package manager.**
- Existing Desktop permissions and runtimes: `source/electron-main/mcp/*`, `source/node-agent-coordinator/src/mcp*`, `source/host/src/extensions/mcp/*`, `source/host/src/extensions/local_tool_permission/*`, `source/host/src/storage/*`, and `source/electron-preload/*`.
- iOS `fabushi-ios/fabushi-ios@f098639ab178a839690b9dece60fa0d11c5d2e25`: `mobile/ios/Fabushi/MarketplaceModel.swift` is the current marketplace model; `MiniAppWebMcpSurface.swift` mounts a `WKWebView` and currently handles local HTML with `loadHTMLString` plus a synthetic base URL. This **does not prove verified multi-file package installation, durable distinct origins or offline Wasm asset loading**; these require new code and real device evidence.
- Android `fabu-android/fabushi-android@59f6fc8885ce1cb8d1ad4fc5d4ab36f690fb99a2` has an Android application tree at `mobile/android/app`. Its complete mini-app shipping entrypoint and parity must be re-inspected by the Android implementer before claiming coverage.
- Existing Grok/Fabushi and communication source-of-truth docs predate this feature; do not infer that old Mini App compatibility shims are approved to become a second product model.
- These SHAs are **design evidence snapshots**, not a promise of latest state; each implementer re-reads live `main`, active PR HEAD, source owner, and Actions before editing.

## 6. Target product model and user journeys

The **same Plugins Marketplace** exposes an installable type beside Connectors and Skills, with a distinct capability model:

- **Web mini app (offline-ready):** verified full content snapshot installed in Fabushi-owned storage; opens offline. Optional approved egress does not affect the advertised offline-core guarantee.
- **Web mini app (online-only):** hosted Web app with a distinct UX badge and stricter remote navigation policy; **never labeled installed-offline**.
- **MCP connector / Skill:** existing behavior and contracts remain; may not be treated as an installable Web package merely because listed in the same store.

User flow: `Browse → View publisher/content/size/capabilities → Install & verify → Ready offline → Launch → Manage data/permissions → Check update → Remove`. Network may be used for **discovery/download/update**; no network is required to start a fully installed offline app. A first-party preinstalled app can use the same package format. Auto-update is opt-in or a clearly disclosed store setting, must never replace the active package mid-session, and must not strand an offline user.

User-visible categories, publisher, moderation/report/block mechanism, age rating, app inventory and universal links are catalog-owned, not invented in a separate second store.

## 7. Architecture and ownership — binding decision

### 7.1 Single canonical composition

```text
Fabushi Desktop / iOS / Android native shell
  └── existing Plugins / Marketplace UI and account scope [the only store]
        ├── Connector + Skill (existing unchanged owners)
        └── Installable Web Mini App facet
              │ list / install / update / launch / remove / revoke
              ▼
      existing Host / MCP extension + platform install adapter
        ├── signed catalog + publisher trust policy
        ├── manifest verifier + bounded archive extractor
        ├── content-addressed immutable package store
        ├── durable install/activation journal + account-scoped app data index
        └── existing permission/audit/storage owners
              │ verified immutable package descriptor only
              ▼
      native Web surface (one isolated instance per app session)
        ├── Desktop Electron: sandboxed webContents/frame
        ├── iOS: WKWebView + restricted local resource loader
        └── Android: WebView + WebViewAssetLoader
              │ standard Web APIs; optional permitted .wasm
              ▼
      HTML / CSS / JavaScript / Wasm + app-scoped Web storage
              │ [optional approved and explicitly granted WebMCP]
              ▼
      existing mcp-app-sdk → scoped bridge → Coordinator → Host
```

**Owner assignment, existing-owner-first:**

| Responsibility | Existing canonical owner and permitted additions | Explicitly rejected alternative |
| --- | --- | --- |
| Entry, navigation, installed list, updates | `frontend/src/recovered/features/plugins/overlay/*` + shipping `ProductionRenderer`; equivalent native Marketplace views | Separate “Mini Apps Store” product/shell |
| Catalog, package eligibility, publisher trust & permissions | Existing Plugins/MCP catalog/policy path; narrowly add `installable-web-app` kind, verifier, store records | Treating every MCP server as a downloadable script |
| Durable lifecycle, verified package metadata & recovery | Existing Host extension/storage owners; add **minimal package subdomain** co-owned by Plugins/MCP, using existing durable storage recovery | Standalone MiniAppHost daemon / second global database truth |
| Desktop download, filesystem and Web boundary | `source/electron-main` native boundary with explicit typed IPC; `source/electron-preload` capability allowlist; Coordinator only where account/session orchestration is required | Renderer directly extracting archives or reading host filesystem |
| First-party App UI semantics and MCP App bridge | `frontend/packages/mcp-app-sdk/src/*`; scope existing private port/nonce/grants | Second WebMCP implementation or global `window` native super-bridge |
| Agent tool execution and approvals | Existing Coordinator, Host MCP and local-tool-permission owners | Downloaded app invoking Runner, OS tools or credentials directly |
| iOS platform | iOS repo's `MarketplaceModel`, `MiniAppWebMcpSurface`, `MahayanaHost` and iOS-owned native Host bindings | Copying Electron runtime or a shared Fabushi Core repository |
| Android platform | Android repo's canonical Marketplace/WebView/native Host owners, resolved at live HEAD before implementation | Parallel app shell or dynamically downloaded Android executable code |

**New-owner justification:** Package verification + content-addressed installation are real new responsibilities that a WebMCP registration helper, MCP OAuth manager, Electron attachment manager or rendered marketplace list cannot safely own alone. Introduce the **smallest cohesive installable-package subdomain within the existing Plugins/Host storage owner**; preserve existing trust, permissions, account, storage and analytics paths. If actual source inspection finds an existing compatible installable-package owner, use it instead and record the decision; do not create duplicate truth.

### 7.2 Dependency direction and source authority

Renderer/SwiftUI/Compose expresses intent; native edge validates surface and routes to **one Host-owned package state machine** per platform. Coordinator may manage session identity and Agent requests but does not copy package bytes, become a second store, or execute Web content. Runner is unrelated to passive mini-app execution; it is entered only by explicitly user-approved Agent actions.

Shared wire schemas, golden vectors and behavior semantics may be mirrored to Desktop/iOS/Android **inside their own repositories**. No `shared runtime` repository, no cross-repo mutable executable core, no Telegram-centric architecture. Desktop is the product contract authority; platform repos port responsibilities and platform-specific behavior rather than sharing running binaries.

## 8. Package format and versioned contracts

### 8.1 `.fabapp` v1 envelope (normative)

`.fabapp` is a ZIP-format resource archive with UTF-8 names and **one root `manifest.json`**. Manifest v1 is strict-schema: reject unknown security-critical fields, duplicate JSON keys, ambiguous Unicode/path normalization, unsupported version, executable/native payload declarations, or unsupported algorithm. No executable permission bits, links or special devices. Canonical UTF-8 JSON signing uses RFC 8785 JSON Canonicalization Scheme (JCS); signatures are Ed25519 over a **domain-separated v1 manifest**, excluding only the `signature` field. File table digests bind every allowed extracted resource; the envelope identifies publisher key and app ID.

Illustrative shape (hash/signature placeholders are schematic, not valid production keys):

```json
{
  "manifestVersion": 1,
  "appId": "org.example.offlinecalculator",
  "publisherId": "org.example",
  "version": "1.0.0",
  "displayName": "Offline Calculator",
  "entrypoint": "index.html",
  "runtime": "web",
  "minHostApi": 1,
  "offlineCore": true,
  "requestedCapabilities": [],
  "networkPolicy": { "mode": "deny", "allowedOrigins": [] },
  "files": {
    "index.html": {
      "sha256": "<64 lowercase hex>",
      "bytes": 1835,
      "mime": "text/html"
    },
    "assets/app.js": {
      "sha256": "<64 lowercase hex>",
      "bytes": 3072,
      "mime": "text/javascript"
    },
    "assets/calc.wasm": {
      "sha256": "<64 lowercase hex>",
      "bytes": 8192,
      "mime": "application/wasm"
    }
  },
  "signature": {
    "algorithm": "Ed25519",
    "keyId": "example-publisher-key-1",
    "value": "<base64url signature>"
  }
}
```

**Normative verifier order:** bounded download to staging → archive central-directory/streaming limits → path/duplication/link normalization → parsed strict manifest → verified Fabushi catalog **publisher-to-appId key authorization** + signature + revocation/expiry/compatibility → full file-size/hash verification including `index.html` and Wasm bytes → content-addressed immutable store → durable journal commit → atomic active-version pointer. **A publisher self-signed package is not automatically trusted**; catalog policy binds publisher signing key to ID and the permitted platform/content/age/capability set. Offline installation from files is allowed only for previously verified catalog/authorized policy or a separately approved local-import policy; **default iOS marketplace has no arbitrary sideload path**.

ZIP limits: initial proposed policy defaults are **100 MiB compressed per app, 250 MiB expanded, 10,000 files, max 100:1 inflation for any file and 500 MiB aggregate installed cache**. Limits must be configurable by channel/device capacity but **may not be widened without security review**. Use normalized case/Unicode comparison as if on a case-insensitive filesystem and reject links, path escapes (`..`, absolute, drive, NUL), root duplicates, hidden external resources, unexpected files, zip bombs and arithmetic overflow. Strict content-type mapping (especially `application/wasm`); do not execute from unverified temporary directories.

### 8.2 Catalog/trust and update protocol

Catalog record: `{appId, publisherId, metadata, ageRating, categories, reportingLink, universalLink, bundleUrl, digest, version, platformPolicy, minHostApi, status, trustEpoch}`. Catalog metadata is **signed or delivered through an authenticated trusted policy channel with a locally pinned trust root**, separate from untrusted download locations. Publisher key rotation and store signing-key rotation require an explicit versioned trust update and replay protection. Untrusted CDN location cannot upgrade trust by itself.

Releases are immutable by `(appId, version, packageDigest)`. Normal activation accepts a monotonic new version; rollback may activate only a **previously installed verified, still-authorized** digest through the documented recovery path. Reject version reuse with changed bytes, downgrade replay, revoked publisher/app and expiry beyond defined policy. **Offline behavior:** already installed last-good content works without refreshing the online catalog unless a locally cached hard revocation/security hold prohibits launching; online-required install/upgrade/revocation refresh may fail closed without breaking otherwise permitted existing offline launch.

### 8.3 Host boundary (logical contract, not an exposed OS API)

Privileged calls are **native UI → typed Host service only**; they are not script-callable JavaScript APIs in v1:

```ts
type MiniAppId = string;
type PackageDigest = string;
type MiniAppState =
  | "absent" | "downloading" | "staged" | "verifying"
  | "installed" | "updating" | "ready" | "running"
  | "suspended" | "quarantined" | "uninstalling" | "error";

type MiniAppDescriptor = {
  appId: MiniAppId; version: string; digest: PackageDigest;
  publisherId: string; offlineCore: boolean;
  policyEpoch: number; hostApi: number;
};
type MiniAppScope = {
  accountScope: string; appId: MiniAppId; version: string;
  instanceId: string; loadNonce: string; generation: number;
};
type MiniAppCommand =
  | { kind: "listInstalled" }
  | { kind: "installApproved"; appId: string; version?: string }
  | { kind: "launch"; appId: string }
  | { kind: "suspend"; appId: string; instanceId: string }
  | { kind: "update"; appId: string }
  | { kind: "uninstall"; appId: string; data: "retain" | "delete" };
```

Exact wire serialization, schema namespace, concurrency control and error shapes are added as repository-owned contracts during implementation; the values above are the **minimum semantics**, not license to create a new public Host method family behind the existing owners without review. Calls are idempotent using `operationId`/journal identity; response always includes immutable descriptor, scope generation and structured error codes, not raw filesystem paths, HTML or account tokens. UI never supplies an arbitrary download URL to bypass catalog trust. If Platform policy denies capability, surface `unsupported_policy` rather than trying a privileged fallback.

### 8.4 Web app ↔ host

Baseline mini apps use normal Web APIs (DOM, CSS, fetch of **verified in-package** resources, IndexedDB/localStorage where supported). The app may import JS/Wasm from its *own* verified origin only. No generic `window.fabushiNative`, `WKScriptMessageHandler` tool multiplexer or Android `addJavascriptInterface` on third-party pages.

Optional WebMCP integration is a **separate privileged feature flag**, not implicit in package install. Reuse `McpAppBridge` private channel with `pluginInstanceId`, unpredictable per-load `nonce`, method grants, origin/frame source validation, bounded request IDs/timeout, cancellation and teardown. A local WebMCP fallback registry is in-app JavaScript only and **never acts as a second Host-grant owner**. `fabushi.app.*` controls remain first-party Host UI tools and are unavailable to downloaded content. iOS may only enable native bridge abilities where Apple's written approval/policy assessment permits; v1 **no native API extension to downloaded mini apps**.

## 9. Web origin, storage and platform policy

### 9.1 Isolation and content policy

- **Every installable mini app has a unique, stable browser origin** derived from its validated ID, not one global shared origin. Do not put different app IDs under a shared same-origin path and assume paths isolate storage.
- **Android:** use AndroidX `WebViewAssetLoader` and a reserved per-app HTTPS virtual origin (or a verified equivalent), with `shouldInterceptRequest` mapping **only** manifest-listed files to private immutable bytes. Deny all misses, block unauthorized redirects and navigation; explicitly disable risky file/universal file access and mixed content. **Do not allow reserved-domain misses to escape to the network**.
- **iOS:** use `WKWebView` with a vetted restricted local-resource strategy (e.g. `WKURLSchemeHandler` for a non-http(s) scheme and per-app host, or `loadFileURL(_:allowingReadAccessTo:)` confined to the app's immutable directory). **Do not treat `loadHTMLString` plus a fabricated HTTPS base URL as a verified offline package loader.** URL-scheme, Origin, fetch, IndexedDB, Wasm, worker and MIME behavior must be exercised on supported iOS versions/real devices **before choosing the final shipping loader**. If a strategy cannot achieve app-to-app isolation and required offline APIs, it **fails the gate**; never widen `file://` access or claim unsupported Web API parity.
- **Desktop:** use existing Electron browser-content isolation patterns with one per-instance sandboxed webContents/iframe, session partition/origin isolation, `nodeIntegration: false`, no arbitrary preload APIs, scoped navigation and protected session lifecycle. Untrusted renderer must not share the production renderer's privileged origin.
- Content Security Policy default baseline: deny by default (`default-src 'none'`), app-only script/style/assets, `connect-src 'none'` for offline mode, `object-src 'none'`, `base-uri 'none'`, `frame-ancestors`/navigation policy appropriate to embedded surface. Do **not** silently enable eval, arbitrary workers/remote scripts, cross-origin images or unreviewed `wasm-unsafe-eval`. Wasm CSP may be allowed **only after** browser/policy tests and stays inside app sandbox. Any Web-only remote origin must be explicitly declared and reviewed.
- Screen/tab/app lifecycle teardown closes private channels, unregisters tools, cancels pending requests, removes observers, and fences late `postMessage` or Host results. Web process crash cannot leave grants alive.
- Do not assume Service Worker or PWA installation is required or supported in `WKWebView`. **The native verified package store is the offline source of truth**, not browser cache.

**Egress warning:** a navigation delegate **alone** does not block every script subresource, fetch, WebSocket or worker path. Implementation must prove deny/allowlist enforcement for actual network traffic using policy, content restriction and platform-supported interception, or disable that capability on the platform. Security review must document any engine limitation; a CSP string without negative tests is not acceptance evidence.

### 9.2 Data ownership

Separate (a) immutable, re-downloadable app code/cache, (b) per-app **user content** and (c) any authorized host↔app shared data. Code packages are keyed by digest and may be physically de-duplicated with safe account-independent identity; users' saved data is keyed by **account + appId + origin/profile**, encrypted or OS-protected as appropriate, never cross-account. Sign-out closes sessions and denies prior-account access. Uninstall explicitly offers **remove code only / remove code and data**, subject to verified policy and platform UI. Update cannot silently delete user-generated content; use transactional schema migration with rollback or fail without activation. OS may evict browser-managed data, so **do not promise durable note storage solely because WebView IndexedDB exists**; define export/backup/storage durability in platform acceptance. A host-backed storage bridge on iOS is **not approved by this spec** unless compliant native-API permission is obtained.

If data is shared with an individual mini app, require explicit and contextual consent for that app and that data category; no inherited account permission, silent sharing, or global implicit Agent access. Analytics must not collect user app contents.

### 9.3 Platform release matrix (conservative default)

| Capability | Desktop | Android | iOS |
| --- | --- | --- | --- |
| Offline signed HTML/CSS/JS package | Target supported after packaged gate | Target supported after instrumentation | Target supported **only after `WKWebView`/policy/device proof** |
| Wasm file as WebView module | Browser feature-gated | Browser + Google Play policy-gated | **Disabled for downloaded packages until Apple's applicable language/code policy is cleared**; test-only cannot be mistaken for release permission |
| Arbitrary externally sourced packages | Disabled by default; optional explicit developer/import policy | Disabled by default; optional reviewed local import | Disabled in baseline public App Store distribution |
| Custom native APIs exposed to downloaded mini apps | No general API; allow narrowly scoped reviewed bridge | No general API; optional reviewed bridge | **Denied without required Apple permission** |
| App data/permissions | Account/app bounded | Account/app bounded | Account/app bounded; Web-standard storage baseline |
| Payments / entitlements | Existing Fabushi owner | Google Play/channel-specific rules | Existing StoreKit/Apple 3.1 policy owner |
| Offline-running background Agent tasks | Not included | Not included | Not included |

Before iOS distribution, check Apple App Review **2.5.2, 4.7, 4.7.1–4.7.5, 5.1, 3.1** and Mini Apps Partner Program where relevant. Specifically: app catalog/index with metadata **and universal links**, content filtering/reporting/blocking, age protections, privacy consent, payment compliance, and **no platform-native API extension to downloaded code without Apple's permission**. **Do not assume a WebAssembly module is an Apple-approved downloadable mini-app language simply because WebKit runs Wasm.** Android must satisfy Google Play's downloaded code / interpreted-WebView exception and all other abuse/security rules. Policies change: re-verify on submission date and record reviewer/approval evidence. These gates apply to downloaded content independently of whether local execution works.

## 10. Lifecycle state machines, failure modes and recovery

### 10.1 Install and version state machine

```text
absent
  → downloading → staged → verifying → installed/ready
        │            │          │
        └─ error/retry with bounded backoff; no visible unverified content
ready → running ↔ suspended → ready
ready → updating → staged → verifying → [atomic active pointer] → ready
  └─ update failure → last-good ready, no partial resource mix
any allowed state → quarantined (cached revocation/explicit security hold)
ready/suspended → uninstalling → absent (with independent data retention choice)
```

Operations are scoped by appId, account, packageDigest and transaction ID. Concurrency is serialized **per app** with bounded independent parallel downloads; repeated install/uninstall/update requests are idempotent. When active, an app executes against one immutable digest; update activation must close/restart that instance or defer until the next launch. A crash after download, verification, content move or pointer flip recovers deterministically from durable journal. Partial staging is garbage-collected only when unreferenced; no cleanup may delete the last-good package or user data.

### 10.2 Negative paths / explicit behavior

1. **No connectivity:** installed offline app opens; marketplace search/update can show offline; fresh install cannot claim success without full package verification.
2. **Corrupt/unsigned/tampered package:** clear security error, delete staging only, no execution.
3. **Publisher key compromised/revoked:** block new installs/updates immediately when known; apply signed cached local security hold to affected installed digest; retain/export user data through trusted host UI.
4. **Low disk/quota:** preflight budget and bounded staging; cancel/update failure leaves active good package and user data intact.
5. **Power loss/process termination between file move and journal commit:** restart repairs journal; no mixed file tree.
6. **Account sign-out, switch, session rollover, app navigation:** revoke grants, cancel requests, isolate data and ignore stale responses.
7. **URL / file path trickery, Unicode collisions, infinite redirect, malicious MIME, attempted external links, JS iframe/popup/worker/WS egress:** default deny with negative test evidence.
8. **Wasm unsupported/blocked by policy:** show supported-platform explanation; never silently download a native executable, disable sandbox or use a hidden JIT.
9. **Failed data migration/update:** defer activation, preserve original schema and user data, enable clean rollback.
10. **Offline entitlement:** only a validated cached entitlement with approved offline policy may unlock previously acquired content. New purchases/restores and server-dependent rights fail clearly offline; do not invent purchases.
11. **Web process crash or hung script:** destroy instance, revoke channels, enforce CPU/memory budgets where OS permits, offer restart, and do not spin a background retry loop.

## 11. Implementation plan: owner-first, parallelizable, no fake completion

| Track | Work | Deliverable / required proof |
| --- | --- | --- |
| **P0 — contract/security** | Re-read live Desktop/iOS/Android owners; fix host/store identity, draft and implement strict `.fabapp` verifier + catalog trust binding + golden vectors; design threats; establish platform/legal decision log. | Manifest grammar/versioned contract, threat model, fuzz/property/negative tests in Actions; policy status recorded separately. |
| **P1 — existing Plugins integration** | Add the `installable-web-app` facet to **existing** Plugins UI/catalog/Host storage and typed commands; add immutable package store + journal, offline data separation and rollback. | End-to-end Desktop install/offline launch/update/uninstall using a real signed sample on packaged Electron. |
| **P2 — platform implementations** | iOS: implement native restricted loader and WebView origin policy inside current Marketplace/miniapp owner, investigate durable Web storage; Android: current-head owner resolution, WebViewAssetLoader + native store adapter. | Simulator/emulator and real-device offline/attack/permission/crash evidence; **independent repo-owned code**, no shared core. |
| **P3 — optional capabilities** | Add feature-probed browser Wasm; release-enable per platform only after explicit code-distribution review. Optional WebMCP tool registration/Agent handoff uses existing SDK, Host grants, lifecycle and permissions. | WebAssembly conformance, nonce/revocation/cancel tests, App Review/Play review evidence. A platform block does not block unrelated offline HTML/JS work. |
| **P4 — review/release** | Accessibility, moderation, age/catalog/universal links, payment/privacy approvals, telemetry, staged rollout and rollback drills. | App Store/TestFlight/Play/distribution acceptance and exact-shipping-source provenance, not CI-only green. |

**Blocked-track rule:** if iOS Wasm approval or physical-device access is blocked, record it and continue independent verifier, Desktop/Android, offline JS, data and UX tracks. Do not lower gate or claim cross-platform completion. **No phase is a substitute for full final acceptance.**

## 12. Verification / tests — exclusively GitHub Actions

All executable actions including lint, format, compile, unit, property, fuzz harness, schema generation, integration, UI and packaged acceptance **run only in GitHub Actions**. Do not use the assistant's container or user's Mac/device for build/test. Tests must record command, exit code, exact commit SHA, runner OS, evidence artifact/digest, test counts and failed cases. If a real device / signing / Apple review is unavailable, report **blocked**.

| Verification ID | Requirements | Positive and negative acceptance |
| --- | --- | --- |
| **T-PKG** | PKG-01/02/03 | JCS/Ed25519 cross-language golden vectors, strict schema, publisher binding, content digest; mutations, key swap, duplicate names, symlink, traversal, bombs, overflow, unknown fields and unlisted files fail closed. |
| **T-OFF** | OFF-01/02, WEB-01 | Signed HTML/JS sample cold launches **airplane-mode from a freshly restarted app** with no remote resource; optional Wasm sample executes entirely from verified local bytes where enabled. No reliance on service worker cache. |
| **T-ISO** | ISO-01/02, CAP-01/02 | Two installed apps cannot read each other's origin/cookies/IndexedDB, host token, local files or remote MCP; cross-instance nonce replay, iframe spoof, stale grants, navigation/teardown rejection, denied network, WebSocket/worker/fetch paths. |
| **T-LIFE** | LIFE-01/02, DATA-01 | Kill at every install/update journal edge; concurrent install/update/uninstall; data schema rollback; off-network update failure; last-good app remains usable and account switch never leaks data. |
| **T-MKT** | MKT-01, UX-01 | Existing Plugins UI keeps Connectors/Skills functional; installable app shows accurate size/author/offline/permissions/version/progress; keyboard/screen reader/reporting/age UX. |
| **T-MCP** | MCP-01 | Existing bridge session/nonce/grants verified; no `fabushi.app.*` to downloaded UI; reject pending requests on dispose; fallback is in-page only; no native privileges on iOS absent approval. |
| **T-PLAT** | PLAT-01, POL-01 | Desktop signed packaged Electron + iOS simulator/**device** + Android emulator/**device**, across supported OS; policy compliance register and release blockers inspected; Wasm enabled only on cleared platform. |
| **T-PERF** | OFF-01, LIFE-01, UX-01 | Record cold/warm offline launch p50/p95, install CPU/storage amplification, WebView memory, frame responsiveness, idle wakeups and energy; compare against a documented device baseline before setting final SLOs. |

Test sample matrix: fully bundled calculator, persistent notes, local-only image asset app, optional Rust→Wasm computation, update-with-migration, intentionally corrupt or malicious package. Include WebView version/OS differences, content security policy and MIME/`fetch` conformance; a green browser-only unit test does not satisfy packaged/device acceptance.

## 13. Acceptance criteria / Definition of Done

| ID | Must be proven on shipping paths |
| --- | --- |
| **AC-01** | One marketplace owner: Connectors and Skills preserved; Mini App facet uses existing navigation, account, permissions and Host/Coordinator boundaries. No second Marketplace/Core/runtime. |
| **AC-02** | Package format, trust keys, exact bytes, extractor limits and signed catalog are verifiable; negative adversarial suite passes; untrusted code never launches. |
| **AC-03** | On **each claimed platform**, a fresh real installed offline app cold launches while network is disabled, reads its bundled assets and can complete supported Web/Wasm actions with no external access. |
| **AC-04** | Two apps and two accounts pass isolation, consent and denial tests, including malformed frame messages and any native bridge paths. |
| **AC-05** | Interrupted update/reboot, data migration/rollback, low disk, disabled network, downgrade/revocation and uninstall preserve last-good allowed content and user data as specified. |
| **AC-06** | Platform matrix truthfully gates native APIs and Wasm; store-policy index/universal links, report/block/moderation, age, privacy, payments and review records exist for relevant distribution channel. |
| **AC-07** | Real native iOS/Android Web surfaces use locally verified complete packages and evidenced per-app origin/resource handling, not synthetic hosted URLs, test-only HTML injection or directory-name inference. |
| **AC-08** | Exact-HEAD GitHub Actions static/security/integration/E2E + relevant signed/installed acceptance all completed/success; every failure repaired without weakening gates. |
| **AC-09** | User can discover, install, launch, inspect permissions/storage, export/delete data (where supported), update and uninstall with accessible offline/error UI. |
| **AC-10** | Release includes revocation/rollback, privacy/observability review, source identity, store policy and platform-specific approvals; blocked items remain blocked, not “implemented/verified.” |

A Desktop-only release may be designated **Desktop preview**, Android-only **Android preview**, or iOS HTML/JS-only **iOS policy-limited preview** when its own scoped ACs pass. None may be called “cross-platform/Wasm-complete” until every claimed platform and feature has its own evidence.

## 14. Release, migration, rollback and observability

- **Migration:** legacy hosted Mini App entries remain hosted. Only matching publisher-verified signed packages may be offered as separate opt-in installations. Do not silently copy hosted app cookies/localStorage into new local origin or elevate an existing plugin/connector's privileges.
- **Rollout:** feature flag by platform/channel; start curated internal signed packages, then opt-in catalog, then broader store only after policy and abuse systems. Keep catalog release policy and package binary immutable.
- **Rollback:** atomically switch to last-good permitted digest; disable updates or a capability independently; quarantine compromised package without silently erasing user content; offer trusted host-level data export/removal. Retain only bounded old versions.
- **Observability:** appId, publisher fingerprint (non-secret), package digest, policy epoch, account pseudonymous scope, operationId, state transition, verifier rejection code, bytes/duration, platform/WebView version, load nonce hash, crash count, rollback reason, offline start result, denied capability/network event. **No app document content, auth token, full local path or private MCP payloads in logs.** Honor privacy/consent.
- **Provenance:** store GitHub workflow/run/job URL, exact HEAD, device matrix, signed app build hash, signed package hash, catalog/publisher trust hash, signing and App Store/Play review evidence. Automatic source HEAD changes invalidate stale final evidence.

## 15. Security and policy review checklist (release blocking)

- Threat actors: malicious publisher, compromised CDN/catalog, compromised signer, malicious installed Web content, cross-app/account data thief, replayed bridge message, forged download, resource bomb, network exfiltration, compromised device state.
- All assets verified **before** execution; trust policy pinned, key rotation/revocation/anti-rollback documented.
- Different app origins and account scopes actually isolate browser data (not just different URL path prefixes); clipboard, external deep link, downloads and user gesture boundaries enforced.
- iOS 4.7 content moderation/report/block, index with universal links, age restrictions, 4.7.2 native API prohibition and 4.7.3 per-app explicit privacy consent demonstrated.
- Native/Agent tool access **not** inferred from an MCP tool name, `readOnlyHint`, `window` fallback registry or package signature. Separate Host authorization and per-load grant required.
- Test the **actual WebKit/WebView versions** for CSP, Wasm MIME/streaming, browser storage, local scheme and origin behavior. Unknown = unsupported/blocked, never permissive fallback.

## 16. References / provenance (revalidate when implementing)

### Repository source of truth

- `AGENTS.md` and `docs/specs/spec-first-ai-development.md`; `docs/specs/SPEC_TEMPLATE.md`.
- `docs/specs/grok-bot-018-runtime-product-parity-recovery.md`; `projects/fabushi-communication-platform/SOURCE_OF_TRUTH.md` — single canonical product architecture and existing-owner-first principle.
- Desktop snapshot: `https://github.com/bhrumom/fabushi-desktop/tree/92fcc2b0f64b6b8f8f8829305b30874b558d6b35`.
- iOS snapshot: `https://github.com/fabushi-ios/fabushi-ios/tree/f098639ab178a839690b9dece60fa0d11c5d2e25`.
- Android snapshot: `https://github.com/fabu-android/fabushi-android/tree/59f6fc8885ce1cb8d1ad4fc5d4ab36f690fb99a2`.

### Platform and format references (checked 2026-10-11; normative policy is current official text on release date)

- Apple App Store Review Guidelines: `https://developer.apple.com/app-store/review/guidelines/` — ``2.5.2`, ``4.7–4.7.5`, ``3.1`, ``5.1`.
- Apple Mini Apps Partner Program: `https://developer.apple.com/programs/mini-apps-partner/`.
- Apple `WKWebView.loadFileURL`: `https://developer.apple.com/documentation/webkit/wkwebview/loadfileurl(_:allowingreadaccessto:)`.
- Android `WebViewAssetLoader`: `https://developer.android.com/reference/androidx/webkit/WebViewAssetLoader`.
- Android local Web content security: `https://developer.android.com/develop/ui/views/layout/webapps/load-local-content`.
- Google Play Device and Network Abuse / executable code: `https://support.google.com/googleplay/android-developer/answer/16559646`.
- RFC 8785 JCS: `https://www.rfc-editor.org/rfc/rfc8785`.
- Latest user request: 2026-10-11 “本地小程序 / App 作为宿主 / HTML、JS、Wasm / 离线安装运行”，并明确要求把最佳架构 Spec 放入现有 Fabushi 仓库。

## 17. Spec compliance record (at the time of this design-only commit)

| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| Spec-first and owner inspection | **passed** | Read root `AGENTS.md`, template, Grok/FBCP owner guidance, Desktop Plugins/MCP SDK and iOS/Android source snapshots listed in `5`. |
| OMA-PKG-01 through OMA-QA-01 — shipping functionality | **blocked** | Design specified; no production changes or exact-HEAD runtime evidence in this docs-only commit. |
| AC-01 through AC-10 — release acceptance | **blocked** | Depends on subsequent implementation, platform policy review, verification, signed builds and release gates. |

**Handoff rule:** Before implementing, reread exact current main/PR HEAD, check for conflicts with current FBCP/TDRP/iOS/Android task owners, update this spec if necessary, and record each requirement's implementation + exact-head acceptance evidence. “Spec merged” is not “feature implemented.”

## 18. AI-assisted mobile creation, preview and private installation — OMA-AI (2026-10-11)

**Status:** approved target architecture / **implementation and acceptance pending**. This section is a **normative extension** to OMA-001 §§1–17, not a new product or a change to the existing Grok/FBCP acceptance scope. It answers the additional product requirement: **a user may ask Fabushi AI on a phone to make a mini app, inspect/preview it, explicitly install it into Fabushi, and run it offline**, without needing a laptop or an App Store submission for each mini app. The App Store eligibility of the host's resulting dynamic-code behavior **remains a release gate**, not an assumption.

### 18.1 Scope: two kinds of "offline"

- **Offline execution (baseline requirement):** the Agent may generate a project using a cloud model *while online*. After the project is verified and installed as a complete local Web package, **launch/use/edit saved local data works without network**. This is the primary cross-device experience.
- **Offline AI generation (separate optional feature):** the **AI itself** creates or revises HTML/CSS/JavaScript without Internet only when an appropriate **on-device code-generation-capable model is installed, available, permitted and sufficiently capable** on that exact device. iOS may feature-probe Apple's on-device Foundation Models where eligible, **but Apple's documentation explicitly lists code generation as a use case to avoid for the on-device model, so passing model availability alone is never sufficient to claim reliable offline mini-app creation**; Android may feature-probe suitable ML Kit GenAI / AICore Prompt API support, with explicit output-budget and per-file capability checks. The host may additionally support a separately reviewed local inference adapter, but must respect model size, thermal, RAM, battery, language, storage and sandbox limits. Unsupported devices explicitly show "离线 AI 生成不可用" while still running already installed local mini apps.
- Never describe cloud AI creation as fully offline; never imply that **all** iOS/Android devices have a usable on-device coding model. **Local-model claim requires an independently demonstrated code-generation quality gate, not mere model API availability.** Downloading a large model or sending local files to a cloud AI requires distinct explicit user consent.
- **Code format:** generate standalone or multi-file HTML + CSS + JavaScript and local assets by default. Generated Wasm is **not required**; a phone Rust/C/C++ toolchain, a native downloaded compiler or just-in-time OS-native execution is **not** authorized. An optional precompiled Wasm artifact is permitted only when covered by the same package integrity, Web sandbox and platform distribution-policy gates of §§8–9.

### 18.2 Existing-owner-first architecture (binding)

```text
Fabushi Chat / Agent creation intent on iOS or Android
     │ user: "做一个可以离线使用的记账小程序"
     ▼
existing Agent / Coordinator / Host / Runner creation and artifact flow
     ├── Cloud inference (online; disclose/consent for sent data)
     └── Supported device-local model (truly offline; capability gated)
     │
     ▼
Account-scoped LOCAL SOURCE DRAFT (inert data, versioned)
   index.html + style.css + app.js + assets/* + source digest
     │ static policy checks and complete-resource validation
     ▼
DISPOSABLE UNTRUSTED WEBVIEW PREVIEW (no native privileges)
     │ user feedback → model edits draft → fresh preview/digest
     ▼
USER REVIEWS SOURCE / PERMISSIONS / SIZE AND CLICKS INSTALL
     │ explicit approval bound to exact draft digest
     ▼
existing Plugins / MCP / Marketplace package-install owner
     │ host-created private-local-author signature
     │ same .fabapp archive rules / verifier / account journal
     ▼
"My Mini Apps" → PRIVATE INSTALLED WEB APP
     │ separate app origin and durable data; atomic versions
     └── cold launch and use fully offline
```

**No second owner:** Agent/Runner creates draft source as an artifact, *not privileged application code*. The existing Plugins Marketplace owns the installed-app list and install/update/uninstall UX. Its native Host package-install subdomain owns verified bytes, signature validation, durable journal and version pointer; the iOS/Android native Web surface runs the resulting app. The existing permission system owns any permitted optional grants. Agent, WebView, MCP bridge and remote model **never own installation authority or signing keys**. This builds on the current `frontend/packages/mcp-app-sdk`, `frontend/src/recovered/features/plugins/overlay/*`, Desktop Host/Coordinator, iOS `MarketplaceModel` / `MiniAppWebMcpSurface` and Android native Web owner, with each platform implementing in its existing repository. Do not introduce `MobileMiniAppCore`, a separate builder daemon or a shared runtime repository.

**Created locally is not "published".** The user's private app is shown under **Plugins → My Mini Apps → Created locally**; it is not in the public Marketplace catalog, cannot inherit a publisher reputation, and is not reachable by other accounts without a separate, moderated publication/sharing flow.

### 18.3 Stable AI requirements

| ID | Normative behavior |
| --- | --- |
| **OMA-AI-01** | User asks the existing mobile Fabushi Agent to create or modify an offline mini app by natural language; source files, assets, app name and a versioned draft are produced within Fabushi. User can see, edit and export source. |
| **OMA-AI-02** | Source output is **untrusted inert data**, not Agent/Host-executable code. All drafts pass schema/path, CSP, import/URL, size and content-policy validation before isolated preview or installation. |
| **OMA-AI-03** | A preview is ephemeral and uses a separate isolated origin/session/profile without OS/native/MCP/admin permissions, external network access or another installed mini app's storage; failure/cancel/timeout destroys the instance. |
| **OMA-AI-04** | The user explicitly approves installation of the **exact draft digest** after reviewing source, requested permissions, data scope and size. No auto-install, background publish, automatic permission escalation or model-granted consent. |
| **OMA-AI-05** | The canonical `.fabapp` verifier, content-addressed immutable store, account isolation, native loader, update journal, rollback and uninstall apply equally to AI-generated packages. No unsigned or bypass install path. |
| **OMA-AI-06** | **Cloud-assisted creation and offline AI creation are separately detected and disclosed**. A ready compatible on-device model may create while offline; model missing/incompatible → explicit unsupported state with preservation of draft and installed apps. |
| **OMA-AI-07** | AI revisions produce a new draft digest; upgrading an installed app requires a new valid signed package, user-visible changes and explicit activation. Last-good allowed install and user data survive failed AI edits. |
| **OMA-AI-08** | AI prompt/source permission boundaries block accidental sensitive-data upload, prompt-injection tool escalation, Host/shell/Computer/MCP access, cross-app reads, network egress and silent payment actions. Cloud prompts that include user files require separate informed consent. |
| **OMA-AI-09** | Code output uses Web standards first; Wasm only if verified and approved. Neither iOS local runtime nor Android loads generated machine code, arbitrary native binaries, DEX, downloadable runtime engines or compilers as a shortcut. |
| **OMA-AI-10** | iOS App Review §§2.5.2/4.7 and Android/Google Play policy **independently assess executing AI-generated private Web code**. If a release channel forbids that behavior, disable *installation* in that channel and offer only a policy-compliant editor/source export or preview **if independently permitted**. Never declare store eligibility from technical proof alone. |

### 18.4 New trust class, explicit exception to §8.1 catalog verification

§§8.1–8.2 assume a **catalog-authorized publisher key**. Private AI-generated projects require a **strictly separate local-author trust domain**, not a relaxed verifier:

- Manifest v1 now recognizes an optional strict-schema discriminant `packageClass` with enum values `catalog` and `local-generated`. Absence means `catalog` for backward compatibility. The rest of the canonical `.fabapp` envelope, JCS/Ed25519 signature, asset hashes, paths/zip limits, compatibility, MIME and policy checks remain mandatory.
- `catalog` continues to require **trusted catalog publisher-to-app-ID key binding** and relevant moderation/distribution gates, as §8.1 states.
- `local-generated` can be constructed **only inside a trusted Fabushi creation transaction** for the authenticated user's current project, with **native-host allocated reserved private app ID**, active account binding, exact project/draft digest, explicit user approval ID and device/installation-local signing key held in secure native storage. The native Host installer signs **after** source is validated. The model and generated Web content cannot call the signer, register keys or propose their own privileged signature.
- The verifier dispatches to a **disjoint** trust-root path based on validated class, not by trying all keys. A local-author key **only** authorizes its own reserved private namespace and currently authorized account/installation. It is **not trusted for downloaded/imported third-party archives, a public marketplace publisher, a different account, or a forged `local-generated` bundle**. Raw signing keys are never exposed to JavaScript or exported.
- For `local-generated`, local source approval and a genuine Host-allocated draft ID are mandatory and must survive durable journal validation. If the consent is absent, expired, for another digest, or replayed in another account, **reject**. A local preview is **not** installation and is never automatically signed.
- A local-generated app uses the same denied-by-default isolated Web surface as third-party packages. Host signatures show verified provenance/integrity, **not that AI code is safe, user-consented, store-authorized or eligible for extra privileges**.
- Publishing to other users, syncing across devices or submitting to Marketplace is a separate audited process: publisher enrollment, review/moderation/age and rights checks, explicit sharing consent, new publisher signature, fresh catalog authorization, and no transfer of local device signing keys. Do not silently sync private code/project content.
- An explicit “import external unsigned app” request may not be routed to the local-author signer to launder untrusted downloaded code. Such imports remain prohibited by default and require a separate reviewed policy.

The §8.3 Host boundary gains a privileged **native-UI-only** intent:

```ts
type AiMiniAppDraft = {
  projectId: string; accountScope: string; generationMode: "cloud" | "device-local";
  generationId: string; draftDigest: string; fileDigests: Record<string, string>;
  previousInstalledDigest?: string; requestedCapabilities: string[];
};
type LocalInstallIntent = {
  kind: "installLocalGenerated"; projectId: string;
  approvedDraftDigest: string; userApprovalId: string;
};
```

These are **logical contract semantics**, not an exposed JavaScript or IPC bypass. The Host authorizer binds intent to the active account, user gesture, approved source version, package verifier and atomic install journal. Error types distinguish `model_unavailable`, `draft_invalid`, `preview_blocked`, `source_changed`, `consent_required`, `unsupported_platform_policy`, `insufficient_storage` and `verification_failed`; no raw OS file paths or secrets are returned. Store sources and AI draft sources can share a verifier but never a trust-root shortcut.

### 18.5 Security, privacy, lifecycle and UX

1. **Creation projects are private**, versioned and bounded. An Agent's edit tool can write only its current draft/source files through existing artifact ownership, never another mini app's saved database, Fabushi tokens or native filesystem. Generated source is inspectable and editable before installation. Model output is parsed as data and cannot invent an effective grant, tool action or app-store signature.
2. The preview has **a different origin and profile** from the eventual installed app; no read/write access to the installed app's user data, clipboard or MCP by default. A preview run is **product runtime**, not a developer build/test; its zero-privilege validation is a security boundary, not a claim of formal test success. Use disposable scope, CSP, blocked requests, bounded memory/time and process teardown; untrusted JS and Wasm are never run in Host/Coordinator/Runner native contexts.
3. Prompt-based repair is allowed, but **every** revision gets new validation, exact digest and preview. After installation an update must follow the existing atomic package activation + data-migration journal. Model interruptions, low disk, Web process crash or failed edits leave the last-good installed app usable.
4. Distinguish **"AI created online, app runs offline"**, **"AI created fully offline, app runs offline"** and **"local model not available"** in the UX. Store versions and sources can remain accessible offline. Local AI model download is opt-in and cannot be required to run installed apps.
5. Sending private files, note data or chat content to a remote coding model requires explicit understandable consent; privacy protection extends to generation logs/provenance. Retain non-sensitive model/provider/version, generation mode, package/draft digests and error classes; do not record raw personal prompts, secrets, file contents or user app data in analytics.
6. App Store `4.7.2` prohibits exposing native platform APIs to offered mini-app software without prior Apple permission; `4.7.3` requires explicit per-app consent before sharing data/permissions. The local-author trust class **does not exempt AI-generated software from §§2.5.2 or 4.7**. Distribution approval must be separately documented.

### 18.6 Plan, GitHub Actions verification and acceptance gates

**Plan:** First implement a constrained HTML/CSS/JS **online-AI → on-phone preview/install → offline-use** flow in existing Agent and Plugins owners; this is the least resource-intensive baseline. In parallel, build the local-author signing/installer verifier path, native Web-origin isolation, draft/consent transaction and real mobile acceptance. After these gates pass, add optional on-device model support where the model and device meet measured quality/thermal/memory budgets; do not make it a dependency of ordinary offline apps. Independent iOS/Android implementations own native adapters. iOS policy blockage must not halt Android/Desktop, shared contract/golden vectors, or documentation and security work.

**T-AI-GENERATE:** With exact-HEAD GitHub Actions evidence, exercise an actual Agent request to generate a multi-file calculator and offline notes mini app with all assets; inspect generated source, exercise safe preview, consent and in-app local signing; restart host and **cold launch in airplane mode** on the target mobile platform. Then request a revision, reject partial/malicious output, perform a user-approved update, verify data preservation and rollback. This covers OMA-AI-01/02/03/04/05/07.

**T-AI-TRUST:** Reject a package with a forged local-author signature, leaked/replayed consent, edited post-approval bytes, imported untrusted source pretending to be locally generated, wrong account, swapped origin or duplicate code identity. Reject attempts to use native Host, shell, high-privilege MCP, Computer, file reads, remote fetch/WebSocket and cross-app storage. Source provenance is attributable but privacy-redacted. This covers OMA-AI-04/05/08/09.

**T-AI-MODEL:** Run cloud-generation+offline-execution separately from true no-network model+generation+execution on an actually eligible device with model already downloaded. Cover unavailable model, model download-needed, unsupported locale/device, thermal/resource limits and safe fallback. **An offline-installed app is NOT proof of offline AI generation.** This covers OMA-AI-06.

**T-AI-POLICY:** Record the current Apple/Google rules at submission time, app moderation/age/index/privacy requirements where applicable, downloaded/local-created script policy, allowed scripting/preview/origin scheme behavior and any Apple prior permission needed. Absent review/signing/device proof = `blocked`; no policy-deemed pass from unit tests. This covers OMA-AI-10.

| Acceptance ID | Required evidence |
| --- | --- |
| **AC-11 AI-to-install** | An actual existing Fabushi mobile Agent creates editable source, the app displays a sandboxed preview, the user explicitly approves the exact draft, the regular signed/verified local-author package path installs it under My Mini Apps, and the app launches offline after restart. No second owner. |
| **AC-12 Fully offline generation (only if claimed)** | A real eligible phone generates source with **no network or cloud inference**, previews, explicitly installs and launches the result with airplane mode enabled. Missing local model must fail honestly without disabling already installed mini apps. |
| **AC-13 Security and policy** | Local-author and catalog trust are disjoint; no privilege/publisher/consent laundering; offline/resource/cross-app/Host boundaries survive malicious source and model prompt-injection tests; iOS/Android distribution acceptance and privacy/age requirements are documented separately. |

**Compliance at documentation submission (2026-10-11):** OMA-AI-01 through OMA-AI-10, T-AI-GENERATE/TRUST/MODEL/POLICY and AC-11–AC-13 are **blocked for production acceptance**, because this commit does not implement or run the feature. They supplement—not replace—§17 and AC-01–AC-10. Only the design/specification and owner-resolution record is completed.

**Supporting references:** Apple App Review Guidelines §§2.5.2, 4.7–4.7.5, 5.1 — https://developer.apple.com/app-store/review/guidelines/; Apple's on-device Foundation Models availability — `https://developer.apple.com/documentation/foundationmodels/systemlanguagemodel`; Android ML Kit on-device GenAI — `https://developers.google.com/ml-kit/genai/prompt/android/get-started`; Android WebViewAssetLoader — `https://developer.android.com/reference/androidx/webkit/WebViewAssetLoader`.
