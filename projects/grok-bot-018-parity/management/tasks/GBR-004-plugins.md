# GBR-004 — Grok-style plugins

Status: in-progress — placeholder catalog removed; local + stdio MCP execution implemented

## Actual result
The production Plugins surface no longer exposes GitHub or Memory as catalog-only placeholders. Every currently visible item has an executable backend:
- Files → local Files executor
- Terminal → foreground/background shell executor
- Browser → HTTPS open executor
- Computer → macOS screenshot/input executor
- configured MCP servers → spawned stdio MCP provider

The MCP provider performs a real JSON-RPC initialize handshake, `tools/list`, and `tools/call`. Server lifecycle supports add/remove/enable/disable and the UI supports loading server tools and enabling/disabling individual tools. Enabled MCP tools are dynamically added to the agent host tool set.

## Evidence
- MCP provider: `04eb69e0c24398bc6bb5a375753a0d43f5a43d86`
- host MCP routing: `bc3205e7eee056861f5c6d75a68c5649e68511c5`
- coordinator MCP state/call chain: `e771ff995da61fad619c86b6c7833d1ac900ecef`
- MCP UI: `9dd622db060dacf6c36f0658dc71098e5ecdd8ab`
- exact source check: Run `35333859439` PASS

## Remaining reference parity
- account/session provider
- OAuth including login/cancel/logout/status/token lifecycle
- HTTP/remote MCP and OAuth loopback where reference requires it
- provider/account-scoped plugin sync and popularity/marketplace metadata
- private skills provider and execution semantics
- workflows and their execution chain
- reference plugin auth/GitHub flow
- exact settings/custom-instructions/disabled-tool synchronization with host/coordinator

A visible provider/item must not be added until its execution path exists.
