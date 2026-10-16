# FDN-001 embedded tsnet network worker — Phase 2 slice

Status: active
Owner: Desktop Rust Host + packaged Go network worker
Last updated: 2026-10-14
Parent authority: docs/specs/fabushi-same-account-device-network.md (FDN-001)

## Purpose

Introduce the first production Phase 2 networking slice without creating a second Fabushi account, device, permission, Remote Computer, or MCP authority.

This slice embeds Tailscale through a separately packaged Go worker supervised by the existing Rust Host process lifecycle. The worker owns only network-node lifecycle, encrypted Tailscale state, network status, and shutdown. It does not decide whether an MCP/device tool may execute.

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

The worker validates bounded hostname/control URL/state path/key shape. `controlUrl` must be HTTPS. `authKey` is optional after a durable node state exists; it is an enrollment credential, not a Fabushi login token.

After initialization the worker accepts only:
- `{"type":"status","requestId":"..."}`
- `{"type":"shutdown","requestId":"..."}`

This initial slice deliberately does not expose arbitrary dial/listen requests. A later FDN slice must add byte-stream transport only after application-layer session/authorization verification is wired.

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
- Host rejects a worker that exits before a `ready` event;
- Host has bounded startup/shutdown deadlines and force-terminates a stuck child;
- status failures are returned as typed worker errors;
- account switch must stop the old worker before another account generation starts;
- no worker event may echo enrollment/state secrets.

## Verification

All executable checks run in GitHub Actions only.

Required for this slice:
1. Go unit tests for encrypted state round-trip, no plaintext persistence, tamper rejection and deletion.
2. Go unit tests for protocol validation/redaction.
3. Go compile/test on Linux, macOS and Windows using Go 1.27.1.
4. Rust Host contract tests for sibling-binary resolution, secret-free process command construction and parent-lifetime protocol.
5. Exact-head evidence for the PR and post-merge main before this slice is marked passed.

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
| real private-network connectivity | pending | later integration slice |
| cross-account isolation | pending | later coordination/gateway slice |
| FDN-A01..A16 | pending | full program acceptance required |
