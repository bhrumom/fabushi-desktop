# Plugin SDK Core, Channel Runtime, Approval Security, Session Memory & Tool Runtime Dossier

## 1. Scope and Responsibilities
- **Scope**: Native Rust host runtime extensions and contract tests for Plugin SDK channel runtime, approval security, session memory catalog, and tool schema validation. Bulk shard coverage claims for `src/plugin-sdk/` have been revoked and downgraded to unread/unclassified pending individual per-file reading per Spec §6.2-6.3.
- **Upstream Commit**: `ad8d12f126d29b0db204187adf5092db80294e70` (tree `04ccce62f1d74dc8e0c88290ae8ded027b7b0f16`).
- **Mapped Responsibilities**:
  1. `OCRM-RESP-PLUGIN-SDK-CHANNEL-RUNTIME-001`: Channel Config Primitives, Replay Guard, Ingress/Outbound Pipeline & Debounce.
  2. `OCRM-RESP-PLUGIN-SDK-APPROVAL-SECURITY-001`: Approval Approver Resolution, Native Controls Registry, Action Sandboxing & SSRF Guard.
  3. `OCRM-RESP-PLUGIN-SDK-SESSION-MEMORY-001`: Session Catalog Paging, Transcript Memory Lock, Path Resolution & Visibility Scope.
  4. `OCRM-RESP-PLUGIN-SDK-TOOL-RUNTIME-001`: Tool Schema Definition, Command Dispatch Facade, Realtime Speech/Browser Hook Bridge.

## 2. Producer -> Owner -> Storage -> Consumer Traces
1. **Channel Runtime Pipeline**:
   - `Producer`: Inbound channel messages and streaming events from external gateways or plugins.
   - `Owner`: `source/host/src/extensions/plugin_sdk/channel_runtime.rs` (`ChannelConfig`, `ChannelReplayGuard`).
   - `Storage`: In-memory dedupe map with TTL expiration and atomic in-flight claim set.
   - `Consumer`: Gateway ingress/egress routers and outbound dispatch pipelines.
2. **Approval Security Pipeline**:
   - `Producer`: Plugin actions requiring elevated permissions or manual user approval.
   - `Owner`: `source/host/src/extensions/plugin_sdk/approval_security.rs` (`NativeApprovalControlRegistry`, `validate_outbound_url`).
   - `Storage`: Ephemeral token binding table with TTL expiration.
   - `Consumer`: Approval UI cards, action runners, and outbound network clients.
3. **Session Catalog & Memory Pipeline**:
   - `Producer`: Session queries, search requests, and transcript appends from desktop UI or agents.
   - `Owner`: `source/host/src/extensions/plugin_sdk/session_memory.rs` (`paginate_sessions`, `SessionTranscriptLockRegistry`).
   - `Storage`: Session catalog database and mutex-guarded transcript locks.
   - `Consumer`: Desktop session list view, transcript viewers, and state sync engines.
4. **Tool & Command Execution Pipeline**:
   - `Producer`: Agent LLM tool invocation requests and plugin extension commands.
   - `Owner`: `source/host/src/extensions/plugin_sdk/tool_runtime.rs` (`PluginCommandDispatcher`, `BrowserHookBridge`).
   - `Storage`: Process-local command and tab lock registries.
   - `Consumer`: LLM agent execution loops and browser automation controllers.

## 3. Verification & Oracle Evidence
- **Contract Tests**: `source/host/tests/openclaw_plugin_sdk_core_contract.rs`:
  - `test_ocrm_plugin_sdk_channel_runtime` (PASSED: validates replay guard claim, TTL deduplication, channel access policy, and BundledChannelContractValidator).
  - `test_ocrm_plugin_sdk_approval_security` (PASSED)
  - `test_ocrm_plugin_sdk_session_memory` (PASSED)
  - `test_ocrm_plugin_sdk_tool_runtime` (PASSED)

## 4. Audited Bundled Channel Contract Invariants
The following test suites in `src/plugins/contracts/` have been audited line-by-line and mapped to `OCRM-RESP-PLUGIN-SDK-CHANNEL-RUNTIME-001`:
1. `channel-config-writes.contract.test.ts` (71 lines):
   - Validates that non-channel plugins cannot declare or perform channel config writes.
   - Enforces admitted config write boundaries for bundled channels.
2. `channel-history-limit.contract.test.ts` (26 lines):
   - Enforces that channels exposing `historyLimit` support positive message fetching boundaries.
   - Restricts account-level `historyLimit` support exclusively to `line` and `mattermost`.
3. `channel-media-max-mb.contract.test.ts` (31 lines):
   - Enforces that `mediaMaxMb` configuration values must be numbers strictly greater than 0 (`> 0`).
   - Rejects non-numeric, 0, negative, or string values.
4. `channel-provider-skills.contract.test.ts` (119 lines):
   - Enforces skill eligibility based on authentication tokens (root token or named-account tokens).
   - Enforces disabled state when `skills.entries.<id>.enabled` is explicitly `false`.
   - Prohibits retired skill patterns in documentation and skill prompts (e.g. `sendMessage`/`pinMessage` for Slack, `feishu_doc_list_blocks`, `owner_open_id`).
5. `channel-reply-to-mode.contract.test.ts` (15 lines):
   - Enforces that `replyToMode` is only admitted on compatible channels (`feishu`, `irc`, `nextcloud-talk`).
6. `channel-response-prefix.contract.test.ts` (39 lines):
   - Enforces that `responsePrefix` configuration must be a valid string (including empty strings, `"[bot]"`, or `"auto"`).
   - Rejects booleans, numbers, arrays, and objects.

## 5. Channel-runtime authorization boundary (2026-10-29)

`ChannelConfig::is_user_allowed` now delegates to the shared `allow_from`
normalization owner. DM and group allowlist decisions therefore trim and
lowercase both configured entries and the sender, deny an empty sender or
empty allowlist, and recognize the explicit `*` wildcard consistently with
the locked `src/plugin-sdk/allow-from.ts` oracle. This removes a duplicate raw
comparison in the channel-runtime owner; it does not register a provider,
create channel delivery wiring, or change the ledger's `wiring`/`verification`
state.

