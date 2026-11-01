# FDN-001 Desktop Host target-side Core owner

Status: implementation in progress.

The Desktop Host extends the existing Fabushi product-auth boundary rather than creating a second account or credential lifecycle.

## Credential boundary

- `FABUSHI_HOST_ACCESS_CREDENTIAL_FILE` is written atomically by the existing Fabushi account owner.
- Host reads it only through an internal first-party Core credential provider.
- The product credential is never a Cursor/Sand renewal credential and is never exposed to renderer, tool descriptions, or logs.
- Reads are on-demand so account switch, logout, refresh, and file replacement fence stale account identity without a Host-side token cache.
- Malformed, expired, missing, or unsupported credential records fail closed.

## Target operation authority

Remote Computer target execution is authorized by Platform Core using the current Fabushi account plus the registered device secret. The official MCP controller credential is not target authority.

The target path must use the existing Core session activation, exact-tool grant/revoke, operation claim/result, network-binding generation/revoke, and durable operation journal contracts. Transport only delivers a bounded operation envelope to the target; it does not decide authorization.

Before native execution, Host must correlate account, device, session, requestId, requestSha256, exact tool grant generation, binding generation, expiry and revocation. Network timeout, pipe loss, or lost result acknowledgement after a side effect is `outcome_unknown`; Host must not automatically repeat side-effecting actions.

## Network and executor boundary

The target path reuses `network_worker_process` bounded peer streams and the existing native Computer/browser/file/terminal owners. No second registry, device database, or executor is introduced.

Tests and builds run only in GitHub Actions.
