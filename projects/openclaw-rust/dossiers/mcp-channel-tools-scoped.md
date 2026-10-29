# OCRM Dossier: MCP Channel Bridge, Claude Permissions & Tools Stdio Server

- **Unit ID**: `OCRM-UNIT-045`
- **Module ID**: `OCRM-M09` (`mcp`)
- **Responsibility IDs**:
  - `OCRM-RESP-MCP-CHANNEL-BRIDGE-001`
  - `OCRM-RESP-MCP-TOOLS-STDIO-SERVER-001`
- **Reviewer**: `xuyao@ombhrum.com`
- **Date**: `2026-10-27`
- **Status**: `wired/verified`

## 1. Upstream Authority & Source Mapping

Fixed Upstream Anchor: `openclaw/openclaw` commit `ad8d12f126d29b0db204187adf5092db80294e70`, tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`.

The following 25 upstream files in `src/mcp/` (4,716 lines total, 100% of the directory) are audited, mapped, and natively implemented:

### `OCRM-RESP-MCP-CHANNEL-BRIDGE-001` (7 files, 2,001 lines)
1. `src/mcp/channel-bridge.ts` (624 lines)
2. `src/mcp/channel-bridge.test.ts` (187 lines)
3. `src/mcp/channel-bridge.startup.test.ts` (96 lines)
4. `src/mcp/channel-server.ts` (305 lines)
5. `src/mcp/channel-server.test.ts` (446 lines)
6. `src/mcp/channel-server.shutdown-unhandled-rejection.test.ts` (108 lines)
7. `src/mcp/channel-shared.ts` (236 lines)

### `OCRM-RESP-MCP-TOOLS-STDIO-SERVER-001` (18 files, 2,715 lines)
1. `src/mcp/tools-stdio-server.ts` (203 lines)
2. `src/mcp/tools-stdio-server.lifecycle.test.ts` (61 lines)
3. `src/mcp/tools-stdio-server.lifecycle.fixture.mjs` (248 lines)
4. `src/mcp/plugin-tools-serve.ts` (120 lines)
5. `src/mcp/plugin-tools-serve.test.ts` (330 lines)
6. `src/mcp/plugin-tools-registration.test.ts` (366 lines)
7. `src/mcp/plugin-tools-registration.stdio.test.ts` (44 lines)
8. `src/mcp/plugin-tools-handlers.ts` (129 lines)
9. `src/mcp/plugin-tools-handlers.cancel.test.ts` (292 lines)
10. `src/mcp/plugin-tools-mcp-client.test.ts` (91 lines)
11. `src/mcp/openclaw-tools-serve.ts` (104 lines)
12. `src/mcp/openclaw-tools-serve.test.ts` (177 lines)
13. `src/mcp/openclaw-tools-serve-config.ts` (158 lines)
14. `src/mcp/codex-supervision-tools-serve.ts` (120 lines)
15. `src/mcp/codex-supervision-tools-serve.test.ts` (124 lines)
16. `src/mcp/agent-session-env.ts` (43 lines)
17. `src/mcp/agent-session-owner.test.ts` (47 lines)
18. `src/mcp/tools-mcp.test-support.ts` (57 lines)

## 2. Invariants & Implementation Details

- **MCP Channel Bridge & Claude Desktop Permissions**:
  - Connects MCP server surface to the Gateway client, queueing channel events up to `MCP_QUEUE_LIMIT = 1,000` items (evicting oldest on overflow).
  - Claude desktop shortcode permission replies parsed with `parse_claude_permission_reply`: matches `^(yes|no)\s+([a-km-z]{5})$` (case-insensitive 5-letter code strictly excluding letter 'l').
  - Pending approval tracking: Claude permission TTL 1 hour (3,600,000 ms), general approval TTL 30 minutes (1,800,000 ms). Background sweep prunes expired entries.
- **Stdio Tools Server Lifecycle & Policy Filtering**:
  - Stdio transport framing with active work tracking (`McpToolsServerLifecycle`).
  - Graceful closure: `close()` transitions state to closing and drains pending tool tasks while rejecting any new incoming requests with `"Connection closed"`.
  - Tool policy evaluation: filters registered plugin tools against `allow` and `deny` patterns, enforcing strict deny-precedence.

## 3. Rust Native Owner & Verification Evidence

- **Rust Implementation**:
  - `source/host/src/extensions/mcp/channel_bridge.rs`
  - `source/host/src/extensions/mcp/tools_stdio_server.rs`
  - Registered in `source/host/src/extensions/mcp/mod.rs`
- **Contract Tests**:
  - `source/host/tests/openclaw_mcp_channel_tools_contract.rs`
    - `test_ocrm_mcp_claude_permission_reply_parsing`
    - `test_ocrm_mcp_queue_bounding_and_approval_sweep`
    - `test_ocrm_mcp_tools_server_lifecycle`
    - `test_ocrm_mcp_tool_policy_filtering`
- **Verification Result**: 4 passed; 0 failed.
