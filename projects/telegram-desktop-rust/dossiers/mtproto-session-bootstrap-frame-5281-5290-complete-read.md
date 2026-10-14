# MTProto session / bootstrap / frame 5281-5290 complete read

Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d` (tree `6ac9bbc1b44edcb119b1a724e7b0a321c3b7b8fa`)

Recursive orders 5281-5290 were read directly from exact upstream blobs. Credit is limited to source read-through and responsibility decomposition. `unknown` is intentionally unchanged and `omitted=0`.

## Responsibilities
- **5281 — `Telegram/SourceFiles/mtproto/sender.h` @ `5330a0559cff9c5693e71b7cda938a30e15ce305`**: typed request builder plus RAII cancellation, explicit handled settlement, dependency/delay/override identity and retry/flood failure policy.
- **5282 — `Telegram/SourceFiles/mtproto/session.cpp` @ `40d7048c3057c7c5523b786048670524c8bebe4c`**: session owner bridging thread-safe queued transport state to main-thread lifecycle: option refresh, restart/stop/kill, key creation, receive/send scheduling and connection state.
- **5283 — `Telegram/SourceFiles/mtproto/session.h` @ `d6f197d233fa4dcc6d2b19745e6d2787c5227911`**: typed SessionData/Session ownership for queued sends, received responses, connection/key lifecycle, request cancellation/state and serialized cross-thread access.
- **5284 — `Telegram/SourceFiles/mtproto/session_private.cpp` @ `b8be9bbdda1b788c4ab118f91d23cd4feeb82aec`**: transport session state machine: connection candidate selection, exponential retry, ping/timeouts, ack/state/resend/replay handling, bounded container retention, auth-key/session reset and stale connection cleanup.
- **5285 — `Telegram/SourceFiles/mtproto/session_private.h` @ `62b01e38dcb21832125b67dcd35fba3194a26829`**: typed private session state for retry timers, connection generations, sent/acked/resending identities, ping/session/key state and deterministic teardown.
- **5286 — `Telegram/SourceFiles/mtproto/special_config_request.cpp` @ `fdb65506b0a886bc2daeccb61090bc44be7e2d88`**: bounded multi-source bootstrap/config discovery with staggered requests, HTTPS-only providers, response parsing/decryption, server-time synchronization and successful-endpoint publication.
- **5287 — `Telegram/SourceFiles/mtproto/special_config_request.h` @ `91901489e778089b03187682a9556e929867b94f`**: typed bootstrap attempt/request ownership and completion callbacks.
- **5288 — `Telegram/SourceFiles/mtproto/type_utils.h` @ `fd6cdd1e5631f7a7f9690c1ffa8cb438ba6b8036`**: source-specific TL boolean conversion helpers with no independent product state owner.
- **5289 — `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_frame.cpp` @ `bd1d7b310f40203f9da914efc8bdd6711035884b`**: bounded multiplexed proxy frame serialization/parser: known type validation, 24-bit stream identity, payload cap, batch cap and partial-buffer retention.
- **5290 — `Telegram/SourceFiles/mtproto/web_proxy/web_proxy_frame.h` @ `a5ba491fed67d70aee88e6ec2e3ac47c991e78f3`**: typed web-proxy frame/window protocol contract and hard resource limits.

## Canonical Fabushi mapping

The applicable lifecycle belongs to existing Fabushi Coordinator/Host/backend transport, request settlement/idempotency, reconnect/retry, bootstrap/config and network-security owners. The strongest requirements in this batch are deterministic request cancellation, stale-generation fencing, bounded retry/timeouts, replay/duplicate protection, correlated error settlement, bounded buffers and bootstrap fail-closed validation. Telegram MTProto session/frame/provider machinery is not a new Fabushi owner.

Order 5288 is a protocol helper, but the current completion rule still prevents reducing `unknown` from read-only evidence. Required closure remains production-owner audit/implementation, focused fault-recovery and temporal tests, exact-head Actions, packaged acceptance and independent release review.
