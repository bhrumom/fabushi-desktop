# OpenClaw Plugin SDK Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-PLUGIN-SDK-ACCESS-ACCOUNT-001`, `OCRM-RESP-PLUGIN-SDK-ACP-RUNTIME-001`, `OCRM-RESP-PLUGIN-SDK-AGENT-HARNESS-001`
- **Scope**: 40 upstream files in `src/plugin-sdk/` within `part-0042` inventory shard.
- **Owner**: `mahayana_host_runtime::extensions::plugin_sdk` (`access_groups.rs`, `acp_runtime.rs`, `agent_harness.rs`).

## 2. Audited Files & Upstream Anchors
- **Access Groups & Account Resolution** (9 files):
  - `src/plugin-sdk/access-groups.ts`, `src/plugin-sdk/access-groups.test.ts`
  - `src/plugin-sdk/account-configured-ids.ts`, `src/plugin-sdk/account-core.ts`, `src/plugin-sdk/account-helpers.ts`
  - `src/plugin-sdk/account-id.ts`, `src/plugin-sdk/account-resolution-runtime.ts`, `src/plugin-sdk/account-resolution.ts`
  - `src/plugin-sdk/AGENTS.md`
- **ACP Control Plane & Session Safety** (7 files):
  - `src/plugin-sdk/acp-binding-resolve-runtime.ts`, `src/plugin-sdk/acp-binding-runtime.ts`
  - `src/plugin-sdk/acp-runtime-backend.ts`, `src/plugin-sdk/acp-runtime.manager-api.test.ts`
  - `src/plugin-sdk/acp-runtime.test.ts`, `src/plugin-sdk/acp-runtime.ts`, `src/plugin-sdk/acpx.ts`
- **Agent Harness, Attempts & Structured Forms** (24 files):
  - `src/plugin-sdk/agent-core.ts`, `src/plugin-sdk/agent-harness-attempt-runtime.ts`
  - `src/plugin-sdk/agent-harness-completion.test.ts`, `src/plugin-sdk/agent-harness-completion.ts`
  - `src/plugin-sdk/agent-harness-exec-review-runtime.test.ts`, `src/plugin-sdk/agent-harness-exec-review-runtime.ts`
  - `src/plugin-sdk/agent-harness-mcp-forms.test.ts`, `src/plugin-sdk/agent-harness-operation.contract.test.ts`
  - `src/plugin-sdk/agent-harness-private-options.test.ts`, `src/plugin-sdk/agent-harness-registration.ts`
  - `src/plugin-sdk/agent-harness-runtime.test.ts`, `src/plugin-sdk/agent-harness-runtime.ts`
  - `src/plugin-sdk/agent-harness-session-runtime.ts`, `src/plugin-sdk/agent-harness-skill-resources.test.ts`
  - `src/plugin-sdk/agent-harness-tool-runtime.ts`, `src/plugin-sdk/agent-harness.ts`
  - `src/plugin-sdk/agent-media-payload.ts`, `src/plugin-sdk/agent-runtime-auth-profile-contract.test.ts`
  - `src/plugin-sdk/agent-runtime-model-catalog-contract.test.ts`, `src/plugin-sdk/agent-runtime-test-contracts.ts`
  - `src/plugin-sdk/agent-runtime.ts`, `src/plugin-sdk/agent-scope-runtime.test.ts`
  - `src/plugin-sdk/agent-scope-runtime.ts`, `src/plugin-sdk/agent-sessions.ts`

## 3. Rust Native Equivalence Architecture
1. **Access Group Policy & Account Identification** (`access_groups.rs`):
   - `parse_access_group_allow_from_entry` with `accessGroup:` prefix validation.
   - Membership verification with case-insensitive sender evaluation, wildcard matching, and `allow_all` override.
   - Normalized `AccountIdentifier` canonical key serialization and parsing (`provider:account_id`).
2. **ACP Runtime & Incognito Session Safety** (`acp_runtime.rs`):
   - Incognito session security enforcement: ended sessions reject access with `IncognitoSessionEnded`, sync operations rejected with `IncognitoSessionSyncAccessDenied`.
   - Doctor health report aggregation (`Ready`, `Degraded`, `Unavailable`).
3. **Agent Harness Lifecycle & Question Forms** (`agent_harness.rs`):
   - Attempt execution tracking with graceful `abort_and_drain`.
   - Structured input form modeling with required field validation.

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_plugin_sdk_contract.rs`
  - `test_ocrm_plugin_sdk_access_groups_and_account_id`
  - `test_ocrm_plugin_sdk_acp_runtime`
  - `test_ocrm_plugin_sdk_agent_harness`
- These are static Rust contract tests, not production ACP wiring or live
  service acceptance evidence. The current parity row therefore remains
  `wiring=not-wired`, `verification=not-run`, and `acceptance=pending` for the
  ACP runtime responsibility.

## 5. Production owner/consumer audit (2026-10-31)

The current checkout was re-read for `AcpSessionManager`,
`verify_session_access`, and `generate_doctor_report`. The manager is exported
from `plugin_sdk/mod.rs`, but the only caller of its session methods is
`source/host/tests/openclaw_plugin_sdk_contract.rs`. No Host/Coordinator/Runner
ACP transport, session lifecycle producer, plugin dispatcher, or typed doctor
route reaches this module.

`generate_doctor_report` counts the manager's local in-memory entries and
always emits `AcpRuntimeStatus::Ready`; it is therefore a local contract
projection, not evidence of a live ACP backend or production doctor state.
The parity integration now records the missing production producer and
consumer explicitly. No second session store, synthetic transport, or
client-supplied lifecycle was introduced; the existing
`BL-PLUGIN-SDK-ACP-WIRING-001` blocker and all pending statuses remain.

The same source/caller audit found that `AgentHarnessManager`,
`abort_and_drain`, and `StructuredInputForm` are only called by
`source/host/tests/openclaw_plugin_sdk_contract.rs`. The production session
namespace guard in `source/host/src/extensions/session/session_paths.rs` is a
separate boundary and does not launch, own, drain, or consume
`AgentHarnessManager` attempts. The parity integration therefore records no
production harness producer or consumer and retains
`BL-PLUGIN-SDK-HARNESS-WIRING-001`; the static contract implementation is not
production harness equivalence evidence.
