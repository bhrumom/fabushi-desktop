# OpenClaw LLM Layer Parity Dossier

## 1. Overview & Responsibility Mapping
- **Module ID**: `OCRM-RESP-LLM-TRANSPORT-RATE-LIMIT-001`, `OCRM-RESP-LLM-PROVIDER-STREAM-WRAPPERS-001`, `OCRM-RESP-LLM-OAUTH-REGISTRY-001`
- **Scope**: 55 upstream files covering 100% of `src/llm/` (including `src/llm/providers/` and `src/llm/utils/`).
- **Owner**: `mahayana_host_runtime::extensions::inference` (`llm_transport_rate_limit.rs`, `stream_wrappers_service.rs`, `oauth_registry_service.rs`).

## 2. Audited Files & Upstream Anchors
- **Core Transport & Rate Limiting** (20 files):
  - `src/llm/ai-transport-host.ts`
  - `src/llm/ai-transport-host.test.ts`
  - `src/llm/model-registry.ts`
  - `src/llm/model-runtime-binding.ts`
  - `src/llm/model-transport-context.ts`
  - `src/llm/openai-compatible-auth.test.ts`
  - `src/llm/provider-rate-limit.ts`
  - `src/llm/provider-rate-limit-stream.ts`
  - `src/llm/simple-completion-sentinel-egress.test.ts`
  - `src/llm/stream.ts`
  - `src/llm/stream.complete-host.test.ts`
  - `src/llm/stream.sync-host.test.ts`
  - `src/llm/tool-result-redaction.test.ts`
  - `src/llm/types.ts`
  - `src/llm/utils/event-stream.ts`
  - `src/llm/utils/node-http-proxy.ts`
  - `src/llm/utils/node-http-proxy.test.ts`
  - `src/llm/utils/retry.ts`
  - `src/llm/utils/retry.test.ts`
  - `src/llm/utils/stream-wrapper.ts`
- **Provider Stream Wrappers & Adapters** (26 files):
  - `src/llm/providers/anthropic-cloudflare.fetch-proof.test.ts`
  - `src/llm/providers/minimax-fast-mode.ts`
  - `src/llm/providers/openai-fast-mode.ts`
  - `src/llm/providers/stream-wrappers/anthropic-cache-control-payload.test.ts`
  - `src/llm/providers/stream-wrappers/anthropic-family-cache-semantics.ts`
  - `src/llm/providers/stream-wrappers/anthropic-family-tool-payload-compat.live.test.ts`
  - `src/llm/providers/stream-wrappers/anthropic-family-tool-payload-compat.test.ts`
  - `src/llm/providers/stream-wrappers/anthropic-family-tool-payload-compat.ts`
  - `src/llm/providers/stream-wrappers/google-thinking-payload.ts`
  - `src/llm/providers/stream-wrappers/google.test.ts`
  - `src/llm/providers/stream-wrappers/google.ts`
  - `src/llm/providers/stream-wrappers/minimax.test.ts`
  - `src/llm/providers/stream-wrappers/minimax.ts`
  - `src/llm/providers/stream-wrappers/moonshot-thinking.ts`
  - `src/llm/providers/stream-wrappers/moonshot.ts`
  - `src/llm/providers/stream-wrappers/openai-service-tier-observation.test.ts`
  - `src/llm/providers/stream-wrappers/openai-service-tier-observation.ts`
  - `src/llm/providers/stream-wrappers/openai.test.ts`
  - `src/llm/providers/stream-wrappers/openai.ts`
  - `src/llm/providers/stream-wrappers/proxy.test.ts`
  - `src/llm/providers/stream-wrappers/proxy.ts`
  - `src/llm/providers/stream-wrappers/reasoning-effort-utils.test.ts`
  - `src/llm/providers/stream-wrappers/reasoning-effort-utils.ts`
  - `src/llm/providers/stream-wrappers/stream-payload-utils.test.ts`
  - `src/llm/providers/stream-wrappers/stream-payload-utils.ts`
  - `src/llm/providers/stream-wrappers/zai.ts`
- **OAuth Provider Registry & Handshake** (9 files):
  - `src/llm/oauth.ts`
  - `src/llm/utils/oauth/abort.test.ts`
  - `src/llm/utils/oauth/abort.ts`
  - `src/llm/utils/oauth/anthropic.test.ts`
  - `src/llm/utils/oauth/anthropic.ts`
  - `src/llm/utils/oauth/index.ts`
  - `src/llm/utils/oauth/openai-chatgpt.test.ts`
  - `src/llm/utils/oauth/openai-chatgpt.ts`
  - `src/llm/utils/oauth/types.ts`

## 3. Rust Native Equivalence Architecture
1. **Admission Controller & Rate Limiting Gate** (`llm_transport_rate_limit.rs`):
   - Real `RateLimitGate` and `RateLimitAdmissionController` with prioritization queue ordering (`Interactive` preempts `Background`).
   - Timeout threshold bounding (`TimeoutExceeded` when retry delay exceeds `max_retry_delay_ms`).
   - Terminal assistant error classifications according to upstream `retry.ts` (`repeated_tool`, `repetitive_output`, `provider_failure_with_output`, `provider_post_dispatch_ambiguity`, `websocket_non_retryable_close`, 4xx non-retryable vs 429/5xx transient).
   - Egress secret sentinels unwrap and redaction (`__OPENCLAW_SECRET_...__` masked to `[REDACTED_SECRET]`).
2. **Provider Stream Wrappers & Adapters** (`stream_wrappers_service.rs`):
   - Anthropic Bedrock ARN detection and Claude family cache semantics (`AnthropicDirect`, `AnthropicBedrock`, `CustomAnthropicApi`).
   - Tool schema normalization ensuring valid JSON schema objects (`{}` -> `{"type": "object", "properties": {}}`).
   - OpenAI service tier observation with cooloff circuit-breaker fallback to default tier on rejection.
   - Google Gemini 2.5 Pro mandatory thinking mode handling and budget level resolution (`Minimal`, `Low`, `Medium`, `High`).
3. **OAuth Provider Registry** (`oauth_registry_service.rs`):
   - Strict loopback callback host verification (`localhost`, `127.0.0.1`, `::1` only).
   - PKCE SHA-256 base64url challenge computation according to RFC 7636.
   - Credentials management with deterministic expiry verification.

## 4. Contract Tests & Verification
- Test harness: `source/host/tests/openclaw_llm_contract.rs`
  - `test_ocrm_llm_rate_limit_priority_and_admission`
  - `test_ocrm_llm_terminal_error_and_secret_redaction`
  - `test_ocrm_llm_stream_wrappers_cache_and_tools`
  - `test_ocrm_llm_oauth_registry_and_pkce`
- Evidence: 4/4 passing contract tests with positive and negative cases.
