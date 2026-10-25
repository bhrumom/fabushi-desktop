# OCRM Technical Dossier: Round 94 — OpenClaw Agent Turn & Channel Delivery Native Fusion

## 1. Executive Summary

This dossier records the full-source native Rust equivalence and fusion for:
1. **`OCRM-M05` & `OCRM-M06`**: Agent Harness, ACP, Turn Execution & Model Catalog
2. **`OCRM-M14` & `OCRM-M15`**: Channels Ingress, Routing & Outbound Delivery

All responsibilities from upstream `openclaw/openclaw` in `src/agents/`, `src/acp/`, `packages/agent-core/`, `packages/acp-core/`, `src/channels/`, and `src/routing/` are mapped natively into canonical Fabushi Rust architecture under `source/host/src/runner/` and `source/host/src/connectors/`.

Additionally, this round documents the discovery and remediation of an SSRF bypass vulnerability in `source/host/src/runner/tools/sand_web_tools.rs` where bracket-enclosed IPv6 hosts (such as `[::1]`) were not stripped prior to IP address parsing.

---

## 2. Upstream Source to Canonical Fabushi Rust Mapping

### 2.1 Agent Harness & Turn Execution (`OCRM-M05` / `OCRM-M06`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/agents/agent-loop.ts` | Active turn run loop & conflict detection | `TurnRunShell::begin_run` (`TurnRunShellError::AlreadyRunning`) | Host Runner |
| `packages/agent-core/turn-lifecycle.ts` | Turn attempt lifecycle & generation tagging | `TurnOwnerToken { request_id, generation }` | Host Runner |
| `src/agents/steer.ts` | Interrupt & cancel active streaming turn | `TurnRunShell::interrupt`, `CheckpointBoundary::Cancel` | Host Runner |
| `src/agents/safety/turn-taint.ts` | Viral turn tainting on unreviewed execution | `SessionSecurityContext`, `is_tainted` | Host Runner |
| `src/acp/acp-review.ts` | Preflight auto-review gate | `SandAutoReviewDecision::Approved` / `Denied` | Host Auto-Review |
| `src/model-catalog/pricing.ts` | Model prompt & completion token billing | `TurnUsage`, `merge_turn_usage`, `ModelPricing` | Host Usage / Runner |
| `src/provider-runtime/fallback.ts` | Provider failover chain on HTTP 429 | `ProviderFallbackChain` | Host Inference Router |
| `src/context/compaction.ts` | Context window token threshold compaction | `ContextWindowTracker` | Host Runner |
| `src/agents/tools/argument-validator.ts` | Tool parameter schema validation | `validate_tool_schema` | Host Runner |
| `src/agents/settlement.ts` | Turn settlement receipt emission | `TurnSettlement::settle`, `TerminalOutcome` | Host Settlement |
| `src/agents/watchdog.ts` | Zombie attempt lease reaper | `TurnWatchdogLease` | Host Supervisor |

### 2.2 Channels Ingress & Outbound Delivery (`OCRM-M14` / `OCRM-M15`)

| Upstream Source Path (`openclaw`) | Upstream Responsibility | Target Fabushi Rust Symbol | Process & Boundary |
|---|---|---|---|
| `src/channels/dedup.ts` | Inbound message cursor + id deduplication | `InboundChannelDeduplicator` | Host Channel Connector |
| `src/channels/routing.ts` | Thread routing & conversation affinity | `ChannelThreadRouter` | Host Roster / Session |
| `src/channels/delivery-port.ts` | Channel delivery port registration & dispatch | `HostChannelDelivery`, `ChannelDeliveryPort` | Host Connector |
| `src/channels/dead-letter.ts` | Fallback to DLQ on unregistered delivery | `UnregisteredChannelDeliveryPort`, `ChannelDeadLetterQueue` | Host Connector |
| `src/channels/attachments.ts` | Local file attachment resolution & 50 MiB cap | `resolve_channel_attachment`, `CHANNEL_ATTACHMENT_MAX_UPLOAD_BYTES` | Host Attachment |
| `src/channels/chunking.ts` | Message chunking with code fence preservation | `split_channel_message` (4,000 char threshold) | Host Connector |
| `src/channels/rate-limit.ts` | Rate limiter backoff on HTTP 429 Retry-After | `ChannelRateLimiter` | Host Connector |
| `src/channels/receipts.ts` | Delivery receipt settlement state machine | `DeliveryReceiptTracker`, `DeliveryStatus` | Host Connector |
| `src/channels/replay.ts` | Dead letter replay upon reconnection | `ChannelDeadLetterQueue::pop_front` | Host Connector |
| `src/channels/backoff.ts` | Exponential backoff with max attempt bounds | `ReconnectBackoffPolicy` | Host Connector |
| `src/channels/webhook-verify.ts` | Webhook HMAC SHA-256 signature verification | `verify_webhook_hmac_sha256`, `sha256_hex` | Host Security |
| `src/channels/acl.ts` | Channel allowlist / denylist authorization | `ChannelAclGate` | Host Security |

---

## 3. Falsifiable Test Oracles Summary

- **Agent Harness (`source/host/tests/openclaw_agent_turn_contract.rs`)**:
  - `OCRM-AGENT-O01`: Active inference stream conflict rejection (`TurnRunShellError::AlreadyRunning`).
  - `OCRM-AGENT-O02`: Deferred supersedes on pending turn start.
  - `OCRM-AGENT-O03`: Immediate supersedes on tool execution cancellation.
  - `OCRM-AGENT-O04`: Monotonic generation increments in `TurnOwnerToken`.
  - `OCRM-AGENT-O05`: Viral turn tainting upon unreviewed arbitrary tool execution.
  - `OCRM-AGENT-O06`: Auto-review decision resolution clearing execution gates.
  - `OCRM-AGENT-O07`: Model pricing calculation and turn usage merge integrity.
  - `OCRM-AGENT-O08`: Provider fallback chain on HTTP 429 rate limit exhaustion.
  - `OCRM-AGENT-O09`: Context window token count tracking and compaction triggers.
  - `OCRM-AGENT-O10`: Tool call argument schema preflight validation and rejection.
  - `OCRM-AGENT-O11`: Runner settlement receipt emission and outcome finalization.
  - `OCRM-AGENT-O12`: Zombie turn attempt lease expiration and watchdog timeout.

- **Channels & Delivery (`source/host/tests/openclaw_channel_delivery_contract.rs`)**:
  - `OCRM-CHANNEL-O01`: Inbound channel message deduplication (cursor + message_id).
  - `OCRM-CHANNEL-O02`: Thread routing & conversation affinity resolution.
  - `OCRM-CHANNEL-O03`: Delivery port registration lifecycle in `HostChannelDelivery`.
  - `OCRM-CHANNEL-O04`: Unregistered delivery port fallback to dead letter queue.
  - `OCRM-CHANNEL-O05`: Attachment size limit enforcement (50 MiB ceiling).
  - `OCRM-CHANNEL-O06`: Outbound message chunking preserving code fence balance.
  - `OCRM-CHANNEL-O07`: Channel rate limiter backoff on HTTP 429 Retry-After.
  - `OCRM-CHANNEL-O08`: Delivery receipt settlement lifecycle.
  - `OCRM-CHANNEL-O09`: Dead letter replay queue draining after reconnection.
  - `OCRM-CHANNEL-O10`: Reconnection exponential backoff with bounded attempts.
  - `OCRM-CHANNEL-O11`: Webhook HMAC SHA-256 signature verification.
  - `OCRM-CHANNEL-O12`: Channel access control list (ACL) allowlist/denylist gating.
