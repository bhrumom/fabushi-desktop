# MTProto transport and auth 5219-5240 complete read

Status: exact-source read complete; platform/protocol replacement mapping only; no unknown closure.
Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`

Orders 5219-5240 were read directly from accepted upstream exact blobs. The range covers DC/config failover, abstract/HTTP/TCP/custom-resolving transports, core protocol identifiers, dedicated file transfer, socket abstractions, auth-key creation/binding and per-DC key ownership.

The applicable responsibilities are not "ship MTProto inside Fabushi". They are:
- resilient endpoint/config acquisition and transport/proxy/DNS selection with bounded timeout/cancellation and stale-result fencing;
- one authoritative connection/request lifecycle with granular errors and deterministic teardown/reconnect;
- bounded file-transfer chunk/progress/retry/cancel behavior;
- typed request/session/message correlation;
- secure credential/key/session creation, exclusivity, server validation, invalidation and secret cleanup.

Fabushi uses Coordinator/Host/backend protocols and Electron/Node/Rust networking, so Qt/MTProto framing, HTTP port behavior, obfuscation and RSA/DH auth-key algorithms are protocol-specific replacement details. Their product/security responsibilities remain mapped-open until equivalent current Fabushi owners and exact-head tests/evidence are audited. No second Telegram protocol stack is introduced.

Accounting after this direct read: recursive 16,120; read-through 5,240; unread 10,880; unknown 15,841; omitted 0. `baseline_ready=false`; `acceptance.accepted=false`.
