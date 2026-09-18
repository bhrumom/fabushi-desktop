# GBR-004 — Grok-style plugins / MCP / OAuth / private skills / workflows

Status: in-progress — real execution stack exists; exact reference provider/marketplace breadth remains open

## Objective
Recover the reference plugin architecture so every visible installable/provider item has a real execution path. Static catalog rows, fake installed/enabled booleans, and placeholder GitHub/Memory items are forbidden.

## Current executable stack
- Local providers: Files, Terminal, Browser and Computer map to real host execution.
- MCP stdio: initialize, tools/list, tools/call, process lifecycle.
- MCP Streamable HTTP: remote request path, server config, discovery/call routing.
- MCP configuration: add/remove/enable server, custom instructions, tool enable/disable.
- MCP OAuth: metadata discovery, dynamic client registration where supported, PKCE, loopback callback, access/refresh token lifecycle.
- MCP accounts: multiple account keys, secure token storage and active account selection.
- Plugins UI: server/tool/auth/account state is backed by coordinator calls.
- Marketplace: optional HTTP provider seam; search results are not hard-coded.
- Marketplace install: a catalog row is considered installable only if install metadata materializes a real MCP server and/or a real private skill.
- Private skills: file-backed `SKILL.md` records, enable/disable and model prompt injection.
- Workflows: CRUD, enable-for-agent, trigger matching/prompt injection.
- Secure secret requests: user values enter safeStorage directly and are never included in the model transcript.
- Fake GitHub/Memory catalog rows remain removed.

## Reference-backed external dependency
The pinned reference uses a DashboardService/provider for its production Marketplace catalog. The catalog service contents are not self-contained at commit `107877b4e2134fd167d239411386f09e42eadd6d`. Fabushi therefore exposes a real provider seam (`FABUSHI_PLUGIN_MARKETPLACE_URL`) rather than inventing catalog entries. Missing external catalog deployment evidence must be reported as an external dependency, not simulated as success.

## Acceptance status
1. Visible built-in capability has executable backend — **PASS**.
2. stdio MCP tools list/call lifecycle — **PASS**.
3. HTTP MCP route — **PASS**.
4. Server/tool configuration and host synchronization — **PASS for implemented settings**.
5. OAuth PKCE/refresh/multi-account secret path — **PASS for implemented providers**.
6. Private skills/workflows execute in Agent prompt/tool flow — **PASS for current local provider model**.
7. Marketplace result can install only a real backend — **PASS for provider contract**.
8. Exact reference Marketplace/provider metadata/popularity/auth-specific UI — **PARTIAL / external production service dependency**.
9. Exact MCP management/meta-tool and listener-card breadth — **PARTIAL**.
10. Event-listener connector automations — **PARTIAL**.

## Objective evidence
- real marketplace/MCP/private-skill/workflow implementation lives in:
  - `desktop/electron/grok-plugin-marketplace.cjs`
  - `desktop/electron/grok-mcp-manager.cjs`
  - `desktop/electron/grok-mcp-oauth.cjs`
  - `desktop/electron/grok-workflow-manager.cjs`
  - `desktop/electron/grok-secret-store.cjs`
  - coordinator/host wiring in `grok-agent-coordinator.cjs` and `grok-host-runtime.cjs`
- exact module-by-module state is recorded in `../parity-inventory.generated.json`.
- last confirmed source check before the newest commits: Run `35345930555` SUCCESS.

## Remaining blockers
Exact reference provider/account scoped synchronization, production Marketplace service evidence, listener/event trigger cards, remaining MCP management/meta-tool behaviors, private-skill/workflow UI visual depth, and claim-for-claim plugin IPC parity remain open under A8.
