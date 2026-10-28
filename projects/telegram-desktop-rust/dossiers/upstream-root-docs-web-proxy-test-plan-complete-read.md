# Upstream WEB proxy execution test plan complete read — strict index 122

- Upstream: `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`, tree `c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.
- Source: `docs/web-proxy-test-plan.md@aa5752bf7a9cf57bafd0489961f8fa49265d238b` (25,526 bytes), fully read.
- Parent exact HEAD: `417a226a88897ddc3dd88f21d7dcbf854f40eaa2`, tree `58c4e60b0ca66cefb5b007c6c1c0e001290a8e8d`.
- Parent TDRP `39955991807`: 61/61 success; authority `12502486893` sha256:c25dbc800732765a6b1a4f7e17a578168ecae00fda6785b9d2c3fbf5930859f8; closure `12502289686` sha256:ec43cd8420f34a5b63abd308e190d0077e4499a8f8864c9d3ddfbd110477c298; reachability `12502792223` sha256:43f80bc02036c519e87e136b52f675f2fafceb4734f3c7b657a6faff819ceb76.
- Parent Rust `39955991731`: 6/6 success; artifact `12501879825` sha256:e305cbf47b8cd93a513edba421e4e2e8c13711147cad4ea98664d6f1b52dfe0d.
- Preflight: PR/ref `417a226a88897ddc3dd88f21d7dcbf854f40eaa2`/`58c4e60b0ca66cefb5b007c6c1c0e001290a8e8d`; live main `7394f6dc7687f9b986ee5589d40aa680ce23b90a`/`fbeb0bb32d944d0c1053880c471fe5c52279f052`; upstream dev `a7f453612f4896ac04d0007a36ab0d377e5bcbcf`/`c539eabfb05e7dcaa84821ac44c35cdfb6551ae5`.

## Responsibility decomposition

The document is a server-ready release/validation authority for Telegram Desktop WEB proxy. It requires compatible real relay inputs, exact candidate identity, secret-safe logs, endpoint/TLS preflight, invalid-input rejection, network-origin ownership, functional traffic, large-transfer hashes and memory/throughput observation, multiplexing/concurrency, destruction races, lifecycle/reconnect, server-fault recovery, loopback WebSocket negative tests, persistence/compatibility, Windows/macOS/Linux WebView/browser matrices, an unreliable-network field hypothesis and deployment-boundary checks.

Applicable source-neutral responsibilities are exact-candidate binding, fail-fast service compatibility, bounded sessions/chunks/timeouts, secret redaction, policy before connection, TLS trust, account-generation fencing, lifecycle/fault recovery, cancellation/destruction safety, real-service evidence and release fail-closed semantics. Telegram WEB relay, stock MTProxy, hidden WebView carriers, browser fallback protocol and Telegram-specific field hypothesis are not imported into Fabushi.

## Existing-owner / validator uniqueness

Exact-head owners remain the strict121 set: Host/network-worker generation and bounded streams, Rust NetworkProxy/NetworkPolicy, Coordinator HTTPS/rustls, and egress WebSocket. The current validators were re-read at `417a226a88897ddc3dd88f21d7dcbf854f40eaa2`:

- `source/fabushi-network-worker/protocol_test.go@1b6115771bd8922c828d474a47c161680eded3fd`: unsafe control URL/key rejection, secret redaction, bounded request/session/stream/chunk/timeout handling.
- `source/host/tests/network_worker_process_contract.rs@9d9a1101949092f49e7774a2bdb718b6676abbec`: private secret handoff and supervised worker lifecycle.
- `source/host/tests/network_enrollment_owner_contract.rs@087f3831f946f8bad4360b7cb63f29dee787de75`: first-party authority uniqueness plus account-switch/logout fencing.
- `source/mahayana/codex-rs/core/src/network_proxy_loader_tests.rs@f2758e8946132c6899996d190ea3e482883896bc`: profile/policy precedence and network-proxy constraints.
- `source/mahayana/codex-rs/network-proxy/src/upstream_tests.rs@85f5da9017b708dda5728ab6daa8975d78061409`: rustls/custom-CA upstream behavior.
- `source/node-agent-coordinator/tests/gateway_https.rs@6fc868c108d5600df239f4572d5cf438e24e5120`: real HTTPS command/health/SSE transport.

No second network root, Telegram proxy harness, alternate permission authority or fake relay is warranted.

## Evidence boundary

The parent exact-head Actions prove current Fabushi validators and Bot/network regressions remain green. They do **not** execute the upstream Telegram WEB P0/P1 relay/WebView/browser/field matrix. A mock relay, local fixture or source-only checklist cannot upgrade those cases to passing production evidence. Any future Fabushi service-backed proxy must name a real service owner, bind immutable candidate/service versions, keep secrets/payloads out of evidence, and run real E2E on the supported release matrix.

## Traceability anchors

- Requirement: `TDRP-R9-UPSTREAM-WEB-PROXY-EXECUTION-TEST-PROVENANCE-001`.
- Oracles: `ORA-TDRP-WEB-PROXY-TEST-EXACT-SOURCE-001`, `ORA-TDRP-WEB-PROXY-TEST-EXISTING-VALIDATORS-001`, `ORA-TDRP-WEB-PROXY-TEST-SECRET-SAFE-EVIDENCE-001`, `ORA-TDRP-WEB-PROXY-TEST-REAL-SERVICE-FAIL-CLOSED-001`, `ORA-TDRP-WEB-PROXY-TEST-PLATFORM-FIELD-EVIDENCE-001`.
- Invariants: `INV-TDRP-WEB-PROXY-TEST-NO-FAKE-RELAY-001`, `INV-TDRP-WEB-PROXY-TEST-NO-SECRET-LOGGING-001`, `INV-TDRP-WEB-PROXY-TEST-POLICY-BEFORE-CONNECT-001`, `INV-TDRP-WEB-PROXY-TEST-BOUNDED-STATE-001`, `INV-TDRP-WEB-PROXY-TEST-UNEXECUTED-NOT-PASS-001`, `INV-TDRP-WEB-PROXY-TEST-ACTIONS-ONLY-001` (no fake relay, no secret logging, policy before connect, bounded state, unexecuted is not pass, exact-head Actions only).
- Fault/temporal: account switch, timeout/overflow, malformed transport, retry/recovery, idle/restart, cancellation/destruction.
- Security: TLS trust, secret-safe evidence, non-public-target policy, loopback protocol retained only as upstream provenance unless independently applicable.
- Regression: no second network root; current Fabushi Bot/network capability must not regress.

## Fail-closed boundaries

No local/container/Mac/Windows build or test was run. No Telegram relay, MTProxy, WebView or browser tab was deployed or exercised. The upstream P0/P1 matrix is therefore not claimed as passed. Recursive source closure and independent release acceptance remain open.

Closing strict122 yields unknown=16,254, unread=16,229, omitted=0, responsibilities=140. Next strict123 is `lib/xdg/org.telegram.desktop.desktop@573648491b97f9b1eb460a4c8add4ddf17ce171a` (558 bytes). Fresh child exact-head TDRP/Rust Actions are required.
