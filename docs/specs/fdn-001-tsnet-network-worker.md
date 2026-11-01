# FDN-001 embedded tsnet network worker — Phase 2 slice

Status: active
Owner: Desktop Rust Host + packaged Go network worker
Last updated: 2026-10-14
Parent authority: docs/specs/fabushi-same-account-device-network.md (FDN-001)

## Purpose

Introduce the first production Phase 2 networking slice without creating a second Fabushi account, device, permission, Remote Computer, or MCP authority.

This slice embeds Tailscale through a separately packaged Go worker supervised by the existing Rust Host process lifecycle. The worker owns network-node lifecycle, encrypted Tailscale state, network status, shutdown, and bounded private byte-stream transport. It does not decide whether an MCP/device tool may execute.

## Pinned upstreams

- Tailscale: `tailscale.com v1.104.1`, released 2026-10-07, BSD-3-Clause.
- Tailscale source authority for this adoption: `tsnet` from tag `v1.104.1`.
- Headscale candidate for the later coordination-service slice: `v0.29.5`, released 2026-10-13, BSD-3-Clause.

Headscale is not deployed by this slice. Production coordination remains blocked until account-to-node binding, default-deny policy and cross-account isolation have dedicated integration evidence.

## Architecture

```text
Fabushi account / Remote Computer / authorization authorities
                         |
                    Rust Host
                         |
             private inherited stdio
                         |
              fabushi-network-worker
                         |
                 tailscale.com/tsnet
```

The worker is a sibling packaged binary named `fabushi-network-worker` (or `.exe` on Windows). Rust Host starts it, sends initialization over stdin, waits for a structured ready event on stdout, keeps stdin open as a parent-lifetime/control pipe, and terminates the child on Host shutdown or generation replacement.

The macOS release workflow builds the pinned Go worker from the exact package source
with Go 1.27.1, stages it beside `mahayana-app-host` in `Contents/Resources/bin`,
and relies on the existing Electron signed-resource path to sign and notarize the
same sibling executable. Packaging must fail when the worker is absent; a Host-only
candidate is not a valid FDN package.

No auth key or state-encryption key may appear in process argv, environment variables, user-visible status, or worker events.

## Upstream behavior intentionally disabled

- no full Tailscale GUI or CLI;
- no separate tailscaled service;
- no system TUN/VPN route ownership;
- no exit node;
- no DNS/system route mutation;
- no Funnel;
- no web client;
- upstream log upload disabled through `envknob.SetNoLogsNoSupport()`.

## Worker initialization contract

The first stdin frame is one newline-delimited JSON object:

```json
{
  "type": "initialize",
  "protocolVersion": 1,
  "hostname": "opaque-network-hostname",
  "controlUrl": "https://coord.example.invalid",
  "authKey": "short-lived-enrollment-secret-or-empty-for-restored-state",
  "stateDir": "/private/fabushi/path",
  "stateKeyB64": "32-byte-base64-key"
}
```

The worker validates bounded hostname/control URL/state path/key shape. `controlUrl` must be HTTPS. `authKey` is optional after a durable node state exists; it is an enrollment credential, not a Fabushi login token. Every newline-delimited control frame is bounded to 64 KiB.

After initialization the worker accepts:
- `{"type":"status","requestId":"..."}`
- `{"type":"shutdown","requestId":"..."}`
- bounded `streamDial`, `streamListen`, `streamWrite`, `streamRead`, `streamCancel`, and `streamClose` requests for private byte streams.

These stream operations are transport-only. The worker restricts peer addresses to Tailscale address ranges and bounds ports, chunks, reads, timeouts, and concurrent stream sessions; listeners verify the expected remote peer. The worker does not authenticate a Fabushi account/client/session or authorize a tool. The Rust Host's existing `RemoteComputerTarget` owner must validate the current device/network binding and generation, session/client grant, request digest, and operation claim before any native side effect. Network reachability never substitutes for application authorization. No arbitrary URL, shell, or generic tool dispatch is exposed.

## State store

The worker supplies a custom concurrent `ipn.StateStore` to tsnet:
- AES-256-GCM at rest;
- fresh random nonce per write;
- state key name used as AEAD additional authenticated data;
- file name is SHA-256 of the state key, not the plaintext state key;
- private directory and file permissions where supported;
- temporary-file + rename replacement;
- `nil` writes delete the state;
- plaintext node/machine state is never persisted by Fabushi.

The 32-byte state encryption key is supplied by Host over inherited stdin. The eventual production owner for that key is Fabushi Host secret storage; this slice does not invent a second secret store.

## Lifecycle and failure behavior

- worker exits if initialization is missing/invalid;
- worker exits when parent stdin reaches EOF;
- a malformed frame with a recoverable newline boundary emits one generic request error and may continue;
- an oversized frame or terminal stdin scanner/read error emits one generic request error and exits; it must not retry an exhausted scanner or enter an unbounded error loop;
- Host rejects a worker that exits before a `ready` event;
- Host has bounded startup/shutdown deadlines and force-terminates a stuck child;
- status failures are returned as typed worker errors;
- account switch must stop the old worker before another account generation starts;
- no worker event may echo enrollment/state secrets.

## Verification

All executable checks run in GitHub Actions only.

Required for this slice:
1. Go unit tests for encrypted state round-trip, no plaintext persistence, tamper rejection and deletion.
2. Go unit tests for protocol validation/redaction and bounded control-frame handling, including recoverable malformed frames and terminal oversized-frame behavior.
3. Go compile/test on Linux, macOS and Windows using Go 1.27.1.
4. Rust Host contract tests for sibling-binary resolution, secret-free process command construction and parent-lifetime protocol.
5. Exact-head evidence for the PR and post-merge main before this slice is marked passed.
6. The signed macOS candidate contains the executable sibling expected by
   `resolve_network_worker_executable`, and its packaged acceptance records that
   resource before any production release promotion.

## Non-goals / still pending

This slice does not satisfy G02/G03/G05/G16 by itself. Still pending:
- one-time Fabushi-issued network enrollment tokens;
- self-hosted coordination deployment;
- account-scoped node projection and default-deny ACLs;
- real WireGuard direct-path evidence;
- DERP evidence;
- authenticated reverse-WebSocket fallback;
- device byte-stream gateway;
- app-layer Host authorization around every remote tool call;
- packaging/signing/update/rollback acceptance.

## Acceptance record

| Item | Status | Evidence |
| --- | --- | --- |
| fixed Tailscale version + license | implemented | v1.104.1 / BSD-3-Clause |
| Go tsnet worker lifecycle | pending | exact-head Actions required |
| encrypted tsnet StateStore | pending | exact-head Actions required |
| Host-supervised private stdio | pending | exact-head Actions required |
| signed macOS worker sibling staged beside Host | pending | exact-head package Actions required |
| real private-network connectivity | pending | later integration slice |
| cross-account isolation | pending | later coordination/gateway slice |
| FDN-A01..A16 | pending | full program acceptance required |
