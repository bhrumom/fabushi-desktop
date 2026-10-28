# Upstream WEB proxy plan complete read — strict index 121

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `docs/web-proxy-plan.md@9c805e32d33e8e1ac6b0417b26242b4a09198bd9` (34,990 bytes), fully read.
- Parent exact HEAD: `ea8cf7b5abfac5ec5ec59bb06495975a9b4aa3d3`, tree `5dca627d64a3b8d7285e0cdfe2905dd18f74b15c`.
- Parent TDRP `39953873302`: 61/61 success; authority `12500974554` sha256:25e993d6aa1ffb61e076963e558a34abecee61b0985dac2585808a1fd784ecb0; closure `12502420573` sha256:48c67de85f3e8068f1b1a07ba2f09c5f1eaeb52dfd79d2588e1ac469149fde28; reachability `12501563758` sha256:9d113c76cbe8cd220e5e740a63d6069a7f470cf37579c898982ecf3b0ee77b69.
- Parent Rust `39953873050`: 6/6 success; artifact `12502220382` sha256:6c06bfd0be89d9b000774e47724719923887873e811f9194a09375a9f78d3e44.
- Preflight: PR/ref `ea8cf7b5abfac5ec5ec59bb06495975a9b4aa3d3`/`5dca627d64a3b8d7285e0cdfe2905dd18f74b15c`; main `7394f6dc7687f9b986ee5589d40aa680ce23b90a`/`fbeb0bb32d944d0c1053880c471fe5c52279f052`; upstream dev `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`/`c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Responsibility decomposition

The WEB plan is transport/security authority, not presentation-only documentation. It defines one process-wide hidden native WebView carrier for WEB MTProxy, a worker transport, stream multiplexing, HELLO/WELCOME and OPEN/DATA/CLOSE/WINDOW framing, flow-control windows, queue/memory caps, health deadlines, reconnect/backoff, canonical host/base-path validation, explicit browser fallback, loopback WebSocket authentication, strict origin checks, derived HMAC capability handling, restricted WebView profiles and fail-closed hosted-relay prerequisites.

Applicable source-neutral responsibilities are bounded byte streams, lifecycle/reconnect, endpoint validation, TLS/WebSocket boundaries, policy before reachability, capability/secret separation, explicit fallback, account-generation fencing and real-service evidence. Telegram-specific MTProxy framing, Qt/lib_webview, WebView2/WKWebView/WebKitGTK, tg://webproxy and the Telegram relay are not promoted into Fabushi architecture.

## Existing-owner-first

- Rust Host + packaged fabushi-network-worker already own account/device network generation, private inherited stdio, encrypted worker state and bounded stream operations.
- core/network_proxy_loader + codex-rs/network-proxy already own HTTP/SOCKS configuration, trusted constraints, domain/local-target policy, upstream proxy selection, TLS roots, MITM boundary and blocked-request auditing.
- Coordinator gateway/http_transport owns HTTP/HTTPS endpoint parsing, timeouts and rustls trust.
- shared/node/egress-tunnel/websocket-client owns production ws@8 WebSocket creation.
- Current validators are network-worker protocol/encrypted-store tests, Host network-worker/enrollment contract tests, NetworkProxy loader/upstream/MITM tests and Coordinator gateway HTTPS tests.
- Recursive exact-head tree inspection found no Telegram-style web_proxy transport owner. Existing Computer VNC and WebAuthn WebViews are unrelated and rejected as generic network owners.

No TelegramWebProxy, TelegramRuntime, MTProto sidecar, second NetworkProxy, second permission authority or second account/session store is introduced.

## Security and service boundary

The raw proxy/account/device secret must never cross into renderer, hosted JavaScript, URL or log. Fallback remains user-explicit. Loopback/non-public targets remain policy-gated. TLS/WebSocket reachability never substitutes for application authorization. Buffers, windows, queues and timeouts remain bounded. The upstream plan itself requires a hosted relay before execution acceptance; Fabushi has no production owner for that Telegram relay and does not need one merely to absorb the responsibilities. Any future server-backed proxy must name a real Fabushi service owner and produce real E2E evidence; mocks cannot satisfy acceptance.

## Traceability anchors

- Requirement: `TDRP-R9-UPSTREAM-WEB-PROXY-NETWORK-SECURITY-PROVENANCE-001`.
- Oracles: `ORA-TDRP-WEB-PROXY-EXACT-SOURCE-001`, `ORA-TDRP-WEB-PROXY-EXISTING-NETWORK-OWNER-001`, `ORA-TDRP-WEB-PROXY-BOUNDS-001`, `ORA-TDRP-WEB-PROXY-SECURITY-BOUNDARY-001`, `ORA-TDRP-WEB-PROXY-SERVICE-FAIL-CLOSED-001`.
- Invariants: `INV-TDRP-WEB-PROXY-NO-TELEGRAM-RUNTIME-001`, `INV-TDRP-WEB-PROXY-NO-SECRET-TO-WEB-001`, `INV-TDRP-WEB-PROXY-EXPLICIT-FALLBACK-001`, `INV-TDRP-WEB-PROXY-BOUNDED-QUEUES-001`, `INV-TDRP-WEB-PROXY-POLICY-BEFORE-REACHABILITY-001`, `INV-TDRP-WEB-PROXY-ACTIONS-ONLY-001`.
- Contracts: `CONTRACT-TDRP-WEB-PROXY-EXACT-SOURCE-001`, `CONTRACT-TDRP-WEB-PROXY-EXISTING-OWNER-001`, `CONTRACT-TDRP-WEB-PROXY-SERVICE-EVIDENCE-001`.
- Temporal/fault: `TEMP-TDRP-WEB-PROXY-ACCOUNT-SWITCH-001`, `TEMP-TDRP-WEB-PROXY-RETRY-HEALTH-001`, `FAULT-TDRP-WEB-PROXY-TIMEOUT-OVERFLOW-001`, `FAULT-TDRP-WEB-PROXY-SERVICE-ABSENT-001`.
- Security: `SEC-TDRP-WEB-PROXY-NO-SECRET-EXPOSURE-001`, `SEC-TDRP-WEB-PROXY-POLICY-BEFORE-REACHABILITY-001`, `SEC-TDRP-WEB-PROXY-LOOPBACK-BOUNDARY-001`.
- Regression: `REG-TDRP-WEB-PROXY-NO-SECOND-NETWORK-ROOT-001`, `REG-TDRP-WEB-PROXY-BOT-NETWORK-NO-REGRESSION-001`.

## Fail-closed boundaries

No local/container/Mac/Windows build or test was run. Telegram MTProxy/lib_webview/relay is not retained as a Fabushi runtime. Transport does not equal authorization. Missing service/E2E evidence remains open. Any source/owner/validator/PR/main/upstream drift requires reconciliation. Recursive source closure and independent release acceptance remain open.

Closing strict121 yields unknown=16,255, unread=16,230, omitted=0, responsibilities=139. Non-truncated root order fixes strict122 as `docs/web-proxy-test-plan.md@aa5752bf7a9cf57bafd0489961f8fa49265d238b`. Fresh child exact-head TDRP/Rust Actions are required.
