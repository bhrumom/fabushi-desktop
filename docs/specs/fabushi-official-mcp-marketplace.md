# Fabushi official MCP marketplace — Specification
Status: active
Owner: Fabushi Plugins/MCP
Last updated: 2026-10-08

## 1. Context / problem
The user requests a Fabushi-owned official marketplace listing common provider-operated MCP servers, especially Google and GitHub. Canonical main is 3bc92400826cc4ca7ac665b467708e22261edc61. The current recovered MCP catalog, effective-install lookup and HTTP MCP execution depend on Cursor Dashboard. Fabushi account credentials are correctly refused for that foreign backend; the UI's all-or-nothing loading hides even available catalog entries.

## 2. Goal
Extend the existing Plugins surface and Host/Coordinator MCP relay with a Fabushi-maintained catalog, durable account-scoped installation, provider credential isolation and real remote MCP discovery/execution. Distinguish listed, installed, authorized and connected.

## 3. Non-goals
Do not rebuild Google/GitHub APIs, impersonate provider publishers, weaken account-token origin checks, replace Coordinator/Host/Runner, or invent provider tools. Do not claim OAuth/live production acceptance before configuration and evidence.

## 4. Requirements
- FMCP-001: list GitHub and Google Gmail, Drive, Docs, Sheets, Slides, Calendar, Chat and People using exact provider-documented HTTPS endpoints. Google listings disclose Developer Preview, Cloud project/API enablement and OAuth prerequisites.
- FMCP-002: distinguish Fabushi directory curation from upstream provider ownership. Stable namespaced IDs, descriptions and official provenance are bundled and available without foreign Dashboard access.
- FMCP-003: installation/update/removal and account credentials persist in the existing encrypted storage mechanism, in a dedicated file isolated from Box secret export. Fabushi account identity scopes every operation. No third-party service receives a Fabushi session token.
- FMCP-004: installed connectors participate in shipping MCP server/tool inventory and Coordinator/Host routed tools. Implement initialize, notifications/initialized, tools/list pagination and tools/call with Streamable HTTP JSON and SSE responses. Requests are bounded, redirects refused, tools validated and errors redacted; uncertain writes are never automatically retried.
- FMCP-005: install without authorization remains needsAuth. Explicit provider token setup is supported as an interim route (GitHub PAT / Google OAuth access token), stored only in encrypted native storage and never returned by catalog/list/setup reads. Production one-click OAuth needs registered Fabushi client/broker configuration and is a separate blocked acceptance item, not silently replaced by token setup.
- FMCP-006: failures in legacy installed/custom connectors must not hide the official directory or its installed connectors. Show a partial-load warning; do not silently report legacy lookup success.
- FMCP-007: preserve custom/team/private-skill flows and existing permission/tool-disable gates. Account switches, uninstall or credential changes fence in-flight results. Native error/tool output must not reveal provider credentials.
- FMCP-008: all executable verification runs in GitHub Actions. No local build/test/lint/generator.

## 5. Current state
source/shared/node/mcp/mcp-marketplace.ts reads Dashboard catalog; desktop-mcp-manager.ts owns the production manager and routed tool facade. frontend PluginsDesktopSurface loads catalog, effective plugins and server state together. The existing encrypted SandUserSecretsStore supports dedicated store paths and account scopes. OAuth applications/Google preview entitlement have not been confirmed.

## 6. Target state
Existing Plugins entry displays the Fabushi official catalog. Install configures a real remote connector; actual authorization and MCP discovery determine status/tool count. The directory remains available during foreign Dashboard failures. Normal OAuth login will be activated only after registered application configuration and live acceptance.

## 7. Architecture / ownership
Catalog metadata lives in shared MCP. Native connection state, encrypted vault and transport are narrow Electron MCP edges, composed by desktop-mcp-manager.ts. The existing independent Coordinator -> Host routed tools path remains the caller; no second Agent/runtime or Mini App market is introduced. Legacy plugins retain their current owner.

## 8. Contracts / data flow
Catalog IDs use fabushi-official-*; server IDs use the same namespace. Native installed records contain plugin ID, token, account label and disabled tools in encrypted JSON. Renderer receives only public metadata/status/tools. Unknown IDs cannot select arbitrary remote hosts. Install values accept optional ACCESS_TOKEN; updates with absent/blank token preserve the current token. Disconnect deletes credentials; uninstall deletes the install. OAuth start without registered configuration returns not-supported with an actionable reason. Successful tools/call is projected into canonical generated MCP results. Tool discovery/calls are fenced against current account and install revision.

## 9. Constraints
HTTPS fixed endpoints, no redirects, 60s request timeout, bounded response size, provider credentials only. No automatic retries of writes. Offline public catalog; no pretend connection or hardcoded tool inventory.

## 10. Failures / edge cases
401/403 -> needsAuth; preview denial is a provider error. Missing secure storage -> session-only state via the existing vault, never plaintext persistence. Logout/account change/removal invalidates pending output. Legacy failures produce a visible warning. Pagination has a bounded page count and rejects repeated cursors. Unsupported content types/protocol errors fail explicitly.

## 11. Implementation strategy
Commit this spec first. Add provider-source catalog; add injectable native connection/transport owner with existing encrypted vault adapter; compose catalog/install/server/tool lifecycle in the current production facade. Repair renderer partial-load handling. Add Actions contracts and existing renderer/Electron build checks. Publish a reviewable PR, inspect exact-head results, then integrate only when required gates pass.

## 12. Verification
Actions unit/contract tests: all official endpoints/provenance; install without credentials; update/disconnect/uninstall; account isolation and stale-result fencing; no account token sent to provider; JSON/SSE MCP initialization and discovery/call; redirects/errors/no write retry; partial-load UI state. Existing production renderer and Electron compile/build gates cover shipping composition.

## 13. Acceptance
- AC-1: catalog visible in existing Plugins, with all nine entries and honest provider/preview labeling.
- AC-2: account-scoped encrypted install/update/remove and real MCP tool discovery/call pass exact-head Actions.
- AC-3: Coordinator/Host uses enabled installed connector tools; disabled tools cannot execute.
- AC-4: fresh packaged product connects GitHub and Google through registered Fabushi OAuth, handles disconnect/expiry/re-auth, and records exact-head live evidence.
- AC-5: current-head CI, integration and release verified. Code push alone is not completion.

## 14. Release / rollback
Deliver through a dedicated main-based PR. No schema migration is needed for the dedicated native encrypted vault. Existing installs are retained. Rollback removes official connector composition without altering legacy installs; users may remove the dedicated encrypted native installs.

## 15. Observability / evidence
Retain exact commit/run/job/step links. Emit sanitized error classes and partial-load warnings; never log tokens, raw auth responses or secrets. Live OAuth/configuration blocks remain explicit.

## 16. References
User request: create Fabushi official marketplace and list common Google/GitHub MCPs.
Canonical Plugins requirements CONN-001..008,010 in docs/specs/grok-bot-018-runtime-product-parity-recovery.md.
https://developers.google.com/workspace/guides/configure-mcp-servers
https://developers.google.com/workspace/gmail/api/guides/configure-mcp-server
https://github.com/github/github-mcp-server
https://github.com/github/github-mcp-server/blob/main/docs/host-integration.md

## 17. Spec compliance
| Requirement / AC | Status | Evidence / reason |
| --- | --- | --- |
| AC-1..3 | blocked | implementation/Actions pending |
| AC-4 | blocked | Fabushi OAuth registration and Google preview configuration not confirmed |
| AC-5 | blocked | integration/release evidence pending |
