# MTProto instance, resolver and socket responsibilities 5241-5260 complete read

Status: exact-source read complete; platform/protocol replacement mapping; no unknown closure.
Accepted upstream: `42f8a36d43b8c805bc821905bea4cfeb3af1d41d`

Orders 5241-5260 were read directly from the accepted tree. This range completes the Dcenter declaration and covers multi-provider domain resolution, bounded diagnostic dumping, duplicate received-id tracking, RSA/request serialization helpers, raw/TLS-obfuscated/web-proxy sockets, the MTProto facade and the 2,224-line Instance protocol authority.

The most important product responsibility is in `mtp_instance.cpp`: one authority owns config/proxy/DC/session/key state and request registration, callback correlation, after/dependency relationships, cancellation, delayed retry, DC migration, auth import/export, session restart, logout and key destruction. Fabushi must preserve equivalent single-owner request/session settlement and secure credential lifecycle in its existing Coordinator/Host/backend architecture. It must not gain a second Telegram protocol runtime.

DomainResolver adds bounded racing of multiple DoH providers with IPv4/IPv6 cache expiry and cancellation. ReceivedIdsManager provides a bounded replay/idempotency window. TCP/TLS/web-proxy files define transport-state and stale-callback/flow-control responsibilities. RSA, serialized-request and DC-shift mechanics are MTProto-specific details; only their security/correlation responsibilities are applicable to Fabushi replacement owners.

Accounting after this exact read: recursive 16,120; read-through 5,260; unread 10,860; unknown 15,841; omitted 0. No unknown, verification, baseline-ready or release credit is granted by reading.
