# Remote Computer mobile session create reconciliation

Status: active implementation spec  
Authority: latest explicit Fabushi Desktop → Android parity requirement, then current Desktop main source.

## Context

The current Remote Computer control-plane creates a fresh control session and short-lived mobile token for every `POST /v1/computers/:device_id/sessions`. If the server commits creation but the mobile client loses the HTTP response, retrying create can produce a second live session. Android process-death recovery therefore cannot safely distinguish “not created” from “created but response lost”.

## Goal

Make mobile control-session creation durable and reconcilable without exposing the desktop `deviceSecret`, weakening account/client fences, or reusing a Remote Computer control credential as an Authorized Remote Runner bearer.

## Requirements

- **RCS-01 Stable request identity.** Mobile create MUST include a bounded opaque `requestId`. The server persists it with the created session and treats the tuple account + device + paired client + requestId as one creation identity.
- **RCS-02 Idempotent duplicate create.** Repeating an identical live request returns the same durable `sessionId` and same session-scoped mobile credential; it MUST NOT create a second session.
- **RCS-03 Concurrent create.** Concurrent requests with the same identity converge through a database uniqueness constraint. Different request IDs remain subject to normal per-client/per-device limits.
- **RCS-04 Lost-response recovery.** A client-authenticated reconcile endpoint MUST accept device + clientId + clientToken + requestId and recover the original live session after create response loss or mobile process death.
- **RCS-05 Fencing.** Reconcile/create MUST be fenced by authenticated account, exact device, exact paired client and the current client credential generation/hash. Cross-account, cross-device, cross-client or rotated/revoked client credentials fail closed.
- **RCS-06 Expiry/revocation.** Closed/expired sessions and sessions whose paired client/computer has been revoked are not recoverable as live control sessions. Expired rows are closed before reconciliation.
- **RCS-07 Token secrecy and rotation.** `deviceSecret` is desktop-only. The mobile token remains session-scoped and is never a Runner bearer. Its durable recovery material MUST NOT persist the raw token in D1. Rotating the paired client token invalidates recovery and use of sessions created under the previous client credential.
- **RCS-08 No credential projection expansion.** The API may return `mobileToken` only to an authenticated currently paired mobile client for its own request identity. It MUST NOT return `deviceSecret`, TURN secrets outside the existing ICE response contract, or Remote Runner credentials.
- **RCS-09 Existing transport semantics preserved.** Activation, transport route locking, signal authorization, grants, expiry and close behavior remain fail-closed.
- **RCS-10 Android compatibility.** The contract is designed for Android to persist one requestId before create, mark outcome-unknown on ambiguous delivery, and reconcile that same identity after reconnect/process death rather than replaying create with a new identity.

## Contract

Create request:

```json
{
  "clientId": "paired-client-id",
  "clientToken": "paired-client-secret",
  "requestId": "stable-random-mobile-request-id"
}
```

Reconcile:

`POST /v1/computers/:device_id/sessions/reconcile`

with the same three fields. Success returns the same public create payload shape, including the original `sessionId`, regenerated session-scoped `mobileToken`, current state/expiry/permissions and ICE servers. The server derives the recoverable mobile token from domain-separated one-way material that includes the current paired client secret, requestId and durable session identity; D1 stores only the token hash plus the paired-client-token hash captured at creation.

## Data model

Add to `remote_computer_sessions`:

- `mobile_request_id TEXT`
- `mobile_request_hash TEXT`
- `client_token_hash_at_create TEXT`

Backfill is intentionally absent for legacy sessions: they remain usable through their existing lifetime but are not mobile-reconcilable. New create requests require the fields. Add a unique index on `(user_id, device_id, client_id, mobile_request_id)` for non-null request identities.

## Failure behavior

- malformed requestId → 400;
- stale/revoked/rotated client credential → 403;
- same request identity with nonmatching request hash → 409;
- reconcile identity not found → 404;
- expired/closed reconcile → 410/409-style fail-closed response, without creating a replacement;
- database/create race → reload the unique identity and return it only if all fences match.

No timeout or network error is interpreted as “not executed”.

## Verification

GitHub Actions only. Required focused tests cover deterministic token recovery, duplicate request identity, mismatched identity, credential rotation, account/device/client separation, expiry/revoke decision logic, and concurrent-create uniqueness semantics. Existing worker/runtime tests must remain green.

## Acceptance

This change is complete only when a PR exact HEAD passes applicable GitHub Actions, is merged to Desktop `main`, and Android rebases its Desktop authority to that new exact SHA before claiming the new contract as parity evidence.
