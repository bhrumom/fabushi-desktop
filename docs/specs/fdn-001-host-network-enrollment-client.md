# FDN-001 Host target network enrollment client

Status: implementation slice.
Owner: shipping Desktop Host / Remote Computer target authority.
Parent: FDN-001.
Dependencies: Platform Core one-time enrollment grants and Backend account-bound enrollment proxy.

## Boundary

Only the Host may hold Remote Computer deviceSecret, Core grantSecret or Headscale pre-auth AuthKey. Renderer, Cursor/Sand renewers, tool descriptions, ordinary logs and official MCP controller credentials never receive those values.

The existing first-party Fabushi Core credential provider supplies the account bearer and device identity. The existing Remote Computer device-secret provider supplies target device authority.

## Flow

1. Host issues a Core network enrollment grant with a stable bounded requestId. Repeating that requestId uses Core reconciliation rather than creating unrelated generations.
2. Host calls the Backend product enrollment proxy with only grantId/grantSecret and the ordinary first-party Fabushi account bearer.
3. Host validates grantId, deviceId, cellId and generation against its Core projection before accepting the returned Headscale AuthKey.
4. A later slice supplies that AuthKey and the configured Headscale ControlURL to the existing supervised network worker.
5. After worker registration, Host calls enrollment complete and accepts only the same grant/device/generation plus completed=true.
6. Host reads the Core network binding before accepting any private Remote Computer request.
7. If enrollment must be abandoned before redemption, Host can revoke the Core grant with target device authority.

## Secrets

grantSecret and AuthKey fields are private and Debug-redacted. They are not serializable public model fields. Network worker secrets continue to cross the process boundary only through the existing private stdin initialization contract.

## Verification

GitHub Actions only. Rust compilation plus focused source/contract tests lock endpoints, first-party authority dependencies, identity matching and secret redaction. Shipping worker lifecycle, restart recovery and actual Headscale registration are intentionally not claimed by this slice.
