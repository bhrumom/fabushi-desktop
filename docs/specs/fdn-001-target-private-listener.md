# FDN-001 Desktop target private listener

Status: implementation in progress
Parent: `docs/specs/fabushi-same-account-device-network.md`
Depends on:
- Desktop Host network enrollment owner
- Platform Core target network ingress peer projection
- existing Remote Computer target session/tool/operation contracts

## Goal

Serve the existing Remote Computer target protocol on the same embedded tsnet worker that owns this Desktop device's account-scoped network identity, while preserving exact-peer routing and all application-layer permission fences.

## Authority boundaries

- Fabushi account credential remains owned by `FabushiCoreCredentialProvider`.
- Remote Computer deviceSecret remains owned by `RemoteComputerDeviceSecretProvider`.
- The target asks Core for `official-device-mcp` ingress projection using ordinary account + matching deviceSecret.
- Core ingress response is routing authority only. It never authorizes a tool or operation.
- The target independently verifies its current network binding, cell, target generation, expiry, and active state before accepting the projected peer.
- Only Tailscale CGNAT or Tailscale ULA node addresses are accepted as listener peers.
- The existing network worker still receives an exact `expectedPeer`; wildcard/all-tailnet listeners are forbidden.

## Product permission boundary

Network availability MUST NOT synthesize remote-control permission.

The private target service requires explicit injected implementations of:
- `RemoteComputerToolCatalog`
- `RemoteComputerToolAuthorizer`
- `RemoteComputerNativeExecutor`

There is deliberately no allow-all authorizer. Shipping wiring is not complete until those dependencies are connected to existing Fabushi Host/Coordinator product permission owners and native executors.

## Shared worker lifecycle

The network enrollment supervisor owns one `OwnedNetworkWorker` for the current Fabushi account/device identity. An optional `NetworkWorkerService` may poll that same worker; it may not spawn a second tsnet identity.

On account switch/logout:
1. current identity changes or becomes unavailable;
2. the existing shared worker is closed;
3. the previous service can no longer listen on that identity;
4. a new worker can be reconciled only from the new account/device authority.

The service listener uses a bounded one-second accept timeout and a short supervisor idle interval so shutdown/account fencing remains responsive. Ingress projection is cached only briefly and is refreshed before expiry.

## Request safety

Each accepted private request still performs existing target checks:
- current network generation before controller/tool/session action;
- service connection/delegation binding;
- explicit local tool authorization;
- session activation and Core exact-tool grant;
- stable requestId + requestSha256;
- Core operation claim;
- second current-network-generation fence immediately before native side effect;
- exactly one native executor dispatch;
- result digest correlation;
- `outcome_unknown` on uncertain result persistence/transport.

No side-effecting operation may be replayed automatically because a private stream timed out or closed.

## Verification

GitHub Actions only. Exact-head Rust Host and FDN network-worker workflows must pass. Focused source contracts lock:
- Core ingress path/body/response identity;
- same-cell/current-generation correlation;
- private tailnet peer validation;
- exact `stream_listen(expectedPeer,...)`;
- no wildcard peer;
- explicit catalog/authorizer/executor injection;
- same-worker lifecycle and account-switch close.

## Not yet acceptance-complete

This slice alone does not satisfy G05/G09/G10/G16. Shipping Host still needs the existing product permission owner and native executor dependencies wired into the service; production controller worker enrollment/binding and real direct/DERP/WSS calls still require true network-runner evidence.
